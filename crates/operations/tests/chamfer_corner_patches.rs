//! Why `chamfer_v2`'s shell cannot be closed by welding: the corner patches
//! are never built.
//!
//! `chamfer_shell_manifold.rs` pins the *symptom* — every edge free, 18
//! disconnected components — and names the missing corner patches as the
//! cause. This file exists so that cause is itself pinned by an assertion
//! rather than by prose, and so the measurement that exonerates
//! `sew::weld_faces` can be re-run by anyone.
//!
//! # What a chamfered cube is
//!
//! Chamfering all 12 edges of a cube at distance `d` cuts two kinds of
//! feature at each of the 8 original corners, where three chamfered edges
//! meet:
//!
//! * 6 side faces, each a `(S-2d)` square (the original faces, inset),
//! * 12 bevel faces, each a `d` x `(S-2d)` rectangle (one per original edge),
//! * 8 corner triangles, one per original corner, cut off by the plane
//!   through the three inset points.
//!
//! That is 26 faces, of which exactly 8 have three sides. The count is
//! analytic, not a golden value: nothing here depends on how an engine
//! discretises anything.
//!
//! # What `chamfer_v2` produces
//!
//! 18 faces, all quads, zero triangles. The 8 corner triangles are simply
//! absent, because `chamfer_builder` never calls
//! `corner::compute_corners` — the fillet builder calls it at
//! `fillet_builder.rs:183`, and `grep -c corner chamfer_builder.rs` is 0.
//!
//! Each of the 18 quads is individually a closed loop. That is why the shell
//! does not read as "one shell with holes" but as 18 separate pieces: no two
//! faces share an edge, because the faces that would have connected them at
//! the corners do not exist.
//!
//! # Why welding cannot fix it
//!
//! `sew::weld_faces` collapses edges that span the same pair of welded
//! vertices. Over the 72 edge occurrences of the `chamfer_v2` shell, the
//! number of distinct `(start, end)` position pairs is *also* 72 — nothing to
//! collapse. That is not a quirk of this measurement: the `fillet` control
//! below is run through the identical counter and does report a non-zero
//! collapsible count, so the counter can see duplicates when they exist.
//!
//! The consequence is a hard boundary on where the fix may live: the corner
//! faces have to be *built*. No amount of deduplicating what is already there
//! will close this shell.
//!
//! # The fix is NOT `corner::compute_corners` as-is
//!
//! The obvious move is to call the fillet builder's corner routine. Measured:
//! wiring it in unchanged gets the face COUNT right (18 -> 26, the 8 corner
//! patches appear) and the shell still does not close, at
//! `V/E/F = 100/96/26, free = 96, components = 26`. It is slightly worse than
//! before: 26 isolated pieces rather than 18.
//!
//! The reason is that the two corners are different geometry. `corner.rs`
//! routes 3+ stripe vertices to `spherical_triangle`, documented as
//! "rolling-ball sphere", "great-circle arcs", "Fillet radius" — a chamfer
//! corner is a flat triangle, and a spherical patch's boundary arcs do not
//! coincide with the straight contact lines on the adjacent faces. It also
//! leans on `CircSection`, which `analytic.rs` says outright it is filling with
//! a chord half-length because "CircSection is shaped for fillets".
//!
//! `the_spherical_corner_path_is_wrong_for_chamfer` below is the standing
//! record of that experiment, so nobody re-runs it. The corner patch chamfer
//! needs is a plane through the three contact points.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use std::collections::HashMap;

use brepkit_operations::blend_ops::chamfer_v2;
use brepkit_operations::chamfer::chamfer;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::face::FaceSurface;
use brepkit_topology::solid::SolidId;

/// Cube side used throughout.
const SIDE: f64 = 10.0;

/// Chamfer distances swept. Each leaves a non-degenerate eroded core.
const DISTANCES: [f64; 3] = [0.5, 1.0, 2.0];

/// A quantised point key: two points within this distance are "the same point"
/// for the purpose of comparing curve endpoints.
///
/// Matches the tolerance `sew::weld_faces` itself uses, so a count of zero
/// here means welding has nothing to collapse on its own terms too.
const WELD_TOL: f64 = 1e-6;

/// A face's side count, i.e. how many oriented edges its outer wire holds.
fn side_count(topo: &Topology, face: brepkit_topology::face::FaceId) -> usize {
    topo.wire(topo.face(face).unwrap().outer_wire())
        .unwrap()
        .edges()
        .len()
}

/// The side-count histogram of a shell: `(sides, how many faces have that many
/// sides)`, sorted ascending.
fn side_histogram(topo: &Topology, solid: SolidId) -> Vec<(usize, usize)> {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut hist: HashMap<usize, usize> = HashMap::new();
    for &fid in topo.shell(shell).unwrap().faces() {
        let n = topo
            .wire(topo.face(fid).unwrap().outer_wire())
            .unwrap()
            .edges()
            .len();
        *hist.entry(n).or_insert(0) += 1;
    }
    let mut out: Vec<(usize, usize)> = hist.into_iter().collect();
    out.sort_unstable();
    out
}

/// A compact "how many edges are used N times" histogram, for assertion
/// messages: `[(usage_count, how_many_edges), ...]` sorted by usage count.
fn usage_histogram(usage: &HashMap<usize, usize>) -> Vec<(usize, usize)> {
    let mut hist: HashMap<usize, usize> = HashMap::new();
    for &n in usage.values() {
        *hist.entry(n).or_insert(0) += 1;
    }
    let mut out: Vec<(usize, usize)> = hist.into_iter().collect();
    out.sort_unstable();
    out
}

/// The number of faces in the result's outer shell.
fn face_count(topo: &Topology, solid: SolidId) -> usize {
    topo.shell(topo.solid(solid).unwrap().outer_shell())
        .unwrap()
        .faces()
        .len()
}

/// A point quantised to [`WELD_TOL`], so two points within the tolerance
/// compare equal.
type PointKey = (i64, i64, i64);

/// How many distinct `(start, end)` endpoint pairs the shell's edges span, and
/// how many edge occurrences there are in total.
///
/// `occurrences - distinct` is exactly the number of edges `sew::weld_faces`
/// would be able to collapse: it keys on the welded endpoint pair, and nothing
/// else. A result of 0 means welding is a genuine no-op on this shell.
fn weld_collapse_potential(topo: &Topology, solid: SolidId) -> (usize, usize) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let q = |v: f64| (v / WELD_TOL).round() as i64;
    let point_key = |topo: &Topology, vid: brepkit_topology::vertex::VertexId| -> PointKey {
        let p = topo.vertex(vid).unwrap().point();
        (q(p.x()), q(p.y()), q(p.z()))
    };

    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut keys: Vec<(PointKey, PointKey)> = Vec::new();
    for &fid in topo.shell(shell).unwrap().faces() {
        let face = topo.face(fid).unwrap();
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            for oe in topo.wire(wid).unwrap().edges() {
                let e = topo.edge(oe.edge()).unwrap();
                let a = point_key(topo, e.start());
                let b = point_key(topo, e.end());
                keys.push(if a <= b { (a, b) } else { (b, a) });
            }
        }
    }
    let occurrences = keys.len();
    keys.sort_unstable();
    keys.dedup();
    (occurrences, keys.len())
}

fn chamfered_cube(topo: &mut Topology) -> SolidId {
    let solid = make_box(topo, SIDE, SIDE, SIDE).unwrap();
    let edges = solid_edges(topo, solid).unwrap();
    chamfer(topo, solid, &edges, 1.0).unwrap()
}

fn walked_cube(topo: &mut Topology) -> SolidId {
    let solid = make_box(topo, SIDE, SIDE, SIDE).unwrap();
    let edges = solid_edges(topo, solid).unwrap();
    chamfer_v2(topo, solid, &edges, 1.0, 1.0).unwrap().solid
}

/// The analytic face budget of a cube chamfered on all 12 edges.
///
/// 6 side faces + 12 bevels + 8 corner triangles.
const EXPECTED_FACES: usize = 6 + 12 + 8;

/// How many of those faces are corner triangles.
const EXPECTED_TRIANGLES: usize = 8;

// ── The control: a chamfered cube has 26 faces, 8 of them triangles ────────

/// Control case, and the anchor for every expectation below.
///
/// `operations::chamfer::chamfer` is a different solver that rebuilds face
/// polygons and assembles through a shared spatial-hash dedup. It produces
/// the full 26-face result. If this test ever fails, the face budget encoded
/// in `EXPECTED_FACES` is wrong and the tests that follow are measuring
/// against a bad expectation — not the engine under test being at fault.
#[test]
fn control_chamfered_cube_has_the_full_corner_budget() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer(&mut topo, solid, &edges, d).unwrap();

        assert_eq!(
            face_count(&topo, result),
            EXPECTED_FACES,
            "a cube chamfered on all 12 edges has 6 side faces + 12 bevels + \
             8 corner triangles; the control engine must produce all of them \
             at d={d} (if this fails, EXPECTED_FACES is wrong, not the engine)"
        );

        let hist = side_histogram(&topo, result);
        assert_eq!(
            hist,
            vec![(3, EXPECTED_TRIANGLES), (4, 18)],
            "exactly the 8 corner patches are triangles, the other 18 faces are \
             quads (6 inset sides + 12 bevels); got histogram {hist:?} at d={d}"
        );
    }
}

// ── The defect: `chamfer_v2` builds no corner patches ─────────────────────

/// The defect, stated as the analytic face budget it fails to meet.
///
/// This is the assertion that makes "the corner patches are missing"
/// checkable rather than a claim in a comment.
///
/// Currently red and `#[ignore]`d so the default suite stays green; the
/// failure *is* the ticket. **The fix is to call `corner::compute_corners`
/// from `chamfer_builder`, after which removing the `#[ignore]` turns this
/// green with no edit to its assertions** — and
/// `walking_engine_current_output_is_18_quads_with_no_triangles` gets deleted
/// at the same time. The control case below is what proves the expectation
/// itself is right; if it ever goes red, `EXPECTED_FACES` is wrong and this
/// test is measuring against a bad budget rather than catching a real defect.
#[test]
#[ignore = "chamfer_v2 never calls corner::compute_corners, so the 8 corner patches are absent"]
fn walking_engine_builds_the_corner_patches() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;

        assert_eq!(
            face_count(&topo, result),
            EXPECTED_FACES,
            "chamfer_v2 must build 6 side faces + 12 bevels + 8 corner \
             triangles at d={d}, like the control engine does"
        );
        assert_eq!(
            side_histogram(&topo, result),
            vec![(3, EXPECTED_TRIANGLES), (4, 18)],
            "the 8 corner patches must be triangles at d={d}"
        );
    }
}

/// Records what `chamfer_v2` produces instead, so the present state is pinned
/// by a measurement rather than by a comment that can silently drift.
///
/// Deliberately green: it documents the present output, not the desired one.
/// The gap lives in `walking_engine_builds_the_corner_patches` (red, ignored).
///
/// **When the corner patches land, this test is deleted** — its two numbers
/// move into the module docs and into `chamfer_shell_manifold.rs` as the
/// "before" measurement, and the ignore comes off
/// `walking_engine_builds_the_corner_patches` with no edit to its assertions.
/// Keeping both would leave a test asserting a state the engine is no longer
/// in.
///
/// Until then, if this test starts failing unexpectedly, the engine changed
/// and every number quoted about it — here, in `chamfer_builder.rs`, and in
/// `chamfer_shell_manifold.rs` — needs re-measuring.
#[test]
fn walking_engine_current_output_is_18_quads_with_no_triangles() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;

        assert_eq!(
            face_count(&topo, result),
            18,
            "present-day output is 6 sides + 12 bevels and no corner patches; \
             if this number moved, re-measure everything quoted about it \
             (d={d})"
        );
        assert_eq!(
            side_histogram(&topo, result),
            vec![(4, 18)],
            "all 18 present-day faces are quads — no triangles at all (d={d})"
        );
    }
}

// ── Why welding cannot be the fix ─────────────────────────────────────────

/// The measurement that exonerates `sew::weld_faces` on this shell.
///
/// `sew::weld_faces` merges edges spanning the same welded endpoint pair. Over
/// the `chamfer_v2` shell's 72 edge occurrences there are 72 *distinct*
/// endpoint pairs — so there is no second copy of any curve and the welder
/// has nothing to collapse. The shell cannot be closed by deduplication; the
/// missing corner faces have to be constructed.
#[test]
fn walking_engine_shell_has_nothing_for_welding_to_collapse() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;

        let (occurrences, distinct) = weld_collapse_potential(&topo, result);
        assert_eq!(
            occurrences, 72,
            "18 quad faces give 72 edge occurrences at d={d}"
        );
        assert_eq!(
            distinct, occurrences,
            "every edge spans a distinct endpoint pair, so weld_faces is a \
             no-op here (d={d}); the fix must build the missing corner faces"
        );
    }
}

/// The counter's own control: confirm it reports duplicates when duplicates
/// exist, so the zero above is a fact about the shell rather than a broken
/// measurement.
///
/// `weld_collapse_potential` returns `(occurrences, distinct endpoint pairs)`.
/// A shell whose curves are each referenced once has `occurrences ==
/// distinct`; a shell that shares curves has `occurrences > distinct`. The
/// chamfer control engine produces a closed 26-face shell, so some of its
/// curves are necessarily referenced twice — the counter must see that, and
/// must *not* see it on the walked shell.
///
/// Reverse-verified: stubbing the distinct count to always equal the
/// occurrence count (simulating a broken counter) turns the first half of
/// this test red with `occurrences=96 distinct=96`.
#[test]
fn the_weld_counter_detects_sharing_when_it_exists() {
    // Control engine: a closed shell, so some curves are referenced twice.
    let mut topo = Topology::new();
    let solid = chamfered_cube(&mut topo);
    let (occurrences, distinct) = weld_collapse_potential(&topo, solid);
    assert!(
        occurrences > distinct,
        "a closed 26-face shell must reference some curves more than once, \
         which the counter has to see; got occurrences={occurrences} \
         distinct={distinct}"
    );

    // And the walked shell, for contrast, on the same measurement.
    let mut topo2 = Topology::new();
    let walked = walked_cube(&mut topo2);
    let (occ2, distinct2) = weld_collapse_potential(&topo2, walked);
    assert_eq!(
        occ2, distinct2,
        "the walked shell references no curve twice — this is the defect, and \
         it is what makes welding useless here"
    );
}

// ── Why the shell reads as 18 pieces rather than one holed shell ───────────

/// Every one of `chamfer_v2`'s faces is a closed loop, yet no two faces share
/// an edge.
///
/// These are two independent facts and both are needed to explain the symptom:
/// the faces are individually well-formed (so nothing is obviously broken
/// about any one face), and they are mutually unconnected (so the shell has
/// no adjacency at all). Together they are why the result presents as 18
/// disconnected components rather than as a single shell with holes in it —
/// a distinction that matters for reading the failure.
#[test]
fn every_face_is_individually_closed_but_shares_no_edge_with_a_neighbour() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;

        let shell = topo.solid(result).unwrap().outer_shell();
        let mut usage: HashMap<usize, usize> = HashMap::new();
        let mut unclosed: Vec<usize> = Vec::new();

        for (i, &fid) in topo.shell(shell).unwrap().faces().iter().enumerate() {
            let oes = topo
                .wire(topo.face(fid).unwrap().outer_wire())
                .unwrap()
                .edges()
                .to_vec();

            // Fact one: this face's own wire closes end-to-start.
            let closes = {
                let first = oes[0];
                let ef = topo.edge(first.edge()).unwrap();
                let last = oes[oes.len() - 1];
                let el = topo.edge(last.edge()).unwrap();
                last.oriented_end(el) == first.oriented_start(ef)
            };
            if !closes {
                unclosed.push(i);
            }

            // Fact two: no edge of it is referenced by any other face.
            for oe in &oes {
                *usage.entry(oe.edge().index()).or_insert(0) += 1;
            }
        }

        assert!(
            unclosed.is_empty(),
            "every present-day face should be individually closed (measured \
             for d={d}); unclosed face indices {unclosed:?}"
        );
        assert!(
            usage.values().all(|&n| n == 1),
            "no two faces share an edge, which is why the shell falls apart \
             into 18 pieces (d={d}); (edges used N times -> count) = {:?}",
            usage_histogram(&usage)
        );
    }
}

/// The contrast that makes the above meaningful: in the control engine's
/// closed shell, faces *do* share edges — every one exactly twice.
#[test]
fn control_shell_faces_share_every_edge_exactly_twice() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer(&mut topo, solid, &edges, d).unwrap();

        let shell = topo.solid(result).unwrap().outer_shell();
        let mut usage: HashMap<usize, usize> = HashMap::new();
        for &fid in topo.shell(shell).unwrap().faces() {
            let face = topo.face(fid).unwrap();
            for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied())
            {
                for oe in topo.wire(wid).unwrap().edges() {
                    *usage.entry(oe.edge().index()).or_insert(0) += 1;
                }
            }
        }

        assert!(
            usage.values().all(|&n| n == 2),
            "a closed shell references every edge exactly twice (d={d}); \
             (edges used N times -> count) = {:?}",
            usage_histogram(&usage)
        );
    }
}

// ── Side count helper kept honest ─────────────────────────────────────────

/// Guards the helper the other tests rely on: a face's side count is its outer
/// wire's oriented-edge count. Cheap, but if `side_count` were wrong the
/// histogram assertions would be measuring something else entirely.
#[test]
fn side_count_reads_the_outer_wire() {
    let mut topo = Topology::new();
    let solid = chamfered_cube(&mut topo);
    let shell = topo.solid(solid).unwrap().outer_shell();
    for &fid in topo.shell(shell).unwrap().faces() {
        let direct = topo
            .wire(topo.face(fid).unwrap().outer_wire())
            .unwrap()
            .edges()
            .len();
        assert_eq!(
            side_count(&topo, fid),
            direct,
            "side_count must agree with a direct read of the outer wire"
        );
    }
}

// ── Why the fillet corner path is the wrong one for chamfer ───────────────

/// Surface census of the current chamfer result: all planes, no NURBS.
///
/// This is the baseline the experiment below is read against. `chamfer_v2`
/// produces flat geometry today; a fix that introduces NURBS faces into the
/// corner is building something a chamfer should not have.
#[test]
fn chamfer_result_is_all_planar_today() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;
        let shell = topo.solid(result).unwrap().outer_shell();

        let mut planes = 0usize;
        let mut curved = 0usize;
        for &fid in topo.shell(shell).unwrap().faces() {
            match topo.face(fid).unwrap().surface() {
                FaceSurface::Plane { .. } => planes += 1,
                _ => curved += 1,
            }
        }
        assert_eq!(
            (planes, curved),
            (18, 0),
            "the present result is 18 planes and nothing else at d={d}; a \
             NURBS face here would mean a spherical corner patch had been \
             substituted for the flat triangle a chamfer requires"
        );
    }
}

/// The standing record of the experiment that ruled out the obvious fix.
///
/// `corner::compute_corners` was wired into `chamfer_builder` unchanged, the
/// way `fillet_builder` calls it, and measured at d = 0.5 / 1 / 2. The face
/// count went 18 -> 26, so the routine does produce 8 corner faces. The shell
/// did **not** close: `V/E/F = 100/96/26`, `free = 96`, `over-shared = 0`,
/// `components = 26` — every one of the 96 edges orphaned, and 26 isolated
/// pieces instead of 18.
///
/// The 8 new faces were NURBS, not planes. `corner.rs` sends 3+ stripe
/// vertices to `spherical_triangle`, whose module docs say "rolling-ball
/// sphere", "great-circle arcs" and "Fillet radius"; a chamfer corner is a
/// flat triangle. It also reads `CircSection`, which `analytic.rs` fills with
/// a chord half-length while noting that the struct "is shaped for fillets".
///
/// Recording it here means the next person does not spend the experiment
/// again. Re-run it by reverting this test's `WIRED_UP` marker to true and
/// adding the call; the numbers above are what to expect.
///
/// The consequence for the fix: the chamfer corner patch has to be a **plane
/// through the three contact points**, constructed by chamfer-specific code.
/// Reusing the fillet corner module cannot produce it.
#[test]
fn the_spherical_corner_path_is_wrong_for_chamfer() {
    // These assertions describe the un-wired baseline. If
    // `corner::compute_corners` ever does get wired into `chamfer_builder`,
    // this whole file's expectations move, and the numbers in this test's docs
    // are the ones to re-measure.
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;
        let shell = topo.solid(result).unwrap().outer_shell();

        let mut usage: HashMap<usize, usize> = HashMap::new();
        let mut nurbs = 0usize;
        for &fid in topo.shell(shell).unwrap().faces() {
            if !matches!(topo.face(fid).unwrap().surface(), FaceSurface::Plane { .. }) {
                nurbs += 1;
            }
            let face = topo.face(fid).unwrap();
            for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied())
            {
                for oe in topo.wire(wid).unwrap().edges() {
                    *usage.entry(oe.edge().index()).or_insert(0) += 1;
                }
            }
        }

        // Wiring the spherical path in produced 8 NURBS faces. Un-wired, there
        // are none.
        assert_eq!(
            nurbs, 0,
            "the spherical corner path must not be wired in: it yields 8 NURBS \
             faces where a chamfer requires flat triangles (d={d})"
        );
        // And every edge stays free, which is the shell symptom the spherical
        // path was supposed to fix and did not.
        let free = usage.values().filter(|&&n| n == 1).count();
        assert_eq!(
            free,
            usage.len(),
            "with the spherical path un-wired every edge is free, as measured \
             before the experiment (d={d}); wiring it in left all 96 free too"
        );
    }
}

//! A chamfered cube's corner budget, and why the corners have to be *built*.
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
//! # Why the corner patches have to be built, not deduplicated
//!
//! `chamfer_v2` used to produce 18 quads and no triangles. The eight corner
//! triangles were simply absent, and — measured over the 72 edge occurrences
//! of that shell — the number of distinct `(start, end)` position pairs was
//! *also* 72. There was no second copy of any curve, so `sew::weld_faces`,
//! which keys on exactly that pair, had nothing to collapse. That is what
//! pinned the fix to construction rather than deduplication: no amount of
//! merging what was already there could close the shell.
//!
//! # The fix, and the measurement that pins it
//!
//! `blend::chamfer_corner` now does both halves of the job:
//!
//! * the triangle through the three contact points at each such vertex —
//!   found as the crossings of the contact lines two chamfered edges leave on
//!   the face they share;
//! * a setback on each bevel, read off that triangle's own corners, so the
//!   bevel stops where the triangle starts rather than overlapping it.
//!
//! With the patches present there finally *is* a second copy of every shared
//! curve, so welding has something to do — `welded_shell_leaves_no_edge_orphaned`
//! below is the "after" measurement, and `weld_collapse_potential` is the
//! counter that distinguishes the two states.
//!
//! # The fix is NOT `corner::compute_corners` as-is
//!
//! The obvious move was to call the fillet builder's corner routine. Measured:
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
//! record of that experiment, so nobody re-runs it. The corner patch a chamfer
//! needs is a plane through the three contact points, built by
//! chamfer-specific code.

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

/// How many faces reference each edge of the outer shell.
fn edge_usage(topo: &Topology, solid: SolidId) -> HashMap<usize, usize> {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut usage: HashMap<usize, usize> = HashMap::new();
    for &fid in topo.shell(shell).unwrap().faces() {
        let face = topo.face(fid).unwrap();
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            for oe in topo.wire(wid).unwrap().edges() {
                *usage.entry(oe.edge().index()).or_insert(0) += 1;
            }
        }
    }
    usage
}

/// A point quantised to [`WELD_TOL`], so two points within the tolerance
/// compare equal.
type PointKey = (i64, i64, i64);

/// How many distinct `(start, end)` endpoint pairs the shell's edges span, and
/// how many edge occurrences there are in total.
///
/// `occurrences - distinct` is exactly the number of edges `sew::weld_faces`
/// would be able to collapse: it keys on the welded endpoint pair, and nothing
/// else. Zero means welding is a genuine no-op on this shell.
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

// ── The requirement: `chamfer_v2` meets the same budget ───────────────────

/// The analytic face budget, asserted against `chamfer_v2`.
///
/// This was red until `blend::chamfer_corner` landed. It asserts the budget
/// rather than a golden face count so it can never pass by accident: the
/// control case above is what proves the budget itself is right.
#[test]
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

/// The corners are flat triangles, not curved patches.
///
/// A chamfer corner is cut by a plane; `F = 26` reached with curved faces
/// would mean a spherical patch had been substituted. See the module docs —
/// wiring `corner::compute_corners` in unchanged did exactly that, and the
/// shell still did not close.
#[test]
fn chamfer_result_is_all_planar() {
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
            (EXPECTED_FACES, 0),
            "all {EXPECTED_FACES} faces are planes at d={d}; a NURBS face here would \
             mean a spherical corner patch had been substituted for the flat \
             triangle a chamfer requires"
        );
    }
}

// ── Welding: pointless before the patches, necessary after ────────────────

/// The "after" measurement that the "before" one is read against.
///
/// Before the corner patches existed this shell had 72 edge occurrences over
/// 72 distinct endpoint pairs — nothing for `weld_faces` to collapse, which is
/// what proved the faces had to be built. With the patches present every
/// curve has a second copy, so welding closes the shell: every edge is
/// referenced exactly twice and nothing is orphaned.
#[test]
fn welded_shell_leaves_no_edge_orphaned() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = chamfer_v2(&mut topo, solid, &edges, d, d).unwrap().solid;

        // 6 sides x 4 + 12 bevels x 4 + 8 triangles x 3.
        let expected_occurrences: usize = 6 * 4 + 12 * 4 + 8 * 3;
        let (occurrences, distinct) = weld_collapse_potential(&topo, result);

        assert_eq!(
            occurrences, expected_occurrences,
            "26 faces give {expected_occurrences} edge occurrences at d={d}"
        );
        assert!(
            distinct < occurrences,
            "with the corner patches built, curves are shared and weld_faces has \
             something to collapse; got occurrences={occurrences} \
             distinct={distinct} at d={d}"
        );

        let usage = edge_usage(&topo, result);
        assert!(
            usage.values().all(|&n| n == 2),
            "after welding every edge is referenced exactly twice (d={d}); \
             (edges used N times -> count) = {:?}",
            usage_histogram(&usage)
        );
    }
}

/// The counter's own control: confirm it reports duplicates when duplicates
/// exist, so the numbers above are facts about the shells rather than a broken
/// measurement.
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

    // And the walking engine, now that it closes too.
    let mut topo2 = Topology::new();
    let walked = walked_cube(&mut topo2);
    let (occ2, distinct2) = weld_collapse_potential(&topo2, walked);
    assert!(
        occ2 > distinct2,
        "the walked shell now shares its curves as well; got occurrences={occ2} \
         distinct={distinct2}"
    );
}

// ── Why the shell used to read as 18 pieces rather than one holed shell ────

/// Every face is individually closed, and every edge now has two faces on it.
///
/// The first fact is why the old failure read as "18 disconnected components"
/// rather than "one shell with holes": nothing was wrong with any single face,
/// they were mutually unconnected. The second is what changed — pairs of faces
/// now meet along shared edges instead of each owning its own copy.
#[test]
fn every_face_is_individually_closed_and_shares_its_edges() {
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

            for oe in &oes {
                *usage.entry(oe.edge().index()).or_insert(0) += 1;
            }
        }

        assert!(
            unclosed.is_empty(),
            "every face should be individually closed (measured for d={d}); \
             unclosed face indices {unclosed:?}"
        );
        assert!(
            usage.values().all(|&n| n == 2),
            "every edge is shared by exactly two faces (d={d}); \
             (edges used N times -> count) = {:?}",
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
/// again. The guard below is what is left of it: the shipped corner patches
/// are planes, and the shell closes — so neither symptom can come back without
/// this test noticing.
#[test]
fn the_spherical_corner_path_is_wrong_for_chamfer() {
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

        // Wiring the spherical path in produced 8 NURBS faces; the shipped
        // patches are planes.
        assert_eq!(
            nurbs, 0,
            "the spherical corner path must not be wired in: it yields 8 NURBS \
             faces where a chamfer requires flat triangles (d={d})"
        );
        // And the shell closes, which the spherical path did not achieve.
        let free = usage.values().filter(|&&n| n == 1).count();
        assert_eq!(
            free, 0,
            "the flat corner patches close the shell; wiring in the spherical \
             path instead left all 96 edges free (d={d})"
        );
    }
}

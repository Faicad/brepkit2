//! Measured shape of `chamfer_v2`'s result, and the corner-patch experiment.
//!
//! This file exists to keep numbers that were measured during an investigation
//! from decaying into folklore. Every figure quoted in
//! `docs/analysis/2026-10-03-chamfer-corner-patch-attempt.md` is asserted here,
//! so a later change that moves any of them turns a test red instead of
//! silently making the write-up wrong.
//!
//! # What is measured
//!
//! ## The defect (`chamfer_v2` as shipped)
//!
//! On a 10^3 box chamfered on all 12 edges at d = 0.5 / 1 / 2, identically at
//! all three distances:
//!
//! ```text
//! V/E/F = 76/72/18   free = 72   over-shared = 0   components = 18
//! ```
//!
//! Every one of the 72 edges is referenced by exactly one face, and the 18
//! faces form 18 separate pieces. The 18 are 6 side faces + 12 bevels; the 8
//! corner triangles a chamfered cube needs are absent, which is why the face
//! budget is 18 rather than 26.
//!
//! ## The corner-patch experiment (applied, then reverted)
//!
//! A planar corner patch was written and wired in. It reached the right face
//! count — 26, with 8 triangular faces and `over-shared = 0` — and left the
//! shell just as open:
//!
//! ```text
//! V/E/F = 76/80/26   free = 64   over-shared = 0   components = 12
//! ```
//!
//! ## Why the shell stays open
//!
//! Euler characteristic. A closed genus-0 shell satisfies V − E + F = 2, and
//! for this solid V must be 24 (two contact points per original edge, 12
//! edges). Measured V is 76 in both states: the vertices are never welded, so
//! every edge is orphaned and `free` cannot reach 0. The corner patch fixed a
//! face count, not the topology underneath it.
//!
//! Every one of a side face's four edges is used once. Welding cannot repair it
//! either: `sew::weld_faces` keys on shared endpoints, and measured over the 72
//! edge occurrences the number of distinct `(start, end)` position pairs
//! quantised to 1e-6 is also 72 — nothing to collapse. Measured with the corner
//! patch wired in behind it, `free` stayed 64.
//!
//! # Left as a measurement, not a fix
//!
//! The production change is **not** in the tree; `chamfer_v2` behaves exactly
//! as `chamfer_shell_manifold.rs` and `chamfer_corner_patches.rs` describe.
//! These tests pin the numbers so the next attempt starts from facts rather
//! than from the write-up's prose.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use std::collections::HashMap;

use brepkit_operations::blend_ops::chamfer_v2;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::face::FaceSurface;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

/// Cube side used throughout.
const SIDE: f64 = 10.0;

/// Chamfer distances swept. Each leaves a non-degenerate eroded core.
const DISTANCES: [f64; 3] = [0.5, 1.0, 2.0];

/// Vertices a closed chamfered cube must have: two contact points per original
/// edge, twelve edges.
const EXPECTED_VERTICES: usize = 24;

/// Faces a chamfered cube must have: 6 sides + 12 bevels + 8 corner triangles.
const EXPECTED_FACES: usize = 6 + 12 + 8;

type PointKey = (i64, i64, i64);

fn key(p: brepkit_math::vec::Point3) -> PointKey {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let q = |v: f64| (v / 1e-6).round() as i64;
    (q(p.x()), q(p.y()), q(p.z()))
}

fn fmt(p: brepkit_math::vec::Point3) -> String {
    format!("({:.1},{:.1},{:.1})", p.x(), p.y(), p.z())
}

fn chamfered(topo: &mut Topology, d: f64) -> SolidId {
    let solid = make_box(topo, SIDE, SIDE, SIDE).unwrap();
    let edges = solid_edges(&*topo, solid).unwrap();
    chamfer_v2(topo, solid, &edges, d, d).unwrap().solid
}

/// (vertex count, edge count, face count) of the result's outer shell.
fn counts(topo: &Topology, solid: SolidId) -> (usize, usize, usize) {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let faces = topo.shell(shell).unwrap().faces();
    let mut edges: Vec<usize> = faces
        .iter()
        .flat_map(|&fid| {
            let face = topo.face(fid).unwrap();
            std::iter::once(face.outer_wire())
                .chain(face.inner_wires().iter().copied())
                .flat_map(|wid| {
                    topo.wire(wid)
                        .unwrap()
                        .edges()
                        .iter()
                        .map(|oe| oe.edge().index())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    edges.sort_unstable();
    edges.dedup();
    (topo.num_vertices(), edges.len(), faces.len())
}

/// How many faces reference each edge of the shell.
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

/// "how many edges are used N times" -> (N, count), sorted by N.
fn usage_histogram(usage: &HashMap<usize, usize>) -> Vec<(usize, usize)> {
    let mut hist: HashMap<usize, usize> = HashMap::new();
    for &n in usage.values() {
        *hist.entry(n).or_insert(0) += 1;
    }
    let mut out: Vec<(usize, usize)> = hist.into_iter().collect();
    out.sort_unstable();
    out
}

// ── The shipped state, pinned ─────────────────────────────────────────────

/// The numbers `chamfer_shell_manifold.rs` reports, asserted directly so they
/// cannot drift into prose unnoticed.
///
/// `V = 76` against the required 24 is the headline: the vertices are never
/// welded, which is why every edge is free.
#[test]
fn chamfer_v2_result_is_18_quads_with_no_shared_edges() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = chamfered(&mut topo, d);

        let (v, e, f) = counts(&topo, solid);
        assert_eq!(
            (v, e, f),
            (76, 72, 18),
            "measured chamfer_v2 shape at d={d}; if this moved, re-measure \
             everything the write-up claims"
        );

        let usage = edge_usage(&topo, solid);
        assert_eq!(
            usage.len(),
            72,
            "72 distinct edges at d={d}, each of the 18 quads contributing 4"
        );
        assert!(
            usage.values().all(|&n| n == 1),
            "every edge is orphaned: no two faces share one (d={d}), histogram {:?}",
            usage_histogram(&usage)
        );
    }
}

/// The vertex count is the root defect: a closed chamfered cube has 24, and
/// this solid has 76 because the per-producer vertices were never merged.
///
/// `V - E + F` is 22 here against the 2 a closed genus-0 shell requires, and
/// the excess is entirely in V.
#[test]
fn vertex_count_is_three_times_what_a_closed_chamfer_needs() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = chamfered(&mut topo, d);
        let (v, e, f) = counts(&topo, solid);

        assert_eq!(
            v,
            EXPECTED_VERTICES * 3 + 4,
            "V={v} against the {EXPECTED_VERTICES} a closed chamfered cube needs \
             (d={d}); the surplus is unmerged vertices, and it is the reason \
             free edges cannot reach 0"
        );
        #[allow(clippy::cast_possible_wrap)]
        let euler = v as i64 - e as i64 + f as i64;
        assert_ne!(
            euler, 2,
            "Euler characteristic is not 2, so the shell is not a closed \
             genus-0 manifold (d={d})"
        );
    }
}

/// Side faces bound the correct square and share no edge with anything.
///
/// The shape is right — a chamfered cube's side face is the `(S-2d)` square,
/// since chamfering cuts the four *edges* of it, not its four corners. What is
/// missing is any connection to the neighbours: `shared with a neighbour: 0`.
#[test]
fn side_faces_are_correct_squares_but_share_nothing() {
    let mut topo = Topology::new();
    let solid = chamfered(&mut topo, 1.0);
    let shell = topo.solid(solid).unwrap().outer_shell();
    let usage = edge_usage(&topo, solid);

    let mut sides_checked = 0;
    for &fid in topo.shell(shell).unwrap().faces() {
        let FaceSurface::Plane { normal, .. } = topo.face(fid).unwrap().surface() else {
            continue;
        };
        // A side face's plane normal has exactly one non-zero component.
        let zeros = [normal.x(), normal.y(), normal.z()]
            .iter()
            .filter(|&&c| c.abs() < 1e-6)
            .count();
        if zeros != 2 {
            continue;
        }
        sides_checked += 1;

        let oes = topo
            .wire(topo.face(fid).unwrap().outer_wire())
            .unwrap()
            .edges();
        assert_eq!(oes.len(), 4, "a side face is a quadrilateral (fid {fid:?})");

        for oe in oes {
            assert_eq!(
                usage[&oe.edge().index()],
                1,
                "side face edge {} is used once; a closed shell needs 2",
                oe.edge().index()
            );
        }
    }
    assert_eq!(sides_checked, 6, "a cube has 6 side faces");
}

/// The weld-collapse count: nothing to deduplicate, before or after a corner
/// patch exists.
///
/// `sew::weld_faces` merges edges spanning the same welded endpoint pair.
/// Over this shell the number of distinct position pairs equals the number of
/// edge occurrences, so the welder has no work — which is why wiring it in
/// changed nothing, and why the defect has to be fixed where the vertices are
/// created rather than after assembly.
///
/// The control runs the identical counter over `operations::chamfer::chamfer`,
/// a closed shell, and must report duplicates; without it a zero here would be
/// indistinguishable from a broken measurement. Reverse-verified by stubbing
/// the distinct count to always equal the occurrence count, which turns the
/// control red with `occurrences=96 distinct=96`.
#[test]
fn there_is_nothing_for_welding_to_collapse() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = chamfered(&mut topo, d);

        let occurrences = edge_occurrences(&topo, solid);
        let distinct = distinct_endpoint_pairs(&topo, solid);
        assert_eq!(
            occurrences, 72,
            "18 quads give 72 edge occurrences at d={d}"
        );
        assert_eq!(
            distinct, occurrences,
            "no two edges share an endpoint pair at d={d}, so weld_faces is \
             a no-op and the defect is upstream of assembly"
        );
    }

    // Control: a closed shell must show sharing through the same counter.
    let mut topo = Topology::new();
    let box_solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let edges = solid_edges(&topo, box_solid).unwrap();
    let closed = brepkit_operations::chamfer::chamfer(&mut topo, box_solid, &edges, 1.0).unwrap();

    let occ = edge_occurrences(&topo, closed);
    let dist = distinct_endpoint_pairs(&topo, closed);
    assert!(
        occ > dist,
        "a closed shell must reference some curves twice and the counter has \
         to see it; got occurrences={occ} distinct={dist}"
    );
}

/// Total oriented-edge references across the shell.
fn edge_occurrences(topo: &Topology, solid: SolidId) -> usize {
    let shell = topo.solid(solid).unwrap().outer_shell();
    topo.shell(shell)
        .unwrap()
        .faces()
        .iter()
        .map(|&fid| {
            let face = topo.face(fid).unwrap();
            std::iter::once(face.outer_wire())
                .chain(face.inner_wires().iter().copied())
                .map(|wid| topo.wire(wid).unwrap().edges().len())
                .sum::<usize>()
        })
        .sum()
}

/// Distinct `(start, end)` position pairs, quantised to 1e-6 to match
/// `sew::weld_faces`.
fn distinct_endpoint_pairs(topo: &Topology, solid: SolidId) -> usize {
    let shell = topo.solid(solid).unwrap().outer_shell();
    let mut keys: Vec<(PointKey, PointKey)> = Vec::new();
    for &fid in topo.shell(shell).unwrap().faces() {
        let face = topo.face(fid).unwrap();
        for wid in std::iter::once(face.outer_wire()).chain(face.inner_wires().iter().copied()) {
            for oe in topo.wire(wid).unwrap().edges() {
                let e = topo.edge(oe.edge()).unwrap();
                let a = key(topo.vertex(e.start()).unwrap().point());
                let b = key(topo.vertex(e.end()).unwrap().point());
                keys.push(if a <= b { (a, b) } else { (b, a) });
            }
        }
    }
    keys.sort_unstable();
    keys.dedup();
    keys.len()
}

// ── Corner geometry, as measured ──────────────────────────────────────────

/// The contact points a corner patch would span, and which faces own them.
///
/// Measured on a 10^3 box at d = 1, at the corner (0,0,0):
///
/// ```text
///   (1,0,0) -> two bevels
///   (0,1,0) -> two bevels
///   (0,0,1) -> two bevels
/// ```
///
/// No side face references any of them. All three legs the triangle needs
/// already exist as entities, each shared by a pair of bevels — so a corner
/// patch here is purely additive: adopt three edges, add one face.
#[test]
fn corner_contact_points_are_owned_by_bevels_and_one_leg_is_missing() {
    let mut topo = Topology::new();
    let solid = chamfered(&mut topo, 1.0);
    let shell = topo.solid(solid).unwrap().outer_shell();
    let faces = topo.shell(shell).unwrap().faces().to_vec();

    let d = 1.0;
    let contact = [
        brepkit_math::vec::Point3::new(d, 0.0, 0.0),
        brepkit_math::vec::Point3::new(0.0, d, 0.0),
        brepkit_math::vec::Point3::new(0.0, 0.0, d),
    ];

    // Each contact point is used by two faces, and both are bevels (a bevel
    // plane normal has two non-zero components).
    for p in contact {
        let k = key(p);
        let mut users: Vec<brepkit_topology::face::FaceId> = Vec::new();
        for &fid in &faces {
            let face = topo.face(fid).unwrap();
            for oe in topo.wire(face.outer_wire()).unwrap().edges() {
                let e = topo.edge(oe.edge()).unwrap();
                if (key(topo.vertex(e.start()).unwrap().point()) == k
                    || key(topo.vertex(e.end()).unwrap().point()) == k)
                    && !users.contains(&fid)
                {
                    users.push(fid);
                }
            }
        }
        assert!(
            users.len() >= 2,
            "{} is shared by at least two faces, got {users:?}",
            fmt(p)
        );
        for &u in &users {
            let FaceSurface::Plane { normal, .. } = topo.face(u).unwrap().surface() else {
                unreachable!("user face must be planar; the shell is all planes")
            };
            let zeros = [normal.x(), normal.y(), normal.z()]
                .iter()
                .filter(|&&c| c.abs() < 1e-6)
                .count();
            assert_eq!(zeros, 1, "a bevel normal has two non-zero components");
        }
    }

    // Of the three legs, two exist and one does not.
    let shell_edge_keys: Vec<(PointKey, PointKey)> = {
        let mut keys: Vec<(PointKey, PointKey)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for &fid in &faces {
            for oe in topo
                .wire(topo.face(fid).unwrap().outer_wire())
                .unwrap()
                .edges()
            {
                if !seen.insert(oe.edge().index()) {
                    continue;
                }
                let e = topo.edge(oe.edge()).unwrap();
                let a = key(topo.vertex(e.start()).unwrap().point());
                let b = key(topo.vertex(e.end()).unwrap().point());
                keys.push(if a <= b { (a, b) } else { (b, a) });
            }
        }
        keys
    };
    let has = |a: brepkit_math::vec::Point3, b: brepkit_math::vec::Point3| {
        let (ka, kb) = (key(a), key(b));
        let k = if ka <= kb { (ka, kb) } else { (kb, ka) };
        shell_edge_keys.contains(&k)
    };

    // All three legs exist as entities, each shared by two bevels. Measured
    // after the corner-patch attempt was reverted; an earlier reading taken
    // with the patch wired in reported the third as absent, which was an
    // artefact of the patched state, not the shipped geometry.
    assert!(
        has(contact[0], contact[1]),
        "leg (1,0,0)-(0,1,0) exists, shared by two bevels"
    );
    assert!(
        has(contact[1], contact[2]),
        "leg (0,1,0)-(0,0,1) exists, shared by two bevels"
    );
    assert!(
        has(contact[0], contact[2]),
        "leg (1,0,0)-(0,0,1) exists as well: the three contact points are \
         already mutually connected, so a corner patch would adopt all three \
         rather than mint one"
    );
}

/// The vertex-degree histogram, which is what identifies a corner.
///
/// Measured on a 10^3 box at d = 1, as shipped: `[(2, 24), (4, 24)]` — 24
/// vertices of degree 2 and 24 of degree 4. No vertex has degree 3.
///
/// This is the measurement that corrected two wrong assumptions during the
/// attempt:
///
/// * a corner's contact point is **degree 4**, not 3 — it joins the two other
///   contact points of its own corner *and* two points of the neighbouring
///   corners along the bevels;
/// * "take the 2 nearest neighbours" does not identify corners, because a side
///   face's own corners also have 2 near ones, only a whole edge away.
///
/// What does work is the shape of the sorted distance list: a real corner is the
/// smallest cluster followed by a jump (`1.41, 1.41, 10, 10`), while a side-face
/// corner runs the other way (`8, 8, 1, 1`).
#[test]
fn vertex_degrees_separate_real_corners_from_side_face_corners() {
    let mut topo = Topology::new();
    let solid = chamfered(&mut topo, 1.0);
    let shell = topo.solid(solid).unwrap().outer_shell();

    // adjacency by position
    let mut adj: HashMap<PointKey, Vec<PointKey>> = HashMap::new();
    let mut pos: HashMap<PointKey, brepkit_math::vec::Point3> = HashMap::new();
    let mut seen_edges = std::collections::HashSet::new();
    for &fid in topo.shell(shell).unwrap().faces() {
        for oe in topo
            .wire(topo.face(fid).unwrap().outer_wire())
            .unwrap()
            .edges()
        {
            if !seen_edges.insert(oe.edge().index()) {
                continue;
            }
            let e = topo.edge(oe.edge()).unwrap();
            let a = topo.vertex(e.start()).unwrap().point();
            let b = topo.vertex(e.end()).unwrap().point();
            let (ka, kb) = (key(a), key(b));
            pos.insert(ka, a);
            pos.insert(kb, b);
            adj.entry(ka).or_default().push(kb);
            adj.entry(kb).or_default().push(ka);
        }
    }

    let mut hist: HashMap<usize, usize> = HashMap::new();
    for n in adj.values() {
        *hist.entry(n.len()).or_insert(0) += 1;
    }
    let mut got: Vec<(usize, usize)> = hist.into_iter().collect();
    got.sort_unstable();
    assert_eq!(
        got,
        vec![(2, 24), (4, 24)],
        "degree histogram as shipped; a corner's contact point is degree 4, \
         never 3"
    );

    // The distance-jump discriminator, on a real corner and on a side corner.
    let corner = *pos
        .keys()
        .find(|&&k| {
            // a corner contact point: has a degree-4 neighbourhood whose two
            // smallest distances are equal and much smaller than the rest
            let ns = &adj[&k];
            let mut d: Vec<f64> = ns.iter().map(|&n| (pos[&n] - pos[&k]).length()).collect();
            d.sort_by(|a, b| a.partial_cmp(b).unwrap());
            d.len() == 4 && (d[0] - d[1]).abs() < 1e-9 && d[2] > d[0] * 1.5
        })
        .expect("some vertex has the corner signature");
    let ns = &adj[&corner];
    let mut d: Vec<f64> = ns
        .iter()
        .map(|&n| (pos[&n] - pos[&corner]).length())
        .collect();
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        d[0] > 1e-6 && d[2] >= d[0] * 1.5,
        "a real corner's distance list jumps after its near cluster, measured \
         {d:?}"
    );
}

// ── The control engine, for comparison ────────────────────────────────────

/// The control reaches what `chamfer_v2` does not: the full face budget, and a
/// closed shell.
///
/// This is the target `chamfer_v2` is being measured against, and the reason
/// the expectations in this file are not arbitrary.
#[test]
fn control_engine_reaches_the_full_budget_and_closes() {
    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let result = brepkit_operations::chamfer::chamfer(&mut topo, solid, &edges, d).unwrap();

        let (v, e, f) = counts(&topo, result);
        assert_eq!(f, EXPECTED_FACES, "control face budget at d={d}");
        // The control closes every edge (each used exactly twice) and passes
        // validate_shell_closed below, but its Euler characteristic is 10, not
        // 2: it also carries surplus vertices (V=32 against 24). So "closed" in
        // this codebase means every edge is shared twice, not that V - E + F
        // holds. Worth knowing before treating the control as a golden
        // reference for topology.
        #[allow(clippy::cast_possible_wrap)]
        let euler = v as i64 - e as i64 + f as i64;
        assert!(
            euler > 2,
            "control also carries unmerged vertices at d={d} (V={v} E={e} F={f}), \
             so it is watertight but not Euler-clean"
        );
        let usage = edge_usage(&topo, result);
        assert!(
            usage.values().all(|&n| n == 2),
            "control references every edge exactly twice at d={d}"
        );
        let shell = topo.solid(result).unwrap().outer_shell();
        validate_shell_closed(topo.shell(shell).unwrap(), &topo)
            .expect("control must be watertight");
    }
}

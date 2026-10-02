//! Trimming for a chamfer must keep the bulk of the face, not the sliver.
//!
//! Chamfering an edge cuts a bevel along it and trims the two faces that share
//! that edge back to the contact line. The contact line splits each of those
//! faces in two: the sliver between the contact line and the old edge, which
//! the bevel replaces, and everything else — the bulk, which must stay.
//!
//! The two faces of one edge traverse that edge in opposite directions, so
//! "the side the bevel sits on" is *not* the same Left/Right in each face's own
//! frame. Picking one side for both therefore trims one of them backwards:
//! that face collapses to the sliver and the bulk of it simply disappears from
//! the solid, leaving nothing to bound the volume.
//!
//! The assertion needs no closed form and does not depend on the shell being
//! watertight: after chamfering with distance `d`, each trimmed face must still
//! carry a vertex at roughly the full width of the original face away from the
//! chamfered edge — not merely at `d`.
//!
//! Faces are tracked by the plane they lie on, which survives trimming, rather
//! than by `FaceId` — the builder replaces a trimmed face with a new one, and
//! by their vertices, which the trim removes.

#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

use brepkit_math::vec::{Point3, Vec3};
use brepkit_operations::blend_ops::chamfer_v2;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::edge::EdgeId;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::face::{FaceId, FaceSurface};

/// Cube side.
const SIDE: f64 = 10.0;

/// Chamfer distances swept.
const DISTANCES: [f64; 3] = [0.5, 1.0, 2.0];

/// The edge running along `x` at `y = 0`, `z = SIDE`.
fn target_edge(topo: &Topology, edges: &[EdgeId]) -> EdgeId {
    *edges
        .iter()
        .find(|e| {
            let ed = topo.edge(**e).unwrap();
            let a = topo.vertex(ed.start()).unwrap().point();
            let b = topo.vertex(ed.end()).unwrap().point();
            let m = a + (b - a) * 0.5;
            m.y().abs() < 1e-9 && (m.z() - SIDE).abs() < 1e-9
        })
        .expect("a box has an edge along x at y=0, z=SIDE")
}

/// The plane a face lies on, if it is planar.
fn plane_of(topo: &Topology, face: FaceId) -> Option<(Vec3, f64)> {
    match topo.face(face).unwrap().surface() {
        FaceSurface::Plane { normal, d } => Some((*normal, *d)),
        _ => None,
    }
}

fn same_plane(a: (Vec3, f64), b: (Vec3, f64)) -> bool {
    a.0.dot(b.0) > 1.0 - 1e-9 && (a.1 - b.1).abs() < 1e-6
}

/// Furthest any vertex of `face` sits from the infinite line through `a`, `b`.
///
/// Distance is measured to the *line*, not the segment, so it reads the face's
/// extent across the edge rather than along it.
fn extent_across(topo: &Topology, face: FaceId, a: Point3, b: Point3) -> f64 {
    let dir = (b - a).normalize().unwrap();
    let fd = topo.face(face).unwrap();
    let mut best = 0.0_f64;
    for oe in topo.wire(fd.outer_wire()).unwrap().edges() {
        let e = topo.edge(oe.edge()).unwrap();
        for v in [e.start(), e.end()] {
            let rel = topo.vertex(v).unwrap().point() - a;
            let perp = rel - dir * rel.dot(dir);
            best = best.max(perp.length());
        }
    }
    best
}

/// Furthest any face lying on `plane` reaches across the line through `a`, `b`.
fn plane_extent(
    topo: &Topology,
    shell_faces: &[FaceId],
    plane: (Vec3, f64),
    a: Point3,
    b: Point3,
) -> f64 {
    shell_faces
        .iter()
        .filter(|f| plane_of(topo, **f).is_some_and(|p| same_plane(p, plane)))
        .map(|f| extent_across(topo, *f, a, b))
        .fold(0.0_f64, f64::max)
}

/// The two faces sharing `edge` — the ones the chamfer has to trim — identified
/// by the plane they lie on so they can still be found after trimming.
fn faces_on_edge(topo: &Topology, faces: &[FaceId], edge: EdgeId) -> Vec<(Vec3, f64)> {
    let ed = topo.edge(edge).unwrap();
    let ends = [ed.start(), ed.end()];

    faces
        .iter()
        .copied()
        // Both endpoints, counted distinctly: a face perpendicular to the
        // edge touches only one of them, but sees it twice (as the end of one
        // side and the start of the next).
        .filter(|f| {
            let fd = topo.face(*f).unwrap();
            let mut seen = [false; 2];
            for oe in topo.wire(fd.outer_wire()).unwrap().edges() {
                let e = topo.edge(oe.edge()).unwrap();
                for v in [e.start(), e.end()] {
                    if let Some(i) = ends.iter().position(|end| *end == v) {
                        seen[i] = true;
                    }
                }
            }
            seen[0] && seen[1]
        })
        .filter_map(|f| plane_of(topo, f))
        .collect()
}

/// Control: before any chamfer, both faces sharing the edge span the full width
/// of the cube. Pins the measurement — if this fails, the expectation below is
/// wrong, not `chamfer_v2`.
#[test]
fn control_unchamfered_faces_span_the_whole_side() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let shell = topo.solid(solid).unwrap().outer_shell();
    let faces = topo.shell(shell).unwrap().faces().to_vec();

    let edges = solid_edges(&topo, solid).unwrap();
    let target = target_edge(&topo, &edges);
    let ed = topo.edge(target).unwrap();
    let a = topo.vertex(ed.start()).unwrap().point();
    let b = topo.vertex(ed.end()).unwrap().point();

    for plane in faces_on_edge(&topo, &faces, target) {
        let extent = plane_extent(&topo, &faces, plane, a, b);
        assert!(
            (extent - SIDE).abs() < 1e-6,
            "a face sharing this edge of an un-chamfered {SIDE}³ box must reach {SIDE} across it, \
             got {extent:.4}"
        );
    }
}

/// After chamfering, each face that shared the chamfered edge must still reach
/// essentially the full width of the cube away from that edge. Being trimmed
/// down to `d` means the wrong side was kept and the bulk was thrown away.
#[test]
fn trimmed_faces_keep_their_bulk() {
    let mut offenders = Vec::new();

    for &d in &DISTANCES {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let target = target_edge(&topo, &edges);
        let ed = topo.edge(target).unwrap();
        let a = topo.vertex(ed.start()).unwrap().point();
        let b = topo.vertex(ed.end()).unwrap().point();
        let shell0 = topo.solid(solid).unwrap().outer_shell();
        let faces0 = topo.shell(shell0).unwrap().faces().to_vec();
        let planes = faces_on_edge(&topo, &faces0, target);
        assert_eq!(planes.len(), 2, "a manifold edge has 2 adjacent faces");

        let result = match chamfer_v2(&mut topo, solid, &[target], d, d) {
            Ok(r) => r.solid,
            Err(e) => {
                offenders.push(format!("d={d}: chamfer_v2 failed: {e}"));
                continue;
            }
        };

        let shell = topo.solid(result).unwrap().outer_shell();
        let faces = topo.shell(shell).unwrap().faces().to_vec();
        for plane in planes {
            let extent = plane_extent(&topo, &faces, plane, a, b);
            if extent < 0.9 * SIDE {
                offenders.push(format!(
                    "d={d}: a face sharing the chamfered edge reaches only {extent:.4} across it \
                     (the bevel is {d}) — the bulk was dropped"
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "trimming must keep the bulk of each face it trims, not the sliver next to the \
         bevel:\n{}",
        offenders.join("\n")
    );
}

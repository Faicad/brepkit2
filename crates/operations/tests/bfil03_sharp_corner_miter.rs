//! Ready-repro for B-FIL-03: fillet two ADJACENT edges of a 10×10×10 box
//! (sharing the origin vertex) with the same radius. The equal-radius pair
//! must meet at the shared sharp corner with a precise miter — no twin
//! edges, no gap, closed manifold.
//!
//! Status (2026-10-12): OPEN. Confirmed on the current tree —
//! `fillet_v2` hands back a non-manifold shell with free edges at the
//! shared corner, while the control engine (`fillet_rolling_ball`) closes
//! the same scenario. Two failure layers, from the face/wire dump:
//!
//! 1. **Stale/split contact edges.** Plane z=0 is touched by both stripes
//!    and trimmed twice; the second trim splits the first stripe's
//!    full-span contact edge (e.g. (0,2,0)→(10,2,0)) into sub-segments
//!    ((2,2,0)→(10,2,0) kept, (0,2,0)→(2,2,0) cut away). Each blend
//!    cylinder still references the full-span edge, so each contact side
//!    is use-1 (cylinder copy vs the plane's split segments). Same family
//!    as B-FIL-02, but a single-edge wire re-scan cannot fix it: no single
//!    wire edge spans the wanted contact span.
//! 2. **Naive TwoEdge corner fill.** Both cylinders stay full-length and
//!    overlap in the corner wedge; the `corner::build_two_edge_patch`
//!    triangular patch is inserted between the two end cross-section arcs
//!    and a miter line on plane z=0 — with the miter line use-1, and the
//!    far-end cap notch missing (two more free edges on the y=10 cap).
//!    The true geometry trims each cylinder against the other (they meet
//!    along a cylinder-cylinder intersection arc); implementing that
//!    miter is the remaining feature work.
//!
//! Acceptance target for the fix: `validate_shell_closed` accepts the
//! result — and the control below shows an engine in this repo already
//! achieves it, so the expectation is sound.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_math::vec::Point3;
use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::edge::EdgeId;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

const SIDE: f64 = 10.0;
const R: f64 = 2.0;

/// The x-edge (along y=0,z=0) and the y-edge (along x=0,z=0) of the box,
/// sharing the origin vertex.
fn adjacent_edges(topo: &Topology, solid: SolidId) -> Vec<EdgeId> {
    let origin = Point3::new(0.0, 0.0, 0.0);
    solid_edges(topo, solid)
        .unwrap()
        .into_iter()
        .filter(|eid| {
            let e = topo.edge(*eid).unwrap();
            let a = topo.vertex(e.start()).unwrap().point();
            let b = topo.vertex(e.end()).unwrap().point();
            let incident_origin = (a - origin).length() < 1e-9 || (b - origin).length() < 1e-9;
            incident_origin && a.z().abs() < 1e-9 && b.z().abs() < 1e-9
        })
        .collect()
}

/// Control (always green): the rolling-ball engine closes the two-edge
/// corner. Pins that the acceptance expectation below is achievable.
#[test]
fn bfil03_control_rolling_ball_closes_the_corner() {
    use brepkit_operations::fillet::fillet_rolling_ball;
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let edges = adjacent_edges(&topo, solid);
    let result = fillet_rolling_ball(&mut topo, solid, &edges, R).unwrap();
    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "control: rolling_ball must close the two-edge corner, got {validity:?}"
    );
}

/// The defect (ignored until fixed): fillet_v2 must close the same corner.
#[test]
#[ignore = "open: B-FIL-03 — fillet_v2's TwoEdge corner leaves free edges (stale split contact edges + naive miter patch)"]
fn bfil03_equal_radius_adjacent_fillets_close_the_sharp_corner() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();

    let edges = adjacent_edges(&topo, solid);
    assert_eq!(edges.len(), 2, "two adjacent box edges share the origin");

    let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "B-FIL-03: equal-radius adjacent fillets must close the sharp corner, got {validity:?}"
    );
}

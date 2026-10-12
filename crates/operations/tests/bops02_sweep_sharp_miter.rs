//! Ready-repro for B-OPS-02: sweeping a square profile along an L-shaped
//! path (two perpendicular legs) must produce a PRECISE mitered elbow — a
//! shared bisector ring at the kink, no rounding. The mitered volume is
//! exactly 5 + 5 − 1 (the shared corner cube) = 9; a rounded/naive corner
//! undercuts it (an arc corner gives 9 − (1 − π/4) ≈ 8.785).
//!
//! Status (2026-10-12): OPEN. Confirmed — the current kink-splitting
//! `sweep_miter` hands back volume 8.666667, i.e. 1/3 of material missing
//! at the junction, worse than even a rounded corner. The fix needs the
//! shared bisector-ring miter at the kink (the same feature-scale corner
//! work still open for B-FIL-03/06/08), so this stays an ignored
//! ready-repro encoding the acceptance target.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_math::nurbs::curve::NurbsCurve;
use brepkit_math::vec::Point3;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::sweep::sweep;
use brepkit_topology::Topology;

/// Profile: 1×1 square in the yz plane at x=0 (perpendicular to the +x
/// first leg): (0,0,0),(0,1,0),(0,1,1),(0,0,1).
fn square_profile_yz(topo: &mut Topology) -> brepkit_topology::face::FaceId {
    let pts = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(0.0, 0.0, 1.0),
    ];
    let wire = brepkit_topology::builder::make_polygon_wire(topo, &pts, 1e-7).unwrap();
    brepkit_topology::builder::make_planar_face_from_wire(topo, wire).unwrap()
}

/// The defect (ignored until fixed): the mitered elbow must be exact.
#[test]
#[ignore = "open: B-OPS-02 — the L-path kink junction loses 1/3 of the corner cube (measured 8.666667 vs miter target 9.0); needs the shared bisector-ring miter"]
fn bops02_l_path_sweep_miters_the_corner_precisely() {
    let mut topo = Topology::new();
    let face = square_profile_yz(&mut topo);

    // L path: +x for 5, then +z for 5 (a kink at (5,0,0)).
    let path = NurbsCurve::new(
        1,
        vec![0.0, 0.0, 0.5, 1.0, 1.0],
        vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(5.0, 0.0, 0.0),
            Point3::new(5.0, 0.0, 5.0),
        ],
        vec![1.0, 1.0, 1.0],
    )
    .unwrap();

    let solid = sweep(&mut topo, face, &path).unwrap();
    let volume = solid_volume(&topo, solid, 0.005).unwrap();

    // Mitered elbow: 10 leg volume minus the 1³ shared corner cube.
    assert!(
        (volume - 9.0).abs() < 0.05,
        "B-OPS-02: mitered elbow volume must be 9.0, got {volume:.6} (a rounded corner \
         undercuts to ~8.785; a mis-mitered corner drifts either way)"
    );

    // Positioned bbox: the profile spans y [0,1] and both legs span their
    // axes fully.
    let bbox = brepkit_operations::measure::solid_bounding_box(&topo, solid).unwrap();
    let eps = 1e-6;
    assert!((bbox.min.x()).abs() < eps && (bbox.max.x() - 5.0).abs() < eps);
    assert!((bbox.min.y()).abs() < eps && (bbox.max.y() - 1.0).abs() < eps);
    assert!((bbox.min.z()).abs() < eps && (bbox.max.z() - 5.0).abs() < eps);
}

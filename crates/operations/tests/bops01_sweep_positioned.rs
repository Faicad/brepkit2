//! Ready-repro for B-OPS-01: sweep must be POSITIONED — a profile not at
//! the path origin keeps its world offset. The registry (faijs sweep-loft)
//! reports brepkit2 re-centers the profile onto the path start, destroying
//! the offset (xmax 1.25 vs expected 2).
//!
//! Positioned invariant for a straight perpendicular path: the result bbox
//! equals the profile bbox with ONLY the path axis extended by the path
//! length; offsets in the other axes are preserved exactly.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_math::nurbs::curve::NurbsCurve;
use brepkit_math::vec::Point3;
use brepkit_operations::sweep::sweep;
use brepkit_topology::Topology;

/// Profile: a 1×1 square in the xy plane from (0.5,0) to (1.5,1).
/// Path: straight line along +z from (0,0,0), length 3.
///
/// Positioned sweep bbox: x [0.5, 1.5], y [0, 1], z [0, 3].
/// Re-centered sweep (the bug) moves the centroid (1, 0.5, 0) onto the
/// path start, yielding x [−0.5, 0.5] — the offset is destroyed.
#[test]
fn bops01_straight_sweep_keeps_profile_offset() {
    let mut topo = Topology::new();

    // Build the square profile face from (0.5,0,0) to (1.5,1,0).
    let pts = [
        Point3::new(0.5, 0.0, 0.0),
        Point3::new(1.5, 0.0, 0.0),
        Point3::new(1.5, 1.0, 0.0),
        Point3::new(0.5, 1.0, 0.0),
    ];
    let wire = brepkit_topology::builder::make_polygon_wire(&mut topo, &pts, 1e-7).unwrap();
    let face = brepkit_topology::builder::make_planar_face_from_wire(&mut topo, wire).unwrap();

    // Straight path along +z: control points (0,0,0) → (0,0,3).
    let path = NurbsCurve::new(
        1,
        vec![0.0, 0.0, 1.0, 1.0],
        vec![Point3::new(0.0, 0.0, 0.0), Point3::new(0.0, 0.0, 3.0)],
        vec![1.0, 1.0],
    )
    .unwrap();

    let solid = sweep(&mut topo, face, &path).unwrap();

    let bbox = brepkit_operations::measure::solid_bounding_box(&topo, solid).unwrap();

    let eps = 1e-6;
    assert!(
        (bbox.min.x() - 0.5).abs() < eps && (bbox.max.x() - 1.5).abs() < eps,
        "B-OPS-01: x span must keep the profile offset [0.5, 1.5], got [{}, {}]",
        bbox.min.x(),
        bbox.max.x()
    );
    assert!(
        (bbox.min.y() - 0.0).abs() < eps && (bbox.max.y() - 1.0).abs() < eps,
        "B-OPS-01: y span must keep the profile span [0, 1], got [{}, {}]",
        bbox.min.y(),
        bbox.max.y()
    );
    assert!(
        (bbox.min.z() - 0.0).abs() < eps && (bbox.max.z() - 3.0).abs() < eps,
        "B-OPS-01: z span must be the path extent [0, 3], got [{}, {}]",
        bbox.min.z(),
        bbox.max.z()
    );
}

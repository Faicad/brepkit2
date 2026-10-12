//! Regression test for B-OPS-04: extruding a CLOCKWISE-wound profile must
//! rewind it before building, yielding an outward solid with the correct
//! volume.
//!
//! Provenance (2026-10-12): the registry claimed CW profiles produce an
//! inside-out solid. It does not reproduce — the extrude path already
//! rewinds mis-wound profiles (see `crate::winding`), and the CW square
//! below extrudes to volume 3 with Inside/Outside classification correct.
//! Pinned green so a winding regression cannot slip back in.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_check::classify::{ClassifyOptions, PointClassification, classify_point};
use brepkit_math::vec::{Point3, Vec3};
use brepkit_operations::extrude::extrude;
use brepkit_topology::Topology;
use brepkit_topology::builder::{make_planar_face_from_wire, make_polygon_wire};

#[test]
fn bops04_cw_profile_extrude_is_outward_and_correct() {
    let mut topo = Topology::new();

    // CLOCKWISE square in the xy plane (viewed from +z): (0,0)→(0,1)→(1,1)→(1,0).
    let cw = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let wire = make_polygon_wire(&mut topo, &cw, 1e-7).unwrap();
    let face = make_planar_face_from_wire(&mut topo, wire).unwrap();

    let solid = extrude(&mut topo, face, Vec3::new(0.0, 0.0, 1.0), 3.0).unwrap();

    // Volume must be area × height = 3.
    let volume = brepkit_operations::measure::solid_volume(&topo, solid, 0.005).unwrap();
    assert!(
        (volume - 3.0).abs() < 0.02,
        "B-OPS-04: CW profile extrude volume must be 3, got {volume}"
    );

    // Orientation: centre Inside, above the top face Outside.
    let opts = ClassifyOptions::default();
    assert_eq!(
        classify_point(&topo, solid, Point3::new(0.5, 0.5, 1.5), &opts).unwrap(),
        PointClassification::Inside,
        "B-OPS-04: CW-extruded solid centre must be Inside"
    );
    assert_eq!(
        classify_point(&topo, solid, Point3::new(0.5, 0.5, 4.0), &opts).unwrap(),
        PointClassification::Outside,
        "B-OPS-04: point above the CW-extruded solid must be Outside"
    );
}

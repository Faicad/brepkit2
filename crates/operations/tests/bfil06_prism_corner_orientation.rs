//! Ready-repro for B-FIL-06: fillet ALL edges of a triangular prism whose
//! corner angles are NOT 90°. Every face of the result must be consistently
//! oriented (closed shell accepted by `validate_shell_closed`; centre
//! Inside, exterior point Outside).
//!
//! Status (2026-10-12): OPEN, two layers.
//!
//! Fixed this session (see blend commits): (1) `compute_setbacks` dropped a
//! setback that exceeded the per-end budget outright, letting stripes run
//! full-length into acute-angle corners — it now clamps to the budget;
//! (2) contact adoption re-scans the trimmed neighbour's wire and adopts the
//! surviving sub-segment CHAIN when a later stripe's re-trim split the
//! recorded contact edge.
//!
//! Remaining gap: at an acute corner (e.g. the prism's 36.9° corner at
//! (4,0,0)) no corner sphere exists — the point at distance r from all
//! three faces lies OUTSIDE the material (x = −2.4 for r = 0.8 here) — so
//! the spherical corner patch cannot close the shell. The true geometry
//! trims the stripe cylinders against each other along their
//! cylinder-cylinder intersection arcs (the x-edge and hypotenuse stripe
//! axes cross at (1.6, 0.8, 0.8)); implementing that mutual trim is the
//! same feature-scale miter work still open for B-FIL-03.
//!
//! The box control below is always green: the orthogonal case closes and
//! orients correctly, so the acceptance expectation is sound.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_check::classify::{ClassifyOptions, PointClassification, classify_point};
use brepkit_math::vec::{Point3, Vec3};
use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::extrude::extrude;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::builder::{make_planar_face_from_wire, make_polygon_wire};
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

const R: f64 = 0.8;

/// A triangular prism: right triangle (0,0,0),(4,0,0),(0,3,0) extruded
/// along +z by 5. Corner angles 90°, ~53°, ~37° — the acute corners do not
/// exist in a plain box.
fn make_triangular_prism(topo: &mut Topology) -> SolidId {
    let pts = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(4.0, 0.0, 0.0),
        Point3::new(0.0, 3.0, 0.0),
    ];
    let wire = make_polygon_wire(topo, &pts, 1e-7).unwrap();
    let face = make_planar_face_from_wire(topo, wire).unwrap();
    extrude(topo, face, Vec3::new(0.0, 0.0, 1.0), 5.0).unwrap()
}

/// The defect (ignored until fixed): a full prism fillet must close and
/// orient consistently.
#[test]
#[ignore = "open: B-FIL-06 — acute prism corners have no corner sphere; stripe cylinders must be trimmed against each other (cylinder-cylinder miter)"]
fn bfil06_prism_corner_fillet_is_consistently_oriented() {
    let mut topo = Topology::new();
    let solid = make_triangular_prism(&mut topo);

    // Fillet every edge of the prism.
    let edges = solid_edges(&topo, solid).unwrap();
    assert_eq!(edges.len(), 9, "a triangular prism has 9 edges");

    let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "B-FIL-06: prism corner fillet must close, got {validity:?}"
    );

    // Orientation: centre of the eroded prism is material; a point outside
    // the original prism is void.
    let opts = ClassifyOptions::default();
    let centre = Point3::new(1.0, 0.8, 2.5);
    assert_eq!(
        classify_point(&topo, result, centre, &opts).unwrap(),
        PointClassification::Inside,
        "B-FIL-06: prism centre must classify Inside (outward orientation)"
    );
    let outside = Point3::new(-1.0, -1.0, 2.5);
    assert_eq!(
        classify_point(&topo, result, outside, &opts).unwrap(),
        PointClassification::Outside,
        "B-FIL-06: point off the prism must classify Outside (outward orientation)"
    );
}

/// Box control (all 12 edges): the orthogonal case is the baseline the
/// registry's "8 corners" refers to; it must stay green.
#[test]
fn bfil06_box_control_all_edges_fillet_is_consistently_oriented() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    let result = fillet_v2(&mut topo, solid, &edges, 2.0).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    assert!(validate_shell_closed(topo.shell(shell).unwrap(), &topo).is_ok());

    let opts = ClassifyOptions::default();
    assert_eq!(
        classify_point(&topo, result, Point3::new(5.0, 5.0, 5.0), &opts).unwrap(),
        PointClassification::Inside
    );
    assert_eq!(
        classify_point(&topo, result, Point3::new(5.0, 15.0, 5.0), &opts).unwrap(),
        PointClassification::Outside
    );
}

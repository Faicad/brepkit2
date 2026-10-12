//! Regression test for B-OPS-03: the bbox of a solid with a TRIMMED
//! cylinder face must follow the trimming edges, not the cylinder's full
//! analytic range.
//!
//! Provenance (2026-10-12): the registry claimed trimmed cylinder/cone
//! faces expand the bbox to the full analytic range (a half cylinder would
//! report y ∈ [-5, 5]). It does not reproduce: `measure/bounding_box.rs`
//! already bounds cylinders/cones by wire-bounded expansion
//! (`expand_trimmed_revolution`), and the half-cylinder bbox below reads
//! y ∈ [0, 5] exactly. The x span reads 4.99695 instead of 5.0 — that is
//! the vertex/midpoint sampling of the polygonised rim (a ~0.06% chord
//! undercount inherent to the sampled bbox), not a full-range expansion,
//! and is asserted only within tolerance here.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_math::mat::Mat4;
use brepkit_operations::boolean::{BooleanOp, boolean};
use brepkit_operations::measure::solid_bounding_box;
use brepkit_operations::primitives::{make_box, make_cylinder};
use brepkit_operations::transform::transform_solid;
use brepkit_topology::Topology;

#[test]
fn bops03_half_cylinder_bbox_follows_the_trim() {
    let mut topo = Topology::new();
    let cyl = make_cylinder(&mut topo, 5.0, 10.0).unwrap();
    // Cutter: everything below y = 0 (a box from (-20,-20,-1) to (20,0,11)).
    let knife = make_box(&mut topo, 40.0, 20.0, 12.0).unwrap();
    transform_solid(&mut topo, knife, &Mat4::translation(-20.0, -20.0, -1.0)).unwrap();
    let half = boolean(&mut topo, BooleanOp::Cut, cyl, knife).unwrap();

    let bbox = solid_bounding_box(&topo, half).unwrap();
    let eps = 1e-6;
    // The trim is honoured exactly: no full analytic range below y = 0.
    assert!(
        (bbox.min.y() - 0.0).abs() < eps,
        "B-OPS-03: half-cylinder bbox must start at y=0 (trimmed), got {}",
        bbox.min.y()
    );
    assert!(
        (bbox.max.y() - 5.0).abs() < 1e-3,
        "B-OPS-03: half-cylinder bbox must reach y=5 (the arc apex), got {}",
        bbox.max.y()
    );
    // x span is the full diameter within the sampled-bbox chord tolerance.
    assert!(
        (bbox.min.x() + 5.0).abs() < 0.01 && (bbox.max.x() - 5.0).abs() < 0.01,
        "B-OPS-03: half-cylinder x span must be ~[-5, 5], got [{}, {}]",
        bbox.min.x(),
        bbox.max.x()
    );
    // z span is the full height (the trim only affects y).
    assert!((bbox.min.z()).abs() < eps && (bbox.max.z() - 10.0).abs() < 1e-3);
}

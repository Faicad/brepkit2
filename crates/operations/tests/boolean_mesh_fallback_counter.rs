//! B-06: the boolean pipeline silently degrades to the mesh (co-refinement)
//! fallback when GFA cannot produce a usable result, and callers have no way
//! to observe it.
//!
//! The mesh fallback loses all analytic surface types (planes become triangle
//! soups, cylinders/tori disappear), so "did this boolean fall back?" is the
//! single most important quality signal for the GFA engine. Before this file
//! existed there was no counter anywhere in `crates/` — the degradation was
//! visible only as a `log::debug!` line, i.e. invisible in production and
//! unassertable in tests.
//!
//! This file holds exactly ONE `#[test]` on purpose: the counter is
//! process-global, so a second test running concurrently in the same binary
//! could bump it and turn a deterministic assertion flaky. Integration tests
//! are separate binaries, so other files cannot interfere.

#![allow(clippy::unwrap_used)]

use brepkit_math::mat::Mat4;
use brepkit_operations::boolean::{
    BooleanOp, boolean, mesh_fallback_count, reset_mesh_fallback_count,
};
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;

#[test]
fn mesh_fallback_is_observable() {
    let _ = env_logger::try_init();
    let mut topo = Topology::new();

    // ── Control case: an analytic fuse must NOT fall back ──────────────
    // Two overlapping boxes is the configuration GFA handles natively.
    reset_mesh_fallback_count();
    let a = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
    let b = make_box(&mut topo, 6.0, 6.0, 6.0).unwrap();
    brepkit_operations::transform::transform_solid(&mut topo, b, &Mat4::translation(5.0, 5.0, 5.0))
        .unwrap();
    let fused = boolean(&mut topo, BooleanOp::Fuse, a, b).unwrap();
    let vol = brepkit_operations::measure::solid_volume(&topo, fused, 0.01).unwrap();
    // Analytic: 10³ + 6³ − overlap. A spans [0,10]³, B spans [5,11]³, so the
    // overlap is 5×5×5 = 125 → 1000 + 216 − 125 = 1091.
    assert!(
        (vol - 1091.0).abs() < 1.0,
        "overlapping-box fuse volume should be ≈1091, got {vol}"
    );
    assert_eq!(
        mesh_fallback_count(),
        0,
        "an analytic fuse must not degrade to the mesh fallback"
    );

    // ── Main case: two crossing cylinders DO fall back ─────────────────
    // Measured with the counter itself: a cylinder×cylinder fuse currently
    // defeats GFA (the quadric×quadric section has no analytic path here) and
    // degrades to co-refinement. That is both the proof that the counter
    // fires, and a live regression baseline for the A-domain: when the
    // analytic section lands, this assertion flips to 0 and must be updated.
    reset_mesh_fallback_count();
    let cyl_a = brepkit_operations::primitives::make_cylinder(&mut topo, 5.0, 40.0).unwrap();
    let cyl_b = brepkit_operations::primitives::make_cylinder(&mut topo, 5.0, 40.0).unwrap();
    brepkit_operations::transform::transform_solid(
        &mut topo,
        cyl_b,
        &Mat4::rotation_x(std::f64::consts::FRAC_PI_2),
    )
    .unwrap();
    let _cross = boolean(&mut topo, BooleanOp::Fuse, cyl_a, cyl_b).unwrap();

    assert_eq!(
        mesh_fallback_count(),
        1,
        "the crossing-cylinder fuse degrades to the mesh fallback; \
         the counter must observe it (this is the whole point of B-06)"
    );

    // ── Counter is readable as a delta too ─────────────────────────────
    let before = mesh_fallback_count();
    let c = make_box(&mut topo, 4.0, 4.0, 4.0).unwrap();
    let d = make_box(&mut topo, 4.0, 4.0, 4.0).unwrap();
    brepkit_operations::transform::transform_solid(&mut topo, d, &Mat4::translation(2.0, 2.0, 2.0))
        .unwrap();
    let _ = boolean(&mut topo, BooleanOp::Fuse, c, d).unwrap();
    assert_eq!(
        mesh_fallback_count(),
        before,
        "an analytic fuse must not bump the counter (delta-style read)"
    );
}

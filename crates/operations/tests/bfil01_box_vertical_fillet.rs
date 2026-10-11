//! Regression test for B-FIL-01: fillet the four vertical edges of a
//! 10×10×10 box with r=2. The result must be an outward solid,
//! `validate_shell_closed` must accept it, and the volume must match
//! 1000 − 4·10·r²(1 − π/4).
//!
//! Provenance (2026-10-12): the registry claimed brepkit2 flips this
//! fillet inside-out (`isValid=false`, volume ≈968.58). Neither reproduces
//! on the current tree: the shell is a closed manifold, the centre and
//! above-face probes classify Inside/Outside, and the volume matches the
//! closed form (the registry's "1000 − 40(1 − π/4) ≈ 968.58" is itself
//! arithmetically wrong — that expression equals 991.42, and the correct
//! closed form is 1000 − 160(1 − π/4) ≈ 965.66, which the engine meets).
//! This test pins the acceptance target permanently.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use brepkit_check::classify::{ClassifyOptions, PointClassification, classify_point};
use brepkit_math::vec::Point3;
use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

const SIDE: f64 = 10.0;
const R: f64 = 2.0;
const TOL: f64 = 1e-3;

/// 4 vertical edges, each removing an r²(1 − π/4) cross-section over the
/// full 10-unit height: 1000 − 4·10·r²(1 − π/4) = 1000 − 160(1 − π/4).
/// (The registry's "1000 − 40(1 − π/4) ≈ 968.58" is arithmetically wrong;
/// that expression equals 991.42, not 968.58.)
fn expected_volume() -> f64 {
    1000.0 - 160.0 * (1.0 - std::f64::consts::FRAC_PI_4)
}

/// The four vertical edges: both endpoints share the same (x, y).
fn vertical_edges(topo: &Topology, solid: SolidId) -> Vec<brepkit_topology::edge::EdgeId> {
    solid_edges(topo, solid)
        .unwrap()
        .into_iter()
        .filter(|eid| {
            let edge = topo.edge(*eid).unwrap();
            let p0 = topo.vertex(edge.start()).unwrap().point();
            let p1 = topo.vertex(edge.end()).unwrap().point();
            (p0.x() - p1.x()).abs() < TOL && (p0.y() - p1.y()).abs() < TOL
        })
        .collect()
}

/// The acceptance target for the fixed engine.
#[test]
fn bfil01_box_vertical_fillet_is_outward_and_valid() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();

    let edges = vertical_edges(&topo, solid);
    assert_eq!(edges.len(), 4, "a box has exactly 4 vertical edges");

    let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "B-FIL-01: filleted box must be a closed manifold, got {:?}",
        validity.err()
    );

    let volume = solid_volume(&topo, result, 0.05).unwrap();
    let expected = expected_volume();
    assert!(
        (volume - expected).abs() < 0.05,
        "B-FIL-01: volume {volume} must match closed form {expected}"
    );

    // Orientation: the solid must be outward, i.e. a point just above the
    // top face (y = SIDE) is outside and the centre is inside.
    let centre = Point3::new(SIDE / 2.0, SIDE / 2.0, SIDE / 2.0);
    let above = Point3::new(SIDE / 2.0, SIDE + 1.0, SIDE / 2.0);
    let opts = ClassifyOptions::default();
    assert_eq!(
        classify_point(&topo, result, centre, &opts).unwrap(),
        PointClassification::Inside,
        "B-FIL-01: solid centre must classify Inside (outward orientation)"
    );
    assert_eq!(
        classify_point(&topo, result, above, &opts).unwrap(),
        PointClassification::Outside,
        "B-FIL-01: point above the top face must classify Outside (outward orientation)"
    );
}

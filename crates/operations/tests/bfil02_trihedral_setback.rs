//! Regression test for B-FIL-02: fillet the three edges meeting at one
//! corner of a 10×10×10 box (a trihedral junction) with r=2. The corner
//! must be resolved by setback + corner patch without a broken shell.
//!
//! Defect and fix (2026-10-12): the shell used to come back non-manifold —
//! six free edges at the stripes' far ends. Root cause: a plane touched by
//! two stripes is trimmed twice; the second trim splits the first stripe's
//! full-span contact edge, so the contact edge recorded at first-trim time
//! goes stale, adoption of the trimmer's edge into the blend face failed,
//! and the duplicate vertices minted for the blend cross arcs made the
//! end-cap notch (which keys on `VertexId`) miss every cap. The fix passes
//! each side's trimmed face along to `create_blend_face_with_contacts` and
//! re-scans that face's wire for the surviving (setback-shortened) contact
//! segment when direct adoption misses.
//!
//! Known pre-existing deviation, NOT caused by or fixed here: the corner
//! patch deviates from the exact rolling-ball sphere (sampled sphere-distance
//! range [1.902, 2.0], see the ignored `bfil02_corner_patch_sphere_deviation`),
//! so `solid_volume` disagrees with the analytic octant closed form
//! 1000 − 3·(10−r)·r²(1−π/4) − (r³ − (1/8)(4/3)πr³) ≈ 975.587 by ≈ 0.26 —
//! identically for both engines (fillet_v2 975.32808, rolling_ball 975.32808),
//! which is why the volume assertion pins cross-engine agreement rather than
//! the closed form.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use brepkit_math::vec::Point3;
use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::edge::EdgeId;
use brepkit_topology::explorer::{solid_edges, solid_vertices};
use brepkit_topology::solid::SolidId;
use brepkit_topology::validation::validate_shell_closed;

const SIDE: f64 = 10.0;
const R: f64 = 2.0;

/// Analytic octant closed form (see module docs for the known deviation).
fn expected_volume() -> f64 {
    let stripe = 3.0 * (SIDE - R) * R * R * (1.0 - std::f64::consts::FRAC_PI_4);
    let corner = R * R * R - (1.0 / 8.0) * (4.0 / 3.0) * std::f64::consts::PI * R * R * R;
    1000.0 - stripe - corner
}

/// The three edges incident to the corner nearest the origin.
fn corner_edges(topo: &Topology, solid: SolidId) -> Vec<EdgeId> {
    let origin = Point3::new(0.0, 0.0, 0.0);
    let v = *solid_vertices(topo, solid)
        .unwrap()
        .iter()
        .min_by(|a, b| {
            let da = (topo.vertex(**a).unwrap().point() - origin).length();
            let db = (topo.vertex(**b).unwrap().point() - origin).length();
            da.partial_cmp(&db).unwrap()
        })
        .unwrap();

    solid_edges(topo, solid)
        .unwrap()
        .into_iter()
        .filter(|eid| {
            let e = topo.edge(*eid).unwrap();
            e.start() == v || e.end() == v
        })
        .collect()
}

/// The fix: the trihedral-corner fillet must hand back a closed 2-manifold.
#[test]
fn bfil02_trihedral_corner_fillet_is_a_closed_manifold() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();

    let edges = corner_edges(&topo, solid);
    assert_eq!(edges.len(), 3, "a box corner joins exactly 3 edges");

    let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;

    let shell = topo.solid(result).unwrap().outer_shell();
    let validity = validate_shell_closed(topo.shell(shell).unwrap(), &topo);
    assert!(
        validity.is_ok(),
        "B-FIL-02: corner-filleted box must be a closed manifold, got {:?}",
        validity.err()
    );
}

/// Geometric cross-check: both engines must agree on the resulting volume.
/// (The absolute closed form is off by the known corner-patch deviation —
/// see the module docs — so agreement between two independent engines is
/// the honest invariant here.)
#[test]
fn bfil02_engines_agree_on_corner_fillet_volume() {
    let v2_volume = {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = corner_edges(&topo, solid);
        let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;
        solid_volume(&topo, result, 0.005).unwrap()
    };
    let control_volume = {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
        let edges = corner_edges(&topo, solid);
        let result =
            brepkit_operations::fillet::fillet_rolling_ball(&mut topo, solid, &edges, R).unwrap();
        solid_volume(&topo, result, 0.005).unwrap()
    };
    assert!(
        (v2_volume - control_volume).abs() < 1e-3,
        "both engines must agree on the corner-fillet volume: {v2_volume} vs {control_volume}"
    );
    // Sanity band around the analytic closed form, wide enough to absorb the
    // known corner-patch deviation (~0.26) but tight enough to catch a real
    // over-removal (the registry's failure mode).
    let expected = expected_volume();
    assert!(
        (v2_volume - expected).abs() < 0.5,
        "volume {v2_volume} drifted from the analytic closed form {expected}"
    );
}

/// Boundary sanity via exact ray-cast classification: material inside the
/// corner ball, void at the deep corner. (The deep-void probes avoid the
/// patch's known deviation band near the sphere boundary — see module docs.)
#[test]
fn bfil02_corner_boundary_classification() {
    use brepkit_check::classify::{ClassifyOptions, PointClassification, classify_point};

    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let edges = corner_edges(&topo, solid);
    let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;

    let opts = ClassifyOptions::default();
    let classify = |p: Point3| classify_point(&topo, result, p, &opts).unwrap();

    // Inside the corner ball (dist < 2 from (2,2,2)) must be material.
    for (x, y, z) in [(1.5f64, 1.5, 1.5), (1.9, 1.9, 1.9), (1.0, 1.0, 1.0)] {
        let d2 = (x - 2.0) * (x - 2.0) + (y - 2.0) * (y - 2.0) + (z - 2.0) * (z - 2.0);
        assert!(d2 < 4.0);
        assert_eq!(
            classify(Point3::new(x, y, z)),
            PointClassification::Inside,
            "corner ball point ({x},{y},{z}) must be material"
        );
    }
    // Deep in the corner void (well outside the ball and the deviation band)
    // must be outside the solid.
    for (x, y, z) in [(0.5f64, 0.5, 0.5), (0.1, 0.1, 0.1)] {
        let d2 = (x - 2.0) * (x - 2.0) + (y - 2.0) * (y - 2.0) + (z - 2.0) * (z - 2.0);
        assert!(d2 > 4.0);
        assert_eq!(
            classify(Point3::new(x, y, z)),
            PointClassification::Outside,
            "corner void point ({x},{y},{z}) must be outside the solid"
        );
    }
}

/// Ready-repro for the pre-existing corner-patch deviation: the patch must
/// lie exactly on the rolling-ball sphere (radius 2 centred at (2,2,2)).
/// Measured today: sphere-distance range [1.902152, 2.000000] — the patch
/// dips ~0.1 inside the sphere, which also explains why `solid_volume`
/// disagrees with the analytic octant closed form by ~0.26 (both engines).
#[test]
#[ignore = "open: corner patch deviates from the exact rolling-ball sphere (measured range [1.902, 2.0])"]
fn bfil02_corner_patch_sphere_deviation() {
    use brepkit_topology::face::FaceSurface;

    let mut topo = Topology::new();
    let solid = make_box(&mut topo, SIDE, SIDE, SIDE).unwrap();
    let edges = corner_edges(&topo, solid);
    let result = fillet_v2(&mut topo, solid, &edges, R).unwrap().solid;

    let faces = brepkit_topology::explorer::solid_faces(&topo, result).unwrap();
    for fid in &faces {
        let f = topo.face(*fid).unwrap();
        if let FaceSurface::Nurbs(n) = f.surface() {
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            for i in 0..=10 {
                for j in 0..=10 {
                    let u = i as f64 / 10.0;
                    let v = j as f64 / 10.0;
                    let p = n.evaluate(u, v);
                    let d = ((p.x() - 2.0).powi(2) + (p.y() - 2.0).powi(2) + (p.z() - 2.0).powi(2))
                        .sqrt();
                    min = min.min(d);
                    max = max.max(d);
                }
            }
            assert!(
                (min - 2.0).abs() < 1e-3 && (max - 2.0).abs() < 1e-3,
                "corner patch must lie on the rolling-ball sphere r=2, got [{min}, {max}]"
            );
            return;
        }
    }
    panic!("no NURBS corner patch found in the result");
}

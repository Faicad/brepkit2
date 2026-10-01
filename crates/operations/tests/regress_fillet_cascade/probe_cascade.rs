//! Known-failure regression test for C-01.
//! Reproduces the `try_fillet` cascade in operations-testable form.
//!
//! This test is **expected to fail** until C-01 is fixed; the failing assertion
//! is the bug ticket. It deliberately runs all three engines and reports where
//! each one lands before asserting, so the cascade's behaviour stays visible in
//! the test output while the bug is open.
//!
//! Runs as a module of `regress_fillet_cascade/main.rs`; the shared lint
//! levels for this target live in that root file.

use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::fillet::{fillet, fillet_rolling_ball};
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::{solid_edges, solid_faces};
use brepkit_topology::face::FaceSurface;
use brepkit_topology::validation::validate_shell_closed;
use std::fmt::Write;

fn is_valid(topo: &Topology, s: brepkit_topology::solid::SolidId) -> bool {
    topo.solid(s)
        .and_then(|sd| topo.shell(sd.outer_shell()))
        .map(|sh| validate_shell_closed(sh, topo).is_ok())
        .unwrap_or(false)
}

#[test]
#[ignore = "C-01: try_fillet cascade ends on the deprecated bevel engine; unfixed"]
fn cascade_at_tangency() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 5.0, 5.0, 20.0).unwrap();
    let edges = solid_edges(&topo, solid).unwrap();
    // The per-engine trace is accumulated instead of printed: `cargo test`
    // only surfaces assertion text, so the trace rides along in the failure
    // message below.
    let mut cur = solid;
    let mut trace = String::new();
    writeln!(trace, "edges={}", edges.len()).unwrap();

    if let Ok(s) = fillet_rolling_ball(&mut topo, solid, &edges, 2.5) {
        let valid = is_valid(&topo, s);
        writeln!(trace, "1 rolling_ball: ok valid={valid}").unwrap();
        cur = s;
    } else {
        writeln!(trace, "1 rolling_ball: rejected").unwrap();
    }
    if let Ok(r) = fillet_v2(&mut topo, solid, &edges, 2.5) {
        let valid = is_valid(&topo, r.solid);
        writeln!(trace, "2 fillet_v2: ok valid={valid}").unwrap();
        if valid {
            cur = r.solid;
        }
    } else {
        writeln!(trace, "2 fillet_v2: rejected").unwrap();
    }
    if let Ok(s) = fillet(&mut topo, solid, &edges, 2.5) {
        let valid = is_valid(&topo, s);
        writeln!(trace, "3 bevel-fillet: ok valid={valid}").unwrap();
        if valid {
            cur = s;
        }
    } else {
        writeln!(trace, "3 bevel-fillet: rejected").unwrap();
    }
    let faces = solid_faces(&topo, cur).unwrap();
    let vol = solid_volume(&topo, cur, 0.05).unwrap();
    writeln!(
        trace,
        "FINAL faces={} vol={:.4} analytic=359.9742",
        faces.len(),
        vol
    )
    .unwrap();

    // ── KNOWN FAILURE (C-01) ─────────────────────────────────────────
    // The cascade must land on the capsule: `segment core of length 15`
    // swept by a ball of radius 2.5. Every engine currently falls short, so
    // this assert is the bug ticket, not a regression guard.
    let capsule = 15.0 * std::f64::consts::PI * 2.5 * 2.5
        + (4.0 / 3.0) * std::f64::consts::PI * 2.5f64.powi(3);
    assert!(
        (vol - capsule).abs() < 0.5,
        "cascade trace:\n{trace}\nthe fillet cascade must return the capsule (volume {capsule:.4}), \
         got {vol:.4} with {} faces — C-01 unfixed",
        faces.len()
    );

    // The result must carry analytic surfaces; a flat bevel or a mesh fallback
    // leaves only planes (or a triangle soup).
    let mut analytic = 0usize;
    for &fid in &faces {
        if matches!(
            topo.face(fid).unwrap().surface(),
            FaceSurface::Cylinder { .. }
                | FaceSurface::Cone { .. }
                | FaceSurface::Sphere { .. }
                | FaceSurface::Torus { .. }
        ) {
            analytic += 1;
        }
    }
    assert!(
        analytic >= 1,
        "the capsule needs at least a cylindrical band; {} of {} faces are analytic — bevel/mesh fallback",
        analytic,
        faces.len()
    );
}

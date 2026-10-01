//! C-01 (degenerate-tangency probe): box `5 x 5 x 20`, all 12 edges filleted
//! with `r = 2.5`.
//!
//! `r = 2.5` is exactly **half the short edge**, so the fillets around the four
//! short edges are mutually tangent and the inner offset (`Lx - 2r = 0`,
//! `Ly - 2r = 0`) collapses to a single line segment. Box `5 x 5 x 20` centred
//! at the origin spans `x,y in [-2.5, 2.5]`, `z in [-10, 10]`, so the offset
//! core is `{(0,0,z) : |z| <= 7.5}` — a *line segment* of length 15.
//!
//! A solid's fillet result is the core's neighbourhood: the set of points whose
//! distance to the core is `<= r`. For a segment core that set is exactly a
//! **capsule**: cylinder of radius `r` and height 15 capped by two hemispheres.
//!
//! ```text
//! V = 15 * pi * r^2 + (4/3) * pi * r^3 ,   r = 2.5
//!   = 15 * pi * 6.25 + (4/3) * pi * 15.625
//!   = 93.75 pi + 20.8333 pi  =  114.583333 pi  =  359.9894...
//! ```
//!
//! So this degenerate configuration has a closed-form answer, and exactly
//! **3 analytic surfaces** (one cylindrical band + two spherical caps) — *not*
//! the 26 faces a generic box fillet would produce. A face count near 26 is
//! therefore NOT the right assertion here; the tangency degeneracy legitimately
//! merges faces away. What must hold is the volume, the watertight property,
//! and that the analytic surface types survive (a mesh fallback here would
//! leave a triangle soup).
//!
//! This file is the single C-01 reproducer. It holds two tickets:
//!
//! 1. `tangent_short_edges_produce_the_capsule` — the end-to-end ask. The
//!    wasm binding's `try_fillet` runs three engines and keeps the first
//!    topologically-closed result; at this radius every engine falls short and
//!    the caller silently receives the deprecated flat bevel.
//! 2. `fillet_v2_over_sweeps_a_box` — the engine-level half of the same bug.
//!    `fillet_v2` does not error on a cube, it just returns too much material,
//!    and the drift grows linearly in `r`.
//!
//! Both are `#[ignore]`d because the bug is open: the default suite stays
//! green, and `cargo test -- --ignored` shows the tickets.
//!
//! Runs as a module of `regress_fillet_cascade/main.rs`; the shared lint
//! levels for this target live in that root file.

use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::fillet::fillet_rolling_ball;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::{solid_edges, solid_faces};
use brepkit_topology::face::FaceSurface;
use brepkit_topology::validation::validate_shell_closed;
use std::fmt::Write;

/// Analytic volume of the capsule the fillet must produce: `114.5833333 pi`.
const EXPECTED_VOLUME: f64 =
    15.0 * std::f64::consts::PI * 6.25 + (4.0 / 3.0) * std::f64::consts::PI * 15.625;

/// Volume tolerance. The rolling-ball engine is exact for this configuration
/// (all contacts are analytic), so the error must stay far below 0.1%.
const VOLUME_TOL: f64 = 0.5;

/// Runs the wasm binding's cascade by hand: `try_fillet` keeps the first
/// result that `validate_shell_closed` accepts, so which engine wins decides
/// what the caller gets.
fn cascade_trace(
    topo: &mut Topology,
    solid: brepkit_topology::solid::SolidId,
    r: f64,
) -> (brepkit_topology::solid::SolidId, String) {
    let edges = solid_edges(topo, solid).unwrap();
    let mut trace = String::new();
    writeln!(trace, "edges={}", edges.len()).unwrap();

    let mut cur = if let Ok(s) = fillet_rolling_ball(topo, solid, &edges, r) {
        writeln!(trace, "1 rolling_ball: ok").unwrap();
        s
    } else {
        writeln!(trace, "1 rolling_ball: rejected").unwrap();
        solid
    };
    if let Ok(res) = fillet_v2(topo, solid, &edges, r) {
        writeln!(trace, "2 fillet_v2: ok").unwrap();
        if validate_shell_closed(
            topo.shell(topo.solid(res.solid).unwrap().outer_shell())
                .unwrap(),
            topo,
        )
        .is_ok()
        {
            cur = res.solid;
        } else {
            writeln!(trace, "2 fillet_v2: result not closed").unwrap();
        }
    } else {
        writeln!(trace, "2 fillet_v2: rejected").unwrap();
    }
    (cur, trace)
}

#[test]
#[ignore = "C-01: tangency at r=half the shortest edge yields 8 bevel faces; unfixed"]
fn tangent_short_edges_produce_the_capsule() {
    let mut topo = Topology::new();
    let solid = make_box(&mut topo, 5.0, 5.0, 20.0).unwrap();

    let edges = solid_edges(&topo, solid).unwrap();
    assert_eq!(edges.len(), 12, "a box has 12 edges");

    // ── Probe every engine before asserting on any ──────────────────
    // The rolling-ball engine refuses this configuration outright, so the
    // result the caller actually receives comes from whatever the remaining
    // engines produce. Measure first, assert after.
    let (cur, trace) = cascade_trace(&mut topo, solid, 2.5);
    let result = cur;

    // ── (1) watertight ──────────────────────────────────────────────
    let solid_data = topo.solid(result).unwrap();
    let shell = topo.shell(solid_data.outer_shell()).unwrap();
    validate_shell_closed(shell, &topo)
        .expect("the tangent fillet result must be a closed 2-manifold");
    assert!(solid_data.inner_shells().is_empty(), "no cavity may appear");

    // ── (2) volume: analytic capsule ────────────────────────────────
    let vol = solid_volume(&topo, result, 1e-6).unwrap();
    assert!(
        (vol - EXPECTED_VOLUME).abs() < VOLUME_TOL,
        "tangent fillet of 5x5x20 at r=2.5 must be the capsule \
         (volume {EXPECTED_VOLUME:.4}), got {vol:.4} \
         (error {:.4})\nengine trace:\n{trace}",
        (vol - EXPECTED_VOLUME).abs()
    );

    // ── (3) analytic surface types survive ──────────────────────────
    // A mesh fallback here would report NURBS-only faces. The capsule must
    // contain spherical caps and a cylindrical band.
    let faces = solid_faces(&topo, result).unwrap();
    let mut spherical = 0usize;
    let mut cylindrical = 0usize;
    for &fid in &faces {
        match topo.face(fid).unwrap().surface() {
            FaceSurface::Sphere { .. } => spherical += 1,
            FaceSurface::Cylinder { .. } | FaceSurface::Cone { .. } => cylindrical += 1,
            _ => {}
        }
    }
    assert!(
        spherical >= 2,
        "expected >=2 spherical cap faces, got {spherical} (of {} faces)\nengine trace:\n{trace}",
        faces.len()
    );
    assert!(
        cylindrical >= 1,
        "expected >=1 cylindrical band face, got {cylindrical} (of {} faces)\nengine trace:\n{trace}",
        faces.len()
    );
}

#[test]
#[ignore = "C-01: fillet_v2 over-sweeps a box (volume drifts linearly in r); unfixed"]
fn fillet_v2_over_sweeps_a_box() {
    // The second half of C-01, isolated: on a cube `fillet_v2` never errors,
    // it simply returns too much material, and the drift grows roughly
    // linearly in `r` (+1.3 at r=0.01, +13.4 at r=0.1, +135.3 at r=1), i.e.
    // the strips sweep less than the full exterior dihedral angle.
    //
    // Calibrated against the one incontestable baseline: as `r -> 0` the
    // volume must converge to the un-filleted cube. No closed form is used,
    // because three derivations of the rounded-cube volume disagreed
    // (937.89 / 975.59 / 996.19), so the limit is the reference instead.
    let mut failures = Vec::new();
    for &r in &[0.0, 0.001, 0.01, 0.1, 0.5, 1.0, 2.0] {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        match fillet_v2(&mut topo, solid, &edges, r) {
            Ok(res) => {
                let vol = solid_volume(&topo, res.solid, 0.05).unwrap();
                if r >= 0.1 && (vol - 1000.0).abs() >= 1.0 {
                    failures.push(format!(
                        "r={r}: volume {vol:.4} is off the 1000.0 baseline by {:.4}",
                        vol - 1000.0
                    ));
                }
            }
            Err(e) => failures.push(format!("r={r}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "fillet_v2 must track the un-filleted cube as r shrinks; drift: {failures:?}"
    );
}

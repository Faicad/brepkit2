//! Known-failure regression test for C-01.
//!
//! Calibrates `fillet_v2` against the only incontestable baseline available
//! for a box fillet: as `r -> 0` the volume must converge to the un-filleted
//! volume of the box. No closed form is used, because three derivations of the
//! rounded-cube volume disagreed (937.89 / 975.59 / 996.19), so the limit is
//! the reference instead.
//!
//! Runs as a module of `regress_fillet_cascade/main.rs`; the shared lint
//! levels for this target live in that root file.

use brepkit_operations::blend_ops::fillet_v2;
use brepkit_operations::measure::solid_volume;
use brepkit_operations::primitives::make_box;
use brepkit_topology::Topology;
use brepkit_topology::explorer::solid_edges;

use crate::analytic_box_fillet_volume as analytic_fillet_volume;

/// Tolerance used for the volume assertions below (measurement is tessellation
/// based, so exact equality is not available).
const MEASURE_TOL: f64 = 1.0;

/// Tolerance for the box closed-form comparisons.
///
/// Wider than [`MEASURE_TOL`] because the *measurement* itself drifts at large
/// radii: at `r = 2` on the 10-cube the tessellation-based volume lands 1.96
/// under the closed form, and `fillet_rolling_ball` — a separate solver —
/// lands on 905.7461 against `fillet_v2`'s 905.7464, i.e. the two engines agree
/// to 3e-4 and the residual belongs to the ruler, not the geometry.
const CLOSED_FORM_TOL: f64 = 2.0;

#[test]
fn probe_cube() {
    // Cube reference: `fillet_v2` against the analytic volume on a 10-cube.
    //
    // This used to be a ticket asserting that the volume stayed within 1.0 of
    // the un-filleted 1000.0 — the theory being that a box fillet only shaves
    // a thin r-scaled sliver. That theory was wrong: the closed form gives
    // 993.729 at r=0.5 and 975.587 at r=1, i.e. 2.5752*(lx+ly+lz)*r^2, so the
    // old assertion would fail on a *correct* engine. It now asserts the
    // closed form, which the fixed engine reproduces across the whole sweep.
    let mut failures = Vec::new();
    // `r = 0` is excluded: a zero radius is rejected up front as invalid input,
    // and the `r -> 0` convergence itself is pinned in `fillet_box_volume.rs`.
    for &r in &[0.001, 0.01, 0.1, 0.5, 1.0, 2.0] {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        match fillet_v2(&mut topo, solid, &edges, r) {
            Ok(res) => {
                let vol =
                    brepkit_operations::measure::solid_volume(&topo, res.solid, 0.05).unwrap();
                let expect = analytic_fillet_volume(10.0, 10.0, 10.0, r);
                if (vol - expect).abs() >= CLOSED_FORM_TOL {
                    failures.push(format!(
                        "r={r}: volume {vol:.4}, analytic {expect:.4} (delta {:+.4})",
                        vol - expect
                    ));
                }
            }
            Err(e) => failures.push(format!("r={r}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "fillet_v2 must reproduce the closed form across the radius sweep; drift: {failures:?}"
    );
}

#[test]
#[ignore = "C-01: fillet_rolling_ball rejects the edge-midpoint tangency; unfixed"]
fn probe() {
    // Calibration: the measure path against the unfilled box. A correct measure
    // must return exactly 500.0000 here, otherwise every volume assertion in
    // this file would be measuring the ruler instead of the solid.
    {
        let mut topo = Topology::new();
        let s = make_box(&mut topo, 5.0, 5.0, 20.0).unwrap();
        let vol = solid_volume(&topo, s, 0.05).unwrap();
        assert!(
            (vol - 500.0).abs() < 1e-6,
            "measure baseline is broken: unfilled 5x5x20 measured {vol:.4}, expected exactly 500"
        );
    }
    for &r in &[2.49, 2.5] {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, 5.0, 5.0, 20.0).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let expect = analytic_fillet_volume(5.0, 5.0, 20.0, r);
        // ── rolling_ball (the engine that owns the setback check) ────
        // ── KNOWN FAILURE (C-01) ─────────────────────────────────────
        // `r` == half the shortest edge is *tangency*, not overlap: the two
        // fillet strips meet exactly at the edge midpoint, so
        // `setback_start + setback_end == e_len`. The correct answer for
        // 5x5x20 @ r=2.5 is a capsule of volume 359.974. rolling_ball
        // rejects the whole configuration, so the whole cascade has to fall
        // back and the caller silently gets a flat bevel (8 faces, volume
        // 468.75) instead.
        let (vol, refusal) =
            match brepkit_operations::fillet::fillet_rolling_ball(&mut topo, solid, &edges, r) {
                Ok(id) => (solid_volume(&topo, id, 0.05).unwrap(), None),
                Err(e) => (f64::NAN, Some(e.to_string())),
            };
        let analytic = analytic_fillet_volume(5.0, 5.0, 20.0, r);
        assert!(
            refusal.is_none(),
            "fillet_rolling_ball must round the tangency at r={r}: rejected with \
             \"{msg}\", analytic answer is a capsule of volume {analytic:.4}",
            msg = refusal.unwrap_or_default()
        );
        // The assertion the fix has to make green; under-swept strips and a
        // silently rejected tangency both fail here.
        assert!(
            (vol - expect).abs() < MEASURE_TOL,
            "5x5x20 filleted at r={r} must measure {expect:.4} (capsule), measured {vol:.4} \
             (delta {:+.4})",
            vol - expect
        );
    }
}

#[test]
#[ignore = "C-01: fillet_v2 must return the capsule at r=2.5; unfixed"]
fn probe_fillet_v2_slab() {
    // `fillet_v2` on the tangency configuration: it does not error, it returns
    // a solid whose volume is far too large. Kept as a separate ticket so the
    // two engines can be fixed on different commits.
    let r = 2.5;
    {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, 5.0, 5.0, 20.0).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        let expect = analytic_fillet_volume(5.0, 5.0, 20.0, r);
        let res = fillet_v2(&mut topo, solid, &edges, r).expect("fillet_v2 must succeed here");
        let vol = solid_volume(&topo, res.solid, 0.05).unwrap();
        assert!(
            (vol - expect).abs() < MEASURE_TOL,
            "fillet_v2 must return the capsule at r={r}: volume {vol:.4} is {:.4} above the \
             analytic {expect:.4}",
            vol - expect
        );
    }
}

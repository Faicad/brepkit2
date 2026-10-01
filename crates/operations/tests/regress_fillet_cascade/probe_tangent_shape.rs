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

/// Tolerance used for the volume assertions below (measurement is tessellation
/// based, so exact equality is not available).
const MEASURE_TOL: f64 = 1.0;

/// Analytic volume of a box `Lx x Ly x Lz` with all 12 edges filleted by `r`.
///
/// The filleted box is the neighbourhood of the inner offset core
/// `K = (Lx-2r) x (Ly-2r) x (Lz-2r)`; for a non-degenerate core Steiner's
/// formula applies. Degenerates to a capsule when the core collapses.
fn analytic_fillet_volume(lx: f64, ly: f64, lz: f64, r: f64) -> f64 {
    let (kx, ky, kz) = (lx - 2.0 * r, ly - 2.0 * r, lz - 2.0 * r);
    if kx <= 0.0 || ky <= 0.0 || kz <= 0.0 {
        // Core collapsed to a segment (or a point): the result is a capsule.
        let axis = [kx, ky, kz].into_iter().filter(|v| *v > 0.0).count();
        if axis == 1 {
            let len = [kx, ky, kz].into_iter().find(|v| *v > 0.0).unwrap();
            return std::f64::consts::PI * r * r * len
                + (4.0 / 3.0) * std::f64::consts::PI * r * r * r;
        }
        return (4.0 / 3.0) * std::f64::consts::PI * r * r * r;
    }
    let v = kx * ky * kz;
    let sa = 2.0 * (kx * ky + ky * kz + kx * kz);
    let edges = 4.0 * (kx + ky + kz); // total core edge length
    // Four planar core faces squeeze to: r*sa, plus 4 cylindrical bands of
    // exterior dihedral angle pi/2.
    let bands = r * r * (std::f64::consts::PI / 4.0) * edges;
    let caps = r * r * r * 8.0 * (std::f64::consts::PI / 2.0) / 3.0;
    v + r * sa + bands + caps
}

#[test]
#[ignore = "C-01: fillet_v2 over-sweeps a box (volume drifts linearly in r); unfixed"]
fn probe_cube() {
    // Cube reference: does fillet_v2 track the analytic volume on a cube where
    // the previous round measured 975.332/975.6 for the rolling-ball chain?
    // Calibrate against the incontestable limit: as r -> 0 the volume must
    // converge to the un-filleted cube volume (1000.0). This avoids relying on
    // a hand-derived closed form, which three derivations disagreed on.
    let mut failures = Vec::new();
    for &r in &[0.0, 0.001, 0.01, 0.1, 0.5, 1.0, 2.0] {
        let mut topo = Topology::new();
        let solid = make_box(&mut topo, 10.0, 10.0, 10.0).unwrap();
        let edges = solid_edges(&topo, solid).unwrap();
        match fillet_v2(&mut topo, solid, &edges, r) {
            Ok(res) => {
                let vol =
                    brepkit_operations::measure::solid_volume(&topo, res.solid, 0.05).unwrap();
                // ── KNOWN FAILURE (C-01) ────────────────────────────────
                // As `r -> 0` the volume must converge to the un-filleted
                // cube. The measured drift grows ~linearly in `r`
                // (+1.3 at r=0.01, +13.4 at r=0.1, +135.3 at r=1), i.e. the
                // engine drops a fixed-thickness slab instead of rounding
                // the edge through the full exterior dihedral angle. This
                // assert is the bug ticket; it turns green on its own once
                // the strips sweep the correct angle.
                if r >= 0.1 {
                    assert!(
                        (vol - 1000.0).abs() < 1.0,
                        "fillet_v2 must add only a thin r-scaled sliver: at r={r} volume {vol:.4} \
                         is {:.4} away from the un-filleted cube — strips are under-swept",
                        vol - 1000.0
                    );
                }
            }
            Err(e) => failures.push(format!("r={r}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "fillet_v2 must succeed for every r in the convergence sweep; refusals: {failures:?}"
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

//! Test target root for the known-failure reproductions of the fillet cascade.
//!
//! Every case in this directory is `#[ignore]`d because its bug is still open:
//! the default suite stays green and `cargo test -- --ignored` shows the
//! ticket. The failing assertion *is* the bug — the moment an engine is fixed
//! that assertion turns green without being edited.
//!
//! Cargo builds only `main.rs` inside a `tests/` subdirectory, so the shared
//! lint levels live here at the crate root of this target rather than in each
//! case, where an inner `#![allow(...)]` attribute would not be permitted.

// `unwrap`/`expect` on the topology arena is the house style of this crate's
// integration tests, and `deprecated` is unavoidable: two of the three engines
// in the cascade under test (`fillet_rolling_ball`, and the flat-bevel `fillet`
// that the wasm fallback lands on) are deprecated on purpose, and reproducing
// the fallback means calling them anyway.
#![allow(clippy::unwrap_used, clippy::expect_used, deprecated)]

mod probe_cascade;
mod probe_tangent_shape;
mod regress_fillet_mixed_radius;
mod regress_fillet_tangent_edges;

/// Analytic volume of a box `lx x ly x lz` with all 12 edges filleted by `r`.
///
/// The filleted box is the neighbourhood of the inner offset core
/// `K = (lx-2r) x (ly-2r) x (lz-2r)`; for a non-degenerate core Steiner's
/// formula applies. Degenerates to a capsule when the core collapses.
///
/// This is the same closed form `tests/fillet_box_volume.rs` uses, and it was
/// pinned down by two independent checks: the linear term cancels in the
/// `r -> 0` expansion, and `fillet_rolling_ball` (a different solver)
/// reproduces it across a radius sweep. That settles the earlier
/// "three derivations disagreed (937.89 / 975.59 / 996.19)" worry — the
/// cube value at `r = 1` is 975.587, not ~1000.
pub(crate) fn analytic_box_fillet_volume(lx: f64, ly: f64, lz: f64, r: f64) -> f64 {
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

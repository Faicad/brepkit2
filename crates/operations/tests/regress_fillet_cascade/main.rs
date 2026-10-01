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

//! Typed result structs for structured WASM returns.
//!
//! Types annotated with `Tsify` automatically generate TypeScript definitions.
//! The bindings serialize them with `serde_json` and pass the resulting string
//! to JS, so no `WasmAbi` conversion is derived.
//!
//! Note: `Tsify` copies these doc comments into the generated `.d.ts`, so the
//! explanation for `#[allow(dead_code)]` below is kept in plain `//` comments
//! to avoid changing the published TypeScript declarations.

use tsify::Tsify;

/// Typed result for `tessellateSolidGrouped`.
#[derive(serde::Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct GroupedMeshResult {
    pub positions: Vec<f64>,
    pub normals: Vec<f64>,
    pub indices: Vec<u32>,
    pub face_offsets: Vec<u32>,
}

/// Typed result for `tessellateSolidUV`.
#[derive(serde::Serialize, Tsify)]
pub struct UvMeshResult {
    pub positions: Vec<f64>,
    pub normals: Vec<f64>,
    pub indices: Vec<u32>,
    pub uvs: Vec<f64>,
}

// `boundingBox` returns a flat `Vec<f64>`, not this struct. Kept only to emit
// the TypeScript declaration; not constructed in Rust.
/// Typed result for `boundingBox`.
#[allow(dead_code)]
#[derive(serde::Serialize, Tsify)]
pub struct BoundingBoxResult {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

// `fuseWithEvolution` builds its JSON with `format!`, not this struct. Kept
// only to emit the TypeScript declaration; not constructed in Rust.
/// Typed result for boolean operations with evolution tracking.
#[allow(dead_code)]
#[derive(serde::Serialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionResult {
    pub solid: u32,
    pub generated: Vec<u32>,
    pub modified: Vec<u32>,
}

// `sketchSolve` returns a JSON string, not this struct. Kept only to emit the
// TypeScript declaration; not constructed in Rust.
/// Typed result for `sketchSolve`.
#[allow(dead_code)]
#[derive(serde::Serialize, Tsify)]
pub struct SketchSolveResult {
    pub converged: bool,
    pub points: Vec<f64>,
    pub residual: f64,
}

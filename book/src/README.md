# brepkit2

A B-Rep modeling kernel in Rust, compiled to WebAssembly.

brepkit2 is a fork of [brepkit](https://github.com/andymai/brepkit) (upstream tag
`v2.129.15`). It handles NURBS geometry, boolean operations, filleting,
tessellation, and data exchange — in memory-safe Rust with first-class WASM
support.

## Why brepkit2?

- **Pure Rust** — no C/C++ dependencies, no complex build systems
- **WASM-first** — designed for browser and Node.js environments
- **Memory-safe** — no undefined behavior, no use-after-free
- **Layered architecture** — clean separation of math, topology, operations, and I/O
- **Modern tooling** — strict linting, property-based testing, comprehensive CI

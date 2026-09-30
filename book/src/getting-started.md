# Getting Started

## Prerequisites

- [Rust](https://rustup.rs/) (stable, edition 2024)
- [wasm-bindgen CLI](https://rustwasm.github.io/wasm-bindgen/) for WASM builds
- [Node.js](https://nodejs.org/) 20+ for TypeScript bindings

## Building

```bash
# Clone the repository
git clone https://github.com/Faicad/brepkit2.git
cd brepkit2

# Build all Rust crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build WASM target
cargo build -p brepkit-wasm --target wasm32-unknown-unknown
```

## Using from JavaScript and TypeScript

The maintained JS surface is the `@faicad/brepkit2-wasm` package, built from
`crates/wasm`. It ships its own TypeScript declarations.

```bash
npm install @faicad/brepkit2-wasm
```

```typescript
import { BrepKernel } from '@faicad/brepkit2-wasm';

const kernel = new BrepKernel();
const solid = kernel.makeBox(10, 20, 30);
```

To build it from a checkout instead of installing the release:

```bash
cargo xtask wasm-build
node scripts/test-wasm-smoke.mjs
```

## Development

```bash
# Install development tooling
npm install          # Husky hooks, commitlint
cargo install cargo-deny cargo-llvm-cov  # CI tools

# Format and lint
cargo fmt --all
cargo clippy --all-targets

# Check crate boundaries
./scripts/check-boundaries.sh
```

# Digital PCR Data Essentials Standard (DDES)

Tools for reading and writing DDES datasets.

- [`ddes-rs/`](ddes-rs/): the Rust core for DDES v1.0.0 main, assays, and
  endpoint-intensity files. Includes tests, examples, and proposed numeric-field
  clarifications for the specification.
- [`typescript/`](typescript/): TypeScript/WebAssembly bindings with editable
  objects, exact bigint counts, and roundtrip and clustering examples.
- [`archive/`](archive/): the original R/Shiny converter and annotation app,
  with its demo datasets and deployment metadata preserved.

## Rust core

Run these commands from `ddes-rs/`. Rustup selects the pinned Rust 1.88.0 toolchain.

```sh
cargo test --locked --all-targets
cargo test --locked --doc
cargo run --locked --example roundtrip -- tests/fixtures target/example.ddes
```

The output archive must not already exist. See the [Rust README](ddes-rs/README.md)
for API usage, supported data, validation rules, and verification commands.
The core loads datasets into memory and retains reported scientific results.

## TypeScript bindings

Run these commands from `typescript/` with Node 24.2+, Rustup, and wasm-pack installed:

```sh
npm ci
npm test
```

The build produces JS, WASM, and documented TypeScript declarations in `pkg/`.
See the [TypeScript README](typescript/README.md) for initialization, component
and archive APIs, ownership, and examples.

## CI and releases

CI checks the Rust core on Linux, macOS, and Windows and verifies a source
package on Linux. See [CI checks](ci/README.md) for commands, coverage, required
statuses, and publishing prerequisites. Contributor guidance is in [AGENTS.md](AGENTS.md).
Publishing is disabled; CI artifacts are for testing, not public distribution.

License selection remains with DIGPCR; no new license is assigned here.

# Digital PCR Data Essentials Standard (DDES)

Tools for reading and writing DDES datasets.

- [`ddes-rs/`](ddes-rs/): the Rust core for DDES v1.0.0 main, assays, and
  endpoint-intensity files. Includes tests, examples, and proposed numeric-field
  clarifications for the specification.
- [`r/`](r/): R bindings with editable lists and data frames, component and
  archive I/O, roundtrip and clustering examples, and a Shiny converter and
  annotation app.
- [`archive/`](archive/): the reference R/Shiny converter and annotation app
  and its demo datasets.

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

## R bindings

From the repository root with R 4.2+, Rustup, Cargo, and C build tools installed:

```sh
R CMD INSTALL r
Rscript r/examples/roundtrip.R ddes-rs/tests/fixtures
```

See the [R README](r/README.md) for API usage, exact counts, validation,
installation requirements, and development checks.
The [Shiny example](r/examples/shiny/README.md) provides a browser interface
for conversion, annotation, inspection, and DDES downloads using the R bindings.

## CI and releases

CI checks the Rust core on Linux, macOS, and Windows and verifies a source
package on Linux. See [CI checks](ci/README.md) for commands, coverage, required
statuses, and publishing prerequisites. Contributor guidance is in [AGENTS.md](AGENTS.md).
Publishing is disabled; CI artifacts are for testing, not public distribution.

License selection remains with DIGPCR; no new license is assigned here.

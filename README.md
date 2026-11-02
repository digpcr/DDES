# Digital PCR Data Essentials Standard (DDES)

Tools for reading and writing DDES datasets.

- [`ddes-rs/`](ddes-rs/): the Rust core for DDES v1.0.0 main, assays, and
  endpoint-intensity files. Includes tests, examples, and proposed numeric-field
  clarifications for the specification.
- [`python/`](python/): Python bindings for all three component types and
  archives, with editable classes, type hints, and roundtrip and clustering examples.
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

## Python bindings

Run these commands from `python/` with Python 3.10+, Rustup, and uv installed:

```sh
uv sync --locked
uv run --locked python examples/roundtrip.py ../ddes-rs/tests/fixtures
```

The distribution is named `ddes-python`; import it as `ddes`. See the
[Python README](python/README.md) for API usage, the clustering example,
and development checks.

## CI and releases

CI checks the Rust core on Linux, macOS, and Windows and verifies a source
package on Linux. See [CI checks](ci/README.md) for commands, coverage, required
statuses, and publishing prerequisites. Contributor guidance is in [AGENTS.md](AGENTS.md).
Publishing is disabled; CI artifacts are for testing, not public distribution.

License selection remains with DIGPCR; no new license is assigned here.

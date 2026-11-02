# CI and package checks

Workflows run on every pull request, pushes to `main`, manual runs, and a Monday
schedule. They have no path filters and use read-only GitHub permissions.

## Rust checks

The `Rust CI` workflow runs formatting, Clippy, tests, doctests, and rustdoc on
Linux x86-64, macOS arm64, and Windows x64. Linux also builds and verifies a Rust
source package. Successful package artifacts are retained for 14 days.

The workflow lints every workflow file with Actionlint and checks package
metadata using `ci/check-versions.py`. For each binding directory present in the
checkout, the checker requires matching language/Cargo package versions, a
dependency on the local core, matching locked core versions, and the same Rust
toolchain. Absent binding directories do not require installing that language.

Require **`Rust checks`** in branch rules. It fails if workflow lint or any Rust
matrix job fails, is cancelled, or is skipped. Binding workflows have their own
required statuses; enable those alongside `Rust checks` when the binding lands.
Adding workflow files does not configure branch protection.

PRs use Rust 1.88.0. Scheduled runs use stable Rust with the same dependency
lockfiles; they do not change pins or publish anything. Schedules and Dependabot
updates start when their configuration reaches the default branch. Dependabot
proposes weekly Actions and Cargo updates, with no auto-merge.

## Local validation

From the repository root, with Python 3.11+ and Actionlint installed:

```sh
python3 ci/check-versions.py
actionlint
```

From `ddes-rs/`:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps
cargo package --locked
```

Packaging requires a clean Git tree. The shared fixtures in
`ddes-rs/tests/contract/` exercise exact counts above 2^53 and at the u64 maximum,
missing and zero values, negative measurements, Unicode and leading-zero IDs,
and whole-list/individual missing probe concentrations. Expectations are
specified independently of the writers.

## Publishing prerequisites

Publishing is disabled. CI artifacts are test packages, not a complete release
matrix. No workflow uses registry credentials or creates releases.

Before enabling a separate, approved release workflow, DIGPCR must select the
license, package ownership/names, and coordinated package versions. The DDES
format version is independent of package versions. Define supported binary
targets and test source packages outside the checkout; binding source builds
must not assume an adjacent core directory. Select the R distribution
destination and satisfy its checks before releasing an R package.

Use a protected publishing environment and trusted publishing where supported.
Never expose publishing credentials to PR builds or use `pull_request_target`
to execute PR code. Removing upload prohibitions is part of an approved release,
not CI maintenance.

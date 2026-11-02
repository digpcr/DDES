# Maintaining DDES

`ddes-rs/` owns the format and scientific validation. Binding directories
(`python/`, `typescript/`, and `r/`, when present) expose that core through
idiomatic language APIs. Keep `archive/` as reference material; do not change it
when maintaining the bindings.

When changing public records, metadata, validation, archive behavior, or errors:

- Inspect every binding's conversions and public API. A passing Rust build alone
  does not establish compatibility.
- Update Python type hints/docstrings, TypeScript declarations, and R help where
  the public contract changes. Check examples and READMEs too.
- Preserve exact counts, the distinction between missing and zero, finite-number
  rules, identifiers, and whole-list/individual missing probe concentrations.
- Extend shared contract fixtures and independently specified expectations when
  the scientific contract changes. Do not derive expected values from the writer
  being tested. Language-specific ownership rules stay in each binding's tests.
- Run the core and affected binding checks before declaring the change complete.
  Commands are in `ci/README.md`, the workflow files, and each binding's README.
  Report checks you could not run, including bindings absent from the checkout.

Document current capabilities, values, errors, and usage—not implementation
history. Retain limitations that affect users. Keep package versions separate
from the DDES format version. Coordinate package version changes across language
manifests; do not enable publishing or select a license without approval.

Each binding owns its CI workflow; run all available workflows on every PR.
Keep their permissions read-only, do not execute PR code through
`pull_request_target`, and do not add publishing credentials to build/test jobs.
Build and test distributable artifacts before publishing them.

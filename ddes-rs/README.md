# ddes-rs

An in-memory Rust core for **DDES v1.0.0**. Read and write main,
assays, and endpoint-intensity TSVs, either individually, in a flat directory,
or in a ZIP/`.ddes` archive.

## Try it

Requires Rust **1.88 or newer**; `rust-toolchain.toml` pins **1.88.0** for local
development. The Cargo package is `ddes-rs`; the Rust import name is `ddes`.
Run from this directory:

```sh
cargo test --locked
cargo run --locked --example roundtrip -- tests/fixtures target/example.ddes
```

The example prints dataset counts and a result, writes a new archive, reloads
it, and checks semantic equality. Output paths must not already exist; choose
a new path when rerunning it, or remove your previous output first.

Read, edit, and round-trip a dataset in memory (this example uses the bundled
fixtures and writes no files):

```rust
use std::io::Cursor;

use ddes::Dataset;

fn main() -> ddes::Result<()> {
    let mut data = Dataset::read_path("tests/fixtures")?;
    data.main.records[0].sample_id = "renamed sample".into();
    data.main.records[0].counts_positive = Some(4236);
    data.main.records[0].threshold_if_threshold = None; // Written as NA, not zero.
    let bytes = data.write_zip(Cursor::new(Vec::new()))?.into_inner();
    let reloaded = Dataset::read_zip(Cursor::new(bytes))?;
    assert_eq!(reloaded, data);
    Ok(())
}
```

- `Dataset::read_path` reads directories or archives.
- `Dataset::{read_zip, write_zip}` operate on seekable streams, including `Cursor`.
- `Dataset::write_path` creates an archive without overwriting an existing file.
- Filesystem locations accept native `Path`/`PathBuf` values, including non-UTF-8
  paths. TSV contents and portable component filenames remain UTF-8.
- `MainFile`, `AssaysFile`, and `IntensityFile` expose `from_tsv` and `to_tsv`
  for individual TSV byte buffers. Their path argument is only a diagnostic
  label; these methods do not open or create files. Each file has `metadata`
  and `records` fields, with named attributes such as
  `data.main.metadata.experiment_id`.

## Stream I/O is not row-by-row streaming

`read_zip` takes `Read + Seek`, such as `File`, `BufReader<File>`, or
`Cursor<&[u8]>`. `write_zip` takes `Write + Seek`, such as `File` or
`Cursor<Vec<u8>>`, and returns that writer after finishing the ZIP. These methods
let callers manage storage without passing a filesystem path to the library.
They operate on ZIP archives only, not directories or individual TSVs.

```sh
# Use the archive produced by the roundtrip example above:
cargo run --locked --example stream_io -- target/example.ddes
```

`examples/stream_io.rs` reads a buffered file, writes ZIP bytes to a cursor, then
reads those bytes back and checks equality. It does not create an output file.

This is still an in-memory API: reading buffers each decompressed component and
retains all parsed records; writing builds all serialized TSV buffers before
writing the ZIP. A cursor also retains the ZIP bytes.
Pipes, stdin/stdout, and ordinary HTTP bodies do not
implement `Seek` and cannot be passed directly to these methods.

All records remain in memory; these APIs do not support incremental processing.

## Compatibility baseline and normalization

The baseline is the supplied April 2026 DDES paper and
[DIGPCR/DDES revision 8776b8a](https://github.com/digpcr/DDES/commit/8776b8a71278bd1e17b3599f2ae4bfe29c8f065a),
particularly `global.R`, `server.R`, and `demo_data/QIAcuity`. This is not a
claim of complete specification conformance.

The reference app and demo data are preserved under `../archive/` in this repository.

- Preserve metadata values, including the reference data's `1.0` version spelling.
  `1.0.0` is also accepted and written unchanged. Date/version grammar is not
  tightened, and version strings alone do not establish schema compatibility.
- Normalize LF/CRLF input to LF output, canonical header/column order, and
  double-quoted TSV cells. Quoting protects apostrophes, `#`, tabs, and newlines
  inside values. ZIP bytes, original filenames, and compression need not match.
- Accept `#head` and `#header`. Write the reference layout: main/assays use
  `#header`; intensity uses `#head`; assays retain `#assay_information`.
- Identifiers stay strings, including Unicode and leading zeros. No delimiter,
  datetime, probe-list cardinality, or cross-file-reference restrictions are added.
  Assay probe lists remain strings, with concentration values validated as below.
- Numeric handling deliberately narrows the former permissive R-like acceptance;
  see the numeric contract below. Columns and their meanings are unchanged.
- Unknown metadata, table columns, and components are rejected, not silently
  discarded. Required known columns may appear in any order.
  Duplicate columns/metadata/components are errors.
- A dataset needs one main and one assays component; intensity files may be absent.
  `MainFile` supports using a main file alone. When present, there
  may be only one intensity component per well. Loaded intensity components sort
  by well ID; row order within a component is preserved.

### Numeric fields and missing values

The agreed narrowing and its paper references are recorded in
the repository's `notes/paper-numeric-fields.md` for contribution
to the paper/specification. These are implementation decisions, not a claim that
the published paper already mandates every restriction.

- Positive/negative counts: `Option<u64>`, exact unsigned decimal integers.
- Reaction concentration: `Option<f64>`, finite and nonnegative when present.
- Threshold: `Option<f64>`, finite when present; negative values are allowed.
- Endpoint intensity: required finite `f64`; negative processed values are allowed.
- Probe concentrations: the existing semicolon-separated `String`, validated as
  finite nonnegative numbers or `NA` entries. Both `NA` and `125;NA;125` are valid;
  empty entries are not. Entry positions and list spelling are retained.

All TSV columns remain required; optional values do not make columns optional.
`NA` is the only missing marker and maps to `None` for optional scalar fields.
Zero is never missing. Empty cells, `-`, arbitrary text, nonfinite floats, and
out-of-range counts are rejected on input; writers also reject invalid values
constructed by callers. Invalid intensity rows are rejected, not silently dropped.
Surrounding numeric-token whitespace is accepted; identifier whitespace is
preserved. Scalars normalize on output with normal binary64 precision for floats
(e.g. `125` may become `125.0`); counts retain the full `u64` range.

### Deliberate limits and areas to revisit

- Endpoint columns only. Real-time/melting fluorescence columns and arbitrary
  table extensions return an unsupported-input error. No extension data is dropped.
- UTF-8 and CSV-style double quoting are supported. R-specific single-quote
  parsing and inline `#` comment semantics are not emulated; quote such values
  with double quotes. Whole-line table comments and blank lines are skipped.
  The library does not reproduce R's identifier coercion or column-name rewriting.
- Structure must be representable: required metadata/columns and `#data` must
  exist, `DDES_type` must match the component (including `assay`/`assays`), and
  metadata must fit one key/value line. No full schema-validation framework.
- Archive inputs must be flat, regular TSV files with portable names. Components
  are identified by `#DDES_type`, not R's broad filename regex, so identifiers
  containing words like `main` do not confuse this reader. The R reader may still
  misclassify such filenames; we do not restrict or rewrite identifiers to fix that.
  Reject paths, symlinks, reserved Windows names, duplicate names (including
  ASCII case collisions), and inconsistent ZIP headers. Members are never extracted.
  ZIP stored/deflated compression is supported; encrypted and other compression
  variants are outside this MVP. Output names use experiment/run/datetime/well
  metadata; unportable names fail rather than silently sanitizing identifiers.
- Reads load all data into memory without size or file-count caps. Account for
  uncompressed input size, especially with untrusted archives. Parsed records
  and serialization buffers also consume memory.
- Writes validate/serialize before creating the destination, but an I/O failure
  may leave a partial output. Writes are not atomic. Streaming and performance
  tuning are deferred.
- Candidate future validation: cross-file IDs, probe/channel relationships,
  date/version syntax, and additional header rules. These remain unenforced.

## Fixtures and verification

`tests/fixtures` is derived from DIGPCR's QIAcuity data at the pinned revision:

- Main: original header/columns plus the three A1 rows.
- Assays: the complete original file (one triplex assay).
- Intensity A1: original header/columns plus the first three rows for each channel.
- Filenames shorten the run ID to `run`; metadata and retained data values are
  unchanged. The subset is for parsing tests, not scientific count validation.

Tests also generate main, assays, and intensity TSVs in temporary directories,
independently of these fixtures and the Rust writer. They check parsed values
and directory-to-archive round trips, using non-UTF-8 filesystem paths on Linux
and Unicode paths elsewhere.

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps

# Optional independent check using base R, after running the roundtrip example:
Rscript tests/r_interop.R tests/fixtures target/example.ddes

# Also run the preserved R reader (requires R's stringr):
Rscript tests/r_interop.R tests/fixtures target/example.ddes ../archive
```

The README is also the crate's landing-page documentation. `cargo test --doc`
executes its Rust example; `--all-targets` alone does not run doctests. The CLI
examples are compiled by the tests but must be run with the commands above to
exercise their round-trip checks. Generated API documentation is available at
`target/doc/ddes/index.html`.

The R script checks every table and fixed metadata header against the source.
With `../archive` it executes the preserved `retrieve_demo_dataset_ddes` reader,
stubbing only Shiny progress indicators. Run it only on trusted, locally generated
archives and the preserved reference code.

The routine checks use the small fixtures and generated temporary data, with
local verification on **macOS ARM64**. Earlier full-upstream-data validation
covered 72 main records, one assay, 24 wells, and 1,829,297 intensity records:
Rust output matched the source through both R `read.table` and the upstream
reader. That larger check predates the numeric narrowing and is not part of `cargo test`.
Windows, Linux (GNU/musl), and other architectures are design targets, not yet
runtime-tested platforms. No cross-platform CI has been added.

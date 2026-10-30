# ddes

Read, edit, and write **DDES v1.0.0** datasets in R. Work with main, assays,
and endpoint-intensity TSVs or ZIP/`.ddes` archives. Reported scientific results
are preserved without recalculation; text and ZIP formatting may normalize.

## Install from source

Requires R 4.2+, Rust/rustup, Cargo, and the platform's C build tools. The
toolchain file selects Rust 1.88.0. The source layout requires `r/` beside
`ddes-rs/`.

From the repository root:

```sh
R CMD INSTALL r
```

On macOS install the Xcode command-line tools; on Linux install R development
headers and a C compiler. On Windows use Rtools matching your R version and
the `x86_64-pc-windows-gnu` Rust target (`rustup target add
x86_64-pc-windows-gnu`). The Windows build targets x64 R.
Builds are verified on macOS arm64; Windows and Linux are unverified.

Reinstall with `R CMD INSTALL r` after changing package code or documentation.

## Read, edit, and write

From the repository root after installation:

```r
library(ddes)

dataset <- read_archive("ddes-rs/tests/fixtures") # Or a ZIP/.ddes path.
dataset$main$metadata$experiment_id <- "updated"
if (nrow(dataset$main$records)) {
  dataset$main$records$counts_positive[1] <- "42"
  dataset$main$records$threshold_if_threshold[1] <- NA_real_
}
if (length(dataset$intensities) && nrow(dataset$intensities[[1]]$records)) {
  dataset$intensities[[1]]$records$fi_endpoint[1] <- -0.5
}

restored <- archive_from_raw(archive_to_raw(dataset))
stopifnot(identical(restored, dataset))
write_archive(dataset, "updated.ddes") # Destination must not already exist.
```

A dataset is a list of `main`, `assays`, and `intensities`. Each component
contains `metadata`, a named list of scalar character values, and `records`,
a data frame. `intensities` is a list of components; `list()` means none.
Construct new components with ordinary R lists and data frames, or edit those
returned by readers. There are no handles to close; values use normal R
copy-on-modify behavior. Writers leave the input unchanged.

| Operation | Input → output |
| --- | --- |
| `read_main`, `read_assays`, `read_intensity` | Complete TSV raw vector → component list |
| `write_main`, `write_assays`, `write_intensity` | Component list → normalized TSV raw vector |
| `read_archive` | Directory or ZIP path → dataset list |
| `write_archive` | Dataset and new output path → invisible `NULL` |
| `archive_from_raw` | ZIP raw vector → dataset list |
| `archive_to_raw` | Dataset list → ZIP raw vector |

To read a standalone component, supply its bytes, not its filename:

```r
path <- "intensity.tsv"
file <- read_intensity(readBin(path, "raw", n = file.info(path)$size))
file$records$fi_endpoint <- file$records$fi_endpoint + 0.25
bytes <- write_intensity(file)
```

See `?read_main`, `?read_archive`, and `help("ddes-data")` for signatures,
all metadata/record fields, units, missing values, and errors.

## Values and validation

Fields use snake case, such as `experiment_id` and `fi_endpoint`. All fields
and columns are required; extra and duplicate fields are rejected. Identifiers
are character values so leading zeros and Unicode survive. Factors are not
accepted. Row names are not stored in DDES. To keep an empty typed table, use
`file$records <- file$records[0, , drop = FALSE]`.

| Fields | R values |
| --- | --- |
| `counts_positive`, `counts_negative` | Decimal character values from `"0"` to `"18446744073709551615"`, or `NA_character_` |
| `concentration_reaction_cp_ul` | Finite nonnegative numeric, or numeric NA |
| `threshold_if_threshold` | Finite numeric, or numeric NA; negatives allowed |
| `fi_endpoint` | Required finite numeric; zero and negatives allowed |
| `probe_concentrations_nm` | Semicolon-separated string such as `"125;NA;0"`, or `"NA"` for a missing whole list |

Counts always use decimal character values, even when small. Numeric count
columns are rejected. Do not convert large counts to doubles, which lose integer
precision above 2^53. Missing counts are not zero. Measurements accept unclassed
double or integer vectors; reads return doubles. Use `NA_real_` or `NA_integer_`,
not a logical column of bare `NA`. `NaN` and infinities are invalid, not missing.

Probe concentrations retain their spelling; numeric entries must be finite and
nonnegative. Probe/list lengths are not cross-checked. Use the literal `"NA"`
inside these strings, not `NA_character_`. Both `assay` and `assays` metadata
discriminators are accepted. Readers and writers validate scientific values;
temporary invalid edits can exist in an R object until it is written.

## Archives and errors

Errors are ordinary R errors with diagnostic messages. Writers validate before
creating a path destination; an I/O failure can still leave a partial file.
Unsafe and duplicate output filenames are rejected. Shared filename prefixes
come from main metadata `experiment_id`, `run_id`, and `datetime`; intensity
filenames add their `well_id`. Reads sort intensities by well ID. Writes preserve
supplied order. Cross-file scientific consistency is not checked.

Operations are synchronous and load complete datasets into memory. Reads have
no size or file-count limits. Allow space for uncompressed input, R tables,
conversion buffers, and output, especially when working with untrusted archives.

## Examples

From the repository root after installation:

```sh
Rscript r/examples/roundtrip.R ddes-rs/tests/fixtures updated.ddes
Rscript r/examples/clustering.R updated.ddes 2
```

The roundtrip example edits each component type and checks the resulting TSVs
and archive. The output path is optional and must not exist. Clustering uses
`stats::kmeans` on each well's multichannel measurements, aligns channels by
exact partition IDs, and writes CSV assignments
(`well_id,assay_id,partition_id,cluster`) to stdout.
It uses a fixed seed, ten starts, and no scaling; larger-scale channels dominate
distances. Missing/duplicate channel measurements, insufficient distinct
vectors, and convergence failures cause errors. Labels are arbitrary and local
to each well, not positive/negative calls. Input datasets and reported results
are unchanged. Choose cluster count and preprocessing for your experiment.

The [Shiny converter and annotation app](examples/shiny/README.md) supports
DDES uploads, a QIAcuity demo, vendor CSV conversion, run/probe/assay/sample
editing, and validated downloads. Install `shiny` and `rhandsontable`, then
run from the repository root:

```r
shiny::runApp("r/examples/shiny", host = "127.0.0.1", launch.browser = TRUE)
```

Vendor conversion is unverified with real exports. Renames preserve results,
while changing assay assignments or probe definitions clears affected reported
results. See the example README for input columns, save behavior, and memory
limits.

## Development checks

Tests require `testthat` 3+; app tests also require `shiny` and `rhandsontable`.
From the repository root:

```sh
Rscript -e 'testthat::test_dir("r/tests/testthat")'
Rscript r/examples/shiny/tests.R
```

The tests cover component and archive round trips, exact counts, missing values,
validation errors, annotation edits, session isolation, and synthetic vendor
inputs.

Check Rust code from `r/`:

```sh
cargo fmt --manifest-path src/rust/Cargo.toml --check
cargo clippy --locked --manifest-path src/rust/Cargo.toml --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --manifest-path src/rust/Cargo.toml --no-deps
```

License selection remains with DIGPCR; no new license is assigned here.

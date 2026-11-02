# ddes-python

An in-memory Python library for **DDES v1.0.0**. Read and write main, assays,
and endpoint-intensity TSVs, edit their metadata and records, and work with
flat TSV directories or ZIP/`.ddes` archives. Reported results are preserved
without recalculation; text formatting and ZIP bytes may normalize on output.

The distribution is named `ddes-python`; import it as `ddes`.

## Install from source

Requires Python **3.10+**, Rust **1.88+** with rustup, and uv. The source layout
requires the Rust crate at `../ddes-rs`. From this directory:

```sh
uv sync --locked
uv run --locked python examples/roundtrip.py ../ddes-rs/tests/fixtures
```

The example edits all three component types and checks TSV and archive round
trips. Supply an optional second path to save a new archive; the destination
must not already exist.

After changing Rust code, rebuild the extension:

```sh
uv run --locked maturin develop --locked
```

## Cluster partitions

`examples/clustering.py` runs scikit-learn k-means independently for each well.
It aligns channel measurements by partition ID, uses every channel without
scaling, and prints CSV assignments (`well_id,assay_id,partition_id,cluster`).
The `examples` dependency group installs scikit-learn; the library itself does
not require it.

```sh
uv run --locked --group examples python examples/clustering.py ../ddes-rs/tests/fixtures --clusters 2
```

Replace the fixture path with a TSV directory or `.ddes` archive. Redirect stdout
to save the CSV. Each partition must have exactly one measurement per channel;
missing/duplicate measurements and too few distinct vectors cause an error.
A failed run may leave partial CSV output; only use output from a successful run.

Cluster labels are arbitrary and local to each well, not positive/negative calls
or target assignments. Larger-scale channels dominate distances because values
are not scaled. The example uses a fixed random seed and leaves the archive,
counts, concentrations, and thresholds unchanged. Choose cluster count and
preprocessing appropriate to the experiment before interpreting results.

## Read, edit, and write

```python
from pathlib import Path
from ddes import DdesArchive, read_assays, write_assays

archive = DdesArchive.read_path(Path("../ddes-rs/tests/fixtures"))
assays = archive.assays()
assays.records[0].probe_concentrations_nm = "125;NA;0"
assert read_assays(write_assays(assays)) == assays

# Getters return detached snapshots; explicitly put edits back.
archive.set_assays(assays)
restored = DdesArchive.from_bytes(archive.to_bytes())
assert restored.assays() == assays

# Construct an independent archive from Python component objects.
created = DdesArchive(archive.main(), assays, archive.intensities())
assert created.assays() == assays
# created.write_path(Path("updated.ddes"))  # Must be a new path.
```

- `read_main(bytes)` / `write_main(file)`,
  `read_assays(bytes)` / `write_assays(file)`, and
  `read_intensity(bytes)` / `write_intensity(file)` handle individual TSVs.
  Use `Path.read_bytes()` / `Path.write_bytes()` for individual TSV files on disk.
- `MainFile`, `AssaysFile`, and `IntensityFile` contain metadata objects and
  lists of records. Attributes use snake_case names
  such as `file.metadata.experiment_id` and `file.records[0].fi_endpoint`.
- Classes can be constructed directly, for example
  `IntensityRecord("001", "BLUE", -12.75)`. Metadata and main-record
  constructors require keyword arguments, including arguments that accept `None`.
  Files take `(metadata, records)` and compare by their contents.
- Nested attribute edits persist. `records` is a mutable Python list:
  append, delete, replace elements, or assign a new list. Writers reject list
  elements of the wrong class. Constructors copy the list but retain its child
  objects, so explicitly reusing metadata/record instances shares those objects.

Use `help(ddes.DdesArchive)`, `help(ddes.AssayRecord)`, or
`help(ddes.read_assays)` after `import ddes` for API details and examples.
Type hints are included for editors and type checkers.

### Values and validation

- Counts use exact Python `int` values from 0 through `2**64 - 1`, or `None`.
  Measurements use Python `float`; optional main measurements allow `None`.
  Endpoint intensity is required and finite. Negative threshold/intensity values
  are accepted; negative concentrations are not. Zero is never treated as missing.
- `AssayRecord(assay_id, probe_combination, probe_concentrations_nm)` retains the
  probe combination and concentrations as semicolon-separated strings.
  Concentrations accept `"NA"` for the whole list or entries such as
  `"125;NA;0"`; all numeric entries must be finite and nonnegative. Probe/list
  lengths are not cross-checked. Both `"assay"` and `"assays"` metadata types are accepted.
- Attributes cannot be deleted and arbitrary attributes cannot be added. Use
  `None`, not the TSV string `"NA"`, for optional numeric attributes. Constructors
  and setters check attribute types. Finiteness, nonnegative concentration, and
  file structure are checked when writing, so edits can temporarily contain
  invalid values.

## Archives

| Operation | Behavior |
| --- | --- |
| `DdesArchive(main, assays, intensities)` | Copies Python components into a new archive; use `[]` for no intensities. |
| `DdesArchive.read_path(path)` | Reads a flat TSV directory or ZIP/.ddes archive; accepts `str` or `PathLike`. |
| `DdesArchive.from_bytes(data)` | Reads ZIP/.ddes bytes. |
| `archive.main()`, `archive.assays()` | Returns a detached component snapshot. |
| `archive.intensity(index)`, `archive.intensities()` | Returns one snapshot or a list of all snapshots. |
| `archive.intensity_count` | Number of intensity components. |
| `archive.set_main(file)`, `archive.set_assays(file)` | Replaces a component with a copy of the supplied file. |
| `archive.set_intensities(files)` | Replaces the entire intensity collection; use `[]` to remove all. |
| `archive.to_bytes()` | Returns normalized ZIP bytes. |
| `archive.write_path(path)` | Writes a new ZIP/.ddes file, refusing to overwrite an existing path. |

To add or remove an intensity, edit the list returned by `intensities()` and pass
it to `set_intensities()`. Constructors and setters copy input data; later changes
to those objects do not change the archive. A type-conversion failure leaves the
archive unchanged. Values and filenames are validated when writing, not when
copying components into an archive.

Intensity order is preserved by constructors/setters and sorted by well ID on
read. Writes reject duplicate intensity filenames and unsafe
names. Cross-file scientific consistency is not checked. `write_path()`
validates before creating a destination; an I/O failure may leave a partial file.

## Errors and numeric conversions

Invalid TSV or archive data raises `ValueError`; filesystem failures raise
`OSError` subclasses such as `FileNotFoundError`, `PermissionError`, and
`FileExistsError`, retaining the OS error's `errno` and `filename`.
Wrong constructor/attribute/function types
raise `TypeError`; negative/out-of-range counts raise `OverflowError`.
Unknown/deleted attributes raise `AttributeError`. Invalid scientific values fail
with `ValueError` on write. Negative/out-of-range representable intensity indices raise
`IndexError`; indices are not Python-style negative offsets.

Float attributes accept Python integers, and `bool` follows Python's integer
semantics (`True` becomes 1). Counts reject fractional floats,
negative/out-of-range integers, and strings.

## Memory and I/O

Archives require no manual cleanup. Snapshots and returned bytes remain usable
after the archive is deleted. Snapshot mutation never edits an archive without
an explicit setter. Each snapshot contains a copy of the component's metadata
and records.

Operations are synchronous and load complete files into memory. There is no
row-by-row streaming or asynchronous API.

Reads have no size or file-count limits. Account for uncompressed input size,
especially with untrusted archives. Component copies and serialization buffers
also consume memory.

## Development checks

```sh
uv run --locked python -m unittest discover -s tests -v
uv run --locked mypy
uv run --locked python -m mypy.stubtest ddes
uv run --locked --group examples python -m unittest discover -s tests -v
uv run --locked --group examples mypy examples/clustering.py
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
```

The tests exercise the installed extension, run its docstring examples, and
check archive round trips with the Rust library. Temporary output files are
removed after each test.

## Continuous integration

[Python CI](../.github/workflows/python.yml) builds and installs CPython 3.12
wheels on Linux x86-64, macOS arm64, and Windows x64. Tests and stubtest run
against the installed wheel, alongside mypy, Rust formatting, Clippy, and
rustdoc checks. Linux also runs the clustering tests and example type check.
Require `Python checks` alongside `Rust checks` in branch rules.

Wheels are retained for 14 days but are not published. The Linux wheel uses a
native platform tag, not a portable manylinux/musllinux build; this is not the
public release matrix. Dependabot proposes weekly Cargo and uv updates.
See [CI checks](../ci/README.md) for shared checks and publishing prerequisites.

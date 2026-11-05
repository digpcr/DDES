# ddes-typescript

Read, edit, and write **DDES v1.0.0** datasets in TypeScript. Work with main,
assays, and endpoint-intensity TSVs or ZIP/`.ddes` archives, and edit their
metadata and records. Reported scientific results are preserved without
recalculation; text formatting and ZIP bytes may normalize on output.

## Build from source

Requires Rust/rustup, wasm-pack, and Node **24.2+** with npm. The toolchain file pins
Rust 1.88.0 and the WASM target. The Rust core must be at `../ddes-rs`.
From this directory:

```sh
npm ci
npm run build
```

The build produces `pkg/ddes_typescript.js`, its WASM binary, and TypeScript
declarations with API documentation. Keep the JS and WASM files together.
TypeScript consumers need `lib: ["esnext", "dom"]` (or equivalent web and
disposable-symbol types). Rebuild after changing Rust code; generated files
are not edited by hand.

## Read, edit, and write

Initialize WASM once before calling the API. For a browser consumer, serve the
generated files over HTTP and use the default initializer:

```ts
import init, {
  DdesArchive, readMain, writeMain,
} from './pkg/ddes_typescript.js';

await init();

export function editMain(bytes: Uint8Array): Uint8Array {
  const file = readMain(bytes);
  if (file.records.length) {
    file.records[0].countsPositive = 42n;
    file.records[0].thresholdIfThreshold = null;
  }
  return writeMain(file);
}

export function editArchive(bytes: Uint8Array): Uint8Array {
  const archive = DdesArchive.fromBytes(bytes);
  try {
    const assays = archive.assays();
    if (assays.records.length) assays.records[0].probeConcentrationsNm = '125;NA;0';
    archive.setAssays(assays);
    return archive.toBytes();
  } finally {
    archive.free();
  }
}
```

Browser `File.arrayBuffer()` and `Response.arrayBuffer()` return `ArrayBuffer`.
Wrap that buffer in `Uint8Array` before passing it to a reader. Using
`editArchive` above:

```ts
export async function editFile(file: File): Promise<Uint8Array> {
  return editArchive(new Uint8Array(await file.arrayBuffer()));
}

export async function editUrl(url: string): Promise<Uint8Array> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`Failed to fetch archive: HTTP ${response.status}`);
  return editArchive(new Uint8Array(await response.arrayBuffer()));
}
```

Node can initialize the same artifact from bytes instead of fetching a URL:

```ts
import { readFileSync } from 'node:fs';
import { initSync } from './pkg/ddes_typescript.js';

initSync({ module: readFileSync('./pkg/ddes_typescript_bg.wasm') });
```

- `readMain(bytes)` / `writeMain(file)`, `readAssays(bytes)` / `writeAssays(file)`,
  and `readIntensity(bytes)` / `writeIntensity(file)` handle complete TSVs,
  including metadata and column headers. Inputs and outputs are `Uint8Array`.
- `MainFile`, `AssaysFile`, and `IntensityFile` are TypeScript interfaces for
  editable plain objects, not classes to instantiate with `new`. Each has
  `metadata` and `records`; fields use camelCase, such as `experimentId` and
  `fiEndpoint`. Construct objects directly, push/remove rows, or replace arrays.
- Writers validate data without modifying the input. Extra enumerable string
  fields are rejected rather than silently discarded.
- Filesystem and network I/O belong to the caller: use browser `File.arrayBuffer()`,
  `fetch`, or Node `fs` to obtain bytes. There are no WASM path/directory methods.
  Node filesystem failures retain Node's normal error codes and path information.

## Values and validation

| Fields | JavaScript values |
| --- | --- |
| `countsPositive`, `countsNegative` | `bigint` from `0n` through `18446744073709551615n`, or `null` |
| `concentrationReactionCpUl` | Finite nonnegative `number`, or `null` |
| `thresholdIfThreshold` | Finite `number`, or `null`; negative values are valid |
| `fiEndpoint` | Required finite `number`; zero and negative values are valid |
| `probeConcentrationsNm` | Semicolon-separated string, e.g. `'125;NA;0'`, or `'NA'` for the whole list |

All properties remain required. `null` represents a missing main numeric value;
omitted properties, `undefined`, numeric strings, and booleans are not substitutes.
Counts always use bigint, even when small. Do not convert large counts to
`number`, which can lose precision. `JSON.stringify` does not accept bigint;
use TSV/ZIP output for interchange, or `structuredClone` for an in-memory copy.

Probe concentrations retain their spelling and accept whole-list or individual
`NA` entries. Numeric entries must be finite and nonnegative. Probe/list lengths
are not cross-checked. Both `assay` and `assays` metadata discriminators are accepted.
Identifiers remain strings, preserving leading zeros and Unicode.

Plain objects can temporarily contain invalid edits. Writers check scientific
constraints and file structure. Archive construction/setters check object shapes,
field types, and count range, but scientific values and filenames are validated
on write. NaN and infinities are never converted to missing values.

## Archives and ownership

| Operation | Behavior |
| --- | --- |
| `new DdesArchive(main, assays, intensities)` | Copies component objects; `[]` means no intensities. |
| `DdesArchive.fromBytes(bytes)` | Reads a ZIP/.ddes archive and sorts intensities by well ID. |
| `archive.main()`, `archive.assays()` | Returns a detached component snapshot. |
| `archive.intensity(index)`, `archive.intensities()` | Returns one snapshot or all snapshots in archive order. |
| `archive.intensityCount` | Read-only number of intensity files. |
| `archive.setMain(file)`, `archive.setAssays(file)` | Replaces a component with a copy. |
| `archive.setIntensities(files)` | Replaces the entire collection in supplied order; `[]` removes all. |
| `archive.toBytes()` | Returns normalized ZIP bytes without changing the archive. |
| `archive.free()` | Releases the WASM handle; never use it afterward. |

Getters return copies. Editing a snapshot does not change an archive until you
call a setter. Constructors/setters also copy inputs; later edits to those inputs
do not affect the archive. A conversion failure leaves the archive unchanged,
including when a later item in `setIntensities()` is invalid.

Use `free()` in `finally`, or TypeScript's `using` with disposable support.
Snapshots and returned byte arrays remain usable after freeing the archive.
Do not send archive handles through `postMessage`; send snapshots or ZIP bytes.

Invalid data and indices throw JavaScript `Error` objects, with file/line/field
details in messages where available. Indices must be finite nonnegative integers
below `intensityCount`; negative offsets and coercion from strings are not supported.
Writes reject unsafe or duplicate output filenames. Cross-file scientific
consistency is not checked and reported results are never recalculated.

Generated archive filenames use the main file's `experimentId`, `runId`, and
`datetime` as their shared prefix. Intensity filenames also include that
component's `wellId`. Experiment/run IDs in assays and intensity metadata are
preserved in their TSV headers but do not determine archive filenames.

Operations are synchronous and load complete data into memory. Reads have no
size or file-count limits. Account for uncompressed input size, snapshots,
conversion buffers, and output buffers, especially for untrusted archives.

## Examples

After building, run either example against a ZIP/`.ddes` file:

```sh
node examples/roundtrip.ts input.ddes updated.ddes
node examples/clustering.ts input.ddes 2
```

`roundtrip.ts` edits all three component types and verifies their TSV and archive
round trips. The output path is optional and must not already exist.

`clustering.ts` runs per-well multichannel k-means using `ml-kmeans`, a development
dependency used only by the example. It aligns measurements by partition ID,
uses all channels without scaling, and prints CSV
(`well_id,assay_id,partition_id,cluster`). Redirect stdout to save assignments.
Each partition must have one measurement per channel. Duplicate/missing values,
too few distinct vectors, or failure to converge cause an error. Discard partial
CSV output if the process exits unsuccessfully.

Labels are arbitrary and local to each well, not positive/negative calls or
target assignments. The random seed is fixed; larger-scale channels dominate
distances. Counts, concentrations, thresholds, and archive data are unchanged.
Choose cluster count and preprocessing appropriate to your experiment.

## Development checks

```sh
npm test
cargo fmt --check
cargo clippy --locked --target wasm32-unknown-unknown --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --target wasm32-unknown-unknown --no-deps
```

`npm test` builds the web artifact, checks generated declarations and examples,
and runs runtime tests in Node, including native Rust ↔ WASM archive round trips.
Temporary test output is removed afterward.

## Continuous integration

[TypeScript CI](../.github/workflows/typescript.yml) builds release WASM, runs
type/runtime tests, and checks Rust formatting, Clippy, and rustdoc on Linux
with Node 24.11.0. It installs the packed npm tarball in a temporary directory
and tests both runtime imports and a TypeScript consumer. Require
`TypeScript checks` alongside `Rust checks` in branch rules.

The npm tarball is retained for 14 days and is not published. Dependabot proposes
weekly Cargo and npm updates. See [CI checks](../ci/README.md) for shared checks
and publishing prerequisites.

License selection remains with DIGPCR; no new license is assigned here.

# Numeric fields and missing values: proposed paper clarification

Decision recorded 2026-10-27. Approved for this implementation; contribute these
rules to the DDES paper/specification for author review. They narrow the current
reader's acceptance and are not claimed to be existing published requirements.
No columns are added, removed, renamed, or assigned a different meaning.

## Agreed table

All columns remain required. Missingness refers to cell values, not omitted columns.

| File | Field | Allowed Missing | Value if Missing | Proposed concrete value |
|---|---|---|---|---|
| Main | `counts_positive` | Yes, if unavailable | `NA` | Nonnegative integer, `Option<u64>` |
| Main | `counts_negative` | Yes, if unavailable | `NA` | Nonnegative integer, `Option<u64>` |
| Main | `concentration_reaction_cp_µL` | Yes, if unavailable or no finite estimate exists | `NA` | Finite nonnegative decimal, `Option<f64>` |
| Main | `threshold_if_threshold` | Yes, if inapplicable or unavailable | `NA` | Finite decimal; negative values allowed, `Option<f64>` |
| Intensity | `FI_endpoint` | No, for retained endpoint-measurement rows | Not permitted | Finite decimal; negative values allowed, `f64` |
| Assays | `probe_concentrations_nM` | Yes, for the whole list or individual entries | `NA`, or e.g. `125;NA;125` | Existing semicolon-separated string, each known entry a finite nonnegative `f64` value |

## Encoding and validation

- `NA` is the sole missing marker, case-sensitive. Blank cells, `-`, arbitrary
  text, `NaN`, and infinities are invalid numeric values, not synonyms for missing.
  Writers reject invalid values rather than replacing them with `NA`.
- Zero always denotes a value. Zero positives does not mean unavailable counts;
  zero negatives can accompany saturation. Two known zero counts are different
  from two unknown counts. Do not infer concentration from either case in the reader.
- Counts use decimal digits in the range 0 through 18,446,744,073,709,551,615.
  Fractional, negative, exponential, and overflowing counts are rejected. Leading
  zeros and an optional leading `+` are accepted and normalize on output.
- Floating-point fields accept decimal/scientific notation with normal binary64
  rounding. Overflow to infinity is rejected. Finiteness and nonnegativity where
  applicable are checked on both read and write, including caller-constructed data.
- Surrounding whitespace is ignored when validating numeric tokens. Scalars
  normalize on output; the probe-list string is retained without reordering or
  dropping entries. No rounding to integers or clamping negative values is performed.
- A whole-list `NA` is allowed, as are `NA;NA` and `125;NA;125`. Empty lists and
  empty entries (`125;;125`, `125;`) are invalid. A single `NA` denotes a missing
  whole list; the wire format cannot distinguish that from a single unknown item.
  No new probe-count/channel-cardinality validation is introduced.
- These rules apply through standalone TSV, directory, and ZIP read/write APIs.
  Public Rust scalar fields use the concrete types above. Main/assays Serde values
  retain text-cell encoding, including `NA`; intensity uses native numeric Serde
  on its public record type with the specification's field names. This does not
  change these file-format rules. Float-bearing models implement `PartialEq`,
  not `Eq`.

## Paper basis and questions to take upstream

Source: [April 13, 2026 preprint](https://doi.org/10.64898/2026.04.10.717368).

- Lines 221–223 explicitly use `NA` for information unavailable in platform exports.
  Counts therefore retain missingness rather than silently inventing zero counts.
- Lines 189–190 say a threshold “can be included”; lines 196–197 describe probe
  concentrations “if applicable.” Both need a defined missing-value representation.
- Lines 202–208 restrict intensity files to QC-passing partitions and describe
  instrument-processed measurements. Requiring a finite endpoint value in every
  retained row is our adopted narrowing. Ask the authors to confirm the treatment
  of otherwise-valid partitions with missing channels. The reader rejects such
  rows; it does not silently discard partitions or fill in zeros.
- Propose `NA` for unavailable/nonfinite concentration estimates, including
  saturation, rather than writing infinity or an arbitrary upper bound. This is
  an encoding recommendation; the reader does not calculate an estimate or infer
  why one is absent. `NA` cannot distinguish unavailable, inapplicable, and
  unquantifiable without changing the format, which is outside this decision.
- Whole-list and per-entry missing probe concentrations are approved here and
  should be made explicit in the paper, preserving positional correspondence.

This deliberately breaks the former permissive handling of numeric text and
missing/nonfinite endpoint measurements. It does not tighten date/version grammar,
identifiers, cross-file references, or introduce new columns or streaming APIs.

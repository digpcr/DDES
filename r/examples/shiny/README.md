# DDES Converter & Annotation Software

A local Shiny app for converting QIAcuity and QX200 exports, editing DDES run,
probe, assay, and sample annotations, inspecting components, and downloading a
DDES archive. DDES reads, validation, and writes use the `ddes` R package.

## Run the app

Install the [R package](../../README.md), then run from the repository root:

```r
install.packages(c("shiny", "rhandsontable"))
shiny::runApp("r/examples/shiny", host = "127.0.0.1", launch.browser = TRUE)
```

Keep this example in the checkout: **Start QIAcuity Demo** reads
`archive/demo_data/QIAcuity`. Alternatively upload a DDES ZIP/`.ddes` archive,
or choose a vendor format and supply the files listed below. Click **Start DDES
Conversion** to load uploads. A failed conversion leaves the current dataset
unchanged.

## Annotate and download

- **Run Info:** edit experiment/run IDs, instrument, software version, and
  plate type. Saving updates experiment/run IDs in every component.
- **Probe & Assay Annotation:** edit target names, add target/channel pairs,
  select probes to create an assay, and edit assay names and probe concentrations.
  Concentrations are in nM: `125;NA;0` has a missing entry; `NA` means the whole
  list is missing. Existing channels and probe combinations are read-only in
  the tables; use **Replace Selected Assay's Probes** to change a combination.
  Column widths can be resized.
- **Sample Annotation:** edit sample IDs, or select wells and assign an assay.
  Well and assay columns are read-only; assignment uses the separate selector.
- **Main**, **Assays**, and **Intensity Files:** inspect metadata and records.
  Intensity previews show the first ten measurements for the selected well.
- **Download:** click **Validate Download**, then download the ZIP archive.
  Only saved changes are included. Saving another change requires validation
  again. Errors appear in the sidebar and as notifications.

Table edits need their **Save** button. Add, delete, assign, and replace buttons
take effect immediately using saved definitions and the checked rows. Save a
table before using another action; refreshing a table discards unsaved edits.
Datasets and downloads are private to each browser session and are not persisted
on the server. Closing the session loses edits that have not been downloaded.

Renaming targets or assays preserves reported results and updates linked IDs.
**Assigning a different assay clears the selected wells' counts, concentrations,
and thresholds**, replacing result rows with the new assay's targets. Replacing
an assay's probes also clears its probe concentrations and affected wells'
reported results. Fluorescence measurements stay unchanged. Assigning the same
assay does nothing. There is no clustering, thresholding, or result recalculation.

Assay assignment requires one sample per selected well. Replacing probes is
rejected for affected wells with multiple assays. In-use assays/probes cannot
be deleted; unused probes exist only in the app, not in the downloaded DDES.
The annotation UI requires unique assay IDs and `target.channel;...` probe
combinations. New or renamed targets and new channels cannot contain `.` or `;`.
Rename all channels of a target consistently; merging targets is not supported.

## Vendor inputs

**Conversion is unverified with real vendor exports.** Compare converted data
with its source results before scientific use.
CSV files must be rectangular, with one header row and comma-separated fields;
an initial `sep=,` or `sep=;` line selects the delimiter. Unsupported export
shapes cause an error.

Identifiers retain leading zeros, and counts retain their exact values across
the unsigned 64-bit range. Blank/`NA` optional counts and measurements become missing,
never zero. Counts must be unsigned decimal integers; measurements must be
finite, and reaction concentrations nonnegative. Unsupported tokens such as
`No Call` and missing intensities are rejected.

### QIAcuity

Supply an analysis CSV containing all samples/channels, a plate-layout CSV,
and a ZIP of RFU CSVs for the same wells. Required column names below are
**after R's `make.names()` normalization**. For example, `Plate ID` becomes
`Plate.ID`; check normalized headings when a required column is reported missing.

| Input | Required columns |
| --- | --- |
| Analysis | `Plate.ID`, `Plate.type`, `Well`, `Sample.NTC.Control`, `Reaction.Mix`, `Target..Name.`, `Partitions..Positive.`, `Partitions..Negative.`, `Conc...cp.µL...dPCR.reaction.`, `Threshold` |
| Plate layout | `Well`, `Reaction.mix.name`, and all five `Channel.1`/`Target.1` through `Channel.5`/`Target.5` pairs |
| RFU | `Well`, `Partition`, `Channel`, `RFU` |

RFU filenames must contain `RFU` and end in `.csv` (case-insensitive); nested ZIP
entries are supported. An RFU CSV may start with one descriptive line before
its header. Channel codes `C/G/Y/R/O` map to
`CRIMSON/GREEN/YELLOW/RED/ORANGE`. Partition IDs are preserved across channels.
Conflicting assay definitions, mismatched wells/assay IDs, and duplicate
partition/channel measurements are rejected. Reported counts are not inferred
from RFU row totals.

### QX200 / QuantaSoft

Supply one results CSV and one amplitude CSV per well (individual files, not a
ZIP). Required results columns are `Well`, `Sample`, `TargetType`, `Target`,
`Positives`, `Negatives`, `Concentration`, and `Threshold`. `TargetType` must be
`Ch1`, `Ch2`, `Ch1Unknown`, or `Ch2Unknown`.

Each amplitude filename must contain its exact well between underscores, such
as `plate_A1_Amplitude.csv`. Its first two columns are Ch1 and Ch2; additional
columns are ignored. Both measurements from a row share that row's partition
ID. Assay IDs are sorted `target.channel` combinations and can be renamed after
conversion. Results files with shifted or unequal-width headers are unsupported.

For both vendors, probe concentrations start missing. The export timestamp is
conversion time, not acquisition time; experiment ID starts as `expID` and
software version as `NA`. QIAcuity supplies run/plate IDs from analysis metadata
and starts instrument as `NA`. QX200 uses the results filename stem as run ID,
`QX200` as instrument, and `Droplets` as plate type. Review run metadata before
downloading.

## Limits

This is a local example, not an authenticated hosted service. Upload requests
are limited to 50 MiB, but decompressed data and output have no size limit.
Complete datasets and conversion buffers reside in memory. Use trusted files
and allow extra memory for large datasets.

## Development checks

Run the app's annotation, session-isolation, download, and synthetic conversion
tests from the repository root with `testthat`, `shiny`, and `rhandsontable`
installed:

```sh
Rscript r/examples/shiny/tests.R
```

The QIAcuity test uses the system `zip` command to create a temporary input ZIP.
Execution is verified on macOS arm64; Windows and Linux are unverified.

## Credits

Converter and annotation workflow: Wim Trypsteen and Emile Roose, DIGPCR UGent.

//! Main-file records and schema: reaction summaries and instrument metadata.
//!
//! See [`MainFile::from_tsv`] for reading, editing, and writing a main file.

use std::{io::BufRead, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, numeric_serde, tsv};

/// A main TSV, including metadata and reaction-summary rows.
///
/// See [`Self::from_tsv`] for a read/edit/write example.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MainFile {
    /// Experiment, run, and instrument headers for this component.
    pub metadata: MainMetadata,
    /// Reaction-summary rows in their original order.
    pub records: Vec<MainRecord>,
}

/// Experiment, run, and instrument metadata for a main file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MainMetadata {
    /// DDES version spelling from the file, such as `1.0` or `1.0.0`.
    #[serde(rename = "DDES_version")]
    pub ddes_version: String,
    /// Export timestamp as written; no date grammar is enforced.
    pub datetime: String,
    /// Experiment identifier, also used in generated archive filenames.
    #[serde(rename = "experiment_ID")]
    pub experiment_id: String,
    /// Run identifier, also used in generated archive filenames.
    #[serde(rename = "run_ID")]
    pub run_id: String,
    /// Component discriminator; must be `main` for this file type.
    #[serde(rename = "DDES_type")]
    pub ddes_type: String,
    /// Instrument name reported by the exporter.
    pub instrument: String,
    /// Plate type reported by the exporter.
    pub plate_type: String,
    /// Instrument software version reported by the exporter.
    pub instrument_software_version: String,
}

/// One target's reaction summary for a well, sample, and assay.
///
/// Missing numeric values are written as `NA`; all TSV columns remain required.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MainRecord {
    /// Well identifier; preserved as text without cross-file validation.
    #[serde(rename = "well_ID")]
    pub well_id: String,
    /// Sample identifier, preserving leading zeros and Unicode.
    #[serde(rename = "sample_ID")]
    pub sample_id: String,
    /// Identifier of the assay associated with this result.
    #[serde(rename = "assay_ID")]
    pub assay_id: String,
    /// Target identifier within the assay.
    #[serde(rename = "target_ID")]
    pub target_id: String,
    /// Reported positive-partition count; `None` is encoded as `NA`, not zero.
    #[serde(with = "numeric_serde::optional_count")]
    pub counts_positive: Option<u64>,
    /// Reported negative-partition count; `None` is encoded as `NA`, not zero.
    #[serde(with = "numeric_serde::optional_count")]
    pub counts_negative: Option<u64>,
    /// Finite nonnegative reaction concentration in copies per microlitre, or NA.
    #[serde(
        rename = "concentration_reaction_cp_µL",
        with = "numeric_serde::optional_concentration"
    )]
    pub concentration_reaction_cp_ul: Option<f64>,
    /// Finite threshold, or NA if inapplicable/unavailable. Negative values are allowed.
    #[serde(with = "numeric_serde::optional_threshold")]
    pub threshold_if_threshold: Option<f64>,
}

impl MainFile {
    /// Parses UTF-8 TSV bytes into reaction summaries and metadata.
    ///
    /// `file` labels diagnostics; it is not opened. All rows are loaded into memory.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid UTF-8, malformed headers or rows, invalid
    /// numeric values, or unsupported fields.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use ddes::MainFile;
    /// let mut file = MainFile::from_tsv("main.tsv", &std::fs::read("main.tsv")?)?;
    /// file.metadata.experiment_id = "updated experiment".into();
    /// let bytes = file.to_tsv("updated-main.tsv")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_tsv(file: impl AsRef<Path>, input: &[u8]) -> Result<Self> {
        let file = file.as_ref().to_string_lossy();
        let mut reader = tsv::validated_input(&file, input)?;
        let (metadata, lines) = read_header(&file, &mut reader)?;
        let records = tsv::read_records(&file, reader, lines, &[])?;
        Ok(Self { metadata, records })
    }

    /// Encodes canonical headers, LF endings, and quoted TSV fields.
    ///
    /// `file` labels diagnostics; nothing is written to disk. See
    /// [`Self::from_tsv`] for an example.
    ///
    /// # Errors
    ///
    /// Returns an error for an incorrect `DDES_type`, metadata that cannot fit
    /// a key/value line, NUL characters, or invalid numeric values.
    pub fn to_tsv(&self, file: impl AsRef<Path>) -> Result<Vec<u8>> {
        let file = file.as_ref().to_string_lossy();
        check_type(&file, &self.metadata.ddes_type)?;
        let mut output = String::from("#header\n");
        tsv::write_metadata(&file, &self.metadata, &mut output)?;
        output.push_str("#data\n");
        tsv::write_records(&file, output.into_bytes(), self.records.iter())
    }
}

fn read_header(file: &str, reader: &mut impl BufRead) -> Result<(MainMetadata, usize)> {
    let mut fields = Vec::new();
    let mut line = String::new();
    let mut lines = 0;
    let mut data_marker = false;
    // Leave the column-name line unread for the CSV reader.
    while matches!(
        reader.fill_buf().map_err(|e| Error::io(file, e))?.first(),
        Some(b'#' | b'\r' | b'\n')
    ) {
        line.clear();
        reader
            .read_line(&mut line)
            .map_err(|e| Error::io(file, e))?;
        lines += 1;
        match line.trim_end_matches(['\r', '\n']) {
            "#data" => data_marker = true,
            "" | "#head" | "#header" | "#assay_information" => {}
            line => fields.push(tsv::metadata_field(file, lines, line)?),
        }
    }
    if !data_marker {
        return Err(Error::format(file, 0, "header", "missing #data marker"));
    }
    // Serde checks required, unknown, and duplicate fields directly from the pairs.
    let metadata = MainMetadata::deserialize(serde::de::value::MapDeserializer::<
        _,
        serde::de::value::Error,
    >::new(fields.into_iter()))
    .map_err(|e| Error::format(file, 0, "metadata", e.to_string()))?;
    check_type(file, &metadata.ddes_type)?;
    Ok((metadata, lines))
}

fn check_type(file: &str, actual: &str) -> Result<()> {
    if actual != "main" {
        return Err(Error::format(file, 0, "DDES_type", "expected main"));
    }
    Ok(())
}

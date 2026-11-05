//! Intensity-file records and schema: partition fluorescence for one well.
//!
//! See [`IntensityFile::from_tsv`] for reading, editing, and writing measurements.

use std::{io::BufRead, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, numeric_serde, tsv};

/// An intensity TSV, including well metadata and partition measurements.
///
/// See [`Self::from_tsv`] for a read/edit/write example.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntensityFile {
    /// Experiment, run, well, and assay headers for this component.
    pub metadata: IntensityMetadata,
    /// Partition/channel measurements in their original order.
    pub records: Vec<IntensityRecord>,
}

/// Experiment, run, well, and assay metadata for an intensity file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntensityMetadata {
    /// DDES version spelling from the file, such as `1.0` or `1.0.0`.
    #[serde(rename = "DDES_version")]
    pub ddes_version: String,
    /// Export timestamp as written; no date grammar is enforced.
    pub datetime: String,
    /// Experiment identifier associated with these measurements.
    #[serde(rename = "experiment_ID")]
    pub experiment_id: String,
    /// Run identifier associated with these measurements.
    #[serde(rename = "run_ID")]
    pub run_id: String,
    /// Component discriminator; must be `intensity` for this file type.
    #[serde(rename = "DDES_type")]
    pub ddes_type: String,
    /// Well containing these partitions, also used in the output filename.
    #[serde(rename = "well_ID")]
    pub well_id: String,
    /// Assay identifier for this well; cross-file references are not checked.
    #[serde(rename = "assay_ID")]
    pub assay_id: String,
}

/// One partition's endpoint fluorescence measurement in one detection channel.
///
/// Reading and writing reject missing or nonfinite endpoint values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntensityRecord {
    /// Partition identifier, preserving leading zeros and Unicode.
    #[serde(rename = "partition_ID")]
    pub partition_id: String,
    /// Detection-channel identifier reported by the exporter.
    #[serde(rename = "channel_ID")]
    pub channel_id: String,
    /// Finite endpoint fluorescence; negative processed values are allowed.
    #[serde(rename = "FI_endpoint", with = "numeric_serde::finite_intensity")]
    pub fi_endpoint: f64,
}

impl IntensityFile {
    /// Parses UTF-8 TSV bytes into endpoint fluorescence measurements and metadata.
    ///
    /// `file` labels diagnostics; it is not opened. All rows are loaded into memory.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid UTF-8, malformed headers or rows, missing or
    /// nonfinite endpoint measurements, or unsupported fields.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use ddes::IntensityFile;
    /// let mut file = IntensityFile::from_tsv("intensity.tsv", &std::fs::read("intensity.tsv")?)?;
    /// file.metadata.experiment_id = "updated experiment".into();
    /// let bytes = file.to_tsv("updated-intensity.tsv")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_tsv(file: impl AsRef<Path>, input: &[u8]) -> Result<Self> {
        let file = file.as_ref().to_string_lossy();
        let mut reader = tsv::validated_input(&file, input)?;
        let (metadata, lines) = read_header(&file, &mut reader)?;
        let records = tsv::read_records(&file, reader, lines, &["FI_endpoint"])?;
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
    /// a key/value line, NUL characters, or nonfinite endpoint measurements.
    pub fn to_tsv(&self, file: impl AsRef<Path>) -> Result<Vec<u8>> {
        let file = file.as_ref().to_string_lossy();
        check_type(&file, &self.metadata.ddes_type)?;
        let mut output = String::from("#head\n");
        tsv::write_metadata(&file, &self.metadata, &mut output)?;
        output.push_str("#data\n");
        tsv::write_records(&file, output.into_bytes(), self.records.iter())
    }
}

fn read_header(file: &str, reader: &mut impl BufRead) -> Result<(IntensityMetadata, usize)> {
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
    let metadata = IntensityMetadata::deserialize(serde::de::value::MapDeserializer::<
        _,
        serde::de::value::Error,
    >::new(fields.into_iter()))
    .map_err(|e| Error::format(file, 0, "metadata", e.to_string()))?;
    check_type(file, &metadata.ddes_type)?;
    Ok((metadata, lines))
}

fn check_type(file: &str, actual: &str) -> Result<()> {
    if actual != "intensity" {
        return Err(Error::format(file, 0, "DDES_type", "expected intensity"));
    }
    Ok(())
}

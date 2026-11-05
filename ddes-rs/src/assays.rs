//! Assays-file records and schema: probe combinations and concentrations.
//!
//! See [`AssaysFile::from_tsv`] for reading, editing, and writing assay definitions.

use std::{io::BufRead, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, Result, numeric_serde, tsv};

/// An assays TSV, including metadata and assay descriptions.
///
/// See [`Self::from_tsv`] for a read/edit/write example.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssaysFile {
    /// Experiment and run headers for this component.
    pub metadata: AssaysMetadata,
    /// Assay descriptions in their original order.
    pub records: Vec<AssayRecord>,
}

/// Experiment and run metadata for an assays file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssaysMetadata {
    /// DDES version spelling from the file, such as `1.0` or `1.0.0`.
    #[serde(rename = "DDES_version")]
    pub ddes_version: String,
    /// Export timestamp as written; no date grammar is enforced.
    pub datetime: String,
    /// Experiment identifier associated with these assays.
    #[serde(rename = "experiment_ID")]
    pub experiment_id: String,
    /// Run identifier associated with these assays.
    #[serde(rename = "run_ID")]
    pub run_id: String,
    /// Component discriminator; accepts `assay` and `assays` unchanged.
    #[serde(rename = "DDES_type")]
    pub ddes_type: String,
}

/// One assay's probe combination and corresponding concentrations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssayRecord {
    /// Assay identifier, preserving leading zeros and Unicode.
    #[serde(rename = "assay_ID")]
    pub assay_id: String,
    /// Probe/target and channel combinations, kept as an unparsed list string.
    pub probe_combination: String,
    /// Semicolon-separated finite nonnegative concentrations in nanomolar.
    /// `NA` denotes a missing whole list or an individual entry (e.g. `125;NA;125`).
    /// Entry spelling is preserved; list length is not checked against the probes.
    #[serde(
        rename = "probe_concentrations_nM",
        with = "numeric_serde::probe_concentrations"
    )]
    pub probe_concentrations_nm: String,
}

impl AssaysFile {
    /// Parses UTF-8 TSV bytes into assay definitions and metadata.
    ///
    /// `file` labels diagnostics; it is not opened. All rows are loaded into memory.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid UTF-8, malformed headers or rows, invalid
    /// probe concentrations, or unsupported fields.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use ddes::AssaysFile;
    /// let mut file = AssaysFile::from_tsv("assays.tsv", &std::fs::read("assays.tsv")?)?;
    /// file.metadata.experiment_id = "updated experiment".into();
    /// let bytes = file.to_tsv("updated-assays.tsv")?;
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
    /// a key/value line, NUL characters, or invalid probe concentrations.
    pub fn to_tsv(&self, file: impl AsRef<Path>) -> Result<Vec<u8>> {
        let file = file.as_ref().to_string_lossy();
        check_type(&file, &self.metadata.ddes_type)?;
        let mut output = String::from("#header\n");
        tsv::write_metadata(&file, &self.metadata, &mut output)?;
        output.push_str("#data\n#assay_information\n");
        tsv::write_records(&file, output.into_bytes(), self.records.iter())
    }
}

fn read_header(file: &str, reader: &mut impl BufRead) -> Result<(AssaysMetadata, usize)> {
    let mut fields = Vec::new();
    let mut line = String::new();
    let mut lines = 0;
    let mut data_marker = false;
    // Consume #assay_information too, leaving the column-name line unread.
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
    let metadata = AssaysMetadata::deserialize(serde::de::value::MapDeserializer::<
        _,
        serde::de::value::Error,
    >::new(fields.into_iter()))
    .map_err(|e| Error::format(file, 0, "metadata", e.to_string()))?;
    check_type(file, &metadata.ddes_type)?;
    Ok((metadata, lines))
}

fn check_type(file: &str, actual: &str) -> Result<()> {
    // Retain the reference reader's singular spelling unchanged.
    if !matches!(actual, "assay" | "assays") {
        return Err(Error::format(file, 0, "DDES_type", "expected assays"));
    }
    Ok(())
}

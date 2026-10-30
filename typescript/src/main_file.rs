//! Editable reaction summaries and instrument metadata.

use serde::{Deserialize, Deserializer, Serialize, de::Error};
use tsify::Tsify;
use wasm_bindgen::JsValue;

/// A detached, editable main TSV containing metadata and reaction-summary rows.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MainFile {
    /// Experiment, run, and instrument headers.
    pub metadata: MainMetadata,
    /// Reaction summaries in input order. Add, remove, or replace rows freely.
    pub records: Vec<MainRecord>,
}

/// Experiment, run, and instrument headers for a main file.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MainMetadata {
    /// DDES version spelling, such as "1.0" or "1.0.0".
    pub ddes_version: String,
    /// Export timestamp as written; no date grammar is enforced.
    pub datetime: String,
    /// Experiment identifier; also used in generated archive filenames.
    pub experiment_id: String,
    /// Run identifier; also used in generated archive filenames.
    pub run_id: String,
    /// Component discriminator; must be "main" when writing.
    pub ddes_type: String,
    /// Instrument name reported by the exporter.
    pub instrument: String,
    /// Plate type reported by the exporter.
    pub plate_type: String,
    /// Instrument software version reported by the exporter.
    pub instrument_software_version: String,
}

/// One target's reported results. All properties are required; null means NA.
#[derive(Tsify, Serialize, Deserialize)]
#[tsify(missing_as_null, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MainRecord {
    /// Well identifier, preserved as text.
    pub well_id: String,
    /// Sample identifier, preserving leading zeros and Unicode.
    pub sample_id: String,
    /// Assay identifier, without cross-file consistency checks.
    pub assay_id: String,
    /// Target identifier within the assay.
    pub target_id: String,
    /// Positive count: bigint from 0n through 18446744073709551615n, or null.
    #[serde(deserialize_with = "optional_count")]
    pub counts_positive: Option<u64>,
    /// Negative count: bigint from 0n through 18446744073709551615n, or null.
    #[serde(deserialize_with = "optional_count")]
    pub counts_negative: Option<u64>,
    /// Finite nonnegative concentration in copies/µL, or null. Zero is not missing.
    #[serde(deserialize_with = "optional_measurement")]
    pub concentration_reaction_cp_ul: Option<f64>,
    /// Finite threshold, or null if unavailable/inapplicable. Negative values are valid.
    #[serde(deserialize_with = "optional_measurement")]
    pub threshold_if_threshold: Option<f64>,
}

fn optional_count<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
    let value: JsValue = serde_wasm_bindgen::preserve::deserialize(deserializer)?;
    if value.is_null() {
        Ok(None)
    } else if value.is_bigint() {
        serde_wasm_bindgen::from_value(value)
            .map(Some)
            .map_err(D::Error::custom)
    } else {
        Err(D::Error::custom("expected a bigint count or null"))
    }
}

fn optional_measurement<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f64>, D::Error> {
    let value: JsValue = serde_wasm_bindgen::preserve::deserialize(deserializer)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_f64()
            .map(Some)
            .ok_or_else(|| D::Error::custom("expected a numeric measurement or null"))
    }
}

impl From<ddes::MainFile> for MainFile {
    fn from(file: ddes::MainFile) -> Self {
        let m = file.metadata;
        Self {
            metadata: MainMetadata {
                ddes_version: m.ddes_version,
                datetime: m.datetime,
                experiment_id: m.experiment_id,
                run_id: m.run_id,
                ddes_type: m.ddes_type,
                instrument: m.instrument,
                plate_type: m.plate_type,
                instrument_software_version: m.instrument_software_version,
            },
            records: file
                .records
                .into_iter()
                .map(|row| MainRecord {
                    well_id: row.well_id,
                    sample_id: row.sample_id,
                    assay_id: row.assay_id,
                    target_id: row.target_id,
                    counts_positive: row.counts_positive,
                    counts_negative: row.counts_negative,
                    concentration_reaction_cp_ul: row.concentration_reaction_cp_ul,
                    threshold_if_threshold: row.threshold_if_threshold,
                })
                .collect(),
        }
    }
}

impl From<MainFile> for ddes::MainFile {
    fn from(file: MainFile) -> Self {
        let m = file.metadata;
        Self {
            metadata: ddes::MainMetadata {
                ddes_version: m.ddes_version,
                datetime: m.datetime,
                experiment_id: m.experiment_id,
                run_id: m.run_id,
                ddes_type: m.ddes_type,
                instrument: m.instrument,
                plate_type: m.plate_type,
                instrument_software_version: m.instrument_software_version,
            },
            records: file
                .records
                .into_iter()
                .map(|row| ddes::MainRecord {
                    well_id: row.well_id,
                    sample_id: row.sample_id,
                    assay_id: row.assay_id,
                    target_id: row.target_id,
                    counts_positive: row.counts_positive,
                    counts_negative: row.counts_negative,
                    concentration_reaction_cp_ul: row.concentration_reaction_cp_ul,
                    threshold_if_threshold: row.threshold_if_threshold,
                })
                .collect(),
        }
    }
}

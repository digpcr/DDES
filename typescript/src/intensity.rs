//! Editable endpoint fluorescence measurements for one well.

use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// A detached, editable JS snapshot, not a live view of WASM memory.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntensityFile {
    /// Experiment, run, well, and assay headers.
    pub metadata: IntensityMetadata,
    /// Endpoint measurements in input order. Add, remove, or replace rows freely.
    pub records: Vec<IntensityRecord>,
}

/// Experiment, run, well, and assay headers for an intensity file.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntensityMetadata {
    /// DDES version spelling, such as "1.0" or "1.0.0".
    pub ddes_version: String,
    /// Export timestamp as written; no date grammar is enforced.
    pub datetime: String,
    /// Experiment identifier recorded in this component.
    pub experiment_id: String,
    /// Run identifier recorded in this component.
    pub run_id: String,
    /// Component discriminator; must be "intensity" when writing.
    pub ddes_type: String,
    /// Well identifier; also used in the generated component filename.
    pub well_id: String,
    /// Assay identifier, without cross-file consistency checks.
    pub assay_id: String,
}

/// One partition's endpoint fluorescence in one channel; the value must be finite.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntensityRecord {
    /// Partition identifier, preserving leading zeros and Unicode.
    pub partition_id: String,
    /// Channel identifier, preserved as text.
    pub channel_id: String,
    /// Required finite endpoint intensity. Zero and negative values are accepted.
    pub fi_endpoint: f64,
}

impl From<ddes::IntensityFile> for IntensityFile {
    fn from(file: ddes::IntensityFile) -> Self {
        let m = file.metadata;
        Self {
            metadata: IntensityMetadata {
                ddes_version: m.ddes_version,
                datetime: m.datetime,
                experiment_id: m.experiment_id,
                run_id: m.run_id,
                ddes_type: m.ddes_type,
                well_id: m.well_id,
                assay_id: m.assay_id,
            },
            records: file
                .records
                .into_iter()
                .map(|row| IntensityRecord {
                    partition_id: row.partition_id,
                    channel_id: row.channel_id,
                    fi_endpoint: row.fi_endpoint,
                })
                .collect(),
        }
    }
}

impl From<IntensityFile> for ddes::IntensityFile {
    fn from(file: IntensityFile) -> Self {
        let m = file.metadata;
        Self {
            metadata: ddes::IntensityMetadata {
                ddes_version: m.ddes_version,
                datetime: m.datetime,
                experiment_id: m.experiment_id,
                run_id: m.run_id,
                ddes_type: m.ddes_type,
                well_id: m.well_id,
                assay_id: m.assay_id,
            },
            records: file
                .records
                .into_iter()
                .map(|row| ddes::IntensityRecord {
                    partition_id: row.partition_id,
                    channel_id: row.channel_id,
                    fi_endpoint: row.fi_endpoint,
                })
                .collect(),
        }
    }
}

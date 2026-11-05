//! Editable assay definitions and probe concentrations.

use serde::{Deserialize, Serialize};
use tsify::Tsify;

/// A detached, editable assays TSV containing metadata and assay definitions.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssaysFile {
    /// Experiment and run headers.
    pub metadata: AssaysMetadata,
    /// Assay definitions in input order. Add, remove, or replace rows freely.
    pub records: Vec<AssayRecord>,
}

/// Experiment and run headers for an assays file.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssaysMetadata {
    /// DDES version spelling, such as "1.0" or "1.0.0".
    pub ddes_version: String,
    /// Export timestamp as written; no date grammar is enforced.
    pub datetime: String,
    /// Experiment identifier recorded in this component.
    pub experiment_id: String,
    /// Run identifier recorded in this component.
    pub run_id: String,
    /// Component discriminator; accepts "assay" and "assays" unchanged.
    pub ddes_type: String,
}

/// One assay's probe combination and corresponding concentration list.
#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssayRecord {
    /// Assay identifier, preserving leading zeros and Unicode.
    pub assay_id: String,
    /// Semicolon-separated probe/target and channel combinations, kept as text.
    pub probe_combination: String,
    /// Semicolon-separated finite nonnegative concentrations in nM.
    /// "NA" means a missing whole list or entry, e.g. "125;NA;0".
    /// Spelling is preserved; list lengths are not cross-checked.
    pub probe_concentrations_nm: String,
}

impl From<ddes::AssaysFile> for AssaysFile {
    fn from(file: ddes::AssaysFile) -> Self {
        let m = file.metadata;
        Self {
            metadata: AssaysMetadata {
                ddes_version: m.ddes_version,
                datetime: m.datetime,
                experiment_id: m.experiment_id,
                run_id: m.run_id,
                ddes_type: m.ddes_type,
            },
            records: file
                .records
                .into_iter()
                .map(|row| AssayRecord {
                    assay_id: row.assay_id,
                    probe_combination: row.probe_combination,
                    probe_concentrations_nm: row.probe_concentrations_nm,
                })
                .collect(),
        }
    }
}

impl From<AssaysFile> for ddes::AssaysFile {
    fn from(file: AssaysFile) -> Self {
        let m = file.metadata;
        Self {
            metadata: ddes::AssaysMetadata {
                ddes_version: m.ddes_version,
                datetime: m.datetime,
                experiment_id: m.experiment_id,
                run_id: m.run_id,
                ddes_type: m.ddes_type,
            },
            records: file
                .records
                .into_iter()
                .map(|row| ddes::AssayRecord {
                    assay_id: row.assay_id,
                    probe_combination: row.probe_combination,
                    probe_concentrations_nm: row.probe_concentrations_nm,
                })
                .collect(),
        }
    }
}

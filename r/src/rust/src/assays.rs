//! Assay definitions and probe concentrations.

use ddes::{AssayRecord, AssaysFile, AssaysMetadata};
use extendr_api::prelude::*;

use crate::{Result, convert::*};

/// Read a complete assays TSV from raw bytes into metadata and a data frame.
#[extendr]
fn read_assays(bytes: Raw) -> Result<List> {
    to_r(AssaysFile::from_tsv("assays", bytes.as_slice()).map_err(error)?)
}

/// Validate an assays component and return normalized TSV bytes without modifying it.
#[extendr]
fn write_assays(file: List) -> Result<Raw> {
    Ok(Raw::from_bytes(
        &from_r(file.into())?.to_tsv("assays").map_err(error)?,
    ))
}

pub(crate) fn to_r(file: AssaysFile) -> Result<List> {
    let m = file.metadata;
    let rows = &file.records;
    Ok(list!(
        metadata = list!(
            ddes_version = m.ddes_version,
            datetime = m.datetime,
            experiment_id = m.experiment_id,
            run_id = m.run_id,
            ddes_type = m.ddes_type
        ),
        records = dataframe(
            list!(
                assay_id = rows
                    .iter()
                    .map(|r| r.assay_id.as_str())
                    .collect::<Strings>(),
                probe_combination = rows
                    .iter()
                    .map(|r| r.probe_combination.as_str())
                    .collect::<Strings>(),
                probe_concentrations_nm = rows
                    .iter()
                    .map(|r| r.probe_concentrations_nm.as_str())
                    .collect::<Strings>()
            ),
            rows.len()
        )?
    ))
}

pub(crate) fn from_r(value: Robj) -> Result<AssaysFile> {
    let file = fields(value, &["metadata", "records"])?;
    let m = fields(
        get(&file, "metadata")?,
        &[
            "ddes_version",
            "datetime",
            "experiment_id",
            "run_id",
            "ddes_type",
        ],
    )?;
    let metadata = AssaysMetadata {
        ddes_version: text(&m, "ddes_version")?,
        datetime: text(&m, "datetime")?,
        experiment_id: text(&m, "experiment_id")?,
        run_id: text(&m, "run_id")?,
        ddes_type: text(&m, "ddes_type")?,
    };
    let (r, n) = records(
        &file,
        &["assay_id", "probe_combination", "probe_concentrations_nm"],
    )?;
    let assay = strings(&r, "assay_id")?;
    let probe = strings(&r, "probe_combination")?;
    let concentrations = strings(&r, "probe_concentrations_nm")?;
    let records = (0..n)
        .map(|i| AssayRecord {
            assay_id: assay[i].clone(),
            probe_combination: probe[i].clone(),
            probe_concentrations_nm: concentrations[i].clone(),
        })
        .collect();
    Ok(AssaysFile { metadata, records })
}

extendr_module! { mod assays; fn read_assays; fn write_assays; }

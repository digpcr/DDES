//! Per-well endpoint fluorescence measurements.

use ddes::{IntensityFile, IntensityMetadata, IntensityRecord};
use extendr_api::prelude::*;

use crate::{Result, convert::*};

/// Read a complete intensity TSV from raw bytes into metadata and a data frame.
#[extendr]
fn read_intensity(bytes: Raw) -> Result<List> {
    to_r(IntensityFile::from_tsv("intensity", bytes.as_slice()).map_err(error)?)
}

/// Validate an intensity component and return normalized TSV bytes without modifying it.
#[extendr]
fn write_intensity(file: List) -> Result<Raw> {
    Ok(Raw::from_bytes(
        &from_r(file.into())?.to_tsv("intensity").map_err(error)?,
    ))
}

pub(crate) fn to_r(file: IntensityFile) -> Result<List> {
    let m = file.metadata;
    let rows = &file.records;
    Ok(list!(
        metadata = list!(
            ddes_version = m.ddes_version,
            datetime = m.datetime,
            experiment_id = m.experiment_id,
            run_id = m.run_id,
            ddes_type = m.ddes_type,
            well_id = m.well_id,
            assay_id = m.assay_id
        ),
        records = dataframe(
            list!(
                partition_id = rows
                    .iter()
                    .map(|r| r.partition_id.as_str())
                    .collect::<Strings>(),
                channel_id = rows
                    .iter()
                    .map(|r| r.channel_id.as_str())
                    .collect::<Strings>(),
                fi_endpoint = rows.iter().map(|r| r.fi_endpoint).collect::<Doubles>()
            ),
            rows.len()
        )?
    ))
}

pub(crate) fn from_r(value: Robj) -> Result<IntensityFile> {
    let file = fields(value, &["metadata", "records"])?;
    let m = fields(
        get(&file, "metadata")?,
        &[
            "ddes_version",
            "datetime",
            "experiment_id",
            "run_id",
            "ddes_type",
            "well_id",
            "assay_id",
        ],
    )?;
    let metadata = IntensityMetadata {
        ddes_version: text(&m, "ddes_version")?,
        datetime: text(&m, "datetime")?,
        experiment_id: text(&m, "experiment_id")?,
        run_id: text(&m, "run_id")?,
        ddes_type: text(&m, "ddes_type")?,
        well_id: text(&m, "well_id")?,
        assay_id: text(&m, "assay_id")?,
    };
    let (r, n) = records(&file, &["partition_id", "channel_id", "fi_endpoint"])?;
    let partition = strings(&r, "partition_id")?;
    let channel = strings(&r, "channel_id")?;
    let values = numbers(&r, "fi_endpoint")?;
    let records = (0..n)
        .map(|i| {
            Ok(IntensityRecord {
                partition_id: partition[i].clone(),
                channel_id: channel[i].clone(),
                fi_endpoint: values[i].ok_or("fi_endpoint: NA is not allowed")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(IntensityFile { metadata, records })
}

extendr_module! { mod intensity; fn read_intensity; fn write_intensity; }

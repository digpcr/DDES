//! Reaction summaries and instrument metadata.

use ddes::{MainFile, MainMetadata, MainRecord};
use extendr_api::prelude::*;

use crate::{Result, convert::*};

/// Read a complete main TSV from a raw vector into metadata and a data frame.
#[extendr]
fn read_main(bytes: Raw) -> Result<List> {
    to_r(MainFile::from_tsv("main", bytes.as_slice()).map_err(error)?)
}

/// Validate a main component and return normalized TSV bytes without modifying it.
#[extendr]
fn write_main(file: List) -> Result<Raw> {
    Ok(Raw::from_bytes(
        &from_r(file.into())?.to_tsv("main").map_err(error)?,
    ))
}

pub(crate) fn to_r(file: MainFile) -> Result<List> {
    let m = file.metadata;
    let rows = &file.records;
    Ok(list!(
        metadata = list!(
            ddes_version = m.ddes_version,
            datetime = m.datetime,
            experiment_id = m.experiment_id,
            run_id = m.run_id,
            ddes_type = m.ddes_type,
            instrument = m.instrument,
            plate_type = m.plate_type,
            instrument_software_version = m.instrument_software_version
        ),
        records = dataframe(
            list!(
                well_id = rows.iter().map(|r| r.well_id.as_str()).collect::<Strings>(),
                sample_id = rows
                    .iter()
                    .map(|r| r.sample_id.as_str())
                    .collect::<Strings>(),
                assay_id = rows
                    .iter()
                    .map(|r| r.assay_id.as_str())
                    .collect::<Strings>(),
                target_id = rows
                    .iter()
                    .map(|r| r.target_id.as_str())
                    .collect::<Strings>(),
                counts_positive = count_values(rows.iter().map(|r| r.counts_positive)),
                counts_negative = count_values(rows.iter().map(|r| r.counts_negative)),
                concentration_reaction_cp_ul =
                    number_values(rows.iter().map(|r| r.concentration_reaction_cp_ul)),
                threshold_if_threshold =
                    number_values(rows.iter().map(|r| r.threshold_if_threshold))
            ),
            rows.len()
        )?
    ))
}

pub(crate) fn from_r(value: Robj) -> Result<MainFile> {
    let file = fields(value, &["metadata", "records"])?;
    let m = fields(
        get(&file, "metadata")?,
        &[
            "ddes_version",
            "datetime",
            "experiment_id",
            "run_id",
            "ddes_type",
            "instrument",
            "plate_type",
            "instrument_software_version",
        ],
    )?;
    let metadata = MainMetadata {
        ddes_version: text(&m, "ddes_version")?,
        datetime: text(&m, "datetime")?,
        experiment_id: text(&m, "experiment_id")?,
        run_id: text(&m, "run_id")?,
        ddes_type: text(&m, "ddes_type")?,
        instrument: text(&m, "instrument")?,
        plate_type: text(&m, "plate_type")?,
        instrument_software_version: text(&m, "instrument_software_version")?,
    };
    let (r, n) = records(
        &file,
        &[
            "well_id",
            "sample_id",
            "assay_id",
            "target_id",
            "counts_positive",
            "counts_negative",
            "concentration_reaction_cp_ul",
            "threshold_if_threshold",
        ],
    )?;
    let well = strings(&r, "well_id")?;
    let sample = strings(&r, "sample_id")?;
    let assay = strings(&r, "assay_id")?;
    let target = strings(&r, "target_id")?;
    let positive = counts(&r, "counts_positive")?;
    let negative = counts(&r, "counts_negative")?;
    let concentration = numbers(&r, "concentration_reaction_cp_ul")?;
    let threshold = numbers(&r, "threshold_if_threshold")?;
    let records = (0..n)
        .map(|i| MainRecord {
            well_id: well[i].clone(),
            sample_id: sample[i].clone(),
            assay_id: assay[i].clone(),
            target_id: target[i].clone(),
            counts_positive: positive[i],
            counts_negative: negative[i],
            concentration_reaction_cp_ul: concentration[i],
            threshold_if_threshold: threshold[i],
        })
        .collect();
    Ok(MainFile { metadata, records })
}

extendr_module! { mod main_file; fn read_main; fn write_main; }

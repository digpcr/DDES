//! Dataset lists and ZIP/directory I/O.

use std::io::Cursor;

use ddes::Dataset;
use extendr_api::prelude::*;

use crate::{Result, assays, convert::*, intensity, main_file};

/// Read a flat TSV directory or ZIP archive into an editable dataset list.
#[extendr]
fn read_archive(path: &str) -> Result<List> {
    to_r(Dataset::read_path(path).map_err(error)?)
}

/// Validate a dataset and create a ZIP archive, refusing to overwrite any file.
#[extendr]
fn write_archive(dataset: List, path: &str) -> Result<()> {
    from_r(dataset.into())?.write_path(path).map_err(error)
}

/// Read ZIP bytes without extracting to disk.
#[extendr]
fn archive_from_raw(bytes: Raw) -> Result<List> {
    to_r(Dataset::read_zip(Cursor::new(bytes.as_slice())).map_err(error)?)
}

/// Validate a dataset and return ZIP bytes without modifying the input.
#[extendr]
fn archive_to_raw(dataset: List) -> Result<Raw> {
    let bytes = from_r(dataset.into())?
        .write_zip(Cursor::new(Vec::new()))
        .map_err(error)?
        .into_inner();
    Ok(Raw::from_bytes(&bytes))
}

fn to_r(dataset: Dataset) -> Result<List> {
    let intensities = dataset
        .intensities
        .into_iter()
        .map(intensity::to_r)
        .collect::<Result<Vec<_>>>()?;
    Ok(list!(
        main = main_file::to_r(dataset.main)?,
        assays = assays::to_r(dataset.assays)?,
        intensities = List::from_values(intensities)
    ))
}

fn from_r(value: Robj) -> Result<Dataset> {
    let dataset = fields(value, &["main", "assays", "intensities"])?;
    let values = get(&dataset, "intensities")?
        .as_list()
        .ok_or("intensities: expected a list, including list() if empty")?;
    Ok(Dataset {
        main: main_file::from_r(get(&dataset, "main")?)?,
        assays: assays::from_r(get(&dataset, "assays")?)?,
        intensities: values
            .values()
            .map(intensity::from_r)
            .collect::<Result<_>>()?,
    })
}

extendr_module! { mod archive; fn read_archive; fn write_archive; fn archive_from_raw; fn archive_to_raw; }

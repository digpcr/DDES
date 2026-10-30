//! Editable reaction summaries and instrument metadata.

use pyo3::{
    gc::{PyTraverseError, PyVisit},
    prelude::*,
    types::PyList,
};

/// Experiment, run, and instrument metadata for a main file.
///
/// Parameters
/// ----------
/// ddes_version : str
///     Format version as written, for example "1.0.0".
/// datetime : str
///     Export timestamp; no date grammar is enforced.
/// experiment_id : str
///     Experiment identifier.
/// run_id : str
///     Run identifier.
/// ddes_type : str
///     Component type; must be "main" when writing.
/// instrument : str
///     Instrument description.
/// plate_type : str
///     Plate description.
/// instrument_software_version : str
///     Instrument software version as written.
///
/// Raises
/// ------
/// TypeError
///     An argument is missing or is not a string. All arguments are keyword-only.
///
/// Notes
/// -----
/// Fields are editable. Component type and single-line header constraints are
/// checked on write, not at construction or assignment.
#[pyclass(get_all, set_all, eq, module = "ddes._native")]
#[derive(PartialEq)]
pub struct MainMetadata {
    /// str: Format version spelling, such as "1.0.0".
    ddes_version: String,
    /// str: Export timestamp, retained without date parsing.
    datetime: String,
    /// str: Experiment identifier, preserving Unicode and leading zeros.
    experiment_id: String,
    /// str: Run identifier, preserving Unicode and leading zeros.
    run_id: String,
    /// str: Must be "main" when writing the file.
    ddes_type: String,
    /// str: Instrument description.
    instrument: String,
    /// str: Plate description.
    plate_type: String,
    /// str: Instrument software version as written.
    instrument_software_version: String,
}

#[pymethods]
impl MainMetadata {
    #[new]
    #[allow(clippy::too_many_arguments)] // One required argument per metadata field.
    #[pyo3(signature = (*, ddes_version, datetime, experiment_id, run_id, ddes_type, instrument, plate_type, instrument_software_version))]
    fn new(
        ddes_version: String,
        datetime: String,
        experiment_id: String,
        run_id: String,
        ddes_type: String,
        instrument: String,
        plate_type: String,
        instrument_software_version: String,
    ) -> Self {
        Self {
            ddes_version,
            datetime,
            experiment_id,
            run_id,
            ddes_type,
            instrument,
            plate_type,
            instrument_software_version,
        }
    }
}

impl From<ddes::MainMetadata> for MainMetadata {
    fn from(value: ddes::MainMetadata) -> Self {
        Self {
            ddes_version: value.ddes_version,
            datetime: value.datetime,
            experiment_id: value.experiment_id,
            run_id: value.run_id,
            ddes_type: value.ddes_type,
            instrument: value.instrument,
            plate_type: value.plate_type,
            instrument_software_version: value.instrument_software_version,
        }
    }
}

impl From<&MainMetadata> for ddes::MainMetadata {
    fn from(value: &MainMetadata) -> Self {
        Self {
            ddes_version: value.ddes_version.clone(),
            datetime: value.datetime.clone(),
            experiment_id: value.experiment_id.clone(),
            run_id: value.run_id.clone(),
            ddes_type: value.ddes_type.clone(),
            instrument: value.instrument.clone(),
            plate_type: value.plate_type.clone(),
            instrument_software_version: value.instrument_software_version.clone(),
        }
    }
}

/// One target's reaction summary for a well, sample, and assay.
///
/// Parameters
/// ----------
/// well_id : str
///     Well identifier.
/// sample_id : str
///     Sample identifier.
/// assay_id : str
///     Assay identifier.
/// target_id : str
///     Target identifier.
/// counts_positive : int or None
///     Positive partition count, from 0 through 2**64 - 1, or missing.
/// counts_negative : int or None
///     Negative partition count, from 0 through 2**64 - 1, or missing.
/// concentration_reaction_cp_ul : float or None
///     Finite, nonnegative reaction concentration in copies per microliter,
///     or missing. Integers are also accepted.
/// threshold_if_threshold : float or None
///     Finite fluorescence threshold, or missing. Negative values and integers
///     are accepted. The intensity scale is instrument-dependent.
///
/// Raises
/// ------
/// TypeError
///     An argument is missing or has the wrong type. All arguments are
///     keyword-only and required, including those that accept None.
/// OverflowError
///     A count is outside the unsigned 64-bit range, or a numeric conversion
///     cannot represent the supplied value.
///
/// Notes
/// -----
/// Fields are editable. Counts are checked on assignment; float finiteness and
/// nonnegative concentration are checked on write. None is missing, never zero.
/// Identifiers preserve their spelling. Python bool values follow integer
/// semantics; True converts to 1. Reported results are not recalculated.
///
/// Examples
/// --------
/// >>> from ddes import MainRecord
/// >>> row = MainRecord(well_id="A1", sample_id="001", assay_id="assay",
/// ...     target_id="PSI", counts_positive=0, counts_negative=None,
/// ...     concentration_reaction_cp_ul=None, threshold_if_threshold=-2.5)
/// >>> row.counts_positive = 2**64 - 1
/// >>> row.counts_positive
/// 18446744073709551615
#[pyclass(get_all, set_all, eq, module = "ddes._native")]
#[derive(PartialEq)]
pub struct MainRecord {
    /// str: Well identifier, preserving its original spelling.
    well_id: String,
    /// str: Sample identifier; leading zeros are preserved.
    sample_id: String,
    /// str: Assay identifier.
    assay_id: String,
    /// str: Target identifier.
    target_id: String,
    /// int or None: Positive count in 0..2**64 - 1; None means missing, not zero.
    counts_positive: Option<u64>,
    /// int or None: Negative count in 0..2**64 - 1; None means missing, not zero.
    counts_negative: Option<u64>,
    /// float or None: Copies per microliter; finite and nonnegative on write.
    concentration_reaction_cp_ul: Option<f64>,
    /// float or None: Fluorescence threshold; finite on write, negatives allowed.
    threshold_if_threshold: Option<f64>,
}

#[pymethods]
impl MainRecord {
    #[new]
    #[allow(clippy::too_many_arguments)] // One required argument per record field.
    #[pyo3(signature = (*, well_id, sample_id, assay_id, target_id, counts_positive, counts_negative, concentration_reaction_cp_ul, threshold_if_threshold))]
    fn new(
        well_id: String,
        sample_id: String,
        assay_id: String,
        target_id: String,
        counts_positive: Option<u64>,
        counts_negative: Option<u64>,
        concentration_reaction_cp_ul: Option<f64>,
        threshold_if_threshold: Option<f64>,
    ) -> Self {
        Self {
            well_id,
            sample_id,
            assay_id,
            target_id,
            counts_positive,
            counts_negative,
            concentration_reaction_cp_ul,
            threshold_if_threshold,
        }
    }
}

impl From<ddes::MainRecord> for MainRecord {
    fn from(value: ddes::MainRecord) -> Self {
        Self {
            well_id: value.well_id,
            sample_id: value.sample_id,
            assay_id: value.assay_id,
            target_id: value.target_id,
            counts_positive: value.counts_positive,
            counts_negative: value.counts_negative,
            concentration_reaction_cp_ul: value.concentration_reaction_cp_ul,
            threshold_if_threshold: value.threshold_if_threshold,
        }
    }
}

impl From<&MainRecord> for ddes::MainRecord {
    fn from(value: &MainRecord) -> Self {
        Self {
            well_id: value.well_id.clone(),
            sample_id: value.sample_id.clone(),
            assay_id: value.assay_id.clone(),
            target_id: value.target_id.clone(),
            counts_positive: value.counts_positive,
            counts_negative: value.counts_negative,
            concentration_reaction_cp_ul: value.concentration_reaction_cp_ul,
            threshold_if_threshold: value.threshold_if_threshold,
        }
    }
}

/// A main file with editable metadata and a mutable list of reaction summaries.
///
/// Parameters
/// ----------
/// metadata : MainMetadata
///     Metadata object retained by reference.
/// records : list[MainRecord]
///     Records in table order. The list is copied; its record objects are shared.
///
/// Raises
/// ------
/// TypeError
///     Metadata or records have the wrong types.
///
/// Notes
/// -----
/// Nested edits and list append/delete/replacement persist in this file.
/// Files compare by content. Writers check record types and values, so edits
/// may temporarily be invalid. Archive getters return detached files; use
/// DdesArchive.set_main() to save changes back into an archive.
///
/// Examples
/// --------
/// >>> from ddes import MainFile, MainMetadata, read_main, write_main
/// >>> metadata = MainMetadata(ddes_version="1.0.0", datetime="20260101",
/// ...     experiment_id="exp", run_id="run", ddes_type="main",
/// ...     instrument="instrument", plate_type="plate", instrument_software_version="1")
/// >>> file = MainFile(metadata, [])
/// >>> read_main(write_main(file)) == file
/// True
#[pyclass(get_all, set_all, module = "ddes._native")]
pub struct MainFile {
    /// MainMetadata: Editable metadata; assigning it retains the supplied object.
    metadata: Py<MainMetadata>,
    /// list[MainRecord]: Mutable rows in table order. Writers reject other types.
    records: Py<PyList>,
}

#[pymethods]
impl MainFile {
    #[new]
    fn new(
        py: Python<'_>,
        metadata: Py<MainMetadata>,
        records: Vec<Py<MainRecord>>,
    ) -> PyResult<Self> {
        Ok(Self {
            metadata,
            records: PyList::new(py, records)?.unbind(),
        })
    }

    fn __eq__(&self, other: &Self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.metadata.bind(py).eq(other.metadata.bind(py))?
            && self.records.bind(py).eq(other.records.bind(py))?)
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.metadata)?;
        visit.call(&self.records)
    }

    fn __clear__(&mut self, py: Python<'_>) {
        self.records = PyList::empty(py).unbind();
    }
}

impl MainFile {
    pub fn from_core(py: Python<'_>, file: ddes::MainFile) -> PyResult<Self> {
        let metadata = Py::new(py, MainMetadata::from(file.metadata))?;
        let records = file
            .records
            .into_iter()
            .map(|row| Py::new(py, MainRecord::from(row)))
            .collect::<PyResult<Vec<_>>>()?;
        Self::new(py, metadata, records)
    }

    pub fn to_core(&self, py: Python<'_>) -> PyResult<ddes::MainFile> {
        let metadata = (&*self.metadata.try_borrow(py)?).into();
        let records = self
            .records
            .bind(py)
            .iter()
            .map(|item| {
                let row = item.extract::<PyRef<'_, MainRecord>>()?;
                Ok((&*row).into())
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(ddes::MainFile { metadata, records })
    }
}

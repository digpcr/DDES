//! Editable endpoint fluorescence measurements for one well.

use pyo3::{
    gc::{PyTraverseError, PyVisit},
    prelude::*,
    types::PyList,
};

/// Experiment, run, well, and assay metadata for an intensity file.
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
///     Component type; must be "intensity" when writing.
/// well_id : str
///     Well identifier, also used in the generated archive filename.
/// assay_id : str
///     Assay identifier.
///
/// Raises
/// ------
/// TypeError
///     An argument is missing or is not a string. All arguments are keyword-only.
///
/// Notes
/// -----
/// Fields are editable. Component type and single-line header constraints are
/// checked on write. Archive writes additionally validate generated filenames.
#[pyclass(get_all, set_all, eq, module = "ddes._native")]
#[derive(PartialEq)]
pub struct IntensityMetadata {
    /// str: Format version spelling, such as "1.0.0".
    ddes_version: String,
    /// str: Export timestamp, retained without date parsing.
    datetime: String,
    /// str: Experiment identifier, preserving Unicode and leading zeros.
    experiment_id: String,
    /// str: Run identifier, preserving Unicode and leading zeros.
    run_id: String,
    /// str: Must be "intensity" when writing the file.
    ddes_type: String,
    /// str: Well identifier; must produce a safe, unique archive filename on write.
    well_id: String,
    /// str: Assay identifier.
    assay_id: String,
}

#[pymethods]
impl IntensityMetadata {
    #[new]
    #[pyo3(signature = (*, ddes_version, datetime, experiment_id, run_id, ddes_type, well_id, assay_id))]
    fn new(
        ddes_version: String,
        datetime: String,
        experiment_id: String,
        run_id: String,
        ddes_type: String,
        well_id: String,
        assay_id: String,
    ) -> Self {
        Self {
            ddes_version,
            datetime,
            experiment_id,
            run_id,
            ddes_type,
            well_id,
            assay_id,
        }
    }
}

impl From<ddes::IntensityMetadata> for IntensityMetadata {
    fn from(value: ddes::IntensityMetadata) -> Self {
        Self {
            ddes_version: value.ddes_version,
            datetime: value.datetime,
            experiment_id: value.experiment_id,
            run_id: value.run_id,
            ddes_type: value.ddes_type,
            well_id: value.well_id,
            assay_id: value.assay_id,
        }
    }
}

impl From<&IntensityMetadata> for ddes::IntensityMetadata {
    fn from(value: &IntensityMetadata) -> Self {
        Self {
            ddes_version: value.ddes_version.clone(),
            datetime: value.datetime.clone(),
            experiment_id: value.experiment_id.clone(),
            run_id: value.run_id.clone(),
            ddes_type: value.ddes_type.clone(),
            well_id: value.well_id.clone(),
            assay_id: value.assay_id.clone(),
        }
    }
}

/// One partition's endpoint fluorescence measurement in one detection channel.
///
/// Parameters
/// ----------
/// partition_id : str
///     Partition identifier; leading zeros and Unicode are preserved.
/// channel_id : str
///     Detection channel identifier.
/// fi_endpoint : float
///     Required finite endpoint fluorescence on the instrument's intensity
///     scale. Zero and negative processed values are accepted, as are integers.
///
/// Raises
/// ------
/// TypeError
///     An argument is missing or has the wrong type; intensity cannot be None.
/// OverflowError
///     A numeric conversion cannot represent the supplied value.
///
/// Notes
/// -----
/// Fields are editable. Finiteness is checked on write, not assignment.
/// Python bool follows integer semantics: True converts to 1.0.
///
/// Examples
/// --------
/// >>> from ddes import IntensityRecord
/// >>> row = IntensityRecord("001", "BLUE", -12.75)
/// >>> row.fi_endpoint = 0
/// >>> row.fi_endpoint
/// 0.0
#[pyclass(get_all, set_all, eq, module = "ddes._native")]
#[derive(PartialEq)]
pub struct IntensityRecord {
    /// str: Partition identifier; not converted to a number.
    partition_id: String,
    /// str: Detection channel identifier.
    channel_id: String,
    /// float: Required endpoint fluorescence; finite on write, negatives allowed.
    fi_endpoint: f64,
}

#[pymethods]
impl IntensityRecord {
    #[new]
    fn new(partition_id: String, channel_id: String, fi_endpoint: f64) -> Self {
        Self {
            partition_id,
            channel_id,
            fi_endpoint,
        }
    }
}

impl From<ddes::IntensityRecord> for IntensityRecord {
    fn from(value: ddes::IntensityRecord) -> Self {
        Self {
            partition_id: value.partition_id,
            channel_id: value.channel_id,
            fi_endpoint: value.fi_endpoint,
        }
    }
}

impl From<&IntensityRecord> for ddes::IntensityRecord {
    fn from(value: &IntensityRecord) -> Self {
        Self {
            partition_id: value.partition_id.clone(),
            channel_id: value.channel_id.clone(),
            fi_endpoint: value.fi_endpoint,
        }
    }
}

/// An intensity file with editable metadata and a mutable list of measurements.
///
/// Parameters
/// ----------
/// metadata : IntensityMetadata
///     Metadata object retained by reference.
/// records : list[IntensityRecord]
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
/// DdesArchive.set_intensities() to save the edited collection back.
///
/// Examples
/// --------
/// >>> from ddes import IntensityFile, IntensityMetadata, IntensityRecord
/// >>> from ddes import read_intensity, write_intensity
/// >>> metadata = IntensityMetadata(ddes_version="1.0.0", datetime="20260101",
/// ...     experiment_id="exp", run_id="run", ddes_type="intensity",
/// ...     well_id="A1", assay_id="assay")
/// >>> file = IntensityFile(metadata, [IntensityRecord("001", "BLUE", -12.75)])
/// >>> read_intensity(write_intensity(file)) == file
/// True
#[pyclass(get_all, set_all, module = "ddes._native")]
pub struct IntensityFile {
    /// IntensityMetadata: Editable metadata; assignment retains the supplied object.
    metadata: Py<IntensityMetadata>,
    /// list[IntensityRecord]: Mutable rows in table order. Writers reject other types.
    records: Py<PyList>,
}

#[pymethods]
impl IntensityFile {
    #[new]
    fn new(
        py: Python<'_>,
        metadata: Py<IntensityMetadata>,
        records: Vec<Py<IntensityRecord>>,
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

impl IntensityFile {
    pub fn from_core(py: Python<'_>, file: ddes::IntensityFile) -> PyResult<Self> {
        let metadata = Py::new(py, IntensityMetadata::from(file.metadata))?;
        let records = file
            .records
            .into_iter()
            .map(|row| Py::new(py, IntensityRecord::from(row)))
            .collect::<PyResult<Vec<_>>>()?;
        Self::new(py, metadata, records)
    }

    pub fn to_core(&self, py: Python<'_>) -> PyResult<ddes::IntensityFile> {
        let metadata = (&*self.metadata.try_borrow(py)?).into();
        let records = self
            .records
            .bind(py)
            .iter()
            .map(|item| {
                let row = item.extract::<PyRef<'_, IntensityRecord>>()?;
                Ok((&*row).into())
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(ddes::IntensityFile { metadata, records })
    }
}

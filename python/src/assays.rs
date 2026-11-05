//! Editable assay definitions and probe concentrations.

use pyo3::{
    gc::{PyTraverseError, PyVisit},
    prelude::*,
    types::PyList,
};

/// Experiment and run metadata for an assays file.
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
///     Component type: "assay" or "assays". The spelling is preserved.
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
pub struct AssaysMetadata {
    /// str: Format version spelling, such as "1.0.0".
    ddes_version: String,
    /// str: Export timestamp, retained without date parsing.
    datetime: String,
    /// str: Experiment identifier, preserving Unicode and leading zeros.
    experiment_id: String,
    /// str: Run identifier, preserving Unicode and leading zeros.
    run_id: String,
    /// str: "assay" or "assays"; checked when writing the file.
    ddes_type: String,
}

#[pymethods]
impl AssaysMetadata {
    #[new]
    #[pyo3(signature = (*, ddes_version, datetime, experiment_id, run_id, ddes_type))]
    fn new(
        ddes_version: String,
        datetime: String,
        experiment_id: String,
        run_id: String,
        ddes_type: String,
    ) -> Self {
        Self {
            ddes_version,
            datetime,
            experiment_id,
            run_id,
            ddes_type,
        }
    }
}

impl From<ddes::AssaysMetadata> for AssaysMetadata {
    fn from(value: ddes::AssaysMetadata) -> Self {
        Self {
            ddes_version: value.ddes_version,
            datetime: value.datetime,
            experiment_id: value.experiment_id,
            run_id: value.run_id,
            ddes_type: value.ddes_type,
        }
    }
}

impl From<&AssaysMetadata> for ddes::AssaysMetadata {
    fn from(value: &AssaysMetadata) -> Self {
        Self {
            ddes_version: value.ddes_version.clone(),
            datetime: value.datetime.clone(),
            experiment_id: value.experiment_id.clone(),
            run_id: value.run_id.clone(),
            ddes_type: value.ddes_type.clone(),
        }
    }
}

/// One assay's probe combination and semicolon-separated concentrations in nM.
///
/// Parameters
/// ----------
/// assay_id : str
///     Assay identifier; leading zeros and Unicode are preserved.
/// probe_combination : str
///     Probe/target and channel combinations, kept as an unparsed list string.
/// probe_concentrations_nm : str
///     Semicolon-separated finite, nonnegative concentrations in nanomolar.
///     "NA" means a missing whole list or individual entry, as in "125;NA;0".
///
/// Raises
/// ------
/// TypeError
///     An argument is missing or is not a string.
///
/// Notes
/// -----
/// Fields are editable. Concentrations are validated on write; list lengths
/// are not checked against probe combinations. Numeric spelling is preserved.
///
/// Examples
/// --------
/// >>> from ddes import AssayRecord
/// >>> row = AssayRecord("001", "PSI.GREEN;ENV.YELLOW", "125;NA")
/// >>> row.probe_concentrations_nm = "0;NA"
/// >>> row.assay_id
/// '001'
#[pyclass(get_all, set_all, eq, module = "ddes._native")]
#[derive(PartialEq)]
pub struct AssayRecord {
    /// str: Assay identifier; not converted to a number.
    assay_id: String,
    /// str: Unparsed probe/target and channel combination list.
    probe_combination: String,
    /// str: Semicolon-separated nM values; "NA" marks a missing list or entry.
    /// Numeric entries must be finite and nonnegative; checked on write.
    probe_concentrations_nm: String,
}

#[pymethods]
impl AssayRecord {
    #[new]
    fn new(assay_id: String, probe_combination: String, probe_concentrations_nm: String) -> Self {
        Self {
            assay_id,
            probe_combination,
            probe_concentrations_nm,
        }
    }
}

impl From<ddes::AssayRecord> for AssayRecord {
    fn from(value: ddes::AssayRecord) -> Self {
        Self {
            assay_id: value.assay_id,
            probe_combination: value.probe_combination,
            probe_concentrations_nm: value.probe_concentrations_nm,
        }
    }
}

impl From<&AssayRecord> for ddes::AssayRecord {
    fn from(value: &AssayRecord) -> Self {
        Self {
            assay_id: value.assay_id.clone(),
            probe_combination: value.probe_combination.clone(),
            probe_concentrations_nm: value.probe_concentrations_nm.clone(),
        }
    }
}

/// An assays file with editable metadata and a mutable list of assay definitions.
///
/// Parameters
/// ----------
/// metadata : AssaysMetadata
///     Metadata object retained by reference.
/// records : list[AssayRecord]
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
/// DdesArchive.set_assays() to save changes back into an archive.
///
/// Examples
/// --------
/// >>> from ddes import AssaysFile, AssaysMetadata, AssayRecord, read_assays, write_assays
/// >>> metadata = AssaysMetadata(ddes_version="1.0.0", datetime="20260101",
/// ...     experiment_id="exp", run_id="run", ddes_type="assays")
/// >>> file = AssaysFile(metadata, [AssayRecord("001", "PSI.GREEN", "NA")])
/// >>> file.records[0].probe_concentrations_nm = "0"
/// >>> read_assays(write_assays(file)) == file
/// True
#[pyclass(get_all, set_all, module = "ddes._native")]
pub struct AssaysFile {
    /// AssaysMetadata: Editable metadata; assigning it retains the supplied object.
    metadata: Py<AssaysMetadata>,
    /// list[AssayRecord]: Mutable rows in table order. Writers reject other types.
    records: Py<PyList>,
}

#[pymethods]
impl AssaysFile {
    #[new]
    fn new(
        py: Python<'_>,
        metadata: Py<AssaysMetadata>,
        records: Vec<Py<AssayRecord>>,
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

impl AssaysFile {
    pub fn from_core(py: Python<'_>, file: ddes::AssaysFile) -> PyResult<Self> {
        let metadata = Py::new(py, AssaysMetadata::from(file.metadata))?;
        let records = file
            .records
            .into_iter()
            .map(|row| Py::new(py, AssayRecord::from(row)))
            .collect::<PyResult<Vec<_>>>()?;
        Self::new(py, metadata, records)
    }

    pub fn to_core(&self, py: Python<'_>) -> PyResult<ddes::AssaysFile> {
        let metadata = (&*self.metadata.try_borrow(py)?).into();
        let records = self
            .records
            .bind(py)
            .iter()
            .map(|item| {
                let row = item.extract::<PyRef<'_, AssayRecord>>()?;
                Ok((&*row).into())
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(ddes::AssaysFile { metadata, records })
    }
}

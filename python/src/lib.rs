//! Read, edit, and write DDES archives and all three component types from Python.

use std::{io::Cursor, path::PathBuf};

use pyo3::{
    exceptions::{PyIndexError, PyOSError, PyValueError},
    prelude::*,
    types::PyBytes,
};

mod assays;
mod intensity;
mod main_file;

use assays::{AssayRecord, AssaysFile, AssaysMetadata};
use intensity::{IntensityFile, IntensityMetadata, IntensityRecord};
use main_file::{MainFile, MainMetadata, MainRecord};

fn core_error(error: ddes::Error) -> PyErr {
    match error {
        ddes::Error::Io { file, source } => {
            // Let Python choose the exception subclass from the OS error code.
            #[cfg(windows)]
            let args = (None::<i32>, source.to_string(), file, source.raw_os_error());
            #[cfg(not(windows))]
            let args = (source.raw_os_error(), source.to_string(), file);
            PyOSError::new_err(args)
        }
        error => PyValueError::new_err(error.to_string()),
    }
}

/// Parse reaction summaries and instrument metadata from a main TSV.
///
/// Parameters
/// ----------
/// data : bytes
///     Complete UTF-8 TSV, including metadata and column headers.
///
/// Returns
/// -------
/// MainFile
///     Independently owned metadata and records in input order. NA cells in
///     numeric columns become None; zero remains zero.
///
/// Raises
/// ------
/// TypeError
///     Input is not bytes.
/// ValueError
///     Invalid encoding, headers, columns, or numeric values. Diagnostics use
///     the label "main.tsv"; no file is opened.
///
/// See Also
/// --------
/// MainFile : Construction and read/write example.
#[pyfunction]
fn read_main(py: Python<'_>, data: &[u8]) -> PyResult<MainFile> {
    let file = py
        .detach(|| ddes::MainFile::from_tsv("main.tsv", data))
        .map_err(core_error)?;
    MainFile::from_core(py, file)
}

/// Serialize a main file as normalized UTF-8 TSV bytes.
///
/// Parameters
/// ----------
/// file : MainFile
///     File to serialize; not modified by writing.
///
/// Returns
/// -------
/// bytes
///     UTF-8 TSV with canonical headers, LF endings, and quoted cells.
///     None becomes NA; numeric formatting may normalize. Nothing is written to disk.
///
/// Raises
/// ------
/// TypeError
///     File or record objects have the wrong types.
/// ValueError
///     Invalid metadata, NUL characters, nonfinite measurements, or negative
///     concentration. Diagnostics use the label "main.tsv".
#[pyfunction]
fn write_main<'py>(py: Python<'py>, file: &MainFile) -> PyResult<Bound<'py, PyBytes>> {
    let file = file.to_core(py)?;
    let bytes = py.detach(|| file.to_tsv("main.tsv")).map_err(core_error)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Parse an endpoint-intensity TSV into an editable IntensityFile.
///
/// Parameters
/// ----------
/// data : bytes
///     Complete UTF-8 TSV, including metadata and column headers.
///
/// Returns
/// -------
/// IntensityFile
///     Independently owned metadata and records in input order, with numeric
///     endpoint intensities. Zero and negative processed values are accepted.
///
/// Raises
/// ------
/// TypeError
///     Input is not bytes.
/// ValueError
///     Invalid encoding, headers, columns, or missing/nonfinite intensity.
///     Diagnostics use "intensity.tsv"; no file is opened.
///
/// See Also
/// --------
/// IntensityFile : Construction and read/write example.
#[pyfunction]
fn read_intensity(py: Python<'_>, data: &[u8]) -> PyResult<IntensityFile> {
    let file = py
        .detach(|| ddes::IntensityFile::from_tsv("intensity.tsv", data))
        .map_err(core_error)?;
    IntensityFile::from_core(py, file)
}

/// Serialize an intensity file as normalized UTF-8 TSV bytes.
///
/// Parameters
/// ----------
/// file : IntensityFile
///     File to serialize; not modified by writing.
///
/// Returns
/// -------
/// bytes
///     UTF-8 TSV with canonical headers, LF endings, and quoted cells.
///     Numeric formatting may normalize. Nothing is written to disk.
///
/// Raises
/// ------
/// TypeError
///     File or record objects have the wrong types.
/// ValueError
///     Invalid metadata, NUL characters, or nonfinite intensity.
///     Diagnostics use the label "intensity.tsv".
#[pyfunction]
fn write_intensity<'py>(py: Python<'py>, file: &IntensityFile) -> PyResult<Bound<'py, PyBytes>> {
    let file = file.to_core(py)?;
    let bytes = py
        .detach(|| file.to_tsv("intensity.tsv"))
        .map_err(core_error)?;
    Ok(PyBytes::new(py, &bytes))
}

/// Parse an assays TSV into editable assay definitions.
///
/// Parameters
/// ----------
/// data : bytes
///     Complete UTF-8 TSV, including metadata and column headers.
///
/// Returns
/// -------
/// AssaysFile
///     Independently owned metadata and records in input order. Probe lists
///     remain strings, including whole-list or individual NA concentrations.
///
/// Raises
/// ------
/// TypeError
///     Input is not bytes.
/// ValueError
///     Invalid encoding, headers, columns, or concentrations.
///     Diagnostics use "assays.tsv"; no file is opened.
///
/// See Also
/// --------
/// AssaysFile : Construction and read/write example.
#[pyfunction]
fn read_assays(py: Python<'_>, data: &[u8]) -> PyResult<AssaysFile> {
    let file = py
        .detach(|| ddes::AssaysFile::from_tsv("assays.tsv", data))
        .map_err(core_error)?;
    AssaysFile::from_core(py, file)
}

/// Serialize an assays file as normalized UTF-8 TSV bytes.
///
/// Parameters
/// ----------
/// file : AssaysFile
///     File to serialize; not modified by writing.
///
/// Returns
/// -------
/// bytes
///     UTF-8 TSV with canonical headers, LF endings, and quoted cells.
///     Probe-list spelling is preserved. Nothing is written to disk.
///
/// Raises
/// ------
/// TypeError
///     File or record objects have the wrong types.
/// ValueError
///     Invalid metadata, NUL characters, or invalid concentration entries.
///     Numeric entries must be finite and nonnegative; NA is accepted.
///     Diagnostics use the label "assays.tsv".
#[pyfunction]
fn write_assays<'py>(py: Python<'py>, file: &AssaysFile) -> PyResult<Bound<'py, PyBytes>> {
    let file = file.to_core(py)?;
    let bytes = py
        .detach(|| file.to_tsv("assays.tsv"))
        .map_err(core_error)?;
    Ok(PyBytes::new(py, &bytes))
}

/// An in-memory DDES archive. Editing a returned snapshot does not change it.
///
/// Parameters
/// ----------
/// main : MainFile
///     Reaction summaries and instrument metadata.
/// assays : AssaysFile
///     Assay definitions and probe concentrations.
/// intensities : list[IntensityFile]
///     Per-well measurements in desired order; [] means none.
///
/// Raises
/// ------
/// TypeError
///     Components or their records have the wrong types.
///
/// Notes
/// -----
/// Construction and setters copy all component data. Later changes to inputs
/// do not alter the archive. Getters also return copies; explicitly use setters
/// to save edits. Values and generated filenames are validated on write.
/// Reads sort intensity files by well ID. Cross-file scientific consistency
/// is not checked, and reported results are never recalculated.
/// All operations load complete datasets into memory without size limits.
///
/// Examples
/// --------
/// >>> from ddes import DdesArchive, MainFile, MainMetadata, AssaysFile, AssaysMetadata
/// >>> main = MainFile(MainMetadata(ddes_version="1.0.0", datetime="20260101",
/// ...     experiment_id="exp", run_id="run", ddes_type="main", instrument="test",
/// ...     plate_type="plate", instrument_software_version="1"), [])
/// >>> assays = AssaysFile(AssaysMetadata(ddes_version="1.0.0", datetime="20260101",
/// ...     experiment_id="exp", run_id="run", ddes_type="assays"), [])
/// >>> archive = DdesArchive(main, assays, [])
/// >>> edited = archive.main()
/// >>> edited.metadata.instrument = "updated"
/// >>> archive.main().metadata.instrument
/// 'test'
/// >>> archive.set_main(edited)
/// >>> restored = DdesArchive.from_bytes(archive.to_bytes())
/// >>> restored.main().metadata.instrument
/// 'updated'
/// >>> from pathlib import Path
/// >>> from tempfile import TemporaryDirectory
/// >>> with TemporaryDirectory() as directory:
/// ...     path = Path(directory) / "results.ddes"
/// ...     archive.write_path(path)
/// ...     assert DdesArchive.read_path(path).main() == edited
#[pyclass(module = "ddes._native")]
struct DdesArchive {
    inner: ddes::Dataset,
}

#[pymethods]
impl DdesArchive {
    /// Copy components into a new archive. Scientific validation occurs on write.
    #[new]
    fn new(
        py: Python<'_>,
        main: &MainFile,
        assays: &AssaysFile,
        intensities: Vec<PyRef<'_, IntensityFile>>,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: ddes::Dataset {
                main: main.to_core(py)?,
                assays: assays.to_core(py)?,
                intensities: intensities
                    .iter()
                    .map(|file| file.to_core(py))
                    .collect::<PyResult<_>>()?,
            },
        })
    }

    /// Read a directory or ZIP/.ddes path into memory without input limits.
    ///
    /// Parameters
    /// ----------
    /// path : str or os.PathLike[str]
    ///     Flat directory of component TSVs or a ZIP file, regardless of extension.
    ///     Component filenames must be portable UTF-8 basenames.
    ///
    /// Returns
    /// -------
    /// DdesArchive
    ///     Complete dataset with intensity files sorted by well ID.
    ///
    /// Raises
    /// ------
    /// TypeError
    ///     Path has the wrong type.
    /// OSError
    ///     A filesystem operation fails. OS errors retain errno and filename
    ///     and use subclasses such as FileNotFoundError or PermissionError.
    /// ValueError
    ///     Malformed ZIP/TSV data, unsupported components, unsafe names,
    ///     duplicate components, or missing main/assays files.
    #[staticmethod]
    fn read_path(py: Python<'_>, path: PathBuf) -> PyResult<Self> {
        py.detach(|| ddes::Dataset::read_path(path))
            .map(|inner| Self { inner })
            .map_err(core_error)
    }

    /// Read ZIP/.ddes bytes into memory without input limits.
    ///
    /// Parameters
    /// ----------
    /// data : bytes
    ///     Complete ZIP archive; members are not extracted to disk.
    ///
    /// Returns
    /// -------
    /// DdesArchive
    ///     Complete dataset with intensity files sorted by well ID.
    ///
    /// Raises
    /// ------
    /// TypeError
    ///     Input is not bytes.
    /// ValueError
    ///     Malformed archive or components, unsafe names, duplicates, or missing
    ///     main/assays files.
    /// OSError
    ///     Reading a decompressed member fails.
    #[staticmethod]
    fn from_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Self> {
        py.detach(|| ddes::Dataset::read_zip(Cursor::new(data)))
            .map(|inner| Self { inner })
            .map_err(core_error)
    }

    /// Return the dataset as normalized ZIP bytes.
    ///
    /// Returns
    /// -------
    /// bytes
    ///     Independent ZIP bytes containing all components. Formatting and ZIP
    ///     metadata may differ from the input; the archive is not modified.
    ///
    /// Raises
    /// ------
    /// ValueError
    ///     Invalid data, unsafe/duplicate output filenames, or ZIP failures.
    fn to_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = py
            .detach(|| self.inner.write_zip(Cursor::new(Vec::new())))
            .map_err(core_error)?
            .into_inner();
        Ok(PyBytes::new(py, &bytes))
    }

    /// Write a ZIP/.ddes file without overwriting an existing destination.
    ///
    /// Parameters
    /// ----------
    /// path : str or os.PathLike[str]
    ///     New destination file. Parent directories must already exist.
    ///
    /// Returns
    /// -------
    /// None
    ///
    /// Raises
    /// ------
    /// TypeError
    ///     Path has the wrong type.
    /// ValueError
    ///     Invalid data, unsafe/duplicate output filenames, or ZIP failures.
    /// OSError
    ///     Destination exists (FileExistsError) or a filesystem operation fails.
    ///     OS errors retain errno and filename.
    ///
    /// Notes
    /// -----
    /// Data and filenames are validated before creating the destination.
    /// An I/O failure may leave a partial file; remove it before retrying.
    fn write_path(&self, py: Python<'_>, path: PathBuf) -> PyResult<()> {
        py.detach(|| self.inner.write_path(path))
            .map_err(core_error)
    }

    /// int: Number of per-well intensity files in the archive (read-only).
    #[getter]
    fn intensity_count(&self) -> usize {
        self.inner.intensities.len()
    }

    /// Return a detached main-file snapshot. Editing it does not edit this archive.
    ///
    /// Returns
    /// -------
    /// MainFile
    ///     Independent editable copy. Use set_main() to save changes back.
    fn main(&self, py: Python<'_>) -> PyResult<MainFile> {
        MainFile::from_core(py, self.inner.main.clone())
    }

    /// Return a detached assays snapshot. Editing it does not edit this archive.
    ///
    /// Returns
    /// -------
    /// AssaysFile
    ///     Independent editable copy. Use set_assays() to save changes back.
    fn assays(&self, py: Python<'_>) -> PyResult<AssaysFile> {
        AssaysFile::from_core(py, self.inner.assays.clone())
    }

    /// Return detached intensity snapshots in the archive's current order.
    ///
    /// Returns
    /// -------
    /// list[IntensityFile]
    ///     Independent editable copies. Append, delete, or edit files, then use
    ///     set_intensities() to save the collection back. Empty if none exist.
    fn intensities(&self, py: Python<'_>) -> PyResult<Vec<IntensityFile>> {
        self.inner
            .intensities
            .iter()
            .cloned()
            .map(|file| IntensityFile::from_core(py, file))
            .collect()
    }

    /// Replace the main component with a copy. Scientific validation occurs on write.
    ///
    /// Parameters
    /// ----------
    /// file : MainFile
    ///     Replacement component. Later edits to it do not affect the archive.
    ///
    /// Returns
    /// -------
    /// None
    ///
    /// Raises
    /// ------
    /// TypeError
    ///     File or records have the wrong types; the archive remains unchanged.
    fn set_main(&mut self, py: Python<'_>, file: &MainFile) -> PyResult<()> {
        self.inner.main = file.to_core(py)?;
        Ok(())
    }

    /// Replace the assays component with a copy. Scientific validation occurs on write.
    ///
    /// Parameters
    /// ----------
    /// file : AssaysFile
    ///     Replacement component. Later edits to it do not affect the archive.
    ///
    /// Returns
    /// -------
    /// None
    ///
    /// Raises
    /// ------
    /// TypeError
    ///     File or records have the wrong types; the archive remains unchanged.
    fn set_assays(&mut self, py: Python<'_>, file: &AssaysFile) -> PyResult<()> {
        self.inner.assays = file.to_core(py)?;
        Ok(())
    }

    /// Replace all intensity components with copies, in the supplied order.
    ///
    /// Parameters
    /// ----------
    /// files : list[IntensityFile]
    ///     Replacement collection; [] removes all intensities. Later changes
    ///     to the list or its files do not affect the archive.
    ///
    /// Returns
    /// -------
    /// None
    ///
    /// Raises
    /// ------
    /// TypeError
    ///     A file or its records have the wrong types; no replacements are applied.
    ///
    /// Notes
    /// -----
    /// Scientific and filename validation occurs on write, including duplicate
    /// intensity filenames. Reading an archive back sorts intensities by well ID.
    fn set_intensities(
        &mut self,
        py: Python<'_>,
        files: Vec<PyRef<'_, IntensityFile>>,
    ) -> PyResult<()> {
        self.inner.intensities = files
            .iter()
            .map(|file| file.to_core(py))
            .collect::<PyResult<_>>()?;
        Ok(())
    }

    /// Return a detached intensity snapshot; negative indices are not supported.
    ///
    /// Parameters
    /// ----------
    /// index : int
    ///     Zero-based index, less than intensity_count.
    ///
    /// Returns
    /// -------
    /// IntensityFile
    ///     Independent editable copy. For edits to save back, prefer retrieving
    ///     intensities() and replacing the collection with set_intensities().
    ///
    /// Raises
    /// ------
    /// IndexError
    ///     Index is negative or outside the collection.
    /// OverflowError
    ///     Integer is outside the platform's signed pointer-sized range.
    /// TypeError
    ///     Index is not an integer.
    fn intensity(&self, py: Python<'_>, index: isize) -> PyResult<IntensityFile> {
        let file = usize::try_from(index)
            .ok()
            .and_then(|i| self.inner.intensities.get(i))
            .ok_or_else(|| PyIndexError::new_err("intensity index out of range"))?;
        IntensityFile::from_core(py, file.clone())
    }
}

// Free-threaded interpreter support has not been evaluated.
/// Read, edit, and write DDES main, assays, intensity files, and ZIP archives.
///
/// Files expose editable metadata and records. Archives copy components on
/// construction, getters, and setters; call a setter to save snapshot edits.
/// Operations are synchronous and in-memory. Scientific values and generated
/// filenames are validated when writing; reported results are not recalculated.
#[pymodule(gil_used = true)]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_main, m)?)?;
    m.add_function(wrap_pyfunction!(write_main, m)?)?;
    m.add_function(wrap_pyfunction!(read_intensity, m)?)?;
    m.add_function(wrap_pyfunction!(write_intensity, m)?)?;
    m.add_function(wrap_pyfunction!(read_assays, m)?)?;
    m.add_function(wrap_pyfunction!(write_assays, m)?)?;
    m.add_class::<DdesArchive>()?;
    m.add_class::<AssaysFile>()?;
    m.add_class::<AssaysMetadata>()?;
    m.add_class::<AssayRecord>()?;
    m.add_class::<MainFile>()?;
    m.add_class::<MainMetadata>()?;
    m.add_class::<MainRecord>()?;
    m.add_class::<IntensityFile>()?;
    m.add_class::<IntensityMetadata>()?;
    m.add_class::<IntensityRecord>()?;
    Ok(())
}

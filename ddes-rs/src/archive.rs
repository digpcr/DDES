//! In-memory digital PCR datasets and directory/ZIP archive I/O.

use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek, Write},
    path::Path,
};

use camino::Utf8Path;
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::{AssaysFile, Error, IntensityFile, MainFile, Result};

/// An in-memory processed digital PCR dataset.
///
/// Contains reaction summaries, assay definitions, and per-well endpoint
/// fluorescence. Reading and writing preserve reported results without
/// recalculating them. See the [crate example](crate) for a read/edit/write workflow.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    /// Reaction summaries and instrument metadata.
    pub main: MainFile,
    /// Assay definitions and probe concentrations.
    pub assays: AssaysFile,
    /// Per-well measurements, sorted by well ID when read; empty if absent.
    /// At most one intensity component per well is accepted on input.
    pub intensities: Vec<IntensityFile>,
}

impl Dataset {
    /// Reads a flat directory of TSVs or a ZIP archive, independent of extension.
    ///
    /// Loads the full dataset into memory. Paths accept native OS encoding.
    /// See the [crate example](crate) for usage.
    ///
    /// # Errors
    ///
    /// Returns an error for I/O or ZIP failures, malformed or unsupported
    /// components, unsafe filenames, duplicates, or missing main/assays files.
    pub fn read_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let label = path.to_string_lossy();
        if !path.is_dir() {
            return Self::read_zip(File::open(path).map_err(|e| Error::io(&label, e))?);
        }
        let mut files = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|e| Error::io(&label, e))? {
            let entry = entry.map_err(|e| Error::io(&label, e))?;
            if !entry
                .file_type()
                .map_err(|e| Error::io(&label, e))?
                .is_file()
            {
                return Err(Error::unsupported(
                    &label,
                    "only regular, root-level TSV files are supported",
                ));
            }
            files.push(entry.path());
        }
        files.sort();
        let mut builder = Builder::default();
        for file in files {
            let label = file.to_string_lossy();
            let name = file
                .file_name()
                .and_then(|name| Utf8Path::from_path(Path::new(name)))
                .ok_or_else(|| {
                    Error::unsupported(&label, "DDES component filename must be UTF-8")
                })?;
            let name = name.as_str();
            let bytes = std::fs::read(&file).map_err(|e| Error::io(&label, e))?;
            builder.insert(name, &bytes)?;
        }
        builder.finish()
    }

    /// Reads a ZIP archive into memory without extracting files to disk.
    ///
    /// Loads the full dataset into memory. The reader must support seeking.
    /// See the [crate example](crate) for reading ZIP bytes with a cursor.
    ///
    /// # Errors
    ///
    /// Returns an error for I/O or ZIP failures, unsafe filenames, duplicate or
    /// unsupported components, invalid TSV data, or missing main/assays files.
    pub fn read_zip(reader: impl Read + Seek) -> Result<Self> {
        let mut archive = ZipArchive::new(reader)?;
        // ZipArchive indexes names in a map; this also detects duplicates hidden by it.
        archive.validate_header_consistency()?;
        let mut builder = Builder::default();
        for index in 0..archive.len() {
            let mut member = archive.by_index(index)?;
            let name = member.name()?.into_owned();
            safe_name(&name)?;
            if member.is_dir() || member.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
                return Err(Error::unsupported(
                    &name,
                    "directories and symlinks are not supported",
                ));
            }
            let mut bytes = Vec::new();
            member
                .read_to_end(&mut bytes)
                .map_err(|e| Error::io(&name, e))?;
            builder.insert(&name, &bytes)?;
        }
        builder.finish()
    }

    /// Creates a ZIP archive at a new path without overwriting an existing file.
    ///
    /// Validates the dataset and filenames before creating the destination.
    /// For byte output instead, see [`Self::write_zip`] and the [crate example](crate).
    ///
    /// # Errors
    ///
    /// Returns an error for invalid data, unsafe or duplicate output filenames,
    /// an existing destination, or I/O/ZIP failures. An I/O failure may leave a
    /// partial file; remove it before retrying.
    pub fn write_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let files = self.files()?;
        let path = path.as_ref();
        let output = File::create_new(path).map_err(|e| Error::io(&path.to_string_lossy(), e))?;
        write_files(output, files)?;
        Ok(())
    }

    /// Writes a ZIP archive and returns the writer after finishing it.
    ///
    /// The caller controls the writer's position and whether an existing
    /// destination may be overwritten. See the [crate example](crate) for byte output.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid data, unsafe or duplicate output filenames,
    /// or I/O/ZIP failures. Invalid data is rejected before writing; an I/O failure
    /// may leave partial output.
    pub fn write_zip<W: Write + Seek>(&self, writer: W) -> Result<W> {
        write_files(writer, self.files()?)
    }

    /// Serialize components and check generated names before touching the destination.
    fn files(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let main = self.main.to_tsv("main")?;
        let assays = self.assays.to_tsv("assays")?;
        let prefix = format!(
            "DDES_{}_{}_{}",
            self.main.metadata.experiment_id,
            self.main.metadata.run_id,
            self.main.metadata.datetime
        );
        let mut files = vec![
            (format!("{prefix}_main.tsv"), main),
            (format!("{prefix}_assays.tsv"), assays),
        ];
        for intensity in &self.intensities {
            let bytes = intensity.to_tsv("intensity")?;
            files.push((
                format!("{prefix}_intensity_{}.tsv", intensity.metadata.well_id),
                bytes,
            ));
        }
        let mut names = BTreeSet::new();
        for (name, _) in &files {
            safe_name(name)?;
            if !names.insert(name.to_ascii_lowercase()) {
                return Err(Error::unsupported(name, "duplicate output filename"));
            }
        }
        Ok(files)
    }
}

/// Write prepared components with deflate compression and finalize the ZIP directory.
fn write_files<W: Write + Seek>(writer: W, files: Vec<(String, Vec<u8>)>) -> Result<W> {
    let mut archive = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, bytes) in files {
        archive.start_file(&name, options)?;
        archive.write_all(&bytes).map_err(|e| Error::io(&name, e))?;
    }
    Ok(archive.finish()?)
}

/// Require one portable TSV basename, applying Windows constraints on every OS.
fn safe_name(name: &str) -> Result<()> {
    // Apply Windows constraints on every OS, and never allow archive paths.
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix)
                .is_some_and(|s| matches!(s, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
        });
    if name.is_empty()
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:<>\"|?*".contains(c))
        || reserved
        || !name.ends_with(".tsv")
    {
        return Err(Error::unsupported(
            name,
            "expected a portable, root-level .tsv filename",
        ));
    }
    Ok(())
}

/// Assemble components while rejecting duplicate filenames, file roles, and wells.
#[derive(Default)]
struct Builder {
    main: Option<MainFile>,
    assays: Option<AssaysFile>,
    intensities: Vec<IntensityFile>,
    names: BTreeSet<String>,
    wells: BTreeSet<String>,
}

impl Builder {
    /// Dispatch by the component's DDES_type header, not its filename spelling.
    fn insert(&mut self, name: &str, bytes: &[u8]) -> Result<()> {
        safe_name(name)?;
        if !self.names.insert(name.to_ascii_lowercase()) {
            return Err(Error::unsupported(name, "duplicate filename"));
        }
        // Metadata is authoritative: identifiers can themselves contain role names.
        let kind = std::str::from_utf8(bytes)
            .map_err(|e| Error::format(name, 0, "encoding", e.to_string()))?
            .trim_start_matches('\u{feff}')
            .lines()
            .take_while(|line| line.starts_with('#') || line.is_empty())
            .find_map(|line| line.strip_prefix("#DDES_type\t"));
        if kind == Some("main") {
            if self.main.is_some() {
                return Err(Error::unsupported(name, "duplicate main component"));
            }
            self.main = Some(MainFile::from_tsv(name, bytes)?);
        } else if matches!(kind, Some("assays" | "assay")) {
            if self.assays.is_some() {
                return Err(Error::unsupported(name, "duplicate assays component"));
            }
            self.assays = Some(AssaysFile::from_tsv(name, bytes)?);
        } else if kind == Some("intensity") {
            let intensity = IntensityFile::from_tsv(name, bytes)?;
            if !self.wells.insert(intensity.metadata.well_id.clone()) {
                return Err(Error::unsupported(
                    name,
                    "duplicate intensity component for well",
                ));
            }
            self.intensities.push(intensity);
        } else {
            return Err(Error::unsupported(
                name,
                "missing or unsupported #DDES_type; refusing to discard component",
            ));
        }
        Ok(())
    }

    /// Require main and assays components, and normalize intensity component order.
    fn finish(mut self) -> Result<Dataset> {
        self.intensities
            .sort_by(|a, b| a.metadata.well_id.cmp(&b.metadata.well_id));
        Ok(Dataset {
            main: self
                .main
                .ok_or_else(|| Error::unsupported("dataset", "missing main component"))?,
            assays: self
                .assays
                .ok_or_else(|| Error::unsupported("dataset", "missing assays component"))?,
            intensities: self.intensities,
        })
    }
}

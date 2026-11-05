//! Read, edit, and write DDES TSV components and ZIP archives from JavaScript.

use std::io::Cursor;

use serde::{Serialize, de::DeserializeOwned};
use wasm_bindgen::prelude::*;

mod assays;
mod intensity;
mod main_file;

use assays::AssaysFile;
use intensity::IntensityFile;
use main_file::MainFile;

fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    value
        .serialize(
            &serde_wasm_bindgen::Serializer::new()
                .serialize_missing_as_null(true)
                .serialize_large_number_types_as_bigints(true),
        )
        .map_err(|e| JsError::new(&e.to_string()))
}

fn from_js<T: DeserializeOwned + Serialize>(value: JsValue) -> Result<T, JsError> {
    let decoded: T =
        serde_wasm_bindgen::from_value(value.clone()).map_err(|e| JsError::new(&e.to_string()))?;
    // serde-wasm-bindgen only visits known struct fields, even with
    // deny_unknown_fields. Compare against the decoded shape before accepting
    // edits, so extra data cannot silently disappear. Keep the schema in the types.
    check_keys(&value, &to_js(&decoded)?)?;
    Ok(decoded)
}

fn check_keys(input: &JsValue, decoded: &JsValue) -> Result<(), JsError> {
    if input.is_object() && !input.is_null() {
        for key in js_sys::Object::keys(input.unchecked_ref::<js_sys::Object>()).iter() {
            if !js_sys::Object::has_own(decoded.unchecked_ref::<js_sys::Object>(), &key) {
                return Err(JsError::new(&format!(
                    "unknown field {}",
                    key.as_string().unwrap()
                )));
            }
            check_keys(
                &js_sys::Reflect::get(input, &key).map_err(serde_wasm_bindgen::Error::from)?,
                &js_sys::Reflect::get(decoded, &key).map_err(serde_wasm_bindgen::Error::from)?,
            )?;
        }
    }
    Ok(())
}

fn core_error(error: ddes::Error) -> JsError {
    JsError::new(&error.to_string())
}

/// Parse complete UTF-8 main TSV bytes, including metadata and column headers.
/// Returns an editable object with bigint counts and null missing values.
///
/// @throws {Error} Invalid encoding, headers, columns, or numeric values.
/// Diagnostics use "main.tsv"; no file is opened.
#[wasm_bindgen(js_name = readMain, unchecked_return_type = "MainFile")]
pub fn read_main(bytes: &[u8]) -> Result<JsValue, JsError> {
    let file = ddes::MainFile::from_tsv("main.tsv", bytes).map_err(core_error)?;
    to_js(&MainFile::from(file))
}

/// Serialize a main file to normalized UTF-8 TSV bytes without modifying it.
/// Null becomes NA; numeric formatting may normalize. No file is opened.
///
/// @throws {Error} Wrong object shape, unknown fields, invalid counts, nonfinite
/// measurements, negative concentration, invalid metadata, or NUL characters.
#[wasm_bindgen(js_name = writeMain)]
pub fn write_main(
    #[wasm_bindgen(unchecked_param_type = "MainFile")] file: JsValue,
) -> Result<Vec<u8>, JsError> {
    let file: MainFile = from_js(file)?;
    ddes::MainFile::from(file)
        .to_tsv("main.tsv")
        .map_err(core_error)
}

/// Parse complete UTF-8 assays TSV bytes into editable metadata and records.
/// Probe combinations and concentration lists remain strings, including NA entries.
///
/// @throws {Error} Invalid encoding, headers, columns, or probe concentrations.
/// Diagnostics use "assays.tsv"; no file is opened.
#[wasm_bindgen(js_name = readAssays, unchecked_return_type = "AssaysFile")]
pub fn read_assays(bytes: &[u8]) -> Result<JsValue, JsError> {
    let file = ddes::AssaysFile::from_tsv("assays.tsv", bytes).map_err(core_error)?;
    to_js(&AssaysFile::from(file))
}

/// Serialize an assays file to normalized UTF-8 TSV bytes without modifying it.
/// Probe-list spelling is preserved. No file is opened.
///
/// @throws {Error} Wrong object shape, unknown fields, invalid metadata, NUL
/// characters, or concentrations other than finite nonnegative numbers and NA.
#[wasm_bindgen(js_name = writeAssays)]
pub fn write_assays(
    #[wasm_bindgen(unchecked_param_type = "AssaysFile")] file: JsValue,
) -> Result<Vec<u8>, JsError> {
    let file: AssaysFile = from_js(file)?;
    ddes::AssaysFile::from(file)
        .to_tsv("assays.tsv")
        .map_err(core_error)
}

/// Parse complete UTF-8 endpoint-intensity TSV bytes into an editable object.
/// Endpoint intensities are numbers; zero and negative values are accepted.
///
/// @throws {Error} Invalid encoding, headers, columns, or missing/nonfinite intensity.
/// Diagnostics use "intensity.tsv"; no file is opened.
#[wasm_bindgen(js_name = readIntensity, unchecked_return_type = "IntensityFile")]
pub fn read_intensity(bytes: &[u8]) -> Result<JsValue, JsError> {
    let file = ddes::IntensityFile::from_tsv("intensity.tsv", bytes).map_err(core_error)?;
    to_js(&IntensityFile::from(file))
}

/// Serialize an intensity file to normalized UTF-8 TSV bytes without modifying it.
/// Numeric formatting may normalize. No file is opened.
///
/// @throws {Error} Wrong object shape, unknown fields, invalid metadata,
/// NUL characters, or nonfinite intensity.
#[wasm_bindgen(js_name = writeIntensity)]
pub fn write_intensity(
    #[wasm_bindgen(unchecked_param_type = "IntensityFile")] file: JsValue,
) -> Result<Vec<u8>, JsError> {
    let file: IntensityFile = from_js(file)?;
    ddes::IntensityFile::from(file)
        .to_tsv("intensity.tsv")
        .map_err(core_error)
}

/// An in-memory DDES archive. Getters return detached, editable plain objects.
/// Use setters to save edits. Reported scientific results are never recalculated.
///
/// Call free() in finally when finished, and never use the handle afterward.
/// Snapshots and returned bytes remain usable after free(). Do not pass handles
/// through postMessage; pass snapshots or serialized bytes instead.
#[wasm_bindgen]
pub struct DdesArchive {
    inner: ddes::Dataset,
}

#[wasm_bindgen]
impl DdesArchive {
    /// Copy components into an independent archive; [] means no intensity files.
    /// Preserves supplied intensity order. Later input edits do not change the archive.
    /// Scientific values and filenames are validated on write, not construction.
    ///
    /// @throws {Error} Invalid object shapes, unknown fields, or incorrect field types.
    #[wasm_bindgen(constructor)]
    pub fn new(
        #[wasm_bindgen(unchecked_param_type = "MainFile")] main: JsValue,
        #[wasm_bindgen(unchecked_param_type = "AssaysFile")] assays: JsValue,
        #[wasm_bindgen(unchecked_param_type = "IntensityFile[]")] intensities: JsValue,
    ) -> Result<DdesArchive, JsError> {
        let main: MainFile = from_js(main)?;
        let assays: AssaysFile = from_js(assays)?;
        let intensities: Vec<IntensityFile> = from_js(intensities)?;
        Ok(Self {
            inner: ddes::Dataset {
                main: main.into(),
                assays: assays.into(),
                intensities: intensities.into_iter().map(Into::into).collect(),
            },
        })
    }

    /// Load complete ZIP/.ddes bytes into memory, sorting intensity files by well ID.
    /// There are no size or file-count limits; account for uncompressed input size.
    ///
    /// @throws {Error} Malformed ZIP/TSV data, unsafe names, duplicate or unsupported
    /// components, or missing main/assays files. No files are extracted to disk.
    #[wasm_bindgen(js_name = fromBytes)]
    pub fn from_bytes(bytes: &[u8]) -> Result<DdesArchive, JsError> {
        ddes::Dataset::read_zip(Cursor::new(bytes))
            .map(|inner| Self { inner })
            .map_err(core_error)
    }

    /// Return normalized ZIP bytes in a new JS-owned Uint8Array.
    /// Does not modify the archive. Formatting and ZIP metadata may normalize.
    ///
    /// @throws {Error} Invalid scientific values or metadata, unsafe/duplicate
    /// output filenames, NUL characters, or ZIP failures.
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&self) -> Result<Vec<u8>, JsError> {
        self.inner
            .write_zip(Cursor::new(Vec::new()))
            .map(Cursor::into_inner)
            .map_err(core_error)
    }

    /// Number of per-well intensity files (read-only).
    #[wasm_bindgen(getter, js_name = intensityCount)]
    pub fn intensity_count(&self) -> u32 {
        self.inner.intensities.len() as u32
    }

    /// Return a detached main-file snapshot. Editing it does not change the archive.
    #[wasm_bindgen(unchecked_return_type = "MainFile")]
    pub fn main(&self) -> Result<JsValue, JsError> {
        to_js(&MainFile::from(self.inner.main.clone()))
    }

    /// Return a detached assays-file snapshot. Editing it does not change the archive.
    #[wasm_bindgen(unchecked_return_type = "AssaysFile")]
    pub fn assays(&self) -> Result<JsValue, JsError> {
        to_js(&AssaysFile::from(self.inner.assays.clone()))
    }

    /// Return one detached intensity file by zero-based index.
    ///
    /// @throws {Error} Index is not a finite nonnegative integer below intensityCount.
    #[wasm_bindgen(unchecked_return_type = "IntensityFile")]
    pub fn intensity(
        &self,
        #[wasm_bindgen(unchecked_param_type = "number")] index: JsValue,
    ) -> Result<JsValue, JsError> {
        let index = index
            .as_f64()
            .filter(|n| n.is_finite() && n.fract() == 0.0 && *n >= 0.0)
            .ok_or_else(|| JsError::new("intensity index must be a nonnegative integer"))?;
        let file = self
            .inner
            .intensities
            .get(index as usize)
            .ok_or_else(|| JsError::new("intensity index out of range"))?;
        to_js(&IntensityFile::from(file.clone()))
    }

    /// Return all intensity files as detached snapshots in archive order.
    #[wasm_bindgen(unchecked_return_type = "IntensityFile[]")]
    pub fn intensities(&self) -> Result<JsValue, JsError> {
        let files: Vec<_> = self
            .inner
            .intensities
            .iter()
            .cloned()
            .map(IntensityFile::from)
            .collect();
        to_js(&files)
    }

    /// Replace the main file with a copy; later input edits do not affect the archive.
    /// Scientific values and filenames are validated on write.
    ///
    /// @throws {Error} Invalid shape or types. On failure the archive is unchanged.
    #[wasm_bindgen(js_name = setMain)]
    pub fn set_main(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "MainFile")] file: JsValue,
    ) -> Result<(), JsError> {
        self.inner.main = from_js::<MainFile>(file)?.into();
        Ok(())
    }

    /// Replace the assays file with a copy; later input edits do not affect the archive.
    /// Scientific values and filenames are validated on write.
    ///
    /// @throws {Error} Invalid shape or types. On failure the archive is unchanged.
    #[wasm_bindgen(js_name = setAssays)]
    pub fn set_assays(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "AssaysFile")] file: JsValue,
    ) -> Result<(), JsError> {
        self.inner.assays = from_js::<AssaysFile>(file)?.into();
        Ok(())
    }

    /// Replace all intensity files with copies, preserving their order. [] removes all.
    /// Scientific values and filenames are validated on write.
    ///
    /// @throws {Error} Invalid shape or types in any file. Failure leaves all files unchanged.
    #[wasm_bindgen(js_name = setIntensities)]
    pub fn set_intensities(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "IntensityFile[]")] files: JsValue,
    ) -> Result<(), JsError> {
        let files: Vec<IntensityFile> = from_js(files)?;
        self.inner.intensities = files.into_iter().map(Into::into).collect();
        Ok(())
    }
}

#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod archive;
pub mod assays;
mod error;
pub mod intensity;
pub mod main_file;
mod numeric_serde;
mod tsv;

pub use archive::Dataset;
pub use assays::{AssayRecord, AssaysFile, AssaysMetadata};
pub use error::{Error, Result};
pub use intensity::{IntensityFile, IntensityMetadata, IntensityRecord};
pub use main_file::{MainFile, MainMetadata, MainRecord};

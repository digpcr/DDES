//! Errors from DDES parsing, serialization, and archive I/O.

use thiserror::Error;

/// Result returned by DDES parsing, serialization, and archive operations.
pub type Result<T> = std::result::Result<T, Error>;

/// An error from reading or writing DDES data.
///
/// Variants retain a component name and location when available.
#[derive(Debug, Error)]
pub enum Error {
    /// A filesystem or stream operation failed.
    #[error("{file}: {source}")]
    Io {
        /// Path or component label associated with the operation.
        file: String,
        /// Underlying operating-system or stream error.
        #[source]
        source: std::io::Error,
    },
    /// ZIP decoding, encoding, or consistency checking failed.
    #[error("ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// Component contents do not match the supported DDES structure.
    #[error("{file}, line {line}, field {field}: {message}")]
    Format {
        /// Path or component label supplied to the parser or writer.
        file: String,
        /// One-based input line when available; zero means no precise location.
        line: u64,
        /// Field name, or a section label such as `header` or `table`.
        field: String,
        /// Explanation of the format violation.
        message: String,
    },
    /// The input or output uses an unsupported structure or feature.
    #[error("{file}: {message}")]
    Unsupported {
        /// Path, component name, or dataset label.
        file: String,
        /// Explanation of the unsupported feature or structure.
        message: String,
    },
}

impl Error {
    /// Attach a diagnostic label to an I/O error.
    pub(crate) fn io(file: &str, source: std::io::Error) -> Self {
        Self::Io {
            file: file.into(),
            source,
        }
    }

    /// Attach a component location to a format error.
    pub(crate) fn format(file: &str, line: u64, field: &str, message: impl Into<String>) -> Self {
        Self::Format {
            file: file.into(),
            line,
            field: field.into(),
            message: message.into(),
        }
    }

    /// Describe unsupported data without silently discarding it.
    pub(crate) fn unsupported(file: &str, message: impl Into<String>) -> Self {
        Self::Unsupported {
            file: file.into(),
            message: message.into(),
        }
    }
}

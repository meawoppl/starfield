//! Error types for the jplephem module
//!
//! This module defines error types for the JPL ephemeris functionality.

use thiserror::Error;
use std::path::PathBuf;

/// Main error type for jplephem functionality
#[derive(Error, Debug)]
pub enum JplephemError {
    /// Error when a file I/O operation fails
    #[error("File I/O error on {path:?}: {source}")]
    FileError {
        /// The path of the file that caused the error
        path: PathBuf,
        /// The underlying I/O error
        source: std::io::Error,
    },

    /// Error when a date is outside the range covered by the ephemeris
    #[error("Date {jd} is outside ephemeris range ({start_jd}..{end_jd})")]
    OutOfRangeError {
        /// The Julian date that was requested
        jd: f64,
        /// The start of the ephemeris range
        start_jd: f64,
        /// The end of the ephemeris range
        end_jd: f64,
        /// Optional array of boolean values for batch requests
        out_of_range_times: Option<Vec<bool>>,
    },

    /// Error when the file format is invalid or unsupported
    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    /// Error when a memory mapping operation fails
    #[error("Memory mapping error: {0}")]
    MemoryMapError(String),

    /// Error when the requested body is not found in the ephemeris
    #[error("Body not found: center={center}, target={target}")]
    BodyNotFound {
        /// The center body ID
        center: i32,
        /// The target body ID
        target: i32,
    },

    /// Error when the data type is not supported
    #[error("Unsupported data type: {0}")]
    UnsupportedDataType(i32),

    /// Other, miscellaneous errors
    #[error("{0}")]
    Other(String),
}

/// Extension of the Result type for jplephem operations
pub type Result<T> = std::result::Result<T, JplephemError>;

/// Helper function to convert a std::io::Error to JplephemError
pub fn io_err(path: impl Into<PathBuf>, err: std::io::Error) -> JplephemError {
    JplephemError::FileError {
        path: path.into(),
        source: err,
    }
}
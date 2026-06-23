//! Unified error type for 3MF package I/O operations.

use thiserror::Error;
use zip::result::ZipError;

use crate::model::PathResourceError;

/// An error that can occur while writing a 3MF file
/// An error that can occur during 3MF package I/O operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error while reading or writing a 3MF file.
    #[error("I/O error while importing/exporting to 3MF file")]
    Io(#[from] std::io::Error),

    /// Error in the underlying ZIP archive.
    #[error("Error writing ZIP file (3MF files are ZIP files)")]
    Zip(#[from] ZipError),

    /// Error reading a 3MF file.
    #[error("Error reading 3mf file: {0}")]
    ReadError(String),

    /// Error writing a 3MF file.
    #[error("Error writing 3mf file: {0}")]
    WriteError(String),

    /// XML deserialization error from instant-xml.
    #[cfg(any(feature = "write", feature = "memory-optimized-read"))]
    #[error("(De)Serialization error from Instant-Xml")]
    InstantXmlError(#[from] instant_xml::Error),

    /// Thumbnail processing error.
    #[error("Thumbnail error: {0}")]
    ThumbnailError(String),

    /// Requested resource was not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Path validation error.
    #[error("Path Resource error")]
    PathResourceError(#[from] PathResourceError),

    /// Deserialization error from serde-roxmltree.
    #[cfg(feature = "speed-optimized-read")]
    #[error("Deserialization error from serde-roxmltree")]
    SerdeRoxmltreeError(#[from] serde_roxmltree::Error),
}

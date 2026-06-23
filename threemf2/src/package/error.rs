//! Unified error type for 3MF package I/O operations.

use thiserror::Error;
use zip::result::ZipError;

use crate::model::PathResourceError;

/// An error that can occur while writing a 3MF file
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error while writing 3MF file
    #[error("I/O error while importing/exporting to 3MF file")]
    Io(#[from] std::io::Error),

    /// Error writing ZIP file (3MF files are ZIP files)
    #[error("Error writing ZIP file (3MF files are ZIP files)")]
    Zip(#[from] ZipError),

    #[error("Error reading 3mf file: {0}")]
    ReadError(String),

    #[error("Error writing 3mf file: {0}")]
    WriteError(String),

    #[cfg(any(feature = "write", feature = "memory-optimized-read"))]
    #[error("(De)Serialization error from Instant-Xml")]
    InstantXmlError(#[from] instant_xml::Error),

    #[error("Thumbnail error: {0}")]
    ThumbnailError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Path Respurce error")]
    PathResourceError(#[from] PathResourceError),

    #[cfg(feature = "speed-optimized-read")]
    #[error("Deserialization error from serde-roxmltree")]
    SerdeRoxmltreeError(#[from] serde_roxmltree::Error),
}

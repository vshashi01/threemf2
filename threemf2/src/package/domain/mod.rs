//! Internal types for the Open Packaging Conventions (OPC) layer.
//!
//! These modules define the low-level ZIP archive structure: content types, relationships,
//! thumbnails, and validation. Most users interact with these only through the higher-level
//! [`ThreemfPackage`](crate::package::ThreemfPackage) and [`ThreemfPackageBuilder`](crate::package::ThreemfPackageBuilder) APIs.

pub mod content_types;
pub mod relationship;

pub mod thumbnail_handle;

pub mod validator;
mod validator_rules;

/// Represents a generic XML namespace declaration with its prefix and URI
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XmlNamespace {
    /// The namespace prefix (None for default namespace)
    pub prefix: Option<String>,
    /// The namespace URI
    pub uri: String,
}

#[cfg(any(
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub(crate) mod zip_utils;

pub(crate) mod utils;

pub mod content_types;
pub mod relationship;

mod error;
pub use error::Error;

mod utils;

pub mod thumbnail_handle;

#[cfg(feature = "io-write")]
pub mod builder;
#[cfg(feature = "io-write")]
pub use builder::ThreemfPackageBuilder;

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
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod zip_utils;

#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod threemf_package;
#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub use threemf_package::ThreemfPackage;

#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub mod query;

#[cfg(all(
    feature = "io-lazy-read",
    any(
        feature = "io-memory-optimized-read",
        feature = "io-speed-optimized-read"
    )
))]
mod threemf_package_lazy_reader;
#[cfg(all(
    feature = "io-lazy-read",
    any(
        feature = "io-memory-optimized-read",
        feature = "io-speed-optimized-read"
    )
))]
pub use threemf_package_lazy_reader::{CachePolicy, ThreemfPackageLazyReader};

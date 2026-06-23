//! ZIP-based I/O for reading and writing 3MF packages.
//!
//! This module provides the main types for working with 3MF files as physical ZIP archives.
//!
//! # Key Types
//!
//! - `ThreemfPackage` — Eager loading: reads all models, thumbnails, and relationships upfront.
//! - `ThreemfPackageLazyReader` — Lazy loading: reads metadata first, defers model data until accessed.
//! - `ThreemfPackageBuilder` — Assembles models into a valid 3MF package with proper ZIP structure.
//!
//! # Feature Gating
//!
//! All types in this module are feature-gated. The module is only compiled when at least one of these
//! features is enabled: `package-write`, `package-memory-optimized-read`, `io-speed-optimized-read`, or `package-lazy-read`.

pub mod domain;

mod error;
pub use error::Error;

#[cfg(feature = "package-write")]
pub mod builder;
#[cfg(feature = "package-write")]
pub use builder::ThreemfPackageBuilder;

#[cfg(any(
    feature = "package-write",
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod threemf_package;
#[cfg(any(
    feature = "package-write",
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub use threemf_package::ThreemfPackage;

#[cfg(any(
    feature = "package-write",
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
pub mod query;

#[cfg(all(
    feature = "package-lazy-read",
    any(
        feature = "package-memory-optimized-read",
        feature = "io-speed-optimized-read"
    )
))]
mod threemf_package_lazy_reader;
#[cfg(all(
    feature = "package-lazy-read",
    any(
        feature = "package-memory-optimized-read",
        feature = "io-speed-optimized-read"
    )
))]
pub use threemf_package_lazy_reader::{CachePolicy, ThreemfPackageLazyReader};

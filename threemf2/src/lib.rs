//! # 3MF (3D Manufacturing Format) support for Rust
//!
//! This library provides support for [3MF] files to programs written in the
//! Rust programming language. 3MF is a file format commonly used for 3D
//! printing. It is typically exported from a CAD program, and imported to a
//! slicer.
//!
//!
//! [3MF]: https://en.wikipedia.org/wiki/3D_Manufacturing_Format
//! This library was originally taken from the [threemf] crate, however my goals deviated from the goals
//! of the original package and its maintainers as such I decided to take this into my own packages.
//! Thanks for the great work of the original maintainers.
//!
//! ## Further Reading
//!
//! See [3MF specification] and [Open Packaging Conventions].
//!
//! [threemf]:https://crates.io/crates/threemf
//! [3MF specification]: https://3mf.io/specification/
//! [Open Packaging Conventions]: https://standards.iso.org/ittf/PubliclyAvailableStandards/c061796_ISO_IEC_29500-2_2012.zip

#![forbid(unsafe_code)]

/// [`core`] module maps and defines the elements in the 3MF Specifications to Rust Types.
/// The Serialization and Deserialization implementations are also provided in this module.
/// As a crate user you can use these types directly to serialize and deserialize 3MF Model element.
pub mod core;

/// This module defines all the namespaces used by the supported 3MF Extensions by this library.
/// The default prefixes used when writing a 3MF Model is also defined here.
pub mod threemf_namespaces;

/// [`io`] module implements the actual Reader and Writers for a 3MF Package. If you want one stop centre
/// to read and write 3MF file, then this is the module you require to work with them.
/// This module can be disabled if you only want the [`core`] module.
#[cfg(any(
    feature = "io-write",
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "io-lazy-read"
))]
pub mod io;

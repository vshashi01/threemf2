////////////////////////////////////////////////////////////////////////////////////
///  Namespaces & Prefixes related to the Core specification and its extensions
pub const CORE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";

pub const CORE_TRIANGLESET_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/trianglesets/2021/07";
pub const CORE_TRIANGLESET_PREFIX: &str = "t";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Slice extension
pub const SLICE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/slice/2015/07";
pub const SLICE_PREFIX: &str = "s";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Boolean operations extension
pub const BOOLEAN_NS: &str = "http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07";
pub const BOOLEAN_PREFIX: &str = "bo";

//////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Production extension
pub const PROD_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/production/2015/06";
pub const PROD_PREFIX: &str = "p";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Beam Lattice extension
pub const BEAM_LATTICE_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02";
pub const BEAM_LATTICE_PREFIX: &str = "b";

pub const BEAM_LATTICE_BALLS_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07";
pub const BEAM_LATTICE_BALLS_PREFIX: &str = "b2";

/// Enum representing the different 3MF specifications supported by this library
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreemfNamespace {
    /// Core 3MF
    Core,

    /// Slice extension specification
    Slice,

    /// Boolean operations extension specification
    Boolean,

    /// Production extension specification
    Prod,

    /// Beam Lattice extension
    BeamLattice,

    /// Triangle Set extension (Part of the Core Spec)
    CoreTriangleSet,
}

impl ThreemfNamespace {
    /// Returns the namespace Uri
    pub fn uri(&self) -> &'static str {
        match self {
            Self::Core => CORE_NS,
            Self::Slice => SLICE_NS,
            Self::Boolean => BOOLEAN_NS,
            Self::Prod => PROD_NS,
            Self::BeamLattice => BEAM_LATTICE_NS,
            Self::CoreTriangleSet => CORE_TRIANGLESET_NS,
        }
    }

    /// Returns the default prefix used by this library for this namespace
    /// Note: This is not reflective of prefix that maybe used by other
    /// producers.
    pub fn prefix(&self) -> Option<&'static str> {
        match self {
            Self::Core => None, // default namespace
            Self::Slice => Some(SLICE_PREFIX),
            Self::Boolean => Some(BOOLEAN_PREFIX),
            Self::Prod => Some(PROD_PREFIX),
            Self::BeamLattice => Some(BEAM_LATTICE_PREFIX),
            Self::CoreTriangleSet => Some(CORE_TRIANGLESET_PREFIX),
        }
    }

    /// Utility to return XML Namespace definition as String
    pub fn xmlns_declaration(&self) -> String {
        match self.prefix() {
            Some(prefix) => format!(r#" xmlns:{}="{}""#, prefix, self.uri()),
            None => format!(r#" xmlns="{}""#, self.uri()),
        }
    }
}

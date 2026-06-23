use crate::model::StrResource;

////////////////////////////////////////////////////////////////////////////////////
///  Namespaces & Prefixes related to the Core specification and its extensions
/// Core 3MF namespace URI.
pub const CORE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";

/// Triangle Set extension namespace URI.
pub const CORE_TRIANGLESET_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/trianglesets/2021/07";
/// Triangle Set extension XML prefix.
pub const CORE_TRIANGLESET_PREFIX: &str = "t";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Slice extension
/// Slice extension namespace URI.
pub const SLICE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/slice/2015/07";
/// Slice extension XML prefix.
pub const SLICE_PREFIX: &str = "s";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Boolean operations extension
/// Boolean operations extension namespace URI.
pub const BOOLEAN_NS: &str = "http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07";
/// Boolean operations extension XML prefix.
pub const BOOLEAN_PREFIX: &str = "bo";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Production extension
/// Production extension namespace URI.
pub const PROD_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/production/2015/06";
/// Production extension XML prefix.
pub const PROD_PREFIX: &str = "p";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Beam Lattice extension
/// Beam Lattice extension namespace URI.
pub const BEAM_LATTICE_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02";
/// Beam Lattice extension XML prefix.
pub const BEAM_LATTICE_PREFIX: &str = "b";

/// Beam Lattice balls extension namespace URI.
pub const BEAM_LATTICE_BALLS_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07";
/// Beam Lattice balls extension XML prefix.
pub const BEAM_LATTICE_BALLS_PREFIX: &str = "b2";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Material extension
/// Material extension namespace URI.
pub const MATERIAL_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/material/2015/02";
/// Material extension XML prefix.
pub const MATERIAL_PREFIX: &str = "m";

////////////////////////////////////////////////////////////////////////////////////
/// Namespaces & Prefixes related to the Displacement extension
/// Displacement extension namespace URI.
pub const DISPLACEMENT_NS: &str = "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10";
/// Displacement extension XML prefix.
pub const DISPLACEMENT_PREFIX: &str = "d";

/// Enum representing the different 3MF specifications supported by this library
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// Beam Lattice balls extension
    BeamLatticeBalls,

    /// Triangle Set extension (Part of the Core Spec)
    CoreTriangleSet,

    /// Material extension
    Material,

    /// Displacement extension
    Displacement,

    /// Unknown namespace
    Unknown {
        /// XML prefix for the unknown namespace.
        prefix: StrResource,
        /// Namespace URI.
        uri: StrResource,
    },
}

impl ThreemfNamespace {
    /// Returns the namespace Uri
    pub fn uri(&self) -> &str {
        match self {
            Self::Core => CORE_NS,
            Self::Slice => SLICE_NS,
            Self::Boolean => BOOLEAN_NS,
            Self::Prod => PROD_NS,
            Self::BeamLattice => BEAM_LATTICE_NS,
            Self::BeamLatticeBalls => BEAM_LATTICE_BALLS_NS,
            Self::CoreTriangleSet => CORE_TRIANGLESET_NS,
            Self::Material => MATERIAL_NS,
            Self::Displacement => DISPLACEMENT_NS,
            Self::Unknown { prefix: _, uri } => uri,
        }
    }

    /// Returns the default prefix used by this library for this namespace
    /// Note: This is not reflective of prefix that maybe used by other
    /// producers.
    pub fn prefix(&self) -> Option<&str> {
        match self {
            Self::Core => None, // default namespace
            Self::Slice => Some(SLICE_PREFIX),
            Self::Boolean => Some(BOOLEAN_PREFIX),
            Self::Prod => Some(PROD_PREFIX),
            Self::BeamLattice => Some(BEAM_LATTICE_PREFIX),
            Self::BeamLatticeBalls => Some(BEAM_LATTICE_BALLS_PREFIX),
            Self::CoreTriangleSet => Some(CORE_TRIANGLESET_PREFIX),
            Self::Material => Some(MATERIAL_PREFIX),
            Self::Displacement => Some(DISPLACEMENT_PREFIX),
            Self::Unknown { prefix, uri: _ } => Some(prefix),
        }
    }

    pub fn try_from_uri(uri: &str, assigned_prefix: Option<&str>) -> Option<Self> {
        match uri {
            CORE_NS => Some(Self::Core),
            CORE_TRIANGLESET_NS => Some(Self::CoreTriangleSet),
            PROD_NS => Some(Self::Prod),
            BEAM_LATTICE_NS => Some(Self::BeamLattice),
            BEAM_LATTICE_BALLS_NS => Some(Self::BeamLatticeBalls),
            SLICE_NS => Some(Self::Slice),
            BOOLEAN_NS => Some(Self::Boolean),
            MATERIAL_NS => Some(Self::Material),
            DISPLACEMENT_NS => Some(Self::Displacement),
            _ => assigned_prefix.map(|prefix| Self::Unknown {
                prefix: prefix.into(),
                uri: uri.into(),
            }),
        }
    }

    pub fn try_from_prefix(prefix: &str, specified_uri: Option<&str>) -> Option<Self> {
        match prefix {
            //CORE_NS => Some(Self::Core),
            CORE_TRIANGLESET_PREFIX => Some(Self::CoreTriangleSet),
            PROD_PREFIX => Some(Self::Prod),
            BEAM_LATTICE_PREFIX => Some(Self::BeamLattice),
            BEAM_LATTICE_BALLS_PREFIX => Some(Self::BeamLatticeBalls),
            SLICE_PREFIX => Some(Self::Slice),
            BOOLEAN_PREFIX => Some(Self::Boolean),
            MATERIAL_PREFIX => Some(Self::Material),
            DISPLACEMENT_PREFIX => Some(Self::Displacement),
            _ => match specified_uri {
                Some(uri) => Some(Self::Unknown {
                    prefix: prefix.into(),
                    uri: uri.into(),
                }),
                None => Some(Self::Unknown {
                    prefix: prefix.into(),
                    uri: "".into(),
                }),
            },
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

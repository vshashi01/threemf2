/// Defines the type representing the 3MF Model element and elements and attributes related to it.
pub mod model;

/// Defines the type representing the Metadata element and other elements and attributes related to it.
pub mod metadata;

/// Defines the type representing the Resources elements and other elements and attributes related to it.
pub mod resources;

/// Defines the type representing a  Object and other elements and atrributes related to it.
pub mod object;

/// Defines the type representing a Mesh object and other elements and attributes related to it.
pub mod mesh;

/// Defines the type representing a Components object and other elements and attributes related to it.
pub mod component;

/// Defines the Transform type for use in Model.
pub mod transform;

/// Defines the type representing Build and other elements and attributes related to it.
pub mod build;

/// Defines the type representing Triangle Sets and other elements and attributes related to it.
pub mod triangle_set;

/// Defines the type representing a Beam Lattice element and other elements and attributes related to it.
pub mod beamlattice;

/// Defines the type representing a Boolean Shape element and other elements and attributes related to it.
pub mod boolean;

/// Defines the type representing the Slice extension element and related elements.
pub mod slice;

/// Defines the type representing the Material extension elements and related elements.
pub mod material;

/// Defines the type representing the Displacement extension elements and related elements.
pub mod displacement;

pub mod types;

/// Owned foreign XML content for the `xs:any` and `xs:anyAttribute` extension points.
#[cfg(feature = "any")]
pub mod any;

/// Always-present containers for foreign XML content, populated with the `any` feature.
pub mod any_extensions;

#[cfg(feature = "read")]
pub(crate) mod xml_helpers;

#[cfg(any(feature = "read", feature = "write"))]
pub(crate) mod xml_struct;

#[cfg(any(feature = "read", feature = "write"))]
pub(crate) mod xml_item;

mod constants;

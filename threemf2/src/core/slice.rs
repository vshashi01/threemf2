//! 3MF Slice Extension types
//!
//! This module provides data structures for the 3MF Slice Extension, which enables
//! 2.5D geometry support by defining slice stack data alongside mesh geometry.
//!
//! The Slice Extension allows models to be stored as stacks of 2D slices alongside
//! the 3D meshes of the core specification. This is particularly useful for sliced
//! model data that represents layers of a 3D object.
//!
//! # Key Types
//!
//! - [`SliceStack`] - Container for slice data, referenced by objects
//! - [`Slice`] - Individual 2D layer with vertices and polygons
//! - [`SliceRef`] - Reference to external slice files in the 3MF package
//! - [`SliceVertex`] - 2D vertex with x, y coordinates
//! - [`Polygon`] - Closed or open contour defined by segments
//! - [`Segment`] - Line segment connecting vertices
//! - [`MeshResolution`] - Indicates mesh quality (fullres or lowres)
//!
//! # Usage
//!
//! Objects reference slice stacks via the `slicestackid` attribute. When slice data
//! is in a separate file, the `slicepath` attribute is used in combination with
//! `slicestackid`. The `meshresolution` attribute indicates whether the mesh is
//! intended to be printed (`fullres`) or is a lower resolution representation
//! (`lowres`).
//!
//! # Example XML Structure
//!
//! ```xml
//! <object id="2" s:slicestackid="1" s:meshresolution="lowres">
//!   <mesh>...</mesh>
//! </object>
//! <s:slicestack id="1" zbottom="0.0">
//!   <s:slice ztop="0.1">
//!     <s:vertices>...</s:vertices>
//!     <s:polygon startv="0">
//!       <s:segment v2="1"/>
//!     </s:polygon>
//!   </s:slice>
//! </s:slicestack>
//! ```

use crate::{
    core::{types::ResourceId, OptionalResourceId, OptionalResourceIndex, ResourceIndex},
    threemf_namespaces::SLICE_NS,
};

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

/// Indicates the intended resolution of mesh models when slice data is present.
///
/// When a 3MF package contains both mesh and slice data, this attribute helps
/// consumers understand whether the mesh is intended for fabrication or is a
/// lower-resolution representation.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, ns(SLICE_NS), rename_all = "lowercase")
)]
pub enum MeshResolution {
    /// The included mesh data is full resolution and could be used to re-generate
    /// the slices contained in the 3MF package.
    #[default]
    FullRes,

    /// The included mesh is not sufficiently accurate to re-generate the slices
    /// contained in the 3MF package. Packages containing lowres objects MUST
    /// list the slice extension in requiredextensions.
    LowRes,
}

impl From<String> for MeshResolution {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "fullres" => MeshResolution::FullRes,
            "lowres" => MeshResolution::LowRes,
            _ => MeshResolution::FullRes,
        }
    }
}

/// Container for slice data, referenced by objects to provide 2.5D geometry.
///
/// A SliceStack encapsulates all slice data for an object. It can either contain
/// actual slice data directly or reference slices from separate model parts in the
/// 3MF package via [`SliceRef`] elements.
///
/// # Important Notes
///
/// - SliceStacks MUST NOT contain both `<slice>` and `<sliceref>` elements concurrently
/// - The zbottom attribute indicates the starting level relative to the build platform
/// - SliceStack IDs must be unique within a single model part
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "slicestack")
)]
pub struct SliceStack {
    /// Unique identifier for this slice stack within the model part.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub id: ResourceId,

    /// Starting level relative to the build platform in model units.
    /// This allows alignment between mesh vertices and slice data.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub zbottom: Option<f64>,

    /// Actual slice data if this stack contains slices directly.
    /// Mutually exclusive with `slicerefs`.
    /// Note: serialized directly as `<slice>` elements without a wrapper
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "slice", default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(rename = "slice")
    )]
    pub slice: Vec<Slice>,

    /// References to external slice stacks if data is in separate files.
    /// Mutually exclusive with `slice`.
    /// Note: serialized directly as `<sliceref>` elements without a wrapper
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "sliceref", default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(rename = "sliceref")
    )]
    pub sliceref: Vec<SliceRef>,
}

/// Container for slice elements within a slice stack.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "slices")
)]
pub struct Slices {
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "slice", default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(rename = "slice")
    )]
    pub slice: Vec<Slice>,
}

/// Container for sliceref elements within a slice stack.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "slicerefs")
)]
pub struct SliceRefs {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub sliceref: Vec<SliceRef>,
}

/// Reference to slice data in a separate model part within the 3MF package.
///
/// This allows slice data to be stored in separate XML files for easier parsing
/// and better organization of large datasets.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "sliceref")
)]
pub struct SliceRef {
    /// Identifies the SliceStack in the referenced file.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub slicestackid: ResourceId,

    /// Absolute path to the model file containing the slice data.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub slicepath: String,
}

/// Individual 2D slice layer representing a horizontal cross-section.
///
/// A slice defines the geometry at a specific z-height. It contains vertices
/// and polygons that describe the 2D contours. A slice can be empty (containing
/// only ztop) to represent void space.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "slice")
)]
pub struct Slice {
    /// Z-position of the top of this slice relative to the build platform.
    /// Must be monotonically increasing throughout the slice stack.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub ztop: f64,

    /// 2D vertices for this slice. Required if slice contains geometry.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub vertices: Option<SliceVertices>,

    /// Polygons defining the contours of this slice.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub polygon: Vec<Polygon>,
}

/// Container for 2D vertices within a slice.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "vertices")
)]
pub struct SliceVertices {
    /// 2D vertex data. Must contain at least 2 vertices if present.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub vertex: Vec<SliceVertex>,
}

/// 2D vertex representing a point in slice space.
///
/// Vertices are referenced by zero-based indices in segments.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "vertex")
)]
pub struct SliceVertex {
    /// X coordinate of the vertex.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub x: f64,

    /// Y coordinate of the vertex.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub y: f64,
}

/// Closed or open contour defined by a sequence of segments.
///
/// For model/solidsupport objects, polygons MUST be closed (final segment
/// connects back to startv). For support objects, polygons are open contours.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "polygon")
)]
pub struct Polygon {
    /// Index of the first vertex of the first segment.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub startv: ResourceIndex,

    /// Segments defining this polygon.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub segment: Vec<Segment>,
}

/// Line segment connecting vertices in a slice polygon.
///
/// Each segment connects from the previous segment's v2 (or startv for the
/// first segment) to this segment's v2. This creates a chain of connected
/// vertices that form the polygon.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(SLICE_NS), rename = "segment")
)]
pub struct Segment {
    /// Index of the second vertex of this segment.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub v2: ResourceIndex,

    /// Property index for the first vertex of this segment (overrides slice-level).
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub p1: OptionalResourceIndex,

    /// Property index for the second vertex of this segment (overrides slice-level).
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub p2: OptionalResourceIndex,

    /// Property group ID for this segment (overrides object-level).
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_optional_resource_id::default_none",
            deserialize_with = "crate::core::types::serde_optional_resource_id::deserialize"
        )
    )]
    pub pid: OptionalResourceId,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::SLICE_NS;

    use super::{MeshResolution, Slice, SliceRef, SliceStack, SliceVertex, SliceVertices};

    #[test]
    pub fn toxml_meshresolution_fullres_test() {
        let xml_string = format!(
            r#"<MeshResolution xmlns="{}">fullres</MeshResolution>"#,
            SLICE_NS
        );
        let resolution = MeshResolution::FullRes;
        let resolution_string = to_string(&resolution).unwrap();

        assert_eq!(resolution_string, xml_string);
    }

    #[test]
    pub fn toxml_meshresolution_lowres_test() {
        let xml_string = format!(
            r#"<MeshResolution xmlns="{}">lowres</MeshResolution>"#,
            SLICE_NS
        );
        let resolution = MeshResolution::LowRes;
        let resolution_string = to_string(&resolution).unwrap();

        assert_eq!(resolution_string, xml_string);
    }

    #[test]
    pub fn toxml_slice_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="1.5" y="2.5" />"#, SLICE_NS);
        let vertex = SliceVertex { x: 1.5, y: 2.5 };
        let vertex_string = to_string(&vertex).unwrap();

        assert_eq!(vertex_string, xml_string);
    }

    #[test]
    pub fn toxml_slice_test() {
        let xml_string = format!(
            r#"<slice xmlns="{}" ztop="0.1"><vertices><vertex x="0.0" y="0.0" /><vertex x="1.0" y="0.0" /><vertex x="1.0" y="1.0" /><vertex x="0.0" y="1.0" /></vertices><polygon startv="0"><segment v2="1" /><segment v2="2" /><segment v2="3" /></polygon></slice>"#,
            SLICE_NS
        );
        let slice = Slice {
            ztop: 0.1,
            vertices: Some(SliceVertices {
                vertex: vec![
                    SliceVertex { x: 0.0, y: 0.0 },
                    SliceVertex { x: 1.0, y: 0.0 },
                    SliceVertex { x: 1.0, y: 1.0 },
                    SliceVertex { x: 0.0, y: 1.0 },
                ],
            }),
            polygon: vec![super::Polygon {
                startv: 0,
                segment: vec![
                    super::Segment {
                        v2: 1,
                        p1: crate::core::OptionalResourceIndex::none(),
                        p2: crate::core::OptionalResourceIndex::none(),
                        pid: crate::core::OptionalResourceId::none(),
                    },
                    super::Segment {
                        v2: 2,
                        p1: crate::core::OptionalResourceIndex::none(),
                        p2: crate::core::OptionalResourceIndex::none(),
                        pid: crate::core::OptionalResourceId::none(),
                    },
                    super::Segment {
                        v2: 3,
                        p1: crate::core::OptionalResourceIndex::none(),
                        p2: crate::core::OptionalResourceIndex::none(),
                        pid: crate::core::OptionalResourceId::none(),
                    },
                ],
            }],
        };
        let slice_string = to_string(&slice).unwrap();

        assert_eq!(slice_string, xml_string);
    }

    #[test]
    pub fn toxml_sliceref_test() {
        let xml_string = format!(
            r#"<sliceref xmlns="{}" slicestackid="2" slicepath="/2D/slices.model" />"#,
            SLICE_NS
        );
        let sliceref = SliceRef {
            slicestackid: 2,
            slicepath: "/2D/slices.model".to_owned(),
        };
        let sliceref_string = to_string(&sliceref).unwrap();

        assert_eq!(sliceref_string, xml_string);
    }

    #[test]
    pub fn toxml_slicestack_with_slices_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1" zbottom="0.0"><slice ztop="0.1"><vertices><vertex x="0.0" y="0.0" /><vertex x="1.0" y="0.0" /></vertices><polygon startv="0"><segment v2="1" /></polygon></slice></slicestack>"#,
            SLICE_NS
        );
        let slicestack = SliceStack {
            id: 1,
            zbottom: Some(0.0),
            slice: vec![Slice {
                ztop: 0.1,
                vertices: Some(SliceVertices {
                    vertex: vec![
                        SliceVertex { x: 0.0, y: 0.0 },
                        SliceVertex { x: 1.0, y: 0.0 },
                    ],
                }),
                polygon: vec![super::Polygon {
                    startv: 0,
                    segment: vec![super::Segment {
                        v2: 1,
                        p1: crate::core::OptionalResourceIndex::none(),
                        p2: crate::core::OptionalResourceIndex::none(),
                        pid: crate::core::OptionalResourceId::none(),
                    }],
                }],
            }],
            sliceref: vec![],
        };
        let slicestack_string = to_string(&slicestack).unwrap();

        assert_eq!(slicestack_string, xml_string);
    }

    #[test]
    pub fn toxml_slicestack_with_slicerefs_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1" zbottom="0.0"><sliceref slicestackid="2" slicepath="/2D/slices1.model" /><sliceref slicestackid="3" slicepath="/2D/slices2.model" /></slicestack>"#,
            SLICE_NS
        );
        let slicestack = SliceStack {
            id: 1,
            zbottom: Some(0.0),
            slice: vec![],
            sliceref: vec![
                SliceRef {
                    slicestackid: 2,
                    slicepath: "/2D/slices1.model".to_owned(),
                },
                SliceRef {
                    slicestackid: 3,
                    slicepath: "/2D/slices2.model".to_owned(),
                },
            ],
        };
        let slicestack_string = to_string(&slicestack).unwrap();

        assert_eq!(slicestack_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::core::{OptionalResourceId, OptionalResourceIndex};
    use crate::threemf_namespaces::SLICE_NS;

    use super::{MeshResolution, Slice, SliceRef, SliceStack, SliceVertex, SliceVertices};

    #[test]
    pub fn fromxml_meshresolution_fullres_test() {
        let xml_string = format!(
            r#"<MeshResolution xmlns="{}">fullres</MeshResolution>"#,
            SLICE_NS
        );
        let resolution = from_str::<MeshResolution>(&xml_string).unwrap();

        assert_eq!(resolution, MeshResolution::FullRes);
    }

    #[test]
    pub fn fromxml_meshresolution_lowres_test() {
        let xml_string = format!(
            r#"<MeshResolution xmlns="{}">lowres</MeshResolution>"#,
            SLICE_NS
        );
        let resolution = from_str::<MeshResolution>(&xml_string).unwrap();

        assert_eq!(resolution, MeshResolution::LowRes);
    }

    #[test]
    pub fn fromxml_slice_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="1.5" y="2.5" />"#, SLICE_NS);
        let vertex = from_str::<SliceVertex>(&xml_string).unwrap();

        assert_eq!(vertex, SliceVertex { x: 1.5, y: 2.5 });
    }

    #[test]
    pub fn fromxml_slice_test() {
        let xml_string = format!(
            r#"<slice xmlns="{}" ztop="0.1"><vertices><vertex x="0.0" y="0.0" /><vertex x="1.0" y="0.0" /></vertices><polygon startv="0"><segment v2="1" /></polygon></slice>"#,
            SLICE_NS
        );
        let slice = from_str::<Slice>(&xml_string).unwrap();

        assert_eq!(
            slice,
            Slice {
                ztop: 0.1,
                vertices: Some(SliceVertices {
                    vertex: vec![
                        SliceVertex { x: 0.0, y: 0.0 },
                        SliceVertex { x: 1.0, y: 0.0 },
                    ],
                }),
                polygon: vec![super::Polygon {
                    startv: 0,
                    segment: vec![super::Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_sliceref_test() {
        let xml_string = format!(
            r#"<sliceref xmlns="{}" slicestackid="2" slicepath="/2D/slices.model" />"#,
            SLICE_NS
        );
        let sliceref = from_str::<SliceRef>(&xml_string).unwrap();

        assert_eq!(
            sliceref,
            SliceRef {
                slicestackid: 2,
                slicepath: "/2D/slices.model".to_owned(),
            }
        );
    }

    #[test]
    pub fn fromxml_slicestack_with_slices_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1" zbottom="0.0"><slice ztop="0.1"><vertices><vertex x="0.0" y="0.0" /></vertices></slice></slicestack>"#,
            SLICE_NS
        );
        let slicestack = from_str::<SliceStack>(&xml_string).unwrap();

        assert_eq!(slicestack.id, 1);
        assert_eq!(slicestack.zbottom, Some(0.0));
        assert!(!slicestack.slice.is_empty());
        assert!(slicestack.sliceref.is_empty());
    }

    #[test]
    pub fn fromxml_slicestack_with_slicerefs_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1"><sliceref slicestackid="2" slicepath="/2D/slices.model" /></slicestack>"#,
            SLICE_NS
        );
        let slicestack = from_str::<SliceStack>(&xml_string).unwrap();

        assert_eq!(slicestack.id, 1);
        assert!(slicestack.slice.is_empty());
        assert!(!slicestack.sliceref.is_empty());
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::core::{OptionalResourceId, OptionalResourceIndex};
    use crate::threemf_namespaces::SLICE_NS;

    use super::{MeshResolution, Slice, SliceRef, SliceStack, SliceVertex, SliceVertices};

    #[test]
    pub fn fromxml_meshresolution_fullres_test() {
        let xml_string = format!(
            r#"<MeshResolution xmlns="{}">fullres</MeshResolution>"#,
            SLICE_NS
        );
        let resolution = from_str::<MeshResolution>(&xml_string).unwrap();

        assert_eq!(resolution, MeshResolution::FullRes);
    }

    #[test]
    pub fn fromxml_meshresolution_lowres_test() {
        let xml_string = format!(
            r#"<MeshResolution xmlns="{}">lowres</MeshResolution>"#,
            SLICE_NS
        );
        let resolution = from_str::<MeshResolution>(&xml_string).unwrap();

        assert_eq!(resolution, MeshResolution::LowRes);
    }

    #[test]
    pub fn fromxml_slice_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="1.5" y="2.5" />"#, SLICE_NS);
        let vertex = from_str::<SliceVertex>(&xml_string).unwrap();

        assert_eq!(vertex, SliceVertex { x: 1.5, y: 2.5 });
    }

    #[test]
    pub fn fromxml_slice_test() {
        let xml_string = format!(
            r#"<slice xmlns="{}" ztop="0.1"><vertices><vertex x="0.0" y="0.0" /><vertex x="1.0" y="0.0" /></vertices><polygon startv="0"><segment v2="1" /></polygon></slice>"#,
            SLICE_NS
        );
        let slice = from_str::<Slice>(&xml_string).unwrap();

        assert_eq!(
            slice,
            Slice {
                ztop: 0.1,
                vertices: Some(SliceVertices {
                    vertex: vec![
                        SliceVertex { x: 0.0, y: 0.0 },
                        SliceVertex { x: 1.0, y: 0.0 },
                    ],
                }),
                polygon: vec![super::Polygon {
                    startv: 0,
                    segment: vec![super::Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_sliceref_test() {
        let xml_string = format!(
            r#"<sliceref xmlns="{}" slicestackid="2" slicepath="/2D/slices.model" />"#,
            SLICE_NS
        );
        let sliceref = from_str::<SliceRef>(&xml_string).unwrap();

        assert_eq!(
            sliceref,
            SliceRef {
                slicestackid: 2,
                slicepath: "/2D/slices.model".to_owned(),
            }
        );
    }

    #[test]
    pub fn fromxml_slicestack_with_slices_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1" zbottom="0.0"><slice ztop="0.1"><vertices><vertex x="0.0" y="0.0" /></vertices></slice></slicestack>"#,
            SLICE_NS
        );
        let slicestack = from_str::<SliceStack>(&xml_string).unwrap();

        assert_eq!(slicestack.id, 1);
        assert_eq!(slicestack.zbottom, Some(0.0));
        assert!(!slicestack.slice.is_empty());
        assert!(slicestack.sliceref.is_empty());
    }

    #[test]
    pub fn fromxml_slicestack_with_slicerefs_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1"><sliceref slicestackid="2" slicepath="/2D/slices.model" /></slicestack>"#,
            SLICE_NS
        );
        let slicestack = from_str::<SliceStack>(&xml_string).unwrap();

        assert_eq!(slicestack.id, 1);
        assert!(slicestack.slice.is_empty());
        assert!(!slicestack.sliceref.is_empty());
    }
}

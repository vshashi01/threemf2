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
    core::{
        OptionalResourceId, OptionalResourceIndex, ResourceIndex,
        types::{Double, ResourceId},
    },
    threemf_namespaces::SLICE_NS,
};

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

const MAX_VERTEX_BUFFER: usize = 1000;

/// Indicates the intended resolution of mesh models when slice data is present.
///
/// When a 3MF package contains both mesh and slice data, this attribute helps
/// consumers understand whether the mesh is intended for fabrication or is a
/// lower-resolution representation.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(
    feature = "speed-optimized-read",
    serde(from = "String", rename_all = "lowercase")
)]
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
    pub zbottom: Option<Double>,

    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub slice: Vec<Slice>,

    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub sliceref: Vec<SliceRef>,
}

impl SliceStack {
    pub fn has_owned_slices(&self) -> bool {
        // matches!(self.kind, SliceDataKind::Slice(_))
        !self.slice.is_empty()
    }
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
    pub ztop: Double,

    /// 2D vertices for this slice. Required if slice contains geometry.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub vertices: Option<Vertices>,

    /// Polygons defining the contours of this slice.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub polygon: Vec<Polygon>,
}

/// Container for 2D vertices within a slice.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "write", xml(ns(SLICE_NS), rename = "vertices"))]
pub struct Vertices {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub vertex: Vec<Vertex>,
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Vertices {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: SLICE_NS,
            name: "vertices",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        let mut vertices: Vec<Vertex> = Vec::with_capacity(MAX_VERTEX_BUFFER);

        while let Some(node) = deserializer.next() {
            if let Ok(n) = node
                && let instant_xml::de::Node::Open(element) = n
            {
                //println!("This is element value {:?}", element);
                let mut vertex_value: Option<Vertex> = None;
                let mut nested = deserializer.nested(element);

                if <Vertex as instant_xml::FromXml>::deserialize(
                    &mut vertex_value,
                    "vertex",
                    &mut nested,
                )
                .is_ok()
                    && let Some(vertex) = vertex_value
                {
                    vertices.push(vertex);
                };
            }
        }

        vertices.shrink_to_fit();
        *into = Some(Vertices { vertex: vertices });

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Scalar;
}

/// 2D vertex representing a point in slice space.
///
/// Vertices are referenced by zero-based indices in segments.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "write", xml(ns(SLICE_NS), rename = "vertex"))]
pub struct Vertex {
    #[cfg_attr(feature = "write", xml(attribute))]
    pub x: Double,

    #[cfg_attr(feature = "write", xml(attribute))]
    pub y: Double,
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Vertex {
    #[inline]
    fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: SLICE_NS,
            name: "vertex",
        }
    }
    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        _: &'static str,
        deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
    ) -> ::std::result::Result<(), ::instant_xml::Error> {
        use ::instant_xml::Error;
        use ::instant_xml::de::Node;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;

        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;

                    match id.name.as_bytes().first() {
                        Some(b'x') => {
                            x = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default()
                        }
                        Some(b'y') => {
                            y = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default()
                        }
                        _ => {}
                    };
                }
                Node::Open(data) => {
                    let mut nested = deserializer.nested(data);
                    nested.ignore()?;
                }
                Node::Text(_) => {}
                _ => {
                    return Err(Error::UnexpectedNode("Unexpected".to_owned()));
                }
            }
        }

        *into = Some(Self {
            x: Double::new(x),
            y: Double::new(y),
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

/// Closed or open contour defined by a sequence of segments.
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
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "write", xml(ns(SLICE_NS), rename = "segment"))]
pub struct Segment {
    /// Index of the second vertex of this segment.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v2: ResourceIndex,

    /// Property index for the first vertex of this segment (overrides slice-level).
    #[cfg_attr(feature = "write", xml(attribute))]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::opt_res_index_impl::default_none",
            deserialize_with = "crate::core::types::opt_res_index_impl::deserialize"
        )
    )]
    pub p1: OptionalResourceIndex,

    /// Property index for the second vertex of this segment (overrides slice-level).
    #[cfg_attr(feature = "write", xml(attribute))]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::opt_res_index_impl::default_none",
            deserialize_with = "crate::core::types::opt_res_index_impl::deserialize"
        )
    )]
    pub p2: OptionalResourceIndex,

    /// Property group ID for this segment (overrides object-level).
    #[cfg_attr(feature = "write", xml(attribute))]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::opt_res_id_impl::default_none",
            deserialize_with = "crate::core::types::opt_res_id_impl::deserialize"
        )
    )]
    pub pid: OptionalResourceId,
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Segment {
    #[inline]
    fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: SLICE_NS,
            name: "segment",
        }
    }
    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        _: &'static str,
        deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
    ) -> ::std::result::Result<(), ::instant_xml::Error> {
        use ::instant_xml::Error;
        use ::instant_xml::de::Node;
        let mut v2: ResourceIndex = 0;
        let mut p1: OptionalResourceIndex = OptionalResourceIndex::none();
        let mut p2: OptionalResourceIndex = OptionalResourceIndex::none();
        let mut pid: OptionalResourceId = OptionalResourceId::none();

        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;

                    match id.name {
                        "v2" => v2 = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default(),
                        "p1" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                p1 = OptionalResourceIndex::new(value);
                            }
                        }
                        "p2" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                p2 = OptionalResourceIndex::new(value);
                            }
                        }
                        "pid" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                pid = OptionalResourceId::new(value);
                            }
                        }
                        _ => {}
                    };
                }
                Node::Open(data) => {
                    let mut nested = deserializer.nested(data);
                    nested.ignore()?;
                }
                Node::Text(_) => {}
                _ => {
                    return Err(Error::UnexpectedNode("Unexpected".to_owned()));
                }
            }
        }

        *into = Some(Self { v2, p1, p2, pid });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::SLICE_NS;

    use super::{MeshResolution, Slice, SliceRef, SliceStack, Vertex, Vertices};

    #[derive(Debug, ToXml)]
    struct WriteResolution {
        res: MeshResolution,
    }

    #[test]
    pub fn toxml_meshresolution_fullres_test() {
        let xml_string = format!(
            r#"<WriteResolution><res xmlns="{}">fullres</res></WriteResolution>"#,
            SLICE_NS
        );
        let resolution = WriteResolution {
            res: MeshResolution::FullRes,
        };
        let resolution_string = to_string(&resolution).unwrap();

        assert_eq!(resolution_string, xml_string);
    }

    #[test]
    pub fn toxml_meshresolution_lowres_test() {
        let xml_string = format!(
            r#"<WriteResolution><res xmlns="{}">lowres</res></WriteResolution>"#,
            SLICE_NS
        );
        let resolution = WriteResolution {
            res: MeshResolution::LowRes,
        };
        let resolution_string = to_string(&resolution).unwrap();

        assert_eq!(resolution_string, xml_string);
    }

    #[test]
    pub fn toxml_slice_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="1.5" y="2.5" />"#, SLICE_NS);
        let vertex = Vertex {
            x: 1.5.into(),
            y: 2.5.into(),
        };
        let vertex_string = to_string(&vertex).unwrap();

        assert_eq!(vertex_string, xml_string);
    }

    #[test]
    pub fn toxml_slice_test() {
        let xml_string = format!(
            r#"<slice xmlns="{}" ztop="0.1"><vertices><vertex x="0" y="0" /><vertex x="1" y="0" /><vertex x="1" y="1" /><vertex x="0" y="1" /></vertices><polygon startv="0"><segment v2="1" /><segment v2="2" /><segment v2="3" /></polygon></slice>"#,
            SLICE_NS
        );
        let slice = Slice {
            ztop: 0.1.into(),
            vertices: Some(Vertices {
                vertex: vec![
                    Vertex {
                        x: 0.0.into(),
                        y: 0.0.into(),
                    },
                    Vertex {
                        x: 1.0.into(),
                        y: 0.0.into(),
                    },
                    Vertex {
                        x: 1.0.into(),
                        y: 1.0.into(),
                    },
                    Vertex {
                        x: 0.0.into(),
                        y: 1.0.into(),
                    },
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
            r#"<slicestack xmlns="{}" id="1" zbottom="0"><slice ztop="0.1"><vertices><vertex x="0" y="0" /><vertex x="1" y="0" /></vertices><polygon startv="0"><segment v2="1" /></polygon></slice></slicestack>"#,
            SLICE_NS
        );
        let slicestack = SliceStack {
            id: 1,
            zbottom: Some(0.0.into()),
            sliceref: vec![],
            slice: vec![Slice {
                ztop: 0.1.into(),
                vertices: Some(Vertices {
                    vertex: vec![
                        Vertex {
                            x: 0.0.into(),
                            y: 0.0.into(),
                        },
                        Vertex {
                            x: 1.0.into(),
                            y: 0.0.into(),
                        },
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
        };
        let slicestack_string = to_string(&slicestack).unwrap();

        assert_eq!(slicestack_string, xml_string);
    }

    #[test]
    pub fn toxml_slicestack_with_slicerefs_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1" zbottom="0"><sliceref slicestackid="2" slicepath="/2D/slices1.model" /><sliceref slicestackid="3" slicepath="/2D/slices2.model" /></slicestack>"#,
            SLICE_NS
        );
        let slicestack = SliceStack {
            id: 1,
            zbottom: Some(0.0.into()),
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

    use super::{MeshResolution, Slice, SliceRef, SliceStack, Vertex, Vertices};

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
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(
            vertex,
            Vertex {
                x: 1.5.into(),
                y: 2.5.into()
            }
        );
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
                ztop: 0.1.into(),
                vertices: Some(Vertices {
                    vertex: vec![
                        Vertex {
                            x: 0.0.into(),
                            y: 0.0.into()
                        },
                        Vertex {
                            x: 1.0.into(),
                            y: 0.0.into()
                        },
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
        assert_eq!(slicestack.zbottom, Some(0.0.into()));
        assert!(slicestack.has_owned_slices());
    }

    #[test]
    pub fn fromxml_slicestack_with_slicerefs_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1"><sliceref slicestackid="2" slicepath="/2D/slices.model" /></slicestack>"#,
            SLICE_NS
        );
        let slicestack = from_str::<SliceStack>(&xml_string).unwrap();

        assert_eq!(slicestack.id, 1);
        assert!(!slicestack.has_owned_slices());
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::core::{OptionalResourceId, OptionalResourceIndex};
    use crate::threemf_namespaces::SLICE_NS;

    use super::{MeshResolution, Slice, SliceRef, SliceStack, Vertex, Vertices};

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
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(
            vertex,
            Vertex {
                x: 1.5.into(),
                y: 2.5.into()
            }
        );
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
                ztop: 0.1.into(),
                vertices: Some(Vertices {
                    vertex: vec![
                        Vertex {
                            x: 0.0.into(),
                            y: 0.0.into()
                        },
                        Vertex {
                            x: 1.0.into(),
                            y: 0.0.into()
                        },
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
        assert_eq!(slicestack.zbottom, Some(0.0.into()));
        assert!(slicestack.has_owned_slices())
    }

    #[test]
    pub fn fromxml_slicestack_with_slicerefs_test() {
        let xml_string = format!(
            r#"<slicestack xmlns="{}" id="1"><sliceref slicestackid="2" slicepath="/2D/slices.model" /></slicestack>"#,
            SLICE_NS
        );
        let slicestack = from_str::<SliceStack>(&xml_string).unwrap();

        assert_eq!(slicestack.id, 1);
        assert!(!slicestack.has_owned_slices());
    }
}

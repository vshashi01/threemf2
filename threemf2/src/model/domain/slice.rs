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
//! - `SliceStack` - Container for slice data, referenced by objects
//! - `Slice` - Individual 2D layer with vertices and polygons
//! - `SliceRef` - Reference to external slice files in the 3MF package
//! - `SliceVertex` - 2D vertex with x, y coordinates
//! - `Polygon` - Closed or open contour defined by segments
//! - `Segment` - Line segment connecting vertices
//! - `MeshResolution` - Indicates mesh quality (fullres or lowres)
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

use crate::model::domain::any_extensions::{
    AnyAttributeExtensions, AnyExtensions, ItemAttributeExtensions,
};
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::{
    model::{
        Double, OptionalResourceId, OptionalResourceIndex, PathResource, ResourceId, ResourceIndex,
    },
    threemf_namespaces::SLICE_NS,
};

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

const MAX_VERTEX_BUFFER: usize = 1000;

/// Indicates the intended resolution of mesh models when slice data is present.
///
/// When a 3MF package contains both mesh and slice data, this attribute helps
/// consumers understand whether the mesh is intended for fabrication or is a
/// lower-resolution representation.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
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
#[derive(Debug, PartialEq, Clone)]
pub struct SliceStack {
    /// Unique identifier for this slice stack within the model part.
    pub id: ResourceId,

    /// Starting level relative to the build platform in model units.
    /// This allows alignment between mesh vertices and slice data.
    pub zbottom: Option<Double>,

    /// Owned slice data entries.
    pub slice: Vec<Slice>,

    /// References to external slice data.
    pub sliceref: Vec<SliceRef>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    SliceStack {
        tag: "slicestack",
        ns: SLICE_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
            zbottom: Option<Double> => ("", "zbottom"),
        },
        elements: {
            slice: Vec<Slice> => (SLICE_NS, "slice"),
            sliceref: Vec<SliceRef> => (SLICE_NS, "sliceref"),
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

impl SliceStack {
    /// Returns true if this slice stack contains owned slice data.
    pub fn has_owned_slices(&self) -> bool {
        // matches!(self.kind, SliceDataKind::Slice(_))
        !self.slice.is_empty()
    }
}

/// Reference to slice data in a separate model part within the 3MF package.
///
/// This allows slice data to be stored in separate XML files for easier parsing
/// and better organization of large datasets.
#[derive(Debug, PartialEq, Clone)]
pub struct SliceRef {
    /// Identifies the SliceStack in the referenced file.
    pub slicestackid: ResourceId,

    /// Absolute path to the model file containing the slice data.
    pub slicepath: PathResource,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    SliceRef {
        tag: "sliceref",
        ns: SLICE_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            slicestackid: ResourceId => ("", "slicestackid"),
            slicepath: PathResource => ("", "slicepath"),
        },
        elements: {
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// Individual 2D slice layer representing a horizontal cross-section.
///
/// A slice defines the geometry at a specific z-height. It contains vertices
/// and polygons that describe the 2D contours. A slice can be empty (containing
/// only ztop) to represent void space.
#[derive(Debug, PartialEq, Clone)]
pub struct Slice {
    /// Z-position of the top of this slice relative to the build platform.
    /// Must be monotonically increasing throughout the slice stack.
    pub ztop: Double,

    /// 2D vertices for this slice. Required if slice contains geometry.
    pub vertices: Option<Vertices>,

    /// Polygons defining the contours of this slice.
    pub polygon: Vec<Polygon>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Slice {
        tag: "slice",
        ns: SLICE_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            ztop: Double => ("", "ztop"),
        },
        elements: {
            vertices: Option<Vertices> => (SLICE_NS, "vertices"),
            polygon: Vec<Polygon> => (SLICE_NS, "polygon"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// Container for 2D vertices within a slice.
#[derive(Debug, PartialEq, Clone)]
pub struct Vertices {
    /// 2D vertices within a slice.
    pub vertex: Vec<Vertex>,

    /// Foreign attributes of the individual `vertex` entries, by index in [`Self::vertex`].
    ///
    /// Only populated with the `any` feature.
    pub vertex_any_extensions: ItemAttributeExtensions,
}

#[cfg(feature = "write")]
impl ToXml for Vertices {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let mut context = instant_xml::ser::Context::<0>::default();
        context.default_ns = SLICE_NS;
        context.force_prefix = true;
        let element = serializer.write_start("vertices", SLICE_NS, Some(context))?;
        #[cfg(feature = "any")]
        let scope = write_item_namespaces(
            serializer,
            &self.vertex_any_extensions,
            self.vertex.len(),
            "vertex",
        )?;
        serializer.end_start()?;
        #[cfg(feature = "any")]
        crate::model::domain::xml_item::write_items(
            serializer,
            &self.vertex,
            &self.vertex_any_extensions,
            &scope,
        )?;
        #[cfg(not(feature = "any"))]
        crate::model::domain::xml_item::write_items(
            serializer,
            &self.vertex,
            &self.vertex_any_extensions,
        )?;
        serializer.write_close(element)
    }
}

/// Declares the namespaces of the per-entry foreign attributes of a slice list element, which
/// has no foreign attributes of its own.
#[cfg(all(feature = "write", feature = "any"))]
fn write_item_namespaces<W: std::fmt::Write + ?Sized>(
    serializer: &mut instant_xml::Serializer<W>,
    table: &ItemAttributeExtensions,
    len: usize,
    entry: &str,
) -> Result<crate::model::domain::any::write::ForeignScope, instant_xml::Error> {
    let mut scope = crate::model::domain::any::write::write_foreign_attributes(
        serializer,
        &[],
        SLICE_NS,
        crate::model::domain::any::AnyAttributes::empty(),
        &[],
    )?;
    crate::model::domain::any::write::declare_item_namespaces(
        serializer, &mut scope, SLICE_NS, table, len, entry,
    )?;
    Ok(scope)
}

#[cfg(feature = "read")]
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
        let mut vertex_any_extensions = ItemAttributeExtensions::default();

        while let Some(node) = deserializer.next() {
            if let Ok(n) = node
                && let instant_xml::de::Node::Open(element) = n
            {
                let mut nested = deserializer.nested(element);
                // Malformed entries are skipped.
                let _ = crate::model::domain::xml_item::read_item_into(
                    &mut nested,
                    &mut vertices,
                    &mut vertex_any_extensions,
                );
            }
        }

        vertices.shrink_to_fit();
        *into = Some(Vertices {
            vertex: vertices,
            vertex_any_extensions,
        });

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Scalar;
}

/// 2D vertex representing a point in slice space.
///
/// Vertices are referenced by zero-based indices in segments.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "write", xml(ns(SLICE_NS), rename = "vertex", force_prefix))]
pub struct Vertex {
    /// X coordinate in slice space.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub x: Double,

    /// Y coordinate in slice space.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub y: Double,
}

#[cfg(feature = "read")]
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
        *into = Some(
            <Self as crate::model::domain::xml_item::XmlItem>::read_item(
                deserializer,
                &mut crate::model::domain::any_extensions::ItemCapture::default(),
            )?,
        );
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

#[cfg(any(feature = "read", feature = "write"))]
impl crate::model::domain::xml_item::XmlItem for Vertex {
    const TAG: &'static str = "vertex";
    const NS: &'static str = SLICE_NS;

    #[cfg(feature = "read")]
    fn read_item(
        deserializer: &mut ::instant_xml::Deserializer<'_, '_>,
        capture: &mut crate::model::domain::any_extensions::ItemCapture,
    ) -> ::std::result::Result<Self, ::instant_xml::Error> {
        use ::instant_xml::Error;
        use ::instant_xml::de::Node;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;

        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    // Schema attributes are unqualified; prefixed ones belong to other namespaces.
                    if !id.ns.is_empty() {
                        capture.attribute(id, SLICE_NS, attr.prefix, &attr.value);
                        continue;
                    }

                    match id.name {
                        "x" => x = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default(),
                        "y" => y = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default(),
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

        Ok(Self {
            x: Double::new(x),
            y: Double::new(y),
        })
    }

    #[cfg(feature = "write")]
    fn write_open<W: std::fmt::Write + ?Sized>(
        &self,
        serializer: &mut ::instant_xml::Serializer<'_, W>,
    ) -> ::std::result::Result<(), ::instant_xml::Error> {
        let mut context = ::instant_xml::ser::Context::<0>::default();
        context.default_ns = SLICE_NS;
        context.force_prefix = true;
        serializer.write_start(Self::TAG, SLICE_NS, Some(context))?;
        if ToXml::present(&self.x) {
            serializer.write_attr("x", "", &self.x)?;
        }
        if ToXml::present(&self.y) {
            serializer.write_attr("y", "", &self.y)?;
        }
        Ok(())
    }
}

/// Closed or open contour defined by a sequence of segments.
#[derive(Debug, PartialEq, Clone)]
pub struct Polygon {
    /// Index of the first vertex of the first segment.
    pub startv: ResourceIndex,

    /// Segments defining this polygon.
    pub segment: Vec<Segment>,

    /// Foreign attributes of the individual `segment` entries, by index in [`Self::segment`].
    ///
    /// Only populated with the `any` feature.
    pub segment_any_extensions: ItemAttributeExtensions,
}

#[cfg(feature = "write")]
impl ToXml for Polygon {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let mut context = instant_xml::ser::Context::<0>::default();
        context.default_ns = SLICE_NS;
        context.force_prefix = true;
        let element = serializer.write_start("polygon", SLICE_NS, Some(context))?;
        if self.startv.present() {
            serializer.write_attr("startv", "", &self.startv)?;
        }
        #[cfg(feature = "any")]
        let scope = write_item_namespaces(
            serializer,
            &self.segment_any_extensions,
            self.segment.len(),
            "segment",
        )?;
        serializer.end_start()?;
        #[cfg(feature = "any")]
        crate::model::domain::xml_item::write_items(
            serializer,
            &self.segment,
            &self.segment_any_extensions,
            &scope,
        )?;
        #[cfg(not(feature = "any"))]
        crate::model::domain::xml_item::write_items(
            serializer,
            &self.segment,
            &self.segment_any_extensions,
        )?;
        serializer.write_close(element)
    }
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Polygon {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: SLICE_NS,
            name: "polygon",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, de::Node};

        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        let mut startv = <ResourceIndex as FromXml<'xml>>::Accumulator::default();
        let mut segment = Vec::new();
        let mut segment_any_extensions = ItemAttributeExtensions::default();
        while let Some(node) = deserializer.next() {
            match node? {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    if id.ns.is_empty() && id.name == "startv" {
                        crate::model::domain::xml_helpers::read_attribute::<ResourceIndex>(
                            &mut startv,
                            "startv",
                            deserializer,
                            attr.value,
                        )?;
                    }
                }
                Node::Open(element) => {
                    let id = deserializer.element_id(&element)?;
                    let mut nested = deserializer.nested(element);
                    if <Segment as FromXml<'xml>>::matches(id, None) {
                        crate::model::domain::xml_item::read_item_into(
                            &mut nested,
                            &mut segment,
                            &mut segment_any_extensions,
                        )?;
                    } else {
                        nested.ignore()?;
                    }
                }
                _ => {}
            }
        }

        *into = Some(Self {
            startv: startv.try_done("startv")?,
            segment,
            segment_any_extensions,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

/// Line segment connecting vertices in a slice polygon.
///
/// Each segment connects from the previous segment's v2 (or startv for the
/// first segment) to this segment's v2. This creates a chain of connected
/// vertices that form the polygon.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "write", xml(ns(SLICE_NS), rename = "segment", force_prefix))]
pub struct Segment {
    /// Index of the second vertex of this segment.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v2: ResourceIndex,

    /// Property index for the first vertex of this segment (overrides slice-level).
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p1: OptionalResourceIndex,

    /// Property index for the second vertex of this segment (overrides slice-level).
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p2: OptionalResourceIndex,

    /// Property group ID for this segment (overrides object-level).
    #[cfg_attr(feature = "write", xml(attribute))]
    pub pid: OptionalResourceId,
}

#[cfg(feature = "read")]
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
        *into = Some(
            <Self as crate::model::domain::xml_item::XmlItem>::read_item(
                deserializer,
                &mut crate::model::domain::any_extensions::ItemCapture::default(),
            )?,
        );
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

#[cfg(any(feature = "read", feature = "write"))]
impl crate::model::domain::xml_item::XmlItem for Segment {
    const TAG: &'static str = "segment";
    const NS: &'static str = SLICE_NS;

    #[cfg(feature = "read")]
    fn read_item(
        deserializer: &mut ::instant_xml::Deserializer<'_, '_>,
        capture: &mut crate::model::domain::any_extensions::ItemCapture,
    ) -> ::std::result::Result<Self, ::instant_xml::Error> {
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
                    // Schema attributes are unqualified; prefixed ones belong to other namespaces.
                    if !id.ns.is_empty() {
                        capture.attribute(id, SLICE_NS, attr.prefix, &attr.value);
                        continue;
                    }

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

        Ok(Self { v2, p1, p2, pid })
    }

    #[cfg(feature = "write")]
    fn write_open<W: std::fmt::Write + ?Sized>(
        &self,
        serializer: &mut ::instant_xml::Serializer<'_, W>,
    ) -> ::std::result::Result<(), ::instant_xml::Error> {
        let mut context = ::instant_xml::ser::Context::<0>::default();
        context.default_ns = SLICE_NS;
        context.force_prefix = true;
        serializer.write_start(Self::TAG, SLICE_NS, Some(context))?;
        if ToXml::present(&self.v2) {
            serializer.write_attr("v2", "", &self.v2)?;
        }
        if ToXml::present(&self.p1) {
            serializer.write_attr("p1", "", &self.p1)?;
        }
        if ToXml::present(&self.p2) {
            serializer.write_attr("p2", "", &self.p2)?;
        }
        if ToXml::present(&self.pid) {
            serializer.write_attr("pid", "", &self.pid)?;
        }
        Ok(())
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        model::{OptionalResourceId, OptionalResourceIndex, PathResource},
        threemf_namespaces::SLICE_NS,
    };

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
                vertex_any_extensions: Default::default(),
            }),
            polygon: vec![super::Polygon {
                startv: 0,
                segment: vec![
                    super::Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    super::Segment {
                        v2: 2,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    super::Segment {
                        v2: 3,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                ],
                segment_any_extensions: Default::default(),
            }],
            any_extensions: Default::default(),
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
            slicepath: PathResource::try_from("/2D/slices.model").unwrap(),
            any_extensions: Default::default(),
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
                    vertex_any_extensions: Default::default(),
                }),
                polygon: vec![super::Polygon {
                    startv: 0,
                    segment: vec![super::Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                    segment_any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
            }],
            any_extensions: Default::default(),
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
                    slicepath: PathResource::try_from("/2D/slices1.model").unwrap(),
                    any_extensions: Default::default(),
                },
                SliceRef {
                    slicestackid: 3,
                    slicepath: PathResource::try_from("/2D/slices2.model").unwrap(),
                    any_extensions: Default::default(),
                },
            ],
            any_extensions: Default::default(),
        };
        let slicestack_string = to_string(&slicestack).unwrap();

        assert_eq!(slicestack_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::model::{OptionalResourceId, OptionalResourceIndex, PathResource};
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
                    vertex_any_extensions: Default::default(),
                }),
                polygon: vec![super::Polygon {
                    startv: 0,
                    segment: vec![super::Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                    segment_any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
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
                slicepath: PathResource::try_from("/2D/slices.model").unwrap(),
                any_extensions: Default::default(),
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

#[cfg(test)]
mod from_string_tests {
    use super::MeshResolution;

    #[test]
    fn mesh_resolution_from_string() {
        assert_eq!(
            MeshResolution::from("lowres".to_string()),
            MeshResolution::LowRes
        );
        assert_eq!(
            MeshResolution::from("LOWRES".to_string()),
            MeshResolution::LowRes
        );
        assert_eq!(
            MeshResolution::from("fullres".to_string()),
            MeshResolution::FullRes
        );
        assert_eq!(
            MeshResolution::from("FULLRES".to_string()),
            MeshResolution::FullRes
        );
        assert_eq!(
            MeshResolution::from("unknown".to_string()),
            MeshResolution::FullRes
        );
    }
}

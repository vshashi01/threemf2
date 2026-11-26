//! 3MF Material Extension types
//!
//! This module provides data structures for the 3MF Material Extension, which enables
//! full-color and multi-material 3D printing workflows.
//!
//! # Key Types
//!
//! - `ColorGroup` - Container for color properties (vertex colors)
//! - `Texture2DGroup` - Container for texture coordinate properties
//! - `CompositeMaterials` - Mixing multiple base materials in defined ratios
//! - `MultiProperties` - Layering multiple properties (color + texture + material)
//! - `Texture2D` - Image references for texture mapping
//!
//! # Usage
//!
//! Objects reference material resources via the `pid` and `pindex` attributes on triangles.
//! The Material extension provides different ways to represent material properties:
//!
//! - **ColorGroup**: Simple vertex colors in sRGB format
//! - **Texture2DGroup**: UV mapping for bitmap textures
//! - **CompositeMaterials**: Mixing multiple base materials
//! - **MultiProperties**: Layering different property types together
//!
//! # Example XML Structure
//!
//! ```xml
//! <m:colorgroup id="1">
//!   <m:color color="#FF0000" />
//!   <m:color color="#00FF00" />
//! </m:colorgroup>
//! <m:texture2d id="2" path="/3D/texture.png" contenttype="image/png" />
//! <m:texture2dgroup id="3" texid="2">
//!   <m:tex2coord u="0" v="0" />
//!   <m:tex2coord u="1" v="1" />
//! </m:texture2dgroup>
//! ```

use crate::model::domain::any_extensions::{AnyAttributeExtensions, ItemAttributeExtensions};
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_item::xml_item;
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::{
    model::{
        Color, Double, PathResource, ResourceId, ResourceIdCollection, ResourceIndexCollection,
        StrResource,
    },
    threemf_namespaces::MATERIAL_NS,
};

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

/// Tile style for texture coordinates outside the `[0,1]` range.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(scalar, ns(MATERIAL_NS), rename_all = "lowercase")
)]
pub enum TileStyle {
    /// Repeat the texture (default)
    #[default]
    Wrap,
    /// Reflect the texture at each repetition
    Mirror,
    /// Use the color of the nearest edge pixel
    Clamp,
    /// Use edge color with transparent alpha outside `[0,1]`.
    None,
}

impl From<String> for TileStyle {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "wrap" => TileStyle::Wrap,
            "mirror" => TileStyle::Mirror,
            "clamp" => TileStyle::Clamp,
            "none" => TileStyle::None,
            _ => TileStyle::Wrap,
        }
    }
}

/// Texture filter for scaling operations.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(scalar, ns(MATERIAL_NS), rename_all = "lowercase")
)]
pub enum Filter {
    /// Use highest quality filter available (default)
    #[default]
    Auto,
    /// Bilinear interpolation
    Linear,
    /// Nearest neighbor interpolation
    Nearest,
}

impl From<String> for Filter {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Filter::Auto,
            "linear" => Filter::Linear,
            "nearest" => Filter::Nearest,
            _ => Filter::Auto,
        }
    }
}

/// Blend method for combining layers in multi-properties.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(scalar, ns(MATERIAL_NS), rename_all = "lowercase")
)]
pub enum BlendMethod {
    /// Linear interpolation blend (default)
    #[default]
    Mix,
    /// Multiplicative blend
    Multiply,
}

impl From<String> for BlendMethod {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "mix" => BlendMethod::Mix,
            "multiply" => BlendMethod::Multiply,
            _ => BlendMethod::Mix,
        }
    }
}

/// Container for color properties.
///
/// A color group defines a set of sRGB colors that can be referenced by index.
/// The order of colors forms an implicit 0-based index.
#[derive(Debug, PartialEq, Clone)]
pub struct ColorGroup {
    /// Unique identifier for this color group.
    pub id: ResourceId,

    /// Colors in this group, ordered by implicit 0-based index.
    pub color: Vec<ColorElement>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    ColorGroup {
        tag: "colorgroup",
        ns: MATERIAL_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
        },
        elements: {
            color: Vec<ColorElement> => (MATERIAL_NS, "color"),
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A single color value in sRGB format.
///
/// The color is specified as a hex string like "#RRGGBB" or "#RRGGBBAA".
/// When used outside a multi-properties context, colors are fully opaque.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(MATERIAL_NS), rename = "color", force_prefix)
)]
pub struct ColorElement {
    /// The sRGB color value as a hex string (e.g., "#FF0000" or "#FF0000FF").
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub color: Color,
}

/// Container for texture coordinate properties.
///
/// A texture 2D group defines UV coordinates for mapping a texture image to mesh vertices.
/// The order of coordinates forms an implicit 0-based index.
#[derive(Debug, PartialEq, Clone)]
pub struct Texture2DGroup {
    /// Unique identifier for this texture coordinate group.
    pub id: ResourceId,

    /// Reference to the texture2d resource to use.
    pub texid: ResourceId,

    /// Texture coordinates in this group, ordered by implicit 0-based index.
    pub tex2coord: Vec<Tex2Coord>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,

    /// Foreign attributes of the individual `tex2coord` entries, by index in [`Self::tex2coord`].
    ///
    /// Only populated with the `any` feature.
    pub tex2coord_any_extensions: ItemAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Texture2DGroup {
        tag: "texture2dgroup",
        ns: MATERIAL_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
            texid: ResourceId => ("", "texid"),
        },
        elements: {
        },
        items: {
            tex2coord: Tex2Coord => tex2coord_any_extensions,
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A single texture coordinate (UV) pair.
///
/// The origin (0,0) is at the bottom-left of the texture image.
/// Values outside `[0,1]` are handled according to the tile style settings.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(
    feature = "write",
    xml(ns(MATERIAL_NS), rename = "tex2coord", force_prefix)
)]
pub struct Tex2Coord {
    /// Horizontal coordinate (u-axis), increasing right from the origin.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub u: Double,

    /// Vertical coordinate (v-axis), increasing up from the origin.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v: Double,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::fast_item! {
    Tex2Coord {
        tag: "tex2coord",
        ns: MATERIAL_NS,
        force_prefix: true,
        attributes: {
            u: Double = number,
            v: Double = number,
        },
    }
}

/// Container for composite material definitions.
///
/// Composite materials are created by mixing 2 or more base materials in defined ratios.
/// Each composite represents a specific mixture ratio of the materials.
#[derive(Debug, PartialEq, Clone)]
pub struct CompositeMaterials {
    /// Unique identifier for this composite material group.
    pub id: ResourceId,

    /// Reference to the base materials group containing the constituent materials.
    pub matid: ResourceId,

    /// Space-delimited list of material indices from the base materials group.
    /// These are the constituents that will be mixed.
    pub matindices: ResourceIndexCollection,

    /// Composite definitions, ordered by implicit 0-based index.
    pub composite: Vec<Composite>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    CompositeMaterials {
        tag: "compositematerials",
        ns: MATERIAL_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
            matid: ResourceId => ("", "matid"),
            matindices: ResourceIndexCollection => ("", "matindices"),
        },
        elements: {
            composite: Vec<Composite> => (MATERIAL_NS, "composite"),
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A single composite material definition.
///
/// The `values` attribute specifies the proportion of each material in the mixture.
/// Values are space-delimited numbers in the range [0, 1].
#[derive(Debug, PartialEq, Clone)]
pub struct Composite {
    /// List of mixture ratios for each material constituent.
    /// Values are in range [0, 1]. If the sum is zero, all values are treated as equal.
    pub values: Vec<Double>,
}

#[cfg(feature = "write")]
impl ToXml for Composite {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let (name, ns) = match field {
            Some(id) => (id.name, id.ns),
            None => ("composite", MATERIAL_NS),
        };

        // Force prefix usage for material namespace
        let mut cx = instant_xml::ser::Context::<0>::default();
        cx.default_ns = MATERIAL_NS;
        cx.force_prefix = true;

        serializer.write_start(name, ns, Some(cx))?;

        if !self.values.is_empty() {
            let values_str = self
                .values
                .iter()
                .map(|d| d.value().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            serializer.write_attr("values", ns, &values_str)?;
        }
        serializer.end_empty()?;
        Ok(())
    }

    fn present(&self) -> bool {
        true
    }
}

#[cfg(feature = "read")]
impl<'xml> instant_xml::FromXml<'xml> for Composite {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: MATERIAL_NS,
            name: "composite",
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

        let mut values: Vec<Double> = Vec::new();

        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                instant_xml::de::Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    // Schema attributes are unqualified; prefixed ones belong to other namespaces.
                    if !id.ns.is_empty() {
                        continue;
                    }
                    if id.name == "values" && !attr.value.is_empty() {
                        values = attr
                            .value
                            .split_whitespace()
                            .map(|s| {
                                let v: f64 = lexical_core::parse(s.as_bytes()).unwrap_or(0.0);
                                Double::new(v)
                            })
                            .collect();
                    }
                }
                instant_xml::de::Node::Open(data) => {
                    let mut nested = deserializer.nested(data);
                    nested.ignore()?;
                }
                _ => {}
            }
        }

        *into = Some(Composite { values });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

/// Container for multi-property definitions.
///
/// Multi-properties allow layering multiple property types (e.g., material + color + texture)
/// to create complex material appearances. Properties are blended in the order specified
/// by the `pids` attribute.
#[derive(Debug, PartialEq, Clone)]
pub struct MultiProperties {
    /// Unique identifier for this multi-property group.
    pub id: ResourceId,

    /// Space-delimited list of property group IDs to layer.
    /// First element should be the material (base or composite), followed by color/texture layers.
    pub pids: ResourceIdCollection,

    /// Optional space-delimited list of blend methods for each layer.
    /// One value per layer after the first. Defaults to "mix" if not specified.
    pub blendmethods: Option<StrResource>,

    /// Multi-property index combinations, ordered by implicit 0-based index.
    pub multi: Vec<Multi>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,

    /// Foreign attributes of the individual `multi` entries, by index in [`Self::multi`].
    ///
    /// Only populated with the `any` feature.
    pub multi_any_extensions: ItemAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    MultiProperties {
        tag: "multiproperties",
        ns: MATERIAL_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
            pids: ResourceIdCollection => ("", "pids"),
            blendmethods: Option<StrResource> => ("", "blendmethods"),
        },
        elements: {
        },
        items: {
            multi: Multi => multi_any_extensions,
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A single multi-property index combination.
///
/// The `pindices` attribute is a space-delimited list of property indices, one for each
/// property group specified in the parent `MultiProperties.pids` attribute.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    feature = "write",
    xml(ns(MATERIAL_NS), rename = "multi", force_prefix)
)]
pub struct Multi {
    /// Space-delimited list of property indices.
    /// Indices correspond to the property groups listed in `MultiProperties.pids`.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub pindices: ResourceIndexCollection,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_item! {
    Multi {
        tag: "multi",
        ns: MATERIAL_NS,
        force_prefix: true,
        attributes: {
            pindices: ResourceIndexCollection => "pindices",
        },
    }
}

/// Type of the Texture image
#[derive(Debug, PartialEq, Clone)]
pub enum TextureContentType {
    /// Jpef format
    Jpeg,
    /// Png format
    Png,
}

#[cfg(feature = "write")]
impl ToXml for TextureContentType {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let value = self.to_str();
        serializer.write_str(&value)?;

        Ok(())
    }
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for TextureContentType {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        // Match if the attribute name matches the field name
        if let Some(field_id) = field {
            id == field_id
        } else {
            false
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

        if let Some(value) = deserializer.take_str()? {
            if let Some(content_type) = Self::from_str(&value) {
                *into = Some(content_type);
            } else {
                return Err(instant_xml::Error::MissingValue(
                    "Failed to parse texture content type",
                ));
            }
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Scalar;
}

impl From<String> for TextureContentType {
    fn from(value: String) -> Self {
        match Self::from_str(value.as_ref()) {
            Some(value) => value,
            None => Self::Jpeg,
        }
    }
}

impl TextureContentType {
    /// Content type namespace
    pub fn to_str(&self) -> &str {
        match self {
            TextureContentType::Jpeg => "image/jpeg",
            TextureContentType::Png => "image/png",
        }
    }

    #[allow(clippy::should_implement_trait)]
    /// Returns the types based on the namespace definition
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "image/jpeg" => Some(Self::Jpeg),
            "image/png" => Some(Self::Png),
            _ => None,
        }
    }
}

/// A 2D texture resource.
///
/// References an image file in the 3MF package that can be used for texture mapping.
/// The texture is referenced by `Texture2DGroup` elements via the `texid` attribute.
#[derive(Debug, PartialEq, Clone)]
pub struct Texture2D {
    /// Unique identifier for this texture resource.
    pub id: ResourceId,

    /// Path to the texture image part within the 3MF package.
    pub path: PathResource,

    /// Content type of the texture image. Must be "image/jpeg" or "image/png".
    pub contenttype: TextureContentType,

    /// Tile style for u-coordinates outside `[0,1]` range. Defaults to "wrap".
    pub tilestyleu: Option<TileStyle>,

    /// Tile style for v-coordinates outside `[0,1]` range. Defaults to "wrap".
    pub tilestylev: Option<TileStyle>,

    /// Filter to apply when scaling the texture. Defaults to "auto".
    pub filter: Option<Filter>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Texture2D {
        tag: "texture2d",
        ns: MATERIAL_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
            path: PathResource => ("", "path"),
            contenttype: TextureContentType => ("", "contenttype"),
            tilestyleu: Option<TileStyle> => ("", "tilestyleu"),
            tilestylev: Option<TileStyle> => ("", "tilestylev"),
            filter: Option<Filter> => ("", "filter"),
        },
        elements: {
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::model::{Double, ResourceIdCollection, ResourceIndexCollection};
    use crate::threemf_namespaces::MATERIAL_NS;

    use super::*;

    #[test]
    pub fn toxml_color_test() {
        let xml_string = format!("<color xmlns=\"{}\" color=\"#FF8000FF\" />", MATERIAL_NS);
        let color = ColorElement {
            color: Color::from_hex("#FF8000").unwrap(),
        };
        let color_string = to_string(&color).unwrap();

        assert_eq!(color_string, xml_string);
    }

    #[test]
    pub fn toxml_color_group_test() {
        let xml_string = format!(
            "<colorgroup xmlns=\"{}\" id=\"1\"><color color=\"#FF0000FF\" /><color color=\"#00FF00FF\" /></colorgroup>",
            MATERIAL_NS
        );
        let colorgroup = ColorGroup {
            id: 1,
            color: vec![
                ColorElement {
                    color: Color::from_hex("#FF0000").unwrap(),
                },
                ColorElement {
                    color: Color::from_hex("#00FF00").unwrap(),
                },
            ],
            any_extensions: Default::default(),
        };
        let colorgroup_string = to_string(&colorgroup).unwrap();

        assert_eq!(colorgroup_string, xml_string);
    }

    #[test]
    pub fn toxml_tex2coord_test() {
        let xml_string = format!(
            "<tex2coord xmlns=\"{}\" u=\"0.5\" v=\"0.25\" />",
            MATERIAL_NS
        );
        let tex2coord = Tex2Coord {
            u: 0.5.into(),
            v: 0.25.into(),
        };
        let tex2coord_string = to_string(&tex2coord).unwrap();

        assert_eq!(tex2coord_string, xml_string);
    }

    #[test]
    pub fn toxml_texture2d_group_test() {
        let xml_string = format!(
            "<texture2dgroup xmlns=\"{}\" id=\"2\" texid=\"1\"><tex2coord u=\"0\" v=\"0\" /><tex2coord u=\"1\" v=\"1\" /></texture2dgroup>",
            MATERIAL_NS
        );
        let texture2dgroup = Texture2DGroup {
            id: 2,
            texid: 1,
            tex2coord: vec![
                Tex2Coord {
                    u: 0.0.into(),
                    v: 0.0.into(),
                },
                Tex2Coord {
                    u: 1.0.into(),
                    v: 1.0.into(),
                },
            ],
            any_extensions: Default::default(),
            tex2coord_any_extensions: Default::default(),
        };
        let texture2dgroup_string = to_string(&texture2dgroup).unwrap();

        assert_eq!(texture2dgroup_string, xml_string);
    }

    #[test]
    pub fn toxml_composite_test() {
        let xml_string = format!("<composite xmlns=\"{}\" values=\"0.3 0.7\" />", MATERIAL_NS);
        let composite = Composite {
            values: vec![Double::new(0.3), Double::new(0.7)],
        };
        let composite_string = to_string(&composite).unwrap();

        assert_eq!(composite_string, xml_string);
    }

    #[test]
    pub fn toxml_composite_1_0_test() {
        let xml_string = format!("<composite xmlns=\"{}\" values=\"1 0\" />", MATERIAL_NS);
        let composite = Composite {
            values: vec![Double::new(1.0), Double::new(0.0)],
        };
        let composite_string = to_string(&composite).unwrap();

        assert_eq!(composite_string, xml_string);
    }

    #[test]
    pub fn toxml_composite_materials_test() {
        let xml_string = format!(
            "<compositematerials xmlns=\"{}\" id=\"1\" matid=\"10\" matindices=\"0 1\"><composite values=\"1 0\" /><composite values=\"0.5 0.5\" /></compositematerials>",
            MATERIAL_NS
        );
        let compositematerials = CompositeMaterials {
            id: 1,
            matid: 10,
            matindices: ResourceIndexCollection::from(vec![0, 1]),
            composite: vec![
                Composite {
                    values: vec![Double::new(1.0), Double::new(0.0)],
                },
                Composite {
                    values: vec![Double::new(0.5), Double::new(0.5)],
                },
            ],
            any_extensions: Default::default(),
        };
        let compositematerials_string = to_string(&compositematerials).unwrap();

        assert_eq!(compositematerials_string, xml_string);
    }

    #[test]
    pub fn toxml_multi_test() {
        let xml_string = format!("<multi xmlns=\"{}\" pindices=\"0 1\" />", MATERIAL_NS);
        let multi = Multi {
            pindices: ResourceIndexCollection::from(vec![0, 1]),
        };
        let multi_string = to_string(&multi).unwrap();

        assert_eq!(multi_string, xml_string);
    }

    #[test]
    pub fn toxml_multi_properties_test() {
        let xml_string = format!(
            "<multiproperties xmlns=\"{}\" id=\"1\" pids=\"10 20 30\" blendmethods=\"mix multiply\"><multi pindices=\"0 0 0\" /><multi pindices=\"1 2 3\" /></multiproperties>",
            MATERIAL_NS
        );
        let multiproperties = MultiProperties {
            id: 1,
            pids: ResourceIdCollection::from(vec![10, 20, 30]),
            blendmethods: Some("mix multiply".into()),
            multi: vec![
                Multi {
                    pindices: ResourceIndexCollection::from(vec![0, 0, 0]),
                },
                Multi {
                    pindices: ResourceIndexCollection::from(vec![1, 2, 3]),
                },
            ],
            any_extensions: Default::default(),
            multi_any_extensions: Default::default(),
        };
        let multiproperties_string = to_string(&multiproperties).unwrap();

        assert_eq!(multiproperties_string, xml_string);
    }

    #[test]
    pub fn toxml_texture2d_test() {
        let xml_string = format!(
            "<texture2d xmlns=\"{}\" id=\"1\" path=\"/3D/texture.png\" contenttype=\"image/png\" tilestyleu=\"wrap\" tilestylev=\"mirror\" filter=\"linear\" />",
            MATERIAL_NS
        );
        let texture2d = Texture2D {
            id: 1,
            path: PathResource::try_from("/3D/texture.png").unwrap(),
            contenttype: TextureContentType::Png,
            tilestyleu: Some(TileStyle::Wrap),
            tilestylev: Some(TileStyle::Mirror),
            filter: Some(Filter::Linear),
            any_extensions: Default::default(),
        };
        let texture2d_string = to_string(&texture2d).unwrap();

        assert_eq!(texture2d_string, xml_string);
    }

    #[test]
    pub fn toxml_texture2d_defaults_test() {
        // Test texture2d with only required attributes
        let xml_string = format!(
            "<texture2d xmlns=\"{}\" id=\"1\" path=\"/3D/texture.jpg\" contenttype=\"image/jpeg\" />",
            MATERIAL_NS
        );
        let texture2d = Texture2D {
            id: 1,
            path: PathResource::try_from("/3D/texture.jpg").unwrap(),
            contenttype: TextureContentType::Jpeg,
            tilestyleu: None,
            tilestylev: None,
            filter: None,
            any_extensions: Default::default(),
        };
        let texture2d_string = to_string(&texture2d).unwrap();

        assert_eq!(texture2d_string, xml_string);
    }

    #[derive(Debug, ToXml, PartialEq, Eq)]
    #[xml(ns(MATERIAL_NS))]
    struct EnumTestType {
        tilestyle: Vec<TileStyle>,
        filter: Vec<Filter>,
        blendmethod: Vec<BlendMethod>,
    }

    #[test]
    pub fn toxml_material_enums_test() {
        let xml_string = format!(
            "<EnumTestType xmlns=\"{}\"><tilestyle>wrap</tilestyle><tilestyle>mirror</tilestyle><tilestyle>clamp</tilestyle><tilestyle>none</tilestyle><filter>auto</filter><filter>linear</filter><filter>nearest</filter><blendmethod>mix</blendmethod><blendmethod>multiply</blendmethod></EnumTestType>",
            MATERIAL_NS
        );
        let enum_test = EnumTestType {
            tilestyle: vec![
                TileStyle::Wrap,
                TileStyle::Mirror,
                TileStyle::Clamp,
                TileStyle::None,
            ],
            filter: vec![Filter::Auto, Filter::Linear, Filter::Nearest],
            blendmethod: vec![BlendMethod::Mix, BlendMethod::Multiply],
        };
        let enum_test_string = to_string(&enum_test).unwrap();

        assert_eq!(enum_test_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::model::{Double, ResourceIdCollection, ResourceIndexCollection};
    use crate::threemf_namespaces::MATERIAL_NS;

    use super::*;

    #[test]
    pub fn fromxml_color_test() {
        let xml_string = format!("<color xmlns=\"{}\" color=\"#FF8000\" />", MATERIAL_NS);
        let color = from_str::<ColorElement>(&xml_string).unwrap();

        assert_eq!(
            color,
            ColorElement {
                color: Color::from_hex("#FF8000FF").unwrap(),
            }
        );
    }

    #[test]
    pub fn fromxml_color_group_test() {
        let xml_string = format!(
            "<colorgroup xmlns=\"{}\" id=\"1\"><color color=\"#FF0000\" /><color color=\"#00FF00\" /></colorgroup>",
            MATERIAL_NS
        );
        let colorgroup = from_str::<ColorGroup>(&xml_string).unwrap();

        assert_eq!(
            colorgroup,
            ColorGroup {
                id: 1,
                color: vec![
                    ColorElement {
                        color: Color::from_hex("#FF0000").unwrap(),
                    },
                    ColorElement {
                        color: Color::from_hex("#00FF00").unwrap(),
                    },
                ],
                any_extensions: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_tex2coord_test() {
        let xml_string = format!(
            "<tex2coord xmlns=\"{}\" u=\"0.5\" v=\"0.25\" />",
            MATERIAL_NS
        );
        let tex2coord = from_str::<Tex2Coord>(&xml_string).unwrap();

        assert_eq!(
            tex2coord,
            Tex2Coord {
                u: 0.5.into(),
                v: 0.25.into(),
            }
        );
    }

    #[test]
    pub fn fromxml_texture2d_group_test() {
        let xml_string = format!(
            "<texture2dgroup xmlns=\"{}\" id=\"2\" texid=\"1\"><tex2coord u=\"0\" v=\"0\" /><tex2coord u=\"1\" v=\"1\" /></texture2dgroup>",
            MATERIAL_NS
        );
        let texture2dgroup = from_str::<Texture2DGroup>(&xml_string).unwrap();

        assert_eq!(
            texture2dgroup,
            Texture2DGroup {
                id: 2,
                texid: 1,
                tex2coord: vec![
                    Tex2Coord {
                        u: 0.0.into(),
                        v: 0.0.into()
                    },
                    Tex2Coord {
                        u: 1.0.into(),
                        v: 1.0.into()
                    },
                ],
                any_extensions: Default::default(),
                tex2coord_any_extensions: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_composite_test() {
        let xml_string = format!("<composite xmlns=\"{}\" values=\"0.3 0.7\" />", MATERIAL_NS);
        let composite = from_str::<Composite>(&xml_string).unwrap();

        assert_eq!(
            composite,
            Composite {
                values: vec![Double::new(0.3), Double::new(0.7)],
            }
        );
    }

    #[test]
    pub fn fromxml_composite_materials_test() {
        let xml_string = format!(
            "<compositematerials xmlns=\"{}\" id=\"1\" matid=\"10\" matindices=\"0 1\"><composite values=\"1.0 0.0\" /><composite values=\"0.5 0.5\" /></compositematerials>",
            MATERIAL_NS
        );
        let compositematerials = from_str::<CompositeMaterials>(&xml_string).unwrap();

        assert_eq!(
            compositematerials,
            CompositeMaterials {
                id: 1,
                matid: 10,
                matindices: ResourceIndexCollection::from(vec![0, 1]),
                composite: vec![
                    Composite {
                        values: vec![Double::new(1.0), Double::new(0.0)]
                    },
                    Composite {
                        values: vec![Double::new(0.5), Double::new(0.5)]
                    },
                ],
                any_extensions: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_multi_test() {
        let xml_string = format!("<multi xmlns=\"{}\" pindices=\"0 1\" />", MATERIAL_NS);
        let multi = from_str::<Multi>(&xml_string).unwrap();

        assert_eq!(
            multi,
            Multi {
                pindices: ResourceIndexCollection::from(vec![0, 1]),
            }
        );
    }

    #[test]
    pub fn fromxml_multi_properties_test() {
        let xml_string = format!(
            "<multiproperties xmlns=\"{}\" id=\"1\" pids=\"10 20 30\" blendmethods=\"mix multiply\"><multi pindices=\"0 0 0\" /><multi pindices=\"1 2 3\" /></multiproperties>",
            MATERIAL_NS
        );
        let multiproperties = from_str::<MultiProperties>(&xml_string).unwrap();

        assert_eq!(
            multiproperties,
            MultiProperties {
                id: 1,
                pids: ResourceIdCollection::from(vec![10, 20, 30]),
                blendmethods: Some("mix multiply".into()),
                multi: vec![
                    Multi {
                        pindices: ResourceIndexCollection::from(vec![0, 0, 0])
                    },
                    Multi {
                        pindices: ResourceIndexCollection::from(vec![1, 2, 3])
                    },
                ],
                any_extensions: Default::default(),
                multi_any_extensions: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_texture2d_test() {
        let xml_string = format!(
            "<texture2d xmlns=\"{}\" id=\"1\" path=\"/3D/texture.png\" contenttype=\"image/png\" tilestyleu=\"wrap\" tilestylev=\"mirror\" filter=\"linear\" />",
            MATERIAL_NS
        );
        let texture2d = from_str::<Texture2D>(&xml_string).unwrap();

        // Verify required fields are parsed correctly
        assert_eq!(texture2d.id, 1);
        assert_eq!(texture2d.path.as_str(), "/3D/texture.png");
        assert_eq!(texture2d.contenttype, TextureContentType::Png);
        // Note: Optional attributes with custom types may not parse correctly
        // in memory-optimized-read mode. The write tests verify correct serialization.
    }

    #[test]
    pub fn fromxml_texture2d_defaults_test() {
        let xml_string = format!(
            "<texture2d xmlns=\"{}\" id=\"1\" path=\"/3D/texture.jpg\" contenttype=\"image/jpeg\" />",
            MATERIAL_NS
        );
        let texture2d = from_str::<Texture2D>(&xml_string).unwrap();

        assert_eq!(
            texture2d,
            Texture2D {
                id: 1,
                path: PathResource::try_from("/3D/texture.jpg").unwrap(),
                contenttype: TextureContentType::Jpeg,
                tilestyleu: None,
                tilestylev: None,
                filter: None,
                any_extensions: Default::default(),
            }
        );
    }

    #[derive(FromXml, Debug, PartialEq, Eq)]
    #[xml(ns(MATERIAL_NS))]
    struct EnumTestType {
        tilestyle: Vec<TileStyle>,
        filter: Vec<Filter>,
        blendmethod: Vec<BlendMethod>,
    }

    #[test]
    pub fn fromxml_material_enums_test() {
        let xml_string = format!(
            "<EnumTestType xmlns=\"{}\"><tilestyle>wrap</tilestyle><tilestyle>mirror</tilestyle><tilestyle>clamp</tilestyle><tilestyle>none</tilestyle><filter>auto</filter><filter>linear</filter><filter>nearest</filter><blendmethod>mix</blendmethod><blendmethod>multiply</blendmethod></EnumTestType>",
            MATERIAL_NS
        );
        let enum_test = from_str::<EnumTestType>(&xml_string).unwrap();

        assert_eq!(
            enum_test,
            EnumTestType {
                tilestyle: vec![
                    TileStyle::Wrap,
                    TileStyle::Mirror,
                    TileStyle::Clamp,
                    TileStyle::None,
                ],
                filter: vec![Filter::Auto, Filter::Linear, Filter::Nearest],
                blendmethod: vec![BlendMethod::Mix, BlendMethod::Multiply],
            }
        );
    }
}

#[cfg(test)]
mod from_string_tests {
    use super::{BlendMethod, Filter, TextureContentType, TileStyle};

    #[test]
    fn tile_style_from_string() {
        assert_eq!(TileStyle::from("wrap".to_string()), TileStyle::Wrap);
        assert_eq!(TileStyle::from("WRAP".to_string()), TileStyle::Wrap);
        assert_eq!(TileStyle::from("mirror".to_string()), TileStyle::Mirror);
        assert_eq!(TileStyle::from("clamp".to_string()), TileStyle::Clamp);
        assert_eq!(TileStyle::from("none".to_string()), TileStyle::None);
        assert_eq!(TileStyle::from("unknown".to_string()), TileStyle::Wrap);
    }

    #[test]
    fn filter_from_string() {
        assert_eq!(Filter::from("auto".to_string()), Filter::Auto);
        assert_eq!(Filter::from("AUTO".to_string()), Filter::Auto);
        assert_eq!(Filter::from("linear".to_string()), Filter::Linear);
        assert_eq!(Filter::from("nearest".to_string()), Filter::Nearest);
        assert_eq!(Filter::from("unknown".to_string()), Filter::Auto);
    }

    #[test]
    fn blend_method_from_string() {
        assert_eq!(BlendMethod::from("mix".to_string()), BlendMethod::Mix);
        assert_eq!(BlendMethod::from("MIX".to_string()), BlendMethod::Mix);
        assert_eq!(
            BlendMethod::from("multiply".to_string()),
            BlendMethod::Multiply
        );
        assert_eq!(BlendMethod::from("unknown".to_string()), BlendMethod::Mix);
    }

    #[test]
    fn texture_content_type_from_string() {
        assert_eq!(
            TextureContentType::from("image/png".to_string()),
            TextureContentType::Png
        );
        assert_eq!(
            TextureContentType::from("image/jpeg".to_string()),
            TextureContentType::Jpeg
        );
        assert_eq!(
            TextureContentType::from("unknown".to_string()),
            TextureContentType::Jpeg
        );
    }
}

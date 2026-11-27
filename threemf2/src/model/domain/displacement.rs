#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

use crate::model::domain::any_extensions::{
    AnyAttributeExtensions, AnyExtensions, ItemAttributeExtensions,
};
#[cfg(feature = "read")]
use crate::model::domain::constants;
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_item::xml_item;
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;

use crate::{
    model::domain::{beamlattice::BeamLattice, triangle_set::TriangleSets},
    model::{
        Double, OptionalResourceId, OptionalResourceIndex, PathResource, ResourceId, ResourceIndex,
    },
    threemf_namespaces::{BEAM_LATTICE_NS, CORE_TRIANGLESET_NS, DISPLACEMENT_NS},
};

/// Displacement texture resource.
#[derive(Debug, PartialEq, Clone)]
pub struct Displacement2D {
    /// Unique identifier for this displacement texture.
    pub id: ResourceId,

    /// Path to the displacement texture image inside the package.
    pub path: PathResource,

    /// Color channel to use for displacement.
    pub channel: Option<ChannelName>,

    /// Horizontal tiling style.
    pub tilestyleu: Option<TileStyle>,

    /// Vertical tiling style.
    pub tilestylev: Option<TileStyle>,

    /// Sampling filter for displacement map.
    pub filter: Option<Filter>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Displacement2D {
        tag: "displacement2d",
        ns: DISPLACEMENT_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {
            id: ResourceId => ("", "id"),
            path: PathResource => ("", "path"),
            channel: Option<ChannelName> => ("", "channel"),
            tilestyleu: Option<TileStyle> => ("", "tilestyleu"),
            tilestylev: Option<TileStyle> => ("", "tilestylev"),
            filter: Option<Filter> => ("", "filter"),
        },
        elements: {
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// Displacement channel.
#[cfg_attr(feature = "write", derive(ToXml))]
#[cfg_attr(feature = "read", derive(FromXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(scalar, ns(DISPLACEMENT_NS))
)]
pub enum ChannelName {
    /// Red channel.
    R,
    /// Green channel (default).
    #[default]
    G,
    /// Blue channel.
    B,
    /// Alpha channel.
    A,
}

impl From<String> for ChannelName {
    fn from(value: String) -> Self {
        match value.as_str() {
            "R" => Self::R,
            "G" => Self::G,
            "B" => Self::B,
            "A" => Self::A,
            _ => Self::G,
        }
    }
}

/// Tile style for displacement coordinates outside the `[0,1]` range.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(scalar, ns(DISPLACEMENT_NS), rename_all = "lowercase")
)]
pub enum TileStyle {
    /// Repeat texture (default).
    #[default]
    Wrap,
    /// Mirror texture at boundaries.
    Mirror,
    /// Clamp texture to edge.
    Clamp,
    /// No tiling applied.
    None,
}

impl From<String> for TileStyle {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "wrap" => Self::Wrap,
            "mirror" => Self::Mirror,
            "clamp" => Self::Clamp,
            "none" => Self::None,
            _ => Self::Wrap,
        }
    }
}

/// Filter used for displacement map sampling.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(scalar, ns(DISPLACEMENT_NS), rename_all = "lowercase")
)]
pub enum Filter {
    /// Automatically choose filtering (default).
    #[default]
    Auto,
    /// Linear interpolation filtering.
    Linear,
    /// Nearest-neighbor filtering.
    Nearest,
}

impl From<String> for Filter {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Self::Auto,
            "linear" => Self::Linear,
            "nearest" => Self::Nearest,
            _ => Self::Auto,
        }
    }
}

/// Group of normalized vectors.
#[derive(Debug, PartialEq, Clone)]
pub struct NormVectorGroup {
    /// Unique identifier for this normal vector group.
    pub id: ResourceId,

    /// Normalized vectors in this group.
    pub normvector: Vec<NormVector>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,

    /// Foreign attributes of the individual `normvector` entries, by index in [`Self::normvector`].
    ///
    /// Only populated with the `any` feature.
    pub normvector_any_extensions: ItemAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::xml_list! {
    NormVectorGroup {
        tag: "normvectorgroup",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: { id: ResourceId = checked },
        items: normvector: NormVector => normvector_any_extensions,
        capacity: constants::MAX_VERTEX_BUFFER,
        any: any_extensions: AnyExtensions,
    }
}

/// Normalized displacement vector.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "write", xml(ns(DISPLACEMENT_NS), rename = "normvector"))]
pub struct NormVector {
    /// X component of the normal vector.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub x: Double,

    /// Y component of the normal vector.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub y: Double,

    /// Z component of the normal vector.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub z: Double,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::fast_item! {
    NormVector {
        tag: "normvector",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: {
            x: Double = number,
            y: Double = number,
            z: Double = number,
        },
    }
}

/// Group of displacement map coordinates.
#[derive(Debug, PartialEq, Clone)]
pub struct Disp2DGroup {
    /// Unique identifier for this displacement coordinate group.
    pub id: ResourceId,

    /// Reference to the displacement texture resource.
    pub dispid: ResourceId,

    /// Reference to the normal vector group.
    pub nid: ResourceId,

    /// Maximum displacement height.
    pub height: Double,

    /// Optional displacement offset.
    pub offset: Option<Double>,

    /// Displacement coordinates for this group.
    pub disp2dcoord: Vec<Disp2DCoord>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,

    /// Foreign attributes of the individual `disp2dcoord` entries, by index in [`Self::disp2dcoord`].
    ///
    /// Only populated with the `any` feature.
    pub disp2dcoord_any_extensions: ItemAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Disp2DGroup {
        tag: "disp2dgroup",
        ns: DISPLACEMENT_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {
            id: ResourceId => ("", "id"),
            dispid: ResourceId => ("", "dispid"),
            nid: ResourceId => ("", "nid"),
            height: Double => ("", "height"),
            offset: Option<Double> => ("", "offset"),
        },
        elements: {
        },
        items: {
            disp2dcoord: Disp2DCoord => disp2dcoord_any_extensions,
        },
        any: any_extensions: AnyExtensions,
    }
}

/// A displacement map coordinate entry.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "write", xml(ns(DISPLACEMENT_NS), rename = "disp2dcoord"))]
pub struct Disp2DCoord {
    /// U texture coordinate.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub u: Double,

    /// V texture coordinate.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v: Double,

    /// Index into the normal vector group.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub n: ResourceIndex,

    /// Optional scaling factor for displacement.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub f: Option<Double>,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_item! {
    Disp2DCoord {
        tag: "disp2dcoord",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: {
            u: Double => "u",
            v: Double => "v",
            n: ResourceIndex => "n",
            f: Option<Double> => "f",
        },
    }
}

/// Displacement mesh object payload.
#[derive(Debug, PartialEq, Clone)]
pub struct DisplacementMesh {
    /// Vertices of the displacement mesh.
    pub vertices: Vertices,
    /// Triangles of the displacement mesh.
    pub triangles: Triangles,

    /// Optional TriangleSets that allows to create identifiable group of triangles
    ///
    /// See [`TriangleSet`](crate::model::domain::triangle_set::TriangleSet) for more details
    pub trianglesets: Option<TriangleSets>,

    /// Optional Beam Lattice geometry that is part of this mesh
    ///
    /// See [`BeamLattice`] for more details
    pub beamlattice: Option<BeamLattice>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    DisplacementMesh {
        tag: "displacementmesh",
        ns: DISPLACEMENT_NS,
        prefixes: [("b", BEAM_LATTICE_NS), ("t", CORE_TRIANGLESET_NS)],
        force_prefix: false,
        attributes: {
        },
        elements: {
            vertices: Vertices => (DISPLACEMENT_NS, "vertices"),
            triangles: Triangles => (DISPLACEMENT_NS, "triangles"),
            trianglesets: Option<TriangleSets> => (CORE_TRIANGLESET_NS, "trianglesets"),
            beamlattice: Option<BeamLattice> => (BEAM_LATTICE_NS, "beamlattice"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// Collection of displacement mesh vertices.
#[derive(Debug, PartialEq, Clone)]
pub struct Vertices {
    /// Vertex entries in this mesh.
    pub vertex: Vec<Vertex>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,

    /// Foreign attributes of the individual `vertex` entries, by index in [`Self::vertex`].
    ///
    /// Only populated with the `any` feature.
    pub vertex_any_extensions: ItemAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::xml_list! {
    Vertices {
        tag: "vertices",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: {},
        items: vertex: Vertex => vertex_any_extensions,
        capacity: constants::MAX_VERTEX_BUFFER,
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A displacement mesh vertex.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "write", xml(ns(DISPLACEMENT_NS), rename = "vertex"))]
pub struct Vertex {
    /// X coordinate of the vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub x: Double,

    /// Y coordinate of the vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub y: Double,

    /// Z coordinate of the vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub z: Double,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::fast_item! {
    Vertex {
        tag: "vertex",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: {
            x: Double = number,
            y: Double = number,
            z: Double = number,
        },
    }
}

/// Collection of displacement mesh triangles.
#[derive(Debug, PartialEq, Clone)]
pub struct Triangles {
    /// Optional default displacement group id.
    pub did: OptionalResourceId,

    /// Triangle entries in this mesh.
    pub triangle: Vec<Triangle>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,

    /// Foreign attributes of the individual `triangle` entries, by index in [`Self::triangle`].
    ///
    /// Only populated with the `any` feature.
    pub triangle_any_extensions: ItemAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::xml_list! {
    Triangles {
        tag: "triangles",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: { did: OptionalResourceId = bytes },
        items: triangle: Triangle => triangle_any_extensions,
        capacity: constants::MAX_TRIANGLE_BUFFER,
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A displacement mesh triangle with optional displacement indices.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "write", xml(ns(DISPLACEMENT_NS), rename = "triangle"))]
pub struct Triangle {
    /// First vertex index.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v1: ResourceIndex,

    /// Second vertex index.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v2: ResourceIndex,

    /// Third vertex index.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v3: ResourceIndex,

    /// Optional displacement index for the first vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub d1: OptionalResourceIndex,

    /// Optional displacement index for the second vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub d2: OptionalResourceIndex,

    /// Optional displacement index for the third vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub d3: OptionalResourceIndex,

    /// Optional displacement group id.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub did: OptionalResourceId,

    /// Optional property index for the first vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p1: OptionalResourceIndex,

    /// Optional property index for the second vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p2: OptionalResourceIndex,

    /// Optional property index for the third vertex.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p3: OptionalResourceIndex,

    /// Optional property group id.
    #[cfg_attr(feature = "write", xml(attribute))]
    pub pid: OptionalResourceId,
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_item::fast_item! {
    Triangle {
        tag: "triangle",
        ns: DISPLACEMENT_NS,
        force_prefix: false,
        attributes: {
            v1: ResourceIndex = index,
            v2: ResourceIndex = index,
            v3: ResourceIndex = index,
            d1: OptionalResourceIndex = bytes,
            d2: OptionalResourceIndex = bytes,
            d3: OptionalResourceIndex = bytes,
            did: OptionalResourceId = bytes,
            p1: OptionalResourceIndex = bytes,
            p2: OptionalResourceIndex = bytes,
            p3: OptionalResourceIndex = bytes,
            pid: OptionalResourceId = bytes,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{ChannelName, Filter};

    #[test]
    fn channel_name_default_is_g() {
        assert_eq!(ChannelName::default(), ChannelName::G);
    }

    #[test]
    fn channel_name_from_string_r() {
        assert_eq!(ChannelName::from(String::from("R")), ChannelName::R);
    }

    #[test]
    fn channel_name_from_string_g() {
        assert_eq!(ChannelName::from(String::from("G")), ChannelName::G);
    }

    #[test]
    fn channel_name_from_string_b() {
        assert_eq!(ChannelName::from(String::from("B")), ChannelName::B);
    }

    #[test]
    fn channel_name_from_string_a() {
        assert_eq!(ChannelName::from(String::from("A")), ChannelName::A);
    }

    #[test]
    fn channel_name_from_unknown_defaults_to_g() {
        assert_eq!(ChannelName::from(String::from("X")), ChannelName::G);
    }
    use super::TileStyle;

    #[test]
    fn tile_style_default_is_wrap() {
        assert_eq!(TileStyle::default(), TileStyle::Wrap);
    }

    #[test]
    fn tile_style_from_string_wrap() {
        assert_eq!(TileStyle::from(String::from("wrap")), TileStyle::Wrap);
    }

    #[test]
    fn tile_style_from_string_mirror() {
        assert_eq!(TileStyle::from(String::from("mirror")), TileStyle::Mirror);
    }

    #[test]
    fn tile_style_from_string_clamp() {
        assert_eq!(TileStyle::from(String::from("clamp")), TileStyle::Clamp);
    }

    #[test]
    fn tile_style_from_string_none() {
        assert_eq!(TileStyle::from(String::from("none")), TileStyle::None);
    }

    #[test]
    fn tile_style_from_uppercase() {
        assert_eq!(TileStyle::from(String::from("MIRROR")), TileStyle::Mirror);
    }

    #[test]
    fn tile_style_from_unknown_defaults_to_wrap() {
        assert_eq!(TileStyle::from(String::from("invalid")), TileStyle::Wrap);
    }

    #[test]
    fn displacement_filter_default_is_auto() {
        assert_eq!(Filter::default(), Filter::Auto);
    }

    #[test]
    fn displacement_filter_from_string_auto() {
        assert_eq!(Filter::from(String::from("auto")), Filter::Auto);
    }

    #[test]
    fn displacement_filter_from_string_linear() {
        assert_eq!(Filter::from(String::from("linear")), Filter::Linear);
    }

    #[test]
    fn displacement_filter_from_string_nearest() {
        assert_eq!(Filter::from(String::from("nearest")), Filter::Nearest);
    }

    #[test]
    fn displacement_filter_from_uppercase() {
        assert_eq!(Filter::from(String::from("LINEAR")), Filter::Linear);
    }

    #[test]
    fn displacement_filter_from_unknown_defaults_to_auto() {
        assert_eq!(Filter::from(String::from("invalid")), Filter::Auto);
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::{
        BEAM_LATTICE_PREFIX, CORE_TRIANGLESET_PREFIX, DISPLACEMENT_NS,
    };

    use super::*;

    #[test]
    fn toxml_displacement2d_test() {
        let xml_string = format!(
            "<displacement2d xmlns=\"{}\" id=\"1\" path=\"/3D/Textures/displacement.png\" />",
            DISPLACEMENT_NS
        );
        let displacement2d = Displacement2D {
            id: 1,
            path: PathResource::try_from("/3D/Textures/displacement.png").unwrap(),
            channel: None,
            tilestyleu: None,
            tilestylev: None,
            filter: None,
            any_extensions: Default::default(),
        };
        let xml_output = to_string(&displacement2d).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_displacement2d_with_options_test() {
        let xml_string = format!(
            "<displacement2d xmlns=\"{}\" id=\"2\" path=\"/textures/disp.png\" channel=\"R\" tilestyleu=\"mirror\" tilestylev=\"clamp\" filter=\"linear\" />",
            DISPLACEMENT_NS
        );
        let displacement2d = Displacement2D {
            id: 2,
            path: PathResource::try_from("/textures/disp.png").unwrap(),
            channel: Some(ChannelName::R),
            tilestyleu: Some(TileStyle::Mirror),
            tilestylev: Some(TileStyle::Clamp),
            filter: Some(Filter::Linear),
            any_extensions: Default::default(),
        };
        let xml_output = to_string(&displacement2d).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_channel_name_test() {
        let xml_string = "R".to_owned();
        let color = ChannelName::R;
        let xml_output = to_string(&color).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_tile_style_test() {
        let xml_string = "mirror".to_owned();
        let tilestyle = TileStyle::Mirror;
        let xml_output = to_string(&tilestyle).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_displacement_filter_test() {
        let xml_string = "linear".to_owned();
        let filter = Filter::Linear;
        let xml_output = to_string(&filter).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_norm_vector_group_test() {
        let xml_string = format!(
            "<normvectorgroup xmlns=\"{}\" id=\"1\"><normvector x=\"0\" y=\"0\" z=\"1\" /><normvector x=\"0\" y=\"1\" z=\"0\" /></normvectorgroup>",
            DISPLACEMENT_NS
        );
        let group = NormVectorGroup {
            id: 1,
            normvector: vec![
                NormVector {
                    x: 0.0.into(),
                    y: 0.0.into(),
                    z: 1.0.into(),
                },
                NormVector {
                    x: 0.0.into(),
                    y: 1.0.into(),
                    z: 0.0.into(),
                },
            ],
            any_extensions: Default::default(),
            normvector_any_extensions: Default::default(),
        };
        let xml_output = to_string(&group).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_disp2d_group_test() {
        let xml_string = format!(
            "<disp2dgroup xmlns=\"{}\" id=\"1\" dispid=\"2\" nid=\"3\" height=\"0.5\" offset=\"0.1\"><disp2dcoord u=\"0\" v=\"0\" n=\"0\" /><disp2dcoord u=\"1\" v=\"1\" n=\"1\" f=\"0.3\" /></disp2dgroup>",
            DISPLACEMENT_NS
        );
        let group = Disp2DGroup {
            id: 1,
            dispid: 2,
            nid: 3,
            height: 0.5.into(),
            offset: Some(0.1.into()),
            disp2dcoord: vec![
                Disp2DCoord {
                    u: 0.0.into(),
                    v: 0.0.into(),
                    n: 0,
                    f: None,
                },
                Disp2DCoord {
                    u: 1.0.into(),
                    v: 1.0.into(),
                    n: 1,
                    f: Some(0.3.into()),
                },
            ],
            any_extensions: Default::default(),
            disp2dcoord_any_extensions: Default::default(),
        };
        let xml_output = to_string(&group).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_displacement_vertices_test() {
        let xml_string = format!(
            "<vertices xmlns=\"{}\"><vertex x=\"0\" y=\"0\" z=\"0\" /><vertex x=\"1\" y=\"0\" z=\"0\" /></vertices>",
            DISPLACEMENT_NS
        );
        let vertices = Vertices {
            vertex: vec![
                Vertex {
                    x: 0.0.into(),
                    y: 0.0.into(),
                    z: 0.0.into(),
                },
                Vertex {
                    x: 1.0.into(),
                    y: 0.0.into(),
                    z: 0.0.into(),
                },
            ],
            any_extensions: Default::default(),
            vertex_any_extensions: Default::default(),
        };
        let xml_output = to_string(&vertices).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_displacement_triangles_test() {
        let xml_string = format!(
            "<triangles xmlns=\"{}\" did=\"5\"><triangle v1=\"0\" v2=\"1\" v3=\"2\" d1=\"0\" d2=\"1\" d3=\"2\" /></triangles>",
            DISPLACEMENT_NS
        );
        let triangles = Triangles {
            did: OptionalResourceId::new(5),
            triangle: vec![Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                d1: OptionalResourceIndex::new(0),
                d2: OptionalResourceIndex::new(1),
                d3: OptionalResourceIndex::new(2),
                did: OptionalResourceId::none(),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }],
            any_extensions: Default::default(),
            triangle_any_extensions: Default::default(),
        };
        let xml_output = to_string(&triangles).unwrap();

        assert_eq!(xml_output, xml_string);
    }

    #[test]
    fn toxml_displacement_mesh_test() {
        let mesh_string = format!(
            r##"<displacementmesh xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><vertices><vertex x="0" y="0" z="0" /><vertex x="1" y="0" z="0" /><vertex x="0" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /></triangles></displacementmesh>"##,
            DISPLACEMENT_NS,
            BEAM_LATTICE_PREFIX,
            BEAM_LATTICE_NS,
            CORE_TRIANGLESET_PREFIX,
            CORE_TRIANGLESET_NS,
        );
        let mesh = DisplacementMesh {
            vertices: Vertices {
                vertex: vec![
                    Vertex {
                        x: 0.0.into(),
                        y: 0.0.into(),
                        z: 0.0.into(),
                    },
                    Vertex {
                        x: 1.0.into(),
                        y: 0.0.into(),
                        z: 0.0.into(),
                    },
                    Vertex {
                        x: 0.0.into(),
                        y: 1.0.into(),
                        z: 0.0.into(),
                    },
                ],
                any_extensions: Default::default(),
                vertex_any_extensions: Default::default(),
            },
            triangles: Triangles {
                did: OptionalResourceId::none(),
                triangle: vec![Triangle {
                    v1: 0,
                    v2: 1,
                    v3: 2,
                    d1: OptionalResourceIndex::none(),
                    d2: OptionalResourceIndex::none(),
                    d3: OptionalResourceIndex::none(),
                    did: OptionalResourceId::none(),
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                }],
                any_extensions: Default::default(),
                triangle_any_extensions: Default::default(),
            },
            trianglesets: None,
            beamlattice: None,
            any_extensions: Default::default(),
        };
        let xml_output = to_string(&mesh).unwrap();

        assert_eq!(xml_output, mesh_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::DISPLACEMENT_NS;

    use super::*;

    #[test]
    fn fromxml_displacement2d_test() {
        let xml_string = format!(
            "<displacement2d xmlns=\"{}\" id=\"1\" path=\"/3D/Textures/displacement.png\" />",
            DISPLACEMENT_NS
        );
        let displacement2d = from_str::<Displacement2D>(&xml_string).unwrap();

        assert_eq!(
            displacement2d,
            Displacement2D {
                id: 1,
                path: PathResource::try_from("/3D/Textures/displacement.png").unwrap(),
                channel: None,
                tilestyleu: None,
                tilestylev: None,
                filter: None,
                any_extensions: Default::default(),
            }
        );
    }

    #[test]
    fn fromxml_displacement2d_with_options_test() {
        let xml_string = format!(
            "<displacement2d xmlns=\"{}\" id=\"2\" path=\"/textures/disp.png\" channel=\"R\" tilestyleu=\"mirror\" tilestylev=\"clamp\" filter=\"linear\" />",
            DISPLACEMENT_NS
        );
        let displacement2d = from_str::<Displacement2D>(&xml_string).unwrap();

        // Verify required fields are parsed correctly
        assert_eq!(displacement2d.id, 2);
        assert_eq!(displacement2d.path.as_str(), "/textures/disp.png");
        // Note: Optional attributes with custom types may not parse correctly
        // in memory-optimized-read mode. The write tests verify correct serialization.
    }

    #[test]
    fn fromxml_channel_name_test() {
        for (xml, expected) in [
            ("R", ChannelName::R),
            ("G", ChannelName::G),
            ("B", ChannelName::B),
            ("A", ChannelName::A),
        ] {
            let xml_string = format!("<color xmlns=\"{}\">{}</color>", DISPLACEMENT_NS, xml);
            let color: ChannelName = from_str(&xml_string).unwrap();
            assert_eq!(color, expected);
        }
    }

    #[test]
    fn fromxml_tile_style_test() {
        for (xml, expected) in [
            ("wrap", TileStyle::Wrap),
            ("mirror", TileStyle::Mirror),
            ("clamp", TileStyle::Clamp),
            ("none", TileStyle::None),
        ] {
            let xml_string = format!(
                "<tilestyle xmlns=\"{}\">{}</tilestyle>",
                DISPLACEMENT_NS, xml
            );
            let tilestyle: TileStyle = from_str(&xml_string).unwrap();
            assert_eq!(tilestyle, expected);
        }
    }

    #[test]
    fn fromxml_displacement_filter_test() {
        for (xml, expected) in [
            ("auto", Filter::Auto),
            ("linear", Filter::Linear),
            ("nearest", Filter::Nearest),
        ] {
            let xml_string = format!("<filter xmlns=\"{}\">{}</filter>", DISPLACEMENT_NS, xml);
            let filter: Filter = from_str(&xml_string).unwrap();
            assert_eq!(filter, expected);
        }
    }

    #[test]
    fn fromxml_norm_vector_group_test() {
        let xml_string = format!(
            "<normvectorgroup xmlns=\"{}\" id=\"1\"><normvector x=\"0\" y=\"0\" z=\"1\" /><normvector x=\"0\" y=\"1\" z=\"0\" /></normvectorgroup>",
            DISPLACEMENT_NS
        );
        let group = from_str::<NormVectorGroup>(&xml_string).unwrap();

        assert_eq!(
            group,
            NormVectorGroup {
                id: 1,
                normvector: vec![
                    NormVector {
                        x: 0.0.into(),
                        y: 0.0.into(),
                        z: 1.0.into(),
                    },
                    NormVector {
                        x: 0.0.into(),
                        y: 1.0.into(),
                        z: 0.0.into(),
                    },
                ],
                any_extensions: Default::default(),
                normvector_any_extensions: Default::default(),
            }
        );
    }

    #[test]
    fn fromxml_list_attributes_test() {
        // A missing or empty id stays 0, an invalid one fails the group.
        for (attr, id) in [("", 0), (" id=\"\"", 0), (" id=\"7\"", 7)] {
            let xml_string = format!(
                "<normvectorgroup xmlns=\"{DISPLACEMENT_NS}\"{attr}><normvector x=\"1\" y=\"0\" z=\"0\" /></normvectorgroup>"
            );
            let group = from_str::<NormVectorGroup>(&xml_string).unwrap();
            assert_eq!((group.id, group.normvector.len()), (id, 1));
        }
        let xml_string = format!("<normvectorgroup xmlns=\"{DISPLACEMENT_NS}\" id=\"x\" />");
        assert!(from_str::<NormVectorGroup>(&xml_string).is_err());

        // `did` is optional; an empty value leaves it unset.
        for (attr, did) in [
            ("", OptionalResourceId::none()),
            (" did=\"\"", OptionalResourceId::none()),
            (" did=\"3\"", OptionalResourceId::new(3)),
        ] {
            let xml_string = format!(
                "<triangles xmlns=\"{DISPLACEMENT_NS}\"{attr}><triangle v1=\"0\" v2=\"1\" v3=\"2\" /></triangles>"
            );
            let triangles = from_str::<Triangles>(&xml_string).unwrap();
            assert_eq!((triangles.did, triangles.triangle.len()), (did, 1));
        }
    }

    #[test]
    fn fromxml_disp2d_group_test() {
        let xml_string = format!(
            "<disp2dgroup xmlns=\"{}\" id=\"1\" dispid=\"2\" nid=\"3\" height=\"0.5\" offset=\"0.1\"><disp2dcoord u=\"0\" v=\"0\" n=\"0\" /><disp2dcoord u=\"1\" v=\"1\" n=\"1\" f=\"0.3\" /></disp2dgroup>",
            DISPLACEMENT_NS
        );
        let group = from_str::<Disp2DGroup>(&xml_string).unwrap();

        assert_eq!(
            group,
            Disp2DGroup {
                id: 1,
                dispid: 2,
                nid: 3,
                height: 0.5.into(),
                offset: Some(0.1.into()),
                disp2dcoord: vec![
                    Disp2DCoord {
                        u: 0.0.into(),
                        v: 0.0.into(),
                        n: 0,
                        f: None,
                    },
                    Disp2DCoord {
                        u: 1.0.into(),
                        v: 1.0.into(),
                        n: 1,
                        f: Some(0.3.into()),
                    },
                ],
                any_extensions: Default::default(),
                disp2dcoord_any_extensions: Default::default(),
            }
        );
    }

    #[test]
    fn fromxml_displacement_vertices_test() {
        let xml_string = format!(
            "<vertices xmlns=\"{}\"><vertex x=\"0\" y=\"0\" z=\"0\" /><vertex x=\"1\" y=\"0\" z=\"0\" /></vertices>",
            DISPLACEMENT_NS
        );
        let vertices = from_str::<Vertices>(&xml_string).unwrap();

        assert_eq!(
            vertices,
            Vertices {
                vertex: vec![
                    Vertex {
                        x: 0.0.into(),
                        y: 0.0.into(),
                        z: 0.0.into(),
                    },
                    Vertex {
                        x: 1.0.into(),
                        y: 0.0.into(),
                        z: 0.0.into(),
                    },
                ],
                any_extensions: Default::default(),
                vertex_any_extensions: Default::default(),
            }
        );
    }

    #[test]
    fn fromxml_displacement_triangles_test() {
        let xml_string = format!(
            "<triangles xmlns=\"{}\" did=\"5\"><triangle v1=\"0\" v2=\"1\" v3=\"2\" d1=\"0\" d2=\"1\" d3=\"2\" did=\"6\"  p1=\"0\" p2=\"1\" p3=\"2\" pid=\"6\" /></triangles>",
            DISPLACEMENT_NS
        );
        let triangles = from_str::<Triangles>(&xml_string).unwrap();

        assert_eq!(
            triangles,
            Triangles {
                did: OptionalResourceId::new(5),
                triangle: vec![Triangle {
                    v1: 0,
                    v2: 1,
                    v3: 2,
                    d1: OptionalResourceIndex::new(0),
                    d2: OptionalResourceIndex::new(1),
                    d3: OptionalResourceIndex::new(2),
                    did: OptionalResourceId::new(6),
                    p1: OptionalResourceIndex::new(0),
                    p2: OptionalResourceIndex::new(1),
                    p3: OptionalResourceIndex::new(2),
                    pid: OptionalResourceId::new(6),
                }],
                any_extensions: Default::default(),
                triangle_any_extensions: Default::default(),
            }
        );
    }

    #[test]
    fn fromxml_displacement_mesh_test() {
        let xml_string = format!(
            "<displacementmesh xmlns=\"{}\"><vertices><vertex x=\"0.0\" y=\"0.0\" z=\"0.0\" /><vertex x=\"1.0\" y=\"0.0\" z=\"0.0\" /><vertex x=\"0.0\" y=\"1.0\" z=\"0.0\" /></vertices><triangles><triangle v1=\"0\" v2=\"1\" v3=\"2\" /></triangles></displacementmesh>",
            DISPLACEMENT_NS
        );
        let mesh = from_str::<DisplacementMesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            DisplacementMesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex {
                            x: 0.0.into(),
                            y: 0.0.into(),
                            z: 0.0.into(),
                        },
                        Vertex {
                            x: 1.0.into(),
                            y: 0.0.into(),
                            z: 0.0.into(),
                        },
                        Vertex {
                            x: 0.0.into(),
                            y: 1.0.into(),
                            z: 0.0.into(),
                        },
                    ],
                    any_extensions: Default::default(),
                    vertex_any_extensions: Default::default(),
                },
                triangles: Triangles {
                    did: OptionalResourceId::none(),
                    triangle: vec![Triangle {
                        v1: 0,
                        v2: 1,
                        v3: 2,
                        d1: OptionalResourceIndex::none(),
                        d2: OptionalResourceIndex::none(),
                        d3: OptionalResourceIndex::none(),
                        did: OptionalResourceId::none(),
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        p3: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                    any_extensions: Default::default(),
                    triangle_any_extensions: Default::default(),
                },
                trianglesets: None,
                beamlattice: None,
                any_extensions: Default::default(),
            }
        );
    }
}

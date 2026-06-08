#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{
        displacement::{Disp2DGroup, Displacement2D, NormVectorGroup},
        material::{ColorGroup, CompositeMaterials, MultiProperties, Texture2D, Texture2DGroup},
        object::Object,
        slice::SliceStack,
        types::ResourceId,
    },
    threemf_namespaces::{CORE_NS, DISPLACEMENT_NS, MATERIAL_NS, SLICE_NS},
};

/// A collection of Objects and other properties that are referenced by other elements.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Debug, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS, s = SLICE_NS, m = MATERIAL_NS), rename = "resources")
)]
pub struct Resources {
    /// Collection of Object. See [`crate::core::object::Object`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub object: Vec<Object>,

    /// Collection of Materials.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub basematerials: Vec<BaseMaterials>,

    /// Collection of SliceStack. See [`crate::core::slice::SliceStack`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(SLICE_NS))
    )]
    pub slicestack: Vec<SliceStack>,

    /// Collection of ColorGroup. See [`crate::core::material::ColorGroup`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(MATERIAL_NS))
    )]
    pub colorgroup: Vec<ColorGroup>,

    /// Collection of Texture2DGroup. See [`crate::core::material::Texture2DGroup`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(MATERIAL_NS))
    )]
    pub texture2dgroup: Vec<Texture2DGroup>,

    /// Collection of CompositeMaterials. See [`crate::core::material::CompositeMaterials`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(MATERIAL_NS))
    )]
    pub compositematerials: Vec<CompositeMaterials>,

    /// Collection of MultiProperties. See [`crate::core::material::MultiProperties`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(MATERIAL_NS))
    )]
    pub multiproperties: Vec<MultiProperties>,

    /// Collection of Texture2D. See [`crate::core::material::Texture2D`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(MATERIAL_NS))
    )]
    pub texture2d: Vec<Texture2D>,

    /// Collection of displacement texture resources.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(DISPLACEMENT_NS))
    )]
    pub displacement2d: Vec<Displacement2D>,

    /// Collection of normalized vector groups for displacement.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(DISPLACEMENT_NS))
    )]
    pub normvectorgroup: Vec<NormVectorGroup>,

    /// Collection of displacement coordinate groups.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(DISPLACEMENT_NS))
    )]
    pub disp2dgroup: Vec<Disp2DGroup>,
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "base")
)]
pub struct Base {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub name: String,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub displaycolor: String, //ToDo: Make this a specific color struct for flexibility
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "basematerials")
)]
pub struct BaseMaterials {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub id: ResourceId,

    pub base: Vec<Base>,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            Color, OptionalResourceId, OptionalResourceIndex,
            material::{
                ColorElement, ColorGroup, Composite, CompositeMaterials, Filter, Multi,
                MultiProperties, Tex2Coord, Texture2D, Texture2DGroup, TextureContentType,
                TileStyle,
            },
            object::Object,
            slice,
            types::{Double, ResourceIdCollection, ResourceIndexCollection},
        },
        threemf_namespaces::{
            BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS, MATERIAL_NS, MATERIAL_PREFIX, PROD_NS,
            PROD_PREFIX, SLICE_NS, SLICE_PREFIX,
        },
    };

    use super::{Base, BaseMaterials, Resources};

    #[test]
    pub fn toxml_resources_with_object_test() {
        let xml_string = format!(
            r#"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><object xmlns:{}="{}" xmlns:{}="{}" id="1"></object></resources>"#,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            BOOLEAN_PREFIX,
            BOOLEAN_NS,
            PROD_PREFIX,
            PROD_NS,
        );
        let resources = Resources {
            object: vec![Object {
                id: 1,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                kind: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
            }],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_basematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><basematerials id="1"><base name="Base" displaycolor="#FEFEFE00" /></basematerials></resources>"##,
            CORE_NS, MATERIAL_PREFIX, MATERIAL_NS, SLICE_PREFIX, SLICE_NS
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![BaseMaterials {
                id: 1,
                base: vec![Base {
                    name: "Base".to_owned(),
                    displaycolor: "#FEFEFE00".to_owned(),
                }],
            }],
            slicestack: vec![],
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_slicestack_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:slicestack id="236" zbottom="0.5"><{}:sliceref slicestackid="154" slicepath="/2D/model.model" /></{}:slicestack></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            SLICE_PREFIX,
            SLICE_PREFIX,
            SLICE_PREFIX,
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![slice::SliceStack {
                id: 236,
                zbottom: Some(0.5.into()),
                slice: vec![],
                sliceref: vec![slice::SliceRef {
                    slicestackid: 154,
                    slicepath: "/2D/model.model".to_owned(),
                }],
            }],
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_base_test() {
        let xml_string = format!(
            r##"<base xmlns="{}" name="Base" displaycolor="#FEF100" />"##,
            CORE_NS
        );
        let base = Base {
            name: "Base".to_string(),
            displaycolor: "#FEF100".to_string(),
        };
        let base_string = to_string(&base).unwrap();

        assert_eq!(base_string, xml_string);
    }

    #[test]
    pub fn toxml_basematerials_test() {
        let xml_string = format!(
            r##"<basematerials xmlns="{}" id="256"><base name="Base 1" displaycolor="#FEF100" /><base name="Base 2" displaycolor="#FEF369" /></basematerials>"##,
            CORE_NS
        );
        let basematerials = BaseMaterials {
            id: 256,
            base: vec![
                Base {
                    name: "Base 1".to_string(),
                    displaycolor: "#FEF100".to_string(),
                },
                Base {
                    name: "Base 2".to_string(),
                    displaycolor: "#FEF369".to_string(),
                },
            ],
        };
        let base_string = to_string(&basematerials).unwrap();

        assert_eq!(base_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_colorgroup_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:colorgroup id="1"><{}:color color="#FF0000FF" /><{}:color color="#00FF00FF" /></{}:colorgroup></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: vec![ColorGroup {
                id: 1,
                color: vec![
                    ColorElement {
                        color: Color::from_hex("#FF0000").unwrap(),
                    },
                    ColorElement {
                        color: Color::from_hex("#00FF00").unwrap(),
                    },
                ],
            }],
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_texture2dgroup_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:texture2dgroup id="2" texid="1"><{}:tex2coord u="0" v="0" /><{}:tex2coord u="1" v="1" /></{}:texture2dgroup></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: Vec::new(),
            texture2dgroup: vec![Texture2DGroup {
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
            }],
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_compositematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:compositematerials id="1" matid="10" matindices="0 1"><{}:composite values="1 0" /><{}:composite values="0.5 0.5" /></{}:compositematerials></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: vec![CompositeMaterials {
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
            }],
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_multiproperties_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:multiproperties id="1" pids="10 20 30" blendmethods="mix multiply"><{}:multi pindices="0 0 0" /><{}:multi pindices="1 2 3" /></{}:multiproperties></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: vec![MultiProperties {
                id: 1,
                pids: ResourceIdCollection::from(vec![10, 20, 30]),
                blendmethods: Some("mix multiply".to_owned()),
                multi: vec![
                    Multi {
                        pindices: ResourceIndexCollection::from(vec![0, 0, 0]),
                    },
                    Multi {
                        pindices: ResourceIndexCollection::from(vec![1, 2, 3]),
                    },
                ],
            }],
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_texture2d_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:texture2d id="1" path="/3D/texture.png" contenttype="image/png" tilestyleu="wrap" tilestylev="mirror" filter="linear" /></resources>"##,
            CORE_NS, MATERIAL_PREFIX, MATERIAL_NS, SLICE_PREFIX, SLICE_NS, MATERIAL_PREFIX
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: vec![Texture2D {
                id: 1,
                path: "/3D/texture.png".to_owned(),
                contenttype: TextureContentType::Png,
                tilestyleu: Some(TileStyle::Wrap),
                tilestylev: Some(TileStyle::Mirror),
                filter: Some(Filter::Linear),
            }],
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_multiple_material_types_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}" xmlns:{}="{}"><{}:colorgroup id="1"><{}:color color="#FF0000FF" /></{}:colorgroup><{}:texture2d id="2" path="/3D/texture.png" contenttype="image/png" /></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            SLICE_PREFIX,
            SLICE_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: vec![ColorGroup {
                id: 1,
                color: vec![ColorElement {
                    color: Color::from_hex("#FF0000").unwrap(),
                }],
            }],
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: vec![Texture2D {
                id: 2,
                path: "/3D/texture.png".to_owned(),
                contenttype: TextureContentType::Png,
                tilestyleu: None,
                tilestylev: None,
                filter: None,
            }],
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            Color, OptionalResourceId, OptionalResourceIndex,
            material::{
                ColorElement, ColorGroup, Composite, CompositeMaterials, Multi, MultiProperties,
                Tex2Coord, Texture2D, Texture2DGroup, TextureContentType,
            },
            object::Object,
            slice,
            types::{Double, ResourceIdCollection, ResourceIndexCollection},
        },
        threemf_namespaces::{CORE_NS, MATERIAL_NS, MATERIAL_PREFIX, SLICE_NS, SLICE_PREFIX},
    };

    use super::{Base, BaseMaterials, Resources};

    #[test]
    pub fn fromxml_resources_with_object_test() {
        let xml_string = format!(
            r#"<resources xmlns="{}"><object id="1"></object></resources>"#,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: None,
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: None,
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                    meshresolution: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_basematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}"><basematerials id="1"><base name="Base" displaycolor="#FEFEFE00" /></basematerials></resources>"##,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![BaseMaterials {
                    id: 1,
                    base: vec![Base {
                        name: "Base".to_owned(),
                        displaycolor: "#FEFEFE00".to_owned(),
                    }],
                }],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_slicestack_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:slicestack id="236" zbottom="0.5"><{}:sliceref slicestackid="2" slicepath="/2D/slices.model" /></{}:slicestack></resources>"##,
            CORE_NS, SLICE_PREFIX, SLICE_NS, SLICE_PREFIX, SLICE_PREFIX, SLICE_PREFIX,
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![slice::SliceStack {
                    id: 236,
                    zbottom: Some(0.5.into()),
                    slice: vec![],
                    sliceref: vec![slice::SliceRef {
                        slicestackid: 2,
                        slicepath: "/2D/slices.model".to_owned(),
                    }],
                }],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_base_test() {
        let xml_string = format!(
            r##"<base xmlns="{}" name="Base" displaycolor="#FEF100" />"##,
            CORE_NS
        );
        let base = from_str::<Base>(&xml_string).unwrap();

        assert_eq!(
            base,
            Base {
                name: "Base".to_string(),
                displaycolor: "#FEF100".to_string(),
            }
        );
    }

    #[test]
    pub fn fromxml_basematerials_test() {
        let xml_string = format!(
            r##"<basematerials xmlns="{}" id="256"><base name="Base 1" displaycolor="#FEF100" /><base name="Base 2" displaycolor="#FEF369" /></basematerials>"##,
            CORE_NS
        );
        let base = from_str::<BaseMaterials>(&xml_string).unwrap();

        assert_eq!(
            base,
            BaseMaterials {
                id: 256,
                base: vec![
                    Base {
                        name: "Base 1".to_string(),
                        displaycolor: "#FEF100".to_string(),
                    },
                    Base {
                        name: "Base 2".to_string(),
                        displaycolor: "#FEF369".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_colorgroup_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:colorgroup id="1"><{}:color color="#FF0000" /><{}:color color="#00FF00" /></{}:colorgroup></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: vec![ColorGroup {
                    id: 1,
                    color: vec![
                        ColorElement {
                            color: Color::from_hex("#FF0000").unwrap(),
                        },
                        ColorElement {
                            color: Color::from_hex("#00FF00").unwrap(),
                        },
                    ],
                }],
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_texture2dgroup_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:texture2dgroup id="2" texid="1"><{}:tex2coord u="0" v="0" /><{}:tex2coord u="1" v="1" /></{}:texture2dgroup></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: vec![Texture2DGroup {
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
                }],
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_compositematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:compositematerials id="1" matid="10" matindices="0 1"><{}:composite values="1.0 0.0" /><{}:composite values="0.5 0.5" /></{}:compositematerials></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: vec![CompositeMaterials {
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
                }],
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_multiproperties_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:multiproperties id="1" pids="10 20 30" blendmethods="mix multiply"><{}:multi pindices="0 0 0" /><{}:multi pindices="1 2 3" /></{}:multiproperties></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: vec![MultiProperties {
                    id: 1,
                    pids: ResourceIdCollection::from(vec![10, 20, 30]),
                    blendmethods: Some("mix multiply".to_owned()),
                    multi: vec![
                        Multi {
                            pindices: ResourceIndexCollection::from(vec![0, 0, 0])
                        },
                        Multi {
                            pindices: ResourceIndexCollection::from(vec![1, 2, 3])
                        },
                    ],
                }],
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_texture2d_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:texture2d id="1" path="/3D/texture.png" contenttype="image/png" /></resources>"##,
            CORE_NS, MATERIAL_PREFIX, MATERIAL_NS, MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: vec![Texture2D {
                    id: 1,
                    path: "/3D/texture.png".to_owned(),
                    contenttype: TextureContentType::Png,
                    tilestyleu: None,
                    tilestylev: None,
                    filter: None,
                }],
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_multiple_material_types_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:colorgroup id="1"><{}:color color="#FF0000" /></{}:colorgroup><{}:texture2d id="2" path="/3D/texture.png" contenttype="image/png" /></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: vec![ColorGroup {
                    id: 1,
                    color: vec![ColorElement {
                        color: Color::from_hex("#FF0000").unwrap()
                    }],
                }],
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: vec![Texture2D {
                    id: 2,
                    path: "/3D/texture.png".to_owned(),
                    contenttype: TextureContentType::Png,
                    tilestyleu: None,
                    tilestylev: None,
                    filter: None,
                }],
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::{
        core::{
            Color, OptionalResourceId, OptionalResourceIndex,
            material::{
                ColorElement, ColorGroup, Composite, CompositeMaterials, Multi, MultiProperties,
                Tex2Coord, Texture2D, Texture2DGroup, TextureContentType,
            },
            object::Object,
            slice,
            types::{Double, ResourceIdCollection, ResourceIndexCollection},
        },
        threemf_namespaces::{CORE_NS, MATERIAL_NS, MATERIAL_PREFIX, SLICE_NS, SLICE_PREFIX},
    };

    use super::{Base, BaseMaterials, Resources};

    #[test]
    pub fn fromxml_resources_with_object_test() {
        let xml_string = format!(
            r#"<resources xmlns="{}"><object id="1"></object></resources>"#,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: None,
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: None,
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                    meshresolution: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_basematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}"><basematerials id="1"><base name="Base" displaycolor="#FEFEFE00" /></basematerials></resources>"##,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![BaseMaterials {
                    id: 1,
                    base: vec![Base {
                        name: "Base".to_owned(),
                        displaycolor: "#FEFEFE00".to_owned(),
                    }],
                }],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_slicestack_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:slicestack id="236" zbottom="0.5"><sliceref slicestackid="154" slicepath="/2D/model.model" /></{}:slicestack></resources>"##,
            CORE_NS, SLICE_PREFIX, SLICE_NS, SLICE_PREFIX, SLICE_PREFIX,
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![slice::SliceStack {
                    id: 236,
                    zbottom: Some(0.5.into()),
                    slice: vec![],
                    sliceref: vec![slice::SliceRef {
                        slicestackid: 154,
                        slicepath: "/2D/model.model".to_owned(),
                    }],
                }],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_base_test() {
        let xml_string = format!(
            r##"<base xmlns="{}" name="Base" displaycolor="#FEF100" />"##,
            CORE_NS
        );
        let base = from_str::<Base>(&xml_string).unwrap();

        assert_eq!(
            base,
            Base {
                name: "Base".to_string(),
                displaycolor: "#FEF100".to_string(),
            }
        );
    }

    #[test]
    pub fn fromxml_basematerials_test() {
        let xml_string = format!(
            r##"<basematerials xmlns="{}" id="256"><base name="Base 1" displaycolor="#FEF100" /><base name="Base 2" displaycolor="#FEF369" /></basematerials>"##,
            CORE_NS
        );
        let base = from_str::<BaseMaterials>(&xml_string).unwrap();

        assert_eq!(
            base,
            BaseMaterials {
                id: 256,
                base: vec![
                    Base {
                        name: "Base 1".to_string(),
                        displaycolor: "#FEF100".to_string(),
                    },
                    Base {
                        name: "Base 2".to_string(),
                        displaycolor: "#FEF369".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_colorgroup_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:colorgroup id="1"><{}:color color="#FF0000" /><{}:color color="#00FF00" /></{}:colorgroup></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: vec![ColorGroup {
                    id: 1,
                    color: vec![
                        ColorElement {
                            color: Color::from_hex("#FF0000").unwrap(),
                        },
                        ColorElement {
                            color: Color::from_hex("#00FF00").unwrap(),
                        },
                    ],
                }],
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_texture2dgroup_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:texture2dgroup id="2" texid="1"><{}:tex2coord u="0" v="0" /><{}:tex2coord u="1" v="1" /></{}:texture2dgroup></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: vec![Texture2DGroup {
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
                }],
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_compositematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:compositematerials id="1" matid="10" matindices="0 1"><{}:composite values="1.0 0.0" /><{}:composite values="0.5 0.5" /></{}:compositematerials></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: vec![CompositeMaterials {
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
                }],
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_multiproperties_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:multiproperties id="1" pids="10 20 30" blendmethods="mix multiply"><{}:multi pindices="0 0 0" /><{}:multi pindices="1 2 3" /></{}:multiproperties></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: vec![MultiProperties {
                    id: 1,
                    pids: ResourceIdCollection::from(vec![10, 20, 30]),
                    blendmethods: Some("mix multiply".to_owned()),
                    multi: vec![
                        Multi {
                            pindices: ResourceIndexCollection::from(vec![0, 0, 0])
                        },
                        Multi {
                            pindices: ResourceIndexCollection::from(vec![1, 2, 3])
                        },
                    ],
                }],
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_texture2d_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:texture2d id="1" path="/3D/texture.png" contenttype="image/png" /></resources>"##,
            CORE_NS, MATERIAL_PREFIX, MATERIAL_NS, MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: vec![Texture2D {
                    id: 1,
                    path: "/3D/texture.png".to_owned(),
                    contenttype: TextureContentType::Png,
                    tilestyleu: None,
                    tilestylev: None,
                    filter: None,
                }],
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_multiple_material_types_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}" xmlns:{}="{}"><{}:colorgroup id="1"><{}:color color="#FF0000" /></{}:colorgroup><{}:texture2d id="2" path="/3D/texture.png" contenttype="image/png" /></resources>"##,
            CORE_NS,
            MATERIAL_PREFIX,
            MATERIAL_NS,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX,
            MATERIAL_PREFIX
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: vec![ColorGroup {
                    id: 1,
                    color: vec![ColorElement {
                        color: Color::from_hex("#FF0000").unwrap()
                    }],
                }],
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: vec![Texture2D {
                    id: 2,
                    path: "/3D/texture.png".to_owned(),
                    contenttype: TextureContentType::Png,
                    tilestyleu: None,
                    tilestylev: None,
                    filter: None,
                }],
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
            }
        );
    }
}

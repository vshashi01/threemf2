#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::model::domain::{
    any_extensions::{AnyAttributeExtensions, AnyExtensions},
    displacement::{Disp2DGroup, Displacement2D, NormVectorGroup},
    material::{ColorGroup, CompositeMaterials, MultiProperties, Texture2D, Texture2DGroup},
    object::Object,
    slice::SliceStack,
    types::{ResourceId, StrResource},
};
#[cfg(any(feature = "read", feature = "write"))]
use crate::threemf_namespaces::{CORE_NS, DISPLACEMENT_NS, MATERIAL_NS, SLICE_NS};
#[cfg(feature = "write")]
use crate::threemf_namespaces::{MATERIAL_PREFIX, SLICE_PREFIX};

/// A collection of Objects and other properties that are referenced by other elements.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct Resources {
    /// Collection of 3D objects. See [`Object`]
    pub object: Vec<Object>,

    /// Collection of Materials.
    pub basematerials: Vec<BaseMaterials>,

    /// Collection of slice stacks. See [`SliceStack`]
    pub slicestack: Vec<SliceStack>,

    /// Collection of color groups. See [`ColorGroup`]
    pub colorgroup: Vec<ColorGroup>,

    /// Collection of texture coordinate groups. See [`Texture2DGroup`]
    pub texture2dgroup: Vec<Texture2DGroup>,

    /// Collection of composite materials. See [`CompositeMaterials`]
    pub compositematerials: Vec<CompositeMaterials>,

    /// Collection of multi-property definitions. See [`MultiProperties`]
    pub multiproperties: Vec<MultiProperties>,

    /// Collection of 2D texture references. See [`Texture2D`]
    pub texture2d: Vec<Texture2D>,

    /// Collection of displacement texture resources.
    pub displacement2d: Vec<Displacement2D>,

    /// Collection of normalized vector groups for displacement.
    pub normvectorgroup: Vec<NormVectorGroup>,

    /// Collection of displacement coordinate groups.
    pub disp2dgroup: Vec<Disp2DGroup>,

    /// Foreign attributes and foreign resources (`xs:anyAttribute` / `xs:any`).
    ///
    /// Foreign resources are written after all known resources. Only populated with the `any`
    /// feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Resources {
        tag: "resources",
        ns: CORE_NS,
        prefixes: [(MATERIAL_PREFIX, MATERIAL_NS), (SLICE_PREFIX, SLICE_NS)],
        force_prefix: false,
        attributes: {},
        elements: {
            object: Vec<Object> => (CORE_NS, "object"),
            basematerials: Vec<BaseMaterials> => (CORE_NS, "basematerials"),
            slicestack: Vec<SliceStack> => (SLICE_NS, "slicestack"),
            colorgroup: Vec<ColorGroup> => (MATERIAL_NS, "colorgroup"),
            texture2dgroup: Vec<Texture2DGroup> => (MATERIAL_NS, "texture2dgroup"),
            compositematerials: Vec<CompositeMaterials> => (MATERIAL_NS, "compositematerials"),
            multiproperties: Vec<MultiProperties> => (MATERIAL_NS, "multiproperties"),
            texture2d: Vec<Texture2D> => (MATERIAL_NS, "texture2d"),
            displacement2d: Vec<Displacement2D> => (DISPLACEMENT_NS, "displacement2d"),
            normvectorgroup: Vec<NormVectorGroup> => (DISPLACEMENT_NS, "normvectorgroup"),
            disp2dgroup: Vec<Disp2DGroup> => (DISPLACEMENT_NS, "disp2dgroup"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// A single base material entry with name and color.
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Base {
    /// Name of the base material.
    pub name: StrResource,

    /// Display color of the base material in hex format.
    pub displaycolor: StrResource, //ToDo: Make this a specific color struct for flexibility

    /// Foreign attributes of the base material (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Base {
        tag: "base",
        ns: CORE_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {
            name: StrResource => ("", "name"),
            displaycolor: StrResource => ("", "displaycolor"),
        },
        elements: {},
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// A group of base materials referenced by objects.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BaseMaterials {
    /// Unique identifier for this base materials group.
    pub id: ResourceId,

    /// Base material entries in this group.
    pub base: Vec<Base>,

    /// Foreign attributes of the base materials group (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    BaseMaterials {
        tag: "basematerials",
        ns: CORE_NS,
        prefixes: [],
        force_prefix: false,
        attributes: { id: ResourceId => ("", "id") },
        elements: { base: Vec<Base> => (CORE_NS, "base") },
        any: any_extensions: AnyAttributeExtensions,
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            material::{
                ColorElement, ColorGroup, Composite, CompositeMaterials, Filter, Multi,
                MultiProperties, Tex2Coord, Texture2D, Texture2DGroup, TextureContentType,
                TileStyle,
            },
            object::Object,
            slice,
            types::{Double, ResourceIdCollection, ResourceIndexCollection},
        },
        model::{Color, OptionalResourceId, OptionalResourceIndex, PathResource},
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
                any_extensions: Default::default(),
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
            any_extensions: Default::default(),
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
                    name: "Base".into(),
                    displaycolor: "#FEFEFE00".into(),
                    any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
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
            any_extensions: Default::default(),
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
                    slicepath: PathResource::try_from("/2D/model.model").unwrap(),
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
            any_extensions: Default::default(),
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
            name: "Base".into(),
            displaycolor: "#FEF100".into(),
            any_extensions: Default::default(),
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
                    name: "Base 1".into(),
                    displaycolor: "#FEF100".into(),
                    any_extensions: Default::default(),
                },
                Base {
                    name: "Base 2".into(),
                    displaycolor: "#FEF369".into(),
                    any_extensions: Default::default(),
                },
            ],
            any_extensions: Default::default(),
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
            any_extensions: Default::default(),
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
            any_extensions: Default::default(),
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
            any_extensions: Default::default(),
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
                blendmethods: Some("mix multiply".into()),
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
            any_extensions: Default::default(),
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
                path: PathResource::try_from("/3D/texture.png").unwrap(),
                contenttype: TextureContentType::Png,
                tilestyleu: Some(TileStyle::Wrap),
                tilestylev: Some(TileStyle::Mirror),
                filter: Some(Filter::Linear),
            }],
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
            any_extensions: Default::default(),
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
                path: PathResource::try_from("/3D/texture.png").unwrap(),
                contenttype: TextureContentType::Png,
                tilestyleu: None,
                tilestylev: None,
                filter: None,
            }],
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
            any_extensions: Default::default(),
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            material::{
                ColorElement, ColorGroup, Composite, CompositeMaterials, Multi, MultiProperties,
                Tex2Coord, Texture2D, Texture2DGroup, TextureContentType,
            },
            object::Object,
            slice,
            types::{Double, ResourceIdCollection, ResourceIndexCollection},
        },
        model::{Color, OptionalResourceId, OptionalResourceIndex, PathResource},
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
                    any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                        name: "Base".into(),
                        displaycolor: "#FEFEFE00".into(),
                        any_extensions: Default::default(),
                    }],
                    any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                        slicepath: PathResource::try_from("/2D/slices.model").unwrap(),
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
                any_extensions: Default::default(),
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
                name: "Base".into(),
                displaycolor: "#FEF100".into(),
                any_extensions: Default::default(),
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
                        name: "Base 1".into(),
                        displaycolor: "#FEF100".into(),
                        any_extensions: Default::default(),
                    },
                    Base {
                        name: "Base 2".into(),
                        displaycolor: "#FEF369".into(),
                        any_extensions: Default::default(),
                    },
                ],
                any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                    blendmethods: Some("mix multiply".into()),
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
                any_extensions: Default::default(),
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
                    path: PathResource::try_from("/3D/texture.png").unwrap(),
                    contenttype: TextureContentType::Png,
                    tilestyleu: None,
                    tilestylev: None,
                    filter: None,
                }],
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
                any_extensions: Default::default(),
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
                    path: PathResource::try_from("/3D/texture.png").unwrap(),
                    contenttype: TextureContentType::Png,
                    tilestyleu: None,
                    tilestylev: None,
                    filter: None,
                }],
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
                any_extensions: Default::default(),
            }
        );
    }
}

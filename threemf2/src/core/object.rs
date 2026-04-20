#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{
        boolean::BooleanShape,
        component::Components,
        mesh::Mesh,
        slice::MeshResolution,
        types::{OptionalResourceId, OptionalResourceIndex, ResourceId},
    },
    threemf_namespaces::{BOOLEAN_NS, CORE_NS, PROD_NS, SLICE_NS},
};

/// Represents a 3D object in a 3MF model, either a mesh, component assembly, or boolean shape.
///
/// Objects are the primary building blocks of 3MF models. They can contain:
/// - Triangle mesh geometry directly ([`ObjectKind::Mesh`])
/// - References to other objects through components ([`ObjectKind::Components`])
/// - Boolean operations defining a shape ([`ObjectKind::BooleanShape`])
///
/// These three options are mutually exclusive - an object can only have one of them set.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(any(feature="write", feature="memory-optimized-read"), xml(ns(CORE_NS, p=PROD_NS, bo=BOOLEAN_NS, s=SLICE_NS), rename="object"))]
pub struct Object {
    /// A unique identifier for this object
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub id: ResourceId,

    /// Optionally defines the intend of this object. If not defined, the
    /// consumer of the file is safe to assume the object is meant to be a [`ObjectType::Model`]
    ///
    /// See [`ObjectType`] for more details.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, rename = "type")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "type"))]
    pub objecttype: Option<ObjectType>,

    /// Optional path to the thumbnail in the 3MF Package for this object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub thumbnail: Option<String>,

    /// Optional string defining the part number for this object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub partnumber: Option<String>,

    /// Optional string defining the name for this object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub name: Option<String>,

    /// Reference to the property group element with the
    /// matching id attribute value (e.g. Basematerials).
    /// It is REQUIRED if pindex is specified.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::opt_res_id_impl::default_none",
            deserialize_with = "crate::core::types::opt_res_id_impl::deserialize"
        )
    )]
    pub pid: OptionalResourceId,

    /// References a zero-based index into the properties
    /// group specified by pid. This property is used to build the object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::opt_res_index_impl::default_none",
            deserialize_with = "crate::core::types::opt_res_index_impl::deserialize"
        )
    )]
    pub pindex: OptionalResourceIndex,

    /// Optional UUID as a string.
    ///
    /// The UUID is required when using the Production extension.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(PROD_NS), rename = "UUID")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "UUID"))]
    pub uuid: Option<String>,

    /// Identifies the SliceStack that contains the slice data for this object.
    /// If used alone, the slice data exists in the same file as the object.
    /// If used with slicepath, the slice data is in the specified file.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(SLICE_NS))
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::opt_res_id_impl::default_none",
            deserialize_with = "crate::core::types::opt_res_id_impl::deserialize"
        )
    )]
    pub slicestackid: OptionalResourceId,

    /// Absolute path to a non-root model file containing slice data.
    /// Used in combination with slicestackid when slices are in separate files.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(SLICE_NS))
    )]
    pub slicepath: Option<String>,

    /// Indicates the intended resolution of mesh models when slice data is present.
    /// "fullres" means the mesh can regenerate the slices; "lowres" means it cannot.
    /// Packages with "lowres" MUST list the slice extension in requiredextensions.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(SLICE_NS))
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub meshresolution: Option<MeshResolution>,

    /// The actual geometry that is contained in this [`Object`].
    /// This deviates from the standard 3MF Data Model intentionally for
    /// Rust library ergonomics..
    /// This is optional only for developemnt. In Practice an empty
    /// object is not a valid 3MF Model. This may be changed in the future.
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            rename = "#content",
            default = "crate::core::object::serde_object_kind::default_none",
            deserialize_with = "crate::core::object::serde_object_kind::deserialize"
        )
    )]
    pub kind: Option<ObjectKind>,
}

impl Object {
    /// Convenince fn that returns an [`Some<&Mesh>`] if this
    /// object contains a core mesh geometry
    pub fn get_mesh(&self) -> Option<&Mesh> {
        if let Some(kind) = &self.kind
            && let ObjectKind::Mesh(mesh) = kind
        {
            Some(mesh)
        } else {
            None
        }
    }

    /// Convenince fn that returns an [`Some<&Components>`] if this
    /// object contains a core mesh geometry
    pub fn get_components_object(&self) -> Option<&Components> {
        if let Some(kind) = &self.kind
            && let ObjectKind::Components(comps) = kind
        {
            Some(comps)
        } else {
            None
        }
    }

    /// Convenince fn that returns an [`Some<&BooleanShape>`] if this
    /// object contains a core mesh geometry
    pub fn get_boolean_shape_object(&self) -> Option<&BooleanShape> {
        if let Some(kind) = &self.kind
            && let ObjectKind::BooleanShape(shape) = kind
        {
            Some(shape)
        } else {
            None
        }
    }
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, rename_all = "lowercase")
)]
/// Specifies the type of a 3MF object, indicating its role in the build process.
pub enum ObjectType {
    /// The Object is intended as a production model.
    #[default]
    Model,

    /// The Object is intended as Non-Solid Support structure
    Support,

    /// The Object is intended as a Solid Support Structure
    SolidSupport,

    /// The Object is intended as an Open Surface, usually not water-tight mesh
    Surface,

    /// Other auxiliary uses
    Other,
}

impl From<String> for ObjectType {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "model" => Self::Model,
            "support" => Self::Support,
            "solidsupport" => Self::SolidSupport,
            "surface" => Self::Surface,
            "other" => Self::Other,
            _ => Self::Model,
        }
    }
}

#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(forward)
)]
#[cfg_attr(feature = "speed-optimized-read", serde(rename_all = "lowercase"))]
#[non_exhaustive]
/// This is not a type found in the 3MF data model but it is introduced here
/// for better ergonomics of this library. This kind will specify all different variants
/// of a 3MF Object.
pub enum ObjectKind {
    Mesh(Mesh),
    Components(Components),
    BooleanShape(BooleanShape),
}

/// Custom deserializer for `Option<ObjectKind>` to handle empty elements
/// when using speed-optimized-read feature.
#[cfg(feature = "speed-optimized-read")]
pub mod serde_object_kind {
    use super::ObjectKind;
    use serde::{Deserialize, Deserializer};

    /// Returns `None` as the default value for `Option<ObjectKind>`.
    pub fn default_none() -> Option<ObjectKind> {
        None
    }

    /// Deserializes `Option<ObjectKind>` with fallback to `None` on error.
    ///
    /// This handles the case where an `<object>` element has no child elements
    /// (empty object), which would otherwise fail with `MissingChildOrAttribute`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ObjectKind>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Try to deserialize as Option<ObjectKind>
        // If it fails (e.g., no child elements match the enum), return None
        match Option::<ObjectKind>::deserialize(deserializer) {
            Ok(val) => Ok(val),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            component::{Component, Components},
            mesh::{Mesh, Triangles, Vertices},
            slice,
        },
        threemf_namespaces::{
            BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX, BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS,
            CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX, SLICE_NS,
            SLICE_PREFIX,
        },
    };

    use super::{Object, ObjectKind, ObjectType};

    use std::vec;

    #[test]
    pub fn toxml_simple_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="4"></object>"#,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS, PROD_PREFIX, PROD_NS, SLICE_PREFIX, SLICE_NS
        );
        let object = Object {
            id: 4,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_production_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" {}:UUID="someUUID"></object>"#,
            CORE_NS,
            BOOLEAN_PREFIX,
            BOOLEAN_NS,
            PROD_PREFIX,
            PROD_NS,
            SLICE_PREFIX,
            SLICE_NS,
            PROD_PREFIX
        );
        let object = Object {
            id: 4,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: Some("someUUID".to_owned()),
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"></object>"#,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS, PROD_PREFIX, PROD_NS, SLICE_PREFIX, SLICE_NS
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
            partnumber: Some("part_1".to_string()),
            name: Some("Object Part".to_string()),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
        };
        let object_string = to_string(&object).unwrap();
        println!("{}", object_string);

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{CORE_NS}" xmlns:{BOOLEAN_PREFIX}="{BOOLEAN_NS}" xmlns:{PROD_PREFIX}="{PROD_NS}" xmlns:{SLICE_PREFIX}="{SLICE_NS}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><mesh xmlns:{BEAM_LATTICE_PREFIX}="{BEAM_LATTICE_NS}" xmlns:{CORE_TRIANGLESET_PREFIX}="{CORE_TRIANGLESET_NS}"><vertices></vertices><triangles></triangles></mesh></object>"##,
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
            partnumber: Some("part_1".to_string()),
            name: Some("Object Part".to_string()),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: Some(ObjectKind::Mesh(Mesh {
                vertices: Vertices { vertex: vec![] },
                triangles: Triangles { triangle: vec![] },
                trianglesets: None,
                beamlattice: None,
            })),
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" {}:slicestackid="236" {}:slicepath="/2D/model_with_slice_stack.model" {}:meshresolution="lowres"><components><component objectid="23" /></components></object>"##,
            CORE_NS,
            BOOLEAN_PREFIX,
            BOOLEAN_NS,
            PROD_PREFIX,
            PROD_NS,
            SLICE_PREFIX,
            SLICE_NS,
            SLICE_PREFIX,
            SLICE_PREFIX,
            SLICE_PREFIX
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
            partnumber: Some("part_1".to_string()),
            name: Some("Object Part".to_string()),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::new(236),
            slicepath: Some("/2D/model_with_slice_stack.model".to_owned()),
            meshresolution: Some(slice::MeshResolution::LowRes),
            kind: Some(ObjectKind::Components(Components {
                component: vec![Component {
                    objectid: 23,
                    transform: None,
                    path: None,
                    uuid: None,
                }],
            })),
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[derive(Debug, ToXml)]
    pub struct ObjectTypes {
        #[xml(rename = "children")]
        objecttype: Vec<ObjectType>,

        #[xml(rename = "attr", attribute)]
        attribute: Option<ObjectType>,
    }

    #[test]
    pub fn toxml_objecttype_test() {
        let xml_string = format!(
            r#"<ObjectTypes attr="model"><{s}>model</{s}><{s}>support</{s}><{s}>solidsupport</{s}><{s}>support</{s}><{s}>other</{s}></ObjectTypes>"#,
            s = "children"
        );
        let objecttypes = ObjectTypes {
            attribute: Some(ObjectType::Model),
            objecttype: vec![
                ObjectType::Model,
                ObjectType::Support,
                ObjectType::SolidSupport,
                ObjectType::Support,
                ObjectType::Other,
            ],
        };
        let objecttype_string = to_string(&objecttypes).unwrap();

        assert_eq!(objecttype_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::{FromXml, from_str};
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            component::{Component, Components},
            mesh::{Mesh, Triangles, Vertices},
            slice,
        },
        threemf_namespaces::{
            CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX, SLICE_NS,
            SLICE_PREFIX,
        },
    };

    use super::{Object, ObjectKind, ObjectType};

    use std::vec;

    #[test]
    pub fn fromxml_simple_object_test() {
        let xml_string = format!(r#"<object xmlns="{}" id="4"></object>"#, CORE_NS);
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_production_object_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4" {}:UUID="someUUID"></object>"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX,
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: Some("someUUID".to_owned()),
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::new(123),
                pindex: OptionalResourceIndex::new(123),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test_x() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::new(123),
                pindex: OptionalResourceIndex::new(123),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" {}:slicestackid="236" {}:meshresolution="lowres"><mesh xmlns:{}="{}"><vertices></vertices><triangles></triangles></mesh></object>"##,
            CORE_NS,
            PROD_PREFIX,
            PROD_NS,
            SLICE_PREFIX,
            SLICE_NS,
            SLICE_PREFIX,
            SLICE_PREFIX,
            CORE_TRIANGLESET_PREFIX,
            CORE_TRIANGLESET_NS,
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::new(236),
                slicepath: None,
                meshresolution: Some(slice::MeshResolution::LowRes),
                kind: Some(ObjectKind::Mesh(Mesh {
                    vertices: Vertices { vertex: vec![] },
                    triangles: Triangles { triangle: vec![] },
                    trianglesets: None,
                    beamlattice: None
                }))
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><components><component objectid="23" /></components></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: Some(ObjectKind::Components(Components {
                    component: vec![Component {
                        objectid: 23,
                        transform: None,
                        path: None,
                        uuid: None
                    }]
                }))
            }
        );
    }

    #[derive(Debug, FromXml, PartialEq)]
    pub struct ObjectTypes {
        #[xml(rename = "children")]
        childs: Vec<ObjectType>,

        #[xml(rename = "attr", attribute)]
        attribute: Option<ObjectType>,
    }

    #[test]
    pub fn fromxml_objecttype_test() {
        let xml_string = format!(
            r#"<ObjectTypes attr="model"><{s}>model</{s}><{s}>support</{s}><{s}>solidsupport</{s}><{s}>support</{s}><{s}>other</{s}></ObjectTypes>"#,
            s = "children"
        );
        let objecttypes = from_str::<ObjectTypes>(&xml_string).unwrap();

        assert_eq!(
            objecttypes,
            ObjectTypes {
                attribute: Some(ObjectType::Model),
                childs: vec![
                    ObjectType::Model,
                    ObjectType::Support,
                    ObjectType::SolidSupport,
                    ObjectType::Support,
                    ObjectType::Other,
                ],
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_roxmltree::from_str;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            component::{Component, Components},
            mesh::{Mesh, Triangles, Vertices},
            slice,
        },
        threemf_namespaces::{
            CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX, SLICE_NS,
            SLICE_PREFIX,
        },
    };

    use super::{Object, ObjectKind, ObjectType};

    use std::vec;

    #[test]
    pub fn fromxml_simple_object_test() {
        let xml_string = format!(r#"<object xmlns="{}" id="4"></object>"#, CORE_NS);
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_production_object_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" id="4" {}:UUID="someUUID"></object>"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX,
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: Some("someUUID".to_owned()),
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::new(123),
                pindex: OptionalResourceIndex::new(123),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test_x() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::new(123),
                pindex: OptionalResourceIndex::new(123),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part" {}:slicestackid="236" {}:meshresolution="lowres"><mesh xmlns:{}="{}"><vertices></vertices><triangles></triangles></mesh></object>"##,
            CORE_NS,
            PROD_PREFIX,
            PROD_NS,
            SLICE_PREFIX,
            SLICE_NS,
            SLICE_PREFIX,
            SLICE_PREFIX,
            CORE_TRIANGLESET_PREFIX,
            CORE_TRIANGLESET_NS,
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::new(236),
                slicepath: None,
                meshresolution: Some(slice::MeshResolution::LowRes),
                kind: Some(ObjectKind::Mesh(Mesh {
                    vertices: Vertices { vertex: vec![] },
                    triangles: Triangles { triangle: vec![] },
                    trianglesets: None,
                    beamlattice: None,
                }))
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="\thumbnail\part_thumbnail.png" partnumber="part_1" name="Object Part"><components><component objectid="23" /></components></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = from_str::<Object>(&xml_string);
        println!("{object:?}");
        assert_eq!(
            object.unwrap(),
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some("\\thumbnail\\part_thumbnail.png".to_string()),
                partnumber: Some("part_1".to_string()),
                name: Some("Object Part".to_string()),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: Some(ObjectKind::Components(Components {
                    component: vec![Component {
                        objectid: 23,
                        transform: None,
                        path: None,
                        uuid: None,
                    }]
                }))
            }
        );
    }

    #[derive(Debug, Deserialize, PartialEq)]
    pub struct ObjectTypes {
        #[serde(rename = "children")]
        childs: Vec<ObjectType>,

        #[serde(rename = "attr")]
        attribute: Option<ObjectType>,
    }

    #[test]
    pub fn fromxml_objecttype_test() {
        let xml_string = format!(
            r#"<ObjectTypes attr="model"><{s}>model</{s}><{s}>support</{s}><{s}>solidsupport</{s}><{s}>support</{s}><{s}>other</{s}><{s}>somethingelse</{s}></ObjectTypes>"#,
            s = "children"
        );
        let objecttypes = from_str::<ObjectTypes>(&xml_string).unwrap();

        assert_eq!(
            objecttypes,
            ObjectTypes {
                attribute: Some(ObjectType::Model),
                childs: vec![
                    ObjectType::Model,
                    ObjectType::Support,
                    ObjectType::SolidSupport,
                    ObjectType::Support,
                    ObjectType::Other,
                    ObjectType::Model,
                ],
            }
        );
    }
}

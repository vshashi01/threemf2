#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

use crate::{
    model::domain::{
        boolean::BooleanShape, component::Components, displacement::DisplacementMesh, mesh::Mesh,
        slice::MeshResolution,
    },
    model::{
        OptionalResourceId, OptionalResourceIndex, PathResource, ResourceId, StrResource,
        UuidResource,
    },
    threemf_namespaces::{BOOLEAN_NS, CORE_NS, PROD_NS, SLICE_NS},
};

#[cfg(feature = "any")]
use crate::model::domain::any::AnyElement;
use crate::model::domain::any_extensions::AnyExtensions;
use crate::model::domain::metadata::MetadataGroup;
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
#[cfg(feature = "write")]
use crate::threemf_namespaces::{BOOLEAN_PREFIX, PROD_PREFIX, SLICE_PREFIX};

/// Represents a 3D object in a 3MF model, either a mesh, component assembly, or boolean shape.
///
/// Objects are the primary building blocks of 3MF models. They can contain:
/// - Triangle mesh geometry directly ([`ObjectKind::Mesh`])
/// - References to other objects through components ([`ObjectKind::Components`])
/// - Boolean operations defining a shape ([`ObjectKind::BooleanShape`])
///
/// These three options are mutually exclusive - an object can only have one of them set.
#[derive(PartialEq, Debug, Clone)]
pub struct Object {
    /// A unique identifier for this object
    pub id: ResourceId,

    /// Optionally defines the intend of this object. If not defined, the
    /// consumer of the file is safe to assume the object is meant to be a [`ObjectType::Model`]
    ///
    /// See [`ObjectType`] for more details.
    pub objecttype: Option<ObjectType>,

    /// Optional path to the thumbnail in the 3MF Package for this object.
    pub thumbnail: Option<PathResource>,

    /// Optional string defining the part number for this object.
    pub partnumber: Option<StrResource>,

    /// Optional string defining the name for this object.
    pub name: Option<StrResource>,

    /// Reference to the property group element with the
    /// matching id attribute value (e.g. Basematerials).
    /// It is REQUIRED if pindex is specified.
    pub pid: OptionalResourceId,

    /// References a zero-based index into the properties
    /// group specified by pid. This property is used to build the object.
    pub pindex: OptionalResourceIndex,

    /// Optional UUID as a string.
    ///
    /// The UUID is required when using the Production extension.
    pub uuid: Option<UuidResource>,

    /// Identifies the SliceStack that contains the slice data for this object.
    /// If used alone, the slice data exists in the same file as the object.
    /// If used with slicepath, the slice data is in the specified file.
    pub slicestackid: OptionalResourceId,

    /// Absolute path to a non-root model file containing slice data.
    /// Used in combination with slicestackid when slices are in separate files.
    pub slicepath: Option<PathResource>,

    /// Indicates the intended resolution of mesh models when slice data is present.
    /// "fullres" means the mesh can regenerate the slices; "lowres" means it cannot.
    /// Packages with "lowres" MUST list the slice extension in requiredextensions.
    pub meshresolution: Option<MeshResolution>,

    /// Metadata attached to this object (`<metadatagroup>`).
    pub metadatagroup: Option<MetadataGroup>,

    /// The actual geometry that is contained in this [`Object`].
    /// This deviates from the standard 3MF Data Model intentionally for
    /// Rust library ergonomics..
    /// This is optional only for developemnt. In Practice an empty
    /// object is not a valid 3MF Model. This may be changed in the future.
    pub kind: Option<ObjectKind>,

    /// Foreign attributes and trailing child elements of the object (`xs:anyAttribute` /
    /// `xs:any`). A foreign element used as the object shape is stored in
    /// [`Object::kind`] as `ObjectKind::Any` instead.
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Object {
        tag: "object",
        ns: CORE_NS,
        prefixes: [
            (BOOLEAN_PREFIX, BOOLEAN_NS),
            (PROD_PREFIX, PROD_NS),
            (SLICE_PREFIX, SLICE_NS),
        ],
        force_prefix: true,
        attributes: {
            id: ResourceId => ("", "id"),
            objecttype: Option<ObjectType> => ("", "type"),
            thumbnail: Option<PathResource> => ("", "thumbnail"),
            partnumber: Option<StrResource> => ("", "partnumber"),
            name: Option<StrResource> => ("", "name"),
            pid: OptionalResourceId => ("", "pid"),
            pindex: OptionalResourceIndex => ("", "pindex"),
            uuid: Option<UuidResource> => (PROD_NS, "UUID"),
            slicestackid: OptionalResourceId => (SLICE_NS, "slicestackid"),
            slicepath: Option<PathResource> => (SLICE_NS, "slicepath"),
            meshresolution: Option<MeshResolution> => (SLICE_NS, "meshresolution"),
        },
        elements: {
            metadatagroup: Option<MetadataGroup> => (CORE_NS, "metadatagroup"),
            kind: Option<ObjectKind> => (CORE_NS, "kind"),
        },
        any: any_extensions: AnyExtensions,
        shape: kind => ObjectKind::Any,
    }
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

    /// Convenince fn that returns an [`Some<&DisplacementMesh>`] if this
    /// object contains a displacement mesh geometry
    pub fn get_displacement_mesh(&self) -> Option<&DisplacementMesh> {
        if let Some(kind) = &self.kind
            && let ObjectKind::DisplacementMesh(mesh) = kind
        {
            Some(mesh)
        } else {
            None
        }
    }
}

#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
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

#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(any(feature = "write", feature = "read"), xml(forward))]
#[non_exhaustive]
/// This is not a type found in the 3MF data model but it is introduced here
/// for better ergonomics of this library. This kind will specify all different variants
/// of a 3MF Object.
pub enum ObjectKind {
    /// Object contains triangle mesh geometry.
    Mesh(Mesh),
    /// Object references other objects via components.
    Components(Components),
    /// Object defines a boolean shape operation.
    BooleanShape(BooleanShape),
    /// Object contains displacement mesh geometry.
    DisplacementMesh(DisplacementMesh),
    /// Object shape defined by a foreign (non 3MF core) element, for example from an extension
    /// this library does not model. Only produced with the `any` feature.
    #[cfg(feature = "any")]
    Any(AnyElement),
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            component::{Component, Components},
            mesh::{Mesh, Triangles, Vertices},
            slice,
        },
        model::{
            OptionalResourceId, OptionalResourceIndex, PathResource, StrResource, UuidResource,
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
            any_extensions: Default::default(),
            metadatagroup: Default::default(),
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
            uuid: Some(UuidResource::from("someUUID")),
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
            any_extensions: Default::default(),
            metadatagroup: Default::default(),
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part"></object>"#,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS, PROD_PREFIX, PROD_NS, SLICE_PREFIX, SLICE_NS
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
            partnumber: Some(StrResource::new("part_1")),
            name: Some(StrResource::new("Object Part")),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
            any_extensions: Default::default(),
            metadatagroup: Default::default(),
        };
        let object_string = to_string(&object).unwrap();
        println!("{}", object_string);

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{CORE_NS}" xmlns:{BOOLEAN_PREFIX}="{BOOLEAN_NS}" xmlns:{PROD_PREFIX}="{PROD_NS}" xmlns:{SLICE_PREFIX}="{SLICE_NS}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part"><mesh xmlns:{BEAM_LATTICE_PREFIX}="{BEAM_LATTICE_NS}" xmlns:{CORE_TRIANGLESET_PREFIX}="{CORE_TRIANGLESET_NS}"><vertices></vertices><triangles></triangles></mesh></object>"##,
        );
        let object = Object {
            id: 4,
            objecttype: Some(ObjectType::Model),
            thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
            partnumber: Some(StrResource::new("part_1")),
            name: Some(StrResource::new("Object Part")),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: Some(ObjectKind::Mesh(Mesh {
                vertices: Vertices {
                    vertex: vec![],
                    any_extensions: Default::default(),
                },
                triangles: Triangles {
                    triangle: vec![],
                    any_extensions: Default::default(),
                },
                trianglesets: None,
                beamlattice: None,
                any_extensions: Default::default(),
            })),
            any_extensions: Default::default(),
            metadatagroup: Default::default(),
        };
        let object_string = to_string(&object).unwrap();

        assert_eq!(object_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part" {}:slicestackid="236" {}:slicepath="/2D/model_with_slice_stack.model" {}:meshresolution="lowres"><components><component objectid="23" /></components></object>"##,
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
            thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
            partnumber: Some(StrResource::new("part_1")),
            name: Some(StrResource::new("Object Part")),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::new(236),
            slicepath: Some(PathResource::try_from("/2D/model_with_slice_stack.model").unwrap()),
            meshresolution: Some(slice::MeshResolution::LowRes),
            kind: Some(ObjectKind::Components(Components {
                component: vec![Component {
                    objectid: 23,
                    transform: None,
                    path: None,
                    uuid: None,
                    any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
            })),
            any_extensions: Default::default(),
            metadatagroup: Default::default(),
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

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::{FromXml, from_str};
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            component::{Component, Components},
            mesh::{Mesh, Triangles, Vertices},
            slice,
        },
        model::{
            OptionalResourceId, OptionalResourceIndex, PathResource, StrResource, UuidResource,
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
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
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
                uuid: Some(UuidResource::from("someUUID")),
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
                partnumber: Some(StrResource::new("part_1")),
                name: Some(StrResource::new("Object Part")),
                pid: OptionalResourceId::new(123),
                pindex: OptionalResourceIndex::new(123),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_intermediate_object_test_x() {
        let xml_string = format!(
            r#"<object xmlns="{}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part" pid="123" pindex="123"></object>"#,
            CORE_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
                partnumber: Some(StrResource::new("part_1")),
                name: Some(StrResource::new("Object Part")),
                pid: OptionalResourceId::new(123),
                pindex: OptionalResourceIndex::new(123),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: None,
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_mesh_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part" {}:slicestackid="236" {}:meshresolution="lowres"><mesh xmlns:{}="{}"><vertices></vertices><triangles></triangles></mesh></object>"##,
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
                thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
                partnumber: Some(StrResource::new("part_1")),
                name: Some(StrResource::new("Object Part")),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::new(236),
                slicepath: None,
                meshresolution: Some(slice::MeshResolution::LowRes),
                kind: Some(ObjectKind::Mesh(Mesh {
                    vertices: Vertices {
                        vertex: vec![],
                        any_extensions: Default::default()
                    },
                    triangles: Triangles {
                        triangle: vec![],
                        any_extensions: Default::default()
                    },
                    trianglesets: None,
                    beamlattice: None,
                    any_extensions: Default::default(),
                })),
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
            }
        );
    }

    #[test]
    pub fn fromxml_advanced_component_object_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="4" type="model" thumbnail="/thumbnail/part_thumbnail.png" partnumber="part_1" name="Object Part"><components><component objectid="23" /></components></object>"##,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let object = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            object,
            Object {
                id: 4,
                objecttype: Some(ObjectType::Model),
                thumbnail: Some(PathResource::try_from("/thumbnail/part_thumbnail.png").unwrap()),
                partnumber: Some(StrResource::new("part_1")),
                name: Some(StrResource::new("Object Part")),
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
                        any_extensions: Default::default(),
                    }],
                    any_extensions: Default::default(),
                })),
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
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

#[cfg(test)]
mod from_string_tests {
    use super::ObjectType;

    #[test]
    fn object_type_from_string() {
        assert_eq!(ObjectType::from("model".to_string()), ObjectType::Model);
        assert_eq!(ObjectType::from("MODEL".to_string()), ObjectType::Model);
        assert_eq!(ObjectType::from("support".to_string()), ObjectType::Support);
        assert_eq!(
            ObjectType::from("solidsupport".to_string()),
            ObjectType::SolidSupport
        );
        assert_eq!(ObjectType::from("other".to_string()), ObjectType::Other);
        assert_eq!(ObjectType::from("unknown".to_string()), ObjectType::Model);
    }
}

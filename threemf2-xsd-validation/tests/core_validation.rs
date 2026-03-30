//! Core 3MF XSD validation tests
//!
//! Tests validation of basic 3MF models against the core specification XSD.

use std::collections::HashMap;
use std::io::Cursor;
use threemf2::{
    core::{
        OptionalResourceId,
        build::{Build, Item},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        metadata::Preserve,
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
        types::OptionalResourceIndex,
    },
    io::{
        ThreemfPackage,
        content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
        relationship::{Relationship, RelationshipType, Relationships},
    },
};

mod validation_utils;
use validation_utils::validation::{extract_model_xml, validate_or_panic};

const CORE_XSD: &str = include_str!("data/xsd/3mf-core-1.3.0.xsd");

#[test]
fn validate_simple_mesh_against_core_xsd() {
    // Create a simple mesh model
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(0.0, 2.0, 0.0),
            Vertex::new(0.0, 1.0, 1.0),
            Vertex::new(1.0, 0.0, 0.0),
        ],
    };

    let triangles = Triangles {
        triangle: vec![
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: None,
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Simple Mesh".to_owned()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(mesh)),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        },
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::from([(
            "_rels/.rels".to_owned(),
            Relationships {
                relationships: vec![Relationship {
                    id: "rel0".to_owned(),
                    target: "3D/3Dmodel.model".to_owned(),
                    relationship_type: RelationshipType::Model,
                }],
            },
        )]),
        ContentTypes {
            defaults: vec![
                DefaultContentTypes {
                    extension: "rels".to_owned(),
                    content_type: DefaultContentTypeEnum::Relationship,
                },
                DefaultContentTypes {
                    extension: "model".to_owned(),
                    content_type: DefaultContentTypeEnum::Model,
                },
            ],
        },
    );

    let mut buf = Cursor::new(Vec::new());
    write_package
        .write(&mut buf)
        .expect("Error writing package");

    let model_xml =
        extract_model_xml(buf.get_ref()).expect("Failed to extract model XML from package");

    validate_or_panic(
        &model_xml,
        &[(threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes())],
        "3MF Core Schema",
    );
}

#[test]
fn validate_model_with_metadata_against_core_xsd() {
    use threemf2::core::metadata::Metadata;

    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.5, 1.0, 0.0),
            Vertex::new(0.5, 0.5, 1.0),
        ],
    };

    let triangles = Triangles {
        triangle: vec![
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: None,
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![
                Metadata {
                    name: "Title".to_owned(),
                    value: Some("Test Model".to_owned()),
                    preserve: Some(Preserve(false)),
                },
                Metadata {
                    name: "Description".to_owned(),
                    value: Some("A test model for XSD validation".to_owned()),
                    preserve: Some(Preserve(false)),
                },
            ],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Mesh with Metadata".to_owned()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(mesh)),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        },
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::from([(
            "_rels/.rels".to_owned(),
            Relationships {
                relationships: vec![Relationship {
                    id: "rel0".to_owned(),
                    target: "3D/3Dmodel.model".to_owned(),
                    relationship_type: RelationshipType::Model,
                }],
            },
        )]),
        ContentTypes {
            defaults: vec![
                DefaultContentTypes {
                    extension: "rels".to_owned(),
                    content_type: DefaultContentTypeEnum::Relationship,
                },
                DefaultContentTypes {
                    extension: "model".to_owned(),
                    content_type: DefaultContentTypeEnum::Model,
                },
            ],
        },
    );

    let mut buf = Cursor::new(Vec::new());
    write_package
        .write(&mut buf)
        .expect("Error writing package");

    let model_xml =
        extract_model_xml(buf.get_ref()).expect("Failed to extract model XML from package");

    validate_or_panic(
        &model_xml,
        &[(threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes())],
        "3MF Core Schema",
    );
}

#[test]
fn validate_model_with_different_units() {
    for unit in [
        Unit::Micron,
        Unit::Millimeter,
        Unit::Centimeter,
        Unit::Inch,
        Unit::Foot,
        Unit::Meter,
    ] {
        let vertices = Vertices {
            vertex: vec![
                Vertex::new(0.0, 0.0, 0.0),
                Vertex::new(1.0, 0.0, 0.0),
                Vertex::new(0.5, 1.0, 0.0),
                Vertex::new(0.5, 0.5, 1.0),
            ],
        };

        let triangles = Triangles {
            triangle: vec![
                Triangle {
                    v1: 0,
                    v2: 1,
                    v3: 2,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 0,
                    v2: 2,
                    v3: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 0,
                    v2: 1,
                    v3: 2,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 0,
                    v2: 2,
                    v3: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        };

        let mesh = Mesh {
            triangles,
            vertices,
            trianglesets: None,
            beamlattice: None,
        };

        let write_package = ThreemfPackage::new(
            Model {
                unit: Some(unit.clone()),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some(format!("Mesh in {:?}", unit)),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(mesh)),
                    }],
                    basematerials: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![Item {
                        objectid: 1,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: None,
                    }],
                },
            },
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(
                "_rels/.rels".to_owned(),
                Relationships {
                    relationships: vec![Relationship {
                        id: "rel0".to_owned(),
                        target: "3D/3Dmodel.model".to_owned(),
                        relationship_type: RelationshipType::Model,
                    }],
                },
            )]),
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                ],
            },
        );

        let mut buf = Cursor::new(Vec::new());
        write_package
            .write(&mut buf)
            .expect("Error writing package");

        let model_xml =
            extract_model_xml(buf.get_ref()).expect("Failed to extract model XML from package");

        validate_or_panic(
            &model_xml,
            &[(threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes())],
            &format!("3MF Core Schema (unit: {:?})", unit),
        );
    }
}

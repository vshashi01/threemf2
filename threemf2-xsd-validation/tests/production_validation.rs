//! Production extension XSD validation tests
//!
//! Tests validation of 3MF models with Production extension features (UUIDs, paths)
//! against the Production extension XSD schemas.

use std::collections::HashMap;
use std::io::Cursor;
use threemf2::{
    core::{
        OptionalResourceId,
        build::{Build, Item},
        component::Component,
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
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
const PRODUCTION_XSD: &str = include_str!("data/xsd/3mf-production-1.2.0.xsd");
// const PRODUCTION_ALTERNATIVES_XSD: &str =
//     include_str!("data/xsd/3mf-production-alternatives-2021-04.xsd");

fn validate_production_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::PROD_NS,
                PRODUCTION_XSD.as_bytes(),
            ),
        ],
        "Production Schema",
    );
}

#[test]
fn validate_simple_production_model_with_uuids() {
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
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("p".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Production Model".to_owned()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: Some("01cbb956-1d24-062d-fbe6-7362e5727594".to_owned()),
                    kind: Some(ObjectKind::Mesh(mesh)),
                    meshresolution: None,
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: Some("96681a5d-5b0f-e592-8c51-da7ed587cb5f".to_owned()),
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: Some("b3de5826-ccb6-3dbc-d6c4-29a2d730766c".to_owned()),
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

    validate_production_model(&model_xml);
}

#[test]
fn validate_production_model_with_components() {
    // Create two objects that will be referenced as components
    let vertices1 = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.5, 1.0, 0.0),
            Vertex::new(0.5, 0.5, 1.0),
        ],
    };

    let triangles1 = Triangles {
        triangle: vec![
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            };
            4
        ],
    };

    let vertices2 = Vertices {
        vertex: vec![
            Vertex::new(2.0, 0.0, 0.0),
            Vertex::new(3.0, 0.0, 0.0),
            Vertex::new(2.5, 1.0, 0.0),
            Vertex::new(2.5, 0.5, 1.0),
        ],
    };

    let triangles2 = Triangles {
        triangle: vec![
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            };
            4
        ],
    };

    let mesh1 = Mesh {
        triangles: triangles1,
        vertices: vertices1,
        trianglesets: None,
        beamlattice: None,
    };

    let mesh2 = Mesh {
        triangles: triangles2,
        vertices: vertices2,
        trianglesets: None,
        beamlattice: None,
    };

    let component = Component {
        objectid: 1,
        transform: None,
        path: None,
        uuid: Some("comp-uuid-1234-5678-90ab-cdef12345678".to_owned()),
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("p".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![
                    Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Component Part 1".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("uuid-part1-1234-5678-90ab-cdef12345678".to_owned()),
                        kind: Some(ObjectKind::Mesh(mesh1)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 2,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Component Part 2".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("uuid-part2-1234-5678-90ab-cdef12345678".to_owned()),
                        kind: Some(ObjectKind::Mesh(mesh2)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 3,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Assembly".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("uuid-assembly-1234-5678-90ab-cdef12345678".to_owned()),
                        kind: Some(ObjectKind::Components(
                            threemf2::core::component::Components {
                                component: vec![component],
                            },
                        )),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                ],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: Some("build-uuid-1234-5678-90ab-cdef12345678".to_owned()),
                item: vec![Item {
                    objectid: 3,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: Some("item-uuid-1234-5678-90ab-cdef12345678".to_owned()),
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

    validate_production_model(&model_xml);
}

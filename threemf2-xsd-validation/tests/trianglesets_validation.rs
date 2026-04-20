//! Triangle Sets extension XSD validation tests
//!
//! Tests validation of 3MF models with Triangle Sets against
//! the Core Triangle Sets extension XSD schema.

use std::collections::HashMap;
use std::io::Cursor;
use threemf2::{
    core::{
        OptionalResourceId,
        build::{Build, Item},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
        triangle_set::{TriangleRef, TriangleRefRange, TriangleSet, TriangleSets},
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
const TRIANGLE_SETS_XSD: &str = include_str!("data/xsd/3mf-trianglesets-2021-07.xsd");

fn validate_trianglesets_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::CORE_TRIANGLESET_NS,
                TRIANGLE_SETS_XSD.as_bytes(),
            ),
        ],
        "Triangle Sets Schema",
    );
}

#[test]
fn validate_simple_trianglesets() {
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

    let triangle_sets = TriangleSets {
        trianglesets: vec![
            TriangleSet {
                name: "Top Surface".to_owned(),
                identifier: "urn:3mf:example:top-surface".to_owned(),
                triangle_ref: vec![TriangleRef { index: 0 }, TriangleRef { index: 1 }],
                triangle_refrange: vec![],
            },
            TriangleSet {
                name: "Side Surface".to_owned(),
                identifier: "urn:3mf:example:side-surface".to_owned(),
                triangle_ref: vec![TriangleRef { index: 2 }],
                triangle_refrange: vec![],
            },
        ],
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: Some(triangle_sets),
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: Some("t".to_owned()),
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Mesh with Triangle Sets".to_owned()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(mesh)),
                    meshresolution: None,
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
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

    validate_trianglesets_model(&model_xml);
}

#[test]
fn validate_trianglesets_with_ref_ranges() {
    // Create a mesh with many triangles and use ref ranges
    let mut vertices = vec![
        Vertex::new(0.0, 0.0, 0.0),
        Vertex::new(1.0, 0.0, 0.0),
        Vertex::new(0.5, 1.0, 0.0),
    ];

    // Add more vertices
    for i in 0..10 {
        vertices.push(Vertex::new(i as f64, 0.0, 0.0));
        vertices.push(Vertex::new(i as f64 + 0.5, 1.0, 0.0));
    }

    let mut triangles = vec![];
    // Create triangles
    for i in 0..15 {
        triangles.push(Triangle {
            v1: i as u32,
            v2: (i + 1) as u32,
            v3: (i + 2) as u32,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });
    }

    let vertices = Vertices { vertex: vertices };
    let triangles = Triangles {
        triangle: triangles,
    };

    let triangle_sets = TriangleSets {
        trianglesets: vec![
            TriangleSet {
                name: "First Half".to_owned(),
                identifier: "urn:3mf:example:first-half".to_owned(),
                triangle_ref: vec![],
                triangle_refrange: vec![TriangleRefRange {
                    startindex: 0,
                    endindex: 7,
                }],
            },
            TriangleSet {
                name: "Second Half".to_owned(),
                identifier: "urn:3mf:example:second-half".to_owned(),
                triangle_ref: vec![],
                triangle_refrange: vec![TriangleRefRange {
                    startindex: 8,
                    endindex: 14,
                }],
            },
        ],
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: Some(triangle_sets),
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: Some("t".to_owned()),
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Mesh with Triangle Set Ranges".to_owned()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(mesh)),
                    meshresolution: None,
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
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

    validate_trianglesets_model(&model_xml);
}

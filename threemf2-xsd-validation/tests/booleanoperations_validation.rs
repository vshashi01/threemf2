//! Boolean Operations extension XSD validation tests
//!
//! Tests validation of 3MF models with boolean operations against
//! the Boolean Operations extension XSD schemas.

use std::collections::HashMap;
use std::io::Cursor;
use threemf2::{
    core::{
        OptionalResourceId,
        boolean::{Boolean, BooleanOperation, BooleanShape},
        build::{Build, Item},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
        transform::Transform,
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
const BOOLEAN_XSD: &str = include_str!("data/xsd/3mf-boolean-operations-1.1.1.xsd");
const PROD_XSD: &str = include_str!("data/xsd/3mf-production-1.2.0.xsd");

fn validate_boolean_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::BOOLEAN_NS,
                BOOLEAN_XSD.as_bytes(),
            ),
        ],
        "Boolean Operations Schema",
    );
}

fn validate_boolean_with_production_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::BOOLEAN_NS,
                BOOLEAN_XSD.as_bytes(),
            ),
            (threemf2::threemf_namespaces::PROD_NS, PROD_XSD.as_bytes()),
        ],
        "Boolean Operations + Production Schema",
    );
}

/// Helper function to create a simple cube mesh
fn create_cube_mesh() -> Mesh {
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),    // 0
            Vertex::new(10.0, 0.0, 0.0),   // 1
            Vertex::new(10.0, 10.0, 0.0),  // 2
            Vertex::new(0.0, 10.0, 0.0),   // 3
            Vertex::new(0.0, 0.0, 10.0),   // 4
            Vertex::new(10.0, 0.0, 10.0),  // 5
            Vertex::new(10.0, 10.0, 10.0), // 6
            Vertex::new(0.0, 10.0, 10.0),  // 7
        ],
    };

    let triangles = Triangles {
        triangle: vec![
            // Bottom face
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
            // Top face
            Triangle {
                v1: 4,
                v2: 6,
                v3: 5,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 4,
                v2: 7,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Front face
            Triangle {
                v1: 0,
                v2: 5,
                v3: 1,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 4,
                v3: 5,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Back face
            Triangle {
                v1: 3,
                v2: 2,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 3,
                v2: 6,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Left face
            Triangle {
                v1: 0,
                v2: 3,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 7,
                v3: 4,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Right face
            Triangle {
                v1: 1,
                v2: 5,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 1,
                v2: 6,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    };

    Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    }
}

/// Helper function to create a simple sphere-like mesh (approximated as a small cube)
fn create_sphere_mesh() -> Mesh {
    let r = 3.0_f64;
    let cx = 5.0_f64;
    let cy = 5.0_f64;
    let cz = 5.0_f64;

    let vertices = Vertices {
        vertex: vec![
            Vertex::new(cx - r, cy - r, cz - r), // 0
            Vertex::new(cx + r, cy - r, cz - r), // 1
            Vertex::new(cx + r, cy + r, cz - r), // 2
            Vertex::new(cx - r, cy + r, cz - r), // 3
            Vertex::new(cx - r, cy - r, cz + r), // 4
            Vertex::new(cx + r, cy - r, cz + r), // 5
            Vertex::new(cx + r, cy + r, cz + r), // 6
            Vertex::new(cx - r, cy + r, cz + r), // 7
        ],
    };

    let triangles = Triangles {
        triangle: vec![
            // Same pattern as cube
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
                v1: 4,
                v2: 6,
                v3: 5,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 4,
                v2: 7,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 5,
                v3: 1,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 4,
                v3: 5,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 3,
                v2: 2,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 3,
                v2: 6,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 3,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 7,
                v3: 4,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 1,
                v2: 5,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 1,
                v2: 6,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    };

    Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    }
}

/// Helper function to create a cylinder mesh
fn create_cylinder_mesh() -> Mesh {
    let r = 1.0_f64;
    let h = 8.0_f64;
    let cx = 5.0_f64;
    let cy = 5.0_f64;
    let cz = 1.0_f64;

    let mut vertices = Vertices { vertex: vec![] };
    let mut triangles = Triangles { triangle: vec![] };

    // Top and bottom center vertices
    vertices.vertex.push(Vertex::new(cx, cy, cz)); // 0 - bottom center
    vertices.vertex.push(Vertex::new(cx, cy, cz + h)); // 1 - top center

    // Circle vertices (8 points around)
    for i in 0..8 {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / 8.0;
        let x = cx + r * angle.cos();
        let y = cy + r * angle.sin();
        vertices.vertex.push(Vertex::new(x, y, cz)); // bottom circle vertices 2-9
        vertices.vertex.push(Vertex::new(x, y, cz + h)); // top circle vertices 10-17
    }

    // Triangles for cylinder sides and caps
    for i in 0..8 {
        let next = (i + 1) % 8;
        let bottom_curr = (2 + 2 * i) as u32;
        let bottom_next = (2 + 2 * next) as u32;
        let top_curr = (3 + 2 * i) as u32;
        let top_next = (3 + 2 * next) as u32;

        // Side triangles
        triangles.triangle.push(Triangle {
            v1: bottom_curr,
            v2: top_curr,
            v3: bottom_next,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });
        triangles.triangle.push(Triangle {
            v1: bottom_next,
            v2: top_curr,
            v3: top_next,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });

        // Bottom cap
        triangles.triangle.push(Triangle {
            v1: 0,
            v2: bottom_next,
            v3: bottom_curr,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });

        // Top cap
        triangles.triangle.push(Triangle {
            v1: 1,
            v2: top_curr,
            v3: top_next,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });
    }

    Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    }
}

#[test]
fn validate_simple_boolean_difference() {
    let cube_mesh = create_cube_mesh();
    let sphere_mesh = create_sphere_mesh();

    // Create boolean shape: Cube - Sphere
    let boolean_shape = BooleanShape {
        objectid: 1u32,
        operation: BooleanOperation::Difference,
        transform: None,
        path: None,
        booleans: vec![Boolean {
            objectid: 2u32,
            transform: None,
            path: None,
        }],
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("bo".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![
                    Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cube".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(cube_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 2,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Sphere".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(sphere_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 3,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("CubeMinusSphere".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::BooleanShape(boolean_shape)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                ],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 3,
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

    validate_boolean_model(&model_xml);
}

#[test]
fn validate_simple_boolean_intersection() {
    let cube_mesh = create_cube_mesh();
    let sphere_mesh = create_sphere_mesh();

    // Create boolean shape: Cube ∩ Sphere with base transform
    let boolean_shape = BooleanShape {
        objectid: 1u32,
        operation: BooleanOperation::Intersection,
        transform: Some(Transform([
            0.65, 0.0, 0.0, 0.0, 0.65, 0.0, 0.0, 0.0, 0.65, -37.0, 8.0, -18.0,
        ])),
        path: None,
        booleans: vec![Boolean {
            objectid: 2u32,
            transform: None,
            path: None,
        }],
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("bo".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![
                    Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cube".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(cube_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 2,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Sphere".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(sphere_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 3,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("CubeIntersectSphere".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::BooleanShape(boolean_shape)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                ],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 3,
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

    validate_boolean_model(&model_xml);
}

#[test]
fn validate_boolean_with_multiple_operands() {
    let cube_mesh = create_cube_mesh();
    let cylinder_mesh = create_cylinder_mesh();

    // Create boolean shape: Cube - (3 Cylinders with different transforms)
    let boolean_shape = BooleanShape {
        objectid: 1u32,
        operation: BooleanOperation::Difference,
        transform: None,
        path: None,
        booleans: vec![
            Boolean {
                objectid: 2u32,
                transform: Some(Transform([
                    0.1, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1, 79.0, 206.0, 43.0,
                ])),
                path: None,
            },
            Boolean {
                objectid: 2u32,
                transform: Some(Transform([
                    0.1, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1, 79.0, 129.0, 0.0,
                ])),
                path: None,
            },
            Boolean {
                objectid: 2u32,
                transform: Some(Transform([
                    0.0, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1, 42.0, 129.0, 82.0,
                ])),
                path: None,
            },
        ],
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("bo".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![
                    Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cube".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(cube_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 2,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cylinder".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(cylinder_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 3,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("CubeMinusThreeCylinders".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::BooleanShape(boolean_shape)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                ],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 3,
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

    validate_boolean_model(&model_xml);
}

#[test]
fn validate_nested_boolean_shapes() {
    let cube_mesh = create_cube_mesh();
    let sphere_mesh = create_sphere_mesh();
    let cylinder_mesh = create_cylinder_mesh();

    // Create first boolean shape: Cube ∩ Sphere
    let intersected_shape = BooleanShape {
        objectid: 1u32,
        operation: BooleanOperation::Intersection,
        transform: Some(Transform([
            0.65, 0.0, 0.0, 0.0, 0.65, 0.0, 0.0, 0.0, 0.65, -37.0, 8.0, -18.0,
        ])),
        path: None,
        booleans: vec![Boolean {
            objectid: 2u32,
            transform: Some(Transform([
                0.87, 0.0, 0.0, 0.0, 0.87, 0.0, 0.0, 0.0, 0.87, -86.0, -2.0, -36.0,
            ])),
            path: None,
        }],
    };

    // Create second boolean shape using first as base: Intersected - Cylinder
    let final_shape = BooleanShape {
        objectid: 3u32,
        operation: BooleanOperation::Difference,
        transform: None,
        path: None,
        booleans: vec![Boolean {
            objectid: 4u32,
            transform: None,
            path: None,
        }],
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("bo".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![
                    Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cube".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(cube_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 2,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Sphere".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(sphere_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 3,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Intersected".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::BooleanShape(intersected_shape)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 4,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cylinder".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(cylinder_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 5,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("FinalPart".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::BooleanShape(final_shape)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                ],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 5,
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

    validate_boolean_model(&model_xml);
}

#[test]
fn validate_boolean_with_production_extension() {
    let cube_mesh = create_cube_mesh();
    let sphere_mesh = create_sphere_mesh();

    // Create boolean shape with UUIDs
    let boolean_shape = BooleanShape {
        objectid: 1u32,
        operation: BooleanOperation::Difference,
        transform: None,
        path: None,
        booleans: vec![Boolean {
            objectid: 2u32,
            transform: None,
            path: None,
        }],
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("bo p".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![
                    Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Cube".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("11111111-1111-1111-1111-111111111111".to_owned()),
                        kind: Some(ObjectKind::Mesh(cube_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 2,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Sphere".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("22222222-2222-2222-2222-222222222222".to_owned()),
                        kind: Some(ObjectKind::Mesh(sphere_mesh)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                    Object {
                        id: 3,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("BooleanShapeWithUUID".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("33333333-3333-3333-3333-333333333333".to_owned()),
                        kind: Some(ObjectKind::BooleanShape(boolean_shape)),
                        meshresolution: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                    },
                ],
                basematerials: vec![],
                slicestack: vec![],
            },
            build: Build {
                uuid: Some("44444444-4444-4444-4444-444444444444".to_owned()),
                item: vec![Item {
                    objectid: 3,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: Some("55555555-5555-5555-5555-555555555555".to_owned()),
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

    validate_boolean_with_production_model(&model_xml);
}

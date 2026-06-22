//! Slice extension XSD validation tests
//!
//! Tests validation of 3MF models with slice data against
//! the Slice extension XSD schema.

use std::io::Cursor;
use threemf2::{
    model::{
        OptionalResourceId, OptionalResourceIndex,
        domain::{
            build::{Build, Item},
            mesh::{self, Mesh, Triangle, Triangles, Vertex, Vertices},
            model::{Model, ThreemfExtensions, Unit},
            object::{Object, ObjectKind, ObjectType},
            resources::Resources,
            slice::{self, MeshResolution, Polygon, Segment, Slice, SliceStack},
        },
    },
    package::ThreemfPackageBuilder,
    threemf_namespaces::ThreemfNamespace,
};

mod validation_utils;
use validation_utils::validation::{extract_model_xml, validate_or_panic};

const CORE_XSD: &str = include_str!("data/xsd/3mf-core-1.3.0.xsd");
const SLICE_XSD: &str = include_str!("data/xsd/3mf-slice-1.0.2.xsd");

fn validate_slice_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (threemf2::threemf_namespaces::SLICE_NS, SLICE_XSD.as_bytes()),
        ],
        "Slice Schema",
    );
}

#[test]
fn validate_simple_slice() {
    // Create vertices for mesh - need at least 4 triangles for valid 3MF
    let vertices = Vertices {
        vertex: vec![
            mesh::Vertex::new(0.0, 0.0, 0.0),
            mesh::Vertex::new(10.0, 0.0, 0.0),
            mesh::Vertex::new(10.0, 10.0, 0.0),
            mesh::Vertex::new(0.0, 10.0, 0.0),
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

    // Create slice data
    let slice_vertices = slice::Vertices {
        vertex: vec![
            slice::Vertex {
                x: 0.0.into(),
                y: 0.0.into(),
            },
            slice::Vertex {
                x: 10.0.into(),
                y: 0.0.into(),
            },
            slice::Vertex {
                x: 10.0.into(),
                y: 10.0.into(),
            },
        ],
    };

    let slice = Slice {
        ztop: 0.1.into(),
        vertices: Some(slice_vertices),
        polygon: vec![Polygon {
            startv: 0,
            segment: vec![
                Segment {
                    v2: 1,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Segment {
                    v2: 2,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        }],
    };

    let slicestack = SliceStack {
        id: 1,
        zbottom: Some(0.0.into()),
        slice: vec![slice],
        sliceref: vec![],
    };

    let object = Object {
        id: 1,
        objecttype: Some(ObjectType::Model),
        thumbnail: None,
        partnumber: None,
        name: Some("TestObject".into()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        uuid: None,
        slicestackid: OptionalResourceId::new(1),
        slicepath: None,
        meshresolution: Some(MeshResolution::LowRes),
        kind: Some(ObjectKind::Mesh(Mesh {
            vertices,
            triangles,
            trianglesets: None,
            beamlattice: None,
        })),
    };

    let model = Model {
        unit: Some(Unit::Millimeter),
        requiredextensions: ThreemfExtensions::new(&[ThreemfNamespace::Slice]),
        recommendedextensions: ThreemfExtensions::default(),
        metadata: vec![],
        resources: Resources {
            object: vec![object],
            basematerials: vec![],
            slicestack: vec![slicestack],
            colorgroup: vec![],
            compositematerials: vec![],
            texture2dgroup: vec![],
            multiproperties: vec![],
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        },
        build: Build {
            uuid: None,
            item: vec![Item {
                objectid: 1,
                transform: None,
                partnumber: None,
                uuid: None,
                path: None,
            }],
        },
    };

    // Write to buffer
    let mut buf = Cursor::new(Vec::new());
    let mut builder = ThreemfPackageBuilder::new();
    builder.set_root_model(model);
    let package = builder.build().expect("Error building package");
    package.write(&mut buf).expect("Error writing");

    // Extract and validate
    let model_xml = extract_model_xml(buf.get_ref()).unwrap();
    validate_slice_model(&model_xml);
}

#[test]
fn validate_slice_multiple_polygons() {
    // Create slice with multiple polygons
    let slice_vertices = slice::Vertices {
        vertex: vec![
            slice::Vertex {
                x: 0.0.into(),
                y: 0.0.into(),
            },
            slice::Vertex {
                x: 10.0.into(),
                y: 0.0.into(),
            },
            slice::Vertex {
                x: 10.0.into(),
                y: 10.0.into(),
            },
            slice::Vertex {
                x: 0.0.into(),
                y: 10.0.into(),
            },
            slice::Vertex {
                x: 2.0.into(),
                y: 2.0.into(),
            },
            slice::Vertex {
                x: 8.0.into(),
                y: 2.0.into(),
            },
            slice::Vertex {
                x: 8.0.into(),
                y: 8.0.into(),
            },
            slice::Vertex {
                x: 2.0.into(),
                y: 8.0.into(),
            },
        ],
    };

    let slice = Slice {
        ztop: 0.1.into(),
        vertices: Some(slice_vertices),
        polygon: vec![
            // Outer polygon
            Polygon {
                startv: 0,
                segment: vec![
                    Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Segment {
                        v2: 2,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Segment {
                        v2: 3,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                ],
            },
            // Inner polygon (hole)
            Polygon {
                startv: 4,
                segment: vec![
                    Segment {
                        v2: 5,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Segment {
                        v2: 6,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Segment {
                        v2: 7,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                ],
            },
        ],
    };

    let slicestack = SliceStack {
        id: 1,
        zbottom: Some(0.0.into()),
        slice: vec![slice],
        sliceref: vec![],
    };

    let object = Object {
        id: 1,
        objecttype: Some(ObjectType::Model),
        thumbnail: None,
        partnumber: None,
        name: Some("MultiPolygonSlice".into()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        uuid: None,
        slicestackid: OptionalResourceId::new(1),
        slicepath: None,
        meshresolution: Some(MeshResolution::FullRes),
        kind: Some(ObjectKind::Mesh(Mesh {
            vertices: Vertices {
                vertex: vec![
                    Vertex::new(0.0, 0.0, 0.0),
                    Vertex::new(10.0, 0.0, 0.0),
                    Vertex::new(10.0, 10.0, 0.0),
                    Vertex::new(0.0, 10.0, 0.0),
                ],
            },
            triangles: Triangles {
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
            },
            trianglesets: None,
            beamlattice: None,
        })),
    };

    let model = Model {
        unit: Some(Unit::Millimeter),
        requiredextensions: ThreemfExtensions::default(), // FullRes doesn't require slice extension
        recommendedextensions: ThreemfExtensions::new(&[ThreemfNamespace::Slice]),
        metadata: vec![],
        resources: Resources {
            object: vec![object],
            basematerials: vec![],
            slicestack: vec![slicestack],
            colorgroup: vec![],
            compositematerials: vec![],
            texture2dgroup: vec![],
            multiproperties: vec![],
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        },
        build: Build {
            uuid: None,
            item: vec![Item {
                objectid: 1,
                transform: None,
                partnumber: None,
                uuid: None,
                path: None,
            }],
        },
    };

    // Write and validate
    let mut buf = Cursor::new(Vec::new());
    let mut builder = ThreemfPackageBuilder::new();
    builder.set_root_model(model);
    let package = builder.build().expect("Error building package");
    package.write(&mut buf).expect("Error writing");

    let model_xml = extract_model_xml(buf.get_ref()).unwrap();
    validate_slice_model(&model_xml);
}

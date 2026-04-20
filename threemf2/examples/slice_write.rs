use threemf2::{
    core::{
        OptionalResourceId, OptionalResourceIndex,
        build::{Build, Item},
        mesh::{self, Mesh, Triangle, Triangles, Vertices},
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
        slice::{self, MeshResolution, Polygon, Segment, Slice, SliceStack},
    },
    io::{
        ThreemfPackage,
        content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
        relationship::{Relationship, RelationshipType, Relationships},
    },
};

use std::collections::HashMap;
use std::{io::Cursor, vec};

/// This example shows how to create and write a 3MF file with slice extension data.
/// This demonstrates the Slice Extension support in threemf2.
///
/// Run with:
/// `cargo run --example slice_write --no-default-features --features io-write`
///
fn main() {
    // Create vertices for a simple cube mesh
    let vertices = Vertices {
        vertex: vec![
            mesh::Vertex::new(0.0, 0.0, 0.0),
            mesh::Vertex::new(10.0, 0.0, 0.0),
            mesh::Vertex::new(10.0, 10.0, 0.0),
            mesh::Vertex::new(0.0, 10.0, 0.0),
            mesh::Vertex::new(0.0, 0.0, 10.0),
            mesh::Vertex::new(10.0, 0.0, 10.0),
            mesh::Vertex::new(10.0, 10.0, 10.0),
            mesh::Vertex::new(0.0, 10.0, 10.0),
        ],
    };

    // Create triangles for the mesh
    let triangles = Triangles {
        triangle: vec![
            // Bottom face
            Triangle {
                v1: 0,
                v2: 2,
                v3: 1,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 3,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Top face
            Triangle {
                v1: 4,
                v2: 5,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 4,
                v2: 6,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    };

    // Create slice vertices for the first layer
    let slice_vertices_1 = slice::Vertices {
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
        ],
    };

    // Create first slice at z=0.1
    let slice_1 = Slice {
        ztop: 0.1.into(),
        vertices: Some(slice_vertices_1),
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
                Segment {
                    v2: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        }],
    };

    // Create slice vertices for the second layer
    let slice_vertices_2 = slice::Vertices {
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
        ],
    };

    // Create second slice at z=0.2
    let slice_2 = Slice {
        ztop: 0.2.into(),
        vertices: Some(slice_vertices_2),
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
                Segment {
                    v2: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        }],
    };

    // Create the slice stack
    let slicestack = SliceStack {
        id: 1,
        zbottom: Some(0.0.into()),
        slice: vec![slice_1, slice_2],
        sliceref: vec![],
    };

    // Create the object with mesh and slice reference
    let object = Object {
        id: 1,
        objecttype: Some(ObjectType::Model),
        thumbnail: None,
        partnumber: Some("SLICE_EXAMPLE_001".to_owned()),
        name: Some("SlicedCube".to_owned()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        uuid: None,
        slicestackid: OptionalResourceId::new(1), // References slice stack with id=1
        slicepath: None,                          // Slice stack is in the same file
        meshresolution: Some(MeshResolution::LowRes), // Mesh is low resolution
        kind: Some(ObjectKind::Mesh(Mesh {
            vertices,
            triangles,
            trianglesets: None,
            beamlattice: None,
        })),
    };

    // Create resources
    let resources = Resources {
        object: vec![object],
        basematerials: vec![],
        slicestack: vec![slicestack],
    };

    // Create build section
    let build = Build {
        uuid: None,
        item: vec![Item {
            objectid: 1,
            transform: None,
            partnumber: None,
            path: None,
            uuid: None,
        }],
    };

    // Create the model
    let model = Model {
        unit: Some(Unit::Millimeter),
        requiredextensions: Some("s ".to_owned()), // Slice extension is required
        recommendedextensions: None,
        metadata: vec![],
        resources,
        build,
    };

    // Create content types
    let content_types = ContentTypes {
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
    };

    // Create relationships
    let relationships = HashMap::from([(
        "_rels/.rels".to_owned(),
        Relationships {
            relationships: vec![Relationship {
                id: "rel0".to_owned(),
                target: "3D/3Dmodel.model".to_owned(),
                relationship_type: RelationshipType::Model,
            }],
        },
    )]);

    // Create the package
    let package = ThreemfPackage::new(
        model,
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        relationships,
        content_types,
    );

    // Write to buffer (in real usage, this would be a file)
    let mut buf = Cursor::new(Vec::new());
    package.write(&mut buf).expect("Error writing 3MF package");

    println!("Successfully created 3MF file with slice extension!");
    println!("Package size: {} bytes", buf.get_ref().len());
    println!();
    println!("This example demonstrates:");
    println!("  - Creating a SliceStack with multiple 2D slices");
    println!("  - Defining vertices and polygons for each slice layer");
    println!("  - Referencing slice data from an object");
    println!("  - Setting meshresolution to 'lowres' (requires slice extension)");
    println!("  - Automatic namespace and requiredextensions generation");
}

//! Example: Creating a 3MF file with Boolean Operations
//!
//! This example demonstrates how to use the Boolean Operations extension
//! to create complex shapes through Constructive Solid Geometry (CSG).
//!
//! The example creates:
//! - A cube mesh (base object)
//! - A sphere mesh (operand)
//! - A boolean shape that subtracts the sphere from the cube
//!
//! # Running the example
//!
//! ```bash
//! cargo run --example boolean --features "io-write"
//! ```

use std::fs::File;
use std::io::BufWriter;

use threemf2::{
    core::{
        OptionalResourceId, OptionalResourceIndex,
        boolean::{Boolean, BooleanOperation, BooleanShape},
        build::{Build, Item},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        metadata::Metadata,
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
    },
    io::ThreemfPackage,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating 3MF file with boolean operations...");

    // Create a simple cube mesh (base object for boolean operation)
    let cube_mesh_1 = Mesh {
        vertices: Vertices {
            vertex: vec![
                // Bottom face (z = 0)
                Vertex::new(-5.0, -5.0, 0.0), // 0
                Vertex::new(5.0, -5.0, 0.0),  // 1
                Vertex::new(5.0, 5.0, 0.0),   // 2
                Vertex::new(-5.0, 5.0, 0.0),  // 3
                // Top face (z = 10)
                Vertex::new(-5.0, -5.0, 10.0), // 4
                Vertex::new(5.0, -5.0, 10.0),  // 5
                Vertex::new(5.0, 5.0, 10.0),   // 6
                Vertex::new(-5.0, 5.0, 10.0),  // 7
            ],
        },
        triangles: Triangles {
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
                    v1: 2,
                    v2: 7,
                    v3: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 2,
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
                    v2: 7,
                    v3: 4,
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
                // Right face
                Triangle {
                    v1: 1,
                    v2: 6,
                    v3: 2,
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
            ],
        },
        trianglesets: None,
        beamlattice: None,
    };

    // Create another cube mesh (operand for boolean operation)
    let cube_mesh_2 = Mesh {
        vertices: Vertices {
            vertex: vec![
                // Bottom face (z = 3)
                Vertex::new(-2.0, -2.0, 3.0), // 0
                Vertex::new(2.0, -2.0, 3.0),  // 1
                Vertex::new(2.0, 2.0, 3.0),   // 2
                Vertex::new(-2.0, 2.0, 3.0),  // 3
                // Top face (z = 7)
                Vertex::new(-2.0, -2.0, 7.0), // 4
                Vertex::new(2.0, -2.0, 7.0),  // 5
                Vertex::new(2.0, 2.0, 7.0),   // 6
                Vertex::new(-2.0, 2.0, 7.0),  // 7
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
                    v1: 2,
                    v2: 7,
                    v3: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 2,
                    v2: 6,
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
                    v1: 0,
                    v2: 3,
                    v3: 7,
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
                Triangle {
                    v1: 1,
                    v2: 5,
                    v3: 6,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        },
        trianglesets: None,
        beamlattice: None,
    };

    // Create the boolean shape: cube minus sphere (difference operation)
    let boolean_shape = BooleanShape {
        objectid: 1, // References cube object
        operation: BooleanOperation::Difference,
        transform: None,
        path: None,
        booleans: vec![Boolean {
            objectid: 2, // References sphere object
            transform: None,
            path: None,
        }],
    };

    // Create objects
    let cube_object_1 = Object {
        id: 1,
        objecttype: Some(ObjectType::Model),
        thumbnail: None,
        partnumber: None,
        name: Some("Cube".to_string()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        uuid: None,
        kind: Some(ObjectKind::Mesh(cube_mesh_1)),
        // mesh: Some(cube_mesh_1),
    };

    let cube_object_2 = Object {
        id: 2,
        objecttype: Some(ObjectType::Model),
        thumbnail: None,
        partnumber: None,
        name: Some("Sphere".to_string()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        uuid: None,
        kind: Some(ObjectKind::Mesh(cube_mesh_2)),
        // mesh: Some(cube_mesh_2),
    };

    let result_object = Object {
        id: 3,
        objecttype: Some(ObjectType::Model),
        thumbnail: None,
        partnumber: None,
        name: Some("CubeMinusSphere".to_string()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        uuid: None,
        kind: Some(ObjectKind::BooleanShape(boolean_shape)),
    };

    // Create the model
    let model = Model {
        unit: Some(Unit::Millimeter),
        requiredextensions: Some("bo".to_string()), // Boolean extension is required
        recommendedextensions: None,
        metadata: vec![
            Metadata {
                name: "Application".to_string(),
                preserve: None,
                value: Some("Boolean Example".to_string()),
            },
            Metadata {
                name: "Description".to_string(),
                preserve: None,
                value: Some("Cube 1 minus Cube 1 using CSG".to_string()),
            },
        ],
        resources: Resources {
            object: vec![cube_object_1, cube_object_2, result_object],
            basematerials: vec![],
        },
        build: Build {
            uuid: None,
            item: vec![Item {
                objectid: 3, // Build the boolean result
                transform: None,
                partnumber: None,
                path: None,
                uuid: None,
            }],
        },
    };

    println!("  Created cube mesh 1 (object id: 1)");
    println!("  Created cube mesh 2 (object id: 2)");
    println!("  Created boolean shape: cube 1 - cube 2 (object id: 3)");
    println!("  Added boolean result to build plate");

    println!("\nModel statistics:");
    println!("  Objects: {}", model.resources.object.len());
    println!("  Build items: {}", model.build.item.len());
    if let Some(ref extensions) = model.requiredextensions {
        println!("  Required extensions: {}", extensions);
    }

    // Create 3MF package and write to file
    let package: ThreemfPackage = model.into();

    let output_path = "boolean_example.3mf";
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);

    package.write(writer)?;

    println!("\nSuccessfully wrote 3MF file: {}", output_path);
    println!("\nNote: This example uses simple cubes to approximate geometry.");

    Ok(())
}

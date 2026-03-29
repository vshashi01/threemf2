use threemf2::{
    core::{
        OptionalResourceId, OptionalResourceIndex,
        beamlattice::{Ball, BallMode, Balls, Beam, BeamLattice, Beams, CapMode},
        build::{Build, Item},
        mesh::{Mesh, Triangles, Vertex, Vertices},
        metadata::Metadata,
        model::{Model, Unit},
        object::{Object, ObjectType},
        object_kind::ObjectKind,
        resources::Resources,
    },
    io::ThreemfPackage,
};

use std::{io::Cursor, vec};

/// This example shows how to create and write a 3MF file with beam lattice structures.
/// This demonstrates the Beam Lattice Extension support in threemf2.
///
/// Run with:
/// `cargo run --example beamlattice_write --no-default-features --features io-write`
///
fn main() {
    // Create vertices for a simple cube structure
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(45.0, 55.0, 55.0),
            Vertex::new(45.0, 45.0, 55.0),
            Vertex::new(45.0, 55.0, 45.0),
            Vertex::new(45.0, 45.0, 45.0),
            Vertex::new(55.0, 55.0, 45.0),
            Vertex::new(55.0, 55.0, 55.0),
            Vertex::new(55.0, 45.0, 55.0),
            Vertex::new(55.0, 45.0, 45.0),
        ],
    };

    // Create beams connecting the vertices
    let beams = Beams {
        beam: vec![
            Beam {
                v1: 0,
                v2: 1,
                r1: Some(1.5),
                r2: Some(1.6),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 2,
                v2: 0,
                r1: Some(3.0),
                r2: Some(1.5),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 1,
                v2: 3,
                r1: Some(1.6),
                r2: Some(3.0),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 3,
                v2: 2,
                r1: Some(3.0),
                r2: None,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 2,
                v2: 4,
                r1: Some(3.0),
                r2: Some(2.0),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 4,
                v2: 5,
                r1: Some(2.0),
                r2: None,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 5,
                v2: 6,
                r1: Some(2.0),
                r2: None,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 7,
                v2: 6,
                r1: Some(2.0),
                r2: None,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 1,
                v2: 6,
                r1: Some(1.6),
                r2: Some(2.0),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 7,
                v2: 4,
                r1: Some(2.0),
                r2: None,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 7,
                v2: 3,
                r1: Some(2.0),
                r2: Some(3.0),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
            Beam {
                v1: 0,
                v2: 5,
                r1: Some(1.5),
                r2: Some(2.0),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            },
        ],
    };

    // Create balls at some vertices for a mixed ball mode example
    let balls = Some(Balls {
        ball: vec![
            Ball {
                vindex: 0,
                r: Some(0.5),
                p: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Ball {
                vindex: 5,
                r: None,
                p: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }, // Uses default ballradius
            Ball {
                vindex: 7,
                r: Some(0.5),
                p: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    });

    // Create the beam lattice
    let beam_lattice = BeamLattice {
        minlength: 0.0001,
        radius: 1.0,
        ballmode: Some(BallMode::Mixed),
        ballradius: Some(0.25),
        clippingmode: None,
        clippingmesh: OptionalResourceId::none(),
        representationmesh: OptionalResourceId::none(),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        cap: Some(CapMode::Sphere),
        beams,
        balls,
        beamsets: None,
    };

    // Create a mesh with the beam lattice (no triangles for lattice-only object)
    let mesh = Mesh {
        vertices,
        triangles: Triangles { triangle: vec![] },
        trianglesets: None,
        beamlattice: Some(beam_lattice),
    };

    // Create the complete 3MF model
    let model = Model {
        unit: Some(Unit::Millimeter),
        requiredextensions: Some("b b2".to_string()), // Mark beam lattice extensions as required
        recommendedextensions: None,
        metadata: vec![Metadata {
            name: "Beam Lattice Example".to_string(),
            preserve: None,
            value: None,
        }],
        resources: Resources {
            object: vec![Object {
                id: 1,
                objecttype: Some(ObjectType::Model),
                thumbnail: None,
                partnumber: Some("beam-lattice-example".to_string()),
                name: Some("Beam Lattice Cube".to_string()),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                kind: Some(ObjectKind::Mesh(mesh)), // mesh: Some(mesh),
                                                    // components: None,
                                                    // booleanshape: None,
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
    };

    let package: ThreemfPackage = model.into();

    let mut bytes: Vec<u8> = vec![];
    let writer = Cursor::new(&mut bytes);

    let result = package.write(writer);

    match result {
        Ok(_) => {
            println!("Successfully created 3MF file with beam lattice!");
            println!("File size: {} bytes", bytes.len());
            println!("The file contains:");
            println!("  - {} beams with variable radii", 12);
            println!("  - {} balls at select vertices", 3);
            println!("  - Mixed ball mode with sphere capping");
            println!("  - Minimum beam length: 0.0001 mm");
            println!("  - Default beam radius: 1.0 mm");
            println!("  - Default ball radius: 0.25 mm");
        }
        Err(err) => println!("Error writing beam lattice 3MF file: {:?}", err),
    }
}

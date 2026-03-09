use threemf2::core::{
    build::{Build, Item},
    mesh::Mesh,
    mesh::Triangle,
    mesh::Triangles,
    mesh::Vertex,
    mesh::Vertices,
    metadata::Metadata,
    model::{Model, Unit},
    object::{Object, ObjectType},
    resources::Resources,
    types::OptionalResourceIndex,
};

use std::vec;

/// This is an example showing the core 3MF Types available without any default features.
/// With no default features only the core structs are available from this library.
/// run with
/// `cargo run --example core-no-default-features --no-default-features`
///
fn main() {
    let model = Model {
        unit: Some(Unit::Inch),
        requiredextensions: None,
        recommendedextensions: None,
        metadata: vec![Metadata {
            name: "Test metadata".to_string(),
            preserve: None,
            value: None,
        }],
        resources: Resources {
            object: vec![Object {
                id: 1,
                objecttype: Some(ObjectType::Model),
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: None,
                pindex: None,
                uuid: None,
                mesh: Some(Mesh {
                    vertices: Vertices {
                        vertex: vec![
                            Vertex {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            },
                            Vertex {
                                x: -1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                            Vertex {
                                x: -1.0,
                                y: 1.0,
                                z: 0.0,
                            },
                        ],
                    },
                    triangles: Triangles {
                        triangle: vec![Triangle {
                            v1: 0,
                            v2: 2,
                            v3: 3,
                            p1: OptionalResourceIndex::none(),
                            p2: OptionalResourceIndex::none(),
                            p3: OptionalResourceIndex::none(),
                            pid: None,
                        }],
                    },
                    trianglesets: None,
                    // #[cfg(feature = "beam-lattice")]
                    beamlattice: None,
                }),
                components: None,
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

    println!("Number of build items: {}", model.build.item.len());
}

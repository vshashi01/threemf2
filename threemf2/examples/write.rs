use threemf2::{
    core::{
        OptionalResourceId,
        build::{Build, Item},
        mesh::*,
        metadata::Metadata,
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
        types::OptionalResourceIndex,
    },
    io::ThreemfPackage,
};

use std::{io::Cursor, vec};

/// This is an example showing the core 3MF Types available without any default features.
/// With no default features only the core structs are available from this library.
/// run with
/// `cargo run --example write --no-default-features --features io-write`
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
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
                kind: Some(ObjectKind::Mesh(Mesh {
                    vertices: Vertices {
                        vertex: vec![
                            Vertex::new(0.0, 0.0, 0.0),
                            Vertex::new(-1.0, 0.0, 0.0),
                            Vertex::new(-1.0, 1.0, 0.0),
                        ],
                    },
                    triangles: Triangles {
                        triangle: vec![Triangle {
                            v1: 0,
                            v2: 1,
                            v3: 2,
                            p1: OptionalResourceIndex::none(),
                            p2: OptionalResourceIndex::none(),
                            p3: OptionalResourceIndex::none(),
                            pid: OptionalResourceId::none(),
                        }],
                    },
                    trianglesets: None,
                    beamlattice: None,
                })),
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
    };

    let package: ThreemfPackage = model.into();

    let mut bytes: Vec<u8> = vec![];
    let writer = Cursor::new(&mut bytes);

    let result = package.write(writer);

    match result {
        Ok(_) => println!("The length of the ThreemfPackage in bytes: {}", bytes.len()),
        Err(err) => println!("Error writing the model into a ThreemfPackage: {:?}", err),
    }
}

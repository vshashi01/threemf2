//! Beam Lattice extension XSD validation tests
//!
//! Tests validation of 3MF models with beam lattice structures against
//! the Beam Lattice extension XSD schemas.

use std::collections::HashMap;
use std::io::Cursor;
use threemf2::{
    core::{
        OptionalResourceId,
        beamlattice::{Ball, BallMode, Balls, Beam, BeamLattice, Beams, CapMode},
        build::{Build, Item},
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
const BEAM_LATTICE_XSD: &str = include_str!("data/xsd/3mf-beamlattice-1.2.0.xsd");
const BEAM_LATTICE_BALLS_XSD: &str = include_str!("data/xsd/3mf-beamlattice-balls-2020-07.xsd");

fn validate_beamlattice_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::BEAM_LATTICE_NS,
                BEAM_LATTICE_XSD.as_bytes(),
            ),
            (
                threemf2::threemf_namespaces::BEAM_LATTICE_BALLS_NS,
                BEAM_LATTICE_BALLS_XSD.as_bytes(),
            ),
        ],
        "Beam Lattice Schema",
    );
}

#[test]
fn validate_simple_beamlattice() {
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

    let beamlattice = BeamLattice {
        minlength: 0.0001,
        radius: 1.0,
        cap: Some(CapMode::Sphere),
        beams: Beams {
            beam: vec![
                Beam {
                    v1: 0,
                    v2: 1,
                    r1: Some(0.5),
                    r2: Some(0.8),
                    cap1: None,
                    cap2: None,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Beam {
                    v1: 1,
                    v2: 2,
                    r1: None,
                    r2: None,
                    cap1: None,
                    cap2: None,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Beam {
                    v1: 2,
                    v2: 3,
                    r1: Some(1.2),
                    r2: None,
                    cap1: Some(CapMode::Hemisphere),
                    cap2: Some(CapMode::Butt),
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        },
        balls: None,
        ballmode: None,
        ballradius: None,
        beamsets: None,
        clippingmode: None,
        clippingmesh: OptionalResourceId::none(),
        representationmesh: OptionalResourceId::none(),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: None,
        beamlattice: Some(beamlattice),
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("b".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Beam Lattice Mesh".to_owned()),
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

    validate_beamlattice_model(&model_xml);
}

#[test]
fn validate_beamlattice_with_balls() {
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
            };
            4
        ],
    };

    let beamlattice = BeamLattice {
        minlength: 0.0001,
        radius: 1.0,
        cap: Some(CapMode::Sphere),
        beams: Beams {
            beam: vec![
                Beam {
                    v1: 0,
                    v2: 1,
                    r1: Some(0.5),
                    r2: Some(0.8),
                    cap1: None,
                    cap2: None,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Beam {
                    v1: 1,
                    v2: 2,
                    r1: None,
                    r2: None,
                    cap1: None,
                    cap2: None,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        },
        ballmode: Some(BallMode::Mixed),
        ballradius: Some(1.0),
        balls: Some(Balls {
            ball: vec![
                Ball {
                    vindex: 0,
                    r: Some(0.6),
                    p: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Ball {
                    vindex: 1,
                    r: None, // Uses default ballradius
                    p: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        }),
        beamsets: None,
        clippingmode: None,
        clippingmesh: OptionalResourceId::none(),
        representationmesh: OptionalResourceId::none(),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: None,
        beamlattice: Some(beamlattice),
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("b b2".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Beam Lattice with Balls".to_owned()),
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

    validate_beamlattice_model(&model_xml);
    // Validate against all schemas
    // validate_or_panic(&model_xml, CORE_XSD, "3MF Core Schema");
    // validate_or_panic(&model_xml, BEAM_LATTICE_XSD, "Beam Lattice Schema");
    // validate_or_panic(
    //     &model_xml,
    //     BEAM_LATTICE_BALLS_XSD,
    //     "Beam Lattice Balls Schema",
    // );
}

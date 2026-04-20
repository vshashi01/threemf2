//! Combined extension XSD validation tests
//!
//! Tests validation of 3MF models that use multiple extensions simultaneously.

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
        triangle_set::{TriangleRef, TriangleSet, TriangleSets},
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
const TRIANGLE_SETS_XSD: &str = include_str!("data/xsd/3mf-trianglesets-2021-07.xsd");
const PRODUCTION_XSD: &str = include_str!("data/xsd/3mf-production-1.2.0.xsd");

fn validate_combined_model(model_xml: &str) {
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
            (
                threemf2::threemf_namespaces::CORE_TRIANGLESET_NS,
                TRIANGLE_SETS_XSD.as_bytes(),
            ),
            (
                threemf2::threemf_namespaces::PROD_NS,
                PRODUCTION_XSD.as_bytes(),
            ),
        ],
        "All Schema",
    );
}

#[test]
fn validate_beamlattice_with_trianglesets() {
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
            beam: vec![Beam {
                v1: 0,
                v2: 1,
                r1: Some(0.5),
                r2: Some(0.8),
                cap1: None,
                cap2: None,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }],
        },
        ballmode: Some(BallMode::Mixed),
        ballradius: Some(0.3),
        balls: Some(Balls {
            ball: vec![Ball {
                vindex: 0,
                r: Some(0.5),
                p: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }],
        }),
        beamsets: None,
        clippingmode: None,
        clippingmesh: OptionalResourceId::none(),
        representationmesh: OptionalResourceId::none(),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
    };

    let triangle_sets = TriangleSets {
        trianglesets: vec![
            TriangleSet {
                name: "Beam Lattice Surface".to_owned(),
                identifier: "urn:3mf:example:beams".to_owned(),
                triangle_ref: vec![TriangleRef { index: 0 }],
                triangle_refrange: vec![],
            },
            TriangleSet {
                name: "Mesh Surface".to_owned(),
                identifier: "urn:3mf:example:mesh".to_owned(),
                triangle_ref: vec![TriangleRef { index: 1 }],
                triangle_refrange: vec![],
            },
        ],
    };

    let mesh = Mesh {
        triangles,
        vertices,
        trianglesets: Some(triangle_sets),
        beamlattice: Some(beamlattice),
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("b b2".to_owned()),
            recommendedextensions: Some("t".to_owned()),
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Beam Lattice with Triangle Sets".to_owned()),
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

    validate_combined_model(&model_xml);
}

//! Displacement extension XSD validation tests

use std::io::Cursor;

use threemf2::{
    core::{
        OptionalResourceId, OptionalResourceIndex, PathResource,
        build::{Build, Item},
        displacement::{
            Disp2DCoord, Disp2DGroup, Displacement2D, DisplacementMesh, NormVector,
            NormVectorGroup, Triangle, Triangles, Vertex, Vertices,
        },
        model::{Model, ThreemfExtensions, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
    },
    io::ThreemfPackageBuilder,
    threemf_namespaces::ThreemfNamespace,
};

mod validation_utils;
use validation_utils::validation::{extract_model_xml, validate_or_panic};

const CORE_XSD: &str = include_str!("data/xsd/3mf-core-1.3.0.xsd");
const DISPLACEMENT_XSD: &str = include_str!("data/xsd/3mf-displacement-1.0.0.xsd");

#[test]
fn validate_displacement_model_schema() {
    let model = Model {
        unit: Some(Unit::Millimeter),
        requiredextensions: ThreemfExtensions::new(&[ThreemfNamespace::Displacement]),
        recommendedextensions: ThreemfExtensions::default(),
        metadata: vec![],
        resources: Resources {
            object: vec![Object {
                id: 1,
                objecttype: Some(ObjectType::Model),
                thumbnail: None,
                partnumber: None,
                name: Some("Disp".into()),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                kind: Some(ObjectKind::DisplacementMesh(DisplacementMesh {
                    vertices: Vertices {
                        vertex: vec![
                            Vertex {
                                x: 0.0.into(),
                                y: 0.0.into(),
                                z: 0.0.into(),
                            },
                            Vertex {
                                x: 1.0.into(),
                                y: 0.0.into(),
                                z: 0.0.into(),
                            },
                            Vertex {
                                x: 0.0.into(),
                                y: 1.0.into(),
                                z: 0.0.into(),
                            },
                            Vertex {
                                x: 0.0.into(),
                                y: 0.0.into(),
                                z: 1.0.into(),
                            },
                        ],
                    },
                    triangles: Triangles {
                        did: OptionalResourceId::new(7),
                        triangle: vec![
                            Triangle {
                                v1: 0,
                                v2: 1,
                                v3: 2,
                                d1: OptionalResourceIndex::new(0),
                                d2: OptionalResourceIndex::new(1),
                                d3: OptionalResourceIndex::new(2),
                                did: OptionalResourceId::none(),
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                p3: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                            },
                            Triangle {
                                v1: 0,
                                v2: 1,
                                v3: 3,
                                d1: OptionalResourceIndex::new(0),
                                d2: OptionalResourceIndex::new(1),
                                d3: OptionalResourceIndex::new(2),
                                did: OptionalResourceId::none(),
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                p3: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                            },
                            Triangle {
                                v1: 0,
                                v2: 2,
                                v3: 3,
                                d1: OptionalResourceIndex::new(0),
                                d2: OptionalResourceIndex::new(1),
                                d3: OptionalResourceIndex::new(2),
                                did: OptionalResourceId::new(7),
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                p3: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                            },
                            Triangle {
                                v1: 1,
                                v2: 2,
                                v3: 3,
                                d1: OptionalResourceIndex::new(0),
                                d2: OptionalResourceIndex::new(1),
                                d3: OptionalResourceIndex::new(2),
                                did: OptionalResourceId::new(7),
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
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,
            }],
            basematerials: vec![],
            slicestack: vec![],
            colorgroup: vec![],
            texture2dgroup: vec![],
            compositematerials: vec![],
            multiproperties: vec![],
            texture2d: vec![],
            displacement2d: vec![Displacement2D {
                id: 3,
                path: PathResource::try_from("/3D/Textures/displacement.png").unwrap(),
                channel: None,
                tilestyleu: None,
                tilestylev: None,
                filter: None,
            }],
            normvectorgroup: vec![NormVectorGroup {
                id: 5,
                normvector: vec![
                    NormVector {
                        x: 0.0.into(),
                        y: 0.0.into(),
                        z: 1.0.into(),
                    },
                    NormVector {
                        x: 0.0.into(),
                        y: 1.0.into(),
                        z: 0.0.into(),
                    },
                    NormVector {
                        x: 1.0.into(),
                        y: 0.0.into(),
                        z: 0.0.into(),
                    },
                ],
            }],
            disp2dgroup: vec![Disp2DGroup {
                id: 7,
                dispid: 3,
                nid: 5,
                height: 1.5.into(),
                offset: Some(0.0.into()),
                disp2dcoord: vec![
                    Disp2DCoord {
                        u: 0.0.into(),
                        v: 0.0.into(),
                        n: 0,
                        f: None,
                    },
                    Disp2DCoord {
                        u: 1.0.into(),
                        v: 0.0.into(),
                        n: 1,
                        f: None,
                    },
                    Disp2DCoord {
                        u: 0.0.into(),
                        v: 1.0.into(),
                        n: 2,
                        f: Some(1.0.into()),
                    },
                ],
            }],
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

    let mut builder = ThreemfPackageBuilder::new();
    builder.set_root_model(model);
    let package = builder.build().expect("Error building package");

    let mut buf = Cursor::new(Vec::new());
    package.write(&mut buf).expect("Error writing package");

    let model_xml =
        extract_model_xml(buf.get_ref()).expect("Failed to extract model XML from package");

    validate_or_panic(
        &model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::DISPLACEMENT_NS,
                DISPLACEMENT_XSD.as_bytes(),
            ),
        ],
        "Displacement extension schema",
    );
}

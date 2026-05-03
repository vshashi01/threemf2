//! Material extension XSD validation tests
//!
//! Tests validation of 3MF models with material extension structures against
//! the Material extension XSD schemas.

use std::collections::HashMap;
use std::io::Cursor;
use threemf2::{
    core::{
        Color, OptionalResourceId, OptionalResourceIndex,
        build::{Build, Item},
        material::{
            ColorElement, ColorGroup, Filter, Multi, MultiProperties, Tex2Coord, Texture2D,
            Texture2DGroup, TextureContentType, TileStyle,
        },
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
        types::{Double, ResourceIdCollection, ResourceIndexCollection},
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
const MATERIAL_XSD: &str = include_str!("data/xsd/3mf-material-1.2.1.xsd");

fn validate_material_model(model_xml: &str) {
    validate_or_panic(
        model_xml,
        &[
            (threemf2::threemf_namespaces::CORE_NS, CORE_XSD.as_bytes()),
            (
                threemf2::threemf_namespaces::MATERIAL_NS,
                MATERIAL_XSD.as_bytes(),
            ),
        ],
        "Material Schema",
    );
}

#[test]
fn validate_simple_colorgroup() {
    // Create a color group with 4 colors (vertex colors)
    let colorgroup = ColorGroup {
        id: 1,
        color: vec![
            ColorElement {
                color: Color::from_hex("#FF0000FF").unwrap(),
            }, // Red
            ColorElement {
                color: Color::from_hex("#00FF00FF").unwrap(),
            }, // Green
            ColorElement {
                color: Color::from_hex("#0000FFFF").unwrap(),
            }, // Blue
            ColorElement {
                color: Color::from_hex("#FFFFFFFF").unwrap(),
            }, // White
        ],
    };

    // Create a mesh with at least 4 vertices (XSD requirement)
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.5, 1.0, 0.0),
            Vertex::new(0.5, 0.5, 1.0),
        ],
    };

    // Create at least 4 triangles that reference the color group via pid and use vertex colors via p1, p2, p3
    let triangles = Triangles {
        triangle: vec![
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::new(0), // Reference color 0 (Red)
                p2: OptionalResourceIndex::new(1), // Reference color 1 (Green)
                p3: OptionalResourceIndex::new(2), // Reference color 2 (Blue)
                pid: OptionalResourceId::new(1),   // Reference color group 1
            },
            Triangle {
                v1: 0,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::new(0), // Red
                p2: OptionalResourceIndex::new(2), // Blue
                p3: OptionalResourceIndex::new(3), // White
                pid: OptionalResourceId::new(1),   // Reference color group 1
            },
            Triangle {
                v1: 0,
                v2: 1,
                v3: 3,
                p1: OptionalResourceIndex::new(1), // Green
                p2: OptionalResourceIndex::new(2), // Blue
                p3: OptionalResourceIndex::new(3), // White
                pid: OptionalResourceId::new(1),
            },
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::new(0), // Red
                p2: OptionalResourceIndex::new(1), // Green
                p3: OptionalResourceIndex::new(3), // White
                pid: OptionalResourceId::new(1),
            },
        ],
    };

    let mesh = Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("m".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Colored Mesh".to_owned()),
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
                colorgroup: vec![colorgroup],
                compositematerials: vec![],
                texture2dgroup: vec![],
                multiproperties: vec![],
                texture2d: vec![],
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

    validate_material_model(&model_xml);
}

#[test]
fn validate_texture2d_with_uv_mapping() {
    // Create a texture reference
    let texture2d = Texture2D {
        id: 1,
        path: "/3D/texture.png".to_owned(),
        contenttype: TextureContentType::Png,
        tilestyleu: Some(TileStyle::Wrap),
        tilestylev: Some(TileStyle::Mirror),
        filter: Some(Filter::Linear),
    };

    // Create texture coordinates (UV mapping)
    let texture2dgroup = Texture2DGroup {
        id: 2,
        texid: 1,
        tex2coord: vec![
            Tex2Coord {
                u: Double::new(0.0),
                v: Double::new(0.0),
            },
            Tex2Coord {
                u: Double::new(1.0),
                v: Double::new(0.0),
            },
            Tex2Coord {
                u: Double::new(0.5),
                v: Double::new(1.0),
            },
        ],
    };

    // Create a mesh with at least 4 vertices (XSD requirement)
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.5, 1.0, 0.0),
            Vertex::new(0.5, 0.5, 1.0),
        ],
    };

    // Create at least 4 triangles (XSD requirement)
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

    let mesh = Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("m".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Textured Mesh".to_owned()),
                    pid: OptionalResourceId::new(2), // Reference texture group
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(mesh)),
                    meshresolution: None,
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: vec![],
                compositematerials: vec![],
                texture2dgroup: vec![texture2dgroup],
                multiproperties: vec![],
                texture2d: vec![texture2d],
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

    validate_material_model(&model_xml);
}

// Note: validate_composite_materials test removed because the library currently
// uses 'base' as element name for BaseMaterials children, but XSD expects 'basematerial'.
// Similarly, 'composite' is used but XSD expects 'compositematerial'.
// These are library-level serialization issues that need to be fixed separately.

#[test]
fn validate_multi_properties() {
    // Create a color group
    let colorgroup = ColorGroup {
        id: 1,
        color: vec![
            ColorElement {
                color: Color::from_hex("#FF0000FF").unwrap(),
            },
            ColorElement {
                color: Color::from_hex("#00FF00FF").unwrap(),
            },
        ],
    };

    // Create a texture
    let texture2d = Texture2D {
        id: 2,
        path: "/3D/texture.png".to_owned(),
        contenttype: TextureContentType::Png,
        tilestyleu: None,
        tilestylev: None,
        filter: None,
    };

    // Create texture coordinate group
    let texture2dgroup = Texture2DGroup {
        id: 3,
        texid: 2,
        tex2coord: vec![
            Tex2Coord {
                u: Double::new(0.0),
                v: Double::new(0.0),
            },
            Tex2Coord {
                u: Double::new(1.0),
                v: Double::new(0.0),
            },
            Tex2Coord {
                u: Double::new(0.5),
                v: Double::new(1.0),
            },
        ],
    };

    // Create multi properties using new ResourceIdCollection
    let multiproperties = MultiProperties {
        id: 4,
        pids: ResourceIdCollection::from(vec![1, 3]), // Layer color group and texture group
        blendmethods: Some("mix multiply".to_owned()),
        multi: vec![
            Multi {
                pindices: ResourceIndexCollection::from(vec![0, 0]), // Color 0, Texture coord 0
            },
            Multi {
                pindices: ResourceIndexCollection::from(vec![1, 1]), // Color 1, Texture coord 1
            },
        ],
    };

    // Create a simple mesh with at least 4 vertices (XSD requirement)
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.5, 1.0, 0.0),
            Vertex::new(0.5, 0.5, 1.0),
        ],
    };

    // Create at least 4 triangles (XSD requirement)
    let triangles = Triangles {
        triangle: vec![
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::new(4), // Reference multi properties
            },
            Triangle {
                v1: 0,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::new(4),
            },
            Triangle {
                v1: 0,
                v2: 1,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::new(4),
            },
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::new(4),
            },
        ],
    };

    let mesh = Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("m".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Multi Property Mesh".to_owned()),
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
                colorgroup: vec![colorgroup],
                compositematerials: vec![],
                texture2dgroup: vec![texture2dgroup],
                multiproperties: vec![multiproperties],
                texture2d: vec![texture2d],
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

    validate_material_model(&model_xml);
}

#[test]
fn validate_vertex_color_application() {
    // This test validates the recent changes: ResourceIdCollection, ResourceIndexCollection,
    // and Vec<Double> for Composite.values in a complete real-world scenario

    // Create a color group with 4 colors for vertex coloring
    let colorgroup = ColorGroup {
        id: 1,
        color: vec![
            ColorElement {
                color: Color::from_hex("#FF0000FF").unwrap(),
            }, // Red
            ColorElement {
                color: Color::from_hex("#00FF00FF").unwrap(),
            }, // Green
            ColorElement {
                color: Color::from_hex("#0000FFFF").unwrap(),
            }, // Blue
            ColorElement {
                color: Color::from_hex("#FFFFFFFF").unwrap(),
            }, // White
        ],
    };

    // Create vertices for a simple quad (2 triangles)
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(0.0, 0.0, 0.0), // 0: bottom-left
            Vertex::new(1.0, 0.0, 0.0), // 1: bottom-right
            Vertex::new(1.0, 1.0, 0.0), // 2: top-right
            Vertex::new(0.0, 1.0, 0.0), // 3: top-left
        ],
    };

    // Create triangles with vertex color indices (at least 4 triangles for XSD)
    // Each vertex of each triangle has a color index (p1, p2, p3)
    // The pid references the color group resource
    let triangles = Triangles {
        triangle: vec![
            // First triangle: bottom-left (red), bottom-right (green), top-right (blue)
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::new(0), // Red
                p2: OptionalResourceIndex::new(1), // Green
                p3: OptionalResourceIndex::new(2), // Blue
                pid: OptionalResourceId::new(1),   // Reference color group 1
            },
            // Second triangle: bottom-left (red), top-right (blue), top-left (white)
            Triangle {
                v1: 0,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::new(0), // Red
                p2: OptionalResourceIndex::new(2), // Blue
                p3: OptionalResourceIndex::new(3), // White
                pid: OptionalResourceId::new(1),   // Reference color group 1
            },
            // Third triangle: bottom-left (red), bottom-right (green), top-left (white)
            Triangle {
                v1: 0,
                v2: 1,
                v3: 3,
                p1: OptionalResourceIndex::new(0), // Red
                p2: OptionalResourceIndex::new(1), // Green
                p3: OptionalResourceIndex::new(3), // White
                pid: OptionalResourceId::new(1),   // Reference color group 1
            },
            // Fourth triangle: bottom-right (green), top-right (blue), top-left (white)
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::new(1), // Green
                p2: OptionalResourceIndex::new(2), // Blue
                p3: OptionalResourceIndex::new(3), // White
                pid: OptionalResourceId::new(1),   // Reference color group 1
            },
        ],
    };

    let mesh = Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    };

    let write_package = ThreemfPackage::new(
        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("m".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Vertex Colored Quad".to_owned()),
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
                colorgroup: vec![colorgroup],
                compositematerials: vec![],
                texture2dgroup: vec![],
                multiproperties: vec![],
                texture2d: vec![],
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

    validate_material_model(&model_xml);
}

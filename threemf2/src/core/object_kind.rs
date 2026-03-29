#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "write")]
use instant_xml::ToXml;
use serde::Deserialize;

use crate::{
    core::{
        boolean::{Boolean, BooleanOperation, BooleanShape},
        component::Components,
        mesh::{Mesh, Triangles, Vertices},
    },
    threemf_namespaces::{BOOLEAN_NS, CORE_NS},
};

// In test file, define minimal versions of:
// #[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
// #[cfg_attr(feature = "write", derive(ToXml))]
// #[derive(PartialEq, Debug, Clone)]
// #[cfg_attr(
//     any(feature = "write", feature = "memory-optimized-read"),
//     xml(ns(CORE_NS), rename = "mesh")
// )]
// pub struct TestMesh {
//     #[xml(attribute)]
//     pub vertex_count: u32,

//     pub name: String,
// }

// #[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
// #[cfg_attr(feature = "write", derive(ToXml))]
// #[derive(PartialEq, Debug, Clone)]
// #[cfg_attr(
//     any(feature = "write", feature = "memory-optimized-read"),
//     xml(ns(BOOLEAN_NS, bo = BOOLEAN_NS), rename = "booleanshape")
// )]
// pub struct TestBooleanShape {
//     #[xml(attribute)]
//     pub objectid: u32,

//     #[xml(ns(BOOLEAN_NS))]
//     pub booleans: Vec<TestBoolean>,
// }

// #[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
// #[cfg_attr(feature = "write", derive(ToXml))]
// #[derive(PartialEq, Debug, Clone)]
// #[cfg_attr(
//     any(feature = "write", feature = "memory-optimized-read"),
//     xml(ns(BOOLEAN_NS, bo=BOOLEAN_NS), rename = "boolean")
// )]
// pub struct TestBoolean {
//     #[xml(attribute)]
//     name: String,
// }

#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(any(feature="write", feature="memory-optimized-read"), /* xml(ns(CORE_NS, bo=BOOLEAN_NS), */ xml(forward))]
#[cfg_attr(feature = "speed-optimized-read", serde(untagged))]
// #[xml(forward)]
pub enum ObjectKind {
    Mesh(Mesh),
    Components(Components),
    BooleanShape(BooleanShape),
}

impl ObjectKind {
    pub fn get_mesh(&self) -> Option<&Mesh> {
        if let Self::Mesh(mesh) = self {
            Some(mesh)
        } else {
            None
        }
    }

    pub fn get_components_object(&self) -> Option<&Components> {
        if let Self::Components(comps) = self {
            Some(comps)
        } else {
            None
        }
    }

    pub fn get_boolean_shape_object(&self) -> Option<&BooleanShape> {
        if let Self::BooleanShape(shape) = self {
            Some(shape)
        } else {
            None
        }
    }
}

#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(any(feature="write", feature="memory-optimized-read"), xml(ns(CORE_NS, bo=BOOLEAN_NS)))]
pub struct TestObject {
    #[xml(attribute)]
    pub id: u32,
    pub kind: ObjectKind,
}

#[test]
fn test_mesh_serialization() {
    let obj = TestObject {
        id: 1,
        kind: ObjectKind::Mesh(Mesh {
            vertices: Vertices { vertex: vec![] },
            triangles: Triangles { triangle: vec![] },
            trianglesets: None,
            beamlattice: None,
        }),
    };
    let xml = instant_xml::to_string(&obj).unwrap();
    // Expected: <object xmlns="..." id="1"><mesh vertex_count="8" /></object>
    println!("{xml}");
    assert!(xml.contains("<mesh"));
    assert!(xml.contains("vertex_count=\"8\""));
    // Should NOT have bo: prefix for mesh
    assert!(!xml.contains("bo:mesh"));
}

#[test]
fn test_boolean_shape_serialization() {
    let obj = TestObject {
        id: 1,
        kind: ObjectKind::BooleanShape(BooleanShape {
            objectid: 1,
            operation: BooleanOperation::Union,
            transform: None,
            path: None,
            booleans: vec![Boolean {
                objectid: 2,
                transform: None,
                path: None,
            }],
        }),
    };
    let xml = instant_xml::to_string(&obj).unwrap();
    println!("{xml}");
    // Expected: <object xmlns="..." xmlns:bo="..." id="1"><bo:booleanshape bo:objectid="2" /></object>
    assert!(xml.contains("</bo:booleanshape>"));
    assert!(xml.contains("objectid=\"2\""));
}

#[test]
fn test_mesh_deserialization() {
    let obj = TestObject {
        id: 1,
        kind: ObjectKind::Mesh(Mesh {
            vertices: Vertices { vertex: vec![] },
            triangles: Triangles { triangle: vec![] },
            trianglesets: None,
            beamlattice: None,
        }),
    };
    let xml_string = r##"<TestObject xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" id="1"><mesh vertex_count="8"><name>Lala</name></mesh></TestObject>"##;
    let de_obj = instant_xml::from_str::<TestObject>(xml_string).unwrap();
    assert_eq!(obj, de_obj);
}

#[test]
fn test_boolean_shape_deserialization() {
    let obj = TestObject {
        id: 1,
        kind: ObjectKind::BooleanShape(BooleanShape {
            objectid: 1,
            operation: BooleanOperation::Intersection,
            transform: None,
            path: None,
            booleans: vec![Boolean {
                objectid: 2,
                transform: None,
                path: None,
            }],
        }),
    };
    let xml_string = r##"<TestObject xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" id="1"><bo:booleanshape objectid="2"><bo:boolean name="Baba"></bo:boolean></bo:booleanshape></TestObject>"##;
    let de_obj = instant_xml::from_str::<TestObject>(xml_string).unwrap();

    assert_eq!(obj, de_obj);
}

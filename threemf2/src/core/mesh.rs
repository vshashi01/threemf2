#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::core::beamlattice::BeamLattice;
use crate::core::triangle_set::TriangleSets;
use crate::threemf_namespaces::BEAM_LATTICE_NS;
use crate::threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS};

/// A triangle mesh
///
/// It is expected that users of this library will use their own mesh type,
/// and the simplicity of [`Mesh`] provides an easy target for conversion to and from.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(any(feature = "write", feature = "memory-optimized-read"), xml(ns(CORE_NS, t = CORE_TRIANGLESET_NS, b = BEAM_LATTICE_NS), rename = "mesh"))]
pub struct Mesh {
    /// The vertices of the mesh
    ///
    /// This defines the vertices that are part of the mesh, but not the mesh's
    /// structure. See the [`Mesh::triangles`] field.
    pub vertices: Vertices,

    /// The triangles that make up the mesh
    ///
    /// Each triangle consists of indices that refer back to the `vertices`
    /// field.
    pub triangles: Triangles,

    /// Optional TriangleSets that allows to create identifiable group of triangles
    ///
    /// See [`crate::core::triangle_set::TriangleSet`] for more details
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(CORE_TRIANGLESET_NS))
    )]
    pub trianglesets: Option<TriangleSets>,

    /// Optional Beam Lattice geometry that is part of this mesh
    ///
    /// See [`crate::core::beamlattice::BeamLattice`] for more details
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_NS))
    )]
    pub beamlattice: Option<BeamLattice>,
}

/// Collection of Vertex
///
/// See [`Vertex`] for more details
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "vertices")
)]
pub struct Vertices {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub vertex: Vec<Vertex>,
}

/// A vertex in a mesh
///
/// A vertex is defined as a Point coordinate in 3D coordinate system.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(
    all(
        feature = "memory-optimized-read",
        not(feature = "memory-optimized-fast-float-read")
    ),
    derive(FromXml)
)]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "vertex")
)]
pub struct Vertex {
    /// X position
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub x: f64,

    /// Y position
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub y: f64,

    /// Z position
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub z: f64,
}

#[cfg(feature = "memory-optimized-fast-float-read")]
impl<'xml> FromXml<'xml> for Vertex {
    #[inline]
    fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: "",
            name: "vertex",
        }
    }
    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        _: &'static str,
        deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
    ) -> ::std::result::Result<(), ::instant_xml::Error> {
        use ::instant_xml::Error;
        use ::instant_xml::de::Node;
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut z: f64 = 0.0;
        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    // println!("Attr value: {:?}", attr.value);
                    match id.name {
                        "x" => {
                            x = fast_float2::parse(attr.value.as_ref()).unwrap_or_default();
                        }

                        "y" => {
                            y = fast_float2::parse(attr.value.as_ref()).unwrap_or_default();
                        }
                        "z" => {
                            z = fast_float2::parse(attr.value.as_ref()).unwrap_or_default();
                        }
                        _ => {
                            let mut nested =
                                deserializer.for_node(Node::AttributeValue(attr.value));
                            nested.ignore()?;
                        }
                    }
                }
                Node::Open(data) => {
                    let mut nested = deserializer.nested(data);
                    nested.ignore()?;
                }
                Node::Text(_) => {}
                _ => {
                    return Err(Error::UnexpectedNode("Unexpected".to_owned()));
                }
            }
        }
        *into = Some(Self { x, y, z });
        Ok(())
    }
    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

/// Collection of Triangle
///
/// See [`Triangle`] for more details.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "triangles")
)]
pub struct Triangles {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub triangle: Vec<Triangle>,
}

/// A triangle in Mesh
///
/// The triangle consists of indices that refer to the vertices of the mesh.
/// Each vertex of the triangle are defined as an index into [`Vertices`]
/// additional indices into other resources can be specified
/// for each vertex of the triangle as well.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "triangle")
)]
pub struct Triangle {
    /// Vertex 1
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub v1: usize,

    /// Vertex 2
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub v2: usize,

    /// Vertex 3
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub v3: usize,

    /// Overrides the object level pindex for Vertex 1 of this [`Triangle`]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub p1: Option<usize>,

    /// Overrides the object level pindex for Vertex 2 of this [`Triangle`]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub p2: Option<usize>,

    /// Overrides the object level pindex for Vertex 3 of this [`Triangle`]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub p3: Option<usize>,

    /// Overrides the object level pid for this [`Triangle`]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub pid: Option<usize>,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::{
        BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX, CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX,
    };

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn toxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = Vertex {
            x: 100.5,
            y: 100.0,
            z: 0.0,
        };
        let vertex_string = to_string(&vertex).unwrap();

        assert_eq!(vertex_string, xml_string);
    }

    #[test]
    pub fn toxml_vertices_test() {
        let xml_string = format!(
            r#"<vertices xmlns="{}"><vertex x="100" y="110.5" z="0" /><vertex x="0.156" y="55.6896" z="-10" /></vertices>"#,
            CORE_NS
        );
        let vertices = Vertices {
            vertex: vec![
                Vertex {
                    x: 100.,
                    y: 110.5,
                    z: 0.0,
                },
                Vertex {
                    x: 0.156,
                    y: 55.6896,
                    z: -10.0,
                },
            ],
        };
        let vertices_string = to_string(&vertices).unwrap();

        assert_eq!(vertices_string, xml_string)
    }

    #[test]
    pub fn toxml_required_fields_triangle_test() {
        let xml_string = format!(r#"<triangle xmlns="{}" v1="1" v2="2" v3="3" />"#, CORE_NS);
        let triangle = Triangle {
            v1: 1,
            v2: 2,
            v3: 3,
            p1: None,
            p2: None,
            p3: None,
            pid: None,
        };
        let triangle_string = to_string(&triangle).unwrap();

        assert_eq!(triangle_string, xml_string);
    }

    #[test]
    pub fn toxml_triangles_test() {
        let xml_string = format!(
            r#"<triangles xmlns="{}"><triangle v1="1" v2="2" v3="3" /><triangle v1="2" v2="3" v3="4" /></triangles>"#,
            CORE_NS
        );
        let triangles = Triangles {
            triangle: vec![
                Triangle {
                    v1: 1,
                    v2: 2,
                    v3: 3,
                    p1: None,
                    p2: None,
                    p3: None,
                    pid: None,
                },
                Triangle {
                    v1: 2,
                    v2: 3,
                    v3: 4,
                    p1: None,
                    p2: None,
                    p3: None,
                    pid: None,
                },
            ],
        };
        let triangles_string = to_string(&triangles).unwrap();

        assert_eq!(triangles_string, xml_string);
    }

    #[test]
    pub fn toxml_mesh_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{core_ns}" xmlns:{bl_prefix}="{bl_ns}" xmlns:{ts_prefix}="{ts_ns}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles></mesh>"##,
            core_ns = CORE_NS,
            ts_prefix = CORE_TRIANGLESET_PREFIX,
            ts_ns = CORE_TRIANGLESET_NS,
            bl_prefix = BEAM_LATTICE_PREFIX,
            bl_ns = BEAM_LATTICE_NS,
        );
        let mesh = Mesh {
            vertices: Vertices {
                vertex: vec![
                    Vertex {
                        x: -1.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    Vertex {
                        x: 1.0,
                        y: -1.0,
                        z: 0.0,
                    },
                    Vertex {
                        x: 1.0,
                        y: 1.0,
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
                triangle: vec![
                    Triangle {
                        v1: 0,
                        v2: 1,
                        v3: 2,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                    Triangle {
                        v1: 0,
                        v2: 2,
                        v3: 3,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                ],
            },
            trianglesets: None,
            beamlattice: None,
        };
        let mesh_string = to_string(&mesh).unwrap();

        assert_eq!(mesh_string, xml_string);
    }
}

#[cfg(all(
    feature = "memory-optimized-read",
    not(feature = "memory-optimized-fast-float-read")
))]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::CORE_NS;

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn fromxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(
            vertex,
            Vertex {
                x: 100.5,
                y: 100.0,
                z: 0.0,
            }
        );
    }

    #[test]
    pub fn fromxml_vertices_test() {
        let xml_string = format!(
            r#"<vertices xmlns="{}"><vertex x="100" y="110.5" z="0" /><vertex x="0.156" y="55.6896" z="-10" /></vertices>"#,
            CORE_NS
        );
        let vertices = from_str::<Vertices>(&xml_string).unwrap();

        assert_eq!(
            vertices,
            Vertices {
                vertex: vec![
                    Vertex {
                        x: 100.,
                        y: 110.5,
                        z: 0.0,
                    },
                    Vertex {
                        x: 0.156,
                        y: 55.6896,
                        z: -10.0,
                    },
                ],
            }
        )
    }

    #[test]
    pub fn fromxml_required_fields_triangle_test() {
        let xml_string = format!(r#"<triangle xmlns="{}" v1="1" v2="2" v3="3" />"#, CORE_NS);
        let triangle = from_str::<Triangle>(&xml_string).unwrap();

        assert_eq!(
            triangle,
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: None,
                p2: None,
                p3: None,
                pid: None,
            }
        );
    }

    #[test]
    pub fn fromxml_triangles_test() {
        let xml_string = format!(
            r#"<triangles xmlns="{}"><triangle v1="1" v2="2" v3="3" /><triangle v1="2" v2="3" v3="4" /></triangles>"#,
            CORE_NS
        );
        let triangles = from_str::<Triangles>(&xml_string).unwrap();

        assert_eq!(
            triangles,
            Triangles {
                triangle: vec![
                    Triangle {
                        v1: 1,
                        v2: 2,
                        v3: 3,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                    Triangle {
                        v1: 2,
                        v2: 3,
                        v3: 4,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_mesh_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles></mesh>"##,
            CORE_NS
        );
        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex {
                            x: -1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: 1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: -1.0,
                            y: 1.0,
                            z: 0.0
                        }
                    ]
                },
                triangles: Triangles {
                    triangle: vec![
                        Triangle {
                            v1: 0,
                            v2: 1,
                            v3: 2,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        },
                        Triangle {
                            v1: 0,
                            v2: 2,
                            v3: 3,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        }
                    ]
                },
                trianglesets: None,
                beamlattice: None,
            }
        )
    }
}

#[cfg(feature = "memory-optimized-fast-float-read")]
#[cfg(test)]
mod memory_optimized_fast_float_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::CORE_NS;

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn fromxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(
            vertex,
            Vertex {
                x: 100.5,
                y: 100.0,
                z: 0.0,
            }
        );
    }

    #[test]
    pub fn fromxml_vertices_test() {
        let xml_string = format!(
            r#"<vertices xmlns="{}"><vertex x="100" y="110.5" z="0" /><vertex x="0.156" y="55.6896" z="-10" /></vertices>"#,
            CORE_NS
        );
        let vertices = from_str::<Vertices>(&xml_string).unwrap();

        assert_eq!(
            vertices,
            Vertices {
                vertex: vec![
                    Vertex {
                        x: 100.,
                        y: 110.5,
                        z: 0.0,
                    },
                    Vertex {
                        x: 0.156,
                        y: 55.6896,
                        z: -10.0,
                    },
                ],
            }
        )
    }

    #[test]
    pub fn fromxml_required_fields_triangle_test() {
        let xml_string = format!(r#"<triangle xmlns="{}" v1="1" v2="2" v3="3" />"#, CORE_NS);
        let triangle = from_str::<Triangle>(&xml_string).unwrap();

        assert_eq!(
            triangle,
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: None,
                p2: None,
                p3: None,
                pid: None,
            }
        );
    }

    #[test]
    pub fn fromxml_triangles_test() {
        let xml_string = format!(
            r#"<triangles xmlns="{}"><triangle v1="1" v2="2" v3="3" /><triangle v1="2" v2="3" v3="4" /></triangles>"#,
            CORE_NS
        );
        let triangles = from_str::<Triangles>(&xml_string).unwrap();

        assert_eq!(
            triangles,
            Triangles {
                triangle: vec![
                    Triangle {
                        v1: 1,
                        v2: 2,
                        v3: 3,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                    Triangle {
                        v1: 2,
                        v2: 3,
                        v3: 4,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_mesh_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles></mesh>"##,
            CORE_NS
        );
        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex {
                            x: -1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: 1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: -1.0,
                            y: 1.0,
                            z: 0.0
                        }
                    ]
                },
                triangles: Triangles {
                    triangle: vec![
                        Triangle {
                            v1: 0,
                            v2: 1,
                            v3: 2,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        },
                        Triangle {
                            v1: 0,
                            v2: 2,
                            v3: 3,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        }
                    ]
                },
                trianglesets: None,
                beamlattice: None,
            }
        )
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::threemf_namespaces::CORE_NS;

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn fromxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(
            vertex,
            Vertex {
                x: 100.5,
                y: 100.0,
                z: 0.0,
            }
        );
    }

    #[test]
    pub fn fromxml_vertices_test() {
        let xml_string = format!(
            r#"<vertices xmlns="{}"><vertex x="100" y="110.5" z="0" /><vertex x="0.156" y="55.6896" z="-10" /></vertices>"#,
            CORE_NS
        );
        let vertices = from_str::<Vertices>(&xml_string).unwrap();

        assert_eq!(
            vertices,
            Vertices {
                vertex: vec![
                    Vertex {
                        x: 100.,
                        y: 110.5,
                        z: 0.0,
                    },
                    Vertex {
                        x: 0.156,
                        y: 55.6896,
                        z: -10.0,
                    },
                ],
            }
        )
    }

    #[test]
    pub fn fromxml_required_fields_triangle_test() {
        let xml_string = format!(r#"<triangle xmlns="{}" v1="1" v2="2" v3="3" />"#, CORE_NS);
        let triangle = from_str::<Triangle>(&xml_string).unwrap();

        assert_eq!(
            triangle,
            Triangle {
                v1: 1,
                v2: 2,
                v3: 3,
                p1: None,
                p2: None,
                p3: None,
                pid: None,
            }
        );
    }

    #[test]
    pub fn fromxml_triangles_test() {
        let xml_string = format!(
            r#"<triangles xmlns="{}"><triangle v1="1" v2="2" v3="3" /><triangle v1="2" v2="3" v3="4" /></triangles>"#,
            CORE_NS
        );
        let triangles = from_str::<Triangles>(&xml_string).unwrap();

        assert_eq!(
            triangles,
            Triangles {
                triangle: vec![
                    Triangle {
                        v1: 1,
                        v2: 2,
                        v3: 3,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                    Triangle {
                        v1: 2,
                        v2: 3,
                        v3: 4,
                        p1: None,
                        p2: None,
                        p3: None,
                        pid: None,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_mesh_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles></mesh>"##,
            CORE_NS
        );
        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex {
                            x: -1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: -1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: 1.0,
                            y: 1.0,
                            z: 0.0
                        },
                        Vertex {
                            x: -1.0,
                            y: 1.0,
                            z: 0.0
                        }
                    ]
                },
                triangles: Triangles {
                    triangle: vec![
                        Triangle {
                            v1: 0,
                            v2: 1,
                            v3: 2,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        },
                        Triangle {
                            v1: 0,
                            v2: 2,
                            v3: 3,
                            p1: None,
                            p2: None,
                            p3: None,
                            pid: None,
                        }
                    ]
                },
                trianglesets: None,
                beamlattice: None,
            }
        )
    }
}

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;
#[cfg(feature = "read")]
use instant_xml::de::RawToken;

use crate::model::domain::beamlattice::BeamLattice;
#[cfg(feature = "read")]
use crate::model::domain::constants;
use crate::model::domain::triangle_set::TriangleSets;
use crate::model::{Double, OptionalResourceId, OptionalResourceIndex, ResourceIndex};
use crate::threemf_namespaces::BEAM_LATTICE_NS;
use crate::threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS};

/// A triangle mesh
///
/// It is expected that users of this library will use their own mesh type,
/// and the simplicity of [`Mesh`] provides an easy target for conversion to and from.
#[cfg_attr(all(feature = "read",), derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(any(feature = "write", feature = "read"), xml(ns(CORE_NS, t = CORE_TRIANGLESET_NS, b = BEAM_LATTICE_NS), rename = "mesh"))]
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
    /// See [`TriangleSet`](crate::model::domain::triangle_set::TriangleSet) for more details
    #[cfg_attr(any(feature = "write", feature = "read"), xml(ns(CORE_TRIANGLESET_NS)))]
    pub trianglesets: Option<TriangleSets>,

    /// Optional Beam Lattice geometry that is part of this mesh
    ///
    /// See [`BeamLattice`] for more details
    #[cfg_attr(any(feature = "write", feature = "read"), xml(ns(BEAM_LATTICE_NS)))]
    pub beamlattice: Option<BeamLattice>,
}

/// Collection of Vertex
///
/// See [`Vertex`] for more details
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "write", xml(ns(CORE_NS), rename = "vertices"))]
pub struct Vertices {
    /// Field containing the collection
    pub vertex: Vec<Vertex>,
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Vertices {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: CORE_NS,
            name: "vertices",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        // Hot loop: thousands to millions of `<vertex x=".." y=".." z=".."/>`
        // children. Read raw tokens instead of going through a nested
        // Deserializer per child (see `Deserializer::raw_children`).
        let mut vertices: Vec<Vertex> = Vec::with_capacity(constants::MAX_VERTEX_BUFFER);
        let mut current: Option<[f64; 3]> = None;
        let mut depth = 0usize;

        for token in deserializer.raw_children() {
            match token? {
                RawToken::ElementStart { local, .. } => {
                    depth += 1;
                    if depth == 1 && local == "vertex" {
                        current = Some([0.0; 3]);
                    }
                }
                RawToken::Attribute { local, value, .. } if depth == 1 => {
                    if let Some(xyz) = current.as_mut() {
                        match local {
                            "x" => xyz[0] = constants::parse_num_attr(&value, "x")?,
                            "y" => xyz[1] = constants::parse_num_attr(&value, "y")?,
                            "z" => xyz[2] = constants::parse_num_attr(&value, "z")?,
                            _ => {}
                        }
                    }
                }
                RawToken::EmptyElementEnd | RawToken::Close { .. } => {
                    if depth == 1
                        && let Some([x, y, z]) = current.take()
                    {
                        vertices.push(Vertex::new(x, y, z));
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        vertices.shrink_to_fit();
        *into = Some(Vertices { vertex: vertices });

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Scalar;
}

/// A vertex in a mesh
///
/// A vertex is defined as a Point coordinate in 3D coordinate system.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "write", xml(ns(CORE_NS), rename = "vertex"))]
pub struct Vertex {
    /// X position
    #[cfg_attr(feature = "write", xml(attribute))]
    pub x: Double,

    /// Y position
    #[cfg_attr(feature = "write", xml(attribute))]
    pub y: Double,

    /// Z position
    #[cfg_attr(feature = "write", xml(attribute))]
    pub z: Double,
}

impl Vertex {
    /// Easy constructor from [`f64`] values
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Double::new(x),
            y: Double::new(y),
            z: Double::new(z),
        }
    }
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Vertex {
    #[inline]
    fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: CORE_NS,
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

                    match id.name {
                        "x" => x = constants::parse_num_attr(&attr.value, "x")?,
                        "y" => y = constants::parse_num_attr(&attr.value, "y")?,
                        "z" => z = constants::parse_num_attr(&attr.value, "z")?,
                        _ => {}
                    };
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

        *into = Some(Self {
            x: Double::new(x),
            y: Double::new(y),
            z: Double::new(z),
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

/// Collection of Triangle
///
/// See [`Triangle`] for more details.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "write", xml(ns(CORE_NS), rename = "triangles"))]
pub struct Triangles {
    /// Field containing the triangles
    pub triangle: Vec<Triangle>,
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Triangles {
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: CORE_NS,
            name: "triangles",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        // Hot loop, same approach as `Vertices`.
        let mut triangles: Vec<Triangle> = Vec::with_capacity(constants::MAX_TRIANGLE_BUFFER);
        let mut current: Option<Triangle> = None;
        let mut depth = 0usize;

        for token in deserializer.raw_children() {
            match token? {
                RawToken::ElementStart { local, .. } => {
                    depth += 1;
                    if depth == 1 && local == "triangle" {
                        current = Some(Triangle::empty());
                    }
                }
                RawToken::Attribute { local, value, .. } if depth == 1 => {
                    if let Some(triangle) = current.as_mut() {
                        triangle.set_attribute(local, &value)?;
                    }
                }
                RawToken::EmptyElementEnd | RawToken::Close { .. } => {
                    if depth == 1
                        && let Some(triangle) = current.take()
                    {
                        triangles.push(triangle);
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        triangles.shrink_to_fit();
        *into = Some(Triangles {
            triangle: triangles,
        });

        Ok(())
    }

    type Accumulator = Option<Self>;

    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

/// A triangle in Mesh
///
/// The triangle consists of indices that refer to the vertices of the mesh.
/// Each vertex of the triangle are defined as an index into [`Vertices`]
/// additional indices into other resources can be specified
/// for each vertex of the triangle as well.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "write", xml(ns(CORE_NS), rename = "triangle"))]
pub struct Triangle {
    /// Vertex 1
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v1: ResourceIndex,

    /// Vertex 2
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v2: ResourceIndex,

    /// Vertex 3
    #[cfg_attr(feature = "write", xml(attribute))]
    pub v3: ResourceIndex,

    /// Overrides the object level pindex for Vertex 1 of this [`Triangle`]
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p1: OptionalResourceIndex,

    /// Overrides the object level pindex for Vertex 2 of this [`Triangle`]
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p2: OptionalResourceIndex,

    /// Overrides the object level pindex for Vertex 3 of this [`Triangle`]
    #[cfg_attr(feature = "write", xml(attribute))]
    pub p3: OptionalResourceIndex,

    /// Overrides the object level pid for this [`Triangle`]
    #[cfg_attr(feature = "write", xml(attribute))]
    pub pid: OptionalResourceId,
}

#[cfg(feature = "read")]
impl Triangle {
    fn empty() -> Self {
        Self {
            v1: 0,
            v2: 0,
            v3: 0,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        }
    }

    /// Applies one `<triangle>` attribute; unknown attributes are ignored.
    #[inline]
    fn set_attribute(&mut self, name: &str, value: &str) -> Result<(), instant_xml::Error> {
        match name {
            "v1" => self.v1 = constants::parse_num_attr(value, "v1")?,
            "v2" => self.v2 = constants::parse_num_attr(value, "v2")?,
            "v3" => self.v3 = constants::parse_num_attr(value, "v3")?,
            "p1" => self.p1 = OptionalResourceIndex::new(constants::parse_num_attr(value, "p1")?),
            "p2" => self.p2 = OptionalResourceIndex::new(constants::parse_num_attr(value, "p2")?),
            "p3" => self.p3 = OptionalResourceIndex::new(constants::parse_num_attr(value, "p3")?),
            "pid" => self.pid = OptionalResourceId::new(constants::parse_num_attr(value, "pid")?),
            _ => {}
        }
        Ok(())
    }
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Triangle {
    #[inline]
    fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
        id == ::instant_xml::Id {
            ns: CORE_NS,
            name: "triangle",
        }
    }
    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        _: &'static str,
        deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
    ) -> ::std::result::Result<(), ::instant_xml::Error> {
        use ::instant_xml::Error;
        use ::instant_xml::de::Node;
        let mut triangle = Triangle::empty();

        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    triangle.set_attribute(id.name, &attr.value)?;
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

        *into = Some(triangle);
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::model::{OptionalResourceId, OptionalResourceIndex};
    use crate::threemf_namespaces::{
        BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX, CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX,
    };

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn toxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = Vertex::new(100.5, 100.0, 0.0);
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
                Vertex::new(100., 110.5, 0.0),
                Vertex::new(0.156, 55.6896, -10.0),
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
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
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
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 2,
                    v2: 3,
                    v3: 4,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
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
                    Vertex::new(-1.0, -1.0, 0.0),
                    Vertex::new(1.0, -1.0, 0.0),
                    Vertex::new(1.0, 1.0, 0.0),
                    Vertex::new(-1.0, 1.0, 0.0),
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
                ],
            },
            trianglesets: None,
            beamlattice: None,
        };
        let mesh_string = to_string(&mesh).unwrap();

        assert_eq!(mesh_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::model::{OptionalResourceId, OptionalResourceIndex};
    use crate::threemf_namespaces::CORE_NS;

    use super::{Mesh, Triangle, Triangles, Vertex, Vertices};

    #[test]
    pub fn fromxml_vertex_test() {
        let xml_string = format!(r#"<vertex xmlns="{}" x="100.5" y="100" z="0" />"#, CORE_NS);
        let vertex = from_str::<Vertex>(&xml_string).unwrap();

        assert_eq!(vertex, Vertex::new(100.5, 100.0, 0.0,));
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
                    Vertex::new(100., 110.5, 0.0,),
                    Vertex::new(0.156, 55.6896, -10.0,),
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
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
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
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        p3: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Triangle {
                        v1: 2,
                        v2: 3,
                        v3: 4,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        p3: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
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
                        Vertex::new(-1.0, -1.0, 0.0),
                        Vertex::new(1.0, -1.0, 0.0),
                        Vertex::new(1.0, 1.0, 0.0),
                        Vertex::new(-1.0, 1.0, 0.0),
                    ]
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
                        }
                    ]
                },
                trianglesets: None,
                beamlattice: None,
            }
        )
    }

    #[test]
    pub fn fromxml_raw_path_edge_cases_test() {
        // Prefixed core namespace, padded numbers, entity-free values, a nested
        // unknown child, an open/close vertex form and a comment in between.
        let xml_string = format!(
            r#"<m:mesh xmlns:m="{CORE_NS}"><m:vertices><m:vertex x=" 1.5" y="2 " z="3"/><!-- c --><m:vertex x="4" y="5" z="6"><m:ext a="9"/></m:vertex></m:vertices><m:triangles><m:triangle v1="0" v2="1" v3="0" p1="1 " pid="7"></m:triangle></m:triangles></m:mesh>"#
        );
        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh.vertices.vertex,
            vec![Vertex::new(1.5, 2.0, 3.0), Vertex::new(4.0, 5.0, 6.0)]
        );
        assert_eq!(
            mesh.triangles.triangle,
            vec![Triangle {
                v1: 0,
                v2: 1,
                v3: 0,
                p1: OptionalResourceIndex::new(1),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::new(7),
            }]
        );
    }

    #[test]
    pub fn fromxml_empty_collections_test() {
        let xml_string =
            format!(r#"<mesh xmlns="{CORE_NS}"><vertices/><triangles></triangles></mesh>"#);
        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert!(mesh.vertices.vertex.is_empty());
        assert!(mesh.triangles.triangle.is_empty());
    }

    #[test]
    pub fn fromxml_invalid_number_is_error_test() {
        let xml_string =
            format!(r#"<vertices xmlns="{CORE_NS}"><vertex x="1" y="oops" z="3"/></vertices>"#);
        assert!(from_str::<Vertices>(&xml_string).is_err());

        let xml_string = format!(
            r#"<triangles xmlns="{CORE_NS}"><triangle v1="1" v2="2" v3="-3"/></triangles>"#
        );
        assert!(from_str::<Triangles>(&xml_string).is_err());
    }
}

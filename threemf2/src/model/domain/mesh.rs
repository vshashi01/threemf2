#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

use crate::model::domain::any_extensions::{AnyAttributeExtensions, AnyExtensions};
use crate::model::domain::beamlattice::BeamLattice;
#[cfg(feature = "read")]
use crate::model::domain::constants;
use crate::model::domain::triangle_set::TriangleSets;
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::model::{Double, OptionalResourceId, OptionalResourceIndex, ResourceIndex};
use crate::threemf_namespaces::BEAM_LATTICE_NS;
#[cfg(feature = "write")]
use crate::threemf_namespaces::{BEAM_LATTICE_PREFIX, CORE_TRIANGLESET_PREFIX};
use crate::threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS};

/// A triangle mesh
///
/// It is expected that users of this library will use their own mesh type,
/// and the simplicity of [`Mesh`] provides an easy target for conversion to and from.
#[derive(PartialEq, Clone, Debug)]
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
    pub trianglesets: Option<TriangleSets>,

    /// Optional Beam Lattice geometry that is part of this mesh
    ///
    /// See [`BeamLattice`] for more details
    pub beamlattice: Option<BeamLattice>,

    /// Foreign attributes and trailing child elements of the mesh (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Mesh {
        tag: "mesh",
        ns: CORE_NS,
        prefixes: [
            (BEAM_LATTICE_PREFIX, BEAM_LATTICE_NS),
            (CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS),
        ],
        force_prefix: false,
        attributes: {},
        elements: {
            vertices: Vertices => (CORE_NS, "vertices"),
            triangles: Triangles => (CORE_NS, "triangles"),
            trianglesets: Option<TriangleSets> => (CORE_TRIANGLESET_NS, "trianglesets"),
            beamlattice: Option<BeamLattice> => (BEAM_LATTICE_NS, "beamlattice"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// Collection of Vertex
///
/// See [`Vertex`] for more details
#[derive(PartialEq, Clone, Debug)]
pub struct Vertices {
    /// Field containing the collection
    pub vertex: Vec<Vertex>,

    /// Foreign attributes of the `vertices` element (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(feature = "write")]
impl ToXml for Vertices {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let mut context = instant_xml::ser::Context::<0>::default();
        context.default_ns = CORE_NS;
        let element = serializer.write_start("vertices", CORE_NS, Some(context))?;
        #[cfg(feature = "any")]
        crate::model::domain::any::write::write_foreign_attributes(
            serializer,
            &[],
            CORE_NS,
            self.any_extensions.attributes(),
            &[],
        )?;
        serializer.end_start()?;
        for item in &self.vertex {
            item.serialize(
                Some(instant_xml::Id {
                    ns: CORE_NS,
                    name: "vertex",
                }),
                serializer,
            )?;
        }
        serializer.write_close(element)
    }
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

        let mut vertices: Vec<Vertex> = Vec::with_capacity(constants::MAX_VERTEX_BUFFER);
        #[allow(unused_mut)]
        let mut any_extensions = AnyAttributeExtensions::default();

        while let Some(node) = deserializer.next() {
            #[cfg(feature = "any")]
            if let Ok(instant_xml::de::Node::Attribute(attr)) = &node
                && let Ok(id) = deserializer.attribute_id(attr)
            {
                crate::model::domain::any::read::capture_attribute(
                    any_extensions.attributes_mut(),
                    id,
                    CORE_NS,
                    attr.prefix,
                    &attr.value,
                );
                continue;
            }

            if let Ok(n) = node
                && let instant_xml::de::Node::Open(element) = n
            {
                //println!("This is element value {:?}", element);
                let mut vertex_value: Option<Vertex> = None;
                let mut nested = deserializer.nested(element);

                if <Vertex as instant_xml::FromXml>::deserialize(
                    &mut vertex_value,
                    "vertex",
                    &mut nested,
                )
                .is_ok()
                    && let Some(vertex) = vertex_value
                {
                    vertices.push(vertex);
                };
            }
        }

        vertices.shrink_to_fit();
        *into = Some(Vertices {
            vertex: vertices,
            any_extensions,
        });

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

                    match id.name.as_bytes().first() {
                        Some(b'x') => {
                            x = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default()
                        }
                        Some(b'y') => {
                            y = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default()
                        }
                        Some(b'z') => {
                            z = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default()
                        }
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
#[derive(PartialEq, Clone, Debug)]
pub struct Triangles {
    /// Field containing the triangles
    pub triangle: Vec<Triangle>,

    /// Foreign attributes of the `triangles` element (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(feature = "write")]
impl ToXml for Triangles {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let mut context = instant_xml::ser::Context::<0>::default();
        context.default_ns = CORE_NS;
        let element = serializer.write_start("triangles", CORE_NS, Some(context))?;
        #[cfg(feature = "any")]
        crate::model::domain::any::write::write_foreign_attributes(
            serializer,
            &[],
            CORE_NS,
            self.any_extensions.attributes(),
            &[],
        )?;
        serializer.end_start()?;
        for item in &self.triangle {
            item.serialize(
                Some(instant_xml::Id {
                    ns: CORE_NS,
                    name: "triangle",
                }),
                serializer,
            )?;
        }
        serializer.write_close(element)
    }
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

        let mut triangles: Vec<Triangle> = Vec::with_capacity(constants::MAX_TRIANGLE_BUFFER);
        #[allow(unused_mut)]
        let mut any_extensions = AnyAttributeExtensions::default();
        while let Some(node) = deserializer.next() {
            #[cfg(feature = "any")]
            if let Ok(instant_xml::de::Node::Attribute(attr)) = &node
                && let Ok(id) = deserializer.attribute_id(attr)
            {
                crate::model::domain::any::read::capture_attribute(
                    any_extensions.attributes_mut(),
                    id,
                    CORE_NS,
                    attr.prefix,
                    &attr.value,
                );
                continue;
            }

            if let Ok(n) = node
                && let instant_xml::de::Node::Open(element) = n
            {
                let mut triangle_value: Option<Triangle> = None;
                let mut nested = deserializer.nested(element);
                if <Triangle as instant_xml::FromXml>::deserialize(
                    &mut triangle_value,
                    field,
                    &mut nested,
                )
                .is_ok()
                    && let Some(vertex) = triangle_value
                {
                    triangles.push(vertex);
                }
            }
        }

        triangles.shrink_to_fit();
        *into = Some(Triangles {
            triangle: triangles,
            any_extensions,
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
        let mut v1: ResourceIndex = 0;
        let mut v2: ResourceIndex = 0;
        let mut v3: ResourceIndex = 0;
        let mut p1: OptionalResourceIndex = OptionalResourceIndex::none();
        let mut p2: OptionalResourceIndex = OptionalResourceIndex::none();
        let mut p3: OptionalResourceIndex = OptionalResourceIndex::none();
        let mut pid: OptionalResourceId = OptionalResourceId::none();

        while let Some(node) = deserializer.next() {
            let node = node?;
            match node {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;

                    match id.name {
                        "v1" => v1 = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default(),
                        "v2" => v2 = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default(),
                        "v3" => v3 = lexical_core::parse(attr.value.as_bytes()).unwrap_or_default(),
                        "p1" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                p1 = OptionalResourceIndex::new(value);
                            }
                        }
                        "p2" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                p2 = OptionalResourceIndex::new(value);
                            }
                        }
                        "p3" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                p3 = OptionalResourceIndex::new(value);
                            }
                        }
                        "pid" => {
                            if let Ok(value) = lexical_core::parse(attr.value.as_bytes()) {
                                pid = OptionalResourceId::new(value);
                            }
                        }
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
            v1,
            v2,
            v3,
            p1,
            p2,
            p3,
            pid,
        });
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
            any_extensions: Default::default(),
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
            any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                any_extensions: Default::default(),
            },
            trianglesets: None,
            beamlattice: None,
            any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                any_extensions: Default::default(),
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
                    ],
                    any_extensions: Default::default(),
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
                    ],
                    any_extensions: Default::default(),
                },
                trianglesets: None,
                beamlattice: None,
                any_extensions: Default::default(),
            }
        )
    }
}

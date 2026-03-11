#[cfg(feature = "write")]
use instant_xml::{Error, Id, Serializer, ToXml};

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{core::types::ResourceIndex, threemf_namespaces::CORE_TRIANGLESET_NS};

/// Collection of Triangle Set. See [`TriangleSet`] for more details.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "trianglesets"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(
    feature = "memory-optimized-read",
    xml(ns(CORE_TRIANGLESET_NS), rename = "trianglesets")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleSets {
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "triangleset"))]
    pub trianglesets: Vec<TriangleSet>,
}

#[cfg(feature = "write")]
impl ToXml for TriangleSets {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), Error> {
        let prefix = match field {
            Some(id) => {
                let prefix = serializer.write_start(id.name, id.ns)?;
                serializer.end_start()?;
                Some((prefix, id.name))
            }
            None => {
                let _ = serializer.write_start("trianglesets", CORE_TRIANGLESET_NS)?;
                serializer.push(instant_xml::ser::Context {
                    default_ns: CORE_TRIANGLESET_NS,
                    prefixes: [],
                })?;

                serializer.end_start()?;
                Some((None, "trianglesets"))
            }
        };

        for set in &self.trianglesets {
            set.serialize(
                Some(Id {
                    ns: CORE_TRIANGLESET_NS,
                    name: "triangleset",
                }),
                serializer,
            )?;
        }

        if let Some((prefix, name)) = prefix {
            serializer.write_close(prefix, name)?;
        }

        Ok(())
    }
}

/// Triangle Set allows to define a collection of triangles as grouped collection
/// with a unique identifier for reusable references.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "triangleset"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(
    feature = "memory-optimized-read",
    xml(ns(CORE_TRIANGLESET_NS), rename = "triangleset")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleSet {
    /// Name of this set.
    #[cfg_attr(feature = "memory-optimized-read", xml(attribute))]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "name"))]
    pub name: String,

    /// A string based unique identifier of this set.
    #[cfg_attr(feature = "memory-optimized-read", xml(attribute))]
    pub identifier: String,

    /// A collection of Triangle references. See [`TriangleRef`] for more details.
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "ref", default))]
    pub triangle_ref: Vec<TriangleRef>,

    /// A collection of Triangle range references. See [`TriangleRefRange`] for more details.
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "refrange", default))]
    pub triangle_refrange: Vec<TriangleRefRange>,
}

#[cfg(feature = "write")]
impl ToXml for TriangleSet {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), Error> {
        let prefix = match field {
            Some(id) => {
                let prefix = serializer.write_start(id.name, id.ns)?;
                Some((prefix, id.name))
            }
            None => None,
        };

        //work around to ensure the attributes do not get the prefix
        //in the case that default root namespace is not triangleset namespace
        let attr_ns = if let CORE_TRIANGLESET_NS = serializer.default_ns() {
            CORE_TRIANGLESET_NS
        } else {
            serializer.default_ns()
        };

        serializer.write_attr("name", attr_ns, &self.name)?;
        serializer.write_attr("identifier", attr_ns, &self.identifier)?;
        serializer.end_start()?;

        for triangle_ref in &self.triangle_ref {
            triangle_ref.serialize(field, serializer)?;
        }

        for triangle_refrange in &self.triangle_refrange {
            triangle_refrange.serialize(field, serializer)?;
        }

        if let Some((prefix, name)) = prefix {
            serializer.write_close(prefix, name)?;
        }

        Ok(())
    }
}

/// A reference to a Triangle in the Mesh as an index into [`crate::core::mesh::Triangles::triangle`].
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_TRIANGLESET_NS), rename = "ref")
)]
pub struct TriangleRef {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub index: ResourceIndex,
}

/// A reference to continous Range of Triangles in the Mesh.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_TRIANGLESET_NS), rename = "refrange")
)]
pub struct TriangleRefRange {
    /// The start index of the range.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub startindex: ResourceIndex,

    /// The end idnex of the range.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub endindex: ResourceIndex,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            triangle_set::{TriangleRef, TriangleRefRange, TriangleSet, TriangleSets},
        },
        threemf_namespaces::{
            BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX, CORE_NS, CORE_TRIANGLESET_NS,
            CORE_TRIANGLESET_PREFIX,
        },
    };

    #[test]
    pub fn toxml_mesh_with_triangletest_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{CORE_NS}" xmlns:{BEAM_LATTICE_PREFIX}="{BEAM_LATTICE_NS}" xmlns:{CORE_TRIANGLESET_PREFIX}="{CORE_TRIANGLESET_NS}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles><t:trianglesets><t:triangleset name="Triangle Set 1" identifier="someUniqueID1"><t:ref index="2" /><t:refrange startindex="22" endindex="102" /></t:triangleset><t:triangleset name="Triangle Set 2" identifier="someUniqueID2"><t:refrange startindex="1" endindex="12" /><t:refrange startindex="100236" endindex="4566893" /></t:triangleset></t:trianglesets></mesh>"##,
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
            trianglesets: Some(TriangleSets {
                trianglesets: vec![
                    TriangleSet {
                        name: "Triangle Set 1".to_owned(),
                        identifier: "someUniqueID1".to_owned(),
                        triangle_ref: vec![TriangleRef { index: 2 }],
                        triangle_refrange: vec![TriangleRefRange {
                            startindex: 22,
                            endindex: 102,
                        }],
                    },
                    TriangleSet {
                        name: "Triangle Set 2".to_owned(),
                        identifier: "someUniqueID2".to_owned(),
                        triangle_ref: vec![],
                        triangle_refrange: vec![
                            TriangleRefRange {
                                startindex: 1,
                                endindex: 12,
                            },
                            TriangleRefRange {
                                startindex: 100236,
                                endindex: 4566893,
                            },
                        ],
                    },
                ],
            }),
            beamlattice: None,
        };
        let mesh_string = to_string(&mesh).unwrap();

        assert_eq!(mesh_string, xml_string);
    }

    #[test]
    pub fn toxml_trianglesets_test() {
        let xml_string = format!(
            r##"<trianglesets xmlns="{ns}"><triangleset name="Triangle Set 1" identifier="someUniqueID1"><ref index="2" /><refrange startindex="22" endindex="102" /></triangleset><triangleset name="Triangle Set 2" identifier="someUniqueID2"><refrange startindex="1" endindex="12" /><refrange startindex="100236" endindex="4566893" /></triangleset></trianglesets>"##,
            ns = CORE_TRIANGLESET_NS
        );
        let trianglesets = TriangleSets {
            trianglesets: vec![
                TriangleSet {
                    name: "Triangle Set 1".to_owned(),
                    identifier: "someUniqueID1".to_owned(),
                    triangle_ref: vec![TriangleRef { index: 2 }],
                    triangle_refrange: vec![TriangleRefRange {
                        startindex: 22,
                        endindex: 102,
                    }],
                },
                TriangleSet {
                    name: "Triangle Set 2".to_owned(),
                    identifier: "someUniqueID2".to_owned(),
                    triangle_ref: vec![],
                    triangle_refrange: vec![
                        TriangleRefRange {
                            startindex: 1,
                            endindex: 12,
                        },
                        TriangleRefRange {
                            startindex: 100236,
                            endindex: 4566893,
                        },
                    ],
                },
            ],
        };

        let trianglesets_string = to_string(&trianglesets).unwrap();

        assert_eq!(trianglesets_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;

    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            triangle_set::{TriangleRef, TriangleRefRange, TriangleSet, TriangleSets},
        },
        threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX},
    };

    #[test]
    pub fn fromxml_mesh_with_triangleset_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}" xmlns:{}="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles><t:trianglesets><t:triangleset name="Triangle Set 1" identifier="someUniqueID1"><t:ref index="2" /><t:refrange startindex="22" endindex="102" /></t:triangleset><t:triangleset name="Triangle Set 2" identifier="someUniqueID2"><t:refrange startindex="1" endindex="12" /><t:refrange startindex="100236" endindex="4566893" /></t:triangleset></t:trianglesets></mesh>"##,
            CORE_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
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
                trianglesets: Some(TriangleSets {
                    trianglesets: vec![
                        TriangleSet {
                            name: "Triangle Set 1".to_owned(),
                            identifier: "someUniqueID1".to_owned(),
                            triangle_ref: vec![TriangleRef { index: 2 }],
                            triangle_refrange: vec![TriangleRefRange {
                                startindex: 22,
                                endindex: 102,
                            }],
                        },
                        TriangleSet {
                            name: "Triangle Set 2".to_owned(),
                            identifier: "someUniqueID2".to_owned(),
                            triangle_ref: vec![],
                            triangle_refrange: vec![
                                TriangleRefRange {
                                    startindex: 1,
                                    endindex: 12,
                                },
                                TriangleRefRange {
                                    startindex: 100236,
                                    endindex: 4566893,
                                },
                            ],
                        },
                    ],
                }),
                beamlattice: None,
            }
        )
    }

    #[test]
    pub fn fromxml_trianglesets_test() {
        let xml_string = format!(
            r##"<trianglesets xmlns="{ns}"><triangleset name="Triangle Set 1" identifier="someUniqueID1"><ref index="2" /><refrange startindex="22" endindex="102" /></triangleset><triangleset name="Triangle Set 2" identifier="someUniqueID2"><refrange startindex="1" endindex="12" /><refrange startindex="100236" endindex="4566893" /></triangleset></trianglesets>"##,
            ns = CORE_TRIANGLESET_NS
        );
        let trianglesets = from_str::<TriangleSets>(&xml_string).unwrap();

        assert_eq!(
            trianglesets,
            TriangleSets {
                trianglesets: vec![
                    TriangleSet {
                        name: "Triangle Set 1".to_owned(),
                        identifier: "someUniqueID1".to_owned(),
                        triangle_ref: vec![TriangleRef { index: 2 }],
                        triangle_refrange: vec![TriangleRefRange {
                            startindex: 22,
                            endindex: 102,
                        }],
                    },
                    TriangleSet {
                        name: "Triangle Set 2".to_owned(),
                        identifier: "someUniqueID2".to_owned(),
                        triangle_ref: vec![],
                        triangle_refrange: vec![
                            TriangleRefRange {
                                startindex: 1,
                                endindex: 12,
                            },
                            TriangleRefRange {
                                startindex: 100236,
                                endindex: 4566893,
                            },
                        ],
                    },
                ],
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use serde_roxmltree::from_str;

    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            triangle_set::{TriangleRef, TriangleRefRange, TriangleSet, TriangleSets},
        },
        threemf_namespaces::{CORE_NS, CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX},
    };

    #[test]
    pub fn fromxml_mesh_with_triangleset_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}" xmlns:{}="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" /><triangle v1="0" v2="2" v3="3" /></triangles><t:trianglesets><t:triangleset name="Triangle Set 1" identifier="someUniqueID1"><t:ref index="2" /><t:refrange startindex="22" endindex="102" /></t:triangleset><t:triangleset name="Triangle Set 2" identifier="someUniqueID2"><t:refrange startindex="1" endindex="12" /><t:refrange startindex="100236" endindex="4566893" /></t:triangleset></t:trianglesets></mesh>"##,
            CORE_NS, CORE_TRIANGLESET_PREFIX, CORE_TRIANGLESET_NS
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
                trianglesets: Some(TriangleSets {
                    trianglesets: vec![
                        TriangleSet {
                            name: "Triangle Set 1".to_owned(),
                            identifier: "someUniqueID1".to_owned(),
                            triangle_ref: vec![TriangleRef { index: 2 }],
                            triangle_refrange: vec![TriangleRefRange {
                                startindex: 22,
                                endindex: 102,
                            }],
                        },
                        TriangleSet {
                            name: "Triangle Set 2".to_owned(),
                            identifier: "someUniqueID2".to_owned(),
                            triangle_ref: vec![],
                            triangle_refrange: vec![
                                TriangleRefRange {
                                    startindex: 1,
                                    endindex: 12,
                                },
                                TriangleRefRange {
                                    startindex: 100236,
                                    endindex: 4566893,
                                },
                            ],
                        },
                    ],
                }),
                beamlattice: None,
            }
        )
    }

    #[test]
    pub fn fromxml_trianglesets_test() {
        let xml_string = format!(
            r##"<trianglesets xmlns="{ns}"><triangleset name="Triangle Set 1" identifier="someUniqueID1"><ref index="2" /><refrange startindex="22" endindex="102" /></triangleset><triangleset name="Triangle Set 2" identifier="someUniqueID2"><refrange startindex="1" endindex="12" /><refrange startindex="100236" endindex="4566893" /></triangleset></trianglesets>"##,
            ns = CORE_TRIANGLESET_NS
        );
        let trianglesets = from_str::<TriangleSets>(&xml_string).unwrap();

        assert_eq!(
            trianglesets,
            TriangleSets {
                trianglesets: vec![
                    TriangleSet {
                        name: "Triangle Set 1".to_owned(),
                        identifier: "someUniqueID1".to_owned(),
                        triangle_ref: vec![TriangleRef { index: 2 }],
                        triangle_refrange: vec![TriangleRefRange {
                            startindex: 22,
                            endindex: 102,
                        }],
                    },
                    TriangleSet {
                        name: "Triangle Set 2".to_owned(),
                        identifier: "someUniqueID2".to_owned(),
                        triangle_ref: vec![],
                        triangle_refrange: vec![
                            TriangleRefRange {
                                startindex: 1,
                                endindex: 12,
                            },
                            TriangleRefRange {
                                startindex: 100236,
                                endindex: 4566893,
                            },
                        ],
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_triangleset_test_correction() {
        let xml_string = r##"<triangleset name="Triangle Set 1" identifier="someUniqueID1"><ref index="2" /><refrange startindex="22" endindex="102" /></triangleset>"##.to_string();
        let trianglesets = from_str::<TriangleSet>(&xml_string).unwrap();

        assert_eq!(
            trianglesets,
            TriangleSet {
                name: "Triangle Set 1".to_owned(),
                identifier: "someUniqueID1".to_owned(),
                triangle_ref: vec![TriangleRef { index: 2 }],
                triangle_refrange: vec![TriangleRefRange {
                    startindex: 22,
                    endindex: 102,
                }],
            }
        );
    }
}

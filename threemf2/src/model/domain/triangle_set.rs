#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

use crate::model::domain::any_extensions::{AnyAttributeExtensions, AnyExtensions};
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::{
    model::{ResourceIndex, StrResource},
    threemf_namespaces::CORE_TRIANGLESET_NS,
};

/// Collection of Triangle Set. See [`TriangleSet`] for more details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleSets {
    /// Field of triangle set
    pub trianglesets: Vec<TriangleSet>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    TriangleSets {
        tag: "trianglesets",
        ns: CORE_TRIANGLESET_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
        },
        elements: {
            trianglesets: Vec<TriangleSet> => (CORE_TRIANGLESET_NS, "trianglesets"),
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// Triangle Set allows to define a collection of triangles as grouped collection
/// with a unique identifier for reusable references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriangleSet {
    /// Name of this set.
    pub name: StrResource,

    /// A string based unique identifier of this set.
    pub identifier: StrResource,

    /// A collection of Triangle references. See [`TriangleRef`] for more details.
    pub triangle_ref: Vec<TriangleRef>,

    /// A collection of Triangle range references. See [`TriangleRefRange`] for more details.
    pub triangle_refrange: Vec<TriangleRefRange>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    TriangleSet {
        tag: "triangleset",
        ns: CORE_TRIANGLESET_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            name: StrResource => ("", "name"),
            identifier: StrResource => ("", "identifier"),
        },
        elements: {
            triangle_ref: Vec<TriangleRef> => (CORE_TRIANGLESET_NS, "triangle_ref"),
            triangle_refrange: Vec<TriangleRefRange> => (CORE_TRIANGLESET_NS, "triangle_refrange"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// A reference to a Triangle in the Mesh as an index into [`Triangles`](crate::model::domain::mesh::Triangles).
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(CORE_TRIANGLESET_NS), rename = "ref", force_prefix)
)]
pub struct TriangleRef {
    /// Index of the Triangle
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub index: ResourceIndex,
}

/// A reference to continous Range of Triangles in the Mesh.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(CORE_TRIANGLESET_NS), rename = "refrange", force_prefix)
)]
pub struct TriangleRefRange {
    /// The start index of the range.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub startindex: ResourceIndex,

    /// The end idnex of the range.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub endindex: ResourceIndex,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            triangle_set::{TriangleRef, TriangleRefRange, TriangleSet, TriangleSets},
        },
        model::{OptionalResourceId, OptionalResourceIndex},
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
            trianglesets: Some(TriangleSets {
                trianglesets: vec![
                    TriangleSet {
                        name: "Triangle Set 1".into(),
                        identifier: "someUniqueID1".into(),
                        triangle_ref: vec![TriangleRef { index: 2 }],
                        triangle_refrange: vec![TriangleRefRange {
                            startindex: 22,
                            endindex: 102,
                        }],
                        any_extensions: Default::default(),
                    },
                    TriangleSet {
                        name: "Triangle Set 2".into(),
                        identifier: "someUniqueID2".into(),
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
                        any_extensions: Default::default(),
                    },
                ],
                any_extensions: Default::default(),
            }),
            beamlattice: None,
            any_extensions: Default::default(),
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
                    name: "Triangle Set 1".into(),
                    identifier: "someUniqueID1".into(),
                    triangle_ref: vec![TriangleRef { index: 2 }],
                    triangle_refrange: vec![TriangleRefRange {
                        startindex: 22,
                        endindex: 102,
                    }],
                    any_extensions: Default::default(),
                },
                TriangleSet {
                    name: "Triangle Set 2".into(),
                    identifier: "someUniqueID2".into(),
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
                    any_extensions: Default::default(),
                },
            ],
            any_extensions: Default::default(),
        };

        let trianglesets_string = to_string(&trianglesets).unwrap();

        assert_eq!(trianglesets_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;

    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            triangle_set::{TriangleRef, TriangleRefRange, TriangleSet, TriangleSets},
        },
        model::{OptionalResourceId, OptionalResourceIndex},
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
                trianglesets: Some(TriangleSets {
                    trianglesets: vec![
                        TriangleSet {
                            name: "Triangle Set 1".into(),
                            identifier: "someUniqueID1".into(),
                            triangle_ref: vec![TriangleRef { index: 2 }],
                            triangle_refrange: vec![TriangleRefRange {
                                startindex: 22,
                                endindex: 102,
                            }],
                            any_extensions: Default::default(),
                        },
                        TriangleSet {
                            name: "Triangle Set 2".into(),
                            identifier: "someUniqueID2".into(),
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
                            any_extensions: Default::default(),
                        },
                    ],
                    any_extensions: Default::default(),
                }),
                beamlattice: None,
                any_extensions: Default::default(),
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
                        name: "Triangle Set 1".into(),
                        identifier: "someUniqueID1".into(),
                        triangle_ref: vec![TriangleRef { index: 2 }],
                        triangle_refrange: vec![TriangleRefRange {
                            startindex: 22,
                            endindex: 102,
                        }],
                        any_extensions: Default::default(),
                    },
                    TriangleSet {
                        name: "Triangle Set 2".into(),
                        identifier: "someUniqueID2".into(),
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
                        any_extensions: Default::default(),
                    },
                ],
                any_extensions: Default::default(),
            }
        );
    }
}

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{transform::Transform, types::ResourceId},
    threemf_namespaces::BOOLEAN_NS,
};

/// Represents a boolean shape object that applies boolean operations to a base object.
///
/// A boolean shape defines a new object by applying a sequence of boolean operations
/// (union, difference, intersection) between a base object and one or more operand objects.
/// The operations are applied sequentially from left to right.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BOOLEAN_NS), rename = "booleanshape")
)]
pub struct BooleanShape {
    /// Reference to the base object ID to apply boolean operations.
    /// The base object must be a model object defining a shape (mesh, booleanshape,
    /// or other extension shapes), but NOT a components object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, rename = "objectid")
    )]
    pub objectid: ResourceId,

    /// The boolean operation to perform on the base object with the operands.
    /// Default is `BooleanOperation::Union`.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub operation: BooleanOperation,

    /// Optional transform to apply to the base object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub transform: Option<Transform>,

    /// Optional path to the base object file (for Production extension).
    /// Only valid in root model files when used with Production extension.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub path: Option<String>,

    /// The sequence of boolean operations to apply to the base object.
    /// Must contain at least one boolean operation.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(rename = "boolean")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(default, rename = "boolean"))]
    pub booleans: Vec<Boolean>,
}

/// Represents a single boolean operation operand.
///
/// A boolean operation references a mesh object and optionally applies a transform
/// before performing the boolean operation with the base object.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BOOLEAN_NS), rename = "boolean")
)]
pub struct Boolean {
    /// Reference to the mesh object ID to use as the boolean operand.
    /// Must be a triangle mesh object of type "model", and must NOT contain
    /// shapes defined in other extensions.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, rename = "objectid")
    )]
    pub objectid: ResourceId,

    /// Optional transform to apply to the operand object.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub transform: Option<Transform>,

    /// Optional path to the operand object file (for Production extension).
    /// Only valid in root model files when used with Production extension.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub path: Option<String>,
}

/// Specifies the type of boolean operation to perform.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, rename_all = "lowercase")
)]
pub enum BooleanOperation {
    /// Union: Merges the shapes together.
    /// The resulting object is the merger of all shapes.
    #[default]
    Union,

    /// Difference: Subtracts operand shapes from the base.
    /// The resulting object is the base shape minus all operand shapes.
    Difference,

    /// Intersection: Keeps only the common (overlapping) volume.
    /// The resulting object is the portion common to all objects.
    Intersection,
}

impl From<String> for BooleanOperation {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "union" => Self::Union,
            "difference" => Self::Difference,
            "intersection" => Self::Intersection,
            _ => Self::Union,
        }
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            boolean::{Boolean, BooleanOperation, BooleanShape},
            object::Object,
            transform::Transform,
        },
        threemf_namespaces::{BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS},
    };

    #[test]
    pub fn toxml_boolean_shape_union_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union"><boolean objectid="2" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = BooleanShape {
            objectid: 1,
            operation: BooleanOperation::Union,
            transform: None,
            path: None,
            booleans: vec![Boolean {
                objectid: 2,
                transform: None,
                path: None,
            }],
        };
        let boolean_shape_string = to_string(&boolean_shape).unwrap();

        assert_eq!(boolean_shape_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_shape_difference_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="3" operation="difference"><boolean objectid="4" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = BooleanShape {
            objectid: 3,
            operation: BooleanOperation::Difference,
            transform: None,
            path: None,
            booleans: vec![Boolean {
                objectid: 4,
                transform: None,
                path: None,
            }],
        };
        let boolean_shape_string = to_string(&boolean_shape).unwrap();

        assert_eq!(boolean_shape_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_shape_intersection_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="5" operation="intersection"><boolean objectid="6" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = BooleanShape {
            objectid: 5,
            operation: BooleanOperation::Intersection,
            transform: None,
            path: None,
            booleans: vec![Boolean {
                objectid: 6,
                transform: None,
                path: None,
            }],
        };
        let boolean_shape_string = to_string(&boolean_shape).unwrap();

        assert_eq!(boolean_shape_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_shape_with_transform_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union" transform="1.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 1.000000 10.000000 0.000000 0.000000"><boolean objectid="2" transform="0.500000 0.000000 0.000000 0.000000 0.500000 0.000000 0.000000 0.000000 0.500000 0.000000 0.000000 0.000000" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = BooleanShape {
            objectid: 1,
            operation: BooleanOperation::Union,
            transform: Some(Transform([
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0,
            ])),
            path: None,
            booleans: vec![Boolean {
                objectid: 2,
                transform: Some(Transform([
                    0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0,
                ])),
                path: None,
            }],
        };
        let boolean_shape_string = to_string(&boolean_shape).unwrap();

        assert_eq!(boolean_shape_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_shape_multiple_booleans_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union"><boolean objectid="2" /><boolean objectid="3" /><boolean objectid="4" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = BooleanShape {
            objectid: 1,
            operation: BooleanOperation::default(),
            transform: None,
            path: None,
            booleans: vec![
                Boolean {
                    objectid: 2,
                    transform: None,
                    path: None,
                },
                Boolean {
                    objectid: 3,
                    transform: None,
                    path: None,
                },
                Boolean {
                    objectid: 4,
                    transform: None,
                    path: None,
                },
            ],
        };
        let boolean_shape_string = to_string(&boolean_shape).unwrap();

        assert_eq!(boolean_shape_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_operation_test() {
        let xml_string = "union";
        let operation = BooleanOperation::Union;
        let operation_string = to_string(&operation).unwrap();

        assert_eq!(operation_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_operation_difference_test() {
        let xml_string = "difference";
        let operation = BooleanOperation::Difference;
        let operation_string = to_string(&operation).unwrap();

        assert_eq!(operation_string, xml_string);
    }

    #[test]
    pub fn toxml_boolean_operation_intersection_test() {
        let xml_string = "intersection";
        let operation = BooleanOperation::Intersection;
        let operation_string = to_string(&operation).unwrap();

        assert_eq!(operation_string, xml_string);
    }

    #[test]
    pub fn toxml_obj_with_booleanshape_test() {
        let obj = Object {
            id: 100,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            mesh: None,
            components: None,
            booleanshape: Some(BooleanShape {
                objectid: 95,
                operation: BooleanOperation::Difference,
                transform: None,
                path: None,
                booleans: vec![
                    Boolean {
                        objectid: 66,
                        transform: None,
                        path: None,
                    },
                    Boolean {
                        objectid: 213,
                        transform: None,
                        path: None,
                    },
                ],
            }),
        };

        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="100"><bo:booleanshape objectid="95" operation="difference"><bo:boolean objectid="66" /><bo:boolean objectid="213" /></bo:booleanshape></object>"##,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS,
        );

        let obj_string = to_string(&obj).unwrap();

        assert_eq!(xml_string, obj_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex, ResourceId,
            boolean::{Boolean, BooleanOperation, BooleanShape},
            mesh::Mesh,
            object::Object,
            transform::Transform,
        },
        threemf_namespaces::{BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS},
    };

    #[test]
    pub fn fromxml_boolean_shape_union_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union"><boolean objectid="2" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 1,
                operation: BooleanOperation::Union,
                transform: None,
                path: None,
                booleans: vec![Boolean {
                    objectid: 2,
                    transform: None,
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_difference_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="3" operation="difference"><boolean objectid="4" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 3,
                operation: BooleanOperation::Difference,
                transform: None,
                path: None,
                booleans: vec![Boolean {
                    objectid: 4,
                    transform: None,
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_intersection_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="5" operation="intersection"><boolean objectid="6" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 5,
                operation: BooleanOperation::Intersection,
                transform: None,
                path: None,
                booleans: vec![Boolean {
                    objectid: 6,
                    transform: None,
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_with_transform_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union" transform="1 0 0 0 1 0 0 0 1 10 0 0"><boolean objectid="2" transform="0.5 0 0 0 0.5 0 0 0 0.5 0 0 0" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 1,
                operation: BooleanOperation::Union,
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0,
                ])),
                path: None,
                booleans: vec![Boolean {
                    objectid: 2,
                    transform: Some(Transform([
                        0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0,
                    ])),
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_multiple_booleans_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union"><boolean objectid="2" /><boolean objectid="3" /><boolean objectid="4" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 1,
                operation: BooleanOperation::Union,
                transform: None,
                path: None,
                booleans: vec![
                    Boolean {
                        objectid: 2,
                        transform: None,
                        path: None,
                    },
                    Boolean {
                        objectid: 3,
                        transform: None,
                        path: None,
                    },
                    Boolean {
                        objectid: 4,
                        transform: None,
                        path: None,
                    },
                ],
            }
        );
    }

    #[test]
    pub fn fromxml_obj_with_booleanshape_test() {
        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" id="100"><bo:booleanshape objectid="95" operation="difference"><bo:boolean objectid="66" /><bo:boolean objectid="213" /></bo:booleanshape></object>"##,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS,
        );

        let obj = from_str::<Object>(&xml_string).unwrap();

        assert_eq!(
            obj,
            Object {
                id: 100,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                mesh: None,
                components: None,
                booleanshape: Some(BooleanShape {
                    objectid: 95,
                    operation: BooleanOperation::Difference,
                    transform: None,
                    path: None,
                    booleans: vec![
                        Boolean {
                            objectid: 66,
                            transform: None,
                            path: None,
                        },
                        Boolean {
                            objectid: 213,
                            transform: None,
                            path: None
                        }
                    ]
                })
            }
        )
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::{
        core::boolean::{Boolean, BooleanOperation, BooleanShape},
        core::transform::Transform,
        threemf_namespaces::BOOLEAN_NS,
    };

    #[test]
    pub fn fromxml_boolean_shape_union_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union"><boolean objectid="2" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 1,
                operation: BooleanOperation::Union,
                transform: None,
                path: None,
                booleans: vec![Boolean {
                    objectid: 2,
                    transform: None,
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_difference_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="3" operation="difference"><boolean objectid="4" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 3,
                operation: BooleanOperation::Difference,
                transform: None,
                path: None,
                booleans: vec![Boolean {
                    objectid: 4,
                    transform: None,
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_intersection_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="5" operation="intersection"><boolean objectid="6" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 5,
                operation: BooleanOperation::Intersection,
                transform: None,
                path: None,
                booleans: vec![Boolean {
                    objectid: 6,
                    transform: None,
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_with_transform_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union" transform="1 0 0 0 1 0 0 0 1 10 0 0"><boolean objectid="2" transform="0.5 0 0 0 0.5 0 0 0 0.5 0 0 0" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 1,
                operation: BooleanOperation::Union,
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0,
                ])),
                path: None,
                booleans: vec![Boolean {
                    objectid: 2,
                    transform: Some(Transform([
                        0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0,
                    ])),
                    path: None,
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_boolean_shape_multiple_booleans_test() {
        let xml_string = format!(
            r#"<booleanshape xmlns="{}" objectid="1" operation="union"><boolean objectid="2" /><boolean objectid="3" /><boolean objectid="4" /></booleanshape>"#,
            BOOLEAN_NS
        );
        let boolean_shape = from_str::<BooleanShape>(&xml_string).unwrap();

        assert_eq!(
            boolean_shape,
            BooleanShape {
                objectid: 1,
                operation: BooleanOperation::Union,
                transform: None,
                path: None,
                booleans: vec![
                    Boolean {
                        objectid: 2,
                        transform: None,
                        path: None,
                    },
                    Boolean {
                        objectid: 3,
                        transform: None,
                        path: None,
                    },
                    Boolean {
                        objectid: 4,
                        transform: None,
                        path: None,
                    },
                ],
            }
        );
    }

    // Note: BooleanOperation is always used as an attribute in BooleanShape.
    // Testing it in isolation requires special handling, so we skip those tests.
}

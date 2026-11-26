#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

use crate::model::domain::any_extensions::{AnyAttributeExtensions, AnyExtensions};
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::{
    model::domain::transform::Transform,
    model::{PathResource, ResourceId},
    threemf_namespaces::BOOLEAN_NS,
};

/// Represents a boolean shape object that applies boolean operations to a base object.
///
/// A boolean shape defines a new object by applying a sequence of boolean operations
/// (union, difference, intersection) between a base object and one or more operand objects.
/// The operations are applied sequentially from left to right.
#[derive(Debug, PartialEq, Clone)]
pub struct BooleanShape {
    /// Reference to the base object ID to apply boolean operations.
    /// The base object must be a model object defining a shape (mesh, booleanshape,
    /// or other extension shapes), but NOT a components object.
    pub objectid: ResourceId,

    /// The boolean operation to perform on the base object with the operands.
    /// Default is `BooleanOperation::Union`.
    pub operation: BooleanOperation,

    /// Optional transform to apply to the base object.
    pub transform: Option<Transform>,

    /// Optional path to the base object file (for Production extension).
    /// Only valid in root model files when used with Production extension.
    pub path: Option<PathResource>,

    /// The sequence of boolean operations to apply to the base object.
    /// Must contain at least one boolean operation.
    pub booleans: Vec<Boolean>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    BooleanShape {
        tag: "booleanshape",
        ns: BOOLEAN_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            objectid: ResourceId => ("", "objectid"),
            operation: BooleanOperation => ("", "operation"),
            transform: Option<Transform> => ("", "transform"),
            path: Option<PathResource> => ("", "path"),
        },
        elements: {
            booleans: Vec<Boolean> => (BOOLEAN_NS, "boolean"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// Represents a single boolean operation operand.
///
/// A boolean operation references a mesh object and optionally applies a transform
/// before performing the boolean operation with the base object.
#[derive(Debug, PartialEq, Clone)]
pub struct Boolean {
    /// Reference to the mesh object ID to use as the boolean operand.
    /// Must be a triangle mesh object of type "model", and must NOT contain
    /// shapes defined in other extensions.
    pub objectid: ResourceId,

    /// Optional transform to apply to the operand object.
    pub transform: Option<Transform>,

    /// Optional path to the operand object file (for Production extension).
    /// Only valid in root model files when used with Production extension.
    pub path: Option<PathResource>,

    /// Foreign attributes (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Boolean {
        tag: "boolean",
        ns: BOOLEAN_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
            objectid: ResourceId => ("", "objectid"),
            transform: Option<Transform> => ("", "transform"),
            path: Option<PathResource> => ("", "path"),
        },
        elements: {
        },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// Specifies the type of boolean operation to perform.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
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
        model::domain::{
            boolean::{Boolean, BooleanOperation, BooleanShape},
            object::{Object, ObjectKind},
            transform::Transform,
        },
        model::{OptionalResourceId, OptionalResourceIndex},
        threemf_namespaces::{
            BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS, PROD_NS, PROD_PREFIX, SLICE_NS, SLICE_PREFIX,
        },
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
                any_extensions: Default::default(),
            }],
            any_extensions: Default::default(),
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
                any_extensions: Default::default(),
            }],
            any_extensions: Default::default(),
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
                any_extensions: Default::default(),
            }],
            any_extensions: Default::default(),
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
                any_extensions: Default::default(),
            }],
            any_extensions: Default::default(),
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
                    any_extensions: Default::default(),
                },
                Boolean {
                    objectid: 3,
                    transform: None,
                    path: None,
                    any_extensions: Default::default(),
                },
                Boolean {
                    objectid: 4,
                    transform: None,
                    path: None,
                    any_extensions: Default::default(),
                },
            ],
            any_extensions: Default::default(),
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
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,

            kind: Some(ObjectKind::BooleanShape(BooleanShape {
                objectid: 95,
                operation: BooleanOperation::Difference,
                transform: None,
                path: None,
                booleans: vec![
                    Boolean {
                        objectid: 66,
                        transform: None,
                        path: None,
                        any_extensions: Default::default(),
                    },
                    Boolean {
                        objectid: 213,
                        transform: None,
                        path: None,
                        any_extensions: Default::default(),
                    },
                ],
                any_extensions: Default::default(),
            })),
            any_extensions: Default::default(),
            metadatagroup: Default::default(),
        };

        let xml_string = format!(
            r##"<object xmlns="{}" xmlns:{}="{}" xmlns:{}="{}" xmlns:{}="{}" id="100"><bo:booleanshape objectid="95" operation="difference"><bo:boolean objectid="66" /><bo:boolean objectid="213" /></bo:booleanshape></object>"##,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS, PROD_PREFIX, PROD_NS, SLICE_PREFIX, SLICE_NS,
        );

        let obj_string = to_string(&obj).unwrap();

        assert_eq!(obj_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            boolean::{Boolean, BooleanOperation, BooleanShape},
            object::{Object, ObjectKind},
            transform::Transform,
        },
        model::{OptionalResourceId, OptionalResourceIndex},
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
                    any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
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
                    any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
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
                    any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
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
                    any_extensions: Default::default(),
                }],
                any_extensions: Default::default(),
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
                        any_extensions: Default::default(),
                    },
                    Boolean {
                        objectid: 3,
                        transform: None,
                        path: None,
                        any_extensions: Default::default(),
                    },
                    Boolean {
                        objectid: 4,
                        transform: None,
                        path: None,
                        any_extensions: Default::default(),
                    },
                ],
                any_extensions: Default::default(),
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
                slicestackid: OptionalResourceId::none(),
                slicepath: None,
                meshresolution: None,

                kind: Some(ObjectKind::BooleanShape(BooleanShape {
                    objectid: 95,
                    operation: BooleanOperation::Difference,
                    transform: None,
                    path: None,
                    booleans: vec![
                        Boolean {
                            objectid: 66,
                            transform: None,
                            path: None,
                            any_extensions: Default::default(),
                        },
                        Boolean {
                            objectid: 213,
                            transform: None,
                            path: None,
                            any_extensions: Default::default(),
                        }
                    ],
                    any_extensions: Default::default(),
                })),
                any_extensions: Default::default(),
                metadatagroup: Default::default(),
            }
        )
    }
}

#[cfg(test)]
mod from_string_tests {
    use super::BooleanOperation;

    #[test]
    fn boolean_operation_from_string() {
        assert_eq!(
            BooleanOperation::from("union".to_string()),
            BooleanOperation::Union
        );
        assert_eq!(
            BooleanOperation::from("UNION".to_string()),
            BooleanOperation::Union
        );
        assert_eq!(
            BooleanOperation::from("difference".to_string()),
            BooleanOperation::Difference
        );
        assert_eq!(
            BooleanOperation::from("DIFFERENCE".to_string()),
            BooleanOperation::Difference
        );
        assert_eq!(
            BooleanOperation::from("intersection".to_string()),
            BooleanOperation::Intersection
        );
        assert_eq!(
            BooleanOperation::from("INTERSECTION".to_string()),
            BooleanOperation::Intersection
        );
        assert_eq!(
            BooleanOperation::from("unknown".to_string()),
            BooleanOperation::Union
        );
    }
}

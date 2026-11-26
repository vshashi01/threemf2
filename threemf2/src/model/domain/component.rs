#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
#[cfg(feature = "write")]
use crate::threemf_namespaces::PROD_PREFIX;
#[cfg(any(feature = "read", feature = "write"))]
use crate::threemf_namespaces::{CORE_NS, PROD_NS};
use crate::{
    model::domain::{
        any_extensions::{AnyAttributeExtensions, AnyExtensions},
        transform::Transform,
    },
    model::{
        PathResource, {ResourceId, UuidResource},
    },
};

/// Container for component references that make up a components object.
///
/// Components allow building complex objects by referencing other objects
/// with transforms, enabling hierarchical model construction.
#[derive(PartialEq, Debug, Clone)]
pub struct Components {
    /// Component references within this components object.
    pub component: Vec<Component>,

    /// Foreign attributes of the components element (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Components {
        tag: "components",
        ns: CORE_NS,
        prefixes: [(PROD_PREFIX, PROD_NS)],
        force_prefix: false,
        attributes: {},
        elements: { component: Vec<Component> => (CORE_NS, "component") },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// Reference to an object with an applied transform.
///
/// Components enable reusing objects in different positions, orientations,
/// and scales within a components object.
#[derive(PartialEq, Debug, Clone)]
pub struct Component {
    /// Unique identifier of the referenced object.
    pub objectid: ResourceId,

    /// Optional transform applied to the referenced object.
    pub transform: Option<Transform>,

    /// Optional path to the referenced object (Production extension).
    pub path: Option<PathResource>,

    /// Optional UUID for the referenced object (Production extension).
    pub uuid: Option<UuidResource>,

    /// Foreign attributes and trailing child elements of the component
    /// (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Component {
        tag: "component",
        ns: CORE_NS,
        prefixes: [(PROD_PREFIX, PROD_NS)],
        force_prefix: false,
        attributes: {
            objectid: ResourceId => ("", "objectid"),
            transform: Option<Transform> => ("", "transform"),
            path: Option<PathResource> => (PROD_NS, "path"),
            uuid: Option<UuidResource> => (PROD_NS, "UUID"),
        },
        elements: {},
        any: any_extensions: AnyExtensions,
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::transform::Transform,
        model::{PathResource, UuidResource},
        threemf_namespaces::{CORE_NS, PROD_NS, PROD_PREFIX},
    };

    use super::{Component, Components};

    #[test]
    pub fn toxml_component_test() {
        let xml_string = format!(
            r#"<component xmlns="{}" xmlns:{}="{}" objectid="3" transform="1.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 1.000000 35.000000 35.000000 5.100000" />"#,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let component = Component {
            objectid: 3,
            transform: Some(Transform([
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1,
            ])),
            path: None,
            uuid: None,
            any_extensions: Default::default(),
        };
        let component_string = to_string(&component).unwrap();

        assert_eq!(component_string, xml_string);
    }

    #[test]
    pub fn toxml_production_component_test() {
        let xml_string = format!(
            r#"<component xmlns="{}" xmlns:{}="{}" objectid="3" transform="1.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 1.000000 35.000000 35.000000 5.100000" {}:path="/somePath/Component" {}:UUID="someComponentUUID" />"#,
            CORE_NS, PROD_PREFIX, PROD_NS, PROD_PREFIX, PROD_PREFIX
        );
        let component = Component {
            objectid: 3,
            transform: Some(Transform([
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1,
            ])),
            path: Some(PathResource::try_from("//somePath//Component").unwrap()),
            uuid: Some(UuidResource::from("someComponentUUID")),
            any_extensions: Default::default(),
        };
        let component_string = to_string(&component).unwrap();

        assert_eq!(component_string, xml_string);
    }

    #[test]
    pub fn toxml_components_test() {
        let xml_string = format!(
            r#"<components xmlns="{}" xmlns:{}="{}"><component objectid="4" transform="1.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 1.000000 35.000000 35.000000 5.100000" /><component objectid="5" /></components>"#,
            CORE_NS, PROD_PREFIX, PROD_NS,
        );
        let components = Components {
            component: vec![
                Component {
                    objectid: 4,
                    transform: Some(Transform([
                        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1,
                    ])),
                    path: None,
                    uuid: None,
                    any_extensions: Default::default(),
                },
                Component {
                    objectid: 5,
                    transform: None,
                    path: None,
                    uuid: None,
                    any_extensions: Default::default(),
                },
            ],
            any_extensions: Default::default(),
        };
        let components_string = to_string(&components).unwrap();

        assert_eq!(components_string, xml_string);
    }
}

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::transform::Transform,
        model::{PathResource, UuidResource},
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Component, Components};

    #[test]
    pub fn fromxml_component_test() {
        let xml_string = format!(
            r#"<component xmlns="{}" objectid="3" transform="1.0000 0.0000 0.0000 0.0000 1.0000 0.0000 0.0000 0.0000 1.0000 35 35 5.1000" />"#,
            CORE_NS
        );
        let component = from_str::<Component>(&xml_string).unwrap();

        assert_eq!(
            component,
            Component {
                objectid: 3,
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.10
                ])),
                path: None,
                uuid: None,
                any_extensions: Default::default(),
            }
        )
    }

    #[test]
    pub fn fromxml_production_component_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<component xmlns="{}" xmlns:{}="{}" objectid="3" transform="1.0000 0.0000 0.0000 0.0000 1.0000 0.0000 0.0000 0.0000 1.0000 35 35 5.1000" {}:path="//somePath//Component" {}:UUID="someComponentUUID" />"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX, CUSTOM_PROD_PREFIX
        );
        let component = from_str::<Component>(&xml_string).unwrap();

        assert_eq!(
            component,
            Component {
                objectid: 3,
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.10
                ])),
                path: Some(PathResource::try_from("//somePath//Component").unwrap()),
                uuid: Some(UuidResource::from("someComponentUUID")),
                any_extensions: Default::default(),
            }
        )
    }

    #[test]
    pub fn fromxml_components_test() {
        let xml_string = format!(
            r#"<components xmlns="{}"><component objectid="4" transform="1.000000 0.000000 0.000000 0.000000 1.000000 0.000000 0.000000 0.000000 1.000000 35.000000 35.000000 5.100000" /><component objectid="5" /></components>"#,
            CORE_NS
        );
        let components = from_str::<Components>(&xml_string).unwrap();

        assert_eq!(
            components,
            Components {
                component: vec![
                    Component {
                        objectid: 4,
                        transform: Some(Transform([
                            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1,
                        ])),
                        path: None,
                        uuid: None,
                        any_extensions: Default::default(),
                    },
                    Component {
                        objectid: 5,
                        transform: None,
                        path: None,
                        uuid: None,
                        any_extensions: Default::default(),
                    },
                ],
                any_extensions: Default::default(),
            }
        );
    }
}

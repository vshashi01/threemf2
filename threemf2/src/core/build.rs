#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{transform::Transform, types::ResourceId},
    threemf_namespaces::{CORE_NS, PROD_NS},
};

/// Contains the build configuration for a 3MF model, listing items to be printed.
///
/// The build section specifies which objects from the resources should be included
/// in the final 3D print, along with their transforms and metadata.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Debug, Clone)]
#[cfg_attr(any(feature="write", feature="memory-optimized-read"), xml(ns(CORE_NS, p=PROD_NS), rename = "build"))]
pub struct Build {
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "UUID"))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(PROD_NS), rename = "UUID")
    )]
    pub uuid: Option<String>,

    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub item: Vec<Item>,
}

/// Represents a single item in the build configuration, referencing an object with transform.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Debug, Clone)]
#[cfg_attr(any(feature="write", feature="memory-optimized-read"), xml(ns(CORE_NS, p=PROD_NS), rename = "item"))]
pub struct Item {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub objectid: ResourceId,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub transform: Option<Transform>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub partnumber: Option<String>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(PROD_NS))
    )]
    pub path: Option<String>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, ns(PROD_NS), rename = "UUID")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "UUID"))]
    pub uuid: Option<String>,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::{CORE_NS, PROD_NS, PROD_PREFIX};

    use super::{Build, Item};

    #[test]
    pub fn toxml_item_test() {
        let xml_string = format!(
            r#"<item xmlns="{}" xmlns:{}="{}" objectid="6" partnumber="part_1" />"#,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let item = Item {
            objectid: 6,
            partnumber: Some("part_1".to_string()),
            transform: None,
            path: None,
            uuid: None,
        };
        let item_string = to_string(&item).unwrap();

        assert_eq!(item_string, xml_string);
    }

    #[test]
    pub fn toxml_production_item_test() {
        let xml_string = format!(
            r#"<item xmlns="{}" xmlns:{}="{}" objectid="6" partnumber="part_1" {}:path="//somePath//Item" {}:UUID="someUUID" />"#,
            CORE_NS, PROD_PREFIX, PROD_NS, PROD_PREFIX, PROD_PREFIX
        );
        let item = Item {
            objectid: 6,
            partnumber: Some("part_1".to_string()),
            transform: None,
            path: Some("//somePath//Item".to_owned()),
            uuid: Some("someUUID".to_owned()),
        };
        let item_string = to_string(&item).unwrap();

        assert_eq!(item_string, xml_string);
    }

    #[test]
    pub fn toxml_build_test() {
        let xml_string = format!(
            r#"<build xmlns="{}" xmlns:{}="{}"><item objectid="6" partnumber="part_1" /><item objectid="6" partnumber="part_2" /></build>"#,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let build = Build {
            uuid: None,
            item: vec![
                Item {
                    objectid: 6,
                    partnumber: Some("part_1".to_string()),
                    transform: None,
                    path: None,
                    uuid: None,
                },
                Item {
                    objectid: 6,
                    partnumber: Some("part_2".to_string()),
                    transform: None,
                    path: None,
                    uuid: None,
                },
            ],
        };
        let build_string = to_string(&build).unwrap();

        assert_eq!(build_string, xml_string);
    }

    #[test]
    pub fn toxml_production_build_test() {
        let xml_string = format!(
            r#"<build xmlns="{}" xmlns:{}="{}" {}:UUID="someUUID"><item objectid="6" partnumber="part_1" {}:UUID="someItemUUID1" /><item objectid="6" partnumber="part_2" {}:UUID="someItemUUID2" /></build>"#,
            CORE_NS, PROD_PREFIX, PROD_NS, PROD_PREFIX, PROD_PREFIX, PROD_PREFIX
        );
        let build = Build {
            uuid: Some("someUUID".to_owned()),
            item: vec![
                Item {
                    objectid: 6,
                    partnumber: Some("part_1".to_string()),
                    transform: None,
                    path: None,
                    uuid: Some("someItemUUID1".to_owned()),
                },
                Item {
                    objectid: 6,
                    partnumber: Some("part_2".to_string()),
                    transform: None,
                    path: None,
                    uuid: Some("someItemUUID2".to_owned()),
                },
            ],
        };
        let build_string = to_string(&build).unwrap();

        assert_eq!(build_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        core::transform::Transform,
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Build, Item};

    #[test]
    pub fn fromxml_item_test() {
        let xml_string = format!(
            r#"<item xmlns="{}" objectid="6" partnumber="part_1" transform="1 0 0 0 1 0 0 0 1 35 35 5.1"/>"#,
            CORE_NS
        );
        let item = from_str::<Item>(&xml_string).unwrap();

        assert_eq!(
            item,
            Item {
                objectid: 6,
                partnumber: Some("part_1".to_string()),
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1
                ])),
                path: None,
                uuid: None,
            }
        );
    }

    #[test]
    pub fn fromxml_production_item_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<item xmlns="{}" xmlns:{}="{}" objectid="6" partnumber="part_1" transform="1 0 0 0 1 0 0 0 1 35 35 5.1" {}:path="//somePath//Item" {}:UUID="someUUID"/>"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX, CUSTOM_PROD_PREFIX
        );
        let item = from_str::<Item>(&xml_string).unwrap();

        assert_eq!(
            item,
            Item {
                objectid: 6,
                partnumber: Some("part_1".to_string()),
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1
                ])),
                path: Some("//somePath//Item".to_owned()),
                uuid: Some("someUUID".to_owned()),
            }
        );
    }

    #[test]
    pub fn fromxml_build_test() {
        let xml_string = format!(
            r#"<build xmlns="{}"><item objectid="6" partnumber="part_1" /><item objectid="6" partnumber="part_2" /></build>"#,
            CORE_NS
        );
        let build_string = from_str::<Build>(&xml_string).unwrap();

        assert_eq!(
            build_string,
            Build {
                uuid: None,
                item: vec![
                    Item {
                        objectid: 6,
                        partnumber: Some("part_1".to_string()),
                        transform: None,
                        path: None,
                        uuid: None,
                    },
                    Item {
                        objectid: 6,
                        partnumber: Some("part_2".to_string()),
                        transform: None,
                        path: None,
                        uuid: None,
                    },
                ],
            }
        )
    }

    #[test]
    pub fn fromxml_production_build_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<build xmlns="{}" xmlns:{}="{}" {}:UUID="someBuildUUID"><item objectid="6" partnumber="part_1" {}:UUID="someItemUUID1" /><item objectid="6" partnumber="part_2" {}:UUID="someItemUUID2" /></build>"#,
            CORE_NS,
            CUSTOM_PROD_PREFIX,
            PROD_NS,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX
        );
        let build_string = from_str::<Build>(&xml_string).unwrap();

        assert_eq!(
            build_string,
            Build {
                uuid: Some("someBuildUUID".to_owned()),
                item: vec![
                    Item {
                        objectid: 6,
                        partnumber: Some("part_1".to_string()),
                        transform: None,
                        path: None,
                        uuid: Some("someItemUUID1".to_owned()),
                    },
                    Item {
                        objectid: 6,
                        partnumber: Some("part_2".to_string()),
                        transform: None,
                        path: None,
                        uuid: Some("someItemUUID2".to_owned()),
                    },
                ],
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
        core::transform::Transform,
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Build, Item};

    #[test]
    pub fn fromxml_item_test() {
        let xml_string = format!(
            r#"<item xmlns="{}" objectid="6" partnumber="part_1" transform="1 0 0 0 1 0 0 0 1 35 35 5.1"/>"#,
            CORE_NS
        );
        let item = from_str::<Item>(&xml_string).unwrap();

        assert_eq!(
            item,
            Item {
                objectid: 6,
                partnumber: Some("part_1".to_string()),
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1
                ])),
                path: None,
                uuid: None,
            }
        );
    }

    #[test]
    pub fn fromxml_production_item_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<item xmlns="{}" xmlns:{}="{}" objectid="6" partnumber="part_1" transform="1 0 0 0 1 0 0 0 1 35 35 5.1" {}:path="//somePath//Item" {}:UUID="someUUID"/>"#,
            CORE_NS, CUSTOM_PROD_PREFIX, PROD_NS, CUSTOM_PROD_PREFIX, CUSTOM_PROD_PREFIX
        );
        let item = from_str::<Item>(&xml_string).unwrap();

        assert_eq!(
            item,
            Item {
                objectid: 6,
                partnumber: Some("part_1".to_string()),
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1
                ])),
                path: Some("//somePath//Item".to_owned()),
                uuid: Some("someUUID".to_owned()),
            }
        );
    }

    #[test]
    pub fn fromxml_build_test() {
        let xml_string = format!(
            r#"<build xmlns="{}"><item objectid="6" partnumber="part_1" /><item objectid="6" partnumber="part_2" /></build>"#,
            CORE_NS
        );
        let build_string = from_str::<Build>(&xml_string).unwrap();

        assert_eq!(
            build_string,
            Build {
                uuid: None,
                item: vec![
                    Item {
                        objectid: 6,
                        partnumber: Some("part_1".to_string()),
                        transform: None,
                        path: None,
                        uuid: None,
                    },
                    Item {
                        objectid: 6,
                        partnumber: Some("part_2".to_string()),
                        transform: None,
                        path: None,
                        uuid: None,
                    },
                ],
            }
        )
    }

    #[test]
    pub fn fromxml_production_build_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<build xmlns="{}" xmlns:{}="{}" {}:UUID="someBuildUUID"><item objectid="6" partnumber="part_1" {}:UUID="someItemUUID1" /><item objectid="6" partnumber="part_2" {}:UUID="someItemUUID2" /></build>"#,
            CORE_NS,
            CUSTOM_PROD_PREFIX,
            PROD_NS,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX
        );
        let build_string = from_str::<Build>(&xml_string).unwrap();

        assert_eq!(
            build_string,
            Build {
                uuid: Some("someBuildUUID".to_owned()),
                item: vec![
                    Item {
                        objectid: 6,
                        partnumber: Some("part_1".to_string()),
                        transform: None,
                        path: None,
                        uuid: Some("someItemUUID1".to_owned()),
                    },
                    Item {
                        objectid: 6,
                        partnumber: Some("part_2".to_string()),
                        transform: None,
                        path: None,
                        uuid: Some("someItemUUID2".to_owned()),
                    },
                ],
            }
        )
    }
}

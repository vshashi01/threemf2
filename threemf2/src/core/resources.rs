#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::{
    core::{object::Object, types::ResourceId},
    threemf_namespaces::CORE_NS,
};

/// A collection of Objects and other properties that are referenced by other elements.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Debug, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "resources")
)]
pub struct Resources {
    /// Collection of Object. See [`crate::core::object::Object`]
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub object: Vec<Object>,

    /// Collection of Materials.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub basematerials: Vec<BaseMaterials>,
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "base")
)]
pub struct Base {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub name: String,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub displaycolor: String, //ToDo: Make this a specific color struct for flexibility
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "basematerials")
)]
pub struct BaseMaterials {
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub id: ResourceId,

    pub base: Vec<Base>,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{OptionalResourceId, OptionalResourceIndex, object::Object},
        threemf_namespaces::{BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS, PROD_NS, PROD_PREFIX},
    };

    use super::{Base, BaseMaterials, Resources};

    #[test]
    pub fn toxml_resources_with_object_test() {
        let xml_string = format!(
            r#"<resources xmlns="{}"><object xmlns:{}="{}" xmlns:{}="{}" id="1"></object></resources>"#,
            CORE_NS, BOOLEAN_PREFIX, BOOLEAN_NS, PROD_PREFIX, PROD_NS
        );
        let resources = Resources {
            object: vec![Object {
                id: 1,
                objecttype: None,
                thumbnail: None,
                partnumber: None,
                name: None,
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                uuid: None,
                mesh: None,
                components: None,
                booleanshape: None,
            }],
            basematerials: vec![],
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_resources_with_basematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}"><basematerials id="1"><base name="Base" displaycolor="#FEFEFE00" /></basematerials></resources>"##,
            CORE_NS
        );
        let resources = Resources {
            object: vec![],
            basematerials: vec![BaseMaterials {
                id: 1,
                base: vec![Base {
                    name: "Base".to_owned(),
                    displaycolor: "#FEFEFE00".to_owned(),
                }],
            }],
        };
        let resources_string = to_string(&resources).unwrap();

        assert_eq!(resources_string, xml_string);
    }

    #[test]
    pub fn toxml_base_test() {
        let xml_string = format!(
            r##"<base xmlns="{}" name="Base" displaycolor="#FEF100" />"##,
            CORE_NS
        );
        let base = Base {
            name: "Base".to_string(),
            displaycolor: "#FEF100".to_string(),
        };
        let base_string = to_string(&base).unwrap();

        assert_eq!(base_string, xml_string);
    }

    #[test]
    pub fn toxml_basematerials_test() {
        let xml_string = format!(
            r##"<basematerials xmlns="{}" id="256"><base name="Base 1" displaycolor="#FEF100" /><base name="Base 2" displaycolor="#FEF369" /></basematerials>"##,
            CORE_NS
        );
        let basematerials = BaseMaterials {
            id: 256,
            base: vec![
                Base {
                    name: "Base 1".to_string(),
                    displaycolor: "#FEF100".to_string(),
                },
                Base {
                    name: "Base 2".to_string(),
                    displaycolor: "#FEF369".to_string(),
                },
            ],
        };
        let base_string = to_string(&basematerials).unwrap();

        assert_eq!(base_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{OptionalResourceId, OptionalResourceIndex, object::Object},
        threemf_namespaces::CORE_NS,
    };

    use super::{Base, BaseMaterials, Resources};

    #[test]
    pub fn fromxml_resources_with_object_test() {
        let xml_string = format!(
            r#"<resources xmlns="{}"><object id="1"></object></resources>"#,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: None,
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    mesh: None,
                    components: None,
                    booleanshape: None,
                }],
                basematerials: vec![],
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_basematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}"><basematerials id="1"><base name="Base" displaycolor="#FEFEFE00" /></basematerials></resources>"##,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![BaseMaterials {
                    id: 1,
                    base: vec![Base {
                        name: "Base".to_owned(),
                        displaycolor: "#FEFEFE00".to_owned(),
                    }],
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_base_test() {
        let xml_string = format!(
            r##"<base xmlns="{}" name="Base" displaycolor="#FEF100" />"##,
            CORE_NS
        );
        let base = from_str::<Base>(&xml_string).unwrap();

        assert_eq!(
            base,
            Base {
                name: "Base".to_string(),
                displaycolor: "#FEF100".to_string(),
            }
        );
    }

    #[test]
    pub fn fromxml_basematerials_test() {
        let xml_string = format!(
            r##"<basematerials xmlns="{}" id="256"><base name="Base 1" displaycolor="#FEF100" /><base name="Base 2" displaycolor="#FEF369" /></basematerials>"##,
            CORE_NS
        );
        let base = from_str::<BaseMaterials>(&xml_string).unwrap();

        assert_eq!(
            base,
            BaseMaterials {
                id: 256,
                base: vec![
                    Base {
                        name: "Base 1".to_string(),
                        displaycolor: "#FEF100".to_string(),
                    },
                    Base {
                        name: "Base 2".to_string(),
                        displaycolor: "#FEF369".to_string(),
                    },
                ],
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::{
        core::{OptionalResourceId, OptionalResourceIndex, object::Object},
        threemf_namespaces::CORE_NS,
    };

    use super::{Base, BaseMaterials, Resources};

    #[test]
    pub fn fromxml_resources_with_object_test() {
        let xml_string = format!(
            r#"<resources xmlns="{}"><object id="1"></object></resources>"#,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: None,
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    mesh: None,
                    components: None,
                    booleanshape: None,
                }],
                basematerials: vec![],
            }
        );
    }

    #[test]
    pub fn fromxml_resources_with_basematerials_test() {
        let xml_string = format!(
            r##"<resources xmlns="{}"><basematerials id="1"><base name="Base" displaycolor="#FEFEFE00" /></basematerials></resources>"##,
            CORE_NS
        );
        let resources = from_str::<Resources>(&xml_string).unwrap();

        assert_eq!(
            resources,
            Resources {
                object: vec![],
                basematerials: vec![BaseMaterials {
                    id: 1,
                    base: vec![Base {
                        name: "Base".to_owned(),
                        displaycolor: "#FEFEFE00".to_owned(),
                    }],
                }],
            }
        );
    }

    #[test]
    pub fn fromxml_base_test() {
        let xml_string = format!(
            r##"<base xmlns="{}" name="Base" displaycolor="#FEF100" />"##,
            CORE_NS
        );
        let base = from_str::<Base>(&xml_string).unwrap();

        assert_eq!(
            base,
            Base {
                name: "Base".to_string(),
                displaycolor: "#FEF100".to_string(),
            }
        );
    }

    #[test]
    pub fn fromxml_basematerials_test() {
        let xml_string = format!(
            r##"<basematerials xmlns="{}" id="256"><base name="Base 1" displaycolor="#FEF100" /><base name="Base 2" displaycolor="#FEF369" /></basematerials>"##,
            CORE_NS
        );
        let base = from_str::<BaseMaterials>(&xml_string).unwrap();

        assert_eq!(
            base,
            BaseMaterials {
                id: 256,
                base: vec![
                    Base {
                        name: "Base 1".to_string(),
                        displaycolor: "#FEF100".to_string(),
                    },
                    Base {
                        name: "Base 2".to_string(),
                        displaycolor: "#FEF369".to_string(),
                    },
                ],
            }
        );
    }
}

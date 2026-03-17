#[cfg(any(feature = "write", feature = "memory-optimized-read"))]
use instant_xml::Error;

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::{FromXml, Kind};

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::threemf_namespaces::CORE_NS;

//ToDo: Add additional optional fields on Metadata
/// Key-value metadata associated with a 3MF model or object.
///
/// Metadata provides additional information about the model, such as author,
/// description, or custom properties.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
// #[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone, Eq)]
#[cfg_attr(
    any(feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "metadata")
)]
pub struct Metadata {
    #[cfg_attr(any(feature = "memory-optimized-read"), xml(attribute))]
    pub name: String,

    #[cfg_attr(any(feature = "memory-optimized-read"), xml(attribute))]
    pub preserve: Option<Preserve>,

    #[cfg_attr(any(feature = "memory-optimized-read"), xml(direct))]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "#content"))]
    pub value: Option<String>,
}

#[cfg(feature = "write")]
impl ToXml for Metadata {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), Error> {
        // let id = field.unwrap_or(instant_xml::Id {
        //     name: "metadata",
        //     ns: CORE_NS,
        // });

        let (name, ns) = match field {
            Some(field) => (field.name, field.ns),
            None => ("metadata", CORE_NS),
        };

        let _ = serializer.write_start(name, ns)?;

        if field.is_none() {
            let _ = serializer.push(instant_xml::ser::Context {
                default_ns: CORE_NS,
                prefixes: [],
            });
        }

        serializer.write_attr("name", ns, &self.name)?;
        if let Some(preserve) = &self.preserve {
            serializer.write_attr("preserve", ns, &preserve.0)?;
        }

        // Write value as text content if present
        if let Some(value) = &self.value {
            serializer.end_start()?;
            serializer.write_str(value)?;
            serializer.write_close(None, name)?;
        } else {
            serializer.end_empty()?;
        }

        Ok(())
    }
}

/// Group of metadata entries.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(CORE_NS), rename = "metadatagroup")
)]
pub struct MetadataGroup {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub metadata: Vec<Metadata>,
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "write", xml(ns(CORE_NS), rename = "preserve"))]
pub struct Preserve(pub bool);

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Preserve {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let value = match deserializer.take_str()? {
            Some(value) => value,
            None => return Err(Error::MissingValue("No Must Preserve value found")),
        };

        if let Ok(must_preserve) = value.parse::<bool>() {
            *into = Some(Preserve(must_preserve));
            Ok(())
        } else {
            Err(Error::MissingValue("Not a valid boolean value"))
        }
    }

    type Accumulator = Option<Self>;

    const KIND: Kind = Kind::Scalar;
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{core::metadata::Preserve, threemf_namespaces::CORE_NS};

    use super::{Metadata, MetadataGroup};

    #[test]
    pub fn toxml_metadata_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="Copyright">Copyright (c) 2018 3MF Consortium. All rights reserved.</metadata>"#,
            CORE_NS
        );
        let metadata = Metadata {
            name: "Copyright".to_string(),
            preserve: None,
            value: Some("Copyright (c) 2018 3MF Consortium. All rights reserved.".to_string()),
        };
        let metadata_string = to_string(&metadata).unwrap();

        assert_eq!(metadata_string, xml_string);
    }

    #[test]
    pub fn toxml_simple_metadata_test() {
        let xml_string = format!(r#"<metadata xmlns="{}" name="From Test" />"#, CORE_NS);
        let metadata = Metadata {
            name: "From Test".to_string(),
            preserve: None,
            value: None,
        };
        let metadata_string = to_string(&metadata).unwrap();

        assert_eq!(metadata_string, xml_string);
    }

    #[test]
    pub fn toxml_advanced_metadata_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="From Test" preserve="true">This is a metadata</metadata>"#,
            CORE_NS
        );
        let metadata = Metadata {
            name: "From Test".to_string(),
            preserve: Some(Preserve(true)),
            value: Some("This is a metadata".to_string()),
        };
        let metadata_string = to_string(&metadata).unwrap();

        assert_eq!(metadata_string, xml_string);
    }

    #[test]
    pub fn toxml_metadatagroup_test() {
        let xml_string = format!(
            r#"<metadatagroup xmlns="{}"><metadata name="From Test"></metadata><metadata name="From Test 2"></metadata></metadatagroup>"#,
            CORE_NS
        );
        let metadatagroup = MetadataGroup {
            metadata: vec![
                Metadata {
                    name: "From Test".to_string(),
                    preserve: None,
                    value: Some("".to_string()),
                },
                Metadata {
                    name: "From Test 2".to_string(),
                    preserve: None,
                    value: Some("".to_string()),
                },
            ],
        };
        let metadatagroup_string = to_string(&metadatagroup).unwrap();

        assert_eq!(metadatagroup_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::CORE_NS;

    use super::{Metadata, MetadataGroup};

    #[test]
    pub fn fromxml_metadata_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="Copyright">Copyright (c) 2018 3MF Consortium. All rights reserved.</metadata>"#,
            CORE_NS
        );
        let metadata = from_str::<Metadata>(&xml_string).unwrap();

        assert_eq!(
            metadata,
            Metadata {
                name: "Copyright".to_string(),
                preserve: None,
                value: Some("Copyright (c) 2018 3MF Consortium. All rights reserved.".to_string())
            }
        )
    }

    #[test]
    pub fn fromxml_simple_metadata_test() {
        let xml_string = format!(r#"<metadata xmlns="{}" name="From Test"/>"#, CORE_NS);
        let metadata = from_str::<Metadata>(&xml_string).unwrap();

        assert_eq!(
            metadata,
            Metadata {
                name: "From Test".to_string(),
                preserve: None,
                value: None,
            }
        )
    }

    #[test]
    pub fn fromxml_metadatagroup_test() {
        let xml_string = format!(
            r#"<metadatagroup xmlns="{}"><metadata name="From Test"></metadata><metadata name="From Test 2"></metadata></metadatagroup>"#,
            CORE_NS
        );
        let metadatagroup = from_str::<MetadataGroup>(&xml_string).unwrap();

        assert_eq!(
            metadatagroup,
            MetadataGroup {
                metadata: vec![
                    Metadata {
                        name: "From Test".to_string(),
                        preserve: None,
                        value: None,
                    },
                    Metadata {
                        name: "From Test 2".to_string(),
                        preserve: None,
                        value: None,
                    }
                ]
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

    use super::{Metadata, MetadataGroup};

    #[test]
    pub fn fromxml_metadata_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="Copyright">Copyright (c) 2018 3MF Consortium. All rights reserved.</metadata>"#,
            CORE_NS
        );
        let metadata = from_str::<Metadata>(&xml_string).unwrap();

        assert_eq!(
            metadata,
            Metadata {
                name: "Copyright".to_string(),
                preserve: None,
                value: Some("Copyright (c) 2018 3MF Consortium. All rights reserved.".to_string())
            }
        )
    }

    #[test]
    pub fn fromxml_simple_metadata_test() {
        let xml_string = format!(r#"<metadata xmlns="{}" name="From Test"/>"#, CORE_NS);
        let metadata = from_str::<Metadata>(&xml_string).unwrap();

        assert_eq!(
            metadata,
            Metadata {
                name: "From Test".to_string(),
                preserve: None,
                value: Some("".to_owned()),
            }
        )
    }

    #[test]
    pub fn fromxml_metadatagroup_test() {
        let xml_string = format!(
            r#"<metadatagroup xmlns="{}"><metadata name="From Test"></metadata><metadata name="From Test 2"></metadata></metadatagroup>"#,
            CORE_NS
        );
        let metadatagroup = from_str::<MetadataGroup>(&xml_string).unwrap();

        assert_eq!(
            metadatagroup,
            MetadataGroup {
                metadata: vec![
                    Metadata {
                        name: "From Test".to_string(),
                        preserve: None,
                        value: Some("".to_owned()),
                    },
                    Metadata {
                        name: "From Test 2".to_string(),
                        preserve: None,
                        value: Some("".to_owned()),
                    }
                ]
            }
        )
    }
}

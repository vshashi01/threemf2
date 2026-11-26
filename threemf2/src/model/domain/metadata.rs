#[cfg(any(feature = "write", feature = "read"))]
use instant_xml::Error;

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::{FromXml, Kind};

use crate::{
    model::{StrResource, domain::any_extensions::AnyAttributeExtensions},
    threemf_namespaces::CORE_NS,
};

#[cfg(feature = "read")]
use crate::model::domain::xml_helpers::read_attribute;

/// Key-value metadata associated with a 3MF model or object.
///
/// Metadata provides additional information about the model, such as author,
/// description, or custom properties.
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Metadata {
    /// Name of the metadata entry.
    pub name: StrResource,

    /// Whether this metadata must be preserved by consumers.
    pub preserve: Option<Preserve>,

    /// Value of the metadata entry.
    pub value: Option<StrResource>,

    /// Foreign attributes of the metadata entry (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Metadata {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: CORE_NS,
            name: "metadata",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, de::Node};

        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let mut name = <StrResource as FromXml>::Accumulator::default();
        let mut preserve = <Option<Preserve> as FromXml>::Accumulator::default();
        let mut value: Option<String> = None;
        #[allow(unused_mut)]
        let mut any_extensions = AnyAttributeExtensions::default();

        while let Some(node) = deserializer.next() {
            match node? {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    match (id.ns, id.name) {
                        ("", "name") => read_attribute::<StrResource>(
                            &mut name,
                            "name",
                            deserializer,
                            attr.value,
                        )?,
                        ("", "preserve") => read_attribute::<Option<Preserve>>(
                            &mut preserve,
                            "preserve",
                            deserializer,
                            attr.value,
                        )?,
                        #[cfg(feature = "any")]
                        _ => crate::model::domain::any::read::capture_attribute(
                            any_extensions.attributes_mut(),
                            id,
                            attr.prefix,
                            &attr.value,
                        ),
                        #[cfg(not(feature = "any"))]
                        _ => {}
                    }
                }
                Node::Text(text) => value.get_or_insert_with(String::new).push_str(&text),
                Node::Open(element) => deserializer.nested(element).ignore()?,
                _ => {}
            }
        }

        *into = Some(Metadata {
            name: name.try_done("name")?,
            preserve: preserve.try_done("preserve")?,
            value: value.map(StrResource::from),
            any_extensions,
        });
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Element;
}

#[cfg(feature = "write")]
impl ToXml for Metadata {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let (name, ns) = match field {
            Some(field) => (field.name, field.ns),
            None => ("metadata", CORE_NS),
        };

        let mut cx = instant_xml::ser::Context::<0>::default();
        cx.default_ns = CORE_NS;

        let element = serializer.write_start(name, ns, Some(cx))?;

        serializer.write_attr("name", CORE_NS, &self.name)?;
        if let Some(preserve) = &self.preserve {
            serializer.write_attr("preserve", CORE_NS, &preserve.0)?;
        }
        #[cfg(feature = "any")]
        crate::model::domain::any::write::write_foreign_attributes(
            serializer,
            &[],
            self.any_extensions.attributes(),
            &[],
        )?;

        if let Some(value) = &self.value {
            serializer.end_start()?;
            serializer.write_str(value)?;
            serializer.write_close(element)?;
        } else {
            serializer.end_empty()?;
        }

        Ok(())
    }
}

/// Group of metadata entries attached to an object or a build item (`<metadatagroup>`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MetadataGroup {
    /// Metadata entries in this group.
    pub metadata: Vec<Metadata>,

    /// Foreign attributes of the group (`xs:anyAttribute`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyAttributeExtensions,
}

impl MetadataGroup {
    /// Creates a group from metadata entries, or `None` if there are no entries.
    pub fn from_entries(metadata: Vec<Metadata>) -> Option<Self> {
        (!metadata.is_empty()).then(|| Self {
            metadata,
            any_extensions: Default::default(),
        })
    }
}

#[cfg(any(feature = "read", feature = "write"))]
crate::model::domain::xml_struct::xml_struct! {
    MetadataGroup {
        tag: "metadatagroup",
        ns: CORE_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {},
        elements: { metadata: Vec<Metadata> => (CORE_NS, "metadata") },
        any: any_extensions: AnyAttributeExtensions,
    }
}

/// Whether metadata must be preserved when processing the model.
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "write", xml(ns(CORE_NS), rename = "preserve"))]
pub struct Preserve(pub bool);

#[cfg(feature = "read")]
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

    use crate::{model::domain::metadata::Preserve, threemf_namespaces::CORE_NS};

    use super::{Metadata, MetadataGroup};

    #[test]
    pub fn toxml_metadata_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="Copyright">Copyright (c) 2018 3MF Consortium. All rights reserved.</metadata>"#,
            CORE_NS
        );
        let metadata = Metadata {
            name: "Copyright".into(),
            preserve: None,
            value: Some("Copyright (c) 2018 3MF Consortium. All rights reserved.".into()),
            any_extensions: Default::default(),
        };
        let metadata_string = to_string(&metadata).unwrap();

        assert_eq!(metadata_string, xml_string);
    }

    #[test]
    pub fn toxml_simple_metadata_test() {
        let xml_string = format!(r#"<metadata xmlns="{}" name="From Test" />"#, CORE_NS);
        let metadata = Metadata {
            name: "From Test".into(),
            preserve: None,
            value: None,
            any_extensions: Default::default(),
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
            name: "From Test".into(),
            preserve: Some(Preserve(true)),
            value: Some("This is a metadata".into()),
            any_extensions: Default::default(),
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
                    name: "From Test".into(),
                    preserve: None,
                    value: Some("".into()),
                    any_extensions: Default::default(),
                },
                Metadata {
                    name: "From Test 2".into(),
                    preserve: None,
                    value: Some("".into()),
                    any_extensions: Default::default(),
                },
            ],
            any_extensions: Default::default(),
        };
        let metadatagroup_string = to_string(&metadatagroup).unwrap();

        assert_eq!(metadatagroup_string, xml_string);
    }
}

#[cfg(feature = "read")]
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
                name: "Copyright".into(),
                preserve: None,
                value: Some("Copyright (c) 2018 3MF Consortium. All rights reserved.".into()),
                any_extensions: Default::default(),
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
                name: "From Test".into(),
                preserve: None,
                value: None,
                any_extensions: Default::default(),
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
                        name: "From Test".into(),
                        preserve: None,
                        value: None,
                        any_extensions: Default::default(),
                    },
                    Metadata {
                        name: "From Test 2".into(),
                        preserve: None,
                        value: None,
                        any_extensions: Default::default(),
                    }
                ],
                any_extensions: Default::default(),
            }
        )
    }

    #[test]
    pub fn fromxml_metadata_with_preserve_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="Test" preserve="true">value</metadata>"#,
            CORE_NS
        );
        let metadata = from_str::<Metadata>(&xml_string).unwrap();

        assert_eq!(metadata.name.as_ref(), "Test");
        assert_eq!(metadata.value.as_deref(), Some("value"));
        assert!(metadata.preserve.is_some());
        assert_eq!(metadata.preserve.as_ref().unwrap().0, true);
    }

    #[test]
    pub fn fromxml_metadata_preserve_false_test() {
        let xml_string = format!(
            r#"<metadata xmlns="{}" name="Test2" preserve="false"></metadata>"#,
            CORE_NS
        );
        let metadata = from_str::<Metadata>(&xml_string).unwrap();

        assert_eq!(metadata.name.as_ref(), "Test2");
        assert!(metadata.preserve.is_some());
        assert_eq!(metadata.preserve.as_ref().unwrap().0, false);
    }
}

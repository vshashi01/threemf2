#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

use crate::model::domain::any_extensions::AnyExtensions;
#[cfg(feature = "read")]
use crate::model::domain::xml_helpers::read_attribute;
#[cfg(feature = "write")]
use crate::threemf_namespaces::PROD_PREFIX;
#[cfg(any(feature = "read", feature = "write"))]
use crate::threemf_namespaces::{CORE_NS, PROD_NS};
use crate::{
    model::domain::transform::Transform,
    model::{
        PathResource, StrResource, {ResourceId, UuidResource},
    },
};

#[cfg(all(feature = "any", feature = "write"))]
use crate::model::domain::any::write as any_write;
#[cfg(all(feature = "any", feature = "read"))]
use crate::model::domain::any::{is_foreign_namespace, read as any_read};

/// Contains the build configuration for a 3MF model, listing items to be printed.
///
/// The build section specifies which objects from the resources should be included
/// in the final 3D print, along with their transforms and metadata.
///
/// # Root vs Sub-Models
///
/// - **Root models** MUST have a `Build` section with at least one item.
/// - **Sub-models** CANNOT have a `Build` section.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, PartialEq, Debug, Clone)]
#[cfg_attr(any(feature="write", feature="read"), xml(ns(CORE_NS, p=PROD_NS), rename = "build"))]
pub struct Build {
    /// Uuid specified to this Build
    #[cfg_attr(
        any(feature = "write", feature = "read"),
        xml(attribute, ns(PROD_NS), rename = "UUID")
    )]
    pub uuid: Option<UuidResource>,

    /// Field of items
    pub item: Vec<Item>,
}

/// Represents a single item in the build configuration, referencing an object with transform.
///
/// `FromXml` and `ToXml` are implemented by hand so that foreign attributes and elements
/// (`xs:anyAttribute` / trailing `xs:any` in the core schema) can be kept with the `any` feature.
#[derive(Default, PartialEq, Debug, Clone)]
pub struct Item {
    /// Id of the Object referenced by this Item
    pub objectid: ResourceId,

    /// Transform associated to this Item
    pub transform: Option<Transform>,

    /// Part number associated with this Item
    pub partnumber: Option<StrResource>,

    /// Optional Path to the model where the Object referenced by this Item is located in
    pub path: Option<PathResource>,

    /// Uuuid assigned to this Item
    pub uuid: Option<UuidResource>,

    /// Foreign attributes and trailing child elements of this Item (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(feature = "read")]
impl<'xml> FromXml<'xml> for Item {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        id == instant_xml::Id {
            ns: CORE_NS,
            name: "item",
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        use instant_xml::{Accumulate, de::Node};

        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }

        let mut objectid = <ResourceId as FromXml>::Accumulator::default();
        let mut transform = <Option<Transform> as FromXml>::Accumulator::default();
        let mut partnumber = <Option<StrResource> as FromXml>::Accumulator::default();
        let mut path = <Option<PathResource> as FromXml>::Accumulator::default();
        let mut uuid = <Option<UuidResource> as FromXml>::Accumulator::default();
        #[cfg_attr(not(feature = "any"), allow(unused_mut))]
        let mut any_extensions = AnyExtensions::default();

        while let Some(node) = deserializer.next() {
            match node? {
                Node::Attribute(attr) => {
                    let id = deserializer.attribute_id(&attr)?;
                    match (id.ns, id.name) {
                        ("", "objectid") => read_attribute::<ResourceId>(
                            &mut objectid,
                            "objectid",
                            deserializer,
                            attr.value,
                        )?,
                        ("", "transform") => read_attribute::<Option<Transform>>(
                            &mut transform,
                            "transform",
                            deserializer,
                            attr.value,
                        )?,
                        ("", "partnumber") => read_attribute::<Option<StrResource>>(
                            &mut partnumber,
                            "partnumber",
                            deserializer,
                            attr.value,
                        )?,
                        (PROD_NS, "path") => read_attribute::<Option<PathResource>>(
                            &mut path,
                            "path",
                            deserializer,
                            attr.value,
                        )?,
                        (PROD_NS, "UUID") => read_attribute::<Option<UuidResource>>(
                            &mut uuid,
                            "uuid",
                            deserializer,
                            attr.value,
                        )?,
                        #[cfg(feature = "any")]
                        _ => any_read::capture_attribute(
                            &mut any_extensions.attributes,
                            id,
                            attr.prefix,
                            &attr.value,
                        ),
                        #[cfg(not(feature = "any"))]
                        _ => {}
                    }
                }
                Node::Open(element) => {
                    #[cfg(feature = "any")]
                    let id = deserializer.element_id(&element)?;
                    let mut nested = deserializer.nested(element);

                    #[cfg(feature = "any")]
                    if is_foreign_namespace(id.ns) {
                        any_extensions
                            .elements
                            .push(any_read::read_element(&mut nested, id)?);
                        continue;
                    }

                    nested.ignore()?;
                }
                _ => {}
            }
        }

        *into = Some(Item {
            objectid: objectid.try_done("objectid")?,
            transform: transform.try_done("transform")?,
            partnumber: partnumber.try_done("partnumber")?,
            path: path.try_done("path")?,
            uuid: uuid.try_done("uuid")?,
            any_extensions,
        });

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

#[cfg(feature = "write")]
impl ToXml for Item {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let mut context = instant_xml::ser::Context::<1>::default();
        context.default_ns = CORE_NS;
        context.prefixes[0] = instant_xml::ser::Prefix {
            ns: PROD_NS,
            prefix: PROD_PREFIX,
        };
        let element = serializer.write_start("item", CORE_NS, Some(context))?;

        serializer.write_attr("objectid", "", &self.objectid)?;
        if self.transform.present() {
            serializer.write_attr("transform", "", &self.transform)?;
        }
        if self.partnumber.present() {
            serializer.write_attr("partnumber", "", &self.partnumber)?;
        }
        if self.path.present() {
            serializer.write_attr("path", PROD_NS, &self.path)?;
        }
        if self.uuid.present() {
            serializer.write_attr("UUID", PROD_NS, &self.uuid)?;
        }

        #[cfg(feature = "any")]
        {
            let scope = any_write::write_foreign_attributes(
                serializer,
                self.any_extensions.attributes(),
                self.any_extensions.elements(),
            )?;
            if !self.any_extensions.elements().is_empty() {
                serializer.end_start()?;
                any_write::write_foreign_elements(
                    serializer,
                    &scope,
                    self.any_extensions.elements(),
                )?;
                return serializer.write_close(element);
            }
        }

        let _ = element;
        serializer.end_empty()
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::{
        model::{PathResource, UuidResource},
        threemf_namespaces::{CORE_NS, PROD_NS, PROD_PREFIX},
    };

    use super::{Build, Item};

    #[test]
    pub fn toxml_item_test() {
        let xml_string = format!(
            r#"<item xmlns="{}" xmlns:{}="{}" objectid="6" partnumber="part_1" />"#,
            CORE_NS, PROD_PREFIX, PROD_NS
        );
        let item = Item {
            objectid: 6,
            partnumber: Some("part_1".into()),
            transform: None,
            path: None,
            uuid: None,
            ..Default::default()
        };
        let item_string = to_string(&item).unwrap();

        assert_eq!(item_string, xml_string);
    }

    #[test]
    pub fn toxml_production_item_test() {
        let xml_string = format!(
            r#"<item xmlns="{}" xmlns:{}="{}" objectid="6" partnumber="part_1" {}:path="/somePath/Item" {}:UUID="someUUID" />"#,
            CORE_NS, PROD_PREFIX, PROD_NS, PROD_PREFIX, PROD_PREFIX
        );
        let item = Item {
            objectid: 6,
            partnumber: Some("part_1".into()),
            transform: None,
            path: Some(PathResource::try_from("//somePath//Item").unwrap()),
            uuid: Some(UuidResource::from("someUUID")),
            ..Default::default()
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
                    partnumber: Some("part_1".into()),
                    transform: None,
                    path: None,
                    uuid: None,
                    ..Default::default()
                },
                Item {
                    objectid: 6,
                    partnumber: Some("part_2".into()),
                    transform: None,
                    path: None,
                    uuid: None,
                    ..Default::default()
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
            uuid: Some(UuidResource::from("someUUID")),
            item: vec![
                Item {
                    objectid: 6,
                    partnumber: Some("part_1".into()),
                    transform: None,
                    path: None,
                    uuid: Some(UuidResource::from("someItemUUID1")),
                    ..Default::default()
                },
                Item {
                    objectid: 6,
                    partnumber: Some("part_2".into()),
                    transform: None,
                    path: None,
                    uuid: Some(UuidResource::from("someItemUUID2")),
                    ..Default::default()
                },
            ],
        };
        let build_string = to_string(&build).unwrap();

        assert_eq!(build_string, xml_string);
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
                partnumber: Some("part_1".into()),
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1
                ])),
                path: None,
                uuid: None,
                ..Default::default()
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
                partnumber: Some("part_1".into()),
                transform: Some(Transform([
                    1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 35.0, 35.0, 5.1
                ])),
                path: Some(PathResource::try_from("//somePath//Item").unwrap()),
                uuid: Some(UuidResource::from("someUUID")),
                ..Default::default()
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
                        partnumber: Some("part_1".into()),
                        transform: None,
                        path: None,
                        uuid: None,
                        ..Default::default()
                    },
                    Item {
                        objectid: 6,
                        partnumber: Some("part_2".into()),
                        transform: None,
                        path: None,
                        uuid: None,
                        ..Default::default()
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
                uuid: Some(UuidResource::from("someBuildUUID")),
                item: vec![
                    Item {
                        objectid: 6,
                        partnumber: Some("part_1".into()),
                        transform: None,
                        path: None,
                        uuid: Some(UuidResource::from("someItemUUID1")),
                        ..Default::default()
                    },
                    Item {
                        objectid: 6,
                        partnumber: Some("part_2".into()),
                        transform: None,
                        path: None,
                        uuid: Some(UuidResource::from("someItemUUID2")),
                        ..Default::default()
                    },
                ],
            }
        )
    }
}

#[cfg(all(feature = "any", feature = "read", feature = "write"))]
#[cfg(test)]
mod any_tests {
    use instant_xml::{from_str, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        model::domain::{
            any::{AnyAttributes, AnyElement, AnyNode},
            any_extensions::AnyExtensions,
        },
        threemf_namespaces::{CORE_NS, PROD_NS, PROD_PREFIX},
    };

    use super::{Build, Item};

    const ACME_NS: &str = "http://acme.example/3mf/2026";

    #[test]
    fn fromxml_item_keeps_foreign_content() {
        let xml = format!(
            r#"<item xmlns="{CORE_NS}" xmlns:acme="{ACME_NS}" objectid="1" acme:batch="42" unknown="x"><acme:info rev="3"><acme:note>hello &amp; bye</acme:note></acme:info><ignored /></item>"#
        );
        let item = from_str::<Item>(&xml).unwrap();

        assert_eq!(item.objectid, 1);
        assert_eq!(
            item.any_extensions.attributes().get(ACME_NS, "batch"),
            Some("42")
        );
        assert_eq!(item.any_extensions.attributes().len(), 1);
        assert_eq!(
            item.any_extensions
                .attributes()
                .get_attribute(ACME_NS, "batch")
                .unwrap()
                .prefix_hint(),
            Some("acme")
        );

        assert_eq!(item.any_extensions.elements().len(), 1);
        let info = &item.any_extensions.elements()[0];
        assert!(info.is(ACME_NS, "info"));
        assert_eq!(info.attribute("", "rev"), Some("3"));
        assert_eq!(info.child(ACME_NS, "note").unwrap().text(), "hello & bye");
    }

    #[test]
    fn toxml_item_writes_foreign_content() {
        let mut attributes = AnyAttributes::new();
        attributes.set(ACME_NS, "batch", "42").unwrap();

        let info = AnyElement::new(ACME_NS, "info")
            .unwrap()
            .with_prefix_hint("acme")
            .with_attribute("", "rev", "3")
            .unwrap()
            .with_child(AnyElement::new(ACME_NS, "note").unwrap().with_text("a < b"));

        let item = Item {
            objectid: 1,
            uuid: Some("id".into()),
            any_extensions: AnyExtensions::new()
                .with_attributes(attributes)
                .with_element(info),
            ..Default::default()
        };

        let expected = format!(
            r#"<item xmlns="{CORE_NS}" xmlns:{PROD_PREFIX}="{PROD_NS}" objectid="1" {PROD_PREFIX}:UUID="id" xmlns:acme="{ACME_NS}" acme:batch="42"><acme:info rev="3"><acme:note>a &lt; b</acme:note></acme:info></item>"#
        );
        assert_eq!(to_string(&item).unwrap(), expected);
    }

    #[test]
    fn item_foreign_content_roundtrips() {
        let xml = format!(
            r#"<build xmlns="{CORE_NS}"><item xmlns:x="{ACME_NS}" xmlns:p2="{PROD_NS}" objectid="1" x:a="1" p2:UUID="u"><x:e xmlns:y="urn:other" y:b="2"><plain xmlns="">text</plain><y:z /></x:e></item></build>"#
        );
        let build = from_str::<Build>(&xml).unwrap();
        let written = to_string(&build).unwrap();
        let reread = from_str::<Build>(&written).unwrap();

        assert_eq!(build, reread);
        assert_eq!(build.item[0].uuid, Some("u".into()));

        let e = &build.item[0].any_extensions.elements()[0];
        assert_eq!(e.attribute("urn:other", "b"), Some("2"));
        let AnyNode::Element(plain) = &e.children()[0] else {
            panic!("expected element");
        };
        assert!(plain.is("", "plain"));
        assert_eq!(plain.text(), "text");
    }

    #[test]
    fn toxml_rejects_non_foreign_elements() {
        let item = Item {
            objectid: 1,
            any_extensions: AnyExtensions::new()
                .with_element(AnyElement::new(CORE_NS, "metadatagroup").unwrap()),
            ..Default::default()
        };
        assert!(to_string(&item).is_err());
    }

    #[test]
    fn any_attributes_reject_non_foreign_namespaces() {
        let mut attributes = AnyAttributes::new();
        assert!(attributes.set("", "a", "1").is_err());
        assert!(attributes.set(CORE_NS, "a", "1").is_err());
        assert!(attributes.set(ACME_NS, "not:valid", "1").is_err());
    }
}

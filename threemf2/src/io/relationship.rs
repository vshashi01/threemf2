#[cfg(any(feature = "write", feature = "memory-optimized-read"))]
use instant_xml::Error;

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::{FromXml, Kind};

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use crate::model::{PathResource, StrResource};

const RELATIONSHIP_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

/// Represents a relationship of a single part in the 3mf package along with its [RelationshipType]
/// and target path of the part in the archive.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(RELATIONSHIP_NS))
)]
pub struct Relationship {
    /// The unique identifier of the relationship.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, rename = "Id")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "Id"))]
    pub id: StrResource,

    /// Target path of the part in the archive.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, rename = "Target")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "Target"))]
    pub target: PathResource,

    /// The actual relationship of the target part
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute, rename = "Type")
    )]
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "Type"))]
    pub relationship_type: RelationshipType,
}

/// Represents a collection of [Relationship]s where each collection is an independent
/// relationship part in the 3mf package. A single 3mf package may contain multiple [Relationships].
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(RELATIONSHIP_NS))
)]
pub struct Relationships {
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "Relationship"))]
    pub relationships: Vec<Relationship>,
}

/// Represents the type of relationship of a part in the 3mf package.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipType {
    /// Represents a thumbnail part in the package.
    Thumbnail,

    /// Represents a model part in the package.
    Model,

    /// Represents an unknown part currently by this library
    /// The namespaces of the relationship type is stored in the tuple.
    Unknown(StrResource),
}

const THUMBNAIL_TYPE_NS: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";
const MODEL_TYPE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";

#[cfg(feature = "write")]
impl ToXml for RelationshipType {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), Error> {
        let ns_str = match self {
            Self::Thumbnail => THUMBNAIL_TYPE_NS,
            Self::Model => MODEL_TYPE_NS,
            Self::Unknown(value) => value,
        };

        serializer.write_str(ns_str)?;
        Ok(())
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for RelationshipType {
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
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let value = match deserializer.take_str()? {
            Some(value) => value,
            None => return Err(Error::MissingValue("No RelationshipType string found")),
        };

        match value.into_owned().as_ref() {
            THUMBNAIL_TYPE_NS => *into = Some(Self::Thumbnail),
            MODEL_TYPE_NS => *into = Some(Self::Model),
            value => *into = Some(Self::Unknown(value.into())),
        }

        Ok(())
    }

    type Accumulator = Option<Self>;

    const KIND: Kind = Kind::Scalar;
}

impl From<String> for RelationshipType {
    fn from(value: String) -> Self {
        match value.as_ref() {
            THUMBNAIL_TYPE_NS => Self::Thumbnail,
            MODEL_TYPE_NS => Self::Model,
            value => Self::Unknown(value.into()),
        }
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::model::PathResource;

    use super::{
        MODEL_TYPE_NS, RELATIONSHIP_NS, Relationship, RelationshipType, Relationships,
        THUMBNAIL_TYPE_NS,
    };

    #[test]
    pub fn toxml_relationships_test() {
        let xml_string = format!(
            r#"<Relationships xmlns="{}"><Relationship Id="someId" Target="/somePath/Of/Resources" Type="{}" /><Relationship Id="someId1" Target="/somePath/Of/Resources" Type="{}" /><Relationship Id="someId2" Target="/somePath/Of/Unknown" Type="unknown" /></Relationships>"#,
            RELATIONSHIP_NS, MODEL_TYPE_NS, THUMBNAIL_TYPE_NS
        );
        let relationships = Relationships {
            relationships: vec![
                Relationship {
                    id: "someId".into(),
                    target: PathResource::new("/somePath/Of/Resources", true).unwrap(),
                    relationship_type: RelationshipType::Model,
                },
                Relationship {
                    id: "someId1".into(),
                    target: PathResource::new("/somePath/Of/Resources", true).unwrap(),
                    relationship_type: RelationshipType::Thumbnail,
                },
                Relationship {
                    id: "someId2".into(),
                    target: PathResource::new("//somePath//Of/Unknown", false).unwrap(),
                    relationship_type: RelationshipType::Unknown("unknown".into()),
                },
            ],
        };
        let relationships_string = to_string(&relationships).unwrap();

        assert_eq!(relationships_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::model::PathResource;

    use super::{
        MODEL_TYPE_NS, RELATIONSHIP_NS, Relationship, RelationshipType, Relationships,
        THUMBNAIL_TYPE_NS,
    };

    #[test]
    pub fn fromxml_relationships_test() {
        let xml_string = format!(
            r#"<Relationships xmlns="{}"><Relationship Id="someId" Target="//somePath//Of//Resources" Type="{}" /><Relationship Id="someId1" Target="//somePath//Of//Resources" Type="{}" /><Relationship Id="someId2" Target="//somePath//Of//Unknown" Type="unknown" /></Relationships>"#,
            RELATIONSHIP_NS, MODEL_TYPE_NS, THUMBNAIL_TYPE_NS
        );
        let relationships = from_str::<Relationships>(&xml_string).unwrap();

        assert_eq!(
            relationships,
            Relationships {
                relationships: vec![
                    Relationship {
                        id: "someId".into(),
                        target: PathResource::new("/somePath/Of/Resources", true).unwrap(),
                        relationship_type: RelationshipType::Model,
                    },
                    Relationship {
                        id: "someId1".into(),
                        target: PathResource::new("/somePath/Of/Resources", true).unwrap(),
                        relationship_type: RelationshipType::Thumbnail,
                    },
                    Relationship {
                        id: "someId2".into(),
                        target: PathResource::new("//somePath//Of/Unknown", false).unwrap(),
                        relationship_type: RelationshipType::Unknown("unknown".into()),
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

    use crate::model::PathResource;

    use super::{
        MODEL_TYPE_NS, RELATIONSHIP_NS, Relationship, RelationshipType, Relationships,
        THUMBNAIL_TYPE_NS,
    };

    #[test]
    pub fn fromxml_relationships_test() {
        let xml_string = format!(
            r#"<Relationships xmlns="{}"><Relationship Id="someId" Target="//somePath//Of//Resources" Type="{}" /><Relationship Id="someId1" Target="//somePath//Of//Resources" Type="{}" /><Relationship Id="someId2" Target="//somePath//Of//Unknown" Type="unknown" /></Relationships>"#,
            RELATIONSHIP_NS, MODEL_TYPE_NS, THUMBNAIL_TYPE_NS
        );
        let relationships = from_str::<Relationships>(&xml_string).unwrap();

        assert_eq!(
            relationships,
            Relationships {
                relationships: vec![
                    Relationship {
                        id: "someId".into(),
                        target: PathResource::new("/somePath/Of/Resources", true).unwrap(),
                        relationship_type: RelationshipType::Model,
                    },
                    Relationship {
                        id: "someId1".into(),
                        target: PathResource::new("/somePath/Of/Resources", true).unwrap(),
                        relationship_type: RelationshipType::Thumbnail,
                    },
                    Relationship {
                        id: "someId2".into(),
                        target: PathResource::new("//somePath//Of/Unknown", false).unwrap(),
                        relationship_type: RelationshipType::Unknown("unknown".into()),
                    },
                ],
            }
        );
    }
}

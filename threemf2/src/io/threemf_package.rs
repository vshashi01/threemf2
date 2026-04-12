use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[cfg(feature = "io-write")]
use instant_xml::ToXml;

#[cfg(feature = "io-write")]
use crate::threemf_namespaces::ThreemfNamespace;

use crate::{
    core::model::Model,
    io::{
        XmlNamespace,
        content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
        error::Error,
        parse_xmlns_attributes,
        relationship::{Relationship, RelationshipType, Relationships},
        thumbnail_handle::ThumbnailHandle,
        utils,
    },
};

#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
use crate::io::zip_utils::XmlDeserializer;

use std::collections::HashMap;
use std::io::{self, Read, Seek, Write};

/// Represents a 3mf package, the nested folder structure of the parts
/// in the 3mf package will be flattened into respective dictionaries with
/// the key being the path of the part in the archive package.
#[derive(Debug, Clone)]
pub struct ThreemfPackage {
    /// The root model of the 3mf package.
    /// Expected to always exist and be a valid model with a [Build](crate::core::build::Build) object.
    pub root: Model,

    /// The sub models contained in the file. Usually this is to represent the [Object](crate::core::object::Object)
    /// that are to be referenced in the [root](ThreemfPackage::root) model part.
    /// The key is the path of the model in the archive package.
    pub sub_models: HashMap<String, Model>,

    /// The thumbnails contained in the file.
    /// The key is the path of the thumbnail in the archive package.
    /// The thumbnail paths defined in the [Model](crate::core::model::Model) object should match the keys in this dictionary.
    pub thumbnails: HashMap<String, ThumbnailHandle>,

    /// Bytes of additional data found through Unknown relationship
    /// The key is the path of the thumbnail in the archive package.
    pub unknown_parts: HashMap<String, Vec<u8>>,

    /// The relationships between the different parts in the 3mf package.
    /// The key is the path of the relationship file in the archive package.
    /// Always expected to have at least 1 relationship file,
    /// the root relationship file placed within "_rels" folder at the root of the package
    pub relationships: HashMap<String, Relationships>,

    /// A summary of all Default Content Types that exists in the current 3mf package.
    /// The reader/writer will still read and write data not currently known to library as
    /// unknown data.
    /// The extensions defined in the [ContentTypes.xml]
    ///  file should match the extensions of the parts in the package.
    pub content_types: ContentTypes,

    namespaces: HashMap<String, Vec<XmlNamespace>>,
}

impl ThreemfPackage {
    pub fn new(
        root: Model,
        sub_models: HashMap<String, Model>,
        thumbnails: HashMap<String, ThumbnailHandle>,
        unknown_parts: HashMap<String, Vec<u8>>,
        relationships: HashMap<String, Relationships>,
        content_types: ContentTypes,
    ) -> Self {
        Self {
            root,
            sub_models,
            thumbnails,
            unknown_parts,
            relationships,
            content_types,
            namespaces: HashMap::new(),
        }
    }

    pub(crate) fn new_with_namespaces_map(
        root: Model,
        sub_models: HashMap<String, Model>,
        thumbnails: HashMap<String, ThumbnailHandle>,
        unknown_parts: HashMap<String, Vec<u8>>,
        relationships: HashMap<String, Relationships>,
        content_types: ContentTypes,
        namespaces: HashMap<String, Vec<XmlNamespace>>,
    ) -> Self {
        Self {
            root,
            sub_models,
            thumbnails,
            unknown_parts,
            relationships,
            content_types,
            namespaces,
        }
    }
}

#[cfg(feature = "io-write")]
impl ThreemfPackage {
    /// Writes the 3mf package to a [`io::Write`].
    /// Expects a well formed [ThreemfPackage] object to write the package.
    /// A well formed packaged requires atleast 1 root model and 1 relationship file along with the content types.
    pub fn write<W: Write + Seek>(&self, threemf_archive: W) -> Result<(), Error> {
        let mut zip = ZipWriter::new(threemf_archive);

        Self::archive_write_xml_with_header(
            &mut zip,
            "[Content_Types].xml",
            &self.content_types,
            None,
        )?;

        for (path, relationships) in &self.relationships {
            Self::archive_write_xml_with_header(&mut zip, path, &relationships, None)?;

            for relationship in &relationships.relationships {
                let filename = utils::try_strip_leading_slash(&relationship.target);
                match relationship.relationship_type {
                    RelationshipType::Model => {
                        let model = if *path == *"_rels/.rels" {
                            &self.root
                        } else if let Some(model) = self.sub_models.get(&relationship.target) {
                            model
                        } else {
                            return Err(Error::WriteError(format!(
                                "No model found for relationship target {}",
                                relationship.target
                            )));
                        };
                        Self::archive_write_xml_with_header(
                            &mut zip,
                            filename,
                            model,
                            Some(model.used_namespaces()),
                        )?;
                    }
                    RelationshipType::Thumbnail => {
                        if let Some(image) = self.thumbnails.get(&relationship.target) {
                            zip.start_file(filename, SimpleFileOptions::default())?;
                            zip.write_all(&image.data)?;
                        } else {
                            return Err(Error::WriteError(format!(
                                "No thumbnail image found for relationship target {}",
                                &relationship.target
                            )));
                        }
                    }
                    RelationshipType::Unknown(_) => {
                        if let Some(bytes) = self.unknown_parts.get(&relationship.target) {
                            zip.start_file(filename, SimpleFileOptions::default())?;
                            zip.write_all(bytes)?;
                        } else {
                            return Err(Error::WriteError(format!(
                                "No data found for relationship target {}",
                                &relationship.target
                            )));
                        }
                    }
                }
            }
        }
        zip.finish()?;
        Ok(())
    }

    fn archive_write_xml_with_header<W: Write + Seek, T: ToXml + ?Sized>(
        archive: &mut ZipWriter<W>,
        filename: &str,
        content: &T,
        optional_namespaces_to_keep: Option<Vec<ThreemfNamespace>>,
    ) -> Result<(), Error> {
        use instant_xml::to_string;

        const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#;

        let mut content_string = to_string(&content)?;

        if let Some(namespaces) = optional_namespaces_to_keep {
            Self::filter_unused_namespaces(&mut content_string, &namespaces);
        }

        content_string.insert_str(0, XML_HEADER);

        archive.start_file(filename, SimpleFileOptions::default())?;
        archive.write_all(content_string.as_bytes())?;
        Ok(())
    }

    fn filter_unused_namespaces(xml: &mut String, keep_namespaces: &[ThreemfNamespace]) {
        let keep_uris: std::collections::HashSet<_> =
            keep_namespaces.iter().map(|ns| ns.uri()).collect();

        // Find model tag
        if let Some(model_pos) = xml.find("<model")
            && let Some(end_pos) = xml[model_pos..].find('>')
        {
            let tag_end = model_pos + end_pos + 1;
            let tag_content = &xml[model_pos..tag_end];

            let xmlns_attrs = parse_xmlns_attributes(tag_content);

            // Parse all attributes (simple approach: split by spaces)
            let mut all_attrs = Vec::new();
            let mut current_attr = String::new();
            let mut in_quotes = false;

            for ch in tag_content[6..].chars() {
                // Skip "<model"
                if ch == '"' {
                    in_quotes = !in_quotes;
                    current_attr.push(ch);
                } else if ch == ' ' && !in_quotes {
                    if !current_attr.is_empty() {
                        all_attrs.push(current_attr);
                        current_attr = String::new();
                    }
                } else if ch == '>' {
                    if !current_attr.is_empty() {
                        all_attrs.push(current_attr);
                    }
                    break;
                } else {
                    current_attr.push(ch);
                }
            }

            // Build new tag
            let mut new_tag = String::from("<model");

            // Add kept xmlns attributes
            for ns in &xmlns_attrs {
                if keep_uris.contains(ns.uri.as_str()) {
                    new_tag.push(' ');
                    let attr_name = if let Some(prefix) = &ns.prefix {
                        format!("xmlns:{}", prefix)
                    } else {
                        "xmlns".to_string()
                    };
                    new_tag.push_str(&attr_name);
                    new_tag.push_str("=\"");
                    new_tag.push_str(&ns.uri);
                    new_tag.push('"');
                }
            }

            // Add non-xmlns attributes
            for attr in all_attrs {
                if !attr.starts_with("xmlns") {
                    new_tag.push(' ');
                    new_tag.push_str(&attr);
                }
            }

            new_tag.push('>');

            // Replace in original XML
            xml.replace_range(model_pos..tag_end, &new_tag);
        }
    }
}

#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
impl ThreemfPackage {
    #[cfg(feature = "io-memory-optimized-read")]
    pub fn from_reader_with_memory_optimized_deserializer<R: Read + io::Seek>(
        reader: R,
        process_sub_models: bool,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, process_sub_models, XmlDeserializer::MemoryOptimized)
    }

    #[cfg(feature = "io-speed-optimized-read")]
    pub fn from_reader_with_speed_optimized_deserializer<R: Read + io::Seek>(
        reader: R,
        process_sub_models: bool,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, process_sub_models, XmlDeserializer::SpeedOptimized)
    }

    /// Reads a 3mf package from a type [Read] + [io::Seek].
    /// Expected to deal with nested parts of the 3mf package and flatten them into the respective dictionaries.
    /// Only If [process_sub_models] is set to true, it will process the sub models and thumbnails associated with the sub models in the package.
    /// Will return an error if the package is not a valid 3mf package or if the package contains unsupported content types.
    fn from_reader<R: Read + io::Seek>(
        reader: R,
        process_sub_models: bool,
        deserializer: XmlDeserializer,
    ) -> Result<Self, Error> {
        use crate::io::zip_utils;

        let (mut zip, content_types, _, root_rels_filename) =
            zip_utils::setup_archive_and_content_types(reader, deserializer)?;

        let rels_ext = {
            let rels_content = content_types
                .defaults
                .iter()
                .find(|t| t.content_type == DefaultContentTypeEnum::Relationship);

            match rels_content {
                Some(rels) => &rels.extension,
                None => "rels",
            }
        };

        let mut relationships = HashMap::<String, Relationships>::new();

        let root_rels: Relationships = zip_utils::relationships_from_zip_by_name(
            &mut zip,
            &root_rels_filename,
            &deserializer,
        )?;

        let root_model_rel = root_rels
            .relationships
            .iter()
            .find(|rels| rels.relationship_type == RelationshipType::Model);

        let root_model_path = match root_model_rel {
            Some(rel) => rel.target.clone(),
            None => {
                return Err(Error::ReadError(
                    "Root model relationship not found".to_owned(),
                ));
            }
        };

        relationships.insert(root_rels_filename.clone(), root_rels.clone());

        if process_sub_models {
            let rel_files =
                zip_utils::discover_relationship_files(&mut zip, rels_ext, &root_rels_filename)?;
            for rel_file_path in rel_files {
                let rels = zip_utils::relationships_from_zip_by_name(
                    &mut zip,
                    &rel_file_path[1..],
                    &deserializer,
                )?;
                relationships.insert(rel_file_path, rels);
            }
        }

        let mut processor = processor::ThreemfPackageProcessor::new(content_types, relationships);

        processor.process_relationships(&mut zip, &deserializer, &root_model_path)?;

        Ok(processor.into_threemf_package())
    }

    //only exists in the loading flow and not on the writing flow
    //if a path is not set then its the root model
    pub fn get_namespaces_on_model(&self, model_path: Option<&str>) -> Option<Vec<XmlNamespace>> {
        let path = model_path.unwrap_or("root model");

        if self.namespaces.contains_key(path) {
            let namespaces = self.namespaces.get(path);
            namespaces.cloned()
        } else {
            None
        }
    }
}

impl PartialEq for ThreemfPackage {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
            && self.sub_models == other.sub_models
            && self.thumbnails == other.thumbnails
            && self.unknown_parts == other.unknown_parts
            && self.relationships == other.relationships
            && self.content_types == other.content_types
        //skip namespaces comparison altogether
    }
}

#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
mod processor {
    use zip::ZipArchive;

    use crate::{
        core::model::Model,
        io::{
            ThreemfPackage, XmlNamespace,
            content_types::ContentTypes,
            error::Error,
            relationship::{RelationshipType, Relationships},
            thumbnail_handle::{ImageFormat, ThumbnailHandle},
            utils,
            zip_utils::XmlDeserializer,
        },
    };

    use std::{
        collections::HashMap,
        io::{Read, Seek},
    };
    /// Temporary processor for building ThreemfPackage
    pub(crate) struct ThreemfPackageProcessor {
        root: Option<Model>,
        sub_models: HashMap<String, Model>,
        thumbnails: HashMap<String, ThumbnailHandle>,
        unknown_parts: HashMap<String, Vec<u8>>,
        relationships: HashMap<String, Relationships>,
        content_types: ContentTypes,
        namespaces_map: HashMap<String, Vec<XmlNamespace>>,
    }

    impl ThreemfPackageProcessor {
        pub(crate) fn new(
            content_types: ContentTypes,
            relationships: HashMap<String, Relationships>,
        ) -> Self {
            Self {
                root: None,
                sub_models: HashMap::new(),
                thumbnails: HashMap::new(),
                unknown_parts: HashMap::new(),
                relationships,
                content_types,
                namespaces_map: HashMap::new(),
            }
        }

        pub(crate) fn into_threemf_package(self) -> ThreemfPackage {
            ThreemfPackage::new_with_namespaces_map(
                self.root.expect("Root model should be set"),
                self.sub_models,
                self.thumbnails,
                self.unknown_parts,
                self.relationships,
                self.content_types,
                self.namespaces_map,
            )
        }

        pub(crate) fn process_relationships<R: Read + Seek>(
            &mut self,
            zip: &mut ZipArchive<R>,
            deserializer: &XmlDeserializer,
            root_model_path: &str,
        ) -> Result<(), Error> {
            for rels in self.relationships.values() {
                for rel in &rels.relationships {
                    let name = utils::try_strip_leading_slash(&rel.target);
                    let zip_file = zip.by_name(name);

                    match zip_file {
                        Ok(mut file) => {
                            if file.is_dir() {
                                return Err(Error::ReadError(format!(
                                    r#"Found a folder "{:?}" instead of a file"#,
                                    file.enclosed_name()
                                )));
                            }

                            match rel.relationship_type {
                                RelationshipType::Thumbnail => {
                                    let mut bytes = Vec::new();
                                    file.read_to_end(&mut bytes)?;

                                    let format = {
                                        if let Some(filepath) = file.enclosed_name()
                                            && let Some(os_ext) = filepath.extension()
                                            && let Some(ext) = os_ext.to_str()
                                        {
                                            ImageFormat::from_ext(ext)
                                        } else {
                                            ImageFormat::Unknown
                                        }
                                    };

                                    let thumbnail_rep = ThumbnailHandle {
                                        data: bytes,
                                        format,
                                    };
                                    self.thumbnails
                                        .insert(rel.target.to_string(), thumbnail_rep);
                                }
                                RelationshipType::Model => {
                                    let is_root = rel.target == root_model_path;

                                    let (model, namespaces) =
                                        deserializer.deserialize_model(&mut file)?;
                                    if is_root {
                                        self.root = Some(model);
                                        self.namespaces_map
                                            .insert("root model".to_string(), namespaces);
                                    } else {
                                        self.sub_models.insert(rel.target.to_string(), model);
                                        self.namespaces_map
                                            .insert(rel.target.to_string(), namespaces);
                                    }
                                }
                                RelationshipType::Unknown(_) => {
                                    let mut bytes = Vec::new();
                                    file.read_to_end(&mut bytes)?;

                                    self.unknown_parts.insert(rel.target.to_string(), bytes);
                                }
                            }
                        }
                        Err(err) => return Err(Error::Zip(err)),
                    }
                }
            }
            Ok(())
        }
    }
}

impl From<Model> for ThreemfPackage {
    fn from(value: Model) -> Self {
        let mut rels = HashMap::new();
        rels.insert(
            "_rels/.rels".to_owned(),
            Relationships {
                relationships: vec![Relationship {
                    id: "rel0".to_owned(),
                    target: "3D/3dmodel.model".to_owned(),
                    relationship_type: RelationshipType::Model,
                }],
            },
        );
        Self::new(
            value,
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            rels,
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                ],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            build::Build,
            model::{self, Model},
            object::{Object, ObjectType},
            resources::Resources,
        },
        io::{content_types::*, relationship::*},
    };

    use super::ThreemfPackage;

    use std::fs::File;
    use std::path::PathBuf;
    use std::{collections::HashMap, io::Cursor};

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    pub fn from_reader_root_model_with_memory_optimized_read_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);
        // println!("{:?}", result);

        match result {
            Ok(threemf) => {
                assert_eq!(threemf.content_types.defaults.len(), 3);
                assert_eq!(threemf.sub_models.len(), 1);
                assert_eq!(threemf.thumbnails.len(), 1);
                assert_eq!(threemf.relationships.len(), 2);

                assert!(threemf.sub_models.contains_key("/3D/midway.model"));

                assert!(threemf.relationships.contains_key("_rels/.rels"));
                assert!(
                    threemf
                        .relationships
                        .contains_key("/3D/_rels/3dmodel.model.rels")
                );
                assert!(
                    threemf
                        .thumbnails
                        .contains_key("/Thumbnails/P_XPX_0702_02.png")
                )
            }
            Err(err) => panic!("{:?}", err),
        }
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    pub fn from_reader_root_model_with_speed_optimized_read_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);
        // println!("{:?}", result);

        match result {
            Ok(threemf) => {
                assert_eq!(threemf.content_types.defaults.len(), 3);
                assert_eq!(threemf.sub_models.len(), 1);
                assert_eq!(threemf.thumbnails.len(), 1);
                assert_eq!(threemf.relationships.len(), 2);

                assert!(threemf.sub_models.contains_key("/3D/midway.model"));

                assert!(threemf.relationships.contains_key("_rels/.rels"));
                assert!(
                    threemf
                        .relationships
                        .contains_key("/3D/_rels/3dmodel.model.rels")
                );
                assert!(
                    threemf
                        .thumbnails
                        .contains_key("/Thumbnails/P_XPX_0702_02.png")
                )
            }
            Err(err) => panic!("{:?}", err),
        }
    }

    #[cfg(feature = "io-write")]
    #[test]
    pub fn write_root_model_test() {
        let bytes = {
            use crate::core::{OptionalResourceId, OptionalResourceIndex};

            let bytes = Vec::<u8>::new();
            let mut writer = Cursor::new(bytes);
            let threemf = ThreemfPackage::new(
                Model {
                    unit: Some(model::Unit::Centimeter),
                    requiredextensions: None,
                    recommendedextensions: None,
                    metadata: vec![],
                    resources: Resources {
                        object: vec![Object {
                            id: 1,
                            objecttype: Some(ObjectType::Model),
                            thumbnail: None,
                            partnumber: None,
                            name: Some("Some object".to_owned()),
                            pid: OptionalResourceId::none(),
                            pindex: OptionalResourceIndex::none(),
                            uuid: Some("uuid".to_owned()),
                            slicestackid: OptionalResourceId::none(),
                            slicepath: None,
                            meshresolution: None,
                            kind: None,
                        }],
                        basematerials: vec![],
                        slicestack: vec![],
                    },
                    build: Build {
                        uuid: None,
                        item: vec![],
                    },
                },
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::from([(
                    "_rels/.rels".to_owned(),
                    Relationships {
                        relationships: vec![Relationship {
                            id: "rel0".to_owned(),
                            target: "3D/3Dmodel.model".to_owned(),
                            relationship_type: RelationshipType::Model,
                        }],
                    },
                )]),
                ContentTypes {
                    defaults: vec![
                        DefaultContentTypes {
                            extension: "rels".to_owned(),
                            content_type: DefaultContentTypeEnum::Relationship,
                        },
                        DefaultContentTypes {
                            extension: "model".to_owned(),
                            content_type: DefaultContentTypeEnum::Model,
                        },
                    ],
                },
            );
            threemf.write(&mut writer).unwrap();
            writer
        };

        // usually breaks due to additional namespace declaration that arent filtered out
        assert_eq!(bytes.into_inner().len(), 944);
    }

    #[cfg(all(feature = "io-memory-optimized-read", feature = "io-write"))]
    #[test]
    pub fn io_unknown_content_test() {
        let test_file_bytes = include_bytes!("../../tests/data/test.txt");
        let mut writer = Cursor::new(Vec::<u8>::new());
        let unknown_target = "/Metadata/test.txt";

        let package = ThreemfPackage::new(
            Model {
                unit: Some(model::Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![],
                    basematerials: vec![],
                    slicestack: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![],
                },
            },
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(unknown_target.to_owned(), test_file_bytes.into())]),
            HashMap::from([(
                "_rels/.rels".to_owned(),
                Relationships {
                    relationships: vec![
                        Relationship {
                            id: "rel0".to_owned(),
                            target: "3D/3Dmodel.model".to_owned(),
                            relationship_type: RelationshipType::Model,
                        },
                        Relationship {
                            id: "rel1".to_owned(),
                            target: unknown_target.to_owned(),
                            relationship_type: RelationshipType::Unknown(
                                "Metadata/text".to_owned(),
                            ),
                        },
                    ],
                },
            )]),
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::Relationship,
                        extension: "rels".to_owned(),
                    },
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::Unknown("Metadata/text".to_owned()),
                        extension: "txt".to_owned(),
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                ],
            },
        );

        let write_result = package.write(&mut writer);
        assert!(write_result.is_ok());

        let read_result =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(writer, false);

        match read_result {
            Ok(package) => {
                assert!(package.unknown_parts.contains_key(unknown_target));
                let read_unknown_bytes = package.unknown_parts.get(unknown_target).unwrap();
                assert_eq!(read_unknown_bytes, test_file_bytes);
            }
            Err(_) => panic!("io unknown content test failed"),
        }
    }

    #[cfg(all(feature = "io-memory-optimized-read", feature = "io-write"))]
    #[test]
    pub fn io_thumbnail_content_test() {
        use crate::io::thumbnail_handle::{ImageFormat, ThumbnailHandle};

        let test_file_bytes = include_bytes!("../../tests/data/test_thumbnail.png");
        let thumbnail_rep = ThumbnailHandle {
            data: test_file_bytes.to_vec(),
            format: ImageFormat::Png,
        };

        let mut writer = Cursor::new(Vec::<u8>::new());
        let thumbnail_target = "/Thumbnails/test_thumbnail.png";

        let package = ThreemfPackage::new(
            Model {
                unit: Some(model::Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![],
                    basematerials: vec![],
                    slicestack: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![],
                },
            },
            HashMap::new(),
            HashMap::from([(thumbnail_target.to_owned(), thumbnail_rep)]),
            HashMap::new(),
            HashMap::from([(
                "_rels/.rels".to_owned(),
                Relationships {
                    relationships: vec![
                        Relationship {
                            id: "rel0".to_owned(),
                            target: "3D/3Dmodel.model".to_owned(),
                            relationship_type: RelationshipType::Model,
                        },
                        Relationship {
                            id: "rel0x".to_owned(),
                            target: thumbnail_target.to_owned(),
                            relationship_type: RelationshipType::Thumbnail,
                        },
                    ],
                },
            )]),
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::Relationship,
                        extension: "rels".to_owned(),
                    },
                    DefaultContentTypes {
                        content_type: DefaultContentTypeEnum::ImagePng,
                        extension: "png".to_owned(),
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                ],
            },
        );

        let write_result = package.write(&mut writer);
        assert!(write_result.is_ok());

        let read_result =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(writer, false);

        match read_result {
            Ok(package) => {
                use crate::io::thumbnail_handle::ImageFormat;

                assert!(package.thumbnails.contains_key(thumbnail_target));
                let thumbnail_rep = package.thumbnails.get(thumbnail_target).unwrap();
                assert_eq!(thumbnail_rep.data.len(), 8571);
                assert_eq!(thumbnail_rep.format, ImageFormat::Png);
            }
            Err(_) => panic!("io thumbnail test failed"),
        }
    }

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn i_root_namespaces_tracking_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mgx-core-prod-beamlattice-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);
        match result {
            Ok(threemf) => {
                let root_namespaces = threemf.get_namespaces_on_model(None).unwrap();
                //println!("Namespaces: {:?}", root_namespaces);
                assert_eq!(root_namespaces.len(), 5);
            }
            Err(err) => panic!("{:?}", err),
        }
    }

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn i_submodel_namespaces_tracking_test() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);
        match result {
            Ok(threemf) => {
                let root_namespaces = threemf
                    .get_namespaces_on_model(Some("/3D/Objects/Object(3).model"))
                    .unwrap();
                //println!("Namespaces: {:?}", root_namespaces);
                assert_eq!(root_namespaces.len(), 4);
            }
            Err(err) => panic!("{:?}", err),
        }
    }

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn test_boolean_operations_namespace_tracking() {
        use crate::threemf_namespaces::BOOLEAN_NS;

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);
        match result {
            Ok(threemf) => {
                let root_namespaces = threemf.get_namespaces_on_model(None).unwrap();

                // Verify that Boolean Operations namespace is tracked
                let has_boolean_ns = root_namespaces.iter().any(|ns| ns.uri == BOOLEAN_NS);
                assert!(
                    has_boolean_ns,
                    "Boolean Operations namespace ({})",
                    BOOLEAN_NS
                );

                // Also verify the namespace prefix
                let boolean_ns = root_namespaces
                    .iter()
                    .find(|ns| ns.uri == BOOLEAN_NS)
                    .expect("Boolean namespace should be present");
                assert_eq!(
                    boolean_ns.prefix.as_deref(),
                    Some("bo"),
                    "Boolean namespace should have 'bo' prefix"
                );
            }
            Err(err) => panic!("{:?}", err),
        }
    }
}

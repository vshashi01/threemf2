use zip::ZipArchive;

use crate::io::{
    XmlNamespace,
    content_types::{ContentTypes, DefaultContentTypeEnum},
    error::Error,
    parse_xmlns_attributes,
    relationship::Relationships,
};

use crate::core::model::Model;

use std::ffi::OsStr;
use std::io::{Read, Seek};
use std::path::Path;

/// Enum for different XML deserialization strategies
#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read"
))]
#[derive(Clone, Copy)]
pub(crate) enum XmlDeserializer {
    #[cfg(feature = "io-memory-optimized-read")]
    MemoryOptimized,
    #[cfg(feature = "io-speed-optimized-read")]
    SpeedOptimized,
}

impl XmlDeserializer {
    pub(crate) fn deserialize_content_types<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<ContentTypes, Error> {
        match self {
            #[cfg(feature = "io-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                instant_xml::from_str::<ContentTypes>(&xml_string).map_err(Error::from)
            }
            #[cfg(feature = "io-speed-optimized-read")]
            XmlDeserializer::SpeedOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                serde_roxmltree::from_str::<ContentTypes>(&xml_string).map_err(Error::from)
            }
        }
    }

    pub(crate) fn deserialize_relationships<R: Read>(
        &self,
        mut reader: R,
    ) -> Result<Relationships, Error> {
        match self {
            #[cfg(feature = "io-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                instant_xml::from_str::<Relationships>(&xml_string).map_err(Error::from)
            }
            #[cfg(feature = "io-speed-optimized-read")]
            XmlDeserializer::SpeedOptimized => {
                let mut xml_string = String::new();
                reader.read_to_string(&mut xml_string)?;
                serde_roxmltree::from_str::<Relationships>(&xml_string).map_err(Error::from)
            }
        }
    }

    pub(crate) fn deserialize_model<R: Read>(
        &self,
        reader: &mut R,
    ) -> Result<(Model, Vec<XmlNamespace>), Error> {
        let mut xml_string = String::new();
        reader.read_to_string(&mut xml_string)?;

        let namespaces = parse_xmlns_attributes(&xml_string);

        let model = match self {
            #[cfg(feature = "io-memory-optimized-read")]
            XmlDeserializer::MemoryOptimized => instant_xml::from_str::<Model>(&xml_string)?,
            #[cfg(feature = "io-speed-optimized-read")]
            XmlDeserializer::SpeedOptimized => serde_roxmltree::from_str::<Model>(&xml_string)?,
        };

        Ok((model, namespaces))
    }
}

pub(crate) fn setup_archive_and_content_types<R: Read + Seek>(
    reader: R,
    deserializer: XmlDeserializer,
) -> Result<(ZipArchive<R>, ContentTypes, String, String), Error> {
    let mut zip = ZipArchive::new(reader)?;

    let (content_types, content_types_string) = parse_content_types(&mut zip, deserializer)?;
    let rels_ext = determine_relationships_extension(&content_types);

    let root_rels_filename = "_rels/.{extension}".replace("{extension}", &rels_ext);

    Ok((zip, content_types, content_types_string, root_rels_filename))
}

fn parse_content_types<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    deserializer: XmlDeserializer,
) -> Result<(ContentTypes, String), Error> {
    let content_types_file = zip.by_name("[Content_Types].xml");
    match content_types_file {
        Ok(mut file) => {
            let mut xml_string = String::new();
            file.read_to_string(&mut xml_string)?;
            let content_types = deserializer.deserialize_content_types(xml_string.as_bytes())?;
            Ok((content_types, xml_string))
        }
        Err(err) => Err(Error::Zip(err)),
    }
}

fn determine_relationships_extension(content_types: &ContentTypes) -> String {
    content_types
        .defaults
        .iter()
        .find(|t| t.content_type == DefaultContentTypeEnum::Relationship)
        .map(|rels| rels.extension.clone())
        .unwrap_or_else(|| "rels".to_string())
}

/// Find all relationship files in the archive (excluding the root relationships file)
pub(crate) fn discover_relationship_files<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    rels_ext: &str,
    root_rels_filename: &str,
) -> Result<Vec<String>, Error> {
    let mut rel_files = Vec::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;

        if file.is_file()
            && let Some(path) = file.enclosed_name()
            && path.extension() == Some(OsStr::new(rels_ext))
            && path != Path::new(root_rels_filename)
        {
            let zip_name = path
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let final_path = format!("/{zip_name}");

            rel_files.push(final_path);
        }
    }

    Ok(rel_files)
}

pub(crate) fn relationships_from_zipfile<R: Read>(
    file: zip::read::ZipFile<'_, R>,
    deserializer: &XmlDeserializer,
) -> Result<Relationships, Error> {
    deserializer.deserialize_relationships(file)
}

pub(crate) fn relationships_from_zip_by_name<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    zip_filename: &str,
    deserializer: &XmlDeserializer,
) -> Result<Relationships, Error> {
    let rels_file = zip.by_name(zip_filename);
    match rels_file {
        Ok(file) => relationships_from_zipfile(file, deserializer),
        Err(err) => Err(Error::Zip(err)),
    }
}

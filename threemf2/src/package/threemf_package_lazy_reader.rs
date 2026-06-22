use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek};

use once_cell::unsync::OnceCell;
use zip::ZipArchive;

use crate::model::PathResource;
use crate::model::domain::model::Model;
use crate::package::{
    domain::{
        content_types::{ContentTypes, DefaultContentTypeEnum},
        relationship::{RelationshipType, Relationships},
        thumbnail_handle::{ImageFormat, ThumbnailHandle},
        zip_utils::{self, XmlDeserializer},
    },
    error::Error,
};

/// Cache policy for lazy-loaded data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CachePolicy {
    /// Cache everything after first access (best for typical usage where data is accessed multiple times)
    CacheAll,
    /// Never cache, always re-read from zip (best for memory-constrained environments, read-once patterns)
    #[default]
    NoCache,
}

/// Represents a 3mf package with lazy loading.
/// Unlike [`ThreemfPackage`](crate::io::ThreemfPackage), this struct only parses metadata upfront
/// (content types and relationships), and loads models, thumbnails, and other data on-demand.
///
/// This is ideal for memory-constrained environments or when you need to inspect package contents
/// without loading all data.
///
pub struct ThreemfPackageLazyReader<R: Read + Seek> {
    archive: RefCell<ZipArchive<R>>,
    deserializer: XmlDeserializer,
    cache_policy: CachePolicy,

    // always eagerly loaded
    content_types: ContentTypes,
    relationships: HashMap<PathResource, Relationships>,
    root_model_path: PathResource,

    // always cached on first access
    root_model: OnceCell<Model>,

    // cached based on cachepolicy
    sub_models: RefCell<HashMap<PathResource, Model>>,
    thumbnails: RefCell<HashMap<PathResource, ThumbnailHandle>>,
    unknown_parts: RefCell<HashMap<PathResource, Vec<u8>>>,
}

impl<R: Read + Seek> ThreemfPackageLazyReader<R> {
    fn from_reader(
        reader: R,
        deserializer: XmlDeserializer,
        cache_policy: CachePolicy,
    ) -> Result<Self, Error> {
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

        let mut relationships = HashMap::new();
        let root_rels: Relationships = zip_utils::relationships_from_zip_by_name(
            &mut zip,
            &root_rels_filename,
            &deserializer,
        )?;

        let root_model_path = root_rels
            .relationships
            .iter()
            .find(|rels| rels.relationship_type == RelationshipType::Model)
            .map(|rel| rel.target.clone())
            .ok_or_else(|| Error::ReadError("Root model relationship not found".to_owned()))?;

        relationships.insert(root_rels_filename.to_owned(), root_rels);

        let rel_files = zip_utils::discover_relationship_files(
            &mut zip,
            rels_ext,
            root_rels_filename.as_str(),
        )?;
        for rel_file_path in rel_files {
            let rels =
                zip_utils::relationships_from_zip_by_name(&mut zip, &rel_file_path, &deserializer)?;
            relationships.insert(rel_file_path, rels);
        }

        Ok(Self {
            archive: RefCell::new(zip),
            deserializer,
            cache_policy,
            content_types,
            relationships,
            root_model_path,
            root_model: OnceCell::new(),
            sub_models: RefCell::new(HashMap::new()),
            thumbnails: RefCell::new(HashMap::new()),
            unknown_parts: RefCell::new(HashMap::new()),
        })
    }

    pub fn content_types(&self) -> &ContentTypes {
        &self.content_types
    }

    pub fn relationships(&self) -> &HashMap<PathResource, Relationships> {
        &self.relationships
    }

    pub fn root_model_path(&self) -> &PathResource {
        &self.root_model_path
    }

    pub fn model_paths(&self) -> impl Iterator<Item = &PathResource> {
        self.relationships
            .values()
            .flat_map(|r| &r.relationships)
            .filter_map(|rel| {
                if matches!(rel.relationship_type, RelationshipType::Model) {
                    Some(&rel.target)
                } else {
                    None
                }
            })
    }

    pub fn thumbnail_paths(&self) -> impl Iterator<Item = &PathResource> {
        self.relationships
            .values()
            .flat_map(|r| &r.relationships)
            .filter_map(|rel| {
                if matches!(rel.relationship_type, RelationshipType::Thumbnail) {
                    Some(&rel.target)
                } else {
                    None
                }
            })
    }

    pub fn unknown_part_paths(&self) -> impl Iterator<Item = &str> {
        self.relationships
            .values()
            .flat_map(|r| &r.relationships)
            .filter_map(|rel| {
                if matches!(rel.relationship_type, RelationshipType::Unknown(_)) {
                    Some(rel.target.as_str())
                } else {
                    None
                }
            })
    }

    pub fn root_model(&self) -> Result<&Model, Error> {
        self.root_model
            .get_or_try_init(|| self.load_model_from_archive(&self.root_model_path))
    }

    pub fn with_model<F, T>(&self, path: &PathResource, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Model) -> T,
    {
        if path == &self.root_model_path {
            let model = self.root_model()?;
            return Ok(f(model));
        }

        let is_model = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                &rel.target == path && matches!(rel.relationship_type, RelationshipType::Model)
            });

        if !is_model {
            return Err(Error::ResourceNotFound(path.to_string()));
        }

        match self.cache_policy {
            CachePolicy::NoCache => {
                // Always load fresh, don't cache
                // We can't return a reference to temporary data, so we must cache at least temporarily
                // Check if already in cache from a previous call
                if self.sub_models.borrow().contains_key(path) {
                    let cache = self.sub_models.borrow();
                    let model = cache.get(path).unwrap();
                    Ok(f(model))
                } else {
                    let model = self.load_model_from_archive(path)?;
                    self.sub_models.borrow_mut().insert(path.clone(), model);
                    let cache = self.sub_models.borrow();
                    let model = cache.get(path).unwrap();
                    Ok(f(model))
                }
            }
            CachePolicy::CacheAll => {
                // Check cache first
                if self.sub_models.borrow().contains_key(path) {
                    let cache = self.sub_models.borrow();
                    let model = cache.get(path).unwrap();
                    Ok(f(model))
                } else {
                    // Load and cache
                    let model = self.load_model_from_archive(path)?;
                    self.sub_models.borrow_mut().insert(path.clone(), model);
                    let cache = self.sub_models.borrow();
                    let model = cache.get(path).unwrap();
                    Ok(f(model))
                }
            }
        }
    }

    pub fn with_thumbnail<F, T>(&self, path: &PathResource, f: F) -> Result<T, Error>
    where
        F: FnOnce(&ThumbnailHandle) -> T,
    {
        // Check if it's a valid thumbnail path
        let is_thumbnail = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                &rel.target == path && matches!(rel.relationship_type, RelationshipType::Thumbnail)
            });

        if !is_thumbnail {
            return Err(Error::ResourceNotFound(path.to_string()));
        }

        if self.thumbnails.borrow().contains_key(path) {
            let cache = self.thumbnails.borrow();
            let image = cache.get(path).unwrap();
            Ok(f(image))
        } else {
            let image = self.load_thumbnail_from_archive(path)?;
            self.thumbnails.borrow_mut().insert(path.clone(), image);
            let cache = self.thumbnails.borrow();
            let image = cache.get(path).unwrap();
            Ok(f(image))
        }
    }

    /// Get an unknown part by path (lazy loaded, cached based on policy)
    ///
    /// Returns `None` if no unknown part exists at the given path.
    pub fn with_unknown_part<F, T>(&self, path: &PathResource, f: F) -> Result<T, Error>
    where
        F: FnOnce(&[u8]) -> T,
    {
        // Check if it's a valid unknown part path
        let is_unknown = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                &rel.target == path && matches!(rel.relationship_type, RelationshipType::Unknown(_))
            });

        if !is_unknown {
            return Err(Error::ResourceNotFound(path.to_string()));
        }

        // Check cache (works for both policies since we need to return a reference)
        if self.unknown_parts.borrow().contains_key(path) {
            let cache = self.unknown_parts.borrow();
            let bytes = cache.get(path).unwrap();
            Ok(f(bytes))
        } else {
            // Load and cache
            let bytes = self.load_unknown_part_from_archive(path)?;
            self.unknown_parts.borrow_mut().insert(path.clone(), bytes);
            let cache = self.unknown_parts.borrow();
            let bytes = cache.get(path).unwrap();
            Ok(f(bytes))
        }
    }

    /// Access raw XML content of a model by path (pull-based, reads from ZIP each time)
    ///
    /// Returns an error if no model exists at the given path.
    pub fn with_model_xml<F, T>(&self, path: &PathResource, f: F) -> Result<T, Error>
    where
        F: FnOnce(&str) -> T,
    {
        // Validate path exists and is a model relationship
        let is_model = self
            .relationships
            .values()
            .flat_map(|r| &r.relationships)
            .any(|rel| {
                &rel.target == path && matches!(rel.relationship_type, RelationshipType::Model)
            });

        if !is_model {
            return Err(Error::ResourceNotFound(format!("Model at path: {}", path)));
        }

        // Read XML directly from ZIP archive
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(path.as_str_without_leading_slash())?;
        let mut xml_string = String::new();
        file.read_to_string(&mut xml_string)?;

        Ok(f(&xml_string))
    }

    /// Access raw XML content of relationships by path (pull-based, reads from ZIP each time)
    ///
    /// Returns an error if no relationships file exists at the given path.
    pub fn with_relationships_xml<F, T>(&self, path: &PathResource, f: F) -> Result<T, Error>
    where
        F: FnOnce(&str) -> T,
    {
        // Check if relationships file exists
        if !self.relationships.contains_key(path) {
            return Err(Error::ResourceNotFound(format!(
                "Relationships file at path: {}",
                path
            )));
        }

        // Read relationships XML directly from ZIP
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(path.as_str_without_leading_slash())?;
        let mut xml_string = String::new();
        file.read_to_string(&mut xml_string)?;

        Ok(f(&xml_string))
    }

    /// Access raw XML content of content types (pull-based, reads from ZIP each time)
    pub fn with_content_types_xml<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&str) -> T,
    {
        // Read content types XML directly from ZIP
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name("[Content_Types].xml")?;
        let mut xml_string = String::new();
        file.read_to_string(&mut xml_string)?;

        Ok(f(&xml_string))
    }

    fn load_model_from_archive(&self, path: &PathResource) -> Result<Model, Error> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(path.as_str_without_leading_slash())?;
        let model_string = zip_utils::read_zipfile_to_string(&mut file)?;
        self.deserializer.deserialize_model(&model_string)
    }

    fn load_thumbnail_from_archive(&self, path: &PathResource) -> Result<ThumbnailHandle, Error> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(path.as_str_without_leading_slash())?;
        let mut bytes: Vec<u8> = vec![];
        file.read_to_end(&mut bytes)?;

        let format = {
            if let Some(filepath) = file.enclosed_name()
                && let Some(os_ext) = filepath.extension()
                && let Some(ext) = os_ext.to_str()
            {
                ImageFormat::from_ext(ext)
            } else {
                return Err(Error::ThumbnailError(format!(
                    "Referenced thumbnail path: {path} is not a valid archive path to thumbnail data"
                )));
            }
        };

        let thumbnail_rep = ThumbnailHandle {
            data: bytes,
            format,
        };
        Ok(thumbnail_rep)
    }

    fn load_unknown_part_from_archive(&self, path: &PathResource) -> Result<Vec<u8>, Error> {
        let mut archive = self.archive.borrow_mut();
        let mut file = archive.by_name(path.as_str_without_leading_slash())?;
        let mut bytes: Vec<u8> = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

#[cfg(feature = "package-memory-optimized-read")]
impl<R: Read + Seek> ThreemfPackageLazyReader<R> {
    /// Create a pull-based package with memory-optimized deserialization
    ///
    /// * `reader` - A readable and seekable source (e.g., `File`)
    /// * `cache_policy` - Whether to cache loaded data (`CachePolicy::NoCache` is default)
    pub fn from_reader_with_memory_optimized_deserializer(
        reader: R,
        cache_policy: CachePolicy,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, XmlDeserializer::MemoryOptimized, cache_policy)
    }
}

#[cfg(feature = "io-speed-optimized-read")]
impl<R: Read + Seek> ThreemfPackageLazyReader<R> {
    /// Create a pull-based package with speed-optimized deserialization
    ///
    /// * `reader` - A readable and seekable source (e.g., `File`)
    /// * `cache_policy` - Whether to cache loaded data (`CachePolicy::NoCache` is default)
    #[deprecated(
        note = "speed-optimized-read is deprecated; use from_reader_with_memory_optimized_deserializer"
    )]
    pub fn from_reader_with_speed_optimized_deserializer(
        reader: R,
        cache_policy: CachePolicy,
    ) -> Result<Self, Error> {
        Self::from_reader(reader, XmlDeserializer::SpeedOptimized, cache_policy)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::fs::File;
    use std::path::PathBuf;

    use super::*;

    #[cfg(feature = "package-memory-optimized-read")]
    #[test]
    fn test_pull_based_root_model_lazy_load() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        )
        .unwrap();

        assert_eq!(package.relationships().len(), 1);
        assert!(package.root_model_path().as_str().contains("3dmodel.model"));

        let paths: Vec<_> = package.model_paths().collect();
        assert!(!paths.is_empty());

        let root_model = package.root_model().unwrap();
        assert_eq!(root_model.build.item.len(), 2);
        assert_eq!(root_model.used_namespaces().len(), 3);
    }

    #[cfg(feature = "package-memory-optimized-read")]
    #[test]
    fn test_pull_based_with_sub_models() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::CacheAll,
        )
        .unwrap();

        assert_eq!(package.content_types().defaults.len(), 3);
        assert_eq!(package.relationships().len(), 2);

        let model_paths: Vec<_> = package.model_paths().collect();
        assert!(model_paths.len() >= 2); // root + at least one sub-model

        let root_model = package.root_model().unwrap();
        assert!(!root_model.resources.object.is_empty());
        assert_eq!(root_model.used_namespaces().len(), 2);

        let sub_model_path = PathResource::new("/3D/midway.model", true).unwrap();
        let exists = package.with_model(&sub_model_path, |_| true);
        assert!(exists.is_ok());

        let sub_model_path = PathResource::new("/SomeThing/ThatDoesNotExist.model", true).unwrap();
        let exists = package.with_model(&sub_model_path, |_| true);
        assert!(exists.is_err());
    }

    #[cfg(feature = "package-memory-optimized-read")]
    #[test]
    fn test_pull_based_thumbnails() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        )
        .unwrap();

        let thumbnail_paths: Vec<_> = package.thumbnail_paths().collect();
        assert!(!thumbnail_paths.is_empty());

        let thumbnail_path = thumbnail_paths[0];
        package
            .with_thumbnail(thumbnail_path, |rep| {
                assert_eq!(rep.data.len(), 8571);
                assert_eq!(rep.format, ImageFormat::Png);
            })
            .unwrap();
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    fn test_pull_based_speed_optimized() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackageLazyReader::from_reader_with_speed_optimized_deserializer(
            reader,
            CachePolicy::CacheAll,
        )
        .unwrap();

        assert!(!package.relationships().is_empty());

        let root_model = package.root_model().unwrap();
        assert_eq!(root_model.build.item.len(), 2);
        assert_eq!(root_model.used_namespaces().len(), 3);
    }

    #[cfg(feature = "package-memory-optimized-read")]
    #[test]
    fn test_string_extraction() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/P_XPX_0702_02.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        )
        .unwrap();

        // Test model XML extraction
        package
            .with_model_xml(
                &PathResource::new("/3D/3dmodel.model", true).unwrap(),
                |xml| {
                    assert!(xml.contains("<model"));
                    assert!(xml.contains("</model>"));
                    assert!(xml.contains("xmlns"));
                },
            )
            .unwrap();

        // Test sub-model XML extraction
        package
            .with_model_xml(
                &PathResource::new("/3D/midway.model", true).unwrap(),
                |xml| {
                    assert!(xml.contains("<model"));
                    assert!(xml.contains("</model>"));
                },
            )
            .unwrap();

        // Test relationships XML extraction
        package
            .with_relationships_xml(&PathResource::new("_rels/.rels", true).unwrap(), |xml| {
                assert!(xml.contains("<Relationships"));
                assert!(xml.contains("<Relationship"));
            })
            .unwrap();

        // Test sub-model relationships XML extraction
        package
            .with_relationships_xml(
                &PathResource::new("/3D/_rels/3dmodel.model.rels", true).unwrap(),
                |xml| {
                    assert!(xml.contains("<Relationships"));
                },
            )
            .unwrap();

        // Test content types XML extraction
        package
            .with_content_types_xml(|xml| {
                assert!(xml.contains("<Types"));
                assert!(xml.contains("<Default"));
            })
            .unwrap();

        // Test invalid paths return errors
        let invalid_result = package.with_model_xml(
            &PathResource::new("/invalid/path.model", true).unwrap(),
            |_| (),
        );
        assert!(matches!(invalid_result, Err(Error::ResourceNotFound(_))));

        let invalid_rels = package.with_relationships_xml(
            &PathResource::new("/invalid/rels.xml", true).unwrap(),
            |_| (),
        );
        assert!(matches!(invalid_rels, Err(Error::ResourceNotFound(_))));
    }
}

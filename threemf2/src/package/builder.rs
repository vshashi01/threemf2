//! Package builder for assembling 3MF files from multiple models.
//!
//! [`ThreemfPackageBuilder`] collects root models, sub-models, thumbnails, and unknown parts,
//! then generates the correct ZIP structure with content types and relationship files.

use std::collections::{HashMap, HashSet};

use compact_str::format_compact;
use thiserror::Error;

use crate::{
    model::{
        PathResource, PathResourceError, StrResource,
        domain::{component, model::Model},
    },
    package::{
        ThreemfPackage,
        domain::{
            content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
            relationship::{Relationship, RelationshipType, Relationships},
            thumbnail_handle::{ImageFormat, ThumbnailHandle},
        },
    },
};

const DEFAULT_ROOT_MODEL_PATH: &str = "/3D/3Dmodel.model";

/// Errors that can occur when building a 3MF package.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PackageBuildError {
    /// Root model is not set.
    #[error("Root model is not set")]
    RootModelNotSet,

    /// Duplicate model path detected.
    #[error("Duplicate model path: {0}")]
    DuplicateModelPath(String),

    /// Referenced model path not found in the package.
    #[error("Referenced model path not found: {0}")]
    MissingModel(String),

    /// Invalid model path format.
    #[error("Invalid model path: {0}")]
    InvalidModelPath(String),

    /// OPC Part Path is missing an extension.
    #[error("OPC Part Path is missing an extension: {0}")]
    MissingOPCPartExtension(String),
}

/// Builder for assembling 3MF packages with multiple model files.
pub struct ThreemfPackageBuilder {
    root_model: Option<Model>,
    root_model_path: PathResource,
    sub_models: HashMap<PathResource, Model>,
    content_types: ContentTypes,
    relationships_overrides: HashMap<PathResource, Relationships>,
    thumbnails: HashMap<PathResource, ThumbnailHandle>,
    unknown_parts: HashMap<PathResource, (String, Vec<u8>)>,
}

impl Default for ThreemfPackageBuilder {
    fn default() -> Self {
        Self {
            root_model: None,
            root_model_path: PathResource::new(DEFAULT_ROOT_MODEL_PATH, false).unwrap(),
            sub_models: HashMap::new(),
            content_types: ContentTypes { defaults: vec![] },
            relationships_overrides: HashMap::new(),
            thumbnails: HashMap::new(),
            unknown_parts: HashMap::new(),
        }
    }
}

impl ThreemfPackageBuilder {
    /// Creates a new package builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the root model for the package.
    pub fn set_root_model(&mut self, model: Model) -> &mut Self {
        self.root_model = Some(model);
        self.add_model_content_type();
        self
    }

    // pub fn set_root_model_path(&mut self, path: &str) -> Result<&mut Self, PackageBuildError> {
    //     let normalized = normalize_model_path(path)?;
    //     self.root_model_path = normalized;
    //     Ok(self)
    // }

    /// Adds a sub-model to the package.
    pub fn add_model(&mut self, path: &str, model: Model) -> Result<&mut Self, PackageBuildError> {
        let normalized = normalize_model_path(path)?;
        if normalized == self.root_model_path || self.sub_models.contains_key(&normalized) {
            return Err(PackageBuildError::DuplicateModelPath(
                normalized.to_string(),
            ));
        }
        self.sub_models.insert(normalized, model);
        self.add_model_content_type();
        Ok(self)
    }

    // pub fn set_relationships(&mut self, path: &str, relationships: Relationships) -> &mut Self {
    //     self.relationships_overrides
    //         .insert(path.to_owned(), relationships);
    //     self
    // }

    /// Adds a thumbnail image to the package.
    pub fn add_thumbnail(
        &mut self,
        path: &str,
        data: Vec<u8>,
    ) -> Result<&mut Self, PackageBuildError> {
        let normalized = normalize_model_path(path)?;
        let ext = path.rsplit('.').next();
        match ext {
            Some(ext) => {
                let format = ImageFormat::from_ext(ext);
                self.thumbnails.insert(
                    normalized,
                    ThumbnailHandle {
                        data,
                        format: format.clone(),
                    },
                );

                match format {
                    ImageFormat::Png => {
                        if !self
                            .content_types
                            .defaults
                            .iter()
                            .any(|c| c.content_type == DefaultContentTypeEnum::ImagePng)
                        {
                            self.content_types.defaults.push(DefaultContentTypes {
                                extension: ext.into(),
                                content_type: DefaultContentTypeEnum::ImagePng,
                            });
                        }
                    }
                    ImageFormat::Jpeg => {
                        if !self
                            .content_types
                            .defaults
                            .iter()
                            .any(|c| c.content_type == DefaultContentTypeEnum::ImageJPEG)
                        {
                            self.content_types.defaults.push(DefaultContentTypes {
                                extension: ext.into(),
                                content_type: DefaultContentTypeEnum::ImageJPEG,
                            });
                        }
                    }
                    ImageFormat::Unknown(unknown_ext) => {
                        if !self
                            .content_types
                            .defaults
                            .iter()
                            .any(|c| c.extension == unknown_ext)
                        {
                            self.content_types.defaults.push(DefaultContentTypes {
                                extension: unknown_ext.clone(),
                                content_type: DefaultContentTypeEnum::Unknown(StrResource::from(
                                    format_compact!("Image/{unknown_ext}"),
                                )),
                            });
                        }
                    }
                }
                Ok(self)
            }
            None => Err(PackageBuildError::MissingOPCPartExtension(path.to_owned())),
        }
    }

    /// Adds an unknown OPC part to the package.
    pub fn add_unknown_opc_part(
        &mut self,
        path: &str,
        ns: &str,
        data: Vec<u8>,
    ) -> Result<&mut Self, PackageBuildError> {
        let normalized = normalize_model_path(path)?;
        self.unknown_parts.insert(normalized, (ns.to_owned(), data));
        let ext = path.rsplit('.').next();
        match ext {
            Some(ext) => {
                if !self
                    .content_types
                    .defaults
                    .iter()
                    .any(|c| c.extension == ext.into())
                {
                    self.content_types.defaults.push(DefaultContentTypes {
                        extension: ext.into(),
                        content_type: DefaultContentTypeEnum::Unknown(StrResource::from(ns)),
                    });
                }
                Ok(self)
            }
            None => Err(PackageBuildError::MissingOPCPartExtension(path.to_owned())),
        }
    }

    /// Builds the 3MF package.
    pub fn build(mut self) -> Result<ThreemfPackage, PackageBuildError> {
        let root = self.root_model.ok_or(PackageBuildError::RootModelNotSet)?;
        let referenced_paths =
            collect_referenced_model_paths(&self.root_model_path, &root, &self.sub_models);

        for reference in referenced_paths {
            if *reference == self.root_model_path {
                continue;
            }
            if !self.sub_models.contains_key(reference) {
                return Err(PackageBuildError::MissingModel(reference.to_string()));
            }
        }

        let relationships = build_relationships(
            &self.root_model_path,
            &root,
            &self.sub_models,
            &self.relationships_overrides,
            &self.thumbnails,
        );

        self.content_types.defaults.push(DefaultContentTypes {
            extension: "rels".into(),
            content_type: DefaultContentTypeEnum::Relationship,
        });

        // validate_content_types(&self.content_types)?;

        let mut stripped_unknown_parts = HashMap::new();
        for (path, (_, part)) in self.unknown_parts {
            stripped_unknown_parts.insert(path, part);
        }

        Ok(ThreemfPackage::new(
            root,
            self.sub_models,
            self.thumbnails,
            stripped_unknown_parts,
            relationships,
            self.content_types,
        ))
    }

    fn add_model_content_type(&mut self) {
        if !self
            .content_types
            .defaults
            .iter()
            .any(|c| c.content_type == DefaultContentTypeEnum::Model)
        {
            self.content_types.defaults.push(DefaultContentTypes {
                extension: "model".into(),
                content_type: DefaultContentTypeEnum::Model,
            });
        }
    }
}

fn normalize_model_path(path: &str) -> Result<PathResource, PackageBuildError> {
    PathResource::new(path, true).map_err(|_| PackageBuildError::InvalidModelPath(path.to_owned()))
}

fn collect_referenced_model_paths<'a>(
    root_model_path: &'a PathResource,
    root: &'a Model,
    sub_models: &'a HashMap<PathResource, Model>,
) -> HashSet<&'a PathResource> {
    let mut referenced = HashSet::new();
    referenced.extend(collect_model_references(root));
    referenced.insert(root_model_path);
    for model in sub_models.values() {
        referenced.extend(collect_model_references(model));
    }
    referenced
}

fn collect_model_references(model: &Model) -> HashSet<&PathResource> {
    let mut referenced = HashSet::new();

    for item in &model.build.item {
        if let Some(path) = &item.path {
            referenced.insert(path);
        }
    }

    for object in &model.resources.object {
        if let Some(path) = &object.slicepath {
            referenced.insert(path);
        }

        if let Some(components) = object.get_components_object() {
            collect_component_paths(&components.component, &mut referenced);
        }

        if let Some(shape) = object.get_boolean_shape_object() {
            if let Some(path) = &shape.path {
                referenced.insert(path);
            }
            for boolean in &shape.booleans {
                if let Some(path) = &boolean.path {
                    referenced.insert(path);
                }
            }
        }
    }

    for slicestack in &model.resources.slicestack {
        for sliceref in &slicestack.sliceref {
            referenced.insert(&sliceref.slicepath);
        }
    }

    referenced
}

fn collect_component_paths<'a>(
    components: &'a [component::Component],
    referenced: &mut HashSet<&'a PathResource>,
) {
    for component in components {
        if let Some(path) = &component.path {
            referenced.insert(path);
        }
    }
}

fn build_relationships(
    root_model_path: &PathResource,
    root: &Model,
    sub_models: &HashMap<PathResource, Model>,
    overrides: &HashMap<PathResource, Relationships>,
    thumbnails: &HashMap<PathResource, ThumbnailHandle>,
) -> HashMap<PathResource, Relationships> {
    let mut relationships = HashMap::new();

    let root_rels_path = PathResource::new("_rels/.rels", true).unwrap();
    if let Some(overrides) = overrides.get(&root_rels_path) {
        relationships.insert(root_rels_path, overrides.clone());
    } else {
        let mut rels = Vec::new();
        rels.push(Relationship {
            id: "rel0".into(),
            target: root_model_path.to_owned(),
            relationship_type: RelationshipType::Model,
        });

        let mut rel_index = 1;
        for path in thumbnails.keys() {
            rels.push(Relationship {
                id: format_compact!("rel{rel_index}").into(),
                target: path.clone(),
                relationship_type: RelationshipType::Thumbnail,
            });
            rel_index += 1;
        }

        relationships.insert(
            root_rels_path,
            Relationships {
                relationships: rels,
            },
        );
    }

    let mut model_entries = vec![(root_model_path, root)];
    for (path, model) in sub_models {
        model_entries.push((path, model));
    }

    for (model_path, model) in model_entries {
        if let Ok(rels_path) = rels_path_for_model(model_path) {
            if overrides.contains_key(&rels_path) {
                relationships.insert(rels_path.clone(), overrides[&rels_path].clone());
                continue;
            }

            let referenced = collect_model_references(model);
            if referenced.is_empty() {
                continue;
            }

            let mut rels = Vec::new();
            for (index, target) in referenced.into_iter().enumerate() {
                if target == model_path {
                    continue;
                }
                rels.push(Relationship {
                    id: format_compact!("rel{}", index).into(),
                    target: target.clone(),
                    relationship_type: RelationshipType::Model,
                });
            }

            if rels.is_empty() {
                continue;
            }

            relationships.insert(
                rels_path,
                Relationships {
                    relationships: rels,
                },
            );
        } else {
            panic!("Something went wrong with Model Path")
        }
    }

    relationships
}

fn rels_path_for_model(model_path: &PathResource) -> Result<PathResource, PathResourceError> {
    if let Some((dir, filename)) = model_path.as_str_without_leading_slash().rsplit_once('/') {
        let path = PathResource::new(
            format_compact!("{}/_rels/{}.rels", dir, filename).as_str(),
            true,
        )?;
        Ok(path)
    } else {
        let path = PathResource::new(
            format!("_rels/{}.rels", model_path.as_str_without_leading_slash()).as_str(),
            true,
        )?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::builder::ModelBuilder;
    use crate::model::domain::{model::Unit, object::ObjectType};

    #[test]
    fn build_multimodel_package() {
        let mut root_builder = ModelBuilder::new(Unit::Millimeter, true);
        root_builder.make_production_extension_required().unwrap();
        root_builder
            .add_build(Some(crate::model::UuidResource::from("build-uuid")))
            .unwrap();

        let obj_id = root_builder
            .add_mesh_object(|obj| {
                obj.object_type(ObjectType::Model).uuid("obj-uuid");
                obj.add_vertex(&[0.0, 0.0, 0.0]);
                Ok(())
            })
            .unwrap();
        root_builder
            .add_components_object(|obj| {
                obj.uuid("components-uuid");
                obj.add_component_advanced(obj_id, |comp| {
                    comp.uuid("comp-uuid").path("/3D/sub/sub.model");
                });
                Ok(())
            })
            .unwrap();
        root_builder
            .add_build_item_advanced(obj_id, |item| {
                item.uuid("item-uuid");
            })
            .unwrap();

        let root = root_builder.build().unwrap();

        let mut sub_builder = ModelBuilder::new(Unit::Millimeter, false);
        sub_builder
            .add_mesh_object(|obj| {
                obj.object_type(ObjectType::Model);
                obj.add_vertex(&[0.0, 0.0, 0.0]);
                Ok(())
            })
            .unwrap();
        let sub = sub_builder.build().unwrap();

        let mut builder = ThreemfPackageBuilder::new();
        builder.set_root_model(root);
        builder.add_model("/3D/sub/sub.model", sub).unwrap();
        builder
            .add_thumbnail("Thumbnail/thumbnail.png", vec![222, 169])
            .unwrap();
        builder
            .add_unknown_opc_part("/3D/unknown/unknown.txt", "File/Text", vec![255, 180])
            .unwrap();
        let package = builder.build().unwrap();

        assert_eq!(package.sub_models.len(), 1);
        assert!(
            package
                .relationships
                .contains_key(&PathResource::new("_rels/.rels", true).unwrap())
        );
        assert!(
            package
                .relationships
                .contains_key(&PathResource::new("3D/_rels/3Dmodel.model.rels", true).unwrap())
        );
        assert!(
            package
                .thumbnails
                .contains_key(&PathResource::new("Thumbnail/thumbnail.png", true).unwrap())
        );
        assert!(
            package
                .sub_models
                .contains_key(&PathResource::new("3D/sub/sub.model", true).unwrap())
        );
        assert!(
            package
                .unknown_parts
                .contains_key(&PathResource::new("3D/unknown/unknown.txt", true).unwrap())
        );
        assert_eq!(package.root.build.item.len(), 1);

        assert_eq!(package.content_types.defaults.len(), 4);
    }

    #[test]
    fn missing_referenced_model_errors() {
        let mut root_builder = ModelBuilder::new(Unit::Millimeter, true);
        root_builder.make_production_extension_required().unwrap();
        root_builder
            .add_build(Some(crate::model::UuidResource::from("build-uuid")))
            .unwrap();

        let obj_id = root_builder
            .add_mesh_object(|obj| {
                obj.object_type(ObjectType::Model).uuid("obj-uuid");
                obj.add_vertex(&[0.0, 0.0, 0.0]);
                Ok(())
            })
            .unwrap();
        root_builder
            .add_components_object(|obj| {
                obj.uuid("components-uuid");
                obj.add_component_advanced(obj_id, |comp| {
                    comp.uuid("comp-uuid").path("/3D/missing.model");
                });
                Ok(())
            })
            .unwrap();
        root_builder
            .add_build_item_advanced(obj_id, |item| {
                item.uuid("item-uuid");
            })
            .unwrap();

        let root = root_builder.build().unwrap();
        let mut builder = ThreemfPackageBuilder::new();
        builder.set_root_model(root);
        let result = builder.build();

        assert!(matches!(
            result,
            Err(PackageBuildError::MissingModel(path))
                if path == "/3D/missing.model"
        ));
    }

    #[test]
    fn rels_path_for_model_is_derived() {
        assert_eq!(
            rels_path_for_model(&PathResource::new("/3D/sub.model", true).unwrap())
                .unwrap()
                .as_str(),
            "/3D/_rels/sub.model.rels"
        );
        assert_eq!(
            rels_path_for_model(&PathResource::new("sub.model", true).unwrap())
                .unwrap()
                .as_str(),
            "/_rels/sub.model.rels"
        );
    }

    #[test]
    fn invalid_model_path_rejected() {
        let mut builder = ThreemfPackageBuilder::new();
        let mut root_builder = ModelBuilder::new(Unit::Millimeter, false);
        root_builder
            .add_mesh_object(|obj| {
                obj.object_type(ObjectType::Model);
                obj.add_vertex(&[0.0, 0.0, 0.0]);
                Ok(())
            })
            .unwrap();
        let model = root_builder.build().unwrap();

        let result = builder.add_model("../bad.model", model);
        assert!(matches!(
            result,
            Err(PackageBuildError::InvalidModelPath(path)) if path == "../bad.model"
        ));
    }

    #[test]
    fn content_types_includes_thumbnail_ext() {
        let mut builder = ThreemfPackageBuilder::new();
        let mut root_builder = ModelBuilder::new(Unit::Millimeter, true);
        root_builder.add_build(None).unwrap();
        let obj_id = root_builder
            .add_mesh_object(|obj| {
                obj.object_type(ObjectType::Model);
                obj.add_vertex(&[0.0, 0.0, 0.0]);
                Ok(())
            })
            .unwrap();
        root_builder.add_build_item(obj_id).unwrap();
        builder.set_root_model(root_builder.build().unwrap());

        builder
            .add_thumbnail("/3D/Thumbnails/thumb.png", vec![0x1, 0x2])
            .unwrap();

        let package = builder.build().unwrap();
        assert!(package.content_types.defaults.iter().any(|entry| {
            entry.extension == "png".into()
                && entry.content_type == DefaultContentTypeEnum::ImagePng
        }));
    }
}

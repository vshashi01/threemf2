//! Builder API for constructing 3MF models programmatically.
//!
//! This module provides a builder based API to make 3MF Models.
//! The builder pattern makes it easy to construct complex 3D models with proper validation
//! and automatic handling of IDs, extensions, and relationships.
//!
//! # Overview
//!
//! The builder API is organized around several key builders:
//!
//! - [`ModelBuilder`] - Main entry point for creating 3MF models (root or sub-models)
//! - [`MeshObjectBuilder`] - Creates objects with triangle mesh geometry
//! - [`ComponentsObjectBuilder`] - Creates assembly objects that reference other objects
//! - [`BooleanObjectBuilder`] - Creates boolean shape objects for CSG operations
//! - [`BeamLatticeBuilder`] - Adds beam lattice structures to meshes
//! - [`BuildBuilder`] - Configures the build section (what gets printed)
//! - [`TriangleSetsBuilder`] - Organizes triangles into named groups
//!
//! # Root vs Sub-Models
//!
//! 3MF models can be either root models or sub-models:
//!
//! - **Root models** (`is_root = true`): Must have a `Build` section that specifies which
//!   objects should be printed. This is the main model file in a 3MF package.
//! - **Sub-models** (`is_root = false`): Cannot have a `Build` section. These are referenced
//!   by other models and stored in separate files within the 3MF package.
//!
//! # Production Extension
//!
//! The 3MF Production extension adds UUIDs for tracking objects through the manufacturing
//! process. When enabled via [`ModelBuilder::make_production_extension_required()`], the
//! builder enforces that all objects, components, build items, and builds have UUIDs set.
//!
//! # Automatic Extension Management
//!
//! The builder automatically detects and adds required 3MF extensions based on features used:
//!
//! - Beam lattice extension when beams are added
//! - Beam lattice balls extension when balls are used
//! - Boolean operations extension when boolean shapes are added
//! - Production extension when enabled
//! - Triangle sets as recommended extension
//!
//! # Type Safety
//!
//! The builder uses [`ObjectId`] as a type-safe wrapper around object IDs to prevent
//! accidental misuse of raw integer IDs. Object IDs are automatically assigned and managed
//! by the [`ModelBuilder`]. If you prefer to opt out of them, you can use the [`from_builder`] methods
//! to create and add custom objects based on specific object builders.

use thiserror::Error;

use crate::{
    model::domain::{
        beamlattice::{self},
        boolean::{self},
        build::{Build, Item},
        component::{Component, Components},
        displacement::{self},
        material::{self},
        mesh::{self},
        metadata::Metadata,
        model::{self, Model, ThreemfExtensions},
        object::{self},
        resources::{Base, BaseMaterials, Resources},
        slice::{self},
        transform::Transform,
        triangle_set,
    },
    threemf_namespaces::ThreemfNamespace,
};

pub use beamlattice::{BallMode, CapMode};
pub use boolean::BooleanOperation;
pub use model::Unit;
pub use object::ObjectType;
pub use slice::MeshResolution;

use crate::model::{
    OptionalResourceId, OptionalResourceIndex, PathResource, ResourceId, ResourceIdCollection,
    ResourceIndex, ResourceIndexCollection, StrResource, UuidResource,
};

use std::ops::{Deref, DerefMut};

/// Errors that can occur when building a [`Model`].
///
/// These errors are returned from [`ModelBuilder::build()`] and related methods
/// when validation fails or invalid state is detected.
#[derive(Debug, Error, Clone)]
pub enum ModelError {
    /// Root model requires a Build section but none was added.
    ///
    /// Root models must have at least one build item. Call [`ModelBuilder::add_build()`]
    /// before calling [`ModelBuilder::build()`].
    #[error("Build is not set for the Model. Root Model and adding Build Items requires a Build!")]
    BuildItemNotSet,

    /// Attempted to add a Build section to a non-root model.
    ///
    /// Sub-models cannot have Build sections. Either create a root model
    /// (`is_root = true`) or don't call [`ModelBuilder::add_build()`].
    #[error("Build is not allowed in non-root Model")]
    BuildOnlyAllowedInRootModel,

    /// Error occurred while building the Build section.
    #[error("Something wrong when adding Build")]
    BuildError(#[from] BuildError),

    /// Error occurred while building a build item.
    #[error("Something wrong when adding Items")]
    ItemError(#[from] ItemError),
}

/// Errors related to the 3MF Production extension.
///
/// When the Production extension is enabled via [`ModelBuilder::make_production_extension_required()`],
/// all objects, components, build items, and builds must have UUIDs. Additionally, the `path`
/// attribute is only allowed when the Production extension is enabled.
#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum ProductionExtensionError {
    /// Object is missing a UUID when Production extension is required.
    ///
    /// Call [`MeshObjectBuilder::uuid()`] or [`ComponentsObjectBuilder::uuid()`] to set it.
    #[error("Object Uuid is not set with Production extension enabled!")]
    ObjectUuidNotSet,

    /// Component is missing a UUID when Production extension is required.
    ///
    /// Call [`ComponentBuilder::uuid()`] when configuring the component.
    #[error("Component Uuid is not set with Production extension enabled!")]
    ComponentUuidNotSet,

    /// Build item is missing a UUID when Production extension is required.
    ///
    /// Use [`ModelBuilder::add_build_item_advanced()`] with [`ItemBuilder::uuid()`].
    #[error("Item Uuid is not set with Production extension enabled!")]
    ItemUuidNotSet,

    /// Build is missing a UUID when Production extension is required.
    ///
    /// Pass a UUID when calling [`ModelBuilder::add_build()`].
    #[error("Build Uuid is not set with Production extension enabled!")]
    BuildUuidNotSet,

    /// Component has a path set but Production extension is not enabled.
    ///
    /// Call [`ModelBuilder::make_production_extension_required()`] before setting paths.
    #[error("Component Path is set without Production extension enabled!")]
    PathUsedOnComponent,

    /// Build item has a path set but Production extension is not enabled.
    ///
    /// Call [`ModelBuilder::make_production_extension_required()`] before setting paths.
    #[error("Item Path is set without Production extension enabled!")]
    PathUsedOnItem,
}

/// Builder for constructing 3MF [`Model`] structs with a fluent API.
///
/// `ModelBuilder` is the main entry point for programmatically creating 3MF models.
/// It manages object IDs, validates relationships, and automatically handles 3MF
/// extensions based on features used.
///
/// # Root vs Sub-Models
///
/// Models can be either root models or sub-models:
///
/// - **Root models** (`is_root = true`): Must contain a Build section. This is the main
///   model file in a 3MF package (typically `/3D/3dmodel.model`).
/// - **Sub-models** (`is_root = false`): Cannot have a Build section. These are auxiliary
///   models referenced by the root model or other sub-models.
pub struct ModelBuilder {
    unit: Option<Unit>,
    requiredextensions: Vec<ThreemfNamespace>,
    recommendedextensions: Vec<ThreemfNamespace>,
    metadata: Vec<Metadata>,
    resources: ResourcesBuilder,
    build: Option<BuildBuilder>, //in submodels, Build item is not allowed

    // tracks if the model is intended as a root model
    // if true, Build is required
    // else adding Build is not allowed
    is_root: bool,

    // tracks next object id
    next_object_id: ObjectId,

    // tracks next slicestack id
    next_slicestack_id: SliceStackId,

    // tracks next material resource ids
    next_colorgroup_id: ResourceId,
    next_texture2dgroup_id: ResourceId,
    next_texture2d_id: ResourceId,
    next_composite_materials_id: ResourceId,
    next_multi_properties_id: ResourceId,
    next_basematerials_id: ResourceId,

    // tracks next displacement resource ids
    next_displacement2d_id: ResourceId,
    next_normvectorgroup_id: ResourceId,
    next_disp2dgroup_id: ResourceId,

    // tracks if the model requires production ext
    // ensures UUID is set at the minimum
    is_production_ext_required: bool,
}

impl ModelBuilder {
    /// Create a new `ModelBuilder` with default values.
    ///
    /// # Parameters
    ///
    /// - `unit`: The unit of measurement for the model (e.g., [`Unit::Millimeter`])
    /// - `is_root`: Whether this is a root model (`true`) or sub-model (`false`)
    ///
    /// Root models must have a Build section, while sub-models cannot have one.
    pub fn new(unit: Unit, is_root: bool) -> Self {
        Self {
            unit: Some(unit),
            requiredextensions: vec![],
            recommendedextensions: vec![],
            metadata: Vec::new(),
            resources: ResourcesBuilder::new(),
            build: None,
            is_root,
            next_object_id: 1.into(),
            next_slicestack_id: 1.into(),
            next_colorgroup_id: 1,
            next_texture2dgroup_id: 1,
            next_texture2d_id: 1,
            next_composite_materials_id: 1,
            next_multi_properties_id: 1,
            next_basematerials_id: 1,
            next_displacement2d_id: 1,
            next_normvectorgroup_id: 1,
            next_disp2dgroup_id: 1,
            is_production_ext_required: false,
        }
    }

    /// Set the unit of measurement for the model.
    ///
    /// This can be called multiple times; the last value set will be used.
    pub fn unit(&mut self, unit: Unit) -> &mut Self {
        self.unit = Some(unit);
        self
    }

    /// Change whether this model is a root model or sub-model.
    ///
    /// - `true`: Root model (must have Build section)
    /// - `false`: Sub-model (cannot have Build section)
    pub fn make_root(&mut self, is_root: bool) -> &mut Self {
        self.is_root = is_root;
        self
    }

    /// Enable the 3MF Production extension and enforce UUID requirements.
    ///
    /// When the Production extension is enabled:
    /// - All objects must have UUIDs set via [`MeshObjectBuilder::uuid()`] or [`ComponentsObjectBuilder::uuid()`]
    /// - All components must have UUIDs set via [`ComponentBuilder::uuid()`]
    /// - The build must have a UUID passed to [`ModelBuilder::add_build()`]
    /// - All build items must have UUIDs set via [`ItemBuilder::uuid()`]
    /// - The `path` attribute becomes available on components and items
    ///
    /// This method validates that all existing objects, components, and build items
    /// already have UUIDs set. If any are missing, it returns an error.
    ///
    /// The Production extension namespace is automatically added to required extensions.
    ///
    /// # Errors
    ///
    /// Returns [`ProductionExtensionError`] if any existing objects, components, builds,
    /// or build items are missing required UUIDs.
    pub fn make_production_extension_required(
        &mut self,
    ) -> Result<&mut Self, ProductionExtensionError> {
        // at the time the production extension is set as required,
        // we should anyway check if existing items fulfil the contract
        //
        for o in &self.resources.objects {
            if o.uuid.is_none() {
                return Err(ProductionExtensionError::ObjectUuidNotSet);
            } else if let Some(kind) = &o.kind
                && let object::ObjectKind::Components(components) = kind
                && components.component.iter().any(|c| c.uuid.is_none())
            {
                return Err(ProductionExtensionError::ComponentUuidNotSet);
            }

            if let Some(build) = &self.build {
                if build.uuid.is_some() {
                    if build.items.iter().any(|i| i.uuid.is_none()) {
                        return Err(ProductionExtensionError::ItemUuidNotSet);
                    }
                } else {
                    return Err(ProductionExtensionError::BuildUuidNotSet);
                }
            }
        }
        self.is_production_ext_required = true;
        Ok(self)
    }

    /// Add a required 3MF extension to the model.
    ///
    /// Required extensions must be understood by readers to process the file.
    /// Some extensions are automatically added based on features used (e.g., beam lattice).
    pub fn add_required_extension(&mut self, extension: ThreemfNamespace) -> &mut Self {
        match extension {
            ThreemfNamespace::Core => self,
            _ => {
                self.requiredextensions.push(extension);
                self
            }
        }
    }

    /// Add a recommended 3MF extension to the model.
    ///
    /// Recommended extensions can be ignored by readers if not supported.
    /// Some extensions are automatically added based on features used (e.g., triangle sets).
    pub fn add_recommended_extension(&mut self, extension: ThreemfNamespace) -> &mut Self {
        match extension {
            ThreemfNamespace::Core => self,
            _ => {
                self.recommendedextensions.push(extension);
                self
            }
        }
    }

    /// Add metadata key-value pair to the model.
    ///
    /// Metadata provides information about the model such as application name,
    /// author, creation date, etc.
    ///
    /// # Parameters
    ///
    /// - `name`: The metadata key
    /// - `value`: The metadata value (or `None` for an empty value)
    pub fn add_metadata(&mut self, name: &str, value: Option<&str>) -> &mut Self {
        self.metadata.push(Metadata {
            name: name.into(),
            preserve: None,
            value: value.map(Into::into),
        });
        self
    }

    /// Add a mesh object to the model using a builder closure.
    ///
    /// The object is automatically assigned a unique [`ObjectId`] which is returned.
    /// Use this ID to reference the object in build items or components.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`MeshObjectBuilder`]
    ///
    /// # Returns
    ///
    /// The auto-assigned [`ObjectId`] for the created object.
    pub fn add_mesh_object<F>(&mut self, f: F) -> Result<ObjectId, MeshObjectError>
    where
        F: FnOnce(&mut MeshObjectBuilder) -> Result<(), MeshObjectError>,
    {
        let id = self.next_object_id;

        let mut obj_builder = MeshObjectBuilder::new(id, self.is_production_ext_required);
        f(&mut obj_builder)?;

        self.add_mesh_object_from_builder(obj_builder)
    }

    /// Add a mesh object from a pre-configured [`MeshObjectBuilder`].
    ///
    /// This is an advanced method for cases where you need to construct the builder
    /// separately. Most users should use [`add_mesh_object()`](ModelBuilder::add_mesh_object) instead.
    pub fn add_mesh_object_from_builder(
        &mut self,
        builder: MeshObjectBuilder,
    ) -> Result<ObjectId, MeshObjectError> {
        let id = builder.object_id;
        let object = builder.build()?;

        if let Some(mesh) = object.get_mesh() {
            self.set_recommended_namespaces_for_mesh(mesh);
        }

        self.resources.objects.push(object);
        self.next_object_id = ObjectId(id.0 + 1);

        Ok(id)
    }

    /// Add a displacement mesh object to the model using a builder closure.
    pub fn add_displacement_mesh_object<F>(
        &mut self,
        f: F,
    ) -> Result<ObjectId, DisplacementMeshObjectError>
    where
        F: FnOnce(&mut DisplacementMeshObjectBuilder) -> Result<(), DisplacementMeshObjectError>,
    {
        let id = self.next_object_id;

        let mut obj_builder =
            DisplacementMeshObjectBuilder::new(id, self.is_production_ext_required);
        f(&mut obj_builder)?;

        self.add_displacement_mesh_object_from_builder(obj_builder)
    }

    /// Add a displacement mesh object from a pre-configured [`DisplacementMeshObjectBuilder`].
    pub fn add_displacement_mesh_object_from_builder(
        &mut self,
        builder: DisplacementMeshObjectBuilder,
    ) -> Result<ObjectId, DisplacementMeshObjectError> {
        let id = builder.object_id;
        let object = builder.build()?;

        if let Some(mesh) = object.get_displacement_mesh() {
            self.set_recommended_namespaces_for_triangle_sets(mesh.trianglesets.is_some());
        }

        self.resources.objects.push(object);
        self.next_object_id = ObjectId(id.0 + 1);

        Ok(id)
    }

    /// Add a components (assembly) object to the model using a builder closure.
    ///
    /// Components objects reference other objects to create assemblies or composed parts.
    /// Each component can have its own transform and references an existing object by ID.
    ///
    /// The object is automatically assigned a unique [`ObjectId`] which is returned.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`ComponentsObjectBuilder`]
    ///
    /// # Returns
    ///
    /// The auto-assigned [`ObjectId`] for the created object.
    pub fn add_components_object<F>(&mut self, f: F) -> Result<ObjectId, ComponentsObjectError>
    where
        F: FnOnce(&mut ComponentsObjectBuilder) -> Result<(), ComponentsObjectError>,
    {
        let id = self.next_object_id;

        let all_object_ids = self
            .resources
            .objects
            .iter()
            .map(|o| ObjectId(o.id))
            .collect::<Vec<_>>();

        let mut obj_builder =
            ComponentsObjectBuilder::new(id, &all_object_ids, self.is_production_ext_required);
        f(&mut obj_builder)?;

        self.add_composed_part_object_from_builder(obj_builder)
    }

    /// Add a components object from a pre-configured [`ComponentsObjectBuilder`].
    ///
    /// This is an advanced method for cases where you need to construct the builder
    /// separately. Most users should use [`add_components_object()`](ModelBuilder::add_components_object) instead.
    pub fn add_composed_part_object_from_builder(
        &mut self,
        builder: ComponentsObjectBuilder,
    ) -> Result<ObjectId, ComponentsObjectError> {
        let id = builder.object_id;
        let object = builder.build()?;

        self.resources.objects.push(object);
        self.next_object_id = ObjectId(id.0 + 1);

        Ok(id)
    }

    /// Add a boolean shape object to the model using a builder closure.
    ///
    /// Boolean shape objects define CSG (Constructive Solid Geometry) operations between
    /// a base object and one or more operand objects. The supported operations are:
    /// - **Union**: Merges shapes together
    /// - **Difference**: Subtracts operands from the base
    /// - **Intersection**: Keeps only the overlapping volume
    ///
    /// The object is automatically assigned a unique [`ObjectId`] which is returned.
    /// The boolean operations extension is automatically marked as required.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`BooleanObjectBuilder`]
    ///
    /// # Returns
    ///
    /// The auto-assigned [`ObjectId`] for the created object.
    ///
    /// # Errors
    ///
    /// Returns [`BooleanShapeError`] if validation fails (e.g., missing base object,
    /// no operands, or missing UUID when Production extension is enabled).
    pub fn add_booleanshape_object<F>(&mut self, f: F) -> Result<ObjectId, BooleanShapeError>
    where
        F: FnOnce(&mut BooleanObjectBuilder) -> Result<(), BooleanShapeError>,
    {
        let id = self.next_object_id;

        let mut obj_builder = BooleanObjectBuilder::new(id, self.is_production_ext_required);
        f(&mut obj_builder)?;

        self.add_booleanshape_object_from_builder(obj_builder)
    }

    /// Add a boolean shape object from a pre-configured [`BooleanObjectBuilder`].
    ///
    /// This is an advanced method for cases where you need to construct the builder
    /// separately. Most users should use [`add_booleanshape_object()`](ModelBuilder::add_booleanshape_object) instead.
    pub fn add_booleanshape_object_from_builder(
        &mut self,
        builder: BooleanObjectBuilder,
    ) -> Result<ObjectId, BooleanShapeError> {
        let id = builder.object_id;
        let object = builder.build()?;

        self.resources.objects.push(object);
        self.next_object_id = ObjectId(id.0 + 1);

        Ok(id)
    }

    /// Add a Build section to the model.
    ///
    /// The Build section specifies which objects should be manufactured (printed).
    /// Only root models can have a Build section.
    ///
    /// After calling this method, use [`add_build_item()`](ModelBuilder::add_build_item) or
    /// [`add_build_item_advanced()`](ModelBuilder::add_build_item_advanced) to add objects
    /// to the build plate.
    ///
    /// # Parameters
    ///
    /// - `uuid`: Optional UUID for the build (required if Production extension is enabled)
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BuildOnlyAllowedInRootModel`] if called on a sub-model.
    /// Returns [`BuildError::BuildUuidNotSet`] if Production extension is enabled but no UUID is provided.
    pub fn add_build(&mut self, uuid: Option<UuidResource>) -> Result<&mut Self, ModelError> {
        if !self.is_root {
            return Err(ModelError::BuildOnlyAllowedInRootModel);
        }
        let mut build_builder = BuildBuilder::new();
        if let Some(uuid) = uuid {
            build_builder.uuid(uuid);
        }

        //check if the Build can be created at this time
        build_builder.can_build(self.is_production_ext_required)?;
        self.build = Some(build_builder);

        Ok(self)
    }

    /// Add a simple build item referencing an object by ID.
    ///
    /// The object will be added to the build plate with default settings (no transform,
    /// no partnumber, no UUID unless required by Production extension).
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the object to add to the build plate
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BuildItemNotSet`] if [`add_build()`](ModelBuilder::add_build) hasn't been called yet.
    pub fn add_build_item(&mut self, object_id: ObjectId) -> Result<&mut Self, ModelError> {
        self.add_build_item_advanced(object_id, |_f| {})
    }

    /// Add a build item with advanced configuration.
    ///
    /// This method allows you to configure transforms, partnumber, UUID, and path
    /// for the build item using a closure.
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the object to add to the build plate
    /// - `f`: A closure that configures the [`ItemBuilder`]
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::BuildItemNotSet`] if [`add_build()`](ModelBuilder::add_build) hasn't been called yet.
    pub fn add_build_item_advanced<F>(
        &mut self,
        object_id: ObjectId,
        f: F,
    ) -> Result<&mut Self, ModelError>
    where
        F: FnOnce(&mut ItemBuilder),
    {
        match &mut self.build {
            Some(build) => {
                build.add_build_item(object_id, self.is_production_ext_required, f)?;

                Ok(self)
            }
            None => Err(ModelError::BuildItemNotSet),
        }
    }

    /// Add a slice stack to the model using a builder closure.
    ///
    /// Slice stacks contain 2.5D slice data (layers of 2D polygons) that can be
    /// referenced by objects. The slice stack ID is automatically assigned and returned.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`SliceStackBuilder`]
    ///
    /// # Returns
    ///
    /// The assigned slice stack ID as [`ResourceId`]
    ///
    pub fn add_slice_stack<F>(&mut self, f: F) -> Result<SliceStackId, SliceStackBuilderError>
    where
        F: FnOnce(&mut SliceStackBuilder),
    {
        let id = self.next_slicestack_id;
        let mut builder = SliceStackBuilder::new(id);

        f(&mut builder);
        let slice_stack = builder.build()?;

        self.resources.slicestack.push(slice_stack);
        self.next_slicestack_id = SliceStackId(id.0 + 1);

        Ok(id)
    }

    /// Add a boolean shape object from a pre-configured [`BooleanObjectBuilder`].
    ///
    /// This is an advanced method for cases where you need to construct the builder
    /// separately. Most users should use [`add_booleanshape_object()`](ModelBuilder::add_booleanshape_object) instead.
    pub fn add_slice_stack_from_builder(
        &mut self,
        builder: SliceStackBuilder,
    ) -> Result<SliceStackId, SliceStackBuilderError> {
        let id = builder.id;
        let object = builder.build()?;

        self.resources.slicestack.push(object);
        self.next_slicestack_id = SliceStackId(id.0 + 1);

        Ok(SliceStackId(id.0))
    }

    /// Add a color group resource.
    pub fn add_color_group<F>(&mut self, f: F) -> ResourceId
    where
        F: FnOnce(&mut ColorGroupBuilder),
    {
        let id = self.next_colorgroup_id;
        let mut builder = ColorGroupBuilder::new(id);
        f(&mut builder);
        self.resources.colorgroup.push(builder.build());
        self.next_colorgroup_id += 1;
        id
    }

    /// Add a texture2d resource.
    pub fn add_texture2d<F>(&mut self, f: F) -> Result<ResourceId, Texture2DError>
    where
        F: FnOnce(&mut Texture2DBuilder),
    {
        let id = self.next_texture2d_id;
        let mut builder = Texture2DBuilder::new(id);
        f(&mut builder);
        let texture = builder.build()?;
        self.resources.texture2d.push(texture);
        self.next_texture2d_id += 1;
        Ok(id)
    }

    /// Add a texture2d group resource.
    pub fn add_texture2d_group<F>(&mut self, f: F) -> Result<ResourceId, Texture2DGroupError>
    where
        F: FnOnce(&mut Texture2DGroupBuilder),
    {
        let id = self.next_texture2dgroup_id;
        let mut builder = Texture2DGroupBuilder::new(id);
        f(&mut builder);
        let group = builder.build()?;
        self.resources.texture2dgroup.push(group);
        self.next_texture2dgroup_id += 1;
        Ok(id)
    }

    /// Add a composite materials resource.
    pub fn add_composite_materials<F>(
        &mut self,
        f: F,
    ) -> Result<ResourceId, CompositeMaterialsError>
    where
        F: FnOnce(&mut CompositeMaterialsBuilder),
    {
        let id = self.next_composite_materials_id;
        let mut builder = CompositeMaterialsBuilder::new(id);
        f(&mut builder);
        let materials = builder.build()?;
        self.resources.compositematerials.push(materials);
        self.next_composite_materials_id += 1;
        Ok(id)
    }

    /// Add a multi-properties resource.
    pub fn add_multi_properties<F>(&mut self, f: F) -> Result<ResourceId, MultiPropertiesError>
    where
        F: FnOnce(&mut MultiPropertiesBuilder),
    {
        let id = self.next_multi_properties_id;
        let mut builder = MultiPropertiesBuilder::new(id);
        f(&mut builder);
        let props = builder.build()?;
        self.resources.multiproperties.push(props);
        self.next_multi_properties_id += 1;
        Ok(id)
    }

    /// Add a base materials resource.
    pub fn add_base_materials<F>(&mut self, f: F) -> ResourceId
    where
        F: FnOnce(&mut BaseMaterialsBuilder),
    {
        let id = self.next_basematerials_id;
        let mut builder = BaseMaterialsBuilder::new(id);
        f(&mut builder);
        self.resources.basematerials.push(builder.build());
        self.next_basematerials_id += 1;
        id
    }

    /// Add a displacement2d resource.
    pub fn add_displacement2d<F>(&mut self, f: F) -> Result<ResourceId, Displacement2DError>
    where
        F: FnOnce(&mut Displacement2DBuilder),
    {
        let id = self.next_displacement2d_id;
        let mut builder = Displacement2DBuilder::new(id);
        f(&mut builder);
        let displacement = builder.build()?;
        self.resources.displacement2d.push(displacement);
        self.next_displacement2d_id += 1;
        Ok(id)
    }

    /// Add a norm vector group resource.
    pub fn add_norm_vector_group<F>(&mut self, f: F) -> ResourceId
    where
        F: FnOnce(&mut NormVectorGroupBuilder),
    {
        let id = self.next_normvectorgroup_id;
        let mut builder = NormVectorGroupBuilder::new(id);
        f(&mut builder);
        self.resources.normvectorgroup.push(builder.build());
        self.next_normvectorgroup_id += 1;
        id
    }

    /// Add a displacement 2d group resource.
    pub fn add_disp2d_group<F>(&mut self, f: F) -> Result<ResourceId, Disp2DGroupError>
    where
        F: FnOnce(&mut Disp2DGroupBuilder),
    {
        let id = self.next_disp2dgroup_id;
        let mut builder = Disp2DGroupBuilder::new(id);
        f(&mut builder);
        let group = builder.build()?;
        self.resources.disp2dgroup.push(group);
        self.next_disp2dgroup_id += 1;
        Ok(id)
    }

    /// Build the final [`Model`].
    ///
    /// This consumes the builder and performs final validation:
    /// - Root models must have a Build section
    /// - Sub-models must not have a Build section
    /// - All required extensions are added automatically
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] if validation fails.
    pub fn build(self) -> Result<Model, ModelError> {
        let required_extensions = self.process_required_extensions();

        let requiredextensions = ThreemfExtensions::new(&required_extensions);
        let recommendedextensions = ThreemfExtensions::new(&self.recommendedextensions);

        if self.is_root && self.build.is_none() {
            return Err(ModelError::BuildItemNotSet);
        }

        if !self.is_root && self.build.is_some() {
            return Err(ModelError::BuildOnlyAllowedInRootModel);
        }

        let build = if let Some(builder) = self.build {
            builder.build(self.is_production_ext_required)?
        } else {
            Build {
                uuid: None,
                item: vec![],
            }
        };

        Ok(Model {
            unit: self.unit,
            requiredextensions,
            recommendedextensions,
            metadata: self.metadata,
            resources: self.resources.build(),
            build,
        })
    }

    fn set_recommended_namespaces_for_mesh(&mut self, mesh: &mesh::Mesh) {
        self.set_recommended_namespaces_for_triangle_sets(mesh.trianglesets.is_some());
    }

    fn set_recommended_namespaces_for_triangle_sets(&mut self, has_triangle_sets: bool) {
        if has_triangle_sets
            && !self
                .recommendedextensions
                .contains(&ThreemfNamespace::CoreTriangleSet)
        {
            self.recommendedextensions
                .push(ThreemfNamespace::CoreTriangleSet);
        }
    }

    fn process_required_extensions(&self) -> Vec<ThreemfNamespace> {
        let mut required_extensions = self.requiredextensions.clone();
        if self.is_production_ext_required {
            let is_prod_ext_set = required_extensions
                .iter()
                .find(|ns| **ns == ThreemfNamespace::Prod);
            if is_prod_ext_set.is_none() {
                required_extensions.push(ThreemfNamespace::Prod);
            }
        }

        let mut is_beam_lattice_required = false;
        let mut is_beam_lattice_balls_required = false;

        for object in &self.resources.objects {
            if let Some(mesh) = object.get_mesh()
                && let Some(beam_lattice) = &mesh.beamlattice
            {
                is_beam_lattice_required = true;
                is_beam_lattice_balls_required = beam_lattice.balls.is_some();
            }
            if let Some(mesh) = object.get_displacement_mesh()
                && let Some(beam_lattice) = &mesh.beamlattice
            {
                is_beam_lattice_required = true;
                is_beam_lattice_balls_required = beam_lattice.balls.is_some();
            }
        }

        if is_beam_lattice_required {
            let is_bl_ext_set = required_extensions
                .iter()
                .find(|ns| **ns == ThreemfNamespace::BeamLattice);
            if is_bl_ext_set.is_none() {
                required_extensions.push(ThreemfNamespace::BeamLattice);
            }

            if is_beam_lattice_balls_required {
                let is_bl_balls_ext_set = required_extensions
                    .iter()
                    .find(|ns| **ns == ThreemfNamespace::BeamLatticeBalls);
                if is_bl_balls_ext_set.is_none() {
                    required_extensions.push(ThreemfNamespace::BeamLatticeBalls);
                }
            }
        }

        // Detect boolean operations extension
        let is_boolean_required = self
            .resources
            .objects
            .iter()
            .any(|obj| obj.get_boolean_shape_object().is_some());

        if is_boolean_required {
            let is_boolean_ext_set = required_extensions
                .iter()
                .find(|ns| **ns == ThreemfNamespace::Boolean);
            if is_boolean_ext_set.is_none() {
                required_extensions.push(ThreemfNamespace::Boolean);
            }
        }

        // Detect slice extension
        let is_slice_required = !self.resources.slicestack.is_empty()
            || self.resources.objects.iter().any(|obj| {
                obj.slicestackid.is_some()
                    || obj.slicepath.is_some()
                    || matches!(obj.meshresolution, Some(slice::MeshResolution::LowRes))
            });

        if is_slice_required {
            let is_slice_ext_set = required_extensions
                .iter()
                .find(|ns| **ns == ThreemfNamespace::Slice);
            if is_slice_ext_set.is_none() {
                required_extensions.push(ThreemfNamespace::Slice);
            }
        }

        let has_material_resources = !self.resources.basematerials.is_empty()
            || !self.resources.colorgroup.is_empty()
            || !self.resources.texture2dgroup.is_empty()
            || !self.resources.texture2d.is_empty()
            || !self.resources.compositematerials.is_empty()
            || !self.resources.multiproperties.is_empty();

        let has_material_references =
            self.resources.objects.iter().any(|obj| {
                if obj.pid.is_some() || obj.pindex.is_some() {
                    return true;
                }

                if let Some(mesh) = obj.get_mesh() {
                    if mesh.triangles.triangle.iter().any(|t| {
                        t.pid.is_some() || t.p1.is_some() || t.p2.is_some() || t.p3.is_some()
                    }) {
                        return true;
                    }

                    if let Some(lattice) = &mesh.beamlattice {
                        if lattice.pid.is_some() || lattice.pindex.is_some() {
                            return true;
                        }

                        if lattice.beams.beam.iter().any(|beam| {
                            beam.pid.is_some() || beam.p1.is_some() || beam.p2.is_some()
                        }) {
                            return true;
                        }

                        if let Some(balls) = &lattice.balls
                            && balls
                                .ball
                                .iter()
                                .any(|ball| ball.pid.is_some() || ball.p.is_some())
                        {
                            return true;
                        }
                    }
                }

                if let Some(mesh) = obj.get_displacement_mesh() {
                    if mesh.triangles.triangle.iter().any(|t| {
                        t.pid.is_some() || t.p1.is_some() || t.p2.is_some() || t.p3.is_some()
                    }) {
                        return true;
                    }

                    if let Some(lattice) = &mesh.beamlattice {
                        if lattice.pid.is_some() || lattice.pindex.is_some() {
                            return true;
                        }

                        if lattice.beams.beam.iter().any(|beam| {
                            beam.pid.is_some() || beam.p1.is_some() || beam.p2.is_some()
                        }) {
                            return true;
                        }

                        if let Some(balls) = &lattice.balls
                            && balls
                                .ball
                                .iter()
                                .any(|ball| ball.pid.is_some() || ball.p.is_some())
                        {
                            return true;
                        }
                    }
                }

                false
            }) || self.resources.slicestack.iter().any(|stack| {
                stack.slice.iter().any(|slice| {
                    slice.polygon.iter().any(|polygon| {
                        polygon
                            .segment
                            .iter()
                            .any(|seg| seg.pid.is_some() || seg.p1.is_some() || seg.p2.is_some())
                    })
                })
            });

        if has_material_resources || has_material_references {
            let is_material_ext_set = required_extensions
                .iter()
                .find(|ns| **ns == ThreemfNamespace::Material);
            if is_material_ext_set.is_none() {
                required_extensions.push(ThreemfNamespace::Material);
            }
        }

        let has_displacement_resources = !self.resources.displacement2d.is_empty()
            || !self.resources.normvectorgroup.is_empty()
            || !self.resources.disp2dgroup.is_empty();

        let has_displacement_mesh = self
            .resources
            .objects
            .iter()
            .any(|obj| obj.get_displacement_mesh().is_some());

        if has_displacement_resources || has_displacement_mesh {
            let is_displacement_ext_set = required_extensions
                .iter()
                .find(|ns| **ns == ThreemfNamespace::Displacement);
            if is_displacement_ext_set.is_none() {
                required_extensions.push(ThreemfNamespace::Displacement);
            }
        }

        required_extensions
    }
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new(Unit::Millimeter, true)
    }
}

/// Builder for Resources
pub struct ResourcesBuilder {
    objects: Vec<object::Object>,
    slicestack: Vec<slice::SliceStack>,
    colorgroup: Vec<material::ColorGroup>,
    texture2dgroup: Vec<material::Texture2DGroup>,
    texture2d: Vec<material::Texture2D>,
    compositematerials: Vec<material::CompositeMaterials>,
    multiproperties: Vec<material::MultiProperties>,
    basematerials: Vec<BaseMaterials>,
    displacement2d: Vec<displacement::Displacement2D>,
    normvectorgroup: Vec<displacement::NormVectorGroup>,
    disp2dgroup: Vec<displacement::Disp2DGroup>,
}

impl ResourcesBuilder {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            texture2d: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            basematerials: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        }
    }

    fn build(self) -> Resources {
        Resources {
            object: self.objects,
            basematerials: self.basematerials,
            slicestack: self.slicestack,
            colorgroup: self.colorgroup,
            texture2dgroup: self.texture2dgroup,
            compositematerials: self.compositematerials,
            multiproperties: self.multiproperties,
            texture2d: self.texture2d,
            displacement2d: self.displacement2d,
            normvectorgroup: self.normvectorgroup,
            disp2dgroup: self.disp2dgroup,
        }
    }
}

/// Errors that can occur when building the Build section.
#[derive(Debug, Error, Clone, Copy)]
pub enum BuildError {
    /// Build is missing a UUID when Production extension is required.
    ///
    /// Pass a UUID when calling [`ModelBuilder::add_build()`].
    #[error("Production extension is enabled but Uuid for the Build is not set")]
    BuildUuidNotSet,
}

/// Builder for the Build section of a 3MF model.
///
/// The Build section specifies which objects should be manufactured. It is only
/// used in root models (not sub-models). This builder is primarily used internally
/// by [`ModelBuilder`].
pub struct BuildBuilder {
    items: Vec<Item>,
    uuid: Option<UuidResource>,
}

impl BuildBuilder {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            uuid: None,
        }
    }

    fn uuid(&mut self, uuid: UuidResource) -> &mut Self {
        self.uuid = Some(uuid);

        self
    }

    fn add_build_item<F>(
        &mut self,
        objectid: ObjectId,
        is_production_ext_enabled: bool,
        f: F,
    ) -> Result<&mut Self, ItemError>
    where
        F: FnOnce(&mut ItemBuilder),
    {
        let mut builder = ItemBuilder::new(objectid);
        f(&mut builder);

        let item = builder.build(is_production_ext_enabled)?;
        self.items.push(item);
        Ok(self)
    }

    fn can_build(&self, is_production_ext_enabled: bool) -> Result<(), BuildError> {
        if is_production_ext_enabled && self.uuid.is_none() {
            return Err(BuildError::BuildUuidNotSet);
        }

        Ok(())
    }

    fn build(self, is_production_ext_required: bool) -> Result<Build, BuildError> {
        self.can_build(is_production_ext_required)?;

        Ok(Build {
            uuid: self.uuid,
            item: self.items,
        })
    }
}

/// Errors that can occur when building a build item.
#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum ItemError {
    /// Build item has a path set but Production extension is not enabled.
    ///
    /// Call [`ModelBuilder::make_production_extension_required()`] before setting paths.
    #[error("Item path is set without the Production extension enabled!")]
    ItemPathSetWithoutProductionExtension,

    /// Build item is missing a UUID when Production extension is required.
    ///
    /// Use [`ModelBuilder::add_build_item_advanced()`] with [`ItemBuilder::uuid()`].
    #[error("Production extension is enabled but Uuid is not set!")]
    ItemUuidNotSet,
}

/// Errors that can occur when building a Texture2D resource.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Texture2DError {
    #[error("Texture2D path is not set")]
    PathNotSet,
    #[error("Texture2D content type is not set")]
    ContentTypeNotSet,
}

/// Errors that can occur when building a Texture2DGroup resource.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Texture2DGroupError {
    #[error("Texture2DGroup texid is not set")]
    TexIdNotSet,
}

/// Errors that can occur when building CompositeMaterials.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum CompositeMaterialsError {
    #[error("CompositeMaterials matid is not set")]
    MatIdNotSet,
    #[error("CompositeMaterials matindices are empty")]
    MatIndicesEmpty,
}

/// Errors that can occur when building MultiProperties.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MultiPropertiesError {
    #[error("MultiProperties pids are empty")]
    PidsEmpty,
}

/// Errors that can occur when building Displacement2D.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Displacement2DError {
    #[error("Displacement2D path is not set")]
    PathNotSet,
}

/// Errors that can occur when building Disp2DGroup.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum Disp2DGroupError {
    #[error("Disp2DGroup displacement id is not set")]
    DispIdNotSet,
    #[error("Disp2DGroup norm vector group id is not set")]
    NormVectorGroupIdNotSet,
    #[error("Disp2DGroup height is not set")]
    HeightNotSet,
}

/// Builder for configuring a build item.
///
/// Build items specify which objects should be manufactured and how they should
/// be positioned on the build plate. Each item references an object and can have
/// a transform, partnumber, UUID, and path.
///
/// This builder is used via [`ModelBuilder::add_build_item_advanced()`].
pub struct ItemBuilder {
    objectid: ObjectId,
    transform: Option<Transform>,
    partnumber: Option<String>,
    path: Option<PathResource>,
    uuid: Option<UuidResource>,
}

impl ItemBuilder {
    fn new(objectid: ObjectId) -> Self {
        Self {
            objectid,
            transform: None,
            partnumber: None,
            path: None,
            uuid: None,
        }
    }

    /// Set the transformation matrix for this build item.
    ///
    /// The transform is a 4x3 affine transformation matrix stored as a 12-element array
    /// in row-major order.
    pub fn transform(&mut self, transform: Transform) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// Set the part number for this build item.
    ///
    /// Part numbers can be used to identify specific manufacturing runs or variants.
    pub fn partnumber(&mut self, partnumber: &str) -> &mut Self {
        self.partnumber = Some(partnumber.to_owned());
        self
    }

    /// Set the UUID for this build item.
    ///
    /// Required when Production extension is enabled.
    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = Some(UuidResource::from(uuid));
        self
    }

    /// Set the path for this build item.
    ///
    /// Only allowed when Production extension is enabled. The path specifies
    /// an alternative model file where the referenced object can be found.
    pub fn path(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.path = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    fn build(self, is_production_ext_enabled: bool) -> Result<Item, ItemError> {
        if !is_production_ext_enabled && self.path.is_some() {
            return Err(ItemError::ItemPathSetWithoutProductionExtension);
        } else if is_production_ext_enabled && self.uuid.is_none() {
            return Err(ItemError::ItemUuidNotSet);
        }
        Ok(Item {
            objectid: self.objectid.0,
            transform: self.transform,
            partnumber: self.partnumber.map(Into::into),
            path: self.path,
            uuid: self.uuid,
        })
    }
}

/// Type-safe wrapper for object IDs to prevent mix-ups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub ResourceId);

impl From<ResourceId> for ObjectId {
    fn from(id: ResourceId) -> Self {
        ObjectId(id)
    }
}

impl From<ObjectId> for ResourceId {
    fn from(id: ObjectId) -> ResourceId {
        id.0
    }
}

/// Type-safe wrapper for SliceStack Ids to prevent mix-ups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliceStackId(ResourceId);

impl From<ResourceId> for SliceStackId {
    fn from(id: ResourceId) -> Self {
        SliceStackId(id)
    }
}

impl From<SliceStackId> for ResourceId {
    fn from(id: SliceStackId) -> ResourceId {
        id.0
    }
}

/// Builder for Object
pub struct ObjectBuilder<T> {
    entity: T,
    object_id: ObjectId,
    objecttype: Option<object::ObjectType>,
    thumbnail: Option<PathResource>,
    partnumber: Option<String>,
    name: Option<String>,
    pid: OptionalResourceId,
    pindex: OptionalResourceIndex,
    uuid: Option<UuidResource>,
    slicestackid: OptionalResourceId,
    slicepath: Option<PathResource>,
    meshresolution: Option<slice::MeshResolution>,

    // sets if the production ext is required.
    // if yes will ensure UUID is set before building the object
    is_production_ext_required: bool,
}

impl<T> ObjectBuilder<T> {
    /// Set the object type
    pub fn object_type(&mut self, object_type: object::ObjectType) -> &mut Self {
        self.objecttype = Some(object_type);
        self
    }

    /// Set a thumbnail path for this object.
    pub fn thumbnail(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.thumbnail = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    /// Set the object name
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set the part number
    pub fn part_number(&mut self, part_number: &str) -> &mut Self {
        self.partnumber = Some(part_number.to_owned());
        self
    }

    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = Some(UuidResource::from(uuid));
        self
    }

    /// Set the property resource ID for this object.
    pub fn pid(&mut self, pid: ResourceId) -> &mut Self {
        self.pid = OptionalResourceId::new(pid);
        self
    }

    /// Set the property index for this object.
    pub fn pindex(&mut self, pindex: ResourceIndex) -> &mut Self {
        self.pindex = OptionalResourceIndex::new(pindex);
        self
    }

    /// Set the slice stack reference for this object.
    pub fn slice_stack_id(&mut self, slicestack_id: SliceStackId) -> &mut Self {
        self.slicestackid = OptionalResourceId::new(slicestack_id.0);
        self
    }

    /// Set the slice path for this object.
    pub fn slicepath(&mut self, slicepath: &str) -> &mut Self {
        match PathResource::try_from(slicepath) {
            Ok(path) => {
                self.slicepath = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    /// Set the mesh resolution for this object when slice data is present.
    pub fn meshresolution(&mut self, resolution: slice::MeshResolution) -> &mut Self {
        self.meshresolution = Some(resolution);
        self
    }

    /// Configure slice stack attributes together.
    pub fn slice_stack(
        &mut self,
        slicestack_id: SliceStackId,
        slicepath: Option<&str>,
        meshresolution: Option<slice::MeshResolution>,
    ) -> &mut Self {
        self.slice_stack_id(slicestack_id);
        if let Some(path) = slicepath {
            self.slicepath(path);
        }
        if let Some(resolution) = meshresolution {
            self.meshresolution(resolution);
        }
        self
    }
}

impl<T> Deref for ObjectBuilder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T> DerefMut for ObjectBuilder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

/// Errors that can occur when building a mesh object.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum MeshObjectError {
    /// Object is missing a UUID when Production extension is required.
    ///
    /// Call [`MeshObjectBuilder::uuid()`] to set the UUID.
    #[error("Production extension is enabled but Uuid is not set!")]
    ObjectUuidNotSet,
}

/// Errors that can occur when building a displacement mesh object.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DisplacementMeshObjectError {
    /// Object is missing a UUID when Production extension is required.
    ///
    /// Call [`DisplacementMeshObjectBuilder::uuid()`] to set the UUID.
    #[error("Production extension is enabled but Uuid is not set!")]
    ObjectUuidNotSet,
}

/// Builder for creating mesh objects with triangle geometry.
///
/// `MeshObjectBuilder` combines object metadata (name, type, UUID, etc.) with
/// mesh geometry. Access mesh-building methods directly via [`Deref`] to [`MeshBuilder`].
pub type MeshObjectBuilder = ObjectBuilder<MeshBuilder>;

impl MeshObjectBuilder {
    fn new(object_id: ObjectId, is_production_ext_required: bool) -> Self {
        Self {
            entity: MeshBuilder::new(),
            object_id,
            objecttype: Some(object::ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<object::Object, MeshObjectError> {
        let mesh = self.entity.build_mesh().unwrap();

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(MeshObjectError::ObjectUuidNotSet);
        }

        Ok(object::Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber.map(Into::into),
            name: self.name.map(Into::into),
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            slicestackid: self.slicestackid,
            slicepath: self.slicepath,
            meshresolution: self.meshresolution,
            kind: Some(object::ObjectKind::Mesh(mesh)),
            // mesh: Some(mesh),
        })
    }
}

/// Builder for creating displacement mesh objects.
pub type DisplacementMeshObjectBuilder = ObjectBuilder<DisplacementMeshBuilder>;

impl DisplacementMeshObjectBuilder {
    fn new(object_id: ObjectId, is_production_ext_required: bool) -> Self {
        Self {
            entity: DisplacementMeshBuilder::new(),
            object_id,
            objecttype: Some(object::ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<object::Object, DisplacementMeshObjectError> {
        let mesh = self.entity.build_mesh();

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(DisplacementMeshObjectError::ObjectUuidNotSet);
        }

        Ok(object::Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber.map(Into::into),
            name: self.name.map(Into::into),
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            slicestackid: self.slicestackid,
            slicepath: self.slicepath,
            meshresolution: self.meshresolution,
            kind: Some(object::ObjectKind::DisplacementMesh(mesh)),
        })
    }
}

/// Builder for constructing triangle mesh geometry.
///
/// `MeshBuilder` allows you to define 3D geometry by adding vertices and triangles.
/// It also supports optional features like triangle sets and beam lattices.
///
/// Vertices are referenced by their 0-based index in the order they were added.
/// Triangles reference vertices by index.
pub struct MeshBuilder {
    vertices: Vec<mesh::Vertex>,
    triangles: Vec<mesh::Triangle>,
    triangle_sets: Option<TriangleSetsBuilder>,
    beam_lattice: Option<BeamLatticeBuilder>,
}

impl MeshBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            triangle_sets: None,
            beam_lattice: None,
        }
    }

    /// Add a single vertex at the specified coordinates.
    ///
    /// Returns the builder for method chaining.
    ///
    /// # Parameters
    ///
    /// - `coords`: 3D coordinates as `[x, y, z]`
    pub fn add_vertex(&mut self, coords: &[f64; 3]) -> &mut Self {
        self.vertices
            .push(mesh::Vertex::new(coords[0], coords[1], coords[2]));
        self
    }

    /// Add multiple vertices from a slice of coordinate arrays.
    ///
    /// Each element should be a 3D coordinate `[x, y, z]`.
    pub fn add_vertices(&mut self, vertices: &[[f64; 3]]) -> &mut Self {
        for vertex in vertices {
            self.add_vertex(vertex);
        }

        self
    }

    /// Add vertices from a flattened coordinate array.
    ///
    /// The slice should contain coordinate values in sequence: `[x0, y0, z0, x1, y1, z1, ...]`.
    /// The length must be a multiple of 3.
    pub fn add_vertices_flat(&mut self, vertices: &[f64]) -> &mut Self {
        for vertex in vertices.chunks_exact(3) {
            self.vertices
                .push(mesh::Vertex::new(vertex[0], vertex[1], vertex[2]));
        }

        self
    }

    /// Add a single triangle referencing three vertices by index.
    ///
    /// Vertices are referenced by their 0-based index in the order they were added.
    /// Indices must reference existing vertices.
    ///
    /// # Parameters
    ///
    /// - `indices`: Triangle vertex indices as `[v1, v2, v3]`
    pub fn add_triangle(&mut self, indices: &[usize; 3]) -> &mut Self {
        self.triangles.push(mesh::Triangle {
            v1: indices[0] as ResourceIndex,
            v2: indices[1] as ResourceIndex,
            v3: indices[2] as ResourceIndex,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });
        self
    }

    /// Add a triangle with explicit property references.
    pub fn add_triangle_with_properties(
        &mut self,
        indices: &[usize; 3],
        p1: OptionalResourceIndex,
        p2: OptionalResourceIndex,
        p3: OptionalResourceIndex,
        pid: OptionalResourceId,
    ) -> &mut Self {
        self.triangles.push(mesh::Triangle {
            v1: indices[0] as ResourceIndex,
            v2: indices[1] as ResourceIndex,
            v3: indices[2] as ResourceIndex,
            p1,
            p2,
            p3,
            pid,
        });
        self
    }

    /// Add a triangle with advanced configuration.
    pub fn add_triangle_advanced<F>(&mut self, indices: &[usize; 3], f: F) -> &mut Self
    where
        F: FnOnce(TriangleBuilder) -> TriangleBuilder,
    {
        let builder = TriangleBuilder::new(
            indices[0] as ResourceIndex,
            indices[1] as ResourceIndex,
            indices[2] as ResourceIndex,
        );
        self.triangles.push(f(builder).build());
        self
    }

    /// Add multiple triangles from a slice of index arrays.
    ///
    /// Each element should be a triangle with three vertex indices.
    pub fn add_triangles(&mut self, triangles: &[[usize; 3]]) -> &mut Self {
        for triangle in triangles {
            self.add_triangle(triangle);
        }

        self
    }

    /// Add triangles from a flattened index array.
    ///
    /// The slice should contain vertex indices in sequence: `[v1, v2, v3, v1, v2, v3, ...]`.
    /// The length must be a multiple of 3.
    pub fn add_triangles_flat(&mut self, triangles: &[usize]) -> &mut Self {
        for triangle in triangles.chunks_exact(3) {
            self.triangles.push(mesh::Triangle {
                v1: triangle[0] as ResourceIndex,
                v2: triangle[1] as ResourceIndex,
                v3: triangle[2] as ResourceIndex,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            });
        }

        self
    }

    /// Add triangle sets to organize triangles into named groups.
    ///
    /// Triangle sets allow you to group triangles by name and identifier for
    /// organizational purposes. See [`TriangleSetsBuilder`] for details.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`TriangleSetsBuilder`]
    pub fn add_triangle_sets<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut TriangleSetsBuilder),
    {
        if let Some(ref mut builder) = self.triangle_sets {
            f(builder);
        } else {
            let mut builder = TriangleSetsBuilder::new();
            f(&mut builder);
            self.triangle_sets = Some(builder);
        }

        self
    }

    /// Add a beam lattice structure to the mesh.
    ///
    /// Beam lattices define strut-like structures connecting vertices. See
    /// [`BeamLatticeBuilder`] for details.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`BeamLatticeBuilder`]
    pub fn add_beam_lattice<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut BeamLatticeBuilder),
    {
        if let Some(builder) = &mut self.beam_lattice {
            f(builder);
        } else {
            let mut builder = BeamLatticeBuilder::new();
            f(&mut builder);
            self.beam_lattice = Some(builder);
        }
        self
    }

    fn build_mesh(self) -> Result<mesh::Mesh, MeshObjectError> {
        let trianglesets = self.triangle_sets.map(|b| b.build());
        let beamlattice = self.beam_lattice.map(|b| b.build());
        Ok(mesh::Mesh {
            vertices: mesh::Vertices {
                vertex: self.vertices,
            },
            triangles: mesh::Triangles {
                triangle: self.triangles,
            },
            trianglesets,
            beamlattice,
        })
    }
}

/// Builder for triangle properties in a mesh.
pub struct TriangleBuilder {
    v1: ResourceIndex,
    v2: ResourceIndex,
    v3: ResourceIndex,
    p1: OptionalResourceIndex,
    p2: OptionalResourceIndex,
    p3: OptionalResourceIndex,
    pid: OptionalResourceId,
}

impl TriangleBuilder {
    fn new(v1: ResourceIndex, v2: ResourceIndex, v3: ResourceIndex) -> Self {
        Self {
            v1,
            v2,
            v3,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        }
    }

    pub fn pindex_1(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p1 = pindex;
        self
    }

    pub fn pindex_2(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p2 = pindex;
        self
    }

    pub fn pindex_3(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p3 = pindex;
        self
    }

    pub fn pid(mut self, pid: ResourceId) -> Self {
        self.pid = OptionalResourceId::new(pid);
        self
    }

    fn build(self) -> mesh::Triangle {
        mesh::Triangle {
            v1: self.v1,
            v2: self.v2,
            v3: self.v3,
            p1: self.p1,
            p2: self.p2,
            p3: self.p3,
            pid: self.pid,
        }
    }
}

/// Builder for constructing displacement mesh geometry.
pub struct DisplacementMeshBuilder {
    vertices: Vec<displacement::Vertex>,
    triangles: Vec<displacement::Triangle>,
    triangle_sets: Option<TriangleSetsBuilder>,
    beam_lattice: Option<BeamLatticeBuilder>,
    triangles_did: OptionalResourceId,
}

impl DisplacementMeshBuilder {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            triangle_sets: None,
            beam_lattice: None,
            triangles_did: OptionalResourceId::none(),
        }
    }

    /// Set the default displacement group ID for triangles.
    pub fn displacement_id(&mut self, disp2dgroup_id: ResourceId) -> &mut Self {
        self.triangles_did = OptionalResourceId::new(disp2dgroup_id);
        self
    }

    pub fn add_vertex(&mut self, coords: &[f64; 3]) -> &mut Self {
        self.vertices
            .push(crate::model::domain::displacement::Vertex {
                x: coords[0].into(),
                y: coords[1].into(),
                z: coords[2].into(),
            });
        self
    }

    pub fn add_vertices(&mut self, vertices: &[[f64; 3]]) -> &mut Self {
        for vertex in vertices {
            self.add_vertex(vertex);
        }
        self
    }

    pub fn add_vertices_flat(&mut self, vertices: &[f64]) -> &mut Self {
        for vertex in vertices.chunks_exact(3) {
            self.vertices
                .push(crate::model::domain::displacement::Vertex {
                    x: vertex[0].into(),
                    y: vertex[1].into(),
                    z: vertex[2].into(),
                });
        }
        self
    }

    pub fn add_triangle(&mut self, indices: &[usize; 3]) -> &mut Self {
        self.triangles
            .push(crate::model::domain::displacement::Triangle {
                v1: indices[0] as ResourceIndex,
                v2: indices[1] as ResourceIndex,
                v3: indices[2] as ResourceIndex,
                d1: OptionalResourceIndex::none(),
                d2: OptionalResourceIndex::none(),
                d3: OptionalResourceIndex::none(),
                did: OptionalResourceId::none(),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            });
        self
    }

    pub fn add_triangle_with_properties(
        &mut self,
        indices: &[usize; 3],
        p1: OptionalResourceIndex,
        p2: OptionalResourceIndex,
        p3: OptionalResourceIndex,
        pid: OptionalResourceId,
    ) -> &mut Self {
        self.triangles
            .push(crate::model::domain::displacement::Triangle {
                v1: indices[0] as ResourceIndex,
                v2: indices[1] as ResourceIndex,
                v3: indices[2] as ResourceIndex,
                d1: OptionalResourceIndex::none(),
                d2: OptionalResourceIndex::none(),
                d3: OptionalResourceIndex::none(),
                did: OptionalResourceId::none(),
                p1,
                p2,
                p3,
                pid,
            });
        self
    }

    pub fn add_triangle_advanced<F>(&mut self, indices: &[usize; 3], f: F) -> &mut Self
    where
        F: FnOnce(DisplacementTriangleBuilder) -> DisplacementTriangleBuilder,
    {
        let builder = DisplacementTriangleBuilder::new(
            indices[0] as ResourceIndex,
            indices[1] as ResourceIndex,
            indices[2] as ResourceIndex,
        );
        self.triangles.push(f(builder).build());
        self
    }

    pub fn add_triangles(&mut self, triangles: &[[usize; 3]]) -> &mut Self {
        for triangle in triangles {
            self.add_triangle(triangle);
        }
        self
    }

    pub fn add_triangles_flat(&mut self, triangles: &[usize]) -> &mut Self {
        for triangle in triangles.chunks_exact(3) {
            self.triangles
                .push(crate::model::domain::displacement::Triangle {
                    v1: triangle[0] as ResourceIndex,
                    v2: triangle[1] as ResourceIndex,
                    v3: triangle[2] as ResourceIndex,
                    d1: OptionalResourceIndex::none(),
                    d2: OptionalResourceIndex::none(),
                    d3: OptionalResourceIndex::none(),
                    did: OptionalResourceId::none(),
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                });
        }
        self
    }

    pub fn add_triangle_sets<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut TriangleSetsBuilder),
    {
        if let Some(ref mut builder) = self.triangle_sets {
            f(builder);
        } else {
            let mut builder = TriangleSetsBuilder::new();
            f(&mut builder);
            self.triangle_sets = Some(builder);
        }
        self
    }

    pub fn add_beam_lattice<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut BeamLatticeBuilder),
    {
        if let Some(builder) = &mut self.beam_lattice {
            f(builder);
        } else {
            let mut builder = BeamLatticeBuilder::new();
            f(&mut builder);
            self.beam_lattice = Some(builder);
        }
        self
    }

    fn build_mesh(self) -> displacement::DisplacementMesh {
        let trianglesets = self.triangle_sets.map(|b| b.build());
        let beamlattice = self.beam_lattice.map(|b| b.build());
        displacement::DisplacementMesh {
            vertices: crate::model::domain::displacement::Vertices {
                vertex: self.vertices,
            },
            triangles: crate::model::domain::displacement::Triangles {
                did: self.triangles_did,
                triangle: self.triangles,
            },
            trianglesets,
            beamlattice,
        }
    }
}

/// Builder for displacement mesh triangles.
pub struct DisplacementTriangleBuilder {
    v1: ResourceIndex,
    v2: ResourceIndex,
    v3: ResourceIndex,
    d1: OptionalResourceIndex,
    d2: OptionalResourceIndex,
    d3: OptionalResourceIndex,
    did: OptionalResourceId,
    p1: OptionalResourceIndex,
    p2: OptionalResourceIndex,
    p3: OptionalResourceIndex,
    pid: OptionalResourceId,
}

impl DisplacementTriangleBuilder {
    fn new(v1: ResourceIndex, v2: ResourceIndex, v3: ResourceIndex) -> Self {
        Self {
            v1,
            v2,
            v3,
            d1: OptionalResourceIndex::none(),
            d2: OptionalResourceIndex::none(),
            d3: OptionalResourceIndex::none(),
            did: OptionalResourceId::none(),
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            p3: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        }
    }

    pub fn displacement_index_1(mut self, index: OptionalResourceIndex) -> Self {
        self.d1 = index;
        self
    }

    pub fn displacement_index_2(mut self, index: OptionalResourceIndex) -> Self {
        self.d2 = index;
        self
    }

    pub fn displacement_index_3(mut self, index: OptionalResourceIndex) -> Self {
        self.d3 = index;
        self
    }

    pub fn displacement_id(mut self, disp2dgroup_id: ResourceId) -> Self {
        self.did = OptionalResourceId::new(disp2dgroup_id);
        self
    }

    pub fn pindex_1(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p1 = pindex;
        self
    }

    pub fn pindex_2(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p2 = pindex;
        self
    }

    pub fn pindex_3(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p3 = pindex;
        self
    }

    pub fn pid(mut self, pid: ResourceId) -> Self {
        self.pid = OptionalResourceId::new(pid);
        self
    }

    fn build(self) -> crate::model::domain::displacement::Triangle {
        crate::model::domain::displacement::Triangle {
            v1: self.v1,
            v2: self.v2,
            v3: self.v3,
            d1: self.d1,
            d2: self.d2,
            d3: self.d3,
            did: self.did,
            p1: self.p1,
            p2: self.p2,
            p3: self.p3,
            pid: self.pid,
        }
    }
}

/// Errors that can occur when building a components (assembly) object.
#[derive(Debug, Error, Clone)]
pub enum ComponentsObjectError {
    /// Component is missing a UUID when Production extension is required.
    ///
    /// Call [`ComponentBuilder::uuid()`] when configuring components.
    #[error("production extension is enabled but uuid is not set")]
    ComponentUuidNotSet,

    /// Component has a path set but Production extension is not enabled.
    ///
    /// Call [`ModelBuilder::make_production_extension_required()`] before setting paths.
    #[error("Path is set for a Component without enabling the Production extension")]
    PathSetWithoutProductionExtension,

    /// Component references an object ID that doesn't exist.
    ///
    /// Ensure the referenced object has been added to the model before creating
    /// components that reference it.
    #[error("One or more Component References unknown objects")]
    ObjectReferenceNotFoundForComponent,

    /// Object is missing a UUID when Production extension is required.
    ///
    /// Call [`ComponentsObjectBuilder::uuid()`] to set the UUID.
    #[error("Production extension is enabled but Uuid is not set")]
    ObjectUuidNotSet,
}

/// Builder for creating assembly objects that reference other objects.
///
/// `ComponentsObjectBuilder` creates objects that don't have their own geometry,
/// but instead reference other objects (mesh objects or other assemblies) as components.
/// Each component can have its own transform.
///
/// This is useful for:
/// - Creating assemblies of multiple parts
/// - Building hierarchical model structures
///
/// Access component-building methods directly via [`Deref`] to [`ComponentsBuilder`].
///
/// # Examples
///
/// ```rust,ignore
/// // First create a mesh object to reference
/// let bolt_id = builder.add_mesh_object(|obj| {
///     obj.name("Bolt");
///     // ... add geometry
///     Ok(())
/// })?;
///
/// // Create an assembly referencing the bolt multiple times
/// let assembly_id = builder.add_components_object(|obj| {
///     obj.name("BoltAssembly");
///
///     // Add first bolt
///     obj.add_component(bolt_id);
///
///     // Add second bolt with transform
///     obj.add_component_advanced(bolt_id, |comp| {
///         comp.transform(Transform([
///             1.0, 0.0, 0.0,
///             0.0, 1.0, 0.0,
///             0.0, 0.0, 1.0,
///             10.0, 0.0, 0.0  // Translated 10mm in X
///         ]));
///     });
///
///     Ok(())
/// })?;
/// ```
pub type ComponentsObjectBuilder = ObjectBuilder<ComponentsBuilder>;

impl ComponentsObjectBuilder {
    fn new(
        object_id: ObjectId,
        all_existing_object_ids: &[ObjectId],
        is_production_ext_required: bool,
    ) -> Self {
        Self {
            entity: ComponentsBuilder::new(all_existing_object_ids),
            object_id,
            objecttype: Some(object::ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<object::Object, ComponentsObjectError> {
        let components = self
            .entity
            .build_components(self.is_production_ext_required)?;

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(ComponentsObjectError::ObjectUuidNotSet);
        }

        Ok(object::Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber.map(Into::into),
            name: self.name.map(Into::into),
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            slicestackid: self.slicestackid,
            slicepath: self.slicepath,
            meshresolution: self.meshresolution,
            kind: Some(object::ObjectKind::Components(components)),
            // components: Some(components),
        })
    }
}

/// Builder for managing components in an assembly object.
///
/// Components are references to other objects with optional transforms, UUIDs, and paths.
/// The builder validates that referenced objects exist in the model.
pub struct ComponentsBuilder {
    components: Vec<Component>,

    all_existing_object_ids: Vec<ObjectId>,
}

impl ComponentsBuilder {
    fn new(all_existing_object_ids: &[ObjectId]) -> Self {
        ComponentsBuilder {
            components: vec![],
            all_existing_object_ids: all_existing_object_ids.to_vec(),
        }
    }

    /// Add a simple component referencing an object by ID.
    ///
    /// The component will use the identity transform and have no UUID or path
    /// (unless required by Production extension).
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the object to reference
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_component(part_id);
    /// ```
    pub fn add_component(&mut self, object_id: ObjectId) -> &mut Self {
        self.add_component_advanced(object_id, |_| {});
        self
    }

    /// Add a component with advanced configuration.
    ///
    /// This method allows you to configure transforms, UUID, and path for the
    /// component using a closure.
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the object to reference
    /// - `f`: A closure that configures the [`ComponentBuilder`]
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_component_advanced(part_id, |comp| {
    ///     comp.transform(Transform([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0]));
    ///     comp.uuid("component-uuid");
    ///     comp.path("/path/to/model.model");
    /// });
    /// ```
    pub fn add_component_advanced<F>(&mut self, object_id: ObjectId, f: F) -> &mut Self
    where
        F: FnOnce(&mut ComponentBuilder),
    {
        let mut builder = ComponentBuilder::new(object_id);
        f(&mut builder);

        let component = builder.build();
        self.components.push(component);

        self
    }

    fn build_components(
        self,
        is_production_ext_required: bool,
    ) -> Result<Components, ComponentsObjectError> {
        if is_production_ext_required {
            let all_uuid_set = self.components.iter().all(|c| c.uuid.is_some());
            if !all_uuid_set {
                return Err(ComponentsObjectError::ComponentUuidNotSet);
            }
        } else {
            let all_path_is_not_set = self.components.iter().all(|c| c.path.is_none());
            if !all_path_is_not_set {
                return Err(ComponentsObjectError::PathSetWithoutProductionExtension);
            }
        }

        let all_object_exists = self
            .components
            .iter()
            .all(|c| self.all_existing_object_ids.contains(&ObjectId(c.objectid)));

        if !all_object_exists {
            return Err(ComponentsObjectError::ObjectReferenceNotFoundForComponent);
        }

        Ok(Components {
            component: self.components,
        })
    }
}

/// Builder for configuring an individual component within an assembly.
///
/// Components reference existing objects and can have their own transform,
/// UUID, and path attributes.
pub struct ComponentBuilder {
    objectid: u32,
    transform: Option<Transform>,
    path: Option<PathResource>,
    uuid: Option<UuidResource>,
}

impl ComponentBuilder {
    pub fn new(object_id: ObjectId) -> Self {
        Self {
            objectid: object_id.0,
            transform: None,
            path: None,
            uuid: None,
        }
    }

    pub fn transform(&mut self, transform: Transform) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// Set the UUID for this component.
    ///
    /// Required when Production extension is enabled.
    pub fn uuid(&mut self, uuid: &str) -> &mut Self {
        self.uuid = Some(UuidResource::from(uuid));
        self
    }

    /// Set the path for this component.
    ///
    /// Only allowed when Production extension is enabled. The path specifies
    /// an alternative model file where the referenced object can be found.
    pub fn path(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.path = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    fn build(self) -> Component {
        Component {
            objectid: self.objectid,
            transform: self.transform,
            path: self.path,
            uuid: self.uuid,
        }
    }
}

/// Errors that can occur when building a boolean shape object.
#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum BooleanShapeError {
    /// Base object ID is not set.
    ///
    /// Call [`BooleanShapeBuilder::base_object()`] before building.
    #[error("Base object ID is not set")]
    BaseObjectNotSet,

    /// No boolean operands were added.
    ///
    /// At least one boolean operand must be added via [`BooleanShapeBuilder::add_boolean()`].
    #[error("At least one boolean operand is required")]
    NoBooleanOperands,

    /// Object is missing a UUID when Production extension is required.
    ///
    /// Call [`BooleanObjectBuilder::uuid()`] to set the UUID.
    #[error("Production extension is enabled but Uuid is not set")]
    ObjectUuidNotSet,
}

/// Builder for creating boolean shape objects for CSG operations.
///
/// `BooleanObjectBuilder` creates objects that define boolean operations (union, difference,
/// intersection) between a base object and one or more operand objects. This is useful for
/// creating complex shapes through constructive solid geometry (CSG).
///
/// Access boolean operation configuration methods directly via [`Deref`] to [`BooleanShapeBuilder`].
///
/// # Examples
///
/// ```rust,ignore
/// // First create base mesh and operand meshes
/// let cube_id = builder.add_mesh_object(|obj| {
///     obj.name("Cube");
///     // ... add cube geometry
///     Ok(())
/// })?;
///
/// let sphere_id = builder.add_mesh_object(|obj| {
///     obj.name("Sphere");
///     // ... add sphere geometry
///     Ok(())
/// })?;
///
/// // Create boolean shape: cube minus sphere
/// let result_id = builder.add_booleanshape_object(|obj| {
///     obj.name("CubeMinusSphere");
///     obj.base_object(cube_id, BooleanOperation::Difference);
///     obj.add_boolean(sphere_id);
///     Ok(())
/// })?;
/// ```
pub type BooleanObjectBuilder = ObjectBuilder<BooleanShapeBuilder>;

impl BooleanObjectBuilder {
    fn new(object_id: ObjectId, is_production_ext_required: bool) -> Self {
        Self {
            entity: BooleanShapeBuilder::new(),
            object_id,
            objecttype: Some(object::ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<object::Object, BooleanShapeError> {
        let boolean_shape = self.entity.build_boolean_shape()?;

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(BooleanShapeError::ObjectUuidNotSet);
        }

        Ok(object::Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber.map(Into::into),
            name: self.name.map(Into::into),
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            slicestackid: self.slicestackid,
            slicepath: self.slicepath,
            meshresolution: self.meshresolution,
            kind: Some(object::ObjectKind::BooleanShape(boolean_shape)),
        })
    }
}

/// Builder for configuring boolean operations in a boolean shape object.
///
/// `BooleanShapeBuilder` manages the base object reference, boolean operation type,
/// and the list of operand objects for CSG operations.
pub struct BooleanShapeBuilder {
    base_object_id: Option<ObjectId>,
    operation: boolean::BooleanOperation,
    base_transform: Option<Transform>,
    base_path: Option<PathResource>,
    booleans: Vec<boolean::Boolean>,
}

impl BooleanShapeBuilder {
    fn new() -> Self {
        Self {
            base_object_id: None,
            operation: boolean::BooleanOperation::Union,
            base_transform: None,
            base_path: None,
            booleans: Vec::new(),
        }
    }

    /// Set the base object and boolean operation type.
    ///
    /// The base object is the primary object to which boolean operations are applied.
    /// The operation type determines how operands are combined with the base.
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the base object
    /// - `operation`: The boolean operation (Union, Difference, or Intersection)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.base_object(cube_id, BooleanOperation::Difference);
    /// ```
    pub fn base_object(
        &mut self,
        object_id: ObjectId,
        operation: boolean::BooleanOperation,
    ) -> &mut Self {
        self.base_object_id = Some(object_id);
        self.operation = operation;
        self
    }

    /// Set a transform to apply to the base object.
    ///
    /// # Parameters
    ///
    /// - `transform`: A 4x3 transformation matrix
    pub fn base_transform(&mut self, transform: Transform) -> &mut Self {
        self.base_transform = Some(transform);
        self
    }

    /// Set a path for the base object (Production extension).
    ///
    /// Only valid when used with the Production extension in root model files.
    ///
    /// # Parameters
    ///
    /// - `path`: Path to the model file containing the base object
    pub fn base_path(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.base_path = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    /// Add a boolean operand with the default configuration.
    ///
    /// This is a convenience method that adds an operand with no transform.
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the operand object
    pub fn add_boolean(&mut self, object_id: ObjectId) -> &mut Self {
        self.add_boolean_advanced(object_id, |_| {});
        self
    }

    /// Add a boolean operand with advanced configuration.
    ///
    /// This method allows you to configure transform and path for the operand
    /// using a closure.
    ///
    /// # Parameters
    ///
    /// - `object_id`: The ID of the operand object
    /// - `f`: A closure that configures the [`BooleanBuilder`]
    pub fn add_boolean_advanced<F>(&mut self, object_id: ObjectId, f: F) -> &mut Self
    where
        F: FnOnce(&mut BooleanBuilder),
    {
        let mut builder = BooleanBuilder::new(object_id);
        f(&mut builder);
        self.booleans.push(builder.build());
        self
    }

    fn build_boolean_shape(self) -> Result<boolean::BooleanShape, BooleanShapeError> {
        let base_object_id = self
            .base_object_id
            .ok_or(BooleanShapeError::BaseObjectNotSet)?;

        if self.booleans.is_empty() {
            return Err(BooleanShapeError::NoBooleanOperands);
        }

        Ok(boolean::BooleanShape {
            objectid: base_object_id.0,
            operation: self.operation,
            transform: self.base_transform,
            path: self.base_path,
            booleans: self.booleans,
        })
    }
}

/// Builder for configuring individual boolean operands.
///
/// `BooleanBuilder` allows you to set transforms and paths for boolean operands.
pub struct BooleanBuilder {
    objectid: ObjectId,
    transform: Option<Transform>,
    path: Option<PathResource>,
}

impl BooleanBuilder {
    fn new(objectid: ObjectId) -> Self {
        Self {
            objectid,
            transform: None,
            path: None,
        }
    }

    /// Set a transform to apply to this operand.
    ///
    /// # Parameters
    ///
    /// - `transform`: A 4x3 transformation matrix
    pub fn transform(&mut self, transform: Transform) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// Set a path for this operand (Production extension).
    ///
    /// Only valid when used with the Production extension in root model files.
    ///
    /// # Parameters
    ///
    /// - `path`: Path to the model file containing the operand object
    pub fn path(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.path = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    fn build(self) -> boolean::Boolean {
        boolean::Boolean {
            objectid: self.objectid.0,
            transform: self.transform,
            path: self.path,
        }
    }
}

/// Builder for organizing triangles into named sets.
///
/// Triangle sets allow you to group triangles within a mesh for organizational purposes,
/// such as identifying different faces or regions. Each set has a name and identifier,
/// and references triangles either individually or as ranges.
///
/// Triangle sets are added as a recommended extension (not required).
pub struct TriangleSetsBuilder {
    sets: Vec<triangle_set::TriangleSet>,
}

impl TriangleSetsBuilder {
    fn new() -> Self {
        Self { sets: Vec::new() }
    }

    /// Add a triangle set with a name, identifier, and triangle references.
    ///
    /// Triangles can be referenced either individually (via `refs`) or as ranges
    /// (via `ranges`). Both methods can be used together.
    ///
    /// # Parameters
    ///
    /// - `name`: Human-readable name for the set
    /// - `identifier`: Unique identifier for the set
    /// - `refs`: Slice of individual triangle indices to include
    /// - `ranges`: Slice of triangle index ranges (inclusive start, inclusive end)
    pub fn add_set(
        &mut self,
        name: &str,
        identifier: &str,
        refs: &[u32],
        ranges: &[(u32, u32)],
    ) -> &mut Self {
        use crate::model::domain::triangle_set::{TriangleRef, TriangleRefRange, TriangleSet};

        let triangle_ref = refs.iter().map(|&index| TriangleRef { index }).collect();
        let triangle_refrange = ranges
            .iter()
            .map(|&(start, end)| TriangleRefRange {
                startindex: start,
                endindex: end,
            })
            .collect();

        self.sets.push(TriangleSet {
            name: name.into(),
            identifier: identifier.into(),
            triangle_ref,
            triangle_refrange,
        });
        self
    }

    fn build(self) -> crate::model::domain::triangle_set::TriangleSets {
        crate::model::domain::triangle_set::TriangleSets {
            trianglesets: self.sets,
        }
    }
}

/// Builder for constructing beam lattice structures.
///
/// Beam lattices define strut-like structures connecting vertices in a mesh.
/// They are part of the 3MF Beam Lattice extension and are useful for representing
/// lightweight lattice structures, supports, or truss designs.
///
/// The builder allows you to:
/// - Define beams connecting pairs of vertices
/// - Add balls (spherical nodes) at vertices
/// - Organize beams and balls into named sets
/// - Configure default properties like radius, ball mode, clipping, etc.
///
/// The Beam Lattice extension namespace is automatically added when beams are present.
pub struct BeamLatticeBuilder {
    minlength: Option<f64>,
    radius: Option<f64>,
    ballmode: Option<beamlattice::BallMode>,
    ballradius: Option<f64>,
    clippingmode: Option<beamlattice::ClippingMode>,
    clippingmesh: OptionalResourceId,
    representationmesh: OptionalResourceId,
    pid: OptionalResourceId,
    pindex: OptionalResourceIndex,
    cap: Option<beamlattice::CapMode>,
    beams: Vec<beamlattice::Beam>,
    balls: Vec<beamlattice::Ball>,
    beamsets: Vec<beamlattice::BeamSet>,
}

impl BeamLatticeBuilder {
    fn new() -> Self {
        Self {
            minlength: None,
            radius: None,
            ballmode: None,
            ballradius: None,
            clippingmode: None,
            clippingmesh: OptionalResourceId::none(),
            representationmesh: OptionalResourceId::none(),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            cap: None,
            beams: Vec::new(),
            balls: Vec::new(),
            beamsets: Vec::new(),
        }
    }

    /// Set the minimum length for beams (default: 0.0001).
    ///
    /// Beams shorter than this length may be ignored during processing.
    pub fn minlength(&mut self, minlength: f64) -> &mut Self {
        self.minlength = Some(minlength);
        self
    }

    /// Set the default radius for all beams (default: 0.0001).
    ///
    /// Individual beams can override this with their own radius values.
    pub fn radius(&mut self, radius: f64) -> &mut Self {
        self.radius = Some(radius);
        self
    }

    /// Set how balls (nodes) are rendered at beam connections.
    ///
    /// See [`BallMode`] for available options.
    pub fn ballmode(&mut self, mode: beamlattice::BallMode) -> &mut Self {
        self.ballmode = Some(mode);
        self
    }

    /// Set the default radius for all balls.
    ///
    /// Individual balls can override this with their own radius values.
    pub fn ballradius(&mut self, radius: f64) -> &mut Self {
        self.ballradius = Some(radius);
        self
    }

    /// Set how the lattice is clipped by the clipping mesh.
    ///
    /// See [`ClippingMode`] for available options.
    pub fn clippingmode(&mut self, mode: beamlattice::ClippingMode) -> &mut Self {
        self.clippingmode = Some(mode);
        self
    }

    /// Set the mesh object used for clipping the lattice.
    ///
    /// The lattice will be clipped to the bounds of this mesh based on the clipping mode.
    pub fn clippingmesh(&mut self, object_id: ObjectId) -> &mut Self {
        self.clippingmesh = OptionalResourceId::new(object_id.0);
        self
    }

    /// Set an alternative mesh for visualization.
    ///
    /// This mesh can be used as a simplified representation of the lattice.
    pub fn representationmesh(&mut self, object_id: ObjectId) -> &mut Self {
        self.representationmesh = OptionalResourceId::new(object_id.0);
        self
    }

    /// Set the property ID for the lattice.
    pub fn pid(&mut self, pid: ResourceId) -> &mut Self {
        self.pid = OptionalResourceId::new(pid);
        self
    }

    /// Set the property index for the lattice.
    pub fn pindex(&mut self, pindex: ResourceIndex) -> &mut Self {
        self.pindex = Some(pindex).into();
        self
    }

    /// Set the default cap mode for beam ends.
    ///
    /// Individual beams can override this. See [`CapMode`] for available options.
    pub fn cap(&mut self, cap: beamlattice::CapMode) -> &mut Self {
        self.cap = Some(cap);
        self
    }

    /// Add a simple beam connecting two vertices.
    ///
    /// The beam will use default properties (radius, cap mode, etc.).
    ///
    /// # Parameters
    ///
    /// - `v1`: Index of the first vertex
    /// - `v2`: Index of the second vertex
    pub fn add_beam(&mut self, v1: ResourceIndex, v2: ResourceIndex) -> &mut Self {
        let beam = BeamBuilder::new(v1, v2).build();
        self.beams.push(beam);
        self
    }

    /// Add a beam with custom configuration.
    ///
    /// Use this method to configure individual beam properties like radius, cap modes, etc.
    ///
    /// # Parameters
    ///
    /// - `v1`: Index of the first vertex
    /// - `v2`: Index of the second vertex
    /// - `f`: A closure that configures the [`BeamBuilder`]
    pub fn add_beam_advanced<F>(&mut self, v1: ResourceIndex, v2: ResourceIndex, f: F) -> &mut Self
    where
        F: FnOnce(BeamBuilder) -> BeamBuilder,
    {
        let builder = BeamBuilder::new(v1, v2);
        let beam = f(builder).build();
        self.beams.push(beam);
        self
    }

    /// Add multiple simple beams from vertex pairs.
    ///
    /// # Parameters
    ///
    /// - `vertex_pairs`: Slice of `(v1, v2)` vertex index pairs
    pub fn add_beams(&mut self, vertex_pairs: &[(ResourceIndex, ResourceIndex)]) -> &mut Self {
        for &(v1, v2) in vertex_pairs {
            self.add_beam(v1, v2);
        }
        self
    }

    /// Add a simple ball (spherical node) at a vertex.
    ///
    /// The ball will use default properties (radius, etc.).
    ///
    /// # Parameters
    ///
    /// - `vindex`: Index of the vertex where the ball is located
    pub fn add_ball(&mut self, vindex: ResourceIndex) -> &mut Self {
        let ball = BallBuilder::new(vindex).build();
        self.balls.push(ball);
        self
    }

    /// Add a ball with custom configuration.
    ///
    /// Use this method to configure individual ball properties like radius.
    ///
    /// # Parameters
    ///
    /// - `vindex`: Index of the vertex where the ball is located
    /// - `f`: A closure that configures the [`BallBuilder`]
    pub fn add_ball_advanced<F>(&mut self, vindex: ResourceIndex, f: F) -> &mut Self
    where
        F: FnOnce(BallBuilder) -> BallBuilder,
    {
        let builder = BallBuilder::new(vindex);
        let ball = f(builder).build();
        self.balls.push(ball);
        self
    }

    /// Add multiple simple balls from vertex indices.
    ///
    /// # Parameters
    ///
    /// - `vindices`: Slice of vertex indices where balls should be placed
    pub fn add_balls(&mut self, vindices: &[ResourceIndex]) -> &mut Self {
        for &vindex in vindices {
            self.add_ball(vindex);
        }
        self
    }

    /// Add a beam set to organize beams and balls into named groups.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures the [`BeamSetBuilder`]
    pub fn add_beamset<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut BeamSetBuilder),
    {
        let mut builder = BeamSetBuilder::new();
        f(&mut builder);
        self.beamsets.push(builder.build());
        self
    }

    fn build(self) -> beamlattice::BeamLattice {
        let beams = beamlattice::Beams { beam: self.beams };

        let balls = if self.balls.is_empty() {
            None
        } else {
            Some(beamlattice::Balls { ball: self.balls })
        };

        let beamsets = if self.beamsets.is_empty() {
            None
        } else {
            Some(beamlattice::BeamSets {
                beamset: self.beamsets,
            })
        };

        beamlattice::BeamLattice {
            minlength: self.minlength.unwrap_or(0.0001),
            radius: self.radius.unwrap_or(0.0001),
            ballmode: self.ballmode,
            ballradius: self.ballradius,
            clippingmode: self.clippingmode,
            clippingmesh: self.clippingmesh,
            representationmesh: self.representationmesh,
            pid: self.pid,
            pindex: self.pindex,
            cap: self.cap,
            beams,
            balls,
            beamsets,
        }
    }
}

/// Builder for individual beams in a beam lattice
pub struct BeamBuilder {
    v1: ResourceIndex,
    v2: ResourceIndex,
    r1: Option<f64>,
    r2: Option<f64>,
    p1: OptionalResourceIndex,
    p2: OptionalResourceIndex,
    pid: OptionalResourceId,
    cap1: Option<beamlattice::CapMode>,
    cap2: Option<beamlattice::CapMode>,
}

impl BeamBuilder {
    /// Create a new beam with the specified vertex indices
    pub fn new(v1: ResourceIndex, v2: ResourceIndex) -> Self {
        Self {
            v1,
            v2,
            r1: None,
            r2: None,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
            cap1: None,
            cap2: None,
        }
    }

    /// Set the radius at the first vertex
    pub fn radius_1(mut self, radius: f64) -> Self {
        self.r1 = Some(radius);
        self
    }

    /// Set the radius at the second vertex
    pub fn radius_2(mut self, radius: f64) -> Self {
        self.r2 = Some(radius);
        self
    }

    /// Set the property index for the first vertex
    pub fn pindex_1(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p1 = pindex;
        self
    }

    /// Set the property index for the second vertex
    pub fn pindex_2(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p2 = pindex;
        self
    }

    /// Set the property ID for the beam
    pub fn pid(mut self, pid: ResourceId) -> Self {
        self.pid = OptionalResourceId::new(pid);
        self
    }

    /// Set the cap mode for the first end of the beam
    pub fn cap_1(mut self, cap: beamlattice::CapMode) -> Self {
        self.cap1 = Some(cap);
        self
    }

    /// Set the cap mode for the second end of the beam
    pub fn cap_2(mut self, cap: beamlattice::CapMode) -> Self {
        self.cap2 = Some(cap);
        self
    }

    fn build(self) -> beamlattice::Beam {
        beamlattice::Beam {
            v1: self.v1,
            v2: self.v2,
            r1: self.r1,
            r2: self.r2,
            p1: self.p1,
            p2: self.p2,
            pid: self.pid,
            cap1: self.cap1,
            cap2: self.cap2,
        }
    }
}

/// Builder for balls in a beam lattice
pub struct BallBuilder {
    vindex: ResourceIndex,
    r: Option<f64>,
    p: OptionalResourceIndex,
    pid: OptionalResourceId,
}

impl BallBuilder {
    /// Create a new ball at the specified vertex index
    pub fn new(vindex: ResourceIndex) -> Self {
        Self {
            vindex,
            r: None,
            p: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        }
    }

    /// Set the radius of the ball
    pub fn radius(mut self, radius: f64) -> Self {
        self.r = Some(radius);
        self
    }

    /// Set the property index for the ball
    pub fn pindex(mut self, pindex: OptionalResourceIndex) -> Self {
        self.p = pindex;
        self
    }

    /// Set the property ID for the ball
    pub fn pid(mut self, pid: ResourceIndex) -> Self {
        self.pid = OptionalResourceId::new(pid);
        self
    }

    fn build(self) -> beamlattice::Ball {
        beamlattice::Ball {
            vindex: self.vindex,
            r: self.r,
            p: self.p,
            pid: self.pid,
        }
    }
}

/// Builder for beam sets in a beam lattice
pub struct BeamSetBuilder {
    name: Option<String>,
    identifier: Option<String>,
    beam_refs: Vec<ResourceIndex>,
    ball_refs: Vec<ResourceIndex>,
}

impl BeamSetBuilder {
    fn new() -> Self {
        Self {
            name: None,
            identifier: None,
            beam_refs: Vec::new(),
            ball_refs: Vec::new(),
        }
    }

    /// Set the name of the beam set
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Set the identifier of the beam set
    pub fn identifier(&mut self, id: &str) -> &mut Self {
        self.identifier = Some(id.to_owned());
        self
    }

    /// Add a reference to a beam by index
    pub fn add_beam_ref(&mut self, index: ResourceIndex) -> &mut Self {
        self.beam_refs.push(index);
        self
    }

    /// Add multiple beam references
    pub fn add_beam_refs(&mut self, indices: &[ResourceIndex]) -> &mut Self {
        self.beam_refs.extend_from_slice(indices);
        self
    }

    /// Add a reference to a ball by index
    pub fn add_ball_ref(&mut self, index: ResourceIndex) -> &mut Self {
        self.ball_refs.push(index);
        self
    }

    /// Add multiple ball references
    pub fn add_ball_refs(&mut self, indices: &[ResourceIndex]) -> &mut Self {
        self.ball_refs.extend_from_slice(indices);
        self
    }

    fn build(self) -> beamlattice::BeamSet {
        beamlattice::BeamSet {
            name: self.name.map(Into::into),
            identifier: self.identifier.map(Into::into),
            refs: self
                .beam_refs
                .into_iter()
                .map(|index| beamlattice::BeamRef { index })
                .collect(),
            ballref: self
                .ball_refs
                .into_iter()
                .map(|index| beamlattice::BallRef { index })
                .collect(),
        }
    }
}

/// Builder for a color group resource.
pub struct ColorGroupBuilder {
    id: ResourceId,
    colors: Vec<material::ColorElement>,
}

impl ColorGroupBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            colors: Vec::new(),
        }
    }

    pub fn add_color(&mut self, color: crate::model::Color) -> &mut Self {
        self.colors.push(material::ColorElement { color });
        self
    }

    pub fn add_colors(&mut self, colors: &[crate::model::Color]) -> &mut Self {
        for color in colors {
            self.add_color(*color);
        }
        self
    }

    fn build(self) -> material::ColorGroup {
        material::ColorGroup {
            id: self.id,
            color: self.colors,
        }
    }
}

/// Builder for a texture2d resource.
pub struct Texture2DBuilder {
    id: ResourceId,
    path: Option<PathResource>,
    contenttype: Option<material::TextureContentType>,
    tilestyleu: Option<material::TileStyle>,
    tilestylev: Option<material::TileStyle>,
    filter: Option<material::Filter>,
}

impl Texture2DBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            path: None,
            contenttype: None,
            tilestyleu: None,
            tilestylev: None,
            filter: None,
        }
    }

    pub fn path(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.path = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    pub fn content_type(&mut self, content_type: material::TextureContentType) -> &mut Self {
        self.contenttype = Some(content_type);
        self
    }

    pub fn tilestyle_u(&mut self, tilestyle: material::TileStyle) -> &mut Self {
        self.tilestyleu = Some(tilestyle);
        self
    }

    pub fn tilestyle_v(&mut self, tilestyle: material::TileStyle) -> &mut Self {
        self.tilestylev = Some(tilestyle);
        self
    }

    pub fn filter(&mut self, filter: material::Filter) -> &mut Self {
        self.filter = Some(filter);
        self
    }

    fn build(self) -> Result<material::Texture2D, Texture2DError> {
        Ok(material::Texture2D {
            id: self.id,
            path: self.path.ok_or(Texture2DError::PathNotSet)?,
            contenttype: self.contenttype.ok_or(Texture2DError::ContentTypeNotSet)?,
            tilestyleu: self.tilestyleu,
            tilestylev: self.tilestylev,
            filter: self.filter,
        })
    }
}

/// Builder for a texture2d group resource.
pub struct Texture2DGroupBuilder {
    id: ResourceId,
    texid: Option<ResourceId>,
    tex2coord: Vec<material::Tex2Coord>,
}

impl Texture2DGroupBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            texid: None,
            tex2coord: Vec::new(),
        }
    }

    pub fn texid(&mut self, texid: ResourceId) -> &mut Self {
        self.texid = Some(texid);
        self
    }

    pub fn add_tex_coord(&mut self, u: f64, v: f64) -> &mut Self {
        self.tex2coord.push(material::Tex2Coord {
            u: u.into(),
            v: v.into(),
        });
        self
    }

    pub fn add_tex_coords(&mut self, coords: &[(f64, f64)]) -> &mut Self {
        for &(u, v) in coords {
            self.add_tex_coord(u, v);
        }
        self
    }

    fn build(self) -> Result<material::Texture2DGroup, Texture2DGroupError> {
        Ok(material::Texture2DGroup {
            id: self.id,
            texid: self.texid.ok_or(Texture2DGroupError::TexIdNotSet)?,
            tex2coord: self.tex2coord,
        })
    }
}

/// Builder for composite materials.
pub struct CompositeMaterialsBuilder {
    id: ResourceId,
    matid: Option<ResourceId>,
    matindices: Vec<ResourceIndex>,
    composite: Vec<material::Composite>,
}

impl CompositeMaterialsBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            matid: None,
            matindices: Vec::new(),
            composite: Vec::new(),
        }
    }

    pub fn matid(&mut self, matid: ResourceId) -> &mut Self {
        self.matid = Some(matid);
        self
    }

    pub fn matindices(&mut self, matindices: &[ResourceIndex]) -> &mut Self {
        self.matindices = matindices.to_vec();
        self
    }

    pub fn add_matindex(&mut self, matindex: ResourceIndex) -> &mut Self {
        self.matindices.push(matindex);
        self
    }

    pub fn add_composite(&mut self, values: &[f64]) -> &mut Self {
        self.composite.push(material::Composite {
            values: values.iter().copied().map(Into::into).collect(),
        });
        self
    }

    pub fn add_composites(&mut self, values: &[Vec<f64>]) -> &mut Self {
        for composite in values {
            self.add_composite(composite);
        }
        self
    }

    fn build(self) -> Result<material::CompositeMaterials, CompositeMaterialsError> {
        if self.matindices.is_empty() {
            return Err(CompositeMaterialsError::MatIndicesEmpty);
        }
        Ok(material::CompositeMaterials {
            id: self.id,
            matid: self.matid.ok_or(CompositeMaterialsError::MatIdNotSet)?,
            matindices: ResourceIndexCollection::from(self.matindices),
            composite: self.composite,
        })
    }
}

/// Builder for multi-properties.
pub struct MultiPropertiesBuilder {
    id: ResourceId,
    pids: Vec<ResourceId>,
    blendmethods: Option<StrResource>,
    multi: Vec<material::Multi>,
}

impl MultiPropertiesBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            pids: Vec::new(),
            blendmethods: None,
            multi: Vec::new(),
        }
    }

    pub fn pids(&mut self, pids: &[ResourceId]) -> &mut Self {
        self.pids = pids.to_vec();
        self
    }

    pub fn add_pid(&mut self, pid: ResourceId) -> &mut Self {
        self.pids.push(pid);
        self
    }

    pub fn blendmethods(&mut self, methods: &[material::BlendMethod]) -> &mut Self {
        let value = methods
            .iter()
            .map(|method| match method {
                material::BlendMethod::Mix => "mix",
                material::BlendMethod::Multiply => "multiply",
            })
            .collect::<Vec<_>>()
            .join(" ");
        self.blendmethods = Some(StrResource::new(&value));
        self
    }

    pub fn blendmethods_raw(&mut self, methods: &str) -> &mut Self {
        self.blendmethods = Some(StrResource::new(methods));
        self
    }

    pub fn add_multi(&mut self, pindices: &[ResourceIndex]) -> &mut Self {
        self.multi.push(material::Multi {
            pindices: ResourceIndexCollection::from(pindices.to_vec()),
        });
        self
    }

    fn build(self) -> Result<material::MultiProperties, MultiPropertiesError> {
        if self.pids.is_empty() {
            return Err(MultiPropertiesError::PidsEmpty);
        }
        Ok(material::MultiProperties {
            id: self.id,
            pids: ResourceIdCollection::from(self.pids),
            blendmethods: self.blendmethods,
            multi: self.multi,
        })
    }
}

/// Builder for base materials.
pub struct BaseMaterialsBuilder {
    id: ResourceId,
    bases: Vec<Base>,
}

impl BaseMaterialsBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            bases: Vec::new(),
        }
    }

    pub fn add_base(&mut self, name: &str, displaycolor: &str) -> &mut Self {
        self.bases.push(Base {
            name: StrResource::new(name),
            displaycolor: StrResource::new(displaycolor),
        });
        self
    }

    pub fn add_base_color(&mut self, name: &str, displaycolor: crate::model::Color) -> &mut Self {
        self.bases.push(Base {
            name: StrResource::new(name),
            displaycolor: StrResource::new(displaycolor.to_hex_compact()),
        });
        self
    }

    fn build(self) -> BaseMaterials {
        BaseMaterials {
            id: self.id,
            base: self.bases,
        }
    }
}

/// Builder for displacement2d resources.
pub struct Displacement2DBuilder {
    id: ResourceId,
    path: Option<PathResource>,
    channel: Option<displacement::ChannelName>,
    tilestyleu: Option<displacement::TileStyle>,
    tilestylev: Option<displacement::TileStyle>,
    filter: Option<displacement::Filter>,
}

impl Displacement2DBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            path: None,
            channel: None,
            tilestyleu: None,
            tilestylev: None,
            filter: None,
        }
    }

    pub fn path(&mut self, path: &str) -> &mut Self {
        match PathResource::try_from(path) {
            Ok(path) => {
                self.path = Some(path);
                self
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    pub fn channel(&mut self, channel: displacement::ChannelName) -> &mut Self {
        self.channel = Some(channel);
        self
    }

    pub fn tilestyle_u(&mut self, tilestyle: displacement::TileStyle) -> &mut Self {
        self.tilestyleu = Some(tilestyle);
        self
    }

    pub fn tilestyle_v(&mut self, tilestyle: displacement::TileStyle) -> &mut Self {
        self.tilestylev = Some(tilestyle);
        self
    }

    pub fn filter(&mut self, filter: displacement::Filter) -> &mut Self {
        self.filter = Some(filter);
        self
    }

    fn build(self) -> Result<displacement::Displacement2D, Displacement2DError> {
        Ok(displacement::Displacement2D {
            id: self.id,
            path: self.path.ok_or(Displacement2DError::PathNotSet)?,
            channel: self.channel,
            tilestyleu: self.tilestyleu,
            tilestylev: self.tilestylev,
            filter: self.filter,
        })
    }
}

/// Builder for norm vector groups.
pub struct NormVectorGroupBuilder {
    id: ResourceId,
    vectors: Vec<displacement::NormVector>,
}

impl NormVectorGroupBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            vectors: Vec::new(),
        }
    }

    pub fn add_norm_vector(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.vectors.push(displacement::NormVector {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        });
        self
    }

    pub fn add_norm_vectors(&mut self, vectors: &[[f64; 3]]) -> &mut Self {
        for vector in vectors {
            self.add_norm_vector(vector[0], vector[1], vector[2]);
        }
        self
    }

    fn build(self) -> displacement::NormVectorGroup {
        displacement::NormVectorGroup {
            id: self.id,
            normvector: self.vectors,
        }
    }
}

/// Builder for displacement 2d groups.
pub struct Disp2DGroupBuilder {
    id: ResourceId,
    dispid: Option<ResourceId>,
    nid: Option<ResourceId>,
    height: Option<f64>,
    offset: Option<f64>,
    coords: Vec<displacement::Disp2DCoord>,
}

impl Disp2DGroupBuilder {
    fn new(id: ResourceId) -> Self {
        Self {
            id,
            dispid: None,
            nid: None,
            height: None,
            offset: None,
            coords: Vec::new(),
        }
    }

    pub fn displacement_map_id(&mut self, dispid: ResourceId) -> &mut Self {
        self.dispid = Some(dispid);
        self
    }

    pub fn norm_vector_group_id(&mut self, nid: ResourceId) -> &mut Self {
        self.nid = Some(nid);
        self
    }

    pub fn height(&mut self, height: f64) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn offset(&mut self, offset: f64) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    pub fn add_coord(&mut self, u: f64, v: f64, n: ResourceIndex, f: Option<f64>) -> &mut Self {
        self.coords.push(displacement::Disp2DCoord {
            u: u.into(),
            v: v.into(),
            n,
            f: f.map(Into::into),
        });
        self
    }

    pub fn add_coords(&mut self, coords: &[(f64, f64, ResourceIndex, Option<f64>)]) -> &mut Self {
        for &(u, v, n, f) in coords {
            self.add_coord(u, v, n, f);
        }
        self
    }

    fn build(self) -> Result<displacement::Disp2DGroup, Disp2DGroupError> {
        Ok(displacement::Disp2DGroup {
            id: self.id,
            dispid: self.dispid.ok_or(Disp2DGroupError::DispIdNotSet)?,
            nid: self.nid.ok_or(Disp2DGroupError::NormVectorGroupIdNotSet)?,
            height: self.height.ok_or(Disp2DGroupError::HeightNotSet)?.into(),
            offset: self.offset.map(Into::into),
            disp2dcoord: self.coords,
        })
    }
}

/// Errors that can occur when SliceStack is built.
#[derive(Debug, Error, Clone, Copy, PartialEq)]
pub enum SliceStackBuilderError {
    /// Both Slice and Slice ref is set.
    #[error("Slice stack cannot contain both Slices and SliceRefs")]
    BothSliceAndSliceRefCannotBeSet,
}

/// Builder for constructing a slice stack with 2D slice data.
///
/// `SliceStackBuilder` allows you to define 2.5D geometry by adding slices
/// at different z-heights. Each slice contains vertices and polygons that
/// define the 2D contours at that layer.
///
/// Slice stacks can be referenced by objects to provide sliced model data
/// alongside or instead of mesh geometry.
///
/// # Examples
///
/// ```rust,ignore
/// use package::builder::{ModelBuilder, Unit};
///
/// let mut builder = ModelBuilder::new(Unit::Millimeter, true);
///
/// // Create a slice stack with two layers
/// let stack_id = builder.add_slice_stack(|stack| {
///     stack.zbottom(0.0);
///     
///     // Add first slice at z=0.1
///     stack.add_slice(|slice| {
///         slice.ztop(0.1);
///         slice.add_vertices(&[
///             (0.0, 0.0),
///             (10.0, 0.0),
///             (10.0, 10.0),
///             (0.0, 10.0),
///         ]);
///         slice.add_polygon(|poly| {
///             poly.start_vertex(0);
///             poly.add_segment(1);
///             poly.add_segment(2);
///             poly.add_segment(3);
///         });
///     });
/// });
///
/// // Create an object that references the slice stack
/// let obj_id = builder.add_mesh_object(|obj| {
///     obj.name("SlicedObject");
///     obj.slice_stack(stack_id, None, None); // slicestack_id, slicepath, meshresolution
///     // ... add mesh data
///     Ok(())
/// }).unwrap();
/// ```
pub struct SliceStackBuilder {
    id: SliceStackId,
    zbottom: Option<f64>,
    slices: Vec<slice::Slice>,
    slicerefs: Vec<slice::SliceRef>,
}

impl SliceStackBuilder {
    fn new(id: SliceStackId) -> Self {
        Self {
            id,
            zbottom: None,
            slices: Vec::new(),
            slicerefs: Vec::new(),
        }
    }

    /// Set the starting z-level relative to the build platform.
    ///
    /// This allows alignment between mesh vertices and slice data.
    ///
    /// # Parameters
    ///
    /// - `zbottom`: The z-bottom position in model units
    pub fn zbottom(&mut self, zbottom: f64) -> &mut Self {
        self.zbottom = Some(zbottom);
        self
    }

    /// Add a slice (2D layer) to this stack.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures a [`SliceBuilder`]
    pub fn add_slice<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut SliceBuilder),
    {
        let mut builder = SliceBuilder::new();
        f(&mut builder);
        self.slices.push(builder.build());
        self
    }

    /// Add a reference to an external slice stack.
    ///
    /// This is used when slice data is stored in a separate model file.
    ///
    /// # Parameters
    ///
    /// - `slicestackid`: The ID of the slice stack in the external file
    /// - `slicepath`: Path to the model file containing the slice stack
    pub fn add_sliceref(&mut self, slicestackid: ResourceId, slicepath: &str) -> &mut Self {
        self.slicerefs.push(slice::SliceRef {
            slicestackid,
            slicepath: PathResource::try_from(slicepath).expect("Invalid PathResource"),
        });
        self
    }

    fn build(self) -> Result<slice::SliceStack, SliceStackBuilderError> {
        if !self.slicerefs.is_empty() && !self.slices.is_empty() {
            Err(SliceStackBuilderError::BothSliceAndSliceRefCannotBeSet)
        } else {
            Ok(slice::SliceStack {
                id: self.id.0,
                zbottom: self.zbottom.map(|zbot| zbot.into()),
                slice: self.slices,
                sliceref: self.slicerefs,
            })
        }
    }
}

/// Builder for constructing individual 2D slices within a slice stack.
///
/// Each slice represents a horizontal cross-section at a specific z-height.
pub struct SliceBuilder {
    ztop: Option<f64>,
    vertices: Vec<slice::Vertex>,
    polygons: Vec<slice::Polygon>,
}

impl SliceBuilder {
    fn new() -> Self {
        Self {
            ztop: None,
            vertices: Vec::new(),
            polygons: Vec::new(),
        }
    }

    /// Set the z-position of the top of this slice.
    ///
    /// # Parameters
    ///
    /// - `ztop`: The z-top position in model units
    pub fn ztop(&mut self, ztop: f64) -> &mut Self {
        self.ztop = Some(ztop);
        self
    }

    /// Add a 2D vertex to this slice.
    ///
    /// Vertices are referenced by their 0-based index in the order added.
    ///
    /// # Parameters
    ///
    /// - `x`: X coordinate
    /// - `y`: Y coordinate
    pub fn add_vertex(&mut self, x: f64, y: f64) -> &mut Self {
        self.vertices.push(slice::Vertex {
            x: x.into(),
            y: y.into(),
        });
        self
    }

    /// Add multiple vertices from (x, y) tuples.
    ///
    /// # Parameters
    ///
    /// - `vertices`: Slice of (x, y) tuples
    pub fn add_vertices(&mut self, vertices: &[(f64, f64)]) -> &mut Self {
        for &(x, y) in vertices {
            self.vertices.push(slice::Vertex {
                x: x.into(),
                y: y.into(),
            });
        }
        self
    }

    /// Add a polygon to this slice.
    ///
    /// # Parameters
    ///
    /// - `f`: A closure that configures a [`PolygonBuilder`]
    pub fn add_polygon<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut PolygonBuilder),
    {
        let mut builder = PolygonBuilder::new();
        f(&mut builder);
        self.polygons.push(builder.build());
        self
    }

    fn build(self) -> slice::Slice {
        let vertices = if self.vertices.is_empty() {
            None
        } else {
            Some(slice::Vertices {
                vertex: self.vertices,
            })
        };

        slice::Slice {
            ztop: self.ztop.unwrap_or(0.0).into(),
            vertices,
            polygon: self.polygons,
        }
    }
}

/// Builder for constructing polygons within a slice.
///
/// Polygons define closed or open contours made of line segments.
pub struct PolygonBuilder {
    startv: Option<ResourceIndex>,
    segments: Vec<slice::Segment>,
}

impl PolygonBuilder {
    fn new() -> Self {
        Self {
            startv: None,
            segments: Vec::new(),
        }
    }

    /// Set the starting vertex index for this polygon.
    ///
    /// This is the index of the first vertex of the first segment.
    ///
    /// # Parameters
    ///
    /// - `startv`: The starting vertex index
    pub fn start_vertex(&mut self, startv: ResourceIndex) -> &mut Self {
        self.startv = Some(startv);
        self
    }

    /// Add a segment to this polygon.
    ///
    /// Each segment connects from the previous segment's v2 (or startv for
    /// the first segment) to the specified vertex v2.
    ///
    /// # Parameters
    ///
    /// - `v2`: The index of the second vertex of this segment
    pub fn add_segment(&mut self, v2: ResourceIndex) -> &mut Self {
        self.segments.push(slice::Segment {
            v2,
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        });
        self
    }

    /// Add a segment with property indices.
    ///
    /// # Parameters
    ///
    /// - `v2`: The index of the second vertex of this segment
    /// - `p1`: Property index for the first vertex
    /// - `p2`: Property index for the second vertex
    pub fn add_segment_with_properties(
        &mut self,
        v2: ResourceIndex,
        p1: OptionalResourceIndex,
        p2: OptionalResourceIndex,
    ) -> &mut Self {
        self.segments.push(slice::Segment {
            v2,
            p1,
            p2,
            pid: OptionalResourceId::none(),
        });
        self
    }

    /// Add a segment with property indices and property id.
    pub fn add_segment_with_properties_and_pid(
        &mut self,
        v2: ResourceIndex,
        p1: OptionalResourceIndex,
        p2: OptionalResourceIndex,
        pid: OptionalResourceId,
    ) -> &mut Self {
        self.segments.push(slice::Segment { v2, p1, p2, pid });
        self
    }

    fn build(self) -> slice::Polygon {
        slice::Polygon {
            startv: self.startv.unwrap_or(0),
            segment: self.segments,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_model_builder_basic() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.unit(Unit::Millimeter);
        builder.add_metadata("Application", Some("Test App"));
        builder.add_build(None).unwrap();

        let cube_id = builder
            .add_mesh_object(|obj| {
                obj.name("Cube");
                obj.object_type(object::ObjectType::Model);
                obj.add_vertex(&[0.0, 0.0, 0.0])
                    .add_vertex(&[10.0, 0.0, 0.0])
                    .add_vertex(&[10.0, 10.0, 0.0])
                    .add_vertex(&[0.0, 10.0, 0.0])
                    .add_triangle(&[0, 1, 2])
                    .add_triangle(&[0, 2, 3]);

                Ok(())
            })
            .unwrap();

        builder.add_build_item(cube_id).unwrap();

        let model = builder.build().unwrap();

        assert_eq!(model.unit, Some(Unit::Millimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.metadata[0].name.as_ref(), "Application");
        assert_eq!(model.resources.object.len(), 1);
        assert_eq!(model.resources.object[0].name.as_deref(), Some("Cube"));
        assert_eq!(model.build.item.len(), 1);
        assert_eq!(model.build.item[0].objectid, 1);
    }

    #[test]
    fn test_object_id_assignment() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, false);

        let id1 = builder
            .add_mesh_object(|obj| {
                obj.name("Obj1");
                Ok(())
            })
            .unwrap();
        let id2 = builder
            .add_mesh_object(|obj| {
                obj.name("Obj2");
                Ok(())
            })
            .unwrap();

        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);

        let model = builder.build().unwrap();
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.resources.object[0].id, 1);
        assert_eq!(model.resources.object[1].id, 2);
    }

    #[test]
    fn test_multiple_passes() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);

        // First pass
        builder.unit(Unit::Centimeter);
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("First");
                Ok(())
            })
            .unwrap();

        // Second pass
        builder.add_metadata("Pass", Some("Second"));
        let obj2_id = builder
            .add_mesh_object(|obj| {
                obj.name("Second");
                Ok(())
            })
            .unwrap();

        // Third pass
        builder.add_build(None).unwrap();
        builder.add_build_item(obj1_id).unwrap();
        builder.add_build_item(obj2_id).unwrap();

        let model = builder.build().unwrap();
        assert_eq!(model.unit, Some(Unit::Centimeter));
        assert_eq!(model.metadata.len(), 1);
        assert_eq!(model.resources.object.len(), 2);
        assert_eq!(model.build.item.len(), 2);
    }

    #[test]
    fn test_error_cases_for_build_in_model() {
        // Test BuildItemNotSet: root model without build
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_mesh_object(|_obj| Ok(())).unwrap();
        assert!(matches!(builder.build(), Err(ModelError::BuildItemNotSet)));

        // Test BuildOnlyAllowedInRootModel: non-root model with build
        let mut builder = ModelBuilder::new(Unit::Millimeter, false);
        assert!(matches!(
            builder.add_build(None),
            Err(ModelError::BuildOnlyAllowedInRootModel)
        ));
    }

    #[test]
    fn test_production_ext_add_prod_ns_to_required_extensions() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); //should not return error
        builder
            .add_build(Some(UuidResource::from("build-uuid")))
            .unwrap();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.uuid("obj-uuid");
                obj.name("test");

                Ok(())
            })
            .unwrap();
        builder
            .add_build_item_advanced(obj_id, |i| {
                i.uuid("item-uuid");
            })
            .unwrap();
        let model = builder.build().unwrap();

        assert_eq!(
            model.requiredextensions,
            ThreemfExtensions::new(&[ThreemfNamespace::Prod])
        );
    }

    #[test]
    fn test_production_ext_requires_object_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // should not return error;
        builder
            .add_build(Some(UuidResource::from("build-uuid")))
            .unwrap();
        let obj_id = builder.add_mesh_object(|obj| {
            obj.name("test");
            // no uuid
            Ok(())
        });
        assert!(matches!(obj_id, Err(MeshObjectError::ObjectUuidNotSet)));
    }

    #[test]
    fn test_production_ext_requires_build_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); //should not return error
        let result = builder.add_build(None);
        assert!(matches!(
            result,
            Err(ModelError::BuildError(BuildError::BuildUuidNotSet))
        ));
    }

    #[test]
    fn test_production_ext_requires_build_item_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); //should not return error
        builder
            .add_build(Some(UuidResource::from("build-uuid")))
            .unwrap();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("test");
                obj.uuid("some-uuid");
                Ok(())
            })
            .unwrap();

        let err = builder.add_build_item(obj_id);
        assert!(matches!(
            err,
            Err(ModelError::ItemError(ItemError::ItemUuidNotSet))
        ));
    }

    #[test]
    fn test_production_ext_required_for_build_item_path() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_build(None).unwrap();
        let obj_id = builder.add_mesh_object(|_obj| Ok(())).unwrap();
        let result = builder.add_build_item_advanced(obj_id, |i| {
            i.path("some-path");
        });
        assert!(matches!(
            result,
            Err(ModelError::ItemError(
                ItemError::ItemPathSetWithoutProductionExtension
            ))
        ));
    }

    #[test]
    fn test_production_ext_allows_path_on_component_and_build_item() {
        // Test path allowed when production ext enabled
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // should not return error

        let mesh_obj_id = builder
            .add_mesh_object(|obj| {
                obj.uuid("object-uuid");
                Ok(())
            })
            .unwrap();

        if let Err(err) = builder.add_components_object(|obj| {
            obj.uuid("obj-uuid");
            obj.add_component_advanced(mesh_obj_id, |c| {
                c.uuid("comp-uuid").path("comp-path");
            });

            Ok(())
        }) {
            panic!("{err:?}");
        }

        builder
            .add_build(Some(UuidResource::from("build-uuid")))
            .unwrap();
        if let Err(err) = builder.add_build_item_advanced(mesh_obj_id, |i| {
            i.uuid("some-uuid").path("some-path");
        }) {
            panic!("{err:?}");
        }
    }

    #[test]
    fn test_custom_extensions() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);

        builder.add_required_extension(ThreemfNamespace::Unknown {
            prefix: "test".into(),
            uri: "http://example.com/test".into(),
        });

        builder.add_recommended_extension(ThreemfNamespace::Unknown {
            prefix: "rec".into(),
            uri: "http://example.com/rec".into(),
        });
        builder.add_build(None).unwrap();
        let model = builder.build().unwrap();

        assert_eq!(
            model.requiredextensions,
            ThreemfExtensions::new(&[ThreemfNamespace::Unknown {
                prefix: "test".into(),
                uri: "http://example.com/test".into()
            }])
        );
        assert_eq!(
            model.recommendedextensions,
            ThreemfExtensions::new(&[ThreemfNamespace::Unknown {
                prefix: "rec".into(),
                uri: "http://example.com/rec".into()
            }])
        );
    }

    #[test]
    fn test_material_and_displacement_extensions_from_resources() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_build(None).unwrap();

        builder.add_color_group(|group| {
            group.add_color(crate::model::Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            });
        });

        builder
            .add_displacement2d(|disp| {
                disp.path("/3D/Textures/disp.png");
            })
            .unwrap();

        let model = builder.build().unwrap();

        assert_eq!(
            model.requiredextensions,
            ThreemfExtensions::new(&[ThreemfNamespace::Displacement, ThreemfNamespace::Material,])
        );
    }

    #[test]
    fn test_add_displacement_mesh_object() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_build(None).unwrap();

        let obj_id = builder
            .add_displacement_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
                obj.add_triangle(&[0, 1, 2]);
                Ok(())
            })
            .unwrap();

        builder.add_build_item(obj_id).unwrap();

        let model = builder.build().unwrap();
        let obj = &model.resources.object[0];
        assert!(obj.get_displacement_mesh().is_some());
    }

    #[cfg(not(feature = "uuid"))]
    #[test]
    fn test_build_item_advanced_tests() {
        use crate::model::StrResource;

        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let _ = builder.make_production_extension_required();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("test").uuid("obj-uuid");
                Ok(())
            })
            .unwrap();
        builder
            .add_build(Some(UuidResource::from("build-uuid")))
            .unwrap();

        let transform = crate::model::transform::Transform([
            1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        ]);
        builder
            .add_build_item_advanced(obj_id, |i| {
                i.transform(transform.clone())
                    .partnumber("part")
                    .path("/path/path.model")
                    .uuid("uuid");
            })
            .unwrap();

        let model = builder.build().unwrap();
        let item = &model.build.item[0];
        assert_eq!(item.objectid, 1);
        assert_eq!(item.transform, Some(transform));
        assert_eq!(item.partnumber, Some(StrResource::new("part")));
        assert_eq!(
            item.path,
            Some(PathResource::try_from("/path/path.model").unwrap())
        );
        assert_eq!(item.uuid, Some(UuidResource::MaybeUuid("uuid".into())));
    }

    #[cfg(not(feature = "uuid"))]
    #[test]
    fn test_object_builder_tests() {
        use crate::model::StrResource;

        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.object_type(crate::model::object::ObjectType::Support);
                obj.name("support obj");
                obj.part_number("part123");
                obj.uuid("obj-uuid");
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();
        let model = builder.build().unwrap();
        let obj = &model.resources.object[0];
        assert_eq!(
            obj.objecttype,
            Some(crate::core::object::ObjectType::Support)
        );
        assert_eq!(obj.name, Some(StrResource::new("support obj")));
        assert_eq!(obj.partnumber, Some(StrResource::new("part123")));
        assert_eq!(obj.uuid, Some(UuidResource::MaybeUuid("obj-uuid".into())));
    }

    #[test]
    fn test_add_mesh_object() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
                obj.add_vertices_flat(&[0.0, 0.0, 1.0, 1.0, 1.0, 0.0]);
                obj.add_triangles(&[[0, 1, 2]]);
                obj.add_triangles_flat(&[0, 2, 3, 1, 3, 4]);

                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();
        let model = builder.build().unwrap();
        let mesh = model.resources.object[0].get_mesh().unwrap();
        assert_eq!(mesh.vertices.vertex.len(), 5);
        assert_eq!(mesh.triangles.triangle.len(), 3);
    }

    #[test]
    fn test_add_composed_part_object() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("obj1");

                Ok(())
            })
            .unwrap();
        let obj2_id = builder
            .add_components_object(|obj| {
                obj.add_component(obj1_id);
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj2_id).unwrap();
        let model = builder.build().unwrap();
        let obj = &model.resources.object[1];
        assert!(obj.get_components_object().is_some());
        let comp = &obj.get_components_object().unwrap().component[0];
        assert_eq!(comp.objectid, obj1_id.0);
    }

    #[test]
    fn test_add_composed_part_object_errors_without_production_extension() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("obj1");

                Ok(())
            })
            .unwrap();
        let _obj2_id = builder
            .add_components_object(|obj| {
                obj.add_component(obj1_id);
                Ok(())
            })
            .unwrap();

        // Test non-existent object reference
        let result = builder.add_components_object(|obj| {
            obj.add_component(ObjectId(999));
            Ok(())
        });
        assert!(matches!(
            result,
            Err(ComponentsObjectError::ObjectReferenceNotFoundForComponent)
        ));

        // Path set without Production extension
        let result = builder.add_components_object(|obj| {
            obj.add_component_advanced(obj1_id, |c| {
                c.path("some-path");
            });

            Ok(())
        });

        assert!(matches!(
            result,
            Err(ComponentsObjectError::PathSetWithoutProductionExtension)
        ));
    }

    #[test]
    fn test_add_composed_part_object_errors_with_production_extension() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // dont expect to return error;
        let obj1_id = builder
            .add_mesh_object(|obj| {
                obj.name("obj1");
                obj.uuid("some-mesh-uuid");

                Ok(())
            })
            .unwrap();

        //Test missing Object Uuid
        let result = builder.add_components_object(|obj| {
            obj.add_component_advanced(obj1_id, |c| {
                c.uuid("some-component-uuid").path("some-component-path");
            });
            Ok(())
        });

        assert!(matches!(
            result,
            Err(ComponentsObjectError::ObjectUuidNotSet)
        ));

        //Test missing Component Uuid
        let result = builder.add_components_object(|obj| {
            obj.uuid("some-obj-uuid");
            obj.add_component_advanced(obj1_id, |c| {
                c.path("some-component-path");
            });
            Ok(())
        });

        assert!(matches!(
            result,
            Err(ComponentsObjectError::ComponentUuidNotSet)
        ));
    }

    #[test]
    fn test_root_nonroot_tests() {
        // Non-root cannot have build
        let mut builder = ModelBuilder::new(Unit::Millimeter, false);
        assert!(matches!(
            builder.add_build(None),
            Err(ModelError::BuildOnlyAllowedInRootModel)
        ));

        // Root requires build
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_mesh_object(|_obj| Ok(())).unwrap();
        assert!(matches!(builder.build(), Err(ModelError::BuildItemNotSet)));

        // make_root(false) allows no build
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_root(false);
        builder.add_mesh_object(|_obj| Ok(())).unwrap();
        let model = builder.build().unwrap();
        assert!(model.build.item.is_empty());
    }

    #[test]
    fn test_metadata_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_metadata("key1", Some("value1"));
        builder.add_metadata("key2", None);
        builder.add_metadata("key3", Some("value3"));
        builder.add_build(None).unwrap();
        let model = builder.build().unwrap();
        assert_eq!(model.metadata.len(), 3);
        assert_eq!(model.metadata[0].name.as_ref(), "key1");
        assert_eq!(model.metadata[0].value.as_deref(), Some("value1"));
        assert_eq!(model.metadata[1].name.as_ref(), "key2");
        assert_eq!(model.metadata[1].value, None);
        assert_eq!(model.metadata[2].name.as_ref(), "key3");
        assert_eq!(model.metadata[2].value.as_deref(), Some("value3"));
    }

    #[test]
    fn test_triangle_sets_builder() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
                obj.add_triangles(&[[0, 1, 2]]);
                obj.add_triangle_sets(|ts| {
                    ts.add_set("Set1", "id1", &[0], &[(1, 5)]);
                    ts.add_set("Set2", "id2", &[], &[(10, 20), (30, 40)]);
                });
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();
        let model = builder.build().unwrap();
        let mesh = model.resources.object[0].get_mesh().unwrap();
        assert!(mesh.trianglesets.is_some());
        let sets = &mesh.trianglesets.as_ref().unwrap().trianglesets;
        assert_eq!(sets.len(), 2);
        assert_eq!(sets[0].name.as_ref(), "Set1");
        assert_eq!(sets[0].identifier.as_ref(), "id1");
        assert_eq!(sets[0].triangle_ref.len(), 1);
        assert_eq!(sets[0].triangle_ref[0].index, 0);
        assert_eq!(sets[0].triangle_refrange.len(), 1);
        assert_eq!(sets[0].triangle_refrange[0].startindex, 1);
        assert_eq!(sets[0].triangle_refrange[0].endindex, 5);
        assert_eq!(sets[1].name.as_ref(), "Set2");
        assert_eq!(sets[1].identifier.as_ref(), "id2");
        assert_eq!(sets[1].triangle_ref.len(), 0);
        assert_eq!(sets[1].triangle_refrange.len(), 2);

        //check if Triangle set is in recommended extensions
        assert_eq!(
            model.recommendedextensions,
            ThreemfExtensions::new(&[ThreemfNamespace::CoreTriangleSet])
        );
    }

    #[test]
    fn test_object_id_tests() {
        let id: ObjectId = 42.into();
        assert_eq!(id.0, 42);
    }

    // ========== BeamBuilder Tests ==========

    #[test]
    fn test_beam_builder_basic() {
        let beam = BeamBuilder::new(0, 1).build();

        assert_eq!(beam.v1, 0);
        assert_eq!(beam.v2, 1);
        assert_eq!(beam.r1, None);
        assert_eq!(beam.r2, None);
        assert_eq!(beam.p1, OptionalResourceIndex::none());
        assert_eq!(beam.p2, OptionalResourceIndex::none());
        assert_eq!(beam.pid, OptionalResourceId::none());
        assert_eq!(beam.cap1, None);
        assert_eq!(beam.cap2, None);
    }

    #[test]
    fn test_beam_builder_with_all_options() {
        let beam = BeamBuilder::new(0, 1)
            .radius_1(1.5)
            .radius_2(2.0)
            .pindex_1(OptionalResourceIndex::new(10))
            .pindex_2(OptionalResourceIndex::new(20))
            .pid(5)
            .cap_1(beamlattice::CapMode::Hemisphere)
            .cap_2(beamlattice::CapMode::Butt)
            .build();

        assert_eq!(beam.v1, 0);
        assert_eq!(beam.v2, 1);
        assert_eq!(beam.r1, Some(1.5));
        assert_eq!(beam.r2, Some(2.0));
        assert_eq!(beam.p1, OptionalResourceIndex::new(10));
        assert_eq!(beam.p2, OptionalResourceIndex::new(20));
        assert_eq!(beam.pid, OptionalResourceId::new(5));
        assert_eq!(beam.cap1, Some(beamlattice::CapMode::Hemisphere));
        assert_eq!(beam.cap2, Some(beamlattice::CapMode::Butt));
    }

    // ========== BallBuilder Tests ==========

    #[test]
    fn test_ball_builder_basic() {
        let ball = BallBuilder::new(5).build();

        assert_eq!(ball.vindex, 5);
        assert_eq!(ball.r, None);
        assert_eq!(ball.p, OptionalResourceIndex::none());
        assert_eq!(ball.pid, OptionalResourceId::none());
    }

    #[test]
    fn test_ball_builder_with_options() {
        let ball = BallBuilder::new(5)
            .radius(0.75)
            .pindex(OptionalResourceIndex::new(15))
            .pid(3)
            .build();

        assert_eq!(ball.vindex, 5);
        assert_eq!(ball.r, Some(0.75));
        assert_eq!(ball.p, OptionalResourceIndex::new(15));
        assert_eq!(ball.pid, OptionalResourceId::new(3));
    }

    // ========== BeamSetBuilder Tests ==========

    #[test]
    fn test_beamset_builder() {
        let mut builder = BeamSetBuilder::new();
        builder
            .name("Test Set")
            .identifier("test-set-001")
            .add_beam_refs(&[0, 1, 2, 3])
            .add_ball_ref(0)
            .add_ball_ref(1);

        let beamset = builder.build();

        assert_eq!(beamset.name, Some("Test Set".into()));
        assert_eq!(beamset.identifier, Some("test-set-001".into()));
        assert_eq!(beamset.refs.len(), 4);
        assert_eq!(beamset.refs[0].index, 0);
        assert_eq!(beamset.refs[3].index, 3);
        assert_eq!(beamset.ballref.len(), 2);
        assert_eq!(beamset.ballref[0].index, 0);
        assert_eq!(beamset.ballref[1].index, 1);
    }

    // ========== BeamLatticeBuilder Tests ==========

    #[test]
    fn test_beam_lattice_builder_minimal() {
        let mut builder = BeamLatticeBuilder::new();
        builder.add_beam(0, 1);

        let beamlattice = builder.build();

        // Should use default values
        assert_eq!(beamlattice.minlength, 0.0001);
        assert_eq!(beamlattice.radius, 0.0001);
        assert_eq!(beamlattice.beams.beam.len(), 1);
        assert_eq!(beamlattice.beams.beam[0].v1, 0);
        assert_eq!(beamlattice.beams.beam[0].v2, 1);
        assert_eq!(beamlattice.balls, None);
        assert_eq!(beamlattice.beamsets, None);
    }

    #[test]
    fn test_beam_lattice_default_values() {
        let mut builder = BeamLatticeBuilder::new();
        // Don't set minlength or radius
        builder.add_beam(0, 1);

        let beamlattice = builder.build();

        assert_eq!(beamlattice.minlength, 0.0001);
        assert_eq!(beamlattice.radius, 0.0001);
    }

    #[test]
    fn test_beam_lattice_builder_with_custom_values() {
        let mut builder = BeamLatticeBuilder::new();
        builder
            .minlength(0.001)
            .radius(1.0)
            .add_beam(0, 1)
            .add_beam(1, 2);

        let beamlattice = builder.build();

        assert_eq!(beamlattice.minlength, 0.001);
        assert_eq!(beamlattice.radius, 1.0);
        assert_eq!(beamlattice.beams.beam.len(), 2);
    }

    #[test]
    fn test_beam_lattice_builder_with_balls() {
        let mut builder = BeamLatticeBuilder::new();
        builder
            .minlength(0.001)
            .radius(1.0)
            .ballmode(beamlattice::BallMode::Mixed)
            .ballradius(0.5)
            .add_beam(0, 1)
            .add_ball(0)
            .add_ball_advanced(1, |b| b.radius(0.75));

        let beamlattice = builder.build();

        assert_eq!(beamlattice.ballmode, Some(beamlattice::BallMode::Mixed));
        assert_eq!(beamlattice.ballradius, Some(0.5));
        assert!(beamlattice.balls.is_some());

        let balls = beamlattice.balls.as_ref().unwrap();
        assert_eq!(balls.ball.len(), 2);
        assert_eq!(balls.ball[0].vindex, 0);
        assert_eq!(balls.ball[0].r, None); // Uses default
        assert_eq!(balls.ball[1].vindex, 1);
        assert_eq!(balls.ball[1].r, Some(0.75)); // Custom radius
    }

    #[test]
    fn test_beam_lattice_builder_with_all_features() {
        let clip_mesh_id = ObjectId(10);
        let repr_mesh_id = ObjectId(20);

        let mut builder = BeamLatticeBuilder::new();
        builder
            .minlength(0.002)
            .radius(2.0)
            .ballmode(beamlattice::BallMode::All)
            .ballradius(1.0)
            .clippingmode(beamlattice::ClippingMode::Inside)
            .clippingmesh(clip_mesh_id)
            .representationmesh(repr_mesh_id)
            .pid(5)
            .pindex(10)
            .cap(beamlattice::CapMode::Sphere)
            .add_beam(0, 1);

        let beamlattice = builder.build();

        assert_eq!(beamlattice.minlength, 0.002);
        assert_eq!(beamlattice.radius, 2.0);
        assert_eq!(beamlattice.ballmode, Some(beamlattice::BallMode::All));
        assert_eq!(beamlattice.ballradius, Some(1.0));
        assert_eq!(
            beamlattice.clippingmode,
            Some(beamlattice::ClippingMode::Inside)
        );
        assert_eq!(beamlattice.clippingmesh, OptionalResourceId::new(10));
        assert_eq!(beamlattice.representationmesh, OptionalResourceId::new(20));
        assert_eq!(beamlattice.pid, OptionalResourceId::new(5));
        assert_eq!(beamlattice.pindex, OptionalResourceIndex::new(10));
        assert_eq!(beamlattice.cap, Some(beamlattice::CapMode::Sphere));
    }

    #[test]
    fn test_beam_lattice_add_beams_bulk() {
        let mut builder = BeamLatticeBuilder::new();
        builder
            .radius(1.0)
            .add_beams(&[(0, 1), (1, 2), (2, 3), (3, 0)]);

        let beamlattice = builder.build();

        assert_eq!(beamlattice.beams.beam.len(), 4);
        assert_eq!(beamlattice.beams.beam[0].v1, 0);
        assert_eq!(beamlattice.beams.beam[0].v2, 1);
        assert_eq!(beamlattice.beams.beam[3].v1, 3);
        assert_eq!(beamlattice.beams.beam[3].v2, 0);
    }

    #[test]
    fn test_beam_lattice_add_balls_bulk() {
        let mut builder = BeamLatticeBuilder::new();
        builder
            .radius(1.0)
            .ballmode(beamlattice::BallMode::Mixed)
            .ballradius(0.5)
            .add_beam(0, 1)
            .add_balls(&[0, 1, 2, 3]);

        let beamlattice = builder.build();

        assert!(beamlattice.balls.is_some());
        let balls = beamlattice.balls.as_ref().unwrap();
        assert_eq!(balls.ball.len(), 4);
        assert_eq!(balls.ball[0].vindex, 0);
        assert_eq!(balls.ball[3].vindex, 3);
    }

    #[test]
    fn test_beam_lattice_with_multiple_beamsets() {
        let mut builder = BeamLatticeBuilder::new();
        builder
            .radius(1.0)
            .add_beams(&[(0, 1), (1, 2), (2, 3), (3, 0)])
            .add_beamset(|bs| {
                bs.name("Bottom")
                    .identifier("bottom-001")
                    .add_beam_refs(&[0, 1]);
            })
            .add_beamset(|bs| {
                bs.name("Top").identifier("top-001").add_beam_refs(&[2, 3]);
            });

        let beamlattice = builder.build();

        assert!(beamlattice.beamsets.is_some());
        let beamsets = beamlattice.beamsets.as_ref().unwrap();
        assert_eq!(beamsets.beamset.len(), 2);
        assert_eq!(beamsets.beamset[0].name, Some("Bottom".into()));
        assert_eq!(beamsets.beamset[0].refs.len(), 2);
        assert_eq!(beamsets.beamset[1].name, Some("Top".into()));
        assert_eq!(beamsets.beamset[1].refs.len(), 2);
    }

    #[test]
    fn test_mesh_with_beam_lattice() {
        let mut mesh_builder = MeshBuilder::new();
        mesh_builder
            .add_vertices(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]])
            .add_beam_lattice(|bl| {
                bl.minlength(0.001)
                    .radius(1.0)
                    .add_beam(0, 1)
                    .add_beam(1, 2);
            });

        let mesh = mesh_builder.build_mesh().unwrap();

        assert_eq!(mesh.vertices.vertex.len(), 3);
        assert!(mesh.beamlattice.is_some());

        let bl = mesh.beamlattice.as_ref().unwrap();
        assert_eq!(bl.minlength, 0.001);
        assert_eq!(bl.radius, 1.0);
        assert_eq!(bl.beams.beam.len(), 2);
    }

    #[test]
    fn test_model_with_beam_lattice() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);

        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("Lattice");
                obj.add_vertices(&[
                    [0.0, 0.0, 0.0],
                    [10.0, 0.0, 0.0],
                    [10.0, 10.0, 0.0],
                    [0.0, 10.0, 0.0],
                ])
                .add_beam_lattice(|bl| {
                    bl.minlength(0.001)
                        .radius(1.0)
                        .cap(beamlattice::CapMode::Sphere)
                        .add_beams(&[(0, 1), (1, 2), (2, 3), (3, 0)]);
                });
                Ok(())
            })
            .unwrap();

        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();

        let model = builder.build().unwrap();

        assert_eq!(model.resources.object.len(), 1);
        let obj = &model.resources.object[0];
        assert!(obj.get_mesh().is_some());

        let mesh = obj.get_mesh().unwrap();
        assert!(mesh.beamlattice.is_some());

        assert_eq!(
            model.requiredextensions,
            ThreemfExtensions::new(&[ThreemfNamespace::BeamLattice,])
        );
    }

    #[test]
    fn test_model_with_beam_lattice_and_balls() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);

        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("Lattice with Balls");
                obj.add_vertices(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]])
                    .add_beam_lattice(|bl| {
                        bl.minlength(0.001)
                            .radius(1.0)
                            .ballmode(beamlattice::BallMode::Mixed)
                            .ballradius(0.5)
                            .add_beams(&[(0, 1), (1, 2)])
                            .add_balls(&[0, 2]);
                    });
                Ok(())
            })
            .unwrap();

        builder.add_build(None).unwrap();
        builder.add_build_item(obj_id).unwrap();

        let model = builder.build().unwrap();
        let mesh = model.resources.object[0].get_mesh().unwrap();
        let bl = mesh.beamlattice.as_ref().unwrap();
        assert!(bl.balls.is_some());

        assert_eq!(
            model.requiredextensions,
            ThreemfExtensions::new(&[
                ThreemfNamespace::BeamLatticeBalls,
                ThreemfNamespace::BeamLattice,
            ])
        )
    }
}

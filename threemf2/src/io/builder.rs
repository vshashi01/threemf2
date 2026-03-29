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
//! # Basic Usage
//!
//! ```rust,ignore
//! use threemf2::io::builder::{ModelBuilder, Unit};
//!
//! // Create a root model
//! let mut builder = ModelBuilder::new(Unit::Millimeter, true);
//! builder.add_metadata("Application", Some("MyApp"));
//!
//! // Add a build section
//! builder.add_build(None)?;
//!
//! // Add a mesh object
//! let cube_id = builder.add_mesh_object(|obj| {
//!     obj.name("Cube");
//!     obj.add_vertices(&[
//!         [0.0, 0.0, 0.0],
//!         [10.0, 0.0, 0.0],
//!         [10.0, 10.0, 0.0],
//!         [0.0, 10.0, 0.0],
//!     ]);
//!     obj.add_triangles(&[
//!         [0, 1, 2],
//!         [0, 2, 3],
//!     ]);
//!     Ok(())
//! })?;
//!
//! // Add to build plate
//! builder.add_build_item(cube_id)?;
//!
//! // Build the final model
//! let model = builder.build()?;
//! ```
//!
//! # Boolean Operations Example
//!
//! ```rust,ignore
//! use threemf2::io::builder::{ModelBuilder, Unit, BooleanOperation};
//!
//! let mut builder = ModelBuilder::new(Unit::Millimeter, true);
//! builder.add_build(None)?;
//!
//! // Create base mesh (cube)
//! let cube_id = builder.add_mesh_object(|obj| {
//!     obj.name("Cube");
//!     // ... add cube geometry
//!     Ok(())
//! })?;
//!
//! // Create sphere mesh
//! let sphere_id = builder.add_mesh_object(|obj| {
//!     obj.name("Sphere");
//!     // ... add sphere geometry
//!     Ok(())
//! })?;
//!
//! // Create a boolean shape (cube minus sphere)
//! let result_id = builder.add_booleanshape_object(|obj| {
//!     obj.name("CubeMinusSphere");
//!     obj.base_object(cube_id, BooleanOperation::Difference);
//!     obj.add_boolean(sphere_id);
//!     Ok(())
//! })?;
//!
//! builder.add_build_item(result_id)?;
//! let model = builder.build()?;
//! ```
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
    core::{
        beamlattice::{Ball, BallRef, Balls, Beam, BeamLattice, BeamRef, BeamSet, BeamSets, Beams},
        boolean::{Boolean as BooleanOp, BooleanOperation, BooleanShape},
        build::{Build, Item},
        component::{Component, Components},
        mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
        metadata::Metadata,
        model::Model,
        object::Object,
        object_kind::ObjectKind,
        resources::Resources,
        transform::Transform,
    },
    io::XmlNamespace,
    threemf_namespaces::{
        self, BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_NS,
        BEAM_LATTICE_PREFIX, BOOLEAN_NS, BOOLEAN_PREFIX, PROD_NS, PROD_PREFIX,
    },
};

use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub use crate::core::beamlattice::{BallMode, CapMode, ClippingMode};
pub use crate::core::model::Unit;
pub use crate::core::object::ObjectType;
use crate::core::types::{OptionalResourceId, OptionalResourceIndex, ResourceId, ResourceIndex};

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
///
/// # Examples
///
/// ## Creating a simple root model with a mesh object
///
/// ```rust,ignore
/// use threemf2::io::builder::{ModelBuilder, Unit, ObjectType};
///
/// let mut builder = ModelBuilder::new(Unit::Millimeter, true);
///
/// // Add metadata
/// builder.add_metadata("Application", Some("MyApp"));
///
/// // Create build section
/// builder.add_build(None)?;
///
/// // Add a cube mesh object
/// let cube_id = builder.add_mesh_object(|obj| {
///     obj.name("Cube")
///        .object_type(ObjectType::Model);
///
///     // Add vertices
///     obj.add_vertices(&[
///         [0.0, 0.0, 0.0],
///         [10.0, 0.0, 0.0],
///         [10.0, 10.0, 0.0],
///         [0.0, 10.0, 0.0],
///     ]);
///
///     // Add triangles
///     obj.add_triangles(&[[0, 1, 2], [0, 2, 3]]);
///
///     Ok(())
/// })?;
///
/// // Add object to build plate
/// builder.add_build_item(cube_id)?;
///
/// // Build the final model
/// let model = builder.build()?;
/// ```
///
/// ## Creating a sub-model
///
/// ```rust,ignore
/// // Sub-models cannot have a build section
/// let mut builder = ModelBuilder::new(Unit::Millimeter, false);
///
/// let obj_id = builder.add_mesh_object(|obj| {
///     obj.name("SubModelPart");
///     obj.add_vertex(&[0.0, 0.0, 0.0]);
///     obj.add_vertex(&[1.0, 0.0, 0.0]);
///     obj.add_vertex(&[0.0, 1.0, 0.0]);
///     obj.add_triangle(&[0, 1, 2]);
///     Ok(())
/// })?;
///
/// let model = builder.build()?;
/// ```
///
/// ## Using the Production extension
///
/// ```rust,ignore
/// let mut builder = ModelBuilder::new(Unit::Millimeter, true);
///
/// // Enable Production extension - requires UUIDs on all objects and items
/// builder.make_production_extension_required()?;
///
/// builder.add_build(Some("build-uuid-12345".to_string()))?;
///
/// let obj_id = builder.add_mesh_object(|obj| {
///     obj.name("TrackedPart")
///        .uuid("object-uuid-67890");  // UUID required!
///     obj.add_vertex(&[0.0, 0.0, 0.0]);
///     obj.add_triangle(&[0, 1, 2]);
///     Ok(())
/// })?;
///
/// builder.add_build_item_advanced(obj_id, |item| {
///     item.uuid("item-uuid-abcdef");  // UUID required!
/// })?;
///
/// let model = builder.build()?;
/// ```
pub struct ModelBuilder {
    unit: Option<Unit>,
    requiredextensions: Vec<XmlNamespace>,
    recommendedextensions: Vec<XmlNamespace>,
    metadata: Vec<Metadata>,
    resources: ResourcesBuilder,
    build: Option<BuildBuilder>, //in submodels, Build item is not allowed

    // tracks if the model is intended as a root model
    // if true, Build is required
    // else adding Build is not allowed
    is_root: bool,

    // tracks next object id
    next_object_id: ObjectId,

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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create a root model in millimeters
    /// let builder = ModelBuilder::new(Unit::Millimeter, true);
    ///
    /// // Create a sub-model in inches
    /// let sub_builder = ModelBuilder::new(Unit::Inch, false);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut builder = ModelBuilder::new(Unit::Millimeter, true);
    ///
    /// // Enable production extension first
    /// builder.make_production_extension_required()?;
    ///
    /// // Now all objects and items require UUIDs
    /// let obj_id = builder.add_mesh_object(|obj| {
    ///     obj.uuid("unique-object-id");  // Required!
    ///     obj.name("Part");
    ///     Ok(())
    /// })?;
    /// ```
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
                && let ObjectKind::Components(components) = kind
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
    pub fn add_required_extension(&mut self, extension: XmlNamespace) -> &mut Self {
        self.requiredextensions.push(extension);
        self
    }

    /// Add a recommended 3MF extension to the model.
    ///
    /// Recommended extensions can be ignored by readers if not supported.
    /// Some extensions are automatically added based on features used (e.g., triangle sets).
    pub fn add_recommended_extension(&mut self, extension: XmlNamespace) -> &mut Self {
        self.recommendedextensions.push(extension);
        self
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
            name: name.to_owned(),
            preserve: None,
            value: value.map(|v| v.to_owned()),
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let cube_id = builder.add_mesh_object(|obj| {
    ///     obj.name("Cube")
    ///        .object_type(ObjectType::Model);
    ///
    ///     // Add geometry
    ///     obj.add_vertices(&[
    ///         [0.0, 0.0, 0.0],
    ///         [10.0, 0.0, 0.0],
    ///         [0.0, 10.0, 0.0],
    ///     ]);
    ///     obj.add_triangles(&[[0, 1, 2]]);
    ///
    ///     Ok(())
    /// })?;
    /// ```
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

        if let Some(kind) = &object.kind
            && let Some(mesh) = kind.get_mesh()
        {
            self.set_recommended_namespaces_for_mesh(mesh);
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // First create a mesh object
    /// let part_id = builder.add_mesh_object(|obj| {
    ///     obj.name("Part");
    ///     obj.add_vertex(&[0.0, 0.0, 0.0]);
    ///     obj.add_triangle(&[0, 1, 2]);
    ///     Ok(())
    /// })?;
    ///
    /// // Create an assembly that references the part multiple times
    /// let assembly_id = builder.add_components_object(|obj| {
    ///     obj.name("Assembly");
    ///
    ///     // Add first instance
    ///     obj.add_component(part_id);
    ///
    ///     // Add second instance with transform
    ///     obj.add_component_advanced(part_id, |comp| {
    ///         comp.transform(Transform([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0]));
    ///     });
    ///
    ///     Ok(())
    /// })?;
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // First create base and operand mesh objects
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Simple build without UUID
    /// builder.add_build(None)?;
    ///
    /// // With Production extension
    /// builder.make_production_extension_required()?;
    /// builder.add_build(Some("build-12345".to_string()))?;
    /// ```
    pub fn add_build(&mut self, uuid: Option<String>) -> Result<&mut Self, ModelError> {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// builder.add_build(None)?;
    /// let obj_id = builder.add_mesh_object(|obj| { /* ... */ Ok(()) })?;
    /// builder.add_build_item(obj_id)?;
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// builder.add_build_item_advanced(obj_id, |item| {
    ///     item.transform(Transform([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, 0.0]));
    ///     item.partnumber("PART-001");
    ///     item.uuid("item-uuid");  // Required if Production extension enabled
    /// })?;
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let model = builder.build()?;
    /// ```
    pub fn build(self) -> Result<Model, ModelError> {
        let required_extensions = self.process_required_extensions();

        let requiredextensions = get_extensions_definition(&required_extensions);
        let recommendedextensions = get_extensions_definition(&self.recommendedextensions);

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

    fn set_recommended_namespaces_for_mesh(&mut self, mesh: &Mesh) {
        use threemf_namespaces::{CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX};
        if mesh.trianglesets.is_some()
            && self
                .recommendedextensions
                .iter()
                .all(|ns| ns.uri == CORE_TRIANGLESET_NS)
        {
            self.recommendedextensions.push(XmlNamespace {
                prefix: Some(CORE_TRIANGLESET_PREFIX.to_owned()),
                uri: CORE_TRIANGLESET_NS.to_owned(),
            });
        }
    }

    fn process_required_extensions(&self) -> Vec<XmlNamespace> {
        let mut required_extensions = self.requiredextensions.clone();
        if self.is_production_ext_required {
            let is_prod_ext_set = required_extensions.iter().find(|ns| ns.uri == PROD_NS);
            if is_prod_ext_set.is_none() {
                required_extensions.push(XmlNamespace {
                    prefix: Some(PROD_PREFIX.to_owned()),
                    uri: PROD_NS.to_owned(),
                });
            }
        }

        let mut is_beam_lattice_required = false;
        let mut is_beam_lattice_balls_required = false;

        for object in &self.resources.objects {
            //early exit to speed up things
            if is_beam_lattice_balls_required {
                break;
            }

            if let Some(kind) = &object.kind
                && let Some(mesh) = kind.get_mesh()
                && let Some(beam_lattice) = &mesh.beamlattice
            {
                is_beam_lattice_required = true;
                is_beam_lattice_balls_required = beam_lattice.balls.is_some();
            }
        }

        if is_beam_lattice_required {
            let is_bl_ext_set = required_extensions
                .iter()
                .find(|ns| ns.uri == BEAM_LATTICE_NS);
            if is_bl_ext_set.is_none() {
                required_extensions.push(XmlNamespace {
                    prefix: Some(BEAM_LATTICE_PREFIX.to_owned()),
                    uri: BEAM_LATTICE_NS.to_owned(),
                });
            }

            if is_beam_lattice_balls_required {
                let is_bl_balls_ext_set = required_extensions
                    .iter()
                    .find(|ns| ns.uri == BEAM_LATTICE_BALLS_NS);
                if is_bl_balls_ext_set.is_none() {
                    required_extensions.push(XmlNamespace {
                        prefix: Some(BEAM_LATTICE_BALLS_PREFIX.to_owned()),
                        uri: BEAM_LATTICE_BALLS_NS.to_owned(),
                    });
                }
            }
        }

        // Detect boolean operations extension
        let is_boolean_required = self.resources.objects.iter().any(|obj| {
            if let Some(kind) = &obj.kind
                && kind.get_boolean_shape_object().is_some()
            {
                true
            } else {
                false
            }
        });

        if is_boolean_required {
            let is_boolean_ext_set = required_extensions.iter().find(|ns| ns.uri == BOOLEAN_NS);
            if is_boolean_ext_set.is_none() {
                required_extensions.push(XmlNamespace {
                    prefix: Some(BOOLEAN_PREFIX.to_owned()),
                    uri: BOOLEAN_NS.to_owned(),
                });
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

fn get_extensions_definition(extensions: &[XmlNamespace]) -> Option<String> {
    if extensions.is_empty() {
        None
    } else {
        let mut extension_string = String::new();
        let mut unique_namespaces: HashSet<XmlNamespace> = HashSet::new();

        for ns in extensions {
            unique_namespaces.insert(ns.clone());
        }

        for ns in unique_namespaces {
            if let Some(prefix) = &ns.prefix {
                extension_string.push_str(prefix);
                extension_string.push(' ');
            }
        }

        Some(extension_string)
    }
}

/// Builder for Resources
pub struct ResourcesBuilder {
    objects: Vec<Object>,
}

impl ResourcesBuilder {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    fn build(self) -> Resources {
        Resources {
            object: self.objects,
            basematerials: Vec::new(),
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
    uuid: Option<String>,
}

impl BuildBuilder {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            uuid: None,
        }
    }

    fn uuid(&mut self, uuid: String) -> &mut Self {
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
            uuid: None,
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
    path: Option<String>,
    uuid: Option<String>,
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Identity transform (no transformation)
    /// comp.transform(Transform([
    ///     1.0, 0.0, 0.0,
    ///     0.0, 1.0, 0.0,
    ///     0.0, 0.0, 1.0,
    ///     0.0, 0.0, 0.0
    /// ]));
    ///
    /// // Translate 10mm in X direction
    /// comp.transform(Transform([
    ///     1.0, 0.0, 0.0,
    ///     0.0, 1.0, 0.0,
    ///     0.0, 0.0, 1.0,
    ///     10.0, 0.0, 0.0
    /// ]));
    /// ```
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
        self.uuid = Some(uuid.to_owned());
        self
    }

    /// Set the path for this build item.
    ///
    /// Only allowed when Production extension is enabled. The path specifies
    /// an alternative model file where the referenced object can be found.
    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.to_owned());
        self
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
            partnumber: self.partnumber,
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

/// Builder for Object
pub struct ObjectBuilder<T> {
    entity: T,
    object_id: ObjectId,
    objecttype: Option<ObjectType>,
    thumbnail: Option<String>,
    partnumber: Option<String>,
    name: Option<String>,
    pid: OptionalResourceId,
    pindex: OptionalResourceIndex,
    uuid: Option<String>,

    // sets if the production ext is required.
    // if yes will ensure UUID is set before building the object
    is_production_ext_required: bool,
}

impl<T> ObjectBuilder<T> {
    /// Set the object type
    pub fn object_type(&mut self, object_type: ObjectType) -> &mut Self {
        self.objecttype = Some(object_type);
        self
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
        self.uuid = Some(uuid.to_owned());
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

/// Builder for creating mesh objects with triangle geometry.
///
/// `MeshObjectBuilder` combines object metadata (name, type, UUID, etc.) with
/// mesh geometry. Access mesh-building methods directly via [`Deref`] to [`MeshBuilder`].
///
/// # Examples
///
/// ```rust,ignore
/// let cube_id = builder.add_mesh_object(|obj| {
///     // Object properties
///     obj.name("Cube")
///        .object_type(ObjectType::Model)
///        .part_number("CUBE-001");
///
///     // Mesh geometry (via Deref to MeshBuilder)
///     obj.add_vertices(&[
///         [0.0, 0.0, 0.0],
///         [10.0, 0.0, 0.0],
///         [10.0, 10.0, 0.0],
///         [0.0, 10.0, 0.0],
///     ]);
///
///     obj.add_triangles(&[
///         [0, 1, 2],
///         [0, 2, 3],
///     ]);
///
///     Ok(())
/// })?;
/// ```
pub type MeshObjectBuilder = ObjectBuilder<MeshBuilder>;

impl MeshObjectBuilder {
    fn new(object_id: ObjectId, is_production_ext_required: bool) -> Self {
        Self {
            entity: MeshBuilder::new(),
            object_id,
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<Object, MeshObjectError> {
        let mesh = self.entity.build_mesh().unwrap();

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(MeshObjectError::ObjectUuidNotSet);
        }

        Ok(Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber,
            name: self.name,
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            kind: Some(ObjectKind::Mesh(mesh)),
            // mesh: Some(mesh),
            // components: None,
            // booleanshape: None,
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
///
/// # Examples
///
/// ```rust,ignore
/// obj.add_vertices(&[
///     [0.0, 0.0, 0.0],   // vertex 0
///     [10.0, 0.0, 0.0],  // vertex 1
///     [10.0, 10.0, 0.0], // vertex 2
/// ]);
///
/// obj.add_triangles(&[
///     [0, 1, 2],  // Triangle using vertices 0, 1, 2
/// ]);
/// ```
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    triangles: Vec<Triangle>,
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_vertex(&[0.0, 0.0, 0.0])
    ///    .add_vertex(&[10.0, 0.0, 0.0])
    ///    .add_vertex(&[0.0, 10.0, 0.0]);
    /// ```
    pub fn add_vertex(&mut self, coords: &[f64; 3]) -> &mut Self {
        self.vertices
            .push(Vertex::new(coords[0], coords[1], coords[2]));
        self
    }

    /// Add multiple vertices from a slice of coordinate arrays.
    ///
    /// Each element should be a 3D coordinate `[x, y, z]`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_vertices(&[
    ///     [0.0, 0.0, 0.0],
    ///     [10.0, 0.0, 0.0],
    ///     [10.0, 10.0, 0.0],
    /// ]);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_vertices_flat(&[
    ///     0.0, 0.0, 0.0,     // vertex 0
    ///     10.0, 0.0, 0.0,    // vertex 1
    ///     10.0, 10.0, 0.0,   // vertex 2
    /// ]);
    /// ```
    pub fn add_vertices_flat(&mut self, vertices: &[f64]) -> &mut Self {
        for vertex in vertices.chunks_exact(3) {
            self.vertices
                .push(Vertex::new(vertex[0], vertex[1], vertex[2]));
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_vertex(&[0.0, 0.0, 0.0]);    // index 0
    /// obj.add_vertex(&[10.0, 0.0, 0.0]);   // index 1
    /// obj.add_vertex(&[0.0, 10.0, 0.0]);   // index 2
    /// obj.add_triangle(&[0, 1, 2]);
    /// ```
    pub fn add_triangle(&mut self, indices: &[usize; 3]) -> &mut Self {
        self.triangles.push(Triangle {
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

    /// Add multiple triangles from a slice of index arrays.
    ///
    /// Each element should be a triangle with three vertex indices.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_triangles(&[
    ///     [0, 1, 2],
    ///     [0, 2, 3],
    ///     [0, 3, 4],
    /// ]);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_triangles_flat(&[
    ///     0, 1, 2,  // triangle 0
    ///     0, 2, 3,  // triangle 1
    /// ]);
    /// ```
    pub fn add_triangles_flat(&mut self, triangles: &[usize]) -> &mut Self {
        for triangle in triangles.chunks_exact(3) {
            self.triangles.push(Triangle {
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_triangle_sets(|sets| {
    ///     sets.add_set("TopFace", "top-id", &[0, 1], &[]);
    ///     sets.add_set("BottomFace", "bottom-id", &[2, 3], &[]);
    /// });
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// obj.add_beam_lattice(|lattice| {
    ///     lattice.radius(0.5)
    ///            .add_beam(0, 1)
    ///            .add_beam(1, 2);
    /// });
    /// ```
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

    fn build_mesh(self) -> Result<Mesh, MeshObjectError> {
        let trianglesets = self.triangle_sets.map(|b| b.build());
        let beamlattice = self.beam_lattice.map(|b| b.build());
        Ok(Mesh {
            vertices: Vertices {
                vertex: self.vertices,
            },
            triangles: Triangles {
                triangle: self.triangles,
            },
            trianglesets,
            beamlattice,
        })
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
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<Object, ComponentsObjectError> {
        let components = self
            .entity
            .build_components(self.is_production_ext_required)?;

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(ComponentsObjectError::ObjectUuidNotSet);
        }

        Ok(Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber,
            name: self.name,
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            kind: Some(ObjectKind::Components(components)), // mesh: None,
                                                            // components: Some(components),
                                                            // booleanshape: None,
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
    path: Option<String>,
    uuid: Option<String>,
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
        self.uuid = Some(uuid.to_owned());
        self
    }

    /// Set the path for this component.
    ///
    /// Only allowed when Production extension is enabled. The path specifies
    /// an alternative model file where the referenced object can be found.
    pub fn path(&mut self, path: &str) -> &mut Self {
        self.path = Some(path.to_owned());
        self
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
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            is_production_ext_required,
        }
    }

    fn build(self) -> Result<Object, BooleanShapeError> {
        let boolean_shape = self.entity.build_boolean_shape()?;

        if self.is_production_ext_required && self.uuid.is_none() {
            return Err(BooleanShapeError::ObjectUuidNotSet);
        }

        Ok(Object {
            id: self.object_id.0,
            objecttype: self.objecttype,
            thumbnail: self.thumbnail,
            partnumber: self.partnumber,
            name: self.name,
            pid: self.pid,
            pindex: self.pindex,
            uuid: self.uuid,
            kind: Some(ObjectKind::BooleanShape(boolean_shape)),
            // mesh: None,
            // components: None,
            // booleanshape: Some(boolean_shape),
        })
    }
}

/// Builder for configuring boolean operations in a boolean shape object.
///
/// `BooleanShapeBuilder` manages the base object reference, boolean operation type,
/// and the list of operand objects for CSG operations.
pub struct BooleanShapeBuilder {
    base_object_id: Option<ObjectId>,
    operation: BooleanOperation,
    base_transform: Option<Transform>,
    base_path: Option<String>,
    booleans: Vec<BooleanOp>,
}

impl BooleanShapeBuilder {
    fn new() -> Self {
        Self {
            base_object_id: None,
            operation: BooleanOperation::Union,
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
    pub fn base_object(&mut self, object_id: ObjectId, operation: BooleanOperation) -> &mut Self {
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
        self.base_path = Some(path.to_owned());
        self
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

    fn build_boolean_shape(self) -> Result<BooleanShape, BooleanShapeError> {
        let base_object_id = self
            .base_object_id
            .ok_or(BooleanShapeError::BaseObjectNotSet)?;

        if self.booleans.is_empty() {
            return Err(BooleanShapeError::NoBooleanOperands);
        }

        Ok(BooleanShape {
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
    path: Option<String>,
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
        self.path = Some(path.to_owned());
        self
    }

    fn build(self) -> BooleanOp {
        BooleanOp {
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
///
/// # Examples
///
/// ```rust,ignore
/// obj.add_triangle_sets(|sets| {
///     // Add a set referencing specific triangle indices
///     sets.add_set("TopFace", "top-id", &[0, 1, 2], &[]);
///
///     // Add a set using a range of triangles
///     sets.add_set("SideFaces", "side-id", &[], &[(3, 10)]);
///
///     // Mix individual refs and ranges
///     sets.add_set("Mixed", "mixed-id", &[11, 12], &[(20, 30), (40, 50)]);
/// });
/// ```
pub struct TriangleSetsBuilder {
    sets: Vec<crate::core::triangle_set::TriangleSet>,
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
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Reference triangles 0, 1, and 2 individually
    /// sets.add_set("Group1", "g1", &[0, 1, 2], &[]);
    ///
    /// // Reference triangles 10-20 (inclusive)
    /// sets.add_set("Group2", "g2", &[], &[(10, 20)]);
    ///
    /// // Mix both approaches
    /// sets.add_set("Group3", "g3", &[5, 6], &[(10, 15), (20, 25)]);
    /// ```
    pub fn add_set(
        &mut self,
        name: &str,
        identifier: &str,
        refs: &[u32],
        ranges: &[(u32, u32)],
    ) -> &mut Self {
        use crate::core::triangle_set::{TriangleRef, TriangleRefRange, TriangleSet};

        let triangle_ref = refs.iter().map(|&index| TriangleRef { index }).collect();
        let triangle_refrange = ranges
            .iter()
            .map(|&(start, end)| TriangleRefRange {
                startindex: start,
                endindex: end,
            })
            .collect();

        self.sets.push(TriangleSet {
            name: name.to_owned(),
            identifier: identifier.to_owned(),
            triangle_ref,
            triangle_refrange,
        });
        self
    }

    fn build(self) -> crate::core::triangle_set::TriangleSets {
        crate::core::triangle_set::TriangleSets {
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
    ballmode: Option<BallMode>,
    ballradius: Option<f64>,
    clippingmode: Option<ClippingMode>,
    clippingmesh: OptionalResourceId,
    representationmesh: OptionalResourceId,
    pid: OptionalResourceId,
    pindex: OptionalResourceIndex,
    cap: Option<CapMode>,
    beams: Vec<Beam>,
    balls: Vec<Ball>,
    beamsets: Vec<BeamSet>,
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
    pub fn ballmode(&mut self, mode: BallMode) -> &mut Self {
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
    pub fn clippingmode(&mut self, mode: ClippingMode) -> &mut Self {
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
    pub fn cap(&mut self, cap: CapMode) -> &mut Self {
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

    fn build(self) -> BeamLattice {
        let beams = Beams { beam: self.beams };

        let balls = if self.balls.is_empty() {
            None
        } else {
            Some(Balls { ball: self.balls })
        };

        let beamsets = if self.beamsets.is_empty() {
            None
        } else {
            Some(BeamSets {
                beamset: self.beamsets,
            })
        };

        BeamLattice {
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
    cap1: Option<CapMode>,
    cap2: Option<CapMode>,
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
    pub fn cap_1(mut self, cap: CapMode) -> Self {
        self.cap1 = Some(cap);
        self
    }

    /// Set the cap mode for the second end of the beam
    pub fn cap_2(mut self, cap: CapMode) -> Self {
        self.cap2 = Some(cap);
        self
    }

    fn build(self) -> Beam {
        Beam {
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

    fn build(self) -> Ball {
        Ball {
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

    fn build(self) -> BeamSet {
        BeamSet {
            name: self.name,
            identifier: self.identifier,
            refs: self
                .beam_refs
                .into_iter()
                .map(|index| BeamRef { index })
                .collect(),
            ballref: self
                .ball_refs
                .into_iter()
                .map(|index| BallRef { index })
                .collect(),
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
                obj.object_type(ObjectType::Model);
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
        assert_eq!(model.metadata[0].name, "Application");
        assert_eq!(model.resources.object.len(), 1);
        assert_eq!(model.resources.object[0].name, Some("Cube".to_owned()));
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
        builder.add_build(Some("build-uuid".to_string())).unwrap();
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
        assert_eq!(model.requiredextensions, Some("p ".to_string()));
    }

    #[test]
    fn test_production_ext_requires_object_uuid() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.make_production_extension_required().unwrap(); // should not return error;
        builder.add_build(Some("build-uuid".to_string())).unwrap();
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
        builder.add_build(Some("build-uuid".to_string())).unwrap();
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

        builder.add_build(Some("build-uuid".to_owned())).unwrap();
        if let Err(err) = builder.add_build_item_advanced(mesh_obj_id, |i| {
            i.uuid("some-uuid").path("some-path");
        }) {
            panic!("{err:?}");
        }
    }

    #[test]
    fn test_extension_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.add_required_extension(crate::io::XmlNamespace {
            prefix: Some("test".to_string()),
            uri: "http://example.com/test".to_string(),
        });
        builder.add_recommended_extension(crate::io::XmlNamespace {
            prefix: Some("rec".to_string()),
            uri: "http://example.com/rec".to_string(),
        });
        builder.add_build(None).unwrap();
        let model = builder.build().unwrap();
        assert_eq!(model.requiredextensions, Some("test ".to_string()));
        assert_eq!(model.recommendedextensions, Some("rec ".to_string()));
    }

    #[test]
    fn test_build_item_advanced_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let _ = builder.make_production_extension_required();
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.name("test").uuid("obj-uuid");
                Ok(())
            })
            .unwrap();
        builder.add_build(Some("build-uuid".to_owned())).unwrap();

        let transform = crate::core::transform::Transform([
            1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        ]);
        builder
            .add_build_item_advanced(obj_id, |i| {
                i.transform(transform.clone())
                    .partnumber("part")
                    .path("path")
                    .uuid("uuid");
            })
            .unwrap();

        let model = builder.build().unwrap();
        let item = &model.build.item[0];
        assert_eq!(item.objectid, 1);
        assert_eq!(item.transform, Some(transform));
        assert_eq!(item.partnumber, Some("part".to_string()));
        assert_eq!(item.path, Some("path".to_string()));
        assert_eq!(item.uuid, Some("uuid".to_string()));
    }

    #[test]
    fn test_object_builder_tests() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let obj_id = builder
            .add_mesh_object(|obj| {
                obj.object_type(crate::core::object::ObjectType::Support);
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
        assert_eq!(obj.name, Some("support obj".to_string()));
        assert_eq!(obj.partnumber, Some("part123".to_string()));
        assert_eq!(obj.uuid, Some("obj-uuid".to_string()));
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
        let mesh = model.resources.object[0]
            .kind
            .as_ref()
            .unwrap()
            .get_mesh()
            .unwrap();
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
        assert!(obj.kind.as_ref().unwrap().get_components_object().is_some());
        let comp = &obj
            .kind
            .as_ref()
            .unwrap()
            .get_components_object()
            .unwrap()
            .component[0];
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
        assert_eq!(model.metadata[0].name, "key1");
        assert_eq!(model.metadata[0].value, Some("value1".to_string()));
        assert_eq!(model.metadata[1].name, "key2");
        assert_eq!(model.metadata[1].value, None);
        assert_eq!(model.metadata[2].name, "key3");
        assert_eq!(model.metadata[2].value, Some("value3".to_string()));
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
        let mesh = model.resources.object[0]
            .kind
            .as_ref()
            .unwrap()
            .get_mesh()
            .unwrap();
        assert!(mesh.trianglesets.is_some());
        let sets = &mesh.trianglesets.as_ref().unwrap().trianglesets;
        assert_eq!(sets.len(), 2);
        assert_eq!(sets[0].name, "Set1");
        assert_eq!(sets[0].identifier, "id1");
        assert_eq!(sets[0].triangle_ref.len(), 1);
        assert_eq!(sets[0].triangle_ref[0].index, 0);
        assert_eq!(sets[0].triangle_refrange.len(), 1);
        assert_eq!(sets[0].triangle_refrange[0].startindex, 1);
        assert_eq!(sets[0].triangle_refrange[0].endindex, 5);
        assert_eq!(sets[1].name, "Set2");
        assert_eq!(sets[1].identifier, "id2");
        assert_eq!(sets[1].triangle_ref.len(), 0);
        assert_eq!(sets[1].triangle_refrange.len(), 2);

        //check if Triangle set is in recommended extensions
        assert_eq!(model.recommendedextensions, Some("t ".to_owned()))
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
            .cap_1(CapMode::Hemisphere)
            .cap_2(CapMode::Butt)
            .build();

        assert_eq!(beam.v1, 0);
        assert_eq!(beam.v2, 1);
        assert_eq!(beam.r1, Some(1.5));
        assert_eq!(beam.r2, Some(2.0));
        assert_eq!(beam.p1, OptionalResourceIndex::new(10));
        assert_eq!(beam.p2, OptionalResourceIndex::new(20));
        assert_eq!(beam.pid, OptionalResourceId::new(5));
        assert_eq!(beam.cap1, Some(CapMode::Hemisphere));
        assert_eq!(beam.cap2, Some(CapMode::Butt));
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

        assert_eq!(beamset.name, Some("Test Set".to_owned()));
        assert_eq!(beamset.identifier, Some("test-set-001".to_owned()));
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
            .ballmode(BallMode::Mixed)
            .ballradius(0.5)
            .add_beam(0, 1)
            .add_ball(0)
            .add_ball_advanced(1, |b| b.radius(0.75));

        let beamlattice = builder.build();

        assert_eq!(beamlattice.ballmode, Some(BallMode::Mixed));
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
            .ballmode(BallMode::All)
            .ballradius(1.0)
            .clippingmode(ClippingMode::Inside)
            .clippingmesh(clip_mesh_id)
            .representationmesh(repr_mesh_id)
            .pid(5)
            .pindex(10)
            .cap(CapMode::Sphere)
            .add_beam(0, 1);

        let beamlattice = builder.build();

        assert_eq!(beamlattice.minlength, 0.002);
        assert_eq!(beamlattice.radius, 2.0);
        assert_eq!(beamlattice.ballmode, Some(BallMode::All));
        assert_eq!(beamlattice.ballradius, Some(1.0));
        assert_eq!(beamlattice.clippingmode, Some(ClippingMode::Inside));
        assert_eq!(beamlattice.clippingmesh, OptionalResourceId::new(10));
        assert_eq!(beamlattice.representationmesh, OptionalResourceId::new(20));
        assert_eq!(beamlattice.pid, OptionalResourceId::new(5));
        assert_eq!(beamlattice.pindex, OptionalResourceIndex::new(10));
        assert_eq!(beamlattice.cap, Some(CapMode::Sphere));
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
            .ballmode(BallMode::Mixed)
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
        assert_eq!(beamsets.beamset[0].name, Some("Bottom".to_owned()));
        assert_eq!(beamsets.beamset[0].refs.len(), 2);
        assert_eq!(beamsets.beamset[1].name, Some("Top".to_owned()));
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
                        .cap(CapMode::Sphere)
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
        assert!(obj.kind.as_ref().unwrap().get_mesh().is_some());

        let mesh = obj.kind.as_ref().unwrap().get_mesh().unwrap();
        assert!(mesh.beamlattice.is_some());

        assert_eq!(model.requiredextensions, Some("b ".to_owned()));
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
                            .ballmode(BallMode::Mixed)
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
        let mesh = model.resources.object[0]
            .kind
            .as_ref()
            .unwrap()
            .get_mesh()
            .unwrap();
        let bl = mesh.beamlattice.as_ref().unwrap();
        assert!(bl.balls.is_some());

        let prefixes = ["b", "b2"];
        if let Some(exts) = &model.requiredextensions {
            let split_exts = exts.split_whitespace().collect::<Vec<_>>();
            for prefix in prefixes {
                assert!(split_exts.contains(&prefix));
            }
        }
    }
}

//! Query API for inspecting 3MF packages and models.
//!
//! This module provides helper functions and reference types for querying objects, build items,
//! and other entities within 3MF packages. It supports eager-loaded [`ThreemfPackage`] instances
//! and handles multi-model packages (root model + sub-models) seamlessly.
//!
//! # Overview
//!
//! The query API is organized around lightweight reference types that wrap entities with
//! additional context like the originating model path:
//!
//! - [`ObjectRef`] - References to any object (mesh, composed, or boolean shape)
//! - [`MeshObjectRef`] - References to mesh objects with triangle geometry
//! - [`ComponentsObjectRef`] - References to Components Object (assemblies)
//! - [`BooleanShapeRef`] - References to Boolean Shape objects (CSG operations)
//! - [`ItemRef`] - References to build items (objects to be manufactured)
//! - [`ComponentRef`] - References to components within composed parts
//! - [`BooleanRef`] - References to boolean operands within boolean shapes
//! - [`ModelRef`] - References to models with their path information
//!
//! # Common Patterns
//!
//! ## Iterating All Objects
//!
//! ```rust,ignore
//! use threemf2::io::{ThreemfPackage, query::*};
//!
//! let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
//!
//! // Iterate all objects across root and sub-models
//! for obj_ref in get_objects(&package) {
//!     println!("Object ID: {}", obj_ref.object.id);
//!     if let Some(path) = obj_ref.path {
//!         println!("  From sub-model: {}", path);
//!     }
//! }
//! ```
//!
//! ## Finding Build Items
//!
//! ```rust,ignore
//! // Get all build items
//! for item in get_items(&package) {
//!     println!("Item references object {}", item.objectid());
//!     if let Some(transform) = item.transform() {
//!         println!("  With transform: {:?}", transform);
//!     }
//! }
//!
//! // Find items that reference a specific object
//! for item in get_items_by_objectid(&package, 42) {
//!     println!("Found item referencing object 42");
//! }
//! ```
//!
//! ## Working with Mesh Objects
//!
//! ```rust,ignore
//! // Get only mesh objects (filters out composed parts)
//! for mesh_ref in get_mesh_objects(&package) {
//!     let mesh = mesh_ref.mesh();
//!     println!("Mesh with {} vertices, {} triangles",
//!         mesh.vertices.vertex.len(),
//!         mesh.triangles.triangle.len()
//!     );
//!     
//!     // Access object metadata via Deref
//!     if let Some(name) = &mesh_ref.name {
//!         println!("  Name: {}", name);
//!     }
//! }
//! ```
//!
//! ## Traversing Composed Parts
//!
//! ```rust,ignore
//! // Get composed objects (assemblies)
//! for composed in get_components_objects(&package) {
//!     println!("Assembly ID: {}", composed.id);
//!     
//!     // Iterate components within this assembly
//!     for component in composed.components() {
//!         println!("  References object {}", component.objectid);
//!         if let Some(path) = &component.path_to_look_for {
//!             println!("    In model: {}", path);
//!         }
//!     }
//! }
//! ```
//!
//! ## Working with Multi-Model Packages
//!
//! ```rust,ignore
//! // Iterate all models (root + sub-models)
//! for model_ref in iter_models(&package) {
//!     if let Some(path) = model_ref.path {
//!         println!("Sub-model at: {}", path);
//!     } else {
//!         println!("Root model");
//!     }
//!     
//!     // Query objects in this specific model
//!     for obj in get_objects_from_model(model_ref.model) {
//!         println!("  Object {}", obj.object.id);
//!     }
//! }
//! ```
//!
//! ## Working with Boolean Shape Objects
//!
//! Boolean shapes define objects through constructive solid geometry (CSG) operations:
//!
//! ```rust,ignore
//! // Get all boolean shape objects
//! for boolean_ref in get_boolean_shape_objects(&package) {
//!     println!("Boolean shape {}:", boolean_ref.id);
//!     println!("  Base object: {}", boolean_ref.base_objectid());
//!     println!("  Operation: {:?}", boolean_ref.operation());
//!     
//!     // Check operation type
//!     if boolean_ref.is_difference() {
//!         println!("  Subtracting volumes");
//!     } else if boolean_ref.is_union() {
//!         println!("  Merging volumes");
//!     } else if boolean_ref.is_intersection() {
//!         println!("  Keeping intersection");
//!     }
//!     
//!     // List operands
//!     for operand in boolean_ref.booleans() {
//!         println!("  Operand: object {}", operand.objectid);
//!         if let Some(path) = &operand.path {
//!             println!("    From model: {}", path);
//!         }
//!     }
//! }
//!
//! // Find all difference operations
//! let diff_count = get_boolean_shape_objects(&package)
//!     .filter(|b| b.is_difference())
//!     .count();
//! println!("Found {} subtraction operations", diff_count);
//! ```
//!
//! # Production Extension Support
//!
//! The 3MF Production extension adds UUIDs for tracking objects and items through
//! manufacturing workflows. When present, you can query by UUID:
//!
//! ```rust,ignore
//! // Find build item by UUID
//! if let Some(item) = get_item_by_uuid(&package, "550e8400-e29b-41d4-a716-446655440000") {
//!     println!("Found item with UUID");
//! }
//!
//! // Access UUIDs on items and objects
//! for item in get_items(&package) {
//!     if let Some(uuid) = item.uuid() {
//!         println!("Item UUID: {}", uuid);
//!     }
//! }
//! ```
//!
//! # Reference Types and Model Paths
//!
//! Reference types like [`ObjectRef`] and [`ItemRef`] include path information to track
//! which model an entity came from:
//!
//! - `path: None` or `origin_model_path: None` - Entity is from the root model
//! - `path: Some("path/to/model.model")` - Entity is from a sub-model
//!
//! This is essential for resolving cross-model references in the production extension,
//! where components can reference objects in different model files.
//!
//! # Model-Level vs Package-Level Queries
//!
//! Most query functions come in two variants:
//!
//! - **Package-level** (e.g., [`get_items`]) - Query across all models (root + sub-models)
//! - **Model-level** (e.g., [`get_items_from_model`]) - Query a single model
//!
//! Use package-level queries for most cases. Use model-level queries when you need
//! fine-grained control or are working with a specific model instance.
//!
//! # Performance Considerations
//!
//! - All query functions return iterators, enabling lazy evaluation
//! - Reference types are lightweight wrappers with no data copying
//! - Queries work directly on the loaded package data with no additional allocations
//!
//! # See Also
//!
//! - [`ThreemfPackage`] - The eager-loaded package type these queries work with
//! - [`examples/query_example.rs`](https://github.com/vshashi01/threemf2/blob/main/examples/query_example.rs) - Complete usage examples

#![allow(clippy::needless_lifetimes)]

use std::ops::Deref;

use crate::{
    core::{
        OptionalResourceId, OptionalResourceIndex,
        boolean::{BooleanOperation, BooleanShape},
        build::Item,
        component::Components,
        mesh::Mesh,
        model::Model,
        object::{Object, ObjectKind, ObjectType},
        transform::Transform,
    },
    io::ThreemfPackage,
};

/// A reference to an object within a 3MF model, including its path if from a sub-model.
///
/// Objects are the primary resources in 3MF models and can be either mesh objects
/// (containing triangle geometry) or composed parts (assemblies of other objects).
///
/// # Fields
///
/// * `object` - Reference to the underlying [`Object`] data
/// * `path` - Path to the model containing this object (`None` for root model)
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for obj_ref in get_objects(&package) {
///     println!("Object ID: {}", obj_ref.object.id);
///     
///     if let Some(name) = &obj_ref.object.name {
///         println!("  Name: {}", name);
///     }
///     
///     if let Some(path) = obj_ref.path {
///         println!("  From sub-model: {}", path);
///     }
///     
///     // Check what type of object this is
///     if obj_ref.object.mesh.is_some() {
///         println!("  Type: Mesh object");
///     } else if obj_ref.object.components.is_some() {
///         println!("  Type: Composed part");
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`MeshObjectRef`] - Specialized reference for mesh objects
/// * [`ComponentsObjectRef`] - Specialized reference for composed parts
/// * [`get_objects()`] - Get all objects from a package
pub struct ObjectRef<'a> {
    /// The object itself.
    pub object: &'a Object,
    /// The path to the model containing this object, if None then it is the root model.
    pub path: Option<&'a str>,
}

/// Retrieves an object by ID from a given model.
///
/// Object IDs are unique within a single model but may be duplicated across
/// different sub-models. This function only searches within the specified model.
///
/// # Arguments
///
/// * `object_id` - The object ID to search for
/// * `model` - The model to search in
///
/// # Returns
///
/// `Some(ObjectRef)` if found, `None` otherwise. The returned reference will
/// have `path` set to `None` since this is a single-model query.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Find object in root model
/// if let Some(obj) = get_object_from_model(42, &package.root) {
///     println!("Found object 42: {:?}", obj.object.name);
/// }
///
/// // Find object in a specific sub-model
/// if let Some(model) = package.sub_models.get("/3D/Objects/parts.model") {
///     if let Some(obj) = get_object_from_model(1, model) {
///         println!("Found object 1 in sub-model");
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_objects()`] - Search across all models in a package
pub fn get_object_from_model<'a>(object_id: u32, model: &'a Model) -> Option<ObjectRef<'a>> {
    model
        .resources
        .object
        .iter()
        .find(|o| o.id == object_id)
        .map(|object| ObjectRef { object, path: None })
}

/// Returns an iterator over all objects in the package, including sub-models.
///
/// Objects are the primary resources in 3MF and can be mesh objects (with triangle
/// geometry) or composed parts (assemblies). This function traverses all models
/// (root + sub-models) and returns every object with path tracking.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
///
/// # Returns
///
/// An iterator over [`ObjectRef`] for all objects in the package.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Count objects by type
/// let mut mesh_count = 0;
/// let mut composed_count = 0;
///
/// for obj_ref in get_objects(&package) {
///     if obj_ref.object.mesh.is_some() {
///         mesh_count += 1;
///     } else if obj_ref.object.components.is_some() {
///         composed_count += 1;
///     }
/// }
///
/// println!("Mesh objects: {}", mesh_count);
/// println!("Composed parts: {}", composed_count);
///
/// // Find objects by name
/// for obj_ref in get_objects(&package) {
///     if let Some(name) = &obj_ref.object.name {
///         if name.contains("gear") {
///             println!("Found gear part: {} (ID: {})", name, obj_ref.object.id);
///         }
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_mesh_objects()`] - Get only mesh objects (filters out composed parts)
/// * [`get_components_objects()`] - Get only composed parts
/// * [`get_objects_from_model()`] - Query a specific model
pub fn get_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ObjectRef<'a>> {
    iter_objects_from(package, get_objects_from_model_ref)
}

/// Returns an iterator over all objects in a specific model.
///
/// Unlike [`get_objects()`], this only queries a single model instance.
/// The returned objects will have `path` set to `None`.
///
/// # Arguments
///
/// * `model` - The model to query
///
/// # Returns
///
/// An iterator over [`ObjectRef`] for objects in this model only.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Get objects only from root model
/// let root_objects: Vec<_> = get_objects_from_model(&package.root).collect();
/// println!("Root model has {} objects", root_objects.len());
///
/// // Compare with sub-model objects
/// for (path, model) in &package.sub_models {
///     let sub_objects = get_objects_from_model(model).count();
///     println!("Sub-model {} has {} objects", path, sub_objects);
/// }
/// ```
///
/// # See Also
///
/// * [`get_objects()`] - Query all objects across all models
pub fn get_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ObjectRef<'a>> {
    get_objects_from_model_ref(ModelRef { model, path: None })
}

/// Returns an iterator over all objects in the model reference.
///
/// This is an internal helper function that preserves model path information.
/// Most users should use [`get_objects()`] or [`get_objects_from_model()`] instead.
pub fn get_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// A generic reference to an object entity with common metadata fields.
pub struct GenericObjectRef<'a, T> {
    /// The entity itself (e.g., Mesh, Components).
    entity: &'a T,
    pub id: u32,
    pub object_type: ObjectType,
    pub thumbnail: Option<String>,
    pub part_number: Option<String>,
    pub name: Option<String>,
    pub pid: OptionalResourceId,
    pub pindex: OptionalResourceIndex,
    pub uuid: Option<String>,
    /// Path to the originating model.
    pub origin_model_path: Option<&'a str>,
}

/// A reference to a mesh object with convenient access to both mesh data and object metadata.
///
/// Mesh objects contain triangle geometry and are the primary printable entities in 3MF.
/// This type provides direct access to the [`Mesh`] data plus all object metadata through
/// the [`Deref`] trait.
///
/// # Accessing Data
///
/// * Call [`mesh()`](MeshObjectRef::mesh) to get the mesh geometry
/// * Access object metadata directly (id, name, uuid, etc.) via [`Deref`]
/// * Check `origin_model_path` to see which model the object came from
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for mesh_ref in get_mesh_objects(&package) {
///     // Access object metadata via Deref
///     println!("Object ID: {}", mesh_ref.id);
///     if let Some(name) = &mesh_ref.name {
///         println!("  Name: {}", name);
///     }
///     
///     // Access mesh geometry
///     let mesh = mesh_ref.mesh();
///     println!("  Vertices: {}", mesh.vertices.vertex.len());
///     println!("  Triangles: {}", mesh.triangles.triangle.len());
///     
///     // Check for beam lattice
///     if let Some(beamlattice) = &mesh.beamlattice {
///         println!("  Has beam lattice with {} beams", beamlattice.beams.beam.len());
///     }
///     
///     // Check model origin
///     if let Some(path) = mesh_ref.origin_model_path {
///         println!("  From: {}", path);
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_mesh_objects()`] - Get all mesh objects from a package
/// * [`Mesh`] - The mesh geometry type
/// * [`ObjectRef`] - Generic object reference (includes composed parts)
pub struct MeshObjectRef<'a>(GenericObjectRef<'a, Mesh>);

impl<'a> MeshObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        MeshObjectRef(GenericObjectRef {
            entity: o.object.get_mesh().unwrap(),
            id: o.object.id,
            object_type: o.object.objecttype.unwrap_or(ObjectType::Model),
            thumbnail: o.object.thumbnail.clone(),
            part_number: o.object.partnumber.clone(),
            name: o.object.name.clone(),
            pid: o.object.pid,
            pindex: o.object.pindex,
            uuid: o.object.uuid.clone(),
            origin_model_path: o.path,
        })
    }

    /// Returns a reference to the mesh geometry data.
    ///
    /// The mesh contains vertices, triangles, and optional extensions like
    /// beam lattice structures or triangle sets.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for mesh_ref in get_mesh_objects(&package) {
    ///     let mesh = mesh_ref.mesh();
    ///     
    ///     // Access vertices
    ///     for vertex in &mesh.vertices.vertex {
    ///         println!("Vertex: ({}, {}, {})", vertex.x, vertex.y, vertex.z);
    ///     }
    ///     
    ///     // Access triangles
    ///     for triangle in &mesh.triangles.triangle {
    ///         println!("Triangle: ({}, {}, {})", triangle.v1, triangle.v2, triangle.v3);
    ///     }
    /// }
    /// ```
    pub fn mesh(&self) -> &'a Mesh {
        self.entity
    }
}

impl<'a> Deref for MeshObjectRef<'a> {
    type Target = GenericObjectRef<'a, Mesh>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Returns an iterator over mesh objects in the package.
///
/// Filters out composed parts and returns only objects containing triangle geometry.
/// Mesh objects are the primary printable entities in 3MF.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
///
/// # Returns
///
/// An iterator over [`MeshObjectRef`] for all mesh objects.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Analyze mesh complexity
/// for mesh_ref in get_mesh_objects(&package) {
///     let mesh = mesh_ref.mesh();
///     let vertex_count = mesh.vertices.vertex.len();
///     let triangle_count = mesh.triangles.triangle.len();
///     
///     println!("Mesh {} ({:?}): {} vertices, {} triangles",
///         mesh_ref.id,
///         mesh_ref.name,
///         vertex_count,
///         triangle_count
///     );
///     
///     // Check for material properties
///     if mesh_ref.pid.is_some() {
///         println!("  Has material assigned");
///     }
/// }
///
/// // Find meshes with beam lattice
/// let lattice_count = get_mesh_objects(&package)
///     .filter(|m| m.mesh().beamlattice.is_some())
///     .count();
/// println!("Objects with beam lattice: {}", lattice_count);
/// ```
///
/// # See Also
///
/// * [`get_objects()`] - Get all objects (includes composed parts)
/// * [`get_components_objects()`] - Get only composed parts
/// * [`MeshObjectRef`] - The reference type returned
pub fn get_mesh_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = MeshObjectRef<'a>> {
    iter_objects_from(package, get_mesh_objects_from_model_ref).map(MeshObjectRef::new)
}

/// Returns an iterator over mesh objects in a specific model.
///
/// Like [`get_mesh_objects()`] but queries only a single model instance.
///
/// # Arguments
///
/// * `model` - The model to query
///
/// # Returns
///
/// An iterator over [`MeshObjectRef`] for mesh objects in this model.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Compare mesh counts across models
/// let root_meshes = get_mesh_objects_from_model(&package.root).count();
/// println!("Root model: {} mesh objects", root_meshes);
///
/// for (path, model) in &package.sub_models {
///     let sub_meshes = get_mesh_objects_from_model(model).count();
///     println!("{}: {} mesh objects", path, sub_meshes);
/// }
/// ```
///
/// # See Also
///
/// * [`get_mesh_objects()`] - Query all mesh objects across all models
pub fn get_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MeshObjectRef<'a>> {
    get_mesh_objects_from_model_ref(ModelRef { model, path: None }).map(MeshObjectRef::new)
}

/// Returns an iterator over mesh objects in the model reference.
///
/// Internal helper that preserves model path information.
/// Most users should use [`get_mesh_objects()`] or [`get_mesh_objects_from_model()`].
pub fn get_mesh_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        //.filter(|o| o.mesh.is_some())
        .filter(|o| {
            if let Some(kind) = &o.kind
                && let ObjectKind::Mesh(_) = kind
            {
                true
            } else {
                false
            }
        })
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// A reference to a composed part object (assembly) with convenient access to components.
///
/// Composed parts are assemblies that reference other objects (which can be mesh objects
/// or other composed parts). Each component can have its own transform and can reference
/// objects in different model files (via the production extension).
///
/// # Accessing Data
///
/// * Call [`components()`](ComponentsObjectRef::components) to iterate components
/// * Access object metadata directly (id, name, uuid, etc.) via [`Deref`]
/// * Check `origin_model_path` to see which model the composed part came from
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for composed in get_components_objects(&package) {
///     // Access object metadata via Deref
///     println!("Assembly: {} (ID: {})", composed.name.as_deref().unwrap_or("unnamed"), composed.id);
///     
///     // Iterate components
///     let mut component_count = 0;
///     for component in composed.components() {
///         component_count += 1;
///         println!("  Component references object {}", component.objectid);
///         
///         if let Some(path) = &component.path_to_look_for {
///             println!("    Look in model: {}", path);
///         }
///         
///         if component.transform.is_some() {
///             println!("    Has transform");
///         }
///     }
///     println!("  Total components: {}", component_count);
/// }
/// ```
///
/// # See Also
///
/// * [`get_components_objects()`] - Get all composed parts from a package
/// * [`ComponentRef`] - References to individual components
/// * [`MeshObjectRef`] - References to mesh objects (not assemblies)
pub struct ComponentsObjectRef<'a>(GenericObjectRef<'a, Components>);

impl<'a> ComponentsObjectRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        ComponentsObjectRef(GenericObjectRef {
            entity: o.object.get_components_object().unwrap(),
            id: o.object.id,
            object_type: o.object.objecttype.unwrap_or(ObjectType::Model),
            thumbnail: o.object.thumbnail.clone(),
            part_number: o.object.partnumber.clone(),
            name: o.object.name.clone(),
            pid: o.object.pid,
            pindex: o.object.pindex,
            uuid: o.object.uuid.clone(),
            origin_model_path: o.path,
        })
    }

    /// Returns an iterator over the components within this composed part.
    ///
    /// Components reference other objects and can apply transforms. The `path_to_look_for`
    /// field indicates which model file contains the referenced object (production extension).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for composed in get_components_objects(&package) {
    ///     for component in composed.components() {
    ///         println!("Component references object {}", component.objectid);
    ///         
    ///         // Check for cross-model references
    ///         if let Some(path) = &component.path_to_look_for {
    ///             println!("  In model: {}", path);
    ///         }
    ///         
    ///         // Check for UUID (production extension)
    ///         if let Some(uuid) = &component.uuid {
    ///             println!("  UUID: {}", uuid);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn components(&self) -> impl Iterator<Item = ComponentRef> {
        self.entity.component.iter().map(|c| {
            let comp_path = match &c.path {
                Some(path) => Some(path.clone()),
                None => self
                    .origin_model_path
                    .map(|parent_path| parent_path.to_owned()),
            };

            ComponentRef {
                objectid: c.objectid,
                transform: c.transform.clone(),
                path_to_look_for: comp_path,
                uuid: c.uuid.clone(),
            }
        })
    }
}

impl<'a> Deref for ComponentsObjectRef<'a> {
    type Target = GenericObjectRef<'a, Components>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A reference to a boolean shape object with convenient access to boolean operations.
///
/// Boolean shapes define objects by applying boolean operations (union, difference, intersection)
/// between a base object and one or more operand objects.
///
/// # Accessing Data
///
/// * Call [`boolean_shape()`](BooleanShapeRef::boolean_shape) to get the underlying boolean shape data
/// * Use [`base_objectid()`](BooleanShapeRef::base_objectid) to get the base object reference
/// * Use [`operation()`](BooleanShapeRef::operation) to determine the operation type
/// * Call [`booleans()`](BooleanShapeRef::booleans) to iterate over operands
/// * Use convenience methods [`is_union()`](BooleanShapeRef::is_union), [`is_difference()`](BooleanShapeRef::is_difference),
///   [`is_intersection()`](BooleanShapeRef::is_intersection) for quick operation type checking
/// * Access object metadata directly (id, name, uuid, etc.) via [`Deref`]
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for boolean_ref in get_boolean_shape_objects(&package) {
///     // Access object metadata via Deref
///     println!("Boolean Shape: {} (ID: {})",
///         boolean_ref.name.as_deref().unwrap_or("unnamed"),
///         boolean_ref.id
///     );
///     
///     // Access boolean operation details
///     println!("  Base object: {}", boolean_ref.base_objectid());
///     println!("  Operation: {:?}", boolean_ref.operation());
///     
///     // Check operation type
///     if boolean_ref.is_difference() {
///         println!("  This is a subtraction operation");
///     }
///     
///     // List operands
///     for operand in boolean_ref.booleans() {
///         println!("  Operand: object {}", operand.objectid);
///         if let Some(transform) = &operand.transform {
///             println!("    Has transform");
///         }
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_boolean_shape_objects()`] - Get all boolean shapes from a package
/// * [`BooleanShape`] - The underlying boolean shape type
/// * [`BooleanOperation`] - The operation type enum
pub struct BooleanShapeRef<'a>(GenericObjectRef<'a, BooleanShape>);

impl<'a> BooleanShapeRef<'a> {
    fn new(o: ObjectRef<'a>) -> Self {
        BooleanShapeRef(GenericObjectRef {
            entity: o.object.get_boolean_shape_object().unwrap(),
            id: o.object.id,
            object_type: o.object.objecttype.unwrap_or(ObjectType::Model),
            thumbnail: o.object.thumbnail.clone(),
            part_number: o.object.partnumber.clone(),
            name: o.object.name.clone(),
            pid: o.object.pid,
            pindex: o.object.pindex,
            uuid: o.object.uuid.clone(),
            origin_model_path: o.path,
        })
    }

    /// Returns a reference to the boolean shape data.
    ///
    /// The boolean shape contains the base object ID, operation type, transform,
    /// and the sequence of boolean operands.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     let shape = boolean_ref.boolean_shape();
    ///     println!("Base object: {}", shape.objectid);
    ///     println!("Number of operands: {}", shape.booleans.len());
    ///     
    ///     if let Some(transform) = &shape.transform {
    ///         println!("Base has transform");
    ///     }
    /// }
    /// ```
    pub fn boolean_shape(&self) -> &'a BooleanShape {
        self.entity
    }

    /// Returns the ID of the base object.
    ///
    /// The base object is the primary shape to which boolean operations are applied.
    /// It can be a mesh object, another boolean shape, or other shape types.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     let base_id = boolean_ref.base_objectid();
    ///     println!("Boolean shape {} operates on object {}",
    ///         boolean_ref.id, base_id);
    /// }
    /// ```
    pub fn base_objectid(&self) -> u32 {
        self.entity.objectid
    }

    /// Returns the boolean operation type.
    ///
    /// The operation determines how operands are combined with the base object:
    /// - `Union`: Merges shapes together
    /// - `Difference`: Subtracts operands from the base
    /// - `Intersection`: Keeps only overlapping volume
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use threemf2::io::query::*;
    /// use threemf2::core::boolean::BooleanOperation;
    ///
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     match boolean_ref.operation() {
    ///         BooleanOperation::Union => println!("Union operation"),
    ///         BooleanOperation::Difference => println!("Difference operation"),
    ///         BooleanOperation::Intersection => println!("Intersection operation"),
    ///     }
    /// }
    /// ```
    pub fn operation(&self) -> BooleanOperation {
        self.entity.operation
    }

    /// Returns true if the operation is Union.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     if boolean_ref.is_union() {
    ///         println!("Merging shapes together");
    ///     }
    /// }
    /// ```
    pub fn is_union(&self) -> bool {
        matches!(self.entity.operation, BooleanOperation::Union)
    }

    /// Returns true if the operation is Difference.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     if boolean_ref.is_difference() {
    ///         println!("Subtracting shapes");
    ///     }
    /// }
    /// ```
    pub fn is_difference(&self) -> bool {
        matches!(self.entity.operation, BooleanOperation::Difference)
    }

    /// Returns true if the operation is Intersection.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     if boolean_ref.is_intersection() {
    ///         println!("Keeping only overlapping volume");
    ///     }
    /// }
    /// ```
    pub fn is_intersection(&self) -> bool {
        matches!(self.entity.operation, BooleanOperation::Intersection)
    }

    /// Returns an iterator over the boolean operands.
    ///
    /// Operands are the mesh objects that participate in the boolean operation
    /// with the base object. Each operand can have its own transform and path
    /// for cross-model references.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for boolean_ref in get_boolean_shape_objects(&package) {
    ///     println!("Boolean shape {} has {} operands:",
    ///         boolean_ref.id,
    ///         boolean_ref.booleans().count()
    ///     );
    ///     
    ///     for (i, operand) in boolean_ref.booleans().enumerate() {
    ///         println!("  Operand {}: object {}", i + 1, operand.objectid);
    ///         
    ///         if let Some(transform) = &operand.transform {
    ///             println!("    Has transform applied");
    ///         }
    ///         
    ///         if let Some(path) = &operand.path {
    ///             println!("    From model: {}", path);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn booleans(&self) -> impl Iterator<Item = BooleanRef> + '_ {
        self.entity.booleans.iter().map(|b| BooleanRef {
            objectid: b.objectid,
            transform: b.transform.clone(),
            path: b.path.clone(),
        })
    }
}

impl<'a> Deref for BooleanShapeRef<'a> {
    type Target = GenericObjectRef<'a, BooleanShape>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A reference to a component within a composed part (assembly).
///
/// Components are references to other objects with optional transforms.
/// They enable building assemblies and hierarchical structures.
///
/// # Fields
///
/// * `objectid` - The ID of the object this component references
/// * `path_to_look_for` - Model file path for cross-model references (production extension)
/// * `transform` - Optional transform applied to this component instance
/// * `uuid` - Optional UUID for tracking (production extension)
///
/// # Cross-Model References
///
/// The `path_to_look_for` field enables referencing objects in different model files:
/// * `None` - Object is in the same model as the composed part
/// * `Some(path)` - Object is in the specified sub-model file
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for composed in get_components_objects(&package) {
///     for component in composed.components() {
///         // Basic info
///         println!("Component -> Object {}", component.objectid);
///         
///         // Cross-model reference?
///         match &component.path_to_look_for {
///             Some(path) => println!("  References object in: {}", path),
///             None => println!("  References object in same model"),
///         }
///         
///         // Transform info
///         if let Some(transform) = &component.transform {
///             println!("  Transform: {:?}", &transform.0[..3]);
///         }
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`ComponentsObjectRef::components()`] - Get components from a composed part
/// * [`get_components_objects()`] - Find composed parts in a package
pub struct ComponentRef {
    /// ID of the referenced object.
    pub objectid: u32,
    /// Path to look for the object,
    /// if specified else it will be the parent Model where the object is originating from.
    pub path_to_look_for: Option<String>,
    /// Transform applied to the component.
    pub transform: Option<Transform>,
    /// UUID of the component.
    pub uuid: Option<String>,
}

/// A reference to a boolean operand within a boolean shape.
///
/// Boolean operands reference mesh objects that participate in boolean operations
/// (union, difference, intersection) with a base object.
///
/// # Fields
///
/// * `objectid` - The ID of the mesh object used as the boolean operand
/// * `transform` - Optional transform applied to the operand before the operation
/// * `path` - Optional path for cross-model references (production extension)
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for boolean_shape in get_boolean_shape_objects(&package) {
///     for operand in boolean_shape.booleans() {
///         println!("Operand references object {}", operand.objectid);
///         
///         if let Some(transform) = &operand.transform {
///             println!("  Has transform applied");
///         }
///         
///         if let Some(path) = &operand.path {
///             println!("  From model: {}", path);
///         }
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`BooleanShapeRef::booleans()`] - Get operands from a boolean shape
/// * [`BooleanShapeRef`] - The parent boolean shape reference
pub struct BooleanRef {
    /// ID of the referenced mesh object.
    pub objectid: u32,
    /// Transform applied to the operand.
    pub transform: Option<Transform>,
    /// Path to the operand model file (production extension).
    pub path: Option<String>,
}
/// A reference to a build item with convenient accessor methods.
///
/// Build items specify which objects should be manufactured and optionally
/// apply transforms. Items are part of the `Build` section in a 3MF model.
///
/// # Fields
///
/// * `item` - Reference to the underlying [`Item`] data
/// * `origin_model_path` - Path to the model containing this item (`None` for root model)
///
/// # Accessor Methods
///
/// This type provides convenient accessor methods for common item properties:
/// * [`objectid()`](ItemRef::objectid) - Get the referenced object ID
/// * [`transform()`](ItemRef::transform) - Get optional transform matrix
/// * [`partnumber()`](ItemRef::partnumber) - Get optional part number
/// * [`uuid()`](ItemRef::uuid) - Get UUID (production extension)
/// * [`path()`](ItemRef::path) - Get path for cross-model references (production extension)
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for item in get_items(&package) {
///     println!("Item references object {}", item.objectid());
///     
///     if let Some(partnumber) = item.partnumber() {
///         println!("  Part number: {}", partnumber);
///     }
///     
///     if let Some(transform) = item.transform() {
///         println!("  Has transform");
///     }
///     
///     if item.origin_model_path.is_none() {
///         println!("  From root model");
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_items()`] - Get all items from a package
/// * [`get_items_by_objectid()`] - Find items referencing a specific object
/// * [`Item`] - The underlying 3MF item type
pub struct ItemRef<'a> {
    /// The item itself.
    pub item: &'a Item,
    /// The path to the model containing this item, if None then it is the root model.
    pub origin_model_path: Option<&'a str>,
}

impl<'a> ItemRef<'a> {
    /// Returns the ID of the object this item references.
    ///
    /// Build items reference objects from the resources section that should be manufactured.
    /// Multiple items can reference the same object with different transforms or part numbers.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for item in get_items(&package) {
    ///     println!("Item references object {}", item.objectid());
    /// }
    /// ```
    pub fn objectid(&self) -> u32 {
        self.item.objectid
    }

    /// Returns the transform applied to this item, if any.
    ///
    /// The transform is a 4x3 affine transformation matrix (stored as 12 floats)
    /// that positions and orients the object on the build plate.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for item in get_items(&package) {
    ///     if let Some(transform) = item.transform() {
    ///         println!("Item has transform: {:?}", &transform.0[..3]);
    ///     }
    /// }
    /// ```
    pub fn transform(&self) -> Option<&Transform> {
        self.item.transform.as_ref()
    }

    /// Returns the part number of this item.
    ///
    /// Part numbers are used for manufacturing tracking and can be set independently
    /// from the referenced object's name or properties.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for item in get_items(&package) {
    ///     if let Some(partnumber) = item.partnumber() {
    ///         println!("Part number: {}", partnumber);
    ///     }
    /// }
    /// ```
    pub fn partnumber(&self) -> Option<&str> {
        self.item.partnumber.as_deref()
    }

    /// Returns the path attribute for cross-model object references (production extension).
    ///
    /// When set, this indicates the item references an object in a different model file
    /// within the package. This is part of the 3MF production extension.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for item in get_items(&package) {
    ///     if let Some(path) = item.path() {
    ///         println!("References object in model: {}", path);
    ///     }
    /// }
    /// ```
    pub fn path(&self) -> Option<&str> {
        self.item.path.as_deref()
    }

    /// Returns the UUID of this item (production extension).
    ///
    /// UUIDs provide unique identification for tracking items through manufacturing
    /// workflows. This is part of the 3MF production extension.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// for item in get_items(&package) {
    ///     if let Some(uuid) = item.uuid() {
    ///         println!("Item UUID: {}", uuid);
    ///     }
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// * [`get_item_by_uuid()`] - Find an item by its UUID
    pub fn uuid(&self) -> Option<&str> {
        self.item.uuid.as_deref()
    }
}

/// Returns an iterator over composed part objects (assemblies) in the package.
///
/// Filters out mesh objects and returns only objects that are assemblies of components.
/// Composed parts enable building hierarchical structures where objects reference other objects.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
///
/// # Returns
///
/// An iterator over [`ComponentsObjectRef`] for all composed parts.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Find and analyze assemblies
/// for composed in get_components_objects(&package) {
///     println!("Assembly: {} (ID: {})",
///         composed.name.as_deref().unwrap_or("unnamed"),
///         composed.id
///     );
///     
///     // Count components
///     let component_count = composed.components().count();
///     println!("  Contains {} components", component_count);
///     
///     // List referenced objects
///     for component in composed.components() {
///         print!("  -> Object {}", component.objectid);
///         if let Some(path) = &component.path_to_look_for {
///             print!(" in {}", path);
///         }
///         println!();
///     }
/// }
///
/// // Find assemblies referencing a specific object
/// let target_id = 42;
/// for composed in get_components_objects(&package) {
///     let references_target = composed.components()
///         .any(|c| c.objectid == target_id);
///     
///     if references_target {
///         println!("Assembly {} references object {}", composed.id, target_id);
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_mesh_objects()`] - Get mesh objects (not assemblies)
/// * [`get_objects()`] - Get all objects (meshes and composed parts)
/// * [`ComponentsObjectRef`] - The reference type returned
pub fn get_components_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = ComponentsObjectRef<'a>> {
    iter_objects_from(package, get_components_objects_from_model_ref).map(ComponentsObjectRef::new)
}

/// Returns an iterator over composed part objects in a specific model.
///
/// Like [`get_components_objects()`] but queries only a single model instance.
///
/// # Arguments
///
/// * `model` - The model to query
///
/// # Returns
///
/// An iterator over [`ComponentsObjectRef`] for composed parts in this model.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Count assemblies per model
/// let root_assemblies = get_composedpart_objects_from_model(&package.root).count();
/// println!("Root model: {} assemblies", root_assemblies);
///
/// for (path, model) in &package.sub_models {
///     let count = get_composedpart_objects_from_model(model).count();
///     if count > 0 {
///         println!("{}: {} assemblies", path, count);
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_components_objects()`] - Query across all models
pub fn get_components_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ComponentsObjectRef<'a>> {
    get_components_objects_from_model_ref(ModelRef { model, path: None })
        .map(ComponentsObjectRef::new)
}

/// Returns an iterator over composed part objects in the model reference.
///
/// Internal helper that preserves model path information.
/// Most users should use [`get_components_objects()`] or [`get_components_objects_from_model()`].
pub fn get_components_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        //.filter(|o| o.components.is_some())
        .filter(|o| {
            if let Some(kind) = &o.kind
                && let ObjectKind::Components(_) = kind
            {
                true
            } else {
                false
            }
        })
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// Returns an iterator over boolean shape objects in the package.
///
/// Boolean shapes define objects by applying boolean operations between a base object
/// and one or more operands. This function returns only objects of type BooleanShape,
/// filtering out mesh objects and composed parts.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
///
/// # Returns
///
/// An iterator over [`BooleanShapeRef`] for all boolean shape objects.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Analyze boolean operations
/// for boolean_ref in get_boolean_shape_objects(&package) {
///     println!("Boolean shape: {} (ID: {})",
///         boolean_ref.name.as_deref().unwrap_or("unnamed"),
///         boolean_ref.id
///     );
///     
///     println!("  Base object: {}", boolean_ref.base_objectid());
///     println!("  Operation: {:?}", boolean_ref.operation());
///     
///     // Count operands
///     let operand_count = boolean_ref.booleans().count();
///     println!("  Operands: {}", operand_count);
///     
///     // Check operation type
///     if boolean_ref.is_difference() {
///         println!("  This subtracts volumes");
///     }
/// }
///
/// // Find all difference operations
/// let difference_count = get_boolean_shape_objects(&package)
///     .filter(|b| b.is_difference())
///     .count();
/// println!("Total difference operations: {}", difference_count);
/// ```
///
/// # See Also
///
/// * [`get_mesh_objects()`] - Get mesh objects (not boolean shapes)
/// * [`get_components_objects()`] - Get composed parts
/// * [`get_objects()`] - Get all objects (meshes, components, and boolean shapes)
/// * [`BooleanShapeRef`] - The reference type returned
pub fn get_boolean_shape_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = BooleanShapeRef<'a>> {
    iter_objects_from(package, get_boolean_shape_objects_from_model_ref).map(BooleanShapeRef::new)
}

/// Returns an iterator over boolean shape objects in a specific model.
///
/// Like [`get_boolean_shape_objects()`] but queries only a single model instance.
///
/// # Arguments
///
/// * `model` - The model to query
///
/// # Returns
///
/// An iterator over [`BooleanShapeRef`] for boolean shape objects in this model.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Count boolean shapes per model
/// let root_booleans = get_boolean_shape_objects_from_model(&package.root).count();
/// println!("Root model: {} boolean shapes", root_booleans);
///
/// for (path, model) in &package.sub_models {
///     let sub_booleans = get_boolean_shape_objects_from_model(model).count();
///     if sub_booleans > 0 {
///         println!("{}: {} boolean shapes", path, sub_booleans);
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_boolean_shape_objects()`] - Query across all models
pub fn get_boolean_shape_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = BooleanShapeRef<'a>> {
    get_boolean_shape_objects_from_model_ref(ModelRef { model, path: None })
        .map(BooleanShapeRef::new)
}

/// Returns an iterator over boolean shape objects in the model reference.
///
/// Internal helper that preserves model path information.
/// Most users should use [`get_boolean_shape_objects()`] or [`get_boolean_shape_objects_from_model()`].
pub fn get_boolean_shape_objects_from_model_ref<'a>(
    model_ref: ModelRef<'a>,
) -> impl Iterator<Item = ObjectRef<'a>> {
    model_ref
        .model
        .resources
        .object
        .iter()
        .filter(|o| {
            if let Some(kind) = &o.kind
                && let ObjectKind::BooleanShape(_) = kind
            {
                true
            } else {
                false
            }
        })
        .map(move |o| ObjectRef {
            object: o,
            path: model_ref.path,
        })
}

/// Returns an iterator over all build items in the package, including sub-models.
///
/// Build items specify which objects should be manufactured. This function
/// traverses the root model and all sub-models, returning items with their
/// origin model path tracked via [`ItemRef::origin_model_path`].
///
/// Use this when you need to:
/// - List all objects scheduled for manufacturing
/// - Find which items reference specific objects
/// - Inspect transforms applied to build items
/// - Access production extension attributes (UUIDs, part numbers)
///
/// # Arguments
///
/// * `package` - The 3MF package to query
///
/// # Returns
///
/// An iterator over [`ItemRef`] containing all build items across all models.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Print all build items
/// for item in get_items(&package) {
///     println!("Item references object {}", item.objectid());
///     if let Some(path) = item.origin_model_path {
///         println!("  From model: {}", path);
///     }
/// }
///
/// // Count items per model
/// let root_items = get_items(&package)
///     .filter(|i| i.origin_model_path.is_none())
///     .count();
/// println!("Root model has {} items", root_items);
/// ```
///
/// # See Also
///
/// * [`get_items_from_model()`] - Query items from a specific model
/// * [`get_items_by_objectid()`] - Find items referencing a specific object
/// * [`get_item_by_uuid()`] - Find item by UUID (production extension)
/// * [`ItemRef`] - The reference type returned by this function
pub fn get_items<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ItemRef<'a>> {
    iter_models(package).flat_map(get_items_from_model_ref)
}

/// Returns an iterator over all build items in a specific model.
///
/// Unlike [`get_items()`], this function only queries a single model instance,
/// not the entire package. The returned items will have `origin_model_path` set to `None`
/// since we don't track path information for single-model queries.
///
/// Use this when:
/// - Working with a specific model instance
/// - You already know which model contains the items you need
/// - Building custom traversal logic
///
/// # Arguments
///
/// * `model` - The model to query
///
/// # Returns
///
/// An iterator over [`ItemRef`] containing build items from this model only.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Query items only from the root model
/// for item in get_items_from_model(&package.root) {
///     println!("Root item references object {}", item.objectid());
/// }
///
/// // Query items from a specific sub-model
/// if let Some(model) = package.sub_models.get("/3D/model.model") {
///     for item in get_items_from_model(model) {
///         println!("Sub-model item: {}", item.objectid());
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_items()`] - Query items across all models in a package
/// * [`get_items_from_model_ref()`] - Internal function that preserves model path
pub fn get_items_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ItemRef<'a>> {
    get_items_from_model_ref(ModelRef { model, path: None })
}

/// Returns an iterator over all build items in the model reference.
///
/// This is an internal helper function used by [`get_items()`] and [`get_items_from_model()`].
/// It preserves the model path information when iterating across multiple models.
///
/// Most users should use [`get_items()`] or [`get_items_from_model()`] instead.
pub fn get_items_from_model_ref<'a>(model_ref: ModelRef<'a>) -> impl Iterator<Item = ItemRef<'a>> {
    model_ref.model.build.item.iter().map(move |item| ItemRef {
        item,
        origin_model_path: model_ref.path,
    })
}

/// Returns an iterator over build items that reference a specific object ID.
///
/// In 3MF, multiple build items can reference the same object with different
/// transforms, part numbers, or other properties. This function finds all such items.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
/// * `objectid` - The object ID to search for
///
/// # Returns
///
/// An iterator over [`ItemRef`] for items referencing the specified object.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Find all items that reference object 42
/// for item in get_items_by_objectid(&package, 42) {
///     println!("Found item with part number: {:?}", item.partnumber());
///     
///     if let Some(transform) = item.transform() {
///         println!("  Position on build plate: {:?}", &transform.0[9..12]);
///     }
/// }
///
/// // Count how many times each object appears in the build
/// for obj_ref in get_objects(&package) {
///     let count = get_items_by_objectid(&package, obj_ref.object.id).count();
///     if count > 0 {
///         println!("Object {} appears {} times", obj_ref.object.id, count);
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`get_items()`] - Get all items in a package
/// * [`get_objects()`] - Get all objects to find IDs to query
pub fn get_items_by_objectid<'a>(
    package: &'a ThreemfPackage,
    objectid: u32,
) -> impl Iterator<Item = ItemRef<'a>> {
    get_items(package).filter(move |item_ref| item_ref.item.objectid == objectid)
}

/// Finds a build item by its UUID (production extension).
///
/// UUIDs provide unique identification for items in manufacturing workflows.
/// This is part of the 3MF production extension. UUIDs should be unique across
/// the entire package, so this function returns at most one item.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
/// * `uuid` - The UUID string to search for
///
/// # Returns
///
/// `Some(ItemRef)` if an item with the UUID exists, `None` otherwise.
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Find item by UUID
/// let uuid = "550e8400-e29b-41d4-a716-446655440000";
/// if let Some(item) = get_item_by_uuid(&package, uuid) {
///     println!("Found item referencing object {}", item.objectid());
///     println!("  Part number: {:?}", item.partnumber());
/// } else {
///     println!("No item found with UUID: {}", uuid);
/// }
///
/// // Collect all item UUIDs
/// for item in get_items(&package) {
///     if let Some(item_uuid) = item.uuid() {
///         println!("Item UUID: {}", item_uuid);
///         
///         // Verify we can find it again
///         assert!(get_item_by_uuid(&package, item_uuid).is_some());
///     }
/// }
/// ```
///
/// # See Also
///
/// * [`ItemRef::uuid()`] - Get UUID from an item reference
/// * [`get_items()`] - Get all items (to find items with UUIDs)
pub fn get_item_by_uuid<'a>(package: &'a ThreemfPackage, uuid: &str) -> Option<ItemRef<'a>> {
    get_items(package).find(|item_ref| {
        if let Some(item_uuid) = &item_ref.item.uuid {
            item_uuid == uuid
        } else {
            false
        }
    })
}

/// A reference to a model within a package, with path information for sub-models.
///
/// 3MF packages can contain multiple model files: one root model and zero or more sub-models.
/// This type wraps a model reference with its path for tracking purposes.
///
/// # Fields
///
/// * `model` - Reference to the [`Model`] data
/// * `path` - Path to the model file (`None` for root model, `Some(path)` for sub-models)
///
/// # Root vs Sub-Models
///
/// - **Root model** (`path = None`): The main model file, always has a `Build` section
/// - **Sub-model** (`path = Some(...)`): Additional model files referenced by other models
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// for model_ref in iter_models(&package) {
///     match model_ref.path {
///         None => {
///             println!("Root model:");
///             println!("  Build items: {}", model_ref.model.build.item.len());
///         }
///         Some(path) => {
///             println!("Sub-model: {}", path);
///         }
///     }
///     
///     println!("  Objects: {}", model_ref.model.resources.object.len());
/// }
/// ```
///
/// # See Also
///
/// * [`iter_models()`] - Get all models from a package
/// * [`Model`] - The underlying model type
pub struct ModelRef<'a> {
    /// The model itself.
    pub model: &'a Model,
    /// The path to the model, if it's a sub-model.
    pub path: Option<&'a str>,
}

/// Returns an iterator over all models in the package, including the root and sub-models.
///
/// 3MF packages consist of a root model (which must have a Build section) and optional
/// sub-models that contain additional resources. This function provides access to all of them.
///
/// The root model is always returned first with `path = None`, followed by sub-models
/// with their file paths.
///
/// # Arguments
///
/// * `package` - The 3MF package to query
///
/// # Returns
///
/// An iterator over [`ModelRef`] for all models (root first, then sub-models).
///
/// # Examples
///
/// ```rust,ignore
/// use threemf2::io::{ThreemfPackage, query::*};
///
/// let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true)?;
///
/// // Count models
/// let total_models = iter_models(&package).count();
/// println!("Package contains {} model(s)", total_models);
///
/// // Analyze each model
/// for model_ref in iter_models(&package) {
///     let location = model_ref.path.unwrap_or("root");
///     let objects = model_ref.model.resources.object.len();
///     
///     println!("{}: {} objects", location, objects);
///     
///     // Root model specific
///     if model_ref.path.is_none() {
///         let items = model_ref.model.build.item.len();
///         println!("  {} build items", items);
///     }
/// }
///
/// // Get specific model statistics
/// let root_objects = iter_models(&package)
///     .find(|m| m.path.is_none())
///     .map(|m| m.model.resources.object.len())
///     .unwrap_or(0);
/// println!("Root model has {} objects", root_objects);
/// ```
///
/// # See Also
///
/// * [`ModelRef`] - The reference type returned
/// * [`get_objects()`] - Query objects across all models
/// * [`get_items()`] - Query items across all models
pub fn iter_models<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ModelRef<'a>> {
    std::iter::once(ModelRef {
        model: &package.root,
        path: None,
    })
    .chain(package.sub_models.iter().map(|(path, model)| ModelRef {
        model,
        path: Some(path),
    }))
}

fn iter_objects_from<'a, I, F>(
    package: &'a ThreemfPackage,
    f: F,
) -> impl Iterator<Item = ObjectRef<'a>>
where
    F: Fn(ModelRef<'a>) -> I + Copy,
    I: IntoIterator<Item = ObjectRef<'a>>,
{
    iter_models(package).flat_map(f)
}

#[cfg(feature = "io-memory-optimized-read")]
#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    #[test]
    fn test_get_object_ref_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let object_ref = get_objects(&package)
            .filter(|r| matches!(r.path, Some("/3D/Objects/Object.model")))
            .find(|r| r.object.id == 1);

        match object_ref {
            Some(obj_ref) => {
                assert!(obj_ref.object.get_mesh().is_some());
                assert_eq!(obj_ref.object.id, 1);
            }
            None => panic!("Object ref not found"),
        }
    }

    #[test]
    fn test_get_objects_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 6);
    }

    #[test]
    fn test_get_mesh_objects_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_mesh_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 5);
    }

    #[test]
    fn test_get_composedpart_objects_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_components_objects(&package).collect::<Vec<_>>();
        assert_eq!(objects.len(), 1);
    }

    #[test]
    fn test_get_beamlattice_objects_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let objects = get_mesh_objects(&package)
            .filter(|mesh_ref| mesh_ref.mesh().beamlattice.is_some())
            .count();
        assert_eq!(objects, 2);
    }

    #[test]
    fn test_iter_models_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let models = iter_models(&package).collect::<Vec<_>>();
        assert_eq!(models.len(), 5);
        assert!(models[0].path.is_none());
        for model_ref in &models[1..] {
            assert!(model_ref.path.is_some());
        }
    }

    #[test]
    fn test_integration_component_resolution() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let composed_objects = get_components_objects(&package).collect::<Vec<_>>();
        assert_eq!(composed_objects.len(), 1);
        let components = composed_objects[0].components().collect::<Vec<_>>();
        assert!(!components.is_empty());
        // Check that components have valid objectids
        for comp in components {
            assert!(comp.objectid > 0);
            // ToDo: Optionally check if path is set for sub-model references
        }
    }

    #[test]
    fn test_get_items_from_package() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let items = get_items(&package).collect::<Vec<_>>();
        assert!(!items.is_empty());
    }

    #[test]
    fn test_get_items_by_objectid() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        // Get the first item's objectid
        let items = get_items(&package).collect::<Vec<_>>();
        assert!(!items.is_empty());
        let first_objectid = items[0].objectid();

        // Search for items with that objectid
        let items_with_id = get_items_by_objectid(&package, first_objectid).collect::<Vec<_>>();
        assert!(!items_with_id.is_empty());
        for item in items_with_id {
            assert_eq!(item.objectid(), first_objectid);
        }
    }

    #[test]
    fn test_item_ref_origin_model_path() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let items = get_items(&package).collect::<Vec<_>>();
        // At least one item should have origin_model_path = None (from root)
        let root_items = items
            .iter()
            .filter(|i| i.origin_model_path.is_none())
            .count();
        assert!(root_items > 0);
    }

    #[test]
    fn test_boolean_shape_query_api_exists() {
        // This test verifies that all the boolean shape query types and functions exist
        // and can be called. It doesn't require a boolean shape test file.

        // Verify BooleanRef struct exists
        let _boolean_ref = BooleanRef {
            objectid: 1,
            transform: None,
            path: None,
        };

        // Test that BooleanRef fields are accessible
        assert_eq!(_boolean_ref.objectid, 1);
        assert!(_boolean_ref.transform.is_none());
        assert!(_boolean_ref.path.is_none());
    }

    #[test]
    fn test_get_boolean_shape_objects_from_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let boolean_shapes = get_boolean_shape_objects(&package).collect::<Vec<_>>();

        // Verify we find exactly 2 boolean shapes
        assert_eq!(boolean_shapes.len(), 2, "Expected 2 boolean shapes");

        // Verify we can access all boolean shapes
        for boolean_ref in &boolean_shapes {
            assert!(!boolean_ref.boolean_shape().booleans.is_empty());
        }
    }

    #[test]
    fn test_boolean_shape_operations() {
        use crate::core::boolean::BooleanOperation;

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let boolean_shapes: std::collections::HashMap<u32, _> = get_boolean_shape_objects(&package)
            .map(|b| (b.id, b))
            .collect();

        // Verify Object 6 has Intersection operation
        let intersected = boolean_shapes
            .get(&6)
            .expect("Object 6 (Intersected) not found");
        assert_eq!(intersected.operation(), BooleanOperation::Intersection);
        assert!(intersected.is_intersection());
        assert!(!intersected.is_difference());
        assert!(!intersected.is_union());

        // Verify Object 8 has Difference operation
        let full_part = boolean_shapes
            .get(&8)
            .expect("Object 8 (Full part) not found");
        assert_eq!(full_part.operation(), BooleanOperation::Difference);
        assert!(full_part.is_difference());
        assert!(!full_part.is_intersection());
        assert!(!full_part.is_union());
    }

    #[test]
    fn test_boolean_shape_base_objects() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let boolean_shapes: std::collections::HashMap<u32, _> = get_boolean_shape_objects(&package)
            .map(|b| (b.id, b))
            .collect();

        // Verify Object 6 (Intersected) base is Object 4 (Cube)
        let intersected = boolean_shapes.get(&6).expect("Object 6 not found");
        assert_eq!(
            intersected.base_objectid(),
            4,
            "Object 6 base should be Object 4 (Cube)"
        );

        // Verify Object 8 (Full part) base is Object 6 (Intersected - nested boolean!)
        let full_part = boolean_shapes.get(&8).expect("Object 8 not found");
        assert_eq!(
            full_part.base_objectid(),
            6,
            "Object 8 base should be Object 6 (nested boolean)"
        );
    }

    #[test]
    fn test_boolean_operands_count() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let boolean_shapes: std::collections::HashMap<u32, _> = get_boolean_shape_objects(&package)
            .map(|b| (b.id, b))
            .collect();

        // Verify "Intersected" (Object 6) has 1 operand (Sphere)
        let intersected = boolean_shapes.get(&6).expect("Object 6 not found");
        let intersected_operands: Vec<_> = intersected.booleans().collect();
        assert_eq!(
            intersected_operands.len(),
            1,
            "Object 6 should have 1 operand (Sphere)"
        );

        // Verify "Full part" (Object 8) has 3 operands (same Cylinder with 3 different transforms)
        let full_part = boolean_shapes.get(&8).expect("Object 8 not found");
        let full_part_operands: Vec<_> = full_part.booleans().collect();
        assert_eq!(
            full_part_operands.len(),
            3,
            "Object 8 should have 3 operands (Cylinder with different transforms)"
        );
    }

    #[test]
    fn test_boolean_operands_properties() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let boolean_shapes: std::collections::HashMap<u32, _> = get_boolean_shape_objects(&package)
            .map(|b| (b.id, b))
            .collect();

        // Test Object 6 (Intersected) operand
        let intersected = boolean_shapes.get(&6).expect("Object 6 not found");
        let intersected_operands: Vec<_> = intersected.booleans().collect();
        assert_eq!(
            intersected_operands[0].objectid, 5,
            "Object 6 operand should be Object 5 (Sphere)"
        );
        assert!(
            intersected_operands[0].transform.is_some(),
            "Sphere should have a transform"
        );
        assert!(
            intersected_operands[0].path.is_none(),
            "Sphere should not have a path (same model)"
        );

        // Test Object 8 (Full part) operands - all should be Object 3 (Cylinder)
        let full_part = boolean_shapes.get(&8).expect("Object 8 not found");
        let full_part_operands: Vec<_> = full_part.booleans().collect();

        for (i, operand) in full_part_operands.iter().enumerate() {
            assert_eq!(
                operand.objectid, 3,
                "Operand {} should reference Object 3 (Cylinder)",
                i
            );
            assert!(
                operand.transform.is_some(),
                "Operand {} should have a transform",
                i
            );
        }
    }

    #[test]
    fn test_boolean_base_transform() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/mesh-booleans-operations-material.3mf");
        let file = std::fs::File::open(path).unwrap();
        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();

        let boolean_shapes: std::collections::HashMap<u32, _> = get_boolean_shape_objects(&package)
            .map(|b| (b.id, b))
            .collect();

        // Test Object 6 (Intersected) - base transform should exist
        let intersected = boolean_shapes.get(&6).expect("Object 6 not found");
        assert!(
            intersected.boolean_shape().transform.is_some(),
            "Object 6 base should have a transform"
        );
    }
}

//! Query API for inspecting model-level data with stable views.

#![allow(clippy::needless_lifetimes)]

use crate::model::domain::{
    beamlattice::{self},
    boolean::{self},
    build::Item,
    component::Components,
    displacement::{self},
    material::{self},
    mesh::Mesh,
    metadata::Metadata,
    model::{Model, Unit},
    object::{self, Object},
    resources::BaseMaterials,
    slice::{self},
    transform::Transform,
    triangle_set::TriangleSet,
};
use crate::model::{Color, OptionalResourceId, OptionalResourceIndex, UuidResource};
use crate::threemf_namespaces::ThreemfNamespace;

use std::{borrow::Cow, num::NonZeroU32};

/// Stable classification for object kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKindView {
    /// Object contains a triangle mesh.
    Mesh,
    /// Object is an assembly of components.
    Components,
    /// Object defines a boolean shape.
    BooleanShape,
    /// Object contains a displacement mesh.
    DisplacementMesh,
}

/// Stable view over a model.
pub struct ModelView<'a> {
    model: &'a Model,
}

impl<'a> ModelView<'a> {
    pub(crate) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Returns the unit of measurement for the model.
    pub fn unit(&self) -> Unit {
        self.model.unit.clone().unwrap_or(Unit::Millimeter)
    }

    /// Returns the number of objects in the model.
    pub fn object_count(&self) -> usize {
        self.model.resources.object.len()
    }

    /// Returns the number of items in the build.
    pub fn build_item_count(&self) -> usize {
        self.model.build.item.len()
    }

    /// Returns the UUID of the build, if any.
    pub fn build_uuid(&self) -> Option<Cow<'a, str>> {
        self.model.build.uuid.as_ref().map(|uuid| match uuid {
            #[cfg(feature = "uuid")]
            UuidResource::NotUuid(res) => Cow::Borrowed(res.as_ref()),
            #[cfg(feature = "uuid")]
            UuidResource::Uuid(uuid) => Cow::Owned(uuid.to_string()),
            #[cfg(not(feature = "uuid"))]
            UuidResource::MaybeUuid(res) => Cow::Borrowed(res.as_ref()),
        })
    }

    /// Returns the number of slice stacks in the model.
    pub fn slice_stack_count(&self) -> usize {
        self.model.resources.slicestack.len()
    }

    /// Returns the number of color groups in the model.
    pub fn color_group_count(&self) -> usize {
        self.model.resources.colorgroup.len()
    }

    /// Returns the number of texture 2D groups in the model.
    pub fn texture2d_group_count(&self) -> usize {
        self.model.resources.texture2dgroup.len()
    }

    /// Returns the number of texture 2D resources in the model.
    pub fn texture2d_count(&self) -> usize {
        self.model.resources.texture2d.len()
    }

    /// Returns the number of composite materials in the model.
    pub fn composite_materials_count(&self) -> usize {
        self.model.resources.compositematerials.len()
    }

    /// Returns the number of multi-properties in the model.
    pub fn multi_properties_count(&self) -> usize {
        self.model.resources.multiproperties.len()
    }

    /// Returns the number of base materials in the model.
    pub fn base_materials_count(&self) -> usize {
        self.model.resources.basematerials.len()
    }

    /// Returns the number of displacement 2D resources in the model.
    pub fn displacement2d_count(&self) -> usize {
        self.model.resources.displacement2d.len()
    }

    /// Returns the number of normal vector groups in the model.
    pub fn normvectorgroup_count(&self) -> usize {
        self.model.resources.normvectorgroup.len()
    }

    /// Returns the number of displacement 2D groups in the model.
    pub fn disp2dgroup_count(&self) -> usize {
        self.model.resources.disp2dgroup.len()
    }

    /// Returns the number of metadata entries in the model.
    pub fn metadata_count(&self) -> usize {
        self.model.metadata.len()
    }

    /// Returns an iterator over the metadata entries in the model.
    pub fn metadata_iter(&self) -> impl Iterator<Item = MetadataView<'a>> + '_ {
        self.model.metadata.iter().map(MetadataView::new)
    }

    /// Returns the required extensions for the model, if any.
    pub fn required_extensions(&self) -> Option<&[ThreemfNamespace]> {
        let extensions = self.model.requiredextensions.get();
        if extensions.is_empty() {
            None
        } else {
            Some(extensions)
        }
    }

    /// Returns the recommended extensions for the model, if any.
    pub fn recommended_extensions(&self) -> Option<&[ThreemfNamespace]> {
        let extensions = self.model.recommendedextensions.get();
        if extensions.is_empty() {
            None
        } else {
            Some(extensions)
        }
    }

    /// Returns an iterator over the namespaces used by the model.
    pub fn used_namespaces(&self) -> impl Iterator<Item = ThreemfNamespace> {
        self.model.used_namespaces().into_iter()
    }
}

/// Stable view over a metadata entry.
pub struct MetadataView<'a> {
    metadata: &'a Metadata,
}

impl<'a> MetadataView<'a> {
    pub(crate) fn new(metadata: &'a Metadata) -> Self {
        Self { metadata }
    }

    /// Returns the name of the metadata entry.
    pub fn name(&self) -> &str {
        self.metadata.name.as_ref()
    }

    /// Returns the preserve flag of the metadata entry, if any.
    pub fn preserve(&self) -> Option<bool> {
        self.metadata.preserve.as_ref().map(|p| p.0)
    }

    /// Returns the value of the metadata entry, if any.
    pub fn value(&self) -> Option<&str> {
        self.metadata.value.as_ref().map(|v| v.as_ref())
    }
}

/// Stable view over a model object.
pub struct ObjectView<'a> {
    object: &'a Object,
}

impl<'a> ObjectView<'a> {
    pub(crate) fn new(object: &'a Object) -> Self {
        Self { object }
    }

    /// Returns the identifier of the object.
    pub fn id(&self) -> u32 {
        self.object.id
    }

    /// Returns the name of the object, if any.
    pub fn name(&self) -> Option<&'a str> {
        self.object.name.as_deref()
    }

    /// Returns the part number of the object, if any.
    pub fn part_number(&self) -> Option<&'a str> {
        self.object.partnumber.as_deref()
    }

    /// Returns the UUID of the object, if any.
    pub fn uuid(&self) -> Option<Cow<'a, str>> {
        self.object.uuid.as_ref().map(|uuid| match uuid {
            #[cfg(feature = "uuid")]
            super::UuidResource::NotUuid(res) => Cow::Borrowed(res.as_ref()),
            #[cfg(feature = "uuid")]
            super::UuidResource::Uuid(uuid) => Cow::Owned(uuid.to_string()),
            #[cfg(not(feature = "uuid"))]
            super::UuidResource::MaybeUuid(res) => Cow::Borrowed(res.as_ref()),
        })
    }

    /// Returns the type of the object.
    pub fn object_type(&self) -> object::ObjectType {
        self.object.objecttype.unwrap_or(object::ObjectType::Model)
    }

    /// Returns the kind of the object.
    pub fn kind(&self) -> ObjectKindView {
        match &self.object.kind {
            Some(object::ObjectKind::Mesh(_)) => ObjectKindView::Mesh,
            Some(object::ObjectKind::Components(_)) => ObjectKindView::Components,
            Some(object::ObjectKind::BooleanShape(_)) => ObjectKindView::BooleanShape,
            Some(object::ObjectKind::DisplacementMesh(_)) => ObjectKindView::DisplacementMesh,
            None => panic!("Invalid object found"),
        }
    }

    /// Returns true if the object is a mesh.
    pub fn is_mesh(&self) -> bool {
        matches!(self.object.kind, Some(object::ObjectKind::Mesh(_)))
    }

    /// Returns true if the object is a components assembly.
    pub fn is_components(&self) -> bool {
        matches!(self.object.kind, Some(object::ObjectKind::Components(_)))
    }

    /// Returns true if the object is a boolean shape.
    pub fn is_boolean_shape(&self) -> bool {
        matches!(self.object.kind, Some(object::ObjectKind::BooleanShape(_)))
    }

    /// Returns true if the object is a displacement mesh.
    /// Returns true if the object is a displacement mesh.
    pub fn is_displacement_mesh(&self) -> bool {
        matches!(
            self.object.kind,
            Some(object::ObjectKind::DisplacementMesh(_))
        )
    }

    /// Returns the property group id of the object.
    pub fn pid(&self) -> OptionalResourceId {
        self.object.pid
    }

    /// Returns the property index of the object.
    pub fn pindex(&self) -> OptionalResourceIndex {
        self.object.pindex
    }

    /// Returns the slice path of the object, if any.
    pub fn slicepath(&self) -> Option<&str> {
        self.object.slicepath.as_ref().map(|p| p.as_str())
    }

    /// Returns the slice stack id of the object, if any.
    pub fn slicestack_id(&self) -> Option<u32> {
        self.object.slicestackid.get()
    }
}

/// Stable view over a mesh object.
pub struct MeshObjectView<'a> {
    object: &'a Object,
    mesh: &'a Mesh,
}

/// Stable view over a beam lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct LatticeView<'a> {
    lattice: &'a beamlattice::BeamLattice,
}

/// Summary data for a beam lattice.
pub struct LatticeData {
    /// Number of beams in the lattice.
    pub beam_count: u32,
    /// Number of balls in the lattice.
    pub ball_count: u32,
    /// Minimum beam length.
    pub minlength: f64,
    /// Default beam radius.
    pub radius: f64,
    /// Optional clipping mesh object id.
    pub clipping_mesh_id: Option<NonZeroU32>,
    /// Clipping mode for the lattice.
    pub clippingmode: beamlattice::ClippingMode,
    /// Optional representation mesh object id.
    pub representation_mesh_id: Option<NonZeroU32>,
    /// Optional property group id.
    pub pid: Option<NonZeroU32>,
    /// Optional property index.
    pub pindex: Option<u32>,
    /// Optional ball radius.
    pub ball_radius: Option<f64>,
    /// Ball mode for the lattice.
    pub ball_mode: beamlattice::BallMode,
}

/// Stable view over a beam.
#[derive(Debug, Clone, PartialEq)]
pub struct BeamView {
    /// First vertex index.
    pub v1: u32,
    /// Second vertex index.
    pub v2: u32,
    /// Radius at the first vertex.
    pub r1: f64,
    /// Radius at the second vertex.
    pub r2: f64,
    /// Cap mode at the first vertex.
    pub cap1: beamlattice::CapMode,
    /// Cap mode at the second vertex.
    pub cap2: beamlattice::CapMode,
}

/// Stable view over a ball in a beam lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct BallView {
    /// Vertex index where the ball is centered.
    pub vindex: u32,
    /// Radius of the ball.
    pub radius: f64,
    /// Optional property index.
    pub pindex: Option<u32>,
    /// Optional property group id.
    pub pid: Option<NonZeroU32>,
}

/// Stable view over a beam set.
pub struct BeamSetView<'a> {
    set: &'a beamlattice::BeamSet,
}

impl<'a> BeamSetView<'a> {
    /// Returns the name of the beam set, if any.
    pub fn name(&self) -> Option<&str> {
        self.set.name.as_ref().map(|n| n.as_ref())
    }

    /// Returns the identifier of the beam set, if any.
    pub fn identifier(&self) -> Option<&str> {
        self.set.identifier.as_ref().map(|i| i.as_ref())
    }

    /// Returns the number of beams in the set.
    pub fn beam_count(&self) -> u32 {
        self.set.refs.len() as u32
    }

    /// Returns an iterator over the beam indices in the set.
    pub fn beam_refs(&self) -> impl Iterator<Item = u32> + '_ {
        self.set.refs.iter().map(|r| r.index)
    }

    /// Returns the number of balls in the set.
    pub fn ball_count(&self) -> u32 {
        self.set.ballref.len() as u32
    }

    /// Returns an iterator over the ball indices in the set.
    pub fn ball_refs(&self) -> impl Iterator<Item = u32> + '_ {
        self.set.ballref.iter().map(|r| r.index)
    }
}

impl<'a> LatticeView<'a> {
    /// Returns summary data for the beam lattice.
    pub fn data(&self) -> LatticeData {
        LatticeData {
            beam_count: self.lattice.beams.beam.len() as u32,
            ball_count: self
                .lattice
                .balls
                .as_ref()
                .map_or_else(|| 0, |b| b.ball.len()) as u32,
            minlength: self.lattice.minlength,
            radius: self.lattice.radius,
            clipping_mesh_id: self.lattice.clippingmesh.into(),
            clippingmode: self
                .lattice
                .clippingmode
                .clone()
                .unwrap_or(beamlattice::ClippingMode::None),
            representation_mesh_id: self.lattice.representationmesh.into(),
            pid: self.lattice.pid.into(),
            pindex: self.lattice.pindex.into(),
            ball_radius: self.lattice.ballradius,
            ball_mode: self
                .lattice
                .ballmode
                .clone()
                .unwrap_or(beamlattice::BallMode::None),
        }
    }

    /// Returns the number of beams in the lattice.
    pub fn beam_count(&self) -> usize {
        self.lattice.beams.beam.len()
    }

    /// Returns an iterator over the beams in the lattice.
    pub fn beams(&self) -> impl Iterator<Item = BeamView> {
        let default_radius = self.lattice.radius;
        let default_cap_mode = self
            .lattice
            .cap
            .clone()
            .unwrap_or(beamlattice::CapMode::Sphere);
        self.lattice.beams.beam.iter().map(move |beam| BeamView {
            v1: beam.v1,
            v2: beam.v2,
            r1: beam.r1.unwrap_or(default_radius),
            r2: beam.r2.unwrap_or(default_radius),
            cap1: beam.cap1.clone().unwrap_or(default_cap_mode.clone()),
            cap2: beam.cap2.clone().unwrap_or(default_cap_mode.clone()),
        })
    }

    /// Returns the number of balls in the lattice.
    pub fn ball_count(&self) -> usize {
        self.lattice
            .balls
            .as_ref()
            .map_or_else(|| 0, |b| b.ball.len())
    }

    /// Returns an iterator over the balls in the lattice, if any.
    pub fn balls(&self) -> Option<impl Iterator<Item = BallView>> {
        let default_ball_radius = self.lattice.ballradius.unwrap_or(0.0);
        self.lattice.balls.as_ref().map(|b| {
            b.ball.iter().map(move |ball| BallView {
                vindex: ball.vindex,
                radius: ball.r.unwrap_or(default_ball_radius),
                pindex: ball.p.into(),
                pid: ball.pid.into(),
            })
        })
    }

    /// Returns the number of beam sets in the lattice.
    pub fn beamset_count(&self) -> usize {
        self.lattice
            .beamsets
            .as_ref()
            .map_or_else(|| 0, |sets| sets.beamset.len())
    }

    /// Returns an iterator over the beam sets in the lattice, if any.
    pub fn beamsets(&self) -> Option<impl Iterator<Item = BeamSetView<'a>> + '_> {
        self.lattice
            .beamsets
            .as_ref()
            .map(|sets| sets.beamset.iter().map(|set| BeamSetView { set }))
    }
}

/// Stable view over a triangle set.
pub struct TriangleSetView<'a> {
    set: &'a TriangleSet,
}

impl<'a> TriangleSetView<'a> {
    /// Returns the name of the triangle set.
    pub fn name(&self) -> &str {
        self.set.name.as_ref()
    }

    /// Returns the identifier of the triangle set.
    pub fn identifier(&self) -> &str {
        self.set.identifier.as_ref()
    }

    /// Returns an iterator over the triangle indices in the set.
    pub fn triangles_iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.set.triangle_ref.iter().map(|r| r.index).chain(
            self.set
                .triangle_refrange
                .iter()
                .flat_map(|range| range.startindex..=range.endindex),
        )
    }
}

impl<'a> MeshObjectView<'a> {
    pub(crate) fn from_object(object: &'a Object) -> Option<Self> {
        object.get_mesh().map(|mesh| Self { object, mesh })
    }

    /// Returns the identifier of the mesh object.
    pub fn id(&self) -> u32 {
        self.object.id
    }

    /// Returns the name of the mesh object, if any.
    pub fn name(&self) -> Option<&'a str> {
        self.object.name.as_deref()
    }

    /// Returns the property group id of the mesh object.
    pub fn pid(&self) -> OptionalResourceId {
        self.object.pid
    }

    /// Returns the property index of the mesh object.
    pub fn pindex(&self) -> OptionalResourceIndex {
        self.object.pindex
    }

    /// Returns the UUID of the mesh object, if any.
    pub fn uuid(&self) -> Option<Cow<'a, str>> {
        self.object
            .uuid
            .as_ref()
            .and_then(|uuid| uuid.to_string())
            .map(Cow::Owned)
    }

    /// Returns the number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.mesh.vertices.vertex.len()
    }

    /// Returns the number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.mesh.triangles.triangle.len()
    }

    /// Returns true if the mesh has a beam lattice.
    pub fn has_beamlattice(&self) -> bool {
        self.mesh.beamlattice.is_some()
    }

    /// Returns true if the mesh has triangle sets.
    pub fn has_triangle_sets(&self) -> bool {
        self.mesh.trianglesets.is_some()
    }

    /// Returns an iterator over the vertex positions in the mesh.
    pub fn vertices(&self) -> impl Iterator<Item = [f64; 3]> {
        self.mesh
            .vertices
            .vertex
            .iter()
            .map(|v| [v.x.value(), v.y.value(), v.z.value()])
    }

    /// Returns an iterator over the triangle vertex indices in the mesh.
    pub fn triangles(&self) -> impl Iterator<Item = [u32; 3]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.v1, t.v2, t.v3])
    }

    /// Returns an iterator over the triangle property data in the mesh.
    pub fn triangles_data(&self) -> impl Iterator<Item = [Option<u32>; 4]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.p1.into(), t.p2.into(), t.p3.into(), t.pid.into()])
    }

    /// Returns the number of triangle sets in the mesh.
    pub fn triangle_set_count(&self) -> usize {
        self.mesh
            .trianglesets
            .as_ref()
            .map_or_else(|| 0, |sets| sets.trianglesets.len())
    }

    /// Returns an iterator over the triangle sets in the mesh, if any.
    pub fn triangle_sets(&self) -> Option<impl Iterator<Item = TriangleSetView<'a>> + '_> {
        self.mesh
            .trianglesets
            .as_ref()
            .map(|sets| sets.trianglesets.iter().map(|set| TriangleSetView { set }))
    }

    /// Returns the beam lattice of the mesh, if any.
    pub fn lattice(&self) -> Option<LatticeView<'a>> {
        self.mesh
            .beamlattice
            .as_ref()
            .map(|b| LatticeView { lattice: b })
    }
}

/// Stable view over a displacement mesh object.
pub struct DisplacementMeshObjectView<'a> {
    object: &'a Object,
    mesh: &'a displacement::DisplacementMesh,
}

impl<'a> DisplacementMeshObjectView<'a> {
    pub(crate) fn from_object(object: &'a Object) -> Option<Self> {
        object
            .get_displacement_mesh()
            .map(|mesh| Self { object, mesh })
    }

    /// Returns the identifier of the displacement mesh object.
    pub fn id(&self) -> u32 {
        self.object.id
    }

    /// Returns the name of the displacement mesh object, if any.
    pub fn name(&self) -> Option<Cow<'a, str>> {
        self.object.name.as_deref().map(Cow::Borrowed)
    }

    /// Returns true if the displacement mesh has a beam lattice.
    pub fn has_beamlattice(&self) -> bool {
        self.mesh.beamlattice.is_some()
    }

    /// Returns true if the displacement mesh has triangle sets.
    pub fn has_triangle_sets(&self) -> bool {
        self.mesh.trianglesets.is_some()
    }

    /// Returns an iterator over the vertex positions in the displacement mesh.
    pub fn vertices(&self) -> impl Iterator<Item = [f64; 3]> {
        self.mesh
            .vertices
            .vertex
            .iter()
            .map(|v| [v.x.value(), v.y.value(), v.z.value()])
    }

    /// Returns an iterator over the triangle vertex indices in the displacement mesh.
    pub fn triangles(&self) -> impl Iterator<Item = [u32; 3]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.v1, t.v2, t.v3])
    }

    /// Returns an iterator over the triangle property data in the displacement mesh.
    pub fn triangles_data(&self) -> impl Iterator<Item = [Option<u32>; 4]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.p1.into(), t.p2.into(), t.p3.into(), t.pid.into()])
    }

    /// Returns an iterator over the triangle displacement data in the displacement mesh.
    pub fn triangles_displacement_data(&self) -> impl Iterator<Item = [Option<u32>; 4]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.d1.into(), t.d2.into(), t.d3.into(), t.did.into()])
    }

    /// Returns the number of triangle sets in the displacement mesh.
    pub fn triangle_set_count(&self) -> usize {
        self.mesh
            .trianglesets
            .as_ref()
            .map_or_else(|| 0, |sets| sets.trianglesets.len())
    }

    /// Returns an iterator over the triangle sets in the displacement mesh, if any.
    pub fn triangle_sets(&self) -> Option<impl Iterator<Item = TriangleSetView<'a>> + '_> {
        self.mesh
            .trianglesets
            .as_ref()
            .map(|sets| sets.trianglesets.iter().map(|set| TriangleSetView { set }))
    }

    /// Returns the beam lattice of the displacement mesh, if any.
    pub fn lattice(&self) -> Option<LatticeView<'a>> {
        self.mesh
            .beamlattice
            .as_ref()
            .map(|b| LatticeView { lattice: b })
    }
}

/// Stable view over a component entry.
pub struct ComponentView<'a> {
    objectid: u32,
    transform: Option<[f64; 16]>,
    path: Option<&'a str>,
    uuid: Option<Cow<'a, str>>,
}

impl<'a> ComponentView<'a> {
    /// Returns the object id of the component.
    pub fn object_id(&self) -> u32 {
        self.objectid
    }

    /// Returns the transform matrix of the component, if any.
    pub fn transform(&self) -> Option<[f64; 16]> {
        self.transform
    }

    /// Returns the path of the component, if any.
    pub fn path(&self) -> Option<&str> {
        self.path
    }

    /// Returns the UUID of the component, if any.
    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }
}

/// Stable view over a components object.
pub struct ComponentsObjectView<'a> {
    object: &'a Object,
    components: &'a Components,
}

impl<'a> ComponentsObjectView<'a> {
    pub(crate) fn from_object(object: &'a Object) -> Option<Self> {
        object
            .get_components_object()
            .map(|components| Self { object, components })
    }

    /// Returns the identifier of the components object.
    pub fn id(&self) -> u32 {
        self.object.id
    }

    /// Returns the name of the components object, if any.
    pub fn name(&self) -> Option<Cow<'a, str>> {
        self.object.name.as_deref().map(Cow::Borrowed)
    }

    /// Returns the number of components in the object.
    pub fn component_count(&self) -> usize {
        self.components.component.len()
    }

    /// Returns an iterator over the components in the object.
    pub fn components(&self) -> impl Iterator<Item = ComponentView<'a>> + '_ {
        self.components.component.iter().map(|c| ComponentView {
            objectid: c.objectid,
            transform: c.transform.as_ref().map(Transform::to_column_major_matrix),
            path: c.path.as_ref().map(|path| path.as_str()),
            uuid: c
                .uuid
                .as_ref()
                .and_then(|uuid| uuid.to_string())
                .map(Cow::Owned),
        })
    }
}

/// Stable view over a boolean operand.
pub struct BooleanOperandView<'a> {
    objectid: u32,
    transform: Option<[f64; 16]>,
    path: Option<&'a str>,
}

impl<'a> BooleanOperandView<'a> {
    /// Returns the object id of the operand.
    pub fn object_id(&self) -> u32 {
        self.objectid
    }

    /// Returns the transform matrix of the operand, if any.
    pub fn transform(&self) -> Option<[f64; 16]> {
        self.transform
    }

    /// Returns the path of the operand, if any.
    pub fn path(&self) -> Option<&str> {
        self.path
    }
}

/// Stable view over a boolean shape object.
pub struct BooleanShapeView<'a> {
    object: &'a Object,
    shape: &'a boolean::BooleanShape,
}

impl<'a> BooleanShapeView<'a> {
    pub(crate) fn from_object(object: &'a Object) -> Option<Self> {
        object
            .get_boolean_shape_object()
            .map(|shape| Self { object, shape })
    }

    /// Returns the identifier of the boolean shape object.
    pub fn id(&self) -> u32 {
        self.object.id
    }

    /// Returns the name of the boolean shape object, if any.
    pub fn name(&self) -> Option<&'a str> {
        self.object.name.as_deref()
    }

    /// Returns the base object id of the boolean shape.
    pub fn base_objectid(&self) -> u32 {
        self.shape.objectid
    }

    /// Returns the boolean operation of the shape.
    pub fn operation(&self) -> boolean::BooleanOperation {
        self.shape.operation
    }

    /// Returns true if the operation is a union.
    pub fn is_union(&self) -> bool {
        matches!(self.shape.operation, boolean::BooleanOperation::Union)
    }

    /// Returns true if the operation is a difference.
    pub fn is_difference(&self) -> bool {
        matches!(self.shape.operation, boolean::BooleanOperation::Difference)
    }

    /// Returns true if the operation is an intersection.
    pub fn is_intersection(&self) -> bool {
        matches!(
            self.shape.operation,
            boolean::BooleanOperation::Intersection
        )
    }

    /// Returns an iterator over the boolean operands in the shape.
    pub fn booleans(&self) -> impl Iterator<Item = BooleanOperandView<'a>> + '_ {
        self.shape.booleans.iter().map(|b| BooleanOperandView {
            objectid: b.objectid,
            transform: b.transform.as_ref().map(Transform::to_column_major_matrix),
            path: b.path.as_ref().map(|p| p.as_str()),
        })
    }
}

/// Stable view over a build item.
pub struct ItemView<'a> {
    item: &'a Item,
}

impl<'a> ItemView<'a> {
    pub(crate) fn new(item: &'a Item) -> Self {
        Self { item }
    }

    /// Returns the object id of the build item.
    pub fn object_id(&self) -> u32 {
        self.item.objectid
    }

    /// Returns the transform matrix of the build item, if any.
    pub fn transform(&self) -> Option<[f64; 16]> {
        self.item
            .transform
            .as_ref()
            .map(Transform::to_column_major_matrix)
    }

    /// Returns the part number of the build item, if any.
    pub fn part_number(&self) -> Option<&str> {
        self.item.partnumber.as_deref()
    }

    /// Returns the path of the build item, if any.
    pub fn path(&self) -> Option<&str> {
        self.item.path.as_ref().map(|p| p.as_str())
    }

    /// Returns the UUID of the build item, if any.
    pub fn uuid(&self) -> Option<Cow<'a, str>> {
        self.item
            .uuid
            .as_ref()
            .and_then(|uuid| uuid.to_string())
            .map(Cow::Owned)
    }
}

/// Stable view over a slicestack reference.
pub struct SliceRefView<'a> {
    slicestack_id: u32,
    slicepath: &'a str,
}

impl<'a> SliceRefView<'a> {
    /// Returns the slice stack id of the reference.
    pub fn slicestack_id(&self) -> u32 {
        self.slicestack_id
    }

    /// Returns the slice path of the reference.
    pub fn slicepath(&self) -> &str {
        self.slicepath
    }
}

/// Stable view over a polygon.
pub struct PolygonView<'a> {
    polygon: &'a slice::Polygon,
    segment_count: usize,
}

impl<'a> PolygonView<'a> {
    /// Returns the number of segments in the polygon.
    pub fn segment_count(&self) -> usize {
        self.segment_count
    }

    /// Returns an iterator over the segment vertex pairs in the polygon.
    pub fn segments(self) -> impl Iterator<Item = [u32; 2]> {
        self.polygon
            .segment
            .iter()
            .scan(self.polygon.startv, |prev, s| {
                let pair = [*prev, s.v2];
                *prev = s.v2;
                Some(pair)
            })
    }

    /// Returns an iterator over the segment property data in the polygon.
    pub fn segments_data(self) -> impl Iterator<Item = [Option<u32>; 3]> {
        self.polygon
            .segment
            .iter()
            .map(|s| [s.p1.into(), s.p2.into(), s.pid.into()])
    }
}

/// Stable view over a slice.
pub struct SliceView<'a> {
    slice: &'a slice::Slice,
}

impl<'a> SliceView<'a> {
    fn new(slice: &'a slice::Slice) -> Self {
        Self { slice }
    }

    /// Returns the top Z position of the slice.
    pub fn ztop(&self) -> f64 {
        self.slice.ztop.value()
    }

    /// Returns the number of vertices in the slice, if any.
    pub fn vertex_count(&self) -> Option<usize> {
        self.slice.vertices.as_ref().map(|v| v.vertex.len())
    }

    /// Returns an iterator over the vertex positions in the slice, if any.
    pub fn vertices(&self) -> Option<impl Iterator<Item = [f64; 2]>> {
        self.slice
            .vertices
            .as_ref()
            .map(|v| v.vertex.iter().map(|v| [v.x.into(), v.y.into()]))
    }

    /// Returns the number of polygons in the slice.
    pub fn polygon_count(&self) -> usize {
        self.slice.polygon.len()
    }

    /// Returns an iterator over the polygons in the slice.
    pub fn polygons(&self) -> impl Iterator<Item = PolygonView<'a>> + '_ {
        self.slice.polygon.iter().map(|p| PolygonView {
            polygon: p,
            segment_count: p.segment.len(),
        })
    }
}

/// Stable view over a slice stack.
pub struct SliceStackView<'a> {
    stack: &'a slice::SliceStack,
}

impl<'a> SliceStackView<'a> {
    pub(crate) fn new(stack: &'a slice::SliceStack) -> Self {
        Self { stack }
    }

    /// Returns the identifier of the slice stack.
    pub fn id(&self) -> u32 {
        self.stack.id
    }

    /// Returns the bottom Z position of the slice stack, if any.
    pub fn zbottom(&self) -> Option<f64> {
        self.stack.zbottom.map(|d| d.value())
    }

    /// Returns true if the slice stack contains owned slices.
    pub fn has_owned_slices(&self) -> bool {
        self.stack.has_owned_slices()
    }

    /// Returns the number of slices in the stack.
    pub fn slice_count(&self) -> usize {
        self.stack.slice.len()
    }

    /// Returns the number of slice references in the stack.
    pub fn sliceref_count(&self) -> usize {
        self.stack.sliceref.len()
    }

    /// Returns an iterator over the slice references in the stack.
    pub fn slicerefs(&self) -> impl Iterator<Item = SliceRefView<'a>> + '_ {
        self.stack.sliceref.iter().map(|r| SliceRefView {
            slicestack_id: r.slicestackid,
            slicepath: r.slicepath.as_str(),
        })
    }

    /// Returns an iterator over the slices in the stack.
    pub fn slices(&self) -> impl Iterator<Item = SliceView<'a>> + '_ {
        self.stack.slice.iter().map(SliceView::new)
    }
}

/// Stable view over a color group.
pub struct ColorGroupView<'a> {
    group: &'a material::ColorGroup,
}

impl<'a> ColorGroupView<'a> {
    pub(crate) fn new(group: &'a material::ColorGroup) -> Self {
        Self { group }
    }

    /// Returns the identifier of the color group.
    pub fn id(&self) -> u32 {
        self.group.id
    }

    /// Returns the number of colors in the group.
    pub fn color_count(&self) -> usize {
        self.group.color.len()
    }

    /// Returns the color at the given index, if any.
    pub fn color_at(&self, index: usize) -> Option<Color> {
        self.group.color.get(index).map(|c| c.color)
    }
}

/// Stable view over a texture 2d group.
pub struct Texture2DGroupView<'a> {
    group: &'a material::Texture2DGroup,
}

impl<'a> Texture2DGroupView<'a> {
    pub(crate) fn new(group: &'a material::Texture2DGroup) -> Self {
        Self { group }
    }

    /// Returns the identifier of the texture 2D group.
    pub fn id(&self) -> u32 {
        self.group.id
    }

    /// Returns the texture id of the group.
    pub fn texid(&self) -> u32 {
        self.group.texid
    }

    /// Returns the number of texture coordinates in the group.
    pub fn texcoord_count(&self) -> usize {
        self.group.tex2coord.len()
    }

    /// Returns an iterator over the texture coordinates in the group.
    pub fn tex_coords(&self) -> impl Iterator<Item = [f64; 2]> {
        self.group
            .tex2coord
            .iter()
            .map(|t| [t.u.into(), t.v.into()])
    }
}

/// Stable view over composite materials.
pub struct CompositeMaterialsView<'a> {
    materials: &'a material::CompositeMaterials,
}

impl<'a> CompositeMaterialsView<'a> {
    pub(crate) fn new(materials: &'a material::CompositeMaterials) -> Self {
        Self { materials }
    }

    /// Returns the identifier of the composite materials.
    pub fn id(&self) -> u32 {
        self.materials.id
    }

    /// Returns the number of composite entries in the materials.
    pub fn composite_count(&self) -> usize {
        self.materials.composite.len()
    }
}

/// Stable view over multi-properties.
pub struct MultiPropertiesView<'a> {
    props: &'a material::MultiProperties,
}

impl<'a> MultiPropertiesView<'a> {
    pub(crate) fn new(props: &'a material::MultiProperties) -> Self {
        Self { props }
    }

    /// Returns the identifier of the multi-properties.
    pub fn id(&self) -> u32 {
        self.props.id
    }

    /// Returns the number of multi-property entries.
    pub fn multi_count(&self) -> usize {
        self.props.multi.len()
    }
}

/// Stable view over a texture2d resource.
pub struct Texture2DView<'a> {
    texture: &'a material::Texture2D,
}

impl<'a> Texture2DView<'a> {
    pub(crate) fn new(texture: &'a material::Texture2D) -> Self {
        Self { texture }
    }

    /// Returns the identifier of the texture 2D resource.
    pub fn id(&self) -> u32 {
        self.texture.id
    }

    /// Returns the path of the texture 2D resource.
    pub fn path(&self) -> &str {
        self.texture.path.as_str()
    }

    /// Returns the content type of the texture.
    pub fn content_type(&self) -> material::TextureContentType {
        self.texture.contenttype.clone()
    }

    /// Returns the tile style in the U direction.
    pub fn tile_style_u(&self) -> material::TileStyle {
        self.texture.tilestyleu.unwrap_or(material::TileStyle::Wrap)
    }

    /// Returns the tile style in the V direction.
    pub fn tile_style_v(&self) -> material::TileStyle {
        self.texture.tilestylev.unwrap_or(material::TileStyle::Wrap)
    }

    /// Returns the filter mode of the texture.
    pub fn filter(&self) -> material::Filter {
        self.texture.filter.unwrap_or(material::Filter::Auto)
    }
}

/// Stable view over a displacement2d resource.
pub struct Displacement2DView<'a> {
    displacement: &'a displacement::Displacement2D,
}

impl<'a> Displacement2DView<'a> {
    pub(crate) fn new(displacement: &'a displacement::Displacement2D) -> Self {
        Self { displacement }
    }

    /// Returns the identifier of the displacement 2D resource.
    pub fn id(&self) -> u32 {
        self.displacement.id
    }
}

/// Stable view over a norm vector group.
pub struct NormVectorGroupView<'a> {
    group: &'a displacement::NormVectorGroup,
}

impl<'a> NormVectorGroupView<'a> {
    pub(crate) fn new(group: &'a displacement::NormVectorGroup) -> Self {
        Self { group }
    }

    /// Returns the identifier of the normal vector group.
    pub fn id(&self) -> u32 {
        self.group.id
    }

    /// Returns an iterator over the normal vectors in the group.
    pub fn norm_vectors(&self) -> impl Iterator<Item = [f64; 3]> {
        self.group
            .normvector
            .iter()
            .map(|n| [n.x.into(), n.y.into(), n.z.into()])
    }
}

/// Stable view over a displacement 2d group.
pub struct Disp2DGroupView<'a> {
    group: &'a displacement::Disp2DGroup,
}

/// Stable view over a 2D displacement coordinate.
pub struct Disp2DCoordView {
    /// U texture coordinate.
    pub u: f64,
    /// V texture coordinate.
    pub v: f64,
    /// Index into the normal vector group.
    pub norm_index: u32,
    /// Scaling factor for displacement.
    pub f: f64,
}

impl<'a> Disp2DGroupView<'a> {
    pub(crate) fn new(group: &'a displacement::Disp2DGroup) -> Self {
        Self { group }
    }

    /// Returns the identifier of the displacement 2D group.
    pub fn id(&self) -> u32 {
        self.group.id
    }

    /// Returns the displacement map id of the group.
    pub fn displacement_map_id(&self) -> u32 {
        self.group.dispid
    }

    /// Returns the normal vector group id of the group.
    pub fn norm_vector_group_id(&self) -> u32 {
        self.group.nid
    }

    /// Returns the height of the displacement.
    pub fn height(&self) -> f64 {
        self.group.height.into()
    }

    /// Returns the offset of the displacement.
    pub fn offset(&self) -> f64 {
        self.group.offset.map_or(0.0, |o| o.into())
    }

    /// Returns an iterator over the displacement 2D coordinates in the group.
    pub fn disp_2d_coords(&self) -> impl Iterator<Item = Disp2DCoordView> {
        self.group.disp2dcoord.iter().map(|d| Disp2DCoordView {
            u: d.u.into(),
            v: d.v.into(),
            norm_index: d.n,
            f: d.f.map_or(0.0, |f| f.into()),
        })
    }
}

/// Stable view over base materials.
pub struct BaseMaterialsView<'a> {
    materials: &'a BaseMaterials,
}

impl<'a> BaseMaterialsView<'a> {
    pub(crate) fn new(materials: &'a BaseMaterials) -> Self {
        Self { materials }
    }

    /// Returns the identifier of the base materials.
    pub fn id(&self) -> u32 {
        self.materials.id
    }

    /// Returns the number of base materials entries.
    pub fn base_count(&self) -> usize {
        self.materials.base.len()
    }
}

/// Stable material property value.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialPropertyValue {
    /// Solid color property.
    Color(Color),
    /// Texture coordinate property.
    TextureCoord {
        /// U texture coordinate.
        u: f64,
        /// V texture coordinate.
        v: f64,
    },
    /// Composite material values.
    Composite {
        /// Composite material values.
        values: Vec<f64>,
    },
    /// Multi-property indices.
    Multi {
        /// Multi-property indices.
        indices: Vec<u32>,
    },
    /// Base material name and color.
    Base {
        /// Base material name.
        name: String,
        /// Base material display color.
        displaycolor: String,
    },
}

/// Returns a stable view over the given model.
pub fn get_model_view<'a>(model: &'a Model) -> ModelView<'a> {
    ModelView::new(model)
}

/// Returns a view over the object with the given id from the model, if found.
pub fn get_object_from_model<'a>(object_id: u32, model: &'a Model) -> Option<ObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .find(|o| o.id == object_id)
        .map(ObjectView::new)
}

/// Returns an iterator over all objects in the model.
pub fn get_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ObjectView<'a>> {
    model.resources.object.iter().map(ObjectView::new)
}

/// Returns an iterator over all mesh objects in the model.
pub fn get_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MeshObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(MeshObjectView::from_object)
}

/// Returns an iterator over all displacement mesh objects in the model.
pub fn get_displacement_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = DisplacementMeshObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(DisplacementMeshObjectView::from_object)
}

/// Returns an iterator over all components objects in the model.
pub fn get_components_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ComponentsObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(ComponentsObjectView::from_object)
}

/// Returns an iterator over all boolean shape objects in the model.
pub fn get_boolean_shape_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = BooleanShapeView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(BooleanShapeView::from_object)
}

/// Returns an iterator over all build items in the model.
pub fn get_items_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ItemView<'a>> {
    model.build.item.iter().map(ItemView::new)
}

/// Returns an iterator over all slice stacks in the model.
pub fn get_slice_stacks_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = SliceStackView<'a>> {
    model.resources.slicestack.iter().map(SliceStackView::new)
}

/// Returns a view over the slice stack with the given id from the model, if found.
pub fn get_slice_stack_from_model<'a>(
    slicestack_id: u32,
    model: &'a Model,
) -> Option<SliceStackView<'a>> {
    model
        .resources
        .slicestack
        .iter()
        .find(|s| s.id == slicestack_id)
        .map(SliceStackView::new)
}

/// Returns an iterator over all color groups in the model.
pub fn get_color_groups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ColorGroupView<'a>> {
    model.resources.colorgroup.iter().map(ColorGroupView::new)
}

/// Returns a view over the color group with the given id from the model, if found.
pub fn get_color_group_by_id<'a>(
    colorgroup_id: u32,
    model: &'a Model,
) -> Option<ColorGroupView<'a>> {
    model
        .resources
        .colorgroup
        .iter()
        .find(|cg| cg.id == colorgroup_id)
        .map(ColorGroupView::new)
}

/// Returns an iterator over all texture 2D groups in the model.
pub fn get_texture2d_groups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = Texture2DGroupView<'a>> {
    model
        .resources
        .texture2dgroup
        .iter()
        .map(Texture2DGroupView::new)
}

/// Returns a view over the texture 2D group with the given id from the model, if found.
pub fn get_texture2d_group_by_id<'a>(
    texture2dgroup_id: u32,
    model: &'a Model,
) -> Option<Texture2DGroupView<'a>> {
    model
        .resources
        .texture2dgroup
        .iter()
        .find(|tg| tg.id == texture2dgroup_id)
        .map(Texture2DGroupView::new)
}

/// Returns an iterator over all composite materials in the model.
pub fn get_composite_materials_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = CompositeMaterialsView<'a>> {
    model
        .resources
        .compositematerials
        .iter()
        .map(CompositeMaterialsView::new)
}

/// Returns a view over the composite materials with the given id from the model, if found.
pub fn get_composite_materials_by_id<'a>(
    compositematerials_id: u32,
    model: &'a Model,
) -> Option<CompositeMaterialsView<'a>> {
    model
        .resources
        .compositematerials
        .iter()
        .find(|cm| cm.id == compositematerials_id)
        .map(CompositeMaterialsView::new)
}

/// Returns an iterator over all multi-properties in the model.
pub fn get_multi_properties_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MultiPropertiesView<'a>> {
    model
        .resources
        .multiproperties
        .iter()
        .map(MultiPropertiesView::new)
}

/// Returns a view over the multi-properties with the given id from the model, if found.
pub fn get_multi_properties_by_id<'a>(
    multiproperties_id: u32,
    model: &'a Model,
) -> Option<MultiPropertiesView<'a>> {
    model
        .resources
        .multiproperties
        .iter()
        .find(|mp| mp.id == multiproperties_id)
        .map(MultiPropertiesView::new)
}

/// Returns an iterator over all texture 2D resources in the model.
pub fn get_texture2ds_from_model<'a>(model: &'a Model) -> impl Iterator<Item = Texture2DView<'a>> {
    model.resources.texture2d.iter().map(Texture2DView::new)
}

/// Returns a view over the texture 2D resource with the given id from the model, if found.
pub fn get_texture2d_by_id<'a>(texture2d_id: u32, model: &'a Model) -> Option<Texture2DView<'a>> {
    model
        .resources
        .texture2d
        .iter()
        .find(|t| t.id == texture2d_id)
        .map(Texture2DView::new)
}

/// Returns an iterator over all displacement 2D resources in the model.
pub fn get_displacement2ds_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = Displacement2DView<'a>> {
    model
        .resources
        .displacement2d
        .iter()
        .map(Displacement2DView::new)
}

/// Returns a view over the displacement 2D resource with the given id from the model, if found.
pub fn get_displacement2d_by_id<'a>(
    displacement2d_id: u32,
    model: &'a Model,
) -> Option<Displacement2DView<'a>> {
    model
        .resources
        .displacement2d
        .iter()
        .find(|d| d.id == displacement2d_id)
        .map(Displacement2DView::new)
}

/// Returns an iterator over all normal vector groups in the model.
pub fn get_normvectorgroups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = NormVectorGroupView<'a>> {
    model
        .resources
        .normvectorgroup
        .iter()
        .map(NormVectorGroupView::new)
}

/// Returns a view over the normal vector group with the given id from the model, if found.
pub fn get_normvectorgroup_by_id<'a>(
    normvectorgroup_id: u32,
    model: &'a Model,
) -> Option<NormVectorGroupView<'a>> {
    model
        .resources
        .normvectorgroup
        .iter()
        .find(|n| n.id == normvectorgroup_id)
        .map(NormVectorGroupView::new)
}

/// Returns an iterator over all displacement 2D groups in the model.
pub fn get_disp2dgroups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = Disp2DGroupView<'a>> {
    model.resources.disp2dgroup.iter().map(Disp2DGroupView::new)
}

/// Returns a view over the displacement 2D group with the given id from the model, if found.
pub fn get_disp2dgroup_by_id<'a>(
    disp2dgroup_id: u32,
    model: &'a Model,
) -> Option<Disp2DGroupView<'a>> {
    model
        .resources
        .disp2dgroup
        .iter()
        .find(|d| d.id == disp2dgroup_id)
        .map(Disp2DGroupView::new)
}

/// Returns an iterator over all base materials in the model.
pub fn get_base_materials_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = BaseMaterialsView<'a>> {
    model
        .resources
        .basematerials
        .iter()
        .map(BaseMaterialsView::new)
}

/// Returns a view over the base materials with the given id from the model, if found.
pub fn get_base_materials_by_id<'a>(
    basematerials_id: u32,
    model: &'a Model,
) -> Option<BaseMaterialsView<'a>> {
    model
        .resources
        .basematerials
        .iter()
        .find(|bm| bm.id == basematerials_id)
        .map(BaseMaterialsView::new)
}

/// Resolves a material property from the model using the given property group id and index.
pub fn resolve_material_property<'a>(
    pid: OptionalResourceId,
    pindex: OptionalResourceIndex,
    model: &'a Model,
) -> Option<MaterialPropertyValue> {
    let pid = pid.get()?;
    let pindex = pindex.get()? as usize;

    if let Some(cg_ref) = get_color_group_by_id(pid, model) {
        return cg_ref
            .group
            .color
            .get(pindex)
            .map(|c| MaterialPropertyValue::Color(c.color));
    }

    if let Some(tg_ref) = get_texture2d_group_by_id(pid, model) {
        return tg_ref
            .group
            .tex2coord
            .get(pindex)
            .map(|tc| MaterialPropertyValue::TextureCoord {
                u: tc.u.value(),
                v: tc.v.value(),
            });
    }

    if let Some(cm_ref) = get_composite_materials_by_id(pid, model) {
        return cm_ref
            .materials
            .composite
            .get(pindex)
            .map(|c| MaterialPropertyValue::Composite {
                values: c.values.iter().map(|d| d.value()).collect(),
            });
    }

    if let Some(mp_ref) = get_multi_properties_by_id(pid, model) {
        return mp_ref
            .props
            .multi
            .get(pindex)
            .map(|m| MaterialPropertyValue::Multi {
                indices: m.pindices.iter().copied().collect(),
            });
    }

    if let Some(bm_ref) = get_base_materials_by_id(pid, model) {
        return bm_ref
            .materials
            .base
            .get(pindex)
            .map(|b| MaterialPropertyValue::Base {
                name: b.name.to_string(),
                displaycolor: b.displaycolor.to_string(),
            });
    }

    None
}

/// Returns the texture 2D resource associated with the given texture 2D group from the model, if found.
pub fn get_texture_for_group<'a>(
    texture2dgroup: &material::Texture2DGroup,
    model: &'a Model,
) -> Option<Texture2DView<'a>> {
    get_texture2d_by_id(texture2dgroup.texid, model)
}

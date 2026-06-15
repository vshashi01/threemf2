//! Query API for inspecting model-level data with stable views.

#![allow(clippy::needless_lifetimes)]

use crate::core::{
    OptionalResourceId, OptionalResourceIndex,
    beamlattice::{BeamLattice, BeamSet, CapMode},
    boolean::{BooleanOperation, BooleanShape},
    build::Item,
    builder::{BallMode, ClippingMode},
    component::Components,
    displacement::{Disp2DGroup, Displacement2D, DisplacementMesh, NormVectorGroup},
    material::{self, ColorGroup, CompositeMaterials, MultiProperties, Texture2D, Texture2DGroup},
    mesh::Mesh,
    metadata::Metadata,
    model::{Model, Unit},
    object::{Object, ObjectKind},
    resources::BaseMaterials,
    slice::{Polygon, Slice, SliceStack},
    transform::Transform,
    triangle_set::TriangleSet,
    types::{Color, UuidResource},
};
use crate::threemf_namespaces::ThreemfNamespace;

use std::{borrow::Cow, num::NonZeroU32};

/// Stable classification for object kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKindView {
    Mesh,
    Components,
    BooleanShape,
    DisplacementMesh,
}

pub struct ModelView<'a> {
    model: &'a Model,
}

impl<'a> ModelView<'a> {
    pub(crate) fn new(model: &'a Model) -> Self {
        Self { model }
    }

    pub fn unit(&self) -> Unit {
        self.model.unit.clone().unwrap_or(Unit::Millimeter)
    }

    pub fn object_count(&self) -> usize {
        self.model.resources.object.len()
    }

    pub fn build_item_count(&self) -> usize {
        self.model.build.item.len()
    }

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

    pub fn slice_stack_count(&self) -> usize {
        self.model.resources.slicestack.len()
    }

    pub fn color_group_count(&self) -> usize {
        self.model.resources.colorgroup.len()
    }

    pub fn texture2d_group_count(&self) -> usize {
        self.model.resources.texture2dgroup.len()
    }

    pub fn texture2d_count(&self) -> usize {
        self.model.resources.texture2d.len()
    }

    pub fn composite_materials_count(&self) -> usize {
        self.model.resources.compositematerials.len()
    }

    pub fn multi_properties_count(&self) -> usize {
        self.model.resources.multiproperties.len()
    }

    pub fn base_materials_count(&self) -> usize {
        self.model.resources.basematerials.len()
    }

    pub fn displacement2d_count(&self) -> usize {
        self.model.resources.displacement2d.len()
    }

    pub fn normvectorgroup_count(&self) -> usize {
        self.model.resources.normvectorgroup.len()
    }

    pub fn disp2dgroup_count(&self) -> usize {
        self.model.resources.disp2dgroup.len()
    }

    pub fn metadata_count(&self) -> usize {
        self.model.metadata.len()
    }

    pub fn metadata_iter(&self) -> impl Iterator<Item = MetadataView<'a>> + '_ {
        self.model.metadata.iter().map(MetadataView::new)
    }

    pub fn required_extensions(&self) -> Option<&[ThreemfNamespace]> {
        let extensions = self.model.requiredextensions.get();
        if extensions.is_empty() {
            None
        } else {
            Some(extensions)
        }
    }

    pub fn recommended_extensions(&self) -> Option<&[ThreemfNamespace]> {
        let extensions = self.model.recommendedextensions.get();
        if extensions.is_empty() {
            None
        } else {
            Some(extensions)
        }
    }

    pub fn used_namespaces(&self) -> impl Iterator<Item = ThreemfNamespace> {
        self.model.used_namespaces().into_iter()
    }
}

pub struct MetadataView<'a> {
    metadata: &'a Metadata,
}

impl<'a> MetadataView<'a> {
    pub(crate) fn new(metadata: &'a Metadata) -> Self {
        Self { metadata }
    }

    pub fn name(&self) -> &str {
        self.metadata.name.as_ref()
    }

    pub fn preserve(&self) -> Option<bool> {
        self.metadata.preserve.as_ref().map(|p| p.0)
    }

    pub fn value(&self) -> Option<&str> {
        self.metadata.value.as_ref().map(|v| v.as_ref())
    }
}

/// A stable view over a model object.
pub struct ObjectView<'a> {
    object: &'a Object,
}

impl<'a> ObjectView<'a> {
    pub(crate) fn new(object: &'a Object) -> Self {
        Self { object }
    }

    pub fn id(&self) -> u32 {
        self.object.id
    }

    pub fn name(&self) -> Option<&'a str> {
        self.object.name.as_deref()
    }

    pub fn part_number(&self) -> Option<&'a str> {
        self.object.partnumber.as_deref()
    }

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

    pub fn kind(&self) -> ObjectKindView {
        match &self.object.kind {
            Some(ObjectKind::Mesh(_)) => ObjectKindView::Mesh,
            Some(ObjectKind::Components(_)) => ObjectKindView::Components,
            Some(ObjectKind::BooleanShape(_)) => ObjectKindView::BooleanShape,
            Some(ObjectKind::DisplacementMesh(_)) => ObjectKindView::DisplacementMesh,
            None => panic!("Invalid object found"),
        }
    }

    pub fn is_mesh(&self) -> bool {
        matches!(self.object.kind, Some(ObjectKind::Mesh(_)))
    }

    pub fn is_components(&self) -> bool {
        matches!(self.object.kind, Some(ObjectKind::Components(_)))
    }

    pub fn is_boolean_shape(&self) -> bool {
        matches!(self.object.kind, Some(ObjectKind::BooleanShape(_)))
    }

    pub fn is_displacement_mesh(&self) -> bool {
        matches!(self.object.kind, Some(ObjectKind::DisplacementMesh(_)))
    }

    pub fn pid(&self) -> OptionalResourceId {
        self.object.pid
    }

    pub fn pindex(&self) -> OptionalResourceIndex {
        self.object.pindex
    }

    pub fn slicepath(&self) -> Option<&str> {
        self.object.slicepath.as_ref().map(|p| p.as_str())
    }

    pub fn slicestack_id(&self) -> Option<u32> {
        self.object.slicestackid.get()
    }
}

/// Stable view over a mesh object.
pub struct MeshObjectView<'a> {
    object: &'a Object,
    mesh: &'a Mesh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatticeView<'a> {
    lattice: &'a BeamLattice,
}

pub struct LatticeData {
    pub beam_count: u32,
    pub ball_count: u32,
    pub minlength: f64,
    pub radius: f64,
    pub clipping_mesh_id: Option<NonZeroU32>,
    pub clippingmode: ClippingMode,
    pub representation_mesh_id: Option<NonZeroU32>,
    pub pid: Option<NonZeroU32>,
    pub pindex: Option<u32>,
    pub ball_radius: Option<f64>,
    pub ball_mode: BallMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamView {
    pub v1: u32,
    pub v2: u32,
    pub r1: f64,
    pub r2: f64,
    pub cap1: CapMode,
    pub cap2: CapMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BallView {
    pub vindex: u32,
    pub radius: f64,
    pub pindex: Option<u32>,
    pub pid: Option<NonZeroU32>,
}

pub struct BeamSetView<'a> {
    set: &'a BeamSet,
}

impl<'a> BeamSetView<'a> {
    pub fn name(&self) -> Option<&str> {
        self.set.name.as_ref().map(|n| n.as_ref())
    }

    pub fn identifier(&self) -> Option<&str> {
        self.set.identifier.as_ref().map(|i| i.as_ref())
    }

    pub fn beam_count(&self) -> u32 {
        self.set.refs.len() as u32
    }

    pub fn beam_refs(&self) -> impl Iterator<Item = u32> + '_ {
        self.set.refs.iter().map(|r| r.index)
    }

    pub fn ball_count(&self) -> u32 {
        self.set.ballref.len() as u32
    }

    pub fn ball_refs(&self) -> impl Iterator<Item = u32> + '_ {
        self.set.ballref.iter().map(|r| r.index)
    }
}

impl<'a> LatticeView<'a> {
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
                .unwrap_or(ClippingMode::None),
            representation_mesh_id: self.lattice.representationmesh.into(),
            pid: self.lattice.pid.into(),
            pindex: self.lattice.pindex.into(),
            ball_radius: self.lattice.ballradius,
            ball_mode: self.lattice.ballmode.clone().unwrap_or(BallMode::None),
        }
    }

    pub fn beam_count(&self) -> usize {
        self.lattice.beams.beam.len()
    }

    pub fn beams(&self) -> impl Iterator<Item = BeamView> {
        let default_radius = self.lattice.radius;
        let default_cap_mode = self.lattice.cap.clone().unwrap_or(CapMode::Sphere);
        self.lattice.beams.beam.iter().map(move |beam| BeamView {
            v1: beam.v1,
            v2: beam.v2,
            r1: beam.r1.unwrap_or(default_radius),
            r2: beam.r2.unwrap_or(default_radius),
            cap1: beam.cap1.clone().unwrap_or(default_cap_mode.clone()),
            cap2: beam.cap2.clone().unwrap_or(default_cap_mode.clone()),
        })
    }

    pub fn ball_count(&self) -> usize {
        self.lattice
            .balls
            .as_ref()
            .map_or_else(|| 0, |b| b.ball.len())
    }

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

    pub fn beamset_count(&self) -> usize {
        self.lattice
            .beamsets
            .as_ref()
            .map_or_else(|| 0, |sets| sets.beamset.len())
    }

    pub fn beamsets(&self) -> Option<impl Iterator<Item = BeamSetView<'a>> + '_> {
        self.lattice
            .beamsets
            .as_ref()
            .map(|sets| sets.beamset.iter().map(|set| BeamSetView { set }))
    }
}

pub struct TriangleSetView<'a> {
    set: &'a TriangleSet,
}

impl<'a> TriangleSetView<'a> {
    pub fn name(&self) -> &str {
        self.set.name.as_ref()
    }

    pub fn identifier(&self) -> &str {
        self.set.identifier.as_ref()
    }

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

    pub fn id(&self) -> u32 {
        self.object.id
    }

    pub fn name(&self) -> Option<&'a str> {
        self.object.name.as_deref()
    }

    pub fn pid(&self) -> OptionalResourceId {
        self.object.pid
    }

    pub fn pindex(&self) -> OptionalResourceIndex {
        self.object.pindex
    }

    pub fn uuid(&self) -> Option<Cow<'a, str>> {
        self.object
            .uuid
            .as_ref()
            .and_then(|uuid| uuid.to_string())
            .map(Cow::Owned)
    }

    pub fn vertex_count(&self) -> usize {
        self.mesh.vertices.vertex.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.mesh.triangles.triangle.len()
    }

    pub fn has_beamlattice(&self) -> bool {
        self.mesh.beamlattice.is_some()
    }

    pub fn has_triangle_sets(&self) -> bool {
        self.mesh.trianglesets.is_some()
    }

    pub fn vertices(&self) -> impl Iterator<Item = [f64; 3]> {
        self.mesh
            .vertices
            .vertex
            .iter()
            .map(|v| [v.x.value(), v.y.value(), v.z.value()])
    }

    pub fn triangles(&self) -> impl Iterator<Item = [u32; 3]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.v1, t.v2, t.v3])
    }

    pub fn triangles_data(&self) -> impl Iterator<Item = [Option<u32>; 4]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.p1.into(), t.p2.into(), t.p3.into(), t.pid.into()])
    }

    pub fn triangle_set_count(&self) -> usize {
        self.mesh
            .trianglesets
            .as_ref()
            .map_or_else(|| 0, |sets| sets.trianglesets.len())
    }

    pub fn triangle_sets(&self) -> Option<impl Iterator<Item = TriangleSetView<'a>> + '_> {
        self.mesh
            .trianglesets
            .as_ref()
            .map(|sets| sets.trianglesets.iter().map(|set| TriangleSetView { set }))
    }

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
    mesh: &'a DisplacementMesh,
}

impl<'a> DisplacementMeshObjectView<'a> {
    pub(crate) fn from_object(object: &'a Object) -> Option<Self> {
        object
            .get_displacement_mesh()
            .map(|mesh| Self { object, mesh })
    }

    pub fn id(&self) -> u32 {
        self.object.id
    }

    pub fn name(&self) -> Option<Cow<'a, str>> {
        self.object.name.as_deref().map(Cow::Borrowed)
    }

    pub fn has_beamlattice(&self) -> bool {
        self.mesh.beamlattice.is_some()
    }

    pub fn has_triangle_sets(&self) -> bool {
        self.mesh.trianglesets.is_some()
    }

    pub fn vertices(&self) -> impl Iterator<Item = [f64; 3]> {
        self.mesh
            .vertices
            .vertex
            .iter()
            .map(|v| [v.x.value(), v.y.value(), v.z.value()])
    }

    pub fn triangles(&self) -> impl Iterator<Item = [u32; 3]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.v1, t.v2, t.v3])
    }

    pub fn triangles_data(&self) -> impl Iterator<Item = [Option<u32>; 4]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.p1.into(), t.p2.into(), t.p3.into(), t.pid.into()])
    }

    pub fn triangles_displacement_data(&self) -> impl Iterator<Item = [Option<u32>; 4]> {
        self.mesh
            .triangles
            .triangle
            .iter()
            .map(|t| [t.d1.into(), t.d2.into(), t.d3.into(), t.did.into()])
    }

    pub fn triangle_set_count(&self) -> usize {
        self.mesh
            .trianglesets
            .as_ref()
            .map_or_else(|| 0, |sets| sets.trianglesets.len())
    }

    pub fn triangle_sets(&self) -> Option<impl Iterator<Item = TriangleSetView<'a>> + '_> {
        self.mesh
            .trianglesets
            .as_ref()
            .map(|sets| sets.trianglesets.iter().map(|set| TriangleSetView { set }))
    }

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
    pub fn object_id(&self) -> u32 {
        self.objectid
    }

    pub fn transform(&self) -> Option<[f64; 16]> {
        self.transform
    }

    pub fn path(&self) -> Option<&str> {
        self.path
    }

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

    pub fn id(&self) -> u32 {
        self.object.id
    }

    pub fn name(&self) -> Option<Cow<'a, str>> {
        self.object.name.as_deref().map(Cow::Borrowed)
    }

    pub fn component_count(&self) -> usize {
        self.components.component.len()
    }

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
    pub fn object_id(&self) -> u32 {
        self.objectid
    }

    pub fn transform(&self) -> Option<[f64; 16]> {
        self.transform
    }

    pub fn path(&self) -> Option<&str> {
        self.path
    }
}

/// Stable view over a boolean shape object.
pub struct BooleanShapeView<'a> {
    object: &'a Object,
    shape: &'a BooleanShape,
}

impl<'a> BooleanShapeView<'a> {
    pub(crate) fn from_object(object: &'a Object) -> Option<Self> {
        object
            .get_boolean_shape_object()
            .map(|shape| Self { object, shape })
    }

    pub fn id(&self) -> u32 {
        self.object.id
    }

    pub fn name(&self) -> Option<&'a str> {
        self.object.name.as_deref()
    }

    pub fn base_objectid(&self) -> u32 {
        self.shape.objectid
    }

    pub fn operation(&self) -> BooleanOperation {
        self.shape.operation
    }

    pub fn is_union(&self) -> bool {
        matches!(self.shape.operation, BooleanOperation::Union)
    }

    pub fn is_difference(&self) -> bool {
        matches!(self.shape.operation, BooleanOperation::Difference)
    }

    pub fn is_intersection(&self) -> bool {
        matches!(self.shape.operation, BooleanOperation::Intersection)
    }

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

    pub fn object_id(&self) -> u32 {
        self.item.objectid
    }

    pub fn transform(&self) -> Option<[f64; 16]> {
        self.item
            .transform
            .as_ref()
            .map(Transform::to_column_major_matrix)
    }

    pub fn part_number(&self) -> Option<&str> {
        self.item.partnumber.as_deref()
    }

    pub fn path(&self) -> Option<&str> {
        self.item.path.as_ref().map(|p| p.as_str())
    }

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
    pub fn slicestack_id(&self) -> u32 {
        self.slicestack_id
    }

    pub fn slicepath(&self) -> &str {
        self.slicepath
    }
}

/// Stable view over a polygon.
pub struct PolygonView<'a> {
    polygon: &'a Polygon,
    segment_count: usize,
}

impl<'a> PolygonView<'a> {
    pub fn segment_count(&self) -> usize {
        self.segment_count
    }

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

    pub fn segments_data(self) -> impl Iterator<Item = [Option<u32>; 3]> {
        self.polygon
            .segment
            .iter()
            .map(|s| [s.p1.into(), s.p2.into(), s.pid.into()])
    }
}

/// Stable view over a slice.
pub struct SliceView<'a> {
    slice: &'a Slice,
}

impl<'a> SliceView<'a> {
    fn new(slice: &'a Slice) -> Self {
        Self { slice }
    }

    pub fn ztop(&self) -> f64 {
        self.slice.ztop.value()
    }

    pub fn vertex_count(&self) -> Option<usize> {
        self.slice.vertices.as_ref().map(|v| v.vertex.len())
    }

    pub fn vertices(&self) -> Option<impl Iterator<Item = [f64; 2]>> {
        self.slice
            .vertices
            .as_ref()
            .map(|v| v.vertex.iter().map(|v| [v.x.into(), v.y.into()]))
    }

    pub fn polygon_count(&self) -> usize {
        self.slice.polygon.len()
    }

    pub fn polygons(&self) -> impl Iterator<Item = PolygonView<'a>> + '_ {
        self.slice.polygon.iter().map(|p| PolygonView {
            polygon: p,
            segment_count: p.segment.len(),
        })
    }
}

/// Stable view over a slice stack.
pub struct SliceStackView<'a> {
    stack: &'a SliceStack,
}

impl<'a> SliceStackView<'a> {
    pub(crate) fn new(stack: &'a SliceStack) -> Self {
        Self { stack }
    }

    pub fn id(&self) -> u32 {
        self.stack.id
    }

    pub fn zbottom(&self) -> Option<f64> {
        self.stack.zbottom.map(|d| d.value())
    }

    pub fn has_owned_slices(&self) -> bool {
        self.stack.has_owned_slices()
    }

    pub fn slice_count(&self) -> usize {
        self.stack.slice.len()
    }

    pub fn sliceref_count(&self) -> usize {
        self.stack.sliceref.len()
    }

    pub fn slicerefs(&self) -> impl Iterator<Item = SliceRefView<'a>> + '_ {
        self.stack.sliceref.iter().map(|r| SliceRefView {
            slicestack_id: r.slicestackid,
            slicepath: r.slicepath.as_str(),
        })
    }

    pub fn slices(&self) -> impl Iterator<Item = SliceView<'a>> + '_ {
        self.stack.slice.iter().map(SliceView::new)
    }
}

/// Stable view over a color group.
pub struct ColorGroupView<'a> {
    group: &'a ColorGroup,
}

impl<'a> ColorGroupView<'a> {
    pub(crate) fn new(group: &'a ColorGroup) -> Self {
        Self { group }
    }

    pub fn id(&self) -> u32 {
        self.group.id
    }

    pub fn color_count(&self) -> usize {
        self.group.color.len()
    }

    pub fn color_at(&self, index: usize) -> Option<Color> {
        self.group.color.get(index).map(|c| c.color)
    }
}

/// Stable view over a texture 2d group.
pub struct Texture2DGroupView<'a> {
    group: &'a Texture2DGroup,
}

impl<'a> Texture2DGroupView<'a> {
    pub(crate) fn new(group: &'a Texture2DGroup) -> Self {
        Self { group }
    }

    pub fn id(&self) -> u32 {
        self.group.id
    }

    pub fn texid(&self) -> u32 {
        self.group.texid
    }

    pub fn texcoord_count(&self) -> usize {
        self.group.tex2coord.len()
    }

    pub fn tex_coords(&self) -> impl Iterator<Item = [f64; 2]> {
        self.group
            .tex2coord
            .iter()
            .map(|t| [t.u.into(), t.v.into()])
    }
}

/// Stable view over composite materials.
pub struct CompositeMaterialsView<'a> {
    materials: &'a CompositeMaterials,
}

impl<'a> CompositeMaterialsView<'a> {
    pub(crate) fn new(materials: &'a CompositeMaterials) -> Self {
        Self { materials }
    }

    pub fn id(&self) -> u32 {
        self.materials.id
    }

    pub fn composite_count(&self) -> usize {
        self.materials.composite.len()
    }
}

/// Stable view over multi-properties.
pub struct MultiPropertiesView<'a> {
    props: &'a MultiProperties,
}

impl<'a> MultiPropertiesView<'a> {
    pub(crate) fn new(props: &'a MultiProperties) -> Self {
        Self { props }
    }

    pub fn id(&self) -> u32 {
        self.props.id
    }

    pub fn multi_count(&self) -> usize {
        self.props.multi.len()
    }
}

/// Stable view over a texture2d resource.
pub struct Texture2DView<'a> {
    texture: &'a Texture2D,
}

impl<'a> Texture2DView<'a> {
    pub(crate) fn new(texture: &'a Texture2D) -> Self {
        Self { texture }
    }

    pub fn id(&self) -> u32 {
        self.texture.id
    }

    pub fn path(&self) -> &str {
        self.texture.path.as_str()
    }

    pub fn content_type(&self) -> material::TextureContentType {
        self.texture.contenttype.clone()
    }

    pub fn tile_style_u(&self) -> material::TileStyle {
        self.texture.tilestyleu.unwrap_or(material::TileStyle::Wrap)
    }

    pub fn tile_style_v(&self) -> material::TileStyle {
        self.texture.tilestylev.unwrap_or(material::TileStyle::Wrap)
    }

    pub fn filter(&self) -> material::Filter {
        self.texture.filter.unwrap_or(material::Filter::Auto)
    }
}

/// Stable view over a displacement2d resource.
pub struct Displacement2DView<'a> {
    displacement: &'a Displacement2D,
}

impl<'a> Displacement2DView<'a> {
    pub(crate) fn new(displacement: &'a Displacement2D) -> Self {
        Self { displacement }
    }

    pub fn id(&self) -> u32 {
        self.displacement.id
    }
}

/// Stable view over a norm vector group.
pub struct NormVectorGroupView<'a> {
    group: &'a NormVectorGroup,
}

impl<'a> NormVectorGroupView<'a> {
    pub(crate) fn new(group: &'a NormVectorGroup) -> Self {
        Self { group }
    }

    pub fn id(&self) -> u32 {
        self.group.id
    }

    pub fn norm_vectors(&self) -> impl Iterator<Item = [f64; 3]> {
        self.group
            .normvector
            .iter()
            .map(|n| [n.x.into(), n.y.into(), n.z.into()])
    }
}

/// Stable view over a displacement 2d group.
pub struct Disp2DGroupView<'a> {
    group: &'a Disp2DGroup,
}

pub struct Disp2DCoordView {
    pub u: f64,
    pub v: f64,
    pub norm_index: u32,
    pub f: f64,
}

impl<'a> Disp2DGroupView<'a> {
    pub(crate) fn new(group: &'a Disp2DGroup) -> Self {
        Self { group }
    }

    pub fn id(&self) -> u32 {
        self.group.id
    }

    pub fn displacement_map_id(&self) -> u32 {
        self.group.dispid
    }

    pub fn norm_vector_group_id(&self) -> u32 {
        self.group.nid
    }

    pub fn height(&self) -> f64 {
        self.group.height.into()
    }

    pub fn offset(&self) -> f64 {
        self.group.offset.map_or(0.0, |o| o.into())
    }

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

    pub fn id(&self) -> u32 {
        self.materials.id
    }

    pub fn base_count(&self) -> usize {
        self.materials.base.len()
    }
}

/// Stable material property value.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialPropertyValue {
    Color(Color),
    TextureCoord { u: f64, v: f64 },
    Composite { values: Vec<f64> },
    Multi { indices: Vec<u32> },
    Base { name: String, displaycolor: String },
}

pub fn get_model_view<'a>(model: &'a Model) -> ModelView<'a> {
    ModelView::new(model)
}

pub fn get_object_from_model<'a>(object_id: u32, model: &'a Model) -> Option<ObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .find(|o| o.id == object_id)
        .map(ObjectView::new)
}

pub fn get_objects_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ObjectView<'a>> {
    model.resources.object.iter().map(ObjectView::new)
}

pub fn get_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MeshObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(MeshObjectView::from_object)
}

pub fn get_displacement_mesh_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = DisplacementMeshObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(DisplacementMeshObjectView::from_object)
}

pub fn get_components_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ComponentsObjectView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(ComponentsObjectView::from_object)
}

pub fn get_boolean_shape_objects_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = BooleanShapeView<'a>> {
    model
        .resources
        .object
        .iter()
        .filter_map(BooleanShapeView::from_object)
}

pub fn get_items_from_model<'a>(model: &'a Model) -> impl Iterator<Item = ItemView<'a>> {
    model.build.item.iter().map(ItemView::new)
}

pub fn get_slice_stacks_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = SliceStackView<'a>> {
    model.resources.slicestack.iter().map(SliceStackView::new)
}

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

pub fn get_color_groups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = ColorGroupView<'a>> {
    model.resources.colorgroup.iter().map(ColorGroupView::new)
}

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

pub fn get_texture2d_groups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = Texture2DGroupView<'a>> {
    model
        .resources
        .texture2dgroup
        .iter()
        .map(Texture2DGroupView::new)
}

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

pub fn get_composite_materials_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = CompositeMaterialsView<'a>> {
    model
        .resources
        .compositematerials
        .iter()
        .map(CompositeMaterialsView::new)
}

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

pub fn get_multi_properties_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = MultiPropertiesView<'a>> {
    model
        .resources
        .multiproperties
        .iter()
        .map(MultiPropertiesView::new)
}

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

pub fn get_texture2ds_from_model<'a>(model: &'a Model) -> impl Iterator<Item = Texture2DView<'a>> {
    model.resources.texture2d.iter().map(Texture2DView::new)
}

pub fn get_texture2d_by_id<'a>(texture2d_id: u32, model: &'a Model) -> Option<Texture2DView<'a>> {
    model
        .resources
        .texture2d
        .iter()
        .find(|t| t.id == texture2d_id)
        .map(Texture2DView::new)
}

pub fn get_displacement2ds_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = Displacement2DView<'a>> {
    model
        .resources
        .displacement2d
        .iter()
        .map(Displacement2DView::new)
}

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

pub fn get_normvectorgroups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = NormVectorGroupView<'a>> {
    model
        .resources
        .normvectorgroup
        .iter()
        .map(NormVectorGroupView::new)
}

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

pub fn get_disp2dgroups_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = Disp2DGroupView<'a>> {
    model.resources.disp2dgroup.iter().map(Disp2DGroupView::new)
}

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

pub fn get_base_materials_from_model<'a>(
    model: &'a Model,
) -> impl Iterator<Item = BaseMaterialsView<'a>> {
    model
        .resources
        .basematerials
        .iter()
        .map(BaseMaterialsView::new)
}

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

pub fn get_texture_for_group<'a>(
    texture2dgroup: &Texture2DGroup,
    model: &'a Model,
) -> Option<Texture2DView<'a>> {
    get_texture2d_by_id(texture2dgroup.texid, model)
}

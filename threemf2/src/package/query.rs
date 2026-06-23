//! Query API for inspecting 3MF packages.

#![allow(clippy::needless_lifetimes)]

use crate::model::{
    domain::model::Model,
    query::{
        BooleanShapeView, ComponentsObjectView, DisplacementMeshObjectView, ItemView,
        MeshObjectView, ModelView, ObjectView, SliceStackView,
    },
};
use crate::package::ThreemfPackage;

/// Package view for an object with origin model path.
pub struct ObjectPackageView<'a> {
    /// The object view.
    pub view: ObjectView<'a>,
    /// Path to the model containing this object, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a mesh object.
pub struct MeshObjectPackageView<'a> {
    /// The mesh object view.
    pub view: MeshObjectView<'a>,
    /// Path to the model containing this mesh, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a displacement mesh object.
pub struct DisplacementMeshObjectPackageView<'a> {
    /// The displacement mesh object view.
    pub view: DisplacementMeshObjectView<'a>,
    /// Path to the model containing this object, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a components object.
pub struct ComponentsObjectPackageView<'a> {
    /// The components object view.
    pub view: ComponentsObjectView<'a>,
    /// Path to the model containing this object, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a boolean shape object.
pub struct BooleanShapePackageView<'a> {
    /// The boolean shape view.
    pub view: BooleanShapeView<'a>,
    /// Path to the model containing this object, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a build item.
pub struct ItemPackageView<'a> {
    /// The build item view.
    pub view: ItemView<'a>,
    /// Path to the model containing this item, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a slice stack.
pub struct SliceStackPackageView<'a> {
    /// The slice stack view.
    pub view: SliceStackView<'a>,
    /// Path to the model containing this slice stack, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a model.
pub struct ModelPackageView<'a> {
    /// The model view.
    pub view: ModelView<'a>,
    /// Path to the model file, if a sub-model.
    pub origin_model_path: Option<&'a str>,
}

/// A reference to a model within a package, with path information for sub-models.
pub struct ModelRef<'a> {
    /// The referenced model.
    pub model: &'a Model,
    /// Path to the model file, if a sub-model.
    pub path: Option<&'a str>,
}

/// Returns an iterator over all models in the package.
pub fn iter_models<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ModelRef<'a>> {
    std::iter::once(ModelRef {
        model: &package.root,
        path: None,
    })
    .chain(package.sub_models.iter().map(|(path, model)| ModelRef {
        model,
        path: Some(path.as_str()),
    }))
}

/// Returns an iterator over package views for all models.
pub fn get_models<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ModelPackageView<'a>> {
    iter_models(package).map(|model_ref| ModelPackageView {
        view: ModelView::new(model_ref.model),
        origin_model_path: model_ref.path,
    })
}

/// Returns an iterator over package views for all objects.
pub fn get_objects<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ObjectPackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .resources
            .object
            .iter()
            .map(move |o| ObjectPackageView {
                view: ObjectView::new(o),
                origin_model_path: model_ref.path,
            })
    })
}

/// Returns an iterator over package views for all mesh objects.
pub fn get_mesh_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = MeshObjectPackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .resources
            .object
            .iter()
            .filter_map(move |o| {
                MeshObjectView::from_object(o).map(|view| MeshObjectPackageView {
                    view,
                    origin_model_path: model_ref.path,
                })
            })
    })
}

/// Returns an iterator over package views for all displacement mesh objects.
pub fn get_displacement_mesh_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = DisplacementMeshObjectPackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .resources
            .object
            .iter()
            .filter_map(move |o| {
                DisplacementMeshObjectView::from_object(o).map(|view| {
                    DisplacementMeshObjectPackageView {
                        view,
                        origin_model_path: model_ref.path,
                    }
                })
            })
    })
}

/// Returns an iterator over package views for all components objects.
pub fn get_components_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = ComponentsObjectPackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .resources
            .object
            .iter()
            .filter_map(move |o| {
                ComponentsObjectView::from_object(o).map(|view| ComponentsObjectPackageView {
                    view,
                    origin_model_path: model_ref.path,
                })
            })
    })
}

/// Returns an iterator over package views for all boolean shape objects.
pub fn get_boolean_shape_objects<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = BooleanShapePackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .resources
            .object
            .iter()
            .filter_map(move |o| {
                BooleanShapeView::from_object(o).map(|view| BooleanShapePackageView {
                    view,
                    origin_model_path: model_ref.path,
                })
            })
    })
}

/// Returns an iterator over package views for all build items.
pub fn get_items<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ItemPackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .build
            .item
            .iter()
            .map(move |item| ItemPackageView {
                view: ItemView::new(item),
                origin_model_path: model_ref.path,
            })
    })
}

/// Returns an iterator over build items referencing the given object id.
pub fn get_items_by_objectid<'a>(
    package: &'a ThreemfPackage,
    objectid: u32,
) -> impl Iterator<Item = ItemPackageView<'a>> {
    get_items(package).filter(move |item_ref| item_ref.view.object_id() == objectid)
}

/// Returns the first build item matching the given UUID.
pub fn get_item_by_uuid<'a>(
    package: &'a ThreemfPackage,
    uuid: &str,
) -> Option<ItemPackageView<'a>> {
    get_items(package).find(|item_ref| item_ref.view.uuid().as_deref() == Some(uuid))
}

/// Returns an iterator over package views for all slice stacks.
pub fn get_slice_stacks<'a>(
    package: &'a ThreemfPackage,
) -> impl Iterator<Item = SliceStackPackageView<'a>> {
    iter_models(package).flat_map(|model_ref| {
        model_ref
            .model
            .resources
            .slicestack
            .iter()
            .map(move |s| SliceStackPackageView {
                view: SliceStackView::new(s),
                origin_model_path: model_ref.path,
            })
    })
}

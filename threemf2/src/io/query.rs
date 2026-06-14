//! Query API for inspecting 3MF packages.

#![allow(clippy::needless_lifetimes)]

use crate::core::{
    model::Model,
    query::{
        BooleanShapeView, ComponentsObjectView, DisplacementMeshObjectView, ItemView,
        MeshObjectView, ModelView, ObjectView, SliceStackView,
    },
};
use crate::io::ThreemfPackage;

/// Package view for an object with origin model path.
pub struct ObjectPackageView<'a> {
    pub view: ObjectView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a mesh object.
pub struct MeshObjectPackageView<'a> {
    pub view: MeshObjectView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a displacement mesh object.
pub struct DisplacementMeshObjectPackageView<'a> {
    pub view: DisplacementMeshObjectView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a components object.
pub struct ComponentsObjectPackageView<'a> {
    pub view: ComponentsObjectView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a boolean shape object.
pub struct BooleanShapePackageView<'a> {
    pub view: BooleanShapeView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a build item.
pub struct ItemPackageView<'a> {
    pub view: ItemView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a slice stack.
pub struct SliceStackPackageView<'a> {
    pub view: SliceStackView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// Package view for a model.
pub struct ModelPackageView<'a> {
    pub view: ModelView<'a>,
    pub origin_model_path: Option<&'a str>,
}

/// A reference to a model within a package, with path information for sub-models.
pub struct ModelRef<'a> {
    pub model: &'a Model,
    pub path: Option<&'a str>,
}

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

pub fn get_models<'a>(package: &'a ThreemfPackage) -> impl Iterator<Item = ModelPackageView<'a>> {
    iter_models(package).map(|model_ref| ModelPackageView {
        view: ModelView::new(model_ref.model),
        origin_model_path: model_ref.path,
    })
}

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

pub fn get_items_by_objectid<'a>(
    package: &'a ThreemfPackage,
    objectid: u32,
) -> impl Iterator<Item = ItemPackageView<'a>> {
    get_items(package).filter(move |item_ref| item_ref.view.object_id() == objectid)
}

pub fn get_item_by_uuid<'a>(
    package: &'a ThreemfPackage,
    uuid: &str,
) -> Option<ItemPackageView<'a>> {
    get_items(package).find(|item_ref| item_ref.view.uuid().as_deref() == Some(uuid))
}

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

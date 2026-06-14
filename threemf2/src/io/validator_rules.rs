//! Validation rule implementations for 3MF models.
//!
//! This module contains the concrete implementations for each validation rule.

use crate::core::model::Model;
use crate::core::query as core_query;
use crate::io::ThreemfPackage;
use crate::io::query as io_query;
use crate::io::validator::{Severity, ValidationIssue, ValidationRule};
use std::collections::HashSet;

/// Runs a single validation rule against a model.
///
/// This function dispatches to the appropriate rule implementation based on the rule type.
pub fn run_rule_for_model(rule: &ValidationRule, model: &Model) -> Vec<ValidationIssue> {
    match rule {
        ValidationRule::ObjectIdReference => validate_object_id_reference(model, None),
        ValidationRule::BaseMaterialReference => validate_resource_id_reference(model, None),
        ValidationRule::BuildItemReference => validate_build_item_references(model, None),
        ValidationRule::ComponentReference => validate_component_references(model, None),
        ValidationRule::ObjectToSliceStackReference => {
            validate_object_to_slicestack_references(model, None)
        }
    }
}

/// Runs a single validation rule against the whole Package.
///
/// This function dispatches to the appropriate rule implementation based on the rule type.
pub fn run_rule_for_package(
    rule: &ValidationRule,
    package: &ThreemfPackage,
) -> Vec<ValidationIssue> {
    match rule {
        ValidationRule::ObjectIdReference => {
            validate_object_id_reference(&package.root, Some(package))
        }
        ValidationRule::BaseMaterialReference => {
            validate_resource_id_reference(&package.root, Some(package))
        }
        ValidationRule::BuildItemReference => {
            validate_build_item_references(&package.root, Some(package))
        }
        ValidationRule::ComponentReference => {
            validate_component_references(&package.root, Some(package))
        }
        ValidationRule::ObjectToSliceStackReference => {
            validate_object_to_slicestack_references(&package.root, Some(package))
        }
    }
}

fn validate_component_references(
    model: &Model,
    package: Option<&ThreemfPackage>,
) -> Vec<ValidationIssue> {
    let comp_objs = get_component_objects(model, package);
    let objs = get_objects(model, package);

    let mut issues = vec![];
    for obj in comp_objs {
        for comp in &obj.components {
            if !objs
                .iter()
                .filter(|o| o.origin_model_path == comp.path.as_deref())
                .any(|o| o.id == comp.object_id)
            {
                issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    message: format!(
                        "A Component in Components Object with Id: {} at path: {} is referencing an unknown Object with Id: {} at path: {:?}",
                        obj.id,
                        obj.origin_model_path.clone().unwrap_or("root".to_owned()),
                        comp.object_id,
                        comp.path.clone().unwrap_or("root".to_owned())
                    ),
                });
            }
        }
    }

    issues
}

fn validate_build_item_references(
    model: &Model,
    package: Option<&ThreemfPackage>,
) -> Vec<ValidationIssue> {
    let items = get_build_items(model, package);
    let objects = get_objects(model, package);

    let mut issues = vec![];

    if package.is_some() && items.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            message: "Package does not contain any Build Items".to_owned(),
        });
    }

    for item in items {
        if !objects
            .iter()
            .filter(|o| o.origin_model_path == item.path.as_deref())
            .any(|o| o.id == item.object_id)
        {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: format!(
                    "A Build Item is referencing an unknown Object with Id: {} at path: {:?}",
                    item.object_id,
                    item.path.unwrap_or("root".to_owned())
                ),
            });
        }
    }

    issues
}

/// Validates object ID constraints:
/// - Object IDs must be unique within the model
/// - Object IDs must start at value 1
/// - Object IDs must be within valid range (1..=2_147_483_647)
fn validate_object_id_reference(
    model: &Model,
    package: Option<&ThreemfPackage>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut seen_ids = HashSet::new();
    const MAX_RESOURCE_ID: u32 = 2_147_483_647;

    let obj_refs = get_objects(model, package);

    for obj_ref in obj_refs {
        let id = obj_ref.id;

        // Check ID starts at 1
        if id == 0 {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!(
                    "Object ID cannot be 0. Object IDs must start at 1. Found at path: {}",
                    obj_ref.origin_model_path.unwrap_or("root")
                ),
            ));
            continue;
        }

        // Check ID within valid range
        if id > MAX_RESOURCE_ID {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!(
                    "Object ID {} exceeds maximum allowed value of {}.",
                    id, MAX_RESOURCE_ID
                ),
            ));
            continue;
        }

        // Check ID uniqueness
        if !seen_ids.insert(id) {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!("Duplicate object ID {} found in model resources.", id),
            ));
        }
    }

    issues
}

fn validate_object_to_slicestack_references(
    model: &Model,
    package: Option<&ThreemfPackage>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    let obj_refs = get_objects(model, package);
    let slicestack_refs = get_slicestacks(model, package);

    for obj_ref in obj_refs {
        if slicestack_refs.iter().any(|stack_ref| {
            stack_ref.path == obj_ref.slicepath.as_deref()
                && stack_ref.id == obj_ref.slicestack_id.unwrap_or(u32::MAX)
        }) {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!(
                    "Unable to find slicestack with id: {:?} in model path: {:?} referenced by Object with object id: {:?} in model path: {:?}",
                    obj_ref.slicestack_id,
                    obj_ref.slicepath,
                    obj_ref.id,
                    obj_ref.origin_model_path
                ),
            ));
        }
    }

    issues
}

struct ObjectInfo<'a> {
    id: u32,
    origin_model_path: Option<&'a str>,
    slicepath: Option<String>,
    slicestack_id: Option<u32>,
    pid: crate::core::OptionalResourceId,
    pindex: crate::core::OptionalResourceIndex,
}

struct BuildItemInfo {
    object_id: u32,
    path: Option<String>,
}

struct ComponentInfo {
    object_id: u32,
    path: Option<String>,
}

struct ComponentsObjectInfo {
    id: u32,
    origin_model_path: Option<String>,
    components: Vec<ComponentInfo>,
}

struct SliceStackInfo<'a> {
    id: u32,
    path: Option<&'a str>,
}

fn get_objects<'a>(model: &'a Model, package: Option<&'a ThreemfPackage>) -> Vec<ObjectInfo<'a>> {
    if let Some(package) = package {
        io_query::get_objects(package)
            .map(|o| ObjectInfo {
                id: o.view.id(),
                origin_model_path: o.origin_model_path,
                slicepath: o.view.slicepath().map(|p| p.to_owned()),
                slicestack_id: o.view.slicestack_id(),
                pid: o.view.pid(),
                pindex: o.view.pindex(),
            })
            .collect()
    } else {
        core_query::get_objects_from_model(model)
            .map(|o| ObjectInfo {
                id: o.id(),
                origin_model_path: None,
                slicepath: o.slicepath().map(|p| p.to_owned()),
                slicestack_id: o.slicestack_id(),
                pid: o.pid(),
                pindex: o.pindex(),
            })
            .collect()
    }
}

fn get_build_items<'a>(
    model: &'a Model,
    package: Option<&'a ThreemfPackage>,
) -> Vec<BuildItemInfo> {
    if let Some(package) = package {
        io_query::get_items(package)
            .map(|i| BuildItemInfo {
                object_id: i.view.object_id(),
                path: i.view.path().map(|p| p.to_owned()),
            })
            .collect()
    } else {
        core_query::get_items_from_model(model)
            .map(|i| BuildItemInfo {
                object_id: i.object_id(),
                path: i.path().map(|p| p.to_owned()),
            })
            .collect()
    }
}

fn get_component_objects<'a>(
    model: &'a Model,
    package: Option<&'a ThreemfPackage>,
) -> Vec<ComponentsObjectInfo> {
    if let Some(package) = package {
        io_query::get_components_objects(package)
            .map(|c| ComponentsObjectInfo {
                id: c.view.id(),
                origin_model_path: c.origin_model_path.map(|p| p.to_owned()),
                components: c
                    .view
                    .components()
                    .map(|comp| ComponentInfo {
                        object_id: comp.object_id(),
                        path: comp.path().map(|p| p.to_owned()),
                    })
                    .collect(),
            })
            .collect()
    } else {
        core_query::get_components_objects_from_model(model)
            .map(|c| ComponentsObjectInfo {
                id: c.id(),
                origin_model_path: None,
                components: c
                    .components()
                    .map(|comp| ComponentInfo {
                        object_id: comp.object_id(),
                        path: comp.path().map(|p| p.to_owned()),
                    })
                    .collect(),
            })
            .collect()
    }
}

fn get_slicestacks<'a>(
    model: &'a Model,
    package: Option<&'a ThreemfPackage>,
) -> Vec<SliceStackInfo<'a>> {
    if let Some(package) = package {
        io_query::get_slice_stacks(package)
            .map(|s| SliceStackInfo {
                id: s.view.id(),
                path: s.origin_model_path,
            })
            .collect()
    } else {
        core_query::get_slice_stacks_from_model(model)
            .map(|s| SliceStackInfo {
                id: s.id(),
                path: None,
            })
            .collect()
    }
}

/// Validates resource ID references:
/// - pid references must point to existing BaseMaterials
fn validate_resource_id_reference(
    model: &Model,
    package: Option<&ThreemfPackage>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Collect valid BaseMaterials IDs
    let valid_basematerials_ids: HashSet<u32> = model
        .resources
        .basematerials
        .iter()
        .map(|bm| bm.id)
        .collect();

    let obj_refs = get_objects(model, package);
    // Check all objects' pid references
    for obj_ref in obj_refs {
        if let Some(pid) = obj_ref.pid.get() {
            // Check pid points to existing BaseMaterials
            if !valid_basematerials_ids.contains(&pid) {
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    format!(
                        "Object {} references pid={} but no BaseMaterials with that ID exists.",
                        obj_ref.id, pid
                    ),
                ));
            }
        }
    }

    let obj_refs = get_objects(model, package);
    // Check pindex consistency: if pindex is specified, pid must be specified too
    for obj_ref in obj_refs {
        if obj_ref.pindex.is_some() && obj_ref.pid.is_none() {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!(
                    "Object {} has pindex but no pid. pindex requires pid to be specified.",
                    obj_ref.id
                ),
            ));
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        build::Build,
        model::ThreemfExtensions,
        object::{Object, ObjectKind},
        resources::{BaseMaterials, Resources},
        types::OptionalResourceId,
    };

    fn create_test_model(resources: Resources, build: Build) -> Model {
        Model {
            unit: None,
            requiredextensions: ThreemfExtensions::default(),
            recommendedextensions: ThreemfExtensions::default(),
            metadata: Vec::new(),
            resources,
            build,
        }
    }

    fn create_test_object(id: u32) -> Object {
        Object {
            id,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: crate::core::types::OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
        }
    }

    fn create_test_object_with_pid(id: u32, pid: u32) -> Object {
        Object {
            id,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::new(pid),
            pindex: crate::core::types::OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
        }
    }

    fn create_empty_build() -> Build {
        Build {
            uuid: None,
            item: Vec::new(),
        }
    }

    #[test]
    fn test_object_id_unique_valid() {
        let resources = Resources {
            object: vec![
                create_test_object(1),
                create_test_object(2),
                create_test_object(3),
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model, None);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_object_id_duplicate() {
        let resources = Resources {
            object: vec![create_test_object(1), create_test_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("Duplicate object ID 1"));
    }

    #[test]
    fn test_object_id_zero() {
        let resources = Resources {
            object: vec![create_test_object(0)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("Object ID cannot be 0"));
    }

    #[test]
    fn test_object_id_out_of_range() {
        let resources = Resources {
            object: vec![create_test_object(2_147_483_648)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("exceeds maximum allowed value"));
    }

    #[test]
    fn test_object_id_multiple_issues() {
        let resources = Resources {
            object: vec![
                create_test_object(0),             // Error: starts at 0
                create_test_object(1),             // Valid
                create_test_object(1),             // Error: duplicate
                create_test_object(2_147_483_648), // Error: out of range
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model, None);
        assert_eq!(issues.len(), 3);
        assert!(issues.iter().all(|i| i.severity == Severity::Error));
    }

    #[test]
    fn test_resource_id_valid_pid() {
        let basematerials = vec![BaseMaterials {
            id: 10,
            base: vec![],
        }];
        let resources = Resources {
            object: vec![create_test_object_with_pid(1, 10)],
            basematerials,
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model, None);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_resource_id_missing_pid_reference() {
        let resources = Resources {
            object: vec![create_test_object_with_pid(1, 10)],
            basematerials: Vec::new(),
            slicestack: Vec::new(), // No BaseMaterials with id=10
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(
            issues[0]
                .message
                .contains("no BaseMaterials with that ID exists")
        );
    }

    #[test]
    fn test_resource_id_no_pid() {
        // Object without pid should be valid
        let resources = Resources {
            object: vec![create_test_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model, None);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_pindex_without_pid() {
        let object = Object {
            id: 1,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: crate::core::types::OptionalResourceIndex::new(0),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: None,
        };
        let resources = Resources {
            object: vec![object],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model, None);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("pindex but no pid"));
    }

    #[test]
    fn test_multiple_objects_with_pid_issues() {
        let basematerials = vec![BaseMaterials {
            id: 5,
            base: vec![],
        }];
        let resources = Resources {
            object: vec![
                create_test_object_with_pid(1, 5),  // Valid
                create_test_object_with_pid(2, 10), // Error: missing reference
                create_test_object(3),              // Valid: no pid
                create_test_object_with_pid(4, 5),  // Valid
            ],
            basematerials,
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model, None);
        assert_eq!(issues.len(), 1); // Only the missing reference error
        assert!(
            issues[0]
                .message
                .contains("no BaseMaterials with that ID exists")
        );
    }

    #[test]
    fn test_run_rule_dispatcher() {
        let resources = Resources {
            object: vec![create_test_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let object_issues = run_rule_for_model(&ValidationRule::ObjectIdReference, &model);
        assert!(object_issues.is_empty());

        let resource_issues = run_rule_for_model(&ValidationRule::BaseMaterialReference, &model);
        assert!(resource_issues.is_empty());
    }

    // Helper functions for Build and Component tests
    fn create_test_build_with_items(items: Vec<crate::core::build::Item>) -> Build {
        Build {
            uuid: None,
            item: items,
        }
    }

    fn create_test_build_item(objectid: u32) -> crate::core::build::Item {
        crate::core::build::Item {
            objectid,
            transform: None,
            partnumber: None,
            path: None,
            uuid: None,
        }
    }

    fn create_test_component(objectid: u32) -> crate::core::component::Component {
        crate::core::component::Component {
            objectid,
            transform: None,
            path: None,
            uuid: None,
        }
    }

    fn create_test_object_with_components(
        id: u32,
        components: Vec<crate::core::component::Component>,
    ) -> Object {
        Object {
            id,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: crate::core::types::OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: Some(ObjectKind::Components(crate::core::component::Components {
                component: components,
            })),
        }
    }

    fn create_test_mesh_object(id: u32) -> Object {
        Object {
            id,
            objecttype: None,
            thumbnail: None,
            partnumber: None,
            name: None,
            pid: OptionalResourceId::none(),
            pindex: crate::core::types::OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::none(),
            slicepath: None,
            meshresolution: None,
            kind: Some(ObjectKind::Mesh(crate::core::mesh::Mesh {
                vertices: crate::core::mesh::Vertices { vertex: vec![] },
                triangles: crate::core::mesh::Triangles { triangle: vec![] },
                trianglesets: None,
                beamlattice: None,
            })),
        }
    }

    // BuildItemReference Tests
    #[test]
    fn test_build_item_reference_valid() {
        // Create a model with one object and a build item referencing it
        let resources = Resources {
            object: vec![create_test_mesh_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let build = create_test_build_with_items(vec![create_test_build_item(1)]);
        let model = create_test_model(resources, build);

        let issues = validate_build_item_references(&model, None);
        assert!(
            issues.is_empty(),
            "Expected no issues for valid build item reference"
        );
    }

    #[test]
    fn test_build_item_reference_missing_object() {
        // Create a model with build item referencing non-existent object
        let resources = Resources {
            object: vec![create_test_mesh_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let build = create_test_build_with_items(vec![create_test_build_item(2)]); // References object 2 which doesn't exist
        let model = create_test_model(resources, build);

        let issues = validate_build_item_references(&model, None);
        assert_eq!(
            issues.len(),
            1,
            "Expected one warning for missing object reference"
        );
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("unknown Object with Id: 2"));
    }

    #[test]
    fn test_build_item_reference_multiple_items() {
        // Create model with multiple objects and build items - mix of valid and invalid
        let resources = Resources {
            object: vec![create_test_mesh_object(1), create_test_mesh_object(2)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let build = create_test_build_with_items(vec![
            create_test_build_item(1), // Valid
            create_test_build_item(2), // Valid
            create_test_build_item(3), // Invalid - object 3 doesn't exist
            create_test_build_item(4), // Invalid - object 4 doesn't exist
        ]);
        let model = create_test_model(resources, build);

        let issues = validate_build_item_references(&model, None);
        assert_eq!(
            issues.len(),
            2,
            "Expected two warnings for missing object references"
        );
        assert!(issues.iter().all(|i| i.severity == Severity::Warning));
        assert!(issues[0].message.contains("Id: 3") || issues[0].message.contains("Id: 4"));
    }

    #[test]
    fn test_build_item_reference_empty_build() {
        // Empty build should pass validation
        let resources = Resources {
            object: vec![create_test_mesh_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let build = create_test_build_with_items(vec![]);
        let model = create_test_model(resources, build);

        let issues = validate_build_item_references(&model, None);
        assert!(
            issues.is_empty(),
            "Empty build should have no validation issues"
        );
    }

    // ComponentReference Tests
    #[test]
    fn test_component_reference_valid() {
        // Create a model with a components object referencing existing objects
        let resources = Resources {
            object: vec![
                create_test_mesh_object(1),
                create_test_object_with_components(2, vec![create_test_component(1)]),
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_component_references(&model, None);
        assert!(
            issues.is_empty(),
            "Expected no issues for valid component reference"
        );
    }

    #[test]
    fn test_component_reference_missing_object() {
        // Create a model with a component referencing non-existent object
        let resources = Resources {
            object: vec![
                create_test_mesh_object(1),
                create_test_object_with_components(2, vec![create_test_component(999)]), // References object 999 which doesn't exist
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_component_references(&model, None);
        assert_eq!(
            issues.len(),
            1,
            "Expected one warning for missing object reference"
        );
        assert_eq!(issues[0].severity, Severity::Warning);
        assert!(issues[0].message.contains("unknown Object with Id: 999"));
    }

    #[test]
    fn test_component_reference_multiple_components() {
        // Create model with multiple components - mix of valid and invalid
        let resources = Resources {
            object: vec![
                create_test_mesh_object(1),
                create_test_mesh_object(2),
                create_test_object_with_components(
                    3,
                    vec![
                        create_test_component(1),  // Valid
                        create_test_component(2),  // Valid
                        create_test_component(10), // Invalid
                        create_test_component(20), // Invalid
                    ],
                ),
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_component_references(&model, None);
        assert_eq!(
            issues.len(),
            2,
            "Expected two warnings for missing object references"
        );
        assert!(issues.iter().all(|i| i.severity == Severity::Warning));
    }

    #[test]
    fn test_component_reference_no_components() {
        // Model with only mesh objects (no components) should pass
        let resources = Resources {
            object: vec![create_test_mesh_object(1), create_test_mesh_object(2)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_component_references(&model, None);
        assert!(
            issues.is_empty(),
            "Model without components should have no issues"
        );
    }

    #[test]
    fn test_component_reference_self_referential() {
        // Component that references its own object ID (circular reference)
        let resources = Resources {
            object: vec![
                create_test_object_with_components(1, vec![create_test_component(1)]), // Component references itself
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        // This should NOT produce a warning because the object itself exists
        // (even though it's a components object)
        let issues = validate_component_references(&model, None);
        assert!(
            issues.is_empty(),
            "Self-reference should be valid since object exists"
        );
    }

    #[test]
    fn test_component_reference_cross_reference() {
        // Two components objects referencing each other
        let resources = Resources {
            object: vec![
                create_test_object_with_components(1, vec![create_test_component(2)]),
                create_test_object_with_components(2, vec![create_test_component(1)]),
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_component_references(&model, None);
        assert!(
            issues.is_empty(),
            "Cross-references between existing objects should be valid"
        );
    }

    // Integration Tests for New Rules
    #[test]
    fn test_run_rule_for_model_build_item_reference() {
        let resources = Resources {
            object: vec![create_test_mesh_object(1)],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let build = create_test_build_with_items(vec![create_test_build_item(1)]);
        let model = create_test_model(resources, build);

        let issues = run_rule_for_model(&ValidationRule::BuildItemReference, &model);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_run_rule_for_model_component_reference() {
        let resources = Resources {
            object: vec![
                create_test_mesh_object(1),
                create_test_object_with_components(2, vec![create_test_component(1)]),
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = run_rule_for_model(&ValidationRule::ComponentReference, &model);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_all_new_rules_with_issues() {
        // Comprehensive test with both build item and component issues
        let resources = Resources {
            object: vec![
                create_test_mesh_object(1),
                create_test_object_with_components(2, vec![create_test_component(999)]),
            ],
            basematerials: Vec::new(),
            slicestack: Vec::new(),
            colorgroup: Vec::new(),
            texture2dgroup: Vec::new(),
            compositematerials: Vec::new(),
            multiproperties: Vec::new(),
            texture2d: Vec::new(),
            displacement2d: Vec::new(),
            normvectorgroup: Vec::new(),
            disp2dgroup: Vec::new(),
        };
        let build = create_test_build_with_items(vec![
            create_test_build_item(1),   // Valid
            create_test_build_item(998), // Invalid
        ]);
        let model = create_test_model(resources, build);

        let build_issues = run_rule_for_model(&ValidationRule::BuildItemReference, &model);
        let component_issues = run_rule_for_model(&ValidationRule::ComponentReference, &model);

        assert_eq!(build_issues.len(), 1);
        assert_eq!(component_issues.len(), 1);
        assert!(build_issues[0].message.contains("998"));
        assert!(component_issues[0].message.contains("999"));
    }
}

//! Validation rule implementations for 3MF models.
//!
//! This module contains the concrete implementations for each validation rule.

use crate::core::model::Model;
use crate::io::validator::{Severity, ValidationIssue, ValidationRule};
use std::collections::HashSet;

/// Runs a single validation rule against a model.
///
/// This function dispatches to the appropriate rule implementation based on the rule type.
pub fn run_rule(rule: &ValidationRule, model: &Model) -> Vec<ValidationIssue> {
    match rule {
        ValidationRule::ObjectIdReference => validate_object_id_reference(model),
        ValidationRule::ResourceIdReference => validate_resource_id_reference(model),
    }
}

/// Validates object ID constraints:
/// - Object IDs must be unique within the model
/// - Object IDs must start at value 1
/// - Object IDs must be within valid range (1..=2_147_483_647)
fn validate_object_id_reference(model: &Model) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut seen_ids = HashSet::new();
    const MAX_RESOURCE_ID: u32 = 2_147_483_647;

    for object in &model.resources.object {
        let id = object.id;

        // Check ID starts at 1
        if id == 0 {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!("Object ID cannot be 0. Object IDs must start at 1."),
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

/// Validates resource ID references:
/// - pid references must point to existing BaseMaterials
fn validate_resource_id_reference(model: &Model) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Collect valid BaseMaterials IDs
    let valid_basematerials_ids: HashSet<u32> = model
        .resources
        .basematerials
        .iter()
        .map(|bm| bm.id)
        .collect();

    // Check all objects' pid references
    for object in &model.resources.object {
        if let Some(pid) = object.pid.get() {
            // Check pid points to existing BaseMaterials
            if !valid_basematerials_ids.contains(&pid) {
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    format!(
                        "Object {} references pid={} but no BaseMaterials with that ID exists.",
                        object.id, pid
                    ),
                ));
            }
        }
    }

    // Check pindex consistency: if pindex is specified, pid must be specified too
    for object in &model.resources.object {
        if object.pindex.is_some() && object.pid.is_none() {
            issues.push(ValidationIssue::new(
                Severity::Error,
                format!(
                    "Object {} has pindex but no pid. pindex requires pid to be specified.",
                    object.id
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
        object::Object,
        resources::{BaseMaterials, Resources},
        types::OptionalResourceId,
    };

    fn create_test_model(resources: Resources, build: Build) -> Model {
        Model {
            unit: None,
            requiredextensions: None,
            recommendedextensions: None,
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
            mesh: None,
            components: None,
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
            mesh: None,
            components: None,
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
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_object_id_duplicate() {
        let resources = Resources {
            object: vec![create_test_object(1), create_test_object(1)],
            basematerials: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("Duplicate object ID 1"));
    }

    #[test]
    fn test_object_id_zero() {
        let resources = Resources {
            object: vec![create_test_object(0)],
            basematerials: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0].message.contains("Object ID cannot be 0"));
    }

    #[test]
    fn test_object_id_out_of_range() {
        let resources = Resources {
            object: vec![create_test_object(2_147_483_648)],
            basematerials: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model);
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
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_object_id_reference(&model);
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
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_resource_id_missing_pid_reference() {
        let resources = Resources {
            object: vec![create_test_object_with_pid(1, 10)],
            basematerials: Vec::new(), // No BaseMaterials with id=10
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert!(issues[0]
            .message
            .contains("no BaseMaterials with that ID exists"));
    }

    #[test]
    fn test_resource_id_no_pid() {
        // Object without pid should be valid
        let resources = Resources {
            object: vec![create_test_object(1)],
            basematerials: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model);
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
            mesh: None,
            components: None,
        };
        let resources = Resources {
            object: vec![object],
            basematerials: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model);
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
        };
        let model = create_test_model(resources, create_empty_build());

        let issues = validate_resource_id_reference(&model);
        assert_eq!(issues.len(), 1); // Only the missing reference error
        assert!(issues[0]
            .message
            .contains("no BaseMaterials with that ID exists"));
    }

    #[test]
    fn test_run_rule_dispatcher() {
        let resources = Resources {
            object: vec![create_test_object(1)],
            basematerials: Vec::new(),
        };
        let model = create_test_model(resources, create_empty_build());

        let object_issues = run_rule(&ValidationRule::ObjectIdReference, &model);
        assert!(object_issues.is_empty());

        let resource_issues = run_rule(&ValidationRule::ResourceIdReference, &model);
        assert!(resource_issues.is_empty());
    }
}

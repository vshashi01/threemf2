//! Validator module for 3MF package and model validation.
//!
//! Provides a framework for validating 3MF packages against configurable rules.

use crate::model::domain::model::Model;
use crate::package::threemf_package::ThreemfPackage;

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Critical issue that makes the model invalid.
    Error,
    /// Non-critical issue that should be addressed.
    Warning,
    /// Informational message.
    Info,
}

/// A single validation issue found during validation.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    /// The severity level of this issue.
    pub severity: Severity,
    /// Human-readable message describing the issue.
    pub message: String,
}

impl ValidationIssue {
    /// Creates a new validation issue.
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
        }
    }
}

/// Result of running validation on a package or model.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// All issues found during validation.
    pub issues: Vec<ValidationIssue>,
    /// True if no Error-level issues were found.
    pub is_valid: bool,
}

impl ValidationResult {
    /// Creates a new validation result with the given issues.
    pub fn new(issues: Vec<ValidationIssue>) -> Self {
        let is_valid = !issues.iter().any(|issue| issue.severity == Severity::Error);
        Self { issues, is_valid }
    }

    /// Creates a successful validation result with no issues.
    pub fn valid() -> Self {
        Self {
            issues: Vec::new(),
            is_valid: true,
        }
    }
}

/// Scope of validation - determines what data is needed to run a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationScope {
    /// Can run on a single model (no cross-model references needed).
    ModelOrPackage,
    /// Needs access to the full package (cross-model references).
    PackageOnly,
}

/// Validation rules that can be applied to 3MF packages or models.
/// Validation rules has per rule scope that can be checked with scope impl function
/// Scope: ModelOrPackage can be run for Model parts or the whole 3MF Package, usually when run only
/// on specific model only, the validation may fail due to missing information referenced in other Model parts
/// PackageOnly rules are only run on packages always
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationRule {
    /// Validates that object IDs are unique, start at 1, and within valid range.
    ObjectIdReference,
    /// Validates that resource ID references (pid) exist in BaseMaterials and
    /// if pindex is defined then pid should also be defined.
    BaseMaterialReference,
    /// Validates that all Build items are referencing a valid Object
    BuildItemReference,
    /// Validates that all Components are referencing a valid Object
    ComponentReference,
    /// Validates that Objects are referencing valid SliceStacks
    ObjectToSliceStackReference,
}

impl ValidationRule {
    /// Returns the scope required to run this validation rule.
    pub fn scope(&self) -> ValidationScope {
        match self {
            ValidationRule::ObjectIdReference => ValidationScope::ModelOrPackage,
            ValidationRule::BaseMaterialReference => ValidationScope::ModelOrPackage,
            ValidationRule::BuildItemReference => ValidationScope::ModelOrPackage,
            ValidationRule::ComponentReference => ValidationScope::ModelOrPackage,
            ValidationRule::ObjectToSliceStackReference => ValidationScope::PackageOnly,
        }
    }
}

/// Validator that runs a set of validation rules against packages or models.
///
/// # Example
///
/// ```ignore
/// use package::validator::{Validator, ValidationRule};
///
/// let validator = Validator::new()
///     .with_rule(ValidationRule::ObjectIdReference)
///     .with_rule(ValidationRule::ResourceIdReference);
///
/// // Validate a single model
/// let result = validator.validate_model(&model);
/// assert!(result.is_valid);
/// ```
#[derive(Debug, Clone)]
pub struct Validator {
    rules: Vec<ValidationRule>,
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    /// Creates a new validator with no rules.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Adds a validation rule to this validator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use package::validator::{Validator, ValidationRule};
    ///
    /// let validator = Validator::new()
    ///     .with_rule(ValidationRule::ObjectIdReference);
    /// ```
    pub fn with_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Validates a single model using Model-scope rules only.
    ///
    /// This method runs all specified Validation on the target Model that has scope [`ValidationScope::ModelOrPackage`].
    /// Rules with [`ValidationScope::PackageOnly`] are skipped.
    ///
    /// # Returns
    ///
    /// A [ValidationResult] containing all issues found.
    pub fn validate_model(&self, model: &Model) -> ValidationResult {
        let mut issues = Vec::new();

        for rule in &self.rules {
            if rule.scope() == ValidationScope::ModelOrPackage {
                let rule_issues =
                    crate::package::domain::validator_rules::run_rule_for_model(rule, model);
                issues.extend(rule_issues);
            }
        }

        ValidationResult::new(issues)
    }

    /// Validates a full package using all rules.
    ///
    /// This method runs all validation rules on the root model and all sub-models.
    /// Rules with [`ValidationScope::ModelOrPackage`] are run on each model individually.
    /// Rules with [`ValidationScope::PackageOnly`] are run once on the entire package.
    ///
    /// # Returns
    ///
    /// A [ValidationResult] containing all issues found across all models.
    pub fn validate_package(&self, package: &ThreemfPackage) -> ValidationResult {
        let mut issues = Vec::new();

        for rule in &self.rules {
            let rule_issues =
                crate::package::domain::validator_rules::run_rule_for_package(rule, package);
            issues.extend(rule_issues);
        }

        ValidationResult::new(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validation_result_with_warning() {
        let issues = vec![ValidationIssue::new(Severity::Warning, "Test warning")];
        let result = ValidationResult::new(issues);
        assert!(result.is_valid);
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_validation_result_with_error() {
        let issues = vec![ValidationIssue::new(Severity::Error, "Test error")];
        let result = ValidationResult::new(issues);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
    }

    #[test]
    fn test_validation_result_mixed() {
        let issues = vec![
            ValidationIssue::new(Severity::Info, "Test info"),
            ValidationIssue::new(Severity::Warning, "Test warning"),
            ValidationIssue::new(Severity::Error, "Test error"),
        ];
        let result = ValidationResult::new(issues);
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 3);
    }

    #[test]
    fn test_validator_builder_pattern() {
        let validator = Validator::new()
            .with_rule(ValidationRule::ObjectIdReference)
            .with_rule(ValidationRule::BaseMaterialReference);

        assert_eq!(validator.rules.len(), 2);
        assert!(validator.rules.contains(&ValidationRule::ObjectIdReference));
        assert!(
            validator
                .rules
                .contains(&ValidationRule::BaseMaterialReference)
        );
    }

    #[test]
    fn test_rule_scopes() {
        assert_eq!(
            ValidationRule::ObjectIdReference.scope(),
            ValidationScope::ModelOrPackage
        );
        assert_eq!(
            ValidationRule::BaseMaterialReference.scope(),
            ValidationScope::ModelOrPackage
        );
    }
}

#[cfg(test)]
mod validator_tests {
    use crate::model::domain::{
        build::{Build, Item},
        mesh::{Mesh, Triangles, Vertex, Vertices},
        model::{Model, Unit},
        object::{Object, ObjectKind, ObjectType},
        resources::Resources,
    };
    use crate::package::domain::validator::Validator;

    #[test]
    fn default_validator_equals_new() {
        let v1 = Validator::default();
        let v2 = Validator::new();
        assert_eq!(v1.rules, v2.rules);
    }

    #[test]
    fn validate_model_basic() {
        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Default::default(),
            recommendedextensions: Default::default(),
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: Default::default(),
                    pindex: Default::default(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(Mesh {
                        vertices: Vertices {
                            vertex: vec![
                                Vertex::new(0.0, 0.0, 0.0),
                                Vertex::new(1.0, 0.0, 0.0),
                                Vertex::new(0.0, 1.0, 0.0),
                            ],
                            any_extensions: Default::default(),
                        },
                        triangles: Triangles {
                            triangle: vec![],
                            any_extensions: Default::default(),
                        },
                        trianglesets: None,
                        beamlattice: None,
                        any_extensions: Default::default(),
                    })),
                    slicestackid: Default::default(),
                    slicepath: None,
                    meshresolution: None,
                    any_extensions: Default::default(),
                }],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
                any_extensions: Default::default(),
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                    ..Default::default()
                }],
                any_extensions: Default::default(),
            },
            any_extensions: Default::default(),
        };

        let validator = Validator::new();
        let result = validator.validate_model(&model);
        assert!(result.is_valid);
        assert!(result.issues.is_empty());
    }
}

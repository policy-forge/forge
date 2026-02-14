//! Validation error types for enhanced error reporting (WI-20).
//!
//! Defines `ValidationErrorCategory`, `ValidationError`, and `ValidationReport`
//! used across schema and semantic validation passes.

use serde::{Deserialize, Serialize};

/// Category of validation error (PRD M-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationErrorCategory {
    /// Error from JSON Schema validation.
    Schema,
    /// Error from semantic validation (orphaned links, missing references).
    Semantic,
}

impl std::fmt::Display for ValidationErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema => write!(f, "Schema"),
            Self::Semantic => write!(f, "Semantic"),
        }
    }
}

/// A single validation error with full context for actionable reporting (PRD M-1).
///
/// Every error MUST include path + expected + actual (PRD M-1).
/// Actual values MUST be truncated to 100 characters (SEC-1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    /// Category of error: Schema or Semantic.
    pub category: ValidationErrorCategory,
    /// JSON Path to the offending field (e.g., `$.catalog.metadata.uuid`).
    pub path: String,
    /// Human-readable description of the error.
    /// MUST NOT contain raw jsonschema crate messages (SEC-2).
    pub message: String,
    /// What the schema or rule expected (e.g., "required string field").
    pub expected: String,
    /// What was actually found (e.g., "field not present").
    /// Truncated to 100 characters with "..." suffix (SEC-1).
    pub actual: String,
}

/// Aggregated validation report (PRD M-2, S-2).
///
/// Invariant: `schema_error_count + semantic_error_count == errors.len()` (SEC-8).
/// Invariant: `is_valid == errors.is_empty()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Path to the validated artifact.
    pub artifact_path: String,
    /// Whether the artifact passed all validation.
    pub is_valid: bool,
    /// All collected errors — schema AND semantic (empty if valid).
    pub errors: Vec<ValidationError>,
    /// Count of schema errors.
    pub schema_error_count: usize,
    /// Count of semantic errors.
    pub semantic_error_count: usize,
}

impl ValidationReport {
    /// Create a new `ValidationReport` with enforced invariants.
    ///
    /// Computes `is_valid`, `schema_error_count`, and `semantic_error_count`
    /// from the provided errors list, ensuring all invariants hold.
    #[must_use]
    pub fn new(artifact_path: String, errors: Vec<ValidationError>) -> Self {
        let schema_error_count =
            errors.iter().filter(|e| e.category == ValidationErrorCategory::Schema).count();
        let semantic_error_count =
            errors.iter().filter(|e| e.category == ValidationErrorCategory::Semantic).count();
        let is_valid = errors.is_empty();

        Self { artifact_path, is_valid, errors, schema_error_count, semantic_error_count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T004: ValidationErrorCategory tests ---

    #[test]
    fn category_debug_trait() {
        let schema = ValidationErrorCategory::Schema;
        let semantic = ValidationErrorCategory::Semantic;
        assert_eq!(format!("{schema:?}"), "Schema");
        assert_eq!(format!("{semantic:?}"), "Semantic");
    }

    #[test]
    fn category_clone_trait() {
        let original = ValidationErrorCategory::Schema;
        #[allow(clippy::clone_on_copy)]
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn category_copy_trait() {
        let original = ValidationErrorCategory::Semantic;
        let copied = original;
        // original is still usable after copy
        assert_eq!(original, copied);
    }

    #[test]
    fn category_partial_eq() {
        assert_eq!(ValidationErrorCategory::Schema, ValidationErrorCategory::Schema);
        assert_ne!(ValidationErrorCategory::Schema, ValidationErrorCategory::Semantic);
    }

    #[test]
    fn category_serialize_deserialize_schema() {
        let category = ValidationErrorCategory::Schema;
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: ValidationErrorCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);
    }

    #[test]
    fn category_serialize_deserialize_semantic() {
        let category = ValidationErrorCategory::Semantic;
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: ValidationErrorCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);
    }

    #[test]
    fn category_display() {
        assert_eq!(ValidationErrorCategory::Schema.to_string(), "Schema");
        assert_eq!(ValidationErrorCategory::Semantic.to_string(), "Semantic");
    }

    // --- T005: ValidationError tests ---

    #[test]
    fn validation_error_construction() {
        let error = ValidationError {
            category: ValidationErrorCategory::Schema,
            path: "$.catalog.metadata.uuid".to_string(),
            message: "required field missing".to_string(),
            expected: "required string field".to_string(),
            actual: "field not present".to_string(),
        };
        assert_eq!(error.category, ValidationErrorCategory::Schema);
        assert_eq!(error.path, "$.catalog.metadata.uuid");
        assert_eq!(error.message, "required field missing");
        assert_eq!(error.expected, "required string field");
        assert_eq!(error.actual, "field not present");
    }

    #[test]
    fn validation_error_serialize_deserialize() {
        let error = ValidationError {
            category: ValidationErrorCategory::Semantic,
            path: "$.catalog.back-matter.resources[0]".to_string(),
            message: "orphaned link".to_string(),
            expected: "referenced resource exists".to_string(),
            actual: "resource not found".to_string(),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ValidationError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn validation_error_actual_field_truncation_enforced_by_formatter() {
        // Truncation is enforced by formatter::truncate_value() before construction.
        // Verify that format_schema_error() produces truncated actual values.
        use crate::validate::formatter::format_schema_error;

        let long_value = "x".repeat(200);
        let schema_json: serde_json::Value = serde_json::from_str(
            r#"{"type": "object", "properties": {"name": {"type": "integer"}}}"#,
        )
        .unwrap();
        let instance: serde_json::Value =
            serde_json::from_str(&format!(r#"{{"name": "{long_value}"}}"#)).unwrap();

        let validator = jsonschema::validator_for(&schema_json).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();
        assert!(!errors.is_empty());

        let formatted = format_schema_error(&errors[0], &instance);
        // SEC-1: actual must be truncated — 100 content chars + "..." = 103 max
        assert!(
            formatted.actual.chars().count() <= 103,
            "Actual should be truncated to 103 chars max, got: {}",
            formatted.actual.chars().count()
        );
    }

    // --- T006: ValidationReport tests ---

    #[test]
    fn report_new_empty_errors_is_valid() {
        let report = ValidationReport::new("test.json".to_string(), vec![]);
        assert!(report.is_valid);
        assert!(report.errors.is_empty());
        assert_eq!(report.schema_error_count, 0);
        assert_eq!(report.semantic_error_count, 0);
    }

    #[test]
    fn report_new_single_schema_error() {
        let errors = vec![ValidationError {
            category: ValidationErrorCategory::Schema,
            path: "$.catalog.uuid".to_string(),
            message: "invalid uuid".to_string(),
            expected: "valid UUID".to_string(),
            actual: "not-a-uuid".to_string(),
        }];
        let report = ValidationReport::new("test.json".to_string(), errors);
        assert!(!report.is_valid);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.schema_error_count, 1);
        assert_eq!(report.semantic_error_count, 0);
    }

    #[test]
    fn report_new_single_semantic_error() {
        let errors = vec![ValidationError {
            category: ValidationErrorCategory::Semantic,
            path: "$.catalog.groups[0].links[0].href".to_string(),
            message: "orphaned link".to_string(),
            expected: "referenced resource exists".to_string(),
            actual: "#missing-uuid".to_string(),
        }];
        let report = ValidationReport::new("test.json".to_string(), errors);
        assert!(!report.is_valid);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.schema_error_count, 0);
        assert_eq!(report.semantic_error_count, 1);
    }

    #[test]
    fn report_new_mixed_errors_counts_correct() {
        let errors = vec![
            ValidationError {
                category: ValidationErrorCategory::Schema,
                path: "$.a".to_string(),
                message: "err1".to_string(),
                expected: "x".to_string(),
                actual: "y".to_string(),
            },
            ValidationError {
                category: ValidationErrorCategory::Schema,
                path: "$.b".to_string(),
                message: "err2".to_string(),
                expected: "x".to_string(),
                actual: "y".to_string(),
            },
            ValidationError {
                category: ValidationErrorCategory::Semantic,
                path: "$.c".to_string(),
                message: "err3".to_string(),
                expected: "x".to_string(),
                actual: "y".to_string(),
            },
        ];
        let report = ValidationReport::new("test.json".to_string(), errors);
        assert!(!report.is_valid);
        assert_eq!(report.errors.len(), 3);
        assert_eq!(report.schema_error_count, 2);
        assert_eq!(report.semantic_error_count, 1);
        // SEC-8: counts must sum to total
        assert_eq!(report.schema_error_count + report.semantic_error_count, report.errors.len());
    }

    #[test]
    fn report_invariant_is_valid_equals_errors_empty() {
        let empty_report = ValidationReport::new("a.json".to_string(), vec![]);
        assert_eq!(empty_report.is_valid, empty_report.errors.is_empty());

        let error_report = ValidationReport::new(
            "b.json".to_string(),
            vec![ValidationError {
                category: ValidationErrorCategory::Schema,
                path: "$".to_string(),
                message: "err".to_string(),
                expected: "x".to_string(),
                actual: "y".to_string(),
            }],
        );
        assert_eq!(error_report.is_valid, error_report.errors.is_empty());
    }
}

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
    /// Guaranteed to contain at most 100 characters plus an ellipsis (SEC-1).
    #[serde(deserialize_with = "deserialize_actual")]
    actual: String,
}

impl ValidationError {
    /// Create a validation error while enforcing the SEC-1 diagnostic-value cap.
    #[must_use]
    pub fn new(
        category: ValidationErrorCategory,
        path: String,
        message: String,
        expected: String,
        actual: impl AsRef<str>,
    ) -> Self {
        Self { category, path, message, expected, actual: truncate_actual(actual.as_ref()) }
    }

    /// The bounded diagnostic representation of the actual value.
    #[must_use]
    pub fn actual(&self) -> &str {
        &self.actual
    }
}

fn deserialize_actual<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(|actual| truncate_actual(&actual))
}

fn truncate_actual(value: &str) -> String {
    const MAX_ACTUAL_CHARS: usize = 100;
    if value.chars().count() <= MAX_ACTUAL_CHARS {
        value.to_string()
    } else {
        format!("{}...", value.chars().take(MAX_ACTUAL_CHARS).collect::<String>())
    }
}

/// Aggregated validation report (PRD M-2, S-2).
///
/// Invariant: `schema_error_count + semantic_error_count == errors.len()` (SEC-8).
/// Invariant: `is_valid == errors.is_empty()`.
///
/// Fields are private to prevent invariant violations. Use [`ValidationReport::new`]
/// to construct, and accessor methods to read. Custom `Deserialize` recomputes
/// derived fields from `errors`, so deserialized instances always satisfy invariants.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    artifact_path: String,
    model_type: String,
    declared_oscal_version: Option<String>,
    schema_version_used: String,
    supported_input: bool,
    is_valid: bool,
    errors: Vec<ValidationError>,
    schema_error_count: usize,
    semantic_error_count: usize,
}

impl<'de> Deserialize<'de> for ValidationReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            artifact_path: String,
            #[serde(default = "unknown_model_type")]
            model_type: String,
            #[serde(default)]
            declared_oscal_version: Option<String>,
            #[serde(default = "current_schema_version")]
            schema_version_used: String,
            // Reports written before this field existed cannot prove their
            // input compatibility, so they default to the safe value.
            #[serde(default)]
            supported_input: Option<bool>,
            errors: Vec<ValidationError>,
        }
        let raw = Raw::deserialize(deserializer)?;
        let supported_input = raw.supported_input.unwrap_or(false);
        Ok(Self::new_with_schema_context(
            raw.artifact_path,
            raw.model_type,
            raw.declared_oscal_version,
            raw.schema_version_used,
            supported_input,
            raw.errors,
        ))
    }
}

impl ValidationReport {
    /// Create a new `ValidationReport` with enforced invariants.
    ///
    /// Computes `is_valid`, `schema_error_count`, and `semantic_error_count`
    /// from the provided errors list, ensuring all invariants hold.
    #[must_use]
    pub fn new(artifact_path: String, errors: Vec<ValidationError>) -> Self {
        Self::new_with_context(artifact_path, "unknown".to_string(), None, false, errors)
    }

    /// Create a report with the model and version evidence used by validation.
    #[must_use]
    pub fn new_with_context(
        artifact_path: String,
        model_type: String,
        declared_oscal_version: Option<String>,
        supported_input: bool,
        errors: Vec<ValidationError>,
    ) -> Self {
        Self::new_with_schema_context(
            artifact_path,
            model_type,
            declared_oscal_version,
            crate::validate::version::SCHEMA_VERSION_USED.to_string(),
            supported_input,
            errors,
        )
    }

    fn new_with_schema_context(
        artifact_path: String,
        model_type: String,
        declared_oscal_version: Option<String>,
        schema_version_used: String,
        supported_input: bool,
        errors: Vec<ValidationError>,
    ) -> Self {
        let schema_error_count =
            errors.iter().filter(|e| e.category == ValidationErrorCategory::Schema).count();
        // Derive this count to preserve SEC-8 if categories are added later.
        let semantic_error_count = errors.len() - schema_error_count;
        let is_valid = errors.is_empty();

        Self {
            artifact_path,
            model_type,
            declared_oscal_version,
            schema_version_used,
            supported_input,
            is_valid,
            errors,
            schema_error_count,
            semantic_error_count,
        }
    }

    /// Path to the validated artifact.
    #[must_use]
    pub fn artifact_path(&self) -> &str {
        &self.artifact_path
    }

    /// Detected or explicitly selected model type.
    #[must_use]
    pub fn model_type(&self) -> &str {
        &self.model_type
    }

    /// OSCAL version declared by the document, when it is a string.
    #[must_use]
    pub fn declared_oscal_version(&self) -> Option<&str> {
        self.declared_oscal_version.as_deref()
    }

    /// Actual pinned schema baseline used for validation.
    #[must_use]
    pub fn schema_version_used(&self) -> &str {
        &self.schema_version_used
    }

    /// Whether the declared version is accepted by the compatibility policy.
    #[must_use]
    pub fn supported_input(&self) -> bool {
        self.supported_input
    }

    /// Whether the artifact passed all validation.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    /// All collected errors — schema AND semantic.
    #[must_use]
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Count of schema errors.
    #[must_use]
    pub fn schema_error_count(&self) -> usize {
        self.schema_error_count
    }

    /// Count of semantic errors.
    #[must_use]
    pub fn semantic_error_count(&self) -> usize {
        self.semantic_error_count
    }
}

fn unknown_model_type() -> String {
    "unknown".to_string()
}

fn current_schema_version() -> String {
    crate::validate::version::SCHEMA_VERSION_USED.to_string()
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
    fn validation_error_constructor_and_deserializer_enforce_actual_cap() {
        let error = ValidationError::new(
            ValidationErrorCategory::Schema,
            "$.catalog.metadata.uuid".to_string(),
            "required field missing".to_string(),
            "required string field".to_string(),
            "x".repeat(101),
        );
        assert_eq!(error.category, ValidationErrorCategory::Schema);
        assert_eq!(error.actual().chars().count(), 103);

        let deserialized: ValidationError = serde_json::from_str(
            r#"{"category":"Semantic","path":"$.x","message":"m","expected":"e","actual":"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#,
        )
        .unwrap();
        assert_eq!(deserialized.actual().chars().count(), 103);
    }

    #[test]
    fn validation_error_serialize_deserialize() {
        let error = ValidationError::new(
            ValidationErrorCategory::Semantic,
            "$.catalog.back-matter.resources[0]".to_string(),
            "orphaned link".to_string(),
            "referenced resource exists".to_string(),
            "resource not found",
        );
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ValidationError = serde_json::from_str(&json).unwrap();
        assert_eq!(error, deserialized);
    }

    #[test]
    fn validation_error_actual_field_truncation_enforced_by_formatter() {
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
        assert!(formatted.actual().chars().count() <= 103);
    }

    // --- T006: ValidationReport tests ---

    #[test]
    fn report_new_empty_errors_is_valid() {
        let report = ValidationReport::new("test.json".to_string(), vec![]);
        assert!(report.is_valid());
        assert!(report.errors().is_empty());
        assert_eq!(report.schema_error_count(), 0);
        assert_eq!(report.semantic_error_count(), 0);
    }

    #[test]
    fn legacy_report_defaults_supported_input_to_false() {
        let legacy = r#"{
            "artifact_path": "catalog.json",
            "is_valid": true,
            "errors": [],
            "schema_error_count": 0,
            "semantic_error_count": 0
        }"#;
        let report: ValidationReport = serde_json::from_str(legacy).unwrap();
        assert!(report.is_valid());
        assert!(!report.supported_input());
    }

    #[test]
    fn explicit_supported_input_overrides_legacy_validity() {
        let report = r#"{
            "artifact_path": "catalog.json",
            "supported_input": false,
            "is_valid": true,
            "errors": []
        }"#;
        let report: ValidationReport = serde_json::from_str(report).unwrap();
        assert!(!report.supported_input());
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
        assert!(!report.is_valid());
        assert_eq!(report.errors().len(), 1);
        assert_eq!(report.schema_error_count(), 1);
        assert_eq!(report.semantic_error_count(), 0);
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
        assert!(!report.is_valid());
        assert_eq!(report.errors().len(), 1);
        assert_eq!(report.schema_error_count(), 0);
        assert_eq!(report.semantic_error_count(), 1);
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
        assert!(!report.is_valid());
        assert_eq!(report.errors().len(), 3);
        assert_eq!(report.schema_error_count(), 2);
        assert_eq!(report.semantic_error_count(), 1);
        // SEC-8: counts must sum to total
        assert_eq!(
            report.schema_error_count() + report.semantic_error_count(),
            report.errors().len()
        );
    }

    #[test]
    fn report_invariant_is_valid_equals_errors_empty() {
        let empty_report = ValidationReport::new("a.json".to_string(), vec![]);
        assert_eq!(empty_report.is_valid(), empty_report.errors().is_empty());

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
        assert_eq!(error_report.is_valid(), error_report.errors().is_empty());
    }
}

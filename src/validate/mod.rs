//! OSCAL schema validation module (WI-19, WI-20).
//!
//! Validates OSCAL JSON artifacts against embedded NIST OSCAL v1.2.0 JSON schemas.
//! Supports Catalog and Component Definition model types with auto-detection.
//!
//! WI-20 adds enhanced error reporting with:
//! - Actionable error messages with JSON Path notation (M-1)
//! - Semantic validation for orphaned links and missing references (M-3, M-4)
//! - Categorized error reports with summary counts (M-2, M-6)
//! - Text and JSON report rendering (S-1, S-2)

pub mod error_types;
pub mod formatter;
pub mod report;
pub mod semantic;

pub use error_types::{ValidationError, ValidationErrorCategory, ValidationReport};

use std::path::{Path, PathBuf};

use serde_json::Value;
use tracing::{debug, info};

/// Maximum file size for validation (50MB) — SEC-3.
const MAX_VALIDATE_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Supported OSCAL model types for schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalModelType {
    Catalog,
    ComponentDefinition,
}

impl std::fmt::Display for OscalModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Catalog => write!(f, "catalog"),
            Self::ComponentDefinition => write!(f, "component-definition"),
        }
    }
}

/// Result of schema validation.
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether the artifact is valid (`errors.is_empty()`).
    pub is_valid: bool,
    /// Detected or specified model type.
    pub model_type: OscalModelType,
    /// All schema validation errors (empty if valid).
    pub errors: Vec<SchemaError>,
}

/// A single schema validation error.
#[derive(Debug)]
pub struct SchemaError {
    /// Human-readable error message.
    pub message: String,
    /// JSON pointer path to the failing element (e.g., "/catalog/metadata/title").
    pub instance_path: Option<String>,
    /// JSON pointer path within the schema that was violated.
    pub schema_path: Option<String>,
}

/// Errors from validation operations.
#[derive(Debug, thiserror::Error)]
pub enum ValidateError {
    #[error("Failed to read artifact file: {}", path.display())]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error(
        "Unable to detect OSCAL model type from JSON structure. Use --schema-type to specify the model type."
    )]
    UnknownModelType,

    #[error("Schema compilation failed for {model_type}: {message}")]
    SchemaCompilation { model_type: String, message: String },

    #[allow(clippy::cast_precision_loss)]
    #[error("Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)")]
    FileTooLarge { size_mb: f64, limit_mb: u64 },
}

/// Detect the OSCAL model type from a parsed JSON value.
///
/// Inspects top-level keys: `"catalog"` → `Catalog`, `"component-definition"` → `ComponentDefinition`.
///
/// # Errors
///
/// Returns `ValidateError::UnknownModelType` if no recognized top-level key is found.
pub fn detect_model_type(json: &Value) -> Result<OscalModelType, ValidateError> {
    if json.get("catalog").is_some() {
        Ok(OscalModelType::Catalog)
    } else if json.get("component-definition").is_some() {
        Ok(OscalModelType::ComponentDefinition)
    } else {
        Err(ValidateError::UnknownModelType)
    }
}

/// Load the embedded OSCAL JSON schema for a given model type.
///
/// Returns the schema as a `serde_json::Value`.
///
/// # Errors
///
/// Returns `ValidateError::SchemaCompilation` if the embedded schema cannot be parsed.
pub fn load_schema(model_type: OscalModelType) -> Result<Value, ValidateError> {
    debug!("Loading embedded OSCAL schema for {model_type}");
    let schema_str = match model_type {
        OscalModelType::Catalog => {
            include_str!("../../schemas/oscal_catalog_schema.json")
        }
        OscalModelType::ComponentDefinition => {
            include_str!("../../schemas/oscal_component_schema.json")
        }
    };

    let schema =
        serde_json::from_str(schema_str).map_err(|e| ValidateError::SchemaCompilation {
            model_type: model_type.to_string(),
            message: e.to_string(),
        })?;
    debug!("Schema loaded successfully for {model_type}");
    Ok(schema)
}

/// Validate a JSON value against the OSCAL schema for the given model type.
///
/// Collects all errors (does not stop at the first).
///
/// # Errors
///
/// Returns `ValidateError::SchemaCompilation` if the schema cannot be compiled.
pub fn validate_artifact(
    json: &Value,
    model_type: OscalModelType,
) -> Result<ValidationResult, ValidateError> {
    let schema = load_schema(model_type)?;

    let validator =
        jsonschema::validator_for(&schema).map_err(|e| ValidateError::SchemaCompilation {
            model_type: model_type.to_string(),
            message: e.to_string(),
        })?;

    let errors: Vec<SchemaError> = validator
        .iter_errors(json)
        .map(|error| {
            let instance_path = error.instance_path().to_string();
            let schema_path = error.schema_path().to_string();
            SchemaError {
                message: error.to_string(),
                instance_path: if instance_path.is_empty() { None } else { Some(instance_path) },
                schema_path: if schema_path.is_empty() { None } else { Some(schema_path) },
            }
        })
        .collect();

    if errors.is_empty() {
        info!("Schema validation passed for {model_type}");
    } else {
        info!("Schema validation failed for {model_type}: {} error(s)", errors.len());
    }

    Ok(ValidationResult { is_valid: errors.is_empty(), model_type, errors })
}

/// Check file size against the 50MB limit (SEC-3).
///
/// # Errors
///
/// Returns `ValidateError::FileTooLarge` if the file exceeds the limit.
/// Returns `ValidateError::FileRead` if the file metadata cannot be read.
#[allow(clippy::cast_precision_loss)]
pub fn check_file_size(path: &Path) -> Result<(), ValidateError> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| ValidateError::FileRead { path: path.to_path_buf(), source: e })?;
    let size = metadata.len();
    if size > MAX_VALIDATE_FILE_SIZE {
        return Err(ValidateError::FileTooLarge {
            size_mb: size as f64 / (1024.0 * 1024.0),
            limit_mb: MAX_VALIDATE_FILE_SIZE / (1024 * 1024),
        });
    }
    Ok(())
}

/// Run full validation (schema + semantic) and produce a `ValidationReport`.
///
/// This is the main entry point for enhanced validation (WI-20).
/// Uses `load_schema()` + `jsonschema::validator_for()` directly to get raw
/// `jsonschema::ValidationError`s, transforms each through `format_schema_error()`,
/// then runs `SemanticValidator` for semantic checks, and combines results.
///
/// MUST collect ALL errors from both passes (PRD M-2).
/// MUST NOT stop at the first error.
///
/// # Errors
///
/// Returns `ValidateError::SchemaCompilation` if the schema cannot be compiled.
pub fn run_full_validation(
    artifact_path: &str,
    json: &Value,
    model_type: OscalModelType,
) -> Result<error_types::ValidationReport, ValidateError> {
    let schema = load_schema(model_type)?;

    let validator =
        jsonschema::validator_for(&schema).map_err(|e| ValidateError::SchemaCompilation {
            model_type: model_type.to_string(),
            message: e.to_string(),
        })?;

    // Schema validation: collect all raw errors and format them
    let schema_errors: Vec<error_types::ValidationError> = validator
        .iter_errors(json)
        .map(|error| formatter::format_schema_error(&error, json))
        .collect();

    // Semantic validation
    let semantic_validator = semantic::SemanticValidator;
    let semantic_errors = semantic_validator.validate(json, model_type);

    // Combine all errors
    let mut all_errors = schema_errors;
    all_errors.extend(semantic_errors);

    if all_errors.is_empty() {
        info!("Full validation passed for {model_type}");
    } else {
        info!("Full validation failed for {model_type}: {} error(s)", all_errors.len());
    }

    Ok(error_types::ValidationReport::new(artifact_path.to_string(), all_errors))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- detect_model_type tests (T010) ---

    #[test]
    fn detect_model_type_catalog() {
        let json: Value = serde_json::from_str(r#"{"catalog": {}}"#).unwrap();
        let result = detect_model_type(&json).unwrap();
        assert_eq!(result, OscalModelType::Catalog);
    }

    #[test]
    fn detect_model_type_component_definition() {
        let json: Value = serde_json::from_str(r#"{"component-definition": {}}"#).unwrap();
        let result = detect_model_type(&json).unwrap();
        assert_eq!(result, OscalModelType::ComponentDefinition);
    }

    #[test]
    fn detect_model_type_unknown_returns_error() {
        let json: Value = serde_json::from_str(r#"{"profile": {}}"#).unwrap();
        let result = detect_model_type(&json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidateError::UnknownModelType));
    }

    #[test]
    fn detect_model_type_empty_object_returns_error() {
        let json: Value = serde_json::from_str("{}").unwrap();
        let result = detect_model_type(&json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidateError::UnknownModelType));
    }

    // --- load_schema tests (T012) ---

    #[test]
    fn load_schema_catalog_returns_valid_json() {
        let schema = load_schema(OscalModelType::Catalog).unwrap();
        assert!(schema.is_object());
        assert!(schema.get("$schema").is_some());
    }

    #[test]
    fn load_schema_component_definition_returns_valid_json() {
        let schema = load_schema(OscalModelType::ComponentDefinition).unwrap();
        assert!(schema.is_object());
        assert!(schema.get("$schema").is_some());
    }

    // --- validate_artifact tests (T014) ---

    #[test]
    fn validate_valid_minimal_catalog() {
        let catalog_json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
                    "metadata": {
                        "title": "Test Catalog",
                        "last-modified": "2026-01-01T00:00:00Z",
                        "version": "1.0",
                        "oscal-version": "1.2.0"
                    }
                }
            }"#,
        )
        .unwrap();
        let result = validate_artifact(&catalog_json, OscalModelType::Catalog).unwrap();
        assert!(result.is_valid, "Expected valid catalog, got errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
        assert_eq!(result.model_type, OscalModelType::Catalog);
    }

    #[test]
    fn validate_invalid_catalog_missing_metadata() {
        let catalog_json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d"
                }
            }"#,
        )
        .unwrap();
        let result = validate_artifact(&catalog_json, OscalModelType::Catalog).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn validate_collects_all_errors_not_just_first() {
        // Missing both metadata and uuid
        let catalog_json: Value = serde_json::from_str(
            r#"{
                "catalog": {}
            }"#,
        )
        .unwrap();
        let result = validate_artifact(&catalog_json, OscalModelType::Catalog).unwrap();
        assert!(!result.is_valid);
        assert!(
            result.errors.len() > 1,
            "Expected multiple errors, got {}: {:?}",
            result.errors.len(),
            result.errors
        );
    }

    #[test]
    fn validate_errors_have_instance_path() {
        let catalog_json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "uuid": "not-a-uuid",
                    "metadata": {
                        "title": "Test",
                        "last-modified": "2026-01-01T00:00:00Z",
                        "version": "1.0",
                        "oscal-version": "1.2.0"
                    }
                }
            }"#,
        )
        .unwrap();
        let result = validate_artifact(&catalog_json, OscalModelType::Catalog).unwrap();
        // The invalid uuid should produce an error with instance_path populated
        assert!(!result.is_valid, "Expected invalid result for malformed UUID");
        let has_path = result.errors.iter().any(|e| e.instance_path.is_some());
        assert!(has_path, "Expected at least one error with instance_path populated");
    }

    #[test]
    fn validate_valid_minimal_component_definition() {
        let comp_json: Value = serde_json::from_str(
            r#"{
                "component-definition": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
                    "metadata": {
                        "title": "Test Component",
                        "last-modified": "2026-01-01T00:00:00Z",
                        "version": "1.0",
                        "oscal-version": "1.2.0"
                    }
                }
            }"#,
        )
        .unwrap();
        let result = validate_artifact(&comp_json, OscalModelType::ComponentDefinition).unwrap();
        assert!(
            result.is_valid,
            "Expected valid component definition, got errors: {:?}",
            result.errors
        );
    }

    // --- Display tests ---

    #[test]
    fn oscal_model_type_display() {
        assert_eq!(OscalModelType::Catalog.to_string(), "catalog");
        assert_eq!(OscalModelType::ComponentDefinition.to_string(), "component-definition");
    }

    #[test]
    fn validate_error_unknown_model_type_display() {
        let err = ValidateError::UnknownModelType;
        assert!(err.to_string().contains("--schema-type"));
    }

    #[test]
    fn validate_error_file_too_large_display() {
        let err = ValidateError::FileTooLarge { size_mb: 75.5, limit_mb: 50 };
        assert!(err.to_string().contains("75.5MB"));
        assert!(err.to_string().contains("50MB"));
    }

    // --- T027: run_full_validation tests ---

    #[test]
    fn full_validation_valid_catalog() {
        let json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
                    "metadata": {
                        "title": "Test Catalog",
                        "last-modified": "2026-01-01T00:00:00Z",
                        "version": "1.0",
                        "oscal-version": "1.2.0"
                    }
                }
            }"#,
        )
        .unwrap();
        let report = run_full_validation("test.json", &json, OscalModelType::Catalog).unwrap();
        assert!(report.is_valid());
        assert!(report.errors().is_empty());
    }

    #[test]
    fn full_validation_invalid_catalog_schema_errors() {
        let json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d"
                }
            }"#,
        )
        .unwrap();
        let report = run_full_validation("test.json", &json, OscalModelType::Catalog).unwrap();
        assert!(!report.is_valid());
        assert!(report.schema_error_count() > 0);
        // Errors should use JSON Path notation
        for error in report.errors() {
            if error.category == error_types::ValidationErrorCategory::Schema {
                assert!(error.path.starts_with('$'), "Path should start with $: {}", error.path);
            }
        }
    }

    #[test]
    fn full_validation_semantic_errors_included() {
        // Create artifact with orphaned links
        let json: Value = serde_json::from_str(
            r##"{
                "catalog": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
                    "metadata": {
                        "title": "Test",
                        "last-modified": "2026-01-01T00:00:00Z",
                        "version": "1.0",
                        "oscal-version": "1.2.0"
                    },
                    "groups": [{
                        "id": "group-1",
                        "title": "Test Group",
                        "links": [{"href": "#orphaned-uuid", "rel": "reference"}]
                    }]
                }
            }"##,
        )
        .unwrap();
        let report = run_full_validation("test.json", &json, OscalModelType::Catalog).unwrap();
        assert!(!report.is_valid());
        assert!(report.semantic_error_count() > 0);
        assert!(report.errors().iter().any(|e| e.message.contains("orphaned")));
    }

    #[test]
    fn full_validation_both_schema_and_semantic_errors() {
        // Missing metadata AND orphaned link
        let json: Value = serde_json::from_str(
            r##"{
                "catalog": {
                    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
                    "groups": [{
                        "id": "g1",
                        "title": "Group",
                        "links": [{"href": "#orphan", "rel": "ref"}]
                    }]
                }
            }"##,
        )
        .unwrap();
        let report = run_full_validation("test.json", &json, OscalModelType::Catalog).unwrap();
        assert!(!report.is_valid());
        assert!(report.schema_error_count() > 0);
        assert!(report.semantic_error_count() > 0);
        // SEC-8: counts must sum correctly
        assert_eq!(
            report.schema_error_count() + report.semantic_error_count(),
            report.errors().len()
        );
    }

    #[test]
    fn full_validation_no_raw_crate_messages() {
        // SEC-2: error messages should not contain raw crate text
        let json: Value = serde_json::from_str(
            r#"{
                "catalog": {
                    "uuid": "not-a-valid-uuid"
                }
            }"#,
        )
        .unwrap();
        let report = run_full_validation("test.json", &json, OscalModelType::Catalog).unwrap();
        for error in report.errors() {
            assert!(
                !error.message.contains("jsonschema"),
                "Raw crate message leaked: {}",
                error.message
            );
            assert!(!error.message.contains("::"), "Rust path leaked: {}", error.message);
        }
    }
}

//! OSCAL schema validation module (WI-19, WI-20).
//!
//! Validates OSCAL JSON artifacts against embedded NIST OSCAL v1.2.3 JSON schemas.
//! Supports Catalog, Component Definition, Profile, and Control Mapping model types with
//! auto-detection.
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
pub mod version;

pub use error_types::{ValidationError, ValidationErrorCategory, ValidationReport};

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;
use tracing::{debug, info};

pub use crate::types::OscalModelType;

/// Maximum file size for validation — reuses the shared constant from `io`.
const MAX_VALIDATE_FILE_SIZE: u64 = crate::io::MAX_FILE_SIZE;

/// Result of schema validation.
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether the artifact is valid (`errors.is_empty()`).
    pub is_valid: bool,
    /// Detected or specified model type.
    pub model_type: OscalModelType,
    /// Document-declared OSCAL version, when it is a string.
    pub declared_oscal_version: Option<String>,
    /// Actual pinned schema baseline used for validation.
    pub schema_version_used: &'static str,
    /// Whether the declaration is accepted by the current compatibility policy.
    pub supported_input: bool,
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
    /// Failed to read the artifact file from disk.
    #[error("Failed to read artifact file: {}", path.display())]
    FileRead {
        /// Path to the artifact file that could not be read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse the artifact as valid JSON.
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// The JSON artifact does not contain a recognized OSCAL root key.
    #[error(
        "Unable to detect OSCAL model type from JSON structure. Use --schema-type to specify the model type."
    )]
    UnknownModelType,

    /// The embedded OSCAL schema could not be compiled for validation.
    #[error("Schema compilation failed for {model_type}: {message}")]
    SchemaCompilation {
        /// The OSCAL model type for which the schema failed.
        model_type: String,
        /// The error message from the schema compiler.
        message: String,
        /// Structured compiler or schema-parse cause.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// The artifact contains multiple OSCAL model types in one file.
    #[error(
        "Ambiguous OSCAL artifact: file contains multiple model types ({detail}). Each file must contain exactly one OSCAL model."
    )]
    AmbiguousArtifact {
        /// Comma-separated list of the model types detected.
        detail: String,
    },

    /// The artifact file exceeds the maximum allowed size for validation.
    #[allow(clippy::cast_precision_loss)]
    #[error("Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)")]
    FileTooLarge {
        /// Actual file size in megabytes.
        size_mb: f64,
        /// Maximum allowed size in megabytes.
        limit_mb: u64,
    },
}

/// Detect the OSCAL model type from a parsed JSON value.
///
/// Inspects top-level object roots: `catalog` → `Catalog`,
/// `component-definition` → `ComponentDefinition`, `profile` → `Profile`,
/// and `mapping-collection` → `Mapping`. Roots mapped to `null` or another
/// non-object value are ignored. Exactly one recognized root must be present.
///
/// # Errors
///
/// Returns [`ValidateError::UnknownModelType`] when no recognized object root
/// is found, or [`ValidateError::AmbiguousArtifact`] when multiple roots exist.
pub fn detect_model_type(json: &Value) -> Result<OscalModelType, ValidateError> {
    let roots = [
        ("catalog", OscalModelType::Catalog),
        ("component-definition", OscalModelType::ComponentDefinition),
        ("profile", OscalModelType::Profile),
        ("mapping-collection", OscalModelType::Mapping),
    ];
    let found: Vec<_> = roots
        .iter()
        .filter(|(root, _)| matches!(json.get(*root), Some(Value::Object(_))))
        .collect();

    match found.as_slice() {
        [] => Err(ValidateError::UnknownModelType),
        [(_, model)] => Ok(*model),
        _ => Err(ValidateError::AmbiguousArtifact {
            detail: found.iter().map(|(root, _)| *root).collect::<Vec<_>>().join(", "),
        }),
    }
}

/// Load the embedded OSCAL JSON schema for a given model type.
///
/// Returns the schema as a `serde_json::Value`.
///
/// # Errors
///
/// Returns [`ValidateError::SchemaCompilation`] if the embedded schema cannot be parsed.
pub fn load_schema(model_type: OscalModelType) -> Result<Value, ValidateError> {
    debug!("Loading embedded OSCAL schema for {model_type}");
    let schema_str = match model_type {
        OscalModelType::Catalog => {
            include_str!("../../schemas/oscal_catalog_schema.json")
        }
        OscalModelType::ComponentDefinition => {
            include_str!("../../schemas/oscal_component_schema.json")
        }
        OscalModelType::Profile => {
            include_str!("../../schemas/oscal_profile_schema.json")
        }
        OscalModelType::Mapping => {
            include_str!("../../schemas/oscal_mapping_schema.json")
        }
    };

    let schema =
        serde_json::from_str(schema_str).map_err(|source| ValidateError::SchemaCompilation {
            model_type: model_type.to_string(),
            message: source.to_string(),
            source: Box::new(source),
        })?;
    debug!("Schema loaded successfully for {model_type}");
    Ok(schema)
}

static CATALOG_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
static COMPONENT_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
static PROFILE_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
static MAPPING_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();

/// Return the process-wide compiled validator for an OSCAL model.
///
/// Successful compilation is cached. Failures are deliberately retried so each
/// returned [`ValidateError::SchemaCompilation`] retains its typed source chain.
pub(crate) fn compiled_validator(
    model_type: OscalModelType,
) -> Result<&'static jsonschema::Validator, ValidateError> {
    let cell = match model_type {
        OscalModelType::Catalog => &CATALOG_VALIDATOR,
        OscalModelType::ComponentDefinition => &COMPONENT_VALIDATOR,
        OscalModelType::Profile => &PROFILE_VALIDATOR,
        OscalModelType::Mapping => &MAPPING_VALIDATOR,
    };
    if let Some(validator) = cell.get() {
        return Ok(validator);
    }

    let schema = load_schema(model_type)?;
    let validator =
        jsonschema::validator_for(&schema).map_err(|source| ValidateError::SchemaCompilation {
            model_type: model_type.to_string(),
            message: source.to_string(),
            source: Box::new(source),
        })?;
    let _ = cell.set(validator);

    cell.get().ok_or_else(|| ValidateError::SchemaCompilation {
        model_type: model_type.to_string(),
        message: "compiled validator was unavailable after initialization".to_string(),
        source: Box::new(std::io::Error::other("validator cache initialization failed")),
    })
}

/// Collect schema and version-policy errors in JSON Path notation for both public validation paths.
fn collect_schema_and_version_errors(
    json: &Value,
    model_type: OscalModelType,
) -> Result<(Vec<ValidationError>, Option<String>, bool), ValidateError> {
    let validator = compiled_validator(model_type)?;
    let mut errors = validator
        .iter_errors(json)
        .map(|error| formatter::format_schema_error(&error, json))
        .collect::<Vec<_>>();
    let version = version::inspect_oscal_version(json, model_type);
    let declared = version.declared;
    let supported = version.supported;
    if let Some(error) = version.error {
        errors.push(error);
    }
    Ok((errors, declared, supported))
}

/// Validate a JSON value against the OSCAL schema for the given model type.
///
/// Collects all errors (does not stop at the first).
///
/// # Errors
///
/// Returns [`ValidateError::SchemaCompilation`] if the schema cannot be compiled.
pub fn validate_artifact(
    json: &Value,
    model_type: OscalModelType,
) -> Result<ValidationResult, ValidateError> {
    let (errors, declared_oscal_version, supported_input) =
        collect_schema_and_version_errors(json, model_type)?;
    let schema_errors = errors
        .into_iter()
        .map(|error| SchemaError {
            message: error.message,
            instance_path: Some(error.path),
            schema_path: None,
        })
        .collect::<Vec<_>>();

    if schema_errors.is_empty() {
        info!("Schema validation passed for {model_type}");
    } else {
        info!("Schema validation failed for {model_type}: {} error(s)", schema_errors.len());
    }

    Ok(ValidationResult {
        is_valid: schema_errors.is_empty(),
        model_type,
        declared_oscal_version,
        schema_version_used: version::SCHEMA_VERSION_USED,
        supported_input,
        errors: schema_errors,
    })
}

/// Read one validation artifact while enforcing the 50MB limit (SEC-3).
///
/// The limit is enforced on bytes read from one open descriptor, preventing a
/// metadata/read time-of-check-time-of-use race. Like `File::open`, this API
/// follows symlinks; callers that prohibit symlinks must enforce that policy
/// before invoking it.
///
/// # Errors
///
/// Returns [`ValidateError::FileTooLarge`] when more than the configured limit
/// can be read, or [`ValidateError::FileRead`] for open/read failures.
#[allow(clippy::cast_precision_loss)]
pub fn read_bounded_file(path: &Path) -> Result<Vec<u8>, ValidateError> {
    use std::io::Read as _;

    let file = std::fs::File::open(path)
        .map_err(|source| ValidateError::FileRead { path: path.to_path_buf(), source })?;
    let mut bytes = Vec::new();
    file.take(MAX_VALIDATE_FILE_SIZE + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| ValidateError::FileRead { path: path.to_path_buf(), source })?;
    if bytes.len() as u64 > MAX_VALIDATE_FILE_SIZE {
        return Err(ValidateError::FileTooLarge {
            size_mb: bytes.len() as f64 / (1024.0 * 1024.0),
            limit_mb: MAX_VALIDATE_FILE_SIZE / (1024 * 1024),
        });
    }
    Ok(bytes)
}

/// Check file size against the 50MB limit (SEC-3).
///
/// Prefer [`read_bounded_file`] and validate the returned bytes. This legacy
/// preflight preserves the public API but cannot bind its read to a later read.
///
/// # Errors
///
/// Returns [`ValidateError::FileTooLarge`] when the input exceeds the configured
/// limit, or [`ValidateError::FileRead`] when it cannot be opened or read.
pub fn check_file_size(path: &Path) -> Result<(), ValidateError> {
    read_bounded_file(path).map(|_| ())
}

/// Run full validation (schema + semantic) and produce a `ValidationReport`.
///
/// This is the main entry point for enhanced validation (WI-20).
/// Reuses the schema and version-policy collection shared with
/// [`validate_artifact`], then runs `SemanticValidator` and combines results.
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
    let (mut all_errors, declared_oscal_version, supported_input) =
        collect_schema_and_version_errors(json, model_type)?;

    let semantic_validator = semantic::SemanticValidator;
    all_errors.extend(semantic_validator.validate(json, model_type));

    if all_errors.is_empty() {
        info!("Full validation passed for {model_type}");
    } else {
        info!("Full validation failed for {model_type}: {} error(s)", all_errors.len());
    }

    Ok(error_types::ValidationReport::new_with_context(
        artifact_path.to_string(),
        model_type.to_string(),
        declared_oscal_version,
        supported_input,
        all_errors,
    ))
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
    fn detect_model_type_mapping() {
        let json: Value = serde_json::from_str(r#"{"mapping-collection": {}}"#).unwrap();
        let result = detect_model_type(&json).unwrap();
        assert_eq!(result, OscalModelType::Mapping);
    }

    #[test]
    fn detect_model_type_unknown_returns_error() {
        // Use a key that is not any recognized OSCAL root key
        let json: Value = serde_json::from_str(r#"{"unknown-oscal-type": {}}"#).unwrap();
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
    fn detect_model_type_rejects_null_and_non_object_roots() {
        for json in [serde_json::json!({"catalog": null}), serde_json::json!({"profile": []})] {
            assert!(matches!(detect_model_type(&json), Err(ValidateError::UnknownModelType)));
        }
    }

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

    #[test]
    fn load_schema_mapping_returns_valid_json() {
        let schema = load_schema(OscalModelType::Mapping).unwrap();
        assert!(schema.is_object());
        assert_eq!(
            schema.get("$id").and_then(Value::as_str),
            Some("http://csrc.nist.gov/ns/oscal/1.2.3/oscal-mapping-schema.json")
        );
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
    fn validate_valid_minimal_mapping_collection() {
        let mapping_json: Value = serde_json::from_str(
            r#"{
                "mapping-collection": {
                    "uuid": "11111111-1111-4111-8111-111111111111",
                    "metadata": {
                        "title": "Reviewed mapping",
                        "last-modified": "2026-08-22T17:00:00Z",
                        "version": "1.0.0",
                        "oscal-version": "1.2.3"
                    },
                    "provenance": {
                        "method": "human",
                        "matching-rationale": "semantic",
                        "status": "draft",
                        "mapping-description": "Human-reviewed relationship set."
                    },
                    "mappings": [{
                        "uuid": "22222222-2222-4222-8222-222222222222",
                        "source-resource": {"type": "catalog", "href": "source.json"},
                        "target-resource": {"type": "catalog", "href": "target.json"},
                        "maps": [{
                            "uuid": "33333333-3333-4333-8333-333333333333",
                            "relationship": "subset-of",
                            "sources": [{"type": "control", "id-ref": "source-1"}],
                            "targets": [{"type": "control", "id-ref": "target-1"}]
                        }]
                    }]
                }
            }"#,
        )
        .unwrap();

        let result = validate_artifact(&mapping_json, OscalModelType::Mapping).unwrap();
        assert!(result.is_valid, "Expected valid mapping, got errors: {:?}", result.errors);
        assert_eq!(result.declared_oscal_version.as_deref(), Some("1.2.3"));
        assert_eq!(result.schema_version_used, "1.2.3");
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

    // --- Profile variant tests (T002, WI-32) ---

    #[test]
    fn load_schema_profile() {
        let result = load_schema(OscalModelType::Profile);
        assert!(result.is_ok(), "Expected Ok from load_schema(Profile), got: {result:?}");
        let schema = result.unwrap();
        let s = serde_json::to_string(&schema).unwrap();
        assert!(!s.is_empty(), "Profile schema string must not be empty");
    }

    #[test]
    fn detect_model_type_profile() {
        let json = serde_json::json!({ "profile": {} });
        let result = detect_model_type(&json);
        assert!(
            result.is_ok(),
            "Expected Ok from detect_model_type for profile JSON, got: {result:?}"
        );
        assert_eq!(result.unwrap(), OscalModelType::Profile);
    }

    // --- Display tests ---

    #[test]
    fn oscal_model_type_display() {
        assert_eq!(OscalModelType::Catalog.to_string(), "catalog");
        assert_eq!(OscalModelType::ComponentDefinition.to_string(), "component-definition");
        assert_eq!(OscalModelType::Profile.to_string(), "profile");
        assert_eq!(OscalModelType::Mapping.to_string(), "mapping-collection");
    }

    #[test]
    fn detect_model_type_rejects_ambiguous_artifact() {
        let json: serde_json::Value = serde_json::json!({
            "catalog": {"uuid": "x", "metadata": {}},
            "component-definition": {"uuid": "y", "metadata": {}}
        });
        let result = detect_model_type(&json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidateError::AmbiguousArtifact { .. }));
    }

    #[test]
    fn detect_model_type_rejects_three_way_ambiguity() {
        let json: serde_json::Value = serde_json::json!({
            "catalog": {},
            "component-definition": {},
            "profile": {}
        });
        let result = detect_model_type(&json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ValidateError::AmbiguousArtifact { .. }));
        let msg = err.to_string();
        assert!(msg.contains("catalog"));
        assert!(msg.contains("component-definition"));
        assert!(msg.contains("profile"));
    }

    #[test]
    fn detect_model_type_rejects_catalog_and_mapping_ambiguity() {
        let json = serde_json::json!({
            "catalog": {},
            "mapping-collection": {}
        });
        let error = detect_model_type(&json).expect_err("multiple roots must be ambiguous");
        assert!(matches!(error, ValidateError::AmbiguousArtifact { .. }));
        let message = error.to_string();
        assert!(message.contains("catalog"));
        assert!(message.contains("mapping-collection"));
    }

    #[test]
    fn compiled_validator_is_cached_per_model() {
        let first = compiled_validator(OscalModelType::Mapping).expect("compile Mapping schema");
        let second = compiled_validator(OscalModelType::Mapping).expect("reuse Mapping schema");
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn validate_error_ambiguous_artifact_display() {
        let err = ValidateError::AmbiguousArtifact { detail: "catalog, profile".to_string() };
        assert!(err.to_string().contains("catalog, profile"));
        assert!(err.to_string().contains("Ambiguous"));
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
        assert_eq!(report.model_type(), "catalog");
        assert_eq!(report.declared_oscal_version(), Some("1.2.0"));
        assert_eq!(report.schema_version_used(), "1.2.3");
        assert!(report.supported_input());
    }

    #[test]
    fn full_validation_rejects_unsupported_declaration_with_baseline_context() {
        let json = serde_json::json!({
            "catalog": {
                "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
                "metadata": {
                    "title": "Unsupported Catalog",
                    "last-modified": "2026-01-01T00:00:00Z",
                    "version": "1.0",
                    "oscal-version": "1.3.0"
                }
            }
        });
        let report = run_full_validation("test.json", &json, OscalModelType::Catalog).unwrap();
        assert!(!report.is_valid());
        assert_eq!(report.declared_oscal_version(), Some("1.3.0"));
        assert_eq!(report.schema_version_used(), "1.2.3");
        assert!(!report.supported_input());
        let version_error = report
            .errors()
            .iter()
            .find(|error| error.path == "$.catalog.metadata.oscal-version")
            .expect("unsupported version must produce a metadata-path error");
        assert!(version_error.message.contains("1.3.0"));
        assert!(version_error.message.contains("1.2.3"));
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

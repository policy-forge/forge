// API Contract: 019-schema-validation
// This file defines the public interface of the validation module.
// Implementation must conform to these signatures.

use std::path::{Path, PathBuf};
use serde_json::Value;

// --- Types ---

/// Supported OSCAL model types for schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscalModelType {
    Catalog,
    ComponentDefinition,
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

// --- Errors ---

/// Errors from validation operations.
#[derive(Debug, thiserror::Error)]
pub enum ValidateError {
    #[error("Failed to read artifact file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Unable to detect OSCAL model type from JSON structure")]
    UnknownModelType,

    #[error("Schema compilation failed for {model_type}: {message}")]
    SchemaCompilation {
        model_type: String,
        message: String,
    },

    #[error("Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)")]
    FileTooLarge {
        size_mb: f64,
        limit_mb: u64,
    },
}

// --- Public Functions ---

/// Detect the OSCAL model type from a parsed JSON value.
/// Inspects top-level keys: "catalog" → Catalog, "component-definition" → ComponentDefinition.
///
/// # Errors
/// Returns `ValidateError::UnknownModelType` if no recognized top-level key is found.
pub fn detect_model_type(json: &Value) -> Result<OscalModelType, ValidateError>;

/// Load the embedded OSCAL JSON schema for a given model type.
/// Returns the schema as a `serde_json::Value`.
///
/// # Errors
/// Returns `ValidateError::SchemaCompilation` if the embedded schema cannot be parsed.
pub fn load_schema(model_type: OscalModelType) -> Result<Value, ValidateError>;

/// Validate a JSON value against the OSCAL schema for the given model type.
/// Collects all errors (does not stop at the first).
///
/// # Errors
/// Returns `ValidateError::SchemaCompilation` if the schema cannot be compiled.
pub fn validate_artifact(
    json: &Value,
    model_type: OscalModelType,
) -> Result<ValidationResult, ValidateError>;

// --- CLI ---

/// CLI enum for --schema-type override.
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SchemaType {
    Catalog,
    ComponentDefinition,
}

// --- ForgeError Extension ---

// Add to existing ForgeError enum:
// #[error("Schema validation failed: {0}")]
// SchemaValidation(String),

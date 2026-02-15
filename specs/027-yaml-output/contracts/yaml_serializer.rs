//! Interface contract for YAML serialization (WI-27).
//!
//! This file defines the public API contract for the YAML serializer module.
//! It is NOT production code — it defines the interface that implementation must satisfy.
//!
//! Location: `src/export/yaml.rs`
//! Traces to: AR 027-ar-yaml-output (Option 1: serde_yaml with existing derive macros)

use serde::Serialize;
use serde::de::DeserializeOwned;

// Assumes ForgeError is defined in src/error.rs (already exists)
use crate::error::ForgeError;

// ─── Public API ──────────────────────────────────────────────────────────

/// Serialize any serde-serializable OSCAL model to a YAML string.
///
/// Wraps `serde_yaml::to_string()` with FORGE-specific error handling.
/// Produces valid YAML 1.2 output.
///
/// # Arguments
/// * `model` - Reference to any type implementing `serde::Serialize`
///
/// # Errors
/// Returns `ForgeError::Serialization` if `serde_yaml::to_string()` fails.
///
/// # Traces
/// - PRD M-1: Catalog YAML serialization
/// - PRD M-2: Component Definition YAML serialization
/// - AR: "serde_yaml::to_string() exclusively"
/// - SEC-5: Must use serde_yaml::to_string() only (no custom formatting)
///
/// # Examples
/// ```
/// let catalog = build_catalog(&doc, None)?;
/// let yaml = serialize_to_yaml(&catalog)?;
/// assert!(yaml.starts_with("catalog:") || yaml.contains("catalog:"));
/// ```
pub fn serialize_to_yaml<T: Serialize>(model: &T) -> Result<String, ForgeError> {
    serde_yaml::to_string(model)
        .map_err(|e| ForgeError::Serialization(format!("YAML serialization failed: {e}")))
}

/// Deserialize a YAML string to any serde-deserializable type.
///
/// Used for semantic equivalence testing (PRD M-3) and future `forge export` (WI-29).
/// Wraps `serde_yaml::from_str()` with FORGE-specific error handling.
///
/// # Arguments
/// * `yaml` - A YAML string to deserialize
///
/// # Errors
/// Returns `ForgeError::Serialization` if `serde_yaml::from_str()` fails.
///
/// # Traces
/// - PRD M-3: Semantic equivalence verification
/// - SEC-4: Deserialization comparison for equivalence
///
/// # Examples
/// ```
/// let yaml = serialize_to_yaml(&catalog)?;
/// let value: serde_json::Value = deserialize_from_yaml(&yaml)?;
/// ```
pub fn deserialize_from_yaml<T: DeserializeOwned>(yaml: &str) -> Result<T, ForgeError> {
    serde_yaml::from_str(yaml)
        .map_err(|e| ForgeError::Serialization(format!("YAML deserialization failed: {e}")))
}

// ─── Pipeline Integration Contract ──────────────────────────────────────

// The following describes the expected changes to existing functions.
// These are NOT new functions — they are modifications to existing signatures.

// CURRENT (src/pipeline.rs):
//   pub fn run_catalog_pipeline(
//       input_path: &Path,
//       output_path: Option<&Path>,
//       max_size_bytes: u64,
//   ) -> Result<(), ForgeError>
//
// NEW (add format parameter):
//   pub fn run_catalog_pipeline(
//       input_path: &Path,
//       output_path: Option<&Path>,
//       max_size_bytes: u64,
//       format: &OutputFormat,
//   ) -> Result<(), ForgeError>
//
// BEHAVIOR CHANGE:
//   1. Replace: serde_json::to_string_pretty(&envelope) + serde_json::from_str(&json)
//      With: serde_json::to_value(&envelope) for validation
//   2. After validation passes, dispatch serialization:
//      - OutputFormat::Json  → serde_json::to_string_pretty(&envelope)
//      - OutputFormat::Yaml  → export::yaml::serialize_to_yaml(&envelope)
//      - OutputFormat::Xml   → return Err(ForgeError::Validation("XML not yet supported"))
//   3. Call write_output(content, output_path)

// CURRENT (src/cli/convert.rs):
//   Line 24: if !matches!(format, OutputFormat::Json) { return Err(...) }
//
// NEW:
//   Remove the non-JSON guard entirely (or change to reject only OutputFormat::Xml).
//   Pass format to pipeline functions.

// CURRENT (src/pipeline.rs):
//   pub fn write_output(json: &str, output_path: Option<&Path>) -> Result<(), ForgeError>
//
// NEW (rename parameter only — no behavior change):
//   pub fn write_output(content: &str, output_path: Option<&Path>) -> Result<(), ForgeError>

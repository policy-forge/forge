//! YAML serialization module for OSCAL models (WI-27).
//!
//! Provides format-agnostic serialization/deserialization using `serde_yaml`.
//! All OSCAL model structs already derive `Serialize`, so this module adds
//! zero model-level code — it wraps `serde_yaml::to_string()` and
//! `serde_yaml::from_str()` with `ForgeError::Serialization` error mapping.

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::ForgeError;

/// Serialize any serde-serializable OSCAL model to a YAML string.
///
/// Wraps `serde_yaml_ng::to_string()` (aliased as `serde_yaml`) with FORGE-specific error handling.
///
/// # Errors
/// Returns `ForgeError::Serialization` if `serde_yaml::to_string()` fails.
pub fn serialize_to_yaml<T: Serialize>(model: &T) -> Result<String, ForgeError> {
    tracing::debug!("Serializing model to YAML via serde_yaml::to_string");
    let result = serde_yaml::to_string(model)
        .map_err(|e| ForgeError::Serialization(format!("YAML serialization failed: {e}")));
    if result.is_ok() {
        tracing::debug!("YAML serialization succeeded");
    }
    result
}

/// Deserialize a YAML string to any serde-deserializable type.
///
/// Wraps `serde_yaml_ng::from_str()` (aliased as `serde_yaml`) with FORGE-specific error handling.
///
/// # Security
/// Callers MUST enforce a byte-size limit (normally [`crate::io::MAX_FILE_SIZE`])
/// and accept YAML only from an appropriate trust boundary before calling this
/// generic parser. Deeply nested or alias-heavy YAML can otherwise consume
/// excessive memory or stack.
///
/// # Errors
/// Returns `ForgeError::Serialization` if `serde_yaml::from_str()` fails.
pub fn deserialize_from_yaml<T: DeserializeOwned>(yaml: &str) -> Result<T, ForgeError> {
    serde_yaml::from_str(yaml)
        .map_err(|e| ForgeError::Serialization(format!("YAML deserialization failed: {e}")))
}

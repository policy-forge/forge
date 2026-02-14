// Validation Error Reporting API Contract (WI-20)
//
// This file defines the public API types and function signatures for the
// enhanced validation error reporting feature. These contracts MUST be
// defined before implementation begins (Constitution Principle III).
//
// Source: AR 020 — Interface Definitions section

use serde::{Deserialize, Serialize};
use serde_json::Value;

// Re-export existing type from WI-19
// use crate::validate::OscalModelType;

// ---------------------------------------------------------------------------
// Error Types (src/validate/error_types.rs)
// ---------------------------------------------------------------------------

/// Category of validation error (PRD M-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationErrorCategory {
    /// Error from JSON Schema validation.
    Schema,
    /// Error from semantic validation (orphaned links, missing references).
    Semantic,
}

/// A single validation error with full context for actionable reporting (PRD M-1).
///
/// Every error MUST include path + expected + actual (PRD M-1).
/// Actual values MUST be truncated to 100 characters (SEC-1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidationError {
    /// Category of error: Schema or Semantic.
    pub category: ValidationErrorCategory,
    /// JSON Path to the offending field (e.g., "$.catalog.metadata.uuid").
    /// Uses JSON Path notation, NOT JSON Pointer.
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

// ---------------------------------------------------------------------------
// Formatter Functions (src/validate/formatter.rs)
// ---------------------------------------------------------------------------

/// Convert a JSON Pointer (RFC 6901) to JSON Path notation.
///
/// # Examples
/// - `""` → `"$"`
/// - `"/catalog/metadata/uuid"` → `"$.catalog.metadata.uuid"`
/// - `"/catalog/groups/0/controls/2/id"` → `"$.catalog.groups[0].controls[2].id"`
///
/// Handles malformed pointers gracefully — no panics (SEC-6).
pub fn pointer_to_json_path(pointer: &str) -> String {
    todo!("WI-20: Implement JSON Pointer to JSON Path conversion")
}

/// Format a raw jsonschema crate error into an actionable ValidationError.
///
/// Transforms the raw error by:
/// 1. Converting `instance_path` from JSON Pointer to JSON Path
/// 2. Extracting the expected constraint from the error context
/// 3. Extracting the actual value from the JSON instance (truncated to 100 chars)
///
/// Raw crate messages are NEVER passed through to the output (SEC-2).
pub fn format_schema_error(
    raw_error: &jsonschema::ValidationError,
    json: &Value,
) -> ValidationError {
    todo!("WI-20: Implement schema error formatting")
}

/// Truncate a string to max_len characters, appending "..." if truncated.
///
/// SEC-1: Actual values in errors MUST be truncated to 100 characters.
pub fn truncate_value(value: &str, max_len: usize) -> String {
    todo!("WI-20: Implement value truncation")
}

/// Extract the actual value at a JSON Pointer path, serialized and truncated.
///
/// Navigates the JSON tree to the location specified by `pointer`,
/// serializes the value found there, and truncates via `truncate_value()`.
/// Returns `"(not found)"` if the path does not resolve.
///
/// This is an internal helper used by `format_schema_error()`.
/// Not part of the public API.
fn extract_actual_value(json: &Value, pointer: &str) -> String {
    todo!("WI-20: Implement actual value extraction")
}

// ---------------------------------------------------------------------------
// Semantic Validator (src/validate/semantic.rs)
// ---------------------------------------------------------------------------

/// Semantic validator for OSCAL artifacts (PRD M-3, M-4).
///
/// Detects logical inconsistencies beyond JSON Schema compliance:
/// - Orphaned back-matter links (PRD M-3)
/// - Missing required references (PRD M-4)
pub struct SemanticValidator;

impl SemanticValidator {
    /// Run all semantic validation checks on an OSCAL artifact.
    ///
    /// Returns a list of semantic `ValidationError`s.
    /// Does NOT follow external URLs (SEC-5).
    pub fn validate(
        &self,
        json: &Value,
        model_type: OscalModelType,
    ) -> Vec<ValidationError> {
        todo!("WI-20: Implement semantic validation orchestrator")
    }

    /// Check for orphaned back-matter links (PRD M-3).
    ///
    /// Finds `href` values starting with `#` that reference UUIDs
    /// not present in `back-matter.resources[].uuid`.
    fn check_orphaned_links(&self, json: &Value) -> Vec<ValidationError> {
        todo!("WI-20: Implement orphaned link detection")
    }

    /// Check for missing required references (PRD M-4).
    ///
    /// For Component Definitions: verify control-id references in
    /// implemented-requirements are non-empty and well-formed.
    fn check_missing_references(
        &self,
        json: &Value,
        model_type: OscalModelType,
    ) -> Vec<ValidationError> {
        todo!("WI-20: Implement missing reference detection")
    }
}

// ---------------------------------------------------------------------------
// Report Renderers (src/validate/report.rs)
// ---------------------------------------------------------------------------

/// Render a ValidationReport as human-readable text (PRD S-2).
///
/// Output format:
/// ```text
/// Validation failed: 3 schema errors, 1 semantic error
///
/// Schema Errors:
///   [1] $.catalog.metadata.uuid — required field missing
///       Expected: required string field
///       Actual: field not present
/// ...
/// ```
pub fn render_text_report(report: &ValidationReport) -> String {
    todo!("WI-20: Implement text report renderer")
}

/// Render a ValidationReport as JSON (PRD S-1).
///
/// Uses serde_json serialization of the ValidationReport struct.
/// Output MUST contain only ValidationReport/ValidationError fields (SEC-3).
pub fn render_json_report(report: &ValidationReport) -> String {
    todo!("WI-20: Implement JSON report renderer")
}

// ---------------------------------------------------------------------------
// Validation Orchestrator (src/validate/mod.rs — enhanced)
// ---------------------------------------------------------------------------

/// Run full validation (schema + semantic) and produce a ValidationReport.
///
/// This is the main entry point for enhanced validation (WI-20).
/// Calls `validate_artifact()` (WI-19) for schema validation,
/// then `SemanticValidator` for semantic validation,
/// then combines results into a single report.
///
/// MUST collect ALL errors from both passes (PRD M-2).
/// MUST NOT stop at the first error.
pub fn run_full_validation(
    artifact_path: &str,
    json: &Value,
    model_type: OscalModelType,
) -> Result<ValidationReport, ValidateError> {
    todo!("WI-20: Implement full validation orchestrator")
}

// Placeholder for existing types referenced above
type OscalModelType = ();
type ValidateError = ();

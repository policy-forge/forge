//! Public API contract for `src/parameter/` module (WI-34).
//!
//! This file documents the complete public interface of the `parameter` module.
//! All function signatures, return types, and error conditions are specified
//! before implementation begins (Contract-First per constitution III).
//!
//! # Module Location
//! `src/parameter/mod.rs` (public API) + `src/parameter/matchers.rs` (internal)
//!
//! # Exposed via `src/lib.rs`
//! `pub mod parameter;`

use crate::error::ForgeError;
use crate::model::{PolicyDocument, PolicyParameter};
use crate::oscal::catalog::OscalParam;

// ─── Primary Public API ────────────────────────────────────────────────────

/// Document-level enrichment pass: extract parameters from all requirements.
///
/// Iterates over every `PolicyRequirement` in the document. For each requirement
/// that has a `stable_id`, calls `extract_parameters_from_text` and:
/// - Updates `requirement.text` with OSCAL insertion placeholders
/// - Populates `requirement.parameters` with extracted `PolicyParameter` objects
///
/// Requirements without a `stable_id` are skipped (cannot generate param ID).
/// Requirements with an empty `text` are preserved as-is.
///
/// # Idempotence
/// Running this function twice on the same document produces identical results.
/// OSCAL insertion placeholders (`{{ insert: param, id-ref: ... }}`) do not
/// match any parameter regex pattern — double extraction is a no-op.
///
/// # Arguments
/// * `document` — Mutable reference to the `PolicyDocument` to enrich in-place
///
/// # Returns
/// * `Ok(())` on success (including documents with zero requirements or no parameters)
/// * `Err(ForgeError::ParameterExtraction)` if extraction fails for a specific requirement
///
/// # Errors
/// Only errors if regex compilation fails (panic — static patterns) or a
/// replacement operation produces invalid UTF-8 (should never occur with Rust `String`).
pub fn extract_parameters(document: &mut PolicyDocument) -> Result<(), ForgeError> {
    todo!("Implement in src/parameter/mod.rs")
}

/// Extract parameters from a single requirement's text.
///
/// Runs all four matchers (time window, threshold, frequency, quantity) against
/// the text, resolves overlapping matches (first-match-wins, by start position),
/// replaces matched spans in reverse order with OSCAL insertion placeholders,
/// and assigns deterministic parameter IDs.
///
/// # Arguments
/// * `requirement_id` — The `stable_id` of the source requirement (used for ID generation)
/// * `text` — The requirement text to extract parameters from
///
/// # Returns
/// A tuple of:
/// * `String` — The updated text with matched spans replaced by insertion placeholders
/// * `Vec<PolicyParameter>` — Extracted parameters in position order (lowest start offset first)
///
/// # Errors
/// Returns `Err` only in exceptional circumstances (should not occur with valid UTF-8 input).
///
/// # Examples
/// ```
/// let (updated_text, params) = extract_parameters_from_text(
///     "POL-AC-001",
///     "Passwords must be changed within 30 days of compromise",
/// )?;
/// assert!(updated_text.contains("{{ insert: param, id-ref: POL-AC-001_prm_0 }}"));
/// assert_eq!(params.len(), 1);
/// assert_eq!(params[0].value, "30 days");
/// ```
pub fn extract_parameters_from_text(
    requirement_id: &str,
    text: &str,
) -> Result<(String, Vec<PolicyParameter>), ForgeError> {
    todo!("Implement in src/parameter/mod.rs")
}

/// Generate a deterministic parameter ID from requirement ID and position.
///
/// Format: `"{requirement_id}_prm_{position}"`
///
/// # Arguments
/// * `requirement_id` — The `stable_id` of the source requirement
/// * `position` — 0-based index of this parameter within the requirement
///
/// # Examples
/// ```
/// assert_eq!(parameter_id("POL-AC-001", 0), "POL-AC-001_prm_0");
/// assert_eq!(parameter_id("POL-AC-001", 1), "POL-AC-001_prm_1");
/// ```
pub fn parameter_id(requirement_id: &str, position: usize) -> String {
    // Note: value parameter removed vs AR spec — position alone is sufficient
    // for deterministic IDs within a requirement. Content-hash would add complexity
    // without benefit at this scale.
    format!("{requirement_id}_prm_{position}")
}

/// Convert a `PolicyParameter` to an OSCAL `param` element.
///
/// # Mapping
/// - `PolicyParameter.id` → `OscalParam.id`
/// - `PolicyParameter.label` → `OscalParam.label`
/// - `PolicyParameter.value` → `OscalParam.values[0]`
/// - `PolicyParameter.constraint` → `OscalParam.constraints[0].description` (format: `"minimum: 30 days"`)
///
/// # Arguments
/// * `parameter` — The `PolicyParameter` to convert
///
/// # Returns
/// An `OscalParam` ready for embedding in an `OscalControl.params` array.
pub fn to_oscal_param(parameter: &PolicyParameter) -> OscalParam {
    todo!("Implement in src/parameter/mod.rs")
}

// ─── Internal Types (src/parameter/matchers.rs) ───────────────────────────

/// Internal matcher result (NOT pub — crate-private).
///
/// Produced by `ParameterMatcher::find_parameters()`.
/// Consumed by `extract_parameters_from_text()`.
pub(crate) struct ParameterMatch {
    pub start: usize,
    pub end: usize,
    pub matched_text: String,
    pub value: String,
    pub parameter_type: crate::model::ParameterType,
    pub label: String,
    pub constraint: Option<crate::model::ParameterConstraint>,
}

/// Common interface for type-specific parameter matchers (NOT pub — crate-private).
pub(crate) trait ParameterMatcher {
    fn find_parameters(&self, text: &str) -> Vec<ParameterMatch>;
}

// ─── Error Variant (src/error.rs addition) ────────────────────────────────

/// DIFF: Add `ParameterExtraction` variant to `ForgeError`.
///
/// ```
/// #[error("Parameter extraction error: {0}")]
/// ParameterExtraction(String),
/// ```
///
/// Exit code: 2 (parse/transform error, consistent with `CatalogBuild`, `BackMatter`).
/// Add to `exit_code()` match arm:
/// ```
/// ForgeError::ParameterExtraction(_) => 2,
/// ```
pub struct _ForgeErrorDiff;

// ─── OSCAL Types (src/oscal/catalog.rs addition) ──────────────────────────

/// DIFF: New types and field in `src/oscal/catalog.rs`.
///
/// ```rust
/// /// OSCAL param element within a catalog control (WI-34).
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct OscalParam {
///     pub id: String,
///     pub label: String,
///     #[serde(default, skip_serializing_if = "Vec::is_empty")]
///     pub values: Vec<String>,
///     #[serde(default, skip_serializing_if = "Vec::is_empty")]
///     pub constraints: Vec<OscalParamConstraint>,
/// }
///
/// /// OSCAL param.constraint element.
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct OscalParamConstraint {
///     pub description: String,
/// }
///
/// // In OscalControl, add before `parts`:
/// #[serde(default, skip_serializing_if = "Vec::is_empty")]
/// pub params: Vec<OscalParam>,
/// ```
///
/// OSCAL v1.2.0 schema places `params` before `parts` in control object.
/// Verify serialization order matches schema after implementation.
pub struct _OscalCatalogDiff;

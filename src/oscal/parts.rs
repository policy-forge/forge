//! OSCAL Part and Prop builders for control content and metadata.
//!
//! Provides [`OscalPart`] and [`OscalProp`] structs plus composable builder
//! functions that generate statement parts, guidance parts, and structured
//! metadata props from the domain model.

use serde::Serialize;
use tracing::{debug, warn};

use crate::model::PolicyRequirement;

// ─── OSCAL Structs ──────────────────────────────────────────────────────

/// An OSCAL control part (statement, guidance, objective).
///
/// Parts carry the actual content of a control. Every control has at least
/// one statement part; guidance and objective parts are optional.
///
/// # OSCAL v1.2.0
///
/// Serializes to the OSCAL JSON `parts` array structure:
/// ```json
/// { "id": "POL-AC-001_smt", "name": "statement", "prose": "..." }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OscalPart {
    /// Part ID following `{control-id}_{suffix}` convention.
    /// Example: `"POL-AC-001_smt"`, `"POL-AC-001_gdn"`.
    pub id: String,

    /// OSCAL part name: `"statement"`, `"guidance"`, `"objective"`, or `"item"`.
    pub name: String,

    /// Human-readable text content. Direct copy from source (SEC-1).
    pub prose: String,

    /// Nested sub-parts (e.g., enumerated sub-items within a statement).
    /// Omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<OscalPart>,

    /// Properties on this part.
    /// Omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
}

/// An OSCAL property for structured metadata on controls or parts.
///
/// # OSCAL v1.2.0
///
/// Serializes to the OSCAL JSON `props` array structure:
/// ```json
/// { "name": "source-file", "ns": "https://forge.policy-forge.github.io/ns/trace", "value": "policy.md" }
/// ```
///
/// The `ns` field is optional; when `None` it is omitted from JSON output.
/// FORGE trace props always set `ns` to [`crate::oscal::trace_embedding::FORGE_TRACE_NS`].
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OscalProp {
    /// Property name (e.g., `"source-file"`, `"source-section"`, `"source-line"`).
    pub name: String,

    /// Optional namespace URI. FORGE trace props use `FORGE_TRACE_NS`.
    /// Omitted from JSON when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,

    /// Property value as string.
    pub value: String,
}

// ─── Builder Functions ──────────────────────────────────────────────────

/// Generate a part ID from a control ID and a suffix.
///
/// Convention: `{control-id}_{suffix}`
///
/// # Examples
///
/// ```
/// use forge::oscal::parts::generate_part_id;
///
/// assert_eq!(generate_part_id("POL-AC-001", "smt"), "POL-AC-001_smt");
/// assert_eq!(generate_part_id("POL-DP-003", "gdn"), "POL-DP-003_gdn");
/// ```
#[must_use]
pub fn generate_part_id(control_id: &str, suffix: &str) -> String {
    format!("{control_id}_{suffix}")
}

/// Generate statement parts (and optionally guidance parts) for a control.
///
/// Always produces at least one part with `name: "statement"` and `prose`
/// from `requirement.text` (SEC-1: direct copy, no transformation).
///
/// When `guidance_text` is `Some(non_empty_text)`, also produces a guidance
/// part with `name: "guidance"` and prose from the guidance text.
///
/// Logs `tracing::warn` if `requirement.text` is empty or whitespace-only (EC-1, EC-2).
///
/// # Arguments
///
/// * `control_id` — The control's ID (e.g., `"POL-AC-001"`)
/// * `requirement` — The source `PolicyRequirement`
/// * `guidance_text` — Optional guidance text from `PolicySection.body_text`
///
/// # Examples
///
/// ```
/// use forge::oscal::parts::build_control_parts;
/// use forge::model::PolicyRequirement;
///
/// let req = PolicyRequirement {
///     text: "All users must use MFA.".to_string(),
///     source_line: 42,
///     stable_id: Some("uuid-1".to_string()),
///     nesting_depth: 0,
///     atom_index: 0,
///     parent_text: None,
///     citations: vec![],
/// };
///
/// let parts = build_control_parts("POL-AC-001", &req, None);
/// assert_eq!(parts.len(), 1);
/// assert_eq!(parts[0].name, "statement");
/// assert_eq!(parts[0].prose, "All users must use MFA.");
/// assert_eq!(parts[0].id, "POL-AC-001_smt");
/// ```
#[must_use]
pub fn build_control_parts(
    control_id: &str,
    requirement: &PolicyRequirement,
    guidance_text: Option<&str>,
) -> Vec<OscalPart> {
    let mut parts = Vec::with_capacity(2);

    // Warn on empty or whitespace-only requirement text (EC-1, EC-2, SEC-2).
    if requirement.text.trim().is_empty() {
        warn!(control_id, "Empty/whitespace requirement text — statement prose preserved as-is");
    }

    // Statement part — always present (FR-001).
    // Prose is a direct copy of requirement.text (SEC-1) — no transformation.
    parts.push(OscalPart {
        id: generate_part_id(control_id, "smt"),
        name: "statement".to_string(),
        prose: requirement.text.clone(),
        parts: vec![],
        props: vec![],
    });

    // Guidance part — only when guidance_text is Some and non-empty.
    if let Some(text) = guidance_text
        && !text.is_empty()
    {
        parts.push(OscalPart {
            id: generate_part_id(control_id, "gdn"),
            name: "guidance".to_string(),
            prose: text.to_string(),
            parts: vec![],
            props: vec![],
        });
    }

    debug!(control_id, part_count = parts.len(), "Built control parts");

    parts
}

/// Generate props for a control from a `PolicyRequirement`.
///
/// Returns an empty vec — trace props (source-file, source-section, source-line)
/// are now added by [`crate::oscal::trace_embedding::embed_trace_in_catalog`]
/// post-processing instead of inline during catalog construction.
///
/// Retained for API compatibility with existing callers.
#[must_use]
pub fn build_control_props(_requirement: &PolicyRequirement) -> Vec<OscalProp> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PolicyRequirement;

    fn test_req(text: &str, source_line: usize) -> PolicyRequirement {
        PolicyRequirement {
            text: text.to_string(),
            source_line,
            stable_id: Some("test-uuid".to_string()),
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
        }
    }

    // ── T006: generate_part_id tests ──────────────────────────────────

    #[test]
    fn test_generate_part_id_statement() {
        assert_eq!(generate_part_id("POL-AC-001", "smt"), "POL-AC-001_smt");
    }

    #[test]
    fn test_generate_part_id_guidance() {
        assert_eq!(generate_part_id("POL-AC-001", "gdn"), "POL-AC-001_gdn");
    }

    #[test]
    fn test_generate_part_id_objective() {
        assert_eq!(generate_part_id("POL-AC-001", "obj"), "POL-AC-001_obj");
    }

    #[test]
    fn test_generate_part_id_special_chars() {
        assert_eq!(generate_part_id("POL-DP&P-001", "smt"), "POL-DP&P-001_smt");
    }

    // ── T007: build_control_parts (statement-only, no guidance) ───────

    #[test]
    fn test_build_parts_statement_only() {
        let req = test_req("All users must use MFA.", 10);
        let parts = build_control_parts("POL-AC-001", &req, None);

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "statement");
        assert_eq!(parts[0].id, "POL-AC-001_smt");
        assert_eq!(parts[0].prose, "All users must use MFA.");
        assert!(parts[0].parts.is_empty());
        assert!(parts[0].props.is_empty());
    }

    #[test]
    fn test_build_parts_prose_exact_copy() {
        let original = "All users must use MFA.";
        let req = test_req(original, 10);
        let parts = build_control_parts("POL-AC-001", &req, None);

        assert_eq!(parts[0].prose, original);
        assert_eq!(parts[0].prose.as_bytes(), original.as_bytes());
    }

    // ── build_control_props tests (returns empty — trace props handled by embed_trace_in_catalog) ──

    #[test]
    fn test_build_props_returns_empty_for_positive_line() {
        let req = test_req("text", 42);
        let props = build_control_props(&req);
        assert!(
            props.is_empty(),
            "build_control_props now returns empty — trace props added by post-processing"
        );
    }

    #[test]
    fn test_build_props_returns_empty_for_zero_line() {
        let req = test_req("text", 0);
        let props = build_control_props(&req);
        assert!(props.is_empty());
    }

    // ── T016: build_control_parts with guidance ───────────────────────

    #[test]
    fn test_build_parts_with_guidance() {
        let req = test_req("Statement text.", 10);
        let parts = build_control_parts("POL-AC-001", &req, Some("Guidance text."));

        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].name, "statement");
        assert_eq!(parts[0].id, "POL-AC-001_smt");
        assert_eq!(parts[1].name, "guidance");
        assert_eq!(parts[1].id, "POL-AC-001_gdn");
        assert_eq!(parts[1].prose, "Guidance text.");
    }

    #[test]
    fn test_build_parts_guidance_order() {
        let req = test_req("Statement.", 10);
        let parts = build_control_parts("POL-AC-001", &req, Some("Guidance."));

        assert_eq!(parts[0].name, "statement");
        assert_eq!(parts[1].name, "guidance");
    }

    // ── T017: build_control_parts with empty guidance ─────────────────

    #[test]
    fn test_build_parts_empty_guidance_string() {
        let req = test_req("Statement.", 10);
        let parts = build_control_parts("POL-AC-001", &req, Some(""));

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "statement");
    }

    // ── T021: multi-paragraph prose preservation ──────────────────────

    #[test]
    fn test_build_parts_multi_paragraph_prose() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let req = test_req(text, 10);
        let parts = build_control_parts("POL-AC-001", &req, None);

        assert_eq!(parts[0].prose, text);
        assert!(parts[0].prose.contains("\n\n"));
    }

    // ── T023: edge case tests ─────────────────────────────────────────

    #[test]
    fn test_build_parts_empty_text() {
        let req = test_req("", 10);
        let parts = build_control_parts("POL-AC-001", &req, None);

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "statement");
        assert_eq!(parts[0].prose, "");
    }

    #[test]
    fn test_build_parts_whitespace_only_text() {
        let req = test_req("   \t\n  ", 10);
        let parts = build_control_parts("POL-AC-001", &req, None);

        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].prose, "   \t\n  ");
    }

    #[test]
    fn test_build_parts_markdown_formatting_preserved() {
        let text = "Must use **bold** and [link](url) and `code` formatting.";
        let req = test_req(text, 10);
        let parts = build_control_parts("POL-AC-001", &req, None);

        assert_eq!(parts[0].prose, text);
        assert!(parts[0].prose.contains("**bold**"));
        assert!(parts[0].prose.contains("[link](url)"));
        assert!(parts[0].prose.contains("`code`"));
    }
}

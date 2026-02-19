//! Parameter extraction pipeline pass (WI-34).
//!
//! Detects time windows, thresholds, frequencies, and quantities embedded in
//! `PolicyRequirement` text, extracts them into `PolicyParameter` objects with
//! value domains and deterministic IDs, replaces matched spans with OSCAL
//! insertion placeholders, and emits OSCAL `param` elements within the
//! corresponding catalog controls.
//!
//! # Security properties
//! - SEC-1: Parameter values are logged at DEBUG level only, never INFO.
//! - SEC-2: All matchers require qualifier words to prevent false positives.
//! - SEC-3: Bounded quantifiers only; no nested repetition.
//! - SEC-4: `LazyLock` ensures one-time regex compilation.
//! - SEC-5: Span replacement is done in reverse-order to preserve byte offsets.
//! - SEC-6: Extraction is idempotent; OSCAL placeholders do not re-match.

pub(crate) mod matchers;

use crate::error::ForgeError;
use crate::model::{ConstraintType, PolicyDocument, PolicyParameter};
#[cfg(test)]
use crate::model::{ParameterConstraint, ParameterType};
use crate::oscal::catalog::{OscalParam, OscalParamConstraint};
use crate::parameter::matchers::{
    FrequencyMatcher, ParameterMatcher, QuantityMatcher, ThresholdMatcher, TimeWindowMatcher,
};

// ─── Public API ─────────────────────────────────────────────────────────────

/// Generate a deterministic parameter ID from requirement ID and position.
///
/// Format: `"{requirement_id}_prm_{position}"`
///
/// # Examples
/// ```
/// assert_eq!(forge::parameter::parameter_id("POL-AC-001", 0), "POL-AC-001_prm_0");
/// assert_eq!(forge::parameter::parameter_id("POL-AC-001", 1), "POL-AC-001_prm_1");
/// ```
#[must_use]
pub fn parameter_id(requirement_id: &str, position: usize) -> String {
    format!("{requirement_id}_prm_{position}")
}

/// Extract parameters from a single requirement's text.
///
/// Runs all four matchers against the text, resolves overlapping matches
/// (first-match-wins by start position; ties broken by longer match),
/// replaces matched spans in reverse order with OSCAL insertion placeholders,
/// and assigns deterministic IDs.
///
/// # Arguments
/// * `requirement_id` — The `stable_id` of the source requirement (for ID generation)
/// * `text` — The requirement text to extract parameters from
///
/// # Returns
/// A tuple: `(updated_text, parameters)` where `updated_text` has matched spans
/// replaced by `{{ insert: param, id-ref: <id> }}` placeholders.
///
/// (Note: `<id>` in the placeholder represents the actual parameter ID string.)
///
/// # Errors
/// Returns `Err` only in exceptional circumstances (never occurs with valid UTF-8 input).
pub fn extract_parameters_from_text(
    requirement_id: &str,
    text: &str,
) -> Result<(String, Vec<PolicyParameter>), ForgeError> {
    // Collect matches from all four matchers
    let matchers: Vec<Box<dyn ParameterMatcher>> = vec![
        Box::new(TimeWindowMatcher),
        Box::new(ThresholdMatcher),
        Box::new(FrequencyMatcher),
        Box::new(QuantityMatcher),
    ];

    let mut all_matches: Vec<matchers::ParameterMatch> =
        matchers.iter().flat_map(|m| m.find_parameters(text)).collect();

    // Sort by start position ascending; ties broken by longer match (larger end)
    all_matches.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));

    // Resolve overlaps: first-match-wins (by start position)
    let mut resolved: Vec<matchers::ParameterMatch> = Vec::new();
    let mut last_end: usize = 0;
    for m in all_matches {
        if m.start >= last_end {
            last_end = m.end;
            resolved.push(m);
        }
        // Overlapping match is discarded (first-match-wins)
    }

    // Build parameters with deterministic IDs
    let parameters: Vec<PolicyParameter> = resolved
        .iter()
        .enumerate()
        .map(|(pos, m)| {
            let id = parameter_id(requirement_id, pos);
            tracing::debug!(
                requirement_id,
                pos,
                parameter_type = ?m.parameter_type,
                "Extracted parameter"
            );
            PolicyParameter {
                id: id.clone(),
                requirement_id: requirement_id.to_string(),
                label: m.label.clone(),
                value: m.value.clone(),
                parameter_type: m.parameter_type.clone(),
                constraint: m.constraint.clone(),
            }
        })
        .collect();

    // Replace spans in reverse order to preserve byte offsets (SEC-5)
    let mut updated_text = text.to_string();
    for (pos, m) in resolved.iter().enumerate().rev() {
        let placeholder =
            format!("{{{{ insert: param, id-ref: {} }}}}", parameter_id(requirement_id, pos));
        updated_text.replace_range(m.start..m.end, &placeholder);
    }

    Ok((updated_text, parameters))
}

/// Document-level enrichment pass: extract parameters from all requirements.
///
/// Iterates over every `PolicyRequirement` in the document. For each requirement
/// that has a `stable_id`, calls `extract_parameters_from_text` and:
/// - Updates `requirement.text` with OSCAL insertion placeholders
/// - Populates `requirement.parameters` with extracted `PolicyParameter` objects
///
/// Requirements without a `stable_id` are skipped (cannot generate param ID).
/// Requirements with empty text are preserved as-is.
///
/// # Idempotence
/// Running this function twice on the same document produces identical results.
/// OSCAL insertion placeholders do not match any parameter pattern (SEC-6).
///
/// # Errors
/// Returns `Err(ForgeError::ParameterExtraction)` if extraction fails for a requirement.
pub fn extract_parameters(document: &mut PolicyDocument) -> Result<(), ForgeError> {
    let mut total_params: usize = 0;

    for section in &mut document.sections {
        extract_section_parameters(section, &mut total_params)?;
    }

    tracing::info!(param_count = total_params, "Parameter extraction complete");

    Ok(())
}

/// Convert a `PolicyParameter` to an OSCAL `param` element.
///
/// # Mapping
/// - `PolicyParameter.id` → `OscalParam.id`
/// - `PolicyParameter.label` → `OscalParam.label`
/// - `PolicyParameter.value` → `OscalParam.values[0]`
/// - `PolicyParameter.constraint` → `OscalParam.constraints[0].description`
///   (format: `"minimum: 30 days"`, `"maximum: 15 minutes"`, `"exact: annually"`)
pub fn to_oscal_param(parameter: &PolicyParameter) -> OscalParam {
    let values = vec![parameter.value.clone()];
    let constraints = parameter.constraint.as_ref().map_or_else(Vec::new, |c| {
        let constraint_label = constraint_type_label(&c.constraint_type);
        vec![OscalParamConstraint { description: format!("{constraint_label}: {}", c.value) }]
    });

    OscalParam { id: parameter.id.clone(), label: parameter.label.clone(), values, constraints }
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Recursively extract parameters from all requirements in a section and children.
fn extract_section_parameters(
    section: &mut crate::model::PolicySection,
    total_params: &mut usize,
) -> Result<(), ForgeError> {
    for req in &mut section.requirements {
        let Some(stable_id) = req.stable_id.clone() else {
            // Skip requirements without a stable_id (cannot generate param IDs)
            continue;
        };

        if req.text.is_empty() {
            continue;
        }

        // Idempotence: if parameters were already extracted, skip (SEC-6).
        // OSCAL placeholders in `req.text` won't match any pattern, so a second
        // extraction would produce zero params — we preserve the original result.
        if !req.parameters.is_empty() {
            *total_params += req.parameters.len();
            continue;
        }

        // OSCAL TokenDatatype requires IDs to start with a letter or underscore.
        // UUID stable_ids may start with a hex digit; prefix with 'p' if needed.
        let requirement_id = if stable_id.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            stable_id.clone()
        } else {
            format!("p{stable_id}")
        };

        let (updated_text, mut params) = extract_parameters_from_text(&requirement_id, &req.text)
            .map_err(|e| {
            ForgeError::ParameterExtraction(format!(
                "Failed to extract parameters from requirement '{stable_id}': {e}"
            ))
        })?;

        // PolicyParameter.requirement_id must reference the original stable_id for
        // traceability. The OSCAL-safe prefixed id is only for parameter ID generation.
        if requirement_id != stable_id {
            for param in &mut params {
                param.requirement_id.clone_from(&stable_id);
            }
        }

        let param_count = params.len();
        *total_params += param_count;

        if param_count > 0 {
            tracing::debug!(
                requirement_id = %stable_id,
                param_count,
                "Parameters extracted from requirement"
            );
        }

        req.text = updated_text;
        req.parameters = params;
    }

    for child in &mut section.children {
        extract_section_parameters(child, total_params)?;
    }

    Ok(())
}

/// Map a `ConstraintType` to its lowercase string label for OSCAL descriptions.
fn constraint_type_label(ct: &ConstraintType) -> &'static str {
    match ct {
        ConstraintType::Minimum => "minimum",
        ConstraintType::Maximum => "maximum",
        ConstraintType::Exact => "exact",
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::model::{DocumentMetadata, PolicySection};

    // ── T022: parameter_id() ─────────────────────────────────────────────

    #[test]
    fn parameter_id_position_zero() {
        assert_eq!(parameter_id("POL-AC-001", 0), "POL-AC-001_prm_0");
    }

    #[test]
    fn parameter_id_position_one() {
        assert_eq!(parameter_id("POL-AC-001", 1), "POL-AC-001_prm_1");
    }

    #[test]
    fn parameter_id_position_ten() {
        assert_eq!(parameter_id("POL-AC-001", 10), "POL-AC-001_prm_10");
    }

    #[test]
    fn parameter_id_different_requirement_ids() {
        assert_eq!(parameter_id("POL-DP-002", 0), "POL-DP-002_prm_0");
        assert_eq!(parameter_id("POL-IR-042", 3), "POL-IR-042_prm_3");
    }

    // ── T023: extract_parameters_from_text() ─────────────────────────────

    #[test]
    fn extract_single_time_window_produces_placeholder() {
        let (text, params) =
            extract_parameters_from_text("POL-AC-001", "Passwords must change within 30 days")
                .unwrap();
        assert_eq!(params.len(), 1);
        assert!(
            text.contains("{{ insert: param, id-ref: POL-AC-001_prm_0 }}"),
            "Expected OSCAL placeholder in: {text}"
        );
        assert!(!text.contains("within 30 days"), "Matched span should be replaced");
        assert_eq!(params[0].id, "POL-AC-001_prm_0");
        assert_eq!(params[0].value, "30 days");
    }

    #[test]
    fn extract_multi_parameter_requirement_in_order() {
        let text = "Change passwords within 30 days and audit at least annually";
        let (updated, params) = extract_parameters_from_text("POL-AC-001", text).unwrap();
        assert_eq!(params.len(), 2, "Expected 2 params, got: {params:?}");
        // First param (lower start offset)
        assert_eq!(params[0].id, "POL-AC-001_prm_0");
        assert_eq!(params[0].parameter_type, ParameterType::TimeWindow);
        // Second param
        assert_eq!(params[1].id, "POL-AC-001_prm_1");
        assert_eq!(params[1].parameter_type, ParameterType::Frequency);
        assert!(updated.contains("{{ insert: param, id-ref: POL-AC-001_prm_0 }}"));
        assert!(updated.contains("{{ insert: param, id-ref: POL-AC-001_prm_1 }}"));
    }

    #[test]
    fn extract_overlap_quantity_wins_over_threshold() {
        // "no fewer than 3 authentication factors" — QuantityMatcher wins (longer match)
        let (_, params) = extract_parameters_from_text(
            "POL-MFA-001",
            "MFA must require no fewer than 3 authentication factors",
        )
        .unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].parameter_type, ParameterType::Quantity);
        assert_eq!(params[0].value, "3");
        assert_eq!(params[0].constraint.as_ref().unwrap().constraint_type, ConstraintType::Minimum);
    }

    #[test]
    fn extract_no_fewer_than_without_unit_threshold_wins() {
        // "no fewer than 3" without unit noun — ThresholdMatcher fires
        let (_, params) =
            extract_parameters_from_text("POL-AC-001", "allow no fewer than 3").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].parameter_type, ParameterType::Threshold);
    }

    #[test]
    fn extract_placeholder_format_correct() {
        let (text, _) =
            extract_parameters_from_text("POL-AC-001", "reviews within 30 days").unwrap();
        // Verify double-brace OSCAL placeholder format (non-negotiable per OSCAL markup spec)
        assert!(text.contains("{{ insert: param, id-ref: POL-AC-001_prm_0 }}"));
    }

    #[test]
    fn extract_no_params_text_unchanged() {
        let original = "Users must authenticate before accessing the system";
        let (text, params) = extract_parameters_from_text("POL-AC-001", original).unwrap();
        assert_eq!(params.len(), 0);
        assert_eq!(text, original, "Text should be unchanged when no params found");
    }

    #[test]
    fn extract_empty_text_no_error() {
        let (text, params) = extract_parameters_from_text("POL-AC-001", "").unwrap();
        assert!(params.is_empty());
        assert_eq!(text, "");
    }

    // ── T025: extract_parameters() document-level ────────────────────────

    #[test]
    fn document_extract_enriches_requirements() {
        let mut doc = make_doc(vec![
            make_req("POL-AC-001", "Change passwords within 30 days"),
            make_req("POL-AC-002", "Audit at least annually"),
            make_req("POL-AC-003", "Users must authenticate"),
        ]);

        extract_parameters(&mut doc).unwrap();

        let reqs = &doc.sections[0].requirements;
        assert_eq!(reqs[0].parameters.len(), 1, "Req 0 should have 1 param");
        assert_eq!(reqs[1].parameters.len(), 1, "Req 1 should have 1 param");
        assert_eq!(reqs[2].parameters.len(), 0, "Req 2 should have 0 params (no pattern)");
        assert!(reqs[0].text.contains("{{ insert: param, id-ref:"));
    }

    #[test]
    fn document_extract_skips_requirements_without_stable_id() {
        let mut doc = PolicyDocument {
            id: "test".to_string(),
            metadata: test_metadata(),
            sections: vec![PolicySection {
                title: "Test".to_string(),
                heading_level: 1,
                source_line: 1,
                body_text: None,
                children: vec![],
                requirements: vec![
                    // No stable_id — should be skipped
                    crate::model::PolicyRequirement {
                        stable_id: None,
                        text: "within 30 days".to_string(),
                        source_line: 1,
                        nesting_depth: 0,
                        atom_index: 0,
                        parent_text: None,
                        citations: vec![],
                        modality: None,
                        parameters: vec![],
                    },
                ],
            }],
        };

        extract_parameters(&mut doc).unwrap();
        assert!(doc.sections[0].requirements[0].parameters.is_empty());
        assert_eq!(doc.sections[0].requirements[0].text, "within 30 days");
    }

    #[test]
    fn document_extract_empty_document_no_error() {
        let mut doc =
            PolicyDocument { id: "empty".to_string(), metadata: test_metadata(), sections: vec![] };
        assert!(extract_parameters(&mut doc).is_ok());
    }

    // ── T026: to_oscal_param() ───────────────────────────────────────────

    #[test]
    fn to_oscal_param_time_window() {
        let param = PolicyParameter {
            id: "POL-AC-001_prm_0".to_string(),
            requirement_id: "POL-AC-001".to_string(),
            label: "within 30 days".to_string(),
            value: "30 days".to_string(),
            parameter_type: ParameterType::TimeWindow,
            constraint: Some(ParameterConstraint {
                constraint_type: ConstraintType::Minimum,
                value: "30 days".to_string(),
            }),
        };
        let oscal = to_oscal_param(&param);
        assert_eq!(oscal.id, "POL-AC-001_prm_0");
        assert_eq!(oscal.label, "within 30 days");
        assert_eq!(oscal.values, vec!["30 days"]);
        assert_eq!(oscal.constraints.len(), 1);
        assert_eq!(oscal.constraints[0].description, "minimum: 30 days");
    }

    #[test]
    fn to_oscal_param_maximum_constraint() {
        let param = PolicyParameter {
            id: "POL-AC-001_prm_0".to_string(),
            requirement_id: "POL-AC-001".to_string(),
            label: "no more than 15 minutes".to_string(),
            value: "15 minutes".to_string(),
            parameter_type: ParameterType::Threshold,
            constraint: Some(ParameterConstraint {
                constraint_type: ConstraintType::Maximum,
                value: "15 minutes".to_string(),
            }),
        };
        let oscal = to_oscal_param(&param);
        assert_eq!(oscal.constraints[0].description, "maximum: 15 minutes");
    }

    #[test]
    fn to_oscal_param_exact_constraint() {
        let param = PolicyParameter {
            id: "POL-AC-001_prm_0".to_string(),
            requirement_id: "POL-AC-001".to_string(),
            label: "annually".to_string(),
            value: "annually".to_string(),
            parameter_type: ParameterType::Frequency,
            constraint: Some(ParameterConstraint {
                constraint_type: ConstraintType::Exact,
                value: "annually".to_string(),
            }),
        };
        let oscal = to_oscal_param(&param);
        assert_eq!(oscal.constraints[0].description, "exact: annually");
    }

    #[test]
    fn to_oscal_param_no_constraint() {
        let param = PolicyParameter {
            id: "POL-AC-001_prm_0".to_string(),
            requirement_id: "POL-AC-001".to_string(),
            label: "annually".to_string(),
            value: "annually".to_string(),
            parameter_type: ParameterType::Frequency,
            constraint: None,
        };
        let oscal = to_oscal_param(&param);
        assert!(oscal.constraints.is_empty());
        assert_eq!(oscal.values, vec!["annually"]);
    }

    // ── T034: Idempotence ────────────────────────────────────────────────

    #[test]
    fn double_extraction_identical() {
        let mut doc = make_doc(vec![
            make_req("POL-AC-001", "Change passwords within 30 days"),
            make_req("POL-AC-002", "Review at least annually"),
        ]);

        extract_parameters(&mut doc).unwrap();
        let first_texts: Vec<String> =
            doc.sections[0].requirements.iter().map(|r| r.text.clone()).collect();
        let first_params: Vec<Vec<PolicyParameter>> =
            doc.sections[0].requirements.iter().map(|r| r.parameters.clone()).collect();

        // Second extraction — should be identical
        extract_parameters(&mut doc).unwrap();
        let second_texts: Vec<String> =
            doc.sections[0].requirements.iter().map(|r| r.text.clone()).collect();
        let second_params: Vec<Vec<PolicyParameter>> =
            doc.sections[0].requirements.iter().map(|r| r.parameters.clone()).collect();

        assert_eq!(first_texts, second_texts, "Texts should be identical after double extraction");
        assert_eq!(
            first_params, second_params,
            "Parameters should be identical after double extraction"
        );
    }

    // ── T035: Negative fixtures (SEC-2, EC-2) ────────────────────────────

    #[test]
    fn negative_fixtures_no_false_positives() {
        let cases = vec![
            "See Section 3.2 for details",
            "Per NIST SP 800-53 guidance",
            "Per RFC 2119 requirements",
            "per Appendix A",
            "See Table 5",
        ];
        for text in cases {
            let (_, params) = extract_parameters_from_text("POL-AC-001", text).unwrap();
            assert!(params.is_empty(), "Expected no params for: {text:?}, got: {params:?}");
        }
    }

    // ── T037: Edge cases ─────────────────────────────────────────────────

    #[test]
    fn ec1_no_params_text_unchanged() {
        let (text, params) =
            extract_parameters_from_text("POL-AC-001", "Users must authenticate").unwrap();
        assert!(params.is_empty());
        assert_eq!(text, "Users must authenticate");
    }

    #[test]
    fn ec4_multi_param_unique_ids() {
        let (_, params) = extract_parameters_from_text(
            "POL-AC-001",
            "Change within 30 days and audit at least annually",
        )
        .unwrap();
        assert_eq!(params.len(), 2);
        assert_ne!(params[0].id, params[1].id);
    }

    #[test]
    fn ec5_empty_document_no_error() {
        let mut doc =
            PolicyDocument { id: "empty".to_string(), metadata: test_metadata(), sections: vec![] };
        assert!(extract_parameters(&mut doc).is_ok());
    }

    #[test]
    fn ec6_bare_quarterly_exact() {
        let (_, params) =
            extract_parameters_from_text("POL-AC-001", "Reviews conducted quarterly").unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].constraint.as_ref().unwrap().constraint_type, ConstraintType::Exact);
    }

    #[test]
    fn ec7_no_less_than_minimum() {
        let (_, params) =
            extract_parameters_from_text("POL-AC-001", "at least no less than 30 days retention")
                .unwrap();
        // "no less than" → Threshold::Minimum
        assert!(!params.is_empty());
    }

    #[test]
    fn ec9_time_window_weeks_months_years() {
        let cases = vec![
            ("within 6 weeks", "6 weeks", ConstraintType::Minimum),
            ("after 3 months", "3 months", ConstraintType::Minimum),
            ("every 2 years", "2 years", ConstraintType::Exact),
        ];
        for (text, expected_value, expected_constraint) in cases {
            let (_, params) = extract_parameters_from_text("POL-AC-001", text).unwrap();
            assert_eq!(params.len(), 1, "Expected 1 param for: {text:?}");
            assert_eq!(params[0].value, expected_value);
            assert_eq!(params[0].constraint.as_ref().unwrap().constraint_type, expected_constraint);
        }
    }

    #[test]
    fn ec10_empty_text_no_error() {
        let (text, params) = extract_parameters_from_text("POL-AC-001", "").unwrap();
        assert!(params.is_empty());
        assert_eq!(text, "");
    }

    // ── US5: control_with_params integration test ────────────────────────

    #[test]
    fn control_with_params_produces_oscal_params() {
        use crate::oscal::catalog::build_catalog;

        let mut doc = make_doc(vec![make_req(
            "some-stable-id-001",
            "Passwords must be changed within 30 days",
        )]);

        // Enrich document with parameters
        extract_parameters(&mut doc).unwrap();
        assert_eq!(
            doc.sections[0].requirements[0].parameters.len(),
            1,
            "Requirement should have 1 parameter after extraction"
        );

        // Build catalog and emit params
        let mut catalog = build_catalog(&doc, None).unwrap();

        // Populate params in catalog controls from extracted parameters
        for group in &mut catalog.groups {
            for control in &mut group.controls {
                // Find the matching requirement
                for section in &doc.sections {
                    for req in &section.requirements {
                        if req.stable_id.as_deref() == Some("some-stable-id-001") {
                            control.params = req.parameters.iter().map(to_oscal_param).collect();
                        }
                    }
                }
            }
        }

        let json =
            serde_json::to_string_pretty(&crate::oscal::catalog::CatalogEnvelope { catalog })
                .unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let ctrl = &v["catalog"]["groups"][0]["controls"][0];
        assert!(ctrl.get("params").is_some(), "Control should have 'params' array in JSON");
        let params_arr = ctrl["params"].as_array().unwrap();
        assert_eq!(params_arr.len(), 1);
        assert!(params_arr[0]["id"].as_str().unwrap().contains("prm_0"));
        assert_eq!(params_arr[0]["label"].as_str().unwrap(), "within 30 days");
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    fn test_metadata() -> DocumentMetadata {
        DocumentMetadata {
            title: "Test Policy".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("test.md"),
            content_hash: None,
        }
    }

    fn make_req(stable_id: &str, text: &str) -> crate::model::PolicyRequirement {
        crate::model::PolicyRequirement {
            stable_id: Some(stable_id.to_string()),
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
        }
    }

    fn make_doc(reqs: Vec<crate::model::PolicyRequirement>) -> PolicyDocument {
        PolicyDocument {
            id: "test-doc".to_string(),
            metadata: test_metadata(),
            sections: vec![PolicySection {
                title: "Test Section".to_string(),
                heading_level: 1,
                source_line: 1,
                body_text: None,
                children: vec![],
                requirements: reqs,
            }],
        }
    }
}

//! OSCAL Assessment Plan builder (WI-41).
//!
//! Produces an Assessment Plan JSON envelope with `reviewed-controls` populated
//! from conversion output control IDs.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::ForgeError;
use crate::model::DocumentMetadata;
use crate::uuid::generate_stable_id;

// ─── Structs (T005) ────────────────────────────────────────────────────

/// Top-level Assessment Plan JSON envelope.
/// Serializes to `{"assessment-plan": {...}}`.
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentPlanEnvelope {
    #[serde(rename = "assessment-plan")]
    pub assessment_plan: AssessmentPlan,
}

/// OSCAL Assessment Plan root object.
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentPlan {
    pub uuid: String,
    pub metadata: ApMetadata,
    #[serde(rename = "import-ssp")]
    pub import_ssp: ImportSsp,
    #[serde(rename = "reviewed-controls")]
    pub reviewed_controls: ReviewedControls,
}

/// OSCAL metadata for the Assessment Plan.
#[derive(Debug, Clone, Serialize)]
pub struct ApMetadata {
    pub title: String,
    #[serde(rename = "last-modified")]
    pub last_modified: String,
    pub version: String,
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
}

/// SSP reference — href passed through verbatim from CLI flag.
#[derive(Debug, Clone, Serialize)]
pub struct ImportSsp {
    pub href: String,
}

/// Container defining assessment scope with one control-selections group.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewedControls {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "control-selections")]
    pub control_selections: Vec<ApControlSelection>,
}

/// A single control-selection group listing included controls.
#[derive(Debug, Clone, Serialize)]
pub struct ApControlSelection {
    #[serde(rename = "include-controls")]
    pub include_controls: Vec<ApIncludeControl>,
}

/// A single control identifier entry.
#[derive(Debug, Clone, Serialize)]
pub struct ApIncludeControl {
    #[serde(rename = "control-id")]
    pub control_id: String,
}

// ─── Builder (T006) ────────────────────────────────────────────────────

/// Build an OSCAL Assessment Plan JSON envelope from conversion output.
///
/// # Arguments
///
/// * `control_ids` — Control IDs from the conversion pipeline. Duplicates
///   are removed and order is normalized (sorted) for determinism.
/// * `import_ssp_href` — Path to the SSP, from `--import-ssp` CLI flag.
///   Must be non-empty; returns `ForgeError::Validation` if empty.
/// * `policy_title` — Title of the source policy document.
///
/// # Errors
///
/// * `ForgeError::Validation` — if `import_ssp_href` is empty or whitespace-only
/// * `ForgeError::AssessmentPlanBuild` — if metadata assembly fails
pub fn build_assessment_plan(
    control_ids: &[String],
    import_ssp_href: &str,
    policy_title: &str,
) -> Result<AssessmentPlanEnvelope, ForgeError> {
    // SEC-2: Validate non-empty SSP href
    if import_ssp_href.trim().is_empty() {
        return Err(ForgeError::Validation("--import-ssp must not be empty".to_string()));
    }

    // Sort + dedup control IDs (SEC-3)
    let mut sorted_ids: Vec<String> = control_ids.to_vec();
    sorted_ids.sort();
    sorted_ids.dedup();

    if sorted_ids.is_empty() {
        tracing::warn!("Zero controls found — Assessment Plan will have empty include-controls");
    }

    // S-2: Use shared assemble_metadata function
    let doc_meta = DocumentMetadata {
        title: format!("Assessment Plan for {policy_title}"),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    let real_metadata = crate::oscal::assemble_metadata(&doc_meta, None)?;

    // UUID v5 seed: deterministic from sorted control IDs + SSP href (SEC-4)
    let seed = format!("assessment-plan|{}|{}", sorted_ids.join(","), import_ssp_href);
    let uuid = generate_stable_id(&seed);

    let deduped_count = sorted_ids.len();
    let include_controls: Vec<ApIncludeControl> =
        sorted_ids.into_iter().map(|id| ApIncludeControl { control_id: id }).collect();

    let envelope = AssessmentPlanEnvelope {
        assessment_plan: AssessmentPlan {
            uuid: uuid.to_string(),
            metadata: ApMetadata {
                title: real_metadata.title,
                last_modified: real_metadata.last_modified.to_rfc3339(),
                version: real_metadata.version,
                oscal_version: real_metadata.oscal_version,
            },
            import_ssp: ImportSsp { href: import_ssp_href.to_string() },
            reviewed_controls: ReviewedControls {
                description: Some(format!(
                    "Controls derived from {policy_title} for assessment review."
                )),
                control_selections: vec![ApControlSelection { include_controls }],
            },
        },
    };

    tracing::info!(
        control_count = deduped_count,
        ssp_href = import_ssp_href,
        "Assessment Plan generated"
    );

    Ok(envelope)
}

// ─── Path Helper (T007) ────────────────────────────────────────────────

/// Derive the Assessment Plan output file path from the input and primary output paths.
///
/// Output filename: `{input_stem}-assessment-plan.json`
/// Output directory: parent of `primary_output` if `Some`; else `.` (cwd)
#[must_use]
pub fn derive_ap_output_path(input: &Path, primary_output: Option<&Path>) -> PathBuf {
    let stem = input.file_stem().map_or("policy", |s| s.to_str().unwrap_or("policy"));
    let filename = format!("{stem}-assessment-plan.json");

    match primary_output.and_then(|p| p.parent()) {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(filename),
        _ => PathBuf::from(format!("./{filename}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── T004: AC-1 — Root key "assessment-plan" ───────────────────────

    #[test]
    fn ac1_root_key_is_assessment_plan() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "./ssp.json", "Test Policy").unwrap();
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("\"assessment-plan\""), "Root key must be assessment-plan");
    }

    // ─── T004: AC-2 — Metadata fields ──────────────────────────────────

    #[test]
    fn ac2_metadata_fields_present_and_correct() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "./ssp.json", "Corp Policy").unwrap();
        let ap = &envelope.assessment_plan;
        assert_eq!(ap.metadata.title, "Assessment Plan for Corp Policy");
        assert_eq!(ap.metadata.version, "1.0.0");
        assert_eq!(ap.metadata.oscal_version, "1.2.0");
        assert!(!ap.metadata.last_modified.is_empty());
    }

    // ─── T004: AC-4 — Reviewed controls with 10 IDs ────────────────────

    #[test]
    fn ac4_reviewed_controls_with_10_ids() {
        let ids: Vec<String> = (1..=10).map(|i| format!("POL-AC-{i:03}")).collect();
        let envelope = build_assessment_plan(&ids, "./ssp.json", "Policy").unwrap();
        let controls =
            &envelope.assessment_plan.reviewed_controls.control_selections[0].include_controls;
        assert_eq!(controls.len(), 10);
        for (i, ctrl) in controls.iter().enumerate() {
            assert_eq!(ctrl.control_id, format!("POL-AC-{:03}", i + 1));
        }
    }

    // ─── T004: AC-7 — reviewed-controls.description references policy title ─

    #[test]
    fn ac7_description_references_policy_title() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "./ssp.json", "My Security Policy").unwrap();
        let desc = envelope.assessment_plan.reviewed_controls.description.as_deref().unwrap();
        assert!(
            desc.contains("My Security Policy"),
            "Description should reference policy title, got: {desc}"
        );
    }

    // ─── T004: EC-1 — Zero controls → empty include-controls ───────────

    #[test]
    fn ec1_zero_controls_empty_include_controls() {
        let ids: Vec<String> = vec![];
        let envelope = build_assessment_plan(&ids, "./ssp.json", "Policy").unwrap();
        let controls =
            &envelope.assessment_plan.reviewed_controls.control_selections[0].include_controls;
        assert!(controls.is_empty());
    }

    // ─── T004: EC-3 — Duplicate IDs → deduplicated output ──────────────

    #[test]
    fn ec3_duplicate_ids_deduplicated() {
        let ids = vec![
            "AC-001".to_string(),
            "AC-002".to_string(),
            "AC-001".to_string(),
            "AC-003".to_string(),
            "AC-002".to_string(),
        ];
        let envelope = build_assessment_plan(&ids, "./ssp.json", "Policy").unwrap();
        let controls =
            &envelope.assessment_plan.reviewed_controls.control_selections[0].include_controls;
        assert_eq!(controls.len(), 3);
        assert_eq!(controls[0].control_id, "AC-001");
        assert_eq!(controls[1].control_id, "AC-002");
        assert_eq!(controls[2].control_id, "AC-003");
    }

    // ─── T004: EC-4 — JSON parseable with hyphenated root key ──────────

    #[test]
    fn ec4_json_parseable_hyphenated_root_key() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "./ssp.json", "Policy").unwrap();
        let json = serde_json::to_string_pretty(&envelope).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("assessment-plan").is_some(), "Must have hyphenated root key");
    }

    // ─── T004: EC-5/unit — Different inputs → different UUIDs ──────────

    #[test]
    fn ec5_unit_different_inputs_different_uuids() {
        let ids_a = vec!["AC-001".to_string()];
        let ids_b = vec!["AC-002".to_string()];
        let envelope_a = build_assessment_plan(&ids_a, "./ssp.json", "Policy").unwrap();
        let envelope_b = build_assessment_plan(&ids_b, "./ssp.json", "Policy").unwrap();
        assert_ne!(
            envelope_a.assessment_plan.uuid, envelope_b.assessment_plan.uuid,
            "Different control sets must produce different UUIDs"
        );
    }

    // ─── T004: EC-5/unit — Same inputs → same UUIDs ────────────────────

    #[test]
    fn ec5_unit_same_inputs_same_uuids() {
        let ids = vec!["AC-001".to_string(), "AC-002".to_string()];
        let envelope_a = build_assessment_plan(&ids, "./ssp.json", "Policy").unwrap();
        let envelope_b = build_assessment_plan(&ids, "./ssp.json", "Policy").unwrap();
        assert_eq!(
            envelope_a.assessment_plan.uuid, envelope_b.assessment_plan.uuid,
            "Same inputs must produce identical UUIDs"
        );
    }

    // ─── T011: AC-3 — import-ssp.href equals provided path ────────────

    #[test]
    fn ac3_import_ssp_href_matches_input() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "./ssp/system-ssp.json", "Policy").unwrap();
        assert_eq!(envelope.assessment_plan.import_ssp.href, "./ssp/system-ssp.json");
    }

    // ─── T011: EC-2 — Empty or whitespace-only href → Validation error ─

    #[test]
    fn ec2_empty_href_returns_validation_error() {
        let ids = vec!["AC-001".to_string()];
        let result = build_assessment_plan(&ids, "", "Policy");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("import-ssp"),
            "Error should mention import-ssp, got: {err}"
        );
    }

    #[test]
    fn ec2_whitespace_only_href_returns_validation_error() {
        let ids = vec!["AC-001".to_string()];
        let result = build_assessment_plan(&ids, "   ", "Policy");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("import-ssp"),
            "Error should mention import-ssp, got: {err}"
        );
    }

    // ─── T007: derive_ap_output_path tests ─────────────────────────────

    #[test]
    fn derive_ap_path_no_primary_output() {
        let path = derive_ap_output_path(Path::new("policy.md"), None);
        assert_eq!(path, PathBuf::from("./policy-assessment-plan.json"));
    }

    #[test]
    fn derive_ap_path_with_primary_output() {
        let path =
            derive_ap_output_path(Path::new("policy.md"), Some(Path::new("out/catalog.json")));
        assert_eq!(path, PathBuf::from("out/policy-assessment-plan.json"));
    }
}

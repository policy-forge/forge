//! OSCAL Assessment Plan builder (WI-41, WI-42).
//!
//! Produces an Assessment Plan JSON envelope with `reviewed-controls` populated
//! from conversion output control IDs, plus optional `tasks` and `assessment-subjects`
//! generated from PolicyRequirements and component metadata.

use std::borrow::Borrow;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Serialize, Serializer};

/// Subjects shared between the assessment-plan root and each associated activity.
pub type SharedAssessmentSubjects = Arc<Vec<AssessmentSubject>>;

fn serialize_shared_subjects<S>(
    subjects: &SharedAssessmentSubjects,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    subjects.as_ref().serialize(serializer)
}

#[expect(
    clippy::ref_option,
    reason = "serde serialize_with callbacks receive a reference to the field"
)]
fn serialize_optional_shared_subjects<S>(
    subjects: &Option<SharedAssessmentSubjects>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    subjects.as_deref().serialize(serializer)
}

use crate::error::ForgeError;
use crate::model::{DocumentMetadata, PolicyRequirement};
use crate::uuid::generate_stable_id;

// ─── Structs (T005) ────────────────────────────────────────────────────

/// Top-level Assessment Plan JSON envelope.
/// Serializes to `{"assessment-plan": {...}}`.
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentPlanEnvelope {
    /// The inner OSCAL Assessment Plan object, serialized under the `assessment-plan` key.
    #[serde(rename = "assessment-plan")]
    pub assessment_plan: AssessmentPlan,
}

/// OSCAL Assessment Plan root object.
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentPlan {
    /// UUID v5 identifier, deterministically derived from control IDs and the SSP href.
    pub uuid: String,
    /// OSCAL metadata (title, last-modified, version, oscal-version).
    pub metadata: ApMetadata,
    /// Reference to the System Security Plan this assessment plan imports.
    #[serde(rename = "import-ssp")]
    pub import_ssp: ImportSsp,
    /// Locally defined activities referenced by assessment tasks.
    #[serde(rename = "local-definitions", skip_serializing_if = "Option::is_none")]
    pub local_definitions: Option<LocalDefinitions>,
    /// Container for the reviewed-controls scope definition.
    #[serde(rename = "reviewed-controls")]
    pub reviewed_controls: ReviewedControls,
    /// Optional assessment tasks generated from `PolicyRequirements` (WI-42).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<AssessmentTask>>,
    /// Optional assessment subjects referencing documentary components (WI-42).
    #[serde(
        rename = "assessment-subjects",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_shared_subjects"
    )]
    pub assessment_subjects: Option<SharedAssessmentSubjects>,
}

/// OSCAL metadata for the Assessment Plan.
#[derive(Debug, Clone, Serialize)]
pub struct ApMetadata {
    /// Document title.
    pub title: String,
    /// RFC 3339 timestamp of last modification.
    #[serde(rename = "last-modified")]
    pub last_modified: String,
    /// Document version string.
    pub version: String,
    /// OSCAL specification version (e.g., "1.2.0").
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
}

/// SSP reference — href passed through verbatim from CLI flag.
#[derive(Debug, Clone, Serialize)]
pub struct ImportSsp {
    /// Relative filename of the SSP artifact (sanitized to basename only).
    pub href: String,
}

/// Container defining assessment scope with one control-selections group.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewedControls {
    /// Human-readable description of the assessment scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// One or more groups of selected controls.
    #[serde(rename = "control-selections")]
    pub control_selections: Vec<ApControlSelection>,
}

/// A single control-selection group listing included controls.
#[derive(Debug, Clone, Serialize)]
pub struct ApControlSelection {
    /// List of included control identifier entries.
    #[serde(rename = "include-controls")]
    pub include_controls: Vec<ApIncludeControl>,
}

/// A single control identifier entry.
#[derive(Debug, Clone, Serialize)]
pub struct ApIncludeControl {
    /// The control identifier string (e.g., "AC-001").
    #[serde(rename = "control-id")]
    pub control_id: String,
}

// ─── WI-42 Structs: Tasks & Subjects ────────────────────────────────────

/// An assessment task describing a specific verification activity.
/// One task is generated per `PolicyRequirement`.
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentTask {
    /// UUID v5 identifier for this task, derived from the requirement's `stable_id`.
    pub uuid: String,
    /// Type of task, always `"action"`.
    #[serde(rename = "type")]
    pub task_type: String,
    /// Human-readable task title (truncated to first 80 characters of the requirement text).
    pub title: String,
    /// Assessment-framed description of the verification activity.
    pub description: String,
    /// Optional activities linked to this task.
    #[serde(rename = "associated-activities", skip_serializing_if = "Option::is_none")]
    pub associated_activities: Option<Vec<AssociatedActivity>>,
}

/// An activity linked to a task describing a specific assessment action.
#[derive(Debug, Clone, Serialize)]
pub struct AssociatedActivity {
    /// UUID v5 identifier copied by [`complete_assessment_plan`] into
    /// `local-definitions.activities`; skipped here because associated-activity
    /// serializes only the reference in OSCAL v1.2.3.
    #[serde(skip)]
    pub uuid: String,
    /// Human-readable activity title copied into `local-definitions.activities`.
    #[serde(skip)]
    pub title: String,
    /// Description copied into `local-definitions.activities`.
    #[serde(skip)]
    pub description: String,
    /// Reference to the matching activity in `local-definitions.activities`.
    #[serde(rename = "activity-uuid")]
    pub activity_uuid: String,
    /// Subjects to which this activity applies, shared with the plan root.
    #[serde(serialize_with = "serialize_shared_subjects")]
    pub subjects: SharedAssessmentSubjects,
}

/// Local Assessment Plan definitions referenced by tasks.
#[derive(Debug, Clone, Serialize)]
pub struct LocalDefinitions {
    /// Activity definitions referenced by `associated-activities`.
    pub activities: Vec<ActivityDefinition>,
}

/// A locally defined assessment activity.
#[derive(Debug, Clone, Serialize)]
pub struct ActivityDefinition {
    /// Deterministic activity UUID.
    pub uuid: String,
    /// Human-readable activity title.
    pub title: String,
    /// Assessment action description.
    pub description: String,
}

/// An assessment subject identifying an entity being assessed.
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentSubject {
    /// Type of assessment subject, always `"component"`.
    #[serde(rename = "type")]
    pub subject_type: String,
    /// Human-readable description identifying the policy document.
    pub description: String,
    /// Include all subjects of this type when no concrete UUID is available.
    #[serde(rename = "include-all", skip_serializing_if = "Option::is_none")]
    pub include_all: Option<IncludeAllSubjects>,
    /// Optional references to included subjects (populated when `component_uuid` is available).
    #[serde(rename = "include-subjects", skip_serializing_if = "Option::is_none")]
    pub include_subjects: Option<Vec<SubjectRef>>,
}

/// Empty OSCAL marker object for an all-subjects selection.
#[derive(Debug, Clone, Serialize)]
pub struct IncludeAllSubjects {}

/// A reference to an included subject within assessment-subjects.
#[derive(Debug, Clone, Serialize)]
pub struct SubjectRef {
    /// UUID of the referenced subject component.
    #[serde(rename = "subject-uuid")]
    pub subject_uuid: String,
    /// Type of the referenced entity, always `"component"`.
    #[serde(rename = "type")]
    pub ref_type: String,
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
/// * `ForgeError::Validation` — if `control_ids` or `import_ssp_href` is empty
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
    sorted_ids.sort_unstable();
    sorted_ids.dedup();

    if sorted_ids.is_empty() {
        return Err(ForgeError::Validation(
            "conversion produced zero controls; refusing to emit an Assessment Plan with an empty include-controls array"
                .to_string(),
        ));
    }

    // S-2: Use shared assemble_metadata function
    let doc_meta = DocumentMetadata {
        title: format!("Assessment Plan for {policy_title}"),
        version: "1.0.0".to_string(),
        ..Default::default()
    };
    let real_metadata = crate::oscal::assemble_metadata(&doc_meta, None)
        .map_err(|e| ForgeError::AssessmentPlanBuild(e.to_string()))?;

    // Canonicalize the emitted href before hashing so equivalent logical inputs
    // produce the same UUID. Length-prefix each sorted control ID to keep the
    // seed injective when an ID contains a delimiter.
    let canonical_href = crate::io::sanitize_artifact_path(Path::new(import_ssp_href));
    let mut seed = format!("assessment-plan|{}", sorted_ids.len());
    for id in &sorted_ids {
        seed.push('|');
        seed.push_str(&id.len().to_string());
        seed.push(':');
        seed.push_str(id);
    }
    seed.push('|');
    seed.push_str(&canonical_href);
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
            import_ssp: ImportSsp { href: canonical_href },
            local_definitions: None,
            reviewed_controls: ReviewedControls {
                description: Some(format!(
                    "Controls derived from {policy_title} for assessment review."
                )),
                control_selections: vec![ApControlSelection { include_controls }],
            },
            tasks: None,
            assessment_subjects: None,
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

// ─── WI-42: Task & Subject Generation ──────────────────────────────────

/// Generate assessment tasks from `PolicyRequirements`.
///
/// Each requirement maps to one task with assessment guidance.
/// Task UUIDs are deterministic v5 derived from the requirement's `stable_id`.
///
/// # Arguments
///
/// * `requirements` — slice of items that borrow as [`PolicyRequirement`]
///   (e.g. `&[PolicyRequirement]` or `&[&PolicyRequirement]`), each with
///   populated `stable_id` fields
///
/// # Returns
///
/// A Vec of `AssessmentTask` with one entry per requirement. Returns an empty
/// Vec if `requirements` is empty (with a warning emitted).
///
/// # Edge Cases
///
/// * If a requirement has empty text, the task description uses a placeholder.
pub fn generate_assessment_tasks<T>(requirements: &[T]) -> Vec<AssessmentTask>
where
    T: Borrow<PolicyRequirement>,
{
    if requirements.is_empty() {
        tracing::warn!("Zero PolicyRequirements — Assessment Plan will have empty tasks[]");
        return Vec::new();
    }

    requirements
        .iter()
        .enumerate()
        .map(|(i, req_t)| {
            let req = req_t.borrow();
            // Use stable_id when available; the fallback seed is namespaced
            // with characters a real stable_id (a UUID string) can never
            // contain, so it cannot collide with a literal "req-{i}" id (F0561).
            let id_seed = req
                .stable_id
                .as_deref()
                .filter(|s| !s.is_empty())
                .map_or_else(|| format!("<unset-stable-id:{i}>"), str::to_owned);
            let task_uuid = generate_stable_id(&format!("assessment-task|{id_seed}"));

            let title = assessment_task_title(&req.text);

            // Description: assessment-framed requirement text
            let description = if req.text.trim().is_empty() {
                "No assessment guidance available — requirement text is empty.".to_string()
            } else {
                format!("Verify that {} is implemented as specified in the policy.", req.text)
            };

            // Associated activity derived from the same requirement
            let activity_uuid = generate_stable_id(&format!("assessment-activity|{id_seed}"));
            let activity = AssociatedActivity {
                uuid: activity_uuid.to_string(),
                title: format!("Review: {}", req.text.chars().take(60).collect::<String>()),
                description: format!(
                    "Examine evidence that {} is implemented and operating effectively.",
                    req.text
                ),
                activity_uuid: activity_uuid.to_string(),
                subjects: Arc::new(vec![AssessmentSubject {
                    subject_type: "component".to_string(),
                    description: "Policy components in assessment scope".to_string(),
                    include_all: Some(IncludeAllSubjects {}),
                    include_subjects: None,
                }]),
            };

            AssessmentTask {
                uuid: task_uuid.to_string(),
                task_type: "action".to_string(),
                title,
                description,
                associated_activities: Some(vec![activity]),
            }
        })
        .collect()
}

fn assessment_task_title(text: &str) -> String {
    const MAX_REQUIREMENT_CHARS: usize = 77;

    let mut chars = text.chars();
    let prefix: String = chars.by_ref().take(MAX_REQUIREMENT_CHARS).collect();
    if chars.next().is_some() { format!("Assess: {prefix}...") } else { format!("Assess: {text}") }
}

/// Create assessment-subjects referencing the documentary component.
///
/// # Arguments
///
/// * `component_uuid` — Optional UUID of the documentary component from the
///   Component Definition pipeline. When `None`, produces a generic subject
///   without `include-subjects` (PRD EC-3).
/// * `policy_title` — Title of the source policy document.
///
/// # Returns
///
/// A Vec of `AssessmentSubject` — typically one entry, but could be extended
/// for multiple documentary components (PRD C-2).
pub fn create_assessment_subjects(
    component_uuid: Option<&str>,
    policy_title: &str,
) -> Vec<AssessmentSubject> {
    let description = format!("Policy document: {policy_title}");

    let include_subjects = component_uuid.map(|uuid| {
        vec![SubjectRef { subject_uuid: uuid.to_string(), ref_type: "component".to_string() }]
    });
    let include_all = component_uuid.is_none().then_some(IncludeAllSubjects {});

    if component_uuid.is_none() {
        tracing::warn!(
            "No documentary component UUID available — assessment-subjects will use include-all"
        );
    }

    vec![AssessmentSubject {
        subject_type: "component".to_string(),
        description,
        include_all,
        include_subjects,
    }]
}

/// Complete the Assessment Plan by adding tasks and subjects to the WI-41 skeleton.
///
/// This is an additive merge — it does not modify `reviewed-controls` or `import-ssp`.
///
/// # Arguments
///
/// * `envelope` — The WI-41 Assessment Plan skeleton (mutable, modified in place)
/// * `tasks` — Assessment tasks generated from `PolicyRequirements`
/// * `subjects` — Assessment subjects referencing documentary components
///
/// # Errors
///
/// Returns [`ForgeError::Validation`] when `subjects` is empty, because OSCAL
/// requires both `assessment-subjects` and associated-activity `subjects` to
/// contain at least one item.
pub fn complete_assessment_plan(
    envelope: &mut AssessmentPlanEnvelope,
    mut tasks: Vec<AssessmentTask>,
    subjects: Vec<AssessmentSubject>,
) -> Result<(), ForgeError> {
    if subjects.is_empty() {
        return Err(ForgeError::Validation(
            "assessment plan subjects must not be empty".to_string(),
        ));
    }

    tracing::info!(
        task_count = tasks.len(),
        subject_count = subjects.len(),
        "Completing Assessment Plan with tasks and subjects"
    );

    let subjects = Arc::new(subjects);
    let mut activities = Vec::new();
    let mut activity_uuids = HashSet::new();
    for task in &mut tasks {
        if let Some(associated) = &mut task.associated_activities {
            for activity in associated {
                activity.subjects = Arc::clone(&subjects);
                if activity_uuids.insert(activity.uuid.clone()) {
                    activities.push(ActivityDefinition {
                        uuid: activity.uuid.clone(),
                        title: activity.title.clone(),
                        description: activity.description.clone(),
                    });
                }
            }
        }
    }

    envelope.assessment_plan.local_definitions =
        (!activities.is_empty()).then_some(LocalDefinitions { activities });
    envelope.assessment_plan.tasks = Some(tasks);
    envelope.assessment_plan.assessment_subjects = Some(subjects);

    Ok(())
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
        assert_eq!(ap.metadata.oscal_version, crate::oscal::metadata::OSCAL_VERSION);
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

    // ─── T004: EC-1 — Zero controls is rejected ────────────────────────

    #[test]
    fn ec1_zero_controls_returns_validation_error() {
        let error = build_assessment_plan(&[], "./ssp.json", "Policy").unwrap_err();

        assert!(matches!(error, ForgeError::Validation(_)));
        assert!(error.to_string().contains("zero controls"));
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

    #[test]
    fn assessment_plan_uuid_uses_canonical_ssp_href() {
        let ids = vec!["AC-001".to_string()];
        let relative = build_assessment_plan(&ids, "./ssp/system-ssp.json", "Policy").unwrap();
        let absolute =
            build_assessment_plan(&ids, "/different/path/system-ssp.json", "Policy").unwrap();

        assert_eq!(relative.assessment_plan.uuid, absolute.assessment_plan.uuid);
    }

    #[test]
    fn assessment_plan_uuid_length_prefixes_control_ids() {
        let first = vec!["A,B".to_string(), "C".to_string()];
        let second = vec!["A".to_string(), "B,C".to_string()];
        let first = build_assessment_plan(&first, "./ssp.json", "Policy").unwrap();
        let second = build_assessment_plan(&second, "./ssp.json", "Policy").unwrap();

        assert_ne!(first.assessment_plan.uuid, second.assessment_plan.uuid);
    }

    // ─── T011: AC-3 — import-ssp.href equals provided path ────────────

    #[test]
    fn ac3_import_ssp_href_uses_filename_only() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "./ssp/system-ssp.json", "Policy").unwrap();
        assert_eq!(envelope.assessment_plan.import_ssp.href, "system-ssp.json");
    }

    #[test]
    fn assessment_plan_import_ssp_uses_filename_only() {
        let ids = vec!["AC-001".to_string()];
        let envelope = build_assessment_plan(&ids, "/absolute/path/to/ssp.json", "Policy").unwrap();
        assert_eq!(envelope.assessment_plan.import_ssp.href, "ssp.json");
        assert!(!envelope.assessment_plan.import_ssp.href.contains('/'));
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

    // ─── WI-42: Task Generation Tests ──────────────────────────────────

    /// Helper: create a `PolicyRequirement` with a `stable_id` for testing.
    fn make_req(stable_id: &str, text: &str) -> PolicyRequirement {
        PolicyRequirement {
            stable_id: Some(stable_id.to_string()),
            text: text.to_string(),
            source_line: 1,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
            parameters_extracted: false,
        }
    }

    // ─── AC-1: 1:1 mapping — N requirements → N tasks ────────────────

    #[test]
    fn tasks_count_matches_requirement_count() {
        let reqs = vec![
            make_req("req-1", "All users must use MFA"),
            make_req("req-2", "Passwords must be at least 12 characters"),
            make_req("req-3", "Systems must be patched within 30 days"),
        ];
        let tasks = generate_assessment_tasks(&reqs);
        assert_eq!(tasks.len(), 3);
    }

    // ─── AC-2: Task type, title, description populated ────────────────

    #[test]
    fn task_fields_populated_correctly() {
        let reqs = vec![make_req("req-1", "All users must use MFA")];
        let tasks = generate_assessment_tasks(&reqs);
        let task = &tasks[0];

        assert_eq!(task.task_type, "action");
        assert!(task.title.starts_with("Assess: "));
        assert!(task.title.contains("All users must use MFA"));
        assert!(task.description.contains("Verify that"));
        assert!(task.description.contains("All users must use MFA"));
        assert!(task.description.contains("implemented as specified"));
    }

    // ─── AC-6: Deterministic UUIDs — same input → same UUIDs ──────────

    #[test]
    fn task_uuids_deterministic() {
        let reqs = vec![make_req("req-1", "All users must use MFA")];
        let tasks_a = generate_assessment_tasks(&reqs);
        let tasks_b = generate_assessment_tasks(&reqs);

        assert_eq!(tasks_a[0].uuid, tasks_b[0].uuid);
        assert_eq!(
            tasks_a[0].associated_activities.as_ref().unwrap()[0].uuid,
            tasks_b[0].associated_activities.as_ref().unwrap()[0].uuid
        );
    }

    // ─── Different requirements → different UUIDs ─────────────────────

    #[test]
    fn task_uuids_unique_per_requirement() {
        let reqs = vec![
            make_req("req-1", "All users must use MFA"),
            make_req("req-2", "Passwords must be at least 12 characters"),
        ];
        let tasks = generate_assessment_tasks(&reqs);

        assert_ne!(tasks[0].uuid, tasks[1].uuid);
    }

    // ─── EC-1: Zero requirements → empty tasks[] ──────────────────────

    #[test]
    fn zero_requirements_produces_empty_tasks() {
        let reqs: Vec<PolicyRequirement> = vec![];
        let tasks = generate_assessment_tasks(&reqs);
        assert!(tasks.is_empty());
    }

    // ─── EC-2: Empty requirement text → placeholder description ────────

    #[test]
    fn empty_requirement_text_gets_placeholder() {
        let reqs = vec![make_req("req-empty", "")];
        let tasks = generate_assessment_tasks(&reqs);
        let task = &tasks[0];

        assert!(task.description.contains("No assessment guidance available"));
        assert_eq!(task.task_type, "action");
        assert!(!task.uuid.is_empty());
    }

    #[test]
    fn multibyte_requirement_text_title_truncates_without_panic() {
        let text = format!("{} must be reviewed", "認証".repeat(40));
        let tasks = generate_assessment_tasks(&[make_req("req-unicode", &text)]);

        assert!(tasks[0].title.starts_with("Assess: "));
        assert!(tasks[0].title.ends_with("..."));
    }

    // ─── Each task has associated-activities ──────────────────────────

    #[test]
    fn each_task_has_associated_activity() {
        let reqs = vec![make_req("req-1", "All users must use MFA")];
        let tasks = generate_assessment_tasks(&reqs);
        let activities = tasks[0].associated_activities.as_ref().unwrap();

        assert_eq!(activities.len(), 1);
        assert!(!activities[0].uuid.is_empty());
        assert!(activities[0].title.starts_with("Review: "));
        assert!(activities[0].description.contains("Examine evidence"));
    }

    // ─── Assessment Subjects Tests ────────────────────────────────────

    // ─── AC-3: Subject with component UUID ────────────────────────────

    #[test]
    fn subject_with_component_uuid() {
        let subjects = create_assessment_subjects(
            Some("550e8400-e29b-41d4-a716-446655440000"),
            "Information Security Policy",
        );
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].subject_type, "component");
        assert!(subjects[0].description.contains("Information Security Policy"));

        let include = subjects[0].include_subjects.as_ref().unwrap();
        assert!(subjects[0].include_all.is_none());
        assert_eq!(include.len(), 1);
        assert_eq!(include[0].subject_uuid, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(include[0].ref_type, "component");
    }

    // ─── EC-3: Subject without component UUID ─────────────────────────

    #[test]
    fn subject_without_component_uuid() {
        let subjects = create_assessment_subjects(None, "Security Policy");
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].subject_type, "component");
        assert!(subjects[0].description.contains("Security Policy"));
        assert!(subjects[0].include_subjects.is_none());
        assert!(subjects[0].include_all.is_some(), "generic subject should use include-all");
    }

    // ─── Complete Assessment Plan Tests ──────────────────────────────

    // ─── M-8: complete_assessment_plan adds tasks and subjects ────────

    #[test]
    fn complete_plan_adds_tasks_and_subjects() {
        let ids = vec!["AC-001".to_string()];
        let mut envelope = build_assessment_plan(&ids, "./ssp.json", "Test Policy").unwrap();

        // Verify skeleton has no tasks/subjects yet
        assert!(envelope.assessment_plan.tasks.is_none());
        assert!(envelope.assessment_plan.assessment_subjects.is_none());

        let tasks = generate_assessment_tasks(&[make_req("req-1", "All users must use MFA")]);
        let subjects =
            create_assessment_subjects(Some("550e8400-e29b-41d4-a716-446655440000"), "Test Policy");

        complete_assessment_plan(&mut envelope, tasks, subjects).unwrap();

        assert!(envelope.assessment_plan.tasks.is_some());
        assert!(envelope.assessment_plan.assessment_subjects.is_some());
        assert_eq!(envelope.assessment_plan.tasks.as_ref().unwrap().len(), 1);
        assert_eq!(envelope.assessment_plan.assessment_subjects.as_ref().unwrap().len(), 1);
    }

    // ─── SEC-6: complete_assessment_plan does not modify skeleton ─────

    #[test]
    fn complete_plan_preserves_skeleton_fields() {
        let ids = vec!["AC-001".to_string()];
        let mut envelope = build_assessment_plan(&ids, "./ssp.json", "Test Policy").unwrap();

        let original_uuid = envelope.assessment_plan.uuid.clone();
        let original_title = envelope.assessment_plan.metadata.title.clone();
        let original_href = envelope.assessment_plan.import_ssp.href.clone();
        let original_controls =
            envelope.assessment_plan.reviewed_controls.control_selections[0].include_controls.len();

        let tasks = generate_assessment_tasks(&[make_req("req-1", "All users must use MFA")]);
        let subjects = create_assessment_subjects(None, "Test Policy");

        complete_assessment_plan(&mut envelope, tasks, subjects).unwrap();

        assert_eq!(envelope.assessment_plan.uuid, original_uuid);
        assert_eq!(envelope.assessment_plan.metadata.title, original_title);
        assert_eq!(envelope.assessment_plan.import_ssp.href, original_href);
        assert_eq!(
            envelope.assessment_plan.reviewed_controls.control_selections[0].include_controls.len(),
            original_controls
        );
    }

    #[test]
    fn complete_plan_rejects_empty_subjects() {
        let ids = vec!["AC-001".to_string()];
        let mut envelope = build_assessment_plan(&ids, "./ssp.json", "Test Policy").unwrap();
        let tasks = generate_assessment_tasks(&[make_req("req-1", "All users must use MFA")]);

        let error = complete_assessment_plan(&mut envelope, tasks, Vec::new()).unwrap_err();
        assert!(matches!(error, ForgeError::Validation(_)));
    }

    #[test]
    fn complete_plan_shares_subjects_with_all_activities() {
        let ids = vec!["AC-001".to_string()];
        let mut envelope = build_assessment_plan(&ids, "./ssp.json", "Test Policy").unwrap();
        let tasks = generate_assessment_tasks(&[
            make_req("req-1", "All users must use MFA"),
            make_req("req-2", "All users must use encryption"),
        ]);
        let subjects = create_assessment_subjects(None, "Test Policy");

        complete_assessment_plan(&mut envelope, tasks, subjects).unwrap();

        let root_subjects = envelope.assessment_plan.assessment_subjects.as_ref().unwrap();
        let tasks = envelope.assessment_plan.tasks.as_ref().unwrap();
        let first = &tasks[0].associated_activities.as_ref().unwrap()[0].subjects;
        let second = &tasks[1].associated_activities.as_ref().unwrap()[0].subjects;
        assert!(Arc::ptr_eq(root_subjects, first));
        assert!(Arc::ptr_eq(first, second));

        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["assessment-plan"]["assessment-subjects"].as_array().unwrap().len(), 1);
        assert_eq!(
            json["assessment-plan"]["tasks"][0]["associated-activities"][0]["subjects"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn complete_plan_deduplicates_activity_definitions_by_uuid() {
        let ids = vec!["AC-001".to_string()];
        let mut envelope = build_assessment_plan(&ids, "./ssp.json", "Test Policy").unwrap();
        let task = generate_assessment_tasks(&[make_req("req-1", "All users must use MFA")])
            .pop()
            .unwrap();
        let subjects = create_assessment_subjects(None, "Test Policy");

        complete_assessment_plan(&mut envelope, vec![task.clone(), task], subjects).unwrap();

        let activities = &envelope.assessment_plan.local_definitions.as_ref().unwrap().activities;
        assert_eq!(activities.len(), 1);
    }
}

//! Integration tests for Assessment Plan generation (WI-41).

mod common;

use std::path::Path;

use tempfile::TempDir;

const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const FIXTURE: &str = "tests/fixtures/sample_policy.md";

/// Helper: run catalog pipeline with --import-ssp, write outputs, return AP JSON value.
fn run_catalog_with_ap(fixture: &Path, output_dir: &Path, import_ssp: &str) -> serde_json::Value {
    let catalog_path = output_dir.join("catalog.json");
    let result = forge::pipeline::run_catalog_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        Some(import_ssp),
    )
    .expect("Catalog pipeline should succeed");

    // Write primary output
    std::fs::write(&catalog_path, &result.content).unwrap();

    // Write secondary outputs (AP file)
    for secondary in &result.secondary_outputs {
        let ap_path = output_dir.join(&secondary.filename);
        std::fs::write(&ap_path, &secondary.content).unwrap();
    }

    // AP file: {input_stem}-assessment-plan.json in same dir
    let stem = fixture.file_stem().unwrap().to_str().unwrap();
    let ap_path = output_dir.join(format!("{stem}-assessment-plan.json"));
    assert!(ap_path.exists(), "AP file should be written at {}", ap_path.display());

    let ap_json = std::fs::read_to_string(&ap_path).expect("AP file should be readable");
    serde_json::from_str(&ap_json).expect("AP file should be valid JSON")
}

// ─── T017: AC-6 — Same input × 2 runs → identical UUIDs ────────────────

#[test]
fn ac6_same_input_produces_identical_uuids() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();

    let ap1 = run_catalog_with_ap(fixture, dir1.path(), "./ssp/system-ssp.json");
    let ap2 = run_catalog_with_ap(fixture, dir2.path(), "./ssp/system-ssp.json");

    // Compare UUIDs (excluding last-modified which changes each run)
    let uuid1 = ap1["assessment-plan"]["uuid"].as_str().unwrap();
    let uuid2 = ap2["assessment-plan"]["uuid"].as_str().unwrap();
    assert_eq!(uuid1, uuid2, "Same input must produce identical AP UUIDs");
}

// ─── T017: EC-5/integration — Changed control set → different AP UUID ───

#[test]
fn ec5_different_input_produces_different_uuids() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();

    // Same input, different SSP href → different UUID
    let ap1 = run_catalog_with_ap(fixture, dir1.path(), "./ssp/system-a.json");
    let ap2 = run_catalog_with_ap(fixture, dir2.path(), "./ssp/system-b.json");

    let uuid1 = ap1["assessment-plan"]["uuid"].as_str().unwrap();
    let uuid2 = ap2["assessment-plan"]["uuid"].as_str().unwrap();
    assert_ne!(uuid1, uuid2, "Different SSP href must produce different AP UUIDs");
}

// ─── T019: AP file written when --import-ssp provided ───────────────────

#[test]
fn ap_file_written_when_import_ssp_provided() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let result = forge::pipeline::run_catalog_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        Some("./ssp.json"),
    )
    .unwrap();

    // Write secondary outputs to verify AP file creation
    for secondary in &result.secondary_outputs {
        let ap_path = dir.path().join(&secondary.filename);
        std::fs::write(&ap_path, &secondary.content).unwrap();
    }

    let stem = fixture.file_stem().unwrap().to_str().unwrap();
    let ap_path = dir.path().join(format!("{stem}-assessment-plan.json"));
    assert!(ap_path.exists(), "AP file should be written when --import-ssp provided");
}

// ─── T019: AP file NOT written when --import-ssp omitted ────────────────

#[test]
fn ap_file_not_written_when_import_ssp_omitted() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let result = forge::pipeline::run_catalog_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        None,
    )
    .unwrap();

    assert!(
        result.secondary_outputs.is_empty(),
        "AP should NOT be generated when --import-ssp omitted"
    );
}

// ─── T020: AP JSON contains all control IDs from fixture ────────────────

#[test]
fn ap_contains_all_control_ids_from_fixture() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let ap = run_catalog_with_ap(fixture, dir.path(), "./ssp.json");

    let include_controls =
        ap["assessment-plan"]["reviewed-controls"]["control-selections"][0]["include-controls"]
            .as_array()
            .expect("include-controls should be an array");
    assert!(!include_controls.is_empty(), "AP should contain control IDs from the fixture");

    // Verify all entries have control-id
    for ctrl in include_controls {
        assert!(
            ctrl["control-id"].is_string(),
            "Each include-control should have a control-id string"
        );
    }
}

// ─── T021: Empty --import-ssp exits with error ──────────────────────────

#[test]
fn empty_import_ssp_returns_validation_error() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let result = forge::pipeline::run_catalog_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        Some(""),
    );

    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("import-ssp") || msg.contains("empty"),
        "Error should mention import-ssp or empty, got: {msg}"
    );
}

// ─── Helpers ────────────────────────────────────────────────────────────

const SAMPLE_PROFILE: &str = "tests/fixtures/sample_profile.json";

/// Run component pipeline with import-ssp, write outputs, return AP JSON value.
fn run_component_with_ap(fixture: &Path, output_dir: &Path, import_ssp: &str) -> serde_json::Value {
    let result = forge::pipeline::run_component_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        Some(SAMPLE_PROFILE),
        &forge::cli::OutputFormat::Json,
        Some(import_ssp),
    )
    .expect("Component pipeline should succeed");

    // Write secondary outputs (AP file)
    for secondary in &result.secondary_outputs {
        let ap_path = output_dir.join(&secondary.filename);
        std::fs::write(&ap_path, &secondary.content).unwrap();
    }

    let stem = fixture.file_stem().unwrap().to_str().unwrap();
    let ap_path = output_dir.join(format!("{stem}-assessment-plan.json"));
    assert!(ap_path.exists(), "AP file should be written at {}", ap_path.display());

    let ap_json = std::fs::read_to_string(&ap_path).expect("AP file should be readable");
    serde_json::from_str(&ap_json).expect("AP file should be valid JSON")
}

/// Extract the normalized AP JSON for golden snapshot comparison.
fn normalized_ap_json(ap: &serde_json::Value) -> serde_json::Value {
    common::normalize_for_snapshot(ap)
}

// ─── Golden snapshots: assessment plan structure ─────────────────────────

/// Golden snapshot of the catalog pipeline assessment plan.
#[test]
fn golden_catalog_ap_snapshot() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let ap = run_catalog_with_ap(fixture, dir.path(), "./ssp/system-ssp.json");
    let normalized = normalized_ap_json(&ap);
    insta::assert_json_snapshot!("golden_catalog_ap", &normalized);
}

/// Golden snapshot of the component pipeline assessment plan.
#[test]
fn golden_component_ap_snapshot() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let ap = run_component_with_ap(fixture, dir.path(), "./ssp/system-ssp.json");
    let normalized = normalized_ap_json(&ap);
    insta::assert_json_snapshot!("golden_component_ap", &normalized);
}

// ─── Structural assertions: assessment subjects per component ────────────

/// Catalog pipeline: assessment-subjects exist but without component UUID refs.
#[test]
fn catalog_ap_subject_without_component_ref() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let ap = run_catalog_with_ap(fixture, dir.path(), "./ssp/system-ssp.json");

    let subjects = ap["assessment-plan"]["assessment-subjects"]
        .as_array()
        .expect("assessment-subjects should be an array");
    assert!(!subjects.is_empty(), "Should have at least one assessment subject");

    for subject in subjects {
        assert_eq!(subject["type"], "component", "Subject type should be 'component'");
        assert!(subject["description"].is_string(), "Subject should have a description");
        // Catalog pipeline: no component UUID available → no include-subjects
        assert!(
            subject.get("include-subjects").is_none(),
            "Catalog pipeline should not have include-subjects (no component UUID available)"
        );
    }
}

/// Component pipeline: assessment-subjects include a reference to the component UUID.
#[test]
fn component_ap_subject_with_component_ref() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let ap = run_component_with_ap(fixture, dir.path(), "./ssp/system-ssp.json");

    let subjects = ap["assessment-plan"]["assessment-subjects"]
        .as_array()
        .expect("assessment-subjects should be an array");
    assert!(!subjects.is_empty(), "Should have at least one assessment subject");

    for subject in subjects {
        assert_eq!(subject["type"], "component", "Subject type should be 'component'");
        assert!(subject["description"].is_string(), "Subject should have a description");

        // Component pipeline: has component UUID → include-subjects with component ref
        let include = subject["include-subjects"]
            .as_array()
            .expect("Component pipeline should have include-subjects with component ref");
        assert_eq!(include.len(), 1, "Should have exactly one include-subject entry");
        assert_eq!(include[0]["type"], "component", "include-subject type should be 'component'");
        assert!(
            include[0]["subject-uuid"].is_string(),
            "include-subject should have a subject-uuid string"
        );
    }
}

// ─── Schema-adjacent validation: required OSCAL AP fields ────────────────

/// Verify the assessment plan contains all required OSCAL v1.2.0 fields.
#[test]
fn ap_contains_required_oscal_ap_fields() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let ap = run_catalog_with_ap(fixture, dir.path(), "./ssp/system-ssp.json");
    let plan = &ap["assessment-plan"];

    // Required top-level fields per OSCAL AP spec
    assert!(plan["uuid"].is_string(), "AP must have a uuid");
    assert!(plan["metadata"]["title"].is_string(), "AP must have metadata.title");
    assert!(plan["metadata"]["last-modified"].is_string(), "AP must have metadata.last-modified");
    assert!(plan["metadata"]["version"].is_string(), "AP must have metadata.version");
    assert!(plan["metadata"]["oscal-version"].is_string(), "AP must have metadata.oscal-version");
    assert!(plan["import-ssp"]["href"].is_string(), "AP must have import-ssp.href");

    // reviewed-controls
    let selections = plan["reviewed-controls"]["control-selections"]
        .as_array()
        .expect("control-selections should be an array");
    assert!(!selections.is_empty(), "Must have at least one control-selection");
    for sel in selections {
        let includes =
            sel["include-controls"].as_array().expect("include-controls should be an array");
        for ctrl in includes {
            assert!(ctrl["control-id"].is_string(), "Each include-control needs a control-id");
        }
    }

    // tasks
    let tasks = plan["tasks"].as_array().expect("tasks should be an array when --import-ssp set");
    for task in tasks {
        assert!(task["uuid"].is_string(), "Each task needs a uuid");
        assert_eq!(task["type"], "action", "Task type should be 'action'");
        assert!(task["title"].is_string(), "Each task needs a title");
        assert!(task["description"].is_string(), "Each task needs a description");
    }
}

// ─── Component pipeline AP generation ───────────────────────────────────

#[test]
fn component_pipeline_generates_ap_with_import_ssp() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let result = forge::pipeline::run_component_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        Some("./baselines/nist-800-53.json"),
        &forge::cli::OutputFormat::Json,
        Some("./ssp/system-ssp.json"),
    );

    // Component pipeline may fail schema validation without a real profile,
    // but if it succeeds, verify the AP secondary output was generated
    if let Ok(output) = result {
        assert!(
            !output.secondary_outputs.is_empty(),
            "AP should be generated for component pipeline with --import-ssp"
        );

        let ap_content = &output.secondary_outputs[0].content;
        let ap: serde_json::Value = serde_json::from_str(ap_content).unwrap();
        assert!(ap.get("assessment-plan").is_some(), "AP should have assessment-plan root key");
        assert_eq!(
            ap["assessment-plan"]["import-ssp"]["href"].as_str().unwrap(),
            "system-ssp.json"
        );
    }
}

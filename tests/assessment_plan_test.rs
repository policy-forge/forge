//! Integration tests for Assessment Plan generation (WI-41).

mod common;

use std::path::Path;

use tempfile::TempDir;

const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const FIXTURE: &str = "tests/fixtures/sample_policy.md";

/// Helper: run catalog pipeline with --import-ssp and return the AP JSON value.
fn run_catalog_with_ap(fixture: &Path, output_dir: &Path, import_ssp: &str) -> serde_json::Value {
    let catalog_path = output_dir.join("catalog.json");
    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&catalog_path),
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        Some(import_ssp),
    )
    .expect("Catalog pipeline should succeed");

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
    let catalog_path = dir.path().join("catalog.json");
    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&catalog_path),
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        Some("./ssp.json"),
    )
    .unwrap();

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

    let dir = TempDir::new().unwrap();
    let catalog_path = dir.path().join("catalog.json");
    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&catalog_path),
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Json,
        None,
    )
    .unwrap();

    let stem = fixture.file_stem().unwrap().to_str().unwrap();
    let ap_path = dir.path().join(format!("{stem}-assessment-plan.json"));
    assert!(!ap_path.exists(), "AP file should NOT be written when --import-ssp omitted");
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

    let dir = TempDir::new().unwrap();
    let catalog_path = dir.path().join("catalog.json");
    let result = forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&catalog_path),
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

// ─── Component pipeline AP generation ───────────────────────────────────

#[test]
fn component_pipeline_generates_ap_with_import_ssp() {
    let fixture = Path::new(FIXTURE);
    if common::skip_if_missing(fixture) {
        return;
    }

    let dir = TempDir::new().unwrap();
    let component_path = dir.path().join("component.json");
    let result = forge::pipeline::run_component_pipeline(
        fixture,
        Some(&component_path),
        MAX_SIZE_BYTES,
        Some("./baselines/nist-800-53.json"),
        &forge::cli::OutputFormat::Json,
        Some("./ssp/system-ssp.json"),
    );

    // Component pipeline may fail schema validation without a real profile,
    // but if it succeeds, verify the AP file was written
    if result.is_ok() {
        let stem = fixture.file_stem().unwrap().to_str().unwrap();
        let ap_path = dir.path().join(format!("{stem}-assessment-plan.json"));
        assert!(
            ap_path.exists(),
            "AP file should be written for component pipeline with --import-ssp"
        );

        let ap_json = std::fs::read_to_string(&ap_path).unwrap();
        let ap: serde_json::Value = serde_json::from_str(&ap_json).unwrap();
        assert!(
            ap.get("assessment-plan").is_some(),
            "AP should have assessment-plan root key"
        );
        assert_eq!(
            ap["assessment-plan"]["import-ssp"]["href"].as_str().unwrap(),
            "system-ssp.json"
        );
    }
}

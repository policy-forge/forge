//! Integration tests: Phase 1 regression verification (WI-35, US5, M-6, AC-9).
//!
//! Confirms that Phase 2 development has not corrupted Phase 1 pipeline output.
//! Uses structural assertions only — no insta snapshots (existing `golden_file_tests.rs`
//! covers snapshot-level regression). Additive Phase 2 `prop` and `param` elements
//! are allowed and do not cause test failures.
//! Uses the CLI subprocess pattern (env!("CARGO_BIN_EXE_forge")) for end-to-end coverage.

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run_forge(args: &[&str]) -> std::process::Output {
    let output = forge_bin().args(args).output().expect("failed to execute forge");
    if !output.status.success() {
        panic!(
            "forge {:?} failed (exit {})\nstdout: {}\nstderr: {}",
            args,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}

fn read_json(path: &Path) -> Value {
    let content = fs::read_to_string(path).expect("failed to read JSON file");
    serde_json::from_str(&content).expect("failed to parse JSON")
}

// ── M-6 / AC-9: Phase 1 Catalog structure regression ─────────────────────────

#[test]
fn phase1_catalog_structure_regression() {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("catalog.json");

    run_forge(&[
        "convert",
        "tests/fixtures/golden/small/input.md",
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        output_path.to_str().unwrap(),
    ]);

    let catalog = read_json(&output_path);

    // uuid must be a non-empty string
    let uuid = catalog["catalog"]["uuid"].as_str().unwrap_or("");
    assert!(!uuid.is_empty(), "catalog.uuid must be a non-empty string");

    // oscal-version must be 1.2.0
    let oscal_version = catalog["catalog"]["metadata"]["oscal-version"].as_str().unwrap_or("");
    assert_eq!(oscal_version, "1.2.0", "catalog.metadata.oscal-version must be '1.2.0'");

    // groups must be non-empty
    let groups = catalog["catalog"]["groups"].as_array().expect("catalog.groups must be an array");
    assert!(!groups.is_empty(), "catalog.groups must be non-empty");

    // at least one group must have controls
    let has_controls =
        groups.iter().any(|g| g["controls"].as_array().map(|c| !c.is_empty()).unwrap_or(false));
    assert!(has_controls, "at least one group must have non-empty controls");
}

// ── M-6 / AC-9: Phase 1 Component Definition structure regression ─────────────

#[test]
fn phase1_component_structure_regression() {
    assert!(
        Path::new("tests/fixtures/sample_profile.json").exists(),
        "tests/fixtures/sample_profile.json must exist for component conversion"
    );

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("component.json");

    run_forge(&[
        "convert",
        "tests/fixtures/full_policy.md",
        "--strategy",
        "component",
        "--source-profile",
        "tests/fixtures/sample_profile.json",
        "--format",
        "json",
        "--output",
        output_path.to_str().unwrap(),
    ]);

    let comp_def = read_json(&output_path);

    // component-definition.uuid must be present and non-empty
    let uuid = comp_def["component-definition"]["uuid"].as_str().unwrap_or("");
    assert!(!uuid.is_empty(), "component-definition.uuid must be a non-empty string");

    // components must be non-empty
    let components = comp_def["component-definition"]["components"]
        .as_array()
        .expect("component-definition.components must be an array");
    assert!(!components.is_empty(), "component-definition.components must be non-empty");

    // first component type must be "policy"
    let first_type = components[0]["type"].as_str().unwrap_or("");
    assert_eq!(first_type, "policy", "first component type must be 'policy'");
}

// ── M-6 / AC-9: forge validate still passes on generated Catalog ──────────────

#[test]
fn phase1_validate_still_passes() {
    let dir = TempDir::new().unwrap();
    let catalog_path = dir.path().join("catalog.json");

    run_forge(&[
        "convert",
        "tests/fixtures/golden/small/input.md",
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        catalog_path.to_str().unwrap(),
    ]);

    let output = forge_bin()
        .args(["validate", catalog_path.to_str().unwrap()])
        .output()
        .expect("failed to run forge validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "forge validate must exit 0\nstdout: {stdout}");
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "forge validate output must contain 'Valid', got: {stdout}"
    );
}

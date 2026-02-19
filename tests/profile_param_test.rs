//! Integration and CLI tests for `forge profile --set-param` (WI-31).
//!
//! Covers: US1 (single param), US2 (multiple params + aggregation), US3 (structural validity).
//! TDD: tests written before implementation where applicable; all pass GREEN with Phase 3 impl.

use std::io::Write;
use std::process::Command;

use clap::Parser;
use tempfile::NamedTempFile;

use forge::cli::{Cli, Commands};

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Run `forge profile` with the given arguments and return (exit_code, stdout, stderr).
fn run_profile(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .arg("profile")
        .args(args)
        .output()
        .expect("Failed to execute forge binary");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (exit_code, stdout, stderr)
}

/// Write a minimal OSCAL Catalog JSON to a temp file and return the file handle.
fn temp_catalog_file() -> NamedTempFile {
    let content = r#"{
    "catalog": {
        "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
        "metadata": {
            "title": "Test Catalog",
            "last-modified": "2026-01-01T00:00:00Z",
            "version": "1.0",
            "oscal-version": "1.2.0"
        },
        "groups": []
    }
}"#;
    let mut f = NamedTempFile::with_suffix(".json").expect("Failed to create temp file");
    f.write_all(content.as_bytes()).expect("Failed to write temp file");
    f.flush().expect("Failed to flush temp file");
    f
}

// ─── T014: CLI parse tests for --set-param (US1) ────────────────────────────

#[test]
fn cli_parse_single_set_param() {
    let cli = Cli::try_parse_from([
        "forge",
        "profile",
        "--catalog",
        "cat.json",
        "--include",
        "AC-1",
        "--set-param",
        "POL-AC-001_prm",
        "60 days",
    ])
    .expect("Should parse --set-param with two arguments");

    if let Commands::Profile { set_params, .. } = cli.command {
        assert_eq!(set_params, vec!["POL-AC-001_prm", "60 days"]);
    } else {
        panic!("Expected Profile command");
    }
}

// ─── T023: CLI parse test for two --set-param flags (US2) ────────────────────

#[test]
fn cli_parse_two_set_params() {
    let cli = Cli::try_parse_from([
        "forge",
        "profile",
        "--catalog",
        "cat.json",
        "--include",
        "AC-1,AC-2",
        "--set-param",
        "id1",
        "val1",
        "--set-param",
        "id2",
        "val2",
    ])
    .expect("Should parse two --set-param flags");

    if let Commands::Profile { set_params, .. } = cli.command {
        assert_eq!(set_params, vec!["id1", "val1", "id2", "val2"]);
    } else {
        panic!("Expected Profile command");
    }
}

#[test]
fn cli_parse_set_param_space_in_value() {
    let cli = Cli::try_parse_from([
        "forge",
        "profile",
        "--catalog",
        "cat.json",
        "--include",
        "AC-1",
        "--set-param",
        "prm",
        "at least 60 days",
    ])
    .expect("Should parse --set-param with space in value");

    if let Commands::Profile { set_params, .. } = cli.command {
        assert_eq!(set_params, vec!["prm", "at least 60 days"]);
    } else {
        panic!("Expected Profile command");
    }
}

// ─── T017: Integration test — single --set-param (US1) ──────────────────────

#[test]
fn integration_single_set_param_produces_modify_section() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_profile(&[
        "--catalog",
        catalog_path,
        "--include",
        "AC-1",
        "--set-param",
        "POL-AC-001_prm",
        "60 days",
    ]);

    assert_eq!(exit_code, 0, "Expected exit 0.\nstdout: {stdout}\nstderr: {stderr}");

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Output must be valid JSON");

    let set_params = &json["profile"]["modify"]["set-parameters"];
    assert!(set_params.is_array(), "modify.set-parameters must be an array");
    assert_eq!(set_params.as_array().unwrap().len(), 1);
    assert_eq!(set_params[0]["param-id"], "POL-AC-001_prm");
    assert_eq!(set_params[0]["values"][0], "60 days");
}

// ─── T024: Integration test — two distinct --set-param flags (US2) ──────────

#[test]
fn integration_two_distinct_set_params_alphabetical() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_profile(&[
        "--catalog",
        catalog_path,
        "--include",
        "AC-1",
        "--set-param",
        "zzz_prm",
        "v1",
        "--set-param",
        "aaa_prm",
        "v2",
    ]);

    assert_eq!(exit_code, 0, "stdout: {stdout}\nstderr: {stderr}");

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let set_params = json["profile"]["modify"]["set-parameters"].as_array().unwrap();
    assert_eq!(set_params.len(), 2);
    // Alphabetical order: aaa_prm before zzz_prm
    assert_eq!(set_params[0]["param-id"], "aaa_prm");
    assert_eq!(set_params[1]["param-id"], "zzz_prm");
}

// ─── T025: Integration test — duplicate param-id aggregation (US2) ──────────

#[test]
fn integration_duplicate_param_id_aggregated() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) = run_profile(&[
        "--catalog",
        catalog_path,
        "--include",
        "AC-1",
        "--set-param",
        "prm1",
        "60 days",
        "--set-param",
        "prm1",
        "quarterly",
    ]);

    assert_eq!(exit_code, 0, "stdout: {stdout}\nstderr: {stderr}");

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let set_params = json["profile"]["modify"]["set-parameters"].as_array().unwrap();
    assert_eq!(set_params.len(), 1, "Duplicate param-id should be aggregated into one entry");
    assert_eq!(set_params[0]["param-id"], "prm1");
    let values = set_params[0]["values"].as_array().unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0], "60 days");
    assert_eq!(values[1], "quarterly");
}

// ─── T027: JSON structure — modify is sibling of imports (US3) ─────────────

#[test]
fn json_structure_modify_is_sibling_of_imports() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, _stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "AC-1", "--set-param", "prm", "val"]);
    assert_eq!(exit_code, 0);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let profile = &json["profile"];
    // modify must be a direct child of "profile", same level as imports and metadata
    assert!(profile.get("imports").is_some(), "profile must have 'imports'");
    assert!(profile.get("metadata").is_some(), "profile must have 'metadata'");
    assert!(profile.get("modify").is_some(), "profile must have 'modify' as direct child");
}

// ─── T028: Each set-parameters entry has param-id (string) and values (array) ──

#[test]
fn json_structure_set_parameters_entry_fields() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, _stderr) = run_profile(&[
        "--catalog",
        catalog_path,
        "--include",
        "AC-1",
        "--set-param",
        "POL-AC-001_prm",
        "60 days",
        "--set-param",
        "POL-AC-002_prm",
        "quarterly",
    ]);
    assert_eq!(exit_code, 0);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let set_params = json["profile"]["modify"]["set-parameters"].as_array().unwrap();
    for entry in set_params {
        assert!(
            entry.get("param-id").and_then(|v| v.as_str()).is_some(),
            "Each entry must have a string 'param-id'"
        );
        assert!(
            entry.get("values").and_then(|v| v.as_array()).is_some(),
            "Each entry must have an array 'values'"
        );
        for val in entry["values"].as_array().unwrap() {
            assert!(val.is_string(), "Each element of 'values' must be a string");
        }
    }
}

// ─── T029: Backward-compat — no modify key without --set-param (US3) ────────

#[test]
fn backward_compat_no_set_param_no_modify_key() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, _stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "AC-1"]);
    assert_eq!(exit_code, 0);

    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(
        json["profile"].get("modify").is_none(),
        "Profile must NOT have a 'modify' key when no --set-param flags are provided"
    );
}

// ─── T030: Determinism — same inputs produce identical output ────────────────

#[test]
fn determinism_same_inputs_produce_identical_output() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let args = [
        "--catalog",
        catalog_path,
        "--include",
        "AC-1",
        "--set-param",
        "prm1",
        "val1",
        "--set-param",
        "prm1",
        "val2",
    ];

    // Run twice and compare stdout byte-for-byte
    let (_c1, stdout1, _e1) = run_profile(&args);
    let (_c2, stdout2, _e2) = run_profile(&args);

    // UUIDs differ across runs, so compare the modify section specifically
    let json1: serde_json::Value = serde_json::from_str(&stdout1).unwrap();
    let json2: serde_json::Value = serde_json::from_str(&stdout2).unwrap();

    assert_eq!(
        json1["profile"]["modify"], json2["profile"]["modify"],
        "modify section must be identical across runs"
    );
    assert_eq!(
        json1["profile"]["imports"], json2["profile"]["imports"],
        "imports section must be identical across runs"
    );
}

// ─── T020: Snapshot test — single --set-param output (US1) ─────────────────

#[test]
fn snapshot_single_set_param_modify_section() {
    use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile};

    let pairs = vec![("POL-AC-001_prm".to_string(), "60 days".to_string())];
    let profile = build_profile(
        "/tmp/catalog.json",
        vec!["AC-1".to_string()],
        SelectionMode::Include,
        &pairs,
    )
    .unwrap();
    let root = ProfileRoot { profile };
    let json = serde_json::to_value(&root).unwrap();

    // Snapshot the modify section (UUID differs per run, so we check modify deterministically)
    let modify = &json["profile"]["modify"];
    insta::assert_json_snapshot!("single_set_param_modify", modify);
}

// ─── C-2: --set-param without --include/--exclude exits 0, warns, empty imports ──

#[test]
fn c2_set_param_without_include_exits_zero_with_warning() {
    let catalog = temp_catalog_file();
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--set-param", "POL-AC-001_prm", "60 days"]);

    assert_eq!(exit_code, 0, "C-2: expected exit 0\nstdout: {stdout}\nstderr: {stderr}");
    assert!(stderr.contains("warning:"), "C-2: expected warning on stderr, got: {stderr}");

    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("C-2: output must be valid JSON");
    let imports = json["profile"]["imports"].as_array().expect("imports must be an array");
    assert!(imports.is_empty(), "C-2: imports must be empty when no selection flag given");

    let set_params = &json["profile"]["modify"]["set-parameters"];
    assert!(set_params.is_array(), "C-2: modify.set-parameters must be an array");
    assert_eq!(set_params[0]["param-id"], "POL-AC-001_prm");
    assert_eq!(set_params[0]["values"][0], "60 days");
}

// ─── T026: Snapshot test — multi-param output (US2) ─────────────────────────

#[test]
fn snapshot_multi_param_modify_section() {
    use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile};

    let pairs = vec![
        ("zzz_prm".to_string(), "v1".to_string()),
        ("aaa_prm".to_string(), "v2".to_string()),
        ("mmm_prm".to_string(), "v3".to_string()),
    ];
    let profile = build_profile(
        "/tmp/catalog.json",
        vec!["AC-1".to_string()],
        SelectionMode::Include,
        &pairs,
    )
    .unwrap();
    let root = ProfileRoot { profile };
    let json = serde_json::to_value(&root).unwrap();

    let modify = &json["profile"]["modify"];
    insta::assert_json_snapshot!("multi_param_modify", modify);
}

//! Integration tests for the summary dashboard feature (044).

use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn forge_binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

/// Run forge convert with --summary and return (output, tempdir, stderr).
/// Panics if the process cannot be spawned.
fn run_summary(strategy: &str, format: &str, ext: &str) -> (Output, TempDir, String) {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Fixture missing");

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join(format!("output.{ext}"));

    let mut args = vec![
        "convert",
        fixture.to_str().unwrap(),
        "--strategy",
        strategy,
        "--output",
        output_path.to_str().unwrap(),
        "--summary",
    ];
    if format != "json" {
        args.push("--format");
        args.push(format);
    }

    let result = forge_binary().args(&args).output().expect("Failed to execute forge");
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();
    (result, dir, stderr)
}

/// Run forge convert without --summary.
fn run_no_summary(strategy: &str, ext: &str) -> (Output, TempDir, String) {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Fixture missing");

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join(format!("output.{ext}"));

    let result = forge_binary()
        .args([
            "convert",
            fixture.to_str().unwrap(),
            "--strategy",
            strategy,
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute forge");
    let stderr = String::from_utf8_lossy(&result.stderr).to_string();
    (result, dir, stderr)
}

// T018 [US1]: catalog pipeline with --summary shows core statistics
#[test]
fn summary_dashboard_catalog_shows_statistics() {
    let (result, dir, stderr) = run_summary("catalog", "json", "json");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(stderr.contains("Sections parsed:"), "Missing sections count in: {stderr}");
    assert!(stderr.contains("Requirements:"), "Missing requirements count in: {stderr}");
    assert!(stderr.contains("Controls generated:"), "Missing controls count in: {stderr}");
    assert!(dir.path().join("output.json").exists(), "Output file should exist");
}

// T019 [US1]: without --summary, no dashboard printed
#[test]
fn no_summary_flag_no_dashboard() {
    let (result, _dir, stderr) = run_no_summary("catalog", "json");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        !stderr.contains("FORGE Conversion Summary"),
        "Dashboard should NOT appear without --summary flag. Got: {stderr}"
    );
}

// T020 [US1]: --summary does not alter output artifact
#[test]
fn summary_flag_does_not_alter_artifact() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Fixture missing");

    let dir = TempDir::new().unwrap();
    let output_with = dir.path().join("with_summary.json");
    let output_without = dir.path().join("without_summary.json");

    // Run without --summary
    let result_without = forge_binary()
        .args([
            "convert",
            fixture.to_str().unwrap(),
            "--strategy",
            "catalog",
            "--output",
            output_without.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute forge");
    assert!(result_without.status.success());

    // Run with --summary
    let result_with = forge_binary()
        .args([
            "convert",
            fixture.to_str().unwrap(),
            "--strategy",
            "catalog",
            "--output",
            output_with.to_str().unwrap(),
            "--summary",
        ])
        .output()
        .expect("Failed to execute forge");
    assert!(result_with.status.success());

    let content_with = std::fs::read_to_string(&output_with).unwrap();
    let content_without = std::fs::read_to_string(&output_without).unwrap();

    // Parse both as JSON, normalize volatile fields, and compare full structure
    let mut json_with: serde_json::Value = serde_json::from_str(&content_with).unwrap();
    let mut json_without: serde_json::Value = serde_json::from_str(&content_without).unwrap();

    normalize_artifact(&mut json_with);
    normalize_artifact(&mut json_without);

    assert_eq!(json_with, json_without, "Artifacts should be identical with/without --summary");
}

/// Remove volatile fields (UUIDs, timestamps) so two independent conversions
/// of the same input can be compared for structural equality.
fn normalize_artifact(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            // Remove keys that change between runs
            map.remove("uuid");
            map.remove("last-modified");
            // Recurse into remaining values
            for v in map.values_mut() {
                normalize_artifact(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                normalize_artifact(v);
            }
        }
        _ => {}
    }
}

// T026 [US2]: catalog with --summary shows validation PASSED
#[test]
fn summary_dashboard_shows_validation_passed() {
    let (result, _dir, stderr) = run_summary("catalog", "json", "json");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        stderr.contains("Validation:") && stderr.contains("PASSED"),
        "Should show validation PASSED in: {stderr}"
    );
}

// T029 [US3]: dashboard shows mapping coverage with counts
#[test]
fn summary_dashboard_shows_mapping_coverage() {
    let (result, _dir, stderr) = run_summary("catalog", "json", "json");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(stderr.contains("Mapping coverage:"), "Should show mapping coverage in: {stderr}");
    assert!(stderr.contains('%'), "Mapping coverage should include percentage in: {stderr}");
}

// T033 [US4]: dashboard shows strategy and output path
#[test]
fn summary_dashboard_shows_context() {
    let (result, _dir, stderr) = run_summary("catalog", "json", "json");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        stderr.contains("Strategy:") && stderr.contains("catalog"),
        "Should show strategy 'catalog' in: {stderr}"
    );
    assert!(stderr.contains("Output:"), "Should show output path in: {stderr}");
}

// T037: component pipeline with --summary
#[test]
fn summary_dashboard_component_pipeline() {
    let (result, _dir, stderr) = run_summary("component", "json", "json");

    // Component pipeline may succeed or fail depending on schema validation.
    // Both outcomes are valid — verify the correct behavior for each.
    if result.status.success() {
        assert!(
            stderr.contains("Strategy:") && stderr.contains("component"),
            "Should show strategy 'component' in: {stderr}"
        );
        assert!(stderr.contains("Sections parsed:"), "Missing sections in: {stderr}");
    } else {
        // On validation failure (EC-5), dashboard must NOT appear
        assert!(
            !stderr.contains("FORGE Conversion Summary"),
            "Dashboard should NOT appear on validation failure (EC-5). Got: {stderr}"
        );
    }
}

// T038: dashboard contains only aggregate counts, no policy content
#[test]
fn summary_dashboard_no_policy_content_leaked() {
    let (result, _dir, stderr) = run_summary("catalog", "json", "json");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        !stderr.contains("must authenticate"),
        "Dashboard should not contain policy content (SEC-1)"
    );
    assert!(
        !stderr.contains("multi-factor authentication"),
        "Dashboard should not contain policy content (SEC-1)"
    );
    assert!(!stderr.contains("AES-256"), "Dashboard should not contain policy content (SEC-1)");
}

// T039: --summary with XML format
#[test]
fn summary_dashboard_with_xml_format() {
    let (result, dir, stderr) = run_summary("catalog", "xml", "xml");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        stderr.contains("FORGE Conversion Summary"),
        "Dashboard should appear with XML format in: {stderr}"
    );
    assert!(dir.path().join("output.xml").exists(), "XML output file should exist");
}

// T039b: --summary with YAML format
#[test]
fn summary_dashboard_with_yaml_format() {
    let (result, dir, stderr) = run_summary("catalog", "yaml", "yaml");

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        stderr.contains("FORGE Conversion Summary"),
        "Dashboard should appear with YAML format in: {stderr}"
    );
    assert!(dir.path().join("output.yaml").exists(), "YAML output file should exist");
}

// T044: conversion error with --summary suppresses dashboard
#[test]
fn conversion_error_suppresses_dashboard() {
    let result = forge_binary()
        .args(["convert", "nonexistent_file_12345.md", "--strategy", "catalog", "--summary"])
        .output()
        .expect("Failed to execute forge");

    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(!result.status.success(), "Should fail on non-existent file");
    assert!(
        !stderr.contains("FORGE Conversion Summary"),
        "Dashboard should NOT appear on conversion error (EC-5). Got: {stderr}"
    );
}

// --quiet suppresses --summary dashboard
#[test]
fn quiet_suppresses_summary_dashboard() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Fixture missing");

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("output.json");

    let result = forge_binary()
        .args([
            "-q",
            "convert",
            fixture.to_str().unwrap(),
            "--strategy",
            "catalog",
            "--output",
            output_path.to_str().unwrap(),
            "--summary",
        ])
        .output()
        .expect("Failed to execute forge");

    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(result.status.success(), "Command failed: {stderr}");
    assert!(
        !stderr.contains("FORGE Conversion Summary"),
        "Dashboard should NOT appear when --quiet is set. Got: {stderr}"
    );
    assert!(output_path.exists(), "Output file should still be written");
}

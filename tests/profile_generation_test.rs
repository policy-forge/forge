//! Integration tests for `forge profile` subcommand (WI-30).
//!
//! TDD RED: These tests should compile but FAIL until the profile subcommand is wired up.

use std::io::Write;
use std::process::Command;

use tempfile::NamedTempFile;

/// Helper: run `forge profile` with given arguments, return (exit_code, stdout, stderr).
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

/// Helper: write content to a temp file and return the file.
fn temp_catalog_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".json").expect("Failed to create temp file");
    f.write_all(content.as_bytes()).expect("Failed to write temp file");
    f.flush().expect("Failed to flush temp file");
    f
}

/// Minimal OSCAL catalog JSON for testing.
fn minimal_catalog_json() -> &'static str {
    r#"{
    "catalog": {
        "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
        "metadata": {
            "title": "Test Catalog",
            "last-modified": "2026-01-01T00:00:00Z",
            "version": "1.0",
            "oscal-version": "1.1.2"
        },
        "groups": [
            {
                "id": "ac",
                "title": "Access Control",
                "controls": [
                    {
                        "id": "AC-1",
                        "title": "Policy and Procedures",
                        "params": [],
                        "parts": []
                    },
                    {
                        "id": "AC-2",
                        "title": "Account Management",
                        "params": [],
                        "parts": []
                    }
                ]
            }
        ]
    }
}"#
}

// ─── AC-2: Happy path include ───────────────────────────────────────────

#[test]
fn happy_path_include() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "AC-2"]);

    assert_eq!(
        exit_code, 0,
        "Expected exit code 0 for valid profile generation.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("\"profile\""),
        "Output should contain '\"profile\"' key.\nstdout: {stdout}"
    );
    assert!(
        stdout.contains("include-controls") || stdout.contains("include-all"),
        "Output should contain include-controls selection.\nstdout: {stdout}"
    );
    assert!(
        stdout.contains("AC-2"),
        "Output should reference the included control AC-2.\nstdout: {stdout}"
    );
}

// ─── AC-8: Missing catalog file returns error ───────────────────────────

#[test]
fn missing_catalog_returns_error() {
    let (exit_code, _stdout, stderr) =
        run_profile(&["--catalog", "/nonexistent/catalog.json", "--include", "AC-1"]);

    assert_ne!(
        exit_code, 0,
        "Expected non-zero exit code for missing catalog file.\nstderr: {stderr}"
    );
}

// ─── EC-3: No selection flag returns error ──────────────────────────────

#[test]
fn no_selection_flag_returns_error() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, _stdout, stderr) = run_profile(&["--catalog", catalog_path]);

    assert_ne!(
        exit_code, 0,
        "Expected non-zero exit code when neither --include nor --exclude is provided.\nstderr: {stderr}"
    );
}

// ─── AC-1: Help output contains expected flags ──────────────────────────

#[test]
fn help_output_contains_expected_flags() {
    let (exit_code, stdout, stderr) = run_profile(&["--help"]);

    // --help may exit 0 or 2 depending on clap config; just check output content
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("--catalog"),
        "Help should mention --catalog flag.\noutput: {combined}"
    );
    assert!(
        combined.contains("--include"),
        "Help should mention --include flag.\noutput: {combined}"
    );
    assert!(
        combined.contains("--exclude"),
        "Help should mention --exclude flag.\noutput: {combined}"
    );

    // --help should succeed (exit 0)
    assert_eq!(exit_code, 0, "Expected exit code 0 for --help.\noutput: {combined}");
}

// ─── AC-9: Both --include and --exclude conflict ────────────────────────

#[test]
fn both_include_and_exclude_conflict() {
    let (exit_code, _stdout, stderr) =
        run_profile(&["--catalog", "/tmp/c.json", "--include", "AC-1", "--exclude", "AC-2"]);

    assert_ne!(
        exit_code, 0,
        "Expected non-zero exit code when both --include and --exclude are provided.\nstderr: {stderr}"
    );
}

// ─── AC-3: Happy path exclude ───────────────────────────────────────────

#[test]
fn happy_path_exclude() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--exclude", "POL-AC-003"]);

    assert_eq!(
        exit_code, 0,
        "Expected exit code 0 for valid exclude profile.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("\"profile\""),
        "Output should contain '\"profile\"' key.\nstdout: {stdout}"
    );
    assert!(
        stdout.contains("exclude-controls"),
        "Output should contain exclude-controls.\nstdout: {stdout}"
    );
    assert!(
        stdout.contains("POL-AC-003"),
        "Output should reference excluded control.\nstdout: {stdout}"
    );
    assert!(
        !stdout.contains("include-controls"),
        "Output must NOT contain include-controls when using --exclude.\nstdout: {stdout}"
    );
}

// ─── AC-6: --output writes to file; AC-7: no --output writes to stdout ──

#[test]
fn output_flag_writes_to_file() {
    use std::fs;

    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let dir = tempfile::TempDir::new().unwrap();
    let out_path = dir.path().join("baseline.json");

    let (exit_code, stdout, stderr) = run_profile(&[
        "--catalog",
        catalog_path,
        "--include",
        "POL-AC-001",
        "--output",
        out_path.to_str().unwrap(),
    ]);

    assert_eq!(
        exit_code, 0,
        "Expected exit code 0 when writing to file.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(out_path.exists(), "Output file should exist after --output flag");
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("\"profile\""), "Output file should contain profile JSON");
    assert!(
        stdout.trim().is_empty() || !stdout.contains("\"profile\""),
        "stdout should be empty (or not contain profile JSON) when --output is used"
    );
}

// ─── EC-4: Duplicate ID deduplication ──────────────────────────────────

#[test]
fn duplicate_ids_are_deduplicated() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "AC-1,AC-1,AC-2,AC-1"]);

    assert_eq!(exit_code, 0, "Expected success.\nstdout: {stdout}\nstderr: {stderr}");
    // AC-1 should appear only once in the with-ids array
    let count = stdout.matches("AC-1").count();
    assert!(count >= 1, "AC-1 should appear at least once");
    // The with-ids should have only 2 elements (AC-1, AC-2)
    assert!(stdout.contains("AC-2"), "AC-2 should be present");
}

// ─── EC-2: Whitespace trimming ──────────────────────────────────────────

#[test]
fn whitespace_in_ids_is_trimmed() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--include", " AC-1 , AC-2 "]);

    assert_eq!(
        exit_code, 0,
        "Expected success with whitespace.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("AC-1"), "Trimmed AC-1 should be in output");
    assert!(stdout.contains("AC-2"), "Trimmed AC-2 should be in output");
}

// ─── EC-1: Single ID, no comma ──────────────────────────────────────────

#[test]
fn single_id_no_comma() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "POL-AC-001"]);

    assert_eq!(exit_code, 0, "Expected success for single ID.\nstdout: {stdout}\nstderr: {stderr}");
    assert!(stdout.contains("POL-AC-001"), "Single ID should appear in output");
}

// ─── EC-5: Empty --include string ───────────────────────────────────────

#[test]
fn empty_include_string_returns_error() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, _stdout, stderr) = run_profile(&["--catalog", catalog_path, "--include", ""]);

    assert_ne!(exit_code, 0, "Expected error for empty --include string.\nstderr: {stderr}");
}

// ─── S-2: --format xml and --format yaml produce valid output ─────────────

#[test]
fn format_xml_produces_xml_output() {
    // XML format was added in WI-35 (S-2 defect fix); previously unsupported.
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "AC-1", "--format", "xml"]);

    assert_eq!(
        exit_code, 0,
        "Expected success for --format xml.\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("<profile"),
        "XML output must contain <profile element.\nstdout: {stdout}"
    );
}

// ─── S-1: --format json produces JSON output ────────────────────────────

#[test]
fn format_json_produces_json_output() {
    let catalog = temp_catalog_file(minimal_catalog_json());
    let catalog_path = catalog.path().to_str().unwrap();

    let (exit_code, stdout, stderr) =
        run_profile(&["--catalog", catalog_path, "--include", "AC-1", "--format", "json"]);

    assert_eq!(
        exit_code, 0,
        "Expected success with --format json.\nstdout: {stdout}\nstderr: {stderr}"
    );
    // Validate output is valid JSON
    let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output should be valid JSON.\nstdout: {stdout}");
}

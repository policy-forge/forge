//! Integration tests for `forge validate` (WI-19).

use std::io::Write;
use std::process::Command;

use tempfile::NamedTempFile;

/// Helper: run `forge validate` with args, return (stdout, stderr, exit code).
fn run_validate(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .arg("validate")
        .args(args)
        .output()
        .expect("Failed to execute forge binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

/// Helper: write content to a temp file and return the file.
fn temp_json_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("Failed to create temp file");
    f.write_all(content.as_bytes()).expect("Failed to write temp file");
    f.flush().expect("Failed to flush temp file");
    f
}

// --- US1: Valid artifact validation (AC-1, AC-2) ---

#[test]
fn validate_valid_catalog_exits_0() {
    let content = r#"{
        "catalog": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "metadata": {
                "title": "Test Catalog",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, _stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 0, "Expected exit 0 for valid catalog");
    assert!(stdout.contains("Valid"), "Expected 'Valid' in output, got: {stdout}");
}

#[test]
fn validate_valid_component_definition_exits_0() {
    let content = r#"{
        "component-definition": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "metadata": {
                "title": "Test Component",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, _stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 0, "Expected exit 0 for valid component definition");
    assert!(stdout.contains("Valid"), "Expected 'Valid' in output, got: {stdout}");
}

// --- US1: Invalid artifact validation (AC-3, AC-4) ---

#[test]
fn validate_invalid_catalog_exits_1_with_errors() {
    let content = r#"{
        "catalog": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d"
        }
    }"#;
    let file = temp_json_file(content);
    let (_stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 3, "Expected exit 3 for schema validation errors");
    assert!(
        stderr.contains("Validation failed") || stderr.contains("validation error(s)"),
        "Expected validation failure report in stderr, got: {stderr}"
    );
}

// --- US1: Auto-detection (AC-4) ---

#[test]
fn validate_auto_detects_catalog_model_type() {
    let content = r#"{
        "catalog": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "metadata": {
                "title": "Test",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, _stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(stdout.contains("catalog"), "Expected 'catalog' model type in output");
}

#[test]
fn validate_auto_detects_component_definition_model_type() {
    let content = r#"{
        "component-definition": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "metadata": {
                "title": "Test",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, _stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 0);
    assert!(
        stdout.contains("component-definition"),
        "Expected 'component-definition' model type in output"
    );
}

// --- Edge cases (T018) ---

#[test]
fn validate_nonexistent_file_returns_error() {
    let (_stdout, stderr, code) = run_validate(&["/nonexistent/path/artifact.json"]);
    assert_ne!(code, 0, "Expected non-zero exit for missing file");
    assert!(
        stderr.contains("Failed to read") || stderr.contains("error"),
        "Expected descriptive error for missing file, got: {stderr}"
    );
}

#[test]
fn validate_empty_file_returns_error() {
    let file = temp_json_file("");
    let (_stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_ne!(code, 0, "Expected non-zero exit for empty file");
    assert!(stderr.contains("empty"), "Expected 'empty' in error for empty file, got: {stderr}");
}

#[test]
fn validate_non_json_file_returns_parse_error() {
    let file = temp_json_file("This is not JSON at all.");
    let (_stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_ne!(code, 0, "Expected non-zero exit for non-JSON file");
    assert!(
        stderr.contains("JSON") || stderr.contains("parse"),
        "Expected JSON parse error, got: {stderr}"
    );
}

#[test]
fn validate_unknown_model_type_suggests_schema_type() {
    let content = r#"{"profile": {}}"#;
    let file = temp_json_file(content);
    let (_stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_ne!(code, 0, "Expected non-zero exit for unknown model type");
    assert!(
        stderr.contains("--schema-type"),
        "Expected --schema-type guidance in error, got: {stderr}"
    );
}

#[test]
fn validate_multiple_violations_reports_all_errors() {
    // Missing both metadata and uuid — should report multiple errors
    let content = r#"{"catalog": {}}"#;
    let file = temp_json_file(content);
    let (_stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 3, "Expected exit 3 for schema validation errors");
    // Should report more than 1 error
    assert!(stderr.contains("error(s)"), "Expected error count, got: {stderr}");
}

// --- SEC-3: File size limit ---

#[test]
fn validate_oversized_file_returns_too_large_error() {
    // Create a temp file just over the 50MB limit
    let mut f = NamedTempFile::new().expect("Failed to create temp file");
    let chunk = vec![b' '; 1024 * 1024]; // 1MB chunk
    for _ in 0..51 {
        f.write_all(&chunk).expect("Failed to write chunk");
    }
    f.flush().expect("Failed to flush");

    let (_stdout, stderr, code) = run_validate(&[f.path().to_str().unwrap()]);
    assert_ne!(code, 0, "Expected non-zero exit for oversized file");
    assert!(
        stderr.contains("too large") || stderr.contains("limit"),
        "Expected file size error, got: {stderr}"
    );
}

// --- US3: --schema-type override (T030) ---

#[test]
fn validate_schema_type_override_forces_validation() {
    // Component definition JSON validated as catalog should fail
    let content = r#"{
        "component-definition": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "metadata": {
                "title": "Test",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (_stdout, stderr, code) =
        run_validate(&[file.path().to_str().unwrap(), "--schema-type", "catalog"]);
    assert_eq!(code, 3, "Expected exit 3 when component-definition validated as catalog");
    assert!(
        stderr.contains("Validation failed") || stderr.contains("validation error(s)"),
        "Expected validation failure report in stderr, got: {stderr}"
    );
}

// --- US3: External OSCAL artifacts (T028, T029) ---

#[test]
fn validate_external_catalog_artifact() {
    // Minimal but schema-valid catalog following NIST structure
    let content = r#"{
        "catalog": {
            "uuid": "c3d4e5f6-a7b8-4c9d-8e1f-2a3b4c5d6e7f",
            "metadata": {
                "title": "External Policy Catalog",
                "last-modified": "2026-02-13T12:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.0"
            },
            "groups": [
                {
                    "id": "ac",
                    "title": "Access Control",
                    "controls": [
                        {
                            "id": "ac-1",
                            "title": "Access Control Policy and Procedures"
                        }
                    ]
                }
            ]
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, _stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 0, "Expected external catalog to pass validation");
    assert!(stdout.contains("Valid"));
}

#[test]
fn validate_external_component_definition_artifact() {
    let content = r#"{
        "component-definition": {
            "uuid": "d4e5f6a7-b8c9-4d0e-9f1a-2b3c4d5e6f70",
            "metadata": {
                "title": "External Component Definition",
                "last-modified": "2026-02-13T12:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, _stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 0, "Expected external component definition to pass validation");
    assert!(stdout.contains("Valid"));
}

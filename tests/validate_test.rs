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
fn validate_legacy_catalog_json_reports_declared_and_schema_versions() {
    let file = temp_json_file(include_str!("fixtures/legacy/v1.2.0/catalog/catalog.json"));
    let (stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap(), "--format", "json"]);
    assert_eq!(code, 0, "legacy catalog should remain supported: {stderr}");
    let report: serde_json::Value =
        serde_json::from_str(&stdout).expect("validation stdout must be JSON");
    assert_eq!(report["model_type"], "catalog");
    assert_eq!(report["declared_oscal_version"], "1.2.0");
    assert_eq!(report["schema_version_used"], "1.2.3");
    assert_eq!(report["supported_input"], true);
    assert_eq!(report["is_valid"], true);
}

#[test]
fn validate_rejects_unsupported_declaration_and_names_available_baseline() {
    let content = r#"{
        "catalog": {
            "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
            "metadata": {
                "title": "Unsupported Catalog",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.3.0"
            }
        }
    }"#;
    let file = temp_json_file(content);
    let (_stdout, stderr, code) = run_validate(&[file.path().to_str().unwrap()]);
    assert_eq!(code, 3);
    assert!(stderr.contains("unsupported OSCAL version declaration '1.3.0'"));
    assert!(stderr.contains("available schema baseline is 1.2.3"));
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

#[test]
fn validate_valid_mapping_collection_exits_0_with_explicit_schema_type() {
    let content = r#"{
        "mapping-collection": {
            "uuid": "11111111-1111-4111-8111-111111111111",
            "metadata": {
                "title": "Reviewed mapping",
                "last-modified": "2026-08-22T17:00:00Z",
                "version": "1.0.0",
                "oscal-version": "1.2.3"
            },
            "provenance": {
                "method": "human",
                "matching-rationale": "semantic",
                "status": "draft",
                "mapping-description": "Human-reviewed relationship set."
            },
            "mappings": [{
                "uuid": "22222222-2222-4222-8222-222222222222",
                "source-resource": {"type": "catalog", "href": "source.json"},
                "target-resource": {"type": "catalog", "href": "target.json"},
                "maps": [{
                    "uuid": "33333333-3333-4333-8333-333333333333",
                    "relationship": "subset-of",
                    "sources": [{"type": "control", "id-ref": "source-1"}],
                    "targets": [{"type": "control", "id-ref": "target-1"}]
                }]
            }]
        }
    }"#;
    let file = temp_json_file(content);
    let (stdout, stderr, code) =
        run_validate(&[file.path().to_str().unwrap(), "--schema-type", "mapping"]);
    assert_eq!(code, 0, "Expected valid mapping to pass validation: {stderr}");
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
    // Use a key that is not any recognized OSCAL root type
    let content = r#"{"assessment-plan": {}}"#;
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

#[test]
fn wi22_expected_catalog_artifacts_pass_schema_validation() {
    let catalog_paths = [
        "tests/fixtures/edge-cases/ec02-compound-atomic/expected-catalog.json",
        "tests/fixtures/edge-cases/ec03-empty-sections/expected-catalog.json",
        "tests/fixtures/edge-cases/ec04-missing-metadata/expected-catalog.json",
        "tests/fixtures/edge-cases/ec07-malformed-citation/expected-catalog.json",
        "tests/fixtures/edge-cases/ec-citation-unusual-positions/expected-catalog.json",
        "tests/fixtures/edge-cases/ec-parameter-like-content/expected-catalog.json",
    ];

    for path in catalog_paths {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read WI-22 catalog fixture {path}: {e}"));
        let json: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("Invalid JSON in WI-22 catalog fixture {path}: {e}"));
        let result =
            forge::validate::validate_artifact(&json, forge::validate::OscalModelType::Catalog)
                .unwrap_or_else(|e| {
                    panic!("Schema validation infrastructure failed for {path}: {e}")
                });
        assert!(result.is_valid, "WI-22 catalog fixture should be schema-valid: {path}");
    }
}

#[test]
fn wi22_expected_component_artifacts_pass_schema_validation() {
    let component_paths = [
        "tests/fixtures/edge-cases/ec02-compound-atomic/expected-component-definition.json",
        "tests/fixtures/edge-cases/ec-citation-unusual-positions/expected-component-definition.json",
        "tests/fixtures/edge-cases/ec-parameter-like-content/expected-component-definition.json",
    ];

    for path in component_paths {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read WI-22 component fixture {path}: {e}"));
        let json: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("Invalid JSON in WI-22 component fixture {path}: {e}"));
        let result = forge::validate::validate_artifact(
            &json,
            forge::validate::OscalModelType::ComponentDefinition,
        )
        .unwrap_or_else(|e| panic!("Schema validation infrastructure failed for {path}: {e}"));
        assert!(result.is_valid, "WI-22 component fixture should be schema-valid: {path}");
    }
}

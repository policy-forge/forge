//! CLI integration tests for `forge export` subcommand.
//!
//! Tests binary invocation via `std::process::Command`.

use std::process::Command;

use tempfile::TempDir;

const CATALOG_JSON: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.json");
const CATALOG_XML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.xml");
const CATALOG_YAML: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.yaml");

/// Helper: run `forge export` with given arguments, return (`exit_code`, stdout, stderr).
fn run_export(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["export"])
        .args(args)
        .output()
        .expect("Failed to execute forge binary");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (exit_code, stdout, stderr)
}

// ─── T037: CLI integration tests ─────────────────────────────────────────

#[test]
fn cli_export_json_to_xml_stdout() {
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_JSON, "--format", "xml"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert!(stdout.contains("<catalog"), "stdout should contain XML catalog");
    assert!(stdout.contains("xmlns"), "stdout should contain OSCAL namespace");
}

#[test]
fn cli_export_json_to_yaml_stdout() {
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_JSON, "--format", "yaml"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert!(stdout.contains("catalog:"), "stdout should contain YAML catalog key");
}

#[test]
fn cli_export_json_to_xml_output_file() {
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    let output_str = output.to_str().unwrap();

    let (exit_code, _stdout, _stderr) =
        run_export(&[CATALOG_JSON, "--format", "xml", "--output", output_str]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert!(output.exists(), "Output file should exist");
    let content = std::fs::read_to_string(&output).unwrap();
    assert!(content.contains("<catalog"), "Output file should contain XML catalog");
}

#[test]
fn cli_export_xml_to_json() {
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_XML, "--format", "json"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert!(stdout.contains("\"catalog\""), "stdout should contain JSON catalog key");
}

#[test]
fn cli_export_yaml_to_json() {
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_YAML, "--format", "json"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert!(stdout.contains("\"catalog\""), "stdout should contain JSON catalog key");
}

#[test]
fn cli_export_invalid_input_nonzero_exit() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path, r#"{"not_oscal": true}"#).unwrap();
    let path_str = path.to_str().unwrap();

    let (exit_code, _stdout, stderr) = run_export(&[path_str, "--format", "xml"]);
    assert_ne!(exit_code, 0, "Expected non-zero exit code for invalid input");
    assert!(
        stderr.contains("not a valid OSCAL") || stderr.contains("OSCAL"),
        "stderr should contain descriptive error. Got: {stderr}"
    );
}

#[test]
fn cli_export_nonexistent_file_nonzero_exit() {
    let (exit_code, _stdout, stderr) = run_export(&["nonexistent_file.json", "--format", "xml"]);
    assert_ne!(exit_code, 0, "Expected non-zero exit code for missing file");
    assert!(
        stderr.contains("not found") || stderr.contains("No such file"),
        "stderr should report file not found. Got: {stderr}"
    );
}

#[test]
fn cli_export_missing_format_arg() {
    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["export", CATALOG_JSON])
        .output()
        .expect("Failed to execute forge binary");

    let exit_code = output.status.code().unwrap_or(-1);
    assert_ne!(exit_code, 0, "Expected non-zero exit code for missing --format");
}

// ─── T046: Read-only output path test (EC-4) ─────────────────────────────

#[cfg(unix)]
#[test]
fn cli_export_read_only_output_path() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let readonly_dir = dir.path().join("readonly");
    std::fs::create_dir(&readonly_dir).unwrap();
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o444)).unwrap();

    let output = readonly_dir.join("out.xml");
    let output_str = output.to_str().unwrap();

    let (exit_code, _stdout, stderr) =
        run_export(&[CATALOG_JSON, "--format", "xml", "--output", output_str]);

    // Restore permissions for cleanup
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o755)).unwrap();

    assert_ne!(exit_code, 0, "Expected non-zero exit code for read-only path");
    assert!(
        stderr.contains("Permission denied") || stderr.contains("permission"),
        "stderr should report permission error. Got: {stderr}"
    );
}

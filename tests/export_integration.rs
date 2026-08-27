//! CLI integration tests for `forge export` subcommand.
//!
//! Tests binary invocation via `std::process::Command`.

use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

const CATALOG_JSON: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.json");
const CATALOG_XML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.xml");
const CATALOG_YAML: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.yaml");

fn require_fixture(path: &str) {
    assert!(Path::new(path).is_file(), "required export fixture is missing: {path}");
}

fn assert_catalog_json(content: &str) {
    let value: serde_json::Value =
        serde_json::from_str(content).expect("exported catalog JSON must parse");
    let catalog = &value["catalog"];
    assert!(catalog["metadata"]["title"].as_str().is_some_and(|title| !title.is_empty()));
    assert!(catalog["groups"].as_array().is_some_and(|groups| !groups.is_empty()));
    assert!(
        catalog["groups"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|group| group["controls"].as_array().into_iter().flatten())
            .any(|control| control["id"].as_str().is_some_and(|id| !id.is_empty()))
    );
}

fn assert_catalog_yaml(content: &str) {
    let value: serde_json::Value =
        serde_yaml::from_str(content).expect("exported catalog YAML must parse");
    assert_catalog_json(&serde_json::to_string(&value).expect("YAML value must serialize as JSON"));
}

fn assert_catalog_xml(content: &str) {
    let mut reader = quick_xml::Reader::from_str(content);
    let mut buffer = Vec::new();
    let (mut opened_catalog, mut closed_catalog, mut saw_control) = (false, false, false);
    loop {
        match reader.read_event_into(&mut buffer).expect("exported catalog XML must parse") {
            quick_xml::events::Event::Start(event) => {
                let name = event.name();
                opened_catalog |= name.as_ref() == b"catalog";
                saw_control |= name.as_ref() == b"control";
            }
            quick_xml::events::Event::End(event) => {
                closed_catalog |= event.name().as_ref() == b"catalog";
            }
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    assert!(
        opened_catalog && closed_catalog && saw_control,
        "catalog XML must contain catalog wrapper and control"
    );
}

/// Helper: run `forge export` with given arguments, return (`exit_code`, stdout, stderr).
fn run_export(args: &[&str]) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["export"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute forge binary");
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if child.try_wait().expect("failed to poll forge export").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("failed to kill timed-out forge export");
            let output =
                child.wait_with_output().expect("failed to collect timed-out forge export output");
            panic!(
                "forge export {:?} timed out after 120s\nstdout: {}\nstderr: {}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("failed to collect forge export output");
    let exit_code = output.status.code().unwrap_or_else(|| {
        panic!("forge export {:?} terminated without an exit code: {:?}", args, output.status)
    });
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (exit_code, stdout, stderr)
}

// ─── T037: CLI integration tests ─────────────────────────────────────────

#[test]
fn cli_export_json_to_xml_stdout() {
    require_fixture(CATALOG_JSON);
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_JSON, "--format", "xml"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert_catalog_xml(&stdout);
}

#[test]
fn cli_export_json_to_yaml_stdout() {
    require_fixture(CATALOG_JSON);
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_JSON, "--format", "yaml"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert_catalog_yaml(&stdout);
}

#[test]
fn cli_export_json_to_xml_output_file() {
    require_fixture(CATALOG_JSON);
    let dir = TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    let output_str = output.to_str().unwrap();

    let (exit_code, _stdout, _stderr) =
        run_export(&[CATALOG_JSON, "--format", "xml", "--output", output_str]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert!(output.exists(), "Output file should exist");
    let content = std::fs::read_to_string(&output).unwrap();
    assert_catalog_xml(&content);
}

#[test]
fn cli_export_xml_to_json() {
    require_fixture(CATALOG_XML);
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_XML, "--format", "json"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert_catalog_json(&stdout);
}

#[test]
fn cli_export_yaml_to_json() {
    require_fixture(CATALOG_YAML);
    let (exit_code, stdout, _stderr) = run_export(&[CATALOG_YAML, "--format", "json"]);
    assert_eq!(exit_code, 0, "Expected exit code 0");
    assert_catalog_json(&stdout);
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
        stderr.contains("not a valid OSCAL artifact"),
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
    require_fixture(CATALOG_JSON);
    let (exit_code, _stdout, _stderr) = run_export(&[CATALOG_JSON]);
    assert_ne!(exit_code, 0, "Expected non-zero exit code for missing --format");
}

// ─── T046: Read-only output path test (EC-4) ─────────────────────────────

#[cfg(unix)]
#[test]
fn cli_export_read_only_output_path() {
    use std::os::unix::fs::PermissionsExt;

    require_fixture(CATALOG_JSON);

    let dir = TempDir::new().unwrap();
    let readonly_dir = dir.path().join("readonly");
    std::fs::create_dir(&readonly_dir).unwrap();
    std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o444)).unwrap();

    // Mode bits do not restrict root (or CAP_DAC_OVERRIDE) — the default user
    // in many privileged CI containers — so probe whether the sandbox can
    // actually induce EACCES before asserting on it (F0836).
    if std::fs::write(readonly_dir.join(".probe"), b"x").is_ok() {
        std::fs::remove_file(readonly_dir.join(".probe")).ok();
        std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        eprintln!("skipping cli_export_read_only_output_path: running with write bypass (root?)");
        return;
    }

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

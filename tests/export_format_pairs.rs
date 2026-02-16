//! Comprehensive format-pair tests for the export pipeline.
//!
//! Tests all 9 format pairs (JSON/XML/YAML x JSON/XML/YAML) for both
//! Catalog and Component Definition model types (18 cases total).

use std::path::PathBuf;

use forge::cli::OutputFormat;
use forge::cli::export::export_artifact;

// ── Catalog: all 9 format pairs ──────────────────────────────────────────

#[test]
fn format_pair_catalog_json_to_json() {
    let input = PathBuf::from("tests/fixtures/export/catalog.json");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.json");
    assert!(export_artifact(&input, OutputFormat::Json, Some(&output)).is_ok());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    assert!(v.get("catalog").is_some());
}

#[test]
fn format_pair_catalog_json_to_xml() {
    let input = PathBuf::from("tests/fixtures/export/catalog.json");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    assert!(export_artifact(&input, OutputFormat::Xml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("<catalog"));
}

#[test]
fn format_pair_catalog_json_to_yaml() {
    let input = PathBuf::from("tests/fixtures/export/catalog.json");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.yaml");
    assert!(export_artifact(&input, OutputFormat::Yaml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("catalog:"));
}

#[test]
fn format_pair_catalog_xml_to_json() {
    let input = PathBuf::from("tests/fixtures/export/catalog.xml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.json");
    assert!(export_artifact(&input, OutputFormat::Json, Some(&output)).is_ok());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    assert!(v.get("catalog").is_some());
}

#[test]
fn format_pair_catalog_xml_to_xml() {
    let input = PathBuf::from("tests/fixtures/export/catalog.xml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    assert!(export_artifact(&input, OutputFormat::Xml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("<catalog"));
}

#[test]
fn format_pair_catalog_xml_to_yaml() {
    let input = PathBuf::from("tests/fixtures/export/catalog.xml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.yaml");
    assert!(export_artifact(&input, OutputFormat::Yaml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("catalog:"));
}

#[test]
fn format_pair_catalog_yaml_to_json() {
    let input = PathBuf::from("tests/fixtures/export/catalog.yaml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.json");
    assert!(export_artifact(&input, OutputFormat::Json, Some(&output)).is_ok());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    assert!(v.get("catalog").is_some());
}

#[test]
fn format_pair_catalog_yaml_to_xml() {
    let input = PathBuf::from("tests/fixtures/export/catalog.yaml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    assert!(export_artifact(&input, OutputFormat::Xml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("<catalog"));
}

#[test]
fn format_pair_catalog_yaml_to_yaml() {
    let input = PathBuf::from("tests/fixtures/export/catalog.yaml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.yaml");
    assert!(export_artifact(&input, OutputFormat::Yaml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("catalog:"));
}

// ── Component: all 9 format pairs ────────────────────────────────────────

#[test]
fn format_pair_component_json_to_json() {
    let input = PathBuf::from("tests/fixtures/export/component.json");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.json");
    assert!(export_artifact(&input, OutputFormat::Json, Some(&output)).is_ok());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    assert!(v.get("component-definition").is_some());
}

#[test]
fn format_pair_component_json_to_xml() {
    let input = PathBuf::from("tests/fixtures/export/component.json");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    assert!(export_artifact(&input, OutputFormat::Xml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("<component-definition"));
}

#[test]
fn format_pair_component_json_to_yaml() {
    let input = PathBuf::from("tests/fixtures/export/component.json");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.yaml");
    assert!(export_artifact(&input, OutputFormat::Yaml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("component-definition:"));
}

#[test]
fn format_pair_component_xml_to_json() {
    let input = PathBuf::from("tests/fixtures/export/component.xml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.json");
    assert!(export_artifact(&input, OutputFormat::Json, Some(&output)).is_ok());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    assert!(v.get("component-definition").is_some());
}

#[test]
fn format_pair_component_xml_to_xml() {
    let input = PathBuf::from("tests/fixtures/export/component.xml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    assert!(export_artifact(&input, OutputFormat::Xml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("<component-definition"));
}

#[test]
fn format_pair_component_xml_to_yaml() {
    let input = PathBuf::from("tests/fixtures/export/component.xml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.yaml");
    assert!(export_artifact(&input, OutputFormat::Yaml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("component-definition:"));
}

#[test]
fn format_pair_component_yaml_to_json() {
    let input = PathBuf::from("tests/fixtures/export/component.yaml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.json");
    assert!(export_artifact(&input, OutputFormat::Json, Some(&output)).is_ok());
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
    assert!(v.get("component-definition").is_some());
}

#[test]
fn format_pair_component_yaml_to_xml() {
    let input = PathBuf::from("tests/fixtures/export/component.yaml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.xml");
    assert!(export_artifact(&input, OutputFormat::Xml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("<component-definition"));
}

#[test]
fn format_pair_component_yaml_to_yaml() {
    let input = PathBuf::from("tests/fixtures/export/component.yaml");
    let dir = tempfile::TempDir::new().unwrap();
    let output = dir.path().join("out.yaml");
    assert!(export_artifact(&input, OutputFormat::Yaml, Some(&output)).is_ok());
    let c = std::fs::read_to_string(&output).unwrap();
    assert!(c.contains("component-definition:"));
}

//! Comprehensive format-pair tests for the export pipeline.
//!
//! Tests all 9 format pairs (JSON/XML/YAML x JSON/XML/YAML) for both
//! Catalog and Component Definition model types (18 cases total).

use std::path::Path;

use forge::cli::OutputFormat;
use forge::cli::export::export_artifact;

/// Run `export_artifact` and return the output file contents.
fn export_and_read(input: &str, format: OutputFormat) -> String {
    let input = Path::new(input);
    let dir = tempfile::TempDir::new().unwrap();
    let ext = match format {
        OutputFormat::Json => "json",
        OutputFormat::Xml => "xml",
        OutputFormat::Yaml => "yaml",
    };
    let output = dir.path().join(format!("out.{ext}"));
    export_artifact(input, format, Some(&output)).unwrap();
    std::fs::read_to_string(&output).unwrap()
}

// ── Catalog: all 9 format pairs ──────────────────────────────────────────

#[test]
fn format_pair_catalog_json_to_json() {
    let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&c).unwrap();
    assert!(v.get("catalog").is_some());
}

#[test]
fn format_pair_catalog_json_to_xml() {
    let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Xml);
    assert!(c.contains("<catalog"));
}

#[test]
fn format_pair_catalog_json_to_yaml() {
    let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Yaml);
    assert!(c.contains("catalog:"));
}

#[test]
fn format_pair_catalog_xml_to_json() {
    let c = export_and_read("tests/fixtures/export/catalog.xml", OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&c).unwrap();
    assert!(v.get("catalog").is_some());
}

#[test]
fn format_pair_catalog_xml_to_xml() {
    let c = export_and_read("tests/fixtures/export/catalog.xml", OutputFormat::Xml);
    assert!(c.contains("<catalog"));
}

#[test]
fn format_pair_catalog_xml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/catalog.xml", OutputFormat::Yaml);
    assert!(c.contains("catalog:"));
}

#[test]
fn format_pair_catalog_yaml_to_json() {
    let c = export_and_read("tests/fixtures/export/catalog.yaml", OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&c).unwrap();
    assert!(v.get("catalog").is_some());
}

#[test]
fn format_pair_catalog_yaml_to_xml() {
    let c = export_and_read("tests/fixtures/export/catalog.yaml", OutputFormat::Xml);
    assert!(c.contains("<catalog"));
}

#[test]
fn format_pair_catalog_yaml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/catalog.yaml", OutputFormat::Yaml);
    assert!(c.contains("catalog:"));
}

// ── Component: all 9 format pairs ────────────────────────────────────────

#[test]
fn format_pair_component_json_to_json() {
    let c = export_and_read("tests/fixtures/export/component.json", OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&c).unwrap();
    assert!(v.get("component-definition").is_some());
}

#[test]
fn format_pair_component_json_to_xml() {
    let c = export_and_read("tests/fixtures/export/component.json", OutputFormat::Xml);
    assert!(c.contains("<component-definition"));
}

#[test]
fn format_pair_component_json_to_yaml() {
    let c = export_and_read("tests/fixtures/export/component.json", OutputFormat::Yaml);
    assert!(c.contains("component-definition:"));
}

#[test]
fn format_pair_component_xml_to_json() {
    let c = export_and_read("tests/fixtures/export/component.xml", OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&c).unwrap();
    assert!(v.get("component-definition").is_some());
}

#[test]
fn format_pair_component_xml_to_xml() {
    let c = export_and_read("tests/fixtures/export/component.xml", OutputFormat::Xml);
    assert!(c.contains("<component-definition"));
}

#[test]
fn format_pair_component_xml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/component.xml", OutputFormat::Yaml);
    assert!(c.contains("component-definition:"));
}

#[test]
fn format_pair_component_yaml_to_json() {
    let c = export_and_read("tests/fixtures/export/component.yaml", OutputFormat::Json);
    let v: serde_json::Value = serde_json::from_str(&c).unwrap();
    assert!(v.get("component-definition").is_some());
}

#[test]
fn format_pair_component_yaml_to_xml() {
    let c = export_and_read("tests/fixtures/export/component.yaml", OutputFormat::Xml);
    assert!(c.contains("<component-definition"));
}

#[test]
fn format_pair_component_yaml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/component.yaml", OutputFormat::Yaml);
    assert!(c.contains("component-definition:"));
}

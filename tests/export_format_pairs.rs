//! Comprehensive format-pair tests for the export pipeline.
//!
//! Tests all 9 format pairs (JSON/XML/YAML x JSON/XML/YAML) for both
//! Catalog and Component Definition model types (18 cases total).

use std::path::Path;

use forge::cli::OutputFormat;
use forge::cli::export::export_artifact;
use serde_json::Value;

/// Run `export_artifact` and return the output file contents.
fn export_and_read(relative_path: &str, format: OutputFormat) -> String {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let dir = tempfile::TempDir::new().unwrap();
    let ext = match format {
        OutputFormat::Json => "json",
        OutputFormat::Xml => "xml",
        OutputFormat::Yaml => "yaml",
    };
    let output = dir.path().join(format!("out.{ext}"));
    export_artifact(&input, format, Some(&output)).unwrap();
    std::fs::read_to_string(&output).unwrap()
}

fn assert_catalog_structure(value: &Value) {
    let catalog = &value["catalog"];
    assert!(catalog["metadata"]["title"].as_str().is_some_and(|title| !title.is_empty()));
    let groups = catalog["groups"].as_array().expect("catalog.groups must be an array");
    assert!(!groups.is_empty(), "catalog.groups must not be empty");
    assert!(
        groups
            .iter()
            .flat_map(|group| group["controls"].as_array().into_iter().flatten())
            .any(|control| control["id"].as_str().is_some_and(|id| !id.is_empty()))
    );
}

fn assert_component_structure(value: &Value) {
    let component_definition = &value["component-definition"];
    assert!(
        component_definition["metadata"]["title"].as_str().is_some_and(|title| !title.is_empty())
    );
    let components = component_definition["components"]
        .as_array()
        .expect("component-definition.components must be an array");
    assert!(!components.is_empty(), "component-definition.components must not be empty");
    assert!(
        components
            .iter()
            .any(|component| component["uuid"].as_str().is_some_and(|uuid| !uuid.is_empty()))
    );
}

fn assert_xml_structure(content: &str, root: &[u8], nested: &[u8]) {
    let mut reader = quick_xml::Reader::from_str(content);
    let mut buffer = Vec::new();
    let (mut opened_root, mut closed_root, mut saw_nested) = (false, false, false);
    loop {
        match reader.read_event_into(&mut buffer).expect("exported XML must be well-formed") {
            quick_xml::events::Event::Start(event) => {
                let name = event.name();
                opened_root |= name.as_ref() == root;
                saw_nested |= name.as_ref() == nested;
            }
            quick_xml::events::Event::End(event) => closed_root |= event.name().as_ref() == root,
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    assert!(
        opened_root && closed_root && saw_nested,
        "XML must contain its root wrapper and a nested semantic element"
    );
}

fn assert_catalog_xml(content: &str) {
    assert_xml_structure(content, b"catalog", b"control");
}
fn assert_component_xml(content: &str) {
    assert_xml_structure(content, b"component-definition", b"component");
}

fn assert_catalog_yaml(content: &str) {
    let value: Value = serde_yaml::from_str(content).expect("exported catalog YAML must parse");
    assert_catalog_structure(&value);
}

fn assert_component_yaml(content: &str) {
    let value: Value = serde_yaml::from_str(content).expect("exported component YAML must parse");
    assert_component_structure(&value);
}

// ── Catalog: all 9 format pairs ──────────────────────────────────────────

#[test]
fn format_pair_catalog_json_to_json() {
    let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Json);
    let value: Value = serde_json::from_str(&c).expect("exported catalog JSON must parse");
    assert_catalog_structure(&value);
}

#[test]
fn format_pair_catalog_json_to_xml() {
    let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Xml);
    assert_catalog_xml(&c);
}

#[test]
fn format_pair_catalog_json_to_yaml() {
    let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Yaml);
    assert_catalog_yaml(&c);
}

#[test]
fn format_pair_catalog_xml_to_json() {
    let c = export_and_read("tests/fixtures/export/catalog.xml", OutputFormat::Json);
    let value: Value = serde_json::from_str(&c).expect("exported catalog JSON must parse");
    assert_catalog_structure(&value);
}

#[test]
fn format_pair_catalog_xml_to_xml() {
    let c = export_and_read("tests/fixtures/export/catalog.xml", OutputFormat::Xml);
    assert_catalog_xml(&c);
}

#[test]
fn format_pair_catalog_xml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/catalog.xml", OutputFormat::Yaml);
    assert_catalog_yaml(&c);
}

#[test]
fn format_pair_catalog_yaml_to_json() {
    let c = export_and_read("tests/fixtures/export/catalog.yaml", OutputFormat::Json);
    let value: Value = serde_json::from_str(&c).expect("exported catalog JSON must parse");
    assert_catalog_structure(&value);
}

#[test]
fn format_pair_catalog_yaml_to_xml() {
    let c = export_and_read("tests/fixtures/export/catalog.yaml", OutputFormat::Xml);
    assert_catalog_xml(&c);
}

#[test]
fn format_pair_catalog_yaml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/catalog.yaml", OutputFormat::Yaml);
    assert_catalog_yaml(&c);
}

// ── Component: all 9 format pairs ────────────────────────────────────────

#[test]
fn format_pair_component_json_to_json() {
    let c = export_and_read("tests/fixtures/export/component.json", OutputFormat::Json);
    let value: Value = serde_json::from_str(&c).expect("exported component JSON must parse");
    assert_component_structure(&value);
}

#[test]
fn format_pair_component_json_to_xml() {
    let c = export_and_read("tests/fixtures/export/component.json", OutputFormat::Xml);
    assert_component_xml(&c);
}

#[test]
fn format_pair_component_json_to_yaml() {
    let c = export_and_read("tests/fixtures/export/component.json", OutputFormat::Yaml);
    assert_component_yaml(&c);
}

#[test]
fn format_pair_component_xml_to_json() {
    let c = export_and_read("tests/fixtures/export/component.xml", OutputFormat::Json);
    let value: Value = serde_json::from_str(&c).expect("exported component JSON must parse");
    assert_component_structure(&value);
}

#[test]
fn format_pair_component_xml_to_xml() {
    let c = export_and_read("tests/fixtures/export/component.xml", OutputFormat::Xml);
    assert_component_xml(&c);
}

#[test]
fn format_pair_component_xml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/component.xml", OutputFormat::Yaml);
    assert_component_yaml(&c);
}

#[test]
fn format_pair_component_yaml_to_json() {
    let c = export_and_read("tests/fixtures/export/component.yaml", OutputFormat::Json);
    let value: Value = serde_json::from_str(&c).expect("exported component JSON must parse");
    assert_component_structure(&value);
}

#[test]
fn format_pair_component_yaml_to_xml() {
    let c = export_and_read("tests/fixtures/export/component.yaml", OutputFormat::Xml);
    assert_component_xml(&c);
}

#[test]
fn format_pair_component_yaml_to_yaml() {
    let c = export_and_read("tests/fixtures/export/component.yaml", OutputFormat::Yaml);
    assert_component_yaml(&c);
}

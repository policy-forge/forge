//! Integration tests for OSCAL Component Definition XML serialization (T031, T033, T039, T054).

mod common;

use std::path::Path;

use common::MAX_SIZE_BYTES;
use forge::cli::OutputFormat;
use tempfile::TempDir;

/// Helper: run component pipeline with XML format, return XML string.
fn run_component_xml(fixture: &Path, source_profile: Option<&str>) -> String {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("component.xml");

    forge::pipeline::run_component_pipeline(
        fixture,
        Some(&output_path),
        MAX_SIZE_BYTES,
        source_profile,
        &OutputFormat::Xml,
    )
    .unwrap_or_else(|e| panic!("Component XML pipeline failed: {e}"));

    std::fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read XML output: {e}"))
}

/// Helper: run component pipeline with JSON format, return JSON string.
fn run_component_json(fixture: &Path, source_profile: Option<&str>) -> String {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("component.json");

    forge::pipeline::run_component_pipeline(
        fixture,
        Some(&output_path),
        MAX_SIZE_BYTES,
        source_profile,
        &OutputFormat::Json,
    )
    .unwrap_or_else(|e| panic!("Component JSON pipeline failed: {e}"));

    std::fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read JSON output: {e}"))
}

// ─── T031: Component XML Integration Test ─────────────────────────────────

#[test]
fn component_xml_contains_required_elements() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let xml = run_component_xml(fixture, Some("./baselines/nist-800-53.json"));

    // XML declaration
    assert!(xml.contains("<?xml"), "XML output must contain XML declaration");

    // OSCAL namespace
    assert!(
        xml.contains("xmlns=\"http://csrc.nist.gov/ns/oscal/1.0\""),
        "XML output must contain OSCAL namespace"
    );

    // Root element
    assert!(
        xml.contains("<component-definition"),
        "XML output must contain <component-definition> root element"
    );

    // Component (exclude <component-definition> matches)
    let has_component = xml
        .match_indices("<component")
        .any(|(idx, _)| !xml[idx..].starts_with("<component-definition"));
    assert!(has_component, "XML output must contain <component> elements");

    // Metadata
    assert!(xml.contains("<metadata>"), "XML output must contain <metadata>");

    // Well-formed: ends with closing tag
    assert!(
        xml.trim().ends_with("</component-definition>"),
        "XML output must end with </component-definition>"
    );
}

#[test]
fn component_xml_has_uuid_attribute() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let xml = run_component_xml(fixture, Some("./baselines/nist-800-53.json"));

    // component-definition must have uuid attribute
    let cd_start = xml.find("<component-definition").expect("Must have <component-definition>");
    let cd_tag_end = xml[cd_start..].find('>').unwrap() + cd_start;
    let cd_tag = &xml[cd_start..=cd_tag_end];
    assert!(
        cd_tag.contains("uuid=\""),
        "component-definition element must have uuid attribute. Got: {cd_tag}"
    );
}

// ─── T033: Semantic Equivalence (JSON vs XML) ─────────────────────────────

#[test]
fn component_json_and_xml_have_equivalent_metadata() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let json_str = run_component_json(fixture, Some("./baselines/nist-800-53.json"));
    let xml_str = run_component_xml(fixture, Some("./baselines/nist-800-53.json"));

    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let metadata = &json["component-definition"]["metadata"];

    // Title
    let title = metadata["title"].as_str().unwrap();
    assert!(
        xml_str.contains(&format!("<title>{title}</title>")),
        "XML must contain same title as JSON: {title}"
    );

    // Both outputs should have UUIDs (they differ between runs since v4 is random)
    assert!(xml_str.contains("uuid=\""), "XML must contain a uuid attribute");
}

#[test]
fn component_json_and_xml_have_same_component_count() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let json_str = run_component_json(fixture, Some("./baselines/nist-800-53.json"));
    let xml_str = run_component_xml(fixture, Some("./baselines/nist-800-53.json"));

    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Count components in JSON
    let json_component_count = json["component-definition"]["components"].as_array().unwrap().len();

    // Count components in XML (exclude <component-definition> matches)
    let xml_component_count = xml_str
        .match_indices("<component ")
        .filter(|(idx, _)| !xml_str[*idx..].starts_with("<component-definition"))
        .count();
    assert_eq!(
        json_component_count, xml_component_count,
        "JSON ({json_component_count}) and XML ({xml_component_count}) must have same component count"
    );
}

// ─── T039: JSON-to-XML Round-Trip (US4) ───────────────────────────────────

#[test]
fn component_json_fixture_round_trips_to_xml() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    // Generate JSON from pipeline
    let json_str = run_component_json(fixture, Some("./baselines/nist-800-53.json"));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Deserialize to ComponentDefinitionEnvelope
    let envelope: forge::oscal::component_definition::ComponentDefinitionEnvelope =
        serde_json::from_str(&json_str)
            .unwrap_or_else(|e| panic!("Failed to deserialize ComponentDefinitionEnvelope: {e}"));

    // Serialize inner component_definition to XML
    let xml = forge::export::xml_serializer::serialize_component_definition_to_xml(
        &envelope.component_definition,
    )
    .unwrap_or_else(|e| panic!("Failed to serialize component definition to XML: {e}"));

    // Verify XML has matching metadata
    let title = json["component-definition"]["metadata"]["title"].as_str().unwrap();
    assert!(xml.contains(title), "Round-tripped XML must contain title: {title}");

    let uuid = json["component-definition"]["uuid"].as_str().unwrap();
    assert!(
        xml.contains(&format!("uuid=\"{uuid}\"")),
        "Round-tripped XML must contain matching UUID"
    );

    // Verify it's valid XML structure
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<component-definition"));
    assert!(xml.trim().ends_with("</component-definition>"));
}

// ─── T054: EC-8 Malformed JSON error handling ─────────────────────────────

#[test]
fn malformed_json_catalog_deserialize_returns_error() {
    let malformed = r#"{"catalog": {"uuid": "test", "metadata": "not-an-object"}}"#;
    let result: Result<forge::oscal::CatalogEnvelope, _> = serde_json::from_str(malformed);
    assert!(result.is_err(), "Malformed JSON should fail deserialization: {:?}", result);
}

#[test]
fn malformed_json_component_definition_deserialize_returns_error() {
    let malformed = r#"{"component-definition": {"uuid": 123}}"#;
    let result: Result<forge::oscal::component_definition::ComponentDefinitionEnvelope, _> =
        serde_json::from_str(malformed);
    assert!(result.is_err(), "Malformed JSON should fail deserialization: {:?}", result);
}

#[test]
fn invalid_json_returns_parse_error() {
    let invalid = r#"{not valid json at all"#;
    let result: Result<forge::oscal::CatalogEnvelope, _> = serde_json::from_str(invalid);
    assert!(result.is_err(), "Invalid JSON should fail parsing");

    let result2: Result<forge::oscal::component_definition::ComponentDefinitionEnvelope, _> =
        serde_json::from_str(invalid);
    assert!(result2.is_err(), "Invalid JSON should fail parsing");
}

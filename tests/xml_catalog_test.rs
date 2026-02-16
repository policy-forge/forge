//! Integration tests for OSCAL Catalog XML serialization (T030, T032, T038).

mod common;

use std::path::Path;

use common::MAX_SIZE_BYTES;
use forge::cli::OutputFormat;
use tempfile::TempDir;

/// Helper: run catalog pipeline with XML format, return XML string.
fn run_catalog_xml(fixture: &Path) -> String {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("catalog.xml");

    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&output_path),
        MAX_SIZE_BYTES,
        &OutputFormat::Xml,
    )
    .unwrap_or_else(|e| panic!("Catalog XML pipeline failed: {e}"));

    std::fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read XML output: {e}"))
}

/// Helper: run catalog pipeline with JSON format, return JSON string.
fn run_catalog_json(fixture: &Path) -> String {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("catalog.json");

    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&output_path),
        MAX_SIZE_BYTES,
        &OutputFormat::Json,
    )
    .unwrap_or_else(|e| panic!("Catalog JSON pipeline failed: {e}"));

    std::fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read JSON output: {e}"))
}

// ─── T030: Catalog XML Integration Test ───────────────────────────────────

#[test]
fn catalog_xml_contains_required_elements() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let xml = run_catalog_xml(fixture);

    // XML declaration
    assert!(xml.contains("<?xml"), "XML output must contain XML declaration");

    // OSCAL namespace
    assert!(
        xml.contains("xmlns=\"http://csrc.nist.gov/ns/oscal/1.0\""),
        "XML output must contain OSCAL namespace"
    );

    // Root element
    assert!(xml.contains("<catalog"), "XML output must contain <catalog> root element");

    // Metadata
    assert!(xml.contains("<metadata>"), "XML output must contain <metadata>");

    // Groups and controls
    assert!(xml.contains("<group"), "XML output must contain <group> elements");
    assert!(xml.contains("<control"), "XML output must contain <control> elements");

    // Well-formed: ends with closing tag
    assert!(xml.trim().ends_with("</catalog>"), "XML output must end with </catalog>");
}

#[test]
fn catalog_xml_has_uuid_attribute() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let xml = run_catalog_xml(fixture);

    // catalog element must have uuid attribute
    let catalog_start = xml.find("<catalog").expect("Must have <catalog> element");
    let catalog_tag_end = xml[catalog_start..].find('>').unwrap() + catalog_start;
    let catalog_tag = &xml[catalog_start..=catalog_tag_end];
    assert!(
        catalog_tag.contains("uuid=\""),
        "catalog element must have uuid attribute. Got: {catalog_tag}"
    );
}

// ─── T032: Semantic Equivalence (JSON vs XML) ─────────────────────────────

#[test]
fn catalog_json_and_xml_have_equivalent_metadata() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let json_str = run_catalog_json(fixture);
    let xml_str = run_catalog_xml(fixture);

    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let metadata = &json["catalog"]["metadata"];

    // Title
    let title = metadata["title"].as_str().unwrap();
    assert!(
        xml_str.contains(&format!("<title>{title}</title>")),
        "XML must contain same title as JSON: {title}"
    );

    // Version
    let version = metadata["version"].as_str().unwrap();
    assert!(
        xml_str.contains(&format!("<version>{version}</version>")),
        "XML must contain same version as JSON: {version}"
    );

    // OSCAL version
    let oscal_version = metadata["oscal-version"].as_str().unwrap();
    assert!(
        xml_str.contains(&format!("<oscal-version>{oscal_version}</oscal-version>")),
        "XML must contain same oscal-version as JSON: {oscal_version}"
    );

    // Both outputs should have UUIDs (they differ between runs since v4 is random)
    assert!(xml_str.contains("uuid=\""), "XML must contain a uuid attribute");
}

#[test]
fn catalog_json_and_xml_have_same_group_and_control_count() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    let json_str = run_catalog_json(fixture);
    let xml_str = run_catalog_xml(fixture);

    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Count groups in JSON
    let json_groups = json["catalog"]["groups"].as_array().unwrap();
    let json_group_count = json_groups.len();

    // Count groups in XML (by counting <group id= occurrences)
    let xml_group_count = xml_str.matches("<group id=").count();
    assert_eq!(
        json_group_count, xml_group_count,
        "JSON ({json_group_count}) and XML ({xml_group_count}) must have same group count"
    );

    // Count controls in JSON
    let json_control_count: usize =
        json_groups.iter().filter_map(|g| g["controls"].as_array()).map(Vec::len).sum();

    // Count controls in XML
    let xml_control_count = xml_str.matches("<control id=").count();
    assert_eq!(
        json_control_count, xml_control_count,
        "JSON ({json_control_count}) and XML ({xml_control_count}) must have same control count"
    );
}

// ─── T038: JSON-to-XML Round-Trip (US4) ───────────────────────────────────

#[test]
fn catalog_json_fixture_round_trips_to_xml() {
    // T038: Read a valid OSCAL Catalog JSON, deserialize to CatalogEnvelope,
    // serialize inner catalog to XML, verify output is valid XML with matching data.
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    if common::skip_if_missing(fixture) {
        return;
    }

    // First generate JSON from the pipeline
    let json_str = run_catalog_json(fixture);
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Deserialize to CatalogEnvelope
    let envelope: forge::oscal::CatalogEnvelope = serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("Failed to deserialize CatalogEnvelope: {e}"));

    // Serialize inner catalog to XML
    let xml = forge::export::xml_serializer::serialize_catalog_to_xml(&envelope.catalog)
        .unwrap_or_else(|e| panic!("Failed to serialize catalog to XML: {e}"));

    // Verify XML has matching metadata
    let title = json["catalog"]["metadata"]["title"].as_str().unwrap();
    assert!(xml.contains(title), "Round-tripped XML must contain title: {title}");

    let uuid = json["catalog"]["uuid"].as_str().unwrap();
    assert!(
        xml.contains(&format!("uuid=\"{uuid}\"")),
        "Round-tripped XML must contain matching UUID"
    );

    // Verify it's valid XML structure
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<catalog"));
    assert!(xml.trim().ends_with("</catalog>"));
}

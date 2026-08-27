//! Integration tests for OSCAL Catalog XML serialization (T030, T032, T038).

mod common;

use std::path::Path;

use common::DEFAULT_MAX_SIZE_BYTES;
use forge::cli::OutputFormat;

/// Helper: run catalog pipeline with XML format, return XML string.
fn run_catalog_xml(fixture: &Path) -> String {
    forge::pipeline::run_catalog_pipeline(fixture, DEFAULT_MAX_SIZE_BYTES, &OutputFormat::Xml, None)
        .unwrap_or_else(|e| panic!("Catalog XML pipeline failed: {e}"))
        .content
}

/// Helper: run catalog pipeline with JSON format, return JSON string.
fn run_catalog_json(fixture: &Path) -> String {
    forge::pipeline::run_catalog_pipeline(
        fixture,
        DEFAULT_MAX_SIZE_BYTES,
        &OutputFormat::Json,
        None,
    )
    .unwrap_or_else(|e| panic!("Catalog JSON pipeline failed: {e}"))
    .content
}

// ─── T030: Catalog XML Integration Test ───────────────────────────────────

#[test]
fn catalog_xml_contains_required_elements() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    common::require_fixture(fixture);

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
    common::require_fixture(fixture);

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
    common::require_fixture(fixture);

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
    common::require_fixture(fixture);

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
    common::require_fixture(fixture);

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

// ─── T033b: Parameter Extraction in XML Output ────────────────────────────

/// T033b: Verify that `<param>` elements appear within `<control>` blocks
/// in the catalog XML output for requirements that contain extractable parameters.
///
/// The `sample_policy.md` fixture contains "at least 12 characters" (threshold)
/// and "annually" (frequency), so the XML must contain at least one `<param>`.
#[test]
fn catalog_xml_contains_param_elements_for_parameterized_requirements() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    common::require_fixture(fixture);

    let xml = run_catalog_xml(fixture);

    // At least one <param> element must be present
    assert!(
        xml.contains("<param "),
        "Catalog XML must contain <param> elements for parameterized requirements.\nXML snippet:\n{}",
        xml.chars().take(2000).collect::<String>()
    );

    // Each <param> must have an id attribute
    assert!(xml.contains("<param id=\""), "Catalog XML <param> elements must have id attribute");

    // Each <param> must contain a <label> child
    assert!(xml.contains("<label>"), "Catalog XML <param> elements must contain <label>");

    // Params must appear inside <control> blocks (not at catalog level).
    // Check that AT LEAST ONE control block contains a <param> element.
    // Not every control has parameters, so we search all blocks.
    let has_param_in_control = xml
        .split("<control ")
        .skip(1) // skip text before the first <control>
        .any(|segment| {
            // Each segment starts after "<control "; find the closing tag for this control
            let end = segment.find("</control>").unwrap_or(segment.len());
            segment[..end].contains("<param ")
        });
    assert!(
        has_param_in_control,
        "Catalog XML must contain <param> elements nested within at least one <control> element"
    );
}

/// T033b (XSD): Verify that the catalog XML with `<param>` elements
/// still validates against the OSCAL v1.2.0 XSD schema.
///
/// This test relies on xmllint and is skipped if not available.
#[test]
fn catalog_xml_with_params_validates_against_xsd() {
    if !std::process::Command::new("xmllint")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success() || !o.stderr.is_empty())
    {
        eprintln!("Skipping XSD validation: xmllint not available");
        return;
    }

    let fixture = Path::new("tests/fixtures/sample_policy.md");
    common::require_fixture(fixture);

    let xsd_path = Path::new("tests/fixtures/xsd/oscal_catalog_schema.xsd");
    common::require_fixture(xsd_path);

    let xml = run_catalog_xml(fixture);

    // Verify params are present before testing XSD conformance
    assert!(xml.contains("<param "), "Test precondition: XML must contain <param> elements");

    let dir = tempfile::TempDir::new().unwrap();
    let xml_path = dir.path().join("catalog_with_params.xml");
    std::fs::write(&xml_path, &xml).unwrap();

    let output = std::process::Command::new("xmllint")
        .arg("--schema")
        .arg(xsd_path)
        .arg(&xml_path)
        .arg("--noout")
        .output()
        .expect("Failed to run xmllint");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Catalog XML with <param> elements must validate against OSCAL v1.2.0 XSD.\nxmllint stderr:\n{stderr}"
    );
}

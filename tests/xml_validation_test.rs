//! XSD validation integration tests (T035, T036).
//!
//! These tests serialize OSCAL structures to XML, write them to temporary files,
//! and validate against the OSCAL v1.2.0 XSD schemas using `xmllint`.
//!
//! Tests are skipped if `xmllint` is not available on the system.

mod common;

use std::path::Path;
use std::process::Command;

use common::MAX_SIZE_BYTES;
use tempfile::TempDir;

/// Check if xmllint is available on the system.
fn xmllint_available() -> bool {
    Command::new("xmllint")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success() || !o.stderr.is_empty())
}

/// Build catalog XML from the sample policy fixture using the pipeline API.
fn build_catalog_xml(fixture_path: &Path) -> String {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("catalog.xml");

    forge::pipeline::run_catalog_pipeline(
        fixture_path,
        Some(&output_path),
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Xml,
    )
    .unwrap();

    std::fs::read_to_string(&output_path).unwrap()
}

/// Build component definition XML from the sample policy fixture.
///
/// Uses direct serialization (not the pipeline) because the XSD validation test
/// exercises XML structure independently of JSON schema validation. Without a
/// `source_profile`, the component definition has empty `control-implementations`
/// which the JSON schema rejects but the XSD schema permits.
fn build_component_definition_xml(fixture_path: &Path) -> String {
    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
    let content = ingested.reconstruct_content();
    let sections = forge::parse::extract_sections(&content).unwrap();
    let clauses = forge::parse::extract_clauses(&content).unwrap();
    let document = forge::model::assemble_document(&ingested, &sections, &clauses).unwrap();
    let atomized = forge::parse::atomize_document(&document).unwrap();
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc = forge::citation::extract_citations(doc).unwrap();

    let envelope =
        forge::oscal::build_component_definition(&doc, None, None, Some("sample_policy.md"))
            .unwrap();

    forge::export::xml_serializer::serialize_component_definition_to_xml(
        &envelope.component_definition,
    )
    .unwrap()
}

/// T035: Validate catalog XML against OSCAL v1.2.0 catalog XSD.
#[test]
fn catalog_xml_validates_against_xsd() {
    if !xmllint_available() {
        eprintln!("Skipping XSD validation test: xmllint not available");
        return;
    }

    let fixture_path = Path::new("tests/fixtures/sample_policy.md");
    if common::skip_if_missing(fixture_path) {
        return;
    }

    let xsd_path = Path::new("tests/fixtures/xsd/oscal_catalog_schema.xsd");
    if common::skip_if_missing(xsd_path) {
        return;
    }

    let xml = build_catalog_xml(fixture_path);
    let dir = TempDir::new().unwrap();
    let xml_path = dir.path().join("catalog.xml");
    std::fs::write(&xml_path, &xml).unwrap();

    let output = Command::new("xmllint")
        .arg("--schema")
        .arg(xsd_path)
        .arg(&xml_path)
        .arg("--noout")
        .output()
        .expect("Failed to run xmllint");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Catalog XML must validate against OSCAL v1.2.0 XSD.\nxmllint stderr:\n{stderr}"
    );
}

/// T036: Validate component definition XML against OSCAL v1.2.0 component XSD.
#[test]
fn component_definition_xml_validates_against_xsd() {
    if !xmllint_available() {
        eprintln!("Skipping XSD validation test: xmllint not available");
        return;
    }

    let fixture_path = Path::new("tests/fixtures/sample_policy.md");
    if common::skip_if_missing(fixture_path) {
        return;
    }

    let xsd_path = Path::new("tests/fixtures/xsd/oscal_component_schema.xsd");
    if common::skip_if_missing(xsd_path) {
        return;
    }

    let xml = build_component_definition_xml(fixture_path);
    let dir = TempDir::new().unwrap();
    let xml_path = dir.path().join("component-definition.xml");
    std::fs::write(&xml_path, &xml).unwrap();

    let output = Command::new("xmllint")
        .arg("--schema")
        .arg(xsd_path)
        .arg(&xml_path)
        .arg("--noout")
        .output()
        .expect("Failed to run xmllint");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Component definition XML must validate against OSCAL v1.2.0 XSD.\nxmllint stderr:\n{stderr}"
    );
}

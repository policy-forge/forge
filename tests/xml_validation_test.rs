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

/// Build catalog XML from the sample policy fixture.
fn build_catalog_xml(fixture_path: &Path) -> String {
    let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
    let content = ingested.reconstruct_content();
    let sections = forge::parse::extract_sections(&content).unwrap();
    let clauses = forge::parse::extract_clauses(&content).unwrap();
    let document = forge::model::assemble_document(&ingested, &sections, &clauses).unwrap();
    let atomized = forge::parse::atomize_document(&document).unwrap();
    let doc = forge::uuid::assign_stable_ids(atomized);
    let doc = forge::citation::extract_citations(doc).unwrap();

    let mut trace_links = forge::TraceLinkCollection::new();
    let mut catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();
    forge::oscal::trace_embedding::embed_trace_in_catalog(&mut catalog, &trace_links);

    let metadata = forge::oscal::assemble_metadata(&doc.metadata, None).unwrap();
    let citations = doc.collect_citations();
    let (back_matter_resources, _) = forge::oscal::generate_back_matter(&citations).unwrap();
    let back_matter = if back_matter_resources.is_empty() {
        None
    } else {
        Some(forge::BackMatter { resources: back_matter_resources })
    };

    let oscal_catalog = forge::oscal::OscalCatalog {
        uuid: metadata.uuid.to_string(),
        metadata: forge::oscal::catalog::OscalMetadata {
            title: metadata.title,
            last_modified: metadata.last_modified.to_rfc3339(),
            version: metadata.version,
            oscal_version: metadata.oscal_version,
        },
        groups: catalog.groups,
        back_matter,
    };

    forge::export::xml_serializer::serialize_catalog_to_xml(&oscal_catalog).unwrap()
}

/// Build component definition XML from the sample policy fixture.
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

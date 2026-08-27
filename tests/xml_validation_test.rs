//! XSD validation integration tests (T035, T036).
//!
//! These tests serialize OSCAL structures to XML, write them to temporary files,
//! and validate against the OSCAL v1.2.3 XSD schemas using `xmllint`.
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
    forge::pipeline::run_catalog_pipeline(
        fixture_path,
        MAX_SIZE_BYTES,
        &forge::cli::OutputFormat::Xml,
        None,
    )
    .unwrap()
    .content
}

/// Build component definition XML from the sample policy fixture using the pipeline API.
fn build_component_definition_xml(fixture_path: &Path) -> String {
    forge::pipeline::run_component_pipeline(
        fixture_path,
        MAX_SIZE_BYTES,
        Some("./baselines/nist-800-53.json"),
        &forge::cli::OutputFormat::Xml,
        None,
    )
    .unwrap()
    .content
}

/// T035: Validate catalog XML against OSCAL v1.2.3 catalog XSD.
#[test]
fn catalog_xml_validates_against_xsd() {
    if !xmllint_available() {
        eprintln!("Skipping XSD validation test: xmllint not available");
        return;
    }

    let fixture_path = Path::new("tests/fixtures/sample_policy.md");
    common::require_fixture(fixture_path);

    let xsd_path = Path::new("tests/fixtures/xsd/oscal_catalog_schema.xsd");
    common::require_fixture(xsd_path);

    let xml = build_catalog_xml(fixture_path);
    let dir = TempDir::new().unwrap();
    let xml_path = dir.path().join("catalog.xml");
    std::fs::write(&xml_path, &xml).unwrap();

    let output = Command::new("xmllint")
        .arg("--nonet")
        .arg("--schema")
        .arg(xsd_path)
        .arg(&xml_path)
        .arg("--noout")
        .output()
        .expect("Failed to run xmllint");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Catalog XML must validate against OSCAL v1.2.3 XSD.\nxmllint stderr:\n{stderr}"
    );
}

/// T036: Validate component definition XML against OSCAL v1.2.3 component XSD.
#[test]
fn component_definition_xml_validates_against_xsd() {
    if !xmllint_available() {
        eprintln!("Skipping XSD validation test: xmllint not available");
        return;
    }

    let fixture_path = Path::new("tests/fixtures/sample_policy.md");
    common::require_fixture(fixture_path);

    let xsd_path = Path::new("tests/fixtures/xsd/oscal_component_schema.xsd");
    common::require_fixture(xsd_path);

    let xml = build_component_definition_xml(fixture_path);
    let dir = TempDir::new().unwrap();
    let xml_path = dir.path().join("component-definition.xml");
    std::fs::write(&xml_path, &xml).unwrap();

    let output = Command::new("xmllint")
        .arg("--nonet")
        .arg("--schema")
        .arg(xsd_path)
        .arg(&xml_path)
        .arg("--noout")
        .output()
        .expect("Failed to run xmllint");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Component definition XML must validate against OSCAL v1.2.3 XSD.\nxmllint stderr:\n{stderr}"
    );
}

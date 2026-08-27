//! Comprehensive format-pair tests for the export pipeline.
//!
//! Tests all 9 format pairs (JSON/XML/YAML x JSON/XML/YAML) for both
//! Catalog and Component Definition model types (18 cases total).

use std::path::Path;

use forge::cli::OutputFormat;
use forge::cli::export::export_artifact;
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
enum Model {
    Catalog,
    ComponentDefinition,
}

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
    export_artifact(&input, format, Some(&output)).unwrap_or_else(|e| {
        panic!("export_artifact failed for {relative_path:?} -> {format:?}: {e}")
    });
    std::fs::read_to_string(&output)
        .unwrap_or_else(|e| panic!("reading exported file {}: {e}", output.display()))
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

fn assert_exported_structure(model: Model, format: OutputFormat, content: &str) {
    match (model, format) {
        (Model::Catalog, OutputFormat::Json) => {
            let value: Value =
                serde_json::from_str(content).expect("exported catalog JSON must parse");
            assert_catalog_structure(&value);
        }
        (Model::Catalog, OutputFormat::Xml) => {
            assert_xml_structure(content, b"catalog", b"control");
        }
        (Model::Catalog, OutputFormat::Yaml) => {
            let value: Value =
                serde_yaml::from_str(content).expect("exported catalog YAML must parse");
            assert_catalog_structure(&value);
        }
        (Model::ComponentDefinition, OutputFormat::Json) => {
            let value: Value =
                serde_json::from_str(content).expect("exported component JSON must parse");
            assert_component_structure(&value);
        }
        (Model::ComponentDefinition, OutputFormat::Xml) => {
            assert_xml_structure(content, b"component-definition", b"component");
        }
        (Model::ComponentDefinition, OutputFormat::Yaml) => {
            let value: Value =
                serde_yaml::from_str(content).expect("exported component YAML must parse");
            assert_component_structure(&value);
        }
    }
}

#[test]
fn all_format_pairs_preserve_model_structure() {
    const CATALOG_INPUTS: [&str; 3] = [
        "tests/fixtures/export/catalog.json",
        "tests/fixtures/export/catalog.xml",
        "tests/fixtures/export/catalog.yaml",
    ];
    const COMPONENT_INPUTS: [&str; 3] = [
        "tests/fixtures/export/component.json",
        "tests/fixtures/export/component.xml",
        "tests/fixtures/export/component.yaml",
    ];

    for (model, inputs) in
        [(Model::Catalog, CATALOG_INPUTS), (Model::ComponentDefinition, COMPONENT_INPUTS)]
    {
        for input in inputs {
            for output in [OutputFormat::Json, OutputFormat::Xml, OutputFormat::Yaml] {
                let content = export_and_read(input, output);
                assert_exported_structure(model, output, &content);
            }
        }
    }
}

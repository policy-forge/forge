//! OSCAL v1.2.3 compatibility gates for every currently generated model and format.

mod common;

use std::path::Path;
use std::process::Command;

use clap::CommandFactory as _;
use forge::cli::OutputFormat;
use forge::oscal::profile::{ProfileRoot, SelectionMode, build_profile};
use forge::validate::{OscalModelType, validate_artifact};
use tempfile::TempDir;

const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

fn assert_json_schema_valid(value: &serde_json::Value, model: OscalModelType) {
    let result = validate_artifact(value, model).expect("schema validation must run");
    assert!(
        result.is_valid,
        "generated {model} must validate against OSCAL v1.2.3: {:#?}",
        result.errors
    );
    assert_eq!(result.schema_version_used, "1.2.3");
    assert_eq!(result.declared_oscal_version.as_deref(), Some("1.2.3"));
}

fn xmllint_available() -> bool {
    Command::new("xmllint").arg("--version").output().is_ok()
}

fn assert_xsd_valid(xml: &str, xsd: &Path, artifact_name: &str) {
    if !xmllint_available() {
        eprintln!("Skipping {artifact_name} XSD gate: xmllint is unavailable");
        return;
    }

    let dir = TempDir::new().expect("temporary XSD validation directory");
    let xml_path = dir.path().join(format!("{artifact_name}.xml"));
    std::fs::write(&xml_path, xml).expect("write generated XML");
    let output = Command::new("xmllint")
        .arg("--nonet")
        .arg("--schema")
        .arg(xsd)
        .arg("--noout")
        .arg(&xml_path)
        .output()
        .expect("run xmllint");
    assert!(
        output.status.success(),
        "generated {artifact_name} XML must validate against OSCAL v1.2.3 XSD:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_catalog_json_xml_yaml_pass_v1_2_3_gates() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");

    let json =
        forge::pipeline::run_catalog_pipeline(fixture, MAX_SIZE_BYTES, &OutputFormat::Json, None)
            .expect("generate catalog JSON")
            .content;
    let json_value: serde_json::Value = serde_json::from_str(&json).expect("parse catalog JSON");
    assert_json_schema_valid(&json_value, OscalModelType::Catalog);

    let yaml =
        forge::pipeline::run_catalog_pipeline(fixture, MAX_SIZE_BYTES, &OutputFormat::Yaml, None)
            .expect("generate catalog YAML")
            .content;
    let yaml_value: serde_json::Value = serde_yaml::from_str(&yaml).expect("parse catalog YAML");
    assert_json_schema_valid(&yaml_value, OscalModelType::Catalog);

    let xml =
        forge::pipeline::run_catalog_pipeline(fixture, MAX_SIZE_BYTES, &OutputFormat::Xml, None)
            .expect("generate catalog XML")
            .content;
    assert_xsd_valid(&xml, Path::new("tests/fixtures/xsd/oscal_catalog_schema.xsd"), "catalog");
}

#[test]
fn generated_component_json_xml_yaml_pass_v1_2_3_gates() {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    let source_profile = Some("./baselines/nist-800-53.json");

    let json = forge::pipeline::run_component_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        source_profile,
        &OutputFormat::Json,
        None,
    )
    .expect("generate component JSON")
    .content;
    let json_value: serde_json::Value = serde_json::from_str(&json).expect("parse component JSON");
    assert_json_schema_valid(&json_value, OscalModelType::ComponentDefinition);

    let yaml = forge::pipeline::run_component_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        source_profile,
        &OutputFormat::Yaml,
        None,
    )
    .expect("generate component YAML")
    .content;
    let yaml_value: serde_json::Value = serde_yaml::from_str(&yaml).expect("parse component YAML");
    assert_json_schema_valid(&yaml_value, OscalModelType::ComponentDefinition);

    let xml = forge::pipeline::run_component_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        source_profile,
        &OutputFormat::Xml,
        None,
    )
    .expect("generate component XML")
    .content;
    assert_xsd_valid(
        &xml,
        Path::new("tests/fixtures/xsd/oscal_component_schema.xsd"),
        "component-definition",
    );
}

#[test]
fn generated_profile_variants_pass_v1_2_3_json_xml_yaml_gates() {
    let cases = [
        (SelectionMode::Include, vec!["AC-1".to_string()], vec![]),
        (SelectionMode::Exclude, vec!["AC-2".to_string()], vec![]),
        (
            SelectionMode::Include,
            vec!["AC-3".to_string()],
            vec![("ac-3_prm_1".to_string(), "approved".to_string())],
        ),
    ];

    for (index, (mode, control_ids, params)) in cases.into_iter().enumerate() {
        let profile = build_profile("catalog.json", control_ids, mode, &params, None)
            .expect("generate profile variant");
        let root = ProfileRoot { profile };

        let json_value = serde_json::to_value(&root).expect("serialize profile JSON");
        assert_json_schema_valid(&json_value, OscalModelType::Profile);

        let yaml = forge::export::yaml::serialize_to_yaml(&root).expect("serialize profile YAML");
        let yaml_value: serde_json::Value =
            serde_yaml::from_str(&yaml).expect("parse profile YAML");
        assert_json_schema_valid(&yaml_value, OscalModelType::Profile);

        let xml = forge::export::xml_serializer::serialize_profile_to_xml(&root.profile)
            .expect("serialize profile XML");
        assert_xsd_valid(
            &xml,
            Path::new("tests/fixtures/xsd/oscal_profile_schema.xsd"),
            &format!("profile-{index}"),
        );
    }
}

#[test]
fn legacy_v1_2_0_profile_remains_valid_against_the_current_schema() {
    let profile: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/legacy/v1.2.0/profile/profile.json"))
            .expect("legacy profile fixture must parse");
    let result = validate_artifact(&profile, OscalModelType::Profile)
        .expect("legacy profile validation must run");

    assert!(result.is_valid, "legacy profile must remain compatible: {:#?}", result.errors);
    assert_eq!(result.declared_oscal_version.as_deref(), Some("1.2.0"));
    assert_eq!(result.schema_version_used, "1.2.3");
}

#[test]
fn compatibility_upgrade_does_not_expand_runtime_or_cli_scope() {
    let command = forge::cli::Cli::command();
    let subcommand_names: Vec<_> = command.get_subcommands().map(clap::Command::get_name).collect();
    for excluded in ["mapping", "assessment-plan", "ssp"] {
        assert!(
            !subcommand_names.contains(&excluded),
            "compatibility work must not add a {excluded} command"
        );
    }

    for subcommand in command.get_subcommands() {
        assert!(
            subcommand.get_arguments().all(|argument| argument.get_id() != "schema_version"),
            "compatibility work must not add --schema-version to {}",
            subcommand.get_name()
        );
    }

    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/oscal-schema-manifest.json"))
            .expect("schema manifest must parse");
    let runtime_models: std::collections::BTreeSet<_> = manifest["assets"]
        .as_array()
        .expect("manifest assets")
        .iter()
        .filter(|asset| asset["role"] == "runtime")
        .filter_map(|asset| asset["model"].as_str())
        .collect();
    assert_eq!(
        runtime_models,
        std::collections::BTreeSet::from(["catalog", "component-definition", "profile"]),
        "runtime schema selection must remain limited to the existing model families"
    );

    let cargo_toml = include_str!("../Cargo.toml");
    for http_client in ["reqwest", "ureq"] {
        assert!(
            !cargo_toml.lines().any(|line| line.trim_start().starts_with(http_client)),
            "runtime schema loading must not add the {http_client} HTTP client"
        );
    }
}

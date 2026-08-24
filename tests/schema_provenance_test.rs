use std::collections::HashSet;
use std::path::{Component, Path};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct Manifest {
    repository: String,
    tag: String,
    release_commit: String,
    published_at: String,
    schema_version: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    url: String,
    local_path: String,
    size: u64,
    sha256: String,
    format: String,
    model: String,
    role: String,
}

fn manifest() -> Manifest {
    serde_json::from_str(include_str!("../schemas/oscal-schema-manifest.json"))
        .expect("schema provenance manifest must be valid JSON")
}

fn has_remote_json_ref(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            (key == "$ref" && value.as_str().is_some_and(|reference| !reference.starts_with('#')))
                || has_remote_json_ref(value)
        }),
        Value::Array(values) => values.iter().any(has_remote_json_ref),
        _ => false,
    }
}

#[test]
fn manifest_pins_the_approved_release_and_complete_allowlist() {
    let manifest = manifest();
    assert_eq!(manifest.repository, "usnistgov/OSCAL");
    assert_eq!(manifest.tag, "v1.2.3");
    assert_eq!(manifest.release_commit, "e061961");
    assert_eq!(manifest.published_at, "2026-08-07");
    assert_eq!(manifest.schema_version, "1.2.3");
    assert_eq!(manifest.assets.len(), 9);

    let names: HashSet<_> = manifest.assets.iter().map(|asset| asset.name.as_str()).collect();
    assert_eq!(names.len(), manifest.assets.len(), "asset names must be unique");
    assert_eq!(
        names,
        HashSet::from([
            "oscal_catalog_schema.json",
            "oscal_component_schema.json",
            "oscal_profile_schema.json",
            "oscal_assessment-plan_schema.json",
            "oscal_ssp_schema.json",
            "oscal_catalog_schema.xsd",
            "oscal_component_schema.xsd",
            "oscal_profile_schema.xsd",
            "oscal_complete_schema.xsd",
        ])
    );
}

#[test]
fn vendored_assets_match_release_sizes_and_sha256_digests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for asset in manifest().assets {
        let relative = Path::new(&asset.local_path);
        assert!(
            !relative.is_absolute()
                && relative.components().all(|component| matches!(component, Component::Normal(_))),
            "{} has an unsafe local path: {}",
            asset.name,
            asset.local_path
        );
        assert!(
            asset.local_path.starts_with("schemas/")
                || asset.local_path.starts_with("tests/fixtures/"),
            "{} is outside the schema allowlist",
            asset.name
        );
        assert_eq!(
            asset.url,
            format!("https://github.com/usnistgov/OSCAL/releases/download/v1.2.3/{}", asset.name)
        );
        assert!(matches!(asset.format.as_str(), "json-schema" | "xsd"));
        assert!(matches!(asset.role.as_str(), "runtime" | "test"));
        assert!(!asset.model.trim().is_empty());

        let bytes = std::fs::read(root.join(relative)).expect("manifest asset must exist");
        assert_eq!(bytes.len() as u64, asset.size, "{} size mismatch", asset.name);
        let actual = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(actual, asset.sha256, "{} SHA-256 mismatch", asset.name);
    }
}

#[test]
fn vendored_schemas_are_offline_and_compile_where_applicable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let remote_schema_location = regex::Regex::new(r#"schemaLocation\s*=\s*[\"']\s*https?://"#)
        .expect("remote schema-location regex must compile");

    for asset in manifest().assets {
        let bytes = std::fs::read(root.join(&asset.local_path)).expect("manifest asset must exist");
        if asset.format == "json-schema" {
            let schema: Value = serde_json::from_slice(&bytes).expect("schema must be JSON");
            assert!(!has_remote_json_ref(&schema), "{} contains a non-local $ref", asset.name);
            jsonschema::validator_for(&schema)
                .unwrap_or_else(|error| panic!("{} must compile offline: {error}", asset.name));
        } else {
            let xsd = std::str::from_utf8(&bytes).expect("XSD must be UTF-8");
            assert!(xsd.contains("<m:schema-version>1.2.3</m:schema-version>"));
            assert!(
                !remote_schema_location.is_match(xsd),
                "{} contains a remote schema location",
                asset.name
            );
        }
    }
}

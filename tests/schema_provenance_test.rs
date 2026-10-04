use std::collections::HashSet;
use std::path::{Component, Path};

use serde::Deserialize;
use serde_json::Value;

mod common;

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

/// Verify exact release identity and the complete unique schema-asset allowlist.
#[test]
fn manifest_pins_the_approved_release_and_complete_allowlist() {
    let manifest = manifest();
    assert_eq!(manifest.repository, "usnistgov/OSCAL");
    assert_eq!(manifest.tag, "v1.2.3");
    assert_eq!(manifest.release_commit, "e061961");
    assert_eq!(manifest.published_at, "2026-08-07");
    assert_eq!(manifest.schema_version, "1.2.3");
    assert_eq!(manifest.assets.len(), 12);

    let names: HashSet<_> = manifest.assets.iter().map(|asset| asset.name.as_str()).collect();
    assert_eq!(names.len(), manifest.assets.len(), "asset names must be unique");
    assert_eq!(
        names,
        HashSet::from([
            "oscal_catalog_schema.json",
            "oscal_component_schema.json",
            "oscal_profile_schema.json",
            "oscal_mapping_schema.json",
            "oscal_assessment-results_schema.json",
            "oscal_assessment-plan_schema.json",
            "oscal_ssp_schema.json",
            "oscal_poam_schema.json",
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
        if asset.role == "runtime" {
            assert!(
                asset.local_path.starts_with("schemas/"),
                "runtime asset {} must be loaded from schemas/",
                asset.name
            );
        }
        assert_ne!(asset.model.trim(), "");

        let bytes = std::fs::read(root.join(relative)).expect("manifest asset must exist");
        assert_eq!(bytes.len() as u64, asset.size, "{} size mismatch", asset.name);
        let actual = common::sha256_hex(&bytes);
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

/// Decode the exact embedded POA&M release schema for offline provenance controls.
fn poam_schema() -> Value {
    serde_json::from_str(include_str!("../schemas/oscal_poam_schema.json"))
        .expect("pinned POA&M schema must be valid JSON")
}

/// Build a nonempty schema fixture without asserting remediation or source acceptance.
fn minimal_native_poam() -> Value {
    serde_json::json!({
        "plan-of-action-and-milestones": {
            "uuid": "3360ade4-47bf-4b4b-ad32-2576dcaa0001",
            "metadata": {
                "title": "Schema boundary fixture",
                "last-modified": "2026-10-02T00:00:00Z",
                "version": "1",
                "oscal-version": "1.2.3"
            },
            "import-ssp": { "href": "ssp.json" },
            "poam-items": [{
                "uuid": "3360ade4-47bf-4b4b-ad32-2576dcaa0002",
                "title": "Explicit fixture item",
                "description": "Tests native structure without asserting source or workflow acceptance."
            }]
        }
    })
}

/// Verify that poam release asset and schema identity are pinned.
#[test]
fn poam_release_asset_and_schema_identity_are_pinned() {
    let asset = manifest()
        .assets
        .into_iter()
        .find(|asset| asset.name == "oscal_poam_schema.json")
        .expect("POA&M release asset must be listed");
    assert_eq!(asset.local_path, "schemas/oscal_poam_schema.json");
    assert_eq!(asset.size, 148_253);
    assert_eq!(asset.sha256, "f4fd94487408a9589954b5b92d88965c984d4f79b85365cc574b860898759437");
    assert_eq!(asset.format, "json-schema");
    assert_eq!(asset.model, "poam");
    assert_eq!(asset.role, "runtime");

    let schema = poam_schema();
    assert_eq!(schema["$schema"], "http://json-schema.org/draft-07/schema#");
    assert_eq!(schema["$id"], "http://csrc.nist.gov/ns/oscal/1.2.3/oscal-poam-schema.json");
}

/// Verify that native poam schema accepts an explicit named item offline.
#[test]
fn native_poam_schema_accepts_an_explicit_named_item_offline() {
    let validator = jsonschema::validator_for(&poam_schema())
        .expect("pinned POA&M schema must compile offline");
    let candidate = minimal_native_poam();
    assert!(validator.is_valid(&candidate), "minimal native fixture must match the exact schema");
}

/// Verify that native poam schema rejects an empty scaffold.
#[test]
fn native_poam_schema_rejects_an_empty_scaffold() {
    let validator = jsonschema::validator_for(&poam_schema())
        .expect("pinned POA&M schema must compile offline");
    let mut candidate = minimal_native_poam();
    assert!(validator.is_valid(&candidate));
    candidate["plan-of-action-and-milestones"]["poam-items"] = serde_json::json!([]);
    assert!(!validator.is_valid(&candidate), "a zero-selection scaffold is not a native POA&M");
}

/// Verify that native poam schema leaves system context choice to semantic validation.
#[test]
fn native_poam_schema_leaves_system_context_choice_to_semantic_validation() {
    let validator = jsonschema::validator_for(&poam_schema())
        .expect("pinned POA&M schema must compile offline");
    let mut candidate = minimal_native_poam();
    candidate["plan-of-action-and-milestones"]
        .as_object_mut()
        .expect("fixture root must be an object")
        .remove("import-ssp");
    // The official model requires import-ssp or system-id. Its generated JSON
    // Schema does not encode that choice; Forge must enforce it separately.
    assert!(validator.is_valid(&candidate), "the exact schema leaves this semantic gap open");
}

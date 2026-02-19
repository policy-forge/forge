//! Semantic equivalence tests between JSON and YAML output (WI-27, US3, T014-T015).
//!
//! Verifies that serializing the same OSCAL model to both JSON and YAML
//! then deserializing both to `serde_json::Value` produces structurally
//! identical results (PRD M-3, R-4).
//!
//! Strategy: Run the pipeline ONCE to get the JSON output, parse it to a
//! `serde_json::Value`, then serialize that Value to YAML and parse back.
//! This ensures we compare the SAME model in both formats (avoiding timestamp
//! and UUID drift from separate pipeline runs).

use std::path::Path;

use tempfile::TempDir;

use forge::export::{deserialize_from_yaml, serialize_to_yaml};

// ---------------------------------------------------------------------------
// T014: Catalog Semantic Equivalence
// ---------------------------------------------------------------------------

/// Helper: run catalog pipeline once, return (JSON-parsed Value, YAML-round-tripped Value).
fn catalog_json_and_yaml_values() -> (serde_json::Value, serde_json::Value) {
    let fixture = Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Fixture must exist: {}", fixture.display());

    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("catalog.json");

    // Run pipeline once → JSON
    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&json_path),
        10 * 1024 * 1024,
        &forge::cli::OutputFormat::Json,
    )
    .expect("Catalog pipeline should succeed");

    let json_str = std::fs::read_to_string(&json_path).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Serialize same Value to YAML, then parse back
    let yaml_str = serialize_to_yaml(&json_value).unwrap();
    let yaml_value: serde_json::Value = deserialize_from_yaml(&yaml_str).unwrap();

    (json_value, yaml_value)
}

#[test]
fn catalog_json_and_yaml_are_semantically_equivalent() {
    let (json_value, yaml_value) = catalog_json_and_yaml_values();
    assert_eq!(
        json_value, yaml_value,
        "Catalog JSON and YAML should deserialize to identical serde_json::Value"
    );
}

#[test]
fn catalog_empty_collections_serialize_consistently() {
    // EC-1: empty collections should be identical across formats
    let (json_value, yaml_value) = catalog_json_and_yaml_values();

    let json_groups = &json_value["catalog"]["groups"];
    let yaml_groups = &yaml_value["catalog"]["groups"];
    assert_eq!(json_groups, yaml_groups, "Groups should be identical across formats");
}

#[test]
fn catalog_unicode_text_preserved_across_formats() {
    // EC-2: Unicode text should survive serialization in both formats
    let model = serde_json::json!({
        "catalog": {
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "metadata": {
                "title": "Politique de S\u{00e9}curit\u{00e9} \u{2014} \u{00c9}dition Fran\u{00e7}aise",
                "version": "1.0",
                "oscal-version": "1.2.0",
                "last-modified": "2025-01-01T00:00:00Z"
            },
            "groups": []
        }
    });

    let json_str = serde_json::to_string_pretty(&model).unwrap();
    let yaml_str = serialize_to_yaml(&model).unwrap();

    let json_parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let yaml_parsed: serde_json::Value = deserialize_from_yaml(&yaml_str).unwrap();

    assert_eq!(json_parsed, yaml_parsed, "Unicode text should be identical across formats");
    assert!(
        yaml_str.contains("Politique de S\u{00e9}curit\u{00e9}"),
        "YAML should preserve Unicode: {yaml_str}"
    );
}

#[test]
fn catalog_null_optional_fields_handled_identically() {
    // EC-7: null/None optional fields should be handled identically
    let model = serde_json::json!({
        "catalog": {
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "metadata": {
                "title": "Test",
                "version": "1.0",
                "oscal-version": "1.2.0",
                "last-modified": "2025-01-01T00:00:00Z"
            },
            "groups": [],
            "back-matter": null
        }
    });

    let json_str = serde_json::to_string_pretty(&model).unwrap();
    let yaml_str = serialize_to_yaml(&model).unwrap();

    let json_parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let yaml_parsed: serde_json::Value = deserialize_from_yaml(&yaml_str).unwrap();

    assert_eq!(json_parsed, yaml_parsed, "Null fields should be handled identically");
}

// ---------------------------------------------------------------------------
// T015: Component Definition Semantic Equivalence
// ---------------------------------------------------------------------------

/// Helper: run component pipeline once, return (JSON-parsed Value, YAML-round-tripped Value).
fn component_json_and_yaml_values() -> (serde_json::Value, serde_json::Value) {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists(), "Fixture must exist: {}", fixture.display());

    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("component.json");

    // Run pipeline once → JSON
    forge::pipeline::run_component_pipeline(
        fixture,
        Some(&json_path),
        10 * 1024 * 1024,
        Some("./baselines/nist-800-53.json"),
        &forge::cli::OutputFormat::Json,
    )
    .expect("Component pipeline should succeed");

    let json_str = std::fs::read_to_string(&json_path).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Serialize same Value to YAML, then parse back
    let yaml_str = serialize_to_yaml(&json_value).unwrap();
    let yaml_value: serde_json::Value = deserialize_from_yaml(&yaml_str).unwrap();

    (json_value, yaml_value)
}

#[test]
fn component_json_and_yaml_are_semantically_equivalent() {
    let (json_value, yaml_value) = component_json_and_yaml_values();
    assert_eq!(
        json_value, yaml_value,
        "Component Definition JSON and YAML should deserialize to identical serde_json::Value"
    );
}

#[test]
fn component_deeply_nested_structures_serialize_consistently() {
    // EC-3: deeply nested structures (control-implementations -> implemented-requirements -> props)
    let (json_value, yaml_value) = component_json_and_yaml_values();

    let json_impl_reqs = &json_value["component-definition"]["components"][0]["control-implementations"]
        [0]["implemented-requirements"];
    let yaml_impl_reqs = &yaml_value["component-definition"]["components"][0]["control-implementations"]
        [0]["implemented-requirements"];

    assert_eq!(
        json_impl_reqs, yaml_impl_reqs,
        "Deeply nested implemented-requirements should be identical across formats"
    );
}

#[test]
fn component_control_implementations_vec_fidelity() {
    // R-2: Vec<serde_json::Value> control_implementations cross-format fidelity
    let (json_value, yaml_value) = component_json_and_yaml_values();

    let json_ci = &json_value["component-definition"]["components"][0]["control-implementations"];
    let yaml_ci = &yaml_value["component-definition"]["components"][0]["control-implementations"];

    assert_eq!(
        json_ci, yaml_ci,
        "control-implementations Vec should be identical across formats (R-2)"
    );
}

// ---------------------------------------------------------------------------
// T033c: Parameter Extraction in YAML Output
// ---------------------------------------------------------------------------

/// T033c: Verify that `params` arrays appear within catalog controls in YAML output
/// for requirements that contain extractable parameters.
///
/// The sample_policy.md fixture contains "at least 12 characters" (threshold)
/// and "annually" (frequency), so each parameterized control's YAML must include
/// a non-empty `params` key serialized by serde (via `skip_serializing_if`).
#[test]
fn catalog_yaml_contains_params_arrays_for_parameterized_controls() {
    use forge::cli::OutputFormat;

    let fixture = Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Fixture must exist: {}", fixture.display());

    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("catalog.yaml");

    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&yaml_path),
        10 * 1024 * 1024,
        &OutputFormat::Yaml,
    )
    .expect("Catalog YAML pipeline should succeed");

    let yaml_str = std::fs::read_to_string(&yaml_path).unwrap();

    // At least one `params:` key must be present
    assert!(
        yaml_str.contains("params:"),
        "Catalog YAML must contain 'params:' arrays for parameterized controls.\nYAML:\n{}",
        &yaml_str[..yaml_str.len().min(3000)]
    );

    // Each param must have an `id:` field
    assert!(yaml_str.contains("id:"), "YAML params must include 'id:' fields");

    // Each param must have a `label:` field
    assert!(yaml_str.contains("label:"), "YAML params must include 'label:' fields");

    // `values:` must be present (threshold params include extracted numeric values)
    assert!(yaml_str.contains("values:"), "YAML params must include 'values:' arrays");

    // `constraints:` must be present
    assert!(yaml_str.contains("constraints:"), "YAML params must include 'constraints:' arrays");
}

/// T033c (round-trip): Verify that `params` in YAML catalog output round-trip
/// through serde_json::Value identity — YAML params must equal JSON params.
#[test]
fn catalog_yaml_params_match_json_params() {
    let (json_value, yaml_value) = catalog_json_and_yaml_values();

    let json_groups = json_value["catalog"]["groups"].as_array().expect("Must have groups");
    let yaml_groups = yaml_value["catalog"]["groups"].as_array().expect("Must have groups");

    // Find any control with params in JSON and verify YAML has the same
    for (gi, json_group) in json_groups.iter().enumerate() {
        let Some(json_controls) = json_group["controls"].as_array() else {
            continue;
        };
        for (ci, json_control) in json_controls.iter().enumerate() {
            let json_params = &json_control["params"];
            let yaml_params = &yaml_groups[gi]["controls"][ci]["params"];
            assert_eq!(
                json_params, yaml_params,
                "params in control [{gi}][{ci}] must match between JSON and YAML"
            );
        }
    }
}

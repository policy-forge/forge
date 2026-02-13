use std::path::Path;

use tempfile::TempDir;

/// Helper: run component pipeline on full_policy.md fixture, return parsed JSON.
fn run_component_pipeline_on_fixture(source_profile: &str) -> serde_json::Value {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists(), "Fixture file must exist: {}", fixture.display());

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("component.json");

    let result = forge::pipeline::run_component_pipeline(
        fixture,
        Some(&output_path),
        10 * 1024 * 1024,
        source_profile,
    );
    assert!(result.is_ok(), "Pipeline failed on {}: {:?}", fixture.display(), result.unwrap_err());

    let json_str = std::fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read output {}: {e}", output_path.display()));
    serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("Output is not valid JSON: {e}\nContent: {json_str}"))
}

// ─── T024: Component Pipeline End-to-End Integration Test ────────────────

#[test]
fn component_pipeline_produces_valid_component_definition_json() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");

    // Top-level key must be "component-definition"
    let top_keys: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert_eq!(
        top_keys,
        vec!["component-definition"],
        "Top-level should only contain 'component-definition' key"
    );

    let cd = &json["component-definition"];
    assert!(cd.is_object(), "component-definition must be an object");

    // Required fields: uuid, metadata, components
    assert!(cd["uuid"].is_string(), "Must have uuid");
    assert!(cd["metadata"].is_object(), "Must have metadata");
    assert!(cd["components"].is_array(), "Must have components");
}

#[test]
fn component_pipeline_metadata_populated() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let metadata = &json["component-definition"]["metadata"];

    assert_eq!(metadata["title"].as_str().unwrap(), "Sample Security Policy");
    assert_eq!(metadata["version"].as_str().unwrap(), "1.0.0");
    assert_eq!(metadata["oscal-version"].as_str().unwrap(), "1.2.0");
    assert!(metadata["last-modified"].is_string(), "Must have last-modified");
}

#[test]
fn component_pipeline_has_single_policy_component() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let components = json["component-definition"]["components"].as_array().unwrap();

    assert_eq!(components.len(), 1, "Must have exactly one component");
    assert_eq!(components[0]["type"], "policy");
    assert_eq!(components[0]["title"], "Sample Security Policy");
}

#[test]
fn component_pipeline_has_populated_control_implementations() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let comp = &json["component-definition"]["components"][0];
    let ci = comp["control-implementations"].as_array().unwrap();

    assert_eq!(ci.len(), 1, "Must have exactly one control-implementations entry");

    let entry = &ci[0];

    // source matches provided source_profile
    assert_eq!(entry["source"], "./baselines/nist-800-53.json");

    // description follows expected pattern
    let desc = entry["description"].as_str().unwrap();
    assert!(
        desc.contains("Implementation narratives derived from"),
        "Description must follow template pattern. Got: {desc}"
    );

    // uuid is present
    assert!(entry["uuid"].is_string(), "control-implementation must have uuid");

    // implemented-requirements is populated
    let impl_reqs = entry["implemented-requirements"].as_array().unwrap();
    assert!(
        !impl_reqs.is_empty(),
        "implemented-requirements must be populated for a document with requirements"
    );
}

#[test]
fn component_pipeline_implemented_requirements_have_required_fields() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let impl_reqs = json["component-definition"]["components"][0]["control-implementations"][0]
        ["implemented-requirements"]
        .as_array()
        .unwrap();

    for (i, req) in impl_reqs.iter().enumerate() {
        assert!(req["uuid"].is_string(), "implemented-requirement[{i}] must have uuid");
        assert!(req["control-id"].is_string(), "implemented-requirement[{i}] must have control-id");
        assert!(
            req["description"].is_string(),
            "implemented-requirement[{i}] must have description"
        );

        // control-id must follow POL-{ABBR}-{NNN} format
        let control_id = req["control-id"].as_str().unwrap();
        assert!(
            control_id.starts_with("POL-"),
            "control-id[{i}] must start with 'POL-'. Got: {control_id}"
        );
    }
}

// ─── T025: Cross-Artifact Consistency Test ───────────────────────────────

#[test]
fn control_ids_match_between_catalog_and_component() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists());

    let dir = TempDir::new().unwrap();

    // Generate Catalog
    let catalog_path = dir.path().join("catalog.json");
    forge::pipeline::run_catalog_pipeline(fixture, Some(&catalog_path), 10 * 1024 * 1024)
        .expect("Catalog pipeline should succeed");
    let catalog_str = std::fs::read_to_string(&catalog_path).unwrap();
    let catalog_json: serde_json::Value = serde_json::from_str(&catalog_str).unwrap();

    // Generate Component Definition
    let component_path = dir.path().join("component.json");
    forge::pipeline::run_component_pipeline(
        fixture,
        Some(&component_path),
        10 * 1024 * 1024,
        "./baselines/nist.json",
    )
    .expect("Component pipeline should succeed");
    let component_str = std::fs::read_to_string(&component_path).unwrap();
    let component_json: serde_json::Value = serde_json::from_str(&component_str).unwrap();

    // Extract control-ids from Catalog (groups → controls → id)
    let mut catalog_ids: Vec<String> = Vec::new();
    if let Some(groups) = catalog_json["catalog"]["groups"].as_array() {
        for group in groups {
            if let Some(controls) = group["controls"].as_array() {
                for control in controls {
                    if let Some(id) = control["id"].as_str() {
                        catalog_ids.push(id.to_string());
                    }
                }
            }
        }
    }

    // Extract control-ids from Component Definition (control-implementations → implemented-requirements → control-id)
    let mut component_ids: Vec<String> = Vec::new();
    if let Some(ci) =
        component_json["component-definition"]["components"][0]["control-implementations"]
            .as_array()
    {
        for entry in ci {
            if let Some(impl_reqs) = entry["implemented-requirements"].as_array() {
                for req in impl_reqs {
                    if let Some(id) = req["control-id"].as_str() {
                        component_ids.push(id.to_string());
                    }
                }
            }
        }
    }

    assert!(!catalog_ids.is_empty(), "Catalog should have control-ids");
    assert!(!component_ids.is_empty(), "Component should have control-ids");

    // Both should have the same count
    assert_eq!(
        catalog_ids.len(),
        component_ids.len(),
        "Catalog ({}) and Component ({}) should produce same number of control-ids",
        catalog_ids.len(),
        component_ids.len()
    );

    // Control-ids should match exactly (same document, same traversal order)
    // Note: EC-2 fallback (REQ-{index}) is excluded since the fixture has stable_ids assigned
    for (i, (cat_id, comp_id)) in catalog_ids.iter().zip(component_ids.iter()).enumerate() {
        assert_eq!(
            cat_id, comp_id,
            "Control-id mismatch at position {i}: catalog={cat_id}, component={comp_id}"
        );
    }
}

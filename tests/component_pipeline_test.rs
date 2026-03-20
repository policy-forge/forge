use std::path::Path;

use forge::cli::OutputFormat;

/// Helper: run component pipeline on `full_policy.md` fixture, return parsed JSON.
fn run_component_pipeline_on_fixture(source_profile: &str) -> serde_json::Value {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists(), "Fixture file must exist: {}", fixture.display());

    let result = forge::pipeline::run_component_pipeline(
        fixture,
        10 * 1024 * 1024,
        Some(source_profile),
        &OutputFormat::Json,
        None,
    );
    assert!(result.is_ok(), "Pipeline failed on {}: {:?}", fixture.display(), result.unwrap_err());

    let output = result.unwrap();
    serde_json::from_str(&output.content)
        .unwrap_or_else(|e| panic!("Output is not valid JSON: {e}\nContent: {}", output.content))
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

    // source uses filename-only (sanitize_artifact_path)
    assert_eq!(entry["source"], "nist-800-53.json");

    // description follows expected pattern
    let desc = entry["description"].as_str().unwrap();
    assert!(
        desc.contains("Implementation narratives derived from"),
        "Description must follow template pattern. Got: {desc}"
    );

    // uuid is present
    assert!(entry["uuid"].is_string(), "control-implementation must have uuid");

    // implemented-requirements: full_policy.md has 14 controls (including atomized compound)
    let impl_reqs = entry["implemented-requirements"].as_array().unwrap();
    assert_eq!(
        impl_reqs.len(),
        14,
        "full_policy.md should produce exactly 14 implemented-requirements (including atomized compound). Got: {}",
        impl_reqs.len()
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

// ─── T020: Component Definition Trace Embedding Integration (WI-17) ───────

#[test]
fn component_pipeline_documentary_component_has_source_file_prop() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let comp = &json["component-definition"]["components"][0];

    let props = comp["props"].as_array().expect("Documentary component must have props");
    assert_eq!(props.len(), 1, "Must have exactly 1 source-file prop");
    assert_eq!(props[0]["name"], "source-file");
    assert_eq!(props[0]["ns"], "https://forge.policy-forge.github.io/ns/trace");

    let value = props[0]["value"].as_str().unwrap();
    assert!(
        value.contains("full_policy.md"),
        "source-file prop must reference the input file. Got: {value}"
    );
}

#[test]
fn component_pipeline_implemented_requirements_have_trace_props() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let impl_reqs = json["component-definition"]["components"][0]["control-implementations"][0]
        ["implemented-requirements"]
        .as_array()
        .unwrap();

    for (i, req) in impl_reqs.iter().enumerate() {
        let props = req["props"]
            .as_array()
            .unwrap_or_else(|| panic!("implemented-requirement[{i}] must have props"));
        // 3 trace props + 1 modality prop (WI-33)
        assert_eq!(
            props.len(),
            4,
            "implemented-requirement[{i}] must have 3 trace props + 1 modality prop"
        );

        // Verify trace props by name (not position, for robustness)
        let trace_props: Vec<_> = props.iter().filter(|p| p["ns"].as_str().is_some()).collect();
        assert_eq!(trace_props.len(), 3, "Must have 3 trace props with FORGE ns at [{i}]");

        let find_prop = |name: &str| props.iter().find(|p| p["name"].as_str() == Some(name));

        let source_file_prop =
            find_prop("source-file").unwrap_or_else(|| panic!("Missing source-file prop at [{i}]"));
        assert_eq!(
            source_file_prop["ns"], "https://forge.policy-forge.github.io/ns/trace",
            "source-file ns at [{i}]"
        );
        let file_val = source_file_prop["value"].as_str().unwrap();
        assert!(
            file_val.contains("full_policy.md"),
            "source-file at [{i}] must reference fixture. Got: {file_val}"
        );

        find_prop("source-section")
            .unwrap_or_else(|| panic!("Missing source-section prop at [{i}]"));

        let source_line_prop =
            find_prop("source-line").unwrap_or_else(|| panic!("Missing source-line prop at [{i}]"));
        let line_val = source_line_prop["value"].as_str().unwrap();
        let line_num: usize = line_val
            .parse()
            .unwrap_or_else(|_| panic!("source-line at [{i}] must be a number. Got: {line_val}"));
        assert!(line_num > 0, "source-line at [{i}] must be > 0. Got: {line_num}");

        // Verify modality prop (WI-33)
        let modality_prop =
            find_prop("modality").unwrap_or_else(|| panic!("Missing modality prop at [{i}]"));
        let mv = modality_prop["value"].as_str().unwrap_or("");
        assert!(
            mv == "normative" || mv == "advisory",
            "Modality at [{i}] must be 'normative' or 'advisory', got '{mv}'"
        );
    }
}

#[test]
fn component_pipeline_implemented_requirements_have_source_link() {
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let impl_reqs = json["component-definition"]["components"][0]["control-implementations"][0]
        ["implemented-requirements"]
        .as_array()
        .unwrap();

    for (i, req) in impl_reqs.iter().enumerate() {
        let links = req["links"]
            .as_array()
            .unwrap_or_else(|| panic!("implemented-requirement[{i}] must have links"));
        assert_eq!(links.len(), 1, "implemented-requirement[{i}] must have exactly 1 source link");

        assert_eq!(links[0]["rel"], "source", "Link rel at [{i}] must be 'source'");

        let href = links[0]["href"].as_str().unwrap();
        assert!(
            href.contains("full_policy.md"),
            "Link href at [{i}] must reference fixture. Got: {href}"
        );
        assert!(
            href.contains("#line="),
            "Link href at [{i}] must have #line= fragment. Got: {href}"
        );
    }
}

// ─── T002: Source-file prop must be filename-only (SEC-1) ─────────────────

#[test]
fn component_pipeline_source_file_prop_is_filename_only() {
    // T002 (SEC-1): source-file prop must NOT contain path separators (absolute path leakage)
    let json = run_component_pipeline_on_fixture("./baselines/nist-800-53.json");
    let comp = &json["component-definition"]["components"][0];

    let props = comp["props"].as_array().expect("Component must have props");
    let source_file_prop =
        props.iter().find(|p| p["name"] == "source-file").expect("Must have source-file prop");
    let value = source_file_prop["value"].as_str().unwrap();

    // Must be just a filename — no path separators
    assert!(
        !value.contains('/') && !value.contains('\\'),
        "SEC-1: source-file prop must be filename-only, not a path. Got: {value}"
    );
    assert_eq!(value, "full_policy.md", "Should be just the filename");

    // Also verify trace props on implemented-requirements are filename-only
    let impl_reqs = json["component-definition"]["components"][0]["control-implementations"][0]
        ["implemented-requirements"]
        .as_array()
        .unwrap();
    for (i, req) in impl_reqs.iter().enumerate() {
        let req_props = req["props"].as_array().unwrap();
        let sf = req_props.iter().find(|p| p["name"] == "source-file").unwrap();
        let sf_val = sf["value"].as_str().unwrap();
        assert!(
            !sf_val.contains('/') && !sf_val.contains('\\'),
            "SEC-1: implemented-requirement[{i}] source-file must be filename-only. Got: {sf_val}"
        );
    }
}

// ─── T025: Cross-Artifact Consistency Test ───────────────────────────────

fn extract_catalog_control_ids(catalog_json: &serde_json::Value) -> Vec<String> {
    catalog_json["catalog"]["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|group| {
            group["controls"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|control| control["id"].as_str().map(String::from))
        })
        .collect()
}

fn extract_component_control_ids(component_json: &serde_json::Value) -> Vec<String> {
    component_json["component-definition"]["components"][0]["control-implementations"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|entry| {
            entry["implemented-requirements"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|req| req["control-id"].as_str().map(String::from))
        })
        .collect()
}

#[test]
fn control_ids_match_between_catalog_and_component() {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists());

    // Generate Catalog
    let catalog_result =
        forge::pipeline::run_catalog_pipeline(fixture, 10 * 1024 * 1024, &OutputFormat::Json, None)
            .expect("Catalog pipeline should succeed");
    let catalog_json: serde_json::Value = serde_json::from_str(&catalog_result.content).unwrap();

    // Generate Component Definition
    let component_result = forge::pipeline::run_component_pipeline(
        fixture,
        10 * 1024 * 1024,
        Some("./baselines/nist.json"),
        &OutputFormat::Json,
        None,
    )
    .expect("Component pipeline should succeed");
    let component_json: serde_json::Value =
        serde_json::from_str(&component_result.content).unwrap();

    let catalog_ids = extract_catalog_control_ids(&catalog_json);
    let component_ids = extract_component_control_ids(&component_json);

    assert_eq!(
        catalog_ids.len(),
        14,
        "Catalog should have exactly 14 control-ids. Got: {}",
        catalog_ids.len()
    );
    assert_eq!(
        component_ids.len(),
        14,
        "Component should have exactly 14 control-ids. Got: {}",
        component_ids.len()
    );

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

// ─── T010: Component Pipeline with source_profile: None ───────────────────

#[test]
fn component_pipeline_none_source_profile_produces_empty_control_implementations() {
    // T010 (S-1, AC-7): Pipeline with source_profile: None → empty control-implementations.
    // With skip_serializing_if, empty control-implementations is omitted from output,
    // making the output valid OSCAL (field is optional in schema).
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists());

    let result = forge::pipeline::run_component_pipeline(
        fixture,
        10 * 1024 * 1024,
        None, // No source profile
        &OutputFormat::Json,
        None,
    );

    // Empty control-implementations is omitted (skip_serializing_if),
    // so the output passes schema validation.
    assert!(
        result.is_ok(),
        "Pipeline should succeed: empty control-implementations omitted, err: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert!(!output.content.is_empty(), "Output content should not be empty");
}

// ─── T021: Modality props in component definition output (WI-33) ──────────

/// T021 [US2] Integration test: verify modality prop appears on implemented-requirements
/// in component definition output from `033-mixed-modality.md` fixture.
#[test]
fn modality_props_present_in_implemented_requirements_for_mixed_fixture() {
    let fixture = std::path::Path::new("tests/fixtures/033-mixed-modality.md");
    assert!(fixture.exists(), "Fixture must exist: {}", fixture.display());

    let result = forge::pipeline::run_component_pipeline(
        fixture,
        10 * 1024 * 1024,
        Some("./baselines/nist-800-53.json"),
        &forge::cli::OutputFormat::Json,
        None,
    );
    assert!(result.is_ok(), "Pipeline failed: {:?}", result.unwrap_err());

    let output = result.unwrap();
    let json: serde_json::Value = serde_json::from_str(&output.content).unwrap();

    let components = json["component-definition"]["components"]
        .as_array()
        .expect("components should be an array");
    assert!(!components.is_empty(), "Expected at least one component");

    let mut impl_req_count = 0;
    let mut modality_count = 0;

    for component in components {
        let Some(control_impls) = component["control-implementations"].as_array() else { continue };
        for ci in control_impls {
            let Some(impl_reqs) = ci["implemented-requirements"].as_array() else { continue };
            for ir in impl_reqs {
                impl_req_count += 1;
                let props = ir["props"].as_array();
                let has_modality = props.is_some_and(|p| {
                    p.iter().any(|prop| prop["name"].as_str() == Some("modality"))
                });
                if has_modality {
                    modality_count += 1;
                }
            }
        }
    }

    assert!(impl_req_count > 0, "Expected at least one implemented-requirement");
    assert_eq!(
        modality_count, impl_req_count,
        "Every implemented-requirement should have a modality prop ({modality_count}/{impl_req_count} have it)"
    );
}

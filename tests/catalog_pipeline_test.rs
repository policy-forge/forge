use std::path::Path;

use tempfile::TempDir;

/// Helper: run pipeline on full_policy.md fixture, return parsed JSON
fn run_pipeline_on_fixture() -> serde_json::Value {
    let fixture = Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists(), "Fixture file must exist: {}", fixture.display());

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("catalog.json");

    let result =
        forge::pipeline::run_catalog_pipeline(fixture, Some(&output_path), 10 * 1024 * 1024);
    assert!(result.is_ok(), "Pipeline failed on {}: {:?}", fixture.display(), result.unwrap_err());

    let json_str = std::fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read output {}: {e}", output_path.display()));
    serde_json::from_str(&json_str)
        .unwrap_or_else(|e| panic!("Output is not valid JSON: {e}\nContent: {json_str}"))
}

/// T005 [US1] End-to-end smoke test: call run_catalog_pipeline with full_policy.md fixture,
/// capture output to temp file, parse JSON, assert OSCAL structure.
#[test]
fn smoke_test_full_pipeline_produces_valid_oscal_json() {
    let json = run_pipeline_on_fixture();

    // Assert top-level structure: catalog object with metadata, groups, and controls
    let catalog = &json["catalog"];
    assert!(catalog.is_object(), "JSON should have a 'catalog' object");
    assert!(catalog["metadata"].is_object(), "catalog should have 'metadata'");
    assert!(catalog["groups"].is_array(), "catalog should have 'groups'");
    assert!(catalog["uuid"].is_string(), "catalog should have 'uuid'");

    // SEC-1: Assert top-level JSON contains only expected OSCAL keys (no extraneous data)
    let top_keys: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert_eq!(top_keys, vec!["catalog"], "Top-level should only contain 'catalog' key");

    // Verify groups contain controls
    let groups = catalog["groups"].as_array().unwrap();
    assert!(!groups.is_empty(), "Should have at least one group");
    for group in groups {
        assert!(group["id"].is_string(), "Group should have 'id'");
        assert!(group["title"].is_string(), "Group should have 'title'");
    }
}

/// T012 [US3] Smoke test: groups contain the 3 top-level sections from fixture
#[test]
fn smoke_test_groups_contain_expected_sections() {
    let json = run_pipeline_on_fixture();
    let groups = json["catalog"]["groups"].as_array().expect("groups should be an array");

    let titles: Vec<&str> = groups.iter().filter_map(|g| g["title"].as_str()).collect();

    // full_policy.md has 3 top-level sections: Access Control, Data Protection, Incident Response
    assert!(
        titles.contains(&"Access Control"),
        "Groups should contain 'Access Control'. Got: {titles:?}"
    );
    assert!(
        titles.contains(&"Data Protection"),
        "Groups should contain 'Data Protection'. Got: {titles:?}"
    );
    assert!(
        titles.contains(&"Incident Response"),
        "Groups should contain 'Incident Response'. Got: {titles:?}"
    );
    // At least 3 groups for the 3 content sections
    assert!(
        groups.len() >= 3,
        "Should have at least 3 groups for content sections. Got: {}",
        groups.len()
    );
}

/// T013 [US3] Smoke test: metadata fields correctly populated from frontmatter
#[test]
fn smoke_test_metadata_fields_populated() {
    let json = run_pipeline_on_fixture();
    let metadata = &json["catalog"]["metadata"];

    // Title matches frontmatter
    assert_eq!(
        metadata["title"].as_str().unwrap(),
        "Sample Security Policy",
        "metadata.title should match frontmatter title"
    );

    // Version matches frontmatter
    assert_eq!(
        metadata["version"].as_str().unwrap(),
        "1.0.0",
        "metadata.version should match frontmatter version"
    );

    // OSCAL version is 1.2.0
    assert_eq!(
        metadata["oscal-version"].as_str().unwrap(),
        "1.2.0",
        "metadata.oscal-version should be '1.2.0'"
    );

    // last-modified is valid RFC 3339 timestamp
    let last_modified =
        metadata["last-modified"].as_str().expect("last-modified should be a string");
    assert!(
        chrono::DateTime::parse_from_rfc3339(last_modified).is_ok(),
        "last-modified should be a valid RFC 3339 timestamp, got: {last_modified}"
    );
}

/// T014 [US3] Smoke test: compound requirements atomized into separate controls
#[test]
fn smoke_test_compound_requirements_atomized() {
    let json = run_pipeline_on_fixture();
    let groups = json["catalog"]["groups"].as_array().unwrap();

    // Collect all control IDs and prose text across all groups
    let mut all_controls: Vec<(&str, String)> = Vec::new();
    for group in groups {
        if let Some(controls) = group["controls"].as_array() {
            for control in controls {
                let id = control["id"].as_str().unwrap_or("?");
                // Extract prose from parts
                let prose = control["parts"]
                    .as_array()
                    .and_then(|parts| parts.iter().find_map(|p| p["prose"].as_str()))
                    .unwrap_or("")
                    .to_string();
                all_controls.push((id, prose));
            }
        }
    }

    // The fixture contains: "Systems must log all authentication attempts and must log all privilege escalation events"
    // This compound statement should be atomized into 2+ separate controls
    let auth_logging_controls: Vec<_> = all_controls
        .iter()
        .filter(|(_, prose)| prose.contains("log") && prose.contains("authentication"))
        .collect();
    let priv_logging_controls: Vec<_> = all_controls
        .iter()
        .filter(|(_, prose)| prose.contains("log") && prose.contains("privilege"))
        .collect();

    assert!(
        !auth_logging_controls.is_empty(),
        "Should have a control about logging authentication attempts (AC-7)"
    );
    assert!(
        !priv_logging_controls.is_empty(),
        "Should have a control about logging privilege escalation (AC-7)"
    );

    // Verify atomization: auth and priv controls must not share the same set of IDs
    if !auth_logging_controls.is_empty() && !priv_logging_controls.is_empty() {
        let auth_ids: Vec<_> = auth_logging_controls.iter().map(|(id, _)| *id).collect();
        let priv_ids: Vec<_> = priv_logging_controls.iter().map(|(id, _)| *id).collect();
        assert_ne!(
            auth_ids, priv_ids,
            "Compound requirements should be atomized into separate controls with different IDs"
        );
    }
}

/// T015 [US3] Edge case: input with no sections produces catalog with empty groups
#[test]
fn pipeline_no_sections_produces_empty_groups() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("no_sections.md");
    // Content with no headings — just plain text requirements
    std::fs::write(&path, "Some requirement without any section heading.\n").unwrap();

    let output_path = dir.path().join("output.json");
    let result = forge::pipeline::run_catalog_pipeline(&path, Some(&output_path), 10 * 1024 * 1024);

    // Pipeline should either succeed with empty/absent groups or fail gracefully
    match result {
        Ok(()) => {
            let json_str = std::fs::read_to_string(&output_path).unwrap();
            let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            // groups may be absent (null) or an empty array — both are acceptable (EC-6)
            let groups = json["catalog"]["groups"].as_array();
            match groups {
                None => {} // groups key omitted — acceptable
                Some(arr) => assert!(
                    arr.is_empty(),
                    "No sections should produce empty groups array (EC-6), got: {arr:?}"
                ),
            }
        }
        Err(e) => {
            // If it fails, the error should be descriptive
            let err_msg = e.to_string();
            assert!(!err_msg.is_empty(), "Error for no-sections input should be descriptive");
        }
    }
}

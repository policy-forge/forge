//! Integration tests: multi-format round-trip semantic equivalence (WI-35, US1, M-1).
//!
//! Verifies that `forge export` round-trips preserve all semantic content for both
//! Catalog and Component Definition artifacts across JSON→XML→JSON and JSON→YAML→JSON paths.
//! Uses the CLI subprocess pattern (env!("CARGO_BIN_EXE_forge")) for end-to-end coverage.

use forge::testing::assert_semantic_equivalence;
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run_forge(args: &[&str]) -> std::process::Output {
    let output = forge_bin().args(args).output().expect("failed to execute forge");
    if !output.status.success() {
        panic!(
            "forge {:?} failed (exit {})\nstdout: {}\nstderr: {}",
            args,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}

fn read_json(path: &std::path::Path) -> Value {
    let content = fs::read_to_string(path).expect("failed to read JSON file");
    serde_json::from_str(&content).expect("failed to parse JSON")
}

/// Remove `control-implementations` from all components in a component-definition JSON.
///
/// XML serialization intentionally omits this field (WI-28 normalization pattern, EC-5).
fn clear_control_implementations(value: &mut Value) {
    if let Some(comp_def) = value.pointer_mut("/component-definition") {
        if let Some(components) = comp_def.get_mut("components").and_then(Value::as_array_mut) {
            for component in components {
                if let Some(obj) = component.as_object_mut() {
                    obj.remove("control-implementations");
                }
            }
        }
    }
}

// ── M-1 / AC-1: Catalog JSON → XML → JSON ────────────────────────────────────

#[test]
fn catalog_json_xml_json_round_trip() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("catalog.json");
    let xml_path = dir.path().join("catalog.xml");
    let rt_path = dir.path().join("catalog_rt.json");

    // Step 1: convert Markdown → Catalog JSON
    run_forge(&[
        "convert",
        "tests/fixtures/golden/small/input.md",
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        json_path.to_str().unwrap(),
    ]);

    // Step 2: JSON → XML
    run_forge(&[
        "export",
        json_path.to_str().unwrap(),
        "--format",
        "xml",
        "--output",
        xml_path.to_str().unwrap(),
    ]);

    // Step 3: XML → JSON (round-tripped)
    run_forge(&[
        "export",
        xml_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        rt_path.to_str().unwrap(),
    ]);

    // Step 4: semantic equivalence
    let original = read_json(&json_path);
    let round_tripped = read_json(&rt_path);
    let result = assert_semantic_equivalence(&original, &round_tripped);
    assert!(
        result.is_equivalent,
        "Catalog JSON→XML→JSON round-trip failed.\nDifferences:\n{}",
        result
            .differences
            .iter()
            .map(|d| format!("  {}: {}", d.path, d.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── M-1 / AC-2: Catalog JSON → YAML → JSON ───────────────────────────────────

#[test]
fn catalog_json_yaml_json_round_trip() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("catalog.json");
    let yaml_path = dir.path().join("catalog.yaml");
    let rt_path = dir.path().join("catalog_rt.json");

    run_forge(&[
        "convert",
        "tests/fixtures/golden/small/input.md",
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        json_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        json_path.to_str().unwrap(),
        "--format",
        "yaml",
        "--output",
        yaml_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        yaml_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        rt_path.to_str().unwrap(),
    ]);

    let original = read_json(&json_path);
    let round_tripped = read_json(&rt_path);
    let result = assert_semantic_equivalence(&original, &round_tripped);
    assert!(
        result.is_equivalent,
        "Catalog JSON→YAML→JSON round-trip failed.\nDifferences:\n{}",
        result
            .differences
            .iter()
            .map(|d| format!("  {}: {}", d.path, d.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── M-1 / EC-5: Component Definition JSON → XML → JSON (normalized) ──────────

#[test]
fn component_definition_json_xml_json_round_trip() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("component.json");
    let xml_path = dir.path().join("component.xml");
    let rt_path = dir.path().join("component_rt.json");

    run_forge(&[
        "convert",
        "tests/fixtures/full_policy.md",
        "--strategy",
        "component",
        "--source-profile",
        "tests/fixtures/sample_profile.json",
        "--format",
        "json",
        "--output",
        json_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        json_path.to_str().unwrap(),
        "--format",
        "xml",
        "--output",
        xml_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        xml_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        rt_path.to_str().unwrap(),
    ]);

    let mut original = read_json(&json_path);
    let mut round_tripped = read_json(&rt_path);

    // Normalize: XML intentionally omits control-implementations (WI-28/EC-5)
    clear_control_implementations(&mut original);
    clear_control_implementations(&mut round_tripped);

    let result = assert_semantic_equivalence(&original, &round_tripped);
    assert!(
        result.is_equivalent,
        "Component Definition JSON→XML→JSON round-trip failed (after normalization).\nDifferences:\n{}",
        result
            .differences
            .iter()
            .map(|d| format!("  {}: {}", d.path, d.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── M-1 / EC-5: Component Definition JSON → YAML → JSON (full) ───────────────

#[test]
fn component_definition_json_yaml_json_round_trip() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("component.json");
    let yaml_path = dir.path().join("component.yaml");
    let rt_path = dir.path().join("component_rt.json");

    run_forge(&[
        "convert",
        "tests/fixtures/full_policy.md",
        "--strategy",
        "component",
        "--source-profile",
        "tests/fixtures/sample_profile.json",
        "--format",
        "json",
        "--output",
        json_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        json_path.to_str().unwrap(),
        "--format",
        "yaml",
        "--output",
        yaml_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        yaml_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        rt_path.to_str().unwrap(),
    ]);

    // YAML preserves all fields including control-implementations (no normalization)
    let original = read_json(&json_path);
    let round_tripped = read_json(&rt_path);
    let result = assert_semantic_equivalence(&original, &round_tripped);
    assert!(
        result.is_equivalent,
        "Component Definition JSON→YAML→JSON round-trip failed.\nDifferences:\n{}",
        result
            .differences
            .iter()
            .map(|d| format!("  {}: {}", d.path, d.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── EC-1: Empty-groups catalog round-trip (JSON → XML → JSON) ────────────────

#[test]
fn empty_groups_catalog_round_trip() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("empty_groups.json");
    let xml_path = dir.path().join("empty_groups.xml");
    let rt_path = dir.path().join("empty_groups_rt.json");

    // Construct minimal OSCAL Catalog with one group that has no controls.
    // UUID uses valid v4 format (version nibble = 4, variant nibble in [89ab]).
    // Note: `controls` key is intentionally omitted (not `[]`) because OscalGroup serializes
    // with skip_serializing_if = "Vec::is_empty", so both original and round-tripped JSON
    // will have no `controls` key — the semantic equivalence comparison will pass.
    let catalog_json = serde_json::json!({
        "catalog": {
            "uuid": "a1b2c3d4-e5f6-4890-abcd-ef1234567890",
            "metadata": {
                "title": "Empty Groups Test Catalog",
                "last-modified": "2026-01-01T00:00:00+00:00",
                "version": "1.0.0",
                "oscal-version": "1.2.0"
            },
            "groups": [
                {
                    "id": "test-group",
                    "title": "Test Group With No Controls"
                }
            ]
        }
    });
    fs::write(&json_path, serde_json::to_string_pretty(&catalog_json).unwrap()).unwrap();

    // Round-trip via XML
    run_forge(&[
        "export",
        json_path.to_str().unwrap(),
        "--format",
        "xml",
        "--output",
        xml_path.to_str().unwrap(),
    ]);
    run_forge(&[
        "export",
        xml_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        rt_path.to_str().unwrap(),
    ]);

    let original = read_json(&json_path);
    let round_tripped = read_json(&rt_path);
    let result = assert_semantic_equivalence(&original, &round_tripped);
    assert!(
        result.is_equivalent,
        "Empty-groups Catalog JSON→XML→JSON round-trip failed.\nDifferences:\n{}",
        result
            .differences
            .iter()
            .map(|d| format!("  {}: {}", d.path, d.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

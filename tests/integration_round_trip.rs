//! Integration tests: multi-format round-trip semantic equivalence (WI-35, US1, M-1).
//!
//! Verifies that `forge export` round-trips preserve all semantic content for both
//! Catalog and Component Definition artifacts across JSON→XML→JSON and JSON→YAML→JSON paths.
//! Uses the CLI subprocess pattern (`env!("CARGO_BIN_EXE_forge")`) for end-to-end coverage.

use forge::testing::assert_semantic_equivalence;
use serde_json::Value;
use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run_forge(args: &[&str]) -> std::process::Output {
    let mut child = forge_bin()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute forge");
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if child.try_wait().expect("failed to poll forge").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("failed to kill timed-out forge");
            let output =
                child.wait_with_output().expect("failed to collect timed-out forge output");
            panic!(
                "forge {:?} timed out after 120s\nstdout: {}\nstderr: {}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("failed to collect forge output");
    assert!(
        output.status.success(),
        "forge {:?} failed (exit {})\nstdout: {}\nstderr: {}",
        args,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn read_json(path: &std::path::Path) -> Value {
    let content = fs::read_to_string(path).expect("failed to read JSON file");
    serde_json::from_str(&content).expect("failed to parse JSON")
}

/// Remove `control-implementations` from all components in a component-definition JSON.
///
/// XML serialization intentionally omits this field (WI-28 normalization pattern, EC-5).
fn clear_control_implementations(value: &mut Value) -> usize {
    let component_definition = value
        .pointer_mut("/component-definition")
        .expect("component-definition root must be present for normalization");
    let components = component_definition
        .get_mut("components")
        .and_then(Value::as_array_mut)
        .expect("component-definition.components must be an array for normalization");

    components
        .iter_mut()
        .map(|component| {
            usize::from(
                component
                    .as_object_mut()
                    .expect("component-definition.components entries must be objects")
                    .remove("control-implementations")
                    .is_some(),
            )
        })
        .sum()
}

fn fixture_path(relative_path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn assert_round_trip_preserves(
    artifact_name: &str,
    input_path: &std::path::Path,
    strategy: &str,
    source_profile: Option<&std::path::Path>,
    intermediate_format: &str,
    normalize_control_implementations: bool,
) {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join(format!("{artifact_name}.json"));
    let intermediate_path = dir.path().join(format!("{artifact_name}.{intermediate_format}"));
    let round_tripped_path = dir.path().join(format!("{artifact_name}_rt.json"));

    let input = input_path.to_str().expect("fixture path must be UTF-8");
    let mut convert_args = vec!["convert", input, "--strategy", strategy];
    if let Some(source_profile) = source_profile {
        convert_args.extend([
            "--source-profile",
            source_profile.to_str().expect("source profile path must be UTF-8"),
        ]);
    }
    convert_args.extend([
        "--format",
        "json",
        "--output",
        json_path.to_str().expect("output path must be UTF-8"),
    ]);
    run_forge(&convert_args);

    run_forge(&[
        "export",
        json_path.to_str().expect("output path must be UTF-8"),
        "--format",
        intermediate_format,
        "--output",
        intermediate_path.to_str().expect("output path must be UTF-8"),
    ]);
    run_forge(&[
        "export",
        intermediate_path.to_str().expect("output path must be UTF-8"),
        "--format",
        "json",
        "--output",
        round_tripped_path.to_str().expect("output path must be UTF-8"),
    ]);

    let mut original = read_json(&json_path);
    let mut round_tripped = read_json(&round_tripped_path);
    if normalize_control_implementations {
        let original_removals = clear_control_implementations(&mut original);
        clear_control_implementations(&mut round_tripped);
        assert!(
            original_removals > 0,
            "component normalization must remove at least one control-implementations field"
        );
    }

    let result = assert_semantic_equivalence(&original, &round_tripped);
    assert!(
        result.is_equivalent,
        "{artifact_name} JSON→{intermediate_format}→JSON round-trip failed.\nDifferences:\n{}",
        result
            .differences
            .iter()
            .map(|difference| format!("  {}: {}", difference.path, difference.description))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ── M-1 / AC-1: Catalog JSON → XML → JSON ────────────────────────────────────

#[test]
fn catalog_json_xml_json_round_trip() {
    assert_round_trip_preserves(
        "catalog",
        &fixture_path("tests/fixtures/golden/small/input.md"),
        "catalog",
        None,
        "xml",
        false,
    );
}

// ── M-1 / AC-2: Catalog JSON → YAML → JSON ───────────────────────────────────

#[test]
fn catalog_json_yaml_json_round_trip() {
    assert_round_trip_preserves(
        "catalog",
        &fixture_path("tests/fixtures/golden/small/input.md"),
        "catalog",
        None,
        "yaml",
        false,
    );
}

// ── M-1 / EC-5: Component Definition JSON → XML → JSON (normalized) ──────────

#[test]
fn component_definition_json_xml_json_round_trip() {
    let input = fixture_path("tests/fixtures/full_policy.md");
    let source_profile = fixture_path("tests/fixtures/sample_profile.json");
    assert_round_trip_preserves(
        "component",
        &input,
        "component",
        Some(&source_profile),
        "xml",
        // XML intentionally omits control-implementations (WI-28/EC-5).
        true,
    );
}

// ── M-1 / EC-5: Component Definition JSON → YAML → JSON (full) ───────────────

#[test]
fn component_definition_json_yaml_json_round_trip() {
    let input = fixture_path("tests/fixtures/full_policy.md");
    let source_profile = fixture_path("tests/fixtures/sample_profile.json");
    assert_round_trip_preserves(
        "component",
        &input,
        "component",
        Some(&source_profile),
        "yaml",
        false,
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

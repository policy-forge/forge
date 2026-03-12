//! Golden-file regression tests for the FORGE pipeline (WI-21).
//!
//! Compares the FORGE pipeline's actual OSCAL JSON output against hand-verified
//! expected outputs for Markdown policy fixtures of varying complexity.
//!
//! Uses `insta` snapshot testing with custom UUID/timestamp normalization.
//! Measures extraction accuracy against a >= 95% target (MS-4 exit criterion).
//!
//! ## Fixture Layout
//!
//! ```text
//! tests/fixtures/golden/
//!   small/    — 1 section, 3-5 requirements
//!   medium/   — 3-5 sections, 10-15 requirements, citations
//!   complex/  — 5+ sections, 20+ requirements, citations, cross-refs
//! ```
//!
//! ## Running
//!
//! ```bash
//! cargo test golden                    # Run all golden-file tests
//! cargo test golden -- --nocapture     # With accuracy reports
//! cargo mutants -- --test golden       # Mutation testing for test quality
//! cargo insta review                   # Review pending snapshots
//! ```
//!
//! ## Updating Golden Files
//!
//! After intentional pipeline changes:
//! - For insta snapshots: `cargo insta test` then `cargo insta review`
//! - For expected JSON files: `UPDATE_GOLDEN_FILES=1 cargo test golden`

use std::path::Path;
use std::sync::LazyLock;

use forge::cli::OutputFormat;
use regex::Regex;
use serde_json::{Map, Value, json};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Fixed UUID placeholder used during normalization.
const NORMALIZED_UUID: &str = "00000000-0000-0000-0000-000000000000";

/// Fixed reference timestamp for test normalization; replaces `last-modified` fields (not current date).
const NORMALIZED_TIMESTAMP: &str = "2026-01-01T00:00:00Z";

/// Minimum extraction accuracy percentage required to pass (PRD M-8).
/// Inclusive threshold: accuracy >= 95.0% passes (see research.md R-7).
const ACCURACY_THRESHOLD: f64 = 95.0;

/// Fixed path placeholder for machine-specific absolute paths in source-file
/// props and link hrefs, ensuring snapshots are portable across environments.
const NORMALIZED_PATH: &str = "NORMALIZED_PATH";

/// Maximum input file size for pipeline tests (10 MB).
const MAX_INPUT_SIZE: u64 = 10 * 1024 * 1024;

/// Pre-compiled UUID regex (case-insensitive, RFC 4122 format).
static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .expect("UUID regex is valid")
});

// ---------------------------------------------------------------------------
// Fixture loading helper (EC-6: descriptive errors, not panics)
// ---------------------------------------------------------------------------

/// Load a fixture file, returning a descriptive error if missing or unreadable.
fn load_fixture(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to load fixture '{}': {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Normalization (stub — T005 implements)
// ---------------------------------------------------------------------------

/// Normalize non-deterministic fields in OSCAL JSON for stable comparison.
///
/// Recursively walks the JSON tree and:
/// 1. Replaces UUID-format string values with [`NORMALIZED_UUID`]
/// 2. Replaces `last-modified` field values with [`NORMALIZED_TIMESTAMP`]
/// 3. Object keys are inherently sorted by `serde_json::Map` (`BTreeMap`)
///
/// Idempotent: `normalize(normalize(v)) == normalize(v)`
fn normalize_for_comparison(json: &Value) -> Value {
    normalize_value(json, None)
}

/// Recursively normalize a JSON value, with optional parent key context.
fn normalize_value(value: &Value, parent_key: Option<&str>) -> Value {
    match value {
        Value::String(s) => {
            if parent_key == Some("last-modified") {
                Value::String(NORMALIZED_TIMESTAMP.to_string())
            } else if UUID_RE.is_match(s) {
                Value::String(NORMALIZED_UUID.to_string())
            } else if parent_key == Some("href") {
                // Normalize absolute path hrefs (Unix and Windows): keep #fragment, replace path
                let (path_part, fragment) = match s.find('#') {
                    Some(idx) => (&s[..idx], &s[idx..]),
                    None => (s.as_str(), ""),
                };
                if Path::new(path_part).is_absolute() {
                    Value::String(format!("{NORMALIZED_PATH}{fragment}"))
                } else {
                    value.clone()
                }
            } else {
                value.clone()
            }
        }
        Value::Object(map) => {
            // Check if this is a prop object with name="source-file" — normalize its value
            let is_source_file_prop =
                map.get("name").and_then(Value::as_str).is_some_and(|n| n == "source-file");

            let normalized: Map<String, Value> = map
                .iter()
                .map(|(k, v)| {
                    if is_source_file_prop && k == "value" {
                        (k.clone(), Value::String(NORMALIZED_PATH.to_string()))
                    } else {
                        (k.clone(), normalize_value(v, Some(k)))
                    }
                })
                .collect();
            Value::Object(normalized)
        }
        Value::Array(arr) => {
            let normalized: Vec<Value> = arr.iter().map(|v| normalize_value(v, None)).collect();
            Value::Array(normalized)
        }
        _ => value.clone(),
    }
}

// ---------------------------------------------------------------------------
// Accuracy measurement (stub — T007 implements)
// ---------------------------------------------------------------------------

/// Extraction accuracy result for a single fixture and strategy.
#[derive(Debug)]
struct AccuracyReport {
    expected_count: usize,
    correct_count: usize,
    accuracy_pct: f64,
    missed_requirements: Vec<String>,
}

/// Extract control IDs from an OSCAL JSON value based on strategy.
///
/// - Catalog: `$.catalog.groups[*].controls[*].id`
/// - Component: `$.component-definition.components[*].control-implementations[*].implemented-requirements[*].control-id`
fn extract_control_ids(json: &Value, strategy: &str) -> Vec<String> {
    let mut ids = Vec::new();
    match strategy {
        "catalog" => {
            if let Some(groups) = json.pointer("/catalog/groups").and_then(Value::as_array) {
                for group in groups {
                    if let Some(controls) = group.get("controls").and_then(Value::as_array) {
                        for control in controls {
                            if let Some(id) = control.get("id").and_then(Value::as_str) {
                                ids.push(id.to_string());
                            }
                        }
                    }
                }
            }
        }
        "component" => {
            if let Some(components) =
                json.pointer("/component-definition/components").and_then(Value::as_array)
            {
                for component in components {
                    if let Some(impls) =
                        component.get("control-implementations").and_then(Value::as_array)
                    {
                        for ci in impls {
                            if let Some(reqs) =
                                ci.get("implemented-requirements").and_then(Value::as_array)
                            {
                                for req in reqs {
                                    if let Some(id) = req.get("control-id").and_then(Value::as_str)
                                    {
                                        ids.push(id.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    ids
}

/// Measure extraction accuracy by comparing control IDs in expected vs actual.
fn measure_accuracy(
    _fixture_name: &str,
    strategy: &str,
    expected: &Value,
    actual: &Value,
) -> AccuracyReport {
    let expected_ids = extract_control_ids(expected, strategy);
    let actual_ids = extract_control_ids(actual, strategy);

    let expected_count = expected_ids.len();

    if expected_count == 0 {
        return AccuracyReport {
            expected_count: 0,
            correct_count: 0,
            accuracy_pct: 100.0,
            missed_requirements: Vec::new(),
        };
    }

    let actual_set: std::collections::HashSet<&str> =
        actual_ids.iter().map(String::as_str).collect();

    let mut correct_count = 0;
    let mut missed_requirements = Vec::new();

    for id in &expected_ids {
        if actual_set.contains(id.as_str()) {
            correct_count += 1;
        } else {
            missed_requirements.push(id.clone());
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let accuracy_pct = (correct_count as f64 / expected_count as f64) * 100.0;

    AccuracyReport { expected_count, correct_count, accuracy_pct, missed_requirements }
}

// ---------------------------------------------------------------------------
// Unit tests — Normalization (T004: RED)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod normalization_tests {
    use super::*;

    #[test]
    fn replaces_uuid_v4_format() {
        let input = json!({"uuid": "550e8400-e29b-41d4-a716-446655440000"});
        let result = normalize_for_comparison(&input);
        assert_eq!(result["uuid"], NORMALIZED_UUID);
    }

    #[test]
    fn replaces_uuid_v5_format() {
        let input = json!({"id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8"});
        let result = normalize_for_comparison(&input);
        assert_eq!(result["id"], NORMALIZED_UUID);
    }

    #[test]
    fn replaces_last_modified_timestamp() {
        let input = json!({"last-modified": "2026-02-14T10:30:00.000Z"});
        let result = normalize_for_comparison(&input);
        assert_eq!(result["last-modified"], NORMALIZED_TIMESTAMP);
    }

    #[test]
    fn preserves_non_uuid_strings() {
        let input = json!({
            "title": "Access Control Policy",
            "prose": "All users must authenticate"
        });
        let result = normalize_for_comparison(&input);
        assert_eq!(result["title"], "Access Control Policy");
        assert_eq!(result["prose"], "All users must authenticate");
    }

    #[test]
    fn handles_nested_objects_and_arrays() {
        let input = json!({
            "catalog": {
                "uuid": "550e8400-e29b-41d4-a716-446655440000",
                "groups": [
                    {
                        "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
                        "title": "AC"
                    }
                ]
            }
        });
        let result = normalize_for_comparison(&input);
        assert_eq!(result["catalog"]["uuid"], NORMALIZED_UUID);
        assert_eq!(result["catalog"]["groups"][0]["id"], NORMALIZED_UUID);
        assert_eq!(result["catalog"]["groups"][0]["title"], "AC");
    }

    #[test]
    fn idempotent() {
        let input = json!({
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "last-modified": "2026-02-14T10:30:00Z",
            "title": "Test"
        });
        let once = normalize_for_comparison(&input);
        let twice = normalize_for_comparison(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn handles_null_and_empty() {
        let input = json!({"empty": "", "nothing": null, "num": 42, "flag": true});
        let result = normalize_for_comparison(&input);
        assert_eq!(result["empty"], "");
        assert!(result["nothing"].is_null());
        assert_eq!(result["num"], 42);
        assert_eq!(result["flag"], true);
    }
}

// ---------------------------------------------------------------------------
// Unit tests — Accuracy measurement (T006: RED)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod accuracy_tests {
    use super::*;

    fn make_catalog(control_ids: &[&str]) -> Value {
        let controls: Vec<Value> = control_ids
            .iter()
            .map(|id| json!({"id": id, "title": format!("Control {id}")}))
            .collect();
        json!({
            "catalog": {
                "groups": [{"id": "g1", "controls": controls}]
            }
        })
    }

    fn make_component_def(control_ids: &[&str]) -> Value {
        let impl_reqs: Vec<Value> =
            control_ids.iter().map(|id| json!({"control-id": id, "uuid": "test"})).collect();
        json!({
            "component-definition": {
                "components": [{
                    "control-implementations": [{
                        "implemented-requirements": impl_reqs
                    }]
                }]
            }
        })
    }

    #[test]
    fn catalog_100_percent_accuracy() {
        let expected = make_catalog(&["ac-1", "ac-2", "ac-3"]);
        let actual = make_catalog(&["ac-1", "ac-2", "ac-3"]);
        let report = measure_accuracy("test", "catalog", &expected, &actual);
        assert_eq!(report.expected_count, 3);
        assert_eq!(report.correct_count, 3);
        assert!((report.accuracy_pct - 100.0).abs() < f64::EPSILON);
        assert!(report.missed_requirements.is_empty());
    }

    #[test]
    fn catalog_partial_accuracy() {
        let expected = make_catalog(&["ac-1", "ac-2", "ac-3", "ac-4"]);
        let actual = make_catalog(&["ac-1", "ac-3"]);
        let report = measure_accuracy("test", "catalog", &expected, &actual);
        assert_eq!(report.expected_count, 4);
        assert_eq!(report.correct_count, 2);
        assert!((report.accuracy_pct - 50.0).abs() < f64::EPSILON);
        assert!(report.missed_requirements.contains(&"ac-2".to_string()));
        assert!(report.missed_requirements.contains(&"ac-4".to_string()));
    }

    #[test]
    fn catalog_zero_accuracy() {
        let expected = make_catalog(&["ac-1", "ac-2"]);
        let actual = make_catalog(&["dp-1"]);
        let report = measure_accuracy("test", "catalog", &expected, &actual);
        assert_eq!(report.correct_count, 0);
        assert!((report.accuracy_pct).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_expected_returns_100() {
        let expected = make_catalog(&[]);
        let actual = make_catalog(&["ac-1"]);
        let report = measure_accuracy("test", "catalog", &expected, &actual);
        assert_eq!(report.expected_count, 0);
        assert!((report.accuracy_pct - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn component_accuracy() {
        let expected = make_component_def(&["ac-1", "ac-2", "ac-3"]);
        let actual = make_component_def(&["ac-1", "ac-2"]);
        let report = measure_accuracy("test", "component", &expected, &actual);
        assert_eq!(report.expected_count, 3);
        assert_eq!(report.correct_count, 2);
        assert!(report.missed_requirements.contains(&"ac-3".to_string()));
    }

    #[test]
    fn missed_requirements_reported_by_id() {
        let expected = make_catalog(&["ac-1", "ac-2", "dp-1"]);
        let actual = make_catalog(&["ac-1"]);
        let report = measure_accuracy("test", "catalog", &expected, &actual);
        assert_eq!(report.missed_requirements.len(), 2);
        assert!(report.missed_requirements.contains(&"ac-2".to_string()));
        assert!(report.missed_requirements.contains(&"dp-1".to_string()));
    }

    // EC-4: boundary condition — exactly 95.0% must PASS, 90.0% must FAIL
    #[test]
    fn boundary_95_percent_passes() {
        // 19/20 = 95.0% — must pass threshold
        let ids: Vec<String> = (1..=20).map(|i| format!("c-{i}")).collect();
        let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let expected = make_catalog(&id_refs);
        let actual_ids: Vec<String> = (1..=19).map(|i| format!("c-{i}")).collect();
        let actual_refs: Vec<&str> = actual_ids.iter().map(String::as_str).collect();
        let actual = make_catalog(&actual_refs);
        let report = measure_accuracy("boundary", "catalog", &expected, &actual);
        assert!(
            report.accuracy_pct >= ACCURACY_THRESHOLD,
            "95.0% should pass: got {:.1}%",
            report.accuracy_pct
        );
    }

    #[test]
    fn boundary_90_percent_fails() {
        // 18/20 = 90.0% — must fail threshold
        let ids: Vec<String> = (1..=20).map(|i| format!("c-{i}")).collect();
        let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let expected = make_catalog(&id_refs);
        let actual_ids: Vec<String> = (1..=18).map(|i| format!("c-{i}")).collect();
        let actual_refs: Vec<&str> = actual_ids.iter().map(String::as_str).collect();
        let actual = make_catalog(&actual_refs);
        let report = measure_accuracy("boundary", "catalog", &expected, &actual);
        assert!(
            report.accuracy_pct < ACCURACY_THRESHOLD,
            "90.0% should fail: got {:.1}%",
            report.accuracy_pct
        );
    }
}

// ---------------------------------------------------------------------------
// Fixture loading tests (EC-6)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod fixture_loading_tests {
    use super::*;

    #[test]
    fn missing_fixture_returns_descriptive_error() {
        let result = load_fixture(Path::new("tests/fixtures/golden/nonexistent/input.md"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("nonexistent/input.md"), "Error should contain file path, got: {err}");
    }
}

// ---------------------------------------------------------------------------
// Golden-file regression tests — Catalog strategy (T010, T013, T016)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod golden_catalog_tests {
    use super::*;

    /// Run catalog pipeline on a fixture, return (normalized JSON, raw actual JSON).
    fn run_catalog(fixture_dir: &str) -> (Value, Value) {
        let input_path = Path::new("tests/fixtures/golden").join(fixture_dir).join("input.md");
        assert!(input_path.exists(), "Fixture missing: {}", input_path.display());

        let dir = TempDir::new().expect("create temp dir");
        let output_path = dir.path().join("catalog.json");

        forge::pipeline::run_catalog_pipeline(
            &input_path,
            Some(&output_path),
            MAX_INPUT_SIZE,
            &OutputFormat::Json,
            None,
        )
        .unwrap_or_else(|e| panic!("Catalog pipeline failed on {fixture_dir}: {e}"));

        let json_str = std::fs::read_to_string(&output_path)
            .unwrap_or_else(|e| panic!("Failed to read output: {e}"));
        let actual: Value =
            serde_json::from_str(&json_str).unwrap_or_else(|e| panic!("Invalid JSON output: {e}"));
        let normalized = normalize_for_comparison(&actual);

        (normalized, actual)
    }

    /// Load expected catalog JSON and measure accuracy.
    fn assert_accuracy(fixture_dir: &str, actual: &Value) {
        let expected_path =
            Path::new("tests/fixtures/golden").join(fixture_dir).join("expected-catalog.json");
        let expected_str = load_fixture(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to load expected catalog: {e}"));
        let expected: Value = serde_json::from_str(&expected_str)
            .unwrap_or_else(|e| panic!("Invalid expected JSON: {e}"));

        let report = measure_accuracy(fixture_dir, "catalog", &expected, actual);
        eprintln!(
            "[{fixture_dir}/catalog] accuracy: {:.1}% ({}/{}) missed: {:?}",
            report.accuracy_pct,
            report.correct_count,
            report.expected_count,
            report.missed_requirements
        );
        assert!(
            report.accuracy_pct >= ACCURACY_THRESHOLD,
            "{fixture_dir}/catalog accuracy {:.1}% < {ACCURACY_THRESHOLD}% threshold. \
             Missed: {:?}",
            report.accuracy_pct,
            report.missed_requirements
        );
    }

    #[test]
    fn golden_small_catalog() {
        let (normalized, actual) = run_catalog("small");
        insta::assert_json_snapshot!("small_catalog", normalized);
        assert_accuracy("small", &actual);
    }

    #[test]
    fn golden_medium_catalog() {
        let (normalized, actual) = run_catalog("medium");
        insta::assert_json_snapshot!("medium_catalog", normalized);
        assert_accuracy("medium", &actual);
    }

    #[test]
    fn golden_complex_catalog() {
        let (normalized, actual) = run_catalog("complex");
        insta::assert_json_snapshot!("complex_catalog", normalized);
        assert_accuracy("complex", &actual);
    }
}

// ---------------------------------------------------------------------------
// Golden-file regression tests — Component strategy (T018, T020, T022)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod golden_component_tests {
    use super::*;

    /// Default source-profile reference used for component pipeline tests.
    const SOURCE_PROFILE: &str = "./baselines/nist-800-53.json";

    /// Run component pipeline on a fixture, return (normalized JSON, raw actual JSON).
    fn run_component(fixture_dir: &str) -> (Value, Value) {
        let input_path = Path::new("tests/fixtures/golden").join(fixture_dir).join("input.md");
        assert!(input_path.exists(), "Fixture missing: {}", input_path.display());

        let dir = TempDir::new().expect("create temp dir");
        let output_path = dir.path().join("component.json");

        forge::pipeline::run_component_pipeline(
            &input_path,
            Some(&output_path),
            MAX_INPUT_SIZE,
            Some(SOURCE_PROFILE),
            &OutputFormat::Json,
            None,
        )
        .unwrap_or_else(|e| panic!("Component pipeline failed on {fixture_dir}: {e}"));

        let json_str = std::fs::read_to_string(&output_path)
            .unwrap_or_else(|e| panic!("Failed to read output: {e}"));
        let actual: Value =
            serde_json::from_str(&json_str).unwrap_or_else(|e| panic!("Invalid JSON output: {e}"));
        let normalized = normalize_for_comparison(&actual);

        (normalized, actual)
    }

    /// Load expected component definition JSON and measure accuracy.
    fn assert_accuracy(fixture_dir: &str, actual: &Value) {
        let expected_path = Path::new("tests/fixtures/golden")
            .join(fixture_dir)
            .join("expected-component-definition.json");
        let expected_str = load_fixture(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to load expected component def: {e}"));
        let expected: Value = serde_json::from_str(&expected_str)
            .unwrap_or_else(|e| panic!("Invalid expected JSON: {e}"));

        let report = measure_accuracy(fixture_dir, "component", &expected, actual);
        eprintln!(
            "[{fixture_dir}/component] accuracy: {:.1}% ({}/{}) missed: {:?}",
            report.accuracy_pct,
            report.correct_count,
            report.expected_count,
            report.missed_requirements
        );
        assert!(
            report.accuracy_pct >= ACCURACY_THRESHOLD,
            "{fixture_dir}/component accuracy {:.1}% < {ACCURACY_THRESHOLD}% threshold. \
             Missed: {:?}",
            report.accuracy_pct,
            report.missed_requirements
        );
    }

    #[test]
    fn golden_small_component() {
        let (normalized, actual) = run_component("small");
        insta::assert_json_snapshot!("small_component", normalized);
        assert_accuracy("small", &actual);
    }

    #[test]
    fn golden_medium_component() {
        let (normalized, actual) = run_component("medium");
        insta::assert_json_snapshot!("medium_component", normalized);
        assert_accuracy("medium", &actual);
    }

    #[test]
    fn golden_complex_component() {
        let (normalized, actual) = run_component("complex");
        insta::assert_json_snapshot!("complex_component", normalized);
        assert_accuracy("complex", &actual);
    }
}

// ---------------------------------------------------------------------------
// Schema validation tests (T026)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod schema_validation_tests {
    use super::*;

    #[test]
    fn all_expected_catalog_files_pass_schema_validation() {
        for fixture_dir in &["small", "medium", "complex"] {
            let path =
                Path::new("tests/fixtures/golden").join(fixture_dir).join("expected-catalog.json");
            let content = load_fixture(&path)
                .unwrap_or_else(|e| panic!("Failed to load {fixture_dir} expected catalog: {e}"));
            let json: Value = serde_json::from_str(&content).unwrap_or_else(|e| {
                panic!("{fixture_dir} expected catalog is not valid JSON: {e}")
            });

            let result =
                forge::validate::validate_artifact(&json, forge::validate::OscalModelType::Catalog);
            assert!(
                result.is_ok(),
                "{fixture_dir}/expected-catalog.json schema validation error: {:?}",
                result.unwrap_err()
            );
            let validation = result.unwrap();
            assert!(
                validation.is_valid,
                "{fixture_dir}/expected-catalog.json failed OSCAL schema: {:?}",
                validation.errors
            );
        }
    }

    #[test]
    fn all_expected_component_files_pass_schema_validation() {
        for fixture_dir in &["small", "medium", "complex"] {
            let path = Path::new("tests/fixtures/golden")
                .join(fixture_dir)
                .join("expected-component-definition.json");
            let content = load_fixture(&path).unwrap_or_else(|e| {
                panic!("Failed to load {fixture_dir} expected component def: {e}")
            });
            let json: Value = serde_json::from_str(&content).unwrap_or_else(|e| {
                panic!("{fixture_dir} expected component def is not valid JSON: {e}")
            });

            let result = forge::validate::validate_artifact(
                &json,
                forge::validate::OscalModelType::ComponentDefinition,
            );
            assert!(
                result.is_ok(),
                "{fixture_dir}/expected-component-definition.json schema validation error: {:?}",
                result.unwrap_err()
            );
            let validation = result.unwrap();
            assert!(
                validation.is_valid,
                "{fixture_dir}/expected-component-definition.json failed OSCAL schema: {:?}",
                validation.errors
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Determinism verification (T027 — S-2)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn catalog_pipeline_produces_identical_output_across_runs() {
        let input_path = Path::new("tests/fixtures/golden/small/input.md");

        let dir1 = TempDir::new().unwrap();
        let out1 = dir1.path().join("run1.json");
        forge::pipeline::run_catalog_pipeline(
            input_path,
            Some(&out1),
            MAX_INPUT_SIZE,
            &OutputFormat::Json,
            None,
        )
        .expect("First run failed");

        let dir2 = TempDir::new().unwrap();
        let out2 = dir2.path().join("run2.json");
        forge::pipeline::run_catalog_pipeline(
            input_path,
            Some(&out2),
            MAX_INPUT_SIZE,
            &OutputFormat::Json,
            None,
        )
        .expect("Second run failed");

        let json1: Value = serde_json::from_str(&std::fs::read_to_string(&out1).unwrap()).unwrap();
        let json2: Value = serde_json::from_str(&std::fs::read_to_string(&out2).unwrap()).unwrap();

        let norm1 = normalize_for_comparison(&json1);
        let norm2 = normalize_for_comparison(&json2);

        assert_eq!(
            norm1, norm2,
            "Two consecutive pipeline runs should produce identical normalized output"
        );
    }
}

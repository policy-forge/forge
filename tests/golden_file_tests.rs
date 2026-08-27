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
use forge::types::Strategy;
use forge::validate::OscalModelType;
use regex::Regex;
use serde_json::{Map, Value, json};
use uuid::Uuid;

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

fn normalizes_timestamp(s: &str, parent_key: Option<&str>) -> bool {
    parent_key.is_some_and(|key| key.ends_with("-modified") || key.ends_with("-created"))
        || chrono::DateTime::parse_from_rfc3339(s).is_ok()
}

/// Normalize only standalone dynamic UUID fields.
///
/// Current serializers emit run-dynamic UUIDs as whole-string values; UUIDs embedded in href
/// fragments are deterministic v5 identifiers and must remain visible to golden comparisons.
fn is_normalizable_uuid(s: &str) -> bool {
    UUID_RE.is_match(s) && Uuid::parse_str(s).is_ok_and(|uuid| !uuid.is_nil())
}

fn is_machine_specific_href(path: &str) -> bool {
    let bytes = path.as_bytes();
    Path::new(path).is_absolute()
        || path.starts_with("file://")
        || path.starts_with("./")
        || path.starts_with("../")
        || matches!(bytes, [drive, b':', ..] if drive.is_ascii_alphabetic())
}

fn normalize_string_value(s: &str, parent_key: Option<&str>) -> Value {
    if normalizes_timestamp(s, parent_key) {
        return Value::String(NORMALIZED_TIMESTAMP.to_string());
    }
    if is_normalizable_uuid(s) {
        return Value::String(NORMALIZED_UUID.to_string());
    }
    if parent_key == Some("href") {
        let (path_part, fragment) = match s.find('#') {
            Some(idx) => (&s[..idx], &s[idx..]),
            None => (s, ""),
        };
        if is_machine_specific_href(path_part) {
            return Value::String(format!("{NORMALIZED_PATH}{fragment}"));
        }
    }
    Value::String(s.to_string())
}

/// Recursively normalize a JSON value, with optional parent key context.
fn normalize_value(value: &Value, parent_key: Option<&str>) -> Value {
    match value {
        Value::String(s) => normalize_string_value(s, parent_key),
        Value::Object(map) => {
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
            let normalized: Vec<Value> =
                arr.iter().map(|value| normalize_value(value, parent_key)).collect();
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
    extra_requirements: Vec<String>,
    duplicate_count: usize,
}

fn collect_catalog_control_ids(container: &Value, ids: &mut Vec<String>) {
    for control in container.get("controls").and_then(Value::as_array).into_iter().flatten() {
        if let Some(id) = control.get("id").and_then(Value::as_str) {
            ids.push(id.to_string());
        }
        collect_catalog_control_ids(control, ids);
    }

    for group in container.get("groups").and_then(Value::as_array).into_iter().flatten() {
        collect_catalog_control_ids(group, ids);
    }
}

fn extract_catalog_control_ids(json: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(catalog) = json.get("catalog") {
        collect_catalog_control_ids(catalog, &mut ids);
    }
    ids
}

fn extract_component_control_ids(json: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    let components = json.pointer("/component-definition/components").and_then(Value::as_array);
    for component in components.into_iter().flatten() {
        let impls = component.get("control-implementations").and_then(Value::as_array);
        for ci in impls.into_iter().flatten() {
            let reqs = ci.get("implemented-requirements").and_then(Value::as_array);
            for req in reqs.into_iter().flatten() {
                if let Some(id) = req.get("control-id").and_then(Value::as_str) {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids
}

/// Extract control IDs from an OSCAL JSON value based on strategy.
///
/// - Catalog: `$.catalog.groups[*].controls[*].id`
/// - Component: `$.component-definition.components[*].control-implementations[*].implemented-requirements[*].control-id`
fn extract_control_ids(json: &Value, strategy: Strategy) -> Vec<String> {
    match strategy {
        Strategy::Catalog => extract_catalog_control_ids(json),
        Strategy::Component => extract_component_control_ids(json),
    }
}

/// Measure extraction accuracy by comparing control IDs in expected vs actual.
fn measure_accuracy(
    fixture_name: &str,
    strategy: Strategy,
    expected: &Value,
    actual: &Value,
) -> AccuracyReport {
    let expected_ids = extract_control_ids(expected, strategy);
    let actual_ids = extract_control_ids(actual, strategy);

    let expected_count = expected_ids.len();
    assert!(
        expected_count > 0,
        "{fixture_name}: expected fixture contains no requirements; it may be corrupted or truncated"
    );

    let expected_set: std::collections::HashSet<&str> =
        expected_ids.iter().map(String::as_str).collect();
    let actual_set: std::collections::HashSet<&str> =
        actual_ids.iter().map(String::as_str).collect();

    let missed_requirements: Vec<String> =
        expected_ids.iter().filter(|id| !actual_set.contains(id.as_str())).cloned().collect();
    let extra_requirements =
        actual_ids.iter().filter(|id| !expected_set.contains(id.as_str())).cloned().collect();
    let duplicate_count = actual_ids.len().saturating_sub(actual_set.len());
    let correct_count = expected_count - missed_requirements.len();

    #[allow(clippy::cast_precision_loss)]
    let accuracy_pct = (correct_count as f64 / expected_count as f64) * 100.0;

    AccuracyReport {
        expected_count,
        correct_count,
        accuracy_pct,
        missed_requirements,
        extra_requirements,
        duplicate_count,
    }
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
    fn preserves_degenerate_uuid_and_normalizes_array_timestamps() {
        let input = json!({
            "uuid": "00000000-0000-0000-0000-000000000000",
            "events": [{"last-modified": "2026-02-14T10:30:00Z"}],
        });
        let result = normalize_for_comparison(&input);
        assert_eq!(result["uuid"], "00000000-0000-0000-0000-000000000000");
        assert_eq!(result["events"][0]["last-modified"], NORMALIZED_TIMESTAMP);
    }

    #[test]
    fn normalizes_windows_and_file_href_paths() {
        let input = json!({
            "hrefs": [
                {"href": "C:\\\\build\\catalog.json#control"},
                {"href": "file:///C:/build/catalog.json#control"},
            ],
        });
        let result = normalize_for_comparison(&input);
        assert_eq!(result["hrefs"][0]["href"], "NORMALIZED_PATH#control");
        assert_eq!(result["hrefs"][1]["href"], "NORMALIZED_PATH#control");
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
        let report = measure_accuracy("test", Strategy::Catalog, &expected, &actual);
        assert_eq!(report.expected_count, 3);
        assert_eq!(report.correct_count, 3);
        assert!((report.accuracy_pct - 100.0).abs() < f64::EPSILON);
        assert!(report.missed_requirements.is_empty());
    }

    #[test]
    fn catalog_partial_accuracy() {
        let expected = make_catalog(&["ac-1", "ac-2", "ac-3", "ac-4"]);
        let actual = make_catalog(&["ac-1", "ac-3"]);
        let report = measure_accuracy("test", Strategy::Catalog, &expected, &actual);
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
        let report = measure_accuracy("test", Strategy::Catalog, &expected, &actual);
        assert_eq!(report.correct_count, 0);
        assert!((report.accuracy_pct).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "expected fixture contains no requirements")]
    fn empty_expected_fails_loudly() {
        let expected = make_catalog(&[]);
        let actual = make_catalog(&["ac-1"]);
        let _ = measure_accuracy("test", Strategy::Catalog, &expected, &actual);
    }

    #[test]
    fn recursive_and_root_catalog_controls_are_measured() {
        let catalog = json!({
            "catalog": {
                "controls": [{"id": "root-1"}],
                "groups": [{
                    "controls": [{"id": "group-1"}],
                    "groups": [{"controls": [{"id": "nested-1"}]}]
                }]
            }
        });
        assert_eq!(
            extract_catalog_control_ids(&catalog),
            vec!["root-1".to_string(), "group-1".to_string(), "nested-1".to_string()]
        );
    }

    #[test]
    fn extras_and_duplicates_are_reported() {
        let expected = make_catalog(&["ac-1", "ac-2"]);
        let actual = make_catalog(&["ac-1", "ac-1", "dp-1"]);
        let report = measure_accuracy("test", Strategy::Catalog, &expected, &actual);
        assert_eq!(report.extra_requirements, vec!["dp-1".to_string()]);
        assert_eq!(report.duplicate_count, 1);
    }

    #[test]
    fn component_accuracy() {
        let expected = make_component_def(&["ac-1", "ac-2", "ac-3"]);
        let actual = make_component_def(&["ac-1", "ac-2"]);
        let report = measure_accuracy("test", Strategy::Component, &expected, &actual);
        assert_eq!(report.expected_count, 3);
        assert_eq!(report.correct_count, 2);
        assert!(report.missed_requirements.contains(&"ac-3".to_string()));
    }

    #[test]
    fn missed_requirements_reported_by_id() {
        let expected = make_catalog(&["ac-1", "ac-2", "dp-1"]);
        let actual = make_catalog(&["ac-1"]);
        let report = measure_accuracy("test", Strategy::Catalog, &expected, &actual);
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
        let report = measure_accuracy("boundary", Strategy::Catalog, &expected, &actual);
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
        let report = measure_accuracy("boundary", Strategy::Catalog, &expected, &actual);
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

        let result = forge::pipeline::run_catalog_pipeline(
            &input_path,
            MAX_INPUT_SIZE,
            &OutputFormat::Json,
            None,
        )
        .unwrap_or_else(|e| panic!("Catalog pipeline failed on {fixture_dir}: {e}"));

        let actual: Value = serde_json::from_str(&result.content)
            .unwrap_or_else(|e| panic!("Invalid JSON output: {e}"));
        let validation = forge::validate::validate_artifact(&actual, OscalModelType::Catalog)
            .unwrap_or_else(|e| panic!("{fixture_dir}/catalog schema validation error: {e}"));
        assert!(
            validation.is_valid,
            "{fixture_dir}/catalog failed OSCAL schema: {:?}",
            validation.errors
        );
        let normalized = normalize_for_comparison(&actual);

        (normalized, actual)
    }

    /// Load expected catalog JSON and measure accuracy.
    fn assert_accuracy(fixture_dir: &str, actual: &Value) {
        let expected_path =
            Path::new("tests/fixtures/golden").join(fixture_dir).join("expected-catalog.json");
        if std::env::var_os("UPDATE_GOLDEN_FILES").is_some() {
            let content = serde_json::to_string_pretty(actual)
                .unwrap_or_else(|e| panic!("Failed to serialize catalog golden: {e}"));
            std::fs::write(&expected_path, format!("{content}\n"))
                .unwrap_or_else(|e| panic!("Failed to update {}: {e}", expected_path.display()));
            return;
        }
        let expected_str = load_fixture(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to load expected catalog: {e}"));
        let expected: Value = serde_json::from_str(&expected_str)
            .unwrap_or_else(|e| panic!("Invalid expected JSON: {e}"));

        let report = measure_accuracy(fixture_dir, Strategy::Catalog, &expected, actual);
        eprintln!(
            "[{fixture_dir}/catalog] accuracy: {:.1}% ({}/{}) missed: {:?} extras: {:?} duplicates: {}",
            report.accuracy_pct,
            report.correct_count,
            report.expected_count,
            report.missed_requirements,
            report.extra_requirements,
            report.duplicate_count
        );
        assert!(
            report.extra_requirements.is_empty(),
            "{fixture_dir}/catalog emitted unexpected requirements: {:?}",
            report.extra_requirements
        );
        assert_eq!(
            report.duplicate_count, 0,
            "{fixture_dir}/catalog emitted duplicate requirements"
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

        let result = forge::pipeline::run_component_pipeline(
            &input_path,
            MAX_INPUT_SIZE,
            Some(SOURCE_PROFILE),
            &OutputFormat::Json,
            None,
        )
        .unwrap_or_else(|e| panic!("Component pipeline failed on {fixture_dir}: {e}"));

        let actual: Value = serde_json::from_str(&result.content)
            .unwrap_or_else(|e| panic!("Invalid JSON output: {e}"));
        let validation =
            forge::validate::validate_artifact(&actual, OscalModelType::ComponentDefinition)
                .unwrap_or_else(|e| panic!("{fixture_dir}/component schema validation error: {e}"));
        assert!(
            validation.is_valid,
            "{fixture_dir}/component failed OSCAL schema: {:?}",
            validation.errors
        );
        let normalized = normalize_for_comparison(&actual);

        (normalized, actual)
    }

    /// Load expected component definition JSON and measure accuracy.
    fn assert_accuracy(fixture_dir: &str, actual: &Value) {
        let expected_path = Path::new("tests/fixtures/golden")
            .join(fixture_dir)
            .join("expected-component-definition.json");
        if std::env::var_os("UPDATE_GOLDEN_FILES").is_some() {
            let content = serde_json::to_string_pretty(actual)
                .unwrap_or_else(|e| panic!("Failed to serialize component golden: {e}"));
            std::fs::write(&expected_path, format!("{content}\n"))
                .unwrap_or_else(|e| panic!("Failed to update {}: {e}", expected_path.display()));
            return;
        }
        let expected_str = load_fixture(&expected_path)
            .unwrap_or_else(|e| panic!("Failed to load expected component def: {e}"));
        let expected: Value = serde_json::from_str(&expected_str)
            .unwrap_or_else(|e| panic!("Invalid expected JSON: {e}"));

        let report = measure_accuracy(fixture_dir, Strategy::Component, &expected, actual);
        eprintln!(
            "[{fixture_dir}/component] accuracy: {:.1}% ({}/{}) missed: {:?} extras: {:?} duplicates: {}",
            report.accuracy_pct,
            report.correct_count,
            report.expected_count,
            report.missed_requirements,
            report.extra_requirements,
            report.duplicate_count
        );
        assert!(
            report.extra_requirements.is_empty(),
            "{fixture_dir}/component emitted unexpected requirements: {:?}",
            report.extra_requirements
        );
        assert_eq!(
            report.duplicate_count, 0,
            "{fixture_dir}/component emitted duplicate requirements"
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

        let result1 = forge::pipeline::run_catalog_pipeline(
            input_path,
            MAX_INPUT_SIZE,
            &OutputFormat::Json,
            None,
        )
        .expect("First run failed");

        let result2 = forge::pipeline::run_catalog_pipeline(
            input_path,
            MAX_INPUT_SIZE,
            &OutputFormat::Json,
            None,
        )
        .expect("Second run failed");

        let json1: Value = serde_json::from_str(&result1.content).unwrap();
        let json2: Value = serde_json::from_str(&result2.content).unwrap();

        let norm1 = normalize_for_comparison(&json1);
        let norm2 = normalize_for_comparison(&json2);

        assert_eq!(
            norm1, norm2,
            "Two consecutive pipeline runs should produce identical normalized output"
        );
    }
}

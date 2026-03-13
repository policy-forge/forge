//! Integration tests for oscal-cli round-trip validation (WI-37).
//!
//! Tests skip gracefully when oscal-cli is not available on the system PATH.

use std::path::Path;
use std::time::Duration;

use forge::oscal_cli::OscalCliDetect;
use forge::oscal_cli::detector::PathDetector;
use forge::oscal_cli::invoker::ProcessInvoker;
use forge::round_trip::{
    Divergence, DivergenceClass, OscalComparisonRules, ResolutionStatus, RoundTripResult,
    compare_oscal_json, run_round_trip_chain, write_divergence_log,
};
use forge::types::OutputFormat;

const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(30);

/// Build a `ProcessInvoker` from a detector, or None if oscal-cli is unavailable.
fn invoker_if_available(detector: &dyn OscalCliDetect) -> Option<ProcessInvoker> {
    let info = detector.detect();
    if !info.available || !info.functional {
        eprintln!("SKIP: oscal-cli not available ({info:?})");
        return None;
    }
    let path = info.executable_path.unwrap();
    Some(ProcessInvoker::new(path))
}

/// Helper: detect oscal-cli using system PATH and return a `ProcessInvoker`, or None.
fn skip_if_no_oscal_cli() -> Option<ProcessInvoker> {
    invoker_if_available(&PathDetector::new())
}

/// Generate a Catalog JSON artifact from a Markdown fixture via the FORGE pipeline.
fn generate_catalog_json(fixture: &Path, output: &Path) {
    let result =
        forge::pipeline::run_catalog_pipeline(fixture, MAX_SIZE_BYTES, &OutputFormat::Json, None)
            .expect("Catalog pipeline should succeed");
    std::fs::write(output, &result.content).expect("Failed to write catalog JSON");
}

/// Generate a Component Definition JSON artifact from a Markdown fixture.
fn generate_component_json(fixture: &Path, output: &Path) {
    let result = forge::pipeline::run_component_pipeline(
        fixture,
        MAX_SIZE_BYTES,
        None,
        &OutputFormat::Json,
        None,
    )
    .expect("Component pipeline should succeed");
    std::fs::write(output, &result.content).expect("Failed to write component JSON");
}

/// Reclassify divergences based on investigation (T018).
///
/// Divergences classified as Acceptable with no resolution are marked as Accepted.
fn reclassify(divergences: Vec<Divergence>) -> Vec<Divergence> {
    divergences
        .into_iter()
        .map(|d| {
            if d.classification == DivergenceClass::Acceptable && d.resolution.is_none() {
                Divergence { resolution: Some(ResolutionStatus::Accepted), ..d }
            } else {
                d
            }
        })
        .collect()
}

/// Run the round-trip chain, compare, reclassify, and write divergence log.
fn run_round_trip_and_compare(
    original_json_path: &Path,
    invoker: &ProcessInvoker,
    artifact_type: &str,
    log_output_path: &Path,
) -> RoundTripResult {
    let temp_dir = tempfile::tempdir().unwrap();

    let rt_json_path = run_round_trip_chain(original_json_path, invoker, temp_dir.path(), TIMEOUT)
        .expect("Round-trip chain should succeed");

    let original: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(original_json_path).unwrap()).unwrap();
    let round_tripped: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&rt_json_path).unwrap()).unwrap();

    let rules = OscalComparisonRules::default();
    let divergences = compare_oscal_json(&original, &round_tripped, "", &rules);
    let divergences = reclassify(divergences);

    let passed = divergences.iter().all(|d| d.classification == DivergenceClass::Acceptable);

    let result = RoundTripResult {
        artifact_type: artifact_type.to_string(),
        source_path: original_json_path.to_path_buf(),
        passed,
        divergences,
    };

    // Write divergence log (SC-004)
    write_divergence_log(&result, log_output_path).expect("Divergence log write should succeed");

    result
}

/// Validate divergence log file contents (T019).
fn validate_divergence_log(log_path: &Path) {
    assert!(log_path.exists(), "Divergence log file should exist");
    let content: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(log_path).unwrap())
            .expect("Divergence log should be valid JSON");

    // Verify top-level fields
    assert!(content.get("artifact_type").is_some(), "Log should have artifact_type");
    assert!(content.get("source_path").is_some(), "Log should have source_path");
    assert!(content.get("passed").is_some(), "Log should have passed");
    assert!(content.get("divergences").is_some(), "Log should have divergences");

    // Verify each divergence has required fields
    let divs = content["divergences"].as_array().expect("divergences must be a JSON array");
    for div in divs {
        assert!(div.get("json_path").is_some(), "Divergence should have json_path");
        assert!(div.get("expected").is_some(), "Divergence should have expected");
        assert!(div.get("actual").is_some(), "Divergence should have actual");
        assert!(div.get("classification").is_some(), "Divergence should have classification");
        assert!(div.get("description").is_some(), "Divergence should have description");
        // resolution field must be present (even if null)
        assert!(
            div.get("resolution").is_some(),
            "Divergence should have resolution field (even if null)"
        );

        // For ForgeFix and OscalCliDiff, resolution must be non-null (SC-004 / T019)
        let classification = div.get("classification").and_then(|v| v.as_str()).unwrap_or("");
        if classification == "ForgeFix" || classification == "OscalCliDiff" {
            assert!(
                !div["resolution"].is_null(),
                "Divergence with classification {classification} must have a non-null resolution"
            );
        }
    }
}

// SC-001: Catalog JSON → XML → YAML → JSON round-trip
#[test]
fn catalog_json_xml_yaml_json_round_trip() {
    let Some(invoker) = skip_if_no_oscal_cli() else { return };

    let temp_dir = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/full_policy.md");
    let catalog_json = temp_dir.path().join("catalog.json");
    let log_path = temp_dir.path().join("divergences-catalog.json");

    generate_catalog_json(fixture, &catalog_json);
    let result = run_round_trip_and_compare(&catalog_json, &invoker, "Catalog", &log_path);

    // Log divergences for investigation
    log_divergence_summary(&result);

    // Validate divergence log (SC-004)
    validate_divergence_log(&log_path);

    // SC-001: Zero unresolved FORGE-caused divergences
    let forge_fix_count =
        result.divergences.iter().filter(|d| d.classification == DivergenceClass::ForgeFix).count();
    assert_eq!(
        forge_fix_count, 0,
        "SC-001: Expected zero ForgeFix divergences, found {forge_fix_count}"
    );
}

// SC-002: Component Definition JSON → XML → YAML → JSON round-trip
#[test]
fn component_json_xml_yaml_json_round_trip() {
    let Some(invoker) = skip_if_no_oscal_cli() else { return };

    let temp_dir = tempfile::tempdir().unwrap();
    let fixture = Path::new("tests/fixtures/full_policy.md");
    let component_json = temp_dir.path().join("component-definition.json");
    let log_path = temp_dir.path().join("divergences-component.json");

    generate_component_json(fixture, &component_json);
    let result =
        run_round_trip_and_compare(&component_json, &invoker, "ComponentDefinition", &log_path);

    // Log divergences for investigation
    log_divergence_summary(&result);

    // Validate divergence log (SC-004)
    validate_divergence_log(&log_path);

    // SC-002: Zero unresolved FORGE-caused divergences
    let forge_fix_count =
        result.divergences.iter().filter(|d| d.classification == DivergenceClass::ForgeFix).count();
    assert_eq!(
        forge_fix_count, 0,
        "SC-002: Expected zero ForgeFix divergences, found {forge_fix_count}"
    );
}

// SC-005: Tests skip cleanly when oscal-cli unavailable
//
// Verified two ways:
// 1. This test uses a mock detector that always returns "not found" and confirms
//    the skip helper returns None without panicking.
// 2. On machines without oscal-cli, the catalog/component tests above return early
//    without failure, demonstrating the actual skip path.
#[test]
fn round_trip_skip_when_oscal_cli_unavailable() {
    use forge::oscal_cli::OscalCliInfo;

    struct MockUnavailableDetector;
    impl OscalCliDetect for MockUnavailableDetector {
        fn detect(&self) -> OscalCliInfo {
            OscalCliInfo::not_found()
        }
    }

    // SC-005: verify the actual skip helper returns None with a mock unavailable detector
    assert!(
        invoker_if_available(&MockUnavailableDetector).is_none(),
        "Unavailable detector should cause skip helper to return None"
    );
}

/// Log INFO-level summary after test run (T022).
fn log_divergence_summary(result: &RoundTripResult) {
    let total = result.divergences.len();
    let forge_fix =
        result.divergences.iter().filter(|d| d.classification == DivergenceClass::ForgeFix).count();
    let oscal_cli_diff = result
        .divergences
        .iter()
        .filter(|d| d.classification == DivergenceClass::OscalCliDiff)
        .count();
    let acceptable = result
        .divergences
        .iter()
        .filter(|d| d.classification == DivergenceClass::Acceptable)
        .count();

    eprintln!(
        "Round-trip summary [{}]: total={}, ForgeFix={}, OscalCliDiff={}, Acceptable={}, passed={}",
        result.artifact_type, total, forge_fix, oscal_cli_diff, acceptable, result.passed
    );

    if !result.divergences.is_empty() {
        for d in &result.divergences {
            eprintln!(
                "  [{:?}] {} — expected: {}, actual: {}",
                d.classification, d.json_path, d.expected, d.actual
            );
        }
    }
}

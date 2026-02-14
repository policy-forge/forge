//! Contract definitions for the golden-file test harness.
//!
//! These types and function signatures define the interface for golden-file
//! comparison and accuracy measurement. Implementation lives in
//! `tests/golden_file_tests.rs`.

use serde_json::Value;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Fixed UUID placeholder used during normalization.
pub const NORMALIZED_UUID: &str = "00000000-0000-0000-0000-000000000000";

/// Fixed timestamp used to replace `last-modified` fields during normalization.
pub const NORMALIZED_TIMESTAMP: &str = "2026-01-01T00:00:00Z";

/// Minimum extraction accuracy percentage required to pass (PRD M-8).
/// Inclusive threshold: accuracy >= 95.0% passes (see research.md R-7).
pub const ACCURACY_THRESHOLD: f64 = 95.0;

// ---------------------------------------------------------------------------
// Normalization
// ---------------------------------------------------------------------------

/// Normalize non-deterministic fields in OSCAL JSON for stable comparison.
///
/// Recursively walks the JSON tree and:
/// 1. Replaces UUID-format string values with [`NORMALIZED_UUID`]
/// 2. Replaces `last-modified` field values with [`NORMALIZED_TIMESTAMP`]
/// 3. Object keys are inherently sorted by `serde_json::Map` (BTreeMap)
///
/// # Idempotency
/// `normalize_for_comparison(&normalize_for_comparison(v)) == normalize_for_comparison(v)`
///
/// # Arguments
/// * `json` - The OSCAL JSON value to normalize
///
/// # Returns
/// A new `Value` with non-deterministic fields replaced by fixed placeholders.
pub fn normalize_for_comparison(json: &Value) -> Value {
    todo!("Implement in tests/golden_file_tests.rs")
}

// ---------------------------------------------------------------------------
// Accuracy Measurement
// ---------------------------------------------------------------------------

/// Extraction accuracy result for a single fixture and strategy.
#[derive(Debug)]
pub struct AccuracyReport {
    /// Fixture name (e.g., "small", "medium", "complex")
    pub fixture_name: String,

    /// Strategy tested (e.g., "catalog", "component")
    pub strategy: String,

    /// Number of requirements expected (from golden file)
    pub expected_count: usize,

    /// Number of requirements correctly extracted in actual output
    pub correct_count: usize,

    /// Accuracy percentage: (correct_count / expected_count) * 100.0
    pub accuracy_pct: f64,

    /// Control IDs or requirement IDs present in expected but missing from actual
    pub missed_requirements: Vec<String>,
}

/// Measure extraction accuracy by comparing control IDs in expected vs actual.
///
/// For Catalog strategy: extracts control IDs from `$.catalog.groups[*].controls[*].id`
/// For Component strategy: extracts control-ids from
///   `$.component-definition.components[*].control-implementations[*].implemented-requirements[*].control-id`
///
/// # Arguments
/// * `fixture_name` - Name of the fixture (for reporting)
/// * `strategy` - "catalog" or "component"
/// * `expected` - Expected OSCAL JSON (golden file)
/// * `actual` - Actual OSCAL JSON (pipeline output)
///
/// # Returns
/// An [`AccuracyReport`] with counts and missed requirement IDs.
pub fn measure_accuracy(
    fixture_name: &str,
    strategy: &str,
    expected: &Value,
    actual: &Value,
) -> AccuracyReport {
    todo!("Implement in tests/golden_file_tests.rs")
}

// ---------------------------------------------------------------------------
// Test Function Signatures
// ---------------------------------------------------------------------------

// Each golden-file test follows this pattern:
//
// #[test]
// fn golden_{fixture}_{strategy}() {
//     // 1. Load input.md from tests/fixtures/golden/{fixture}/
//     // 2. Run pipeline via library API (run_catalog_pipeline or run_component_pipeline)
//     // 3. Parse actual output as serde_json::Value
//     // 4. Normalize with normalize_for_comparison()
//     // 5. Compare with insta::assert_json_snapshot!("{fixture}_{strategy}", normalized)
//     // 6. Load expected-{strategy}.json
//     // 7. Measure accuracy with measure_accuracy()
//     // 8. Assert accuracy >= ACCURACY_THRESHOLD
//     // 9. Print accuracy report
// }
//
// Expected tests (6 total = 3 fixtures × 2 strategies):
//   golden_small_catalog
//   golden_small_component
//   golden_medium_catalog
//   golden_medium_component
//   golden_complex_catalog
//   golden_complex_component

// Contract: Semantic Equivalence Module
// Location: src/testing/semantic_eq.rs
// PRD Traceability: M-5 (key-order-independent), M-6 (array order), M-7 (structural diff), M-8 (type preservation)

use serde_json::Value;

/// Result of a semantic equivalence comparison between two OSCAL documents.
#[derive(Debug, Clone)]
pub struct EquivalenceResult {
    /// Whether the two documents are semantically equivalent.
    pub is_equivalent: bool,
    /// Human-readable diff details if not equivalent; empty if equivalent.
    pub differences: Vec<EquivalenceDiff>,
}

/// A single difference found during semantic comparison.
#[derive(Debug, Clone)]
pub struct EquivalenceDiff {
    /// JSON Pointer-style path to the differing element (e.g., "/catalog/metadata/title").
    pub path: String,
    /// Description of the difference.
    pub description: String,
    /// The expected value (from the original document); None if key is extra in actual.
    pub expected: Option<String>,
    /// The actual value (from the round-tripped document); None if key is missing in actual.
    pub actual: Option<String>,
}

/// Compare two OSCAL documents for semantic equivalence.
///
/// Objects: keys compared as unordered sets; values compared recursively.
/// Arrays: elements compared in order (OSCAL array order is significant per PRD M-6).
/// Primitives: compared by value and type (PRD M-8).
///
/// Returns `EquivalenceResult` with `is_equivalent = true` if the documents match,
/// or a list of `EquivalenceDiff` entries describing each discrepancy.
pub fn assert_semantic_equivalence(
    original: &Value,
    round_tripped: &Value,
) -> EquivalenceResult;

// Internal: Recursive comparison of JSON Value nodes.
// fn compare_values(expected: &Value, actual: &Value, path: &str, diffs: &mut Vec<EquivalenceDiff>);

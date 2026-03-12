//! OSCAL-specific comparison rules for semantic JSON diffing.

use std::collections::HashSet;

/// OSCAL-specific rules for the semantic comparison algorithm.
pub struct OscalComparisonRules {
    /// JSON key names whose array values are compared without regard to element order.
    pub unordered_array_paths: HashSet<String>,
    /// JSON Pointer prefixes to skip entirely (reserved for future use; empty by default).
    pub ignored_paths: Vec<String>,
}

impl Default for OscalComparisonRules {
    fn default() -> Self {
        Self {
            unordered_array_paths: ["props", "links", "parts"]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            ignored_paths: vec![],
        }
    }
}

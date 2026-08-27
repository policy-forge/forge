//! OSCAL-specific comparison rules for semantic JSON diffing.

use std::collections::HashSet;

/// OSCAL-specific rules for the semantic comparison algorithm.
#[derive(Debug, Clone)]
pub struct OscalComparisonRules {
    /// JSON object key names whose array values are compared without regard to
    /// element order. A key matches at any nesting depth; entries are not JSON
    /// Pointer paths.
    pub unordered_array_keys: HashSet<String>,
    /// JSON Pointer prefixes to skip entirely during comparison.
    pub ignored_paths: Vec<String>,
    /// JSON Pointer suffixes identifying RFC 3339 timestamps whose equivalent
    /// serializations are acceptable.
    pub acceptable_timestamp_path_suffixes: Vec<String>,
}

impl Default for OscalComparisonRules {
    fn default() -> Self {
        Self {
            unordered_array_keys: ["props", "links", "parts"]
                .iter()
                .map(|key| (*key).to_string())
                .collect(),
            ignored_paths: Vec::new(),
            acceptable_timestamp_path_suffixes: ["/last-modified", "/published", "/updated"]
                .iter()
                .map(|suffix| (*suffix).to_string())
                .collect(),
        }
    }
}

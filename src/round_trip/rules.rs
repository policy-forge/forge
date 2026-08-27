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
            unordered_array_keys: HashSet::from([
                "props".to_string(),
                "links".to_string(),
                "parts".to_string(),
            ]),
            ignored_paths: Vec::new(),
            acceptable_timestamp_path_suffixes: vec![
                "/last-modified".to_string(),
                "/published".to_string(),
                "/updated".to_string(),
            ],
        }
    }
}

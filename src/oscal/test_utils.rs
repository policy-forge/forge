//! Shared test utilities for OSCAL module tests.
//!
//! This module is only compiled when running tests (`#[cfg(test)]`).

/// Recursively collect all values under "remarks" keys in a JSON tree.
///
/// Used by multiple test modules to verify that trace data does not leak
/// into OSCAL `remarks` fields (SEC-1, SEC-2, M-7).
pub fn collect_remarks(value: &serde_json::Value, collected: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key == "remarks"
                    && let Some(s) = val.as_str()
                {
                    collected.push(s.to_string());
                }
                collect_remarks(val, collected);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                collect_remarks(item, collected);
            }
        }
        _ => {}
    }
}

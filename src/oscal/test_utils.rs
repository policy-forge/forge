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
                if key == "remarks" {
                    let Some(remarks) = val.as_str() else {
                        panic!("remarks value must be a string, got {val}");
                    };
                    collected.push(remarks.to_string());
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

#[cfg(test)]
mod tests {
    use super::collect_remarks;

    #[test]
    fn collects_nested_string_remarks() {
        let value = serde_json::json!({
            "remarks": "root",
            "nested": [{"remarks": "child"}],
        });
        let mut collected = Vec::new();

        collect_remarks(&value, &mut collected);

        collected.sort();
        assert_eq!(collected, vec!["child", "root"]);
    }

    #[test]
    #[should_panic(expected = "remarks value must be a string")]
    fn rejects_non_string_remarks() {
        let value = serde_json::json!({"remarks": ["not a string"]});
        let mut collected = Vec::new();

        collect_remarks(&value, &mut collected);
    }
}

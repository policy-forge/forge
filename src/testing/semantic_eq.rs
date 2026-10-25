//! Semantic equivalence comparison for OSCAL documents.
//!
//! Compares two `serde_json::Value` trees for semantic equivalence:
//! - Objects: keys compared as unordered sets; values compared recursively
//! - Arrays: elements compared in order (OSCAL array order is significant per PRD M-6)
//! - Primitives: compared by value and type (PRD M-8)

use serde_json::Value;

/// Match `serde_json`'s default nesting limit for programmatically built test values.
const MAX_COMPARISON_DEPTH: usize = 128;

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
/// Primitives: compared by JSON semantic value. Numerically equal JSON numbers
/// are equivalent even when their source representations differ.
///
/// Returns `EquivalenceResult` with `is_equivalent = true` if the documents match,
/// or a list of `EquivalenceDiff` entries describing each discrepancy.
#[must_use]
pub fn assert_semantic_equivalence(original: &Value, round_tripped: &Value) -> EquivalenceResult {
    let mut differences = Vec::new();
    compare_values(original, round_tripped, "", &mut differences, 0);
    EquivalenceResult { is_equivalent: differences.is_empty(), differences }
}

/// Escape a JSON object key for use in a JSON Pointer token per RFC 6901.
/// `~` is replaced with `~0` and `/` is replaced with `~1`.
fn escape_json_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

/// Recursively compare JSON value nodes and accumulate every difference.
fn compare_values(
    expected: &Value,
    actual: &Value,
    path: &str,
    diffs: &mut Vec<EquivalenceDiff>,
    depth: usize,
) {
    if depth >= MAX_COMPARISON_DEPTH {
        diffs.push(EquivalenceDiff {
            path: path.to_string(),
            description: format!("nesting deeper than {MAX_COMPARISON_DEPTH} levels"),
            expected: None,
            actual: None,
        });
        return;
    }

    match (expected, actual) {
        (Value::Object(exp_map), Value::Object(act_map)) => {
            compare_objects(exp_map, act_map, path, diffs, depth);
        }
        (Value::Array(exp_arr), Value::Array(act_arr)) => {
            compare_arrays(exp_arr, act_arr, path, diffs, depth);
        }
        _ => {
            compare_primitives(expected, actual, path, diffs);
        }
    }
}

fn compare_objects(
    exp_map: &serde_json::Map<String, Value>,
    act_map: &serde_json::Map<String, Value>,
    path: &str,
    diffs: &mut Vec<EquivalenceDiff>,
    depth: usize,
) {
    for key in exp_map.keys() {
        let child_path = format!("{path}/{}", escape_json_pointer_token(key));
        if let Some(act_val) = act_map.get(key) {
            compare_values(&exp_map[key], act_val, &child_path, diffs, depth + 1);
        } else {
            diffs.push(EquivalenceDiff {
                path: child_path,
                description: format!("missing key \"{key}\""),
                expected: Some(format_value(&exp_map[key])),
                actual: None,
            });
        }
    }
    for key in act_map.keys() {
        if !exp_map.contains_key(key) {
            let child_path = format!("{path}/{}", escape_json_pointer_token(key));
            diffs.push(EquivalenceDiff {
                path: child_path,
                description: format!("extra key \"{key}\""),
                expected: None,
                actual: Some(format_value(&act_map[key])),
            });
        }
    }
}

fn compare_arrays(
    exp_arr: &[Value],
    act_arr: &[Value],
    path: &str,
    diffs: &mut Vec<EquivalenceDiff>,
    depth: usize,
) {
    if exp_arr.len() != act_arr.len() {
        diffs.push(EquivalenceDiff {
            path: path.to_string(),
            description: format!(
                "array length mismatch: expected {}, got {}",
                exp_arr.len(),
                act_arr.len()
            ),
            expected: Some(exp_arr.len().to_string()),
            actual: Some(act_arr.len().to_string()),
        });
    }
    let min_len = exp_arr.len().min(act_arr.len());
    for index in 0..min_len {
        let child_path = format!("{path}/{index}");
        compare_values(&exp_arr[index], &act_arr[index], &child_path, diffs, depth + 1);
    }
    for (index, value) in exp_arr.iter().enumerate().skip(min_len) {
        diffs.push(EquivalenceDiff {
            path: format!("{path}/{index}"),
            description: "missing element".to_string(),
            expected: Some(format_value(value)),
            actual: None,
        });
    }
    for (index, value) in act_arr.iter().enumerate().skip(min_len) {
        diffs.push(EquivalenceDiff {
            path: format!("{path}/{index}"),
            description: "extra element".to_string(),
            expected: None,
            actual: Some(format_value(value)),
        });
    }
}

/// Exact numeric equality across integer and float representations.
///
/// Integers compare as integers, so distinct large values never collapse through
/// `f64` rounding. An integer equals a float only when the float is integral and
/// converts to exactly that integer (e.g. `1` == `1.0`, `2^53` == `2^53.0`).
fn numbers_exactly_equal(left: &serde_json::Number, right: &serde_json::Number) -> bool {
    fn integer(number: &serde_json::Number) -> Option<i128> {
        number.as_i64().map(i128::from).or_else(|| number.as_u64().map(i128::from))
    }
    #[allow(clippy::cast_possible_truncation)] // Integral and bounded: the cast is exact.
    fn integral_float(value: f64) -> Option<i128> {
        let integral = value.is_finite() && value.trunc().total_cmp(&value).is_eq();
        (integral && value.abs() < 2f64.powi(127)).then_some(value as i128)
    }
    match (integer(left), integer(right)) {
        (Some(left), Some(right)) => left == right,
        (Some(int), None) => right.as_f64().and_then(integral_float) == Some(int),
        (None, Some(int)) => left.as_f64().and_then(integral_float) == Some(int),
        (None, None) => left.as_f64().zip(right.as_f64()).is_some_and(|(left, right)| {
            left.partial_cmp(&right).is_some_and(std::cmp::Ordering::is_eq)
        }),
    }
}

fn compare_primitives(
    expected: &Value,
    actual: &Value,
    path: &str,
    diffs: &mut Vec<EquivalenceDiff>,
) {
    if expected == actual {
        return;
    }
    if let (Value::Number(expected_number), Value::Number(actual_number)) = (expected, actual)
        && numbers_exactly_equal(expected_number, actual_number)
    {
        return;
    }

    let description = if discriminant_name(expected) == discriminant_name(actual) {
        "value mismatch".to_string()
    } else {
        format!(
            "type mismatch: expected {}, got {}",
            discriminant_name(expected),
            discriminant_name(actual)
        )
    };
    diffs.push(EquivalenceDiff {
        path: path.to_string(),
        description,
        expected: Some(format_value(expected)),
        actual: Some(format_value(actual)),
    });
}

/// Return a human-readable type name for a `serde_json::Value` variant.
fn discriminant_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Format a `Value` as a compact display string for diff reporting.
fn format_value(value: &Value) -> String {
    let rendered = match value {
        Value::Null => "null".to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => format!("\"{text}\""),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "<unrepresentable>".to_string())
        }
    };
    let char_count = rendered.chars().count();
    if char_count > 500 {
        let truncated: String = rendered.chars().take(500).collect();
        format!("{truncated}… ({char_count} chars total)")
    } else {
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // T008: Equal objects (same keys, same values) -> equivalent
    #[test]
    fn equal_objects_are_equivalent() {
        let original = json!({
            "catalog": {
                "metadata": { "title": "Test Policy" },
                "version": "1.0"
            }
        });
        let round_tripped = json!({
            "catalog": {
                "metadata": { "title": "Test Policy" },
                "version": "1.0"
            }
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(result.is_equivalent);
        assert!(result.differences.is_empty());
    }

    // T009: Different key ordering -> equivalent
    #[test]
    fn different_key_ordering_is_equivalent() {
        let original = json!({
            "version": "1.0",
            "title": "Policy",
            "id": "abc-123"
        });
        let round_tripped = json!({
            "id": "abc-123",
            "title": "Policy",
            "version": "1.0"
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(result.is_equivalent);
        assert!(result.differences.is_empty());
    }

    // T010: Missing key -> not equivalent, diff shows path
    #[test]
    fn missing_key_is_not_equivalent() {
        let original = json!({
            "catalog": {
                "metadata": { "title": "Test", "version": "1.0" }
            }
        });
        let round_tripped = json!({
            "catalog": {
                "metadata": { "title": "Test" }
            }
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "/catalog/metadata/version");
        assert!(result.differences[0].description.contains("missing"));
        assert!(result.differences[0].expected.is_some());
        assert!(result.differences[0].actual.is_none());
    }

    // T011: Extra key -> not equivalent, diff shows path
    #[test]
    fn extra_key_is_not_equivalent() {
        let original = json!({
            "catalog": { "title": "Test" }
        });
        let round_tripped = json!({
            "catalog": { "title": "Test", "extra_field": "unexpected" }
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "/catalog/extra_field");
        assert!(result.differences[0].description.contains("extra"));
        assert!(result.differences[0].expected.is_none());
        assert!(result.differences[0].actual.is_some());
    }

    // T012: Value mismatch -> not equivalent, diff shows path and values
    #[test]
    fn value_mismatch_is_not_equivalent() {
        let original = json!({
            "catalog": { "metadata": { "title": "Original Title" } }
        });
        let round_tripped = json!({
            "catalog": { "metadata": { "title": "Changed Title" } }
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "/catalog/metadata/title");
        assert!(result.differences[0].description.contains("mismatch"));
        assert_eq!(result.differences[0].expected.as_deref(), Some("\"Original Title\""));
        assert_eq!(result.differences[0].actual.as_deref(), Some("\"Changed Title\""));
    }

    // T013: Arrays with same elements in same order -> equivalent
    #[test]
    fn arrays_same_order_are_equivalent() {
        let original = json!({
            "controls": [
                { "id": "ac-1", "title": "Access Control" },
                { "id": "ac-2", "title": "Account Management" }
            ]
        });
        let round_tripped = json!({
            "controls": [
                { "id": "ac-1", "title": "Access Control" },
                { "id": "ac-2", "title": "Account Management" }
            ]
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(result.is_equivalent);
        assert!(result.differences.is_empty());
    }

    // T014: Arrays with same elements in different order -> not equivalent (PRD M-6)
    #[test]
    fn arrays_different_order_are_not_equivalent() {
        let original = json!({
            "controls": [
                { "id": "ac-1", "title": "Access Control" },
                { "id": "ac-2", "title": "Account Management" }
            ]
        });
        let round_tripped = json!({
            "controls": [
                { "id": "ac-2", "title": "Account Management" },
                { "id": "ac-1", "title": "Access Control" }
            ]
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert!(!result.differences.is_empty());
    }

    // T015: Type mismatch (string "1" vs number 1) -> not equivalent (PRD M-8)
    #[test]
    fn type_mismatch_is_not_equivalent() {
        let original = json!({ "value": "1" });
        let round_tripped = json!({ "value": 1 });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "/value");
        assert!(result.differences[0].description.contains("type mismatch"));
        assert_eq!(result.differences[0].expected.as_deref(), Some("\"1\""));
        assert_eq!(result.differences[0].actual.as_deref(), Some("1"));
    }

    // T016: Deeply nested objects (5+ levels) -> correct JSON Pointer path on mismatch
    #[test]
    fn deeply_nested_path_is_correct() {
        let original = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "deep_value": "expected"
                            }
                        }
                    }
                }
            }
        });
        let round_tripped = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "deep_value": "actual"
                            }
                        }
                    }
                }
            }
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "/level1/level2/level3/level4/level5/deep_value");
    }

    // T017: Empty objects `{}` and empty arrays `[]` -> equivalent
    #[test]
    fn empty_objects_and_arrays_are_equivalent() {
        let original = json!({
            "empty_obj": {},
            "empty_arr": []
        });
        let round_tripped = json!({
            "empty_obj": {},
            "empty_arr": []
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(result.is_equivalent);
        assert!(result.differences.is_empty());
    }

    // T018: Array length mismatch -> not equivalent with length difference
    #[test]
    fn array_length_mismatch_is_not_equivalent() {
        let original = json!({
            "items": ["a", "b", "c"]
        });
        let round_tripped = json!({
            "items": ["a", "b"]
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        // Should have at least the length mismatch diff
        let length_diff =
            result.differences.iter().find(|d| d.description.contains("array length mismatch"));
        assert!(length_diff.is_some(), "expected array length mismatch diff");
        let diff = length_diff.unwrap();
        assert_eq!(diff.path, "/items");
        assert_eq!(diff.expected.as_deref(), Some("3"));
        assert_eq!(diff.actual.as_deref(), Some("2"));
    }

    #[test]
    fn array_length_mismatch_reports_each_surplus_element() {
        let original = json!({ "items": ["first", "surplus"] });
        let round_tripped = json!({ "items": ["first"] });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        let surplus = result
            .differences
            .iter()
            .find(|difference| difference.path == "/items/1")
            .expect("surplus element must be reported at its index");
        assert_eq!(surplus.description, "missing element");
        assert_eq!(surplus.expected.as_deref(), Some("\"surplus\""));
        assert!(surplus.actual.is_none());
    }

    #[test]
    fn distinct_large_integers_are_not_equivalent() {
        for (left, right) in [
            (json!(9_007_199_254_740_993_u64), json!(9_007_199_254_740_992_u64)),
            (json!(u64::MAX), json!(u64::MAX - 1)),
            (json!(-9_007_199_254_740_993_i64), json!(-9_007_199_254_740_992_i64)),
            (json!(9_007_199_254_740_993_u64), json!(9_007_199_254_740_992.0)),
            (json!(1), json!(1.5)),
        ] {
            assert!(!assert_semantic_equivalence(&left, &right).is_equivalent);
        }
    }

    #[test]
    fn exact_integer_float_pairs_at_and_beyond_2_pow_53_are_equivalent() {
        for (left, right) in [
            (json!(9_007_199_254_740_992_u64), json!(9_007_199_254_740_992.0)),
            (json!(-9_007_199_254_740_992_i64), json!(-9_007_199_254_740_992.0)),
            (json!(18_014_398_509_481_984_u64), json!(18_014_398_509_481_984.0)),
            (json!(0), json!(-0.0)),
        ] {
            assert!(assert_semantic_equivalence(&left, &right).is_equivalent, "{left} {right}");
            assert!(assert_semantic_equivalence(&right, &left).is_equivalent, "{right} {left}");
        }
    }

    #[test]
    fn numerically_equal_number_representations_are_equivalent() {
        let original = json!({ "value": 1 });
        let round_tripped = json!({ "value": 1.0 });

        assert!(assert_semantic_equivalence(&original, &round_tripped).is_equivalent);
    }

    #[test]
    fn deeply_nested_programmatic_values_stop_at_comparison_limit() {
        let mut original = Value::Null;
        let mut round_tripped = Value::Null;
        for _ in 0..150 {
            original = json!({ "nested": original });
            round_tripped = json!({ "nested": round_tripped });
        }

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].description, "nesting deeper than 128 levels");
    }

    // Additional edge-case tests for completeness

    #[test]
    fn reports_all_differences_not_just_first() {
        let original = json!({
            "a": "1",
            "b": "2",
            "c": "3"
        });
        let round_tripped = json!({
            "a": "X",
            "b": "Y",
            "c": "Z"
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 3);
    }

    #[test]
    fn null_values_are_equivalent() {
        let original = json!({ "field": null });
        let round_tripped = json!({ "field": null });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(result.is_equivalent);
    }

    #[test]
    fn boolean_values_compared_correctly() {
        let original = json!({ "flag": true });
        let round_tripped = json!({ "flag": false });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].expected.as_deref(), Some("true"));
        assert_eq!(result.differences[0].actual.as_deref(), Some("false"));
    }

    #[test]
    fn object_vs_array_type_mismatch() {
        let original = json!({ "data": {} });
        let round_tripped = json!({ "data": [] });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert!(result.differences[0].description.contains("type mismatch"));
    }

    #[test]
    fn array_element_differences_reported_with_index_path() {
        let original = json!({ "items": ["a", "b", "c"] });
        let round_tripped = json!({ "items": ["a", "X", "c"] });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 1);
        assert_eq!(result.differences[0].path, "/items/1");
    }

    #[test]
    fn keys_with_tilde_and_slash_are_escaped_per_rfc_6901() {
        let original = json!({
            "a/b": "value1",
            "c~d": "value2"
        });
        let round_tripped = json!({
            "a/b": "changed1",
            "c~d": "changed2"
        });

        let result = assert_semantic_equivalence(&original, &round_tripped);

        assert!(!result.is_equivalent);
        assert_eq!(result.differences.len(), 2);
        let paths: Vec<&str> = result.differences.iter().map(|d| d.path.as_str()).collect();
        assert!(paths.contains(&"/a~1b"), "expected /a~1b for key 'a/b', got {paths:?}");
        assert!(paths.contains(&"/c~0d"), "expected /c~0d for key 'c~d', got {paths:?}");
    }

    #[test]
    fn escape_json_pointer_token_correctness() {
        assert_eq!(escape_json_pointer_token("simple"), "simple");
        assert_eq!(escape_json_pointer_token("a/b"), "a~1b");
        assert_eq!(escape_json_pointer_token("c~d"), "c~0d");
        assert_eq!(escape_json_pointer_token("a~b/c"), "a~0b~1c");
        assert_eq!(escape_json_pointer_token(""), "");
    }

    #[test]
    fn format_value_truncates_large_json_values() {
        let value = json!({ "description": "a".repeat(600) });
        let formatted = format_value(&value);

        assert!(formatted.contains('…'));
        assert!(formatted.contains("chars total"));
    }

    #[test]
    fn format_value_truncates_multibyte_json_values_by_character_count() {
        let value = json!({ "description": format!("{}{}", "a".repeat(495), "認証".repeat(20)) });
        let formatted = format_value(&value);

        assert!(formatted.contains("… ("));
        assert!(formatted.contains("chars total)"));
    }

    #[test]
    fn format_value_truncates_large_string_values() {
        let formatted = format_value(&Value::String("a".repeat(600)));

        assert!(formatted.contains("… (602 chars total)"));
        assert!(formatted.chars().count() < 550);
    }
}

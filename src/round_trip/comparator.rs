//! OSCAL-aware recursive JSON comparison algorithm.

use serde_json::Value;

use super::divergence::{Divergence, DivergenceClass};
use super::rules::OscalComparisonRules;

/// Compare two OSCAL JSON trees semantically.
///
/// Applies OSCAL-aware comparison rules:
/// - JSON objects: keys compared as unordered sets; values compared recursively
/// - Arrays at `props`, `links`, `parts` paths: elements matched by identity key
/// - All other arrays: elements compared positionally
/// - Primitives: compared by type and value
///
/// Returns a `Vec<Divergence>` — empty if the documents are semantically equivalent.
#[must_use]
pub fn compare_oscal_json(
    expected: &Value,
    actual: &Value,
    path: &str,
    rules: &OscalComparisonRules,
) -> Vec<Divergence> {
    let mut divergences = Vec::new();
    compare_values(expected, actual, path, rules, &mut divergences);
    divergences
}

fn compare_values(
    expected: &Value,
    actual: &Value,
    path: &str,
    rules: &OscalComparisonRules,
    divergences: &mut Vec<Divergence>,
) {
    match (expected, actual) {
        (Value::Object(exp_map), Value::Object(act_map)) => {
            for key in exp_map.keys() {
                let child_path = format!("{path}/{key}");
                match act_map.get(key) {
                    Some(act_val) => {
                        compare_values(&exp_map[key], act_val, &child_path, rules, divergences);
                    }
                    None => {
                        divergences.push(missing_key_divergence(
                            child_path,
                            exp_map[key].clone(),
                            Value::Null,
                            &exp_map[key],
                            "Empty array in expected vs absent key in actual",
                            "Key present in expected but absent in actual",
                        ));
                    }
                }
            }
            for key in act_map.keys() {
                if !exp_map.contains_key(key) {
                    let child_path = format!("{path}/{key}");
                    divergences.push(missing_key_divergence(
                        child_path,
                        Value::Null,
                        act_map[key].clone(),
                        &act_map[key],
                        "Absent key in expected vs empty array in actual",
                        "Extra key in actual not present in expected",
                    ));
                }
            }
        }
        (Value::Array(exp_arr), Value::Array(act_arr)) => {
            let key_name = path.rsplit('/').next().unwrap_or("");
            if rules.unordered_array_paths.contains(key_name) {
                compare_unordered_array(exp_arr, act_arr, path, rules, divergences);
            } else {
                compare_ordered_array(exp_arr, act_arr, path, rules, divergences);
            }
        }
        _ => {
            if expected != actual {
                let acceptable = acceptable_scalar_normalization(expected, actual, path);
                divergences.push(Divergence {
                    json_path: path.to_string(),
                    expected: expected.clone(),
                    actual: actual.clone(),
                    classification: if acceptable.is_some() {
                        DivergenceClass::Acceptable
                    } else {
                        DivergenceClass::ForgeFix
                    },
                    description: acceptable.map_or_else(
                        || {
                            format!(
                                "Value mismatch: expected {}, actual {}",
                                summarize_value(expected),
                                summarize_value(actual)
                            )
                        },
                        str::to_string,
                    ),
                    resolution: None,
                });
            }
        }
    }
}

fn acceptable_scalar_normalization(
    expected: &Value,
    actual: &Value,
    path: &str,
) -> Option<&'static str> {
    let (Some(expected), Some(actual)) = (expected.as_str(), actual.as_str()) else {
        return None;
    };

    if path.ends_with("/last-modified")
        && let (Ok(expected), Ok(actual)) = (
            chrono::DateTime::parse_from_rfc3339(expected),
            chrono::DateTime::parse_from_rfc3339(actual),
        )
        && expected == actual
    {
        return Some("Equivalent RFC 3339 timestamp representation");
    }

    if path.ends_with("/prose") && soft_line_breaks_equivalent(expected, actual) {
        return Some("Markup prose differs only by whitespace normalization");
    }

    None
}

fn soft_line_breaks_equivalent(expected: &str, actual: &str) -> bool {
    normalize_soft_line_breaks(expected).is_some_and(|normalized| normalized == actual)
        || normalize_soft_line_breaks(actual).is_some_and(|normalized| normalized == expected)
}

fn normalize_soft_line_breaks(value: &str) -> Option<String> {
    if !value.contains('\n')
        || value.contains('\r')
        || value.contains('\t')
        || value.contains("<pre")
        || value.contains("</pre")
        || value.contains('`')
        || value.contains("~~~")
    {
        return None;
    }

    let lines: Vec<_> = value.split('\n').collect();
    if lines.iter().any(|line| line.is_empty()) {
        return None;
    }

    for (index, line) in lines.iter().enumerate() {
        if line.ends_with("  ") || line.ends_with('\\') {
            return None;
        }
        if index > 0 && is_markdown_block_start(line) {
            return None;
        }
    }

    Some(lines.join(" "))
}

fn is_markdown_block_start(line: &str) -> bool {
    if line.starts_with(char::is_whitespace) {
        return true;
    }

    let trimmed = line.trim_start();
    if ["- ", "* ", "+ ", "> ", "# "].iter().any(|marker| trimmed.starts_with(marker)) {
        return true;
    }

    let Some((marker, _)) = trimmed.split_once(' ') else {
        return false;
    };
    marker.strip_suffix('.').or_else(|| marker.strip_suffix(')')).is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn missing_key_divergence(
    json_path: String,
    expected: Value,
    actual: Value,
    present_value: &Value,
    empty_array_desc: &str,
    non_empty_desc: &str,
) -> Divergence {
    let is_empty_array = present_value.as_array().is_some_and(Vec::is_empty);
    let (classification, description) = if is_empty_array {
        (DivergenceClass::Acceptable, empty_array_desc.to_string())
    } else {
        (DivergenceClass::ForgeFix, non_empty_desc.to_string())
    };
    Divergence { json_path, expected, actual, classification, description, resolution: None }
}

fn compare_ordered_array(
    exp_arr: &[Value],
    act_arr: &[Value],
    path: &str,
    rules: &OscalComparisonRules,
    divergences: &mut Vec<Divergence>,
) {
    let max_len = exp_arr.len().max(act_arr.len());
    for i in 0..max_len {
        let child_path = format!("{path}/{i}");
        match (exp_arr.get(i), act_arr.get(i)) {
            (Some(e), Some(a)) => compare_values(e, a, &child_path, rules, divergences),
            (Some(e), None) => {
                divergences.push(Divergence {
                    json_path: child_path,
                    expected: e.clone(),
                    actual: Value::Null,
                    classification: DivergenceClass::ForgeFix,
                    description: "Element present in expected array but missing in actual"
                        .to_string(),
                    resolution: None,
                });
            }
            (None, Some(a)) => {
                divergences.push(Divergence {
                    json_path: child_path,
                    expected: Value::Null,
                    actual: a.clone(),
                    classification: DivergenceClass::ForgeFix,
                    description: "Extra element in actual array not present in expected"
                        .to_string(),
                    resolution: None,
                });
            }
            (None, None) => unreachable!(),
        }
    }
}

fn compare_unordered_array(
    exp_arr: &[Value],
    act_arr: &[Value],
    path: &str,
    rules: &OscalComparisonRules,
    divergences: &mut Vec<Divergence>,
) {
    let mut matched_actual: Vec<bool> = vec![false; act_arr.len()];

    for (exp_idx, exp_elem) in exp_arr.iter().enumerate() {
        let match_idx = find_matching_element(exp_elem, act_arr, &matched_actual);

        if let Some(act_idx) = match_idx {
            matched_actual[act_idx] = true;
            let child_path = format!("{path}/{exp_idx}");
            compare_values(exp_elem, &act_arr[act_idx], &child_path, rules, divergences);
        } else {
            let child_path = format!("{path}/{exp_idx}");
            divergences.push(Divergence {
                json_path: child_path,
                expected: exp_elem.clone(),
                actual: Value::Null,
                classification: DivergenceClass::ForgeFix,
                description: "Element in expected unordered array not found in actual".to_string(),
                resolution: None,
            });
        }
    }

    // Report unmatched actual elements
    for (act_idx, elem) in act_arr.iter().enumerate() {
        if !matched_actual[act_idx] {
            let child_path = format!("{path}/{act_idx}");
            divergences.push(Divergence {
                json_path: child_path,
                expected: Value::Null,
                actual: elem.clone(),
                classification: DivergenceClass::ForgeFix,
                description: "Extra element in actual unordered array not found in expected"
                    .to_string(),
                resolution: None,
            });
        }
    }
}

/// Find an element in `act_arr` that matches `exp_elem` by identity key.
///
/// Priority: uuid → name+ns composite → positional fallback.
fn find_matching_element(
    exp_elem: &Value,
    act_arr: &[Value],
    already_matched: &[bool],
) -> Option<usize> {
    if let Some(i) = find_by_exact_equality(exp_elem, act_arr, already_matched) {
        return Some(i);
    }

    if let Some(exp_uuid) = exp_elem.get("uuid").and_then(Value::as_str) {
        return find_by_uuid(exp_uuid, act_arr, already_matched);
    }

    if let Some(exp_name) = exp_elem.get("name").and_then(Value::as_str) {
        let exp_ns = exp_elem.get("ns").and_then(Value::as_str);
        return find_by_name_ns(exp_name, exp_ns, act_arr, already_matched);
    }

    find_positional_fallback(already_matched)
}

fn find_by_exact_equality(
    exp_elem: &Value,
    act_arr: &[Value],
    already_matched: &[bool],
) -> Option<usize> {
    act_arr
        .iter()
        .enumerate()
        .find(|(i, act_elem)| !already_matched[*i] && *act_elem == exp_elem)
        .map(|(i, _)| i)
}

fn find_by_uuid(exp_uuid: &str, act_arr: &[Value], already_matched: &[bool]) -> Option<usize> {
    act_arr
        .iter()
        .enumerate()
        .find(|(i, act_elem)| {
            !already_matched[*i] && act_elem.get("uuid").and_then(Value::as_str) == Some(exp_uuid)
        })
        .map(|(i, _)| i)
}

fn find_by_name_ns(
    exp_name: &str,
    exp_ns: Option<&str>,
    act_arr: &[Value],
    already_matched: &[bool],
) -> Option<usize> {
    act_arr
        .iter()
        .enumerate()
        .find(|(i, act_elem)| {
            !already_matched[*i]
                && act_elem.get("name").and_then(Value::as_str) == Some(exp_name)
                && act_elem.get("ns").and_then(Value::as_str) == exp_ns
        })
        .map(|(i, _)| i)
}

fn find_positional_fallback(already_matched: &[bool]) -> Option<usize> {
    already_matched.iter().position(|matched| !matched)
}

fn summarize_value(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if s.len() > 50 {
                let truncated: String = s.chars().take(47).collect();
                format!("\"{truncated}...\"")
            } else {
                format!("\"{s}\"")
            }
        }
        Value::Array(a) => format!("array[{}]", a.len()),
        Value::Object(o) => format!("object{{{} keys}}", o.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn default_rules() -> OscalComparisonRules {
        OscalComparisonRules::default()
    }

    // T009(a): Identical documents → empty divergence list
    #[test]
    fn identical_documents_produce_no_divergences() {
        let doc = json!({
            "catalog": {
                "metadata": { "title": "Test" },
                "groups": []
            }
        });
        let result = compare_oscal_json(&doc, &doc, "", &default_rules());
        assert!(result.is_empty(), "Identical documents should produce zero divergences");
    }

    // T009(b): Documents differing only in JSON object field order → empty list
    #[test]
    fn different_field_order_produces_no_divergences() {
        let expected = json!({"a": 1, "b": 2, "c": 3});
        let actual = json!({"c": 3, "a": 1, "b": 2});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert!(result.is_empty(), "Different field order should not produce divergences");
    }

    // T009(c): Scalar value difference → single divergence with correct path
    #[test]
    fn scalar_value_difference_produces_divergence() {
        let expected = json!({"catalog": {"metadata": {"title": "Original"}}});
        let actual = json!({"catalog": {"metadata": {"title": "Changed"}}});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json_path, "/catalog/metadata/title");
        assert_eq!(result[0].expected, json!("Original"));
        assert_eq!(result[0].actual, json!("Changed"));
    }

    #[test]
    fn equivalent_timestamp_spelling_is_acceptable() {
        let expected = json!({"last-modified": "2026-08-24T18:07:31.190199+00:00"});
        let actual = json!({"last-modified": "2026-08-24T18:07:31.190199Z"});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].classification, DivergenceClass::Acceptable);
    }

    #[test]
    fn prose_whitespace_normalization_is_acceptable() {
        let expected = json!({"prose": "first line\nsecond line"});
        let actual = json!({"prose": "first line second line"});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].classification, DivergenceClass::Acceptable);
    }

    #[test]
    fn markdown_sensitive_prose_whitespace_requires_a_forge_fix() {
        let cases = [
            ("<pre>line one\n  line two</pre>", "<pre>line one line two</pre>"),
            ("line one  \nline two", "line one line two"),
            ("first paragraph\n\nsecond paragraph", "first paragraph second paragraph"),
            ("- parent\n  - child", "- parent - child"),
        ];

        for (expected_prose, actual_prose) in cases {
            let expected = json!({"prose": expected_prose});
            let actual = json!({"prose": actual_prose});
            let result = compare_oscal_json(&expected, &actual, "", &default_rules());
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].classification, DivergenceClass::ForgeFix);
        }
    }

    // T009(d): Missing key in actual → divergence
    #[test]
    fn missing_key_in_actual_produces_divergence() {
        let expected = json!({"a": 1, "b": 2});
        let actual = json!({"a": 1});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json_path, "/b");
        assert_eq!(result[0].expected, json!(2));
        assert_eq!(result[0].actual, serde_json::Value::Null);
    }

    // T009(e): Extra key in actual → divergence
    #[test]
    fn extra_key_in_actual_produces_divergence() {
        let expected = json!({"a": 1});
        let actual = json!({"a": 1, "b": 2});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json_path, "/b");
        assert_eq!(result[0].expected, serde_json::Value::Null);
        assert_eq!(result[0].actual, json!(2));
    }

    // T009(f): Unordered array (props) with reordered elements matched by uuid → empty
    #[test]
    fn unordered_props_array_reordered_by_uuid_no_divergence() {
        let expected = json!({
            "props": [
                {"uuid": "aaa", "name": "x", "value": "1"},
                {"uuid": "bbb", "name": "y", "value": "2"}
            ]
        });
        let actual = json!({
            "props": [
                {"uuid": "bbb", "name": "y", "value": "2"},
                {"uuid": "aaa", "name": "x", "value": "1"}
            ]
        });
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert!(result.is_empty(), "Reordered props by uuid should produce no divergences");
    }

    // T009(g): Unordered array with element missing by uuid → divergence
    #[test]
    fn unordered_array_missing_element_by_uuid_produces_divergence() {
        let expected = json!({
            "props": [
                {"uuid": "aaa", "name": "x", "value": "1"},
                {"uuid": "bbb", "name": "y", "value": "2"}
            ]
        });
        let actual = json!({
            "props": [
                {"uuid": "aaa", "name": "x", "value": "1"}
            ]
        });
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert!(!result.is_empty(), "Missing element by uuid should produce a divergence");
    }

    // T009(h): Empty array in expected vs absent key → Acceptable divergence (EC-2)
    #[test]
    fn empty_array_vs_absent_key_is_acceptable() {
        let expected = json!({"controls": []});
        let actual = json!({});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].classification, DivergenceClass::Acceptable);
    }

    // T009(i): Key present in expected but absent in actual (non-empty value) → divergence
    #[test]
    fn key_present_in_expected_absent_in_actual_produces_divergence() {
        let expected = json!({"metadata": {"version": "1.0"}});
        let actual = json!({"metadata": {}});
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json_path, "/metadata/version");
    }

    // Regression: reordered links (no uuid/name) matched by exact equality
    #[test]
    fn reordered_links_matched_by_exact_equality() {
        let expected = json!({
            "links": [
                {"href": "https://example.com/a", "rel": "reference"},
                {"href": "https://example.com/b", "rel": "related"}
            ]
        });
        let actual = json!({
            "links": [
                {"href": "https://example.com/b", "rel": "related"},
                {"href": "https://example.com/a", "rel": "reference"}
            ]
        });
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert!(
            result.is_empty(),
            "Reordered links matched by exact equality should produce no divergences"
        );
    }

    // Regression: duplicate props with same (name, ns) matched by exact equality
    #[test]
    fn duplicate_name_ns_props_matched_by_exact_equality() {
        let expected = json!({
            "props": [
                {"name": "label", "ns": "https://example.com", "value": "A"},
                {"name": "label", "ns": "https://example.com", "value": "B"}
            ]
        });
        let actual = json!({
            "props": [
                {"name": "label", "ns": "https://example.com", "value": "B"},
                {"name": "label", "ns": "https://example.com", "value": "A"}
            ]
        });
        let result = compare_oscal_json(&expected, &actual, "", &default_rules());
        assert!(
            result.is_empty(),
            "Duplicate props matched by exact equality should produce no divergences"
        );
    }
}

//! OSCAL-aware recursive JSON comparison algorithm.

use std::collections::HashMap;

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
    compare_values(expected, actual, path, rules, None, &mut divergences);
    divergences
}

/// Escape one JSON Pointer token according to RFC 6901.
fn escape_json_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

fn compare_values(
    expected: &Value,
    actual: &Value,
    path: &str,
    rules: &OscalComparisonRules,
    positions: Option<(usize, usize)>,
    divergences: &mut Vec<Divergence>,
) {
    if rules.ignored_paths.iter().any(|ignored| {
        path == ignored || path.strip_prefix(ignored).is_some_and(|suffix| suffix.starts_with('/'))
    }) {
        return;
    }

    match (expected, actual) {
        (Value::Object(exp_map), Value::Object(act_map)) => {
            for key in exp_map.keys() {
                let child_path = format!("{path}/{}", escape_json_pointer_token(key));
                match act_map.get(key) {
                    Some(act_val) => {
                        compare_values(
                            &exp_map[key],
                            act_val,
                            &child_path,
                            rules,
                            positions,
                            divergences,
                        );
                    }
                    None => {
                        divergences.push(missing_key_divergence(
                            child_path,
                            exp_map[key].clone(),
                            Value::Null,
                            &exp_map[key],
                            "Empty array in expected vs absent key in actual",
                            "Key present in expected but absent in actual",
                            positions,
                        ));
                    }
                }
            }
            for key in act_map.keys() {
                if !exp_map.contains_key(key) {
                    let child_path = format!("{path}/{}", escape_json_pointer_token(key));
                    divergences.push(missing_key_divergence(
                        child_path,
                        Value::Null,
                        act_map[key].clone(),
                        &act_map[key],
                        "Absent key in expected vs empty array in actual",
                        "Extra key in actual not present in expected",
                        positions,
                    ));
                }
            }
        }
        (Value::Array(exp_arr), Value::Array(act_arr)) => {
            let key_name = path.rsplit('/').next().unwrap_or("");
            if rules.unordered_array_keys.contains(key_name) {
                compare_unordered_array(exp_arr, act_arr, path, rules, divergences);
            } else {
                compare_ordered_array(exp_arr, act_arr, path, rules, positions, divergences);
            }
        }
        _ => {
            if expected != actual {
                let acceptable = acceptable_scalar_normalization(expected, actual, path, rules);
                divergences.push(Divergence {
                    json_path: path.to_string(),
                    expected_index: positions.map(|(expected_index, _)| expected_index),
                    actual_index: positions.map(|(_, actual_index)| actual_index),
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
    rules: &OscalComparisonRules,
) -> Option<&'static str> {
    let (Some(expected), Some(actual)) = (expected.as_str(), actual.as_str()) else {
        return None;
    };

    if rules.acceptable_timestamp_path_suffixes.iter().any(|suffix| path.ends_with(suffix))
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
    if trimmed.starts_with('=')
        || trimmed.starts_with('|')
        || (trimmed.len() >= 3
            && trimmed.chars().all(|character| matches!(character, '-' | '*' | '_')))
        || ["- ", "* ", "+ ", "> ", "# "].iter().any(|marker| trimmed.starts_with(marker))
    {
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
    positions: Option<(usize, usize)>,
) -> Divergence {
    let is_empty_array = present_value.as_array().is_some_and(Vec::is_empty);
    let (classification, description) = if is_empty_array {
        (DivergenceClass::Acceptable, empty_array_desc.to_string())
    } else {
        (DivergenceClass::ForgeFix, non_empty_desc.to_string())
    };
    Divergence {
        json_path,
        expected_index: positions.map(|(expected_index, _)| expected_index),
        actual_index: positions.map(|(_, actual_index)| actual_index),
        expected,
        actual,
        classification,
        description,
        resolution: None,
    }
}

fn compare_ordered_array(
    exp_arr: &[Value],
    act_arr: &[Value],
    path: &str,
    rules: &OscalComparisonRules,
    positions: Option<(usize, usize)>,
    divergences: &mut Vec<Divergence>,
) {
    let max_len = exp_arr.len().max(act_arr.len());
    for i in 0..max_len {
        let child_path = format!("{path}/{i}");
        match (exp_arr.get(i), act_arr.get(i)) {
            (Some(e), Some(a)) => {
                compare_values(e, a, &child_path, rules, positions, divergences);
            }
            (Some(e), None) => {
                divergences.push(Divergence {
                    json_path: child_path,
                    expected_index: positions.map(|(expected_index, _)| expected_index),
                    actual_index: positions.map(|(_, actual_index)| actual_index),
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
                    expected_index: positions.map(|(expected_index, _)| expected_index),
                    actual_index: positions.map(|(_, actual_index)| actual_index),
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

struct ArrayIndexes<'a> {
    exact: HashMap<String, Vec<usize>>,
    uuid: HashMap<&'a str, Vec<usize>>,
    name_ns: HashMap<(&'a str, Option<&'a str>), Vec<usize>>,
}

impl<'a> ArrayIndexes<'a> {
    fn for_values(values: &'a [Value]) -> Self {
        let mut indexes =
            Self { exact: HashMap::new(), uuid: HashMap::new(), name_ns: HashMap::new() };

        for (index, value) in values.iter().enumerate() {
            indexes.exact.entry(canonical_value_key(value)).or_default().push(index);
            if let Some(uuid) = value.get("uuid").and_then(Value::as_str) {
                indexes.uuid.entry(uuid).or_default().push(index);
            }
            if let Some(name) = value.get("name").and_then(Value::as_str) {
                indexes
                    .name_ns
                    .entry((name, value.get("ns").and_then(Value::as_str)))
                    .or_default()
                    .push(index);
            }
        }

        indexes
    }
}

fn compare_unordered_array(
    exp_arr: &[Value],
    act_arr: &[Value],
    path: &str,
    rules: &OscalComparisonRules,
    divergences: &mut Vec<Divergence>,
) {
    let indexes = ArrayIndexes::for_values(act_arr);
    let mut matched_actual = vec![false; act_arr.len()];

    for (expected_index, expected_element) in exp_arr.iter().enumerate() {
        if let Some(actual_index) =
            find_matching_element(expected_element, &indexes, &matched_actual)
        {
            matched_actual[actual_index] = true;
            let child_path = format!("{path}/{expected_index}");
            compare_values(
                expected_element,
                &act_arr[actual_index],
                &child_path,
                rules,
                Some((expected_index, actual_index)),
                divergences,
            );
        } else {
            let child_path = format!("{path}/{expected_index}");
            divergences.push(Divergence {
                json_path: child_path,
                expected_index: Some(expected_index),
                actual_index: None,
                expected: expected_element.clone(),
                actual: Value::Null,
                classification: DivergenceClass::ForgeFix,
                description: "Element in expected unordered array not found in actual".to_string(),
                resolution: None,
            });
        }
    }

    for (actual_index, element) in act_arr.iter().enumerate() {
        if !matched_actual[actual_index] {
            let child_path = format!("{path}/{actual_index}");
            divergences.push(Divergence {
                json_path: child_path,
                expected_index: None,
                actual_index: Some(actual_index),
                expected: Value::Null,
                actual: element.clone(),
                classification: DivergenceClass::ForgeFix,
                description: "Extra element in actual unordered array not found in expected"
                    .to_string(),
                resolution: None,
            });
        }
    }
}

fn canonical_value_key(value: &Value) -> String {
    fn append_value(key: &mut String, value: &Value) {
        match value {
            Value::Null => key.push('n'),
            Value::Bool(value) => key.push(if *value { 't' } else { 'f' }),
            Value::Number(value) => {
                key.push('d');
                key.push_str(&value.to_string());
                key.push(';');
            }
            Value::String(value) => {
                key.push('s');
                key.push_str(&value.len().to_string());
                key.push(':');
                key.push_str(value);
            }
            Value::Array(values) => {
                key.push('[');
                for value in values {
                    append_value(key, value);
                }
                key.push(']');
            }
            Value::Object(values) => {
                let mut entries: Vec<_> = values.iter().collect();
                entries.sort_unstable_by_key(|(left, _)| *left);
                key.push('{');
                for (name, value) in entries {
                    key.push_str(&name.len().to_string());
                    key.push(':');
                    key.push_str(name);
                    append_value(key, value);
                }
                key.push('}');
            }
        }
    }

    let mut key = String::new();
    append_value(&mut key, value);
    key
}

fn first_unmatched(candidates: Option<&[usize]>, already_matched: &[bool]) -> Option<usize> {
    candidates
        .and_then(|candidates| candidates.iter().copied().find(|index| !already_matched[*index]))
}

/// Find an unmatched actual element using exact equality, UUID, name/namespace,
/// then positional fallback.
fn find_matching_element(
    expected_element: &Value,
    indexes: &ArrayIndexes<'_>,
    already_matched: &[bool],
) -> Option<usize> {
    first_unmatched(
        indexes.exact.get(&canonical_value_key(expected_element)).map(Vec::as_slice),
        already_matched,
    )
    .or_else(|| {
        expected_element.get("uuid").and_then(Value::as_str).and_then(|uuid| {
            first_unmatched(indexes.uuid.get(uuid).map(Vec::as_slice), already_matched)
        })
    })
    .or_else(|| {
        expected_element.get("name").and_then(Value::as_str).and_then(|name| {
            let namespace = expected_element.get("ns").and_then(Value::as_str);
            first_unmatched(
                indexes.name_ns.get(&(name, namespace)).map(Vec::as_slice),
                already_matched,
            )
        })
    })
    .or_else(|| find_positional_fallback(already_matched))
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

    #[test]
    fn markdown_block_continuations_are_not_soft_line_breaks() {
        let cases = [
            ("Overview\n-----", "Overview -----"),
            ("Intro\n***", "Intro ***"),
            ("| Heading |\n| --- |", "| Heading | | --- |"),
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

    #[test]
    fn regenerated_uuid_falls_through_to_name_namespace_matching() {
        let expected = json!({
            "props": [{"uuid": "expected-uuid", "name": "label", "ns": "https://example.com"}]
        });
        let actual = json!({
            "props": [{"uuid": "actual-uuid", "name": "label", "ns": "https://example.com"}]
        });

        let result = compare_oscal_json(&expected, &actual, "", &default_rules());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json_path, "/props/0/uuid");
        assert_eq!(result[0].expected_index, Some(0));
        assert_eq!(result[0].actual_index, Some(0));
    }

    #[test]
    fn unordered_divergence_reports_both_array_positions() {
        let expected = json!({
            "props": [
                {"uuid": "first", "name": "label", "value": "expected"},
                {"uuid": "second", "name": "label", "value": "unchanged"}
            ]
        });
        let actual = json!({
            "props": [
                {"uuid": "second", "name": "label", "value": "unchanged"},
                {"uuid": "first", "name": "label", "value": "actual"}
            ]
        });

        let result = compare_oscal_json(&expected, &actual, "", &default_rules());

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json_path, "/props/0/value");
        assert_eq!(result[0].expected_index, Some(0));
        assert_eq!(result[0].actual_index, Some(1));
    }

    #[test]
    fn metadata_timestamp_spellings_are_acceptable() {
        for key in ["published", "updated"] {
            let expected = json!({"metadata": {(key): "2026-08-26T12:00:00Z"}});
            let actual = json!({"metadata": {(key): "2026-08-26T12:00:00+00:00"}});

            let result = compare_oscal_json(&expected, &actual, "", &default_rules());

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].classification, DivergenceClass::Acceptable);
        }
    }

    #[test]
    fn ignored_path_prefix_skips_nested_comparison() {
        let expected = json!({"metadata": {"title": "expected"}});
        let actual = json!({"metadata": {"title": "actual"}});
        let mut rules = default_rules();
        rules.ignored_paths.push("/metadata".to_string());

        let result = compare_oscal_json(&expected, &actual, "", &rules);

        assert!(result.is_empty());
    }

    #[test]
    fn object_key_paths_use_rfc_6901_escaping() {
        let expected = json!({ "a/b~c": "expected" });
        let actual = json!({ "a/b~c": "actual" });

        let divergences = compare_oscal_json(&expected, &actual, "", &default_rules());

        assert_eq!(divergences.len(), 1);
        assert_eq!(divergences[0].json_path, "/a~1b~0c");
    }
}

//! Schema error formatting utilities (WI-20).
//!
//! Converts raw `jsonschema` crate errors into user-friendly `ValidationError`
//! structs with JSON Path notation, expected constraints, and truncated actual values.

use jsonschema::error::{TypeKind, ValidationErrorKind};
use serde_json::Value;

use super::error_types::{ValidationError, ValidationErrorCategory};

/// Truncate a string to `max_len` characters, appending "..." if truncated.
///
/// SEC-1: Actual values in errors MUST be truncated to 100 characters.
#[must_use]
pub fn truncate_value(value: &str, max_len: usize) -> String {
    if value.chars().count() <= max_len {
        value.to_string()
    } else {
        let truncated: String = value.chars().take(max_len).collect();
        format!("{truncated}...")
    }
}

/// Convert a JSON Pointer (RFC 6901) to JSON Path notation.
///
/// # Examples
/// - `""` → `"$"`
/// - `"/catalog/metadata/uuid"` → `"$.catalog.metadata.uuid"`
/// - `"/catalog/groups/0/controls/2/id"` → `"$.catalog.groups[0].controls[2].id"`
///
/// Handles malformed pointers gracefully — no panics (SEC-6).
#[must_use]
pub fn pointer_to_json_path(pointer: &str) -> String {
    if pointer.is_empty() {
        return "$".to_string();
    }

    let mut result = String::from("$");

    // Split on '/' and skip the first empty segment (from leading '/')
    let segments: Vec<&str> = pointer.split('/').collect();
    let start = usize::from(segments.first() == Some(&""));

    for segment in &segments[start..] {
        // RFC 6901: unescape ~1 → /, then ~0 → ~ (order matters)
        let unescaped = segment.replace("~1", "/").replace("~0", "~");
        if unescaped.chars().all(|c| c.is_ascii_digit()) && !unescaped.is_empty() {
            result.push('[');
            result.push_str(&unescaped);
            result.push(']');
        } else {
            result.push('.');
            result.push_str(&unescaped);
        }
    }

    result
}

/// Extract the actual value at a JSON Pointer path, serialized and truncated.
///
/// Navigates the JSON tree to the location specified by `pointer`,
/// serializes the value found there, and truncates via `truncate_value()`.
/// Returns `"(not found)"` if the path does not resolve.
fn extract_actual_value(json: &Value, pointer: &str) -> String {
    if pointer.is_empty() {
        return truncate_value(&json.to_string(), 100);
    }

    match json.pointer(pointer) {
        Some(value) => {
            let serialized = match value {
                Value::String(s) => format!("\"{s}\""),
                Value::Null => "null".to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                other => other.to_string(),
            };
            truncate_value(&serialized, 100)
        }
        None => "(not found)".to_string(),
    }
}

/// Format a raw `jsonschema` crate error into an actionable `ValidationError`.
///
/// Transforms the raw error by:
/// 1. Converting `instance_path` from JSON Pointer to JSON Path
/// 2. Extracting the expected constraint from the error context
/// 3. Extracting the actual value from the JSON instance (truncated to 100 chars)
///
/// Raw crate messages are NEVER passed through to the output (SEC-2).
#[must_use]
pub fn format_schema_error(
    raw_error: &jsonschema::ValidationError,
    json: &Value,
) -> ValidationError {
    let instance_path = raw_error.instance_path().to_string();
    let path = pointer_to_json_path(&instance_path);
    let (message, expected) = classify_error(raw_error);
    let actual = extract_actual_value(json, &instance_path);

    ValidationError { category: ValidationErrorCategory::Schema, path, message, expected, actual }
}

/// Classify a structured `jsonschema` error into a user-friendly message and expectation.
///
/// SEC-2: Never pass through raw crate messages. Classification uses the
/// validator's structured error kind so wording changes cannot alter output.
fn classify_error(error: &jsonschema::ValidationError) -> (String, String) {
    match error.kind() {
        ValidationErrorKind::Required { property } => {
            let field = property.as_str().unwrap_or("unknown");
            (format!("required field missing: {field}"), "required field".to_string())
        }
        ValidationErrorKind::Type { kind } => {
            let type_name = match kind {
                TypeKind::Single(type_name) => type_name.to_string(),
                TypeKind::Multiple(type_names) => type_names
                    .into_iter()
                    .map(|type_name| type_name.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            };
            (format!("wrong type: expected {type_name}"), format!("type: {type_name}"))
        }
        ValidationErrorKind::AnyOf { .. } | ValidationErrorKind::OneOfNotValid { .. } => (
            "value does not match any allowed schema".to_string(),
            "valid schema match".to_string(),
        ),
        ValidationErrorKind::MaxLength { limit } => (
            format!("string too long: {limit} characters"),
            format!("max length: {limit} characters"),
        ),
        ValidationErrorKind::MinLength { limit } => (
            format!("string too short: {limit} characters"),
            format!("min length: {limit} characters"),
        ),
        ValidationErrorKind::Pattern { .. } => {
            ("value does not match required pattern".to_string(), "pattern match".to_string())
        }
        ValidationErrorKind::Format { format } => {
            (format!("invalid format: expected {format}"), format!("format: {format}"))
        }
        ValidationErrorKind::AdditionalProperties { unexpected } => (
            format!("unexpected additional properties: {}", unexpected.join(", ")),
            "no additional properties".to_string(),
        ),
        ValidationErrorKind::Enum { .. } => {
            ("value not in allowed set".to_string(), "one of the allowed values".to_string())
        }
        _ => ("schema validation failed".to_string(), "valid value per schema".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T007: truncate_value tests ---

    #[test]
    fn truncate_short_string_no_truncation() {
        assert_eq!(truncate_value("hello", 100), "hello");
    }

    #[test]
    fn truncate_exactly_100_chars_no_truncation() {
        let s = "a".repeat(100);
        assert_eq!(truncate_value(&s, 100), s);
    }

    #[test]
    fn truncate_101_chars_truncated_with_ellipsis() {
        let s = "a".repeat(101);
        let result = truncate_value(&s, 100);
        assert_eq!(result, format!("{}...", "a".repeat(100)));
        assert!(result.ends_with("..."));
    }

    #[test]
    fn truncate_empty_string() {
        assert_eq!(truncate_value("", 100), "");
    }

    #[test]
    fn truncate_unicode_at_boundary() {
        let s = "🔥".repeat(101);
        let result = truncate_value(&s, 100);
        let expected = format!("{}...", "🔥".repeat(100));
        assert_eq!(result, expected);
    }

    #[test]
    fn truncate_value_zero_max() {
        assert_eq!(truncate_value("hello", 0), "...");
    }

    // --- T013: pointer_to_json_path tests ---

    #[test]
    fn pointer_empty_returns_root() {
        assert_eq!(pointer_to_json_path(""), "$");
    }

    #[test]
    fn pointer_simple_path() {
        assert_eq!(pointer_to_json_path("/catalog"), "$.catalog");
    }

    #[test]
    fn pointer_nested_path() {
        assert_eq!(pointer_to_json_path("/catalog/metadata/uuid"), "$.catalog.metadata.uuid");
    }

    #[test]
    fn pointer_with_array_indices() {
        assert_eq!(pointer_to_json_path("/groups/0/controls/2/id"), "$.groups[0].controls[2].id");
    }

    #[test]
    fn pointer_deeply_nested_6_levels() {
        assert_eq!(
            pointer_to_json_path("/catalog/groups/0/controls/5/parts/0/props/3/value"),
            "$.catalog.groups[0].controls[5].parts[0].props[3].value"
        );
    }

    #[test]
    fn pointer_rfc6901_escape_sequences() {
        // RFC 6901: ~1 → /, ~0 → ~
        assert_eq!(pointer_to_json_path("/props~1name~0value"), "$.props/name~value");
        assert_eq!(pointer_to_json_path("/a~0b"), "$.a~b");
        assert_eq!(pointer_to_json_path("/a~1b"), "$.a/b");
    }

    #[test]
    fn pointer_malformed_no_leading_slash() {
        // SEC-6: graceful handling, no panic
        let result = pointer_to_json_path("catalog/metadata");
        assert!(result.starts_with('$'));
        assert!(result.contains("catalog"));
    }

    // --- T014: format_schema_error tests ---

    #[test]
    fn format_missing_required_field() {
        let schema_json: Value = serde_json::from_str(
            r#"{
                "type": "object",
                "required": ["name"],
                "properties": {
                    "name": { "type": "string" }
                }
            }"#,
        )
        .unwrap();
        let instance: Value = serde_json::from_str(r"{}").unwrap();

        let validator = jsonschema::validator_for(&schema_json).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();
        assert!(!errors.is_empty(), "Expected validation errors");

        let formatted = format_schema_error(&errors[0], &instance);
        assert_eq!(formatted.category, ValidationErrorCategory::Schema);
        assert!(!formatted.message.is_empty());
        assert!(!formatted.expected.is_empty());
        // SEC-2: should not contain raw crate text patterns
        assert!(!formatted.message.contains("jsonschema"));
        assert!(!formatted.message.contains("::"));
    }

    #[test]
    fn format_wrong_type_error() {
        let schema_json: Value = serde_json::from_str(
            r#"{
                "type": "object",
                "properties": {
                    "age": { "type": "string" }
                }
            }"#,
        )
        .unwrap();
        let instance: Value = serde_json::from_str(r#"{"age": 42}"#).unwrap();

        let validator = jsonschema::validator_for(&schema_json).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();
        assert!(!errors.is_empty());

        let formatted = format_schema_error(&errors[0], &instance);
        assert_eq!(formatted.category, ValidationErrorCategory::Schema);
        assert!(formatted.path.contains("age"));
        assert!(!formatted.message.contains("::"));
    }

    #[test]
    fn format_pattern_error_uses_structured_kind() {
        let schema = serde_json::json!({"pattern": "^[apples]+C"});
        let instance = serde_json::json!("banana");
        let validator = jsonschema::validator_for(&schema).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();

        assert_eq!(errors[0].to_string(), r#""banana" does not match "^[apples]+C""#);

        let formatted = format_schema_error(&errors[0], &instance);
        assert_eq!(formatted.message, "value does not match required pattern");
        assert_eq!(formatted.expected, "pattern match");
        assert_eq!(formatted.actual, "\"banana\"");
    }

    #[test]
    fn format_format_error_uses_structured_kind() {
        let schema = serde_json::json!({"format": "email"});
        let instance = serde_json::json!("not-an-email");
        let validator = jsonschema::options().should_validate_formats(true).build(&schema).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();

        assert_eq!(errors[0].to_string(), r#""not-an-email" is not a "email""#);

        let formatted = format_schema_error(&errors[0], &instance);
        assert_eq!(formatted.message, "invalid format: expected email");
        assert_eq!(formatted.expected, "format: email");
        assert_eq!(formatted.actual, "\"not-an-email\"");
    }

    #[test]
    fn format_actual_value_truncated_to_100_chars() {
        let long_value = "x".repeat(200);
        let schema_json: Value = serde_json::from_str(
            r#"{
                "type": "object",
                "properties": {
                    "name": { "type": "integer" }
                }
            }"#,
        )
        .unwrap();
        let instance: Value =
            serde_json::from_str(&format!(r#"{{"name": "{long_value}"}}"#)).unwrap();

        let validator = jsonschema::validator_for(&schema_json).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();
        assert!(!errors.is_empty());

        let formatted = format_schema_error(&errors[0], &instance);
        // SEC-1: actual must be <= 100 chars + "..."
        assert!(
            formatted.actual.chars().count() <= 103,
            "Actual should be truncated: {}",
            formatted.actual
        );
    }

    #[test]
    fn format_no_rust_module_paths_in_output() {
        // SEC-4: no Rust module paths in output
        let schema_json: Value =
            serde_json::from_str(r#"{"type": "object", "required": ["id"]}"#).unwrap();
        let instance: Value = serde_json::from_str(r"{}").unwrap();

        let validator = jsonschema::validator_for(&schema_json).unwrap();
        let errors: Vec<_> = validator.iter_errors(&instance).collect();
        let formatted = format_schema_error(&errors[0], &instance);

        assert!(!formatted.message.contains("src/"));
        assert!(!formatted.message.contains("mod::"));
        assert!(!formatted.path.contains("src/"));
    }

    // --- extract_actual_value tests ---

    #[test]
    fn extract_actual_at_root() {
        let json: Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        let result = extract_actual_value(&json, "");
        assert!(result.contains('a'));
    }

    #[test]
    fn extract_actual_not_found() {
        let json: Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        let result = extract_actual_value(&json, "/nonexistent/path");
        assert_eq!(result, "(not found)");
    }

    #[test]
    fn extract_actual_string_value() {
        let json: Value = serde_json::from_str(r#"{"name": "hello"}"#).unwrap();
        let result = extract_actual_value(&json, "/name");
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn extract_actual_number_value() {
        let json: Value = serde_json::from_str(r#"{"count": 42}"#).unwrap();
        let result = extract_actual_value(&json, "/count");
        assert_eq!(result, "42");
    }
}

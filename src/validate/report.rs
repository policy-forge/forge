//! Validation report renderers (WI-20).
//!
//! Renders `ValidationReport` as human-readable text (PRD S-2)
//! or machine-parseable JSON (PRD S-1).

use std::fmt::Write;

use super::error_types::{ValidationErrorCategory, ValidationReport};

/// Render a `ValidationReport` as human-readable text (PRD S-2).
///
/// Output format:
/// ```text
/// Validation failed: 3 schema errors, 1 semantic error
///
/// Schema Errors:
///   [1] $.catalog.metadata.uuid — required field missing
///       Expected: required string field
///       Actual: field not present
/// ```
#[must_use]
pub fn render_text_report(report: &ValidationReport) -> String {
    if report.is_valid() {
        return format!("Valid: {} artifact passes all validation.", report.artifact_path());
    }

    let mut output = String::new();

    // Summary line
    let mut parts = Vec::new();
    if report.schema_error_count() > 0 {
        parts.push(format!(
            "{} schema error{}",
            report.schema_error_count(),
            if report.schema_error_count() == 1 { "" } else { "s" }
        ));
    }
    if report.semantic_error_count() > 0 {
        parts.push(format!(
            "{} semantic error{}",
            report.semantic_error_count(),
            if report.semantic_error_count() == 1 { "" } else { "s" }
        ));
    }
    let _ = writeln!(output, "Validation failed: {}", parts.join(", "));

    // Schema errors section
    let schema_errors: Vec<_> =
        report.errors().iter().filter(|e| e.category == ValidationErrorCategory::Schema).collect();
    if !schema_errors.is_empty() {
        output.push_str("\nSchema Errors:\n");
        for (i, error) in schema_errors.iter().enumerate() {
            let _ = write!(
                output,
                "  [{}] {} — {}\n      Expected: {}\n      Actual: {}\n",
                i + 1,
                error.path,
                error.message,
                error.expected,
                error.actual
            );
        }
    }

    // Semantic errors section
    let semantic_errors: Vec<_> = report
        .errors()
        .iter()
        .filter(|e| e.category == ValidationErrorCategory::Semantic)
        .collect();
    if !semantic_errors.is_empty() {
        output.push_str("\nSemantic Errors:\n");
        for (i, error) in semantic_errors.iter().enumerate() {
            let _ = write!(
                output,
                "  [{}] {} — {}\n      Expected: {}\n      Actual: {}\n",
                i + 1,
                error.path,
                error.message,
                error.expected,
                error.actual
            );
        }
    }

    output
}

/// Render a `ValidationReport` as JSON (PRD S-1).
///
/// Uses `serde_json` serialization of the `ValidationReport` struct.
/// Output MUST contain only `ValidationReport`/`ValidationError` fields (SEC-3).
#[must_use]
pub fn render_json_report(report: &ValidationReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|_| {
        // SEC-3: fallback must conform to ValidationReport schema (no extra fields).
        r#"{"artifact_path":"","is_valid":false,"errors":[],"schema_error_count":0,"semantic_error_count":0}"#.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::error_types::{ValidationError, ValidationErrorCategory};

    fn make_schema_error(path: &str, message: &str) -> ValidationError {
        ValidationError {
            category: ValidationErrorCategory::Schema,
            path: path.to_string(),
            message: message.to_string(),
            expected: "expected value".to_string(),
            actual: "actual value".to_string(),
        }
    }

    fn make_semantic_error(path: &str, message: &str) -> ValidationError {
        ValidationError {
            category: ValidationErrorCategory::Semantic,
            path: path.to_string(),
            message: message.to_string(),
            expected: "expected value".to_string(),
            actual: "actual value".to_string(),
        }
    }

    // --- T025: render_text_report tests ---

    #[test]
    fn text_valid_report() {
        let report = ValidationReport::new("test.json".to_string(), vec![]);
        let text = render_text_report(&report);
        assert!(text.contains("Valid"));
    }

    #[test]
    fn text_single_schema_error() {
        let errors = vec![make_schema_error("$.catalog.uuid", "invalid uuid")];
        let report = ValidationReport::new("test.json".to_string(), errors);
        let text = render_text_report(&report);
        assert!(text.contains("$.catalog.uuid"));
        assert!(text.contains("Expected:"));
        assert!(text.contains("Actual:"));
    }

    #[test]
    fn text_mixed_errors_grouped_by_category() {
        let errors = vec![
            make_schema_error("$.a", "err1"),
            make_schema_error("$.b", "err2"),
            make_semantic_error("$.c", "err3"),
        ];
        let report = ValidationReport::new("test.json".to_string(), errors);
        let text = render_text_report(&report);
        assert!(text.contains("2 schema errors, 1 semantic error"));
        assert!(text.contains("Schema Errors:"));
        assert!(text.contains("Semantic Errors:"));
    }

    #[test]
    fn text_50_plus_errors_all_rendered() {
        // PRD EC-2: 50+ errors all rendered without truncation
        let errors: Vec<_> = (0..55)
            .map(|i| make_schema_error(&format!("$.field{i}"), &format!("error {i}")))
            .collect();
        let report = ValidationReport::new("test.json".to_string(), errors);
        let text = render_text_report(&report);
        assert!(text.contains("55 schema errors"));
        assert!(text.contains("$.field54"));
    }

    #[test]
    fn text_deeply_nested_path_displayed_fully() {
        // PRD EC-6: deeply nested path not truncated
        let deep_path = "$.catalog.groups[0].controls[5].parts[0].props[3].value";
        let errors = vec![make_schema_error(deep_path, "deep error")];
        let report = ValidationReport::new("test.json".to_string(), errors);
        let text = render_text_report(&report);
        assert!(text.contains(deep_path));
    }

    // --- T026: render_json_report tests ---

    #[test]
    fn json_valid_report() {
        let report = ValidationReport::new("test.json".to_string(), vec![]);
        let json = render_json_report(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["is_valid"], true);
        assert!(parsed["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn json_report_with_errors() {
        let errors = vec![make_schema_error("$.a", "test error")];
        let report = ValidationReport::new("test.json".to_string(), errors);
        let json = render_json_report(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["is_valid"], false);
        assert_eq!(parsed["schema_error_count"], 1);
        assert_eq!(parsed["errors"][0]["category"], "Schema");
        assert_eq!(parsed["errors"][0]["path"], "$.a");
    }

    #[test]
    fn json_report_contains_only_defined_fields() {
        // SEC-3: only defined fields
        let report = ValidationReport::new("test.json".to_string(), vec![]);
        let json = render_json_report(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = parsed.as_object().unwrap();
        let allowed_keys =
            ["artifact_path", "is_valid", "errors", "schema_error_count", "semantic_error_count"];
        for key in obj.keys() {
            assert!(allowed_keys.contains(&key.as_str()), "Unexpected key in JSON: {key}");
        }
    }

    #[test]
    fn json_round_trip() {
        let errors = vec![make_schema_error("$.a", "err1"), make_semantic_error("$.b", "err2")];
        let original = ValidationReport::new("test.json".to_string(), errors);
        let json = render_json_report(&original);
        let parsed: ValidationReport = serde_json::from_str(&json).unwrap();
        assert_eq!(original.is_valid(), parsed.is_valid());
        assert_eq!(original.errors().len(), parsed.errors().len());
        assert_eq!(original.schema_error_count(), parsed.schema_error_count());
        assert_eq!(original.semantic_error_count(), parsed.semantic_error_count());
    }
}

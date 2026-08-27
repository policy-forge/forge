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
    let declared = report.declared_oscal_version().unwrap_or("unavailable");
    if report.is_valid() {
        let support_warning = if report.supported_input() {
            ""
        } else {
            "\n  Warning: input support policy violation reported"
        };
        return format!(
            "Valid: {} artifact passes all validation.\n  Artifact: {}\n  Declared OSCAL version: {}\n  Schema version used: {}{support_warning}\n",
            report.model_type(),
            report.artifact_path(),
            declared,
            report.schema_version_used()
        );
    }

    // Classify once so summary counts and rendered sections cannot diverge.
    let schema_errors: Vec<_> = report
        .errors()
        .iter()
        .filter(|error| error.category == ValidationErrorCategory::Schema)
        .collect();
    let semantic_errors: Vec<_> = report
        .errors()
        .iter()
        .filter(|error| error.category == ValidationErrorCategory::Semantic)
        .collect();

    let mut output = String::new();
    let mut parts = Vec::new();
    if !schema_errors.is_empty() {
        parts.push(format!(
            "{} schema error{}",
            schema_errors.len(),
            if schema_errors.len() == 1 { "" } else { "s" }
        ));
    }
    if !semantic_errors.is_empty() {
        parts.push(format!(
            "{} semantic error{}",
            semantic_errors.len(),
            if semantic_errors.len() == 1 { "" } else { "s" }
        ));
    }
    let _ = writeln!(output, "Validation failed: {}", parts.join(", "));
    let _ = writeln!(output, "  Model: {}", report.model_type());
    let _ = writeln!(output, "  Declared OSCAL version: {declared}");
    let _ = writeln!(output, "  Schema version used: {}", report.schema_version_used());

    if !schema_errors.is_empty() {
        output.push_str("\nSchema Errors:\n");
        for (index, error) in schema_errors.iter().enumerate() {
            let _ = write!(
                output,
                "  [{}] {} — {}\n      Expected: {}\n      Actual: {}\n",
                index + 1,
                error.path,
                error.message,
                error.expected,
                error.actual()
            );
        }
    }

    if !semantic_errors.is_empty() {
        output.push_str("\nSemantic Errors:\n");
        for (index, error) in semantic_errors.iter().enumerate() {
            let _ = write!(
                output,
                "  [{}] {} — {}\n      Expected: {}\n      Actual: {}\n",
                index + 1,
                error.path,
                error.message,
                error.expected,
                error.actual()
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
    serde_json::to_string_pretty(report).unwrap_or_else(|error| {
        tracing::error!(error = %error, "ValidationReport serialization failed; returning fallback JSON structure");
        let fallback = ValidationReport::new(String::new(), Vec::new());
        serde_json::to_string_pretty(&fallback).unwrap_or_else(|fallback_error| {
            tracing::error!(error = %fallback_error, "ValidationReport fallback serialization failed");
            "{}".to_owned()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::error_types::{ValidationError, ValidationErrorCategory};

    fn make_schema_error(path: &str, message: &str) -> ValidationError {
        ValidationError::new(
            ValidationErrorCategory::Schema,
            path.to_string(),
            message.to_string(),
            "expected value".to_string(),
            "actual value",
        )
    }

    fn make_semantic_error(path: &str, message: &str) -> ValidationError {
        ValidationError::new(
            ValidationErrorCategory::Semantic,
            path.to_string(),
            message.to_string(),
            "expected value".to_string(),
            "actual value",
        )
    }

    // --- T025: render_text_report tests ---

    #[test]
    fn text_valid_report() {
        let report = ValidationReport::new("test.json".to_string(), vec![]);
        let text = render_text_report(&report);
        assert!(text.contains("Valid"));
    }

    #[test]
    fn unsupported_valid_report_warns_in_text_and_json() {
        let report = ValidationReport::new_with_context(
            "legacy.json".to_string(),
            "catalog".to_string(),
            Some("1.0.0".to_string()),
            false,
            vec![],
        );
        let text = render_text_report(&report);
        assert!(text.ends_with('\n'));
        assert!(text.contains("input support policy violation"));

        let json: serde_json::Value = serde_json::from_str(&render_json_report(&report)).unwrap();
        assert_eq!(json["supported_input"], false);
        assert_eq!(json["is_valid"], true);
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
        let allowed_keys = [
            "artifact_path",
            "model_type",
            "declared_oscal_version",
            "schema_version_used",
            "supported_input",
            "is_valid",
            "errors",
            "schema_error_count",
            "semantic_error_count",
        ];
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

    #[test]
    fn contextual_report_exposes_declared_and_actual_baselines() {
        let report = ValidationReport::new_with_context(
            "legacy-catalog.json".to_string(),
            "catalog".to_string(),
            Some("1.2.0".to_string()),
            true,
            vec![],
        );
        let json: serde_json::Value =
            serde_json::from_str(&render_json_report(&report)).expect("report must be JSON");
        assert_eq!(json["model_type"], "catalog");
        assert_eq!(json["declared_oscal_version"], "1.2.0");
        assert_eq!(json["schema_version_used"], "1.2.3");
        assert_eq!(json["supported_input"], true);

        let text = render_text_report(&report);
        assert!(text.contains("Declared OSCAL version: 1.2.0"));
        assert!(text.contains("Schema version used: 1.2.3"));
    }
}

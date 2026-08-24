//! OSCAL declaration inspection for the single pinned compatibility baseline.

use serde_json::Value;

use super::{OscalModelType, ValidationError, ValidationErrorCategory};

/// Oldest OSCAL declaration accepted by the current compatibility policy.
pub const MIN_SUPPORTED_OSCAL_VERSION: &str = "1.2.0";
/// Pinned schema baseline and newest accepted OSCAL declaration.
pub const SCHEMA_VERSION_USED: &str = crate::oscal::metadata::OSCAL_VERSION;

/// Result of inspecting `metadata.oscal-version` without selecting a schema.
#[derive(Debug, Clone)]
pub struct VersionInspection {
    /// Exact string declaration when the field is a string.
    pub declared: Option<String>,
    /// Whether the declaration is in the supported v1.2.0-v1.2.3 range.
    pub supported: bool,
    /// Actionable policy error for missing, malformed, or unsupported declarations.
    pub error: Option<ValidationError>,
}

/// Return whether a declaration is a canonical supported OSCAL version.
///
/// Parsing is numeric and requires exactly three canonical decimal components;
/// prereleases, prefixes, whitespace, and leading-zero variants are rejected.
#[must_use]
pub fn is_supported_oscal_version(value: &str) -> bool {
    let Some(declared) = parse_version(value) else {
        return false;
    };
    let Some(minimum) = parse_version(MIN_SUPPORTED_OSCAL_VERSION) else {
        return false;
    };
    let Some(maximum) = parse_version(SCHEMA_VERSION_USED) else {
        return false;
    };

    declared >= minimum && declared <= maximum
}

fn parse_version(value: &str) -> Option<(u64, u64, u64)> {
    let mut components = value.split('.');
    let major = parse_component(components.next())?;
    let minor = parse_component(components.next())?;
    let patch = parse_component(components.next())?;
    if components.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_component(component: Option<&str>) -> Option<u64> {
    let component = component?;
    if component.is_empty()
        || !component.bytes().all(|byte| byte.is_ascii_digit())
        || (component.len() > 1 && component.starts_with('0'))
    {
        return None;
    }
    component.parse().ok()
}

/// Inspect the declaration for a detected model and produce reporting context.
#[must_use]
pub fn inspect_oscal_version(json: &Value, model_type: OscalModelType) -> VersionInspection {
    let path = format!("$.{}.metadata.oscal-version", model_type.as_str());
    let value = json
        .get(model_type.as_str())
        .and_then(|model| model.get("metadata"))
        .and_then(|metadata| metadata.get("oscal-version"));

    if let Some(declared) = value.and_then(Value::as_str) {
        if is_supported_oscal_version(declared) {
            return VersionInspection {
                declared: Some(declared.to_string()),
                supported: true,
                error: None,
            };
        }

        let safe_declared = escape_for_diagnostic(declared);
        return VersionInspection {
            declared: Some(safe_declared.clone()),
            supported: false,
            error: Some(version_error(
                path,
                format!(
                    "unsupported OSCAL version declaration '{safe_declared}'; available schema baseline is {SCHEMA_VERSION_USED}"
                ),
                safe_declared,
            )),
        };
    }

    let actual = value.map_or_else(
        || "field not present".to_string(),
        |value| escape_for_diagnostic(&value.to_string()),
    );
    VersionInspection {
        declared: None,
        supported: false,
        error: Some(version_error(
            path,
            format!(
                "OSCAL version declaration must be a non-empty supported string; available schema baseline is {SCHEMA_VERSION_USED}"
            ),
            actual,
        )),
    }
}

fn version_error(path: String, message: String, actual: String) -> ValidationError {
    ValidationError {
        category: ValidationErrorCategory::Schema,
        path,
        message,
        expected: format!(
            "a canonical OSCAL version from {MIN_SUPPORTED_OSCAL_VERSION} through {SCHEMA_VERSION_USED}"
        ),
        actual,
    }
}

fn escape_for_diagnostic(value: &str) -> String {
    value.chars().take(100).flat_map(char::escape_default).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_range_is_parsed_numerically() {
        for version in ["1.2.0", "1.2.1", "1.2.2", "1.2.3"] {
            assert!(is_supported_oscal_version(version), "{version} should be supported");
        }
    }

    #[test]
    fn unsupported_and_noncanonical_versions_are_rejected() {
        for version in [
            "1.1.9",
            "1.2.4",
            "1.2.10",
            "1.3.0",
            "1.2.3-rc1",
            "v1.2.3",
            "01.2.3",
            "1.02.3",
            "1.2.03",
            "1.2",
            "1.2.3.0",
            " 1.2.3",
            "1.2.3 ",
            "",
            "   ",
        ] {
            assert!(!is_supported_oscal_version(version), "{version:?} should be rejected");
        }
    }

    #[test]
    fn inspection_reports_declared_and_schema_versions_separately() {
        let json = serde_json::json!({
            "catalog": {"metadata": {"version": "1.2.3", "oscal-version": "1.2.0"}}
        });
        let inspection = inspect_oscal_version(&json, OscalModelType::Catalog);
        assert_eq!(inspection.declared.as_deref(), Some("1.2.0"));
        assert!(inspection.supported);
        assert!(inspection.error.is_none());
        assert_eq!(SCHEMA_VERSION_USED, "1.2.3");
    }

    #[test]
    fn inspection_rejects_missing_non_string_and_unsupported_declarations() {
        for value in [Value::Null, serde_json::json!(123), serde_json::json!("1.3.0")] {
            let json = serde_json::json!({"profile": {"metadata": {"oscal-version": value}}});
            let inspection = inspect_oscal_version(&json, OscalModelType::Profile);
            assert!(!inspection.supported);
            let error = inspection.error.expect("invalid declaration must produce an error");
            assert_eq!(error.path, "$.profile.metadata.oscal-version");
            assert!(error.message.contains("1.2.3"));
        }
    }

    #[test]
    fn invalid_declaration_is_bounded_in_report_context() {
        let declaration = format!("{}\n{}", "x".repeat(99), "y".repeat(500));
        let json = serde_json::json!({
            "catalog": {"metadata": {"oscal-version": declaration}}
        });
        let inspection = inspect_oscal_version(&json, OscalModelType::Catalog);
        let declared = inspection.declared.expect("string declaration should be reported");
        assert_eq!(declared, format!("{}\\n", "x".repeat(99)));
        assert!(!declared.contains('\n'));
    }
}

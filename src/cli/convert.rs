use std::path::Path;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};

/// Execute the convert subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute(
    input: &Path,
    strategy: &Strategy,
    format: &OutputFormat,
    output: Option<&Path>,
    max_size: u64,
    source_profile: Option<&str>,
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))?;

    // W-2: reject non-JSON formats (XML/YAML deferred to WI-26, WI-27)
    if !matches!(format, OutputFormat::Json) {
        return Err(ForgeError::Validation(
            "Only 'json' output format is currently supported. XML and YAML formats will be available in a future release.".to_string(),
        ));
    }

    match strategy {
        Strategy::Catalog => crate::pipeline::run_catalog_pipeline(input, output, max_size_bytes),
        Strategy::Component => {
            // Runtime validation for --source-profile (SEC-3, SEC-4, EC-4)
            let profile = match source_profile {
                None => {
                    return Err(ForgeError::Validation(
                        "--source-profile is required when using --strategy component".to_string(),
                    ));
                }
                Some(p) if p.trim().is_empty() => {
                    return Err(ForgeError::Validation(
                        "--source-profile must not be empty".to_string(),
                    ));
                }
                Some(p) => p,
            };
            crate::pipeline::run_component_pipeline(input, output, max_size_bytes, profile)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn component_strategy_missing_source_profile_errors() {
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            None,
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("--source-profile is required"),
            "Expected source-profile required error, got: {err}"
        );
    }

    #[test]
    fn component_strategy_empty_source_profile_errors() {
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            Some(""),
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("--source-profile must not be empty"),
            "Expected empty source-profile error, got: {err}"
        );
    }

    #[test]
    fn component_strategy_whitespace_only_source_profile_errors() {
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            Some("   "),
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("--source-profile must not be empty"),
            "Expected empty source-profile error for whitespace-only, got: {err}"
        );
    }
}

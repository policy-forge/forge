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
            let profile_ref = match source_profile {
                None => {
                    tracing::warn!(
                        "--source-profile not provided; control-id mapping will be skipped. The generated Component Definition will have empty control-implementations."
                    );
                    None
                }
                Some(p) if p.trim().is_empty() => {
                    return Err(ForgeError::Validation(
                        "--source-profile must not be empty".to_string(),
                    ));
                }
                Some(p) => {
                    // SEC-3: Validate source-profile path exists and is a regular file
                    let profile_path = std::path::Path::new(p);
                    if !profile_path.exists() {
                        return Err(ForgeError::Validation(format!(
                            "--source-profile path '{}' does not exist (not found)",
                            profile_path.display()
                        )));
                    }
                    if !profile_path.is_file() {
                        return Err(ForgeError::Validation(format!(
                            "--source-profile path '{}' is not a regular file (is a directory or special file)",
                            profile_path.display()
                        )));
                    }
                    Some(p)
                }
            };
            crate::pipeline::run_component_pipeline(input, output, max_size_bytes, profile_ref)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn component_strategy_none_source_profile_does_not_error_on_missing_profile() {
        // T013: With source_profile: None, should NOT get source-profile-required error.
        // It will fail on file-not-found (test.md doesn't exist), which proves the profile
        // check is no longer blocking.
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            None,
        );
        let err = result.unwrap_err();
        // Should NOT be about source-profile — should be about the missing input file
        assert!(
            !err.to_string().contains("--source-profile is required"),
            "Should not require source-profile. Got: {err}"
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

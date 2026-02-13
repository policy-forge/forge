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
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))?;

    // S-3: reject unsupported strategies
    // W-2: reject non-JSON formats (XML/YAML deferred to WI-26, WI-27)
    if !matches!(format, OutputFormat::Json) {
        return Err(ForgeError::Validation(
            "Only 'json' output format is currently supported. XML and YAML formats will be available in a future release.".to_string(),
        ));
    }

    match strategy {
        Strategy::Catalog => {
            crate::pipeline::run_catalog_pipeline(input, output, max_size_bytes)
        }
        Strategy::Component => Err(ForgeError::Validation(
            "Only 'catalog' strategy is currently supported. Component support will be available in a future release.".to_string(),
        )),
    }
}

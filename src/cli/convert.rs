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
    _format: &OutputFormat,
    output: Option<&Path>,
    max_size: u64,
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))?;

    match strategy {
        Strategy::Catalog => {
            crate::pipeline::run_catalog_pipeline(input, output, max_size_bytes)
        }
        Strategy::Component => Err(ForgeError::Validation(
            "Only 'catalog' strategy is currently supported. Component support will be available in a future release.".to_string(),
        )),
    }
}

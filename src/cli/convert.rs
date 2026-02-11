use std::path::Path;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};
use crate::ingest;

/// Execute the convert subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute(
    input: &Path,
    _strategy: Option<&Strategy>,
    _format: &OutputFormat,
    _output: Option<&Path>,
    max_size: u64,
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))?;
    let doc = ingest::ingest_file(input, max_size_bytes)?;
    let json = serde_json::to_string_pretty(&doc).map_err(|e| ForgeError::Parse(e.to_string()))?;
    println!("{json}");
    Ok(())
}

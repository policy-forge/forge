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
    let max_size_bytes = max_size * 1024 * 1024;
    let doc = ingest::ingest_file(input, max_size_bytes)?;
    let json = serde_json::to_string_pretty(&doc).map_err(|e| ForgeError::Parse(e.to_string()))?;
    println!("{json}");
    Ok(())
}

use std::path::Path;

use crate::ForgeError;
use crate::cli::{OutputFormat, Strategy};

/// Execute the convert subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute(
    _input: &Path,
    _strategy: Option<&Strategy>,
    _format: &OutputFormat,
    _output: Option<&Path>,
) -> Result<(), ForgeError> {
    println!("Convert command not yet implemented");
    Ok(())
}

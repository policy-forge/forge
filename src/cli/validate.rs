use std::path::Path;

use crate::ForgeError;

/// Execute the validate subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the validation fails.
pub fn execute(_input: &Path) -> Result<(), ForgeError> {
    println!("Validate command not yet implemented");
    Ok(())
}

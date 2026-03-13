//! Shared CLI output utilities.

use std::path::Path;

use crate::error::ForgeError;

/// Write content to a file (atomically) or stdout.
///
/// # Errors
/// * `ForgeError::Validation` if parent directory does not exist
/// * `ForgeError::Io` if file write fails
pub fn write_output(content: &str, output_path: Option<&Path>) -> Result<(), ForgeError> {
    match output_path {
        None => {
            println!("{content}");
            Ok(())
        }
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !parent.exists()
            {
                return Err(ForgeError::Validation(format!(
                    "Output directory '{}' does not exist",
                    parent.display()
                )));
            }
            crate::io::write_atomic(path, content.as_bytes())?;
            Ok(())
        }
    }
}

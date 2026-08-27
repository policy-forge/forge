//! Shared CLI output utilities.

use std::io::{self, Write};
use std::path::Path;

use crate::error::ForgeError;

/// Write content to a file (atomically) or stdout.
///
/// Handles `BrokenPipe` gracefully (e.g., when piped into `head`) instead of
/// panicking like `print!` would.
///
/// # Errors
/// * `ForgeError::Validation` if parent directory does not exist
/// * `ForgeError::Io` if writing stdout or the named output file fails
pub fn write_output(content: &str, output_path: Option<&Path>) -> Result<(), ForgeError> {
    match output_path {
        None => {
            let mut stdout = io::stdout().lock();
            match stdout.write_all(content.as_bytes()) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                Err(error) => {
                    return Err(ForgeError::Io(io::Error::new(
                        error.kind(),
                        format!("failed writing to stdout: {error}"),
                    )));
                }
            }
            match stdout.flush() {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                Err(error) => {
                    return Err(ForgeError::Io(io::Error::new(
                        error.kind(),
                        format!("failed flushing stdout: {error}"),
                    )));
                }
            }
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
            crate::io::write_atomic(path, content.as_bytes()).map_err(|error| {
                ForgeError::Io(io::Error::other(format!(
                    "failed writing output to '{}': {error}",
                    path.display()
                )))
            })?;
            Ok(())
        }
    }
}

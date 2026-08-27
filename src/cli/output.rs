//! Shared CLI output utilities.

use std::io::{self, Write};
use std::path::Path;

use crate::error::ForgeError;

/// Write content to a file (atomically) or stdout.
///
/// `BrokenPipe` from stdout writing or flushing is treated as successful so pipelines
/// such as `forge … | head` exit cleanly. File writes use [`crate::io::write_atomic`].
///
/// # Errors
/// * `ForgeError::Validation` if the output parent is absent at the advisory
///   preflight check. The parent can still disappear before the atomic write.
/// * `ForgeError::Io` if writing or flushing stdout, or creating/writing the
///   atomic file output, fails.
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
            // This only improves the missing-parent diagnostic; `write_atomic` remains
            // authoritative because the filesystem can change after this check.
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

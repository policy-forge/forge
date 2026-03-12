use std::path::Path;

use crate::diff::{diff_artifacts, format_diff_report};
use crate::error::ForgeError;

/// # Errors
///
/// Returns `ForgeError::DiffError` for file I/O, JSON parsing, and artifact comparison errors.
pub fn execute(old_path: &Path, new_path: &Path) -> Result<bool, ForgeError> {
    tracing::info!(old = %old_path.display(), new = %new_path.display(), "Starting diff");

    let report = diff_artifacts(old_path, new_path)?;

    tracing::info!(artifact_type = %report.artifact_type, "Artifact type detected");

    let output = format_diff_report(&report);
    print!("{output}");

    Ok(report.summary.has_changes())
}

use std::path::{Path, PathBuf};

use crate::cli::MigrationOutputFormat;
use crate::error::ForgeError;

/// Execute a read-only policy migration analysis and emit its complete report.
///
/// Returns `true` when the completed report contains any reviewable change.
/// Analysis failures return before output is written.
///
/// # Errors
///
/// Returns [`ForgeError::MigrationError`] when an input cannot be analyzed,
/// the size is invalid, the output aliases an input, or the report cannot be written.
pub fn execute(
    old_policy: &Path,
    new_policy: &Path,
    format: &MigrationOutputFormat,
    output: Option<&Path>,
    max_size_mb: u64,
) -> Result<bool, ForgeError> {
    reject_output_alias(output, old_policy, new_policy)?;
    let max_size_bytes = max_size_mb
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::MigrationError("--max-size value is too large".to_string()))?;
    let report = crate::migration::analyze_paths(old_policy, new_policy, max_size_bytes)?;
    let rendered = match format {
        MigrationOutputFormat::Text => crate::migration::format_text(&report),
        MigrationOutputFormat::Json => crate::migration::format_json(&report)?,
    };
    crate::cli::output::write_output(&rendered, output)
        .map_err(|error| ForgeError::MigrationError(error.to_string()))?;
    Ok(report.has_reviewable_changes())
}

fn reject_output_alias(
    output: Option<&Path>,
    old_policy: &Path,
    new_policy: &Path,
) -> Result<(), ForgeError> {
    let Some(output) = output else {
        return Ok(());
    };
    let output_identity = path_identity(output)?;
    for (label, input) in [("old policy", old_policy), ("new policy", new_policy)] {
        let input_identity = path_identity(input)?;
        if output_identity == input_identity {
            return Err(ForgeError::MigrationError(format!(
                "--output must not overwrite the {label}"
            )));
        }
    }
    Ok(())
}

fn path_identity(path: &Path) -> Result<PathBuf, ForgeError> {
    if path.exists() {
        return path.canonicalize().map_err(|error| {
            ForgeError::MigrationError(format!("unable to resolve path: {error}"))
        });
    }
    let parent =
        path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let canonical_parent = parent.canonicalize().map_err(|error| {
        ForgeError::MigrationError(format!("unable to resolve output directory: {error}"))
    })?;
    let file_name = path
        .file_name()
        .ok_or_else(|| ForgeError::MigrationError("--output must name a file".to_string()))?;
    Ok(canonical_parent.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_output_that_aliases_an_input() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("old.md");
        let new = directory.path().join("new.md");
        std::fs::write(&old, "# Old\n").unwrap();
        std::fs::write(&new, "# New\n").unwrap();
        let result = reject_output_alias(Some(&old), &old, &new);
        assert!(matches!(result, Err(ForgeError::MigrationError(_))));
    }
}

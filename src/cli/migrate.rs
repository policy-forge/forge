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
    successor_map: Option<&Path>,
    max_size_mb: u64,
) -> Result<bool, ForgeError> {
    reject_output_alias(output, old_policy, new_policy, successor_map)?;
    let max_size_bytes = max_size_mb
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::MigrationError("--max-size value is too large".to_string()))?;
    let report =
        crate::migration::analyze_paths(old_policy, new_policy, successor_map, max_size_bytes)?;
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
    successor_map: Option<&Path>,
) -> Result<(), ForgeError> {
    let Some(output) = output else {
        return Ok(());
    };
    let output_identity = path_identity(output, "--output path")?;
    for (label, input) in [("old policy path", old_policy), ("new policy path", new_policy)] {
        let input_identity = path_identity(input, label)?;
        if output_identity == input_identity {
            return Err(ForgeError::MigrationError(format!(
                "--output must not overwrite the {label}"
            )));
        }
    }
    if let Some(successor_map) = successor_map {
        let successor_identity = path_identity(successor_map, "successor map path")?;
        if output_identity == successor_identity {
            return Err(ForgeError::MigrationError(
                "--output must not overwrite the successor map path".to_string(),
            ));
        }
    }
    Ok(())
}

fn path_identity(path: &Path, role: &str) -> Result<PathBuf, ForgeError> {
    if path.exists() {
        return path.canonicalize().map_err(|error| {
            ForgeError::MigrationError(format!("unable to resolve {role}: {error}"))
        });
    }
    let parent =
        path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let canonical_parent = parent.canonicalize().map_err(|error| {
        ForgeError::MigrationError(format!("unable to resolve parent directory of {role}: {error}"))
    })?;
    let file_name = path
        .file_name()
        .ok_or_else(|| ForgeError::MigrationError(format!("{role} must name a file")))?;
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
        let result = reject_output_alias(Some(&old), &old, &new, None);
        assert!(matches!(result, Err(ForgeError::MigrationError(_))));
    }

    #[test]
    fn missing_input_directory_names_the_input_role() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("missing").join("old.md");
        let new = directory.path().join("new.md");
        let output = directory.path().join("report.json");
        std::fs::write(&new, "# New\n").unwrap();

        let error = reject_output_alias(Some(&output), &old, &new, None).unwrap_err().to_string();
        assert!(error.contains("parent directory of old policy path"), "unexpected error: {error}");
        assert!(!error.contains("output directory"), "unexpected error: {error}");
    }
}

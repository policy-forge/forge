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
/// Returns [`ForgeError::UnsupportedFormat`] (exit 1) when either policy
/// extension is outside the shared migration set: `.md`, `.markdown`, `.pdf`,
/// or `.docx`. Returns [`ForgeError::MigrationError`] when supported inputs
/// cannot be analyzed, the size is invalid, or the output aliases an input.
/// Output failures retain their original [`ForgeError`] variant for accurate
/// diagnostics and exit codes.
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
    // Analysis can be slow enough for an output path to be swapped after the first check.
    reject_output_alias(output, old_policy, new_policy, successor_map)?;
    crate::cli::output::write_output(&rendered, output)?;
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
    for (label, input) in [("old policy path", old_policy), ("new policy path", new_policy)] {
        if paths_alias(output, input)? {
            return Err(ForgeError::MigrationError(format!(
                "--output must not overwrite the {label}"
            )));
        }
    }
    if let Some(successor_map) = successor_map
        && paths_alias(output, successor_map)?
    {
        return Err(ForgeError::MigrationError(
            "--output must not overwrite the successor map path".to_string(),
        ));
    }
    Ok(())
}

fn path_identity(path: &Path, role: &str) -> Result<PathBuf, ForgeError> {
    match path.canonicalize() {
        Ok(canonical_path) => Ok(canonical_path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let canonical_parent = parent.canonicalize().map_err(|parent_error| {
                ForgeError::MigrationError(format!(
                    "unable to resolve parent directory of {role}: {parent_error}"
                ))
            })?;
            let file_name = path
                .file_name()
                .ok_or_else(|| ForgeError::MigrationError(format!("{role} must name a file")))?;
            Ok(canonical_parent.join(file_name))
        }
        Err(error) => Err(ForgeError::MigrationError(format!("unable to resolve {role}: {error}"))),
    }
}

fn paths_alias(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    if path_identity(left, "--output path")? == path_identity(right, "input path")? {
        return Ok(true);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let left_metadata = match std::fs::metadata(left) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(ForgeError::MigrationError(format!(
                    "unable to inspect '--output path': {error}"
                )));
            }
        };
        let right_metadata = match std::fs::metadata(right) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(ForgeError::MigrationError(format!(
                    "unable to inspect input path: {error}"
                )));
            }
        };
        Ok(left_metadata.dev() == right_metadata.dev()
            && left_metadata.ino() == right_metadata.ino())
    }

    #[cfg(windows)]
    {
        crate::mapping::paths_alias(left, right)
            .map_err(|error| ForgeError::MigrationError(error.to_string()))
    }

    #[cfg(not(any(unix, windows)))]
    {
        Ok(false)
    }
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

    #[cfg(unix)]
    #[test]
    fn rejects_output_hard_linked_to_an_input() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("failed to create temporary directory: {error}"));
        let old = directory.path().join("old.md");
        let new = directory.path().join("new.md");
        let output = directory.path().join("report.json");
        std::fs::write(&old, "# Old\n")
            .unwrap_or_else(|error| panic!("failed to write old policy: {error}"));
        std::fs::write(&new, "# New\n")
            .unwrap_or_else(|error| panic!("failed to write new policy: {error}"));
        std::fs::hard_link(&old, &output)
            .unwrap_or_else(|error| panic!("failed to create hard link: {error}"));

        let result = reject_output_alias(Some(&output), &old, &new, None);

        assert!(
            matches!(result, Err(ForgeError::MigrationError(message)) if message.contains("old policy path"))
        );
    }

    #[test]
    fn rejects_output_that_aliases_successor_map() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("old.md");
        let new = directory.path().join("new.md");
        let successor_map = directory.path().join("successors.json");
        std::fs::write(&old, "# Old\n").unwrap();
        std::fs::write(&new, "# New\n").unwrap();
        std::fs::write(&successor_map, "{}").unwrap();

        let result = reject_output_alias(Some(&successor_map), &old, &new, Some(&successor_map));

        assert!(
            matches!(result, Err(ForgeError::MigrationError(message)) if message.contains("successor map"))
        );
    }

    #[test]
    fn accepts_distinct_successor_map_output() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("old.md");
        let new = directory.path().join("new.md");
        let successor_map = directory.path().join("successors.json");
        let output = directory.path().join("report.json");
        std::fs::write(&old, "# Old\n").unwrap();
        std::fs::write(&new, "# New\n").unwrap();
        std::fs::write(&successor_map, "{}").unwrap();

        assert!(reject_output_alias(Some(&output), &old, &new, Some(&successor_map)).is_ok());
    }

    #[test]
    fn missing_input_directory_names_the_input_role() {
        let directory = tempfile::tempdir().unwrap();
        let old = directory.path().join("missing").join("old.md");
        let new = directory.path().join("new.md");
        let output = directory.path().join("report.json");
        std::fs::write(&new, "# New\n").unwrap();

        let error = reject_output_alias(Some(&output), &old, &new, None).unwrap_err().to_string();
        assert!(error.contains("parent directory of input path"), "unexpected error: {error}");
        assert!(!error.contains("output directory"), "unexpected error: {error}");
    }
}

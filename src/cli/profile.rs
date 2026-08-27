//! CLI handler for `forge profile` — OSCAL Profile generation subcommand (WI-30/WI-31).

use std::io::{BufWriter, Write};
use std::path::Path;

use tracing::info;

use super::OutputFormat;
use crate::error::ForgeError;
use crate::oscal::profile::{ProfileRoot, SelectionMode, build_profile, parse_control_ids};

/// Serialize a profile directly to an atomically replaced output file.
fn write_profile_file(
    root: &ProfileRoot,
    format: OutputFormat,
    output: &Path,
) -> Result<(), ForgeError> {
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temp = tempfile::NamedTempFile::new_in(parent).map_err(|error| {
        ForgeError::Io(std::io::Error::other(format!(
            "failed creating output for '{}': {error}",
            output.display()
        )))
    })?;
    let mut writer = BufWriter::new(temp);

    match format {
        OutputFormat::Json => serde_json::to_writer_pretty(&mut writer, root).map_err(|error| {
            ForgeError::Serialization(format!("Profile JSON serialization failed: {error}"))
        })?,
        OutputFormat::Xml => writer
            .write_all(
                crate::export::xml_serializer::serialize_profile_to_xml(&root.profile)?.as_bytes(),
            )
            .map_err(ForgeError::Io)?,
        OutputFormat::Yaml => serde_yaml::to_writer(&mut writer, root).map_err(|error| {
            ForgeError::Serialization(format!("Profile YAML serialization failed: {error}"))
        })?,
    }
    writer.flush().map_err(ForgeError::Io)?;
    let temp = writer.into_inner().map_err(|error| ForgeError::Io(error.into_error()))?;
    temp.as_file().sync_all().map_err(ForgeError::Io)?;
    #[cfg(unix)]
    if let Ok(existing) = std::fs::metadata(output) {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(existing.permissions().mode());
        // Best effort: a concurrent removal must not prevent an otherwise-safe write.
        let _ = std::fs::set_permissions(temp.path(), permissions);
    }
    let persisted = temp.persist(output).map_err(|error| {
        ForgeError::Io(std::io::Error::other(format!(
            "failed persisting output '{}': {}",
            output.display(),
            error.error
        )))
    })?;
    persisted.sync_all().map_err(ForgeError::Io)?;
    #[cfg(unix)]
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(ForgeError::Io)?;
    Ok(())
}

/// Execute the `forge profile` subcommand.
///
/// # Arguments
///
/// * `catalog` — Path to the source Catalog file (must be a regular file).
/// * `include` — Comma-separated control IDs to include, or `None`.
/// * `exclude` — Comma-separated control IDs to exclude, or `None`.
/// * `format` — Output format: `json`, `xml`, or `yaml`.
/// * `output` — Optional output file path; if `None`, writes to stdout.
/// * `set_params` — Flat `[id, value, id, value, ...]` slice from `--set-param` flags (WI-31).
///   Pass `&[]` when no `--set-param` flags are provided.
/// * `timestamp` — Optional RFC 3339 `last-modified` override for reproducible output.
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — neither `--include` nor `--exclude` nor `--set-param`
///   provided, empty parameter IDs, a non-regular catalog path, or a non-UTF-8 catalog path.
/// * `ForgeError::FileNotFound` — catalog path does not exist.
/// * `ForgeError::Io` — catalog metadata cannot be read or output cannot be written.
///
/// # Behavior
///
/// 1. Validate exactly one of `include` or `exclude` is `Some` (C-2: warn if neither + `set_params`).
/// 2. Validate that `catalog` is a regular file with a UTF-8-safe path.
/// 3. Parse control IDs via `parse_control_ids`.
/// 4. Determine `SelectionMode`.
/// 5. Call `build_profile` with parsed parameter overrides.
/// 6. Wrap in `ProfileRoot` and serialize.
/// 7. Write to `output` path or stdout.
pub fn execute(
    catalog: &Path,
    include: Option<&str>,
    exclude: Option<&str>,
    format: &OutputFormat,
    output: Option<&Path>,
    set_params: &[String],
    timestamp: Option<&str>,
) -> Result<(), ForgeError> {
    let pairs = parse_set_param_pairs(set_params)?;
    tracing::info!(
        param_count = pairs.len(),
        "profile: {} parameter override(s) specified",
        pairs.len()
    );

    let (control_ids, mode) = match (include, exclude) {
        (Some(inc), None) => (parse_control_ids(inc)?, SelectionMode::Include),
        (None, Some(exc)) => (parse_control_ids(exc)?, SelectionMode::Exclude),
        (None, None) => {
            if set_params.is_empty() {
                return Err(ForgeError::InvalidArgument(
                    "Either --include or --exclude must be provided".to_string(),
                ));
            }
            tracing::warn!(
                "--set-param specified without --include or --exclude; Profile will have no control imports"
            );
            (vec![], SelectionMode::Include)
        }
        (Some(_), Some(_)) => {
            return Err(ForgeError::InvalidArgument(
                "--include and --exclude are mutually exclusive".to_string(),
            ));
        }
    };

    match std::fs::metadata(catalog) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => {
            return Err(ForgeError::InvalidArgument(format!(
                "catalog path '{}' is not a regular file",
                catalog.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ForgeError::FileNotFound { path: catalog.to_path_buf() });
        }
        Err(error) => return Err(ForgeError::Io(error)),
    }

    let ts_override = match timestamp {
        Some(ts_str) => {
            let parsed = chrono::DateTime::parse_from_rfc3339(ts_str).map_err(|e| {
                ForgeError::InvalidArgument(format!(
                    "--timestamp must be a valid ISO 8601 / RFC 3339 string: {e}"
                ))
            })?;
            Some(parsed.with_timezone(&chrono::Utc))
        }
        None => None,
    };

    let catalog_str = catalog.to_str().ok_or_else(|| {
        ForgeError::InvalidArgument(format!(
            "catalog path '{}' is not valid UTF-8; refusing to embed a corrupted href",
            catalog.display()
        ))
    })?;
    let control_count = control_ids.len();
    let oscal_profile = build_profile(catalog_str, control_ids, mode, &pairs, ts_override)?;

    info!(
        catalog = %catalog.display(),
        selected_controls = control_count,
        "Profile generation complete"
    );

    let root = ProfileRoot { profile: oscal_profile };
    if let Some(output) = output {
        write_profile_file(&root, *format, output)
    } else {
        let serialized = match format {
            OutputFormat::Json => serde_json::to_string_pretty(&root).map_err(|error| {
                ForgeError::Serialization(format!("Profile JSON serialization failed: {error}"))
            })?,
            OutputFormat::Xml => {
                crate::export::xml_serializer::serialize_profile_to_xml(&root.profile)?
            }
            OutputFormat::Yaml => crate::export::yaml::serialize_to_yaml(&root)?,
        };
        crate::cli::output::write_output(&serialized, None)
    }
}

/// Parse a flat `[id, value, id, value, ...]` slice into normalized `(id, value)` pairs.
///
/// Returns `Err(ForgeError::InvalidArgument)` if the slice has an odd length
/// (i.e., an unpaired value) or an ID is empty after trimming. In practice clap's
/// `num_args = 2` prevents the former, but the check is retained as a defensive invariant.
fn parse_set_param_pairs(set_params: &[String]) -> Result<Vec<(String, String)>, ForgeError> {
    if set_params.len() % 2 != 0 {
        return Err(ForgeError::InvalidArgument(format!(
            "--set-param requires ID VALUE pairs but received an odd number of arguments ({})",
            set_params.len()
        )));
    }

    set_params
        .chunks_exact(2)
        .map(|pair| {
            let id = pair[0].trim();
            let value = &pair[1];
            if id.is_empty() {
                return Err(ForgeError::InvalidArgument(
                    "Empty or whitespace-only --set-param ID".to_string(),
                ));
            }
            Ok((id.to_string(), value.clone()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{execute, parse_set_param_pairs};
    use crate::error::ForgeError;
    use crate::types::OutputFormat;

    #[test]
    fn profile_file_output_is_serialized_atomically() {
        let directory = tempfile::tempdir().unwrap();
        let catalog = directory.path().join("catalog.json");
        let output = directory.path().join("profile.json");
        std::fs::write(&catalog, "{}").unwrap();

        execute(&catalog, Some("ac-1"), None, &OutputFormat::Json, Some(&output), &[], None)
            .unwrap();

        let rendered: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&output).unwrap()).unwrap();
        assert!(rendered.get("profile").is_some());
    }

    #[test]
    fn parameter_identifier_is_trimmed_before_profile_generation() {
        let pairs = parse_set_param_pairs(&[" prm ".to_string(), "value".to_string()])
            .unwrap_or_else(|error| panic!("unexpected parse failure: {error}"));

        assert_eq!(pairs, [("prm".to_string(), "value".to_string())]);
    }

    #[test]
    fn catalog_directory_is_rejected_as_non_regular_file() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("failed to create temporary directory: {error}"));
        let error =
            execute(directory.path(), Some("ac-1"), None, &OutputFormat::Json, None, &[], None)
                .unwrap_err();

        assert!(
            matches!(error, ForgeError::InvalidArgument(message) if message.contains("not a regular file"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_catalog_path_is_rejected_without_lossy_href() {
        use std::os::unix::ffi::OsStrExt;

        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("failed to create temporary directory: {error}"));
        let catalog = directory.path().join(OsStr::from_bytes(b"catalog-\xff.json"));
        if let Err(error) = std::fs::write(&catalog, "{}") {
            // Some Unix filesystems (notably macOS defaults) reject malformed
            // UTF-8 path bytes before FORGE can exercise its own boundary.
            eprintln!("skipping non-UTF-8 path test: {error}");
            return;
        }

        let error =
            execute(Path::new(&catalog), Some("ac-1"), None, &OutputFormat::Json, None, &[], None)
                .unwrap_err();

        assert!(
            matches!(error, ForgeError::InvalidArgument(message) if message.contains("not valid UTF-8"))
        );
    }
}

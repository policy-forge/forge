//! CLI handler for `forge profile` — OSCAL Profile generation subcommand (WI-30).

use std::path::Path;

use tracing::info;

use super::OutputFormat;
use crate::error::ForgeError;
use crate::oscal::profile::{ProfileRoot, SelectionMode, build_profile, parse_control_ids};

/// Execute the `forge profile` subcommand.
///
/// # Arguments
///
/// * `catalog` — Path to the source Catalog file (checked for existence).
/// * `include` — Comma-separated control IDs to include, or `None`.
/// * `exclude` — Comma-separated control IDs to exclude, or `None`.
/// * `format` — Output format (only `OutputFormat::Json` supported in WI-30).
/// * `output` — Optional output file path; if `None`, writes to stdout.
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — neither `--include` nor `--exclude` provided,
///   or empty control ID list after parsing.
/// * `ForgeError::FileNotFound` — catalog path does not exist.
/// * `ForgeError::Io` — output file write failure.
///
/// # Behavior
///
/// 1. Validate exactly one of `include` or `exclude` is `Some`.
/// 2. Check that `catalog` path exists.
/// 3. Parse control IDs via `parse_control_ids`.
/// 4. Determine `SelectionMode`.
/// 5. Call `build_profile`.
/// 6. Wrap in `ProfileRoot` and serialize.
/// 7. Write to `output` path or stdout.
pub fn execute(
    catalog: &Path,
    include: Option<&str>,
    exclude: Option<&str>,
    _format: &OutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    // Step 1: validate exactly one selection flag is provided
    let (raw_ids, mode) = match (include, exclude) {
        (Some(inc), None) => (inc, SelectionMode::Include),
        (None, Some(exc)) => (exc, SelectionMode::Exclude),
        (None, None) => {
            return Err(ForgeError::InvalidArgument(
                "Either --include or --exclude must be provided".to_string(),
            ));
        }
        (Some(_), Some(_)) => {
            // clap conflicts_with handles this case; defensive fallback
            return Err(ForgeError::InvalidArgument(
                "--include and --exclude are mutually exclusive".to_string(),
            ));
        }
    };

    // Step 2: check catalog exists
    if !catalog.exists() {
        return Err(ForgeError::FileNotFound { path: catalog.to_path_buf() });
    }

    // Step 3: parse control IDs
    let catalog_str = catalog.to_string_lossy();
    let control_ids = parse_control_ids(raw_ids)?;

    // Step 4+5: build profile
    let oscal_profile = build_profile(&catalog_str, control_ids.clone(), mode)?;

    info!(
        catalog = %catalog.display(),
        selected_controls = control_ids.len(),
        "Profile generation complete"
    );

    // Step 6: wrap and serialize
    let root = ProfileRoot { profile: oscal_profile };
    let json = serde_json::to_string_pretty(&root).map_err(|e| {
        ForgeError::Serialization(format!("Profile JSON serialization failed: {e}"))
    })?;

    // Step 7: write to file or stdout
    match output {
        Some(path) => std::fs::write(path, json)?,
        None => println!("{json}"),
    }

    Ok(())
}

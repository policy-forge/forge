//! CLI handler for `forge profile` — OSCAL Profile generation subcommand (WI-30/WI-31).

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
/// * `format` — Output format: `json`, `xml`, or `yaml`.
/// * `output` — Optional output file path; if `None`, writes to stdout.
/// * `set_params` — Flat `[id, value, id, value, ...]` slice from `--set-param` flags (WI-31).
///   Pass `&[]` when no `--set-param` flags are provided.
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — neither `--include` nor `--exclude` nor `--set-param`
///   provided, or empty control ID list after parsing. C-2: when `--set-param` is given without
///   a selection flag, a warning is emitted and the Profile is generated with empty imports (exit 0).
/// * `ForgeError::FileNotFound` — catalog path does not exist.
/// * `ForgeError::Io` — output file write failure.
///
/// # Behavior
///
/// 1. Validate exactly one of `include` or `exclude` is `Some` (C-2: warn if neither + `set_params`).
/// 2. Check that `catalog` path exists.
/// 3. Parse control IDs via `parse_control_ids`.
/// 4. Determine `SelectionMode`.
/// 5. Call `build_profile` with parsed param overrides.
/// 6. Wrap in `ProfileRoot` and serialize.
/// 7. Write to `output` path or stdout.
pub fn execute(
    catalog: &Path,
    include: Option<&str>,
    exclude: Option<&str>,
    format: &OutputFormat,
    output: Option<&Path>,
    set_params: &[String],
) -> Result<(), ForgeError> {
    // Parse param overrides from flat [id, value, id, value, ...] slice
    let pairs = parse_set_param_pairs(set_params)?;
    for (id, _) in &pairs {
        if id.trim().is_empty() {
            return Err(ForgeError::InvalidArgument(
                "Empty or whitespace-only --set-param ID".to_string(),
            ));
        }
    }
    tracing::info!(
        param_count = pairs.len(),
        "profile: {} parameter override(s) specified",
        pairs.len()
    );

    // Step 1: validate exactly one selection flag is provided; C-2 produces empty imports
    let (control_ids, mode) = match (include, exclude) {
        (Some(inc), None) => (parse_control_ids(inc)?, SelectionMode::Include),
        (None, Some(exc)) => (parse_control_ids(exc)?, SelectionMode::Exclude),
        (None, None) => {
            if set_params.is_empty() {
                return Err(ForgeError::InvalidArgument(
                    "Either --include or --exclude must be provided".to_string(),
                ));
            }
            // C-2: --set-param without --include/--exclude → warn, continue with no imports (exit 0)
            eprintln!(
                "warning: --set-param specified without --include or --exclude; the Profile will have no control imports"
            );
            tracing::warn!(
                "--set-param specified without --include or --exclude; Profile will have no control imports"
            );
            (vec![], SelectionMode::Include)
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

    // Step 3: build profile with param overrides
    let catalog_str = catalog.to_string_lossy();
    let control_count = control_ids.len();
    // Step 4: call build_profile with param overrides
    let oscal_profile = build_profile(&catalog_str, control_ids, mode, &pairs)?;

    info!(
        catalog = %catalog.display(),
        selected_controls = control_count,
        "Profile generation complete"
    );

    // Step 6: wrap and serialize
    let root = ProfileRoot { profile: oscal_profile };
    let serialized = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&root).map_err(|e| {
            ForgeError::Serialization(format!("Profile JSON serialization failed: {e}"))
        })?,
        OutputFormat::Xml => {
            crate::export::xml_serializer::serialize_profile_to_xml(&root.profile)?
        }
        OutputFormat::Yaml => crate::export::yaml::serialize_to_yaml(&root)?,
    };

    // Step 7: write to file or stdout
    match output {
        Some(path) => std::fs::write(path, serialized)?,
        None => println!("{serialized}"),
    }

    Ok(())
}

/// Parse a flat `[id, value, id, value, ...]` slice into `(id, value)` pairs.
///
/// Returns `Err(ForgeError::InvalidArgument)` if the slice has an odd length
/// (i.e., an unpaired value). In practice clap's `num_args = 2` prevents this,
/// but the check is retained as a defensive invariant.
fn parse_set_param_pairs(set_params: &[String]) -> Result<Vec<(String, String)>, ForgeError> {
    if !set_params.len().is_multiple_of(2) {
        return Err(ForgeError::InvalidArgument(format!(
            "--set-param requires ID VALUE pairs but received an odd number of arguments ({})",
            set_params.len()
        )));
    }
    Ok(set_params.chunks_exact(2).map(|c| (c[0].clone(), c[1].clone())).collect())
}

//! CLI contract for WI-30: `forge profile` subcommand.
//!
//! Shows the exact clap argument definitions and dispatch signature.
//! Location in codebase: src/cli/mod.rs (Commands enum) + src/cli/profile.rs

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Commands variant (to be added to existing Commands enum in src/cli/mod.rs)
// ---------------------------------------------------------------------------

// In src/cli/mod.rs, add to Commands enum:
//
// /// Generate an OSCAL Profile by selecting controls from a source Catalog
// Profile {
//     /// Path to the source Catalog file (OSCAL Catalog JSON)
//     #[arg(long)]
//     catalog: PathBuf,
//
//     /// Comma-separated control IDs to include (mutually exclusive with --exclude)
//     #[arg(long, conflicts_with = "exclude")]
//     include: Option<String>,
//
//     /// Comma-separated control IDs to exclude (mutually exclusive with --include)
//     #[arg(long, conflicts_with = "include")]
//     exclude: Option<String>,
//
//     /// Output format (currently only 'json' is supported)
//     #[arg(long, default_value = "json")]
//     format: OutputFormat,
//
//     /// Write output to a file instead of stdout
//     #[arg(long)]
//     output: Option<PathBuf>,
// },

// ---------------------------------------------------------------------------
// Dispatch addition (to be added to execute() in src/cli/mod.rs)
// ---------------------------------------------------------------------------

// In execute() match arm:
//
// Commands::Profile { catalog, include, exclude, format, output } => {
//     profile::execute(catalog, include.as_deref(), exclude.as_deref(), format, output.as_deref())
// }

// ---------------------------------------------------------------------------
// profile::execute — handler in src/cli/profile.rs
// ---------------------------------------------------------------------------

use crate::cli::OutputFormat;
use crate::error::ForgeError;

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
/// * `ForgeError::Io` — catalog path does not exist, or output file write failure.
/// * `ForgeError` — from `build_profile` or serialization.
///
/// # Behavior
///
/// 1. Validate that exactly one of `include` or `exclude` is `Some`.
///    (clap `conflicts_with` prevents both; this handles neither.)
/// 2. Check that `catalog` path exists (S-3).
/// 3. Parse control IDs via `parse_control_ids`.
/// 4. Determine `SelectionMode` from which flag was provided.
/// 5. Call `build_profile(catalog_str, ids, mode)`.
/// 6. Wrap in `ProfileRoot` and serialize with `serde_json::to_string_pretty`.
/// 7. Write to `output` path or stdout.
pub fn execute(
    catalog: &PathBuf,
    include: Option<&str>,
    exclude: Option<&str>,
    format: &OutputFormat,
    output: Option<&std::path::Path>,
) -> Result<(), ForgeError>;

//! Interface contract for WI-30: Profile Generation.
//!
//! This file defines the public API for Profile generation. Implementations
//! MUST match these signatures and serde attributes exactly.
//!
//! Location in codebase: src/oscal/profile.rs

use serde::Serialize;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::DocumentMetadata;
use crate::oscal::metadata::OscalMetadata;

// ---------------------------------------------------------------------------
// Root wrapper (produces {"profile": {...}} at JSON root)
// ---------------------------------------------------------------------------

/// Root wrapper for OSCAL Profile JSON: `{"profile": {...}}`.
///
/// Serializes with the OSCAL-required `"profile"` root key.
#[derive(Debug, Serialize)]
pub struct ProfileRoot {
    pub profile: OscalProfile,
}

// ---------------------------------------------------------------------------
// OSCAL Profile model
// ---------------------------------------------------------------------------

/// OSCAL Profile model. Contains metadata and one or more import entries.
///
/// WI-30 produces profiles with exactly one `ProfileImport`.
/// WI-31 will add `modify: Option<ProfileModify>`.
#[derive(Debug, Serialize)]
pub struct OscalProfile {
    /// UUID v4 — unique per generation.
    pub uuid: Uuid,

    /// OSCAL metadata (title, last-modified, version, oscal-version).
    pub metadata: OscalMetadata,

    /// Import entries: which catalog(s) to draw controls from.
    pub imports: Vec<ProfileImport>,
}

// ---------------------------------------------------------------------------
// ProfileImport
// ---------------------------------------------------------------------------

/// A single entry in the Profile's `imports[]` array.
///
/// Exactly one of `include_controls` or `exclude_controls` is `Some`.
/// The other must be `None`. Serializes with OSCAL hyphenated field names.
#[derive(Debug, Serialize)]
pub struct ProfileImport {
    /// URI reference to the source Catalog (stored as-is from `--catalog`).
    pub href: String,

    /// Controls to include (from `--include`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "include-controls")]
    pub include_controls: Option<Vec<ControlSelection>>,

    /// Controls to exclude (from `--exclude`).
    #[serde(skip_serializing_if = "Option::is_none", rename = "exclude-controls")]
    pub exclude_controls: Option<Vec<ControlSelection>>,
}

// ---------------------------------------------------------------------------
// ControlSelection
// ---------------------------------------------------------------------------

/// A control selection with a list of control IDs.
///
/// Used for both `include-controls` and `exclude-controls`.
#[derive(Debug, Serialize)]
pub struct ControlSelection {
    /// Control identifiers to include or exclude.
    #[serde(rename = "with-ids")]
    pub with_ids: Vec<String>,
}

// ---------------------------------------------------------------------------
// SelectionMode
// ---------------------------------------------------------------------------

/// Whether the Profile includes or excludes the specified controls.
///
/// `Include` → populates `include_controls`.
/// `Exclude` → populates `exclude_controls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Include,
    Exclude,
}

// ---------------------------------------------------------------------------
// build_profile — primary construction function
// ---------------------------------------------------------------------------

/// Build an `OscalProfile` from CLI arguments.
///
/// # Arguments
///
/// * `catalog_path` — Path to the source Catalog, stored as-is in `imports[0].href`.
/// * `control_ids` — Trimmed, deduplicated control IDs from `--include` or `--exclude`.
/// * `mode` — Whether the IDs represent included or excluded controls.
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — if `control_ids` is empty.
/// * `ForgeError` from `assemble_metadata` (currently infallible, but API is `Result`).
///
/// # Guardrails
///
/// * Does NOT read or parse the source Catalog file.
/// * Does NOT generate a `modify` section (WI-31 scope).
/// * Does NOT perform Profile Resolution (NIST oscal-cli concern).
pub fn build_profile(
    catalog_path: &str,
    control_ids: Vec<String>,
    mode: SelectionMode,
) -> Result<OscalProfile, ForgeError>;

// ---------------------------------------------------------------------------
// parse_control_ids — ID list parsing helper
// ---------------------------------------------------------------------------

/// Parse a comma-separated control ID string into a trimmed, deduplicated Vec.
///
/// # Arguments
///
/// * `raw` — Comma-separated string from `--include` or `--exclude` CLI flag.
///
/// # Returns
///
/// A `Vec<String>` with:
/// - Whitespace trimmed from each ID
/// - Empty strings removed (if any remain after trim)
/// - Duplicate IDs removed (first occurrence preserved, order maintained)
///
/// # Errors
///
/// * `ForgeError::InvalidArgument` — if the resulting Vec is empty.
pub fn parse_control_ids(raw: &str) -> Result<Vec<String>, ForgeError>;

//! Interface Contract: src/diff module (WI-43)
//!
//! This file defines the public API surface for the diff engine.
//! It is a design artifact — not compiled directly, but matches the
//! implementation in src/diff/*.
//!
//! Reference: specs/043-diff-report/data-model.md

use std::path::Path;

// ─── Types (src/diff/types.rs) ───────────────────────────────────────────────

/// Detected OSCAL artifact type from the root JSON key.
pub enum ArtifactType {
    Catalog,
    ComponentDefinition,
}

/// Snapshot of a control's diffable fields captured during extraction.
/// Internal type — not exposed in DiffReport, only used during comparison.
pub struct ControlSnapshot {
    pub control_id: String,
    /// UUID of this control. Empty string for FORGE Catalog outputs
    /// (uuid is skip_serializing on OscalControl). Populated for Component
    /// Definition implemented-requirements.
    pub uuid: String,
    /// Present for Catalog controls (control["title"]).
    /// None for Component Definition implemented-requirements (no title field).
    pub title: Option<String>,
    /// Implementation narrative for Component Definition implemented-requirements
    /// (ir["description"]). None for Catalog controls (which use parts_prose instead).
    /// FieldChange.field_name = "description" when this field differs.
    pub description: Option<String>,
    /// Statement prose for Catalog controls (from parts[*].prose where name=="statement").
    /// Empty for Component Definition (which uses description instead).
    /// FieldChange.field_name = "statement[N]" for index N when this differs.
    pub parts_prose: Vec<String>,
}

/// A single field-level difference within a changed control.
pub struct FieldChange {
    /// Human-readable field name shown in the report (e.g., "title", "statement[0]", "description").
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

/// A categorized comparison result for one control-id.
///
/// # Classification Rules
///
/// Given old_snap and new_snap for the same control_id:
/// - uuid differs AND fields differ → Changed { uuid_changed: true, field_changes: [...] }
/// - uuid differs AND fields same  → UuidChanged { ... }
/// - uuid same AND fields differ   → Changed { uuid_changed: false, field_changes: [...] }
/// - uuid same AND fields same     → Unchanged (not stored)
pub enum DiffEntry {
    Added {
        control_id: String,
        new_uuid: String,
    },
    Removed {
        control_id: String,
        old_uuid: String,
    },
    Changed {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
        /// True when the UUID also changed (co-occurring with field changes).
        /// The summary uuid_changes counter does NOT include these entries.
        uuid_changed: bool,
        field_changes: Vec<FieldChange>,
    },
    /// UUID changed but all diffable field values are identical.
    /// Increments summary.uuid_changes.
    UuidChanged {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
    },
}

/// Aggregate counts for the diff summary header.
pub struct DiffSummary {
    pub total_old: usize,
    pub total_new: usize,
    pub added: usize,
    pub removed: usize,
    /// Count of Changed entries (uuid_changed true or false).
    pub changed: usize,
    pub unchanged: usize,
    /// Count of standalone UuidChanged entries only (not Changed{uuid_changed:true}).
    pub uuid_changes: usize,
}

/// Complete result of comparing two OSCAL artifacts.
/// Entries are sorted by control_id (ascending, lexicographic).
pub struct DiffReport {
    pub old_file: String,
    pub new_file: String,
    pub artifact_type: ArtifactType,
    pub entries: Vec<DiffEntry>,
    pub summary: DiffSummary,
}

// ─── Public API (src/diff/mod.rs) ────────────────────────────────────────────

/// Compare two OSCAL JSON artifacts and produce a diff report.
///
/// # Steps
/// 1. Validate both files exist and are readable.
/// 2. Parse both as JSON (serde_json::Value).
/// 3. Detect artifact type from root key ("catalog" or "component-definition").
/// 4. Validate both files are the same artifact type.
/// 5. Extract controls from each file into HashMap<String, ControlSnapshot>.
/// 6. Compare the two HashMaps (added/removed/matched).
/// 7. For matched controls: compare fields + UUIDs → Changed or UuidChanged.
/// 8. Build DiffSummary from entries.
/// 9. Sort entries by control_id.
/// 10. Return DiffReport.
///
/// # Errors
///
/// Returns `ForgeError::DiffError(msg)` for:
/// - File not found or unreadable
/// - Invalid JSON (not parseable)
/// - Not a recognized OSCAL artifact (missing root key)
/// - Mismatched artifact types (one Catalog, one Component Definition)
pub fn diff_artifacts(
    old_path: &Path,
    new_path: &Path,
) -> Result<DiffReport, crate::error::ForgeError>;

/// Format a DiffReport as human-readable text for stdout.
///
/// Output format:
/// ```text
/// OSCAL Diff Report
/// =================
/// Old: <old_file>  (Catalog|ComponentDefinition)
/// New: <new_file>  (Catalog|ComponentDefinition)
///
/// Summary
/// -------
/// Controls (old): N  |  Controls (new): M
/// Added: A  |  Removed: R  |  Changed: C  |  Unchanged: U  |  UUID changes: X
///
/// [if no changes: "No differences found."]
///
/// [if changes:]
/// Added (N)
/// ─────────
///   + <control_id>  [uuid: <new_uuid>]
///
/// Changed (N)
/// ───────────
///   ~ <control_id>
///       <field_name>: "<old_value>"  →  "<new_value>"
///       [UUID: <old_uuid> → <new_uuid>]  ← only if uuid_changed
///
/// Removed (N)
/// ───────────
///   - <control_id>  [uuid: <old_uuid>]
///
/// UUID Stability Changes (N)
/// ──────────────────────────
///   ! <control_id>  <old_uuid>  →  <new_uuid>
/// ```
///
/// Empty sections display "(none)" rather than being omitted, for clarity.
pub fn format_diff_report(report: &DiffReport) -> String;

// ─── CLI Handler (src/cli/diff.rs) ───────────────────────────────────────────

/// Execute the `forge diff` subcommand.
///
/// Calls `diff_artifacts`, formats the report, prints to stdout, and
/// returns `Ok(true)` if differences were found, `Ok(false)` if none.
///
/// # Errors
///
/// Returns `ForgeError::DiffError` for any error condition.
/// The caller (cli/mod.rs dispatch) converts `Ok(true)` to `Err(ForgeError::DiffHasChanges)`
/// to signal exit code 1 per diff(1) convention.
pub fn execute(
    old_path: &std::path::Path,
    new_path: &std::path::Path,
) -> Result<bool, crate::error::ForgeError>;

// ─── Error Additions (src/error.rs) ──────────────────────────────────────────

// New variants to add to ForgeError:
//
// /// Diff completed but differences were found.
// /// Silent sentinel: main.rs exits with code 1 without printing "Error: ...".
// #[error("")]
// DiffHasChanges,
//
// /// Diff failed due to invalid input, type mismatch, or comparison error.
// /// Maps to exit code 2 per diff(1) convention.
// #[error("Diff error: {0}")]
// DiffError(String),

// ─── Extractor (src/diff/extractor.rs) ───────────────────────────────────────

/// Extract controls from an OSCAL JSON value into a HashMap keyed by control_id.
///
/// For Catalog: recursively traverses groups[].controls[] at all depths.
/// For ComponentDefinition: traverses components[].control-implementations[].implemented-requirements[].
///
/// Returns a HashMap directly (never errors — malformed structures yield empty maps).
fn extract_controls(
    json: &serde_json::Value,
    artifact_type: &ArtifactType,
) -> std::collections::HashMap<String, ControlSnapshot>;

// ─── Engine (src/diff/engine.rs) ─────────────────────────────────────────────

/// Compare two control HashMaps and produce a sorted Vec<DiffEntry>.
///
/// Classification:
/// - control_id only in new_map → Added
/// - control_id only in old_map → Removed
/// - control_id in both:
///   - Compare uuid, title, description, parts_prose
///   - uuid differs AND fields differ → Changed { uuid_changed: true, ... }
///   - uuid differs AND fields same  → UuidChanged
///   - uuid same AND fields differ   → Changed { uuid_changed: false, ... }
///   - uuid same AND fields same     → Unchanged (not included in entries)
///
/// FieldChange.field_name values: "title" | "description" | "statement[N]"
///
/// Output is sorted by control_id (ascending).
fn compare_controls(
    old_map: &std::collections::HashMap<String, ControlSnapshot>,
    new_map: &std::collections::HashMap<String, ControlSnapshot>,
) -> Vec<DiffEntry>;

/// Build DiffSummary from a Vec<DiffEntry> and the original control counts.
fn build_summary(
    entries: &[DiffEntry],
    total_old: usize,
    total_new: usize,
) -> DiffSummary;

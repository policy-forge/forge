//! Deterministic, read-only comparison of two source-policy revisions.

mod engine;
mod formatter;
mod inventory;
mod types;

use std::path::Path;

pub use formatter::{format_json, format_text};
pub use types::{
    ApprovalStatus, Classification, ConfidenceBasis, EvidenceCode, InputFormat,
    InventoryRequirement, LocationBasis, MIGRATION_REPORT_SCHEMA_VERSION, MigrationEntry,
    MigrationOutcomeCounts, MigrationReport, MigrationSummary, RequirementLocation,
    SourceProvenance,
};

use crate::error::ForgeError;

/// Analyze two policy files through the shared conversion preparation pipeline.
///
/// The function reads but never writes either policy. All analysis failures are
/// normalized to [`ForgeError::MigrationError`] so CLI callers can honor the
/// migration command's documented exit-2 contract.
///
/// # Errors
///
/// Returns [`ForgeError::MigrationError`] when either policy cannot be fully
/// inventoried or when classification invariants fail.
pub fn analyze_paths(
    old_path: &Path,
    new_path: &Path,
    max_size_bytes: u64,
) -> Result<MigrationReport, ForgeError> {
    let old = inventory::build_inventory(old_path, max_size_bytes)?;
    let new = inventory::build_inventory(new_path, max_size_bytes)?;
    engine::classify(old, new)
}

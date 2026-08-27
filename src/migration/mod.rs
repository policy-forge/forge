//! Deterministic, read-only comparison of two source-policy revisions.

mod engine;
mod formatter;
mod inventory;
pub mod successor;
mod types;

use std::path::Path;

pub use formatter::{format_json, format_text};
pub use types::{
    ApprovalStatus, Classification, ConfidenceBasis, DeclarationEvidence, EvidenceCode,
    InputFormat, InventoryRequirement, LocationBasis, MIGRATION_REPORT_SCHEMA_VERSION,
    MigrationEntry, MigrationOutcomeCounts, MigrationReport, MigrationSummary, RequirementLocation,
    SourceProvenance,
};

use crate::error::ForgeError;

/// Analyze two policy revisions and optionally apply a reviewer successor map.
///
/// # Errors
///
/// Returns [`ForgeError::MigrationError`] for either inventory/load failure,
/// successor-map invalidity, or an incomplete classification result.
pub fn analyze_paths(
    old_path: &Path,
    new_path: &Path,
    successor_map_path: Option<&Path>,
    max_size_bytes: u64,
) -> Result<MigrationReport, ForgeError> {
    let old =
        inventory::build_inventory(old_path, max_size_bytes).map_err(normalize_to_migration)?;
    let new =
        inventory::build_inventory(new_path, max_size_bytes).map_err(normalize_to_migration)?;
    let successor_map =
        successor_map_path.map(successor::load).transpose().map_err(normalize_to_migration)?;
    engine::classify(old, new, successor_map.as_ref()).map_err(normalize_to_migration)
}

fn normalize_to_migration(error: ForgeError) -> ForgeError {
    match error {
        error @ ForgeError::MigrationError(_) => error,
        error => ForgeError::MigrationError(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_boundary_preserves_or_normalizes_errors() {
        let local = ForgeError::MigrationError("local defect".to_string());
        assert!(matches!(
            normalize_to_migration(local),
            ForgeError::MigrationError(message) if message == "local defect"
        ));

        let normalized =
            normalize_to_migration(ForgeError::Validation("invalid input".to_string()));
        assert!(matches!(
            normalized,
            ForgeError::MigrationError(message) if message.contains("Validation error: invalid input")
        ));
    }
}

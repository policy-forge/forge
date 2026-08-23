//! Content-safe CLI output for canonical generated-artifact drift checks.

use std::path::Path;

use serde::Serialize;

use crate::cli::DriftOutputFormat;
use crate::diff::{
    DRIFT_COMPARISON_CONTRACT_VERSION, DriftComparison, compare_artifacts_for_drift,
};
use crate::error::ForgeError;

#[derive(Serialize)]
struct JsonOutput<'a> {
    status: &'a str,
    artifact_type: &'a str,
    comparison_contract: u8,
}

/// Compare committed and generated artifacts and emit content-free status.
///
/// # Errors
///
/// Returns [`ForgeError::DiffError`] for file, JSON, or model errors and
/// [`ForgeError::Serialization`] if machine output cannot be serialized.
pub fn execute(
    committed_path: &Path,
    generated_path: &Path,
    format: &DriftOutputFormat,
) -> Result<bool, ForgeError> {
    let comparison = compare_artifacts_for_drift(committed_path, generated_path)?;
    let output = match format {
        DriftOutputFormat::Text => format_text(&comparison),
        DriftOutputFormat::Json => format_json(&comparison)?,
    };
    crate::cli::output::write_output(&output, None)?;
    Ok(comparison.has_drift())
}

fn artifact_type_name(comparison: &DriftComparison) -> &'static str {
    match comparison.artifact_type {
        crate::diff::ArtifactType::Catalog => "catalog",
        crate::diff::ArtifactType::ComponentDefinition => "component-definition",
    }
}

fn format_text(comparison: &DriftComparison) -> String {
    format!(
        "status: {}\nartifact-type: {}\ncomparison-contract: {}\n",
        comparison.status.as_str(),
        artifact_type_name(comparison),
        DRIFT_COMPARISON_CONTRACT_VERSION
    )
}

fn format_json(comparison: &DriftComparison) -> Result<String, ForgeError> {
    let output = JsonOutput {
        status: comparison.status.as_str(),
        artifact_type: artifact_type_name(comparison),
        comparison_contract: DRIFT_COMPARISON_CONTRACT_VERSION,
    };
    let mut rendered = serde_json::to_string(&output)
        .map_err(|error| ForgeError::Serialization(error.to_string()))?;
    rendered.push('\n');
    Ok(rendered)
}

//! Read-only local MCP transport with explicit disclosure provenance and current originals.

mod artifact_status;

use std::path::Path;

use crate::ForgeError;
use artifact_status::stdio::{self, RunError, Startup};

/// Serve the selected local project through one stdio owner with fixed safe runtime errors.
/// Missing or partial decision declarations retain static discovery and closed unavailability.
pub(crate) fn serve(
    project: &Path,
    decision_root: Option<&Path>,
    decision_sha256: Option<&str>,
    profile_sha256: Option<&str>,
) -> Result<(), ForgeError> {
    let startup = Startup {
        project_root: project.to_path_buf(),
        decision_root: decision_root.map(Path::to_path_buf),
        decision_sha256: decision_sha256.map(str::to_owned),
        profile_sha256: profile_sha256.unwrap_or_default().to_owned(),
    };
    stdio::run_stdio(&startup).map_err(|error| {
        let message = match error {
            RunError::Catalog => "MCP shipped tool schemas unavailable",
            RunError::Input => "MCP input failed",
            RunError::Output => "MCP output failed; written bytes cannot be retracted",
            RunError::Encoding => "MCP complete response unavailable",
            RunError::DuplicateId => "MCP repeated accepted request ID; transport retired",
            #[cfg(not(any(unix, windows)))]
            RunError::UnsupportedPlatform => "MCP platform input profile unavailable",
        };
        ForgeError::InvalidArgument(message.to_owned())
    })
}

/// Check actual complete offline inputs through the retained original native owner.
/// No index is constructed and the terminal error contains no captured private facts.
pub(crate) fn check_index_inputs(
    project: &Path,
    intent_root: &Path,
    intent_sha256: &str,
    profile_sha256: &str,
) -> Result<(), ForgeError> {
    artifact_status::index_build::check_index_inputs(
        project,
        intent_root,
        intent_sha256,
        profile_sha256,
    )
    .map_err(|_| ForgeError::InvalidArgument("MCP index input check unavailable".to_owned()))
}

/// Explicit declaration selection preserves the complete default /1 path.
pub(crate) fn serve_selected(
    project: &Path,
    decision_root: Option<&Path>,
    decision_sha256: Option<&str>,
    profile_sha256: Option<&str>,
    family: crate::cli::McpDeclarationFamily,
) -> Result<(), ForgeError> {
    if family == crate::cli::McpDeclarationFamily::V1 {
        return serve(project, decision_root, decision_sha256, profile_sha256);
    }
    let startup = Startup {
        project_root: project.to_path_buf(),
        decision_root: decision_root.map(Path::to_path_buf),
        decision_sha256: decision_sha256.map(str::to_owned),
        profile_sha256: profile_sha256.unwrap_or_default().to_owned(),
    };
    stdio::run_stdio_selected(&startup, family).map_err(|error| {
        let message = match error {
            RunError::Catalog => "MCP shipped tool schemas unavailable",
            RunError::Input => "MCP input failed",
            RunError::Output => "MCP output failed; written bytes cannot be retracted",
            RunError::Encoding => "MCP complete response unavailable",
            RunError::DuplicateId => "MCP repeated accepted request ID; transport retired",
            #[cfg(not(any(unix, windows)))]
            RunError::UnsupportedPlatform => "MCP platform input profile unavailable",
        };
        ForgeError::InvalidArgument(message.to_owned())
    })
}

//! Traceability: map OSCAL output fields back to source Markdown lines.
//!
//! Produces [`TraceLink`](crate::model::trace::TraceLink) entries for every generated
//! OSCAL control, parameter, and citation, enabling bidirectional drill-down
//! from compliance artifact to policy source.

/// Extract trace links from a parsed policy document.
pub mod extractor;
/// Format trace reports for human-readable output.
pub mod formatter;
/// Trace report types: structured trace output for a conversion run.
pub mod report;
/// Resolve trace links against source files.
pub mod resolver;
/// Walk the OSCAL output tree and collect trace references.
pub mod walker;

use std::path::Path;

use crate::error::ForgeError;
use crate::types::OscalModelType;
use report::{TraceReport, TraceSummary};

/// Generate a complete traceability report from an OSCAL artifact and source policy.
///
/// 1. Reads and parses artifact JSON
/// 2. Detects artifact type
/// 3. Walks elements and extracts trace metadata
/// 4. Computes summary statistics
/// 5. Checks source staleness
///
/// # Errors
///
/// - `ForgeError::FileNotFound` if artifact or source file doesn't exist
/// - `ForgeError::Parse` if artifact is invalid JSON
/// - `ForgeError::TraceUnsupportedArtifact` if artifact type is unrecognized
pub fn generate_trace_report(
    artifact_path: &Path,
    source_path: &Path,
) -> Result<TraceReport, ForgeError> {
    // Read and parse artifact JSON (handles FileNotFound via Io mapping)
    let artifact_content = read_file(artifact_path)?;
    let json: serde_json::Value = serde_json::from_str(&artifact_content)
        .map_err(|e| ForgeError::Parse(format!("Invalid JSON in artifact: {e}")))?;

    // Read source file for line count validation
    let source_content = read_file(source_path)?;
    let source_line_count = source_content.lines().count();

    // Detect artifact type and walk elements
    let art_type = walker::detect_artifact_type(&json)?;
    let entries = match art_type {
        OscalModelType::Catalog => {
            let catalog = &json["catalog"];
            walker::walk_catalog_elements(catalog)
        }
        OscalModelType::ComponentDefinition => {
            let compdef = &json["component-definition"];
            walker::walk_compdef_elements(compdef)
        }
        OscalModelType::Profile => {
            return Err(ForgeError::TraceUnsupportedArtifact {
                detail: "Profile artifacts are not supported for traceability".to_string(),
            });
        }
        OscalModelType::Mapping => {
            return Err(ForgeError::TraceUnsupportedArtifact {
                detail: "Control Mapping artifacts are not supported for traceability".to_string(),
            });
        }
    };

    // Compute summary
    let summary = TraceSummary::from_entries(&entries);

    // Check source staleness
    let metadata_last_modified = json
        .get(art_type.as_str())
        .and_then(|v| v.get("metadata"))
        .and_then(|m| m.get("last-modified"))
        .and_then(|v| v.as_str());
    let source_stale = resolver::check_source_staleness(source_path, metadata_last_modified);

    Ok(TraceReport {
        artifact_path: artifact_path.to_path_buf(),
        source_path: source_path.to_path_buf(),
        artifact_type: art_type,
        entries,
        summary,
        source_stale,
        source_line_count,
    })
}

/// Read a file, mapping `NotFound` to `ForgeError::FileNotFound`,
/// `PermissionDenied` to `ForgeError::PermissionDenied`, and other I/O errors to `ForgeError::Io`.
fn read_file(path: &Path) -> Result<String, ForgeError> {
    // Guard against oversized files; ignore NotFound so read_to_string maps it.
    match crate::io::check_file_size(path, crate::io::MAX_FILE_SIZE) {
        Ok(_) => {}
        Err(ForgeError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    std::fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: path.to_path_buf() }
        }
        _ => ForgeError::Io(e),
    })
}

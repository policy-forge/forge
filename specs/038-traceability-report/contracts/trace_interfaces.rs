// WI-38: Traceability Report — Interface Contracts
//
// These interfaces define the public API for the trace module.
// Implementation must match these signatures exactly.

use std::path::{Path, PathBuf};

use crate::error::ForgeError;
use crate::oscal::trace_embedding::{FORGE_TRACE_NS, PROP_SOURCE_FILE, PROP_SOURCE_LINE, PROP_SOURCE_SECTION};

// ─── Data Structures ────────────────────────────────────────────────────

/// Trace metadata extracted from an OSCAL element's WI-17 trace props.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceMetadata {
    /// Source file name (from `source-file` prop).
    pub source_file: String,
    /// Source section title (from `source-section` prop).
    pub source_section: String,
    /// 1-based source line number (from `source-line` prop, parsed).
    pub source_line: usize,
}

/// A single entry in the traceability report.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// OSCAL element identifier (control-id or group-id).
    pub element_id: String,
    /// Type of OSCAL element: "group", "control", or "implemented-requirement".
    pub element_type: String,
    /// Resolved trace metadata. `None` if element has no WI-17 trace props.
    pub trace: Option<TraceMetadata>,
}

/// Aggregate statistics for the traceability report.
#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub total_elements: usize,
    pub mapped_elements: usize,
    pub unmapped_elements: usize,
    /// Coverage percentage (0.0–100.0). 0.0 if total_elements == 0.
    pub coverage_percent: f64,
}

/// Full traceability report.
#[derive(Debug)]
pub struct TraceReport {
    /// Path to the OSCAL artifact file.
    pub artifact_path: PathBuf,
    /// Path to the source policy file.
    pub source_path: PathBuf,
    /// Detected artifact type: "catalog" or "component-definition".
    pub artifact_type: String,
    /// All trace entries (one per walked OSCAL element).
    pub entries: Vec<TraceEntry>,
    /// Computed summary statistics.
    pub summary: TraceSummary,
    /// True if source file mtime > OSCAL metadata.last-modified.
    pub source_stale: bool,
}

// ─── Extractor ──────────────────────────────────────────────────────────

/// Extract trace metadata from an OSCAL element's `props` array (serde_json::Value).
///
/// Scans `element["props"]` for props with `ns == FORGE_TRACE_NS`.
/// Returns `Some(TraceMetadata)` if at least `source-section` is found.
/// Returns `None` if no trace props exist.
///
/// For groups: may return TraceMetadata with `source_line == 0` (no line prop).
pub fn extract_trace_metadata(element: &serde_json::Value) -> Option<TraceMetadata>;

// ─── Walker ─────────────────────────────────────────────────────────────

/// Detected OSCAL artifact type.
pub enum ArtifactType {
    Catalog,
    ComponentDefinition,
}

/// Detect whether a parsed JSON value is a Catalog or Component Definition.
///
/// Returns `Err(ForgeError)` if neither top-level key is found.
pub fn detect_artifact_type(json: &serde_json::Value) -> Result<ArtifactType, ForgeError>;

/// Walk a Catalog's groups and controls, extracting trace entries.
///
/// Yields: groups (element_type "group") then controls (element_type "control")
/// within each group. Parts are excluded.
pub fn walk_catalog_elements(catalog: &serde_json::Value) -> Vec<TraceEntry>;

/// Walk a Component Definition's components → control-implementations →
/// implemented-requirements, extracting trace entries.
///
/// Yields: implemented-requirements (element_type "implemented-requirement").
/// Components themselves are not yielded.
pub fn walk_compdef_elements(compdef: &serde_json::Value) -> Vec<TraceEntry>;

// ─── Resolver ───────────────────────────────────────────────────────────

/// Check whether the source file has been modified after the OSCAL artifact
/// was generated, by comparing source file mtime against the OSCAL
/// `metadata.last-modified` ISO 8601 timestamp.
///
/// Returns `true` if source appears newer (stale), `false` otherwise.
/// Returns `false` if either timestamp cannot be determined (graceful fallback).
pub fn check_source_staleness(
    source_path: &Path,
    metadata_last_modified: Option<&str>,
) -> bool;

/// Validate that a source line number is within the actual file's line count.
///
/// Returns `true` if `line_number <= total_lines`, `false` otherwise.
pub fn validate_line_reference(line_number: usize, source_line_count: usize) -> bool;

// ─── Formatter ──────────────────────────────────────────────────────────

/// Format a TraceReport as a column-aligned text table.
///
/// Columns: OSCAL Element ID, Element Type, Source Section, Source Line
/// Includes header, separator, data rows, and summary footer.
/// Unmapped elements show "[unmapped]" in source columns.
/// Groups with section but no line show "—" for Source Line.
///
/// All source-derived strings have control characters stripped (SEC-5).
pub fn format_trace_table(report: &TraceReport) -> String;

/// Strip ASCII control characters (0x00-0x1F, excluding 0x0A and 0x09)
/// from a string. Used to prevent terminal escape injection (SEC-5).
pub fn strip_control_chars(s: &str) -> String;

// ─── Report Builder (Orchestrator) ──────────────────────────────────────

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
) -> Result<TraceReport, ForgeError>;

// ─── CLI Handler ────────────────────────────────────────────────────────

/// Execute the `forge trace` subcommand.
///
/// Validates inputs, generates report, formats table, outputs to stdout or file.
///
/// # Errors
///
/// Returns `ForgeError` for all failure modes documented in the spec.
pub fn execute(
    artifact: &Path,
    source: &Path,
    output: Option<&Path>,
) -> Result<(), ForgeError>;

// ─── Error Variant (added to ForgeError) ────────────────────────────────

// Add to src/error.rs:
//
// #[error("Unsupported OSCAL artifact type for tracing: {detail}")]
// TraceUnsupportedArtifact { detail: String },
//
// Map to exit code 2 (parse/structure errors).

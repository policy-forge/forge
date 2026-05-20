//! Batch conversion: convert multiple Markdown policy files to OSCAL in parallel.
//!
//! Coordinates pipeline execution across multiple input files, collects per-file
//! results, and produces an aggregated `BatchSummary`.

/// Format a batch summary for human-readable output.
pub mod formatter;
/// Orchestrate parallel batch conversion across multiple policy files.
pub mod orchestrator;
/// Generate deterministic output filenames for batch artifacts.
pub mod output_naming;
/// Batch conversion result types: `FileOutcome`, `FileResult`, and `BatchSummary`.
pub mod summary;

pub use formatter::format_batch_summary;
pub use summary::{BatchSummary, FileOutcome, FileResult};

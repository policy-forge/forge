// Batch Conversion Interface Contracts
// These define the public API surface for the batch module.
// Implementation MUST match these signatures.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cli::{OutputFormat, Strategy};
use crate::ForgeError;

// --- Data Structures (batch/summary.rs) ---

/// Result of converting a single file in a batch.
#[derive(Debug)]
pub struct FileResult {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration: Duration,
}

impl FileResult {
    /// Create a successful result.
    pub fn success(input_path: PathBuf, output_path: PathBuf, duration: Duration) -> Self;

    /// Create a failed result.
    pub fn failure(input_path: PathBuf, error_message: String, duration: Duration) -> Self;
}

/// Aggregated summary of a batch conversion run.
#[derive(Debug)]
pub struct BatchSummary {
    pub total_files: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub total_duration: Duration,
    pub results: Vec<FileResult>,
}

impl BatchSummary {
    /// Build summary from a list of results and total duration.
    /// Results are sorted by input filename.
    pub fn from_results(results: Vec<FileResult>, total_duration: Duration) -> Self;

    /// Returns true if any file in the batch failed.
    pub fn has_failures(&self) -> bool;
}

// --- Input Validation (batch/orchestrator.rs) ---

/// Validate that all input files exist and are readable.
///
/// Returns Ok(()) if all valid, or Err listing all invalid paths with reasons.
/// This is a fail-fast check run before any processing begins.
pub fn validate_inputs(input_paths: &[PathBuf]) -> Result<(), ForgeError>;

// --- Output Naming (batch/output_naming.rs) ---

/// Derive output file paths for all inputs, with collision avoidance.
///
/// Rules:
/// 1. Output filename = {input_stem}.{format_extension}
/// 2. If output_dir is Some, place in that directory
/// 3. If output_dir is None, place in current directory
/// 4. For collisions (same stem from different dirs), append _{n} suffix (n starts at 2)
///
/// Returns a Vec of (input_path, output_path) pairs in the same order as input.
pub fn derive_output_paths(
    input_paths: &[PathBuf],
    format: OutputFormat,
    output_dir: Option<&Path>,
) -> Vec<(PathBuf, PathBuf)>;

// --- Batch Orchestrator (batch/orchestrator.rs) ---

/// Run batch conversion on multiple input files.
///
/// Processes files in parallel using rayon with the specified parallelism level.
/// Each file is converted independently; a failure in one does not affect others.
/// Results are sorted by input filename for deterministic display order.
///
/// Preconditions:
/// - path_pairs is non-empty and all input paths validated
/// - output paths derived via derive_output_paths
/// - jobs: 0 = auto (num_cpus), 1 = sequential, 2..=256 = explicit thread count
pub fn run_batch_conversion(
    path_pairs: &[(PathBuf, PathBuf)],
    strategy: Strategy,
    format: OutputFormat,
    max_size_bytes: u64,
    source_profile: Option<&str>,
    jobs: usize,
) -> BatchSummary;

// --- Status Formatting (batch/formatter.rs) ---

/// Format the batch summary as a human-readable string for stderr display.
///
/// Format:
/// ```text
/// Batch conversion complete: 3 files (2 succeeded, 1 failed) in 1.23s
///
///   ✓ policy1.md → output/policy1.json (0.45s)
///   ✗ policy3.md — Parse error: ... (0.40s)
/// ```
pub fn format_batch_summary(summary: &BatchSummary) -> String;

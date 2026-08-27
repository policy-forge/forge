use std::path::PathBuf;
use std::time::Duration;

use crate::error::ForgeError;

/// Outcome of converting a single file — either success with an output path,
/// or failure retaining its typed error.
#[derive(Debug)]
pub enum FileOutcome {
    /// The file was converted successfully.
    Success {
        /// Path to the generated output artifact.
        output_path: PathBuf,
    },
    /// The file could not be converted.
    Failure {
        /// Underlying conversion error, retained for classification and source context.
        error: ForgeError,
    },
}

/// Result of converting a single file in a batch.
#[derive(Debug)]
pub struct FileResult {
    /// Path to the input Markdown file.
    pub input_path: PathBuf,
    /// The conversion outcome (success or failure).
    pub outcome: FileOutcome,
    /// Wall-clock duration of the conversion attempt.
    pub duration: Duration,
}

impl FileResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(input_path: PathBuf, output_path: PathBuf, duration: Duration) -> Self {
        Self { input_path, outcome: FileOutcome::Success { output_path }, duration }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failure(input_path: PathBuf, error: ForgeError, duration: Duration) -> Self {
        Self { input_path, outcome: FileOutcome::Failure { error }, duration }
    }

    /// Returns true if this file converted successfully.
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self.outcome, FileOutcome::Success { .. })
    }
}

/// Aggregated summary of a batch conversion run.
#[derive(Debug)]
pub struct BatchSummary {
    /// Total number of files processed.
    total_files: usize,
    /// Number of files that converted successfully.
    succeeded: usize,
    /// Number of files that failed conversion.
    failed: usize,
    /// Total wall-clock duration of the entire batch run.
    total_duration: Duration,
    /// Per-file results, sorted by raw `OsStr` filename order (full path as tie-breaker).
    results: Vec<FileResult>,
    /// Whether a requested custom pool could not be created and conversion ran sequentially.
    degraded_to_sequential: bool,
}

impl BatchSummary {
    /// Build summary from a list of results and total duration.
    /// Results are sorted by input filename, with full path as tie-breaker.
    #[must_use]
    pub fn from_results(results: Vec<FileResult>, total_duration: Duration) -> Self {
        Self::from_results_with_execution_mode(results, total_duration, false)
    }

    /// Build a summary while recording whether a custom-pool fallback occurred.
    #[must_use]
    pub(crate) fn from_results_with_execution_mode(
        mut results: Vec<FileResult>,
        total_duration: Duration,
        degraded_to_sequential: bool,
    ) -> Self {
        results.sort_by(|a, b| {
            // Raw `OsStr` ordering is deterministic for a platform but intentionally
            // case-sensitive and platform-specific.
            let a_key = a.input_path.file_name().unwrap_or_else(|| a.input_path.as_os_str());
            let b_key = b.input_path.file_name().unwrap_or_else(|| b.input_path.as_os_str());
            a_key.cmp(b_key).then_with(|| a.input_path.cmp(&b.input_path))
        });

        let total_files = results.len();
        let succeeded = results.iter().filter(|r| r.is_success()).count();
        let failed = total_files - succeeded;

        Self { total_files, succeeded, failed, total_duration, results, degraded_to_sequential }
    }

    /// Total number of processed files.
    #[must_use]
    pub const fn total_files(&self) -> usize {
        self.total_files
    }

    /// Number of successful conversions.
    #[must_use]
    pub const fn succeeded(&self) -> usize {
        self.succeeded
    }

    /// Number of failed conversions.
    #[must_use]
    pub const fn failed(&self) -> usize {
        self.failed
    }

    /// Total wall-clock duration of the batch.
    #[must_use]
    pub const fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Per-file results in deterministic filename order.
    #[must_use]
    pub fn results(&self) -> &[FileResult] {
        &self.results
    }

    /// Returns true when a requested custom pool could not be created.
    #[must_use]
    pub const fn degraded_to_sequential(&self) -> bool {
        self.degraded_to_sequential
    }

    /// Returns true if any file in the batch failed.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T005: FileResult constructor invariants

    #[test]
    fn success_result_has_output_path_and_no_error() {
        let result = FileResult::success(
            PathBuf::from("input.md"),
            PathBuf::from("output.json"),
            Duration::from_millis(100),
        );
        assert!(result.is_success());
        assert!(
            matches!(result.outcome, FileOutcome::Success { ref output_path } if output_path == &PathBuf::from("output.json"))
        );
    }

    #[test]
    fn failure_result_has_error_and_no_output_path() {
        let result = FileResult::failure(
            PathBuf::from("input.md"),
            ForgeError::Parse("parse error".to_string()),
            Duration::from_millis(50),
        );
        assert!(!result.is_success());
        assert!(
            matches!(result.outcome, FileOutcome::Failure { ref error } if matches!(error, ForgeError::Parse(message) if message == "parse error"))
        );
    }

    // T006: BatchSummary::from_results tests

    #[test]
    fn from_results_counts_correct() {
        let results = vec![
            FileResult::success(
                PathBuf::from("a.md"),
                PathBuf::from("a.json"),
                Duration::from_millis(100),
            ),
            FileResult::failure(
                PathBuf::from("b.md"),
                ForgeError::Parse("error".to_string()),
                Duration::from_millis(50),
            ),
            FileResult::success(
                PathBuf::from("c.md"),
                PathBuf::from("c.json"),
                Duration::from_millis(75),
            ),
        ];

        let summary = BatchSummary::from_results(results, Duration::from_millis(300));
        assert_eq!(summary.total_files(), 3);
        assert_eq!(summary.succeeded(), 2);
        assert_eq!(summary.failed(), 1);
        assert!(summary.has_failures());
    }

    #[test]
    fn fallback_execution_state_is_preserved() {
        let summary =
            BatchSummary::from_results_with_execution_mode(Vec::new(), Duration::ZERO, true);

        assert!(summary.degraded_to_sequential());
    }

    #[test]
    fn from_results_sorted_by_input_filename() {
        let results = vec![
            FileResult::success(
                PathBuf::from("/dir2/charlie.md"),
                PathBuf::from("charlie.json"),
                Duration::from_millis(100),
            ),
            FileResult::success(
                PathBuf::from("/dir1/alpha.md"),
                PathBuf::from("alpha.json"),
                Duration::from_millis(100),
            ),
            FileResult::success(
                PathBuf::from("/dir3/bravo.md"),
                PathBuf::from("bravo.json"),
                Duration::from_millis(100),
            ),
        ];

        let summary = BatchSummary::from_results(results, Duration::from_millis(300));
        let names: Vec<_> = summary
            .results
            .iter()
            .map(|r| r.input_path.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["alpha.md", "bravo.md", "charlie.md"]);
    }

    #[test]
    fn has_failures_false_when_all_succeed() {
        let results = vec![FileResult::success(
            PathBuf::from("a.md"),
            PathBuf::from("a.json"),
            Duration::from_millis(100),
        )];
        let summary = BatchSummary::from_results(results, Duration::from_millis(100));
        assert!(!summary.has_failures());
    }
}

use std::path::PathBuf;
use std::time::Duration;

/// Outcome of converting a single file — either success with an output path,
/// or failure with an error message.
#[derive(Debug)]
pub enum FileOutcome {
    Success { output_path: PathBuf },
    Failure { error_message: String },
}

/// Result of converting a single file in a batch.
#[derive(Debug)]
pub struct FileResult {
    pub input_path: PathBuf,
    pub outcome: FileOutcome,
    pub duration: Duration,
}

impl FileResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(input_path: PathBuf, output_path: PathBuf, duration: Duration) -> Self {
        Self {
            input_path,
            outcome: FileOutcome::Success { output_path },
            duration,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failure(input_path: PathBuf, error_message: String, duration: Duration) -> Self {
        Self {
            input_path,
            outcome: FileOutcome::Failure { error_message },
            duration,
        }
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
    pub total_files: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub total_duration: Duration,
    pub results: Vec<FileResult>,
}

impl BatchSummary {
    /// Build summary from a list of results and total duration.
    /// Results are sorted by input filename.
    #[must_use]
    pub fn from_results(mut results: Vec<FileResult>, total_duration: Duration) -> Self {
        results.sort_by(|a, b| {
            let a_name = a.input_path.file_name().unwrap_or_default();
            let b_name = b.input_path.file_name().unwrap_or_default();
            a_name.cmp(b_name)
        });

        let total_files = results.len();
        let succeeded = results.iter().filter(|r| r.is_success()).count();
        let failed = total_files - succeeded;

        Self { total_files, succeeded, failed, total_duration, results }
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
        assert!(matches!(result.outcome, FileOutcome::Success { ref output_path } if output_path == &PathBuf::from("output.json")));
    }

    #[test]
    fn failure_result_has_error_and_no_output_path() {
        let result = FileResult::failure(
            PathBuf::from("input.md"),
            "parse error".to_string(),
            Duration::from_millis(50),
        );
        assert!(!result.is_success());
        assert!(matches!(result.outcome, FileOutcome::Failure { ref error_message } if error_message == "parse error"));
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
                "error".to_string(),
                Duration::from_millis(50),
            ),
            FileResult::success(
                PathBuf::from("c.md"),
                PathBuf::from("c.json"),
                Duration::from_millis(75),
            ),
        ];

        let summary = BatchSummary::from_results(results, Duration::from_millis(300));
        assert_eq!(summary.total_files, 3);
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 1);
        assert!(summary.has_failures());
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

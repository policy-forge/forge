use std::fmt::Write;

use super::summary::{BatchSummary, FileOutcome};

/// Format the batch summary as a human-readable string for stderr display.
///
/// Format:
/// ```text
/// Batch conversion complete: 3 files (2 succeeded, 1 failed) in 1.23s
///
///   ✓ policy1.md → output/policy1.json (0.45s)
///   ✗ policy3.md — Parse error: ... (0.40s)
/// ```
#[must_use]
pub fn format_batch_summary(summary: &BatchSummary) -> String {
    let mut buf = String::new();

    let total_secs = summary.total_duration.as_secs_f64();
    let files_label = if summary.total_files == 1 { "file" } else { "files" };
    let _ = writeln!(
        buf,
        "Batch conversion complete: {} {files_label} ({} succeeded, {} failed) in {total_secs:.2}s",
        summary.total_files, summary.succeeded, summary.failed,
    );
    let _ = writeln!(buf);

    for result in &summary.results {
        let input_name = result.input_path.file_name().map_or_else(
            || result.input_path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let duration_secs = result.duration.as_secs_f64();

        match &result.outcome {
            FileOutcome::Success { output_path } => {
                let output_display = output_path.display();
                let _ = writeln!(
                    buf,
                    "  \u{2713} {input_name} \u{2192} {output_display} ({duration_secs:.2}s)"
                );
            }
            FileOutcome::Failure { error_message } => {
                let _ = writeln!(
                    buf,
                    "  \u{2717} {input_name} \u{2014} {error_message} ({duration_secs:.2}s)"
                );
            }
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;
    use crate::batch::summary::{BatchSummary, FileResult};

    // T008: format_batch_summary tests

    #[test]
    fn all_success_format() {
        let results = vec![
            FileResult::success(
                PathBuf::from("alpha.md"),
                PathBuf::from("out/alpha.json"),
                Duration::from_millis(450),
            ),
            FileResult::success(
                PathBuf::from("bravo.md"),
                PathBuf::from("out/bravo.json"),
                Duration::from_millis(380),
            ),
        ];
        let summary = BatchSummary::from_results(results, Duration::from_millis(1230));
        let output = format_batch_summary(&summary);

        assert!(output.contains("2 files (2 succeeded, 0 failed)"));
        assert!(output.contains("\u{2713} alpha.md"));
        assert!(output.contains("\u{2713} bravo.md"));
        assert!(!output.contains("\u{2717}"));
    }

    #[test]
    fn all_failure_format() {
        let results = vec![
            FileResult::failure(
                PathBuf::from("bad1.md"),
                "Parse error".to_string(),
                Duration::from_millis(100),
            ),
            FileResult::failure(
                PathBuf::from("bad2.md"),
                "Empty file".to_string(),
                Duration::from_millis(50),
            ),
        ];
        let summary = BatchSummary::from_results(results, Duration::from_millis(200));
        let output = format_batch_summary(&summary);

        assert!(output.contains("2 files (0 succeeded, 2 failed)"));
        assert!(output.contains("\u{2717} bad1.md"));
        assert!(output.contains("Parse error"));
        assert!(output.contains("\u{2717} bad2.md"));
        assert!(output.contains("Empty file"));
    }

    #[test]
    fn mixed_success_failure_format() {
        let results = vec![
            FileResult::success(
                PathBuf::from("good.md"),
                PathBuf::from("out/good.json"),
                Duration::from_millis(400),
            ),
            FileResult::failure(
                PathBuf::from("bad.md"),
                "No structure".to_string(),
                Duration::from_millis(100),
            ),
        ];
        let summary = BatchSummary::from_results(results, Duration::from_millis(500));
        let output = format_batch_summary(&summary);

        assert!(output.contains("2 files (1 succeeded, 1 failed)"));
        assert!(output.contains("\u{2713} good.md"));
        assert!(output.contains("\u{2717} bad.md"));
    }

    #[test]
    fn single_file_summary() {
        let results = vec![FileResult::success(
            PathBuf::from("only.md"),
            PathBuf::from("only.json"),
            Duration::from_millis(250),
        )];
        let summary = BatchSummary::from_results(results, Duration::from_millis(250));
        let output = format_batch_summary(&summary);

        assert!(output.contains("1 file (1 succeeded, 0 failed)"));
        assert!(output.contains("\u{2713} only.md"));
    }
}

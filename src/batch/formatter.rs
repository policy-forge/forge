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
    // Writing to a String cannot fail; discard fmt::Result to keep the formatter infallible.

    let total_secs = summary.total_duration().as_secs_f64();
    let files_label = if summary.total_files() == 1 { "file" } else { "files" };
    let _ = writeln!(
        buf,
        "Batch conversion complete: {} {files_label} ({} succeeded, {} failed) in {total_secs:.2}s",
        summary.total_files(),
        summary.succeeded(),
        summary.failed(),
    );
    let _ = writeln!(buf);
    if summary.degraded_to_sequential() {
        let _ = writeln!(
            buf,
            "Warning: requested parallel execution fell back to sequential processing."
        );
        let _ = writeln!(buf);
    }

    for result in summary.results() {
        let input_name = match result.input_path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => std::borrow::Cow::Owned(result.input_path.display().to_string()),
        };
        let duration_secs = result.duration.as_secs_f64();

        match &result.outcome {
            FileOutcome::Success { output_path } => {
                let output_display = output_path.display();
                let _ = writeln!(
                    buf,
                    "  \u{2713} {input_name} \u{2192} {output_display} ({duration_secs:.2}s)"
                );
            }
            FileOutcome::Failure { error } => {
                let error_message = sanitize_terminal_line(&error.to_string());
                let _ = writeln!(
                    buf,
                    "  \u{2717} {input_name} \u{2014} {error_message} ({duration_secs:.2}s)"
                );
            }
        }
    }

    buf
}

/// Flatten control characters so untrusted errors cannot forge terminal rows.
fn sanitize_terminal_line(value: &str) -> String {
    value.chars().map(|character| if character.is_control() { ' ' } else { character }).collect()
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
                crate::error::ForgeError::Parse("Parse error".to_string()),
                Duration::from_millis(100),
            ),
            FileResult::failure(
                PathBuf::from("bad2.md"),
                crate::error::ForgeError::Parse("Empty file".to_string()),
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
                crate::error::ForgeError::Parse("No structure".to_string()),
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

    #[test]
    fn failure_message_controls_cannot_forge_rows() {
        let summary = BatchSummary::from_results(
            vec![FileResult::failure(
                PathBuf::from("bad.md"),
                crate::error::ForgeError::Parse("bad\n\u{1b}[2J".to_string()),
                Duration::ZERO,
            )],
            Duration::ZERO,
        );

        let output = format_batch_summary(&summary);
        assert!(!output.contains('\u{1b}'));
        assert!(output.contains("Parse error: bad  [2J"));
        assert_eq!(output.lines().filter(|line| line.contains("bad.md")).count(), 1);
    }

    #[test]
    fn empty_summary_has_exact_header() {
        let summary = BatchSummary::from_results(Vec::new(), Duration::ZERO);

        assert_eq!(
            format_batch_summary(&summary),
            "Batch conversion complete: 0 files (0 succeeded, 0 failed) in 0.00s\n\n"
        );
    }

    #[test]
    fn componentless_input_uses_full_path() {
        let summary = BatchSummary::from_results(
            vec![FileResult::success(PathBuf::from(""), PathBuf::from("out.json"), Duration::ZERO)],
            Duration::ZERO,
        );

        assert!(format_batch_summary(&summary).contains("✓  → out.json (0.00s)"));
    }

    #[test]
    fn multi_line_failure_is_rendered_as_one_row() {
        let summary = BatchSummary::from_results(
            vec![FileResult::failure(
                PathBuf::from("bad.md"),
                crate::error::ForgeError::Parse("first\nsecond".to_string()),
                Duration::ZERO,
            )],
            Duration::ZERO,
        );

        let output = format_batch_summary(&summary);
        assert!(output.contains("Parse error: first second"));
        assert_eq!(output.lines().filter(|line| line.contains("bad.md")).count(), 1);
    }
}

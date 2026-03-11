//! Dashboard formatting: box-drawing table with ANSI color support.

use std::fmt::Write;
use std::time::Duration;

use super::{ConversionStatistics, ValidationStatus};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

const MIN_WIDTH: usize = 41;
const LABEL_WIDTH: usize = 22; // "│ Strategy:           " = 22 chars

fn color(code: &str, text: &str, use_color: bool) -> String {
    if use_color { format!("{code}{text}{RESET}") } else { text.to_string() }
}

/// Compute visible (non-ANSI) character length of a string.
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Pad a string to `width` based on its visible (non-ANSI) length.
fn pad_right(text: &str, width: usize) -> String {
    let vis = visible_len(text);
    if vis >= width { text.to_string() } else { format!("{text}{}", " ".repeat(width - vis)) }
}

/// Format a `Duration` as a human-readable string.
///
/// - < 60s: `"X.XXs"` (2 decimal places for sub-second, 1 for multi-second)
/// - >= 60s: `"Xm Xs"`
#[must_use]
pub fn format_elapsed(duration: Duration) -> String {
    let secs = duration.as_secs_f64();
    // Round to 2dp first, then select display bucket based on the rounded value.
    // This avoids boundary jumps (e.g. 0.999s displaying as "1.00s" or 59.95s as "60.0s").
    let rounded = (secs * 100.0).round() / 100.0;
    if rounded >= 60.0 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let total = rounded as u64;
        format!("{}m {}s", total / 60, total % 60)
    } else if rounded >= 1.0 {
        format!("{rounded:.1}s")
    } else {
        format!("{rounded:.2}s")
    }
}

fn compute_width(
    stats: &ConversionStatistics,
    coverage_str: &str,
    validation_label: &str,
    validation_details: &[String],
) -> usize {
    let detail_max = validation_details
        .iter()
        .map(|line| visible_len(line) + 3) // "│   " prefix = 3 chars inner indent
        .max()
        .unwrap_or(0);
    let longest_value = [
        visible_len(&stats.strategy),
        visible_len(&stats.output_path),
        format_elapsed(stats.elapsed).len(),
        coverage_str.len(),
        visible_len(validation_label),
        detail_max,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);
    // LABEL_WIDTH + value + 1 (trailing space before │)
    (LABEL_WIDTH + longest_value + 1).max(MIN_WIDTH)
}

/// Returns the status label only (single line). Error message lines are
/// rendered separately in `format_summary_dashboard` so they get proper
/// box-drawing alignment.
fn format_validation_label(stats: &ConversionStatistics, use_color: bool) -> String {
    match &stats.validation_status {
        ValidationStatus::Passed => color(GREEN, "PASSED", use_color),
        ValidationStatus::PassedWithWarnings => {
            let label = format!("PASSED with {} warnings", stats.validation_warnings);
            color(YELLOW, &label, use_color)
        }
        ValidationStatus::Failed => color(RED, "FAILED", use_color),
        ValidationStatus::NotRun => "Not run".to_string(),
    }
}

/// Returns extra validation detail lines (error messages + overflow) for the Failed variant.
fn format_validation_detail_lines(stats: &ConversionStatistics) -> Vec<String> {
    if stats.validation_status != ValidationStatus::Failed {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let msgs = &stats.validation_error_messages;
    let total = stats.validation_errors;
    let shown = msgs.len().min(3);
    for msg in msgs.iter().take(shown) {
        lines.push(format!("  {msg}"));
    }
    if total > shown {
        lines.push(format!("  and {} more...", total - shown));
    }
    lines
}

fn format_mapping_coverage(
    pct: f64,
    coverage_raw: &str,
    stats: &ConversionStatistics,
    use_color: bool,
) -> String {
    if stats.requirements_extracted == 0 || pct > 100.0 {
        color(YELLOW, coverage_raw, use_color)
    } else {
        coverage_raw.to_string()
    }
}

/// Format conversion statistics as a human-readable box-drawing dashboard.
///
/// Uses ANSI color codes when `use_color` is true.
#[must_use]
pub fn format_summary_dashboard(stats: &ConversionStatistics, use_color: bool) -> String {
    let pct = stats.mapping_coverage();
    let coverage_raw =
        format!("{pct:.1}% ({}/{})", stats.controls_generated, stats.requirements_extracted);
    let validation = format_validation_label(stats, use_color);
    let validation_details = format_validation_detail_lines(stats);
    let w = compute_width(stats, &coverage_raw, &validation, &validation_details);
    // LABEL_WIDTH includes the leading │ border char, so +1 to align
    // row total (LABEL_WIDTH + vw + 1 for closing │) with border total (1 + w + 1)
    let vw = w - LABEL_WIDTH + 1;

    let top = format!("┌{}┐", "─".repeat(w));
    let bot = format!("└{}┘", "─".repeat(w));
    let sep = format!("├{}┤", "─".repeat(w));

    let title = "FORGE Conversion Summary";
    let title_pad = (w - title.len()) / 2;
    let title_line = format!(
        "│{:>pad$}{}{:remaining$}│",
        "",
        color(BOLD, title, use_color),
        "",
        pad = title_pad,
        remaining = w - title_pad - title.len()
    );

    let coverage = format_mapping_coverage(pct, &coverage_raw, stats, use_color);

    let rows = [
        format!("│ Strategy:           {:<vw$}│", stats.strategy),
        format!("│ Output:             {:<vw$}│", stats.output_path),
        format!("│ Elapsed:            {:<vw$}│", format_elapsed(stats.elapsed)),
    ];

    let stat_rows = [
        format!("│ Sections parsed:    {:<vw$}│", stats.sections_parsed),
        format!("│ Requirements:       {:<vw$}│", stats.requirements_extracted),
        format!("│ Controls generated: {:<vw$}│", stats.controls_generated),
        format!("│ Mapping coverage:   {}│", pad_right(&coverage, vw)),
    ];

    let val_row = format!("│ Validation:         {}│", pad_right(&validation, vw));

    let mut out = String::new();
    writeln!(out, "{top}").unwrap();
    writeln!(out, "{title_line}").unwrap();
    writeln!(out, "{sep}").unwrap();
    for row in &rows {
        writeln!(out, "{row}").unwrap();
    }
    writeln!(out, "{sep}").unwrap();
    for row in &stat_rows {
        writeln!(out, "{row}").unwrap();
    }
    writeln!(out, "{sep}").unwrap();
    writeln!(out, "{val_row}").unwrap();
    // Render validation error detail lines with proper box alignment.
    // Format: "│   {detail}│" → border(1) + spaces(3) + detail(detail_vw) + border(1) = w + 2
    let detail_vw = w - 3; // 3 = inner indent (3 spaces after border │)
    for detail in &validation_details {
        writeln!(out, "│   {detail:<detail_vw$}│").unwrap();
    }
    write!(out, "{bot}").unwrap();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // T007: format_elapsed() tests
    #[test]
    fn format_elapsed_sub_second() {
        let d = Duration::from_secs_f64(0.42);
        assert_eq!(format_elapsed(d), "0.42s");
    }

    #[test]
    fn format_elapsed_multi_second() {
        let d = Duration::from_secs_f64(3.2);
        assert_eq!(format_elapsed(d), "3.2s");
    }

    #[test]
    fn format_elapsed_over_minute() {
        let d = Duration::from_secs(83);
        assert_eq!(format_elapsed(d), "1m 23s");
    }

    #[test]
    fn format_elapsed_zero() {
        let d = Duration::ZERO;
        assert_eq!(format_elapsed(d), "0.00s");
    }

    #[test]
    fn format_elapsed_boundary_rounds_up_to_1s() {
        // 0.999s rounds to 1.00 at 2dp, so it enters the >=1s bucket.
        // This is correct: round first, then format.
        let d = Duration::from_secs_f64(0.999);
        assert_eq!(format_elapsed(d), "1.0s");
    }

    #[test]
    fn format_elapsed_boundary_stays_sub_second() {
        // 0.994s rounds to 0.99 at 2dp, stays in sub-second bucket.
        let d = Duration::from_secs_f64(0.994);
        assert_eq!(format_elapsed(d), "0.99s");
    }

    #[test]
    fn format_elapsed_boundary_just_over_1s() {
        let d = Duration::from_secs_f64(1.001);
        assert_eq!(format_elapsed(d), "1.0s");
    }

    #[test]
    fn format_elapsed_boundary_just_under_60s() {
        // 59.994s rounds to 59.99 at 2dp, stays in seconds bucket.
        let d = Duration::from_secs_f64(59.994);
        let result = format_elapsed(d);
        assert!(!result.contains('m'), "59.994s should stay in seconds bucket: {result}");
    }

    #[test]
    fn format_elapsed_boundary_rounds_up_to_60s() {
        // 59.999s rounds to 60.00 at 2dp, enters minute bucket.
        let d = Duration::from_secs_f64(59.999);
        assert_eq!(format_elapsed(d), "1m 0s");
    }

    #[test]
    fn format_elapsed_boundary_at_60s() {
        let d = Duration::from_secs(60);
        assert_eq!(format_elapsed(d), "1m 0s");
    }

    // T008: format_summary_dashboard() tests (no color)
    #[test]
    fn format_dashboard_contains_key_elements_no_color() {
        let stats = ConversionStatistics {
            sections_parsed: 5,
            requirements_extracted: 10,
            controls_generated: 10,
            validation_status: ValidationStatus::Passed,
            strategy: "catalog".into(),
            output_path: "out.json".into(),
            elapsed: Duration::from_secs_f64(0.42),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);

        assert!(output.contains("FORGE Conversion Summary"), "Missing title");
        assert!(output.contains("Sections parsed:"), "Missing sections label");
        assert!(output.contains("Requirements:"), "Missing requirements label");
        assert!(output.contains("Controls generated:"), "Missing controls label");
        assert!(output.contains("Mapping coverage:"), "Missing coverage label");
        assert!(output.contains("Validation:"), "Missing validation label");
        // Box-drawing characters
        assert!(output.contains('┌'), "Missing top-left corner");
        assert!(output.contains('─'), "Missing horizontal line");
        assert!(output.contains('┐'), "Missing top-right corner");
        assert!(output.contains('│'), "Missing vertical line");
        assert!(output.contains('└'), "Missing bottom-left corner");
        assert!(output.contains('┘'), "Missing bottom-right corner");
    }

    #[test]
    fn format_dashboard_snapshot_no_color() {
        let stats = ConversionStatistics {
            sections_parsed: 12,
            requirements_extracted: 47,
            controls_generated: 47,
            validation_status: ValidationStatus::Passed,
            strategy: "catalog".into(),
            output_path: "output/catalog.json".into(),
            elapsed: Duration::from_secs_f64(0.42),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn format_dashboard_with_color_has_ansi() {
        let stats = ConversionStatistics {
            sections_parsed: 5,
            requirements_extracted: 10,
            controls_generated: 10,
            validation_status: ValidationStatus::Passed,
            strategy: "catalog".into(),
            output_path: "out.json".into(),
            elapsed: Duration::from_secs_f64(1.0),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        assert!(output.contains("\x1b["), "Expected ANSI escape codes with use_color=true");
    }

    // T009: Failed validation with overflow
    #[test]
    fn format_dashboard_failed_validation_overflow() {
        let stats = ConversionStatistics {
            sections_parsed: 5,
            requirements_extracted: 10,
            controls_generated: 8,
            validation_status: ValidationStatus::Failed,
            validation_errors: 5,
            validation_error_messages: vec!["Error 1".into(), "Error 2".into(), "Error 3".into()],
            strategy: "catalog".into(),
            output_path: "out.json".into(),
            elapsed: Duration::from_secs_f64(0.5),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("FAILED"), "Should show FAILED");
        assert!(output.contains("Error 1"), "Should show first error");
        assert!(output.contains("Error 2"), "Should show second error");
        assert!(output.contains("Error 3"), "Should show third error");
        assert!(output.contains("and 2 more..."), "Should show overflow count");
    }

    // T023: ValidationStatus::Passed renders as "PASSED"
    #[test]
    fn validation_passed_renders_correctly() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::Passed,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("PASSED"));
    }

    #[test]
    fn validation_passed_green_when_color() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::Passed,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        assert!(output.contains("\x1b[32mPASSED\x1b[0m"));
    }

    // T024: ValidationStatus::NotRun renders as "Not run"
    #[test]
    fn validation_not_run_renders_correctly() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::NotRun,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("Not run"));
    }

    // T025: Failed with 5 errors shows 3 + overflow
    #[test]
    fn validation_failed_five_errors_shows_three_plus_overflow() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::Failed,
            validation_errors: 5,
            validation_error_messages: vec!["msg1".into(), "msg2".into(), "msg3".into()],
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("FAILED"));
        assert!(output.contains("msg1"));
        assert!(output.contains("msg2"));
        assert!(output.contains("msg3"));
        assert!(output.contains("and 2 more..."));
    }

    // T030: zero requirements renders "0.0% (0/0)" with warning
    #[test]
    fn mapping_coverage_zero_renders_warning() {
        let stats = ConversionStatistics {
            requirements_extracted: 0,
            controls_generated: 0,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("0.0% (0/0)"));
    }

    // T031: >100% coverage renders correctly
    #[test]
    fn mapping_coverage_over_100_renders_correctly() {
        let stats = ConversionStatistics {
            requirements_extracted: 15,
            controls_generated: 18,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("120.0% (18/15)"));
    }

    // T034: elapsed time appears in dashboard
    #[test]
    fn elapsed_time_in_dashboard() {
        let stats =
            ConversionStatistics { elapsed: Duration::from_secs_f64(0.42), ..Default::default() };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("0.42s"));
    }

    // T035: ANSI color codes present/absent
    #[test]
    fn ansi_codes_present_when_color_true() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::Passed,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn ansi_codes_absent_when_color_false() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::Passed,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(!output.contains("\x1b["));
    }

    // T045: PassedWithWarnings renders correctly
    #[test]
    fn validation_passed_with_warnings_renders_correctly() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::PassedWithWarnings,
            validation_warnings: 3,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, false);
        assert!(output.contains("PASSED with 3 warnings"));
    }

    // Fix 8: PassedWithWarnings renders with yellow ANSI when color enabled
    #[test]
    fn validation_passed_with_warnings_yellow_when_color() {
        let stats = ConversionStatistics {
            validation_status: ValidationStatus::PassedWithWarnings,
            validation_warnings: 2,
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        assert!(
            output.contains("\x1b[33mPASSED with 2 warnings\x1b[0m"),
            "PassedWithWarnings should use yellow ANSI"
        );
    }

    // Fix 7: Snapshot test for colored output (catches ANSI alignment regressions)
    #[test]
    fn format_dashboard_snapshot_with_color() {
        let stats = ConversionStatistics {
            sections_parsed: 12,
            requirements_extracted: 47,
            controls_generated: 47,
            validation_status: ValidationStatus::Passed,
            strategy: "catalog".into(),
            output_path: "output/catalog.json".into(),
            elapsed: Duration::from_secs_f64(0.42),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        insta::assert_snapshot!(output);
    }

    // Fix 1: Verify colored rows have correct alignment (right border │ is at consistent position)
    #[test]
    fn colored_rows_have_consistent_width() {
        let stats = ConversionStatistics {
            sections_parsed: 5,
            requirements_extracted: 0,
            controls_generated: 0,
            validation_status: ValidationStatus::Passed,
            strategy: "catalog".into(),
            output_path: "out.json".into(),
            elapsed: Duration::from_secs_f64(1.0),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        assert_all_lines_same_visible_width(&output);
    }

    // H-1: Verify Failed validation with error messages also has consistent alignment
    #[test]
    fn failed_validation_rows_have_consistent_width() {
        let stats = ConversionStatistics {
            sections_parsed: 5,
            requirements_extracted: 10,
            controls_generated: 8,
            validation_status: ValidationStatus::Failed,
            validation_errors: 5,
            validation_error_messages: vec!["Error 1".into(), "Error 2".into(), "Error 3".into()],
            strategy: "catalog".into(),
            output_path: "out.json".into(),
            elapsed: Duration::from_secs_f64(0.5),
            ..Default::default()
        };
        let output = format_summary_dashboard(&stats, true);
        assert_all_lines_same_visible_width(&output);
    }

    fn assert_all_lines_same_visible_width(output: &str) {
        let lines: Vec<&str> = output.lines().collect();
        let top_len = visible_len(lines[0]);
        for (i, line) in lines.iter().enumerate() {
            let vis = visible_len(line);
            assert_eq!(
                vis, top_len,
                "Line {i} visible width {vis} != top width {top_len}: {line:?}"
            );
        }
    }
}

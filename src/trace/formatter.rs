use std::fmt::Write;

use super::report::{ElementType, TraceReport};
use super::resolver::validate_line_reference;
use crate::sanitize::strip_control_chars;

/// Format a `TraceReport` as a column-aligned text table.
///
/// Columns: OSCAL Element ID, Element Type, Source Section, Source Line
/// Includes header, separator, data rows, and summary footer.
/// Unmapped elements show `[unmapped]` in source columns.
/// Groups with section but no line show "—" for Source Line.
///
/// All source-derived strings have control characters stripped and whitespace
/// controls replaced with spaces so each entry occupies a single table row.
#[must_use]
pub fn format_trace_table(report: &TraceReport) -> String {
    let source_line_count = report.source_line_count;
    let headers = ["OSCAL Element ID", "Element Type", "Source Section", "Source Line"];

    // Compute display values for each entry.
    let rows: Vec<[String; 4]> = report
        .entries
        .iter()
        .map(|entry| {
            let (section, line) = match &entry.trace {
                Some(meta) => (
                    format_table_cell(&meta.source_section),
                    format_source_line(meta.source_line, entry.element_type, source_line_count),
                ),
                None => ("[unmapped]".to_string(), "[unmapped]".to_string()),
            };
            [
                format_table_cell(&entry.element_id),
                entry.element_type.as_str().to_string(),
                section,
                line,
            ]
        })
        .collect();

    // First pass: compute max column widths.
    let mut widths = [0usize; 4];
    for (i, header) in headers.iter().enumerate() {
        widths[i] = header.len();
    }
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            let display_len = cell.chars().count();
            if display_len > widths[i] {
                widths[i] = display_len;
            }
        }
    }

    let mut output = String::new();

    // Staleness warning.
    if report.source_stale {
        output.push_str("Warning: Source file may have been modified since conversion (source is newer than artifact)\n\n");
    }

    // Writing to `String` via `fmt::Write` is infallible; `let _` is intentional.
    let _ = writeln!(
        output,
        "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}",
        headers[0],
        headers[1],
        headers[2],
        headers[3],
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2],
        w3 = widths[3],
    );

    let _ = writeln!(
        output,
        "{:-<w0$}  {:-<w1$}  {:-<w2$}  {:-<w3$}",
        "",
        "",
        "",
        "",
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2],
        w3 = widths[3],
    );

    for row in &rows {
        let _ = writeln!(
            output,
            "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}",
            row[0],
            row[1],
            row[2],
            row[3],
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2],
            w3 = widths[3],
        );
    }

    let summary = report.summary();
    let elem_word = if summary.total_elements == 1 { "element" } else { "elements" };
    let _ = write!(
        output,
        "\nSummary: {} {elem_word}, {} mapped, {} unmapped ({:.1}% coverage)\n",
        summary.total_elements,
        summary.mapped_elements,
        summary.unmapped_elements(),
        summary.coverage_percent()
    );

    output
}

fn format_source_line(
    source_line: Option<usize>,
    element_type: ElementType,
    source_line_count: usize,
) -> String {
    match source_line {
        None if element_type == ElementType::Group => "—".to_string(),
        None => "[missing] ⚠".to_string(),
        Some(line) if validate_line_reference(Some(line), source_line_count) => line.to_string(),
        Some(line) => format!("{line} ⚠"),
    }
}

fn format_table_cell(value: &str) -> String {
    strip_control_chars(&value.replace(['\n', '\r', '\t'], " "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::report::{ElementType, TraceEntry, TraceMetadata, TraceReport};
    use crate::types::OscalModelType;
    use std::path::PathBuf;

    fn make_report(entries: Vec<TraceEntry>, source_stale: bool) -> TraceReport {
        TraceReport {
            artifact_path: PathBuf::from("artifact.json"),
            source_path: PathBuf::from("policy.md"),
            artifact_type: OscalModelType::Catalog,
            entries,
            source_stale,
            source_line_count: 100,
        }
    }

    // T015: format_trace_table snapshot tests

    #[test]
    fn format_mapped_entries() {
        let entries = vec![
            TraceEntry {
                element_id: "access-control".to_string(),
                element_type: ElementType::Group,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access Control".to_string(),
                    source_line: None,
                }),
            },
            TraceEntry {
                element_id: "POL-AC-001".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access Control".to_string(),
                    source_line: Some(10),
                }),
            },
            TraceEntry {
                element_id: "POL-AC-002".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access Control".to_string(),
                    source_line: Some(25),
                }),
            },
        ];
        let report = make_report(entries, false);
        let output = format_trace_table(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn format_with_unmapped_entries() {
        let entries = vec![
            TraceEntry {
                element_id: "POL-AC-001".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access Control".to_string(),
                    source_line: Some(10),
                }),
            },
            TraceEntry {
                element_id: "POL-AC-002".to_string(),
                element_type: ElementType::Control,
                trace: None,
            },
        ];
        let report = make_report(entries, false);
        let output = format_trace_table(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn format_empty_report() {
        let report = make_report(vec![], false);
        let output = format_trace_table(&report);
        insta::assert_snapshot!(output);
    }

    #[test]
    fn format_with_group_em_dash() {
        let entries = vec![TraceEntry {
            element_id: "access-control".to_string(),
            element_type: ElementType::Group,
            trace: Some(TraceMetadata {
                source_file: "policy.md".to_string(),
                source_section: "Access Control".to_string(),
                source_line: None,
            }),
        }];
        let report = make_report(entries, false);
        let output = format_trace_table(&report);
        assert!(output.contains('\u{2014}')); // em dash
    }

    // T034: Coverage summary display tests

    #[test]
    fn summary_100_percent() {
        let entries = vec![TraceEntry {
            element_id: "a".to_string(),
            element_type: ElementType::Control,
            trace: Some(TraceMetadata {
                source_file: "p.md".to_string(),
                source_section: "S".to_string(),
                source_line: Some(1),
            }),
        }];
        let report = make_report(entries, false);
        let output = format_trace_table(&report);
        assert!(output.contains("1 element, 1 mapped, 0 unmapped (100.0% coverage)"));
    }

    #[test]
    fn summary_0_percent() {
        let entries = vec![TraceEntry {
            element_id: "a".to_string(),
            element_type: ElementType::Control,
            trace: None,
        }];
        let report = make_report(entries, false);
        let output = format_trace_table(&report);
        assert!(output.contains("1 element, 0 mapped, 1 unmapped (0.0% coverage)"));
    }

    #[test]
    fn staleness_warning_shown() {
        let report = make_report(vec![], true);
        let output = format_trace_table(&report);
        assert!(output.contains("Warning: Source file may have been modified since conversion"));
    }

    #[test]
    fn staleness_warning_not_shown() {
        let report = make_report(vec![], false);
        let output = format_trace_table(&report);
        assert!(!output.contains("Warning:"));
    }

    #[test]
    fn source_derived_cells_replace_whitespace_controls() {
        let report = make_report(
            vec![TraceEntry {
                element_id: "control\nwith\tid".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access\nControl\tDone\rNow".to_string(),
                    source_line: Some(1),
                }),
            }],
            false,
        );

        let output = format_trace_table(&report);
        assert!(output.contains("control with id"));
        assert!(output.contains("Access Control Done Now"));
        assert!(!output.contains("control\nwith"));
        assert!(!output.contains("Access\nControl"));
    }
}

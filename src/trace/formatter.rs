use std::fmt::Write;

use super::extractor::strip_control_chars;
use super::report::{ElementType, TraceReport};
use super::resolver::validate_line_reference;

/// Format a `TraceReport` as a column-aligned text table.
///
/// Columns: OSCAL Element ID, Element Type, Source Section, Source Line
/// Includes header, separator, data rows, and summary footer.
/// Unmapped elements show `[unmapped]` in source columns.
/// Groups with section but no line show "\u{2014}" for Source Line.
///
/// All source-derived strings have control characters stripped (SEC-5).
#[must_use]
pub fn format_trace_table(report: &TraceReport) -> String {
    let source_line_count = report.source_line_count;
    let headers = ["OSCAL Element ID", "Element Type", "Source Section", "Source Line"];

    // Compute display values for each entry
    let rows: Vec<[String; 4]> = report
        .entries
        .iter()
        .map(|entry| {
            let (section, line) = match &entry.trace {
                Some(meta) => {
                    let section = strip_control_chars(&meta.source_section);
                    let line = if meta.source_line == 0 && entry.element_type == ElementType::Group
                    {
                        "\u{2014}".to_string() // em dash for groups
                    } else if meta.source_line == 0 {
                        "0 \u{26A0}".to_string() // missing line on non-group element
                    } else {
                        let line_str = meta.source_line.to_string();
                        if validate_line_reference(meta.source_line, source_line_count) {
                            line_str
                        } else {
                            format!("{line_str} \u{26A0}")
                        }
                    };
                    (section, line)
                }
                None => ("[unmapped]".to_string(), "[unmapped]".to_string()),
            };
            [
                strip_control_chars(&entry.element_id),
                entry.element_type.as_str().to_string(),
                section,
                line,
            ]
        })
        .collect();

    // First pass: compute max column widths
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

    // Staleness warning
    if report.source_stale {
        output.push_str("Warning: Source file may have been modified since conversion (source is newer than artifact)\n\n");
    }

    // Header
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

    // Separator
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

    // Data rows
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

    // Summary footer
    let s = &report.summary;
    let elem_word = if s.total_elements == 1 { "element" } else { "elements" };
    let _ = write!(
        output,
        "\nSummary: {} {elem_word}, {} mapped, {} unmapped ({:.1}% coverage)\n",
        s.total_elements, s.mapped_elements, s.unmapped_elements(), s.coverage_percent()
    );

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::report::{ElementType, TraceEntry, TraceMetadata, TraceReport, TraceSummary};
    use crate::types::OscalModelType;
    use std::path::PathBuf;

    fn make_report(entries: Vec<TraceEntry>, source_stale: bool) -> TraceReport {
        let summary = TraceSummary::from_entries(&entries);
        TraceReport {
            artifact_path: PathBuf::from("artifact.json"),
            source_path: PathBuf::from("policy.md"),
            artifact_type: OscalModelType::Catalog,
            entries,
            summary,
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
                    source_line: 0,
                }),
            },
            TraceEntry {
                element_id: "POL-AC-001".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access Control".to_string(),
                    source_line: 10,
                }),
            },
            TraceEntry {
                element_id: "POL-AC-002".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "policy.md".to_string(),
                    source_section: "Access Control".to_string(),
                    source_line: 25,
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
                    source_line: 10,
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
                source_line: 0,
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
                source_line: 1,
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
}

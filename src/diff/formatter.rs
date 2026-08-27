use std::fmt::Write;

use super::types::{DiffEntry, DiffReport};

/// Format a `DiffReport` as a human-readable plain-text report.
///
/// Renders added, removed, modified, and unchanged control counts, followed
/// by per-control field-level change details.
#[must_use]
pub fn format_diff_report(report: &DiffReport) -> String {
    debug_assert_eq!(
        report.summary.added,
        report.entries.iter().filter(|entry| matches!(entry, DiffEntry::Added { .. })).count(),
        "diff summary added count must match entries"
    );
    debug_assert_eq!(
        report.summary.removed,
        report.entries.iter().filter(|entry| matches!(entry, DiffEntry::Removed { .. })).count(),
        "diff summary removed count must match entries"
    );
    debug_assert_eq!(
        report.summary.changed,
        report.entries.iter().filter(|entry| matches!(entry, DiffEntry::Changed { .. })).count(),
        "diff summary changed count must match entries"
    );
    debug_assert_eq!(
        report.summary.uuid_changes,
        report
            .entries
            .iter()
            .filter(|entry| matches!(entry, DiffEntry::UuidChanged { .. }))
            .count(),
        "diff summary UUID-change count must match entries"
    );

    let mut out = String::new();

    // Writing to String is infallible; every `writeln!` below uses that invariant.
    writeln!(out, "OSCAL Diff Report").unwrap();
    writeln!(out, "=================").unwrap();
    writeln!(out, "Old: {}  ({})", report.old_file, report.artifact_type).unwrap();
    writeln!(out, "New: {}  ({})", report.new_file, report.artifact_type).unwrap();
    writeln!(out).unwrap();

    let s = &report.summary;
    writeln!(out, "Summary").unwrap();
    writeln!(out, "-------").unwrap();
    writeln!(out, "Controls (old): {}  |  Controls (new): {}", s.total_old, s.total_new).unwrap();
    writeln!(
        out,
        "Added: {}  |  Removed: {}  |  Changed: {}  |  Unchanged: {}  |  UUID changes: {}",
        s.added, s.removed, s.changed, s.unchanged, s.uuid_changes
    )
    .unwrap();
    writeln!(out).unwrap();

    if report.entries.is_empty() {
        writeln!(out, "No differences found.").unwrap();
        return out;
    }

    let mut added = Vec::new();
    let mut changed = Vec::new();
    let mut removed = Vec::new();
    let mut uuid_changed = Vec::new();
    for entry in &report.entries {
        match entry {
            DiffEntry::Added { .. } => added.push(entry),
            DiffEntry::Changed { .. } => changed.push(entry),
            DiffEntry::Removed { .. } => removed.push(entry),
            DiffEntry::UuidChanged { .. } => uuid_changed.push(entry),
        }
    }

    format_added_section(&mut out, &added);
    format_changed_section(&mut out, &changed);
    format_removed_section(&mut out, &removed);
    format_uuid_section(&mut out, &uuid_changed);

    out
}

fn write_section_heading(out: &mut String, title: &str) {
    writeln!(out, "{title}").unwrap();
    writeln!(out, "{}", "─".repeat(title.chars().count())).unwrap();
}

fn write_rows<F>(out: &mut String, title: &str, entries: &[&DiffEntry], mut write_row: F)
where
    F: FnMut(&mut String, &DiffEntry),
{
    write_section_heading(out, &format!("{title} ({})", entries.len()));
    if entries.is_empty() {
        writeln!(out, "  (none)").unwrap();
    } else {
        for entry in entries {
            write_row(out, entry);
        }
    }
    writeln!(out).unwrap();
}

fn display_uuid(uuid: Option<&String>) -> &str {
    uuid.map_or("(absent)", String::as_str)
}

fn one_line(value: &str, max_chars: usize) -> String {
    let mut rendered = String::with_capacity(value.len().min(max_chars));
    for character in value.chars() {
        match character {
            '\n' | '\r' | '\t' => rendered.push(' '),
            '→' => rendered.push_str("\\u{2192}"),
            _ => rendered.push(character),
        }
    }
    if rendered.chars().count() > max_chars {
        let cutoff =
            rendered.char_indices().nth(max_chars).map_or(rendered.len(), |(index, _)| index);
        rendered.truncate(cutoff);
        rendered.push_str("[...truncated]");
    }
    rendered
}

fn format_added_section(out: &mut String, added: &[&DiffEntry]) {
    write_rows(out, "Added", added, |out, entry| {
        if let DiffEntry::Added { control_id, new_uuid } = entry {
            writeln!(out, "  + {control_id}  [uuid: {}]", display_uuid(new_uuid.as_ref())).unwrap();
        }
    });
}

fn format_changed_section(out: &mut String, changed: &[&DiffEntry]) {
    write_rows(out, "Changed", changed, |out, entry| {
        if let DiffEntry::Changed { control_id, old_uuid, new_uuid, field_changes } = entry {
            writeln!(out, "  ~ {control_id}").unwrap();
            for field_change in field_changes {
                let old_value = field_change
                    .old_value
                    .as_deref()
                    .map_or_else(|| "(absent)".to_string(), |value| one_line(value, 200));
                let new_value = field_change
                    .new_value
                    .as_deref()
                    .map_or_else(|| "(absent)".to_string(), |value| one_line(value, 200));
                writeln!(
                    out,
                    "      {}: \"{}\"  →  \"{}\"",
                    field_change.field_name, old_value, new_value
                )
                .unwrap();
            }
            if entry.uuid_changed() {
                writeln!(
                    out,
                    "      [UUID: {} → {}]",
                    display_uuid(old_uuid.as_ref()),
                    display_uuid(new_uuid.as_ref())
                )
                .unwrap();
            }
        }
    });
}

fn format_removed_section(out: &mut String, removed: &[&DiffEntry]) {
    write_rows(out, "Removed", removed, |out, entry| {
        if let DiffEntry::Removed { control_id, old_uuid } = entry {
            writeln!(out, "  - {control_id}  [uuid: {}]", display_uuid(old_uuid.as_ref())).unwrap();
        }
    });
}

fn format_uuid_section(out: &mut String, uuid_changed: &[&DiffEntry]) {
    write_rows(out, "UUID Stability Changes", uuid_changed, |out, entry| {
        if let DiffEntry::UuidChanged { control_id, old_uuid, new_uuid } = entry {
            writeln!(
                out,
                "  ! {control_id}  {}  →  {}",
                display_uuid(old_uuid.as_ref()),
                display_uuid(new_uuid.as_ref())
            )
            .unwrap();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::*;

    fn empty_report() -> DiffReport {
        DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![],
            summary: DiffSummary {
                total_old: 5,
                total_new: 5,
                added: 0,
                removed: 0,
                changed: 0,
                unchanged: 5,
                uuid_changes: 0,
            },
        }
    }

    #[test]
    fn test_format_output_contains_header() {
        let report = empty_report();
        let output = format_diff_report(&report);
        assert!(output.contains("OSCAL Diff Report"));
        assert!(output.contains("old.json"));
        assert!(output.contains("new.json"));
        assert!(output.contains("Catalog"));
    }

    #[test]
    fn test_summary_section_in_output() {
        let report = empty_report();
        let output = format_diff_report(&report);
        assert!(output.contains("Summary"));
        assert!(output.contains("Controls (old): 5"));
        assert!(output.contains("Controls (new): 5"));
        assert!(output.contains("Added: 0"));
        assert!(output.contains("Removed: 0"));
        assert!(output.contains("Changed: 0"));
        assert!(output.contains("Unchanged: 5"));
        assert!(output.contains("UUID changes: 0"));
    }

    #[test]
    fn test_no_differences_message() {
        let report = empty_report();
        let output = format_diff_report(&report);
        assert!(output.contains("No differences found."));
    }

    #[test]
    fn test_format_with_added_and_removed() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![
                DiffEntry::Added {
                    control_id: "POL-AC-002".into(),
                    new_uuid: Some("uuid-new".into()),
                },
                DiffEntry::Removed {
                    control_id: "POL-AC-003".into(),
                    old_uuid: Some("uuid-old".into()),
                },
            ],
            summary: DiffSummary {
                total_old: 2,
                total_new: 2,
                added: 1,
                removed: 1,
                changed: 0,
                unchanged: 1,
                uuid_changes: 0,
            },
        };
        let output = format_diff_report(&report);
        assert!(output.contains("Added (1)"));
        assert!(output.contains("+ POL-AC-002"));
        assert!(output.contains("Removed (1)"));
        assert!(output.contains("- POL-AC-003"));
    }

    #[test]
    fn test_format_changed_with_field_changes() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::Changed {
                control_id: "POL-IA-002".into(),
                old_uuid: None,
                new_uuid: None,
                field_changes: vec![FieldChange {
                    field_name: "title".into(),
                    old_value: Some("Old title".into()),
                    new_value: Some("New title".into()),
                }],
            }],
            summary: DiffSummary {
                total_old: 1,
                total_new: 1,
                added: 0,
                removed: 0,
                changed: 1,
                unchanged: 0,
                uuid_changes: 0,
            },
        };
        let output = format_diff_report(&report);
        assert!(output.contains("Changed (1)"));
        assert!(output.contains("~ POL-IA-002"));
        assert!(output.contains("title:"));
        assert!(output.contains("\"Old title\""));
        assert!(output.contains("\"New title\""));
    }

    #[test]
    fn format_changed_renders_absence_and_single_line_values() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::Changed {
                control_id: "AC-1".into(),
                old_uuid: Some("same".into()),
                new_uuid: Some("same".into()),
                field_changes: vec![FieldChange {
                    field_name: "description".into(),
                    old_value: None,
                    new_value: Some("first line\nsecond\tline → value".into()),
                }],
            }],
            summary: DiffSummary {
                total_old: 0,
                total_new: 1,
                added: 0,
                removed: 0,
                changed: 1,
                unchanged: 0,
                uuid_changes: 0,
            },
        };

        let output = format_diff_report(&report);
        let change_line = output.lines().find(|line| line.contains("description:")).unwrap();
        assert!(change_line.contains("(absent)"));
        assert!(change_line.contains("first line second line \\u{2192} value"));
    }

    #[test]
    fn one_line_truncates_long_values() {
        let value = "x".repeat(201);
        let rendered = one_line(&value, 200);
        assert!(rendered.ends_with("[...truncated]"));
        assert!(!rendered.contains('\n'));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "diff summary added count must match entries")]
    fn format_diff_rejects_desynchronized_summary() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::Added {
                control_id: "AC-1".into(),
                new_uuid: Some("uuid".into()),
            }],
            summary: DiffSummary {
                total_old: 0,
                total_new: 1,
                added: 0,
                removed: 0,
                changed: 0,
                unchanged: 0,
                uuid_changes: 0,
            },
        };
        let _ = format_diff_report(&report);
    }

    // --- US2: UUID Stability Changes formatter tests (T020) ---

    #[test]
    fn test_format_uuid_stability_section_populated() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::UuidChanged {
                control_id: "POL-AC-001".into(),
                old_uuid: Some("old-uuid-123".into()),
                new_uuid: Some("new-uuid-456".into()),
            }],
            summary: DiffSummary {
                total_old: 1,
                total_new: 1,
                added: 0,
                removed: 0,
                changed: 0,
                unchanged: 0,
                uuid_changes: 1,
            },
        };
        let output = format_diff_report(&report);
        assert!(output.contains("UUID Stability Changes (1)"));
        assert!(output.contains("! POL-AC-001"));
        assert!(output.contains("old-uuid-123"));
        assert!(output.contains("new-uuid-456"));
    }

    #[test]
    fn test_format_uuid_stability_section_none() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::Added { control_id: "POL-AC-001".into(), new_uuid: None }],
            summary: DiffSummary {
                total_old: 0,
                total_new: 1,
                added: 1,
                removed: 0,
                changed: 0,
                unchanged: 0,
                uuid_changes: 0,
            },
        };
        let output = format_diff_report(&report);
        assert!(output.contains("UUID Stability Changes (0)"));
        let uuid_section_start = output.find("UUID Stability Changes").unwrap();
        let after_uuid = &output[uuid_section_start..];
        assert!(after_uuid.contains("(none)"));
    }

    #[test]
    fn test_empty_sections_show_none() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::Added { control_id: "POL-AC-001".into(), new_uuid: None }],
            summary: DiffSummary {
                total_old: 0,
                total_new: 1,
                added: 1,
                removed: 0,
                changed: 0,
                unchanged: 0,
                uuid_changes: 0,
            },
        };
        let output = format_diff_report(&report);
        assert!(output.contains("(none)"));
    }
}

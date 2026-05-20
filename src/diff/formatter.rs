use std::fmt::Write;

use super::types::{DiffEntry, DiffReport};

/// Format a `DiffReport` as a human-readable plain-text report.
///
/// Renders added, removed, modified, and unchanged control counts, followed
/// by per-control field-level change details.
#[must_use]
pub fn format_diff_report(report: &DiffReport) -> String {
    let mut out = String::new();

    // Header
    writeln!(out, "OSCAL Diff Report").unwrap();
    writeln!(out, "=================").unwrap();
    writeln!(out, "Old: {}  ({})", report.old_file, report.artifact_type).unwrap();
    writeln!(out, "New: {}  ({})", report.new_file, report.artifact_type).unwrap();
    writeln!(out).unwrap();

    // Summary
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

    if !s.has_changes() {
        writeln!(out, "No differences found.").unwrap();
        return out;
    }

    format_added_section(&mut out, report);
    format_changed_section(&mut out, report);
    format_removed_section(&mut out, report);
    format_uuid_section(&mut out, report);

    out
}

fn write_section_heading(out: &mut String, title: &str) {
    writeln!(out, "{title}").unwrap();
    writeln!(out, "{}", "\u{2500}".repeat(title.len())).unwrap();
}

fn format_added_section(out: &mut String, report: &DiffReport) {
    let added: Vec<_> =
        report.entries.iter().filter(|e| matches!(e, DiffEntry::Added { .. })).collect();
    write_section_heading(out, &format!("Added ({})", added.len()));
    if added.is_empty() {
        writeln!(out, "  (none)").unwrap();
    } else {
        for entry in &added {
            if let DiffEntry::Added { control_id, new_uuid } = entry {
                writeln!(out, "  + {control_id}  [uuid: {new_uuid}]").unwrap();
            }
        }
    }
    writeln!(out).unwrap();
}

fn format_changed_section(out: &mut String, report: &DiffReport) {
    let changed: Vec<_> =
        report.entries.iter().filter(|e| matches!(e, DiffEntry::Changed { .. })).collect();
    write_section_heading(out, &format!("Changed ({})", changed.len()));
    if changed.is_empty() {
        writeln!(out, "  (none)").unwrap();
    } else {
        for entry in &changed {
            if let DiffEntry::Changed {
                control_id,
                old_uuid,
                new_uuid,
                uuid_changed,
                field_changes,
            } = entry
            {
                writeln!(out, "  ~ {control_id}").unwrap();
                for fc in field_changes {
                    writeln!(
                        out,
                        "      {}: \"{}\"  \u{2192}  \"{}\"",
                        fc.field_name, fc.old_value, fc.new_value
                    )
                    .unwrap();
                }
                if *uuid_changed {
                    writeln!(out, "      [UUID: {old_uuid} \u{2192} {new_uuid}]").unwrap();
                }
            }
        }
    }
    writeln!(out).unwrap();
}

fn format_removed_section(out: &mut String, report: &DiffReport) {
    let removed: Vec<_> =
        report.entries.iter().filter(|e| matches!(e, DiffEntry::Removed { .. })).collect();
    write_section_heading(out, &format!("Removed ({})", removed.len()));
    if removed.is_empty() {
        writeln!(out, "  (none)").unwrap();
    } else {
        for entry in &removed {
            if let DiffEntry::Removed { control_id, old_uuid } = entry {
                writeln!(out, "  - {control_id}  [uuid: {old_uuid}]").unwrap();
            }
        }
    }
    writeln!(out).unwrap();
}

fn format_uuid_section(out: &mut String, report: &DiffReport) {
    let uuid_changed: Vec<_> =
        report.entries.iter().filter(|e| matches!(e, DiffEntry::UuidChanged { .. })).collect();
    write_section_heading(out, &format!("UUID Stability Changes ({})", uuid_changed.len()));
    if uuid_changed.is_empty() {
        writeln!(out, "  (none)").unwrap();
    } else {
        for entry in &uuid_changed {
            if let DiffEntry::UuidChanged { control_id, old_uuid, new_uuid } = entry {
                writeln!(out, "  ! {control_id}  {old_uuid}  \u{2192}  {new_uuid}").unwrap();
            }
        }
    }
    writeln!(out).unwrap();
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
                DiffEntry::Added { control_id: "POL-AC-002".into(), new_uuid: "uuid-new".into() },
                DiffEntry::Removed { control_id: "POL-AC-003".into(), old_uuid: "uuid-old".into() },
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
                old_uuid: String::new(),
                new_uuid: String::new(),
                uuid_changed: false,
                field_changes: vec![FieldChange {
                    field_name: "title".into(),
                    old_value: "Old title".into(),
                    new_value: "New title".into(),
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

    // --- US2: UUID Stability Changes formatter tests (T020) ---

    #[test]
    fn test_format_uuid_stability_section_populated() {
        let report = DiffReport {
            old_file: "old.json".into(),
            new_file: "new.json".into(),
            artifact_type: ArtifactType::Catalog,
            entries: vec![DiffEntry::UuidChanged {
                control_id: "POL-AC-001".into(),
                old_uuid: "old-uuid-123".into(),
                new_uuid: "new-uuid-456".into(),
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
            entries: vec![DiffEntry::Added {
                control_id: "POL-AC-001".into(),
                new_uuid: String::new(),
            }],
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
            entries: vec![DiffEntry::Added {
                control_id: "POL-AC-001".into(),
                new_uuid: String::new(),
            }],
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

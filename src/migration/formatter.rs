use std::fmt::Write;

use super::types::{InventoryRequirement, MigrationReport};
use crate::error::ForgeError;

/// Serialize a migration report using the versioned JSON contract.
///
/// # Errors
///
/// Returns [`ForgeError::MigrationError`] if serialization fails.
pub fn format_json(report: &MigrationReport) -> Result<String, ForgeError> {
    let mut output = serde_json::to_string_pretty(report).map_err(|error| {
        ForgeError::MigrationError(format!("unable to serialize report: {error}"))
    })?;
    output.push('\n');
    Ok(output)
}

#[must_use]
pub fn format_text(report: &MigrationReport) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "FORGE policy migration report");
    let _ = writeln!(
        output,
        "old: {} [{}; sha256={}]",
        escape_controls(&report.old_source.label),
        report.old_source.format.as_str(),
        report.old_source.sha256
    );
    let _ = writeln!(
        output,
        "new: {} [{}; sha256={}]",
        escape_controls(&report.new_source.label),
        report.new_source.format.as_str(),
        report.new_source.sha256
    );
    if report.old_source.location_basis != super::types::LocationBasis::SourceLine
        || report.new_source.location_basis != super::types::LocationBasis::SourceLine
    {
        let _ = writeln!(
            output,
            "note: PDF/DOCX lines are normalized extracted-text lines, not native page or paragraph coordinates"
        );
    }
    let summary = &report.summary;
    let _ = writeln!(
        output,
        "summary: old={} new={} unchanged-entries={} declared-successor-entries={} declared-split-entries={} declared-merge-entries={} observed-id-change-entries={} substantive-candidate-entries={} atomization-candidate-entries={} ambiguity-groups={} retired-entries={} added-entries={}",
        summary.total_old,
        summary.total_new,
        summary.unchanged,
        summary.declared_successors,
        summary.declared_splits,
        summary.declared_merges,
        summary.observed_id_changes,
        summary.substantive_change_candidates,
        summary.atomization_change_candidates,
        summary.ambiguity_groups,
        summary.retired,
        summary.added
    );
    write_outcome_counts(&mut output, "old outcomes", &summary.old_requirements);
    write_outcome_counts(&mut output, "new outcomes", &summary.new_requirements);

    for entry in &report.entries {
        let _ = writeln!(output, "\n{}", entry.classification.as_str());
        let evidence =
            entry.evidence.iter().map(|evidence| evidence.as_str()).collect::<Vec<_>>().join(",");
        let _ = writeln!(output, "  evidence: {evidence}");
        let _ = writeln!(
            output,
            "  confidence: {}; approval: {}",
            entry.confidence_basis.as_str(),
            entry.approval_status.as_str()
        );
        if let Some(declaration) = &entry.declaration {
            let _ = writeln!(
                output,
                "  declared-by: {}; declared-at: {}; rationale: {}",
                escape_controls(&declaration.approved_by),
                escape_controls(&declaration.approved_at),
                escape_controls(&declaration.rationale)
            );
        }
        for item in &entry.old {
            write_item(&mut output, "old", item);
        }
        for item in &entry.new {
            write_item(&mut output, "new", item);
        }
    }
    output
}

fn write_outcome_counts(
    output: &mut String,
    label: &str,
    counts: &super::types::MigrationOutcomeCounts,
) {
    let _ = writeln!(
        output,
        "{label}: unchanged={} declared-successor={} declared-split={} declared-merge={} observed-id-change={} substantive-candidate={} atomization-candidate={} ambiguous={} retired={} added={} total={}",
        counts.unchanged,
        counts.declared_successors,
        counts.declared_splits,
        counts.declared_merges,
        counts.observed_id_changes,
        counts.substantive_change_candidates,
        counts.atomization_change_candidates,
        counts.ambiguous,
        counts.retired,
        counts.added,
        counts.total()
    );
}

fn write_item(output: &mut String, side: &str, item: &InventoryRequirement) {
    let _ = writeln!(
        output,
        "  {side}: {} text-sha256={} {}:{} atom={} section={}",
        item.stable_id,
        item.normalized_text_sha256,
        escape_controls(&item.location.file_label),
        item.location.line,
        item.location.atom_index,
        escape_controls(&item.location.section_path)
    );
}

fn escape_controls(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::escape_controls;

    #[test]
    fn text_output_escapes_terminal_controls() {
        assert_eq!(escape_controls("safe\u{1b}[31m"), "safe\\u{1b}[31m");
    }
}

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
        format_name(report.old_source.format),
        report.old_source.sha256
    );
    let _ = writeln!(
        output,
        "new: {} [{}; sha256={}]",
        escape_controls(&report.new_source.label),
        format_name(report.new_source.format),
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
        "summary: old={} new={} unchanged-entries={} observed-id-change-entries={} substantive-candidate-entries={} atomization-candidate-entries={} ambiguity-groups={} retired-entries={} added-entries={}",
        summary.total_old,
        summary.total_new,
        summary.unchanged,
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
        let _ = writeln!(output, "\n{}", classification_name(entry.classification));
        let evidence = entry
            .evidence
            .iter()
            .map(|evidence| evidence_name(*evidence))
            .collect::<Vec<_>>()
            .join(",");
        let _ = writeln!(output, "  evidence: {evidence}");
        let _ = writeln!(
            output,
            "  confidence: {}; approval: {}",
            confidence_name(entry.confidence_basis),
            approval_name(entry.approval_status)
        );
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
        "{label}: unchanged={} observed-id-change={} substantive-candidate={} atomization-candidate={} ambiguous={} retired={} added={} total={}",
        counts.unchanged,
        counts.observed_id_changes,
        counts.substantive_change_candidates,
        counts.atomization_change_candidates,
        counts.ambiguous,
        counts.retired,
        counts.added,
        counts.total()
    );
}

const fn classification_name(classification: super::types::Classification) -> &'static str {
    use super::types::Classification;
    match classification {
        Classification::Unchanged => "unchanged",
        Classification::ObservedIdChange => "observed_id_change",
        Classification::SubstantiveChangeCandidate => "substantive_change_candidate",
        Classification::AtomizationChangeCandidate => "atomization_change_candidate",
        Classification::Ambiguous => "ambiguous",
        Classification::Retired => "retired",
        Classification::Added => "added",
    }
}

const fn evidence_name(evidence: super::types::EvidenceCode) -> &'static str {
    use super::types::EvidenceCode;
    match evidence {
        EvidenceCode::ExactId => "exact_id",
        EvidenceCode::UniqueNormalizedText => "unique_normalized_text",
        EvidenceCode::SameLocator => "same_locator",
        EvidenceCode::DuplicateNormalizedText => "duplicate_normalized_text",
        EvidenceCode::CompetingLocator => "competing_locator",
        EvidenceCode::SourceFileChanged => "source_file_changed",
        EvidenceCode::SectionPathChanged => "section_path_changed",
        EvidenceCode::SourceLineChanged => "source_line_changed",
        EvidenceCode::AtomIndexChanged => "atom_index_changed",
    }
}

const fn confidence_name(confidence: super::types::ConfidenceBasis) -> &'static str {
    use super::types::ConfidenceBasis;
    match confidence {
        ConfidenceBasis::Exact => "exact",
        ConfidenceBasis::Candidate => "candidate",
        ConfidenceBasis::Unresolved => "unresolved",
        ConfidenceBasis::Unmatched => "unmatched",
    }
}

const fn approval_name(approval: super::types::ApprovalStatus) -> &'static str {
    match approval {
        super::types::ApprovalStatus::NotRequired => "not_required",
        super::types::ApprovalStatus::NotApproved => "not_approved",
    }
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

const fn format_name(format: super::types::InputFormat) -> &'static str {
    match format {
        super::types::InputFormat::Markdown => "markdown",
        super::types::InputFormat::Pdf => "pdf",
        super::types::InputFormat::Docx => "docx",
    }
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

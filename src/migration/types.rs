use serde::Serialize;

/// Versioned machine-readable migration report contract.
pub const MIGRATION_REPORT_SCHEMA_VERSION: &str = "forge.migration-report/1";

/// Raw source-policy provenance included in every report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceProvenance {
    pub label: String,
    pub format: InputFormat,
    pub sha256: String,
    pub location_basis: LocationBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum InputFormat {
    Markdown,
    Pdf,
    Docx,
}

impl InputFormat {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::Pdf => "pdf",
            Self::Docx => "docx",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationBasis {
    SourceLine,
    NormalizedExtractedTextLine,
}

/// Audit location and stable-ID seed fields for one atomized requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementLocation {
    pub file_label: String,
    pub section_path: String,
    pub section_title: String,
    pub line: usize,
    pub line_basis: LocationBasis,
    pub atom_index: usize,
}

/// One requirement in a migration inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InventoryRequirement {
    pub stable_id: String,
    pub normalized_text_sha256: String,
    pub location: RequirementLocation,
    #[serde(skip)]
    pub(crate) normalized_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Unchanged,
    ObservedIdChange,
    SubstantiveChangeCandidate,
    AtomizationChangeCandidate,
    Ambiguous,
    Retired,
    Added,
}

impl Classification {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::ObservedIdChange => "observed_id_change",
            Self::SubstantiveChangeCandidate => "substantive_change_candidate",
            Self::AtomizationChangeCandidate => "atomization_change_candidate",
            Self::Ambiguous => "ambiguous",
            Self::Retired => "retired",
            Self::Added => "added",
        }
    }

    pub(crate) const fn rank(self) -> u8 {
        match self {
            Self::Unchanged => 0,
            Self::ObservedIdChange => 1,
            Self::SubstantiveChangeCandidate => 2,
            Self::AtomizationChangeCandidate => 3,
            Self::Ambiguous => 4,
            Self::Retired => 5,
            Self::Added => 6,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCode {
    ExactId,
    UniqueNormalizedText,
    SameLocator,
    DuplicateNormalizedText,
    CompetingLocator,
    SourceFileChanged,
    SectionPathChanged,
    SourceLineChanged,
    AtomIndexChanged,
}

impl EvidenceCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactId => "exact_id",
            Self::UniqueNormalizedText => "unique_normalized_text",
            Self::SameLocator => "same_locator",
            Self::DuplicateNormalizedText => "duplicate_normalized_text",
            Self::CompetingLocator => "competing_locator",
            Self::SourceFileChanged => "source_file_changed",
            Self::SectionPathChanged => "section_path_changed",
            Self::SourceLineChanged => "source_line_changed",
            Self::AtomIndexChanged => "atom_index_changed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBasis {
    Exact,
    Candidate,
    Unresolved,
    Unmatched,
}

impl ConfidenceBasis {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Candidate => "candidate",
            Self::Unresolved => "unresolved",
            Self::Unmatched => "unmatched",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    NotRequired,
    NotApproved,
}

impl ApprovalStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not_required",
            Self::NotApproved => "not_approved",
        }
    }
}

/// A top-level, mutually exclusive migration outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationEntry {
    pub classification: Classification,
    pub evidence: Vec<EvidenceCode>,
    pub confidence_basis: ConfidenceBasis,
    pub approval_status: ApprovalStatus,
    pub old: Vec<InventoryRequirement>,
    pub new: Vec<InventoryRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationSummary {
    pub total_old: usize,
    pub total_new: usize,
    /// Counts of top-level report entries. Grouped outcomes count once here.
    pub unchanged: usize,
    pub observed_id_changes: usize,
    pub substantive_change_candidates: usize,
    pub atomization_change_candidates: usize,
    pub ambiguity_groups: usize,
    pub retired: usize,
    pub added: usize,
    /// Old-side requirement counts by outcome; these sum to `total_old`.
    pub old_requirements: MigrationOutcomeCounts,
    /// New-side requirement counts by outcome; these sum to `total_new`.
    pub new_requirements: MigrationOutcomeCounts,
}

/// Number of requirements assigned to each outcome on one side of a migration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MigrationOutcomeCounts {
    pub unchanged: usize,
    pub observed_id_changes: usize,
    pub substantive_change_candidates: usize,
    pub atomization_change_candidates: usize,
    pub ambiguous: usize,
    pub retired: usize,
    pub added: usize,
}

impl MigrationOutcomeCounts {
    #[must_use]
    pub const fn total(&self) -> usize {
        self.unchanged
            + self.observed_id_changes
            + self.substantive_change_candidates
            + self.atomization_change_candidates
            + self.ambiguous
            + self.retired
            + self.added
    }
}

/// Complete deterministic policy migration analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationReport {
    pub schema_version: &'static str,
    pub forge_version: &'static str,
    pub analysis_complete: bool,
    pub old_source: SourceProvenance,
    pub new_source: SourceProvenance,
    pub summary: MigrationSummary,
    pub entries: Vec<MigrationEntry>,
}

impl MigrationReport {
    #[must_use]
    pub fn has_reviewable_changes(&self) -> bool {
        self.entries.iter().any(|entry| {
            entry.classification != Classification::Unchanged
                || entry.evidence.iter().any(|evidence| {
                    matches!(
                        evidence,
                        EvidenceCode::SourceFileChanged
                            | EvidenceCode::SectionPathChanged
                            | EvidenceCode::SourceLineChanged
                            | EvidenceCode::AtomIndexChanged
                    )
                })
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RequirementInventory {
    pub source: SourceProvenance,
    pub requirements: Vec<InventoryRequirement>,
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    fn assert_json_name<T: Serialize>(value: T, name: &str) {
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::Value::String(name.to_string())
        );
    }

    #[test]
    fn canonical_names_match_json_serialization() {
        for value in [InputFormat::Markdown, InputFormat::Pdf, InputFormat::Docx] {
            assert_json_name(value, value.as_str());
        }
        for value in [
            Classification::Unchanged,
            Classification::ObservedIdChange,
            Classification::SubstantiveChangeCandidate,
            Classification::AtomizationChangeCandidate,
            Classification::Ambiguous,
            Classification::Retired,
            Classification::Added,
        ] {
            assert_json_name(value, value.as_str());
        }
        for value in [
            EvidenceCode::ExactId,
            EvidenceCode::UniqueNormalizedText,
            EvidenceCode::SameLocator,
            EvidenceCode::DuplicateNormalizedText,
            EvidenceCode::CompetingLocator,
            EvidenceCode::SourceFileChanged,
            EvidenceCode::SectionPathChanged,
            EvidenceCode::SourceLineChanged,
            EvidenceCode::AtomIndexChanged,
        ] {
            assert_json_name(value, value.as_str());
        }
        for value in [
            ConfidenceBasis::Exact,
            ConfidenceBasis::Candidate,
            ConfidenceBasis::Unresolved,
            ConfidenceBasis::Unmatched,
        ] {
            assert_json_name(value, value.as_str());
        }
        for value in [ApprovalStatus::NotRequired, ApprovalStatus::NotApproved] {
            assert_json_name(value, value.as_str());
        }
    }
}

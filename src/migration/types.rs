use serde::Serialize;

/// Versioned machine-readable migration report contract.
pub const MIGRATION_REPORT_SCHEMA_VERSION: &str = "forge.migration-report/1";

/// Raw source-policy provenance included in every report.
///
/// `sha256` is a 64-character lowercase hexadecimal SHA-256 digest of the raw source bytes.
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
#[derive(Debug, Clone, Serialize)]
pub struct InventoryRequirement {
    pub stable_id: String,
    pub normalized_text_sha256: String,
    pub location: RequirementLocation,
    #[serde(skip)]
    pub(crate) normalized_text: String,
}

impl PartialEq for InventoryRequirement {
    fn eq(&self, other: &Self) -> bool {
        self.stable_id == other.stable_id
            && self.normalized_text_sha256 == other.normalized_text_sha256
            && self.location == other.location
    }
}

impl Eq for InventoryRequirement {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Unchanged,
    DeclaredSuccessor,
    DeclaredSplit,
    DeclaredMerge,
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
            Self::DeclaredSuccessor => "declared_successor",
            Self::DeclaredSplit => "declared_split",
            Self::DeclaredMerge => "declared_merge",
            Self::ObservedIdChange => "observed_id_change",
            Self::SubstantiveChangeCandidate => "substantive_change_candidate",
            Self::AtomizationChangeCandidate => "atomization_change_candidate",
            Self::Ambiguous => "ambiguous",
            Self::Retired => "retired",
            Self::Added => "added",
        }
    }

    /// Declaration order defines precedence when report entries are sorted.
    pub(crate) const fn rank(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCode {
    ExactId,
    ReviewerDeclaration,
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
            Self::ReviewerDeclaration => "reviewer_declaration",
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

    #[must_use]
    pub const fn indicates_location_drift(self) -> bool {
        match self {
            Self::ExactId
            | Self::ReviewerDeclaration
            | Self::UniqueNormalizedText
            | Self::SameLocator
            | Self::DuplicateNormalizedText
            | Self::CompetingLocator => false,
            Self::SourceFileChanged
            | Self::SectionPathChanged
            | Self::SourceLineChanged
            | Self::AtomIndexChanged => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceBasis {
    Exact,
    Declared,
    Candidate,
    Unresolved,
    Unmatched,
}

impl ConfidenceBasis {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Declared => "declared",
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
    Declared,
}

impl ApprovalStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not_required",
            Self::NotApproved => "not_approved",
            Self::Declared => "declared",
        }
    }
}

/// Reviewer evidence preserved verbatim for a declared identity relationship.
///
/// `approved_by` is personal data and MUST NOT be emitted to logs or non-report diagnostics.
/// `approved_at` is an RFC 3339 UTC timestamp validated when the declaration is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclarationEvidence {
    pub approved_by: String,
    pub approved_at: String,
    pub rationale: String,
}

/// A top-level, mutually exclusive migration outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationEntry {
    pub classification: Classification,
    pub evidence: Vec<EvidenceCode>,
    pub confidence_basis: ConfidenceBasis,
    pub approval_status: ApprovalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<DeclarationEvidence>,
    pub old: Vec<InventoryRequirement>,
    pub new: Vec<InventoryRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationSummary {
    pub total_old: usize,
    pub total_new: usize,
    /// Counts of top-level report entries. Grouped outcomes count once here.
    pub unchanged: usize,
    pub declared_successors: usize,
    pub declared_splits: usize,
    pub declared_merges: usize,
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

impl MigrationSummary {
    /// Validate the requirement-side totals promised by this report contract.
    pub(crate) const fn validate(&self) -> Result<(), &'static str> {
        if self.old_requirements.total() != self.total_old {
            return Err("old requirement outcome counts do not sum to total_old");
        }
        if self.new_requirements.total() != self.total_new {
            return Err("new requirement outcome counts do not sum to total_new");
        }
        Ok(())
    }
}

/// Number of requirements assigned to each outcome on one side of a migration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MigrationOutcomeCounts {
    pub unchanged: usize,
    pub declared_successors: usize,
    pub declared_splits: usize,
    pub declared_merges: usize,
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
            + self.declared_successors
            + self.declared_splits
            + self.declared_merges
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
    /// Whether PRD 53 M-20 requires a review signal. Any source-location
    /// change is intentionally reviewable, including source-line or atom-index
    /// drift that does not alter normalized requirement prose.
    #[must_use]
    pub fn has_reviewable_changes(&self) -> bool {
        self.entries.iter().any(|entry| {
            entry.classification != Classification::Unchanged
                || entry.evidence.iter().copied().any(EvidenceCode::indicates_location_drift)
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
    fn inventory_requirement_equality_matches_serialized_identity() {
        let requirement = InventoryRequirement {
            stable_id: "AC-1".to_string(),
            normalized_text_sha256: "abc123".to_string(),
            location: RequirementLocation {
                file_label: "policy.md".to_string(),
                section_path: "Access Control".to_string(),
                section_title: "Access Control".to_string(),
                line: 1,
                line_basis: LocationBasis::SourceLine,
                atom_index: 0,
            },
            normalized_text: "first normalized form".to_string(),
        };
        let mut different_normalized_text = requirement.clone();
        different_normalized_text.normalized_text = "second normalized form".to_string();

        assert_eq!(requirement, different_normalized_text);
        assert_eq!(
            serde_json::to_value(&requirement).unwrap(),
            serde_json::to_value(&different_normalized_text).unwrap(),
        );
    }

    #[test]
    fn canonical_names_match_json_serialization() {
        for value in [InputFormat::Markdown, InputFormat::Pdf, InputFormat::Docx] {
            assert_json_name(value, value.as_str());
        }
        for value in [
            Classification::Unchanged,
            Classification::DeclaredSuccessor,
            Classification::DeclaredSplit,
            Classification::DeclaredMerge,
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
            EvidenceCode::ReviewerDeclaration,
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
            ConfidenceBasis::Declared,
            ConfidenceBasis::Candidate,
            ConfidenceBasis::Unresolved,
            ConfidenceBasis::Unmatched,
        ] {
            assert_json_name(value, value.as_str());
        }
        for value in
            [ApprovalStatus::NotRequired, ApprovalStatus::NotApproved, ApprovalStatus::Declared]
        {
            assert_json_name(value, value.as_str());
        }
    }

    #[test]
    fn migration_summary_validates_requirement_totals() {
        let summary = MigrationSummary {
            total_old: 1,
            total_new: 1,
            unchanged: 1,
            declared_successors: 0,
            declared_splits: 0,
            declared_merges: 0,
            observed_id_changes: 0,
            substantive_change_candidates: 0,
            atomization_change_candidates: 0,
            ambiguity_groups: 0,
            retired: 0,
            added: 0,
            old_requirements: MigrationOutcomeCounts {
                unchanged: 1,
                ..MigrationOutcomeCounts::default()
            },
            new_requirements: MigrationOutcomeCounts {
                unchanged: 1,
                ..MigrationOutcomeCounts::default()
            },
        };
        assert_eq!(summary.validate(), Ok(()));

        let mut invalid = summary;
        invalid.total_new = 2;
        assert_eq!(
            invalid.validate(),
            Err("new requirement outcome counts do not sum to total_new")
        );
    }

    #[test]
    fn location_drift_classifies_every_evidence_code() {
        for (evidence, indicates_drift) in [
            (EvidenceCode::ExactId, false),
            (EvidenceCode::ReviewerDeclaration, false),
            (EvidenceCode::UniqueNormalizedText, false),
            (EvidenceCode::SameLocator, false),
            (EvidenceCode::DuplicateNormalizedText, false),
            (EvidenceCode::CompetingLocator, false),
            (EvidenceCode::SourceFileChanged, true),
            (EvidenceCode::SectionPathChanged, true),
            (EvidenceCode::SourceLineChanged, true),
            (EvidenceCode::AtomIndexChanged, true),
        ] {
            assert_eq!(evidence.indicates_location_drift(), indicates_drift);
        }
    }

    #[test]
    fn classification_rank_tracks_declaration_order() {
        assert_eq!(Classification::Unchanged.rank(), 0);
        assert_eq!(Classification::DeclaredSuccessor.rank(), 1);
        assert_eq!(Classification::Added.rank(), 9);
    }
}

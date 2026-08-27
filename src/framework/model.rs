//! Deterministic framework change and review-queue report types.

use serde::{Deserialize, Serialize};

use crate::mapping::inventory::ResourceEvidence;

pub const REPORT_SCHEMA_VERSION: &str = "forge.framework-impact-report/1";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ReportStatus {
    Complete,
}

impl ReportStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImpactReport {
    pub schema_version: String,
    pub status: ReportStatus,
    pub old: ResourceEvidence,
    pub new: ResourceEvidence,
    pub summary: ChangeSummary,
    pub filters: ImpactFilters,
    pub matched_findings: usize,
    pub changes: Vec<ControlChange>,
    pub findings: Vec<ImpactFinding>,
    #[serde(skip)]
    pub(crate) filtered_out_findings: Vec<ImpactFinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prior_only_dispositions: Vec<super::disposition::DispositionRecord>,
}

/// Optional exact-match filters for review finding details.
///
/// Change counts describe the complete validated analysis, while disposition counts describe only
/// emitted findings. Multiple filters are combined with AND semantics and never affect gate
/// evaluation.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ImpactFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_state: Option<crate::applicability::manifest::DecisionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<FindingPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl ImpactFilters {
    #[must_use]
    pub fn matches(&self, finding: &ImpactFinding) -> bool {
        self.group.as_ref().is_none_or(|group| finding.framework_groups.contains(group))
            && self.decision_state.is_none_or(|state| finding.prior_decision_state == Some(state))
            && self
                .policy_source
                .as_ref()
                .is_none_or(|source| finding.policy_sources.contains(source))
            && self.priority.is_none_or(|priority| priority == finding.priority)
            && self.owner.as_ref().is_none_or(|owner| finding.owner.as_ref() == Some(owner))
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.group.is_none()
            && self.decision_state.is_none()
            && self.policy_source.is_none()
            && self.priority.is_none()
            && self.owner.is_none()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChangeSummary {
    pub old_controls: usize,
    pub new_controls: usize,
    pub added: usize,
    pub removed: usize,
    pub content_changed: usize,
    pub identity_migrated: usize,
    pub unchanged: usize,
    pub findings: usize,
    pub blocking: usize,
    pub review_required: usize,
    pub informational: usize,
    pub dispositioned_resolved: usize,
    pub dispositioned_accepted_risk: usize,
    pub dispositioned_still_open: usize,
    pub undispositioned: usize,
}

impl ChangeSummary {
    #[must_use]
    pub const fn rows(&self) -> [(&'static str, usize); 15] {
        [
            ("Old controls", self.old_controls),
            ("New controls", self.new_controls),
            ("Added", self.added),
            ("Removed", self.removed),
            ("Content changed", self.content_changed),
            ("Identity migrated", self.identity_migrated),
            ("Unchanged", self.unchanged),
            ("Findings", self.findings),
            ("Blocking", self.blocking),
            ("Review required", self.review_required),
            ("Informational", self.informational),
            ("Resolved", self.dispositioned_resolved),
            ("Accepted risk", self.dispositioned_accepted_risk),
            ("Still open", self.dispositioned_still_open),
            ("Undispositioned", self.undispositioned),
        ]
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlChange {
    pub subject_id: String,
    pub change_class: ChangeClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_sha256: Option<String>,
    pub old_subjects: Vec<SubjectFingerprint>,
    pub new_subjects: Vec<SubjectFingerprint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration: Option<IdentityMigrationEvidence>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubjectFingerprint {
    pub id: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdentityMigrationEvidence {
    pub relationship: crate::migration::RelationshipType,
    pub approved_by: String,
    pub approved_at: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeClass {
    Added,
    Removed,
    ContentChanged,
    IdentityMigrated,
    Unchanged,
}

impl ChangeClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::ContentChanged => "content-changed",
            Self::IdentityMigrated => "identity-migrated",
            Self::Unchanged => "unchanged",
        }
    }
}

/// A review finding scoped to this report's exact old/new resource-evidence pair.
///
/// `finding_id` is stable only within that pair. Disposition loading verifies the prior report's
/// evidence before applying IDs, so identifiers from a different comparison are never reusable.
/// A review finding scoped to this report's exact old/new resource-evidence pair.
///
/// `finding_id` is stable only within that pair. Disposition loading verifies the prior report's
/// evidence before applying IDs, so identifiers from a different comparison are never reusable.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImpactFinding {
    pub finding_id: String,
    pub priority: FindingPriority,
    pub reason_code: ReasonCode,
    pub required_action: RequiredAction,
    pub subject_id: String,
    pub change_class: ChangeClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_sha256: Option<String>,
    pub old_subjects: Vec<SubjectFingerprint>,
    pub new_subjects: Vec<SubjectFingerprint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration: Option<IdentityMigrationEvidence>,
    pub dependency_path: Vec<String>,
    pub framework_groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_artifact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_resource_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_gap_classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prior_decision_state: Option<crate::applicability::manifest::DecisionState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_sources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disposition: Option<super::disposition::DispositionRecord>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum FindingPriority {
    Blocking,
    ReviewRequired,
    Informational,
}

impl FindingPriority {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocking => "blocking",
            Self::ReviewRequired => "review-required",
            Self::Informational => "informational",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ReasonCode {
    ControlAdded,
    ControlRemoved,
    ControlContentChanged,
    MappingReferenceRemoved,
    MappingSubjectChanged,
    ApplicabilityDecisionRemoved,
    ApplicabilityDecisionChanged,
    ApplicabilityDecisionMigrated,
    IdentityMigrationDeclared,
    MappingSubjectMigrated,
    ResourceMetadataChanged,
    MigrationEvidenceMissing,
}

impl ReasonCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ControlAdded => "control_added",
            Self::ControlRemoved => "control_removed",
            Self::ControlContentChanged => "control_content_changed",
            Self::MappingReferenceRemoved => "mapping_reference_removed",
            Self::MappingSubjectChanged => "mapping_subject_changed",
            Self::ApplicabilityDecisionRemoved => "applicability_decision_removed",
            Self::ApplicabilityDecisionChanged => "applicability_decision_changed",
            Self::ApplicabilityDecisionMigrated => "applicability_decision_migrated",
            Self::IdentityMigrationDeclared => "identity_migration_declared",
            Self::MappingSubjectMigrated => "mapping_subject_migrated",
            Self::ResourceMetadataChanged => "resource_metadata_changed",
            Self::MigrationEvidenceMissing => "migration_evidence_missing",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RequiredAction {
    ReviewApplicability,
    ReviewFrameworkRemoval,
    ReviewControlChange,
    RepairOrApproveMapping,
    ReapproveMappingRationale,
    ReviewApplicabilityDecision,
    ReviewIdentityMigration,
    ReviewResourceMetadata,
}

impl RequiredAction {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReviewApplicability => "review-applicability",
            Self::ReviewFrameworkRemoval => "review-framework-removal",
            Self::ReviewControlChange => "review-control-change",
            Self::RepairOrApproveMapping => "repair-or-approve-mapping",
            Self::ReapproveMappingRationale => "reapprove-mapping-rationale",
            Self::ReviewApplicabilityDecision => "review-applicability-decision",
            Self::ReviewIdentityMigration => "review-identity-migration",
            Self::ReviewResourceMetadata => "review-resource-metadata",
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{
        ChangeClass, ChangeSummary, FindingPriority, ImpactFilters, ImpactReport,
        REPORT_SCHEMA_VERSION, ReasonCode, ReportStatus, RequiredAction,
    };
    use crate::mapping::inventory::ResourceEvidence;
    use crate::mapping::manifest::ResourceType;
    #[test]
    fn summary_rows_cover_each_summary_metric() {
        assert_eq!(ChangeSummary::default().rows().len(), 15);
    }

    #[test]
    fn impact_report_round_trips_through_json() {
        let resource = ResourceEvidence {
            resource_type: ResourceType::Catalog,
            href: "catalog.json".to_string(),
            raw_sha256: "a".repeat(64),
            root_uuid: "11111111-1111-4111-8111-111111111111".to_string(),
            document_version: "1.0.0".to_string(),
            oscal_version: crate::oscal::OSCAL_VERSION.to_string(),
            resolved_catalog_sha256: None,
        };
        let report = ImpactReport {
            schema_version: REPORT_SCHEMA_VERSION.to_string(),
            status: ReportStatus::Complete,
            old: resource.clone(),
            new: resource,
            summary: ChangeSummary::default(),
            filters: ImpactFilters::default(),
            matched_findings: 0,
            changes: Vec::new(),
            findings: Vec::new(),
            filtered_out_findings: Vec::new(),
            prior_only_dispositions: Vec::new(),
        };

        let parsed: ImpactReport =
            serde_json::from_str(&serde_json::to_string(&report).expect("serialize impact report"))
                .expect("deserialize impact report");

        assert_eq!(parsed.schema_version, REPORT_SCHEMA_VERSION);
        assert_eq!(parsed.old, report.old);
        assert_eq!(parsed.new, report.new);
        assert_eq!(parsed.status, ReportStatus::Complete);
    }

    #[test]
    fn enum_as_str_values_match_their_serialized_contracts() {
        assert_serialized_string(&ReportStatus::Complete, ReportStatus::Complete.as_str());
        for value in [
            ChangeClass::Added,
            ChangeClass::Removed,
            ChangeClass::ContentChanged,
            ChangeClass::IdentityMigrated,
            ChangeClass::Unchanged,
        ] {
            assert_serialized_string(&value, value.as_str());
        }
        for value in [
            FindingPriority::Blocking,
            FindingPriority::ReviewRequired,
            FindingPriority::Informational,
        ] {
            assert_serialized_string(&value, value.as_str());
        }
        for value in [
            ReasonCode::ControlAdded,
            ReasonCode::ControlRemoved,
            ReasonCode::ControlContentChanged,
            ReasonCode::MappingReferenceRemoved,
            ReasonCode::MappingSubjectChanged,
            ReasonCode::ApplicabilityDecisionRemoved,
            ReasonCode::ApplicabilityDecisionChanged,
            ReasonCode::ApplicabilityDecisionMigrated,
            ReasonCode::IdentityMigrationDeclared,
            ReasonCode::MappingSubjectMigrated,
            ReasonCode::ResourceMetadataChanged,
            ReasonCode::MigrationEvidenceMissing,
        ] {
            assert_serialized_string(&value, value.as_str());
        }
        for value in [
            RequiredAction::ReviewApplicability,
            RequiredAction::ReviewFrameworkRemoval,
            RequiredAction::ReviewControlChange,
            RequiredAction::RepairOrApproveMapping,
            RequiredAction::ReapproveMappingRationale,
            RequiredAction::ReviewApplicabilityDecision,
            RequiredAction::ReviewIdentityMigration,
            RequiredAction::ReviewResourceMetadata,
        ] {
            assert_serialized_string(&value, value.as_str());
        }
    }

    fn assert_serialized_string(value: &impl Serialize, expected: &str) {
        assert_eq!(serde_json::to_value(value).expect("serialize enum"), expected);
    }
}

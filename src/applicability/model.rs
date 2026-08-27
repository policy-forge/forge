//! Deterministic applicability and policy-gap report model.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::manifest::{ApplicabilityManifest, ControlDecision, DecisionState};
use crate::mapping::inventory::{Inventory, ResourceEvidence};
use crate::mapping::manifest::{ReviewerManifest, ReviewerType, SubjectType};

/// Applicability report schema identifier emitted by this release.
pub const REPORT_SCHEMA_VERSION: &str = "forge.applicability-report/1";

/// Accumulated Mapping Collection participation facts for one framework control.
#[derive(Debug, Clone, Default)]
pub struct ControlMappingFacts {
    pub positive_count: usize,
    pub no_relationship_count: usize,
    pub policy_sources: BTreeSet<String>,
}

/// Reviewer evidence preserved from an accepted Mapping Collection.
#[derive(Debug, Clone, Serialize)]
pub struct MappingReviewerEvidence {
    pub uuid: String,
    #[serde(rename = "type")]
    pub reviewer_type: ReviewerType,
    pub name: String,
}

/// Provenance for one accepted OSCAL Mapping Collection.
#[derive(Debug, Clone, Serialize)]
pub struct MappingEvidence {
    pub uuid: String,
    pub raw_sha256: String,
    pub version: String,
    pub oscal_version: String,
    pub reviewed_at: String,
    pub reviewers: Vec<MappingReviewerEvidence>,
    pub source_resources: Vec<ResourceEvidence>,
}

/// Optional report-detail filters. Denominator totals always describe the complete inventory.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReportFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GapClassification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_source: Option<String>,
}

/// Complete deterministic analysis report.
#[derive(Debug, Clone, Serialize)]
pub struct ApplicabilityReport {
    pub schema_version: &'static str,
    pub manifest_sha256: String,
    pub framework: ResourceEvidence,
    pub mapping_collections: Vec<MappingEvidence>,
    pub reviewers: Vec<ReviewerManifest>,
    pub counts: ClassificationCounts,
    pub filters: ReportFilters,
    pub matched_controls: usize,
    pub controls: Vec<ControlResult>,
    pub review_queue: Vec<ReviewQueueItem>,
}

/// Reconciled classification totals.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ClassificationCounts {
    pub total: usize,
    pub applicable_mapped: usize,
    pub applicable_reviewed_no_relationship: usize,
    pub applicable_unmapped: usize,
    pub not_applicable: usize,
    pub deferred: usize,
    pub under_review: usize,
}

/// Exactly one classification for one eligible framework control.
#[derive(Debug, Clone, Serialize)]
pub struct ControlResult {
    pub control_id: String,
    pub groups: Vec<String>,
    pub classification: GapClassification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revisit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub positive_mapping_count: usize,
    pub no_relationship_count: usize,
    pub policy_sources: Vec<String>,
}

/// Closed vocabulary for machine-readable review queue reasons.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewReason {
    ReviewedNoPositiveRelationship,
    NoReviewedMapping,
    DeferredScopeDecision,
    ScopeDecisionRequired,
}

impl ReviewReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReviewedNoPositiveRelationship => "reviewed-no-positive-relationship",
            Self::NoReviewedMapping => "no-reviewed-mapping",
            Self::DeferredScopeDecision => "deferred-scope-decision",
            Self::ScopeDecisionRequired => "scope-decision-required",
        }
    }
}

impl std::fmt::Display for ReviewReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable, machine-readable human review queue entry.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewQueueItem {
    pub control_id: String,
    pub reason_code: ReviewReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revisit_date: Option<String>,
    pub policy_sources: Vec<String>,
}

/// Truthful mapping-participation and human review-state labels.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GapClassification {
    ApplicableMapped,
    ApplicableReviewedNoRelationship,
    ApplicableUnmapped,
    NotApplicable,
    Deferred,
    UnderReview,
}

impl GapClassification {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicableMapped => "applicable-mapped",
            Self::ApplicableReviewedNoRelationship => "applicable-reviewed-no-relationship",
            Self::ApplicableUnmapped => "applicable-unmapped",
            Self::NotApplicable => "not-applicable",
            Self::Deferred => "deferred",
            Self::UnderReview => "under-review",
        }
    }
}

/// Build a sorted, reconciled report from validated inputs.
#[must_use]
pub fn build_report(
    manifest: &ApplicabilityManifest,
    manifest_sha256: String,
    framework: ResourceEvidence,
    inventory: &Inventory,
    mapping_collections: Vec<MappingEvidence>,
    mapping_facts: &BTreeMap<String, ControlMappingFacts>,
    filters: ReportFilters,
) -> ApplicabilityReport {
    let mut mapping_collections = mapping_collections;
    sort_mapping_collections(&mut mapping_collections);
    let decisions: BTreeMap<_, _> = manifest
        .decisions
        .iter()
        .map(|decision| (decision.control_id.as_str(), decision))
        .collect();
    let mut counts = ClassificationCounts::default();
    let all_controls: Vec<_> = inventory
        .ids_of_type(SubjectType::Control)
        .into_iter()
        .map(|control_id| {
            let decision = decisions.get(control_id.as_str()).copied();
            let facts = mapping_facts.get(&control_id).cloned().unwrap_or_default();
            let classification =
                classify(decision, facts.positive_count, facts.no_relationship_count);
            counts.record(classification);
            ControlResult {
                groups: inventory.groups_for_control(&control_id).to_vec(),
                control_id,
                classification,
                reviewer_key: decision.and_then(|item| item.reviewer_key.clone()),
                reviewed_at: decision.and_then(|item| item.reviewed_at.clone()),
                rationale: decision.and_then(|item| item.rationale.clone()),
                revisit_date: decision.and_then(|item| item.revisit_date.clone()),
                note: decision.and_then(|item| item.note.clone()),
                positive_mapping_count: facts.positive_count,
                no_relationship_count: facts.no_relationship_count,
                policy_sources: facts.policy_sources.into_iter().collect(),
            }
        })
        .collect();
    let visible_ids: BTreeSet<_> = all_controls
        .iter()
        .filter(|control| filters.matches(control))
        .map(|control| control.control_id.clone())
        .collect();
    let review_queue = all_controls
        .iter()
        .filter(|control| visible_ids.contains(&control.control_id))
        .filter_map(review_queue_item)
        .collect();
    let controls: Vec<_> = all_controls
        .into_iter()
        .filter(|control| visible_ids.contains(&control.control_id))
        .collect();
    ApplicabilityReport {
        schema_version: REPORT_SCHEMA_VERSION,
        manifest_sha256,
        framework,
        mapping_collections,
        reviewers: manifest.reviewers.clone(),
        counts,
        filters,
        matched_controls: controls.len(),
        controls,
        review_queue,
    }
}

fn sort_mapping_collections(mapping_collections: &mut [MappingEvidence]) {
    mapping_collections.sort_unstable_by(|left, right| left.uuid.cmp(&right.uuid));
}

/// Classify each control by its explicit decision and reviewed mapping evidence.
///
/// Positive mappings take precedence over reviewed no-relationship mappings, and a missing
/// decision is treated as [`DecisionState::UnderReview`].
fn classify(
    decision: Option<&ControlDecision>,
    positive_mapping_count: usize,
    no_relationship_count: usize,
) -> GapClassification {
    match decision.map_or(DecisionState::UnderReview, |item| item.state) {
        DecisionState::Applicable if positive_mapping_count > 0 => {
            GapClassification::ApplicableMapped
        }
        DecisionState::Applicable if no_relationship_count > 0 => {
            GapClassification::ApplicableReviewedNoRelationship
        }
        DecisionState::Applicable => GapClassification::ApplicableUnmapped,
        DecisionState::NotApplicable => GapClassification::NotApplicable,
        DecisionState::Deferred => GapClassification::Deferred,
        DecisionState::UnderReview => GapClassification::UnderReview,
    }
}

fn review_queue_item(control: &ControlResult) -> Option<ReviewQueueItem> {
    let reason_code = match control.classification {
        GapClassification::ApplicableMapped | GapClassification::NotApplicable => return None,
        GapClassification::ApplicableReviewedNoRelationship => {
            ReviewReason::ReviewedNoPositiveRelationship
        }
        GapClassification::ApplicableUnmapped => ReviewReason::NoReviewedMapping,
        GapClassification::Deferred => ReviewReason::DeferredScopeDecision,
        GapClassification::UnderReview => ReviewReason::ScopeDecisionRequired,
    };
    Some(ReviewQueueItem {
        control_id: control.control_id.clone(),
        reason_code,
        owner: control.reviewer_key.clone(),
        revisit_date: control.revisit_date.clone(),
        policy_sources: control.policy_sources.clone(),
    })
}

impl ReportFilters {
    fn matches(&self, control: &ControlResult) -> bool {
        self.group.as_ref().is_none_or(|group| control.groups.contains(group))
            && self
                .control_prefix
                .as_ref()
                .is_none_or(|prefix| control.control_id.starts_with(prefix))
            && self.state.is_none_or(|state| state == control.classification)
            && self
                .reviewer
                .as_ref()
                .is_none_or(|reviewer| control.reviewer_key.as_ref() == Some(reviewer))
            && self
                .policy_source
                .as_ref()
                .is_none_or(|source| control.policy_sources.contains(source))
    }
}

impl ClassificationCounts {
    fn record(&mut self, classification: GapClassification) {
        self.total += 1;
        match classification {
            GapClassification::ApplicableMapped => self.applicable_mapped += 1,
            GapClassification::ApplicableReviewedNoRelationship => {
                self.applicable_reviewed_no_relationship += 1;
            }
            GapClassification::ApplicableUnmapped => self.applicable_unmapped += 1,
            GapClassification::NotApplicable => self.not_applicable += 1,
            GapClassification::Deferred => self.deferred += 1,
            GapClassification::UnderReview => self.under_review += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_reason_serialization_preserves_the_queue_contract() {
        for reason in [
            ReviewReason::ReviewedNoPositiveRelationship,
            ReviewReason::NoReviewedMapping,
            ReviewReason::DeferredScopeDecision,
            ReviewReason::ScopeDecisionRequired,
        ] {
            assert_eq!(
                serde_json::to_value(reason).expect("serialize review reason"),
                reason.as_str()
            );
        }
    }

    #[test]
    fn gap_classification_serialization_matches_display_contract() {
        for classification in [
            GapClassification::ApplicableMapped,
            GapClassification::ApplicableReviewedNoRelationship,
            GapClassification::ApplicableUnmapped,
            GapClassification::NotApplicable,
            GapClassification::Deferred,
            GapClassification::UnderReview,
        ] {
            assert_eq!(
                serde_json::to_value(classification).expect("serialize gap classification"),
                classification.as_str()
            );
        }
    }

    #[test]
    fn mapping_evidence_ordering_is_owned_by_the_report_builder() {
        let mut mapping_collections = vec![mapping_evidence("z"), mapping_evidence("a")];
        sort_mapping_collections(&mut mapping_collections);
        assert_eq!(
            mapping_collections.iter().map(|evidence| evidence.uuid.as_str()).collect::<Vec<_>>(),
            ["a", "z"]
        );
    }

    fn mapping_evidence(uuid: &str) -> MappingEvidence {
        MappingEvidence {
            uuid: uuid.to_string(),
            raw_sha256: "a".repeat(64),
            version: "1.0.0".to_string(),
            oscal_version: "1.2.3".to_string(),
            reviewed_at: "2026-08-26T00:00:00Z".to_string(),
            reviewers: Vec::new(),
            source_resources: Vec::new(),
        }
    }
}

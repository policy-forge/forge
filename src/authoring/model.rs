//! Shared authoring input evidence and deterministic planning contracts.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::manifest::{
    AuthorProject, AuthoringPack, ControlAssignment, Deferral, FamilyAssignment, HumanClause,
    Review, Reviewer, Sensitivity,
};
use crate::applicability::model::ApplicabilityReport;
use crate::mapping::inventory::ResourceEvidence;

pub const PLAN_SCHEMA_VERSION: &str = "forge.authoring-plan/1";

#[derive(Debug, Clone)]
pub struct LoadedAuthorProject {
    pub project: AuthorProject,
    pub pack: AuthoringPack,
    pub baseline_report: ApplicabilityReport,
    pub inputs: Vec<InputFingerprint>,
    pub project_sha256: String,
    pub pack_sha256: String,
    pub report_sha256: String,
    pub clauses: BTreeMap<String, LoadedClause>,
}

#[derive(Debug, Clone)]
pub struct LoadedClause {
    pub source: HumanClause,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InputFingerprint {
    pub role: String,
    pub path: String,
    pub sha256: String,
    pub byte_length: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthoringPlan {
    pub schema_version: String,
    pub project_key: String,
    pub as_of: String,
    pub provenance: PlanProvenance,
    pub counts: GapCounts,
    pub gaps: Vec<GapPlan>,
    pub policies: Vec<PolicyPlan>,
    pub questions: Vec<QuestionEvaluation>,
    pub unresolved_questions: Vec<QuestionEvaluation>,
    pub unresolved_gaps: Vec<GapPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanProvenance {
    pub project_sha256: String,
    pub pack_sha256: String,
    pub report_sha256: String,
    pub framework: ResourceEvidence,
    pub inputs: Vec<InputFingerprint>,
    pub baseline_review: Review,
    pub pack_reviewers: Vec<Reviewer>,
    pub project_reviewers: Vec<Reviewer>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct GapCounts {
    pub total: usize,
    pub assigned: usize,
    pub deferred: usize,
    pub unresolved: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapPlan {
    pub gap_id: String,
    pub control_id: String,
    pub classification: String,
    pub disposition: GapDisposition,
    pub assignments: Vec<GapAssignment>,
    pub deferral: Option<Deferral>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GapAssignment {
    pub policy_key: String,
    pub topic_key: String,
    pub control_assignment: ControlAssignment,
    pub family_assignment: FamilyAssignment,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyPlan {
    pub policy_key: String,
    pub policy_family_key: String,
    pub title: String,
    pub state: DraftState,
    pub sections: Vec<SectionPlan>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionPlan {
    pub topic_key: String,
    pub title: String,
    pub order: u32,
    pub state: DraftState,
    pub gap_ids: Vec<String>,
    pub control_ids: Vec<String>,
    pub assignments: Vec<GapAssignment>,
    pub questions: Vec<QuestionEvaluation>,
    pub clause_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuestionEvaluation {
    pub question_key: String,
    pub question_sha256: String,
    pub required: bool,
    pub owner: String,
    pub sensitivity: Sensitivity,
    pub source_label: String,
    pub answer_key: Option<String>,
    pub answer_sha256: Option<String>,
    pub state: AnswerStatus,
    pub review: Option<Review>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DraftState {
    Planned,
    BlockedContext,
    SkeletonReady,
    HumanDraftPresent,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GapDisposition {
    Assigned,
    Deferred,
    Unresolved,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AnswerStatus {
    Available,
    Missing,
    NoAnswer,
    Stale,
    Expired,
    Invalid,
}

impl DraftState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::BlockedContext => "blocked-context",
            Self::SkeletonReady => "skeleton-ready",
            Self::HumanDraftPresent => "human-draft-present",
        }
    }
}

impl GapDisposition {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assigned => "assigned",
            Self::Deferred => "deferred",
            Self::Unresolved => "unresolved",
        }
    }
}

impl AnswerStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::NoAnswer => "no-answer",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Invalid => "invalid",
        }
    }
}

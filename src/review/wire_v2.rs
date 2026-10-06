//! Separately closed inert Lifecycle exchange declarations.
//! No declaration, digest or loaded currentness label supplies a native owner.
//! Required explicit nulls are enforced by the actual strict schema consumer.

use serde::{Deserialize, Serialize};

pub(crate) use super::wire::{
    AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, DispositionCounts,
    IDENTITY_DISCLAIMER, ItemDisposition, ItemState, MetSeat, RecordedCurrentness,
    RecordedResponse, RequestedAction, ResponseClassification, ReviewPolicy, Reviewer,
    RoleDefinition, Sensitivity, SupersessionReference,
};

/// Fixed separately closed domain token; this is not native approval.
pub(crate) const DOMAIN: &str = "lifecycle-policy-version";
/// Fixed identity-only Lifecycle adapter revision.
pub(crate) const ADAPTER: &str = "forge.lifecycle-review/1";

/// Complete purpose occurrence kind, separate from native validation authority.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceKindV2 {
    /// Full intrinsic current record original.
    LifecycleRecord,
    /// Actual arbitrary original bytes, including empty/non-UTF8 source.
    OpaqueSource,
    /// Generated JSON model/root/UUID identity only.
    GeneratedArtifact,
}

/// Exactly the six recognized native generated object roots.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NativeModelV2 {
    /// Exact catalog object root.
    Catalog,
    /// Exact profile object root; imports remain inert data.
    Profile,
    /// Exact component-definition object root.
    ComponentDefinition,
    /// Exact mapping-collection object root.
    MappingCollection,
    /// Exact system-security-plan object root.
    SystemSecurityPlan,
    /// Exact plan-of-action-and-milestones object root.
    PlanOfActionAndMilestones,
}
impl NativeModelV2 {
    /// Preserve exact native root labels without normalization or schema claims.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Profile => "profile",
            Self::ComponentDefinition => "component-definition",
            Self::MappingCollection => "mapping-collection",
            Self::SystemSecurityPlan => "system-security-plan",
            Self::PlanOfActionAndMilestones => "plan-of-action-and-milestones",
        }
    }
}

/// Lifecycle-only inert domain; old /1 Domain is never widened or cast.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DomainV2 {
    /// One explicitly selected policy-version, with no native transition authority.
    LifecyclePolicyVersion,
}

/// Eight-field declared purpose pin; full equality includes every required field.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourcePinV2 {
    /// Fixed role/ordinal key, never a source path or private native policy key.
    pub(crate) artifact_key: String,
    /// Complete occurrence kind, independent of the captured byte role.
    pub(crate) kind: SourceKindV2,
    /// Exact original raw bytes digest, including whitespace/final LF.
    pub(crate) raw_sha256: String,
    /// Full occurrence extent; an opaque source may be empty.
    pub(crate) byte_length: u64,
    /// Intrinsic record marker only; generated and opaque pins require explicit null.
    pub(crate) schema_identity: Option<String>,
    /// Exact declared intrinsic, opaque-byte or identity-only validation profile.
    pub(crate) validation_profile: String,
    /// One actual recognized generated root label, otherwise explicit null.
    pub(crate) native_model: Option<NativeModelV2>,
    /// Original parseable native UUID text without canonicalization, otherwise null.
    pub(crate) native_root_uuid: Option<String>,
}

/// An inert item snapshot whose native facts require a separate adapter.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewItemV2 {
    /// Stable queue-local item key.
    pub(crate) key: String,
    /// Deterministic review-namespace UUID.
    pub(crate) item_id: String,
    /// Separately closed Lifecycle domain.
    pub(crate) domain: DomainV2,
    /// Exact version for the selected domain adapter.
    pub(crate) adapter_version: String,
    /// Original stable subject ID, without normalization.
    pub(crate) subject_id: String,
    /// Explicit re-review, never domain promotion.
    pub(crate) requested_action: RequestedAction,
    /// Sorted keys of the complete declared item source set.
    pub(crate) source_keys: Vec<String>,
    /// Recorded complete subject fingerprint; the consuming adapter verifies its binding.
    pub(crate) subject_sha256: String,
    /// Bounded minimized context.
    pub(crate) context: ContextSnapshot,
    /// Typed context encoding fingerprint.
    pub(crate) context_sha256: String,
    /// Declared policy selected by this item.
    pub(crate) policy_key: String,
    /// Typed complete policy/assignment/author/deadline binding fingerprint.
    pub(crate) policy_sha256: String,
    /// Sorted explicitly asserted source author keys.
    pub(crate) author_keys: Vec<String>,
    /// Sorted assigned reviewer-role pairs; authors are excluded.
    pub(crate) assignments: Vec<Assignment>,
    /// Explicit canonical UTC seconds, or no deadline.
    pub(crate) due_at: Option<String>,
    /// All five ordered dispositions; dissent cannot be hidden.
    pub(crate) allowed_dispositions: Vec<Disposition>,
}

/// Closed queue declarations; successful decoding is not source approval.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct QueueDocumentV2 {
    /// Exact forge.review-queue/2 version.
    pub(crate) schema_version: String,
    /// Mandatory asserted-only identity disclaimer.
    pub(crate) identity_disclaimer: String,
    /// Fixed IDs-and-hashes public projection only.
    pub(crate) sensitivity: Sensitivity,
    /// Explicit canonical nonnil caller UUID.
    pub(crate) queue_id: String,
    /// Explicit canonical UTC seconds.
    pub(crate) created_at: String,
    /// Sorted complete declared source pins.
    pub(crate) source_pins: Vec<SourcePinV2>,
    /// Sorted declared role keys.
    pub(crate) roles: Vec<RoleDefinition>,
    /// Sorted asserted reviewer keys.
    pub(crate) reviewers: Vec<Reviewer>,
    /// Sorted closed finite policies.
    pub(crate) policies: Vec<ReviewPolicy>,
    /// Sorted complete selected snapshots.
    pub(crate) items: Vec<ReviewItemV2>,
}

/// Private immutable response declarations; rationale is never public by default.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResponseDocumentV2 {
    /// Exact forge.review-response/2 version.
    pub(crate) schema_version: String,
    /// Mandatory asserted-only identity disclaimer.
    pub(crate) identity_disclaimer: String,
    /// Explicit unique caller response UUID.
    pub(crate) response_id: String,
    /// Referenced queue UUID.
    pub(crate) queue_id: String,
    /// Exact original queue package hash.
    pub(crate) queue_raw_sha256: String,
    /// Selected item key.
    pub(crate) item_key: String,
    /// Selected item UUID.
    pub(crate) item_id: String,
    /// Exact selected domain.
    pub(crate) domain: DomainV2,
    /// Exact selected adapter version.
    pub(crate) adapter_version: String,
    /// Exact selected re-review action.
    pub(crate) requested_action: RequestedAction,
    /// Complete queue source-pin set; no detached currentness.
    pub(crate) source_pins: Vec<SourcePinV2>,
    /// Exact subject binding.
    pub(crate) subject_sha256: String,
    /// Exact minimized context binding.
    pub(crate) context_sha256: String,
    /// Exact complete policy binding.
    pub(crate) policy_sha256: String,
    /// Asserted reviewer key.
    pub(crate) reviewer_key: String,
    /// Asserted role for this response.
    pub(crate) reviewer_role: String,
    /// Preserved independent disposition.
    pub(crate) disposition: Disposition,
    /// Explicit canonical UTC seconds.
    pub(crate) responded_at: String,
    /// Private rationale, at most eight KiB UTF-8.
    pub(crate) rationale: String,
    /// Explicit documented abstention reason; otherwise null.
    pub(crate) abstention_reason: Option<String>,
    /// Explicit null; domain edit syntax is unsupported by this slice.
    pub(crate) proposed_edit: (),
    /// Explicit bound prior reference only for superseded disposition.
    pub(crate) supersedes: Option<SupersessionReference>,
}

/// Inert recorded bundle; decoding cannot establish fresh closure or promotion.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispositionsDocumentV2 {
    /// Exact forge.review-dispositions/2 version.
    pub(crate) schema_version: String,
    /// Mandatory asserted-only identity disclaimer.
    pub(crate) identity_disclaimer: String,
    /// Exact recorded queue UUID.
    pub(crate) queue_id: String,
    /// Exact original queue byte pin.
    pub(crate) queue_raw_sha256: String,
    /// Explicit canonical UTC seconds.
    pub(crate) as_of: String,
    /// A recorded claim only, never a fresh proof type.
    pub(crate) currentness: RecordedCurrentness,
    /// Recorded opaque closure digest iff currentness claims recorded-current.
    pub(crate) closure_generation: Option<String>,
    /// Complete original declared source set.
    pub(crate) source_pins: Vec<SourcePinV2>,
    /// All unique response identities, sorted by UUID.
    pub(crate) responses: Vec<RecordedResponse>,
    /// All selected statuses, sorted by key.
    pub(crate) items: Vec<ItemDisposition>,
    /// Complete recorded counters.
    pub(crate) counts: DispositionCounts,
}

/// Private complete locator declarations; routes are inert and carry no lease.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LifecycleInputsV1 {
    /// Exact forge.review-lifecycle-inputs/1 marker.
    pub(crate) schema_version: String,
    /// Explicit project-root-relative record route.
    pub(crate) record: LifecycleRecordRoute,
    /// Explicit native declared source path and confined capture route.
    pub(crate) source: LifecycleArtifactRoute,
    /// Complete exact native-path-ordered generated declarations, at most 98.
    pub(crate) generated_artifacts: Vec<LifecycleArtifactRoute>,
}
/// Record capture route; its containing native directory is derived by the reader.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LifecycleRecordRoute {
    /// Nonempty bounded declared route, never a native currentness assertion.
    pub(crate) path: String,
}
/// Exact private native route plus explicit receiver capture route.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LifecycleArtifactRoute {
    /// Original exact native declared path; no normalization is allowed in hashes.
    pub(crate) declared_path: String,
    /// Explicit project-root-relative capture route under maintained geometry.
    pub(crate) path: String,
}

//! Separately closed inert Authoring-plan exchange declarations.
//! No declaration, digest or loaded currentness label supplies a native owner.
//! Required explicit nulls remain an obligation of the future strict schema consumer.

use serde::{Deserialize, Serialize};

pub(crate) use super::wire::{
    AbstentionRule, Assignment, AuthorSeparation, ContextSnapshot, Disposition, DispositionCounts,
    IDENTITY_DISCLAIMER, ItemDisposition, ItemState, MetSeat, RecordedCurrentness,
    RecordedResponse, RequestedAction, ResponseClassification, ReviewPolicy, Reviewer,
    RoleDefinition, SeatRequirement, Sensitivity, StateCounts, Substitution, SupersessionReference,
    UnmetSeat,
};

/// Fixed separately closed domain token; this is not native approval.
pub(crate) const DOMAIN: &str = "authoring-plan";
/// Fixed identity-only Authoring-plan adapter revision.
pub(crate) const ADAPTER: &str = "forge.authoring-plan-review/1";

/// Closed source-purpose labels; none proves an actual native original or role.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceKindV3 {
    /// Complete author-project original.
    AuthorProject,
    /// Complete authoring-pack original.
    AuthoringPack,
    /// Complete saved unfiltered gap report.
    GapReport,
    /// Complete applicability manifest original.
    ApplicabilityManifest,
    /// Complete native Catalog or Profile framework.
    Framework,
    /// Explicit resolved Catalog companion for a Profile.
    ResolvedCatalog,
    /// Complete declared Mapping Collection occurrence.
    MappingCollection,
    /// Complete explicitly supplied human clause occurrence.
    HumanClause,
    /// Separate saved full-plan original; excluded from native provenance.
    StoredPlan,
}

/// Closed native labels retained as plain data with original root UUID spelling.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NativeModelV3 {
    /// Native Catalog framework or explicit resolved companion.
    Catalog,
    /// Native Profile framework; no import traversal capability.
    Profile,
    /// Complete native Mapping Collection.
    MappingCollection,
}
impl NativeModelV3 {
    /// Exact lowercase root label; no normalization or native validation is implied.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Profile => "profile",
            Self::MappingCollection => "mapping-collection",
        }
    }
}

/// A separately closed authoring-plan domain, with no envelope conversion.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DomainV3 {
    /// One complete native plan's public identity; never promotion or approval.
    AuthoringPlan,
}

/// Eight-field declared purpose pin; full equality includes every required field.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourcePinV3 {
    /// Fixed role/ordinal key, never a source path or private native policy key.
    pub(crate) artifact_key: String,
    /// Complete occurrence kind, independent of the captured byte role.
    pub(crate) kind: SourceKindV3,
    /// Exact original raw bytes digest, including whitespace/final LF.
    pub(crate) raw_sha256: String,
    /// Full nonempty occurrence extent; zero is invalid for every /3 purpose.
    pub(crate) byte_length: u64,
    /// Exact intrinsic marker for this purpose, or required explicit null.
    pub(crate) schema_identity: Option<String>,
    /// Exact intrinsic/native/clause/full-plan profile label; the label grants no authority.
    pub(crate) validation_profile: String,
    /// Exact declared framework/companion/Mapping label, otherwise explicit null.
    pub(crate) native_model: Option<NativeModelV3>,
    /// Original native UUID text without canonicalization, otherwise required null.
    pub(crate) native_root_uuid: Option<String>,
}

/// An inert item snapshot whose native facts require a separate adapter.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewItemV3 {
    /// Stable queue-local item key.
    pub(crate) key: String,
    /// Deterministic review-namespace UUID.
    pub(crate) item_id: String,
    /// Separately closed Authoring-plan domain.
    pub(crate) domain: DomainV3,
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
pub(crate) struct QueueDocumentV3 {
    /// Exact forge.review-queue/3 version.
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
    pub(crate) source_pins: Vec<SourcePinV3>,
    /// Sorted declared role keys.
    pub(crate) roles: Vec<RoleDefinition>,
    /// Sorted asserted reviewer keys.
    pub(crate) reviewers: Vec<Reviewer>,
    /// Sorted closed finite policies.
    pub(crate) policies: Vec<ReviewPolicy>,
    /// Sorted complete selected snapshots.
    pub(crate) items: Vec<ReviewItemV3>,
}

/// Private immutable response declarations; rationale is never public by default.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResponseDocumentV3 {
    /// Exact forge.review-response/3 version.
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
    pub(crate) domain: DomainV3,
    /// Exact selected adapter version.
    pub(crate) adapter_version: String,
    /// Exact selected re-review action.
    pub(crate) requested_action: RequestedAction,
    /// Complete queue source-pin set; no detached currentness.
    pub(crate) source_pins: Vec<SourcePinV3>,
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
pub(crate) struct DispositionsDocumentV3 {
    /// Exact forge.review-dispositions/3 version.
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
    pub(crate) source_pins: Vec<SourcePinV3>,
    /// All unique response identities, sorted by UUID.
    pub(crate) responses: Vec<RecordedResponse>,
    /// All selected statuses, sorted by key.
    pub(crate) items: Vec<ItemDisposition>,
    /// Complete recorded counters.
    pub(crate) counts: DispositionCounts,
}

/// Private locator declarations only; paths carry no capture, lease or membership proof.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoringPlanInputsV1 {
    /// Exact forge.review-authoring-plan-inputs/1 marker.
    pub(crate) schema_version: String,
    /// Explicit selected project route; native root derives from its actual parent.
    pub(crate) project: AuthoringRouteV1,
    /// Separate complete stored-plan route.
    pub(crate) plan: AuthoringRouteV1,
}
/// Exact private route without filesystem or native authority.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoringRouteV1 {
    /// Complete review-root-relative route; future strict consumer validates geometry.
    pub(crate) path: String,
}
/// Private asserted init declarations; native project/plan/pins/identities are absent.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoringPlanInitV1 {
    /// Exact forge.review-authoring-plan-init/1 marker.
    pub(crate) schema_version: String,
    /// Complete explicitly asserted sorted role registry.
    pub(crate) roles: Vec<RoleDefinition>,
    /// Complete explicitly asserted sorted reviewer registry.
    pub(crate) reviewers: Vec<Reviewer>,
    /// Exactly one asserted policy after future strict validation.
    pub(crate) policies: Vec<ReviewPolicy>,
    /// Exactly one asserted item after future strict validation.
    pub(crate) items: Vec<AuthoringInitItem>,
}
/// Closed asserted init item; none of these values supplies native subject facts.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthoringInitItem {
    /// Exact public item key.
    pub(crate) key: String,
    /// Explicit asserted policy selection.
    pub(crate) policy_key: String,
    /// Explicit sorted nonempty source author keys.
    pub(crate) author_keys: Vec<String>,
    /// Explicit complete sorted reviewer-role declarations.
    pub(crate) assignments: Vec<Assignment>,
    /// Required nullable canonical review deadline.
    pub(crate) due_at: Option<String>,
}

//! Closed inert F12 review declarations. No type in this module is a captured
//! source proof, authenticated identity, evaluated quorum or native approval.
//! Explicit nulls and every field are required by the consumed JSON schemas.

use serde::{Deserialize, Serialize};

/// Mandatory limitation present in every exchange artifact.
pub(crate) const IDENTITY_DISCLAIMER: &str =
    "keys-roles-authors-and-times-are-asserted-not-authenticated-signed-or-non-repudiable";

/// Closed first-adapter source families.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceModel {
    /// Exact `mapping` wire value.
    Mapping,
    /// Exact `catalog` wire value.
    Catalog,
    /// Exact `profile` wire value.
    Profile,
    /// Exact `resolved-catalog` wire value.
    ResolvedCatalog,
    /// Exact `component-definition` wire value.
    ComponentDefinition,
    /// Exact `mapping-manifest` wire value.
    MappingManifest,
    /// Exact `applicability-manifest` wire value.
    ApplicabilityManifest,
    /// Exact `applicability-report` wire value.
    ApplicabilityReport,
    /// Exact `lifecycle-record` wire value.
    LifecycleRecord,
}

/// First-profile supported native adapter domains.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Domain {
    /// Exact `mapping-assertion` wire value.
    MappingAssertion,
    /// Exact `applicability-decision` wire value.
    ApplicabilityDecision,
}

/// Explicit review intent without promotion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RequestedAction {
    /// Exact `re-review` wire value.
    ReReview,
}

/// Every independent decision remains available.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Disposition {
    /// Exact `approve` wire value.
    Approve,
    /// Exact `reject` wire value.
    Reject,
    /// Exact `request-changes` wire value.
    RequestChanges,
    /// Exact `abstain` wire value.
    Abstain,
    /// Exact `superseded` wire value.
    Superseded,
}

/// No source prose or personal names in this first profile.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Sensitivity {
    /// Exact `ids-and-hashes` wire value.
    IdsAndHashes,
}

/// Abstention never fills or removes an approval seat.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AbstentionRule {
    /// Exact `nonapproving` wire value.
    Nonapproving,
}

/// Declared author-key exclusion, without authentication.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AuthorSeparation {
    /// Exact `declared-keys` wire value.
    DeclaredKeys,
}

/// Recorded response category, not currentness proof.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResponseClassification {
    /// Exact `current` wire value.
    Current,
    /// Exact `stale` wire value.
    Stale,
    /// Exact `foreign` wire value.
    Foreign,
    /// Exact `future` wire value.
    Future,
    /// Exact `late` wire value.
    Late,
    /// Exact `unassigned` wire value.
    Unassigned,
    /// Exact `superseded` wire value.
    Superseded,
    /// Exact `conflicted` wire value.
    Conflicted,
}

/// PRD presentation states; quorum-met is review policy only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ItemState {
    /// Exact `unassigned` wire value.
    Unassigned,
    /// Exact `assigned` wire value.
    Assigned,
    /// Exact `in-review` wire value.
    InReview,
    /// Exact `conflicted` wire value.
    Conflicted,
    /// Exact `changes-requested` wire value.
    ChangesRequested,
    /// Exact `quorum-met` wire value.
    QuorumMet,
    /// Exact `expired` wire value.
    Expired,
    /// Exact `stale` wire value.
    Stale,
}

/// Recorded claims remain inert until an actual consuming closure verifies them.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RecordedCurrentness {
    /// Exact `unverified` wire value.
    Unverified,
    /// Exact `recorded-current` wire value.
    RecordedCurrent,
}

/// A declared original source identity, not captured currentness or approval.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourcePin {
    /// Stable private-adapter source key.
    pub(crate) artifact_key: String,
    /// Closed source family.
    pub(crate) model: SourceModel,
    /// Canonical native root UUID, or explicit null for a companion.
    pub(crate) native_root_uuid: Option<String>,
    /// Hash of the claimed exact original bytes.
    pub(crate) raw_sha256: String,
    /// Claimed original byte count.
    pub(crate) byte_length: u64,
    /// Recorded intrinsic schema identity; only the consuming capture verifies it.
    pub(crate) schema_identity: String,
}

/// One declared role; the key grants no authority.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleDefinition {
    /// Exact asserted role key.
    pub(crate) key: String,
}

/// One asserted reviewer key with an explicit finite role set.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Reviewer {
    /// Asserted key, not an authenticated person.
    pub(crate) key: String,
    /// Sorted unique declared role keys.
    pub(crate) role_keys: Vec<String>,
}

/// A positive finite number of approval seats for one role.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SeatRequirement {
    /// Declared role whose seats are requested.
    pub(crate) role_key: String,
    /// Positive bounded required seat count.
    pub(crate) count: u32,
}

/// An explicit candidate edge that never manufactures an approval.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Substitution {
    /// Required seat role.
    pub(crate) seat_role: String,
    /// Explicit substitute asserted key.
    pub(crate) reviewer_key: String,
    /// Role this key must actually declare.
    pub(crate) asserted_role: String,
    /// Minimized declared substitution reason.
    pub(crate) reason_code: String,
}

/// Closed asserted policy data; no seat matching is performed here.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewPolicy {
    /// Stable policy key.
    pub(crate) key: String,
    /// Role-ordered positive approval requirements.
    pub(crate) seats: Vec<SeatRequirement>,
    /// Sorted explicit substitute edges.
    pub(crate) substitutions: Vec<Substitution>,
    /// Fixed nonapproving rule; seats are never reduced.
    pub(crate) abstention_rule: AbstentionRule,
    /// Sorted documented reasons permitting an empty abstention rationale.
    pub(crate) empty_abstention_reasons: Vec<String>,
    /// Fixed separation of declared author keys.
    pub(crate) author_separation: AuthorSeparation,
}

/// One explicitly assigned asserted key and role.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Assignment {
    /// Declared reviewer key.
    pub(crate) reviewer_key: String,
    /// Role actually declared for that key.
    pub(crate) role_key: String,
}

/// Minimized context; identifiers may still be sensitive.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextSnapshot {
    /// Sorted fixed-format reason tokens, without rationale prose.
    pub(crate) reason_codes: Vec<String>,
    /// Sorted original subject identifiers; no source excerpt.
    pub(crate) related_subject_ids: Vec<String>,
}

/// An inert item snapshot whose native facts require a separate adapter.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewItem {
    /// Stable queue-local item key.
    pub(crate) key: String,
    /// Deterministic review-namespace UUID.
    pub(crate) item_id: String,
    /// Selected first-profile domain.
    pub(crate) domain: Domain,
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
pub(crate) struct QueueDocument {
    /// Exact forge.review-queue/1 version.
    pub(crate) schema_version: String,
    /// Mandatory asserted-only identity disclaimer.
    pub(crate) identity_disclaimer: String,
    /// First-slice IDs-and-hashes projection only.
    pub(crate) sensitivity: Sensitivity,
    /// Explicit canonical nonnil caller UUID.
    pub(crate) queue_id: String,
    /// Explicit canonical UTC seconds.
    pub(crate) created_at: String,
    /// Sorted complete declared source pins.
    pub(crate) source_pins: Vec<SourcePin>,
    /// Sorted declared role keys.
    pub(crate) roles: Vec<RoleDefinition>,
    /// Sorted asserted reviewer keys.
    pub(crate) reviewers: Vec<Reviewer>,
    /// Sorted closed finite policies.
    pub(crate) policies: Vec<ReviewPolicy>,
    /// Sorted complete selected snapshots.
    pub(crate) items: Vec<ReviewItem>,
}

/// An exact prior response reference; chain validity is future merge work.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SupersessionReference {
    /// Prior response UUID, never this response itself.
    pub(crate) response_id: String,
    /// Exact prior original-byte hash, not semantic equality.
    pub(crate) raw_sha256: String,
}

/// Private immutable response declarations; rationale is never public by default.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResponseDocument {
    /// Exact forge.review-response/1 version.
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
    pub(crate) domain: Domain,
    /// Exact selected adapter version.
    pub(crate) adapter_version: String,
    /// Exact selected re-review action.
    pub(crate) requested_action: RequestedAction,
    /// Complete queue source-pin set; no detached currentness.
    pub(crate) source_pins: Vec<SourcePin>,
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

/// Minimized retained response evidence; no rationale or paths are exported.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecordedResponse {
    /// Original response UUID.
    pub(crate) response_id: String,
    /// Exact original raw hash.
    pub(crate) raw_sha256: String,
    /// Exact claimed original byte count.
    pub(crate) byte_length: u64,
    /// Original asserted target key, including foreign targets.
    pub(crate) item_key: String,
    /// Original asserted reviewer key.
    pub(crate) reviewer_key: String,
    /// Original asserted reviewer role.
    pub(crate) reviewer_role: String,
    /// Original disposition, including historical dissent.
    pub(crate) disposition: Disposition,
    /// Original asserted time.
    pub(crate) responded_at: String,
    /// Recorded classification, not independently established by decoding.
    pub(crate) classification: ResponseClassification,
}

/// An inert recorded matching witness; the matching algorithm is separate.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct MetSeat {
    /// Declared required seat role.
    pub(crate) role_key: String,
    /// Zero-based seat ordinal.
    pub(crate) ordinal: u32,
    /// One asserted key filling at most one item seat.
    pub(crate) reviewer_key: String,
    /// Recorded current approval supporting this seat.
    pub(crate) response_id: String,
}

/// One preserved unsatisfied seat.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnmetSeat {
    /// Declared required role.
    pub(crate) role_key: String,
    /// Zero-based seat ordinal.
    pub(crate) ordinal: u32,
}

/// Complete recorded item status; no native transition authority.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ItemDisposition {
    /// Exact selected item key.
    pub(crate) item_key: String,
    /// Exact selected item UUID.
    pub(crate) item_id: String,
    /// Recorded presentation state.
    pub(crate) state: ItemState,
    /// Sorted complete minimized reason tokens.
    pub(crate) reason_codes: Vec<String>,
    /// Positive declared denominator.
    pub(crate) required_seats: u32,
    /// Recorded witnesses; no duplicated reviewer keys.
    pub(crate) met_seats: Vec<MetSeat>,
    /// Complete remaining seats.
    pub(crate) unmet_seats: Vec<UnmetSeat>,
    /// Sorted complete response references for this item.
    pub(crate) response_ids: Vec<String>,
    /// Sorted references preserving every reject/request-changes record.
    pub(crate) dissent_ids: Vec<String>,
    /// Recorded blockers; a blocked item cannot claim quorum-met.
    pub(crate) blocking: bool,
}

/// Complete recorded status denominator.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StateCounts {
    /// Number of recorded unassigned rows.
    pub(crate) unassigned: u32,
    /// Number of recorded assigned rows.
    pub(crate) assigned: u32,
    /// Number of recorded in-review rows.
    pub(crate) in_review: u32,
    /// Number of recorded conflicted rows.
    pub(crate) conflicted: u32,
    /// Number of recorded changes-requested rows.
    pub(crate) changes_requested: u32,
    /// Number of recorded quorum-met rows.
    pub(crate) quorum_met: u32,
    /// Number of recorded expired rows.
    pub(crate) expired: u32,
    /// Number of recorded stale rows.
    pub(crate) stale: u32,
}

/// Complete file/row counters, checked without evaluating quorum.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispositionCounts {
    /// Complete selected item count.
    pub(crate) items: u32,
    /// Complete original response file count, including exact duplicates.
    pub(crate) response_files: u32,
    /// Complete distinct original response identity count.
    pub(crate) unique_responses: u32,
    /// Additional exact duplicate file count.
    pub(crate) exact_duplicates: u32,
    /// Complete presentation-state counters.
    pub(crate) states: StateCounts,
}

/// Inert recorded bundle; decoding cannot establish fresh closure or promotion.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DispositionsDocument {
    /// Exact forge.review-dispositions/1 version.
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
    pub(crate) source_pins: Vec<SourcePin>,
    /// All unique response identities, sorted by UUID.
    pub(crate) responses: Vec<RecordedResponse>,
    /// All selected statuses, sorted by key.
    pub(crate) items: Vec<ItemDisposition>,
    /// Complete recorded counters.
    pub(crate) counts: DispositionCounts,
}

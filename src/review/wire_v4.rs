//! Separately closed edit-capable Authoring exchange declarations (family /4).
//! Mirrors the /3 typed stack; the queue/item/pin shapes are identical and reused.
//! The only envelope change is `proposed_edit: Option<ProposedEdit>` on responses.
//! Proposed edits are inert unapproved patches: merge validates syntax and target
//! hash per response and NEVER applies them (PRD 068 M-10, G-4). No declaration,
//! digest or loaded currentness label supplies a native owner.

use serde::{Deserialize, Serialize};

pub(crate) use super::wire::{
    Disposition, DispositionCounts, IDENTITY_DISCLAIMER, ItemDisposition, ItemState, MetSeat,
    RecordedCurrentness, RecordedResponse, RequestedAction, ResponseClassification, ReviewPolicy,
    StateCounts, SupersessionReference, UnmetSeat,
};
pub(crate) use super::wire_v3::{DomainV3, ReviewItemV3, SourceKindV3, SourcePinV3};

/// Queue and recorded-bundle shapes are byte-identical to /3; only the version
/// tokens differ, so the closed /3 declarations are reused without a second shape.
pub(crate) use super::wire_v3::{
    DispositionsDocumentV3 as DispositionsDocumentV4, QueueDocumentV3 as QueueDocumentV4,
};
/// Fixed identity-only Authoring-plan adapter revision, shared with family /3.
pub(crate) const ADAPTER: &str = "forge.authoring-plan-review/1";
/// Exact closed edit-capable response version.
pub(crate) const RESPONSE_VERSION: &str = "forge.review-response/4";

/// Complete proposed replacement UTF-8 text for one bound subject original.
/// The subject is one whole Markdown document (plan or clause); the single closed
/// format proposes its full replacement text. Syntax validation is total: bounded
/// nonempty UTF-8 plus exact digest integrity. The patch is never applied by review.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PatchFormat {
    /// Complete replacement text for the target subject original.
    FullText,
}

/// Closed inert proposed-edit envelope; unapproved patch data, never an application.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProposedEdit {
    /// Exact subject original the patch targets; bound against item and response.
    pub(crate) target_subject_sha256: String,
    /// Single closed patch format.
    pub(crate) patch_format: PatchFormat,
    /// Bounded complete replacement UTF-8 text, at most 64 KiB.
    pub(crate) patch: String,
    /// Exact SHA-256 of the patch UTF-8 bytes.
    pub(crate) patch_sha256: String,
}

/// Private immutable edit-capable response declarations; rationale is never public.
/// Every field except `proposed_edit` matches the /3 response envelope exactly.
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResponseDocumentV4 {
    /// Exact forge.review-response/4 version.
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
    /// Optional inert proposed edit; null preserves the zero-edit behavior.
    pub(crate) proposed_edit: Option<ProposedEdit>,
    /// Explicit bound prior reference only for superseded disposition.
    pub(crate) supersedes: Option<SupersessionReference>,
}

#[cfg(test)]
#[path = "wire_v4_tests.rs"]
mod tests;

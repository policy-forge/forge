//! Genuine Lifecycle init queue projection from one complete Approved/current owner.
//! The private request contains only asserted review policy. It cannot supply native
//! pins, subject identity, hashes, currentness or a new captured Queue original.

use serde::Deserialize;

use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{self, DecodedV2};
use super::encode_v2;
use super::hash_v2;
use super::lifecycle_capture::ApprovedLifecycleClosure;
use super::wire::{
    Assignment, ContextSnapshot, Disposition, ReviewPolicy, Reviewer, RoleDefinition,
};
use super::wire_v2::{
    ADAPTER, DomainV2, IDENTITY_DISCLAIMER, QueueDocumentV2, RequestedAction, ReviewItemV2,
    Sensitivity,
};
use crate::workspace::preparation::WorkControl;

/// Exact private request family, distinct from native locator and public Queue markers.
const INIT_MARKER: &str = "forge.review-lifecycle-init/1";
/// Fixed minimized native eligibility context; never caller prose or inferred scheduling.
const REASON: &str = "lifecycle-approved-current";

/// Closed ordinary request declarations; strict schema enforces every required nullable field.
/// This type has no native-owner constructor, proof flag or public serialized output.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LifecycleInitRequest {
    /// Exact private marker, checked again by the genuine consumer.
    schema_version: String,
    /// Complete asserted role registry, moved rather than cloned after admitted decoding.
    roles: Vec<RoleDefinition>,
    /// Complete asserted reviewer registry and original role memberships.
    reviewers: Vec<Reviewer>,
    /// The sole explicit asserted positive-seat policy.
    policies: Vec<ReviewPolicy>,
    /// The sole selected ordinary declaration, containing no native operands.
    items: Vec<InitItem>,
}

/// Closed asserted item inputs only; native fields derive from the genuine closure.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitItem {
    /// Explicit queue-local key, distinct from private native policy/version keys.
    key: String,
    /// Reference to the sole explicitly asserted review policy.
    policy_key: String,
    /// Complete declared author keys, with no inferred native parties.
    author_keys: Vec<String>,
    /// Complete asserted assignments, including the legitimate empty set.
    assignments: Vec<Assignment>,
    /// Required explicit nullable canonical UTC due second.
    due_at: Option<String>,
}

/// Genuine native-owned queue projection; no captured input Queue is fabricated.
/// Private fields, absent Clone/Serialize and the closure borrow retain the real source owner.
pub(crate) struct PreparedLifecycleQueueExport<'native> {
    /// The sole real complete native/physical owner through every output fence.
    closure: &'native ApprovedLifecycleClosure,
    /// Actual internally matched Auxiliary allocation, retained through output binding.
    policy_raw: &'native [u8],
    /// Fully admitted minimized plain Queue projection from native and asserted inputs.
    document: QueueDocumentV2,
}
impl<'native> PreparedLifecycleQueueExport<'native> {
    /// Borrow only this producer's complete admitted projection for finite encoding.
    pub(crate) fn document(&self) -> &QueueDocumentV2 {
        &self.document
    }
    /// Retain the genuine whole original owner through Root's final publication operations.
    pub(crate) fn closure(&self) -> &'native ApprovedLifecycleClosure {
        self.closure
    }
    /// Bind actual newly encoded bytes after strict decoding, then verify every original.
    /// This is a source/output fence, not a recorded-current disposition or publication token.
    pub(crate) fn bind_output(
        &self,
        output: &DecodedV2<'_, QueueDocumentV2>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        decode_v2::phase(ledger, control, |ledger, control| {
            self.closure.held_inputs().lifecycle_init_original(self.policy_raw, ledger, control)?;
            encode_v2::exchange::compare_queues(
                &self.document,
                output.document(),
                ledger,
                control,
            )?;
            native::bind_output(self.closure, output, ledger, control)?;
            self.closure.verify_inputs(ledger, control)
        })
    }
}

/// Bind the actual held private request and derive every native field on the original caller.
/// Explicit `queue_id/created_at` remain canonical asserted inputs; no ambient clock is used.
pub(crate) fn prepare<'native>(
    closure: &'native ApprovedLifecycleClosure,
    policy_raw: &[u8],
    queue_id: &str,
    created_at: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedLifecycleQueueExport<'native>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let policy_raw =
            closure.held_inputs().lifecycle_init_original(policy_raw, ledger, control)?;
        let request = decode_v2::decode_lifecycle_init(policy_raw, ledger, control)?;
        headers(queue_id, created_at, ledger, control)?;
        let document = project(closure, request, queue_id, created_at, ledger, control)?;
        decode_v2::queue(&document, ledger, control)?;
        closure.verify_inputs(ledger, control)?;
        ledger.derived(std::mem::size_of::<PreparedLifecycleQueueExport<'_>>())?;
        ledger.bytes(std::mem::size_of::<PreparedLifecycleQueueExport<'_>>())?;
        Ok(PreparedLifecycleQueueExport { closure, policy_raw, document })
    })
}

/// Preserve actual canonical UUID/calendar validation with complete same-caller precharges.
fn headers(
    queue_id: &str,
    created_at: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    decode_v2::phase(ledger, control, |ledger, _control| {
        ledger.visits(2)?;
        ledger.bytes(queue_id.len())?;
        ledger.derived(100)?;
        super::validate::uuid(queue_id)?;
        ledger.bytes(created_at.len())?;
        ledger.derived(84)?;
        super::validate::time(created_at)
    })
}

/// Copy one complete admitted original/fixed text before owned String growth.
fn copy_text(value: &str, ledger: &mut ContractLedger) -> Result<String, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(value.len())?;
    let owned_extent =
        value.len().checked_add(std::mem::size_of::<String>()).ok_or_else(|| ledger.capacity())?;
    ledger.derived(owned_extent)?;
    let mut copied = String::new();
    copied.try_reserve_exact(value.len()).map_err(|_| ledger.capacity())?;
    copied.push_str(value);
    Ok(copied)
}

/// Produce the fixed complete public eligibility context without native prose.
fn context(ledger: &mut ContractLedger) -> Result<ContextSnapshot, ContractError> {
    ledger.derived(std::mem::size_of::<ContextSnapshot>() + std::mem::size_of::<String>())?;
    ledger.bytes(std::mem::size_of::<String>())?;
    let mut reason_codes = Vec::new();
    reason_codes.try_reserve_exact(1).map_err(|_| ledger.capacity())?;
    reason_codes.push(copy_text(REASON, ledger)?);
    Ok(ContextSnapshot { reason_codes, related_subject_ids: Vec::new() })
}

/// Preserve all five dissent/approval choices with storage and actual copy work admitted.
fn dispositions(ledger: &mut ContractLedger) -> Result<Vec<Disposition>, ContractError> {
    let values = [
        Disposition::Approve,
        Disposition::Reject,
        Disposition::RequestChanges,
        Disposition::Abstain,
        Disposition::Superseded,
    ];
    let extent = values
        .len()
        .checked_mul(std::mem::size_of::<Disposition>())
        .ok_or_else(|| ledger.capacity())?;
    let owned_extent = extent
        .checked_add(std::mem::size_of::<Vec<Disposition>>())
        .ok_or_else(|| ledger.capacity())?;
    ledger.derived(owned_extent)?;
    ledger.visits(values.len())?;
    ledger.bytes(extent)?;
    let mut result = Vec::new();
    result.try_reserve_exact(values.len()).map_err(|_| ledger.capacity())?;
    result.extend_from_slice(&values);
    Ok(result)
}

/// Copy every genuine public pin key in exact lexical pin order before collection growth.
fn source_keys(
    closure: &ApprovedLifecycleClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<String>, ContractError> {
    let pins = closure.source_pins();
    let extent =
        pins.len().checked_mul(std::mem::size_of::<String>()).ok_or_else(|| ledger.capacity())?;
    let owned_extent =
        extent.checked_add(std::mem::size_of::<Vec<String>>()).ok_or_else(|| ledger.capacity())?;
    ledger.derived(owned_extent)?;
    ledger.bytes(extent)?;
    let mut result = Vec::new();
    result.try_reserve_exact(pins.len()).map_err(|_| ledger.capacity())?;
    for pin in pins {
        decode_v2::checkpoint(ledger, control)?;
        result.push(copy_text(&pin.artifact_key, ledger)?);
    }
    Ok(result)
}

/// Move already admitted assertions and build the sole genuine native-derived item.
fn project(
    closure: &ApprovedLifecycleClosure,
    request: LifecycleInitRequest,
    queue_id: &str,
    created_at: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<QueueDocumentV2, ContractError> {
    let LifecycleInitRequest { schema_version, roles, reviewers, policies, mut items } = request;
    ledger.visits(1)?;
    let marker_extent =
        schema_version.len().checked_add(INIT_MARKER.len()).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(marker_extent)?;
    if schema_version != INIT_MARKER || policies.len() != 1 || items.len() != 1 {
        return Err(ContractError::Invalid);
    }
    let declared = items.pop().ok_or(ContractError::Invalid)?;
    let (subject_id, subject_sha256) = native::subject(closure, ledger, control)?;
    let source_pins = native::pins(closure, ledger, control)?;
    let source_keys = source_keys(closure, ledger, control)?;
    let projection_extent = std::mem::size_of::<QueueDocumentV2>()
        .checked_add(std::mem::size_of::<ReviewItemV2>())
        .ok_or_else(|| ledger.capacity())?;
    ledger.derived(projection_extent)?;
    ledger.bytes(projection_extent)?;
    let mut projected = Vec::new();
    projected.try_reserve_exact(1).map_err(|_| ledger.capacity())?;
    projected.push(ReviewItemV2 {
        key: declared.key,
        item_id: String::new(),
        domain: DomainV2::LifecyclePolicyVersion,
        adapter_version: copy_text(ADAPTER, ledger)?,
        subject_id,
        requested_action: RequestedAction::ReReview,
        source_keys,
        subject_sha256,
        context: context(ledger)?,
        context_sha256: String::new(),
        policy_key: declared.policy_key,
        policy_sha256: String::new(),
        author_keys: declared.author_keys,
        assignments: declared.assignments,
        due_at: declared.due_at,
        allowed_dispositions: dispositions(ledger)?,
    });
    let mut document = QueueDocumentV2 {
        schema_version: copy_text("forge.review-queue/2", ledger)?,
        identity_disclaimer: copy_text(IDENTITY_DISCLAIMER, ledger)?,
        sensitivity: Sensitivity::IdsAndHashes,
        queue_id: copy_text(queue_id, ledger)?,
        created_at: copy_text(created_at, ledger)?,
        source_pins,
        roles,
        reviewers,
        policies,
        items: projected,
    };
    let item = &document.items[0];
    let context = hash_v2::context(&document, item, ledger, control)?;
    let policy = hash_v2::policy(&document, &document.policies[0], item, ledger, control)?;
    document.items[0].context_sha256 = context;
    document.items[0].policy_sha256 = policy;
    let item_id = hash_v2::item_id(&document, &document.items[0], ledger, control)?;
    document.items[0].item_id = item_id;
    Ok(document)
}

/// Body-exact native correlations consume genuine closure members and unchanged hash profiles.
#[path = "lifecycle_queue_export_native.rs"]
mod native;

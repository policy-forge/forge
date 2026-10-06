//! Pending /2 Lifecycle queue binding over one genuine complete held owner.
//! Strict declarations, digests and this borrowed pending result confer no new
//! currentness, vote authority, native transition, DTO or publication capability.

use crate::workspace::preparation::WorkControl;

use super::capture::lifecycle::RegisteredLifecycleReviewOriginals;
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{self, DecodedV2};
use super::hash_v2::{self, RosterEntry};
use super::lifecycle_capture::ApprovedLifecycleClosure;
use super::wire_v2::{QueueDocumentV2, ResponseDocumentV2, SourcePinV2};

/// Inert decoded review originals borrowing the genuine Lifecycle owner.
/// No caller raw/index/list/Rc factory, Clone, Serialize or successful-current
/// conversion exists. Root owns the separate native finalizer and publisher.
pub(crate) struct PendingLifecycleBinding<'native> {
    /// Retain the actual receiver-issued complete native/physical owner borrow.
    closure: &'native ApprovedLifecycleClosure,
    /// Internally obtained actual whole Queue/Response registration observation.
    originals: RegisteredLifecycleReviewOriginals<'native>,
    /// Exact real held Queue raw bytes plus inert closed declarations.
    queue: DecodedV2<'native, QueueDocumentV2>,
    /// Every real held Response in registration order, including byte-identical repeats.
    responses: Vec<DecodedV2<'native, ResponseDocumentV2>>,
}
impl<'native> PendingLifecycleBinding<'native> {
    /// Borrow the genuine receiver owner; no detached native state can replace it.
    pub(crate) fn closure(&self) -> &'native ApprovedLifecycleClosure {
        self.closure
    }
    /// Borrow the exact original Queue allocation and its admitted /2 declarations.
    pub(crate) fn queue(&self) -> &DecodedV2<'native, QueueDocumentV2> {
        &self.queue
    }
    /// Borrow the complete real Response occurrence sequence, never a unique-only subset.
    pub(crate) fn responses(&self) -> &[DecodedV2<'native, ResponseDocumentV2>] {
        &self.responses
    }
    /// Borrow only internally observed actual registration indices for Root's finalizer.
    pub(crate) fn originals(&self) -> &RegisteredLifecycleReviewOriginals<'native> {
        &self.originals
    }
}

/// Retain actual raw decodes and complete native queue correlations on the original caller.
/// Foreign/stale response evidence remains available to the existing /2 policy classifier;
/// this leaf does not discard it by treating every response as an eligible current vote.
pub(crate) fn prepare<'native>(
    closure: &'native ApprovedLifecycleClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingLifecycleBinding<'native>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let originals = closure.held_inputs().lifecycle_review_originals(ledger, control)?;
        let queue = decode_v2::decode_queue(originals.queue_raw(), ledger, control)?;
        bind_queue(closure, &queue, ledger, control)?;
        let count = originals.responses().len();
        let extent = count
            .checked_mul(std::mem::size_of::<DecodedV2<'_, ResponseDocumentV2>>())
            .ok_or_else(|| ledger.capacity())?;
        // Complete retained wrapper slots and moves precede vector reservation.
        // Every decoder separately admits its full typed/raw/hash materialization.
        ledger.derived(extent)?;
        ledger.bytes(extent)?;
        let mut responses = Vec::new();
        responses.try_reserve_exact(count).map_err(|_| ledger.capacity())?;
        for raw in originals.responses() {
            decode_v2::checkpoint(ledger, control)?;
            ledger.visits(1)?;
            let response = decode_v2::decode_response(raw, ledger, control)?;
            responses.push(response);
        }
        // Retain this whole-owner verification result through its actual postfence.
        // Root must repeat it after its future DTO/encoding work and before publication.
        closure.verify_inputs(ledger, control)?;
        ledger.derived(std::mem::size_of::<PendingLifecycleBinding<'_>>())?;
        Ok(PendingLifecycleBinding { closure, originals, queue, responses })
    })
}

/// Compare complete actual private/public operands; no normalization or early clipping.
fn same_text(left: &str, right: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(1)?;
    let extent = left.len().checked_add(right.len()).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(extent)?;
    if left != right {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Admit all full pin string/nullable/fixed fields before complete eight-field equality.
fn pin_operands(pin: &SourcePinV2, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(1)?;
    let extent = [
        pin.artifact_key.len(),
        pin.raw_sha256.len(),
        pin.validation_profile.len(),
        pin.schema_identity.as_ref().map_or(0, String::len),
        pin.native_root_uuid.as_ref().map_or(0, String::len),
    ]
    .into_iter()
    .try_fold(64_usize, usize::checked_add)
    .ok_or_else(|| ledger.capacity())?;
    ledger.bytes(extent)
}

/// Bind every public source occurrence and every sole-item key to the actual receiver roster.
fn complete_pins(
    closure: &ApprovedLifecycleClosure,
    queue: &QueueDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let expected = closure.source_pins();
    let item = queue.items.first().ok_or(ContractError::Binding)?;
    ledger.visits(1)?;
    if queue.source_pins.len() != expected.len() || item.source_keys.len() != expected.len() {
        return Err(ContractError::Binding);
    }
    for ((actual, native), key) in queue.source_pins.iter().zip(expected).zip(&item.source_keys) {
        decode_v2::checkpoint(ledger, control)?;
        pin_operands(actual, ledger)?;
        pin_operands(native, ledger)?;
        // Derived equality includes kind, length, both nullable fields/profile and
        // original native model/UUID text in addition to key and exact raw digest.
        if actual != native {
            return Err(ContractError::Binding);
        }
        same_text(key, &native.artifact_key, ledger)?;
    }
    Ok(())
}

/// Borrow the full native-purpose roster after its complete descriptor is admitted.
fn native_roster<'a>(
    closure: &'a ApprovedLifecycleClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<RosterEntry<'a>>, ContractError> {
    decode_v2::checkpoint(ledger, control)?;
    let rows = closure.roster();
    let expected = closure
        .record()
        .policy
        .generated_artifacts
        .len()
        .checked_add(2)
        .ok_or_else(|| ledger.capacity())?;
    if rows.len() != expected || rows.len() != closure.source_pins().len() {
        return Err(ContractError::Binding);
    }
    let extent = rows
        .len()
        .checked_mul(std::mem::size_of::<RosterEntry<'_>>())
        .ok_or_else(|| ledger.capacity())?;
    ledger.derived(extent)?;
    ledger.bytes(extent)?;
    let mut roster = Vec::new();
    roster.try_reserve_exact(rows.len()).map_err(|_| ledger.capacity())?;
    for row in rows {
        decode_v2::checkpoint(ledger, control)?;
        ledger.visits(1)?;
        roster.push(row);
    }
    Ok(roster)
}

/// Close actual native subject/source and complete asserted context/policy/item correlations.
/// Ordinary /2 decoding alone validates public declarations but cannot establish the
/// private native policy/version keys or the complete actual purpose-ordered roster.
fn bind_queue(
    closure: &ApprovedLifecycleClosure,
    queue: &DecodedV2<'_, QueueDocumentV2>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let document = queue.document();
        ledger.visits(1)?;
        if document.items.len() != 1 || document.policies.len() != 1 {
            return Err(ContractError::Binding);
        }
        complete_pins(closure, document, ledger, control)?;
        let item = &document.items[0];
        let policy = &document.policies[0];
        let native = &closure.record().policy;
        let roster = native_roster(closure, ledger, control)?;
        let sources = hash_v2::sources(&roster, ledger, control)?;
        let subject_id =
            hash_v2::subject_id(&native.policy_key, &native.version_key, ledger, control)?;
        let subject =
            hash_v2::subject(&native.policy_key, &native.version_key, &sources, ledger, control)?;
        same_text(&item.subject_id, &subject_id, ledger)?;
        same_text(&item.subject_sha256, &subject, ledger)?;
        // Actual /2 hash functions revalidate all complete roles/reviewers/author/
        // assignment/policy/deadline/disposition operands. No native party is inferred
        // to be an asserted reviewer, author or review seat.
        let context = hash_v2::context(document, item, ledger, control)?;
        let policy_hash = hash_v2::policy(document, policy, item, ledger, control)?;
        let item_id = hash_v2::item_id(document, item, ledger, control)?;
        same_text(&item.context_sha256, &context, ledger)?;
        same_text(&item.policy_sha256, &policy_hash, ledger)?;
        same_text(&item.item_id, &item_id, ledger)
    })
}

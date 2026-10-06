//! Private body-exact native correlations and fully admitted complete pin copies.
//! No caller pin/subject/hash/raw status can issue the genuine closure consumed here.

use crate::review::decode::{ContractError, ContractLedger};
use crate::review::decode_v2::{self, DecodedV2};
use crate::review::hash_v2::{self, RosterEntry};
use crate::review::lifecycle_capture::ApprovedLifecycleClosure;
use crate::review::wire_v2::{QueueDocumentV2, SourcePinV2};
use crate::workspace::preparation::WorkControl;

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

/// Clone a full observed pin with every exact string/nullable field admitted before growth.
fn copy_pin(pin: &SourcePinV2, ledger: &mut ContractLedger) -> Result<SourcePinV2, ContractError> {
    let strings = pin
        .artifact_key
        .len()
        .checked_add(pin.raw_sha256.len())
        .and_then(|n| n.checked_add(pin.schema_identity.as_ref().map_or(0, String::len)))
        .and_then(|n| n.checked_add(pin.validation_profile.len()))
        .and_then(|n| n.checked_add(pin.native_root_uuid.as_ref().map_or(0, String::len)))
        .ok_or(ContractError::Capacity)?;
    ledger.bytes(strings)?;
    ledger.derived(strings)?;
    Ok(SourcePinV2 {
        artifact_key: pin.artifact_key.clone(),
        kind: pin.kind,
        raw_sha256: pin.raw_sha256.clone(),
        byte_length: pin.byte_length,
        schema_identity: pin.schema_identity.clone(),
        validation_profile: pin.validation_profile.clone(),
        native_model: pin.native_model,
        native_root_uuid: pin.native_root_uuid.clone(),
    })
}

/// Compute subject ID and complete private source fingerprint solely from real native facts.
pub(super) fn subject(
    closure: &ApprovedLifecycleClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(String, String), ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let roster = native_roster(closure, ledger, control)?;
        let sources = hash_v2::sources(&roster, ledger, control)?;
        let native = &closure.record().policy;
        let id = hash_v2::subject_id(&native.policy_key, &native.version_key, ledger, control)?;
        let fingerprint =
            hash_v2::subject(&native.policy_key, &native.version_key, &sources, ledger, control)?;
        Ok((id, fingerprint))
    })
}

/// Copy the complete genuine eight-field public pin roster after full before-growth admission.
pub(super) fn pins(
    closure: &ApprovedLifecycleClosure,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePinV2>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let pins = closure.source_pins();
        let extent = pins
            .len()
            .checked_mul(std::mem::size_of::<SourcePinV2>())
            .ok_or_else(|| ledger.capacity())?;
        let owned_extent = extent
            .checked_add(std::mem::size_of::<Vec<SourcePinV2>>())
            .ok_or_else(|| ledger.capacity())?;
        ledger.derived(owned_extent)?;
        ledger.bytes(extent)?;
        let mut copied = Vec::new();
        copied.try_reserve_exact(pins.len()).map_err(|_| ledger.capacity())?;
        for pin in pins {
            decode_v2::checkpoint(ledger, control)?;
            ledger.visits(1)?;
            copied.push(copy_pin(pin, ledger)?);
        }
        Ok(copied)
    })
}

/// Use the exact genuine binding body on strict newly encoded output without Queue capture.
pub(super) fn bind_output(
    closure: &ApprovedLifecycleClosure,
    queue: &DecodedV2<'_, QueueDocumentV2>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    bind_queue(closure, queue, ledger, control)
}

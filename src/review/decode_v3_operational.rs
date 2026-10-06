//! Plain Response/3 and recorded Dispositions/3 intake, with no native or policy owner.
use super::{ContractError, ContractLedger, DecodedV3, compared, hash, key, time, uuid};
use crate::review::decode_v2::{operational_raw, phase, recorded_data};
use crate::review::wire_v3::{
    ADAPTER, Disposition, DispositionsDocumentV3, IDENTITY_DISCLAIMER, ItemState,
    RecordedCurrentness, ResponseDocumentV3,
};
use crate::workspace::preparation::WorkControl;

/// Full response raw ceiling, including emitted LF on finite output.
pub(crate) const MAX_RESPONSE_BYTES: usize = 1_048_576;
/// Full recorded raw ceiling intersected with unchanged monotonic derived/work bounds.
pub(crate) const MAX_RECORDED_BYTES: usize = 33_554_432;

/// Validate all asserted immutable response data; actual Queue binding remains separate.
pub(crate) fn response(
    document: &ResponseDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        crate::review::encode_v3::response_operands(document, ledger, control)?;
        compared(
            &[&document.schema_version, &document.identity_disclaimer, &document.adapter_version],
            ledger,
        )?;
        if document.schema_version != "forge.review-response/3"
            || document.identity_disclaimer != IDENTITY_DISCLAIMER
            || document.adapter_version != ADAPTER
        {
            return Err(ContractError::Invalid);
        }
        for value in [&document.response_id, &document.queue_id, &document.item_id] {
            uuid(value, ledger)?;
        }
        time(&document.responded_at, ledger)?;
        crate::review::hash_v3::validate_public_pins(&document.source_pins, ledger, control)?;
        for value in [&document.item_key, &document.reviewer_key, &document.reviewer_role] {
            key(value, ledger)?;
        }
        for value in [
            &document.queue_raw_sha256,
            &document.subject_sha256,
            &document.context_sha256,
            &document.policy_sha256,
        ] {
            compared(&[value], ledger)?;
            hash(value, ledger)?;
        }
        if document.rationale.len() > 8192 {
            return Err(ledger.capacity());
        }
        compared(&[&document.rationale], ledger)?;
        match (document.disposition, &document.abstention_reason) {
            (Disposition::Abstain, Some(reason)) => key(reason, ledger)?,
            (Disposition::Abstain, None) | (_, Some(_)) => return Err(ContractError::Invalid),
            (_, None) if document.rationale.trim().is_empty() => {
                return Err(ContractError::Invalid);
            }
            (_, None) => {}
        }
        match (document.disposition, &document.supersedes) {
            (Disposition::Superseded, Some(prior)) => {
                uuid(&prior.response_id, ledger)?;
                compared(&[&prior.raw_sha256, &prior.response_id, &document.response_id], ledger)?;
                hash(&prior.raw_sha256, ledger)?;
                if prior.response_id == document.response_id {
                    return Err(ContractError::Invalid);
                }
            }
            (Disposition::Superseded, None) | (_, Some(_)) => return Err(ContractError::Invalid),
            (_, None) => {}
        }
        Ok(())
    })
}
/// Complete recorded counters are data, never an evaluation or currentness certificate.
fn counters(
    document: &DispositionsDocumentV3,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(12)?;
    ledger.bytes(12 * 64)?;
    let counts = &document.counts;
    if counts.items != 1
        || document.items.len() != 1
        || usize::try_from(counts.unique_responses).map_err(|_| ledger.capacity())?
            != document.responses.len()
        || counts.response_files > 10_000
        || counts.response_files
            != counts
                .unique_responses
                .checked_add(counts.exact_duplicates)
                .ok_or_else(|| ledger.capacity())?
    {
        return Err(ContractError::Invalid);
    }
    let states = &counts.states;
    let reported = [
        states.unassigned,
        states.assigned,
        states.in_review,
        states.conflicted,
        states.changes_requested,
        states.quorum_met,
        states.expired,
        states.stale,
    ];
    let selected = match document.items[0].state {
        ItemState::Unassigned => 0,
        ItemState::Assigned => 1,
        ItemState::InReview => 2,
        ItemState::Conflicted => 3,
        ItemState::ChangesRequested => 4,
        ItemState::QuorumMet => 5,
        ItemState::Expired => 6,
        ItemState::Stale => 7,
    };
    let mut actual = [0_u32; 8];
    actual[selected] = 1;
    if reported != actual {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Validate the whole inert recorded roster, witnesses, dissent, counters and nullable claim.
pub(crate) fn dispositions(
    document: &DispositionsDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        crate::review::encode_v3::dispositions_operands(document, ledger, control)?;
        compared(&[&document.schema_version, &document.identity_disclaimer], ledger)?;
        if document.schema_version != "forge.review-dispositions/3"
            || document.identity_disclaimer != IDENTITY_DISCLAIMER
        {
            return Err(ContractError::Invalid);
        }
        uuid(&document.queue_id, ledger)?;
        compared(&[&document.queue_raw_sha256], ledger)?;
        hash(&document.queue_raw_sha256, ledger)?;
        time(&document.as_of, ledger)?;
        match (document.currentness, &document.closure_generation) {
            (RecordedCurrentness::Unverified, None) => {}
            (RecordedCurrentness::RecordedCurrent, Some(value)) => {
                compared(&[value], ledger)?;
                hash(value, ledger)?;
            }
            _ => return Err(ContractError::Invalid),
        }
        crate::review::hash_v3::validate_public_pins(&document.source_pins, ledger, control)?;
        recorded_data::rows(&document.responses, ledger, control)?;
        if document.items.len() != 1 {
            return Err(ContractError::Invalid);
        }
        let item = &document.items[0];
        if item.reason_codes.len() > 32
            || item.response_ids.len() > 10_000
            || item.dissent_ids.len() > 10_000
            || item.met_seats.len() > 100
            || item.unmet_seats.len() > 100
        {
            return Err(ContractError::Invalid);
        }
        recorded_data::item(item, &document.responses, ledger, control)?;
        counters(document, ledger)
    })
}
/// Decode actual borrowed original Response/3 bytes, preserving preflight registration/edit order.
pub(crate) fn decode_response<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV3<'a, ResponseDocumentV3>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let (document, raw_sha256) = operational_raw::admitted_document(
            raw,
            MAX_RESPONSE_BYTES,
            include_str!("../../schemas/forge.review-response-3.schema.json"),
            true,
            ledger,
            control,
        )?;
        response(&document, ledger, control)?;
        Ok(DecodedV3 { raw, raw_sha256, document })
    })
}
/// Decode complete recorded Dispositions/3 claims, without issuing currentness or approval.
pub(crate) fn decode_dispositions<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV3<'a, DispositionsDocumentV3>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let (document, raw_sha256) = operational_raw::admitted_document(
            raw,
            MAX_RECORDED_BYTES,
            include_str!("../../schemas/forge.review-dispositions-3.schema.json"),
            false,
            ledger,
            control,
        )?;
        dispositions(&document, ledger, control)?;
        Ok(DecodedV3 { raw, raw_sha256, document })
    })
}

#[cfg(test)]
#[path = "operational_v3_codec_tests.rs"]
/// Genuine plain byte/record/counter/fence controls; no native success fixtures.
mod tests;

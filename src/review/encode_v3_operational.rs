//! Finite separately closed plain Response/3 and Dispositions/3 output/readback.
use super::{fixed, optional, pin, strings, text};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::decode_v2::{checkpoint, phase};
use crate::review::wire_v3::{
    DispositionCounts, DispositionsDocumentV3, ItemDisposition, MetSeat, RecordedResponse,
    ResponseDocumentV3, StateCounts, UnmetSeat,
};
use crate::workspace::preparation::WorkControl;

/// Every complete response field, including private evidence and required option tags.
pub(crate) fn response_operands(
    value: &ResponseDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let ResponseDocumentV3 {
            schema_version,
            identity_disclaimer,
            response_id,
            queue_id,
            queue_raw_sha256,
            item_key,
            item_id,
            domain: _,
            adapter_version,
            requested_action: _,
            source_pins,
            subject_sha256,
            context_sha256,
            policy_sha256,
            reviewer_key,
            reviewer_role,
            disposition: _,
            responded_at,
            rationale,
            abstention_reason,
            proposed_edit: (),
            supersedes,
        } = value;
        fixed(5, ledger)?;
        for value in [
            schema_version,
            identity_disclaimer,
            response_id,
            queue_id,
            queue_raw_sha256,
            item_key,
            item_id,
            adapter_version,
            subject_sha256,
            context_sha256,
            policy_sha256,
            reviewer_key,
            reviewer_role,
            responded_at,
            rationale,
        ] {
            text(value, ledger)?;
        }
        optional(abstention_reason.as_deref(), ledger)?;
        fixed(1, ledger)?;
        if let Some(value) = supersedes {
            text(&value.response_id, ledger)?;
            text(&value.raw_sha256, ledger)?;
        }
        fixed(1, ledger)?;
        for value in source_pins {
            checkpoint(ledger, control)?;
            pin(value, ledger)?;
        }
        Ok(())
    })
}
/// All minimized original response fields; no rationale is present in recorded data.
fn recorded_row(
    value: &RecordedResponse,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let RecordedResponse {
        response_id,
        raw_sha256,
        byte_length: _,
        item_key,
        reviewer_key,
        reviewer_role,
        disposition: _,
        responded_at,
        classification: _,
    } = value;
    fixed(3, ledger)?;
    for value in [response_id, raw_sha256, item_key, reviewer_key, reviewer_role, responded_at] {
        text(value, ledger)?;
    }
    Ok(())
}
/// Every met-seat identity and ordinal, without performing matching or authenticating a key.
fn met(value: &MetSeat, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let MetSeat { role_key, ordinal: _, reviewer_key, response_id } = value;
    fixed(1, ledger)?;
    for value in [role_key, reviewer_key, response_id] {
        text(value, ledger)?;
    }
    Ok(())
}
/// Preserve complete unmatched role/ordinal data as ordinary recorded evidence.
fn unmet(value: &UnmetSeat, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let UnmetSeat { role_key, ordinal: _ } = value;
    fixed(1, ledger)?;
    text(role_key, ledger)
}
/// Complete item status, witness order, response roster and historical dissent operands.
fn item(
    value: &ItemDisposition,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let ItemDisposition {
        item_key,
        item_id,
        state: _,
        reason_codes,
        required_seats: _,
        met_seats,
        unmet_seats,
        response_ids,
        dissent_ids,
        blocking: _,
    } = value;
    fixed(5, ledger)?;
    text(item_key, ledger)?;
    text(item_id, ledger)?;
    strings(reason_codes, ledger)?;
    strings(response_ids, ledger)?;
    strings(dissent_ids, ledger)?;
    fixed(2, ledger)?;
    for value in met_seats {
        checkpoint(ledger, control)?;
        met(value, ledger)?;
    }
    for value in unmet_seats {
        checkpoint(ledger, control)?;
        unmet(value, ledger)?;
    }
    Ok(())
}
/// Exhaustive four occurrence denominators and eight state counts.
fn counts(value: &DispositionCounts, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let DispositionCounts {
        items: _,
        response_files: _,
        unique_responses: _,
        exact_duplicates: _,
        states,
    } = value;
    let StateCounts {
        unassigned: _,
        assigned: _,
        in_review: _,
        conflicted: _,
        changes_requested: _,
        quorum_met: _,
        expired: _,
        stale: _,
    } = states;
    fixed(12, ledger)
}
/// All eleven recorded-document fields and every full nested operand before repeated work.
pub(crate) fn dispositions_operands(
    value: &DispositionsDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let DispositionsDocumentV3 {
            schema_version,
            identity_disclaimer,
            queue_id,
            queue_raw_sha256,
            as_of,
            currentness: _,
            closure_generation,
            source_pins,
            responses,
            items,
            counts: totals,
        } = value;
        fixed(7, ledger)?;
        for value in [schema_version, identity_disclaimer, queue_id, queue_raw_sha256, as_of] {
            text(value, ledger)?;
        }
        optional(closure_generation.as_deref(), ledger)?;
        for value in source_pins {
            checkpoint(ledger, control)?;
            pin(value, ledger)?;
        }
        for value in responses {
            checkpoint(ledger, control)?;
            recorded_row(value, ledger)?;
        }
        for value in items {
            checkpoint(ledger, control)?;
            item(value, ledger, control)?;
        }
        counts(totals, ledger)
    })
}
/// Exhaustive semantic equality of both complete ordinary response declarations.
pub(crate) fn compare_responses(
    expected: &ResponseDocumentV3,
    actual: &ResponseDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        response_operands(expected, ledger, control)?;
        response_operands(actual, ledger, control)?;
        if expected != actual {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Exhaustive semantic equality of the whole ordinary recorded document and all witnesses.
pub(crate) fn compare_dispositions(
    expected: &DispositionsDocumentV3,
    actual: &DispositionsDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        dispositions_operands(expected, ledger, control)?;
        dispositions_operands(actual, ledger, control)?;
        if expected != actual {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Strict actual response bytes, including original whitespace/LF, precede complete Eq.
pub(crate) fn readback_response(
    expected: &ResponseDocumentV3,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let actual = crate::review::decode_v3::decode_response(raw, ledger, control)?;
        compare_responses(expected, actual.document(), ledger, control)
    })
}
/// Strict complete recorded original bytes precede whole typed Eq, without a selected projection.
pub(crate) fn readback_dispositions(
    expected: &DispositionsDocumentV3,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let actual = crate::review::decode_v3::decode_dispositions(raw, ledger, control)?;
        compare_dispositions(expected, actual.document(), ledger, control)
    })
}
/// Finite compact immutable private response plus exactly one LF and actual-byte readback.
/// Output remains plain data; native binding/publication are separate actual owner duties.
pub(crate) fn response(
    document: &ResponseDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    phase(ledger, control, |ledger, control| {
        crate::review::decode_v3::response(document, ledger, control)?;
        response_operands(document, ledger, control)?;
        let raw = crate::review::encode_v2::admitted_json(document, 1_048_576, ledger, control)?;
        readback_response(document, &raw, ledger, control)?;
        Ok(raw)
    })
}
/// Finite complete recorded bundle plus LF; claimed currentness/quorum stays inert.
pub(crate) fn dispositions(
    document: &DispositionsDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    phase(ledger, control, |ledger, control| {
        crate::review::decode_v3::dispositions(document, ledger, control)?;
        dispositions_operands(document, ledger, control)?;
        let raw = crate::review::encode_v2::admitted_json(document, 33_554_432, ledger, control)?;
        readback_dispositions(document, &raw, ledger, control)?;
        Ok(raw)
    })
}

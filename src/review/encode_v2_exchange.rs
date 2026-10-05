//! Pure typed finite /2 queue/response encoding, exact strict readback and complete equality.
//! Ordinary declaration/byte results cannot issue native ownership or authorize publication.
use super::{encoded, fixed, optional, pin, strings, text};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::decode_v2;
use crate::workspace::preparation::WorkControl;

/// Full queue cap, including LF, unchanged from the recorded /2 input contract.
const MAX_QUEUE_BYTES: usize = 10_485_760;
/// Full immutable private response cap, including LF, without renewed allowance.
const MAX_RESPONSE_BYTES: usize = 1_048_576;

/// Complete asserted seat operand; count remains a positive declared denominator.
fn seat_operands(
    value: &crate::review::wire::SeatRequirement,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let crate::review::wire::SeatRequirement { role_key, count: _ } = value;
    fixed(1, ledger)?;
    text(role_key, ledger)
}
/// Complete explicit substitution tuple; no field is an authenticated authority.
fn substitution_operands(
    value: &crate::review::wire::Substitution,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let crate::review::wire::Substitution { seat_role, reviewer_key, asserted_role, reason_code } =
        value;
    for value in [seat_role, reviewer_key, asserted_role, reason_code] {
        text(value, ledger)?;
    }
    Ok(())
}
/// Exact assignment pair, admitting both complete original strings before inspection.
fn assignment_operands(
    value: &crate::review::wire::Assignment,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let crate::review::wire::Assignment { reviewer_key, role_key } = value;
    text(reviewer_key, ledger)?;
    text(role_key, ledger)
}
/// Exhaustive full policy operands, including unused declared substitutions/reasons.
fn policy_operands(
    value: &crate::review::wire::ReviewPolicy,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let crate::review::wire::ReviewPolicy {
        key,
        seats,
        substitutions,
        abstention_rule: _,
        empty_abstention_reasons,
        author_separation: _,
    } = value;
    fixed(4, ledger)?;
    text(key, ledger)?;
    strings(empty_abstention_reasons, ledger)?;
    ledger.visits(seats.len())?;
    for value in seats {
        seat_operands(value, ledger)?;
    }
    ledger.visits(substitutions.len())?;
    for value in substitutions {
        substitution_operands(value, ledger)?;
    }
    Ok(())
}
/// Full minimized context list operands, without inference from native record prose.
fn context_operands(
    value: &crate::review::wire::ContextSnapshot,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let crate::review::wire::ContextSnapshot { reason_codes, related_subject_ids } = value;
    strings(reason_codes, ledger)?;
    strings(related_subject_ids, ledger)
}
/// Exhaustive single item declaration; exact supplied lists/nullable due-at survive.
fn queue_item_operands(
    value: &crate::review::wire_v2::ReviewItemV2,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let crate::review::wire_v2::ReviewItemV2 {
        key,
        item_id,
        domain: _,
        adapter_version,
        subject_id,
        requested_action: _,
        source_keys,
        subject_sha256,
        context,
        context_sha256,
        policy_key,
        policy_sha256,
        author_keys,
        assignments,
        due_at,
        allowed_dispositions,
    } = value;
    let scalar_fields =
        allowed_dispositions.len().checked_add(4).ok_or_else(|| ledger.capacity())?;
    fixed(scalar_fields, ledger)?;
    for value in [
        key,
        item_id,
        adapter_version,
        subject_id,
        subject_sha256,
        context_sha256,
        policy_key,
        policy_sha256,
    ] {
        text(value, ledger)?;
    }
    strings(source_keys, ledger)?;
    strings(author_keys, ledger)?;
    context_operands(context, ledger)?;
    optional(due_at.as_deref(), ledger)?;
    ledger.visits(assignments.len())?;
    for value in assignments {
        assignment_operands(value, ledger)?;
    }
    Ok(())
}
/// All ten queue fields, full registries and every nested original policy/item tuple.
fn queue_operands(
    value: &crate::review::wire_v2::QueueDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let crate::review::wire_v2::QueueDocumentV2 {
        schema_version,
        identity_disclaimer,
        sensitivity: _,
        queue_id,
        created_at,
        source_pins,
        roles,
        reviewers,
        policies,
        items,
    } = value;
    decode_v2::checkpoint(ledger, control)?;
    fixed(6, ledger)?;
    for value in [schema_version, identity_disclaimer, queue_id, created_at] {
        text(value, ledger)?;
    }
    ledger.visits(source_pins.len())?;
    for value in source_pins {
        decode_v2::checkpoint(ledger, control)?;
        pin(value, ledger)?;
    }
    ledger.visits(roles.len())?;
    for value in roles {
        decode_v2::checkpoint(ledger, control)?;
        let crate::review::wire::RoleDefinition { key } = value;
        text(key, ledger)?;
    }
    ledger.visits(reviewers.len())?;
    for value in reviewers {
        decode_v2::checkpoint(ledger, control)?;
        let crate::review::wire::Reviewer { key, role_keys } = value;
        text(key, ledger)?;
        strings(role_keys, ledger)?;
    }
    ledger.visits(policies.len())?;
    for value in policies {
        decode_v2::checkpoint(ledger, control)?;
        policy_operands(value, ledger)?;
    }
    ledger.visits(items.len())?;
    for value in items {
        decode_v2::checkpoint(ledger, control)?;
        queue_item_operands(value, ledger)?;
    }
    Ok(())
}
/// Admit exact prior assertion fields, preserving explicit null and original hash/UUID.
fn supersedes_operands(
    value: Option<&crate::review::wire::SupersessionReference>,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    fixed(1, ledger)?;
    if let Some(value) = value {
        let crate::review::wire::SupersessionReference { response_id, raw_sha256 } = value;
        text(response_id, ledger)?;
        text(raw_sha256, ledger)?;
    }
    Ok(())
}
/// Complete immutable response operands, including private rationale and every explicit null.
fn response_operands(
    value: &crate::review::wire_v2::ResponseDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let crate::review::wire_v2::ResponseDocumentV2 {
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
    decode_v2::checkpoint(ledger, control)?;
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
    supersedes_operands(supersedes.as_ref(), ledger)?;
    ledger.visits(source_pins.len())?;
    for value in source_pins {
        decode_v2::checkpoint(ledger, control)?;
        pin(value, ledger)?;
    }
    Ok(())
}
/// Real finite queue-byte decoder and full equality; this cannot bind native source facts.
fn readback_queue(
    expected: &crate::review::wire_v2::QueueDocumentV2,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let actual = decode_v2::decode_queue(raw, ledger, control)?;
        queue_operands(expected, ledger, control)?;
        queue_operands(actual.document(), ledger, control)?;
        if actual.document() != expected {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Real finite private response-byte decoder and complete original typed equality.
fn readback_response(
    expected: &crate::review::wire_v2::ResponseDocumentV2,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        let actual = decode_v2::decode_response(raw, ledger, control)?;
        response_operands(expected, ledger, control)?;
        response_operands(actual.document(), ledger, control)?;
        if actual.document() != expected {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Finite plain /2 queue JSON+LF and exact strict typed readback under the original work owner.
/// Root separately binds the genuine native closure and retains its final publication fence.
pub(crate) fn queue(
    document: &crate::review::wire_v2::QueueDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        decode_v2::queue(document, ledger, control)?;
        queue_operands(document, ledger, control)?;
        let output = encoded(document, MAX_QUEUE_BYTES, ledger, control)?;
        readback_queue(document, &output, ledger, control)?;
        Ok(output)
    })
}
/// Finite plain immutable /2 response JSON+LF and strict exact typed readback.
/// Root separately binds its exact captured queue and physical no-replace publication fence.
pub(crate) fn response(
    document: &crate::review::wire_v2::ResponseDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        decode_v2::response(document, ledger, control)?;
        response_operands(document, ledger, control)?;
        let output = encoded(document, MAX_RESPONSE_BYTES, ledger, control)?;
        readback_response(document, &output, ledger, control)?;
        Ok(output)
    })
}

#[cfg(test)]
#[path = "encode_v2_exchange_tests.rs"]
/// Prospective genuine finite typed queue/response controls over ordinary declarations.
mod exchange_tests;

/// Compare complete plain Queue operands using the existing exhaustive accounting.
/// Equal typed declarations confer no raw/native ownership or currentness.
pub(crate) fn compare_queues(
    expected: &crate::review::wire_v2::QueueDocumentV2,
    actual: &crate::review::wire_v2::QueueDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    decode_v2::phase(ledger, control, |ledger, control| {
        queue_operands(expected, ledger, control)?;
        queue_operands(actual, ledger, control)?;
        if actual != expected {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}

//! Finite plain Queue/3 encoding, strict actual-byte readback and exhaustive equality.
//! No encoded byte vector supplies native ownership or publication authority.
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase};
use super::wire_v3::{
    Assignment, ContextSnapshot, QueueDocumentV3, ReviewItemV3, ReviewPolicy, SeatRequirement,
    SourcePinV3, Substitution,
};
use crate::workspace::preparation::WorkControl;

/// Complete Queue raw ceiling, including the single emitted LF.
pub(crate) const MAX_QUEUE_BYTES: usize = 10_485_760;

/// Admit a complete repeated string operand before serialization or equality.
fn text(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.bytes(value.len())
}
/// Admit bounded tags, list cardinalities and exact scalar comparisons.
fn fixed(count: usize, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(count)?;
    let bytes = count.checked_mul(64).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(bytes)
}
/// Preserve an explicit optional tag and full present value, including present-empty.
fn optional(value: Option<&str>, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    fixed(1, ledger)?;
    if let Some(value) = value {
        text(value, ledger)?;
    }
    Ok(())
}
/// Admit every complete string in its actual order without sorting or omission.
fn strings(values: &[String], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    fixed(1, ledger)?;
    for value in values {
        text(value, ledger)?;
    }
    Ok(())
}
/// Exhaustive eight-field pin operands; no native model/root/schema is discarded.
fn pin(value: &SourcePinV3, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let SourcePinV3 {
        artifact_key,
        kind: _,
        raw_sha256,
        byte_length: _,
        schema_identity,
        validation_profile,
        native_model: _,
        native_root_uuid,
    } = value;
    fixed(4, ledger)?;
    for value in [artifact_key, raw_sha256, validation_profile] {
        text(value, ledger)?;
    }
    optional(schema_identity.as_deref(), ledger)?;
    optional(native_root_uuid.as_deref(), ledger)
}
/// Complete positive asserted seat row; this does not perform matching.
fn seat(value: &SeatRequirement, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let SeatRequirement { role_key, count: _ } = value;
    fixed(1, ledger)?;
    text(role_key, ledger)
}
/// Complete asserted substitution identity and reason, including unused rows.
fn substitution(value: &Substitution, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let Substitution { seat_role, reviewer_key, asserted_role, reason_code } = value;
    for value in [seat_role, reviewer_key, asserted_role, reason_code] {
        text(value, ledger)?;
    }
    Ok(())
}
/// Exhaustive whole policy lists and fixed nonapproving/author-separation tags.
fn policy(value: &ReviewPolicy, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let ReviewPolicy {
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
    for value in seats {
        seat(value, ledger)?;
    }
    for value in substitutions {
        substitution(value, ledger)?;
    }
    Ok(())
}
/// Preserve both complete assignment strings in the supplied array order.
fn assignment(value: &Assignment, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let Assignment { reviewer_key, role_key } = value;
    text(reviewer_key, ledger)?;
    text(role_key, ledger)
}
/// Both exact context lists are ordinary data, never a native-plan summary.
fn context(value: &ContextSnapshot, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let ContextSnapshot { reason_codes, related_subject_ids } = value;
    strings(reason_codes, ledger)?;
    strings(related_subject_ids, ledger)
}
/// Exhaustive full item fields, including all lists and explicit nullable due-at.
fn item(value: &ReviewItemV3, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let ReviewItemV3 {
        key,
        item_id,
        domain: _,
        adapter_version,
        subject_id,
        requested_action: _,
        source_keys,
        subject_sha256,
        context: snapshot,
        context_sha256,
        policy_key,
        policy_sha256,
        author_keys,
        assignments,
        due_at,
        allowed_dispositions,
    } = value;
    fixed(allowed_dispositions.len().checked_add(6).ok_or_else(|| ledger.capacity())?, ledger)?;
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
    context(snapshot, ledger)?;
    optional(due_at.as_deref(), ledger)?;
    for value in assignments {
        assignment(value, ledger)?;
    }
    Ok(())
}
/// All ten fields and every full nested scalar/option/list before repeated work.
/// This is accounting only; schema/semantic/native facts are separately checked.
pub(crate) fn queue_operands(
    value: &QueueDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let QueueDocumentV3 {
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
        fixed(6, ledger)?;
        for value in [schema_version, identity_disclaimer, queue_id, created_at] {
            text(value, ledger)?;
        }
        for value in source_pins {
            checkpoint(ledger, control)?;
            pin(value, ledger)?;
        }
        for value in roles {
            checkpoint(ledger, control)?;
            text(&value.key, ledger)?;
        }
        for value in reviewers {
            checkpoint(ledger, control)?;
            text(&value.key, ledger)?;
            strings(&value.role_keys, ledger)?;
        }
        for value in policies {
            checkpoint(ledger, control)?;
            policy(value, ledger)?;
        }
        for value in items {
            checkpoint(ledger, control)?;
            item(value, ledger)?;
        }
        Ok(())
    })
}
/// Compare all complete typed declarations after both full operand admissions.
/// Equal ordinary data carries no raw/native/capture/currentness authority.
pub(crate) fn compare_queues(
    expected: &QueueDocumentV3,
    actual: &QueueDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        queue_operands(expected, ledger, control)?;
        queue_operands(actual, ledger, control)?;
        if actual != expected {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}
/// Strictly read the actual complete output bytes, then compare every typed field.
pub(crate) fn readback_queue(
    expected: &QueueDocumentV3,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let actual = super::decode_v3::decode_queue(raw, ledger, control)?;
        compare_queues(expected, actual.document(), ledger, control)
    })
}
/// Compact finite Queue/3 JSON plus LF under the original monotonic ledger.
/// Native binding and final physical/publication fences remain actual owner duties.
pub(crate) fn queue(
    document: &QueueDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    phase(ledger, control, |ledger, control| {
        super::decode_v3::queue(document, ledger, control)?;
        queue_operands(document, ledger, control)?;
        let output = super::encode_v2::admitted_json(document, MAX_QUEUE_BYTES, ledger, control)?;
        readback_queue(document, &output, ledger, control)?;
        Ok(output)
    })
}

/// Separately closed finite Response/3 and recorded Dispositions/3 output.
#[path = "encode_v3_operational.rs"]
mod operational;
pub(crate) use operational::{
    compare_dispositions, compare_responses, dispositions, dispositions_operands, response,
    response_operands,
};
#[cfg(test)]
pub(crate) use operational::{readback_dispositions, readback_response};

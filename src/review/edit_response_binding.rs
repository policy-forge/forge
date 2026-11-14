//! A-authored ordinary /3 binding and complete raw-original relation.
//! These declarations/correlations authenticate no reviewer or native currentness.
use super::decode::{ContractError, ContractLedger, OriginalRelation};
use super::decode_v2::phase;
use super::decode_v4::{self, DecodedV3};
use super::wire_v4::{
    Disposition, DispositionsDocumentV4, ProposedEdit, QueueDocumentV4, ResponseDocumentV4,
    ReviewItemV3, ReviewPolicy, SourcePinV3,
};
use crate::workspace::preparation::WorkControl;

/// Charge every actual operand before inspecting or comparing full strings.
fn strings(values: &[&str], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(values.len())?;
    for value in values {
        ledger.bytes(value.len())?;
    }
    Ok(())
}
/// Exact ordinary equality after both complete original UTF-8 extents are admitted.
fn same(left: &str, right: &str, ledger: &mut ContractLedger) -> Result<bool, ContractError> {
    strings(&[left, right], ledger)?;
    Ok(left == right)
}
/// Exhaustive eight-field operand accounting before any pin-list equality.
pub(super) fn compared_pins(
    left: &[SourcePinV3],
    right: &[SourcePinV3],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    for pins in [left, right] {
        ledger.visits(pins.len())?;
        for pin in pins {
            let SourcePinV3 {
                artifact_key,
                kind: _,
                raw_sha256,
                byte_length: _,
                schema_identity,
                validation_profile,
                native_model: _,
                native_root_uuid,
            } = pin;
            strings(
                &[
                    artifact_key,
                    raw_sha256,
                    validation_profile,
                    schema_identity.as_deref().unwrap_or(""),
                    native_root_uuid.as_deref().unwrap_or(""),
                ],
                ledger,
            )?;
            ledger.bytes(64)?;
        }
    }
    Ok(())
}
/// All declared reviewer-role/author/assignment/substitution rows remain full operands.
fn eligible(
    queue: &QueueDocumentV4,
    item: &ReviewItemV3,
    policy: &ReviewPolicy,
    reviewer_key: &str,
    reviewer_role: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let mut membership = false;
    for reviewer in &queue.reviewers {
        super::decode_v2::checkpoint(ledger, control)?;
        let key = same(&reviewer.key, reviewer_key, ledger)?;
        for role in &reviewer.role_keys {
            let matched = same(role, reviewer_role, ledger)?;
            membership |= key && matched;
        }
    }
    let mut author = false;
    for key in &item.author_keys {
        author |= same(key, reviewer_key, ledger)?;
    }
    let mut assigned = false;
    for row in &item.assignments {
        super::decode_v2::checkpoint(ledger, control)?;
        let key = same(&row.reviewer_key, reviewer_key, ledger)?;
        let role = same(&row.role_key, reviewer_role, ledger)?;
        assigned |= key && role;
    }
    for row in &policy.substitutions {
        super::decode_v2::checkpoint(ledger, control)?;
        let key = same(&row.reviewer_key, reviewer_key, ledger)?;
        let role = same(&row.asserted_role, reviewer_role, ledger)?;
        assigned |= key && role;
    }
    Ok(membership && !author && assigned)
}
/// Maximum bounded proposed patch text, in UTF-8 bytes (PRD 068 M-10 envelope).
const MAX_PATCH_BYTES: usize = 64 * 1024;

/// Structural edit-envelope syntax: bounded nonempty patch, exact digest grammar and
/// exact patch-byte integrity. Deliberately does NOT check the target binding, so a
/// wrong-target edit still decodes and is classified per response at merge (AC-5).
pub(crate) fn edit_structural(
    edit: &ProposedEdit,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    phase(ledger, control, |ledger, control| {
        strings(&[&edit.target_subject_sha256, &edit.patch, &edit.patch_sha256], ledger)?;
        if edit.patch.is_empty() || edit.patch.len() > MAX_PATCH_BYTES {
            return Ok(false);
        }
        for value in [&edit.target_subject_sha256, &edit.patch_sha256] {
            if value.len() != 64
                || !value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Ok(false);
            }
        }
        let digest =
            phase(ledger, control, |_, _| Ok(crate::hashing::sha256_hex(edit.patch.as_bytes())))?;
        same(&digest, &edit.patch_sha256, ledger)
    })
}

/// Complete proposed-edit validity: structural syntax plus the exact target binding.
/// A wrong target is stale (handled by the merge); a malformed envelope is invalid-edit.
pub(crate) fn edit_well_formed(
    edit: &ProposedEdit,
    subject_sha256: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    phase(ledger, control, |ledger, control| {
        if !same(&edit.target_subject_sha256, subject_sha256, ledger)? {
            return Ok(false);
        }
        edit_structural(edit, ledger, control)
    })
}

/// Bind complete separately typed assertions to the actual raw Queue snapshot.
/// Merge keeps foreign/stale history and calls this only at the eligibility phase.
pub(crate) fn bind_response(
    queue: &DecodedV3<'_, QueueDocumentV4>,
    response: &DecodedV3<'_, ResponseDocumentV4>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let queue_doc = queue.document();
        let response = response.document();
        let item = queue_doc.items.first().ok_or(ContractError::Binding)?;
        let policy = queue_doc.policies.first().ok_or(ContractError::Binding)?;
        compared_pins(&response.source_pins, &queue_doc.source_pins, ledger)?;
        strings(
            &[
                &response.queue_id,
                &queue_doc.queue_id,
                &response.queue_raw_sha256,
                queue.raw_sha256(),
                &response.item_key,
                &item.key,
                &response.item_id,
                &item.item_id,
                &response.adapter_version,
                &item.adapter_version,
                &response.subject_sha256,
                &item.subject_sha256,
                &response.context_sha256,
                &item.context_sha256,
                &response.policy_sha256,
                &item.policy_sha256,
                &response.responded_at,
                &queue_doc.created_at,
            ],
            ledger,
        )?;
        ledger.bytes(32)?;
        if response.queue_id != queue_doc.queue_id
            || response.queue_raw_sha256 != queue.raw_sha256()
            || response.item_key != item.key
            || response.item_id != item.item_id
            || response.domain != item.domain
            || response.adapter_version != item.adapter_version
            || response.requested_action != item.requested_action
            || response.source_pins != queue_doc.source_pins
            || response.subject_sha256 != item.subject_sha256
            || response.context_sha256 != item.context_sha256
            || response.policy_sha256 != item.policy_sha256
            || response.responded_at < queue_doc.created_at
        {
            return Err(ContractError::Binding);
        }
        if !eligible(
            queue_doc,
            item,
            policy,
            &response.reviewer_key,
            &response.reviewer_role,
            ledger,
            control,
        )? {
            return Err(ContractError::Binding);
        }
        ledger.bytes(response.rationale.len())?;
        if response.disposition == Disposition::Abstain && response.rationale.trim().is_empty() {
            let mut permitted = false;
            for reason in &policy.empty_abstention_reasons {
                super::decode_v2::checkpoint(ledger, control)?;
                strings(&[reason, response.abstention_reason.as_deref().unwrap_or("")], ledger)?;
                permitted |= response.abstention_reason.as_deref() == Some(reason.as_str());
            }
            if !permitted {
                return Err(ContractError::Binding);
            }
        }
        Ok(())
    })
}
/// Same identity and equal complete raw bytes are the only exact duplicate relation.
pub(crate) fn original_relation(
    left: &DecodedV3<'_, ResponseDocumentV4>,
    right: &DecodedV3<'_, ResponseDocumentV4>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<OriginalRelation, ContractError> {
    phase(ledger, control, |ledger, _| {
        if !same(&left.document().response_id, &right.document().response_id, ledger)? {
            return Ok(OriginalRelation::DifferentIdentity);
        }
        let extent =
            left.raw().len().checked_add(right.raw().len()).ok_or_else(|| ledger.capacity())?;
        ledger.bytes(extent)?;
        Ok(if left.raw() == right.raw() {
            OriginalRelation::ExactDuplicate
        } else {
            OriginalRelation::IdentityConflict
        })
    })
}
/// Bind exactly the matched seat, rather than eligibility for some other seat.
fn seat_eligible(
    queue: &QueueDocumentV4,
    item: &ReviewItemV3,
    policy: &ReviewPolicy,
    asserted: (&str, &str, &str),
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let (key, role, seat) = asserted;
    if !eligible(queue, item, policy, key, role, ledger, control)? {
        return Ok(false);
    }
    let same_role = same(role, seat, ledger)?;
    let mut found = false;
    for row in &item.assignments {
        super::decode_v2::checkpoint(ledger, control)?;
        let key_match = same(&row.reviewer_key, key, ledger)?;
        let role_match = same(&row.role_key, role, ledger)?;
        found |= key_match && role_match && same_role;
    }
    for row in &policy.substitutions {
        super::decode_v2::checkpoint(ledger, control)?;
        let key_match = same(&row.reviewer_key, key, ledger)?;
        let role_match = same(&row.asserted_role, role, ledger)?;
        let seat_match = same(&row.seat_role, seat, ledger)?;
        found |= key_match && role_match && seat_match;
    }
    Ok(found)
}
/// Bind a recorded witness's asserted key/role to the complete actual Queue policy.
fn witnesses(
    queue: &QueueDocumentV4,
    item: &ReviewItemV3,
    policy: &ReviewPolicy,
    document: &DispositionsDocumentV4,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let result = document.items.first().ok_or(ContractError::Binding)?;
    let mut required = 0_u32;
    for seat in &policy.seats {
        super::decode_v2::checkpoint(ledger, control)?;
        required = required.checked_add(seat.count).ok_or_else(|| ledger.capacity())?;
        for ordinal in 0..seat.count {
            ledger.visits(1)?;
            let mut occurrences = 0_usize;
            for met in &result.met_seats {
                let role = same(&met.role_key, &seat.role_key, ledger)?;
                ledger.bytes(8)?;
                if role && met.ordinal == ordinal {
                    occurrences = occurrences.checked_add(1).ok_or_else(|| ledger.capacity())?;
                }
            }
            for unmet in &result.unmet_seats {
                let role = same(&unmet.role_key, &seat.role_key, ledger)?;
                ledger.bytes(8)?;
                if role && unmet.ordinal == ordinal {
                    occurrences = occurrences.checked_add(1).ok_or_else(|| ledger.capacity())?;
                }
            }
            if occurrences != 1 {
                return Err(ContractError::Binding);
            }
        }
    }
    ledger.bytes(16)?;
    if required != result.required_seats
        || required == 0
        || required > 100
        || result.met_seats.len().checked_add(result.unmet_seats.len()) != Some(required as usize)
    {
        return Err(ContractError::Binding);
    }
    for met in &result.met_seats {
        let mut low = 0;
        let mut high = document.responses.len();
        let mut found = None;
        while low < high {
            super::decode_v2::checkpoint(ledger, control)?;
            let mid = low + (high - low) / 2;
            let row = &document.responses[mid];
            strings(&[&row.response_id, &met.response_id], ledger)?;
            match row.response_id.cmp(&met.response_id) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => {
                    found = Some(row);
                    break;
                }
            }
        }
        let row = found.ok_or(ContractError::Binding)?;
        strings(
            &[
                &row.reviewer_key,
                &met.reviewer_key,
                &row.item_key,
                &item.key,
                &row.responded_at,
                &queue.created_at,
                &document.as_of,
                item.due_at.as_deref().unwrap_or(""),
            ],
            ledger,
        )?;
        ledger.bytes(16)?;
        if row.reviewer_key != met.reviewer_key
            || row.item_key != item.key
            || row.disposition != Disposition::Approve
            || row.classification != super::wire_v4::ResponseClassification::Current
            || row.responded_at < queue.created_at
            || row.responded_at > document.as_of
            || item.due_at.as_ref().is_some_and(|due| row.responded_at.as_str() >= due.as_str())
            || !seat_eligible(
                queue,
                item,
                policy,
                (&row.reviewer_key, &row.reviewer_role, &met.role_key),
                ledger,
                control,
            )?
        {
            return Err(ContractError::Binding);
        }
    }
    Ok(())
}
/// Complete ordinary recorded-output/Queue correlation; loaded Current stays inert.
pub(crate) fn bind_dispositions(
    queue: &DecodedV3<'_, QueueDocumentV4>,
    recorded: &DecodedV3<'_, DispositionsDocumentV4>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        let q = queue.document();
        let d = recorded.document();
        decode_v4::dispositions(d, ledger, control)?;
        let item = q.items.first().ok_or(ContractError::Binding)?;
        let result = d.items.first().ok_or(ContractError::Binding)?;
        let policy = q.policies.first().ok_or(ContractError::Binding)?;
        compared_pins(&q.source_pins, &d.source_pins, ledger)?;
        strings(
            &[
                &q.queue_id,
                &d.queue_id,
                queue.raw_sha256(),
                &d.queue_raw_sha256,
                &q.created_at,
                &d.as_of,
                &item.key,
                &result.item_key,
                &item.item_id,
                &result.item_id,
            ],
            ledger,
        )?;
        if q.queue_id != d.queue_id
            || queue.raw_sha256() != d.queue_raw_sha256
            || q.source_pins != d.source_pins
            || d.as_of < q.created_at
            || d.items.len() != 1
            || item.key != result.item_key
            || item.item_id != result.item_id
        {
            return Err(ContractError::Binding);
        }
        witnesses(q, item, policy, d, ledger, control)
    })
}

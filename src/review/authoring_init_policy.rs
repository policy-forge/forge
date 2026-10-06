//! Strict private asserted Init declarations; no native owner or source proof is issued.
//! The complete original policy is parsed on the caller ledger, without a synthetic Queue.
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase};
use super::wire_v3::{AuthoringInitItem, AuthoringPlanInitV1, ReviewPolicy};
use crate::workspace::preparation::WorkControl;

/// Complete private policy ceiling, including every original byte and optional LF.
const MAX_POLICY_BYTES: usize = 1_048_576;

/// Admit complete compared strings before any repeated exact scan/equality.
fn compared(values: &[&str], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(values.len())?;
    for value in values {
        ledger.bytes(value.len())?;
    }
    Ok(())
}
/// Exact ASCII token grammar under the declared UTF-8 byte bound.
fn key(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    compared(&[value], ledger)?;
    super::validate::token(value, 128)
}
/// Exact real UTC calendar seconds; no offset or wall-clock normalization.
fn time(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let bytes = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(bytes)?;
    ledger.derived(128)?;
    super::validate::time(value)
}
/// Complete strictly increasing list without a copied/sorted temporary registry.
fn ordered<'a>(
    values: impl IntoIterator<Item = &'a str>,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let mut prior = None;
    for value in values {
        key(value, ledger)?;
        if let Some(previous) = prior {
            compared(&[previous, value], ledger)?;
            if previous >= value {
                return Err(ContractError::Invalid);
            }
        }
        prior = Some(value);
    }
    Ok(())
}
/// Scan every actual declared role before returning an exact ordinary index.
fn role_index(
    request: &AuthoringPlanInitV1,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, row) in request.roles.iter().enumerate() {
        compared(&[&row.key, key], ledger)?;
        if row.key == key {
            found = Some(index);
        }
    }
    found.ok_or(ContractError::Invalid)
}
/// Scan the complete actual reviewer registry; it grants no reviewer authentication.
fn reviewer_index(
    request: &AuthoringPlanInitV1,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, row) in request.reviewers.iter().enumerate() {
        compared(&[&row.key, key], ledger)?;
        if row.key == key {
            found = Some(index);
        }
    }
    found.ok_or(ContractError::Invalid)
}
/// Inspect every supplied list operand before returning plain membership.
fn contains(
    values: &[String],
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<bool, ContractError> {
    let mut found = false;
    for value in values {
        compared(&[value, key], ledger)?;
        found |= value == key;
    }
    Ok(found)
}
/// Complete declared key/membership order before policy eligibility or graph work.
fn roster(
    request: &AuthoringPlanInitV1,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    if !(1..=32).contains(&request.roles.len()) || !(1..=100).contains(&request.reviewers.len()) {
        return Err(ContractError::Invalid);
    }
    ordered(request.roles.iter().map(|r| r.key.as_str()), ledger)?;
    ordered(request.reviewers.iter().map(|r| r.key.as_str()), ledger)?;
    let mut memberships = 0_usize;
    for row in &request.reviewers {
        checkpoint(ledger, control)?;
        if !(1..=32).contains(&row.role_keys.len()) {
            return Err(ContractError::Invalid);
        }
        ordered(row.role_keys.iter().map(String::as_str), ledger)?;
        memberships =
            memberships.checked_add(row.role_keys.len()).ok_or_else(|| ledger.capacity())?;
        if memberships > 3200 {
            return Err(ledger.capacity());
        }
        for role in &row.role_keys {
            role_index(request, role, ledger)?;
        }
    }
    Ok(())
}
/// Complete selected seats/reasons/substitution order, before any graph initialization.
fn policy_order(
    policy: &ReviewPolicy,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    key(&policy.key, ledger)?;
    if !(1..=32).contains(&policy.seats.len())
        || policy.substitutions.len() > 3200
        || policy.empty_abstention_reasons.len() > 32
    {
        return Err(ContractError::Invalid);
    }
    ordered(policy.seats.iter().map(|s| s.role_key.as_str()), ledger)?;
    ordered(policy.empty_abstention_reasons.iter().map(String::as_str), ledger)?;
    let mut total = 0_usize;
    for seat in &policy.seats {
        ledger.visits(1)?;
        if !(1..=100).contains(&seat.count) {
            return Err(ContractError::Invalid);
        }
        let count = usize::try_from(seat.count).map_err(|_| ledger.capacity())?;
        total = total.checked_add(count).ok_or_else(|| ledger.capacity())?;
    }
    if total > 100 {
        return Err(ledger.capacity());
    }
    for row in &policy.substitutions {
        for value in [&row.seat_role, &row.reviewer_key, &row.asserted_role, &row.reason_code] {
            key(value, ledger)?;
        }
    }
    for pair in policy.substitutions.windows(2) {
        compared(
            &[
                &pair[0].seat_role,
                &pair[0].reviewer_key,
                &pair[0].asserted_role,
                &pair[1].seat_role,
                &pair[1].reviewer_key,
                &pair[1].asserted_role,
            ],
            ledger,
        )?;
        if (&pair[0].seat_role, &pair[0].reviewer_key, &pair[0].asserted_role)
            >= (&pair[1].seat_role, &pair[1].reviewer_key, &pair[1].asserted_role)
        {
            return Err(ContractError::Invalid);
        }
    }
    Ok(total)
}
/// Exact role-specific positive seat index, after complete comparison admission.
fn seat_index(
    policy: &ReviewPolicy,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<Option<usize>, ContractError> {
    let mut found = None;
    for (index, seat) in policy.seats.iter().enumerate() {
        compared(&[&seat.role_key, key], ledger)?;
        if seat.role_key == key {
            found = Some(index);
        }
    }
    Ok(found)
}
/// Resolve genuine asserted eligibility and author exclusion, never native authority.
fn eligible(
    request: &AuthoringPlanInitV1,
    item: &AuthoringInitItem,
    reviewer: &str,
    role: &str,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    let index = reviewer_index(request, reviewer, ledger)?;
    let row = &request.reviewers[index];
    if !contains(&row.role_keys, role, ledger)? || contains(&item.author_keys, reviewer, ledger)? {
        return Err(ContractError::Invalid);
    }
    Ok(index)
}
/// Complete role/reviewer incidence and finite graph cardinalities, without matching.
fn policy_graph(
    request: &AuthoringPlanInitV1,
    item: &AuthoringInitItem,
    policy: &ReviewPolicy,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let seats = policy_order(policy, ledger)?;
    if !(1..=100).contains(&item.author_keys.len()) || item.assignments.len() > 3200 {
        return Err(ContractError::Invalid);
    }
    ordered(item.author_keys.iter().map(String::as_str), ledger)?;
    for pair in item.assignments.windows(2) {
        compared(
            &[&pair[0].reviewer_key, &pair[0].role_key, &pair[1].reviewer_key, &pair[1].role_key],
            ledger,
        )?;
        if (&pair[0].reviewer_key, &pair[0].role_key) >= (&pair[1].reviewer_key, &pair[1].role_key)
        {
            return Err(ContractError::Invalid);
        }
    }
    ledger.derived(3300)?;
    ledger.visits(3300)?;
    let mut candidates = [[false; 100]; 32];
    let mut keys = [false; 100];
    for seat in &policy.seats {
        role_index(request, &seat.role_key, ledger)?;
    }
    for row in &item.assignments {
        checkpoint(ledger, control)?;
        key(&row.reviewer_key, ledger)?;
        key(&row.role_key, ledger)?;
        let reviewer = eligible(request, item, &row.reviewer_key, &row.role_key, ledger)?;
        if let Some(seat) = seat_index(policy, &row.role_key, ledger)? {
            candidates[seat][reviewer] = true;
        }
    }
    for row in &policy.substitutions {
        checkpoint(ledger, control)?;
        let reviewer = eligible(request, item, &row.reviewer_key, &row.asserted_role, ledger)?;
        let seat = seat_index(policy, &row.seat_role, ledger)?.ok_or(ContractError::Invalid)?;
        candidates[seat][reviewer] = true;
    }
    let mut edges = 0_usize;
    for (index, required) in policy.seats.iter().enumerate() {
        checkpoint(ledger, control)?;
        ledger.visits(100)?;
        let mut count = 0_usize;
        for (reviewer, present) in candidates[index].iter().enumerate() {
            if *present {
                count += 1;
                keys[reviewer] = true;
            }
        }
        let copies = usize::try_from(required.count).map_err(|_| ledger.capacity())?;
        edges = edges
            .checked_add(copies.checked_mul(count).ok_or_else(|| ledger.capacity())?)
            .ok_or_else(|| ledger.capacity())?;
    }
    if edges > 10_000 {
        return Err(ledger.capacity());
    }
    ledger.visits(100)?;
    let vertices =
        seats.checked_add(keys.iter().filter(|v| **v).count()).ok_or_else(|| ledger.capacity())?;
    ledger.graph(seats, edges, vertices)
}
/// Complete asserted policy semantics, with no native subject or quorum evaluation.
fn asserted(
    request: &AuthoringPlanInitV1,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    compared(&[&request.schema_version], ledger)?;
    if request.schema_version != "forge.review-authoring-plan-init/1"
        || request.policies.len() != 1
        || request.items.len() != 1
    {
        return Err(ContractError::Invalid);
    }
    roster(request, ledger, control)?;
    let item = &request.items[0];
    let policy = &request.policies[0];
    key(&item.key, ledger)?;
    key(&item.policy_key, ledger)?;
    compared(&[&item.policy_key, &policy.key], ledger)?;
    if item.policy_key != policy.key {
        return Err(ContractError::Invalid);
    }
    if let Some(due) = &item.due_at {
        time(due, ledger)?;
    }
    policy_graph(request, item, policy, ledger, control)
}

/// Strict complete fixed-family data decode, preserving required nulls and ordinary postfences.
pub(crate) fn decode(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<AuthoringPlanInitV1, ContractError> {
    phase(ledger, control, |ledger, control| {
        let value = super::decode_v2::structural::admitted_value(
            raw,
            MAX_POLICY_BYTES,
            include_str!("../../schemas/forge.review-authoring-plan-init-1.schema.json"),
            ledger,
            control,
        )?;
        ledger.bytes(raw.len())?;
        let request = phase(ledger, control, |_, _| {
            serde_json::from_value(value).map_err(|_| ContractError::Invalid)
        })?;
        asserted(&request, ledger, control)?;
        Ok(request)
    })
}

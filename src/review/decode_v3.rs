//! Strict separately closed plain Queue/3 intake and complete asserted correlations.
//! Decoding supplies no native facts, original file registration or currentness owner.
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase};
use super::wire_v3::{ADAPTER, IDENTITY_DISCLAIMER, QueueDocumentV3, ReviewItemV3, ReviewPolicy};
use crate::workspace::preparation::WorkControl;

/// Exact borrowed input and fully admitted inert declarations; no external constructor.
pub(crate) struct DecodedV3<'a, T> {
    /// Full input bytes, including every original formatting byte.
    raw: &'a [u8],
    /// Full original SHA-256, never a typed reserialization hash.
    raw_sha256: String,
    /// Plain separately closed data, never native source truth.
    document: T,
}
impl<'a, T> DecodedV3<'a, T> {
    /// Borrow every complete inert typed declaration.
    pub(crate) fn document(&self) -> &T {
        &self.document
    }
    /// Borrow the complete actual byte slice passed to the decoder.
    pub(crate) fn raw(&self) -> &'a [u8] {
        self.raw
    }
    /// Borrow the complete original byte digest without cloning it.
    pub(crate) fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
}
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
/// Exact lowercase full digest syntax without coercion or native truth.
fn hash(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    compared(&[value], ledger)?;
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Preserve canonical nonnil review UUID text and its original spelling.
fn uuid(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    let bytes = value.len().checked_mul(3).ok_or_else(|| ledger.capacity())?;
    ledger.bytes(bytes)?;
    ledger.derived(128)?;
    super::validate::uuid(value)
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
    queue: &QueueDocumentV3,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, row) in queue.roles.iter().enumerate() {
        compared(&[&row.key, key], ledger)?;
        if row.key == key {
            found = Some(index);
        }
    }
    found.ok_or(ContractError::Invalid)
}
/// Scan the complete actual reviewer registry; it grants no reviewer authentication.
fn reviewer_index(
    queue: &QueueDocumentV3,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, row) in queue.reviewers.iter().enumerate() {
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
    queue: &QueueDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    if !(1..=32).contains(&queue.roles.len()) || !(1..=100).contains(&queue.reviewers.len()) {
        return Err(ContractError::Invalid);
    }
    ordered(queue.roles.iter().map(|r| r.key.as_str()), ledger)?;
    ordered(queue.reviewers.iter().map(|r| r.key.as_str()), ledger)?;
    let mut memberships = 0_usize;
    for row in &queue.reviewers {
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
            role_index(queue, role, ledger)?;
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
    queue: &QueueDocumentV3,
    item: &ReviewItemV3,
    reviewer: &str,
    role: &str,
    ledger: &mut ContractLedger,
) -> Result<usize, ContractError> {
    let index = reviewer_index(queue, reviewer, ledger)?;
    let row = &queue.reviewers[index];
    if !contains(&row.role_keys, role, ledger)? || contains(&item.author_keys, reviewer, ledger)? {
        return Err(ContractError::Invalid);
    }
    Ok(index)
}
/// Complete role/reviewer incidence and finite graph cardinalities, without matching.
fn policy_graph(
    queue: &QueueDocumentV3,
    item: &ReviewItemV3,
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
        role_index(queue, &seat.role_key, ledger)?;
    }
    for row in &item.assignments {
        checkpoint(ledger, control)?;
        key(&row.reviewer_key, ledger)?;
        key(&row.role_key, ledger)?;
        let reviewer = eligible(queue, item, &row.reviewer_key, &row.role_key, ledger)?;
        if let Some(seat) = seat_index(policy, &row.role_key, ledger)? {
            candidates[seat][reviewer] = true;
        }
    }
    for row in &policy.substitutions {
        checkpoint(ledger, control)?;
        let reviewer = eligible(queue, item, &row.reviewer_key, &row.asserted_role, ledger)?;
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
/// Full single-item identity/source/policy/deadline checks and framed correlations.
fn item(
    queue: &QueueDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let item = &queue.items[0];
    let selected = &queue.policies[0];
    key(&item.key, ledger)?;
    key(&item.policy_key, ledger)?;
    uuid(&item.item_id, ledger)?;
    compared(&[&item.adapter_version, &item.subject_id, &selected.key, &item.policy_key], ledger)?;
    let subject = item.subject_id.strip_prefix("authoring-plan:").ok_or(ContractError::Invalid)?;
    if item.adapter_version != ADAPTER || selected.key != item.policy_key {
        return Err(ContractError::Invalid);
    }
    hash(subject, ledger)?;
    for value in [&item.subject_sha256, &item.context_sha256, &item.policy_sha256] {
        hash(value, ledger)?;
    }
    if item.source_keys.len() != queue.source_pins.len() {
        return Err(ContractError::Invalid);
    }
    ordered(item.source_keys.iter().map(String::as_str), ledger)?;
    for (key, pin) in item.source_keys.iter().zip(&queue.source_pins) {
        compared(&[key, &pin.artifact_key], ledger)?;
        if key != &pin.artifact_key {
            return Err(ContractError::Invalid);
        }
    }
    if let Some(due) = &item.due_at {
        time(due, ledger)?;
        compared(&[due, &queue.created_at], ledger)?;
        if due <= &queue.created_at {
            return Err(ContractError::Invalid);
        }
    }
    policy_graph(queue, item, selected, ledger, control)?;
    let context = super::hash_v3::context(&queue.source_pins, &item.context, ledger, control)?;
    let policy = super::hash_v3::policy(&queue.source_pins, selected, item, ledger, control)?;
    let identity =
        super::hash_v3::item_id(&queue.queue_id, item, &queue.source_pins, ledger, control)?;
    compared(
        &[&context, &item.context_sha256, &policy, &item.policy_sha256, &identity, &item.item_id],
        ledger,
    )?;
    if context != item.context_sha256 || policy != item.policy_sha256 || identity != item.item_id {
        return Err(ContractError::Binding);
    }
    Ok(())
}
/// Validate all Queue/3 typed declarations; this never proves native source truth.
pub(crate) fn queue(
    queue: &QueueDocumentV3,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        super::encode_v3::queue_operands(queue, ledger, control)?;
        compared(&[&queue.schema_version, &queue.identity_disclaimer], ledger)?;
        if queue.schema_version != "forge.review-queue/3"
            || queue.identity_disclaimer != IDENTITY_DISCLAIMER
            || queue.items.len() != 1
            || queue.policies.len() != 1
        {
            return Err(ContractError::Invalid);
        }
        uuid(&queue.queue_id, ledger)?;
        time(&queue.created_at, ledger)?;
        roster(queue, ledger, control)?;
        item(queue, ledger, control)
    })
}
/// Duplicate-safe full raw Queue/3 decode under one original ledger/controller.
/// Required explicit null presence is checked by the fixed closed schema before Serde.
pub(crate) fn decode_queue<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV3<'a, QueueDocumentV3>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let value = super::decode_v2::structural::admitted_value(
            raw,
            super::encode_v3::MAX_QUEUE_BYTES,
            include_str!("../../schemas/forge.review-queue-3.schema.json"),
            ledger,
            control,
        )?;
        ledger.bytes(raw.len())?;
        let document: QueueDocumentV3 = phase(ledger, control, |_, _| {
            serde_json::from_value(value).map_err(|_| ContractError::Invalid)
        })?;
        queue(&document, ledger, control)?;
        ledger.bytes(raw.len())?;
        ledger.derived(128)?;
        let raw_sha256 = phase(ledger, control, |_, _| Ok(crate::hashing::sha256_hex(raw)))?;
        Ok(DecodedV3 { raw, raw_sha256, document })
    })
}

#[cfg(test)]
#[path = "queue_v3_codec_tests.rs"]
/// Plain duplicate/null/registry/framing/output/first-stop controls, without native authority.
mod tests;

/// Separately closed plain Response/3 and recorded Dispositions/3 intake.
#[path = "decode_v3_operational.rs"]
mod operational;
pub(crate) use operational::{decode_dispositions, decode_response, dispositions, response};

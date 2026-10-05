//! Structural correlations and narrow versioned typed encoding. These routines
//! never evaluate native approval, currentness, response-chain withdrawal or
//! quorum. A supplied native subject fingerprint remains an adapter assertion.

use super::decode::{ContractError, ContractLedger, Decoded};
use super::wire::{
    Assignment, ContextSnapshot, Disposition, DispositionsDocument, Domain, IDENTITY_DISCLAIMER,
    ItemDisposition, ItemState, MetSeat, QueueDocument, RecordedCurrentness, RecordedResponse,
    RequestedAction, ResponseClassification, ResponseDocument, ReviewItem, ReviewPolicy, Reviewer,
    SourceModel, SourcePin,
};
use crate::workspace::preparation::WorkControl;
use chrono::NaiveDateTime;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::{self, Write};
use uuid::Uuid;

/// Canonical nonnil lowercase UUID; preserve the caller's original spelling.
pub(crate) fn uuid(value: &str) -> Result<(), ContractError> {
    let parsed = Uuid::parse_str(value).map_err(|_| ContractError::Invalid)?;
    if parsed.is_nil() || parsed.to_string() != value {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// First-profile UTC calendar seconds, years 0001..9999, without leap seconds.
/// No wall clock, offset normalization or source date reinterpretation occurs.
pub(crate) fn time(value: &str) -> Result<(), ContractError> {
    if value.len() != 20 || !value.is_ascii() || value.starts_with("0000") {
        return Err(ContractError::Invalid);
    }
    if value[17..19].parse::<u8>().map_err(|_| ContractError::Invalid)? > 59 {
        return Err(ContractError::Invalid);
    }
    let parsed = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ")
        .map_err(|_| ContractError::Invalid)?;
    if parsed.format("%Y-%m-%dT%H:%M:%SZ").to_string() != value {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Exact bounded ASCII identity token; no key normalization is performed.
fn token(value: &str, maximum: usize) -> Result<(), ContractError> {
    if value.is_empty()
        || value.len() > maximum
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Bounded version/schema identity token, separate from path-free reviewer keys.
fn version(value: &str) -> Result<(), ContractError> {
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Canonical raw/typed pin syntax; syntax is not original-byte qualification.
fn hash(value: &str) -> Result<(), ContractError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Inspect an ordered unique token list before repeated membership comparisons.
fn tokens(
    rows: &[String],
    maximum: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(rows.len())?;
    for row in rows {
        ledger.bytes(row.len().checked_mul(2).ok_or(ContractError::Capacity)?)?;
        token(row, maximum)?;
    }
    if rows.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Precharge a complete bounded key scan; strings are not free work.
fn scan(ledger: &mut ContractLedger, rows: usize, width: usize) -> Result<(), ContractError> {
    ledger.visits(rows)?;
    ledger.bytes(rows.checked_mul(width).ok_or(ContractError::Capacity)?)
}

/// Match exactly one asserted reviewer after complete scan precharge.
fn reviewer<'a>(
    queue: &'a QueueDocument,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<&'a Reviewer, ContractError> {
    scan(ledger, queue.reviewers.len(), 256)?;
    queue.reviewers.iter().find(|row| row.key == key).ok_or(ContractError::Invalid)
}

/// Resolve a declared policy without detached fallback or manufactured defaults.
fn selected_policy<'a>(
    queue: &'a QueueDocument,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<&'a ReviewPolicy, ContractError> {
    scan(ledger, queue.policies.len(), 256)?;
    queue.policies.iter().find(|row| row.key == key).ok_or(ContractError::Invalid)
}

/// Validate complete declared original pins, not their physical capture/currentness.
fn pins(rows: &[SourcePin], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    scan(ledger, rows.len(), 512)?;
    if rows.is_empty()
        || rows.len() > 100
        || rows.windows(2).any(|pair| pair[0].artifact_key >= pair[1].artifact_key)
    {
        return Err(ContractError::Invalid);
    }
    let mut bytes = 0_u64;
    for row in rows {
        ledger.bytes(512)?;
        token(&row.artifact_key, 128)?;
        version(&row.schema_identity)?;
        hash(&row.raw_sha256)?;
        if row.byte_length == 0 || row.byte_length > 10_485_760 {
            return Err(ContractError::Invalid);
        }
        bytes = bytes.checked_add(row.byte_length).ok_or(ContractError::Capacity)?;
        let native = matches!(
            row.model,
            SourceModel::Mapping
                | SourceModel::Catalog
                | SourceModel::Profile
                | SourceModel::ResolvedCatalog
                | SourceModel::ComponentDefinition
        );
        match (&row.native_root_uuid, native) {
            (Some(value), true) => uuid(value)?,
            (None, false) => {}
            _ => return Err(ContractError::Invalid),
        }
    }
    if bytes > 52_428_800 {
        return Err(ContractError::Capacity);
    }
    Ok(())
}

/// Complete fixed disclaimer; never reinterpret a supplied authenticated claim.
fn disclaimer(value: &str) -> Result<(), ContractError> {
    if value != IDENTITY_DISCLAIMER {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Verify every declared role/reviewer membership and complete role pair count.
fn roster(queue: &QueueDocument, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    if queue.roles.is_empty()
        || queue.roles.len() > 32
        || queue.reviewers.is_empty()
        || queue.reviewers.len() > 100
    {
        return Err(ContractError::Invalid);
    }
    scan(
        ledger,
        queue.roles.len().checked_add(queue.reviewers.len()).ok_or(ContractError::Capacity)?,
        256,
    )?;
    if queue.roles.windows(2).any(|pair| pair[0].key >= pair[1].key)
        || queue.reviewers.windows(2).any(|pair| pair[0].key >= pair[1].key)
    {
        return Err(ContractError::Invalid);
    }
    for role in &queue.roles {
        token(&role.key, 128)?;
    }
    let mut memberships = 0_usize;
    for row in &queue.reviewers {
        token(&row.key, 128)?;
        tokens(&row.role_keys, 128, ledger)?;
        if row.role_keys.is_empty() || row.role_keys.len() > 32 {
            return Err(ContractError::Invalid);
        }
        memberships =
            memberships.checked_add(row.role_keys.len()).ok_or(ContractError::Capacity)?;
        if memberships > 3_200 {
            return Err(ContractError::Capacity);
        }
        for role in &row.role_keys {
            scan(ledger, queue.roles.len(), 256)?;
            if !queue.roles.iter().any(|declared| declared.key == *role) {
                return Err(ContractError::Invalid);
            }
        }
    }
    Ok(())
}

/// Inspect positive finite seat requirements before any slot expansion.
fn policy(
    queue: &QueueDocument,
    policy: &ReviewPolicy,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    token(&policy.key, 128)?;
    scan(ledger, policy.seats.len(), 256)?;
    if policy.seats.is_empty()
        || policy.seats.len() > 32
        || policy.seats.windows(2).any(|pair| pair[0].role_key >= pair[1].role_key)
    {
        return Err(ContractError::Invalid);
    }
    let mut seats = 0_usize;
    for seat in &policy.seats {
        token(&seat.role_key, 128)?;
        scan(ledger, queue.roles.len(), 256)?;
        if seat.count == 0
            || seat.count > 100
            || !queue.roles.iter().any(|role| role.key == seat.role_key)
        {
            return Err(ContractError::Invalid);
        }
        seats = seats
            .checked_add(usize::try_from(seat.count).map_err(|_| ContractError::Capacity)?)
            .ok_or(ContractError::Capacity)?;
        if seats > 100 {
            return Err(ContractError::Capacity);
        }
    }
    tokens(&policy.empty_abstention_reasons, 128, ledger)?;
    if policy.empty_abstention_reasons.len() > 32 || policy.substitutions.len() > 3_200 {
        return Err(ContractError::Capacity);
    }
    for pair in policy.substitutions.windows(2) {
        ledger.bytes(768)?;
        if (&pair[0].seat_role, &pair[0].reviewer_key, &pair[0].asserted_role)
            >= (&pair[1].seat_role, &pair[1].reviewer_key, &pair[1].asserted_role)
        {
            return Err(ContractError::Invalid);
        }
    }
    for edge in &policy.substitutions {
        token(&edge.seat_role, 128)?;
        token(&edge.reason_code, 128)?;
        let candidate = reviewer(queue, &edge.reviewer_key, ledger)?;
        scan(ledger, candidate.role_keys.len(), 256)?;
        scan(ledger, policy.seats.len(), 256)?;
        if !candidate.role_keys.contains(&edge.asserted_role)
            || !policy.seats.iter().any(|seat| seat.role_key == edge.seat_role)
        {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}

/// Hold no retaining serialization buffer while hashing narrow typed encodings.
struct HashWriter<'a> {
    /// Actual incremental typed-byte digest.
    digest: Sha256,
    /// Caller-owned shared repeated-byte work ledger.
    ledger: &'a mut ContractLedger,
    /// Exact encoded extent for the context byte ceiling.
    length: usize,
    /// Finite complete encoded ceiling, checked before every digest update.
    cap: usize,
}

impl Write for HashWriter<'_> {
    /// Precharge exact emitted bytes before digesting; never perform filesystem IO.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .length
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("review encoding stopped"))?;
        if next > self.cap {
            self.ledger.capacity();
            return Err(io::Error::other("review encoding stopped"));
        }
        self.ledger.bytes(bytes.len()).map_err(|_| io::Error::other("review encoding stopped"))?;
        self.digest.update(bytes);
        self.length = next;
        Ok(bytes.len())
    }
    /// The incremental digest has no external buffers to flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Prefix plus compact serde JSON, no LF: a narrow typed encoding, not JCS.
fn digest<T: Serialize>(
    prefix: &[u8],
    value: &T,
    cap: usize,
    ledger: &mut ContractLedger,
) -> Result<[u8; 32], ContractError> {
    ledger.bytes(prefix.len())?;
    let mut writer = HashWriter { digest: Sha256::new(), ledger, length: 0, cap };
    writer.digest.update(prefix);
    if serde_json::to_writer(&mut writer, value).is_err() {
        return Err(writer.ledger.failure());
    }
    Ok(writer.digest.finalize().into())
}

/// Every native adapter/schema and context fact in the narrow context encoding.
#[derive(Serialize)]
struct ContextEncoding<'a> {
    /// Actual selected domain declaration.
    domain: Domain,
    /// Exact native adapter version.
    adapter_version: &'a str,
    /// Complete declared source/schema identities, still not captured proof.
    source_pins: &'a [SourcePin],
    /// Complete displayed minimized context.
    context: &'a ContextSnapshot,
}

/// Display context is at most two KiB; its typed fingerprint also binds the
/// selected domain, adapter and complete declared source/schema identities.
pub(crate) fn context_hash(
    queue: &QueueDocument,
    item: &ReviewItem,
    ledger: &mut ContractLedger,
) -> Result<String, ContractError> {
    ledger.bound(|ledger| {
        let context = &item.context;
        tokens(&context.reason_codes, 128, ledger)?;
        tokens(&context.related_subject_ids, 256, ledger)?;
        if context.reason_codes.len() > 32 || context.related_subject_ids.len() > 32 {
            return Err(ContractError::Invalid);
        }
        digest(b"", context, 2_048, ledger)?;
        let encoding = ContextEncoding {
            domain: item.domain,
            adapter_version: &item.adapter_version,
            source_pins: &queue.source_pins,
            context,
        };
        let digest = digest(b"forge.review-context/1\0", &encoding, 1_048_576, ledger)?;
        ledger.derived(64)?;
        Ok(crate::hashing::lower_hex(&digest))
    })
}

/// All decision-bearing policy binding fields, in this exact serialization order.
#[derive(Serialize)]
struct PolicyEncoding<'a> {
    /// Exact selected native domain.
    domain: Domain,
    /// Exact selected native adapter version.
    adapter_version: &'a str,
    /// Complete declared native/schema closure identities.
    source_pins: &'a [SourcePin],
    /// Full selected policy, including substitutions and abstention exceptions.
    policy: &'a ReviewPolicy,
    /// Exact declared source authors.
    author_keys: &'a [String],
    /// Full ordered explicit reviewer-role assignments.
    assignments: &'a [Assignment],
    /// Explicit null or exact UTC deadline.
    due_at: &'a Option<String>,
    /// Complete dissent-preserving disposition vocabulary.
    allowed_dispositions: &'a [Disposition],
}

/// Compute a full typed policy/assignment/deadline binding without native authority.
pub(crate) fn policy_hash(
    queue: &QueueDocument,
    policy: &ReviewPolicy,
    item: &ReviewItem,
    ledger: &mut ContractLedger,
) -> Result<String, ContractError> {
    ledger.bound(|ledger| {
        let encoding = PolicyEncoding {
            domain: item.domain,
            adapter_version: &item.adapter_version,
            source_pins: &queue.source_pins,
            policy,
            author_keys: &item.author_keys,
            assignments: &item.assignments,
            due_at: &item.due_at,
            allowed_dispositions: &item.allowed_dispositions,
        };
        let digest = digest(b"forge.review-policy-binding/1\0", &encoding, 1_048_576, ledger)?;
        ledger.derived(64)?;
        Ok(crate::hashing::lower_hex(&digest))
    })
}

/// Exact item identity payload, excluding only the output item UUID itself.
#[derive(Serialize)]
struct ItemEncoding<'a> {
    /// Explicit queue revision UUID.
    queue_id: &'a str,
    /// Stable queue-local selected item key.
    key: &'a str,
    /// Exact selected adapter domain.
    domain: Domain,
    /// Exact adapter version.
    adapter_version: &'a str,
    /// Original native subject identity.
    subject_id: &'a str,
    /// Explicit review intent.
    requested_action: RequestedAction,
    /// Complete selected source subset.
    source_keys: &'a [String],
    /// Full queue closure pins, without paths or source bytes.
    source_pins: &'a [SourcePin],
    /// Full native subject digest supplied by the future adapter.
    subject_sha256: &'a str,
    /// Full minimized-context digest.
    context_sha256: &'a str,
    /// Full policy binding digest.
    policy_sha256: &'a str,
}

/// Frozen review-only UUID namespace; SHA-256 typed payload feeds `UUIDv5`.
/// The UUID is not authenticated and no native UUID namespace is reused.
pub(crate) fn item_id(
    queue: &QueueDocument,
    item: &ReviewItem,
    ledger: &mut ContractLedger,
) -> Result<String, ContractError> {
    ledger.bound(|ledger| {
        let encoding = ItemEncoding {
            queue_id: &queue.queue_id,
            key: &item.key,
            domain: item.domain,
            adapter_version: &item.adapter_version,
            subject_id: &item.subject_id,
            requested_action: item.requested_action,
            source_keys: &item.source_keys,
            source_pins: &queue.source_pins,
            subject_sha256: &item.subject_sha256,
            context_sha256: &item.context_sha256,
            policy_sha256: &item.policy_sha256,
        };
        let digest = digest(b"forge.review-item/1\0", &encoding, 1_048_576, ledger)?;
        let namespace = Uuid::from_bytes([
            0x9f, 0x7c, 0x24, 0xf8, 0x56, 0x1d, 0x55, 0xa1, 0xa4, 0x3d, 0xfc, 0x4d, 0x81, 0x59,
            0xb4, 0x60,
        ]);
        ledger.bytes(48)?;
        ledger.derived(36)?;
        Ok(Uuid::new_v5(&namespace, &digest).to_string())
    })
}

/// Inspect assignments, author exclusion and complete finite candidate graph size.
fn assignments(
    queue: &QueueDocument,
    item: &ReviewItem,
    policy: &ReviewPolicy,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    tokens(&item.author_keys, 128, ledger)?;
    if item.author_keys.is_empty() || item.author_keys.len() > 100 || item.assignments.len() > 3_200
    {
        return Err(ContractError::Invalid);
    }
    for pair in item.assignments.windows(2) {
        ledger.bytes(512)?;
        if (&pair[0].reviewer_key, &pair[0].role_key) >= (&pair[1].reviewer_key, &pair[1].role_key)
        {
            return Err(ContractError::Invalid);
        }
    }
    for assigned in &item.assignments {
        let declared = reviewer(queue, &assigned.reviewer_key, ledger)?;
        scan(ledger, declared.role_keys.len(), 256)?;
        scan(ledger, item.author_keys.len(), 256)?;
        if !declared.role_keys.contains(&assigned.role_key)
            || item.author_keys.contains(&assigned.reviewer_key)
        {
            return Err(ContractError::Invalid);
        }
    }
    for edge in &policy.substitutions {
        scan(ledger, item.author_keys.len(), 256)?;
        if item.author_keys.contains(&edge.reviewer_key) {
            return Err(ContractError::Invalid);
        }
    }
    let mut keys = BTreeSet::new();
    let mut seats = 0_usize;
    let mut edges = 0_usize;
    for required in &policy.seats {
        let mut candidates = BTreeSet::new();
        scan(ledger, item.assignments.len(), 512)?;
        scan(ledger, policy.substitutions.len(), 768)?;
        for assigned in item.assignments.iter().filter(|row| row.role_key == required.role_key) {
            ledger.derived(128)?;
            candidates.insert(assigned.reviewer_key.as_str());
            keys.insert(assigned.reviewer_key.as_str());
        }
        for edge in policy.substitutions.iter().filter(|row| row.seat_role == required.role_key) {
            ledger.derived(128)?;
            candidates.insert(edge.reviewer_key.as_str());
            keys.insert(edge.reviewer_key.as_str());
        }
        let count = usize::try_from(required.count).map_err(|_| ContractError::Capacity)?;
        seats = seats.checked_add(count).ok_or(ContractError::Capacity)?;
        edges = edges
            .checked_add(count.checked_mul(candidates.len()).ok_or(ContractError::Capacity)?)
            .ok_or(ContractError::Capacity)?;
        if seats > 100 || edges > 10_000 {
            return Err(ContractError::Capacity);
        }
    }
    ledger.graph(seats, edges, seats.checked_add(keys.len()).ok_or(ContractError::Capacity)?)
}

/// Exact selected first-adapter revision; no other domain implicitly opts in.
fn adapter(domain: Domain, version: &str) -> Result<(), ContractError> {
    let expected = match domain {
        Domain::MappingAssertion => "forge.mapping-review/1",
        Domain::ApplicabilityDecision => "forge.applicability-review/1",
    };
    if version != expected {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Validate an entire declared item without native source comparison or matching.
fn item(
    queue: &QueueDocument,
    item: &ReviewItem,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    token(&item.key, 128)?;
    uuid(&item.item_id)?;
    token(&item.subject_id, 256)?;
    adapter(item.domain, &item.adapter_version)?;
    hash(&item.subject_sha256)?;
    tokens(&item.source_keys, 128, ledger)?;
    if item.source_keys.is_empty() || item.source_keys.len() > 100 {
        return Err(ContractError::Invalid);
    }
    for key in &item.source_keys {
        scan(ledger, queue.source_pins.len(), 256)?;
        if !queue.source_pins.iter().any(|pin| pin.artifact_key == *key) {
            return Err(ContractError::Invalid);
        }
    }
    let policy = selected_policy(queue, &item.policy_key, ledger)?;
    assignments(queue, item, policy, ledger)?;
    if let Some(due) = &item.due_at {
        time(due)?;
        if due <= &queue.created_at {
            return Err(ContractError::Invalid);
        }
    }
    let allowed = [
        Disposition::Approve,
        Disposition::Reject,
        Disposition::RequestChanges,
        Disposition::Abstain,
        Disposition::Superseded,
    ];
    if item.allowed_dispositions != allowed {
        return Err(ContractError::Invalid);
    }
    hash(&item.context_sha256)?;
    hash(&item.policy_sha256)?;
    if context_hash(queue, item, ledger)? != item.context_sha256
        || policy_hash(queue, policy, item, ledger)? != item.policy_sha256
        || item_id(queue, item, ledger)? != item.item_id
    {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Complete closed queue correlations; unassigned and unmatched are legitimate.
pub(crate) fn queue(
    queue: &QueueDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        disclaimer(&queue.identity_disclaimer)?;
        if queue.schema_version != "forge.review-queue/1" {
            return Err(ContractError::Invalid);
        }
        uuid(&queue.queue_id)?;
        time(&queue.created_at)?;
        pins(&queue.source_pins, ledger)?;
        roster(queue, ledger)?;
        if queue.policies.is_empty()
            || queue.policies.len() > 10_000
            || queue.items.is_empty()
            || queue.items.len() > 10_000
        {
            return Err(ContractError::Invalid);
        }
        scan(
            ledger,
            queue.policies.len().checked_add(queue.items.len()).ok_or(ContractError::Capacity)?,
            256,
        )?;
        if queue.policies.windows(2).any(|pair| pair[0].key >= pair[1].key)
            || queue.items.windows(2).any(|pair| pair[0].key >= pair[1].key)
        {
            return Err(ContractError::Invalid);
        }
        for declared in &queue.policies {
            ledger.checkpoint(control)?;
            policy(queue, declared, ledger)?;
        }
        let mut ids = BTreeSet::new();
        for selected in &queue.items {
            ledger.checkpoint(control)?;
            ledger.derived(64)?;
            if !ids.insert(selected.item_id.as_str()) {
                return Err(ContractError::Invalid);
            }
            item(queue, selected, ledger)?;
        }
        ledger.checkpoint(control)
    })
}

/// Complete response shape correlations; eligibility and prior-chain checks are separate.
pub(crate) fn response(
    response: &ResponseDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        disclaimer(&response.identity_disclaimer)?;
        if response.schema_version != "forge.review-response/1" {
            return Err(ContractError::Invalid);
        }
        uuid(&response.response_id)?;
        uuid(&response.queue_id)?;
        uuid(&response.item_id)?;
        time(&response.responded_at)?;
        pins(&response.source_pins, ledger)?;
        adapter(response.domain, &response.adapter_version)?;
        token(&response.item_key, 128)?;
        token(&response.reviewer_key, 128)?;
        token(&response.reviewer_role, 128)?;
        for value in [
            &response.queue_raw_sha256,
            &response.subject_sha256,
            &response.context_sha256,
            &response.policy_sha256,
        ] {
            hash(value)?;
        }
        if response.rationale.len() > 8_192 {
            return Err(ledger.capacity());
        }
        ledger.bytes(response.rationale.len())?;
        match (response.disposition, &response.abstention_reason) {
            (Disposition::Abstain, Some(reason)) => token(reason, 128)?,
            (Disposition::Abstain, None) | (_, Some(_)) => return Err(ContractError::Invalid),
            (_, None) => {
                if response.rationale.trim().is_empty() {
                    return Err(ContractError::Invalid);
                }
            }
        }
        match (response.disposition, &response.supersedes) {
            (Disposition::Superseded, Some(prior)) => {
                uuid(&prior.response_id)?;
                hash(&prior.raw_sha256)?;
                if prior.response_id == response.response_id {
                    return Err(ContractError::Invalid);
                }
            }
            (Disposition::Superseded, None) | (_, Some(_)) => return Err(ContractError::Invalid),
            (_, None) => {}
        }
        ledger.checkpoint(control)
    })
}

/// Precharge all compared string extents on both sides, without normalized copies.
fn compared_strings(values: &[&str], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(values.len())?;
    let bytes = values.iter().try_fold(0_usize, |total, value| {
        total.checked_add(value.len()).ok_or(ContractError::Capacity)
    })?;
    ledger.bytes(bytes)
}

/// Complete string extent plus conservative fixed enum/integer/option allowance.
fn pin_extent(pins: &[SourcePin], ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    ledger.visits(pins.len())?;
    pins.iter().try_fold(0_usize, |total, pin| {
        let native = pin.native_root_uuid.as_ref().map_or(0, String::len);
        [pin.artifact_key.len(), pin.schema_identity.len(), pin.raw_sha256.len(), native, 64]
            .into_iter()
            .try_fold(total, |bytes, extent| {
                bytes.checked_add(extent).ok_or(ContractError::Capacity)
            })
    })
}

/// Precharge the complete worst-case typed pin equality, BOTH compared extents.
fn compared_pins(
    left: &[SourcePin],
    right: &[SourcePin],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let left = pin_extent(left, ledger)?;
    let right = pin_extent(right, ledger)?;
    ledger.bytes(left.checked_add(right).ok_or(ContractError::Capacity)?)
}

/// Bind declared response fields to the exact raw queue. No approval, deadline
/// classification, chain retirement, source-currentness or quorum is inferred.
pub(crate) fn bind_response(
    queue: &Decoded<'_, QueueDocument>,
    response: &Decoded<'_, ResponseDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let queue_doc = queue.document();
        let response = response.document();
        scan(ledger, queue_doc.items.len(), 512)?;
        let item = queue_doc
            .items
            .iter()
            .find(|item| item.key == response.item_key)
            .ok_or(ContractError::Binding)?;
        compared_pins(&response.source_pins, &queue_doc.source_pins, ledger)?;
        compared_strings(
            &[
                &response.queue_id,
                &queue_doc.queue_id,
                &response.queue_raw_sha256,
                queue.raw_sha256(),
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
        ledger.bytes(16)?;
        if response.queue_id != queue_doc.queue_id
            || response.queue_raw_sha256 != queue.raw_sha256()
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
        let declared = reviewer(queue_doc, &response.reviewer_key, ledger)?;
        scan(ledger, declared.role_keys.len(), 256)?;
        scan(ledger, item.author_keys.len(), 256)?;
        if !declared.role_keys.contains(&response.reviewer_role)
            || item.author_keys.contains(&response.reviewer_key)
        {
            return Err(ContractError::Binding);
        }
        let policy = selected_policy(queue_doc, &item.policy_key, ledger)?;
        scan(ledger, item.assignments.len(), 512)?;
        scan(ledger, policy.substitutions.len(), 768)?;
        let assigned = item.assignments.iter().any(|row| {
            row.reviewer_key == response.reviewer_key && row.role_key == response.reviewer_role
        });
        let substitute = policy.substitutions.iter().any(|row| {
            row.reviewer_key == response.reviewer_key && row.asserted_role == response.reviewer_role
        });
        if !assigned && !substitute {
            return Err(ContractError::Binding);
        }
        ledger.bytes(response.rationale.len())?;
        if response.disposition == Disposition::Abstain && response.rationale.trim().is_empty() {
            scan(ledger, policy.empty_abstention_reasons.len(), 256)?;
            if !response
                .abstention_reason
                .as_ref()
                .is_some_and(|reason| policy.empty_abstention_reasons.contains(reason))
            {
                return Err(ContractError::Binding);
            }
        }
        ledger.checkpoint(control)
    })
}

/// Preserve every recorded original and its dissent, without rationale disclosure.
fn recorded_responses(
    rows: &[RecordedResponse],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    scan(ledger, rows.len(), 72)?;
    if rows.len() > 10_000 || rows.windows(2).any(|pair| pair[0].response_id >= pair[1].response_id)
    {
        return Err(ContractError::Invalid);
    }
    let mut bytes = 0_u64;
    for row in rows {
        ledger.visits(1)?;
        ledger.bytes(512)?;
        uuid(&row.response_id)?;
        hash(&row.raw_sha256)?;
        token(&row.item_key, 128)?;
        token(&row.reviewer_key, 128)?;
        token(&row.reviewer_role, 128)?;
        time(&row.responded_at)?;
        if row.byte_length == 0 || row.byte_length > 1_048_576 {
            return Err(ContractError::Invalid);
        }
        bytes = bytes.checked_add(row.byte_length).ok_or(ContractError::Capacity)?;
    }
    if bytes > 33_554_432 {
        return Err(ContractError::Capacity);
    }
    Ok(())
}

/// Validate complete recorded seat witnesses without running or trusting matching.
fn recorded_seats(
    item: &ItemDisposition,
    rows: &[RecordedResponse],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    let total =
        item.met_seats.len().checked_add(item.unmet_seats.len()).ok_or(ContractError::Capacity)?;
    if item.required_seats == 0
        || item.required_seats > 100
        || total != usize::try_from(item.required_seats).map_err(|_| ContractError::Capacity)?
    {
        return Err(ContractError::Invalid);
    }
    let mut reviewers = BTreeSet::new();
    let mut seats = BTreeSet::new();
    for witness in &item.met_seats {
        token(&witness.role_key, 128)?;
        token(&witness.reviewer_key, 128)?;
        uuid(&witness.response_id)?;
        if witness.ordinal >= 100 {
            return Err(ContractError::Invalid);
        }
        ledger.derived(128)?;
        if !reviewers.insert(witness.reviewer_key.as_str())
            || !seats.insert((witness.role_key.as_str(), witness.ordinal))
        {
            return Err(ContractError::Invalid);
        }
        scan(ledger, rows.len(), 512)?;
        let row = rows
            .iter()
            .find(|row| row.response_id == witness.response_id)
            .ok_or(ContractError::Invalid)?;
        if row.item_key != item.item_key
            || row.reviewer_key != witness.reviewer_key
            || row.disposition != Disposition::Approve
            || row.classification != ResponseClassification::Current
        {
            return Err(ContractError::Invalid);
        }
        scan(ledger, item.response_ids.len(), 72)?;
        if !item.response_ids.contains(&row.response_id) {
            return Err(ContractError::Invalid);
        }
    }
    for witness in &item.unmet_seats {
        token(&witness.role_key, 128)?;
        if witness.ordinal >= 100 {
            return Err(ContractError::Invalid);
        }
        ledger.derived(64)?;
        if !seats.insert((witness.role_key.as_str(), witness.ordinal)) {
            return Err(ContractError::Invalid);
        }
    }
    if item.state == ItemState::QuorumMet && (item.blocking || !item.unmet_seats.is_empty()) {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Require every matching response and every historical dissent to stay visible.
fn recorded_item(
    item: &ItemDisposition,
    rows: &[RecordedResponse],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    token(&item.item_key, 128)?;
    uuid(&item.item_id)?;
    tokens(&item.reason_codes, 128, ledger)?;
    tokens(&item.response_ids, 36, ledger)?;
    tokens(&item.dissent_ids, 36, ledger)?;
    for id in &item.response_ids {
        uuid(id)?;
        scan(ledger, rows.len(), 512)?;
        if !rows.iter().any(|row| row.response_id == *id && row.item_key == item.item_key) {
            return Err(ContractError::Invalid);
        }
    }
    for row in rows {
        ledger.visits(1)?;
        ledger.bytes(384)?;
        if row.item_key != item.item_key {
            continue;
        }
        scan(ledger, item.response_ids.len(), 72)?;
        if !item.response_ids.contains(&row.response_id) {
            return Err(ContractError::Invalid);
        }
        let dissent = matches!(row.disposition, Disposition::Reject | Disposition::RequestChanges);
        scan(ledger, item.dissent_ids.len(), 72)?;
        if dissent != item.dissent_ids.contains(&row.response_id) {
            return Err(ContractError::Invalid);
        }
        if item.state == ItemState::QuorumMet
            && (row.classification == ResponseClassification::Conflicted
                || (row.classification == ResponseClassification::Current
                    && matches!(
                        row.disposition,
                        Disposition::Reject | Disposition::RequestChanges
                    )))
        {
            return Err(ContractError::Invalid);
        }
        scan(ledger, item.met_seats.len(), 72)?;
        if row.classification == ResponseClassification::Current
            && matches!(
                row.disposition,
                Disposition::Reject
                    | Disposition::RequestChanges
                    | Disposition::Abstain
                    | Disposition::Superseded
            )
            && item.met_seats.iter().any(|seat| seat.response_id == row.response_id)
        {
            return Err(ContractError::Invalid);
        }
    }
    for id in &item.dissent_ids {
        scan(ledger, rows.len(), 512)?;
        if !rows.iter().any(|row| {
            row.response_id == *id
                && row.item_key == item.item_key
                && matches!(row.disposition, Disposition::Reject | Disposition::RequestChanges)
        }) {
            return Err(ContractError::Invalid);
        }
    }
    recorded_seats(item, rows, ledger)
}

/// Match complete counters to every recorded state without short-circuit quorum.
fn counters(document: &DispositionsDocument) -> Result<(), ContractError> {
    let counts = &document.counts;
    if usize::try_from(counts.items).map_err(|_| ContractError::Capacity)? != document.items.len()
        || usize::try_from(counts.unique_responses).map_err(|_| ContractError::Capacity)?
            != document.responses.len()
        || counts.response_files > 10_000
        || counts.response_files
            != counts
                .unique_responses
                .checked_add(counts.exact_duplicates)
                .ok_or(ContractError::Capacity)?
    {
        return Err(ContractError::Invalid);
    }
    let reported = [
        counts.states.unassigned,
        counts.states.assigned,
        counts.states.in_review,
        counts.states.conflicted,
        counts.states.changes_requested,
        counts.states.quorum_met,
        counts.states.expired,
        counts.states.stale,
    ];
    let mut actual = [0_u32; 8];
    for item in &document.items {
        let index = match item.state {
            ItemState::Unassigned => 0,
            ItemState::Assigned => 1,
            ItemState::InReview => 2,
            ItemState::Conflicted => 3,
            ItemState::ChangesRequested => 4,
            ItemState::QuorumMet => 5,
            ItemState::Expired => 6,
            ItemState::Stale => 7,
        };
        actual[index] = actual[index].checked_add(1).ok_or(ContractError::Capacity)?;
    }
    if actual != reported {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Validate a complete inert recorded bundle; no claimed state becomes authority.
pub(crate) fn dispositions(
    document: &DispositionsDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        disclaimer(&document.identity_disclaimer)?;
        if document.schema_version != "forge.review-dispositions/1" {
            return Err(ContractError::Invalid);
        }
        uuid(&document.queue_id)?;
        hash(&document.queue_raw_sha256)?;
        time(&document.as_of)?;
        match (document.currentness, &document.closure_generation) {
            (RecordedCurrentness::Unverified, None) => {}
            (RecordedCurrentness::RecordedCurrent, Some(value)) => hash(value)?,
            _ => return Err(ContractError::Invalid),
        }
        pins(&document.source_pins, ledger)?;
        recorded_responses(&document.responses, ledger)?;
        scan(ledger, document.items.len(), 256)?;
        if document.items.is_empty()
            || document.items.len() > 10_000
            || document.items.windows(2).any(|pair| pair[0].item_key >= pair[1].item_key)
        {
            return Err(ContractError::Invalid);
        }
        let mut ids = BTreeSet::new();
        for item in &document.items {
            ledger.checkpoint(control)?;
            ledger.derived(64)?;
            if !ids.insert(item.item_id.as_str()) {
                return Err(ContractError::Invalid);
            }
            recorded_item(item, &document.responses, ledger)?;
        }
        ledger.visits(document.items.len())?;
        counters(document)?;
        ledger.checkpoint(control)
    })
}

/// Bind a recorded bundle to exact queue declarations, without trusting raw
/// response pins, claimed source currentness or recorded matching witnesses.
pub(crate) fn bind_dispositions(
    queue: &Decoded<'_, QueueDocument>,
    bundle: &Decoded<'_, DispositionsDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let queue_doc = queue.document();
        let recorded = bundle.document();
        compared_pins(&recorded.source_pins, &queue_doc.source_pins, ledger)?;
        compared_strings(
            &[
                &recorded.queue_id,
                &queue_doc.queue_id,
                &recorded.queue_raw_sha256,
                queue.raw_sha256(),
                &recorded.as_of,
                &queue_doc.created_at,
            ],
            ledger,
        )?;
        if recorded.queue_id != queue_doc.queue_id
            || recorded.queue_raw_sha256 != queue.raw_sha256()
            || recorded.source_pins != queue_doc.source_pins
            || recorded.items.len() != queue_doc.items.len()
            || recorded.as_of < queue_doc.created_at
        {
            return Err(ContractError::Binding);
        }
        for (item, row) in queue_doc.items.iter().zip(&recorded.items) {
            ledger.checkpoint(control)?;
            ledger.bytes(512)?;
            if row.item_key != item.key || row.item_id != item.item_id {
                return Err(ContractError::Binding);
            }
            let policy = selected_policy(queue_doc, &item.policy_key, ledger)?;
            let required = policy.seats.iter().try_fold(0_u32, |total, seat| {
                total.checked_add(seat.count).ok_or(ContractError::Capacity)
            })?;
            if row.required_seats != required {
                return Err(ContractError::Binding);
            }
            for seat in &row.unmet_seats {
                scan(ledger, policy.seats.len(), 256)?;
                if !policy.seats.iter().any(|required| {
                    required.role_key == seat.role_key && seat.ordinal < required.count
                }) {
                    return Err(ContractError::Binding);
                }
            }
            for seat in &row.met_seats {
                witness_binding(queue_doc, item, policy, seat, &recorded.responses, ledger)?;
            }
        }
        ledger.checkpoint(control)
    })
}

/// Check a recorded witness against declared eligibility without evaluating a vote.
fn witness_binding(
    queue: &QueueDocument,
    item: &ReviewItem,
    policy: &ReviewPolicy,
    seat: &MetSeat,
    responses: &[RecordedResponse],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    scan(ledger, policy.seats.len(), 256)?;
    scan(ledger, responses.len(), 512)?;
    if !policy
        .seats
        .iter()
        .any(|required| required.role_key == seat.role_key && seat.ordinal < required.count)
    {
        return Err(ContractError::Binding);
    }
    let response = responses
        .iter()
        .find(|response| response.response_id == seat.response_id)
        .ok_or(ContractError::Binding)?;
    let declared = reviewer(queue, &seat.reviewer_key, ledger)?;
    scan(ledger, declared.role_keys.len(), 256)?;
    scan(ledger, item.author_keys.len(), 256)?;
    if !declared.role_keys.contains(&response.reviewer_role)
        || item.author_keys.contains(&seat.reviewer_key)
    {
        return Err(ContractError::Binding);
    }
    scan(ledger, item.assignments.len(), 512)?;
    scan(ledger, policy.substitutions.len(), 768)?;
    let assigned = response.reviewer_role == seat.role_key
        && item.assignments.iter().any(|assignment| {
            assignment.reviewer_key == seat.reviewer_key
                && assignment.role_key == response.reviewer_role
        });
    let substitute = policy.substitutions.iter().any(|edge| {
        edge.seat_role == seat.role_key
            && edge.reviewer_key == seat.reviewer_key
            && edge.asserted_role == response.reviewer_role
    });
    if !assigned && !substitute {
        return Err(ContractError::Binding);
    }
    Ok(())
}

#[cfg(test)]
/// Direct boundary control for the consumed private pin comparator.
mod tests {
    use super::{compared_pins, pin_extent};
    use crate::review::{
        decode::{ContractError, ContractLedger},
        wire::{SourceModel, SourcePin},
    };

    /// Both maximum pin string extents must fit before any equality inspection.
    #[test]
    fn complete_two_sided_pin_extent_precharges_before_equality() {
        let pin = SourcePin {
            artifact_key: "a".repeat(128),
            model: SourceModel::Mapping,
            native_root_uuid: Some("11111111-1111-4111-8111-111111111111".into()),
            raw_sha256: "0".repeat(64),
            byte_length: 10_485_760,
            schema_identity: "s".repeat(128),
        };
        let pins = [pin];
        let mut ledger = ContractLedger::default();
        assert_eq!(pin_extent(&pins, &mut ledger).unwrap(), 420);
        ledger.bytes(268_435_456 - 840).unwrap();
        compared_pins(&pins, &pins, &mut ledger).unwrap();
        let mut ledger = ContractLedger::default();
        ledger.bytes(268_435_456 - 839).unwrap();
        assert!(matches!(compared_pins(&pins, &pins, &mut ledger), Err(ContractError::Capacity)));
        assert!(matches!(
            ledger.checkpoint(&mut crate::workspace::preparation::NoopControl),
            Err(ContractError::Capacity)
        ));
    }
}

//! Complete offline notifications from one genuinely captured recorded queue.
//! This producer never establishes native currentness, recipient authentication,
//! evaluated quorum, schedule classification or delivery. All rows borrow the
//! admitted queue; one fixed identity registry and complete encoded output are
//! charged before retention. Parser/schema internals and allocator overhead are
//! separately raw/shape bounded, not a total-heap or syscall-preemption promise.

use std::io::{self, Write};
use std::rc::Rc;

use serde::Serialize;
use serde::ser::{SerializeSeq, Serializer};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::capture::HeldReviewInputs;
use super::decode::{ContractError, ContractLedger, Decoded, decode_queue};
use super::wire::{Domain, IDENTITY_DISCLAIMER, QueueDocument, ReviewItem, SourceModel, SourcePin};
use crate::workspace::preparation::WorkControl;

/// Complete row ceiling; exhaustion refuses the whole export.
const MAX_ROWS: usize = 100_000;
/// Whole compact JSON plus its final LF ceiling, sharing the invocation ledger.
const MAX_OUTPUT: usize = 10_485_760;
/// Reviewed packaged-schema contract; Root must supply these exact bytes.
const SCHEMA: &str = include_str!("../../schemas/forge.review-notifications-1.schema.json");
/// Fixed captured-original-only label; it is not a native approval assertion.
const SOURCE_SCOPE: &str = "captured-queue-recorded-snapshot-only";
/// Fixed local-artifact-only label; no delivery receipt exists.
const DELIVERY_SCOPE: &str = "local-export-only";

/// Opaque encoded export borrowing the complete genuine original owner.
/// No detached byte constructor, clone or deserialization grants this capability.
pub(crate) struct PreparedQueueNotifications<'a> {
    /// Actual confined original/root/namespace proof retained through publication.
    held: &'a Rc<HeldReviewInputs>,
    /// Actual successful Queue registration, rechecked at every final fence.
    queue_index: usize,
    /// Complete admitted JSON plus LF, with no omitted or paged notices.
    bytes: Vec<u8>,
}

impl PreparedQueueNotifications<'_> {
    /// Borrow the whole artifact; recorded pins are metadata, not native approval.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Recheck complete queue-only membership and genuine original generations.
    /// The final guarded publisher consumes this same owner immediately before rename.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            self.held.bind_queue_export_original(self.queue_index, ledger, control)?;
            self.held.verify_inputs(ledger, control)
        })
    }
}

/// Complete unfiltered denominators; assigned items and assignment rows differ.
#[derive(Clone, Copy, Serialize)]
struct Counts {
    /// All original selected queue items.
    queue_items: usize,
    /// Items having at least one explicit reviewer-role assignment.
    assigned_items: usize,
    /// Items having no explicit reviewer-role assignment.
    unassigned_items: usize,
    /// Complete explicit assignment occurrences, including distinct roles for one key.
    assignment_occurrences: usize,
    /// Assignment occurrences plus one null-recipient row per unassigned item.
    notification_rows: usize,
    /// Full declared asserted reviewer roster, not inferred recipients.
    declared_reviewers: usize,
    /// Full declared role roster.
    declared_roles: usize,
    /// Full declared policy roster.
    declared_policies: usize,
    /// Full recorded source-pin roster, not independently captured native files.
    recorded_source_pins: usize,
}

/// Fixed digest storage with original row ordinal; no expanded notice is retained.
struct Identity {
    /// SHA-256 of the exact versioned tuple, not the queue raw generation.
    digest: [u8; 32],
    /// Complete original item/assignment iteration position.
    ordinal: usize,
}

/// Exact collision order or restoration of original emitted order.
#[derive(Clone, Copy)]
enum IdentityOrder {
    /// Compare every complete digest for whole-registry collision detection.
    Digest,
    /// Restore the original row ordinal after collision inspection.
    Ordinal,
}

/// Borrowed raw queue generation reference; no reconstruction of original bytes.
#[derive(Serialize)]
struct QueueOriginal<'a> {
    /// Exact original queue schema marker.
    schema_version: &'a str,
    /// Asserted queue revision UUID.
    queue_id: &'a str,
    /// Actual captured whole-byte SHA-256, including whitespace and LF.
    raw_sha256: &'a str,
    /// Actual captured original length.
    byte_length: usize,
    /// Explicit recorded queue time, never a clock-derived export time.
    created_at: &'a str,
}

/// One borrowed notice with a fixed stack-rendered identity.
#[derive(Serialize)]
struct Notice<'a> {
    /// Exact versioned digest key, with indirect semantic dependence through item ID.
    notification_key: &'a str,
    /// Original queue-local stable item key.
    item_key: &'a str,
    /// Actual admitted derived review-item UUID.
    item_id: &'a str,
    /// Original declared domain; no native approval inferred.
    domain: Domain,
    /// Original exact adapter profile.
    adapter_version: &'a str,
    /// Original native subject identifier assertion.
    subject_id: &'a str,
    /// Recorded complete subject hash assertion.
    subject_sha256: &'a str,
    /// Recorded complete context hash assertion.
    context_sha256: &'a str,
    /// Original policy key.
    policy_key: &'a str,
    /// Complete recorded assignment/deadline/policy fingerprint.
    policy_sha256: &'a str,
    /// Complete original selected source keys, copied by borrowing.
    source_keys: &'a [String],
    /// Complete bounded recorded reason codes, with no prose.
    reason_codes: &'a [String],
    /// Exact recorded deadline or explicit null; no schedule evaluation.
    due_at: Option<&'a str>,
    /// Fixed explicit assignment or missing-assignment observation.
    intention: &'static str,
    /// Explicit asserted assigned reviewer key or null.
    recipient_key: Option<&'a str>,
    /// Explicit asserted assigned role or null.
    recipient_role: Option<&'a str>,
}

/// Complete borrowed sequence; no prefix or expanded DTO vector is allocated.
struct Rows<'a> {
    /// Actual admitted queue declarations.
    queue: &'a QueueDocument,
    /// Complete precharged identities restored to original row order.
    identities: &'a [Identity],
}

impl Serialize for Rows<'_> {
    /// Serialize exactly the complete original assignment/unassigned row bijection.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.identities.len()))?;
        let mut ordinal = 0;
        for item in &self.queue.items {
            if item.assignments.is_empty() {
                let identity = self
                    .identities
                    .get(ordinal)
                    .ok_or_else(|| serde::ser::Error::custom("notification row binding refused"))?;
                if identity.ordinal != ordinal {
                    return Err(serde::ser::Error::custom("notification row binding refused"));
                }
                let key = rendered_key(&identity.digest);
                let key = std::str::from_utf8(&key)
                    .map_err(|_| serde::ser::Error::custom("notification key encoding refused"))?;
                sequence.serialize_element(&notice(item, key, None, None))?;
                ordinal += 1;
            } else {
                for assignment in &item.assignments {
                    let identity = self.identities.get(ordinal).ok_or_else(|| {
                        serde::ser::Error::custom("notification row binding refused")
                    })?;
                    if identity.ordinal != ordinal {
                        return Err(serde::ser::Error::custom("notification row binding refused"));
                    }
                    let key = rendered_key(&identity.digest);
                    let key = std::str::from_utf8(&key).map_err(|_| {
                        serde::ser::Error::custom("notification key encoding refused")
                    })?;
                    sequence.serialize_element(&notice(
                        item,
                        key,
                        Some(&assignment.reviewer_key),
                        Some(&assignment.role_key),
                    ))?;
                    ordinal += 1;
                }
            }
        }
        if ordinal != self.identities.len() {
            return Err(serde::ser::Error::custom("notification row binding refused"));
        }
        sequence.end()
    }
}

/// The complete borrowed output profile, not a captured-source constructor.
#[derive(Serialize)]
struct Document<'a> {
    /// Exact additive notification marker.
    schema_version: &'static str,
    /// Existing mandatory asserted-only limitation.
    identity_disclaimer: &'static str,
    /// Original-only source label, never native currentness.
    source_scope: &'static str,
    /// Local-only delivery label.
    delivery_scope: &'static str,
    /// Existing minimized sensitivity profile.
    sensitivity: &'static str,
    /// Exact actual queue raw generation.
    queue_original: QueueOriginal<'a>,
    /// Complete original recorded native pins; none are read or approved here.
    recorded_source_pins: &'a [SourcePin],
    /// Complete unfiltered denominators.
    counts: Counts,
    /// Complete generated row bijection.
    notifications: Rows<'a>,
}

/// Borrow all emitted fields from the one admitted item and explicit assignment.
fn notice<'a>(
    item: &'a ReviewItem,
    key: &'a str,
    recipient: Option<&'a str>,
    role: Option<&'a str>,
) -> Notice<'a> {
    Notice {
        notification_key: key,
        item_key: &item.key,
        item_id: &item.item_id,
        domain: item.domain,
        adapter_version: &item.adapter_version,
        subject_id: &item.subject_id,
        subject_sha256: &item.subject_sha256,
        context_sha256: &item.context_sha256,
        policy_key: &item.policy_key,
        policy_sha256: &item.policy_sha256,
        source_keys: &item.source_keys,
        reason_codes: &item.context.reason_codes,
        due_at: item.due_at.as_deref(),
        intention: if recipient.is_some() { "request-review" } else { "assign-reviewer" },
        recipient_key: recipient,
        recipient_role: role,
    }
}

/// Count every item/assignment before identity registry or output growth.
fn counts(
    queue: &QueueDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Counts, ContractError> {
    ledger.visits(queue.items.len())?;
    let mut assigned = 0_usize;
    let mut unassigned = 0_usize;
    let mut assignments = 0_usize;
    for item in &queue.items {
        ledger.checkpoint(control)?;
        ledger.visits(item.assignments.len().checked_add(1).ok_or(ContractError::Capacity)?)?;
        if item.assignments.is_empty() {
            unassigned = unassigned.checked_add(1).ok_or(ContractError::Capacity)?;
        } else {
            assigned = assigned.checked_add(1).ok_or(ContractError::Capacity)?;
            assignments =
                assignments.checked_add(item.assignments.len()).ok_or(ContractError::Capacity)?;
        }
    }
    let rows = assignments.checked_add(unassigned).ok_or(ContractError::Capacity)?;
    if rows == 0 || rows > MAX_ROWS || assigned.checked_add(unassigned) != Some(queue.items.len()) {
        return Err(ledger.capacity());
    }
    Ok(Counts {
        queue_items: queue.items.len(),
        assigned_items: assigned,
        unassigned_items: unassigned,
        assignment_occurrences: assignments,
        notification_rows: rows,
        declared_reviewers: queue.reviewers.len(),
        declared_roles: queue.roles.len(),
        declared_policies: queue.policies.len(),
        recorded_source_pins: queue.source_pins.len(),
    })
}

/// Bounded hash writer keeps the exact first typed cause through serde's IO wrapper.
struct HashWriter<'a> {
    /// Fixed digest state; no JSON tuple buffer is retained.
    hash: Sha256,
    /// Actual monotonic command owner.
    ledger: &'a mut ContractLedger,
    /// Actual unchanged accepted control/deadline.
    control: &'a mut dyn WorkControl,
    /// First exact local typed failure.
    error: Option<ContractError>,
}

impl Write for HashWriter<'_> {
    /// Admit a full tuple token before digest work.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = self
            .ledger
            .checkpoint(self.control)
            .and_then(|()| self.ledger.visits(1))
            .and_then(|()| self.ledger.bytes(bytes.len()));
        if let Err(error) = result {
            self.error.get_or_insert(error);
            return Err(io::Error::other("notification identity refused"));
        }
        self.hash.update(bytes);
        Ok(bytes.len())
    }
    /// Digest flushing checks the same sticky caller control.
    fn flush(&mut self) -> io::Result<()> {
        self.ledger.checkpoint(self.control).map_err(|error| {
            self.error.get_or_insert(error);
            io::Error::other("notification identity refused")
        })
    }
}

/// Hash the exact compact ordered tuple; item ID retains its semantic dependencies.
fn identity(
    queue: &QueueDocument,
    item: &ReviewItem,
    recipient: Option<&str>,
    role: Option<&str>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<[u8; 32], ContractError> {
    let mut writer = HashWriter { hash: Sha256::new(), ledger, control, error: None };
    writer
        .write_all(b"forge.review-notification-key/1\0")
        .map_err(|_| writer.error.unwrap_or_else(|| writer.ledger.failure()))?;
    let intention = if recipient.is_some() { "request-review" } else { "assign-reviewer" };
    if serde_json::to_writer(
        &mut writer,
        &(&queue.queue_id, &item.item_id, intention, recipient, role),
    )
    .is_err()
    {
        return Err(writer.error.unwrap_or_else(|| writer.ledger.failure()));
    }
    writer.flush().map_err(|_| writer.error.unwrap_or_else(|| writer.ledger.failure()))?;
    Ok(writer.hash.finalize().into())
}

/// Render a fixed identity into stack ASCII without a retained per-row String.
fn rendered_key(digest: &[u8; 32]) -> [u8; 71] {
    let mut out = [0_u8; 71];
    out[..7].copy_from_slice(b"notice_");
    let alphabet = b"0123456789abcdef";
    for (index, byte) in digest.iter().enumerate() {
        out[7 + index * 2] = alphabet[usize::from(byte >> 4)];
        out[8 + index * 2] = alphabet[usize::from(byte & 15)];
    }
    out
}

/// Compare complete digest or original ordinal under the same actual work ledger.
fn less(
    left: &Identity,
    right: &Identity,
    order: IdentityOrder,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    ledger.checkpoint(control)?;
    ledger.matching(1)?;
    ledger.bytes(64 + 2 * std::mem::size_of::<usize>())?;
    Ok(match order {
        IdentityOrder::Digest => left.digest < right.digest,
        IdentityOrder::Ordinal => left.ordinal < right.ordinal,
    })
}

/// Restore heap order with immediate typed stop at every actual comparison.
fn sift(
    rows: &mut [Identity],
    mut root: usize,
    end: usize,
    order: IdentityOrder,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    loop {
        let Some(left) = root.checked_mul(2).and_then(|n| n.checked_add(1)).filter(|n| *n < end)
        else {
            return Ok(());
        };
        let mut child = left;
        if left + 1 < end && less(&rows[left], &rows[left + 1], order, ledger, control)? {
            child = left + 1;
        }
        if !less(&rows[root], &rows[child], order, ledger, control)? {
            return Ok(());
        }
        ledger.bytes(2 * std::mem::size_of::<Identity>())?;
        rows.swap(root, child);
        root = child;
    }
}

/// Complete bounded in-place heap sort, never an unchecked sort comparator callback.
fn sort(
    rows: &mut [Identity],
    order: IdentityOrder,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let length = rows.len();
    for root in (0..length / 2).rev() {
        sift(rows, root, length, order, ledger, control)?;
    }
    for end in (1..rows.len()).rev() {
        ledger.checkpoint(control)?;
        ledger.bytes(2 * std::mem::size_of::<Identity>())?;
        rows.swap(0, end);
        sift(rows, 0, end, order, ledger, control)?;
    }
    Ok(())
}

/// Precharge and construct the complete fixed identity registry before output growth.
fn identities(
    queue: &QueueDocument,
    rows: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<Identity>, ContractError> {
    ledger.derived(
        rows.checked_mul(std::mem::size_of::<Identity>())
            .and_then(|n| n.checked_add(std::mem::size_of::<Vec<Identity>>()))
            .ok_or(ContractError::Capacity)?,
    )?;
    let mut result = Vec::new();
    result.try_reserve_exact(rows).map_err(|_| ledger.capacity())?;
    for item in &queue.items {
        ledger.checkpoint(control)?;
        if item.assignments.is_empty() {
            let digest = identity(queue, item, None, None, ledger, control)?;
            result.push(Identity { digest, ordinal: result.len() });
        } else {
            for assignment in &item.assignments {
                let digest = identity(
                    queue,
                    item,
                    Some(&assignment.reviewer_key),
                    Some(&assignment.role_key),
                    ledger,
                    control,
                )?;
                result.push(Identity { digest, ordinal: result.len() });
            }
        }
    }
    if result.len() != rows {
        return Err(ContractError::Binding);
    }
    admit_identities(&mut result, ledger, control)?;
    Ok(result)
}

/// Inspect every complete digest for collision, then restore exact original row order.
/// Admitted queue identities/assignments already exclude equal tuples, so any repeated
/// digest is a whole-export refusal, never an overwritten notice or partial prefix.
fn admit_identities(
    rows: &mut [Identity],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    sort(rows, IdentityOrder::Digest, ledger, control)?;
    for pair in rows.windows(2) {
        ledger.checkpoint(control)?;
        ledger.matching(1)?;
        ledger.bytes(64)?;
        if pair[0].digest == pair[1].digest {
            return Err(ContractError::Binding);
        }
    }
    sort(rows, IdentityOrder::Ordinal, ledger, control)
}

/// A nonretaining complete JSON counter or one exactly pre-admitted retained sink.
struct JsonWriter<'a> {
    /// None for counting; Some for the sole complete encoded output.
    output: Option<Vec<u8>>,
    /// Complete byte count, including the final LF.
    count: usize,
    /// Exact pre-admitted limit (global ceiling only in dry-count mode).
    limit: usize,
    /// Actual monotonic invocation ledger.
    ledger: &'a mut ContractLedger,
    /// Actual accepted control/deadline.
    control: &'a mut dyn WorkControl,
    /// First typed writer cause survives a serde IO wrapper.
    error: Option<ContractError>,
}

impl JsonWriter<'_> {
    /// Admit each full token before any retained byte growth.
    fn append(&mut self, bytes: &[u8]) -> Result<(), ContractError> {
        self.ledger.checkpoint(self.control)?;
        self.ledger.visits(1)?;
        self.ledger.bytes(bytes.len())?;
        let next = self.count.checked_add(bytes.len()).ok_or_else(|| self.ledger.capacity())?;
        if next > self.limit {
            return Err(self.ledger.capacity());
        }
        if let Some(output) = &mut self.output {
            output.extend_from_slice(bytes);
        }
        self.count = next;
        Ok(())
    }
}

impl Write for JsonWriter<'_> {
    /// Write complete tokens without partial success or a retained prefix receipt.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.error.is_some() {
            return Err(io::Error::other("notification JSON refused"));
        }
        self.append(bytes).map_err(|error| {
            self.error.get_or_insert(error);
            io::Error::other("notification JSON refused")
        })?;
        Ok(bytes.len())
    }
    /// Check the original sticky control even though no external stream exists.
    fn flush(&mut self) -> io::Result<()> {
        self.ledger.checkpoint(self.control).map_err(|error| {
            self.error.get_or_insert(error);
            io::Error::other("notification JSON refused")
        })
    }
}

/// Count or serialize one complete borrowed document plus LF at the selected bound.
fn encode(
    document: &Document<'_>,
    limit: usize,
    retained: bool,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(usize, Option<Vec<u8>>), ContractError> {
    if limit == 0 || limit > MAX_OUTPUT {
        return Err(ContractError::Invalid);
    }
    let output = if retained {
        ledger.derived(
            limit.checked_add(std::mem::size_of::<Vec<u8>>()).ok_or(ContractError::Capacity)?,
        )?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(limit).map_err(|_| ledger.capacity())?;
        Some(bytes)
    } else {
        None
    };
    let mut writer = JsonWriter { output, count: 0, limit, ledger, control, error: None };
    if serde_json::to_writer(&mut writer, document).is_err() {
        return Err(writer.error.unwrap_or_else(|| writer.ledger.failure()));
    }
    writer.append(b"\n")?;
    writer.flush().map_err(|_| writer.error.unwrap_or_else(|| writer.ledger.failure()))?;
    Ok((writer.count, writer.output))
}

/// Precharge conservative decoded Value nodes, key bytes and string payload.
/// This is the existing 64-byte-per-node logical profile, not allocator accounting.
struct Storage<'a> {
    /// Same monotonic command storage/work owner.
    ledger: &'a mut ContractLedger,
    /// Same accepted cooperative control.
    control: &'a mut dyn WorkControl,
}

impl Storage<'_> {
    /// Reserve one actual projected Value node before final parser allocation.
    fn scalar(&mut self) -> Result<(), ContractError> {
        self.ledger.checkpoint(self.control)?;
        self.ledger.visits(1)?;
        self.ledger.derived(64)
    }
    /// Reserve a complete string payload without creating a copy.
    fn string(&mut self, text: &str) -> Result<(), ContractError> {
        self.scalar()?;
        self.ledger.bytes(text.len())?;
        self.ledger.derived(text.len())
    }
    /// Reserve a nullable string's node and complete present payload.
    fn optional(&mut self, text: Option<&str>) -> Result<(), ContractError> {
        match text {
            Some(text) => self.string(text),
            None => self.scalar(),
        }
    }
    /// Reserve one object node and every fixed output key before projection growth.
    fn object(&mut self, keys: &[&str]) -> Result<(), ContractError> {
        self.scalar()?;
        for key in keys {
            self.ledger.bytes(key.len())?;
            self.ledger.derived(key.len())?;
        }
        Ok(())
    }
    /// Reserve the full original string array before parser-owned elements grow.
    fn strings(&mut self, rows: &[String]) -> Result<(), ContractError> {
        self.scalar()?;
        for text in rows {
            self.string(text)?;
        }
        Ok(())
    }
}

/// Exact serde domain wire names, with no classification or authority computation.
fn domain_name(domain: Domain) -> &'static str {
    match domain {
        Domain::MappingAssertion => "mapping-assertion",
        Domain::ApplicabilityDecision => "applicability-decision",
    }
}

/// Exact closed source model wire names; no model is promoted or admitted here.
fn model_name(model: SourceModel) -> &'static str {
    match model {
        SourceModel::Mapping => "mapping",
        SourceModel::Catalog => "catalog",
        SourceModel::Profile => "profile",
        SourceModel::ResolvedCatalog => "resolved-catalog",
        SourceModel::ComponentDefinition => "component-definition",
        SourceModel::MappingManifest => "mapping-manifest",
        SourceModel::ApplicabilityManifest => "applicability-manifest",
        SourceModel::ApplicabilityReport => "applicability-report",
        SourceModel::LifecycleRecord => "lifecycle-record",
    }
}

/// Reserve the complete actual schema-Value projection before final strict parse.
fn reserve_projection(
    document: &Document<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut storage = Storage { ledger, control };
    storage.object(&[
        "schema_version",
        "identity_disclaimer",
        "source_scope",
        "delivery_scope",
        "sensitivity",
        "queue_original",
        "recorded_source_pins",
        "counts",
        "notifications",
    ])?;
    for text in [
        document.schema_version,
        document.identity_disclaimer,
        document.source_scope,
        document.delivery_scope,
        document.sensitivity,
    ] {
        storage.string(text)?;
    }
    storage.object(&["schema_version", "queue_id", "raw_sha256", "byte_length", "created_at"])?;
    for text in [
        document.queue_original.schema_version,
        document.queue_original.queue_id,
        document.queue_original.raw_sha256,
        document.queue_original.created_at,
    ] {
        storage.string(text)?;
    }
    storage.scalar()?;
    storage.scalar()?;
    for pin in document.recorded_source_pins {
        storage.object(&[
            "artifact_key",
            "model",
            "native_root_uuid",
            "raw_sha256",
            "byte_length",
            "schema_identity",
        ])?;
        for text in [
            pin.artifact_key.as_str(),
            model_name(pin.model),
            pin.raw_sha256.as_str(),
            pin.schema_identity.as_str(),
        ] {
            storage.string(text)?;
        }
        storage.optional(pin.native_root_uuid.as_deref())?;
        storage.scalar()?;
    }
    storage.object(&[
        "queue_items",
        "assigned_items",
        "unassigned_items",
        "assignment_occurrences",
        "notification_rows",
        "declared_reviewers",
        "declared_roles",
        "declared_policies",
        "recorded_source_pins",
    ])?;
    for _ in 0..9 {
        storage.scalar()?;
    }
    storage.scalar()?;
    for item in &document.notifications.queue.items {
        if item.assignments.is_empty() {
            reserve_notice(&mut storage, item, None, None)?;
        } else {
            for assignment in &item.assignments {
                reserve_notice(
                    &mut storage,
                    item,
                    Some(&assignment.reviewer_key),
                    Some(&assignment.role_key),
                )?;
            }
        }
    }
    Ok(())
}

/// Reserve all actual complete row fields, arrays and nullable values by borrowing.
fn reserve_notice(
    storage: &mut Storage<'_>,
    item: &ReviewItem,
    recipient: Option<&str>,
    role: Option<&str>,
) -> Result<(), ContractError> {
    storage.object(&[
        "notification_key",
        "item_key",
        "item_id",
        "domain",
        "adapter_version",
        "subject_id",
        "subject_sha256",
        "context_sha256",
        "policy_key",
        "policy_sha256",
        "source_keys",
        "reason_codes",
        "due_at",
        "intention",
        "recipient_key",
        "recipient_role",
    ])?;
    storage.scalar()?;
    storage.ledger.derived(71)?;
    for text in [
        item.key.as_str(),
        item.item_id.as_str(),
        domain_name(item.domain),
        item.adapter_version.as_str(),
        item.subject_id.as_str(),
        item.subject_sha256.as_str(),
        item.context_sha256.as_str(),
        item.policy_key.as_str(),
        item.policy_sha256.as_str(),
        if recipient.is_some() { "request-review" } else { "assign-reviewer" },
    ] {
        storage.string(text)?;
    }
    storage.strings(&item.source_keys)?;
    storage.strings(&item.context.reason_codes)?;
    storage.optional(item.due_at.as_deref())?;
    storage.optional(recipient)?;
    storage.optional(role)
}

/// Require exact actual string/null equality after charging the complete compared bytes.
fn same_text(
    value: &Value,
    key: &str,
    expected: Option<&str>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    ledger.bytes(key.len())?;
    let actual = value.get(key).ok_or(ContractError::Binding)?;
    let equal = match expected {
        Some(expected) => {
            let actual = actual.as_str().ok_or(ContractError::Binding)?;
            ledger
                .bytes(actual.len().checked_add(expected.len()).ok_or(ContractError::Capacity)?)?;
            actual == expected
        }
        None => actual.is_null(),
    };
    if !equal {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Require exact integer output without a floating-point conversion.
fn same_count(
    value: &Value,
    key: &str,
    expected: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.matching(1)?;
    ledger.bytes(key.len() + std::mem::size_of::<usize>())?;
    if value.get(key).and_then(Value::as_u64)
        != Some(u64::try_from(expected).map_err(|_| ContractError::Capacity)?)
    {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Compare the complete actual array, with no selected prefix or copied expected Value.
fn same_strings(
    value: &Value,
    key: &str,
    expected: &[String],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.bytes(key.len())?;
    let actual = value.get(key).and_then(Value::as_array).ok_or(ContractError::Binding)?;
    if actual.len() != expected.len() {
        return Err(ContractError::Binding);
    }
    for (actual, expected) in actual.iter().zip(expected) {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        ledger.matching(1)?;
        let actual = actual.as_str().ok_or(ContractError::Binding)?;
        ledger.bytes(actual.len().checked_add(expected.len()).ok_or(ContractError::Capacity)?)?;
        if actual != expected {
            return Err(ContractError::Binding);
        }
    }
    Ok(())
}

/// Compare one entire emitted row with the actual queue/assignment and fixed digest.
fn compare_notice(
    value: &Value,
    item: &ReviewItem,
    key: &str,
    recipient: Option<&str>,
    role: Option<&str>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for (field, expected) in [
        ("notification_key", Some(key)),
        ("item_key", Some(item.key.as_str())),
        ("item_id", Some(item.item_id.as_str())),
        ("domain", Some(domain_name(item.domain))),
        ("adapter_version", Some(item.adapter_version.as_str())),
        ("subject_id", Some(item.subject_id.as_str())),
        ("subject_sha256", Some(item.subject_sha256.as_str())),
        ("context_sha256", Some(item.context_sha256.as_str())),
        ("policy_key", Some(item.policy_key.as_str())),
        ("policy_sha256", Some(item.policy_sha256.as_str())),
        ("due_at", item.due_at.as_deref()),
        ("intention", Some(if recipient.is_some() { "request-review" } else { "assign-reviewer" })),
        ("recipient_key", recipient),
        ("recipient_role", role),
    ] {
        same_text(value, field, expected, ledger, control)?;
    }
    same_strings(value, "source_keys", &item.source_keys, ledger, control)?;
    same_strings(value, "reason_codes", &item.context.reason_codes, ledger, control)
}

/// Validate the consumed schema and every complete generated projection field.
fn oracle(
    bytes: &[u8],
    decoded: &Decoded<'_, QueueDocument>,
    document: &Document<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.bytes(bytes.len())?;
    let parsed = crate::json_strict::parse_value(
        bytes,
        "review-notifications",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: 65_536 },
    );
    ledger.checkpoint(control)?;
    let value = parsed.map_err(|_| ContractError::Invalid)?;
    ledger.bytes(SCHEMA.len())?;
    ledger.derived(SCHEMA.len().checked_mul(8).ok_or(ContractError::Capacity)?)?;
    let parsed_definition = serde_json::from_str::<Value>(SCHEMA);
    ledger.checkpoint(control)?;
    let definition = parsed_definition.map_err(|_| ContractError::SchemaDefinition)?;
    let compiled = jsonschema::validator_for(&definition);
    ledger.checkpoint(control)?;
    let validator = compiled.map_err(|_| ContractError::SchemaDefinition)?;
    ledger.bytes(bytes.len())?;
    let schema_valid = validator.is_valid(&value);
    ledger.checkpoint(control)?;
    if !schema_valid {
        return Err(ContractError::Invalid);
    }
    compare_projection(&value, decoded, document, ledger, control)
}

/// Compare every complete admitted projection field after the strict schema phase.
/// The same borrowed queue, ledger and caller retain the original ordered checks.
fn compare_projection(
    value: &Value,
    decoded: &Decoded<'_, QueueDocument>,
    document: &Document<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for (field, expected) in [
        ("schema_version", "forge.review-notifications/1"),
        ("identity_disclaimer", IDENTITY_DISCLAIMER),
        ("source_scope", SOURCE_SCOPE),
        ("delivery_scope", DELIVERY_SCOPE),
        ("sensitivity", "ids-and-hashes"),
    ] {
        same_text(value, field, Some(expected), ledger, control)?;
    }
    let original = value.get("queue_original").ok_or(ContractError::Binding)?;
    let queue = decoded.document();
    for (field, expected) in [
        ("schema_version", queue.schema_version.as_str()),
        ("queue_id", queue.queue_id.as_str()),
        ("raw_sha256", decoded.raw_sha256()),
        ("created_at", queue.created_at.as_str()),
    ] {
        same_text(original, field, Some(expected), ledger, control)?;
    }
    same_count(original, "byte_length", decoded.raw().len(), ledger)?;
    let pins = value
        .get("recorded_source_pins")
        .and_then(Value::as_array)
        .ok_or(ContractError::Binding)?;
    if pins.len() != queue.source_pins.len() {
        return Err(ContractError::Binding);
    }
    for (actual, pin) in pins.iter().zip(&queue.source_pins) {
        for (field, expected) in [
            ("artifact_key", Some(pin.artifact_key.as_str())),
            ("model", Some(model_name(pin.model))),
            ("native_root_uuid", pin.native_root_uuid.as_deref()),
            ("raw_sha256", Some(pin.raw_sha256.as_str())),
            ("schema_identity", Some(pin.schema_identity.as_str())),
        ] {
            same_text(actual, field, expected, ledger, control)?;
        }
        let length = usize::try_from(pin.byte_length).map_err(|_| ContractError::Capacity)?;
        same_count(actual, "byte_length", length, ledger)?;
    }
    let actual_counts = value.get("counts").ok_or(ContractError::Binding)?;
    let expected = document.counts;
    for (field, expected) in [
        ("queue_items", expected.queue_items),
        ("assigned_items", expected.assigned_items),
        ("unassigned_items", expected.unassigned_items),
        ("assignment_occurrences", expected.assignment_occurrences),
        ("notification_rows", expected.notification_rows),
        ("declared_reviewers", expected.declared_reviewers),
        ("declared_roles", expected.declared_roles),
        ("declared_policies", expected.declared_policies),
        ("recorded_source_pins", expected.recorded_source_pins),
    ] {
        same_count(actual_counts, field, expected, ledger)?;
    }
    let rows =
        value.get("notifications").and_then(Value::as_array).ok_or(ContractError::Binding)?;
    if rows.len() != document.notifications.identities.len() {
        return Err(ContractError::Binding);
    }
    let mut ordinal = 0;
    for item in &queue.items {
        ledger.checkpoint(control)?;
        if item.assignments.is_empty() {
            let key = rendered_key(&document.notifications.identities[ordinal].digest);
            let key = std::str::from_utf8(&key).map_err(|_| ContractError::Binding)?;
            compare_notice(&rows[ordinal], item, key, None, None, ledger, control)?;
            ordinal += 1;
        } else {
            for assignment in &item.assignments {
                let key = rendered_key(&document.notifications.identities[ordinal].digest);
                let key = std::str::from_utf8(&key).map_err(|_| ContractError::Binding)?;
                compare_notice(
                    &rows[ordinal],
                    item,
                    key,
                    Some(&assignment.reviewer_key),
                    Some(&assignment.role_key),
                    ledger,
                    control,
                )?;
                ordinal += 1;
            }
        }
    }
    if ordinal != rows.len() {
        return Err(ContractError::Binding);
    }
    ledger.checkpoint(control)
}

/// Prepare all local notifications from an exact complete genuinely captured queue.
/// Native pins and due times stay recorded metadata; no native source is loaded.
/// The pending Root-owned queue-only binder excludes every extra retained original.
pub(crate) fn prepare<'a>(
    held: &'a Rc<HeldReviewInputs>,
    queue_index: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedQueueNotifications<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        held.bind_queue_export_original(queue_index, ledger, control)?;
        let decoded_result = decode_queue(held.bytes(queue_index)?, ledger, control);
        ledger.checkpoint(control)?;
        let decoded = decoded_result?;
        let counts = counts(decoded.document(), ledger, control)?;
        let identities = identities(decoded.document(), counts.notification_rows, ledger, control)?;
        let document = Document {
            schema_version: "forge.review-notifications/1",
            identity_disclaimer: IDENTITY_DISCLAIMER,
            source_scope: SOURCE_SCOPE,
            delivery_scope: DELIVERY_SCOPE,
            sensitivity: "ids-and-hashes",
            queue_original: QueueOriginal {
                schema_version: &decoded.document().schema_version,
                queue_id: &decoded.document().queue_id,
                raw_sha256: decoded.raw_sha256(),
                byte_length: decoded.raw().len(),
                created_at: &decoded.document().created_at,
            },
            recorded_source_pins: &decoded.document().source_pins,
            counts,
            notifications: Rows { queue: decoded.document(), identities: &identities },
        };
        let (admitted, _) = encode(&document, MAX_OUTPUT, false, ledger, control)?;
        reserve_projection(&document, ledger, control)?;
        let (actual, output) = encode(&document, admitted, true, ledger, control)?;
        if actual != admitted {
            return Err(ContractError::Binding);
        }
        let bytes = output.ok_or(ContractError::Binding)?;
        oracle(&bytes, &decoded, &document, ledger, control)?;
        held.bind_queue_export_original(queue_index, ledger, control)?;
        held.verify_inputs(ledger, control)?;
        Ok(PreparedQueueNotifications { held, queue_index, bytes })
    })
}

#[cfg(test)]
#[path = "notifications_tests.rs"]
/// Real queue-original controls and separately scoped primitive capacity controls.
mod tests;

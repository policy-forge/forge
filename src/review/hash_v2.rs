//! Non-authorizing bounded binary profiles for the separately closed Lifecycle exchange.
//! Complete raw/native ownership is supplied only by the genuine receiver, never
//! by these plain digest operands. No JSON serialization substitutes for framing.

use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase, validate_pin, validate_pins};
use super::wire_v2::{
    ADAPTER, AbstentionRule, AuthorSeparation, DOMAIN, Disposition, NativeModelV2, QueueDocumentV2,
    RequestedAction, ReviewItemV2, ReviewPolicy, SourceKindV2, SourcePinV2,
};
use crate::workspace::preparation::WorkControl;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

/// Fixed streaming state and logical scratch allowance, not measured heap usage.
const STATE_BYTES: usize = 256;
/// Work is interruptible between actual borrowed update chunks.
const CHUNK: usize = 32 * 1024;

/// Complete private purpose occurrence; ordinary data cannot create a native lease.
pub(crate) struct RosterEntry<'a> {
    /// Complete actual native occurrence kind, including compatible repeats.
    pub(crate) purpose: SourceKindV2,
    /// Record is null; source/generated retain exact original private UTF-8 routes.
    pub(crate) route: Option<&'a str>,
    /// Full eight-field pin of this actual occurrence.
    pub(crate) pin: &'a SourcePinV2,
}
/// Actual original response registration data, including exact byte duplicates.
pub(crate) struct ResponseOriginal<'a> {
    /// Actual decoded original response identifier.
    pub(crate) response_id: &'a str,
    /// Actual full original byte hash, never semantic reconstruction.
    pub(crate) raw_sha256: &'a str,
    /// Actual full original extent, including LF.
    pub(crate) byte_length: u64,
}

/// Private streaming encoder precharges every actual prefix, tag, frame and operand.
struct Stream<'a> {
    /// One original invocation ledger.
    ledger: &'a mut ContractLedger,
    /// Same accepted control; no deadline renewal or replacement.
    control: &'a mut dyn WorkControl,
    /// Actual running hash state; never a complete payload allocation.
    digest: Sha256,
    /// Complete encoded count, including prefix and every frame/tag.
    encoded: usize,
    /// Fixed profile ceiling, not an alternate invocation allowance.
    cap: usize,
}
impl<'a> Stream<'a> {
    /// Reserve streaming state before construction, then emit the exact NUL prefix.
    fn new(
        prefix: &[u8],
        cap: usize,
        ledger: &'a mut ContractLedger,
        control: &'a mut dyn WorkControl,
    ) -> Result<Self, ContractError> {
        ledger.derived(STATE_BYTES)?;
        let mut stream = Self { ledger, control, digest: Sha256::new(), encoded: 0, cap };
        stream.put(prefix)?;
        Ok(stream)
    }
    /// Precharge the whole encoded extent and each actual borrowed digest update.
    fn put(&mut self, bytes: &[u8]) -> Result<(), ContractError> {
        let next = self.encoded.checked_add(bytes.len()).ok_or_else(|| self.ledger.capacity())?;
        if next > self.cap {
            return Err(self.ledger.capacity());
        }
        self.ledger.bytes(bytes.len())?;
        for chunk in bytes.chunks(CHUNK) {
            checkpoint(self.ledger, self.control)?;
            self.digest.update(chunk);
        }
        self.encoded = next;
        Ok(())
    }
    /// Exact eight-byte unsigned little-endian frame.
    fn u64(&mut self, value: u64) -> Result<(), ContractError> {
        self.put(&value.to_le_bytes())
    }
    /// Convert real Rust cardinalities with checked representation before emission.
    fn count(&mut self, value: usize) -> Result<(), ContractError> {
        let value = u64::try_from(value).map_err(|_| self.ledger.capacity())?;
        self.u64(value)
    }
    /// Exact one-byte tag; no unknown-enum fallback is permitted.
    fn byte(&mut self, value: u8) -> Result<(), ContractError> {
        self.put(&[value])
    }
    /// Length-framed original UTF-8 bytes, with no normalization or JSON escapes.
    fn string(&mut self, value: &str) -> Result<(), ContractError> {
        self.count(value.len())?;
        self.put(value.as_bytes())
    }
    /// Null is 00; present empty is 01 followed by the empty string frame.
    fn optional_string(&mut self, value: Option<&str>) -> Result<(), ContractError> {
        match value {
            None => self.byte(0),
            Some(value) => {
                self.byte(1)?;
                self.string(value)
            }
        }
    }
    /// Decode exactly lowercase64 hex to fixed stack bytes before the hash update.
    fn hash(&mut self, value: &str) -> Result<(), ContractError> {
        self.ledger.bytes(value.len())?;
        if value.len() != 64 {
            return Err(ContractError::Invalid);
        }
        let mut decoded = [0_u8; 32];
        for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
            /// Accept exact lowercase hexadecimal digits without coercion.
            fn nibble(byte: u8) -> Option<u8> {
                match byte {
                    b'0'..=b'9' => Some(byte - b'0'),
                    b'a'..=b'f' => Some(byte - b'a' + 10),
                    _ => None,
                }
            }
            let high = nibble(pair[0]).ok_or(ContractError::Invalid)?;
            let low = nibble(pair[1]).ok_or(ContractError::Invalid)?;
            decoded[index] = high * 16 + low;
        }
        self.put(&decoded)
    }
    /// Exact complete declared string list in its already validated order.
    fn strings(&mut self, values: &[String]) -> Result<(), ContractError> {
        self.count(values.len())?;
        for value in values {
            self.ledger.visits(1)?;
            self.string(value)?;
        }
        Ok(())
    }
    /// Full eight-field pin in normative order; original generated UUID text survives.
    fn pin(&mut self, pin: &SourcePinV2) -> Result<(), ContractError> {
        self.ledger.visits(1)?;
        self.string(&pin.artifact_key)?;
        self.byte(kind_tag(pin.kind))?;
        self.hash(&pin.raw_sha256)?;
        self.u64(pin.byte_length)?;
        self.optional_string(pin.schema_identity.as_deref())?;
        self.string(&pin.validation_profile)?;
        self.optional_string(pin.native_model.map(NativeModelV2::as_str))?;
        self.optional_string(pin.native_root_uuid.as_deref())
    }
    /// Complete public pin order; source validation occurs before the stream opens.
    fn pins(&mut self, pins: &[SourcePinV2]) -> Result<(), ContractError> {
        self.count(pins.len())?;
        for pin in pins {
            self.pin(pin)?;
        }
        Ok(())
    }
    /// Native and review enums use fixed explicit contract tags only.
    fn dispositions(&mut self, values: &[Disposition]) -> Result<(), ContractError> {
        self.count(values.len())?;
        for value in values {
            self.byte(disposition_tag(*value))?;
        }
        Ok(())
    }
    /// Selected complete asserted policy, without evaluating quorum or native authority.
    fn policy(&mut self, policy: &ReviewPolicy) -> Result<(), ContractError> {
        self.string(&policy.key)?;
        self.count(policy.seats.len())?;
        for seat in &policy.seats {
            self.ledger.visits(1)?;
            self.string(&seat.role_key)?;
            self.u64(u64::from(seat.count))?;
        }
        self.count(policy.substitutions.len())?;
        for edge in &policy.substitutions {
            self.ledger.visits(1)?;
            self.string(&edge.seat_role)?;
            self.string(&edge.reviewer_key)?;
            self.string(&edge.asserted_role)?;
            self.string(&edge.reason_code)?;
        }
        match policy.abstention_rule {
            AbstentionRule::Nonapproving => self.byte(1)?,
        }
        self.strings(&policy.empty_abstention_reasons)?;
        match policy.author_separation {
            AuthorSeparation::DeclaredKeys => self.byte(1),
        }
    }
    /// Final digest state is ordinary data; the same caller is fenced before return.
    fn finish(self) -> Result<[u8; 32], ContractError> {
        checkpoint(self.ledger, self.control)?;
        Ok(self.digest.finalize().into())
    }
    /// Reserve all64 output bytes and its fixed string representation before formatting.
    fn finish_hex(self) -> Result<String, ContractError> {
        self.ledger.derived(128)?;
        let bytes = self.finish()?;
        Ok(crate::hashing::lower_hex(&bytes))
    }
}
/// Closed purpose tags from the frozen byte contract.
fn kind_tag(kind: SourceKindV2) -> u8 {
    match kind {
        SourceKindV2::LifecycleRecord => 1,
        SourceKindV2::OpaqueSource => 2,
        SourceKindV2::GeneratedArtifact => 3,
    }
}
/// Closed disposition tags preserve all dissent choices.
fn disposition_tag(value: Disposition) -> u8 {
    match value {
        Disposition::Approve => 1,
        Disposition::Reject => 2,
        Disposition::RequestChanges => 3,
        Disposition::Abstain => 4,
        Disposition::Superseded => 5,
    }
}
/// Private native keys retain full UTF-8 bytes, including interior NUL and spaces.
fn keys(policy: &str, version: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    for key in [policy, version] {
        ledger.bytes(key.len())?;
        if key.len() > 4096 || key.trim().is_empty() {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}
/// Public fingerprints accept only the exact separate domain/adapter, fixed context and pins.
fn public(
    queue: &QueueDocumentV2,
    item: &ReviewItemV2,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    validate_pins(&queue.source_pins, ledger)?;
    ledger.bytes(item.adapter_version.len())?;
    if item.adapter_version != ADAPTER {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Stable native key-pair ID; reproducing it supplies no native/source approval.
pub(crate) fn subject_id(
    policy: &str,
    version: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        keys(policy, version, ledger)?;
        let mut stream =
            Stream::new(b"forge.lifecycle-review-subject-id/1\0", 16_384, ledger, control)?;
        stream.string(policy)?;
        stream.string(version)?;
        stream.ledger.derived(74 + 64)?;
        let digest = stream.finish_hex()?;
        let mut id = String::with_capacity(74);
        id.push_str("lifecycle:");
        id.push_str(&digest);
        Ok(id)
    })
}
/// Validate complete ordinary native purpose data before hashing, never sealing it.
fn roster(rows: &[RosterEntry<'_>], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    if !(2..=100).contains(&rows.len()) {
        return Err(ContractError::Invalid);
    }
    let mut total = 0_u64;
    let mut previous: Option<&str> = None;
    for (index, row) in rows.iter().enumerate() {
        ledger.visits(1)?;
        validate_pin(row.pin, ledger)?;
        total = total.checked_add(row.pin.byte_length).ok_or_else(|| ledger.capacity())?;
        let expected = match index {
            0 => SourceKindV2::LifecycleRecord,
            1 => SourceKindV2::OpaqueSource,
            _ => SourceKindV2::GeneratedArtifact,
        };
        if row.purpose != expected || row.pin.kind != expected {
            return Err(ContractError::Invalid);
        }
        if index == 0 {
            if row.route.is_some() {
                return Err(ContractError::Invalid);
            }
        } else {
            let route = row.route.ok_or(ContractError::Invalid)?;
            let extent = route.len().checked_mul(2).ok_or_else(|| ledger.capacity())?;
            ledger.bytes(extent)?;
            if route.is_empty() || route.len() > 4096 {
                return Err(ContractError::Invalid);
            }
            if index >= 2 {
                let comparison_extent = previous
                    .map_or(0, str::len)
                    .checked_add(route.len())
                    .ok_or_else(|| ledger.capacity())?;
                ledger.bytes(comparison_extent)?;
                if super::decode_v2::generated_ordinal(&row.pin.artifact_key) != Some(index - 2)
                    || previous.is_some_and(|prior| prior >= route)
                {
                    return Err(ContractError::Invalid);
                }
                previous = Some(route);
            }
        }
    }
    if total > 52_428_800 {
        return Err(ledger.capacity());
    }
    Ok(())
}
/// Full private record/source/generated roster fingerprint; purpose repeats remain complete.
pub(crate) fn sources(
    rows: &[RosterEntry<'_>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        roster(rows, ledger)?;
        let mut stream =
            Stream::new(b"forge.lifecycle-review-sources/1\0", 1_048_576, ledger, control)?;
        stream.count(rows.len())?;
        for row in rows {
            stream.byte(kind_tag(row.purpose))?;
            stream.optional_string(row.route)?;
            stream.pin(row.pin)?;
        }
        stream.finish_hex()
    })
}
/// Whole private source-roster composition; a supplied digest cannot issue a reader owner.
pub(crate) fn subject(
    policy: &str,
    version: &str,
    sources: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        keys(policy, version, ledger)?;
        let mut stream =
            Stream::new(b"forge.lifecycle-review-subject/1\0", 16_384, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.string(policy)?;
        stream.string(version)?;
        stream.hash(sources)?;
        stream.finish_hex()
    })
}
/// Exact minimized context encoding with complete public source pins.
pub(crate) fn context(
    queue: &QueueDocumentV2,
    item: &ReviewItemV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        public(queue, item, ledger)?;
        let reason = item.context.reason_codes.as_slice();
        ledger.visits(reason.len())?;
        for value in reason {
            ledger.bytes(value.len())?;
        }
        if reason.len() != 1
            || reason[0] != "lifecycle-approved-current"
            || !item.context.related_subject_ids.is_empty()
        {
            return Err(ContractError::Invalid);
        }
        let mut stream = Stream::new(b"forge.review-context/2\0", 1_048_576, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.pins(&queue.source_pins)?;
        stream.strings(reason)?;
        stream.strings(&item.context.related_subject_ids)?;
        stream.finish_hex()
    })
}
/// Selected asserted policy binding; exact queue raw identity separately binds all rosters.
pub(crate) fn policy(
    queue: &QueueDocumentV2,
    selected: &ReviewPolicy,
    item: &ReviewItemV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        public(queue, item, ledger)?;
        super::decode_v2::validate_policy_operands(queue, selected, item, ledger)?;
        let mut stream =
            Stream::new(b"forge.review-policy-binding/2\0", 1_048_576, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.pins(&queue.source_pins)?;
        stream.policy(selected)?;
        stream.strings(&item.author_keys)?;
        stream.count(item.assignments.len())?;
        for assignment in &item.assignments {
            stream.ledger.visits(1)?;
            stream.string(&assignment.reviewer_key)?;
            stream.string(&assignment.role_key)?;
        }
        stream.optional_string(item.due_at.as_deref())?;
        stream.dispositions(&item.allowed_dispositions)?;
        stream.finish_hex()
    })
}
/// Queue-local item `UUIDv5` over the exact32-byte SHA-256 framing digest.
pub(crate) fn item_id(
    queue: &QueueDocumentV2,
    item: &ReviewItemV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        public(queue, item, ledger)?;
        let mut stream = Stream::new(b"forge.review-item/2\0", 1_048_576, ledger, control)?;
        stream.string(&queue.queue_id)?;
        stream.string(&item.key)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.string(&item.subject_id)?;
        match item.requested_action {
            RequestedAction::ReReview => stream.byte(1)?,
        }
        stream.strings(&item.source_keys)?;
        stream.pins(&queue.source_pins)?;
        stream.hash(&item.subject_sha256)?;
        stream.hash(&item.context_sha256)?;
        stream.hash(&item.policy_sha256)?;
        stream.ledger.bytes(48)?;
        stream.ledger.derived(100)?;
        let digest = stream.finish()?;
        let namespace = Uuid::from_bytes([
            0x9f, 0x7c, 0x24, 0xf8, 0x56, 0x1d, 0x55, 0xa1, 0xa4, 0x3d, 0xfc, 0x4d, 0x81, 0x59,
            0xb4, 0x60,
        ]);
        Ok(Uuid::new_v5(&namespace, &digest).to_string())
    })
}
/// Inert complete original registration hash; only a genuine finalizer may record it as current.
pub(crate) fn closure_generation(
    queue_hash: &str,
    queue_length: u64,
    pins: &[SourcePinV2],
    responses: &[ResponseOriginal<'_>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    phase(ledger, control, |ledger, control| {
        validate_pins(pins, ledger)?;
        if queue_length == 0 || queue_length > 10_485_760 || responses.len() > 10_000 {
            return Err(ContractError::Invalid);
        }
        let mut extent = 0_u64;
        for row in responses {
            ledger.visits(1)?;
            ledger.bytes(row.response_id.len())?;
            ledger.derived(100)?;
            super::validate::uuid(row.response_id)?;
            if row.byte_length == 0 || row.byte_length > 1_048_576 {
                return Err(ContractError::Invalid);
            }
            extent = extent.checked_add(row.byte_length).ok_or_else(|| ledger.capacity())?;
        }
        if extent > 33_554_432 {
            return Err(ledger.capacity());
        }
        let mut stream =
            Stream::new(b"forge.review-current-closure/2\0", 33_554_432, ledger, control)?;
        stream.string(DOMAIN)?;
        stream.string(ADAPTER)?;
        stream.hash(queue_hash)?;
        stream.u64(queue_length)?;
        stream.pins(pins)?;
        stream.count(responses.len())?;
        for row in responses {
            stream.string(row.response_id)?;
            stream.hash(row.raw_sha256)?;
            stream.u64(row.byte_length)?;
        }
        stream.finish_hex()
    })
}

#[cfg(test)]
#[path = "hash_v2_tests.rs"]
mod tests;

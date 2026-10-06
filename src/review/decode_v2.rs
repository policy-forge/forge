//! Strict separately closed Lifecycle declarations with exact borrowed raw identity.
//! Successful decoding is inert data. It supplies no native facts, captured lease,
//! accepted reviewer, currentness holder or successful domain transition.
//! Raw/tree/typed plans use logical units; primitive allocation internals are not
//! a measured total heap or hard syscall-preemption guarantee.

use super::decode::{ContractError, ContractLedger, OriginalRelation};
use super::wire_v2::{
    ADAPTER, Disposition, DispositionsDocumentV2, IDENTITY_DISCLAIMER, ItemDisposition, ItemState,
    LifecycleInputsV1, MetSeat, QueueDocumentV2, RecordedCurrentness, RecordedResponse,
    ResponseClassification, ResponseDocumentV2, ReviewItemV2, ReviewPolicy, Reviewer, SourceKindV2,
    SourcePinV2,
};
use crate::workspace::preparation::WorkControl;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeSet;

/// Observe the actual original controller's interruption even after an Ok checkpoint.
pub(crate) fn checkpoint(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        if let Some(reason) = control.interruption() {
            return Err(ContractError::Interrupted(reason));
        }
        Ok(())
    })
}
/// First admission stop wins; ordinary Ok/Invalid/Binding remain retained until a postfence.
pub(crate) fn phase<T>(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
    work: impl FnOnce(&mut ContractLedger, &mut dyn WorkControl) -> Result<T, ContractError>,
) -> Result<T, ContractError> {
    ledger.bound(|ledger| {
        checkpoint(ledger, control)?;
        let ordinary = work(ledger, control);
        let ordinary = ledger.bound(|_| ordinary);
        checkpoint(ledger, control)?;
        ordinary
    })
}

/// Exact original extent plus admitted inert declarations; no public constructor or Clone.
pub(crate) struct DecodedV2<'a, T> {
    /// Borrow actual unchanged original bytes, including raw whitespace and LF.
    raw: &'a [u8],
    /// Complete raw SHA-256, not a reserialization digest.
    raw_sha256: String,
    /// Inert closed typed data; no native or file authority is present.
    document: T,
}
impl<'a, T> DecodedV2<'a, T> {
    /// Borrow complete inert declarations.
    pub(crate) fn document(&self) -> &T {
        &self.document
    }
    /// Borrow the actual original passed to this decoder.
    pub(crate) fn raw(&self) -> &'a [u8] {
        self.raw
    }
    /// Borrow exact complete original byte identity.
    pub(crate) fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
}
/// Quote-aware raw allocation descriptor counts every occurrence before parser growth.
struct RawPlan {
    /// Conservative scalar/key/container occurrences, including duplicate keys.
    nodes: usize,
    /// Full decoded UTF-8 string/key byte extents, with no deduplication credit.
    strings: usize,
}
impl RawPlan {
    /// Reserve64 per actual raw occurrence plus all decoded string/key payload bytes.
    fn reserve(&self, ledger: &mut ContractLedger) -> Result<(), ContractError> {
        let total = self
            .nodes
            .checked_mul(64)
            .and_then(|n| n.checked_add(self.strings))
            .ok_or_else(|| ledger.capacity())?;
        ledger.derived(total)
    }
}
/// Decode a four-digit JSON Unicode escape without allocations or native interpretation.
fn hex4(bytes: &[u8]) -> Result<u32, ContractError> {
    if bytes.len() != 4 {
        return Err(ContractError::Invalid);
    }
    bytes.iter().try_fold(0_u32, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'a'..=b'f' => u32::from(byte - b'a' + 10),
            b'A'..=b'F' => u32::from(byte - b'A' + 10),
            _ => return Err(ContractError::Invalid),
        };
        Ok(value * 16 + digit)
    })
}
/// Inspect a complete JSON string before parser allocation, counting exact decoded bytes.
fn string_extent(
    raw: &[u8],
    at: &mut usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    *at += 1;
    let mut length = 0_usize;
    let mut next_fence = *at + 32 * 1024;
    while *at < raw.len() {
        if *at >= next_fence {
            checkpoint(ledger, control)?;
            next_fence = *at + 32 * 1024;
        }
        let byte = raw[*at];
        *at += 1;
        if byte == b'"' {
            return Ok(length);
        }
        let width = if byte == b'\\' {
            let escaped = *raw.get(*at).ok_or(ContractError::Invalid)?;
            *at += 1;
            match escaped {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => 1,
                b'u' => {
                    let code = hex4(raw.get(*at..*at + 4).ok_or(ContractError::Invalid)?)?;
                    *at += 4;
                    if (0xd800..=0xdbff).contains(&code) {
                        if raw.get(*at..*at + 2) != Some(b"\\u") {
                            return Err(ContractError::Invalid);
                        }
                        *at += 2;
                        let low = hex4(raw.get(*at..*at + 4).ok_or(ContractError::Invalid)?)?;
                        *at += 4;
                        if !(0xdc00..=0xdfff).contains(&low) {
                            return Err(ContractError::Invalid);
                        }
                        4
                    } else if (0xdc00..=0xdfff).contains(&code) {
                        return Err(ContractError::Invalid);
                    } else if code <= 0x7f {
                        1
                    } else if code <= 0x7ff {
                        2
                    } else {
                        3
                    }
                }
                _ => return Err(ContractError::Invalid),
            }
        } else {
            if byte < 0x20 {
                return Err(ContractError::Invalid);
            }
            1
        };
        length = length.checked_add(width).ok_or(ContractError::Capacity)?;
        if length > 65_536 {
            return Err(ContractError::Invalid);
        }
    }
    Err(ContractError::Invalid)
}
/// Complete nonallocating raw scan; actual syntax/duplicate parsing follows separately.
fn raw_plan(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<RawPlan, ContractError> {
    ledger.bytes(raw.len())?;
    let mut plan = RawPlan { nodes: 0, strings: 0 };
    let mut at = 0_usize;
    let mut depth = 0_usize;
    let mut next_fence = 0_usize;
    while at < raw.len() {
        if at >= next_fence {
            checkpoint(ledger, control)?;
            next_fence = at + 32 * 1024;
        }
        match raw[at] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                at += 1;
            }
            b'{' | b'[' => {
                ledger.visits(1)?;
                plan.nodes = plan.nodes.checked_add(1).ok_or_else(|| ledger.capacity())?;
                depth = depth.checked_add(1).ok_or_else(|| ledger.capacity())?;
                if depth > 65 {
                    return Err(ContractError::Invalid);
                }
                at += 1;
            }
            b'}' | b']' => {
                ledger.visits(1)?;
                depth = depth.checked_sub(1).ok_or(ContractError::Invalid)?;
                at += 1;
            }
            b',' | b':' => {
                ledger.visits(1)?;
                at += 1;
            }
            b'"' => {
                if depth > 64 {
                    return Err(ContractError::Invalid);
                }
                ledger.visits(1)?;
                plan.nodes = plan.nodes.checked_add(1).ok_or_else(|| ledger.capacity())?;
                let size = string_extent(raw, &mut at, ledger, control)?;
                plan.strings = plan.strings.checked_add(size).ok_or_else(|| ledger.capacity())?;
            }
            _ => {
                if depth > 64 {
                    return Err(ContractError::Invalid);
                }
                ledger.visits(1)?;
                plan.nodes = plan.nodes.checked_add(1).ok_or_else(|| ledger.capacity())?;
                let start = at;
                while at < raw.len()
                    && !matches!(
                        raw[at],
                        b' ' | b'\t'
                            | b'\r'
                            | b'\n'
                            | b'{'
                            | b'}'
                            | b'['
                            | b']'
                            | b','
                            | b':'
                            | b'"'
                    )
                {
                    if at >= next_fence {
                        checkpoint(ledger, control)?;
                        next_fence = at + 32 * 1024;
                    }
                    at += 1;
                }
                if at == start {
                    return Err(ContractError::Invalid);
                }
            }
        }
    }
    if depth != 0 {
        return Err(ContractError::Invalid);
    }
    Ok(plan)
}
/// Reserve complete typed node/string/key logical storage before `from_value` allocation.
fn typed(value: &Value, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.derived(64)?;
    match value {
        Value::String(value) => ledger.derived(value.len())?,
        Value::Array(rows) => {
            for row in rows {
                typed(row, ledger)?;
            }
        }
        Value::Object(rows) => {
            for (key, row) in rows {
                ledger.bytes(key.len())?;
                ledger.derived(key.len())?;
                typed(row, ledger)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}
/// Admit raw shape before strict parsing; preserve ordinary failures through a same-ledger fence.
fn value(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Value, ContractError> {
    phase(ledger, control, |ledger, control| {
        ledger.bytes(raw.len())?;
        std::str::from_utf8(raw).map_err(|_| ContractError::Invalid)?;
        let plan = phase(ledger, control, |ledger, control| raw_plan(raw, ledger, control))?;
        plan.reserve(ledger)?;
        ledger.bytes(raw.len())?;
        phase(ledger, control, |_, _| {
            crate::json_strict::parse_value(
                raw,
                "review-v2",
                crate::json_strict::Limits { max_depth: 64, max_string_bytes: 65_536 },
            )
            .map_err(|_| ContractError::Invalid)
        })
    })
}
/// Trusted embedded schema parse/compile/validation each has a retained-result postfence.
fn schema(
    value: &Value,
    raw_len: usize,
    text: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let definition = phase(ledger, control, |ledger, control| {
        let plan = raw_plan(text.as_bytes(), ledger, control)?;
        plan.reserve(ledger)?;
        // Fixed compiled-node/operand logical descriptor; internal heap is unmeasured.
        let compiled = plan
            .nodes
            .checked_mul(128)
            .and_then(|n| n.checked_add(plan.strings))
            .ok_or_else(|| ledger.capacity())?;
        ledger.derived(compiled)?;
        ledger.bytes(text.len())?;
        serde_json::from_str::<Value>(text).map_err(|_| ContractError::SchemaDefinition)
    })?;
    let validator = phase(ledger, control, |ledger, _| {
        ledger.bytes(text.len())?;
        jsonschema::validator_for(&definition).map_err(|_| ContractError::SchemaDefinition)
    })?;
    phase(ledger, control, |ledger, _| {
        ledger.bytes(raw_len)?;
        if validator.is_valid(value) { Ok(()) } else { Err(ContractError::Invalid) }
    })
}
/// Generic exact-original decoding; aggregate response counters use the Root forwarding seam.
fn decode<'a, T: DeserializeOwned>(
    raw: &'a [u8],
    cap: usize,
    schema_text: &str,
    response: bool,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV2<'a, T>, ContractError> {
    phase(ledger, control, |ledger, control| {
        if raw.len() > cap {
            return Err(ledger.capacity());
        }
        if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
            return Err(ContractError::Invalid);
        }
        if response {
            ledger.response_registration(raw.len())?;
        }
        let value = value(raw, ledger, control)?;
        if response && value.get("proposed_edit").is_some_and(|edit| !edit.is_null()) {
            return Err(ContractError::UnsupportedEdit);
        }
        schema(&value, raw.len(), schema_text, ledger, control)?;
        typed(&value, ledger)?;
        ledger.bytes(raw.len())?;
        let document = phase(ledger, control, |_, _| {
            serde_json::from_value(value).map_err(|_| ContractError::Invalid)
        })?;
        ledger.bytes(raw.len())?;
        ledger.derived(128)?;
        let raw_sha256 = phase(ledger, control, |_, _| Ok(crate::hashing::sha256_hex(raw)))?;
        Ok(DecodedV2 { raw, raw_sha256, document })
    })
}

/// Decode only forge.review-queue/2; native eligibility and original leases are absent.
pub(crate) fn decode_queue<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV2<'a, QueueDocumentV2>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let decoded = decode(
            raw,
            10_485_760,
            include_str!("../../schemas/forge.review-queue-2.schema.json"),
            false,
            ledger,
            control,
        )?;
        queue(decoded.document(), ledger, control)?;
        Ok(decoded)
    })
}
/// Decode immutable asserted /2 response data; exact queue binding is separate.
pub(crate) fn decode_response<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV2<'a, ResponseDocumentV2>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let decoded = decode(
            raw,
            1_048_576,
            include_str!("../../schemas/forge.review-response-2.schema.json"),
            true,
            ledger,
            control,
        )?;
        response(decoded.document(), ledger, control)?;
        Ok(decoded)
    })
}
/// Decode recorded /2 labels and complete denominators without issuing currentness.
pub(crate) fn decode_dispositions<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV2<'a, DispositionsDocumentV2>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let decoded = decode(
            raw,
            33_554_432,
            include_str!("../../schemas/forge.review-dispositions-2.schema.json"),
            false,
            ledger,
            control,
        )?;
        dispositions(decoded.document(), ledger, control)?;
        Ok(decoded)
    })
}
/// Decode a complete inert private locator; receiver route/native bijections remain required.
pub(crate) fn decode_lifecycle_inputs<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<DecodedV2<'a, LifecycleInputsV1>, ContractError> {
    phase(ledger, control, |ledger, control| {
        let decoded: DecodedV2<'_, LifecycleInputsV1> = decode(
            raw,
            1_048_576,
            include_str!("../../schemas/forge.review-lifecycle-inputs-1.schema.json"),
            false,
            ledger,
            control,
        )?;
        let doc = decoded.document();
        for path in std::iter::once(doc.record.path.as_str())
            .chain(std::iter::once(doc.source.path.as_str()))
            .chain(std::iter::once(doc.source.declared_path.as_str()))
            .chain(
                doc.generated_artifacts
                    .iter()
                    .flat_map(|row| [row.path.as_str(), row.declared_path.as_str()]),
            )
        {
            ledger.visits(1)?;
            ledger.bytes(path.len())?;
            if path.is_empty() || path.len() > 4096 {
                return Err(ContractError::Invalid);
            }
        }
        for pair in doc.generated_artifacts.windows(2) {
            strings(&[&pair[0].declared_path, &pair[1].declared_path], ledger)?;
            if pair[0].declared_path >= pair[1].declared_path {
                return Err(ContractError::Invalid);
            }
        }
        Ok(decoded)
    })
}
/// Same UUID/same complete raw bytes is distinct from semantic JSON equality.
pub(crate) fn original_relation(
    left: &DecodedV2<'_, ResponseDocumentV2>,
    right: &DecodedV2<'_, ResponseDocumentV2>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<OriginalRelation, ContractError> {
    phase(ledger, control, |ledger, _| {
        strings(&[&left.document.response_id, &right.document.response_id], ledger)?;
        if left.document.response_id != right.document.response_id {
            return Ok(OriginalRelation::DifferentIdentity);
        }
        let extent =
            left.raw.len().checked_add(right.raw.len()).ok_or_else(|| ledger.capacity())?;
        ledger.bytes(extent)?;
        Ok(if left.raw == right.raw {
            OriginalRelation::ExactDuplicate
        } else {
            OriginalRelation::IdentityConflict
        })
    })
}
/// Complete canonical hash syntax is plain data validation, not actual-byte verification.
fn hash(value: &str) -> Result<(), ContractError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Canonical review UUID differs from exact parseable generated native UUID text.
fn uuid(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.bytes(value.len())?;
    ledger.derived(100)?;
    super::validate::uuid(value)
}
/// Preserve maintained UTC calendar-second semantics.
fn time(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.bytes(value.len())?;
    ledger.derived(84)?;
    super::validate::time(value)
}
/// Preserve the maintained public ASCII token grammar, separate from native private keys.
fn token(value: &str, maximum: usize) -> Result<(), ContractError> {
    super::validate::token(value, maximum)
}
/// Charge complete compared original UTF-8 extents before inspecting strings.
fn strings(values: &[&str], ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(values.len())?;
    let bytes = values.iter().try_fold(0_usize, |sum, value| {
        sum.checked_add(value.len()).ok_or_else(|| ledger.capacity())
    })?;
    ledger.bytes(bytes)
}
/// Canonical generated ordinal grammar, with no allocation, leading zeros or gaps.
pub(crate) fn generated_ordinal(key: &str) -> Option<usize> {
    let value = key.strip_prefix("lifecycle:generated:")?;
    if value.is_empty()
        || value.len() > 2
        || value.len() > 1 && value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let ordinal = value.parse::<usize>().ok()?;
    (ordinal < 98).then_some(ordinal)
}
/// Full eight-field declared role/profile/native identity correlations.
pub(crate) fn validate_pin(
    pin: &SourcePinV2,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    strings(
        &[
            &pin.artifact_key,
            &pin.raw_sha256,
            &pin.validation_profile,
            pin.schema_identity.as_deref().unwrap_or(""),
            pin.native_root_uuid.as_deref().unwrap_or(""),
        ],
        ledger,
    )?;
    token(&pin.artifact_key, 128)?;
    hash(&pin.raw_sha256)?;
    if pin.byte_length > 10_485_760 {
        return Err(ContractError::Invalid);
    }
    match pin.kind {
        SourceKindV2::LifecycleRecord => {
            if pin.artifact_key != "lifecycle:record"
                || pin.byte_length == 0
                || pin.byte_length > 2_097_152
                || pin.schema_identity.as_deref() != Some("forge.policy-lifecycle/2")
                || pin.validation_profile != "forge.lifecycle-record-intrinsic/1"
                || pin.native_model.is_some()
                || pin.native_root_uuid.is_some()
            {
                return Err(ContractError::Invalid);
            }
        }
        SourceKindV2::OpaqueSource => {
            if pin.artifact_key != "lifecycle:source"
                || pin.schema_identity.is_some()
                || pin.validation_profile != "forge.opaque-source-bytes/1"
                || pin.native_model.is_some()
                || pin.native_root_uuid.is_some()
            {
                return Err(ContractError::Invalid);
            }
        }
        SourceKindV2::GeneratedArtifact => {
            if generated_ordinal(&pin.artifact_key).is_none()
                || pin.byte_length == 0
                || pin.schema_identity.is_some()
                || pin.validation_profile != "forge.lifecycle-generated-identity/1"
                || pin.native_model.is_none()
            {
                return Err(ContractError::Invalid);
            }
            let original = pin.native_root_uuid.as_deref().ok_or(ContractError::Invalid)?;
            if !(32..=45).contains(&original.len()) {
                return Err(ContractError::Invalid);
            }
            uuid::Uuid::parse_str(original).map_err(|_| ContractError::Invalid)?;
        }
    }
    Ok(())
}
/// Complete artifact-key order plus exact one-record/one-source/gap-free generated roster.
pub(crate) fn validate_pins(
    pins: &[SourcePinV2],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    if !(2..=100).contains(&pins.len()) {
        return Err(ContractError::Invalid);
    }
    let mut record = 0_usize;
    let mut source = 0_usize;
    let mut generated = 0_usize;
    let mut seen = [false; 98];
    let mut bytes = 0_u64;
    for pair in pins.windows(2) {
        strings(&[&pair[0].artifact_key, &pair[1].artifact_key], ledger)?;
        if pair[0].artifact_key >= pair[1].artifact_key {
            return Err(ContractError::Invalid);
        }
    }
    for pin in pins {
        ledger.visits(1)?;
        validate_pin(pin, ledger)?;
        bytes = bytes.checked_add(pin.byte_length).ok_or_else(|| ledger.capacity())?;
        match pin.kind {
            SourceKindV2::LifecycleRecord => record += 1,
            SourceKindV2::OpaqueSource => source += 1,
            SourceKindV2::GeneratedArtifact => {
                let ordinal = generated_ordinal(&pin.artifact_key).ok_or(ContractError::Invalid)?;
                if seen[ordinal] {
                    return Err(ContractError::Invalid);
                }
                seen[ordinal] = true;
                generated += 1;
            }
        }
    }
    if bytes > 52_428_800 {
        return Err(ledger.capacity());
    }
    if record != 1
        || source != 1
        || generated + 2 != pins.len()
        || seen.iter().take(generated).any(|present| !*present)
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
    queue: &'a QueueDocumentV2,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<&'a Reviewer, ContractError> {
    scan(ledger, queue.reviewers.len(), 256)?;
    queue.reviewers.iter().find(|row| row.key == key).ok_or(ContractError::Invalid)
}

/// Resolve a declared policy without detached fallback or manufactured defaults.
fn selected_policy<'a>(
    queue: &'a QueueDocumentV2,
    key: &str,
    ledger: &mut ContractLedger,
) -> Result<&'a ReviewPolicy, ContractError> {
    scan(ledger, queue.policies.len(), 256)?;
    queue.policies.iter().find(|row| row.key == key).ok_or(ContractError::Invalid)
}

/// Complete fixed disclaimer; never reinterpret a supplied authenticated claim.
fn disclaimer(value: &str) -> Result<(), ContractError> {
    if value != IDENTITY_DISCLAIMER {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Verify every declared role/reviewer membership and complete role pair count.
fn roster(queue: &QueueDocumentV2, ledger: &mut ContractLedger) -> Result<(), ContractError> {
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
    queue: &QueueDocumentV2,
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
/// Inspect assignments, author exclusion and complete finite candidate graph size.
fn assignments(
    queue: &QueueDocumentV2,
    item: &ReviewItemV2,
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
        uuid(&row.response_id, ledger)?;
        hash(&row.raw_sha256)?;
        token(&row.item_key, 128)?;
        token(&row.reviewer_key, 128)?;
        token(&row.reviewer_role, 128)?;
        time(&row.responded_at, ledger)?;
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
        uuid(&witness.response_id, ledger)?;
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
    uuid(&item.item_id, ledger)?;
    tokens(&item.reason_codes, 128, ledger)?;
    tokens(&item.response_ids, 36, ledger)?;
    tokens(&item.dissent_ids, 36, ledger)?;
    for id in &item.response_ids {
        uuid(id, ledger)?;
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
fn counters(document: &DispositionsDocumentV2) -> Result<(), ContractError> {
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
    document: &DispositionsDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        checkpoint(ledger, control)?;
        strings(&[&document.schema_version, &document.identity_disclaimer], ledger)?;
        disclaimer(&document.identity_disclaimer)?;
        if document.schema_version != "forge.review-dispositions/2" {
            return Err(ContractError::Invalid);
        }
        uuid(&document.queue_id, ledger)?;
        hash(&document.queue_raw_sha256)?;
        time(&document.as_of, ledger)?;
        match (document.currentness, &document.closure_generation) {
            (RecordedCurrentness::Unverified, None) => {}
            (RecordedCurrentness::RecordedCurrent, Some(value)) => hash(value)?,
            _ => return Err(ContractError::Invalid),
        }
        validate_pins(&document.source_pins, ledger)?;
        recorded_responses(&document.responses, ledger)?;
        scan(ledger, document.items.len(), 256)?;
        if document.items.len() != 1
            || document.items.windows(2).any(|pair| pair[0].item_key >= pair[1].item_key)
        {
            return Err(ContractError::Invalid);
        }
        let mut ids = BTreeSet::new();
        for item in &document.items {
            checkpoint(ledger, control)?;
            ledger.derived(64)?;
            if !ids.insert(item.item_id.as_str()) {
                return Err(ContractError::Invalid);
            }
            recorded_item(item, &document.responses, ledger)?;
        }
        ledger.visits(document.items.len())?;
        counters(document)?;
        checkpoint(ledger, control)
    })
}
/// Bind a recorded bundle to exact queue declarations, without trusting raw
/// response pins, claimed source currentness or recorded matching witnesses.
pub(crate) fn bind_dispositions(
    queue: &DecodedV2<'_, QueueDocumentV2>,
    bundle: &DecodedV2<'_, DispositionsDocumentV2>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        checkpoint(ledger, control)?;
        let queue_doc = queue.document();
        let recorded = bundle.document();
        compared_pins(&recorded.source_pins, &queue_doc.source_pins, ledger)?;
        strings(
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
            checkpoint(ledger, control)?;
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
        checkpoint(ledger, control)
    })
}

/// Check a recorded witness against declared eligibility without evaluating a vote.
fn witness_binding(
    queue: &QueueDocumentV2,
    item: &ReviewItemV2,
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

/// Complete response shape correlations; eligibility and prior-chain checks are separate.
pub(crate) fn response(
    response: &ResponseDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        checkpoint(ledger, control)?;
        strings(
            &[&response.schema_version, &response.identity_disclaimer, &response.adapter_version],
            ledger,
        )?;
        disclaimer(&response.identity_disclaimer)?;
        if response.schema_version != "forge.review-response/2" {
            return Err(ContractError::Invalid);
        }
        uuid(&response.response_id, ledger)?;
        uuid(&response.queue_id, ledger)?;
        uuid(&response.item_id, ledger)?;
        time(&response.responded_at, ledger)?;
        validate_pins(&response.source_pins, ledger)?;
        if response.adapter_version != ADAPTER {
            return Err(ContractError::Invalid);
        }
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
                uuid(&prior.response_id, ledger)?;
                hash(&prior.raw_sha256)?;
                if prior.response_id == response.response_id {
                    return Err(ContractError::Invalid);
                }
            }
            (Disposition::Superseded, None) | (_, Some(_)) => return Err(ContractError::Invalid),
            (_, None) => {}
        }
        checkpoint(ledger, control)
    })
}
/// Bind declared response fields to the exact raw queue. No approval, deadline
/// classification, chain retirement, source-currentness or quorum is inferred.
pub(crate) fn bind_response(
    queue: &DecodedV2<'_, QueueDocumentV2>,
    response: &DecodedV2<'_, ResponseDocumentV2>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        checkpoint(ledger, control)?;
        let queue_doc = queue.document();
        let response = response.document();
        scan(ledger, queue_doc.items.len(), 512)?;
        let item = queue_doc
            .items
            .iter()
            .find(|item| item.key == response.item_key)
            .ok_or(ContractError::Binding)?;
        compared_pins(&response.source_pins, &queue_doc.source_pins, ledger)?;
        strings(
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
        checkpoint(ledger, control)
    })
}

/// Precharge every exact typed pin field on both sides before complete equality.
fn compared_pins(
    left: &[SourcePinV2],
    right: &[SourcePinV2],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    for pins in [left, right] {
        ledger.visits(pins.len())?;
        for pin in pins {
            strings(
                &[
                    &pin.artifact_key,
                    &pin.raw_sha256,
                    &pin.validation_profile,
                    pin.schema_identity.as_deref().unwrap_or(""),
                    pin.native_root_uuid.as_deref().unwrap_or(""),
                ],
                ledger,
            )?;
            ledger.bytes(64)?;
        }
    }
    Ok(())
}
/// Full source-key roster in public pin order; no caller-selected subset is accepted.
fn source_keys(
    queue: &QueueDocumentV2,
    item: &ReviewItemV2,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    tokens(&item.source_keys, 128, ledger)?;
    if item.source_keys.len() != queue.source_pins.len() {
        return Err(ContractError::Invalid);
    }
    for (key, pin) in item.source_keys.iter().zip(&queue.source_pins) {
        strings(&[key, &pin.artifact_key], ledger)?;
        if *key != pin.artifact_key {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}
/// Complete asserted policy/author/assignment/deadline/disposition hash operands.
pub(crate) fn validate_policy_operands(
    queue: &QueueDocumentV2,
    selected: &ReviewPolicy,
    item: &ReviewItemV2,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    roster(queue, ledger)?;
    policy(queue, selected, ledger)?;
    assignments(queue, item, selected, ledger)?;
    strings(&[&selected.key, &item.policy_key], ledger)?;
    if selected.key != item.policy_key {
        return Err(ContractError::Invalid);
    }
    if let Some(due) = &item.due_at {
        time(due, ledger)?;
        strings(&[due, &queue.created_at], ledger)?;
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
    ledger.visits(item.allowed_dispositions.len())?;
    if item.allowed_dispositions != allowed {
        return Err(ContractError::Invalid);
    }
    Ok(())
}
/// Complete single selected item and all separately framed public binding correlations.
pub(crate) fn queue(
    queue: &QueueDocumentV2,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    phase(ledger, control, |ledger, control| {
        strings(&[&queue.schema_version, &queue.identity_disclaimer], ledger)?;
        disclaimer(&queue.identity_disclaimer)?;
        if queue.schema_version != "forge.review-queue/2"
            || queue.items.len() != 1
            || queue.policies.len() != 1
        {
            return Err(ContractError::Invalid);
        }
        uuid(&queue.queue_id, ledger)?;
        time(&queue.created_at, ledger)?;
        validate_pins(&queue.source_pins, ledger)?;
        let item = &queue.items[0];
        let selected = &queue.policies[0];
        strings(
            &[
                &item.key,
                &item.adapter_version,
                &item.subject_id,
                &item.subject_sha256,
                &item.context_sha256,
                &item.policy_sha256,
            ],
            ledger,
        )?;
        token(&item.key, 128)?;
        uuid(&item.item_id, ledger)?;
        if item.adapter_version != ADAPTER
            || item.subject_id.len() != 74
            || !item.subject_id.starts_with("lifecycle:")
        {
            return Err(ContractError::Invalid);
        }
        hash(&item.subject_id[10..])?;
        hash(&item.subject_sha256)?;
        hash(&item.context_sha256)?;
        hash(&item.policy_sha256)?;
        source_keys(queue, item, ledger)?;
        validate_policy_operands(queue, selected, item, ledger)?;
        ledger.visits(item.context.reason_codes.len())?;
        for reason in &item.context.reason_codes {
            ledger.bytes(reason.len())?;
        }
        if item.context.reason_codes.len() != 1
            || item.context.reason_codes[0] != "lifecycle-approved-current"
            || !item.context.related_subject_ids.is_empty()
        {
            return Err(ContractError::Invalid);
        }
        let context = super::hash_v2::context(queue, item, ledger, control)?;
        let policy = super::hash_v2::policy(queue, selected, item, ledger, control)?;
        let identity = super::hash_v2::item_id(queue, item, ledger, control)?;
        strings(
            &[
                &context,
                &item.context_sha256,
                &policy,
                &item.policy_sha256,
                &identity,
                &item.item_id,
            ],
            ledger,
        )?;
        if context != item.context_sha256
            || policy != item.policy_sha256
            || identity != item.item_id
        {
            return Err(ContractError::Binding);
        }
        Ok(())
    })
}

/// Decode only the closed asserted private Lifecycle init request; no native authority.
/// Its actual Auxiliary allocation is bound by the separate genuine owner consumer.
pub(crate) fn decode_lifecycle_init(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<super::lifecycle_queue_export::LifecycleInitRequest, ContractError> {
    phase(ledger, control, |ledger, control| {
        let decoded: DecodedV2<'_, super::lifecycle_queue_export::LifecycleInitRequest> = decode(
            raw,
            1_048_576,
            include_str!("../../schemas/forge.review-lifecycle-init-1.schema.json"),
            false,
            ledger,
            control,
        )?;
        // Move fully admitted ordinary typed declarations; do not clone/reparse them
        // or expose a generic document-success/currentness constructor.
        Ok(decoded.document)
    })
}

/// Plain admitted JSON primitives shared by separately closed later codecs.
/// They do not construct a family envelope or any native/capture/currentness owner.
pub(crate) mod structural {
    use super::{ContractError, ContractLedger, Value, phase};
    use crate::workspace::preparation::WorkControl;

    /// Bound raw input, reserve the complete duplicate-aware tree, and validate a
    /// caller-selected fixed schema before reserving complete typed conversion.
    /// The returned Value is inert; its consumer still owns family semantics.
    pub(crate) fn admitted_value(
        raw: &[u8],
        cap: usize,
        schema_text: &str,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<Value, ContractError> {
        phase(ledger, control, |ledger, control| {
            if cap == 0 || cap > 33_554_432 {
                return Err(ContractError::Invalid);
            }
            if raw.len() > cap {
                return Err(ledger.capacity());
            }
            ledger.bytes(raw.len().min(3))?;
            if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
                return Err(ContractError::Invalid);
            }
            let value = super::value(raw, ledger, control)?;
            super::schema(&value, raw.len(), schema_text, ledger, control)?;
            phase(ledger, control, |ledger, _| super::typed(&value, ledger))?;
            Ok(value)
        })
    }
}

/// Plain original-byte intake forwarding; no envelope or native authority is issued.
/// All old /1-/2 bodies above remain byte-exact.
pub(crate) mod operational_raw {
    use super::{ContractError, ContractLedger, DeserializeOwned, WorkControl, phase};

    /// Preserve cap/empty/BOM preflight, response accounting and edit-before-schema order.
    /// Return only owned inert data and an actual original-byte digest, without a /2 cast.
    pub(crate) fn admitted_document<T: DeserializeOwned>(
        raw: &[u8],
        cap: usize,
        schema_text: &str,
        response: bool,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(T, String), ContractError> {
        phase(ledger, control, |ledger, control| {
            if cap == 0 || cap > 33_554_432 {
                return Err(ContractError::Invalid);
            }
            if raw.len() > cap {
                return Err(ledger.capacity());
            }
            ledger.bytes(raw.len().min(3))?;
            if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
                return Err(ContractError::Invalid);
            }
            if response {
                ledger.response_registration(raw.len())?;
            }
            let value = super::value(raw, ledger, control)?;
            ledger.bytes(raw.len())?;
            ledger.visits(1)?;
            if response && value.get("proposed_edit").is_some_and(|edit| !edit.is_null()) {
                return Err(ContractError::UnsupportedEdit);
            }
            super::schema(&value, raw.len(), schema_text, ledger, control)?;
            phase(ledger, control, |ledger, _| super::typed(&value, ledger))?;
            ledger.bytes(raw.len())?;
            let document = phase(ledger, control, |_, _| {
                serde_json::from_value(value).map_err(|_| ContractError::Invalid)
            })?;
            ledger.bytes(raw.len())?;
            ledger.derived(128)?;
            let digest = phase(ledger, control, |_, _| Ok(crate::hashing::sha256_hex(raw)))?;
            Ok((document, digest))
        })
    }
}

/// Shared inert recorded-row checks, independent of any versioned document or source pin.
pub(crate) mod recorded_data {
    use super::{
        ContractError, ContractLedger, ItemDisposition, RecordedResponse, WorkControl, checkpoint,
        phase,
    };

    /// Retain complete recorded original metadata, order and unique-byte denominator.
    pub(crate) fn rows(
        rows: &[RecordedResponse],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            // Full actual identity extents cover ordered comparisons before canonical grammar.
            for row in rows {
                checkpoint(ledger, control)?;
                ledger.visits(1)?;
                let identity_work =
                    row.response_id.len().checked_mul(2).ok_or_else(|| ledger.capacity())?;
                ledger.bytes(identity_work)?;
            }
            super::recorded_responses(rows, ledger)
        })
    }
    /// Check all response/dissent/witness correlations without matching or policy authority.
    /// Admit a full conservative pair bound before the borrowed seat/reviewer trees grow.
    pub(crate) fn item(
        item: &ItemDisposition,
        rows: &[RecordedResponse],
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, _| {
            let seats = item
                .met_seats
                .len()
                .checked_add(item.unmet_seats.len())
                .ok_or_else(|| ledger.capacity())?;
            if seats > 100 {
                return Err(ContractError::Invalid);
            }
            let pairs = seats.checked_mul(seats).ok_or_else(|| ledger.capacity())?;
            ledger.visits(pairs)?;
            let pair_work = pairs.checked_mul(512).ok_or_else(|| ledger.capacity())?;
            ledger.bytes(pair_work)?;
            super::recorded_item(item, rows, ledger)
        })
    }
}

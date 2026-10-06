//! Complete native phase admission over real borrowed originals.
//! Measurements and descriptors are private ordinary data. The receiver supplies
//! every byte; this helper never captures, seals or issues approved/current authority.
//! Native parse/validation internals are synchronous: logical before-growth budgets
//! and cooperative fences do not establish heap confinement or syscall preemption.

use std::io::Write;

use serde::Serialize;
use serde_json::Value;

use crate::lifecycle::record::{self, LifecycleRecord};
use crate::lifecycle::status::{CurrentArtifacts, StatusReport, status_from_captured};
use crate::workspace::preparation::WorkControl;

use super::super::decode::{ContractError, ContractLedger};

/// Complete measured tree operands retained only until the corresponding native phase.
#[derive(Default)]
struct TreePlan {
    /// All decoded Value nodes, including scalar/empty/null rows.
    nodes: usize,
    /// Every actual array/object entry, including object property names.
    entries: usize,
    /// Full actual UTF-8 string and property-name payload.
    strings: usize,
    /// Complete longest actual comparison/diagnostic string operand.
    longest: usize,
    /// Complete native bounds-walker diagnostic path payload, measured without copies.
    paths: usize,
}

/// Private conservative intrinsic-validation plan from the complete actual decoded record.
pub(super) struct NativePlan {
    /// Complete native validation registry/comparison/projection visits.
    visits: usize,
    /// Complete comparison, native seed serialization/hash and private string work.
    bytes: usize,
    /// Logical native registry, seed buffers, error/path scratch and event-ID storage.
    logical: usize,
}

/// Overflow-safe complete sum; the actual caller bound latches this capacity refusal.
fn add(a: usize, b: usize) -> Result<usize, ContractError> {
    a.checked_add(b).ok_or(ContractError::Capacity)
}
/// Overflow-safe complete product; no count is truncated or saturated into success.
fn mul(a: usize, b: usize) -> Result<usize, ContractError> {
    a.checked_mul(b).ok_or(ContractError::Capacity)
}

/// Run an ordinary phase result through the actual original post-failure fence.
fn phase<T>(
    result: Result<T, ContractError>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<T, ContractError> {
    // bound returns an already latched first stop before calling the later checkpoint.
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        result
    })
}

/// Quote-aware raw topology and whole byte-work admission before any Value allocation.
fn raw_plan(
    raw: &[u8],
    maximum: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    ledger.checkpoint(control)?;
    if raw.len() > maximum {
        return Err(ledger.capacity());
    }
    if raw.is_empty() {
        return Err(ContractError::Invalid);
    }
    if raw.starts_with(b"\xef\xbb\xbf") {
        return Err(ContractError::Invalid);
    }
    let mut quote = false;
    let mut escaped = false;
    let mut depth = 0_usize;
    let mut nodes = 1_usize;
    let mut string_extent = 0_usize;
    for chunk in raw.chunks(32 * 1024) {
        ledger.checkpoint(control)?;
        ledger.bytes(chunk.len())?;
        ledger.visits(1)?;
        for byte in chunk {
            if quote {
                string_extent = add(string_extent, 1)?;
                if string_extent > 6 * 65_536 + 1 {
                    return Err(ContractError::Invalid);
                }
                if escaped {
                    escaped = false;
                } else if *byte == b'\\' {
                    escaped = true;
                } else if *byte == b'"' {
                    quote = false;
                }
            } else {
                match byte {
                    b'"' => {
                        quote = true;
                        string_extent = 0;
                    }
                    b'{' | b'[' => {
                        depth = add(depth, 1)?;
                        nodes = add(nodes, 1)?;
                        if depth > 64 {
                            return Err(ContractError::Invalid);
                        }
                    }
                    b'}' | b']' => {
                        depth = depth.saturating_sub(1);
                    }
                    b',' | b':' => {
                        nodes = add(nodes, 1)?;
                    }
                    _ => {}
                }
            }
        }
    }
    ledger.bytes(raw.len())?;
    if std::str::from_utf8(raw).is_err() {
        return Err(ContractError::Invalid);
    }
    // For valid JSON each actual value/property requires an opening separator or
    // frame already counted above. String decoding cannot exceed its raw UTF-8 extent.
    // Value + String + property pair + three links is the declared logical tree slot,
    // not an allocator-size claim. A second raw payload allows native strict key copies.
    let unit = add(
        std::mem::size_of::<Value>(),
        add(
            std::mem::size_of::<String>(),
            add(std::mem::size_of::<(String, Value)>(), mul(3, std::mem::size_of::<usize>())?)?,
        )?,
    )?;
    ledger.derived(add(mul(nodes, unit)?, mul(raw.len(), 2)?)?)?;
    ledger.checkpoint(control)?;
    Ok(nodes)
}

/// Admit and inspect the whole actual generated/record Value without changing duplicate semantics.
fn observed_value(
    raw: &[u8],
    maximum: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(Value, TreePlan), ContractError> {
    ledger.bound(|ledger| {
        let scan = raw_plan(raw, maximum, ledger, control);
        phase(scan, ledger, control)?;
        ledger.bytes(raw.len())?;
        let decoded = serde_json::from_slice::<Value>(raw).map_err(|_| ContractError::Invalid);
        let value = phase(decoded, ledger, control)?;
        let mut plan = TreePlan::default();
        let inspected = inspect(&value, 0, 1, &mut plan, ledger, control);
        phase(inspected, ledger, control)?;
        Ok((value, plan))
    })
}

/// Meter every actual node/key/string before later private typed/native growth.
fn inspect(
    value: &Value,
    depth: usize,
    path_bytes: usize,
    plan: &mut TreePlan,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    plan.nodes = add(plan.nodes, 1)?;
    plan.paths = add(plan.paths, path_bytes)?;
    if plan.nodes > 100_000 {
        return Err(ledger.capacity());
    }
    if depth > 64 {
        return Err(ContractError::Invalid);
    }
    match value {
        Value::String(text) => text_plan(text, plan, ledger),
        Value::Array(rows) => {
            plan.entries = add(plan.entries, rows.len())?;
            for (index, row) in rows.iter().enumerate() {
                inspect(
                    row,
                    depth + 1,
                    add(path_bytes, add(decimal_width(index), 2)?)?,
                    plan,
                    ledger,
                    control,
                )?;
            }
            Ok(())
        }
        Value::Object(rows) => {
            plan.entries = add(plan.entries, rows.len())?;
            for (key, row) in rows {
                text_plan(key, plan, ledger)?;
                inspect(
                    row,
                    depth + 1,
                    add(path_bytes, add(key.len(), 1)?)?,
                    plan,
                    ledger,
                    control,
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
/// Measure a native diagnostic array index without formatting or allocating text.
fn decimal_width(mut value: usize) -> usize {
    let mut width = 1;
    while value >= 10 {
        value /= 10;
        width += 1;
    }
    width
}
/// Measure full private strings without normalization or path/key interpretation.
fn text_plan(
    text: &str,
    plan: &mut TreePlan,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.bytes(text.len())?;
    if text.len() > 65_536 {
        return Err(ContractError::Invalid);
    }
    plan.strings = add(plan.strings, text.len())?;
    plan.longest = plan.longest.max(text.len());
    Ok(())
}

/// Exact compact serializer byte counter; retains no output or detached source payload.
struct Count<'a> {
    /// Complete serialized count, including every actual escaped scalar/property.
    bytes: usize,
    /// Caller original ledger; an IO wrapper preserves its actual first failure.
    ledger: &'a mut ContractLedger,
    /// Caller original control; no deadline is reset.
    control: &'a mut dyn WorkControl,
}
impl Write for Count<'_> {
    /// Charge each complete emitted chunk before measuring it.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let result = self.ledger.bound(|ledger| {
            ledger.checkpoint(self.control)?;
            ledger.bytes(bytes.len())?;
            ledger.visits(1)?;
            self.bytes = add(self.bytes, bytes.len())?;
            Ok(())
        });
        result.map_err(|_| std::io::Error::other("private lifecycle measurement failed"))?;
        Ok(bytes.len())
    }
    /// No retained sink or external flush exists.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Measure complete actual native subtree encoding before native seed growth/hash work.
fn encoded<T: Serialize>(
    value: &T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let mut sink = Count { bytes: 0, ledger, control };
        let result = serde_json::to_writer(&mut sink, value).map_err(|_| sink.ledger.failure());
        let count = sink.bytes;
        phase(result, sink.ledger, sink.control)?;
        Ok(count)
    })
}
/// Count an actual array declaration, including malformed rows later rejected by native parse.
fn count(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}

/// Derive complete intrinsic validation work from actual full native declarations.
fn validation_plan(
    value: &Value,
    tree: &TreePlan,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<NativePlan, ContractError> {
    ledger.checkpoint(control)?;
    let policy = value.get("policy");
    let generated = count(policy.and_then(|p| p.get("generated_artifacts")));
    let parties = count(value.get("parties"));
    let owners = count(policy.and_then(|p| p.get("owner_keys")));
    let requirements = count(value.get("approval_policy").and_then(|p| p.get("required_roles")));
    let history = value.get("history").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let mut assertions = 0_usize;
    let mut hash_rows = 0_usize;
    let mut impact_rows = 0_usize;
    let mut approval_probes = 0_usize;
    let mut seeds = 0_usize;
    let mut context = 0_usize;
    for key in ["schema_version", "policy", "parties", "approval_policy", "review"] {
        ledger.visits(1)?;
        if let Some(row) = value.get(key) {
            context = add(context, encoded(row, ledger, control)?)?;
        }
    }
    for (index, event) in history.iter().enumerate() {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        assertions = add(assertions, count(event.get("assertions")))?;
        hash_rows = add(
            hash_rows,
            count(event.get("fingerprints").and_then(|p| p.get("generated_artifacts"))),
        )?;
        impact_rows = add(impact_rows, count(event.get("impact_finding_ids")))?;
        let event_bytes = encoded(event, ledger, control)?;
        // Native current and optional legacy seeds select complete context/event
        // subsets. This literal covers all outer field frames and the four optional
        // fields serialized by seed structs even when ordinary record serde omits them.
        let seed_frames = br#"{"schema_version":null,"policy":null,"parties":null,"approval_policy":null,"review":null,"legacy_event_id":null,"assertions":[],"impact_finding_ids":[],"replacement":null}"#.len();
        // At most two native seed Vecs/UUIDv5 operations occur per declared event.
        seeds = add(seeds, mul(2, add(add(context, event_bytes)?, seed_frames)?)?)?;
        if event.get("next_state").and_then(Value::as_str) == Some("approved") {
            // Native approval scans a preceding window and builds actor-role sets.
            // Every actual preceding event is admitted conservatively, including a
            // malformed missing review boundary which the native validator refuses.
            approval_probes = add(approval_probes, index + 1)?;
            for prior in &history[..=index] {
                ledger.visits(1)?;
                approval_probes = add(approval_probes, count(prior.get("assertions")))?;
                approval_probes = add(
                    approval_probes,
                    count(prior.get("fingerprints").and_then(|p| p.get("generated_artifacts"))),
                )?;
            }
        }
    }
    // Full cross-operand registry allowance dominates native BTree comparisons
    // and actor lookups without assuming a particular allocator/tree implementation.
    let registry = add(add(generated, parties)?, add(owners, requirements)?)?;
    let actor_rows = add(history.len(), assertions)?;
    let comparisons = add(
        mul(registry, registry)?,
        add(
            mul(actor_rows, parties)?,
            add(mul(approval_probes, add(actor_rows, hash_rows)?)?, add(hash_rows, impact_rows)?)?,
        )?,
    )?;
    let visits = add(mul(tree.nodes, 16)?, add(comparisons, add(actor_rows, approval_probes)?)?)?;
    let bytes = add(
        add(seeds, mul(history.len(), 2 * 16)?)?,
        add(
            add(mul(tree.strings, 16)?, mul(tree.paths, 2)?)?,
            mul(comparisons, mul(tree.longest, 2)?)?,
        )?,
    )?;
    let row_size = [
        std::mem::size_of::<record::TransitionEvent>(),
        std::mem::size_of::<record::ArtifactFingerprint>(),
        std::mem::size_of::<record::Party>(),
        std::mem::size_of::<record::ActorAssertion>(),
        std::mem::size_of::<record::NamedHash>(),
        std::mem::size_of::<record::PolicyReference>(),
        std::mem::size_of::<record::LifecycleRecord>(),
    ]
    .into_iter()
    .max()
    .ok_or(ContractError::Capacity)?;
    let logical = add(
        add(seeds, mul(tree.paths, 2)?)?,
        add(
            mul(tree.nodes, row_size)?,
            add(
                mul(tree.strings, 2)?,
                mul(
                    add(registry, add(actor_rows, approval_probes)?)?,
                    add(
                        std::mem::size_of::<(&str, record::DeclaredRole)>(),
                        mul(3, std::mem::size_of::<usize>())?,
                    )?,
                )?,
            )?,
        )?,
    )?;
    Ok(NativePlan { visits, bytes, logical })
}
impl NativePlan {
    /// Admit all repeated native validation/registry/hash work before invoking it again.
    fn admit(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.checkpoint(control)?;
        ledger.visits(self.visits)?;
        ledger.bytes(self.bytes)?;
        ledger.derived(self.logical)
    }
}

/// Preserve maintained intrinsic duplicate/error semantics after complete pre-growth admission.
pub(super) fn record(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(LifecycleRecord, NativePlan), ContractError> {
    ledger.bound(|ledger| {
        let maximum =
            usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| ContractError::Capacity)?;
        let (value, tree) = observed_value(raw, maximum, ledger, control)?;
        let plan = validation_plan(&value, &tree, ledger, control)?;
        drop(value);
        let scan = raw_plan(raw, maximum, ledger, control);
        phase(scan, ledger, control)?;
        plan.admit(ledger, control)?;
        ledger.bytes(raw.len())?;
        let parsed = record::parse(raw).map_err(|_| ContractError::Invalid);
        let record = phase(parsed, ledger, control)?;
        Ok((record, plan))
    })
}

/// Preserve maintained generated serde/model/root/UUID text semantics, including duplicates.
pub(super) fn identity(
    path: &std::path::Path,
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(Option<String>, Option<String>), ContractError> {
    ledger.bound(|ledger| {
        let (value, tree) = observed_value(raw, 10 * 1024 * 1024, ledger, control)?;
        drop(value);
        let scan = raw_plan(raw, 10 * 1024 * 1024, ledger, control);
        phase(scan, ledger, control)?;
        ledger.visits(add(tree.nodes, 6)?)?;
        ledger.bytes(add(tree.strings, path.as_os_str().as_encoded_bytes().len())?)?;
        ledger.derived(add(tree.strings, mul(2, std::mem::size_of::<String>())?)?)?;
        ledger.bytes(raw.len())?;
        let actual = crate::lifecycle::artifact_identity_from_bytes(path, raw, true)
            .map_err(|_| ContractError::Binding);
        phase(actual, ledger, control)
    })
}

/// Use the full maintained neutral status, including another intrinsic validation.
pub(super) fn status(
    record: &LifecycleRecord,
    current: &CurrentArtifacts,
    plan: &NativePlan,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<StatusReport, ContractError> {
    ledger.bound(|ledger| {
        plan.admit(ledger, control)?;
        let record_bytes = encoded(record, ledger, control)?;
        let current_bytes = encoded(&current.fingerprints, ledger, control)?;
        // Complete status copies: record selected fields/history/replacement, current
        // and latest-approved fingerprints, and unique sorted carried impact IDs.
        ledger.derived(add(
            std::mem::size_of::<StatusReport>(),
            add(record_bytes, mul(current_bytes, 2)?)?,
        )?)?;
        ledger.bytes(add(record_bytes, mul(current_bytes, 2)?)?)?;
        ledger
            .visits(add(record.history.len(), add(record.policy.generated_artifacts.len(), 1)?)?)?;
        let actual =
            status_from_captured(record, current, None).map_err(|_| ContractError::Binding);
        phase(actual, ledger, control)
    })
}

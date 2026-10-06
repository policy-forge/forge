//! Complete ordinary native operand admission over the one held /2 owner.
//!
//! No helper in this module creates capture, approval or currentness. Retained logical
//! descriptors are conservative source/string/row envelopes, not measured heap or CPU.
//! Native library phases remain cooperative before/after calls, never preempted midcall.

use serde_json::Value;

use super::capture_v2::{Admission, AdmittedValue};
use super::declarations_v2::DeclarationAdmission;
use super::{MAX_STRING_BYTES, Role, policy_failure, strict_value};
use crate::lifecycle::record::{self, ParsedLifecycleRecord};
use crate::lifecycle::status::{self, CurrentArtifacts, StatusReport};
use crate::workspace::preparation::{Stage, WorkControl, WorkResult};

/// Complete borrowed byte pass granularity, matching the shared declaration codec.
const WORK_BYTES: usize = 32 * 1024;
/// Native/source JSON is never admitted beyond the existing ten-MiB raw ceiling.
const RAW_LIMIT: usize = 10 * 1024 * 1024;

/// Actual complete parsed operand descriptor, retaining no detached source authority.
#[derive(Clone, Copy, Default)]
pub(super) struct Operand {
    /// Every actual Value node, including scalars and containers.
    pub(super) nodes: usize,
    /// Every complete actual object field, including private native metadata.
    pub(super) fields: usize,
    /// Exact UTF-8 bytes of every actual key and string occurrence.
    pub(super) strings: usize,
    /// Complete measured traversal units used again before downstream phases.
    pub(super) work: usize,
}

/// Actual private observations computed from one already held original.
pub(super) struct MemberFacts {
    /// Strict complete actual native/domain tree, or genuine ordinary parse failure.
    pub(super) value: Option<Value>,
    /// Actual maintained full native identity observation, never a supplied verdict.
    pub(super) identity: Option<super::NativeIdentity>,
    /// Actual full intrinsically validated lifecycle record, or ordinary invalidity.
    pub(super) record: Option<ParsedLifecycleRecord>,
    /// Complete observed tree descriptor before repeated native/domain work.
    pub(super) operand: Operand,
}

/// Retain the entire ordinary phase result until the same original owner's post-fence.
pub(super) fn run<'control, C: WorkControl + ?Sized, T>(
    admission: &mut Admission<'control, C>,
    amount: usize,
    stage: Stage,
    ordinary: impl FnOnce(&mut Admission<'control, C>) -> WorkResult<T>,
) -> WorkResult<T> {
    fence(admission, stage)?;
    let admitted = admission.charge(amount);
    admission.phase(admitted, stage)?;
    let result = ordinary(admission);
    admission.phase(result, stage)
}

/// Use the codec checkpoint contract explicitly; native `WorkControl` shares this same owner.
pub(super) fn fence<C: WorkControl + ?Sized>(
    admission: &mut Admission<'_, C>,
    stage: Stage,
) -> WorkResult<()> {
    DeclarationAdmission::checkpoint(admission, stage)
}

/// Preserve checked descriptor overflow as the original owner's first capacity cause.
pub(super) fn add<C: WorkControl + ?Sized>(
    left: usize,
    right: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    left.checked_add(right).ok_or_else(|| admission.capacity())
}

/// Preserve complete multiplication before any corresponding work or growth.
pub(super) fn multiply<C: WorkControl + ?Sized>(
    left: usize,
    right: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    left.checked_mul(right).ok_or_else(|| admission.capacity())
}

/// Borrowed byte units do not allocate a rounded byte buffer.
pub(super) fn byte_work(bytes: usize) -> usize {
    bytes.div_ceil(WORK_BYTES)
}

/// Complete raw lexical envelope admitted before any strict native Value construction.
/// This does not validate JSON or confer a native proof; the maintained parser does both
/// syntax/duplicate checks. Conservative escaped-code-unit bounds can refuse earlier.
pub(super) fn raw_bound<C: WorkControl + ?Sized>(
    raw: &[u8],
    limit: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<(usize, usize)> {
    run(admission, 1, Stage::ValidateResource, |admission| {
        if raw.len() > limit { Err(admission.capacity()) } else { Ok(()) }
    })?;
    run(admission, byte_work(raw.len()) + 1, Stage::ValidateResource, |admission| {
        let mut scan = Lexical::default();
        for chunk in raw.chunks(WORK_BYTES) {
            fence(admission, Stage::ValidateResource)?;
            for byte in chunk {
                scan.visit(*byte, admission)?;
            }
            fence(admission, Stage::ValidateResource)?;
        }
        let nodes = add(scan.separators, 1, admission)?;
        let node_bytes = multiply(nodes, 512, admission)?;
        let strings = multiply(raw.len(), 8, admission)?;
        let logical = add(add(node_bytes, strings, admission)?, 4096, admission)?;
        Ok((nodes, logical))
    })
}

/// Complete actual tree accounting before repeated semantics or owned registry growth.
pub(super) fn operand<C: WorkControl + ?Sized>(
    value: &Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Operand> {
    fence(admission, Stage::ValidateResource)?;
    let result = operand_inner(value, admission);
    admission.phase(result, Stage::ValidateResource)
}

/// Charge every inspected actual occurrence before recursing or accumulating it.
fn operand_inner<C: WorkControl + ?Sized>(
    value: &Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Operand> {
    admission.charge(1)?;
    let mut result = Operand { nodes: 1, work: 1, ..Operand::default() };
    match value {
        Value::Array(values) => {
            for child in values {
                result = sum(result, operand_inner(child, admission)?, admission)?;
            }
        }
        Value::Object(fields) => {
            admission.charge(fields.len())?;
            result.fields = fields.len();
            result.work = add(result.work, fields.len(), admission)?;
            for (key, child) in fields {
                admission.charge(byte_work(key.len()))?;
                result.strings = add(result.strings, key.len(), admission)?;
                result.work = add(result.work, byte_work(key.len()), admission)?;
                result = sum(result, operand_inner(child, admission)?, admission)?;
            }
        }
        Value::String(text) => {
            admission.charge(byte_work(text.len()))?;
            result.strings = text.len();
            result.work = add(result.work, byte_work(text.len()), admission)?;
        }
        _ => {}
    }
    Ok(result)
}

/// Combine full logical descriptors without truncating the retained denominator.
fn sum<C: WorkControl + ?Sized>(
    left: Operand,
    right: Operand,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Operand> {
    Ok(Operand {
        nodes: add(left.nodes, right.nodes, admission)?,
        fields: add(left.fields, right.fields, admission)?,
        strings: add(left.strings, right.strings, admission)?,
        work: add(left.work, right.work, admission)?,
    })
}

/// Pre-admit actual strict parse, retained tree, maintained full native identity and record.
/// Ordinary invalid/unsupported observations remain data; actual owner stops cannot be hidden.
pub(super) fn prepare_member<C: WorkControl + ?Sized>(
    raw: &[u8],
    role: Role,
    admission: &mut Admission<'_, C>,
) -> WorkResult<AdmittedValue<MemberFacts>> {
    let limit = if role == Role::LifecycleRecord {
        usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| admission.capacity())?
    } else {
        RAW_LIMIT
    };
    let (tree_upper, logical) = raw_bound(raw, limit, admission)?;
    admission.retain(logical, |admission| {
        let value =
            run(admission, tree_upper + byte_work(raw.len()), Stage::ValidateResource, |_| {
                Ok(strict_value(raw, limit).ok())
            })?;
        let measured =
            value.as_ref().map(|value| operand(value, admission)).transpose()?.unwrap_or_default();
        let identity = if role.native_model().is_some() {
            value
                .as_ref()
                .map(|value| native_identity(value, role, measured, admission))
                .transpose()?
                .flatten()
        } else {
            None
        };
        let record_work = if role == Role::LifecycleRecord {
            value.as_ref().map(|value| lifecycle_work(value, admission)).transpose()?.unwrap_or(0)
        } else {
            0
        };
        let record =
            if role == Role::LifecycleRecord && value.is_some() {
                run(admission, record_work, Stage::PrepareDomain, |_| {
                    Ok(record::parse_owned(raw).ok())
                })?
            } else {
                None
            };
        Ok(MemberFacts { value, identity, record, operand: measured })
    })
}

/// Run genuine maintained schema/version/semantic validation on the already held strict tree.
/// Schema-invalid inputs are rejected by the same validator predicate before detailed errors
/// can grow. Supported/schema-valid inputs still run the maintained complete native report.
/// /1's original `native_facts` implementation and ordinary result predicate remain unchanged.
fn native_identity<C: WorkControl + ?Sized>(
    value: &Value,
    role: Role,
    measured: Operand,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Option<super::NativeIdentity>> {
    let expected = role.native_model().ok_or_else(policy_failure)?;
    admit_native_schema(role, admission)?;
    let preliminary = run(admission, measured.work + 1, Stage::ValidateResource, |_| {
        if crate::validate::detect_model_type(value).ok() != Some(expected) {
            return Ok(false);
        }
        let Some(version) = value
            .get(expected.as_str())
            .and_then(|root| root.get("metadata"))
            .and_then(|metadata| metadata.get("oscal-version"))
            .and_then(Value::as_str)
        else {
            return Ok(false);
        };
        Ok(crate::validate::version::is_supported_oscal_version(version)
            && crate::validate::compiled_validator(expected)
                .is_ok_and(|validator| validator.is_valid(value)))
    })?;
    if !preliminary {
        return Ok(None);
    }
    let path = path_upper(value, 1, admission)?;
    // Native semantic traversal emits at most one orphaned-link error per object and one
    // control-id error per requirement: <=2*nodes. Each path is <=path, fixed message/row
    // storage <=1024; repeated href/control-id/UUID strings consume <=8*all strings.
    let rows = multiply(measured.nodes, 2, admission)?;
    let per_row = add(path, 1024, admission)?;
    let row_bytes = multiply(rows, per_row, admission)?;
    let strings = multiply(measured.strings, 8, admission)?;
    let scratch = add(add(row_bytes, strings, admission)?, add(path, 8192, admission)?, admission)?;
    let complete = multiply(measured.work, 8, admission)?;
    let validated = admission.retain(scratch, |admission| {
        run(admission, complete, Stage::ValidateResource, |_| {
            if !crate::validate::run_full_validation("captured MCP artifact", value, expected)
                .is_ok_and(|report| report.is_valid())
            {
                return Ok(None);
            }
            let Some(root) = value.get(expected.as_str()) else {
                return Ok(None);
            };
            let Some(metadata) = root.get("metadata") else {
                return Ok(None);
            };
            let (Some(root_id), Some(document_version), Some(oscal_version)) = (
                root.get("uuid").and_then(Value::as_str),
                metadata.get("version").and_then(Value::as_str),
                metadata.get("oscal-version").and_then(Value::as_str),
            ) else {
                return Ok(None);
            };
            if root_id.len() > 45 || document_version.len() > 4096 || oscal_version.len() > 64 {
                return Ok(None);
            }
            Ok(Some(super::NativeIdentity {
                model: expected.as_str().to_owned(),
                root_id: root_id.to_owned(),
                document_version: document_version.to_owned(),
                oscal_version: oscal_version.to_owned(),
            }))
        })
    })?;
    // MemberFacts' original tree/string envelope reserves this small identity copy too;
    // the validation scratch ticket remains live until the actual clone has completed.
    run(admission, 1, Stage::ValidateResource, |_| Ok(validated.value().clone()))
}

/// Compute the complete longest semantic JSON-path width without constructing any path.
/// Object keys escape at most twice their actual UTF-8 bytes plus bracket punctuation;
/// each actual array index fits at most usize's decimal width plus two brackets.
fn path_upper<C: WorkControl + ?Sized>(
    value: &Value,
    current: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    admission.charge(1)?;
    let mut maximum = current;
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                admission.charge(1 + byte_work(key.len()))?;
                let key_width = multiply(key.len(), 2, admission)?;
                let segment = add(key_width, 4, admission)?;
                let child_width = add(current, segment, admission)?;
                maximum = maximum.max(path_upper(child, child_width, admission)?);
            }
        }
        Value::Array(values) => {
            let width = add(current, 2 + usize::BITS as usize, admission)?;
            for child in values {
                maximum = maximum.max(path_upper(child, width, admission)?);
            }
        }
        _ => {}
    }
    Ok(maximum)
}

/// Charge the entire fixed official schema operand before maintained cached compilation/validation.
/// This is complete operand admission, not a bound on jsonschema's internal instruction count.
pub(super) fn admit_native_schema<C: WorkControl + ?Sized>(
    role: Role,
    admission: &mut Admission<'_, C>,
) -> WorkResult<()> {
    let raw: &[u8] = match role {
        Role::OscalCatalogArtifact => include_bytes!("../../schemas/oscal_catalog_schema.json"),
        Role::OscalComponentArtifact => include_bytes!("../../schemas/oscal_component_schema.json"),
        Role::OscalProfileArtifact => include_bytes!("../../schemas/oscal_profile_schema.json"),
        Role::OscalSspArtifact => include_bytes!("../../schemas/oscal_ssp_schema.json"),
        Role::MappingCollection => include_bytes!("../../schemas/oscal_mapping_schema.json"),
        _ => return Err(policy_failure()),
    };
    let (nodes, _) = raw_bound(raw, RAW_LIMIT, admission)?;
    let complete = add(nodes, byte_work(raw.len()), admission)?;
    run(admission, complete, Stage::ValidateResource, |_| Ok(()))
}

/// Account complete /2 raw parse, event context, event serialization and intrinsic registries.
/// Initial parse keeps this full descriptor. Status from its immutable owned result uses
/// separately admitted captured correspondence/projection operations below.
fn lifecycle_work<C: WorkControl + ?Sized>(
    value: &Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let complete = operand(value, admission)?;
    let mut context = 0;
    for key in ["policy", "parties", "approval_policy", "review"] {
        if let Some(field) = value.get(key) {
            context = add(context, operand(field, admission)?.work, admission)?;
        }
    }
    let history = value.get("history").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    admission.charge(history.len())?;
    let mut events = 0;
    for event in history {
        events = add(events, operand(event, admission)?.work, admission)?;
    }
    let repeated = multiply(context, history.len(), admission)?;
    let windows = multiply(history.len(), history.len(), admission)?;
    let generated =
        value.pointer("/policy/generated_artifacts").and_then(Value::as_array).map_or(0, Vec::len);
    let paths = multiply(generated, generated, admission)?;
    let tree = multiply(complete.work, 8, admission)?;
    let events = multiply(events, 2, admission)?;
    add(
        add(add(tree, repeated, admission)?, events, admission)?,
        add(windows, paths, admission)?,
        admission,
    )
}

/// Project complete current status from actual immutable parsed data on the original ledger.
/// Initial raw parsing/intrinsic validation remains fully charged by `prepare_member`.
/// All actual tuple/path work stays in `route_work`; projection has its own complete
/// source-backed operation descriptor before the unchanged native helper can grow output.
pub(super) fn project_status<C: WorkControl + ?Sized>(
    parsed: &ParsedLifecycleRecord,
    current: &CurrentArtifacts,
    as_of: Option<chrono::NaiveDate>,
    route_work: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Option<StatusReport>> {
    let projection = status_projection_work(parsed.record(), current, admission)?;
    let amount = add(route_work, projection, admission)?;
    run(admission, amount, Stage::PrepareDomain, |_| {
        Ok(status::status_from_parsed_captured(parsed, current, as_of).ok())
    })
}

/// Bound every still-executed captured correspondence, history scan and status-owned string.
/// This descriptor excludes only intrinsic record/event-ID revalidation now absent from
/// the immutable parser-output path. Measurement itself charges every borrowed row first.
fn status_projection_work<C: WorkControl + ?Sized>(
    record: &record::LifecycleRecord,
    current: &CurrentArtifacts,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    // <=32 fixed root/cardinality/state/blocker/date branches and fixed report/error labels.
    let mut amount = 32;
    let (actual, current_width, current_hash_width) =
        captured_projection_work(record, current, &mut amount, admission)?;
    record_projection_work(record, &mut amount, admission)?;
    // Drift equality inspects complete current/Approved fingerprint strings, not context.
    let fingerprints = add(current_width, current_hash_width, admission)?;
    let approved_width = approved_projection_width(record, admission)?;
    let equal_width = add(fingerprints, approved_width, admission)?;
    amount = add(amount, add(actual, 1, admission)?, admission)?;
    amount = add(amount, byte_work(equal_width), admission)?;
    Ok(amount)
}

/// Measure full captured-path/hash/identity operands, then admit unchanged native checks/copies.
/// The returned widths/count describe only borrowed ordinary data, never native authority.
fn captured_projection_work<C: WorkControl + ?Sized>(
    record: &record::LifecycleRecord,
    current: &CurrentArtifacts,
    amount: &mut usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<(usize, usize, usize)> {
    let mut expected_width = 0;
    for row in &record.policy.generated_artifacts {
        admission.charge(1)?;
        expected_width = add(expected_width, row.path.len(), admission)?;
    }
    let expected = record.policy.generated_artifacts.len();
    *amount =
        add(*amount, comparison_work(expected, expected, expected_width, admission)?, admission)?;
    let mut current_width = 0;
    let mut current_hash_width = current.fingerprints.source_sha256.len();
    *amount =
        add(*amount, projected_text(&current.fingerprints.source_sha256, admission)?, admission)?;
    for row in &current.fingerprints.generated_artifacts {
        admission.charge(1)?;
        current_width = add(current_width, row.path.len(), admission)?;
        current_hash_width = add(current_hash_width, row.sha256.len(), admission)?;
        // Complete hash syntax scan plus the later exact fingerprint path/hash copy.
        *amount = add(*amount, projected_text(&row.sha256, admission)?, admission)?;
        *amount = add(*amount, projected_text(&row.sha256, admission)?, admission)?;
        *amount = add(*amount, projected_text(&row.path, admission)?, admission)?;
    }
    // Source hash is scanned before the later full fingerprint copy.
    *amount =
        add(*amount, projected_text(&current.fingerprints.source_sha256, admission)?, admission)?;
    let actual = current.fingerprints.generated_artifacts.len();
    let path_width = add(expected_width, current_width, admission)?;
    *amount = add(*amount, comparison_work(actual, expected, path_width, admission)?, admission)?;
    *amount = add(*amount, comparison_work(actual, 1, current_width, admission)?, admission)?;
    let mut changed_width = 0;
    for path in &current.identity_changes {
        admission.charge(1)?;
        changed_width = add(changed_width, path.len(), admission)?;
        *amount = add(*amount, projected_text(path, admission)?, admission)?;
    }
    let changed = current.identity_changes.len();
    *amount = add(
        *amount,
        comparison_work(
            changed,
            expected,
            add(changed_width, expected_width, admission)?,
            admission,
        )?,
        admission,
    )?;
    *amount =
        add(*amount, comparison_work(changed, changed, changed_width, admission)?, admission)?;
    Ok((actual, current_width, current_hash_width))
}

/// Admit complete policy/owner/history/private payload and both native sorted-impact passes.
fn record_projection_work<C: WorkControl + ?Sized>(
    record: &record::LifecycleRecord,
    amount: &mut usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<()> {
    for text in [&record.policy.policy_key, &record.policy.version_key] {
        *amount = add(*amount, projected_text(text, admission)?, admission)?;
    }
    for owner in &record.policy.owner_keys {
        admission.charge(1)?;
        *amount = add(*amount, projected_text(owner, admission)?, admission)?;
    }
    let history = record.history.len();
    *amount = add(*amount, history, admission)?; // Latest Approved reverse selection.
    let mut impacts = 0;
    let mut impact_width = 0;
    for event in &record.history {
        admission.charge(1)?;
        // The output event-ID clone and the full history flattening participation.
        *amount = add(*amount, projected_text(&event.event_id, admission)?, admission)?;
        *amount = add(*amount, 1, admission)?;
        for finding in &event.impact_finding_ids {
            admission.charge(1)?;
            impacts = add(impacts, 1, admission)?;
            impact_width = add(impact_width, finding.len(), admission)?;
            *amount = add(*amount, projected_text(finding, admission)?, admission)?;
        }
    }
    *amount = add(*amount, comparison_work(impacts, impacts, impact_width, admission)?, admission)?;
    *amount = add(*amount, impacts, admission)?; // Complete sorted set-to-vector move.
    // Descriptor discovery of the latest approval charges its actual reverse row visits;
    // projection's independent selection was pre-admitted above and still runs natively.
    for event in record.history.iter().rev() {
        admission.charge(1)?;
        if event.next_state == record::LifecycleState::Approved {
            *amount = add(
                *amount,
                projected_text(&event.fingerprints.source_sha256, admission)?,
                admission,
            )?;
            for row in &event.fingerprints.generated_artifacts {
                admission.charge(1)?;
                *amount = add(*amount, projected_text(&row.path, admission)?, admission)?;
                *amount = add(*amount, projected_text(&row.sha256, admission)?, admission)?;
            }
            break;
        }
    }
    if let Some(replacement) = &record.replaced_by {
        *amount = add(*amount, projected_text(&replacement.policy_key, admission)?, admission)?;
        *amount = add(*amount, projected_text(&replacement.version_key, admission)?, admission)?;
    }
    Ok(())
}

/// Charge the complete actual latest-approval search and measure its full fingerprint width.
/// This measurement scan remains separate from the pre-admitted native reverse selection.
fn approved_projection_width<C: WorkControl + ?Sized>(
    record: &record::LifecycleRecord,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut approved_width = 0;
    for event in record.history.iter().rev() {
        admission.charge(1)?;
        if event.next_state == record::LifecycleState::Approved {
            approved_width =
                add(approved_width, event.fingerprints.source_sha256.len(), admission)?;
            for row in &event.fingerprints.generated_artifacts {
                admission.charge(1)?;
                approved_width = add(
                    approved_width,
                    add(row.path.len(), row.sha256.len(), admission)?,
                    admission,
                )?;
            }
            break;
        }
    }
    Ok(approved_width)
}

/// Admit operand measurement, then return the full future native copy/scan operation units.
fn projected_text<C: WorkControl + ?Sized>(
    text: &str,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    admission.charge(1)?;
    add(1, byte_work(text.len()), admission)
}

/// Pre-admit complete node comparisons and both full string operands for native ordered sets.
/// Worst-case Cartesian comparisons conservatively cover `BTree` insertion/contains work.
fn comparison_work<C: WorkControl + ?Sized>(
    left: usize,
    right: usize,
    width: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let pairs = multiply(left, right, admission)?;
    add(pairs, multiply(pairs, byte_work(width), admission)?, admission)
}

/// Scalar preallocation state; no parsed tree, proof or source text is constructed.
#[derive(Default)]
struct Lexical {
    /// Whether the raw cursor is inside a quoted operand.
    quoted: bool,
    /// Whether a backslash introduced the next escape byte.
    escaped: bool,
    /// Remaining raw hexadecimal positions in a Unicode escape.
    unicode: usize,
    /// Conservative complete current quoted UTF-8 width.
    string_upper: usize,
    /// Complete currently open container count.
    containers: usize,
    /// Complete unquoted opening/colon/comma occurrence count.
    separators: usize,
}

impl Lexical {
    /// Charge-free scalar inspection occurs only inside a complete pre-admitted raw pass.
    fn visit<C: WorkControl + ?Sized>(
        &mut self,
        byte: u8,
        admission: &mut Admission<'_, C>,
    ) -> WorkResult<()> {
        if self.quoted {
            if self.unicode != 0 {
                self.unicode -= 1;
            } else if self.escaped {
                self.escaped = false;
                let width = if byte == b'u' {
                    self.unicode = 4;
                    3
                } else {
                    1
                };
                self.string_upper = add(self.string_upper, width, admission)?;
            } else if byte == b'\\' {
                self.escaped = true;
            } else if byte == b'"' {
                self.quoted = false;
            } else {
                self.string_upper = add(self.string_upper, 1, admission)?;
            }
            if self.string_upper > MAX_STRING_BYTES {
                return Err(policy_failure());
            }
        } else if byte == b'"' {
            self.quoted = true;
            self.string_upper = 0;
        } else {
            if matches!(byte, b'[' | b'{') {
                self.containers = add(self.containers, 1, admission)?;
                if self.containers > 65 {
                    return Err(policy_failure());
                }
            } else if matches!(byte, b']' | b'}') {
                self.containers = self.containers.saturating_sub(1);
            }
            if matches!(byte, b'[' | b'{' | b',' | b':') {
                self.separators = add(self.separators, 1, admission)?;
            }
        }
        Ok(())
    }
}

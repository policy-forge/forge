//! Genuine once-captured Mapping/lifecycle closure for portable re-review.
//! Approval is the maintained neutral recorded lifecycle relation, not locator
//! declarations, native schema, recomputation, or source hashes alone. Private
//! pending data seals only against the exact actual original Rc owners and a
//! complete controlled generation fence. No MCP/Snapshot authority is reused.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};
use crate::lifecycle::record::{
    self, ArtifactFingerprint, FingerprintSet, LifecycleRecord, LifecycleState, NamedHash,
};
use crate::lifecycle::status::{CurrentArtifacts, status_from_captured};
use crate::mapping::{inventory, manifest, model};
use crate::validate::{self, OscalModelType};
use crate::workspace::preparation::WorkControl;

use super::capture::{HeldReviewInputs, Pool, ReviewCapture};
use super::decode::{ContractError, ContractLedger};
use super::wire::{SourceModel, SourcePin};

/// Fixed private routing format; it declares inputs but never approval.
const LOCATOR_SCHEMA: &str = "forge.review-source-locator/1";
/// Complete physical source/native/lifecycle union, never a selected prefix.
const MAX_SOURCES: usize = 100;
/// Existing source pool whole raw extent, before original retention.
const MAX_SOURCE_BYTES: usize = 50 * 1024 * 1024;
/// One private locator original occupies the existing auxiliary pool.
const MAX_LOCATOR_BYTES: usize = 1024 * 1024;
/// Existing individual native/manifest source ceiling.
const MAX_SOURCE_FILE: usize = 10 * 1024 * 1024;
/// Complete bounded parsed tree profile, without parser preemption guarantees.
const MAX_NODES: usize = 100_000;
/// Actual `ReviewCapture` maximum successful-entry/attempt extent for no-IO scans.
const MAX_HELD_ORIGINALS: usize = 10_105;

/// Closed private source route consumed only from an actual auxiliary original.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Locator {
    /// Exact internal format marker.
    schema_version: String,
    /// Declared actual mapping manifest original key.
    mapping_manifest_key: String,
    /// Declared actual lifecycle record original key.
    lifecycle_record_key: String,
    /// Declared actual Mapping native original key.
    mapping_key: String,
    /// Complete key-sorted, physically unique required original declarations.
    sources: Vec<SourceDeclaration>,
}

/// One private source spelling/model, not an expected approval assertion.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDeclaration {
    /// Canonical stable source token, max128 ASCII bytes.
    key: String,
    /// Original normalized root-relative descendant spelling.
    path: PathBuf,
    /// Closed declared source family; unsupported families refuse completely.
    model: SourceModel,
}

/// Actual allocation binding retained before final capture sealing.
struct OriginalBinding {
    /// Index in the same actual ReviewCapture/HeldReviewInputs owner.
    index: usize,
    /// Opaque actual original Rc lease; no raw duplicate allocation.
    lease: CaptureLease,
}

/// Parsed original native identity, owned under the precharged tree profile.
struct NativeOriginal {
    /// Exact complete strict decoded native Value, never a generated stand-in.
    value: Value,
    /// Actual detected and offline-schema-validated native model.
    model: OscalModelType,
    /// Original native UUID spelling used for lifecycle identity comparison.
    root_uuid: String,
    /// Canonical same UUID used in the closed path-free source pin.
    canonical_uuid: String,
    /// Actual bounded ASCII supported OSCAL schema version.
    oscal_version: String,
}

/// Private pending native/report result; it is not a successful closure proof.
pub(crate) struct PendingMappingClosure {
    /// Exact source native Value from an actual original generation.
    native: Value,
    /// Complete path-free observed source union in stable declared-key order.
    pins: Vec<SourcePin>,
    /// Locator and every actual source allocation binding, never raw hash alone.
    originals: Vec<OriginalBinding>,
}

/// Actual neutral Approved/current Mapping closure, with complete held originals.
/// No external detached constructor, serde, Clone, or Debug is provided.
pub(crate) struct ApprovedMappingClosure {
    /// Complete original source Mapping value, not a caller-authored payload.
    native: Value,
    /// Complete current original pins; paths/prose/party names stay private.
    pins: Vec<SourcePin>,
    /// Genuine final proof owner kept alive through every review/final fence.
    held: Rc<HeldReviewInputs>,
}

impl PendingMappingClosure {
    /// Seal only the same actual original allocations after all original fences.
    pub(crate) fn seal(
        self,
        held: Rc<HeldReviewInputs>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<ApprovedMappingClosure, ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            // Count scans every distinct held entry, including other bounded pools.
            // Charge the actual complete maximum before this no-IO scan; pins
            // contain only Source originals, not duplicate response registrations.
            ledger.visits(MAX_HELD_ORIGINALS)?;
            ledger.bytes(
                MAX_HELD_ORIGINALS
                    .checked_mul(std::mem::size_of::<Pool>())
                    .ok_or(ContractError::Capacity)?,
            )?;
            if held.source_original_count() != self.pins.len() {
                return Err(ContractError::Binding);
            }
            for original in &self.originals {
                ledger.checkpoint(control)?;
                ledger.visits(1)?;
                if !held.same_original(original.index, &original.lease)? {
                    return Err(ContractError::Binding);
                }
            }
            held.verify_inputs(ledger, control)?;
            // The actual controlled native fence includes its own final stop;
            // this extra boundary prevents a stale caller result from sealing.
            ledger.checkpoint(control)?;
            Ok(ApprovedMappingClosure { native: self.native, pins: self.pins, held })
        })
    }
}

impl ApprovedMappingClosure {
    /// Borrow the actual complete native original used by item minimization.
    pub(crate) fn native_value(&self) -> &Value {
        &self.native
    }
    /// Borrow the complete exact current source union, never a selected subset.
    pub(crate) fn source_pins(&self) -> &[SourcePin] {
        &self.pins
    }
    /// Borrow the exact held owner for finalizer Rc identity binding, without IO.
    /// No caller can construct or substitute a successful closure through this port.
    pub(crate) fn held_inputs(&self) -> &Rc<HeldReviewInputs> {
        &self.held
    }
    /// Consume the same actual held original fence and sticky caller ledger.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        self.held.verify_inputs(ledger, control)
    }
}

/// Build pending facts from one genuine shared capture, without source rereads.
///
/// The locator must be an actual Auxiliary/ReviewPrivateConfig original in this
/// capture. The whole declared source union is admitted before its remaining
/// originals are read. Parser/schema calls are bounded and cooperatively fenced;
/// they are not hard-preempted or literal heap-confined. No native mutation occurs.
pub(crate) fn prepare(
    capture: &mut ReviewCapture,
    locator_index: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingMappingClosure, ContractError> {
    ledger.bound(|ledger| {
        let inputs = prepare_inputs(capture, locator_index, ledger, control)?;
        let ParsedOriginals { originals, mut native_rows, pins } =
            parse_originals(capture, locator_index, &inputs, ledger, control)?;
        verify_product(capture, &inputs, &native_rows, ledger, control)?;
        verify_approval(capture, &inputs, &pins, &native_rows, ledger, control)?;
        let native = native_rows[inputs.native_slot].take().ok_or(ContractError::Invalid)?.value;
        ledger.checkpoint(control)?;
        Ok(PendingMappingClosure { native, pins, originals })
    })
}

/// Parsed initial originals and complete physical union; no native approval is implied.
struct InitialInputs {
    /// Closed actual locator, retained without a copied wire proof.
    locator: Locator,
    /// Complete actual Source-pool registration indices.
    indices: Vec<Option<usize>>,
    /// Actual manifest original index in the same capture.
    manifest_index: usize,
    /// Actual lifecycle original index in the same capture.
    record_index: usize,
    /// Declared Mapping slot, structurally checked before further source reads.
    native_slot: usize,
    /// Maintained parsed manifest from the actual captured original.
    mapping_manifest: manifest::MappingManifest,
    /// Intrinsically validated actual recorded lifecycle original.
    lifecycle: LifecycleRecord,
}

/// Exact parsed source forms and observed pins, still pending both authority gates.
struct ParsedOriginals {
    /// Complete locator/source original ownership bindings.
    originals: Vec<OriginalBinding>,
    /// Complete strict schema-qualified native forms in locator order.
    native_rows: Vec<Option<NativeOriginal>>,
    /// Complete minimized observed source pins.
    pins: Vec<SourcePin>,
}

/// Capture the initial originals, admit the complete union and then capture remaining sources.
fn prepare_inputs(
    capture: &mut ReviewCapture,
    locator_index: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<InitialInputs, ContractError> {
    ledger.checkpoint(control)?;
    if capture.role(locator_index)? != CaptureRole::ReviewPrivateConfig {
        return Err(ContractError::Invalid);
    }
    let locator = decode_locator(capture.bytes(locator_index)?, ledger, control)?;
    let manifest_slot = key_slot(&locator, &locator.mapping_manifest_key, ledger, control)?;
    let record_slot = key_slot(&locator, &locator.lifecycle_record_key, ledger, control)?;
    let native_slot = key_slot(&locator, &locator.mapping_key, ledger, control)?;
    if locator.sources[manifest_slot].model != SourceModel::MappingManifest
        || locator.sources[record_slot].model != SourceModel::LifecycleRecord
        || locator.sources[native_slot].model != SourceModel::Mapping
    {
        return Err(ContractError::Invalid);
    }
    ledger.derived(
        locator
            .sources
            .len()
            .checked_mul(256)
            .and_then(|n| n.checked_add(256))
            .ok_or(ContractError::Capacity)?,
    )?;
    let mut indices = vec![None; locator.sources.len()];
    let manifest_index = capture_source(capture, &locator.sources[manifest_slot], ledger, control)?;
    indices[manifest_slot] = Some(manifest_index);
    let record_index = capture_source(capture, &locator.sources[record_slot], ledger, control)?;
    indices[record_slot] = Some(record_index);
    admit_decode(capture.bytes(manifest_index)?, MAX_SOURCE_FILE, ledger, control)?;
    let mapping_manifest =
        manifest::parse(capture.bytes(manifest_index)?).map_err(|_| ContractError::Invalid)?;
    ledger.checkpoint(control)?;
    admit_decode(
        capture.bytes(record_index)?,
        usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| ContractError::Capacity)?,
        ledger,
        control,
    )?;
    let lifecycle =
        record::parse(capture.bytes(record_index)?).map_err(|_| ContractError::Invalid)?;
    ledger.checkpoint(control)?;
    // Refuse missing/extra paths and incompatible roles before reading any
    // remaining current resource/native original. Hrefs never authorize IO.
    validate_union(
        capture.path(manifest_index)?,
        capture.path(record_index)?,
        &locator,
        &mapping_manifest,
        &lifecycle,
        ledger,
        control,
    )?;
    for (slot, declaration) in locator.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        if indices[slot].is_none() {
            indices[slot] = Some(capture_source(capture, declaration, ledger, control)?);
        }
    }
    Ok(InitialInputs {
        locator,
        indices,
        manifest_index,
        record_index,
        native_slot,
        mapping_manifest,
        lifecycle,
    })
}

/// Parse every actual source original and preserve exact pins and original-owner bindings.
fn parse_originals(
    capture: &ReviewCapture,
    locator_index: usize,
    inputs: &InitialInputs,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ParsedOriginals, ContractError> {
    let InitialInputs { locator, indices, lifecycle, .. } = inputs;
    ledger.derived(locator.sources.len().checked_mul(1024).ok_or(ContractError::Capacity)?)?;
    let mut originals = Vec::with_capacity(locator.sources.len() + 1);
    originals.push(OriginalBinding { index: locator_index, lease: capture.lease(locator_index)? });
    let mut native_rows = Vec::with_capacity(locator.sources.len());
    let mut pins = Vec::with_capacity(locator.sources.len());
    let mut total = 0_usize;
    for (slot, declaration) in locator.sources.iter().enumerate() {
        let index = indices[slot].ok_or(ContractError::Invalid)?;
        let bytes = capture.bytes(index)?;
        ledger.checkpoint(control)?;
        total = total
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_SOURCE_BYTES)
            .ok_or(ContractError::Capacity)?;
        if bytes.is_empty() {
            return Err(ContractError::Invalid);
        }
        originals.push(OriginalBinding { index, lease: capture.lease(index)? });
        let native = if let Some(expected) = native_model(declaration.model) {
            Some(parse_native(bytes, expected, ledger, control)?)
        } else {
            match declaration.model {
                SourceModel::MappingManifest => {
                    admit_decode(bytes, MAX_SOURCE_FILE, ledger, control)?;
                    manifest::parse(bytes).map_err(|_| ContractError::Invalid)?;
                }
                SourceModel::LifecycleRecord => {
                    admit_decode(
                        bytes,
                        usize::try_from(record::MAX_RECORD_BYTES)
                            .map_err(|_| ContractError::Capacity)?,
                        ledger,
                        control,
                    )?;
                    record::parse(bytes).map_err(|_| ContractError::Invalid)?;
                }
                _ => return Err(ContractError::Invalid),
            }
            None
        };
        ledger.bytes(bytes.len())?;
        let hash = sha256_hex(bytes);
        let schema_identity = match &native {
            Some(row) => {
                let size = row
                    .oscal_version
                    .len()
                    .checked_add(row.model.as_str().len())
                    .and_then(|n| n.checked_add(7))
                    .ok_or(ContractError::Capacity)?;
                if size > 128 {
                    return Err(ContractError::Invalid);
                }
                ledger.derived(size)?;
                format!("oscal:{}:{}", row.oscal_version, row.model.as_str())
            }
            None if declaration.model == SourceModel::MappingManifest => {
                manifest::MANIFEST_SCHEMA_VERSION.to_string()
            }
            None => lifecycle.schema_version.clone(),
        };
        pins.push(SourcePin {
            artifact_key: declaration.key.clone(),
            model: declaration.model,
            native_root_uuid: native.as_ref().map(|row| row.canonical_uuid.clone()),
            raw_sha256: hash,
            byte_length: u64::try_from(bytes.len()).map_err(|_| ContractError::Capacity)?,
            schema_identity,
        });
        native_rows.push(native);
    }
    Ok(ParsedOriginals { originals, native_rows, pins })
}

/// Rebuild through the maintained producer and compare the complete schema-qualified native value.
fn verify_product(
    capture: &ReviewCapture,
    inputs: &InitialInputs,
    native_rows: &[Option<NativeOriginal>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let InitialInputs { locator, indices, manifest_index, native_slot, mapping_manifest, .. } =
        inputs;
    let source = load_resource(
        capture,
        locator,
        indices,
        capture.path(*manifest_index)?,
        &mapping_manifest.mapping.source,
        ledger,
        control,
    )?;
    let target = load_resource(
        capture,
        locator,
        indices,
        capture.path(*manifest_index)?,
        &mapping_manifest.mapping.target,
        ledger,
        control,
    )?;
    let product = model::build_admitted(mapping_manifest, &source, &target, false, &mut |charge| {
        model_charge(charge, ledger, control)
    })
    .map_err(|error| match error {
        model::CapturedBuildError::Admission(error) => error,
        model::CapturedBuildError::Domain(_) => ContractError::Binding,
    })?;
    ledger.checkpoint(control)?;
    let expected = comparison_value(&product.artifact, ledger, control)?;
    let selected = native_rows[*native_slot].as_ref().ok_or(ContractError::Invalid)?;
    if !equal_values(&selected.value, &expected, ledger, control)? {
        return Err(ContractError::Binding);
    }
    // Complete native schema consumed both for the source and the recomputed
    // model; equal values alone are not lifecycle/current approval.
    check_schema(&expected, OscalModelType::Mapping, ledger, control)?;
    Ok(())
}

/// Require the exact complete maintained Approved/current recorded lifecycle tuple.
fn verify_approval(
    capture: &ReviewCapture,
    inputs: &InitialInputs,
    pins: &[SourcePin],
    native_rows: &[Option<NativeOriginal>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let InitialInputs { locator, record_index, native_slot, lifecycle, .. } = inputs;
    let current = current_artifacts(
        capture.path(*record_index)?,
        locator,
        pins,
        native_rows,
        lifecycle,
        ledger,
        control,
    )?;
    let selected_path = &locator.sources[*native_slot].path;
    let generated = lifecycle.policy.generated_artifacts.iter().try_fold(false, |found, row| {
        let path =
            join_relative(capture.path(*record_index)?, Path::new(&row.path), ledger, control)?;
        Ok::<_, ContractError>(found | same_path(&path, selected_path, ledger, control)?)
    })?;
    if !generated {
        return Err(ContractError::Binding);
    }
    // Bound the existing pure projector's complete cloned report and private
    // collections before it grows. No clock or schedule projection is supplied.
    ledger.derived(
        capture
            .bytes(*record_index)?
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(4096))
            .ok_or(ContractError::Capacity)?,
    )?;
    ledger.checkpoint(control)?;
    let status =
        status_from_captured(lifecycle, &current, None).map_err(|_| ContractError::Binding)?;
    ledger.checkpoint(control)?;
    if status.state != LifecycleState::Approved
        || status.derived_status != "approved"
        || !status.blockers.is_empty()
        || !status.artifact_identity_changes.is_empty()
        || status.approved_fingerprints.as_ref() != Some(&status.current_fingerprints)
        || status.current_fingerprints != current.fingerprints
    {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Admit the complete actual generated comparison form before its second serialization/allocation.
/// The maintained model's native/report reservation is separate; both complete
/// passes use this same ledger and control. The logical multiplier is a
/// conservative refusal profile, not a proven allocator/total-heap ceiling.
fn comparison_value(
    artifact: &impl serde::Serialize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Value, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let mut generated = Counter { length: 0, ledger, control };
        serde_json::to_writer(&mut generated, artifact).map_err(|_| generated.ledger.failure())?;
        let generated_extent = generated.length;
        generated.ledger.bytes(generated_extent)?;
        generated.ledger.derived(
            generated_extent
                .checked_mul(16)
                .and_then(|n| n.checked_add(512))
                .ok_or(ContractError::Capacity)?,
        )?;
        let value = serde_json::to_value(artifact).map_err(|_| ContractError::Invalid)?;
        ledger.checkpoint(control)?;
        Ok(value)
    })
}

/// Decode a closed complete locator without any caller-provided native approval.
fn decode_locator(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Locator, ContractError> {
    let value = strict_value(raw, MAX_LOCATOR_BYTES, ledger, control)?;
    let declared = value.get("sources").and_then(Value::as_array).ok_or(ContractError::Invalid)?;
    if declared.is_empty() || declared.len() > MAX_SOURCES {
        return Err(ContractError::Invalid);
    }
    // The complete source row cardinality is admitted before typed Vec growth.
    // A separately admitted typed form is constructed from the bounded original.
    admit_decode(raw, MAX_LOCATOR_BYTES, ledger, control)?;
    let locator: Locator = serde_json::from_slice(raw).map_err(|_| ContractError::Invalid)?;
    drop(value);
    if locator.schema_version != LOCATOR_SCHEMA
        || locator.sources.is_empty()
        || locator.sources.len() > MAX_SOURCES
    {
        return Err(ContractError::Invalid);
    }
    for key in [&locator.mapping_manifest_key, &locator.lifecycle_record_key, &locator.mapping_key]
    {
        token(key, ledger)?;
    }
    for (index, row) in locator.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        token(&row.key, ledger)?;
        portable(&row.path, ledger)?;
        if role(row.model).is_none() {
            return Err(ContractError::Invalid);
        }
        if index > 0
            && compare(&locator.sources[index - 1].key, &row.key, ledger)?
                != std::cmp::Ordering::Less
        {
            return Err(ContractError::Invalid);
        }
        for other in &locator.sources[..index] {
            ledger.checkpoint(control)?;
            if folded_path(&row.path, &other.path, ledger)? {
                return Err(ContractError::Invalid);
            }
        }
    }
    Ok(locator)
}

/// Admit a conservative decoded-form allowance before the bounded parser grows.
/// The multiplier is a logical first-profile refusal, not a proven heap ceiling.
fn admit_decode(
    raw: &[u8],
    limit: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    if raw.is_empty() || raw.len() > limit {
        return Err(ledger.capacity());
    }
    ledger.bytes(raw.len())?;
    ledger.derived(
        raw.len()
            .checked_mul(16)
            .and_then(|n| n.checked_add(512))
            .ok_or(ContractError::Capacity)?,
    )
}

/// Parse complete duplicate-key-safe native JSON with raw/depth/string/tree bounds.
fn strict_value(
    raw: &[u8],
    limit: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Value, ContractError> {
    admit_decode(raw, limit, ledger, control)?;
    if raw.starts_with(b"\xef\xbb\xbf") || std::str::from_utf8(raw).is_err() {
        return Err(ContractError::Invalid);
    }
    let value = json_strict::parse_value(
        raw,
        "private review source",
        Limits { max_depth: 64, max_string_bytes: 65_536 },
    )
    .map_err(|_| ContractError::Invalid)?;
    let mut nodes = 0;
    tree(&value, 0, &mut nodes, ledger, control)?;
    ledger.checkpoint(control)?;
    Ok(value)
}

/// Inspect complete bounded decoded nodes before downstream native registry growth.
fn tree(
    value: &Value,
    depth: usize,
    nodes: &mut usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    *nodes = nodes.checked_add(1).filter(|n| *n <= MAX_NODES).ok_or(ContractError::Capacity)?;
    if depth > 64 {
        return Err(ContractError::Invalid);
    }
    match value {
        Value::String(text) => {
            ledger.bytes(text.len())?;
            if text.len() > 65_536 {
                return Err(ContractError::Invalid);
            }
        }
        Value::Array(rows) => {
            for row in rows {
                tree(row, depth + 1, nodes, ledger, control)?;
            }
        }
        Value::Object(rows) => {
            for (key, row) in rows {
                ledger.bytes(key.len())?;
                if key.len() > 65_536 {
                    return Err(ContractError::Invalid);
                }
                tree(row, depth + 1, nodes, ledger, control)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Select only actually supported native families; no generated authority is inferred.
fn native_model(model: SourceModel) -> Option<OscalModelType> {
    match model {
        SourceModel::Mapping => Some(OscalModelType::Mapping),
        SourceModel::Catalog | SourceModel::ResolvedCatalog => Some(OscalModelType::Catalog),
        SourceModel::Profile => Some(OscalModelType::Profile),
        SourceModel::ComponentDefinition => Some(OscalModelType::ComponentDefinition),
        _ => None,
    }
}

/// Match the first Mapping closure's exact internal admitted role.
fn role(model: SourceModel) -> Option<CaptureRole> {
    match model {
        SourceModel::Mapping => Some(CaptureRole::MappingCollection),
        SourceModel::Catalog | SourceModel::ResolvedCatalog => Some(CaptureRole::Catalog),
        SourceModel::Profile => Some(CaptureRole::Profile),
        SourceModel::ComponentDefinition => Some(CaptureRole::ComponentDefinition),
        SourceModel::MappingManifest => Some(CaptureRole::MappingManifest),
        SourceModel::LifecycleRecord => Some(CaptureRole::LifecycleRecord),
        _ => None,
    }
}

/// Admit original spelling before any parent/join/native capture operation.
fn portable(path: &Path, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.bytes(path.as_os_str().as_encoded_bytes().len())?;
    if path.as_os_str().as_encoded_bytes().len() > 4096 || path.components().count() > 64 {
        return Err(ContractError::Invalid);
    }
    crate::linkage::fresh::validate_relative(path).map_err(|_| ContractError::Invalid)
}

/// Match bounded ASCII source tokens exactly without trimming or inferred keys.
fn token(value: &str, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.bytes(value.len())?;
    if value.is_empty()
        || value.len() > 128
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Meter complete actual operands before private key/UUID/schema comparisons.
fn compare(
    left: &str,
    right: &str,
    ledger: &mut ContractLedger,
) -> Result<std::cmp::Ordering, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(left.len().checked_add(right.len()).ok_or(ContractError::Capacity)?)?;
    Ok(left.cmp(right))
}

/// Compare two original private descendants before any joining or disclosure.
fn same_path(
    left: &Path,
    right: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    ledger.bytes(
        left.as_os_str()
            .as_encoded_bytes()
            .len()
            .checked_add(right.as_os_str().as_encoded_bytes().len())
            .ok_or(ContractError::Capacity)?,
    )?;
    Ok(left == right)
}

/// Refuse portable folded aliases among independently declared source paths.
fn folded_path(
    left: &Path,
    right: &Path,
    ledger: &mut ContractLedger,
) -> Result<bool, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(
        left.as_os_str()
            .as_encoded_bytes()
            .len()
            .checked_add(right.as_os_str().as_encoded_bytes().len())
            .ok_or(ContractError::Capacity)?,
    )?;
    Ok(left
        .as_os_str()
        .as_encoded_bytes()
        .eq_ignore_ascii_case(right.as_os_str().as_encoded_bytes()))
}

/// Resolve exactly one original declaration by its canonical stable private key.
fn key_slot(
    locator: &Locator,
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, row) in locator.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        if compare(&row.key, key, ledger)? == std::cmp::Ordering::Equal
            && found.replace(index).is_some()
        {
            return Err(ContractError::Invalid);
        }
    }
    found.ok_or(ContractError::Invalid)
}

/// Resolve a declared actual path and compatible native role without href IO.
fn path_slot(
    locator: &Locator,
    path: &Path,
    expected: CaptureRole,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, row) in locator.sources.iter().enumerate() {
        if same_path(&row.path, path, ledger, control)?
            && (role(row.model) != Some(expected) || found.replace(index).is_some())
        {
            return Err(ContractError::Binding);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Join only admitted original descendant spelling under its actual captured base.
fn join_relative(
    base_file: &Path,
    relative: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PathBuf, ContractError> {
    ledger.checkpoint(control)?;
    portable(base_file, ledger)?;
    portable(relative, ledger)?;
    let parent = base_file.parent().ok_or(ContractError::Invalid)?;
    let extent = parent
        .as_os_str()
        .as_encoded_bytes()
        .len()
        .checked_add(relative.as_os_str().as_encoded_bytes().len())
        .and_then(|n| n.checked_add(1))
        .ok_or(ContractError::Capacity)?;
    ledger.derived(extent.checked_add(128).ok_or(ContractError::Capacity)?)?;
    let joined = parent.join(relative);
    portable(&joined, ledger)?;
    Ok(joined)
}

/// Capture one declared source under the existing complete source pool.
fn capture_source(
    capture: &mut ReviewCapture,
    row: &SourceDeclaration,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let expected = role(row.model).ok_or(ContractError::Invalid)?;
    let limit = if row.model == SourceModel::LifecycleRecord {
        usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| ContractError::Capacity)?
    } else {
        MAX_SOURCE_FILE
    };
    capture.required(&row.path, expected, Pool::Source, limit, ledger, control)
}

/// Check exact complete source-path union before remaining original reads.
fn validate_union(
    manifest_path: &Path,
    record_path: &Path,
    locator: &Locator,
    mapping: &manifest::MappingManifest,
    record: &LifecycleRecord,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.derived(locator.sources.len().checked_mul(128).ok_or(ContractError::Capacity)?)?;
    let mut needed = vec![false; locator.sources.len()];
    for path in [manifest_path, record_path] {
        let expected = if same_path(path, manifest_path, ledger, control)? {
            CaptureRole::MappingManifest
        } else {
            CaptureRole::LifecycleRecord
        };
        needed[path_slot(locator, path, expected, ledger, control)?] = true;
    }
    for resource in [&mapping.mapping.source, &mapping.mapping.target] {
        let path = join_relative(manifest_path, &resource.artifact, ledger, control)?;
        let expected = match resource.resource_type {
            manifest::ResourceType::Catalog => CaptureRole::Catalog,
            manifest::ResourceType::Profile => CaptureRole::Profile,
        };
        needed[path_slot(locator, &path, expected, ledger, control)?] = true;
        if let Some(companion) = &resource.resolved_catalog {
            let path = join_relative(manifest_path, companion, ledger, control)?;
            needed[path_slot(locator, &path, CaptureRole::Catalog, ledger, control)?] = true;
        }
    }
    for fingerprint in
        std::iter::once(&record.policy.source).chain(&record.policy.generated_artifacts)
    {
        let path = join_relative(record_path, Path::new(&fingerprint.path), ledger, control)?;
        let slot = locator
            .sources
            .iter()
            .enumerate()
            .try_fold(None, |found, (index, row)| {
                if same_path(&row.path, &path, ledger, control)? {
                    if found.is_some() {
                        return Err(ContractError::Binding);
                    }
                    Ok(Some(index))
                } else {
                    Ok(found)
                }
            })?
            .ok_or(ContractError::Binding)?;
        // Generated originals always have actual native identities; source may
        // be the intrinsically validated mapping authoring manifest.
        if fingerprint.oscal_type.is_some() && native_model(locator.sources[slot].model).is_none() {
            return Err(ContractError::Binding);
        }
        needed[slot] = true;
    }
    if needed.iter().any(|needed| !needed) {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Consume actual complete native schema and exact supported version without prose output.
fn check_schema(
    value: &Value,
    model: OscalModelType,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    let validator = validate::compiled_validator(model).map_err(|_| ContractError::Invalid)?;
    let valid = validator.is_valid(value);
    let version_error = validate::version::inspect_oscal_version(value, model).error;
    ledger.checkpoint(control)?;
    if !valid || version_error.is_some() {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Strict native original plus complete schema and per-artifact UUID definitions.
fn parse_native(
    raw: &[u8],
    expected: OscalModelType,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<NativeOriginal, ContractError> {
    let value = strict_value(raw, MAX_SOURCE_FILE, ledger, control)?;
    let model = validate::detect_model_type(&value).map_err(|_| ContractError::Invalid)?;
    if model != expected {
        return Err(ContractError::Binding);
    }
    check_schema(&value, expected, ledger, control)?;
    if expected == OscalModelType::Mapping {
        let count = uuid_count(&value, ledger, control)?;
        ledger.derived(
            count.checked_mul(std::mem::size_of::<Uuid>()).ok_or(ContractError::Capacity)?,
        )?;
        let mut seen = Vec::with_capacity(count);
        uuid_definitions(&value, &mut seen, ledger, control)?;
    }
    let key = match model {
        OscalModelType::Mapping => "mapping-collection",
        OscalModelType::Catalog => "catalog",
        OscalModelType::Profile => "profile",
        OscalModelType::ComponentDefinition => "component-definition",
        _ => return Err(ContractError::Invalid),
    };
    let root = value.get(key).and_then(Value::as_object).ok_or(ContractError::Invalid)?;
    let uuid = root.get("uuid").and_then(Value::as_str).ok_or(ContractError::Invalid)?;
    ledger.bytes(uuid.len())?;
    let canonical_uuid =
        Uuid::parse_str(uuid).map_err(|_| ContractError::Invalid)?.hyphenated().to_string();
    let version = root
        .get("metadata")
        .and_then(|value| value.get("oscal-version"))
        .and_then(Value::as_str)
        .ok_or(ContractError::Invalid)?;
    ledger.bytes(version.len())?;
    if version.is_empty()
        || version.len() > 128
        || !version.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
    {
        return Err(ContractError::Invalid);
    }
    let root_uuid = uuid.to_owned();
    let oscal_version = version.to_owned();
    Ok(NativeOriginal { value, model, root_uuid, canonical_uuid, oscal_version })
}

/// Count the complete original Mapping UUID registry before its one allocation.
fn uuid_count(
    value: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    let mut count = 0_usize;
    match value {
        Value::Object(rows) => {
            for (key, row) in rows {
                ledger.bytes(key.len())?;
                if key == "uuid" {
                    count = count.checked_add(1).ok_or(ContractError::Capacity)?;
                }
                count = count
                    .checked_add(uuid_count(row, ledger, control)?)
                    .ok_or(ContractError::Capacity)?;
            }
        }
        Value::Array(rows) => {
            for row in rows {
                count = count
                    .checked_add(uuid_count(row, ledger, control)?)
                    .ok_or(ContractError::Capacity)?;
            }
        }
        _ => {}
    }
    Ok(count)
}

/// Inspect only actual native `uuid` definitions, never reference-array repeats.
fn uuid_definitions(
    value: &Value,
    seen: &mut Vec<Uuid>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    match value {
        Value::Object(rows) => {
            for (key, row) in rows {
                ledger.bytes(key.len())?;
                if key == "uuid" {
                    let raw = row.as_str().ok_or(ContractError::Invalid)?;
                    ledger.bytes(raw.len())?;
                    let uuid = Uuid::parse_str(raw).map_err(|_| ContractError::Invalid)?;
                    let mut low = 0;
                    let mut high = seen.len();
                    while low < high {
                        ledger.checkpoint(control)?;
                        ledger.visits(1)?;
                        ledger.bytes(32)?;
                        let middle = low + (high - low) / 2;
                        match seen[middle].cmp(&uuid) {
                            std::cmp::Ordering::Less => low = middle + 1,
                            std::cmp::Ordering::Greater => high = middle,
                            std::cmp::Ordering::Equal => return Err(ContractError::Binding),
                        }
                    }
                    let shifted = seen.len() - low;
                    ledger.visits(shifted)?;
                    ledger.bytes(
                        shifted
                            .checked_mul(std::mem::size_of::<Uuid>())
                            .ok_or(ContractError::Capacity)?,
                    )?;
                    if seen.len() == seen.capacity() {
                        return Err(ledger.capacity());
                    }
                    seen.insert(low, uuid);
                }
                uuid_definitions(row, seen, ledger, control)?;
            }
        }
        Value::Array(rows) => {
            for row in rows {
                uuid_definitions(row, seen, ledger, control)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Precharge the maintained native inventory's repeated subtree fingerprint work.
fn inventory_admission(
    value: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut nodes = 0;
    tree(value, 0, &mut nodes, ledger, control)?;
    ledger.visits(nodes.checked_mul(64).ok_or(ContractError::Capacity)?)?;
    // Existing native canonical subtree hashing may revisit complete scalar/key
    // extents at every admitted nesting level. A nonretaining count includes
    // JSON escaping, not a guessed raw-field multiplier.
    let mut sink = Counter { length: 0, ledger, control };
    serde_json::to_writer(&mut sink, value).map_err(|_| sink.ledger.failure())?;
    let encoded = sink.length;
    sink.ledger.bytes(encoded.checked_mul(64).ok_or(ContractError::Capacity)?)?;
    sink.ledger.derived(
        encoded
            .checked_mul(64)
            .and_then(|n| n.checked_add(nodes.checked_mul(256)?))
            .ok_or(ContractError::Capacity)?,
    )
}

/// Borrow actual once-captured resource/companion originals into the maintained loader.
fn load_resource(
    capture: &ReviewCapture,
    locator: &Locator,
    indices: &[Option<usize>],
    base: &Path,
    resource: &manifest::ResourceManifest,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<inventory::LoadedResource, ContractError> {
    let path = join_relative(base, &resource.artifact, ledger, control)?;
    let expected = match resource.resource_type {
        manifest::ResourceType::Catalog => CaptureRole::Catalog,
        manifest::ResourceType::Profile => CaptureRole::Profile,
    };
    let slot = path_slot(locator, &path, expected, ledger, control)?;
    let raw = capture.bytes(indices[slot].ok_or(ContractError::Invalid)?)?;
    admit_decode(raw, MAX_SOURCE_FILE, ledger, control)?;
    let companion = match &resource.resolved_catalog {
        Some(path) => {
            let path = join_relative(base, path, ledger, control)?;
            let slot = path_slot(locator, &path, CaptureRole::Catalog, ledger, control)?;
            let raw = capture.bytes(indices[slot].ok_or(ContractError::Invalid)?)?;
            admit_decode(raw, MAX_SOURCE_FILE, ledger, control)?;
            Some(raw)
        }
        None => None,
    };
    let root_extent = capture.root().as_os_str().as_encoded_bytes().len();
    ledger.derived(
        root_extent
            .checked_add(base.as_os_str().as_encoded_bytes().len())
            .and_then(|n| n.checked_add(128))
            .ok_or(ContractError::Capacity)?,
    )?;
    let actual_base = capture.root().join(base.parent().ok_or(ContractError::Invalid)?);
    // Account the complete actual resource declaration before maintained loader
    // metadata clones, expected-pin comparisons and snapshot checking. This is
    // conservative logical storage/work, not a literal heap bound.
    let mut declaration = Counter { length: 0, ledger, control };
    serde_json::to_writer(&mut declaration, resource).map_err(|_| declaration.ledger.failure())?;
    let declaration_extent = declaration.length;
    declaration.ledger.bytes(declaration_extent.checked_mul(8).ok_or(ContractError::Capacity)?)?;
    declaration.ledger.derived(
        declaration_extent
            .checked_mul(8)
            .and_then(|n| n.checked_add(8192))
            .ok_or(ContractError::Capacity)?,
    )?;
    let loaded = inventory::load_captured_admitted(
        &actual_base,
        "private review mapping resource",
        resource,
        raw,
        companion,
        &mut |value| inventory_admission(value, ledger, control),
    )
    .map_err(|error| match error {
        inventory::CapturedInventoryError::Admission(error) => error,
        inventory::CapturedInventoryError::Domain(_) => ContractError::Binding,
    })?;
    ledger.checkpoint(control)?;
    Ok(loaded)
}

/// Preserve the exact typed model callback and single shared capacity/control owner.
fn model_charge(
    charge: model::BuildCharge,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    match charge {
        model::BuildCharge::Checkpoint => Ok(()),
        model::BuildCharge::Work { visits, byte_work } => {
            ledger.visits(visits)?;
            ledger.bytes(byte_work)
        }
        model::BuildCharge::Reserve { logical_bytes } => ledger.derived(logical_bytes),
    }
}

/// Compare complete native values with all actual key/string operands charged.
fn equal_values(
    left: &Value,
    right: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }
            for (key, value) in a {
                let mut found = None;
                for (other, candidate) in b {
                    ledger.checkpoint(control)?;
                    if compare(key, other, ledger)? == std::cmp::Ordering::Equal {
                        found = Some(candidate);
                        break;
                    }
                }
                if !equal_values(value, found.ok_or(ContractError::Binding)?, ledger, control)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }
            for (a, b) in a.iter().zip(b) {
                if !equal_values(a, b, ledger, control)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Value::String(a), Value::String(b)) => {
            Ok(compare(a, b, ledger)? == std::cmp::Ordering::Equal)
        }
        (Value::Number(a), Value::Number(b)) => {
            ledger.bytes(std::mem::size_of::<serde_json::Number>() * 2)?;
            Ok(a == b)
        }
        (Value::Bool(a), Value::Bool(b)) => Ok(a == b),
        (Value::Null, Value::Null) => Ok(true),
        _ => Ok(false),
    }
}

/// Reconstruct the complete observed lifecycle tuple from actual original pins.
fn current_artifacts(
    base: &Path,
    locator: &Locator,
    pins: &[SourcePin],
    natives: &[Option<NativeOriginal>],
    record: &LifecycleRecord,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentArtifacts, ContractError> {
    let source = fingerprint_slot(base, &record.policy.source, locator, ledger, control)?;
    check_fingerprint_identity(&record.policy.source, natives[source].as_ref(), ledger)?;
    ledger.derived(
        record
            .policy
            .generated_artifacts
            .len()
            .checked_mul(8192)
            .and_then(|n| n.checked_add(256))
            .ok_or(ContractError::Capacity)?,
    )?;
    let mut generated = Vec::with_capacity(record.policy.generated_artifacts.len());
    for fingerprint in &record.policy.generated_artifacts {
        ledger.checkpoint(control)?;
        let slot = fingerprint_slot(base, fingerprint, locator, ledger, control)?;
        let native = natives[slot].as_ref().ok_or(ContractError::Binding)?;
        check_fingerprint_identity(fingerprint, Some(native), ledger)?;
        generated.push(NamedHash {
            path: fingerprint.path.clone(),
            sha256: pins[slot].raw_sha256.clone(),
        });
    }
    // Existing lifecycle record validation requires generated paths sorted, and
    // status independently checks complete ordered bijection. No sorting prefix.
    Ok(CurrentArtifacts {
        fingerprints: FingerprintSet {
            source_sha256: pins[source].raw_sha256.clone(),
            generated_artifacts: generated,
        },
        identity_changes: Vec::new(),
    })
}

/// Resolve one complete lifecycle dependency using original record-relative spelling.
fn fingerprint_slot(
    base: &Path,
    fingerprint: &ArtifactFingerprint,
    locator: &Locator,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let path = join_relative(base, Path::new(&fingerprint.path), ledger, control)?;
    let mut found = None;
    for (slot, row) in locator.sources.iter().enumerate() {
        if same_path(&row.path, &path, ledger, control)? && found.replace(slot).is_some() {
            return Err(ContractError::Binding);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Bind actual native model/root identity, preserving source identity absence rules.
fn check_fingerprint_identity(
    fingerprint: &ArtifactFingerprint,
    native: Option<&NativeOriginal>,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    match (&fingerprint.oscal_type, &fingerprint.root_uuid, native) {
        (None, None, _) => Ok(()),
        (Some(kind), Some(uuid), Some(native))
            if compare(kind, native.model.as_str(), ledger)? == std::cmp::Ordering::Equal
                && compare(uuid, &native.root_uuid, ledger)? == std::cmp::Ordering::Equal =>
        {
            Ok(())
        }
        _ => Err(ContractError::Binding),
    }
}

/// Nonretaining canonical JSON byte counter using the actual shared ledger/control.
struct Counter<'a> {
    /// Exact encoded extent, never serialized into a retained buffer.
    length: usize,
    /// Sole monotonic logical/byte/visit owner.
    ledger: &'a mut ContractLedger,
    /// Sole accepted deadline and sticky cancellation owner.
    control: &'a mut dyn WorkControl,
}

impl Write for Counter<'_> {
    /// Admit every exact JSON chunk before nonretaining counting.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.ledger
            .checkpoint(self.control)
            .and_then(|()| self.ledger.bytes(bytes.len()))
            .map_err(|_| std::io::Error::other("private count refused"))?;
        self.length = self.length.checked_add(bytes.len()).ok_or_else(|| {
            self.ledger.capacity();
            std::io::Error::other("private count refused")
        })?;
        Ok(bytes.len())
    }
    /// Observe the same caller stop at complete serialization.
    fn flush(&mut self) -> std::io::Result<()> {
        self.ledger
            .checkpoint(self.control)
            .map_err(|_| std::io::Error::other("private count refused"))
    }
}

#[cfg(test)]
/// Genuine native originals and declared legacy lifecycle history are test facts only.
mod tests {
    use super::*;
    use crate::workspace::preparation::NoopControl;
    use serde_json::json;

    /// Own a real qualified temporary tree and complete native fixture inputs.
    struct Fixture {
        /// Actual owned temporary directory, never a synthetic capture proof.
        directory: tempfile::TempDir,
        /// Actual canonical native root used only by test orchestration.
        root: PathBuf,
    }

    /// Complete redistributable native Catalog, with no human authority assertion.
    fn catalog(uuid: &str, id: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"catalog":{"uuid":uuid,"metadata":{
            "title":"Synthetic captured Mapping input","last-modified":"2026-08-22T17:00:00Z",
            "version":"1.0.0","oscal-version":"1.2.3"},"controls":[{"id":id,"title":"Synthetic control"}]}})).unwrap()
    }

    /// Rebuild through actual maintained parser/native inventory/model, with no file proof stand-in.
    fn rebuild_native(root: &Path) {
        let bytes = std::fs::read(root.join("manifest.json")).unwrap();
        let manifest = manifest::parse(&bytes).unwrap();
        let source = std::fs::read(root.join(&manifest.mapping.source.artifact)).unwrap();
        let target = std::fs::read(root.join(&manifest.mapping.target.artifact)).unwrap();
        let companion = manifest
            .mapping
            .source
            .resolved_catalog
            .as_ref()
            .map(|path| std::fs::read(root.join(path)).unwrap());
        let mut admit = |_value: &Value| Ok::<(), std::convert::Infallible>(());
        let source = inventory::load_captured_admitted(
            root,
            "synthetic source",
            &manifest.mapping.source,
            &source,
            companion.as_deref(),
            &mut admit,
        )
        .unwrap_or_else(|_| panic!("synthetic source must be native valid"));
        let target = inventory::load_captured_admitted(
            root,
            "synthetic target",
            &manifest.mapping.target,
            &target,
            None,
            &mut admit,
        )
        .unwrap_or_else(|_| panic!("synthetic target must be native valid"));
        let product = model::build(&manifest, &source, &target, false).unwrap();
        let value = serde_json::to_value(&product.artifact).unwrap();
        assert!(validate::validate_artifact(&value, OscalModelType::Mapping).unwrap().is_valid);
        std::fs::write(root.join("mapping.json"), serde_json::to_vec(&value).unwrap()).unwrap();
    }

    /// Write intrinsically validated, explicitly synthetic recorded approval history.
    /// It does not claim an authenticated owner, D064 disposition or real approval.
    fn recorded_approval(root: &Path, profile: bool) {
        let manifest_raw = std::fs::read(root.join("manifest.json")).unwrap();
        let mut artifacts = vec![
            ("mapping.json", "mapping-collection"),
            ("source.json", if profile { "profile" } else { "catalog" }),
            ("target.json", "catalog"),
        ];
        if profile {
            artifacts.push(("source-resolved.json", "catalog"));
        }
        artifacts.sort_by_key(|row| row.0);
        let generated: Vec<Value> = artifacts.into_iter().map(|(path,kind)| {
            let raw = std::fs::read(root.join(path)).unwrap(); let value: Value = serde_json::from_slice(&raw).unwrap();
            let key = kind;
            json!({"path":path,"sha256":sha256_hex(&raw),"oscal_type":kind,"root_uuid":value[key]["uuid"]})
        }).collect();
        let fingerprints = json!({"source_sha256":sha256_hex(&manifest_raw),"generated_artifacts":generated.iter().map(|row| json!({"path":row["path"],"sha256":row["sha256"]})).collect::<Vec<_>>()});
        let value = json!({"schema_version":record::LEGACY_SCHEMA_VERSION,"policy":{
            "policy_key":"synthetic-mapping","version_key":"v1","title":"Synthetic declared approval",
            "owner_keys":["owner"],"source":{"path":"manifest.json","sha256":sha256_hex(&manifest_raw)},"generated_artifacts":generated},
            "parties":[{"key":"owner","roles":["owner"]},{"key":"reviewer","roles":["reviewer"]},{"key":"approver","roles":["approver"]}],
            "approval_policy":{"schema_version":record::APPROVAL_POLICY_VERSION,"required_roles":[{"role":"approver","count":1}],"separation":{}},
            "review":{"cadence_days":30,"next_review_date":"2026-12-01","due_soon_days":7,"timezone_policy":"date-only"},
            "state":"approved","history":[
                {"sequence":1,"event_id":"pending","previous_state":"draft","next_state":"in-review","actor_key":"reviewer","declared_role":"reviewer","timestamp":"2026-08-22T17:01:00Z","rationale":"Synthetic review only.","fingerprints":fingerprints},
                {"sequence":2,"event_id":"pending","previous_state":"in-review","next_state":"approved","actor_key":"approver","declared_role":"approver","timestamp":"2026-08-22T17:02:00Z","rationale":"Synthetic declared approval only.","fingerprints":fingerprints}]});
        let mut record: LifecycleRecord = serde_json::from_value(value).unwrap();
        for index in 0..record.history.len() {
            record.history[index].event_id =
                record::event_id(&record, &record.history[index]).unwrap();
        }
        record::validate(&record).unwrap();
        std::fs::write(root.join("lifecycle.json"), serde_json::to_vec(&record).unwrap()).unwrap();
    }

    /// Write a complete key-sorted locator; every actual path is explicit.
    fn locator(root: &Path, profile: bool) {
        let mut rows = vec![
            json!({"key":"lifecycle","path":"lifecycle.json","model":"lifecycle-record"}),
            json!({"key":"manifest","path":"manifest.json","model":"mapping-manifest"}),
            json!({"key":"mapping","path":"mapping.json","model":"mapping"}),
            json!({"key":"source","path":"source.json","model":if profile {"profile"} else {"catalog"}}),
            json!({"key":"target","path":"target.json","model":"catalog"}),
        ];
        if profile {
            rows.insert(4,json!({"key":"source-resolved","path":"source-resolved.json","model":"resolved-catalog"}));
        }
        std::fs::write(root.join("locator.json"),serde_json::to_vec(&json!({"schema_version":LOCATOR_SCHEMA,
            "mapping_manifest_key":"manifest","lifecycle_record_key":"lifecycle","mapping_key":"mapping","sources":rows})).unwrap()).unwrap();
    }

    /// Build all actual native and recorded originals before any factory capture.
    fn fixture(profile: bool) -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let source = catalog("11111111-1111-4111-8111-111111111111", "a-1");
        std::fs::write(
            root.join(if profile { "source-resolved.json" } else { "source.json" }),
            &source,
        )
        .unwrap();
        std::fs::write(
            root.join("target.json"),
            catalog("22222222-2222-4222-8222-222222222222", "b-1"),
        )
        .unwrap();
        let resource = if profile {
            std::fs::write(root.join("source.json"),serde_json::to_vec(&json!({"profile":{"uuid":"33333333-3333-4333-8333-333333333333",
                "metadata":{"title":"Synthetic Profile","last-modified":"2026-08-22T17:00:00Z","version":"1.0.0","oscal-version":"1.2.3"},
                "imports":[{"href":"source-resolved.json","include-all":{}}]}})).unwrap()).unwrap();
            json!({"type":"profile","artifact":"source.json","href":"source.json","resolved_catalog":"source-resolved.json","resolved_catalog_attestation":true,"expected_resolved_catalog_sha256":sha256_hex(&source)})
        } else {
            json!({"type":"catalog","artifact":"source.json","href":"source.json"})
        };
        let manifest = json!({"schema_version":manifest::MANIFEST_SCHEMA_VERSION,
            "collection":{"key":"collection","title":"Captured native fixture","version":"1.0.0","last_modified":"2026-08-22T17:00:00Z"},
            "reviewers":[{"key":"reviewer-1","type":"person","name":"Synthetic reviewer"}],
            "provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Synthetic fixture only.","reviewer_keys":["reviewer-1"],"reviewed_at":"2026-08-22T17:00:00Z"},
            "mapping":{"key":"mapping","source":resource,"target":{"type":"catalog","artifact":"target.json","href":"target.json"},"maps":[{
                "key":"map-1","relationship":"subset-of","sources":[{"type":"control","id_ref":"a-1"}],"targets":[{"type":"control","id_ref":"b-1"}],
                "reviewer_key":"reviewer-1","reviewed_at":"2026-08-22T17:00:00Z","rationale":"Synthetic reviewed relationship."}]}});
        std::fs::write(root.join("manifest.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
        rebuild_native(&root);
        recorded_approval(&root, profile);
        locator(&root, profile);
        Fixture { directory, root }
    }

    /// Add an explicit synthetic two-reviewer request to the actual native fixture.
    /// No participant, authenticated identity or real approval is asserted by these controls.
    fn command_fixture(profile: bool) -> Fixture {
        let f = fixture(profile);
        let native: Value =
            serde_json::from_slice(&std::fs::read(f.root.join("mapping.json")).unwrap()).unwrap();
        let subject =
            native["mapping-collection"]["mappings"][0]["maps"][0]["uuid"].as_str().unwrap();
        let request = json!({"schema_version":"forge.review-init/1",
            "roles":[{"key":"reviewer"}],
            "reviewers":[{"key":"alice","role_keys":["reviewer"]},
                {"key":"author","role_keys":["reviewer"]},{"key":"bob","role_keys":["reviewer"]}],
            "policies":[{"key":"two-reviewers","seats":[{"role_key":"reviewer","count":2}],
                "substitutions":[],"abstention_rule":"nonapproving","empty_abstention_reasons":["recused"],
                "author_separation":"declared-keys"}],
            "items":[{"key":"one","subject_id":subject,"policy_key":"two-reviewers",
                "author_keys":["author"],"assignments":[{"reviewer_key":"alice","role_key":"reviewer"},
                    {"reviewer_key":"bob","role_key":"reviewer"}],"due_at":null}]});
        std::fs::write(f.root.join("request.json"), serde_json::to_vec(&request).unwrap()).unwrap();
        std::fs::write(f.root.join("rationale.txt"), b"SENSITIVE_REVIEW_RATIONALE").unwrap();
        f
    }

    /// Enter the actual public clap dispatch; every native consumer remains genuine.
    fn invoke(root: &Path, operation: &str, arguments: &[&str]) -> Result<(), crate::ForgeError> {
        use clap::Parser as _;
        let mut words =
            vec!["forge", "review", operation, "--project-root", root.to_str().unwrap()];
        words.extend_from_slice(arguments);
        let cli = crate::cli::Cli::try_parse_from(words).unwrap();
        crate::cli::execute(&cli)
    }

    /// Create one exact queue through the real CLI and genuine native approval/current gate.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn initialize(root: &Path) {
        invoke(
            root,
            "init",
            &[
                "--sources",
                "locator.json",
                "--policy",
                "request.json",
                "--queue-id",
                "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                "--created-at",
                "2026-08-22T17:05:00Z",
                "--output",
                "queue.json",
            ],
        )
        .unwrap();
    }

    /// Produce an actual immutable response with a private rationale original.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn respond(
        root: &Path,
        reviewer: &str,
        disposition: &str,
        id: &str,
        output: &str,
    ) -> Result<(), crate::ForgeError> {
        invoke(
            root,
            "respond",
            &[
                "--queue",
                "queue.json",
                "--item-key",
                "one",
                "--reviewer-key",
                reviewer,
                "--reviewer-role",
                "reviewer",
                "--disposition",
                disposition,
                "--responded-at",
                "2026-08-22T17:06:00Z",
                "--response-id",
                id,
                "--rationale-file",
                "rationale.txt",
                "--output",
                output,
            ],
        )
    }

    /// Reach the real current JSON writer using the exact already-owned path declarations.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn status(
        root: &Path,
        responses: &[PathBuf],
        writer: &mut dyn std::io::Write,
    ) -> Result<(), super::super::commands::CommandError> {
        super::super::commands::status(
            &super::super::commands::CurrentOptions {
                project_root: root,
                sources: Path::new("locator.json"),
                queue: Path::new("queue.json"),
                responses,
                as_of: "2026-08-22T17:07:00Z",
            },
            writer,
            &mut NoopControl,
        )
    }

    /// Actual native preparation, exact response multiplicity, current finalization,
    /// bounded encoding, guarded publication and recorded HTML form one complete pipeline.
    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn genuine_cli_pipeline_preserves_duplicate_denominators_privacy_and_domain_bytes() {
        let f = command_fixture(true);
        let names = [
            "source.json",
            "source-resolved.json",
            "target.json",
            "manifest.json",
            "mapping.json",
            "lifecycle.json",
            "locator.json",
            "request.json",
            "rationale.txt",
        ];
        let originals: Vec<_> =
            names.iter().map(|name| (*name, std::fs::read(f.root.join(name)).unwrap())).collect();
        initialize(&f.root);
        respond(&f.root, "alice", "approve", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "alice.json")
            .unwrap();
        respond(&f.root, "bob", "approve", "cccccccc-cccc-4ccc-8ccc-cccccccccccc", "bob.json")
            .unwrap();
        invoke(
            &f.root,
            "merge",
            &[
                "--sources",
                "locator.json",
                "--queue",
                "queue.json",
                "--response",
                "alice.json",
                "--response",
                "alice.json",
                "--response",
                "bob.json",
                "--as-of",
                "2026-08-22T17:07:00Z",
                "--output",
                "dispositions.json",
            ],
        )
        .unwrap();
        let raw = std::fs::read(f.root.join("dispositions.json")).unwrap();
        let parsed = super::super::decode::decode_dispositions(
            &raw,
            &mut ContractLedger::default(),
            &mut NoopControl,
        )
        .unwrap();
        let document = parsed.document();
        assert_eq!(document.currentness, super::super::wire::RecordedCurrentness::RecordedCurrent);
        assert_eq!(document.items[0].state, super::super::wire::ItemState::QuorumMet);
        assert_eq!(document.items[0].required_seats, 2);
        assert_eq!(document.items[0].met_seats.len(), 2);
        assert_eq!(document.counts.response_files, 3);
        assert_eq!(document.counts.unique_responses, 2);
        assert_eq!(document.counts.exact_duplicates, 1);
        assert_eq!(document.source_pins.len(), 6);
        assert!(raw.ends_with(b"\n"));
        let responses =
            [PathBuf::from("alice.json"), PathBuf::from("alice.json"), PathBuf::from("bob.json")];
        let mut current = Vec::new();
        status(&f.root, &responses, &mut current).unwrap();
        assert_eq!(current, raw);
        invoke(
            &f.root,
            "export-html",
            &["--dispositions", "dispositions.json", "--output", "review.html"],
        )
        .unwrap();
        let html = std::fs::read_to_string(f.root.join("review.html")).unwrap();
        assert!(html.contains("recorded"));
        assert!(!html.contains("SENSITIVE_REVIEW_RATIONALE"));
        assert!(!html.contains("Synthetic reviewer"));
        assert!(!String::from_utf8(raw).unwrap().contains("SENSITIVE_REVIEW_RATIONALE"));
        for (name, original) in originals {
            assert_eq!(std::fs::read(f.root.join(name)).unwrap(), original, "{name}");
        }
        let queue = std::fs::read(f.root.join("queue.json")).unwrap();
        assert!(
            invoke(
                &f.root,
                "init",
                &[
                    "--sources",
                    "locator.json",
                    "--policy",
                    "request.json",
                    "--queue-id",
                    "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                    "--created-at",
                    "2026-08-22T17:05:00Z",
                    "--output",
                    "queue.json"
                ]
            )
            .is_err()
        );
        assert_eq!(std::fs::read(f.root.join("queue.json")).unwrap(), queue);
    }

    /// An actual independent rejection survives current finalization and prevents quorum.
    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn genuine_current_commands_preserve_dissent_without_domain_promotion() {
        let f = command_fixture(false);
        initialize(&f.root);
        respond(&f.root, "alice", "approve", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "alice.json")
            .unwrap();
        respond(&f.root, "bob", "reject", "cccccccc-cccc-4ccc-8ccc-cccccccccccc", "bob.json")
            .unwrap();
        let responses = [PathBuf::from("alice.json"), PathBuf::from("bob.json")];
        let mut output = Vec::new();
        status(&f.root, &responses, &mut output).unwrap();
        let closed = super::super::decode::decode_dispositions(
            &output,
            &mut ContractLedger::default(),
            &mut NoopControl,
        )
        .unwrap();
        let item = &closed.document().items[0];
        assert_eq!(item.state, super::super::wire::ItemState::Conflicted);
        assert_eq!(item.response_ids.len(), 2);
        assert_eq!(item.dissent_ids, vec!["cccccccc-cccc-4ccc-8ccc-cccccccccccc"]);
        assert!(item.blocking);
        assert_eq!(closed.document().counts.states.quorum_met, 0);
    }

    /// Changed valid source bytes cannot be transferred into old queue/response success or output.
    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn genuine_source_drift_refuses_current_output_and_preserves_existing_responses() {
        use std::io::Write as _;
        let f = command_fixture(false);
        initialize(&f.root);
        respond(&f.root, "alice", "approve", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "alice.json")
            .unwrap();
        let response = std::fs::read(f.root.join("alice.json")).unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .open(f.root.join("source.json"))
            .unwrap()
            .write_all(b" ")
            .unwrap();
        let mut output = Vec::new();
        assert!(status(&f.root, &[PathBuf::from("alice.json")], &mut output).is_err());
        assert_eq!(output, [] as [u8; 0]);
        assert!(
            invoke(
                &f.root,
                "merge",
                &[
                    "--sources",
                    "locator.json",
                    "--queue",
                    "queue.json",
                    "--response",
                    "alice.json",
                    "--as-of",
                    "2026-08-22T17:07:00Z",
                    "--output",
                    "refused.json"
                ]
            )
            .is_err()
        );
        assert!(!f.root.join("refused.json").exists());
        assert_eq!(std::fs::read(f.root.join("alice.json")).unwrap(), response);
    }

    /// Actual author and unknown reviewer assertions cannot produce a successful response file.
    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn genuine_response_command_refuses_author_and_unassigned_asserted_keys() {
        let f = command_fixture(false);
        initialize(&f.root);
        for reviewer in ["author", "unknown"] {
            assert!(
                respond(
                    &f.root,
                    reviewer,
                    "approve",
                    "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
                    "refused.json"
                )
                .is_err()
            );
            assert!(!f.root.join("refused.json").exists());
        }
    }

    /// Windows reaches the genuine native closure but refuses unsupported file publication.
    #[test]
    #[cfg(target_os = "windows")]
    fn genuine_init_refuses_unsupported_windows_publication_without_output() {
        let f = command_fixture(false);
        let result = invoke(
            &f.root,
            "init",
            &[
                "--sources",
                "locator.json",
                "--policy",
                "request.json",
                "--queue-id",
                "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
                "--created-at",
                "2026-08-22T17:05:00Z",
                "--output",
                "queue.json",
            ],
        );
        assert!(matches!(result, Err(crate::ForgeError::Io(_))));
        assert!(!f.root.join("queue.json").exists());
    }

    /// Use real caller capture, locator original, source leases and final proof owner.
    fn pending(
        root: &Path,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(PendingMappingClosure, Rc<HeldReviewInputs>), ContractError> {
        let mut capture = ReviewCapture::new(root, &[], ledger, control)?;
        let index = capture.required(
            Path::new("locator.json"),
            CaptureRole::ReviewPrivateConfig,
            Pool::Auxiliary,
            MAX_LOCATOR_BYTES,
            ledger,
            control,
        )?;
        let pending = prepare(&mut capture, index, ledger, control)?;
        Ok((pending, Rc::new(capture.finish())))
    }

    /// Genuine complete Mapping/lifecycle closure seals and preserves every original pin.
    #[test]
    fn genuine_recorded_current_mapping_closure_seals_complete_union() {
        let f = fixture(false);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let (pending, held) = pending(&f.root, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("actual fixture must prepare"));
        let closure = pending
            .seal(held, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("actual fixture must seal"));
        assert_eq!(closure.source_pins().len(), 5);
        let actual: Value =
            serde_json::from_slice(&std::fs::read(f.root.join("mapping.json")).unwrap()).unwrap();
        assert_eq!(closure.native_value(), &actual);
        closure.verify_inputs(&mut ledger, &mut control).unwrap();
        assert!(f.directory.path().exists());
    }

    /// Full native changes refuse even with unchanged stable UUIDs and valid schema.
    #[test]
    fn whole_original_mapping_value_mismatch_refuses() {
        let f = fixture(false);
        let path = f.root.join("mapping.json");
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["mapping-collection"]["mappings"][0]["maps"][0]["remarks"] =
            json!("Different actual original rationale");
        assert!(validate::validate_artifact(&value, OscalModelType::Mapping).unwrap().is_valid);
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        assert!(matches!(pending(&f.root, &mut ledger, &mut control), Err(ContractError::Binding)));
    }

    /// A genuine valid in-review record without latest approval cannot create success.
    #[test]
    fn no_current_recorded_approved_history_refuses() {
        let f = fixture(false);
        let path = f.root.join("lifecycle.json");
        let mut record = record::parse(&std::fs::read(&path).unwrap()).unwrap();
        record.history.pop();
        record.state = LifecycleState::InReview;
        record::validate(&record).unwrap();
        std::fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        assert!(matches!(pending(&f.root, &mut ledger, &mut control), Err(ContractError::Binding)));
    }

    /// Recomputed valid native input still refuses against the old actual approved tuple.
    #[test]
    fn actual_recomputed_mapping_does_not_replace_old_approved_fingerprints() {
        let f = fixture(false);
        let path = f.root.join("source.json");
        let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["catalog"]["metadata"]["title"] = json!("Different current source metadata");
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        rebuild_native(&f.root);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        assert!(matches!(pending(&f.root, &mut ledger, &mut control), Err(ContractError::Binding)));
    }

    /// Complete declared closure refuses missing and extra originals, never a retained prefix.
    #[test]
    fn missing_or_extra_declared_original_union_refuses() {
        for extra in [false, true] {
            let f = fixture(false);
            let path = f.root.join("locator.json");
            let mut value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            if extra {
                std::fs::write(
                    f.root.join("extra.json"),
                    catalog("44444444-4444-4444-8444-444444444444", "extra"),
                )
                .unwrap();
                value["sources"]
                    .as_array_mut()
                    .unwrap()
                    .insert(0, json!({"key":"extra","path":"extra.json","model":"catalog"}));
            } else {
                value["sources"].as_array_mut().unwrap().pop();
            }
            std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            let mut ledger = ContractLedger::default();
            let mut control = NoopControl;
            assert!(matches!(
                pending(&f.root, &mut ledger, &mut control),
                Err(ContractError::Binding)
            ));
        }
    }

    /// Genuine Profile closure uses the actual companion and refuses its changed pin.
    #[test]
    fn complete_profile_companion_is_required_and_actual_drift_refused() {
        let f = fixture(true);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let (prepared, held) = pending(&f.root, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("Profile fixture must prepare"));
        let closure = prepared
            .seal(held, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("Profile fixture must seal"));
        assert_eq!(closure.source_pins().len(), 6);
        drop(closure);
        let path = f.root.join("source-resolved.json");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.push(b' ');
        std::fs::write(path, bytes).unwrap();
        let mut ledger = ContractLedger::default();
        assert!(matches!(pending(&f.root, &mut ledger, &mut control), Err(ContractError::Binding)));
    }

    /// Byte-equal but independently captured Rc owners never substitute for the actual command.
    #[test]
    fn other_real_capture_with_equal_bytes_cannot_seal_pending() {
        let f = fixture(false);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let (first, first_held) = pending(&f.root, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("first capture must prepare"));
        let (other, other_held) = pending(&f.root, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("second capture must prepare"));
        drop(other);
        assert!(matches!(
            first.seal(other_held, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        drop(first_held);
    }

    /// An additional actual Source-pool original cannot hide outside the complete locator union.
    #[test]
    fn extra_actual_source_original_after_prepare_cannot_seal() {
        let f = fixture(false);
        std::fs::write(
            f.root.join("extra.json"),
            catalog("44444444-4444-4444-8444-444444444444", "extra"),
        )
        .unwrap();
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let mut capture = ReviewCapture::new(&f.root, &[], &mut ledger, &mut control).unwrap();
        let locator = capture
            .required(
                Path::new("locator.json"),
                CaptureRole::ReviewPrivateConfig,
                Pool::Auxiliary,
                MAX_LOCATOR_BYTES,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let prepared = prepare(&mut capture, locator, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("complete actual fixture must prepare"));
        capture
            .required(
                Path::new("extra.json"),
                CaptureRole::Catalog,
                Pool::Source,
                MAX_SOURCE_FILE,
                &mut ledger,
                &mut control,
            )
            .unwrap();
        let held = Rc::new(capture.finish());
        assert!(matches!(
            prepared.seal(held, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
    }

    #[cfg(unix)]
    /// A real post-preparation tail-byte change refuses at the complete held original fence.
    #[test]
    fn actual_late_original_tail_change_refuses_seal() {
        let f = fixture(false);
        let mut ledger = ContractLedger::default();
        let mut control = NoopControl;
        let (prepared, held) = pending(&f.root, &mut ledger, &mut control)
            .unwrap_or_else(|_| panic!("fixture must prepare"));
        let mut file =
            std::fs::OpenOptions::new().append(true).open(f.root.join("source.json")).unwrap();
        file.write_all(b" ").unwrap();
        drop(file);
        assert!(matches!(
            prepared.seal(held, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
    }
}

#[cfg(test)]
/// Exact generated serialization/admission controls; no native approval proof is constructed.
mod generated_admission_tests {
    use super::*;
    use crate::workspace::preparation::{Interruption, NoopControl, Stage, test_support::Recorder};
    use std::cell::Cell;

    /// Count actual Serialize invocations to prove refusal precedes comparison Value construction.
    struct Probe<'a> {
        /// Fixed borrowed payload, no allocation made by the fixture serializer.
        text: &'a str,
        /// Actual reached complete serialization passes.
        calls: &'a Cell<usize>,
    }

    impl serde::Serialize for Probe<'_> {
        /// Use the actual serde string encoding for both count and Value construction.
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.calls.set(self.calls.get() + 1);
            serializer.serialize_str(self.text)
        }
    }

    /// Exact logical capacity succeeds, one byte less refuses before the second pass;
    /// exhausted repeated-byte work also refuses without comparison allocation.
    #[test]
    fn generated_complete_admission_precedes_second_serialization() {
        let text = "x".repeat(1024);
        let encoded = 1026;
        let allowance = encoded * 16 + 512;
        for logical_shortfall in [0, 1] {
            let calls = Cell::new(0);
            let mut ledger = ContractLedger::default();
            ledger.derived(33_554_432 - allowance + logical_shortfall).unwrap();
            let result = comparison_value(
                &Probe { text: &text, calls: &calls },
                &mut ledger,
                &mut NoopControl,
            );
            if logical_shortfall == 0 {
                assert_eq!(result.unwrap(), Value::String(text.clone()));
                assert_eq!(calls.get(), 2);
            } else {
                assert_eq!(result, Err(ContractError::Capacity));
                assert_eq!(calls.get(), 1);
                assert_eq!(
                    comparison_value(
                        &Probe { text: &text, calls: &calls },
                        &mut ledger,
                        &mut NoopControl
                    ),
                    Err(ContractError::Capacity)
                );
                assert_eq!(calls.get(), 1);
            }
        }
        let calls = Cell::new(0);
        let mut ledger = ContractLedger::default();
        ledger.bytes(268_435_456 - (2 * encoded - 1)).unwrap();
        assert_eq!(
            comparison_value(&Probe { text: &text, calls: &calls }, &mut ledger, &mut NoopControl),
            Err(ContractError::Capacity)
        );
        assert_eq!(calls.get(), 1);
    }

    /// A reached counting-writer stop never reaches the comparison allocation and cannot be reset.
    #[test]
    fn generated_count_control_stop_preserves_first_reason_before_value_growth() {
        let calls = Cell::new(0);
        let probe = Probe { text: "actual bounded text", calls: &calls };
        let mut ledger = ContractLedger::default();
        let mut stop = Recorder::at(Stage::PrepareDomain, 2);
        let expected = Err(ContractError::Interrupted(Interruption::CancelRequested));
        assert_eq!(comparison_value(&probe, &mut ledger, &mut stop), expected);
        assert_eq!(calls.get(), 1);
        assert_eq!(comparison_value(&probe, &mut ledger, &mut NoopControl), expected);
        assert_eq!(calls.get(), 1);
    }
}

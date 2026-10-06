//! Actual complete report-source applicability closure for portable re-review.
//!
//! One shared capture supplies every native, manifest, report and lifecycle original.
//! Maintained admitted applicability semantics establish domain equality; maintained
//! neutral lifecycle status establishes recorded current approval. Neither locator
//! declarations, schema, hashes nor regenerated reports construct authority.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde::Deserialize;
use serde_json::Value;

use crate::applicability::captured::admitted;
use crate::applicability::{captured, manifest, model};
use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{self, FingerprintSet, LifecycleRecord, LifecycleState, NamedHash};
use crate::lifecycle::status::{CurrentArtifacts, status_from_captured};
use crate::mapping::inventory::ResourceEvidence;
use crate::mapping::manifest::ResourceType;
use crate::validate::OscalModelType;
use crate::workspace::preparation::{WorkControl, WorkError};

use super::capture::{HeldReviewInputs, Pool, ReviewCapture};
use super::chain::reserved;
use super::decode::{ContractError, ContractLedger};
use super::mapping_capture::{self as shared, NativeOriginal};
use super::wire::{SourceModel, SourcePin};

/// Private declaration marker, never a disclosure or lifecycle approval token.
const LOCATOR_SCHEMA: &str = "forge.review-applicability-locator/1";
/// Whole distinct Source-original roster; registrations remain capture-owned.
const MAX_SOURCES: usize = 100;
/// Individual native, applicability source and stored report raw ceiling.
const MAX_SOURCE_FILE: usize = 10 * 1024 * 1024;
/// Complete Source-pool original extent, independent of decoded representations.
const MAX_SOURCE_BYTES: usize = 50 * 1024 * 1024;
/// Closed private auxiliary locator ceiling.
const MAX_LOCATOR_BYTES: usize = 1024 * 1024;
/// Complete bounded capture scan before the final Source-original count.
const MAX_HELD_ORIGINALS: usize = 10_105;

/// Complete closed declaration of original inputs, without claimed approval facts.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Locator {
    /// Exact selected private format marker.
    schema_version: String,
    /// Actual recorded applicability manifest original key.
    applicability_manifest_key: String,
    /// Actual full unfiltered report original key and lifecycle source.
    applicability_report_key: String,
    /// Actual complete recorded lifecycle original key.
    lifecycle_record_key: String,
    /// Actual selected native framework key.
    framework_key: String,
    /// Native Mapping keys in exact actual manifest-path order.
    mapping_keys: Vec<String>,
    /// Complete unique key-sorted physical original roster.
    sources: Vec<SourceDeclaration>,
}

/// One exact original role/path and explicit Profile companion declaration.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceDeclaration {
    /// Stable bounded private key, not a native UUID or href.
    key: String,
    /// Admitted original root-relative spelling before any composition.
    path: PathBuf,
    /// Closed supported original family, checked before capture.
    model: SourceModel,
    /// Required actual Catalog companion for Profile, explicit null otherwise.
    resolved_catalog_key: Option<String>,
}

/// One original lease from the same real capture, not a copied byte pin.
struct OriginalBinding {
    /// Actual successful capture entry index.
    index: usize,
    /// Actual privately held original allocation owner.
    lease: CaptureLease,
}

/// Complete current control facts, constructible only by this actual factory.
pub(crate) struct ControlFacts {
    /// Actual effective-framework control identifier.
    control_id: String,
    /// Actual maintained native canonical subject fingerprint.
    native_fingerprint: String,
    /// Maintained native per-edge classification.
    classification: model::GapClassification,
    /// Exact position in the complete original manifest decisions array.
    decision_index: Option<usize>,
    /// Complete positive native edge-target count, never Cartesian count.
    positive_count: usize,
    /// Complete explicit no-relationship native edge-target count.
    no_relationship_count: usize,
}

impl ControlFacts {
    /// Borrow the complete actual identifier without truncation or reinterpretation.
    pub(crate) fn control_id(&self) -> &str {
        &self.control_id
    }
    /// Borrow the actual current native fingerprint.
    pub(crate) fn native_fingerprint(&self) -> &str {
        &self.native_fingerprint
    }
    /// Return the maintained independent classification.
    pub(crate) fn classification(&self) -> model::GapClassification {
        self.classification
    }
    /// Preserve explicit source-array position or genuine omission.
    pub(crate) fn decision_index(&self) -> Option<usize> {
        self.decision_index
    }
    /// Return the complete native positive edge-target denominator.
    pub(crate) fn positive_count(&self) -> usize {
        self.positive_count
    }
    /// Return the complete native no-relationship edge-target denominator.
    pub(crate) fn no_relationship_count(&self) -> usize {
        self.no_relationship_count
    }
}

/// Private pending facts; no success, serde, Clone or Debug constructor is exposed.
pub(crate) struct PendingApplicabilityClosure {
    /// Exact strict original manifest Value, including null/array distinctions.
    manifest: Value,
    /// Complete sorted actual effective-framework control roster.
    controls: Vec<ControlFacts>,
    /// Actual framework evidence, including the private original href.
    framework: ResourceEvidence,
    /// Complete six classifications and total from the maintained engine.
    counts: model::ClassificationCounts,
    /// Complete native map denominator.
    maps: usize,
    /// Complete Cartesian locator denominator, distinct from native participation.
    pairs: usize,
    /// Complete actual original Source pins in private declaration key order.
    pins: Vec<SourcePin>,
    /// Locator and all actual source allocation bindings required at seal.
    originals: Vec<OriginalBinding>,
}

/// Genuine recorded Approved/current report-source closure retaining the full proof.
/// No detached constructor, serde, Clone or Debug is provided.
pub(crate) struct ApprovedApplicabilityClosure {
    /// Exact original manifest Value used for selected decision fingerprints.
    manifest: Value,
    /// Complete actual native and explicit/omitted decision facts.
    controls: Vec<ControlFacts>,
    /// Complete actual selected framework tuple.
    framework: ResourceEvidence,
    /// Complete maintained category counts.
    counts: model::ClassificationCounts,
    /// Every native map, not selected count.
    maps: usize,
    /// Every actual Cartesian relation locator, not per-target count.
    pairs: usize,
    /// Complete actual current Source-original roster.
    pins: Vec<SourcePin>,
    /// Sole genuine held capture owner surviving through publication fences.
    held: Rc<HeldReviewInputs>,
}

impl PendingApplicabilityClosure {
    /// Seal only the exact original leases and complete actual Source count.
    pub(crate) fn seal(
        self,
        held: Rc<HeldReviewInputs>,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<ApprovedApplicabilityClosure, ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
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
                ledger.bytes(2 * std::mem::size_of::<CaptureLease>())?;
                if !held.same_original(original.index, &original.lease)? {
                    return Err(ContractError::Binding);
                }
            }
            held.verify_inputs(ledger, control)?;
            ledger.checkpoint(control)?;
            Ok(ApprovedApplicabilityClosure {
                manifest: self.manifest,
                controls: self.controls,
                framework: self.framework,
                counts: self.counts,
                maps: self.maps,
                pairs: self.pairs,
                pins: self.pins,
                held,
            })
        })
    }
}

impl PendingApplicabilityClosure {
    /// Consume only this genuine complete factory pending result for S4 preparation.
    /// The ordinary seal retains its exact whole-Source equality.
    pub(crate) fn into_supersession_pending(
        self,
    ) -> super::capture::supersession::PendingNewNativeCohort {
        super::capture::supersession::PendingNewNativeCohort::applicability(self)
    }

    /// Consume this same genuine pending result only through the private complete S4 union issuer.
    /// The token has no caller constructor and is not returned separately from its bound closure.
    pub(super) fn seal_supersession_union(
        self,
        union: &super::capture::supersession::NativeUnionSeal,
    ) -> ApprovedApplicabilityClosure {
        ApprovedApplicabilityClosure {
            manifest: self.manifest,
            controls: self.controls,
            framework: self.framework,
            counts: self.counts,
            maps: self.maps,
            pairs: self.pairs,
            pins: self.pins,
            held: Rc::clone(union.held_inputs()),
        }
    }

    /// Borrow the complete factory-issued pin roster, never a caller-selected subset.
    pub(super) fn supersession_pins(&self) -> &[SourcePin] {
        &self.pins
    }

    /// Count every factory original, including the real captured private locator.
    pub(super) fn supersession_original_count(&self) -> usize {
        self.originals.len()
    }

    /// Borrow the genuine original locator binding retained by this factory.
    pub(super) fn supersession_locator(&self) -> Option<(usize, &CaptureLease)> {
        self.originals.first().map(|original| (original.index, &original.lease))
    }

    /// Borrow one member of the full owned roster in its unchanged factory order.
    pub(super) fn supersession_member(&self, slot: usize) -> Option<(usize, &CaptureLease)> {
        self.originals.get(slot.checked_add(1)?).map(|original| (original.index, &original.lease))
    }
}

impl ApprovedApplicabilityClosure {
    /// Borrow the complete exact current Source-original roster.
    pub(crate) fn source_pins(&self) -> &[SourcePin] {
        &self.pins
    }
    /// Borrow every actual control in maintained lexical inventory order.
    pub(crate) fn control_facts(&self) -> &[ControlFacts] {
        &self.controls
    }
    /// Borrow one complete original decision at its exact manifest-array index.
    pub(crate) fn decision_original(&self, index: usize) -> Option<&Value> {
        self.manifest.get("decisions")?.as_array()?.get(index)
    }
    /// Borrow the actual selected framework's complete resource tuple.
    pub(crate) fn framework_evidence(&self) -> &ResourceEvidence {
        &self.framework
    }
    /// Borrow complete native category counts, including every omitted control.
    pub(crate) fn complete_counts(&self) -> &model::ClassificationCounts {
        &self.counts
    }
    /// Return every admitted native map, independent of selected decisions.
    pub(crate) fn complete_maps(&self) -> usize {
        self.maps
    }
    /// Return complete Cartesian locators, independent of per-edge participation.
    pub(crate) fn complete_pair_inspections(&self) -> usize {
        self.pairs
    }
    /// Borrow the genuine held owner required for finalizer Rc identity binding.
    pub(crate) fn held_inputs(&self) -> &Rc<HeldReviewInputs> {
        &self.held
    }
    /// Recheck all actual originals and generations under the same invocation ledger.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        self.held.verify_inputs(ledger, control)
    }
}

/// Closed initial originals and once-captured complete roster; still no approval.
struct Inputs {
    /// Actual closed private locator.
    locator: Locator,
    /// Actual source indices in complete locator order.
    indices: Vec<Option<usize>>,
    /// Exact selected manifest source slot.
    manifest_slot: usize,
    /// Exact report-source slot.
    report_slot: usize,
    /// Exact lifecycle record slot.
    record_slot: usize,
    /// Exact selected framework native slot.
    framework_slot: usize,
    /// Maintained intrinsic actual manifest.
    manifest: manifest::ApplicabilityManifest,
    /// Maintained intrinsic recorded lifecycle original.
    lifecycle: LifecycleRecord,
}

/// Complete parsed originals with schema-qualified native identities.
struct Parsed {
    /// Every actual original lease, including the private locator.
    originals: Vec<OriginalBinding>,
    /// Actual complete source pins, not expected/caller-provided hashes.
    pins: Vec<SourcePin>,
    /// Complete native strict forms in original source-roster order.
    natives: Vec<Option<NativeOriginal>>,
}

/// Prepare actual report-source approval and current maintained domain facts.
/// Required role/helper/module bridges are Root-owned; no success stub exists.
pub(crate) fn prepare(
    capture: &mut ReviewCapture,
    locator_index: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingApplicabilityClosure, ContractError> {
    ledger.bound(|ledger| {
        let inputs = prepare_inputs(capture, locator_index, ledger, control)?;
        let parsed = parse_originals(capture, locator_index, &inputs, ledger, control)?;
        verify_approval(capture, &inputs, &parsed, ledger, control)?;
        let manifest_index = index(&inputs, inputs.manifest_slot)?;
        let original =
            shared::strict_value(capture.bytes(manifest_index)?, MAX_SOURCE_FILE, ledger, control);
        ledger.checkpoint(control)?;
        let original = original?;
        let mut pending = project(capture, &inputs, &parsed, original, ledger, control)?;
        pending.pins = parsed.pins;
        pending.originals = parsed.originals;
        ledger.checkpoint(control)?;
        Ok(pending)
    })
}

/// Decode exact required locator fields and spellings before source registration.
fn locator(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Locator, ContractError> {
    let value = shared::strict_value(raw, MAX_LOCATOR_BYTES, ledger, control);
    ledger.checkpoint(control)?;
    let value = value?;
    let rows = value.get("sources").and_then(Value::as_array).ok_or(ContractError::Invalid)?;
    let maps = value.get("mapping_keys").and_then(Value::as_array).ok_or(ContractError::Invalid)?;
    if !(4..=MAX_SOURCES).contains(&rows.len()) || maps.len() > MAX_SOURCES {
        return Err(ContractError::Invalid);
    }
    for row in rows {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        if row.get("resolved_catalog_key").is_none() {
            return Err(ContractError::Invalid);
        }
    }
    shared::admit_decode(raw, MAX_LOCATOR_BYTES, ledger, control)?;
    let result = serde_json::from_slice::<Locator>(raw);
    ledger.checkpoint(control)?;
    let result = result.map_err(|_| ContractError::Invalid)?;
    drop(value);
    if result.schema_version != LOCATOR_SCHEMA {
        return Err(ContractError::Invalid);
    }
    for key in [
        &result.applicability_manifest_key,
        &result.applicability_report_key,
        &result.lifecycle_record_key,
        &result.framework_key,
    ] {
        shared::token(key, ledger)?;
    }
    for (slot, row) in result.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        shared::token(&row.key, ledger)?;
        shared::portable(&row.path, ledger)?;
        if role(row.model).is_none() {
            return Err(ContractError::Invalid);
        }
        match (row.model, &row.resolved_catalog_key) {
            (SourceModel::Profile, Some(key)) => shared::token(key, ledger)?,
            (SourceModel::Profile, None) | (_, Some(_)) => return Err(ContractError::Invalid),
            (_, None) => {}
        }
        if slot > 0
            && shared::compare(&result.sources[slot - 1].key, &row.key, ledger)?
                != std::cmp::Ordering::Less
        {
            return Err(ContractError::Invalid);
        }
        for other in &result.sources[..slot] {
            ledger.checkpoint(control)?;
            if shared::folded_path(&row.path, &other.path, ledger)? {
                return Err(ContractError::Invalid);
            }
        }
    }
    for (slot, key) in result.mapping_keys.iter().enumerate() {
        shared::token(key, ledger)?;
        if result.sources[key_slot(&result, key, ledger, control)?].model != SourceModel::Mapping {
            return Err(ContractError::Invalid);
        }
        for other in &result.mapping_keys[..slot] {
            if shared::compare(other, key, ledger)?.is_eq() {
                return Err(ContractError::Invalid);
            }
        }
    }
    for row in &result.sources {
        if let Some(key) = &row.resolved_catalog_key {
            if result.sources[key_slot(&result, key, ledger, control)?].model
                != SourceModel::ResolvedCatalog
            {
                return Err(ContractError::Binding);
            }
        }
    }
    Ok(result)
}

/// Select only this closure's explicitly supported Source roles.
fn role(model: SourceModel) -> Option<CaptureRole> {
    match model {
        SourceModel::Catalog | SourceModel::ResolvedCatalog => Some(CaptureRole::Catalog),
        SourceModel::Profile => Some(CaptureRole::Profile),
        SourceModel::Mapping => Some(CaptureRole::MappingCollection),
        SourceModel::ApplicabilityManifest => Some(CaptureRole::ApplicabilityManifest),
        SourceModel::ApplicabilityReport => Some(CaptureRole::ApplicabilityReport),
        SourceModel::LifecycleRecord => Some(CaptureRole::LifecycleRecord),
        _ => None,
    }
}

/// Resolve exactly one declared source key with both complete key extents charged.
fn key_slot(
    locator: &Locator,
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (slot, row) in locator.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        if shared::compare(&row.key, key, ledger)?.is_eq() && found.replace(slot).is_some() {
            return Err(ContractError::Binding);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Resolve exactly one compatible original path without href or hash discovery.
fn path_slot(
    locator: &Locator,
    path: &Path,
    expected: CaptureRole,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (slot, row) in locator.sources.iter().enumerate() {
        if shared::same_path(&row.path, path, ledger, control)?
            && (role(row.model) != Some(expected) || found.replace(slot).is_some())
        {
            return Err(ContractError::Binding);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Borrow an actual successful source index rather than manufacture one from a key.
fn index(inputs: &Inputs, slot: usize) -> Result<usize, ContractError> {
    inputs.indices.get(slot).copied().flatten().ok_or(ContractError::Invalid)
}

/// Admit one actual Source original under its exact role and complete shared pool.
fn capture_source(
    capture: &mut ReviewCapture,
    row: &SourceDeclaration,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let limit = if row.model == SourceModel::LifecycleRecord {
        usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| ContractError::Capacity)?
    } else {
        MAX_SOURCE_FILE
    };
    capture.required(
        &row.path,
        role(row.model).ok_or(ContractError::Invalid)?,
        Pool::Source,
        limit,
        ledger,
        control,
    )
}

/// Capture manifest/record, qualify the complete required union, then capture remaining originals.
fn prepare_inputs(
    capture: &mut ReviewCapture,
    locator_index: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Inputs, ContractError> {
    ledger.checkpoint(control)?;
    if capture.role(locator_index)? != CaptureRole::ReviewPrivateConfig {
        return Err(ContractError::Invalid);
    }
    let locator = locator(capture.bytes(locator_index)?, ledger, control)?;
    let manifest_slot = key_slot(&locator, &locator.applicability_manifest_key, ledger, control)?;
    let report_slot = key_slot(&locator, &locator.applicability_report_key, ledger, control)?;
    let record_slot = key_slot(&locator, &locator.lifecycle_record_key, ledger, control)?;
    let framework_slot = key_slot(&locator, &locator.framework_key, ledger, control)?;
    if locator.sources[manifest_slot].model != SourceModel::ApplicabilityManifest
        || locator.sources[report_slot].model != SourceModel::ApplicabilityReport
        || locator.sources[record_slot].model != SourceModel::LifecycleRecord
        || !matches!(
            locator.sources[framework_slot].model,
            SourceModel::Catalog | SourceModel::Profile
        )
    {
        return Err(ContractError::Binding);
    }
    let mut indices = reserved(locator.sources.len(), ledger)?;
    indices.resize(locator.sources.len(), None);
    let manifest_index = capture_source(capture, &locator.sources[manifest_slot], ledger, control)?;
    indices[manifest_slot] = Some(manifest_index);
    let record_index = capture_source(capture, &locator.sources[record_slot], ledger, control)?;
    indices[record_slot] = Some(record_index);
    shared::admit_decode(capture.bytes(manifest_index)?, MAX_SOURCE_FILE, ledger, control)?;
    let manifest = manifest::parse(capture.bytes(manifest_index)?);
    ledger.checkpoint(control)?;
    let manifest = manifest.map_err(|_| ContractError::Invalid)?;
    shared::admit_decode(
        capture.bytes(record_index)?,
        usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| ContractError::Capacity)?,
        ledger,
        control,
    )?;
    let lifecycle = record::parse(capture.bytes(record_index)?);
    ledger.checkpoint(control)?;
    let lifecycle = lifecycle.map_err(|_| ContractError::Invalid)?;
    let mut inputs = Inputs {
        locator,
        indices,
        manifest_slot,
        report_slot,
        record_slot,
        framework_slot,
        manifest,
        lifecycle,
    };
    validate_union(capture, &inputs, ledger, control)?;
    for (slot, row) in inputs.locator.sources.iter().enumerate() {
        if inputs.indices[slot].is_none() {
            inputs.indices[slot] = Some(capture_source(capture, row, ledger, control)?);
        }
    }
    Ok(inputs)
}

/// Require exact report-as-source, manifest paths, Profile companion and generated union.
fn validate_union(
    capture: &ReviewCapture,
    inputs: &Inputs,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let manifest_path = capture.path(index(inputs, inputs.manifest_slot)?)?;
    let record_path = capture.path(index(inputs, inputs.record_slot)?)?;
    let locator = &inputs.locator;
    let mut needed = reserved(locator.sources.len(), ledger)?;
    needed.resize(locator.sources.len(), false);
    for slot in [inputs.manifest_slot, inputs.report_slot, inputs.record_slot] {
        needed[slot] = true;
    }
    let framework_path =
        shared::join_relative(manifest_path, &inputs.manifest.framework.artifact, ledger, control)?;
    if path_slot(
        locator,
        &framework_path,
        role(locator.sources[inputs.framework_slot].model).ok_or(ContractError::Binding)?,
        ledger,
        control,
    )? != inputs.framework_slot
    {
        return Err(ContractError::Binding);
    }
    needed[inputs.framework_slot] = true;
    if let Some(companion) = &inputs.manifest.framework.resolved_catalog {
        let path = shared::join_relative(manifest_path, companion, ledger, control)?;
        let slot = path_slot(locator, &path, CaptureRole::Catalog, ledger, control)?;
        let key = locator.sources[inputs.framework_slot]
            .resolved_catalog_key
            .as_deref()
            .ok_or(ContractError::Binding)?;
        if key_slot(locator, key, ledger, control)? != slot {
            return Err(ContractError::Binding);
        }
        needed[slot] = true;
    }
    if inputs.manifest.mapping_collections.len() != locator.mapping_keys.len() {
        return Err(ContractError::Binding);
    }
    for (path, key) in inputs.manifest.mapping_collections.iter().zip(&locator.mapping_keys) {
        let path = shared::join_relative(manifest_path, path, ledger, control)?;
        let slot = path_slot(locator, &path, CaptureRole::MappingCollection, ledger, control)?;
        if key_slot(locator, key, ledger, control)? != slot {
            return Err(ContractError::Binding);
        }
        needed[slot] = true;
    }
    let source = &inputs.lifecycle.policy.source;
    let path = shared::join_relative(record_path, Path::new(&source.path), ledger, control)?;
    if source.oscal_type.is_some()
        || source.root_uuid.is_some()
        || path_slot(locator, &path, CaptureRole::ApplicabilityReport, ledger, control)?
            != inputs.report_slot
    {
        return Err(ContractError::Binding);
    }
    let mut generated = reserved(locator.sources.len(), ledger)?;
    generated.resize(locator.sources.len(), false);
    for artifact in &inputs.lifecycle.policy.generated_artifacts {
        let path = shared::join_relative(record_path, Path::new(&artifact.path), ledger, control)?;
        let slot = path_slot(
            locator,
            &path,
            role_for_native(artifact.oscal_type.as_deref())?,
            ledger,
            control,
        )?;
        if generated[slot] || shared::native_model(locator.sources[slot].model).is_none() {
            return Err(ContractError::Binding);
        }
        generated[slot] = true;
        needed[slot] = true;
    }
    for (slot, row) in locator.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        if !needed[slot] || (shared::native_model(row.model).is_some() && !generated[slot]) {
            return Err(ContractError::Binding);
        }
        if let Some(key) = &row.resolved_catalog_key {
            let slot = key_slot(locator, key, ledger, control)?;
            if !generated[slot] {
                return Err(ContractError::Binding);
            }
        }
    }
    Ok(())
}

/// Require generated fingerprints to name an actually supported native model.
fn role_for_native(model: Option<&str>) -> Result<CaptureRole, ContractError> {
    match model {
        Some("catalog") => Ok(CaptureRole::Catalog),
        Some("profile") => Ok(CaptureRole::Profile),
        Some("mapping-collection") => Ok(CaptureRole::MappingCollection),
        _ => Err(ContractError::Binding),
    }
}

/// Parse every actual original and preserve its complete path-free Source pin and lease.
fn parse_originals(
    capture: &ReviewCapture,
    locator_index: usize,
    inputs: &Inputs,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Parsed, ContractError> {
    let mut originals = reserved(inputs.locator.sources.len() + 1, ledger)?;
    let mut pins = reserved(inputs.locator.sources.len(), ledger)?;
    let mut natives = reserved(inputs.locator.sources.len(), ledger)?;
    originals.push(OriginalBinding { index: locator_index, lease: capture.lease(locator_index)? });
    let mut total = 0_usize;
    for (slot, row) in inputs.locator.sources.iter().enumerate() {
        let index = index(inputs, slot)?;
        let raw = capture.bytes(index)?;
        ledger.checkpoint(control)?;
        total = total
            .checked_add(raw.len())
            .filter(|n| *n <= MAX_SOURCE_BYTES)
            .ok_or(ContractError::Capacity)?;
        let native = match row.model {
            SourceModel::Catalog | SourceModel::ResolvedCatalog => {
                shared::parse_native(raw, OscalModelType::Catalog, ledger, control).map(Some)
            }
            SourceModel::Profile => {
                shared::parse_native(raw, OscalModelType::Profile, ledger, control).map(Some)
            }
            SourceModel::Mapping => {
                shared::parse_native(raw, OscalModelType::Mapping, ledger, control).map(Some)
            }
            SourceModel::ApplicabilityManifest
            | SourceModel::ApplicabilityReport
            | SourceModel::LifecycleRecord => Ok(None),
            _ => Err(ContractError::Binding),
        };
        ledger.checkpoint(control)?;
        let native = native?;
        ledger.bytes(raw.len())?;
        ledger.derived(64)?;
        let hash = sha256_hex(raw);
        let schema_identity = match &native {
            Some(native) => {
                let extent = native
                    .oscal_version
                    .len()
                    .checked_add(native.model.as_str().len())
                    .and_then(|n| n.checked_add(7))
                    .ok_or(ContractError::Capacity)?;
                if extent > 128 {
                    return Err(ContractError::Invalid);
                }
                ledger.derived(extent)?;
                format!("oscal:{}:{}", native.oscal_version, native.model.as_str())
            }
            None => copy_text(
                match row.model {
                    SourceModel::ApplicabilityManifest => manifest::MANIFEST_SCHEMA_VERSION,
                    SourceModel::ApplicabilityReport => model::REPORT_SCHEMA_VERSION,
                    SourceModel::LifecycleRecord => &inputs.lifecycle.schema_version,
                    _ => return Err(ContractError::Binding),
                },
                ledger,
                control,
            )?,
        };
        pins.push(SourcePin {
            artifact_key: copy_text(&row.key, ledger, control)?,
            model: row.model,
            native_root_uuid: native
                .as_ref()
                .map(|value| copy_text(&value.canonical_uuid, ledger, control))
                .transpose()?,
            raw_sha256: hash,
            byte_length: u64::try_from(raw.len()).map_err(|_| ContractError::Capacity)?,
            schema_identity,
        });
        originals.push(OriginalBinding { index, lease: capture.lease(index)? });
        natives.push(native);
    }
    Ok(Parsed { originals, pins, natives })
}

/// Require original raw hashes and native identities for the exact full recorded tuple.
fn verify_approval(
    capture: &ReviewCapture,
    inputs: &Inputs,
    parsed: &Parsed,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let source = &inputs.lifecycle.policy.source;
    if !shared::compare(&source.sha256, &parsed.pins[inputs.report_slot].raw_sha256, ledger)?
        .is_eq()
    {
        return Err(ContractError::Binding);
    }
    let record_index = index(inputs, inputs.record_slot)?;
    let record_path = capture.path(record_index)?;
    let mut generated = reserved(inputs.lifecycle.policy.generated_artifacts.len(), ledger)?;
    for artifact in &inputs.lifecycle.policy.generated_artifacts {
        let path = shared::join_relative(record_path, Path::new(&artifact.path), ledger, control)?;
        let slot = path_slot(
            &inputs.locator,
            &path,
            role_for_native(artifact.oscal_type.as_deref())?,
            ledger,
            control,
        )?;
        let native = parsed.natives[slot].as_ref().ok_or(ContractError::Binding)?;
        if artifact.oscal_type.is_none() || artifact.root_uuid.is_none() {
            return Err(ContractError::Binding);
        }
        shared::check_fingerprint_identity(artifact, Some(native), ledger)?;
        if !shared::compare(&artifact.sha256, &parsed.pins[slot].raw_sha256, ledger)?.is_eq() {
            return Err(ContractError::Binding);
        }
        generated.push(NamedHash {
            path: copy_text(&artifact.path, ledger, control)?,
            sha256: copy_text(&parsed.pins[slot].raw_sha256, ledger, control)?,
        });
    }
    let current = CurrentArtifacts {
        fingerprints: FingerprintSet {
            source_sha256: copy_text(&parsed.pins[inputs.report_slot].raw_sha256, ledger, control)?,
            generated_artifacts: generated,
        },
        identity_changes: Vec::new(),
    };
    let extent = capture
        .bytes(record_index)?
        .len()
        .checked_mul(8)
        .and_then(|n| n.checked_add(4096))
        .ok_or(ContractError::Capacity)?;
    ledger.bytes(extent)?;
    ledger.derived(extent)?;
    ledger.checkpoint(control)?;
    let status = status_from_captured(&inputs.lifecycle, &current, None);
    ledger.checkpoint(control)?;
    let status = status.map_err(|_| ContractError::Binding)?;
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

/// Borrow actual native resources and explicit companions without any href IO.
fn native_inputs<'a>(
    capture: &'a ReviewCapture,
    inputs: &'a Inputs,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<captured::NativeInput<'a>>, ContractError> {
    let mut result = reserved(inputs.locator.sources.len(), ledger)?;
    for (slot, row) in inputs.locator.sources.iter().enumerate() {
        ledger.checkpoint(control)?;
        let resource_type = match row.model {
            SourceModel::Catalog | SourceModel::ResolvedCatalog => ResourceType::Catalog,
            SourceModel::Profile => ResourceType::Profile,
            _ => continue,
        };
        let companion = match &row.resolved_catalog_key {
            Some(key) => {
                let companion = key_slot(&inputs.locator, key, ledger, control)?;
                Some(captured::ResolvedInput {
                    key: &inputs.locator.sources[companion].key,
                    bytes: capture.bytes(index(inputs, companion)?)?,
                })
            }
            None => None,
        };
        result.push(captured::NativeInput {
            key: &row.key,
            resource_type,
            bytes: capture.bytes(index(inputs, slot)?)?,
            resolved_catalog: companion,
        });
    }
    Ok(result)
}

/// Route complete helper admissions into the sole invocation ledger without control reset.
fn domain_charge(
    charge: admitted::ApplicabilityCharge,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| match charge {
        admitted::ApplicabilityCharge::Checkpoint => Ok(()),
        admitted::ApplicabilityCharge::Work { visits, byte_work, matching_steps } => {
            ledger.visits(visits)?;
            ledger.bytes(byte_work)?;
            ledger.matching(matching_steps)
        }
        admitted::ApplicabilityCharge::Reserve { logical_bytes } => ledger.derived(logical_bytes),
        admitted::ApplicabilityCharge::Capacity => Err(ledger.capacity()),
    })
}

/// Preserve caller capacity and interruption; reduce only ordinary domain invalidity.
fn domain_error(error: &admitted::ApplicabilityAdmissionError<ContractError>) -> ContractError {
    match error {
        admitted::ApplicabilityAdmissionError::Domain => ContractError::Binding,
        admitted::ApplicabilityAdmissionError::Admission(error) => *error,
        admitted::ApplicabilityAdmissionError::Work(WorkError::Interrupted(reason)) => {
            ContractError::Interrupted(*reason)
        }
        admitted::ApplicabilityAdmissionError::Work(WorkError::Failed(_)) => {
            ContractError::ControlFailed
        }
    }
}

/// Copy complete actual facts only after full original report equality under admitted semantics.
fn project(
    capture: &ReviewCapture,
    inputs: &Inputs,
    parsed: &Parsed,
    original: Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingApplicabilityClosure, ContractError> {
    let natives = native_inputs(capture, inputs, ledger, control)?;
    let mut mappings = reserved(inputs.locator.mapping_keys.len(), ledger)?;
    for key in &inputs.locator.mapping_keys {
        let slot = key_slot(&inputs.locator, key, ledger, control)?;
        mappings.push(captured::MappingInput { key, bytes: capture.bytes(index(inputs, slot)?)? });
    }
    let admitted = admitted::prepare_admitted(
        capture.bytes(index(inputs, inputs.manifest_slot)?)?,
        &inputs.locator.framework_key,
        &natives,
        &mappings,
        &mut |charge| domain_charge(charge, ledger),
        control,
    )
    .map_err(|error| domain_error(&error))?;
    admitted
        .validate_report_admitted(
            capture.bytes(index(inputs, inputs.report_slot)?)?,
            &mut |charge| domain_charge(charge, ledger),
            control,
        )
        .map_err(|error| domain_error(&error))?;
    ledger.checkpoint(control)?;
    let core = admitted.core();
    let counts = core.counts();
    let sum = [
        counts.applicable_mapped,
        counts.applicable_reviewed_no_relationship,
        counts.applicable_unmapped,
        counts.not_applicable,
        counts.deferred,
        counts.under_review,
    ]
    .into_iter()
    .try_fold(0_usize, |a, b| a.checked_add(b).ok_or(ContractError::Capacity))?;
    if counts.total != sum
        || counts.total > 10_000
        || core.complete_maps() > 10_000
        || core.complete_pair_inspections() > 100_000
    {
        return Err(ledger.capacity());
    }
    let framework = core.framework_evidence().ok_or(ContractError::Binding)?;
    let actual = parsed.natives[inputs.framework_slot].as_ref().ok_or(ContractError::Binding)?;
    if !shared::compare(
        &framework.raw_sha256,
        &parsed.pins[inputs.framework_slot].raw_sha256,
        ledger,
    )?
    .is_eq()
        || !shared::compare(&framework.root_uuid, &actual.canonical_uuid, ledger)?.is_eq()
    {
        return Err(ContractError::Binding);
    }
    let controls = project_controls(core, &original, counts.total, ledger, control)?;
    ledger.derived(std::mem::size_of::<model::ClassificationCounts>())?;
    let framework = copy_evidence(framework, ledger, control)?;
    Ok(PendingApplicabilityClosure {
        manifest: original,
        controls,
        framework,
        counts: counts.clone(),
        maps: core.complete_maps(),
        pairs: core.complete_pair_inspections(),
        pins: Vec::new(),
        originals: Vec::new(),
    })
}

/// Retain the complete sorted control facts after full admitted report equality.
fn project_controls(
    core: &captured::PreparedApplicability<'_>,
    original: &Value,
    total: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<ControlFacts>, ContractError> {
    let mut controls = reserved(total, ledger)?;
    let decisions =
        original.get("decisions").and_then(Value::as_array).ok_or(ContractError::Binding)?;
    for row in core.control_rows() {
        ledger.checkpoint(control)?;
        ledger.visits(1)?;
        if let Some(previous) = controls.last() {
            let previous: &ControlFacts = previous;
            if !shared::compare(previous.control_id(), row.control_id(), ledger)?.is_lt() {
                return Err(ContractError::Binding);
            }
        }
        let fingerprint =
            core.control_fingerprint(row.control_id()).ok_or(ContractError::Binding)?;
        ledger.bytes(fingerprint.len())?;
        if fingerprint.len() != 64
            || !fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(ContractError::Binding);
        }
        let decision_index = core.explicit_decision_index(row.control_id());
        if let Some(index) = decision_index {
            let raw = decisions.get(index).ok_or(ContractError::Binding)?;
            let typed = core.explicit_decision(index).ok_or(ContractError::Binding)?;
            if !shared::compare(&typed.control_id, row.control_id(), ledger)?.is_eq()
                || !shared::compare(
                    raw.get("control_id").and_then(Value::as_str).ok_or(ContractError::Binding)?,
                    row.control_id(),
                    ledger,
                )?
                .is_eq()
            {
                return Err(ContractError::Binding);
            }
        }
        let (positive_count, no_relationship_count) = core.native_participation(row.control_id());
        controls.push(ControlFacts {
            control_id: copy_text(row.control_id(), ledger, control)?,
            native_fingerprint: copy_text(fingerprint, ledger, control)?,
            classification: row.classification(),
            decision_index,
            positive_count,
            no_relationship_count,
        });
    }
    if controls.len() != total {
        return Err(ContractError::Binding);
    }
    Ok(controls)
}

/// Copy one complete admitted retained string before its actual allocation.
fn copy_text(
    value: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)?;
    ledger.bytes(value.len())?;
    ledger.derived(value.len())?;
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| ContractError::Capacity)?;
    result.push_str(value);
    Ok(result)
}

/// Retain full actual framework evidence without exposing its private href in the queue.
fn copy_evidence(
    value: &ResourceEvidence,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ResourceEvidence, ContractError> {
    ledger.derived(std::mem::size_of::<ResourceEvidence>())?;
    Ok(ResourceEvidence {
        resource_type: value.resource_type,
        href: copy_text(&value.href, ledger, control)?,
        raw_sha256: copy_text(&value.raw_sha256, ledger, control)?,
        root_uuid: copy_text(&value.root_uuid, ledger, control)?,
        document_version: copy_text(&value.document_version, ledger, control)?,
        oscal_version: copy_text(&value.oscal_version, ledger, control)?,
        resolved_catalog_sha256: value
            .resolved_catalog_sha256
            .as_deref()
            .map(|hash| copy_text(hash, ledger, control))
            .transpose()?,
    })
}

/// Genuine synthetic file-backed controls never authenticate a human or construct a success proof.
#[cfg(test)]
#[path = "applicability_capture_tests.rs"]
pub(crate) mod tests;

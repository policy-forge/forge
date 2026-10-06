//! Current applicability semantics over genuine externally captured originals.
//!
//! This private byte consumer creates neither disclosure approval nor a capture
//! proof. The MCP gate retains/rechecks the actual originals. All native/domain
//! algorithms below compose maintained inventory, Mapping and classification
//! routines. Complete registries and Cartesian pairs use the supplied monotonic
//! ledger before growth; no URI read, temporary project or computed report exists.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{manifest, model};
use crate::hashing::sha256_hex;
use crate::mapping::inventory::{self, CapturedInventoryError, LoadedResource, ResourceEvidence};
use crate::mapping::manifest::{ResourceManifest, ResourceType, SubjectType};
use crate::mapping::model::{MappingCollectionEnvelope, MappingItem};
use crate::validate::{self, OscalModelType};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// Existing MCP original pool ceiling; this is not a new independent pool.
const MAX_RAW: usize = 50 * 1024 * 1024;
/// Existing persisted-report and MCP domain-original ceiling.
const MAX_DOMAIN: usize = 10 * 1024 * 1024;
/// Complete native control denominator before filtering or pagination.
const MAX_CONTROLS: usize = 10_000;
/// Complete shared MCP original roster domain, including configuration originals.
const MAX_NATIVES: usize = 999;

/// Actual captured native candidate; the gate alone establishes its approval.
#[derive(Clone, Copy)]
pub(crate) struct NativeInput<'a> {
    /// Exact opaque resource key, never a native title or filesystem path.
    pub(crate) key: &'a str,
    /// Intrinsically supported native Catalog or Profile kind.
    pub(crate) resource_type: ResourceType,
    /// Complete original native bytes from the same actual held scope.
    pub(crate) bytes: &'a [u8],
    /// Explicit actual approved visible resolved Catalog companion, if declared.
    pub(crate) resolved_catalog: Option<ResolvedInput<'a>>,
}

/// Explicit captured Profile companion; a hash or href does not create this input.
#[derive(Clone, Copy)]
pub(crate) struct ResolvedInput<'a> {
    /// Exact independently selected companion resource key.
    pub(crate) key: &'a str,
    /// Complete original companion bytes, not generated resolution output.
    pub(crate) bytes: &'a [u8],
}

/// Actual native Mapping original with its separate captured resource identity.
#[derive(Clone, Copy)]
pub(crate) struct MappingInput<'a> {
    /// Captured MCP key; it is not FORGE collection-key metadata.
    pub(crate) key: &'a str,
    /// Complete raw original native Mapping bytes.
    pub(crate) bytes: &'a [u8],
}

/// Plain Catalog input borrowing a complete maintained inventory from its real owner.
///
/// This input grants no capture, lifecycle, approval or disclosure capability. The caller
/// retains the actual inventory and original bytes; this helper binds their raw evidence.
#[derive(Clone, Copy)]
pub(crate) struct BorrowedCatalogInput<'a> {
    /// Complete exact original candidate, with no Profile companion substitution.
    pub(crate) input: NativeInput<'a>,
    /// Whole maintained Catalog inventory, never a selected control/statement subset.
    pub(crate) loaded: &'a LoadedResource,
}

/// Private storage preserves the original owned path and the additive complete borrow.
enum NativeInventory<'a> {
    /// Existing byte consumer owns the full maintained loader output.
    Owned(LoadedResource),
    /// The real inventory remains owned externally; declared report evidence is retained here.
    Borrowed {
        /// Complete immutable maintained inventory and original native evidence.
        loaded: &'a LoadedResource,
        /// Exact declared href overlay without altering the externally owned resource.
        evidence: ResourceEvidence,
    },
}

impl NativeInventory<'_> {
    /// Borrow the same complete inventory for all maintained semantic routines.
    fn resource(&self) -> &LoadedResource {
        match self {
            Self::Owned(loaded) => loaded,
            Self::Borrowed { loaded, .. } => loaded,
        }
    }

    /// Preserve manifest-facing evidence separately from the source owner's href.
    fn evidence(&self) -> &ResourceEvidence {
        match self {
            Self::Owned(loaded) => &loaded.evidence,
            Self::Borrowed { evidence, .. } => evidence,
        }
    }
}

/// One admitted native header and a lazily needed complete effective inventory.
struct NativePrepared<'a> {
    /// Actual caller-supplied borrowed original candidate.
    input: NativeInput<'a>,
    /// Maintained schema-admitted actual native identity and original hashes.
    evidence: ResourceEvidence,
    /// Complete maintained inventory only when the framework or a source needs it.
    loaded: Option<NativeInventory<'a>>,
}

/// Retained native Mapping semantics without duplicating hrefs per control.
struct MappingPrepared<'a> {
    /// Actual resource key borrowed from the real supplied roster.
    key: &'a str,
    /// One decoded native collection admitted by the maintained algorithm.
    collection: MappingCollectionEnvelope,
    /// Actual raw map UUID spelling retained before typed UUID normalization.
    map_ids: Vec<Vec<String>>,
    /// Full native evidence used by the exact current stored-report oracle.
    evidence: model::MappingEvidence,
    /// Exact actual source native index for every native mapping group.
    sources: Vec<usize>,
}

/// Native per-edge/target participation, distinct from Cartesian trace pairs.
#[derive(Default)]
struct Facts {
    /// Complete positive native map participation count.
    positive: usize,
    /// Complete explicitly reviewed no-relationship participation count.
    no_relationship: usize,
    /// Borrowable source-href locators, never copied private URI strings per row.
    sources: BTreeSet<(usize, usize)>,
}

/// One precharged actual Cartesian native relation locator.
struct Relation {
    /// Actual original Mapping roster index.
    collection: usize,
    /// Actual native mapping group index within that original.
    mapping: usize,
    /// Actual native map index, preserving its raw UUID spelling separately.
    edge: usize,
    /// Exact source subject occurrence index.
    source: usize,
    /// Exact target subject occurrence index.
    target: usize,
}

/// Complete current domain facts; no authority or original proof constructor.
pub(crate) struct PreparedApplicability<'a> {
    /// Actual intrinsic applicability source, retained once.
    manifest: manifest::ApplicabilityManifest,
    /// Original raw digest, separate from classification and canonical fingerprints.
    manifest_sha256: String,
    /// Actual unique supplied native roster and effective inventories.
    natives: Vec<NativePrepared<'a>>,
    /// Exact selected framework index, never a hash-only discovered replacement.
    framework: usize,
    /// Actual full Mapping semantics with original UUID spellings.
    mappings: Vec<MappingPrepared<'a>>,
    /// Complete sorted native per-control participation facts.
    facts: BTreeMap<String, Facts>,
    /// Actual explicit decision locators; omitted controls remain under review.
    decisions: BTreeMap<String, usize>,
    /// Reconciled complete maintained classification denominator.
    counts: model::ClassificationCounts,
    /// Actual complete Cartesian locator universe, not a filtered prefix.
    relations: Vec<Relation>,
}

/// One borrowed actual native control and maintained classification.
pub(crate) struct ControlRow<'a> {
    /// Exact control identifier from the admitted effective native inventory.
    control_id: &'a str,
    /// Actual explicit decision or genuine omission.
    decision_state: Option<manifest::DecisionState>,
    /// Classification from the maintained pure model routine.
    classification: model::GapClassification,
}

impl<'a> ControlRow<'a> {
    /// Borrow the complete original native control identifier.
    pub(crate) fn control_id(&self) -> &'a str {
        self.control_id
    }
    /// Preserve explicit native state or an actual omitted decision.
    pub(crate) fn decision_state(&self) -> Option<manifest::DecisionState> {
        self.decision_state
    }
    /// Return the maintained classification, not an MCP-specific inference.
    pub(crate) fn classification(&self) -> model::GapClassification {
        self.classification
    }
}

/// One actual full native source/target pair, with no href or authority projection.
pub(crate) struct MappingRelation<'a> {
    /// Exact supplied MappingInput.key, distinct from native collection-key.
    collection_key: &'a str,
    /// Exact original native map UUID spelling.
    map_id: &'a str,
    /// Independently supplied exact source native key.
    source_key: &'a str,
    /// Actual source subject kind; statement support remains query-owned.
    source_subject_type: SubjectType,
    /// Actual complete source subject identifier.
    source_id: &'a str,
    /// Exact explicitly selected framework native key.
    target_key: &'a str,
    /// Actual target subject kind.
    target_subject_type: SubjectType,
    /// Actual complete target subject identifier.
    target_id: &'a str,
}

impl<'a> MappingRelation<'a> {
    /// Borrow the supplied captured Mapping key, never metadata collection-key.
    pub(crate) fn collection_key(&self) -> &'a str {
        self.collection_key
    }
    /// Borrow original UUID text without typed normalization.
    pub(crate) fn map_id(&self) -> &'a str {
        self.map_id
    }
    /// Borrow the uniquely resolved actual source key.
    pub(crate) fn source_key(&self) -> &'a str {
        self.source_key
    }
    /// Return the actual native source subject kind.
    pub(crate) fn source_subject_type(&self) -> SubjectType {
        self.source_subject_type
    }
    /// Borrow the actual source identifier.
    pub(crate) fn source_id(&self) -> &'a str {
        self.source_id
    }
    /// Borrow the actual framework target key.
    pub(crate) fn target_key(&self) -> &'a str {
        self.target_key
    }
    /// Return the actual target subject kind.
    pub(crate) fn target_subject_type(&self) -> SubjectType {
        self.target_subject_type
    }
    /// Borrow the actual target identifier.
    pub(crate) fn target_id(&self) -> &'a str {
        self.target_id
    }
}

/// Prepare complete actual current semantics using one caller-owned shared ledger.
///
/// Manifest-relative path resolution and recorded approval remain the producer's
/// opaque gate responsibility. This helper revalidates every supplied original's
/// intrinsic native identity, companion and current subject fingerprints.
pub(crate) fn prepare<'a>(
    manifest_bytes: &'a [u8],
    framework_key: &str,
    natives: &[NativeInput<'a>],
    mappings: &[MappingInput<'a>],
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedApplicability<'a>> {
    fence(control)?;
    let value = strict(manifest_bytes, MAX_DOMAIN)?;
    admit_json(&value, 0, charge, control)?;
    let manifest = domain(manifest::parse(manifest_bytes))?;
    if natives.len() > MAX_NATIVES || mappings.len() != manifest.mapping_collections.len() {
        return Err(invalid());
    }
    let mut natives = native_headers(natives, charge, control)?;
    let framework = unique_key(&natives, framework_key)?;
    if natives[framework].input.resource_type != manifest.framework.resource_type {
        return Err(invalid());
    }
    natives[framework].loaded = Some(NativeInventory::Owned(load_native(
        &natives[framework].input,
        &manifest.framework,
        charge,
        control,
    )?));
    finish_preparation(manifest_bytes, manifest, natives, framework, mappings, charge, control)
}

/// Consume only complete Catalog inventories, preserving every maintained manifest/report rule.
///
/// The original `prepare` keeps its native validation/load path. This additive plain-data
/// path performs no native parse/schema/inventory rebuild and cannot issue native authority.
/// Explicit Profile/companion or Mapping declarations refuse the entire borrowed path.
pub(crate) fn prepare_borrowed_catalog<'a>(
    manifest_bytes: &'a [u8],
    framework_key: &str,
    inputs: &[BorrowedCatalogInput<'a>],
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedApplicability<'a>> {
    fence(control)?;
    let value = strict(manifest_bytes, MAX_DOMAIN)?;
    admit_json(&value, 0, charge, control)?;
    let manifest = domain(manifest::parse(manifest_bytes))?;
    if inputs.len() > MAX_NATIVES
        || manifest.framework.resource_type != ResourceType::Catalog
        || !manifest.mapping_collections.is_empty()
    {
        return Err(invalid());
    }
    let mut natives = borrowed_headers(inputs, charge, control)?;
    let framework = unique_key(&natives, framework_key)?;
    let loaded = inputs[framework].loaded;
    if manifest
        .framework
        .expected_sha256
        .as_ref()
        .is_some_and(|expected| expected != &loaded.evidence.raw_sha256)
    {
        return Err(invalid());
    }
    if let Some(expected) = &manifest.framework.inventory
        && expected != &loaded.snapshot()
    {
        return Err(invalid());
    }
    // Copy only actual native metadata plus the manifest's complete private href. The full
    // inventory remains borrowed from its owner and is never cloned or reconstructed here.
    charge(1)?;
    let mut evidence = loaded.evidence.clone();
    evidence.href.clone_from(&manifest.framework.href);
    natives[framework].loaded = Some(NativeInventory::Borrowed { loaded, evidence });
    finish_preparation(manifest_bytes, manifest, natives, framework, &[], charge, control)
}

/// Bind each exact Catalog original to its complete borrowed loader output before growth.
fn borrowed_headers<'a>(
    inputs: &[BorrowedCatalogInput<'a>],
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<Vec<NativePrepared<'a>>> {
    let mut result: Vec<NativePrepared<'a>> = Vec::new();
    for row in inputs {
        fence(control)?;
        let input = row.input;
        let loaded = row.loaded;
        if input.key.is_empty()
            || input.bytes.len() > MAX_RAW
            || input.resource_type != ResourceType::Catalog
            || input.resolved_catalog.is_some()
            || loaded.evidence.resource_type != ResourceType::Catalog
            || loaded.evidence.resolved_catalog_sha256.is_some()
            || result.iter().any(|old| old.input.key == input.key)
        {
            return Err(invalid());
        }
        if sha256_hex(input.bytes) != loaded.evidence.raw_sha256 {
            return Err(invalid());
        }
        charge(1)?;
        let mut evidence = loaded.evidence.clone();
        input.key.clone_into(&mut evidence.href);
        result.push(NativePrepared { input, evidence, loaded: None });
    }
    fence(control)?;
    Ok(result)
}

/// Share complete decision, Mapping, classification and denominator semantics across both paths.
fn finish_preparation<'a>(
    manifest_bytes: &[u8],
    manifest: manifest::ApplicabilityManifest,
    natives: Vec<NativePrepared<'a>>,
    framework: usize,
    mappings: &[MappingInput<'a>],
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedApplicability<'a>> {
    let framework_loaded = natives[framework].loaded.as_ref().ok_or_else(invalid)?.resource();
    if framework_loaded.inventory.count(SubjectType::Control) > MAX_CONTROLS {
        return Err(invalid());
    }
    domain(super::validate_applicability_group_ids(&framework_loaded.inventory))?;
    domain(super::validate_decision_references(&manifest, framework_loaded))?;
    let mut decisions = BTreeMap::new();
    for (index, decision) in manifest.decisions.iter().enumerate() {
        fence(control)?;
        charge(1)?;
        if decisions.insert(decision.control_id.clone(), index).is_some() {
            return Err(invalid());
        }
    }
    let mut prepared = PreparedApplicability {
        manifest,
        manifest_sha256: sha256_hex(manifest_bytes),
        natives,
        framework,
        mappings: Vec::new(),
        facts: BTreeMap::new(),
        decisions,
        counts: model::ClassificationCounts::default(),
        relations: Vec::new(),
    };
    prepare_mappings(&mut prepared, mappings, charge, control)?;
    let mut counts = model::ClassificationCounts::default();
    for id in prepared.framework_inventory()?.ids_of_type_refs(SubjectType::Control) {
        fence(control)?;
        let decision = prepared.decision(id);
        let facts = prepared.facts.get(id);
        let classification = classify(decision, facts);
        counts.record(classification);
    }
    domain(super::validate_classification_counts(&counts))?;
    prepared.counts = counts;
    fence(control)?;
    Ok(prepared)
}

/// Revalidate every supplied native header without forcing unused Profile resolution.
fn native_headers<'a>(
    inputs: &[NativeInput<'a>],
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<Vec<NativePrepared<'a>>> {
    let mut result: Vec<NativePrepared<'a>> = Vec::new();
    for input in inputs {
        fence(control)?;
        if input.key.is_empty() || result.iter().any(|row| row.input.key == input.key) {
            return Err(invalid());
        }
        let value = strict(input.bytes, MAX_RAW)?;
        admit_json(&value, 0, charge, control)?;
        let expected = match input.resource_type {
            ResourceType::Catalog => OscalModelType::Catalog,
            ResourceType::Profile => OscalModelType::Profile,
        };
        if domain(validate::detect_model_type(&value))? != expected {
            return Err(invalid());
        }
        domain(inventory::validate_schema("captured native", &value, expected))?;
        if let Some(companion) = input.resolved_catalog {
            if input.resource_type != ResourceType::Profile
                || companion.key.is_empty()
                || companion.bytes.len() > MAX_RAW
            {
                return Err(invalid());
            }
        }
        let resource = bare_resource(input);
        let mut evidence = domain(inventory::extract_evidence(
            "captured native",
            &resource,
            &value,
            sha256_hex(input.bytes),
        ))?;
        if let Some(companion) = input.resolved_catalog {
            if input.resource_type != ResourceType::Profile {
                return Err(invalid());
            }
            evidence.resolved_catalog_sha256 = Some(sha256_hex(companion.bytes));
        }
        charge(1)?;
        result.push(NativePrepared { input: *input, evidence, loaded: None });
    }
    fence(control)?;
    Ok(result)
}

/// Build only an intrinsic native resource descriptor; it cannot confer approval.
fn bare_resource(input: &NativeInput<'_>) -> ResourceManifest {
    ResourceManifest {
        resource_type: input.resource_type,
        artifact: PathBuf::from(input.key),
        href: input.key.to_owned(),
        resolved_catalog: input.resolved_catalog.map(|row| PathBuf::from(row.key)),
        resolved_catalog_attestation: input.resolved_catalog.map(|_| true),
        expected_sha256: Some(sha256_hex(input.bytes)),
        expected_resolved_catalog_sha256: input.resolved_catalog.map(|row| sha256_hex(row.bytes)),
        inventory: None,
    }
}

/// Resolve only one exact supplied key; bytes/identity matches do not select a framework.
fn unique_key(natives: &[NativePrepared<'_>], key: &str) -> WorkResult<usize> {
    let mut matches = natives.iter().enumerate().filter(|(_, row)| row.input.key == key);
    let index = matches.next().map(|(index, _)| index).ok_or_else(invalid)?;
    if matches.next().is_some() {
        return Err(invalid());
    }
    Ok(index)
}

/// Consume the shared maintained pure inventory loader without a fake `CaptureSession`.
fn load_native(
    input: &NativeInput<'_>,
    resource: &ResourceManifest,
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<LoadedResource> {
    fence(control)?;
    if let Some(companion) = input.resolved_catalog {
        let value = strict(companion.bytes, MAX_RAW)?;
        admit_json(&value, 0, charge, control)?;
    }
    let result = inventory::load_captured_admitted(
        Path::new(""),
        "captured native",
        resource,
        input.bytes,
        input.resolved_catalog.map(|row| row.bytes),
        &mut |value| admit_inventory(value, charge, control),
    );
    let loaded = match result {
        Ok(value) => value,
        Err(CapturedInventoryError::Domain(_)) => return Err(invalid()),
        Err(CapturedInventoryError::Admission(error)) => return Err(error),
    };
    fence(control)?;
    Ok(loaded)
}

/// Admit every complete borrowed JSON occurrence before downstream registries grow.
fn admit_json(
    value: &Value,
    depth: usize,
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    fence(control)?;
    if depth > 64 {
        return Err(invalid());
    }
    match value {
        Value::Array(values) => {
            for child in values {
                charge(1)?;
                admit_json(child, depth + 1, charge, control)?;
            }
        }
        Value::Object(fields) => {
            for child in fields.values() {
                charge(1)?;
                admit_json(child, depth + 1, charge, control)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Admit full group ancestry/subject/part occurrences before existing inventory growth.
fn admit_inventory(
    value: &Value,
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    let root = value.get("catalog").ok_or_else(invalid)?;
    admit_inventory_nodes(root, 0, 0, charge, control)
}

/// Charge every retained ancestry occurrence, including duplicates later rejected.
fn admit_inventory_nodes(
    value: &Value,
    depth: usize,
    ancestors: usize,
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    fence(control)?;
    if depth > inventory::MAX_INVENTORY_DEPTH {
        return Err(invalid());
    }
    for key in ["groups", "controls", "parts"] {
        if let Some(values) = value.get(key).and_then(Value::as_array) {
            for child in values {
                fence(control)?;
                charge(1)?;
                let next = if key == "groups" {
                    charge(ancestors)?;
                    ancestors
                        .checked_add(usize::from(child.get("id").and_then(Value::as_str).is_some()))
                        .ok_or_else(invalid)?
                } else {
                    if key == "controls" {
                        charge(ancestors)?;
                    }
                    ancestors
                };
                admit_inventory_nodes(child, depth + 1, next, charge, control)?;
            }
        }
    }
    Ok(())
}

/// Reuse native collection semantics and independently prove every actual source tuple.
fn prepare_mappings<'a>(
    prepared: &mut PreparedApplicability<'a>,
    inputs: &[MappingInput<'a>],
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    let mut state = super::MappingValidationState::default();
    for input in inputs {
        fence(control)?;
        if input.key.is_empty() || prepared.mappings.iter().any(|row| row.key == input.key) {
            return Err(invalid());
        }
        let value = strict(input.bytes, MAX_RAW)?;
        admit_json(&value, 0, charge, control)?;
        let map_ids = mapping_preflight(&value, charge, control)?;
        let collection = domain(super::decode_mapping_value("captured Mapping", value))?;
        let sources = match_sources(prepared, &collection, charge, control)?;
        let collection_index = prepared.mappings.len();
        let framework =
            prepared.natives[prepared.framework].loaded.as_ref().ok_or_else(invalid)?.resource();
        let facts = &mut prepared.facts;
        let evidence = domain(super::validate_mapping_collection(
            "captured Mapping",
            &collection,
            &sha256_hex(input.bytes),
            framework,
            &mut state,
            &mut |mapping, id, positive, _| {
                let facts = facts.entry(id.to_owned()).or_default();
                if positive {
                    facts.positive += 1;
                } else {
                    facts.no_relationship += 1;
                }
                facts.sources.insert((collection_index, mapping));
            },
        ))?;
        charge(1)?;
        prepared.mappings.push(MappingPrepared {
            key: input.key,
            collection,
            map_ids,
            evidence,
            sources,
        });
        retain_pairs(prepared, collection_index, control)?;
    }
    Ok(())
}

/// Reserve complete Cartesian state/traversal and trace locator denominators before growth.
fn mapping_preflight(
    value: &Value,
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<Vec<Vec<String>>> {
    let mappings = value
        .get("mapping-collection")
        .and_then(|value| value.get("mappings"))
        .and_then(Value::as_array)
        .ok_or_else(invalid)?;
    let mut ids = Vec::new();
    for mapping in mappings {
        fence(control)?;
        let maps = mapping.get("maps").and_then(Value::as_array).ok_or_else(invalid)?;
        let mut current = Vec::new();
        for edge in maps {
            fence(control)?;
            let sources = edge.get("sources").and_then(Value::as_array).ok_or_else(invalid)?;
            let targets = edge.get("targets").and_then(Value::as_array).ok_or_else(invalid)?;
            let pairs = sources.len().checked_mul(targets.len()).ok_or_else(invalid)?;
            charge(pairs)?; // Full maintained polarity/source-target traversal registry.
            charge(pairs)?; // Distinct complete returned trace-locator registry.
            charge(targets.len())?; // Native per-target count/fact registry, not pairs.
            charge(targets.len())?; // Borrowed source-locator set participation before insert.
            let id = edge.get("uuid").and_then(Value::as_str).ok_or_else(invalid)?;
            if id.len() > 45 || uuid::Uuid::parse_str(id).is_err() {
                return Err(invalid());
            }
            charge(1)?;
            current.push(id.to_owned());
        }
        charge(1)?;
        ids.push(current);
    }
    Ok(ids)
}

/// Match full actual native tuple, load its effective inventory, and check all source fingerprints.
fn match_sources(
    prepared: &mut PreparedApplicability<'_>,
    collection: &MappingCollectionEnvelope,
    charge: &mut dyn FnMut(usize) -> WorkResult<()>,
    control: &mut dyn WorkControl,
) -> WorkResult<Vec<usize>> {
    let mut result = Vec::new();
    for mapping in &collection.mapping_collection.mappings {
        fence(control)?;
        let evidence =
            domain(super::source_resource_evidence("captured Mapping", &mapping.source_resource))?;
        let source = matching_native(&prepared.natives, &evidence, control)?;
        if prepared.natives[source].loaded.is_none() {
            let input = prepared.natives[source].input;
            prepared.natives[source].loaded = Some(NativeInventory::Owned(load_native(
                &input,
                &bare_resource(&input),
                charge,
                control,
            )?));
        }
        let loaded = prepared.natives[source].loaded.as_ref().ok_or_else(invalid)?.resource();
        for edge in &mapping.maps {
            for item in &edge.sources {
                fence(control)?;
                match_subject(loaded, item)?;
            }
        }
        charge(1)?;
        result.push(source);
    }
    Ok(result)
}

/// Resolve a unique complete current resource tuple, never href or raw-hash-only authority.
fn matching_native(
    natives: &[NativePrepared<'_>],
    expected: &ResourceEvidence,
    control: &mut dyn WorkControl,
) -> WorkResult<usize> {
    let mut selected = None;
    for (index, native) in natives.iter().enumerate() {
        fence(control)?;
        let actual = &native.evidence;
        if actual.resource_type == expected.resource_type
            && actual.raw_sha256 == expected.raw_sha256
            && actual.root_uuid == expected.root_uuid
            && actual.document_version == expected.document_version
            && actual.oscal_version == expected.oscal_version
            && actual.resolved_catalog_sha256 == expected.resolved_catalog_sha256
        {
            if selected.is_some() {
                return Err(invalid());
            }
            selected = Some(index);
        }
    }
    selected.ok_or_else(invalid)
}

/// Reuse actual current canonical native subject fingerprints for a source endpoint.
fn match_subject(source: &LoadedResource, item: &MappingItem) -> WorkResult<()> {
    let expected =
        source.inventory.fingerprint(item.subject_type, &item.id_ref).ok_or_else(invalid)?;
    let actual = domain(super::require_single_prop(
        "captured Mapping subject",
        &item.props,
        "subject-sha256",
    ))?;
    if actual != expected {
        return Err(invalid());
    }
    Ok(())
}

/// Retain only previously precharged complete Cartesian indices after semantic admission.
fn retain_pairs(
    prepared: &mut PreparedApplicability<'_>,
    collection: usize,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    let native = &prepared.mappings[collection].collection.mapping_collection;
    for (mapping, group) in native.mappings.iter().enumerate() {
        for (edge, map) in group.maps.iter().enumerate() {
            for source in 0..map.sources.len() {
                for target in 0..map.targets.len() {
                    fence(control)?;
                    prepared.relations.push(Relation { collection, mapping, edge, source, target });
                }
            }
        }
    }
    Ok(())
}

impl PreparedApplicability<'_> {
    /// Borrow the full actual effective framework inventory, never a caller snapshot alone.
    fn framework_inventory(&self) -> WorkResult<&inventory::Inventory> {
        self.natives[self.framework]
            .loaded
            .as_ref()
            .map(|row| &row.resource().inventory)
            .ok_or_else(invalid)
    }
    /// Borrow the actual explicit source decision or preserve genuine omission.
    fn decision(&self, id: &str) -> Option<&manifest::ControlDecision> {
        self.decisions.get(id).map(|index| &self.manifest.decisions[*index])
    }
    /// Expose full canonical control order using the unchanged maintained classifier.
    pub(crate) fn control_rows(&self) -> impl Iterator<Item = ControlRow<'_>> {
        self.natives[self.framework]
            .loaded
            .iter()
            .flat_map(|row| row.resource().inventory.ids_of_type_refs(SubjectType::Control))
            .map(|id| {
                let decision = self.decision(id);
                ControlRow {
                    control_id: id,
                    decision_state: decision.map(|row| row.state),
                    classification: classify(decision, self.facts.get(id)),
                }
            })
    }
    /// Borrow full native counts; Cartesian trace pairs never multiply classifications.
    pub(crate) fn counts(&self) -> &model::ClassificationCounts {
        &self.counts
    }
    /// Borrow the actual original source digest, not a normalized/native digest.
    pub(crate) fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
    /// Iterate complete actual admitted pairs matching either exact subject ID.
    pub(crate) fn relations<'a>(
        &'a self,
        control_id: &'a str,
    ) -> impl Iterator<Item = MappingRelation<'a>> + 'a {
        self.relations
            .iter()
            .map(|row| self.relation(row))
            .filter(move |row| row.source_id == control_id || row.target_id == control_id)
    }
    /// Project only original key/UUID/subject locators; no URI or private prose escapes.
    fn relation<'a>(&'a self, row: &Relation) -> MappingRelation<'a> {
        let collection = &self.mappings[row.collection];
        let mapping = &collection.collection.mapping_collection.mappings[row.mapping];
        let edge = &mapping.maps[row.edge];
        let source = &edge.sources[row.source];
        let target = &edge.targets[row.target];
        MappingRelation {
            collection_key: collection.key,
            map_id: &collection.map_ids[row.mapping][row.edge],
            source_key: self.natives[collection.sources[row.mapping]].input.key,
            source_subject_type: source.subject_type,
            source_id: &source.id_ref,
            target_key: self.natives[self.framework].input.key,
            target_subject_type: target.subject_type,
            target_id: &target.id_ref,
        }
    }
    /// Return exact sorted/deduplicated borrowed hrefs for the native report oracle only.
    fn policy_sources(
        &self,
        id: &str,
        charge: &mut dyn FnMut(usize) -> WorkResult<()>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Vec<&str>> {
        fence(control)?;
        charge(self.facts.get(id).map_or(0, |facts| facts.sources.len()))?;
        let mut sources: Vec<_> = self
            .facts
            .get(id)
            .into_iter()
            .flat_map(|facts| &facts.sources)
            .map(|(collection, mapping)| {
                self.mappings[*collection].collection.mapping_collection.mappings[*mapping]
                    .source_resource
                    .href
                    .as_str()
            })
            .collect();
        sources.sort_unstable();
        sources.dedup();
        fence(control)?;
        Ok(sources)
    }
    /// Compare every stored native report field to full current facts, with empty filters.
    pub(crate) fn validate_report(
        &self,
        bytes: &[u8],
        charge: &mut dyn FnMut(usize) -> WorkResult<()>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        fence(control)?;
        let value = strict(bytes, MAX_DOMAIN)?;
        admit_json(&value, 0, charge, control)?;
        let report = domain(super::parse_stored_report(bytes))?;
        self.compare_report_header(&report, charge, control)?;
        for (actual, expected) in report.controls.iter().zip(self.control_rows()) {
            fence(control)?;
            self.compare_control(actual, &expected, charge, control)?;
        }
        let mut queue = report.review_queue.iter();
        for expected in self.control_rows() {
            fence(control)?;
            if let Some(reason) = model::review_reason(expected.classification) {
                let actual = queue.next().ok_or_else(invalid)?;
                let decision = self.decision(expected.control_id);
                let sources = self.policy_sources(expected.control_id, charge, control)?;
                if actual.control_id != expected.control_id
                    || actual.reason_code != reason
                    || actual.owner.as_deref()
                        != decision.and_then(|row| row.reviewer_key.as_deref())
                    || actual.revisit_date.as_deref()
                        != decision.and_then(|row| row.revisit_date.as_deref())
                    || actual.policy_sources.iter().map(String::as_str).ne(sources)
                {
                    return Err(invalid());
                }
            }
        }
        if queue.next().is_some() {
            return Err(invalid());
        }
        fence(control)
    }
    /// Compare all full native header/resource/reviewer/count fields without a computed report Value.
    fn compare_report_header(
        &self,
        report: &model::ApplicabilityReport,
        charge: &mut dyn FnMut(usize) -> WorkResult<()>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        let framework =
            self.natives[self.framework].loaded.as_ref().ok_or_else(invalid)?.evidence();
        let filters = &report.filters;
        if report.schema_version != model::REPORT_SCHEMA_VERSION
            || report.manifest_sha256 != self.manifest_sha256
            || report.framework != *framework
            || report.matched_controls != self.counts.total
            || report.controls.len() != self.counts.total
            || !same_counts(&report.counts, &self.counts)
            || filters.group.is_some()
            || filters.control_prefix.is_some()
            || filters.state.is_some()
            || filters.reviewer.is_some()
            || filters.policy_source.is_some()
            || report.reviewers.len() != self.manifest.reviewers.len()
            || report.mapping_collections.len() != self.mappings.len()
        {
            return Err(invalid());
        }
        if report
            .reviewers
            .iter()
            .zip(&self.manifest.reviewers)
            .any(|(a, b)| a.key != b.key || a.party_type != b.party_type || a.name != b.name)
        {
            return Err(invalid());
        }
        fence(control)?;
        charge(self.mappings.len())?;
        let mut mappings: Vec<_> = self.mappings.iter().map(|row| &row.evidence).collect();
        mappings.sort_unstable_by(|a, b| a.uuid.cmp(&b.uuid));
        if report.mapping_collections.iter().zip(mappings).any(|(a, b)| !same_mapping(a, b)) {
            return Err(invalid());
        }
        Ok(())
    }
    /// Compare complete actual row fields and original private review metadata without disclosing it.
    fn compare_control(
        &self,
        actual: &model::ControlResult,
        row: &ControlRow<'_>,
        charge: &mut dyn FnMut(usize) -> WorkResult<()>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        let decision = self.decision(row.control_id);
        let facts = self.facts.get(row.control_id);
        let sources = self.policy_sources(row.control_id, charge, control)?;
        if actual.control_id != row.control_id
            || actual.classification != row.classification
            || actual.groups != self.framework_inventory()?.groups_for_control(row.control_id)
            || actual.reviewer_key.as_deref() != decision.and_then(|d| d.reviewer_key.as_deref())
            || actual.reviewed_at.as_deref() != decision.and_then(|d| d.reviewed_at.as_deref())
            || actual.rationale.as_deref() != decision.and_then(|d| d.rationale.as_deref())
            || actual.revisit_date.as_deref() != decision.and_then(|d| d.revisit_date.as_deref())
            || actual.note.as_deref() != decision.and_then(|d| d.note.as_deref())
            || actual.positive_mapping_count != facts.map_or(0, |f| f.positive)
            || actual.no_relationship_count != facts.map_or(0, |f| f.no_relationship)
            || actual.policy_sources.iter().map(String::as_str).ne(sources)
        {
            return Err(invalid());
        }
        Ok(())
    }
}

/// Consume the maintained classifier with native per-edge counts, never trace pair totals.
fn classify(
    decision: Option<&manifest::ControlDecision>,
    facts: Option<&Facts>,
) -> model::GapClassification {
    model::classify(
        decision,
        facts.map_or(0, |row| row.positive),
        facts.map_or(0, |row| row.no_relationship),
    )
}

/// Compare all seven complete native classification totals.
fn same_counts(a: &model::ClassificationCounts, b: &model::ClassificationCounts) -> bool {
    a.total == b.total
        && a.applicable_mapped == b.applicable_mapped
        && a.applicable_reviewed_no_relationship == b.applicable_reviewed_no_relationship
        && a.applicable_unmapped == b.applicable_unmapped
        && a.not_applicable == b.not_applicable
        && a.deferred == b.deferred
        && a.under_review == b.under_review
}

/// Compare complete current native Mapping provenance/resources, not minimized MCP rows.
fn same_mapping(a: &model::MappingEvidence, b: &model::MappingEvidence) -> bool {
    a.uuid == b.uuid
        && a.raw_sha256 == b.raw_sha256
        && a.version == b.version
        && a.oscal_version == b.oscal_version
        && a.reviewed_at == b.reviewed_at
        && a.source_resources == b.source_resources
        && a.reviewers.len() == b.reviewers.len()
        && a.reviewers.iter().zip(&b.reviewers).all(|(a, b)| {
            a.uuid == b.uuid && a.reviewer_type == b.reviewer_type && a.name == b.name
        })
}

/// Apply exact raw/depth/string bounds before any intrinsic native/domain admission.
fn strict(bytes: &[u8], limit: usize) -> WorkResult<Value> {
    if bytes.len() > limit {
        return Err(invalid());
    }
    domain(crate::json_strict::parse_value(
        bytes,
        "captured domain",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: 64 * 1024 },
    ))
}

/// Minimize ordinary private domain errors while never converting actual Work errors.
fn domain<T, E>(value: Result<T, E>) -> WorkResult<T> {
    value.map_err(|_| invalid())
}

/// Produce one closed ordinary invalid failure without paths, URI, prose or error text.
fn invalid() -> WorkError {
    WorkError::Failed(Error::invalid())
}

/// Preserve the actual shared sticky control on every full traversal and final fence.
fn fence(control: &mut dyn WorkControl) -> WorkResult<()> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if let Some(reason) = control.interruption() {
        return Err(WorkError::Interrupted(reason));
    }
    Ok(())
}

/// Genuine persisted native fixtures consume these same private helper ports.
#[cfg(test)]
#[path = "captured_tests.rs"]
mod tests;

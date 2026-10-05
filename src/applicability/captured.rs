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

/// One admitted native header and a lazily needed complete effective inventory.
struct NativePrepared<'a> {
    /// Actual caller-supplied borrowed original candidate.
    input: NativeInput<'a>,
    /// Maintained schema-admitted actual native identity and original hashes.
    evidence: ResourceEvidence,
    /// Complete maintained inventory only when the framework or a source needs it.
    loaded: Option<LoadedResource>,
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
    natives[framework].loaded =
        Some(load_native(&natives[framework].input, &manifest.framework, charge, control)?);
    let framework_loaded = natives[framework].loaded.as_ref().ok_or_else(invalid)?;
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
        let framework = prepared.natives[prepared.framework].loaded.as_ref().ok_or_else(invalid)?;
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
            prepared.natives[source].loaded =
                Some(load_native(&input, &bare_resource(&input), charge, control)?);
        }
        let loaded = prepared.natives[source].loaded.as_ref().ok_or_else(invalid)?;
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
        self.natives[self.framework].loaded.as_ref().map(|row| &row.inventory).ok_or_else(invalid)
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
            .flat_map(|row| row.inventory.ids_of_type_refs(SubjectType::Control))
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
        let framework = &self.natives[self.framework].loaded.as_ref().ok_or_else(invalid)?.evidence;
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

pub(crate) mod admitted {
    //! First review-profile admission over the maintained captured applicability oracle.
    //!
    //! This module does not capture files, approve an artifact or construct a proof.
    //! Complete borrowed-input preflight reserves conservative logical payload and
    //! registries before calling the unchanged engine. Parser and schema internals
    //! remain bounded by raw/depth/string admission, not a total-heap or preemption
    //! guarantee. Conservative refusal inside the legacy raw domain is intentional.

    use std::cell::{Cell, RefCell};

    use serde_json::Value;

    use super::{MappingInput, NativeInput, PreparedApplicability};
    use crate::mapping::inventory;
    use crate::mapping::manifest::{ResourceType, SubjectType};
    use crate::workspace::preparation::{
        ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
    };

    /// Complete caller-owned review work; none of these counters creates authority.
    #[derive(Clone, Copy, Debug)]
    pub(crate) enum ApplicabilityCharge {
        /// Observe the caller's first already-latched stop without resetting control.
        Checkpoint,
        /// Precharge complete repeated structural, primitive and matching work.
        Work {
            /// Complete structural inspection occurrences, including repeats.
            visits: usize,
            /// Complete conservative primitive/comparison byte extent.
            byte_work: usize,
            /// Complete candidate/registry comparison occurrences.
            matching_steps: usize,
        },
        /// Reserve logical owned payload before downstream clones or registry growth.
        Reserve {
            /// Conservative monotonic logical storage, not literal allocator usage.
            logical_bytes: usize,
        },
        /// Latch the caller's typed capacity error for checked overflow/profile limits.
        Capacity,
    }

    /// Preserve domain refusal, the original caller admission error and actual work stop.
    #[derive(Debug)]
    pub(crate) enum ApplicabilityAdmissionError<E> {
        /// Intrinsic domain syntax, native semantics or complete report relation failed.
        Domain,
        /// The caller's first actual monotonic admission failure, preserved unchanged.
        Admission(E),
        /// The supplied actual control stopped or failed; no no-op is substituted.
        Work(WorkError),
    }

    /// Complete first-profile current facts plus their precharged original denominator.
    pub(crate) struct AdmittedApplicability<'a> {
        /// Unchanged maintained engine facts; this is not lifecycle approval.
        core: PreparedApplicability<'a>,
        /// Complete expected private-string comparison upper bound, without disclosure.
        comparison_bytes: usize,
        /// Complete expected registry probes before each report comparison.
        comparison_steps: usize,
        /// Complete possible policy locator occurrences, including repeated targets.
        policy_slots: usize,
    }

    impl AdmittedApplicability<'_> {
        /// Borrow unchanged native semantics without a new currentness/proof constructor.
        pub(crate) fn core(&self) -> &PreparedApplicability<'_> {
            &self.core
        }

        /// Precharge all typed/report copies and expected comparisons before the oracle.
        pub(crate) fn validate_report_admitted<E>(
            &self,
            bytes: &[u8],
            admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
            control: &mut dyn WorkControl,
        ) -> Result<(), ApplicabilityAdmissionError<E>> {
            step(admit, control)?;
            let value = parse(bytes, admit, control)?;
            let metrics = tree(&value, 0, admit, control)?;
            drop(value);
            // The legacy oracle holds a strict Value, reparses, clones for typed
            // decoding and constructs its round-trip Value. Five complete payloads
            // plus node headers conservatively cover those logical forms.
            reserve(mul(metrics.owned, 5, admit)?, admit)?;
            work(
                0,
                add(self.comparison_bytes, mul(metrics.text, 32, admit)?, admit)?,
                self.comparison_steps,
                admit,
            )?;
            reserve(mul(self.policy_slots, 32, admit)?, admit)?;
            reserve(mul(self.core.mappings.len(), 16, admit)?, admit)?;
            legacy(admit, control, |charge, control| {
                self.core.validate_report(bytes, charge, control)
            })
        }
    }

    /// Prepare the complete review profile using the same actual control and engine.
    ///
    /// All native candidates and companions are accounted even when later unused.
    /// Compatible repeated originals can therefore conservatively cost more than
    /// the IO owner's distinct-original raw pool. There is no second raw pool or
    /// prefix admission. The caller retains/rechecks the actual original union.
    pub(crate) fn prepare_admitted<'a, E>(
        manifest_bytes: &'a [u8],
        framework_key: &str,
        natives: &[NativeInput<'a>],
        mappings: &[MappingInput<'a>],
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<AdmittedApplicability<'a>, ApplicabilityAdmissionError<E>> {
        step(admit, control)?;
        if natives.len() > 100 || mappings.len() > 100 {
            return capacity(admit);
        }
        let mut plan = Plan::default();
        work(0, framework_key.len(), natives.len(), admit)?;
        let manifest = parse(manifest_bytes, admit, control)?;
        let metrics = tree(&manifest, 0, admit, control)?;
        reserve(mul(metrics.owned, 3, admit)?, admit)?;
        plan.text = add(plan.text, metrics.text, admit)?;
        plan.nodes = add(plan.nodes, metrics.nodes, admit)?;
        if let Some(decisions) = manifest.get("decisions").and_then(Value::as_array) {
            for decision in decisions {
                step(admit, control)?;
                let id = string(decision, "control_id")?;
                reserve(add(id.len(), 128, admit)?, admit)?;
            }
        }
        drop(manifest);
        for input in natives {
            step(admit, control)?;
            let value = parse(input.bytes, admit, control)?;
            let metrics = tree(&value, 0, admit, control)?;
            reserve(mul(metrics.owned, 4, admit)?, admit)?;
            reserve(add(mul(input.key.len(), 4, admit)?, 512, admit)?, admit)?;
            work(0, mul(input.key.len(), 100, admit)?, natives.len(), admit)?;
            plan.text = add(plan.text, input.key.len(), admit)?;
            plan.text = add(plan.text, metrics.text, admit)?;
            plan.nodes = add(plan.nodes, metrics.nodes, admit)?;
            if input.resource_type == ResourceType::Catalog {
                catalog(&value, &mut plan, admit, control)?;
            }
            if let Some(companion) = input.resolved_catalog {
                let resolved = parse(companion.bytes, admit, control)?;
                let metrics = tree(&resolved, 0, admit, control)?;
                reserve(mul(metrics.owned, 4, admit)?, admit)?;
                reserve(add(mul(companion.key.len(), 4, admit)?, 256, admit)?, admit)?;
                plan.text = add(plan.text, companion.key.len(), admit)?;
                plan.text = add(plan.text, metrics.text, admit)?;
                plan.nodes = add(plan.nodes, metrics.nodes, admit)?;
                catalog(&resolved, &mut plan, admit, control)?;
            }
        }
        for input in mappings {
            step(admit, control)?;
            work(0, mul(input.key.len(), 100, admit)?, mappings.len(), admit)?;
            plan.text = add(plan.text, input.key.len(), admit)?;
            let value = parse(input.bytes, admit, control)?;
            let metrics = tree(&value, 0, admit, control)?;
            reserve(mul(metrics.owned, 5, admit)?, admit)?;
            plan.text = add(plan.text, metrics.text, admit)?;
            plan.nodes = add(plan.nodes, metrics.nodes, admit)?;
            mapping(&value, natives.len(), &mut plan, admit, control)?;
        }
        let comparison_bytes = mul(
            add(add(plan.text, plan.ancestry_text, admit)?, plan.policy_text, admit)?,
            32,
            admit,
        )?;
        let comparison_steps = mul(add(plan.nodes, plan.pairs, admit)?, 32, admit)?;
        work(0, comparison_bytes, comparison_steps, admit)?;
        step(admit, control)?;
        let core = legacy(admit, control, |charge, control| {
            super::prepare(manifest_bytes, framework_key, natives, mappings, charge, control)
        })?;
        Ok(AdmittedApplicability {
            core,
            comparison_bytes,
            comparison_steps,
            policy_slots: plan.targets,
        })
    }

    /// Fixed-size complete preflight accumulators; no per-input cloned roster.
    #[derive(Default)]
    struct Plan {
        /// All private primitive/key bytes from complete supplied originals.
        text: usize,
        /// Complete decoded occurrences from all originals, including repeats.
        nodes: usize,
        /// Complete ancestor-string occurrences retained per control/group.
        ancestry_text: usize,
        /// Complete href occurrences participating in target facts/report comparisons.
        policy_text: usize,
        /// Complete eligible native subjects across supplied effective inventories.
        subjects: usize,
        /// Complete native map count, before selection or filtering.
        maps: usize,
        /// Complete Cartesian source-target inspection denominator.
        pairs: usize,
        /// Complete native target occurrences, distinct from Cartesian pairs.
        targets: usize,
    }

    /// Fixed-size borrowed-tree measurements; canonical bound includes every field.
    #[derive(Default)]
    struct Metrics {
        /// Complete node occurrences in this subtree.
        nodes: usize,
        /// Complete primitive and object-key UTF-8 bytes.
        text: usize,
        /// Conservative logical node headers plus primitive/key payload.
        owned: usize,
        /// Conservative canonical encoded bytes, including finite numeric formatting.
        canonical: usize,
        /// Complete recursive object-key sorting reference slots.
        sort_slots: usize,
    }

    /// Inspect a bounded temporary strict tree before downstream typed forms grow.
    fn tree<E>(
        value: &Value,
        depth: usize,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<Metrics, ApplicabilityAdmissionError<E>> {
        step(admit, control)?;
        if depth > 64 {
            return capacity(admit);
        }
        work(1, 0, 0, admit)?;
        let mut result = Metrics { nodes: 1, owned: 128, ..Metrics::default() };
        match value {
            Value::Null => result.canonical = 4,
            Value::Bool(_) => result.canonical = 5,
            // The maintained canonical writer formats finite i64/u64/f64. Its
            // fixed-point integral branch is below 1e21; a finite f64 decimal
            // expansion (including the smallest subnormal) needs fewer than 1024
            // bytes. This deliberately exceeds compact JSON's numeric width.
            Value::Number(_) => result.canonical = 1024,
            Value::String(text) => {
                work(0, text.len(), 0, admit)?;
                result.text = text.len();
                result.owned = add(result.owned, text.len(), admit)?;
                result.canonical = add(mul(text.len(), 6, admit)?, 2, admit)?;
            }
            Value::Array(values) => {
                result.canonical = add(2, values.len(), admit)?;
                for child in values {
                    merge_metrics(&mut result, &tree(child, depth + 1, admit, control)?, admit)?;
                }
            }
            Value::Object(fields) => {
                result.canonical = add(2, mul(fields.len(), 2, admit)?, admit)?;
                result.sort_slots = fields.len();
                for (key, child) in fields {
                    work(0, key.len(), 0, admit)?;
                    result.text = add(result.text, key.len(), admit)?;
                    result.owned = add(result.owned, key.len(), admit)?;
                    result.canonical =
                        add(result.canonical, add(mul(key.len(), 6, admit)?, 2, admit)?, admit)?;
                    merge_metrics(&mut result, &tree(child, depth + 1, admit, control)?, admit)?;
                }
            }
        }
        if result.nodes > 100_000 {
            return capacity(admit);
        }
        Ok(result)
    }

    /// Add complete child metrics with checked arithmetic, never a truncated prefix.
    fn merge_metrics<E>(
        parent: &mut Metrics,
        child: &Metrics,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        parent.nodes = add(parent.nodes, child.nodes, admit)?;
        parent.text = add(parent.text, child.text, admit)?;
        parent.owned = add(parent.owned, child.owned, admit)?;
        parent.canonical = add(parent.canonical, child.canonical, admit)?;
        parent.sort_slots = add(parent.sort_slots, child.sort_slots, admit)?;
        Ok(())
    }

    /// Charge complete actual effective Catalog inventory expansions before loading.
    fn catalog<E>(
        value: &Value,
        plan: &mut Plan,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        let root = value.get("catalog").ok_or(ApplicabilityAdmissionError::Domain)?;
        inventory_nodes(root, 0, 0, 0, plan, admit, control)
    }

    /// Charge actual copied ancestry and repeated canonical buffers by traversal kind.
    fn inventory_nodes<E>(
        value: &Value,
        depth: usize,
        ancestor_slots: usize,
        ancestor_bytes: usize,
        plan: &mut Plan,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        step(admit, control)?;
        if depth > inventory::MAX_INVENTORY_DEPTH {
            return capacity(admit);
        }
        for key in ["groups", "controls", "parts"] {
            if let Some(rows) = value.get(key).and_then(Value::as_array) {
                for row in rows {
                    step(admit, control)?;
                    let mut next_slots = ancestor_slots;
                    let mut next_bytes = ancestor_bytes;
                    let id = row.get("id").and_then(Value::as_str).unwrap_or("");
                    if key == "groups" {
                        let copied = add(ancestor_bytes, mul(ancestor_slots, 24, admit)?, admit)?;
                        reserve(
                            add(copied, add(mul(id.len(), 3, admit)?, 192, admit)?, admit)?,
                            admit,
                        )?;
                        plan.ancestry_text = add(plan.ancestry_text, ancestor_bytes, admit)?;
                        if !id.trim().is_empty() {
                            next_slots = add(next_slots, 1, admit)?;
                            next_bytes = add(next_bytes, id.len(), admit)?;
                        }
                    } else if key == "controls"
                        || row.get("name").and_then(Value::as_str) == Some("statement")
                    {
                        plan.subjects = add(plan.subjects, 1, admit)?;
                        if plan.subjects > 100_000 {
                            return capacity(admit);
                        }
                        let subtree = tree(row, 0, admit, control)?;
                        reserve(
                            add(subtree.canonical, mul(subtree.sort_slots, 8, admit)?, admit)?,
                            admit,
                        )?;
                        // IDs can be copied into IDs, subjects, excerpts, control
                        // groups and the loader snapshot. Six occurrences include
                        // conservative room; excerpts are at most 160 Unicode scalars.
                        reserve(add(mul(id.len(), 6, admit)?, 896, admit)?, admit)?;
                        work(
                            0,
                            add(
                                mul(subtree.canonical, 2, admit)?,
                                mul(subtree.text, 32, admit)?,
                                admit,
                            )?,
                            mul(subtree.nodes, 32, admit)?,
                            admit,
                        )?;
                        if key == "controls" {
                            reserve(
                                add(ancestor_bytes, mul(ancestor_slots, 24, admit)?, admit)?,
                                admit,
                            )?;
                            plan.ancestry_text = add(plan.ancestry_text, ancestor_bytes, admit)?;
                        }
                    } else if let Some(name) = row.get("name").and_then(Value::as_str) {
                        reserve(add(add(id.len(), name.len(), admit)?, 128, admit)?, admit)?;
                    }
                    inventory_nodes(row, depth + 1, next_slots, next_bytes, plan, admit, control)?;
                }
            }
        }
        Ok(())
    }

    /// Charge full native maps, target participation and Cartesian registries before decode.
    fn mapping<E>(
        value: &Value,
        native_count: usize,
        plan: &mut Plan,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        let groups = value
            .get("mapping-collection")
            .and_then(|root| root.get("mappings"))
            .and_then(Value::as_array)
            .ok_or(ApplicabilityAdmissionError::Domain)?;
        for group in groups {
            step(admit, control)?;
            let source = group.get("source-resource").ok_or(ApplicabilityAdmissionError::Domain)?;
            let source_extent = tree(source, 0, admit, control)?;
            // All native headers are probed; evidence comparison never selects the
            // first hash-only match. Private source descriptors can be retained in
            // the shared evidence registry and complete report evidence.
            work(
                0,
                mul(source_extent.text, mul(native_count, 32, admit)?, admit)?,
                native_count,
                admit,
            )?;
            reserve(mul(source_extent.owned, 4, admit)?, admit)?;
            let href = string(source, "href")?;
            let maps = group
                .get("maps")
                .and_then(Value::as_array)
                .ok_or(ApplicabilityAdmissionError::Domain)?;
            for edge in maps {
                step(admit, control)?;
                let sources = array(edge, "sources")?;
                let targets = array(edge, "targets")?;
                let pairs = mul(sources.len(), targets.len(), admit)?;
                plan.maps = add(plan.maps, 1, admit)?;
                plan.pairs = add(plan.pairs, pairs, admit)?;
                plan.targets = add(plan.targets, targets.len(), admit)?;
                if plan.maps > 10_000 || plan.pairs > 100_000 {
                    return capacity(admit);
                }
                reserve(mul(pairs, 256, admit)?, admit)?;
                reserve(mul(add(sources.len(), targets.len(), admit)?, 256, admit)?, admit)?;
                let mut source_bytes = 0;
                for item in sources {
                    let id = string(item, "id-ref")?;
                    source_bytes = add(source_bytes, id.len(), admit)?;
                    reserve(add(mul(id.len(), 4, admit)?, 256, admit)?, admit)?;
                }
                let mut target_bytes = 0;
                for item in targets {
                    let id = string(item, "id-ref")?;
                    target_bytes = add(target_bytes, id.len(), admit)?;
                    reserve(add(mul(id.len(), 3, admit)?, 256, admit)?, admit)?;
                }
                let repeated_ids = add(
                    mul(source_bytes, targets.len(), admit)?,
                    mul(target_bytes, sources.len(), admit)?,
                    admit,
                )?;
                // Conservative repeated interning/lookup work and string payload
                // before source-subject and polarity registries can grow.
                reserve(add(repeated_ids, mul(pairs, 64, admit)?, admit)?, admit)?;
                work(
                    0,
                    mul(add(repeated_ids, mul(pairs, 64, admit)?, admit)?, 32, admit)?,
                    mul(pairs, 8, admit)?,
                    admit,
                )?;
                plan.policy_text =
                    add(plan.policy_text, mul(href.len(), targets.len(), admit)?, admit)?;
                reserve(mul(targets.len(), 192, admit)?, admit)?;
                reserve(add(string(edge, "uuid")?.len(), 64, admit)?, admit)?;
            }
        }
        Ok(())
    }

    /// Parse only after raw/schema-parser admission; no total parser-heap claim.
    fn parse<E>(
        bytes: &[u8],
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<Value, ApplicabilityAdmissionError<E>> {
        step(admit, control)?;
        if bytes.len() > 10 * 1024 * 1024 {
            return capacity(admit);
        }
        // Preflight plus maintained strict/schema/typed/hash passes. Parser and
        // validator internal allocation is qualified separately from retained
        // payload/registry storage; the parser still consumes the same deadline.
        work(0, mul(bytes.len(), 10, admit)?, 0, admit)?;
        let slots = preparse_slots(bytes, admit, control)?;
        reserve(add(add(mul(slots, 128, admit)?, bytes.len(), admit)?, 512, admit)?, admit)?;
        let result = crate::json_strict::parse_value(
            bytes,
            "captured applicability review",
            crate::json_strict::Limits { max_depth: 64, max_string_bytes: 64 * 1024 },
        );
        step(admit, control)?;
        result.map_err(|_| ApplicabilityAdmissionError::Domain)
    }

    /// Bound temporary decoded node/payload storage without retaining a decoded tree.
    ///
    /// On valid JSON every Value begins with a container, a quoted string or one
    /// primitive token. Object keys are additionally counted as string slots. Raw
    /// bytes bound all decoded key/string payload. This scan does not admit syntax,
    /// duplicates or authority; the maintained strict parser still decides those.
    fn preparse_slots<E>(
        bytes: &[u8],
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<usize, ApplicabilityAdmissionError<E>> {
        let mut slots = 0usize;
        let mut quoted = false;
        let mut escaped = false;
        let mut primitive = false;
        for chunk in bytes.chunks(32 * 1024) {
            step(admit, control)?;
            for &byte in chunk {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        quoted = false;
                    }
                    continue;
                }
                match byte {
                    b'"' => {
                        slots = add(slots, 1, admit)?;
                        quoted = true;
                        primitive = false;
                    }
                    b'{' | b'[' => {
                        slots = add(slots, 1, admit)?;
                        primitive = false;
                    }
                    b'}' | b']' | b',' | b':' | b' ' | b'\r' | b'\n' | b'\t' => primitive = false,
                    _ if !primitive => {
                        slots = add(slots, 1, admit)?;
                        primitive = true;
                    }
                    _ => {}
                }
            }
        }
        work(slots, 0, 0, admit)?;
        Ok(slots)
    }

    /// Borrow one required native/string field; no caller-owned values are synthesized.
    fn string<'a, E>(
        value: &'a Value,
        key: &str,
    ) -> Result<&'a str, ApplicabilityAdmissionError<E>> {
        value.get(key).and_then(Value::as_str).ok_or(ApplicabilityAdmissionError::Domain)
    }

    /// Borrow a complete actual native array; no prefix or detached reference registry.
    fn array<'a, E>(
        value: &'a Value,
        key: &str,
    ) -> Result<&'a [Value], ApplicabilityAdmissionError<E>> {
        value
            .get(key)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .ok_or(ApplicabilityAdmissionError::Domain)
    }

    /// Send complete repeated work to the caller's single monotonic ledger.
    fn work<E>(
        visits: usize,
        byte_work: usize,
        matching_steps: usize,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        admit(ApplicabilityCharge::Work { visits, byte_work, matching_steps })
            .map_err(ApplicabilityAdmissionError::Admission)
    }

    /// Reserve logical payload before a downstream typed form/registry is installed.
    fn reserve<E>(
        logical_bytes: usize,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        admit(ApplicabilityCharge::Reserve { logical_bytes })
            .map_err(ApplicabilityAdmissionError::Admission)
    }

    /// Preserve caller admission first, then observe the actual immutable work control.
    fn step<E>(
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
    ) -> Result<(), ApplicabilityAdmissionError<E>> {
        admit(ApplicabilityCharge::Checkpoint).map_err(ApplicabilityAdmissionError::Admission)?;
        super::fence(control).map_err(ApplicabilityAdmissionError::Work)
    }

    /// Ask the actual caller to latch Capacity; an accepting callback still cannot continue.
    fn capacity<T, E>(
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    ) -> Result<T, ApplicabilityAdmissionError<E>> {
        match admit(ApplicabilityCharge::Capacity) {
            Err(error) => Err(ApplicabilityAdmissionError::Admission(error)),
            Ok(()) => Err(ApplicabilityAdmissionError::Domain),
        }
    }

    /// Add bounded denominators without overflow or silent saturating admission.
    fn add<E>(
        left: usize,
        right: usize,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    ) -> Result<usize, ApplicabilityAdmissionError<E>> {
        left.checked_add(right).map_or_else(|| capacity(admit), Ok)
    }

    /// Multiply complete occurrence counts without an overflow/prefix exception.
    fn mul<E>(
        left: usize,
        right: usize,
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    ) -> Result<usize, ApplicabilityAdmissionError<E>> {
        left.checked_mul(right).map_or_else(|| capacity(admit), Ok)
    }

    /// Chronological first cause, shared only by one stack-owned legacy invocation.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum FirstCause {
        /// The actual callback returned its typed admission failure first.
        Admission,
        /// The real forwarding control returned/observed its stop first.
        Work,
    }

    /// Retain only an actual control failure while delegating every original checkpoint.
    struct ObservedControl<'a> {
        /// Real caller control; no fresh deadline or no-op is created.
        actual: &'a mut dyn WorkControl,
        /// First actual ordinary or interrupted work failure, including shared observations.
        failed: RefCell<Option<WorkError>>,
        /// Shared chronological discriminator; no generic error is converted to Work.
        first: &'a Cell<Option<FirstCause>>,
    }

    impl WorkControl for ObservedControl<'_> {
        /// Observe actual control failure before a private domain adapter can minimize it.
        fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
            let result = self.actual.checkpoint(stage, progress);
            if let Err(error) = &result {
                if self.first.get().is_none() {
                    self.first.set(Some(FirstCause::Work));
                }
                let mut failed = self.failed.borrow_mut();
                if failed.is_none() {
                    *failed = Some(match error {
                        WorkError::Failed(error) => WorkError::Failed(error.clone()),
                        WorkError::Interrupted(reason) => WorkError::Interrupted(*reason),
                    });
                }
            }
            result
        }

        /// Borrow and retain the original sticky interruption without replacing or resetting it.
        fn interruption(&self) -> Option<crate::workspace::preparation::Interruption> {
            let reason = self.actual.interruption();
            if let Some(reason) = reason {
                if self.first.get().is_none() {
                    self.first.set(Some(FirstCause::Work));
                }
                let mut failed = self.failed.borrow_mut();
                if failed.is_none() {
                    *failed = Some(WorkError::Interrupted(reason));
                }
            }
            reason
        }
    }

    /// Delegate maintained semantics while preserving the chronological first actual cause.
    fn legacy<T, E>(
        admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
        control: &mut dyn WorkControl,
        run: impl FnOnce(&mut dyn FnMut(usize) -> WorkResult<()>, &mut dyn WorkControl) -> WorkResult<T>,
    ) -> Result<T, ApplicabilityAdmissionError<E>> {
        let first = Cell::new(None);
        let mut admission = None;
        let mut observed =
            ObservedControl { actual: control, failed: RefCell::new(None), first: &first };
        let result = {
            let mut visit = |amount| match admit(ApplicabilityCharge::Work {
                visits: amount,
                byte_work: 0,
                matching_steps: 0,
            }) {
                Ok(()) => Ok(()),
                Err(error) => {
                    if first.get().is_none() {
                        first.set(Some(FirstCause::Admission));
                    }
                    if admission.is_none() {
                        admission = Some(error);
                    }
                    Err(super::invalid())
                }
            };
            run(&mut visit, &mut observed)
        };
        match first.get() {
            Some(FirstCause::Admission) => {
                return Err(admission.map_or(
                    ApplicabilityAdmissionError::Domain,
                    ApplicabilityAdmissionError::Admission,
                ));
            }
            Some(FirstCause::Work) => {
                return Err(observed.failed.into_inner().map_or(
                    ApplicabilityAdmissionError::Domain,
                    ApplicabilityAdmissionError::Work,
                ));
            }
            None => {}
        }
        match result {
            Ok(value) => Ok(value),
            Err(error @ WorkError::Interrupted(_)) => Err(ApplicabilityAdmissionError::Work(error)),
            Err(WorkError::Failed(_)) => Err(ApplicabilityAdmissionError::Domain),
        }
    }

    impl PreparedApplicability<'_> {
        /// Borrow the actual current native control fingerprint, or refuse a missing control.
        pub(crate) fn control_fingerprint(&self, id: &str) -> Option<&str> {
            self.natives[self.framework]
                .loaded
                .as_ref()
                .and_then(|row| row.inventory.fingerprint(SubjectType::Control, id))
        }

        /// Borrow only the actual source decision at its exact original array position.
        pub(crate) fn explicit_decision(
            &self,
            index: usize,
        ) -> Option<&super::manifest::ControlDecision> {
            self.manifest.decisions.get(index)
        }

        /// Return the exact original decision-array locator; omission stays None.
        pub(crate) fn explicit_decision_index(&self, id: &str) -> Option<usize> {
            self.decisions.get(id).copied()
        }

        /// Preserve native per-edge/target participation instead of multiplying by pairs.
        pub(crate) fn native_participation(&self, id: &str) -> (usize, usize) {
            self.facts.get(id).map_or((0, 0), |row| (row.positive, row.no_relationship))
        }

        /// Borrow complete actual framework evidence without creating an approved tuple.
        pub(crate) fn framework_evidence(&self) -> Option<&inventory::ResourceEvidence> {
            self.natives[self.framework].loaded.as_ref().map(|row| &row.evidence)
        }

        /// Count every original native map across every admitted collection and group.
        pub(crate) fn complete_maps(&self) -> usize {
            self.mappings.iter().flat_map(|row| &row.map_ids).map(Vec::len).sum()
        }

        /// Count the complete retained Cartesian locator denominator, before any selection.
        pub(crate) fn complete_pair_inspections(&self) -> usize {
            self.relations.len()
        }
    }

    #[cfg(test)]
    /// Logical-budget arithmetic controls; these are not native or approval fixtures.
    mod budget_tests {
        use super::*;
        use crate::workspace::preparation::NoopControl;

        /// Extreme finite numbers and escaping are counted without a retained output buffer.
        #[test]
        fn canonical_budget_covers_extreme_numbers_and_escaped_non_ascii_keys() {
            let value = serde_json::json!({"\u{001b}\n\"é": [1e-100, 5e-324, -0.0, 0, 1,
                "\r\n\t\u{0085}\u{202e}é😀"]});
            let metrics = tree(&value, 0, &mut |_| Ok::<(), &'static str>(()), &mut NoopControl)
                .expect("small borrowed arithmetic fixture");
            assert!(metrics.canonical >= 5 * 1024);
            assert!(metrics.canonical > serde_json::to_vec(&value).unwrap().len());
            assert!(metrics.sort_slots > 0);
            assert!(metrics.text >= "\r\n\t\u{0085}\u{202e}é😀".len());
        }

        /// Anonymous nested groups still charge copied inherited string bytes.
        #[test]
        fn complete_long_ancestry_is_precharged_before_any_inventory_loader() {
            let id = "g".repeat(4096);
            let value = serde_json::json!({"catalog": {"groups": [{"id": id,
                "groups": [{"groups": [{"controls": [{"id": "c", "title": "T"}]}]}]}]}});
            let mut logical = 0usize;
            let mut plan = Plan::default();
            catalog(
                &value,
                &mut plan,
                &mut |charge| {
                    if let ApplicabilityCharge::Reserve { logical_bytes } = charge {
                        logical += logical_bytes;
                    }
                    Ok::<(), &'static str>(())
                },
                &mut NoopControl,
            )
            .expect("small complete borrowed inventory structure");
            assert_eq!(plan.subjects, 1);
            assert_eq!(plan.ancestry_text, 3 * 4096);
            assert!(logical > 6 * 4096);
        }

        /// Whole Cartesian overflow/capacity refuses before registry creation, never a prefix.
        #[test]
        fn over_bound_cartesian_roster_sends_typed_capacity_before_legacy_growth() {
            let endpoints: Vec<_> =
                (0..317).map(|i| serde_json::json!({"id-ref": format!("c{i}")})).collect();
            let value = serde_json::json!({"mapping-collection": {"mappings": [{
                "source-resource": {"href": "private/source"}, "maps": [{"uuid": "not-admitted",
                    "sources": endpoints, "targets": endpoints}]}]}});
            let mut plan = Plan::default();
            let mut reached_capacity = false;
            let error = mapping(
                &value,
                2,
                &mut plan,
                &mut |charge| {
                    if matches!(charge, ApplicabilityCharge::Capacity) {
                        reached_capacity = true;
                        Err("whole-Cartesian-cap")
                    } else {
                        Ok(())
                    }
                },
                &mut NoopControl,
            );
            assert!(reached_capacity);
            assert!(matches!(
                error,
                Err(ApplicabilityAdmissionError::Admission("whole-Cartesian-cap"))
            ));
            assert_eq!(plan.pairs, 100_489);
        }

        /// Temporary parse storage is reserved before the strict parser's first Value allocation.
        #[test]
        fn strict_preflight_value_is_reserved_before_parse_and_covers_complete_tree() {
            let raw = r#"{"key": [null, true, false, 1e-100, 5e-324, -0, "quoted \" ] } \\ end"], "nested": {"k": "é"}}"#.as_bytes();
            let mut reserves = Vec::new();
            let parsed = parse(
                raw,
                &mut |charge| {
                    if let ApplicabilityCharge::Reserve { logical_bytes } = charge {
                        reserves.push(logical_bytes);
                    }
                    Ok::<(), &'static str>(())
                },
                &mut NoopControl,
            )
            .expect("actual strict parse fixture");
            let metrics = tree(&parsed, 0, &mut |_| Ok::<(), &'static str>(()), &mut NoopControl)
                .expect("complete actual decoded tree");
            assert_eq!(reserves.len(), 1);
            assert!(reserves[0] >= metrics.owned);
            let error = parse(
                b"not JSON",
                &mut |charge| {
                    if matches!(charge, ApplicabilityCharge::Reserve { .. }) {
                        Err("before-parse-stop")
                    } else {
                        Ok(())
                    }
                },
                &mut NoopControl,
            );
            assert!(matches!(
                error,
                Err(ApplicabilityAdmissionError::Admission("before-parse-stop"))
            ));
        }

        /// A legacy fallback cannot reorder real earlier Work and typed Admission failures.
        #[test]
        fn caught_legacy_failures_preserve_the_actual_chronological_first_cause() {
            use crate::workspace::preparation::Interruption;
            use crate::workspace::preparation::test_support::Recorder;
            for work_first in [true, false] {
                let mut actual = Recorder::at(Stage::PrepareDomain, 1);
                let mut callback = |_| Err("actual-admission-stop");
                let result = legacy(&mut callback, &mut actual, |charge, control| {
                    if work_first {
                        let _ = control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged);
                        let _ = charge(1);
                    } else {
                        let _ = charge(1);
                        let _ = control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged);
                    }
                    Ok(()) // Deliberately model a fallback catching both genuine failures.
                });
                if work_first {
                    assert!(matches!(
                        result,
                        Err(ApplicabilityAdmissionError::Work(WorkError::Interrupted(
                            Interruption::CancelRequested
                        )))
                    ));
                } else {
                    assert!(matches!(
                        result,
                        Err(ApplicabilityAdmissionError::Admission("actual-admission-stop"))
                    ));
                }
            }
        }

        /// Malformed strict JSON still observes the real post-parser cancellation fence.
        #[test]
        fn malformed_parse_observes_the_after_parse_actual_work_stop() {
            use crate::workspace::preparation::Interruption;
            use crate::workspace::preparation::test_support::Recorder;
            assert!(matches!(
                parse(b"not JSON", &mut |_| Ok::<(), &'static str>(()), &mut NoopControl),
                Err(ApplicabilityAdmissionError::Domain)
            ));
            // One entry fence, one complete small lexical-scan chunk and the actual
            // post-parser fence. This is a deterministic stage seam, not a timer or
            // claim that the parser itself can be preempted.
            let mut actual = Recorder::at(Stage::PrepareDomain, 3);
            let result = parse(b"not JSON", &mut |_| Ok::<(), &'static str>(()), &mut actual);
            assert_eq!(actual.events.len(), 3);
            assert!(matches!(
                result,
                Err(ApplicabilityAdmissionError::Work(WorkError::Interrupted(
                    Interruption::CancelRequested
                )))
            ));
            assert_eq!(actual.interruption(), Some(Interruption::CancelRequested));
        }
    }
}

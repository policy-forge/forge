//! Read-only operator-selected disclosure gate over actual held originals.
//!
//! These private objects have no detached owner/hash/Value constructor. Operator
//! selection is a declared-record trust assumption, not owner authentication.
//! Transport is implemented separately; real client/corpus evaluation and owner acceptance
//! require their own evidence. Every publication must recheck the entire live proof.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::path::{Component, Path};
use std::rc::Rc;

use chrono::NaiveDate;
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use super::{
    ApprovalBasis, CapturedObject, CapturedProject, NativeIdentity, ProjectDeclaration,
    ResourcePolicy, Role, capture_failure, complete_closure, evaluate, fence, limit_failure,
    native_facts, policy_failure, resolve_record_path, strict_value, valid_hash, validate_policy,
    verify_original,
};
use crate::lifecycle::record::{self, LifecycleRecord};
use crate::lifecycle::status::{self, CurrentArtifacts, StatusReport};
use crate::linkage::fresh::{self, CapturedLocal, PreparedLocal, RootGeneration};
use crate::workspace::preparation::{Stage, WorkControl, WorkError, WorkResult};

/// Three originals share the same pool with every declared project dependency.
const CONFIG_PATHS: [&str; 3] =
    ["forge.mcp.json", "forge.mcp.visibility.json", "forge.mcp.disclosure-decision.json"];
/// Complete original observation count, including actual typed absences.
const ORIGINAL_LIMIT: usize = 1001;
/// One shared actual retained-original byte ceiling, not a heap guarantee.
const RAW_LIMIT: usize = 50 * 1024 * 1024;
/// Each original config is strictly admitted within this existing design bound.
const CONFIG_LIMIT: usize = 1024 * 1024;
/// Full input trees and derived relationship registries share a monotonic ceiling.
const RELATION_LIMIT: usize = 100_000;
/// Exact approved proposal binding; a different proposal needs a reviewed successor.
const PROPOSAL: &str = "d52da047f0166d9d8cd002221ee9608577f935e7c803969a178b91e797801f49";
/// Exact selected wire revision; the gate itself implements no protocol.
const PROTOCOL: &str = "2026-07-28";

/// Fixed unavailable branch or a genuinely captured private disclosure scope.
/// No project data is attached to unavailable and no detached scope is accepted.
pub(crate) enum DisclosureGate {
    /// Missing/invalid selected record; static protocol metadata only.
    Unavailable,
    /// Actual complete decision/config/project proof under operator selection.
    Validated(Box<CapturedQueryScope>),
}

/// Actual selected external decision and profile originals, retained for all queries.
struct PreparedDisclosureDecision {
    /// Held disjoint control root, never serialized or used for project lookup.
    root: Rc<RootGeneration>,
    /// Actual fixed decision file original, not a declared hash-only record.
    original: CapturedLocal,
    /// Actual fixed project profile original from the same project root.
    profile_original: CapturedLocal,
    /// Complete closed private record; owner/provenance fields never leave this module.
    record: Value,
    /// Complete exact profile; no normalization or dropped nullable fields.
    profile: Value,
    /// Actual expected/observed original decision pin supplied by the operator.
    decision_sha256: String,
    /// Actual expected/observed original profile pin supplied by the operator.
    profile_sha256: String,
    /// Actual observed discovery pin paired by the complete decision/profile.
    discovery_sha256: String,
}

/// Opaque one-worker scope; every original and parsed native is owned once here.
/// It is not `Send`; no unsafe cross-thread promotion or detached constructor exists.
pub(crate) struct CapturedQueryScope {
    /// Actual complete project originals, including hidden dependencies and absences.
    project: CapturedProject,
    /// Actual decision and profile originals and private complete scoped choices.
    decision: PreparedDisclosureDecision,
    /// Strict original JSON values aligned exactly with the ordered capture roster.
    values: Vec<Option<Value>>,
    /// Sealed immutable same-scope lifecycle projections, shared without repeated graph clones.
    approvals: RefCell<Vec<Option<Rc<LifecycleProof>>>>,
    /// Complete capture plus query relationship admissions, never reset in place.
    relationships: Cell<usize>,
    /// Digest of all actual original generations and chosen complete raw config pins.
    generation: String,
}

/// Borrowed actual original view with private construction; no path read or mutation port.
#[derive(Clone, Copy)]
pub(crate) struct CapturedResource<'a> {
    /// Real scope containing the held original and its admitted complete native value.
    scope: &'a CapturedQueryScope,
    /// Exact aligned actual roster position, never a caller-supplied external index.
    index: usize,
}

impl<'a> CapturedResource<'a> {
    /// Borrow the exact opaque declared key without copying or disclosing hidden counts.
    pub(crate) fn key(&self) -> &'a str {
        &self.object().policy.key
    }
    /// Borrow the declared role; domain-specific semantic admission remains required.
    pub(crate) fn role(&self) -> Role {
        self.object().policy.role
    }
    /// Borrow the internal portable project-relative path; never return it in public DTOs.
    pub(crate) fn path(&self) -> &'a str {
        &self.object().policy.path
    }
    /// Borrow the declared inert citation token after closed policy admission.
    pub(crate) fn citation_label(&self) -> &'a str {
        &self.object().policy.citation_label
    }
    /// Borrow the exact actual raw digest after the independent pin matched.
    pub(crate) fn raw_sha256(&self) -> &'a str {
        self.object().digest.as_deref().unwrap_or("")
    }
    /// Borrow only the held original bytes; this method never reopens a path.
    pub(crate) fn bytes(&self) -> &'a [u8] {
        match &self.object().original {
            CapturedLocal::Present(bytes, _) => bytes,
            CapturedLocal::Absent(_) => &[],
        }
    }
    /// Return the actual expected pin comparison, distinct from approval authority.
    pub(crate) fn pin_matches(&self) -> bool {
        self.object().pin_matches
    }
    /// Borrow the intrinsically validated native identity, absent for non-native roles.
    pub(crate) fn native_identity(&self) -> Option<&'a NativeIdentity> {
        self.object().native.as_ref()
    }
    /// Borrow only official-schema/expected-identity admitted actual native JSON.
    pub(crate) fn native_value(&self) -> Option<&'a Value> {
        self.role().native_model()?;
        self.scope.values[self.index].as_ref()
    }
    /// Borrow strict original JSON for a role-specific pure domain validator.
    /// Strict decoding alone establishes no current report or approval semantics.
    pub(crate) fn json(&self) -> Option<&'a Value> {
        self.scope.values[self.index].as_ref()
    }
    /// Borrow the exact declared lifecycle key used to resolve complete current proof.
    pub(crate) fn lifecycle_key(&self) -> Option<&'a str> {
        self.object().policy.lifecycle_key.as_deref()
    }
    /// Resolve the private aligned original without manufacturing another owner.
    fn object(&self) -> &'a CapturedObject {
        &self.scope.project.objects[self.index]
    }
}

/// Actual approved/current neutral proof paired with one borrowed original.
/// Private construction requires complete original/native/lifecycle admission.
pub(crate) struct ApprovedObject<'a> {
    /// Actual held native or source original, not copied source content.
    resource: CapturedResource<'a>,
    /// Sealed projection of this same immutable captured record/tuple, never detached input.
    proof: Rc<LifecycleProof>,
}

/// Exact selected domain source with a real recorded/current neutral lifecycle proof.
/// This is source candidacy only; domain validity and independent dependency approval
/// must still be established by the consuming query within this same captured scope.
pub(crate) struct DomainSourceCandidate<'a> {
    /// Exact actual held manifest/report original; no detached Value or hash constructor.
    resource: CapturedResource<'a>,
    /// Retained genuine record/current/neutral proof, with no public authority accessor.
    _proof: Rc<LifecycleProof>,
}

impl<'a> DomainSourceCandidate<'a> {
    /// Borrow the exact original with its real scope lifetime; this performs no IO.
    pub(crate) fn resource(&self) -> CapturedResource<'a> {
        self.resource
    }
}

/// Private first-derived lifecycle graph shared only within one retained original scope.
struct LifecycleProof {
    /// Intrinsically valid record from the actual captured lifecycle bytes.
    record: LifecycleRecord,
    /// Complete source/generated tuple resolved within the same actual roster.
    current: CurrentArtifacts,
    /// Schedule-neutral projection used to establish recorded/current approval.
    #[cfg_attr(not(test), allow(dead_code))]
    // Full intrinsic proof observations retained for precise regression inspection.
    neutral: StatusReport,
    /// Optional trusted-date projection on this exact record and tuple.
    evaluated: Option<StatusReport>,
}

impl<'a> ApprovedObject<'a> {
    /// Return the copyable borrowed resource view, retaining the scope lifetime.
    pub(crate) fn resource(&self) -> CapturedResource<'a> {
        self.resource
    }
    /// Borrow the actual private recorded lifecycle; adapters must minimize all outputs.
    pub(crate) fn record(&self) -> &LifecycleRecord {
        &self.proof.record
    }
    /// Borrow the complete actual captured tuple, never replace it with hash-only input.
    #[cfg_attr(not(test), allow(dead_code))] // Full intrinsic proof observations retained for precise regression inspection.
    pub(crate) fn current(&self) -> &CurrentArtifacts {
        &self.proof.current
    }
    /// Borrow the neutral projection; sensitive fields remain internal to query code.
    #[cfg_attr(not(test), allow(dead_code))] // Full intrinsic proof observations retained for precise regression inspection.
    pub(crate) fn neutral_status(&self) -> &StatusReport {
        &self.proof.neutral
    }
    /// Borrow optional advisory schedule facts; these cannot change recorded approval.
    pub(crate) fn evaluated_status(&self) -> Option<&StatusReport> {
        self.proof.evaluated.as_ref()
    }
}

impl CapturedQueryScope {
    /// Borrow only profile-visible keys; internal resource iteration includes hidden dependencies.
    pub(crate) fn visible_keys(&self) -> impl Iterator<Item = &str> {
        self.project.visible_keys.iter().map(String::as_str)
    }
    /// Borrow the sealed generation token for query/cursor binding, never approval authority.
    pub(crate) fn scope_generation(&self) -> &str {
        &self.generation
    }
    /// Permit only an explicitly selected actual tool; discovery is not permission.
    pub(crate) fn tool_enabled(&self, tool: &str) -> bool {
        contains(&self.decision.profile, "enabled_tools", tool)
    }
    /// Check exact source-text startup scope; per-query opt-in and actual spans remain required.
    pub(crate) fn excerpt_allowed(&self, key: &str) -> bool {
        self.decision.profile["source_text_mode"] == "exact-span-opt-in"
            && contains(&self.decision.profile, "source_text_keys", key)
            && self.project.visible_keys.iter().any(|visible| visible == key)
    }
    /// Borrow all admitted internal dependencies; query code must apply visible-key scope.
    pub(crate) fn resources(&self) -> impl Iterator<Item = CapturedResource<'_>> {
        (0..self.project.objects.len()).filter_map(|index| self.admitted(index))
    }
    /// Borrow an exact admitted dependency without resolving any undeclared path.
    pub(crate) fn resource(&self, key: &str) -> Option<CapturedResource<'_>> {
        let index = self.project.objects.iter().position(|object| object.policy.key == key)?;
        self.admitted(index)
    }
    /// Add complete occurrences before any query registry/map/vector growth.
    /// Failure leaves the previous count unchanged; duplicates and filtered facts still count.
    pub(crate) fn charge_relationships(&self, count: usize) -> WorkResult<()> {
        let next = self.relationships.get().checked_add(count).ok_or_else(limit_failure)?;
        if next > RELATION_LIMIT {
            return Err(limit_failure());
        }
        self.relationships.set(next);
        Ok(())
    }
    /// Recheck both held roots, all three configs and every present/absent original.
    /// Sequential checks are not an atomic snapshot and cannot retract transport prefixes.
    pub(crate) fn verify_inputs(&self, control: &mut dyn WorkControl) -> WorkResult<()> {
        fence(control, Stage::CaptureResource)?;
        fresh::verify_root(&self.decision.root).map_err(|_| capture_failure())?;
        if self.decision.record["visibility_profile"] != self.decision.profile
            || crate::hashing::sha256_hex(present_bytes(&self.decision.original)?)
                != self.decision.decision_sha256
            || crate::hashing::sha256_hex(present_bytes(&self.decision.profile_original)?)
                != self.decision.profile_sha256
            || crate::hashing::sha256_hex(present_bytes(&self.project.manifest)?)
                != self.decision.discovery_sha256
        {
            return Err(policy_failure());
        }
        verify_original(&self.decision.original)?;
        fence(control, Stage::CaptureResource)?;
        verify_original(&self.decision.profile_original)?;
        self.project.verify_inputs(control)?;
        fresh::verify_root(&self.decision.root).map_err(|_| capture_failure())?;
        fence(control, Stage::RetainPrepared)
    }
    /// Derive actual neutral approval before optional advisory evaluation on the same tuple.
    pub(crate) fn approved(
        &self,
        key: &str,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Option<ApprovedObject<'_>>> {
        fence(control, Stage::ValidateResource)?;
        let Some(resource) = self.resource(key) else {
            return Ok(None);
        };
        if let Some(proof) = self.approvals.borrow()[resource.index].as_ref() {
            return Ok(Some(ApprovedObject { resource, proof: Rc::clone(proof) }));
        }
        let object = resource.object();
        let Some(lifecycle_key) = object.policy.lifecycle_key.as_deref() else {
            return Ok(None);
        };
        let Some(lifecycle) = self.resource(lifecycle_key) else {
            return Ok(None);
        };
        if lifecycle.role() != Role::LifecycleRecord {
            return Ok(None);
        }
        let Some(value) = lifecycle.json() else {
            return Ok(None);
        };
        // Full native/record occurrences were charged before parsed capture installation.
        // Repeated derived copies still receive a complete occurrence precharge here.
        self.charge_relationships(tree_work(value)?.checked_mul(8).ok_or_else(limit_failure)?)?;
        if evaluate(&self.project, object, control)? != super::Availability::Available {
            return Ok(None);
        }
        let record = record::parse(lifecycle.bytes()).map_err(|_| policy_failure())?;
        let Some(current) = complete_closure(&self.project, lifecycle.object(), object, &record)
        else {
            return Ok(None);
        };
        let neutral =
            status::status_from_captured(&record, &current, None).map_err(|_| policy_failure())?;
        let evaluated = if contains(&self.decision.profile, "schedule_keys", key) {
            let date = self.decision.profile["as_of"].as_str().ok_or_else(policy_failure)?;
            Some(
                status::status_from_captured(&record, &current, Some(canonical_date(date)?))
                    .map_err(|_| policy_failure())?,
            )
        } else {
            None
        };
        fence(control, Stage::ValidateResource)?;
        let proof = Rc::new(LifecycleProof { record, current, neutral, evaluated });
        self.approvals.borrow_mut()[resource.index] = Some(Rc::clone(&proof));
        Ok(Some(ApprovedObject { resource, proof }))
    }
    /// Prove only an exact visible manifest/report lifecycle source is Approved/current.
    /// Generated natives keep their independent lifecycle bindings: listing a native in
    /// this record never approves it. The query must separately require `approved()` and
    /// visibility for every projected native, then perform complete domain comparison.
    pub(crate) fn domain_source_candidate(
        &self,
        key: &str,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Option<DomainSourceCandidate<'_>>> {
        self.verify_inputs(control)?;
        if !self.visible_keys().any(|visible| visible == key)
            || self.project.approval_basis != ApprovalBasis::RecordedCurrent
        {
            return Ok(None);
        }
        let Some(resource) = self.resource(key) else {
            return Ok(None);
        };
        if !matches!(resource.role(), Role::ApplicabilityManifest | Role::ApplicabilityReport)
            || resource.native_identity().is_some()
            || resource.object().policy.native_identity.is_some()
        {
            return Ok(None);
        }
        let Some(lifecycle_key) = resource.lifecycle_key() else {
            return Ok(None);
        };
        let Some(lifecycle) = self.resource(lifecycle_key) else {
            return Ok(None);
        };
        if lifecycle.role() != Role::LifecycleRecord {
            return Ok(None);
        }
        let Some(value) = lifecycle.json() else {
            return Ok(None);
        };
        // Charge the complete actual record before parsing, tuple strings, validation
        // registries and status copies. This is the same monotonic budget, never reset.
        self.charge_relationships(tree_work(value)?.checked_mul(8).ok_or_else(limit_failure)?)?;
        fence(control, Stage::PrepareDomain)?;
        let record = record::parse(lifecycle.bytes()).map_err(|_| policy_failure())?;
        let Some(current) = domain_current(self, resource, lifecycle, &record, control)? else {
            return Ok(None);
        };
        let neutral =
            status::status_from_captured(&record, &current, None).map_err(|_| policy_failure())?;
        if neutral.state != record::LifecycleState::Approved
            || neutral.approved_fingerprints.as_ref() != Some(&current.fingerprints)
            || !neutral.artifact_identity_changes.is_empty()
            || !neutral.blockers.is_empty()
            || neutral.derived_status != "approved"
        {
            return Ok(None);
        }
        self.verify_inputs(control)?;
        let proof = Rc::new(LifecycleProof { record, current, neutral, evaluated: None });
        Ok(Some(DomainSourceCandidate { resource, _proof: proof }))
    }
    /// Resolve exact recorded native-relative source spelling within the actual sealed roster.
    /// Unknown, mismatched-role/lifecycle or noncurrent bindings never become a path read.
    pub(crate) fn declared_source_for_native(
        &self,
        native_key: &str,
        source_file: &str,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Option<CapturedResource<'_>>> {
        let Some(native) = self.approved(native_key, control)? else {
            return Ok(None);
        };
        if native.resource.native_value().is_none() {
            return Ok(None);
        }
        let Some(path) = resolve_record_path(native.resource.path(), source_file) else {
            return Ok(None);
        };
        let resource = self.resources().find(|candidate| candidate.path() == path);
        Ok(resource.filter(|source| {
            matches!(source.role(), Role::PolicySource | Role::LifecycleSource)
                && source.lifecycle_key() == native.resource.lifecycle_key()
                && native
                    .resource
                    .lifecycle_key()
                    .and_then(|key| self.resource(key))
                    .and_then(|record| {
                        resolve_record_path(record.path(), &native.proof.record.policy.source.path)
                    })
                    .is_some_and(|recorded| recorded == source.path())
                && native.proof.current.fingerprints.source_sha256 == source.raw_sha256()
        }))
    }
    /// Create a borrowed view only for present pin-matched and intrinsically admitted bytes.
    fn admitted(&self, index: usize) -> Option<CapturedResource<'_>> {
        let object = &self.project.objects[index];
        if !object.pin_matches || !matches!(&object.original, CapturedLocal::Present(_, _)) {
            return None;
        }
        if object.policy.role.native_model().is_some() {
            if object.native.is_none() || object.native != object.policy.native_identity {
                return None;
            }
        } else if !matches!(object.policy.role, Role::PolicySource | Role::LifecycleSource)
            && self.values[index].is_none()
        {
            return None;
        }
        Some(CapturedResource { scope: self, index })
    }
}

/// Reconstruct the exact full domain-source tuple from original captured members only.
/// The selected source's lifecycle binding is required; generated members deliberately
/// retain independent lifecycle keys. No href read, inferred domain authority or copy
/// of a detached native Value is permitted by this private source-candidacy seam.
fn domain_current(
    scope: &CapturedQueryScope,
    selected: CapturedResource<'_>,
    lifecycle: CapturedResource<'_>,
    record: &LifecycleRecord,
    control: &mut dyn WorkControl,
) -> WorkResult<Option<CurrentArtifacts>> {
    if record.policy.source.oscal_type.is_some() || record.policy.source.root_uuid.is_some() {
        return Ok(None);
    }
    let Some(source_path) = resolve_record_path(lifecycle.path(), &record.policy.source.path)
    else {
        return Ok(None);
    };
    let Some(source) = exact_path_resource(scope, &source_path) else {
        return Ok(None);
    };
    if source.index != selected.index
        || source.key() != selected.key()
        || source.role() != selected.role()
        || source.lifecycle_key() != Some(lifecycle.key())
        || source.raw_sha256() != record.policy.source.sha256
    {
        return Ok(None);
    }
    // The full record was charged before parsing; charge every generated occurrence
    // again before retaining the tuple. Repeated/filtered calls do not gain credit.
    scope.charge_relationships(record.policy.generated_artifacts.len())?;
    let mut generated = Vec::with_capacity(record.policy.generated_artifacts.len());
    for expected in &record.policy.generated_artifacts {
        fence(control, Stage::PrepareDomain)?;
        let Some(path) = resolve_record_path(lifecycle.path(), &expected.path) else {
            return Ok(None);
        };
        let Some(actual) = exact_path_resource(scope, &path) else {
            return Ok(None);
        };
        let Some(identity) = actual.native_identity() else {
            return Ok(None);
        };
        if actual.native_value().is_none()
            || expected.oscal_type.as_deref() != Some(identity.model.as_str())
            || expected.root_uuid.as_deref() != Some(identity.root_id.as_str())
            || expected.sha256 != actual.raw_sha256()
        {
            return Ok(None);
        }
        generated.push(record::NamedHash {
            path: expected.path.clone(),
            sha256: actual.raw_sha256().to_owned(),
        });
    }
    generated.sort();
    Ok(Some(CurrentArtifacts {
        fingerprints: record::FingerprintSet {
            source_sha256: source.raw_sha256().to_owned(),
            generated_artifacts: generated,
        },
        identity_changes: Vec::new(),
    }))
}

/// Resolve exactly one pin/schema-admitted original by its full portable roster path.
/// No hash equality, generated href or caller role can substitute for that object.
fn exact_path_resource<'a>(
    scope: &'a CapturedQueryScope,
    path: &str,
) -> Option<CapturedResource<'a>> {
    let mut matches = scope.resources().filter(|resource| resource.path() == path);
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first)
}

/// Load the complete operator-selected handoff without writing project/control files.
/// Missing/half pairs perform no root/file IO. Ordinary invalid handoffs are fixed
/// unavailable; sticky interruption is always preserved rather than relabeled.
pub(crate) fn load_disclosure_decision(
    project_raw: &Path,
    decision_root_raw: Option<&Path>,
    decision_sha256: Option<&str>,
    profile_sha256: &str,
    control: &mut dyn WorkControl,
) -> WorkResult<DisclosureGate> {
    if let Some(reason) = control.interruption() {
        return Err(WorkError::Interrupted(reason));
    }
    let (Some(decision_root), Some(decision_pin)) = (decision_root_raw, decision_sha256) else {
        return Ok(DisclosureGate::Unavailable);
    };
    fence(control, Stage::ReadIndex)?;
    match load_complete(project_raw, decision_root, decision_pin, profile_sha256, control) {
        Ok(scope) => Ok(DisclosureGate::Validated(Box::new(scope))),
        Err(WorkError::Interrupted(reason)) => Err(WorkError::Interrupted(reason)),
        Err(WorkError::Failed(_)) => {
            fence(control, Stage::RetainPrepared)?;
            Ok(DisclosureGate::Unavailable)
        }
    }
}

/// Admit actual roots, complete config identities then resource identities before either read phase.
fn load_complete(
    project_raw: &Path,
    decision_raw: &Path,
    decision_pin: &str,
    profile_pin: &str,
    control: &mut dyn WorkControl,
) -> WorkResult<CapturedQueryScope> {
    if !valid_hash(decision_pin) || !valid_hash(profile_pin) {
        return Err(policy_failure());
    }
    let project = qualify(project_raw)?;
    let decision_root = qualify(decision_raw)?;
    reject_overlap(&project, &decision_root)?;
    let mut ledger = InputLedger::new();
    let config_opens = open_configs(&project, &decision_root, &mut ledger, control)?;
    let [manifest, profile_original, decision_original] = read_configs(config_opens, control)?;
    let decision_bytes = present_bytes(&decision_original)?;
    let profile_bytes = present_bytes(&profile_original)?;
    let manifest_bytes = present_bytes(&manifest)?;
    let discovery_pin = crate::hashing::sha256_hex(manifest_bytes);
    if crate::hashing::sha256_hex(decision_bytes) != decision_pin
        || crate::hashing::sha256_hex(profile_bytes) != profile_pin
    {
        return Err(policy_failure());
    }
    let mut relations = 0;
    let record_value =
        parse_config(decision_bytes, include_str!("decision.schema.json"), &mut relations)?;
    let profile_value =
        parse_config(profile_bytes, include_str!("profile.schema.json"), &mut relations)?;
    let discovery_value =
        parse_config(manifest_bytes, include_str!("discovery.schema.json"), &mut relations)?;
    validate_decision(
        &record_value,
        &profile_value,
        &discovery_value,
        &discovery_pin,
        profile_pin,
    )?;
    let declaration: ProjectDeclaration =
        serde_json::from_value(discovery_value).map_err(|_| policy_failure())?;
    validate_scope(&declaration, &profile_value, &discovery_pin)?;
    let opens = open_resources(&project, &declaration.resources, &mut ledger, control)?;
    let raw = manifest_bytes
        .len()
        .checked_add(profile_bytes.len())
        .and_then(|total| total.checked_add(decision_bytes.len()))
        .ok_or_else(limit_failure)?;
    let (objects, values, retained) =
        read_resources(opens, declaration.resources, raw, &mut relations, control)?;
    let visible_keys = strings(&profile_value, "visible_resource_keys")?;
    let captured = CapturedProject {
        root: project,
        manifest,
        objects,
        visible_keys,
        approval_basis: ApprovalBasis::RecordedCurrent,
        retained_raw_bytes: retained,
    };
    let decision = PreparedDisclosureDecision {
        root: decision_root,
        original: decision_original,
        profile_original,
        record: record_value,
        profile: profile_value,
        decision_sha256: decision_pin.to_owned(),
        profile_sha256: profile_pin.to_owned(),
        discovery_sha256: discovery_pin,
    };
    let approvals = RefCell::new(vec![None; values.len()]);
    let mut scope = CapturedQueryScope {
        project: captured,
        decision,
        values,
        approvals,
        relationships: Cell::new(relations),
        generation: String::new(),
    };
    scope.generation = generation(&scope);
    scope.verify_inputs(control)?;
    Ok(scope)
}

/// Shared complete count, native physical identity set and observed byte reservation.
struct InputLedger {
    /// All original slots, including three configs and typed missing resources.
    count: usize,
    /// Actual held file identities across both roots before reading any member.
    identities: BTreeSet<(u64, u64)>,
    /// Complete observed present sizes before content reads; reads also enforce actual bytes.
    bytes: u64,
}

impl InputLedger {
    /// Create one empty local ledger before any held config admission.
    fn new() -> Self {
        Self { count: 0, identities: BTreeSet::new(), bytes: 0 }
    }
    /// Admit one already opened original before retaining it or reading its contents.
    fn admit(&mut self, opened: &PreparedLocal, limit: usize) -> WorkResult<()> {
        let count = self.count.checked_add(1).ok_or_else(limit_failure)?;
        let size = opened.observed_size().map_err(|_| capture_failure())?;
        let bytes = self.bytes.checked_add(size).ok_or_else(limit_failure)?;
        if count > ORIGINAL_LIMIT || size > limit as u64 || bytes > RAW_LIMIT as u64 {
            return Err(limit_failure());
        }
        if let Some(identity) = opened.identity() {
            if !self.identities.insert(identity) {
                return Err(policy_failure());
            }
        }
        self.count = count;
        self.bytes = bytes;
        Ok(())
    }
}

/// Validate the complete raw root spelling before absolute/path composition or opening.
fn qualify(raw: &Path) -> WorkResult<Rc<RootGeneration>> {
    let spelling = raw.to_str().ok_or_else(policy_failure)?;
    if spelling.is_empty()
        || spelling.len() > 64 * 1024
        || raw.components().any(|part| matches!(part, Component::CurDir | Component::ParentDir))
        || !crate::linkage::has_normalized_path_spelling(raw)
    {
        return Err(policy_failure());
    }
    let absolute = std::path::absolute(raw).map_err(|_| capture_failure())?;
    fresh::qualify_root(&absolute).map(Rc::new).map_err(|_| capture_failure())
}

/// Reject actual nested/equal roots and conservative folded portable spelling overlap.
fn reject_overlap(project: &RootGeneration, decision: &RootGeneration) -> WorkResult<()> {
    let left = project.path().to_str().ok_or_else(policy_failure)?.replace('\\', "/");
    let right = decision.path().to_str().ok_or_else(policy_failure)?.replace('\\', "/");
    if fresh::roots_overlap(project, decision) || super::paths_overlap(&left, &right) {
        return Err(policy_failure());
    }
    Ok(())
}

/// Open and admit all three config identities before reading any config content.
fn open_configs(
    project: &Rc<RootGeneration>,
    decision: &Rc<RootGeneration>,
    ledger: &mut InputLedger,
    control: &mut dyn WorkControl,
) -> WorkResult<[PreparedLocal; 3]> {
    let open = |root: &Rc<RootGeneration>,
                path: &str,
                ledger: &mut InputLedger,
                control: &mut dyn WorkControl|
     -> WorkResult<PreparedLocal> {
        fence(control, Stage::ReadIndex)?;
        let held = fresh::prepare_local(Rc::clone(root), Path::new(path), false)
            .map_err(|_| capture_failure())?;
        ledger.admit(&held, CONFIG_LIMIT)?;
        Ok(held)
    };
    Ok([
        open(project, CONFIG_PATHS[0], ledger, control)?,
        open(project, CONFIG_PATHS[1], ledger, control)?,
        open(decision, CONFIG_PATHS[2], ledger, control)?,
    ])
}

/// Consume held configs only after the complete before-read alias phase succeeded.
fn read_configs(
    opens: [PreparedLocal; 3],
    control: &mut dyn WorkControl,
) -> WorkResult<[CapturedLocal; 3]> {
    let [manifest, profile, decision] = opens;
    let read = |held: PreparedLocal, control: &mut dyn WorkControl| -> WorkResult<CapturedLocal> {
        fence(control, Stage::ReadIndex)?;
        held.read(CONFIG_LIMIT as u64).map_err(|_| capture_failure())
    };
    Ok([read(manifest, control)?, read(profile, control)?, read(decision, control)?])
}

/// Preflight the entire declared resource count before opening any member, then hold all aliases.
fn open_resources(
    root: &Rc<RootGeneration>,
    resources: &[ResourcePolicy],
    ledger: &mut InputLedger,
    control: &mut dyn WorkControl,
) -> WorkResult<Vec<PreparedLocal>> {
    if ledger.count.checked_add(resources.len()).is_none_or(|count| count > ORIGINAL_LIMIT) {
        return Err(limit_failure());
    }
    let mut opens = Vec::new();
    for resource in resources {
        fence(control, Stage::CaptureResource)?;
        let held = fresh::prepare_local(Rc::clone(root), Path::new(&resource.path), true)
            .map_err(|_| capture_failure())?;
        ledger.admit(&held, resource_limit(resource.role))?;
        opens.push(held);
    }
    Ok(opens)
}

/// Read each admitted held original under the remaining single raw budget, then admit typed growth.
fn read_resources(
    opens: Vec<PreparedLocal>,
    policies: Vec<ResourcePolicy>,
    mut raw: usize,
    relations: &mut usize,
    control: &mut dyn WorkControl,
) -> WorkResult<(Vec<CapturedObject>, Vec<Option<Value>>, usize)> {
    if opens.len() != policies.len() {
        return Err(policy_failure());
    }
    let mut objects = Vec::new();
    let mut values = Vec::new();
    for (held, policy) in opens.into_iter().zip(policies) {
        fence(control, Stage::CaptureResource)?;
        let remaining = RAW_LIMIT.checked_sub(raw).ok_or_else(limit_failure)?;
        let original = held
            .read(remaining.min(resource_limit(policy.role)) as u64)
            .map_err(|_| capture_failure())?;
        let (digest, value, native) = match &original {
            CapturedLocal::Present(bytes, _) => {
                raw = raw
                    .checked_add(bytes.len())
                    .filter(|total| *total <= RAW_LIMIT)
                    .ok_or_else(limit_failure)?;
                let digest = crate::hashing::sha256_hex(bytes);
                let value = if matches!(policy.role, Role::PolicySource | Role::LifecycleSource) {
                    None
                } else {
                    match strict_value(bytes, resource_limit(policy.role)) {
                        Ok(value) => {
                            let repetitions =
                                if policy.role.native_model().is_some() { 2 } else { 1 };
                            charge(
                                relations,
                                tree_work(&value)?
                                    .checked_mul(repetitions)
                                    .ok_or_else(limit_failure)?,
                            )?;
                            Some(value)
                        }
                        Err(_) => None,
                    }
                };
                let native = if policy.role.native_model().is_some() && value.is_some() {
                    native_facts(bytes, policy.role)
                } else {
                    None
                };
                (Some(digest), value, native)
            }
            CapturedLocal::Absent(_) => (None, None, None),
        };
        let pin_matches = digest.as_deref() == Some(policy.expected_sha256.as_str());
        objects.push(CapturedObject { policy, original, digest, pin_matches, native });
        values.push(value);
        fence(control, Stage::CaptureResource)?;
    }
    Ok((objects, values, raw))
}

/// Bound the complete strict original before schema or typed deserialization work.
fn parse_config(bytes: &[u8], schema: &str, relations: &mut usize) -> WorkResult<Value> {
    let value = strict_value(bytes, CONFIG_LIMIT)?;
    charge(relations, tree_work(&value)?)?;
    let schema = strict_value(schema.as_bytes(), CONFIG_LIMIT)?;
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .map_err(|_| policy_failure())?;
    if !validator.is_valid(&value) {
        return Err(policy_failure());
    }
    Ok(value)
}

/// Validate complete scoped decision and all six distinct approved provenance-bearing rows.
fn validate_decision(
    record: &Value,
    profile: &Value,
    discovery: &Value,
    discovery_pin: &str,
    profile_pin: &str,
) -> WorkResult<()> {
    if record["proposal_index_sha256"] != PROPOSAL
        || record["protocol_revision"] != PROTOCOL
        || record["profile_sha256"] != profile_pin
        || record["discovery_sha256"] != discovery_pin
        || record["visibility_profile"] != *profile
        || record["project_key"] != discovery["project_key"]
        || profile["project_key"] != discovery["project_key"]
        || profile["discovery_sha256"] != discovery_pin
    {
        return Err(policy_failure());
    }
    let rows = record["policy_decisions"].as_array().ok_or_else(policy_failure)?;
    let mut pairs = BTreeSet::new();
    for row in rows {
        let subject = row["subject"].as_str().ok_or_else(policy_failure)?;
        let role = row["role"].as_str().ok_or_else(policy_failure)?;
        if row["disposition"] != "approved"
            || !pairs.insert((subject, role))
            || row["proposal_index_sha256"] != PROPOSAL
            || row["profile_sha256"] != profile_pin
            || row["discovery_sha256"] != discovery_pin
        {
            return Err(policy_failure());
        }
        for field in ["declared_owner_key", "source_record_key", "source_record_sha256"] {
            if row[field].as_str().is_none() {
                return Err(policy_failure());
            }
        }
        canonical_date(row["recorded_on"].as_str().ok_or_else(policy_failure)?)?;
    }
    let expected = BTreeSet::from([
        ("D067-P1", "product"),
        ("D067-P2", "product"),
        ("D067-S1", "security"),
        ("D067-E1", "engineering"),
        ("D067-S2", "product"),
        ("D067-S2", "security"),
    ]);
    if pairs != expected {
        return Err(policy_failure());
    }
    if let Some(date) = record["evaluation_plan"]["recorded_on"].as_str() {
        canonical_date(date)?;
    }
    Ok(())
}

/// Apply full resource/visibility and source/schedule pairing before resource opens.
fn validate_scope(
    declaration: &ProjectDeclaration,
    profile: &Value,
    discovery_pin: &str,
) -> WorkResult<()> {
    let visible = strings(profile, "visible_resource_keys")?;
    validate_policy(&super::CallerPolicy {
        manifest_sha256: discovery_pin.to_owned(),
        declaration: declaration.clone(),
        visible_keys: visible.clone(),
        approval_basis: ApprovalBasis::RecordedCurrent,
    })?;
    let known: BTreeSet<_> =
        declaration.resources.iter().map(|resource| resource.key.as_str()).collect();
    for name in [
        "noncurrent_metadata_keys",
        "source_text_keys",
        "schedule_keys",
        "evidence_metadata_keys",
        "static_report_keys",
    ] {
        for key in profile[name].as_array().ok_or_else(policy_failure)? {
            let key = key.as_str().ok_or_else(policy_failure)?;
            if !known.contains(key) {
                return Err(policy_failure());
            }
            if matches!(name, "source_text_keys" | "schedule_keys")
                && !visible.iter().any(|visible| visible == key)
            {
                return Err(policy_failure());
            }
        }
    }
    if let Some(index) = profile["search_index_key"].as_str() {
        if !known.contains(index) {
            return Err(policy_failure());
        }
    }
    for key in profile["source_text_keys"].as_array().ok_or_else(policy_failure)? {
        let key = key.as_str().ok_or_else(policy_failure)?;
        let source = declaration
            .resources
            .iter()
            .find(|resource| resource.key == key)
            .ok_or_else(policy_failure)?;
        if !matches!(source.role, Role::PolicySource | Role::LifecycleSource) {
            return Err(policy_failure());
        }
    }
    for key in profile["schedule_keys"].as_array().ok_or_else(policy_failure)? {
        let key = key.as_str().ok_or_else(policy_failure)?;
        let resource = declaration
            .resources
            .iter()
            .find(|resource| resource.key == key)
            .ok_or_else(policy_failure)?;
        if !resource.role.status_supported() || resource.lifecycle_key.is_none() {
            return Err(policy_failure());
        }
    }
    if profile["source_text_mode"] == "omit"
        && !profile["source_text_keys"].as_array().ok_or_else(policy_failure)?.is_empty()
    {
        return Err(policy_failure());
    }
    if profile["schedule_mode"] == "explicit-as-of" {
        canonical_date(profile["as_of"].as_str().ok_or_else(policy_failure)?)?;
    }
    for resource in &declaration.resources {
        if super::paths_overlap(&resource.path, CONFIG_PATHS[0])
            || super::paths_overlap(&resource.path, CONFIG_PATHS[1])
        {
            return Err(policy_failure());
        }
    }
    Ok(())
}

/// Consume the maintained canonical ASCII ten-byte calendar rule without a clock.
fn canonical_date(raw: &str) -> WorkResult<NaiveDate> {
    let bytes = raw.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return Err(policy_failure());
    }
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|_| policy_failure())
}

/// Count every complete Value node/property before downstream typed registry growth.
/// Recursion uses the already admitted depth64, not an unbounded walker stack.
fn tree_work(value: &Value) -> WorkResult<usize> {
    let mut count = 1_usize;
    match value {
        Value::Array(values) => {
            for value in values {
                count = count.checked_add(tree_work(value)?).ok_or_else(limit_failure)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                let child = tree_work(value)?;
                count = count
                    .checked_add(1)
                    .and_then(|count| count.checked_add(child))
                    .ok_or_else(limit_failure)?;
            }
        }
        _ => {}
    }
    if count > RELATION_LIMIT {
        return Err(limit_failure());
    }
    Ok(count)
}

/// Check a complete monotonic count before installing an additional parsed structure.
fn charge(count: &mut usize, amount: usize) -> WorkResult<()> {
    let next = count
        .checked_add(amount)
        .filter(|next| *next <= RELATION_LIMIT)
        .ok_or_else(limit_failure)?;
    *count = next;
    Ok(())
}

/// Borrow only actual present config bytes; typed absence cannot become an empty config.
fn present_bytes(original: &CapturedLocal) -> WorkResult<&[u8]> {
    match original {
        CapturedLocal::Present(bytes, _) => Ok(bytes),
        CapturedLocal::Absent(_) => Err(policy_failure()),
    }
}

/// Choose the existing record or ordinary resource cap without another input pool.
fn resource_limit(role: Role) -> usize {
    if role == Role::LifecycleRecord {
        usize::try_from(record::MAX_RECORD_BYTES).unwrap_or(0)
    } else {
        super::MAX_RESOURCE_BYTES
    }
}

/// Read a schema-admitted bounded string list without reordering or normalizing values.
fn strings(value: &Value, name: &str) -> WorkResult<Vec<String>> {
    value[name]
        .as_array()
        .ok_or_else(policy_failure)?
        .iter()
        .map(|value| value.as_str().map(str::to_owned).ok_or_else(policy_failure))
        .collect()
}

/// Test exact membership of already schema-admitted original profile arrays.
fn contains(profile: &Value, name: &str, key: &str) -> bool {
    profile[name]
        .as_array()
        .is_some_and(|keys| keys.iter().any(|value| value.as_str() == Some(key)))
}

/// Bind complete actual original generations/config pins without retaining another source buffer.
fn generation(scope: &CapturedQueryScope) -> String {
    let mut digest = Sha256::new();
    for value in [
        &scope.decision.decision_sha256,
        &scope.decision.profile_sha256,
        &scope.decision.discovery_sha256,
    ] {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    for original in
        [&scope.project.manifest, &scope.decision.profile_original, &scope.decision.original]
            .into_iter()
            .chain(scope.project.objects.iter().map(|object| &object.original))
    {
        digest.update(fresh::original_generation_digest(original).as_bytes());
        match original {
            CapturedLocal::Present(bytes, _) => {
                digest.update([1]);
                digest.update(crate::hashing::sha256_hex(bytes).as_bytes());
            }
            CapturedLocal::Absent(_) => {
                digest.update([0]);
            }
        }
    }
    crate::hashing::lower_hex(&digest.finalize())
}

/// File-backed synthetic controls remain separate from production and owner acceptance.
#[cfg(test)]
#[path = "disclosure_tests.rs"]
mod tests;

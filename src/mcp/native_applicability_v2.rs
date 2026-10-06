//! Bounded genuine Catalog applicability over the one complete held /2 native owner.
//!
//! Source and dependency approval are separate maintained lifecycle predicates. Native
//! evaluation and complete stored-report equality consume actual captured bytes only.
//! Profile, nonempty Mapping and linkage domain producers remain whole-gate unavailable;
//! this private preparation adds no server DTO, query, index builder or publication port.

use crate::applicability::captured::{
    self, BorrowedCatalogInput, NativeInput, PreparedApplicability,
};
use crate::applicability::manifest;
use crate::mapping::manifest::{ResourceType, SubjectType};
use crate::workspace::preparation::{Stage, WorkControl, WorkResult};

use super::capture_v2::{Admission, AdmittedValue, Observation};
use super::declarations_v2::DeclarationAdmission;
use super::native_domain_v2::{self, DomainSourceV2};
use super::native_sources_v2::NativeSourcesV2;
use super::native_work_v2::{self as work};
use super::{Role, policy_failure, resolve_record_path, valid_hash};

/// Private complete native domain result; borrowed owner and retained native data cannot escape.
pub(super) struct PreparedDomainV2<'scope, 'control, C: WorkControl + ?Sized> {
    /// Actual manifest source with its own complete intrinsic current lifecycle.
    manifest: DomainSourceV2<'scope, 'control, C>,
    /// Actual selected stored report source, independently qualified when requested.
    _report: Option<DomainSourceV2<'scope, 'control, C>>,
    /// Exact same-owner framework whose real inventory supplied every complete native row.
    framework: usize,
    /// Complete maintained facts, including every control, decision and private native field.
    prepared: AdmittedValue<Option<PreparedApplicability<'scope>>>,
}

/// Prepare only a complete supported App source; every ordinary outcome reaches the
/// original caller's final fence before it can become whole-gate unavailability.
pub(super) fn prepare_selected<'scope, 'control, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'control, C>,
    key: &str,
) -> WorkResult<Option<PreparedDomainV2<'scope, 'control, C>>> {
    let mut admission = sources.admission();
    let result = work::run(&mut admission, 1, Stage::PrepareDomain, |_| select(sources, key))?;
    sources.verify_inputs()?;
    admission.phase(Ok(result), Stage::RetainPrepared)
}

/// Select the actual manifest or exactly one full native current report match.
#[expect(
    clippy::used_underscore_binding,
    reason = "The selected actual report view remains in its established ownership field through the complete native result; preserve that retained field and its original lifetime contract."
)]
fn select<'scope, 'control, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'control, C>,
    key: &str,
) -> WorkResult<Option<PreparedDomainV2<'scope, 'control, C>>> {
    let Some(selected) = native_domain_v2::candidate(sources, key)? else {
        return Ok(None);
    };
    let mut admission = sources.admission();
    if !supported_generated(sources, &selected, &mut admission)? {
        return Ok(None);
    }
    match sources.policies()[selected.index()].role {
        Role::ApplicabilityManifest => manifest_plan(sources, selected),
        Role::ApplicabilityReport => {
            let Some(hint) = sources
                .facts(selected.index())
                .and_then(|facts| facts.value.as_ref())
                .and_then(|value| value.get("manifest_sha256"))
                .and_then(serde_json::Value::as_str)
            else {
                return Ok(None);
            };
            if !work::run(
                &mut admission,
                1 + work::byte_work(hint.len()),
                Stage::ValidateResource,
                |_| Ok(valid_hash(hint)),
            )? {
                return Ok(None);
            }
            let mut matched = None;
            let mut match_count = 0;
            for candidate_key in &sources.profile().visible_resource_keys {
                work::fence(&mut admission, Stage::PrepareDomain)?;
                let Some(index) = sources.index(candidate_key)? else {
                    return Ok(None);
                };
                if sources.policies()[index].role != Role::ApplicabilityManifest {
                    continue;
                }
                let Observation::Present { raw_sha256, .. } = sources.observation(index)? else {
                    continue;
                };
                let width = work::add(hint.len(), raw_sha256.len(), &mut admission)?;
                if !work::run(
                    &mut admission,
                    1 + work::byte_work(width),
                    Stage::ValidateResource,
                    |_| Ok(hint == raw_sha256),
                )? {
                    continue;
                }
                let Some(candidate) = native_domain_v2::candidate(sources, candidate_key)? else {
                    continue;
                };
                if !supported_generated(sources, &candidate, &mut admission)? {
                    continue;
                }
                let Some(plan) = manifest_plan(sources, candidate)? else {
                    continue;
                };
                if !report_matches(sources, &plan, &selected, &mut admission)? {
                    continue;
                }
                match_count = work::add(match_count, 1, &mut admission)?;
                // Retain one real plan, but still evaluate every later actual matching original.
                // Distinct equal-byte source occurrences are never collapsed by the hash hint.
                if matched.is_none() {
                    matched = Some(plan);
                }
            }
            if match_count != 1 {
                return Ok(None);
            }
            if let Some(plan) = matched.as_mut() {
                plan._report = Some(selected);
            }
            Ok(matched)
        }
        _ => Ok(None),
    }
}

/// The first domain bridge cannot activate any missing Mapping/Profile/generated family.
/// Independent native dependencies are approved by their own complete real closures.
fn supported_generated<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    source: &DomainSourceV2<'_, '_, C>,
    admission: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    for index in source.generated() {
        admission.charge(1)?;
        let policy = &sources.policies()[*index];
        if !matches!(policy.role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact)
            || sources.approved(&policy.key)?.is_none()
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Parse the actual manifest before discovering any native-relative route; this plain typed
/// discovery does not issue approval. The maintained captured evaluator reparses all semantics.
fn manifest_plan<'scope, 'control, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'control, C>,
    source: DomainSourceV2<'scope, 'control, C>,
) -> WorkResult<Option<PreparedDomainV2<'scope, 'control, C>>> {
    let mut admission = sources.admission();
    let raw = source.bytes()?;
    let (raw_nodes, raw_logical) = work::raw_bound(raw, 10 * 1024 * 1024, &mut admission)?;
    let facts = sources.facts(source.index()).ok_or_else(policy_failure)?;
    let value = facts.value.as_ref().ok_or_else(policy_failure)?;
    let decisions =
        value.get("decisions").and_then(serde_json::Value::as_array).map_or(0, Vec::len);
    let repeated = registry_work(value, &mut admission)?;
    let parse_work = work::add(
        work::multiply(facts.operand.work, 6, &mut admission)?,
        repeated,
        &mut admission,
    )?;
    let parse_work = work::add(
        parse_work,
        work::multiply(work::byte_work(raw.len()), 2, &mut admission)?,
        &mut admission,
    )?;
    let parsed = admission.retain(raw_logical, |admission| {
        work::run(admission, parse_work, Stage::PrepareDomain, |_| Ok(manifest::parse(raw).ok()))
    })?;
    let Some(manifest) = parsed.value().as_ref() else {
        return Ok(None);
    };
    if manifest.framework.resource_type != ResourceType::Catalog
        || !manifest.mapping_collections.is_empty()
    {
        return Ok(None);
    }
    let Some(relative) = manifest.framework.artifact.to_str() else {
        return Ok(None);
    };
    let Some(framework) = framework_at(sources, source.index(), relative, &mut admission)? else {
        return Ok(None);
    };
    // Membership comes only from the real complete native source tuple reconstructed above.
    // Independent native approval alone cannot replace the selected App source's generated binding.
    if !work::run(&mut admission, source.generated().len() + 1, Stage::ValidateResource, |_| {
        Ok(source.generated().contains(&framework))
    })? {
        return Ok(None);
    }
    // All typed discovery and its route scratch drop before native evaluation. The actual raw
    // owner and independent approved framework view remain the only bytes/identity source.
    drop(parsed);
    let roster = native_roster(sources, &mut admission)?;
    let Some(roster) = roster else {
        return Ok(None);
    };
    let measured = construction(
        sources,
        ConstructionInputs {
            framework,
            raw_nodes,
            raw_logical,
            decisions,
            manifest_bytes: raw.len(),
            manifest_work: facts.operand.work,
            registry: repeated,
            manifest_value: value,
            roster: roster.value(),
        },
        &mut admission,
    )?;
    let mut callback = sources.admission();
    let mut native_control = sources.admission();
    let prepared = admission.retain(measured.logical, |admission| {
        work::run(admission, measured.work, Stage::PrepareDomain, |_| {
            // The genuine callback/control aliases borrow the original controller only at their
            // own checkpoints. No RefMut is retained while the maintained helper invokes them.
            let result = captured::prepare_borrowed_catalog(
                raw,
                &sources.policies()[framework].key,
                roster.value(),
                &mut |amount| callback.charge(amount),
                &mut native_control,
            );
            // Native ordinary invalidity is reduced only after the unconditional same-owner
            // post-fence has reasserted the first actual admission/control stop.
            let result = native_control.phase(result, Stage::PrepareDomain);
            if let Ok(prepared) = result {
                Ok(Some(prepared))
            } else {
                work::fence(&mut native_control, Stage::PrepareDomain)?;
                Ok(None)
            }
        })
    })?;
    let Some(actual) = prepared.value().as_ref() else {
        return Ok(None);
    };
    let Observation::Present { raw_sha256, .. } = sources.observation(source.index())? else {
        return Ok(None);
    };
    let width = work::add(actual.manifest_sha256().len(), raw_sha256.len(), &mut admission)?;
    if !work::run(&mut admission, 1 + work::byte_work(width), Stage::ValidateResource, |_| {
        Ok(actual.manifest_sha256() == raw_sha256)
    })? {
        return Ok(None);
    }
    Ok(Some(PreparedDomainV2 { manifest: source, _report: None, framework, prepared }))
}

/// Resolve the validated actual manifest path solely to one visible independently Approved
/// same-owner Catalog occurrence. No href, hash-only replacement, import or file reopen occurs.
fn framework_at<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    source: usize,
    relative: &str,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Option<usize>> {
    let mut width = work::add(sources.policies()[source].path.len(), relative.len(), admission)?;
    for policy in sources.policies() {
        admission.charge(1)?;
        width = work::add(width, policy.path.len(), admission)?;
    }
    let amount = work::add(
        sources.policies().len(),
        work::byte_work(work::multiply(width, 4, admission)?),
        admission,
    )?;
    let logical = work::add(work::multiply(width, 2, admission)?, 512, admission)?;
    let route = admission.retain(logical, |admission| {
        work::run(admission, amount, Stage::PrepareDomain, |_| {
            Ok(resolve_record_path(&sources.policies()[source].path, relative))
        })
    })?;
    let Some(route) = route.value().as_deref() else {
        return Ok(None);
    };
    let mut selected = None;
    for key in &sources.profile().visible_resource_keys {
        admission.charge(1)?;
        let Some(index) = sources.index(key)? else {
            return Ok(None);
        };
        let policy = &sources.policies()[index];
        let width = work::add(route.len(), policy.path.len(), admission)?;
        if !work::run(admission, 1 + work::byte_work(width), Stage::PrepareDomain, |_| {
            Ok(route == policy.path)
        })? {
            continue;
        }
        if selected.is_some()
            || policy.role != Role::OscalCatalogArtifact
            || sources.approved(key)?.is_none()
        {
            return Ok(None);
        }
        selected = Some(index);
    }
    Ok(selected)
}

/// Supply the complete visible native Catalog roster, never a selected dependency subset.
fn native_roster<'scope, 'control, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'control, C>,
    admission: &mut Admission<'control, C>,
) -> WorkResult<Option<AdmittedValue<Vec<BorrowedCatalogInput<'scope>>>>> {
    let slots = work::multiply(
        sources.profile().visible_resource_keys.len(),
        std::mem::size_of::<BorrowedCatalogInput<'_>>(),
        admission,
    )?;
    let slots = work::add(slots, std::mem::size_of::<Vec<BorrowedCatalogInput<'_>>>(), admission)?;
    let result = admission.retain(slots, |admission| {
        let mut roster = Vec::with_capacity(sources.profile().visible_resource_keys.len());
        for key in &sources.profile().visible_resource_keys {
            admission.charge(1)?;
            let Some(index) = sources.index(key)? else {
                return Ok(None);
            };
            let policy = &sources.policies()[index];
            if policy.role == Role::OscalProfileArtifact {
                return Ok(None);
            }
            if policy.role != Role::OscalCatalogArtifact {
                continue;
            }
            let Some(approved) = sources.approved(key)? else {
                return Ok(None);
            };
            let Some(identity) = approved.resource().native_identity() else {
                return Ok(None);
            };
            let Some(loaded) = sources.inventory(index) else {
                return Ok(None);
            };
            let Observation::Present { bytes, raw_sha256 } = sources.observation(index)? else {
                return Ok(None);
            };
            let evidence = &loaded.evidence;
            let mut width = 0;
            for (expected, actual) in [
                (raw_sha256, evidence.raw_sha256.as_str()),
                (identity.model.as_str(), evidence.resource_type.as_str()),
                (identity.root_id.as_str(), evidence.root_uuid.as_str()),
                (identity.document_version.as_str(), evidence.document_version.as_str()),
                (identity.oscal_version.as_str(), evidence.oscal_version.as_str()),
            ] {
                admission.charge(1)?;
                width = work::add(
                    width,
                    work::add(expected.len(), actual.len(), admission)?,
                    admission,
                )?;
            }
            let exact =
                work::run(admission, 6 + work::byte_work(width), Stage::ValidateResource, |_| {
                    Ok(evidence.resource_type == ResourceType::Catalog
                        && evidence.resolved_catalog_sha256.is_none()
                        && identity.model == "catalog"
                        && evidence.raw_sha256 == raw_sha256
                        && evidence.root_uuid == identity.root_id
                        && evidence.document_version == identity.document_version
                        && evidence.oscal_version == identity.oscal_version)
                })?;
            if !exact {
                return Ok(None);
            }
            roster.push(BorrowedCatalogInput {
                input: NativeInput {
                    key: policy.key.as_str(),
                    resource_type: ResourceType::Catalog,
                    bytes,
                    resolved_catalog: None,
                },
                loaded,
            });
        }
        Ok(Some(roster))
    })?;
    // An admitted Option cannot be moved out of its ticket. A second complete borrowed roster
    // is admitted rather than detaching data; the first attempt remains charged and retained.
    let Some(rows) = result.value().as_ref() else {
        return Ok(None);
    };
    let copied = admission.retain(slots, |admission| {
        admission.charge(rows.len())?;
        Ok(rows.clone())
    })?;
    Ok(Some(copied))
}

/// Account every actual manifest registry key/reference before typed contract validation.
/// The bounded pair traversal covers repeated `BTree` probes; private prose is scanned only by
/// the full operand passes, never multiplied by an unrelated complete document per row.
fn registry_work<C: WorkControl + ?Sized>(
    value: &serde_json::Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut rows = 1;
    let mut width = 0;
    for (field, keys) in
        [("reviewers", &["key"][..]), ("decisions", &["control_id", "reviewer_key"][..])]
    {
        for row in value.get(field).and_then(serde_json::Value::as_array).into_iter().flatten() {
            admission.charge(1)?;
            rows = work::add(rows, 1, admission)?;
            for key in keys {
                if let Some(text) = row.get(*key).and_then(serde_json::Value::as_str) {
                    width = work::add(width, text.len(), admission)?;
                }
            }
        }
    }
    for path in
        value.get("mapping_collections").and_then(serde_json::Value::as_array).into_iter().flatten()
    {
        admission.charge(1)?;
        rows = work::add(rows, 1, admission)?;
        width = work::add(width, path.as_str().map_or(0, str::len), admission)?;
    }
    let pairs = work::multiply(rows, rows, admission)?;
    let strings = work::multiply(rows, work::byte_work(width), admission)?;
    work::multiply(work::add(pairs, strings, admission)?, 4, admission)
}

/// Account full exact decision/inventory join operands before references, classification and
/// stored-report lookups. All rows/IDs come from genuine maintained inventory/actual manifest.
fn decision_work<C: WorkControl + ?Sized>(
    inventory: &crate::mapping::inventory::LoadedResource,
    value: &serde_json::Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let controls = inventory.inventory.count(SubjectType::Control);
    let subjects =
        work::add(controls, inventory.inventory.count(SubjectType::Statement), admission)?;
    let decisions =
        value.get("decisions").and_then(serde_json::Value::as_array).map_or(0, Vec::len);
    let mut width = 0;
    for kind in [SubjectType::Control, SubjectType::Statement] {
        for id in inventory.inventory.ids_of_type_refs(kind) {
            admission.charge(1)?;
            width = work::add(width, id.len(), admission)?;
        }
    }
    for decision in
        value.get("decisions").and_then(serde_json::Value::as_array).into_iter().flatten()
    {
        admission.charge(1)?;
        let id = decision
            .get("control_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(policy_failure)?;
        width = work::add(width, id.len(), admission)?;
    }
    let rows = work::add(subjects, work::add(decisions, 1, admission)?, admission)?;
    let lookups = work::multiply(work::add(controls, decisions, admission)?, rows, admission)?;
    let strings = work::multiply(rows, work::byte_work(width), admission)?;
    work::multiply(work::add(lookups, strings, admission)?, 4, admission)
}

/// Exact plain construction descriptor, not a currentness or capture capability.
struct Construction {
    /// Simultaneous manifest strict/typed scratch, borrowed slots and full evidence overlays.
    logical: usize,
    /// All executed manifest, raw-evidence, snapshot, decision and control operations.
    work: usize,
}

/// Borrowed complete operands for the unchanged construction arithmetic; no allocation or authority.
#[derive(Clone, Copy)]
struct ConstructionInputs<'a, 'native> {
    framework: usize,
    raw_nodes: usize,
    raw_logical: usize,
    decisions: usize,
    manifest_bytes: usize,
    manifest_work: usize,
    registry: usize,
    manifest_value: &'a serde_json::Value,
    roster: &'a [BorrowedCatalogInput<'native>],
}

/// Admit all helper construction multiplicities for the bounded empty-Mapping domain.
fn construction<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    inputs: ConstructionInputs<'_, '_>,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Construction> {
    let ConstructionInputs {
        framework,
        raw_nodes,
        raw_logical,
        decisions,
        manifest_bytes,
        manifest_work,
        registry,
        manifest_value,
        roster,
    } = inputs;
    let mut logical = work::multiply(raw_logical, 2, admission)?;
    let mut amount = work::multiply(manifest_work, 6, admission)?;
    amount = work::add(
        amount,
        work::multiply(work::byte_work(manifest_bytes), 3, admission)?,
        admission,
    )?;
    let mut key_width = 0;
    for row in roster {
        admission.charge(1)?;
        let native = &row.input;
        let evidence = &row.loaded.evidence;
        key_width = work::add(key_width, native.key.len(), admission)?;
        let mut metadata = 0;
        for text in [
            &evidence.href,
            &evidence.raw_sha256,
            &evidence.root_uuid,
            &evidence.document_version,
            &evidence.oscal_version,
        ] {
            admission.charge(1)?;
            metadata = work::add(metadata, text.len(), admission)?;
        }
        // Native header strict/schema and framework parse/schema/canonical construction do
        // not execute in the borrowed port. The original inventory's ticket remains alive.
        // The port still hashes each complete raw original, copies actual header evidence,
        // and copies selected manifest-facing evidence with its exact declared href overlay.
        let header = work::add(
            work::multiply(metadata, 4, admission)?,
            work::add(work::multiply(native.key.len(), 4, admission)?, 2048, admission)?,
            admission,
        )?;
        logical = work::add(logical, header, admission)?;
        let native_work = work::add(24, work::byte_work(native.bytes.len()), admission)?;
        amount = work::add(
            amount,
            work::add(
                native_work,
                work::byte_work(work::multiply(metadata, 4, admission)?),
                admission,
            )?,
            admission,
        )?;
    }
    let inventory = sources.inventory(framework).ok_or_else(policy_failure)?;
    let controls = inventory.inventory.count(SubjectType::Control);
    let subjects =
        work::add(controls, inventory.inventory.count(SubjectType::Statement), admission)?;
    let mut ids = 0;
    for kind in [SubjectType::Control, SubjectType::Statement] {
        for id in inventory.inventory.ids_of_type_refs(kind) {
            admission.charge(1)?;
            ids = work::add(ids, id.len(), admission)?;
        }
    }
    // Full snapshot validation can clone both ID sets and hash every fingerprint. Decision
    // registries clone complete IDs; repeated tree lookups are bounded by complete row pairs.
    let rows = work::add(raw_nodes, work::add(subjects, decisions, admission)?, admission)?;
    let rows = work::add(rows, work::add(roster.len(), 1, admission)?, admission)?;
    logical = work::add(
        logical,
        work::add(
            work::multiply(ids, 3, admission)?,
            work::multiply(rows, 256, admission)?,
            admission,
        )?,
        admission,
    )?;
    let repeated = decision_work(inventory, manifest_value, admission)?;
    let repeated = work::add(repeated, registry, admission)?;
    let pairs = work::multiply(roster.len(), roster.len(), admission)?;
    let native_lookup = work::add(
        pairs,
        work::multiply(roster.len(), work::byte_work(key_width), admission)?,
        admission,
    )?;
    amount = work::add(amount, work::add(repeated, native_lookup, admission)?, admission)?;
    // The complete snapshot clones both sorted ID sets, frames every subject fingerprint,
    // and compares every exact native metadata/ID/digest field. Admit full Statement rows
    // as well as Controls, including conservative set construction and repeated ID probes.
    let snapshot = snapshot_work(subjects, ids, admission)?;
    amount = work::add(amount, snapshot, admission)?;
    Ok(Construction { logical, work: amount })
}

/// Account the unchanged complete native snapshot sets, comparisons and framed fingerprint work.
fn snapshot_work<C: WorkControl + ?Sized>(
    subjects: usize,
    ids: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let snapshot_rows = work::multiply(subjects, subjects, admission)?;
    let snapshot_strings = work::multiply(
        work::multiply(work::add(subjects, 2, admission)?, work::byte_work(ids), admission)?,
        2,
        admission,
    )?;
    let snapshot_digest =
        work::byte_work(work::add(ids, work::multiply(subjects, 128, admission)?, admission)?);
    let snapshot = work::add(
        work::multiply(subjects, 6, admission)?,
        work::add(
            work::multiply(snapshot_rows, 2, admission)?,
            work::add(snapshot_strings, snapshot_digest, admission)?,
            admission,
        )?,
        admission,
    )?;
    Ok(snapshot)
}

/// Compare every private stored report field against complete current native facts. The raw
/// digest is only a candidate hint; full equality plus actual independently Approved sources wins.
fn report_matches<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    plan: &PreparedDomainV2<'_, '_, C>,
    report: &DomainSourceV2<'_, '_, C>,
    admission: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    if !work::run(admission, report.generated().len() + 1, Stage::ValidateResource, |_| {
        Ok(report.generated().contains(&plan.framework))
    })? {
        return Ok(false);
    }
    let raw = report.bytes()?;
    let (nodes, logical) = work::raw_bound(raw, 10 * 1024 * 1024, admission)?;
    let facts = sources.facts(report.index()).ok_or_else(policy_failure)?;
    let prepared = plan.facts();
    let count = prepared.counts().total;
    let manifest_facts = sources.facts(plan.manifest.index()).ok_or_else(policy_failure)?;
    let manifest_value = manifest_facts.value.as_ref().ok_or_else(policy_failure)?;
    let framework = plan.framework;
    let inventory = sources.inventory(framework).ok_or_else(policy_failure)?;
    let expected = decision_work(inventory, manifest_value, admission)?;
    let expected = work::multiply(expected, 4, admission)?;
    let actual = work::multiply(facts.operand.work, 6, admission)?;
    let amount = work::add(
        expected,
        work::add(actual, work::add(nodes, count, admission)?, admission)?,
        admission,
    )?;
    let amount =
        work::add(amount, work::multiply(work::byte_work(raw.len()), 2, admission)?, admission)?;
    let payload = work::multiply(logical, 3, admission)?;
    let mut callback = sources.admission();
    let mut native_control = sources.admission();
    let compared = admission.retain(payload, |admission| {
        work::run(admission, amount, Stage::PrepareDomain, |_| {
            let result =
                prepared.validate_report(raw, &mut |n| callback.charge(n), &mut native_control);
            let result = native_control.phase(result, Stage::PrepareDomain);
            if let Ok(()) = result {
                Ok(true)
            } else {
                work::fence(&mut native_control, Stage::PrepareDomain)?;
                Ok(false)
            }
        })
    })?;
    Ok(*compared.value())
}

impl<'scope, C: WorkControl + ?Sized> PreparedDomainV2<'scope, '_, C> {
    /// Borrow the real framework occurrence selected by native preparation on this same owner.
    pub(super) fn framework_index(&self) -> usize {
        self.framework
    }
    /// Borrow the complete maintained native result; this plain data creates no public authority.
    pub(super) fn facts(&self) -> &PreparedApplicability<'scope> {
        self.prepared.value().as_ref().expect("private complete native domain invariant")
    }
}

/// Qualify every actual visible App occurrence on one original factory invocation.
/// Prepared manifests borrow this exact owner, retain their real native/status tickets,
/// and are never installed inside the owned sources or exposed as a caller cache.
pub(super) fn qualify_visible<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
) -> WorkResult<bool> {
    let mut admission = sources.admission();
    let ordinary = work::run(&mut admission, 1, Stage::PrepareDomain, |admission| {
        sources.verify_inputs()?;
        let count = sources.profile().visible_resource_keys.len();
        let slots =
            work::multiply(count, std::mem::size_of::<PreparedDomainV2<'_, '_, C>>(), admission)?;
        let slots = work::add(
            slots,
            std::mem::size_of::<Option<Vec<PreparedDomainV2<'_, '_, C>>>>(),
            admission,
        )?;
        let plans = admission.retain(slots, |admission| {
            let mut plans = Vec::with_capacity(count);
            for key in &sources.profile().visible_resource_keys {
                admission.charge(1)?;
                let Some(index) = sources.index(key)? else {
                    return Ok(None);
                };
                if sources.policies()[index].role != Role::ApplicabilityManifest {
                    continue;
                }
                // Every distinct actual manifest retains its own full source lifecycle and
                // maintained preparation. Equal bytes never collapse original occurrences.
                let Some(plan) = prepare_selected(sources, key)? else {
                    return Ok(None);
                };
                plans.push(plan);
            }
            Ok(Some(plans))
        })?;
        let Some(plans) = plans.value().as_ref() else {
            return Ok(false);
        };
        for key in &sources.profile().visible_resource_keys {
            admission.charge(1)?;
            let Some(index) = sources.index(key)? else {
                return Ok(false);
            };
            if sources.policies()[index].role != Role::ApplicabilityReport {
                continue;
            }
            if !report_from_plans(sources, key, plans, admission)? {
                return Ok(false);
            }
        }
        sources.verify_inputs()?;
        Ok(true)
    });
    admission.phase(ordinary, Stage::RetainPrepared)
}

/// Bind one genuinely current report to ALL matching actual prepared manifest originals.
/// The private borrowed roster is made above from this owner, never selected by raw hash.
/// Every matching original runs full report equality; ambiguity is decided only afterward.
fn report_from_plans<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    key: &str,
    plans: &[PreparedDomainV2<'_, '_, C>],
    admission: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    let Some(report) = native_domain_v2::candidate(sources, key)? else {
        return Ok(false);
    };
    if sources.policies()[report.index()].role != Role::ApplicabilityReport
        || !supported_generated(sources, &report, admission)?
    {
        return Ok(false);
    }
    let Some(hint) = sources
        .facts(report.index())
        .and_then(|facts| facts.value.as_ref())
        .and_then(|value| value.get("manifest_sha256"))
        .and_then(serde_json::Value::as_str)
    else {
        return Ok(false);
    };
    if !work::run(admission, 1 + work::byte_work(hint.len()), Stage::ValidateResource, |_| {
        Ok(valid_hash(hint))
    })? {
        return Ok(false);
    }
    let mut matched = 0_usize;
    for plan in plans {
        work::fence(admission, Stage::PrepareDomain)?;
        admission.charge(1)?;
        let Observation::Present { raw_sha256, .. } = sources.observation(plan.manifest.index())?
        else {
            return Ok(false);
        };
        let width = work::add(hint.len(), raw_sha256.len(), admission)?;
        if !work::run(admission, 1 + work::byte_work(width), Stage::ValidateResource, |_| {
            Ok(hint == raw_sha256)
        })? {
            continue;
        }
        // A retained source/status result is reusable only while EVERY actual owner
        // original still verifies. Every executed native helper retains its original callbacks.
        sources.verify_inputs()?;
        if report_matches(sources, plan, &report, admission)? {
            matched = work::add(matched, 1, admission)?;
        }
        sources.verify_inputs()?;
    }
    work::run(admission, 1, Stage::ValidateResource, |_| Ok(matched == 1))
}

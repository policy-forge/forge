//! Genuine recorded App source candidacy over the complete held /2 owner.
//!
//! The manifest/report's source approval and the generated native members' independent
//! approvals remain distinct. This private producer reconstructs its actual full native
//! tuple and neutral lifecycle status; it never activates App, Mapping or server queries.

use super::capture_v2::{Admission, AdmittedValue, Observation};
use super::declarations_v2::DeclarationAdmission;
use super::native_closure_v2::ClosureFacts;
use super::native_sources_v2::NativeSourcesV2;
use super::native_work_v2::{self as work, MemberFacts};
use super::{Role, policy_failure, resolve_record_path};
use crate::lifecycle::record::{FingerprintSet, NamedHash};
use crate::lifecycle::status::CurrentArtifacts;
use crate::workspace::preparation::{Stage, WorkControl, WorkResult};

/// Private source-current result issued only by the full native producer below.
/// It is insufficient to authorize domain semantics or independently approve dependencies.
pub(super) struct DomainSourceV2<'scope, 'control, C: WorkControl + ?Sized> {
    /// Every actual root/config/original remains borrowed through all native work.
    sources: &'scope NativeSourcesV2<'control, C>,
    /// Exact actual selected visible base occurrence.
    index: usize,
    /// Actual complete tuple/status data remains retained under its original ticket.
    proof: AdmittedValue<Option<ClosureFacts>>,
}

/// Select only an actual visible pin-matched strict App original with its own complete
/// intrinsically validated latest Approved lifecycle tuple. Generated natives keep their
/// independent declared lifecycle keys; this source result does not approve those keys.
pub(super) fn candidate<'scope, 'control, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'control, C>,
    key: &str,
) -> WorkResult<Option<DomainSourceV2<'scope, 'control, C>>> {
    let mut admission = sources.admission();
    work::run(&mut admission, 1, Stage::PrepareDomain, |_| candidate_inner(sources, key))
}

/// Keep every ordinary source refusal behind the same original caller's outer post-fence.
fn candidate_inner<'scope, 'control, C: WorkControl + ?Sized>(
    sources: &'scope NativeSourcesV2<'control, C>,
    key: &str,
) -> WorkResult<Option<DomainSourceV2<'scope, 'control, C>>> {
    sources.verify_inputs()?;
    let mut admission = sources.admission();
    if !visible(sources, key, &mut admission)? {
        return Ok(None);
    }
    let Some(index) = sources.index(key)? else {
        return Ok(None);
    };
    let policy = &sources.policies()[index];
    if !matches!(policy.role, Role::ApplicabilityManifest | Role::ApplicabilityReport)
        || policy.native_identity.is_some()
    {
        return Ok(None);
    }
    let Some(selected) = sources.resource(key)? else {
        return Ok(None);
    };
    let Some(facts) = sources.facts(index) else {
        return Ok(None);
    };
    if facts.value.is_none() || facts.identity.is_some() {
        return Ok(None);
    }
    let Some(lifecycle_key) = selected.lifecycle_key() else {
        return Ok(None);
    };
    let Some(lifecycle) = sources.index(lifecycle_key)? else {
        return Ok(None);
    };
    let Some(life) = sources.resource(lifecycle_key)? else {
        return Ok(None);
    };
    if life.role() != Role::LifecycleRecord {
        return Ok(None);
    }
    let Some(record_facts) = sources.facts(lifecycle) else {
        return Ok(None);
    };
    let Some(parsed) = record_facts.record.as_ref() else {
        return Ok(None);
    };
    let record = parsed.record();
    let budget = bound(sources, lifecycle, record_facts, &mut admission)?;
    let proof = admission.retain(budget.logical, |admission| {
        let current = work::run(admission, budget.work, Stage::PrepareDomain, |admission| {
            current(sources, index, lifecycle, record, admission)
        })?;
        let Some((generated, current)) = current else {
            return Ok(None);
        };
        // Only actual immutable full parser output eliminates intrinsic revalidation.
        // Captured correspondence/projection and original ordinary-result fences remain.
        let neutral = work::project_status(parsed, &current, None, budget.work, admission)?;
        let Some(neutral) = neutral else {
            return Ok(None);
        };
        Ok(Some(ClosureFacts { lifecycle, source: index, generated, current, neutral }))
    })?;
    let approved = match proof.value().as_ref() {
        Some(facts) => approved(facts, &mut admission)?,
        None => false,
    };
    if !approved {
        return Ok(None);
    }
    sources.verify_inputs()?;
    work::fence(&mut admission, Stage::RetainPrepared)?;
    Ok(Some(DomainSourceV2 { sources, index, proof }))
}

/// Admit both exact selector and every selected profile key before inspecting visibility.
fn visible<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    key: &str,
    admission: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    let keys = &sources.profile().visible_resource_keys;
    let mut width = work::multiply(keys.len(), key.len(), admission)?;
    for candidate in keys {
        admission.charge(1)?;
        width = work::add(width, candidate.len(), admission)?;
    }
    let amount = work::add(keys.len(), work::byte_work(width), admission)?;
    work::run(admission, amount, Stage::PrepareDomain, |_| Ok(keys.iter().any(|row| row == key)))
}

/// Complete source-backed tuple/status/path/registry envelope; no data or authority is
/// constructed from this scalar descriptor. All repeated route/tuple/status work is charged.
struct Bound {
    /// Retained tuple vectors, raw fingerprints, status copies and full temporary routes.
    logical: usize,
    /// Complete repeated lookup/comparison operands before native tuple/status evaluation.
    work: usize,
}

/// Account every actual declared path/key and full record tuple before path composition,
/// generated vector construction, native identity comparisons, sorting and status copies.
fn bound<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    lifecycle: usize,
    facts: &MemberFacts,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Bound> {
    let record = facts.record.as_ref().ok_or_else(policy_failure)?.record();
    let rows = work::add(record.policy.generated_artifacts.len(), 1, admission)?;
    let mut route_width = sources.policies()[lifecycle].path.len();
    let mut key_width = 0;
    for policy in sources.policies() {
        admission.charge(1)?;
        route_width = work::add(route_width, policy.path.len(), admission)?;
        key_width = work::add(key_width, policy.key.len(), admission)?;
    }
    for expected in std::iter::once(&record.policy.source).chain(&record.policy.generated_artifacts)
    {
        admission.charge(1)?;
        route_width = work::add(route_width, expected.path.len(), admission)?;
    }
    let paths = work::multiply(route_width, rows, admission)?;
    let ordinary = work::multiply(sources.policies().len(), rows, admission)?;
    let passes = work::multiply(paths, 8, admission)?;
    let work =
        work::add(ordinary, work::byte_work(work::add(passes, key_width, admission)?), admission)?;
    let string_copies = work::multiply(facts.operand.strings, 8, admission)?;
    let nodes = work::add(facts.operand.nodes, facts.operand.fields, admission)?;
    let storage = work::multiply(nodes, 256, admission)?;
    let logical = work::add(
        work::add(string_copies, storage, admission)?,
        work::add(work::multiply(route_width, 4, admission)?, 4096, admission)?,
        admission,
    )?;
    Ok(Bound { logical, work })
}

/// Resolve only the actual selected source plus complete captured generated native tuple.
/// No href, import, generated copy, I/O or same-lifecycle shortcut is permitted here.
fn current<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    selected: usize,
    lifecycle: usize,
    record: &crate::lifecycle::record::LifecycleRecord,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Option<(Vec<usize>, CurrentArtifacts)>> {
    if record.policy.source.oscal_type.is_some() || record.policy.source.root_uuid.is_some() {
        return Ok(None);
    }
    let source_policy = &sources.policies()[selected];
    let life = &sources.policies()[lifecycle];
    let Some(path) = resolve_record_path(&life.path, &record.policy.source.path) else {
        return Ok(None);
    };
    if path != source_policy.path
        || source_policy.lifecycle_key.as_deref() != Some(life.key.as_str())
    {
        return Ok(None);
    }
    let Observation::Present { raw_sha256: source_sha, .. } = sources.observation(selected)? else {
        return Ok(None);
    };
    let width = work::add(source_sha.len(), record.policy.source.sha256.len(), admission)?;
    if !work::run(admission, 1 + work::byte_work(width), Stage::ValidateResource, |_| {
        Ok(source_sha == record.policy.source.sha256)
    })? {
        return Ok(None);
    }
    let count = record.policy.generated_artifacts.len();
    admission.charge(count)?;
    let mut generated = Vec::with_capacity(count);
    let mut hashes = Vec::with_capacity(count);
    for expected in &record.policy.generated_artifacts {
        work::fence(admission, Stage::PrepareDomain)?;
        let Some(path) = resolve_record_path(&life.path, &expected.path) else {
            return Ok(None);
        };
        let Some(index) = sources.policies().iter().position(|policy| policy.path == path) else {
            return Ok(None);
        };
        let policy = &sources.policies()[index];
        if policy.role.native_model().is_none() {
            return Ok(None);
        }
        let Some(resource) = sources.resource(&policy.key)? else {
            return Ok(None);
        };
        let Some(identity) = resource.native_identity() else {
            return Ok(None);
        };
        if resource.native_value().is_none() {
            return Ok(None);
        }
        let Observation::Present { raw_sha256, .. } = sources.observation(index)? else {
            return Ok(None);
        };
        let mut width = work::add(expected.sha256.len(), raw_sha256.len(), admission)?;
        for text in
            [expected.oscal_type.as_deref(), expected.root_uuid.as_deref()].into_iter().flatten()
        {
            width = work::add(width, text.len(), admission)?;
        }
        for text in [&identity.model, &identity.root_id] {
            width = work::add(width, text.len(), admission)?;
        }
        let matches =
            work::run(admission, 1 + work::byte_work(width), Stage::ValidateResource, |_| {
                Ok(expected.sha256 == raw_sha256
                    && expected.oscal_type.as_deref() == Some(identity.model.as_str())
                    && expected.root_uuid.as_deref() == Some(identity.root_id.as_str()))
            })?;
        if !matches {
            return Ok(None);
        }
        generated.push(index);
        hashes.push(NamedHash { path: expected.path.clone(), sha256: raw_sha256.to_owned() });
    }
    let mut width = 0;
    for hash in &hashes {
        admission.charge(1)?;
        width =
            work::add(width, work::add(hash.path.len(), hash.sha256.len(), admission)?, admission)?;
    }
    let pairs = work::multiply(count, count, admission)?;
    let repeated = work::multiply(pairs, work::byte_work(width), admission)?;
    let sorting = work::add(pairs, repeated, admission)?;
    work::run(admission, sorting, Stage::PrepareDomain, |_| {
        hashes.sort();
        Ok(())
    })?;
    Ok(Some((
        generated,
        CurrentArtifacts {
            fingerprints: FingerprintSet {
                source_sha256: source_sha.to_owned(),
                generated_artifacts: hashes,
            },
            identity_changes: Vec::new(),
        },
    )))
}

/// Pre-admit every exact current/approved fingerprint and membership row before equality.
fn approved<C: WorkControl + ?Sized>(
    facts: &ClosureFacts,
    admission: &mut Admission<'_, C>,
) -> WorkResult<bool> {
    let mut rows = work::add(facts.generated.len(), 1, admission)?;
    let mut width = facts.neutral.derived_status.len();
    for fingerprints in std::iter::once(&facts.current.fingerprints)
        .chain(facts.neutral.approved_fingerprints.as_ref())
    {
        width = work::add(width, fingerprints.source_sha256.len(), admission)?;
        rows = work::add(rows, fingerprints.generated_artifacts.len(), admission)?;
        for hash in &fingerprints.generated_artifacts {
            admission.charge(1)?;
            width = work::add(
                width,
                work::add(hash.path.len(), hash.sha256.len(), admission)?,
                admission,
            )?;
        }
    }
    let amount = work::add(rows, work::byte_work(width), admission)?;
    work::run(admission, amount, Stage::ValidateResource, |_| Ok(facts.current_approved()))
}

impl<'scope, C: WorkControl + ?Sized> DomainSourceV2<'scope, '_, C> {
    /// Borrow the actual selected original occurrence; this does not grant native dependency approval.
    pub(super) fn index(&self) -> usize {
        self.index
    }
    /// Borrow every actual generated member; callers must qualify all relevant domain dependencies.
    pub(super) fn generated(&self) -> &[usize] {
        &self.proof.value().as_ref().expect("private source qualification invariant").generated
    }
    /// Borrow actual captured bytes from the same original owner, with absence refused.
    pub(super) fn bytes(&self) -> WorkResult<&'scope [u8]> {
        match self.sources.observation(self.index)? {
            Observation::Present { bytes, .. } => Ok(bytes),
            Observation::Absent => Err(policy_failure()),
        }
    }
}

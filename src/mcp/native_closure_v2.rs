//! Complete actual lifecycle/source/generated closure over the original /2 held roster.
//!
//! Stored labels, fingerprints, caller indices and membership alone issue no approval.
//! All private facts below come from actual full intrinsic record parsing and neutral
//! status projection on complete captured native/source tuples. Mapping/applicability/linkage
//! domain activation remains unavailable.

use super::capture_v2::{Admission, AdmittedValue, HeldInputsV2, Observation};
use super::declarations_v2::{DeclarationAdmission, ProjectDeclarationV2};
use super::native_work_v2::{self as work, MemberFacts};
use super::{ResourcePolicy, Role, policy_failure, resolve_record_path};
use crate::lifecycle::record::{self, FingerprintSet, LifecycleState, NamedHash};
use crate::lifecycle::status::{CurrentArtifacts, StatusReport};
use crate::workspace::preparation::{Stage, WorkControl, WorkResult};

/// Privately computed complete lifecycle facts, not a caller-created currentness token.
pub(super) struct ClosureFacts {
    /// Actual intrinsically validated lifecycle member whose complete tuple was projected.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Retain the exact lifecycle occurrence in the complete native facts; the cfg record oracle reads it without changing production closure geometry."
        )
    )]
    pub(super) lifecycle: usize,
    /// Actual same-owner source occurrence selected by the real record-relative path.
    pub(super) source: usize,
    /// Complete actual generated occurrences, preserving every declared dependency.
    pub(super) generated: Vec<usize>,
    /// Exact full current original source/generated raw tuple and identity changes.
    pub(super) current: CurrentArtifacts,
    /// Actual intrinsically validated neutral schedule-independent status.
    pub(super) neutral: StatusReport,
}
impl ClosureFacts {
    /// Test the actual complete immutable neutral result before issuing an approved view.
    pub(super) fn current_approved(&self) -> bool {
        self.neutral.state == LifecycleState::Approved
            && self.neutral.approved_fingerprints.as_ref() == Some(&self.current.fingerprints)
            && self.neutral.artifact_identity_changes.is_empty()
            && self.neutral.blockers.is_empty()
            && self.neutral.derived_status == "approved"
    }
}

/// Entire retained lifecycle roster under its original joint byte ticket.
pub(super) type PreparedClosures = AdmittedValue<Vec<Option<AdmittedValue<Option<ClosureFacts>>>>>;

/// Prepare each actual lifecycle once; generated members keep exact same independent binding.
pub(super) fn prepare<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    project: &ProjectDeclarationV2,
    members: &[Option<AdmittedValue<MemberFacts>>],
) -> WorkResult<PreparedClosures> {
    let mut admission = held.admission();
    let entry = std::mem::size_of::<Option<AdmittedValue<Option<ClosureFacts>>>>();
    let slots = work::multiply(project.resources.len(), entry, &mut admission)?;
    let slots = work::add(
        slots,
        std::mem::size_of::<Vec<Option<AdmittedValue<Option<ClosureFacts>>>>>(),
        &mut admission,
    )?;
    admission.retain(slots, |admission| {
        let mut result = Vec::with_capacity(project.resources.len());
        for (index, policy) in project.resources.iter().enumerate() {
            admission.charge(1)?;
            let proof = if policy.role == Role::LifecycleRecord {
                let facts = members.get(index).and_then(Option::as_ref).map(AdmittedValue::value);
                match facts.and_then(|facts| facts.record.as_ref().map(|record| (record, facts))) {
                    Some((record, facts)) if pin_matched(held, policy)? => Some(prepare_record(
                        held,
                        &project.resources,
                        members,
                        index,
                        record,
                        facts,
                        admission,
                    )?),
                    _ => None,
                }
            } else {
                None
            };
            result.push(proof);
        }
        Ok(result)
    })
}

/// Actual selected raw bytes must match their complete independent declaration pin.
fn pin_matched<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    policy: &ResourcePolicy,
) -> WorkResult<bool> {
    let mut admission = held.admission();
    let observed = held.original(&policy.key)?.ok_or_else(policy_failure)?;
    work::run(
        &mut admission,
        1 + work::byte_work(policy.expected_sha256.len()),
        Stage::ValidateResource,
        |_| {
            Ok(
                matches!(observed, Observation::Present { raw_sha256, .. } if raw_sha256 == policy.expected_sha256),
            )
        },
    )
}

/// Reserve complete tuple/status/registry/private-string payload before native construction.
fn prepare_record<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    policies: &[ResourcePolicy],
    members: &[Option<AdmittedValue<MemberFacts>>],
    lifecycle: usize,
    parsed: &record::ParsedLifecycleRecord,
    facts: &MemberFacts,
    admission: &mut Admission<'_, C>,
) -> WorkResult<AdmittedValue<Option<ClosureFacts>>> {
    let record = parsed.record();
    let tuple_rows = work::add(record.policy.generated_artifacts.len(), 1, admission)?;
    let lookup = work::multiply(tuple_rows, policies.len(), admission)?;
    let mut path_width = 0;
    for policy in policies {
        path_width = work::add(path_width, policy.path.len(), admission)?;
        let keys = work::add(
            policy.lifecycle_key.as_ref().map_or(0, String::len),
            policies[lifecycle].key.len(),
            admission,
        )?;
        path_width = work::add(path_width, keys, admission)?;
    }
    for expected in std::iter::once(&record.policy.source).chain(&record.policy.generated_artifacts)
    {
        path_width = work::add(path_width, expected.path.len(), admission)?;
    }
    path_width = work::add(path_width, policies[lifecycle].path.len(), admission)?;
    let comparisons = work::multiply(tuple_rows, work::byte_work(path_width), admission)?;
    let ordinary = work::add(lookup, comparisons, admission)?;
    let strings = work::multiply(facts.operand.strings, 8, admission)?;
    let row_count = work::add(facts.operand.nodes, facts.operand.fields, admission)?;
    let rows = work::multiply(row_count, 256, admission)?;
    let payload = work::add(work::add(strings, rows, admission)?, 4096, admission)?;
    // Ordinary incomplete closure returns None, but original capacity/control is never caught.
    admission.retain(payload, |admission| {
        let current = work::run(admission, ordinary, Stage::PrepareDomain, |admission| {
            complete_current(held, policies, members, lifecycle, record, admission)
        })?;
        let Some((source, generated, current)) = current else {
            return Ok(None);
        };
        let neutral = work::project_status(parsed, &current, None, ordinary, admission)?;
        let Some(neutral) = neutral else {
            return Ok(None);
        };
        Ok(Some(ClosureFacts { lifecycle, source, generated, current, neutral }))
    })
}

/// Build only a complete same-lifecycle policy source plus actual generated native tuple.
fn complete_current<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    policies: &[ResourcePolicy],
    members: &[Option<AdmittedValue<MemberFacts>>],
    lifecycle: usize,
    record: &record::LifecycleRecord,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Option<(usize, Vec<usize>, CurrentArtifacts)>> {
    let life = &policies[lifecycle];
    if record.policy.source.oscal_type.is_some() || record.policy.source.root_uuid.is_some() {
        return Ok(None);
    }
    let Some(source_path) = resolve_record_path(&life.path, &record.policy.source.path) else {
        return Ok(None);
    };
    let Some(source) = policies.iter().position(|policy| policy.path == source_path) else {
        return Ok(None);
    };
    let source_policy = &policies[source];
    if !matches!(source_policy.role, Role::PolicySource | Role::LifecycleSource)
        || source_policy.lifecycle_key.as_deref() != Some(life.key.as_str())
        || !pin_matched(held, source_policy)?
    {
        return Ok(None);
    }
    let Observation::Present { raw_sha256: source_sha, .. } =
        held.original(&source_policy.key)?.ok_or_else(policy_failure)?
    else {
        return Ok(None);
    };
    admission.charge(record.policy.generated_artifacts.len())?;
    let mut generated = Vec::with_capacity(record.policy.generated_artifacts.len());
    let mut hashes = Vec::with_capacity(record.policy.generated_artifacts.len());
    let mut identity_changes = Vec::new();
    for expected in &record.policy.generated_artifacts {
        admission.charge(1)?;
        let Some(path) = resolve_record_path(&life.path, &expected.path) else {
            return Ok(None);
        };
        let Some(index) = policies.iter().position(|policy| policy.path == path) else {
            return Ok(None);
        };
        let policy = &policies[index];
        if policy.lifecycle_key.as_deref() != Some(life.key.as_str())
            || policy.role.native_model().is_none()
            || !pin_matched(held, policy)?
        {
            return Ok(None);
        }
        // Facts are borrowed only from this factory's complete same-owner maintained validator output.
        let Observation::Present { raw_sha256, .. } =
            held.original(&policy.key)?.ok_or_else(policy_failure)?
        else {
            return Ok(None);
        };
        let Some(identity) = members
            .get(index)
            .and_then(Option::as_ref)
            .and_then(|facts| facts.value().identity.as_ref())
        else {
            return Ok(None);
        };
        let mut width = 0;
        for observed in std::iter::once(identity).chain(policy.native_identity.as_ref()) {
            for text in [
                &observed.model,
                &observed.root_id,
                &observed.document_version,
                &observed.oscal_version,
            ] {
                width = work::add(width, text.len(), admission)?;
            }
        }
        for text in
            [expected.oscal_type.as_deref(), expected.root_uuid.as_deref()].into_iter().flatten()
        {
            width = work::add(width, text.len(), admission)?;
        }
        let changes =
            work::run(admission, 1 + work::byte_work(width), Stage::ValidateResource, |_| {
                if policy.native_identity.as_ref() != Some(identity) {
                    return Ok(None);
                }
                Ok(Some(
                    expected.oscal_type.as_deref() != Some(identity.model.as_str())
                        || expected.root_uuid.as_deref() != Some(identity.root_id.as_str()),
                ))
            })?;
        let Some(changes) = changes else {
            return Ok(None);
        };
        if changes {
            identity_changes.push(expected.path.clone());
        }
        generated.push(index);
        hashes.push(NamedHash { path: expected.path.clone(), sha256: raw_sha256.to_owned() });
    }
    finish_current(source, generated, hashes, identity_changes, source_sha, admission)
}

/// Finish the already admitted complete tuple with exactly the original sorting/hash order.
fn finish_current<C: WorkControl + ?Sized>(
    source: usize,
    generated: Vec<usize>,
    mut hashes: Vec<NamedHash>,
    identity_changes: Vec<String>,
    source_sha: &str,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Option<(usize, Vec<usize>, CurrentArtifacts)>> {
    let n = hashes.len();
    let comparisons = work::multiply(n, n, admission)?;
    let mut width = 0;
    for hash in &hashes {
        admission.charge(1)?;
        let strings = work::add(hash.path.len(), hash.sha256.len(), admission)?;
        width = work::add(width, strings, admission)?;
    }
    let repeated = work::multiply(comparisons, work::byte_work(width), admission)?;
    let sorting = work::add(comparisons, repeated, admission)?;
    admission.charge(sorting)?;
    hashes.sort();
    Ok(Some((
        source,
        generated,
        CurrentArtifacts {
            fingerprints: FingerprintSet {
                source_sha256: source_sha.to_owned(),
                generated_artifacts: hashes,
            },
            identity_changes,
        },
    )))
}

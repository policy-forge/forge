//! Genuine complete Lifecycle re-review receiver on original held native inputs.
//! Only already Approved/current intrinsic /2 records qualify. Generated validation
//! remains the maintained model/root/UUID identity predicate; it is not OSCAL schema
//! or native semantic validation. No signing, native transition or new eligibility is added.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{
    ArtifactFingerprint, FingerprintSet, LifecycleRecord, LifecycleState, NamedHash,
};
use crate::lifecycle::status::CurrentArtifacts;
use crate::workspace::preparation::WorkControl;

use super::capture::lifecycle::LifecyclePurpose;
use super::capture::{HeldReviewInputs, ReviewCapture};
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::decode_lifecycle_inputs;
use super::hash_v2::RosterEntry;
use super::wire_v2::{
    LifecycleArtifactRoute, LifecycleInputsV1, NativeModelV2, SourceKindV2, SourcePinV2,
};

/// Complete measured native admission; this child cannot capture or issue an owner.
#[path = "lifecycle_capture_native.rs"]
mod native;

/// Intrinsic record identity marker differs from the review envelope revision.
const RECORD_PROFILE: &str = "forge.lifecycle-record-intrinsic/1";
/// Complete opaque byte original profile; no UTF-8 or JSON is required.
const SOURCE_PROFILE: &str = "forge.opaque-source-bytes/1";
/// Maintained generated identity predicate, without an OSCAL schema claim.
const GENERATED_PROFILE: &str = "forge.lifecycle-generated-identity/1";

/// One actual receiver-issued native occurrence; no detached constructor exists.
pub(super) struct LifecycleMember {
    /// Actual successful Entry index, shared only by real compatible repeated originals.
    index: usize,
    /// Actual allocation lease from this original session, never reconstituted by raw hash.
    lease: CaptureLease,
    /// Complete native occurrence purpose, including compatible repeats.
    purpose: LifecyclePurpose,
    /// Actual captured byte role, including intrinsic record/source reuse.
    role: CaptureRole,
    /// Actual confined original capture spelling, retained privately for final membership.
    path: PathBuf,
    /// Exact native declared path; null only for the intrinsic record occurrence.
    declared_path: Option<String>,
    /// Full actual observed occurrence tuple, not an expected locator assertion.
    pin: SourcePinV2,
}
impl LifecycleMember {
    /// Borrow the genuine original Entry registration.
    pub(super) fn index(&self) -> usize {
        self.index
    }
    /// Borrow the genuine native allocation lease.
    pub(super) fn lease(&self) -> &CaptureLease {
        &self.lease
    }
    /// Preserve actual successful purpose order and multiplicity.
    pub(super) fn purpose(&self) -> LifecyclePurpose {
        self.purpose
    }
    /// Preserve actual byte role, independent of the public validation profile.
    pub(super) fn role(&self) -> CaptureRole {
        self.role
    }
    /// Borrow the private actual capture spelling for original membership only.
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

/// Genuine receiver result before complete input registration has finished.
/// No Clone/Debug/Serialize constructor, caller-success flag or arbitrary Rc repair exists.
pub(crate) struct PendingLifecycleCohort {
    /// Real explicit Auxiliary original, retained separately from Source entries.
    locator_index: usize,
    /// Actual original locator allocation survives all native and final phases.
    locator_lease: CaptureLease,
    /// Complete intrinsic current record, including every historical event/private string.
    record: LifecycleRecord,
    /// Complete native Record+Source+Generated occurrence order, including repeats.
    members: Vec<LifecycleMember>,
    /// Complete minimized full tuples in public artifact-key byte order.
    pins: Vec<SourcePinV2>,
}
impl PendingLifecycleCohort {
    /// Borrow only the actual receiver-issued locator original for the private sealer.
    pub(super) fn locator(&self) -> (usize, &CaptureLease) {
        (self.locator_index, &self.locator_lease)
    }
    /// Borrow all actual successful occurrences; an external list cannot construct this pending type.
    pub(super) fn members(&self) -> &[LifecycleMember] {
        &self.members
    }

    /// Consume the real capture only after ALL operation queue/response/auxiliary registrations.
    /// The owner is made from capture.finish, never a supplied Rc or detached list of indices.
    pub(crate) fn finish_and_seal(
        self,
        capture: ReviewCapture,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<ApprovedLifecycleClosure, ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            ledger.derived(
                std::mem::size_of::<HeldReviewInputs>()
                    .checked_add(2 * std::mem::size_of::<usize>())
                    .ok_or(ContractError::Capacity)?,
            )?;
            let held = Rc::new(capture.finish());
            let membership = held.verify_lifecycle_cohort(&self, ledger, control);
            ledger.checkpoint(control)?;
            membership?;
            let current = held.verify_inputs(ledger, control);
            ledger.checkpoint(control)?;
            current?;
            ledger.derived(std::mem::size_of::<ApprovedLifecycleClosure>())?;
            ledger.checkpoint(control)?;
            Ok(ApprovedLifecycleClosure {
                record: self.record,
                members: self.members,
                pins: self.pins,
                locator_lease: self.locator_lease,
                held,
            })
        })
    }
}

/// Actual neutral native Approved/current closure retaining its complete physical owner.
/// No detached/fingerprint/Decoded/boolean constructor, Clone, Debug or serializer exists.
pub(crate) struct ApprovedLifecycleClosure {
    /// Full intrinsic approved record remains private; its raw pin binds all history.
    record: LifecycleRecord,
    /// Complete ordered original occurrence leases and native routes.
    members: Vec<LifecycleMember>,
    /// Complete path-free observed public tuple projection.
    pins: Vec<SourcePinV2>,
    /// Actual private locator allocation remains held even after decode intermediates drop.
    locator_lease: CaptureLease,
    /// One genuine finished proof owner, held through Root's final queue/publication fences.
    held: Rc<HeldReviewInputs>,
}
impl ApprovedLifecycleClosure {
    /// Borrow full native facts only for the genuine adapter; no private prose is public wire data.
    pub(crate) fn record(&self) -> &LifecycleRecord {
        &self.record
    }
    /// Borrow the complete observed `SourcePinV2` set, never a chosen subset.
    pub(crate) fn source_pins(&self) -> &[SourcePinV2] {
        &self.pins
    }
    /// Borrow complete native purpose occurrences for the separately reviewed plain hash codec.
    pub(crate) fn roster(&self) -> impl ExactSizeIterator<Item = RosterEntry<'_>> {
        self.members.iter().map(|row| RosterEntry {
            purpose: row.pin.kind,
            route: row.declared_path.as_deref(),
            pin: &row.pin,
        })
    }
    /// Borrow the sole genuine full-input owner for Root's exact queue/response binding.
    pub(crate) fn held_inputs(&self) -> &Rc<HeldReviewInputs> {
        &self.held
    }
    /// Recheck this complete original generation under the unchanged original invocation work owner.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            // A retained actual locator lease is part of the full held proof, not a hash repair.
            ledger.visits(1)?;
            let _ = self.locator_lease.bytes();
            let result = self.held.verify_inputs(ledger, control);
            ledger.checkpoint(control)?;
            result
        })
    }
}

/// Read the actual captured locator, intrinsic record and complete native artifact roster.
/// All declared routes are compared to real record-relative membership before artifact reads.
/// Ordinary native/shape failures are minimized; actual first original ledger/control stops win.
pub(crate) fn read_pending(
    capture: &mut ReviewCapture,
    locator_path: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingLifecycleCohort, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let ordinary = read_inner(capture, locator_path, ledger, control);
        ledger.checkpoint(control)?;
        ordinary
    })
}

/// Complete receiver chronology; early ordinary failures are fenced by `read_pending`.
fn read_inner(
    capture: &mut ReviewCapture,
    locator_path: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingLifecycleCohort, ContractError> {
    let locator_index = capture.required_lifecycle_locator(locator_path, ledger, control)?;
    ledger.derived(std::mem::size_of::<CaptureLease>())?;
    let locator_lease = capture.lease(locator_index)?;
    let decoded = decode_lifecycle_inputs(locator_lease.bytes(), ledger, control);
    ledger.checkpoint(control)?;
    let decoded = decoded?;
    let locator = decoded.document();
    let record_path = Path::new(&locator.record.path);
    let index = capture.required_lifecycle_source(
        record_path,
        LifecyclePurpose::Record,
        ledger,
        control,
    )?;
    ledger.derived(std::mem::size_of::<CaptureLease>())?;
    let record_lease = capture.lease(index)?;
    let (record, native_plan) = native::record(record_lease.bytes(), ledger, control)?;
    if compare(&record.schema_version, crate::lifecycle::record::SCHEMA_VERSION, ledger)?
        != std::cmp::Ordering::Equal
        || record.state != LifecycleState::Approved
    {
        return Err(ContractError::Binding);
    }
    let ordered = complete_routes(locator, &record, ledger, control)?;
    let mut members = Vec::new();
    let record_pin = pin(LifecyclePurpose::Record, record_lease.bytes(), None, ledger, control)?;
    member_push(
        &mut members,
        LifecycleMember {
            index,
            lease: record_lease,
            purpose: LifecyclePurpose::Record,
            role: CaptureRole::LifecycleRecord,
            path: path_copy(record_path, ledger)?,
            declared_path: None,
            pin: record_pin,
        },
        ledger,
    )?;
    artifact(capture, &locator.source, LifecyclePurpose::Source, &mut members, ledger, control)?;
    for (ordinal, route) in locator.generated_artifacts.iter().enumerate() {
        artifact(
            capture,
            route,
            LifecyclePurpose::Generated(ordinal),
            &mut members,
            ledger,
            control,
        )?;
    }
    let current = observed_artifacts(&ordered, &members, ledger, control)?;
    let status = native::status(&record, &current, &native_plan, ledger, control)?;
    current_status(&status, &current, ledger, control)?;
    let pins = public_pins(&members, ledger, control)?;
    ledger.derived(std::mem::size_of::<PendingLifecycleCohort>())?;
    ledger.checkpoint(control)?;
    Ok(PendingLifecycleCohort { locator_index, locator_lease, record, members, pins })
}

/// Validate exact complete native membership and sort only full borrowed native path operands.
fn complete_routes<'a>(
    locator: &LifecycleInputsV1,
    record: &'a LifecycleRecord,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<&'a ArtifactFingerprint>, ContractError> {
    if record.policy.generated_artifacts.len() != locator.generated_artifacts.len()
        || locator.generated_artifacts.len() > 98
    {
        return Err(ContractError::Binding);
    }
    ledger.visits(1)?;
    ledger.derived(
        record
            .policy
            .generated_artifacts
            .len()
            .checked_mul(std::mem::size_of::<&ArtifactFingerprint>())
            .ok_or(ContractError::Capacity)?,
    )?;
    let mut ordered: Vec<&ArtifactFingerprint> = Vec::new();
    ordered
        .try_reserve_exact(record.policy.generated_artifacts.len())
        .map_err(|_| ledger.capacity())?;
    for row in &record.policy.generated_artifacts {
        ledger.checkpoint(control)?;
        let mut slot = 0;
        while slot < ordered.len() {
            if compare(&row.path, &ordered[slot].path, ledger)? == std::cmp::Ordering::Less {
                break;
            }
            slot += 1;
        }
        ledger.bytes(
            (ordered.len() - slot)
                .checked_mul(std::mem::size_of::<&ArtifactFingerprint>())
                .ok_or(ContractError::Capacity)?,
        )?;
        ledger.visits(ordered.len() - slot)?;
        ordered.insert(slot, row);
    }
    route_match(
        &locator.record.path,
        &record.policy.source.path,
        &locator.source,
        ledger,
        control,
    )?;
    for (native, declared) in ordered.iter().zip(&locator.generated_artifacts) {
        route_match(&locator.record.path, &native.path, declared, ledger, control)?;
    }
    ledger.checkpoint(control)?;
    Ok(ordered)
}

/// Compare exact private declared strings and the complete maintained confined join relation.
fn route_match(
    record_route: &str,
    expected_declared: &str,
    declared: &LifecycleArtifactRoute,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    if compare(expected_declared, &declared.declared_path, ledger)? != std::cmp::Ordering::Equal {
        return Err(ContractError::Binding);
    }
    let record_path = Path::new(record_route);
    let relative = Path::new(expected_declared);
    ledger.bytes(
        record_route.len().checked_add(expected_declared.len()).ok_or(ContractError::Capacity)?,
    )?;
    ledger.visits(
        record_path
            .components()
            .count()
            .checked_add(relative.components().count())
            .ok_or(ContractError::Capacity)?,
    )?;
    // Intrinsic validation has already preserved the native confined route shape.
    // The tighter review normalized route profile may refuse native admitted spellings;
    // do not normalize the private declared string or alter native fingerprint operands.
    crate::linkage::fresh::validate_relative(relative).map_err(|_| ContractError::Invalid)?;
    let parent = record_path.parent().ok_or(ContractError::Invalid)?;
    let extent = parent
        .as_os_str()
        .as_encoded_bytes()
        .len()
        .checked_add(expected_declared.len())
        .and_then(|n| n.checked_add(1))
        .ok_or(ContractError::Capacity)?;
    ledger.derived(
        extent.checked_add(std::mem::size_of::<PathBuf>()).ok_or(ContractError::Capacity)?,
    )?;
    let joined = parent.join(relative);
    ledger.bytes(
        joined
            .as_os_str()
            .as_encoded_bytes()
            .len()
            .checked_add(declared.path.len())
            .ok_or(ContractError::Capacity)?,
    )?;
    ledger.visits(1)?;
    if joined != Path::new(&declared.path) {
        return Err(ContractError::Binding);
    }
    ledger.checkpoint(control)
}

/// Capture one complete artifact occurrence and invoke the actual maintained identity predicate.
fn artifact(
    capture: &mut ReviewCapture,
    route: &LifecycleArtifactRoute,
    purpose: LifecyclePurpose,
    members: &mut Vec<LifecycleMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    let path = Path::new(&route.path);
    let index = capture.required_lifecycle_source(path, purpose, ledger, control)?;
    ledger.derived(std::mem::size_of::<CaptureLease>())?;
    let lease = capture.lease(index)?;
    let identity = if matches!(purpose, LifecyclePurpose::Generated(_)) {
        Some(native::identity(path, lease.bytes(), ledger, control)?)
    } else {
        let ordinary = crate::lifecycle::artifact_identity_from_bytes(path, lease.bytes(), false)
            .map_err(|_| ContractError::Binding);
        ledger.checkpoint(control)?;
        ordinary?;
        None
    };
    let pin = pin(purpose, lease.bytes(), identity, ledger, control)?;
    ledger.derived(
        route
            .declared_path
            .len()
            .checked_add(std::mem::size_of::<String>())
            .ok_or(ContractError::Capacity)?,
    )?;
    ledger.bytes(route.declared_path.len())?;
    let declared_path = Some(route.declared_path.clone());
    let role = capture.role(index)?;
    member_push(
        members,
        LifecycleMember {
            index,
            lease,
            purpose,
            role,
            path: path_copy(path, ledger)?,
            declared_path,
            pin,
        },
        ledger,
    )?;
    ledger.checkpoint(control)
}

/// Clone only an admitted complete actual capture route, privately.
fn path_copy(path: &Path, ledger: &mut ContractLedger) -> Result<PathBuf, ContractError> {
    let length = path.as_os_str().as_encoded_bytes().len();
    ledger.bytes(length)?;
    ledger.derived(
        length.checked_add(std::mem::size_of::<PathBuf>()).ok_or(ContractError::Capacity)?,
    )?;
    Ok(path.to_path_buf())
}
/// Grow the complete occurrence list after its actual descriptor and reallocation work are admitted.
fn member_push(
    members: &mut Vec<LifecycleMember>,
    row: LifecycleMember,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.derived(std::mem::size_of::<LifecycleMember>())?;
    if members.len() == members.capacity() {
        ledger.bytes(
            members
                .len()
                .checked_mul(std::mem::size_of::<LifecycleMember>())
                .ok_or(ContractError::Capacity)?,
        )?;
    }
    members.try_reserve(1).map_err(|_| ledger.capacity())?;
    members.push(row);
    Ok(())
}

/// Meter complete raw hashes and all owned minimized tuple strings before producing them.
fn pin(
    purpose: LifecyclePurpose,
    raw: &[u8],
    identity: Option<(Option<String>, Option<String>)>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<SourcePinV2, ContractError> {
    ledger.checkpoint(control)?;
    ledger.bytes(raw.len())?;
    ledger.derived(
        64_usize.checked_add(std::mem::size_of::<SourcePinV2>()).ok_or(ContractError::Capacity)?,
    )?;
    let hash = sha256_hex(raw);
    ledger.checkpoint(control)?;
    let (key_bytes, schema_bytes, profile) = match purpose {
        LifecyclePurpose::Record => (
            "lifecycle:record".len(),
            crate::lifecycle::record::SCHEMA_VERSION.len(),
            RECORD_PROFILE,
        ),
        LifecyclePurpose::Source => ("lifecycle:source".len(), 0, SOURCE_PROFILE),
        LifecyclePurpose::Generated(ordinal) => {
            if ordinal >= 98 {
                return Err(ContractError::Binding);
            }
            ("lifecycle:generated:".len() + if ordinal < 10 { 1 } else { 2 }, 0, GENERATED_PROFILE)
        }
    };
    let tuple_strings = key_bytes
        .checked_add(schema_bytes)
        .and_then(|n| n.checked_add(profile.len()))
        .ok_or(ContractError::Capacity)?;
    ledger.derived(tuple_strings)?;
    ledger.bytes(tuple_strings)?;
    let (key, kind, schema, model, root_uuid) = match purpose {
        LifecyclePurpose::Record => (
            "lifecycle:record".to_string(),
            SourceKindV2::LifecycleRecord,
            Some(crate::lifecycle::record::SCHEMA_VERSION.to_string()),
            None,
            None,
        ),
        LifecyclePurpose::Source => {
            ("lifecycle:source".to_string(), SourceKindV2::OpaqueSource, None, None, None)
        }
        LifecyclePurpose::Generated(ordinal) => {
            let (model, uuid) = identity.ok_or(ContractError::Binding)?;
            let model_name = model.as_deref().ok_or(ContractError::Binding)?;
            ledger.bytes(model_name.len().checked_mul(6).and_then(|n| n.checked_add(b"catalog profile component-definition mapping-collection system-security-plan plan-of-action-and-milestones".len()))
                .ok_or(ContractError::Capacity)?)?;
            ledger.visits(6)?;
            let model = native_model(model_name)?;
            (
                format!("lifecycle:generated:{ordinal}"),
                SourceKindV2::GeneratedArtifact,
                None,
                Some(model),
                Some(uuid.ok_or(ContractError::Binding)?),
            )
        }
    };
    Ok(SourcePinV2 {
        artifact_key: key,
        kind,
        raw_sha256: hash,
        byte_length: u64::try_from(raw.len()).map_err(|_| ContractError::Capacity)?,
        schema_identity: schema,
        validation_profile: profile.to_string(),
        native_model: model,
        native_root_uuid: root_uuid,
    })
}
/// Map exactly the maintained six recognized native model names, without schema inference.
fn native_model(value: &str) -> Result<NativeModelV2, ContractError> {
    match value {
        "catalog" => Ok(NativeModelV2::Catalog),
        "profile" => Ok(NativeModelV2::Profile),
        "component-definition" => Ok(NativeModelV2::ComponentDefinition),
        "mapping-collection" => Ok(NativeModelV2::MappingCollection),
        "system-security-plan" => Ok(NativeModelV2::SystemSecurityPlan),
        "plan-of-action-and-milestones" => Ok(NativeModelV2::PlanOfActionAndMilestones),
        _ => Err(ContractError::Binding),
    }
}
/// Charge complete exact string operands before native/private comparisons.
fn compare(
    left: &str,
    right: &str,
    ledger: &mut ContractLedger,
) -> Result<std::cmp::Ordering, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(left.len().checked_add(right.len()).ok_or(ContractError::Capacity)?)?;
    Ok(left.cmp(right))
}

/// Build all native current fingerprints and identity changes from the real complete members.
fn observed_artifacts(
    ordered: &[&ArtifactFingerprint],
    members: &[LifecycleMember],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentArtifacts, ContractError> {
    ledger.checkpoint(control)?;
    if members.len() != ordered.len().checked_add(2).ok_or(ContractError::Capacity)? {
        return Err(ContractError::Binding);
    }
    ledger.derived(
        ordered
            .len()
            .checked_mul(std::mem::size_of::<NamedHash>() + std::mem::size_of::<String>())
            .ok_or(ContractError::Capacity)?,
    )?;
    let mut generated = Vec::new();
    let mut identity_changes = Vec::new();
    generated.try_reserve_exact(ordered.len()).map_err(|_| ledger.capacity())?;
    identity_changes.try_reserve_exact(ordered.len()).map_err(|_| ledger.capacity())?;
    for (expected, actual) in ordered.iter().zip(&members[2..]) {
        ledger.checkpoint(control)?;
        let model = actual.pin.native_model.ok_or(ContractError::Binding)?;
        let uuid = actual.pin.native_root_uuid.as_deref().ok_or(ContractError::Binding)?;
        let model_equal = match &expected.oscal_type {
            Some(value) => compare(value, model.as_str(), ledger)? == std::cmp::Ordering::Equal,
            None => false,
        };
        let uuid_equal = match &expected.root_uuid {
            Some(value) => compare(value, uuid, ledger)? == std::cmp::Ordering::Equal,
            None => false,
        };
        let copies = expected
            .path
            .len()
            .checked_add(actual.pin.raw_sha256.len())
            .and_then(|n| {
                n.checked_add(if model_equal && uuid_equal { 0 } else { expected.path.len() })
            })
            .ok_or(ContractError::Capacity)?;
        ledger.derived(copies)?;
        ledger.bytes(copies)?;
        if !model_equal || !uuid_equal {
            identity_changes.push(expected.path.clone());
        }
        generated
            .push(NamedHash { path: expected.path.clone(), sha256: actual.pin.raw_sha256.clone() });
    }
    let source = &members[1].pin.raw_sha256;
    ledger.derived(
        source
            .len()
            .checked_add(std::mem::size_of::<CurrentArtifacts>())
            .ok_or(ContractError::Capacity)?,
    )?;
    ledger.bytes(source.len())?;
    Ok(CurrentArtifacts {
        fingerprints: FingerprintSet {
            source_sha256: source.clone(),
            generated_artifacts: generated,
        },
        identity_changes,
    })
}
/// Require complete latest approval equality through the maintained neutral status projection.
fn current_status(
    status: &crate::lifecycle::status::StatusReport,
    current: &CurrentArtifacts,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    let current_work = fingerprint_work(&current.fingerprints, ledger, control)?;
    let projected_work = fingerprint_work(&status.current_fingerprints, ledger, control)?;
    let approved_work = match &status.approved_fingerprints {
        Some(value) => fingerprint_work(value, ledger, control)?,
        None => 0,
    };
    ledger.bytes(
        current_work
            .checked_mul(2)
            .and_then(|n| n.checked_add(projected_work))
            .and_then(|n| n.checked_add(approved_work))
            .ok_or(ContractError::Capacity)?,
    )?;
    ledger.bytes(
        status.derived_status.len().checked_add("approved".len()).ok_or(ContractError::Capacity)?,
    )?;
    if status.state != LifecycleState::Approved
        || status.derived_status != "approved"
        || status.as_of.is_some()
        || !status.blockers.is_empty()
        || !status.artifact_identity_changes.is_empty()
        || !current.identity_changes.is_empty()
        || status.current_fingerprints != current.fingerprints
        || status.approved_fingerprints.as_ref() != Some(&current.fingerprints)
    {
        return Err(ContractError::Binding);
    }
    ledger.checkpoint(control)
}

/// Measure every actual operand on both sides before native complete fingerprint equality.
fn fingerprint_work(
    value: &FingerprintSet,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    ledger.checkpoint(control)?;
    ledger
        .visits(value.generated_artifacts.len().checked_add(1).ok_or(ContractError::Capacity)?)?;
    let mut work = value.source_sha256.len();
    for row in &value.generated_artifacts {
        work = work
            .checked_add(row.path.len())
            .and_then(|n| n.checked_add(row.sha256.len()))
            .ok_or(ContractError::Capacity)?;
    }
    Ok(work)
}

/// Clone a full observed pin with every exact string/nullable field admitted before growth.
fn copy_pin(pin: &SourcePinV2, ledger: &mut ContractLedger) -> Result<SourcePinV2, ContractError> {
    let strings = pin
        .artifact_key
        .len()
        .checked_add(pin.raw_sha256.len())
        .and_then(|n| n.checked_add(pin.schema_identity.as_ref().map_or(0, String::len)))
        .and_then(|n| n.checked_add(pin.validation_profile.len()))
        .and_then(|n| n.checked_add(pin.native_root_uuid.as_ref().map_or(0, String::len)))
        .ok_or(ContractError::Capacity)?;
    ledger.bytes(strings)?;
    ledger.derived(strings)?;
    Ok(SourcePinV2 {
        artifact_key: pin.artifact_key.clone(),
        kind: pin.kind,
        raw_sha256: pin.raw_sha256.clone(),
        byte_length: pin.byte_length,
        schema_identity: pin.schema_identity.clone(),
        validation_profile: pin.validation_profile.clone(),
        native_model: pin.native_model,
        native_root_uuid: pin.native_root_uuid.clone(),
    })
}
/// Produce the complete public tuple roster in exact artifact-key order, retaining all repeats.
fn public_pins(
    members: &[LifecycleMember],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<SourcePinV2>, ContractError> {
    ledger.derived(
        members
            .len()
            .checked_mul(std::mem::size_of::<SourcePinV2>())
            .ok_or(ContractError::Capacity)?,
    )?;
    let mut pins: Vec<SourcePinV2> = Vec::new();
    pins.try_reserve_exact(members.len()).map_err(|_| ledger.capacity())?;
    let mut extent = 0_u64;
    for member in members {
        ledger.checkpoint(control)?;
        extent = extent
            .checked_add(member.pin.byte_length)
            .filter(|n| *n <= 50 * 1024 * 1024)
            .ok_or(ContractError::Capacity)?;
        let mut slot = 0;
        while slot < pins.len() {
            if compare(&member.pin.artifact_key, &pins[slot].artifact_key, ledger)?
                == std::cmp::Ordering::Less
            {
                break;
            }
            slot += 1;
        }
        ledger.visits(pins.len() - slot)?;
        ledger.bytes(
            (pins.len() - slot)
                .checked_mul(std::mem::size_of::<SourcePinV2>())
                .ok_or(ContractError::Capacity)?,
        )?;
        pins.insert(slot, copy_pin(&member.pin, ledger)?);
    }
    ledger.checkpoint(control)?;
    Ok(pins)
}

/// Genuine native/captured controls are prospective until Root executes this exact source.
#[cfg(test)]
#[path = "lifecycle_capture_tests.rs"]
mod tests;

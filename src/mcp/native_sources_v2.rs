//! Genuine native/source consumers of the purpose-qualified complete /2 raw owner.
//!
//! Private success retains the actual held originals and original caller ledger/control.
//! Native identity, complete lifecycle currentness and complete native requirements are
//! computed here; no caller Value, status, boolean, index vector or /1 scope issues them.
//! Bounded Catalog applicability input qualification uses the genuine captured domain helper.
//! Mapping/Profile/linkage activation, full index build/publication and unsupported server families remain absent.

use std::path::Path;

use super::capture_v2::{
    Admission, AdmittedValue, CaptureBuilderV2, CapturePurpose, Configuration, HeldInputsV2,
    MissingPolicy, Observation,
};
use super::declarations_v2::{
    self, Companion, DeclarationAdmission, InertBuildDeclarations, InertServerDeclarations,
    ProjectDeclarationV2, SearchSourceTuple, VisibilityV2,
};
use super::native_closure_v2::{self, ClosureFacts};
use super::native_inventory_v2;
use super::native_requirements_v2;
use super::native_work_v2::{self as work, MemberFacts};
use super::{NativeIdentity, ResourcePolicy, Role, policy_failure, valid_hash};
use crate::mapping::inventory::LoadedResource;
use crate::workspace::preparation::{Stage, WorkControl, WorkResult};

/// Closed actual purpose family decoded from the three fixed held configurations.
enum Selection {
    /// Genuine final read declaration; no offline intent can substitute for it.
    Server(AdmittedValue<InertServerDeclarations>),
    /// Genuine offline build intent; its complete tuple equality remains required.
    Offline(AdmittedValue<InertBuildDeclarations>),
}
impl Selection {
    /// Borrow the whole actual decoded discovery roster.
    fn project(&self) -> &ProjectDeclarationV2 {
        match self {
            Self::Server(data) => &data.value().project,
            Self::Offline(data) => &data.value().project,
        }
    }
    /// Borrow the complete actual selected visibility assertion.
    fn profile(&self) -> &VisibilityV2 {
        match self {
            Self::Server(data) => &data.value().profile,
            Self::Offline(data) => &data.value().profile,
        }
    }
    /// Observe the closed constructor family, never a caller completeness flag.
    fn purpose(&self) -> CapturePurpose {
        match self {
            Self::Server(_) => CapturePurpose::ServerRead,
            Self::Offline(_) => CapturePurpose::OfflineBuild,
        }
    }
}

/// Private complete facts plus the actual original owner; no constructor escapes this module.
pub(super) struct NativeSourcesV2<'control, C: WorkControl + ?Sized> {
    /// Same original roots/configs/raw present/absence/directory proofs through publication.
    held: HeldInputsV2<'control, C>,
    /// Actual complete configs decoded again on this owner before any native issuance.
    selection: Selection,
    /// One ordinary intrinsic observation for every base row in exact declared order.
    members: AdmittedValue<Vec<Option<AdmittedValue<MemberFacts>>>>,
    /// Genuine complete current tuples/status, independently derived per actual lifecycle.
    closures: AdmittedValue<Vec<Option<AdmittedValue<Option<ClosureFacts>>>>>,
    /// Genuine complete Catalog inventories, never a caller snapshot or approval substitute.
    inventories: AdmittedValue<Vec<Option<AdmittedValue<LoadedResource>>>>,
}

/// Offline-only successful input qualification; it is not an index or server capability.
pub(super) struct OfflineIndexBuildCapture<'control, C: WorkControl + ?Sized> {
    /// Same actual complete current originals retained until final output verification.
    sources: NativeSourcesV2<'control, C>,
    /// Complete native-issued canonical key/ID tuples, compared bijectively to intent.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "The complete admitted tuples and their joint byte ticket must remain alive through the offline final owner fence and fixed output; cfg controls inspect the full denominator."
        )
    )]
    tuples: AdmittedValue<Vec<SearchSourceTuple>>,
}

/// Fixed ordinary unavailability is distinct from typed original owner stops.
#[expect(
    clippy::large_enum_variant,
    reason = "Keep the genuine retained native owner inline; boxing would add an unadmitted allocation and alter its existing ticket/drop lifetime contract."
)]
pub(super) enum OfflineBuildGate<'control, C: WorkControl + ?Sized> {
    /// Actual native/source or missing required-family predicates did not qualify.
    Unavailable,
    /// Complete supported native/source/search-tuple closure genuinely succeeded.
    Validated(OfflineIndexBuildCapture<'control, C>),
}

/// Borrowed actual resource observation; private fields prevent supplied facts becoming a view.
pub(super) struct NativeResourceV2<'scope, 'control, C: WorkControl + ?Sized> {
    /// The genuine private complete owner remains borrowed by every view.
    sources: &'scope NativeSourcesV2<'control, C>,
    /// Actual decoded base declaration occurrence, never a caller-selected proof vector.
    index: usize,
}

/// Independently approved/current actual member; construction checks its full native closure.
pub(super) struct ApprovedNativeV2<'scope, 'control, C: WorkControl + ?Sized> {
    /// Actual admitted native/source occurrence from the complete owner.
    resource: NativeResourceV2<'scope, 'control, C>,
    /// Actual intrinsically checked complete source/generated/current status, privately issued.
    closure: &'scope ClosureFacts,
}

/// Capture/check a real offline intent without IO for malformed pins or root spellings.
pub(super) fn capture_index_build_intent<'control, C: WorkControl + ?Sized>(
    project: &Path,
    intent_root: &Path,
    expected_intent: &str,
    expected_profile: &str,
    control: &'control mut C,
) -> WorkResult<OfflineBuildGate<'control, C>> {
    if !arguments(project, intent_root, expected_intent, expected_profile) {
        return Ok(OfflineBuildGate::Unavailable);
    }
    let builder =
        CaptureBuilderV2::new(project, intent_root, CapturePurpose::OfflineBuild, control)?;
    let selection =
        decode_builder(&builder, CapturePurpose::OfflineBuild, expected_intent, expected_profile)?;
    let held = capture_declared(builder, &selection)?;
    let sources = prepare_sources(held, selection)?;
    if required_family_missing(&sources)? || !destination_absent(&sources)? {
        return Ok(OfflineBuildGate::Unavailable);
    }
    let Some(tuples) = native_requirements_v2::complete(&sources)? else {
        return Ok(OfflineBuildGate::Unavailable);
    };
    if !native_requirements_v2::matches_intent(&sources, tuples.value())? {
        return Ok(OfflineBuildGate::Unavailable);
    }
    sources.verify_inputs()?;
    let mut admission = sources.admission();
    work::fence(&mut admission, Stage::RetainPrepared)?;
    Ok(OfflineBuildGate::Validated(OfflineIndexBuildCapture { sources, tuples }))
}

/// Admit only canonical raw pins and exact absolute UTF-8 Normal-component root spellings.
/// No canonicalize, existence probe, arbitrary filename or family probing occurs here.
fn arguments(project: &Path, external: &Path, purpose: &str, profile: &str) -> bool {
    /// Check absolute UTF-8 root spelling without resolving or probing the filesystem.
    fn root(path: &Path) -> bool {
        path.is_absolute()
            && path.as_os_str().to_str().is_some()
            && crate::linkage::has_normalized_path_spelling(path)
            && !path.components().any(|part| {
                matches!(part, std::path::Component::CurDir | std::path::Component::ParentDir)
            })
    }
    valid_hash(purpose) && valid_hash(profile) && root(project) && root(external)
}

/// Decode the three actual fixed original configs using only the original owner adapter.
fn decode_builder<C: WorkControl + ?Sized>(
    builder: &CaptureBuilderV2<'_, C>,
    actual_constructor: CapturePurpose,
    expected_purpose: &str,
    expected_profile: &str,
) -> WorkResult<Selection> {
    let mut admission = builder.admission();
    let discovery = builder.configuration(Configuration::Discovery)?.ok_or_else(policy_failure)?;
    let profile = builder.configuration(Configuration::Visibility)?.ok_or_else(policy_failure)?;
    let purpose = builder.configuration(Configuration::Purpose)?.ok_or_else(policy_failure)?;
    let mut logical = 0;
    for raw in [discovery, profile, purpose] {
        let (_, bound) = work::raw_bound(raw, 1024 * 1024, &mut admission)?;
        logical = work::add(logical, bound, &mut admission)?;
    }
    match actual_constructor {
        CapturePurpose::OfflineBuild => {
            work::run(&mut admission, 1, Stage::PrepareDomain, |_| {
                if actual_constructor == CapturePurpose::OfflineBuild {
                    Ok(())
                } else {
                    Err(policy_failure())
                }
            })?;
            admission
                .retain(logical, |admission| {
                    declarations_v2::decode_build_intent(
                        discovery,
                        profile,
                        purpose,
                        expected_purpose,
                        expected_profile,
                        admission,
                    )
                })
                .map(Selection::Offline)
        }
        CapturePurpose::ServerRead => admission
            .retain(logical, |admission| {
                declarations_v2::decode_server_declarations(
                    discovery,
                    profile,
                    purpose,
                    expected_purpose,
                    expected_profile,
                    admission,
                )
            })
            .map(Selection::Server),
    }
}

/// Open every complete declared file before any declared content read; no inferred href opens.
fn capture_declared<'control, C: WorkControl + ?Sized>(
    mut builder: CaptureBuilderV2<'control, C>,
    selection: &Selection,
) -> WorkResult<HeldInputsV2<'control, C>> {
    for resource in &selection.project().resources {
        let limit =
            if resource.role == Role::LifecycleRecord { 2 * 1024 * 1024 } else { 10 * 1024 * 1024 };
        builder.open_original(
            &resource.key,
            Path::new(&resource.path),
            limit,
            MissingPolicy::Observe,
        )?;
    }
    for companion in &selection.project().companion_resources {
        let limit = match companion {
            Companion::LinkageManifest { .. } => 2 * 1024 * 1024,
            Companion::StaticReport { .. } | Companion::WorkspaceIndexMetadata { .. } => {
                1024 * 1024
            }
            _ => 10 * 1024 * 1024,
        };
        builder.open_original(
            companion.key(),
            Path::new(companion.path()),
            limit,
            MissingPolicy::Observe,
        )?;
    }
    // Linkage manifest-derived declaration/directory coupling is an explicitly absent producer.
    // Its presence refuses this bounded packet before the first declared content read.
    if selection
        .project()
        .companion_resources
        .iter()
        .any(|row| matches!(row, Companion::LinkageManifest { .. }))
    {
        return Err(policy_failure());
    }
    builder.read_remaining()?;
    builder.finish()
}

/// Compute every base observation/full lifecycle closure on the same complete held roster.
fn prepare_sources<C: WorkControl + ?Sized>(
    held: HeldInputsV2<'_, C>,
    selection: Selection,
) -> WorkResult<NativeSourcesV2<'_, C>> {
    let expected = selection.purpose();
    let mut admission = held.admission();
    work::run(&mut admission, 1, Stage::PrepareDomain, |_| {
        if held.purpose() == expected { Ok(()) } else { Err(policy_failure()) }
    })?;
    verify_roster(&held, selection.project())?;
    let count = selection.project().resources.len();
    let entry = std::mem::size_of::<Option<AdmittedValue<MemberFacts>>>();
    let slots = work::multiply(count, entry, &mut admission)?;
    let slots = work::add(
        slots,
        std::mem::size_of::<Vec<Option<AdmittedValue<MemberFacts>>>>(),
        &mut admission,
    )?;
    let members = admission.retain(slots, |admission| {
        let mut members = Vec::with_capacity(count);
        for resource in &selection.project().resources {
            admission.charge(1)?;
            let observed = held.original(&resource.key)?.ok_or_else(policy_failure)?;
            let data = match observed {
                Observation::Present { bytes, .. }
                    if resource.role.native_model().is_some()
                        || matches!(
                            resource.role,
                            Role::LifecycleRecord
                                | Role::ApplicabilityManifest
                                | Role::ApplicabilityReport
                        ) =>
                {
                    Some(work::prepare_member(bytes, resource.role, admission)?)
                }
                _ => None,
            };
            members.push(data);
        }
        Ok(members)
    })?;
    let closures = native_closure_v2::prepare(&held, selection.project(), members.value())?;
    let inventories = native_inventory_v2::prepare(&held, selection.project(), members.value())?;
    let sources = NativeSourcesV2 { held, selection, members, closures, inventories };
    sources.verify_inputs()?;
    Ok(sources)
}

/// Compare complete actual declared occurrence order/key/routes and the entire directory roster.
fn verify_roster<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    project: &ProjectDeclarationV2,
) -> WorkResult<()> {
    let mut admission = held.admission();
    let count =
        work::add(project.resources.len(), project.companion_resources.len(), &mut admission)?;
    admission.charge(count)?;
    let mut expected = project
        .resources
        .iter()
        .map(|row| (row.key.as_str(), row.path.as_str()))
        .chain(project.companion_resources.iter().map(|row| (row.key(), row.path())));
    held.visit_declared(&mut |member| {
        let (key, path) = expected.next().ok_or_else(policy_failure)?;
        let width = work::add(key.len(), path.len(), &mut admission)?;
        let native = member.route().as_os_str().to_str().ok_or_else(policy_failure)?;
        let width = work::add(
            width,
            work::add(member.key().len(), native.len(), &mut admission)?,
            &mut admission,
        )?;
        work::run(&mut admission, 1 + work::byte_work(width), Stage::PrepareDomain, |_| {
            if member.key() == key && member.route() == Path::new(path) {
                Ok(())
            } else {
                Err(policy_failure())
            }
        })
    })?;
    let result = if expected.next().is_some() { Err(policy_failure()) } else { Ok(()) };
    admission.phase(result, Stage::PrepareDomain)?;
    held.visit_directories(&mut |_| Err(policy_failure()))
}

/// Missing relevant domain activation refuses the WHOLE offline capability, including hidden
/// Mapping/App dependencies actually required by a visible native's complete lifecycle closure.
fn required_family_missing<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
) -> WorkResult<bool> {
    let mut admission = sources.admission();
    let result = (|| {
        // Invocation-local reuse contains only this genuine complete qualification result;
        // the actual borrowed native plans remain private to qualify_visible's one phase.
        let mut applicability_ready = None;
        for key in &sources.profile().visible_resource_keys {
            admission.charge(1)?;
            let Some(index) = sources.index(key)? else {
                return Ok(true);
            };
            let role = sources.policies()[index].role;
            if matches!(role, Role::ApplicabilityManifest | Role::ApplicabilityReport) {
                if applicability_ready.is_none() {
                    applicability_ready =
                        Some(super::native_applicability_v2::qualify_visible(sources)?);
                }
                if applicability_ready == Some(false) {
                    return Ok(true);
                }
                continue;
            }
            if role == Role::MappingCollection
                || (role.native_model().is_some()
                    && !matches!(role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact))
            {
                return Ok(true);
            }
            if matches!(role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact) {
                let Some(approved) = sources.approved_index(index)? else {
                    return Ok(true);
                };
                for member in std::iter::once(approved.closure.source)
                    .chain(approved.closure.generated.iter().copied())
                {
                    admission.charge(1)?;
                    let dependency = sources.policies()[member].role;
                    if matches!(
                        dependency,
                        Role::MappingCollection
                            | Role::ApplicabilityManifest
                            | Role::ApplicabilityReport
                    ) || (dependency.native_model().is_some()
                        && !matches!(
                            dependency,
                            Role::OscalCatalogArtifact | Role::OscalComponentArtifact
                        ))
                    {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(!sources.profile().evidence_metadata_keys.is_empty())
    })();
    admission.phase(result, Stage::PrepareDomain)
}

/// The one decoded offline `SearchIndex` destination must be an actual held absence.
/// A present/changed/empty destination is never relabeled absent or overwritten by this gate.
fn destination_absent<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
) -> WorkResult<bool> {
    let mut admission = sources.admission();
    let Some(intent) = sources.build_intent() else {
        return Err(policy_failure());
    };
    let key = intent.destination.key.as_str();
    let observed = sources.held.original(key)?.ok_or_else(policy_failure)?;
    work::run(&mut admission, 1, Stage::PrepareDomain, |_| {
        Ok(matches!(observed, Observation::Absent))
    })
}

impl<'control, C: WorkControl + ?Sized> NativeSourcesV2<'control, C> {
    /// Borrow every actual declared base occurrence, including hidden/unsupported observations.
    pub(super) fn policies(&self) -> &[ResourcePolicy] {
        &self.selection.project().resources
    }
    /// Borrow the exact actual selected visibility, never an inferred profile.
    pub(super) fn profile(&self) -> &VisibilityV2 {
        self.selection.profile()
    }
    /// Alias only the original retained admission/control, with no allowance or deadline renewal.
    pub(super) fn admission(&self) -> Admission<'control, C> {
        self.held.admission()
    }
    /// Borrow an actual same-owner original; this never reads another file.
    pub(super) fn observation(&self, index: usize) -> WorkResult<Observation<'_>> {
        let policy = self.policies().get(index).ok_or_else(policy_failure)?;
        self.held.original(&policy.key)?.ok_or_else(policy_failure)
    }
    /// Borrow actual full intrinsic data produced here, never supplied native facts.
    pub(super) fn facts(&self, index: usize) -> Option<&MemberFacts> {
        self.members.value().get(index)?.as_ref().map(AdmittedValue::value)
    }
    /// Find one complete known key only after admitting all actual comparison operands.
    pub(super) fn index(&self, key: &str) -> WorkResult<Option<usize>> {
        let mut admission = self.admission();
        let mut width = 0;
        for policy in self.policies() {
            width = work::add(
                width,
                work::add(key.len(), policy.key.len(), &mut admission)?,
                &mut admission,
            )?;
        }
        let amount = work::add(self.policies().len(), work::byte_work(width), &mut admission)?;
        work::run(&mut admission, amount, Stage::PrepareDomain, |_| {
            Ok(self.policies().iter().position(|row| row.key == key))
        })
    }
    /// Borrow only an actual present/raw-pin/native-identity admitted base member.
    pub(super) fn resource(
        &self,
        key: &str,
    ) -> WorkResult<Option<NativeResourceV2<'_, 'control, C>>> {
        let Some(index) = self.index(key)? else {
            return Ok(None);
        };
        self.admitted(index)
    }
    /// Issue a view only after actual same-owner raw/native observations match the declaration.
    fn admitted(&self, index: usize) -> WorkResult<Option<NativeResourceV2<'_, 'control, C>>> {
        let policy = self.policies().get(index).ok_or_else(policy_failure)?;
        let Observation::Present { raw_sha256, .. } = self.observation(index)? else {
            return Ok(None);
        };
        let mut admission = self.admission();
        let actual = self.facts(index).and_then(|facts| facts.identity.as_ref());
        let mut width = work::add(raw_sha256.len(), policy.expected_sha256.len(), &mut admission)?;
        for identity in actual.into_iter().chain(policy.native_identity.as_ref()) {
            for text in [
                &identity.model,
                &identity.root_id,
                &identity.document_version,
                &identity.oscal_version,
            ] {
                width = work::add(width, text.len(), &mut admission)?;
            }
        }
        work::run(&mut admission, 1 + work::byte_work(width), Stage::ValidateResource, |_| {
            if raw_sha256 != policy.expected_sha256 {
                return Ok(None);
            }
            if policy.role.native_model().is_some() && actual != policy.native_identity.as_ref() {
                return Ok(None);
            }
            Ok(Some(NativeResourceV2 { sources: self, index }))
        })
    }
    /// Issue actual approved/current views only from the complete private closure/status facts.
    pub(super) fn approved(
        &self,
        key: &str,
    ) -> WorkResult<Option<ApprovedNativeV2<'_, 'control, C>>> {
        let Some(index) = self.index(key)? else {
            return Ok(None);
        };
        self.approved_index(index)
    }
    /// Check independent lifecycle membership and the complete latest neutral approval predicate.
    fn approved_index(
        &self,
        index: usize,
    ) -> WorkResult<Option<ApprovedNativeV2<'_, 'control, C>>> {
        let Some(resource) = self.admitted(index)? else {
            return Ok(None);
        };
        if !resource.role().status_supported() {
            return Ok(None);
        }
        let Some(key) = resource.lifecycle_key() else {
            return Ok(None);
        };
        let Some(lifecycle) = self.index(key)? else {
            return Ok(None);
        };
        let Some(closure) = self
            .closures
            .value()
            .get(lifecycle)
            .and_then(Option::as_ref)
            .map(AdmittedValue::value)
            .and_then(Option::as_ref)
        else {
            return Ok(None);
        };
        let mut admission = self.admission();
        let mut rows = work::add(closure.generated.len(), 1, &mut admission)?;
        let mut width = 0;
        for fingerprints in std::iter::once(&closure.current.fingerprints)
            .chain(closure.neutral.approved_fingerprints.as_ref())
        {
            width = work::add(width, fingerprints.source_sha256.len(), &mut admission)?;
            rows = work::add(rows, fingerprints.generated_artifacts.len(), &mut admission)?;
            for hash in &fingerprints.generated_artifacts {
                admission.charge(1)?;
                let strings = work::add(hash.path.len(), hash.sha256.len(), &mut admission)?;
                width = work::add(width, strings, &mut admission)?;
            }
        }
        let amount = work::add(rows, work::byte_work(width), &mut admission)?;
        work::run(&mut admission, amount, Stage::ValidateResource, |_| {
            if closure.current_approved()
                && (closure.source == index || closure.generated.contains(&index))
            {
                Ok(Some(ApprovedNativeV2 { resource, closure }))
            } else {
                Ok(None)
            }
        })
    }
    /// Resolve actual native-relative trace only to this native's exact full current source tuple.
    pub(super) fn declared_source(
        &self,
        native: usize,
        source_file: &str,
    ) -> WorkResult<Option<NativeResourceV2<'_, 'control, C>>> {
        let Some(approved) = self.approved_index(native)? else {
            return Ok(None);
        };
        let mut admission = self.admission();
        let width = work::add(approved.resource.path().len(), source_file.len(), &mut admission)?;
        let strings = work::multiply(width, 2, &mut admission)?;
        let payload = work::add(strings, 512, &mut admission)?;
        let result = admission.retain(payload, |admission| {
            work::run(admission, 1 + work::byte_work(width), Stage::PrepareDomain, |_| {
                Ok(super::resolve_record_path(approved.resource.path(), source_file))
            })
        })?;
        let Some(path) = result.value().as_deref() else {
            return Ok(None);
        };
        let source = &self.policies()[approved.closure.source];
        let amount = 1 + work::byte_work(path.len() + source.path.len());
        work::run(&mut admission, amount, Stage::PrepareDomain, |_| {
            if path == source.path
                && matches!(source.role, Role::PolicySource | Role::LifecycleSource)
                && source.lifecycle_key == self.policies()[native].lifecycle_key
            {
                Ok(Some(NativeResourceV2 { sources: self, index: approved.closure.source }))
            } else {
                Ok(None)
            }
        })
    }
    /// Borrow actual complete native inventory only while its original scope/ticket are retained.
    pub(super) fn inventory(&self, index: usize) -> Option<&LoadedResource> {
        self.inventories.value().get(index)?.as_ref().map(AdmittedValue::value)
    }
    /// Borrow only the actual offline intent, never a synthesized final decision.
    pub(super) fn build_intent(&self) -> Option<&declarations_v2::IndexBuildIntent> {
        match &self.selection {
            Selection::Offline(data) => Some(&data.value().intent),
            Selection::Server(_) => None,
        }
    }
    /// Verify the complete actual original owner, including hidden observations and destination absence.
    pub(super) fn verify_inputs(&self) -> WorkResult<()> {
        self.held.verify_complete()
    }
    /// Original concrete caller access for narrow sequential Root publication; no owner reentry.
    pub(super) fn with_control<T>(&self, use_control: impl FnOnce(&mut C) -> T) -> T {
        self.held.with_control(use_control)
    }
}

impl<C: WorkControl + ?Sized> NativeResourceV2<'_, '_, C> {
    /// Exact actual private resource occurrence.
    pub(super) fn key(&self) -> &str {
        &self.sources.policies()[self.index].key
    }
    /// Closed actual declared base role, not a companion kind.
    pub(super) fn role(&self) -> Role {
        self.sources.policies()[self.index].role
    }
    /// Exact private declared contained route; callers must not reopen it.
    pub(super) fn path(&self) -> &str {
        &self.sources.policies()[self.index].path
    }
    /// Actual selected inert citation token, not native private prose.
    pub(super) fn citation_label(&self) -> &str {
        &self.sources.policies()[self.index].citation_label
    }
    /// Actual same-owner raw original bytes; typed absence never becomes empty data.
    pub(super) fn bytes(&self) -> WorkResult<&[u8]> {
        match self.sources.observation(self.index)? {
            Observation::Present { bytes, .. } => Ok(bytes),
            Observation::Absent => Err(policy_failure()),
        }
    }
    /// Actual raw original digest from the same held generation.
    pub(super) fn raw_sha256(&self) -> WorkResult<&str> {
        match self.sources.observation(self.index)? {
            Observation::Present { raw_sha256, .. } => Ok(raw_sha256),
            Observation::Absent => Err(policy_failure()),
        }
    }
    /// Actual full native tree only after maintained intrinsic admission and declaration identity match.
    pub(super) fn native_value(&self) -> Option<&serde_json::Value> {
        self.role().native_model()?;
        self.sources.facts(self.index)?.value.as_ref()
    }
    /// Complete actual native identity with original UUID/version spelling.
    pub(super) fn native_identity(&self) -> Option<&NativeIdentity> {
        self.sources.facts(self.index)?.identity.as_ref()
    }
    /// Exact declared independent lifecycle binding.
    pub(super) fn lifecycle_key(&self) -> Option<&str> {
        self.sources.policies()[self.index].lifecycle_key.as_deref()
    }
}

impl<'scope, 'control, C: WorkControl + ?Sized> ApprovedNativeV2<'scope, 'control, C> {
    /// Borrow actual native/source view under its complete current approval.
    pub(super) fn resource(&self) -> &NativeResourceV2<'scope, 'control, C> {
        &self.resource
    }
    /// Borrow the actual intrinsically validated full record selected by this closure.
    #[cfg(test)]
    pub(super) fn record(&self) -> Option<&crate::lifecycle::record::LifecycleRecord> {
        self.resource
            .sources
            .facts(self.closure.lifecycle)
            .and_then(|facts| facts.record.as_ref())
            .map(crate::lifecycle::record::ParsedLifecycleRecord::record)
    }
    /// Borrow actual complete neutral status, preserving private provenance internally.
    pub(super) fn neutral_status(&self) -> &crate::lifecycle::status::StatusReport {
        &self.closure.neutral
    }
}

impl<'control, C: WorkControl + ?Sized> OfflineIndexBuildCapture<'control, C> {
    /// Complete canonical native tuple set; no filter/page selection precedes intent equality.
    #[cfg(test)]
    pub(super) fn search_sources(&self) -> &[SearchSourceTuple] {
        self.tuples.value()
    }
    /// Original shared admission remains live through Root's fixed final output.
    pub(super) fn admission(&self) -> Admission<'control, C> {
        self.sources.admission()
    }
    /// Verify both roots/all configs/full originals/directory proofs/destination absence again.
    pub(super) fn verify_inputs(&self) -> WorkResult<()> {
        self.sources.verify_inputs()
    }
    /// Sequential Root publication uses the original concrete controller without reentry.
    pub(super) fn with_control<T>(&self, use_control: impl FnOnce(&mut C) -> T) -> T {
        self.sources.with_control(use_control)
    }
}

/// Genuine retained `ServerRead` owner; only the complete native factory can construct it.
pub(super) struct ServerNativeScopeV2<'control, C: WorkControl + ?Sized> {
    /// Complete physical source/config/absence owner and intrinsic native facts.
    sources: NativeSourcesV2<'control, C>,
    /// Complete canonical native source tuples, not a selected query subset.
    tuples: AdmittedValue<Vec<SearchSourceTuple>>,
    /// Actual physical original-generation framing from that same retained owner.
    generation: AdmittedValue<String>,
}

/// Ordinary factory refusal has no success owner; typed first stops remain errors.
#[expect(
    clippy::large_enum_variant,
    reason = "Keep the genuine retained native owner inline; boxing would add an unadmitted allocation and alter its existing ticket/drop lifetime contract."
)]
pub(super) enum ServerReadGateV2<'control, C: WorkControl + ?Sized> {
    /// Complete supported read predicates did not qualify.
    Unavailable,
    /// Complete genuine native/source owner remains held through publication.
    Validated(ServerNativeScopeV2<'control, C>),
}

/// Capture the exact /2 final-read family before any /1 capture opens.
pub(super) fn capture_server_decision<'control, C: WorkControl + ?Sized>(
    project: &Path,
    decision_root: &Path,
    expected_decision: &str,
    expected_profile: &str,
    control: &'control mut C,
) -> WorkResult<ServerReadGateV2<'control, C>> {
    if !arguments(project, decision_root, expected_decision, expected_profile) {
        return Ok(ServerReadGateV2::Unavailable);
    }
    let builder =
        CaptureBuilderV2::new(project, decision_root, CapturePurpose::ServerRead, control)?;
    let selection =
        decode_builder(&builder, CapturePurpose::ServerRead, expected_decision, expected_profile)?;
    let mut admission = builder.admission();
    let supported = work::run(&mut admission, 1, Stage::PrepareDomain, |_| {
        let p = selection.profile();
        Ok(p.noncurrent_metadata_keys.is_empty()
            && p.source_text_keys.is_empty()
            && p.schedule_keys.is_empty()
            && p.evidence_metadata_keys.is_empty()
            && p.static_report_keys.is_empty()
            && p.search_index_key.is_none()
            && p.resource_families.is_empty()
            && p.source_text_mode == "omit"
            && p.schedule_mode == "omit"
            && p.as_of.is_none())
    })?;
    if !supported {
        return Ok(ServerReadGateV2::Unavailable);
    }
    let held = capture_declared(builder, &selection)?;
    let sources = prepare_sources(held, selection)?;
    let result = (|| {
        if required_family_missing(&sources)? {
            return Ok(None);
        }
        native_requirements_v2::complete(&sources)
    })();
    // Ordinary complete-family refusal is fenced while the actual owner still exists.
    sources.verify_inputs()?;
    let tuples = sources.admission().phase(result, Stage::RetainPrepared)?;
    let Some(tuples) = tuples else {
        return Ok(ServerReadGateV2::Unavailable);
    };
    let generation = sources.held.generation_digest()?;
    sources.verify_inputs()?;
    work::fence(&mut sources.admission(), Stage::RetainPrepared)?;
    Ok(ServerReadGateV2::Validated(ServerNativeScopeV2 { sources, tuples, generation }))
}

impl<'control, C: WorkControl + ?Sized> ServerNativeScopeV2<'control, C> {
    /// Borrow only this genuine owner's complete intrinsic data.
    pub(super) fn sources(&self) -> &NativeSourcesV2<'control, C> {
        &self.sources
    }
    /// Borrow the complete canonical native requirement tuple denominator.
    pub(super) fn search_sources(&self) -> &[SearchSourceTuple] {
        self.tuples.value()
    }
    /// Borrow physical generation data; this digest does not replace final verification.
    pub(super) fn scope_generation(&self) -> &str {
        self.generation.value()
    }
    /// Alias only the original controller and its unchanged shared work/byte ledger.
    pub(super) fn admission(&self) -> Admission<'control, C> {
        self.sources.admission()
    }
    /// Verify all hidden/present/absent/config/root originals without recapture.
    pub(super) fn verify_inputs(&self) -> WorkResult<()> {
        self.sources.verify_inputs()
    }
    /// Sequential exact-controller access; callbacks may not reenter owner admission.
    pub(super) fn with_control<T>(&self, use_control: impl FnOnce(&mut C) -> T) -> T {
        self.sources.with_control(use_control)
    }
}

/// Genuine maintained-native/file controls; no acceptance credit is implied.
#[cfg(test)]
#[path = "native_sources_v2_tests.rs"]
mod tests;

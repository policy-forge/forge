//! Genuine complete current authoring-plan receiver, preserving N/S/U separately.
//! The owner is minted only by actual native preparation, full saved-plan comparison,
//! whole registration sealing and maintained physical verification. It implies no quorum,
//! authenticated identity, native promotion, project mutation or publication authority.
use super::capture::authoring::{AuthoringPurpose, RegisteredAuthoringOriginals};
use super::capture::{HeldReviewInputs, ReviewCapture};
use super::decode::{ContractError, ContractLedger};
use super::decode_v2::{checkpoint, phase};
use super::hash_v3::NativeProvenance;
use super::wire_v3::{NativeModelV3, SourceKindV3, SourcePinV3};
use super::{authoring_locator as locator, authoring_work as native};
use crate::authoring::borrowed::{BorrowedAuthoringInputs, PreparedAuthoringFacts};
use crate::evidence_capture::CaptureLease;
use crate::hashing::sha256_hex;
use crate::workspace::preparation::WorkControl;
use std::path::{Path, PathBuf};
use std::rc::Rc;
/// Full private tuple/membership and observed public projection helpers.
#[path = "authoring_capture_native.rs"]
mod projection;

/// One actual successful receiver registration, never a caller-authored lease selector.
pub(super) struct AuthoringMember {
    /// Actual distinct Source Entry index on the original session.
    index: usize,
    /// Genuine original allocation, retained through every final fence.
    lease: CaptureLease,
    /// Actual complete native read purpose; saved-plan is explicitly outside N.
    purpose: AuthoringPurpose,
    /// Actual review-root-relative physical capture route.
    path: PathBuf,
    /// Exact project-root-relative native label, absent only for saved-plan.
    native_path: Option<String>,
    /// Exact legacy native role, including clause keys; absent only for saved-plan.
    native_role: Option<String>,
    /// Exact complete original raw identity, not a reserialization.
    raw_sha256: String,
}
impl AuthoringMember {
    /// Borrow actual Source registration identity for private whole-owner verification.
    pub(super) fn index(&self) -> usize {
        self.index
    }
    /// Borrow exact actual allocation; hashes cannot repair an owner mismatch.
    pub(super) fn lease(&self) -> &CaptureLease {
        &self.lease
    }
    /// Borrow complete actual purpose, preserving declaration order.
    pub(super) fn purpose(&self) -> AuthoringPurpose {
        self.purpose
    }
    /// Borrow the real physical capture route, never an exported source path.
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}
/// Genuine pending native result, before ALL selected operation inputs have been registered.
pub(crate) struct PendingAuthoringCohort {
    /// Actual single locator Entry.
    locator_index: usize,
    /// Actual locator lease, retained separately from N/S.
    locator_lease: CaptureLease,
    /// Entire native loaded input facts and regenerated full plan.
    facts: PreparedAuthoringFacts,
    /// Complete actual N read order followed by the distinct saved-plan S occurrence.
    members: Vec<AuthoringMember>,
    /// Complete eight-field public tuples, sorted by artifact key bytes.
    pins: Vec<SourcePinV3>,
}
impl PendingAuthoringCohort {
    /// Borrow actual locator identity for the private sealer only.
    pub(super) fn locator(&self) -> (usize, &CaptureLease) {
        (self.locator_index, &self.locator_lease)
    }
    /// Borrow the whole actual S roster; no external list can construct this pending type.
    pub(super) fn members(&self) -> &[AuthoringMember] {
        &self.members
    }
    /// Consume this actual capture after the closed operation's full U registration.
    /// No caller Rc/index list/fingerprint/completeness Boolean can issue the owner.
    pub(crate) fn finish_and_seal(
        self,
        capture: ReviewCapture,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<CurrentAuthoringPlanClosure, ContractError> {
        phase(ledger, control, |ledger, control| {
            let extent = std::mem::size_of::<HeldReviewInputs>()
                .checked_add(2 * std::mem::size_of::<usize>())
                .ok_or_else(|| ledger.capacity())?;
            ledger.derived(extent)?;
            let held = Rc::new(capture.finish());
            let membership = held.verify_authoring_cohort(&self, ledger, control);
            checkpoint(ledger, control)?;
            membership?;
            let physical = held.verify_inputs(ledger, control);
            checkpoint(ledger, control)?;
            physical?;
            ledger.derived(std::mem::size_of::<CurrentAuthoringPlanClosure>())?;
            Ok(CurrentAuthoringPlanClosure {
                facts: self.facts,
                members: self.members,
                pins: self.pins,
                locator_lease: self.locator_lease,
                held,
            })
        })
    }
}
/// Current full-native plan data retaining the genuine complete physical owner.
/// Private fields and receiver-only initialization exclude arbitrary success factories.
pub(crate) struct CurrentAuthoringPlanClosure {
    /// Full native plan and private loaded project remain inside the owner.
    facts: PreparedAuthoringFacts,
    /// All native and saved-plan originals stay alive for later Queue/currentness binding.
    members: Vec<AuthoringMember>,
    /// Complete path-free observed tuple projection.
    pins: Vec<SourcePinV3>,
    /// Review-only locator, excluded from native plan provenance.
    locator_lease: CaptureLease,
    /// Actual capture.finish owner, never a supplied Rc or detachable proof flag.
    held: Rc<HeldReviewInputs>,
}
impl CurrentAuthoringPlanClosure {
    /// Borrow full genuine native facts, with no public private-prose projection.
    pub(crate) fn facts(&self) -> &PreparedAuthoringFacts {
        &self.facts
    }
    /// Borrow the complete exact observed public tuples, never a source subset.
    pub(crate) fn source_pins(&self) -> &[SourcePinV3] {
        &self.pins
    }
    /// Borrow all genuinely correlated N tuples in the maintained `PlanProvenance` order.
    /// Actual read order stays private in members; this ordering is exactly hash profile2.
    pub(crate) fn native_provenance(&self) -> impl ExactSizeIterator<Item = NativeProvenance<'_>> {
        self.facts.plan.provenance.inputs.iter().map(|row| NativeProvenance {
            role: &row.role,
            path: &row.path,
            raw_sha256: &row.sha256,
            byte_length: row.byte_length,
        })
    }
    /// Borrow the whole actual locator occurrence for Root's separately admitted generation hash.
    /// This lease is held in the same U proof and never becomes native plan provenance.
    pub(crate) fn locator_raw(&self) -> &[u8] {
        self.locator_lease.bytes()
    }
    /// Borrow only internally issued complete operation registrations, preserving duplicates.
    pub(crate) fn originals(&self) -> Result<RegisteredAuthoringOriginals<'_>, ContractError> {
        self.held.authoring_originals()
    }
    /// Recheck all actual physical inputs under the same original invocation work owner.
    pub(crate) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        phase(ledger, control, |ledger, control| {
            ledger.visits(1)?;
            let _ = self.locator_lease.bytes();
            for row in &self.members {
                checkpoint(ledger, control)?;
                ledger.visits(1)?;
                if !self.held.same_original(row.index, &row.lease)? {
                    return Err(ContractError::Binding);
                }
            }
            let ordinary = self.held.verify_inputs(ledger, control);
            checkpoint(ledger, control)?;
            ordinary
        })
    }
}
/// Read exact native declarations, capture all N plus distinct S, and compare the complete saved plan.
/// Every ordinary native/IO/shape result is postfenced with the original ledger/control.
pub(crate) fn read_pending(
    capture: &mut ReviewCapture,
    locator_path: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingAuthoringCohort, ContractError> {
    phase(ledger, control, |ledger, control| read_inner(capture, locator_path, ledger, control))
}
/// Genuine receiver chronology; detached DTOs cannot select a subset or supply currentness.
fn read_inner(
    capture: &mut ReviewCapture,
    locator_path: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingAuthoringCohort, ContractError> {
    let locator_index = capture.required_authoring_locator(locator_path, ledger, control)?;
    ledger.derived(std::mem::size_of::<CaptureLease>())?;
    let locator_lease = capture.lease(locator_index)?;
    let declaration = locator::decode(locator_lease.bytes(), ledger, control)?;
    let project_route = Path::new(&declaration.project.path);
    let parent = project_route.parent().ok_or(ContractError::Invalid)?;
    let filename = project_route.file_name().ok_or(ContractError::Invalid)?;
    let native_filename = Path::new(filename);
    let mut members = Vec::new();
    projection::capture(
        capture,
        projection::SourceOccurrence {
            path: project_route,
            native_path: Some(native_filename),
            purpose: AuthoringPurpose::Project,
            native_role: Some("author-project"),
            maximum: 2 * 1024 * 1024,
        },
        &mut members,
        ledger,
        control,
    )?;
    let project = native::discover(members[0].lease.bytes(), ledger, control)?;
    capture_project_roots(capture, parent, &project, &mut members, ledger, control)?;
    let has_resolved =
        capture_framework_inputs(capture, parent, &project, &mut members, ledger, control)?;
    capture_clauses(capture, parent, &project, &mut members, ledger, control)?;
    let plan_route = Path::new(&declaration.plan.path);
    projection::capture(
        capture,
        projection::SourceOccurrence {
            path: plan_route,
            native_path: None,
            purpose: AuthoringPurpose::StoredPlan,
            native_role: None,
            maximum: 10 * 1024 * 1024,
        },
        &mut members,
        ledger,
        control,
    )?;
    let mappings = projection::raws(&members, true, ledger, control)?;
    let clauses = projection::raws(&members, false, ledger, control)?;
    let resolved = if has_resolved {
        Some(projection::raw(&members, AuthoringPurpose::Resolved, ledger, control)?)
    } else {
        None
    };
    let inputs = BorrowedAuthoringInputs {
        project_filename: native_filename,
        project: projection::raw(&members, AuthoringPurpose::Project, ledger, control)?,
        pack: projection::raw(&members, AuthoringPurpose::Pack, ledger, control)?,
        gap_report: projection::raw(&members, AuthoringPurpose::GapReport, ledger, control)?,
        applicability_manifest: projection::raw(
            &members,
            AuthoringPurpose::Applicability,
            ledger,
            control,
        )?,
        framework: projection::raw(&members, AuthoringPurpose::Framework, ledger, control)?,
        resolved_catalog: resolved,
        mappings: &mappings,
        clauses: &clauses,
    };
    let facts = native::prepare(inputs, ledger, control)?;
    native::stored(
        projection::raw(&members, AuthoringPurpose::StoredPlan, ledger, control)?,
        &facts.plan,
        ledger,
        control,
    )?;
    projection::provenance(&members, &facts, ledger, control)?;
    let pins = projection::pins(&members, &facts, ledger, control)?;
    ledger.derived(std::mem::size_of::<PendingAuthoringCohort>())?;
    Ok(PendingAuthoringCohort { locator_index, locator_lease, facts, members, pins })
}

/// Capture every full project-declared pack/report/applicability root in original order.
fn capture_project_roots(
    capture: &mut ReviewCapture,
    parent: &Path,
    project: &crate::authoring::manifest::AuthorProject,
    members: &mut Vec<AuthoringMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for (path, purpose, role, maximum) in [
        (&project.authoring_pack.path, AuthoringPurpose::Pack, "authoring-pack", 2 * 1024 * 1024),
        (&project.gap_report.path, AuthoringPurpose::GapReport, "gap-report", 10 * 1024 * 1024),
        (
            &project.applicability_manifest.path,
            AuthoringPurpose::Applicability,
            "applicability-manifest",
            10 * 1024 * 1024,
        ),
    ] {
        let route = projection::join(parent, path, ledger, control)?;
        projection::capture(
            capture,
            projection::SourceOccurrence {
                path: &route,
                native_path: Some(path),
                purpose,
                native_role: Some(role),
                maximum,
            },
            members,
            ledger,
            control,
        )?;
    }
    Ok(())
}

/// Capture framework, explicit resolved companion and every Mapping occurrence in order.
fn capture_framework_inputs(
    capture: &mut ReviewCapture,
    parent: &Path,
    project: &crate::authoring::manifest::AuthorProject,
    members: &mut Vec<AuthoringMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let app = super::legacy_work::discover_applicability_manifest(
        projection::raw(members, AuthoringPurpose::Applicability, ledger, control)?,
        ledger,
        control,
    )
    .map_err(super::legacy_work::BorrowedPreparationError::into_contract)?;
    let app_base = project.applicability_manifest.path.parent().unwrap_or_else(|| Path::new(""));
    let framework = native::route(app_base, &app.framework.artifact, ledger, control)?;
    let route = projection::join(parent, &framework, ledger, control)?;
    projection::capture(
        capture,
        projection::SourceOccurrence {
            path: &route,
            native_path: Some(&framework),
            purpose: AuthoringPurpose::Framework,
            native_role: Some("framework"),
            maximum: 10 * 1024 * 1024,
        },
        members,
        ledger,
        control,
    )?;
    if let Some(relative) = &app.framework.resolved_catalog {
        let resolved = native::route(app_base, relative, ledger, control)?;
        let route = projection::join(parent, &resolved, ledger, control)?;
        projection::capture(
            capture,
            projection::SourceOccurrence {
                path: &route,
                native_path: Some(&resolved),
                purpose: AuthoringPurpose::Resolved,
                native_role: Some("resolved-catalog"),
                maximum: 10 * 1024 * 1024,
            },
            members,
            ledger,
            control,
        )?;
    }
    for (ordinal, relative) in app.mapping_collections.iter().enumerate() {
        checkpoint(ledger, control)?;
        let path = native::route(app_base, relative, ledger, control)?;
        let route = projection::join(parent, &path, ledger, control)?;
        ledger.derived(64)?;
        ledger.visits(1)?;
        let role = format!("mapping-collection-{ordinal}");
        projection::capture(
            capture,
            projection::SourceOccurrence {
                path: &route,
                native_path: Some(&path),
                purpose: AuthoringPurpose::Mapping(ordinal),
                native_role: Some(&role),
                maximum: 10 * 1024 * 1024,
            },
            members,
            ledger,
            control,
        )?;
    }
    Ok(app.framework.resolved_catalog.is_some())
}

/// Capture every full human-clause original after framework dependencies, without filtering.
fn capture_clauses(
    capture: &mut ReviewCapture,
    parent: &Path,
    project: &crate::authoring::manifest::AuthorProject,
    members: &mut Vec<AuthoringMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for (ordinal, clause) in project.human_clauses.iter().enumerate() {
        checkpoint(ledger, control)?;
        let route = projection::join(parent, &clause.source.path, ledger, control)?;
        let extent = clause.key.len().checked_add(32).ok_or_else(|| ledger.capacity())?;
        ledger.derived(extent)?;
        ledger.bytes(clause.key.len())?;
        let role = format!("human-clause-{}", clause.key);
        projection::capture(
            capture,
            projection::SourceOccurrence {
                path: &route,
                native_path: Some(&clause.source.path),
                purpose: AuthoringPurpose::Clause(ordinal),
                native_role: Some(&role),
                maximum: 1024 * 1024,
            },
            members,
            ledger,
            control,
        )?;
    }
    Ok(())
}

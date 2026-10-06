//! Complete captured legacy Impact preparation from the actual private locator.
//!
//! This reader retains every declared actual original lease and computes complete
//! legacy native facts with empty filters on the existing ledger/control. Its private
//! pending constructor is reached only after the strict full stored-report oracle.
//! Pending data grants no approval/currentness, queue-key correlation, Source-union
//! or publication authority; the genuine same-held-owner union issuer is separate.
//! Historical policy hrefs, imports and Mapping producer manifests are never routes.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::applicability::manifest::ApplicabilityManifest;
use crate::evidence_capture::{CaptureLease, CaptureRole};
use crate::framework::analysis::legacy_borrowed::{
    ApplicabilityBytes, FrameworkBytes, ImpactInputs, LegacyImpactFacts,
};
use crate::framework::manifest::{FrameworkResource, ImpactManifest};
use crate::framework::model::ImpactFilters;
use crate::mapping::manifest::ResourceType;
use crate::workspace::preparation::WorkControl;

use super::capture::ReviewCapture;
use super::capture::supersession::{AuxiliaryPurpose, ImpactSourcePurpose};
use super::decode::{ContractError, ContractLedger};
use super::{impact_report, legacy_work, mapping_capture as shared};

/// Exact corrected private locator marker, without public or native authority.
const LOCATOR_SCHEMA: &str = "forge.review-queue-impact-locator/1";
/// Existing individual actual Auxiliary original ceiling.
const MAX_LOCATOR_BYTES: usize = 1024 * 1024;
/// Complete exact corrected locator fields, including explicit nullable companions.
const LOCATOR_FIELDS: [&str; 7] = [
    "schema_version",
    "manifest_path",
    "report_path",
    "old_framework_source_key",
    "new_framework_source_key",
    "old_resolved_source_key",
    "new_resolved_source_key",
];

/// Closed inert route declaration decoded only from the registered actual locator.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Locator {
    /// Corrected private marker; it carries no queue/source/currentness proof.
    schema_version: String,
    /// Explicit root-relative native Impact manifest route.
    manifest_path: PathBuf,
    /// Explicit root-relative complete stored current-report route.
    report_path: PathBuf,
    /// Inert old queue framework source key; final tuple correlation is separate.
    old_framework_source_key: String,
    /// Inert new queue framework source key; final tuple correlation is separate.
    new_framework_source_key: String,
    /// Required explicit nullable old Profile companion key; never a route.
    old_resolved_source_key: Option<String>,
    /// Required explicit nullable new Profile companion key; never a route.
    new_resolved_source_key: Option<String>,
}

/// Borrowed declared Queue source keys; this plain view grants no native tuple binding.
pub(super) struct ImpactLocatorFields<'a> {
    /// Inert old Queue framework key, requiring separate native tuple correlation.
    pub(super) old_framework: &'a str,
    /// Inert new Queue framework key, requiring separate native tuple correlation.
    pub(super) new_framework: &'a str,
    /// Required explicit nullable old Profile companion declaration.
    pub(super) old_resolved: Option<&'a str>,
    /// Required explicit nullable new Profile companion declaration.
    pub(super) new_resolved: Option<&'a str>,
}

/// One real successful native registration; external callers cannot construct a member.
pub(super) struct ImpactMember {
    /// Exact actual capture registration index, never supplied by a caller.
    index: usize,
    /// Actual retained allocation owner, with no raw byte duplication.
    lease: CaptureLease,
    /// Exact maintained declared-read purpose, preserving complete repeats and order.
    purpose: ImpactSourcePurpose,
}

impl ImpactMember {
    /// Borrow the actual registered index for the genuine complete union check.
    pub(super) fn index(&self) -> usize {
        self.index
    }
    /// Borrow the retained actual original owner, never reconstruct it from a hash.
    pub(super) fn lease(&self) -> &CaptureLease {
        &self.lease
    }
    /// Preserve the actual closed capture purpose for same-owner roster comparison.
    pub(super) fn purpose(&self) -> ImpactSourcePurpose {
        self.purpose
    }
    /// Borrow exact actual original bytes for separately admitted native identity extraction.
    pub(super) fn bytes(&self) -> &[u8] {
        self.lease.bytes()
    }
}

/// Genuine complete reader result awaiting exact same-owner union and native queue binding.
/// No public/serde/Clone/Debug constructor or caller-success flag is exposed.
pub(crate) struct PendingImpactCohort {
    /// Actual successfully registered Auxiliary `ImpactLocator` original.
    locator_index: usize,
    /// Actual locator allocation retained throughout native and final owner work.
    locator_lease: CaptureLease,
    /// Complete ordered actual Impact read occurrences, including compatible repeats.
    members: Vec<ImpactMember>,
    /// Complete ordinary native facts computed from those exact captured originals.
    facts: LegacyImpactFacts,
    /// Inert key declarations retained for separate actual queue/native tuple correlation.
    locator: Locator,
}

impl PendingImpactCohort {
    /// Borrow the genuine successful locator registration and its actual held lease.
    pub(super) fn locator(&self) -> (usize, &CaptureLease) {
        (self.locator_index, &self.locator_lease)
    }
    /// Borrow exact framework/companion key declarations without cloning private data.
    pub(super) fn locator_fields(&self) -> ImpactLocatorFields<'_> {
        ImpactLocatorFields {
            old_framework: &self.locator.old_framework_source_key,
            new_framework: &self.locator.new_framework_source_key,
            old_resolved: self.locator.old_resolved_source_key.as_deref(),
            new_resolved: self.locator.new_resolved_source_key.as_deref(),
        }
    }
    /// Borrow every complete actual read occurrence for the constructor-free union issuer.
    pub(super) fn members(&self) -> &[ImpactMember] {
        &self.members
    }
    /// Borrow complete private native data; this getter grants no currentness or vote authority.
    pub(super) fn facts(&self) -> &LegacyImpactFacts {
        &self.facts
    }
}

/// Actual private plan positions in the reader's complete ordered member list.
struct Plan {
    /// Actual complete Impact manifest member position.
    manifest: usize,
    /// Actual complete stored-current report member position.
    report: usize,
    /// Actual old framework and explicit companion member positions.
    old: FrameworkPlan,
    /// Actual new framework and explicit companion member positions.
    new: FrameworkPlan,
    /// Complete Impact Mapping member positions in native declaration order.
    mappings: Vec<usize>,
    /// Configured complete applicability read plan, using that manifest's own base.
    applicability: Option<ApplicabilityPlan>,
    /// Declared native successor original member position.
    successor: Option<usize>,
    /// Paired historical prior report member position.
    prior: Option<usize>,
    /// Paired native Impact disposition member position.
    dispositions: Option<usize>,
}

/// Genuine framework read positions selected from validated native declarations.
struct FrameworkPlan {
    /// Actual declared Catalog/Profile member position.
    artifact: usize,
    /// Actual explicitly declared Profile Catalog companion position.
    resolved: Option<usize>,
}

/// Complete own-base configured applicability read plan.
struct ApplicabilityPlan {
    /// Actual native applicability manifest member position.
    manifest: usize,
    /// Actual declared Catalog/Profile member position.
    framework: usize,
    /// Actual explicit Profile Catalog companion member position.
    resolved: Option<usize>,
    /// Complete own-manifest Mapping member positions, retaining declaration order.
    mappings: Vec<usize>,
}

/// Read the complete maintained Impact closure and compare its full stored-current report.
///
/// The actual registered `AuxiliaryPurpose::ImpactLocator` chooses explicit routes.
/// All remaining reads derive only from admitted maintained manifests, under the same
/// genuine `ReviewCapture` Source pool. A future issuer must bind every retained lease to
/// the same `Rc<HeldReviewInputs>`, exact complete source union and genuine new-native cohort.
///
/// # Errors
///
/// Fixed ordinary invalid/binding refusals preserve the first actual saved ledger stop.
/// No partial roster, native failure, stored mismatch or final control failure issues a result.
pub(crate) fn prepare(
    capture: &mut ReviewCapture,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PendingImpactCohort, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let locator_index =
            capture.supersession_auxiliary_original(AuxiliaryPurpose::ImpactLocator)?;
        if capture.role(locator_index)? != CaptureRole::ReviewPrivateConfig {
            return Err(ContractError::Binding);
        }
        ledger.derived(std::mem::size_of::<CaptureLease>())?;
        let locator_lease = capture.lease(locator_index)?;
        let locator = decode_locator(locator_lease.bytes(), ledger, control)?;
        let mut members = Vec::new();
        let manifest_slot = required(
            capture,
            &locator.manifest_path,
            ImpactSourcePurpose::Manifest,
            &mut members,
            ledger,
            control,
        )?;
        let manifest = legacy_work::discover_impact_manifest(
            original(&members, manifest_slot)?,
            ledger,
            control,
        )
        .map_err(legacy_work::BorrowedPreparationError::into_contract)?;
        validate_companion_keys(&locator, &manifest, ledger)?;
        let impact_base = actual_parent(capture, members[manifest_slot].index, ledger, control)?;
        let native_base = native_base(capture, &impact_base, ledger, control)?;
        let report_slot = required(
            capture,
            &locator.report_path,
            ImpactSourcePurpose::CurrentReport,
            &mut members,
            ledger,
            control,
        )?;
        let old =
            framework(capture, &impact_base, &manifest.old, true, &mut members, ledger, control)?;
        let new =
            framework(capture, &impact_base, &manifest.new, false, &mut members, ledger, control)?;
        let (mappings, applicability) =
            capture_dependencies(capture, &impact_base, &manifest, &mut members, ledger, control)?;
        let successor = optional(
            capture,
            &impact_base,
            manifest.successor_map.as_deref(),
            ImpactSourcePurpose::SuccessorMap,
            &mut members,
            ledger,
            control,
        )?;
        let prior = optional(
            capture,
            &impact_base,
            manifest.prior_report.as_deref(),
            ImpactSourcePurpose::PriorReport,
            &mut members,
            ledger,
            control,
        )?;
        let dispositions = optional(
            capture,
            &impact_base,
            manifest.disposition_file.as_deref(),
            ImpactSourcePurpose::Dispositions,
            &mut members,
            ledger,
            control,
        )?;
        let plan = Plan {
            manifest: manifest_slot,
            report: report_slot,
            old,
            new,
            mappings,
            applicability,
            successor,
            prior,
            dispositions,
        };
        let facts = compute(&members, &plan, &native_base, ledger, control)?;
        // ImpactData borrows facts only during this complete plain oracle; retaining
        // the owned facts below cannot create a self-referential borrowed report view.
        let _ = impact_report::compare_stored(
            original(&members, plan.report)?,
            &facts,
            ledger,
            control,
        )?;
        ledger.checkpoint(control)?;
        Ok(PendingImpactCohort { locator_index, locator_lease, members, facts, locator })
    })
}

/// Capture complete declared Mapping/applicability dependencies in their original phase order.
fn capture_dependencies(
    capture: &mut ReviewCapture,
    impact_base: &Path,
    manifest: &ImpactManifest,
    members: &mut Vec<ImpactMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(Vec<usize>, Option<ApplicabilityPlan>), ContractError> {
    let mut mappings = Vec::new();
    for dependency in &manifest.mapping_collections {
        let path = route(impact_base, &dependency.artifact, ledger, control)?;
        let slot = required(
            capture,
            &path,
            ImpactSourcePurpose::MappingCollection,
            members,
            ledger,
            control,
        )?;
        append_slot(&mut mappings, slot, ledger)?;
    }
    let applicability = if let Some(declared) = &manifest.applicability_manifest {
        let path = route(impact_base, declared, ledger, control)?;
        let slot = required(
            capture,
            &path,
            ImpactSourcePurpose::ApplicabilityManifest,
            members,
            ledger,
            control,
        )?;
        let parsed =
            legacy_work::discover_applicability_manifest(original(members, slot)?, ledger, control)
                .map_err(legacy_work::BorrowedPreparationError::into_contract)?;
        Some(applicability(capture, slot, &parsed, members, ledger, control)?)
    } else {
        None
    };
    Ok((mappings, applicability))
}

/// Decode the exact closed seven-field locator, requiring explicit nullable companions.
fn decode_locator(
    raw: &[u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Locator, ContractError> {
    let value_result = shared::strict_value(raw, MAX_LOCATOR_BYTES, ledger, control);
    ledger.checkpoint(control)?;
    let value = value_result?;
    let object = value.as_object().ok_or(ContractError::Invalid)?;
    ledger.visits(LOCATOR_FIELDS.len())?;
    let key_extent = LOCATOR_FIELDS.iter().try_fold(0usize, |total, key| {
        total.checked_add(key.len()).ok_or(ContractError::Capacity)
    })?;
    // Seven complete fixed-field BTreeMap probes fit the conservative Cartesian
    // key budget; all dynamic object-key bytes are bounded by this full raw input.
    ledger.matching(
        LOCATOR_FIELDS.len().checked_mul(LOCATOR_FIELDS.len()).ok_or(ContractError::Capacity)?,
    )?;
    ledger.bytes(
        raw.len()
            .checked_add(key_extent)
            .and_then(|bytes| bytes.checked_mul(LOCATOR_FIELDS.len()))
            .ok_or(ContractError::Capacity)?,
    )?;
    if object.len() != LOCATOR_FIELDS.len()
        || LOCATOR_FIELDS.iter().any(|key| !object.contains_key(*key))
    {
        return Err(ContractError::Invalid);
    }
    shared::admit_decode(raw, MAX_LOCATOR_BYTES, ledger, control)?;
    let decoded_result = serde_json::from_slice::<Locator>(raw).map_err(|_| ContractError::Invalid);
    ledger.checkpoint(control)?;
    let locator = decoded_result?;
    if shared::compare(&locator.schema_version, LOCATOR_SCHEMA, ledger)?
        != std::cmp::Ordering::Equal
    {
        return Err(ContractError::Invalid);
    }
    for path in [&locator.manifest_path, &locator.report_path] {
        shared::portable(path, ledger)?;
        if path.extension().and_then(|part| part.to_str()) != Some("json") {
            return Err(ContractError::Invalid);
        }
    }
    for key in [&locator.old_framework_source_key, &locator.new_framework_source_key] {
        shared::token(key, ledger)?;
    }
    for key in
        [locator.old_resolved_source_key.as_deref(), locator.new_resolved_source_key.as_deref()]
            .into_iter()
            .flatten()
    {
        shared::token(key, ledger)?;
    }
    ledger.checkpoint(control)?;
    Ok(locator)
}

/// Bind nullable companion declarations to actual maintained Catalog/Profile requirements.
fn validate_companion_keys(
    locator: &Locator,
    manifest: &ImpactManifest,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    for (resource, key, companion) in [
        (
            &manifest.old,
            locator.old_framework_source_key.as_str(),
            locator.old_resolved_source_key.as_deref(),
        ),
        (
            &manifest.new,
            locator.new_framework_source_key.as_str(),
            locator.new_resolved_source_key.as_deref(),
        ),
    ] {
        ledger.visits(1)?;
        if (resource.resource_type == ResourceType::Profile) != companion.is_some() {
            return Err(ContractError::Binding);
        }
        if let Some(companion) = companion {
            if shared::compare(key, companion, ledger)? == std::cmp::Ordering::Equal {
                return Err(ContractError::Binding);
            }
        }
    }
    Ok(())
}

/// Admit and capture one exact actual read before retaining its private purpose/lease record.
fn required(
    capture: &mut ReviewCapture,
    path: &Path,
    purpose: ImpactSourcePurpose,
    members: &mut Vec<ImpactMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    ledger.checkpoint(control)?;
    let index = capture.required_impact_source(path, purpose, ledger, control)?;
    ledger.visits(1)?;
    ledger.derived(std::mem::size_of::<ImpactMember>())?;
    if members.len() == members.capacity() {
        ledger.bytes(
            members
                .len()
                .checked_mul(std::mem::size_of::<ImpactMember>())
                .ok_or(ContractError::Capacity)?,
        )?;
    }
    members.try_reserve(1).map_err(|_| ledger.capacity())?;
    let lease = capture.lease(index)?;
    capture.role(index)?;
    let slot = members.len();
    members.push(ImpactMember { index, lease, purpose });
    ledger.checkpoint(control)?;
    Ok(slot)
}

/// Capture only an explicitly declared optional original, never inferred history or hrefs.
fn optional(
    capture: &mut ReviewCapture,
    base: &Path,
    declared: Option<&Path>,
    purpose: ImpactSourcePurpose,
    members: &mut Vec<ImpactMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Option<usize>, ContractError> {
    declared
        .map(|path| {
            let path = route(base, path, ledger, control)?;
            required(capture, &path, purpose, members, ledger, control)
        })
        .transpose()
}

/// Capture actual old/new framework artifacts and only explicitly declared Profile companions.
fn framework(
    capture: &mut ReviewCapture,
    base: &Path,
    resource: &FrameworkResource,
    old: bool,
    members: &mut Vec<ImpactMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<FrameworkPlan, ContractError> {
    let purpose = match (old, resource.resource_type) {
        (true, ResourceType::Catalog) => ImpactSourcePurpose::OldCatalog,
        (true, ResourceType::Profile) => ImpactSourcePurpose::OldProfile,
        (false, ResourceType::Catalog) => ImpactSourcePurpose::NewCatalog,
        (false, ResourceType::Profile) => ImpactSourcePurpose::NewProfile,
    };
    let path = route(base, &resource.artifact, ledger, control)?;
    let artifact = required(capture, &path, purpose, members, ledger, control)?;
    let companion = if old {
        ImpactSourcePurpose::OldResolvedCatalog
    } else {
        ImpactSourcePurpose::NewResolvedCatalog
    };
    let resolved = optional(
        capture,
        base,
        resource.resolved_catalog.as_deref(),
        companion,
        members,
        ledger,
        control,
    )?;
    Ok(FrameworkPlan { artifact, resolved })
}

/// Capture the entire configured applicability roster under its actual own manifest parent.
fn applicability(
    capture: &mut ReviewCapture,
    manifest_slot: usize,
    manifest: &ApplicabilityManifest,
    members: &mut Vec<ImpactMember>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ApplicabilityPlan, ContractError> {
    let base = actual_parent(capture, members[manifest_slot].index, ledger, control)?;
    let purpose = match manifest.framework.resource_type {
        ResourceType::Catalog => ImpactSourcePurpose::ApplicabilityCatalog,
        ResourceType::Profile => ImpactSourcePurpose::ApplicabilityProfile,
    };
    let path = route(&base, &manifest.framework.artifact, ledger, control)?;
    let framework = required(capture, &path, purpose, members, ledger, control)?;
    let resolved = optional(
        capture,
        &base,
        manifest.framework.resolved_catalog.as_deref(),
        ImpactSourcePurpose::ApplicabilityResolvedCatalog,
        members,
        ledger,
        control,
    )?;
    let mut mappings = Vec::new();
    for declared in &manifest.mapping_collections {
        let path = route(&base, declared, ledger, control)?;
        let slot = required(
            capture,
            &path,
            ImpactSourcePurpose::ApplicabilityMappingCollection,
            members,
            ledger,
            control,
        )?;
        append_slot(&mut mappings, slot, ledger)?;
    }
    Ok(ApplicabilityPlan { manifest: manifest_slot, framework, resolved, mappings })
}

/// Borrow complete actual original bytes from a private successful registration position.
fn original(members: &[ImpactMember], slot: usize) -> Result<&[u8], ContractError> {
    members.get(slot).map(|member| member.lease.bytes()).ok_or(ContractError::Binding)
}

/// Precharge positional plan storage before bounded private collection growth.
fn append_slot(
    slots: &mut Vec<usize>,
    slot: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.derived(std::mem::size_of::<usize>())?;
    if slots.len() == slots.capacity() {
        ledger.bytes(
            slots.len().checked_mul(std::mem::size_of::<usize>()).ok_or(ContractError::Capacity)?,
        )?;
    }
    slots.try_reserve(1).map_err(|_| ledger.capacity())?;
    slots.push(slot);
    Ok(())
}

/// Compose only a validated manifest-declared route under an actual captured manifest base.
fn route(
    base: &Path,
    declared: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PathBuf, ContractError> {
    ledger.checkpoint(control)?;
    shared::portable(declared, ledger)?;
    let extent = base
        .as_os_str()
        .as_encoded_bytes()
        .len()
        .checked_add(declared.as_os_str().as_encoded_bytes().len())
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or(ContractError::Capacity)?;
    ledger.bytes(extent)?;
    ledger.derived(
        extent.checked_add(std::mem::size_of::<PathBuf>()).ok_or(ContractError::Capacity)?,
    )?;
    let path = base.join(declared);
    shared::portable(&path, ledger)?;
    ledger.checkpoint(control)?;
    Ok(path)
}

/// Preserve the genuine registered manifest's own parent with admission before copying.
fn actual_parent(
    capture: &ReviewCapture,
    index: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PathBuf, ContractError> {
    ledger.checkpoint(control)?;
    let parent = capture.path(index)?.parent().unwrap_or_else(|| Path::new(""));
    let extent = parent.as_os_str().as_encoded_bytes().len();
    ledger.bytes(extent)?;
    ledger.derived(
        extent.checked_add(std::mem::size_of::<PathBuf>()).ok_or(ContractError::Capacity)?,
    )?;
    Ok(parent.to_path_buf())
}

/// Compose only the actual qualified root for private maintained native label semantics.
fn native_base(
    capture: &ReviewCapture,
    relative: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PathBuf, ContractError> {
    ledger.checkpoint(control)?;
    let extent = capture
        .root()
        .as_os_str()
        .as_encoded_bytes()
        .len()
        .checked_add(relative.as_os_str().as_encoded_bytes().len())
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or(ContractError::Capacity)?;
    ledger.bytes(extent)?;
    ledger.derived(
        extent.checked_add(std::mem::size_of::<PathBuf>()).ok_or(ContractError::Capacity)?,
    )?;
    Ok(capture.root().join(relative))
}

/// Borrow every complete Mapping original in the exact native manifest declaration order.
fn mapping_originals<'a>(
    members: &'a [ImpactMember],
    slots: &[usize],
    ledger: &mut ContractLedger,
) -> Result<Vec<&'a [u8]>, ContractError> {
    ledger.visits(slots.len())?;
    ledger.derived(
        slots.len().checked_mul(std::mem::size_of::<&[u8]>()).ok_or(ContractError::Capacity)?,
    )?;
    let mut raw = Vec::new();
    raw.try_reserve_exact(slots.len()).map_err(|_| ledger.capacity())?;
    for slot in slots {
        raw.push(original(members, *slot)?);
    }
    Ok(raw)
}

/// Compute complete ordinary native facts exclusively from the retained actual read roster.
fn compute(
    members: &[ImpactMember],
    plan: &Plan,
    base: &Path,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<LegacyImpactFacts, ContractError> {
    ledger.checkpoint(control)?;
    let mappings = mapping_originals(members, &plan.mappings, ledger)?;
    let app_mappings = plan
        .applicability
        .as_ref()
        .map(|app| mapping_originals(members, &app.mappings, ledger))
        .transpose()?;
    let app = plan
        .applicability
        .as_ref()
        .map(|app| {
            Ok::<_, ContractError>(ApplicabilityBytes {
                manifest: original(members, app.manifest)?,
                framework: original(members, app.framework)?,
                resolved_catalog: app.resolved.map(|slot| original(members, slot)).transpose()?,
                mappings: app_mappings.as_ref().ok_or(ContractError::Binding)?.as_slice(),
            })
        })
        .transpose()?;
    let inputs = ImpactInputs {
        manifest_dir: base,
        manifest: original(members, plan.manifest)?,
        old: FrameworkBytes {
            artifact: original(members, plan.old.artifact)?,
            resolved_catalog: plan.old.resolved.map(|slot| original(members, slot)).transpose()?,
        },
        new: FrameworkBytes {
            artifact: original(members, plan.new.artifact)?,
            resolved_catalog: plan.new.resolved.map(|slot| original(members, slot)).transpose()?,
        },
        mappings: &mappings,
        applicability: app,
        successor_map: plan.successor.map(|slot| original(members, slot)).transpose()?,
        prior_report: plan.prior.map(|slot| original(members, slot)).transpose()?,
        dispositions: plan.dispositions.map(|slot| original(members, slot)).transpose()?,
    };
    legacy_work::prepare_impact(inputs, ImpactFilters::default(), ledger, control)
        .map_err(legacy_work::BorrowedPreparationError::into_contract)
}

#[cfg(test)]
#[path = "impact_capture_tests.rs"]
/// Prospective genuine file/native controls; execution belongs to Root qualification.
mod tests;

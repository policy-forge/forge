//! Genuine two-queue and old-dependency binding over the complete same-held union.
//! Recorded queue bytes and plain graph results cannot construct this owner alone.
//! Its lifetime retains the captured comparison, actual native adapter and originals.

use super::{PreparedNative, sort_subjects_domain};
use crate::framework::model::{ImpactFinding, ReasonCode};
use crate::mapping::manifest::ResourceType;
use crate::review::adapters::{applicability, mapping};
use crate::review::capture::supersession::{
    AuxiliaryPurpose, HeldSupersessionInputs, ImpactSourcePurpose, NewNativeCohort, QueuePurpose,
};
use crate::review::chain::{compare, reserved, visit};
use crate::review::decode::{ContractError, ContractLedger, Decoded};
use crate::review::impact_capture::ImpactMember;
use crate::review::links::{DecodedLinks, ItemGraph};
use crate::review::mapping_capture;
use crate::review::wire::{Domain, QueueDocument, ReviewItem, SourceModel, SourcePin};
use crate::workspace::preparation::WorkControl;

/// Complete native finding bindings in exact explicit edge order.
/// No wire fields, copied queues, selected sources or caller proof flags issue this type.
pub(super) struct CapturedSupersessionBindings<'a, 'raw> {
    /// Exact private complete union from the actual A-D issuers.
    held: &'a HeldSupersessionInputs,
    /// Whole historical queue decoded directly from its registered retained allocation.
    old: &'a Decoded<'raw, QueueDocument>,
    /// Whole new queue, subsequently bound by its genuine complete native adapter.
    new: &'a Decoded<'raw, QueueDocument>,
    /// Complete same-domain explicit graph, including all unmatched endpoints.
    graph: ItemGraph<'a>,
    /// Actual selected native facts retaining their real sealed owner.
    native: PreparedNative<'a>,
    /// Complete native findings for each link, in the exact requested ID order.
    linked_findings: Vec<Vec<&'a ImpactFinding>>,
    /// Complete unique unreferenced current finding IDs in lexical order.
    unreferenced_findings: Vec<&'a str>,
}

impl CapturedSupersessionBindings<'_, '_> {
    /// Borrow the exact purpose-bound historical queue decoder, never a reconstructed declaration.
    pub(super) fn old_queue(&self) -> &Decoded<'_, QueueDocument> {
        self.old
    }
    /// Borrow the exact current native-bound new queue decoder retained by this owner.
    pub(super) fn new_queue(&self) -> &Decoded<'_, QueueDocument> {
        self.new
    }
    /// Borrow the actual union for separately minimized recorded serialization.
    pub(super) fn held(&self) -> &HeldSupersessionInputs {
        self.held
    }
    /// Borrow every exact plain graph endpoint and conservation counter.
    pub(super) fn graph(&self) -> &ItemGraph<'_> {
        &self.graph
    }
    /// Borrow complete per-link native references, never report prose or display paths.
    pub(super) fn linked_findings(&self) -> &[Vec<&ImpactFinding>] {
        &self.linked_findings
    }
    /// Borrow the full sorted current unreferenced denominator.
    pub(super) fn unreferenced_findings(&self) -> &[&str] {
        &self.unreferenced_findings
    }
    /// Retain exact complete new native/queue and union fences through final publication.
    pub(super) fn verify_inputs(
        &self,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        ledger.bound(|ledger| {
            ledger.checkpoint(control)?;
            let binding = self.native.view().bind_queue(self.new, ledger, control);
            ledger.checkpoint(control)?;
            binding?;
            bind_storage(
                self.old.raw(),
                self.held
                    .held_inputs()
                    .bytes(self.held.queue_original(QueuePurpose::HistoricalOld)?)?,
                ledger,
            )?;
            let originals = self.held.verify_inputs(ledger, control);
            ledger.checkpoint(control)?;
            originals
        })
    }
}

/// Issue only from the genuine complete union, its exact raw queues/request and native adapter.
/// Old review hashes remain recorded history; new-item relation remains explicit lineage.
pub(super) fn prepare<'a, 'raw>(
    held: &'a HeldSupersessionInputs,
    old: &'a Decoded<'raw, QueueDocument>,
    new: &'a Decoded<'raw, QueueDocument>,
    request: &'a DecodedLinks<'raw>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CapturedSupersessionBindings<'a, 'raw>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let result = prepare_inner(held, old, new, request, ledger, control);
        ledger.checkpoint(control)?;
        result
    })
}

/// Preserve one outer post-phase fence after all ordinary binding outcomes.
fn prepare_inner<'a, 'raw>(
    held: &'a HeldSupersessionInputs,
    old: &'a Decoded<'raw, QueueDocument>,
    new: &'a Decoded<'raw, QueueDocument>,
    request: &'a DecodedLinks<'raw>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CapturedSupersessionBindings<'a, 'raw>, ContractError> {
    ledger.checkpoint(control)?;
    bind_storage(
        old.raw(),
        held.held_inputs().bytes(held.queue_original(QueuePurpose::HistoricalOld)?)?,
        ledger,
    )?;
    bind_storage(
        new.raw(),
        held.held_inputs().bytes(held.queue_original(QueuePurpose::CurrentNew)?)?,
        ledger,
    )?;
    bind_storage(
        request.raw(),
        held.held_inputs().bytes(held.auxiliary_original(AuxiliaryPurpose::Links)?)?,
        ledger,
    )?;
    framework_bindings(held, old.document(), new.document(), ledger, control)?;
    let domain = match held.new_native() {
        NewNativeCohort::Mapping(_) => Domain::MappingAssertion,
        NewNativeCohort::Applicability(_) => Domain::ApplicabilityDecision,
    };
    let mut selected = reserved(new.document().items.len(), ledger)?;
    for item in &new.document().items {
        visit(ledger, control)?;
        if item.domain != domain {
            return Err(ContractError::Binding);
        }
        ledger.bytes(std::mem::size_of::<&str>())?;
        selected.push(item.subject_id.as_str());
    }
    sort_subjects_domain(&mut selected, domain, ledger, control)?;
    let native = match held.new_native() {
        NewNativeCohort::Mapping(closure) => {
            PreparedNative::Mapping(mapping::prepare(closure, &selected, ledger, control)?)
        }
        NewNativeCohort::Applicability(closure) => PreparedNative::Applicability(
            applicability::prepare(closure, &selected, ledger, control)?,
        ),
    };
    let result = native.view().bind_queue(new, ledger, control);
    ledger.checkpoint(control)?;
    result?;
    let graph = crate::review::links::prepare_graph(old, new, request, ledger, control)?;
    let report = &held.impact().facts().report;
    let mut referenced = reserved::<bool>(report.findings.len(), ledger)?;
    ledger.bytes(report.findings.len())?;
    referenced.resize(report.findings.len(), false);
    // Establish unique native existence for the complete distinct requested set.
    for id in graph.requested_findings() {
        let index = finding_index(&report.findings, id, ledger, control)?;
        ledger.visits(1)?;
        ledger.bytes(1)?;
        referenced[index] = true;
    }
    let mut linked_findings = reserved(graph.links().len(), ledger)?;
    for link in graph.links() {
        visit(ledger, control)?;
        let mut findings = reserved(link.requested_findings().len(), ledger)?;
        for id in link.requested_findings() {
            let index = finding_index(&report.findings, id, ledger, control)?;
            let finding = &report.findings[index];
            old_dependency(held, old.document(), link.old_item(), finding, ledger, control)?;
            ledger.bytes(std::mem::size_of::<&ImpactFinding>())?;
            findings.push(finding);
        }
        ledger.bytes(std::mem::size_of::<Vec<&ImpactFinding>>())?;
        linked_findings.push(findings);
    }
    let mut unreferenced_findings = reserved(report.findings.len(), ledger)?;
    for (finding, used) in report.findings.iter().zip(&referenced) {
        visit(ledger, control)?;
        if !used {
            insert_id(&mut unreferenced_findings, &finding.finding_id, ledger, control)?;
        }
    }
    let total = unreferenced_findings
        .len()
        .checked_add(graph.requested_findings().len())
        .ok_or_else(|| ledger.capacity())?;
    if total != report.findings.len() {
        return Err(ContractError::Binding);
    }
    ledger.derived(std::mem::size_of::<CapturedSupersessionBindings<'_, '_>>())?;
    let bound = CapturedSupersessionBindings {
        held,
        old,
        new,
        graph,
        native,
        linked_findings,
        unreferenced_findings,
    };
    bound.verify_inputs(ledger, control)?;
    ledger.checkpoint(control)?;
    Ok(bound)
}

/// Exact retained storage equality after purpose and complete cohort admission.
fn bind_storage(
    raw: &[u8],
    actual: &[u8],
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    ledger.bytes(4 * std::mem::size_of::<usize>())?;
    if raw.len() != actual.len() || !std::ptr::eq(raw.as_ptr(), actual.as_ptr()) {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Compare explicit old/new framework keys against the complete actual native resource pair.
fn framework_bindings(
    held: &HeldSupersessionInputs,
    old: &QueueDocument,
    new: &QueueDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let locator = held.impact().locator_fields();
    let report = &held.impact().facts().report;
    for (queue, resource, key, resolved_key, is_old) in [
        (old, &report.old, locator.old_framework, locator.old_resolved, true),
        (new, &report.new, locator.new_framework, locator.new_resolved, false),
    ] {
        let purpose = match (is_old, resource.resource_type) {
            (true, ResourceType::Catalog) => ImpactSourcePurpose::OldCatalog,
            (true, ResourceType::Profile) => ImpactSourcePurpose::OldProfile,
            (false, ResourceType::Catalog) => ImpactSourcePurpose::NewCatalog,
            (false, ResourceType::Profile) => ImpactSourcePurpose::NewProfile,
        };
        let framework_member = member(held, purpose, ledger, control)?;
        let framework_pin = pin(queue, key, ledger, control)?;
        let (model, schema) = match resource.resource_type {
            ResourceType::Catalog => (SourceModel::Catalog, "oscal:1.2.3:catalog"),
            ResourceType::Profile => (SourceModel::Profile, "oscal:1.2.3:profile"),
        };
        pin_tuple(
            framework_pin,
            framework_member,
            model,
            Some(&resource.root_uuid),
            &resource.raw_sha256,
            schema,
            ledger,
        )?;
        if !compare(&resource.oscal_version, crate::oscal::OSCAL_VERSION, ledger)?.is_eq() {
            return Err(ContractError::Binding);
        }
        match (resource.resource_type, resolved_key, resource.resolved_catalog_sha256.as_deref()) {
            (ResourceType::Catalog, None, None) => {}
            (ResourceType::Profile, Some(key), Some(hash)) => {
                let purpose = if is_old {
                    ImpactSourcePurpose::OldResolvedCatalog
                } else {
                    ImpactSourcePurpose::NewResolvedCatalog
                };
                let member = member(held, purpose, ledger, control)?;
                let root = native_uuid(member, "catalog", ledger, control)?;
                let pin = pin(queue, key, ledger, control)?;
                pin_tuple(
                    pin,
                    member,
                    SourceModel::ResolvedCatalog,
                    Some(root.as_str()),
                    hash,
                    "oscal:1.2.3:catalog",
                    ledger,
                )?;
            }
            _ => return Err(ContractError::Binding),
        }
    }
    ledger.checkpoint(control)
}

/// Resolve one closed singleton purpose from C's complete actual read roster, without caller indices.
fn member<'a>(
    held: &'a HeldSupersessionInputs,
    purpose: ImpactSourcePurpose,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a ImpactMember, ContractError> {
    let mut found = None;
    for member in held.impact().members() {
        visit(ledger, control)?;
        if member.purpose() == purpose {
            if found.is_some() {
                return Err(ContractError::Binding);
            }
            found = Some(member);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Resolve an exact unique historical/current queue pin from the entire already closed roster.
fn pin<'a>(
    queue: &'a QueueDocument,
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a SourcePin, ContractError> {
    let mut found = None;
    for row in &queue.source_pins {
        visit(ledger, control)?;
        if compare(&row.artifact_key, key, ledger)?.is_eq() {
            if found.is_some() {
                return Err(ContractError::Binding);
            }
            found = Some(row);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Compare full admitted model/root/hash/length/schema after complete operand admission.
fn pin_tuple(
    pin: &SourcePin,
    member: &ImpactMember,
    model: SourceModel,
    root: Option<&str>,
    hash: &str,
    schema: &str,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(6)?;
    ledger.bytes(std::mem::size_of::<SourceModel>() + 2 * std::mem::size_of::<u64>())?;
    let roots_equal = match (pin.native_root_uuid.as_deref(), root) {
        (Some(a), Some(b)) => compare(a, b, ledger)?.is_eq(),
        (None, None) => true,
        _ => false,
    };
    let hashes_equal = compare(&pin.raw_sha256, hash, ledger)?.is_eq();
    let schemas_equal = compare(&pin.schema_identity, schema, ledger)?.is_eq();
    let actual_length = u64::try_from(member.bytes().len()).map_err(|_| ledger.capacity())?;
    if pin.model != model
        || !roots_equal
        || !hashes_equal
        || !schemas_equal
        || pin.byte_length != actual_length
    {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Extract only native root spelling after C has fully qualified these actual bytes.
/// Strict decoding remains fully charged; this projection cannot create C membership.
fn native_uuid(
    member: &ImpactMember,
    model: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<String, ContractError> {
    let value = mapping_capture::strict_value(member.bytes(), 10 * 1024 * 1024, ledger, control)?;
    let uuid = value
        .get(model)
        .and_then(|root| root.get("uuid"))
        .and_then(serde_json::Value::as_str)
        .ok_or(ContractError::Binding)?;
    ledger.bytes(uuid.len())?;
    ledger.derived(uuid.len())?;
    let mut result = String::new();
    result.try_reserve_exact(uuid.len()).map_err(|_| ledger.capacity())?;
    result.push_str(uuid);
    ledger.checkpoint(control)?;
    Ok(result)
}

/// Require unique existence in the complete current report, charging every actual comparison.
fn finding_index(
    findings: &[ImpactFinding],
    id: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut found = None;
    for (index, finding) in findings.iter().enumerate() {
        visit(ledger, control)?;
        ledger.matching(1)?;
        if compare(&finding.finding_id, id, ledger)?.is_eq() {
            if found.is_some() {
                return Err(ContractError::Binding);
            }
            found = Some(index);
        }
    }
    found.ok_or(ContractError::Binding)
}

/// Verify the native old dependency ID and actual captured artifact against declared old pins.
/// No display `dependency_path`, private href or historical producer route is parsed or followed.
fn old_dependency(
    held: &HeldSupersessionInputs,
    queue: &QueueDocument,
    item: &ReviewItem,
    finding: &ImpactFinding,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let framework_key = held.impact().locator_fields().old_framework;
    require_old_framework(item, framework_key, ledger, control)?;
    let affected = finding.affected_artifact_id.as_deref().ok_or(ContractError::Binding)?;
    let dependency = finding.dependency_id.as_deref().ok_or(ContractError::Binding)?;
    let mut associated = false;
    match item.domain {
        Domain::MappingAssertion => {
            if !matches!(
                finding.reason_code,
                ReasonCode::MappingReferenceRemoved
                    | ReasonCode::MappingSubjectChanged
                    | ReasonCode::MappingSubjectMigrated
            ) || !compare(dependency, &item.subject_id, ledger)?.is_eq()
            {
                return Err(ContractError::Binding);
            }
            for member in held.impact().members() {
                visit(ledger, control)?;
                if member.purpose() != ImpactSourcePurpose::MappingCollection {
                    continue;
                }
                let uuid = native_uuid(member, "mapping-collection", ledger, control)?;
                if !compare(&uuid, affected, ledger)?.is_eq() {
                    continue;
                }
                ledger.bytes(member.bytes().len())?;
                ledger.derived(64)?;
                let hash = crate::hashing::sha256_hex(member.bytes());
                for key in &item.source_keys {
                    let pin = pin(queue, key, ledger, control)?;
                    if pin.model == SourceModel::Mapping
                        && compare(pin.native_root_uuid.as_deref().unwrap_or(""), &uuid, ledger)?
                            .is_eq()
                    {
                        pin_tuple(
                            pin,
                            member,
                            SourceModel::Mapping,
                            Some(&uuid),
                            &hash,
                            "oscal:1.2.3:mapping-collection",
                            ledger,
                        )?;
                        associated = true;
                    }
                }
            }
        }
        Domain::ApplicabilityDecision => {
            if !matches!(
                finding.reason_code,
                ReasonCode::ApplicabilityDecisionRemoved
                    | ReasonCode::ApplicabilityDecisionChanged
                    | ReasonCode::ApplicabilityDecisionMigrated
            ) || !compare(&finding.subject_id, &item.subject_id, ledger)?.is_eq()
            {
                return Err(ContractError::Binding);
            }
            let dependency_extent =
                item.subject_id.len().checked_add(14).ok_or_else(|| ledger.capacity())?;
            ledger.bytes(dependency_extent)?;
            ledger.derived(dependency_extent)?;
            let expected_dependency = format!("applicability:{}", item.subject_id);
            if !compare(dependency, &expected_dependency, ledger)?.is_eq() {
                return Err(ContractError::Binding);
            }
            let member = member(held, ImpactSourcePurpose::ApplicabilityManifest, ledger, control)?;
            ledger.bytes(member.bytes().len())?;
            ledger.derived(64)?;
            let hash = crate::hashing::sha256_hex(member.bytes());
            if !compare(affected, &hash, ledger)?.is_eq() {
                return Err(ContractError::Binding);
            }
            for key in &item.source_keys {
                let pin = pin(queue, key, ledger, control)?;
                if pin.model == SourceModel::ApplicabilityManifest
                    && compare(&pin.raw_sha256, &hash, ledger)?.is_eq()
                {
                    pin_tuple(
                        pin,
                        member,
                        SourceModel::ApplicabilityManifest,
                        None,
                        &hash,
                        "forge.applicability/1",
                        ledger,
                    )?;
                    associated = true;
                }
            }
        }
    }
    // Native report old-subject evidence must include the exact original control,
    // independently from any historical review fingerprint or new lineage endpoint.
    let subject_present = old_subject_present(finding, ledger, control)?;
    if !associated || !subject_present {
        return Err(ContractError::Binding);
    }
    ledger.checkpoint(control)
}

/// Require the explicitly declared old framework without changing comparison or visit order.
fn require_old_framework(
    item: &ReviewItem,
    framework_key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut present = false;
    for key in &item.source_keys {
        visit(ledger, control)?;
        present |= compare(key, framework_key, ledger)?.is_eq();
    }
    if !present {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Inspect every native old subject with the same original ledger and work control.
fn old_subject_present(
    finding: &ImpactFinding,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let mut present = false;
    for subject in &finding.old_subjects {
        visit(ledger, control)?;
        present |= compare(&subject.id, &finding.subject_id, ledger)?.is_eq();
    }
    Ok(present)
}

/// Insert each complete unreferenced native UUID with admitted actual comparisons and moves.
fn insert_id<'a>(
    rows: &mut Vec<&'a str>,
    value: &'a str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut lower = 0;
    let mut upper = rows.len();
    while lower < upper {
        visit(ledger, control)?;
        ledger.matching(1)?;
        let middle = lower + (upper - lower) / 2;
        match compare(rows[middle], value, ledger)? {
            std::cmp::Ordering::Less => lower = middle + 1,
            std::cmp::Ordering::Greater => upper = middle,
            std::cmp::Ordering::Equal => return Err(ContractError::Binding),
        }
    }
    let moves = rows
        .len()
        .checked_sub(lower)
        .and_then(|n| n.checked_add(1))
        .and_then(|n| n.checked_mul(std::mem::size_of::<&str>()))
        .ok_or_else(|| ledger.capacity())?;
    ledger.bytes(moves)?;
    rows.insert(lower, value);
    Ok(())
}

#[cfg(test)]
#[path = "supersession_binding_tests.rs"]
/// Genuine native queue and old-dependency binding controls.
mod tests;

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[path = "supersession_binding_mapping_tests.rs"]
/// Genuine Mapping-specific complete owner and native old-dependency controls.
mod mapping_tests;

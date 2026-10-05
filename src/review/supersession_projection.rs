//! Minimized recorded projection from genuine complete native/queue bindings.
//! Full private Impact equality and native association precede every copied field.
//! The output is recorded data; its constructor retains no publication authority.

use super::supersession_binding::CapturedSupersessionBindings;
use super::supersession_command::SupersedeOptions;
use crate::framework::model::{ChangeSummary, ImpactFinding};
use crate::mapping::inventory::ResourceEvidence;
use crate::mapping::manifest::ResourceType;
use crate::review::capture::supersession::{ImpactSourcePurpose, QueuePurpose};
use crate::review::chain::{compare, reserved, visit};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::impact_capture::ImpactMember;
use crate::review::links::ItemGraph;
use crate::review::supersession_wire::{
    AssertedIdentity, CompleteStatus, EmptyImpactFilters, Endpoint, FindingReference,
    ImpactCurrentness, ImpactReference, ImpactReportSchema, ImpactResource, ImpactResourceType,
    ImpactSchemaIdentity, ImpactSourceKind, ImpactSourcePin, ImpactSummary, LineageRelation,
    LineageSemantics, LinkProjection, NativeChangeObservation, NewFindingRelation,
    NewQueueCurrentness, OldDependencyBinding, OldQueueCurrentness, OriginalPin, QueueSchema,
    RecordedEndpoint, RecordedOriginalPin, RecordedQueueReference, RecordedQueueSourcePin,
    SupersessionCounts, SupersessionDocument, SupersessionSchema, SupportedOscalVersion,
    queue_reference,
};
use crate::review::wire::{Domain, ReviewItem, SourcePin};
use crate::workspace::preparation::WorkControl;

/// Copy one fully admitted complete string with exact before-growth logical charging.
fn text(value: &str, ledger: &mut ContractLedger) -> Result<String, ContractError> {
    ledger.bytes(value.len())?;
    let extent =
        value.len().checked_add(std::mem::size_of::<String>()).ok_or_else(|| ledger.capacity())?;
    ledger.derived(extent)?;
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| ledger.capacity())?;
    result.push_str(value);
    Ok(result)
}

/// Preserve explicit nullable fields without replacing absence by an inferred value.
fn nullable(
    value: Option<&str>,
    ledger: &mut ContractLedger,
) -> Result<Option<String>, ContractError> {
    value.map(|value| text(value, ledger)).transpose()
}

/// Convert a complete admitted count rather than truncate a platform usize.
fn count(value: usize, ledger: &mut ContractLedger) -> Result<u64, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(std::mem::size_of::<usize>())?;
    u64::try_from(value).map_err(|_| ledger.capacity())
}

/// Hash a whole actual retained allocation before copying its inert raw-pin fragment.
fn original(raw: &[u8], ledger: &mut ContractLedger) -> Result<RecordedOriginalPin, ContractError> {
    ledger.bytes(raw.len())?;
    ledger.derived(64 + std::mem::size_of::<String>())?;
    let hash = crate::hashing::sha256_hex(raw);
    let fragment = OriginalPin { raw_sha256: &hash, byte_length: count(raw.len(), ledger)? };
    Ok(RecordedOriginalPin {
        raw_sha256: text(fragment.raw_sha256, ledger)?,
        byte_length: fragment.byte_length,
    })
}

/// Preserve every original queue source pin in its existing closed vocabulary and order.
fn queue_pin(
    pin: &SourcePin,
    ledger: &mut ContractLedger,
) -> Result<RecordedQueueSourcePin, ContractError> {
    ledger.derived(std::mem::size_of::<RecordedQueueSourcePin>())?;
    Ok(RecordedQueueSourcePin {
        artifact_key: text(&pin.artifact_key, ledger)?,
        model: pin.model,
        native_root_uuid: nullable(pin.native_root_uuid.as_deref(), ledger)?,
        raw_sha256: text(&pin.raw_sha256, ledger)?,
        byte_length: pin.byte_length,
        schema_identity: text(&pin.schema_identity, ledger)?,
    })
}

/// Produce a whole actual queue reference using the native-bound exact original decoder.
fn queue(
    bound: &CapturedSupersessionBindings<'_, '_>,
    purpose: QueuePurpose,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<RecordedQueueReference, ContractError> {
    let decoded = match purpose {
        QueuePurpose::HistoricalOld => bound.old_queue(),
        QueuePurpose::CurrentNew => bound.new_queue(),
    };
    let fragment = queue_reference(decoded, ledger, control)?;
    let mut source_pins = reserved(fragment.source_pins.len(), ledger)?;
    for pin in fragment.source_pins {
        visit(ledger, control)?;
        source_pins.push(queue_pin(pin, ledger)?);
    }
    ledger.derived(std::mem::size_of::<RecordedQueueReference>())?;
    Ok(RecordedQueueReference {
        schema_version: QueueSchema::V1,
        queue_id: text(fragment.queue_id, ledger)?,
        raw_sha256: text(fragment.raw_sha256, ledger)?,
        byte_length: fragment.byte_length,
        source_pins,
        item_count: count(fragment.item_count, ledger)?,
    })
}

/// Copy exactly eleven admitted public endpoint fields, with no policy prose or assignments.
fn endpoint(
    fragment: &Endpoint<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<RecordedEndpoint, ContractError> {
    let mut source_keys = reserved(fragment.source_keys.len(), ledger)?;
    for key in fragment.source_keys {
        visit(ledger, control)?;
        source_keys.push(text(key, ledger)?);
    }
    ledger.derived(std::mem::size_of::<RecordedEndpoint>())?;
    Ok(RecordedEndpoint {
        key: text(fragment.key, ledger)?,
        item_id: text(fragment.item_id, ledger)?,
        domain: fragment.domain,
        adapter_version: text(fragment.adapter_version, ledger)?,
        subject_id: text(fragment.subject_id, ledger)?,
        subject_sha256: text(fragment.subject_sha256, ledger)?,
        context_sha256: text(fragment.context_sha256, ledger)?,
        policy_key: text(fragment.policy_key, ledger)?,
        policy_sha256: text(fragment.policy_sha256, ledger)?,
        requested_action: fragment.requested_action,
        source_keys,
    })
}

/// Extract the exact endpoint whitelist from the complete actual unmatched item roster.
fn item_endpoint(item: &ReviewItem) -> Endpoint<'_> {
    Endpoint {
        key: &item.key,
        item_id: &item.item_id,
        domain: item.domain,
        adapter_version: &item.adapter_version,
        subject_id: &item.subject_id,
        subject_sha256: &item.subject_sha256,
        context_sha256: &item.context_sha256,
        policy_key: &item.policy_key,
        policy_sha256: &item.policy_sha256,
        requested_action: item.requested_action,
        source_keys: &item.source_keys,
    }
}

/// Preserve every complete unmatched endpoint in the original queue item order.
fn unmatched(
    rows: &[&ReviewItem],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<RecordedEndpoint>, ContractError> {
    let mut result = reserved(rows.len(), ledger)?;
    for row in rows {
        visit(ledger, control)?;
        result.push(endpoint(&item_endpoint(row), ledger, control)?);
    }
    Ok(result)
}

/// Copy a genuinely associated native finding without historical paths, owners or rationale.
fn finding(
    finding: &ImpactFinding,
    domain: Domain,
    ledger: &mut ContractLedger,
) -> Result<FindingReference, ContractError> {
    ledger.derived(std::mem::size_of::<FindingReference>())?;
    Ok(FindingReference {
        finding_id: text(&finding.finding_id, ledger)?,
        priority: finding.priority,
        reason_code: finding.reason_code,
        required_action: finding.required_action,
        native_subject_id: text(&finding.subject_id, ledger)?,
        change_class: finding.change_class,
        old_dependency_binding: match domain {
            Domain::MappingAssertion => OldDependencyBinding::Mapping,
            Domain::ApplicabilityDecision => OldDependencyBinding::Applicability,
        },
        new_relation: NewFindingRelation::Declared,
    })
}

/// Emit exactly the explicit graph and all repeated genuinely bound native finding occurrences.
fn links(
    bound: &CapturedSupersessionBindings<'_, '_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<LinkProjection>, ContractError> {
    let graph = bound.graph();
    if graph.links().len() != bound.linked_findings().len() {
        return Err(ContractError::Binding);
    }
    let mut result = reserved(graph.links().len(), ledger)?;
    for (link, actual) in graph.links().iter().zip(bound.linked_findings()) {
        visit(ledger, control)?;
        if actual.len() != link.requested_findings().len() {
            return Err(ContractError::Binding);
        }
        let mut findings = reserved(actual.len(), ledger)?;
        for row in actual {
            visit(ledger, control)?;
            findings.push(finding(row, link.old_item().domain, ledger)?);
        }
        result.push(LinkProjection {
            old: endpoint(&link.old_endpoint(), ledger, control)?,
            new: endpoint(&link.new_endpoint(), ledger, control)?,
            differences: link.differences(),
            relation: LineageRelation::Declared,
            findings,
        });
    }
    Ok(result)
}

/// Preserve native schema families separately from the unchanged review `SourceModel` vocabulary.
fn family(
    purpose: ImpactSourcePurpose,
) -> (ImpactSourceKind, ImpactSchemaIdentity, Option<&'static str>, &'static str) {
    use ImpactSourcePurpose as P;
    match purpose {
        P::Manifest => {
            (ImpactSourceKind::ImpactManifest, ImpactSchemaIdentity::Manifest, None, "manifest")
        }
        P::CurrentReport => {
            (ImpactSourceKind::ImpactReport, ImpactSchemaIdentity::Report, None, "current-report")
        }
        P::PriorReport => {
            (ImpactSourceKind::ImpactReport, ImpactSchemaIdentity::Report, None, "prior-report")
        }
        P::OldCatalog => (
            ImpactSourceKind::Catalog,
            ImpactSchemaIdentity::Catalog,
            Some("catalog"),
            "old-catalog",
        ),
        P::NewCatalog => (
            ImpactSourceKind::Catalog,
            ImpactSchemaIdentity::Catalog,
            Some("catalog"),
            "new-catalog",
        ),
        P::ApplicabilityCatalog => (
            ImpactSourceKind::Catalog,
            ImpactSchemaIdentity::Catalog,
            Some("catalog"),
            "applicability-catalog",
        ),
        P::OldProfile => (
            ImpactSourceKind::Profile,
            ImpactSchemaIdentity::Profile,
            Some("profile"),
            "old-profile",
        ),
        P::NewProfile => (
            ImpactSourceKind::Profile,
            ImpactSchemaIdentity::Profile,
            Some("profile"),
            "new-profile",
        ),
        P::ApplicabilityProfile => (
            ImpactSourceKind::Profile,
            ImpactSchemaIdentity::Profile,
            Some("profile"),
            "applicability-profile",
        ),
        P::OldResolvedCatalog => (
            ImpactSourceKind::ResolvedCatalog,
            ImpactSchemaIdentity::Catalog,
            Some("catalog"),
            "old-resolved",
        ),
        P::NewResolvedCatalog => (
            ImpactSourceKind::ResolvedCatalog,
            ImpactSchemaIdentity::Catalog,
            Some("catalog"),
            "new-resolved",
        ),
        P::ApplicabilityResolvedCatalog => (
            ImpactSourceKind::ResolvedCatalog,
            ImpactSchemaIdentity::Catalog,
            Some("catalog"),
            "applicability-resolved",
        ),
        P::MappingCollection => (
            ImpactSourceKind::Mapping,
            ImpactSchemaIdentity::Mapping,
            Some("mapping-collection"),
            "mapping",
        ),
        P::ApplicabilityMappingCollection => (
            ImpactSourceKind::Mapping,
            ImpactSchemaIdentity::Mapping,
            Some("mapping-collection"),
            "applicability-mapping",
        ),
        P::ApplicabilityManifest => (
            ImpactSourceKind::ApplicabilityManifest,
            ImpactSchemaIdentity::Applicability,
            None,
            "applicability-manifest",
        ),
        P::SuccessorMap => {
            (ImpactSourceKind::SuccessorMap, ImpactSchemaIdentity::Successor, None, "successor")
        }
        P::Dispositions => (
            ImpactSourceKind::Dispositions,
            ImpactSchemaIdentity::Dispositions,
            None,
            "dispositions",
        ),
    }
}

/// Project the exact admitted native UUID spelling after the complete C/native proof.
fn native_uuid(
    member: &ImpactMember,
    model: Option<&str>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Option<String>, ContractError> {
    let Some(model) = model else {
        return Ok(None);
    };
    let value =
        crate::review::mapping_capture::strict_value(member.bytes(), 10_485_760, ledger, control)?;
    let root = value
        .get(model)
        .and_then(|root| root.get("uuid"))
        .and_then(serde_json::Value::as_str)
        .ok_or(ContractError::Binding)?;
    Ok(Some(text(root, ledger)?))
}

/// Record every actual ordered read occurrence, retaining repeats and omitting paths and versions.
fn impact_pins(
    bound: &CapturedSupersessionBindings<'_, '_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<ImpactSourcePin>, ContractError> {
    let members = bound.held().impact().members();
    let mut result = reserved(members.len(), ledger)?;
    for (ordinal, member) in members.iter().enumerate() {
        visit(ledger, control)?;
        let (kind, schema_identity, model, role) = family(member.purpose());
        let key_bound = role.len().checked_add(28).ok_or_else(|| ledger.capacity())?;
        ledger.bytes(key_bound)?;
        ledger.derived(key_bound + std::mem::size_of::<String>())?;
        let key = format!("impact:{role}:{ordinal}");
        crate::review::validate::token(&key, 128)?;
        let pin = original(member.bytes(), ledger)?;
        result.push(ImpactSourcePin {
            key,
            kind,
            native_root_uuid: native_uuid(member, model, ledger, control)?,
            raw_sha256: pin.raw_sha256,
            byte_length: pin.byte_length,
            schema_identity,
        });
    }
    Ok(result)
}

/// Select only a genuine singleton current original, never a prior-report display route.
fn impact_original(
    bound: &CapturedSupersessionBindings<'_, '_>,
    purpose: ImpactSourcePurpose,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<RecordedOriginalPin, ContractError> {
    let mut actual = None;
    for member in bound.held().impact().members() {
        visit(ledger, control)?;
        if member.purpose() == purpose {
            if actual.is_some() {
                return Err(ContractError::Binding);
            }
            actual = Some(member);
        }
    }
    original(actual.ok_or(ContractError::Binding)?.bytes(), ledger)
}

/// Fixed five-field resource projection after all private metadata and href equality gates.
fn resource(
    value: &ResourceEvidence,
    ledger: &mut ContractLedger,
) -> Result<ImpactResource, ContractError> {
    if !compare(&value.oscal_version, crate::oscal::OSCAL_VERSION, ledger)?.is_eq() {
        return Err(ContractError::Binding);
    }
    ledger.derived(std::mem::size_of::<ImpactResource>())?;
    Ok(ImpactResource {
        resource_type: match value.resource_type {
            ResourceType::Catalog => ImpactResourceType::Catalog,
            ResourceType::Profile => ImpactResourceType::Profile,
        },
        raw_sha256: text(&value.raw_sha256, ledger)?,
        root_uuid: text(&value.root_uuid, ledger)?,
        oscal_version: SupportedOscalVersion::V123,
        resolved_catalog_sha256: nullable(value.resolved_catalog_sha256.as_deref(), ledger)?,
    })
}

/// Preserve all fifteen genuine checked summary counters without defaults or filtering.
fn summary(
    value: &ChangeSummary,
    ledger: &mut ContractLedger,
) -> Result<ImpactSummary, ContractError> {
    ledger.derived(std::mem::size_of::<ImpactSummary>())?;
    Ok(ImpactSummary {
        old_controls: count(value.old_controls, ledger)?,
        new_controls: count(value.new_controls, ledger)?,
        added: count(value.added, ledger)?,
        removed: count(value.removed, ledger)?,
        content_changed: count(value.content_changed, ledger)?,
        identity_migrated: count(value.identity_migrated, ledger)?,
        unchanged: count(value.unchanged, ledger)?,
        findings: count(value.findings, ledger)?,
        blocking: count(value.blocking, ledger)?,
        review_required: count(value.review_required, ledger)?,
        informational: count(value.informational, ledger)?,
        dispositioned_resolved: count(value.dispositioned_resolved, ledger)?,
        dispositioned_accepted_risk: count(value.dispositioned_accepted_risk, ledger)?,
        dispositioned_still_open: count(value.dispositioned_still_open, ledger)?,
        undispositioned: count(value.undispositioned, ledger)?,
    })
}

/// Preserve complete graph/native membership denominators, including repeated uses.
fn counts(
    graph: &ItemGraph<'_>,
    unreferenced: usize,
    ledger: &mut ContractLedger,
) -> Result<SupersessionCounts, ContractError> {
    let value = graph.counts();
    ledger.derived(std::mem::size_of::<SupersessionCounts>())?;
    Ok(SupersessionCounts {
        old_items: count(value.old_items, ledger)?,
        new_items: count(value.new_items, ledger)?,
        link_edges: count(value.link_edges, ledger)?,
        linked_old_items: count(value.linked_old_items, ledger)?,
        linked_new_items: count(value.linked_new_items, ledger)?,
        unmatched_old_items: count(value.unmatched_old_items, ledger)?,
        unmatched_new_items: count(value.unmatched_new_items, ledger)?,
        link_finding_occurrences: count(value.link_finding_occurrences, ledger)?,
        distinct_linked_findings: count(value.distinct_requested_findings, ledger)?,
        unreferenced_findings: count(unreferenced, ledger)?,
    })
}

/// Produce only from a genuine complete binding owner, on its original invocation ledger/control.
pub(super) fn build(
    bound: &CapturedSupersessionBindings<'_, '_>,
    options: &SupersedeOptions<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<SupersessionDocument, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let result = build_inner(bound, options, ledger, control);
        ledger.checkpoint(control)?;
        result
    })
}

/// Retain all private owners while copying only the complete fixed public whitelist.
fn build_inner(
    bound: &CapturedSupersessionBindings<'_, '_>,
    options: &SupersedeOptions<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<SupersessionDocument, ContractError> {
    bound.verify_inputs(ledger, control)?;
    for queue in [bound.graph().old_queue(), bound.graph().new_queue()] {
        if compare(options.created_at, &queue.created_at, ledger)?.is_lt() {
            return Err(ContractError::Binding);
        }
    }
    let native = &bound.held().impact().facts().report;
    let mut unreferenced_finding_ids = reserved(bound.unreferenced_findings().len(), ledger)?;
    for id in bound.unreferenced_findings() {
        visit(ledger, control)?;
        unreferenced_finding_ids.push(text(id, ledger)?);
    }
    let locator = bound.held().impact().locator_fields();
    let raw_locator = bound.held().impact().locator().1.bytes();
    ledger.derived(std::mem::size_of::<ImpactReference>())?;
    let impact = ImpactReference {
        manifest: impact_original(bound, ImpactSourcePurpose::Manifest, ledger, control)?,
        report: impact_original(bound, ImpactSourcePurpose::CurrentReport, ledger, control)?,
        locator: original(raw_locator, ledger)?,
        schema_version: ImpactReportSchema::V1,
        status: CompleteStatus::Complete,
        currentness: ImpactCurrentness::Recorded,
        source_pins: impact_pins(bound, ledger, control)?,
        old_framework_source_key: text(locator.old_framework, ledger)?,
        new_framework_source_key: text(locator.new_framework, ledger)?,
        old_resolved_source_key: nullable(locator.old_resolved, ledger)?,
        new_resolved_source_key: nullable(locator.new_resolved, ledger)?,
        old: resource(&native.old, ledger)?,
        new: resource(&native.new, ledger)?,
        summary: summary(&native.summary, ledger)?,
        filters: EmptyImpactFilters {
            group: (),
            decision_state: (),
            policy_source: (),
            priority: (),
            owner: (),
        },
        change_count: count(native.changes.len(), ledger)?,
        finding_count: count(native.findings.len(), ledger)?,
        matched_findings: count(native.matched_findings, ledger)?,
        native_change_observation: if native.summary.added == 0
            && native.summary.removed == 0
            && native.summary.content_changed == 0
            && native.summary.identity_migrated == 0
            && native.findings.is_empty()
        {
            NativeChangeObservation::NoneDetected
        } else {
            NativeChangeObservation::Detected
        },
        distinct_linked_findings: count(
            bound.graph().counts().distinct_requested_findings,
            ledger,
        )?,
        unreferenced_finding_ids,
        prior_only_disposition_count: count(native.prior_only_dispositions.len(), ledger)?,
    };
    ledger.derived(std::mem::size_of::<SupersessionDocument>())?;
    let document = SupersessionDocument {
        schema_version: SupersessionSchema::V1,
        identity_disclaimer: AssertedIdentity::Asserted,
        sensitivity: crate::review::wire::Sensitivity::IdsAndHashes,
        supersession_id: text(options.supersession_id, ledger)?,
        created_at: text(options.created_at, ledger)?,
        semantics: LineageSemantics::Declared,
        old_queue_currentness: OldQueueCurrentness::Historical,
        new_queue_currentness: NewQueueCurrentness::Recorded,
        old_queue: queue(bound, QueuePurpose::HistoricalOld, ledger, control)?,
        new_queue: queue(bound, QueuePurpose::CurrentNew, ledger, control)?,
        impact,
        links: links(bound, ledger, control)?,
        unmatched_old_items: unmatched(bound.graph().unmatched_old(), ledger, control)?,
        unmatched_new_items: unmatched(bound.graph().unmatched_new(), ledger, control)?,
        counts: counts(bound.graph(), bound.unreferenced_findings().len(), ledger)?,
    };
    bound.verify_inputs(ledger, control)?;
    Ok(document)
}

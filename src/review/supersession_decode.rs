//! Closed recorded companion decoding and exact two-queue data binding.
//! Loaded currentness/completeness/association labels remain inert. No captured
//! owner, native proof, source union, authentication or publication capability
//! is issued here. Parser/schema internals are not a total heap or preemption
//! guarantee; inherited cumulative logical limits can refuse smaller inputs.

use super::decode::{ContractError, ContractLedger, Decoded};
use super::supersession_wire::{
    Differences, FindingReference, ImpactReference, ImpactResource, ImpactResourceType,
    ImpactSchemaIdentity, ImpactSourceKind, ImpactSourcePin, NativeChangeObservation,
    OldDependencyBinding, RecordedEndpoint, RecordedOriginalPin, RecordedQueueReference,
    RecordedQueueSourcePin, SupersessionDocument,
};
use super::wire::{
    ContextSnapshot, Domain, QueueDocument, ReviewItem, ReviewPolicy, SourceModel, SourcePin,
};
use crate::framework::model::{ChangeClass, FindingPriority, ReasonCode, RequiredAction};
use crate::workspace::preparation::WorkControl;
use serde::Serialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::io::{self, Write};

/// Entire recorded output extent, including an optional final LF.
const MAX_RAW: usize = 33_554_432;
/// Trusted retained closed shape; it never establishes source authority.
const SCHEMA: &str = include_str!("../../schemas/forge.review-queue-supersession-1.schema.json");

/// Private owner of inert typed assertions plus the exact borrowed original.
pub(crate) struct RecordedSupersession<'a> {
    /// Entire supplied raw slice; never reconstructed from fields.
    #[cfg(test)]
    raw: &'a [u8],
    /// Keep the original's lifetime without restoring capture/currentness capability.
    _original: std::marker::PhantomData<&'a [u8]>,
    /// Digest of that exact original extent.
    #[cfg(test)]
    raw_sha256: String,
    /// Closed, semantically correlated recorded declarations only.
    document: SupersessionDocument,
}

impl RecordedSupersession<'_> {
    /// Obtain inert recorded declarations, without restoring creation authority.
    pub(crate) fn document(&self) -> &SupersessionDocument {
        &self.document
    }
    /// Obtain the entire exact original, including whitespace and LF.
    #[cfg(test)]
    pub(crate) fn raw(&self) -> &[u8] {
        self.raw
    }
    /// Obtain the original-byte digest, without captured/currentness credit.
    #[cfg(test)]
    pub(crate) fn raw_sha256(&self) -> &str {
        &self.raw_sha256
    }
}

/// Decode a complete closed record under the same sticky caller ledger/control.
///
/// # Errors
/// Invalid shape/correlation returns a fixed refusal; first actual capacity or
/// control stops dominate both success and ordinary failure at the final fence.
pub(crate) fn decode<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<RecordedSupersession<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let result = decode_inner(raw, ledger, control);
        ledger.checkpoint(control)?;
        result
    })
}

/// Preserve strict original parsing, complete typed admission and phase fences.
fn decode_inner<'a>(
    raw: &'a [u8],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<RecordedSupersession<'a>, ContractError> {
    if raw.len() > MAX_RAW {
        return Err(ledger.capacity());
    }
    if raw.is_empty() || raw.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(ContractError::Invalid);
    }
    let parsed = super::mapping_capture::strict_value(raw, MAX_RAW, ledger, control);
    ledger.checkpoint(control)?;
    let value = parsed?;
    shape(&value, raw.len(), ledger, control)?;
    preflight_findings(&value, ledger, control)?;
    typed_storage(&value, ledger, control)?;
    ledger.bytes(raw.len())?;
    let typed =
        serde_json::from_value::<SupersessionDocument>(value).map_err(|_| ContractError::Invalid);
    ledger.checkpoint(control)?;
    let document = typed?;
    validate_document(&document, ledger, control)?;
    ledger.bytes(raw.len())?;
    ledger.derived(64 + std::mem::size_of::<String>())?;
    let raw_sha256 = crate::hashing::sha256_hex(raw);
    ledger.checkpoint(control)?;
    #[cfg(not(test))]
    drop(raw_sha256);
    Ok(RecordedSupersession {
        #[cfg(test)]
        raw,
        _original: std::marker::PhantomData,
        #[cfg(test)]
        raw_sha256,
        document,
    })
}

/// Compile the retained schema and fence both validation outcomes on actual control.
fn shape(
    value: &Value,
    raw_len: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bytes(SCHEMA.len())?;
    let reserve = add(mul(SCHEMA.len(), 16, ledger)?, 512, ledger)?;
    ledger.derived(reserve)?;
    let parsed = serde_json::from_str::<Value>(SCHEMA).map_err(|_| ContractError::SchemaDefinition);
    ledger.checkpoint(control)?;
    let schema = parsed?;
    let compiled = jsonschema::validator_for(&schema).map_err(|_| ContractError::SchemaDefinition);
    ledger.checkpoint(control)?;
    let validator = compiled?;
    ledger.bytes(raw_len)?;
    let valid = validator.is_valid(value);
    ledger.checkpoint(control)?;
    if valid { Ok(()) } else { Err(ContractError::Invalid) }
}

/// Admit the entire aggregate finding occurrence count before typed collection growth.
fn preflight_findings(
    value: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    step(ledger, control)?;
    ledger.bytes(1024)?;
    let links = value.get("links").and_then(Value::as_array).ok_or(ContractError::Invalid)?;
    let mut occurrences = 0;
    for link in links {
        step(ledger, control)?;
        ledger.bytes(1024)?;
        let rows = link.get("findings").and_then(Value::as_array).ok_or(ContractError::Invalid)?;
        occurrences = add(occurrences, rows.len(), ledger)?;
        if occurrences > 100_000 {
            return Err(ledger.capacity());
        }
    }
    Ok(())
}

/// Reserve conservative complete owned typed tree/string storage before serde growth.
fn typed_storage(
    value: &Value,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    step(ledger, control)?;
    ledger.derived(64)?;
    match value {
        Value::String(text) => ledger.derived(text.len())?,
        Value::Array(rows) => {
            for row in rows {
                typed_storage(row, ledger, control)?;
            }
        }
        Value::Object(fields) => {
            for (key, row) in fields {
                ledger.derived(key.len())?;
                typed_storage(row, ledger, control)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Correlate complete declarations; none of these checks is native association.
fn validate_document(
    document: &SupersessionDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    step(ledger, control)?;
    ledger.bytes(document.supersession_id.len() + document.created_at.len())?;
    super::validate::uuid(&document.supersession_id)?;
    super::validate::time(&document.created_at)?;
    queue_reference(&document.old_queue, ledger, control)?;
    queue_reference(&document.new_queue, ledger, control)?;
    if compared(&document.old_queue.queue_id, &document.new_queue.queue_id, ledger)?
        == Ordering::Equal
    {
        return Err(ContractError::Invalid);
    }
    let old_linked = endpoint_roster(document, true, ledger, control)?;
    let new_linked = endpoint_roster(document, false, ledger, control)?;
    let occurrences = links(document, ledger, control)?;
    let distinct = findings(document, ledger, control)?;
    counts(document, old_linked, new_linked, occurrences, distinct, ledger)?;
    summary(&document.impact, ledger)?;
    impact_sources(document, ledger, control)?;
    Ok(())
}

/// Validate the complete declared queue pin roster and fixed model/schema pairings.
fn queue_reference(
    queue: &RecordedQueueReference,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    step(ledger, control)?;
    ledger.bytes(queue.queue_id.len())?;
    super::validate::uuid(&queue.queue_id)?;
    if queue.item_count == 0
        || queue.item_count > 10_000
        || queue.byte_length == 0
        || queue.byte_length > 10_485_760
        || queue.source_pins.is_empty()
        || queue.source_pins.len() > 100
    {
        return Err(ContractError::Invalid);
    }
    let mut bytes = 0u64;
    for (index, pin) in queue.source_pins.iter().enumerate() {
        step(ledger, control)?;
        if index > 0
            && compared(&queue.source_pins[index - 1].artifact_key, &pin.artifact_key, ledger)?
                != Ordering::Less
        {
            return Err(ContractError::Invalid);
        }
        ledger.bytes(pin.artifact_key.len() + pin.schema_identity.len())?;
        super::validate::token(&pin.artifact_key, 128)?;
        let expected = match pin.model {
            SourceModel::Mapping => "oscal:1.2.3:mapping-collection",
            SourceModel::Catalog | SourceModel::ResolvedCatalog => "oscal:1.2.3:catalog",
            SourceModel::Profile => "oscal:1.2.3:profile",
            SourceModel::ComponentDefinition => "oscal:1.2.3:component-definition",
            SourceModel::MappingManifest => "forge.mapping-manifest/1",
            SourceModel::ApplicabilityManifest => "forge.applicability/1",
            SourceModel::ApplicabilityReport => "forge.applicability-report/1",
            SourceModel::LifecycleRecord if pin.schema_identity == "forge.policy-lifecycle/1" => {
                "forge.policy-lifecycle/1"
            }
            SourceModel::LifecycleRecord => "forge.policy-lifecycle/2",
        };
        if compared(&pin.schema_identity, expected, ledger)? != Ordering::Equal {
            return Err(ContractError::Invalid);
        }
        let native = matches!(
            pin.model,
            SourceModel::Mapping
                | SourceModel::Catalog
                | SourceModel::Profile
                | SourceModel::ResolvedCatalog
                | SourceModel::ComponentDefinition
        );
        match (native, &pin.native_root_uuid) {
            (true, Some(id)) => {
                ledger.bytes(id.len())?;
                super::validate::uuid(id)?;
            }
            (false, None) => {}
            _ => return Err(ContractError::Invalid),
        }
        bytes = bytes.checked_add(pin.byte_length).ok_or_else(|| ledger.capacity())?;
        if bytes > 52_428_800 {
            return Err(ContractError::Invalid);
        }
    }
    Ok(())
}

/// A borrowed endpoint occurrence retains whether it came from a declared edge.
#[derive(Clone, Copy)]
struct Occurrence<'a> {
    /// Exact entire recorded endpoint.
    endpoint: &'a RecordedEndpoint,
    /// Plain edge membership, without native association.
    linked: bool,
}

/// Validate complete endpoint conservation with repeated-edge consistency and no Cartesian scan.
fn endpoint_roster(
    document: &SupersessionDocument,
    old: bool,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let queue = if old { &document.old_queue } else { &document.new_queue };
    let unmatched = if old { &document.unmatched_old_items } else { &document.unmatched_new_items };
    let extent = add(document.links.len(), unmatched.len(), ledger)?;
    let mut rows = vector::<Occurrence<'_>>(extent, ledger)?;
    for link in &document.links {
        let endpoint = if old { &link.old } else { &link.new };
        endpoint_shape(endpoint, queue, ledger, control)?;
        ledger.bytes(std::mem::size_of::<Occurrence<'_>>())?;
        rows.push(Occurrence { endpoint, linked: true });
    }
    for (index, endpoint) in unmatched.iter().enumerate() {
        endpoint_shape(endpoint, queue, ledger, control)?;
        if index > 0
            && compared(&unmatched[index - 1].key, &endpoint.key, ledger)? != Ordering::Less
        {
            return Err(ContractError::Invalid);
        }
        ledger.bytes(std::mem::size_of::<Occurrence<'_>>())?;
        rows.push(Occurrence { endpoint, linked: false });
    }
    ordered(
        &mut rows,
        &mut |a, b, ledger| compared(&a.endpoint.item_id, &b.endpoint.item_id, ledger),
        ledger,
        control,
    )?;
    let mut retained = 0;
    let mut linked = 0;
    for index in 0..rows.len() {
        step(ledger, control)?;
        let row = rows[index];
        if retained > 0
            && compared(&rows[retained - 1].endpoint.item_id, &row.endpoint.item_id, ledger)?
                == Ordering::Equal
        {
            let previous = rows[retained - 1];
            if !previous.linked
                || !row.linked
                || !endpoint_equal(previous.endpoint, row.endpoint, ledger, control)?
            {
                return Err(ContractError::Invalid);
            }
        } else {
            ledger.bytes(std::mem::size_of::<Occurrence<'_>>())?;
            rows[retained] = row;
            retained = add(retained, 1, ledger)?;
            if row.linked {
                linked = add(linked, 1, ledger)?;
            }
        }
    }
    rows.truncate(retained);
    if count(retained, ledger)? != queue.item_count {
        return Err(ContractError::Invalid);
    }
    ordered(
        &mut rows,
        &mut |a, b, ledger| compared(&a.endpoint.key, &b.endpoint.key, ledger),
        ledger,
        control,
    )?;
    for pair in rows.windows(2) {
        if compared(&pair[0].endpoint.key, &pair[1].endpoint.key, ledger)? != Ordering::Less {
            return Err(ContractError::Invalid);
        }
    }
    Ok(linked)
}

/// Correlate exact adapter and complete item source keys against the recorded queue roster.
fn endpoint_shape(
    endpoint: &RecordedEndpoint,
    queue: &RecordedQueueReference,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    step(ledger, control)?;
    let expected = match endpoint.domain {
        Domain::MappingAssertion => "forge.mapping-review/1",
        Domain::ApplicabilityDecision => "forge.applicability-review/1",
    };
    if compared(&endpoint.adapter_version, expected, ledger)? != Ordering::Equal {
        return Err(ContractError::Invalid);
    }
    if endpoint.domain == Domain::MappingAssertion {
        ledger.bytes(endpoint.subject_id.len())?;
        super::validate::uuid(&endpoint.subject_id)?;
    }
    for (index, key) in endpoint.source_keys.iter().enumerate() {
        step(ledger, control)?;
        if index > 0 && compared(&endpoint.source_keys[index - 1], key, ledger)? != Ordering::Less {
            return Err(ContractError::Invalid);
        }
        locate(&queue.source_pins, key, |row| &row.artifact_key, ledger, control)
            .map_err(|error| ordinary(error, ContractError::Invalid))?;
    }
    Ok(())
}

/// Validate strict pair order, same-domain endpoints and per-edge finding order.
fn links(
    document: &SupersessionDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut occurrences = 0;
    for (index, link) in document.links.iter().enumerate() {
        step(ledger, control)?;
        if link.old.domain != link.new.domain {
            return Err(ContractError::Invalid);
        }
        if index > 0 {
            let previous = &document.links[index - 1];
            let old = compared(&previous.old.item_id, &link.old.item_id, ledger)?;
            if old == Ordering::Greater
                || (old == Ordering::Equal
                    && compared(&previous.new.item_id, &link.new.item_id, ledger)?
                        != Ordering::Less)
            {
                return Err(ContractError::Invalid);
            }
        }
        occurrences = add(occurrences, link.findings.len(), ledger)?;
        if occurrences > 100_000 {
            return Err(ledger.capacity());
        }
        if !link.findings.is_empty() {
            old_dependency_sources(document, &link.old, ledger, control)?;
        }
        for (finding_index, finding) in link.findings.iter().enumerate() {
            step(ledger, control)?;
            if finding_index > 0
                && compared(
                    &link.findings[finding_index - 1].finding_id,
                    &finding.finding_id,
                    ledger,
                )? != Ordering::Less
            {
                return Err(ContractError::Invalid);
            }
            finding_family(finding, link.old.domain)?;
            if link.old.domain == Domain::ApplicabilityDecision
                && compared(&finding.native_subject_id, &link.old.subject_id, ledger)?
                    != Ordering::Equal
            {
                return Err(ContractError::Invalid);
            }
            ledger.visits(1)?;
            ledger.bytes(std::mem::size_of::<ChangeClass>() + std::mem::size_of::<u64>())?;
            let changes = match finding.change_class {
                ChangeClass::Removed => document.impact.summary.removed,
                ChangeClass::ContentChanged => document.impact.summary.content_changed,
                ChangeClass::IdentityMigrated => document.impact.summary.identity_migrated,
                ChangeClass::Added | ChangeClass::Unchanged => return Err(ContractError::Invalid),
            };
            if changes == 0 {
                return Err(ContractError::Invalid);
            }
        }
    }
    Ok(occurrences)
}

/// Correlate only observable selected old dependency tuples, never omitted native IDs or ownership.
fn old_dependency_sources(
    document: &SupersessionDocument,
    endpoint: &RecordedEndpoint,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut framework_present = false;
    for key in &endpoint.source_keys {
        step(ledger, control)?;
        framework_present |=
            compared(key, &document.impact.old_framework_source_key, ledger)? == Ordering::Equal;
    }
    if !framework_present {
        return Err(ContractError::Invalid);
    }
    let (role, model, kind, schema, identity) = match endpoint.domain {
        Domain::MappingAssertion => (
            "mapping",
            SourceModel::Mapping,
            ImpactSourceKind::Mapping,
            ImpactSchemaIdentity::Mapping,
            "oscal:1.2.3:mapping-collection",
        ),
        Domain::ApplicabilityDecision => (
            "applicability-manifest",
            SourceModel::ApplicabilityManifest,
            ImpactSourceKind::ApplicabilityManifest,
            ImpactSchemaIdentity::Applicability,
            "forge.applicability/1",
        ),
    };
    let mut associated = false;
    for key in &endpoint.source_keys {
        step(ledger, control)?;
        let queue =
            locate(&document.old_queue.source_pins, key, |row| &row.artifact_key, ledger, control)
                .map_err(|error| ordinary(error, ContractError::Invalid))?;
        ledger.visits(1)?;
        ledger.bytes(std::mem::size_of::<SourceModel>())?;
        if queue.model != model {
            continue;
        }
        for native in &document.impact.source_pins {
            step(ledger, control)?;
            ledger.bytes(native.key.len())?;
            if compared(role_key(&native.key)?.0, role, ledger)? != Ordering::Equal {
                continue;
            }
            cost(queue, ledger, control)?;
            cost(native, ledger, control)?;
            ledger.visits(1)?;
            ledger.matching(1)?;
            associated |= native.kind == kind
                && native.schema_identity == schema
                && queue.native_root_uuid == native.native_root_uuid
                && queue.raw_sha256 == native.raw_sha256
                && queue.byte_length == native.byte_length
                && compared(&queue.schema_identity, identity, ledger)? == Ordering::Equal;
        }
    }
    if associated { Ok(()) } else { Err(ContractError::Invalid) }
}

/// Correlate only the maintained finding family/class/priority/action vocabulary.
fn finding_family(finding: &FindingReference, domain: Domain) -> Result<(), ContractError> {
    use ChangeClass::{ContentChanged, IdentityMigrated, Removed};
    use ReasonCode::{
        ApplicabilityDecisionChanged, ApplicabilityDecisionMigrated, ApplicabilityDecisionRemoved,
        MappingReferenceRemoved, MappingSubjectChanged, MappingSubjectMigrated,
    };
    let valid =
        match (domain, finding.old_dependency_binding, finding.reason_code, finding.change_class) {
            (
                Domain::MappingAssertion,
                OldDependencyBinding::Mapping,
                MappingReferenceRemoved,
                Removed,
            ) => {
                finding.priority == FindingPriority::Blocking
                    && finding.required_action == RequiredAction::RepairOrApproveMapping
            }
            (
                Domain::MappingAssertion,
                OldDependencyBinding::Mapping,
                MappingSubjectChanged,
                ContentChanged,
            )
            | (
                Domain::MappingAssertion,
                OldDependencyBinding::Mapping,
                MappingSubjectMigrated,
                IdentityMigrated,
            ) => {
                finding.priority == FindingPriority::ReviewRequired
                    && finding.required_action == RequiredAction::ReapproveMappingRationale
            }
            (
                Domain::ApplicabilityDecision,
                OldDependencyBinding::Applicability,
                ApplicabilityDecisionRemoved,
                Removed,
            )
            | (
                Domain::ApplicabilityDecision,
                OldDependencyBinding::Applicability,
                ApplicabilityDecisionChanged,
                ContentChanged,
            )
            | (
                Domain::ApplicabilityDecision,
                OldDependencyBinding::Applicability,
                ApplicabilityDecisionMigrated,
                IdentityMigrated,
            ) => {
                finding.priority == FindingPriority::ReviewRequired
                    && finding.required_action == RequiredAction::ReviewApplicabilityDecision
            }
            _ => false,
        };
    if valid { Ok(()) } else { Err(ContractError::Invalid) }
}

/// Validate repeated full finding references and complete distinct/unreferenced conservation.
fn findings(
    document: &SupersessionDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut length = 0;
    for link in &document.links {
        step(ledger, control)?;
        length = add(length, link.findings.len(), ledger)?;
    }
    let mut rows = vector::<&FindingReference>(length, ledger)?;
    for link in &document.links {
        for row in &link.findings {
            step(ledger, control)?;
            ledger.bytes(std::mem::size_of::<&FindingReference>())?;
            rows.push(row);
        }
    }
    ordered(
        &mut rows,
        &mut |a, b, ledger| compared(&a.finding_id, &b.finding_id, ledger),
        ledger,
        control,
    )?;
    let mut retained = 0;
    let mut priorities = [0u64; 3];
    for index in 0..rows.len() {
        step(ledger, control)?;
        let row = rows[index];
        if retained > 0
            && compared(&rows[retained - 1].finding_id, &row.finding_id, ledger)? == Ordering::Equal
        {
            if !finding_equal(rows[retained - 1], row, ledger, control)? {
                return Err(ContractError::Invalid);
            }
        } else {
            ledger.bytes(std::mem::size_of::<&FindingReference>())?;
            rows[retained] = row;
            retained = add(retained, 1, ledger)?;
            let slot = match row.priority {
                FindingPriority::Blocking => 0,
                FindingPriority::ReviewRequired => 1,
                FindingPriority::Informational => 2,
            };
            priorities[slot] = priorities[slot].checked_add(1).ok_or_else(|| ledger.capacity())?;
        }
    }
    rows.truncate(retained);
    let unreferenced = &document.impact.unreferenced_finding_ids;
    for (index, id) in unreferenced.iter().enumerate() {
        step(ledger, control)?;
        if index > 0 && compared(&unreferenced[index - 1], id, ledger)? != Ordering::Less {
            return Err(ContractError::Invalid);
        }
        match locate(&rows, id, |row| &row.finding_id, ledger, control) {
            Ok(_) => return Err(ContractError::Invalid),
            Err(ContractError::Binding) => {}
            Err(error) => return Err(error),
        }
    }
    let full = count(add(retained, unreferenced.len(), ledger)?, ledger)?;
    if full != document.impact.finding_count {
        return Err(ContractError::Invalid);
    }
    let summary = &document.impact.summary;
    let totals = [summary.blocking, summary.review_required, summary.informational];
    for (linked, total) in priorities.into_iter().zip(totals) {
        ledger.visits(1)?;
        ledger.bytes(16)?;
        if linked > total || (unreferenced.is_empty() && linked != total) {
            return Err(ContractError::Invalid);
        }
    }
    Ok(retained)
}

/// Precharge exact full finding rows before checking all eight repeated fields.
fn finding_equal(
    a: &FindingReference,
    b: &FindingReference,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    cost(a, ledger, control)?;
    cost(b, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    Ok(a.finding_id == b.finding_id
        && a.priority == b.priority
        && a.reason_code == b.reason_code
        && a.required_action == b.required_action
        && a.native_subject_id == b.native_subject_id
        && a.change_class == b.change_class
        && a.old_dependency_binding == b.old_dependency_binding
        && a.new_relation == b.new_relation)
}

/// Correlate every item/edge/finding denominator using checked complete totals.
fn counts(
    document: &SupersessionDocument,
    old: usize,
    new: usize,
    occurrences: usize,
    distinct: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(10)?;
    ledger.bytes(20 * std::mem::size_of::<u64>())?;
    let c = &document.counts;
    let expected = [
        document.old_queue.item_count,
        document.new_queue.item_count,
        count(document.links.len(), ledger)?,
        count(old, ledger)?,
        count(new, ledger)?,
        count(document.unmatched_old_items.len(), ledger)?,
        count(document.unmatched_new_items.len(), ledger)?,
        count(occurrences, ledger)?,
        count(distinct, ledger)?,
        count(document.impact.unreferenced_finding_ids.len(), ledger)?,
    ];
    let actual = [
        c.old_items,
        c.new_items,
        c.link_edges,
        c.linked_old_items,
        c.linked_new_items,
        c.unmatched_old_items,
        c.unmatched_new_items,
        c.link_finding_occurrences,
        c.distinct_linked_findings,
        c.unreferenced_findings,
    ];
    if actual != expected
        || c.distinct_linked_findings != document.impact.distinct_linked_findings
        || sum(&[c.linked_old_items, c.unmatched_old_items], ledger)? != c.old_items
        || sum(&[c.linked_new_items, c.unmatched_new_items], ledger)? != c.new_items
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Correlate all fifteen native summary cells without inventing omitted native detail.
fn summary(impact: &ImpactReference, ledger: &mut ContractLedger) -> Result<(), ContractError> {
    ledger.visits(15)?;
    ledger.bytes(30 * std::mem::size_of::<u64>())?;
    let s = &impact.summary;
    let base = sum(&[s.added, s.removed, s.content_changed, s.identity_migrated], ledger)?;
    if count(impact.unreferenced_finding_ids.len(), ledger)? < base {
        return Err(ContractError::Invalid);
    }
    let changes =
        sum(&[s.added, s.removed, s.content_changed, s.identity_migrated, s.unchanged], ledger)?;
    let old_min = sum(&[s.removed, s.content_changed, s.identity_migrated, s.unchanged], ledger)?;
    let new_min = sum(&[s.added, s.content_changed, s.identity_migrated, s.unchanged], ledger)?;
    let priority = sum(&[s.blocking, s.review_required, s.informational], ledger)?;
    let disposition = sum(
        &[
            s.dispositioned_resolved,
            s.dispositioned_accepted_risk,
            s.dispositioned_still_open,
            s.undispositioned,
        ],
        ledger,
    )?;
    if changes != impact.change_count
        || old_min > s.old_controls
        || new_min > s.new_controls
        || (s.identity_migrated == 0 && (old_min != s.old_controls || new_min != s.new_controls))
        || s.findings != impact.finding_count
        || s.findings != impact.matched_findings
        || priority != s.findings
        || disposition != s.findings
    {
        return Err(ContractError::Invalid);
    }
    let zero = s.added == 0
        && s.removed == 0
        && s.content_changed == 0
        && s.identity_migrated == 0
        && impact.finding_count == 0;
    if zero != (impact.native_change_observation == NativeChangeObservation::NoneDetected) {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Checked counter sum; overflowing declarations never become partial counts.
fn sum(values: &[u64], ledger: &mut ContractLedger) -> Result<u64, ContractError> {
    values.iter().try_fold(0u64, |total, value| {
        ledger.visits(1)?;
        ledger.bytes(16)?;
        total.checked_add(*value).ok_or_else(|| ledger.capacity())
    })
}

/// Validate exact global occurrence keys and the complete maintained recorded role sequence.
fn impact_sources(
    document: &SupersessionDocument,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let impact = &document.impact;
    let pins = &impact.source_pins;
    ledger.visits(1)?;
    ledger.bytes(std::mem::size_of::<u64>())?;
    if impact.old.resource_type != impact.new.resource_type
        || impact.locator.byte_length == 0
        || impact.locator.byte_length > 1_048_576
    {
        return Err(ContractError::Invalid);
    }
    for (ordinal, pin) in pins.iter().enumerate() {
        step(ledger, control)?;
        cost(pin, ledger, control)?;
        let (role, number) = role_key(&pin.key)?;
        if number != ordinal {
            return Err(ContractError::Invalid);
        }
        let (kind, schema) = role_family(role)?;
        if kind != pin.kind || schema != pin.schema_identity || pin.byte_length > role_limit(role) {
            return Err(ContractError::Invalid);
        }
    }
    let mut position = 0;
    let manifest = take(pins, &mut position, "manifest", ledger, control)?;
    let report = take(pins, &mut position, "current-report", ledger, control)?;
    original_equal(&impact.manifest, manifest, ledger, control)?;
    original_equal(&impact.report, report, ledger, control)?;
    let old = framework_roles(pins, &mut position, &impact.old, true, ledger, control)?;
    let new = framework_roles(pins, &mut position, &impact.new, false, ledger, control)?;
    while next_role(pins, position, ledger, control)? == Some("mapping") {
        take(pins, &mut position, "mapping", ledger, control)?;
    }
    if next_role(pins, position, ledger, control)? == Some("applicability-manifest") {
        take(pins, &mut position, "applicability-manifest", ledger, control)?;
        let role = match impact.old.resource_type {
            ImpactResourceType::Catalog => "applicability-catalog",
            ImpactResourceType::Profile => "applicability-profile",
        };
        let framework = take(pins, &mut position, role, ledger, control)?;
        member_equal(framework, old.0, ledger, control)?;
        if let Some(old_resolved) = old.1 {
            let resolved = take(pins, &mut position, "applicability-resolved", ledger, control)?;
            member_equal(resolved, old_resolved, ledger, control)?;
        }
        while next_role(pins, position, ledger, control)? == Some("applicability-mapping") {
            take(pins, &mut position, "applicability-mapping", ledger, control)?;
        }
    }
    if next_role(pins, position, ledger, control)? == Some("successor") {
        take(pins, &mut position, "successor", ledger, control)?;
    }
    if next_role(pins, position, ledger, control)? == Some("prior-report") {
        take(pins, &mut position, "prior-report", ledger, control)?;
        take(pins, &mut position, "dispositions", ledger, control)?;
    } else if impact.prior_only_disposition_count != 0 {
        return Err(ContractError::Invalid);
    }
    if position != pins.len() {
        return Err(ContractError::Invalid);
    }
    framework_queue(
        &document.old_queue,
        &impact.old,
        &impact.old_framework_source_key,
        impact.old_resolved_source_key.as_deref(),
        old,
        ledger,
        control,
    )?;
    framework_queue(
        &document.new_queue,
        &impact.new,
        &impact.new_framework_source_key,
        impact.new_resolved_source_key.as_deref(),
        new,
        ledger,
        control,
    )?;
    Ok(())
}

/// Parse the exact producer grammar without constructing an alternate occurrence key.
fn role_key(key: &str) -> Result<(&str, usize), ContractError> {
    let remaining = key.strip_prefix("impact:").ok_or(ContractError::Invalid)?;
    let (role, ordinal) = remaining.split_once(':').ok_or(ContractError::Invalid)?;
    if ordinal.is_empty()
        || (ordinal.len() > 1 && ordinal.starts_with('0'))
        || !ordinal.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ContractError::Invalid);
    }
    let number = ordinal.parse::<usize>().map_err(|_| ContractError::Invalid)?;
    Ok((role, number))
}

/// Exact seventeen-role producer vocabulary; schema/null pairings stay closed.
fn role_family(role: &str) -> Result<(ImpactSourceKind, ImpactSchemaIdentity), ContractError> {
    use ImpactSchemaIdentity as S;
    use ImpactSourceKind as K;
    let pair = match role {
        "manifest" => (K::ImpactManifest, S::Manifest),
        "current-report" | "prior-report" => (K::ImpactReport, S::Report),
        "old-catalog" | "new-catalog" | "applicability-catalog" => (K::Catalog, S::Catalog),
        "old-profile" | "new-profile" | "applicability-profile" => (K::Profile, S::Profile),
        "old-resolved" | "new-resolved" | "applicability-resolved" => {
            (K::ResolvedCatalog, S::Catalog)
        }
        "mapping" | "applicability-mapping" => (K::Mapping, S::Mapping),
        "applicability-manifest" => (K::ApplicabilityManifest, S::Applicability),
        "successor" => (K::SuccessorMap, S::Successor),
        "dispositions" => (K::Dispositions, S::Dispositions),
        _ => return Err(ContractError::Invalid),
    };
    Ok(pair)
}

/// Preserve stricter actual native companion ceilings within the ten-MiB Source profile.
fn role_limit(role: &str) -> u64 {
    match role {
        "manifest" => crate::framework::manifest::MAX_MANIFEST_BYTES,
        "successor" => 2_097_152, // Exact retained private successor parser ceiling; no visibility change.
        "dispositions" => crate::framework::disposition::MAX_DISPOSITION_BYTES,
        _ => 10_485_760,
    }
}

/// Admit each actual repeated key probe; no absent route is inferred.
fn next_role<'a>(
    pins: &'a [ImpactSourcePin],
    position: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Option<&'a str>, ContractError> {
    step(ledger, control)?;
    let Some(pin) = pins.get(position) else {
        return Ok(None);
    };
    ledger.bytes(pin.key.len())?;
    Ok(Some(role_key(&pin.key)?.0))
}

/// Consume exactly one required ordered recorded occurrence.
fn take<'a>(
    pins: &'a [ImpactSourcePin],
    position: &mut usize,
    expected: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a ImpactSourcePin, ContractError> {
    step(ledger, control)?;
    let pin = pins.get(*position).ok_or(ContractError::Invalid)?;
    ledger.bytes(pin.key.len())?;
    let (role, _) = role_key(&pin.key)?;
    if compared(role, expected, ledger)? != Ordering::Equal {
        return Err(ContractError::Invalid);
    }
    *position = add(*position, 1, ledger)?;
    Ok(pin)
}

/// Correlate recorded manifest/report raw pins with their complete source occurrences.
fn original_equal(
    original: &RecordedOriginalPin,
    pin: &ImpactSourcePin,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    cost(original, ledger, control)?;
    cost(pin, ledger, control)?;
    if original.byte_length != pin.byte_length
        || compared(&original.raw_sha256, &pin.raw_sha256, ledger)? != Ordering::Equal
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Check complete native pin data across compatible repeated framework occurrences.
fn member_equal(
    a: &ImpactSourcePin,
    b: &ImpactSourcePin,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    cost(a, ledger, control)?;
    cost(b, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    if a.kind != b.kind
        || a.schema_identity != b.schema_identity
        || a.native_root_uuid != b.native_root_uuid
        || a.raw_sha256 != b.raw_sha256
        || a.byte_length != b.byte_length
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Consume explicit Catalog/Profile companions and preserve exact native UUID spelling.
fn framework_roles<'a>(
    pins: &'a [ImpactSourcePin],
    position: &mut usize,
    resource: &ImpactResource,
    old: bool,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(&'a ImpactSourcePin, Option<&'a ImpactSourcePin>), ContractError> {
    let role = match (old, resource.resource_type) {
        (true, ImpactResourceType::Catalog) => "old-catalog",
        (false, ImpactResourceType::Catalog) => "new-catalog",
        (true, ImpactResourceType::Profile) => "old-profile",
        (false, ImpactResourceType::Profile) => "new-profile",
    };
    let native = take(pins, position, role, ledger, control)?;
    cost(resource, ledger, control)?;
    cost(native, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    if native.raw_sha256 != resource.raw_sha256
        || native.native_root_uuid.as_deref() != Some(resource.root_uuid.as_str())
    {
        return Err(ContractError::Invalid);
    }
    let resolved = if resource.resource_type == ImpactResourceType::Profile {
        let pin = take(
            pins,
            position,
            if old { "old-resolved" } else { "new-resolved" },
            ledger,
            control,
        )?;
        if compared(
            &pin.raw_sha256,
            resource.resolved_catalog_sha256.as_deref().ok_or(ContractError::Invalid)?,
            ledger,
        )? != Ordering::Equal
        {
            return Err(ContractError::Invalid);
        }
        Some(pin)
    } else {
        None
    };
    Ok((native, resolved))
}

/// Compare each explicitly selected queue framework pin with its recorded native occurrence.
fn framework_queue(
    queue: &RecordedQueueReference,
    resource: &ImpactResource,
    key: &str,
    resolved_key: Option<&str>,
    members: (&ImpactSourcePin, Option<&ImpactSourcePin>),
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let pin = locate(&queue.source_pins, key, |row| &row.artifact_key, ledger, control)
        .map_err(|error| ordinary(error, ContractError::Invalid))?;
    let model = match resource.resource_type {
        ImpactResourceType::Catalog => SourceModel::Catalog,
        ImpactResourceType::Profile => SourceModel::Profile,
    };
    queue_native_equal(pin, members.0, model, ledger, control)?;
    match (resource.resource_type, resolved_key, members.1) {
        (ImpactResourceType::Catalog, None, None) => {}
        (ImpactResourceType::Profile, Some(key), Some(member)) => {
            let resolved =
                locate(&queue.source_pins, key, |row| &row.artifact_key, ledger, control)
                    .map_err(|error| ordinary(error, ContractError::Invalid))?;
            queue_native_equal(resolved, member, SourceModel::ResolvedCatalog, ledger, control)?;
        }
        _ => return Err(ContractError::Invalid),
    }
    Ok(())
}

/// Preserve whole hash/length/family and identity values without version/href fallback.
fn queue_native_equal(
    queue: &RecordedQueueSourcePin,
    native: &ImpactSourcePin,
    model: SourceModel,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    cost(queue, ledger, control)?;
    cost(native, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    let schema = match native.schema_identity {
        ImpactSchemaIdentity::Catalog => "oscal:1.2.3:catalog",
        ImpactSchemaIdentity::Profile => "oscal:1.2.3:profile",
        _ => return Err(ContractError::Invalid),
    };
    if queue.model != model
        || queue.raw_sha256 != native.raw_sha256
        || queue.byte_length != native.byte_length
        || queue.native_root_uuid != native.native_root_uuid
        || compared(&queue.schema_identity, schema, ledger)? != Ordering::Equal
    {
        return Err(ContractError::Invalid);
    }
    Ok(())
}

/// Bind inert declarations to the exact two complete original queue data objects.
///
/// # Errors
/// Every raw/reference/item/difference mismatch refuses. First actual caller
/// capacity/control stops dominate the final success and ordinary failure fence.
/// A successful result confers no native approval, captured owner or currentness.
pub(crate) fn bind_queues(
    recorded: &RecordedSupersession<'_>,
    old: &Decoded<'_, QueueDocument>,
    new: &Decoded<'_, QueueDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let result = bind_inner(recorded.document(), old, new, ledger, control);
        ledger.checkpoint(control)?;
        result
    })
}

/// Resolve the complete actual old/new item rosters and recompute typed differences.
fn bind_inner(
    recorded: &SupersessionDocument,
    old: &Decoded<'_, QueueDocument>,
    new: &Decoded<'_, QueueDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    bind_reference(&recorded.old_queue, old, ledger, control)?;
    bind_reference(&recorded.new_queue, new, ledger, control)?;
    for queue in [old.document(), new.document()] {
        if compared(&recorded.created_at, &queue.created_at, ledger)? == Ordering::Less {
            return Err(ContractError::Binding);
        }
    }
    let old_order = item_order(&old.document().items, ledger, control)?;
    let new_order = item_order(&new.document().items, ledger, control)?;
    let mut old_used = membership(old.document().items.len(), ledger, control)?;
    let mut new_used = membership(new.document().items.len(), ledger, control)?;
    for link in &recorded.links {
        step(ledger, control)?;
        let left =
            locate_item(&old.document().items, &old_order, &link.old.item_id, ledger, control)?;
        let right =
            locate_item(&new.document().items, &new_order, &link.new.item_id, ledger, control)?;
        let a = &old.document().items[left];
        let b = &new.document().items[right];
        bind_endpoint(&link.old, a, ledger, control)?;
        bind_endpoint(&link.new, b, ledger, control)?;
        let actual = differences(old.document(), a, new.document(), b, ledger, control)?;
        ledger.bytes(2 * std::mem::size_of::<Differences>())?;
        ledger.visits(1)?;
        if actual != link.differences {
            return Err(ContractError::Binding);
        }
        ledger.bytes(2 * std::mem::size_of::<bool>())?;
        old_used[left] = true;
        new_used[right] = true;
    }
    bind_unmatched(
        &recorded.unmatched_old_items,
        &old.document().items,
        &old_used,
        ledger,
        control,
    )?;
    bind_unmatched(
        &recorded.unmatched_new_items,
        &new.document().items,
        &new_used,
        ledger,
        control,
    )?;
    Ok(())
}

/// Match all raw/reference fields and the entire ordered declared `SourcePin` roster.
fn bind_reference(
    recorded: &RecordedQueueReference,
    actual: &Decoded<'_, QueueDocument>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    step(ledger, control)?;
    cost(recorded, ledger, control)?;
    cost(&actual.document().source_pins, ledger, control)?;
    let queue = actual.document();
    if compared(&queue.schema_version, "forge.review-queue/1", ledger)? != Ordering::Equal
        || compared(&recorded.queue_id, &queue.queue_id, ledger)? != Ordering::Equal
        || compared(&recorded.raw_sha256, actual.raw_sha256(), ledger)? != Ordering::Equal
        || recorded.byte_length != count(actual.raw().len(), ledger)?
        || recorded.item_count != count(queue.items.len(), ledger)?
        || recorded.source_pins.len() != queue.source_pins.len()
    {
        return Err(ContractError::Binding);
    }
    for (a, b) in recorded.source_pins.iter().zip(&queue.source_pins) {
        step(ledger, control)?;
        // Complete roster serialization above covers all six compared fields.
        ledger.matching(1)?;
        if a.artifact_key != b.artifact_key
            || a.model != b.model
            || a.native_root_uuid != b.native_root_uuid
            || a.raw_sha256 != b.raw_sha256
            || a.byte_length != b.byte_length
            || a.schema_identity != b.schema_identity
        {
            return Err(ContractError::Binding);
        }
    }
    Ok(())
}

/// Precharge full endpoint rows before exact eleven-field repeated equality.
fn endpoint_equal(
    a: &RecordedEndpoint,
    b: &RecordedEndpoint,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    cost(a, ledger, control)?;
    cost(b, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    Ok(a.key == b.key
        && a.item_id == b.item_id
        && a.domain == b.domain
        && a.adapter_version == b.adapter_version
        && a.subject_id == b.subject_id
        && a.subject_sha256 == b.subject_sha256
        && a.context_sha256 == b.context_sha256
        && a.policy_key == b.policy_key
        && a.policy_sha256 == b.policy_sha256
        && a.requested_action == b.requested_action
        && a.source_keys == b.source_keys)
}

/// Match the endpoint whitelist against the actual complete item; no digest-only fallback.
fn bind_endpoint(
    recorded: &RecordedEndpoint,
    actual: &ReviewItem,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    cost(recorded, ledger, control)?;
    cost(actual, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    if recorded.key != actual.key
        || recorded.item_id != actual.item_id
        || recorded.domain != actual.domain
        || recorded.adapter_version != actual.adapter_version
        || recorded.subject_id != actual.subject_id
        || recorded.subject_sha256 != actual.subject_sha256
        || recorded.context_sha256 != actual.context_sha256
        || recorded.policy_key != actual.policy_key
        || recorded.policy_sha256 != actual.policy_sha256
        || recorded.requested_action != actual.requested_action
        || recorded.source_keys != actual.source_keys
    {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Reserve actual queue-wide membership cells before initialization.
fn membership(
    length: usize,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<bool>, ContractError> {
    let mut used = vector(length, ledger)?;
    for _ in 0..length {
        step(ledger, control)?;
        ledger.bytes(std::mem::size_of::<bool>())?;
        used.push(false);
    }
    Ok(used)
}

/// Check complete unmatched rows in original queue item order with no omissions.
fn bind_unmatched(
    recorded: &[RecordedEndpoint],
    actual: &[ReviewItem],
    used: &[bool],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    if actual.len() != used.len() {
        return Err(ContractError::Binding);
    }
    let mut position = 0;
    for (item, linked) in actual.iter().zip(used) {
        step(ledger, control)?;
        if !linked {
            let row = recorded.get(position).ok_or(ContractError::Binding)?;
            bind_endpoint(row, item, ledger, control)?;
            position = add(position, 1, ledger)?;
        }
    }
    if position != recorded.len() {
        return Err(ContractError::Binding);
    }
    Ok(())
}

/// Admit one real structural/control step before inspection.
fn step(ledger: &mut ContractLedger, control: &mut dyn WorkControl) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)
}

/// Convert a complete extent to its fixed wire counter without truncation.
fn count(value: usize, ledger: &mut ContractLedger) -> Result<u64, ContractError> {
    ledger.visits(1)?;
    ledger.bytes(std::mem::size_of::<usize>())?;
    u64::try_from(value).map_err(|_| ledger.capacity())
}

/// Preserve actual sticky stops while mapping an ordinary membership refusal.
fn ordinary(error: ContractError, replacement: ContractError) -> ContractError {
    match error {
        ContractError::Capacity | ContractError::Interrupted(_) | ContractError::ControlFailed => {
            error
        }
        _ => replacement,
    }
}

/// Complete actual subject comparison, matching the declared four-field rule.
#[derive(Serialize, PartialEq, Eq)]
struct Subject<'a> {
    /// Native review family.
    domain: Domain,
    /// Complete actual adapter marker.
    adapter: &'a str,
    /// Exact unnormalized subject ID.
    id: &'a str,
    /// Exact recorded review fingerprint.
    hash: &'a str,
}

/// Complete input vocabulary of the unchanged `ContextEncoding` profile.
#[derive(Serialize, PartialEq, Eq)]
struct ContextInput<'a> {
    /// Actual declared family.
    domain: Domain,
    /// Exact adapter marker.
    adapter: &'a str,
    /// Entire queue pin roster bound by existing context hashing, not just item keys.
    pins: &'a [SourcePin],
    /// Complete minimized typed context, not a display hash alone.
    context: &'a ContextSnapshot,
}

/// Complete input vocabulary of the unchanged `PolicyEncoding` profile.
#[derive(Serialize, PartialEq, Eq)]
struct PolicyInput<'a> {
    /// Actual declared family.
    domain: Domain,
    /// Exact adapter marker.
    adapter: &'a str,
    /// Entire queue pin roster bound by existing policy hashing.
    pins: &'a [SourcePin],
    /// Complete selected policy including its key and substitutions.
    policy: &'a ReviewPolicy,
    /// Complete exact sorted declared authors.
    authors: &'a [String],
    /// Complete exact ordered assignments.
    assignments: &'a [super::wire::Assignment],
    /// Exact asserted deadline or null.
    due: &'a Option<String>,
    /// Entire dissent-preserving disposition vocabulary.
    allowed: &'a [super::wire::Disposition],
}

/// Derive complete typed differences after all comparison extents are admitted.
fn differences(
    old: &QueueDocument,
    left: &ReviewItem,
    new: &QueueDocument,
    right: &ReviewItem,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Differences, ContractError> {
    let subject = differs(
        &Subject {
            domain: left.domain,
            adapter: &left.adapter_version,
            id: &left.subject_id,
            hash: &left.subject_sha256,
        },
        &Subject {
            domain: right.domain,
            adapter: &right.adapter_version,
            id: &right.subject_id,
            hash: &right.subject_sha256,
        },
        ledger,
        control,
    )?;
    let context = differs(
        &ContextInput {
            domain: left.domain,
            adapter: &left.adapter_version,
            pins: &old.source_pins,
            context: &left.context,
        },
        &ContextInput {
            domain: right.domain,
            adapter: &right.adapter_version,
            pins: &new.source_pins,
            context: &right.context,
        },
        ledger,
        control,
    )?;
    let old_policy =
        locate(&old.policies, &left.policy_key, |row| row.key.as_str(), ledger, control)?;
    let new_policy =
        locate(&new.policies, &right.policy_key, |row| row.key.as_str(), ledger, control)?;
    let policy = differs(
        &PolicyInput {
            domain: left.domain,
            adapter: &left.adapter_version,
            pins: &old.source_pins,
            policy: old_policy,
            authors: &left.author_keys,
            assignments: &left.assignments,
            due: &left.due_at,
            allowed: &left.allowed_dispositions,
        },
        &PolicyInput {
            domain: right.domain,
            adapter: &right.adapter_version,
            pins: &new.source_pins,
            policy: new_policy,
            authors: &right.author_keys,
            assignments: &right.assignments,
            due: &right.due_at,
            allowed: &right.allowed_dispositions,
        },
        ledger,
        control,
    )?;
    let source_pins = source_difference(old, left, new, right, ledger, control)?;
    Ok(Differences { subject, context, policy, source_pins })
}

/// Compare every corresponding selected `SourcePin` value, preserving separate roster identity.
fn source_difference(
    old: &QueueDocument,
    left: &ReviewItem,
    new: &QueueDocument,
    right: &ReviewItem,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let mut different = left.source_keys.len() != right.source_keys.len();
    for index in 0..left.source_keys.len().max(right.source_keys.len()) {
        ledger.checkpoint(control)?;
        let a = left
            .source_keys
            .get(index)
            .map(|key| {
                locate(&old.source_pins, key, |pin| pin.artifact_key.as_str(), ledger, control)
            })
            .transpose()?;
        let b = right
            .source_keys
            .get(index)
            .map(|key| {
                locate(&new.source_pins, key, |pin| pin.artifact_key.as_str(), ledger, control)
            })
            .transpose()?;
        match (a, b) {
            (Some(a), Some(b)) => {
                different |= differs(a, b, ledger, control)?;
            }
            (Some(value), None) | (None, Some(value)) => {
                cost(value, ledger, control)?;
                different = true;
            }
            (None, None) => return Err(ContractError::Binding),
        }
    }
    Ok(different)
}

/// Stream actual closed typed comparison extents without retaining serialized text.
struct CostWriter<'a> {
    /// The same caller ledger, with no reset or comparison allowance.
    ledger: &'a mut ContractLedger,
    /// The same actual accepted cooperative control.
    control: &'a mut dyn WorkControl,
    /// Precise first admission/control failure through serde's IO wrapper.
    error: Option<ContractError>,
}

impl Write for CostWriter<'_> {
    /// Admit actual serializer work plus a conservative complete subsequent typed comparison.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let result = self.ledger.bound(|ledger| {
            ledger.checkpoint(self.control)?;
            ledger.visits(1)?;
            let extent = add(mul(bytes.len(), 2, ledger)?, 64, ledger)?;
            ledger.bytes(extent)
        });
        if let Err(error) = result {
            self.error.get_or_insert(error);
            return Err(io::Error::other("review comparison refused"));
        }
        Ok(bytes.len())
    }
    /// No retained output exists; final fence is owned by the enclosing phase.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Charge complete typed output and full comparison work, then fence success and errors.
fn cost<T: Serialize + ?Sized>(
    value: &T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let result = {
        let mut writer = CostWriter { ledger, control, error: None };
        let serialized = serde_json::to_writer(&mut writer, value);
        if serialized.is_err() {
            Err(writer.error.unwrap_or(ContractError::Invalid))
        } else {
            Ok(())
        }
    };
    ledger.checkpoint(control)?;
    result
}

/// Compare exact complete values only after both complete sides have been precharged.
fn differs<T: Serialize + PartialEq + ?Sized>(
    left: &T,
    right: &T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    cost(left, ledger, control)?;
    cost(right, ledger, control)?;
    ledger.visits(1)?;
    ledger.matching(1)?;
    Ok(left != right)
}

/// Build a complete item UUID index with admitted fallible in-place deterministic sorting.
fn item_order(
    items: &[ReviewItem],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<usize>, ContractError> {
    let mut order = vector::<usize>(items.len(), ledger)?;
    for index in 0..items.len() {
        ledger.visits(1)?;
        ledger.bytes(std::mem::size_of::<usize>())?;
        order.push(index);
    }
    ordered(
        &mut order,
        &mut |a, b, ledger| compared(&items[a].item_id, &items[b].item_id, ledger),
        ledger,
        control,
    )?;
    Ok(order)
}

/// Resolve the exact requested UUID by charged binary lookup in the full item index.
fn locate_item(
    items: &[ReviewItem],
    order: &[usize],
    key: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<usize, ContractError> {
    let mut lower = 0;
    let mut upper = order.len();
    while lower < upper {
        ledger.checkpoint(control)?;
        let mid = lower + (upper - lower) / 2;
        match compared(&items[order[mid]].item_id, key, ledger)? {
            Ordering::Less => lower = mid + 1,
            Ordering::Greater => upper = mid,
            Ordering::Equal => return Ok(order[mid]),
        }
    }
    Err(ContractError::Binding)
}

/// Resolve sorted actual policy/source keys without an inferred fallback or full Cartesian scan.
fn locate<'a, T>(
    rows: &'a [T],
    key: &str,
    key_of: impl Fn(&T) -> &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<&'a T, ContractError> {
    let mut lower = 0;
    let mut upper = rows.len();
    while lower < upper {
        ledger.checkpoint(control)?;
        let mid = lower + (upper - lower) / 2;
        match compared(key_of(&rows[mid]), key, ledger)? {
            Ordering::Less => lower = mid + 1,
            Ordering::Greater => upper = mid,
            Ordering::Equal => return Ok(&rows[mid]),
        }
    }
    Err(ContractError::Binding)
}

/// Precharge both complete original string extents for each actual comparison.
fn compared(
    left: &str,
    right: &str,
    ledger: &mut ContractLedger,
) -> Result<Ordering, ContractError> {
    ledger.visits(1)?;
    ledger.matching(1)?;
    let extent = add(left.len(), right.len(), ledger)?;
    ledger.bytes(extent)?;
    Ok(left.cmp(right))
}

/// Fallible heapsort; every real comparison and index/reference swap is admitted first.
fn ordered<T: Copy>(
    rows: &mut [T],
    compare: &mut impl FnMut(T, T, &mut ContractLedger) -> Result<Ordering, ContractError>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    for start in (0..rows.len() / 2).rev() {
        sift(rows, start, rows.len(), compare, ledger, control)?;
    }
    for end in (1..rows.len()).rev() {
        swapped(rows, 0, end, ledger)?;
        sift(rows, 0, end, compare, ledger, control)?;
    }
    Ok(())
}

/// Restore a bounded max heap with fallible real comparisons and no allocation.
fn sift<T: Copy>(
    rows: &mut [T],
    mut parent: usize,
    end: usize,
    compare: &mut impl FnMut(T, T, &mut ContractLedger) -> Result<Ordering, ContractError>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    loop {
        ledger.checkpoint(control)?;
        let mut child = add(mul(parent, 2, ledger)?, 1, ledger)?;
        if child >= end {
            return Ok(());
        }
        let sibling = add(child, 1, ledger)?;
        if sibling < end && compare(rows[child], rows[sibling], ledger)? == Ordering::Less {
            child = sibling;
        }
        if compare(rows[parent], rows[child], ledger)? != Ordering::Less {
            return Ok(());
        }
        swapped(rows, parent, child, ledger)?;
        parent = child;
    }
}

/// Charge complete fixed-width copy work before every actual swap.
fn swapped<T>(
    rows: &mut [T],
    a: usize,
    b: usize,
    ledger: &mut ContractLedger,
) -> Result<(), ContractError> {
    ledger.visits(1)?;
    let extent = mul(std::mem::size_of::<T>(), 3, ledger)?;
    ledger.bytes(extent)?;
    rows.swap(a, b);
    Ok(())
}

/// Reserve complete owned vector payload before growth; borrowed strings are not copied.
fn vector<T>(count: usize, ledger: &mut ContractLedger) -> Result<Vec<T>, ContractError> {
    let extent =
        add(std::mem::size_of::<Vec<T>>(), mul(count, std::mem::size_of::<T>(), ledger)?, ledger)?;
    ledger.derived(extent)?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(count).map_err(|_| ledger.capacity())?;
    Ok(rows)
}

/// Checked sum whose actual first capacity refusal latches before any later fence.
fn add(left: usize, right: usize, ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    left.checked_add(right).ok_or_else(|| ledger.capacity())
}
/// Checked product with the same actual first-stop semantics.
fn mul(left: usize, right: usize, ledger: &mut ContractLedger) -> Result<usize, ContractError> {
    left.checked_mul(right).ok_or_else(|| ledger.capacity())
}

#[cfg(test)]
#[path = "supersession_decode_tests.rs"]
/// Prospective recorded-data/refusal controls; no native/currentness acceptance.
mod tests;

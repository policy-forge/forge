//! Redacted lifecycle inspection over a single complete registered byte capture.
//!
//! No production helper receives a Root, reads a path, samples a date, writes an
//! effect, authenticates an actor or grants publication authority. Schema validity
//! remains distinct from the CLI-compatible captured model/UUID status facts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::NaiveDate;
use serde_json::{Value, json};

use crate::lifecycle::portfolio;
use crate::lifecycle::record::{self, FingerprintSet, LifecycleRecord, NamedHash};
use crate::lifecycle::status::{CurrentArtifacts, StatusReport, status_from_captured};

use super::contract::{Error, Result};
use super::index::{Role, validate_path};
use super::inspection::{self, Query};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};
use super::services::{Item, Snapshot, resource_id};

/// Existing declared-identity qualification, independent of captured status success.
const TRUST_BOUNDARY: &str =
    "actor identities and authority are declared locally and are not authenticated by FORGE";
/// Compact owner memberships retained before paging at most 200 projected rows.
const MAX_OWNER_PLACEMENTS: usize = 64_000;

/// One bounded record admission, carrying no duplicated prose in owner placements.
struct RecordRow {
    /// Authorial registration index, also used for raw captured provenance.
    item_index: usize,
    /// Intrinsically validated declared record; absent for a malformed inventory row.
    record: Option<LifecycleRecord>,
    /// Complete CLI-compatible raw hash/model/UUID observations, when identifiable.
    current: Option<CurrentArtifacts>,
    /// Ordered unique registration indices in the record's exact source/artifact closure.
    closure: Vec<usize>,
    /// Fixed valid/invalid/unavailable admission dimension, not a Ready claim.
    availability: &'static str,
    /// Fixed safe inventory reason with no parser/path text.
    diagnostic: Option<&'static str>,
    /// Explicit-date status; absent until the whole supplied portfolio succeeds.
    status: Option<StatusReport>,
}

/// Exact capture lookups and memoized root identity extraction, without path I/O.
struct CaptureFacts<'a> {
    /// Portable registered paths mapped once to their unique capture indices.
    indices: BTreeMap<&'a str, usize>,
    /// Observed native model/UUID, parsed once per generated captured resource.
    identities: Vec<Option<(String, String)>>,
}

/// Two small indices for one owner/date membership; no cloned record or report body.
#[derive(Clone, Copy)]
struct Placement {
    /// Index in the fully admitted, explicitly dated record collection.
    record: usize,
    /// Index of the exact declared owner key inside that record.
    owner: usize,
}

/// Keep the same cooperative stop typed before/after parsing and projection work.
fn fence(control: &mut dyn WorkControl) -> WorkResult<()> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
}

/// Return a fixed domain failure without retaining a private `ForgeError` string.
fn invalid() -> Error {
    inspection::domain_failure()
}

/// Consume the shared capture guard, then retain bounded borrowed indices and identity memo slots.
fn capture_facts<'a>(
    snapshot: &'a Snapshot,
    control: &mut dyn WorkControl,
) -> WorkResult<CaptureFacts<'a>> {
    inspection::validate_snapshot(snapshot, control)?;
    let indices = snapshot
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.registration.path.as_str(), index))
        .collect();
    Ok(CaptureFacts { indices, identities: vec![None; snapshot.items.len()] })
}

/// Enforce the lifecycle domain's stronger portable descendant spelling first.
fn strong_reference(raw: &str) -> Result<()> {
    if raw.is_empty()
        || raw.contains('\\')
        || raw.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(Error::containment());
    }
    validate_path(raw).map_err(|_| Error::containment())
}

/// Resolve one strongly confined role-admitted edge only inside the immutable capture.
fn captured_reference<'a>(
    snapshot: &'a Snapshot,
    base: &str,
    raw: &str,
    roles: &[Role],
) -> Result<&'a Item> {
    strong_reference(raw)?;
    inspection::reference(snapshot, base, Path::new(raw), roles)
}

/// Locate the exact capture index already checked for path and instance uniqueness.
fn capture_index(facts: &CaptureFacts<'_>, item: &Item) -> Result<usize> {
    facts.indices.get(item.registration.path.as_str()).copied().ok_or_else(invalid)
}

/// Extract exactly the five CLI model/root spellings and validate the observed UUID.
fn observed_identity(bytes: &[u8]) -> Result<(String, String)> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let model = crate::validate::detect_model_type(&value).map_err(|_| invalid())?;
    let root = match model {
        crate::OscalModelType::Catalog => "catalog",
        crate::OscalModelType::ComponentDefinition => "component-definition",
        crate::OscalModelType::Profile => "profile",
        crate::OscalModelType::SystemSecurityPlan => "system-security-plan",
        crate::OscalModelType::Mapping => "mapping-collection",
    };
    let uuid = value
        .get(root)
        .and_then(|value| value.get("uuid"))
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    uuid::Uuid::parse_str(uuid).map_err(|_| invalid())?;
    // Keep the raw string spelling as the existing CLI fingerprint does; UUID
    // parsing is validation rather than an undeclared identity normalization.
    Ok((model.as_str().to_owned(), uuid.to_owned()))
}

/// Capture exact registered source/generated facts; schema validity stays independent.
fn current_artifacts(
    snapshot: &Snapshot,
    item_index: usize,
    record: &LifecycleRecord,
    facts: &mut CaptureFacts<'_>,
    control: &mut dyn WorkControl,
) -> WorkResult<(CurrentArtifacts, Vec<usize>)> {
    fence(control)?;
    let base = &snapshot.items[item_index].registration.path;
    let source = captured_reference(
        snapshot,
        base,
        &record.policy.source.path,
        &[Role::LifecycleSource, Role::PolicySource],
    )?;
    let source_index = capture_index(facts, source)?;
    let mut closure = BTreeSet::from([item_index, source_index]);
    let mut generated = Vec::with_capacity(record.policy.generated_artifacts.len());
    let mut identity_changes = Vec::new();
    for expected in &record.policy.generated_artifacts {
        fence(control)?;
        let item = captured_reference(
            snapshot,
            base,
            &expected.path,
            &[
                Role::OscalCatalogArtifact,
                Role::OscalComponentArtifact,
                Role::OscalProfileArtifact,
                Role::OscalSspArtifact,
                Role::MappingCollection,
            ],
        )?;
        let index = capture_index(facts, item)?;
        closure.insert(index);
        if facts.identities[index].is_none() {
            let actual = observed_identity(&item.captured.bytes);
            fence(control)?;
            facts.identities[index] = Some(actual?);
        }
        let (kind, uuid) = facts.identities[index].as_ref().ok_or_else(invalid)?;
        if expected.oscal_type.as_ref() != Some(kind) || expected.root_uuid.as_ref() != Some(uuid) {
            identity_changes.push(expected.path.clone());
        }
        generated
            .push(NamedHash { path: expected.path.clone(), sha256: item.captured.sha256.clone() });
    }
    generated.sort();
    fence(control)?;
    Ok((
        CurrentArtifacts {
            fingerprints: FingerprintSet {
                source_sha256: source.captured.sha256.clone(),
                generated_artifacts: generated,
            },
            identity_changes,
        },
        closure.into_iter().collect(),
    ))
}

/// Admit every registered record before filtering; preserve stop versus input failure.
fn inventory(snapshot: &Snapshot, control: &mut dyn WorkControl) -> WorkResult<Vec<RecordRow>> {
    let mut facts = capture_facts(snapshot, control)?;
    let mut rows = Vec::new();
    for (item_index, item) in snapshot.items.iter().enumerate() {
        if item.registration.role != Role::LifecycleRecord {
            continue;
        }
        fence(control)?;
        let parsed = record::parse(&item.captured.bytes);
        fence(control)?;
        let mut row = RecordRow {
            item_index,
            record: None,
            current: None,
            closure: vec![item_index],
            availability: "invalid",
            diagnostic: Some("invalid-lifecycle-record"),
            status: None,
        };
        if let Ok(record) = parsed {
            match current_artifacts(snapshot, item_index, &record, &mut facts, control) {
                Ok((current, closure)) => {
                    row.current = Some(current);
                    row.closure = closure;
                    row.availability = "valid";
                    row.diagnostic = None;
                }
                Err(WorkError::Interrupted(reason)) => return Err(WorkError::Interrupted(reason)),
                Err(WorkError::Failed(error)) if error.code == "resource-containment" => {
                    return Err(error.into());
                }
                Err(WorkError::Failed(_)) => {
                    row.availability = "unavailable";
                    row.diagnostic = Some("lifecycle-input-unavailable");
                }
            }
            row.record = Some(record);
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Compute all records or fail the complete portfolio before any filtered result.
fn compute(
    rows: &mut [RecordRow],
    as_of: NaiveDate,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    fence(control)?;
    if rows.iter().any(|row| row.availability != "valid") {
        return Err(invalid().into());
    }
    let records = rows
        .iter()
        .map(|row| row.record.as_ref().ok_or_else(invalid))
        .collect::<Result<Vec<_>>>()?;
    let checked = portfolio::validate(&records);
    fence(control)?;
    checked.map_err(|_| invalid())?;
    for row in rows {
        fence(control)?;
        let record = row.record.as_ref().ok_or_else(invalid)?;
        let current = row.current.as_ref().ok_or_else(invalid)?;
        let status = status_from_captured(record, current, Some(as_of));
        fence(control)?;
        row.status = Some(status.map_err(|_| invalid())?);
    }
    Ok(())
}

/// Mark a structurally invalid complete recorded portfolio without derived status.
fn classify_recorded_portfolio(
    rows: &mut [RecordRow],
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    fence(control)?;
    if rows.iter().all(|row| row.record.is_some()) {
        let records = rows.iter().filter_map(|row| row.record.as_ref()).collect::<Vec<_>>();
        let checked = portfolio::validate(&records);
        fence(control)?;
        if checked.is_err() {
            for row in rows {
                row.availability = "invalid";
                row.diagnostic = Some("invalid-lifecycle-portfolio");
            }
        }
    }
    Ok(())
}

/// Combine fixed setup states with independent record and closure attention needs.
fn availability(snapshot: &Snapshot, rows: &[RecordRow]) -> &'static str {
    let base = inspection::family_availability(snapshot, rows.len());
    if base == "available"
        && rows.iter().any(|row| {
            row.availability != "valid"
                || row
                    .closure
                    .iter()
                    .any(|index| snapshot.items[*index].metadata["validation_state"] != "valid")
        })
    {
        "needs-attention"
    } else {
        base
    }
}

/// Project only a selected inventory row, with no title, rationale or source bytes.
fn record_item(snapshot: &Snapshot, row: &RecordRow) -> Value {
    let item = &snapshot.items[row.item_index];
    let id = resource_id(&item.registration);
    json!({"record_id":id,"record_version":item.captured.sha256,
        "resource_id":id,"policy_key":row.record.as_ref().map(|record| &record.policy.policy_key),
        "version_key":row.record.as_ref().map(|record| &record.policy.version_key),
        "state":row.record.as_ref().map(|record| record.state),
        "derived_status":row.status.as_ref().map(|status| &status.derived_status),
        "owner_keys":row.record.as_ref().map(|record| record.policy.owner_keys.clone()).unwrap_or_default(),
        "next_review_date":row.record.as_ref().map(|record| record.review.next_review_date),
        "availability":row.availability,"diagnostic_code":row.diagnostic,
        "validation_state":item.metadata["validation_state"]})
}

/// Validate the recorded-state filter independently of source contents and date.
fn state_filter(query: &Query<'_>) -> Result<()> {
    if query.optional("state").is_some_and(|state| {
        !matches!(state, "draft" | "in-review" | "approved" | "superseded" | "retired")
    }) {
        return Err(Error::invalid());
    }
    Ok(())
}

/// Select a role-correct registered resource identity without translating foreign IDs.
fn selected_index(snapshot: &Snapshot, selected: &str) -> Result<usize> {
    snapshot
        .items
        .iter()
        .position(|item| {
            item.registration.role == Role::LifecycleRecord
                && resource_id(&item.registration) == selected
        })
        .ok_or_else(|| {
            Error::new("not-found", "The registered lifecycle record was not found.", false)
        })
}

/// Emit a setup page without pretending that an absent/legacy family was computed.
fn setup_page(
    snapshot: &Snapshot,
    domain: &str,
    query: &Query<'_>,
    as_of: Option<NaiveDate>,
    counts: Value,
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    fence(control)?;
    let version = inspection::version(snapshot, domain, "", query);
    let mut value = inspection::page(0, &version, query, |_| Err(invalid()))?;
    value["snapshot_version"] = json!(snapshot.version);
    value["availability"] = json!(inspection::family_availability(snapshot, 0));
    value["counts"] = counts;
    value["as_of"] = json!(as_of);
    inspection::finish(value, control)
}

/// Read complete registered lifecycle inventory, optionally computing an explicit-date portfolio.
pub(crate) fn records(
    snapshot: &Snapshot,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    fence(control)?;
    let query = Query::new(raw, &["as_of", "owner", "state", "page_size", "cursor"])?;
    state_filter(&query)?;
    let as_of = query.date(false)?;
    if !snapshot.index_present || snapshot.index.schema_version != "forge.workspace/2" {
        return setup_page(
            snapshot,
            "/api/v2/lifecycle/records",
            &query,
            as_of,
            json!({"registered_records":0,"matching_records":0,"unavailable_records":0}),
            control,
        );
    }
    let mut rows = inventory(snapshot, control)?;
    if let Some(date) = as_of {
        compute(&mut rows, date, control)?;
    } else {
        classify_recorded_portfolio(&mut rows, control)?;
    }
    let matching = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let matches_owner = query.optional("owner").is_none_or(|owner| {
                row.record
                    .as_ref()
                    .is_some_and(|record| record.policy.owner_keys.iter().any(|key| key == owner))
            });
            let matches_state = query.optional("state").is_none_or(|state| {
                row.record.as_ref().is_some_and(|record| record.state.as_str() == state)
            });
            (matches_owner && matches_state).then_some(index)
        })
        .collect::<Vec<_>>();
    let version = inspection::version(snapshot, "/api/v2/lifecycle/records", "", &query);
    fence(control)?;
    let mut value = inspection::page(matching.len(), &version, &query, |index| {
        Ok(record_item(snapshot, &rows[matching[index]]))
    })?;
    fence(control)?;
    value["snapshot_version"] = json!(snapshot.version);
    value["availability"] = json!(availability(snapshot, &rows));
    value["counts"] = json!({"registered_records":rows.len(),"matching_records":matching.len(),
        "unavailable_records":rows.iter().filter(|row| row.availability != "valid").count()});
    value["as_of"] = json!(as_of);
    inspection::finish(value, control)
}

/// Read one complete redacted status while validating the entire captured portfolio.
pub(crate) fn detail(
    snapshot: &Snapshot,
    selected: &str,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    fence(control)?;
    let query = Query::new(raw, &["as_of"])?;
    let as_of = query.date(true)?.ok_or_else(Error::invalid)?;
    inspection::require_index2(snapshot)?;
    let selected_index = selected_index(snapshot, selected)?;
    let mut rows = inventory(snapshot, control)?;
    compute(&mut rows, as_of, control)?;
    let row = rows.iter().find(|row| row.item_index == selected_index).ok_or_else(invalid)?;
    let record = row.record.as_ref().ok_or_else(invalid)?;
    let status = row.status.as_ref().ok_or_else(invalid)?;
    let replacement_id = record
        .replaced_by
        .as_ref()
        .map(|replacement| {
            rows.iter()
                .find(|row| {
                    row.record.as_ref().is_some_and(|record| {
                        record.policy.policy_key == replacement.policy_key
                            && record.policy.version_key == replacement.version_key
                    })
                })
                .map(|row| resource_id(&snapshot.items[row.item_index].registration))
                .ok_or_else(invalid)
        })
        .transpose()?;
    let provenance = inspection::provenance(snapshot, &row.closure)?;
    fence(control)?;
    inspection::finish(
        json!({"resource_version":inspection::version(snapshot,"/api/v2/lifecycle/records/{record_id}",selected,&query),
        "snapshot_version":snapshot.version,"record_id":selected,"resource_id":selected,
        "policy_key":status.policy_key,"version_key":status.version_key,"as_of":as_of,
        "state":status.state,"derived_status":status.derived_status,"owner_keys":status.owner_keys,
        "next_review_date":status.next_review_date,"blockers":status.blockers,
        "current_fingerprints":status.current_fingerprints,"approved_fingerprints":status.approved_fingerprints,
        "artifact_identity_changes":status.artifact_identity_changes,"replaced_by":status.replaced_by,
        "replacement_record_id":replacement_id,"impact_references":impact_references(&status.impact_finding_ids),
        "provenance":provenance,"trust_boundary":TRUST_BOUNDARY}),
        control,
    )
}

/// Keep declared finding IDs explicitly unresolved rather than inventing a comparison join.
fn impact_references(ids: &[String]) -> Vec<Value> {
    ids.iter().map(|id| json!({"finding_id":id,"binding":"unresolved"})).collect()
}

/// Project recorded event metadata only; rationale and fingerprint/prose payloads stay private.
fn history_item(event: &record::TransitionEvent) -> Value {
    json!({"event_id":event.event_id,"sequence":event.sequence,"timestamp":event.timestamp,
        "previous_state":event.previous_state,"next_state":event.next_state,
        "actor_key":event.actor_key,"declared_role":event.declared_role,
        "assertions":event.assertions,"impact_references":impact_references(&event.impact_finding_ids),
        "replacement":event.replacement})
}

/// Read selected intrinsic recorded history without date, current-status or approval inference.
pub(crate) fn history(
    snapshot: &Snapshot,
    selected: &str,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    fence(control)?;
    let query = Query::new(raw, &["page_size", "cursor"])?;
    inspection::require_index2(snapshot)?;
    let index = selected_index(snapshot, selected)?;
    let _facts = capture_facts(snapshot, control)?;
    let parsed = record::parse(&snapshot.items[index].captured.bytes);
    fence(control)?;
    let record = parsed.map_err(|_| invalid())?;
    let version = inspection::version(
        snapshot,
        "/api/v2/lifecycle/records/{record_id}/history",
        selected,
        &query,
    );
    fence(control)?;
    let mut value = inspection::page(record.history.len(), &version, &query, |index| {
        Ok(history_item(&record.history[index]))
    })?;
    fence(control)?;
    value["snapshot_version"] = json!(snapshot.version);
    value["availability"] = json!("available");
    value["record_id"] = json!(selected);
    value["counts"] = json!({"total_events":record.history.len()});
    value["as_of"] = Value::Null;
    inspection::finish(value, control)
}

/// Reference-index every distinct owner membership with canonical CLI owner/date/policy order.
fn placements(rows: &[RecordRow], control: &mut dyn WorkControl) -> WorkResult<Vec<Placement>> {
    let mut placements = Vec::new();
    let records = rows
        .iter()
        .map(|row| row.record.as_ref().ok_or_else(invalid))
        .collect::<Result<Vec<_>>>()?;
    for (index, row) in rows.iter().enumerate() {
        let record = row.record.as_ref().ok_or_else(invalid)?;
        for owner in 0..record.policy.owner_keys.len() {
            fence(control)?;
            if placements.len() >= MAX_OWNER_PLACEMENTS {
                return Err(Error::new(
                    "payload-too-large",
                    "The complete owner queue exceeds its supported bound.",
                    false,
                )
                .into());
            }
            placements.push(Placement { record: index, owner });
        }
    }
    placements.sort_by(|left, right| {
        let left_record = records[left.record];
        let right_record = records[right.record];
        left_record.policy.owner_keys[left.owner]
            .cmp(&right_record.policy.owner_keys[right.owner])
            .then_with(|| {
                left_record.review.next_review_date.cmp(&right_record.review.next_review_date)
            })
            .then_with(|| left_record.policy.policy_key.cmp(&right_record.policy.policy_key))
            .then_with(|| left_record.policy.version_key.cmp(&right_record.policy.version_key))
    });
    fence(control)?;
    Ok(placements)
}

/// Return exact group identity without treating owner placements as unique policies.
fn placement_group<'a>(
    rows: &'a [RecordRow],
    placement: &Placement,
) -> Result<(&'a str, NaiveDate)> {
    let record = rows[placement.record].record.as_ref().ok_or_else(invalid)?;
    Ok((&record.policy.owner_keys[placement.owner], record.review.next_review_date))
}

/// Emit one bounded owner reference row, not a repeated full lifecycle report.
fn queue_item(snapshot: &Snapshot, rows: &[RecordRow], placement: &Placement) -> Result<Value> {
    let row = &rows[placement.record];
    let record = row.record.as_ref().ok_or_else(invalid)?;
    let status = row.status.as_ref().ok_or_else(invalid)?;
    let id = resource_id(&snapshot.items[row.item_index].registration);
    Ok(
        json!({"record_id":id,"resource_id":id,"owner_key":record.policy.owner_keys[placement.owner],
        "next_review_date":status.next_review_date,"policy_key":status.policy_key,"version_key":status.version_key,
        "state":status.state,"derived_status":status.derived_status,"blockers":status.blockers}),
    )
}

/// Read the complete explicitly dated portfolio queue with distinct membership denominators.
pub(crate) fn queue(
    snapshot: &Snapshot,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    fence(control)?;
    let query = Query::new(raw, &["as_of", "owner", "page_size", "cursor"])?;
    let as_of = query.date(true)?.ok_or_else(Error::invalid)?;
    if !snapshot.index_present || snapshot.index.schema_version != "forge.workspace/2" {
        return setup_page(
            snapshot,
            "/api/v2/lifecycle/queue",
            &query,
            Some(as_of),
            json!({"distinct_records":0,"total_owner_placements":0,"matching_owner_placements":0,"total_groups":0,"matching_groups":0}),
            control,
        );
    }
    let mut rows = inventory(snapshot, control)?;
    compute(&mut rows, as_of, control)?;
    let placements = placements(&rows, control)?;
    let mut all_groups = BTreeSet::new();
    let mut matching_groups = BTreeSet::new();
    let mut matching = Vec::new();
    for (index, placement) in placements.iter().enumerate() {
        fence(control)?;
        let group = placement_group(&rows, placement)?;
        all_groups.insert(group);
        if query.optional("owner").is_none_or(|owner| group.0 == owner) {
            matching.push(index);
            matching_groups.insert(group);
        }
    }
    let version = inspection::version(snapshot, "/api/v2/lifecycle/queue", "", &query);
    fence(control)?;
    let mut value = inspection::page(matching.len(), &version, &query, |index| {
        queue_item(snapshot, &rows, &placements[matching[index]])
    })?;
    fence(control)?;
    value["snapshot_version"] = json!(snapshot.version);
    value["availability"] = json!(availability(snapshot, &rows));
    value["counts"] = json!({"distinct_records":rows.len(),"total_owner_placements":placements.len(),
        "matching_owner_placements":matching.len(),"total_groups":all_groups.len(),"matching_groups":matching_groups.len()});
    value["as_of"] = json!(as_of);
    inspection::finish(value, control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::record::{
        ActorAssertion, ArtifactFingerprint, DeclaredRole, LifecycleState, PolicyReference,
        TransitionEvent,
    };
    use crate::workspace::preparation::{Interruption, NoopControl, test_support::Recorder};
    use crate::workspace::root::Root;

    /// Fixed private source bytes whose content must never appear in inspection DTOs.
    const SOURCE: &[u8] = b"PRIVATE source body: no response content";

    /// Build a validated synthetic declared record with exact source and generated byte pins.
    fn draft(
        policy: &str,
        owners: &[&str],
        artifacts: Vec<ArtifactFingerprint>,
    ) -> LifecycleRecord {
        let artifacts = serde_json::to_value(artifacts).expect("typed artifact fingerprints");
        let parties = owners
            .iter()
            .map(|owner| json!({"key":owner,"roles":["owner","reviewer","approver","author"]}))
            .collect::<Vec<_>>();
        let record: LifecycleRecord = serde_json::from_value(json!({
            "schema_version":"forge.policy-lifecycle/2",
            "policy":{"policy_key":policy,"version_key":"v1","title":"PRIVATE lifecycle title",
                "owner_keys":owners,"source":{"path":"source.bin","sha256":crate::hashing::sha256_hex(SOURCE)},
                "generated_artifacts":artifacts},
            "parties":parties,
            "approval_policy":{"schema_version":"forge.approval-policy/1",
                "required_roles":[{"role":"reviewer","count":1},{"role":"approver","count":1}],"separation":{}},
            "review":{"cadence_days":30,"next_review_date":"2026-10-12","due_soon_days":7,"timezone_policy":"date-only"},
            "state":"draft","history":[]
        })).expect("typed synthetic lifecycle");
        record::validate(&record).expect("intrinsic synthetic lifecycle");
        record
    }

    /// Produce an identifiable but deliberately schema-incomplete native root and its real raw pin.
    fn artifact(
        root: &str,
        path: &str,
        role: Role,
    ) -> (ArtifactFingerprint, (Role, String, Vec<u8>)) {
        let uuid = "00000000-0000-4000-8000-000000000001";
        let mut value = serde_json::Map::new();
        value.insert(root.to_owned(), json!({"uuid":uuid}));
        let bytes = serde_json::to_vec(&Value::Object(value)).expect("synthetic native root");
        let expected = ArtifactFingerprint {
            path: path.to_owned(),
            sha256: crate::hashing::sha256_hex(&bytes),
            oscal_type: Some(root.to_owned()),
            root_uuid: Some(uuid.to_owned()),
        };
        (expected, (role, path.to_owned(), bytes))
    }

    /// Capture only explicit owned temporary registrations using the actual existing capture seam.
    fn fixture(
        records: &[LifecycleRecord],
        extras: &[(Role, String, Vec<u8>)],
    ) -> (tempfile::TempDir, Snapshot) {
        let temp = tempfile::tempdir().expect("owned synthetic root");
        let mut registrations = Vec::new();
        for (index, record) in records.iter().enumerate() {
            let path = format!("record-{index}.json");
            std::fs::write(
                temp.path().join(&path),
                serde_json::to_vec(record).expect("record bytes"),
            )
            .expect("owned record");
            registrations.push(
                json!({"key":format!("record-{index}"),"role":"lifecycle-record","path":path}),
            );
        }
        std::fs::write(temp.path().join("source.bin"), SOURCE).expect("owned source");
        registrations.push(json!({"key":"source","role":"lifecycle-source","path":"source.bin"}));
        for (index, (role, path, bytes)) in extras.iter().enumerate() {
            std::fs::write(temp.path().join(path), bytes).expect("owned generated artifact");
            registrations.push(json!({"key":format!("artifact-{index}"),"role":role,"path":path}));
        }
        std::fs::write(temp.path().join("forge.workspace.json"),serde_json::to_vec(&json!({
            "schema_version":"forge.workspace/2","label":"Synthetic lifecycle","resources":registrations
        })).expect("closed index")).expect("owned index");
        let root = Root::open(temp.path()).expect("owned confined root");
        let snapshot = Snapshot::capture(&root).expect("actual complete registered capture");
        (temp, snapshot)
    }

    /// Select the real emitted registered record ID rather than a fabricated wire label.
    fn id(snapshot: &Snapshot, index: usize) -> String {
        resource_id(&snapshot.items[index].registration)
    }

    /// Supply one explicit date without any implicit time source.
    fn dated(date: &str) -> Vec<(String, String)> {
        vec![("as_of".to_owned(), date.to_owned())]
    }

    /// Append real deterministic declared events, using actual intrinsic record validation.
    fn transition(
        record: &mut LifecycleRecord,
        next: LifecycleState,
        role: DeclaredRole,
        time: &str,
        replacement: Option<PolicyReference>,
    ) {
        let mut generated = record
            .policy
            .generated_artifacts
            .iter()
            .map(|artifact| NamedHash {
                path: artifact.path.clone(),
                sha256: artifact.sha256.clone(),
            })
            .collect::<Vec<_>>();
        generated.sort();
        let mut event = TransitionEvent {
            sequence: u32::try_from(record.history.len() + 1).expect("small fixture"),
            event_id: String::new(),
            legacy_event_id: None,
            previous_state: record.state,
            next_state: next,
            actor_key: record.policy.owner_keys[0].clone(),
            declared_role: role,
            timestamp: time.to_owned(),
            rationale: "PRIVATE event rationale".to_owned(),
            fingerprints: FingerprintSet {
                source_sha256: record.policy.source.sha256.clone(),
                generated_artifacts: generated,
            },
            assertions: vec![],
            impact_finding_ids: if next == LifecycleState::InReview {
                vec!["finding-local".to_owned()]
            } else {
                vec![]
            },
            replacement: replacement.clone(),
        };
        event.event_id = record::event_id(record, &event).expect("real event ID");
        record.state = next;
        record.history.push(event);
        if replacement.is_some() {
            record.replaced_by = replacement;
        }
        record::validate(record).expect("intrinsic declared event");
    }

    /// Establish declared review/approval; these synthetic actors are not authenticated.
    fn approve(record: &mut LifecycleRecord, time: &str) {
        transition(
            record,
            LifecycleState::InReview,
            DeclaredRole::Reviewer,
            "2026-10-01T00:00:00Z",
            None,
        );
        transition(record, LifecycleState::Approved, DeclaredRole::Approver, time, None);
    }

    /// Extract only the safe existing workspace error code for ordinary failure assertions.
    fn error_code(result: WorkResult<Value>) -> &'static str {
        match result {
            Err(WorkError::Failed(error)) => error.code,
            other => panic!("expected fixed failure: {other:?}"),
        }
    }

    /// Exercise all five generated families while keeping schema validity separate from captured facts.
    #[test]
    fn all_five_native_roots_keep_hash_identity_and_invalid_schema_dimension() {
        let definitions = [
            ("catalog", "catalog.json", Role::OscalCatalogArtifact),
            ("component-definition", "component.json", Role::OscalComponentArtifact),
            ("profile", "profile.json", Role::OscalProfileArtifact),
            ("system-security-plan", "ssp.json", Role::OscalSspArtifact),
            ("mapping-collection", "mapping.json", Role::MappingCollection),
        ];
        let pairs = definitions
            .into_iter()
            .map(|(root, path, role)| artifact(root, path, role))
            .collect::<Vec<_>>();
        let record = draft(
            "all-roots",
            &["owner"],
            pairs.iter().map(|(fingerprint, _)| fingerprint.clone()).collect(),
        );
        let extras = pairs.into_iter().map(|(_, extra)| extra).collect::<Vec<_>>();
        let (_temp, snapshot) = fixture(&[record], &extras);
        let result = detail(&snapshot, &id(&snapshot, 0), &dated("2026-10-01"), &mut NoopControl)
            .expect("identifiable facts");
        assert_eq!(
            result["current_fingerprints"]["generated_artifacts"].as_array().expect("array").len(),
            5
        );
        assert_eq!(result["artifact_identity_changes"], json!([]));
        assert_eq!(result["provenance"].as_array().expect("closure").len(), 7);
        assert_eq!(
            result["provenance"]
                .as_array()
                .expect("closure")
                .iter()
                .filter(|pin| pin["validation_state"] == "invalid")
                .count(),
            5
        );
        assert_eq!(result["snapshot_version"], snapshot.version);
        let page = records(&snapshot, &[], &mut NoopControl).expect("recorded inventory");
        assert_eq!(page["availability"], "needs-attention");
        assert!(page["page"]["items"][0]["derived_status"].is_null());
        assert_eq!(page["page"]["items"][0]["record_version"], snapshot.items[0].captured.sha256);
    }

    /// Use changed captured native model and UUID to preserve approved-drifted precedence.
    #[test]
    fn captured_model_uuid_drift_is_not_schema_validity_or_clean_approval() {
        let (fingerprint, mut extra) =
            artifact("catalog", "catalog.json", Role::OscalCatalogArtifact);
        let mut record = draft("drift", &["owner"], vec![fingerprint]);
        approve(&mut record, "2026-10-02T00:00:00Z");
        extra.2 = serde_json::to_vec(
            &json!({"component-definition":{"uuid":"00000000-0000-4000-8000-000000000002"}}),
        )
        .expect("changed root");
        let (_temp, snapshot) = fixture(&[record], &[extra]);
        let result = detail(&snapshot, &id(&snapshot, 0), &dated("2026-10-20"), &mut NoopControl)
            .expect("observed drift");
        assert_eq!(result["state"], "approved");
        assert_eq!(result["derived_status"], "approved-drifted");
        assert_eq!(result["blockers"], json!(["approved-drifted", "artifact-identity-changed"]));
        assert_eq!(result["artifact_identity_changes"], json!(["catalog.json"]));
        assert_ne!(result["current_fingerprints"], result["approved_fingerprints"]);
        assert!(
            result["provenance"]
                .as_array()
                .expect("pins")
                .iter()
                .any(|pin| pin["validation_state"] == "invalid")
        );
    }

    /// Preserve explicit inclusive due-soon and strictly later overdue boundaries.
    #[test]
    fn explicit_as_of_boundaries_and_recorded_expanded_year_are_preserved() {
        let record = draft("date", &["owner"], vec![]);
        let (_temp, snapshot) = fixture(std::slice::from_ref(&record), &[]);
        for (date, expected) in [
            ("2026-10-04", "draft"),
            ("2026-10-05", "due-soon"),
            ("2026-10-12", "due-soon"),
            ("2026-10-13", "overdue"),
        ] {
            let result = detail(&snapshot, &id(&snapshot, 0), &dated(date), &mut NoopControl)
                .expect("explicit date");
            assert_eq!(result["derived_status"], expected);
            assert_eq!(result["as_of"], date);
        }
        let mut expanded = record;
        expanded.review.next_review_date =
            NaiveDate::from_ymd_opt(10000, 1, 1).expect("chrono expanded year");
        record::validate(&expanded).expect("existing intrinsic date contract");
        let (_temp, snapshot) = fixture(&[expanded], &[]);
        let result = detail(&snapshot, &id(&snapshot, 0), &dated("2026-10-01"), &mut NoopControl)
            .expect("recorded date fidelity");
        assert_eq!(result["next_review_date"], "+10000-01-01");
    }

    /// Reject missing closure on every computed view while recorded inventory stays unavailable.
    #[test]
    fn missing_registered_closure_cannot_be_hidden_by_filters_or_selected_record() {
        let (fingerprint, _) = artifact("catalog", "missing.json", Role::OscalCatalogArtifact);
        let valid = draft("valid", &["one"], vec![]);
        let missing = draft("missing", &["two"], vec![fingerprint]);
        let (_temp, snapshot) = fixture(&[valid, missing], &[]);
        let recorded = records(&snapshot, &[], &mut NoopControl).expect("recorded unavailable row");
        assert_eq!(recorded["counts"]["registered_records"], 2);
        assert_eq!(recorded["counts"]["unavailable_records"], 1);
        assert_eq!(recorded["page"]["items"][1]["diagnostic_code"], "lifecycle-input-unavailable");
        let mut filtered = dated("2026-10-01");
        filtered.push(("owner".to_owned(), "one".to_owned()));
        assert_eq!(
            error_code(records(&snapshot, &filtered, &mut NoopControl)),
            "validation-failed"
        );
        assert_eq!(
            error_code(detail(
                &snapshot,
                &id(&snapshot, 0),
                &dated("2026-10-01"),
                &mut NoopControl
            )),
            "validation-failed"
        );
        assert_eq!(error_code(queue(&snapshot, &filtered, &mut NoopControl)), "validation-failed");
    }

    /// Prove strong grammar rejects external spellings before any capture-reference lookup.
    #[test]
    fn strong_paths_and_unregistered_external_sentinels_never_gain_read_authority() {
        for raw in ["../sentinel", "./source.bin", "/absolute", "a\\b", "C:/outside", "a//b"] {
            assert_eq!(
                strong_reference(raw).expect_err("unsafe spelling").code,
                "resource-containment"
            );
        }
        let outside = tempfile::tempdir().expect("owned external sentinel root");
        let sentinel = outside.path().join("sentinel");
        std::fs::write(&sentinel, b"PRIVATE external sentinel").expect("owned off-root sentinel");
        let mut record = draft("outside", &["owner"], vec![]);
        record.policy.source.path = sentinel.to_str().expect("synthetic absolute path").to_owned();
        let (_temp, snapshot) = fixture(&[record], &[]);
        let response = records(&snapshot, &[], &mut NoopControl)
            .expect("invalid row without opening a sentinel");
        assert_eq!(response["page"]["items"][0]["diagnostic_code"], "invalid-lifecycle-record");
        assert_eq!(
            error_code(detail(
                &snapshot,
                &id(&snapshot, 0),
                &dated("2026-10-01"),
                &mut NoopControl
            )),
            "validation-failed"
        );
        assert_eq!(snapshot.items.len(), 2); // Only the record and explicitly registered source were captured.
        assert_eq!(
            std::fs::read(&sentinel).expect("owned sentinel remains"),
            b"PRIVATE external sentinel"
        );
    }

    /// Guard direct-call raw hash and file-instance bijections rather than trusting forged metadata.
    #[test]
    fn captured_alias_and_hash_mismatch_are_whole_failures() {
        let record = draft("integrity", &["owner"], vec![]);
        let (_temp, mut snapshot) = fixture(&[record], &[]);
        let original = snapshot.items[1].captured.identity;
        snapshot.items[1].captured.identity = snapshot.items[0].captured.identity;
        assert_eq!(error_code(records(&snapshot, &[], &mut NoopControl)), "resource-containment");
        snapshot.items[1].captured.identity = original;
        snapshot.items[1].captured.sha256 = "0".repeat(64);
        assert_eq!(error_code(records(&snapshot, &[], &mut NoopControl)), "validation-failed");
    }

    /// Check duplicate identities, declared replacement binding and complete-portfolio cycles.
    #[test]
    fn duplicate_and_replacement_cycles_cannot_produce_partial_ready_views() {
        let record = draft("duplicate", &["owner"], vec![]);
        let (_temp, snapshot) = fixture(&[record.clone(), record], &[]);
        let recorded =
            records(&snapshot, &[], &mut NoopControl).expect("recorded invalid portfolio");
        assert_eq!(recorded["counts"]["unavailable_records"], 2);
        assert_eq!(recorded["page"]["items"][0]["diagnostic_code"], "invalid-lifecycle-portfolio");
        assert_eq!(
            error_code(queue(&snapshot, &dated("2026-10-01"), &mut NoopControl)),
            "validation-failed"
        );
        let mut one = draft("one", &["owner"], vec![]);
        let mut two = draft("two", &["owner"], vec![]);
        approve(&mut one, "2026-10-02T00:00:00Z");
        approve(&mut two, "2026-10-02T00:00:00Z");
        transition(
            &mut one,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "two".to_owned(), version_key: "v1".to_owned() }),
        );
        transition(
            &mut two,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "one".to_owned(), version_key: "v1".to_owned() }),
        );
        let (_temp, snapshot) = fixture(&[one, two], &[]);
        assert_eq!(
            error_code(detail(
                &snapshot,
                &id(&snapshot, 0),
                &dated("2026-10-04"),
                &mut NoopControl
            )),
            "validation-failed"
        );
    }

    /// Resolve a supplied approved successor to its actual registered `res_ID` without inferred joins.
    #[test]
    fn replacement_id_is_bound_to_the_validated_supplied_policy_version() {
        let mut old = draft("old", &["owner"], vec![]);
        let mut new = draft("new", &["owner"], vec![]);
        approve(&mut old, "2026-10-02T00:00:00Z");
        approve(&mut new, "2026-10-02T00:00:00Z");
        transition(
            &mut old,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "new".to_owned(), version_key: "v1".to_owned() }),
        );
        let (_temp, snapshot) = fixture(&[old, new], &[]);
        let result = detail(&snapshot, &id(&snapshot, 0), &dated("2026-10-04"), &mut NoopControl)
            .expect("complete supplied successor");
        assert_eq!(result["replacement_record_id"], id(&snapshot, 1));
        assert_eq!(result["replaced_by"], json!({"policy_key":"new","version_key":"v1"}));
        assert_eq!(
            result["impact_references"],
            json!([{"finding_id":"finding-local","binding":"unresolved"}])
        );
    }

    /// Keep canonical owner/date/policy order and unique records, placements and groups separate.
    #[test]
    fn queue_order_counts_and_owner_filter_use_exact_full_memberships() {
        let z = draft("z-policy", &["z-owner", "a-owner"], vec![]);
        let a = draft("a-policy", &["a-owner"], vec![]);
        let (_temp, snapshot) = fixture(&[z, a], &[]);
        let full = queue(&snapshot, &dated("2026-10-01"), &mut NoopControl)
            .expect("complete membership queue");
        assert_eq!(
            full["counts"],
            json!({"distinct_records":2,"total_owner_placements":3,"matching_owner_placements":3,"total_groups":2,"matching_groups":2})
        );
        assert_eq!(full["page"]["items"][0]["policy_key"], "a-policy");
        assert_eq!(full["page"]["items"][1]["policy_key"], "z-policy");
        assert_eq!(full["page"]["items"][2]["owner_key"], "z-owner");
        let mut filtered = dated("2026-10-01");
        filtered.push(("owner".to_owned(), "a-owner".to_owned()));
        let result = queue(&snapshot, &filtered, &mut NoopControl).expect("filtered presentation");
        assert_eq!(
            result["counts"],
            json!({"distinct_records":2,"total_owner_placements":3,"matching_owner_placements":2,"total_groups":2,"matching_groups":1})
        );
        assert_eq!(result["page"]["total_matching"], 2);
    }

    /// Bind paging to exact date/filter/snapshot context while preserving whole counts.
    #[test]
    fn cursor_versions_reject_changed_dates_filters_and_capture() {
        let (_temp, mut snapshot) =
            fixture(&[draft("one", &["owner"], vec![]), draft("two", &["owner"], vec![])], &[]);
        let mut first_query = dated("2026-10-01");
        first_query.push(("page_size".to_owned(), "1".to_owned()));
        let first = records(&snapshot, &first_query, &mut NoopControl).expect("first bounded page");
        assert_eq!(first["counts"]["registered_records"], 2);
        assert_eq!(first["page"]["total_matching"], 2);
        let cursor = first["page"]["next_cursor"].as_str().expect("second page").to_owned();
        let mut second_query = first_query.clone();
        second_query.push(("cursor".to_owned(), cursor));
        let second =
            records(&snapshot, &second_query, &mut NoopControl).expect("same context second page");
        assert_eq!(first["resource_version"], second["resource_version"]);
        assert_ne!(first["page"]["items"][0]["record_id"], second["page"]["items"][0]["record_id"]);
        let mut changed = second_query.clone();
        changed[0].1 = "2026-10-02".to_owned();
        assert_eq!(error_code(records(&snapshot, &changed, &mut NoopControl)), "version-conflict");
        let mut changed = second_query.clone();
        changed.push(("owner".to_owned(), "missing".to_owned()));
        assert_eq!(error_code(records(&snapshot, &changed, &mut NoopControl)), "version-conflict");
        snapshot.version = "0".repeat(64);
        assert_eq!(
            error_code(records(&snapshot, &second_query, &mut NoopControl)),
            "version-conflict"
        );
    }

    /// Preserve declared event IDs/assertions but omit all private title, rationale and source content.
    #[test]
    fn history_projects_real_intrinsic_events_with_unresolved_impact_and_no_prose() {
        let mut record = draft("history", &["owner"], vec![]);
        approve(&mut record, "2026-10-02T00:00:00Z");
        record.history[1].assertions.push(ActorAssertion {
            actor_key: "owner".to_owned(),
            declared_role: DeclaredRole::Reviewer,
        });
        let updated_id =
            record::event_id(&record, &record.history[1]).expect("updated actual event evidence");
        record.history[1].event_id = updated_id;
        record::validate(&record).expect("actual assertion admission");
        let (_temp, snapshot) = fixture(std::slice::from_ref(&record), &[]);
        let result =
            history(&snapshot, &id(&snapshot, 0), &[], &mut NoopControl).expect("recorded events");
        assert_eq!(result["record_id"], id(&snapshot, 0));
        assert!(result["as_of"].is_null());
        assert_eq!(result["counts"], json!({"total_events":2}));
        assert_eq!(result["page"]["items"][0]["event_id"], record.history[0].event_id);
        assert_eq!(
            result["page"]["items"][1]["assertions"],
            json!([{"actor_key":"owner","declared_role":"reviewer"}])
        );
        assert_eq!(
            result["page"]["items"][0]["impact_references"],
            json!([{"finding_id":"finding-local","binding":"unresolved"}])
        );
        let text = serde_json::to_string(&result).expect("redacted DTO");
        for secret in [
            "PRIVATE lifecycle title",
            "PRIVATE event rationale",
            "PRIVATE source body",
            "source.bin",
        ] {
            assert!(!text.contains(secret));
        }
    }

    /// Reject unknown/duplicate/empty/state/date fields and missing required dates without defaults.
    #[test]
    fn closed_queries_and_selected_roles_never_gain_fallback_authority() {
        let (_temp, snapshot) = fixture(&[draft("query", &["owner"], vec![])], &[]);
        for raw in [
            vec![("unknown".to_owned(), "x".to_owned())],
            vec![("owner".to_owned(), String::new())],
            vec![("state".to_owned(), "ready".to_owned())],
            vec![("as_of".to_owned(), "2026-1-01".to_owned())],
            vec![
                ("as_of".to_owned(), "2026-10-01".to_owned()),
                ("as_of".to_owned(), "2026-10-01".to_owned()),
            ],
        ] {
            assert_eq!(error_code(records(&snapshot, &raw, &mut NoopControl)), "invalid-request");
        }
        assert_eq!(
            error_code(detail(&snapshot, &id(&snapshot, 0), &[], &mut NoopControl)),
            "invalid-request"
        );
        assert_eq!(error_code(queue(&snapshot, &[], &mut NoopControl)), "invalid-request");
        assert_eq!(
            error_code(history(
                &snapshot,
                &id(&snapshot, 0),
                &dated("2026-10-01"),
                &mut NoopControl
            )),
            "invalid-request"
        );
        assert_eq!(
            error_code(detail(
                &snapshot,
                &id(&snapshot, 1),
                &dated("2026-10-01"),
                &mut NoopControl
            )),
            "not-found"
        );
    }

    /// Carry typed sticky interruption before hash, after parse and after selected projection.
    #[test]
    fn cooperative_stops_are_not_invalid_rows_or_partial_results() {
        let (_temp, snapshot) = fixture(&[draft("stop", &["owner"], vec![])], &[]);
        for visit in [1, 4, 8, 16] {
            let mut control = Recorder::at(Stage::PrepareDomain, visit);
            assert!(matches!(
                records(&snapshot, &dated("2026-10-01"), &mut control),
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            ));
            assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
        }
    }

    /// Refuse unidentifiable native JSON/UUID while preserving fixed no-date unavailability.
    #[test]
    fn unidentifiable_generated_roots_cannot_be_clean_computed_status() {
        for bytes in [
            b"not JSON".to_vec(),
            serde_json::to_vec(&json!({"catalog":{"uuid":"not-a-uuid"}}))
                .expect("invalid UUID fixture"),
        ] {
            let (fingerprint, mut extra) =
                artifact("catalog", "catalog.json", Role::OscalCatalogArtifact);
            let record = draft("identity", &["owner"], vec![fingerprint]);
            extra.2 = bytes;
            let (_temp, snapshot) = fixture(&[record], &[extra]);
            let page = records(&snapshot, &[], &mut NoopControl).expect("recorded unavailable row");
            assert_eq!(page["page"]["items"][0]["availability"], "unavailable");
            assert_eq!(page["page"]["items"][0]["diagnostic_code"], "lifecycle-input-unavailable");
            assert!(page["page"]["items"][0]["derived_status"].is_null());
            assert_eq!(
                error_code(detail(
                    &snapshot,
                    &id(&snapshot, 0),
                    &dated("2026-10-01"),
                    &mut NoopControl
                )),
                "validation-failed"
            );
        }
    }

    /// Bound compact memberships inclusively before retaining an excessive placement.
    #[test]
    fn compact_owner_memberships_keep_inclusive_64000_ceiling() {
        let names = (0..64).map(|index| format!("owner-{index:02}")).collect::<Vec<_>>();
        let owners = names.iter().map(String::as_str).collect::<Vec<_>>();
        let record = draft("membership", &owners, vec![]);
        let make_row = |index| RecordRow {
            item_index: index,
            record: Some(record.clone()),
            current: None,
            closure: vec![],
            availability: "valid",
            diagnostic: None,
            status: None,
        };
        let mut rows = (0..1000).map(make_row).collect::<Vec<_>>();
        let admitted = placements(&rows, &mut NoopControl).expect("inclusive compact ceiling");
        assert_eq!(admitted.len(), 64000);
        assert_eq!(placement_group(&rows, &admitted[0]).expect("first group").0, "owner-00");
        rows.push(make_row(1000));
        assert!(
            matches!(placements(&rows,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="payload-too-large")
        );
        // This lower-level membership control does not fabricate a 1001-resource service capture.
    }

    /// Count every one of 1000 invalid registrations before projecting only 200 inventory rows.
    #[test]
    fn thousand_record_inventory_preserves_full_invalid_count_and_page_bound() {
        let temp = tempfile::tempdir().expect("owned large inventory");
        let mut resources = Vec::new();
        for index in 0..1000 {
            let path = format!("record-{index}.json");
            std::fs::write(temp.path().join(&path), b"{}").expect("owned invalid record");
            resources.push(
                json!({"key":format!("record-{index}"),"role":"lifecycle-record","path":path}),
            );
        }
        std::fs::write(temp.path().join("forge.workspace.json"),serde_json::to_vec(&json!({"schema_version":"forge.workspace/2","label":"Bounded invalid rows","resources":resources})).expect("bounded index")).expect("owned index");
        let root = Root::open(temp.path()).expect("owned root");
        let snapshot = Snapshot::capture(&root).expect("all registrations captured");
        let result =
            records(&snapshot, &[("page_size".to_owned(), "200".to_owned())], &mut NoopControl)
                .expect("metadata inventory");
        assert_eq!(
            result["counts"],
            json!({"registered_records":1000,"matching_records":1000,"unavailable_records":1000})
        );
        assert_eq!(result["page"]["total_matching"], 1000);
        assert_eq!(result["page"]["items"].as_array().expect("selected rows").len(), 200);
        assert_eq!(result["availability"], "needs-attention");
        assert!(
            result["page"]["items"]
                .as_array()
                .expect("rows")
                .iter()
                .all(|row| row["derived_status"].is_null())
        );
    }

    /// Distinguish real absent, legacy and empty captures without creating an index or status row.
    #[test]
    fn startup_pages_preserve_absent_legacy_and_empty_index_states_without_writes() {
        let mut snapshot_versions = BTreeSet::new();
        for (schema, availability) in [
            (None, "absent-index"),
            (Some("forge.workspace/1"), "index-upgrade-required"),
            (Some("forge.workspace/2"), "empty"),
        ] {
            let temp = tempfile::tempdir().expect("owned startup root");
            let sentinel = temp.path().join("unregistered-private.bin");
            std::fs::write(&sentinel, b"PRIVATE unregistered startup sentinel")
                .expect("owned unregistered sentinel");
            let index_path = temp.path().join("forge.workspace.json");
            let index_bytes = schema.map(|version| {
                serde_json::to_vec(&json!({
                    "schema_version":version,"label":"Startup-only fixture","resources":[]
                }))
                .expect("closed empty index bytes")
            });
            if let Some(bytes) = &index_bytes {
                std::fs::write(&index_path, bytes).expect("owned empty index");
            }
            let names_before = std::fs::read_dir(temp.path())
                .expect("owned flat root before queries")
                .map(|entry| entry.expect("owned entry").file_name())
                .collect::<BTreeSet<_>>();
            let root = Root::open(temp.path()).expect("actual confined startup root");
            let snapshot = Snapshot::capture(&root).expect("actual complete startup capture");
            assert_eq!(snapshot.index_present, schema.is_some());
            assert_eq!(snapshot.index.schema_version, schema.unwrap_or("forge.workspace/1"));
            assert!(snapshot.index.resources.is_empty());
            assert!(snapshot.items.is_empty());
            assert!(snapshot_versions.insert(snapshot.version.clone()));

            let inventory_query = vec![("page_size".to_owned(), "200".to_owned())];
            let inventory = records(&snapshot, &inventory_query, &mut NoopControl)
                .expect("whole undated startup inventory");
            let mut queue_query = dated("2026-10-12");
            queue_query.push(("page_size".to_owned(), "200".to_owned()));
            let queued = queue(&snapshot, &queue_query, &mut NoopControl)
                .expect("whole explicitly dated startup queue");
            for (name, value) in
                [("LifecycleRecordPage", &inventory), ("LifecycleQueuePage", &queued)]
            {
                crate::workspace::contract::validate_for(
                    crate::workspace::contract::ApiMajor::V2,
                    name,
                    value,
                )
                .expect("actual selected closed startup DTO schema");
                assert_eq!(value["availability"], availability);
                assert_eq!(value["snapshot_version"], snapshot.version);
                assert_eq!(
                    value["page"],
                    json!({"items":[],"next_cursor":null,"total_matching":0})
                );
                let mut invented_status = value.clone();
                invented_status["derived_status"] = json!("ready");
                assert!(
                    crate::workspace::contract::validate_for(
                        crate::workspace::contract::ApiMajor::V2,
                        name,
                        &invented_status,
                    )
                    .is_err()
                );
            }
            assert_eq!(
                inventory["counts"],
                json!({"registered_records":0,"matching_records":0,"unavailable_records":0})
            );
            assert!(inventory["as_of"].is_null());
            assert_eq!(
                queued["counts"],
                json!({"distinct_records":0,"total_owner_placements":0,
                    "matching_owner_placements":0,"total_groups":0,"matching_groups":0})
            );
            assert_eq!(queued["as_of"], "2026-10-12");
            assert_ne!(inventory["resource_version"], queued["resource_version"]);

            let names_after = std::fs::read_dir(temp.path())
                .expect("owned flat root after queries")
                .map(|entry| entry.expect("owned entry").file_name())
                .collect::<BTreeSet<_>>();
            assert_eq!(names_after, names_before);
            assert_eq!(
                std::fs::read(&sentinel).expect("owned sentinel after queries"),
                b"PRIVATE unregistered startup sentinel"
            );
            match &index_bytes {
                Some(bytes) => assert_eq!(
                    std::fs::read(&index_path).expect("original empty index remains"),
                    *bytes
                ),
                None => assert!(!index_path.exists()),
            }
            let recaptured = Snapshot::capture(&root).expect("actual unchanged recapture");
            assert_eq!(recaptured.version, snapshot.version);
            assert_eq!(recaptured.index_present, schema.is_some());
            assert_eq!(recaptured.index.schema_version, schema.unwrap_or("forge.workspace/1"));
            assert!(recaptured.items.is_empty());
        }
        assert_eq!(snapshot_versions.len(), 3);
    }
}

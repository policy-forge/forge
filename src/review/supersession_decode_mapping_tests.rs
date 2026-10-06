//! Genuine file-produced queue/report data; no captured native owner is issued here.

use super::*;
use crate::framework::model::ImpactReport;
use crate::mapping::inventory::ResourceEvidence;
use crate::mapping::manifest::ResourceType;
use crate::review::{decode as queue_decode, links as graph, mapping_capture};
use std::path::Path;

/// Own actual generated files and their complete original queue allocations.
struct Case {
    /// Keep every actual fixture file alive for the data controls.
    directory: tempfile::TempDir,
    /// Entire actual old queue original.
    old: Vec<u8>,
    /// Entire actual new queue original.
    new: Vec<u8>,
    /// Inert recorded projection assembled only from actual files/native report data.
    value: Value,
}

/// Project only the exact eleven endpoint fields from a complete real queue item.
fn actual_endpoint(item: &ReviewItem) -> Value {
    json!({"key":item.key,"item_id":item.item_id,"domain":item.domain,"adapter_version":item.adapter_version,"subject_id":item.subject_id,"subject_sha256":item.subject_sha256,"context_sha256":item.context_sha256,"policy_key":item.policy_key,"policy_sha256":item.policy_sha256,"requested_action":item.requested_action,"source_keys":item.source_keys})
}
/// Retain exact whole queue original metadata and the entire native-producer pin roster.
fn actual_queue(raw: &[u8], queue: &QueueDocument) -> Value {
    json!({"schema_version":queue.schema_version,"queue_id":queue.queue_id,"raw_sha256":crate::hashing::sha256_hex(raw),"byte_length":raw.len(),"source_pins":queue.source_pins,"item_count":queue.items.len()})
}
/// Copy only the five public fields of genuine maintained `ResourceEvidence`.
fn actual_resource(resource: &ResourceEvidence) -> Value {
    json!({"resource_type":resource.resource_type,"raw_sha256":resource.raw_sha256,"root_uuid":resource.root_uuid,"oscal_version":resource.oscal_version,"resolved_catalog_sha256":resource.resolved_catalog_sha256})
}
/// Record whole actual file bytes and their explicit native family root spelling.
fn actual_pin(
    root: &Path,
    name: &str,
    role: &str,
    ordinal: usize,
    kind: &str,
    schema: &str,
    native: Option<&str>,
) -> Value {
    let raw = std::fs::read(root.join(name)).unwrap();
    let native_root = native.map(|family| {
        let value: Value = serde_json::from_slice(&raw).unwrap();
        value[family]["uuid"].as_str().unwrap().to_owned()
    });
    json!({"key":format!("impact:{role}:{ordinal}"),"kind":kind,"native_root_uuid":native_root,"raw_sha256":crate::hashing::sha256_hex(&raw),"byte_length":raw.len(),"schema_identity":schema})
}
/// Produce an exact original pin from actual file bytes, with no capture lease.
fn actual_original(root: &Path, name: &str) -> Value {
    let raw = std::fs::read(root.join(name)).unwrap();
    json!({"raw_sha256":crate::hashing::sha256_hex(&raw),"byte_length":raw.len()})
}
/// Read the complete actual ordered Impact pin roster from genuine fixture files only.
fn actual_pins(root: &Path, profile: bool) -> Vec<Value> {
    let mut pins = vec![
        actual_pin(
            root,
            "impact-manifest.json",
            "manifest",
            0,
            "framework-impact-manifest",
            "forge.framework-impact/1",
            None,
        ),
        actual_pin(
            root,
            "impact-report.json",
            "current-report",
            1,
            "framework-impact-report",
            "forge.framework-impact-report/1",
            None,
        ),
    ];
    let family = if profile { "profile" } else { "catalog" };
    let schema = if profile { "oscal:1.2.3:profile" } else { "oscal:1.2.3:catalog" };
    for (prefix, role, resolved_role) in [
        ("old/", if profile { "old-profile" } else { "old-catalog" }, "old-resolved"),
        ("", if profile { "new-profile" } else { "new-catalog" }, "new-resolved"),
    ] {
        pins.push(actual_pin(
            root,
            &format!("{prefix}source.json"),
            role,
            pins.len(),
            family,
            schema,
            Some(family),
        ));
        if profile {
            pins.push(actual_pin(
                root,
                &format!("{prefix}source-resolved.json"),
                resolved_role,
                pins.len(),
                "resolved-catalog",
                "oscal:1.2.3:catalog",
                Some("catalog"),
            ));
        }
    }
    pins.push(actual_pin(
        root,
        "old/mapping.json",
        "mapping",
        pins.len(),
        "mapping",
        "oscal:1.2.3:mapping-collection",
        Some("mapping-collection"),
    ));
    pins
}

/// Assemble inert data from real init queues and the complete maintained file analysis result.
fn case(profile: bool, changed: bool) -> Case {
    let (directory, root) =
        mapping_capture::tests::supersession_mapping_fixture(profile, changed, false);
    let old = std::fs::read(root.join("old-queue.json")).unwrap();
    let new = std::fs::read(root.join("new-queue.json")).unwrap();
    let old_queue: QueueDocument = serde_json::from_slice(&old).unwrap();
    let new_queue: QueueDocument = serde_json::from_slice(&new).unwrap();
    let report: ImpactReport =
        serde_json::from_slice(&std::fs::read(root.join("impact-report.json")).unwrap()).unwrap();
    assert!(report.filters.is_empty());
    assert!(report.filtered_out_findings.is_empty());
    let pins = actual_pins(&root, profile);
    let mut value = document();
    value["old_queue"] = actual_queue(&old, &old_queue);
    value["new_queue"] = actual_queue(&new, &new_queue);
    value["unmatched_old_items"] =
        json!(old_queue.items.iter().map(actual_endpoint).collect::<Vec<_>>());
    value["unmatched_new_items"] =
        json!(new_queue.items.iter().map(actual_endpoint).collect::<Vec<_>>());
    value["impact"]["manifest"] = actual_original(&root, "impact-manifest.json");
    value["impact"]["report"] = actual_original(&root, "impact-report.json");
    value["impact"]["locator"] = actual_original(&root, "impact-locator.json");
    value["impact"]["source_pins"] = json!(pins);
    value["impact"]["old"] = actual_resource(&report.old);
    value["impact"]["new"] = actual_resource(&report.new);
    assert_eq!(
        report.old.resource_type,
        if profile { ResourceType::Profile } else { ResourceType::Catalog }
    );
    value["impact"]["old_resolved_source_key"] =
        if profile { json!("source-resolved") } else { Value::Null };
    value["impact"]["new_resolved_source_key"] =
        if profile { json!("source-resolved") } else { Value::Null };
    value["impact"]["summary"] = serde_json::to_value(&report.summary).unwrap();
    value["impact"]["change_count"] = json!(report.changes.len());
    value["impact"]["finding_count"] = json!(report.findings.len());
    value["impact"]["matched_findings"] = json!(report.matched_findings);
    let mut ids: Vec<&str> = report.findings.iter().map(|row| row.finding_id.as_str()).collect();
    ids.sort_unstable();
    value["impact"]["unreferenced_finding_ids"] = json!(ids);
    let s = &report.summary;
    value["impact"]["native_change_observation"] = json!(if s.added == 0
        && s.removed == 0
        && s.content_changed == 0
        && s.identity_migrated == 0
        && report.findings.is_empty()
    {
        "no-detected-native-change"
    } else {
        "detected-native-change"
    });
    value["counts"] = json!({"old_items":old_queue.items.len(),"new_items":new_queue.items.len(),"link_edges":0,"linked_old_items":0,"linked_new_items":0,"unmatched_old_items":old_queue.items.len(),"unmatched_new_items":new_queue.items.len(),"link_finding_occurrences":0,"distinct_linked_findings":0,"unreferenced_findings":report.findings.len()});
    Case { directory, old, new, value }
}

/// Exercise the actual proposed decoder and both maintained original queue decoders on one ledger.
fn bound(
    case: &Case,
    value: &Value,
    old_raw: &[u8],
    new_raw: &[u8],
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    assert!(case.directory.path().is_dir());
    let raw = serde_json::to_vec(value).unwrap();
    let mut ledger = ContractLedger::default();
    let old = queue_decode::decode_queue(old_raw, &mut ledger, &mut NoopControl)?;
    let new = queue_decode::decode_queue(new_raw, &mut ledger, &mut NoopControl)?;
    let recorded = decode(&raw, &mut ledger, &mut NoopControl)?;
    bind_queues(&recorded, &old, &new, &mut ledger, control)
}

/// Obtain expected plain differences from the independently maintained real E graph interpreter.
fn linked(case: &Case) -> Value {
    let mut ledger = ContractLedger::default();
    let old = queue_decode::decode_queue(&case.old, &mut ledger, &mut NoopControl).unwrap();
    let new = queue_decode::decode_queue(&case.new, &mut ledger, &mut NoopControl).unwrap();
    let request_raw = serde_json::to_vec(&json!({"schema_version":graph::LINKS_SCHEMA,"links":[{"old_item_id":old.document().items[0].item_id,"new_item_id":new.document().items[0].item_id,"finding_ids":[]}]})).unwrap();
    let request = graph::decode_links(&request_raw, &mut ledger, &mut NoopControl).unwrap();
    let graph = graph::prepare_graph(&old, &new, &request, &mut ledger, &mut NoopControl).unwrap();
    let link = &graph.links()[0];
    let mut value = case.value.clone();
    value["links"] = json!([{"old":actual_endpoint(link.old_item()),"new":actual_endpoint(link.new_item()),"differences":link.differences(),"relation":"explicit-declared-lineage","findings":[]}]);
    value["unmatched_old_items"] =
        json!(graph.unmatched_old().iter().map(|row| actual_endpoint(row)).collect::<Vec<_>>());
    value["unmatched_new_items"] =
        json!(graph.unmatched_new().iter().map(|row| actual_endpoint(row)).collect::<Vec<_>>());
    for (key, count) in [
        ("link_edges", 1),
        ("linked_old_items", 1),
        ("linked_new_items", 1),
        ("unmatched_old_items", graph.unmatched_old().len()),
        ("unmatched_new_items", graph.unmatched_new().len()),
    ] {
        value["counts"][key] = json!(count);
    }
    value
}

/// Actual Catalog/Profile originals bind in changed and unchanged native-report cases.
#[test]
fn genuine_generated_queue_and_full_report_data_bind_for_both_native_models() {
    for profile in [false, true] {
        for changed in [false, true] {
            let case = case(profile, changed);
            assert_eq!(bound(&case, &case.value, &case.old, &case.new, &mut NoopControl), Ok(()));
            let linked = linked(&case);
            assert_eq!(bound(&case, &linked, &case.old, &case.new, &mut NoopControl), Ok(()));
        }
    }
}

/// Exact original hashes and extents include whitespace, independent of unchanged typed queue data.
#[test]
fn whole_queue_original_hash_length_id_and_source_roster_mismatches_refuse() {
    let case = case(false, false);
    for (path, replacement) in [
        ("/old_queue/raw_sha256", json!("b".repeat(64))),
        ("/old_queue/byte_length", json!(case.old.len() + 1)),
        ("/old_queue/queue_id", json!(id(61))),
        ("/old_queue/source_pins/0/raw_sha256", json!("b".repeat(64))),
        ("/old_queue/source_pins/0/byte_length", json!(2)),
    ] {
        let mut value = case.value.clone();
        *value.pointer_mut(path).unwrap() = replacement;
        assert_eq!(
            bound(&case, &value, &case.old, &case.new, &mut NoopControl),
            Err(ContractError::Binding)
        );
    }
    let mut altered = case.old.clone();
    altered.push(b'\n');
    assert_eq!(
        bound(&case, &case.value, &altered, &case.new, &mut NoopControl),
        Err(ContractError::Binding)
    );
    let mut value = case.value.clone();
    value["created_at"] = json!("2026-10-03T00:00:00Z");
    assert_eq!(
        bound(&case, &value, &case.old, &case.new, &mut NoopControl),
        Err(ContractError::Binding)
    );
}

/// All eleven endpoint fields remain bound to actual items, including complete source keys.
#[test]
fn every_recorded_endpoint_binding_field_is_checked_against_actual_items() {
    let case = case(false, false);
    for (key, replacement) in [
        ("key", json!("forged-key")),
        ("item_id", json!(id(61))),
        ("subject_id", json!(id(62))),
        ("subject_sha256", json!("b".repeat(64))),
        ("context_sha256", json!("b".repeat(64))),
        ("policy_key", json!("forged-policy")),
        ("policy_sha256", json!("b".repeat(64))),
        ("source_keys", json!(["source"])),
    ] {
        let mut value = case.value.clone();
        value["unmatched_old_items"][0][key] = replacement;
        assert_eq!(
            bound(&case, &value, &case.old, &case.new, &mut NoopControl),
            Err(ContractError::Binding),
            "endpoint {key}"
        );
    }
    let mut value = case.value.clone();
    value["unmatched_old_items"][0]["domain"] = json!("applicability-decision");
    value["unmatched_old_items"][0]["adapter_version"] = json!("forge.applicability-review/1");
    assert_eq!(
        bound(&case, &value, &case.old, &case.new, &mut NoopControl),
        Err(ContractError::Binding)
    );
    for (key, replacement) in
        [("adapter_version", "forge.mapping-review/2"), ("requested_action", "approve")]
    {
        let mut value = case.value.clone();
        value["unmatched_old_items"][0][key] = json!(replacement);
        assert_eq!(
            bound(&case, &value, &case.old, &case.new, &mut NoopControl),
            Err(ContractError::Invalid)
        );
    }
}

/// Actual E tuple differences are recomputed from full context/policy/roster data, not trusted booleans.
#[test]
fn all_four_recorded_difference_bits_are_recomputed_from_actual_queue_inputs() {
    for profile in [false, true] {
        let case = case(profile, true);
        let baseline = linked(&case);
        assert_eq!(bound(&case, &baseline, &case.old, &case.new, &mut NoopControl), Ok(()));
        for key in ["subject", "context", "policy", "source_pins"] {
            let mut value = baseline.clone();
            value["links"][0]["differences"][key] =
                json!(!value["links"][0]["differences"][key].as_bool().unwrap());
            assert_eq!(
                bound(&case, &value, &case.old, &case.new, &mut NoopControl),
                Err(ContractError::Binding),
                "complete {key} tuple"
            );
        }
    }
}

/// An ordinary exact-binding refusal cannot mask the same actual final control stop.
#[test]
fn binding_failure_final_fence_and_prior_stop_use_the_original_ledger() {
    let case = case(false, false);
    let mut value = case.value.clone();
    value["old_queue"]["raw_sha256"] = json!("b".repeat(64));
    let mut observe = Observe { calls: 0, stop: None, failed: false };
    assert_eq!(
        bound(&case, &value, &case.old, &case.new, &mut observe),
        Err(ContractError::Binding)
    );
    let last = observe.calls;
    for failed in [false, true] {
        let mut control = Observe { calls: 0, stop: Some(last), failed };
        let error = if failed {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::DeadlineExceeded)
        };
        assert_eq!(bound(&case, &value, &case.old, &case.new, &mut control), Err(error));
    }
    let raw = serde_json::to_vec(&case.value).unwrap();
    let mut ledger = ContractLedger::default();
    let old = queue_decode::decode_queue(&case.old, &mut ledger, &mut NoopControl).unwrap();
    let new = queue_decode::decode_queue(&case.new, &mut ledger, &mut NoopControl).unwrap();
    let recorded = decode(&raw, &mut ledger, &mut NoopControl).unwrap();
    let mut control = Observe { calls: 0, stop: Some(1), failed: false };
    let first = ContractError::Interrupted(Interruption::DeadlineExceeded);
    assert_eq!(bind_queues(&recorded, &old, &new, &mut ledger, &mut control), Err(first));
    let calls = control.calls;
    assert_eq!(bind_queues(&recorded, &old, &new, &mut ledger, &mut control), Err(first));
    assert_eq!(control.calls, calls);
}

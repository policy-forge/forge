//! Prospective recorded-data controls. Inert fixtures grant no native authority.
//! Genuine command tests separately consume actual projection/encoding/publication.

use super::*;
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkError, WorkResult,
};
use serde_json::{Value, json};

/// Canonical inert review/native UUID, never a captured identity.
fn id(number: u64) -> String {
    format!("00000000-0000-4000-8000-{number:012x}")
}
/// Inert fixed-length digest syntax only.
fn hash() -> String {
    "a".repeat(64)
}
/// All fifteen recorded native counters for two unchanged controls and no findings.
fn summary_value() -> Value {
    json!({"old_controls":2,"new_controls":2,"added":0,"removed":0,"content_changed":0,"identity_migrated":0,"unchanged":2,"findings":0,"blocking":0,"review_required":0,"informational":0,"dispositioned_resolved":0,"dispositioned_accepted_risk":0,"dispositioned_still_open":0,"undispositioned":0})
}
/// Exact endpoint whitelist used only as plain recorded data.
fn endpoint(number: u64) -> Value {
    json!({"key":format!("item-{number}"),"item_id":id(number),"domain":"mapping-assertion","adapter_version":"forge.mapping-review/1","subject_id":id(number+30),"subject_sha256":hash(),"context_sha256":hash(),"policy_key":"policy","policy_sha256":hash(),"requested_action":"re-review","source_keys":["mapping","source"]})
}
/// Complete sorted recorded `SourcePin` roster with one explicit framework key.
fn queue(number: u64) -> Value {
    json!({"schema_version":"forge.review-queue/1","queue_id":id(number),"raw_sha256":hash(),"byte_length":1,"source_pins":[{"artifact_key":"mapping","model":"mapping","native_root_uuid":id(20),"raw_sha256":hash(),"byte_length":1,"schema_identity":"oscal:1.2.3:mapping-collection"},{"artifact_key":"source","model":"catalog","native_root_uuid":id(21),"raw_sha256":hash(),"byte_length":1,"schema_identity":"oscal:1.2.3:catalog"}],"item_count":1})
}
/// Closed recorded role occurrence with no physical registration/currentness assertion.
fn pin(role: &str, ordinal: usize, kind: &str, schema: &str, root: Option<&str>) -> Value {
    json!({"key":format!("impact:{role}:{ordinal}"),"kind":kind,"native_root_uuid":root,"raw_sha256":hash(),"byte_length":1,"schema_identity":schema})
}
/// Exact five-field inert native projection.
fn resource() -> Value {
    json!({"resource_type":"catalog","raw_sha256":hash(),"root_uuid":id(21),"oscal_version":"1.2.3","resolved_catalog_sha256":null})
}
/// Closed complete inert record, with all queue items explicitly unmatched.
fn document() -> Value {
    let root = id(21);
    let original = json!({"raw_sha256":hash(),"byte_length":1});
    json!({"schema_version":"forge.review-queue-supersession/1","identity_disclaimer":"keys-roles-authors-and-times-are-asserted-not-authenticated-signed-or-non-repudiable","sensitivity":"ids-and-hashes","supersession_id":id(1),"created_at":"2026-10-05T00:00:00Z","semantics":"declared-lineage-no-response-transfer","old_queue_currentness":"historical-unverified","new_queue_currentness":"recorded-current-at-creation-fence","old_queue":queue(2),"new_queue":queue(3),"impact":{"manifest":original,"report":original,"locator":original,"schema_version":"forge.framework-impact-report/1","status":"complete","currentness":"captured-recomputed-at-creation-fence","source_pins":[pin("manifest",0,"framework-impact-manifest","forge.framework-impact/1",None),pin("current-report",1,"framework-impact-report","forge.framework-impact-report/1",None),pin("old-catalog",2,"catalog","oscal:1.2.3:catalog",Some(&root)),pin("new-catalog",3,"catalog","oscal:1.2.3:catalog",Some(&root)),pin("mapping",4,"mapping","oscal:1.2.3:mapping-collection",Some(&id(20)))],"old_framework_source_key":"source","new_framework_source_key":"source","old_resolved_source_key":null,"new_resolved_source_key":null,"old":resource(),"new":resource(),"summary":summary_value(),"filters":{"group":null,"decision_state":null,"policy_source":null,"priority":null,"owner":null},"change_count":2,"finding_count":0,"matched_findings":0,"native_change_observation":"no-detected-native-change","distinct_linked_findings":0,"unreferenced_finding_ids":[],"prior_only_disposition_count":0},"links":[],"unmatched_old_items":[endpoint(4)],"unmatched_new_items":[endpoint(5)],"counts":{"old_items":1,"new_items":1,"link_edges":0,"linked_old_items":0,"linked_new_items":0,"unmatched_old_items":1,"unmatched_new_items":1,"link_finding_occurrences":0,"distinct_linked_findings":0,"unreferenced_findings":0}})
}
/// Execute the actual proposed decoder against a complete serialized inert record.
fn decoded(value: &Value) -> Result<(), ContractError> {
    let raw = serde_json::to_vec(value).unwrap();
    decode(&raw, &mut ContractLedger::default(), &mut NoopControl).map(|_| ())
}
/// Declare one coherent native family row as inert data, without a dependency proof.
fn finding(number: u64) -> Value {
    json!({"finding_id":id(number),"priority":"review-required","reason_code":"mapping_subject_changed","required_action":"reapprove-mapping-rationale","native_subject_id":"a-1","change_class":"content-changed","old_dependency_binding":"captured-native-map-id-and-declared-old-pin","new_relation":"explicit-declared-review-lineage"})
}
/// Declare a full many-to-many graph with repeated finding uses and exact distinct counts.
fn many() -> Value {
    let mut value = document();
    let edge = |a, b, rows: Vec<Value>| json!({"old":endpoint(a),"new":endpoint(b),"differences":{"subject":true,"context":true,"policy":true,"source_pins":false},"relation":"explicit-declared-lineage","findings":rows});
    value["old_queue"]["item_count"] = json!(2);
    value["new_queue"]["item_count"] = json!(2);
    value["links"] = json!([
        edge(4, 5, vec![finding(40)]),
        edge(4, 7, vec![finding(40), finding(41)]),
        edge(6, 7, vec![finding(41)])
    ]);
    value["unmatched_old_items"] = json!([]);
    value["unmatched_new_items"] = json!([]);
    value["counts"] = json!({"old_items":2,"new_items":2,"link_edges":3,"linked_old_items":2,"linked_new_items":2,"unmatched_old_items":0,"unmatched_new_items":0,"link_finding_occurrences":4,"distinct_linked_findings":2,"unreferenced_findings":1});
    value["impact"]["summary"]["content_changed"] = json!(1);
    value["impact"]["summary"]["unchanged"] = json!(1);
    for key in ["findings", "undispositioned"] {
        value["impact"]["summary"][key] = json!(3);
    }
    value["impact"]["summary"]["review_required"] = json!(2);
    value["impact"]["summary"]["informational"] = json!(1);
    for key in ["finding_count", "matched_findings"] {
        value["impact"][key] = json!(3);
    }
    value["impact"]["distinct_linked_findings"] = json!(2);
    value["impact"]["unreferenced_finding_ids"] = json!([id(42)]);
    value["impact"]["native_change_observation"] = json!("detected-native-change");
    value
}

/// Original LF/whitespace participates in the retained raw digest; recorded labels stay inert.
#[test]
fn exact_recorded_original_and_native_uuid_spelling_are_retained() {
    let mut value = document();
    value["impact"]["source_pins"].as_array_mut().unwrap().push(pin(
        "mapping",
        5,
        "mapping",
        "oscal:1.2.3:mapping-collection",
        Some("ABCDEFAB-ABCD-5ABC-8ABC-ABCDEFABCDEF"),
    ));
    let mut raw = serde_json::to_vec_pretty(&value).unwrap();
    raw.push(b'\n');
    let recorded = decode(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(recorded.raw(), raw);
    assert_eq!(recorded.raw_sha256(), crate::hashing::sha256_hex(&raw));
    assert_eq!(
        recorded.document().impact.source_pins[5].native_root_uuid.as_deref(),
        Some("ABCDEFAB-ABCD-5ABC-8ABC-ABCDEFABCDEF")
    );
    assert_eq!(recorded.document().impact.change_count, 2);
    assert!(
        recorded.document().impact.native_change_observation
            == NativeChangeObservation::NoneDetected
    );
}

/// Real strict parser rejects ambiguous keys, malformed UTF-8/BOM, excessive depth and strings.
#[test]
fn strict_original_admission_refuses_duplicates_depth_strings_and_bom() {
    let valid = serde_json::to_string(&document()).unwrap();
    let duplicate = valid.replacen(
        "\"supersession_id\":",
        "\"supersession_id\":\"00000000-0000-4000-8000-000000000099\",\"supersession_id\":",
        1,
    );
    let nested = valid.replacen("\"added\":0", "\"added\":0,\"added\":0", 1);
    let depth = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    for raw in [
        Vec::new(),
        vec![0xff],
        b"{".to_vec(),
        [vec![0xef, 0xbb, 0xbf], valid.as_bytes().to_vec()].concat(),
        duplicate.into_bytes(),
        nested.into_bytes(),
        depth.into_bytes(),
    ] {
        assert_eq!(
            decode(&raw, &mut ContractLedger::default(), &mut NoopControl).err(),
            Some(ContractError::Invalid)
        );
    }
    let mut huge = document();
    huge["unmatched_old_items"][0]["subject_id"] = json!("a".repeat(65_537));
    assert_eq!(decoded(&huge), Err(ContractError::Invalid));
    let oversize = vec![b' '; MAX_RAW + 1];
    assert_eq!(
        decode(&oversize, &mut ContractLedger::default(), &mut NoopControl).err(),
        Some(ContractError::Capacity)
    );
}

/// Required closed fields and explicit nulls cannot be omitted or expanded with private prose.
#[test]
fn closed_required_fields_null_filters_and_native_shapes_are_enforced() {
    let baseline = document();
    for path in
        ["", "/impact", "/impact/filters", "/impact/old", "/impact/new", "/old_queue/source_pins/0"]
    {
        let keys: Vec<String> =
            baseline.pointer(path).unwrap().as_object().unwrap().keys().cloned().collect();
        for key in keys {
            let mut value = baseline.clone();
            value.pointer_mut(path).unwrap().as_object_mut().unwrap().remove(&key);
            assert_eq!(decoded(&value), Err(ContractError::Invalid), "required {path}/{key}");
        }
        let mut value = baseline.clone();
        value.pointer_mut(path).unwrap()["private_prose"] = json!("Synthetic private text");
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
    for key in ["group", "decision_state", "policy_source", "priority", "owner"] {
        let mut value = baseline.clone();
        value["impact"]["filters"][key] = json!("nonempty");
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
}

/// All ten graph counts and all fifteen summary cells correlate with complete arrays.
#[test]
fn every_graph_and_summary_counter_is_checked_without_partial_credit() {
    let baseline = many();
    assert_eq!(decoded(&baseline), Ok(()));
    for path in ["/counts", "/impact/summary"] {
        for key in baseline.pointer(path).unwrap().as_object().unwrap().keys() {
            let mut value = baseline.clone();
            let slot = &mut value.pointer_mut(path).unwrap()[key];
            *slot = json!(slot.as_u64().unwrap() + 1);
            assert_eq!(decoded(&value), Err(ContractError::Invalid), "counter {path}/{key}");
        }
    }
    for key in ["change_count", "finding_count", "matched_findings", "distinct_linked_findings"] {
        let mut value = baseline.clone();
        value["impact"][key] = json!(value["impact"][key].as_u64().unwrap() + 1);
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
}

/// Repeated many-to-many endpoints and findings must agree in every retained field.
#[test]
fn many_to_many_repeated_rows_and_sorted_distinct_pairs_are_conserved() {
    let baseline = many();
    assert_eq!(decoded(&baseline), Ok(()));
    let mut value = baseline.clone();
    value["links"][1]["old"]["policy_sha256"] = json!("b".repeat(64));
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = baseline.clone();
    value["links"][1]["findings"][0]["native_subject_id"] = json!("a-2");
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = baseline.clone();
    value["links"].as_array_mut().unwrap().swap(0, 1);
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = baseline.clone();
    value["links"][1] = value["links"][0].clone();
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = baseline.clone();
    value["links"][1]["findings"].as_array_mut().unwrap().swap(0, 1);
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = document();
    value["unmatched_old_items"].as_array_mut().unwrap().push(endpoint(4));
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
}

/// Recorded native reason/class/action families cannot contradict their declared old domain.
#[test]
fn exact_adapter_and_finding_family_correlations_refuse_cross_family_rows() {
    let baseline = many();
    for (key, replacement) in [
        ("old_dependency_binding", "captured-native-control-id-and-declared-old-pin"),
        ("reason_code", "resource_metadata_changed"),
        ("required_action", "review-resource-metadata"),
        ("change_class", "unchanged"),
        ("priority", "informational"),
    ] {
        let mut value = baseline.clone();
        value["links"][0]["findings"][0][key] = json!(replacement);
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
    let mut value = document();
    value["unmatched_old_items"][0]["adapter_version"] = json!("forge.mapping-review/2");
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = baseline.clone();
    value["links"][0]["new"]["domain"] = json!("applicability-decision");
    value["links"][0]["new"]["adapter_version"] = json!("forge.applicability-review/1");
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
}

/// Metadata-only complete findings force detected change even with unchanged control rows.
#[test]
fn zero_observation_uses_four_change_counters_and_complete_finding_denominator() {
    let mut value = document();
    for key in ["findings", "informational", "undispositioned"] {
        value["impact"]["summary"][key] = json!(1);
    }
    value["impact"]["finding_count"] = json!(1);
    value["impact"]["matched_findings"] = json!(1);
    value["impact"]["unreferenced_finding_ids"] = json!([id(40)]);
    value["counts"]["unreferenced_findings"] = json!(1);
    value["impact"]["native_change_observation"] = json!("detected-native-change");
    assert_eq!(decoded(&value), Ok(()));
    value["impact"]["native_change_observation"] = json!("no-detected-native-change");
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut migrated = document();
    migrated["impact"]["summary"]["old_controls"] = json!(3);
    migrated["impact"]["summary"]["new_controls"] = json!(4);
    migrated["impact"]["summary"]["identity_migrated"] = json!(1);
    migrated["impact"]["summary"]["unchanged"] = json!(0);
    migrated["impact"]["change_count"] = json!(1);
    migrated["impact"]["native_change_observation"] = json!("detected-native-change");
    for key in ["findings", "review_required", "undispositioned"] {
        migrated["impact"]["summary"][key] = json!(1);
    }
    migrated["impact"]["finding_count"] = json!(1);
    migrated["impact"]["matched_findings"] = json!(1);
    migrated["impact"]["unreferenced_finding_ids"] = json!([id(40)]);
    migrated["counts"]["unreferenced_findings"] = json!(1);
    assert_eq!(decoded(&migrated), Ok(()));
}

/// Unreferenced finding IDs form the complete sorted disjoint complement, not a second history.
#[test]
fn unreferenced_ids_are_sorted_unique_disjoint_and_complete() {
    let baseline = many();
    for ids in [vec![id(40)], vec![id(42), id(42)], vec![id(43), id(42)]] {
        let mut value = baseline.clone();
        value["impact"]["unreferenced_finding_ids"] = json!(ids);
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
    let mut value = document();
    value["impact"]["prior_only_disposition_count"] = json!(1);
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
}

/// Exact global ordinal roles and fixed native schema/null tuples are required.
#[test]
fn source_roles_and_private_schema_identity_vocabularies_are_closed() {
    let baseline = document();
    for replacement in [
        "old-framework:0",
        "impact:old-catalog:02",
        "impact:old-catalog:3",
        "impact:prior-report:2",
    ] {
        let mut value = baseline.clone();
        value["impact"]["source_pins"][2]["key"] = json!(replacement);
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
    for (path, replacement) in [
        ("/impact/source_pins/2/schema_identity", json!("oscal:1.2.3:profile")),
        ("/impact/source_pins/2/native_root_uuid", Value::Null),
        ("/impact/source_pins/0/native_root_uuid", json!(id(21))),
        ("/old_queue/source_pins/1/schema_identity", json!("oscal:1.2.3:catalog:private-version")),
        ("/impact/manifest/raw_sha256", json!("b".repeat(64))),
        ("/impact/old_framework_source_key", json!("mapping")),
    ] {
        let mut value = baseline.clone();
        *value.pointer_mut(path).unwrap() = replacement;
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
}

/// Recorded role lengths retain stricter native/Aux limits without a new capacity pool.
#[test]
fn recorded_original_role_lengths_respect_native_and_auxiliary_bounds() {
    let baseline = document();
    for length in [0, 1_048_577] {
        let mut value = baseline.clone();
        value["impact"]["locator"]["byte_length"] = json!(length);
        assert_eq!(decoded(&value), Err(ContractError::Invalid));
    }
    let mut value = baseline.clone();
    value["impact"]["manifest"]["byte_length"] = json!(2_097_153);
    value["impact"]["source_pins"][0]["byte_length"] = json!(2_097_153);
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    let mut value = baseline.clone();
    value["impact"]["source_pins"].as_array_mut().unwrap().push(pin(
        "successor",
        5,
        "successor-map",
        "forge.successor-map/1",
        None,
    ));
    value["impact"]["source_pins"][5]["byte_length"] = json!(2_097_153);
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
}

/// Explicit Profile companions require exact queue/native kind/root/hash/key coherence.
#[test]
fn profile_resolved_companions_are_explicit_and_correlated() {
    let mut value = document();
    let root = id(21);
    let resolved = id(22);
    value["impact"]["source_pins"] = json!([
        pin("manifest", 0, "framework-impact-manifest", "forge.framework-impact/1", None),
        pin(
            "current-report",
            1,
            "framework-impact-report",
            "forge.framework-impact-report/1",
            None
        ),
        pin("old-profile", 2, "profile", "oscal:1.2.3:profile", Some(&root)),
        pin("old-resolved", 3, "resolved-catalog", "oscal:1.2.3:catalog", Some(&resolved)),
        pin("new-profile", 4, "profile", "oscal:1.2.3:profile", Some(&root)),
        pin("new-resolved", 5, "resolved-catalog", "oscal:1.2.3:catalog", Some(&resolved))
    ]);
    for side in ["old", "new"] {
        value["impact"][side]["resource_type"] = json!("profile");
        value["impact"][side]["resolved_catalog_sha256"] = json!(hash());
        value["impact"][format!("{side}_resolved_source_key")] = json!("source-resolved");
        let queue = &mut value[format!("{side}_queue")];
        queue["source_pins"][1]["model"] = json!("profile");
        queue["source_pins"][1]["schema_identity"] = json!("oscal:1.2.3:profile");
        queue["source_pins"].as_array_mut().unwrap().push(json!({"artifact_key":"source-resolved","model":"resolved-catalog","native_root_uuid":resolved,"raw_sha256":hash(),"byte_length":1,"schema_identity":"oscal:1.2.3:catalog"}));
    }
    assert_eq!(decoded(&value), Ok(()));
    for path in ["/impact/old_resolved_source_key", "/impact/old/resolved_catalog_sha256"] {
        let mut forged = value.clone();
        *forged.pointer_mut(path).unwrap() = Value::Null;
        assert_eq!(decoded(&forged), Err(ContractError::Invalid));
    }
    let mut forged = value.clone();
    forged["old_queue"]["source_pins"][2]["raw_sha256"] = json!("b".repeat(64));
    assert_eq!(decoded(&forged), Err(ContractError::Invalid));
}

/// Complete optional read roles retain compatible repeats and require paired history inputs.
#[test]
fn configured_applicability_successor_and_history_roles_are_complete_and_ordered() {
    let mut value = document();
    let root = id(21);
    let pins = value["impact"]["source_pins"].as_array_mut().unwrap();
    pins.push(pin(
        "applicability-manifest",
        5,
        "applicability-manifest",
        "forge.applicability/1",
        None,
    ));
    pins.push(pin("applicability-catalog", 6, "catalog", "oscal:1.2.3:catalog", Some(&root)));
    pins.push(pin(
        "applicability-mapping",
        7,
        "mapping",
        "oscal:1.2.3:mapping-collection",
        Some(&id(20)),
    ));
    pins.push(pin("successor", 8, "successor-map", "forge.successor-map/1", None));
    pins.push(pin(
        "prior-report",
        9,
        "framework-impact-report",
        "forge.framework-impact-report/1",
        None,
    ));
    pins.push(pin(
        "dispositions",
        10,
        "framework-impact-dispositions",
        "forge.framework-impact-dispositions/1",
        None,
    ));
    value["impact"]["prior_only_disposition_count"] = json!(1);
    assert_eq!(decoded(&value), Ok(()));
    let mut forged = value.clone();
    forged["impact"]["source_pins"][6]["raw_sha256"] = json!("b".repeat(64));
    assert_eq!(decoded(&forged), Err(ContractError::Invalid));
    let mut forged = value.clone();
    forged["impact"]["source_pins"].as_array_mut().unwrap().pop();
    assert_eq!(decoded(&forged), Err(ContractError::Invalid));
    let mut forged = document();
    forged["new_queue"]["queue_id"] = forged["old_queue"]["queue_id"].clone();
    assert_eq!(decoded(&forged), Err(ContractError::Invalid));
    let mut forged = document();
    forged["created_at"] = json!("2026-02-30T00:00:00Z");
    assert_eq!(decoded(&forged), Err(ContractError::Invalid));
}

/// Complete inert applicability edge, including its native base-finding complement and selected old tuple.
fn applicability() -> Value {
    let mut value = document();
    let root = id(21);
    value["impact"]["source_pins"].as_array_mut().unwrap().extend([
        pin("applicability-manifest", 5, "applicability-manifest", "forge.applicability/1", None),
        pin("applicability-catalog", 6, "catalog", "oscal:1.2.3:catalog", Some(&root)),
    ]);
    for side in ["old_queue", "new_queue"] {
        value[side]["source_pins"].as_array_mut().unwrap().insert(0,json!({
            "artifact_key":"applicability-manifest","model":"applicability-manifest",
            "native_root_uuid":null,"raw_sha256":hash(),"byte_length":1,"schema_identity":"forge.applicability/1"}));
    }
    let mut old = endpoint(4);
    let mut new = endpoint(5);
    for row in [&mut old, &mut new] {
        row["domain"] = json!("applicability-decision");
        row["adapter_version"] = json!("forge.applicability-review/1");
        row["source_keys"] = json!(["applicability-manifest", "source"]);
    }
    old["subject_id"] = json!("a-1");
    new["subject_id"] = json!("explicit-new-control");
    let mut reference = finding(40);
    reference["reason_code"] = json!("applicability_decision_changed");
    reference["required_action"] = json!("review-applicability-decision");
    reference["old_dependency_binding"] = json!("captured-native-control-id-and-declared-old-pin");
    value["links"] = json!([{"old":old,"new":new,"differences":{"subject":true,"context":true,"policy":true,"source_pins":false},"relation":"explicit-declared-lineage","findings":[reference]}]);
    value["unmatched_old_items"] = json!([]);
    value["unmatched_new_items"] = json!([]);
    for key in [
        "link_edges",
        "linked_old_items",
        "linked_new_items",
        "link_finding_occurrences",
        "distinct_linked_findings",
    ] {
        value["counts"][key] = json!(1);
    }
    for key in ["unmatched_old_items", "unmatched_new_items"] {
        value["counts"][key] = json!(0);
    }
    value["impact"]["summary"]["content_changed"] = json!(1);
    value["impact"]["summary"]["unchanged"] = json!(1);
    for key in ["findings", "undispositioned"] {
        value["impact"]["summary"][key] = json!(2);
    }
    value["impact"]["summary"]["review_required"] = json!(1);
    value["impact"]["summary"]["informational"] = json!(1);
    for key in ["finding_count", "matched_findings"] {
        value["impact"][key] = json!(2);
    }
    value["impact"]["distinct_linked_findings"] = json!(1);
    value["impact"]["unreferenced_finding_ids"] = json!([id(42)]);
    value["counts"]["unreferenced_findings"] = json!(1);
    value["impact"]["native_change_observation"] = json!("detected-native-change");
    value
}

/// Applicability findings name the exact old control, while new endpoint lineage remains explicit.
#[test]
fn applicability_finding_native_subject_matches_old_endpoint_only() {
    let mut value = applicability();
    assert_eq!(decoded(&value), Ok(()));
    value["links"][0]["findings"][0]["native_subject_id"] = json!("a-2");
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    // Mapping native control IDs remain distinct from map UUIDs; no new endpoint equality is inferred.
    assert_eq!(decoded(&many()), Ok(()));
}

/// Each linked native class needs a nonzero complete counter; aggregate arithmetic alone is insufficient.
#[test]
fn linked_finding_class_requires_corresponding_nonzero_full_summary_counter() {
    for (class, reason, key, priority, action) in [
        (
            "content-changed",
            "mapping_subject_changed",
            "content_changed",
            "review-required",
            "reapprove-mapping-rationale",
        ),
        (
            "removed",
            "mapping_reference_removed",
            "removed",
            "blocking",
            "repair-or-approve-mapping",
        ),
        (
            "identity-migrated",
            "mapping_subject_migrated",
            "identity_migrated",
            "review-required",
            "reapprove-mapping-rationale",
        ),
    ] {
        let mut value = many();
        for link in value["links"].as_array_mut().unwrap() {
            for row in link["findings"].as_array_mut().unwrap() {
                row["change_class"] = json!(class);
                row["reason_code"] = json!(reason);
                row["priority"] = json!(priority);
                row["required_action"] = json!(action);
            }
        }
        value["impact"]["summary"]["content_changed"] = json!(0);
        value["impact"]["summary"][key] = json!(1);
        if key == "removed" {
            value["impact"]["summary"]["new_controls"] = json!(1);
            value["impact"]["summary"]["blocking"] = json!(2);
            value["impact"]["summary"]["review_required"] = json!(0);
        }
        if key == "identity_migrated" {
            value["impact"]["summary"]["review_required"] = json!(3);
            value["impact"]["summary"]["informational"] = json!(0);
        }
        assert_eq!(decoded(&value), Ok(()), "coherent recorded {class} baseline");
        // Preserve every existing full aggregate equation, priority partition and native observation.
        value["impact"]["summary"][key] = json!(0);
        value["impact"]["summary"]["unchanged"] = json!(2);
        value["impact"]["summary"]["new_controls"] = json!(2);
        assert_eq!(decoded(&value), Err(ContractError::Invalid), "zero complete {class} counter");
    }
}

/// The complete complement retains at least one excluded native base ID per changed class row.
#[test]
fn native_base_findings_require_complete_unreferenced_lower_bound() {
    let mut value = many();
    assert_eq!(decoded(&value), Ok(()));
    value["impact"]["unreferenced_finding_ids"] = json!([]);
    value["counts"]["unreferenced_findings"] = json!(0);
    for key in ["findings", "undispositioned"] {
        value["impact"]["summary"][key] = json!(2);
    }
    value["impact"]["summary"]["informational"] = json!(0);
    for key in ["finding_count", "matched_findings"] {
        value["impact"][key] = json!(2);
    }
    assert_eq!(decoded(&value), Err(ContractError::Invalid));
    for key in ["added", "removed", "content_changed", "identity_migrated"] {
        let mut value = document();
        value["impact"]["summary"][key] = json!(1);
        value["impact"]["summary"]["unchanged"] = json!(1);
        value["impact"]["change_count"] = json!(2);
        if key == "added" {
            value["impact"]["summary"]["old_controls"] = json!(1);
        }
        if key == "removed" {
            value["impact"]["summary"]["new_controls"] = json!(1);
        }
        for cell in ["findings", "informational", "undispositioned"] {
            value["impact"]["summary"][cell] = json!(1);
        }
        if key == "added" || key == "identity_migrated" {
            value["impact"]["summary"]["informational"] = json!(0);
            value["impact"]["summary"]["review_required"] = json!(1);
        }
        value["impact"]["finding_count"] = json!(1);
        value["impact"]["matched_findings"] = json!(1);
        value["impact"]["unreferenced_finding_ids"] = json!([id(42)]);
        value["counts"]["unreferenced_findings"] = json!(1);
        value["impact"]["native_change_observation"] = json!("detected-native-change");
        assert_eq!(decoded(&value), Ok(()), "complete inert {key} base row");
        // Remove only the omitted base denominator while retaining all existing total equations.
        value["impact"]["unreferenced_finding_ids"] = json!([]);
        value["counts"]["unreferenced_findings"] = json!(0);
        for cell in ["findings", "informational", "review_required", "undispositioned"] {
            value["impact"]["summary"][cell] = json!(0);
        }
        value["impact"]["finding_count"] = json!(0);
        value["impact"]["matched_findings"] = json!(0);
        assert_eq!(decoded(&value), Err(ContractError::Invalid), "missing inert {key} base row");
    }
}

/// Linked families require a selected whole old dependency tuple, without map/Collection identity inference.
#[test]
fn linked_findings_require_selected_old_framework_and_dependency_source_tuples() {
    for baseline in [many(), applicability()] {
        assert_eq!(decoded(&baseline), Ok(()));
        let domain = baseline["links"][0]["old"]["domain"].as_str().unwrap();
        let dependency =
            if domain == "mapping-assertion" { "mapping" } else { "applicability-manifest" };
        for keys in [vec![dependency], vec!["source"]] {
            let mut value = baseline.clone();
            for link in value["links"].as_array_mut().unwrap() {
                link["old"]["source_keys"] = json!(keys);
            }
            assert_eq!(decoded(&value), Err(ContractError::Invalid));
        }
        for (field, replacement) in
            [("raw_sha256", json!("b".repeat(64))), ("byte_length", json!(2))]
        {
            let mut value = baseline.clone();
            let source = value["old_queue"]["source_pins"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|pin| pin["artifact_key"] == dependency)
                .unwrap();
            source[field] = replacement;
            assert_eq!(
                decoded(&value),
                Err(ContractError::Invalid),
                "whole {domain} tuple {field}"
            );
        }
    }
    let mut repeated = many();
    let matching = repeated["impact"]["source_pins"][4].clone();
    repeated["impact"]["source_pins"][4]["raw_sha256"] = json!("b".repeat(64));
    let mut matching = matching;
    matching["key"] = json!("impact:mapping:5");
    repeated["impact"]["source_pins"].as_array_mut().unwrap().push(matching);
    assert_eq!(decoded(&repeated), Ok(()));
    // The Collection root remains different from every private map UUID endpoint.
    assert_ne!(
        repeated["old_queue"]["source_pins"][0]["native_root_uuid"],
        repeated["links"][0]["old"]["subject_id"]
    );
    repeated["impact"]["source_pins"][5]["native_root_uuid"] = json!(id(99));
    assert_eq!(decoded(&repeated), Err(ContractError::Invalid));
    let mut no_role = many();
    no_role["impact"]["source_pins"][4]["key"] = json!("impact:applicability-mapping:4");
    assert_eq!(decoded(&no_role), Err(ContractError::Invalid));
}

/// Observe actual checkpoint calls and optionally stop one with the specified real failure.
struct Observe {
    /// Actual invocation count, never a source proof.
    calls: usize,
    /// Optional exact occurrence to refuse.
    stop: Option<usize>,
    /// Ordinary failure versus preserved typed deadline interruption.
    failed: bool,
}
impl WorkControl for Observe {
    /// Return the actual selected failure once observed.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.stop == Some(self.calls) {
            if self.failed {
                Err(WorkError::Failed(crate::workspace::contract::Error::invalid()))
            } else {
                Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
            }
        } else {
            Ok(())
        }
    }
    /// This one-shot probe exposes no maintained runtime owner or sticky success flag.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// An already-stopped original ledger blocks malformed/new work without consulting control.
#[test]
fn prior_capacity_and_late_ordinary_failure_fences_keep_first_stops_sticky() {
    let mut ledger = ContractLedger::default();
    ledger.derived(33_554_432).unwrap();
    assert_eq!(ledger.derived(1), Err(ContractError::Capacity));
    let mut control = Observe { calls: 0, stop: None, failed: false };
    assert_eq!(decode(b"", &mut ledger, &mut control).err(), Some(ContractError::Capacity));
    assert_eq!(control.calls, 0);
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut control = Observe { calls: 0, stop: Some(2), failed };
        let first = if failed {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::DeadlineExceeded)
        };
        assert_eq!(decode(b"", &mut ledger, &mut control).err(), Some(first));
        let calls = control.calls;
        assert_eq!(decode(b"{}", &mut ledger, &mut control).err(), Some(first));
        assert_eq!(control.calls, calls);
    }
}

/// Genuine real queue/report fixtures remain plain files/data for these binder controls.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "supersession_decode_mapping_tests.rs"]
mod mapping;

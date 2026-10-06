//! Prospective inert-record controls; no shape fixture is captured/native/current proof.

use super::*;
use serde_json::{Value, json};

/// Load only the new closed companion schema; ordinary /1 schemas are unchanged.
fn schema() -> Value {
    serde_json::from_str(include_str!(
        "../../schemas/forge.review-queue-supersession-1.schema.json"
    ))
    .expect("literal new schema")
}

/// Run the genuine maintained validator against an isolated named definition.
fn valid_definition(name: &str, value: &Value) -> bool {
    let mut definition = schema();
    definition.as_object_mut().expect("schema object").remove("required");
    definition.as_object_mut().expect("schema object").remove("properties");
    definition.as_object_mut().expect("schema object").remove("additionalProperties");
    definition["$ref"] = json!(format!("#/$defs/{name}"));
    jsonschema::validator_for(&definition).expect("actual new schema compilation").is_valid(value)
}

/// Produce a canonical review UUID for inert shape controls only.
fn id(number: u64) -> String {
    format!("00000000-0000-4000-8000-{number:012x}")
}

/// Produce a lowercase fixed-length inert digest, never a held-file hash.
fn digest() -> String {
    "a".repeat(64)
}

/// Declare an inert exact native source pin shape with uppercase UUID preserved.
fn source_pin() -> Value {
    json!({"key":"old-framework:0","kind":"catalog","native_root_uuid":"ABCDEFAB-ABCD-4ABC-8ABC-ABCDEFABCDEF","raw_sha256":digest(),"byte_length":1,"schema_identity":"oscal:1.2.3:catalog"})
}

/// Declare an inert five-field Catalog projection; no native resource is manufactured.
fn resource() -> Value {
    json!({"resource_type":"catalog","raw_sha256":digest(),"root_uuid":"ABCDEFAB-ABCD-4ABC-8ABC-ABCDEFABCDEF","oscal_version":"1.2.3","resolved_catalog_sha256":null})
}

/// Declare all fifteen complete summary keys, with two unchanged native rows recorded.
fn summary() -> Value {
    json!({"old_controls":2,"new_controls":2,"added":0,"removed":0,"content_changed":0,"identity_migrated":0,"unchanged":2,"findings":0,"blocking":0,"review_required":0,"informational":0,"dispositioned_resolved":0,"dispositioned_accepted_risk":0,"dispositioned_still_open":0,"undispositioned":0})
}

/// Declare all five mandatory explicit null filter keys.
fn filters() -> Value {
    json!({"group":null,"decision_state":null,"policy_source":null,"priority":null,"owner":null})
}

/// Declare an inert full `ImpactReference` shape; recorded labels restore no evidence.
fn impact() -> Value {
    let pin = json!({"raw_sha256":digest(),"byte_length":1});
    json!({"manifest":pin,"report":pin,"locator":pin,"schema_version":"forge.framework-impact-report/1","status":"complete","currentness":"captured-recomputed-at-creation-fence","source_pins":[source_pin()],"old_framework_source_key":"old-framework:0","new_framework_source_key":"new-framework:0","old_resolved_source_key":null,"new_resolved_source_key":null,"old":resource(),"new":resource(),"summary":summary(),"filters":filters(),"change_count":2,"finding_count":0,"matched_findings":0,"native_change_observation":"no-detected-native-change","distinct_linked_findings":0,"unreferenced_finding_ids":[],"prior_only_disposition_count":0})
}

/// Declare a complete existing queue reference shape, without genuine owner capture.
fn queue(number: u64) -> Value {
    json!({"schema_version":"forge.review-queue/1","queue_id":id(number),"raw_sha256":digest(),"byte_length":1,"source_pins":[{"artifact_key":"mapping","model":"mapping","native_root_uuid":id(20),"raw_sha256":digest(),"byte_length":1,"schema_identity":"oscal:1.2.3:mapping-collection"}],"item_count":1})
}

/// Declare the exact eleven public endpoint fields, without historical approval credit.
fn endpoint(number: u64) -> Value {
    json!({"key":format!("item-{number}"),"item_id":id(number),"domain":"mapping-assertion","adapter_version":"forge.mapping-review/1","subject_id":id(number+30),"subject_sha256":digest(),"context_sha256":digest(),"policy_key":"policy","policy_sha256":digest(),"requested_action":"re-review","source_keys":["mapping"]})
}

/// Declare a complete inert record suitable only for structural serialization controls.
fn document() -> Value {
    json!({"schema_version":"forge.review-queue-supersession/1","identity_disclaimer":"keys-roles-authors-and-times-are-asserted-not-authenticated-signed-or-non-repudiable","sensitivity":"ids-and-hashes","supersession_id":id(1),"created_at":"2026-10-05T00:00:00Z","semantics":"declared-lineage-no-response-transfer","old_queue_currentness":"historical-unverified","new_queue_currentness":"recorded-current-at-creation-fence","old_queue":queue(2),"new_queue":queue(3),"impact":impact(),"links":[],"unmatched_old_items":[endpoint(4)],"unmatched_new_items":[endpoint(5)],"counts":{"old_items":1,"new_items":1,"link_edges":0,"linked_old_items":0,"linked_new_items":0,"unmatched_old_items":1,"unmatched_new_items":1,"link_finding_occurrences":0,"distinct_linked_findings":0,"unreferenced_findings":0}})
}

/// Check the whole new schema through the actual validator, not a mirror predicate.
fn valid(value: &Value) -> bool {
    jsonschema::validator_for(&schema()).expect("actual new schema compilation").is_valid(value)
}

/// Every inert root field survives exact serde round-trip without any authority constructor.
#[test]
fn inert_record_roundtrip_has_only_complete_fixed_public_fields() {
    let value = document();
    assert!(valid(&value));
    let recorded: SupersessionDocument =
        serde_json::from_value(value.clone()).expect("inert record only");
    assert_eq!(serde_json::to_value(recorded).expect("inert serialization"), value);
    assert_eq!(value.as_object().expect("root").len(), 15);
    assert_eq!(value["impact"].as_object().expect("impact").len(), 22);
    assert_eq!(value["unmatched_old_items"][0].as_object().expect("endpoint").len(), 11);
}

/// All new object definitions require their entire closed property roster.
#[test]
fn schema_is_closed_and_requires_every_declared_object_field() {
    let value = schema();
    for object in
        std::iter::once(&value).chain(value["$defs"].as_object().expect("definitions").values())
    {
        if object.get("type").and_then(Value::as_str) == Some("object") {
            assert_eq!(object["additionalProperties"], json!(false));
            let properties = object["properties"].as_object().expect("properties");
            let required = object["required"].as_array().expect("required");
            assert_eq!(required.len(), properties.len());
            for key in properties.keys() {
                assert!(required.iter().any(|field| field.as_str() == Some(key)));
            }
        }
    }
    let full = document();
    for key in full.as_object().expect("root").keys() {
        let mut omitted = full.clone();
        omitted.as_object_mut().expect("root").remove(key);
        assert!(!valid(&omitted));
        assert!(serde_json::from_value::<SupersessionDocument>(omitted).is_err());
    }
}

/// All nine source kinds pair with fixed schema identities and exact native/null shapes.
#[test]
fn nine_impact_kinds_have_exact_schema_and_native_uuid_pairings() {
    let cases = [
        ("catalog", "oscal:1.2.3:catalog", true),
        ("profile", "oscal:1.2.3:profile", true),
        ("resolved-catalog", "oscal:1.2.3:catalog", true),
        ("mapping", "oscal:1.2.3:mapping-collection", true),
        ("applicability-manifest", "forge.applicability/1", false),
        ("framework-impact-manifest", "forge.framework-impact/1", false),
        ("framework-impact-report", "forge.framework-impact-report/1", false),
        ("successor-map", "forge.successor-map/1", false),
        ("framework-impact-dispositions", "forge.framework-impact-dispositions/1", false),
    ];
    for (kind, identity, native) in cases {
        let mut pin = source_pin();
        pin["kind"] = json!(kind);
        pin["schema_identity"] = json!(identity);
        if !native {
            pin["native_root_uuid"] = Value::Null;
        }
        assert!(valid_definition("ImpactSourcePin", &pin));
        let record: ImpactSourcePin =
            serde_json::from_value(pin.clone()).expect("plain fixed kind");
        assert_eq!(serde_json::to_value(record).expect("exact inert fields"), pin);
        pin["native_root_uuid"] = if native { Value::Null } else { json!(id(8)) };
        assert!(!valid_definition("ImpactSourcePin", &pin));
        pin["schema_identity"] = json!("oscal:1.2.3:document-version-2.0.0");
        assert!(!valid_definition("ImpactSourcePin", &pin));
        assert!(serde_json::from_value::<ImpactSourcePin>(pin).is_err());
    }
}

/// Required nullable fields never accept silent omission as null through serde or schema.
#[test]
fn explicit_nullable_fields_cannot_be_omitted() {
    let mut pin = source_pin();
    pin.as_object_mut().expect("source pin").remove("native_root_uuid");
    assert!(!valid_definition("ImpactSourcePin", &pin));
    assert!(serde_json::from_value::<ImpactSourcePin>(pin).is_err());
    let mut queue_pin = queue(1)["source_pins"][0].clone();
    queue_pin.as_object_mut().expect("queue pin").remove("native_root_uuid");
    assert!(!valid_definition("SourcePin", &queue_pin));
    assert!(serde_json::from_value::<RecordedQueueSourcePin>(queue_pin).is_err());
    let mut native = resource();
    native.as_object_mut().expect("resource").remove("resolved_catalog_sha256");
    assert!(!valid_definition("ImpactResource", &native));
    assert!(serde_json::from_value::<ImpactResource>(native).is_err());
    for key in ["old_resolved_source_key", "new_resolved_source_key"] {
        let mut record = impact();
        record.as_object_mut().expect("impact").remove(key);
        assert!(!valid_definition("ImpactReference", &record));
        assert!(serde_json::from_value::<ImpactReference>(record).is_err());
    }
}

/// All fifteen native summary values and five explicit null filter fields remain required.
#[test]
fn summary_and_empty_filters_have_no_partial_default_or_filtered_shape() {
    let full_summary = summary();
    assert_eq!(full_summary.as_object().expect("summary").len(), 15);
    for key in full_summary.as_object().expect("summary").keys() {
        let mut omitted = full_summary.clone();
        omitted.as_object_mut().expect("summary").remove(key);
        assert!(!valid_definition("ImpactSummary", &omitted));
        assert!(serde_json::from_value::<ImpactSummary>(omitted).is_err());
    }
    let empty = filters();
    assert_eq!(empty.as_object().expect("filters").len(), 5);
    for key in empty.as_object().expect("filters").keys() {
        let mut omitted = empty.clone();
        omitted.as_object_mut().expect("filters").remove(key);
        assert!(!valid_definition("EmptyFilters", &omitted));
        assert!(serde_json::from_value::<EmptyImpactFilters>(omitted).is_err());
        let mut selected = empty.clone();
        selected[key] = json!("private-selected-value");
        assert!(!valid_definition("EmptyFilters", &selected));
        assert!(serde_json::from_value::<EmptyImpactFilters>(selected).is_err());
    }
}

/// Catalog null companions and Profile exact companion/key shapes are distinct.
#[test]
fn profile_and_catalog_companion_shapes_use_fixed_actual_oscal_revision() {
    let mut record = impact();
    assert!(valid_definition("ImpactReference", &record));
    record["old"]["resolved_catalog_sha256"] = json!(digest());
    assert!(!valid_definition("ImpactReference", &record));
    for side in ["old", "new"] {
        record[side]["resource_type"] = json!("profile");
        record[side]["resolved_catalog_sha256"] = json!(digest());
    }
    record["old_resolved_source_key"] = json!("old-resolved:0");
    record["new_resolved_source_key"] = json!("new-resolved:0");
    assert!(valid_definition("ImpactReference", &record));
    let inert: ImpactReference =
        serde_json::from_value(record.clone()).expect("inert Profile shape");
    assert_eq!(serde_json::to_value(inert).expect("exact Profile shape"), record);
    record["new_resolved_source_key"] = Value::Null;
    assert!(!valid_definition("ImpactReference", &record));
    record["new_resolved_source_key"] = json!("new-resolved:0");
    record["old"]["oscal_version"] = json!("1.2.3 private-version");
    assert!(!valid_definition("ImpactReference", &record));
    assert!(serde_json::from_value::<ImpactReference>(record).is_err());
}

/// Metadata-only finding presence forbids the zero label even when four change counters are zero.
#[test]
fn zero_observation_uses_four_counters_and_complete_recorded_findings() {
    let baseline = impact();
    assert_eq!(baseline["change_count"], json!(2));
    assert!(valid_definition("ImpactReference", &baseline));
    for counter in ["added", "removed", "content_changed", "identity_migrated"] {
        let mut changed = baseline.clone();
        changed["summary"][counter] = json!(1);
        assert!(!valid_definition("ImpactReference", &changed));
        changed["native_change_observation"] = json!("detected-native-change");
        assert!(valid_definition("ImpactReference", &changed));
    }
    let mut metadata = baseline;
    metadata["summary"]["findings"] = json!(1);
    metadata["finding_count"] = json!(1);
    metadata["matched_findings"] = json!(1);
    metadata["unreferenced_finding_ids"] = json!([id(9)]);
    assert!(!valid_definition("ImpactReference", &metadata));
    metadata["native_change_observation"] = json!("detected-native-change");
    assert!(valid_definition("ImpactReference", &metadata));
}

/// Exact native UUID/token patterns preserve native spelling without review-token substitution.
#[test]
fn native_identity_patterns_preserve_case_and_native_unicode_subjects() {
    let pin = source_pin();
    let inert: ImpactSourcePin =
        serde_json::from_value(pin.clone()).expect("plain native spelling");
    assert_eq!(inert.native_root_uuid.as_deref(), Some("ABCDEFAB-ABCD-4ABC-8ABC-ABCDEFABCDEF"));
    for bad in [
        "00000000-0000-0000-0000-000000000000",
        "00000000-0000-1000-8000-000000000001",
        "00000000-0000-4000-7000-000000000001",
    ] {
        let mut invalid = pin.clone();
        invalid["native_root_uuid"] = json!(bad);
        assert!(!valid_definition("ImpactSourcePin", &invalid));
    }
    let finding = json!({"finding_id":id(6),"priority":"review-required","reason_code":"mapping_subject_changed","required_action":"reapprove-mapping-rationale","native_subject_id":"Épreuve_1.2-ß","change_class":"content-changed","old_dependency_binding":"captured-native-map-id-and-declared-old-pin","new_relation":"explicit-declared-review-lineage"});
    assert!(valid_definition("FindingReference", &finding));
    let inert: FindingReference =
        serde_json::from_value(finding.clone()).expect("actual native enum types");
    assert_eq!(serde_json::to_value(inert).expect("exact native subject"), finding);
    let mut bad = finding;
    bad["native_subject_id"] = json!("1-leading-digit");
    assert!(!valid_definition("FindingReference", &bad));
}

/// Private document version, href, path and free prose never enter the public whitelist.
#[test]
fn private_metadata_and_open_vocabulary_are_refused() {
    for private in ["document_version", "href", "path", "message", "rationale", "title"] {
        let mut full = document();
        full["impact"]["old"][private] = json!("private-name");
        assert!(!valid(&full));
        assert!(serde_json::from_value::<SupersessionDocument>(full).is_err());
        let mut pin = source_pin();
        pin[private] = json!("private-name");
        assert!(!valid_definition("ImpactSourcePin", &pin));
        assert!(serde_json::from_value::<ImpactSourcePin>(pin).is_err());
    }
    let mut full = document();
    full["semantics"] = json!("automatic-vote-transfer");
    assert!(!valid(&full));
    assert!(serde_json::from_value::<SupersessionDocument>(full).is_err());
}

/// Loaded labels and schema success intentionally cannot prove cross-count/native correspondence.
#[test]
fn recorded_labels_and_shape_validation_remain_non_authorizing_data() {
    let mut record = document();
    record["counts"]["old_items"] = json!(9);
    record["impact"]["summary"]["undispositioned"] = json!(7);
    assert!(valid(&record));
    let inert: SupersessionDocument =
        serde_json::from_value(record).expect("shape alone cannot correlate native data");
    assert_eq!(inert.counts.old_items, 9);
    assert_eq!(inert.impact.summary.undispositioned, 7);
    // There is no captured/current/union factory or publication method on this inert type.
}

//! Genuine owned native/domain fixtures for the private captured byte consumer.
//!
//! These source-proposed controls call the real maintained Mapping builder and
//! path-based applicability engine only to produce fixture originals. The helper
//! consumes their exact saved bytes; no disclosure/capture proof is fabricated.

use std::path::Path;

use serde_json::{Value, json};

use super::{
    MappingInput, NativeInput, PreparedApplicability, ResolvedInput, mapping_preflight, prepare,
};
use crate::applicability::model::{GapClassification, ReportFilters};
use crate::hashing::sha256_hex;
use crate::mapping::inventory;
use crate::mapping::manifest::{ResourceManifest, ResourceType};
use crate::workspace::contract::Error;
use crate::workspace::preparation::test_support::Recorder;
use crate::workspace::preparation::{
    Interruption, NoopControl, Stage, WorkControl, WorkError, WorkResult,
};

/// Keep all actual fixture originals and their containing directory alive.
struct Fixture {
    /// Owned private fixture directory; never an MCP project or proof constructor.
    directory: tempfile::TempDir,
    /// Actual saved closed applicability source bytes.
    manifest: Vec<u8>,
    /// Actual original selected Catalog or Profile bytes.
    framework: Vec<u8>,
    /// Actual declared original Profile Catalog companion, if needed.
    companion: Option<Vec<u8>>,
    /// Actual original source Catalog bytes.
    source: Vec<u8>,
    /// Actual maintained builder-produced native Mapping bytes.
    mapping: Vec<u8>,
    /// Actual maintained engine-produced complete persisted report bytes.
    report: Vec<u8>,
    /// Intrinsic selected framework type from the actual fixture source.
    framework_type: ResourceType,
}

impl Fixture {
    /// Build real persisted fixtures before any captured helper call.
    fn new(profile_framework: bool, self_mapping: bool) -> Self {
        let directory = tempfile::tempdir().expect("owned fixture directory");
        let root = directory.path();
        let framework_catalog =
            catalog("22222222-2222-4222-8222-222222222222", &["c1", "c2", "c3"]);
        let source_catalog =
            catalog("11111111-1111-4111-8111-111111111111", &["policy-1", "policy-2"]);
        write_json(&root.join("framework.json"), &framework_catalog);
        write_json(&root.join("policy.json"), &source_catalog);
        if profile_framework {
            write_json(&root.join("resolved-catalog.json"), &framework_catalog);
            write_json(&root.join("framework.json"), &profile());
        }
        let framework_type =
            if profile_framework { ResourceType::Profile } else { ResourceType::Catalog };
        let mut framework_resource = resource(framework_type, "framework.json");
        if profile_framework {
            framework_resource.resolved_catalog = Some("resolved-catalog.json".into());
            framework_resource.resolved_catalog_attestation = Some(true);
            framework_resource.expected_resolved_catalog_sha256 = Some(sha256_hex(
                &std::fs::read(root.join("resolved-catalog.json")).expect("companion original"),
            ));
        }
        let framework_loaded = inventory::load(root, "fixture framework", &framework_resource)
            .expect("actual native inventory");
        framework_resource.expected_sha256 = Some(framework_loaded.evidence.raw_sha256.clone());
        framework_resource.inventory = Some(framework_loaded.snapshot());
        let mapping_manifest = mapping_source(&framework_resource, self_mapping);
        write_json(&root.join("mapping-manifest.json"), &mapping_manifest);
        let built = crate::mapping::prepare(&root.join("mapping-manifest.json"), None, false)
            .expect("real maintained Mapping builder");
        std::fs::write(root.join("mapping.json"), built.artifact_json.as_bytes())
            .expect("save actual native Mapping");
        let manifest = applicability_source(&framework_resource);
        write_json(&root.join("applicability.json"), &manifest);
        let actual = crate::applicability::prepare_analysis(
            &root.join("applicability.json"),
            ReportFilters::default(),
        )
        .expect("real maintained applicability engine");
        write_json(
            &root.join("report.json"),
            &serde_json::to_value(&actual.report).expect("full actual report"),
        );
        Self {
            manifest: std::fs::read(root.join("applicability.json"))
                .expect("actual manifest original"),
            framework: std::fs::read(root.join("framework.json"))
                .expect("actual framework original"),
            companion: profile_framework.then(|| {
                std::fs::read(root.join("resolved-catalog.json"))
                    .expect("actual companion original")
            }),
            source: std::fs::read(root.join("policy.json")).expect("actual policy original"),
            mapping: std::fs::read(root.join("mapping.json")).expect("actual Mapping original"),
            report: std::fs::read(root.join("report.json")).expect("actual report original"),
            framework_type,
            directory,
        }
    }

    /// Borrow actual intrinsic native originals; explicit companion remains separate.
    fn natives(&self) -> Vec<NativeInput<'_>> {
        let mut inputs = vec![
            NativeInput {
                key: "framework",
                resource_type: self.framework_type,
                bytes: &self.framework,
                resolved_catalog: self
                    .companion
                    .as_deref()
                    .map(|bytes| ResolvedInput { key: "resolved", bytes }),
            },
            NativeInput {
                key: "policy",
                resource_type: ResourceType::Catalog,
                bytes: &self.source,
                resolved_catalog: None,
            },
        ];
        if let Some(bytes) = self.companion.as_deref() {
            inputs.push(NativeInput {
                key: "resolved",
                resource_type: ResourceType::Catalog,
                bytes,
                resolved_catalog: None,
            });
        }
        inputs
    }

    /// Consume the actual raw saved bytes with the caller's real ledger/control.
    fn prepared(
        &self,
        charge: &mut dyn FnMut(usize) -> WorkResult<()>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<PreparedApplicability<'_>> {
        prepare(
            &self.manifest,
            "framework",
            &self.natives(),
            &[MappingInput { key: "captured-map", bytes: &self.mapping }],
            charge,
            control,
        )
    }
}

/// Serialize and save one actual private original; fixture generation only.
fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("fixture JSON"))
        .expect("save owned fixture original");
}

/// Construct genuine official-schema Catalog data used by both maintained engines.
fn catalog(uuid: &str, ids: &[&str]) -> Value {
    json!({"catalog": {"uuid": uuid, "metadata": {
        "title": "Synthetic native controls", "last-modified": "2026-08-25T08:00:00Z",
        "version": "1.0.0", "oscal-version": "1.2.3"}, "groups": [{"id": "group-1",
        "title": "Synthetic group", "controls": ids.iter().map(|id| json!({"id": id,
            "title": format!("Control {id}"), "parts": [{"id": format!("{id}_smt"),
                "name": "statement", "prose": format!("Actual synthetic statement {id}.")}]}))
            .collect::<Vec<_>>()}]}})
}

/// Construct a genuine Profile with an explicitly supplied actual resolved Catalog.
fn profile() -> Value {
    json!({"profile": {"uuid": "33333333-3333-4333-8333-333333333333", "metadata": {
        "title": "Synthetic profile", "last-modified": "2026-08-25T08:00:00Z",
        "version": "1.0.0", "oscal-version": "1.2.3"}, "imports": [{"href": "resolved-catalog.json", "include-all": {}}]}})
}

/// Build a real resource descriptor; fixture schema/inventory admission adds pins.
fn resource(resource_type: ResourceType, name: &str) -> ResourceManifest {
    ResourceManifest {
        resource_type,
        artifact: name.into(),
        href: name.to_owned(),
        resolved_catalog: None,
        resolved_catalog_attestation: None,
        expected_sha256: None,
        expected_resolved_catalog_sha256: None,
        inventory: None,
    }
}

/// Author reviewed many-to-many fixture input for the actual maintained Mapping builder.
fn mapping_source(framework: &ResourceManifest, self_mapping: bool) -> Value {
    let source = if self_mapping {
        framework.clone()
    } else {
        resource(ResourceType::Catalog, "policy.json")
    };
    let ids = if self_mapping { ["c1", "c2"] } else { ["policy-1", "policy-2"] };
    json!({"schema_version": "forge.mapping-manifest/1", "collection": {
        "key": "native-collection-key", "title": "Synthetic reviewed collection", "version": "1.0.0",
        "last_modified": "2026-08-25T08:00:00Z"}, "reviewers": [{"key": "mapper", "type": "person", "name": "Mapping Reviewer"}],
        "provenance": {"method": "human", "matching_rationale": "semantic", "status": "complete",
            "mapping_description": "Synthetic current participation.", "reviewer_keys": ["mapper"], "reviewed_at": "2026-08-25T08:00:00Z"},
        "mapping": {"key": "native-mapping-key", "scope": "control-only", "source": source, "target": framework,
            "maps": [{"key": "pair", "relationship": "intersects-with", "sources": ids.iter().map(|id|
                json!({"type": "control", "id_ref": id})).collect::<Vec<_>>(),
                "targets": [{"type": "control", "id_ref": "c1"}, {"type": "control", "id_ref": "c2"}],
                "reviewer_key": "mapper", "reviewed_at": "2026-08-25T08:00:00Z", "rationale": "Reviewed synthetic native pair."}]}})
}

/// Author actual closed scope decisions with every default-filter classification preserved.
fn applicability_source(framework: &ResourceManifest) -> Value {
    json!({"schema_version": "forge.applicability/1", "framework": framework,
        "reviewers": [{"key": "scope-reviewer", "type": "person", "name": "Scope Reviewer"}],
        "decisions": [{"control_id": "c1", "state": "applicable", "reviewer_key": "scope-reviewer", "reviewed_at": "2026-08-25T09:00:00Z"},
            {"control_id": "c2", "state": "deferred", "reviewer_key": "scope-reviewer", "reviewed_at": "2026-08-25T09:00:00Z",
                "rationale": "Pending an actual synthetic scope decision.", "revisit_date": "2026-10-01"}],
        "mapping_collections": ["mapping.json"]})
}

/// Decode only a fixture original for a targeted mutation; this is not admission authority.
fn value(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("saved fixture JSON")
}

/// Encode a targeted fixture mutation without introducing duplicate or stale raw assumptions.
fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("mutated fixture JSON")
}

/// Alter one actual FORGE property while retaining its complete native owner object.
fn set_prop(object: &mut Value, name: &str, replacement: &str) {
    let props = object["props"].as_array_mut().expect("actual native props");
    let property = props.iter_mut().find(|prop| prop["name"] == name).expect("actual named prop");
    property["value"] = json!(replacement);
}

/// Assert genuine fixture admission before any negative vector can earn its intended seam.
fn canary(fixture: &Fixture) {
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("genuine captured canary");
    prepared
        .validate_report(&fixture.report, &mut |_| Ok(()), &mut NoopControl)
        .expect("complete actual report equality");
    assert!(fixture.directory.path().join("mapping.json").is_file());
}

/// Current Catalog facts agree completely with the genuine maintained report.
#[test]
fn genuine_catalog_report_and_cartesian_pairs_preserve_native_counts() {
    let fixture = Fixture::new(false, false);
    let mut calls = Vec::new();
    let prepared = fixture
        .prepared(
            &mut |count| {
                calls.push(count);
                Ok(())
            },
            &mut NoopControl,
        )
        .expect("actual captured facts");
    assert_eq!(prepared.counts().total, 3);
    assert_eq!(prepared.counts().applicable_mapped, 1);
    assert_eq!(prepared.counts().deferred, 1);
    assert_eq!(prepared.counts().under_review, 1);
    assert_eq!(prepared.manifest_sha256(), sha256_hex(&fixture.manifest));
    assert_eq!(prepared.relations("c1").count(), 2);
    assert_eq!(prepared.relations("policy-1").count(), 2);
    assert_eq!(prepared.facts["c1"].positive, 1);
    assert_eq!(prepared.facts["c2"].positive, 1);
    assert!(calls.iter().filter(|&&count| count == 4).count() >= 2);
    assert_eq!(
        prepared.control_rows().next().expect("first native control").classification(),
        GapClassification::ApplicableMapped
    );
    prepared
        .validate_report(&fixture.report, &mut |_| Ok(()), &mut NoopControl)
        .expect("full maintained equality");
}

/// Actual Profile plus exact approved companion admits the real framework inventory/report.
#[test]
fn genuine_profile_companion_and_mapping_currentness_are_consumed() {
    let fixture = Fixture::new(true, false);
    canary(&fixture);
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("actual Profile facts");
    assert_eq!(prepared.control_rows().count(), 3);
    assert_eq!(
        prepared.relations("c1").next().expect("real native pair").target_key(),
        "framework"
    );
}

/// A declared missing/stale Profile companion cannot borrow another inventory or hash.
#[test]
fn profile_companion_missing_stale_and_wrong_kind_are_refused() {
    let fixture = Fixture::new(true, false);
    canary(&fixture);
    let mut inputs = fixture.natives();
    inputs[0].resolved_catalog = None;
    assert!(
        prepare(
            &fixture.manifest,
            "framework",
            &inputs,
            &[MappingInput { key: "captured-map", bytes: &fixture.mapping }],
            &mut |_| Ok(()),
            &mut NoopControl
        )
        .is_err()
    );
    let mut stale = value(fixture.companion.as_deref().expect("actual companion"));
    stale["catalog"]["metadata"]["title"] = json!("Changed companion original");
    let stale = bytes(&stale);
    for replacement in [&stale[..], &fixture.framework[..]] {
        let mut inputs = fixture.natives();
        inputs[0].resolved_catalog = Some(ResolvedInput { key: "resolved", bytes: replacement });
        assert!(
            prepare(
                &fixture.manifest,
                "framework",
                &inputs,
                &[MappingInput { key: "captured-map", bytes: &fixture.mapping }],
                &mut |_| Ok(()),
                &mut NoopControl
            )
            .is_err()
        );
    }
}

/// An unused actual unresolved Profile header creates neither inventory nor false refusal.
#[test]
fn unrelated_unresolved_profile_is_not_a_framework_or_mapping_source() {
    let fixture = Fixture::new(false, false);
    let profile = bytes(&profile());
    let mut inputs = fixture.natives();
    inputs.push(NativeInput {
        key: "unused-profile",
        resource_type: ResourceType::Profile,
        bytes: &profile,
        resolved_catalog: None,
    });
    let prepared = prepare(
        &fixture.manifest,
        "framework",
        &inputs,
        &[MappingInput { key: "captured-map", bytes: &fixture.mapping }],
        &mut |_| Ok(()),
        &mut NoopControl,
    )
    .expect("unused Profile need not resolve");
    prepared
        .validate_report(&fixture.report, &mut |_| Ok(()), &mut NoopControl)
        .expect("unchanged actual current report");
}

/// Exact actual native source tuple is required independently of asserted Mapping href.
#[test]
fn source_and_target_resource_identity_properties_cannot_be_forged() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    for side in ["source-resource", "target-resource"] {
        for (name, replacement) in [
            ("raw-sha256", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            ("root-uuid", "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"),
            ("document-version", "2.0.0"),
        ] {
            let mut changed = value(&fixture.mapping);
            set_prop(&mut changed["mapping-collection"]["mappings"][0][side], name, replacement);
            let changed = bytes(&changed);
            assert!(
                prepare(
                    &fixture.manifest,
                    "framework",
                    &fixture.natives(),
                    &[MappingInput { key: "captured-map", bytes: &changed }],
                    &mut |_| Ok(()),
                    &mut NoopControl
                )
                .is_err()
            );
        }
    }
}

/// Every source and target endpoint needs its actual current canonical native fingerprint.
#[test]
fn complete_source_and_target_fingerprints_are_checked_after_valid_schema() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    for side in ["sources", "targets"] {
        let mut changed = value(&fixture.mapping);
        set_prop(
            &mut changed["mapping-collection"]["mappings"][0]["maps"][0][side][1],
            "subject-sha256",
            &"a".repeat(64),
        );
        assert!(
            crate::applicability::decode_mapping_value("fixture Mapping", changed.clone()).is_ok()
        );
        let changed = bytes(&changed);
        assert!(
            prepare(
                &fixture.manifest,
                "framework",
                &fixture.natives(),
                &[MappingInput { key: "captured-map", bytes: &changed }],
                &mut |_| Ok(()),
                &mut NoopControl
            )
            .is_err()
        );
    }
}

/// Complete native identities cannot select an ambiguous second captured resource.
#[test]
fn duplicate_keys_and_ambiguous_complete_source_tuples_are_refused() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    for key in ["policy", "duplicate-policy"] {
        let mut inputs = fixture.natives();
        inputs.push(NativeInput {
            key,
            resource_type: ResourceType::Catalog,
            bytes: &fixture.source,
            resolved_catalog: None,
        });
        assert!(
            prepare(
                &fixture.manifest,
                "framework",
                &inputs,
                &[MappingInput { key: "captured-map", bytes: &fixture.mapping }],
                &mut |_| Ok(()),
                &mut NoopControl
            )
            .is_err()
        );
    }
}

/// Original map UUID spelling and supplied MCP key survive typed native normalization.
#[test]
fn self_mapping_retains_three_selected_pairs_and_original_uuid_case() {
    let fixture = Fixture::new(false, true);
    canary(&fixture);
    let mut changed = value(&fixture.mapping);
    let original = changed["mapping-collection"]["mappings"][0]["maps"][0]["uuid"]
        .as_str()
        .expect("map UUID")
        .to_uppercase();
    changed["mapping-collection"]["mappings"][0]["maps"][0]["uuid"] = json!(original);
    let changed = bytes(&changed);
    let prepared = prepare(
        &fixture.manifest,
        "framework",
        &fixture.natives(),
        &[MappingInput { key: "captured-map", bytes: &changed }],
        &mut |_| Ok(()),
        &mut NoopControl,
    )
    .expect("actual uppercase native UUID");
    let rows: Vec<_> = prepared.relations("c1").collect();
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter().all(|row| row.collection_key() == "captured-map" && row.map_id() == original)
    );
    assert!(
        rows.iter().all(|row| row.source_key() == "framework" && row.target_key() == "framework")
    );
    assert_eq!(prepared.facts["c1"].positive, 1);
}

/// Full report differences remain ordinary failures even when the stored parser admits them.
#[test]
fn full_report_resources_reviewers_rows_queue_and_filters_are_not_reduced() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("actual current facts");
    for (pointer, replacement) in [
        ("/framework/href", json!("changed-label.json")),
        ("/framework/raw_sha256", json!("a".repeat(64))),
        ("/mapping_collections/0/reviewed_at", json!("2026-08-26T08:00:00Z")),
        ("/mapping_collections/0/reviewers/0/name", json!("Other reviewer")),
        (
            "/mapping_collections/0/source_resources/0/root_uuid",
            json!("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"),
        ),
        ("/reviewers/0/name", json!("Other scope reviewer")),
        ("/controls/0/groups", json!(["other-group"])),
        ("/controls/0/positive_mapping_count", json!(2)),
        ("/controls/0/policy_sources", json!(["changed-policy.json"])),
        ("/controls/1/revisit_date", json!("2027-01-01")),
        ("/review_queue/0/owner", json!("other-owner")),
        ("/review_queue/0/reason_code", json!("scope-decision-required")),
    ] {
        let mut changed = value(&fixture.report);
        *changed.pointer_mut(pointer).expect("existing complete report field") = replacement;
        let changed = bytes(&changed);
        assert!(
            crate::applicability::parse_stored_report(&changed).is_ok(),
            "historical parser admits {pointer}"
        );
        assert!(prepared.validate_report(&changed, &mut |_| Ok(()), &mut NoopControl).is_err());
    }
    let mut changed = value(&fixture.report);
    changed["filters"]["group"] = json!("group-1");
    let changed = bytes(&changed);
    assert!(crate::applicability::parse_stored_report(&changed).is_ok());
    assert!(prepared.validate_report(&changed, &mut |_| Ok(()), &mut NoopControl).is_err());
}

/// Raw strict admission rejects duplicate keys and unknown persisted report fields.
#[test]
fn strict_original_and_complete_stored_report_closure_are_enforced() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    let source = String::from_utf8(fixture.source.clone()).expect("actual source UTF-8");
    let duplicate = source
        .replacen(
            "\"catalog\": {",
            "\"catalog\": {\"uuid\":\"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa\",",
            1,
        )
        .into_bytes();
    let mut inputs = fixture.natives();
    inputs[1].bytes = &duplicate;
    assert!(
        prepare(
            &fixture.manifest,
            "framework",
            &inputs,
            &[MappingInput { key: "captured-map", bytes: &fixture.mapping }],
            &mut |_| Ok(()),
            &mut NoopControl
        )
        .is_err()
    );
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("actual current facts");
    let mut extra = value(&fixture.report);
    extra["controls"][0]["unknown"] = json!("untrusted extra");
    assert!(prepared.validate_report(&bytes(&extra), &mut |_| Ok(()), &mut NoopControl).is_err());
}

/// Both complete Cartesian registries reserve the checked product before any retained map IDs.
#[test]
fn second_cartesian_charge_failure_propagates_the_actual_ledger_error() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    let native = value(&fixture.mapping);
    let mut pairs = 0;
    let result = mapping_preflight(
        &native,
        &mut |count| {
            if count == 4 {
                pairs += 1;
                if pairs == 2 {
                    return Err(WorkError::Failed(Error::new(
                        "ledger-full",
                        "Bound reached.",
                        false,
                    )));
                }
            }
            Ok(())
        },
        &mut NoopControl,
    );
    assert_eq!(pairs, 2);
    assert!(matches!(result, Err(WorkError::Failed(error)) if error.code == "ledger-full"));
}

/// Inventory pre-growth callback errors cannot be minimized into ordinary invalid candidates.
#[test]
fn maintained_inventory_callback_preserves_exact_shared_work_failure() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    let input = fixture.natives()[0];
    let descriptor = super::bare_resource(&input);
    let result = super::load_native(
        &input,
        &descriptor,
        &mut |_| Err(WorkError::Failed(Error::new("shared-ledger", "Bound reached.", false))),
        &mut NoopControl,
    );
    assert!(matches!(result, Err(WorkError::Failed(error)) if error.code == "shared-ledger"));
}

/// Actual cooperative interruption remains sticky rather than becoming stale/unavailable.
#[test]
fn prepare_and_full_report_comparison_preserve_typed_interruption() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    let mut recorder = Recorder::at(Stage::PrepareDomain, 2);
    assert!(matches!(
        fixture.prepared(&mut |_| Ok(()), &mut recorder),
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    ));
    assert_eq!(recorder.interruption(), Some(Interruption::CancelRequested));
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("actual current facts");
    let mut recorder = Recorder::at(Stage::PrepareDomain, 3);
    assert!(matches!(
        prepared.validate_report(&fixture.report, &mut |_| Ok(()), &mut recorder),
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    ));
}

/// Header sorting and policy-source temporary registries consume the same ledger before growth.
#[test]
fn complete_report_comparison_propagates_late_ledger_refusal() {
    let fixture = Fixture::new(false, false);
    canary(&fixture);
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("actual current facts");
    let report = crate::applicability::parse_stored_report(&fixture.report)
        .expect("complete actual stored report");
    let mut calls = Vec::new();
    let result = prepared.compare_report_header(
        &report,
        &mut |count| {
            calls.push(count);
            Err(WorkError::Failed(Error::new("late-header-ledger", "Bound reached.", false)))
        },
        &mut NoopControl,
    );
    assert_eq!(calls, [1]);
    assert!(matches!(result, Err(WorkError::Failed(error)) if error.code == "late-header-ledger"));
    let mut calls = Vec::new();
    let result = prepared.policy_sources(
        "c1",
        &mut |count| {
            calls.push(count);
            Err(WorkError::Failed(Error::new("late-source-ledger", "Bound reached.", false)))
        },
        &mut NoopControl,
    );
    assert_eq!(calls, [1]);
    assert!(matches!(result, Err(WorkError::Failed(error)) if error.code == "late-source-ledger"));
}
/// A genuine Profile source and target share one current effective inventory and complete report.
#[test]
fn genuine_profile_source_self_mapping_matches_full_report_and_native_counts() {
    let fixture = Fixture::new(true, true);
    canary(&fixture);
    let prepared =
        fixture.prepared(&mut |_| Ok(()), &mut NoopControl).expect("actual Profile source facts");
    prepared
        .validate_report(&fixture.report, &mut |_| Ok(()), &mut NoopControl)
        .expect("full actual Profile-source report equality");
    let companion_hash =
        sha256_hex(fixture.companion.as_deref().expect("actual resolved Catalog original"));
    assert_eq!(
        prepared.mappings[0].evidence.source_resources[0].resolved_catalog_sha256.as_deref(),
        Some(companion_hash.as_str())
    );
    assert_eq!(prepared.mappings[0].sources[0], prepared.framework);
    let rows: Vec<_> = prepared.relations("c1").collect();
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter().all(|row| row.source_key() == "framework" && row.target_key() == "framework")
    );
    assert!(rows.iter().all(|row| row.collection_key() == "captured-map"));
    assert_eq!(prepared.facts["c1"].positive, 1);
    assert_eq!(prepared.facts["c2"].positive, 1);
    assert_eq!(prepared.counts().applicable_mapped, 1);
    assert_eq!(prepared.counts().deferred, 1);
    assert_eq!(prepared.counts().under_review, 1);
}

/// Schema-admitted Profile-source companion hashes and fingerprints must match actual originals.
#[test]
fn profile_source_companion_tuple_and_current_fingerprint_cannot_be_forged() {
    let fixture = Fixture::new(true, true);
    canary(&fixture);
    for replacement in ["a".repeat(64), "b".repeat(64)] {
        let mut changed = value(&fixture.mapping);
        set_prop(
            &mut changed["mapping-collection"]["mappings"][0]["source-resource"],
            "resolved-catalog-sha256",
            &replacement,
        );
        assert!(
            crate::applicability::decode_mapping_value("fixture Profile Mapping", changed.clone())
                .is_ok()
        );
        let changed = bytes(&changed);
        assert!(
            prepare(
                &fixture.manifest,
                "framework",
                &fixture.natives(),
                &[MappingInput { key: "captured-map", bytes: &changed }],
                &mut |_| Ok(()),
                &mut NoopControl
            )
            .is_err()
        );
    }
    let mut changed = value(&fixture.mapping);
    set_prop(
        &mut changed["mapping-collection"]["mappings"][0]["maps"][0]["sources"][1],
        "subject-sha256",
        &"a".repeat(64),
    );
    assert!(
        crate::applicability::decode_mapping_value("fixture Profile Mapping", changed.clone())
            .is_ok()
    );
    let changed = bytes(&changed);
    assert!(
        prepare(
            &fixture.manifest,
            "framework",
            &fixture.natives(),
            &[MappingInput { key: "captured-map", bytes: &changed }],
            &mut |_| Ok(()),
            &mut NoopControl
        )
        .is_err()
    );
}

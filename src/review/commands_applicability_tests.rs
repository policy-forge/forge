//! Genuine synthetic applicability command controls over actual native originals.
//! The cfg-only fixture keeps its real root alive and uses maintained Mapping,
//! applicability and lifecycle producers. These assertions do not authenticate
//! reviewers, grant native approval or establish measured platform acceptance.

use super::*;
use crate::evidence_capture::CaptureRole;
use crate::review::adapters::applicability as applicability_adapter;
use crate::review::applicability_capture::{self, tests::Fixture};
use crate::review::capture::{Pool, ReviewCapture};
use crate::review::wire::{ItemState, RecordedCurrentness, SourceModel};
use crate::workspace::preparation::NoopControl;
use serde_json::json;

/// Asserted queue revision chosen explicitly for these synthetic controls.
const QUEUE_ID: &str = "55555555-5555-4555-8555-555555555555";
/// Three distinct asserted response identities; none is a reviewer credential.
const RESPONSE_IDS: [&str; 3] = [
    "66666666-6666-4666-8666-666666666666",
    "77777777-7777-4777-8777-777777777777",
    "88888888-8888-4888-8888-888888888888",
];
/// Explicit asserted policy-evaluation time, independent of the test machine clock.
const AS_OF: &str = "2026-10-04T12:00:00Z";

/// Persist fixture declarations only; this does not create captured authority.
fn write_value(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// Read a saved test fixture for a deliberate mutation, outside production capture.
fn read_value(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// Declare two required distinct seats and three explicitly assigned asserted keys.
/// Native pins, context, subject hashes and proof cannot enter this request.
fn write_policy(fixture: &Fixture, selected: &[&str]) {
    let items = selected
        .iter()
        .map(|id| {
            json!({
                "key": format!("item-{id}"), "subject_id": id, "policy_key": "policy",
                "author_keys": ["source-author"],
                "assignments": [
                    {"reviewer_key":"a-reviewer","role_key":"review"},
                    {"reviewer_key":"b-reviewer","role_key":"review"},
                    {"reviewer_key":"c-reviewer","role_key":"review"}
                ], "due_at": null
            })
        })
        .collect::<Vec<_>>();
    write_value(
        &fixture.root().join("review-policy.json"),
        &json!({
            "schema_version":"forge.review-init/1", "roles":[{"key":"review"}],
            "reviewers":[
                {"key":"a-reviewer","role_keys":["review"]},
                {"key":"b-reviewer","role_keys":["review"]},
                {"key":"c-reviewer","role_keys":["review"]}
            ], "policies":[{
                "key":"policy", "seats":[{"role_key":"review","count":2}],
                "substitutions":[], "abstention_rule":"nonapproving",
                "empty_abstention_reasons":[], "author_separation":"declared-keys"
            }], "items":items
        }),
    );
}

/// Call the actual initialized-domain workflow with every path and declaration explicit.
fn initialize(fixture: &Fixture, output: &Path) -> Result<(), CommandError> {
    init(
        &InitOptions {
            project_root: fixture.root(),
            sources: Path::new("locator.json"),
            policy: Path::new("review-policy.json"),
            queue_id: QUEUE_ID,
            created_at: "2026-10-04T10:00:00Z",
            output,
        },
        &mut NoopControl,
    )
}

/// Keep the genuine fixture owner alive after actual native preparation/publication.
fn initialized(profile: bool, selected: &[&str]) -> Fixture {
    let fixture = Fixture::new(profile);
    write_policy(&fixture, selected);
    initialize(&fixture, Path::new("queue.json")).unwrap();
    fixture
}

/// Publish one immutable actual response using a captured private rationale original.
fn response(fixture: &Fixture, ordinal: usize, disposition: Disposition) -> PathBuf {
    let reviewer = ["a-reviewer", "b-reviewer", "c-reviewer"][ordinal];
    let rationale = PathBuf::from(format!("rationale-{ordinal}.txt"));
    std::fs::write(fixture.root().join(&rationale), b"PRIVATE-REVIEW-RATIONALE").unwrap();
    let output = PathBuf::from(format!("response-{ordinal}.json"));
    respond(
        &RespondOptions {
            project_root: fixture.root(),
            queue: Path::new("queue.json"),
            rationale_file: &rationale,
            item_key: "item-c1",
            reviewer_key: reviewer,
            reviewer_role: "review",
            disposition,
            responded_at: "2026-10-04T11:00:00Z",
            response_id: RESPONSE_IDS[ordinal],
            abstention_reason: None,
            supersedes: None,
            output: &output,
        },
        &mut NoopControl,
    )
    .unwrap();
    output
}

/// Borrow the complete actual ordered response occurrence vector without preprocessing.
fn options<'a>(fixture: &'a Fixture, responses: &'a [PathBuf]) -> CurrentOptions<'a> {
    CurrentOptions {
        project_root: fixture.root(),
        sources: Path::new("locator.json"),
        queue: Path::new("queue.json"),
        responses,
        as_of: AS_OF,
    }
}

/// Exercise the real readonly writer path, retaining no detached currentness token.
fn status_bytes(fixture: &Fixture, responses: &[PathBuf]) -> Vec<u8> {
    let mut bytes = Vec::new();
    status(&options(fixture, responses), &mut bytes, &mut NoopControl).unwrap();
    bytes
}

/// Validate complete current output conservation through the maintained closed decoder.
fn assert_status(bytes: &[u8], state: ItemState, files: u32, unique: u32, duplicates: u32) {
    let closed =
        decode::decode_dispositions(bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let document = closed.document();
    assert_eq!(document.currentness, RecordedCurrentness::RecordedCurrent);
    assert!(document.closure_generation.is_some());
    assert_eq!(document.identity_disclaimer, IDENTITY_DISCLAIMER);
    assert_eq!(document.as_of, AS_OF);
    assert_eq!(document.counts.items, 1);
    assert_eq!(document.items.len(), 1);
    assert_eq!(document.items[0].state, state);
    assert_eq!(document.counts.response_files, files);
    assert_eq!(document.counts.unique_responses, unique);
    assert_eq!(document.counts.exact_duplicates, duplicates);
    assert_eq!(document.responses.len(), usize::try_from(unique).unwrap());
}

/// Preserve every pre-existing native/locator original in the fixture's complete source roster.
fn native_originals(fixture: &Fixture) -> Vec<(&'static str, Vec<u8>)> {
    let mut names = vec![
        "framework.json",
        "policy.json",
        "mapping-manifest.json",
        "mapping.json",
        "applicability.json",
        "report.json",
        "lifecycle.json",
        "locator.json",
    ];
    if fixture.root().join("resolved-catalog.json").exists() {
        names.push("resolved-catalog.json");
    }
    names
        .into_iter()
        .map(|name| (name, std::fs::read(fixture.root().join(name)).unwrap()))
        .collect()
}

/// Compare actual saved bytes after review output without confusing output creation with source writes.
fn assert_native_unchanged(fixture: &Fixture, before: &[(&str, Vec<u8>)]) {
    for (name, raw) in before {
        assert_eq!(std::fs::read(fixture.root().join(name)).unwrap().as_slice(), raw.as_slice());
    }
}

/// Assert that native private prose/paths and response rationale do not enter minimized exchange output.
fn assert_private_absent(fixture: &Fixture, bytes: &[u8]) {
    let text = std::str::from_utf8(bytes).unwrap();
    for marker in [
        "Scope Reviewer",
        "Synthetic native controls",
        "Mapping Reviewer",
        "Pending an actual synthetic scope decision.",
        "PRIVATE-REVIEW-RATIONALE",
    ] {
        assert!(!text.contains(marker), "private fixture marker escaped projection");
    }
    assert!(!text.contains(fixture.root().to_str().unwrap()));
}

/// Check the recorded output digest from exact actual original-byte occurrences.
/// This reconstructs a public output binding, never a native/currentness constructor.
fn assert_generation(fixture: &Fixture, responses: &[PathBuf], bytes: &[u8]) {
    let closed =
        decode::decode_dispositions(bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let queue = std::fs::read(fixture.root().join("queue.json")).unwrap();
    let queue_hash = crate::hashing::sha256_hex(&queue);
    let response_rows = responses
        .iter()
        .map(|path| {
            let raw = std::fs::read(fixture.root().join(path)).unwrap();
            format!(
                r#"{{"raw_sha256":{},"byte_length":{}}}"#,
                serde_json::to_string(&crate::hashing::sha256_hex(&raw)).unwrap(),
                raw.len()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let encoded = format!(
        r#"{{"adapter_version":"forge.applicability-review/1","queue_original":{{"raw_sha256":{},"byte_length":{}}},"response_originals":[{}],"source_pins":{},"as_of":{}}}"#,
        serde_json::to_string(&queue_hash).unwrap(),
        queue.len(),
        response_rows,
        serde_json::to_string(&closed.document().source_pins).unwrap(),
        serde_json::to_string(AS_OF).unwrap()
    );
    let mut hashed = b"forge.review-current-closure/1\0".to_vec();
    hashed.extend_from_slice(encoded.as_bytes());
    let expected = crate::hashing::sha256_hex(&hashed);
    assert_eq!(closed.document().closure_generation.as_deref(), Some(expected.as_str()));
}

/// Run complete real Catalog/Profile review commands and inspect distinct-seat/native pin bindings.
fn actual_quorum(profile: bool, expected_sources: usize) {
    let fixture = initialized(profile, &["c1"]);
    let originals = native_originals(&fixture);
    let queue_raw = std::fs::read(fixture.root().join("queue.json")).unwrap();
    let queue =
        decode::decode_queue(&queue_raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(queue.document().items[0].domain, Domain::ApplicabilityDecision);
    assert_eq!(queue.document().items[0].adapter_version, "forge.applicability-review/1");
    assert_eq!(queue.document().items[0].subject_id, "c1");
    assert_eq!(queue.document().source_pins.len(), expected_sources);
    assert_eq!(queue.document().items[0].source_keys.len(), expected_sources);
    assert_eq!(
        queue.document().items[0].source_keys.iter().map(String::as_str).collect::<Vec<_>>(),
        queue
            .document()
            .source_pins
            .iter()
            .map(|pin| pin.artifact_key.as_str())
            .collect::<Vec<_>>()
    );
    if profile {
        assert!(
            queue
                .document()
                .source_pins
                .iter()
                .any(|pin| pin.artifact_key == "framework" && pin.model == SourceModel::Profile)
        );
        assert!(
            queue
                .document()
                .source_pins
                .iter()
                .any(|pin| pin.artifact_key == "resolved"
                    && pin.model == SourceModel::ResolvedCatalog)
        );
    }
    assert_private_absent(&fixture, &queue_raw);
    let responses = vec![
        response(&fixture, 0, Disposition::Approve),
        response(&fixture, 1, Disposition::Approve),
    ];
    let status = status_bytes(&fixture, &responses);
    assert_status(&status, ItemState::QuorumMet, 2, 2, 0);
    assert_generation(&fixture, &responses, &status);
    let closed =
        decode::decode_dispositions(&status, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let document = closed.document();
    assert_eq!(document.counts.states.quorum_met, 1);
    assert_eq!(document.items[0].required_seats, 2);
    assert_eq!(document.items[0].met_seats.len(), 2);
    assert!(document.items[0].unmet_seats.is_empty());
    assert!(!document.items[0].blocking);
    let mut reviewers = document.items[0]
        .met_seats
        .iter()
        .map(|seat| seat.reviewer_key.as_str())
        .collect::<Vec<_>>();
    reviewers.sort_unstable();
    assert_eq!(reviewers, ["a-reviewer", "b-reviewer"]);
    assert_eq!(
        document.items[0].response_ids.iter().map(String::as_str).collect::<Vec<_>>(),
        &RESPONSE_IDS[..2]
    );
    assert_eq!(document.source_pins.len(), expected_sources);
    assert!(document.source_pins == queue.document().source_pins);
    for (recorded, path) in document.responses.iter().zip(&responses) {
        let actual = std::fs::read(fixture.root().join(path)).unwrap();
        assert_eq!(recorded.raw_sha256, crate::hashing::sha256_hex(&actual));
        assert_eq!(recorded.byte_length, u64::try_from(actual.len()).unwrap());
    }
    assert_private_absent(&fixture, &status);
    merge(&options(&fixture, &responses), Path::new("dispositions.json"), &mut NoopControl)
        .unwrap();
    let merged = std::fs::read(fixture.root().join("dispositions.json")).unwrap();
    assert_eq!(merged, status);
    assert_native_unchanged(&fixture, &originals);
}

/// Actual report-source Catalog proof gates two distinct asserted reviewer approvals.
#[test]
fn catalog_commands_produce_current_quorum_without_native_source_mutation() {
    actual_quorum(false, 6);
}

/// Actual Profile and explicit resolved companion remain in the whole current source pin union.
#[test]
fn profile_commands_include_actual_resolved_companion_in_current_quorum() {
    actual_quorum(true, 7);
}

/// Genuine selected facts keep the full explicit/omitted/state/map/pair denominators separate.
#[test]
fn genuine_adapter_selection_keeps_complete_three_control_denominator() {
    let fixture = Fixture::new(false);
    let mut ledger = ContractLedger::default();
    let mut capture =
        ReviewCapture::new(fixture.root(), &[], &mut ledger, &mut NoopControl).unwrap();
    let locator = capture
        .required(
            Path::new("locator.json"),
            CaptureRole::ReviewPrivateConfig,
            Pool::Auxiliary,
            1_048_576,
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    let pending =
        applicability_capture::prepare(&mut capture, locator, &mut ledger, &mut NoopControl)
            .unwrap();
    let closure = pending.seal(Rc::new(capture.finish()), &mut ledger, &mut NoopControl).unwrap();
    let prepared =
        applicability_adapter::prepare(&closure, &["c1"], &mut ledger, &mut NoopControl).unwrap();
    assert_eq!(prepared.selected_count(), 1);
    assert_eq!(prepared.complete_controls(), 3);
    assert_eq!(prepared.explicit_decisions(), 2);
    assert_eq!(prepared.omitted_decisions(), 1);
    assert_eq!(prepared.complete_counts().total, 3);
    assert_eq!(prepared.complete_counts().applicable_mapped, 1);
    assert_eq!(prepared.complete_counts().applicable_unmapped, 0);
    assert_eq!(prepared.complete_counts().not_applicable, 0);
    assert_eq!(prepared.complete_counts().deferred, 1);
    assert_eq!(prepared.complete_counts().under_review, 1);
    assert_eq!(prepared.complete_counts().applicable_reviewed_no_relationship, 0);
    assert_eq!(prepared.complete_maps(), 1);
    assert_eq!(prepared.complete_pair_inspections(), 4);
    let fact = prepared.facts().next().unwrap();
    assert_eq!(fact.subject_id(), "c1");
    assert_eq!(
        fact.context().reason_codes.iter().map(String::as_str).collect::<Vec<_>>(),
        ["applicability-re-review", "applicable-mapped", "scope-applicable"]
    );
    assert_eq!(fact.context().related_subject_ids, Vec::<String>::new());
    prepared.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
    assert!(
        applicability_adapter::prepare(&closure, &["c3"], &mut ledger, &mut NoopControl).is_err()
    );
}

/// Omitted native control cannot create a selected decision; c1 is a genuine success canary.
#[test]
fn omitted_control_selection_refuses_before_queue_publication() {
    let fixture = initialized(false, &["c1"]);
    let originals = native_originals(&fixture);
    write_policy(&fixture, &["c3"]);
    assert!(initialize(&fixture, Path::new("omitted-queue.json")).is_err());
    assert!(!fixture.root().join("omitted-queue.json").exists());
    assert_native_unchanged(&fixture, &originals);
}

/// A duplicate actual file occurrence remains counted but cannot supply another distinct seat.
#[test]
fn repeated_actual_response_occurrences_preserve_counts_without_extra_quorum_seats() {
    let fixture = initialized(false, &["c1"]);
    let first = response(&fixture, 0, Disposition::Approve);
    let second = response(&fixture, 1, Disposition::Approve);
    let repeated = vec![first.clone(), first.clone()];
    let incomplete = status_bytes(&fixture, &repeated);
    assert_status(&incomplete, ItemState::InReview, 2, 1, 1);
    assert_generation(&fixture, &repeated, &incomplete);
    let closed =
        decode::decode_dispositions(&incomplete, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    assert_eq!(closed.document().items[0].met_seats.len(), 1);
    assert_eq!(closed.document().items[0].unmet_seats.len(), 1);
    let complete = vec![first.clone(), first, second];
    let filled = status_bytes(&fixture, &complete);
    assert_status(&filled, ItemState::QuorumMet, 3, 2, 1);
    assert_generation(&fixture, &complete, &filled);
    merge(&options(&fixture, &complete), Path::new("repeated-dispositions.json"), &mut NoopControl)
        .unwrap();
    assert_eq!(std::fs::read(fixture.root().join("repeated-dispositions.json")).unwrap(), filled);
}

/// A third assigned reviewer's dissent remains complete and blocks otherwise filled seats.
#[test]
fn genuine_dissent_survives_current_output_and_blocks_quorum() {
    let fixture = initialized(false, &["c1"]);
    let responses = vec![
        response(&fixture, 0, Disposition::Approve),
        response(&fixture, 1, Disposition::Approve),
        response(&fixture, 2, Disposition::Reject),
    ];
    let bytes = status_bytes(&fixture, &responses);
    assert_status(&bytes, ItemState::Conflicted, 3, 3, 0);
    let closed =
        decode::decode_dispositions(&bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let item = &closed.document().items[0];
    assert!(item.blocking);
    assert_eq!(item.response_ids.iter().map(String::as_str).collect::<Vec<_>>(), RESPONSE_IDS);
    assert_eq!(item.dissent_ids.iter().map(String::as_str).collect::<Vec<_>>(), [RESPONSE_IDS[2]]);
    assert_eq!(closed.document().counts.states.conflicted, 1);
    assert_eq!(closed.document().counts.states.quorum_met, 0);
    assert_private_absent(&fixture, &bytes);
}

/// Use a prior successful current result before changing one actual complete-source original.
fn actual_drift_refusal(name: &str) {
    let fixture = initialized(false, &["c1"]);
    let responses = vec![
        response(&fixture, 0, Disposition::Approve),
        response(&fixture, 1, Disposition::Approve),
    ];
    assert_status(&status_bytes(&fixture, &responses), ItemState::QuorumMet, 2, 2, 0);
    let path = fixture.root().join(name);
    let mut raw = std::fs::read(&path).unwrap();
    raw.push(b'\n');
    std::fs::write(path, raw).unwrap();
    let mut writer = Vec::new();
    assert!(status(&options(&fixture, &responses), &mut writer, &mut NoopControl).is_err());
    assert_eq!(writer, Vec::<u8>::new());
    assert!(
        merge(&options(&fixture, &responses), Path::new("after-drift.json"), &mut NoopControl)
            .is_err()
    );
    assert!(!fixture.root().join("after-drift.json").exists());
}

/// Semantically equivalent changed raw report cannot reuse the prior report-source approval.
#[test]
fn changed_actual_report_refuses_current_status_and_new_merge() {
    actual_drift_refusal("report.json");
}

/// Full applicability manifest raw drift cannot hide behind an unchanged stored report.
#[test]
fn changed_actual_manifest_refuses_current_status_and_new_merge() {
    actual_drift_refusal("applicability.json");
}

/// Native Mapping original raw drift invalidates the complete current applicability closure.
#[test]
fn changed_actual_mapping_refuses_current_status_and_new_merge() {
    actual_drift_refusal("mapping.json");
}

/// Rebind all inert queue hashes after a deliberate foreign-domain mutation.
/// The resulting closed queue remains an assertion and cannot confer native proof.
fn replace_with_mapping_domain(fixture: &Fixture, index: usize) {
    let queue_path = fixture.root().join("queue.json");
    let mut queue: QueueDocument =
        serde_json::from_slice(&std::fs::read(&queue_path).unwrap()).unwrap();
    let mapping = read_value(&fixture.root().join("mapping.json"));
    let actual_native_id =
        mapping["mapping-collection"]["mappings"][0]["maps"][0]["uuid"].as_str().unwrap();
    queue.items[index].domain = Domain::MappingAssertion;
    queue.items[index].adapter_version = "forge.mapping-review/1".into();
    queue.items[index].subject_id = actual_native_id.into();
    let mut ledger = ContractLedger::default();
    queue.items[index].context_sha256 =
        validate::context_hash(&queue, &queue.items[index], &mut ledger).unwrap();
    queue.items[index].policy_sha256 =
        validate::policy_hash(&queue, &queue.policies[0], &queue.items[index], &mut ledger)
            .unwrap();
    queue.items[index].item_id =
        validate::item_id(&queue, &queue.items[index], &mut ledger).unwrap();
    let raw = serde_json::to_vec(&queue).unwrap();
    decode::decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    std::fs::write(queue_path, raw).unwrap();
}

/// Refuse an actual closed but native-foreign queue without writing even an empty current result.
fn foreign_queue_refuses(fixture: &Fixture) {
    let mut writer = Vec::new();
    assert!(status(&options(fixture, &[]), &mut writer, &mut NoopControl).is_err());
    assert_eq!(writer, Vec::<u8>::new());
    assert!(
        merge(&options(fixture, &[]), Path::new("foreign-dispositions.json"), &mut NoopControl)
            .is_err()
    );
    assert!(!fixture.root().join("foreign-dispositions.json").exists());
}

/// Real canonical Mapping identity with recomputed assertion hashes cannot switch source domain.
#[test]
fn mapping_queue_domain_refuses_actual_applicability_source_closure() {
    let fixture = initialized(false, &["c1"]);
    let mut before = Vec::new();
    status(&options(&fixture, &[]), &mut before, &mut NoopControl).unwrap();
    assert_status(&before, ItemState::Assigned, 0, 0, 0);
    replace_with_mapping_domain(&fixture, 0);
    foreign_queue_refuses(&fixture);
}

/// A current applicability item cannot mask another fully rebound foreign-domain item.
#[test]
fn mixed_domain_queue_refuses_complete_actual_applicability_source_closure() {
    let fixture = initialized(false, &["c1", "c2"]);
    let before = status_bytes(&fixture, &[]);
    let closed =
        decode::decode_dispositions(&before, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    assert_eq!(closed.document().counts.items, 2);
    assert_eq!(closed.document().counts.states.assigned, 2);
    replace_with_mapping_domain(&fixture, 1);
    foreign_queue_refuses(&fixture);
}

/// Real positive workflow cannot overwrite its queue, response or another retained output.
#[test]
fn actual_queue_response_and_merge_outputs_preserve_existing_bytes() {
    let fixture = initialized(false, &["c1"]);
    let originals = native_originals(&fixture);
    let queue = std::fs::read(fixture.root().join("queue.json")).unwrap();
    assert!(initialize(&fixture, Path::new("queue.json")).is_err());
    assert_eq!(std::fs::read(fixture.root().join("queue.json")).unwrap(), queue);
    let first = response(&fixture, 0, Disposition::Approve);
    let first_raw = std::fs::read(fixture.root().join(&first)).unwrap();
    assert!(
        respond(
            &RespondOptions {
                project_root: fixture.root(),
                queue: Path::new("queue.json"),
                rationale_file: Path::new("rationale-0.txt"),
                item_key: "item-c1",
                reviewer_key: "a-reviewer",
                reviewer_role: "review",
                disposition: Disposition::Reject,
                responded_at: "2026-10-04T11:00:00Z",
                response_id: RESPONSE_IDS[0],
                abstention_reason: None,
                supersedes: None,
                output: &first,
            },
            &mut NoopControl
        )
        .is_err()
    );
    assert_eq!(std::fs::read(fixture.root().join(&first)).unwrap(), first_raw);
    let second = response(&fixture, 1, Disposition::Approve);
    let responses = vec![first, second];
    assert_status(&status_bytes(&fixture, &responses), ItemState::QuorumMet, 2, 2, 0);
    std::fs::write(fixture.root().join("existing-output.json"), b"original retained output")
        .unwrap();
    assert!(
        merge(&options(&fixture, &responses), Path::new("existing-output.json"), &mut NoopControl)
            .is_err()
    );
    assert_eq!(
        std::fs::read(fixture.root().join("existing-output.json")).unwrap(),
        b"original retained output"
    );
    assert_native_unchanged(&fixture, &originals);
}

/// Rebind a declared synthetic lifecycle to every actual regenerated native/report original.
/// Maintained record validation and event IDs preserve the whole neutral history;
/// this test orchestration neither constructs capture proof nor authenticates its actors.
fn refresh_synthetic_report_approval(fixture: &Fixture) {
    use crate::lifecycle::record;

    let path = fixture.root().join("lifecycle.json");
    let mut lifecycle = read_value(&path);
    let source_hash =
        crate::hashing::sha256_hex(&std::fs::read(fixture.root().join("report.json")).unwrap());
    lifecycle["policy"]["source"]["sha256"] = json!(source_hash);
    let generated = lifecycle["policy"]["generated_artifacts"].as_array_mut().unwrap();
    for row in generated.iter_mut() {
        let raw = std::fs::read(fixture.root().join(row["path"].as_str().unwrap())).unwrap();
        let native: Value = serde_json::from_slice(&raw).unwrap();
        let kind = row["oscal_type"].as_str().unwrap();
        let model = if kind == "mapping-collection" {
            crate::validate::OscalModelType::Mapping
        } else {
            crate::validate::OscalModelType::Catalog
        };
        assert!(crate::validate::validate_artifact(&native, model).unwrap().is_valid);
        let root_uuid = native[kind]["uuid"].clone();
        row["sha256"] = json!(crate::hashing::sha256_hex(&raw));
        row["root_uuid"] = root_uuid;
    }
    let generated_hashes = generated
        .iter()
        .map(|row| json!({"path":row["path"],"sha256":row["sha256"]}))
        .collect::<Vec<_>>();
    let fingerprints = json!({"source_sha256":source_hash,"generated_artifacts":generated_hashes});
    for event in lifecycle["history"].as_array_mut().unwrap() {
        event["fingerprints"] = fingerprints.clone();
    }
    let mut declared: record::LifecycleRecord = serde_json::from_value(lifecycle).unwrap();
    for index in 0..declared.history.len() {
        declared.history[index].event_id =
            record::event_id(&declared, &declared.history[index]).unwrap();
    }
    record::validate(&declared).unwrap();
    std::fs::write(path, serde_json::to_vec(&declared).unwrap()).unwrap();
}

/// Extend genuine saved Catalog inputs, then rerun maintained inventory/Mapping/report producers.
/// Seven complete controls retain six classifications, six explicit decisions and one omission.
fn expanded_category_fixture() -> Fixture {
    let fixture = Fixture::new(false);
    let mut catalog = read_value(&fixture.root().join("framework.json"));
    let controls = catalog["catalog"]["groups"][0]["controls"].as_array_mut().unwrap();
    for id in ["c4", "c5", "c6", "c7"] {
        controls.push(json!({"id":id,"title":format!("Control {id}"),
            "parts":[{"id":format!("{id}_smt"),"name":"statement",
                "prose":format!("Actual synthetic statement {id}.")}]}));
    }
    write_value(&fixture.root().join("framework.json"), &catalog);
    let mut mapping = read_value(&fixture.root().join("mapping-manifest.json"));
    let mut framework: crate::mapping::manifest::ResourceManifest =
        serde_json::from_value(mapping["mapping"]["target"].clone()).unwrap();
    framework.expected_sha256 = None;
    framework.inventory = None;
    let loaded =
        crate::mapping::inventory::load(fixture.root(), "expanded synthetic framework", &framework)
            .unwrap();
    framework.expected_sha256 = Some(loaded.evidence.raw_sha256.clone());
    framework.inventory = Some(loaded.snapshot());
    mapping["mapping"]["target"] = serde_json::to_value(&framework).unwrap();
    let mut no_relationship = mapping["mapping"]["maps"][0].clone();
    no_relationship["key"] = json!("reviewed-no-relationship");
    no_relationship["relationship"] = json!("no-relationship");
    no_relationship["targets"] = json!([
        {"type":"control","id_ref":"c4"},
        {"type":"control","id_ref":"c6"}
    ]);
    no_relationship["rationale"] = json!("PRIVATE-NO-RELATIONSHIP-RATIONALE");
    mapping["mapping"]["maps"].as_array_mut().unwrap().push(no_relationship);
    write_value(&fixture.root().join("mapping-manifest.json"), &mapping);
    let built = crate::mapping::prepare(&fixture.root().join("mapping-manifest.json"), None, false)
        .unwrap();
    std::fs::write(fixture.root().join("mapping.json"), built.artifact_json.as_bytes()).unwrap();
    let mut source = read_value(&fixture.root().join("applicability.json"));
    source["framework"] = serde_json::to_value(framework).unwrap();
    source["decisions"].as_array_mut().unwrap().extend([
        json!({"control_id":"c4","state":"applicable","reviewer_key":"scope-reviewer",
            "reviewed_at":"2026-08-25T09:00:00Z"}),
        json!({"control_id":"c5","state":"applicable","reviewer_key":"scope-reviewer",
            "reviewed_at":"2026-08-25T09:00:00Z"}),
        json!({"control_id":"c6","state":"not-applicable","reviewer_key":"scope-reviewer",
            "reviewed_at":"2026-08-25T09:00:00Z","rationale":"PRIVATE-SCOPE-RATIONALE"}),
        json!({"control_id":"c7","state":"under-review","reviewer_key":"scope-reviewer",
            "note":"PRIVATE-UNDER-REVIEW-NOTE"}),
    ]);
    write_value(&fixture.root().join("applicability.json"), &source);
    let report = crate::applicability::prepare_analysis(
        &fixture.root().join("applicability.json"),
        crate::applicability::model::ReportFilters::default(),
    )
    .unwrap();
    write_value(
        &fixture.root().join("report.json"),
        &serde_json::to_value(&report.report).unwrap(),
    );
    refresh_synthetic_report_approval(&fixture);
    fixture
}

/// Consume one actual full native capture before inspecting selected command outputs.
fn assert_complete_category_capture(fixture: &Fixture) {
    let mut ledger = ContractLedger::default();
    let mut capture =
        ReviewCapture::new(fixture.root(), &[], &mut ledger, &mut NoopControl).unwrap();
    let locator = capture
        .required(
            Path::new("locator.json"),
            CaptureRole::ReviewPrivateConfig,
            Pool::Auxiliary,
            1_048_576,
            &mut ledger,
            &mut NoopControl,
        )
        .unwrap();
    let pending =
        applicability_capture::prepare(&mut capture, locator, &mut ledger, &mut NoopControl)
            .unwrap();
    let closure = pending.seal(Rc::new(capture.finish()), &mut ledger, &mut NoopControl).unwrap();
    let prepared = applicability_adapter::prepare(
        &closure,
        &["c4", "c5", "c6", "c7"],
        &mut ledger,
        &mut NoopControl,
    )
    .unwrap();
    assert_eq!(prepared.selected_count(), 4);
    assert_eq!(prepared.complete_controls(), 7);
    assert_eq!(prepared.explicit_decisions(), 6);
    assert_eq!(prepared.omitted_decisions(), 1);
    let counts = prepared.complete_counts();
    assert_eq!(counts.total, 7);
    assert_eq!(counts.applicable_mapped, 1);
    assert_eq!(counts.applicable_reviewed_no_relationship, 1);
    assert_eq!(counts.applicable_unmapped, 1);
    assert_eq!(counts.not_applicable, 1);
    assert_eq!(counts.deferred, 1);
    assert_eq!(counts.under_review, 2);
    assert_eq!(prepared.complete_maps(), 2);
    assert_eq!(prepared.complete_pair_inspections(), 8);
    let c4 = closure.control_facts().iter().find(|row| row.control_id() == "c4").unwrap();
    assert_eq!(c4.positive_count(), 0);
    assert_eq!(c4.no_relationship_count(), 1);
    let c6 = closure.control_facts().iter().find(|row| row.control_id() == "c6").unwrap();
    assert_eq!(c6.no_relationship_count(), 1);
    assert_eq!(
        prepared
            .facts()
            .map(applicability_adapter::ApplicabilityFact::subject_id)
            .collect::<Vec<_>>(),
        ["c4", "c5", "c6", "c7"]
    );
    prepared.verify_inputs(&mut ledger, &mut NoopControl).unwrap();
}

/// All genuine categories survive selected re-review, with native edge counts distinct from pairs.
#[test]
fn selected_category_commands_preserve_complete_native_and_omitted_denominators() {
    let fixture = expanded_category_fixture();
    let originals = native_originals(&fixture);
    assert_complete_category_capture(&fixture);
    write_policy(&fixture, &["c4", "c5", "c6", "c7"]);
    initialize(&fixture, Path::new("queue.json")).unwrap();
    let queue_raw = std::fs::read(fixture.root().join("queue.json")).unwrap();
    let queue =
        decode::decode_queue(&queue_raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(queue.document().items.len(), 4);
    let expected_classes = [
        "applicable-reviewed-no-relationship",
        "applicable-unmapped",
        "not-applicable",
        "under-review",
    ];
    for (item, classification) in queue.document().items.iter().zip(expected_classes) {
        assert_eq!(item.domain, Domain::ApplicabilityDecision);
        assert_eq!(item.adapter_version, "forge.applicability-review/1");
        assert!(item.context.reason_codes.iter().any(|reason| reason == classification));
        assert_eq!(item.context.related_subject_ids, Vec::<String>::new());
    }
    let bytes = status_bytes(&fixture, &[]);
    let current =
        decode::decode_dispositions(&bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let document = current.document();
    assert_eq!(document.currentness, RecordedCurrentness::RecordedCurrent);
    assert_eq!(document.identity_disclaimer, IDENTITY_DISCLAIMER);
    assert_eq!(document.counts.items, 4);
    assert_eq!(document.counts.states.assigned, 4);
    assert_eq!(document.counts.response_files, 0);
    assert!(document.source_pins == queue.document().source_pins);
    assert_generation(&fixture, &[], &bytes);
    merge(&options(&fixture, &[]), Path::new("category-dispositions.json"), &mut NoopControl)
        .unwrap();
    assert_eq!(std::fs::read(fixture.root().join("category-dispositions.json")).unwrap(), bytes);
    for raw in [&queue_raw, &bytes] {
        assert_private_absent(&fixture, raw);
        let text = std::str::from_utf8(raw).unwrap();
        for marker in [
            "PRIVATE-NO-RELATIONSHIP-RATIONALE",
            "PRIVATE-SCOPE-RATIONALE",
            "PRIVATE-UNDER-REVIEW-NOTE",
        ] {
            assert!(!text.contains(marker));
        }
    }
    assert_native_unchanged(&fixture, &originals);
}

/// An empty assignment roster remains genuinely unassigned with all required seats intact.
#[test]
fn actual_unassigned_queue_preserves_unsatisfied_seats_in_current_merge() {
    let fixture = Fixture::new(false);
    let originals = native_originals(&fixture);
    write_policy(&fixture, &["c1"]);
    let policy_path = fixture.root().join("review-policy.json");
    let mut policy = read_value(&policy_path);
    policy["items"][0]["assignments"] = json!([]);
    write_value(&policy_path, &policy);
    initialize(&fixture, Path::new("queue.json")).unwrap();
    let bytes = status_bytes(&fixture, &[]);
    assert_status(&bytes, ItemState::Unassigned, 0, 0, 0);
    assert_generation(&fixture, &[], &bytes);
    let current =
        decode::decode_dispositions(&bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let document = current.document();
    assert_eq!(document.counts.states.unassigned, 1);
    assert_eq!(document.counts.states.quorum_met, 0);
    let item = &document.items[0];
    assert_eq!(item.required_seats, 2);
    assert!(item.met_seats.is_empty());
    assert_eq!(item.unmet_seats.len(), 2);
    assert_eq!(item.response_ids, Vec::<String>::new());
    assert_eq!(item.dissent_ids, Vec::<String>::new());
    assert!(!item.blocking);
    merge(&options(&fixture, &[]), Path::new("unassigned-dispositions.json"), &mut NoopControl)
        .unwrap();
    assert_eq!(std::fs::read(fixture.root().join("unassigned-dispositions.json")).unwrap(), bytes);
    assert_private_absent(&fixture, &bytes);
    assert_native_unchanged(&fixture, &originals);
}

/// An actual timely change request preserves dissent and blocks seats in the native current result.
#[test]
fn actual_change_request_blocks_current_status_and_preserves_merge_dissent() {
    let fixture = initialized(false, &["c1"]);
    let originals = native_originals(&fixture);
    let responses = vec![response(&fixture, 0, Disposition::RequestChanges)];
    let bytes = status_bytes(&fixture, &responses);
    assert_status(&bytes, ItemState::ChangesRequested, 1, 1, 0);
    assert_generation(&fixture, &responses, &bytes);
    let current =
        decode::decode_dispositions(&bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let document = current.document();
    assert_eq!(document.counts.states.changes_requested, 1);
    assert_eq!(document.counts.states.quorum_met, 0);
    assert_eq!(
        document.responses[0].classification,
        crate::review::wire::ResponseClassification::Current
    );
    let item = &document.items[0];
    assert!(item.blocking);
    assert_eq!(item.required_seats, 2);
    assert!(item.met_seats.is_empty());
    assert_eq!(item.unmet_seats.len(), 2);
    assert_eq!(item.response_ids.iter().map(String::as_str).collect::<Vec<_>>(), [RESPONSE_IDS[0]]);
    assert_eq!(item.dissent_ids.iter().map(String::as_str).collect::<Vec<_>>(), [RESPONSE_IDS[0]]);
    assert!(item.reason_codes.iter().any(|reason| reason == "request-changes-blocks"));
    merge(&options(&fixture, &responses), Path::new("changes-dispositions.json"), &mut NoopControl)
        .unwrap();
    assert_eq!(std::fs::read(fixture.root().join("changes-dispositions.json")).unwrap(), bytes);
    assert_private_absent(&fixture, &bytes);
    assert_native_unchanged(&fixture, &originals);
}

/// Exact asserted UTC due equality expires incomplete seats without erasing a timely approval.
#[test]
fn actual_due_equality_expires_incomplete_current_quorum_and_preserves_timely_vote() {
    let fixture = Fixture::new(false);
    let originals = native_originals(&fixture);
    write_policy(&fixture, &["c1"]);
    let policy_path = fixture.root().join("review-policy.json");
    let mut policy = read_value(&policy_path);
    policy["items"][0]["due_at"] = json!(AS_OF);
    write_value(&policy_path, &policy);
    initialize(&fixture, Path::new("queue.json")).unwrap();
    let mut assigned_options = options(&fixture, &[]);
    assigned_options.as_of = "2026-10-04T11:59:59Z";
    let mut assigned = Vec::new();
    status(&assigned_options, &mut assigned, &mut NoopControl).unwrap();
    let assigned_closed =
        decode::decode_dispositions(&assigned, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    assert_eq!(assigned_closed.document().items[0].state, ItemState::Assigned);
    assert_eq!(assigned_closed.document().items[0].unmet_seats.len(), 2);
    let responses = vec![response(&fixture, 0, Disposition::Approve)];
    let mut before_options = options(&fixture, &responses);
    before_options.as_of = "2026-10-04T11:59:59Z";
    let mut before = Vec::new();
    status(&before_options, &mut before, &mut NoopControl).unwrap();
    let before_closed =
        decode::decode_dispositions(&before, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    assert_eq!(before_closed.document().items[0].state, ItemState::InReview);
    assert_eq!(before_closed.document().items[0].met_seats.len(), 1);
    let bytes = status_bytes(&fixture, &responses);
    assert_status(&bytes, ItemState::Expired, 1, 1, 0);
    assert_generation(&fixture, &responses, &bytes);
    let current =
        decode::decode_dispositions(&bytes, &mut ContractLedger::default(), &mut NoopControl)
            .unwrap();
    let document = current.document();
    assert_eq!(document.counts.states.expired, 1);
    assert_eq!(document.counts.states.quorum_met, 0);
    assert_eq!(
        document.responses[0].classification,
        crate::review::wire::ResponseClassification::Current
    );
    let item = &document.items[0];
    assert!(!item.blocking);
    assert_eq!(item.required_seats, 2);
    assert_eq!(item.met_seats.len(), 1);
    assert_eq!(item.unmet_seats.len(), 1);
    assert_eq!(item.met_seats[0].reviewer_key, "a-reviewer");
    assert_eq!(item.met_seats[0].response_id, RESPONSE_IDS[0]);
    assert_eq!(item.response_ids.iter().map(String::as_str).collect::<Vec<_>>(), [RESPONSE_IDS[0]]);
    assert_eq!(item.dissent_ids, Vec::<String>::new());
    assert!(item.reason_codes.iter().any(|reason| reason == "deadline-unsatisfied"));
    assert_ne!(before_closed.document().closure_generation, document.closure_generation);
    merge(&options(&fixture, &responses), Path::new("expired-dispositions.json"), &mut NoopControl)
        .unwrap();
    assert_eq!(std::fs::read(fixture.root().join("expired-dispositions.json")).unwrap(), bytes);
    assert_private_absent(&fixture, &bytes);
    assert_native_unchanged(&fixture, &originals);
}

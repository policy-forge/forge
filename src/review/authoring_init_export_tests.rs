// Genuine captured Init exports and strict asserted-policy controls beneath the native fixture.
// No constructed native closure, lease, currentness Boolean or supplied fingerprint issues output.
use super::*;
use crate::review::{
    authoring_init_policy as policy, authoring_queue_export as export, decode_v3, encode_v3,
    hash_v3,
};

/// Explicit stable operation values independent of the actual native private as-of spelling.
const QUEUE_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
/// Exact review UTC seconds, rather than a wall clock or native timestamp substitution.
const CREATED: &str = "2026-10-05T00:00:00Z";
/// Complete asserted policy with a genuine cross-role substitution and no inferred authors.
fn request() -> Value {
    json!({"schema_version":"forge.review-authoring-plan-init/1",
        "roles":[{"key":"finance"},{"key":"security"}],
        "reviewers":[{"key":"alice","role_keys":["security"]},{"key":"bob","role_keys":["finance"]}],
        "policies":[{"key":"team","seats":[{"role_key":"security","count":1}],
            "substitutions":[{"seat_role":"security","reviewer_key":"bob","asserted_role":"finance","reason_code":"cross-review"}],
            "abstention_rule":"nonapproving","empty_abstention_reasons":["no-context","recusal"],
            "author_separation":"declared-keys"}],
        "items":[{"key":"full-plan","policy_key":"team","author_keys":["writer"],
            "assignments":[{"reviewer_key":"alice","role_key":"security"}],"due_at":null}]})
}
/// Persist complete maintained plan/locator plus exactly one actual separate Init policy.
fn saved(f: &Fixture, declarations: &Value) {
    originals(f);
    write(&f.root.join("init-policy.json"), declarations);
}
/// Issue only the real whole Init owner, using this operation's original ledger/controller.
fn current(
    f: &Fixture,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<CurrentAuthoringPlanClosure, ContractError> {
    let mut capture = begin(f, AuthoringOperation::Init, ledger, control);
    let pending = receiver::read_pending(&mut capture, Path::new("locator.json"), ledger, control)?;
    capture.required_authoring_policy(Path::new("init-policy.json"), ledger, control)?;
    pending.finish_and_seal(capture, ledger, control)
}
/// Compare fixed errors without needing Debug/Clone/public constructors on genuine owners.
fn error<T>(result: Result<T, ContractError>, expected: ContractError) {
    assert_eq!(result.err().expect("operation must refuse"), expected);
}
/// Plain policy intake only; a successful result never supplies a native owner.
fn plain(value: &Value) -> Result<crate::review::wire_v3::AuthoringPlanInitV1, ContractError> {
    let raw = serde_json::to_vec(value).unwrap();
    policy::decode(&raw, &mut ContractLedger::default(), &mut NoopControl)
}
/// Actual Catalog receiver derives all pins and private fingerprints before finite strict export.
#[test]
fn actual_catalog_complete_native_queue_export_and_strict_readback() {
    let f = Fixture::new(false, false);
    let asserted = request();
    saved(&f, &asserted);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    assert!(prepared.document().source_pins == owner.source_pins());
    assert_eq!(prepared.document().items[0].source_keys.len(), owner.source_pins().len());
    assert_eq!(serde_json::to_value(&prepared.document().roles).unwrap(), asserted["roles"]);
    assert_eq!(
        serde_json::to_value(&prepared.document().reviewers).unwrap(),
        asserted["reviewers"]
    );
    assert_eq!(serde_json::to_value(&prepared.document().policies).unwrap(), asserted["policies"]);
    let encoded = prepared.encode(&mut ledger, &mut control).unwrap();
    assert_eq!(encoded.bytes().last(), Some(&b'\n'));
    assert!(!encoded.bytes().ends_with(b"\n\n"));
    let decoded = decode_v3::decode_queue(encoded.bytes(), &mut ledger, &mut control).unwrap();
    assert!(decoded.document().source_pins == owner.source_pins());
    assert_eq!(decoded.document().created_at, CREATED);
    assert_eq!(decoded.raw_sha256(), sha256_hex(encoded.bytes()));
    assert_eq!(decoded.document().items[0].context.reason_codes, ["authoring-plan-current"]);
    assert_eq!(decoded.document().items[0].author_keys, ["writer"]);
    assert!(owner.originals().unwrap().queue_raw().is_none());
    assert_eq!(owner.originals().unwrap().responses().len(), 0);
    encoded.verify_inputs(&mut ledger, &mut control).unwrap();
    assert!(!f.root.join("new-output.json").exists());
}
/// Real Profile companion and native Mapping remain in the complete exact eight-field roster.
#[test]
fn actual_profile_mapping_export_keeps_full_native_companion_and_uuid_spelling() {
    let f = Fixture::new(true, true);
    saved(&f, &request());
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    let pins = &prepared.document().source_pins;
    assert!(pins.iter().any(|p| p.artifact_key == "authoring:mapping:0"));
    assert_eq!(
        pins.iter()
            .find(|p| p.artifact_key == "authoring:resolved")
            .unwrap()
            .native_root_uuid
            .as_deref(),
        Some("11111111-1111-4111-8111-111111111111")
    );
    assert!(pins == owner.source_pins());
    let encoded = prepared.encode(&mut ledger, &mut control).unwrap();
    encoded.verify_inputs(&mut ledger, &mut control).unwrap();
}
/// Native N order is genuinely separate from lexical public S order and excludes stored plan.
#[test]
fn multiple_actual_clauses_keep_complete_native_provenance_and_distinct_stored_plan() {
    let mut f = Fixture::new(false, false);
    f.answer();
    saved(&f, &request());
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    assert_eq!(
        owner.native_provenance().filter(|p| p.role.starts_with("human-clause-")).count(),
        2
    );
    assert!(owner.native_provenance().all(|p| p.role != "stored-plan"));
    ledger
        .derived(
            owner.native_provenance().len() * std::mem::size_of::<hash_v3::NativeProvenance<'_>>()
                + std::mem::size_of::<Vec<hash_v3::NativeProvenance<'_>>>(),
        )
        .unwrap();
    let native: Vec<_> = owner.native_provenance().collect();
    let native_digest = hash_v3::native_provenance(&native, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    let stored = owner.source_pins().iter().find(|p| p.artifact_key == "authoring:plan").unwrap();
    let project = &owner.facts().loaded.project;
    let expected = hash_v3::subject(
        &project.project_key,
        &project.as_of,
        &native_digest,
        &stored.raw_sha256,
        owner.source_pins(),
        &mut ledger,
        &mut control,
    )
    .unwrap();
    assert_eq!(prepared.document().items[0].subject_sha256, expected);
    assert_eq!(prepared.document().source_pins.len(), native.len() + 1);
}
/// Private project/date/native prose enter only admitted digests, never the public envelope.
#[test]
fn exact_private_native_date_and_key_are_hashed_without_exporting_private_prose() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    let expected = hash_v3::subject_id("native-project", &mut ledger, &mut control).unwrap();
    assert_eq!(prepared.document().items[0].subject_id, expected);
    let encoded = prepared.encode(&mut ledger, &mut control).unwrap();
    let raw = std::str::from_utf8(encoded.bytes()).unwrap();
    for private in [
        "native-project",
        "2026-09-08T01:30:00.250+01:30",
        "Private access title",
        "Private native interview",
        "café",
        "clause.md",
        "project.json",
    ] {
        assert!(!raw.contains(private));
    }
}
/// Required explicit null, family closure, unknown/duplicate keys and raw syntax are real predicates.
#[test]
fn strict_policy_null_closure_duplicates_and_original_syntax_refuse() {
    assert!(plain(&request()).is_ok());
    let mut missing = request();
    missing["items"][0].as_object_mut().unwrap().remove("due_at");
    let mut extra = request();
    extra["items"][0]["unexpected"] = json!(null);
    let mut family = request();
    family["schema_version"] = json!("forge.review-lifecycle-init/1");
    for value in [missing, extra, family] {
        error(plain(&value), ContractError::Invalid);
    }
    let good = serde_json::to_vec(&request()).unwrap();
    let mut duplicate = br#"{"schema_version":"forge.review-authoring-plan-init/1","#.to_vec();
    duplicate.extend_from_slice(&good[1..]);
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&good);
    let mut trailing = good.clone();
    trailing.extend_from_slice(b"{}");
    for raw in [duplicate, bom, trailing, vec![0xff], Vec::new()] {
        let mut ledger = ContractLedger::default();
        error(policy::decode(&raw, &mut ledger, &mut NoopControl), ContractError::Invalid);
        policy::decode(&good, &mut ledger, &mut NoopControl).unwrap();
    }
}
/// Full raw/string/depth bounds stay the existing logical limits, with no ordinary failure latch.
#[test]
fn policy_raw_string_depth_bounds_and_original_capacity_are_not_renewed() {
    let mut ledger = ContractLedger::default();
    error(
        policy::decode(&vec![b' '; 1_048_577], &mut ledger, &mut NoopControl),
        ContractError::Capacity,
    );
    error(
        policy::decode(&serde_json::to_vec(&request()).unwrap(), &mut ledger, &mut NoopControl),
        ContractError::Capacity,
    );
    for raw in [
        format!("{{\"x\":\"{}\"}}", "a".repeat(65_537)).into_bytes(),
        format!("{}0{}", "[".repeat(66), "]".repeat(66)).into_bytes(),
    ] {
        error(
            policy::decode(&raw, &mut ContractLedger::default(), &mut NoopControl),
            ContractError::Invalid,
        );
    }
}
/// Registry order, exact role membership and references are observed, never normalized.
#[test]
fn complete_asserted_rosters_order_membership_and_unknown_keys_refuse() {
    let mut variants = Vec::new();
    let mut v = request();
    v["roles"].as_array_mut().unwrap().reverse();
    variants.push(v);
    let mut v = request();
    v["reviewers"][1]["key"] = json!("alice");
    variants.push(v);
    let mut v = request();
    v["reviewers"][0]["role_keys"] = json!(["finance", "unknown"]);
    variants.push(v);
    let mut v = request();
    v["items"][0]["policy_key"] = json!("other");
    variants.push(v);
    for value in variants {
        error(plain(&value), ContractError::Invalid);
    }
}
/// Real assignment/substitution incidence applies exact declared-key author exclusion and seat roles.
#[test]
fn asserted_graph_author_exclusion_substitution_and_aggregate_bounds_are_exact() {
    let mut author = request();
    author["items"][0]["author_keys"] = json!(["alice"]);
    let mut wrong_seat = request();
    wrong_seat["policies"][0]["substitutions"][0]["seat_role"] = json!("finance");
    let mut wrong_membership = request();
    wrong_membership["policies"][0]["substitutions"][0]["asserted_role"] = json!("security");
    for value in [author, wrong_seat, wrong_membership] {
        error(plain(&value), ContractError::Invalid);
    }
    let mut over = request();
    over["policies"][0]["seats"] = json!([
        {"role_key":"finance","count":100},{"role_key":"security","count":1}]);
    error(plain(&over), ContractError::Capacity);
    let mut ledger = ContractLedger::default();
    for _ in 0..100_000 {
        ledger.graph(1, 0, 1).unwrap();
    }
    let raw = serde_json::to_vec(&request()).unwrap();
    error(policy::decode(&raw, &mut ledger, &mut NoopControl), ContractError::Capacity);
}
/// Every explicit asserted field, including substitutions/reasons/deadline, survives a genuine export.
#[test]
fn real_policy_arrays_and_due_are_moved_without_filtering_or_defaulting() {
    let f = Fixture::new(false, false);
    let mut declarations = request();
    declarations["items"][0]["due_at"] = json!("2026-10-06T00:00:00Z");
    saved(&f, &declarations);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    let item = &prepared.document().items[0];
    assert_eq!(item.due_at.as_deref(), Some("2026-10-06T00:00:00Z"));
    assert_eq!(
        serde_json::to_value(&item.assignments).unwrap(),
        declarations["items"][0]["assignments"]
    );
    assert_eq!(
        serde_json::to_value(&prepared.document().policies).unwrap(),
        declarations["policies"]
    );
}
/// Explicit review headers and due comparisons remain independent of native dates and clocks.
#[test]
fn actual_export_rejects_noncanonical_headers_and_equal_or_earlier_due() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    for (id, date) in [
        ("AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA", CREATED),
        ("00000000-0000-0000-0000-000000000000", CREATED),
        (QUEUE_ID, "2026-02-30T00:00:00Z"),
    ] {
        error(export::prepare(&owner, id, date, &mut ledger, &mut control), ContractError::Invalid);
    }
    for due in [CREATED, "2026-10-04T00:00:00Z"] {
        let mut declarations = request();
        declarations["items"][0]["due_at"] = json!(due);
        saved(&f, &declarations);
        let mut next = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let next_owner = current(&f, &mut next, &mut control).unwrap();
        error(
            export::prepare(&next_owner, QUEUE_ID, CREATED, &mut next, &mut control),
            ContractError::Invalid,
        );
    }
}
/// A genuinely sealed current-operation owner is not an Init source despite equal native plan data.
#[test]
fn genuine_status_operation_owner_cannot_be_cast_into_init_export() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    std::fs::write(f.root.join("queue.json"), b"actual unbound Queue registry original").unwrap();
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut capture = begin(&f, AuthoringOperation::Status, &mut ledger, &mut control);
    let pending =
        receiver::read_pending(&mut capture, Path::new("locator.json"), &mut ledger, &mut control)
            .unwrap();
    capture
        .required(
            Path::new("queue.json"),
            CaptureRole::ReviewQueue,
            Pool::Queue,
            10 * 1024 * 1024,
            &mut ledger,
            &mut control,
        )
        .unwrap();
    let owner = pending.finish_and_seal(capture, &mut ledger, &mut control).unwrap();
    error(
        export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control),
        ContractError::Binding,
    );
}
/// Genuine incomplete/extra Init registrations refuse before any new export owner can exist.
#[test]
fn actual_missing_policy_and_extra_inputs_cannot_issue_init_owner() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    for extra in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let mut capture = begin(&f, AuthoringOperation::Init, &mut ledger, &mut control);
        let pending = receiver::read_pending(
            &mut capture,
            Path::new("locator.json"),
            &mut ledger,
            &mut control,
        )
        .unwrap();
        if extra {
            capture
                .required_authoring_policy(Path::new("init-policy.json"), &mut ledger, &mut control)
                .unwrap();
            capture
                .required(
                    Path::new("policy.json"),
                    CaptureRole::ReviewPrivateConfig,
                    Pool::Auxiliary,
                    1024 * 1024,
                    &mut ledger,
                    &mut control,
                )
                .unwrap();
        }
        error(pending.finish_and_seal(capture, &mut ledger, &mut control), ContractError::Binding);
    }
}
/// Complete stored-plan and native physical alias refusals remain prerequisites to exporter success.
#[test]
fn actual_saved_plan_mismatch_and_native_alias_never_reach_export() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    let mut stored: Value =
        serde_json::from_slice(&std::fs::read(f.root.join("stored-plan.json")).unwrap()).unwrap();
    stored["provenance"]["baseline_review"]["rationale"] =
        json!("coherent different private review");
    write(&f.root.join("stored-plan.json"), &stored);
    let mut ledger = ContractLedger::default();
    error(current(&f, &mut ledger, &mut NoopControl), ContractError::Binding);
    write(
        &f.root.join("locator.json"),
        &json!({"schema_version":"forge.review-authoring-plan-inputs/1",
        "project":{"path":"project.json"}, "plan":{"path":"project.json"}}),
    );
    error(current(&f, &mut ContractLedger::default(), &mut NoopControl), ContractError::Binding);
}
/// Full output comparison refuses independently valid unused assertions and all native pin operands.
#[test]
fn genuine_prepared_output_binder_observes_whole_fields_and_exact_eight_pin_tuple() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    let expected = serde_json::to_value(prepared.document()).unwrap();
    let mut unused = expected.clone();
    unused["reviewers"].as_array_mut().unwrap().push(json!({"key":"zoe","role_keys":["security"]}));
    let mut header = expected.clone();
    header["created_at"] = json!("2026-10-05T01:00:00Z");
    for changed in [unused, header] {
        let raw = serde_json::to_vec(&changed).unwrap();
        let decoded = decode_v3::decode_queue(&raw, &mut ledger, &mut control).unwrap();
        error(prepared.bind_output(&decoded, &mut ledger, &mut control), ContractError::Binding);
    }
    for field in [
        "artifact_key",
        "kind",
        "raw_sha256",
        "byte_length",
        "schema_identity",
        "validation_profile",
        "native_model",
        "native_root_uuid",
    ] {
        let mut changed = expected.clone();
        let pin = changed["source_pins"][0].as_object_mut().unwrap();
        let replacement = match field {
            "byte_length" => json!(pin[field].as_u64().unwrap() + 1),
            "kind" => json!("authoring-pack"),
            "native_model" => json!("catalog"),
            "native_root_uuid" => json!("11111111-1111-4111-8111-111111111111"),
            "raw_sha256" => json!("b".repeat(64)),
            "schema_identity" => json!("changed"),
            _ => json!("different"),
        };
        pin.insert(field.into(), replacement);
        let actual: crate::review::wire_v3::QueueDocumentV3 =
            serde_json::from_value(changed).unwrap();
        error(
            encode_v3::compare_queues(prepared.document(), &actual, &mut ledger, &mut control),
            ContractError::Binding,
        );
    }
}
/// In-place changes preserve held Windows handles while genuine whole U detects policy/source drift.
#[test]
fn captured_policy_native_and_locator_drift_refuse_live_output_owner() {
    for name in ["init-policy.json", "clause.md", "locator.json", "stored-plan.json"] {
        let f = Fixture::new(false, false);
        saved(&f, &request());
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let owner = current(&f, &mut ledger, &mut control).unwrap();
        let prepared =
            export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
        let encoded = prepared.encode(&mut ledger, &mut control).unwrap();
        std::fs::write(f.root.join(name), b"changed actual held original bytes").unwrap();
        error(encoded.verify_inputs(&mut ledger, &mut control), ContractError::Binding);
        assert!(!f.root.join("new-output.json").exists());
    }
}
/// Real output preparation cannot renew previously consumed work or conceal the original first stop.
#[test]
fn actual_native_export_and_encode_preserve_first_capacity_without_counter_reset() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = current(&f, &mut ledger, &mut control).unwrap();
    let prepared = export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
    error(ledger.derived(33_554_433), ContractError::Capacity);
    error(prepared.encode(&mut ledger, &mut control), ContractError::Capacity);
    assert_eq!(ledger.bytes(0).unwrap_err(), ContractError::Capacity);
    assert!(!f.root.join("new-output.json").exists());
}
/// A malformed policy's real last postfence reaches the original transient stop and latches it.
#[test]
fn ordinary_policy_failure_original_postfence_and_first_stop_priority() {
    let mut baseline = FenceProbe { calls: 0, stop: None, fail: false };
    error(
        policy::decode(b"{", &mut ContractLedger::default(), &mut baseline),
        ContractError::Invalid,
    );
    let terminal = baseline.calls;
    assert!(terminal > 1);
    for fail in [false, true] {
        let mut caller = FenceProbe { calls: 0, stop: Some(terminal), fail };
        let mut ledger = ContractLedger::default();
        let expected = if fail {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::CancelRequested)
        };
        error(policy::decode(b"{", &mut ledger, &mut caller), expected);
        let count = caller.calls;
        error(
            policy::decode(&serde_json::to_vec(&request()).unwrap(), &mut ledger, &mut caller),
            expected,
        );
        assert_eq!(caller.calls, count);
    }
}
/// Calibrated actual final successful encode callback retains the same owner but cannot return output.
#[test]
fn genuine_successful_export_final_checkpoint_stops_output_before_return() {
    let f = Fixture::new(false, false);
    saved(&f, &request());
    let mut baseline = FenceProbe { calls: 0, stop: None, fail: false };
    {
        let mut ledger = ContractLedger::default();
        let mut control = ReviewControl::accept(&mut baseline);
        let owner = current(&f, &mut ledger, &mut control).unwrap();
        export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control)
            .unwrap()
            .encode(&mut ledger, &mut control)
            .unwrap();
    }
    let terminal = baseline.calls;
    assert!(terminal > 100);
    for fail in [false, true] {
        let mut caller = FenceProbe { calls: 0, stop: Some(terminal), fail };
        let mut ledger = ContractLedger::default();
        let mut control = ReviewControl::accept(&mut caller);
        let owner = current(&f, &mut ledger, &mut control).unwrap();
        let prepared =
            export::prepare(&owner, QUEUE_ID, CREATED, &mut ledger, &mut control).unwrap();
        let expected = if fail {
            ContractError::ControlFailed
        } else {
            ContractError::Interrupted(Interruption::CancelRequested)
        };
        error(prepared.encode(&mut ledger, &mut control), expected);
        assert_eq!(ledger.bytes(0).unwrap_err(), expected);
        assert!(!f.root.join("new-output.json").exists());
    }
}

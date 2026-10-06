//! Private declaration/selection controls only, not genuine native-current workflow evidence.
use super::{init_request, sort_subjects};
use crate::review::decode::{ContractError, ContractLedger};
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError,
};
use serde_json::{Value, json};

/// Exact inert private request contains no native pin, hash or proof declaration.
fn request_fixture() -> Value {
    json!({"schema_version":"forge.review-init/1",
        "roles":[{"key":"review"}],
        "reviewers":[{"key":"reviewer","role_keys":["review"]}],
        "policies":[{"key":"policy","seats":[{"role_key":"review","count":1}],
            "substitutions":[],"abstention_rule":"nonapproving","empty_abstention_reasons":[],"author_separation":"declared-keys"}],
        "items":[{"key":"item","subject_id":"11111111-1111-4111-8111-111111111111","policy_key":"policy","author_keys":["author"],
            "assignments":[{"reviewer_key":"reviewer","role_key":"review"}],"due_at":null}]})
}

/// Check private request refusal using the same actual strict raw decoder, never a proof factory.
fn refused(value: &Value) {
    let raw = serde_json::to_vec(value).unwrap();
    assert!(init_request(&raw, &mut ContractLedger::default(), &mut NoopControl).is_err());
}

/// Explicit nullable due time is accepted as declaration storage, not native/current proof.
#[test]
fn closed_private_request_has_required_nullable_due_and_no_proof_fields() {
    let raw = serde_json::to_vec(&request_fixture()).unwrap();
    let request = init_request(&raw, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(request.items.len(), 1);
    assert_eq!(request.items[0].due_at, None);
    assert_eq!(request.items[0].subject_id, "11111111-1111-4111-8111-111111111111");
}

/// Omitting the nullable field must not receive serde's implicit Option default.
#[test]
fn missing_due_and_unknown_native_authority_fields_are_refused() {
    let mut missing = request_fixture();
    missing["items"][0].as_object_mut().unwrap().remove("due_at");
    refused(&missing);
    for field in ["source_pins", "subject_sha256", "context", "approved", "proof"] {
        let mut unknown = request_fixture();
        unknown["items"][0][field] = json!(true);
        refused(&unknown);
    }
    let mut unknown = request_fixture();
    unknown["source_pins"] = json!([]);
    refused(&unknown);
}

/// Duplicate keys, BOM and non-UTF8 raw bytes are rejected without normalization/reconstruction.
#[test]
fn raw_private_request_duplicate_bom_and_binary_refuse() {
    let raw = serde_json::to_vec(&request_fixture()).unwrap();
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&raw);
    assert!(init_request(&bom, &mut ContractLedger::default(), &mut NoopControl).is_err());
    let duplicate =
        br#"{"schema_version":"forge.review-init/1","schema_version":"forge.review-init/1"}"#;
    assert!(init_request(duplicate, &mut ContractLedger::default(), &mut NoopControl).is_err());
    assert!(init_request(&[0xff], &mut ContractLedger::default(), &mut NoopControl).is_err());
}

/// Before typed growth, nested cardinality/closedness and one-MiB raw limits refuse whole requests.
#[test]
fn raw_and_nested_request_limits_precede_typed_growth() {
    let mut seats = request_fixture();
    seats["policies"][0]["seats"] = json!([{"role_key":"review","count":0}]);
    refused(&seats);
    let mut nulls = request_fixture();
    nulls["items"][0]["assignments"] = Value::Null;
    refused(&nulls);
    let mut extra = request_fixture();
    extra["reviewers"][0]["authenticated"] = json!(true);
    refused(&extra);
    let mut wide = request_fixture();
    wide["roles"] = Value::Array((0..33).map(|i| json!({"key":format!("r{i}")})).collect());
    refused(&wide);
    let raw = vec![b' '; 1_048_577];
    assert_eq!(
        init_request(&raw, &mut ContractLedger::default(), &mut NoopControl).err(),
        Some(ContractError::Capacity)
    );
}

/// Native selection is ordered independently of queue-local key/request order.
#[test]
fn selected_native_ids_sort_without_mutating_declared_item_order() {
    let first = "11111111-1111-4111-8111-111111111111";
    let second = "22222222-2222-4222-8222-222222222222";
    let original = [second, first];
    let mut selection = original;
    sort_subjects(&mut selection, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    assert_eq!(selection, [first, second]);
    assert_eq!(original, [second, first]);
}

/// Same subject twice or noncanonical UUID is refused rather than silently deduplicated.
#[test]
fn duplicate_and_noncanonical_native_selection_refuse() {
    let id = "11111111-1111-4111-8111-111111111111";
    let mut duplicate = [id, id];
    assert_eq!(
        sort_subjects(&mut duplicate, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Binding)
    );
    let mut short = ["11111111111141118111111111111111"];
    assert_eq!(
        sort_subjects(&mut short, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    );
}

/// One actual typed caller stop; it supplies no currentness or successful empty result.
struct Cancel;
impl WorkControl for Cancel {
    /// Refuse at the initial actual request checkpoint with the maintained typed reason.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    }
    /// Keep the caller's exact reason sticky.
    fn interruption(&self) -> Option<Interruption> {
        Some(Interruption::CancelRequested)
    }
}

/// Private parse failure preserves the first interruption even after a later valid declaration.
#[test]
fn private_request_control_stop_is_exact_and_sticky() {
    let raw = serde_json::to_vec(&request_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    assert_eq!(
        init_request(&raw, &mut ledger, &mut Cancel).err(),
        Some(ContractError::Interrupted(Interruption::CancelRequested))
    );
    assert_eq!(
        init_request(&raw, &mut ledger, &mut NoopControl).err(),
        Some(ContractError::Interrupted(Interruption::CancelRequested))
    );
}

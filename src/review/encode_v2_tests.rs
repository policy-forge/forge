//! Prospective component controls using the actual finite writer and strict /2 decoder.
//! All typed fixtures and loaded currentness labels are ordinary nonauthorizing data.

use super::{MAX_RECORDED_BYTES, dispositions, encoded, readback};
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::decode_v2;
use crate::review::wire::StateCounts;
use crate::review::wire_v2::{
    Disposition, DispositionCounts, DispositionsDocumentV2, IDENTITY_DISCLAIMER, ItemDisposition,
    ItemState, MetSeat, NativeModelV2, RecordedCurrentness, RecordedResponse,
    ResponseClassification, SourceKindV2, SourcePinV2,
};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};
use serde::Serialize;
use serde::ser::SerializeSeq;

/// Exact canonical asserted response UUIDs; no real reviewer identities are supplied.
const APPROVAL: &str = "44444444-4444-4444-8444-444444444444";
/// Historical dissent remains in every complete recorded view.
const DISSENT: &str = "55555555-5555-4555-8555-555555555555";

/// A complete declared bundle with explicit nulls, native UUID spelling and stale dissent.
fn fixture() -> DispositionsDocumentV2 {
    DispositionsDocumentV2 {
        schema_version: "forge.review-dispositions/2".into(),
        identity_disclaimer: IDENTITY_DISCLAIMER.into(),
        queue_id: "11111111-1111-4111-8111-111111111111".into(),
        queue_raw_sha256: "1".repeat(64),
        as_of: "2026-10-05T00:00:02Z".into(),
        currentness: RecordedCurrentness::Unverified,
        closure_generation: None,
        source_pins: pins(),
        responses: vec![
            RecordedResponse {
                response_id: APPROVAL.into(),
                raw_sha256: "4".repeat(64),
                byte_length: 100,
                item_key: "item".into(),
                reviewer_key: "reviewer-1".into(),
                reviewer_role: "review".into(),
                disposition: Disposition::Approve,
                responded_at: "2026-10-05T00:00:00Z".into(),
                classification: ResponseClassification::Current,
            },
            RecordedResponse {
                response_id: DISSENT.into(),
                raw_sha256: "5".repeat(64),
                byte_length: 150,
                item_key: "item".into(),
                reviewer_key: "reviewer-2".into(),
                reviewer_role: "review".into(),
                disposition: Disposition::Reject,
                responded_at: "2026-10-05T00:00:01Z".into(),
                classification: ResponseClassification::Stale,
            },
        ],
        items: vec![ItemDisposition {
            item_key: "item".into(),
            item_id: "66666666-6666-4666-8666-666666666666".into(),
            state: ItemState::QuorumMet,
            reason_codes: vec!["quorum-met".into()],
            required_seats: 1,
            met_seats: vec![MetSeat {
                role_key: "review".into(),
                ordinal: 0,
                reviewer_key: "reviewer-1".into(),
                response_id: APPROVAL.into(),
            }],
            unmet_seats: vec![],
            response_ids: vec![APPROVAL.into(), DISSENT.into()],
            dissent_ids: vec![DISSENT.into()],
            blocking: false,
        }],
        counts: DispositionCounts {
            items: 1,
            response_files: 3,
            unique_responses: 2,
            exact_duplicates: 1,
            states: StateCounts {
                unassigned: 0,
                assigned: 0,
                in_review: 0,
                conflicted: 0,
                changes_requested: 0,
                quorum_met: 1,
                expired: 0,
                stale: 0,
            },
        },
    }
}
/// Whole role-qualified public roster, retaining the exact native generated UUID spelling.
fn pins() -> Vec<SourcePinV2> {
    vec![
        SourcePinV2 {
            artifact_key: "lifecycle:generated:0".into(),
            kind: SourceKindV2::GeneratedArtifact,
            raw_sha256: "2".repeat(64),
            byte_length: 128,
            schema_identity: None,
            validation_profile: "forge.lifecycle-generated-identity/1".into(),
            native_model: Some(NativeModelV2::Catalog),
            native_root_uuid: Some("{ABCDEFAB-1234-5567-89AB-CDEF01234567}".into()),
        },
        SourcePinV2 {
            artifact_key: "lifecycle:record".into(),
            kind: SourceKindV2::LifecycleRecord,
            raw_sha256: "3".repeat(64),
            byte_length: 512,
            schema_identity: Some("forge.policy-lifecycle/2".into()),
            validation_profile: "forge.lifecycle-record-intrinsic/1".into(),
            native_model: None,
            native_root_uuid: None,
        },
        SourcePinV2 {
            artifact_key: "lifecycle:source".into(),
            kind: SourceKindV2::OpaqueSource,
            raw_sha256: crate::hashing::sha256_hex(b""),
            byte_length: 0,
            schema_identity: None,
            validation_profile: "forge.opaque-source-bytes/1".into(),
            native_model: None,
            native_root_uuid: None,
        },
    ]
}
/// Observe actual callbacks on one controller; tests arm absolute subsequent call numbers.
#[derive(Default)]
struct Control {
    /// Total actual caller callbacks, retained across multiple codec calls.
    calls: usize,
    /// One absolute real call on which this controller must fail.
    at: Option<usize>,
    /// Choose safe control failure instead of cancellation at the same boundary.
    failed: bool,
}
impl WorkControl for Control {
    /// Return the exact requested failure without starting a new operation or deadline.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.at == Some(self.calls) {
            if self.failed {
                return Err(WorkError::Failed(Error::invalid()));
            }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// These controls stop through the actual callback result, not a proof flag.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}
/// Exact expected actual stop, preserving safe failure versus typed cancellation.
fn expected_stop(failed: bool) -> ContractError {
    if failed {
        ContractError::ControlFailed
    } else {
        ContractError::Interrupted(Interruption::CancelRequested)
    }
}
/// Serialize ordinary fixture bytes; only the production codec is qualification under test.
fn raw(document: &DispositionsDocumentV2) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(document).unwrap();
    bytes.push(b'\n');
    bytes
}

/// Actual finite codec retains every null, ordered row, counter and historical dissent.
#[test]
fn complete_recorded_output_has_exact_nulls_order_and_original_spelling() {
    let document = fixture();
    let mut ledger = ContractLedger::default();
    let mut control = Control::default();
    let first = dispositions(&document, &mut ledger, &mut control).unwrap();
    assert_eq!(first, raw(&document));
    assert!(first.ends_with(b"\n"));
    assert!(!first[..first.len() - 1].contains(&b'\n'));
    let loaded = decode_v2::decode_dispositions(&first, &mut ledger, &mut control).unwrap();
    assert!(loaded.document() == &document);
    assert_eq!(loaded.raw(), first.as_slice());
    assert_eq!(loaded.raw_sha256(), crate::hashing::sha256_hex(&first));
    let value: serde_json::Value = serde_json::from_slice(&first).unwrap();
    assert!(value["closure_generation"].is_null());
    for index in [1, 2] {
        for field in ["native_model", "native_root_uuid"] {
            assert!(value["source_pins"][index][field].is_null());
        }
    }
    for index in [0, 2] {
        assert!(value["source_pins"][index]["schema_identity"].is_null());
    }
    assert_eq!(
        value["source_pins"][0]["native_root_uuid"],
        "{ABCDEFAB-1234-5567-89AB-CDEF01234567}"
    );
    assert_eq!(value["items"][0]["dissent_ids"][0], DISSENT);
    assert_eq!(value["counts"]["response_files"], 3);
    assert_eq!(dispositions(&document, &mut ledger, &mut control).unwrap(), first);
}

/// Loaded recorded-current labels round-trip solely as ordinary closed declarations.
#[test]
fn recorded_current_label_is_not_reinterpreted_or_normalized() {
    let mut document = fixture();
    document.currentness = RecordedCurrentness::RecordedCurrent;
    document.closure_generation = Some("a".repeat(64));
    let mut ledger = ContractLedger::default();
    let mut control = Control::default();
    let output = dispositions(&document, &mut ledger, &mut control).unwrap();
    let loaded = decode_v2::decode_dispositions(&output, &mut ledger, &mut control).unwrap();
    assert!(loaded.document() == &document);
    assert_eq!(loaded.document().closure_generation.as_deref(), Some("a".repeat(64).as_str()));
}

/// Change a coherent valid operand; immutable schema/profile tokens are tested separately.
fn change(document: &mut DispositionsDocumentV2, case: usize) {
    match case {
        0 => document.queue_id = "77777777-7777-4777-8777-777777777777".into(),
        1 => document.queue_raw_sha256 = "a".repeat(64),
        2 => document.as_of = "2026-10-06T00:00:02Z".into(),
        3 => {
            document.currentness = RecordedCurrentness::RecordedCurrent;
            document.closure_generation = Some("a".repeat(64));
        }
        4 => document.source_pins[0].raw_sha256 = "a".repeat(64),
        5 => document.source_pins[0].byte_length += 1,
        6 => document.source_pins[0].native_model = Some(NativeModelV2::Profile),
        7 => {
            document.source_pins[0].native_root_uuid =
                Some("abcdefab-1234-5567-89ab-cdef01234567".into());
        }
        8 => document.source_pins[1].raw_sha256 = "a".repeat(64),
        9 => document.source_pins[1].byte_length += 1,
        10 => document.source_pins[2].raw_sha256 = "a".repeat(64),
        11 => document.source_pins[2].byte_length = 1,
        12 => document.responses[0].raw_sha256 = "a".repeat(64),
        13 => document.responses[0].byte_length += 1,
        14 => document.responses[0].responded_at = "2026-10-05T00:00:01Z".into(),
        15 => document.responses[0].reviewer_role = "other-role".into(),
        16 => {
            document.responses[0].reviewer_key = "other-reviewer".into();
            document.items[0].met_seats[0].reviewer_key = "other-reviewer".into();
        }
        17 => document.responses[1].classification = ResponseClassification::Late,
        18 => document.responses[1].disposition = Disposition::RequestChanges,
        19 => document.items[0].item_id = "77777777-7777-4777-8777-777777777777".into(),
        20 => document.items[0].reason_codes = vec!["other-reason".into()],
        21 => document.items[0].met_seats[0].role_key = "other-role".into(),
        22 => {
            document.items[0].state = ItemState::InReview;
            document.counts.states.quorum_met = 0;
            document.counts.states.in_review = 1;
            document.items[0].blocking = true;
        }
        23 => {
            document.counts.response_files += 1;
            document.counts.exact_duplicates += 1;
        }
        24 => {
            document.items[0].state = ItemState::InReview;
            document.counts.states.quorum_met = 0;
            document.counts.states.in_review = 1;
            document.items[0].required_seats = 2;
            document.items[0]
                .unmet_seats
                .push(crate::review::wire::UnmetSeat { role_key: "review".into(), ordinal: 1 });
        }
        25 => {
            let id = "77777777-7777-4777-8777-777777777777";
            document.responses.push(RecordedResponse {
                response_id: id.into(),
                raw_sha256: "7".repeat(64),
                byte_length: 170,
                item_key: "item".into(),
                reviewer_key: "reviewer-3".into(),
                reviewer_role: "review".into(),
                disposition: Disposition::Reject,
                responded_at: "2026-10-05T00:00:01Z".into(),
                classification: ResponseClassification::Stale,
            });
            document.items[0].response_ids.push(id.into());
            document.items[0].dissent_ids.push(id.into());
            document.counts.response_files += 1;
            document.counts.unique_responses += 1;
        }
        26 => {
            document.items[0].item_key = "item-2".into();
            for row in &mut document.responses {
                row.item_key = "item-2".into();
            }
        }
        27 => {
            let id = "33333333-3333-4333-8333-333333333333";
            document.responses[0].response_id = id.into();
            document.items[0].response_ids[0] = id.into();
            document.items[0].met_seats[0].response_id = id.into();
        }
        _ => panic!("unknown ordinary mutation"),
    }
}
/// A complete admitted valid readback still refuses any changed field or native spelling.
#[test]
fn full_readback_refuses_coherent_header_pin_response_witness_and_counter_changes() {
    let expected = fixture();
    for case in 0..28 {
        let mut changed = fixture();
        change(&mut changed, case);
        let raw = raw(&changed);
        let mut ledger = ContractLedger::default();
        let mut control = Control::default();
        let admitted = decode_v2::decode_dispositions(&raw, &mut ledger, &mut control).unwrap();
        assert!(admitted.document() == &changed, "case {case}");
        assert!(admitted.document() != &expected, "case {case}");
        assert_eq!(
            readback(&expected, &raw, &mut ledger, &mut control),
            Err(ContractError::Binding),
            "case {case}"
        );
        assert!(
            dispositions(&expected, &mut ledger, &mut control).is_ok(),
            "ordinary refusal latched in case {case}"
        );
    }
}

/// Closed shape and supplied order are refused rather than silently repaired.
#[test]
fn strict_readback_requires_nulls_closed_fields_and_original_array_order() {
    let expected = fixture();
    let original = raw(&expected);
    let mut missing = serde_json::to_value(&expected).unwrap();
    missing.as_object_mut().unwrap().remove("closure_generation");
    let mut extra = serde_json::to_value(&expected).unwrap();
    extra["unrecognized"] = serde_json::json!(null);
    let mut wrong_version = serde_json::to_value(&expected).unwrap();
    wrong_version["schema_version"] = serde_json::json!("forge.review-dispositions/1");
    let duplicate = format!(
        "{{\"schema_version\":\"forge.review-dispositions/2\",{}",
        std::str::from_utf8(&original[1..]).unwrap()
    )
    .into_bytes();
    let mut ledger = ContractLedger::default();
    let mut control = Control::default();
    for bytes in [
        serde_json::to_vec(&missing).unwrap(),
        serde_json::to_vec(&extra).unwrap(),
        serde_json::to_vec(&wrong_version).unwrap(),
        duplicate,
    ] {
        assert_eq!(
            readback(&expected, &bytes, &mut ledger, &mut control),
            Err(ContractError::Invalid)
        );
    }
    let mut unsorted = fixture();
    unsorted.source_pins.swap(0, 1);
    assert_eq!(dispositions(&unsorted, &mut ledger, &mut control), Err(ContractError::Invalid));
    let mut responses = fixture();
    responses.responses.swap(0, 1);
    assert_eq!(dispositions(&responses, &mut ledger, &mut control), Err(ContractError::Invalid));
    assert_eq!(dispositions(&expected, &mut ledger, &mut control).unwrap(), original);
}

/// Exercise a real token/escape writer boundary with the LF included in the cap.
#[test]
fn exact_finite_writer_limit_refuses_growth_and_preserves_first_capacity() {
    let document = "é\"\\\n";
    let mut expected = serde_json::to_vec(document).unwrap();
    expected.push(b'\n');
    let mut ledger = ContractLedger::default();
    let mut control = Control::default();
    assert_eq!(encoded(&document, expected.len(), &mut ledger, &mut control).unwrap(), expected);
    assert_eq!(
        encoded(&document, expected.len() - 1, &mut ledger, &mut control),
        Err(ContractError::Capacity)
    );
    let before = control.calls;
    control.at = Some(before + 1);
    control.failed = true;
    assert_eq!(
        encoded(&document, MAX_RECORDED_BYTES, &mut ledger, &mut control),
        Err(ContractError::Capacity)
    );
    assert_eq!(control.calls, before);
}

/// A real serde implementation fails after emitting actual tokens, with private diagnostics.
struct OrdinaryFailure;
impl Serialize for OrdinaryFailure {
    /// Preserve an ordinary serializer error through the actual finite writer and postfence.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(2))?;
        sequence.serialize_element("reached")?;
        Err(serde::ser::Error::custom("private ordinary serializer failure"))
    }
}
/// Ordinary serializer failure is unlatched; actual final caller stops take first priority.
#[test]
fn ordinary_serializer_failure_is_postfenced_on_the_same_control() {
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut control = Control::default();
        assert_eq!(
            encoded(&OrdinaryFailure, 1_000, &mut ledger, &mut control),
            Err(ContractError::Invalid)
        );
        let cost = control.calls;
        assert!(cost > 2);
        control.at = Some(control.calls + cost);
        control.failed = failed;
        assert_eq!(
            encoded(&OrdinaryFailure, 1_000, &mut ledger, &mut control),
            Err(expected_stop(failed))
        );
        let before = control.calls;
        assert_eq!(encoded(&"valid", 1_000, &mut ledger, &mut control), Err(expected_stop(failed)));
        assert_eq!(control.calls, before);
    }
    let mut ledger = ContractLedger::default();
    let mut control = Control::default();
    assert_eq!(
        encoded(&OrdinaryFailure, 1_000, &mut ledger, &mut control),
        Err(ContractError::Invalid)
    );
    assert!(encoded(&"valid", 1_000, &mut ledger, &mut control).is_ok());
}

/// A stop after all real encoded-byte readback work prevents any output from escaping.
#[test]
fn complete_codec_final_same_control_stop_is_sticky() {
    let document = fixture();
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut control = Control::default();
        let output = dispositions(&document, &mut ledger, &mut control).unwrap();
        let cost = control.calls;
        assert!(cost > 10);
        control.at = Some(control.calls + cost);
        control.failed = failed;
        assert_eq!(dispositions(&document, &mut ledger, &mut control), Err(expected_stop(failed)));
        let before = control.calls;
        assert_eq!(
            readback(&document, &output, &mut ledger, &mut control),
            Err(expected_stop(failed))
        );
        assert_eq!(dispositions(&document, &mut ledger, &mut control), Err(expected_stop(failed)));
        assert_eq!(control.calls, before);
    }
}

/// Prior same-invocation reservations remain spent; the writer cannot renew allowance.
#[test]
fn finite_writer_observes_existing_derived_reservations() {
    let mut ledger = ContractLedger::default();
    ledger.derived(MAX_RECORDED_BYTES - 7).unwrap();
    let mut control = Control::default();
    assert_eq!(encoded(&"value", 1_000, &mut ledger, &mut control), Err(ContractError::Capacity));
    let before = control.calls;
    control.at = Some(before + 1);
    assert_eq!(dispositions(&fixture(), &mut ledger, &mut control), Err(ContractError::Capacity));
    assert_eq!(control.calls, before);
}

/// Malformed/schema readback failures remain ordinary until the same actual postfence stops.
#[test]
fn ordinary_strict_readback_failure_has_unconditional_postfence() {
    let document = fixture();
    let malformed = b"{}";
    for failed in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut control = Control::default();
        assert_eq!(
            readback(&document, malformed, &mut ledger, &mut control),
            Err(ContractError::Invalid)
        );
        let cost = control.calls;
        assert!(cost >= 4);
        control.at = Some(control.calls + cost);
        control.failed = failed;
        assert_eq!(
            readback(&document, malformed, &mut ledger, &mut control),
            Err(expected_stop(failed))
        );
        let before = control.calls;
        assert_eq!(
            readback(&document, &raw(&document), &mut ledger, &mut control),
            Err(expected_stop(failed))
        );
        assert_eq!(control.calls, before);
    }
    let mut ledger = ContractLedger::default();
    let mut control = Control::default();
    assert_eq!(
        readback(&document, malformed, &mut ledger, &mut control),
        Err(ContractError::Invalid)
    );
    assert!(readback(&document, &raw(&document), &mut ledger, &mut control).is_ok());
}

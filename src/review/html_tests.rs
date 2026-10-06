//! Actual closed-decoder and writer controls; no native currentness,
//! authenticated reviewer, browser/AT acceptance or measured execution is claimed.

use super::{HtmlWriter, OUTPUT_LIMIT, render};
use crate::review::decode::{ContractError, ContractLedger, decode_dispositions, decode_response};
use crate::review::wire::IDENTITY_DISCLAIMER;
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError,
};
use serde_json::{Value, json};

/// Five distinct originals include approval, both dissent kinds and foreign evidence.
const RESPONSE_IDS: [&str; 5] = [
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
    "33333333-3333-4333-8333-333333333333",
    "44444444-4444-4444-8444-444444444444",
    "55555555-5555-4555-8555-555555555555",
];

/// Complete inert declared pins, with genuine closed model/schema spellings.
fn pins_fixture() -> Value {
    json!([
        {"artifact_key":"catalog", "model":"catalog",
         "native_root_uuid":"66666666-6666-4666-8666-666666666666",
         "raw_sha256":"6".repeat(64), "byte_length":80,
         "schema_identity":"oscal.catalog/1"},
        {"artifact_key":"mapping", "model":"mapping",
         "native_root_uuid":"77777777-7777-4777-8777-777777777777",
         "raw_sha256":"7".repeat(64), "byte_length":120,
         "schema_identity":"oscal.mapping/1"}
    ])
}

/// One minimized original summary; native eligibility/currentness remains asserted.
fn response_fixture(index: usize, item: &str, disposition: &str, classification: &str) -> Value {
    json!({
        "response_id":RESPONSE_IDS[index], "raw_sha256":index.to_string().repeat(64),
        "byte_length":100, "item_key":item,
        "reviewer_key":format!("reviewer-{index}"), "reviewer_role":"review",
        "disposition":disposition, "responded_at":"2026-10-04T11:00:00Z",
        "classification":classification
    })
}

/// Closed two-item record conserves seven files/five identities/two duplicates,
/// mixed witnesses and all historical/current dissent without a native proof.
fn document_fixture() -> Value {
    json!({
        "schema_version":"forge.review-dispositions/1",
        "identity_disclaimer":IDENTITY_DISCLAIMER,
        "queue_id":"88888888-8888-4888-8888-888888888888",
        "queue_raw_sha256":"8".repeat(64), "as_of":"2026-10-04T12:00:00Z",
        "currentness":"unverified", "closure_generation":null,
        "source_pins":pins_fixture(),
        "responses":[
            response_fixture(0,"item-a","approve","current"),
            response_fixture(1,"item-a","reject","superseded"),
            response_fixture(2,"item-a","request-changes","current"),
            response_fixture(3,"item-b","approve","current"),
            response_fixture(4,"outside-item","abstain","foreign")
        ],
        "items":[
            {"item_key":"item-a", "item_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
             "state":"changes-requested", "reason_codes":["active-dissent","rereview"],
             "required_seats":2,
             "met_seats":[{"role_key":"review","ordinal":0,"reviewer_key":"reviewer-0","response_id":RESPONSE_IDS[0]}],
             "unmet_seats":[{"role_key":"review","ordinal":1}],
             "response_ids":[RESPONSE_IDS[0],RESPONSE_IDS[1],RESPONSE_IDS[2]],
             "dissent_ids":[RESPONSE_IDS[1],RESPONSE_IDS[2]], "blocking":true},
            {"item_key":"item-b", "item_id":"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
             "state":"quorum-met", "reason_codes":["recorded-seats-filled"],
             "required_seats":1,
             "met_seats":[{"role_key":"review","ordinal":0,"reviewer_key":"reviewer-3","response_id":RESPONSE_IDS[3]}],
             "unmet_seats":[], "response_ids":[RESPONSE_IDS[3]], "dissent_ids":[],
             "blocking":false}
        ],
        "counts":{"items":2,"response_files":7,"unique_responses":5,"exact_duplicates":2,
            "states":{"unassigned":0,"assigned":0,"in_review":0,"conflicted":0,
              "changes_requested":1,"quorum_met":1,"expired":0,"stale":0}}
    })
}

/// Exercise the production closed decoder and the same ledger for the entire export.
fn rendered(value: &Value) -> String {
    let raw = serde_json::to_vec(value).unwrap();
    let mut ledger = ContractLedger::default();
    let decoded = decode_dispositions(&raw, &mut ledger, &mut NoopControl).unwrap();
    String::from_utf8(render(&decoded, &mut ledger, &mut NoopControl).unwrap()).unwrap()
}

/// The low-level text port must encode injection punctuation and preserve Unicode.
#[test]
fn text_port_escapes_markup_and_quoted_attribute_delimiters() {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut writer = HtmlWriter::new(&mut ledger, &mut control);
    writer.text("é<script x=\"a&b\" onload='bad'>λ</script>").unwrap();
    assert_eq!(
        String::from_utf8(writer.finish().unwrap()).unwrap(),
        "é&lt;script x=&quot;a&amp;b&quot; onload=&#39;bad&#39;&gt;λ&lt;/script&gt;"
    );
}

/// Conservation is checked against complete independent fixture IDs/pins/counters,
/// not merely successful HTML construction or an approving recorded state.
#[test]
fn every_recorded_item_response_pin_witness_and_dissent_is_visible() {
    let value = document_fixture();
    let html = rendered(&value);
    assert_eq!(html.matches("data-row=\"item\"").count(), 2);
    assert_eq!(html.matches("data-row=\"response\"").count(), 5);
    assert_eq!(html.matches("data-row=\"source\"").count(), 2);
    for id in RESPONSE_IDS {
        assert!(html.contains(id));
    }
    for id in [RESPONSE_IDS[1], RESPONSE_IDS[2]] {
        assert_eq!(html.matches(id).count(), 3, "response + item reference + dissent reference");
    }
    for (label, count) in [
        ("Items", 2),
        ("Original response files", 7),
        ("Unique responses", 5),
        ("Additional exact duplicate files", 2),
        ("Source pins", 2),
    ] {
        assert!(html.contains(&format!("<th scope=\"row\">{label}</th><td>{count}</td>")));
    }
    for key in [
        "item-a",
        "item-b",
        "outside-item",
        "catalog",
        "mapping",
        "active-dissent",
        "rereview",
        "recorded-seats-filled",
        "reviewer-0",
        "reviewer-3",
    ] {
        assert!(html.contains(key));
    }
    for hash in ["6".repeat(64), "7".repeat(64), "8".repeat(64)] {
        assert!(html.contains(&hash));
    }
    for row in value["responses"].as_array().unwrap() {
        assert!(html.contains(row["raw_sha256"].as_str().unwrap()));
        assert!(html.contains(row["responded_at"].as_str().unwrap()));
    }
    assert!(html.contains("Role review; ordinal 1"));
    assert!(html.contains("Recorded blocking: true"));
    assert!(html.contains("<th scope=\"row\">stale</th><td>0</td>"));
}

/// Even a recorded-current/quorum-met claim is visibly recorded, never a fresh proof.
#[test]
fn recorded_labels_do_not_assert_fresh_status_or_domain_approval() {
    let mut value = document_fixture();
    value["currentness"] = json!("recorded-current");
    value["closure_generation"] = json!("9".repeat(64));
    let html = rendered(&value);
    for required in [
        "Recorded as_of (asserted UTC)",
        "2026-10-04T12:00:00Z",
        "recorded-current (not freshly verified by this export)",
        "Fresh source capture and complete currentness checks are required to assert current status.",
        "Review quorum is a declared review-policy result, not domain approval or a native state transition.",
        "Keys, roles, authors and timestamps are asserted, not authenticated, signed or non-repudiable.",
    ] {
        assert!(html.contains(required));
    }
    assert!(html.contains(&"9".repeat(64)));
}

/// A genuinely decoded private response retains canaries; only its minimized
/// original identity summary enters the independently decoded recorded bundle.
#[test]
fn private_response_rationale_names_paths_and_excerpts_do_not_escape() {
    let mut value = document_fixture();
    let private = json!({
        "schema_version":"forge.review-response/1", "identity_disclaimer":IDENTITY_DISCLAIMER,
        "response_id":RESPONSE_IDS[0], "queue_id":value["queue_id"],
        "queue_raw_sha256":value["queue_raw_sha256"], "item_key":"item-a",
        "item_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "domain":"mapping-assertion",
        "adapter_version":"forge.mapping-review/1", "requested_action":"re-review",
        "source_pins":pins_fixture(), "subject_sha256":"a".repeat(64),
        "context_sha256":"b".repeat(64), "policy_sha256":"c".repeat(64),
        "reviewer_key":"reviewer-0", "reviewer_role":"review", "disposition":"approve",
        "responded_at":"2026-10-04T11:00:00Z",
        "rationale":"PRIVATE-RATIONALE AlicePrivate /private/source/secret.md SOURCE-EXCERPT <script>PRIVATE-EVENT</script>",
        "abstention_reason":null, "proposed_edit":null, "supersedes":null
    });
    let private_raw = serde_json::to_vec(&private).unwrap();
    let mut ledger = ContractLedger::default();
    let private_decoded = decode_response(&private_raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(private_decoded.document().rationale.contains("PRIVATE-RATIONALE"));
    value["responses"][0]["raw_sha256"] = json!(private_decoded.raw_sha256());
    value["responses"][0]["byte_length"] = json!(private_raw.len());
    let raw = serde_json::to_vec(&value).unwrap();
    let decoded = decode_dispositions(&raw, &mut ledger, &mut NoopControl).unwrap();
    let html = String::from_utf8(render(&decoded, &mut ledger, &mut NoopControl).unwrap()).unwrap();
    assert!(html.contains(private_decoded.raw_sha256()));
    for secret in [
        "PRIVATE-RATIONALE",
        "AlicePrivate",
        "/private/source/secret.md",
        "SOURCE-EXCERPT",
        "PRIVATE-EVENT",
    ] {
        assert!(!html.contains(secret));
    }
    for forbidden in ["<script", "<form", " onload=", " onclick=", "src=", "http://", "https://"] {
        assert!(!html.contains(forbidden));
    }
}

/// Structural decoder refusal precedes HTML; there is no permissive rendering path.
#[test]
fn private_or_incomplete_recorded_shapes_cannot_be_rendered() {
    let mut value = document_fixture();
    value["responses"][0]["rationale"] = json!("PRIVATE-RATIONALE");
    let raw = serde_json::to_vec(&value).unwrap();
    assert!(matches!(
        decode_dispositions(&raw, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
    let mut value = document_fixture();
    value["items"][0]["dissent_ids"] = json!([RESPONSE_IDS[2]]);
    let raw = serde_json::to_vec(&value).unwrap();
    assert!(matches!(
        decode_dispositions(&raw, &mut ContractLedger::default(), &mut NoopControl),
        Err(ContractError::Invalid)
    ));
}

/// The inert document provides semantic headings, local navigation, complete table
/// captions/headers and constant focus/reflow/print CSS without active content.
#[test]
fn semantic_static_shell_uses_only_constant_local_navigation() {
    let html = rendered(&document_fixture());
    assert_eq!(html.matches("<h1>").count(), 1);
    assert_eq!(html.matches("<h2>").count(), 4);
    assert_eq!(html.matches("<caption>").count(), 5);
    assert!(html.contains("scope=\"col\""));
    assert!(html.contains("scope=\"row\""));
    assert!(html.contains("lang=\"en\""));
    for anchor in ["summary", "sources", "items", "responses"] {
        assert!(html.contains(&format!("href=\"#{anchor}\"")));
        assert!(html.contains(&format!("id=\"{anchor}\"")));
    }
    for style in
        [":focus-visible", "overflow-wrap:anywhere", "@media print", "@media(max-width:40rem)"]
    {
        assert!(html.contains(style));
    }
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("<input"));
    assert!(!html.contains("<button"));
}

/// Write actual payload bytes through the production writer up to a chosen boundary.
fn fill_to(writer: &mut HtmlWriter<'_>, length: usize) {
    let block = vec![b'x'; 65_536].into_boxed_slice();
    while length - writer.output.len() >= block.len() {
        writer.raw(&block).unwrap();
    }
    let remaining = length - writer.output.len();
    writer.raw(&block[..remaining]).unwrap();
}

/// A five-byte escaped ampersand reaches exactly 32 MiB; the next encoded byte
/// refuses before growth and the same ledger remains stopped on a fresh writer.
#[test]
fn actual_encoded_payload_boundary_refuses_before_growth_and_latches() {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    {
        let mut writer = HtmlWriter::new(&mut ledger, &mut control);
        fill_to(&mut writer, OUTPUT_LIMIT - 5);
        writer.text("&").unwrap();
        assert_eq!(writer.output.len(), OUTPUT_LIMIT);
        assert_eq!(&writer.output[OUTPUT_LIMIT - 5..], b"&amp;");
        assert_eq!(writer.raw(b"x"), Err(ContractError::Capacity));
        assert_eq!(writer.output.len(), OUTPUT_LIMIT);
    }
    let mut next = HtmlWriter::new(&mut ledger, &mut control);
    assert_eq!(next.raw(b""), Err(ContractError::Capacity));
    assert_eq!(next.output, [] as [u8; 0]);
}

/// The escaped extent, rather than unescaped character length, drives whole refusal.
#[test]
fn encoded_expansion_exceeds_limit_without_appending_partial_entity() {
    let mut ledger = ContractLedger::default();
    let mut control = NoopControl;
    let mut writer = HtmlWriter::new(&mut ledger, &mut control);
    fill_to(&mut writer, OUTPUT_LIMIT - 4);
    assert_eq!(writer.text("&"), Err(ContractError::Capacity));
    assert_eq!(writer.output.len(), OUTPUT_LIMIT - 4);
}

/// Previously retained command payload consumes the same cap; a writer cannot
/// reset it or acquire an independent 32 MiB allowance.
#[test]
fn prior_logical_reservation_limits_output_in_the_same_ledger() {
    let mut ledger = ContractLedger::default();
    ledger.derived(64).unwrap();
    let mut control = NoopControl;
    let mut writer = HtmlWriter::new(&mut ledger, &mut control);
    fill_to(&mut writer, OUTPUT_LIMIT - 64);
    assert_eq!(writer.raw(b"x"), Err(ContractError::Capacity));
    assert_eq!(writer.output.len(), OUTPUT_LIMIT - 64);
}

/// Count actual renderer checkpoints and inject one exact typed stop at a selected call.
#[derive(Default)]
struct CheckpointRecorder {
    /// Actual reached checkpoints, not inferred function calls.
    seen: usize,
    /// Optional one-based interruption occurrence.
    stop_at: Option<usize>,
    /// Sticky caller reason once returned.
    stopped: Option<Interruption>,
}

impl WorkControl for CheckpointRecorder {
    /// Stop only at the selected reached occurrence; later calls retain the reason.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        self.seen += 1;
        if self.stop_at == Some(self.seen) {
            self.stopped = Some(Interruption::DeadlineExceeded);
        }
        match self.stopped {
            Some(reason) => Err(WorkError::Interrupted(reason)),
            None => Ok(()),
        }
    }
    /// Borrow the actual first injected caller interruption.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}

/// Actual reached middle/final checkpoints refuse the entire export and retain
/// the exact typed deadline across a later attempted render with no-op control.
#[test]
fn reached_middle_and_final_control_stops_never_return_partial_html() {
    let raw = serde_json::to_vec(&document_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    let decoded = decode_dispositions(&raw, &mut ledger, &mut NoopControl).unwrap();
    let mut observer = CheckpointRecorder::default();
    let expected = render(&decoded, &mut ledger, &mut observer).unwrap();
    assert!(observer.seen > 20);
    for stop_at in [observer.seen / 2, observer.seen] {
        let mut ledger = ContractLedger::default();
        let decoded = decode_dispositions(&raw, &mut ledger, &mut NoopControl).unwrap();
        let mut control =
            CheckpointRecorder { stop_at: Some(stop_at), ..CheckpointRecorder::default() };
        assert!(matches!(
            render(&decoded, &mut ledger, &mut control),
            Err(ContractError::Interrupted(Interruption::DeadlineExceeded))
        ));
        assert_eq!(control.seen, stop_at);
        assert!(matches!(
            render(&decoded, &mut ledger, &mut NoopControl),
            Err(ContractError::Interrupted(Interruption::DeadlineExceeded))
        ));
    }
    assert!(expected.ends_with(b"</body></html>\n"));
}

/// An ordinary caller failure remains distinct from cancellation and blocks reuse.
struct FailedControl;
impl WorkControl for FailedControl {
    /// Return the maintained safe ordinary error without carrying any raw error prose.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> Result<(), WorkError> {
        Err(WorkError::Failed(crate::workspace::contract::Error::invalid()))
    }
    /// An ordinary failure manufactures no interruption reason.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// Ordinary failures cannot produce an empty success or turn into a cancellation.
#[test]
fn ordinary_control_failure_is_sticky_and_precise() {
    let raw = serde_json::to_vec(&document_fixture()).unwrap();
    let mut ledger = ContractLedger::default();
    let decoded = decode_dispositions(&raw, &mut ledger, &mut NoopControl).unwrap();
    assert!(matches!(
        render(&decoded, &mut ledger, &mut FailedControl),
        Err(ContractError::ControlFailed)
    ));
    assert!(matches!(
        render(&decoded, &mut ledger, &mut NoopControl),
        Err(ContractError::ControlFailed)
    ));
}

/// Repeated exports preserve original pins/order and have no clock/random/IO inputs.
#[test]
fn same_recorded_original_renders_identical_complete_bytes() {
    let value = document_fixture();
    assert_eq!(rendered(&value), rendered(&value));
    let raw = serde_json::to_vec(&value).unwrap();
    let html = rendered(&value);
    assert!(html.contains(&crate::hashing::sha256_hex(&raw)));
}

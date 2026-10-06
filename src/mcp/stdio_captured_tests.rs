//! Proposed genuine file-backed query controls through the actual modern worker.
//!
//! Synthetic declared-owner fixtures exercise production capture/currentness and
//! schema consumers; they never establish real human or interoperability acceptance.
//! Final-phase controls use the real private publication ports after actual preparation,
//! not a detached successful response or a new production callback.

use super::super::super::queries::tests::StdioFixture;
use super::*;
use std::fs;

/// A wire chunk or EOF becomes ready only after actual previously written responses.
enum CapturedStep {
    /// Complete original request bytes; delivery may split at the real intake bound.
    Bytes {
        /// Complete actual response count required before this request becomes ready.
        after: usize,
        /// Exact original request record, never an injected query result.
        bytes: Vec<u8>,
    },
    /// Keep EOF unavailable until the worker has actually published the indicated count.
    Eof {
        /// Complete actual response count required before input EOF becomes visible.
        after: usize,
    },
}

/// One armed final input observation, never an injected domain result or approval proof.
enum FinalAction {
    /// Allow the final observation without changing an original or protocol state.
    Idle,
    /// Modify one actual fixture original at the final pre-proof input observation.
    Drift(PathBuf),
    /// Deliver real cancellation JSON through the same parser and owned Machine.
    Cancel(u64),
    /// Deliver actual input-port EOF after real captured preparation.
    Eof,
}

/// Finite fixture input uses actual output bytes as readiness barriers.
struct CapturedInput {
    /// Ordered actual requests, gated by complete LF records actually written.
    steps: VecDeque<CapturedStep>,
    /// Real writer's retained output, observed without injecting or modifying it.
    written: Rc<RefCell<Vec<u8>>>,
    /// Optional test action armed only after an actual `PreparedQuery` exists.
    action: Rc<RefCell<Option<FinalAction>>>,
    /// Count actual input observations, also bounding a defective fixture campaign.
    calls: Rc<Cell<usize>>,
    /// Whether the explicitly armed action actually reached the consumed input port.
    acted: Rc<Cell<bool>>,
}

impl InputPort for CapturedInput {
    /// Consume only bounded ready bytes; no clock, sleep, thread or project-proof override.
    fn read_ready(&mut self, _wait: Duration, destination: &mut [u8]) -> io::Result<InputRead> {
        let calls = self.calls.get().checked_add(1).expect("bounded fixture calls");
        self.calls.set(calls);
        if calls > 100_000 {
            return Err(io::Error::other("finite captured fixture observation limit"));
        }
        if let Some(action) = self.action.borrow_mut().take() {
            self.acted.set(true);
            return match action {
                FinalAction::Idle => Ok(InputRead::Idle),
                FinalAction::Eof => Ok(InputRead::Eof),
                FinalAction::Drift(path) => {
                    let mut bytes = fs::read(&path)?;
                    let first = bytes.first_mut().expect("nonempty actual original fixture");
                    *first = if *first == b'Q' { b'P' } else { b'Q' };
                    fs::write(path, bytes)?;
                    Ok(InputRead::Idle)
                }
                FinalAction::Cancel(id) => {
                    let bytes = cancel(json!(id));
                    assert!(bytes.len() <= destination.len());
                    destination[..bytes.len()].copy_from_slice(&bytes);
                    Ok(InputRead::Data(bytes.len()))
                }
            };
        }
        let responses = self
            .written
            .borrow()
            .iter()
            .fold(0usize, |count, byte| count + usize::from(*byte == b'\n'));
        let Some(next) = self.steps.front() else {
            return Ok(InputRead::Idle);
        };
        let threshold = match next {
            CapturedStep::Bytes { after, .. } | CapturedStep::Eof { after } => *after,
        };
        if responses < threshold {
            return Ok(InputRead::Idle);
        }
        match self.steps.pop_front().expect("observed fixture step") {
            CapturedStep::Eof { .. } => Ok(InputRead::Eof),
            CapturedStep::Bytes { after, bytes } => {
                let count = bytes.len().min(destination.len());
                destination[..count].copy_from_slice(&bytes[..count]);
                if count < bytes.len() {
                    self.steps
                        .push_front(CapturedStep::Bytes { after, bytes: bytes[count..].to_vec() });
                }
                Ok(InputRead::Data(count))
            }
        }
    }
}

/// Run the complete consumed wire/gate/query/helper/publication path over actual originals.
fn captured_worker(
    fixture: &StdioFixture,
    steps: Vec<CapturedStep>,
) -> (Result<(), RunError>, Vec<u8>) {
    let bytes = Rc::new(RefCell::new(Vec::new()));
    let input = CapturedInput {
        steps: steps.into(),
        written: Rc::clone(&bytes),
        action: Rc::new(RefCell::new(None)),
        calls: Rc::new(Cell::new(0)),
        acted: Rc::new(Cell::new(false)),
    };
    let writer = ObservedWriter { bytes: Rc::clone(&bytes), fail_after: None, fail_flush: false };
    let result = run(&fixture.startup(), input, writer);
    let actual = bytes.borrow().clone();
    (result, actual)
}

/// Seven exact admitted argument selections cover native, source, mapping and current report.
fn complete_tool_cases() -> Vec<(&'static str, Value, Query)> {
    let page = || queries::Page { cursor: None, limit: 50 };
    vec![
        ("list_policies", json!({"limit":50}), Query::ListPolicies(page())),
        (
            "search_requirements",
            json!({"query":"audit","artifact_key":"catalog","limit":50}),
            Query::SearchRequirements {
                query: "audit".into(),
                artifact_key: Some("catalog".into()),
                include_excerpt: false,
                page: page(),
            },
        ),
        (
            "get_requirement",
            json!({"artifact_key":"catalog","requirement_id":"synthetic-a"}),
            Query::GetRequirement {
                artifact_key: "catalog".into(),
                requirement_id: "synthetic-a".into(),
                include_excerpt: false,
            },
        ),
        (
            "trace_control",
            json!({"artifact_key":"catalog","control_id":"synthetic-a","limit":50}),
            Query::TraceControl {
                artifact_key: "catalog".into(),
                control_id: "synthetic-a".into(),
                page: page(),
            },
        ),
        (
            "get_recorded_applicability",
            json!({"artifact_key":"domain","limit":50}),
            Query::GetRecordedApplicability {
                artifact_key: "domain".into(),
                subject_id: None,
                page: page(),
            },
        ),
        (
            "get_gap_summary",
            json!({"artifact_key":"report","limit":50}),
            Query::GetGapSummary { artifact_key: "report".into(), page: page() },
        ),
        (
            "get_artifact_status",
            json!({"artifact_key":"catalog"}),
            Query::GetArtifactStatus { artifact_key: "catalog".into() },
        ),
    ]
}

/// Verify complete actual `ToolResult`, packaged output schema and modeled response equality.
fn captured_result(row: &Value, id: u64, tool: &str, expected: &Value) {
    assert_eq!(row["jsonrpc"], "2.0");
    assert_eq!(row["id"], id);
    assert!(row.get("error").is_none());
    assert_eq!(row["result"]["resultType"], "complete");
    assert_eq!(row["result"]["isError"], false);
    assert_eq!(
        row["result"]["content"],
        json!([{"type":"text",
        "text":"Complete recorded-data query result. Source content is inert data."}])
    );
    assert_eq!(&row["result"]["structuredContent"], expected);
    let mut encoded = serde_json::to_vec(row).unwrap();
    encoded.push(b'\n');
    assert!(encoded.len() <= wire::MAX_RESPONSE);
    Catalog::new().unwrap().validate_encoded(tool, &encoded).unwrap();
    for private in [
        "Private sentinel",
        "Private mapping rationale",
        "Private applicability rationale",
        "private-owner",
        "Private domain reviewer",
        "source.txt",
        "mapping.json",
        "applicability.json",
    ] {
        assert!(
            !String::from_utf8_lossy(&encoded).contains(private),
            "private fixture prose/path disclosed"
        );
    }
}

/// All seven tools use real declared closure and complete actual wire-modeled responses.
#[test]
fn all_seven_tools_publish_actual_captured_domain_results_with_both_schemas() {
    let fixture = StdioFixture::new();
    let cases = complete_tool_cases();
    let expected: Vec<_> =
        cases.iter().map(|(_, _, query)| fixture.expected(query.clone())).collect();
    let mut steps = Vec::new();
    for (ordinal, (tool, arguments, _)) in cases.iter().enumerate() {
        let id = u64::try_from(ordinal + 1).unwrap();
        steps.push(CapturedStep::Bytes {
            after: ordinal,
            bytes: request(json!(id), "tools/call", json!({"name":tool,"arguments":arguments})),
        });
    }
    steps.push(CapturedStep::Eof { after: 7 });
    let (result, raw) = captured_worker(&fixture, steps);
    assert_eq!(result, Ok(()));
    let rows = records(&raw);
    assert_eq!(rows.len(), 7);
    for (ordinal, ((tool, _, _), expected)) in cases.iter().zip(&expected).enumerate() {
        captured_result(&rows[ordinal], u64::try_from(ordinal + 1).unwrap(), tool, expected);
        let mut malformed = rows[ordinal].clone();
        malformed["result"]["structuredContent"]["unexpected"] = json!(true);
        let encoded = serde_json::to_vec(&malformed).unwrap();
        assert!(Catalog::new().unwrap().validate_encoded(tool, &encoded).is_err());
    }
    assert_eq!(rows[0]["result"]["structuredContent"]["data"]["matched"], 1);
    assert_eq!(rows[1]["result"]["structuredContent"]["data"]["matched"], 2);
    assert_eq!(rows[2]["result"]["structuredContent"]["data"]["requirement_id"], "synthetic-a");
    let trace = &rows[3]["result"]["structuredContent"]["data"];
    assert_eq!(trace["matched"], 4);
    for relation in &trace["rows"].as_array().unwrap()[1..] {
        assert_eq!(relation["mapping_id"], fixture.mapping_id());
    }
    assert_eq!(rows[4]["result"]["structuredContent"]["data"]["matched"], 2);
    assert_eq!(&rows[5]["result"]["structuredContent"]["data"]["summary"], fixture.report_counts());
    assert_eq!(rows[6]["result"]["structuredContent"]["data"]["lifecycle_state"], "approved");
    assert_eq!(rows[6]["result"]["structuredContent"]["data"]["schedule"], Value::Null);
}

/// Every closed input schema refuses extras while the next valid genuine request still succeeds.
#[test]
fn seven_input_schemas_refuse_extra_arguments_without_poisoning_next_capture() {
    let fixture = StdioFixture::new();
    let cases = complete_tool_cases();
    let expected: Vec<_> =
        cases.iter().map(|(_, _, query)| fixture.expected(query.clone())).collect();
    let mut steps = Vec::new();
    for (ordinal, (tool, arguments, _)) in cases.iter().enumerate() {
        let mut invalid = arguments.clone();
        invalid["unexpected"] = json!("private invalid argument must not echo");
        for (offset, value) in [(0, invalid), (1, arguments.clone())] {
            steps.push(CapturedStep::Bytes {
                after: ordinal * 2 + offset,
                bytes: request(
                    json!(ordinal * 2 + offset + 1),
                    "tools/call",
                    json!({"name":tool,"arguments":value}),
                ),
            });
        }
    }
    steps.push(CapturedStep::Eof { after: 14 });
    let (result, raw) = captured_worker(&fixture, steps);
    assert_eq!(result, Ok(()));
    let rows = records(&raw);
    assert_eq!(rows.len(), 14);
    for (ordinal, ((tool, _, _), expected)) in cases.iter().zip(&expected).enumerate() {
        assert_eq!(rows[ordinal * 2]["error"]["code"], -32602);
        assert!(rows[ordinal * 2].get("result").is_none());
        captured_result(
            &rows[ordinal * 2 + 1],
            u64::try_from(ordinal * 2 + 2).unwrap(),
            tool,
            expected,
        );
    }
    assert!(!String::from_utf8_lossy(&raw).contains("private invalid argument"));
}

/// Actual final-phase observations after complete gate/domain preparation have no detached data.
struct FinalObservation {
    /// Bytes actually accepted by the same consumed writer/publication ports.
    raw: Vec<u8>,
    /// Typed first stop observed by the actual runtime control, if present.
    stop: Option<Interruption>,
    /// Actual pending queue size after the real cancellation/parser observation.
    queued: usize,
    /// Whether the explicitly armed actual `InputPort` action reached the final observation.
    acted: bool,
    /// Intake observations during quiescent proof/write; these must remain absent.
    quiescent_reads: usize,
}

/// Prepare a real query with one control, then exercise the consumed final publication seam.
/// This deliberately targets the private final phase rather than adding a production callback.
fn captured_final_phase(
    fixture: &StdioFixture,
    action: FinalAction,
    with_queued: bool,
) -> FinalObservation {
    let catalog = Catalog::new().unwrap();
    let clock = ManualClock { value: Cell::new(Instant::now()) };
    let raw = Rc::new(RefCell::new(Vec::new()));
    let armed = Rc::new(RefCell::new(None));
    let acted = Rc::new(Cell::new(false));
    let calls = Rc::new(Cell::new(0));
    let mut initial = request(
        json!(1),
        "tools/call",
        json!({"name":"get_gap_summary","arguments":{"artifact_key":"report","limit":50}}),
    );
    if with_queued {
        initial.extend(request(json!(2), "tools/call",
            json!({"name":"get_requirement","arguments":{"artifact_key":"catalog","requirement_id":"synthetic-a"}})));
    }
    let input = CapturedInput {
        steps: vec![CapturedStep::Bytes { after: 0, bytes: initial }].into(),
        written: Rc::clone(&raw),
        action: Rc::clone(&armed),
        calls: Rc::clone(&calls),
        acted: Rc::clone(&acted),
    };
    let output = ObservedWriter { bytes: Rc::clone(&raw), fail_after: None, fail_flush: false };
    let mut pump = Pump {
        input,
        output,
        catalog: &catalog,
        clock: &clock,
        machine: Machine::default(),
        framer: Framer::default(),
        intake: [0; READ_BURST],
        start: 0,
        end: 0,
    };
    pump.service(Duration::ZERO).unwrap();
    let accepted = pump.machine.next().unwrap();
    let id = accepted.id;
    let Operation::Call(query) = accepted.operation else {
        panic!("admitted genuine tool call");
    };
    let startup = fixture.startup();
    let mut control = Control { pump: &mut pump, io_failure: None, quiescent: false };
    let gate = disclosure::load_disclosure_decision(
        &startup.project_root,
        startup.decision_root.as_deref(),
        startup.decision_sha256.as_deref(),
        &startup.profile_sha256,
        &mut control,
    )
    .unwrap();
    let DisclosureGate::Validated(scope) = gate else {
        panic!("genuine complete fixture gate");
    };
    let prepared = queries::prepare(&scope, query.clone(), &mut control)
        .unwrap_or_else(|_| panic!("genuine captured domain preparation before final action"));
    let actual = serde_json::to_value(prepared.response()).unwrap();
    assert_eq!(actual["availability"], "available");
    assert_eq!(&actual["data"]["summary"], fixture.report_counts());
    let encoded = wire::success(&id, &ToolResult::new(prepared.response(), false)).unwrap();
    catalog.validate_encoded(query.tool(), &encoded).unwrap();
    assert!(raw.borrow().is_empty());
    *armed.borrow_mut() = Some(action);
    let entered = control.enter_publication();
    let before_quiescent = calls.get();
    let fenced = entered.and_then(|()| prepared.verify_inputs(&mut control));
    let after_quiescent = calls.get();
    match fenced {
        Ok(()) => {
            assert_eq!(control.publish(&encoded).unwrap(), None);
        }
        Err(error) => {
            control.quiescent = false;
            stopped(&catalog, &id, &query, error, &mut control).unwrap();
        }
    }
    let stop = control.interruption();
    let queued = control.pump.machine.queued.len();
    assert!(control.io_failure.is_none());
    // Actual borrowed prepared scope unwinds before retiring the owned active ID.
    drop(prepared);
    drop(scope);
    control.pump.machine.finish(&id);
    let bytes = raw.borrow().clone();
    FinalObservation {
        raw: bytes,
        stop,
        queued,
        acted: acted.get(),
        quiescent_reads: after_quiescent - before_quiescent,
    }
}

/// A genuine prepared domain result fences all originals without another intake parser before write.
#[test]
fn actual_prepared_domain_final_proof_publishes_complete_without_intervening_intake() {
    let fixture = StdioFixture::new();
    let expected = fixture.expected(Query::GetGapSummary {
        artifact_key: "report".into(),
        page: queries::Page { cursor: None, limit: 50 },
    });
    let observation = captured_final_phase(&fixture, FinalAction::Idle, false);
    assert!(observation.acted);
    assert_eq!(observation.stop, None);
    assert_eq!(observation.quiescent_reads, 0);
    let rows = records(&observation.raw);
    assert_eq!(rows.len(), 1);
    captured_result(&rows[0], 1, "get_gap_summary", &expected);
}

/// Same-size actual source or approved domain drift after preparation blocks all available data.
#[test]
fn final_input_observation_real_original_drift_refuses_completed_domain_publication() {
    for domain in [false, true] {
        let fixture = StdioFixture::new();
        let path = if domain { fixture.domain_path() } else { fixture.source_path() };
        let before = fs::read(&path).unwrap();
        let observation = captured_final_phase(&fixture, FinalAction::Drift(path.clone()), false);
        let after = fs::read(&path).unwrap();
        assert_eq!(before.len(), after.len());
        assert_ne!(before, after);
        assert!(observation.acted);
        assert_eq!(observation.quiescent_reads, 0);
        let rows = records(&observation.raw);
        assert_eq!(rows.len(), 1);
        let response = &rows[0]["result"]["structuredContent"];
        assert_eq!(rows[0]["result"]["isError"], true);
        assert_eq!(response["availability"], "unavailable");
        assert_eq!(response["reason"], "approval-unavailable");
        assert_eq!(response["data"], Value::Null);
        Catalog::new().unwrap().validate_encoded("get_gap_summary", &observation.raw).unwrap();
        assert!(!String::from_utf8_lossy(&observation.raw).contains("Private"));
    }
}

/// Known active cancellation after actual captured preparation suppresses any terminal response.
#[test]
fn active_cancel_at_real_prepared_domain_final_fence_publishes_nothing() {
    let fixture = StdioFixture::new();
    let observation = captured_final_phase(&fixture, FinalAction::Cancel(1), false);
    assert!(observation.acted);
    assert_eq!(observation.stop, Some(Interruption::CancelRequested));
    assert_eq!(observation.raw, Vec::<u8>::new());
    assert_eq!(observation.queued, 0);
}

/// Real queued cancellation removes its admitted request without suppressing the active result.
#[test]
fn queued_cancel_after_actual_capture_keeps_active_result_and_removes_waiter() {
    let fixture = StdioFixture::new();
    let expected = fixture.expected(Query::GetGapSummary {
        artifact_key: "report".into(),
        page: queries::Page { cursor: None, limit: 50 },
    });
    let observation = captured_final_phase(&fixture, FinalAction::Cancel(2), true);
    assert!(observation.acted);
    assert_eq!(observation.stop, None);
    assert_eq!(observation.queued, 0);
    let rows = records(&observation.raw);
    assert_eq!(rows.len(), 1);
    captured_result(&rows[0], 1, "get_gap_summary", &expected);
}

/// Actual EOF after captured preparation discards trusted work without claiming successful response.
#[test]
fn eof_at_real_prepared_domain_final_fence_publishes_nothing() {
    let fixture = StdioFixture::new();
    let observation = captured_final_phase(&fixture, FinalAction::Eof, true);
    assert!(observation.acted);
    assert_eq!(observation.stop, Some(Interruption::Shutdown));
    assert_eq!(observation.queued, 0);
    assert_eq!(observation.raw, Vec::<u8>::new());
}

/// A duplicate arriving only after a genuine completed response retires without a second terminal.
#[test]
fn duplicate_after_genuine_complete_domain_response_never_publishes_competing_result() {
    let fixture = StdioFixture::new();
    let expected = fixture.expected(Query::GetArtifactStatus { artifact_key: "catalog".into() });
    let bytes = request(
        json!(7),
        "tools/call",
        json!({"name":"get_artifact_status","arguments":{"artifact_key":"catalog"}}),
    );
    let (result, raw) = captured_worker(
        &fixture,
        vec![
            CapturedStep::Bytes { after: 0, bytes: bytes.clone() },
            CapturedStep::Bytes { after: 1, bytes },
            CapturedStep::Eof { after: 1 },
        ],
    );
    assert_eq!(result, Err(RunError::DuplicateId));
    let rows = records(&raw);
    assert_eq!(rows.len(), 1);
    captured_result(&rows[0], 7, "get_artifact_status", &expected);
}

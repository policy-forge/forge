//! Deterministic wire/worker regressions; no fixture creates approval authority.
//! Native input controls and captured-query fixtures have separate scopes.
//! Scripted port results provide no real MCP client or platform acceptance.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use serde_json::{Value, json};

use super::*;

/// Controlled monotonic observation consumes the actual runtime clock port.
struct ManualClock {
    /// Current deterministic monotonic instant, with no wall-clock/date authority.
    value: Cell<Instant>,
}
impl Clock for ManualClock {
    /// Return the exact controlled monotonic instant without changing a deadline.
    fn now(&self) -> Instant {
        self.value.get()
    }
}

/// Bounded scripted input observations, not injected captured project/domain states.
enum Event {
    /// Exact actual wire bytes delivered in bounded chunks.
    Bytes(Vec<u8>),
    /// Ready input is absent for this one fence.
    Idle,
    /// Actual scripted EOF through the consumed input port.
    Eof,
    /// Move the real injected monotonic port without sleeps or environment mutation.
    Advance(Duration),
}

/// Scripted input port tests real framing/admission/dispatch/control code.
struct ScriptedInput {
    /// Complete finite fixture events; no receiver thread exists.
    events: VecDeque<Event>,
    /// Same clock observed by real acceptance and later `WorkControl` fences.
    clock: Rc<ManualClock>,
    /// Actual requested input waits, retained only by this synthetic fixture.
    waits: Rc<RefCell<Vec<Duration>>>,
}
impl InputPort for ScriptedInput {
    /// Deliver one bounded chunk or an explicit idle/EOF/time observation.
    fn read_ready(&mut self, wait: Duration, destination: &mut [u8]) -> io::Result<InputRead> {
        self.waits.borrow_mut().push(wait);
        match self.events.pop_front().unwrap_or(Event::Eof) {
            Event::Idle => Ok(InputRead::Idle),
            Event::Eof => Ok(InputRead::Eof),
            Event::Advance(delta) => {
                self.clock
                    .value
                    .set(self.clock.now().checked_add(delta).expect("bounded fixture time"));
                Ok(InputRead::Idle)
            }
            Event::Bytes(bytes) => {
                let count = bytes.len().min(destination.len());
                destination[..count].copy_from_slice(&bytes[..count]);
                if count < bytes.len() {
                    self.events.push_front(Event::Bytes(bytes[count..].to_vec()));
                }
                Ok(InputRead::Data(count))
            }
        }
    }
}

/// Complete protocol bytes retained by a controlled writer, including failure prefixes.
#[derive(Clone)]
struct ObservedWriter {
    /// Actual bytes accepted by the real runtime writer calls.
    bytes: Rc<RefCell<Vec<u8>>>,
    /// Optional first absolute byte at which actual write must fail.
    fail_after: Option<usize>,
    /// Actual flush fault after all preceding accepted bytes.
    fail_flush: bool,
}
impl Write for ObservedWriter {
    /// Preserve actual bounded partial write semantics instead of claiming retraction.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut retained = self.bytes.borrow_mut();
        let count = if let Some(limit) = self.fail_after {
            if retained.len() >= limit {
                return Err(io::Error::other("private fixture write fault"));
            }
            bytes.len().min(limit - retained.len())
        } else {
            bytes.len()
        };
        retained.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    /// Return a real controlled flush fault without deleting already-written bytes.
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush { Err(io::Error::other("private fixture flush fault")) } else { Ok(()) }
    }
}

/// Return a static-only startup selection whose private project path must never matter.
fn startup() -> Startup {
    Startup {
        project_root: PathBuf::from("unavailable-private-project"),
        decision_root: None,
        decision_sha256: None,
        profile_sha256: "invalid-but-unused-without-pair".to_owned(),
    }
}

/// Exact modern mandatory per-request metadata with a bounded inert extension.
fn meta() -> Value {
    json!({"io.modelcontextprotocol/protocolVersion": wire::REVISION,
        "io.modelcontextprotocol/clientCapabilities": {}, "org.example/trace": {"inert": true}})
}

/// Build an ordinary wire request; it supplies no captured project or approval proof.
#[allow(clippy::needless_pass_by_value)] // The finite fixture builder owns each complete JSON ID value.
fn request(id: Value, method: &str, mut params: Value) -> Vec<u8> {
    params.as_object_mut().expect("object fixture").insert("_meta".to_owned(), meta());
    let mut bytes = serde_json::to_vec(
        &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
    )
    .unwrap();
    bytes.push(b'\n');
    bytes
}

/// Build exact cancellation with no response ID and no authority-bearing reason.
#[allow(clippy::needless_pass_by_value)] // The finite fixture builder owns each complete cancellation ID value.
fn cancel(id: Value) -> Vec<u8> {
    let mut bytes =
        serde_json::to_vec(&json!({"jsonrpc": "2.0", "method": "notifications/cancelled",
        "params": {"requestId": id, "reason": "not echoed"}}))
        .unwrap();
    bytes.push(b'\n');
    bytes
}

/// Run the actual worker against finite wire/clock/writer ports, not a fake domain implementation.
fn campaign(events: Vec<Event>) -> (Result<(), RunError>, Vec<u8>, Vec<Duration>) {
    let clock = Rc::new(ManualClock { value: Cell::new(Instant::now()) });
    let waits = Rc::new(RefCell::new(Vec::new()));
    let bytes = Rc::new(RefCell::new(Vec::new()));
    let input =
        ScriptedInput { events: events.into(), clock: Rc::clone(&clock), waits: Rc::clone(&waits) };
    let output = ObservedWriter { bytes: Rc::clone(&bytes), fail_after: None, fail_flush: false };
    let result = run_with_clock(&startup(), input, output, clock.as_ref());
    let raw = bytes.borrow().clone();
    let waits = waits.borrow().clone();
    (result, raw, waits)
}

/// Permit real dispatch/publication fences before the fixture's explicit EOF.
fn after(bytes: Vec<u8>) -> Vec<Event> {
    let mut events = vec![Event::Bytes(bytes)];
    events.extend((0..24).map(|_| Event::Idle));
    events.push(Event::Eof);
    events
}

/// Decode actual complete published records for closed synthetic assertions.
fn records(bytes: &[u8]) -> Vec<Value> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| wire::strict_value(line, wire::MAX_RESPONSE).expect("complete output record"))
        .collect()
}

/// Discover is optional as first call; cache/result fields remain mandatory and private.
#[test]
fn list_without_discover_is_modern_complete_static_metadata() {
    let (result, bytes, _) = campaign(after(request(json!(1), "tools/list", json!({}))));
    assert_eq!(result, Ok(()));
    let rows = records(&bytes);
    assert_eq!(rows.len(), 1);
    let result = &rows[0]["result"];
    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["ttlMs"], 0);
    assert_eq!(result["cacheScope"], "private");
    assert_eq!(result["tools"].as_array().unwrap().len(), 7);
    assert!(result.get("nextCursor").is_none());
    for tool in result["tools"].as_array().unwrap() {
        assert_eq!(tool["inputSchema"]["type"], "object");
        assert_eq!(tool["annotations"]["readOnlyHint"], true);
        assert_eq!(tool["annotations"]["openWorldHint"], false);
    }
}

/// Mandatory discovery advertises only actual tools, without project facts or legacy handshake.
#[test]
fn discover_has_exact_revision_complete_cache_and_only_consumed_tools() {
    let (result, bytes, _) =
        campaign(after(request(json!("discover"), "server/discover", json!({}))));
    assert_eq!(result, Ok(()));
    let rows = records(&bytes);
    assert_eq!(rows[0]["result"]["supportedVersions"], json!([wire::REVISION]));
    assert_eq!(rows[0]["result"]["capabilities"], json!({"tools": {}}));
    assert_eq!(rows[0]["result"]["ttlMs"], 0);
    assert_eq!(rows[0]["result"]["cacheScope"], "private");
    assert!(!String::from_utf8(bytes).unwrap().contains("unavailable-private-project"));
}

/// Missing external pair gives a real closed null-data `ToolResult`, never project discovery.
#[test]
fn absent_owner_pair_returns_plain_text_and_closed_structured_unavailability() {
    let (result, bytes, _) = campaign(after(request(
        json!(7),
        "tools/call",
        json!({"name": "get_artifact_status", "arguments": {"artifact_key": "secret-key"}}),
    )));
    assert_eq!(result, Ok(()));
    let rows = records(&bytes);
    let result = &rows[0]["result"];
    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["isError"], true);
    assert_eq!(result["content"][0]["type"], "text");
    assert_eq!(result["structuredContent"]["availability"], "unavailable");
    assert_eq!(result["structuredContent"]["reason"], "approval-unavailable");
    assert_eq!(result["structuredContent"]["data"], Value::Null);
    assert!(!String::from_utf8(bytes).unwrap().contains("secret-key"));
}

/// Every request has its own metadata; earlier discovery never supplies later missing fields.
#[test]
fn metadata_is_per_request_and_client_info_never_becomes_authentication() {
    let mut bytes = request(json!(1), "server/discover", json!({}));
    bytes.extend_from_slice(
        b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n",
    );
    let (result, output, _) = campaign(after(bytes));
    assert_eq!(result, Ok(()));
    let rows = records(&output);
    assert!(rows.iter().any(|row| row["id"] == 2 && row["error"]["code"] == -32602));
    let mut request = request(json!(3), "tools/list", json!({}));
    request.pop();
    let mut value = wire::strict_value(&request, wire::MAX_FRAME).unwrap();
    value["params"]["_meta"]["io.modelcontextprotocol/clientInfo"] =
        json!({"name":"pretend-owner","version":"1"});
    let value = serde_json::to_vec(&value).unwrap();
    assert!(matches!(wire::parse(&value), Ok(Message::Request(_))));
}

/// Unsupported modern revision returns the exact -32022 data, without initialize fallback.
#[test]
fn unsupported_revision_and_obsolete_initialize_do_not_negotiate_a_session() {
    let mut value =
        json!({"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":meta()}});
    value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("2025-11-25");
    let bytes = serde_json::to_vec(&value).unwrap();
    let rejection = wire::parse(&bytes).err().unwrap();
    assert_eq!(rejection.fault, Fault::Version);
    let encoded = wire::error(&rejection).unwrap();
    let row = &records(&encoded)[0];
    assert_eq!(row["error"]["code"], -32022);
    assert_eq!(
        row["error"]["data"],
        json!({"supported":[wire::REVISION],"requested":"2025-11-25"})
    );
    let (_, bytes, _) = campaign(after(request(json!(2), "initialize", json!({}))));
    assert_eq!(records(&bytes)[0]["error"]["code"], -32601);
}

/// Unknown method is distinct from an unknown tool or invalid closed tool arguments.
#[test]
fn method_tool_and_closed_argument_faults_are_exact_protocol_errors() {
    for (method, params, code) in [
        ("unknown/method", json!({}), -32601),
        ("tools/call", json!({"name":"unknown_tool","arguments":{}}), -32602),
        (
            "tools/call",
            json!({"name":"get_artifact_status","arguments":{"artifact_key":"a","extra":true}}),
            -32602,
        ),
        ("tools/list", json!({"cursor":"invented"}), -32602),
    ] {
        let (_, bytes, _) = campaign(after(request(json!(1), method, params)));
        assert_eq!(records(&bytes)[0]["error"]["code"], code);
    }
}

/// Duplicate keys at any depth, BOM, invalid UTF-8 and batches never reach tool admission.
#[test]
fn strict_raw_tree_refuses_duplicate_bom_utf8_batch_and_trailing_values() {
    for bytes in [
        b"{\"a\":1,\"a\":2}".as_slice(),
        b"{\"a\":{\"x\":1,\"x\":2}}".as_slice(),
        b"\xef\xbb\xbf{}".as_slice(),
        b"{\"x\":\"\xff\"}".as_slice(),
        b"{} {}".as_slice(),
    ] {
        assert!(wire::strict_value(bytes, wire::MAX_FRAME).is_err());
    }
    assert_eq!(wire::parse(b"[]").err().unwrap().fault, Fault::Request);
}

/// IDs preserve exact full integer domains and distinguish numeric-looking strings.
#[test]
fn exact_integer_and_string_ids_refuse_float_overflow_bool_and_null() {
    for id in ["18446744073709551615", "-9223372036854775808", "0", "\"0\"", "\"\""] {
        let raw = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/list\",\"params\":{{\"_meta\":{}}}}}",
            meta()
        );
        assert!(matches!(wire::parse(raw.as_bytes()), Ok(Message::Request(_))), "{id}");
    }
    for id in ["1.0", "1e0", "18446744073709551616", "-9223372036854775809", "null", "false"] {
        let raw = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/list\",\"params\":{{\"_meta\":{}}}}}",
            meta()
        );
        assert_eq!(wire::parse(raw.as_bytes()).err().unwrap().fault, Fault::Request, "{id}");
    }
    let mut machine = Machine::default();
    let now = Instant::now();
    machine.accept(RequestId::Unsigned(0), Operation::Discover, now).unwrap();
    machine.accept(RequestId::Text("0".to_owned()), Operation::Discover, now).unwrap();
    assert_eq!(machine.ids.len(), 2);
}

/// Repeated active/queued IDs retire transport without stealing or duplicating responses.
#[test]
fn duplicate_active_or_queued_id_cannot_publish_two_terminal_records() {
    for duplicate in [1, 2] {
        let mut bytes = request(json!(1), "server/discover", json!({}));
        bytes.extend(request(json!(2), "tools/list", json!({})));
        bytes.extend(request(json!(duplicate), "tools/list", json!({})));
        let (result, output, _) = campaign(after(bytes));
        assert_eq!(result, Err(RunError::DuplicateId));
        assert_eq!(output, [] as [u8; 0]);
    }
}

/// Exact approved string-ID bytes refuse the next whole representation, without clipping.
#[test]
fn string_id_complete_128_byte_boundary_preserves_exact_spelling() {
    let mut allowed = request(json!("x".repeat(128)), "tools/list", json!({}));
    allowed.pop();
    assert!(matches!(wire::parse(&allowed), Ok(Message::Request(_))));
    let mut refused = request(json!("x".repeat(129)), "tools/list", json!({}));
    refused.pop();
    let error = wire::parse(&refused).err().unwrap();
    assert_eq!(error.fault, Fault::Request);
    assert!(error.id.is_none());
}

/// Raw and complete encoded limits include their LF and refuse before retained growth.
#[test]
fn frame_and_complete_output_limits_are_whole_record_bounds() {
    let mut framer = Framer::default();
    for _ in 0..wire::MAX_FRAME - 1 {
        assert!(framer.push(b' ').is_none());
    }
    assert_eq!(framer.push(b'\n').unwrap().unwrap().len(), wire::MAX_FRAME - 1);
    for _ in 0..wire::MAX_FRAME {
        assert!(framer.push(b'x').is_none());
    }
    assert_eq!(framer.push(b'\n').unwrap().err(), Some(Fault::Parse));
    let id = RequestId::Unsigned(1);
    assert!(wire::success(&id, &"x".repeat(wire::MAX_RESPONSE)).is_err());
    assert!(wire::success(&id, &"\u{1f}".repeat(wire::MAX_RESPONSE / 6)).is_err());
    let encoded = wire::success(&id, &"é".repeat(1000)).unwrap();
    assert!(encoded.len() <= wire::MAX_RESPONSE);
    assert_eq!(encoded.last(), Some(&b'\n'));
}

/// Strict depth/node accounting includes complete arrays and keys, not filtered results.
#[test]
fn complete_depth_and_node_counts_refuse_before_tree_insertion() {
    let allowed = format!("{}0{}", "[".repeat(63), "]".repeat(63));
    assert!(wire::strict_value(allowed.as_bytes(), wire::MAX_FRAME).is_ok());
    let refused = format!("{}0{}", "[".repeat(64), "]".repeat(64));
    assert!(wire::strict_value(refused.as_bytes(), wire::MAX_FRAME).is_err());
    let nodes = format!("[{}]", vec!["0"; 100_000].join(","));
    assert!(nodes.len() < wire::MAX_RESPONSE);
    assert!(wire::strict_value(nodes.as_bytes(), wire::MAX_RESPONSE).is_err());
}

/// Partial poll-ready bytes are retained without blocking for their missing newline.
#[test]
fn partial_frames_wait_for_later_bytes_without_full_line_reads() {
    let bytes = request(json!(1), "tools/list", json!({}));
    let middle = bytes.len() / 2;
    let mut events = vec![
        Event::Bytes(bytes[..middle].to_vec()),
        Event::Idle,
        Event::Bytes(bytes[middle..].to_vec()),
    ];
    events.extend((0..12).map(|_| Event::Idle));
    events.push(Event::Eof);
    let (result, output, waits) = campaign(events);
    assert_eq!(result, Ok(()));
    assert_eq!(records(&output).len(), 1);
    assert!(waits.iter().all(|wait| *wait == Duration::ZERO || *wait == IDLE_WAIT));
}

/// One active/two waiting requests admit wholly; the fourth receives fixed capacity refusal.
#[test]
fn complete_queue_capacity_refuses_fourth_without_evicting_accepted_work() {
    let mut bytes = Vec::new();
    for id in 1..=4 {
        bytes.extend(request(json!(id), "server/discover", json!({})));
    }
    let (result, output, _) = campaign(after(bytes));
    assert_eq!(result, Ok(()));
    let rows = records(&output);
    assert_eq!(rows.len(), 4);
    for id in 1..=3 {
        assert!(rows.iter().any(|row| row["id"] == id && row.get("result").is_some()));
    }
    assert!(rows.iter().any(|row| row["id"] == 4 && row["error"]["code"] == -32000));
}

/// Known queued/active cancellation removes work without any notification/request response.
#[test]
fn exact_queued_and_active_cancellation_is_sticky_and_unanswered() {
    let mut bytes = request(json!(1), "server/discover", json!({}));
    bytes.extend(request(json!(2), "tools/list", json!({})));
    bytes.extend(cancel(json!(2)));
    let (_, output, _) = campaign(after(bytes));
    assert_eq!(records(&output).len(), 1);
    let mut active = request(json!(3), "tools/list", json!({}));
    active.extend(cancel(json!(3)));
    let (result, output, _) = campaign(after(active));
    assert_eq!(result, Ok(()));
    assert_eq!(output, [] as [u8; 0]);
}

/// Unknown/late/malformed cancellation never invents work or sends a response.
#[test]
fn unknown_and_invalid_cancellation_are_ignored_without_reason_echo() {
    let mut bytes = cancel(json!(99));
    bytes.extend_from_slice(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/cancelled\",\"params\":{\"requestId\":1.5}}\n");
    bytes.extend(request(json!(1), "tools/list", json!({})));
    let (_, output, _) = campaign(after(bytes));
    assert_eq!(records(&output).len(), 1);
    assert!(!String::from_utf8(output).unwrap().contains("not echoed"));
}

/// EOF observed at the actual control fence retires accepted work and partial bytes.
#[test]
fn eof_during_accepted_work_never_publishes_success_or_a_cancel_response() {
    let (result, output, _) =
        campaign(vec![Event::Bytes(request(json!(1), "tools/list", json!({}))), Event::Eof]);
    assert_eq!(result, Ok(()));
    assert_eq!(output, [] as [u8; 0]);
    let (result, output, _) = campaign(vec![Event::Bytes(b"{\"jsonrpc\":".to_vec()), Event::Eof]);
    assert_eq!(result, Ok(()));
    assert_eq!(output, [] as [u8; 0]);
}

/// Real accepted deadlines survive queue promotion and later ready input without renewal.
#[test]
fn accepted_deadline_is_preserved_through_queue_wait_at_actual_worker_port() {
    let mut bytes = request(json!(1), "server/discover", json!({}));
    bytes.extend(request(json!(2), "tools/list", json!({})));
    let mut events =
        vec![Event::Bytes(bytes), Event::Advance(QUERY_BUDGET + Duration::from_millis(1))];
    events.extend((0..12).map(|_| Event::Idle));
    events.push(Event::Eof);
    let (result, output, _) = campaign(events);
    assert_eq!(result, Ok(()));
    let rows = records(&output);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row["error"]["code"] == -32001));
}

/// A complete accepted ID roster refuses rather than evicting or reviving old identities.
#[test]
fn id_retention_is_bounded_and_never_reuses_completed_or_cancelled_ids() {
    let mut machine = Machine::default();
    let now = Instant::now();
    for value in 0..ID_LIMIT {
        let id = RequestId::Unsigned(value as u64);
        machine.accept(id.clone(), Operation::Discover, now).unwrap();
        machine.next().unwrap();
        machine.finish(&id);
    }
    assert_eq!(machine.ids.len(), ID_LIMIT);
    assert_eq!(
        machine.accept(RequestId::Unsigned(ID_LIMIT as u64), Operation::Discover, now),
        Err(Fault::Capacity)
    );
    assert_eq!(
        machine.accept(RequestId::Unsigned(0), Operation::Discover, now),
        Err(Fault::Request)
    );
}

/// The actual pump processes at most eight records even when a read contains a flood.
#[test]
fn input_pump_frame_burst_is_finite_without_discarding_remaining_bytes() {
    let catalog = Catalog::new().unwrap();
    let clock = ManualClock { value: Cell::new(Instant::now()) };
    let mut bytes = Vec::new();
    for _ in 0..20 {
        bytes.extend(cancel(json!(999)));
    }
    let input = ScriptedInput {
        events: vec![Event::Bytes(bytes)].into(),
        clock: Rc::new(ManualClock { value: Cell::new(clock.now()) }),
        waits: Rc::new(RefCell::new(Vec::new())),
    };
    let output = ObservedWriter {
        bytes: Rc::new(RefCell::new(Vec::new())),
        fail_after: None,
        fail_flush: false,
    };
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
    assert!(pump.start < pump.end);
    assert!(pump.output.bytes.borrow().is_empty());
    while pump.start < pump.end {
        pump.service(Duration::ZERO).unwrap();
    }
    assert!(pump.output.bytes.borrow().is_empty());
}

/// Final publication phase disables parser/other-error writes while preserving latched stop/time.
#[test]
fn final_proof_phase_quiesces_input_until_actual_complete_publication() {
    let catalog = Catalog::new().unwrap();
    let clock = Rc::new(ManualClock { value: Cell::new(Instant::now()) });
    let waits = Rc::new(RefCell::new(Vec::new()));
    let input = ScriptedInput {
        events: vec![Event::Idle, Event::Bytes(cancel(json!(1)))].into(),
        clock: Rc::clone(&clock),
        waits: Rc::clone(&waits),
    };
    let output = ObservedWriter {
        bytes: Rc::new(RefCell::new(Vec::new())),
        fail_after: None,
        fail_flush: false,
    };
    let mut machine = Machine::default();
    machine.accept(RequestId::Unsigned(1), Operation::Discover, clock.now()).unwrap();
    machine.next().unwrap();
    let mut pump = Pump {
        input,
        output,
        catalog: &catalog,
        clock: clock.as_ref(),
        machine,
        framer: Framer::default(),
        intake: [0; READ_BURST],
        start: 0,
        end: 0,
    };
    let mut control = Control { pump: &mut pump, io_failure: None, quiescent: false };
    control.enter_publication().unwrap();
    let observed = waits.borrow().len();
    control.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged).unwrap();
    assert_eq!(waits.borrow().len(), observed);
    clock.value.set(clock.now().checked_add(QUERY_BUDGET).unwrap());
    assert!(matches!(
        control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged),
        Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
    ));
    assert!(control.pump.output.bytes.borrow().is_empty());
}

/// Write/flush faults preserve actual prefixes and return failure without another response.
#[test]
fn actual_writer_failure_preserves_prefix_and_never_reports_complete_success() {
    for (fail_after, fail_flush) in [(Some(9), false), (None, true)] {
        let clock = Rc::new(ManualClock { value: Cell::new(Instant::now()) });
        let input = ScriptedInput {
            events: after(request(json!(1), "server/discover", json!({}))).into(),
            clock: Rc::clone(&clock),
            waits: Rc::new(RefCell::new(Vec::new())),
        };
        let bytes = Rc::new(RefCell::new(Vec::new()));
        let writer = ObservedWriter { bytes: Rc::clone(&bytes), fail_after, fail_flush };
        assert_eq!(
            run_with_clock(&startup(), input, writer, clock.as_ref()),
            Err(RunError::Output)
        );
        if let Some(limit) = fail_after {
            assert_eq!(bytes.borrow().len(), limit);
        } else {
            assert_eq!(records(&bytes.borrow()).len(), 1);
        }
    }
}

/// Invalid later wire using an already accepted ID cannot steal its error correlation.
#[test]
fn rejected_duplicate_metadata_retires_original_instead_of_competing_error() {
    let mut bytes = request(json!(1), "tools/list", json!({}));
    bytes.extend_from_slice(
        b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}\n",
    );
    let (result, output, _) = campaign(after(bytes));
    assert_eq!(result, Err(RunError::DuplicateId));
    assert_eq!(output, [] as [u8; 0]);
}

/// Bounded unknown capabilities stay inert while known malformed capability values refuse.
#[test]
fn capabilities_and_metadata_extensions_never_supply_disclosure_authority() {
    let mut value = json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"_meta":meta()}});
    value["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"] = json!({
        "customCapability":{"arbitrary":"inert"}, "roots":{"legacyHint":true},
        "sampling":{"customHint":[1,2]}, "extensions":{"org.example/feature":{"mode":"inert"}}
    });
    assert!(matches!(wire::parse(&serde_json::to_vec(&value).unwrap()), Ok(Message::Request(_))));
    for bad in [
        json!(false),
        json!({"experimental":{"bad":1}}),
        json!({"extensions":{"unprefixed":{}}}),
        json!({"sampling":{"tools":false}}),
    ] {
        value["params"]["_meta"]["io.modelcontextprotocol/clientCapabilities"] = bad;
        assert_eq!(
            wire::parse(&serde_json::to_vec(&value).unwrap()).err().unwrap().fault,
            Fault::Params
        );
    }
}

/// Actual Unix pipe readiness returns partial bytes without waiting for a newline.
/// This native-driver control is separate from real MCP client/platform campaigns.
#[cfg(unix)]
#[test]
#[allow(unsafe_code)] // Actual pipe descriptors are adopted exactly once; each FFI call documents safety.
fn unix_poll_port_returns_partial_bytes_idle_and_real_eof_without_reader_thread() {
    use std::os::fd::{AsRawFd, FromRawFd};
    let mut descriptors = [-1; 2];
    // SAFETY: descriptors has exactly the two writable slots required by pipe.
    assert_eq!(unsafe { libc::pipe(descriptors.as_mut_ptr()) }, 0);
    // SAFETY: successful pipe returned two new uniquely owned descriptors, adopted once.
    let read = unsafe { std::fs::File::from_raw_fd(descriptors[0]) };
    // SAFETY: this is the other new descriptor; File owns its eventual close.
    let mut write = unsafe { std::fs::File::from_raw_fd(descriptors[1]) };
    let mut input = UnixInput { fd: read.as_raw_fd() };
    let mut bytes = [0; READ_BURST];
    assert!(matches!(input.read_ready(Duration::ZERO, &mut bytes).unwrap(), InputRead::Idle));
    write.write_all(b"{").unwrap();
    assert!(matches!(input.read_ready(Duration::ZERO, &mut bytes).unwrap(), InputRead::Data(1)));
    assert_eq!(bytes[0], b'{');
    assert!(matches!(input.read_ready(Duration::ZERO, &mut bytes).unwrap(), InputRead::Idle));
    drop(write);
    assert!(matches!(input.read_ready(Duration::ZERO, &mut bytes).unwrap(), InputRead::Eof));
}

/// Genuine captured-domain wire and final-publication controls reuse existing file-backed fixtures.
#[path = "stdio_captured_tests.rs"]
mod captured;

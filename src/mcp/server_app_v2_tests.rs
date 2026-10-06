// Genuine ServerRead controls are a private descendant of the maintained real App fixture.
// All asserted owner rows remain synthetic data; only actual native factories issue scope.
use super::*;
use crate::cli::McpDeclarationFamily;
use crate::mcp::artifact_status::native_sources_v2::{ServerReadGateV2, capture_server_decision};
use crate::mcp::artifact_status::{catalog::Catalog, queries, stdio, wire};
use std::cell::{Cell, RefCell};
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Duration;

/// Actual final-read files layered on the unchanged genuine native/App fixture.
struct ServerFixture {
    /// All native/currentness inputs still come from actual maintained init/events.
    app: AppFixture,
    /// Complete ordinary selected read decision, not an approval capability.
    decision: Value,
    /// Exact actual external purpose byte pin.
    pin: String,
}
impl ServerFixture {
    /// Keep a real offline baseline separately valid before selecting the read family.
    fn new(report: bool) -> Self {
        let mut app = AppFixture::new();
        if report {
            app.write_report();
            app.bind();
        }
        let mut fixture = Self { app, decision: json!({}), pin: String::new() };
        fixture.app.base.discovery["companion_resources"] = json!([]);
        fixture.app.base.profile["search_index_key"] = Value::Null;
        fixture.app.base.profile["resource_families"] = json!([]);
        fixture.app.base.profile["enabled_tools"] =
            json!(["get_recorded_applicability", "get_gap_summary"]);
        fixture.bind();
        fixture
    }
    /// Rebind only actual original raw bytes and explicit inert six-pair assertions.
    fn bind(&mut self) {
        let base = &mut self.app.base;
        let discovery = encoded(&base.discovery);
        base.profile["discovery_sha256"] = json!(sha256_hex(&discovery));
        let profile = encoded(&base.profile);
        base.profile_pin = sha256_hex(&profile);
        let pairs = [
            ("D067-P1", "product"),
            ("D067-P2", "product"),
            ("D067-S1", "security"),
            ("D067-E1", "engineering"),
            ("D067-S2", "product"),
            ("D067-S2", "security"),
        ];
        let rows = pairs.into_iter().map(|(subject,role)| json!({"subject":subject,"role":role,"disposition":"approved",
            "declared_owner_key":"synthetic-owner","recorded_on":"2026-10-04","source_record_key":"synthetic-record",
            "source_record_sha256":"e".repeat(64),"proposal_index_sha256":PROPOSAL,
            "profile_sha256":base.profile_pin,"discovery_sha256":sha256_hex(&discovery)})).collect::<Vec<_>>();
        self.decision = json!({"schema_version":"forge.mcp-disclosure-decision/2","decision_key":"synthetic-read",
            "project_key":"synthetic-native","discovery_sha256":sha256_hex(&discovery),"profile_sha256":base.profile_pin,
            "proposal_index_sha256":PROPOSAL,"protocol_revision":"2026-07-28",
            "operator_boundary":"operator-selected-declared-owner-record","visibility_profile":base.profile,
            "policy_decisions":rows,"search_index_sha256":null,"evaluation_plan":{"state":"pending",
                "declared_owner_key":null,"recorded_on":null,"source_record_key":null,"source_record_sha256":null,
                "corpus_acceptance":"unearned","threshold_acceptance":"unearned","client_acceptance":"unearned"}});
        fs::write(base.project.path().join("forge.mcp.json"), discovery).unwrap();
        fs::write(base.project.path().join("forge.mcp.visibility.json"), profile).unwrap();
        self.write_decision();
    }
    /// Publish the deliberate raw decision mutation and independently select its actual pin.
    fn write_decision(&mut self) {
        let raw = encoded(&self.decision);
        self.pin = sha256_hex(&raw);
        fs::write(self.app.base.external.path().join("forge.mcp.disclosure-decision.json"), raw)
            .unwrap();
    }
    /// Actual complete native factory, with no reconstructed captured/proof inputs.
    fn load<'a, C: WorkControl + ?Sized>(
        &self,
        caller: &'a mut C,
    ) -> WorkResult<ServerReadGateV2<'a, C>> {
        capture_server_decision(
            &self.app.base.project_root(),
            &self.app.base.external_root(),
            &self.pin,
            &self.app.base.profile_pin,
            caller,
        )
    }
    /// Ordinary explicit operator arguments consumed by the real selected worker.
    fn startup(&self) -> stdio::Startup {
        stdio::Startup {
            project_root: self.app.base.project_root(),
            decision_root: Some(self.app.base.external_root()),
            decision_sha256: Some(self.pin.clone()),
            profile_sha256: self.app.base.profile_pin.clone(),
        }
    }
    /// Add a genuinely held unused original or typed absence, never a fabricated native member.
    fn hidden(&mut self, absent: bool) {
        let path = self.app.base.project.path().join("opaque.txt");
        fs::write(&path, b"hidden original").unwrap();
        let mut row =
            resource("opaque", "policy-source", "opaque.txt", None, self.app.base.project.path());
        row["lifecycle_key"] = Value::Null;
        self.app.base.discovery["resources"].as_array_mut().unwrap().push(row);
        if absent {
            fs::remove_file(path).unwrap();
        }
        self.bind();
    }
    /// Recompute the actual native App snapshot after a legitimate full framework edit.
    fn rebuild_app(&mut self) {
        self.app.base.write_native(true);
        let path = self.app.base.project.path().join("app.json");
        fs::remove_file(&path).unwrap();
        crate::applicability::execute_init(
            &self.app.base.project.path().join("catalog.json"),
            None,
            Some(&path),
        )
        .unwrap();
        self.app.manifest = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        self.app.write_manifest(true);
        let rows = self.app.base.discovery["resources"].as_array_mut().unwrap();
        for row in rows {
            let path = self.app.base.project.path().join(row["path"].as_str().unwrap());
            row["expected_sha256"] = json!(sha256_hex(&fs::read(path).unwrap()));
            if row["key"] == "catalog" {
                row["native_identity"]["document_version"] =
                    self.app.base.catalog["catalog"]["metadata"]["version"].clone();
            }
        }
        self.bind();
    }
}
/// Actual two-tool typed selection, retaining exact page arguments.
fn app_query(
    key: &str,
    subject: Option<&str>,
    limit: usize,
    cursor: Option<String>,
) -> queries::Query {
    queries::Query::GetRecordedApplicability {
        artifact_key: key.into(),
        subject_id: subject.map(str::to_owned),
        page: queries::Page { cursor, limit },
    }
}
/// Observe the actual admitted typed projection and finite schema-validated wire envelope.
fn render(fixture: &ServerFixture, query: &queries::Query) -> Value {
    let mut caller = NoopControl;
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut caller).unwrap() else {
        panic!("genuine server native baseline");
    };
    let prepared =
        queries::prepare_v2(&scope, query).unwrap_or_else(|_| panic!("genuine complete query"));
    let catalog = Catalog::new().unwrap();
    let raw = queries::encode_v2(
        &scope,
        prepared.response(),
        &wire::RequestId::Unsigned(9),
        &catalog,
        false,
    )
    .unwrap_or_else(|_| panic!("complete finite schema envelope"));
    prepared.verify_inputs().unwrap();
    assert!(raw.value().len() <= wire::MAX_RESPONSE && raw.value().last() == Some(&b'\n'));
    wire::strict_value(raw.value(), wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]
        .clone()
}
/// Any ordinary complete-factory refusal is distinct from a genuine successful scope.
fn refuses(result: WorkResult<ServerReadGateV2<'_, impl WorkControl>>) {
    assert!(result.map_or(true, |gate| matches!(gate, ServerReadGateV2::Unavailable)));
}

/// Both actual purpose constructors remain distinct, and the old offline gate still succeeds.
#[test]
fn genuine_read_and_offline_purpose_families_are_not_interchangeable() {
    let offline = AppFixture::new();
    let mut control = NoopControl;
    assert!(matches!(offline.base.load(&mut control), Ok(OfflineBuildGate::Validated(_))));
    let mut server = ServerFixture::new(false);
    assert_eq!(render(&server, &app_query("app", None, 50, None))["data"]["matched"], 4);
    let actual_intent =
        fs::read(offline.base.external.path().join("forge.mcp.index-build-intent.json")).unwrap();
    fs::write(
        server.app.base.external.path().join("forge.mcp.disclosure-decision.json"),
        &actual_intent,
    )
    .unwrap();
    server.pin = sha256_hex(&actual_intent);
    refuses(server.load(&mut NoopControl));
    let mut caller = NoopControl;
    assert!(!matches!(
        capture_index_build_intent(
            &server.app.base.project_root(),
            &server.app.base.external_root(),
            &server.pin,
            &server.app.base.profile_pin,
            &mut caller
        ),
        Ok(OfflineBuildGate::Validated(_))
    ));
}

/// Maintained full report rows/counts are the expected oracle, including private data exclusion.
#[test]
fn full_native_rows_and_saved_report_match_maintained_output_without_private_prose() {
    for stored in [false, true] {
        let fixture = ServerFixture::new(stored);
        let key = if stored { "report" } else { "app" };
        let actual = crate::applicability::prepare_analysis(
            &fixture.app.base.project.path().join("app.json"),
            ReportFilters::default(),
        )
        .unwrap();
        let page = render(&fixture, &app_query(key, None, 50, None));
        let rows = page["data"]["rows"].as_array().unwrap();
        assert_eq!(rows.len(), actual.report.controls.len());
        for (row, control) in rows.iter().zip(&actual.report.controls) {
            assert_eq!(row["control_id"], control.control_id);
            assert_eq!(row["classification"], json!(control.classification));
        }
        let summary = render(
            &fixture,
            &queries::Query::GetGapSummary {
                artifact_key: key.into(),
                page: queries::Page { cursor: None, limit: 2 },
            },
        );
        assert_eq!(
            summary["data"]["summary"],
            serde_json::to_value(&actual.report.counts).unwrap()
        );
        let text = serde_json::to_string(&page).unwrap();
        for private in [
            "Private App reviewer",
            "Private applicable rationale",
            "app-reviewer",
            "app.json",
            "source.txt",
        ] {
            assert!(!text.contains(private));
        }
    }
}

/// Two genuine distinct full matching manifests and private report mutations refuse whole output.
#[test]
fn complete_saved_report_ambiguity_and_private_mutations_refuse_whole_factory() {
    for case in 0..4 {
        let mut app = AppFixture::new();
        app.write_report();
        if case == 0 {
            app.duplicate = true;
            fs::copy(
                app.base.project.path().join("app.json"),
                app.base.project.path().join("app-copy.json"),
            )
            .unwrap();
            app.write_source_record("app-copy.json", "copy-life.json", true);
        } else {
            let path = app.base.project.path().join("report.json");
            let mut report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            match case {
                1 => report["controls"][0]["rationale"] = json!("different private rationale"),
                2 => report["reviewers"][0]["name"] = json!("different private reviewer"),
                _ => report["counts"]["total"] = json!(5),
            }
            fs::write(path, encoded(&report)).unwrap();
            app.write_source_record("report.json", "report-life.json", true);
        }
        app.bind();
        let mut fixture = ServerFixture::new(false);
        fixture.app = app;
        fixture.app.base.discovery["companion_resources"] = json!([]);
        fixture.app.base.profile["search_index_key"] = Value::Null;
        fixture.app.base.profile["resource_families"] = json!([]);
        fixture.bind();
        refuses(fixture.load(&mut NoopControl));
    }
}

/// Source and framework approval are independent genuine full lifecycle predicates.
#[test]
fn independent_app_and_framework_approvals_are_both_required() {
    for source in [true, false] {
        let mut fixture = ServerFixture::new(false);
        if source {
            fixture.app.write_manifest(false);
        } else {
            fixture.app.base.write_native(false);
        }
        for row in fixture.app.base.discovery["resources"].as_array_mut().unwrap() {
            let path = fixture.app.base.project.path().join(row["path"].as_str().unwrap());
            row["expected_sha256"] = json!(sha256_hex(&fs::read(path).unwrap()));
        }
        fixture.bind();
        refuses(fixture.load(&mut NoopControl));
    }
}

/// Filtering and paging preserve the complete native order/count and implicit classification.
#[test]
fn full_order_subject_filter_and_second_page_conserve_native_denominator() {
    let fixture = ServerFixture::new(false);
    let full = render(&fixture, &app_query("app", None, 50, None));
    let first = render(&fixture, &app_query("app", None, 2, None));
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_owned();
    let second = render(&fixture, &app_query("app", None, 2, Some(cursor)));
    let joined = first["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["data"]["rows"].as_array().unwrap())
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(joined, *full["data"]["rows"].as_array().unwrap());
    assert_eq!(first["data"]["generation"], second["data"]["generation"]);
    let selected = render(&fixture, &app_query("app", Some("c-parent"), 1, None));
    assert_eq!(selected["data"]["matched"], 1);
    assert_eq!(full["data"]["rows"][3]["decision_state"], Value::Null);
    assert_eq!(full["data"]["rows"][3]["decision_source"], "implicit-under-review");
}

/// A private native-valid wide ID/version outside the filter refuses the full public denominator.
#[test]
fn wider_private_native_id_or_version_is_never_filtered_into_public_success() {
    for version in [false, true] {
        let mut fixture = ServerFixture::new(false);
        if version {
            fixture.app.base.catalog["catalog"]["metadata"]["version"] = json!("é-version");
        } else {
            fixture.app.base.catalog["catalog"]["controls"][0]["id"] = json!("x".repeat(129));
        }
        fixture.rebuild_app();
        let mut caller = NoopControl;
        let ServerReadGateV2::Validated(scope) = fixture.load(&mut caller).unwrap() else {
            panic!("genuine private native baseline");
        };
        assert!(scope.search_sources().iter().any(|row| if version {
            row.artifact_identity.document_version == "é-version"
        } else {
            row.native_id.len() == 129
        }));
        assert!(matches!(
            queries::prepare_v2(&scope, &app_query("app", Some("c-child"), 1, None)),
            Err(queries::QueryError::Unavailable(queries::Reason::VisibilityRefused))
        ));
    }
}

/// Canonical generation excludes both cursor and page size; hidden physical originals still bind it.
#[test]
fn canonical_paging_invariance_and_hidden_original_generation_are_real() {
    let mut fixture = ServerFixture::new(false);
    fixture.hidden(false);
    let first = render(&fixture, &app_query("app", None, 1, None));
    let larger = render(&fixture, &app_query("app", None, 3, None));
    assert_eq!(first["data"]["generation"], larger["data"]["generation"]);
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_owned();
    let path = fixture.app.base.project.path().join("opaque.txt");
    fs::write(&path, b"changed hidden original").unwrap();
    let rows = fixture.app.base.discovery["resources"].as_array_mut().unwrap();
    rows.iter_mut().find(|row| row["key"] == "opaque").unwrap()["expected_sha256"] =
        json!(sha256_hex(&fs::read(path).unwrap()));
    fixture.bind();
    let mut control = NoopControl;
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut control).unwrap() else {
        panic!("repinned actual hidden owner");
    };
    assert!(matches!(
        queries::prepare_v2(&scope, &app_query("app", None, 1, Some(cursor))),
        Err(queries::QueryError::Unavailable(queries::Reason::GenerationChanged))
    ));
}

/// Real nested pointers, CRLF spans and raw pins come from the complete native tuple producer.
#[test]
fn every_returned_citation_matches_actual_complete_native_source_tuple() {
    let fixture = ServerFixture::new(false);
    let actual = render(&fixture, &app_query("app", None, 50, None));
    let tuples = fixture.app.base.authored_tuples();
    for row in actual["data"]["rows"].as_array().unwrap() {
        let id = row["control_id"].as_str().unwrap();
        let expected = tuples
            .iter()
            .find(|tuple| tuple["artifact_key"] == "catalog" && tuple["native_id"] == id)
            .unwrap();
        let citation = &row["citations"][0];
        for field in [
            "artifact_key",
            "artifact_identity",
            "artifact_raw_sha256",
            "native_pointer",
            "source_key",
            "source_raw_sha256",
            "source_span",
            "artifact_role",
        ] {
            assert_eq!(citation[field], expected[field]);
        }
        assert_eq!(citation["citation_label"], "catalog-label");
        assert_eq!(citation["source_state"], "exact-captured-source");
    }
}

/// Unsupported declared native families and selected metadata modes refuse the entire read gate.
#[test]
fn unsupported_native_and_metadata_selections_remain_complete_unavailability() {
    for case in 0..4 {
        let mut fixture = ServerFixture::new(false);
        match case {
            0 => fixture.app.manifest["mapping_artifacts"] = json!(["mapping.json"]),
            1 => fixture.app.manifest["framework"]["resource_type"] = json!("profile"),
            2 => fixture.app.base.profile["resource_families"] = json!(["inventory"]),
            _ => fixture.app.base.profile["noncurrent_metadata_keys"] = json!(["catalog"]),
        }
        if case < 2 {
            fixture.app.write_manifest(true);
            fixture.app.base.discovery["resources"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["key"] == "app")
                .unwrap()["expected_sha256"] = json!(sha256_hex(
                &fs::read(fixture.app.base.project.path().join("app.json")).unwrap()
            ));
            fixture.app.base.discovery["resources"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["key"] == "app-life")
                .unwrap()["expected_sha256"] = json!(sha256_hex(
                &fs::read(fixture.app.base.project.path().join("app-life.json")).unwrap()
            ));
        }
        fixture.bind();
        refuses(fixture.load(&mut NoopControl));
    }
}

/// Original final failure checkpoint wins over ordinary coherent report mismatch.
#[test]
fn ordinary_failed_native_report_phase_observes_original_final_control_stop() {
    let mut fixture = ServerFixture::new(true);
    let path = fixture.app.base.project.path().join("report.json");
    let mut report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    report["counts"]["total"] = json!(9);
    fs::write(&path, encoded(&report)).unwrap();
    fixture.app.write_source_record("report.json", "report-life.json", true);
    for row in fixture.app.base.discovery["resources"].as_array_mut().unwrap() {
        row["expected_sha256"] = json!(sha256_hex(
            &fs::read(fixture.app.base.project.path().join(row["path"].as_str().unwrap())).unwrap()
        ));
    }
    fixture.bind();
    let mut baseline = Caller::new(None, false);
    assert!(matches!(fixture.load(&mut baseline), Ok(ServerReadGateV2::Unavailable)));
    let last = baseline.stages.len();
    assert!(last > 0);
    for failed in [false, true] {
        let mut caller = Caller::new(Some(last), failed);
        let actual = fixture.load(&mut caller);
        if failed {
            assert!(
                matches!(actual, Err(WorkError::Failed(error)) if error.code == "actual-test-control")
            );
        } else {
            assert!(matches!(actual, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
    }
}

/// Failed projection remains ordinary until the actual original postphase stop; Capacity is sticky.
#[test]
fn ordinary_projection_failure_and_first_capacity_use_one_original_owner() {
    let fixture = ServerFixture::new(false);
    let mut control = Caller::new(None, false);
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut control).unwrap() else {
        panic!("actual owner");
    };
    let query = queries::Query::ListPolicies(queries::Page { cursor: None, limit: 1 });
    let before = scope.with_control(|control| control.stages.len());
    assert!(matches!(
        queries::prepare_v2(&scope, &query),
        Err(queries::QueryError::Unavailable(queries::Reason::UnsupportedRole))
    ));
    let delta = scope.with_control(|control| control.stages.len() - before);
    assert!(delta > 0);
    scope.with_control(|control| control.at = Some(control.stages.len() + delta));
    assert!(matches!(
        queries::prepare_v2(&scope, &query),
        Err(queries::QueryError::Work(WorkError::Interrupted(Interruption::CancelRequested)))
    ));
    drop(scope);
    let mut control = Caller::new(None, true);
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut control).unwrap() else {
        panic!("second actual operation");
    };
    let before = scope.with_control(|control| control.stages.len());
    assert!(
        matches!(scope.admission().charge(100_000), Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity")
    );
    scope.with_control(|control| control.at = Some(control.stages.len() + 1));
    assert!(
        matches!(queries::prepare_v2(&scope, &app_query("app", None, 1, None)), Err(queries::QueryError::Work(WorkError::Failed(error))) if error.code == "mcp-capture-capacity")
    );
    assert_eq!(scope.with_control(|control| control.stages.len()), before + 1);
}

/// Real large native contents reach unchanged capture capacity, without lowering any ceiling.
#[test]
fn genuine_large_private_native_input_refuses_before_available_projection_growth() {
    let mut fixture = ServerFixture::new(false);
    let template = fixture.app.base.catalog["catalog"]["controls"][0].clone();
    let mut controls = Vec::new();
    for number in 0..900 {
        let mut row = template.clone();
        row["id"] = json!(format!("large-{number:04}"));
        row["title"] = json!("P".repeat(8192));
        controls.push(row);
    }
    fixture.app.base.catalog["catalog"]["controls"] = Value::Array(controls);
    fixture.rebuild_app();
    assert!(
        matches!(fixture.load(&mut NoopControl), Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity")
    );
}

/// Real postencoding whole-owner fences observe hidden content or typed absence appearance.
#[test]
fn postencoding_complete_owner_fence_refuses_hidden_and_absence_drift() {
    for absent in [false, true] {
        let mut fixture = ServerFixture::new(false);
        fixture.hidden(absent);
        let mut control = NoopControl;
        let ServerReadGateV2::Validated(scope) = fixture.load(&mut control).unwrap() else {
            panic!("actual full hidden roster");
        };
        let prepared = queries::prepare_v2(&scope, &app_query("app", None, 1, None))
            .unwrap_or_else(|_| panic!("actual projection"));
        let encoded = queries::encode_v2(
            &scope,
            prepared.response(),
            &wire::RequestId::Unsigned(1),
            &Catalog::new().unwrap(),
            false,
        )
        .unwrap_or_else(|_| panic!("actual complete encoded/schema phase"));
        scope.with_control(|_| {
            fs::write(
                fixture.app.base.project.path().join("opaque.txt"),
                b"drifted actual original",
            )
            .unwrap();
        });
        let mut output = Vec::new();
        if prepared.verify_inputs().is_ok() {
            output.write_all(encoded.value()).unwrap();
        }
        assert_eq!(output, [] as [u8; 0]);
        assert!(scope.verify_inputs().is_err());
        drop(encoded);
        drop(prepared);
        drop(scope);
        control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged).unwrap();
    }
}

/// Actual writer-backed port with response barriers and one authentic prepublication input action.
struct WireInput {
    /// Real complete request, consumed once in bounded chunks.
    request: Vec<u8>,
    /// Actual written bytes; EOF is conditional on real publication, not a forged result.
    output: Rc<RefCell<Vec<u8>>>,
    /// Full actual input observation count used to calibrate the consumed final observation.
    calls: Rc<Cell<usize>>,
    /// Optional one-based authentic input call; no deadline or proof is altered.
    at: Option<usize>,
    /// Drift, cancellation or EOF to deliver through the actual input port.
    action: Option<Action>,
    /// Actual one-based action delivery; zero means no action was delivered.
    delivered_at: Rc<Cell<usize>>,
    /// Actual read which delivered the subsequent cancellation-only EOF.
    cancel_eof_at: Rc<Cell<usize>>,
    /// Set only after cancellation Data; the next input read supplies genuine EOF.
    eof_after_cancel: bool,
}
/// One actual final observation change, not a successful native capability.
enum Action {
    Drift(std::path::PathBuf),
    Cancel,
    Eof,
}
impl stdio::InputPort for WireInput {
    /// Deliver exact request/cancellation/EOF or mutate an actual file at the calibrated native port.
    fn read_ready(
        &mut self,
        _wait: Duration,
        destination: &mut [u8],
    ) -> io::Result<stdio::InputRead> {
        let count = self.calls.get() + 1;
        self.calls.set(count);
        if count > 100_000 {
            return Err(io::Error::other("bounded genuine worker fixture"));
        }
        if self.eof_after_cancel {
            self.eof_after_cancel = false;
            self.cancel_eof_at.set(count);
            return Ok(stdio::InputRead::Eof);
        }
        if self.at == Some(count) {
            self.delivered_at.set(count);
            match self.action.take().expect("actual single action") {
                Action::Drift(path) => {
                    fs::write(path, b"drifted final source")?;
                    return Ok(stdio::InputRead::Idle);
                }
                Action::Eof => return Ok(stdio::InputRead::Eof),
                Action::Cancel => {
                    let bytes = b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/cancelled\",\"params\":{\"requestId\":7}}\n";
                    destination[..bytes.len()].copy_from_slice(bytes);
                    self.eof_after_cancel = true;
                    return Ok(stdio::InputRead::Data(bytes.len()));
                }
            }
        }
        if !self.request.is_empty() {
            let len = self.request.len().min(destination.len());
            destination[..len].copy_from_slice(&self.request[..len]);
            self.request.drain(..len);
            return Ok(stdio::InputRead::Data(len));
        }
        if self.output.borrow().contains(&b'\n') {
            Ok(stdio::InputRead::Eof)
        } else {
            Ok(stdio::InputRead::Idle)
        }
    }
}
/// Actual complete buffer writer; optional real partial error remains truthful.
struct WireWriter {
    bytes: Rc<RefCell<Vec<u8>>>,
    fail_after: Option<usize>,
}
impl Write for WireWriter {
    /// Write exactly accepted bytes and fail on the next real write after the chosen extent.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let old = self.bytes.borrow().len();
        let count =
            self.fail_after.map_or(bytes.len(), |limit| limit.saturating_sub(old).min(bytes.len()));
        if count == 0 {
            return Err(io::Error::other("actual writer failure"));
        }
        self.bytes.borrow_mut().extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    /// No hidden partial frame is retained by the fixture writer.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
/// Actual closed modern per-call wire request with mandatory independent metadata.
fn frame(tool: &str, arguments: &Value) -> Vec<u8> {
    let mut raw = encoded(
        &json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":tool,"arguments":arguments,
        "_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}),
    );
    raw.push(b'\n');
    raw
}
/// Actual input deliveries retained independently of stdout publication.
struct WireObservations {
    /// Complete input read count; includes EOF when the actual worker observes it.
    calls: usize,
    /// One-based action delivery observation, or zero for the calibration baseline.
    delivered_at: usize,
    /// Actual immediately subsequent cancellation-only EOF observation, or zero.
    cancel_eof_at: usize,
}
/// Execute the real selected worker and preserve complete actual output and input chronology.
fn worker(
    startup: &stdio::Startup,
    family: McpDeclarationFamily,
    request: Vec<u8>,
    at: Option<usize>,
    action: Option<Action>,
    fail_after: Option<usize>,
) -> (Result<(), stdio::RunError>, Vec<u8>, WireObservations) {
    let bytes = Rc::new(RefCell::new(Vec::new()));
    let calls = Rc::new(Cell::new(0));
    let delivered_at = Rc::new(Cell::new(0));
    let cancel_eof_at = Rc::new(Cell::new(0));
    let input = WireInput {
        request,
        output: Rc::clone(&bytes),
        calls: Rc::clone(&calls),
        at,
        action,
        delivered_at: Rc::clone(&delivered_at),
        cancel_eof_at: Rc::clone(&cancel_eof_at),
        eof_after_cancel: false,
    };
    let writer = WireWriter { bytes: Rc::clone(&bytes), fail_after };
    let result = stdio::run_selected(startup, family, input, writer);
    let raw = bytes.borrow().clone();
    let observations = WireObservations {
        calls: calls.get(),
        delivered_at: delivered_at.get(),
        cancel_eof_at: cancel_eof_at.get(),
    };
    (result, raw, observations)
}

/// Both supported tools reach the actual dispatch/schema/final owner/stdout path.
#[test]
fn actual_selected_worker_publishes_both_supported_tools_with_complete_wire_schema() {
    let fixture = ServerFixture::new(false);
    for tool in ["get_recorded_applicability", "get_gap_summary"] {
        let (result, raw, _) = worker(
            &fixture.startup(),
            McpDeclarationFamily::V2,
            frame(tool, &json!({"artifact_key":"app","limit":2})),
            None,
            None,
            None,
        );
        assert_eq!(result, Ok(()));
        let value = wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap();
        assert_eq!(value["result"]["structuredContent"]["availability"], "available");
        assert_eq!(value["result"]["structuredContent"]["data"]["matched"], 4);
        Catalog::new().unwrap().validate_encoded(tool, &raw).unwrap();
    }
}

/// Missing/partial selection and explicit /1 mismatch retain only fixed null/static outputs.
#[test]
fn selected_family_never_falls_back_and_static_discovery_reads_no_project() {
    let fixture = ServerFixture::new(false);
    let request = frame("get_gap_summary", &json!({"artifact_key":"app","limit":1}));
    let (result, raw, _) =
        worker(&fixture.startup(), McpDeclarationFamily::V1, request.clone(), None, None, None);
    assert_eq!(result, Ok(()));
    assert_eq!(
        wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]["data"],
        Value::Null
    );
    let mut startup = fixture.startup();
    startup.project_root = Path::new("missing/../never-read").into();
    startup.decision_root = None;
    let (result, raw, _) = worker(&startup, McpDeclarationFamily::V2, request, None, None, None);
    assert_eq!(result, Ok(()));
    assert_eq!(
        wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]["data"],
        Value::Null
    );
    let discover = b"{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"server/discover\",\"params\":{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"2026-07-28\",\"io.modelcontextprotocol/clientCapabilities\":{}}}}\n".to_vec();
    let (result, raw, _) = worker(&startup, McpDeclarationFamily::V2, discover, None, None, None);
    assert_eq!(result, Ok(()));
    let discovered = wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap();
    assert_eq!(discovered["result"]["supportedVersions"], json!(["2026-07-28"]));
    assert_eq!(discovered["result"]["resultType"], "complete");
}

/// Authentic final input count is calibrated only after real complete successful publication.
#[test]
fn actual_worker_final_drift_cancellation_and_eof_publish_no_available_bytes() {
    for kind in 0..3 {
        let fixture = ServerFixture::new(false);
        let request = frame("get_gap_summary", &json!({"artifact_key":"app","limit":2}));
        let (result, raw, observations) =
            worker(&fixture.startup(), McpDeclarationFamily::V2, request.clone(), None, None, None);
        assert_eq!(result, Ok(()));
        assert_eq!(
            wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]["availability"],
            "available"
        );
        let calls = observations.calls;
        assert!(calls > 2);
        assert_eq!(observations.delivered_at, 0);
        assert_eq!(observations.cancel_eof_at, 0);
        // The successful worker's last input call is genuine postpublication EOF; the preceding
        // one is enter_publication's final finite observation, after real encoding/schema.
        let action = match kind {
            0 => Action::Drift(fixture.app.base.project.path().join("source.txt")),
            1 => Action::Cancel,
            _ => Action::Eof,
        };
        let (result, raw, observations) = worker(
            &fixture.startup(),
            McpDeclarationFamily::V2,
            request,
            Some(calls - 1),
            Some(action),
            None,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(observations.delivered_at, calls - 1);
        if kind == 1 {
            assert_eq!(observations.cancel_eof_at, calls);
            assert_eq!(observations.calls, calls);
        } else {
            assert_eq!(observations.cancel_eof_at, 0);
            assert_eq!(observations.calls, if kind == 0 { calls + 1 } else { calls - 1 });
        }
        if kind == 0 {
            assert_ne!(
                wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]
                    ["availability"],
                "available"
            );
        } else {
            assert_eq!(raw, [] as [u8; 0]);
        }
    }
}

/// Actual bounded write failure can leave a truthful prefix and retires the same transport.
#[test]
fn actual_worker_writer_failure_preserves_partial_output_boundary() {
    let fixture = ServerFixture::new(false);
    let (result, raw, _) = worker(
        &fixture.startup(),
        McpDeclarationFamily::V2,
        frame("get_gap_summary", &json!({"artifact_key":"app","limit":1})),
        None,
        None,
        Some(37),
    );
    assert_eq!(result, Err(stdio::RunError::Output));
    assert_eq!(raw.len(), 37);
    assert!(!raw.contains(&b'\n'));
}

/// Real maintained clap parsing selects only the closed explicit family and preserves default /1.
#[test]
fn maintained_clap_defaults_to_v1_and_accepts_only_explicit_v2() {
    use clap::Parser;
    for (extra, expected) in [
        (None, McpDeclarationFamily::V1),
        (Some("v1"), McpDeclarationFamily::V1),
        (Some("v2"), McpDeclarationFamily::V2),
    ] {
        let mut args = vec!["forge", "mcp", "serve", "--project", "missing"];
        if let Some(value) = extra {
            args.extend(["--declaration-family", value]);
        }
        let cli = crate::cli::Cli::try_parse_from(args).unwrap();
        let crate::cli::Commands::Mcp {
            command: crate::cli::McpCommand::Serve { declaration_family, .. },
        } = cli.command
        else {
            panic!("actual clap route");
        };
        assert_eq!(declaration_family, expected);
    }
    assert!(
        crate::cli::Cli::try_parse_from([
            "forge",
            "mcp",
            "serve",
            "--project",
            "missing",
            "--declaration-family",
            "v3"
        ])
        .is_err()
    );
}

/// Additive genuine ordinary-tool controls retain the complete existing App fixture and bodies.
mod ordinary_tools {
    include!("ordinary_server_v2_tests.rs");
}

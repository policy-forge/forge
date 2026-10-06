// Genuine ordinary ServerRead controls below the existing native/App fixture.
// Synthetic owner declarations do not grant real owner or client acceptance.
use super::*;
use crate::mcp::artifact_status::declarations_v2::DeclarationAdmission;

/// Enable the seven actually implemented tools in the real complete selected `ServerRead` fixture.
fn fixture() -> ServerFixture {
    let mut fixture = ServerFixture::new(false);
    fixture.app.base.profile["enabled_tools"] = json!([
        "list_policies",
        "search_requirements",
        "get_requirement",
        "trace_control",
        "get_recorded_applicability",
        "get_gap_summary",
        "get_artifact_status"
    ]);
    fixture.bind();
    fixture
}
/// Actual exact native lookup, preserving the original selector bytes.
fn requirement(key: &str, id: &str) -> queries::Query {
    queries::Query::GetRequirement {
        artifact_key: key.into(),
        requirement_id: id.into(),
        include_excerpt: false,
    }
}
/// Actual whole native control relation page; no Mapping relationship is inferred.
fn trace(key: &str, id: &str, limit: usize, cursor: Option<String>) -> queries::Query {
    queries::Query::TraceControl {
        artifact_key: key.into(),
        control_id: id.into(),
        page: queries::Page { cursor, limit },
    }
}
/// Actual lexical query over the retained native/source tuple universe.
fn search(text: &str, key: Option<&str>, limit: usize, cursor: Option<String>) -> queries::Query {
    queries::Query::SearchRequirements {
        query: text.into(),
        artifact_key: key.map(str::to_owned),
        include_excerpt: false,
        page: queries::Page { cursor, limit },
    }
}
/// Require a fixed ordinary refusal, never accepting a capacity/control failure as a negative result.
fn reason(fixture: &ServerFixture, query: &queries::Query, expected: queries::Reason) {
    let mut control = NoopControl;
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut control).unwrap() else {
        panic!("real baseline owner");
    };
    match queries::prepare_v2(&scope, query) {
        Err(queries::QueryError::Unavailable(actual)) => assert_eq!(json!(actual), json!(expected)),
        _ => panic!("expected ordinary fixed refusal, never a sticky native/control stop"),
    }
    scope.verify_inputs().unwrap();
}
/// Interpret only a real complete finite worker response, with its exact shipped schema.
fn wire_result(fixture: &ServerFixture, tool: &str, arguments: &Value) -> Value {
    let (result, raw, _) = worker(
        &fixture.startup(),
        McpDeclarationFamily::V2,
        frame(tool, arguments),
        None,
        None,
        None,
    );
    assert_eq!(result, Ok(()));
    Catalog::new_v2().unwrap().validate_encoded(tool, &raw).unwrap();
    wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"].clone()
}
/// Rebuild actual native inventories/current lifecycles/App source after one schema-valid native ID edit.
fn wide_fixture(id: &str) -> ServerFixture {
    let mut fixture = fixture();
    fixture.app.base.catalog["catalog"]["controls"][0]["id"] = json!(id);
    fixture.rebuild_app();
    fixture
}

/// All seven tools reach genuine native preparation, full finite schema readback and stdout.
#[test]
fn seven_selected_tools_publish_real_complete_native_results() {
    let fixture = fixture();
    let calls = [
        ("list_policies", json!({"limit":1})),
        ("search_requirements", json!({"query":"control", "limit":2})),
        ("get_requirement", json!({"artifact_key":"catalog", "requirement_id":"c-nested"})),
        ("trace_control", json!({"artifact_key":"component", "control_id":"c-parent", "limit":1})),
        ("get_recorded_applicability", json!({"artifact_key":"app", "limit":2})),
        ("get_gap_summary", json!({"artifact_key":"app", "limit":2})),
        ("get_artifact_status", json!({"artifact_key":"source"})),
    ];
    for (tool, arguments) in calls {
        let result = wire_result(&fixture, tool, &arguments);
        assert_eq!(result["availability"], "available", "{tool}");
        assert_eq!(result["reason"], Value::Null);
    }
}

/// Policy pages conserve the full two-native denominator and canonical artifact-key order.
#[test]
fn policy_listing_complete_denominator_order_and_cursor_are_independent_of_source_rows() {
    let fixture = fixture();
    let first =
        render(&fixture, &queries::Query::ListPolicies(queries::Page { cursor: None, limit: 1 }));
    assert_eq!(first["data"]["matched"], 2);
    assert_eq!(first["data"]["emitted"], 1);
    assert_eq!(first["data"]["rows"][0]["artifact_key"], "catalog");
    let second = render(
        &fixture,
        &queries::Query::ListPolicies(queries::Page {
            cursor: Some(first["data"]["next_cursor"].as_str().unwrap().into()),
            limit: 50,
        }),
    );
    assert_eq!(second["data"]["matched"], 2);
    assert_eq!(second["data"]["rows"][0]["artifact_key"], "component");
    assert_eq!(second["data"]["next_cursor"], Value::Null);
    for row in first["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["data"]["rows"].as_array().unwrap())
    {
        assert_eq!(row["lifecycle_state"], "approved");
        assert_eq!(row["schedule"], Value::Null);
        assert_eq!(row["schedule_scope"], "not-evaluated");
    }
}

/// A nested genuine Catalog pointer and CRLF line cite exact bytes, without title/path/prose output.
#[test]
fn exact_nested_requirement_cites_actual_pointer_hash_and_crlf_span() {
    let fixture = fixture();
    let result = render(&fixture, &requirement("catalog", "c-nested"));
    let citation = &result["data"]["citations"][0];
    assert_eq!(citation["native_pointer"], "/catalog/groups/0/groups/0/controls/0");
    assert_eq!(citation["source_span"], json!({"start_byte":15,"end_byte":27}));
    assert_eq!(citation["source_raw_sha256"], sha256_hex(SOURCE));
    assert_eq!(
        citation["artifact_raw_sha256"],
        sha256_hex(&fs::read(fixture.app.base.project.path().join("catalog.json")).unwrap())
    );
    assert_eq!(citation["source_state"], "exact-captured-source");
    assert_eq!(result["data"]["excerpt"], Value::Null);
    let text = encoded(&result);
    let text = std::str::from_utf8(&text).unwrap();
    for private in
        ["Private native", "Private section", "source.txt", "catalog.json", "Beta control"]
    {
        assert!(!text.contains(private));
    }
}

/// Exact native Component UUIDs remain distinct from their shared control and source-line identities.
#[test]
fn component_and_capability_exact_requirements_conserve_two_recorded_control_relations() {
    let fixture = fixture();
    let ids = ["44444444-4444-4444-8444-444444444444", "55555555-5555-4555-8555-555555555555"];
    for (id, span) in ids.into_iter().zip([(0, 13), (29, 42)]) {
        let result = render(&fixture, &requirement("component", id));
        assert_eq!(result["data"]["requirement_id"], id);
        assert_eq!(
            result["data"]["citations"][0]["source_span"],
            json!({"start_byte":span.0,"end_byte":span.1})
        );
    }
    let first = render(&fixture, &trace("component", "c-parent", 1, None));
    assert_eq!(first["data"]["matched"], 2);
    assert_eq!(first["data"]["emitted"], 1);
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_owned();
    let second = render(&fixture, &trace("component", "c-parent", 50, Some(cursor)));
    for (result, id) in [&first, &second].into_iter().zip(ids) {
        let row = &result["data"]["rows"][0];
        assert_eq!(row["requirement_id"], id);
        assert_eq!(row["control_id"], "c-parent");
        assert_eq!(row["mapping_id"], Value::Null);
        assert_eq!(row["relation"], "exact-recorded-control");
        assert_eq!(row["evidence_metadata"], Value::Null);
    }
}

/// Source-text matching uses the actual unterminated captured line but does not disclose its text.
#[test]
fn lexical_search_matches_real_source_line_and_preserves_full_ordered_denominator() {
    let fixture = fixture();
    let result = render(&fixture, &search("gamma", None, 50, None));
    assert_eq!(result["data"]["matched"], 2);
    assert_eq!(result["data"]["emitted"], 2);
    let rows = result["data"]["rows"].as_array().unwrap();
    assert_eq!(rows[0]["requirement"]["requirement_id"], "z-last");
    assert_eq!(rows[1]["requirement"]["requirement_id"], "55555555-5555-4555-8555-555555555555");
    for (rank, row) in rows.iter().enumerate() {
        assert_eq!(row["rank"], rank + 1);
        assert_eq!(row["match_kind"], "all-tokens");
        assert_eq!(row["matched_fields"], json!(["captured-requirement-text"]));
        assert_eq!(row["requirement"]["excerpt"], Value::Null);
    }
}

/// Exact ID outranks other lexical matches, retaining complete matching before a one-row page.
#[test]
fn lexical_rank_buckets_and_page_generation_use_complete_exact_id_matches() {
    let fixture = fixture();
    let result = render(&fixture, &search("c-parent", None, 1, None));
    assert_eq!(result["data"]["matched"], 5);
    assert_eq!(result["data"]["rows"][0]["requirement"]["requirement_id"], "c-parent");
    assert_eq!(result["data"]["rows"][0]["match_kind"], "exact-id");
    let next = render(
        &fixture,
        &search("c-parent", None, 50, Some(result["data"]["next_cursor"].as_str().unwrap().into())),
    );
    assert_eq!(next["data"]["matched"], 5);
    assert_eq!(next["data"]["rows"][0]["rank"], 2);
    assert_eq!(next["data"]["rows"][0]["match_kind"], "exact-id");
    assert_eq!(next["data"]["rows"][2]["match_kind"], "some-tokens");
}

/// Actual lowercase/dedup semantics preserve source matching while raw query identity remains exact.
#[test]
fn lexical_duplicate_tokens_are_deduplicated_without_normalizing_query_generation() {
    let fixture = fixture();
    let first = render(&fixture, &search("ALPHA alpha", None, 50, None));
    let second = render(&fixture, &search("alpha", None, 50, None));
    assert_eq!(first["data"]["rows"], second["data"]["rows"]);
    assert_ne!(first["data"]["generation"], second["data"]["generation"]);
    assert_eq!(first["data"]["matched"], 2);
    let unicode = render(&fixture, &search("İ", None, 50, None));
    assert_eq!(unicode["data"]["matched"], 0);
    assert_eq!(unicode["data"]["next_cursor"], Value::Null);
}

/// Thirty-two distinct tokens are admitted; a thirty-third ordinary token refuses complete output.
#[test]
fn lexical_distinct_token_bound_and_invalid_text_remain_ordinary_fixed_refusals() {
    let fixture = fixture();
    let text = (0..32).map(|i| format!("word{i}")).collect::<Vec<_>>().join(" ");
    assert_eq!(render(&fixture, &search(&text, None, 50, None))["data"]["matched"], 0);
    reason(&fixture, &search(&(text + " extra"), None, 50, None), queries::Reason::NotFound);
    for text in ["", "!!!", "control\n"] {
        reason(&fixture, &search(text, None, 50, None), queries::Reason::NotFound);
    }
}

/// Full native selectors up to4096 bytes remain emitted and exactly selectable in /2 intake.
#[test]
fn native_wide_and_utf8_ids_are_exactly_selectable_through_real_v2_worker() {
    for id in ["x".repeat(129), "y".repeat(4096), format!("{}é", "z".repeat(4094))] {
        let fixture = wide_fixture(&id);
        let exact = wire_result(
            &fixture,
            "get_requirement",
            &json!({"artifact_key":"catalog","requirement_id":id}),
        );
        assert_eq!(exact["availability"], "available");
        assert_eq!(exact["data"]["requirement_id"], id);
        let trace = wire_result(
            &fixture,
            "trace_control",
            &json!({"artifact_key":"catalog","control_id":id}),
        );
        assert_eq!(trace["data"]["matched"], 1);
        assert_eq!(trace["data"]["rows"][0]["control_id"], id);
    }
}

/// The unchanged /1/default intake rejects129-byte selectors while /2 uses an explicit constructor.
#[test]
fn ordinary_default_v1_admission_does_not_inherit_v2_native_selector_width() {
    let id = "x".repeat(129);
    let fixture = wide_fixture(&id);
    for family in [McpDeclarationFamily::V1, McpDeclarationFamily::V2] {
        let (result, raw, _) = worker(
            &fixture.startup(),
            family,
            frame("get_requirement", &json!({"artifact_key":"catalog","requirement_id":id})),
            None,
            None,
            None,
        );
        assert_eq!(result, Ok(()));
        let value = wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap();
        if family == McpDeclarationFamily::V1 {
            assert!(value.get("error").is_some());
        } else {
            assert_eq!(value["result"]["structuredContent"]["data"]["requirement_id"], id);
        }
    }
}

/// Byte-wide selectors reject above4096 even when JSON-schema's character count remains lawful.
#[test]
fn v2_native_selector_byte_limit_refuses_before_missing_project_capture() {
    let fixture = fixture();
    let mut startup = fixture.startup();
    startup.project_root = Path::new("missing/../never-open").into();
    let id = format!("{}é", "x".repeat(4095));
    assert_eq!(id.len(), 4097);
    let (result, raw, _) = worker(
        &startup,
        McpDeclarationFamily::V2,
        frame("get_requirement", &json!({"artifact_key":"catalog","requirement_id":id})),
        None,
        None,
        None,
    );
    assert_eq!(result, Ok(()));
    assert!(wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap().get("error").is_some());
}

/// Neutral status for actual `PolicySource` has required null identity and no schedule-derived claims.
#[test]
fn genuine_policy_source_and_native_status_keep_identity_and_schedule_boundaries() {
    let fixture = fixture();
    for key in ["source", "catalog", "component"] {
        let result =
            render(&fixture, &queries::Query::GetArtifactStatus { artifact_key: key.into() });
        let row = &result["data"];
        assert_eq!(row["lifecycle_state"], "approved");
        assert_eq!(row["freshness_scope"], "complete-recorded-fingerprint-current");
        assert_eq!(row["schedule_scope"], "not-evaluated");
        assert_eq!(row["schedule"], Value::Null);
        if key == "source" {
            assert_eq!(row["native_identity"], Value::Null);
        } else {
            assert_eq!(
                row["native_identity"]["root_id"],
                if key == "catalog" {
                    "11111111-1111-4111-8111-111111111111"
                } else {
                    "22222222-2222-4222-8222-222222222222"
                }
            );
        }
    }
    reason(
        &fixture,
        &queries::Query::GetArtifactStatus { artifact_key: "app".into() },
        queries::Reason::UnsupportedRole,
    );
}

/// Excerpt opt-in, disabled operation and hidden selection refuse without emitting private bytes.
#[test]
fn actual_visibility_scope_and_excerpt_refusals_are_closed_and_complete() {
    let mut fixture = fixture();
    let mut query = requirement("catalog", "c-parent");
    if let queries::Query::GetRequirement { include_excerpt, .. } = &mut query {
        *include_excerpt = true;
    }
    reason(&fixture, &query, queries::Reason::VisibilityRefused);
    fixture.app.base.profile["enabled_tools"] = json!(["get_artifact_status"]);
    fixture.bind();
    reason(&fixture, &requirement("catalog", "c-parent"), queries::Reason::UnsupportedRole);
    reason(
        &fixture,
        &queries::Query::GetArtifactStatus { artifact_key: "lifecycle".into() },
        queries::Reason::VisibilityRefused,
    );
}

/// Exact misses and empty full matches remain distinguishable from hidden/private native details.
#[test]
fn exact_native_miss_and_zero_trace_search_pages_have_full_null_cursor_semantics() {
    let fixture = fixture();
    reason(&fixture, &requirement("catalog", "missing-control"), queries::Reason::NotFound);
    for query in [trace("catalog", "missing-control", 1, None), search("absentword", None, 1, None)]
    {
        let result = render(&fixture, &query);
        assert_eq!(result["data"]["matched"], 0);
        assert_eq!(result["data"]["emitted"], 0);
        assert_eq!(result["data"]["rows"], json!([]));
        assert_eq!(result["data"]["next_cursor"], Value::Null);
    }
}

/// Cursors bind the whole raw query/selection/tool generation and reject malformed ordinal aliases.
#[test]
fn complete_generation_rejects_filter_tool_and_noncanonical_ordinal_cursor_reuse() {
    let fixture = fixture();
    let page = render(&fixture, &search("control", None, 1, None));
    let cursor = page["data"]["next_cursor"].as_str().unwrap().to_owned();
    reason(
        &fixture,
        &search("control", Some("catalog"), 1, Some(cursor.clone())),
        queries::Reason::GenerationChanged,
    );
    reason(
        &fixture,
        &trace("component", "c-parent", 1, Some(cursor)),
        queries::Reason::GenerationChanged,
    );
    let generation = page["data"]["generation"].as_str().unwrap();
    for suffix in ["01", "9999", "-1", ""] {
        reason(
            &fixture,
            &search("control", None, 1, Some(format!("{generation}:{suffix}"))),
            queries::Reason::GenerationChanged,
        );
    }
}

/// An unchanged visible query still receives a different complete physical generation after hidden drift.
#[test]
fn hidden_original_change_invalidates_complete_cursor_generation_and_prepared_fence() {
    let mut fixture = fixture();
    fixture.hidden(false);
    let first = render(&fixture, &search("control", None, 1, None));
    let mut control = NoopControl;
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut control).unwrap() else {
        panic!("real complete owner");
    };
    let prepared = queries::prepare_v2(&scope, &requirement("catalog", "c-parent"))
        .unwrap_or_else(|_| panic!("real query"));
    fs::write(fixture.app.base.project.path().join("opaque.txt"), b"changed hidden raw bytes")
        .unwrap();
    assert!(prepared.verify_inputs().is_err());
    drop(prepared);
    drop(scope);
    let row = fixture.app.base.discovery["resources"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["key"] == "opaque")
        .unwrap();
    row["expected_sha256"] =
        json!(sha256_hex(&fs::read(fixture.app.base.project.path().join("opaque.txt")).unwrap()));
    fixture.bind();
    let second = render(&fixture, &search("control", None, 1, None));
    assert_ne!(first["data"]["generation"], second["data"]["generation"]);
    reason(
        &fixture,
        &search("control", None, 1, Some(first["data"]["next_cursor"].as_str().unwrap().into())),
        queries::Reason::GenerationChanged,
    );
}

/// Original capacity remains sticky after real native qualification and blocks later control callbacks.
#[test]
fn ordinary_query_does_not_renew_the_original_native_work_allowance() {
    let fixture = fixture();
    let mut caller = Caller::new(None, true);
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut caller).unwrap() else {
        panic!("real native owner");
    };
    let prepared = queries::prepare_v2(&scope, &requirement("catalog", "c-parent"))
        .unwrap_or_else(|_| panic!("real bounded query"));
    drop(prepared);
    let before = scope.with_control(|control| control.stages.len());
    let mut a = scope.admission();
    assert!(
        matches!(a.charge(100_000),Err(WorkError::Failed(error))if error.code=="mcp-capture-capacity")
    );
    let stopped = scope.with_control(|control| control.stages.len());
    assert_eq!(stopped, before + 1);
    assert!(
        matches!(queries::prepare_v2(&scope,&requirement("catalog","c-parent")),Err(queries::QueryError::Work(WorkError::Failed(error)))if error.code=="mcp-capture-capacity")
    );
    assert_eq!(scope.with_control(|control| control.stages.len()), stopped);
}

/// Genuine success and ordinary failure both reach the last same-original query phase checkpoint.
#[test]
fn actual_query_success_and_ordinary_failure_preserve_final_original_control_stop() {
    let fixture = fixture();
    for query in [requirement("catalog", "c-parent"), requirement("catalog", "absent")] {
        let mut baseline = Caller::new(None, false);
        let ServerReadGateV2::Validated(scope) = fixture.load(&mut baseline).unwrap() else {
            panic!("real native baseline");
        };
        let result = queries::prepare_v2(&scope, &query);
        drop(result);
        let count = scope.with_control(|control| control.stages.len());
        drop(scope);
        for failed in [false, true] {
            let mut caller = Caller::new(Some(count), failed);
            let ServerReadGateV2::Validated(scope) = fixture.load(&mut caller).unwrap() else {
                panic!("real unchanged factory");
            };
            let result = queries::prepare_v2(&scope, &query);
            if failed {
                assert!(
                    matches!(result,Err(queries::QueryError::Work(WorkError::Failed(error)))if error.code=="actual-test-control")
                );
            } else {
                assert!(matches!(
                    result,
                    Err(queries::QueryError::Work(WorkError::Interrupted(
                        Interruption::CancelRequested
                    )))
                ));
            }
        }
    }
}

/// New ordinary output preserves actual calibrated final drift/cancel/EOF publication boundaries.
#[test]
fn ordinary_worker_final_hidden_drift_cancel_and_eof_emit_no_available_data() {
    for kind in 0..3 {
        let mut fixture = fixture();
        fixture.hidden(false);
        let request = frame(
            "get_requirement",
            &json!({"artifact_key":"catalog","requirement_id":"c-parent"}),
        );
        let (result, raw, observed) =
            worker(&fixture.startup(), McpDeclarationFamily::V2, request.clone(), None, None, None);
        assert_eq!(result, Ok(()));
        assert_eq!(
            wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]["availability"],
            "available"
        );
        assert!(observed.calls > 2);
        let at = observed.calls - 1;
        let action = match kind {
            0 => Action::Drift(fixture.app.base.project.path().join("opaque.txt")),
            1 => Action::Cancel,
            _ => Action::Eof,
        };
        let (result, raw, actual) = worker(
            &fixture.startup(),
            McpDeclarationFamily::V2,
            request,
            Some(at),
            Some(action),
            None,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(actual.delivered_at, at);
        if kind == 0 {
            assert_ne!(
                wire::strict_value(&raw, wire::MAX_RESPONSE).unwrap()["result"]["structuredContent"]
                    ["availability"],
                "available"
            );
        } else {
            assert_eq!(raw.as_slice(), &[] as &[u8]);
        }
        if kind == 1 {
            assert_eq!(actual.cancel_eof_at, at + 1);
        }
    }
}

/// A genuine ordinary partial stdout error retains its exact prefix and never claims a full frame.
#[test]
fn ordinary_worker_partial_writer_failure_preserves_truthful_output_boundary() {
    let fixture = fixture();
    let (result, raw, _) = worker(
        &fixture.startup(),
        McpDeclarationFamily::V2,
        frame("get_artifact_status", &json!({"artifact_key":"source"})),
        None,
        None,
        Some(37),
    );
    assert_eq!(result, Err(stdio::RunError::Output));
    assert_eq!(raw.len(), 37);
    assert!(!raw.contains(&b'\n'));
}

/// An unselected native metadata operand outside the public profile refuses the complete tuple universe.
#[test]
fn unselected_private_native_version_is_not_filtered_out_of_a_requirement_success() {
    let mut fixture = fixture();
    fixture.app.base.component["component-definition"]["metadata"]["version"] = json!("é-version");
    fixture.rebuild_app();
    let rows = fixture.app.base.discovery["resources"].as_array_mut().unwrap();
    rows.iter_mut().find(|row| row["key"] == "component").unwrap()["native_identity"]["document_version"] =
        json!("é-version");
    fixture.bind();
    reason(&fixture, &requirement("catalog", "c-parent"), queries::Reason::VisibilityRefused);
    reason(
        &fixture,
        &search("alpha", Some("catalog"), 1, None),
        queries::Reason::VisibilityRefused,
    );
}

/// Both actual prepared families coexist under one original native owner and finite output admission.
#[test]
fn app_and_ordinary_prepared_results_share_one_real_owner_without_reset_or_dto_cast() {
    let fixture = fixture();
    let mut caller = Caller::new(None, false);
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut caller).unwrap() else {
        panic!("complete actual owner");
    };
    let before = scope.with_control(|control| control.stages.len());
    let ordinary = queries::prepare_v2(&scope, &requirement("catalog", "c-parent"))
        .unwrap_or_else(|_| panic!("real native projection"));
    let app = queries::prepare_v2(&scope, &app_query("app", None, 1, None))
        .unwrap_or_else(|_| panic!("unchanged App projection"));
    assert!(scope.with_control(|control| control.stages.len()) > before);
    let catalog = Catalog::new_v2().unwrap();
    let one = queries::encode_v2(
        &scope,
        ordinary.response(),
        &wire::RequestId::Unsigned(1),
        &catalog,
        false,
    )
    .unwrap_or_else(|_| panic!("finite native envelope"));
    let two =
        queries::encode_v2(&scope, app.response(), &wire::RequestId::Unsigned(2), &catalog, false)
            .unwrap_or_else(|_| panic!("finite App envelope"));
    catalog.validate_encoded("get_requirement", one.value()).unwrap();
    catalog.validate_encoded("get_recorded_applicability", two.value()).unwrap();
    ordinary.verify_inputs().unwrap();
    app.verify_inputs().unwrap();
    assert_eq!(wire::strict_value(one.value(), wire::MAX_RESPONSE).unwrap()["id"], 1);
    assert_eq!(wire::strict_value(two.value(), wire::MAX_RESPONSE).unwrap()["id"], 2);
}

/// Source text, schedule, evidence, static reports and index selection remain whole-gate refusals.
#[test]
fn unsupported_metadata_families_do_not_acquire_core_disclosure_through_enabled_tools() {
    for case in 0..6 {
        let mut fixture = fixture();
        match case {
            0 => fixture.app.base.profile["source_text_keys"] = json!(["source"]),
            1 => fixture.app.base.profile["schedule_keys"] = json!(["catalog"]),
            2 => fixture.app.base.profile["evidence_metadata_keys"] = json!(["catalog"]),
            3 => fixture.app.base.profile["static_report_keys"] = json!(["catalog"]),
            4 => fixture.app.base.profile["search_index_key"] = json!("index"),
            _ => fixture.app.base.profile["resource_families"] = json!(["inventory"]),
        }
        fixture.bind();
        match fixture.load(&mut NoopControl) {
            Ok(ServerReadGateV2::Unavailable) => (),
            Err(WorkError::Failed(error)) if error.code == "mcp-core-unavailable" => (),
            _ => {
                panic!("ordinary whole-gate refusal only, never capacity/control or partial owner")
            }
        }
    }
}

/// A failed prefix/suffix never becomes a hit; the next delimiter begins a new complete word.
#[test]
fn mismatched_native_words_resume_only_at_real_delimiters_and_never_accept_prefixes() {
    for (id, query, matched) in [
        ("needleword", "needle", 0),
        ("needleword", "word", 0),
        ("needleword", "needleword", 1),
        ("wrong-needleword", "needleword", 1),
        ("needleword-extra", "needleword", 1),
    ] {
        let fixture = wide_fixture(id);
        let result = render(&fixture, &search(query, Some("catalog"), 50, None));
        assert_eq!(result["data"]["matched"], matched, "complete native word boundary");
        if matched == 1 {
            assert_eq!(result["data"]["rows"][0]["requirement"]["requirement_id"], id);
        }
    }
}

/// Unicode lowercase expansions keep their full bytes, including a mismatch after a matched prefix.
#[test]
fn unicode_lowercase_expansions_preserve_whole_native_word_equality() {
    for (id, query, matched) in [
        ("İ", "İ", 1),
        ("İx", "İ", 0),
        ("İx", "İx", 1),
        ("İ-x", "İ", 1),
        ("Éclair", "éclair", 1),
        ("Éclairx", "éclair", 0),
    ] {
        let fixture = wide_fixture(id);
        let result = render(&fixture, &search(query, Some("catalog"), 50, None));
        assert_eq!(result["data"]["matched"], matched, "complete Unicode native word");
        if matched == 1 {
            assert_eq!(result["data"]["rows"][0]["requirement"]["requirement_id"], id);
        }
    }
}

/// Thirty-two real tokens still match the complete two-row native universe before a one-row page.
#[test]
fn thirty_two_tokens_preserve_complete_positive_search_denominator_before_paging() {
    let fixture = fixture();
    let mut tokens = (0..31).map(|i| format!("word{i}")).collect::<Vec<_>>();
    tokens.push("gamma".to_owned());
    let result = render(&fixture, &search(&tokens.join(" "), None, 1, None));
    assert_eq!(result["data"]["matched"], 2);
    assert_eq!(result["data"]["emitted"], 1);
    assert!(result["data"]["next_cursor"].is_string());
    let row = &result["data"]["rows"][0];
    assert_eq!(row["requirement"]["requirement_id"], "z-last");
    assert_eq!(row["match_kind"], "some-tokens");
    assert_eq!(row["matched_fields"], json!(["captured-requirement-text"]));
}

/// Skipped comparisons do not refund complete scalar scans or renew the original sticky work cap.
#[test]
fn complete_long_mismatched_word_scans_still_latch_original_capacity() {
    let fixture = wide_fixture(&"x".repeat(4096));
    let mut caller = Caller::new(None, false);
    let ServerReadGateV2::Validated(scope) = fixture.load(&mut caller).unwrap() else {
        panic!("real complete native owner");
    };
    let text = (0..32).map(|i| format!("word{i}")).collect::<Vec<_>>().join(" ");
    assert!(matches!(queries::prepare_v2(&scope, &search(&text, Some("catalog"), 50, None)),
        Err(queries::QueryError::Work(WorkError::Failed(error))) if error.code == "mcp-capture-capacity"));
    let stopped = scope.with_control(|control| control.stages.len());
    assert!(matches!(queries::prepare_v2(&scope, &requirement("catalog", "c-parent")),
        Err(queries::QueryError::Work(WorkError::Failed(error))) if error.code == "mcp-capture-capacity"));
    assert!(
        matches!(scope.verify_inputs(), Err(WorkError::Failed(error)) if error.code == "mcp-capture-capacity")
    );
    assert_eq!(scope.with_control(|control| control.stages.len()), stopped);
}

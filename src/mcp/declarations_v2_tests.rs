//! Proposed pure declaration controls; no original capture, human approval, native execution,
//! or completed /2 runtime integration is asserted by these synthetic JSON records.

use super::*;
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError,
};
use serde_json::json;

/// Synthetic complete three-config values, retaining explicit nulls and all privacy modes.
struct Fixture {
    /// Closed discovery assertion, with original base roles plus all five companion kinds.
    project: Value,
    /// Exact raw-hash-correlated visibility assertion.
    profile: Value,
    /// Six recorded provenance assertions, not any actual owner decision.
    decision: Value,
}

impl Fixture {
    /// Construct inert Catalog/Profile/Component relations and hidden closure declarations.
    fn new() -> Self {
        let project = project_fixture();
        let profile = profile_fixture();
        let decision = decision_fixture();
        let mut fixture = Self { project, profile, decision };
        fixture.bind();
        fixture
    }
    /// Rebind synthetic raw hashes after an intentional mutation, preserving actual parse errors.
    fn bind(&mut self) {
        let discovery = encoded(&self.project);
        self.profile["discovery_sha256"] = json!(sha256_hex(&discovery));
        let profile = encoded(&self.profile);
        self.decision["discovery_sha256"] = json!(sha256_hex(&discovery));
        self.decision["profile_sha256"] = json!(sha256_hex(&profile));
        self.decision["visibility_profile"] = self.profile.clone();
        self.decision["policy_decisions"] =
            rows(&READ_PAIRS, &sha256_hex(&discovery), &sha256_hex(&profile));
    }
    /// Invoke the real proposed complete raw decoder with independent expected-byte pins.
    fn decode(&self) -> WorkResult<InertServerDeclarations> {
        self.decode_with(&mut TestAdmission::new(&mut NoopControl))
    }
    /// Reuse the caller's actual component ledger/control, including earlier failed work.
    fn decode_with(
        &self,
        admission: &mut dyn DeclarationAdmission,
    ) -> WorkResult<InertServerDeclarations> {
        let project = encoded(&self.project);
        let profile = encoded(&self.profile);
        let decision = encoded(&self.decision);
        decode_server_declarations(
            &project,
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            admission,
        )
    }
    /// Select the same nonnull final profile and create a separate inert offline-intent assertion.
    fn build_intent(&mut self) -> Value {
        self.profile["search_index_key"] = json!("search-index");
        self.bind();
        json!({"schema_version":"forge.mcp-index-build-intent/1","intent_key":"build-record",
            "project_key":"synthetic","discovery_sha256":self.profile["discovery_sha256"],
            "profile_sha256":self.decision["profile_sha256"],"proposal_index_sha256":PROPOSAL,
            "operator_boundary":"operator-selected-declared-owner-record","visibility_profile":self.profile,
            "capture_roster":{"resources":self.project["resources"],"companion_resources":self.project["companion_resources"]},
            "search_sources":[{"artifact_key":"catalog","artifact_role":"oscal-catalog-artifact",
                "artifact_identity":self.project["resources"][2]["native_identity"],
                "artifact_raw_sha256":"a".repeat(64),"native_id":"control-a","control_id":"control-a",
                "native_pointer":"/catalog/controls/0","source_key":"source","source_raw_sha256":"a".repeat(64),
                "source_span":{"start_byte":0,"end_byte":4},"lifecycle_key":"lifecycle"}],
            "destination":{"key":"search-index","path":"search-index.json",
                "index_format":"forge.mcp-search-index/1","tokenizer_identity":"declared-unicode-build-1"},
            "policy_decisions":rows(&BUILD_PAIRS, self.profile["discovery_sha256"].as_str().expect("fixture discovery pin"),
                self.decision["profile_sha256"].as_str().expect("fixture profile pin"))})
    }
    /// Invoke the actual intent decoder without creating any capture or server scope.
    fn decode_intent(&self, intent: &Value) -> WorkResult<InertBuildDeclarations> {
        self.decode_intent_with(intent, &mut TestAdmission::new(&mut NoopControl))
    }
    /// Pass the same existing component owner to the distinct actual build-intent port.
    fn decode_intent_with(
        &self,
        intent: &Value,
        admission: &mut dyn DeclarationAdmission,
    ) -> WorkResult<InertBuildDeclarations> {
        let project = encoded(&self.project);
        let profile = encoded(&self.profile);
        let intent = encoded(intent);
        decode_build_intent(
            &project,
            &profile,
            &intent,
            &sha256_hex(&intent),
            &sha256_hex(&profile),
            admission,
        )
    }
}

/// Declare all native and companion members; no original file or schema is fabricated as valid.
fn project_fixture() -> Value {
    json!({"schema_version":"forge.mcp-project/2", "project_key":"synthetic",
            "resources":[
                base_row("source", "policy-source", None),
                base_row("lifecycle", "lifecycle-record", None),
                base_row("catalog", "oscal-catalog-artifact", Some("catalog")),
                base_row("component", "oscal-component-artifact", Some("component-definition")),
                base_row("profile", "oscal-profile-artifact", Some("profile"))],
            "companion_resources":[
                {"kind":"linkage-manifest","key":"links","path":"links.json",
                    "expected_sha256":"a".repeat(64),"citation_label":"links-label","lifecycle_key":"lifecycle",
                    "bindings":{"requirements":[
                        {"requirement_key":"catalog-native","resource_key":"catalog","resolved_catalog_key":null},
                        {"requirement_key":"profile-native","resource_key":"profile","resolved_catalog_key":"catalog"}],
                        "implementation_resource_key":"component",
                        "local_evidence":[{"evidence_key":"local-native","resource_key":"evidence"}]}},
                {"kind":"local-evidence","key":"evidence","path":"evidence.bin",
                    "expected_sha256":"b".repeat(64),"expected_size":0,"citation_label":"evidence-label"},
                {"kind":"workspace-index-metadata","key":"index-metadata","path":"forge.workspace.json",
                    "expected_sha256":"c".repeat(64),"citation_label":"index-label"},
                {"kind":"static-report","key":"report","path":"report.html",
                    "expected_sha256":"d".repeat(64),"citation_label":"report-label",
                    "report_format":"forge.workspace-report/1","workspace_index_key":"index-metadata"},
                {"kind":"search-index","key":"search-index","path":"search-index.json",
                    "citation_label":"search-label","index_format":"forge.mcp-search-index/1"}]})
}

/// Keep every source/schedule/resource permission and nullable selector explicit in the fixture.
fn profile_fixture() -> Value {
    json!({"schema_version":"forge.mcp-visibility/2","project_key":"synthetic",
            "discovery_sha256":"0".repeat(64),
            "enabled_tools":["list_policies","search_requirements","get_requirement","trace_control",
                "get_recorded_applicability","get_gap_summary","get_artifact_status"],
            "visible_resource_keys":["catalog","component","source"],"noncurrent_metadata_keys":[],
            "source_text_keys":[],"schedule_keys":[],"evidence_metadata_keys":["links"],
            "static_report_keys":["report"],"search_index_key":null,
            "identity_disclosure":"bounded-token-only","approved_current_basis":"caller-established-recorded-policy",
            "source_text_mode":"omit","schedule_mode":"omit","report_content_mode":"minimized-metadata",
            "evidence_bytes":false,"network":false,"process_execution":false,"as_of":null,
            "resource_families":["inventory","artifact-status","requirement-metadata","implementation-evidence","static-report"],
            "index_fallback_mode":"refuse"})
}

/// Declare unearned evaluation with a separate complete read decision assertion.
fn decision_fixture() -> Value {
    json!({"schema_version":"forge.mcp-disclosure-decision/2","decision_key":"record",
            "project_key":"synthetic","discovery_sha256":"0".repeat(64),"profile_sha256":"0".repeat(64),
            "proposal_index_sha256":PROPOSAL,"protocol_revision":"2026-07-28",
            "operator_boundary":"operator-selected-declared-owner-record","visibility_profile":{},
            "policy_decisions":[],"search_index_sha256":null,
            "evaluation_plan":{"state":"pending","declared_owner_key":null,"recorded_on":null,
                "source_record_key":null,"source_record_sha256":null,"corpus_acceptance":"unearned",
                "threshold_acceptance":"unearned","client_acceptance":"unearned"}})
}

/// Construct one original `ResourcePolicy` JSON row with unchanged native/null role pairing.
fn base_row(key: &str, role: &str, model: Option<&str>) -> Value {
    json!({"key":key,"role":role,"path":format!("{key}.json"),"expected_sha256":"a".repeat(64),
        "native_identity":model.map(|model|json!({"model":model,
            "root_id":"11111111-1111-4111-8111-111111111111","document_version":"1","oscal_version":"1.2.3"})),
        "lifecycle_key":if key=="lifecycle" {None} else {Some("lifecycle")},"citation_label":format!("{key}-label")})
}

/// Construct complete distinct declared rows with explicit provenance, never actual approval.
fn rows(pairs: &[(&str, &str); 6], discovery: &str, profile: &str) -> Value {
    Value::Array(pairs.iter().map(|(subject, role)|json!({"subject":subject,"role":role,"disposition":"approved",
        "declared_owner_key":"synthetic-owner","recorded_on":"2026-10-04","source_record_key":"synthetic-record",
        "source_record_sha256":"e".repeat(64),"proposal_index_sha256":PROPOSAL,
        "profile_sha256":profile,"discovery_sha256":discovery})).collect())
}

/// Serialize only synthetic control input; production digest admission consumes raw supplied bytes.
fn encoded(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("synthetic JSON encoding")
}

/// All five companions and Profile/Catalog nullable joins are structurally admitted as inert data.
#[test]
fn complete_companions_admit_without_a_capture_or_approval_factory() {
    let fixture = Fixture::new();
    let mut control = NoopControl;
    let mut admission = TestAdmission::new(&mut control);
    let value = fixture.decode_with(&mut admission).expect("inert complete declared relation");
    assert_eq!(value.project.companion_resources.len(), 5);
    assert_eq!(value.profile, value.decision.visibility_profile);
    assert_eq!(value.decision.evaluation_plan.client_acceptance, "unearned");
    assert!(admission.used > 0);
}

/// All /1 selectors and mixed families remain rejected rather than implicitly upgraded.
#[test]
fn rejects_original_and_mixed_version_records() {
    for version in ["forge.mcp-project/1", "forge.mcp-project/3"] {
        let mut fixture = Fixture::new();
        fixture.project["schema_version"] = json!(version);
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
    let mut fixture = Fixture::new();
    fixture.profile["schema_version"] = json!("forge.mcp-visibility/1");
    fixture.bind();
    assert!(fixture.decode().is_err());
}

/// Strict raw admission catches duplicate fields and BOM before typed deserialization.
#[test]
fn raw_duplicates_and_bom_do_not_become_valid_declarations() {
    let fixture = Fixture::new();
    let profile = encoded(&fixture.profile);
    let decision = encoded(&fixture.decision);
    let mut raw = encoded(&fixture.project);
    raw.splice(1..1, b"\"project_key\":\"duplicate\",".iter().copied());
    assert!(
        decode_server_declarations(
            &raw,
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            &mut TestAdmission::new(&mut NoopControl)
        )
        .is_err()
    );
    let mut bom = b"\xef\xbb\xbf".to_vec();
    bom.extend(encoded(&fixture.project));
    assert!(
        decode_server_declarations(
            &bom,
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            &mut TestAdmission::new(&mut NoopControl)
        )
        .is_err()
    );
}

/// Required null fields and unknown members are consumed by schema, beyond serde Option defaults.
#[test]
fn missing_nullable_and_unknown_fields_refuse() {
    let mut fixture = Fixture::new();
    fixture.project["resources"][0].as_object_mut().expect("row").remove("native_identity");
    fixture.bind();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.project["companion_resources"][1]["uri"] = json!("https://private.invalid");
    fixture.bind();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.decision.as_object_mut().expect("decision").remove("search_index_sha256");
    assert!(fixture.decode().is_err());
}

/// Discovery/profile/decision raw hashes are exact complete byte pins rather than normalized JSON.
#[test]
fn expected_raw_pins_and_complete_embedded_profile_must_match() {
    let fixture = Fixture::new();
    let project = encoded(&fixture.project);
    let profile = encoded(&fixture.profile);
    let decision = encoded(&fixture.decision);
    assert!(
        decode_server_declarations(
            &project,
            &profile,
            &decision,
            &"f".repeat(64),
            &sha256_hex(&profile),
            &mut TestAdmission::new(&mut NoopControl)
        )
        .is_err()
    );
    let mut fixture = fixture;
    fixture.decision["visibility_profile"]["index_fallback_mode"] = json!("direct-search");
    assert!(fixture.decode().is_err());
    let mut profile = profile;
    profile.push(b'\n');
    assert!(
        decode_server_declarations(
            &project,
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            &mut TestAdmission::new(&mut NoopControl)
        )
        .is_err()
    );
}

/// Complete roster rejects base/companion duplicate keys, labels, and folded ancestor conflicts.
#[test]
fn identities_are_unique_across_base_and_companion_rosters() {
    for (field, value) in
        [("key", "catalog"), ("citation_label", "catalog-label"), ("path", "CATALOG.json/child")]
    {
        let mut fixture = Fixture::new();
        fixture.project["companion_resources"][1][field] = json!(value);
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
}

/// Unsafe portable spelling and both fixed project config overlaps refuse before any member read.
#[test]
fn private_paths_cannot_select_configs_or_unsafe_portable_targets() {
    for path in [
        "../outside",
        "CON.json",
        "mixed\\slash.json",
        "forge.mcp.json",
        "forge.mcp.visibility.json/child",
    ] {
        let mut fixture = Fixture::new();
        fixture.project["resources"][0]["path"] = json!(path);
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
}

/// Every old/new foreign key is exact and kind-specific; hidden invalid dependencies still refuse.
#[test]
fn complete_foreign_key_role_checks_do_not_filter_hidden_dependencies() {
    for (field, key) in [("lifecycle_key", "catalog"), ("lifecycle_key", "missing")] {
        let mut fixture = Fixture::new();
        fixture.project["companion_resources"][0][field] = json!(key);
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
    let mut fixture = Fixture::new();
    fixture.project["companion_resources"][2]["path"] = json!("other-index.json");
    fixture.bind();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.project["companion_resources"][3]["workspace_index_key"] = json!("evidence");
    fixture.bind();
    assert!(fixture.decode().is_err());
}

/// A Profile requires a Catalog companion while a Catalog cannot silently inherit one.
#[test]
fn requirement_bindings_keep_profile_and_catalog_semantics() {
    for (row, key) in [(0, json!("catalog")), (1, Value::Null), (1, json!("component"))] {
        let mut fixture = Fixture::new();
        fixture.project["companion_resources"][0]["bindings"]["requirements"][row]["resolved_catalog_key"] =
            key;
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
    let mut fixture = Fixture::new();
    fixture.project["companion_resources"][0]["bindings"]["requirements"][1]["requirement_key"] =
        json!("catalog-native");
    fixture.bind();
    assert!(fixture.decode().is_err());
}

/// Evidence/report/index selectors cannot reuse base or other companion kinds.
#[test]
fn selector_kinds_and_base_visibility_are_explicit() {
    for (field, value) in [
        ("evidence_metadata_keys", json!(["evidence"])),
        ("static_report_keys", json!(["links"])),
        ("visible_resource_keys", json!(["links"])),
        ("search_index_key", json!("report")),
    ] {
        let mut fixture = Fixture::new();
        fixture.profile[field] = value;
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
}

/// Privacy constants and source/schedule pairings cannot be widened by an admitted hash.
#[test]
fn privacy_and_explicit_calendar_pairing_are_required() {
    for (field, value) in [
        ("evidence_bytes", json!(true)),
        ("network", json!(true)),
        ("source_text_keys", json!(["source"])),
        ("as_of", json!("2026-10-04")),
    ] {
        let mut fixture = Fixture::new();
        fixture.profile[field] = value;
        fixture.bind();
        assert!(fixture.decode().is_err());
    }
    let mut fixture = Fixture::new();
    fixture.profile["schedule_mode"] = json!("explicit-as-of");
    fixture.profile["schedule_keys"] = json!(["catalog"]);
    fixture.profile["as_of"] = json!("2026-02-30");
    fixture.bind();
    assert!(fixture.decode().is_err());
    fixture.profile["as_of"] = json!("2026-02-28");
    fixture.bind();
    assert!(fixture.decode().is_ok());
}

/// Noncircular final index hash resides only in the decision and is paired with its selector.
#[test]
fn selected_index_requires_a_final_decision_pin_and_discovery_has_none() {
    let mut fixture = Fixture::new();
    fixture.profile["search_index_key"] = json!("search-index");
    fixture.bind();
    assert!(fixture.decode().is_err());
    fixture.decision["search_index_sha256"] = json!("f".repeat(64));
    assert!(fixture.decode().is_ok());
    fixture.project["companion_resources"][4]["expected_sha256"] = json!("f".repeat(64));
    fixture.bind();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.decision["search_index_sha256"] = json!("f".repeat(64));
    assert!(fixture.decode().is_err());
}

/// Duplicate rows, refused provenance, impossible dates and historic proposal pins do not qualify.
#[test]
fn recorded_pairs_and_successor_contract_are_exact() {
    let mut fixture = Fixture::new();
    fixture.decision["policy_decisions"][1] = fixture.decision["policy_decisions"][0].clone();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.decision["policy_decisions"][0]["disposition"] = json!("proposed");
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.decision["policy_decisions"][0]["recorded_on"] = json!("2026-02-30");
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.decision["proposal_index_sha256"] =
        json!("d52da047f0166d9d8cd002221ee9608577f935e7c803969a178b91e797801f49");
    assert!(fixture.decode().is_err());
}

/// The original fifteen roles remain bounded; a companion kind cannot masquerade as a base role.
#[test]
fn native_role_identity_pairing_stays_original() {
    let mut fixture = Fixture::new();
    fixture.project["resources"][0]["role"] = json!("local-evidence");
    fixture.bind();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.project["resources"][2]["native_identity"]["model"] = json!("profile");
    fixture.bind();
    assert!(fixture.decode().is_err());
    let mut fixture = Fixture::new();
    fixture.project["resources"][2]["native_identity"]["root_id"] = json!("not-a-uuid");
    fixture.bind();
    assert!(fixture.decode().is_err());
}

/// Three configs plus the complete roster preserve the 1,001-original slot refusal before joins.
#[test]
fn complete_roster_slot_limit_is_not_only_a_per_array_cap() {
    let mut fixture = Fixture::new();
    fixture.project["resources"] = Value::Array(
        (0..999).map(|i| base_row(&format!("r{i}"), "lifecycle-record", None)).collect(),
    );
    fixture.project["companion_resources"] = json!([]);
    fixture.profile["visible_resource_keys"] = json!([]);
    fixture.profile["evidence_metadata_keys"] = json!([]);
    fixture.profile["static_report_keys"] = json!([]);
    fixture.bind();
    assert!(fixture.decode().is_err());
}

/// Logical work is charged before identity growth and preserves exact cap behavior.
#[test]
fn work_charge_boundary_refuses_before_next_identity_insert() {
    let mut control = NoopControl;
    let mut budget = TestAdmission::new(&mut control);
    budget.charge(TEST_WORK_LIMIT).expect("exact existing caller allowance");
    let mut identities = IdentityRows::new();
    assert!(identities.admit("k", "label", "file.json", &mut budget).is_err());
    assert!(identities.keys.is_empty());
    assert_eq!(identities.paths, [] as [&str; 0]);
}

/// Finalized profile remains nonnull while the separate build record supplies no final read hash.
#[test]
fn separate_build_intent_admits_only_inert_exact_destination_and_roster() {
    let mut fixture = Fixture::new();
    let intent = fixture.build_intent();
    let mut control = NoopControl;
    let mut admission = TestAdmission::new(&mut control);
    let value =
        fixture.decode_intent_with(&intent, &mut admission).expect("inert build declaration");
    assert_eq!(value.profile.search_index_key.as_deref(), Some("search-index"));
    assert_eq!(value.intent.capture_roster.resources, value.project.resources);
    assert!(admission.used > 0);
    assert!(fixture.decode().is_err());
    assert!(fixture.decode_intent(&fixture.decision).is_err());
}

/// Complete capture roster order, finalized profile and exact output spelling cannot be replaced.
#[test]
fn build_intent_refuses_order_changes_and_temporary_profile_or_destination() {
    let mut fixture = Fixture::new();
    let intent = fixture.build_intent();
    let mut reordered = intent.clone();
    reordered["capture_roster"]["resources"].as_array_mut().expect("roster").swap(0, 1);
    assert!(fixture.decode_intent(&reordered).is_err());
    let mut changed = intent.clone();
    changed["destination"]["path"] = json!("another.json");
    assert!(fixture.decode_intent(&changed).is_err());
    let mut temporary = intent;
    temporary["visibility_profile"]["search_index_key"] = Value::Null;
    assert!(fixture.decode_intent(&temporary).is_err());
}

/// Full tuple declaration equality rejects stale hash/identity/context and duplicate native IDs.
#[test]
fn build_source_tuples_bind_current_declared_roster_without_native_claims() {
    let mut fixture = Fixture::new();
    let intent = fixture.build_intent();
    for (field, value) in [
        ("artifact_raw_sha256", json!("b".repeat(64))),
        ("source_key", json!("component")),
        ("lifecycle_key", json!("catalog")),
        ("source_span", json!({"start_byte":4,"end_byte":4})),
    ] {
        let mut changed = intent.clone();
        changed["search_sources"][0][field] = value;
        assert!(fixture.decode_intent(&changed).is_err());
    }
    let mut duplicate = intent;
    let tuple = duplicate["search_sources"][0].clone();
    duplicate["search_sources"].as_array_mut().expect("sources").push(tuple);
    assert!(fixture.decode_intent(&duplicate).is_err());
}

/// Build rows need separate build/source/index scope and reject foreign read-decision substitution.
#[test]
fn build_scope_cannot_inherit_read_decisions_or_self_pin_fields() {
    let mut fixture = Fixture::new();
    let mut intent = fixture.build_intent();
    intent["policy_decisions"] = fixture.decision["policy_decisions"].clone();
    assert!(fixture.decode_intent(&intent).is_err());
    let mut intent = fixture.build_intent();
    intent["completed_index_sha256"] = json!("f".repeat(64));
    assert!(fixture.decode_intent(&intent).is_err());
}

/// Synthetic sticky cancellation carrier; it supplies no clock, original, or authority proof.
struct Stopped;
impl WorkControl for Stopped {
    /// Preserve cancellation at the very first actual decoder checkpoint.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    }
    /// Keep the observed cancellation sticky across all later attempted fences.
    fn interruption(&self) -> Option<Interruption> {
        Some(Interruption::CancelRequested)
    }
}

/// Cancellation precedes hash/schema work and is not converted to an ordinary policy error.
#[test]
fn already_stopped_decode_preserves_typed_interruption() {
    let fixture = Fixture::new();
    let project = encoded(&fixture.project);
    let profile = encoded(&fixture.profile);
    let decision = encoded(&fixture.decision);
    assert!(matches!(
        decode_server_declarations(
            &project,
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            &mut TestAdmission::new(&mut Stopped)
        ),
        Err(WorkError::Interrupted(Interruption::CancelRequested))
    ));
}

/// Per-config raw size refusal precedes schema work even for bounded valid-looking whitespace.
#[test]
fn complete_config_raw_size_cap_is_checked_before_schema_or_typed_growth() {
    let fixture = Fixture::new();
    let project = encoded(&fixture.project);
    let profile = encoded(&fixture.profile);
    let mut decision = encoded(&fixture.decision);
    decision.resize(CONFIG_BYTES + 1, b' ');
    assert!(
        decode_server_declarations(
            &project,
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            &mut TestAdmission::new(&mut NoopControl)
        )
        .is_err()
    );
}

/// Zero/ten-MiB evidence size assertions are valid while the next byte is outside the closed profile.
#[test]
fn local_evidence_size_is_a_bounded_assertion_not_content_validation() {
    let mut fixture = Fixture::new();
    fixture.project["companion_resources"][1]["expected_size"] = json!(10_485_760);
    fixture.bind();
    assert!(fixture.decode().is_ok());
    fixture.project["companion_resources"][1]["expected_size"] = json!(10_485_761);
    fixture.bind();
    assert!(fixture.decode().is_err());
}

/// Exact unchanged production owner work ceiling used only by component regression callers.
const TEST_WORK_LIMIT: usize = 100_000;

/// Actual component-adapter first cause, distinct from ordinary codec policy errors.
#[derive(Clone)]
enum TestStop {
    /// Safe actual control/admission error, preserved with its authored code and fields.
    Failed(Error),
    /// Actual typed caller interruption, never converted into a policy classification.
    Interrupted(Interruption),
}

impl TestStop {
    /// Retain the first actual adapter failure without consuming or normalizing it.
    fn from_error(error: WorkError) -> Self {
        match error {
            WorkError::Failed(error) => Self::Failed(error),
            WorkError::Interrupted(reason) => Self::Interrupted(reason),
        }
    }
    /// Replay that original safe cause at later adapter fences without probing control.
    fn error(&self) -> WorkError {
        match self {
            Self::Failed(error) => WorkError::Failed(error.clone()),
            Self::Interrupted(reason) => WorkError::Interrupted(*reason),
        }
    }
}

/// Caller-owned component ledger borrowing one actual `WorkControl` for all codec calls.
/// It supplies no captured original, proof, native approval or production owner factory.
struct TestAdmission<'a> {
    /// Monotonic admitted work, including actual failed ordinary codec phases.
    used: usize,
    /// First actual capacity/control failure only; ordinary codec errors never enter it.
    stop: Option<TestStop>,
    /// One original caller control; no clock, deadline or replacement wrapper is created.
    control: &'a mut dyn WorkControl,
}

impl<'a> TestAdmission<'a> {
    /// Establish an explicit test caller ledger before decoder invocation, not inside the codec.
    fn new(control: &'a mut dyn WorkControl) -> Self {
        Self { used: 0, stop: None, control }
    }
    /// Store only the first actual callback failure, preserving its original classification.
    fn latch(&mut self, error: WorkError) -> WorkError {
        if self.stop.is_none() {
            self.stop = Some(TestStop::from_error(error));
        }
        self.stop.as_ref().expect("actual test adapter first cause").error()
    }
}

impl DeclarationAdmission for TestAdmission<'_> {
    /// Precharge the same existing ledger under the exact unchanged 100000 work ceiling.
    fn charge(&mut self, amount: usize) -> WorkResult<()> {
        self.checkpoint(Stage::ValidateResource)?;
        let Some(next) = self.used.checked_add(amount).filter(|next| *next <= TEST_WORK_LIMIT)
        else {
            return Err(self.capacity());
        };
        self.used = next;
        Ok(())
    }
    /// Observe the original control result/interruption and retain only actual callback stops.
    fn checkpoint(&mut self, stage: Stage) -> WorkResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.error());
        }
        if let Err(error) = self.control.checkpoint(stage, ProgressUpdate::Unchanged) {
            return Err(self.latch(error));
        }
        if let Some(reason) = self.control.interruption() {
            return Err(self.latch(WorkError::Interrupted(reason)));
        }
        Ok(())
    }
    /// Latch capacity before a postphase callback so a later caller stop cannot replace it.
    fn capacity(&mut self) -> WorkError {
        self.latch(limit_failure())
    }
}

/// Deterministic original control selecting one genuine reached checkpoint by ordinal.
struct PhaseControl {
    /// Actual producer stages reached in chronological order, without a surrogate phase timer.
    stages: Vec<Stage>,
    /// One-based actual checkpoint to stop, calibrated by a complete ordinary baseline run.
    stop_at: Option<usize>,
    /// Select actual safe control failure rather than typed interruption at that boundary.
    fail: bool,
    /// First actual control stop; retained across later attempted probes.
    stopped: Option<TestStop>,
}

impl PhaseControl {
    /// Record ordinary producer fences or stop at one genuinely calibrated occurrence.
    fn new(stop_at: Option<usize>, fail: bool) -> Self {
        Self { stages: Vec::new(), stop_at, fail, stopped: None }
    }
}

impl WorkControl for PhaseControl {
    /// Record exact passed stages and issue a real caller failure only at the selected fence.
    fn checkpoint(&mut self, stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if let Some(stop) = &self.stopped {
            return Err(stop.error());
        }
        self.stages.push(stage);
        if self.stop_at == Some(self.stages.len()) {
            let error = if self.fail {
                WorkError::Failed(Error::new(
                    "test-original-control-failed",
                    "Original caller failure.",
                    false,
                ))
            } else {
                WorkError::Interrupted(Interruption::CancelRequested)
            };
            self.stopped = Some(TestStop::from_error(error));
            return Err(self.stopped.as_ref().expect("actual phase control stop").error());
        }
        Ok(())
    }
    /// Expose the actual observed interruption; ordinary failed control remains a Failed error.
    fn interruption(&self) -> Option<Interruption> {
        match &self.stopped {
            Some(TestStop::Interrupted(reason)) => Some(*reason),
            _ => None,
        }
    }
}

/// Assert the exact actual original caller stop instead of a weaker generic refusal.
fn assert_phase_stop<T>(result: WorkResult<T>, failed: bool) {
    match result {
        Err(WorkError::Failed(error)) if failed => {
            assert_eq!(error.code, "test-original-control-failed");
        }
        Err(WorkError::Interrupted(reason)) if !failed => {
            assert_eq!(reason, Interruption::CancelRequested);
        }
        _ => panic!("the genuine original phase stop must win"),
    }
}

/// Real malformed raw parsing on both distinct ports reaches the same original post-fence.
#[test]
fn actual_malformed_both_port_post_fences_preserve_original_control_stops() {
    let mut fixture = Fixture::new();
    let intent = fixture.build_intent();
    let profile = encoded(&fixture.profile);
    let decision = encoded(&fixture.decision);
    let intent = encoded(&intent);
    let raw = b"{";
    for offline in [false, true] {
        let mut baseline = PhaseControl::new(None, false);
        {
            let mut admission = TestAdmission::new(&mut baseline);
            let result = if offline {
                decode_build_intent(
                    raw,
                    &profile,
                    &intent,
                    &sha256_hex(&intent),
                    &sha256_hex(&profile),
                    &mut admission,
                )
                .map(|_| ())
            } else {
                decode_server_declarations(
                    raw,
                    &profile,
                    &decision,
                    &sha256_hex(&decision),
                    &sha256_hex(&profile),
                    &mut admission,
                )
                .map(|_| ())
            };
            assert!(result.is_err());
            assert!(admission.used > 0);
            assert!(admission.stop.is_none());
        }
        let final_parse = baseline
            .stages
            .iter()
            .rposition(|stage| *stage == Stage::ReadIndex)
            .expect("actual malformed strict post")
            + 1;
        assert_eq!(baseline.stages.last(), Some(&Stage::RetainPrepared));
        for failed in [false, true] {
            let mut control = PhaseControl::new(Some(final_parse), failed);
            let mut admission = TestAdmission::new(&mut control);
            let result = if offline {
                decode_build_intent(
                    raw,
                    &profile,
                    &intent,
                    &sha256_hex(&intent),
                    &sha256_hex(&profile),
                    &mut admission,
                )
                .map(|_| ())
            } else {
                decode_server_declarations(
                    raw,
                    &profile,
                    &decision,
                    &sha256_hex(&decision),
                    &sha256_hex(&profile),
                    &mut admission,
                )
                .map(|_| ())
            };
            assert_phase_stop(result, failed);
            assert!(admission.used > 0);
        }
    }
}

/// Actual schema compile, schema validity and typed failures each reach unconditional fences.
#[test]
fn actual_schema_compile_validate_and_typed_failures_keep_original_post_fences() {
    for (schema, typed) in [
        (r#"{"type":"not-a-schema-type"}"#, false),
        (r#"{"type":"string"}"#, false),
        (r#"{"type":"integer"}"#, true),
    ] {
        let mut baseline = PhaseControl::new(None, false);
        {
            let mut admission = TestAdmission::new(&mut baseline);
            let result = if typed {
                decode_config::<String>(b"7", schema, &mut admission).map(|_| ())
            } else {
                decode_config::<Value>(b"7", schema, &mut admission).map(|_| ())
            };
            assert!(result.is_err());
            assert!(admission.used > 0);
            assert!(admission.stop.is_none());
        }
        let stage = if typed { Stage::PrepareDomain } else { Stage::ValidateResource };
        let post = baseline
            .stages
            .iter()
            .rposition(|seen| *seen == stage)
            .expect("actual ordinary failure post")
            + 1;
        for failed in [false, true] {
            let mut control = PhaseControl::new(Some(post), failed);
            let mut admission = TestAdmission::new(&mut control);
            let result = if typed {
                decode_config::<String>(b"7", schema, &mut admission).map(|_| ())
            } else {
                decode_config::<Value>(b"7", schema, &mut admission).map(|_| ())
            };
            assert_phase_stop(result, failed);
        }
    }
}

/// Real semantic foreign-key refusal keeps failed work and observes original late caller stops.
#[test]
fn actual_semantic_failure_has_same_original_post_phase_and_failed_work() {
    let mut fixture = Fixture::new();
    fixture.project["resources"][0]["lifecycle_key"] = json!("missing");
    fixture.bind();
    let mut baseline = PhaseControl::new(None, false);
    {
        let mut admission = TestAdmission::new(&mut baseline);
        assert!(fixture.decode_with(&mut admission).is_err());
        assert!(admission.used > 0);
        assert!(admission.stop.is_none());
    }
    let post = baseline
        .stages
        .iter()
        .rposition(|stage| *stage == Stage::ValidateResource)
        .expect("actual semantic post")
        + 1;
    for failed in [false, true] {
        let mut control = PhaseControl::new(Some(post), failed);
        let mut admission = TestAdmission::new(&mut control);
        assert_phase_stop(fixture.decode_with(&mut admission), failed);
        assert!(admission.used > 0);
    }
}

/// Ordinary failed work persists when both actual decoder ports consume the same caller ledger.
#[test]
fn failed_and_successful_both_ports_share_one_monotonic_caller_ledger() {
    let fixture = Fixture::new();
    let mut control = NoopControl;
    let mut admission = TestAdmission::new(&mut control);
    let profile = encoded(&fixture.profile);
    let decision = encoded(&fixture.decision);
    assert!(
        decode_server_declarations(
            b"{",
            &profile,
            &decision,
            &sha256_hex(&decision),
            &sha256_hex(&profile),
            &mut admission
        )
        .is_err()
    );
    let failed_work = admission.used;
    assert!(failed_work > 0);
    assert!(admission.stop.is_none());
    let server =
        fixture.decode_with(&mut admission).expect("same caller after ordinary malformed input");
    assert_eq!(server.profile.search_index_key, None);
    assert!(admission.used > failed_work);
    let server_work = admission.used;
    let mut build = Fixture::new();
    let intent = build.build_intent();
    let offline =
        build.decode_intent_with(&intent, &mut admission).expect("same caller distinct build port");
    assert_eq!(offline.profile.search_index_key.as_deref(), Some("search-index"));
    assert!(admission.used > server_work);
    assert!(admission.used <= TEST_WORK_LIMIT);
}

/// Genuine prior work reaches the unchanged cap, and first capacity blocks later caller probes.
#[test]
fn original_capacity_before_decode_is_never_replaced_by_later_caller_stop() {
    let fixture = Fixture::new();
    let mut control = PhaseControl::new(Some(3), true);
    let mut admission = TestAdmission::new(&mut control);
    admission.charge(TEST_WORK_LIMIT).expect("unchanged exact caller allowance");
    let capacity = admission.charge(1).expect_err("actual next unit exceeds original allowance");
    assert!(
        matches!(capacity, WorkError::Failed(error) if error.code == "mcp-core-limit-exceeded")
    );
    let result = fixture.decode_with(&mut admission);
    assert!(
        matches!(result, Err(WorkError::Failed(error)) if error.code == "mcp-core-limit-exceeded")
    );
    assert_eq!(admission.used, TEST_WORK_LIMIT);
    drop(admission);
    assert_eq!(control.stages.len(), 2);
}

/// Real complete successful codec result is discarded when the original final fence stops.
#[test]
fn actual_complete_success_is_discarded_at_original_final_acceptance_fence() {
    let fixture = Fixture::new();
    let mut baseline = PhaseControl::new(None, false);
    {
        let mut admission = TestAdmission::new(&mut baseline);
        fixture.decode_with(&mut admission).expect("actual complete inert codec baseline");
    }
    assert_eq!(baseline.stages.last(), Some(&Stage::RetainPrepared));
    for failed in [false, true] {
        let mut control = PhaseControl::new(Some(baseline.stages.len()), failed);
        let mut admission = TestAdmission::new(&mut control);
        assert_phase_stop(fixture.decode_with(&mut admission), failed);
        assert!(admission.used > 0);
    }
}

/// Complete quoted/chunk-crossing operands retain maintained parse/hash behavior without authority.
#[test]
fn full_quote_unicode_chunk_crossing_inputs_preserve_actual_strict_and_raw_hash_semantics() {
    let quoted = br#"{"value":"[{},:\\\"","unicode":"\ud83d\ude00"}"#.to_vec();
    let mut crossing = vec![b' '; WORK_BYTES - 3];
    crossing.extend(br#"["\"quoted", "\u0061", "\ud83d\ude00", 7]"#);
    for raw in [quoted, crossing] {
        let expected =
            strict_value(&raw, CONFIG_BYTES).expect("actual maintained valid raw oracle");
        let expected_hash = sha256_hex(&raw);
        let mut control = NoopControl;
        let mut admission = TestAdmission::new(&mut control);
        assert_eq!(
            parse_config(&raw, &mut admission).expect("actual bounded complete raw parse"),
            expected
        );
        assert_eq!(
            bounded_hash(&raw, &mut admission).expect("same original raw byte hash"),
            expected_hash
        );
        assert!(admission.used > 0);
        assert!(admission.stop.is_none());
    }
}

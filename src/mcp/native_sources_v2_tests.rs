//! Genuine file-backed controls for the bounded /2 native/source/offline consumer.
//!
//! Synthetic owner records are inert assertions. Native facts come only from maintained
//! validators, complete real lifecycle histories, actual held originals and the real factory.
//! These controls do not grant human, client, corpus, platform, publication or F20 acceptance.

use super::super::requirement_walk;
use super::*;
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{
    self, APPROVAL_POLICY_VERSION, ApprovalPolicy, ArtifactFingerprint, DeclaredRole,
    FingerprintSet, LifecycleState, NamedHash, Party, PolicyIdentity, ReviewSchedule,
    RoleRequirement, SCHEMA_VERSION, SeparationRules, TimezonePolicy, TransitionEvent,
};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{Interruption, NoopControl, ProgressUpdate, WorkError};
use chrono::NaiveDate;
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;

/// Exact engineering decision binding already enforced by the frozen inert codec.
const PROPOSAL: &str = "365ac7ddbb2cb1058efdfc211a5a525babcbbd69c30557a5113126e0c616cde8";
/// Two source CRLF lines followed by one unterminated original line.
const SOURCE: &[u8] = b"Alpha control\r\nBeta control\r\nGamma control";

/// Independent full native/lifecycle originals and explicit authored search tuple expectations.
struct Fixture {
    /// Real root lifetime retained until every actual captured proof has dropped.
    project: TempDir,
    /// Separate real operator-selected fixed-intent root lifetime.
    external: TempDir,
    /// Genuine native Catalog; no captured/currentness fields are constructed.
    catalog: Value,
    /// Genuine native Component Definition with component and capability requirements.
    component: Value,
    /// Complete explicit discovery assertion, including hidden/opaque members when added.
    discovery: Value,
    /// Complete explicit visibility assertion with the final nonnull destination selector.
    profile: Value,
    /// Separate complete build intent, never a final server disclosure decision.
    intent: Value,
    /// Exact selected intent raw digest after explicit fixture publication.
    intent_pin: String,
    /// Exact selected profile raw digest after explicit fixture publication.
    profile_pin: String,
}
impl Fixture {
    /// Write ordinary local files, maintained intrinsic approval and complete selected configs.
    fn new() -> Self {
        let mut fixture = Self {
            project: tempfile::tempdir().unwrap(),
            external: tempfile::tempdir().unwrap(),
            catalog: catalog(),
            component: component(),
            discovery: json!({}),
            profile: profile(),
            intent: json!({}),
            intent_pin: String::new(),
            profile_pin: String::new(),
        };
        fixture.write_native(true);
        fixture.bind();
        fixture
    }
    /// Pass only a known ordinary canonical test root to the production raw-spelling API.
    fn project_root(&self) -> std::path::PathBuf {
        self.project.path().canonicalize().unwrap()
    }
    /// Retain a separate canonical real fixed-purpose root, never a staged project.
    fn external_root(&self) -> std::path::PathBuf {
        self.external.path().canonicalize().unwrap()
    }
    /// Publish genuine native files and a fully validated intrinsic lifecycle record.
    fn write_native(&mut self, approved: bool) {
        fs::write(self.project.path().join("source.txt"), SOURCE).unwrap();
        fs::write(self.project.path().join("catalog.json"), encoded(&self.catalog)).unwrap();
        fs::write(self.project.path().join("component.json"), encoded(&self.component)).unwrap();
        let record = lifecycle(self.project.path(), approved);
        record::validate(&record).expect("genuine complete fixture record");
        fs::write(self.project.path().join("lifecycle.json"), serde_json::to_vec(&record).unwrap())
            .unwrap();
    }
    /// Rebind complete discovery/config/raw expectations explicitly; no production retry exists.
    fn bind(&mut self) {
        let hidden = self
            .discovery
            .get("resources")
            .and_then(Value::as_array)
            .map(|rows| {
                rows.iter()
                    .filter(|row| row["key"] == "opaque" || row["key"] == "app")
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut resources = vec![
            resource(
                "catalog",
                "oscal-catalog-artifact",
                "catalog.json",
                Some(&self.catalog),
                self.project.path(),
            ),
            resource(
                "component",
                "oscal-component-artifact",
                "component.json",
                Some(&self.component),
                self.project.path(),
            ),
            resource("source", "policy-source", "source.txt", None, self.project.path()),
            resource("lifecycle", "lifecycle-record", "lifecycle.json", None, self.project.path()),
        ];
        resources.extend(hidden);
        self.discovery = json!({"schema_version":"forge.mcp-project/2","project_key":"synthetic-native",
            "resources":resources,"companion_resources":[{"kind":"search-index","key":"index",
                "path":"index.json","citation_label":"index-label","index_format":"forge.mcp-search-index/1"}]});
        let discovery = encoded(&self.discovery);
        self.profile["discovery_sha256"] = json!(sha256_hex(&discovery));
        let profile = encoded(&self.profile);
        self.profile_pin = sha256_hex(&profile);
        let mut tuples = self.authored_tuples();
        tuples.reverse(); // authorial order is valid; actual producer must conserve the whole set.
        self.intent = json!({"schema_version":"forge.mcp-index-build-intent/1","intent_key":"synthetic-build",
            "project_key":"synthetic-native","discovery_sha256":sha256_hex(&discovery),"profile_sha256":self.profile_pin,
            "proposal_index_sha256":PROPOSAL,"operator_boundary":"operator-selected-declared-owner-record",
            "visibility_profile":self.profile,"capture_roster":{"resources":self.discovery["resources"],
                "companion_resources":self.discovery["companion_resources"]},"search_sources":tuples,
            "destination":{"key":"index","path":"index.json","index_format":"forge.mcp-search-index/1",
                "tokenizer_identity":"declared-unicode-build-1"},
            "policy_decisions":decision_rows(&sha256_hex(&discovery),&self.profile_pin)});
        fs::write(self.project.path().join("forge.mcp.json"), discovery).unwrap();
        fs::write(self.project.path().join("forge.mcp.visibility.json"), profile).unwrap();
        self.write_intent();
    }
    /// Publish only the deliberate changed intent for native tuple mismatch controls.
    fn write_intent(&mut self) {
        let raw = encoded(&self.intent);
        self.intent_pin = sha256_hex(&raw);
        fs::write(self.external.path().join("forge.mcp.index-build-intent.json"), raw).unwrap();
    }
    /// Explicit complete expected native locations; no expected tuple comes from the candidate.
    fn authored_tuples(&self) -> Vec<Value> {
        let locations = [
            ("catalog", "/catalog/groups/0/groups/0/controls/0", 2, (15_u64, 27_u64)),
            ("catalog", "/catalog/groups/0/groups/1/controls/0", 1, (0, 13)),
            ("catalog", "/catalog/groups/0/groups/1/controls/0/controls/0", 2, (15, 27)),
            ("catalog", "/catalog/controls/0", 3, (29, 42)),
            (
                "component",
                "/component-definition/components/0/control-implementations/0/implemented-requirements/0",
                1,
                (0, 13),
            ),
            (
                "component",
                "/component-definition/capabilities/0/control-implementations/0/implemented-requirements/0",
                3,
                (29, 42),
            ),
        ];
        locations.into_iter().map(|(key,pointer,_line,(start,end))| {
            let native = if key == "catalog" { &self.catalog } else { &self.component };
            let node = native.pointer(pointer).expect("actual authored native location");
            let id = node[if key == "catalog" { "id" } else { "uuid" }].as_str().unwrap();
            let control = if key == "catalog" { id } else { node["control-id"].as_str().unwrap() };
            let row = self.discovery["resources"].as_array().unwrap().iter().find(|row|row["key"]==key).unwrap();
            json!({"artifact_key":key,"artifact_role":row["role"],"artifact_identity":row["native_identity"],
                "artifact_raw_sha256":row["expected_sha256"],"native_id":id,"control_id":control,
                "native_pointer":pointer,"source_key":"source","source_raw_sha256":sha256_hex(SOURCE),
                "source_span":{"start_byte":start,"end_byte":end},"lifecycle_key":"lifecycle"})
        }).collect()
    }
    /// Invoke only the genuine purpose-qualified native consumer with one original caller.
    fn load<'control, C: WorkControl + ?Sized>(
        &self,
        control: &'control mut C,
    ) -> WorkResult<OfflineBuildGate<'control, C>> {
        capture_index_build_intent(
            &self.project_root(),
            &self.external_root(),
            &self.intent_pin,
            &self.profile_pin,
            control,
        )
    }
}

/// Exact original trace relation; private section/title never becomes a citation token.
fn props(line: usize) -> Value {
    json!([{"name":"source-file","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"source.txt"},
        {"name":"source-section","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":"Private section"},
        {"name":"source-line","ns":crate::oscal::trace_embedding::FORGE_TRACE_NS,"value":line.to_string()}])
}
/// Genuine native control with complete independently authored trace metadata.
fn control(id: &str, line: usize) -> Value {
    json!({"id":id,"title":"Private native control","props":props(line)})
}
/// Genuine nested groups/controls; complete native inventory has four distinct controls.
fn catalog() -> Value {
    let mut parent = control("c-parent", 1);
    parent["controls"] = json!([control("c-child", 2)]);
    json!({"catalog":{"uuid":"11111111-1111-4111-8111-111111111111", "metadata":metadata(),
        "groups":[{"id":"g-a","title":"Outer group","groups":[{"id":"g-b","title":"Inner group",
            "controls":[control("c-nested",2)]},{"id":"g-c","title":"Sibling control group",
            "controls":[parent]}]}],"controls":[control("z-last",3)]}})
}
/// Genuine complete metadata under the maintained pinned OSCAL compatibility policy.
fn metadata() -> Value {
    json!({"title":"Private native title","last-modified":"2026-09-01T00:00:00Z","version":"1","oscal-version":"1.2.3"})
}
/// Native implementation assembly for both genuine Component and Capability containers.
fn implementation(uuid: &str, requirement: &str, line: usize) -> Value {
    json!({"uuid":uuid,"source":"urn:synthetic-framework","description":"Private implementation rationale",
        "implemented-requirements":[{"uuid":requirement,"control-id":"c-parent","description":"Private implemented prose","props":props(line)}]})
}
/// Genuine schema-shaped components and capabilities with distinct actual native requirement UUIDs.
fn component() -> Value {
    json!({"component-definition":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":metadata(),
        "components":[{"uuid":"33333333-3333-4333-8333-333333333333","type":"software","title":"Private component",
            "description":"Private description","control-implementations":[implementation("66666666-6666-4666-8666-666666666666","44444444-4444-4444-8444-444444444444",1)]}],
        "capabilities":[{"uuid":"77777777-7777-4777-8777-777777777777","name":"Private capability","description":"Private capability prose",
            "control-implementations":[implementation("88888888-8888-4888-8888-888888888888","55555555-5555-4555-8555-555555555555",3)]}]}})
}
/// Actual full declaration row pinned to the independently written real fixture bytes.
fn resource(key: &str, role: &str, path: &str, native: Option<&Value>, root: &Path) -> Value {
    let model = if key == "catalog" { "catalog" } else { "component-definition" };
    json!({"key":key,"role":role,"path":path,"expected_sha256":sha256_hex(&fs::read(root.join(path)).unwrap()),
        "native_identity":native.map(|value|json!({"model":model,"root_id":value[model]["uuid"],
            "document_version":value[model]["metadata"]["version"],"oscal_version":value[model]["metadata"]["oscal-version"]})),
        "lifecycle_key":if key=="lifecycle" {None} else {Some("lifecycle")},"citation_label":format!("{key}-label")})
}
/// Final nonnull selected profile, with no source bytes, evidence bytes, network or process scope.
fn profile() -> Value {
    json!({"schema_version":"forge.mcp-visibility/2","project_key":"synthetic-native","discovery_sha256":"0".repeat(64),
        "enabled_tools":["list_policies","search_requirements","get_requirement","get_artifact_status"],
        "visible_resource_keys":["catalog","component","source"],"noncurrent_metadata_keys":[],"source_text_keys":[],"schedule_keys":[],
        "evidence_metadata_keys":[],"static_report_keys":[],"search_index_key":"index","identity_disclosure":"bounded-token-only",
        "approved_current_basis":"caller-established-recorded-policy","source_text_mode":"omit","schedule_mode":"omit","as_of":null,
        "report_content_mode":"minimized-metadata","evidence_bytes":false,"network":false,"process_execution":false,
        "resource_families":["inventory","artifact-status","requirement-metadata"],"index_fallback_mode":"refuse"})
}
/// Six complete inert build/source/index provenance assertions, not real owner authority.
fn decision_rows(discovery: &str, profile: &str) -> Value {
    Value::Array([("F20-BUILD","product"),("F20-BUILD","security"),("F20-SOURCE","product"),("F20-SOURCE","security"),
        ("F20-INDEX","product"),("F20-INDEX","security")].into_iter().map(|(subject,role)| json!({"subject":subject,"role":role,
            "disposition":"approved","declared_owner_key":"synthetic-owner","recorded_on":"2026-10-04",
            "source_record_key":"synthetic-record","source_record_sha256":"e".repeat(64),"proposal_index_sha256":PROPOSAL,
            "profile_sha256":profile,"discovery_sha256":discovery})).collect())
}
/// Construct a genuine complete two-generated-artifact neutral lifecycle record and event chain.
fn lifecycle(root: &Path, approved: bool) -> record::LifecycleRecord {
    let generated = [
        ("catalog.json", "catalog", "11111111-1111-4111-8111-111111111111"),
        ("component.json", "component-definition", "22222222-2222-4222-8222-222222222222"),
    ]
    .into_iter()
    .map(|(path, kind, id)| ArtifactFingerprint {
        path: path.into(),
        sha256: sha256_hex(&fs::read(root.join(path)).unwrap()),
        oscal_type: Some(kind.into()),
        root_uuid: Some(id.into()),
    })
    .collect::<Vec<_>>();
    let mut record = record::LifecycleRecord {
        schema_version: SCHEMA_VERSION.into(),
        policy: PolicyIdentity {
            policy_key: "synthetic-policy".into(),
            version_key: "version-1".into(),
            title: "Private record title".into(),
            owner_keys: vec!["owner".into()],
            source: ArtifactFingerprint {
                path: "source.txt".into(),
                sha256: sha256_hex(SOURCE),
                oscal_type: None,
                root_uuid: None,
            },
            generated_artifacts: generated,
        },
        parties: vec![
            Party { key: "owner".into(), roles: vec![DeclaredRole::Owner] },
            Party { key: "reviewer".into(), roles: vec![DeclaredRole::Reviewer] },
            Party { key: "approver".into(), roles: vec![DeclaredRole::Approver] },
        ],
        approval_policy: ApprovalPolicy {
            schema_version: APPROVAL_POLICY_VERSION.into(),
            required_roles: vec![
                RoleRequirement { role: DeclaredRole::Reviewer, count: 1 },
                RoleRequirement { role: DeclaredRole::Approver, count: 1 },
            ],
            separation: SeparationRules::default(),
        },
        review: ReviewSchedule {
            cadence_days: 30,
            next_review_date: NaiveDate::from_ymd_opt(2026, 12, 1).unwrap(),
            due_soon_days: 7,
            timezone_policy: TimezonePolicy::DateOnly,
        },
        state: LifecycleState::Draft,
        replaced_by: None,
        history: vec![],
    };
    if approved {
        let fingerprints = FingerprintSet {
            source_sha256: sha256_hex(SOURCE),
            generated_artifacts: record
                .policy
                .generated_artifacts
                .iter()
                .map(|row| NamedHash { path: row.path.clone(), sha256: row.sha256.clone() })
                .collect(),
        };
        event(&mut record, LifecycleState::InReview, &fingerprints);
        event(&mut record, LifecycleState::Approved, &fingerprints);
    }
    record
}
/// Use actual maintained deterministic event ID computation with full intrinsic record context.
fn event(
    record: &mut record::LifecycleRecord,
    next: LifecycleState,
    fingerprints: &FingerprintSet,
) {
    let sequence = u32::try_from(record.history.len() + 1).unwrap();
    let (actor, role) = if next == LifecycleState::InReview {
        ("reviewer", DeclaredRole::Reviewer)
    } else {
        ("approver", DeclaredRole::Approver)
    };
    let mut event = TransitionEvent {
        sequence,
        event_id: String::new(),
        legacy_event_id: None,
        previous_state: record.state,
        next_state: next,
        actor_key: actor.into(),
        declared_role: role,
        timestamp: format!("2026-09-01T{sequence:02}:00:00Z"),
        rationale: "Private rationale".into(),
        fingerprints: fingerprints.clone(),
        assertions: vec![],
        impact_finding_ids: vec![],
        replacement: None,
    };
    event.event_id = record::event_id(record, &event).unwrap();
    record.history.push(event);
    record.state = next;
}
/// Fixture-only serialization; actual production decode/hash phases consume raw held originals.
fn encoded(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
/// Refuse only ordinary whole-gate outcomes; capacity/control must never pass a negative control.
fn refused<C: WorkControl + ?Sized>(result: WorkResult<OfflineBuildGate<'_, C>>) {
    match result {
        Ok(OfflineBuildGate::Unavailable) => {}
        Err(WorkError::Failed(error)) if error.code == "mcp-core-unavailable" => {}
        _ => panic!(
            "expected ordinary whole-gate refusal, never capacity/control or partial success"
        ),
    }
}

/// Complete genuine nested Catalog and Component/Capability universe conserves six full tuples.
#[test]
fn complete_native_universe_matches_authored_order_and_exact_crlf_spans() {
    let fixture = Fixture::new();
    let mut caller = NoopControl;
    // Validate both complete native baseline assemblies with the maintained official
    // schema/version/semantic predicate before the actual captured factory is invoked.
    for (value, model) in [
        (&fixture.catalog, crate::validate::OscalModelType::Catalog),
        (&fixture.component, crate::validate::OscalModelType::ComponentDefinition),
    ] {
        let report = crate::validate::run_full_validation("genuine offline fixture", value, model)
            .expect("maintained complete native fixture validation");
        assert!(report.is_valid(), "maintained full native fixture must be valid: {model}");
    }
    let OfflineBuildGate::Validated(capture) = fixture.load(&mut caller).unwrap() else {
        panic!("genuine native fixture unavailable")
    };
    assert_eq!(capture.search_sources().len(), 6);
    let mut expected = fixture.authored_tuples();
    expected.sort_by(|a, b| {
        (a["artifact_key"].as_str(), a["native_id"].as_str())
            .cmp(&(b["artifact_key"].as_str(), b["native_id"].as_str()))
    });
    assert_eq!(serde_json::to_value(capture.search_sources()).unwrap(), Value::Array(expected));
    assert_eq!(
        capture
            .sources
            .inventory(0)
            .unwrap()
            .inventory
            .count(crate::mapping::manifest::SubjectType::Control),
        4
    );
    let approved = capture.sources.approved("catalog").unwrap().unwrap();
    assert_eq!(approved.record().unwrap().history.len(), 2);
    assert_eq!(approved.neutral_status().blockers, [] as [std::string::String; 0]);
    capture.verify_inputs().unwrap();
}

/// Omission, extra native ID, coherent changed ID/pointer/span each refuse the entire universe.
#[test]
fn complete_native_tuple_mismatches_never_accept_selected_subsets() {
    for case in 0..5 {
        let mut fixture = Fixture::new();
        match case {
            0 => {
                fixture.intent["search_sources"].as_array_mut().unwrap().pop();
            }
            1 => {
                let mut row = fixture.intent["search_sources"][0].clone();
                row["native_id"] = json!("99999999-9999-4999-8999-999999999999");
                fixture.intent["search_sources"].as_array_mut().unwrap().push(row);
            }
            2 => fixture.intent["search_sources"][0]["control_id"] = json!("different-control"),
            3 => {
                fixture.intent["search_sources"][0]["native_pointer"] = json!(
                    "/component-definition/components/0/control-implementations/0/implemented-requirements/0"
                );
            }
            _ => {
                fixture.intent["search_sources"][0]["source_span"] =
                    json!({"start_byte":0,"end_byte":12});
            }
        }
        fixture.write_intent();
        refused(fixture.load(&mut NoopControl));
    }
}

/// Native-valid private 129 and 4096-byte IDs are conserved; /1's 128-token rule stays separate.
#[test]
fn private_wide_native_ids_are_not_silently_filtered_to_public_dto_width() {
    for width in [129, 4096] {
        let mut fixture = Fixture::new();
        fixture.catalog["catalog"]["controls"][0]["id"] = json!("x".repeat(width));
        fixture.write_native(true);
        fixture.bind();
        let mut caller = NoopControl;
        let OfflineBuildGate::Validated(capture) = fixture.load(&mut caller).unwrap() else {
            panic!("native-valid private width unavailable")
        };
        assert_eq!(capture.search_sources().len(), 6);
        assert!(capture.search_sources().iter().any(|row| row.native_id.len() == width));
    }
}

/// Full native schema/version/semantic validation refuses even coherently repinned intent data.
#[test]
fn maintained_native_invalidity_is_not_promoted_by_config_and_lifecycle_pins() {
    for case in 0..3 {
        let mut fixture = Fixture::new();
        match case {
            0 => {
                fixture.catalog["catalog"]["metadata"]
                    .as_object_mut()
                    .unwrap()
                    .remove("last-modified");
            }
            1 => fixture.catalog["catalog"]["metadata"]["oscal-version"] = json!("1.3.0"),
            _ => {
                fixture.catalog["catalog"]["controls"][0]["links"] =
                    json!([{"href":"#99999999-9999-4999-8999-999999999999"}]);
            }
        }
        // Preserve the real UUID/version identity while repinning the actual complete
        // native/lifecycle tuple for every schema, version and semantic invalidity.
        fixture.write_native(true);
        fixture.bind();
        refused(fixture.load(&mut NoopControl));
    }
}

/// Real Draft records and latest complete approval fingerprints cannot be replaced by old labels.
#[test]
fn unapproved_and_changed_generated_closure_refuse_whole_offline_gate() {
    let mut draft = Fixture::new();
    draft.write_native(false);
    draft.bind();
    refused(draft.load(&mut NoopControl));
    let mut changed = Fixture::new();
    changed.catalog["catalog"]["controls"][0]["title"] = json!("Changed private title");
    fs::write(changed.project.path().join("catalog.json"), encoded(&changed.catalog)).unwrap();
    changed.bind();
    refused(changed.load(&mut NoopControl));
}

/// Present destination cannot become absent merely because the intent supplies no final digest.
#[test]
fn actual_existing_index_destination_refuses_without_overwrite() {
    let fixture = Fixture::new();
    fs::write(fixture.project.path().join("index.json"), b"existing private index").unwrap();
    refused(fixture.load(&mut NoopControl));
    assert_eq!(
        fs::read(fixture.project.path().join("index.json")).unwrap(),
        b"existing private index"
    );
}

/// Hidden opaque actual bytes remain in the complete final original denominator without activation.
#[test]
fn hidden_nonutf8_and_empty_originals_are_retained_and_drift_is_detected() {
    for raw in [&b"\xff\x00"[..], &b""[..]] {
        let mut fixture = Fixture::new();
        fs::write(fixture.project.path().join("opaque.json"), raw).unwrap();
        fixture.discovery["resources"].as_array_mut().unwrap().push(resource(
            "opaque",
            "trace-report",
            "opaque.json",
            None,
            fixture.project.path(),
        ));
        fixture.bind();
        let mut caller = NoopControl;
        let OfflineBuildGate::Validated(capture) = fixture.load(&mut caller).unwrap() else {
            panic!("hidden opaque observation unavailable")
        };
        assert_eq!(capture.search_sources().len(), 6);
        assert!(capture.sources.approved("opaque").unwrap().is_none());
        fs::write(fixture.project.path().join("opaque.json"), b"changed opaque generation")
            .unwrap();
        assert!(capture.verify_inputs().is_err());
    }
}

/// A visible missing applicability domain refuses the whole family, never a supported prefix.
#[test]
fn visible_uncoupled_applicability_domain_refuses_complete_offline_capability() {
    let mut fixture = Fixture::new();
    fs::write(fixture.project.path().join("app.json"), b"{}").unwrap();
    fixture.discovery["resources"].as_array_mut().unwrap().push(resource(
        "app",
        "applicability-manifest",
        "app.json",
        None,
        fixture.project.path(),
    ));
    fixture.profile["visible_resource_keys"].as_array_mut().unwrap().push(json!("app"));
    fixture.bind();
    refused(fixture.load(&mut NoopControl));
}

/// Every actual native cursor agrees with the maintained borrowed identity pointer oracle.
#[test]
fn located_native_catalog_and_component_cursors_have_complete_pointer_parity() {
    for (tree, field) in [(catalog(), "catalog"), (component(), "component-definition")] {
        let mut path = String::with_capacity(4096);
        let mut charge = |_: usize| Ok(());
        requirement_walk::append_located(&mut path, field, &mut charge)
            .unwrap_or_else(|_| panic!("actual native root pointer"));
        let mut count = 0;
        let mut visit = |node: &Value, pointer: &str, _: &mut dyn WorkControl| {
            let found = requirement_walk::pointer_for_node(&tree, node, &mut NoopControl)?;
            assert_eq!(found, pointer);
            assert!(std::ptr::eq(tree.pointer(pointer).unwrap(), node));
            count += 1;
            Ok(())
        };
        let result = if field == "catalog" {
            requirement_walk::located_catalog_nodes(
                &tree[field],
                0,
                1,
                &mut path,
                &mut visit,
                &mut charge,
                &mut NoopControl,
            )
        } else {
            requirement_walk::located_component_nodes(
                &tree[field],
                &mut path,
                &mut visit,
                &mut charge,
                &mut NoopControl,
            )
        };
        result.unwrap_or_else(|_| panic!("complete actual native cursor walk"));
        assert_eq!(count, if field == "catalog" { 4 } else { 2 });
    }
}

/// Escaped components derive only from actual object cursor keys, never a supplied native citation.
#[test]
fn located_cursor_escape_matches_actual_borrowed_pointer_identity() {
    let tree = json!({"a/b~c":{"controls":[{"id":"actual"}]}});
    let (key, root) = tree.as_object().unwrap().iter().next().unwrap();
    let mut path = String::with_capacity(4096);
    let mut charge = |_: usize| Ok(());
    requirement_walk::append_located(&mut path, key, &mut charge)
        .unwrap_or_else(|_| panic!("actual escaped cursor"));
    let mut visit = |node: &Value, pointer: &str, _: &mut dyn WorkControl| {
        assert_eq!(pointer, "/a~1b~0c/controls/0");
        assert_eq!(requirement_walk::pointer_for_node(&tree, node, &mut NoopControl)?, pointer);
        Ok(())
    };
    requirement_walk::located_catalog_nodes(
        root,
        0,
        1,
        &mut path,
        &mut visit,
        &mut charge,
        &mut NoopControl,
    )
    .unwrap_or_else(|_| panic!("pure actual borrowed escaped traversal"));
}

/// One actual immutable caller records phase checkpoints and an optional sticky first stop.
struct Caller {
    /// Full actual chronology used to calibrate a genuine final factory checkpoint.
    stages: Vec<Stage>,
    /// Optional one-based real checkpoint at which this caller stops.
    at: Option<usize>,
    /// Actual injected failure kind, separate from ordinary native/domain invalidity.
    failed: bool,
    /// Actual observed cancellation remains sticky under every original alias.
    interrupted: Option<Interruption>,
}
impl Caller {
    /// Start one genuine caller chronology with no invented capture generation.
    fn new(at: Option<usize>, failed: bool) -> Self {
        Self { stages: vec![], at, failed, interrupted: None }
    }
}
impl WorkControl for Caller {
    /// Observe every stage before returning the exact selected actual caller stop.
    fn checkpoint(&mut self, stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.stages.push(stage);
        if self.at == Some(self.stages.len()) {
            if self.failed {
                return Err(WorkError::Failed(Error::new(
                    "actual-test-control",
                    "Actual control failure",
                    false,
                )));
            }
            self.interrupted = Some(Interruption::CancelRequested);
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// No ordinary codec/native domain error supplies a cancellation assertion.
    fn interruption(&self) -> Option<Interruption> {
        self.interrupted
    }
}

/// Actual complete factory success is discarded at the same original final acceptance fence.
#[test]
fn real_complete_factory_observes_original_final_stop_and_no_controller_replacement() {
    let fixture = Fixture::new();
    let mut baseline = Caller::new(None, false);
    {
        let OfflineBuildGate::Validated(capture) = fixture.load(&mut baseline).unwrap() else {
            panic!("real baseline")
        };
        drop(capture);
    }
    assert_eq!(baseline.stages.last(), Some(&Stage::RetainPrepared));
    let at = baseline.stages.len();
    for failed in [false, true] {
        let mut caller = Caller::new(Some(at), failed);
        let result = fixture.load(&mut caller);
        if failed {
            assert!(
                matches!(result,Err(WorkError::Failed(error))if error.code=="actual-test-control")
            );
        } else {
            assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        }
    }
}

/// Real held generation mutation after native qualification is refused by the final same-owner fence.
#[test]
fn held_policy_source_mutation_after_native_qualification_is_not_recaptured() {
    let fixture = Fixture::new();
    let mut caller = NoopControl;
    let OfflineBuildGate::Validated(capture) = fixture.load(&mut caller).unwrap() else {
        panic!("genuine native baseline")
    };
    fs::write(fixture.project.path().join("source.txt"), b"changed original source bytes").unwrap();
    assert!(capture.verify_inputs().is_err());
    assert_eq!(capture.search_sources().len(), 6);
}

/// Real already-consumed native work reaches the unchanged owner cap before a later caller stop.
#[test]
fn original_capacity_after_native_success_wins_and_blocks_later_control_probe() {
    let fixture = Fixture::new();
    let mut caller = Caller::new(None, true);
    let OfflineBuildGate::Validated(capture) = fixture.load(&mut caller).unwrap() else {
        panic!("genuine complete baseline")
    };
    let before = capture.with_control(|control| {
        let before = control.stages.len();
        control.at = Some(before + 2);
        before
    });
    let mut admission = capture.admission();
    assert!(
        matches!(admission.charge(100_000),Err(WorkError::Failed(error))if error.code=="mcp-capture-capacity")
    );
    assert!(
        matches!(native_requirements_v2::complete(&capture.sources),Err(WorkError::Failed(error))if error.code=="mcp-capture-capacity")
    );
    assert_eq!(capture.with_control(|control| control.stages.len()), before + 1);
}

/// Malformed selectors/root aliases refuse before attempting any missing project or external IO.
#[test]
fn malformed_pins_and_raw_root_spelling_refuse_before_capture() {
    let mut caller = NoopControl;
    let result = capture_index_build_intent(
        Path::new("missing/../unsafe"),
        Path::new("also-missing"),
        "bad-pin",
        "bad-profile",
        &mut caller,
    );
    assert!(matches!(result, Ok(OfflineBuildGate::Unavailable)));
}

/// Genuine first App bridge controls use the same ordinary valid native/root fixture.
#[path = "native_applicability_v2_tests.rs"]
mod applicability;

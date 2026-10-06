//! Genuine complete native originals for the plain authoring evaluator.
//! These controls fabricate no capture/currentness/approval token; Root alone executes them.

use super::*;
use crate::review::decode::{ContractError, ContractLedger};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// Actual persisted native fixture originals, including the complete declared byte cohorts.
struct Fixture {
    _directory: tempfile::TempDir,
    root: PathBuf,
    project: Value,
    pack: Value,
    project_raw: Vec<u8>,
    pack_raw: Vec<u8>,
    report: Vec<u8>,
    app: Vec<u8>,
    framework: Vec<u8>,
    companion: Option<Vec<u8>>,
    mappings: Vec<Vec<u8>>,
    clauses: Vec<Vec<u8>>,
}

/// Preserve deliberately private human evidence in the complete native oracle.
fn review() -> Value {
    json!({"reviewer_key":"human","reviewed_at":"2026-09-01T00:00:00Z",
        "rationale":"Private synthetic rationale — actual complete native evidence."})
}
/// Actual official-schema Catalog, not an in-memory `PreparedAnalysis` stand-in.
fn catalog(id: &str, controls: &[&str]) -> Value {
    json!({"catalog":{"uuid":id,"metadata":{"title":"Native authoring fixture",
        "last-modified":"2026-09-01T00:00:00Z","version":"1.0.0","oscal-version":"1.2.3"},
        "controls":controls.iter().map(|id|json!({"id":id,"title":format!("Control {id}")}))
            .collect::<Vec<_>>()}})
}
/// Save exact fixture originals and return their byte hash.
fn write(path: &Path, value: &Value) -> Vec<u8> {
    let raw = serde_json::to_vec_pretty(value).expect("fixture encode");
    std::fs::write(path, &raw).expect("fixture write");
    raw
}
/// Existing owned fixture locals grouped after genuine native setup; no proof or quota is supplied.
struct NativeFixtureBaseline {
    report: Vec<u8>,
    app: Vec<u8>,
    framework: Vec<u8>,
    companion: Option<Vec<u8>>,
    mappings: Vec<Vec<u8>>,
    baseline: Value,
}
/// Prepare the same real framework, optional Mapping and complete Applicability baseline once.
fn native_baseline(root: &Path, profile: bool, mapping: bool) -> NativeFixtureBaseline {
    let original = catalog("11111111-1111-4111-8111-111111111111", &["c-1", "c-2", "c-3", "c-4"]);
    write(&root.join("framework.json"), &original);
    let companion = if profile {
        let raw = write(&root.join("resolved-catalog.json"), &original);
        write(
            &root.join("framework.json"),
            &json!({"profile":{
            "uuid":"22222222-2222-4222-8222-222222222222","metadata":{
                "title":"Native authoring Profile","last-modified":"2026-09-01T00:00:00Z",
                "version":"1.0.0","oscal-version":"1.2.3"},
            "imports":[{"href":"resolved-catalog.json","include-all":{}}]}}),
        );
        Some(raw)
    } else {
        None
    };
    crate::applicability::execute_init(
        &root.join("framework.json"),
        companion.as_ref().map(|_| root.join("resolved-catalog.json")).as_deref(),
        Some(&root.join("applicability.json")),
    )
    .expect("actual maintained init");
    let mut app: Value = serde_json::from_slice(
        &std::fs::read(root.join("applicability.json")).expect("actual init bytes"),
    )
    .expect("init contract");
    // The explicit synthetic companion is the same complete Catalog used by this Profile.
    if profile {
        app["framework"]["resolved_catalog_attestation"] = json!(true);
    }
    app["reviewers"] = json!([{"key":"human","type":"person","name":"Native Private Reviewer"}]);
    app["decisions"] = json!(
        (1..=4)
            .map(|n| json!({"control_id":format!("c-{n}"),
        "state":"applicable","reviewer_key":"human","reviewed_at":"2026-09-01T00:00:00Z",
        "rationale":format!("Private original decision {n}")}))
            .collect::<Vec<_>>()
    );
    let mut mappings = Vec::new();
    if mapping {
        write(
            &root.join("policy.json"),
            &catalog("33333333-3333-4333-8333-333333333333", &["policy-1"]),
        );
        let source = json!({"schema_version":"forge.mapping-manifest/1","collection":{
            "key":"native-collection","title":"Native reviewed negative relationship","version":"1.0.0",
            "last_modified":"2026-09-01T00:00:00Z"},"reviewers":[{"key":"human","type":"person",
                "name":"Native Private Reviewer"}],"provenance":{"method":"human","matching_rationale":"semantic",
                "status":"complete","mapping_description":"Actual no-relationship fixture.",
                "reviewer_keys":["human"],"reviewed_at":"2026-09-01T00:00:00Z"},"mapping":{
            "key":"native-mapping","scope":"control-only","source":{"type":"catalog",
                "artifact":"policy.json","href":"policy.json"},"target":app["framework"],"maps":[{
                "key":"negative","relationship":"no-relationship","sources":[{"type":"control","id_ref":"policy-1"}],
                "targets":[{"type":"control","id_ref":"c-1"}],"reviewer_key":"human",
                "reviewed_at":"2026-09-01T00:00:00Z","rationale":"Private negative mapping rationale."}]}});
        write(&root.join("mapping-manifest.json"), &source);
        let built = crate::mapping::prepare(&root.join("mapping-manifest.json"), None, false)
            .expect("actual maintained Mapping");
        let raw = built.artifact_json.as_bytes().to_vec();
        std::fs::write(root.join("mapping.json"), &raw).expect("actual native Mapping bytes");
        mappings.push(raw);
        app["mapping_collections"] = json!(["mapping.json"]);
    }
    let app_raw = write(&root.join("applicability.json"), &app);
    let native = crate::applicability::prepare_analysis(
        &root.join("applicability.json"),
        ReportFilters::default(),
    )
    .expect("actual complete native baseline");
    let report = write(
        &root.join("gap-report.json"),
        &serde_json::to_value(&native.report).expect("full native report"),
    );
    let framework = std::fs::read(root.join("framework.json")).expect("actual framework original");
    let mut baseline =
        json!({"framework_sha256":sha256_hex(&framework),"report_sha256":sha256_hex(&report)});
    if let Some(raw) = &companion {
        baseline["resolved_catalog_sha256"] = json!(sha256_hex(raw));
    }
    NativeFixtureBaseline { report, app: app_raw, framework, companion, mappings, baseline }
}

impl Fixture {
    /// Use real init, native inventory, Mapping builder and complete App engine before testing.
    fn new(profile: bool, mapping: bool) -> Self {
        let directory = tempfile::tempdir().expect("fixture directory");
        let root = directory.path().canonicalize().expect("actual canonical fixture root");
        let NativeFixtureBaseline {
            report,
            app: app_raw,
            framework,
            companion,
            mappings,
            baseline,
        } = native_baseline(&root, profile, mapping);
        let pack = json!({"schema_version":"forge.authoring-pack/1","pack_key":"native-pack","version":"1.0.0",
            "baseline":baseline,"reviewers":[{"key":"human","name":"Native Private Reviewer"}],
            "content_rights":{"source_label":"Synthetic source","statement":"Actual fixture evidence remains private.","review":review()},
            "topics":[{"key":"access-topic","title":"Private access title","order":20,"question_keys":["owner"]},
                {"key":"operations-topic","title":"Private operations title","order":10,"question_keys":[]}],
            "policy_families":[{"key":"access-family","title":"Private access family"},
                {"key":"operations-family","title":"Private operations family"}],
            "questions":[{"key":"owner","prompt":"Who writes the café draft?","type":"string","required":true,
                "owner":"context-owner","sensitivity":"confidential","source_label":"Private native interview",
                "max_age_days":30,"constraints":{"min_length":1,"max_length":100}}],
            "control_assignments":[{"key":"c-access","control_id":"c-1","topic_key":"access-topic","review":review()},
                {"key":"c-operations","control_id":"c-2","topic_key":"operations-topic","review":review()}],
            "family_assignments":[{"key":"f-access","topic_key":"access-topic","policy_family_key":"access-family","review":review()},
                {"key":"f-operations","topic_key":"operations-topic","policy_family_key":"operations-family","review":review()}]});
        let pack_raw = write(&root.join("pack.json"), &pack);
        let clause = "The fictional café team records every supplied native draft change.\r\n"
            .as_bytes()
            .to_vec();
        std::fs::write(root.join("clause.md"), &clause).expect("actual human clause");
        let project = json!({"schema_version":"forge.author-project/1","project_key":"native-project","project_root":".",
            "baseline":baseline,"as_of":"2026-09-08T01:30:00.250+01:30",
            "applicability_manifest":{"path":"applicability.json","expected_sha256":sha256_hex(&app_raw)},
            "gap_report":{"path":"gap-report.json","expected_sha256":sha256_hex(&report)},
            "authoring_pack":{"path":"pack.json","expected_sha256":sha256_hex(&pack_raw)},
            "reviewers":[{"key":"human","name":"Native Private Reviewer"}],"baseline_review":review(),
            "policies":[{"key":"access-policy","policy_family_key":"access-family","title":"Private access policy"},
                {"key":"operations-policy","policy_family_key":"operations-family","title":"Private operations policy"}],
            "answers":[],"deferrals":[{"key":"later","gap_id":manifest::gap_id(&sha256_hex(&report),"c-3"),
                "review":review(),"revisit_date":"2026-10-01"}],"human_clauses":[{"key":"operations-clause",
                "policy_key":"operations-policy","topic_key":"operations-topic","gap_ids":[manifest::gap_id(&sha256_hex(&report),"c-2")],
                "answer_refs":[],"source":{"path":"clause.md","expected_sha256":sha256_hex(&clause)},"review":review()}]});
        let project_raw = write(&root.join("project.json"), &project);
        Self {
            _directory: directory,
            root,
            project,
            pack,
            project_raw,
            pack_raw,
            report,
            app: app_raw,
            framework,
            companion,
            mappings,
            clauses: vec![clause],
        }
    }
    /// Preserve actual pack/project byte pins after a targeted fixture edit.
    fn save(&mut self) {
        self.pack_raw = write(&self.root.join("pack.json"), &self.pack);
        self.project["authoring_pack"]["expected_sha256"] = json!(sha256_hex(&self.pack_raw));
        self.project_raw = write(&self.root.join("project.json"), &self.project);
    }
    /// Derive a complete real human answer and pin the exact record into a clause.
    fn answer(&mut self) {
        let question: manifest::Question =
            serde_json::from_value(self.pack["questions"][0].clone()).expect("actual question");
        let answer = json!({"key":"owner-answer","question_key":"owner","question_sha256":manifest::question_sha256(&question).unwrap(),
            "authoring_pack_sha256":sha256_hex(&self.pack_raw),"owner":"context-owner","source_label":"Private complete interview",
            "sensitivity":"confidential","review":review(),"state":"provided","value":"Private native café custodian"});
        let typed: manifest::Answer = serde_json::from_value(answer.clone()).unwrap();
        self.project["answers"] = json!([answer]);
        let clause = self.project["human_clauses"][0].clone();
        self.project["human_clauses"].as_array_mut().unwrap().push(json!({"key":"access-clause","policy_key":"access-policy",
            "topic_key":"access-topic","gap_ids":[manifest::gap_id(&sha256_hex(&self.report),"c-1")],
            "answer_refs":[{"answer_key":"owner-answer","expected_sha256":manifest::answer_sha256(&typed).unwrap()}],
            "source":{"path":"access-clause.md","expected_sha256":sha256_hex(&self.clauses[0])},"review":clause["review"]}));
        let raw = self.clauses[0].clone();
        std::fs::write(self.root.join("access-clause.md"), &raw).unwrap();
        self.clauses.push(raw);
        self.save();
    }
    /// Consume only complete fixture originals, with one unchanged supplied control/callback.
    fn run<E>(
        &self,
        control: &mut dyn WorkControl,
        admit: &mut impl FnMut(AuthoringCharge) -> Result<(), E>,
    ) -> Result<PreparedAuthoringFacts, BorrowedAuthoringError<E>> {
        let mappings: Vec<_> = self.mappings.iter().map(Vec::as_slice).collect();
        let clauses: Vec<_> = self.clauses.iter().map(Vec::as_slice).collect();
        prepare(
            BorrowedAuthoringInputs {
                project_filename: Path::new("project.json"),
                project: &self.project_raw,
                pack: &self.pack_raw,
                gap_report: &self.report,
                applicability_manifest: &self.app,
                framework: &self.framework,
                resolved_catalog: self.companion.as_deref(),
                mappings: &mappings,
                clauses: &clauses,
            },
            control,
            admit,
        )
    }
    /// Compare the ENTIRE native output and native fingerprint denominator to the legacy file path.
    fn parity(&self) {
        let actual = self
            .run(&mut NoopControl, &mut |_| Ok::<_, ()>(()))
            .expect("plain native complete evaluator");
        let legacy = super::super::input::prepare(&self.root.join("project.json"))
            .expect("genuine retained file wrapper");
        let expected =
            super::super::plan::build_plan(&legacy.loaded).expect("unchanged native plan");
        assert_eq!(
            serde_json::to_value(&actual.plan).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        assert_eq!(actual.loaded.inputs, legacy.loaded.inputs);
        assert_eq!(actual.loaded.clauses.len(), legacy.loaded.clauses.len());
        for (key, clause) in &actual.loaded.clauses {
            assert_eq!(clause.bytes, legacy.loaded.clauses[key].bytes);
        }
        assert_eq!(actual.plan.as_of, self.project["as_of"].as_str().unwrap());
        assert!(actual.plan.provenance.inputs.iter().all(|row| !matches!(
            row.role.as_str(),
            "stored-plan" | "review-locator" | "init-policy"
        )));
    }
}

/// Translate real descriptors into the unchanged actual `ContractLedger` for component controls.
fn charge(ledger: &RefCell<ContractLedger>, charge: AuthoringCharge) -> Result<(), ContractError> {
    ledger.borrow_mut().bound(|ledger| match charge {
        AuthoringCharge::Checkpoint => Ok(()),
        AuthoringCharge::Reserve { logical_bytes } => ledger.derived(logical_bytes),
        AuthoringCharge::Work { visits, byte_work, matching_steps } => {
            ledger.visits(visits)?;
            ledger.bytes(byte_work)?;
            ledger.matching(matching_steps)
        }
        AuthoringCharge::Capacity => Err(ledger.capacity()),
    })
}
/// Observe the real original cooperative checkpoint count, including a calibrated actual stop.
struct Probe {
    calls: usize,
    stop: Option<(usize, bool)>,
}
impl WorkControl for Probe {
    /// Observe only the actual original checkpoint reach and retain its first stop.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        if self.stop.is_some_and(|(at, _)| self.calls >= at) {
            if self.stop.unwrap().1 {
                return Err(WorkError::Failed(Error::invalid()));
            }
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }
    /// Calibrated tests return their actual interruption directly from checkpoint.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}
/// Preserve private native Domain diagnostics without requiring a fake successful facts Debug DTO.
fn domain_text<E>(result: Result<PreparedAuthoringFacts, BorrowedAuthoringError<E>>) -> String {
    match result {
        Err(BorrowedAuthoringError::Domain(error)) => error.to_string(),
        _ => panic!("expected ordinary native Domain"),
    }
}

/// Full private native reviewer/rationale/input/array parity under actual original command caps.
#[test]
fn complete_catalog_plan_matches_legacy_full_private_output() {
    let mut f = Fixture::new(false, false);
    f.answer();
    f.parity();
    let ledger = RefCell::new(ContractLedger::default());
    f.run(&mut NoopControl, &mut |item| charge(&ledger, item))
        .expect("actual unchanged-bound complete plan");
}
/// Profile companion is an explicit original and full native provenance is not a selected subset.
#[test]
fn complete_profile_companion_plan_matches_legacy() {
    let f = Fixture::new(true, false);
    f.parity();
    let result = f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())).unwrap();
    assert_eq!(result.loaded.inputs.iter().filter(|row| row.role == "resolved-catalog").count(), 1);
}
/// Real maintained negative Mapping analysis preserves reviewed no-relationship gap semantics.
#[test]
fn genuine_mapping_full_roster_and_negative_relationship_parity() {
    let f = Fixture::new(false, true);
    f.parity();
    let result = f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())).unwrap();
    assert_eq!(result.loaded.baseline_report.counts.applicable_reviewed_no_relationship, 1);
    assert_eq!(result.plan.counts.total, 4);
    assert_eq!(
        result.loaded.inputs.iter().filter(|row| row.role == "mapping-collection-0").count(),
        1
    );
}
/// Actual native c1 expands across two selected policy destinations with complete private tuples.
#[test]
fn many_to_many_expansion_keeps_all_private_assignments() {
    let mut f = Fixture::new(false, false);
    f.pack["family_assignments"].as_array_mut().unwrap().push(json!({"key":"f-access-operations",
        "topic_key":"access-topic","policy_family_key":"operations-family","review":review()}));
    f.save();
    f.parity();
    let actual = f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())).unwrap();
    let c1 = actual.plan.gaps.iter().find(|gap| gap.control_id == "c-1").unwrap();
    assert_eq!(c1.assignments.len(), 2);
    assert_eq!(actual.plan.counts.assigned, 2);
    assert_eq!(actual.plan.policies.iter().map(|policy| policy.sections.len()).sum::<usize>(), 3);
}
/// App manifest-relative parent routes retain exact UTF8 labels and native date spelling.
#[test]
fn nested_manifest_parent_resolution_and_native_date_spelling() {
    let mut f = Fixture::new(false, false);
    std::fs::create_dir(f.root.join("基準")).unwrap();
    let mut app: Value = serde_json::from_slice(&f.app).unwrap();
    app["framework"]["artifact"] = json!("../framework.json");
    f.app = write(&f.root.join("基準/applicability.json"), &app);
    let native = crate::applicability::prepare_analysis(
        &f.root.join("基準/applicability.json"),
        ReportFilters::default(),
    )
    .unwrap();
    f.report =
        write(&f.root.join("gap-report.json"), &serde_json::to_value(&native.report).unwrap());
    let hash = sha256_hex(&f.report);
    f.project["applicability_manifest"] =
        json!({"path":"基準/applicability.json","expected_sha256":sha256_hex(&f.app)});
    f.project["gap_report"]["expected_sha256"] = json!(hash);
    f.project["baseline"]["report_sha256"] = json!(hash);
    f.pack["baseline"]["report_sha256"] = json!(hash);
    f.project["deferrals"][0]["gap_id"] = json!(manifest::gap_id(&hash, "c-3"));
    f.project["human_clauses"][0]["gap_ids"] = json!([manifest::gap_id(&hash, "c-2")]);
    f.save();
    f.parity();
    let result = f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())).unwrap();
    assert!(result.loaded.inputs.iter().any(|row| row.path == "基準/applicability.json"));
    assert_eq!(result.plan.as_of, "2026-09-08T01:30:00.250+01:30");
}
/// Same counts are insufficient: complete native private fields participate in stored report equality.
#[test]
fn repinned_private_report_difference_is_not_accepted() {
    let mut f = Fixture::new(false, false);
    f.parity();
    let mut report: Value = serde_json::from_slice(&f.report).unwrap();
    report["controls"][0]["rationale"] = json!("Different private claim");
    f.report = write(&f.root.join("gap-report.json"), &report);
    let hash = sha256_hex(&f.report);
    f.project["gap_report"]["expected_sha256"] = json!(hash);
    f.project["baseline"]["report_sha256"] = json!(hash);
    f.pack["baseline"]["report_sha256"] = json!(hash);
    f.save();
    let actual = domain_text(f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())));
    let legacy =
        super::super::input::prepare(&f.root.join("project.json")).err().unwrap().to_string();
    assert_eq!(actual, legacy);
    assert!(actual.contains("complete current unfiltered"));
}
/// Exact declared occurrence roster, not plain caller facts, selects Mapping input consumption.
#[test]
fn omitted_declared_mapping_refuses_before_native_subset() {
    let mut f = Fixture::new(false, true);
    f.parity();
    f.mappings.clear();
    assert!(
        domain_text(f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())))
            .contains("complete declared manifest roster")
    );
}
/// Legacy case-fold alias refusal remains stronger than any generic Source-pool compatibility.
#[test]
fn casefold_native_input_alias_is_rejected_before_framework_parse() {
    let mut f = Fixture::new(false, false);
    f.parity();
    let mut app: Value = serde_json::from_slice(&f.app).unwrap();
    app["framework"]["artifact"] = json!("PACK.json");
    f.app = write(&f.root.join("applicability.json"), &app);
    f.project["applicability_manifest"]["expected_sha256"] = json!(sha256_hex(&f.app));
    f.save();
    let actual = domain_text(f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())));
    let legacy =
        super::super::input::prepare(&f.root.join("project.json")).err().unwrap().to_string();
    assert_eq!(actual, legacy);
    assert!(actual.contains("framework aliases another input"));
}
/// Every native clause is validated even if context blocks its section's inclusion.
#[test]
fn blocked_clause_reserved_syntax_keeps_native_domain_error() {
    let mut f = Fixture::new(false, false);
    f.parity();
    // Move this actual clause into the missing-required-context access section.
    f.project["human_clauses"][0]["policy_key"] = json!("access-policy");
    f.project["human_clauses"][0]["topic_key"] = json!("access-topic");
    f.project["human_clauses"][0]["gap_ids"] =
        json!([manifest::gap_id(&sha256_hex(&f.report), "c-1")]);
    f.clauses[0] = b"{{forge: unsupported native clause}}\n".to_vec();
    std::fs::write(f.root.join("clause.md"), &f.clauses[0]).unwrap();
    f.project["human_clauses"][0]["source"]["expected_sha256"] = json!(sha256_hex(&f.clauses[0]));
    f.save();
    let actual = domain_text(f.run(&mut NoopControl, &mut |_| Ok::<_, ()>(())));
    let legacy =
        super::super::input::prepare(&f.root.join("project.json")).err().unwrap().to_string();
    assert_eq!(actual, legacy);
    assert!(actual.contains("reserved FORGE template syntax"));
}
/// Existing Capacity on the actual original ledger wins before ordinary syntax or later caller stop.
#[test]
fn original_capacity_wins_over_later_domain_and_control_stop() {
    let ledger = RefCell::new(ContractLedger::default());
    assert_eq!(ledger.borrow_mut().derived(usize::MAX), Err(ContractError::Capacity));
    let mut control = Probe { calls: 0, stop: Some((1, false)) };
    let result = discover_project(b"{", &mut control, &mut |item| charge(&ledger, item));
    assert!(matches!(result, Err(BorrowedAuthoringError::Admission(ContractError::Capacity))));
    assert_eq!(control.calls, 0);
}
/// Calibrate the genuine last wrapper checkpoint over the same malformed raw, without fake proof/timer.
#[test]
fn ordinary_decode_failure_crosses_final_original_stop_fence() {
    let mut baseline = Probe { calls: 0, stop: None };
    assert!(matches!(
        discover_project(b"{", &mut baseline, &mut |_| Ok::<_, ()>(())),
        Err(BorrowedAuthoringError::Domain(_))
    ));
    for failed in [false, true] {
        let mut control = Probe { calls: 0, stop: Some((baseline.calls, failed)) };
        let result = discover_project(b"{", &mut control, &mut |_| Ok::<_, ()>(()));
        match (failed, result) {
            (
                false,
                Err(BorrowedAuthoringError::Work(WorkError::Interrupted(
                    Interruption::CancelRequested,
                ))),
            )
            | (true, Err(BorrowedAuthoringError::Work(WorkError::Failed(_)))) => {}
            _ => panic!("actual post-domain stop required"),
        }
        assert_eq!(control.calls, baseline.calls);
    }
}
/// Ordinary invalid discovery cannot renew either the real callback ledger or original control.
#[test]
fn ordinary_domain_then_valid_native_work_uses_same_ledger() {
    let f = Fixture::new(false, false);
    let ledger = RefCell::new(ContractLedger::default());
    let mut control = Probe { calls: 0, stop: None };
    let mut admit = |item| charge(&ledger, item);
    assert!(matches!(
        discover_project(b"{", &mut control, &mut admit),
        Err(BorrowedAuthoringError::Domain(_))
    ));
    let calls = control.calls;
    let result = f.run(&mut control, &mut admit).expect("same real unchanged-bound native owner");
    assert!(control.calls > calls);
    assert_eq!(result.plan.counts.total, 4);
}

/// Genuine physical/native authoring receiver controls, not an independent approval of plain helpers.
mod receiver_tests {
    include!("../review/authoring_capture_tests.rs");
}

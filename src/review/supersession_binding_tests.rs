//! Genuine complete union/queue/native finding binding controls; no proof constructors.
use super::*;
use crate::framework::{analysis, manifest, model::ImpactFilters};
use crate::review::applicability_capture::{self, tests::Fixture};
use crate::review::capture::supersession::seal_complete_union;
use crate::review::capture::{ReviewCapture, ReviewControl};
use crate::review::commands::{self, InitOptions};
use crate::review::{decode, encode, links, validate, wire};
use crate::workspace::preparation::NoopControl;
use serde_json::{Value, json};
use std::path::Path;
use std::rc::Rc;

/// Hold a genuine fixture plus the native-produced complete comparison.
struct Case {
    /// Actual lifetime owner of all old/new/native file originals.
    fixture: Fixture,
    /// Actual current report from the maintained file producer before capture.
    report: crate::framework::model::ImpactReport,
}

/// Persist synthetic declarations only before any qualification capture begins.
fn write(root: &Path, name: &str, value: &Value) {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// Initialize genuine queues while retaining the production Windows publication refusal.
fn init(root: &Path, id: &str, output: &str) {
    let options = InitOptions {
        project_root: root,
        sources: Path::new("locator.json"),
        policy: Path::new("init.json"),
        queue_id: id,
        created_at: "2026-10-04T00:00:00Z",
        output: Path::new(output),
    };
    let raw = native_queue_fixture_bytes(&options).unwrap();
    let result = commands::init(&options, &mut NoopControl);
    #[cfg(not(windows))]
    {
        result.unwrap();
        assert_eq!(std::fs::read(root.join(output)).unwrap(), raw);
    }
    #[cfg(windows)]
    {
        assert!(matches!(result, Err(commands::CommandError::Publication)));
        assert!(!root.join(output).exists());
        // This is cfg-only fixture persistence after the real init core and all
        // its input fences; it is not a supported production publication route.
        std::fs::write(root.join(output), raw).unwrap();
    }
}

/// Generate fixture bytes on every platform through the maintained complete initialization core.
/// No native/approval facts or proof are supplied by this helper; every phase uses
/// one genuine capture, accepted caller and ledger. All owners drop before return,
/// so fixture persistence cannot be mistaken for guarded Windows publication.
fn native_queue_fixture_bytes(options: &InitOptions<'_>) -> Result<Vec<u8>, ContractError> {
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    native_queue_fixture_inner(options, &mut ledger, &mut control)
}

/// Preserve the actual `init_inner` pre-publication chronology and native final fences.
fn native_queue_fixture_inner(
    options: &InitOptions<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<u8>, ContractError> {
    ledger.checkpoint(control)?;
    ledger.bytes(
        options
            .queue_id
            .len()
            .checked_add(options.created_at.len())
            .ok_or(ContractError::Capacity)?,
    )?;
    if options.queue_id.len() != 36 || options.created_at.len() != 20 {
        return Err(ContractError::Invalid);
    }
    validate::uuid(options.queue_id)?;
    validate::time(options.created_at)?;
    let mut capture = ReviewCapture::new(options.project_root, &[options.output], ledger, control)?;
    let sources = commands::auxiliary(&mut capture, options.sources, 1_048_576, ledger, control)?;
    let policy = commands::auxiliary(&mut capture, options.policy, 1_048_576, ledger, control)?;
    let pending = commands::prepare_native(&mut capture, sources, ledger, control)?;
    let held = commands::owned(capture, ledger, control)?;
    let closure = pending.seal(Rc::clone(&held), ledger, control)?;
    let domain = closure.domain();
    let request = commands::init_request_domain(held.bytes(policy)?, domain, ledger, control)?;
    let mut selected = crate::review::chain::reserved(request.items.len(), ledger)?;
    for item in &request.items {
        crate::review::chain::visit(ledger, control)?;
        selected.push(item.subject_id.as_str());
    }
    commands::sort_subjects_domain(&mut selected, domain, ledger, control)?;
    let prepared = closure.prepare(&selected, ledger, control)?;
    let native = prepared.view();
    ledger.visits(1)?;
    if native.selected_count() != selected.len() {
        return Err(ContractError::Binding);
    }
    drop(selected);
    let queue = commands::build_queue(request, options, native, ledger, control)?;
    let output = encode::queue(&queue, ledger, control)?;
    let closed = decode::decode_queue(&output, ledger, control)?;
    native.bind_queue(&closed, ledger, control)?;
    native.verify_inputs(ledger, control)?;
    // Preserve the command publisher's final shared admission, then recheck
    // genuine native originals before accepting bytes for fixture persistence.
    ledger.checkpoint(control)?;
    ledger.bytes(output.len())?;
    native.verify_inputs(ledger, control)?;
    ledger.checkpoint(control)?;
    Ok(output)
}

/// Genuine native fixture with archived old originals and exact current generated sources.
fn case(profile: bool, changed: bool) -> Case {
    case_with_items(profile, changed, &["c1"])
}

/// Select real declared native Controls for old/new queues from the maintained init core.
fn case_with_items(profile: bool, changed: bool, subjects: &[&str]) -> Case {
    let fixture = Fixture::new(profile);
    let root = fixture.root();
    write(
        root,
        "init.json",
        &json!({"schema_version":"forge.review-init/1",
        "roles":[{"key":"review"}], "reviewers":[{"key":"reviewer","role_keys":["review"]}],
        "policies":[{"key":"policy","seats":[{"role_key":"review","count":1}],"substitutions":[],"abstention_rule":"nonapproving","empty_abstention_reasons":[],"author_separation":"declared-keys"}],
        "items":subjects.iter().enumerate().map(|(index, subject)| json!({
            "key":if index == 0 { "item".to_owned() } else { format!("item-{index}") },
            "subject_id":subject,"policy_key":"policy","author_keys":["author"],"assignments":[],"due_at":null
        })).collect::<Vec<_>>()}),
    );
    init(root, "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "old-queue.json");
    std::fs::create_dir(root.join("old")).unwrap();
    for name in ["framework.json", "mapping.json", "applicability.json", "policy.json"] {
        std::fs::copy(root.join(name), root.join("old").join(name)).unwrap();
    }
    if profile {
        std::fs::copy(root.join("resolved-catalog.json"), root.join("old/resolved-catalog.json"))
            .unwrap();
    }
    if changed {
        fixture.refresh_framework_statement_for_supersession();
    }
    init(root, "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "new-queue.json");
    let resource = |prefix: &str| {
        let raw = std::fs::read(root.join(prefix).join("framework.json")).unwrap();
        let value: Value = serde_json::from_slice(&raw).unwrap();
        let model = if profile { "profile" } else { "catalog" };
        let mut resource = json!({"type":model,"artifact":format!("{prefix}framework.json"),"expected_sha256":crate::hashing::sha256_hex(&raw),"root_uuid":value[model]["uuid"],"document_version":value[model]["metadata"]["version"],"oscal_version":"1.2.3"});
        if profile {
            resource["resolved_catalog"] = json!(format!("{prefix}resolved-catalog.json"));
            resource["resolved_catalog_attestation"] = json!(true);
            resource["expected_resolved_catalog_sha256"] = json!(crate::hashing::sha256_hex(
                &std::fs::read(root.join(prefix).join("resolved-catalog.json")).unwrap()
            ));
        }
        resource
    };
    write(
        root,
        "impact-manifest.json",
        &json!({"schema_version":manifest::MANIFEST_SCHEMA_VERSION,
        "old":resource("old/"),"new":resource(""),"mapping_collections":[{"artifact":"old/mapping.json","framework_role":"target"}],"applicability_manifest":"old/applicability.json"}),
    );
    let parsed =
        manifest::parse(&std::fs::read(root.join("impact-manifest.json")).unwrap()).unwrap();
    let (report, _) = analysis::analyze(root, &parsed, ImpactFilters::default()).unwrap();
    write(root, "impact-report.json", &serde_json::to_value(&report).unwrap());
    write(
        root,
        "impact-locator.json",
        &json!({"schema_version":"forge.review-queue-impact-locator/1",
        "manifest_path":"impact-manifest.json","report_path":"impact-report.json",
        "old_framework_source_key":"framework","new_framework_source_key":"framework",
        "old_resolved_source_key":if profile {json!("resolved")} else {Value::Null},
        "new_resolved_source_key":if profile {json!("resolved")} else {Value::Null}}),
    );
    write(root, "links.json", &json!({"schema_version":links::LINKS_SCHEMA,"links":[]}));
    Case { fixture, report }
}

/// Extract actual generated item IDs and request only the explicitly selected real current finding.
fn link(case: &Case, finding: Option<&str>) {
    let root = case.fixture.root();
    let old: Value =
        serde_json::from_slice(&std::fs::read(root.join("old-queue.json")).unwrap()).unwrap();
    let new: Value =
        serde_json::from_slice(&std::fs::read(root.join("new-queue.json")).unwrap()).unwrap();
    write(
        root,
        "links.json",
        &json!({"schema_version":links::LINKS_SCHEMA,"links":[{
        "old_item_id":old["items"][0]["item_id"],"new_item_id":new["items"][0]["item_id"],
        "finding_ids":finding.map(|id|vec![id]).unwrap_or_default()}]}),
    );
}

/// Capture every actual purpose and obtain both genuine pending cohorts before consuming their union.
fn capture(
    case: &Case,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> HeldSupersessionInputs {
    let mut capture = ReviewCapture::new_supersession(
        case.fixture.root(),
        Path::new("out.json"),
        ledger,
        control,
    )
    .unwrap();
    for (path, purpose) in [
        ("old-queue.json", QueuePurpose::HistoricalOld),
        ("new-queue.json", QueuePurpose::CurrentNew),
    ] {
        capture.required_supersession_queue(Path::new(path), purpose, ledger, control).unwrap();
    }
    for (path, purpose) in [
        ("locator.json", AuxiliaryPurpose::NewNativeLocator),
        ("impact-locator.json", AuxiliaryPurpose::ImpactLocator),
        ("links.json", AuxiliaryPurpose::Links),
    ] {
        capture.required_supersession_auxiliary(Path::new(path), purpose, ledger, control).unwrap();
    }
    let locator =
        capture.supersession_auxiliary_original(AuxiliaryPurpose::NewNativeLocator).unwrap();
    let native = applicability_capture::prepare(&mut capture, locator, ledger, control).unwrap();
    let impact = crate::review::impact_capture::prepare(&mut capture, ledger, control).unwrap();
    seal_complete_union(
        Rc::new(capture.finish()),
        native.into_supersession_pending(),
        impact,
        ledger,
        control,
    )
    .unwrap()
}

/// Decode only the exact successful registered raw purpose on the actual union owner.
fn queues<'a>(
    held: &'a HeldSupersessionInputs,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> (Decoded<'a, QueueDocument>, Decoded<'a, QueueDocument>, DecodedLinks<'a>) {
    let old = decode::decode_queue(
        held.held_inputs()
            .bytes(held.queue_original(QueuePurpose::HistoricalOld).unwrap())
            .unwrap(),
        ledger,
        control,
    )
    .unwrap();
    let new = decode::decode_queue(
        held.held_inputs().bytes(held.queue_original(QueuePurpose::CurrentNew).unwrap()).unwrap(),
        ledger,
        control,
    )
    .unwrap();
    let links = links::decode_links(
        held.held_inputs()
            .bytes(held.auxiliary_original(AuxiliaryPurpose::Links).unwrap())
            .unwrap(),
        ledger,
        control,
    )
    .unwrap();
    (old, new, links)
}

/// Real no-change Catalog/Profile queues and complete union bind without carrying old votes.
#[test]
fn genuine_current_adapter_and_complete_historical_framework_pair_bind() {
    for profile in [false, true] {
        let case = case(profile, false);
        link(&case, None);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture(&case, &mut ledger, &mut control);
        let (old, new, links) = queues(&held, &mut ledger, &mut control);
        let bound = prepare(&held, &old, &new, &links, &mut ledger, &mut control).unwrap();
        assert_eq!(bound.graph().links().len(), 1);
        assert!(bound.linked_findings()[0].is_empty());
        let no_findings: [&str; 0] = [];
        assert_eq!(bound.unreferenced_findings(), no_findings);
        bound.verify_inputs(&mut ledger, &mut control).unwrap();
        assert!(!case.fixture.root().join("out.json").exists());
    }
}

/// Changed native Catalog/Profile finding binds the exact old control and archived manifest hash.
#[test]
fn genuine_native_applicability_finding_and_old_declared_pin_bind() {
    for profile in [false, true] {
        let case = case(profile, true);
        let finding = case
            .report
            .findings
            .iter()
            .find(|finding| {
                finding.reason_code == ReasonCode::ApplicabilityDecisionChanged
                    && finding.subject_id == "c1"
            })
            .unwrap();
        link(&case, Some(&finding.finding_id));
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture(&case, &mut ledger, &mut control);
        let (old, new, links) = queues(&held, &mut ledger, &mut control);
        let bound = prepare(&held, &old, &new, &links, &mut ledger, &mut control).unwrap();
        assert_eq!(bound.linked_findings()[0][0].finding_id, finding.finding_id);
        assert_eq!(bound.unreferenced_findings().len() + 1, case.report.findings.len());
        assert!(!case.fixture.root().join("out.json").exists());
    }
}

/// An unrelated real native Mapping finding cannot advertise an applicability association.
#[test]
fn genuine_unrelated_native_dependency_finding_is_refused() {
    let case = case(false, true);
    let finding = case
        .report
        .findings
        .iter()
        .find(|finding| finding.reason_code == ReasonCode::MappingSubjectChanged)
        .unwrap();
    link(&case, Some(&finding.finding_id));
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let held = capture(&case, &mut ledger, &mut control);
    let (old, new, links) = queues(&held, &mut ledger, &mut control);
    assert!(matches!(
        prepare(&held, &old, &new, &links, &mut ledger, &mut control),
        Err(ContractError::Binding)
    ));
}

/// A syntactically valid requested UUID outside the complete native report cannot link.
#[test]
fn nonexistent_native_finding_id_is_refused() {
    let case = case(false, true);
    link(&case, Some("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee"));
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let held = capture(&case, &mut ledger, &mut control);
    let (old, new, links) = queues(&held, &mut ledger, &mut control);
    assert!(matches!(
        prepare(&held, &old, &new, &links, &mut ledger, &mut control),
        Err(ContractError::Binding)
    ));
}

/// Byte-equal detached old/new/link decoders cannot replace the retained actual allocations.
#[test]
fn detached_equal_raw_decoders_are_refused_for_every_purpose() {
    let case = case(false, false);
    link(&case, None);
    for purpose in 0..3 {
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture(&case, &mut ledger, &mut control);
        let (old, new, links) = queues(&held, &mut ledger, &mut control);
        let copied = match purpose {
            0 => old.raw().to_vec(),
            1 => new.raw().to_vec(),
            _ => links.raw().to_vec(),
        };
        let result = match purpose {
            0 => {
                let detached = decode::decode_queue(&copied, &mut ledger, &mut control).unwrap();
                prepare(&held, &detached, &new, &links, &mut ledger, &mut control).map(|_| ())
            }
            1 => {
                let detached = decode::decode_queue(&copied, &mut ledger, &mut control).unwrap();
                prepare(&held, &old, &detached, &links, &mut ledger, &mut control).map(|_| ())
            }
            _ => {
                let detached = links::decode_links(&copied, &mut ledger, &mut control).unwrap();
                prepare(&held, &old, &new, &detached, &mut ledger, &mut control).map(|_| ())
            }
        };
        assert_eq!(result, Err(ContractError::Binding));
    }
}

/// Later real archived framework drift refuses the retained original fence after genuine success.
#[test]
fn real_original_drift_after_native_binding_is_refused() {
    let case = case(false, false);
    link(&case, None);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let held = capture(&case, &mut ledger, &mut control);
    let (old, new, links) = queues(&held, &mut ledger, &mut control);
    let bound = prepare(&held, &old, &new, &links, &mut ledger, &mut control).unwrap();
    let path = case.fixture.root().join("old/framework.json");
    let mut raw = std::fs::read(&path).unwrap();
    raw.push(b' ');
    std::fs::write(path, raw).unwrap();
    assert!(matches!(bound.verify_inputs(&mut ledger, &mut control), Err(ContractError::Binding)));
}

/// Complete historical declarations with a false framework tuple refuse actual native correlation.
#[test]
fn structurally_valid_historical_framework_pin_forgery_is_refused() {
    let case = case(false, false);
    let path = case.fixture.root().join("old-queue.json");
    let mut queue: wire::QueueDocument =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    queue.source_pins.iter_mut().find(|pin| pin.artifact_key == "framework").unwrap().raw_sha256 =
        "f".repeat(64);
    let mut items = std::mem::take(&mut queue.items);
    let mut fixture_ledger = ContractLedger::default();
    for item in &mut items {
        item.context_sha256 = validate::context_hash(&queue, item, &mut fixture_ledger).unwrap();
        item.policy_sha256 =
            validate::policy_hash(&queue, &queue.policies[0], item, &mut fixture_ledger).unwrap();
        item.item_id = validate::item_id(&queue, item, &mut fixture_ledger).unwrap();
    }
    queue.items = items;
    std::fs::write(path, encode::queue(&queue, &mut fixture_ledger, &mut NoopControl).unwrap())
        .unwrap();
    link(&case, None);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let held = capture(&case, &mut ledger, &mut control);
    let (old, new, links) = queues(&held, &mut ledger, &mut control);
    assert!(matches!(
        prepare(&held, &old, &new, &links, &mut ledger, &mut control),
        Err(ContractError::Binding)
    ));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "supersession_command_tests.rs"]
/// Actual full command, finite record, public CLI and guarded publication controls.
mod command_tests;

/// Explicit deterministic caller faults at genuinely observed native consumer checkpoints.
/// This type owns no clock, captured source, native data, pending token or proof.
#[derive(Clone, Copy)]
enum BindingFault {
    /// Actual caller cancellation, preserved by the accepted `ReviewControl` and ledger.
    Interrupted,
    /// Actual ordinary shared-control failure, preserved by the original ledger.
    Failed,
}

/// Runtime test observation shared while the one accepted `ReviewControl` stays borrowed.
#[derive(Default)]
struct BindingStopState {
    /// Every actual caller checkpoint in monotonically increasing order.
    calls: usize,
    /// Exact observed global checkpoint to fault; no phase counter is reset.
    selected: Option<usize>,
    /// Explicit real caller error for the selected checkpoint.
    fault: Option<BindingFault>,
    /// Whether that actual consumer checkpoint was reached and failed.
    fired: bool,
    /// First returned caller interruption, never cleared or replaced.
    interrupted: Option<crate::workspace::preparation::Interruption>,
    /// Last genuine caller boundary; final ordinary Binding fence is `PrepareDomain`.
    last_stage: Option<crate::workspace::preparation::Stage>,
}

/// Actual caller `WorkControl`; observation can be armed without replacing its accepted owner.
struct BindingStop {
    /// Shared observation only, not native/capture authority or an alternate ledger.
    state: Rc<std::cell::RefCell<BindingStopState>>,
}

impl WorkControl for BindingStop {
    /// Count actual producer checkpoints and return the explicit fault at the selected one.
    fn checkpoint(
        &mut self,
        stage: crate::workspace::preparation::Stage,
        _progress: crate::workspace::preparation::ProgressUpdate,
    ) -> crate::workspace::preparation::WorkResult<()> {
        use crate::workspace::preparation::{Interruption, WorkError};
        let mut observation = self.state.borrow_mut();
        if let Some(reason) = observation.interrupted {
            return Err(WorkError::Interrupted(reason));
        }
        observation.calls += 1;
        observation.last_stage = Some(stage);
        if observation.selected == Some(observation.calls) {
            observation.fired = true;
            match observation.fault.expect("explicit real caller fault") {
                BindingFault::Interrupted => {
                    observation.interrupted = Some(Interruption::CancelRequested);
                    return Err(WorkError::Interrupted(Interruption::CancelRequested));
                }
                BindingFault::Failed => {
                    return Err(WorkError::Failed(crate::workspace::contract::Error::invalid()));
                }
            }
        }
        Ok(())
    }

    /// Expose the same first real cancellation to the accepted operation's final fence.
    fn interruption(&self) -> Option<crate::workspace::preparation::Interruption> {
        self.state.borrow().interrupted
    }
}

/// Read only the item UUID actually emitted by the init core for the selected genuine subject.
fn emitted_item(root: &Path, queue: &str, subject: &str) -> String {
    let value: Value = serde_json::from_slice(&std::fs::read(root.join(queue)).unwrap()).unwrap();
    let matching: Vec<_> = value["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["subject_id"].as_str() == Some(subject))
        .collect();
    assert_eq!(matching.len(), 1);
    matching[0]["item_id"].as_str().unwrap().to_owned()
}

/// Write complete explicitly selected many-to-many pairs before the actual capture begins.
fn many_to_many(case: &Case, finding: &str) {
    let root = case.fixture.root();
    let old_c1 = emitted_item(root, "old-queue.json", "c1");
    let old_c2 = emitted_item(root, "old-queue.json", "c2");
    let new_c1 = emitted_item(root, "new-queue.json", "c1");
    let new_c2 = emitted_item(root, "new-queue.json", "c2");
    let mut rows = vec![
        links::LinkRequest {
            old_item_id: old_c1.clone(),
            new_item_id: new_c1.clone(),
            finding_ids: vec![finding.to_owned()],
        },
        links::LinkRequest {
            old_item_id: old_c1,
            new_item_id: new_c2,
            finding_ids: vec![finding.to_owned()],
        },
        links::LinkRequest { old_item_id: old_c2, new_item_id: new_c1, finding_ids: Vec::new() },
    ];
    rows.sort_by(|left, right| {
        (left.old_item_id.as_str(), left.new_item_id.as_str())
            .cmp(&(right.old_item_id.as_str(), right.new_item_id.as_str()))
    });
    write(
        root,
        "links.json",
        &serde_json::to_value(links::LinksRequest {
            schema_version: links::LINKS_SCHEMA.to_owned(),
            links: rows,
        })
        .unwrap(),
    );
}

/// Actual native many-to-many lineage keeps memberships, occurrences and complete distinct findings separate.
#[test]
fn genuine_many_to_many_native_app_bindings_preserve_all_graph_and_finding_denominators() {
    for profile in [false, true] {
        let case = case_with_items(profile, true, &["c1", "c2"]);
        let finding = case
            .report
            .findings
            .iter()
            .find(|finding| {
                finding.reason_code == ReasonCode::ApplicabilityDecisionChanged
                    && finding.subject_id == "c1"
            })
            .unwrap();
        many_to_many(&case, &finding.finding_id);
        let old_raw = std::fs::read(case.fixture.root().join("old-queue.json")).unwrap();
        let new_raw = std::fs::read(case.fixture.root().join("new-queue.json")).unwrap();
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture(&case, &mut ledger, &mut control);
        let (old, new, links) = queues(&held, &mut ledger, &mut control);
        let bound = prepare(&held, &old, &new, &links, &mut ledger, &mut control).unwrap();
        let counts = bound.graph().counts();
        assert_eq!((counts.old_items, counts.new_items, counts.link_edges), (2, 2, 3));
        assert_eq!((counts.linked_old_items, counts.linked_new_items), (2, 2));
        assert_eq!((counts.unmatched_old_items, counts.unmatched_new_items), (0, 0));
        assert_eq!((counts.link_finding_occurrences, counts.distinct_requested_findings), (2, 1));
        assert!(
            bound.graph().unmatched_old().is_empty() && bound.graph().unmatched_new().is_empty()
        );
        assert_eq!(bound.graph().requested_findings(), &[finding.finding_id.as_str()]);
        assert_eq!(bound.linked_findings().len(), 3);
        let mut repeated = Vec::new();
        for (edge, findings) in bound.graph().links().iter().zip(bound.linked_findings()) {
            match (edge.old_item().subject_id.as_str(), edge.new_item().subject_id.as_str()) {
                ("c1", "c1" | "c2") => {
                    assert_eq!(findings.len(), 1);
                    assert_eq!(findings[0].finding_id, finding.finding_id);
                    assert_eq!(findings[0].subject_id, "c1");
                    repeated.push(findings[0]);
                }
                ("c2", "c1") => assert!(findings.is_empty()),
                _ => panic!("no inferred or extra native lineage pair"),
            }
        }
        assert_eq!(repeated.len(), 2);
        assert!(
            std::ptr::eq(repeated[0], repeated[1]),
            "same genuine held native finding repeated on two explicit edges"
        );
        let mut expected: Vec<_> = case
            .report
            .findings
            .iter()
            .filter(|row| row.finding_id != finding.finding_id)
            .map(|row| row.finding_id.as_str())
            .collect();
        expected.sort_unstable();
        assert_eq!(bound.unreferenced_findings(), expected.as_slice());
        assert_eq!(
            bound.unreferenced_findings().len() + counts.distinct_requested_findings,
            case.report.findings.len()
        );
        assert_ne!(counts.link_edges, counts.linked_old_items);
        assert_ne!(counts.link_finding_occurrences, counts.distinct_requested_findings);
        bound.verify_inputs(&mut ledger, &mut control).unwrap();
        assert_eq!(std::fs::read(case.fixture.root().join("old-queue.json")).unwrap(), old_raw);
        assert_eq!(std::fs::read(case.fixture.root().join("new-queue.json")).unwrap(), new_raw);
        assert!(!case.fixture.root().join("out.json").exists());
    }
}

/// Calibrate genuine post-Binding fences on one owner/ledger/control; preserve each first typed stop.
#[test]
fn genuine_native_binding_final_checkpoint_preserves_actual_interruption_and_control_failure() {
    use crate::workspace::preparation::{Interruption, Stage};
    for fault in [BindingFault::Interrupted, BindingFault::Failed] {
        let case = case(false, true);
        let unrelated = case
            .report
            .findings
            .iter()
            .find(|finding| finding.reason_code == ReasonCode::MappingSubjectChanged)
            .unwrap();
        link(&case, Some(&unrelated.finding_id));
        let state = Rc::new(std::cell::RefCell::new(BindingStopState::default()));
        let mut caller = BindingStop { state: Rc::clone(&state) };
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let held = capture(&case, &mut ledger, &mut control);
        let (old, new, links) = queues(&held, &mut ledger, &mut control);
        let before = state.borrow().calls;
        assert!(matches!(
            prepare(&held, &old, &new, &links, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert_eq!(
            ledger.bound(|_| Ok(())),
            Ok(()),
            "ordinary Binding is not the first sticky stop"
        );
        let observed = state.borrow().calls;
        let actual_phase_checkpoints = observed - before;
        assert!(actual_phase_checkpoints > 1);
        assert_eq!(state.borrow().last_stage, Some(Stage::PrepareDomain));
        let selected = observed + actual_phase_checkpoints;
        {
            let mut state = state.borrow_mut();
            state.selected = Some(selected);
            state.fault = Some(fault);
        }
        let result = prepare(&held, &old, &new, &links, &mut ledger, &mut control);
        let expected = match fault {
            BindingFault::Interrupted => ContractError::Interrupted(Interruption::CancelRequested),
            BindingFault::Failed => ContractError::ControlFailed,
        };
        assert!(matches!(result, Err(error) if error == expected));
        assert!(
            state.borrow().fired,
            "the actual post-ordinary-Binding checkpoint must be reached"
        );
        assert_eq!(state.borrow().calls, selected);
        assert_eq!(state.borrow().last_stage, Some(Stage::PrepareDomain));
        assert_eq!(ledger.bound::<()>(|_| Err(ContractError::Binding)), Err(expected));
        assert!(
            matches!(prepare(&held, &old, &new, &links, &mut ledger, &mut control), Err(error) if error == expected)
        );
        assert_eq!(ledger.checkpoint(&mut control), Err(expected));
        assert_eq!(
            state.borrow().calls,
            selected,
            "a sticky first ledger stop bypasses every later caller checkpoint"
        );
        if matches!(fault, BindingFault::Interrupted) {
            assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
        }
        assert!(!case.fixture.root().join("out.json").exists());
    }
}

/// A real original-ledger capacity refusal outranks a later actual accepted-control stop and Binding.
#[test]
fn genuine_native_owner_original_capacity_wins_over_later_caller_stop_and_ordinary_binding() {
    use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkError};
    for fault in [BindingFault::Interrupted, BindingFault::Failed] {
        let case = case(false, true);
        let unrelated = case
            .report
            .findings
            .iter()
            .find(|finding| finding.reason_code == ReasonCode::MappingSubjectChanged)
            .unwrap();
        link(&case, Some(&unrelated.finding_id));
        let state = Rc::new(std::cell::RefCell::new(BindingStopState::default()));
        let mut caller = BindingStop { state: Rc::clone(&state) };
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let held = capture(&case, &mut ledger, &mut control);
        let (old, new, links) = queues(&held, &mut ledger, &mut control);
        assert!(matches!(
            prepare(&held, &old, &new, &links, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert_eq!(ledger.bound(|_| Ok(())), Ok(()));
        // Invoke the real unchanged derived-payload guard on the same already-used
        // command ledger. No cap is lowered, source input removed or owner reset.
        assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
        let selected = state.borrow().calls + 1;
        {
            let mut state = state.borrow_mut();
            state.selected = Some(selected);
            state.fault = Some(fault);
        }
        let later = control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged);
        match fault {
            BindingFault::Interrupted => {
                assert!(matches!(
                    later,
                    Err(WorkError::Interrupted(Interruption::CancelRequested))
                ));
            }
            BindingFault::Failed => assert!(matches!(later, Err(WorkError::Failed(_)))),
        }
        assert!(state.borrow().fired);
        assert_eq!(
            ledger.bound::<()>(|_| Err(ContractError::Binding)),
            Err(ContractError::Capacity)
        );
        assert!(matches!(
            prepare(&held, &old, &new, &links, &mut ledger, &mut control),
            Err(ContractError::Capacity)
        ));
        assert_eq!(ledger.checkpoint(&mut control), Err(ContractError::Capacity));
        assert_eq!(state.borrow().calls, selected);
        assert!(!case.fixture.root().join("out.json").exists());
    }
}

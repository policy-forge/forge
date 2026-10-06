//! Genuine Mapping-specific complete owner/native finding controls; no proof constructors.
//! All native products and recorded lifecycle fixtures remain explicitly synthetic test facts.

use super::*;
use crate::framework::model::ImpactReport;
use crate::review::capture::supersession::seal_complete_union;
use crate::review::capture::{ReviewCapture, ReviewControl};
use crate::review::{decode, encode, links, mapping_capture, validate, wire};
use crate::workspace::preparation::NoopControl;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Own real files plus the maintained file-produced report; neither field grants capture authority.
struct MappingCase {
    /// Actual temporary tree lifetime retained through genuine capture/binding.
    directory: tempfile::TempDir,
    /// Canonical project root used only for test file orchestration.
    root: PathBuf,
    /// Full report read back from the actual maintained native producer's output.
    report: ImpactReport,
}

/// Obtain only fixture data through its existing private owner; all actual proofs are issued later.
fn mapping_case(profile: bool, changed: bool, second_map: bool) -> MappingCase {
    let (directory, root) =
        mapping_capture::tests::supersession_mapping_fixture(profile, changed, second_map);
    assert_eq!(directory.path().canonicalize().unwrap(), root);
    let report =
        serde_json::from_slice(&std::fs::read(root.join("impact-report.json")).unwrap()).unwrap();
    MappingCase { directory, root, report }
}

/// Write explicit test requests before capture, never any expected native/capture success value.
fn write(root: &Path, name: &str, value: &Value) {
    std::fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

/// Read the whole generated historical declaration to select its actual first native map UUID.
fn old_subject(case: &MappingCase) -> String {
    let old: wire::QueueDocument =
        serde_json::from_slice(&std::fs::read(case.root.join("old-queue.json")).unwrap()).unwrap();
    old.items[0].subject_id.clone()
}

/// Choose a real current finding for the selected map or a different genuinely present map.
fn mapping_finding(case: &MappingCase, wrong_dependency: bool) -> &ImpactFinding {
    let selected = old_subject(case);
    case.report
        .findings
        .iter()
        .find(|finding| {
            finding.reason_code == ReasonCode::MappingSubjectChanged
                && finding.subject_id == "a-1"
                && finding
                    .dependency_id
                    .as_deref()
                    .is_some_and(|id| (id != selected) == wrong_dependency)
        })
        .expect("maintained producer emitted the intended actual map dependency")
}

/// Select actual generated old/new endpoints and an explicit real finding ID only.
fn mapping_link(case: &MappingCase, finding: Option<&str>) {
    let old: Value =
        serde_json::from_slice(&std::fs::read(case.root.join("old-queue.json")).unwrap()).unwrap();
    let new: Value =
        serde_json::from_slice(&std::fs::read(case.root.join("new-queue.json")).unwrap()).unwrap();
    write(
        &case.root,
        "links.json",
        &json!({"schema_version":links::LINKS_SCHEMA,"links":[{
        "old_item_id":old["items"][0]["item_id"], "new_item_id":new["items"][0]["item_id"],
        "finding_ids":finding.map(|id|vec![id]).unwrap_or_default()}]}),
    );
}

/// Actual Mapping pending/current native and complete Impact cohorts consume the same capture owner.
fn capture_mapping(
    case: &MappingCase,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> HeldSupersessionInputs {
    assert!(case.directory.path().is_dir());
    let mut capture =
        ReviewCapture::new_supersession(&case.root, Path::new("mapping-out.json"), ledger, control)
            .unwrap();
    for (name, purpose) in [
        ("old-queue.json", QueuePurpose::HistoricalOld),
        ("new-queue.json", QueuePurpose::CurrentNew),
    ] {
        capture.required_supersession_queue(Path::new(name), purpose, ledger, control).unwrap();
    }
    for (name, purpose) in [
        ("locator.json", AuxiliaryPurpose::NewNativeLocator),
        ("impact-locator.json", AuxiliaryPurpose::ImpactLocator),
        ("links.json", AuxiliaryPurpose::Links),
    ] {
        capture.required_supersession_auxiliary(Path::new(name), purpose, ledger, control).unwrap();
    }
    let locator =
        capture.supersession_auxiliary_original(AuxiliaryPurpose::NewNativeLocator).unwrap();
    let native = mapping_capture::prepare(&mut capture, locator, ledger, control).unwrap();
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

/// Decode only exact successfully registered whole originals from the real complete union owner.
fn mapping_queues<'a>(
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
    let request = links::decode_links(
        held.held_inputs()
            .bytes(held.auxiliary_original(AuxiliaryPurpose::Links).unwrap())
            .unwrap(),
        ledger,
        control,
    )
    .unwrap();
    (old, new, request)
}

/// Recompute genuine structural hashes after an intentional declaration mutation, before capture.
/// Native/report originals are unchanged; this cannot create native approval/currentness evidence.
fn rewrite_queue(case: &MappingCase, name: &str, mutate: impl FnOnce(&mut wire::QueueDocument)) {
    let path = case.root.join(name);
    let mut queue: wire::QueueDocument =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    mutate(&mut queue);
    let mut items = std::mem::take(&mut queue.items);
    let mut ledger = ContractLedger::default();
    for item in &mut items {
        item.context_sha256 = validate::context_hash(&queue, item, &mut ledger).unwrap();
        let policy = queue.policies.iter().find(|policy| policy.key == item.policy_key).unwrap();
        item.policy_sha256 = validate::policy_hash(&queue, policy, item, &mut ledger).unwrap();
        item.item_id = validate::item_id(&queue, item, &mut ledger).unwrap();
    }
    queue.items = items;
    let raw = encode::queue(&queue, &mut ledger, &mut NoopControl).unwrap();
    decode::decode_queue(&raw, &mut ledger, &mut NoopControl).unwrap();
    std::fs::write(path, raw).unwrap();
}

/// Genuine Catalog/Profile Mapping queue originals bind the complete current native adapter.
#[test]
fn genuine_mapping_catalog_and_profile_current_adapters_bind_complete_queues() {
    for profile in [false, true] {
        let case = mapping_case(profile, false, false);
        mapping_link(&case, None);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture_mapping(&case, &mut ledger, &mut control);
        let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
        assert!(matches!(held.new_native(), NewNativeCohort::Mapping(_)));
        let bound = prepare(&held, &old, &new, &request, &mut ledger, &mut control).unwrap();
        assert_eq!(bound.graph().links().len(), 1);
        assert!(bound.linked_findings()[0].is_empty());
        let no_findings: [&str; 0] = [];
        assert_eq!(bound.unreferenced_findings(), no_findings);
        assert_eq!(held.impact().facts().report.summary.unchanged, 1);
        bound.verify_inputs(&mut ledger, &mut control).unwrap();
        assert!(!case.root.join("mapping-out.json").exists());
    }
}

/// Full genuine Mapping finding evidence binds its exact archived collection UUID and old map UUID.
#[test]
fn genuine_mapping_findings_bind_archived_collection_and_old_map_uuid() {
    for profile in [false, true] {
        let case = mapping_case(profile, true, false);
        let finding = mapping_finding(&case, false);
        let archived: Value =
            serde_json::from_slice(&std::fs::read(case.root.join("old/mapping.json")).unwrap())
                .unwrap();
        assert_eq!(
            finding.affected_artifact_id.as_deref(),
            archived["mapping-collection"]["uuid"].as_str()
        );
        assert_eq!(finding.dependency_id.as_deref(), Some(old_subject(&case).as_str()));
        assert!(finding.old_subjects.iter().any(|subject| subject.id == "a-1"));
        mapping_link(&case, Some(&finding.finding_id));
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture_mapping(&case, &mut ledger, &mut control);
        let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
        let bound = prepare(&held, &old, &new, &request, &mut ledger, &mut control).unwrap();
        assert_eq!(bound.linked_findings()[0][0].finding_id, finding.finding_id);
        assert_eq!(
            bound.unreferenced_findings().len() + 1,
            held.impact().facts().report.findings.len()
        );
        bound.verify_inputs(&mut ledger, &mut control).unwrap();
        assert!(!case.root.join("old/target.json").exists());
        assert!(!case.root.join("old/manifest.json").exists());
        assert!(!case.root.join("mapping-out.json").exists());
    }
}

/// Syntactically complete historical Mapping tuples with false hash/length/root/schema refuse native binding.
#[test]
fn historical_mapping_pin_forgery_is_refused_after_complete_structural_decode() {
    for profile in [false, true] {
        for field in ["hash", "length", "root", "schema"] {
            let case = mapping_case(profile, true, false);
            let finding = mapping_finding(&case, false);
            rewrite_queue(&case, "old-queue.json", |queue| {
                let pin =
                    queue.source_pins.iter_mut().find(|pin| pin.artifact_key == "mapping").unwrap();
                match field {
                    "hash" => pin.raw_sha256 = "f".repeat(64),
                    "length" => pin.byte_length += 1,
                    "root" => {
                        pin.native_root_uuid = Some("dddddddd-dddd-4ddd-8ddd-dddddddddddd".into());
                    }
                    "schema" => pin.schema_identity = "oscal:1.2.3:catalog".into(),
                    _ => unreachable!(),
                }
            });
            mapping_link(&case, Some(&finding.finding_id));
            let mut ledger = ContractLedger::default();
            let mut caller = NoopControl;
            let mut control = ReviewControl::accept(&mut caller);
            let held = capture_mapping(&case, &mut ledger, &mut control);
            let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
            assert!(matches!(
                prepare(&held, &old, &new, &request, &mut ledger, &mut control),
                Err(ContractError::Binding)
            ));
            assert!(!case.root.join("mapping-out.json").exists());
        }
    }
}

/// An existing native finding for a second real map cannot bind the first actual historical endpoint.
#[test]
fn real_second_map_dependency_finding_is_refused_for_wrong_old_endpoint() {
    for profile in [false, true] {
        let case = mapping_case(profile, true, true);
        let intended = mapping_finding(&case, false);
        let wrong = mapping_finding(&case, true);
        assert_eq!(intended.affected_artifact_id, wrong.affected_artifact_id);
        assert_ne!(intended.dependency_id, wrong.dependency_id);
        assert_eq!(intended.subject_id, wrong.subject_id);
        assert_eq!(
            case.report
                .findings
                .iter()
                .filter(|finding| finding.reason_code == ReasonCode::MappingSubjectChanged)
                .count(),
            2
        );
        mapping_link(&case, Some(&wrong.finding_id));
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture_mapping(&case, &mut ledger, &mut control);
        let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
        assert!(
            held.impact()
                .facts()
                .report
                .findings
                .iter()
                .any(|finding| finding.finding_id == wrong.finding_id)
        );
        assert!(matches!(
            prepare(&held, &old, &new, &request, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert!(!case.root.join("mapping-out.json").exists());
    }
}

/// The explicit old Profile resolved companion pin must equal the actual archived native original.
#[test]
fn historical_profile_companion_pin_forgery_is_refused() {
    let case = mapping_case(true, true, false);
    let finding = mapping_finding(&case, false);
    rewrite_queue(&case, "old-queue.json", |queue| {
        queue
            .source_pins
            .iter_mut()
            .find(|pin| pin.artifact_key == "source-resolved")
            .unwrap()
            .raw_sha256 = "f".repeat(64);
    });
    mapping_link(&case, Some(&finding.finding_id));
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let held = capture_mapping(&case, &mut ledger, &mut control);
    let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
    assert!(matches!(
        prepare(&held, &old, &new, &request, &mut ledger, &mut control),
        Err(ContractError::Binding)
    ));
    assert!(!case.root.join("mapping-out.json").exists());
}

/// A structurally coherent false new queue subject digest cannot bypass the actual current Mapping adapter.
#[test]
fn new_mapping_subject_forgery_is_refused_by_genuine_current_native_adapter() {
    for profile in [false, true] {
        let case = mapping_case(profile, false, false);
        rewrite_queue(&case, "new-queue.json", |queue| {
            queue.items[0].subject_sha256 = "f".repeat(64);
        });
        mapping_link(&case, None);
        let mut ledger = ContractLedger::default();
        let mut caller = NoopControl;
        let mut control = ReviewControl::accept(&mut caller);
        let held = capture_mapping(&case, &mut ledger, &mut control);
        let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
        assert!(matches!(
            prepare(&held, &old, &new, &request, &mut ledger, &mut control),
            Err(ContractError::Binding)
        ));
        assert!(!case.root.join("mapping-out.json").exists());
    }
}

/// Real archived/current Mapping byte drift after genuine success refuses the actual retained owner fence.
#[test]
fn archived_and_current_mapping_original_drift_refuses_final_binding_fence() {
    for profile in [false, true] {
        for path in ["old/mapping.json", "mapping.json"] {
            let case = mapping_case(profile, true, false);
            let finding = mapping_finding(&case, false);
            mapping_link(&case, Some(&finding.finding_id));
            let mut ledger = ContractLedger::default();
            let mut caller = NoopControl;
            let mut control = ReviewControl::accept(&mut caller);
            let held = capture_mapping(&case, &mut ledger, &mut control);
            let (old, new, request) = mapping_queues(&held, &mut ledger, &mut control);
            let bound = prepare(&held, &old, &new, &request, &mut ledger, &mut control).unwrap();
            let original = case.root.join(path);
            let mut raw = std::fs::read(&original).unwrap();
            raw.push(b' ');
            std::fs::write(original, raw).unwrap();
            assert!(matches!(
                bound.verify_inputs(&mut ledger, &mut control),
                Err(ContractError::Binding)
            ));
            assert!(!case.root.join("mapping-out.json").exists());
        }
    }
}

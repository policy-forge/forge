//! Genuine complete command controls under the existing cfg-only native App fixture owner.
//! Actual file products, queues and native proofs are issued by maintained consumers.

use super::*;
use crate::review::commands::{CommandError, SupersedeOptions};
use crate::workspace::preparation::{Interruption, ProgressUpdate, Stage, WorkError, WorkResult};

/// Borrow only explicit file declarations for the actual complete public command.
fn options(root: &Path) -> SupersedeOptions<'_> {
    SupersedeOptions {
        project_root: root,
        old_queue: Path::new("old-queue.json"),
        new_queue: Path::new("new-queue.json"),
        sources: Path::new("locator.json"),
        impact: Path::new("impact-locator.json"),
        links: Path::new("links.json"),
        supersession_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        created_at: "2026-10-05T00:00:00Z",
        output: Path::new("supersession.json"),
    }
}

/// Check only the exact default public whitelist, never treating recorded labels as proof.
fn output(root: &Path) -> Value {
    let bytes = std::fs::read(root.join("supersession.json")).unwrap();
    assert!(bytes.ends_with(b"\n"));
    assert!(!bytes[..bytes.len() - 1].contains(&b'\n'));
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 15);
    for side in ["old", "new"] {
        assert_eq!(value["impact"][side].as_object().unwrap().len(), 5);
    }
    for field in ["group", "decision_state", "policy_source", "priority", "owner"] {
        assert!(value["impact"]["filters"][field].is_null());
    }
    let text = std::str::from_utf8(&bytes).unwrap();
    for private in [
        "document_version",
        "dependency_path",
        "\"href\"",
        "\"rationale\"",
        "Synthetic",
        "old/framework.json",
        "lifecycle.json",
    ] {
        assert!(!text.contains(private));
    }
    value
}

/// Actual Catalog/Profile App commands publish complete minimized no-change records and preserve history.
#[test]
fn complete_app_no_change_command_publishes_minimized_record_and_preserves_queues() {
    for profile in [false, true] {
        let case = case(profile, false);
        link(&case, None);
        let root = case.fixture.root();
        let old = std::fs::read(root.join("old-queue.json")).unwrap();
        let new = std::fs::read(root.join("new-queue.json")).unwrap();
        commands::supersede(&options(root), &mut NoopControl).unwrap();
        let value = output(root);
        assert_eq!(
            value["impact"]["native_change_observation"],
            json!("no-detected-native-change")
        );
        assert!(value["impact"]["change_count"].as_u64().unwrap() > 0);
        assert_eq!(value["impact"]["finding_count"], json!(0));
        assert_eq!(value["counts"]["link_edges"], json!(1));
        assert_eq!(value["old_queue"]["raw_sha256"], json!(crate::hashing::sha256_hex(&old)));
        assert_eq!(value["new_queue"]["raw_sha256"], json!(crate::hashing::sha256_hex(&new)));
        assert_eq!(std::fs::read(root.join("old-queue.json")).unwrap(), old);
        assert_eq!(std::fs::read(root.join("new-queue.json")).unwrap(), new);
        assert_eq!(value["old_queue_currentness"], json!("historical-unverified"));
        assert_eq!(value["semantics"], json!("declared-lineage-no-response-transfer"));
    }
}

/// Actual changed App native findings survive finite encoding and recorded decoder binding.
#[test]
fn complete_app_changed_command_publishes_genuine_associated_finding_and_full_counts() {
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
        commands::supersede(&options(case.fixture.root()), &mut NoopControl).unwrap();
        let value = output(case.fixture.root());
        assert_eq!(value["links"][0]["findings"][0]["finding_id"], json!(finding.finding_id));
        assert_eq!(
            value["links"][0]["findings"][0]["old_dependency_binding"],
            json!("captured-native-control-id-and-declared-old-pin")
        );
        assert_eq!(value["impact"]["finding_count"], json!(case.report.findings.len()));
        assert_eq!(value["impact"]["matched_findings"], json!(case.report.findings.len()));
        assert_eq!(value["counts"]["distinct_linked_findings"], json!(1));
        assert_eq!(
            value["counts"]["unreferenced_findings"].as_u64().unwrap() + 1,
            case.report.findings.len() as u64
        );
        assert_eq!(value["impact"]["native_change_observation"], json!("detected-native-change"));
    }
}

/// Both actual Mapping native families publish genuine current findings without old policy routes.
#[test]
fn complete_mapping_catalog_and_profile_commands_publish_genuine_changed_findings() {
    for profile in [false, true] {
        let (_directory, root) =
            mapping_capture::tests::supersession_mapping_fixture(profile, true, false);
        let old: Value =
            serde_json::from_slice(&std::fs::read(root.join("old-queue.json")).unwrap()).unwrap();
        let new: Value =
            serde_json::from_slice(&std::fs::read(root.join("new-queue.json")).unwrap()).unwrap();
        let report: crate::framework::model::ImpactReport =
            serde_json::from_slice(&std::fs::read(root.join("impact-report.json")).unwrap())
                .unwrap();
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.reason_code == ReasonCode::MappingSubjectChanged
                    && finding.dependency_id.as_deref() == old["items"][0]["subject_id"].as_str()
            })
            .unwrap();
        write(
            &root,
            "links.json",
            &json!({"schema_version":links::LINKS_SCHEMA,"links":[{"old_item_id":old["items"][0]["item_id"],"new_item_id":new["items"][0]["item_id"],"finding_ids":[finding.finding_id]}]}),
        );
        commands::supersede(&options(&root), &mut NoopControl).unwrap();
        let value = output(&root);
        assert_eq!(value["links"][0]["findings"][0]["finding_id"], json!(finding.finding_id));
        assert_eq!(
            value["links"][0]["findings"][0]["old_dependency_binding"],
            json!("captured-native-map-id-and-declared-old-pin")
        );
        assert!(!root.join("old/manifest.json").exists());
        assert!(!root.join("old/target.json").exists());
    }
}

/// Actual existing output, false full native report and contradictory declared chronology refuse.
#[test]
fn complete_command_refusals_preserve_originals_and_never_replace_existing_output() {
    for mode in 0..3 {
        let case = case(false, false);
        link(&case, None);
        let root = case.fixture.root();
        let mut options = options(root);
        if mode == 0 {
            std::fs::write(root.join("supersession.json"), b"preserved\n").unwrap();
        }
        if mode == 1 {
            let mut report: Value =
                serde_json::from_slice(&std::fs::read(root.join("impact-report.json")).unwrap())
                    .unwrap();
            report["summary"]["unchanged"] = json!(99);
            write(root, "impact-report.json", &report);
        }
        if mode == 2 {
            options.created_at = "2026-10-03T00:00:00Z";
        }
        assert!(commands::supersede(&options, &mut NoopControl).is_err());
        if mode == 0 {
            assert_eq!(std::fs::read(root.join("supersession.json")).unwrap(), b"preserved\n");
        } else {
            assert!(!root.join("supersession.json").exists());
        }
    }
}

/// Observe only the genuine publisher's actual private staging directory.
/// No production test hook, caller proof or replacement native owner is installed.
struct PublicationStop<'a> {
    /// Actual test root, held by the full synthetic native fixture owner.
    root: &'a Path,
    /// Explicit real final-fence fault: drift, interruption, control failure or destination race.
    mode: u8,
    /// Whether the actual staging/publication boundary was reached.
    fired: bool,
}

impl WorkControl for PublicationStop<'_> {
    /// Inject only after real private staging, while the complete consumer owner is still alive.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if self.fired {
            return Ok(());
        }
        let staged = std::fs::read_dir(self.root).unwrap().any(|entry| {
            entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")
        });
        if staged {
            self.fired = true;
            match self.mode {
                0 => {
                    let path = self.root.join("old/framework.json");
                    let mut raw = std::fs::read(&path).unwrap();
                    raw.push(b' ');
                    std::fs::write(path, raw).unwrap();
                }
                1 => return Err(WorkError::Interrupted(Interruption::CancelRequested)),
                2 => return Err(WorkError::Failed(crate::workspace::contract::Error::invalid())),
                3 => std::fs::write(self.root.join("supersession.json"), b"raced-preserved\n")
                    .unwrap(),
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    /// Transfer the same real first interruption through the accepted `ReviewControl` fence.
    fn interruption(&self) -> Option<Interruption> {
        if self.fired && self.mode == 1 { Some(Interruption::CancelRequested) } else { None }
    }
}

/// The complete command retains real generation/control fences immediately before native rename.
#[test]
fn actual_guarded_publication_refuses_drift_and_preserves_first_stops_and_raced_destination() {
    for mode in 0..4 {
        let case = case(false, false);
        link(&case, None);
        let root = case.fixture.root();
        let mut control = PublicationStop { root, mode, fired: false };
        let result = commands::supersede(&options(root), &mut control);
        assert!(control.fired, "the actual guarded publication boundary was not reached");
        match mode {
            0 => assert!(matches!(result, Err(CommandError::Contract(ContractError::Binding)))),
            1 => assert!(matches!(
                result,
                Err(CommandError::Contract(ContractError::Interrupted(
                    Interruption::CancelRequested
                )))
            )),
            2 => {
                assert!(matches!(
                    result,
                    Err(CommandError::Contract(ContractError::ControlFailed))
                ));
            }
            3 => assert!(matches!(result, Err(CommandError::Publication))),
            _ => unreachable!(),
        }
        if mode == 3 {
            assert_eq!(
                std::fs::read(root.join("supersession.json")).unwrap(),
                b"raced-preserved\n"
            );
        } else {
            assert!(!root.join("supersession.json").exists());
        }
        assert!(!std::fs::read_dir(root).unwrap().any(|entry| {
            entry.unwrap().file_name().to_string_lossy().starts_with(".forge-authoring-file-")
        }));
    }
}

/// Exact public clap flags dispatch the actual complete source/record/publication consumer.
#[test]
fn public_supersede_cli_dispatches_genuine_complete_native_command() {
    use clap::Parser;
    let case = case(false, false);
    link(&case, None);
    let root = case.fixture.root().to_str().unwrap();
    let cli = crate::cli::Cli::try_parse_from([
        "forge",
        "review",
        "supersede",
        "--project-root",
        root,
        "--old-queue",
        "old-queue.json",
        "--new-queue",
        "new-queue.json",
        "--sources",
        "locator.json",
        "--impact",
        "impact-locator.json",
        "--links",
        "links.json",
        "--supersession-id",
        "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        "--created-at",
        "2026-10-05T00:00:00Z",
        "--output",
        "supersession.json",
    ])
    .unwrap();
    crate::cli::execute(&cli).unwrap();
    output(case.fixture.root());
}

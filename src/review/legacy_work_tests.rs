//! Genuine borrowed-engine controls under one actual review ledger and accepted control.

use super::*;
use crate::framework::analysis::legacy_borrowed::FrameworkBytes;
use crate::review::capture::ReviewControl;
use crate::workspace::contract::Error;
use serde_json::json;

/// Held native originals and a maintained applicability inventory scaffold.
struct Fixture {
    /// Actual root retained for the complete component run.
    directory: tempfile::TempDir,
    /// Complete old native Catalog original.
    old: Vec<u8>,
    /// Complete new native Catalog original.
    new: Vec<u8>,
    /// Complete maintained Impact declaration.
    impact: Vec<u8>,
    /// Complete applicability declaration produced by `execute_init`.
    applicability: Vec<u8>,
}

impl Fixture {
    /// Generate valid native originals without constructing inventory or approval facts.
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("held native fixture root");
        let uuid = "11111111-1111-4111-8111-111111111111";
        let catalog = |version: &str, prose: &str| json!({"catalog":{"uuid":uuid,"metadata":{"title":"Synthetic native component control","last-modified":"2026-08-25T12:00:00Z","version":version,"oscal-version":"1.2.3"},"controls":[{"id":"control-1","title":"Synthetic control","parts":[{"id":"control-1_smt","name":"statement","prose":prose}]}]}});
        let old = serde_json::to_vec(&catalog("1.0.0", "Old content")).unwrap();
        let new = serde_json::to_vec(&catalog("2.0.0 private-name", "New content")).unwrap();
        std::fs::write(directory.path().join("old.json"), &old).unwrap();
        std::fs::write(directory.path().join("new.json"), &new).unwrap();
        let resource = |name: &str, raw: &[u8], version: &str| json!({"type":"catalog","artifact":name,"expected_sha256":crate::hashing::sha256_hex(raw),"root_uuid":uuid,"document_version":version,"oscal_version":"1.2.3"});
        let impact = serde_json::to_vec(&json!({"schema_version":"forge.framework-impact/1","old":resource("old.json",&old,"1.0.0"),"new":resource("new.json",&new,"2.0.0 private-name"),"mapping_collections":[]})).unwrap();
        let app_path = directory.path().join("applicability.json");
        crate::applicability::execute_init(
            &directory.path().join("old.json"),
            None,
            Some(&app_path),
        )
        .expect("genuine maintained inventory scaffold");
        let applicability = std::fs::read(app_path).unwrap();
        Self { directory, old, new, impact, applicability }
    }

    /// Borrow the complete ordinary Impact roster; this creates no original proof.
    fn impact_inputs(&self) -> ImpactInputs<'_> {
        ImpactInputs {
            manifest_dir: self.directory.path(),
            manifest: &self.impact,
            old: FrameworkBytes { artifact: &self.old, resolved_catalog: None },
            new: FrameworkBytes { artifact: &self.new, resolved_catalog: None },
            mappings: &[],
            applicability: None,
            successor_map: None,
            prior_report: None,
            dispositions: None,
        }
    }

    /// Borrow the actual scaffold and declared framework without stronger endpoint inference.
    fn applicability_inputs(&self) -> LegacyApplicabilityInputs<'_> {
        LegacyApplicabilityInputs {
            manifest_bytes: &self.applicability,
            manifest_dir: self.directory.path(),
            framework_bytes: &self.old,
            resolved_catalog_bytes: None,
            mapping_bytes: &[],
        }
    }
}

/// Actual safe caller failure selected at one observed checkpoint.
#[derive(Clone, Copy)]
enum Failure {
    /// One actual ordinary `WorkError`, followed by accepting caller behavior.
    Ordinary,
    /// One actual typed interruption.
    Interrupted(Interruption),
}

/// Count actual native checkpoints and optionally refuse a calibrated real boundary.
#[derive(Default)]
struct Caller {
    /// Complete number of checkpoints observed by this original caller.
    calls: usize,
    /// One-based real checkpoint and actual result to return.
    fail_at: Option<(usize, Failure)>,
}

impl WorkControl for Caller {
    /// Return the configured actual safe error without fabricating native results.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.calls += 1;
        match self.fail_at {
            Some((at, Failure::Ordinary)) if at == self.calls => {
                Err(WorkError::Failed(Error::invalid()))
            }
            Some((at, Failure::Interrupted(reason))) if at == self.calls => {
                Err(WorkError::Interrupted(reason))
            }
            _ => Ok(()),
        }
    }

    /// `ReviewControl` retains actual typed interruptions; ordinary failures stay ledger-owned.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

/// Obtain a fixed contract failure without requiring Debug on complete private native facts.
fn failure<T>(result: Result<T, BorrowedPreparationError>) -> ContractError {
    match result {
        Err(error) => error.into_contract(),
        Ok(_) => panic!("actual preparation must refuse"),
    }
}

/// Compare complete genuine native output from both ordinary file producers.
#[test]
fn shared_work_full_native_parity_retains_private_reports() {
    let fixture = Fixture::new();
    let mut ledger = ContractLedger::default();
    let mut caller = Caller::default();
    let mut control = ReviewControl::accept(&mut caller);
    let app = prepare_applicability(
        fixture.applicability_inputs(),
        ReportFilters::default(),
        &mut ledger,
        &mut control,
    )
    .unwrap();
    let ordinary_app = crate::applicability::prepare_analysis(
        &fixture.directory.path().join("applicability.json"),
        ReportFilters::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&app.report).unwrap(),
        serde_json::to_value(&ordinary_app.report).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&app.manifest).unwrap(),
        serde_json::to_value(&ordinary_app.manifest).unwrap()
    );
    let impact = prepare_impact(
        fixture.impact_inputs(),
        ImpactFilters::default(),
        &mut ledger,
        &mut control,
    )
    .unwrap();
    let declaration = crate::framework::manifest::parse(&fixture.impact).unwrap();
    let (ordinary, _) = crate::framework::analysis::analyze(
        fixture.directory.path(),
        &declaration,
        ImpactFilters::default(),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&impact.report).unwrap(),
        serde_json::to_value(&ordinary).unwrap()
    );
    assert_eq!(impact.report.summary.rows(), ordinary.summary.rows());
    assert_eq!(impact.report.old, ordinary.old);
    assert_eq!(impact.report.new, ordinary.new);
    assert!(impact.applicability_manifest.is_none());
    assert_eq!(
        serde_json::to_value(&impact.manifest).unwrap(),
        serde_json::to_value(&declaration).unwrap()
    );
    ledger.checkpoint(&mut control).unwrap();
}

/// A saved actual capacity refusal blocks both native engines and every later caller probe.
#[test]
fn shared_work_prior_capacity_refuses_without_new_native_entry() {
    let fixture = Fixture::new();
    let mut ledger = ContractLedger::default();
    assert_eq!(ledger.derived(33_554_433), Err(ContractError::Capacity));
    let mut caller = Caller { calls: 0, fail_at: Some((1, Failure::Ordinary)) };
    {
        let mut control = ReviewControl::accept(&mut caller);
        assert_eq!(
            failure(prepare_applicability(
                fixture.applicability_inputs(),
                ReportFilters::default(),
                &mut ledger,
                &mut control
            )),
            ContractError::Capacity
        );
        assert_eq!(
            failure(prepare_impact(
                fixture.impact_inputs(),
                ImpactFilters::default(),
                &mut ledger,
                &mut control
            )),
            ContractError::Capacity
        );
        assert_eq!(ledger.checkpoint(&mut control), Err(ContractError::Capacity));
    }
    assert_eq!(caller.calls, 0);
}

/// Real caller failure classifications remain first across later genuine domain preparations.
#[test]
fn shared_work_prior_actual_control_stops_are_never_reprobed() {
    let fixture = Fixture::new();
    for (stop, expected) in [
        (Failure::Ordinary, ContractError::ControlFailed),
        (
            Failure::Interrupted(Interruption::CancelRequested),
            ContractError::Interrupted(Interruption::CancelRequested),
        ),
        (
            Failure::Interrupted(Interruption::Shutdown),
            ContractError::Interrupted(Interruption::Shutdown),
        ),
        (
            Failure::Interrupted(Interruption::DeadlineExceeded),
            ContractError::Interrupted(Interruption::DeadlineExceeded),
        ),
    ] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller { calls: 0, fail_at: Some((1, stop)) };
        {
            let mut control = ReviewControl::accept(&mut caller);
            assert_eq!(ledger.checkpoint(&mut control), Err(expected));
            assert_eq!(
                failure(prepare_applicability(
                    fixture.applicability_inputs(),
                    ReportFilters::default(),
                    &mut ledger,
                    &mut control
                )),
                expected
            );
            assert_eq!(
                failure(prepare_impact(
                    fixture.impact_inputs(),
                    ImpactFilters::default(),
                    &mut ledger,
                    &mut control
                )),
                expected
            );
            assert_eq!(ledger.checkpoint(&mut control), Err(expected));
        }
        assert_eq!(caller.calls, 1);
    }
}

/// A real forwarded ordinary failure remains sticky even when the caller would later accept.
#[test]
fn shared_work_native_control_failure_is_latched_before_native_conversion() {
    let fixture = Fixture::new();
    for impact_first in [false, true] {
        let mut ledger = ContractLedger::default();
        let mut caller = Caller { calls: 0, fail_at: Some((2, Failure::Ordinary)) };
        {
            let mut control = ReviewControl::accept(&mut caller);
            let error = if impact_first {
                failure(prepare_impact(
                    fixture.impact_inputs(),
                    ImpactFilters::default(),
                    &mut ledger,
                    &mut control,
                ))
            } else {
                failure(prepare_applicability(
                    fixture.applicability_inputs(),
                    ReportFilters::default(),
                    &mut ledger,
                    &mut control,
                ))
            };
            assert_eq!(error, ContractError::ControlFailed);
            assert_eq!(
                failure(prepare_impact(
                    fixture.impact_inputs(),
                    ImpactFilters::default(),
                    &mut ledger,
                    &mut control
                )),
                error
            );
            assert_eq!(
                failure(prepare_applicability(
                    fixture.applicability_inputs(),
                    ReportFilters::default(),
                    &mut ledger,
                    &mut control
                )),
                error
            );
            assert_eq!(ledger.checkpoint(&mut control), Err(error));
        }
        assert_eq!(caller.calls, 2);
    }
}

/// An actual final outer boundary can discard native success while preserving its exact stop.
#[test]
fn shared_work_actual_final_success_fence_refuses_complete_facts() {
    let fixture = Fixture::new();
    for impact in [false, true] {
        let mut baseline_caller = Caller::default();
        {
            let mut baseline_control = ReviewControl::accept(&mut baseline_caller);
            let mut baseline_ledger = ContractLedger::default();
            if impact {
                prepare_impact(
                    fixture.impact_inputs(),
                    ImpactFilters::default(),
                    &mut baseline_ledger,
                    &mut baseline_control,
                )
                .unwrap();
            } else {
                prepare_applicability(
                    fixture.applicability_inputs(),
                    ReportFilters::default(),
                    &mut baseline_ledger,
                    &mut baseline_control,
                )
                .unwrap();
            }
        }
        let terminal = baseline_caller.calls;
        assert!(terminal > 2);
        let mut caller = Caller {
            calls: 0,
            fail_at: Some((terminal, Failure::Interrupted(Interruption::CancelRequested))),
        };
        {
            let mut control = ReviewControl::accept(&mut caller);
            let mut ledger = ContractLedger::default();
            let error = if impact {
                failure(prepare_impact(
                    fixture.impact_inputs(),
                    ImpactFilters::default(),
                    &mut ledger,
                    &mut control,
                ))
            } else {
                failure(prepare_applicability(
                    fixture.applicability_inputs(),
                    ReportFilters::default(),
                    &mut ledger,
                    &mut control,
                ))
            };
            assert_eq!(error, ContractError::Interrupted(Interruption::CancelRequested));
            assert_eq!(ledger.checkpoint(&mut control), Err(error));
        }
        assert_eq!(caller.calls, terminal);
    }
}

/// A genuine native Domain refusal receives its outer fence and stays nonsticky without a stop.
#[test]
fn shared_work_actual_domain_fence_precedes_private_error_minimization() {
    let fixture = Fixture::new();
    let mut baseline_caller = Caller::default();
    {
        let mut baseline_control = ReviewControl::accept(&mut baseline_caller);
        let mut baseline_ledger = ContractLedger::default();
        let mut malformed = fixture.impact_inputs();
        malformed.manifest = b"{";
        assert_eq!(
            failure(prepare_impact(
                malformed,
                ImpactFilters::default(),
                &mut baseline_ledger,
                &mut baseline_control
            )),
            ContractError::Binding
        );
    }
    let terminal = baseline_caller.calls;
    let mut caller = Caller { calls: 0, fail_at: Some((terminal, Failure::Ordinary)) };
    {
        let mut control = ReviewControl::accept(&mut caller);
        let mut ledger = ContractLedger::default();
        let mut malformed = fixture.impact_inputs();
        malformed.manifest = b"{";
        assert_eq!(
            failure(prepare_impact(malformed, ImpactFilters::default(), &mut ledger, &mut control)),
            ContractError::ControlFailed
        );
        assert_eq!(ledger.checkpoint(&mut control), Err(ContractError::ControlFailed));
    }
    assert_eq!(caller.calls, terminal);
    let mut caller = Caller::default();
    let mut control = ReviewControl::accept(&mut caller);
    let mut ledger = ContractLedger::default();
    let mut malformed = fixture.impact_inputs();
    malformed.manifest = b"{";
    assert_eq!(
        failure(prepare_impact(malformed, ImpactFilters::default(), &mut ledger, &mut control)),
        ContractError::Binding
    );
    prepare_applicability(
        fixture.applicability_inputs(),
        ReportFilters::default(),
        &mut ledger,
        &mut control,
    )
    .unwrap();
    prepare_impact(fixture.impact_inputs(), ImpactFilters::default(), &mut ledger, &mut control)
        .unwrap();
}

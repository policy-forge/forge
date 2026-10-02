//! Artifact-only validation and exact case accounting with no acceptance upgrade.

use super::{
    capture::Session,
    manifest::{self, Adjudication, ArtifactRef, Case, Corpus, Outcome, error},
    report::{
        self, ArtifactPin, CaseReport, CaseState, Counts, EvidenceGate, EvidenceStatus, Report,
        Status,
    },
};
use crate::suggest::{
    bundle::SuggestionsBundle, request::SuggestRequest, run_record::RunRecord, validate,
};
use crate::{ForgeError, hashing::sha256_hex};
use std::{collections::BTreeMap, path::Path, sync::Arc};

/// The only file published by this module.
pub const PREFLIGHT_ARTIFACT: &str = "preflight.json";

/// Check a portable root-relative corpus manifest without publishing files.
///
/// # Errors
/// Returns an authoring error for malformed trusted metadata, unsafe or stale
/// captures, bad pins/bindings, or any fixed implementation bound violation.
/// Invalid untrusted response content remains an accounted case outcome.
pub fn preflight(root: &Path, manifest: &Path) -> Result<Report, ForgeError> {
    let (report, captures) = run(root, manifest)?;
    report.to_json_bytes()?;
    captures.verify()?;
    Ok(report)
}

/// Publish a bounded receipt in a new confined generation after final input revalidation.
///
/// `output_dir` is a portable directory path relative to the selected root.
/// The same session retains content and file-identity pins through serialization.
/// Each confined read closes its held file; final verification safely reopens
/// inputs immediately before publication. This is no guarantee of later input
/// stability. The existing no-replace primitive writes only preflight.json.
///
/// # Errors
/// Returns the same errors as preflight, or a publication error. No authoritative
/// artifact or existing generation is replaced.
pub fn preflight_to(root: &Path, manifest: &Path, output_dir: &Path) -> Result<Report, ForgeError> {
    let (report, captures) = run(root, manifest)?;
    let bytes = report.to_json_bytes()?;
    let artifacts = [crate::authoring::output::OutputArtifact {
        relative_path: PREFLIGHT_ARTIFACT.to_owned(),
        bytes,
    }];
    captures.verify()?;
    crate::authoring::output::publish(captures.root(), output_dir, &artifacts)?;
    Ok(report)
}

fn run(root: &Path, manifest_path: &Path) -> Result<(Report, Session), ForgeError> {
    let mut captures = Session::new(root)?;
    let bytes = captures.manifest(manifest_path, manifest::MAX_MANIFEST_BYTES)?;
    let corpus = Corpus::parse(&bytes)?;
    captures.validate_declared(&corpus)?;
    let mut cases: Vec<&Case> = corpus.cases.iter().collect();
    cases.sort_by(|left, right| left.case_key.cmp(&right.case_key));
    let mut reports = Vec::with_capacity(cases.len());
    for case in cases {
        captures.begin_case();
        reports.push(check_case(case, &mut captures)?);
    }
    let mut evidence_gates = Vec::new();
    for (gate, references) in corpus.evidence.groups() {
        let mut references: Vec<_> = references.iter().collect();
        references
            .sort_by(|left, right| (&left.sha256, &left.path).cmp(&(&right.sha256, &right.path)));
        let mut artifacts = Vec::with_capacity(references.len());
        // Opaque evidence is bounded separately from each candidate case.
        captures.begin_case();
        for reference in references {
            captures.read(reference, manifest::MAX_ATTESTATION_BYTES)?;
            artifacts.push(pin("opaque-attestation", reference));
        }
        evidence_gates.push(EvidenceGate {
            gate: gate.to_owned(),
            status: if artifacts.is_empty() {
                EvidenceStatus::Missing
            } else {
                EvidenceStatus::OpaqueUnsupported
            },
            artifacts,
        });
    }
    let counts = count(&reports);
    let report = Report {
        schema_version: report::SCHEMA_VERSION.to_owned(),
        scope: "artifact-preflight".to_owned(),
        status: if counts.response_invalid == 0 { Status::Incomplete } else { Status::Failed },
        preflight_complete: counts.artifacts_valid == counts.expected,
        acceptance_eligible: false,
        generation_enabled: false,
        task_selection_ready: false,
        corpus_key: corpus.corpus_key,
        corpus_version: corpus.version,
        manifest_sha256: sha256_hex(&bytes),
        counts,
        cases: reports,
        evidence_gates,
        pending_gates: [
            "authentic-lawful-inputs",
            "gold-target-universe-and-rubric",
            "semantic-output-adjudication",
            "split-and-contamination-review",
            "approved-quality-safety-thresholds",
            "approved-resource-profile",
            "candidate-model-template-identity",
            "qualified-local-execution",
            "evaluation-metrics-and-regression",
            "usefulness-pilot",
            "first-generation-task-selection",
            "final-roadmap-documentation-review",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    };
    Ok((report, captures))
}

#[allow(clippy::too_many_lines)] // Keeps one case's capture/binding/response classification together.
fn check_case(case: &Case, captures: &mut Session) -> Result<CaseReport, ForgeError> {
    let mut artifacts = Vec::new();
    let mut sources: BTreeMap<&str, (&ArtifactRef, Arc<[u8]>)> = BTreeMap::new();
    for source in &case.sources {
        let bytes = captures.read(&source.artifact, crate::reuse::corpus::MAX_DOCUMENT_BYTES)?;
        artifacts.push(pin(&format!("source-{}", source.key), &source.artifact));
        sources.insert(&source.key, (&source.artifact, bytes));
    }
    let request = case.request.as_ref().map(|reference| {
        let bytes = captures.read(reference, crate::suggest::request::MAX_REQUEST_BYTES)?;
        let request = SuggestRequest::parse(&bytes)
            .map_err(|_| error("evaluation request is invalid"))?;
        if request.task.kind != case.task.kind || request.task.schema_version != case.task.schema_version {
            return Err(error("evaluation request task does not match its case"));
        }
        let payload_ref = ArtifactRef { path: imported(&reference.path, &request.payload.artifact)?,
            sha256: request.payload.sha256.clone(), bytes: request.payload.bytes };
        let payload = captures.read(&payload_ref, crate::suggest::request::MAX_PAYLOAD_BYTES)?;
        let payload_text = std::str::from_utf8(&payload)
            .map_err(|_| error("evaluation payload is not UTF-8"))?;
        for unit in request.units() {
            let start = usize::try_from(unit.payload.start).map_err(|_| error("evaluation payload span is invalid"))?;
            let end = usize::try_from(unit.payload.end).map_err(|_| error("evaluation payload span is invalid"))?;
            if payload_text.get(start..end).is_none() {
                return Err(error("evaluation unit span leaves payload bytes or UTF-8 boundaries"));
            }
            if let Some(source) = &unit.source {
                let (reference, source_bytes) = sources.get(source.key.as_str())
                    .ok_or_else(|| error("evaluation request names a source outside its explicit inventory"))?;
                let resolved = format!("{}/{}", case.source_base, source.path);
                if resolved != reference.path || source.sha256 != reference.sha256 {
                    return Err(error("evaluation source base/path/SHA-256 binding does not match its captured source"));
                }
                let start = usize::try_from(source.start).map_err(|_| error("evaluation source span is invalid"))?;
                let end = usize::try_from(source.end).map_err(|_| error("evaluation source span is invalid"))?;
                if source_bytes.get(start..end).and_then(|span| std::str::from_utf8(span).ok()).is_none() {
                    return Err(error("evaluation source span leaves captured bytes or UTF-8 boundaries"));
                }
            }
        }
        artifacts.push(pin("request", reference));
        artifacts.push(pin("payload", &payload_ref));
        Ok::<_, ForgeError>((bytes, payload, request))
    }).transpose()?;
    let mut report = CaseReport {
        case_key: case.case_key.clone(),
        task: case.task.kind,
        origin: case.origin,
        state: CaseState::NotRun,
        artifacts_valid: false,
        mechanical_counts: None,
        declared_run_mode: None,
        artifacts,
    };
    match &case.outcome {
        Outcome::NotRun => report.state = CaseState::NotRun,
        Outcome::ExecutionBlocked => report.state = CaseState::ExecutionBlocked,
        Outcome::ToolError => report.state = CaseState::ToolError,
        Outcome::Response { run, response, bundle, adjudication } => {
            let (request_bytes, payload, request) =
                request.ok_or_else(|| error("evaluation response has no prepared request"))?;
            let run_bytes = captures.read(run, crate::suggest::run_record::MAX_RUN_RECORD_BYTES)?;
            let record = RunRecord::parse(&run_bytes)
                .map_err(|_| error("evaluation run record is invalid"))?;
            if imported(&run.path, &record.response_artifact)? != response.path {
                return Err(error(
                    "evaluation run response href resolves to a different captured artifact",
                ));
            }
            let response_bytes =
                captures.read(response, crate::suggest::response::MAX_RESPONSE_BYTES)?;
            record
                .authorises(
                    &sha256_hex(&request_bytes),
                    &request,
                    &sha256_hex(&payload),
                    &response_bytes,
                )
                .map_err(|_| {
                    error("evaluation run does not bind the exact request/payload/response")
                })?;
            report.artifacts.push(pin("run", run));
            report.artifacts.push(pin("response", response));
            report.declared_run_mode = Some(record.mode);
            let existing = bundle
                .as_ref()
                .map(|reference| {
                    let bytes =
                        captures.read(reference, crate::suggest::bundle::MAX_BUNDLE_BYTES)?;
                    let bundle = SuggestionsBundle::parse(&bytes)
                        .map_err(|_| error("evaluation quarantine bundle is invalid"))?;
                    if bundle.request.sha256 != sha256_hex(&request_bytes)
                        || bundle.request.payload_sha256 != request.payload.sha256
                        || bundle.response.sha256 != record.response_sha256
                        || bundle.response.bytes != record.response_bytes
                        || bundle.provenance.run_record_sha256 != sha256_hex(&run_bytes)
                        || bundle.project_key != request.project_key
                        || bundle.as_of != request.as_of
                        || bundle.task.kind != request.task.kind
                        || bundle.task.schema_version != request.task.schema_version
                        || bundle.provenance.mode != record.mode
                        || bundle.provenance.adapter_sha256 != record.adapter_executable_sha256
                        || bundle.provenance.model_id != record.model_id
                        || bundle.provenance.argv != record.argv
                        || bundle.provenance.redactions != record.redactions
                        || bundle.provenance.elapsed_ms != record.elapsed_ms
                        || bundle.provenance.exit_code != record.exit_code
                    {
                        return Err(error(
                            "evaluation quarantine bundle provenance does not bind captured inputs",
                        ));
                    }
                    report.artifacts.push(pin("bundle", reference));
                    if let Some(artifact) = &bundle.response.artifact {
                        let retained = ArtifactRef {
                            path: imported(&reference.path, artifact)?,
                            sha256: record.response_sha256.clone(),
                            bytes: record.response_bytes,
                        };
                        captures.read(&retained, crate::suggest::response::MAX_RESPONSE_BYTES)?;
                        report.artifacts.push(pin("retained-response", &retained));
                    }
                    Ok::<_, ForgeError>(bundle)
                })
                .transpose()?;
            let retain_raw = existing.as_ref().is_some_and(|bundle| bundle.response.retained);
            match validate::validate_captured(
                &request_bytes,
                &payload,
                &run_bytes,
                &response_bytes,
                retain_raw,
            ) {
                Ok(validated) => {
                    if existing.as_ref().is_some_and(|existing| existing != &validated) {
                        return Err(error(
                            "evaluation quarantine bundle differs from exact captured response validation",
                        ));
                    }
                    report.artifacts_valid = true;
                    report.mechanical_counts = Some(validated.counts);
                    report.state = match adjudication {
                        Adjudication::Missing => CaseState::AwaitingAdjudication,
                        Adjudication::Disputed => CaseState::AdjudicationDisputed,
                    };
                }
                Err(_) => report.state = CaseState::ResponseInvalid,
            }
        }
    }
    report
        .artifacts
        .sort_by(|left, right| (&left.role, &left.sha256).cmp(&(&right.role, &right.sha256)));
    Ok(report)
}

fn imported(importer: &str, href: &str) -> Result<String, ForgeError> {
    crate::suggest::shared::relative_path("evaluation imported artifact href", href)?;
    let path = importer
        .rsplit_once('/')
        .map_or_else(|| href.to_owned(), |(parent, _)| format!("{parent}/{href}"));
    crate::suggest::shared::relative_path("evaluation resolved artifact path", &path)?;
    Ok(path)
}
fn pin(role: &str, reference: &ArtifactRef) -> ArtifactPin {
    ArtifactPin { role: role.to_owned(), sha256: reference.sha256.clone(), bytes: reference.bytes }
}
fn count(cases: &[CaseReport]) -> Counts {
    let mut counts = Counts { expected: cases.len() as u64, ..Counts::default() };
    for case in cases {
        counts.artifacts_valid += u64::from(case.artifacts_valid);
        match case.state {
            CaseState::NotRun => counts.not_run += 1,
            CaseState::ExecutionBlocked => counts.execution_blocked += 1,
            CaseState::ToolError => counts.tool_error += 1,
            CaseState::ResponseInvalid => counts.response_invalid += 1,
            CaseState::AwaitingAdjudication => counts.awaiting_adjudication += 1,
            CaseState::AdjudicationDisputed => counts.adjudication_disputed += 1,
        }
    }
    counts
}

//! Crate-level `forge suggest validate`: closed decode, citation check, bundle.
//!
//! Validation is where model output stops being text and becomes a quarantined
//! record — or is refused whole. Every citation must name an allowlisted unit
//! from the request, and a quoted citation must match the exact payload bytes at
//! that unit's span. Nothing is repaired, and nothing is admitted on trust.

use std::collections::BTreeMap;
use std::path::Path;

use uuid::Uuid;

use crate::ForgeError;
use crate::cli::AuthorReportFormat;

use super::bundle::{
    EvidenceSupport, Provenance, RequestRef, ResponseRef, SCHEMA_VERSION as BUNDLE_SCHEMA_VERSION,
    Suggestion, SuggestionCounts, SuggestionsBundle,
};
use super::redact;
use super::request::{ContextKind, ContextUnit, SuggestRequest};
use super::response::{MAX_RESPONSE_BYTES, SuggestResponse};
use super::run_record::RunRecord;
use super::shared;
use super::task::{SuggestionBody, TaskKind};

/// Artifact name of the quarantine bundle.
pub const BUNDLE_ARTIFACT: &str = "suggestions.json";
/// Artifact name of the raw response when it is retained.
pub const RETAINED_RESPONSE_ARTIFACT: &str = "response.raw";

/// Arguments for one validate run.
pub struct ValidateArgs<'a> {
    /// The prepared request the response answers.
    pub request: &'a Path,
    /// The run record produced by `forge suggest run`.
    pub run: &'a Path,
    /// New directory beneath the request's directory.
    pub output_dir: &'a Path,
    /// Retain the exact raw response beside the bundle.
    pub retain_raw: bool,
    /// Stdout format.
    pub format: AuthorReportFormat,
}

/// Validate one recorded response into a quarantine bundle.
///
/// Returns `true` when the bundle is complete but empty, which the CLI maps to
/// the action-required exit code.
///
/// # Errors
/// Returns an authoring error for an invalid or unreadable input, a run record
/// that does not authorise the request/payload/response, a response that does
/// not decode against the closed task schema, a response for another task, a
/// citation that names no allowlisted unit, a quote that does not match the
/// payload bytes, or publication failure.
pub fn execute(args: &ValidateArgs<'_>) -> Result<bool, ForgeError> {
    let inputs = load_inputs(args)?;
    let bundle = validate_captured(
        &inputs.request,
        &inputs.payload,
        &inputs.record,
        &inputs.response,
        args.retain_raw,
    )?;
    publish(&inputs.root, &inputs.response, &bundle, args)?;

    let stdout = match args.format {
        AuthorReportFormat::Json => serde_json::to_string_pretty(&bundle)
            .map_err(|cause| shared::error(format!("bundle JSON encoding: {cause}")))?,
        AuthorReportFormat::Text => render_summary(&bundle, args),
    };
    crate::cli::output::write_output(&stdout, None)
        .map_err(|cause| shared::error(format!("cannot write the validate summary: {cause}")))?;
    Ok(bundle.suggestions.is_empty())
}

/// Validate exact captured suggestion artifacts without reading or publishing files.
///
/// The caller owns confinement, importer-relative path binding and source stability.
/// This checks the existing closed contracts, byte identities, selected targets,
/// citation membership and full-unit quotations, then reconstructs the quarantine
/// bundle. A run's mode and model identifier remain assertions from its record;
/// this does not authenticate execution, adjudication or semantic correctness.
///
/// # Errors
/// Returns an authoring error for bounds, malformed contracts, mismatched byte
/// identities, invalid targets/citations, secret-shaped output or an invalid bundle.
pub(crate) fn validate_captured(
    request_bytes: &[u8],
    payload_bytes: &[u8],
    record_bytes: &[u8],
    response_bytes: &[u8],
    retain_raw: bool,
) -> Result<SuggestionsBundle, ForgeError> {
    if payload_bytes.len() as u64 > super::request::MAX_PAYLOAD_BYTES {
        return Err(shared::error("the captured payload exceeds the suggestion payload limit"));
    }
    if response_bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(shared::error("the captured response exceeds the suggestion response limit"));
    }
    let request = SuggestRequest::parse(request_bytes)?;
    let record = RunRecord::parse(record_bytes)?;
    let payload_sha256 = crate::hashing::sha256_hex(payload_bytes);
    if payload_sha256 != request.payload.sha256
        || payload_bytes.len() as u64 != request.payload.bytes
    {
        return Err(shared::error("the payload does not match the request; re-run prepare"));
    }
    let payload = std::str::from_utf8(payload_bytes)
        .map_err(|_| shared::error("the payload artifact is not UTF-8"))?;
    let request_sha256 = crate::hashing::sha256_hex(request_bytes);
    let record_sha256 = crate::hashing::sha256_hex(record_bytes);
    record.authorises(&request_sha256, &request, &payload_sha256, response_bytes)?;
    // Every byte below is untrusted until this decode succeeds.
    let decoded = SuggestResponse::parse(response_bytes)?;
    if decoded.task.kind != request.task.kind {
        return Err(shared::error(
            "the response answers a different task than the request declared",
        ));
    }
    if decoded.task.schema_version != request.task.schema_version {
        return Err(shared::error("the response task schema version does not match the request"));
    }
    // Output-side refusal: the plan promises that model-emitted secret-shaped
    // text never reaches a bundle, retained or not.
    let decoded_json = serde_json::to_value(&decoded)
        .map_err(|cause| shared::error(format!("cannot inspect decoded response: {cause}")))?;
    redact::refuse_json_strings(&decoded_json)?;
    let suggestions = build_suggestions(&decoded, &request, payload)?;
    let counts = count(&suggestions);

    let bundle = SuggestionsBundle {
        schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
        bundle_id: identifier(&[
            "forge.suggestions/1 bundle",
            &request.project_key,
            &request_sha256,
            &record.response_sha256,
        ]),
        project_key: request.project_key.clone(),
        as_of: request.as_of.clone(),
        task: super::task::TaskIdentity {
            kind: request.task.kind,
            schema_version: request.task.schema_version.clone(),
        },
        request: RequestRef {
            sha256: request_sha256.clone(),
            payload_sha256: request.payload.sha256.clone(),
        },
        response: ResponseRef {
            sha256: record.response_sha256.clone(),
            bytes: record.response_bytes,
            retained: retain_raw,
            artifact: retain_raw.then(|| RETAINED_RESPONSE_ARTIFACT.to_string()),
        },
        provenance: Provenance {
            run_record_sha256: record_sha256.clone(),
            mode: record.mode,
            adapter_sha256: record.adapter_executable_sha256.clone(),
            model_id: record.model_id.clone(),
            argv: record.argv.clone(),
            elapsed_ms: record.elapsed_ms,
            exit_code: record.exit_code,
            redactions: record.redactions.clone(),
        },
        suggestions,
        counts,
    };
    let encoded = serde_json::to_vec(&bundle)
        .map_err(|cause| shared::error(format!("cannot encode the suggestions bundle: {cause}")))?;
    SuggestionsBundle::parse(&encoded)?;
    Ok(bundle)
}

/// Paths and bounded bytes read by the existing output-producing command.
struct Inputs {
    root: std::path::PathBuf,
    request: Vec<u8>,
    payload: Vec<u8>,
    response: Vec<u8>,
    record: Vec<u8>,
}

/// Read the bounded artifacts; pure validation binds and decodes the exact bytes.
fn load_inputs(args: &ValidateArgs<'_>) -> Result<Inputs, ForgeError> {
    let request_bytes = crate::io::read_bounded(args.request, super::request::MAX_REQUEST_BYTES)?;
    let request = SuggestRequest::parse(&request_bytes)?;
    let record_bytes = crate::io::read_bounded(args.run, super::run_record::MAX_RUN_RECORD_BYTES)?;
    let record = RunRecord::parse(&record_bytes)?;
    let root = shared::document_root(args.request, "--request")?;
    let payload = crate::io::read_bounded(
        &root.join(&request.payload.artifact),
        super::request::MAX_PAYLOAD_BYTES,
    )?;
    let run_dir = shared::document_root(args.run, "--run")?;
    let response =
        crate::io::read_bounded(&run_dir.join(&record.response_artifact), MAX_RESPONSE_BYTES)?;
    Ok(Inputs { root, request: request_bytes, payload, response, record: record_bytes })
}

/// Check every citation of every suggestion against the allowlist and the payload.
fn build_suggestions(
    decoded: &SuggestResponse,
    request: &SuggestRequest,
    payload: &str,
) -> Result<Vec<Suggestion>, ForgeError> {
    let units: BTreeMap<&str, &ContextUnit> =
        request.context.units.iter().map(|unit| (unit.unit_id.as_str(), unit)).collect();
    let mut suggestions = Vec::new();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for candidate in &decoded.task.mapping_candidates {
        require_subject(request, candidate)?;
        let body = SuggestionBody { mapping: Some(candidate.clone()), drafting: None };
        suggestions.push(checked_suggestion(
            &body,
            &candidate.citations,
            &units,
            payload,
            request,
            TaskKind::MappingCandidates,
        )?);
    }
    for clause in &decoded.task.draft_clauses {
        require_section(request, clause)?;
        let body = SuggestionBody { mapping: None, drafting: Some(clause.clone()) };
        suggestions.push(checked_suggestion(
            &body,
            &clause.citations,
            &units,
            payload,
            request,
            TaskKind::PolicyDrafting,
        )?);
    }
    for (index, suggestion) in suggestions.iter().enumerate() {
        if !seen.insert(suggestion.content_sha256.clone()) {
            return Err(shared::error(format!(
                "suggestion {index} repeats content already quarantined from this response; \
                 the adapter must return distinct suggestions"
            )));
        }
    }
    Ok(suggestions)
}

/// A clause must address a section the request actually selected.
///
/// Kind and schema agreement do not establish that: a well-formed clause for an
/// invented policy or topic used to be admitted with a high evidence rating and
/// then could not be promoted from the supplied request.
///
/// # Errors
/// Returns an authoring error naming the unsupplied target.
fn require_section(
    request: &SuggestRequest,
    clause: &super::task::drafting::DraftClause,
) -> Result<(), ForgeError> {
    if request.task.drafting_sections.iter().any(|section| {
        section.policy_key == clause.policy_key && section.topic_key == clause.topic_key
    }) {
        return Ok(());
    }
    Err(shared::error(format!(
        "the response proposes a clause for '{}/{}', which the request did not supply",
        clause.policy_key, clause.topic_key
    )))
}

/// A mapping candidate must address a supplied subject and one of its controls.
///
/// An empty control list admits no control; it is not a wildcard.
///
/// # Errors
/// Returns an authoring error naming the unsupplied subject or control.
fn require_subject(
    request: &SuggestRequest,
    candidate: &super::task::mapping::MappingCandidate,
) -> Result<(), ForgeError> {
    let subject = request
        .task
        .mapping_subjects
        .iter()
        .find(|subject| {
            subject.policy_key == candidate.policy_key && subject.topic_key == candidate.topic_key
        })
        .ok_or_else(|| {
            shared::error(format!(
                "the response proposes a relationship for '{}/{}', which the request did not supply",
                candidate.policy_key, candidate.topic_key
            ))
        })?;
    if !subject.control_ids.iter().any(|control| control == &candidate.control_id) {
        return Err(shared::error(format!(
            "the response proposes control '{}', which the supplied subject did not include",
            candidate.control_id
        )));
    }
    Ok(())
}

/// Publish the validated bundle, and the raw response only when opted in.
fn publish(
    root: &Path,
    response: &[u8],
    bundle: &SuggestionsBundle,
    args: &ValidateArgs<'_>,
) -> Result<(), ForgeError> {
    let bundle_json = serde_json::to_vec_pretty(bundle)
        .map_err(|cause| shared::error(format!("cannot encode the suggestions bundle: {cause}")))?;
    // A bundle FORGE publishes must pass FORGE's own contract first.
    SuggestionsBundle::parse(&bundle_json)?;
    let mut artifacts = vec![crate::authoring::output::OutputArtifact {
        relative_path: BUNDLE_ARTIFACT.to_string(),
        bytes: bundle_json,
    }];
    if args.retain_raw {
        artifacts.push(crate::authoring::output::OutputArtifact {
            relative_path: RETAINED_RESPONSE_ARTIFACT.to_string(),
            bytes: response.to_vec(),
        });
    }
    crate::authoring::output::publish(root, args.output_dir, &artifacts)
}

/// Resolve every citation and build one checked suggestion.
fn checked_suggestion(
    body: &SuggestionBody,
    citations: &[super::task::Citation],
    units: &BTreeMap<&str, &ContextUnit>,
    payload: &str,
    request: &SuggestRequest,
    kind: TaskKind,
) -> Result<Suggestion, ForgeError> {
    let mut saw_source_span = false;
    let mut all_source_spans = true;
    for citation in citations {
        let unit = units.get(citation.unit_id.as_str()).ok_or_else(|| {
            shared::error(format!(
                "citation names unit '{}' which is not in the request allowlist",
                citation.unit_id
            ))
        })?;
        if unit.kind == ContextKind::SourceSpan {
            saw_source_span = true;
        } else {
            all_source_spans = false;
        }
        if let Some(quote) = &citation.quote {
            let start = usize::try_from(unit.payload.start)
                .map_err(|_| shared::error("unit payload offset does not fit this platform"))?;
            let end = usize::try_from(unit.payload.end)
                .map_err(|_| shared::error("unit payload offset does not fit this platform"))?;
            let exact = payload
                .get(start..end)
                .ok_or_else(|| shared::error("unit payload span leaves the payload"))?;
            if exact != quote {
                return Err(shared::error(format!(
                    "citation to unit '{}' quotes bytes that are not the supplied text",
                    citation.unit_id
                )));
            }
        }
    }
    let support = if all_source_spans {
        EvidenceSupport::High
    } else if saw_source_span {
        EvidenceSupport::Medium
    } else {
        EvidenceSupport::Low
    };
    let canonical = serde_json::to_vec(body)
        .map_err(|cause| shared::error(format!("cannot encode a suggestion: {cause}")))?;
    Ok(Suggestion {
        content_sha256: super::bundle::content_sha256(body)?,
        suggestion_id: identifier(&[
            "forge.suggestions/1 suggestion",
            &request.project_key,
            kind.as_str(),
            std::str::from_utf8(&canonical)
                .map_err(|_| shared::error("suggestion encoding is not UTF-8"))?,
        ]),
        evidence_support: support,
        body: body.clone(),
    })
}

/// Deterministic identifier: a UUID v5 of domain-separated content, so the same
/// suggestion always receives the same identifier.
fn identifier(parts: &[&str]) -> String {
    let mut name = String::new();
    for part in parts {
        name.push_str(part);
        name.push('\n');
    }
    Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, name.as_bytes()).to_string()
}

/// Recompute the bundle counts from the suggestions present.
fn count(suggestions: &[Suggestion]) -> SuggestionCounts {
    let mut counts = SuggestionCounts {
        suggestions: suggestions.len() as u64,
        mapping: 0,
        drafting: 0,
        high: 0,
        medium: 0,
        low: 0,
    };
    for suggestion in suggestions {
        if suggestion.body.mapping.is_some() {
            counts.mapping += 1;
        }
        if suggestion.body.drafting.is_some() {
            counts.drafting += 1;
        }
        match suggestion.evidence_support {
            EvidenceSupport::High => counts.high += 1,
            EvidenceSupport::Medium => counts.medium += 1,
            EvidenceSupport::Low => counts.low += 1,
        }
    }
    counts
}

/// The text summary for one validated response.
fn render_summary(bundle: &SuggestionsBundle, args: &ValidateArgs<'_>) -> String {
    use std::fmt::Write as _;
    let mut summary = String::new();
    let _ = writeln!(summary, "validated {} suggestion(s)", bundle.counts.suggestions);
    let _ =
        writeln!(summary, "task: {} ({})", bundle.task.kind.as_str(), bundle.task.schema_version);
    let _ = writeln!(
        summary,
        "evidence: high {} medium {} low {}",
        bundle.counts.high, bundle.counts.medium, bundle.counts.low
    );
    let _ = writeln!(
        summary,
        "run mode: {} (record {})",
        bundle.provenance.mode.as_str(),
        &bundle.provenance.run_record_sha256[..12.min(bundle.provenance.run_record_sha256.len())]
    );
    let _ = writeln!(summary, "raw response retained: {}", bundle.response.retained);
    let _ = writeln!(summary, "output: {}", args.output_dir.display());
    let _ = writeln!(summary, "next: forge suggest review --bundle ...");
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggest::request::{AdapterTarget, ContextBundle, PayloadPreview};
    use serde_json::json;

    fn unit(kind: ContextKind, start: u64, end: u64) -> ContextUnit {
        ContextUnit {
            unit_id: format!("unit-{start:04}"),
            kind,
            label: "synthetic unit".to_string(),
            sensitivity: super::super::request::Sensitivity::Internal,
            source: None,
            payload: super::super::request::PayloadSpan { start, end },
        }
    }

    fn payload_request(units: Vec<ContextUnit>, payload: &str) -> SuggestRequest {
        SuggestRequest {
            schema_version: super::super::request::SCHEMA_VERSION.to_string(),
            project_key: "synthetic-project".to_string(),
            as_of: "2026-09-12T00:00:00Z".to_string(),
            task: super::super::request::RequestTask {
                kind: TaskKind::PolicyDrafting,
                schema_version: super::super::task::drafting::TASK_SCHEMA_VERSION.to_string(),
                mapping_subjects: Vec::new(),
                drafting_sections: vec![super::super::task::drafting::DraftingSection {
                    policy_key: "access-policy".to_string(),
                    topic_key: "access-topic".to_string(),
                    order: 1,
                    title: "Access".to_string(),
                    prompt: "Describe access.".to_string(),
                    gap_ids: Vec::new(),
                    control_ids: Vec::new(),
                }],
            },
            adapter: AdapterTarget {
                executable: "local-model".to_string(),
                executable_sha256: "a".repeat(64),
                model_id: "synthetic".to_string(),
                argv: Vec::new(),
            },
            context: ContextBundle { units },
            redactions: Vec::new(),
            payload: PayloadPreview {
                artifact: "payload.txt".to_string(),
                sha256: crate::hashing::sha256_hex(payload.as_bytes()),
                bytes: payload.len() as u64,
                units: 1,
            },
            retention_notice: "Local only.".to_string(),
        }
    }

    fn clause(unit_id: &str, quote: Option<&str>) -> super::super::task::drafting::DraftClause {
        super::super::task::drafting::DraftClause {
            policy_key: "access-policy".to_string(),
            topic_key: "access-topic".to_string(),
            question_key: None,
            heading: None,
            draft_text: "Quarterly review.".to_string(),
            citations: vec![super::super::task::Citation {
                unit_id: unit_id.to_string(),
                quote: quote.map(str::to_string),
            }],
            assumptions: Vec::new(),
            unresolved_questions: vec!["Who approves?".to_string()],
        }
    }

    fn body(clause: &super::super::task::drafting::DraftClause) -> SuggestionBody {
        SuggestionBody { mapping: None, drafting: Some(clause.clone()) }
    }

    #[test]
    fn mapping_candidates_require_both_a_supplied_subject_and_control() {
        let mut request = crate::suggest::request::fixture_request();
        let subject =
            serde_json::from_value(crate::suggest::task::mapping::fixture_subject_json()).unwrap();
        request.task.mapping_subjects = vec![subject];
        let mut candidate =
            serde_json::from_value(crate::suggest::task::mapping::fixture_candidate_json())
                .unwrap();
        assert!(require_subject(&request, &candidate).is_ok());
        candidate.control_id = "invented".to_string();
        assert!(require_subject(&request, &candidate).is_err());
        request.task.mapping_subjects[0].control_ids.clear();
        assert!(require_subject(&request, &candidate).is_err());
        candidate.policy_key = "invented".to_string();
        assert!(require_subject(&request, &candidate).is_err());
    }

    #[test]
    fn identifiers_are_deterministic_and_content_addressed() {
        let first = identifier(&["forge.suggestions/1 suggestion", "project", "a"]);
        let second = identifier(&["forge.suggestions/1 suggestion", "project", "a"]);
        let other = identifier(&["forge.suggestions/1 suggestion", "project", "b"]);
        assert_eq!(first, second);
        assert_ne!(first, other);
        assert_eq!(first.len(), 36);
    }

    #[test]
    fn an_unknown_unit_is_refused_and_quotes_must_match_the_payload() {
        let payload = "supplied text";
        let units = vec![unit(ContextKind::SourceSpan, 0, 13)];
        let request = payload_request(units.clone(), payload);
        let index: BTreeMap<&str, &ContextUnit> =
            units.iter().map(|u| (u.unit_id.as_str(), u)).collect();

        assert!(
            checked_suggestion(
                &body(&clause("unit-0000", None)),
                &clause("unit-0000", None).citations,
                &index,
                payload,
                &request,
                TaskKind::PolicyDrafting
            )
            .is_ok()
        );
        let unknown = clause("unit-9999", None);
        assert!(
            checked_suggestion(
                &body(&unknown),
                &unknown.citations,
                &index,
                payload,
                &request,
                TaskKind::PolicyDrafting
            )
            .is_err()
        );
        let altered = clause("unit-0000", Some("supplied tex!"));
        assert!(
            checked_suggestion(
                &body(&altered),
                &altered.citations,
                &index,
                payload,
                &request,
                TaskKind::PolicyDrafting
            )
            .is_err()
        );
        let exact = clause("unit-0000", Some("supplied text"));
        let checked = checked_suggestion(
            &body(&exact),
            &exact.citations,
            &index,
            payload,
            &request,
            TaskKind::PolicyDrafting,
        )
        .unwrap();
        assert_eq!(checked.evidence_support, EvidenceSupport::High);
    }

    #[test]
    fn evidence_support_follows_the_cited_unit_kind() {
        let payload = "supplied text and metadata";
        let units = vec![
            unit(ContextKind::SourceSpan, 0, 13),
            ContextUnit { unit_id: "unit-meta".to_string(), ..unit(ContextKind::Answer, 18, 26) },
        ];
        let request = payload_request(units.clone(), payload);
        let index: BTreeMap<&str, &ContextUnit> =
            units.iter().map(|u| (u.unit_id.as_str(), u)).collect();
        let rated = |clause: &super::super::task::drafting::DraftClause| {
            checked_suggestion(
                &body(clause),
                &clause.citations,
                &index,
                payload,
                &request,
                TaskKind::PolicyDrafting,
            )
            .unwrap()
            .evidence_support
        };
        assert_eq!(rated(&clause("unit-0000", None)), EvidenceSupport::High);

        let mut mixed = clause("unit-0000", None);
        mixed
            .citations
            .push(super::super::task::Citation { unit_id: "unit-meta".to_string(), quote: None });
        assert_eq!(rated(&mixed), EvidenceSupport::Medium);

        let metadata_only = clause("unit-meta", None);
        assert_eq!(rated(&metadata_only), EvidenceSupport::Low);
    }

    #[test]
    fn counts_recompute_from_the_suggestions_present() {
        let payload = "supplied text";
        let units = vec![unit(ContextKind::SourceSpan, 0, 13)];
        let request = payload_request(units.clone(), payload);
        let index: BTreeMap<&str, &ContextUnit> =
            units.iter().map(|u| (u.unit_id.as_str(), u)).collect();
        let suggestion = checked_suggestion(
            &body(&clause("unit-0000", None)),
            &clause("unit-0000", None).citations,
            &index,
            payload,
            &request,
            TaskKind::PolicyDrafting,
        )
        .unwrap();
        let counts = count(std::slice::from_ref(&suggestion));
        assert_eq!(counts.suggestions, 1);
        assert_eq!(counts.drafting, 1);
        assert_eq!(counts.mapping, 0);
        assert_eq!(counts.high, 1);
        assert_eq!(counts, count(std::slice::from_ref(&suggestion)));

        let empty = count(&[]);
        assert_eq!(empty.suggestions, 0);
        assert_eq!(
            serde_json::to_value(empty).unwrap(),
            json!({
                "suggestions": 0, "mapping": 0, "drafting": 0, "high": 0, "medium": 0, "low": 0
            })
        );
    }
    fn captured_fixture(response: &serde_json::Value) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
        let payload = b"Accounts must be reviewed every quarter.".to_vec();
        let mut request = super::super::request::fixture_request_json();
        request["payload"]["bytes"] = json!(payload.len());
        request["payload"]["sha256"] = json!(crate::hashing::sha256_hex(&payload));
        request["context"]["units"][0]["payload"]["end"] = json!(payload.len());
        let request_bytes = serde_json::to_vec(&request).unwrap();
        let response_bytes = serde_json::to_vec(response).unwrap();
        let mut record = super::super::run_record::fixture_run_json();
        record["request_sha256"] = json!(crate::hashing::sha256_hex(&request_bytes));
        record["payload_sha256"] = request["payload"]["sha256"].clone();
        record["response_sha256"] = json!(crate::hashing::sha256_hex(&response_bytes));
        record["response_bytes"] = json!(response_bytes.len());
        record["mode"] = json!("recorded-response");
        (request_bytes, payload, serde_json::to_vec(&record).unwrap(), response_bytes)
    }

    #[test]
    fn captured_validation_reconstructs_identical_bundles_without_paths() {
        let (request, payload, run, response) =
            captured_fixture(&super::super::response::fixture_response_json());
        let first = validate_captured(&request, &payload, &run, &response, false).unwrap();
        let second = validate_captured(&request, &payload, &run, &response, false).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.counts.suggestions, 1);
        assert_eq!(first.suggestions[0].evidence_support, EvidenceSupport::High);
        assert!(!first.response.retained);
        assert_eq!(first.response.artifact, None);
        assert_eq!(first.provenance.mode, super::super::run_record::RunMode::RecordedResponse);
        let retained = validate_captured(&request, &payload, &run, &response, true).unwrap();
        assert_eq!(retained.suggestions, first.suggestions);
        assert_eq!(retained.bundle_id, first.bundle_id);
        assert_eq!(retained.response.artifact.as_deref(), Some(RETAINED_RESPONSE_ARTIFACT));
    }

    #[test]
    fn captured_validation_rejects_changed_payload_and_run_bindings() {
        let (request, payload, run, response) =
            captured_fixture(&super::super::response::fixture_response_json());
        let mut altered = payload.clone();
        altered[0] = b'X';
        assert!(validate_captured(&request, &altered, &run, &response, false).is_err());
        let mut record: serde_json::Value = serde_json::from_slice(&run).unwrap();
        for field in ["request_sha256", "payload_sha256", "response_sha256"] {
            record[field] = json!("b".repeat(64));
            let encoded = serde_json::to_vec(&record).unwrap();
            assert!(validate_captured(&request, &payload, &encoded, &response, false).is_err());
            record = serde_json::from_slice(&run).unwrap();
        }
    }

    #[test]
    fn freshly_bound_invalid_citations_and_targets_still_fail() {
        for field in ["quote", "unit_id", "policy_key", "topic_key"] {
            let mut response = super::super::response::fixture_response_json();
            if matches!(field, "quote" | "unit_id") {
                response["task"]["draft_clauses"][0]["citations"][0][field] = json!("invented");
            } else {
                response["task"]["draft_clauses"][0][field] = json!("invented");
            }
            let (request, payload, run, response) = captured_fixture(&response);
            assert!(
                validate_captured(&request, &payload, &run, &response, false).is_err(),
                "{field}"
            );
        }
    }

    #[test]
    fn captured_validation_enforces_byte_bounds_before_decoding() {
        let (request, payload, run, response) =
            captured_fixture(&super::super::response::fixture_response_json());
        let oversized = vec![b'x'; usize::try_from(MAX_RESPONSE_BYTES).unwrap() + 1];
        assert!(validate_captured(&request, &payload, &run, &oversized, false).is_err());
        assert!(validate_captured(&request, &oversized, &run, &response, false).is_err());
    }
}

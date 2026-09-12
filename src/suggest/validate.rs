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
    // Every byte below is untrusted until this decode succeeds.
    let decoded = SuggestResponse::parse(&inputs.response)?;
    if decoded.task.kind != inputs.request.task.kind {
        return Err(shared::error(
            "the response answers a different task than the request declared",
        ));
    }
    if decoded.task.schema_version != inputs.request.task.schema_version {
        return Err(shared::error("the response task schema version does not match the request"));
    }
    let suggestions = build_suggestions(&decoded, &inputs)?;
    let counts = count(&suggestions);

    let bundle = SuggestionsBundle {
        schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
        bundle_id: identifier(&[
            "forge.suggestions/1 bundle",
            &inputs.request.project_key,
            &inputs.request_sha256,
            &inputs.record.response_sha256,
        ]),
        project_key: inputs.request.project_key.clone(),
        as_of: inputs.request.as_of.clone(),
        task: super::task::TaskIdentity {
            kind: inputs.request.task.kind,
            schema_version: inputs.request.task.schema_version.clone(),
        },
        request: RequestRef {
            sha256: inputs.request_sha256.clone(),
            payload_sha256: inputs.request.payload.sha256.clone(),
        },
        response: ResponseRef {
            sha256: inputs.record.response_sha256.clone(),
            bytes: inputs.record.response_bytes,
            retained: args.retain_raw,
            artifact: args.retain_raw.then(|| RETAINED_RESPONSE_ARTIFACT.to_string()),
        },
        provenance: Provenance {
            adapter_sha256: inputs.record.adapter_executable_sha256.clone(),
            model_id: inputs.record.model_id.clone(),
            argv: inputs.record.argv.clone(),
            elapsed_ms: inputs.record.elapsed_ms,
            exit_code: inputs.record.exit_code,
            redactions: inputs.record.redactions.clone(),
        },
        suggestions,
        counts,
    };
    publish(&inputs, &bundle, args)?;

    let stdout = match args.format {
        AuthorReportFormat::Json => serde_json::to_string_pretty(&bundle)
            .map_err(|cause| shared::error(format!("bundle JSON encoding: {cause}")))?,
        AuthorReportFormat::Text => render_summary(&bundle, args),
    };
    crate::cli::output::write_output(&stdout, None)
        .map_err(|cause| shared::error(format!("cannot write the validate summary: {cause}")))?;
    Ok(bundle.suggestions.is_empty())
}

/// Everything one validation reads, already checked against its own digest.
struct Inputs {
    root: std::path::PathBuf,
    request: SuggestRequest,
    request_sha256: String,
    payload: String,
    response: Vec<u8>,
    record: RunRecord,
}

/// Read the request, run record, payload and response, and bind them together.
fn load_inputs(args: &ValidateArgs<'_>) -> Result<Inputs, ForgeError> {
    let request_bytes = crate::io::read_bounded(args.request, super::request::MAX_REQUEST_BYTES)?;
    let request = SuggestRequest::parse(&request_bytes)?;
    let record = RunRecord::parse(&crate::io::read_bounded(
        args.run,
        super::run_record::MAX_RUN_RECORD_BYTES,
    )?)?;

    let root = args
        .request
        .parent()
        .map(std::fs::canonicalize)
        .transpose()
        .map_err(|cause| shared::error(format!("cannot resolve the request directory: {cause}")))?
        .ok_or_else(|| shared::error("--request must name a file in a directory"))?;
    let payload = crate::io::read_bounded(
        &root.join(&request.payload.artifact),
        super::request::MAX_PAYLOAD_BYTES,
    )?;
    let payload_sha256 = crate::hashing::sha256_hex(&payload);
    if payload_sha256 != request.payload.sha256 || payload.len() as u64 != request.payload.bytes {
        return Err(shared::error("the payload does not match the request; re-run prepare"));
    }
    let payload = String::from_utf8(payload)
        .map_err(|_| shared::error("the payload artifact is not UTF-8"))?;

    let run_dir = args
        .run
        .parent()
        .map(std::fs::canonicalize)
        .transpose()
        .map_err(|cause| shared::error(format!("cannot resolve the run directory: {cause}")))?
        .ok_or_else(|| shared::error("--run must name a file in a directory"))?;
    let response =
        crate::io::read_bounded(&run_dir.join(&record.response_artifact), MAX_RESPONSE_BYTES)?;
    let request_sha256 = crate::hashing::sha256_hex(&request_bytes);
    record.authorises(&request_sha256, &request, &payload_sha256, &response)?;
    Ok(Inputs { root, request, request_sha256, payload, response, record })
}

/// Check every citation of every suggestion against the allowlist and the payload.
fn build_suggestions(
    decoded: &SuggestResponse,
    inputs: &Inputs,
) -> Result<Vec<Suggestion>, ForgeError> {
    let units: BTreeMap<&str, &ContextUnit> =
        inputs.request.context.units.iter().map(|unit| (unit.unit_id.as_str(), unit)).collect();
    let mut suggestions = Vec::new();
    for candidate in &decoded.task.mapping_candidates {
        let body = SuggestionBody { mapping: Some(candidate.clone()), drafting: None };
        suggestions.push(checked_suggestion(
            &body,
            &candidate.citations,
            &units,
            &inputs.payload,
            &inputs.request,
            TaskKind::MappingCandidates,
        )?);
    }
    for clause in &decoded.task.draft_clauses {
        let body = SuggestionBody { mapping: None, drafting: Some(clause.clone()) };
        suggestions.push(checked_suggestion(
            &body,
            &clause.citations,
            &units,
            &inputs.payload,
            &inputs.request,
            TaskKind::PolicyDrafting,
        )?);
    }
    Ok(suggestions)
}

/// Publish the validated bundle, and the raw response only when opted in.
fn publish(
    inputs: &Inputs,
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
            bytes: inputs.response.clone(),
        });
    }
    crate::authoring::output::publish(&inputs.root, args.output_dir, &artifacts)
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
}

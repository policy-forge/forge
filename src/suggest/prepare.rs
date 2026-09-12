//! Crate-level `forge suggest prepare`.
//!
//! Prepare is the allowlist and the promise: it assembles the exact bytes a
//! local adapter would receive, writes them beside a closed request that names
//! only the material the operator selected, renders the exact preview, and —
//! only when the operator asks — writes a consent token bound to the payload
//! digest and the adapter identity. Nothing is sent, and nothing is generated:
//! every payload byte is operator-supplied text or fixed framing.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use crate::ForgeError;
use crate::cli::AuthorReportFormat;

use super::consent::ConsentToken;
use super::redact::{self, RedactionRule};
use super::request::{
    AdapterTarget, ContextBundle, ContextKind, ContextUnit, DATA_HANDLING_NOTICE,
    MAX_PAYLOAD_BYTES, PayloadPreview, PayloadSpan, RedactionRecord, RequestTask, SCHEMA_VERSION,
    Sensitivity, SourceRef, SuggestRequest,
};
use super::shared;
use super::task::{TaskKind, drafting};
use super::{consent as consent_contract, redact as redact_module};

/// Artifact name of the closed request.
pub const REQUEST_ARTIFACT: &str = "request.json";
/// Artifact name of the exact payload.
pub const PAYLOAD_ARTIFACT: &str = "payload.txt";
/// Artifact name of the human preview.
pub const PREVIEW_ARTIFACT: &str = "preview.txt";
/// Artifact name of the consent token, written only with `--consent`.
pub const CONSENT_ARTIFACT: &str = "consent.json";

/// Maximum size of a local adapter executable FORGE will fingerprint.
pub const MAX_ADAPTER_BYTES: u64 = 64 * 1024 * 1024;
/// Default retention notice recorded when the operator does not supply one.
pub const DEFAULT_RETENTION_NOTICE: &str =
    "No copy of this payload or its response leaves this machine.";

const TASK_HEADER: &str = "<<<forge.suggest/1 task>>>\n";
const CONTEXT_HEADER: &str = "<<<context>>>\n";

/// Arguments for one prepare run.
pub struct PrepareArgs<'a> {
    /// Authoring project manifest.
    pub manifest: &'a Path,
    /// Destination generation directory.
    pub output_dir: &'a Path,
    /// Selected task.
    pub task: TaskKind,
    /// Optional `forge.reuse-corpus/1` manifest naming candidate documents.
    pub corpus: Option<&'a Path>,
    /// Corpus document keys to include, in the order they appear in the corpus.
    pub include_documents: &'a [String],
    /// Approved answer keys to include, in the order given.
    pub answers: &'a [String],
    /// Acknowledge confidential or restricted material explicitly.
    pub allow_sensitive: bool,
    /// Optional redaction rules file.
    pub redact_rules: Option<&'a Path>,
    /// Local adapter executable.
    pub adapter: &'a Path,
    /// Operator-supplied model identifier.
    pub model_id: &'a str,
    /// Retention notice to record verbatim.
    pub retention_notice: &'a str,
    /// Write a consent token for this exact payload.
    pub consent: bool,
    /// Operator key granting consent.
    pub operator_key: Option<&'a str>,
    /// Stdout format.
    pub format: AuthorReportFormat,
}

/// One unit before it is placed in the payload.
struct UnitDraft {
    unit_id: String,
    kind: ContextKind,
    label: String,
    sensitivity: Sensitivity,
    source: Option<SourceRef>,
    text: String,
    section: Option<SectionSeed>,
}

/// The authoring-plan coordinates a plan-section unit stands for.
struct SectionSeed {
    policy_key: String,
    topic_key: String,
    order: u32,
    gap_ids: Vec<String>,
    control_ids: Vec<String>,
}

/// Determine the run's artifacts without sending anything.
///
/// # Errors
/// Returns an authoring error for invalid arguments, unsafe or over-bound
/// material, a rule that matched nothing, unredacted secret-shaped payload
/// content, or publication failure.
pub fn execute(args: &PrepareArgs<'_>) -> Result<bool, ForgeError> {
    if args.task != TaskKind::PolicyDrafting {
        return Err(shared::error(
            "the mapping-candidate task has no selected context derivation yet; the owner deferred task \
             selection on 2026-09-12, so only --task drafting is available",
        ));
    }
    if args.consent && args.operator_key.is_none() {
        return Err(shared::error("--consent requires --operator-key"));
    }
    let seed = crate::authoring::suggest_seed(args.manifest)?;
    let rules = read_rules(args.redact_rules)?;
    let mut drafts = Vec::new();

    push_plan_sections(&mut drafts, &seed);
    push_answers(&mut drafts, &seed, args)?;
    let captured = push_source_spans(&mut drafts, args)?;

    if drafts.is_empty() {
        crate::cli::output::write_output(
            "forge suggest prepare: no unresolved section, approved answer or selected document was \
             supplied, so no request was written.\n",
            None,
        )
        .map_err(|cause| shared::error(format!("cannot write prepare summary: {cause}")))?;
        return Ok(true);
    }

    let (task, payload, units, redactions) = assemble(&drafts, &rules)?;
    let payload_sha256 = crate::hashing::sha256_hex(payload.as_bytes());
    let adapter = adapter_target(args.adapter, args.model_id)?;
    let request = SuggestRequest {
        schema_version: SCHEMA_VERSION.to_string(),
        project_key: seed.plan.project_key.clone(),
        as_of: seed.plan.as_of.clone(),
        task,
        adapter,
        context: ContextBundle { units },
        redactions,
        payload: PayloadPreview {
            artifact: PAYLOAD_ARTIFACT.to_string(),
            sha256: payload_sha256.clone(),
            bytes: u64::try_from(payload.len())
                .map_err(|_| shared::error("payload length does not fit the contract"))?,
            units: u64::try_from(drafts.len())
                .map_err(|_| shared::error("unit count does not fit the contract"))?,
        },
        retention_notice: args.retention_notice.to_string(),
    };

    // A request FORGE publishes must pass FORGE's own contract first.
    let request_json = serde_json::to_vec_pretty(&request)
        .map_err(|cause| shared::error(format!("cannot encode the suggest request: {cause}")))?;
    SuggestRequest::parse(&request_json)?;

    let preview = render_preview(&request, &payload);
    let mut artifacts = vec![
        crate::authoring::output::OutputArtifact {
            relative_path: REQUEST_ARTIFACT.to_string(),
            bytes: request_json.clone(),
        },
        crate::authoring::output::OutputArtifact {
            relative_path: PAYLOAD_ARTIFACT.to_string(),
            bytes: payload.clone().into_bytes(),
        },
        crate::authoring::output::OutputArtifact {
            relative_path: PREVIEW_ARTIFACT.to_string(),
            bytes: preview.clone().into_bytes(),
        },
    ];
    if args.consent {
        let operator_key =
            args.operator_key.ok_or_else(|| shared::error("--consent requires --operator-key"))?;
        let token = ConsentToken {
            schema_version: consent_contract::SCHEMA_VERSION.to_string(),
            payload_sha256,
            adapter_sha256: request.adapter.executable_sha256.clone(),
            model_id: request.adapter.model_id.clone(),
            retention_notice: request.retention_notice.clone(),
            operator_key: operator_key.to_string(),
            as_of: seed.plan.as_of.clone(),
        };
        let token_json = serde_json::to_vec_pretty(&token)
            .map_err(|cause| shared::error(format!("cannot encode the consent token: {cause}")))?;
        ConsentToken::parse(&token_json)?;
        artifacts.push(crate::authoring::output::OutputArtifact {
            relative_path: CONSENT_ARTIFACT.to_string(),
            bytes: token_json,
        });
    }

    // Every captured project input and corpus document is revalidated after the
    // payload is built and immediately before any output side effect.
    seed.captures.verify()?;
    if let Some(captured) = &captured {
        captured.verify()?;
    }
    crate::authoring::output::publish(&seed.root, args.output_dir, &artifacts)?;

    let stdout = match args.format {
        AuthorReportFormat::Json => String::from_utf8(request_json)
            .map_err(|cause| shared::error(format!("request JSON encoding: {cause}")))?,
        AuthorReportFormat::Text => render_summary(&request, args),
    };
    crate::cli::output::write_output(&stdout, None)
        .map_err(|cause| shared::error(format!("cannot write prepare summary: {cause}")))?;
    Ok(false)
}

/// Append one unit per unresolved plan section, in plan order.
fn push_plan_sections(drafts: &mut Vec<UnitDraft>, seed: &crate::authoring::SuggestSeed) {
    for policy in &seed.plan.policies {
        for section in &policy.sections {
            if section.state == crate::authoring::model::DraftState::HumanDraftPresent {
                continue;
            }
            let mut text = section.title.clone();
            for question in &section.questions {
                if let Some(prompt) = seed.prompts.get(&question.question_key) {
                    text.push('\n');
                    text.push_str(prompt);
                }
            }
            let unit_id = unit_id(drafts.len());
            drafts.push(UnitDraft {
                unit_id,
                kind: ContextKind::PlanSection,
                label: format!("{} / {}", policy.policy_key, section.title),
                sensitivity: Sensitivity::Internal,
                source: None,
                text,
                section: Some(SectionSeed {
                    policy_key: policy.policy_key.clone(),
                    topic_key: section.topic_key.clone(),
                    order: section.order,
                    gap_ids: section.gap_ids.clone(),
                    control_ids: section.control_ids.clone(),
                }),
            });
        }
    }
}

/// Append one unit per selected approved answer, in the order requested.
fn push_answers(
    drafts: &mut Vec<UnitDraft>,
    seed: &crate::authoring::SuggestSeed,
    args: &PrepareArgs<'_>,
) -> Result<(), ForgeError> {
    for key in args.answers {
        let answer = seed.answers.get(key).ok_or_else(|| {
            shared::error(format!("--answer '{key}' names no provided approved answer"))
        })?;
        let sensitivity = match answer.sensitivity {
            crate::authoring::manifest::Sensitivity::Public => Sensitivity::Public,
            crate::authoring::manifest::Sensitivity::Internal => Sensitivity::Internal,
            crate::authoring::manifest::Sensitivity::Confidential => Sensitivity::Confidential,
            crate::authoring::manifest::Sensitivity::Restricted => Sensitivity::Restricted,
        };
        if sensitivity.requires_acknowledgement() && !args.allow_sensitive {
            return Err(shared::error(format!(
                "--answer '{key}' is {sensitivity} material; pass --allow-sensitive to select it",
                sensitivity = sensitivity.as_str()
            )));
        }
        let unit_id = unit_id(drafts.len());
        drafts.push(UnitDraft {
            unit_id,
            kind: ContextKind::Answer,
            label: format!("answer {} for question {}", answer.key, answer.question_key),
            sensitivity,
            source: None,
            text: answer.value.clone(),
            section: None,
        });
    }
    Ok(())
}

/// Append one unit per selected corpus document, in corpus order.
fn push_source_spans(
    drafts: &mut Vec<UnitDraft>,
    args: &PrepareArgs<'_>,
) -> Result<Option<crate::reuse::capture::CapturedCorpus>, ForgeError> {
    let Some(corpus_path) = args.corpus else {
        if !args.include_documents.is_empty() {
            return Err(shared::error("--include-document requires --corpus"));
        }
        return Ok(None);
    };
    if args.include_documents.is_empty() {
        return Err(shared::error("--corpus requires at least one --include-document"));
    }
    let (root, name) = crate::reuse::corpus_location(corpus_path)?;
    let bytes = crate::reuse::read_manifest(&root, &name)?;
    let corpus = crate::reuse::corpus::Corpus::parse(&bytes)?;
    let captured = crate::reuse::capture::capture(&root, &corpus)?;
    let selected: BTreeSet<&str> = args.include_documents.iter().map(String::as_str).collect();
    for document in &captured.documents {
        if !selected.contains(document.key.as_str()) {
            continue;
        }
        let text = std::str::from_utf8(&document.bytes).map_err(|_| {
            shared::error(format!("corpus document '{}' is not UTF-8", document.key))
        })?;
        let path = document
            .path
            .to_str()
            .ok_or_else(|| shared::error("corpus document path must be UTF-8"))?
            .to_string();
        let span = u64::try_from(document.bytes.len())
            .map_err(|_| shared::error("corpus document length does not fit the contract"))?;
        let unit_id = unit_id(drafts.len());
        drafts.push(UnitDraft {
            unit_id,
            kind: ContextKind::SourceSpan,
            label: format!("document {}", document.key),
            sensitivity: Sensitivity::Internal,
            source: Some(SourceRef {
                key: document.key.clone(),
                path,
                sha256: document.sha256.clone(),
                start: 0,
                end: span,
            }),
            text: text.to_string(),
            section: None,
        });
    }
    for key in &selected {
        if !captured.documents.iter().any(|document| document.key == *key) {
            return Err(shared::error(format!(
                "--include-document '{key}' names no corpus document"
            )));
        }
    }
    Ok(Some(captured))
}

/// Redact every unit, assemble the payload, and build the request's task,
/// units and redaction records.
fn assemble(
    drafts: &[UnitDraft],
    rules: &[RedactionRule],
) -> Result<(RequestTask, String, Vec<ContextUnit>, Vec<RedactionRecord>), ForgeError> {
    let task = RequestTask {
        kind: TaskKind::PolicyDrafting,
        schema_version: drafting::TASK_SCHEMA_VERSION.to_string(),
        mapping_subjects: Vec::new(),
        drafting_sections: drafting_sections(drafts),
    };
    let task_json = serde_json::to_string(&task)
        .map_err(|cause| shared::error(format!("cannot encode the task payload: {cause}")))?;

    let mut payload = String::new();
    payload.push_str(TASK_HEADER);
    payload.push_str(&task_json);
    payload.push('\n');
    payload.push_str(CONTEXT_HEADER);

    let mut units = Vec::new();
    let mut redactions = Vec::new();
    let mut matched: BTreeSet<&str> = BTreeSet::new();
    for draft in drafts {
        let (text, applied) = redact::apply(&draft.text, rules);
        for rule in &applied {
            matched.insert(rule.rule_id.as_str());
            redactions.push(RedactionRecord {
                unit_id: draft.unit_id.clone(),
                rule_id: rule.rule_id.clone(),
                rule_sha256: rule.rule_sha256.clone(),
            });
        }
        payload.push_str(&unit_header(draft));
        let start = payload.len();
        payload.push_str(&text);
        let end = payload.len();
        payload.push_str(&unit_footer(&draft.unit_id));
        units.push(ContextUnit {
            unit_id: draft.unit_id.clone(),
            kind: draft.kind,
            label: draft.label.clone(),
            sensitivity: draft.sensitivity,
            source: draft.source.clone(),
            payload: PayloadSpan {
                start: u64::try_from(start)
                    .map_err(|_| shared::error("payload offset does not fit the contract"))?,
                end: u64::try_from(end)
                    .map_err(|_| shared::error("payload offset does not fit the contract"))?,
            },
        });
    }
    if let Some(unmatched) = redact::unmatched_rule(rules, &matched) {
        return Err(shared::error(format!(
            "redaction rule '{}' matched nothing in any selected unit; remove it or correct the literal",
            unmatched.rule_id
        )));
    }
    if payload.len() as u64 > MAX_PAYLOAD_BYTES {
        return Err(shared::error(format!(
            "the selected context exceeds the {MAX_PAYLOAD_BYTES} byte payload limit; select fewer documents or answers"
        )));
    }
    redact::refuse_secrets(&payload)?;
    Ok((task, payload, units, redactions))
}

/// One drafting section per plan-section unit, in unit order.
fn drafting_sections(drafts: &[UnitDraft]) -> Vec<drafting::DraftingSection> {
    drafts
        .iter()
        .filter_map(|draft| {
            draft.section.as_ref().map(|section| drafting::DraftingSection {
                policy_key: section.policy_key.clone(),
                topic_key: section.topic_key.clone(),
                order: section.order,
                title: draft.label.clone(),
                prompt: draft.text.clone(),
                gap_ids: section.gap_ids.clone(),
                control_ids: section.control_ids.clone(),
            })
        })
        .collect()
}

/// Sequential deterministic unit identifier.
fn unit_id(index: usize) -> String {
    format!("unit-{:04}", index + 1)
}

fn unit_header(draft: &UnitDraft) -> String {
    format!(
        "<<<unit {} kind={} sensitivity={}>>>\n",
        draft.unit_id,
        draft.kind.as_str(),
        draft.sensitivity.as_str()
    )
}

fn unit_footer(unit_id: &str) -> String {
    format!("<<<end {unit_id}>>>\n")
}

/// Read and parse an optional redaction rules file.
fn read_rules(path: Option<&Path>) -> Result<Vec<RedactionRule>, ForgeError> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let bytes = crate::io::read_bounded(path, redact_module::MAX_RULES_BYTES)?;
    redact::parse_rules(&bytes)
}

/// Fingerprint the local adapter and record its operator-supplied identity.
fn adapter_target(adapter: &Path, model_id: &str) -> Result<AdapterTarget, ForgeError> {
    let bytes = crate::io::read_bounded(adapter, MAX_ADAPTER_BYTES)?;
    let executable =
        adapter.to_str().ok_or_else(|| shared::error("--adapter must be UTF-8"))?.to_string();
    Ok(AdapterTarget {
        executable,
        executable_sha256: crate::hashing::sha256_hex(&bytes),
        model_id: model_id.to_string(),
        argv: Vec::new(),
    })
}

/// Render the human preview: metadata, then the exact payload bytes.
fn render_preview(request: &SuggestRequest, payload: &str) -> String {
    let mut preview = String::from("FORGE suggest prepare preview\n");
    let _ = writeln!(preview, "schema: {}", request.schema_version);
    let _ = writeln!(preview, "project: {}", request.project_key);
    let _ = writeln!(preview, "as-of: {}", request.as_of);
    let _ =
        writeln!(preview, "task: {} ({})", request.task.kind.as_str(), request.task.schema_version);
    let _ = writeln!(
        preview,
        "adapter: {} model={} sha256={}",
        request.adapter.executable, request.adapter.model_id, request.adapter.executable_sha256
    );
    let _ = writeln!(
        preview,
        "payload: {} ({} bytes) sha256={}",
        request.payload.artifact, request.payload.bytes, request.payload.sha256
    );
    let _ = writeln!(preview, "units: {}", request.context.units.len());
    for unit in &request.context.units {
        let source = unit.source.as_ref().map_or_else(String::new, |source| {
            format!(
                " source={}[{}..{}) sha256={}",
                source.path, source.start, source.end, source.sha256
            )
        });
        let _ = writeln!(
            preview,
            "  {} kind={} sensitivity={} payload[{}..{}){} label={}",
            unit.unit_id,
            unit.kind.as_str(),
            unit.sensitivity.as_str(),
            unit.payload.start,
            unit.payload.end,
            source,
            unit.label
        );
    }
    let _ = writeln!(preview, "redactions: {}", request.redactions.len());
    preview.push('\n');
    preview.push_str(DATA_HANDLING_NOTICE);
    preview.push_str("\n\nexact payload follows\n===8<===\n");
    preview.push_str(payload);
    preview.push_str("\n===8<===\n");
    preview
}

/// Short stdout summary for the prepared request.
fn render_summary(request: &SuggestRequest, args: &PrepareArgs<'_>) -> String {
    let mut summary = String::new();
    let _ = writeln!(summary, "prepared {}", request.payload.artifact);
    let _ = writeln!(summary, "project: {}", request.project_key);
    let _ =
        writeln!(summary, "task: {} ({})", request.task.kind.as_str(), request.task.schema_version);
    let _ = writeln!(summary, "units: {}", request.context.units.len());
    let _ = writeln!(
        summary,
        "payload bytes: {} sha256: {}",
        request.payload.bytes, request.payload.sha256
    );
    let _ = writeln!(summary, "redactions: {}", request.redactions.len());
    let _ = writeln!(summary, "output: {}", args.output_dir.display());
    summary.push_str(if args.consent {
        "consent token: consent.json (this exact payload only)\n"
    } else {
        "consent token: not written; the run step refuses without one\n"
    });
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_identifiers_are_sequential_and_stable() {
        assert_eq!(unit_id(0), "unit-0001");
        assert_eq!(unit_id(9), "unit-0010");
    }

    #[test]
    fn payload_framing_is_fixed_and_spans_bracket_the_text_exactly() {
        let draft = UnitDraft {
            unit_id: "unit-0001".to_string(),
            kind: ContextKind::SourceSpan,
            label: "document prior-access".to_string(),
            sensitivity: Sensitivity::Internal,
            source: None,
            text: "line one\nline two".to_string(),
            section: None,
        };
        let (task, payload, units, redactions) =
            assemble(std::slice::from_ref(&draft), &[]).unwrap();
        assert_eq!(task.kind, TaskKind::PolicyDrafting);
        assert!(redactions.is_empty());
        let unit = &units[0];
        let start = usize::try_from(unit.payload.start).unwrap();
        let end = usize::try_from(unit.payload.end).unwrap();
        assert_eq!(&payload[start..end], "line one\nline two");
        assert!(payload.contains(TASK_HEADER));
        assert!(payload.contains(CONTEXT_HEADER));
        assert!(payload.contains("<<<unit unit-0001 kind=source-span sensitivity=internal>>>"));
        assert!(payload.contains("<<<end unit-0001>>>"));
    }

    #[test]
    fn payload_assembly_is_byte_identical_across_repeated_runs() {
        let draft = UnitDraft {
            unit_id: "unit-0001".to_string(),
            kind: ContextKind::Answer,
            label: "answer a-1 for question q-1".to_string(),
            sensitivity: Sensitivity::Public,
            source: None,
            text: "quarterly".to_string(),
            section: None,
        };
        let first = assemble(std::slice::from_ref(&draft), &[]).unwrap().1;
        let second = assemble(std::slice::from_ref(&draft), &[]).unwrap().1;
        assert_eq!(first, second);
    }

    #[test]
    fn an_unmatched_rule_or_a_secret_refuses_the_payload() {
        let draft = UnitDraft {
            unit_id: "unit-0001".to_string(),
            kind: ContextKind::Answer,
            label: "answer a-1 for question q-1".to_string(),
            sensitivity: Sensitivity::Public,
            source: None,
            text: "nothing sensitive".to_string(),
            section: None,
        };
        let rules = redact::parse_rules(b"rule\tabsent-literal\n").unwrap();
        assert!(assemble(std::slice::from_ref(&draft), &rules).is_err());

        let secret = UnitDraft { text: "token sk-abcdefghijklmnopqrstuvwxyz".to_string(), ..draft };
        let error = assemble(&[secret], &[]).unwrap_err().to_string();
        assert!(error.contains("secret pattern"), "{error}");
        assert!(!error.contains("sk-abcdefghijklmnopqrstuvwxyz"));
    }

    #[test]
    fn redaction_records_name_the_unit_and_the_rule_digest_only() {
        let draft = UnitDraft {
            unit_id: "unit-0001".to_string(),
            kind: ContextKind::SourceSpan,
            label: "document prior-access".to_string(),
            sensitivity: Sensitivity::Internal,
            source: None,
            text: "account 123456789012".to_string(),
            section: None,
        };
        let rules = redact::parse_rules(b"account-id\t123456789012\n").unwrap();
        let (_, payload, units, redactions) = assemble(&[draft], &rules).unwrap();
        assert_eq!(redactions.len(), 1);
        assert_eq!(redactions[0].unit_id, "unit-0001");
        assert_eq!(redactions[0].rule_id, "account-id");
        assert_eq!(
            redactions[0].rule_sha256,
            crate::hashing::sha256_hex(b"account-id\t123456789012")
        );
        assert!(payload.contains(redact::MARKER));
        assert!(!payload.contains("123456789012"));
        assert_eq!(
            &payload[usize::try_from(units[0].payload.start).unwrap()
                ..usize::try_from(units[0].payload.end).unwrap()],
            "account [redacted]"
        );
    }

    #[test]
    fn the_preview_shows_metadata_and_the_exact_payload_bytes() {
        let draft = UnitDraft {
            unit_id: "unit-0001".to_string(),
            kind: ContextKind::SourceSpan,
            label: "document prior-access".to_string(),
            sensitivity: Sensitivity::Internal,
            source: Some(SourceRef {
                key: "prior-access".to_string(),
                path: "prior/access.md".to_string(),
                sha256: "a".repeat(64),
                start: 0,
                end: 12,
            }),
            text: "exact bytes\n".to_string(),
            section: None,
        };
        let (task, payload, units, redactions) = assemble(&[draft], &[]).unwrap();
        let request = SuggestRequest {
            schema_version: SCHEMA_VERSION.to_string(),
            project_key: "synthetic-project".to_string(),
            as_of: "2026-09-12T00:00:00Z".to_string(),
            task,
            adapter: AdapterTarget {
                executable: "local-model".to_string(),
                executable_sha256: "b".repeat(64),
                model_id: "synthetic".to_string(),
                argv: Vec::new(),
            },
            context: ContextBundle { units },
            redactions,
            payload: PayloadPreview {
                artifact: PAYLOAD_ARTIFACT.to_string(),
                sha256: crate::hashing::sha256_hex(payload.as_bytes()),
                bytes: u64::try_from(payload.len()).unwrap(),
                units: 1,
            },
            retention_notice: DEFAULT_RETENTION_NOTICE.to_string(),
        };
        let preview = render_preview(&request, &payload);
        assert!(preview.contains("task: policy-drafting (forge.suggest-task-drafting/1)"));
        assert!(preview.contains("unit-0001 kind=source-span sensitivity=internal"));
        assert!(preview.contains("source=prior/access.md[0..12)"));
        assert!(preview.contains(DATA_HANDLING_NOTICE));
        assert!(preview.contains("exact bytes"));
        // The preview embeds the payload verbatim, so no absolute path leaks in.
        assert!(!preview.contains(std::env::current_dir().unwrap().to_string_lossy().as_ref()));
    }
}

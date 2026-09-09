//! Deterministic skeletons populated exclusively by exact human clause bytes.

use std::collections::{BTreeMap, BTreeSet};

use pulldown_cmark::{Event, Parser, Tag};
use serde::Serialize;

use super::error;
use super::manifest::{AnswerPin, HumanClause, MAX_CLAUSE_BYTES};
use super::model::{
    AnswerStatus, AuthoringPlan, DraftState, GapPlan, LoadedAuthorProject, LoadedClause,
    PlanProvenance, PolicyPlan, QuestionEvaluation, SectionPlan,
};
use super::output::MAX_OUTPUT_BYTES;
use crate::ForgeError;
use crate::hashing::sha256_hex;

const MAX_SPANS: usize = 100_000;

pub struct RenderedAuthorProject {
    pub policies: Vec<RenderedPolicy>,
    pub provenance: Vec<u8>,
}

pub struct RenderedPolicy {
    pub policy_key: String,
    pub relative_path: String,
    pub markdown: Vec<u8>,
}

/// Zero-based, UTF-8 byte offsets; end is exclusive.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
enum OriginKind {
    GeneratedMetadata,
    HumanClause,
}

#[derive(Debug, Clone, Serialize)]
struct SourceSpan {
    path: String,
    sha256: String,
    bytes: ByteSpan,
}

#[derive(Debug, Clone, Serialize)]
struct SpanOrigin {
    kind: OriginKind,
    field: String,
    policy_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic_key: Option<String>,
    gap_ids: Vec<String>,
    control_ids: Vec<String>,
    control_assignment_keys: Vec<String>,
    family_assignment_keys: Vec<String>,
    answer_refs: Vec<AnswerPin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clause_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<SourceSpan>,
}

#[derive(Debug, Clone, Serialize)]
struct ProvenanceSpan {
    output: ByteSpan,
    origin: SpanOrigin,
}

#[derive(Debug, Serialize)]
struct PolicyProvenance {
    policy_key: String,
    path: String,
    sha256: String,
    byte_length: usize,
    spans: Vec<ProvenanceSpan>,
}

#[derive(Serialize)]
struct ProvenanceGraph<'a> {
    schema_version: &'static str,
    project_key: &'a str,
    plan_sha256: String,
    input_provenance: &'a PlanProvenance,
    gaps: &'a [GapPlan],
    questions: &'a [QuestionEvaluation],
    policy_plans: &'a [PolicyPlan],
    clauses: Vec<&'a HumanClause>,
    policies: Vec<PolicyProvenance>,
}

struct TextBuilder {
    bytes: Vec<u8>,
    spans: Vec<ProvenanceSpan>,
}

struct BoundedJson {
    bytes: Vec<u8>,
    limit: usize,
}

impl std::io::Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("authoring provenance exceeds the output limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl TextBuilder {
    fn append(&mut self, bytes: &[u8], origin: SpanOrigin) -> Result<(), ForgeError> {
        if bytes.is_empty() {
            return Ok(());
        }
        if self.bytes.len().saturating_add(bytes.len()) > MAX_OUTPUT_BYTES {
            return Err(error("rendered policy exceeds the 50 MiB output limit"));
        }
        if self.spans.len() >= MAX_SPANS {
            return Err(error("rendered policy exceeds the provenance span limit"));
        }
        let start = self.bytes.len();
        self.bytes.extend_from_slice(bytes);
        self.spans
            .push(ProvenanceSpan { output: ByteSpan { start, end: self.bytes.len() }, origin });
        Ok(())
    }
}

/// Validate every clause, including clauses whose sections are blocked.
///
/// Phase 1 accepts UTF-8 Markdown paragraphs, lists, blockquotes, and inline
/// formatting. Headings, raw HTML, and code blocks are rejected so independent
/// fragments cannot change or swallow the generated outline. No templating or
/// answer substitution is performed.
///
/// # Errors
///
/// Returns an authoring error for unsupported structural content or bounds.
pub(crate) fn validate_clause(bytes: &[u8]) -> Result<(), ForgeError> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_CLAUSE_BYTES {
        return Err(error("human clause must contain 1 byte through 1 MiB"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| error("human clause must be valid UTF-8"))?;
    if text.trim().is_empty() {
        return Err(error("human clause must contain visible content"));
    }
    if text.contains("{{forge:") {
        return Err(error("human clauses cannot contain reserved FORGE template syntax"));
    }
    // Reference definitions have document-wide scope and may change another
    // fragment's rendering even when no link exists in this clause alone.
    if text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with('[') && line.contains("]:")
    }) {
        return Err(error("human clause reference definitions are unsupported"));
    }
    if text.chars().any(|character| {
        (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
            || matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    }) {
        return Err(error("human clause contains unsupported control characters"));
    }
    for event in Parser::new(text) {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                return Err(error("human clause headings cannot alter the policy outline"));
            }
            Event::Start(Tag::CodeBlock(_)) => {
                return Err(error("human clause code blocks are unsupported in Phase 1"));
            }
            Event::Start(Tag::HtmlBlock) | Event::Html(_) | Event::InlineHtml(_) => {
                return Err(error("human clause raw HTML is unsupported"));
            }
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. })
                if absolute_local_reference(&dest_url) =>
            {
                return Err(error("human clause links cannot contain absolute local paths"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn absolute_local_reference(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with('\\')
        || value.to_ascii_lowercase().starts_with("file:")
        || (value.as_bytes().get(1) == Some(&b':')
            && value.as_bytes().first().is_some_and(u8::is_ascii_alphabetic))
}

/// Render a validated authoring plan and complete span provenance in memory.
///
/// # Errors
///
/// Returns an authoring error if clause bytes drift, provenance cannot be
/// reconciled, clause structure is unsupported, or rendered bounds are exceeded.
pub fn render(
    loaded: &LoadedAuthorProject,
    plan: &AuthoringPlan,
) -> Result<RenderedAuthorProject, ForgeError> {
    for clause in loaded.clauses.values() {
        validate_clause(&clause.bytes)?;
        if sha256_hex(&clause.bytes) != clause.source.source.expected_sha256 {
            return Err(error("human clause SHA-256 changed before rendering"));
        }
    }
    let mut policy_plans = plan.policies.iter().collect::<Vec<_>>();
    policy_plans.sort_by(|left, right| left.policy_key.cmp(&right.policy_key));
    let mut policies = Vec::with_capacity(policy_plans.len());
    let mut provenance = Vec::with_capacity(policy_plans.len());
    let mut total_bytes = 0usize;
    let mut total_spans = 0usize;
    let mut seen = BTreeSet::new();
    let gap_controls = plan
        .gaps
        .iter()
        .map(|gap| (gap.gap_id.clone(), gap.control_id.clone()))
        .collect::<BTreeMap<_, _>>();
    for policy in policy_plans {
        if !seen.insert(&policy.policy_key) {
            return Err(error("duplicate rendered policy key"));
        }
        let (rendered, source_map) = render_policy(policy, &loaded.clauses, &gap_controls)?;
        total_bytes = total_bytes.saturating_add(rendered.markdown.len());
        total_spans = total_spans.saturating_add(source_map.spans.len());
        if total_bytes > MAX_OUTPUT_BYTES || total_spans > MAX_SPANS {
            return Err(error("authoring generation exceeds rendering bounds"));
        }
        policies.push(rendered);
        provenance.push(source_map);
    }
    let graph = ProvenanceGraph {
        schema_version: "forge.authoring-provenance/1",
        project_key: &plan.project_key,
        plan_sha256: sha256_hex(&super::report::render_json(plan)?),
        input_provenance: &plan.provenance,
        gaps: &plan.gaps,
        questions: &plan.questions,
        policy_plans: &plan.policies,
        clauses: loaded.clauses.values().map(|clause| &clause.source).collect(),
        policies: provenance,
    };
    let mut destination = BoundedJson {
        bytes: Vec::new(),
        limit: MAX_OUTPUT_BYTES.saturating_sub(total_bytes).saturating_sub(1),
    };
    serde_json::to_writer_pretty(&mut destination, &graph)
        .map_err(|cause| error(format!("cannot serialize authoring provenance: {cause}")))?;
    let mut provenance = destination.bytes;
    provenance.push(b'\n');
    if total_bytes.saturating_add(provenance.len()) > MAX_OUTPUT_BYTES {
        return Err(error("authoring provenance and policies exceed the 50 MiB output limit"));
    }
    Ok(RenderedAuthorProject { policies, provenance })
}

fn render_policy(
    policy: &PolicyPlan,
    clauses: &BTreeMap<String, LoadedClause>,
    gap_controls: &BTreeMap<String, String>,
) -> Result<(RenderedPolicy, PolicyProvenance), ForgeError> {
    let mut text = TextBuilder { bytes: Vec::new(), spans: Vec::new() };
    text.append(
        format!("# {}\n\n", escape_markdown(&policy.title)).as_bytes(),
        metadata_origin(policy, None, "policy-title"),
    )?;
    text.append(
        format!("> Draft state: {}. Human review remains pending.\n\n", policy.state.as_str())
            .as_bytes(),
        metadata_origin(policy, None, "policy-state"),
    )?;
    let mut sections = policy.sections.iter().collect::<Vec<_>>();
    sections
        .sort_by(|left, right| (left.order, &left.topic_key).cmp(&(right.order, &right.topic_key)));
    let mut seen = BTreeSet::new();
    for section in sections {
        if !seen.insert(&section.topic_key) {
            return Err(error("duplicate rendered section topic"));
        }
        render_section(&mut text, policy, section, clauses, gap_controls)?;
    }
    if policy.sections.is_empty() {
        text.append(
            b"\\[UNRESOLVED: No policy topics have been assigned.\\]\n",
            metadata_origin(policy, None, "unresolved-topics"),
        )?;
    }
    let relative_path = format!("policies/{}.md", policy.policy_key);
    let provenance = PolicyProvenance {
        policy_key: policy.policy_key.clone(),
        path: relative_path.clone(),
        sha256: sha256_hex(&text.bytes),
        byte_length: text.bytes.len(),
        spans: text.spans,
    };
    Ok((
        RenderedPolicy {
            policy_key: policy.policy_key.clone(),
            relative_path,
            markdown: text.bytes,
        },
        provenance,
    ))
}

fn render_section(
    text: &mut TextBuilder,
    policy: &PolicyPlan,
    section: &SectionPlan,
    clauses: &BTreeMap<String, LoadedClause>,
    gap_controls: &BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    text.append(
        format!(
            "## {}\n\nDraft state: {}.\n\n",
            escape_markdown(&section.title),
            section.state.as_str()
        )
        .as_bytes(),
        metadata_origin(policy, Some(section), "section-title-and-state"),
    )?;
    for question in &section.questions {
        if question.state != AnswerStatus::Available {
            let kind = if question.required { "CONTEXT" } else { "OPTIONAL CONTEXT" };
            text.append(
                format!(
                    "\\[UNRESOLVED {kind}: {} ({})\\]\n\n",
                    escape_markdown(&question.question_key),
                    question.state.as_str()
                )
                .as_bytes(),
                metadata_origin(policy, Some(section), "unresolved-question"),
            )?;
        }
    }
    if section.state == DraftState::BlockedContext {
        text.append(
            b"\\[UNRESOLVED: Required context prevents inclusion of human clauses for this section.\\]\n\n",
            metadata_origin(policy, Some(section), "blocked-context"),
        )?;
        return Ok(());
    }
    if section.clause_keys.is_empty() {
        text.append(
            b"\\[UNRESOLVED: Human-authored clause content is pending.\\]\n\n",
            metadata_origin(policy, Some(section), "unresolved-clause"),
        )?;
    }
    let mut clause_keys = section.clause_keys.iter().collect::<Vec<_>>();
    clause_keys.sort();
    let mut seen = BTreeSet::new();
    for key in clause_keys {
        if !seen.insert(key) {
            return Err(error("duplicate rendered clause reference"));
        }
        let clause = clauses.get(key).ok_or_else(|| error("planned human clause is missing"))?;
        let origin = clause_origin(policy, section, clause, gap_controls)?;
        text.append(&clause.bytes, origin)?;
        text.append(
            if clause.bytes.ends_with(b"\n") { b"\n" } else { b"\n\n" },
            metadata_origin(policy, Some(section), "clause-separator"),
        )?;
    }
    Ok(())
}

fn metadata_origin(policy: &PolicyPlan, section: Option<&SectionPlan>, field: &str) -> SpanOrigin {
    SpanOrigin {
        kind: OriginKind::GeneratedMetadata,
        field: field.to_string(),
        policy_key: policy.policy_key.clone(),
        topic_key: section.map(|section| section.topic_key.clone()),
        // Generated spans resolve their complete context through policy_plans
        // using the policy/topic keys. Do not duplicate wide section graphs for
        // every unresolved marker; that would amplify bounded inputs quadratically.
        gap_ids: Vec::new(),
        control_ids: Vec::new(),
        control_assignment_keys: Vec::new(),
        family_assignment_keys: Vec::new(),
        answer_refs: Vec::new(),
        clause_key: None,
        source: None,
    }
}

fn clause_origin(
    policy: &PolicyPlan,
    section: &SectionPlan,
    loaded_clause: &LoadedClause,
    gap_controls: &BTreeMap<String, String>,
) -> Result<SpanOrigin, ForgeError> {
    let clause = &loaded_clause.source;
    if clause.policy_key != policy.policy_key || clause.topic_key != section.topic_key {
        return Err(error("planned clause does not match its explicit policy/topic relationship"));
    }
    let mut origin = metadata_origin(policy, Some(section), "human-clause");
    origin.kind = OriginKind::HumanClause;
    origin.gap_ids.clone_from(&clause.gap_ids);
    origin.gap_ids.sort();
    for gap in &origin.gap_ids {
        if !section.gap_ids.contains(gap) {
            return Err(error("human clause gap is not assigned to its section"));
        }
        origin.control_ids.push(
            gap_controls
                .get(gap)
                .ok_or_else(|| error("human clause gap has no baseline control"))?
                .clone(),
        );
    }
    origin.control_ids.sort();
    origin.control_ids.dedup();
    let assignments = section
        .assignments
        .iter()
        .filter(|assignment| origin.control_ids.contains(&assignment.control_assignment.control_id))
        .collect::<Vec<_>>();
    origin.control_assignment_keys = assignments
        .iter()
        .map(|item| item.control_assignment.key.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    origin.family_assignment_keys = assignments
        .iter()
        .map(|item| item.family_assignment.key.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    origin.answer_refs.clone_from(&clause.answer_refs);
    origin.answer_refs.sort_by(|left, right| left.answer_key.cmp(&right.answer_key));
    origin.clause_key = Some(clause.key.clone());
    let source_path = clause
        .source
        .path
        .to_str()
        .ok_or_else(|| error("human clause source label must be UTF-8"))?;
    if absolute_local_reference(source_path) || source_path.contains('\\') {
        return Err(error("human clause source label must be project-relative"));
    }
    origin.source = Some(SourceSpan {
        path: source_path.to_string(),
        sha256: clause.source.expected_sha256.clone(),
        bytes: ByteSpan { start: 0, end: loaded_clause.bytes.len() },
    });
    Ok(origin)
}

fn escape_markdown(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        if character.is_ascii_punctuation() {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::super::manifest::{PinnedFile, Review, Sensitivity};
    use super::*;

    fn policy() -> PolicyPlan {
        PolicyPlan {
            policy_key: "example".into(),
            policy_family_key: "family".into(),
            title: "Explicit *title*".into(),
            state: DraftState::HumanDraftPresent,
            sections: vec![SectionPlan {
                topic_key: "topic".into(),
                title: "Human section".into(),
                order: 1,
                state: DraftState::HumanDraftPresent,
                gap_ids: vec!["gap-one".into()],
                control_ids: vec!["ctrl-1".into()],
                assignments: Vec::new(),
                questions: Vec::new(),
                clause_keys: vec!["clause".into()],
            }],
        }
    }

    fn clauses(bytes: &[u8]) -> BTreeMap<String, LoadedClause> {
        BTreeMap::from([(
            "clause".into(),
            LoadedClause {
                bytes: bytes.to_vec(),
                source: HumanClause {
                    key: "clause".into(),
                    policy_key: "example".into(),
                    topic_key: "topic".into(),
                    gap_ids: vec!["gap-one".into()],
                    answer_refs: Vec::new(),
                    source: PinnedFile {
                        path: "clauses/human.md".into(),
                        expected_sha256: sha256_hex(bytes),
                    },
                    review: Review {
                        reviewer_key: "human".into(),
                        reviewed_at: "2026-09-08T00:00:00Z".into(),
                        rationale: "Explicit human text".into(),
                    },
                },
            },
        )])
    }

    fn gap_controls() -> BTreeMap<String, String> {
        BTreeMap::from([("gap-one".into(), "ctrl-1".into())])
    }

    #[test]
    fn clause_grammar_rejects_outline_changes_html_fences_and_template_syntax() {
        for source in [
            "# Title",
            "## Section",
            "### Subsection",
            "Title\n=====",
            "<script>alert(1)</script>",
            "<!-- open comment",
            "```\nopen fence",
            "    code",
            "{{forge:param:owner}}",
            "[local](/private/secret)",
            "[local](file:///private/secret)",
            "[unreferenced]: /private/secret",
        ] {
            assert!(validate_clause(source.as_bytes()).is_err(), "{source}");
        }
        assert!(validate_clause(b"\xff").is_err());
        assert!(validate_clause(b" \r\n ").is_err());
        validate_clause("Explicit résumé text.\r\n\r\n- A human list item.\r\n".as_bytes())
            .unwrap();
    }

    #[test]
    fn every_output_byte_has_exact_origin_and_clause_bytes_are_unchanged() {
        let source = "Human résumé text.\r\nSecond line without final newline".as_bytes();
        let (rendered, provenance) =
            render_policy(&policy(), &clauses(source), &gap_controls()).unwrap();
        let mut cursor = 0;
        for span in &provenance.spans {
            assert_eq!(span.output.start, cursor);
            assert!(span.output.end > span.output.start);
            assert!(
                std::str::from_utf8(&rendered.markdown[span.output.start..span.output.end]).is_ok()
            );
            cursor = span.output.end;
            if matches!(span.origin.kind, OriginKind::HumanClause) {
                assert_eq!(&rendered.markdown[span.output.start..span.output.end], source);
                assert_eq!(span.origin.source.as_ref().unwrap().sha256, sha256_hex(source));
                assert_eq!(
                    span.origin.source.as_ref().unwrap().bytes,
                    ByteSpan { start: 0, end: source.len() }
                );
            }
        }
        assert_eq!(cursor, rendered.markdown.len());
        let markdown = std::str::from_utf8(&rendered.markdown).unwrap();
        let headings = Parser::new(markdown)
            .filter(|event| matches!(event, Event::Start(Tag::Heading { .. })))
            .count();
        assert_eq!(headings, 2);
        assert_eq!(provenance.sha256, sha256_hex(&rendered.markdown));
    }

    #[test]
    fn blocked_section_omits_clause_while_unrelated_section_renders() {
        let mut policy = policy();
        let mut blocked = policy.sections[0].clone();
        blocked.topic_key = "blocked-topic".into();
        blocked.title = "Blocked section".into();
        blocked.order = 0;
        blocked.state = DraftState::BlockedContext;
        blocked.questions = vec![QuestionEvaluation {
            question_key: "organization-owner".into(),
            question_sha256: "a".repeat(64),
            required: true,
            owner: "human".into(),
            sensitivity: Sensitivity::Internal,
            source_label: "human interview".into(),
            answer_key: None,
            answer_sha256: None,
            state: AnswerStatus::Missing,
            review: None,
            expires_at: None,
        }];
        policy.sections.push(blocked);
        let (rendered, _) =
            render_policy(&policy, &clauses(b"Exact supplied human words."), &gap_controls())
                .unwrap();
        let markdown = String::from_utf8(rendered.markdown).unwrap();
        assert_eq!(markdown.matches("Exact supplied human words.").count(), 1);
        assert!(markdown.contains("UNRESOLVED CONTEXT"));
        assert!(
            markdown.find("Blocked section").unwrap() < markdown.find("Human section").unwrap()
        );
    }

    #[test]
    fn repeated_render_is_byte_identical_and_contains_no_generated_verdict() {
        let first =
            render_policy(&policy(), &clauses(b"Human-supplied text."), &gap_controls()).unwrap();
        let second =
            render_policy(&policy(), &clauses(b"Human-supplied text."), &gap_controls()).unwrap();
        assert_eq!(first.0.markdown, second.0.markdown);
        assert_eq!(serde_json::to_vec(&first.1).unwrap(), serde_json::to_vec(&second.1).unwrap());
        let text = String::from_utf8(first.0.markdown).unwrap();
        for term in ["compliant", "certified", "implemented", "effective", "approved"] {
            assert!(!text.contains(term));
        }
    }
}

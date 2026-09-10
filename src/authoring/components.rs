//! Explicit, hash-pinned PRD-059 component adaptation without publication.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Parser, Tag};
use serde_json::Value;

use super::component_model::{
    AuthorComponent, AuthorComponents, BindingEvidence, COMPONENTS_SCHEMA_VERSION,
    ComponentEvidence, ComponentFragment, ComponentOrigin, ComponentSpan, LoadedComponents,
    LoadedInstance, ParameterBinding,
};
use super::manifest::{self, AnswerPin, PinnedFile, Sensitivity};
use super::model::{AnswerStatus, AuthoringPlan, DraftState, LoadedAuthorProject, SectionPlan};
use super::render::ByteSpan;
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::policy::manifest::{ComponentInstance, ComponentManifest, ParameterValue};
use crate::policy::render::{ProvenanceOrigin, TextSpan};

const MAX_INSTANCES: usize = 1_000;
const MAX_BINDINGS: usize = 128;
const MAX_GRAPH_RECORDS: usize = 100_000;

/// Decode the separate closed extension; `/1` projects retain their original contract.
///
/// # Errors
/// Rejects unsupported versions, duplicate or unknown keys, nulls and exceeded bounds.
pub fn parse(bytes: &[u8]) -> Result<AuthorComponents, ForgeError> {
    if bytes.len() as u64 > manifest::MAX_MANIFEST_BYTES {
        return Err(super::error("component extension exceeds the 2 MiB manifest limit"));
    }
    let value = crate::json_strict::parse_value(
        bytes,
        "author components",
        crate::json_strict::Limits { max_depth: 32, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|cause| super::error(cause.to_string()))?;
    reject_null(&value)?;
    let extension: AuthorComponents = serde_json::from_value(value)
        .map_err(|cause| super::error(format!("invalid component extension: {cause}")))?;
    if extension.schema_version != COMPONENTS_SCHEMA_VERSION {
        return Err(super::error("unsupported author-components schema_version"));
    }
    sha(&extension.project_sha256)?;
    if extension.instances.is_empty() || extension.instances.len() > MAX_INSTANCES {
        return Err(super::error("component extension requires 1..=1000 instances"));
    }
    let mut keys = BTreeSet::new();
    let mut sections = BTreeSet::new();
    let mut records = 0usize;
    for instance in &extension.instances {
        for name in [&instance.instance_key, &instance.policy_key, &instance.topic_key] {
            key(name)?;
        }
        if !keys.insert(&instance.instance_key) {
            return Err(super::error("duplicate component instance key"));
        }
        if !sections.insert((&instance.policy_key, &instance.topic_key)) {
            return Err(super::error("author-components/1 permits one component per policy/topic"));
        }
        if instance.gap_ids.is_empty() || instance.gap_ids.len() > manifest::MAX_REFERENCES {
            return Err(super::error("component instance requires 1..=128 gap IDs"));
        }
        let mut gaps = BTreeSet::new();
        for gap in &instance.gap_ids {
            sha(gap)?;
            if !gaps.insert(gap) {
                return Err(super::error("duplicate component gap reference"));
            }
        }
        pin(&instance.component_manifest, "json")?;
        pin(&instance.source, "md")?;
        key(&instance.review.reviewer_key)?;
        if !manifest::has_nonblank_text(&instance.review.rationale)
            || instance.review.rationale.len() > manifest::MAX_STRING_BYTES
            || instance.review.rationale.chars().any(|character| character.is_control()
                || matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
        {
            return Err(super::error("component inclusion requires bounded review rationale"));
        }
        timestamp(&instance.review.reviewed_at)?;
        if instance.parameters.len() > MAX_BINDINGS {
            return Err(super::error("component instance exceeds 128 parameter bindings"));
        }
        records = records.saturating_add(instance.parameters.len() + instance.gap_ids.len());
        if records > MAX_GRAPH_RECORDS {
            return Err(super::error("component dependency graph exceeds 100000 records"));
        }
        for (name, binding) in &instance.parameters {
            key(name)?;
            if crate::policy::manifest::is_sensitive_parameter_name(name) {
                return Err(super::error("secret-like component parameter name is forbidden"));
            }
            match binding {
                ParameterBinding::Literal { value, sensitivity } => {
                    allowed_sensitivity(*sensitivity)?;
                    validate_parameter_text(value)?;
                }
                ParameterBinding::Answer { question_key, answer_key, expected_sha256 } => {
                    key(question_key)?;
                    key(answer_key)?;
                    sha(expected_sha256)?;
                }
            }
        }
    }
    Ok(extension)
}

/// Capture and validate every selected component, including blocked sections.
/// The caller supplies its existing confined, bounded, alias-rejecting capture reader.
/// `bytes` must also come from that capture set, reverified before publication.
///
/// # Errors
/// Rejects drift, hidden defaults, unsafe structure, protected values or invalid assignments.
pub fn prepare(
    bytes: &[u8],
    loaded: &LoadedAuthorProject,
    plan: &AuthoringPlan,
    mut read: impl FnMut(&str, &PinnedFile, u64) -> Result<Vec<u8>, ForgeError>,
) -> Result<LoadedComponents, ForgeError> {
    let mut extension = parse(bytes)?;
    if extension.project_sha256 != loaded.project_sha256 {
        return Err(super::error("component extension does not pin the exact author project"));
    }
    extension.instances.sort_by(|left, right| left.instance_key.cmp(&right.instance_key));
    let mut cache = BTreeMap::new();
    let mut total_bytes = 0usize;
    let mut total_spans = 0usize;
    let mut instances = BTreeMap::new();
    for instance in extension.instances {
        let section = validate_assignment(&instance, loaded, plan)?;
        let sidecar = cached_read(
            &mut cache,
            &mut read,
            &format!("component-manifest-{}", instance.instance_key),
            &instance.component_manifest,
            manifest::MAX_MANIFEST_BYTES,
        )?;
        let component = crate::policy::manifest::parse_component(&sidecar)
            .map_err(|cause| super::error(format!("component sidecar: {cause}")))?;
        validate_source_pin(&instance, &component)?;
        let source = cached_read(
            &mut cache,
            &mut read,
            &format!("component-source-{}", instance.instance_key),
            &instance.source,
            manifest::MAX_CLAUSE_BYTES,
        )?;
        validate_structure(&component, &source, &section.title)?;
        let (values, bindings, unavailable) = bind_parameters(&instance, &component, loaded)?;
        let blocked = unavailable || section.state == DraftState::BlockedContext;
        let fragment = if blocked {
            None
        } else {
            let captured = crate::policy::LoadedComponent {
                instance: ComponentInstance {
                    instance_key: instance.instance_key.clone(),
                    component_manifest: instance.component_manifest.path.clone(),
                    parameters: values,
                },
                manifest: component.clone(),
                manifest_sha256: instance.component_manifest.expected_sha256.clone(),
                source_bytes: source.clone(),
                source_sha256: instance.source.expected_sha256.clone(),
                source_label: label(&instance.source.path)?.to_owned(),
            };
            let fragment = convert_fragment(
                &captured,
                &bindings,
                super::output::MAX_OUTPUT_BYTES.saturating_sub(total_bytes),
                MAX_GRAPH_RECORDS.saturating_sub(total_spans),
            )?;
            total_bytes = total_bytes.saturating_add(fragment.markdown.len());
            total_spans = total_spans.saturating_add(fragment.spans.len());
            if total_bytes > super::output::MAX_OUTPUT_BYTES || total_spans > MAX_GRAPH_RECORDS {
                return Err(super::error("component fragments exceed aggregate output bounds"));
            }
            Some(fragment)
        };
        let mut gaps = instance.gap_ids.clone();
        gaps.sort();
        let evidence = ComponentEvidence {
            instance_key: instance.instance_key.clone(),
            policy_key: instance.policy_key,
            topic_key: instance.topic_key,
            gap_ids: gaps,
            component_manifest: instance.component_manifest,
            source: instance.source,
            component_key: component.component_key,
            version: component.version,
            status: component.status,
            review: instance.review,
            bindings,
            blocked,
        };
        instances.insert(instance.instance_key, LoadedInstance { evidence, fragment });
    }
    Ok(LoadedComponents { manifest_sha256: sha256_hex(bytes), instances })
}

fn validate_assignment<'a>(
    instance: &AuthorComponent,
    loaded: &LoadedAuthorProject,
    plan: &'a AuthoringPlan,
) -> Result<&'a SectionPlan, ForgeError> {
    if !loaded.project.reviewers.iter().any(|item| item.key == instance.review.reviewer_key) {
        return Err(super::error("component inclusion names an unknown project reviewer"));
    }
    if timestamp(&instance.review.reviewed_at)? > timestamp(&loaded.project.as_of)? {
        return Err(super::error("component inclusion review is after project as_of"));
    }
    let section = plan
        .policies
        .iter()
        .find(|policy| policy.policy_key == instance.policy_key)
        .and_then(|policy| policy.sections.iter().find(|item| item.topic_key == instance.topic_key))
        .ok_or_else(|| super::error("component has no explicit policy/topic assignment"))?;
    if instance.gap_ids.iter().any(|gap| !section.gap_ids.contains(gap)) {
        return Err(super::error("component gap has no explicit assignment to its section"));
    }
    Ok(section)
}

fn validate_source_pin(
    instance: &AuthorComponent,
    component: &ComponentManifest,
) -> Result<(), ForgeError> {
    let parent = instance.component_manifest.path.parent().unwrap_or_else(|| Path::new(""));
    let base = label(parent)?;
    let relative = label(&component.source)?;
    let source = if base.is_empty() { relative.to_owned() } else { format!("{base}/{relative}") };
    let path = PathBuf::from(source);
    manifest::validate_local_path("component source", &path)?;
    if path != instance.source.path || component.expected_sha256 != instance.source.expected_sha256
    {
        return Err(super::error("component sidecar source must match the explicit source pin"));
    }
    Ok(())
}

fn validate_structure(
    component: &ComponentManifest,
    source: &[u8],
    title: &str,
) -> Result<(), ForgeError> {
    crate::policy::render::validate_static_component(component, "selected component", source)
        .map_err(|cause| super::error(format!("component grammar: {cause}")))?;
    let text =
        std::str::from_utf8(source).map_err(|_| super::error("component source must be UTF-8"))?;
    let escaped_title = super::render::escape_markdown(title);
    let heading = format!("## {escaped_title}");
    if text.lines().next() != Some(heading.as_str()) {
        return Err(super::error(
            "component first heading must exactly match the escaped topic title",
        ));
    }
    if text.chars().any(|character| {
        (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
            || matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    }) {
        return Err(super::error("component source contains unsupported control characters"));
    }
    if text.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with('[') && line.contains("]:")
    }) {
        return Err(super::error(
            "component reference definitions have unsupported document-wide scope",
        ));
    }
    for event in Parser::new(text) {
        match event {
            Event::Html(_) | Event::InlineHtml(_) | Event::Start(Tag::HtmlBlock) => {
                return Err(super::error("raw HTML is unsupported in authoring components"));
            }
            Event::Start(Tag::CodeBlock(_)) => {
                return Err(super::error("code blocks are unsupported in authoring components"));
            }
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                let url = dest_url.to_ascii_lowercase();
                if !(url.starts_with("https://")
                    || url.starts_with("http://")
                    || url.starts_with('#'))
                {
                    return Err(super::error("component link must be http(s) or a local fragment"));
                }
            }
            _ => {}
        }
    }
    let used = crate::policy::render::parameter_names(source)?;
    let declared: BTreeSet<_> = component.parameters.iter().map(|item| item.name.clone()).collect();
    if used != declared {
        return Err(super::error("all component parameter declarations must be used explicitly"));
    }
    Ok(())
}

type BoundParameters = (BTreeMap<String, ParameterValue>, Vec<BindingEvidence>, bool);

#[allow(clippy::too_many_lines)] // One ordered transaction keeps values and redacted evidence aligned.
fn bind_parameters(
    instance: &AuthorComponent,
    component: &ComponentManifest,
    loaded: &LoadedAuthorProject,
) -> Result<BoundParameters, ForgeError> {
    let names: BTreeSet<_> = component.parameters.iter().map(|item| item.name.as_str()).collect();
    if instance.parameters.keys().map(String::as_str).collect::<BTreeSet<_>>() != names {
        return Err(super::error(
            "every component parameter requires an explicit binding; defaults are never implicit",
        ));
    }
    let topic = loaded
        .pack
        .topics
        .iter()
        .find(|topic| topic.key == instance.topic_key)
        .ok_or_else(|| super::error("component references an unknown topic"))?;
    let mut values = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut unavailable = false;
    for declaration in &component.parameters {
        let binding = &instance.parameters[&declaration.name];
        let (value, mut item) = match binding {
            ParameterBinding::Literal { value, sensitivity } => {
                allowed_sensitivity(*sensitivity)?;
                crate::policy::manifest::validate_value("component literal", declaration, value)
                    .map_err(|cause| super::error(format!("invalid component literal: {cause}")))?;
                (
                    Some(value.clone()),
                    BindingEvidence {
                        parameter_name: declaration.name.clone(),
                        value_sha256: None,
                        answer_ref: None,
                        observed_answer_sha256: None,
                        question_key: None,
                        sensitivity: *sensitivity,
                        answer_state: None,
                    },
                )
            }
            ParameterBinding::Answer { question_key, answer_key, expected_sha256 } => {
                if !topic.question_keys.contains(question_key) {
                    return Err(super::error(
                        "component answer binding must identify a question explicitly assigned to its topic",
                    ));
                }
                let question = loaded
                    .pack
                    .questions
                    .iter()
                    .find(|question| question.key == *question_key)
                    .ok_or_else(|| super::error("component binding names an unknown question"))?;
                allowed_sensitivity(question.sensitivity)?;
                let answer = loaded.project.answers.iter().find(|answer| answer.key == *answer_key);
                let mut observed_answer_sha256 = None;
                if let Some(answer) = answer {
                    allowed_sensitivity(answer.sensitivity)?;
                    if answer.question_key != *question_key {
                        return Err(super::error(
                            "component answer binding names the wrong question",
                        ));
                    }
                    observed_answer_sha256 = Some(manifest::answer_sha256(answer)?);
                }
                let evaluated = manifest::evaluate_question(
                    question,
                    answer,
                    &loaded.pack_sha256,
                    &loaded.project.as_of,
                )?;
                let mut state = if observed_answer_sha256
                    .as_ref()
                    .is_some_and(|observed| observed != expected_sha256)
                {
                    AnswerStatus::Stale
                } else {
                    evaluated.state
                };
                let value = if state == AnswerStatus::Available {
                    let value = answer
                        .and_then(|answer| answer.value.clone())
                        .and_then(|value| serde_json::from_value::<ParameterValue>(value).ok());
                    if value.as_ref().is_none_or(|value| {
                        validate_parameter_text(value).is_err()
                            || crate::policy::manifest::validate_value(
                                "component answer",
                                declaration,
                                value,
                            )
                            .is_err()
                    }) {
                        state = AnswerStatus::Invalid;
                        None
                    } else {
                        value
                    }
                } else {
                    None
                };
                unavailable |= state != AnswerStatus::Available;
                (
                    value,
                    BindingEvidence {
                        parameter_name: declaration.name.clone(),
                        value_sha256: None,
                        answer_ref: Some(AnswerPin {
                            answer_key: answer_key.clone(),
                            expected_sha256: expected_sha256.clone(),
                        }),
                        observed_answer_sha256,
                        question_key: Some(question_key.clone()),
                        sensitivity: question.sensitivity,
                        answer_state: Some(state),
                    },
                )
            }
        };
        if let Some(value) = value {
            item.value_sha256 = Some(sha256_hex(&serde_json::to_vec(&value).map_err(|cause| {
                super::error(format!("cannot hash component parameter: {cause}"))
            })?));
            values.insert(declaration.name.clone(), value);
        }
        evidence.push(item);
    }
    evidence.sort_by(|left, right| left.parameter_name.cmp(&right.parameter_name));
    Ok((values, evidence, unavailable))
}

fn convert_fragment(
    component: &crate::policy::LoadedComponent,
    bindings: &[BindingEvidence],
    max_bytes: usize,
    max_spans: usize,
) -> Result<ComponentFragment, ForgeError> {
    let rendered = crate::policy::render::render_fragment(component, max_bytes, max_spans)
        .map_err(|cause| super::error(format!("component rendering: {cause}")))?;
    let output = std::str::from_utf8(&rendered.markdown)
        .map_err(|_| super::error("rendered component must be UTF-8"))?;
    let source = std::str::from_utf8(&component.source_bytes)
        .map_err(|_| super::error("component source must be UTF-8"))?;
    let mut output_cursor = SpanCursor::new(output);
    let mut source_cursor = SpanCursor::new(source);
    let mut spans = Vec::new();
    let mut cursor = 0usize;
    for span in rendered.spans {
        let output_span = output_cursor.bytes_for_span(&span.output)?;
        if output_span.start > cursor {
            append_newline(&mut spans, output, cursor, output_span.start, max_spans)?;
        }
        if output_span.start < cursor {
            return Err(super::error("component output spans overlap"));
        }
        let (kind, source_span, name, digest) = match span.origin {
            ProvenanceOrigin::Component { source: span, .. } => (
                ComponentOrigin::ComponentSource,
                Some(source_cursor.bytes_for_span(&span)?),
                None,
                None,
            ),
            ProvenanceOrigin::Parameter {
                source: span,
                parameter_name,
                parameter_value_sha256,
                ..
            } => (
                ComponentOrigin::Parameter,
                Some(source_cursor.bytes_for_span(&span)?),
                Some(parameter_name),
                Some(parameter_value_sha256),
            ),
            ProvenanceOrigin::GeneratedMetadata { .. } => {
                return Err(super::error("unexpected policy metadata inside component fragment"));
            }
        };
        let answer_ref = name
            .as_ref()
            .and_then(|name| bindings.iter().find(|item| item.parameter_name == *name))
            .and_then(|item| item.answer_ref.clone());
        cursor = output_span.end;
        if spans.len() >= max_spans {
            return Err(super::error(
                "component fragment exceeds remaining provenance span budget",
            ));
        }
        spans.push(ComponentSpan {
            output: output_span,
            kind,
            source: source_span,
            parameter_name: name,
            parameter_value_sha256: digest,
            answer_ref,
        });
    }
    if cursor < output.len() {
        append_newline(&mut spans, output, cursor, output.len(), max_spans)?;
    }
    if spans.len() > MAX_GRAPH_RECORDS {
        return Err(super::error("component fragment exceeds 100000 provenance spans"));
    }
    Ok(ComponentFragment { markdown: rendered.markdown, spans })
}

fn append_newline(
    spans: &mut Vec<ComponentSpan>,
    text: &str,
    start: usize,
    end: usize,
    max_spans: usize,
) -> Result<(), ForgeError> {
    if !text[start..end].bytes().all(|byte| byte == b'\n') {
        return Err(super::error("component rendering omitted provenance for content"));
    }
    if spans.len() >= max_spans {
        return Err(super::error("component fragment exceeds remaining provenance span budget"));
    }
    spans.push(ComponentSpan {
        output: ByteSpan { start, end },
        kind: ComponentOrigin::GeneratedNewline,
        source: None,
        parameter_name: None,
        parameter_value_sha256: None,
        answer_ref: None,
    });
    Ok(())
}

/// PRD-059 emits monotonically ordered scalar-column spans. Advance each byte
/// at most once instead of rescanning an entire long line for each occurrence.
struct SpanCursor<'a> {
    text: &'a str,
    byte: usize,
    line: usize,
    column: usize,
}

impl<'a> SpanCursor<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, byte: 0, line: 1, column: 1 }
    }

    fn bytes_for_span(&mut self, span: &TextSpan) -> Result<ByteSpan, ForgeError> {
        if span.start_column > span.end_column {
            return Err(super::error("inverted component provenance span"));
        }
        let start = self.advance(span.line, span.start_column)?;
        let end = self.advance(span.line, span.end_column)?;
        Ok(ByteSpan { start, end })
    }

    fn advance(&mut self, line: usize, column: usize) -> Result<usize, ForgeError> {
        if line < self.line || column == 0 || (line == self.line && column < self.column) {
            return Err(super::error(
                "component provenance positions must be ordered and one-based",
            ));
        }
        while self.line < line {
            let newline = self.text[self.byte..]
                .find('\n')
                .ok_or_else(|| super::error("invalid component provenance line"))?;
            self.byte += newline + 1;
            self.line += 1;
            self.column = 1;
        }
        while self.column < column {
            let rest = &self.text[self.byte..];
            let character = rest.chars().next().ok_or_else(|| {
                super::error("component provenance column exceeds its source line")
            })?;
            if character == '\n'
                || (character == '\r' && (rest.len() == 1 || rest.starts_with("\r\n")))
            {
                return Err(super::error("component provenance column exceeds its source line"));
            }
            self.byte += character.len_utf8();
            self.column += 1;
        }
        Ok(self.byte)
    }
}

type CaptureCache = BTreeMap<String, (String, Vec<u8>)>;

fn cached_read(
    cache: &mut CaptureCache,
    read: &mut impl FnMut(&str, &PinnedFile, u64) -> Result<Vec<u8>, ForgeError>,
    role: &str,
    pin: &PinnedFile,
    limit: u64,
) -> Result<Vec<u8>, ForgeError> {
    let path = label(&pin.path)?.to_owned();
    if let Some((digest, bytes)) = cache.get(&path) {
        if *digest != pin.expected_sha256 {
            return Err(super::error("repeated component source has conflicting exact pins"));
        }
        return Ok(bytes.clone());
    }
    let bytes = read(role, pin, limit)?;
    if bytes.len() as u64 > limit || sha256_hex(&bytes) != pin.expected_sha256 {
        return Err(super::error("component source violates its byte bound or exact pin"));
    }
    let total: usize = cache.values().map(|(_, bytes)| bytes.len()).sum();
    if u64::try_from(total.saturating_add(bytes.len()))
        .map_err(|_| super::error("component input byte count exceeds supported bounds"))?
        > manifest::MAX_TOTAL_BYTES
    {
        return Err(super::error("component captured inputs exceed 50 MiB"));
    }
    cache.insert(path, (pin.expected_sha256.clone(), bytes.clone()));
    Ok(bytes)
}

fn reject_null(value: &Value) -> Result<(), ForgeError> {
    match value {
        Value::Null => Err(super::error("null is not an authoring component value")),
        Value::Array(values) => values.iter().try_for_each(reject_null),
        Value::Object(values) => values.values().try_for_each(reject_null),
        _ => Ok(()),
    }
}

fn pin(pin: &PinnedFile, extension: &str) -> Result<(), ForgeError> {
    manifest::validate_local_path("component input", &pin.path)?;
    if pin.path.extension().and_then(|part| part.to_str()) != Some(extension) {
        return Err(super::error("component input has the wrong extension"));
    }
    sha(&pin.expected_sha256)
}

fn sha(value: &str) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256("component fingerprint", value)
        .map_err(super::error)
}

fn key(value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > 64
        || !value.as_bytes()[0].is_ascii_lowercase()
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(super::error("component keys must be bounded ASCII kebab-case"));
    }
    Ok(())
}

fn label(path: &Path) -> Result<&str, ForgeError> {
    path.to_str().ok_or_else(|| super::error("component paths must be portable UTF-8"))
}

fn timestamp(value: &str) -> Result<chrono::DateTime<chrono::FixedOffset>, ForgeError> {
    if value.len() > 64 {
        return Err(super::error("component timestamp exceeds 64 bytes"));
    }
    chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|_| super::error("component review time must be RFC3339"))
}

fn validate_parameter_text(value: &ParameterValue) -> Result<(), ForgeError> {
    let strings: Vec<&str> = match value {
        ParameterValue::String(value) => vec![value],
        ParameterValue::StringList(values) => {
            if values.len() > MAX_BINDINGS {
                return Err(super::error("component string-list exceeds 128 items"));
            }
            values.iter().map(String::as_str).collect()
        }
        ParameterValue::Integer(_) | ParameterValue::Boolean(_) => Vec::new(),
    };
    if strings.iter().any(|value| value.len() > manifest::MAX_STRING_BYTES
        || value.chars().any(|character| character.is_control()
            || matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))) {
        return Err(super::error("component values require bounded single-line text without control characters"));
    }
    Ok(())
}

fn allowed_sensitivity(sensitivity: Sensitivity) -> Result<(), ForgeError> {
    if matches!(sensitivity, Sensitivity::Public | Sensitivity::Internal) {
        Ok(())
    } else {
        Err(super::error(
            "confidential and restricted values cannot be rendered as component parameters",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applicability::model::{
        ApplicabilityReport, ClassificationCounts, ControlResult, GapClassification, ReportFilters,
    };
    use crate::authoring::manifest::{Answer, AnswerState};
    use crate::mapping::inventory::ResourceEvidence;
    use crate::mapping::manifest::ResourceType;
    use serde_json::json;

    struct Fixture {
        loaded: LoadedAuthorProject,
        extension: Value,
        sidecar: Value,
        source: Vec<u8>,
    }

    impl Fixture {
        fn new() -> Self {
            let mut loaded = LoadedAuthorProject {
                project: manifest::parse_project(include_bytes!(
                    "../../tests/fixtures/authoring/contracts/valid-project.json"
                ))
                .unwrap(),
                pack: manifest::parse_pack(include_bytes!(
                    "../../tests/fixtures/authoring/contracts/valid-pack.json"
                ))
                .unwrap(),
                baseline_report: ApplicabilityReport {
                    schema_version: crate::applicability::model::REPORT_SCHEMA_VERSION,
                    manifest_sha256: "c".repeat(64),
                    framework: ResourceEvidence {
                        resource_type: ResourceType::Catalog,
                        href: "framework.json".into(),
                        raw_sha256: "a".repeat(64),
                        root_uuid: "11111111-1111-4111-8111-111111111111".into(),
                        document_version: "1.0.0".into(),
                        oscal_version: "1.2.3".into(),
                        resolved_catalog_sha256: None,
                    },
                    mapping_collections: vec![],
                    reviewers: vec![],
                    counts: ClassificationCounts {
                        total: 1,
                        applicable_unmapped: 1,
                        ..Default::default()
                    },
                    filters: ReportFilters::default(),
                    matched_controls: 1,
                    controls: vec![ControlResult {
                        control_id: "sample-1".into(),
                        groups: vec![],
                        classification: GapClassification::ApplicableUnmapped,
                        reviewer_key: None,
                        reviewed_at: None,
                        rationale: None,
                        revisit_date: None,
                        note: None,
                        positive_mapping_count: 0,
                        no_relationship_count: 0,
                        policy_sources: vec![],
                    }],
                    review_queue: vec![],
                },
                inputs: vec![],
                project_sha256: "e".repeat(64),
                pack_sha256: "d".repeat(64),
                report_sha256: "b".repeat(64),
                clauses: BTreeMap::new(),
            };
            let question = &loaded.pack.questions[0];
            loaded.project.answers.push(Answer {
                key: "sample-answer".into(),
                question_key: question.key.clone(),
                question_sha256: manifest::question_sha256(question).unwrap(),
                authoring_pack_sha256: loaded.pack_sha256.clone(),
                owner: question.owner.clone(),
                source_label: "Synthetic interview".into(),
                sensitivity: question.sensitivity,
                review: loaded.project.baseline_review.clone(),
                expires_at: None,
                state: AnswerState::Provided,
                value: Some(json!("Fictional équipe 👩‍💻")),
            });
            let source = b"## Sample drafting section\n\nThe {{forge:param:owner-role}} records synthetic changes.\n".to_vec();
            let sidecar = json!({
                "schema_version":"forge.policy-component/1", "component_key":"sample-component", "version":"1.0.0",
                "title":"Sample drafting section", "owner":"sample-owner", "status":"approved", "source":"component.md",
                "expected_sha256":sha256_hex(&source), "parameters":[{"name":"owner-role","type":"string","required":true}]
            });
            let extension = json!({
                "schema_version":COMPONENTS_SCHEMA_VERSION, "project_sha256":loaded.project_sha256,
                "instances":[{
                    "instance_key":"first", "policy_key":"sample-policy", "topic_key":"sample-topic",
                    "gap_ids":[manifest::gap_id(&loaded.report_sha256,"sample-1")],
                    "component_manifest":{"path":"component.json","expected_sha256":sha256_hex(&serde_json::to_vec(&sidecar).unwrap())},
                    "source":{"path":"component.md","expected_sha256":sha256_hex(&source)},
                    "parameters":{"owner-role":{"kind":"answer","question_key":"sample-question","answer_key":"sample-answer","expected_sha256":manifest::answer_sha256(&loaded.project.answers[0]).unwrap()}},
                    "review":loaded.project.baseline_review
                }]
            });
            Self { loaded, extension, sidecar, source }
        }

        fn refresh(&mut self) {
            self.sidecar["expected_sha256"] = json!(sha256_hex(&self.source));
            self.extension["instances"][0]["source"]["expected_sha256"] =
                json!(sha256_hex(&self.source));
            self.extension["instances"][0]["component_manifest"]["expected_sha256"] =
                json!(sha256_hex(&serde_json::to_vec(&self.sidecar).unwrap()));
            if let Some(answer) = self.loaded.project.answers.first()
                && self.extension["instances"][0]["parameters"]["owner-role"]["kind"] == "answer"
            {
                self.extension["instances"][0]["parameters"]["owner-role"]["expected_sha256"] =
                    json!(manifest::answer_sha256(answer).unwrap());
            }
        }

        fn run(&self) -> Result<LoadedComponents, ForgeError> {
            let plan = super::super::plan::build_plan(&self.loaded)?;
            prepare(
                &serde_json::to_vec(&self.extension).unwrap(),
                &self.loaded,
                &plan,
                |_, pin, _| match pin.path.to_str().unwrap() {
                    "component.json" => Ok(serde_json::to_vec(&self.sidecar).unwrap()),
                    "component.md" => Ok(self.source.clone()),
                    _ => Err(super::super::error("unexpected fixture file")),
                },
            )
        }
    }

    #[test]
    fn components_closed_schema_rejects_forward_unknown_duplicate_null_and_graph_aliases() {
        let fixture = Fixture::new();
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/author-components.schema.json"))
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        assert!(validator.is_valid(&fixture.extension));
        assert!(parse(&serde_json::to_vec(&fixture.extension).unwrap()).is_ok());
        for (pointer, value) in [
            ("/schema_version", json!("forge.author-components/2")),
            ("/instances/0/review/extra", json!(true)),
            ("/instances/0/parameters/owner-role/extra", json!(true)),
            ("/instances/0/parameters/owner-role/expected_sha256", Value::Null),
            ("/instances/0/source/path", json!("../outside.md")),
            ("/instances/0/review/reviewed_at", json!("not-a-time")),
            ("/instances/0/review/rationale", json!("\u{200b}")),
            ("/instances/0/review/rationale", json!("Misleading \u{202e} label")),
        ] {
            let mut invalid = fixture.extension.clone();
            if let Some(parent) = pointer.strip_suffix("/extra") {
                invalid
                    .pointer_mut(parent)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .insert("extra".into(), value);
            } else {
                *invalid.pointer_mut(pointer).unwrap() = value;
            }
            assert!(!validator.is_valid(&invalid), "schema accepted {pointer}");
            assert!(
                parse(&serde_json::to_vec(&invalid).unwrap()).is_err(),
                "runtime accepted {pointer}"
            );
        }
        let duplicate = br#"{"schema_version":"forge.author-components/1","schema\u005fversion":"forge.author-components/1"}"#;
        assert!(parse(duplicate).unwrap_err().to_string().contains("duplicate"));
        let mut duplicate = fixture.extension.clone();
        duplicate["instances"]
            .as_array_mut()
            .unwrap()
            .push(fixture.extension["instances"][0].clone());
        assert!(parse(&serde_json::to_vec(&duplicate).unwrap()).is_err());
    }

    #[test]
    fn literal_binding_variant_rejects_unknown_fields_and_null_sensitivity() {
        let mut fixture = Fixture::new();
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/author-components.schema.json"))
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        fixture.extension["instances"][0]["parameters"]["owner-role"] =
            json!({"kind":"literal","value":"Explicit sample","sensitivity":"public"});
        assert!(validator.is_valid(&fixture.extension));
        assert!(parse(&serde_json::to_vec(&fixture.extension).unwrap()).is_ok());
        for (field, value) in [("extra", json!(true)), ("sensitivity", Value::Null)] {
            let mut invalid = fixture.extension.clone();
            invalid["instances"][0]["parameters"]["owner-role"][field] = value;
            assert!(!validator.is_valid(&invalid));
            assert!(parse(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
    }

    #[test]
    fn ordered_span_cursor_handles_long_unicode_lines_zero_width_and_crlf_exactly() {
        let body = "é🙂".repeat(20_000);
        let text = format!("{body}\r\nnext\n");
        let mut cursor = SpanCursor::new(&text);
        for index in 0..20_000 {
            let column = index * 2 + 1;
            let empty = cursor
                .bytes_for_span(&TextSpan { line: 1, start_column: column, end_column: column })
                .unwrap();
            assert_eq!(empty, ByteSpan { start: index * 6, end: index * 6 });
            let span = cursor
                .bytes_for_span(&TextSpan { line: 1, start_column: column, end_column: column + 2 })
                .unwrap();
            assert_eq!(&text[span.start..span.end], "é🙂");
        }
        let next =
            cursor.bytes_for_span(&TextSpan { line: 2, start_column: 1, end_column: 5 }).unwrap();
        assert_eq!(next, ByteSpan { start: body.len() + 2, end: body.len() + 6 });
        assert!(
            cursor.bytes_for_span(&TextSpan { line: 2, start_column: 1, end_column: 2 }).is_err()
        );
        assert!(
            cursor.bytes_for_span(&TextSpan { line: 3, start_column: 1, end_column: 2 }).is_err()
        );
    }

    #[test]
    fn component_defaults_never_satisfy_missing_explicit_context() {
        let mut fixture = Fixture::new();
        fixture.sidecar["parameters"][0].as_object_mut().unwrap().remove("required");
        fixture.sidecar["parameters"][0]["default"] = json!("Invisible default");
        fixture.extension["instances"][0]["parameters"] = json!({});
        fixture.refresh();
        assert!(fixture.run().unwrap_err().to_string().contains("explicit binding"));
        fixture.extension["instances"][0]["parameters"]["owner-role"] =
            json!({"kind":"literal","value":"Consciously selected","sensitivity":"public"});
        let loaded = fixture.run().unwrap();
        let markdown =
            std::str::from_utf8(&loaded.instances["first"].fragment.as_ref().unwrap().markdown)
                .unwrap();
        assert!(markdown.contains("Consciously selected"));
        assert!(!markdown.contains("Invisible default"));
    }

    #[test]
    fn missing_stale_invalid_expired_and_no_answer_block_only_the_selected_section() {
        for expected in [
            AnswerStatus::Missing,
            AnswerStatus::Stale,
            AnswerStatus::Invalid,
            AnswerStatus::Expired,
            AnswerStatus::NoAnswer,
        ] {
            let mut fixture = Fixture::new();
            // Optional questions become mandatory for a selected parameter dependency.
            fixture.loaded.pack.questions[0].required = false;
            fixture.loaded.project.answers[0].question_sha256 =
                manifest::question_sha256(&fixture.loaded.pack.questions[0]).unwrap();
            match expected {
                AnswerStatus::Missing => fixture.loaded.project.answers.clear(),
                AnswerStatus::Stale => {
                    fixture.loaded.project.answers[0].question_sha256 = "0".repeat(64);
                }
                AnswerStatus::Invalid => fixture.loaded.project.answers[0].value = Some(json!(42)),
                AnswerStatus::Expired => {
                    fixture.loaded.project.answers[0].expires_at =
                        Some("2026-09-07T00:00:00Z".into());
                }
                AnswerStatus::NoAnswer => {
                    fixture.loaded.project.answers[0].state = AnswerState::NoAnswer;
                    fixture.loaded.project.answers[0].value = None;
                }
                AnswerStatus::Available => unreachable!(),
            }
            fixture.refresh();
            let components = fixture.run().unwrap();
            let instance = &components.instances["first"];
            assert!(instance.fragment.is_none());
            assert_eq!(instance.evidence.bindings[0].answer_state, Some(expected));
            let mut plan = super::super::plan::build_plan(&fixture.loaded).unwrap();
            let mut unrelated = plan.policies[0].sections[0].clone();
            unrelated.topic_key = "unrelated".into();
            unrelated.state = DraftState::SkeletonReady;
            plan.policies[0].sections.push(unrelated);
            components.apply_plan(&mut plan);
            assert_eq!(plan.policies[0].sections[0].state, DraftState::BlockedContext);
            assert_eq!(plan.policies[0].sections[1].state, DraftState::SkeletonReady);
        }
    }

    #[test]
    fn component_parameter_constraints_block_invalid_answers_without_literal_fallback() {
        let mut fixture = Fixture::new();
        fixture.sidecar["parameters"][0]["constraints"] = json!({"max_length":2});
        fixture.refresh();
        let result = fixture.run().unwrap();
        assert!(result.instances["first"].fragment.is_none());
        assert_eq!(
            result.instances["first"].evidence.bindings[0].answer_state,
            Some(AnswerStatus::Invalid)
        );
    }

    #[test]
    fn topic_heading_uses_phase1_punctuation_escaping_exactly() {
        let mut fixture = Fixture::new();
        fixture.loaded.pack.topics[0].title = "Sample: [draft].".into();
        fixture.source =
            b"## Sample\\: \\[draft\\]\\.\n\nThe {{forge:param:owner-role}} records changes.\n"
                .to_vec();
        fixture.refresh();
        assert!(fixture.run().unwrap().instances["first"].fragment.is_some());
        fixture.source =
            b"## Sample: \\[draft\\]\\.\n\nThe {{forge:param:owner-role}} records changes.\n"
                .to_vec();
        fixture.refresh();
        assert!(fixture.run().unwrap_err().to_string().contains("exactly match"));
    }

    #[test]
    fn component_source_and_sidecar_drift_fail_even_when_context_is_blocked() {
        let mut fixture = Fixture::new();
        fixture.loaded.project.answers.clear();
        fixture.source.push(b' ');
        assert!(fixture.run().unwrap_err().to_string().contains("exact pin"));
        let mut fixture = Fixture::new();
        fixture.sidecar["owner"] = json!("changed-owner");
        assert!(fixture.run().unwrap_err().to_string().contains("exact pin"));
    }

    #[test]
    fn changed_answer_pin_blocks_only_dependent_section_and_preserves_both_hashes() {
        let mut fixture = Fixture::new();
        fixture.loaded.project.answers[0].value = Some(json!("Changed value"));
        let components = fixture.run().unwrap();
        let instance = &components.instances["first"];
        assert!(instance.fragment.is_none());
        assert!(instance.evidence.blocked);
        let binding = &instance.evidence.bindings[0];
        assert_eq!(binding.answer_state, Some(AnswerStatus::Stale));
        assert_eq!(
            binding.observed_answer_sha256,
            Some(manifest::answer_sha256(&fixture.loaded.project.answers[0]).unwrap())
        );
        assert_ne!(
            binding.observed_answer_sha256.as_ref(),
            Some(&binding.answer_ref.as_ref().unwrap().expected_sha256)
        );
        assert!(binding.value_sha256.is_none());
        assert!(!serde_json::to_string(&instance.evidence).unwrap().contains("Changed value"));
        let mut plan = super::super::plan::build_plan(&fixture.loaded).unwrap();
        let mut unrelated = plan.policies[0].sections[0].clone();
        unrelated.topic_key = "unrelated".into();
        unrelated.state = DraftState::SkeletonReady;
        plan.policies[0].sections.push(unrelated);
        components.apply_plan(&mut plan);
        assert_eq!(plan.policies[0].sections[0].state, DraftState::BlockedContext);
        assert_eq!(plan.policies[0].sections[1].state, DraftState::SkeletonReady);
    }

    #[test]
    fn answer_key_bound_to_the_wrong_question_is_an_invalid_contract() {
        let mut fixture = Fixture::new();
        let mut other = fixture.loaded.pack.questions[0].clone();
        other.key = "other-question".into();
        other.required = false;
        fixture.loaded.pack.questions.push(other);
        fixture.loaded.pack.topics[0].question_keys.push("other-question".into());
        fixture.extension["instances"][0]["parameters"]["owner-role"]["question_key"] =
            json!("other-question");
        assert!(fixture.run().unwrap_err().to_string().contains("wrong question"));
    }

    #[test]
    fn protected_values_and_secret_names_fail_before_rendering() {
        for sensitivity in [Sensitivity::Confidential, Sensitivity::Restricted] {
            let mut fixture = Fixture::new();
            fixture.loaded.pack.questions[0].sensitivity = sensitivity;
            fixture.loaded.project.answers[0].sensitivity = sensitivity;
            fixture.refresh();
            assert!(fixture.run().unwrap_err().to_string().contains("cannot be rendered"));
            fixture.extension["instances"][0]["parameters"]["owner-role"] =
                json!({"kind":"literal","value":"private","sensitivity":sensitivity});
            assert!(fixture.run().is_err());
        }
        let mut fixture = Fixture::new();
        fixture.extension["instances"][0]["parameters"] =
            json!({"api-token":{"kind":"literal","value":"forbidden","sensitivity":"public"}});
        assert!(fixture.run().unwrap_err().to_string().contains("secret-like"));
    }

    #[test]
    fn component_sources_reject_unsafe_markdown_unknown_grammar_and_outline_repair() {
        for source in [
            "## Other title\n\n{{forge:param:owner-role}}",
            "## Sample drafting section\n\n## Extra title\n{{forge:param:owner-role}}",
            "## Sample drafting section\n\n<script>{{forge:param:owner-role}}</script>",
            "## Sample drafting section\n\n`{{forge:param:owner-role}}`",
            "## Sample drafting section\n\n[link](https://example.test/{{forge:param:owner-role}})",
            "## Sample drafting section\n\n[local](file:///private/data)\n{{forge:param:owner-role}}",
            "## Sample drafting section\n\n[x]: https://example.test\n{{forge:param:owner-role}}",
            "## Sample drafting section\n\n{{forge:include:other}}",
            "## Sample drafting section\n\n{{forge:param:undeclared}}",
            "## Sample drafting section\n\n```\n{{forge:param:owner-role}}\n```",
        ] {
            let mut fixture = Fixture::new();
            fixture.loaded.project.answers.clear();
            fixture.source = source.as_bytes().to_vec();
            fixture.refresh();
            assert!(fixture.run().is_err(), "accepted {source}");
        }
    }

    #[test]
    fn fragment_provenance_partitions_utf8_and_retains_exact_source_and_answer_pins() {
        let mut fixture = Fixture::new();
        fixture.source = "## Sample drafting section\r\n\r\nRésumé {{forge:param:owner-role}}.\r\n"
            .as_bytes()
            .to_vec();
        fixture.refresh();
        let result = fixture.run().unwrap();
        let instance = &result.instances["first"];
        let fragment = instance.fragment.as_ref().unwrap();
        let mut cursor = 0usize;
        for span in &fragment.spans {
            assert_eq!(span.output.start, cursor);
            let bytes = &fragment.markdown[span.output.start..span.output.end];
            assert!(std::str::from_utf8(bytes).is_ok());
            cursor = span.output.end;
            match span.kind {
                ComponentOrigin::ComponentSource => {
                    let source = span.source.as_ref().unwrap();
                    assert_eq!(bytes, &fixture.source[source.start..source.end]);
                }
                ComponentOrigin::Parameter => {
                    let source = span.source.as_ref().unwrap();
                    assert_eq!(
                        &fixture.source[source.start..source.end],
                        b"{{forge:param:owner-role}}"
                    );
                    assert_eq!(
                        span.answer_ref.as_ref().unwrap().expected_sha256,
                        manifest::answer_sha256(&fixture.loaded.project.answers[0]).unwrap()
                    );
                    assert!(span.parameter_value_sha256.is_some());
                }
                ComponentOrigin::GeneratedNewline => {
                    assert!(bytes.iter().all(|byte| *byte == b'\n'));
                }
            }
        }
        assert_eq!(cursor, fragment.markdown.len());
        let report = serde_json::to_string(&instance.evidence).unwrap();
        assert!(!report.contains("Fictional équipe"));
        assert!(!report.contains("Résumé"));
        let second = fixture.run().unwrap();
        assert_eq!(
            fragment.markdown,
            second.instances["first"].fragment.as_ref().unwrap().markdown
        );
        assert_eq!(report, serde_json::to_string(&second.instances["first"].evidence).unwrap());
    }

    #[test]
    fn one_pass_substitution_retains_literal_placeholder_text_and_empty_value_provenance() {
        for value in ["{{forge:param:other}}", ""] {
            let mut fixture = Fixture::new();
            fixture.extension["instances"][0]["parameters"]["owner-role"] =
                json!({"kind":"literal","value":value,"sensitivity":"public"});
            let result = fixture.run().unwrap();
            let fragment = result.instances["first"].fragment.as_ref().unwrap();
            let span = fragment
                .spans
                .iter()
                .find(|span| matches!(span.kind, ComponentOrigin::Parameter))
                .unwrap();
            assert_eq!(
                &fragment.markdown[span.output.start..span.output.end],
                crate::policy::render::escape_markdown(value).as_bytes()
            );
            assert!(span.parameter_value_sha256.is_some());
        }
    }

    #[test]
    fn repeated_instances_reuse_exact_capture_and_preserve_separate_identity() {
        let mut fixture = Fixture::new();
        let mut second_policy = fixture.loaded.project.policies[0].clone();
        second_policy.key = "second-policy".into();
        second_policy.policy_family_key = "second-family".into();
        fixture.loaded.project.policies.push(second_policy);
        let mut family = fixture.loaded.pack.policy_families[0].clone();
        family.key = "second-family".into();
        fixture.loaded.pack.policy_families.push(family);
        let mut assignment = fixture.loaded.pack.family_assignments[0].clone();
        assignment.key = "second-assignment".into();
        assignment.policy_family_key = "second-family".into();
        fixture.loaded.pack.family_assignments.push(assignment);
        let mut second = fixture.extension["instances"][0].clone();
        second["instance_key"] = json!("second");
        second["policy_key"] = json!("second-policy");
        fixture.extension["instances"].as_array_mut().unwrap().push(second);
        let plan = super::super::plan::build_plan(&fixture.loaded).unwrap();
        let mut reads = 0usize;
        let result = prepare(
            &serde_json::to_vec(&fixture.extension).unwrap(),
            &fixture.loaded,
            &plan,
            |_, pin, _| {
                reads += 1;
                if pin.path.extension().unwrap() == "json" {
                    Ok(serde_json::to_vec(&fixture.sidecar).unwrap())
                } else {
                    Ok(fixture.source.clone())
                }
            },
        )
        .unwrap();
        assert_eq!(reads, 2);
        assert_eq!(result.instances.len(), 2);
        assert_ne!(
            result.instances["first"].evidence.instance_key,
            result.instances["second"].evidence.instance_key
        );
        assert_eq!(
            result.instances["first"].fragment.as_ref().unwrap().markdown,
            result.instances["second"].fragment.as_ref().unwrap().markdown
        );
    }

    #[test]
    fn component_paths_reject_nonportable_alias_spellings_and_binding_bounds() {
        for path in [
            "../component.md",
            "component.md/",
            "sub//component.md",
            "sub/../component.md",
            "CON.md",
            "dir./component.md",
            "C:\\component.md",
        ] {
            let mut fixture = Fixture::new();
            fixture.extension["instances"][0]["source"]["path"] = json!(path);
            assert!(fixture.run().is_err(), "accepted {path}");
        }
        let mut fixture = Fixture::new();
        fixture.extension["instances"][0]["parameters"]["owner-role"] =
            json!({"kind":"literal","value":vec!["item";129],"sensitivity":"public"});
        assert!(fixture.run().unwrap_err().to_string().contains("128 items"));
    }

    #[test]
    fn generated_newline_provenance_obeys_remaining_budget_before_push() {
        let fixture = Fixture::new();
        let contract = parse(&serde_json::to_vec(&fixture.extension).unwrap()).unwrap();
        let manifest = crate::policy::manifest::parse_component(
            &serde_json::to_vec(&fixture.sidecar).unwrap(),
        )
        .unwrap();
        let (parameters, bindings, _) =
            bind_parameters(&contract.instances[0], &manifest, &fixture.loaded).unwrap();
        let component = crate::policy::LoadedComponent {
            instance: ComponentInstance {
                instance_key: "first".into(),
                component_manifest: "component.json".into(),
                parameters,
            },
            manifest,
            manifest_sha256: contract.instances[0].component_manifest.expected_sha256.clone(),
            source_sha256: sha256_hex(&fixture.source),
            source_bytes: fixture.source,
            source_label: "component.md".into(),
        };
        let failure = convert_fragment(&component, &bindings, 4096, 4).unwrap_err();
        assert!(failure.to_string().contains("remaining provenance span budget"));
        let mut spans = Vec::new();
        assert!(append_newline(&mut spans, "\n", 0, 1, 0).is_err());
        assert!(spans.is_empty());
    }
}

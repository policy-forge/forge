//! One-pass Markdown component rendering with span-level provenance.

use std::collections::{BTreeMap, BTreeSet};

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::manifest::{ComponentInstance, ComponentManifest, ParameterDeclaration, ParameterValue};
use super::{LoadedComponent, composition_error};

const MAX_ASSEMBLED_BYTES: usize = 50 * 1024 * 1024;
const MAX_PROVENANCE_SPANS: usize = 100_000;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CompositionLock {
    pub schema_version: &'static str,
    pub composition_manifest_sha256: String,
    pub policy_key: String,
    pub version: String,
    pub components: Vec<LockedComponent>,
    pub output_sha256: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LockedComponent {
    pub instance_key: String,
    pub component_key: String,
    pub version: String,
    pub component_manifest_sha256: String,
    pub source_sha256: String,
    pub parameter_value_sha256: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProvenanceMap {
    pub schema_version: String,
    pub composition_manifest_sha256: String,
    pub output_sha256: String,
    pub spans: Vec<ProvenanceSpan>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProvenanceSpan {
    pub output: TextSpan,
    pub origin: ProvenanceOrigin,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TextSpan {
    pub line: usize,
    pub start_column: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ProvenanceOrigin {
    GeneratedMetadata {
        field: String,
    },
    Component {
        component_file: String,
        source_sha256: String,
        source: TextSpan,
        instance_key: String,
    },
    Parameter {
        component_file: String,
        source_sha256: String,
        source: TextSpan,
        instance_key: String,
        parameter_name: String,
        parameter_value_sha256: String,
    },
}

pub struct RenderedComposition {
    pub markdown: Vec<u8>,
    pub lock: Vec<u8>,
    pub provenance: Vec<u8>,
}

struct RenderState {
    lines: Vec<String>,
    spans: Vec<ProvenanceSpan>,
    bytes: usize,
}

impl RenderState {
    fn new(title: &str) -> Self {
        let line = format!("# {}", escape_markdown(title));
        let end_column = line.chars().count() + 1;
        Self {
            bytes: line.len() + 2,
            lines: vec![line, String::new()],
            spans: vec![ProvenanceSpan {
                output: TextSpan { line: 1, start_column: 1, end_column },
                origin: ProvenanceOrigin::GeneratedMetadata { field: "policy-title".to_string() },
            }],
        }
    }

    fn next_line(&self) -> usize {
        self.lines.len() + 1
    }

    fn remaining_line_bytes(&self) -> usize {
        MAX_ASSEMBLED_BYTES.saturating_sub(self.bytes).saturating_sub(1)
    }

    fn push_line(&mut self, line: String) -> Result<(), crate::ForgeError> {
        let next = self
            .bytes
            .checked_add(line.len())
            .and_then(|bytes| bytes.checked_add(1))
            .ok_or_else(|| composition_error("assembled Markdown byte count overflowed"))?;
        if next > MAX_ASSEMBLED_BYTES {
            return Err(composition_error(format!(
                "assembled Markdown exceeds the {MAX_ASSEMBLED_BYTES} byte limit"
            )));
        }
        self.bytes = next;
        self.lines.push(line);
        Ok(())
    }

    fn pop_empty_line(&mut self) {
        if self.lines.last().is_some_and(String::is_empty) {
            let _ = self.lines.pop();
            self.bytes = self.bytes.saturating_sub(1);
        }
    }
}

pub(crate) fn render(
    policy_key: &str,
    title: &str,
    version: &str,
    manifest_sha256: &str,
    components: &[LoadedComponent],
) -> Result<RenderedComposition, crate::ForgeError> {
    let mut state = RenderState::new(title);
    let mut locked = Vec::with_capacity(components.len());
    for (index, component) in components.iter().enumerate() {
        let parameter_values = resolve_values(&component.manifest, &component.instance)?;
        let used = render_component(&mut state, component, &parameter_values)?;
        for supplied in component.instance.parameters.keys() {
            if !used.contains(supplied) {
                return Err(composition_error(format!(
                    "instance '{}' supplies unused parameter '{supplied}'",
                    component.instance.instance_key
                )));
            }
        }
        let parameter_value_sha256 = parameter_values
            .iter()
            .map(|(name, value)| Ok((name.clone(), hash_json(value)?)))
            .collect::<Result<BTreeMap<_, _>, crate::ForgeError>>()?;
        locked.push(LockedComponent {
            instance_key: component.instance.instance_key.clone(),
            component_key: component.manifest.component_key.clone(),
            version: component.manifest.version.clone(),
            component_manifest_sha256: component.manifest_sha256.clone(),
            source_sha256: component.source_sha256.clone(),
            parameter_value_sha256,
        });
        if index + 1 < components.len() {
            state.push_line(String::new())?;
        }
    }

    let markdown = format!("{}\n", state.lines.join("\n")).into_bytes();
    let output_sha256 = sha256(&markdown);
    let lock = CompositionLock {
        schema_version: "forge.policy-composition-lock/1",
        composition_manifest_sha256: manifest_sha256.to_string(),
        policy_key: policy_key.to_string(),
        version: version.to_string(),
        components: locked,
        output_sha256: output_sha256.clone(),
    };
    let provenance = ProvenanceMap {
        schema_version: "forge.policy-composition-provenance/1".to_string(),
        composition_manifest_sha256: manifest_sha256.to_string(),
        output_sha256,
        spans: state.spans,
    };
    Ok(RenderedComposition {
        markdown,
        lock: pretty_json(&lock)?,
        provenance: pretty_json(&provenance)?,
    })
}

fn render_component(
    state: &mut RenderState,
    component: &LoadedComponent,
    parameter_values: &BTreeMap<String, ParameterValue>,
) -> Result<BTreeSet<String>, crate::ForgeError> {
    let text = std::str::from_utf8(&component.source_bytes).map_err(|_| {
        composition_error(format!(
            "component source '{}' is not valid UTF-8",
            component.source_label
        ))
    })?;
    validate_heading_structure(text, &component.source_label)?;
    let declarations: BTreeMap<_, _> =
        component.manifest.parameters.iter().map(|item| (item.name.as_str(), item)).collect();
    let mut used = BTreeSet::new();
    let mut fence: Option<char> = None;
    for (line_index, raw_line) in text.split('\n').enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let source_line = line_index + 1;
        let output_line = state.next_line();
        let remaining_line_bytes = state.remaining_line_bytes();
        let rendered = render_line(
            line,
            source_line,
            output_line,
            component,
            &declarations,
            parameter_values,
            &mut used,
            &mut state.spans,
            &mut fence,
            remaining_line_bytes,
        )?;
        state.push_line(rendered)?;
    }
    while state.lines.last().is_some_and(String::is_empty) {
        state.pop_empty_line();
    }
    Ok(used)
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Span construction is one ordered rendering transaction.
fn render_line(
    line: &str,
    source_line: usize,
    output_line: usize,
    component: &LoadedComponent,
    declarations: &BTreeMap<&str, &ParameterDeclaration>,
    values: &BTreeMap<String, ParameterValue>,
    used: &mut BTreeSet<String>,
    spans: &mut Vec<ProvenanceSpan>,
    fence: &mut Option<char>,
    max_line_bytes: usize,
) -> Result<String, crate::ForgeError> {
    let trimmed = line.trim_start_matches(' ');
    let fence_marker = if trimmed.starts_with("```") {
        Some('`')
    } else if trimmed.starts_with("~~~") {
        Some('~')
    } else {
        None
    };
    let inside_fence = fence.is_some() || fence_marker.is_some();
    let mut occurrences = placeholder_occurrences(line)?;
    if inside_fence && !occurrences.is_empty() {
        return Err(context_error(component, source_line, "fenced code block"));
    }
    if let Some(marker) = fence_marker {
        if *fence == Some(marker) {
            *fence = None;
        } else if fence.is_none() {
            *fence = Some(marker);
        }
    }
    if occurrences.is_empty() {
        if line.len() > max_line_bytes {
            return Err(composition_error(format!(
                "assembled Markdown exceeds the {MAX_ASSEMBLED_BYTES} byte limit"
            )));
        }
        if !line.is_empty() {
            push_span(
                spans,
                component_span(
                    component,
                    output_line,
                    1,
                    line.chars().count() + 1,
                    source_line,
                    1,
                    line.chars().count() + 1,
                ),
            )?;
        }
        return Ok(line.to_string());
    }
    if heading_level(line).is_some() {
        return Err(context_error(component, source_line, "heading"));
    }
    if line.contains('<') || line.contains('>') {
        return Err(context_error(component, source_line, "raw HTML or angle-bracket text"));
    }
    for occurrence in &occurrences {
        if is_inline_code(line, occurrence.start) {
            return Err(context_error(component, source_line, "inline code"));
        }
        if is_link_destination(line, occurrence.start) {
            return Err(context_error(component, source_line, "URL or link destination"));
        }
    }

    let mut rendered = String::new();
    let mut previous = 0;
    let mut output_column = 1;
    for occurrence in occurrences.drain(..) {
        let literal = &line[previous..occurrence.start];
        if !literal.is_empty() {
            let length = literal.chars().count();
            push_bounded(&mut rendered, literal, max_line_bytes)?;
            push_span(
                spans,
                component_span(
                    component,
                    output_line,
                    output_column,
                    output_column + length,
                    source_line,
                    byte_to_column(line, previous),
                    byte_to_column(line, occurrence.start),
                ),
            )?;
            output_column += length;
        }
        let declaration = declarations.get(occurrence.name.as_str()).ok_or_else(|| {
            composition_error(format!(
                "component '{}' line {source_line} uses undeclared parameter '{}'",
                component.source_label, occurrence.name
            ))
        })?;
        let value = values.get(&occurrence.name).ok_or_else(|| {
            composition_error(format!(
                "instance '{}' has no value or default for placeholder '{}'",
                component.instance.instance_key, occurrence.name
            ))
        })?;
        super::manifest::validate_value(
            &format!(
                "instance '{}'.parameters.{}",
                component.instance.instance_key, occurrence.name
            ),
            declaration,
            value,
        )?;
        let substituted = render_parameter(value);
        let length = substituted.chars().count();
        push_bounded(&mut rendered, &substituted, max_line_bytes)?;
        if length > 0 {
            push_span(
                spans,
                ProvenanceSpan {
                    output: TextSpan {
                        line: output_line,
                        start_column: output_column,
                        end_column: output_column + length,
                    },
                    origin: ProvenanceOrigin::Parameter {
                        component_file: component.source_label.clone(),
                        source_sha256: component.source_sha256.clone(),
                        source: TextSpan {
                            line: source_line,
                            start_column: byte_to_column(line, occurrence.start),
                            end_column: byte_to_column(line, occurrence.end),
                        },
                        instance_key: component.instance.instance_key.clone(),
                        parameter_name: occurrence.name.clone(),
                        parameter_value_sha256: hash_json(value)?,
                    },
                },
            )?;
        }
        output_column += length;
        used.insert(occurrence.name);
        previous = occurrence.end;
    }
    let literal = &line[previous..];
    if !literal.is_empty() {
        let length = literal.chars().count();
        push_bounded(&mut rendered, literal, max_line_bytes)?;
        push_span(
            spans,
            component_span(
                component,
                output_line,
                output_column,
                output_column + length,
                source_line,
                byte_to_column(line, previous),
                line.chars().count() + 1,
            ),
        )?;
    }
    if rendered.contains("{{forge:") {
        return Err(composition_error(format!(
            "component '{}' line {source_line} contains unresolved reserved placeholder syntax",
            component.source_label
        )));
    }
    Ok(rendered)
}

fn push_bounded(
    rendered: &mut String,
    value: &str,
    max_line_bytes: usize,
) -> Result<(), crate::ForgeError> {
    if rendered.len().saturating_add(value.len()) > max_line_bytes {
        return Err(composition_error(format!(
            "assembled Markdown exceeds the {MAX_ASSEMBLED_BYTES} byte limit"
        )));
    }
    rendered.push_str(value);
    Ok(())
}

fn push_span(
    spans: &mut Vec<ProvenanceSpan>,
    span: ProvenanceSpan,
) -> Result<(), crate::ForgeError> {
    if spans.len() >= MAX_PROVENANCE_SPANS {
        return Err(composition_error(format!(
            "composition provenance exceeds the {MAX_PROVENANCE_SPANS} span limit"
        )));
    }
    spans.push(span);
    Ok(())
}

struct Occurrence {
    start: usize,
    end: usize,
    name: String,
}

fn placeholder_occurrences(line: &str) -> Result<Vec<Occurrence>, crate::ForgeError> {
    const PREFIX: &str = "{{forge:";
    const PARAM_PREFIX: &str = "{{forge:param:";
    let mut cursor = 0;
    let mut occurrences = Vec::new();
    while let Some(relative) = line[cursor..].find(PREFIX) {
        let start = cursor + relative;
        if !line[start..].starts_with(PARAM_PREFIX) {
            return Err(composition_error(
                "reserved {{forge: syntax supports only param placeholders",
            ));
        }
        let name_start = start + PARAM_PREFIX.len();
        let close = line[name_start..]
            .find("}}")
            .ok_or_else(|| composition_error("unterminated policy component placeholder"))?;
        let end = name_start + close + 2;
        let name = &line[name_start..name_start + close];
        if !valid_parameter_name(name) {
            return Err(composition_error(format!(
                "placeholder parameter name '{}' must be ASCII kebab-case",
                crate::json_strict::bounded(name)
            )));
        }
        occurrences.push(Occurrence { start, end, name: name.to_string() });
        cursor = end;
    }
    Ok(occurrences)
}

/// Validate component Markdown structure and placeholder contexts without substituting values.
///
/// # Errors
///
/// Returns [`crate::ForgeError::PolicyComposition`] for invalid UTF-8, headings, placeholder
/// grammar, declarations, or unsupported Markdown contexts.
pub fn validate_static_component(
    manifest: &ComponentManifest,
    source_label: &str,
    bytes: &[u8],
) -> Result<(), crate::ForgeError> {
    let text = std::str::from_utf8(bytes).map_err(|_| {
        composition_error(format!("component source '{source_label}' is not valid UTF-8"))
    })?;
    validate_heading_structure(text, source_label)?;
    let declared: BTreeSet<_> = manifest.parameters.iter().map(|item| item.name.as_str()).collect();
    let mut fence = None;
    for (index, raw_line) in text.split('\n').enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let occurrences = placeholder_occurrences(line)?;
        let trimmed = line.trim_start_matches(' ');
        let marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        if (fence.is_some() || marker.is_some()) && !occurrences.is_empty() {
            return Err(composition_error(format!(
                "component '{source_label}' line {} contains a placeholder in a fenced code block",
                index + 1
            )));
        }
        if let Some(marker) = marker {
            if fence == Some(marker) {
                fence = None;
            } else if fence.is_none() {
                fence = Some(marker);
            }
        }
        if !occurrences.is_empty() && heading_level(line).is_some() {
            return Err(composition_error(format!(
                "component '{source_label}' line {} contains a placeholder in a heading",
                index + 1
            )));
        }
        for occurrence in occurrences {
            if !declared.contains(occurrence.name.as_str()) {
                return Err(composition_error(format!(
                    "component '{source_label}' line {} uses undeclared parameter '{}'",
                    index + 1,
                    occurrence.name
                )));
            }
            if is_inline_code(line, occurrence.start)
                || is_link_destination(line, occurrence.start)
                || line.contains('<')
                || line.contains('>')
            {
                return Err(composition_error(format!(
                    "component '{source_label}' line {} uses a placeholder in an unsupported Markdown context",
                    index + 1
                )));
            }
        }
    }
    if fence.is_some() {
        return Err(composition_error(format!(
            "component '{source_label}' contains an unterminated fenced code block"
        )));
    }
    Ok(())
}

fn validate_heading_structure(text: &str, label: &str) -> Result<(), crate::ForgeError> {
    let mut lines = text.lines();
    let first =
        lines.next().ok_or_else(|| composition_error(format!("component '{label}' is empty")))?;
    if heading_level(first) != Some(2) {
        return Err(composition_error(format!(
            "component '{label}' must begin with exactly one level-two heading"
        )));
    }
    let first_title = first.trim_start().trim_start_matches('#').trim();
    if first_title.is_empty() {
        return Err(composition_error(format!(
            "component '{label}' must begin with a non-empty level-two heading"
        )));
    }
    let mut parsed_headings = 0usize;
    for event in Parser::new(text) {
        if let Event::Start(Tag::Heading { level, .. }) = event {
            parsed_headings += 1;
            if level == HeadingLevel::H1 || (level == HeadingLevel::H2 && parsed_headings > 1) {
                return Err(composition_error(format!(
                    "component '{label}' contains a reserved or repeated top-level heading"
                )));
            }
        }
    }
    for (index, line) in lines.enumerate() {
        if let Some(level) = heading_level(line) {
            if level == 1 {
                return Err(composition_error(format!(
                    "component '{label}' line {} contains a reserved level-one heading",
                    index + 2
                )));
            }
        }
        let trimmed = line.trim();
        if (trimmed.len() >= 3 && trimmed.bytes().all(|byte| byte == b'='))
            || trimmed.to_ascii_lowercase().starts_with("<h1")
            || trimmed.to_ascii_lowercase().starts_with("</h1")
        {
            return Err(composition_error(format!(
                "component '{label}' line {} contains a reserved level-one heading",
                index + 2
            )));
        }
    }
    Ok(())
}

fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let count = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6)
        .contains(&count)
        .then(|| trimmed.as_bytes().get(count).copied())
        .flatten()
        .filter(u8::is_ascii_whitespace)
        .map(|_| count)
}

fn resolve_values(
    manifest: &ComponentManifest,
    instance: &ComponentInstance,
) -> Result<BTreeMap<String, ParameterValue>, crate::ForgeError> {
    let declarations: BTreeMap<_, _> =
        manifest.parameters.iter().map(|item| (item.name.as_str(), item)).collect();
    for name in instance.parameters.keys() {
        if !declarations.contains_key(name.as_str()) {
            return Err(composition_error(format!(
                "instance '{}' supplies unknown parameter '{name}'",
                instance.instance_key
            )));
        }
    }
    let mut values = BTreeMap::new();
    for declaration in &manifest.parameters {
        let supplied = instance.parameters.get(&declaration.name);
        let value = supplied.or(declaration.default.as_ref());
        if declaration.required && value.is_none() {
            return Err(composition_error(format!(
                "instance '{}' is missing required parameter '{}'",
                instance.instance_key, declaration.name
            )));
        }
        if let Some(value) = value {
            super::manifest::validate_value(
                &format!("instance '{}'.parameters.{}", instance.instance_key, declaration.name),
                declaration,
                value,
            )?;
            values.insert(declaration.name.clone(), value.clone());
        }
    }
    Ok(values)
}

fn render_parameter(value: &ParameterValue) -> String {
    match value {
        ParameterValue::String(value) => escape_markdown(value),
        ParameterValue::Integer(value) => value.to_string(),
        ParameterValue::Boolean(value) => value.to_string(),
        ParameterValue::StringList(values) => {
            values.iter().map(|value| escape_markdown(value)).collect::<Vec<_>>().join(", ")
        }
    }
}

/// Escape all punctuation with Markdown structural meaning in accepted text contexts.
#[must_use]
pub fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '<'
                | '>'
                | '#'
                | '+'
                | '-'
                | '.'
                | '!'
                | '|'
                | '~'
        ) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn is_inline_code(line: &str, offset: usize) -> bool {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' {
            cursor += 1;
            continue;
        }
        let start = cursor;
        while cursor < bytes.len() && bytes[cursor] == b'`' {
            cursor += 1;
        }
        let width = cursor - start;
        let delimiter = &line[start..cursor];
        if let Some(relative_end) = line[cursor..].find(delimiter) {
            let end = cursor + relative_end;
            if offset >= cursor && offset < end {
                return true;
            }
            cursor = end + width;
        }
    }
    false
}

fn is_link_destination(line: &str, offset: usize) -> bool {
    let before = &line[..offset];
    let after = &line[offset..];
    before
        .rfind("](")
        .is_some_and(|start| !before[start + 2..].contains(')') && after.contains(')'))
}

fn valid_parameter_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn component_span(
    component: &LoadedComponent,
    output_line: usize,
    output_start: usize,
    output_end: usize,
    source_line: usize,
    source_start: usize,
    source_end: usize,
) -> ProvenanceSpan {
    ProvenanceSpan {
        output: TextSpan { line: output_line, start_column: output_start, end_column: output_end },
        origin: ProvenanceOrigin::Component {
            component_file: component.source_label.clone(),
            source_sha256: component.source_sha256.clone(),
            source: TextSpan {
                line: source_line,
                start_column: source_start,
                end_column: source_end,
            },
            instance_key: component.instance.instance_key.clone(),
        },
    }
}

fn context_error(component: &LoadedComponent, line: usize, context: &str) -> crate::ForgeError {
    composition_error(format!(
        "component '{}' line {line} contains a placeholder in unsupported {context}",
        component.source_label
    ))
}

fn byte_to_column(line: &str, offset: usize) -> usize {
    line[..offset].chars().count() + 1
}

fn hash_json(value: &ParameterValue) -> Result<String, crate::ForgeError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|source| composition_error(format!("failed to hash parameter value: {source}")))
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn pretty_json<T: Serialize>(value: &T) -> Result<Vec<u8>, crate::ForgeError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|source| {
        composition_error(format!("failed to serialize composition output: {source}"))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markdown_and_does_not_reparse_placeholder_values() {
        assert_eq!(escape_markdown("# [admin](x)"), r"\# \[admin\]\(x\)");
        assert_eq!(escape_markdown("{{forge:param:other}}"), r"\{\{forge:param:other\}\}");
    }

    #[test]
    fn placeholder_grammar_is_closed() {
        assert!(placeholder_occurrences("{{forge:param:owner-role}}").is_ok());
        assert!(placeholder_occurrences("{{forge:include:x}}").is_err());
        assert!(placeholder_occurrences("{{forge:param:Bad}}").is_err());
        assert!(placeholder_occurrences("{{forge:param:x}").is_err());
    }

    #[test]
    fn component_heading_contract_rejects_top_level_ambiguity() {
        assert!(validate_heading_structure("## Clause\n\n### Detail\n", "component.md").is_ok());
        for invalid in [
            "# Clause\n",
            "## Clause\n\n## Another\n",
            "## Clause\n\nAnother\n---\n",
            "## \n",
            "## Clause\n\n<h1>Unsafe</h1>\n",
        ] {
            assert!(
                validate_heading_structure(invalid, "component.md").is_err(),
                "accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn multi_backtick_inline_code_is_detected() {
        let line = "before ``{{forge:param:owner}}`` after";
        let offset = line.find("{{forge:").unwrap();
        assert!(is_inline_code(line, offset));
    }
}

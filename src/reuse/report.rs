//! Closed, bounded `forge.authoring-reuse/1`: deterministic reuse candidates.
//!
//! The report is a pointer document. It carries section metadata, ranked
//! candidate spans, source hashes and scores — never excerpt text, paraphrase,
//! or generated prose. A reader resolves the text from the named source at the
//! recorded span, which is what makes "never synthesise" checkable.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ForgeError;
use crate::authoring::model::DraftState;
use crate::json_strict::{self, Limits};

use super::corpus::MAX_STRING_BYTES;
use super::rank::{FLOOR, MAX_CANDIDATE_LIMIT, Reason};

/// Closed report contract version.
pub const REUSE_SCHEMA_VERSION: &str = "forge.authoring-reuse/1";
/// Maximum encoded report size, matching the shared authoring output bound.
pub const MAX_REPORT_BYTES: usize = 50 * 1024 * 1024;
/// Maximum reported sections.
pub const MAX_SECTIONS: usize = 100_000;

const LIMITS: Limits = Limits { max_depth: 16, max_string_bytes: MAX_STRING_BYTES };

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// One captured input file, project pin or corpus document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseInput {
    /// Capture role.
    pub role: String,
    /// Portable descendant path.
    pub path: String,
    /// Lowercase hexadecimal SHA-256 of the captured bytes.
    pub sha256: String,
    /// Captured byte length.
    pub byte_length: u64,
}

/// Half-open byte span into the named source document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseSpan {
    /// Inclusive zero-based byte offset.
    pub start: usize,
    /// Exclusive zero-based byte offset.
    pub end: usize,
}

/// One ranked candidate pointer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseCandidate {
    /// Stable corpus document key.
    pub source_key: String,
    /// Portable descendant path of the source document.
    pub source_path: String,
    /// Lowercase hexadecimal SHA-256 of the source document.
    pub source_sha256: String,
    /// Exact byte span of the candidate block.
    pub span: ReuseSpan,
    /// Quantised deterministic score.
    pub score: f64,
    /// Machine-readable match reasons.
    pub reasons: Vec<Reason>,
    /// Whether the score is below the low-confidence floor.
    pub low_confidence: bool,
}

/// One unresolved authoring section and its candidates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseSection {
    /// Stable policy key.
    pub policy_key: String,
    /// Stable topic key.
    pub topic_key: String,
    /// Topic title.
    pub title: String,
    /// Current drafting state; never `human-draft-present` in a report.
    pub state: DraftState,
    /// Applicability gap identifiers assigned to the section.
    pub gap_ids: Vec<String>,
    /// Framework control identifiers assigned to the section.
    pub control_ids: Vec<String>,
    /// Ranked candidates, best first.
    pub candidates: Vec<ReuseCandidate>,
}

/// Report totals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseCounts {
    /// Number of reported sections.
    pub sections: usize,
    /// Sections with at least one candidate.
    pub with_candidates: usize,
    /// Sections whose candidates are all low confidence.
    pub low_confidence: usize,
    /// Total candidates.
    pub candidates: usize,
}

/// The complete `forge.authoring-reuse/1` report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReuseReport {
    /// Closed contract version.
    pub schema_version: String,
    /// Stable project key.
    pub project_key: String,
    /// Authoring project timestamp; never a wall clock.
    pub as_of: String,
    /// SHA-256 of the corpus manifest bytes.
    pub corpus_sha256: String,
    /// Every captured input file.
    pub inputs: Vec<ReuseInput>,
    /// Reported sections in plan order.
    pub sections: Vec<ReuseSection>,
    /// Derived totals.
    pub counts: ReuseCounts,
}

impl ReuseReport {
    /// Assemble a report and derive its counts.
    #[must_use]
    pub fn new(
        project_key: String,
        as_of: String,
        corpus_sha256: String,
        inputs: Vec<ReuseInput>,
        sections: Vec<ReuseSection>,
    ) -> Self {
        let counts = ReuseCounts {
            sections: sections.len(),
            with_candidates: sections.iter().filter(|item| !item.candidates.is_empty()).count(),
            low_confidence: sections
                .iter()
                .filter(|item| {
                    !item.candidates.is_empty()
                        && item.candidates.iter().all(|candidate| candidate.low_confidence)
                })
                .count(),
            candidates: sections.iter().map(|item| item.candidates.len()).sum(),
        };
        Self {
            schema_version: REUSE_SCHEMA_VERSION.to_owned(),
            project_key,
            as_of,
            corpus_sha256,
            inputs,
            sections,
            counts,
        }
    }

    /// Whether any section still needs attention: no candidate, or only weak ones.
    #[must_use]
    pub fn action_required(&self) -> bool {
        self.sections.iter().any(|section| {
            section.candidates.is_empty()
                || section.candidates.iter().all(|candidate| candidate.low_confidence)
        })
    }

    /// Encode the report as pretty JSON with a trailing newline.
    ///
    /// # Errors
    /// Returns an authoring error when serialization fails or exceeds the bound.
    pub fn render_json(&self) -> Result<Vec<u8>, ForgeError> {
        let mut destination = BoundedJson(Vec::new(), MAX_REPORT_BYTES);
        serde_json::to_writer_pretty(&mut destination, self)
            .map_err(|cause| error(format!("cannot serialize reuse report: {cause}")))?;
        std::io::Write::write_all(&mut destination, b"\n")
            .map_err(|cause| error(format!("cannot finish reuse report: {cause}")))?;
        Ok(destination.0)
    }

    /// Render the same facts as deterministic, control-free text.
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        output.push_str("FORGE reuse candidates\n");
        let _ = writeln!(output, "schema: {}", escaped(&self.schema_version));
        let _ = writeln!(output, "project: {}", escaped(&self.project_key));
        let _ = writeln!(output, "as-of: {}", escaped(&self.as_of));
        let _ = writeln!(output, "corpus-sha256: {}", self.corpus_sha256);
        let _ = writeln!(output, "inputs: {}", self.inputs.len());
        for input in &self.inputs {
            let _ = writeln!(
                output,
                "- role={} path={} sha256={} bytes={}",
                escaped(&input.role),
                escaped(&input.path),
                input.sha256,
                input.byte_length,
            );
        }
        let counts = &self.counts;
        let _ = writeln!(
            output,
            "sections: {} with-candidates={} low-confidence={} candidates={}",
            counts.sections, counts.with_candidates, counts.low_confidence, counts.candidates,
        );
        for section in &self.sections {
            let _ = writeln!(
                output,
                "- policy={} topic={} title={} state={}",
                escaped(&section.policy_key),
                escaped(&section.topic_key),
                escaped(&section.title),
                section.state.as_str(),
            );
            let _ = writeln!(output, "  gaps: {}", labels(&section.gap_ids));
            let _ = writeln!(output, "  controls: {}", labels(&section.control_ids));
            let _ = writeln!(output, "  candidates: {}", section.candidates.len());
            for candidate in &section.candidates {
                let reasons: Vec<&str> =
                    candidate.reasons.iter().map(|reason| reason.as_str()).collect();
                let _ = writeln!(
                    output,
                    "  - source={} path={} sha256={} span={}..{} score={} low-confidence={} reasons={}",
                    escaped(&candidate.source_key),
                    escaped(&candidate.source_path),
                    candidate.source_sha256,
                    candidate.span.start,
                    candidate.span.end,
                    candidate.score,
                    candidate.low_confidence,
                    reasons.join(", "),
                );
            }
        }
        output
    }

    /// Parse and validate a bounded, closed report.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, unknown, duplicate or
    /// null fields, an unsupported version, or incoherent derived facts.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() > MAX_REPORT_BYTES {
            return Err(error(format!("reuse report exceeds the {MAX_REPORT_BYTES} byte limit")));
        }
        let value = json_strict::parse_value(bytes, "reuse report", LIMITS)
            .map_err(|cause| error(cause.to_string()))?;
        reject_nulls(&value, "reuse report")?;
        let report: Self = serde_json::from_value(value)
            .map_err(|cause| error(format!("invalid reuse report contract: {cause}")))?;
        report.validate()?;
        Ok(report)
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != REUSE_SCHEMA_VERSION {
            return Err(error(format!(
                "reuse report schema_version must be {REUSE_SCHEMA_VERSION}"
            )));
        }
        single_line("project_key", &self.project_key)?;
        single_line("as_of", &self.as_of)?;
        sha256("corpus_sha256", &self.corpus_sha256)?;
        if self.inputs.is_empty() {
            return Err(error("reuse report declares no inputs"));
        }
        for input in &self.inputs {
            single_line("input role", &input.role)?;
            single_line("input path", &input.path)?;
            crate::authoring::manifest::validate_local_path(
                "input path",
                std::path::Path::new(&input.path),
            )?;
            sha256("input sha256", &input.sha256)?;
        }
        if self.sections.len() > MAX_SECTIONS {
            return Err(error(format!("reuse report exceeds {MAX_SECTIONS} sections")));
        }
        for section in &self.sections {
            single_line("policy_key", &section.policy_key)?;
            single_line("topic_key", &section.topic_key)?;
            single_line("section title", &section.title)?;
            if section.state == DraftState::HumanDraftPresent {
                return Err(error("reuse report must not include human-draft-present sections"));
            }
            if section.candidates.len() > MAX_CANDIDATE_LIMIT {
                return Err(error(format!(
                    "reuse report section exceeds {MAX_CANDIDATE_LIMIT} candidates"
                )));
            }
            for candidate in &section.candidates {
                single_line("source_key", &candidate.source_key)?;
                single_line("source_path", &candidate.source_path)?;
                sha256("source_sha256", &candidate.source_sha256)?;
                if candidate.span.start > candidate.span.end {
                    return Err(error("reuse candidate span start exceeds its end"));
                }
                if !candidate.score.is_finite() || candidate.score < 0.0 {
                    return Err(error(
                        "reuse candidate score must be a finite non-negative number",
                    ));
                }
                if candidate.low_confidence != (candidate.score < FLOOR) {
                    return Err(error("reuse candidate low_confidence does not match its score"));
                }
                let mut seen = std::collections::BTreeSet::new();
                for reason in &candidate.reasons {
                    if !seen.insert(*reason) {
                        return Err(error("reuse candidate reasons must be unique"));
                    }
                }
            }
        }
        if self.counts.sections != self.sections.len()
            || self.counts.candidates
                != self.sections.iter().map(|section| section.candidates.len()).sum::<usize>()
            || self.counts.with_candidates
                != self.sections.iter().filter(|section| !section.candidates.is_empty()).count()
            || self.counts.low_confidence
                != self
                    .sections
                    .iter()
                    .filter(|section| {
                        !section.candidates.is_empty()
                            && section.candidates.iter().all(|candidate| candidate.low_confidence)
                    })
                    .count()
        {
            return Err(error("reuse report counts do not match its sections"));
        }
        Ok(())
    }
}

struct BoundedJson(Vec<u8>, usize);

impl std::io::Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) > self.1 {
            return Err(std::io::Error::other("reuse report exceeds the byte limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn reject_nulls(value: &Value, path: &str) -> Result<(), ForgeError> {
    match value {
        Value::Null => Err(error(format!("{path} must not be null; omit the key instead"))),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_nulls(item, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        Value::Object(entries) => {
            for (key, item) in entries {
                reject_nulls(item, &format!("{path}.{}", json_strict::bounded(key)))?;
            }
            Ok(())
        }
        Value::Bool(_) | Value::Number(_) | Value::String(_) => Ok(()),
    }
}

fn sha256(name: &str, value: &str) -> Result<(), ForgeError> {
    json_strict::validate_lowercase_sha256(name, value).map_err(error)
}

fn single_line(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err(error(format!("{name} must be one non-empty line")));
    }
    Ok(())
}

fn labels(values: &[String]) -> String {
    values.iter().map(|value| escaped(value)).collect::<Vec<_>>().join(", ")
}

fn escaped(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn candidate(score: f64) -> ReuseCandidate {
        ReuseCandidate {
            source_key: "policy".to_owned(),
            source_path: "prior/policy.md".to_owned(),
            source_sha256: DIGEST.to_owned(),
            span: ReuseSpan { start: 10, end: 20 },
            score,
            reasons: vec![Reason::ControlMatch],
            low_confidence: score < FLOOR,
        }
    }

    fn section(state: DraftState, candidates: Vec<ReuseCandidate>) -> ReuseSection {
        ReuseSection {
            policy_key: "access-policy".to_owned(),
            topic_key: "access-topic".to_owned(),
            title: "Access control".to_owned(),
            state,
            gap_ids: vec!["gap-1".to_owned()],
            control_ids: vec!["ac-1".to_owned()],
            candidates,
        }
    }

    fn report(sections: Vec<ReuseSection>) -> ReuseReport {
        ReuseReport::new(
            "synthetic-project".to_owned(),
            "2026-09-11T00:00:00Z".to_owned(),
            DIGEST.to_owned(),
            vec![ReuseInput {
                role: "author-project".to_owned(),
                path: "project.json".to_owned(),
                sha256: DIGEST.to_owned(),
                byte_length: 10,
            }],
            sections,
        )
    }

    #[test]
    fn json_and_text_agree_and_round_trip() {
        let report = report(vec![section(DraftState::SkeletonReady, vec![candidate(2.5)])]);
        let bytes = report.render_json().unwrap();
        let parsed = ReuseReport::parse(&bytes).unwrap();
        assert_eq!(parsed, report);
        let text = report.render_text();
        assert!(text.contains("schema: forge.authoring-reuse/1"));
        assert!(text.contains("span=10..20 score=2.5 low-confidence=false reasons=control-match"));
        assert!(text.contains("sections: 1 with-candidates=1 low-confidence=0 candidates=1"));
    }

    #[test]
    fn counts_and_action_required_track_weak_or_empty_sections() {
        let weak = report(vec![section(DraftState::SkeletonReady, vec![candidate(0.5)])]);
        assert_eq!(weak.counts.low_confidence, 1);
        assert!(weak.action_required());
        let strong = report(vec![section(DraftState::SkeletonReady, vec![candidate(2.5)])]);
        assert_eq!(strong.counts.low_confidence, 0);
        assert!(!strong.action_required());
        let empty = report(vec![section(DraftState::Planned, Vec::new())]);
        assert_eq!(empty.counts.with_candidates, 0);
        assert!(empty.action_required());
    }

    #[test]
    fn closed_report_rejects_unknown_duplicate_null_and_forward_versions() {
        let mut value = serde_json::to_value(report(vec![section(
            DraftState::SkeletonReady,
            vec![candidate(2.5)],
        )]))
        .unwrap();
        value["pointer"] = json!("nope");
        assert!(ReuseReport::parse(&serde_json::to_vec(&value).unwrap()).is_err());

        let mut forward =
            serde_json::to_value(report(vec![section(DraftState::Planned, Vec::new())])).unwrap();
        forward["schema_version"] = json!("forge.authoring-reuse/2");
        assert!(ReuseReport::parse(&serde_json::to_vec(&forward).unwrap()).is_err());

        let mut null =
            serde_json::to_value(report(vec![section(DraftState::Planned, Vec::new())])).unwrap();
        null["inputs"][0]["role"] = Value::Null;
        assert!(ReuseReport::parse(&serde_json::to_vec(&null).unwrap()).is_err());

        let decoded = format!(
            "{{\"schema_version\":\"{REUSE_SCHEMA_VERSION}\",\"schema_version\":\"{REUSE_SCHEMA_VERSION}\"}}"
        );
        assert!(ReuseReport::parse(decoded.as_bytes()).is_err());
    }

    #[test]
    fn incoherent_counts_spans_and_flags_are_rejected() {
        let mut counts =
            serde_json::to_value(report(vec![section(DraftState::Planned, Vec::new())])).unwrap();
        counts["counts"]["candidates"] = json!(7);
        assert!(ReuseReport::parse(&serde_json::to_vec(&counts).unwrap()).is_err());

        let mut span = serde_json::to_value(report(vec![section(
            DraftState::SkeletonReady,
            vec![candidate(2.5)],
        )]))
        .unwrap();
        span["sections"][0]["candidates"][0]["span"]["end"] = json!(3);
        assert!(ReuseReport::parse(&serde_json::to_vec(&span).unwrap()).is_err());

        let mut flag = serde_json::to_value(report(vec![section(
            DraftState::SkeletonReady,
            vec![candidate(2.5)],
        )]))
        .unwrap();
        flag["sections"][0]["candidates"][0]["low_confidence"] = json!(true);
        assert!(ReuseReport::parse(&serde_json::to_vec(&flag).unwrap()).is_err());

        let mut drafted = serde_json::to_value(report(vec![section(
            DraftState::SkeletonReady,
            vec![candidate(2.5)],
        )]))
        .unwrap();
        drafted["sections"][0]["state"] = json!("human-draft-present");
        assert!(ReuseReport::parse(&serde_json::to_vec(&drafted).unwrap()).is_err());
    }

    #[test]
    fn generated_reports_use_only_contract_terminology() {
        let report = report(vec![section(DraftState::SkeletonReady, vec![candidate(2.5)])]);
        let text = report.render_text();
        for forbidden in ["approved", "compliant", "certified", "validated", "effective"] {
            assert!(!text.to_lowercase().contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn report_schema_file_is_published_and_closed() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/forge.authoring-reuse-1.schema.json"))
                .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/authoring-reuse/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}

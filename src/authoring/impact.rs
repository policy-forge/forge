//! Explicit, independently validated authoring baseline and dependency comparison.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error;
use super::manifest::{self, PinnedFile, Review, Reviewer};
use super::model::{AuthoringPlan, LoadedAuthorProject};
use crate::ForgeError;
use crate::hashing::sha256_hex;

pub const MANIFEST_SCHEMA_VERSION: &str = "forge.authoring-impact/1";
pub const REPORT_SCHEMA_VERSION: &str = "forge.authoring-impact-report/1";
const MAX_FINDINGS: usize = 10_000;
const MAX_GRAPH: usize = 100_000;
type SectionKey = (String, String);

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotPin {
    pub project: PinnedFile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub components: Option<PinnedFile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactManifest {
    pub schema_version: String,
    pub old: SnapshotPin,
    pub new: SnapshotPin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correspondence: Option<ReviewedCorrespondence>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedCorrespondence {
    pub old_framework_sha256: String,
    pub new_framework_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_resolved_catalog_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_resolved_catalog_sha256: Option<String>,
    pub reviewers: Vec<Reviewer>,
    pub review: Review,
    pub controls: Vec<ControlCorrespondence>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlCorrespondence {
    pub old_control_id: String,
    pub new_control_id: String,
}

/// Redacted evidence only. Values must never enter this adapter.
#[derive(Debug, Clone, Serialize)]
pub struct ComponentSnapshot {
    pub instance_key: String,
    pub policy_key: String,
    pub topic_key: String,
    pub source_sha256: String,
    pub sidecar_sha256: String,
    pub record_sha256: String,
    pub rendered_sha256: Option<String>,
    pub answer_keys: Vec<String>,
    pub question_keys: Vec<String>,
}

/// All source capture, baseline recomputation, component validation and rendering
/// must succeed independently before constructing a snapshot. The comparison
/// never reads files, renders mismatched content, or updates inputs.
pub struct ImpactSnapshot {
    pub loaded: LoadedAuthorProject,
    pub plan: AuthoringPlan,
    pub policies: BTreeMap<String, String>,
    pub sections: BTreeMap<SectionKey, String>,
    pub control_fingerprints: BTreeMap<String, String>,
    pub components: Vec<ComponentSnapshot>,
    pub component_manifest_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonStatus {
    Complete,
    Incomplete,
    Unsupported,
}

impl ComparisonStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeCategory {
    InputUnverified,
    ComponentInputUnverified,
    InputDrift,
    InputMissing,
    UnsupportedCorrespondence,
    BaselineBindingChanged,
    ProjectBindingChanged,
    FrameworkControlChanged,
    FrameworkControlAdded,
    FrameworkControlRemoved,
    GapAdded,
    GapRemoved,
    GapClassificationChanged,
    GapBindingChanged,
    AssignmentChanged,
    DeferralChanged,
    AnswerChanged,
    QuestionDefinitionChanged,
    AnswerExpiryChanged,
    AnswerStateChanged,
    PackChanged,
    HumanClauseChanged,
    ComponentChanged,
    ComponentBindingChanged,
    ComponentExtensionBindingChanged,
    PolicyAdded,
    PolicyRemoved,
    PolicyDefinitionChanged,
    SectionAdded,
    SectionRemoved,
    SectionDefinitionChanged,
    DependencyProvenanceChanged,
    AuthoringStateChanged,
    OutputBytesChanged,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // Orthogonal change axes must not imply one another.
pub struct ChangeAxes {
    pub substantive_changed: bool,
    pub provenance_changed: bool,
    pub authoring_state_changed: bool,
    pub output_bytes_changed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UnverifiedReason {
    SnapshotInput,
    ComponentInput,
    RenderedOutput,
    InputDrift,
    MissingInput,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactFinding {
    pub finding_id: String,
    pub category: ChangeCategory,
    pub unverified_reason: Option<UnverifiedReason>,
    pub subject_key: String,
    pub axes: ChangeAxes,
    pub old_sha256: Option<String>,
    pub new_sha256: Option<String>,
    pub old_gap_id: Option<String>,
    pub new_gap_id: Option<String>,
    /// Exact report hash for fully verified comparison inputs; both are null for incomplete comparisons.
    pub old_report_sha256: Option<String>,
    /// Exact report hash for fully verified comparison inputs; both are null for incomplete comparisons.
    pub new_report_sha256: Option<String>,
    pub affected_sections: Vec<SectionReference>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SectionReference {
    pub policy_key: String,
    pub topic_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectionImpact {
    pub policy_key: String,
    pub topic_key: String,
    pub old_output_sha256: Option<String>,
    pub new_output_sha256: Option<String>,
    pub axes: ChangeAxes,
    pub unaffected: bool,
    pub finding_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyImpact {
    pub policy_key: String,
    pub old_output_sha256: Option<String>,
    pub new_output_sha256: Option<String>,
    pub axes: ChangeAxes,
    pub unaffected: bool,
    pub finding_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImpactReport {
    pub schema_version: String,
    pub status: ComparisonStatus,
    pub old_project_sha256: String,
    pub new_project_sha256: String,
    /// Exact report hash for fully verified comparison inputs; both are null for incomplete comparisons.
    pub old_report_sha256: Option<String>,
    /// Exact report hash for fully verified comparison inputs; both are null for incomplete comparisons.
    pub new_report_sha256: Option<String>,
    pub correspondence_sha256: Option<String>,
    pub request_sha256: Option<String>,
    pub findings: Vec<ImpactFinding>,
    pub sections: Vec<SectionImpact>,
    pub policies: Vec<PolicyImpact>,
    #[serde(skip)]
    reference_count: usize,
    #[serde(skip)]
    evidence_bytes: usize,
}

impl ImpactReport {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.status == ComparisonStatus::Complete
    }
    #[must_use]
    pub fn action_required(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// Parse a closed bounded comparison contract. Null is never an absent field.
/// # Errors
/// Rejects invalid, aliased, duplicate-key, unsupported or over-limit contracts.
pub fn parse_manifest(bytes: &[u8]) -> Result<ImpactManifest, ForgeError> {
    if bytes.len() as u64 > manifest::MAX_MANIFEST_BYTES {
        return Err(error(format!(
            "authoring impact manifest exceeds {} byte limit",
            manifest::MAX_MANIFEST_BYTES
        )));
    }
    let value = crate::json_strict::parse_value(
        bytes,
        "authoring impact manifest",
        crate::json_strict::Limits { max_depth: 32, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("invalid strict authoring impact JSON"))?;
    reject_null(&value)?;
    let parsed: ImpactManifest = serde_json::from_value(value)
        .map_err(|_| error("invalid closed authoring impact contract"))?;
    if parsed.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(error("unsupported authoring impact schema version"));
    }
    let mut paths = BTreeSet::new();
    for snapshot in [&parsed.old, &parsed.new] {
        if snapshot
            .components
            .as_ref()
            .is_some_and(|components| components.path == snapshot.project.path)
        {
            return Err(error(
                "a snapshot project and component extension must name distinct files",
            ));
        }
        for pin in std::iter::once(&snapshot.project).chain(snapshot.components.iter()) {
            validate_pin(pin)?;
            let path = pin.path.to_str().ok_or_else(|| error("impact paths must be UTF-8"))?;
            // Identical old/new references support a no-op comparison. Other spellings
            // of the same file are rejected by confined capture identity validation.
            paths.insert(path.to_owned());
        }
    }
    let paths: Vec<_> = paths.into_iter().collect();
    for (index, path) in paths.iter().enumerate() {
        if paths[..index].iter().any(|other| path.eq_ignore_ascii_case(other)) {
            return Err(error("impact input paths alias one another"));
        }
    }
    if let Some(correspondence) = &parsed.correspondence {
        validate_correspondence(correspondence)?;
    }
    Ok(parsed)
}

fn reject_null(value: &Value) -> Result<(), ForgeError> {
    match value {
        Value::Null => return Err(error("impact contracts do not accept null")),
        Value::Array(values) => {
            for value in values {
                reject_null(value)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                reject_null(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_pin(pin: &PinnedFile) -> Result<(), ForgeError> {
    manifest::validate_local_path("impact input", &pin.path)?;
    validate_sha(&pin.expected_sha256)
}

fn validate_sha(value: &str) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256("impact fingerprint", value)
        .map_err(|_| error("invalid impact SHA-256 fingerprint"))
}

fn validate_label(value: &str) -> Result<(), ForgeError> {
    if !manifest::has_nonblank_text(value) || value.len() > manifest::MAX_STRING_BYTES || value.trim() != value
        || value.chars().any(|c| c.is_control() || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')) {
        return Err(error("invalid impact identifier or review metadata"));
    }
    Ok(())
}

fn validate_key(value: &str) -> Result<(), ForgeError> {
    if value.len() > 64
        || !value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        || value.ends_with('-')
        || value.contains("--")
        || !value.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(error("invalid impact stable key"));
    }
    Ok(())
}

fn validate_correspondence(value: &ReviewedCorrespondence) -> Result<(), ForgeError> {
    for sha in [&value.old_framework_sha256, &value.new_framework_sha256]
        .into_iter()
        .chain(value.old_resolved_catalog_sha256.iter())
        .chain(value.new_resolved_catalog_sha256.iter())
    {
        validate_sha(sha)?;
    }
    if value.controls.is_empty()
        || value.controls.len() > MAX_GRAPH
        || value.reviewers.is_empty()
        || value.reviewers.len() > manifest::MAX_REVIEWERS
    {
        return Err(error("impact correspondence exceeds record limits"));
    }
    let mut reviewers = BTreeSet::new();
    for reviewer in &value.reviewers {
        validate_key(&reviewer.key)?;
        validate_label(&reviewer.name)?;
        if !reviewers.insert(&reviewer.key) {
            return Err(error("duplicate correspondence reviewer"));
        }
    }
    validate_key(&value.review.reviewer_key)?;
    validate_label(&value.review.rationale)?;
    validate_label(&value.review.reviewed_at)?;
    if value.review.reviewed_at.len() > 64 {
        return Err(error("impact review timestamp exceeds 64 bytes"));
    }
    if !reviewers.contains(&value.review.reviewer_key)
        || chrono::DateTime::parse_from_rfc3339(&value.review.reviewed_at).is_err()
    {
        return Err(error("correspondence requires a registered reviewer and valid explicit time"));
    }
    let mut old = BTreeSet::new();
    let mut new = BTreeSet::new();
    for pair in &value.controls {
        validate_label(&pair.old_control_id)?;
        validate_label(&pair.new_control_id)?;
        if !old.insert(&pair.old_control_id) || !new.insert(&pair.new_control_id) {
            return Err(error("control correspondence must be one-to-one"));
        }
    }
    Ok(())
}

fn digest(value: &impl Serialize) -> Result<String, ForgeError> {
    let bytes = bounded_json(value, false, super::output::MAX_OUTPUT_BYTES)?;
    Ok(sha256_hex(&bytes))
}

struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(format!(
                "impact output exceeds {} byte limit",
                self.limit
            )));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bounded_json(value: &impl Serialize, pretty: bool, limit: usize) -> Result<Vec<u8>, ForgeError> {
    let mut writer =
        BoundedWriter { bytes: Vec::new(), limit: limit.min(super::output::MAX_OUTPUT_BYTES) };
    if pretty {
        serde_json::to_writer_pretty(&mut writer, value)
    } else {
        serde_json::to_writer(&mut writer, value)
    }
    .map_err(|_| error(format!("impact JSON exceeds {} byte limit", writer.limit)))?;
    if pretty {
        writer
            .write_all(b"\n")
            .map_err(|_| error(format!("impact JSON exceeds {} byte limit", writer.limit)))?;
    }
    Ok(writer.bytes)
}

/// Render only redacted impact evidence, with deterministic field and record order.
/// # Errors
/// Returns an error when output exceeds the aggregate artifact bound.
pub fn render_json(report: &ImpactReport) -> Result<Vec<u8>, ForgeError> {
    render_json_bounded(report, super::output::MAX_OUTPUT_BYTES)
}

/// Render redacted JSON within the caller's remaining generation byte budget.
/// # Errors
/// Returns an error before growing the buffer beyond the supplied limit.
pub fn render_json_bounded(report: &ImpactReport, limit: usize) -> Result<Vec<u8>, ForgeError> {
    bounded_json(report, true, limit)
}

/// Render deterministic identifiers and reason codes; never raw source values.
/// # Errors
/// Returns an error when output exceeds the aggregate artifact bound.
pub fn render_text(report: &ImpactReport) -> Result<String, ForgeError> {
    render_text_bounded(report, super::output::MAX_OUTPUT_BYTES)
}

/// Render redacted text within the caller's remaining generation byte budget.
/// # Errors
/// Returns an error before growing the buffer beyond the supplied limit.
pub fn render_text_bounded(report: &ImpactReport, limit: usize) -> Result<String, ForgeError> {
    let mut out =
        BoundedWriter { bytes: Vec::new(), limit: limit.min(super::output::MAX_OUTPUT_BYTES) };
    writeln!(out, "FORGE authoring impact — drafting dependencies only")
        .map_err(|_| error(format!("impact text exceeds {} byte limit", out.limit)))?;
    writeln!(
        out,
        "Comparison: {}\nOld project: {}\nNew project: {}",
        report.status.as_str(),
        report.old_project_sha256,
        report.new_project_sha256
    )
    .map_err(|_| error(format!("impact text exceeds {} byte limit", out.limit)))?;
    for finding in &report.findings {
        let category = serde_json::to_value(finding.category)
            .map_err(|_| error("impact category serialization failed"))?;
        writeln!(
            out,
            "{}: {} [{}]",
            category.as_str().unwrap_or("unknown"),
            finding.subject_key,
            finding.finding_id
        )
        .map_err(|_| error(format!("impact text exceeds {} byte limit", out.limit)))?;
    }
    for policy in &report.policies {
        writeln!(
            out,
            "Policy {}: {}",
            policy.policy_key,
            if policy.unaffected { "unaffected (exact bytes and dependencies)" } else { "changed" }
        )
        .map_err(|_| error(format!("impact text exceeds {} byte limit", out.limit)))?;
    }
    String::from_utf8(out.bytes).map_err(|_| error("impact text encoding failed"))
}

/// Produce an explicitly unverified result without reflecting sensitive loader errors.
/// # Errors
/// Rejects invalid outer pins or a side other than old/new/both.
pub fn incomplete_report(
    old_project_sha256: &str,
    new_project_sha256: &str,
    side: &str,
) -> Result<ImpactReport, ForgeError> {
    incomplete_report_with_reason(
        old_project_sha256,
        new_project_sha256,
        side,
        UnverifiedReason::SnapshotInput,
    )
}

/// Preserve an independently identified failure category without copying loader errors.
/// Do not infer a reason by parsing diagnostic prose.
/// # Errors
/// Rejects invalid outer pins or a side other than old/new/both.
pub fn incomplete_report_with_reason(
    old_project_sha256: &str,
    new_project_sha256: &str,
    side: &str,
    reason: UnverifiedReason,
) -> Result<ImpactReport, ForgeError> {
    validate_sha(old_project_sha256)?;
    validate_sha(new_project_sha256)?;
    if !matches!(side, "old" | "new" | "both") {
        return Err(error("invalid unverified snapshot side"));
    }
    let mut report = empty_report(old_project_sha256, new_project_sha256, None, None);
    report.status = ComparisonStatus::Incomplete;
    let category = match reason {
        UnverifiedReason::ComponentInput => ChangeCategory::ComponentInputUnverified,
        UnverifiedReason::InputDrift => ChangeCategory::InputDrift,
        UnverifiedReason::MissingInput => ChangeCategory::InputMissing,
        UnverifiedReason::SnapshotInput | UnverifiedReason::RenderedOutput => {
            ChangeCategory::InputUnverified
        }
    };
    add_finding(
        &mut report,
        category,
        side,
        ChangeAxes::default(),
        None,
        None,
        None,
        None,
        BTreeSet::new(),
    )?;
    report.findings[0].unverified_reason = Some(reason);
    // Bind the closed failure reason as well as the outer exact pins.
    report.findings[0].finding_id = digest(&(
        "forge.authoring-unverified-finding/1",
        old_project_sha256,
        new_project_sha256,
        side,
        reason,
    ))?;
    Ok(report)
}

/// Bind the complete exact comparison request, including optional extension pins,
/// to the report and every finding identity before publication.
/// # Errors
/// Rejects invalid SHA-256 or rebinding a report to a different request.
pub fn bind_request(report: &mut ImpactReport, request_sha256: &str) -> Result<(), ForgeError> {
    validate_sha(request_sha256)?;
    if let Some(existing) = &report.request_sha256 {
        if existing == request_sha256 {
            return Ok(());
        }
        return Err(error("impact report already binds a different comparison request"));
    }
    let mut identities = BTreeMap::new();
    for finding in &mut report.findings {
        let id = digest(&(
            "forge.authoring-impact-request-finding/1",
            request_sha256,
            &finding.finding_id,
        ))?;
        identities.insert(finding.finding_id.clone(), id.clone());
        finding.finding_id = id;
    }
    for ids in report
        .sections
        .iter_mut()
        .map(|s| &mut s.finding_ids)
        .chain(report.policies.iter_mut().map(|p| &mut p.finding_ids))
    {
        for id in ids.iter_mut() {
            *id = identities
                .get(id)
                .ok_or_else(|| error("impact finding reference is missing"))?
                .clone();
        }
        ids.sort();
    }
    report.findings.sort_by(|a, b| a.finding_id.cmp(&b.finding_id));
    report.request_sha256 = Some(request_sha256.into());
    Ok(())
}

fn empty_report(
    old: &str,
    new: &str,
    old_report: Option<&str>,
    new_report: Option<&str>,
) -> ImpactReport {
    ImpactReport {
        schema_version: REPORT_SCHEMA_VERSION.into(),
        status: ComparisonStatus::Complete,
        old_project_sha256: old.into(),
        new_project_sha256: new.into(),
        old_report_sha256: old_report.map(str::to_owned),
        new_report_sha256: new_report.map(str::to_owned),
        correspondence_sha256: None,
        request_sha256: None,
        findings: Vec::new(),
        sections: Vec::new(),
        policies: Vec::new(),
        reference_count: 0,
        // Reserve fixed report metadata, including a later exact request binding.
        evidence_bytes: 2048,
    }
}

fn axes(content: bool, provenance: bool, state: bool) -> ChangeAxes {
    ChangeAxes {
        substantive_changed: content,
        provenance_changed: provenance,
        authoring_state_changed: state,
        output_bytes_changed: false,
    }
}
fn merge_axes(target: &mut ChangeAxes, source: &ChangeAxes) {
    target.substantive_changed |= source.substantive_changed;
    target.provenance_changed |= source.provenance_changed;
    target.authoring_state_changed |= source.authoring_state_changed;
    target.output_bytes_changed |= source.output_bytes_changed;
}
fn unchanged(value: &ChangeAxes) -> bool {
    value == &ChangeAxes::default()
}

#[allow(clippy::too_many_arguments)]
fn add_finding(
    report: &mut ImpactReport,
    category: ChangeCategory,
    subject: &str,
    axes: ChangeAxes,
    old: Option<String>,
    new: Option<String>,
    old_gap: Option<String>,
    new_gap: Option<String>,
    sections: BTreeSet<SectionKey>,
) -> Result<(), ForgeError> {
    if report.findings.len() >= MAX_FINDINGS {
        return Err(error("impact exceeds 10000 finding limit"));
    }
    validate_label(subject)?;
    if sections.len() > MAX_GRAPH.saturating_sub(report.reference_count) {
        return Err(error("impact expanded finding references exceed limit"));
    }
    let string_bytes = subject.len().saturating_add(
        sections
            .iter()
            .map(|(policy, topic)| policy.len().saturating_add(topic.len()))
            .sum::<usize>(),
    );
    // Conservative JSON escaping and fixed hash/field overhead are accounted
    // before copying a subject or expanding section references into a finding.
    let additional = string_bytes.saturating_mul(6).saturating_add(2048);
    if additional > super::output::MAX_OUTPUT_BYTES.saturating_sub(report.evidence_bytes) {
        return Err(error("impact finding evidence exceeds aggregate byte budget"));
    }
    report.reference_count += sections.len();
    report.evidence_bytes += additional;
    let id = digest(&(
        "forge.authoring-impact-finding/1",
        &report.old_project_sha256,
        &report.new_project_sha256,
        &report.old_report_sha256,
        &report.new_report_sha256,
        &report.correspondence_sha256,
        category,
        subject,
        &old,
        &new,
        &old_gap,
        &new_gap,
        &sections,
    ))?;
    report.findings.push(ImpactFinding {
        finding_id: id,
        category,
        unverified_reason: None,
        subject_key: subject.into(),
        axes,
        old_sha256: old,
        new_sha256: new,
        old_gap_id: old_gap,
        new_gap_id: new_gap,
        old_report_sha256: report.old_report_sha256.clone(),
        new_report_sha256: report.new_report_sha256.clone(),
        affected_sections: sections
            .into_iter()
            .map(|(policy_key, topic_key)| SectionReference { policy_key, topic_key })
            .collect(),
    });
    Ok(())
}

struct SnapshotIndex<'a> {
    sections: BTreeMap<SectionKey, &'a super::model::SectionPlan>,
    question_sections: BTreeMap<String, BTreeSet<SectionKey>>,
    control_sections: BTreeMap<String, BTreeSet<SectionKey>>,
}

fn reserve_references(total: &mut usize, additional: usize) -> Result<(), ForgeError> {
    if additional > MAX_GRAPH.saturating_sub(*total) {
        return Err(error("impact dependency graph exceeds bounds"));
    }
    *total += additional;
    Ok(())
}

#[allow(clippy::too_many_lines)] // One ordered comparison preserves the independent change axes.
fn index(snapshot: &ImpactSnapshot) -> Result<SnapshotIndex<'_>, ForgeError> {
    if let Some(sha) = &snapshot.component_manifest_sha256 {
        validate_sha(sha)?;
    }
    let mut planned_references = 0usize;
    for count in snapshot
        .plan
        .policies
        .iter()
        .flat_map(|policy| &policy.sections)
        .map(|section| section.questions.len().saturating_add(section.control_ids.len()))
        .chain(snapshot.loaded.project.human_clauses.iter().map(|clause| clause.answer_refs.len()))
        .chain(snapshot.components.iter().map(|component| {
            component.answer_keys.len().saturating_add(component.question_keys.len())
        }))
    {
        reserve_references(&mut planned_references, count)?;
    }
    if snapshot.plan.policies.len() > manifest::MAX_TOPICS
        || snapshot.sections.len() > MAX_GRAPH
        || snapshot.components.len() > manifest::MAX_RECORDS
    {
        return Err(error("impact snapshot records exceed bounds"));
    }
    let mut index = SnapshotIndex {
        sections: BTreeMap::new(),
        question_sections: BTreeMap::new(),
        control_sections: BTreeMap::new(),
    };
    if snapshot.plan.project_key != snapshot.loaded.project.project_key
        || snapshot.plan.provenance.project_sha256 != snapshot.loaded.project_sha256
        || snapshot.plan.provenance.pack_sha256 != snapshot.loaded.pack_sha256
        || snapshot.plan.provenance.report_sha256 != snapshot.loaded.report_sha256
    {
        return Err(error("impact snapshot plan does not match validated project evidence"));
    }
    for sha in [
        &snapshot.loaded.project_sha256,
        &snapshot.loaded.pack_sha256,
        &snapshot.loaded.report_sha256,
    ] {
        validate_sha(sha)?;
    }
    let mut policy_keys = BTreeSet::new();
    let mut references = 0usize;
    for policy in &snapshot.plan.policies {
        if !policy_keys.insert(policy.policy_key.clone()) {
            return Err(error("duplicate impact policy"));
        }
        for section in &policy.sections {
            let key = (policy.policy_key.clone(), section.topic_key.clone());
            if index.sections.insert(key.clone(), section).is_some() {
                return Err(error("duplicate impact section"));
            }
            for question in &section.questions {
                index
                    .question_sections
                    .entry(question.question_key.clone())
                    .or_default()
                    .insert(key.clone());
                references += 1;
            }
            for control in &section.control_ids {
                index.control_sections.entry(control.clone()).or_default().insert(key.clone());
                references += 1;
            }
        }
    }
    let mut topic_sections: BTreeMap<&str, Vec<&SectionKey>> = BTreeMap::new();
    for key in index.sections.keys() {
        topic_sections.entry(key.1.as_str()).or_default().push(key);
    }
    for assignment in &snapshot.loaded.pack.control_assignments {
        for &key in topic_sections.get(assignment.topic_key.as_str()).into_iter().flatten() {
            if index
                .control_sections
                .get(&assignment.control_id)
                .is_some_and(|sections| sections.contains(key))
            {
                continue;
            }
            // Preserve the clause/component edge allowance already reserved by
            // preflight; new no-gap edges consume only its remaining budget.
            reserve_references(&mut planned_references, 1)?;
            index
                .control_sections
                .entry(assignment.control_id.clone())
                .or_default()
                .insert(key.clone());
            references += 1;
        }
    }
    let answer_questions: BTreeMap<_, _> = snapshot
        .loaded
        .project
        .answers
        .iter()
        .map(|answer| (&answer.key, &answer.question_key))
        .collect();
    for clause in &snapshot.loaded.project.human_clauses {
        let key = (clause.policy_key.clone(), clause.topic_key.clone());
        for answer in &clause.answer_refs {
            let question = answer_questions
                .get(&answer.answer_key)
                .ok_or_else(|| error("impact clause answer is missing"))?;
            index.question_sections.entry((*question).clone()).or_default().insert(key.clone());
            references += 1;
        }
    }
    let known_questions: BTreeSet<_> =
        snapshot.loaded.pack.questions.iter().map(|q| q.key.as_str()).collect();
    let mut component_keys = BTreeSet::new();
    for component in &snapshot.components {
        let key = (component.policy_key.clone(), component.topic_key.clone());
        if !component_keys.insert(&component.instance_key) || !index.sections.contains_key(&key) {
            return Err(error("invalid impact component identity or destination"));
        }
        for sha in [&component.source_sha256, &component.sidecar_sha256, &component.record_sha256]
            .into_iter()
            .chain(component.rendered_sha256.iter())
        {
            validate_sha(sha)?;
        }
        for question in &component.question_keys {
            if !known_questions.contains(question.as_str()) {
                return Err(error("impact component names an unknown question dependency"));
            }
            index.question_sections.entry(question.clone()).or_default().insert(key.clone());
            references += 1;
        }
        for answer in &component.answer_keys {
            if let Some(question) = answer_questions.get(answer) {
                if !component.question_keys.contains(question) {
                    return Err(error(
                        "impact component answer lacks its explicit question dependency",
                    ));
                }
                index.question_sections.entry((*question).clone()).or_default().insert(key.clone());
                references += 1;
            } else if component.question_keys.is_empty() {
                return Err(error(
                    "impact missing component answer requires an explicit question dependency",
                ));
            }
            // An absent answer is valid blocked context. Its declared question
            // edge above remains indexed; never drop it or invent a replacement.
        }
    }
    if references > MAX_GRAPH
        || index.sections.len() > MAX_GRAPH
        || snapshot.components.len() > manifest::MAX_RECORDS
    {
        return Err(error("impact dependency graph exceeds bounds"));
    }
    if policy_keys != snapshot.policies.keys().cloned().collect()
        || index.sections.keys().ne(snapshot.sections.keys())
    {
        return Err(error("impact requires exact rendered digests for every policy and section"));
    }
    let controls: BTreeSet<_> = snapshot
        .loaded
        .baseline_report
        .controls
        .iter()
        .map(|control| control.control_id.clone())
        .collect();
    if controls != snapshot.control_fingerprints.keys().cloned().collect() {
        return Err(error("impact requires verified fingerprints for every framework control"));
    }
    for sha in snapshot
        .policies
        .values()
        .chain(snapshot.sections.values())
        .chain(snapshot.control_fingerprints.values())
    {
        validate_sha(sha)?;
    }
    Ok(index)
}

fn union_sections(
    left: Option<&BTreeSet<SectionKey>>,
    right: Option<&BTreeSet<SectionKey>>,
) -> BTreeSet<SectionKey> {
    left.into_iter().chain(right).flat_map(|set| set.iter().cloned()).collect()
}
fn section_set(policy: &str, topic: &str) -> BTreeSet<SectionKey> {
    BTreeSet::from([(policy.into(), topic.into())])
}

/// Compare complete validated snapshots without mutating source data or drafts.
/// Exact framework hashes permit same-ID matching; changed frameworks require
/// explicit nonempty reviewed correspondence bound to both exact resource pairs.
/// Every supplied correspondence must explicitly pair both sides of each ID
/// surviving in both inventories (including an optional exact-resource map). Unpaired controls absent from the opposite inventory
/// are reported as additions/removals without inferring successor relationships.
/// Explicit pack control/topic dependencies are tracked even without a current gap.
/// # Errors
/// Returns an error for invalid snapshots, correspondence or exceeded bounds.
/// Missing correspondence produces an unsupported report with no unaffected claims.
#[allow(clippy::too_many_lines)] // One ordered comparison preserves the independent change axes.
pub fn compare(
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    correspondence: Option<&ReviewedCorrespondence>,
) -> Result<ImpactReport, ForgeError> {
    let old_index = index(old)?;
    let new_index = index(new)?;
    let mut report = empty_report(
        &old.loaded.project_sha256,
        &new.loaded.project_sha256,
        Some(&old.loaded.report_sha256),
        Some(&new.loaded.report_sha256),
    );
    let old_framework = &old.loaded.baseline_report.framework;
    let new_framework = &new.loaded.baseline_report.framework;
    let exact_framework = old_framework.resource_type == new_framework.resource_type
        && old_framework.raw_sha256 == new_framework.raw_sha256
        && old_framework.resolved_catalog_sha256 == new_framework.resolved_catalog_sha256;
    let correspondence_map: BTreeMap<String, String> = if let Some(reviewed) = correspondence {
        validate_correspondence(reviewed)?;
        if reviewed.old_framework_sha256 != old_framework.raw_sha256
            || reviewed.new_framework_sha256 != new_framework.raw_sha256
            || reviewed.old_resolved_catalog_sha256 != old_framework.resolved_catalog_sha256
            || reviewed.new_resolved_catalog_sha256 != new_framework.resolved_catalog_sha256
        {
            return Err(error(
                "reviewed correspondence does not bind both exact framework resources",
            ));
        }
        for pair in &reviewed.controls {
            if !old.control_fingerprints.contains_key(&pair.old_control_id)
                || !new.control_fingerprints.contains_key(&pair.new_control_id)
            {
                return Err(error(
                    "reviewed correspondence names a control absent from its validated framework",
                ));
            }
        }
        report.correspondence_sha256 = Some(digest(reviewed)?);
        reviewed
            .controls
            .iter()
            .map(|pair| (pair.old_control_id.clone(), pair.new_control_id.clone()))
            .collect()
    } else if exact_framework {
        old.control_fingerprints
            .keys()
            .filter(|key| new.control_fingerprints.contains_key(*key))
            .map(|key| (key.clone(), key.clone()))
            .collect()
    } else {
        report.status = ComparisonStatus::Unsupported;
        add_finding(
            &mut report,
            ChangeCategory::UnsupportedCorrespondence,
            "frameworks",
            ChangeAxes::default(),
            Some(old_framework.raw_sha256.clone()),
            Some(new_framework.raw_sha256.clone()),
            None,
            None,
            BTreeSet::new(),
        )?;
        return Ok(report);
    };
    if correspondence.is_some() {
        let mapped_new: BTreeSet<_> = correspondence_map.values().collect();
        let uncovered = old
            .control_fingerprints
            .keys()
            .filter(|id| new.control_fingerprints.contains_key(*id))
            .any(|id| !correspondence_map.contains_key(id) || !mapped_new.contains(id));
        if uncovered {
            report.status = ComparisonStatus::Unsupported;
            add_finding(
                &mut report,
                ChangeCategory::UnsupportedCorrespondence,
                "unpaired-surviving-controls",
                ChangeAxes::default(),
                Some(old_framework.raw_sha256.clone()),
                Some(new_framework.raw_sha256.clone()),
                None,
                None,
                BTreeSet::new(),
            )?;
            return Ok(report);
        }
    }
    if old.loaded.project.project_key != new.loaded.project.project_key {
        report.status = ComparisonStatus::Unsupported;
        add_finding(
            &mut report,
            ChangeCategory::UnsupportedCorrespondence,
            "project-identity",
            ChangeAxes::default(),
            Some(old.loaded.project_sha256.clone()),
            Some(new.loaded.project_sha256.clone()),
            None,
            None,
            BTreeSet::new(),
        )?;
        return Ok(report);
    }
    if old.loaded.project_sha256 != new.loaded.project_sha256 {
        add_finding(
            &mut report,
            ChangeCategory::ProjectBindingChanged,
            "project",
            axes(false, true, false),
            Some(old.loaded.project_sha256.clone()),
            Some(new.loaded.project_sha256.clone()),
            None,
            None,
            BTreeSet::new(),
        )?;
    }
    if old.loaded.project.baseline != new.loaded.project.baseline {
        add_finding(
            &mut report,
            ChangeCategory::BaselineBindingChanged,
            "baseline",
            axes(false, true, false),
            Some(digest(&old.loaded.project.baseline)?),
            Some(digest(&new.loaded.project.baseline)?),
            None,
            None,
            BTreeSet::new(),
        )?;
    }
    compare_controls(&mut report, old, new, &old_index, &new_index, &correspondence_map)?;
    compare_gaps(&mut report, old, new, &old_index, &new_index, &correspondence_map)?;
    compare_questions(&mut report, old, new, &old_index, &new_index)?;
    compare_clauses(&mut report, old, new)?;
    compare_components(&mut report, old, new)?;
    if old.component_manifest_sha256 != new.component_manifest_sha256 {
        add_finding(
            &mut report,
            ChangeCategory::ComponentExtensionBindingChanged,
            "component-extension",
            axes(false, true, false),
            old.component_manifest_sha256.clone(),
            new.component_manifest_sha256.clone(),
            None,
            None,
            BTreeSet::new(),
        )?;
    }
    if old.loaded.pack_sha256 != new.loaded.pack_sha256 {
        // Individual question, section and assignment findings describe semantic
        // changes. The raw pack pin itself is global provenance; reviewer names,
        // rights assertions and version labels do not manufacture drafting content.
        add_finding(
            &mut report,
            ChangeCategory::PackChanged,
            "pack",
            axes(false, true, false),
            Some(old.loaded.pack_sha256.clone()),
            Some(new.loaded.pack_sha256.clone()),
            None,
            None,
            BTreeSet::new(),
        )?;
    }
    compare_sections(&mut report, old, new, &old_index, &new_index)?;
    compare_policies(&mut report, old, new)?;
    report.findings.sort_by(|a, b| a.finding_id.cmp(&b.finding_id));
    // IDs attach causal evidence without copying it into every result.
    finish_impacts(&mut report, old, new, &old_index, &new_index)?;
    Ok(report)
}

fn compare_controls(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    old_index: &SnapshotIndex<'_>,
    new_index: &SnapshotIndex<'_>,
    mapping: &BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    for (old_id, new_id) in mapping {
        let old_sha = &old.control_fingerprints[old_id];
        let new_sha = &new.control_fingerprints[new_id];
        if old_sha != new_sha {
            add_finding(
                report,
                ChangeCategory::FrameworkControlChanged,
                old_id,
                axes(true, true, false),
                Some(old_sha.clone()),
                Some(new_sha.clone()),
                None,
                None,
                union_sections(
                    old_index.control_sections.get(old_id),
                    new_index.control_sections.get(new_id),
                ),
            )?;
        }
    }
    let mapped_new: BTreeSet<_> = mapping.values().collect();
    let old_gap_ids: BTreeMap<_, _> =
        old.plan.gaps.iter().map(|gap| (&gap.control_id, &gap.gap_id)).collect();
    let new_gap_ids: BTreeMap<_, _> =
        new.plan.gaps.iter().map(|gap| (&gap.control_id, &gap.gap_id)).collect();
    for (control, sha) in &old.control_fingerprints {
        if !mapping.contains_key(control) {
            add_finding(
                report,
                ChangeCategory::FrameworkControlRemoved,
                control,
                axes(true, true, true),
                Some(sha.clone()),
                None,
                old_gap_ids.get(control).map(|gap| (*gap).clone()),
                None,
                old_index.control_sections.get(control).cloned().unwrap_or_default(),
            )?;
        }
    }
    for (control, sha) in &new.control_fingerprints {
        if !mapped_new.contains(control) {
            add_finding(
                report,
                ChangeCategory::FrameworkControlAdded,
                control,
                axes(true, true, true),
                None,
                Some(sha.clone()),
                None,
                new_gap_ids.get(control).map(|gap| (*gap).clone()),
                new_index.control_sections.get(control).cloned().unwrap_or_default(),
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // One ordered comparison preserves the independent change axes.
fn compare_gaps(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    old_index: &SnapshotIndex<'_>,
    new_index: &SnapshotIndex<'_>,
    mapping: &BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    let old_gaps: BTreeMap<_, _> = old.plan.gaps.iter().map(|gap| (&gap.control_id, gap)).collect();
    let new_gaps: BTreeMap<_, _> = new.plan.gaps.iter().map(|gap| (&gap.control_id, gap)).collect();
    let mut matched = BTreeSet::new();
    for (old_control, old_gap) in &old_gaps {
        let new_control = mapping.get(*old_control);
        let new_gap = new_control.and_then(|id| new_gaps.get(id));
        if let Some(new_gap) = new_gap {
            matched.insert(&new_gap.control_id);
            let sections = union_sections(
                old_index.control_sections.get(*old_control),
                new_index.control_sections.get(&new_gap.control_id),
            );
            let old_id = Some(old_gap.gap_id.clone());
            let new_id = Some(new_gap.gap_id.clone());
            if old_gap.gap_id != new_gap.gap_id {
                add_finding(
                    report,
                    ChangeCategory::GapBindingChanged,
                    old_control,
                    axes(false, true, false),
                    Some(old.loaded.report_sha256.clone()),
                    Some(new.loaded.report_sha256.clone()),
                    old_id.clone(),
                    new_id.clone(),
                    sections.clone(),
                )?;
            }
            if old_gap.classification != new_gap.classification {
                add_finding(
                    report,
                    ChangeCategory::GapClassificationChanged,
                    old_control,
                    axes(false, true, true),
                    Some(digest(&old_gap.classification)?),
                    Some(digest(&new_gap.classification)?),
                    old_id.clone(),
                    new_id.clone(),
                    sections.clone(),
                )?;
            }
            let old_routes: BTreeSet<_> =
                old_gap.assignments.iter().map(|a| (&a.policy_key, &a.topic_key)).collect();
            let new_routes: BTreeSet<_> =
                new_gap.assignments.iter().map(|a| (&a.policy_key, &a.topic_key)).collect();
            let old_assignment = digest(&old_gap.assignments)?;
            let new_assignment = digest(&new_gap.assignments)?;
            if old_assignment != new_assignment {
                add_finding(
                    report,
                    ChangeCategory::AssignmentChanged,
                    old_control,
                    axes(false, true, old_routes != new_routes),
                    Some(old_assignment),
                    Some(new_assignment),
                    old_id.clone(),
                    new_id.clone(),
                    sections.clone(),
                )?;
            }
            // Exact report-bound IDs are excluded from the deferral's substantive comparison.
            let old_deferral =
                old_gap.deferral.as_ref().map(|d| (&d.key, &d.review, &d.revisit_date));
            let new_deferral =
                new_gap.deferral.as_ref().map(|d| (&d.key, &d.review, &d.revisit_date));
            if old_deferral != new_deferral {
                let state_changed = old_gap.disposition != new_gap.disposition
                    || old_gap.deferral.as_ref().map(|d| &d.revisit_date)
                        != new_gap.deferral.as_ref().map(|d| &d.revisit_date);
                add_finding(
                    report,
                    ChangeCategory::DeferralChanged,
                    old_control,
                    axes(false, true, state_changed),
                    old_deferral.as_ref().map(digest).transpose()?,
                    new_deferral.as_ref().map(digest).transpose()?,
                    old_id,
                    new_id,
                    sections,
                )?;
            }
        } else {
            add_finding(
                report,
                ChangeCategory::GapRemoved,
                old_control,
                axes(false, true, true),
                Some(digest(old_gap)?),
                None,
                Some(old_gap.gap_id.clone()),
                None,
                old_index.control_sections.get(*old_control).cloned().unwrap_or_default(),
            )?;
        }
    }
    for (control, gap) in new_gaps {
        if !matched.contains(control) {
            add_finding(
                report,
                ChangeCategory::GapAdded,
                control,
                axes(false, true, true),
                None,
                Some(digest(gap)?),
                None,
                Some(gap.gap_id.clone()),
                new_index.control_sections.get(control).cloned().unwrap_or_default(),
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // One ordered comparison preserves the independent change axes.
fn compare_questions(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    old_index: &SnapshotIndex<'_>,
    new_index: &SnapshotIndex<'_>,
) -> Result<(), ForgeError> {
    let old_defs: BTreeMap<_, _> = old.loaded.pack.questions.iter().map(|q| (&q.key, q)).collect();
    let new_defs: BTreeMap<_, _> = new.loaded.pack.questions.iter().map(|q| (&q.key, q)).collect();
    let old_answers: BTreeMap<_, _> =
        old.loaded.project.answers.iter().map(|a| (&a.question_key, a)).collect();
    let new_answers: BTreeMap<_, _> =
        new.loaded.project.answers.iter().map(|a| (&a.question_key, a)).collect();
    let old_states: BTreeMap<_, _> =
        old.plan.questions.iter().map(|q| (&q.question_key, q)).collect();
    let new_states: BTreeMap<_, _> =
        new.plan.questions.iter().map(|q| (&q.question_key, q)).collect();
    let keys: BTreeSet<_> = old_defs.keys().chain(new_defs.keys()).copied().collect();
    for key in keys {
        let sections = union_sections(
            old_index.question_sections.get(key),
            new_index.question_sections.get(key),
        );
        let left = old_defs.get(key).copied();
        let right = new_defs.get(key).copied();
        if left != right {
            add_finding(
                report,
                ChangeCategory::QuestionDefinitionChanged,
                key,
                axes(true, true, false),
                left.map(manifest::question_sha256).transpose()?,
                right.map(manifest::question_sha256).transpose()?,
                None,
                None,
                sections.clone(),
            )?;
        }
        let left = old_answers.get(key).copied();
        let right = new_answers.get(key).copied();
        if left != right {
            let content_changed =
                left.and_then(|a| a.value.as_ref()) != right.and_then(|a| a.value.as_ref());
            add_finding(
                report,
                ChangeCategory::AnswerChanged,
                key,
                axes(content_changed, true, false),
                left.map(manifest::answer_sha256).transpose()?,
                right.map(manifest::answer_sha256).transpose()?,
                None,
                None,
                sections.clone(),
            )?;
        }
        let old_expiry = (
            left.and_then(|a| a.expires_at.as_ref()),
            old_defs
                .get(key)
                .and_then(|q| q.max_age_days)
                .map(|age| (age, left.map(|a| &a.review.reviewed_at))),
        );
        let new_expiry = (
            right.and_then(|a| a.expires_at.as_ref()),
            new_defs
                .get(key)
                .and_then(|q| q.max_age_days)
                .map(|age| (age, right.map(|a| &a.review.reviewed_at))),
        );
        if old_expiry != new_expiry {
            add_finding(
                report,
                ChangeCategory::AnswerExpiryChanged,
                key,
                axes(false, true, true),
                Some(digest(&old_expiry)?),
                Some(digest(&new_expiry)?),
                None,
                None,
                sections.clone(),
            )?;
        }
        let old_state = old_states.get(key).map(|q| q.state);
        let new_state = new_states.get(key).map(|q| q.state);
        if old_state != new_state {
            add_finding(
                report,
                ChangeCategory::AnswerStateChanged,
                key,
                axes(false, false, true),
                old_state.as_ref().map(digest).transpose()?,
                new_state.as_ref().map(digest).transpose()?,
                None,
                None,
                sections,
            )?;
        }
    }
    Ok(())
}

fn compare_clauses(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
) -> Result<(), ForgeError> {
    let left: BTreeMap<_, _> =
        old.loaded.project.human_clauses.iter().map(|c| (&c.key, c)).collect();
    let right: BTreeMap<_, _> =
        new.loaded.project.human_clauses.iter().map(|c| (&c.key, c)).collect();
    for key in left.keys().chain(right.keys()).copied().collect::<BTreeSet<_>>() {
        let old = left.get(key).copied();
        let new = right.get(key).copied();
        if old == new {
            continue;
        }
        let sections = old
            .into_iter()
            .chain(new)
            .map(|c| (c.policy_key.clone(), c.topic_key.clone()))
            .collect();
        let content =
            old.map(|c| &c.source.expected_sha256) != new.map(|c| &c.source.expected_sha256);
        let state = old.map(|c| (&c.policy_key, &c.topic_key))
            != new.map(|c| (&c.policy_key, &c.topic_key));
        add_finding(
            report,
            ChangeCategory::HumanClauseChanged,
            key,
            axes(content, true, state),
            old.map(digest).transpose()?,
            new.map(digest).transpose()?,
            None,
            None,
            sections,
        )?;
    }
    Ok(())
}

fn compare_components(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
) -> Result<(), ForgeError> {
    let left: BTreeMap<_, _> = old.components.iter().map(|c| (&c.instance_key, c)).collect();
    let right: BTreeMap<_, _> = new.components.iter().map(|c| (&c.instance_key, c)).collect();
    for key in left.keys().chain(right.keys()).copied().collect::<BTreeSet<_>>() {
        let old = left.get(key).copied();
        let new = right.get(key).copied();
        let old_hash = old.map(digest).transpose()?;
        let new_hash = new.map(digest).transpose()?;
        if old_hash == new_hash {
            continue;
        }
        let sections = old
            .into_iter()
            .chain(new)
            .map(|c| (c.policy_key.clone(), c.topic_key.clone()))
            .collect();
        let source = old.map(|c| &c.source_sha256) != new.map(|c| &c.source_sha256);
        let old_rendered = old.and_then(|c| c.rendered_sha256.as_ref());
        let new_rendered = new.and_then(|c| c.rendered_sha256.as_ref());
        let rendered = matches!((old_rendered,new_rendered),(Some(a),Some(b)) if a != b);
        let availability = old_rendered.is_some() != new_rendered.is_some();
        let sidecar = old.map(|c| &c.sidecar_sha256) != new.map(|c| &c.sidecar_sha256);
        let state = old.map(|c| (&c.policy_key, &c.topic_key))
            != new.map(|c| (&c.policy_key, &c.topic_key));
        // A changed sidecar may alter constraints even when bytes currently agree;
        // report the component change without claiming substantive draft changes.
        let category = if source || sidecar || rendered {
            ChangeCategory::ComponentChanged
        } else {
            ChangeCategory::ComponentBindingChanged
        };
        add_finding(
            report,
            category,
            key,
            axes(source || rendered, true, state || availability),
            old_hash,
            new_hash,
            None,
            None,
            sections,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // One ordered comparison preserves the independent change axes.
fn compare_sections(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    old_index: &SnapshotIndex<'_>,
    new_index: &SnapshotIndex<'_>,
) -> Result<(), ForgeError> {
    let keys: BTreeSet<_> = old_index.sections.keys().chain(new_index.sections.keys()).collect();
    for key in keys {
        let left = old_index.sections.get(key).copied();
        let right = new_index.sections.get(key).copied();
        let sections = section_set(&key.0, &key.1);
        // The topic key is always contextualized by affected_sections and the finding hash.
        let subject = &key.1;
        match (left, right) {
            (None, Some(_)) => add_finding(
                report,
                ChangeCategory::SectionAdded,
                subject,
                axes(true, true, true),
                None,
                new.sections.get(key).cloned(),
                None,
                None,
                sections.clone(),
            )?,
            (Some(_), None) => add_finding(
                report,
                ChangeCategory::SectionRemoved,
                subject,
                axes(true, true, true),
                old.sections.get(key).cloned(),
                None,
                None,
                None,
                sections.clone(),
            )?,
            (Some(left), Some(right)) => {
                let old_definition = (&left.title, left.order);
                let new_definition = (&right.title, right.order);
                if old_definition != new_definition {
                    add_finding(
                        report,
                        ChangeCategory::SectionDefinitionChanged,
                        subject,
                        axes(true, true, false),
                        Some(digest(&old_definition)?),
                        Some(digest(&new_definition)?),
                        None,
                        None,
                        sections.clone(),
                    )?;
                }
                let old_dependencies = section_dependencies(left)?;
                let new_dependencies = section_dependencies(right)?;
                if old_dependencies != new_dependencies {
                    add_finding(
                        report,
                        ChangeCategory::DependencyProvenanceChanged,
                        subject,
                        axes(false, true, false),
                        Some(old_dependencies),
                        Some(new_dependencies),
                        None,
                        None,
                        sections.clone(),
                    )?;
                }
                if left.state != right.state {
                    add_finding(
                        report,
                        ChangeCategory::AuthoringStateChanged,
                        subject,
                        axes(false, false, true),
                        Some(digest(&left.state)?),
                        Some(digest(&right.state)?),
                        None,
                        None,
                        sections.clone(),
                    )?;
                }
            }
            (None, None) => unreachable!("union only includes present sections"),
        }
        if old.sections.get(key) != new.sections.get(key) {
            let changed = ChangeAxes { output_bytes_changed: true, ..ChangeAxes::default() };
            add_finding(
                report,
                ChangeCategory::OutputBytesChanged,
                subject,
                changed,
                old.sections.get(key).cloned(),
                new.sections.get(key).cloned(),
                None,
                None,
                sections,
            )?;
        }
    }
    Ok(())
}

fn section_dependencies(section: &super::model::SectionPlan) -> Result<String, ForgeError> {
    // The explicit section state is tracked on its own axis. Exact binding hashes
    // remain visible even when semantic content and emitted bytes are unchanged.
    let questions: Vec<_> = section
        .questions
        .iter()
        .map(|q| {
            (
                &q.question_key,
                &q.question_sha256,
                q.required,
                &q.answer_key,
                &q.answer_sha256,
                &q.review,
                &q.expires_at,
            )
        })
        .collect();
    digest(&(
        &section.gap_ids,
        &section.control_ids,
        &section.assignments,
        &section.clause_keys,
        questions,
    ))
}

fn compare_policies(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
) -> Result<(), ForgeError> {
    let left: BTreeMap<_, _> = old.plan.policies.iter().map(|p| (&p.policy_key, p)).collect();
    let right: BTreeMap<_, _> = new.plan.policies.iter().map(|p| (&p.policy_key, p)).collect();
    for key in left.keys().chain(right.keys()).copied().collect::<BTreeSet<_>>() {
        let old_policy = left.get(key).copied();
        let new_policy = right.get(key).copied();
        let sections: BTreeSet<_> = old_policy
            .into_iter()
            .chain(new_policy)
            .flat_map(|p| p.sections.iter().map(|s| (key.clone(), s.topic_key.clone())))
            .collect();
        let category = match (old_policy, new_policy) {
            (None, Some(_)) => Some(ChangeCategory::PolicyAdded),
            (Some(_), None) => Some(ChangeCategory::PolicyRemoved),
            (Some(a), Some(b))
                if (&a.title, &a.policy_family_key) != (&b.title, &b.policy_family_key) =>
            {
                Some(ChangeCategory::PolicyDefinitionChanged)
            }
            _ => None,
        };
        if let Some(category) = category {
            add_finding(
                report,
                category,
                key,
                axes(
                    true,
                    true,
                    old_policy.map(|p| &p.policy_family_key)
                        != new_policy.map(|p| &p.policy_family_key),
                ),
                old_policy.map(|p| digest(&(&p.title, &p.policy_family_key))).transpose()?,
                new_policy.map(|p| digest(&(&p.title, &p.policy_family_key))).transpose()?,
                None,
                None,
                sections,
            )?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)] // One ordered comparison preserves the independent change axes.
fn finish_impacts(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    old_index: &SnapshotIndex<'_>,
    new_index: &SnapshotIndex<'_>,
) -> Result<(), ForgeError> {
    reserve_scope_evidence(report, old, new, old_index, new_index)?;
    let mut section_causes: BTreeMap<SectionKey, (ChangeAxes, BTreeSet<String>)> = BTreeMap::new();
    let mut policy_causes: BTreeMap<String, (ChangeAxes, BTreeSet<String>)> = BTreeMap::new();
    for finding in &report.findings {
        for section in &finding.affected_sections {
            let value = section_causes
                .entry((section.policy_key.clone(), section.topic_key.clone()))
                .or_default();
            merge_axes(&mut value.0, &finding.axes);
            value.1.insert(finding.finding_id.clone());
            let value = policy_causes.entry(section.policy_key.clone()).or_default();
            merge_axes(&mut value.0, &finding.axes);
            value.1.insert(finding.finding_id.clone());
        }
        if matches!(
            finding.category,
            ChangeCategory::PolicyAdded
                | ChangeCategory::PolicyRemoved
                | ChangeCategory::PolicyDefinitionChanged
        ) {
            let value = policy_causes.entry(finding.subject_key.clone()).or_default();
            merge_axes(&mut value.0, &finding.axes);
            value.1.insert(finding.finding_id.clone());
        }
    }
    for key in old_index.sections.keys().chain(new_index.sections.keys()).collect::<BTreeSet<_>>() {
        let (mut changes, ids) = section_causes.remove(key).unwrap_or_default();
        changes.output_bytes_changed = old.sections.get(key) != new.sections.get(key);
        report.sections.push(SectionImpact {
            policy_key: key.0.clone(),
            topic_key: key.1.clone(),
            old_output_sha256: old.sections.get(key).cloned(),
            new_output_sha256: new.sections.get(key).cloned(),
            unaffected: unchanged(&changes),
            axes: changes,
            finding_ids: ids.into_iter().collect(),
        });
    }
    for key in old.policies.keys().chain(new.policies.keys()).collect::<BTreeSet<_>>() {
        let (mut changes, ids) = policy_causes.remove(key).unwrap_or_default();
        changes.output_bytes_changed = old.policies.get(key) != new.policies.get(key);
        report.policies.push(PolicyImpact {
            policy_key: key.clone(),
            old_output_sha256: old.policies.get(key).cloned(),
            new_output_sha256: new.policies.get(key).cloned(),
            unaffected: unchanged(&changes),
            axes: changes,
            finding_ids: ids.into_iter().collect(),
        });
    }
    Ok(())
}

fn reserve_scope_evidence(
    report: &mut ImpactReport,
    old: &ImpactSnapshot,
    new: &ImpactSnapshot,
    old_index: &SnapshotIndex<'_>,
    new_index: &SnapshotIndex<'_>,
) -> Result<(), ForgeError> {
    let remaining = super::output::MAX_OUTPUT_BYTES.saturating_sub(report.evidence_bytes);
    let mut additional = 0usize;
    let mut reserve = |bytes: usize| -> Result<(), ForgeError> {
        additional = additional.saturating_add(bytes);
        if additional > remaining {
            return Err(error("impact scope evidence exceeds remaining aggregate byte budget"));
        }
        Ok(())
    };
    // Visit the union without allocating another set of owned scope labels.
    for key in old_index
        .sections
        .keys()
        .chain(new_index.sections.keys().filter(|key| !old_index.sections.contains_key(*key)))
    {
        reserve(
            512usize.saturating_add(key.0.len().saturating_add(key.1.len()).saturating_mul(6)),
        )?;
    }
    for key in old
        .policies
        .keys()
        .chain(new.policies.keys().filter(|key| !old.policies.contains_key(*key)))
    {
        reserve(512usize.saturating_add(key.len().saturating_mul(6)))?;
    }
    for finding in &report.findings {
        // An edge can create both section and policy finding-ID references.
        reserve(finding.affected_sections.len().saturating_mul(2 * 96))?;
        if matches!(
            finding.category,
            ChangeCategory::PolicyAdded
                | ChangeCategory::PolicyRemoved
                | ChangeCategory::PolicyDefinitionChanged
        ) {
            reserve(96)?;
        }
    }
    report.evidence_bytes = report.evidence_bytes.saturating_add(additional);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applicability::model::GapClassification;
    use crate::authoring::model::AnswerStatus;
    use serde_json::json;

    fn load_example() -> LoadedAuthorProject {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/authoring");
        super::super::input::prepare(&root.join("project.json")).unwrap().loaded
    }

    fn refresh(loaded: &mut LoadedAuthorProject, repin_answers: bool) {
        loaded.project.baseline.report_sha256 = loaded.report_sha256.clone();
        loaded.project.gap_report.expected_sha256 = loaded.report_sha256.clone();
        loaded.pack.baseline = loaded.project.baseline.clone();
        loaded.pack_sha256 = digest(&loaded.pack).unwrap();
        loaded.project.authoring_pack.expected_sha256 = loaded.pack_sha256.clone();
        if repin_answers {
            for answer in &mut loaded.project.answers {
                answer.authoring_pack_sha256 = loaded.pack_sha256.clone();
                answer.question_sha256 = manifest::question_sha256(
                    loaded.pack.questions.iter().find(|q| q.key == answer.question_key).unwrap(),
                )
                .unwrap();
            }
        }
        for clause in &mut loaded.project.human_clauses {
            for pin in &mut clause.answer_refs {
                pin.expected_sha256 = manifest::answer_sha256(
                    loaded.project.answers.iter().find(|a| a.key == pin.answer_key).unwrap(),
                )
                .unwrap();
            }
            loaded.clauses.get_mut(&clause.key).unwrap().source = clause.clone();
        }
        loaded.project_sha256 = digest(&loaded.project).unwrap();
    }

    fn snapshot(loaded: LoadedAuthorProject) -> ImpactSnapshot {
        let plan = super::super::plan::build_plan(&loaded).unwrap();
        let rendered = super::super::render::render(&loaded, &plan).unwrap();
        let provenance: Value = serde_json::from_slice(&rendered.provenance).unwrap();
        let policies = rendered
            .policies
            .iter()
            .map(|p| (p.policy_key.clone(), sha256_hex(&p.markdown)))
            .collect();
        let mut section_bytes: BTreeMap<SectionKey, Vec<u8>> = BTreeMap::new();
        for policy in provenance["policies"].as_array().unwrap() {
            let key = policy["policy_key"].as_str().unwrap();
            let bytes = &rendered.policies.iter().find(|p| p.policy_key == key).unwrap().markdown;
            for span in policy["spans"].as_array().unwrap() {
                if let Some(topic) = span["origin"]["topic_key"].as_str() {
                    let start = usize::try_from(span["output"]["start"].as_u64().unwrap()).unwrap();
                    let end = usize::try_from(span["output"]["end"].as_u64().unwrap()).unwrap();
                    section_bytes
                        .entry((key.into(), topic.into()))
                        .or_default()
                        .extend_from_slice(&bytes[start..end]);
                }
            }
        }
        let sections =
            section_bytes.into_iter().map(|(key, bytes)| (key, sha256_hex(&bytes))).collect();
        let control_fingerprints = loaded
            .baseline_report
            .controls
            .iter()
            .map(|c| (c.control_id.clone(), sha256_hex(c.control_id.as_bytes())))
            .collect();
        ImpactSnapshot {
            loaded,
            plan,
            policies,
            sections,
            control_fingerprints,
            components: Vec::new(),
            component_manifest_sha256: None,
        }
    }
    fn has(report: &ImpactReport, category: ChangeCategory) -> bool {
        report.findings.iter().any(|f| f.category == category)
    }
    fn section<'a>(report: &'a ImpactReport, topic: &str) -> &'a SectionImpact {
        report.sections.iter().find(|s| s.topic_key == topic).unwrap()
    }
    fn pair() -> (ImpactSnapshot, LoadedAuthorProject) {
        let mut loaded = load_example();
        refresh(&mut loaded, true);
        (snapshot(loaded.clone()), loaded)
    }

    #[test]
    fn no_op_has_identical_bytes_dependencies_and_no_findings() {
        let (old, new) = pair();
        let new = snapshot(new);
        let report = compare(&old, &new, None).unwrap();
        assert!(report.is_complete());
        assert!(report.findings.is_empty());
        assert!(report.policies.iter().all(|p| p.unaffected));
        assert!(report.sections.iter().all(|s| s.unaffected));
        assert_eq!(
            render_json(&report).unwrap(),
            render_json(&compare(&old, &new, None).unwrap()).unwrap()
        );
    }

    #[test]
    fn report_hash_churn_retains_gap_correspondence_and_exact_old_new_ids() {
        let (old, mut loaded) = pair();
        loaded.report_sha256 = sha256_hex(b"same validated report with different whitespace");
        for clause in &mut loaded.project.human_clauses {
            clause.gap_ids = vec![manifest::gap_id(&loaded.report_sha256, "sample-1")];
        }
        refresh(&mut loaded, true);
        let new = snapshot(loaded);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::BaselineBindingChanged));
        assert!(!has(&report, ChangeCategory::GapAdded));
        assert!(!has(&report, ChangeCategory::GapRemoved));
        assert!(
            report
                .sections
                .iter()
                .all(|s| !s.axes.substantive_changed && !s.axes.output_bytes_changed)
        );
        assert!(report.sections.iter().all(|s| s.axes.provenance_changed));
        assert_ne!(old.plan.gaps[0].gap_id, new.plan.gaps[0].gap_id);
    }

    #[test]
    fn unrelated_frameworks_cannot_match_identical_control_ids_or_uuid() {
        let (old, mut loaded) = pair();
        loaded.baseline_report.framework.raw_sha256 = sha256_hex(b"unrelated framework");
        loaded.project.baseline.framework_sha256 =
            loaded.baseline_report.framework.raw_sha256.clone();
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert_eq!(report.status, ComparisonStatus::Unsupported);
        assert!(report.policies.is_empty() && report.sections.is_empty());
        assert!(has(&report, ChangeCategory::UnsupportedCorrespondence));
    }

    fn correspondence(old: &ImpactSnapshot, new: &ImpactSnapshot) -> ReviewedCorrespondence {
        ReviewedCorrespondence {
            old_framework_sha256: old.loaded.baseline_report.framework.raw_sha256.clone(),
            new_framework_sha256: new.loaded.baseline_report.framework.raw_sha256.clone(),
            old_resolved_catalog_sha256: None,
            new_resolved_catalog_sha256: None,
            reviewers: old.loaded.project.reviewers.clone(),
            review: old.loaded.project.baseline_review.clone(),
            controls: vec![
                ControlCorrespondence {
                    old_control_id: "sample-1".into(),
                    new_control_id: "sample-1".into(),
                },
                ControlCorrespondence {
                    old_control_id: "sample-2".into(),
                    new_control_id: "sample-2".into(),
                },
            ],
        }
    }

    #[test]
    fn reviewed_exact_correspondence_targets_only_changed_control_dependencies() {
        let (old, mut loaded) = pair();
        loaded.baseline_report.framework.raw_sha256 = sha256_hex(b"revised framework");
        loaded.project.baseline.framework_sha256 =
            loaded.baseline_report.framework.raw_sha256.clone();
        refresh(&mut loaded, true);
        let mut new = snapshot(loaded);
        new.control_fingerprints.insert("sample-1".into(), sha256_hex(b"changed control"));
        let reviewed = correspondence(&old, &new);
        let report = compare(&old, &new, Some(&reviewed)).unwrap();
        assert!(report.is_complete());
        assert!(has(&report, ChangeCategory::FrameworkControlChanged));
        assert!(section(&report, "sample-topic").axes.substantive_changed);
        assert!(section(&report, "independent-topic").unaffected);
        let mut invalid = reviewed.clone();
        invalid.new_framework_sha256 = "0".repeat(64);
        assert!(compare(&old, &new, Some(&invalid)).is_err());
        invalid = reviewed.clone();
        invalid.controls[0].new_control_id = "missing".into();
        assert!(compare(&old, &new, Some(&invalid)).is_err());
        invalid = reviewed;
        invalid.controls.push(invalid.controls[0].clone());
        assert!(compare(&old, &new, Some(&invalid)).is_err());
    }

    #[test]
    fn answer_value_changes_are_private_and_do_not_affect_unrelated_sections() {
        let (old, mut loaded) = pair();
        loaded.project.answers[0].value = Some(json!("PRIVATE-CHANGED-ANSWER"));
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::AnswerChanged));
        assert!(section(&report, "sample-topic").axes.substantive_changed);
        assert!(section(&report, "independent-topic").unaffected);
        assert!(
            !String::from_utf8(render_json(&report).unwrap())
                .unwrap()
                .contains("PRIVATE-CHANGED-ANSWER")
        );
        assert!(!render_text(&report).unwrap().contains("PRIVATE-CHANGED-ANSWER"));
    }

    #[test]
    fn answer_review_changes_are_provenance_only() {
        let (old, mut loaded) = pair();
        loaded.project.answers[0].review.rationale = "Updated asserted review rationale".into();
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::AnswerChanged));
        let changed = section(&report, "sample-topic");
        assert!(
            changed.axes.provenance_changed
                && !changed.axes.substantive_changed
                && !changed.axes.output_bytes_changed
        );
        assert!(section(&report, "independent-topic").unaffected);
    }

    #[test]
    fn stale_and_expired_answers_report_state_and_block_only_dependencies() {
        let (old, mut loaded) = pair();
        loaded.project.answers[0].question_sha256 = "0".repeat(64);
        refresh(&mut loaded, false);
        let new = snapshot(loaded);
        assert_eq!(new.plan.questions[0].state, AnswerStatus::Stale);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::AnswerStateChanged));
        assert!(section(&report, "sample-topic").axes.authoring_state_changed);
        assert!(section(&report, "independent-topic").unaffected);
        let (_, mut loaded) = pair();
        loaded.project.as_of = "2026-12-01T00:00:00Z".into();
        refresh(&mut loaded, true);
        let new = snapshot(loaded);
        assert_eq!(new.plan.questions[0].state, AnswerStatus::Expired);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::AnswerStateChanged));
        assert!(section(&report, "independent-topic").unaffected);
    }

    #[test]
    fn definition_and_explicit_expiry_changes_have_distinct_categories() {
        let (old, mut loaded) = pair();
        loaded.pack.questions[0].max_age_days = Some(5);
        loaded.project.answers[0].expires_at = Some("2026-09-05T00:00:00Z".into());
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::QuestionDefinitionChanged));
        assert!(has(&report, ChangeCategory::AnswerExpiryChanged));
        assert!(has(&report, ChangeCategory::PackChanged));
        assert!(section(&report, "independent-topic").unaffected);
    }

    #[test]
    fn unused_question_change_keeps_unrelated_content_and_bytes_unaffected() {
        let mut loaded = load_example();
        let mut unused = loaded.pack.questions[0].clone();
        unused.key = "unused-question".into();
        loaded.pack.questions.push(unused);
        refresh(&mut loaded, true);
        let old = snapshot(loaded.clone());
        loaded.pack.questions[1].prompt = "Updated unused prompt".into();
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        let definition = report
            .findings
            .iter()
            .find(|f| f.category == ChangeCategory::QuestionDefinitionChanged)
            .unwrap();
        assert!(definition.affected_sections.is_empty());
        assert!(
            report
                .sections
                .iter()
                .all(|s| !s.axes.substantive_changed && !s.axes.output_bytes_changed)
        );
        assert!(section(&report, "independent-topic").unaffected);
    }

    #[test]
    fn gap_addition_and_removal_use_applicability_membership_not_report_hash() {
        let (old, mut loaded) = pair();
        loaded.baseline_report.controls[1].classification = GapClassification::NotApplicable;
        loaded.baseline_report.counts.applicable_unmapped -= 1;
        loaded.baseline_report.counts.not_applicable += 1;
        refresh(&mut loaded, true);
        let new = snapshot(loaded);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::GapRemoved));
        assert!(!has(&report, ChangeCategory::GapAdded));
        let removal =
            report.findings.iter().find(|f| f.category == ChangeCategory::GapRemoved).unwrap();
        assert!(removal.old_gap_id.is_some() && removal.new_gap_id.is_none());
        assert!(!section(&report, "sample-topic").axes.substantive_changed);
        let reverse = compare(&new, &old, None).unwrap();
        assert!(has(&reverse, ChangeCategory::GapAdded));
    }

    #[test]
    fn assignment_review_and_deferral_changes_preserve_reasons() {
        let (old, mut loaded) = pair();
        loaded.pack.control_assignments[1].review.rationale = "Explicitly re-reviewed".into();
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::AssignmentChanged));
        let (old, mut loaded) = pair();
        loaded.pack.control_assignments.retain(|a| a.control_id != "sample-2");
        loaded.project.deferrals.push(manifest::Deferral {
            key: "defer-independent".into(),
            gap_id: manifest::gap_id(&loaded.report_sha256, "sample-2"),
            review: loaded.project.baseline_review.clone(),
            revisit_date: Some("2027-01-01".into()),
        });
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::AssignmentChanged));
        assert!(has(&report, ChangeCategory::DeferralChanged));
        assert!(!section(&report, "sample-topic").axes.substantive_changed);
    }

    #[test]
    fn human_clause_changes_are_exact_and_targeted() {
        let (old, mut loaded) = pair();
        let bytes = "Changed synthetic clause with café.\n".as_bytes().to_vec();
        loaded.project.human_clauses[0].source.expected_sha256 = sha256_hex(&bytes);
        loaded.clauses.get_mut("sample-clause").unwrap().bytes = bytes;
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::HumanClauseChanged));
        assert!(section(&report, "sample-topic").axes.output_bytes_changed);
        assert!(section(&report, "independent-topic").unaffected);
    }

    fn component() -> ComponentSnapshot {
        ComponentSnapshot {
            instance_key: "component-one".into(),
            policy_key: "sample-policy".into(),
            topic_key: "sample-topic".into(),
            source_sha256: sha256_hex(b"component source"),
            sidecar_sha256: sha256_hex(b"component sidecar"),
            record_sha256: sha256_hex(b"component record"),
            rendered_sha256: Some(sha256_hex(b"component rendered")),
            answer_keys: vec!["sample-answer".into()],
            question_keys: vec!["sample-question".into()],
        }
    }

    #[test]
    fn component_changes_bind_instance_identity_and_preserve_provenance_axis() {
        let (mut old, loaded) = pair();
        old.components.push(component());
        let mut new = snapshot(loaded.clone());
        let mut changed = component();
        changed.source_sha256 = sha256_hex(b"new component source");
        new.components.push(changed);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::ComponentChanged));
        assert!(section(&report, "sample-topic").axes.substantive_changed);
        assert!(section(&report, "independent-topic").unaffected);
        let mut new = snapshot(loaded);
        let mut changed = component();
        changed.record_sha256 = sha256_hex(b"new explicit review");
        new.components.push(changed);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::ComponentBindingChanged));
        assert!(!section(&report, "sample-topic").axes.substantive_changed);
        assert!(section(&report, "sample-topic").axes.provenance_changed);
    }

    #[test]
    fn moved_component_marks_old_and_new_destinations() {
        let (mut old, loaded) = pair();
        old.components.push(component());
        let mut new = snapshot(loaded);
        let mut changed = component();
        changed.topic_key = "independent-topic".into();
        new.components.push(changed);
        let report = compare(&old, &new, None).unwrap();
        let finding = report
            .findings
            .iter()
            .find(|f| f.category == ChangeCategory::ComponentBindingChanged)
            .unwrap();
        assert_eq!(finding.affected_sections.len(), 2);
        assert!(report.sections.iter().all(|s| !s.unaffected));
    }

    #[test]
    fn policy_addition_removal_and_empty_policies_have_truthful_results() {
        let (old, mut loaded) = pair();
        loaded.pack.policy_families.push(manifest::PolicyFamily {
            key: "empty-family".into(),
            title: "Explicit empty family".into(),
        });
        loaded.project.policies.push(manifest::Policy {
            key: "empty-policy".into(),
            policy_family_key: "empty-family".into(),
            title: "Empty policy draft".into(),
        });
        refresh(&mut loaded, true);
        let new = snapshot(loaded);
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::PolicyAdded));
        assert!(
            !report.policies.iter().find(|p| p.policy_key == "empty-policy").unwrap().unaffected
        );
        assert!(has(&compare(&new, &old, None).unwrap(), ChangeCategory::PolicyRemoved));
    }

    #[test]
    fn incomplete_drift_comparison_never_claims_unaffected_or_reflects_errors() {
        let report = incomplete_report(&"a".repeat(64), &"b".repeat(64), "new").unwrap();
        assert!(!report.is_complete());
        assert!(report.sections.is_empty() && report.policies.is_empty());
        assert!(has(&report, ChangeCategory::InputUnverified));
        assert!(incomplete_report(&"a".repeat(64), &"b".repeat(64), "private/path/token").is_err());
    }

    fn wire() -> Value {
        json!({"schema_version":MANIFEST_SCHEMA_VERSION,
        "old":{"project":{"path":"old/project.json","expected_sha256":"a".repeat(64)}},
        "new":{"project":{"path":"new/project.json","expected_sha256":"b".repeat(64)}}})
    }

    #[test]
    fn closed_contract_schema_and_runtime_reject_forward_null_unknown_and_duplicate_keys() {
        let schema: Value =
            serde_json::from_slice(include_bytes!("../../schemas/authoring-impact.schema.json"))
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let valid = wire();
        assert!(validator.is_valid(&valid));
        parse_manifest(&serde_json::to_vec(&valid).unwrap()).unwrap();
        for (field, value) in [
            ("schema_version", json!("forge.authoring-impact/2")),
            ("unknown", json!(true)),
            ("correspondence", Value::Null),
        ] {
            let mut invalid = wire();
            invalid[field] = value;
            assert!(!validator.is_valid(&invalid));
            assert!(parse_manifest(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
        let mut invalid = wire();
        invalid["old"]["components"] = Value::Null;
        assert!(!validator.is_valid(&invalid));
        assert!(parse_manifest(&serde_json::to_vec(&invalid).unwrap()).is_err());
        assert!(parse_manifest(br#"{"schema_version":"forge.authoring-impact/1","schema_version":"forge.authoring-impact/1"}"#).is_err());
        for path in
            ["../outside.json", "sub/../project.json", "C:/project.json", "old\\project.json"]
        {
            let mut invalid = wire();
            invalid["old"]["project"]["path"] = json!(path);
            assert!(parse_manifest(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
    }

    #[test]
    fn comparison_rejects_missing_exact_render_or_inventory_evidence() {
        let (old, loaded) = pair();
        let mut new = snapshot(loaded.clone());
        new.sections.clear();
        assert!(compare(&old, &new, None).is_err());
        let mut new = snapshot(loaded);
        new.control_fingerprints.clear();
        assert!(compare(&old, &new, None).is_err());
    }

    #[test]
    fn findings_bind_the_exact_snapshot_hashes() {
        let (old, mut loaded) = pair();
        loaded.project.answers[0].value = Some(json!("changed"));
        refresh(&mut loaded, true);
        let mut new = snapshot(loaded);
        let first = compare(&old, &new, None).unwrap();
        new.loaded.project_sha256 = sha256_hex(b"same semantics different exact project bytes");
        new.plan.provenance.project_sha256 = new.loaded.project_sha256.clone();
        let second = compare(&old, &new, None).unwrap();
        let first_ids: BTreeSet<_> = first.findings.iter().map(|f| &f.finding_id).collect();
        assert!(second.findings.iter().all(|f| !first_ids.contains(&f.finding_id)));
    }

    #[test]
    fn independent_cross_directory_snapshots_have_identical_impact_bytes() {
        let left = tempfile::tempdir().unwrap();
        let right = tempfile::tempdir().unwrap();
        let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/authoring");
        let mut snapshots = Vec::new();
        for root in [left.path(), right.path()] {
            for file in [
                "project.json",
                "pack.json",
                "framework.json",
                "applicability.json",
                "gap-report.json",
                "clause.md",
            ] {
                std::fs::copy(example.join(file), root.join(file)).unwrap();
            }
            let root = root.canonicalize().unwrap();
            snapshots.push(snapshot(
                super::super::input::prepare(&root.join("project.json")).unwrap().loaded,
            ));
        }
        let first = compare(&snapshots[0], &snapshots[0], None).unwrap();
        let second = compare(&snapshots[1], &snapshots[1], None).unwrap();
        assert_eq!(render_json(&first).unwrap(), render_json(&second).unwrap());
    }
    #[test]
    fn review_time_without_expiry_dependency_is_only_provenance() {
        let mut loaded = load_example();
        loaded.pack.questions[0].max_age_days = None;
        refresh(&mut loaded, true);
        let old = snapshot(loaded.clone());
        loaded.project.answers[0].review.reviewed_at = "2026-09-02T00:00:00Z".into();
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::AnswerChanged));
        assert!(!has(&report, ChangeCategory::AnswerExpiryChanged));
        assert!(!section(&report, "sample-topic").axes.authoring_state_changed);
        assert!(!section(&report, "sample-topic").axes.substantive_changed);
    }

    #[test]
    fn component_expiry_block_and_unblock_do_not_invent_substantive_changes() {
        let (mut old, mut loaded) = pair();
        old.components.push(component());
        loaded.project.as_of = "2026-12-01T00:00:00Z".into();
        refresh(&mut loaded, true);
        let mut new = snapshot(loaded);
        let mut blocked = component();
        blocked.rendered_sha256 = None;
        new.components.push(blocked);
        for report in [compare(&old, &new, None).unwrap(), compare(&new, &old, None).unwrap()] {
            assert!(section(&report, "sample-topic").axes.authoring_state_changed);
            assert!(!section(&report, "sample-topic").axes.substantive_changed);
            assert!(section(&report, "independent-topic").unaffected);
        }
    }

    #[test]
    fn known_drift_and_component_failures_have_closed_private_reasons() {
        for (reason, category) in [
            (UnverifiedReason::InputDrift, ChangeCategory::InputDrift),
            (UnverifiedReason::MissingInput, ChangeCategory::InputMissing),
            (UnverifiedReason::ComponentInput, ChangeCategory::ComponentInputUnverified),
        ] {
            let report =
                incomplete_report_with_reason(&"a".repeat(64), &"b".repeat(64), "new", reason)
                    .unwrap();
            assert_eq!(report.findings[0].category, category);
            assert_eq!(report.findings[0].unverified_reason, Some(reason));
            assert!(report.sections.is_empty() && report.policies.is_empty());
        }
    }
    #[test]
    fn exact_request_binding_updates_all_references_and_is_idempotent() {
        let (old, mut loaded) = pair();
        loaded.project.answers[0].value = Some(json!("changed"));
        refresh(&mut loaded, true);
        let mut report = compare(&old, &snapshot(loaded), None).unwrap();
        let previous: BTreeSet<_> = report.findings.iter().map(|f| f.finding_id.clone()).collect();
        let request = sha256_hex(b"exact request including component pins");
        bind_request(&mut report, &request).unwrap();
        let ids: BTreeSet<_> = report.findings.iter().map(|f| f.finding_id.clone()).collect();
        assert!(ids.is_disjoint(&previous));
        for id in report
            .sections
            .iter()
            .flat_map(|s| &s.finding_ids)
            .chain(report.policies.iter().flat_map(|p| &p.finding_ids))
        {
            assert!(ids.contains(id));
        }
        let bytes = render_json(&report).unwrap();
        bind_request(&mut report, &request).unwrap();
        assert_eq!(bytes, render_json(&report).unwrap());
        assert!(bind_request(&mut report, &"0".repeat(64)).is_err());
    }

    #[test]
    fn impact_rendering_honors_remaining_generation_budget() {
        let (old, loaded) = pair();
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        let json = render_json(&report).unwrap();
        let text = render_text(&report).unwrap();
        assert_eq!(render_json_bounded(&report, json.len()).unwrap(), json);
        assert_eq!(render_text_bounded(&report, text.len()).unwrap(), text);
        assert!(render_json_bounded(&report, json.len() - 1).is_err());
        assert!(render_text_bounded(&report, text.len() - 1).is_err());
        assert!(render_json_bounded(&report, 0).is_err());
        assert!(render_text_bounded(&report, 0).is_err());
    }

    #[test]
    fn project_reformatting_is_global_provenance_without_local_dependency_change() {
        let (old, loaded) = pair();
        let mut new = snapshot(loaded);
        new.loaded.project_sha256 = sha256_hex(b"same exact model differently formatted");
        new.plan.provenance.project_sha256 = new.loaded.project_sha256.clone();
        let report = compare(&old, &new, None).unwrap();
        assert!(has(&report, ChangeCategory::ProjectBindingChanged));
        assert!(report.policies.iter().all(|p| p.unaffected));
        assert!(report.sections.iter().all(|s| s.unaffected));
    }
    #[test]
    fn pack_reviewer_metadata_is_global_provenance_not_substantive_content() {
        let (old, mut loaded) = pair();
        loaded.pack.reviewers[0].name = "Updated asserted reviewer label".into();
        refresh(&mut loaded, true);
        let report = compare(&old, &snapshot(loaded), None).unwrap();
        assert!(has(&report, ChangeCategory::PackChanged));
        assert!(report.findings.iter().all(|f| !f.axes.substantive_changed));
        assert!(
            report
                .sections
                .iter()
                .all(|s| !s.axes.substantive_changed && !s.axes.output_bytes_changed)
        );
    }
    #[test]
    fn component_extension_reformatting_is_global_provenance_with_unaffected_local_content() {
        let (mut old, loaded) = pair();
        let mut new = snapshot(loaded);
        old.components.push(component());
        new.components.push(component());
        old.component_manifest_sha256 = Some(sha256_hex(b"original exact extension bytes"));
        new.component_manifest_sha256 = Some(sha256_hex(b"reformatted exact extension bytes"));
        let report = compare(&old, &new, None).unwrap();
        assert_eq!(report.findings.len(), 1);
        let finding = &report.findings[0];
        assert_eq!(finding.category, ChangeCategory::ComponentExtensionBindingChanged);
        assert_eq!(finding.old_sha256, old.component_manifest_sha256);
        assert_eq!(finding.new_sha256, new.component_manifest_sha256);
        assert_eq!(finding.axes, axes(false, true, false));
        assert!(finding.affected_sections.is_empty());
        assert!(report.sections.iter().all(|s| s.unaffected));
        assert!(report.policies.iter().all(|p| p.unaffected));
        assert!(report.action_required());
    }
    #[test]
    fn impact_schema_and_runtime_reject_control_character_path_and_hash_suffixes() {
        let schema: Value =
            serde_json::from_slice(include_bytes!("../../schemas/authoring-impact.schema.json"))
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        for path in [
            "old/project.json\n",
            "old/project.json\r",
            "old/project.json\r\n",
            "old/pro\tject.json",
            "old/pro\u{0085}ject.json",
        ] {
            let mut invalid = wire();
            invalid["old"]["project"]["path"] = json!(path);
            assert!(!validator.is_valid(&invalid), "schema accepted {path:?}");
            assert!(
                parse_manifest(&serde_json::to_vec(&invalid).unwrap()).is_err(),
                "runtime accepted {path:?}"
            );
        }
        let mut invalid = wire();
        invalid["old"]["project"]["expected_sha256"] = json!(format!("{}\n", "a".repeat(64)));
        assert!(!validator.is_valid(&invalid));
        assert!(parse_manifest(&serde_json::to_vec(&invalid).unwrap()).is_err());
        let key_validator = jsonschema::validator_for(&schema["$defs"]["key"]).unwrap();
        assert!(!key_validator.is_valid(&json!("reviewer\n")));
        assert!(validate_key("reviewer\n").is_err());
    }

    #[test]
    fn impact_utf8_byte_bounds_remain_distinct_from_schema_character_preflight() {
        let schema: Value =
            serde_json::from_slice(include_bytes!("../../schemas/authoring-impact.schema.json"))
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let mut value = wire();
        value["correspondence"] = json!({
            "old_framework_sha256":"a".repeat(64), "new_framework_sha256":"b".repeat(64),
            "reviewers":[{"key":"reviewer","name":"Synthetic reviewer"}],
            "review":{"reviewer_key":"reviewer","reviewed_at":"2026-09-01T00:00:00Z","rationale":"原".repeat(16384)},
            "controls":[{"old_control_id":"sample-1","new_control_id":"sample-1"}]
        });
        assert!(validator.is_valid(&value));
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap()).is_err());
        value["correspondence"]["review"]["rationale"] = json!("原".repeat(16384 / 3));
        assert!(validator.is_valid(&value));
        parse_manifest(&serde_json::to_vec(&value).unwrap()).unwrap();
    }
    #[test]
    fn snapshot_roles_must_be_distinct_but_old_and_new_may_repeat_exact_pins() {
        for side in ["old", "new"] {
            let mut invalid = wire();
            invalid[side]["components"] = invalid[side]["project"].clone();
            let error = parse_manifest(&serde_json::to_vec(&invalid).unwrap()).unwrap_err();
            assert!(error.to_string().contains("distinct files"));
        }
        let mut repeated = wire();
        repeated["new"] = repeated["old"].clone();
        parse_manifest(&serde_json::to_vec(&repeated).unwrap()).unwrap();
    }

    #[test]
    fn component_question_dependencies_validate_without_rejecting_missing_answers() {
        let (mut old, mut loaded) = pair();
        old.components.push(component());
        loaded.project.answers.clear();
        loaded.project.human_clauses[0].answer_refs.clear();
        refresh(&mut loaded, true);
        let mut new = snapshot(loaded);
        let mut missing = component();
        missing.rendered_sha256 = None;
        new.components.push(missing);
        let report = compare(&old, &new, None).unwrap();
        assert!(report.is_complete());
        assert!(has(&report, ChangeCategory::AnswerStateChanged));
        assert!(section(&report, "sample-topic").axes.authoring_state_changed);
        assert!(section(&report, "independent-topic").unaffected);
        new.components[0].question_keys = vec!["unknown-question".into()];
        assert!(compare(&old, &new, None).is_err());
        new.components[0].question_keys.clear();
        assert!(compare(&old, &new, None).is_err());
        old.components[0].question_keys.clear();
        assert!(index(&old).is_err());
    }

    #[test]
    fn unavailable_report_fingerprints_are_explicit_null_without_fabricated_hashes() {
        let report = incomplete_report(&"a".repeat(64), &"b".repeat(64), "both").unwrap();
        let value: Value = serde_json::from_slice(&render_json(&report).unwrap()).unwrap();
        for field in ["old_report_sha256", "new_report_sha256"] {
            assert!(value[field].is_null());
            assert!(value["findings"][0][field].is_null());
        }
        let (old, new) = pair();
        let complete = compare(&old, &snapshot(new), None).unwrap();
        assert_eq!(complete.old_report_sha256.as_deref(), Some(old.loaded.report_sha256.as_str()));
        assert_eq!(complete.new_report_sha256.as_deref(), Some(old.loaded.report_sha256.as_str()));
    }

    #[test]
    fn effective_impact_output_budgets_are_reported_and_leave_writer_unchanged() {
        let report = incomplete_report(&"a".repeat(64), &"b".repeat(64), "both").unwrap();
        for limit in [0, 1, 32] {
            let json = render_json_bounded(&report, limit).unwrap_err().to_string();
            let text = render_text_bounded(&report, limit).unwrap_err().to_string();
            assert!(json.contains(&format!("{limit} byte limit")));
            assert!(text.contains(&format!("{limit} byte limit")));
        }
        let mut writer = BoundedWriter { bytes: b"a".to_vec(), limit: 2 };
        let error = writer.write_all(b"bc").unwrap_err();
        assert!(error.to_string().contains("2 byte limit"));
        assert_eq!(writer.bytes, b"a");
        let error =
            parse_manifest(&vec![b' '; usize::try_from(manifest::MAX_MANIFEST_BYTES).unwrap() + 1])
                .unwrap_err();
        assert!(
            error.to_string().contains(&format!("{} byte limit", manifest::MAX_MANIFEST_BYTES))
        );
    }

    #[test]
    fn text_comparison_status_matches_the_json_contract_label() {
        let mut report = incomplete_report(&"a".repeat(64), &"b".repeat(64), "both").unwrap();
        for status in [
            ComparisonStatus::Complete,
            ComparisonStatus::Incomplete,
            ComparisonStatus::Unsupported,
        ] {
            report.status = status;
            let value = serde_json::to_value(status).unwrap();
            assert_eq!(value, status.as_str());
            assert!(
                render_text(&report)
                    .unwrap()
                    .contains(&format!("Comparison: {}\n", status.as_str()))
            );
        }
    }

    #[test]
    fn scope_records_reserve_remaining_budget_before_allocation() {
        let (old, loaded) = pair();
        let new = snapshot(loaded);
        let old_index = index(&old).unwrap();
        let new_index = index(&new).unwrap();
        let mut report = empty_report(
            &old.loaded.project_sha256,
            &new.loaded.project_sha256,
            Some(&old.loaded.report_sha256),
            Some(&new.loaded.report_sha256),
        );
        report.evidence_bytes = super::super::output::MAX_OUTPUT_BYTES - 1;
        let error = finish_impacts(&mut report, &old, &new, &old_index, &new_index).unwrap_err();
        assert!(error.to_string().contains("scope evidence"));
        assert!(report.sections.is_empty());
        assert!(report.policies.is_empty());
        assert_eq!(report.evidence_bytes, super::super::output::MAX_OUTPUT_BYTES - 1);
    }
    fn revised_framework(loaded: &mut LoadedAuthorProject) {
        loaded.baseline_report.framework.raw_sha256 =
            sha256_hex(b"independently revised framework");
        loaded.project.baseline.framework_sha256 =
            loaded.baseline_report.framework.raw_sha256.clone();
        refresh(loaded, true);
    }

    #[test]
    fn empty_and_partial_correspondence_cannot_hide_surviving_control_changes() {
        let (old, mut loaded) = pair();
        revised_framework(&mut loaded);
        let mut new = snapshot(loaded);
        new.control_fingerprints
            .insert("sample-2".into(), sha256_hex(b"changed surviving control"));
        let mut reviewed = correspondence(&old, &new);
        reviewed.controls.clear();
        assert!(compare(&old, &new, Some(&reviewed)).is_err());
        let schema: Value =
            serde_json::from_slice(include_bytes!("../../schemas/authoring-impact.schema.json"))
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let mut value = wire();
        value["correspondence"] = serde_json::to_value(&reviewed).unwrap();
        assert!(!validator.is_valid(&value));
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap()).is_err());
        reviewed = correspondence(&old, &new);
        reviewed.controls.pop();
        let report = compare(&old, &new, Some(&reviewed)).unwrap();
        assert_eq!(report.status, ComparisonStatus::Unsupported);
        assert!(report.sections.is_empty() && report.policies.is_empty());
        assert!(has(&report, ChangeCategory::UnsupportedCorrespondence));
        assert!(report.old_report_sha256.is_some() && report.new_report_sha256.is_some());
    }

    #[test]
    fn reviewed_partial_pairs_allow_true_absent_side_control_additions_and_removals() {
        let (old, mut loaded) = pair();
        loaded.baseline_report.controls[1].control_id = "sample-3".into();
        loaded.pack.control_assignments[1].control_id = "sample-3".into();
        revised_framework(&mut loaded);
        let new = snapshot(loaded);
        let mut reviewed = correspondence(&old, &new);
        reviewed.controls.pop();
        let report = compare(&old, &new, Some(&reviewed)).unwrap();
        assert!(report.is_complete());
        assert!(has(&report, ChangeCategory::FrameworkControlAdded));
        assert!(has(&report, ChangeCategory::FrameworkControlRemoved));
        assert!(has(&report, ChangeCategory::GapAdded));
        assert!(has(&report, ChangeCategory::GapRemoved));
        assert!(!section(&report, "independent-topic").unaffected);
    }

    #[test]
    fn explicit_control_topic_dependencies_remain_visible_without_current_gaps() {
        let mut loaded = load_example();
        loaded.baseline_report.controls[1].classification = GapClassification::NotApplicable;
        loaded.baseline_report.counts.applicable_unmapped -= 1;
        loaded.baseline_report.counts.not_applicable += 1;
        refresh(&mut loaded, true);
        let old = snapshot(loaded.clone());
        assert!(
            old.plan.policies[0]
                .sections
                .iter()
                .find(|s| s.topic_key == "independent-topic")
                .unwrap()
                .gap_ids
                .is_empty()
        );
        revised_framework(&mut loaded);
        let mut new = snapshot(loaded);
        new.control_fingerprints
            .insert("sample-2".into(), sha256_hex(b"changed no-gap dependency"));
        let reviewed = correspondence(&old, &new);
        let report = compare(&old, &new, Some(&reviewed)).unwrap();
        assert!(report.is_complete());
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.category == ChangeCategory::FrameworkControlChanged)
            .unwrap();
        assert_eq!(
            finding.affected_sections,
            vec![SectionReference {
                policy_key: "sample-policy".into(),
                topic_key: "independent-topic".into()
            }]
        );
        assert!(section(&report, "independent-topic").axes.substantive_changed);
    }
    #[test]
    fn no_gap_edges_cannot_consume_budget_reserved_for_later_component_edges() {
        // The preflight total includes clause/component edges even before those
        // indexes are built. Additional no-gap edges consume only its remainder.
        let mut reserved = MAX_GRAPH - 1;
        let error = reserve_references(&mut reserved, 2).unwrap_err();
        assert!(error.to_string().contains("dependency graph"));
        assert_eq!(reserved, MAX_GRAPH - 1);
        reserve_references(&mut reserved, 1).unwrap();
        assert_eq!(reserved, MAX_GRAPH);
        assert!(reserve_references(&mut reserved, 1).is_err());
        assert_eq!(reserved, MAX_GRAPH);
    }
}

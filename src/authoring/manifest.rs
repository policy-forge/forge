//! Closed, bounded local authoring contracts and explicit answer evaluation.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error;
use super::model::{AnswerStatus, QuestionEvaluation};
use crate::ForgeError;

pub const PACK_SCHEMA_VERSION: &str = "forge.authoring-pack/1";
pub const PROJECT_SCHEMA_VERSION: &str = "forge.author-project/1";
pub const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_STRING_BYTES: usize = 16 * 1024;
pub const MAX_REVIEWERS: usize = 256;
pub const MAX_TOPICS: usize = 1_000;
pub const MAX_QUESTIONS: usize = 4_096;
pub const MAX_RECORDS: usize = 10_000;
pub const MAX_REFERENCES: usize = 128;
pub const MAX_CLAUSE_BYTES: u64 = 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 50 * 1024 * 1024;
const MAX_REGEX_BYTES: usize = 1_024;
const MAX_REGEX_SIZE: usize = 1024 * 1024;
const MAX_KEY_BYTES: usize = 64;
const MAX_AGE_DAYS: u32 = 36_500;
const LIMITS: crate::json_strict::Limits =
    crate::json_strict::Limits { max_depth: 32, max_string_bytes: MAX_STRING_BYTES };
static SEMVER: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$",
    )
    .expect("static semantic-version grammar")
});

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PinnedFile {
    pub path: PathBuf,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BaselineBinding {
    pub framework_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_catalog_sha256: Option<String>,
    pub report_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reviewer {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub reviewer_key: String,
    pub reviewed_at: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContentRights {
    pub source_label: String,
    pub statement: String,
    pub review: Review,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuthoringPack {
    pub schema_version: String,
    pub pack_key: String,
    pub version: String,
    pub baseline: BaselineBinding,
    pub reviewers: Vec<Reviewer>,
    pub content_rights: ContentRights,
    pub topics: Vec<Topic>,
    pub policy_families: Vec<PolicyFamily>,
    pub questions: Vec<Question>,
    pub control_assignments: Vec<ControlAssignment>,
    pub family_assignments: Vec<FamilyAssignment>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    pub key: String,
    pub title: String,
    pub order: u32,
    pub question_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PolicyFamily {
    pub key: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub key: String,
    pub prompt: String,
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    pub required: bool,
    pub owner: String,
    pub sensitivity: Sensitivity,
    pub source_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age_days: Option<u32>,
    pub constraints: QuestionConstraints,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum QuestionType {
    String,
    Integer,
    Boolean,
    StringList,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Sensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuestionConstraints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ControlAssignment {
    pub key: String,
    pub control_id: String,
    pub topic_key: String,
    pub review: Review,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FamilyAssignment {
    pub key: String,
    pub topic_key: String,
    pub policy_family_key: String,
    pub review: Review,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuthorProject {
    pub schema_version: String,
    pub project_key: String,
    pub project_root: PathBuf,
    pub baseline: BaselineBinding,
    pub applicability_manifest: PinnedFile,
    pub gap_report: PinnedFile,
    pub authoring_pack: PinnedFile,
    pub as_of: String,
    pub reviewers: Vec<Reviewer>,
    pub baseline_review: Review,
    pub policies: Vec<Policy>,
    pub answers: Vec<Answer>,
    pub deferrals: Vec<Deferral>,
    pub human_clauses: Vec<HumanClause>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub key: String,
    pub policy_family_key: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Answer {
    pub key: String,
    pub question_key: String,
    pub question_sha256: String,
    pub authoring_pack_sha256: String,
    pub owner: String,
    pub source_label: String,
    pub sensitivity: Sensitivity,
    pub review: Review,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub state: AnswerState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AnswerState {
    Provided,
    NoAnswer,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Deferral {
    pub key: String,
    pub gap_id: String,
    pub review: Review,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revisit_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AnswerPin {
    pub answer_key: String,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HumanClause {
    pub key: String,
    pub policy_key: String,
    pub topic_key: String,
    pub gap_ids: Vec<String>,
    pub answer_refs: Vec<AnswerPin>,
    pub source: PinnedFile,
    pub review: Review,
}

/// Parse one bounded, duplicate-key-safe authoring pack.
///
/// # Errors
/// Returns an authoring error for unsupported or invalid contracts.
pub fn parse_pack(bytes: &[u8]) -> Result<AuthoringPack, ForgeError> {
    let pack = parse_closed(bytes, "authoring pack")?;
    validate_pack(&pack)?;
    Ok(pack)
}

/// Parse one bounded, duplicate-key-safe author project.
///
/// # Errors
/// Returns an authoring error for unsupported or invalid contracts.
pub fn parse_project(bytes: &[u8]) -> Result<AuthorProject, ForgeError> {
    let value = parse_value(bytes, "author project")?;
    // `Option<Value>` erases explicit null. The wire contract distinguishes
    // absence from a supplied null, so inspect that distinction before Serde.
    if let Some(answers) = value.get("answers").and_then(Value::as_array) {
        for answer in answers {
            if answer.get("value").is_some_and(Value::is_null)
                || (answer.get("state").and_then(Value::as_str) == Some("no-answer")
                    && answer.get("value").is_some())
            {
                return Err(error(
                    "answer value must be a supported value or absent for no-answer",
                ));
            }
        }
    }
    let project: AuthorProject = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid author project contract: {cause}")))?;
    validate_project(&project)?;
    Ok(project)
}

fn parse_closed<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, ForgeError> {
    serde_json::from_value(parse_value(bytes, label)?)
        .map_err(|cause| error(format!("invalid {label} contract: {cause}")))
}

fn parse_value(bytes: &[u8], label: &str) -> Result<Value, ForgeError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error(format!("{label} exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let value = crate::json_strict::parse_value(bytes, label, LIMITS)
        .map_err(|cause| error(cause.to_string()))?;
    reject_null(&value)?;
    Ok(value)
}

fn reject_null(value: &Value) -> Result<(), ForgeError> {
    match value {
        Value::Null => {
            Err(error("null is not an authoring value; omit optional fields explicitly"))
        }
        Value::Array(values) => values.iter().try_for_each(reject_null),
        Value::Object(values) => values.values().try_for_each(reject_null),
        _ => Ok(()),
    }
}

fn validate_pack(pack: &AuthoringPack) -> Result<(), ForgeError> {
    schema_version(&pack.schema_version, PACK_SCHEMA_VERSION)?;
    key("pack_key", &pack.pack_key)?;
    if pack.version.len() > 128 || !SEMVER.is_match(&pack.version) {
        return Err(error("pack version must be a bounded semantic version"));
    }
    validate_baseline(&pack.baseline)?;
    let reviewers = validate_reviewers(&pack.reviewers)?;
    label("content_rights.source_label", &pack.content_rights.source_label)?;
    text("content_rights.statement", &pack.content_rights.statement)?;
    validate_review(&pack.content_rights.review, &reviewers)?;
    let topics =
        unique_keys(pack.topics.iter().map(|item| item.key.as_str()), MAX_TOPICS, "topics")?;
    let families = unique_keys(
        pack.policy_families.iter().map(|item| item.key.as_str()),
        MAX_TOPICS,
        "policy_families",
    )?;
    let questions = unique_keys(
        pack.questions.iter().map(|item| item.key.as_str()),
        MAX_QUESTIONS,
        "questions",
    )?;
    for topic in &pack.topics {
        single_line("topic.title", &topic.title)?;
        unique_refs(&topic.question_keys, "topic.question_keys", false)?;
        for question in &topic.question_keys {
            require_ref(&questions, question, "topic question")?;
        }
    }
    for family in &pack.policy_families {
        single_line("policy_family.title", &family.title)?;
    }
    for question in &pack.questions {
        validate_question(question)?;
    }
    unique_keys(
        pack.control_assignments.iter().map(|item| item.key.as_str()),
        MAX_RECORDS,
        "control_assignments",
    )?;
    let mut control_edges = BTreeSet::new();
    for assignment in &pack.control_assignments {
        control_id(&assignment.control_id)?;
        require_ref(&topics, &assignment.topic_key, "control assignment topic")?;
        validate_review(&assignment.review, &reviewers)?;
        if !control_edges.insert((&assignment.control_id, &assignment.topic_key)) {
            return Err(error("duplicate control-to-topic relationship"));
        }
    }
    unique_keys(
        pack.family_assignments.iter().map(|item| item.key.as_str()),
        MAX_RECORDS,
        "family_assignments",
    )?;
    let mut family_edges = BTreeSet::new();
    for assignment in &pack.family_assignments {
        require_ref(&topics, &assignment.topic_key, "family assignment topic")?;
        require_ref(&families, &assignment.policy_family_key, "assignment policy family")?;
        validate_review(&assignment.review, &reviewers)?;
        if !family_edges.insert((&assignment.topic_key, &assignment.policy_family_key)) {
            return Err(error("duplicate topic-to-policy-family relationship"));
        }
    }
    Ok(())
}

fn validate_project(project: &AuthorProject) -> Result<(), ForgeError> {
    schema_version(&project.schema_version, PROJECT_SCHEMA_VERSION)?;
    key("project_key", &project.project_key)?;
    if project.project_root.as_os_str() != "." {
        return Err(error("Phase 1 project_root must be exactly '.'"));
    }
    validate_baseline(&project.baseline)?;
    let mut paths = BTreeSet::new();
    for (name, input) in [
        ("applicability_manifest", &project.applicability_manifest),
        ("gap_report", &project.gap_report),
        ("authoring_pack", &project.authoring_pack),
    ] {
        validate_pin(name, input, Some("json"))?;
        if !paths.insert(path_key(&input.path)?) {
            return Err(error("project input paths must be distinct"));
        }
    }
    if project.gap_report.expected_sha256 != project.baseline.report_sha256 {
        return Err(error("gap_report pin must equal baseline.report_sha256"));
    }
    let as_of = timestamp("as_of", &project.as_of)?;
    let reviewers = validate_reviewers(&project.reviewers)?;
    validate_review(&project.baseline_review, &reviewers)?;
    review_not_future(&project.baseline_review, as_of)?;
    unique_keys(project.policies.iter().map(|item| item.key.as_str()), MAX_TOPICS, "policies")?;
    let mut families = BTreeSet::new();
    for policy in &project.policies {
        key("policy_family_key", &policy.policy_family_key)?;
        single_line("policy.title", &policy.title)?;
        if !families.insert(&policy.policy_family_key) {
            return Err(error("only one project policy per policy family is allowed"));
        }
    }
    unique_keys(project.answers.iter().map(|item| item.key.as_str()), MAX_QUESTIONS, "answers")?;
    let mut questions = BTreeSet::new();
    for answer in &project.answers {
        key("answer.question_key", &answer.question_key)?;
        if !questions.insert(&answer.question_key) {
            return Err(error("duplicate answer for one question"));
        }
        sha("answer.question_sha256", &answer.question_sha256)?;
        sha("answer.authoring_pack_sha256", &answer.authoring_pack_sha256)?;
        key("answer.owner", &answer.owner)?;
        label("answer.source_label", &answer.source_label)?;
        validate_review(&answer.review, &reviewers)?;
        if let Some(expires) = &answer.expires_at {
            timestamp("answer.expires_at", expires)?;
        }
        match (answer.state, &answer.value) {
            (AnswerState::Provided, Some(value)) if supported_value(value) => {}
            (AnswerState::NoAnswer, None) => {}
            _ => {
                return Err(error(
                    "provided answers require a bounded supported value; no-answer must omit value",
                ));
            }
        }
    }
    unique_keys(project.deferrals.iter().map(|item| item.key.as_str()), MAX_RECORDS, "deferrals")?;
    let mut gaps = BTreeSet::new();
    for deferral in &project.deferrals {
        sha("deferral.gap_id", &deferral.gap_id)?;
        if !gaps.insert(&deferral.gap_id) {
            return Err(error("duplicate gap deferral"));
        }
        validate_review(&deferral.review, &reviewers)?;
        review_not_future(&deferral.review, as_of)?;
        if let Some(revisit) = &deferral.revisit_date {
            date("deferral.revisit_date", revisit)?;
        }
    }
    unique_keys(
        project.human_clauses.iter().map(|item| item.key.as_str()),
        MAX_RECORDS,
        "human_clauses",
    )?;
    for clause in &project.human_clauses {
        key("clause.policy_key", &clause.policy_key)?;
        key("clause.topic_key", &clause.topic_key)?;
        unique_refs(&clause.gap_ids, "clause.gap_ids", true)?;
        if clause.gap_ids.is_empty() {
            return Err(error("human clause must identify at least one gap"));
        }
        unique_keys(
            clause.answer_refs.iter().map(|item| item.answer_key.as_str()),
            MAX_REFERENCES,
            "clause.answer_refs",
        )?;
        for answer in &clause.answer_refs {
            sha("clause.answer_refs.expected_sha256", &answer.expected_sha256)?;
        }
        validate_pin("clause.source", &clause.source, Some("md"))?;
        if !paths.insert(path_key(&clause.source.path)?) {
            return Err(error("clause source aliases another project input path"));
        }
        validate_review(&clause.review, &reviewers)?;
        review_not_future(&clause.review, as_of)?;
    }
    Ok(())
}

/// Validate all cross-contract references and exact clause answer pins.
///
/// # Errors
/// Returns an authoring error for stale baselines, dangling or ambiguous relationships,
/// overlapping deferrals, or mismatched clause answer pins.
pub fn validate_relationships(
    pack: &AuthoringPack,
    project: &AuthorProject,
    report: &crate::applicability::model::ApplicabilityReport,
) -> Result<(), ForgeError> {
    validate_pack(pack)?;
    validate_project(project)?;
    if pack.baseline != project.baseline
        || project.baseline.framework_sha256 != report.framework.raw_sha256
        || project.baseline.resolved_catalog_sha256 != report.framework.resolved_catalog_sha256
    {
        return Err(error(
            "authoring pack, project and framework baseline fingerprints must match exactly",
        ));
    }
    let as_of = timestamp("as_of", &project.as_of)?;
    review_not_future(&pack.content_rights.review, as_of)?;
    for review in pack
        .control_assignments
        .iter()
        .map(|item| &item.review)
        .chain(pack.family_assignments.iter().map(|item| &item.review))
    {
        review_not_future(review, as_of)?;
    }
    let controls: BTreeSet<_> =
        report.controls.iter().map(|item| item.control_id.as_str()).collect();
    if controls.len() != report.controls.len() {
        return Err(error("baseline report contains duplicate control IDs"));
    }
    for assignment in &pack.control_assignments {
        require_ref(&controls, &assignment.control_id, "assignment framework control")?;
    }
    let families: BTreeSet<_> = pack.policy_families.iter().map(|item| item.key.as_str()).collect();
    let topics: BTreeMap<_, _> = pack.topics.iter().map(|item| (item.key.as_str(), item)).collect();
    let questions: BTreeSet<_> = pack.questions.iter().map(|item| item.key.as_str()).collect();
    let policies: BTreeMap<_, _> =
        project.policies.iter().map(|item| (item.key.as_str(), item)).collect();
    for policy in &project.policies {
        require_ref(&families, &policy.policy_family_key, "project policy family")?;
    }
    for answer in &project.answers {
        require_ref(&questions, &answer.question_key, "answer question")?;
    }
    let gaps: BTreeMap<_, _> = report
        .controls
        .iter()
        .filter(|control| {
            matches!(
        control.classification,
        crate::applicability::model::GapClassification::ApplicableUnmapped
            | crate::applicability::model::GapClassification::ApplicableReviewedNoRelationship
    )
        })
        .map(|control| {
            (
                gap_id(&project.baseline.report_sha256, &control.control_id),
                control.control_id.as_str(),
            )
        })
        .collect();
    let family_policies: BTreeSet<_> =
        project.policies.iter().map(|item| item.policy_family_key.as_str()).collect();
    let assigned_topics: BTreeSet<_> = pack
        .family_assignments
        .iter()
        .filter(|item| family_policies.contains(item.policy_family_key.as_str()))
        .map(|item| item.topic_key.as_str())
        .collect();
    let assigned_controls: BTreeSet<_> = pack
        .control_assignments
        .iter()
        .filter(|item| assigned_topics.contains(item.topic_key.as_str()))
        .map(|item| item.control_id.as_str())
        .collect();
    for deferral in &project.deferrals {
        let control = gaps
            .get(&deferral.gap_id)
            .ok_or_else(|| error("deferral references an unknown applicable gap"))?;
        if assigned_controls.contains(control) {
            return Err(error("applicable gap cannot be both assigned and deferred"));
        }
    }
    let answers: BTreeMap<_, _> =
        project.answers.iter().map(|item| (item.key.as_str(), item)).collect();
    for clause in &project.human_clauses {
        validate_clause_relationships(clause, pack, &policies, &topics, &gaps, &answers)?;
    }
    Ok(())
}

fn validate_clause_relationships(
    clause: &HumanClause,
    pack: &AuthoringPack,
    policies: &BTreeMap<&str, &Policy>,
    topics: &BTreeMap<&str, &Topic>,
    gaps: &BTreeMap<String, &str>,
    answers: &BTreeMap<&str, &Answer>,
) -> Result<(), ForgeError> {
    let policy = policies
        .get(clause.policy_key.as_str())
        .ok_or_else(|| error("human clause references an unknown policy"))?;
    let topic = topics
        .get(clause.topic_key.as_str())
        .ok_or_else(|| error("human clause references an unknown topic"))?;
    if !pack.family_assignments.iter().any(|edge| {
        edge.topic_key == clause.topic_key && edge.policy_family_key == policy.policy_family_key
    }) {
        return Err(error("human clause topic has no explicit assignment to its policy family"));
    }
    for gap in &clause.gap_ids {
        let control = gaps
            .get(gap)
            .ok_or_else(|| error("human clause references an unknown applicable gap"))?;
        if !pack
            .control_assignments
            .iter()
            .any(|edge| edge.control_id == *control && edge.topic_key == clause.topic_key)
        {
            return Err(error("human clause gap has no explicit assignment to its topic"));
        }
    }
    let mut pinned_questions = BTreeSet::new();
    for pin in &clause.answer_refs {
        let answer = answers
            .get(pin.answer_key.as_str())
            .ok_or_else(|| error("human clause references an unknown answer"))?;
        if answer_sha256(answer)? != pin.expected_sha256 {
            return Err(error("human clause answer pin does not match exact answer record"));
        }
        pinned_questions.insert(answer.question_key.as_str());
    }
    // Every existing required context dependency must be an explicit clause pin.
    // An absent answer remains a scoped context blocker; it cannot supply text.
    for question in pack
        .questions
        .iter()
        .filter(|question| question.required && topic.question_keys.contains(&question.key))
    {
        let has_answer = answers.values().any(|answer| answer.question_key == question.key);
        if has_answer && !pinned_questions.contains(question.key.as_str()) {
            return Err(error("human clause must pin every required topic answer"));
        }
    }
    Ok(())
}

fn validate_question(question: &Question) -> Result<(), ForgeError> {
    text("question.prompt", &question.prompt)?;
    key("question.owner", &question.owner)?;
    label("question.source_label", &question.source_label)?;
    if question.max_age_days.is_some_and(|days| days == 0 || days > MAX_AGE_DAYS) {
        return Err(error("question.max_age_days must be in 1..=36500"));
    }
    let bounds = &question.constraints;
    if bounds.min_length.zip(bounds.max_length).is_some_and(|(min, max)| min > max)
        || bounds.minimum.zip(bounds.maximum).is_some_and(|(min, max)| min > max)
        || bounds.min_items.zip(bounds.max_items).is_some_and(|(min, max)| min > max)
        || bounds.min_length.is_some_and(|value| value > MAX_STRING_BYTES)
        || bounds.max_length.is_some_and(|value| value > MAX_STRING_BYTES)
        || bounds.min_items.is_some_and(|value| value > MAX_REFERENCES)
        || bounds.max_items.is_some_and(|value| value > MAX_REFERENCES)
    {
        return Err(error("question constraints have invalid or excessive bounds"));
    }
    let has_length =
        bounds.min_length.is_some() || bounds.max_length.is_some() || bounds.regex.is_some();
    let has_range = bounds.minimum.is_some() || bounds.maximum.is_some();
    let has_items = bounds.min_items.is_some() || bounds.max_items.is_some();
    let invalid = match question.question_type {
        QuestionType::String => has_range || has_items,
        QuestionType::Integer => has_length || has_items,
        QuestionType::Boolean => has_length || has_range || has_items,
        QuestionType::StringList => has_range,
    };
    if invalid {
        return Err(error("question constraints are not valid for its declared type"));
    }
    let compiled = bounds.regex.as_deref().map(compile_regex).transpose()?;
    if bounds.allowed_values.len() > MAX_REFERENCES {
        return Err(error("question allowed_values exceeds reference limit"));
    }
    let mut allowed = BTreeSet::new();
    for value in &bounds.allowed_values {
        if !supported_value(value) || !value_valid(question, value, compiled.as_ref(), false) {
            return Err(error(
                "question allowed_values contains a value incompatible with its constraints",
            ));
        }
        let bytes =
            serde_json::to_vec(value).map_err(|_| error("cannot serialize allowed value"))?;
        if !allowed.insert(bytes) {
            return Err(error("question allowed_values contains a duplicate value"));
        }
    }
    Ok(())
}

fn supported_value(value: &Value) -> bool {
    match value {
        Value::String(text) => text.len() <= MAX_STRING_BYTES,
        Value::Bool(_) => true,
        Value::Number(number) => number.as_i64().is_some(),
        Value::Array(values) => {
            values.len() <= MAX_REFERENCES
                && values
                    .iter()
                    .all(|item| item.as_str().is_some_and(|text| text.len() <= MAX_STRING_BYTES))
        }
        Value::Null | Value::Object(_) => false,
    }
}

fn value_valid(
    question: &Question,
    value: &Value,
    pattern: Option<&regex::Regex>,
    check_enum: bool,
) -> bool {
    let bounds = &question.constraints;
    if check_enum && !bounds.allowed_values.is_empty() && !bounds.allowed_values.contains(value) {
        return false;
    }
    let string_valid = |text: &str| {
        let len = text.chars().count();
        bounds.min_length.is_none_or(|min| len >= min)
            && bounds.max_length.is_none_or(|max| len <= max)
            && pattern.is_none_or(|pattern| pattern.is_match(text))
    };
    match (question.question_type, value) {
        (QuestionType::String, Value::String(text)) => string_valid(text),
        (QuestionType::Integer, Value::Number(number)) => number.as_i64().is_some_and(|number| {
            bounds.minimum.is_none_or(|min| number >= min)
                && bounds.maximum.is_none_or(|max| number <= max)
        }),
        (QuestionType::Boolean, Value::Bool(_)) => true,
        (QuestionType::StringList, Value::Array(values)) => {
            bounds.min_items.is_none_or(|min| values.len() >= min)
                && bounds.max_items.is_none_or(|max| values.len() <= max)
                && values.iter().all(|value| value.as_str().is_some_and(string_valid))
        }
        _ => false,
    }
}

/// Evaluate declared context without exposing or substituting answer values.
///
/// # Errors
/// Returns an authoring error for invalid schema-level timestamps or question constraints.
pub fn evaluate_question(
    question: &Question,
    answer: Option<&Answer>,
    pack_sha256: &str,
    as_of: &str,
) -> Result<QuestionEvaluation, ForgeError> {
    validate_question(question)?;
    let now = timestamp("as_of", as_of)?;
    let question_hash = question_sha256(question)?;
    let state = if let Some(answer) = answer {
        evaluate_answer(question, answer, &question_hash, pack_sha256, now)?
    } else {
        AnswerStatus::Missing
    };
    Ok(QuestionEvaluation {
        question_key: question.key.clone(),
        question_sha256: question_hash,
        required: question.required,
        owner: question.owner.clone(),
        sensitivity: question.sensitivity,
        source_label: question.source_label.clone(),
        answer_key: answer.map(|answer| answer.key.clone()),
        answer_sha256: answer.map(answer_sha256).transpose()?,
        state,
        review: answer.map(|answer| answer.review.clone()),
        expires_at: answer.and_then(|answer| answer.expires_at.clone()),
    })
}

fn evaluate_answer(
    question: &Question,
    answer: &Answer,
    question_hash: &str,
    pack_sha256: &str,
    as_of: DateTime<FixedOffset>,
) -> Result<AnswerStatus, ForgeError> {
    if answer.question_sha256 != question_hash || answer.authoring_pack_sha256 != pack_sha256 {
        return Ok(AnswerStatus::Stale);
    }
    let reviewed = timestamp("answer.review.reviewed_at", &answer.review.reviewed_at)?;
    if answer.question_key != question.key
        || answer.owner != question.owner
        || answer.sensitivity != question.sensitivity
        || reviewed > as_of
    {
        return Ok(AnswerStatus::Invalid);
    }
    if let Some(expires) = &answer.expires_at {
        let expires = timestamp("answer.expires_at", expires)?;
        if expires < reviewed {
            return Ok(AnswerStatus::Invalid);
        }
        if as_of >= expires {
            return Ok(AnswerStatus::Expired);
        }
    }
    if question.max_age_days.is_some_and(|days| {
        as_of.signed_duration_since(reviewed).num_seconds() >= i64::from(days) * 86_400
    }) {
        return Ok(AnswerStatus::Expired);
    }
    if answer.state == AnswerState::NoAnswer {
        return Ok(AnswerStatus::NoAnswer);
    }
    let pattern = question.constraints.regex.as_deref().map(compile_regex).transpose()?;
    Ok(
        if answer.value.as_ref().is_some_and(|value| {
            supported_value(value) && value_valid(question, value, pattern.as_ref(), true)
        }) {
            AnswerStatus::Available
        } else {
            AnswerStatus::Invalid
        },
    )
}

/// Hash the complete question contract using canonical JSON and a versioned domain.
///
/// # Errors
/// Returns an authoring error if serialization fails.
pub fn question_sha256(question: &Question) -> Result<String, ForgeError> {
    canonical_hash("forge.authoring-question/1", question)
}

/// Hash the complete answer record, including provenance and its explicit state.
///
/// # Errors
/// Returns an authoring error if serialization fails.
pub fn answer_sha256(answer: &Answer) -> Result<String, ForgeError> {
    canonical_hash("forge.authoring-answer/1", answer)
}

/// Derive an exact-baseline gap identity without machine or location data.
#[must_use]
pub fn gap_id(report_sha256: &str, control_id: &str) -> String {
    hash_tuple(&[b"forge.authoring-gap/1", report_sha256.as_bytes(), control_id.as_bytes()])
}

fn canonical_hash<T: Serialize>(domain: &str, value: &T) -> Result<String, ForgeError> {
    let mut value =
        serde_json::to_value(value).map_err(|_| error("cannot serialize authoring identity"))?;
    sort_objects(&mut value);
    let bytes =
        serde_json::to_vec(&value).map_err(|_| error("cannot encode authoring identity"))?;
    Ok(hash_tuple(&[domain.as_bytes(), &bytes]))
}

fn sort_objects(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for value in map.values_mut() {
                sort_objects(value);
            }
            let ordered: BTreeMap<_, _> = std::mem::take(map).into_iter().collect();
            map.extend(ordered);
        }
        Value::Array(values) => values.iter_mut().for_each(sort_objects),
        _ => {}
    }
}

fn hash_tuple(values: &[&[u8]]) -> String {
    let mut bytes = Vec::new();
    for value in values {
        bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
        bytes.extend_from_slice(value);
    }
    crate::hashing::sha256_hex(&bytes)
}

fn compile_regex(pattern: &str) -> Result<regex::Regex, ForgeError> {
    if pattern.len() > MAX_REGEX_BYTES {
        return Err(error("question regex exceeds 1024 bytes"));
    }
    regex::RegexBuilder::new(pattern)
        .size_limit(MAX_REGEX_SIZE)
        .dfa_size_limit(MAX_REGEX_SIZE)
        .build()
        .map_err(|_| error("question regex is invalid or exceeds the compiled-size bound"))
}

fn validate_baseline(baseline: &BaselineBinding) -> Result<(), ForgeError> {
    sha("baseline.framework_sha256", &baseline.framework_sha256)?;
    sha("baseline.report_sha256", &baseline.report_sha256)?;
    if let Some(hash) = &baseline.resolved_catalog_sha256 {
        sha("baseline.resolved_catalog_sha256", hash)?;
    }
    Ok(())
}

fn validate_pin(name: &str, pin: &PinnedFile, extension: Option<&str>) -> Result<(), ForgeError> {
    validate_local_path(name, &pin.path)?;
    if extension.is_some_and(|extension| {
        pin.path.extension().and_then(|value| value.to_str()) != Some(extension)
    }) {
        return Err(error(format!("{name} has an unsupported file extension")));
    }
    sha("input expected_sha256", &pin.expected_sha256)
}

/// Validate one canonical portable descendant path, before filesystem traversal.
///
/// # Errors
/// Returns an authoring error for absolute, noncanonical, ambiguous or unsafe paths.
pub fn validate_local_path(name: &str, path: &Path) -> Result<(), ForgeError> {
    let value = path.to_str().ok_or_else(|| error(format!("{name} must be UTF-8")))?;
    if value.is_empty()
        || value.len() > 1_024
        || value.contains(['\\', ':'])
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || value.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error(format!("{name} must be a canonical project-relative path")));
    }
    for part in value.split('/') {
        if part.chars().any(|ch| ch.is_control() || matches!(ch, '<' | '>' | '"' | '|' | '?' | '*'))
            || part.ends_with([' ', '.'])
            || part.trim() != part
            || part.len() > 255
        {
            return Err(error(format!("{name} contains an unsafe path component")));
        }
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        let numbered_device = stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9');
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
            || numbered_device
        {
            return Err(error(format!("{name} contains a reserved device name")));
        }
    }
    Ok(())
}

fn path_key(path: &Path) -> Result<String, ForgeError> {
    path.to_str().map(str::to_ascii_lowercase).ok_or_else(|| error("path must be UTF-8"))
}

fn validate_reviewers(reviewers: &[Reviewer]) -> Result<BTreeSet<&str>, ForgeError> {
    let keys =
        unique_keys(reviewers.iter().map(|item| item.key.as_str()), MAX_REVIEWERS, "reviewers")?;
    for reviewer in reviewers {
        single_line("reviewer.name", &reviewer.name)?;
    }
    Ok(keys)
}

fn validate_review(review: &Review, reviewers: &BTreeSet<&str>) -> Result<(), ForgeError> {
    require_ref(reviewers, &review.reviewer_key, "review reviewer")?;
    timestamp("review.reviewed_at", &review.reviewed_at)?;
    text("review.rationale", &review.rationale)
}

fn review_not_future(review: &Review, as_of: DateTime<FixedOffset>) -> Result<(), ForgeError> {
    if timestamp("review.reviewed_at", &review.reviewed_at)? > as_of {
        return Err(error("review time must not follow project as_of"));
    }
    Ok(())
}

fn unique_keys<'a>(
    values: impl Iterator<Item = &'a str>,
    limit: usize,
    name: &str,
) -> Result<BTreeSet<&'a str>, ForgeError> {
    let mut seen = BTreeSet::new();
    for value in values {
        key(name, value)?;
        if !seen.insert(value) {
            return Err(error(format!("{name} contains a duplicate logical key")));
        }
        if seen.len() > limit {
            return Err(error(format!("{name} exceeds {limit} entries")));
        }
    }
    Ok(seen)
}

fn unique_refs(values: &[String], name: &str, hashes: bool) -> Result<(), ForgeError> {
    if values.len() > MAX_REFERENCES {
        return Err(error(format!("{name} exceeds {MAX_REFERENCES} references")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if hashes {
            sha(name, value)?;
        } else {
            key(name, value)?;
        }
        if !seen.insert(value) {
            return Err(error(format!("{name} contains a duplicate reference")));
        }
    }
    Ok(())
}

fn require_ref(values: &BTreeSet<&str>, value: &str, name: &str) -> Result<(), ForgeError> {
    if !values.contains(value) {
        return Err(error(format!("{name} references an undeclared identifier")));
    }
    Ok(())
}

fn schema_version(value: &str, expected: &str) -> Result<(), ForgeError> {
    if value != expected {
        return Err(error(format!("unsupported schema_version; expected {expected}")));
    }
    Ok(())
}

fn key(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > MAX_KEY_BYTES
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == b'-')
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(error(format!(
            "{name} must be lowercase ASCII kebab-case, at most {MAX_KEY_BYTES} bytes"
        )));
    }
    Ok(())
}

fn control_id(value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > 256
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return Err(error("control_id must be bounded and contain no whitespace or controls"));
    }
    Ok(())
}

fn sha(name: &str, value: &str) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256(name, value).map_err(error)
}

fn text(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() || value.len() > MAX_STRING_BYTES
        || value.chars().any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
        || value.chars().any(|ch| matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
    {
        return Err(error(format!("{name} must be nonempty bounded text without unsafe controls")));
    }
    Ok(())
}

fn single_line(name: &str, value: &str) -> Result<(), ForgeError> {
    text(name, value)?;
    if value.contains(['\n', '\r', '\t']) || value.trim() != value {
        return Err(error(format!("{name} must be one trimmed line")));
    }
    Ok(())
}

fn label(name: &str, value: &str) -> Result<(), ForgeError> {
    single_line(name, value)?;
    crate::applicability::manifest::validate_report_href(name, value)
        .map_err(|_| error(format!("{name} must not contain an absolute local path")))
}

fn timestamp(name: &str, value: &str) -> Result<DateTime<FixedOffset>, ForgeError> {
    if value.len() > 64 || value.trim() != value {
        return Err(error(format!("{name} must be a bounded RFC 3339 timestamp")));
    }
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| error(format!("{name} must be an RFC 3339 timestamp")))
}

fn date(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.len() != 10 || chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
        return Err(error(format!("{name} must use YYYY-MM-DD")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applicability::model::{
        ApplicabilityReport, ClassificationCounts, ControlResult, GapClassification, ReportFilters,
    };
    use serde_json::json;

    const PACK: &[u8] = include_bytes!("../../tests/fixtures/authoring/contracts/valid-pack.json");
    const PROJECT: &[u8] =
        include_bytes!("../../tests/fixtures/authoring/contracts/valid-project.json");

    fn fixtures() -> (AuthoringPack, AuthorProject) {
        (parse_pack(PACK).unwrap(), parse_project(PROJECT).unwrap())
    }

    fn answer(question: &Question) -> Answer {
        let (_, project) = fixtures();
        Answer {
            key: "sample-answer".into(),
            question_key: question.key.clone(),
            question_sha256: question_sha256(question).unwrap(),
            authoring_pack_sha256: "d".repeat(64),
            owner: question.owner.clone(),
            source_label: "Synthetic interview".into(),
            sensitivity: question.sensitivity,
            review: project.baseline_review,
            expires_at: None,
            state: AnswerState::Provided,
            value: Some(json!("Fictional draft owner")),
        }
    }

    fn report() -> ApplicabilityReport {
        ApplicabilityReport {
            schema_version: crate::applicability::model::REPORT_SCHEMA_VERSION,
            manifest_sha256: "c".repeat(64),
            framework: crate::mapping::inventory::ResourceEvidence {
                resource_type: crate::mapping::manifest::ResourceType::Catalog,
                href: "framework.json".into(),
                raw_sha256: "a".repeat(64),
                root_uuid: "11111111-1111-4111-8111-111111111111".into(),
                document_version: "1.0.0".into(),
                oscal_version: "1.2.3".into(),
                resolved_catalog_sha256: None,
            },
            mapping_collections: vec![],
            reviewers: vec![],
            counts: ClassificationCounts { total: 1, applicable_unmapped: 1, ..Default::default() },
            filters: ReportFilters::default(),
            matched_controls: 1,
            controls: vec![ControlResult {
                control_id: "sample-1".into(),
                groups: vec![],
                classification: GapClassification::ApplicableUnmapped,
                reviewer_key: Some("synthetic-reviewer".into()),
                reviewed_at: Some("2026-09-01T00:00:00Z".into()),
                rationale: None,
                revisit_date: None,
                note: None,
                positive_mapping_count: 0,
                no_relationship_count: 0,
                policy_sources: vec![],
            }],
            review_queue: vec![],
        }
    }

    #[test]
    fn valid_contract_fixtures_also_validate_against_published_schemas() {
        for (schema, bytes) in [
            (include_bytes!("../../schemas/authoring-pack.schema.json").as_slice(), PACK),
            (include_bytes!("../../schemas/author-project.schema.json").as_slice(), PROJECT),
        ] {
            let schema: Value = serde_json::from_slice(schema).unwrap();
            let document: Value = serde_json::from_slice(bytes).unwrap();
            let validator = jsonschema::validator_for(&schema).unwrap();
            assert!(
                validator.is_valid(&document),
                "{:?}",
                validator.iter_errors(&document).collect::<Vec<_>>()
            );
        }
        let (pack, project) = fixtures();
        validate_relationships(&pack, &project, &report()).unwrap();
    }

    #[test]
    fn adversarial_and_forward_version_fixtures_fail_closed() {
        for bytes in [
            include_bytes!("../../tests/fixtures/authoring/contracts/forward-pack.json").as_slice(),
            include_bytes!("../../tests/fixtures/authoring/contracts/unknown-nested-pack.json")
                .as_slice(),
            include_bytes!("../../tests/fixtures/authoring/contracts/duplicate-logical-pack.json")
                .as_slice(),
            include_bytes!("../../tests/fixtures/authoring/contracts/duplicate-decoded-key.json")
                .as_slice(),
        ] {
            assert!(parse_pack(bytes).is_err());
        }
        for bytes in [
            include_bytes!("../../tests/fixtures/authoring/contracts/forward-project.json")
                .as_slice(),
            include_bytes!("../../tests/fixtures/authoring/contracts/unsafe-project-path.json")
                .as_slice(),
        ] {
            assert!(parse_project(bytes).is_err());
        }
    }

    #[test]
    fn malformed_json_bounds_and_canonical_identifier_rules_are_enforced() {
        assert!(parse_pack(b"{} {}").is_err());
        assert!(parse_pack(b"{\"value\":\"\xff\"}").is_err());
        assert!(parse_pack(&vec![b' '; usize::try_from(MAX_MANIFEST_BYTES).unwrap() + 1]).is_err());
        let mut nested = Value::Null;
        for _ in 0..34 {
            nested = json!([nested]);
        }
        assert!(parse_pack(&serde_json::to_vec(&nested).unwrap()).is_err());
        for invalid in ["A-key", "key--gap", "key-", " key", "é-key", "1-key"] {
            let (mut pack, _) = fixtures();
            pack.pack_key = invalid.into();
            assert!(parse_pack(&serde_json::to_vec(&pack).unwrap()).is_err(), "{invalid}");
        }
        for version in ["1", "01.0.0", "1.0.0-01", "1.2.3.4"] {
            let (mut pack, _) = fixtures();
            pack.version = version.into();
            assert!(parse_pack(&serde_json::to_vec(&pack).unwrap()).is_err());
        }
        let (mut pack, _) = fixtures();
        pack.questions[0].prompt = "x".repeat(MAX_STRING_BYTES + 1);
        assert!(parse_pack(&serde_json::to_vec(&pack).unwrap()).is_err());
    }

    #[test]
    fn paths_reject_cross_platform_ambiguity_before_io() {
        for path in [
            "../a.json",
            "/a.json",
            "a/../b.json",
            "./a.json",
            "a//b.json",
            "a/",
            "C:a.json",
            "a\\b.json",
            "a.json:stream",
            "con.json",
            "AUX",
            "lpt1.md",
            "folder /a.json",
            "a?/b.json",
        ] {
            assert!(validate_local_path("input", Path::new(path)).is_err(), "{path}");
        }
        validate_local_path("input", Path::new("inputs/catalog-v1.json")).unwrap();
        let (_, mut project) = fixtures();
        project.project_root = PathBuf::from("./");
        assert!(parse_project(&serde_json::to_vec(&project).unwrap()).is_err());
        let (_, mut project) = fixtures();
        project.authoring_pack.path = PathBuf::from("GAP-REPORT.JSON");
        assert!(parse_project(&serde_json::to_vec(&project).unwrap()).is_err());
    }

    #[test]
    fn constraints_are_typed_bounded_and_cannot_supply_defaults() {
        for constraints in [
            json!({"min_length":2,"max_length":1}),
            json!({"minimum":1}),
            json!({"regex":"["}),
            json!({"regex":"x".repeat(MAX_REGEX_BYTES + 1)}),
            json!({"allowed_values":[true]}),
            json!({"allowed_values":["a","a"]}),
            json!({"default":"hidden fact"}),
            json!({"max_length":MAX_STRING_BYTES+1}),
        ] {
            let mut pack: Value = serde_json::from_slice(PACK).unwrap();
            pack["questions"][0]["constraints"] = constraints;
            assert!(parse_pack(&serde_json::to_vec(&pack).unwrap()).is_err());
        }
        let (mut pack, _) = fixtures();
        pack.questions[0].question_type = QuestionType::StringList;
        pack.questions[0].constraints = QuestionConstraints {
            min_length: Some(1),
            max_length: Some(3),
            min_items: Some(1),
            max_items: Some(2),
            ..Default::default()
        };
        validate_pack(&pack).unwrap();
        let mut record = answer(&pack.questions[0]);
        record.value = Some(json!(["a", "bbb"]));
        assert_eq!(
            evaluate_question(
                &pack.questions[0],
                Some(&record),
                &"d".repeat(64),
                "2026-09-08T00:00:00Z"
            )
            .unwrap()
            .state,
            AnswerStatus::Available
        );
        record.value = Some(json!(["too long"]));
        assert_eq!(
            evaluate_question(
                &pack.questions[0],
                Some(&record),
                &"d".repeat(64),
                "2026-09-08T00:00:00Z"
            )
            .unwrap()
            .state,
            AnswerStatus::Invalid
        );
    }

    #[test]
    fn missing_stale_expired_invalid_and_no_answer_are_explicit() {
        let (pack, project) = fixtures();
        let question = &pack.questions[0];
        let check = |answer: Option<&Answer>| {
            evaluate_question(question, answer, &"d".repeat(64), &project.as_of).unwrap()
        };
        assert_eq!(check(None).state, AnswerStatus::Missing);
        let valid = answer(question);
        assert_eq!(check(Some(&valid)).state, AnswerStatus::Available);
        let mut changed = valid.clone();
        changed.question_sha256 = "0".repeat(64);
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Stale);
        changed = valid.clone();
        changed.authoring_pack_sha256 = "0".repeat(64);
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Stale);
        changed = valid.clone();
        changed.expires_at = Some(project.as_of.clone());
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Expired);
        changed = valid.clone();
        changed.review.reviewed_at = "2026-08-01T00:00:00Z".into();
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Expired);
        changed = valid.clone();
        changed.value = Some(json!(true));
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Invalid);
        changed = valid.clone();
        changed.owner = "another-owner".into();
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Invalid);
        changed = valid.clone();
        changed.sensitivity = Sensitivity::Public;
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Invalid);
        changed = valid.clone();
        changed.review.reviewed_at = "2026-09-09T00:00:00Z".into();
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Invalid);
        changed = valid.clone();
        changed.expires_at = Some("2026-08-01T00:00:00Z".into());
        assert_eq!(check(Some(&changed)).state, AnswerStatus::Invalid);
        changed = valid;
        changed.state = AnswerState::NoAnswer;
        changed.value = None;
        assert_eq!(check(Some(&changed)).state, AnswerStatus::NoAnswer);
        assert!(
            !serde_json::to_string(&check(Some(&changed)))
                .unwrap()
                .contains("Fictional draft owner")
        );
    }

    #[test]
    fn provided_answers_require_supported_values_and_no_answer_requires_absence() {
        let (pack, mut project) = fixtures();
        project.answers.push(answer(&pack.questions[0]));
        for value in [
            json!(null),
            json!({"fact":"hidden"}),
            json!(1.5),
            json!([true]),
            json!(vec!["x"; 129]),
        ] {
            let mut wire = serde_json::to_value(&project).unwrap();
            wire["answers"][0]["value"] = value;
            assert!(parse_project(&serde_json::to_vec(&wire).unwrap()).is_err());
        }
        project.answers[0].state = AnswerState::NoAnswer;
        assert!(parse_project(&serde_json::to_vec(&project).unwrap()).is_err());
        project.answers[0].value = None;
        parse_project(&serde_json::to_vec(&project).unwrap()).unwrap();
    }

    #[test]
    fn hashes_are_domain_separated_and_bind_complete_records() {
        let (pack, _) = fixtures();
        let question = &pack.questions[0];
        let valid = answer(question);
        let mut changed = valid.clone();
        changed.review.rationale.push_str(" Additional decision.");
        assert_ne!(answer_sha256(&valid).unwrap(), answer_sha256(&changed).unwrap());
        changed = valid.clone();
        changed.value = Some(json!("New explicit input"));
        assert_ne!(answer_sha256(&valid).unwrap(), answer_sha256(&changed).unwrap());
        let mut question_changed = question.clone();
        question_changed.required = false;
        assert_ne!(question_sha256(question).unwrap(), question_sha256(&question_changed).unwrap());
        assert_ne!(gap_id("ab", "c"), gap_id("a", "bc"));
        assert_ne!(gap_id(&"a".repeat(64), "sample-1"), gap_id(&"b".repeat(64), "sample-1"));
        let mut object: Value = serde_json::to_value(question).unwrap();
        let mut ordered = serde_json::Map::new();
        for (key, value) in std::mem::take(object.as_object_mut().unwrap()).into_iter().rev() {
            ordered.insert(key, value);
        }
        let reordered: Question = serde_json::from_value(Value::Object(ordered)).unwrap();
        assert_eq!(question_sha256(question).unwrap(), question_sha256(&reordered).unwrap());
    }

    #[test]
    fn cross_references_deferrals_and_exact_answer_pins_fail_closed() {
        let (mut pack, mut project) = fixtures();
        let baseline = report();
        pack.control_assignments[0].control_id = "unknown".into();
        assert!(validate_relationships(&pack, &project, &baseline).is_err());
        pack.control_assignments[0].control_id = "sample-1".into();
        project.deferrals.push(Deferral {
            key: "later".into(),
            gap_id: gap_id(&project.baseline.report_sha256, "sample-1"),
            review: project.baseline_review.clone(),
            revisit_date: None,
        });
        assert!(validate_relationships(&pack, &project, &baseline).is_err());
        project.deferrals.clear();
        project.answers.push(answer(&pack.questions[0]));
        project.human_clauses.push(HumanClause {
            key: "human-clause".into(),
            policy_key: "sample-policy".into(),
            topic_key: "sample-topic".into(),
            gap_ids: vec![gap_id(&project.baseline.report_sha256, "sample-1")],
            answer_refs: vec![AnswerPin {
                answer_key: "sample-answer".into(),
                expected_sha256: answer_sha256(&project.answers[0]).unwrap(),
            }],
            source: PinnedFile {
                path: PathBuf::from("clause.md"),
                expected_sha256: "e".repeat(64),
            },
            review: project.baseline_review.clone(),
        });
        validate_relationships(&pack, &project, &baseline).unwrap();
        project.answers[0].value = Some(json!("Changed supplied role"));
        let failure = validate_relationships(&pack, &project, &baseline).unwrap_err().to_string();
        assert!(failure.contains("answer pin"));
        assert!(!failure.contains("Changed supplied role"));
    }
}

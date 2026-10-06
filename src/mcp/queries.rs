//! Minimized typed queries over an actual caller-authorized captured scope.
//!
//! This is transport neutral: Root must admit MCP metadata, frame IDs, and the
//! complete JSON-RPC response envelope. Every prepared query keeps the real scope
//! borrowed until its final original-generation fence. No query writes projects,
//! evaluates policy text, fetches URIs, grants authority, or substitutes a prior
//! structurally admitted report for a current captured domain closure.

use std::collections::BTreeMap;
use std::io::{self, Write};

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::disclosure::{ApprovedObject, CapturedQueryScope};
use super::{NativeIdentity, Role};
use crate::workspace::preparation::{ProgressUpdate, Stage, WorkControl, WorkError};

/// Complete captured recorded-applicability semantics, separate from prior admission.
#[path = "recorded.rs"]
mod recorded;
/// Exact native/source requirement extraction; no file or authority constructor.
#[path = "requirements.rs"]
mod requirements;

/// Complete typed core ceiling; Root separately charges the added wire envelope.
const MAX_RESPONSE: usize = 256 * 1024;
/// Existing approved input-frame domain, also applied to canonical query hashing.
const MAX_ARGUMENTS: usize = 64 * 1024;
/// Maximum number of complete page rows after whole-scope matching.
const MAX_PAGE: usize = 50;
/// Maximum request token sequence before any derived search state grows.
const MAX_TOKENS: usize = 32;
/// Exact raw and normalized search token byte domain, never clipped.
const MAX_QUERY: usize = 4096;
/// Maximum lawful exact source substring, additionally gated by profile and caller.
const MAX_EXCERPT: usize = 512;

/// Closed fixed domain reasons; private native errors, paths and prose never echo.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Reason {
    /// No usable operator-selected recorded disclosure decision is present.
    ApprovalUnavailable,
    /// A required actual recorded lifecycle tuple is not approved.
    Unapproved,
    /// Current captured bytes or identity differ from recorded approval.
    Noncurrent,
    /// The intrinsic native or domain artifact cannot be admitted.
    InvalidArtifact,
    /// Complete declared captured dependency or relation binding is absent.
    IncompleteClosure,
    /// An actual captured source span cannot be established exactly.
    SourceSpanUnavailable,
    /// The trusted profile does not permit the requested tool or content.
    VisibilityRefused,
    /// The selected role does not support this native query profile.
    UnsupportedRole,
    /// The exact permitted selection is absent, without private discovery details.
    NotFound,
    /// A page cursor belongs to a different complete query generation.
    GenerationChanged,
    /// Complete core or individual bounded projection exceeds admitted bytes.
    OutputBoundExceeded,
    /// Same immutable accepted query deadline exceeded; Root preserves typed stop.
    QueryBudgetExceeded,
    /// Runtime cancellation/shutdown maps only at Root's exact publication boundary.
    #[allow(dead_code)]
    // Reserved closed wire reason; known transport cancellation suppresses its response.
    Interrupted,
}

/// Domain unavailability remains distinct from sticky runtime interruption.
pub(crate) enum QueryError {
    /// Fixed unavailable domain result, with no trusted partial data.
    Unavailable(Reason),
    /// Actual shared control/capture error, including unchanged typed interruption.
    Work(WorkError),
}

/// Internal query result; Root maps domain errors and controls at the wire boundary.
pub(crate) type QueryResult<T> = Result<T, QueryError>;

impl From<WorkError> for QueryError {
    /// Preserve the entire typed stop without `into_error` or best-effort fallback.
    fn from(error: WorkError) -> Self {
        Self::Work(error)
    }
}

/// Explicit page controls after Root's closed per-tool input schema admission.
#[derive(Clone, Serialize)]
pub(crate) struct Page {
    /// Exact generation-bound cursor; null starts at the complete first match.
    pub(crate) cursor: Option<String>,
    /// Requested complete row count, admitted in 1..=50.
    pub(crate) limit: usize,
}

/// Internal typed dispatch; this is not an independently selected MCP protocol.
#[derive(Clone, Serialize)]
#[serde(tag = "tool", content = "arguments", rename_all = "snake_case")]
pub(crate) enum Query {
    /// Complete approved native policy listing.
    ListPolicies(Page),
    /// Deterministic lexical lookup over the same actual source spans.
    SearchRequirements {
        /// Original caller text, never executed as instructions or a pattern.
        query: String,
        /// Exact optional visible native artifact restriction.
        artifact_key: Option<String>,
        /// Caller excerpt opt-in; trusted lawful scope remains independently required.
        include_excerpt: bool,
        /// Complete generation-bound page controls.
        page: Page,
    },
    /// Exact native requirement selection and captured source citation.
    GetRequirement {
        /// Caller-approved opaque artifact key.
        artifact_key: String,
        /// Exact actual native ID, not a generated display label.
        requirement_id: String,
        /// Caller source opt-in under the additional trusted lawful profile.
        include_excerpt: bool,
    },
    /// Exact recorded direct and native mapping relationships.
    TraceControl {
        /// Exact visible native artifact key.
        artifact_key: String,
        /// Exact actual native control identifier.
        control_id: String,
        /// Complete relation page, never a partial inventory advertised as complete.
        page: Page,
    },
    /// Actual current recorded native applicability classification.
    GetRecordedApplicability {
        /// Exact selected captured applicability domain key.
        artifact_key: String,
        /// Exact optional native control filter, without changing full counts.
        subject_id: Option<String>,
        /// Complete matching page controls.
        page: Page,
    },
    /// Full native classification summary independent of relation paging.
    GetGapSummary {
        /// Exact selected captured applicability domain key.
        artifact_key: String,
        /// Complete matching page controls.
        page: Page,
    },
    /// Actual neutral approved/current native status and optional captured schedule.
    GetArtifactStatus {
        /// Exact opaque visible artifact key.
        artifact_key: String,
    },
}

impl Query {
    /// Return the exact approved public operation name for tool-scope admission.
    pub(crate) const fn tool(&self) -> &'static str {
        match self {
            Self::ListPolicies(_) => "list_policies",
            Self::SearchRequirements { .. } => "search_requirements",
            Self::GetRequirement { .. } => "get_requirement",
            Self::TraceControl { .. } => "trace_control",
            Self::GetRecordedApplicability { .. } => "get_recorded_applicability",
            Self::GetGapSummary { .. } => "get_gap_summary",
            Self::GetArtifactStatus { .. } => "get_artifact_status",
        }
    }
}

/// Complete available typed result; unavailable results are Root-created null data.
#[derive(Serialize)]
pub(crate) struct QueryResponse {
    /// Dedicated engineering result profile, not a protocol version.
    schema_version: &'static str,
    /// Actual selected public operation.
    tool: &'static str,
    /// Every returned prepared response contains complete available data.
    availability: &'static str,
    /// Available responses carry explicit null, never arbitrary error text.
    reason: Option<Reason>,
    /// One exact closed operation-specific result.
    data: Option<Data>,
}

/// Exact closed per-tool data payloads without invented common fields.
#[derive(Serialize)]
#[serde(untagged)]
enum Data {
    /// Bounded complete policy page.
    Policies(Paged<PolicyRow>),
    /// Exact one native requirement.
    Requirement(RequirementRow),
    /// Complete matched lexical search page.
    Search(Paged<SearchRow>),
    /// Complete recorded native relation page.
    Trace(Paged<TraceRow>),
    /// Recorded decisions and native classifications.
    Applicability(Paged<recorded::ApplicabilityRow>),
    /// Full classification universe and bounded rows.
    Gaps(recorded::GapData),
    /// Actual neutral status and evaluated same-capture schedule when selected.
    Status(StatusRow),
}

/// Complete common page with a whole denominator and exact emitted row count.
#[derive(Serialize)]
pub(super) struct Paged<T> {
    /// Complete configuration/source/query generation, not freshness authority.
    pub(super) generation: String,
    /// Entire admitted matching denominator before page selection.
    pub(super) matched: usize,
    /// Exact number of retained page rows.
    pub(super) emitted: usize,
    /// Next same-generation ordinal, or null on the complete last page.
    pub(super) next_cursor: Option<String>,
    /// At most fifty individually precharged complete rows.
    pub(super) rows: Vec<T>,
}

/// Exact original source span; byte offsets never imply extracted line authority.
#[derive(Clone, Serialize)]
pub(super) struct SourceSpan {
    /// Inclusive actual UTF-8 byte position.
    start_byte: usize,
    /// Exclusive actual UTF-8 byte position.
    end_byte: usize,
}

/// Minimized actual native/source citation, not an active href or content assertion.
#[derive(Clone, Serialize)]
pub(super) struct Citation {
    /// Selected safe artifact key.
    artifact_key: String,
    /// Actual bounded native model/root/version identity, or explicit null.
    artifact_identity: Option<NativeIdentity>,
    /// Exact raw native original hash, not normalized JSON or derived fingerprint.
    artifact_raw_sha256: String,
    /// Trusted inert token without private path spelling.
    #[allow(clippy::struct_field_names)]
    // Exact shipped citation field spelling is part of the tool wire schema.
    citation_label: String,
    /// Complete pointer into the actual native Value.
    native_pointer: String,
    /// Exact captured source key for this recorded relation.
    source_key: Option<String>,
    /// Raw original source hash from the same complete capture.
    source_raw_sha256: Option<String>,
    /// Actual UTF-8 source offsets or explicit null.
    source_span: Option<SourceSpan>,
    /// Exact captured-source qualification; no native-only trusted requirement.
    source_state: &'static str,
    /// Existing closed role, without introducing workspace admission.
    artifact_role: Role,
}

/// Explicitly opted-in exact lawful substring; it is always untrusted content.
#[derive(Serialize)]
struct Excerpt {
    /// At most512 UTF-8 bytes from the actual captured line, no ellipsis or synthesis.
    text: String,
    /// Exact source/native citation corresponding to those captured bytes.
    citation: Citation,
    /// Fixed required marker, not interpretation or instructions.
    trust: &'static str,
}

/// Exact native requirement and its source proof; prose is absent by default.
#[derive(Serialize)]
pub(super) struct RequirementRow {
    /// Actual native control identifier or implemented-requirement UUID.
    requirement_id: String,
    /// Trusted opaque native artifact key.
    artifact_key: String,
    /// At least one actual full native/source citation, at most eight.
    citations: Vec<Citation>,
    /// Explicit null unless both caller and profile opt into lawful source text.
    excerpt: Option<Excerpt>,
}

/// Deterministic lexical match; rank is an ordinal, never an adjudicated score.
#[derive(Serialize)]
struct SearchRow {
    /// Actual exact native/source requirement projection.
    requirement: RequirementRow,
    /// One-based ordinal in complete exact-ID/all-token/some-token ordering.
    rank: usize,
    /// Native schema field labels only; matched text is not echoed here.
    matched_fields: Vec<&'static str>,
    /// Exact rule selected by bounded deterministic lexical search.
    match_kind: &'static str,
}

/// Exact recorded native control relation with no equivalence inference.
#[derive(Serialize)]
pub(super) struct TraceRow {
    /// Actual requested native control ID.
    pub(super) control_id: String,
    /// Actual source/native requirement identifier.
    pub(super) requirement_id: String,
    /// Actual native mapping ID, null for a direct trace relation.
    pub(super) mapping_id: Option<String>,
    /// Direct recorded control or recorded native mapping; no generated judgment.
    pub(super) relation: &'static str,
    /// Complete exact captured source/native relation citations.
    pub(super) citations: Vec<Citation>,
    /// F20 metadata requires its own actual typed companion; default explicit null.
    evidence_metadata: Option<recorded::EvidenceMetadata>,
}

/// Same-capture advisory schedule; recorded approval remains separately established.
#[derive(Serialize)]
struct Schedule {
    /// Trusted canonical profile date actually passed to the maintained projector.
    as_of: String,
    /// Actual intrinsically validated recorded review date, not an inferred deadline.
    next_review_date: String,
    /// Actual recorded u16 window, never supplied by a detached output constructor.
    due_soon_days: u16,
    /// Maintained approved/due-soon/overdue calculation over the same originals.
    derived_status: String,
}

/// Exact visible approved policy row; no owner, title, private path or rationale.
#[derive(Serialize)]
struct PolicyRow {
    /// Actual selected safe key.
    artifact_key: String,
    /// Actual safe native identity, required for a native policy listing.
    native_identity: NativeIdentity,
    /// Raw current original hash.
    raw_sha256: String,
    /// Recorded neutral approval established before any schedule evaluation.
    lifecycle_state: &'static str,
    /// Complete recorded currentness, not merely metadata validity.
    freshness_scope: &'static str,
    /// Not evaluated unless the actual trusted profile selected a canonical date.
    schedule_scope: &'static str,
    /// Trusted safe token, not a source path or document title.
    citation_label: String,
    /// Actual minimized schedule or explicit null.
    schedule: Option<Schedule>,
}

/// Exact available artifact status; policy sources may have explicit null identity.
#[derive(Serialize)]
struct StatusRow {
    /// Actual trusted visible key.
    artifact_key: String,
    /// Actual safe native identity, or null for a recorded approved policy source.
    native_identity: Option<NativeIdentity>,
    /// Current raw original hash from the same real proof.
    raw_sha256: String,
    /// Trusted inert citation token.
    citation_label: String,
    /// Actual recorded approval, never replaced by schedule-derived state.
    lifecycle_state: &'static str,
    /// Exact complete captured recorded fingerprint-current scope.
    freshness_scope: &'static str,
    /// Actual schedule evaluation scope, never a metadata assertion.
    schedule_scope: &'static str,
    /// Actual same-capture evaluated schedule or explicit null.
    schedule: Option<Schedule>,
}

/// Prepared exact output borrowing the private capture through final publication.
pub(crate) struct PreparedQuery<'a> {
    /// Real admitted roots/config/native/source/lifecycle original proof.
    scope: &'a CapturedQueryScope,
    /// Closed complete minimized result, not a trusted partial prefix.
    response: QueryResponse,
}

impl PreparedQuery<'_> {
    /// Borrow actual typed data for the Root-owned full-envelope capped encoder.
    pub(crate) fn response(&self) -> &QueryResponse {
        &self.response
    }

    /// Recheck all actual original roots, bytes, EOF, identities and typed absences.
    pub(crate) fn verify_inputs(&self, control: &mut dyn WorkControl) -> Result<(), WorkError> {
        self.scope.verify_inputs(control)
    }
}

/// Dispatch all seven selected tools using one actual captured generation/control.
pub(crate) fn prepare<'a>(
    scope: &'a CapturedQueryScope,
    query: Query,
    control: &mut dyn WorkControl,
) -> QueryResult<PreparedQuery<'a>> {
    checkpoint(control)?;
    validate_query(&query)?;
    if !scope.tool_enabled(query.tool()) {
        return Err(Reason::VisibilityRefused.into());
    }
    let tool = query.tool();
    let generation = generation(scope, &query)?;
    let data = match query {
        Query::ListPolicies(page) => Data::Policies(policies(scope, generation, &page, control)?),
        Query::GetArtifactStatus { artifact_key } => {
            let approved = approve(scope, &artifact_key, control)?;
            Data::Status(status(&approved)?)
        }
        Query::GetRequirement { artifact_key, requirement_id, include_excerpt } => {
            if !safe_token(&artifact_key) || !safe_token(&requirement_id) {
                return Err(Reason::NotFound.into());
            }
            let entries = requirements::collect(scope, Some(&artifact_key), control)?;
            let entry = entries
                .get(&(artifact_key.as_str(), requirement_id.as_str()))
                .ok_or(Reason::NotFound)?;
            Data::Requirement(requirement(scope, entry, include_excerpt, control)?)
        }
        Query::SearchRequirements { query, artifact_key, include_excerpt, page } => {
            Data::Search(search(
                scope,
                generation,
                &query,
                artifact_key.as_deref(),
                include_excerpt,
                &page,
                control,
            )?)
        }
        Query::TraceControl { artifact_key, control_id, page } => {
            Data::Trace(trace(scope, generation, &artifact_key, &control_id, &page, control)?)
        }
        Query::GetRecordedApplicability { artifact_key, subject_id, page } => {
            Data::Applicability(recorded::applicability(
                scope,
                generation,
                &artifact_key,
                subject_id.as_deref(),
                &page,
                control,
            )?)
        }
        Query::GetGapSummary { artifact_key, page } => {
            Data::Gaps(recorded::gaps(scope, generation, &artifact_key, &page, control)?)
        }
    };
    let response = QueryResponse {
        schema_version: "forge.mcp-query/1",
        tool,
        availability: "available",
        reason: None,
        data: Some(data),
    };
    count(&response, MAX_RESPONSE)?;
    scope.verify_inputs(control)?;
    checkpoint(control)?;
    Ok(PreparedQuery { scope, response })
}

/// Root consumes this bounded typed argument admission before capturing originals.
///
/// The raw frame duplicate/BOM/UTF8/closed schema checks remain Root-owned. This
/// helper has no capture, profile, parser, filesystem, timing or policy authority.
pub(crate) fn validate_query(query: &Query) -> QueryResult<()> {
    let (key, selection, page) = match query {
        Query::ListPolicies(page) => (None, None, Some(page)),
        Query::SearchRequirements { query, artifact_key, page, .. } => {
            tokens(query)?;
            (artifact_key.as_deref(), None, Some(page))
        }
        Query::GetRequirement { artifact_key, requirement_id, .. } => {
            (Some(artifact_key.as_str()), Some(requirement_id.as_str()), None)
        }
        Query::TraceControl { artifact_key, control_id, page } => {
            (Some(artifact_key.as_str()), Some(control_id.as_str()), Some(page))
        }
        Query::GetRecordedApplicability { artifact_key, subject_id, page } => {
            (Some(artifact_key.as_str()), subject_id.as_deref(), Some(page))
        }
        Query::GetGapSummary { artifact_key, page } => {
            (Some(artifact_key.as_str()), None, Some(page))
        }
        Query::GetArtifactStatus { artifact_key } => (Some(artifact_key.as_str()), None, None),
    };
    if key.is_some_and(|value| !safe_token(value))
        || selection.is_some_and(|value| !safe_token(value))
        || page.is_some_and(|page| {
            !(1..=MAX_PAGE).contains(&page.limit)
                || page.cursor.as_ref().is_some_and(|cursor| cursor.len() > 256)
        })
    {
        return Err(Reason::NotFound.into());
    }
    Ok(())
}

/// Create only a closed null-data failure; it confers no artifact/capture proof.
pub(crate) fn unavailable(query: &Query, reason: Reason) -> QueryResponse {
    QueryResponse {
        schema_version: "forge.mcp-query/1",
        tool: query.tool(),
        availability: "unavailable",
        reason: Some(reason),
        data: None,
    }
}

/// Require exact selected scope/current neutral proof without echoing private facts.
fn approve<'a>(
    scope: &'a CapturedQueryScope,
    key: &str,
    control: &mut dyn WorkControl,
) -> QueryResult<ApprovedObject<'a>> {
    if !safe_token(key) {
        return Err(Reason::NotFound.into());
    }
    if !scope.visible_keys().any(|visible| visible == key) {
        return Err(Reason::VisibilityRefused.into());
    }
    scope.approved(key, control)?.ok_or_else(|| Reason::Unapproved.into())
}

/// Minimize actual same-capture schedule after neutral current approval.
fn schedule(approved: &ApprovedObject<'_>) -> QueryResult<Option<Schedule>> {
    let Some(status) = approved.evaluated_status() else {
        return Ok(None);
    };
    let as_of = status.as_of.ok_or(Reason::InvalidArtifact)?.to_string();
    if as_of.len() != 10
        || !as_of.bytes().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 { byte == b'-' } else { byte.is_ascii_digit() }
        })
        || !status.artifact_identity_changes.is_empty()
        || status.blockers.iter().any(|reason| reason != "overdue" && reason != "due-soon")
        || !matches!(status.derived_status.as_str(), "approved" | "due-soon" | "overdue")
    {
        return Err(Reason::Noncurrent.into());
    }
    Ok(Some(Schedule {
        as_of,
        next_review_date: status.next_review_date.to_string(),
        due_soon_days: approved.record().review.due_soon_days,
        derived_status: status.derived_status.clone(),
    }))
}

/// Project available neutral status from an opaque approval proof only.
fn status(approved: &ApprovedObject<'_>) -> QueryResult<StatusRow> {
    let resource = approved.resource();
    let identity = resource.native_identity();
    identity.map(validate_identity).transpose()?;
    let schedule = schedule(approved)?;
    Ok(StatusRow {
        artifact_key: resource.key().to_owned(),
        native_identity: identity.cloned(),
        raw_sha256: resource.raw_sha256().to_owned(),
        citation_label: resource.citation_label().to_owned(),
        lifecycle_state: "approved",
        freshness_scope: "complete-recorded-fingerprint-current",
        schedule_scope: if schedule.is_some() { "explicit-as-of" } else { "not-evaluated" },
        schedule,
    })
}

/// Count the entire visible native policy roster before retaining page rows.
fn policies(
    scope: &CapturedQueryScope,
    generation: String,
    page: &Page,
    control: &mut dyn WorkControl,
) -> QueryResult<Paged<PolicyRow>> {
    let mut keys = BTreeMap::new();
    for key in scope.visible_keys() {
        let resource = scope.resource(key).ok_or(Reason::IncompleteClosure)?;
        if matches!(
            resource.role(),
            Role::OscalCatalogArtifact
                | Role::OscalComponentArtifact
                | Role::OscalProfileArtifact
                | Role::OscalSspArtifact
        ) {
            checkpoint(control)?;
            scope.charge_relationships(1)?;
            let approved = approve(scope, key, control)?;
            let identity = approved.resource().native_identity().ok_or(Reason::InvalidArtifact)?;
            validate_identity(identity)?;
            keys.insert(key, (identity.model.as_str(), identity.root_id.as_str()));
        }
    }
    let matched = keys.len();
    let start = page_start(&generation, page, matched)?;
    let mut rows = Vec::new();
    let mut charge = 0;
    for key in keys.keys().skip(start).take(page.limit) {
        let approved = approve(scope, key, control)?;
        let row = status(&approved)?;
        let identity = row.native_identity.ok_or(Reason::InvalidArtifact)?;
        retain(
            &mut rows,
            PolicyRow {
                artifact_key: row.artifact_key,
                native_identity: identity,
                raw_sha256: row.raw_sha256,
                lifecycle_state: row.lifecycle_state,
                freshness_scope: row.freshness_scope,
                schedule_scope: row.schedule_scope,
                citation_label: row.citation_label,
                schedule: row.schedule,
            },
            &mut charge,
        )?;
    }
    Ok(page_result(generation, matched, start, rows))
}

/// Make one complete exact captured-source citation; no native-only requirement success.
fn citation(
    entry: &requirements::Requirement<'_>,
    control: &mut dyn WorkControl,
) -> QueryResult<Citation> {
    validate_identity(entry.identity)?;
    Ok(Citation {
        artifact_key: entry.artifact_key.to_owned(),
        artifact_identity: Some(entry.identity.clone()),
        artifact_raw_sha256: entry.native_sha256.to_owned(),
        citation_label: entry.citation_label.to_owned(),
        native_pointer: entry.pointer(control)?,
        source_key: Some(entry.source_key.to_owned()),
        source_raw_sha256: Some(entry.source_sha256.to_owned()),
        source_span: Some(SourceSpan { start_byte: entry.start_byte, end_byte: entry.end_byte }),
        source_state: "exact-captured-source",
        artifact_role: entry.role,
    })
}

/// Keep opted-in source bytes exact and mark them inert, never trim or invent text.
fn requirement(
    scope: &CapturedQueryScope,
    entry: &requirements::Requirement<'_>,
    include_excerpt: bool,
    control: &mut dyn WorkControl,
) -> QueryResult<RequirementRow> {
    let citation = citation(entry, control)?;
    let excerpt = if include_excerpt {
        if !scope.excerpt_allowed(entry.source_key) {
            return Err(Reason::VisibilityRefused.into());
        }
        let mut end = entry.source_line.len().min(MAX_EXCERPT);
        while !entry.source_line.is_char_boundary(end) {
            end -= 1;
        }
        let mut excerpt_citation = citation.clone();
        excerpt_citation.source_span =
            Some(SourceSpan { start_byte: entry.start_byte, end_byte: entry.start_byte + end });
        Some(Excerpt {
            text: entry.source_line[..end].to_owned(),
            citation: excerpt_citation,
            trust: "untrusted-content",
        })
    } else {
        None
    };
    Ok(RequirementRow {
        requirement_id: entry.id.to_owned(),
        artifact_key: entry.artifact_key.to_owned(),
        citations: vec![citation],
        excerpt,
    })
}

/// Deterministic exact-ID/all-token/some-token search over actual lawful captured text.
#[allow(clippy::too_many_arguments)]
fn search(
    scope: &CapturedQueryScope,
    generation: String,
    query: &str,
    selected: Option<&str>,
    include_excerpt: bool,
    page: &Page,
    control: &mut dyn WorkControl,
) -> QueryResult<Paged<SearchRow>> {
    if selected.is_some_and(|key| !safe_token(key)) {
        return Err(Reason::NotFound.into());
    }
    let tokens = tokens(query)?;
    let entries = requirements::collect(scope, selected, control)?;
    let mut matches = BTreeMap::new();
    for ((key, id), entry) in &entries {
        checkpoint(control)?;
        let exact = entry.id == query || entry.control_id == query;
        let mut hits = 0;
        let mut fields = [false; 4];
        for token in &tokens {
            let found = [
                contains_token(entry.id, token, control)?,
                contains_token(entry.citation_label, token, control)?,
                contains_token(entry.control_id, token, control)?,
                contains_token(entry.source_line, token, control)?,
            ];
            if found.iter().any(|present| *present) {
                hits += 1;
            }
            for index in 0..4 {
                fields[index] |= found[index];
            }
        }
        fields[0] |= entry.id == query;
        fields[2] |= entry.control_id == query;
        if !exact && hits == 0 {
            continue;
        }
        let ranking = if exact {
            0
        } else if hits == tokens.len() {
            1
        } else {
            2
        };
        scope.charge_relationships(1)?;
        matches.insert((ranking, *key, *id), (entry, exact, hits, fields));
    }
    let match_count = matches.len();
    let start = page_start(&generation, page, match_count)?;
    let mut rows = Vec::new();
    let mut charge = 0;
    for (index, (_, (entry, exact, hits, fields))) in
        matches.iter().enumerate().skip(start).take(page.limit)
    {
        let labels = ["native-id", "source-label", "control-id", "captured-requirement-text"];
        let matched_fields = labels
            .into_iter()
            .zip(fields)
            .filter_map(|(label, present)| present.then_some(label))
            .collect();
        retain(
            &mut rows,
            SearchRow {
                requirement: requirement(scope, entry, include_excerpt, control)?,
                rank: index + 1,
                matched_fields,
                match_kind: if *exact {
                    "exact-id"
                } else if *hits == tokens.len() {
                    "all-tokens"
                } else {
                    "some-tokens"
                },
            },
            &mut charge,
        )?;
    }
    Ok(page_result(generation, match_count, start, rows))
}

/// Combine direct and actual admitted mapping rows before page selection.
fn trace(
    scope: &CapturedQueryScope,
    generation: String,
    artifact_key: &str,
    control_id: &str,
    page: &Page,
    control: &mut dyn WorkControl,
) -> QueryResult<Paged<TraceRow>> {
    if !safe_token(artifact_key) || !safe_token(control_id) {
        return Err(Reason::NotFound.into());
    }
    recorded::trace(scope, generation, artifact_key, control_id, page, control)
}

/// Admit exact whole-generation page offsets; no reset, expiry renewal or prefix alias.
pub(super) fn page_start(generation: &str, page: &Page, matched: usize) -> QueryResult<usize> {
    if !(1..=MAX_PAGE).contains(&page.limit) {
        return Err(Reason::NotFound.into());
    }
    let Some(cursor) = page.cursor.as_deref() else {
        return Ok(0);
    };
    if cursor.len() > 256 {
        return Err(Reason::GenerationChanged.into());
    }
    let (prefix, number) = cursor.split_once(':').ok_or(Reason::GenerationChanged)?;
    if prefix != generation
        || number.is_empty()
        || !number.bytes().all(|byte| byte.is_ascii_digit())
        || number.len() > 1 && number.starts_with('0')
    {
        return Err(Reason::GenerationChanged.into());
    }
    let offset = number.parse::<usize>().map_err(|_| Reason::GenerationChanged)?;
    if offset >= matched {
        return Err(Reason::GenerationChanged.into());
    }
    Ok(offset)
}

/// Pair exact whole counts with the actual selected retained page length.
pub(super) fn page_result<T>(
    generation: String,
    matched: usize,
    start: usize,
    rows: Vec<T>,
) -> Paged<T> {
    let emitted = rows.len();
    let next = start + emitted;
    let next_cursor = (next < matched).then(|| format!("{generation}:{next}"));
    Paged { generation, matched, emitted, next_cursor, rows }
}

/// Bind canonical typed query/filter arguments to the exact actual captured scope.
fn generation(scope: &CapturedQueryScope, query: &Query) -> QueryResult<String> {
    let mut hash = HashWriter { hash: Sha256::new(), bytes: 0, limit: MAX_ARGUMENTS };
    hash.write_all(b"forge.mcp-query-generation/1\0").map_err(|_| Reason::OutputBoundExceeded)?;
    hash.write_all(scope.scope_generation().as_bytes()).map_err(|_| Reason::OutputBoundExceeded)?;
    hash.write_all(b"\0").map_err(|_| Reason::OutputBoundExceeded)?;
    // A cursor refers to the same underlying query; its token must not change generation.
    let mut canonical = CanonicalQuery::from(query);
    canonical.cursor = None;
    serde_json::to_writer(&mut hash, &canonical).map_err(|_| Reason::OutputBoundExceeded)?;
    Ok(crate::hashing::lower_hex(&hash.hash.finalize()))
}

/// Stable query fields independent of cursor transport and page size.
#[derive(Serialize)]
struct CanonicalQuery<'a> {
    /// Exact public tool name.
    tool: &'static str,
    /// Exact selected native/domain artifact key or null.
    artifact_key: Option<&'a str>,
    /// Exact raw search or native requirement/control/subject selection, or null.
    selection: Option<&'a str>,
    /// Exact caller excerpt flag, independently constrained by trusted profile.
    include_excerpt: bool,
    /// Deliberately null; cursor is an output derived from this generation.
    cursor: Option<&'a str>,
}

impl<'a> From<&'a Query> for CanonicalQuery<'a> {
    /// Preserve exact identity/filter bytes while removing only page traversal state.
    fn from(query: &'a Query) -> Self {
        let (artifact_key, selection, include_excerpt) = match query {
            Query::ListPolicies(_) => (None, None, false),
            Query::SearchRequirements { query, artifact_key, include_excerpt, .. } => {
                (artifact_key.as_deref(), Some(query.as_str()), *include_excerpt)
            }
            Query::GetRequirement { artifact_key, requirement_id, include_excerpt } => {
                (Some(artifact_key.as_str()), Some(requirement_id.as_str()), *include_excerpt)
            }
            Query::TraceControl { artifact_key, control_id, .. } => {
                (Some(artifact_key.as_str()), Some(control_id.as_str()), false)
            }
            Query::GetRecordedApplicability { artifact_key, subject_id, .. } => {
                (Some(artifact_key.as_str()), subject_id.as_deref(), false)
            }
            Query::GetGapSummary { artifact_key, .. }
            | Query::GetArtifactStatus { artifact_key } => {
                (Some(artifact_key.as_str()), None, false)
            }
        };
        Self { tool: query.tool(), artifact_key, selection, include_excerpt, cursor: None }
    }
}

/// Hash exact canonical JSON incrementally, admitting each byte chunk before hashing.
struct HashWriter {
    /// Actual SHA256 state; no retained canonical query byte buffer.
    hash: Sha256,
    /// Complete encoded bytes already observed by this nonretaining writer.
    bytes: usize,
    /// Complete fixed canonical argument capacity.
    limit: usize,
}

impl Write for HashWriter {
    /// Refuse a chunk before touching digest state if the full capacity would overflow.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|size| *size <= self.limit)
            .ok_or_else(|| io::Error::other("query-bound"))?;
        self.hash.update(bytes);
        Ok(bytes.len())
    }

    /// No buffered data or I/O is hidden by a successful hash flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Count complete encoded values without retaining a serialized buffer.
pub(super) fn count(value: &impl Serialize, limit: usize) -> QueryResult<usize> {
    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, value).map_err(|_| Reason::OutputBoundExceeded)?;
    Ok(counter.bytes)
}

/// Nonretaining capped byte count; not a heap, CPU or parser preemption claim.
struct Counter {
    /// Exact encoded byte total already admitted.
    bytes: usize,
    /// Immutable whole counter ceiling before each supplied chunk.
    limit: usize,
}

impl Write for Counter {
    /// Admit encoded chunks using checked complete arithmetic before increment.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|size| *size <= self.limit)
            .ok_or_else(|| io::Error::other("query-bound"))?;
        Ok(bytes.len())
    }

    /// No retained output needs flushing.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Precharge each complete bounded row before retaining its array member.
pub(super) fn retain<T: Serialize>(
    rows: &mut Vec<T>,
    row: T,
    charge: &mut usize,
) -> QueryResult<()> {
    if rows.len() >= MAX_PAGE {
        return Err(Reason::OutputBoundExceeded.into());
    }
    let size = count(&row, MAX_RESPONSE)?;
    let next = charge
        .checked_add(size)
        .and_then(|size| size.checked_add(1))
        .filter(|size| *size <= MAX_RESPONSE)
        .ok_or(Reason::OutputBoundExceeded)?;
    *charge = next;
    rows.push(row);
    Ok(())
}

/// Admit current first-profile safe metadata tokens without clipping or normalization.
pub(super) fn safe_token(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

/// Apply exact minimized identity domain after native intrinsic/schema validation.
fn validate_identity(identity: &NativeIdentity) -> QueryResult<()> {
    if !safe_token(&identity.document_version)
        || !safe_token(&identity.oscal_version)
        || identity.root_id.len() > 45
        || uuid::Uuid::parse_str(&identity.root_id).is_err()
    {
        return Err(Reason::VisibilityRefused.into());
    }
    Ok(())
}

/// Preserve the runtime's sticky typed stop on every traversal/preparation fence.
pub(super) fn checkpoint(control: &mut dyn WorkControl) -> QueryResult<()> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if let Some(reason) = control.interruption() {
        return Err(QueryError::Work(WorkError::Interrupted(reason)));
    }
    Ok(())
}

/// Bound Unicode alphanumeric lowercase tokens before retaining derived query state.
fn tokens(query: &str) -> QueryResult<Vec<String>> {
    if query.is_empty() || query.len() > MAX_QUERY || query.chars().any(char::is_control) {
        return Err(Reason::NotFound.into());
    }
    let mut result = Vec::new();
    let mut token = String::new();
    for character in query.chars().chain(std::iter::once(' ')) {
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                if token.len() + lowered.len_utf8() > MAX_QUERY {
                    return Err(Reason::NotFound.into());
                }
                token.push(lowered);
            }
        } else if !token.is_empty() {
            if result.contains(&token) {
                token.clear();
            } else {
                if result.len() >= MAX_TOKENS {
                    return Err(Reason::NotFound.into());
                }
                result.push(std::mem::take(&mut token));
            }
        }
    }
    if result.is_empty() {
        return Err(Reason::NotFound.into());
    }
    Ok(result)
}

/// Compare one source token incrementally; overlong tokens never match by prefix.
fn contains_token(text: &str, token: &str, control: &mut dyn WorkControl) -> QueryResult<bool> {
    let mut matched = 0;
    let mut mismatch = false;
    for (index, character) in text.chars().chain(std::iter::once(' ')).enumerate() {
        if index % 4096 == 0 {
            checkpoint(control)?;
        }
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                let mut buffer = [0; 4];
                let bytes = lowered.encode_utf8(&mut buffer).as_bytes();
                if token.as_bytes().get(matched..matched + bytes.len()) != Some(bytes) {
                    mismatch = true;
                }
                matched = matched.saturating_add(bytes.len());
            }
        } else {
            if !mismatch && matched == token.len() {
                return Ok(true);
            }
            matched = 0;
            mismatch = false;
        }
    }
    Ok(false)
}

/// File-backed controls consume only the genuine scoped loader and real query preparation.
#[cfg(test)]
#[path = "queries_tests.rs"]
pub(super) mod tests;

/// Actual /2 prepared dispatch retains each private native owner's complete data and fences.
pub(super) enum PreparedDispatchV2<'scope, 'control, C: WorkControl + ?Sized> {
    /// Existing recorded App preparation and byte-ticket behavior remain unchanged.
    App(recorded::v2::PreparedQueryV2<'scope, 'control, C>),
    /// Five ordinary native projections share the same original accepted owner.
    Ordinary(ordinary_v2::PreparedOrdinaryV2<'scope, 'control, C>),
}
impl<C: WorkControl + ?Sized> PreparedDispatchV2<'_, '_, C> {
    /// Borrow exact typed data from its actual retained producer.
    pub(super) fn response(&self) -> &QueryResponse {
        match self {
            Self::App(value) => value.response(),
            Self::Ordinary(value) => value.response(),
        }
    }
    /// Repeat the genuine producer's complete physical verification after encoding.
    pub(super) fn verify_inputs(&self) -> crate::workspace::preparation::WorkResult<()> {
        match self {
            Self::App(value) => value.verify_inputs(),
            Self::Ordinary(value) => value.verify_inputs(),
        }
    }
}

/// Select genuine preparation without a /1 fallback or conversion of DTOs into owners.
pub(super) fn prepare_v2<'scope, 'control, C: WorkControl + ?Sized>(
    scope: &'scope super::native_sources_v2::ServerNativeScopeV2<'control, C>,
    query: &Query,
) -> QueryResult<PreparedDispatchV2<'scope, 'control, C>> {
    match query {
        Query::GetRecordedApplicability { .. } | Query::GetGapSummary { .. } => {
            recorded::v2::prepare(scope, query).map(PreparedDispatchV2::App)
        }
        _ => ordinary_v2::prepare(scope, query).map(PreparedDispatchV2::Ordinary),
    }
}

/// Encode and validate a complete /2 wire envelope on the original owner before publication.
pub(super) fn encode_v2<C: WorkControl + ?Sized>(
    scope: &super::native_sources_v2::ServerNativeScopeV2<'_, C>,
    response: &QueryResponse,
    id: &super::wire::RequestId,
    catalog: &super::catalog::Catalog,
    is_error: bool,
) -> QueryResult<super::capture_v2::AdmittedValue<Vec<u8>>> {
    recorded::v2::encode(scope, response, id, catalog, is_error)
}

/// Five ordinary projections consume only the genuine complete /2 ServerRead factory.
#[path = "ordinary_queries_v2.rs"]
mod ordinary_v2;

/// Exact native selectors use the actual 4096-byte domain, with original UTF-8 spelling.
pub(super) fn native_selector(value: &str) -> bool {
    (1..=4096).contains(&value.len())
}

/// /2 intake expands only native GetRequirement/Trace selectors; all /1 admission stays separate.
pub(super) fn validate_query_v2(query: &Query) -> QueryResult<()> {
    let (key, selection, page) = match query {
        Query::GetRequirement { artifact_key, requirement_id, .. } => {
            (Some(artifact_key.as_str()), Some(requirement_id.as_str()), None)
        }
        Query::TraceControl { artifact_key, control_id, page } => {
            (Some(artifact_key.as_str()), Some(control_id.as_str()), Some(page))
        }
        Query::SearchRequirements { query, artifact_key, page, .. } => {
            if query.is_empty() || query.len() > MAX_QUERY || query.chars().any(char::is_control) {
                return Err(Reason::NotFound.into());
            }
            (artifact_key.as_deref(), None, Some(page))
        }
        _ => return validate_query(query),
    };
    if key.is_some_and(|key| !safe_token(key))
        || selection.is_some_and(|id| !native_selector(id))
        || page.is_some_and(|page| {
            !(1..=MAX_PAGE).contains(&page.limit)
                || page.cursor.as_ref().is_some_and(|cursor| cursor.len() > 256)
        })
    {
        return Err(Reason::NotFound.into());
    }
    Ok(())
}

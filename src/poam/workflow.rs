//! Proposed author-supplied POA&M item workflow over the unchanged F07 source profile.
//!
//! These checks validate declarations, references and schedule arithmetic. They do
//! not authenticate actors, verify remediation or approve the proposed closure policy.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::{DateTime, Duration, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

use super::manifest::{
    self, DocumentManifest, PartyManifest, RoleManifest, SourceManifest, SourceReference,
};
use super::report::SourceInventory;
use super::source::{self, PreparedSource};
use crate::ForgeError;
use crate::json_strict::{self, Limits};

/// Complete authored milestone ceiling across all items, before graph construction.
pub const MAX_MILESTONES: usize = 10_000;
/// Complete item and milestone event ceiling, without per-item prefix admission.
pub const MAX_HISTORY_EVENTS: usize = 100_000;
/// Inclusive bounded report and native artifact byte ceiling.
pub const MAX_OUTPUT_BYTES: usize = 10 * 1024 * 1024;
/// Explicit maximum due-soon interval supplied by the caller.
pub const MAX_DUE_SOON_DAYS: u16 = 365;

/// Closed authored manifest; the existing foundation scaffold parser remains separate.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowManifest {
    /// Existing `forge.poam/1` version, not a new workspace index or role.
    pub schema_version: String,
    /// Explicit immutable plan key and authored title, version and modification time.
    pub document: DocumentManifest,
    /// Exact F07 AR result tuple and four companion declarations.
    pub source: SourceManifest,
    /// Declared roles; assignment validates references, not authority.
    pub roles: Vec<RoleManifest>,
    /// Declared persons or organizations; no identity authentication is performed.
    pub parties: Vec<PartyManifest>,
    /// Explicit nonempty author-selected work; absent sources are not inferred as no-action.
    pub items: Vec<WorkItem>,
}

/// One immutable work identity with explicit ownership, target and status history.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkItem {
    /// Immutable authored key used by the existing item UUID-v5 protocol.
    pub key: String,
    /// Author-supplied remediation title, never generated from source prose.
    pub title: String,
    /// Author-supplied intended work or outcome description.
    pub description: String,
    /// Exact result/kind/key/native-UUID/computed-object-hash selections.
    pub source_refs: Vec<SourceReference>,
    /// Responsible role/party declarations with explicit ownership rationale.
    pub owners: Vec<Owner>,
    /// Explicit full-date target; no date is generated from wall-clock time.
    pub target_date: String,
    /// Explicit current assertion, equal to the final item's history event.
    pub state: State,
    /// Chronological immutable event identities, beginning with an attributed planned event.
    pub history: Vec<Event>,
    /// Author-ordered milestone DAG; every dependency precedes its dependent record.
    pub milestones: Vec<Milestone>,
}

/// Declared responsibility; this does not authenticate or authorize the party.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    /// Exact role ID declared in the same manifest.
    pub role_id: String,
    /// Exact stable party key declared in the same manifest.
    pub party_key: String,
    /// Author's reason for this responsibility assignment.
    pub rationale: String,
}

/// Explicit actor reference used by every status and reviewer assertion.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    /// Exact role ID declared in the same manifest.
    pub role_id: String,
    /// Exact stable party key declared in the same manifest.
    pub party_key: String,
}

/// Human assertion states; none represent verified effectiveness or acceptance authority.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    /// Explicit plan with no claimed work completion.
    Planned,
    /// Work is asserted to be underway.
    InProgress,
    /// Work is asserted blocked, with attributed rationale.
    Blocked,
    /// Completion is asserted, subject to proposed reviewer evidence validation.
    CompletedAsserted,
    /// Risk acceptance is asserted; actor authority remains unverified.
    AcceptedRiskAsserted,
    /// Work is explicitly cancelled; no finding or risk is changed.
    Cancelled,
}

impl State {
    /// Identify states that cannot transition again under the same work identity.
    #[must_use]
    pub const fn terminal(self) -> bool {
        matches!(self, Self::CompletedAsserted | Self::AcceptedRiskAsserted | Self::Cancelled)
    }

    /// Return the exact manifest label used in deterministic reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::InProgress => "in-progress",
            Self::Blocked => "blocked",
            Self::CompletedAsserted => "completed-asserted",
            Self::AcceptedRiskAsserted => "accepted-risk-asserted",
            Self::Cancelled => "cancelled",
        }
    }
}

/// One immutable status assertion, with optional proposed terminal review record.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    /// Immutable event key; baseline guards reject rewriting an existing event.
    pub key: String,
    /// Explicit party and role making this assertion.
    pub actor: Actor,
    /// RFC3339 time supplied by the author, not synthesized by Forge.
    pub at: String,
    /// Prior state; only the initial planned event has no prior state.
    pub from: Option<State>,
    /// Next assertion state under the proposed finite transition table.
    pub to: State,
    /// Explicit rationale for this event, including the initial plan.
    pub rationale: String,
    /// Reviewer assertions required only for completion or risk-acceptance transitions.
    pub closure: Option<Closure>,
}

/// Proposed terminal review evidence profile; approval of its sufficiency remains open.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Closure {
    /// Explicit different party reviewing the terminal assertion.
    pub reviewer: Actor,
    /// Explicit RFC3339 review time, no earlier than the status assertion.
    pub reviewed_at: String,
    /// Explicit review rationale, without an inference of decision authority.
    pub rationale: String,
    /// Bounded metadata pins asserted by the reviewer; no evidence content is fetched.
    pub evidence: Vec<EvidenceAssertion>,
}

/// Reviewer-declared evidence metadata, not a fresh PRD060 verification result.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAssertion {
    /// Immutable author-supplied evidence key.
    pub key: String,
    /// Explicit confined relative reference without absolute paths or network access.
    pub href: String,
    /// Lowercase asserted SHA256; bytes are not read by this first workflow lane.
    pub expected_sha256: String,
}

/// One accountable milestone with explicit DAG edges, target and assertion history.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Milestone {
    /// Immutable milestone key scoped by its parent work item and plan.
    pub key: String,
    /// Explicit author-supplied intended outcome.
    pub outcome: String,
    /// Explicit full-date milestone target, no later than the item's target.
    pub target_date: String,
    /// Exact earlier milestone keys within this item; no cross-item dependencies.
    pub depends_on: Vec<String>,
    /// Accountable role/party declarations with ownership rationale.
    pub owners: Vec<Owner>,
    /// Current assertion, equal to the final milestone history event.
    pub state: State,
    /// Chronological attributed assertions; risk acceptance is unsupported on milestones.
    pub history: Vec<Event>,
}

/// Complete minimized schedule; actor names, paths, evidence locations and rationale omitted.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ScheduleReport {
    /// Exact proposed report format identifier.
    pub schema_version: &'static str,
    /// Canonical explicit report full date.
    pub as_of: String,
    /// Explicit caller-supplied inclusive due-soon interval.
    pub due_soon_days: u16,
    /// Complete declared source inventory denominator before author selection.
    pub source_objects: usize,
    /// Complete author-selected item denominator, including terminal work.
    pub items: usize,
    /// Complete milestone denominator, including terminal records.
    pub milestones: usize,
    /// Every item followed by its ordered milestones, sorted by item key.
    pub rows: Vec<ScheduleRow>,
    /// Count of open records with target strictly before as-of.
    pub overdue: usize,
    /// Count of open records with target in the inclusive as-of/due-soon interval.
    pub due_soon: usize,
    /// Count of records whose state at as-of is blocked.
    pub blocked: usize,
    /// Fixed assertion boundary; no effectiveness or actor authority was verified.
    pub boundary: &'static str,
}

/// One complete schedule record, including historical or future-first-event scope.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ScheduleRow {
    /// Stable native UUID-v5 item or milestone identity.
    pub uuid: String,
    /// Immutable item key, escaped by consumers when displayed in a terminal.
    pub item_key: String,
    /// Optional immutable milestone key; null identifies an item row.
    pub milestone_key: Option<String>,
    /// Explicit canonical full-date target.
    pub target_date: String,
    /// Last declared state whose UTC date is no later than as-of; null means no assertion yet.
    pub state: Option<State>,
    /// Deterministic strictly-late condition for open work at as-of.
    pub overdue: bool,
    /// Deterministic inclusive near-target condition for open work at as-of.
    pub due_soon: bool,
}

/// Captured source plus bounded native/report bytes prepared before any output write.
#[derive(Debug)]
pub struct PreparedWorkflow {
    /// Actual F07 input generations retained for revalidation before publication.
    source: PreparedSource,
    /// Complete schema-validated typed native projection with a single final newline.
    artifact: Vec<u8>,
    /// Complete minimized deterministic schedule bytes with a single final newline.
    schedule: Vec<u8>,
    /// Valid schedule/review action outcome; invalid input never produces this object.
    review_required: bool,
}

impl PreparedWorkflow {
    /// Borrow complete bounded artifact bytes without granting publication authority.
    #[must_use]
    pub fn artifact(&self) -> &[u8] {
        &self.artifact
    }

    /// Borrow complete bounded schedule bytes without actor names or private paths.
    #[must_use]
    pub fn schedule(&self) -> &[u8] {
        &self.schedule
    }

    /// Select exit1 for valid overdue/due-soon/blocked actions, otherwise exit0.
    #[must_use]
    pub fn review_required(&self) -> bool {
        self.review_required
    }

    /// Require the original five captured source identities and bytes before output.
    ///
    /// The Root-owned command must independently retain/recheck manifest and baseline bytes.
    /// # Errors
    /// Returns an error when an original confined input generation changes or is unsafe.
    pub fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.source.verify_inputs()
    }
}

/// Strict-parse complete authored declarations without reading any source artifacts.
/// # Errors
/// Rejects duplicate/unknown fields, unbounded input, invalid workflow rules or undisposed completion/risk-acceptance history.
pub fn parse(bytes: &[u8]) -> Result<WorkflowManifest, ForgeError> {
    if bytes.len() as u64 > manifest::MAX_MANIFEST_BYTES {
        return Err(error("manifest exceeds the 4 MiB raw bound"));
    }
    let value = json_strict::parse_value(
        bytes,
        "POA&M workflow",
        Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("invalid bounded duplicate-free workflow JSON"))?;
    let manifest: WorkflowManifest = serde_json::from_value(value)
        .map_err(|_| error("workflow fields or types do not match the closed contract"))?;
    validate_shape(&manifest)?;
    refuse_undisposed_closure(&manifest)?;
    Ok(manifest)
}

/// Prepare a complete native artifact and deterministic report from actual F07 capture.
///
/// No writes, network, inferred owners/dates, or source mutation occur. Terminal
/// sufficiency and the editable source-of-truth remain proposed owner dispositions.
/// Completion and risk-acceptance assertions are refused by public admission.
/// # Errors
/// Rejects invalid authored declarations, exact source drift, output bounds or native schema errors.
pub fn prepare(
    path: &Path,
    bytes: &[u8],
    as_of: &str,
    due_soon_days: u16,
    baseline: Option<&[u8]>,
) -> Result<PreparedWorkflow, ForgeError> {
    let manifest = parse(bytes)?;
    let date = full_date(as_of)?;
    if due_soon_days > MAX_DUE_SOON_DAYS {
        return Err(error("due-soon interval exceeds 365 days"));
    }
    if let Some(previous) = baseline {
        validate_successor(&parse(previous)?, &manifest)?;
    }
    let source = source::load(path, &manifest.source)?;
    validate_selection(&manifest, &source)?;
    let report = schedule(&manifest, source.inventory(), date, due_soon_days)?;
    let artifact = super::workflow_model::render(&manifest)?;
    let schedule = super::workflow_model::json_line(&report)?;
    source.verify_inputs()?;
    Ok(PreparedWorkflow {
        source,
        artifact,
        schedule,
        review_required: report.overdue > 0 || report.due_soon > 0 || report.blocked > 0,
    })
}

/// Keep completed/risk-accepted assertions outside shipping admission until the recorded closure disposition exists.
fn refuse_undisposed_closure(manifest: &WorkflowManifest) -> Result<(), ForgeError> {
    let reviewed =
        |event: &Event| matches!(event.to, State::CompletedAsserted | State::AcceptedRiskAsserted);
    if manifest.items.iter().any(|item| {
        item.history.iter().any(reviewed)
            || item.milestones.iter().any(|step| step.history.iter().any(reviewed))
    }) {
        return Err(error(
            "completion and risk-acceptance assertions require the pending recorded closure disposition",
        ));
    }
    Ok(())
}

/// Validate every authored record, reference, history and whole collection bound.
fn validate_shape(manifest: &WorkflowManifest) -> Result<(), ForgeError> {
    if manifest.schema_version != manifest::MANIFEST_SCHEMA_VERSION {
        return Err(error("unsupported manifest version"));
    }
    key(&manifest.document.key)?;
    text(&manifest.document.title)?;
    text(&manifest.document.version)?;
    let modified = time(&manifest.document.last_modified)?;
    manifest::validate_source(&manifest.source)?;
    collection(manifest.roles.len(), 1, manifest::MAX_PARTIES)?;
    collection(manifest.parties.len(), 1, manifest::MAX_PARTIES)?;
    collection(manifest.items.len(), 1, manifest::MAX_ITEMS)?;
    preflight_whole_counts(manifest)?;
    let mut roles = BTreeSet::new();
    for role in &manifest.roles {
        key(&role.id)?;
        text(&role.title)?;
        if !roles.insert(role.id.as_str()) {
            return Err(error("duplicate role ID"));
        }
    }
    let mut parties = BTreeSet::new();
    for party in &manifest.parties {
        key(&party.key)?;
        text(&party.name)?;
        if !parties.insert(party.key.as_str()) {
            return Err(error("duplicate party key"));
        }
    }
    let mut keys = BTreeSet::new();
    for (index, item) in manifest.items.iter().enumerate() {
        if !keys.insert(item.key.as_str()) {
            return Err(error("duplicate item key"));
        }
        validate_item(item, index, &manifest.source.result.uuid, modified, &roles, &parties)?;
    }
    Ok(())
}

/// Admit complete milestone/history counts before expanding any item dependency graph.
fn preflight_whole_counts(manifest: &WorkflowManifest) -> Result<(), ForgeError> {
    let mut milestones = 0usize;
    let mut events = 0usize;
    for item in &manifest.items {
        milestones = add_bound(milestones, item.milestones.len(), MAX_MILESTONES)?;
        events = add_bound(events, item.history.len(), MAX_HISTORY_EVENTS)?;
        for step in &item.milestones {
            events = add_bound(events, step.history.len(), MAX_HISTORY_EVENTS)?;
        }
    }
    Ok(())
}

/// Validate one explicit item and its full selection, attributed history and milestone closure relation.
fn validate_item(
    item: &WorkItem,
    index: usize,
    result_uuid: &str,
    modified: DateTime<FixedOffset>,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    key(&item.key)?;
    text(&item.title)?;
    text(&item.description)?;
    validate_owners(&item.owners, roles, parties)?;
    let target = full_date(&item.target_date)?;
    collection(item.source_refs.len(), 1, manifest::MAX_REFERENCES)?;
    let mut selected = BTreeSet::new();
    for reference in &item.source_refs {
        manifest::validate_source_reference("$.items.source_refs", reference)?;
        if reference.result_uuid != result_uuid
            || !selected.insert((reference.kind, &reference.key))
        {
            return Err(error("source references duplicate or belong to another result"));
        }
    }
    validate_history(&item.history, item.state, modified, roles, parties)
        .map_err(|cause| locate(&format!("$.items[{index}].history"), &cause))?;
    validate_milestones(item, index, target, modified, roles, parties)?;
    if item.state == State::AcceptedRiskAsserted
        && !item.source_refs.iter().any(|reference| reference.kind == manifest::SourceKind::Risk)
    {
        return Err(error("risk acceptance requires an explicitly selected native risk"));
    }
    if item.state == State::CompletedAsserted {
        validate_completion(item)?;
    }
    Ok(())
}

/// Validate the whole author-ordered local DAG with stable index-only diagnostic paths.
fn validate_milestones(
    item: &WorkItem,
    index: usize,
    target: NaiveDate,
    modified: DateTime<FixedOffset>,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    collection(item.milestones.len(), 1, manifest::MAX_REFERENCES)?;
    let mut earlier = BTreeMap::new();
    for (step_index, step) in item.milestones.iter().enumerate() {
        key(&step.key)?;
        text(&step.outcome)?;
        let due = full_date(&step.target_date)?;
        if due > target {
            return Err(error("milestone target exceeds item target"));
        }
        let path = format!("$.items[{index}].milestones[{step_index}]");
        let mut dependencies = BTreeSet::new();
        collection(step.depends_on.len(), 0, manifest::MAX_REFERENCES)?;
        for dependency in &step.depends_on {
            key(dependency)?;
            if !dependencies.insert(dependency)
                || earlier.get(dependency.as_str()).is_none_or(|prior| *prior > due)
            {
                return Err(locate(
                    &format!("{path}.depends_on"),
                    &error(
                        "dependency is duplicate, later, missing, cyclic or scheduled after its dependent",
                    ),
                ));
            }
        }
        if earlier.insert(step.key.as_str(), due).is_some() {
            return Err(locate(&path, &error("duplicate milestone key")));
        }
        validate_owners(&step.owners, roles, parties)?;
        if step.history.iter().any(|event| event.to == State::AcceptedRiskAsserted) {
            return Err(error("milestones cannot assert risk acceptance"));
        }
        validate_history(&step.history, step.state, modified, roles, parties)
            .map_err(|cause| locate(&format!("{path}.history"), &cause))?;
    }
    Ok(())
}

/// Proposed completion relation requires every milestone review to precede the item completion assertion.
fn validate_completion(item: &WorkItem) -> Result<(), ForgeError> {
    let completion =
        item.history.last().ok_or_else(|| error("item completion history is absent"))?;
    let at = time(&completion.at)?;
    for step in &item.milestones {
        if step.state != State::CompletedAsserted {
            return Err(error(
                "item completion requires every milestone's explicit completion assertion",
            ));
        }
        let event =
            step.history.last().ok_or_else(|| error("milestone completion history is absent"))?;
        let reviewed =
            event.closure.as_ref().ok_or_else(|| error("milestone completion review is absent"))?;
        if time(&reviewed.reviewed_at)? > at {
            return Err(error("item completion precedes milestone review"));
        }
    }
    Ok(())
}

/// Attach only a generated index path to an already fixed workflow validation error.
fn locate(path: &str, cause: &ForgeError) -> ForgeError {
    ForgeError::PoamBuild(format!("{path}: {cause}"))
}

/// Verify source selection against the complete captured result inventory, without eligibility inference.
fn validate_selection(
    manifest: &WorkflowManifest,
    prepared: &PreparedSource,
) -> Result<(), ForgeError> {
    let inventory = prepared.inventory();
    if manifest.source.assessment_results.expected_sha256 != inventory.source_sha256
        || manifest.source.result.uuid != inventory.result_uuid
        || manifest.source.result.key != inventory.result_key
    {
        return Err(error(
            "captured source inventory does not match the exact manifest source tuple",
        ));
    }
    for item in &manifest.items {
        source::validate_selection(prepared, &item.source_refs)?;
    }
    Ok(())
}

/// Require every owner binding to resolve declared role/party keys with explicit rationale.
fn validate_owners(
    owners: &[Owner],
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    collection(owners.len(), 1, 64)?;
    let mut seen = BTreeSet::new();
    for owner in owners {
        validate_actor(
            &Actor { role_id: owner.role_id.clone(), party_key: owner.party_key.clone() },
            roles,
            parties,
        )?;
        text(&owner.rationale)?;
        if !seen.insert((&owner.role_id, &owner.party_key)) {
            return Err(error("duplicate responsibility binding"));
        }
    }
    Ok(())
}

/// Validate explicit party/role membership without asserting authenticated identity or authority.
fn validate_actor(
    actor: &Actor,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    if !roles.contains(actor.role_id.as_str()) || !parties.contains(actor.party_key.as_str()) {
        return Err(error("actor role or party is undeclared"));
    }
    Ok(())
}

/// Validate an entire finite history, including explicit initial attribution and terminal review declarations.
fn validate_history(
    history: &[Event],
    state: State,
    modified: DateTime<FixedOffset>,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    collection(history.len(), 1, manifest::MAX_REFERENCES)?;
    let mut keys = BTreeSet::new();
    let mut prior = None;
    let mut prior_time = None;
    for event in history {
        key(&event.key)?;
        text(&event.rationale)?;
        validate_actor(&event.actor, roles, parties)?;
        if !keys.insert(&event.key) {
            return Err(error("duplicate history key"));
        }
        let at = time(&event.at)?;
        if at > modified || prior_time.is_some_and(|last| at <= last) {
            return Err(error(
                "history times are not strictly chronological or exceed last_modified",
            ));
        }
        if event.from != prior || !transition(prior, event.to) {
            return Err(error("history has an invalid prior state or transition"));
        }
        let reviewed = matches!(event.to, State::CompletedAsserted | State::AcceptedRiskAsserted);
        if reviewed {
            let closure = event
                .closure
                .as_ref()
                .ok_or_else(|| error("terminal assertion requires explicit reviewer evidence"))?;
            validate_actor(&closure.reviewer, roles, parties)?;
            text(&closure.rationale)?;
            let reviewed_at = time(&closure.reviewed_at)?;
            if closure.reviewer.party_key == event.actor.party_key
                || reviewed_at < at
                || reviewed_at > modified
            {
                return Err(error(
                    "review must name a different party and a coherent explicit time",
                ));
            }
            collection(closure.evidence.len(), 1, 64)?;
            let mut evidence = BTreeSet::new();
            for reference in &closure.evidence {
                key(&reference.key)?;
                if !evidence.insert(&reference.key) {
                    return Err(error("duplicate closure evidence key"));
                }
                evidence_href(&reference.href)?;
                json_strict::validate_lowercase_sha256(
                    "closure.evidence.expected_sha256",
                    &reference.expected_sha256,
                )
                .map_err(|_| error("closure evidence hash must be lowercase SHA256"))?;
            }
        } else if event.closure.is_some() {
            return Err(error(
                "review evidence is only allowed on completion or risk-acceptance assertions",
            ));
        }
        prior = Some(event.to);
        prior_time = Some(at);
    }
    if prior != Some(state) {
        return Err(error("current state does not match final history event"));
    }
    Ok(())
}

/// Proposed finite transition table; every terminal state requires a new explicitly linked identity to reopen.
fn transition(from: Option<State>, to: State) -> bool {
    match from {
        None => to == State::Planned,
        Some(State::Planned) => matches!(
            to,
            State::InProgress | State::Blocked | State::AcceptedRiskAsserted | State::Cancelled
        ),
        Some(State::InProgress) => matches!(
            to,
            State::Blocked
                | State::CompletedAsserted
                | State::AcceptedRiskAsserted
                | State::Cancelled
        ),
        Some(State::Blocked) => {
            matches!(to, State::InProgress | State::AcceptedRiskAsserted | State::Cancelled)
        }
        Some(State::CompletedAsserted | State::AcceptedRiskAsserted | State::Cancelled) => false,
    }
}

/// Require unchanged prior event prefixes and immutable identities when a baseline is explicitly supplied.
///
/// This is a history integrity guard, not the full M13 baseline impact report or proof
/// of changes before the supplied baseline. Removal and terminal identity reuse are refused.
/// # Errors
/// Rejects plan-key changes, removed identities, rewritten history or terminal work revisions.
pub fn validate_successor(
    previous: &WorkflowManifest,
    current: &WorkflowManifest,
) -> Result<(), ForgeError> {
    validate_shape(previous)?;
    validate_shape(current)?;
    refuse_undisposed_closure(previous)?;
    refuse_undisposed_closure(current)?;
    if previous.document.key != current.document.key {
        return Err(error("baseline plan key differs"));
    }
    if time(&current.document.last_modified)? < time(&previous.document.last_modified)? {
        return Err(error("baseline modification time moved backward"));
    }
    let items: BTreeMap<_, _> =
        current.items.iter().map(|item| (item.key.as_str(), item)).collect();
    for old in &previous.items {
        let item = items.get(old.key.as_str()).ok_or_else(|| {
            error("baseline item removal requires the future explicit supersession workflow")
        })?;
        prefix(&old.history, &item.history)?;
        if old.state.terminal() && *old != **item {
            return Err(error("terminal item revision requires a new explicitly linked identity"));
        }
        let steps: BTreeMap<_, _> =
            item.milestones.iter().map(|step| (step.key.as_str(), step)).collect();
        for old_step in &old.milestones {
            let step = steps
                .get(old_step.key.as_str())
                .ok_or_else(|| error("baseline milestone removal is unsupported"))?;
            prefix(&old_step.history, &step.history)?;
            if old_step.state.terminal() && *old_step != **step {
                return Err(error("terminal milestone revision is unsupported"));
            }
        }
    }
    Ok(())
}

/// Require exact attributed historical records before allowing only new appended events.
fn prefix(previous: &[Event], current: &[Event]) -> Result<(), ForgeError> {
    if !current.starts_with(previous) {
        return Err(error("baseline history was removed, reordered or rewritten"));
    }
    Ok(())
}

/// Produce all item/milestone rows using explicit UTC-date history selection and no wall clock.
fn schedule(
    manifest: &WorkflowManifest,
    inventory: &SourceInventory,
    as_of: NaiveDate,
    due_soon_days: u16,
) -> Result<ScheduleReport, ForgeError> {
    if due_soon_days > MAX_DUE_SOON_DAYS {
        return Err(error("due-soon interval exceeds 365 days"));
    }
    let through = as_of
        .checked_add_signed(Duration::days(i64::from(due_soon_days)))
        .ok_or_else(|| error("due-soon date range overflow"))?;
    let mut items: Vec<_> = manifest.items.iter().collect();
    items.sort_by(|a, b| a.key.cmp(&b.key));
    let mut rows = Vec::new();
    for item in items {
        rows.push(schedule_row(&manifest.document.key, item, None, as_of, through)?);
        for milestone in &item.milestones {
            rows.push(schedule_row(&manifest.document.key, item, Some(milestone), as_of, through)?);
        }
    }
    Ok(ScheduleReport {
        schema_version: "forge.poam-schedule/1",
        as_of: as_of.to_string(),
        due_soon_days,
        source_objects: inventory.objects.len(),
        items: manifest.items.len(),
        milestones: rows.len() - manifest.items.len(),
        overdue: rows.iter().filter(|row| row.overdue).count(),
        due_soon: rows.iter().filter(|row| row.due_soon).count(),
        blocked: rows.iter().filter(|row| row.state == Some(State::Blocked)).count(),
        rows,
        boundary: "Author and reviewer assertions only; no effectiveness, evidence freshness or actor authority verified.",
    })
}

/// Derive one row while retaining terminal and not-yet-asserted records in the denominator.
fn schedule_row(
    plan_key: &str,
    item: &WorkItem,
    milestone: Option<&Milestone>,
    as_of: NaiveDate,
    through: NaiveDate,
) -> Result<ScheduleRow, ForgeError> {
    let (history, target_date, uuid) = match milestone {
        Some(step) => (
            &step.history,
            &step.target_date,
            super::identity::milestone(plan_key, &item.key, &step.key),
        ),
        None => (&item.history, &item.target_date, super::identity::item(plan_key, &item.key)),
    };
    let mut state = None;
    for event in history {
        if time(&event.at)?.with_timezone(&chrono::Utc).date_naive() <= as_of {
            state = Some(event.to);
        }
    }
    let target = full_date(target_date)?;
    let open = state.is_some_and(|value| !value.terminal());
    Ok(ScheduleRow {
        uuid: uuid.to_string(),
        item_key: item.key.clone(),
        milestone_key: milestone.map(|step| step.key.clone()),
        target_date: target.to_string(),
        state,
        overdue: open && target < as_of,
        due_soon: open && target >= as_of && target <= through,
    })
}

/// Require a canonical Gregorian full date, including valid leap days and four-digit years.
fn full_date(value: &str) -> Result<NaiveDate, ForgeError> {
    if value.len() != 10 {
        return Err(error("date must be YYYY-MM-DD"));
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| error("invalid Gregorian full date"))?;
    if date.to_string() != value {
        return Err(error("date must use canonical YYYY-MM-DD"));
    }
    Ok(date)
}

/// Parse one explicit RFC3339 timestamp without defaulting or consulting current time.
fn time(value: &str) -> Result<DateTime<FixedOffset>, ForgeError> {
    DateTime::parse_from_rfc3339(value).map_err(|_| error("timestamp must be RFC3339"))
}

/// Validate an immutable exact key without whitespace normalization or terminal controls.
fn key(value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > 256
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(error(
            "key must be exact nonempty trimmed text within 256 bytes without controls",
        ));
    }
    Ok(())
}

/// Validate author prose while allowing ordinary line breaks but rejecting terminal controls.
fn text(value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty()
        || value.len() > manifest::MAX_STRING_BYTES
        || value.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(error(
            "authored text is empty, unbounded or contains terminal control characters",
        ));
    }
    Ok(())
}

/// Bound a complete collection before validation; no prefix earns admission.
fn collection(count: usize, minimum: usize, maximum: usize) -> Result<(), ForgeError> {
    if !(minimum..=maximum).contains(&count) {
        return Err(error("collection cardinality is outside the complete bound"));
    }
    Ok(())
}

/// Charge a complete whole-manifest count with checked arithmetic before graph/report growth.
fn add_bound(total: usize, count: usize, maximum: usize) -> Result<usize, ForgeError> {
    total
        .checked_add(count)
        .filter(|value| *value <= maximum)
        .ok_or_else(|| error("whole workflow collection bound exceeded"))
}

/// Require a bounded relative evidence metadata reference; no bytes, URIs or authority are resolved.
fn evidence_href(value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > 4096
        || value.contains(['\\', ':', '?', '#'])
        || value.starts_with('/')
        || value.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
        || value.chars().any(char::is_control)
    {
        return Err(error("evidence declaration must use a confined relative reference"));
    }
    Ok(())
}

/// Return a fixed typed workflow error without embedding arbitrary author prose or paths.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// Syntactic artifact pin; these unit declarations are not native capture proof.
    fn pin(name: &str) -> Value {
        json!({"artifact":format!("{name}.json"),"href":format!("{name}.json"),
            "expected_sha256":"a".repeat(64),"root_uuid":"11111111-1111-4111-8111-111111111111",
            "document_version":"1.0.0","oscal_version":"1.2.3"})
    }

    /// Attributed initial event used by item and milestone declarations.
    fn initial() -> Value {
        json!({"key":"initial","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-01T00:00:00Z","from":null,"to":"planned","rationale":"SENSITIVE INITIAL RATIONALE","closure":null})
    }

    /// Explicit responsibility; no actor or rationale is generated by production.
    fn owner() -> Value {
        json!({"role_id":"owner","party_key":"alice","rationale":"SENSITIVE OWNERSHIP RATIONALE"})
    }

    /// Closed syntactic nonterminal workflow with a selected exact source tuple.
    fn declaration() -> Value {
        json!({"schema_version":"forge.poam/1","document":{"key":"plan","title":"Authored plan","version":"1.0.0","last_modified":"2026-02-20T00:00:00Z"},
            "source":{"assessment_results":pin("ar"),"result":{"uuid":"11111111-1111-4111-8111-111111111111","key":"result"},
                "context":{"assessment_plan":pin("ap"),"ssp":pin("ssp"),"profile":pin("profile"),"catalog":pin("catalog")}},
            "roles":[{"id":"owner","title":"Owner"},{"id":"reviewer","title":"Reviewer"}],
            "parties":[{"key":"alice","type":"person","name":"SENSITIVE OWNER NAME"},{"key":"bob","type":"person","name":"SENSITIVE REVIEWER NAME"}],
            "items":[{"key":"work","title":"Authored work","description":"Authored remediation description",
                "source_refs":[{"kind":"finding","key":"finding","uuid":"22222222-2222-4222-8222-222222222222","result_uuid":"11111111-1111-4111-8111-111111111111","expected_sha256":"b".repeat(64)}],
                "owners":[owner()],"target_date":"2026-02-01","state":"planned","history":[initial()],
                "milestones":[{"key":"step","outcome":"Authored outcome","target_date":"2026-01-20","depends_on":[],"owners":[owner()],"state":"planned","history":[initial()]}]}]})
    }

    /// Feed declaration text through the actual strict closed parser.
    fn parsed(value: &Value) -> Result<WorkflowManifest, ForgeError> {
        parse(&serde_json::to_vec(value).unwrap())
    }

    /// Inspect only the proposed structural closure validator; never creates a public preparation or output.
    fn proposal(value: &Value) -> Result<WorkflowManifest, ForgeError> {
        let manifest: WorkflowManifest = serde_json::from_value(value.clone())
            .map_err(|_| error("closed proposal shape invalid"))?;
        validate_shape(&manifest)?;
        Ok(manifest)
    }

    /// Construct a reviewer assertion with metadata-only evidence; no content is read.
    fn closure() -> Value {
        json!({"reviewer":{"role_id":"reviewer","party_key":"bob"},"reviewed_at":"2026-01-10T12:00:00Z",
            "rationale":"SENSITIVE REVIEW RATIONALE","evidence":[{"key":"proof","href":"evidence/proof.json","expected_sha256":"c".repeat(64)}]})
    }

    /// Append one explicit assertion to a JSON work record without mutating production state.
    fn event(record: &mut Value, key: &str, at: &str, from: &str, to: &str, review: Value) {
        let mut next = json!({"key":key,"actor":{"role_id":"owner","party_key":"alice"},
            "at":at,"from":from,"to":to,"rationale":"Explicit event rationale"});
        next["closure"] = review;
        record["history"].as_array_mut().unwrap().push(next);
        record["state"] = json!(to);
    }

    /// Detached inventory is only a schedule denominator fixture, never a source qualification.
    fn inventory() -> SourceInventory {
        SourceInventory {
            schema_version: super::super::report::INVENTORY_SCHEMA_VERSION,
            validation_scope: "source-integrity-only",
            workflow_validated: false,
            source_sha256: "a".repeat(64),
            result_uuid: "11111111-1111-4111-8111-111111111111".to_string(),
            result_key: "result".to_string(),
            objects: vec![],
        }
    }

    /// Positive authored fields remain exact and both item/milestone identities ignore mutable prose.
    #[test]
    fn nonterminal_fields_and_key_scoped_identities_are_explicit() {
        let mut value = declaration();
        let first = parsed(&value).unwrap();
        value["items"][0]["title"] = json!("Edited title");
        value["items"][0]["target_date"] = json!("2026-03-01");
        value["items"][0]["owners"][0]["rationale"] = json!("Edited ownership rationale");
        value["items"][0]["milestones"][0]["outcome"] = json!("Edited explicit outcome");
        value["items"][0]["milestones"][0]["target_date"] = json!("2026-02-05");
        let second = parsed(&value).unwrap();
        assert_eq!(
            super::super::identity::item(&first.document.key, &first.items[0].key),
            super::super::identity::item(&second.document.key, &second.items[0].key)
        );
        assert_eq!(
            super::super::identity::milestone(
                &first.document.key,
                &first.items[0].key,
                &first.items[0].milestones[0].key
            ),
            super::super::identity::milestone(
                &second.document.key,
                &second.items[0].key,
                &second.items[0].milestones[0].key
            )
        );
        assert_ne!(
            super::super::identity::milestone("plan", "work", "step"),
            super::super::identity::milestone("plan", "work", "rekeyed")
        );
        assert_eq!(second.items[0].owners[0].party_key, "alice");
        assert_eq!(second.items[0].target_date, "2026-03-01");
        assert!(manifest::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    /// Unknown keys at every new nested authored shape fail before any source read.
    #[test]
    fn unknown_nested_fields_are_not_silently_discarded() {
        for pointer in [
            "",
            "/items/0",
            "/items/0/owners/0",
            "/items/0/history/0",
            "/items/0/history/0/actor",
            "/items/0/milestones/0",
        ] {
            let mut value = declaration();
            value
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".to_string(), json!(true));
            assert!(parsed(&value).is_err(), "pointer {pointer}");
        }
        let mut value = declaration();
        event(
            &mut value["items"][0],
            "accept",
            "2026-01-10T00:00:00Z",
            "planned",
            "accepted-risk-asserted",
            closure(),
        );
        value["items"][0]["source_refs"][0]["kind"] = json!("risk");
        for pointer in [
            "/items/0/history/1/closure",
            "/items/0/history/1/closure/reviewer",
            "/items/0/history/1/closure/evidence/0",
        ] {
            let mut copy = value.clone();
            copy.pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".to_string(), json!(true));
            assert!(proposal(&copy).is_err());
        }
    }

    /// Duplicate fields, trailing JSON, unsupported versions and whole raw overflow fail closed.
    #[test]
    fn strict_json_and_complete_raw_bounds_are_preserved() {
        let text = serde_json::to_string(&declaration()).unwrap();
        assert!(
            parse(
                text.replace(
                    "\"schema_version\":\"forge.poam/1\"",
                    "\"schema_version\":\"forge.poam/1\",\"schema_version\":\"forge.poam/1\""
                )
                .as_bytes()
            )
            .is_err()
        );
        assert!(parse(format!("{text} {{}}").as_bytes()).is_err());
        let mut value = declaration();
        value["schema_version"] = json!("forge.poam/2");
        assert!(parsed(&value).is_err());
        assert!(
            parse(&vec![b' '; usize::try_from(manifest::MAX_MANIFEST_BYTES).unwrap() + 1]).is_err()
        );
    }

    /// Owners and event actors must resolve declared role/party pairs with explicit rationale.
    #[test]
    fn undeclared_or_empty_owner_and_actor_records_fail() {
        for (pointer, replacement) in [
            ("/items/0/owners/0/role_id", json!("missing")),
            ("/items/0/owners/0/party_key", json!("missing")),
            ("/items/0/owners/0/rationale", json!(" ")),
            ("/items/0/history/0/actor/party_key", json!("missing")),
            ("/items/0/milestones/0/owners", json!([])),
        ] {
            let mut value = declaration();
            *value.pointer_mut(pointer).unwrap() = replacement;
            assert!(parsed(&value).is_err());
        }
        let mut value = declaration();
        let duplicate = value["items"][0]["owners"][0].clone();
        value["items"][0]["owners"].as_array_mut().unwrap().push(duplicate);
        assert!(parsed(&value).is_err());
    }

    /// Every finite transition pair is checked against an explicit proposed edge set.
    #[test]
    fn all_state_pairs_follow_the_proposed_table_and_terminal_states_do_not_reopen() {
        let states = [
            State::Planned,
            State::InProgress,
            State::Blocked,
            State::CompletedAsserted,
            State::AcceptedRiskAsserted,
            State::Cancelled,
        ];
        let edges = [
            (State::Planned, State::InProgress),
            (State::Planned, State::Blocked),
            (State::Planned, State::AcceptedRiskAsserted),
            (State::Planned, State::Cancelled),
            (State::InProgress, State::Blocked),
            (State::InProgress, State::CompletedAsserted),
            (State::InProgress, State::AcceptedRiskAsserted),
            (State::InProgress, State::Cancelled),
            (State::Blocked, State::InProgress),
            (State::Blocked, State::AcceptedRiskAsserted),
            (State::Blocked, State::Cancelled),
        ];
        for from in states {
            for to in states {
                assert_eq!(
                    transition(Some(from), to),
                    edges.contains(&(from, to)),
                    "{from:?} -> {to:?}"
                );
            }
        }
        for to in states {
            assert_eq!(transition(None, to), to == State::Planned);
        }
    }

    /// Full-date parsing rejects invalid leap days, alternate spellings and implicit date/time values.
    #[test]
    fn calendar_validation_and_no_hidden_as_of_default() {
        assert!(full_date("2024-02-29").is_ok());
        for value in ["2026-02-29", "2026-1-01", "2026-01-01T00:00:00Z", "2026-13-01", ""] {
            assert!(full_date(value).is_err());
        }
        let mut value = declaration();
        value["items"][0]["target_date"] = json!("2026-02-29");
        assert!(parsed(&value).is_err());
    }

    /// DAG validation rejects self edges, forward/missing dependencies, duplicates and reverse dates.
    #[test]
    fn milestone_cycles_order_and_target_relations_fail() {
        for dependencies in [json!(["step"]), json!(["missing"]), json!(["step", "step"])] {
            let mut value = declaration();
            value["items"][0]["milestones"][0]["depends_on"] = dependencies;
            assert!(
                parsed(&value)
                    .unwrap_err()
                    .to_string()
                    .contains("$.items[0].milestones[0].depends_on")
            );
        }
        let mut value = declaration();
        let mut next = value["items"][0]["milestones"][0].clone();
        next["key"] = json!("next");
        next["depends_on"] = json!(["step"]);
        next["target_date"] = json!("2026-01-25");
        value["items"][0]["milestones"].as_array_mut().unwrap().push(next);
        assert!(parsed(&value).is_ok());
        value["items"][0]["milestones"][1]["target_date"] = json!("2026-01-19");
        assert!(parsed(&value).is_err());
        value["items"][0]["milestones"][1]["target_date"] = json!("2026-02-02");
        assert!(parsed(&value).is_err());
    }

    /// History requires coherent states, strictly increasing actual instants and explicit modification ceilings.
    #[test]
    fn history_time_prior_state_and_final_state_are_coherent() {
        let mut value = declaration();
        event(
            &mut value["items"][0],
            "start",
            "2026-01-02T00:00:00Z",
            "planned",
            "in-progress",
            Value::Null,
        );
        assert!(parsed(&value).is_ok());
        for (pointer, replacement) in [
            ("/items/0/history/1/at", json!("2026-01-01T01:00:00+01:00")),
            ("/items/0/history/1/from", json!("blocked")),
            ("/items/0/state", json!("planned")),
            ("/items/0/history/1/at", json!("2026-03-01T00:00:00Z")),
        ] {
            let mut copy = value.clone();
            *copy.pointer_mut(pointer).unwrap() = replacement;
            assert!(parsed(&copy).is_err());
        }
    }

    /// Risk acceptance requires an explicit risk source and a different coherent reviewer with evidence metadata.
    #[test]
    fn terminal_assertions_require_the_proposed_review_profile() {
        let mut value = declaration();
        value["items"][0]["source_refs"][0]["kind"] = json!("risk");
        event(
            &mut value["items"][0],
            "accept",
            "2026-01-10T00:00:00Z",
            "planned",
            "accepted-risk-asserted",
            closure(),
        );
        assert!(proposal(&value).is_ok());
        for (pointer, replacement) in [
            ("/items/0/history/1/closure", Value::Null),
            ("/items/0/history/1/closure/reviewer/party_key", json!("alice")),
            ("/items/0/history/1/closure/evidence", json!([])),
            ("/items/0/history/1/closure/evidence/0/href", json!("/private/proof")),
            ("/items/0/history/1/closure/evidence/0/expected_sha256", json!("C".repeat(64))),
            ("/items/0/source_refs/0/kind", json!("finding")),
            ("/items/0/history/1/closure/reviewed_at", json!("2026-01-09T00:00:00Z")),
        ] {
            let mut copy = value.clone();
            *copy.pointer_mut(pointer).unwrap() = replacement;
            assert!(proposal(&copy).is_err());
        }
    }

    /// Completion cannot precede every explicit milestone completion and its review assertion.
    #[test]
    fn item_completion_requires_completed_and_reviewed_milestones() {
        let mut value = declaration();
        for pointer in ["/items/0", "/items/0/milestones/0"] {
            event(
                value.pointer_mut(pointer).unwrap(),
                "start",
                "2026-01-02T00:00:00Z",
                "planned",
                "in-progress",
                Value::Null,
            );
        }
        event(
            &mut value["items"][0]["milestones"][0],
            "done",
            "2026-01-10T00:00:00Z",
            "in-progress",
            "completed-asserted",
            closure(),
        );
        let mut review = closure();
        review["reviewed_at"] = json!("2026-01-11T12:00:00Z");
        event(
            &mut value["items"][0],
            "done",
            "2026-01-11T00:00:00Z",
            "in-progress",
            "completed-asserted",
            review,
        );
        assert!(proposal(&value).is_ok());
        assert!(parsed(&value).unwrap_err().to_string().contains("recorded closure disposition"));
        let mut copy = value.clone();
        copy["items"][0]["history"][2]["at"] = json!("2026-01-10T01:00:00Z");
        assert!(proposal(&copy).is_err());
        let mut copy = value;
        copy["items"][0]["milestones"][0]["history"].as_array_mut().unwrap().pop();
        copy["items"][0]["milestones"][0]["state"] = json!("in-progress");
        assert!(proposal(&copy).is_err());
    }

    /// Public parsing and preparation cannot admit otherwise structurally valid proposed terminal review records.
    #[test]
    fn shipping_admission_refuses_undisposed_closure_without_a_public_bypass() {
        let mut value = declaration();
        value["items"][0]["source_refs"][0]["kind"] = json!("risk");
        event(
            &mut value["items"][0],
            "accept",
            "2026-01-10T00:00:00Z",
            "planned",
            "accepted-risk-asserted",
            closure(),
        );
        assert!(proposal(&value).is_ok());
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(parse(&bytes).unwrap_err().to_string().contains("recorded closure disposition"));
        assert!(
            prepare(Path::new("missing/manifest.json"), &bytes, "2026-01-20", 7, None)
                .unwrap_err()
                .to_string()
                .contains("recorded closure disposition")
        );
    }

    /// Complete history prefixes cannot be rewritten or removed; valid nonterminal appends remain supported.
    #[test]
    fn explicitly_supplied_baseline_enforces_exact_append_only_events() {
        let old = parsed(&declaration()).unwrap();
        let mut value = declaration();
        event(
            &mut value["items"][0],
            "start",
            "2026-01-02T00:00:00Z",
            "planned",
            "in-progress",
            Value::Null,
        );
        let current = parsed(&value).unwrap();
        assert!(validate_successor(&old, &current).is_ok());
        let mut copy = value.clone();
        copy["items"][0]["history"][0]["rationale"] = json!("Rewritten");
        assert!(validate_successor(&old, &parsed(&copy).unwrap()).is_err());
        let mut copy = value;
        copy["items"][0]["milestones"][0]["key"] = json!("replacement");
        assert!(validate_successor(&old, &parsed(&copy).unwrap()).is_err());
    }

    /// Terminal work cannot be revised under the same identity, even when the prior history is preserved.
    #[test]
    fn terminal_baseline_identity_cannot_silently_change_description_or_owner() {
        let mut value = declaration();
        event(
            &mut value["items"][0],
            "cancel",
            "2026-01-02T00:00:00Z",
            "planned",
            "cancelled",
            Value::Null,
        );
        let old = parsed(&value).unwrap();
        assert!(validate_successor(&old, &old).is_ok());
        value["items"][0]["description"] = json!("Changed terminal work");
        assert!(validate_successor(&old, &parsed(&value).unwrap()).is_err());
    }

    /// Schedule retains complete item/milestone rows and deterministic explicit-date boundaries without prose.
    #[test]
    fn schedule_full_denominators_overdue_due_soon_and_redaction_are_deterministic() {
        let plan = parsed(&declaration()).unwrap();
        let date = full_date("2026-01-21").unwrap();
        let first = schedule(&plan, &inventory(), date, 11).unwrap();
        let second = schedule(&plan, &inventory(), date, 11).unwrap();
        assert_eq!(first, second);
        assert_eq!((first.items, first.milestones, first.rows.len()), (1, 1, 2));
        assert_eq!((first.overdue, first.due_soon), (1, 1));
        let bytes = super::super::workflow_model::json_line(&first).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        for private in ["SENSITIVE", "alice", "bob", "ar.json", "proof.json", "rationale"] {
            assert!(!text.contains(private));
        }
        assert!(schedule(&plan, &inventory(), date, 366).is_err());
    }

    /// As-of state uses declared UTC dates, preserving rows before first assertion and excluding later terminal events.
    #[test]
    fn schedule_as_of_before_and_after_terminal_history_preserves_rows() {
        let mut value = declaration();
        event(
            &mut value["items"][0],
            "cancel",
            "2026-01-22T00:00:00Z",
            "planned",
            "cancelled",
            Value::Null,
        );
        let plan = parsed(&value).unwrap();
        let before = schedule(&plan, &inventory(), full_date("2025-12-31").unwrap(), 7).unwrap();
        assert_eq!(before.rows.len(), 2);
        assert!(before.rows.iter().all(|row| row.state.is_none() && !row.overdue && !row.due_soon));
        let earlier = schedule(&plan, &inventory(), full_date("2026-01-21").unwrap(), 11).unwrap();
        assert_eq!(earlier.rows[0].state, Some(State::Planned));
        let later = schedule(&plan, &inventory(), full_date("2026-02-02").unwrap(), 7).unwrap();
        assert_eq!(later.rows[0].state, Some(State::Cancelled));
        assert!(!later.rows[0].overdue);
        assert_eq!(later.overdue, 1);
    }

    /// Typed projection validates the pinned schema without fabricated native local findings or risks.
    #[test]
    fn native_projection_is_schema_valid_and_keeps_complete_authored_fields() {
        let plan = parsed(&declaration()).unwrap();
        let one = super::super::workflow_model::render(&plan).unwrap();
        let two = super::super::workflow_model::render(&plan).unwrap();
        assert_eq!(one, two);
        let value: Value = serde_json::from_slice(&one).unwrap();
        let root = &value["plan-of-action-and-milestones"];
        assert_eq!(root["uuid"], super::super::identity::document("plan").to_string());
        assert_eq!(root["back-matter"]["resources"].as_array().unwrap().len(), 5);
        assert!(root.get("findings").is_none());
        assert!(root.get("risks").is_none());
        assert!(root["poam-items"][0].get("related-findings").is_none());
        let props = root["poam-items"][0]["props"].as_array().unwrap();
        for name in ["owners", "target-date", "history", "milestones", "source-references"] {
            assert!(props.iter().any(|property| property["name"] == name));
        }
        let milestones =
            props.iter().find(|property| property["name"] == "milestones").unwrap()["value"]
                .as_str()
                .unwrap();
        let milestones: Value = serde_json::from_str(milestones).unwrap();
        assert_eq!(
            milestones[0]["uuid"],
            super::super::identity::milestone("plan", "work", "step").to_string()
        );
        let owners = props.iter().find(|property| property["name"] == "owners").unwrap()["value"]
            .as_str()
            .unwrap();
        let owners: Value = serde_json::from_str(owners).unwrap();
        assert_eq!(owners[0]["party_uuid"], root["metadata"]["parties"][0]["uuid"]);
        assert!(one.ends_with(b"\n"));
    }

    /// Multi-record native declaration properties preserve authored newlines as escaped JSON, not forbidden physical LF.
    ///
    /// This is a syntactic native-projection fixture; it does not establish actual
    /// source capture, actor authority, terminal admission or interoperability.
    #[test]
    fn native_compact_properties_preserve_multiple_records_and_authored_newlines_and_tabs() {
        let mut value = declaration();
        value["items"][0]["owners"][0]["rationale"] = json!("Owner line one\nOwner\tline two");
        value["items"][0]["owners"].as_array_mut().unwrap().push(json!({
            "role_id":"owner", "party_key":"bob", "rationale":"Second owner\nexplicit\trationale"
        }));
        value["items"][0]["history"][0]["rationale"] = json!("Event line one\nEvent\tline two");
        value["items"][0]["milestones"][0]["outcome"] =
            json!("Outcome line one\nOutcome\tline two");
        let mut second = value["items"][0].clone();
        second["key"] = json!("second-work");
        value["items"].as_array_mut().unwrap().push(second);
        let manifest = parsed(&value).unwrap();
        let artifact = super::super::workflow_model::render(&manifest).unwrap();
        let again = super::super::workflow_model::render(&manifest).unwrap();
        assert_eq!(artifact, again);
        assert!(artifact.ends_with(b"\n"));
        assert!(artifact.strip_suffix(b"\n").unwrap().contains(&b'\n'));
        let native: Value = serde_json::from_slice(&artifact).unwrap();
        let items = native["plan-of-action-and-milestones"]["poam-items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
        for item in items {
            let properties = item["props"].as_array().unwrap();
            for name in ["owners", "history", "milestones", "source-references"] {
                let text =
                    properties.iter().find(|property| property["name"] == name).unwrap()["value"]
                        .as_str()
                        .unwrap();
                assert!(!text.contains(['\n', '\r', '\t']));
                let decoded: Value = serde_json::from_str(text).unwrap();
                match name {
                    "owners" => {
                        assert_eq!(decoded.as_array().unwrap().len(), 2);
                        assert_eq!(decoded[0]["rationale"], "Owner line one\nOwner\tline two");
                        assert_eq!(decoded[1]["rationale"], "Second owner\nexplicit\trationale");
                    }
                    "history" => {
                        assert_eq!(decoded[0]["rationale"], "Event line one\nEvent\tline two");
                    }
                    "milestones" => {
                        assert_eq!(decoded[0]["outcome"], "Outcome line one\nOutcome\tline two");
                    }
                    "source-references" => {
                        assert_eq!(decoded, value["items"][0]["source_refs"]);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Whole counters, unsafe keys and output expansion are rejected without prefix publication.
    #[test]
    fn complete_counts_controls_and_output_limits_fail_closed() {
        assert_eq!(
            add_bound(MAX_HISTORY_EVENTS - 1, 1, MAX_HISTORY_EVENTS).unwrap(),
            MAX_HISTORY_EVENTS
        );
        assert!(add_bound(MAX_HISTORY_EVENTS, 1, MAX_HISTORY_EVENTS).is_err());
        assert!(add_bound(usize::MAX, 1, MAX_HISTORY_EVENTS).is_err());
        let mut value = declaration();
        value["items"][0]["key"] = json!("work\u{1b}[31m");
        assert!(parsed(&value).is_err());
        value["items"][0]["key"] = json!("work");
        value["items"][0]["description"] = json!("x".repeat(manifest::MAX_STRING_BYTES + 1));
        assert!(parsed(&value).is_err());
        assert!(super::super::workflow_model::json_line(&"x".repeat(MAX_OUTPUT_BYTES)).is_err());
    }
}

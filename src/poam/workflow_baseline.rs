//! Readonly author-declaration comparison against an actual current source capture.
//!
//! No comparison creates a native artifact or weakens workflow admission. Prior
//! declarations receive a bounded structural profile, not prior freshness or
//! authority. Removals, rewritten history, terminal mutations and explicit reopen
//! declarations remain visible refusals rather than accepted artifact revisions.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::manifest::SourceReference;
use super::report::SourceInventory;
use super::source::PreparedSource;
use super::workflow::{Milestone, State, WorkItem, WorkflowManifest};
use super::{identity, manifest, source, workflow};
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};

/// Complete comparison report version, independent of schedule and native schemas.
pub const BASELINE_SCHEMA_VERSION: &str = "forge.poam-baseline/1";

/// Exact declared identity relation; this is not accepted reopening or a native link.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReopenDeclaration {
    /// Immutable key of the prior item asserted as superseded by this declaration.
    pub previous_item_key: String,
    /// Distinct new item key already present in the current author declaration.
    pub current_item_key: String,
    /// Exact original baseline file digest supplied by the ordinary caller.
    pub previous_manifest_sha256: String,
}

/// Item and milestone denominators remain separate complete dimensions.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EntityKind {
    /// A work item uses the frozen plan/item UUID-v5 identity protocol.
    Item,
    /// A milestone uses the frozen plan/item/milestone UUID-v5 protocol.
    Milestone,
}

/// Union membership is descriptive and grants no removal or addition authority.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Presence {
    /// Record exists only in the current declaration.
    Added,
    /// Record exists only in the prior declaration and remains a build refusal.
    Removed,
    /// The exact immutable keys are present in both declarations.
    Common,
}

/// Finite change categories retain exact named-field meaning without prose disclosure.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ChangedField {
    /// Responsible role/party IDs or responsibility rationale changed.
    Owners,
    /// Exact authored target date changed; no schedule date is inferred.
    TargetDate,
    /// Exact item title UTF-8 digest changed.
    Title,
    /// Item description or milestone outcome UTF-8 digest changed.
    Outcome,
    /// Final declared state changed; this is not measured effectiveness.
    Status,
    /// History content, attribution, time or rationale changed/appended.
    History,
    /// Exact ordered event-key/rationale/reviewer-rationale digest changed.
    Rationale,
    /// Exact source tuples changed, independent of current native membership.
    Sources,
    /// Author-ordered milestone keys changed for the item row.
    MilestoneOrder,
    /// Explicit milestone dependency keys changed.
    Dependencies,
}

/// Minimal responsibility projection; IDs can still identify people and be sensitive.
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnerProjection {
    /// Exact declared role ID, not authenticated authority.
    pub role_id: String,
    /// Exact party key; party names and contact details are excluded.
    pub party_key: String,
    /// SHA-256 of the complete original responsibility rationale UTF-8 bytes.
    pub rationale_sha256: String,
    /// Digest of the complete same-manifest role declaration, without its title text.
    pub role_declaration_sha256: String,
    /// Digest of the complete same-manifest party declaration, without its private name.
    pub party_declaration_sha256: String,
}

/// Exact current or previous assertion projection; digests are not approval evidence.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RecordProjection {
    /// Complete sorted ownership bindings, including rationale digests.
    pub owners: Vec<OwnerProjection>,
    /// Exact canonical authored target date.
    pub target_date: String,
    /// Exact final declared state, independent of the explicit report as-of.
    pub state: State,
    /// Exact complete item title digest; null for milestones, which have no separate title.
    pub title_sha256: Option<String>,
    /// Exact complete item-description or milestone-outcome digest.
    pub outcome_sha256: String,
    /// Complete ordered typed event digest, including actor/rationale/closure assertions.
    pub history_sha256: String,
    /// Complete ordered event-key/rationale/reviewer-rationale UTF-8 projection digest.
    pub history_rationale_sha256: String,
    /// Complete event denominator, not a hidden matching prefix.
    pub history_events: usize,
    /// Complete sorted current/prior item source tuples; milestone rows have no independent sources.
    pub source_refs: Vec<SourceReference>,
    /// Complete authored milestone order for item rows; empty for milestone rows.
    pub milestone_order: Vec<String>,
    /// Complete explicit milestone dependency keys; empty for item rows.
    pub dependencies: Vec<String>,
}

/// One stable union row, retaining both projections even when the revision is refused.
#[derive(Debug, Serialize)]
pub struct ChangeRow {
    /// Item or milestone identity domain.
    pub kind: EntityKind,
    /// Frozen deterministic native UUID derived only from immutable keys.
    pub uuid: String,
    /// Exact immutable item key.
    pub item_key: String,
    /// Exact milestone key, or null for an item.
    pub milestone_key: Option<String>,
    /// Complete previous/current union membership.
    pub presence: Presence,
    /// Null only when the record is absent from the previous declaration.
    pub previous: Option<RecordProjection>,
    /// Null only when the record is absent from the current declaration.
    pub current: Option<RecordProjection>,
    /// All changed categories, with deterministic finite enum ordering.
    pub changes: Vec<ChangedField>,
    /// Exact prior event prefix is retained; null when no common record exists.
    pub history_prefix_preserved: Option<bool>,
}

/// Whole stable-identity partition, separately reported for item and milestone rows.
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Counts {
    /// All previous records of this identity domain.
    pub previous: usize,
    /// All current records of this identity domain.
    pub current: usize,
    /// Every exact stable identity occurring on both sides.
    pub common: usize,
    /// Every current-only identity.
    pub added: usize,
    /// Every previous-only identity, including refused removals.
    pub removed: usize,
    /// Common records with at least one actual changed category.
    pub changed: usize,
}

/// Exact reference assessment against current capture, not reconstructed prior freshness.
#[derive(Debug, Serialize)]
pub struct ReferenceRow {
    /// Exact item identity associated with this source selection.
    pub item_key: String,
    /// True for a prior declaration row; false for a current row.
    pub previous: bool,
    /// Entire original native selection tuple, with no paths or assessment prose.
    pub reference: SourceReference,
    /// All tuple members match an actual object in the current captured inventory.
    pub matches_current_capture: bool,
}

/// Explicit author relation reported as refused reopening; never inferred from similar prose or refs.
#[derive(Debug, Serialize)]
pub struct ReopenRow {
    /// Exact previous item identity.
    pub previous_uuid: String,
    /// Exact new item identity.
    pub current_uuid: String,
    /// Exact previous immutable key.
    pub previous_item_key: String,
    /// Exact distinct current immutable key.
    pub current_item_key: String,
    /// Fixed false until a separately approved terminal/supersession admission exists.
    pub accepted_for_build: bool,
}

/// Closed descriptive refusal categories; none changes the existing artifact admission policy.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Refusal {
    /// The readonly prior scaffold is empty and cannot be an artifact baseline under existing guards.
    EmptyPrevious,
    /// Current inspection is explicitly empty and cannot be built as authored work.
    EmptyCurrent,
    /// Prior or current history contains still-undisposed completion/risk acceptance.
    UndisposedTerminal,
    /// Exact current document modification time moved backward.
    ModificationTimeRegressed,
    /// Prior item or milestone identity is absent in current declarations.
    RemovedIdentity,
    /// Common history is not an exact original prefix.
    RewrittenHistory,
    /// A terminal record changed under its original item or milestone identity.
    TerminalMutation,
    /// Current source tuples fail exact current native membership.
    CurrentReferenceStale,
    /// Explicit proposed reopening remains outside accepted build authority.
    ReopeningUnadmitted,
}

/// Complete source-derived readonly comparison; no native output can be obtained from it.
#[derive(Debug, Serialize)]
pub struct BaselineReport {
    /// Exact baseline comparison format identifier.
    pub schema_version: &'static str,
    /// Canonical explicit calendar full date, not wall-clock evaluation.
    pub as_of: String,
    /// Exact original prior declaration byte digest, including formatting.
    pub previous_manifest_sha256: String,
    /// Exact original current declaration byte digest, including formatting.
    pub current_manifest_sha256: String,
    /// Exact declared prior document modification time.
    pub previous_last_modified: String,
    /// Exact declared current document modification time.
    pub current_last_modified: String,
    /// Complete original current captured Assessment Results file digest.
    pub current_source_sha256: String,
    /// Exact current selected result UUID.
    pub current_result_uuid: String,
    /// Exact current selected result stable key.
    pub current_result_key: String,
    /// Prior declaration profile is bounded shape/history only, not historical freshness.
    pub previous_validation: &'static str,
    /// Actual current source membership is evaluated against the complete five-file capture.
    pub reference_basis: &'static str,
    /// Full previous/current source declaration digest comparison, without exposing paths.
    pub source_declaration_changed: bool,
    /// Whether unchanged existing successor guards and actual current selections permit revision.
    pub revision_rules_compatible: bool,
    /// All current fixed refusal observations; never arbitrary source exception messages.
    pub refusal_observations: Vec<Refusal>,
    /// Always false: comparison does not render/validate a native artifact or authorize publication.
    pub artifact_validated: bool,
    /// Complete item identity partition.
    pub items: Counts,
    /// Complete milestone identity partition.
    pub milestones: Counts,
    /// Stable identity rows sorted by item/milestone keys.
    pub rows: Vec<ChangeRow>,
    /// Every previous and current source selection against current capture, including failures.
    pub references: Vec<ReferenceRow>,
    /// Prior references not matching current capture; no assertion about the prior epoch is made.
    pub previous_refs_not_current: usize,
    /// Current references not matching current capture; any nonzero count refuses artifact revision.
    pub current_refs_not_current: usize,
    /// Complete explicit proposed reopen relations, all refused for build.
    pub reopen_declarations: Vec<ReopenRow>,
    /// Fixed assertion, authority, privacy and comparison scope boundary.
    pub boundary: &'static str,
}

/// Actual five-source capture and complete bounded report retained before ordinary caller output.
#[derive(Debug)]
pub struct PreparedComparison {
    /// Real native input generations, never a fabricated detached inventory.
    source: PreparedSource,
    /// Complete deterministic bounded report; no partial prefix is returned.
    bytes: Vec<u8>,
    /// Descriptive revision-rule comparison, not native artifact validation or a publication permit.
    compatible: bool,
    /// Complete changed/refused comparison requires review, independently of artifact compatibility.
    review_required: bool,
}

impl PreparedComparison {
    /// Borrow the complete readonly comparison without granting native output authority.
    #[must_use]
    pub fn report(&self) -> &[u8] {
        &self.bytes
    }

    /// Report whether the existing revision guards and current selection tuples hold.
    #[must_use]
    pub fn revision_rules_compatible(&self) -> bool {
        self.compatible
    }

    /// Identify complete changed/refused observations for the readonly caller's exit-one mapping.
    #[must_use]
    pub fn review_required(&self) -> bool {
        self.review_required
    }

    /// Revalidate every actual captured source generation before report publication.
    ///
    /// The ordinary caller must also retain/recheck actual current/prior/reopen input
    /// identities and bytes; this method does not replace that separate obligation.
    /// # Errors
    /// Rejects changed, missing or unsafe actual source generations.
    pub fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.source.verify_inputs()
    }
}

/// Prepare readonly M13 comparison from original declarations and an actual current native capture.
///
/// Both raw author declarations and optional reopen data are duplicate-safe and bounded.
/// Current references can be reported stale, but receive no artifact authority. Terminal
/// and removal observations remain descriptive refusals. Prior source files are not loaded
/// and their historic freshness or authority is never inferred from a current comparison.
/// # Errors
/// Refuses invalid complete structural/history input, distinct plan keys, malformed explicit
/// reopen relations, actual current five-source capture failure, invalid explicit date or
/// a report exceeding the unchanged complete 10 MiB output bound.
pub fn prepare(
    path: &Path,
    current_bytes: &[u8],
    previous_bytes: &[u8],
    reopen_bytes: Option<&[u8]>,
    as_of: &str,
) -> Result<PreparedComparison, ForgeError> {
    let current = parse_declaration(current_bytes)?;
    let previous = parse_declaration(previous_bytes)?;
    comparison_date(as_of)?;
    if previous.document.key != current.document.key {
        return Err(error("comparison plan keys differ"));
    }
    let previous_sha = sha256_hex(previous_bytes);
    let reopens = parse_reopens(reopen_bytes)?;
    let source = source::load(path, &current.source)
        .map_err(|_| error("current comparison source capture is invalid or stale"))?;
    let report = compare(
        &previous,
        &current,
        source.inventory(),
        &previous_sha,
        &sha256_hex(current_bytes),
        as_of,
        &reopens,
    )?;
    let compatible = report.revision_rules_compatible;
    let review_required = review_required(&report);
    let bytes = super::workflow_model::json_line(&report)?;
    source.verify_inputs()?;
    Ok(PreparedComparison { source, bytes, compatible, review_required })
}

/// Apply the existing ten-byte canonical full-date admission without signed or expanded years.
fn comparison_date(value: &str) -> Result<(), ForgeError> {
    if value.len() != 10 {
        return Err(error("comparison date must be canonical YYYY-MM-DD"));
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| error("invalid explicit comparison date"))?;
    if date.to_string() != value {
        return Err(error("comparison date must be canonical YYYY-MM-DD"));
    }
    Ok(())
}

/// Parse the bounded readonly declaration profile for preflight; this admits no source or artifact authority.
///
/// Root's CLI may inspect declared input paths before output collision preflight.
/// The caller still must capture/recheck the original declarations and `prepare`
/// must establish actual current five-file source identity separately.
/// # Errors
/// Rejects duplicate, malformed, unknown or structurally invalid whole declaration bytes.
pub(crate) fn parse_declaration(bytes: &[u8]) -> Result<WorkflowManifest, ForgeError> {
    let value = bounded_value(bytes)?;
    let value: WorkflowManifest = serde_json::from_value(value)
        .map_err(|_| error("comparison declaration is not the closed workflow shape"))?;
    workflow::validate_comparison_shape(&value)?;
    Ok(value)
}

/// Strict-parse the complete original bytes under existing authoring declaration limits.
fn bounded_value(bytes: &[u8]) -> Result<serde_json::Value, ForgeError> {
    if bytes.len() as u64 > manifest::MAX_MANIFEST_BYTES {
        return Err(error("comparison declaration exceeds its complete raw bound"));
    }
    json_strict::parse_value(
        bytes,
        "POA&M comparison",
        Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("comparison JSON is malformed, duplicate or unbounded"))
}

/// Admit only a closed bounded array of explicit relations; no relation is manufactured.
fn parse_reopens(bytes: Option<&[u8]>) -> Result<Vec<ReopenDeclaration>, ForgeError> {
    let Some(bytes) = bytes else {
        return Ok(Vec::new());
    };
    let values: Vec<ReopenDeclaration> = serde_json::from_value(bounded_value(bytes)?)
        .map_err(|_| error("reopen declarations are not a closed relation array"))?;
    if values.len() > manifest::MAX_ITEMS {
        return Err(error("complete reopen relation count exceeds item bound"));
    }
    Ok(values)
}

/// Compare complete stable identities and selections after both shapes are independently admitted.
fn compare(
    previous: &WorkflowManifest,
    current: &WorkflowManifest,
    inventory: &SourceInventory,
    previous_sha: &str,
    current_sha: &str,
    as_of: &str,
    reopens: &[ReopenDeclaration],
) -> Result<BaselineReport, ForgeError> {
    let mut rows = Vec::new();
    let old: BTreeMap<_, _> = previous.items.iter().map(|item| (item.key.as_str(), item)).collect();
    let new: BTreeMap<_, _> = current.items.iter().map(|item| (item.key.as_str(), item)).collect();
    let keys: BTreeSet<_> = old.keys().chain(new.keys()).copied().collect();
    for key in keys {
        item_rows(
            &current.document.key,
            previous,
            current,
            old.get(key).copied(),
            new.get(key).copied(),
            &mut rows,
        )?;
    }
    let mut references = reference_rows(previous, inventory, true);
    references.extend(reference_rows(current, inventory, false));
    let previous_refs_not_current =
        references.iter().filter(|row| row.previous && !row.matches_current_capture).count();
    let current_refs_not_current =
        references.iter().filter(|row| !row.previous && !row.matches_current_capture).count();
    let reopen_declarations = reopen_rows(previous, current, previous_sha, reopens)?;
    let revision_rules_compatible = workflow::validate_successor(previous, current).is_ok()
        && current_refs_not_current == 0
        && reopen_declarations.is_empty();
    let items = counts(&rows, EntityKind::Item);
    let milestones = counts(&rows, EntityKind::Milestone);
    let refusal_observations = refusals(
        previous,
        current,
        &rows,
        current_refs_not_current,
        !reopen_declarations.is_empty(),
    )?;
    Ok(BaselineReport {
        schema_version: BASELINE_SCHEMA_VERSION,
        as_of: as_of.into(),
        previous_manifest_sha256: previous_sha.into(),
        current_manifest_sha256: current_sha.into(),
        previous_last_modified: previous.document.last_modified.clone(),
        current_last_modified: current.document.last_modified.clone(),
        current_source_sha256: inventory.source_sha256.clone(),
        current_result_uuid: inventory.result_uuid.clone(),
        current_result_key: inventory.result_key.clone(),
        previous_validation: "bounded-structural-history-only",
        reference_basis: "complete-current-five-file-capture-only",
        source_declaration_changed: digest(&previous.source)? != digest(&current.source)?,
        revision_rules_compatible,
        refusal_observations,
        artifact_validated: false,
        items,
        milestones,
        rows,
        references,
        previous_refs_not_current,
        current_refs_not_current,
        reopen_declarations,
        boundary: "Author assertions only. Prior freshness and actor authority unverified. Reopen/removal/history refusals never authorize an artifact. IDs can be sensitive; prose, names and private paths are omitted.",
    })
}

/// Derive review from all complete change/refusal dimensions, never revision compatibility alone.
fn review_required(report: &BaselineReport) -> bool {
    let changed = [&report.items, &report.milestones]
        .iter()
        .any(|counts| counts.added > 0 || counts.removed > 0 || counts.changed > 0);
    changed
        || report.source_declaration_changed
        || report.previous_refs_not_current > 0
        || report.current_refs_not_current > 0
        || !report.reopen_declarations.is_empty()
        || !report.revision_rules_compatible
}

/// Retain every independently visible revision refusal with the same UTC instant semantics as the core.
fn refusals(
    previous: &WorkflowManifest,
    current: &WorkflowManifest,
    rows: &[ChangeRow],
    stale: usize,
    reopened: bool,
) -> Result<Vec<Refusal>, ForgeError> {
    let mut values = Vec::new();
    if previous.items.is_empty() {
        values.push(Refusal::EmptyPrevious);
    }
    if current.items.is_empty() {
        values.push(Refusal::EmptyCurrent);
    }
    let undisposed = |manifest: &WorkflowManifest| {
        manifest.items.iter().any(|item| {
            item.history.iter().chain(item.milestones.iter().flat_map(|step| &step.history)).any(
                |event| matches!(event.to, State::CompletedAsserted | State::AcceptedRiskAsserted),
            )
        })
    };
    if undisposed(previous) || undisposed(current) {
        values.push(Refusal::UndisposedTerminal);
    }
    let before = chrono::DateTime::parse_from_rfc3339(&previous.document.last_modified)
        .map_err(|_| error("invalid prior modification time"))?;
    let after = chrono::DateTime::parse_from_rfc3339(&current.document.last_modified)
        .map_err(|_| error("invalid current modification time"))?;
    if after < before {
        values.push(Refusal::ModificationTimeRegressed);
    }
    if rows.iter().any(|row| row.presence == Presence::Removed) {
        values.push(Refusal::RemovedIdentity);
    }
    if rows.iter().any(|row| row.history_prefix_preserved == Some(false)) {
        values.push(Refusal::RewrittenHistory);
    }
    let current_items: BTreeMap<_, _> =
        current.items.iter().map(|item| (item.key.as_str(), item)).collect();
    let terminal_changed = previous.items.iter().any(|item| {
        current_items.get(item.key.as_str()).is_some_and(|next| {
            if item.state.terminal() && item != *next {
                return true;
            }
            let steps: BTreeMap<_, _> =
                next.milestones.iter().map(|step| (step.key.as_str(), step)).collect();
            item.milestones.iter().any(|step| {
                step.state.terminal()
                    && steps.get(step.key.as_str()).is_some_and(|after| step != *after)
            })
        })
    });
    if terminal_changed {
        values.push(Refusal::TerminalMutation);
    }
    if stale > 0 {
        values.push(Refusal::CurrentReferenceStale);
    }
    if reopened {
        values.push(Refusal::ReopeningUnadmitted);
    }
    Ok(values)
}

/// Emit the full item and milestone union while preserving author-defined stable identity.
fn item_rows(
    plan: &str,
    previous_manifest: &WorkflowManifest,
    current_manifest: &WorkflowManifest,
    previous: Option<&WorkItem>,
    current: Option<&WorkItem>,
    rows: &mut Vec<ChangeRow>,
) -> Result<(), ForgeError> {
    let item =
        current.or(previous).ok_or_else(|| error("comparison identity is absent on both sides"))?;
    rows.push(row(
        plan,
        &item.key,
        None,
        previous.map(|item| item_projection(item, previous_manifest)).transpose()?,
        current.map(|item| item_projection(item, current_manifest)).transpose()?,
        previous.zip(current).map(|(old, new)| new.history.starts_with(&old.history)),
    ));
    let old: BTreeMap<_, _> = previous
        .into_iter()
        .flat_map(|item| &item.milestones)
        .map(|step| (step.key.as_str(), step))
        .collect();
    let new: BTreeMap<_, _> = current
        .into_iter()
        .flat_map(|item| &item.milestones)
        .map(|step| (step.key.as_str(), step))
        .collect();
    let keys: BTreeSet<_> = old.keys().chain(new.keys()).copied().collect();
    for key in keys {
        let before = old.get(key).copied();
        let after = new.get(key).copied();
        rows.push(row(
            plan,
            &item.key,
            Some(key),
            before.map(|step| step_projection(step, previous_manifest)).transpose()?,
            after.map(|step| step_projection(step, current_manifest)).transpose()?,
            before.zip(after).map(|(a, b)| b.history.starts_with(&a.history)),
        ));
    }
    Ok(())
}

/// Construct one finite-category row without treating a refused change as absence.
fn row(
    plan: &str,
    item: &str,
    step: Option<&str>,
    previous: Option<RecordProjection>,
    current: Option<RecordProjection>,
    prefix: Option<bool>,
) -> ChangeRow {
    let presence = match (&previous, &current) {
        (Some(_), Some(_)) => Presence::Common,
        (Some(_), None) => Presence::Removed,
        _ => Presence::Added,
    };
    let changes =
        previous.as_ref().zip(current.as_ref()).map(|(a, b)| changes(a, b)).unwrap_or_default();
    ChangeRow {
        kind: if step.is_some() { EntityKind::Milestone } else { EntityKind::Item },
        uuid: step
            .map_or_else(
                || identity::item(plan, item),
                |step| identity::milestone(plan, item, step),
            )
            .to_string(),
        item_key: item.into(),
        milestone_key: step.map(str::to_owned),
        presence,
        previous,
        current,
        changes,
        history_prefix_preserved: prefix,
    }
}

/// List every actual projection change in a fixed, explicit category order.
fn changes(a: &RecordProjection, b: &RecordProjection) -> Vec<ChangedField> {
    let mut fields = Vec::new();
    for (different, field) in [
        (a.owners != b.owners, ChangedField::Owners),
        (a.target_date != b.target_date, ChangedField::TargetDate),
        (a.title_sha256 != b.title_sha256, ChangedField::Title),
        (a.outcome_sha256 != b.outcome_sha256, ChangedField::Outcome),
        (a.state != b.state, ChangedField::Status),
        (a.history_sha256 != b.history_sha256, ChangedField::History),
        (a.history_rationale_sha256 != b.history_rationale_sha256, ChangedField::Rationale),
        (a.source_refs != b.source_refs, ChangedField::Sources),
        (a.milestone_order != b.milestone_order, ChangedField::MilestoneOrder),
        (a.dependencies != b.dependencies, ChangedField::Dependencies),
    ] {
        if different {
            fields.push(field);
        }
    }
    fields
}

/// Project only explicit item fields; no assessment prose is copied into a comparison.
fn item_projection(
    item: &WorkItem,
    manifest: &WorkflowManifest,
) -> Result<RecordProjection, ForgeError> {
    let mut references = item.source_refs.clone();
    references.sort();
    Ok(RecordProjection {
        owners: owners(&item.owners, manifest)?,
        target_date: item.target_date.clone(),
        state: item.state,
        title_sha256: Some(sha256_hex(item.title.as_bytes())),
        outcome_sha256: sha256_hex(item.description.as_bytes()),
        history_sha256: digest(&item.history)?,
        history_rationale_sha256: rationales(&item.history)?,
        history_events: item.history.len(),
        source_refs: references,
        milestone_order: item.milestones.iter().map(|step| step.key.clone()).collect(),
        dependencies: Vec::new(),
    })
}

/// Project milestone outcome/ownership/history without inventing independent source selection.
fn step_projection(
    step: &Milestone,
    manifest: &WorkflowManifest,
) -> Result<RecordProjection, ForgeError> {
    Ok(RecordProjection {
        owners: owners(&step.owners, manifest)?,
        target_date: step.target_date.clone(),
        state: step.state,
        title_sha256: None,
        outcome_sha256: sha256_hex(step.outcome.as_bytes()),
        history_sha256: digest(&step.history)?,
        history_rationale_sha256: rationales(&step.history)?,
        history_events: step.history.len(),
        source_refs: Vec::new(),
        milestone_order: Vec::new(),
        dependencies: step.depends_on.clone(),
    })
}

/// Preserve exact owner bindings and digest their resolved declarations without private prose.
fn owners(
    values: &[workflow::Owner],
    manifest: &WorkflowManifest,
) -> Result<Vec<OwnerProjection>, ForgeError> {
    let mut owners = Vec::new();
    for owner in values {
        let role = manifest
            .roles
            .iter()
            .find(|role| role.id == owner.role_id)
            .ok_or_else(|| error("comparison owner role declaration is absent"))?;
        let party = manifest
            .parties
            .iter()
            .find(|party| party.key == owner.party_key)
            .ok_or_else(|| error("comparison owner party declaration is absent"))?;
        owners.push(OwnerProjection {
            role_id: owner.role_id.clone(),
            party_key: owner.party_key.clone(),
            rationale_sha256: sha256_hex(owner.rationale.as_bytes()),
            role_declaration_sha256: digest(role)?,
            party_declaration_sha256: digest(party)?,
        });
    }
    owners.sort();
    Ok(owners)
}

/// Hash exact ordered event rationale assertions, including optional reviewer rationale, without prose output.
fn rationales(events: &[workflow::Event]) -> Result<String, ForgeError> {
    let values: Vec<_> = events
        .iter()
        .map(|event| {
            (
                &event.key,
                &event.rationale,
                event.closure.as_ref().map(|closure| closure.rationale.as_str()),
            )
        })
        .collect();
    digest(&values)
}

/// Hash complete typed ordered declarations; this differs from original-file digests.
fn digest<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256_hex(&bytes))
        .map_err(|_| error("comparison typed projection serialization failed"))
}

/// Partition the complete stable-identity union without filtering away refused rows.
fn counts(rows: &[ChangeRow], kind: EntityKind) -> Counts {
    let mut count = Counts::default();
    for row in rows.iter().filter(|row| row.kind == kind) {
        match row.presence {
            Presence::Added => {
                count.current += 1;
                count.added += 1;
            }
            Presence::Removed => {
                count.previous += 1;
                count.removed += 1;
            }
            Presence::Common => {
                count.previous += 1;
                count.current += 1;
                count.common += 1;
                if !row.changes.is_empty() {
                    count.changed += 1;
                }
            }
        }
    }
    count
}

/// Evaluate every complete source tuple against actual current inventory, never producer digest assertions.
fn reference_rows(
    manifest: &WorkflowManifest,
    inventory: &SourceInventory,
    previous: bool,
) -> Vec<ReferenceRow> {
    let objects: BTreeMap<_, _> = inventory
        .objects
        .iter()
        .map(|object| ((object.kind, object.key.as_str()), object))
        .collect();
    let mut items: Vec<_> = manifest.items.iter().collect();
    items.sort_by(|a, b| a.key.cmp(&b.key));
    let mut rows = Vec::new();
    for item in items {
        let mut references = item.source_refs.clone();
        references.sort();
        for reference in references {
            let matches =
                objects.get(&(reference.kind, reference.key.as_str())).is_some_and(|object| {
                    object.uuid == reference.uuid
                        && object.result_uuid == reference.result_uuid
                        && object.sha256 == reference.expected_sha256
                });
            rows.push(ReferenceRow {
                item_key: item.key.clone(),
                previous,
                reference,
                matches_current_capture: matches,
            });
        }
    }
    rows
}

/// Require each explicit relation to bind exact old/new keys and raw prior bytes; always refuse reopening for build.
fn reopen_rows(
    previous: &WorkflowManifest,
    current: &WorkflowManifest,
    previous_sha: &str,
    values: &[ReopenDeclaration],
) -> Result<Vec<ReopenRow>, ForgeError> {
    let old: BTreeMap<_, _> = previous.items.iter().map(|item| (item.key.as_str(), item)).collect();
    let new: BTreeMap<_, _> = current.items.iter().map(|item| (item.key.as_str(), item)).collect();
    let mut prior_keys = BTreeSet::new();
    let mut current_keys = BTreeSet::new();
    let mut rows = Vec::new();
    for value in values {
        let before = old
            .get(value.previous_item_key.as_str())
            .ok_or_else(|| error("reopen prior identity is absent"))?;
        if value.previous_manifest_sha256 != previous_sha
            || !before.state.terminal()
            || old.contains_key(value.current_item_key.as_str())
            || !new.contains_key(value.current_item_key.as_str())
            || !prior_keys.insert(&value.previous_item_key)
            || !current_keys.insert(&value.current_item_key)
        {
            return Err(error(
                "explicit reopen relation has an invalid or duplicate exact identity binding",
            ));
        }
        rows.push(ReopenRow {
            previous_uuid: identity::item(&previous.document.key, &value.previous_item_key)
                .to_string(),
            current_uuid: identity::item(&current.document.key, &value.current_item_key)
                .to_string(),
            previous_item_key: value.previous_item_key.clone(),
            current_item_key: value.current_item_key.clone(),
            accepted_for_build: false,
        });
    }
    rows.sort_by(|a, b| {
        a.previous_item_key
            .cmp(&b.previous_item_key)
            .then(a.current_item_key.cmp(&b.current_item_key))
    });
    Ok(rows)
}

/// Report generic invalid comparison input without arbitrary private path/source exception text.
fn error(message: &str) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// Reuse exact frozen syntactic pin data; this fixture is not a native source capture.
    fn pin(name: &str) -> Value {
        json!({"artifact":format!("{name}.json"),"href":format!("{name}.json"),
            "expected_sha256":"a".repeat(64),"root_uuid":"11111111-1111-4111-8111-111111111111",
            "document_version":"1.0.0","oscal_version":"1.2.3"})
    }

    /// Reuse the frozen attributed initial-event test double, without actor authentication.
    fn initial() -> Value {
        json!({"key":"initial","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-01T00:00:00Z","from":null,"to":"planned","rationale":"SENSITIVE INITIAL RATIONALE","closure":null})
    }

    /// Reuse the frozen syntactic ownership test double, without actor authority.
    fn owner() -> Value {
        json!({"role_id":"owner","party_key":"alice","rationale":"SENSITIVE OWNERSHIP RATIONALE"})
    }

    /// Reuse exact frozen authored workflow structure, independently of actual capture proofs.
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

    /// Construct a detached test inventory solely for private comparison predicate controls.
    fn inventory(manifest: &WorkflowManifest) -> SourceInventory {
        let objects = manifest.items[0]
            .source_refs
            .iter()
            .map(|reference| super::super::report::SourceObject {
                kind: reference.kind,
                key: reference.key.clone(),
                uuid: reference.uuid.clone(),
                result_uuid: reference.result_uuid.clone(),
                sha256: reference.expected_sha256.clone(),
                state: "synthetic-only".into(),
                control_ids: Vec::new(),
                declared_content_sha256: None,
                declared_rationale_sha256: None,
            })
            .collect();
        SourceInventory {
            schema_version: super::super::report::INVENTORY_SCHEMA_VERSION,
            validation_scope: "synthetic-unit-only",
            workflow_validated: false,
            source_sha256: manifest.source.assessment_results.expected_sha256.clone(),
            result_uuid: manifest.source.result.uuid.clone(),
            result_key: manifest.source.result.key.clone(),
            objects,
        }
    }

    /// Compare structurally admitted declarations using a disclosed detached inventory double.
    fn compared(previous: &Value, current: &Value) -> BaselineReport {
        let a = serde_json::to_vec(previous).unwrap();
        let b = serde_json::to_vec(current).unwrap();
        let old = parse_declaration(&a).unwrap();
        let new = parse_declaration(&b).unwrap();
        compare(&old, &new, &inventory(&old), &sha256_hex(&a), &sha256_hex(&b), "2026-02-06", &[])
            .unwrap()
    }

    /// Canonical full-date admission refuses signed/expanded years and validates actual leap days.
    #[test]
    fn comparison_date_requires_ten_byte_full_date_and_valid_calendar() {
        for value in
            ["-0001-01-01", "+10000-12-31", "2026-2-06", "2026-02-29", "2024-02-30", "2026-02-06\n"]
        {
            assert!(comparison_date(value).is_err(), "{value:?}");
        }
        for value in ["2024-02-29", "2026-02-06", "0000-01-01", "9999-12-31"] {
            comparison_date(value).unwrap();
        }
    }

    /// Empty-prior additions stay complete observations with the explicit existing artifact-baseline refusal.
    #[test]
    fn empty_prior_preserves_additions_and_explains_artifact_baseline_refusal() {
        let current = declaration();
        let mut previous = current.clone();
        previous["items"] = json!([]);
        previous["roles"] = json!([]);
        previous["parties"] = json!([]);
        let a = serde_json::to_vec(&previous).unwrap();
        let b = serde_json::to_vec(&current).unwrap();
        let old = parse_declaration(&a).unwrap();
        let new = parse_declaration(&b).unwrap();
        let report = compare(
            &old,
            &new,
            &inventory(&new),
            &sha256_hex(&a),
            &sha256_hex(&b),
            "2026-02-06",
            &[],
        )
        .unwrap();
        assert_eq!(
            report.items,
            Counts { previous: 0, current: 1, common: 0, added: 1, removed: 0, changed: 0 }
        );
        assert_eq!(report.milestones, report.items);
        assert_eq!(report.rows.len(), 2);
        assert_eq!(report.refusal_observations, vec![Refusal::EmptyPrevious]);
        assert!(!report.revision_rules_compatible);
        assert!(review_required(&report));
        assert!(!report.artifact_validated);
        assert!(workflow::validate_successor(&old, &new).is_err());
    }

    /// Every owner/date/outcome/status/rationale category is visible without copying private prose.
    #[test]
    fn mutable_fields_report_exact_digests_and_stable_identities() {
        let previous = declaration();
        let mut current = previous.clone();
        current["items"][0]["owners"][0]["rationale"] =
            json!("SENSITIVE revised owner rationale\n\t");
        current["items"][0]["target_date"] = json!("2026-03-01");
        current["items"][0]["description"] = json!("SENSITIVE revised intended outcome");
        current["items"][0]["state"] = json!("in-progress");
        current["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"start","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-04T00:00:00Z","from":"planned","to":"in-progress","rationale":"SENSITIVE underway rationale","closure":null}));
        current["items"][0]["milestones"][0]["outcome"] =
            json!("SENSITIVE revised milestone outcome");
        let report = compared(&previous, &current);
        assert_eq!(report.items.common, 1);
        assert_eq!(report.items.changed, 1);
        let item = &report.rows[0];
        assert_eq!(item.uuid, identity::item("plan", "work").to_string());
        for expected in [
            ChangedField::Owners,
            ChangedField::TargetDate,
            ChangedField::Outcome,
            ChangedField::Status,
            ChangedField::History,
            ChangedField::Rationale,
        ] {
            assert!(item.changes.contains(&expected));
        }
        assert_eq!(
            item.current.as_ref().unwrap().outcome_sha256,
            sha256_hex(b"SENSITIVE revised intended outcome")
        );
        assert_eq!(item.history_prefix_preserved, Some(true));
        assert!(report.revision_rules_compatible);
        assert!(review_required(&report));
        assert!(!report.artifact_validated);
        let bytes = super::super::workflow_model::json_line(&report).unwrap();
        assert!(!String::from_utf8(bytes).unwrap().contains("SENSITIVE"));
    }

    /// Same owner keys still report changed resolved role/party metadata without disclosing names.
    #[test]
    fn owner_declaration_changes_are_visible_without_prose_disclosure() {
        let previous = declaration();
        assert!(!review_required(&compared(&previous, &previous)));
        let mut current = previous.clone();
        current["parties"][0]["name"] = json!("SENSITIVE revised owner name");
        current["roles"][0]["title"] = json!("SENSITIVE revised responsible role");
        let report = compared(&previous, &current);
        let item = &report.rows[0];
        assert_eq!(item.changes, vec![ChangedField::Owners]);
        assert!(review_required(&report));
        assert_eq!(
            item.previous.as_ref().unwrap().owners[0].party_key,
            item.current.as_ref().unwrap().owners[0].party_key
        );
        assert_ne!(
            item.previous.as_ref().unwrap().owners[0].party_declaration_sha256,
            item.current.as_ref().unwrap().owners[0].party_declaration_sha256
        );
        assert_ne!(
            item.previous.as_ref().unwrap().owners[0].role_declaration_sha256,
            item.current.as_ref().unwrap().owners[0].role_declaration_sha256
        );
        let bytes = super::super::workflow_model::json_line(&report).unwrap();
        assert!(!String::from_utf8(bytes).unwrap().contains("SENSITIVE"));
    }

    /// Complete union partitions retain removed identities even though existing artifact guards refuse them.
    #[test]
    fn additions_removals_and_empty_current_remain_complete_refused_observations() {
        let mut previous = declaration();
        let mut extra = previous["items"][0].clone();
        extra["key"] = json!("retired");
        previous["items"].as_array_mut().unwrap().push(extra);
        let mut current = declaration();
        let mut extra = current["items"][0].clone();
        extra["key"] = json!("added");
        current["items"].as_array_mut().unwrap().push(extra);
        let report = compared(&previous, &current);
        assert_eq!(
            report.items,
            Counts { previous: 2, current: 2, common: 1, added: 1, removed: 1, changed: 0 }
        );
        assert_eq!(
            report.milestones,
            Counts { previous: 2, current: 2, common: 1, added: 1, removed: 1, changed: 0 }
        );
        assert!(!report.revision_rules_compatible);
        assert!(report.refusal_observations.contains(&Refusal::RemovedIdentity));
        current["items"] = json!([]);
        current["roles"] = json!([]);
        current["parties"] = json!([]);
        let empty = compared(&previous, &current);
        assert_eq!(empty.items.removed, 2);
        assert_eq!(empty.milestones.removed, 2);
        assert!(review_required(&empty));
        assert!(empty.refusal_observations.contains(&Refusal::EmptyCurrent));
        assert!(workflow::parse(&serde_json::to_vec(&current).unwrap()).is_err());
    }

    /// Prior event rewrites remain a reported refusal and never acquire artifact admission.
    #[test]
    fn history_rationale_rewrite_is_not_hidden_or_admitted() {
        let previous = declaration();
        let mut current = previous.clone();
        current["items"][0]["history"][0]["rationale"] = json!("Different original rationale");
        let report = compared(&previous, &current);
        assert_eq!(report.rows[0].history_prefix_preserved, Some(false));
        assert!(report.rows[0].changes.contains(&ChangedField::History));
        assert!(report.rows[0].changes.contains(&ChangedField::Rationale));
        assert!(report.refusal_observations.contains(&Refusal::RewrittenHistory));
        assert!(!report.revision_rules_compatible);
        assert!(
            workflow::validate_successor(
                &parse_declaration(&serde_json::to_vec(&previous).unwrap()).unwrap(),
                &parse_declaration(&serde_json::to_vec(&current).unwrap()).unwrap()
            )
            .is_err()
        );
    }

    /// The whole source tuple is checked and prior not-current is not mislabeled historical invalidity.
    #[test]
    fn source_membership_distinguishes_current_stale_and_prior_not_current() {
        let previous = declaration();
        let mut current = previous.clone();
        current["items"][0]["source_refs"][0]["expected_sha256"] = json!("f".repeat(64));
        let stale = compared(&previous, &current);
        assert_eq!(stale.current_refs_not_current, 1);
        assert_eq!(stale.previous_refs_not_current, 0);
        assert!(stale.refusal_observations.contains(&Refusal::CurrentReferenceStale));
        assert!(!stale.revision_rules_compatible);
        let a = serde_json::to_vec(&previous).unwrap();
        let b = serde_json::to_vec(&current).unwrap();
        let old = parse_declaration(&a).unwrap();
        let new = parse_declaration(&b).unwrap();
        let updated = compare(
            &old,
            &new,
            &inventory(&new),
            &sha256_hex(&a),
            &sha256_hex(&b),
            "2026-02-06",
            &[],
        )
        .unwrap();
        assert_eq!(updated.current_refs_not_current, 0);
        assert_eq!(updated.previous_refs_not_current, 1);
        assert_eq!(updated.previous_validation, "bounded-structural-history-only");
        assert!(updated.rows[0].changes.contains(&ChangedField::Sources));
        for (field, value) in [
            ("uuid", json!("99999999-9999-4999-8999-999999999999")),
            ("result_uuid", json!("88888888-8888-4888-8888-888888888888")),
        ] {
            let mut changed = previous.clone();
            changed["items"][0]["source_refs"][0][field] = value;
            if field == "result_uuid" {
                assert!(parse_declaration(&serde_json::to_vec(&changed).unwrap()).is_err());
            } else {
                assert_eq!(compared(&previous, &changed).current_refs_not_current, 1);
            }
        }
    }

    /// Explicit cancelled-item reopening is reported refused; overlapping source refs do not imply a link.
    #[test]
    fn reopening_requires_exact_prior_bytes_and_new_identity_without_build_authority() {
        let mut previous = declaration();
        previous["items"][0]["state"] = json!("cancelled");
        previous["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"cancel","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-04T00:00:00Z","from":"planned","to":"cancelled","rationale":"Explicit plan cancellation","closure":null}));
        let mut current = previous.clone();
        let mut new = declaration()["items"][0].clone();
        new["key"] = json!("new-work");
        current["items"].as_array_mut().unwrap().push(new);
        let a = serde_json::to_vec(&previous).unwrap();
        let b = serde_json::to_vec(&current).unwrap();
        let old = parse_declaration(&a).unwrap();
        let new = parse_declaration(&b).unwrap();
        let sha = sha256_hex(&a);
        let unlinked =
            compare(&old, &new, &inventory(&old), &sha, &sha256_hex(&b), "2026-02-06", &[])
                .unwrap();
        assert!(unlinked.reopen_declarations.is_empty());
        let relation = ReopenDeclaration {
            previous_item_key: "work".into(),
            current_item_key: "new-work".into(),
            previous_manifest_sha256: sha.clone(),
        };
        let linked = compare(
            &old,
            &new,
            &inventory(&old),
            &sha,
            &sha256_hex(&b),
            "2026-02-06",
            std::slice::from_ref(&relation),
        )
        .unwrap();
        assert_eq!(linked.reopen_declarations.len(), 1);
        assert!(!linked.reopen_declarations[0].accepted_for_build);
        assert!(!linked.revision_rules_compatible);
        assert!(linked.refusal_observations.contains(&Refusal::ReopeningUnadmitted));
        assert!(
            reopen_rows(&old, &new, "wrong-raw-digest", std::slice::from_ref(&relation)).is_err()
        );
        assert!(
            reopen_rows(
                &old,
                &new,
                &sha,
                &[
                    relation,
                    ReopenDeclaration {
                        previous_item_key: "work".into(),
                        current_item_key: "new-work".into(),
                        previous_manifest_sha256: sha.clone()
                    }
                ]
            )
            .is_err()
        );
    }

    /// Terminal edits and backward modification instants are reported using existing core refusal rules.
    #[test]
    fn terminal_mutation_and_time_regression_remain_refused() {
        let mut previous = declaration();
        previous["items"][0]["state"] = json!("cancelled");
        previous["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"cancel","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-04T00:00:00Z","from":"planned","to":"cancelled","rationale":"Explicit cancellation","closure":null}));
        let mut current = previous.clone();
        current["items"][0]["milestones"][0]["outcome"] =
            json!("Changed terminal item's nested outcome");
        current["document"]["last_modified"] = json!("2026-02-19T00:00:00Z");
        let report = compared(&previous, &current);
        assert!(!report.revision_rules_compatible);
        assert!(report.refusal_observations.contains(&Refusal::TerminalMutation));
        assert!(report.refusal_observations.contains(&Refusal::ModificationTimeRegressed));
    }

    /// Closed duplicate-safe input and complete serialization caps cannot return prefix success.
    #[test]
    fn malformed_relations_input_and_whole_output_fail_closed() {
        assert!(
            parse_declaration(
                br#"{"schema_version":"forge.poam/1","schema_version":"forge.poam/1"}"#
            )
            .is_err()
        );
        assert!(parse_reopens(Some(br#"[{"previous_item_key":"x","current_item_key":"y","previous_manifest_sha256":"a","private":"z"}]"#)).is_err());
        assert!(parse_reopens(Some(b"{}")).is_err());
        assert!(
            bounded_value(&vec![b' '; usize::try_from(manifest::MAX_MANIFEST_BYTES).unwrap() + 1])
                .is_err()
        );
        assert!(
            super::super::workflow_model::json_line(&"x".repeat(workflow::MAX_OUTPUT_BYTES))
                .is_err()
        );
    }
}

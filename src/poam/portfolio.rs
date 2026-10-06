//! Explicit, bounded S1 portfolio over the public nonterminal five-source workflow.
//!
//! Every plan has an explicit authoring companion. An optional native file must be
//! in that companion's directory and equal its newly generated complete decoded
//! projection. This is not an arbitrary OSCAL importer. Hashes, schedule assertions
//! and schema consistency confer no actor authority, remediation eligibility,
//! assessment freshness or independent consumer approval. No output is written here.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::manifest::{self, ArtifactManifest, SourceKind};
use super::workflow::{self, Event, Owner, State, WorkflowManifest};
use crate::ForgeError;
use crate::json_strict::{self, Limits};

/// New detached report identity; it does not replace the workflow or M13 formats.
pub const SCHEMA_VERSION: &str = "forge.poam-portfolio/1";
/// Complete explicitly supplied plan ceiling; no filesystem discovery is performed.
pub const MAX_PLANS: usize = 32;
/// Aggregate original authoring bytes retained before source preparation.
pub const MAX_AUTHORING_BYTES: usize = 16 * 1024 * 1024;
/// Aggregate supplied native file bytes, not a per-plan multiplying allowance.
pub const MAX_NATIVE_BYTES: usize = 10 * 1024 * 1024;
/// Aggregate five-source occurrences retained as exact originals, charged per plan.
pub const MAX_SOURCE_BYTES: usize = 100 * 1024 * 1024;
/// Complete selected work ceiling across all admitted plans.
pub const MAX_ITEMS: usize = 10_000;
/// Complete milestone ceiling across all admitted plans.
pub const MAX_MILESTONES: usize = 10_000;
/// Complete history ceiling across all item and milestone records.
pub const MAX_EVENTS: usize = 100_000;
/// Independent complete bounds on source selections, owners and dependency edges.
pub const MAX_RELATIONS: usize = 100_000;
/// Entire JSON or HTML result ceiling, including its final newline.
pub const MAX_OUTPUT_BYTES: usize = 10 * 1024 * 1024;
/// Fixed report qualification; integrity rechecks are separately required at publication.
pub(super) const BOUNDARY: &str = "Author/native assertions only. Current five-source consistency and generated native schema are checked during preparation; actor authority, evidence freshness, remediation effectiveness and eligibility are unverified. Counts sum per-plan occurrences, not a unique cross-plan source union.";

/// One caller-selected authoring file and optional native file in its same directory.
#[derive(Debug, Clone)]
pub struct Input {
    /// Explicit local authoring path; its parent is resolved once and captured.
    pub manifest: PathBuf,
    /// A single normal UTF8 `.json` filename relative to the authoring directory.
    pub native_artifact: Option<PathBuf>,
}

/// Complete internally prepared portfolio; fields cannot be constructed by external callers.
#[derive(Debug, Serialize)]
pub struct PortfolioReport {
    /// Exact detached S1 report version.
    pub(super) schema_version: &'static str,
    /// Full supplied-native portfolio or explicitly qualified authoring preview.
    pub(super) validation_scope: &'static str,
    /// Caller-supplied canonical full date; no wall-clock default.
    pub(super) as_of: String,
    /// Caller-supplied inclusive due-soon interval, bounded to 0–365 days.
    pub(super) due_soon_days: u16,
    /// Complete summed per-plan denominators and schedule classifications.
    pub(super) counts: Counts,
    /// Every explicit plan, sorted by its exact immutable key.
    pub(super) plans: Vec<PlanRow>,
    /// Fixed assertion boundary, not a source freshness receipt.
    pub(super) boundary: &'static str,
}

/// Complete occurrence counts; shared sources are intentionally counted for each plan.
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub(super) struct Counts {
    /// Number of explicitly admitted plans.
    pub plans: usize,
    /// Actual explicitly supplied native files matched against full fresh projections.
    pub supplied_native_artifacts: usize,
    /// Authoring-only plans with generated virtual projections; never S1 supplied-artifact credit.
    pub authoring_only_plans: usize,
    /// Exactly five original file occurrences per plan.
    pub source_files: usize,
    /// Actual original source bytes summed per plan, including shared occurrences.
    pub source_bytes: usize,
    /// All finding/risk inventory objects per selected result before work selection.
    pub source_objects: usize,
    /// All selected items, including cancelled and future-first-assertion work.
    pub items: usize,
    /// All milestones under those items, without truncation.
    pub milestones: usize,
    /// Every item plus milestone schedule row.
    pub records: usize,
    /// Every attributed history event, including events after as-of.
    pub history_events: usize,
    /// Every exact author-selected finding/risk tuple, not distinct source objects.
    pub source_selections: usize,
    /// Every owner declaration across work and milestone records.
    pub owner_declarations: usize,
    /// Every declared milestone dependency edge.
    pub dependency_edges: usize,
    /// Open rows with target strictly before as-of.
    pub overdue: usize,
    /// Open rows with target in the inclusive due-soon range.
    pub due_soon: usize,
    /// Rows asserted blocked at as-of.
    pub blocked: usize,
    /// Rows with no history assertion on or before the as-of UTC date.
    pub before_first_assertion: usize,
    /// Rows asserted cancelled at as-of; no source conclusion is changed.
    pub cancelled: usize,
}

/// One fully admitted plan, without assessment prose, names or absolute paths.
#[derive(Debug, Serialize)]
pub(super) struct PlanRow {
    /// Exact author-supplied immutable plan key.
    pub key: String,
    /// Existing plan UUID-v5, never inferred from an arbitrary native artifact.
    pub uuid: String,
    /// SHA256 of the complete original raw authoring file.
    pub authoring_sha256: String,
    /// SHA256 of the freshly generated pretty native bytes, not the supplied file.
    pub generated_native_sha256: String,
    /// Optional exact supplied file digest; decoded comparison allows outer formatting differences.
    pub supplied_native_sha256: Option<String>,
    /// True only when an explicit supplied native file matched the complete generated value.
    pub supplied_native_matched: bool,
    /// Selected result UUID from the admitted source declaration.
    pub result_uuid: String,
    /// Selected result stable key; absence of work never implies no-action eligibility.
    pub result_key: String,
    /// Complete selected-result finding/risk inventory denominator.
    pub source_objects: usize,
    /// All five source receipts, with generated href spellings preserved as display text.
    pub sources: Vec<SourceFileRow>,
    /// All item and milestone records, in the current complete schedule order.
    pub records: Vec<RecordRow>,
    /// All explicit events sorted by actual RFC3339 instant, with deterministic ties.
    pub timeline: Vec<TimelineRow>,
    /// All exact selected source tuples with their existing native link spelling.
    pub trace: Vec<TraceRow>,
}

/// One actual captured file occurrence; a digest is integrity metadata, not freshness approval.
#[derive(Debug, Serialize)]
pub(super) struct SourceFileRow {
    /// Fixed supported source label.
    pub kind: &'static str,
    /// Generated native receipt href, kept unchanged and never activated by HTML.
    pub native_href: String,
    /// Exact whole-file SHA256 matched against the manifest pin.
    pub sha256: String,
    /// Complete captured raw byte length.
    pub bytes: usize,
}

/// One complete schedule record with minimized authored accountability declarations.
#[derive(Debug, Serialize)]
pub(super) struct RecordRow {
    /// Existing key-derived item or milestone UUID-v5.
    pub uuid: String,
    /// Exact immutable item key.
    pub item_key: String,
    /// Optional exact milestone key; null denotes an item.
    pub milestone_key: Option<String>,
    /// Canonical explicit target full date.
    pub target_date: String,
    /// Last assertion on or before as-of; null is kept in the denominator.
    pub state_at_as_of: Option<State>,
    /// Current declared final history state, which may be after as-of.
    pub declared_state: State,
    /// Explicit declared roles/party keys, without names or plaintext rationale.
    pub owners: Vec<OwnerRow>,
    /// Every exact earlier milestone key; empty for an item row.
    pub depends_on: Vec<String>,
    /// Original complete schedule strictly-late flag.
    pub overdue: bool,
    /// Original complete schedule inclusive near-target flag.
    pub due_soon: bool,
}

/// Minimized author-declared responsibility; no identity or authority authentication.
#[derive(Debug, Serialize)]
pub(super) struct OwnerRow {
    /// Authored role identifier.
    pub role_id: String,
    /// Authored stable party key, still potentially sensitive identity metadata.
    pub party_key: String,
    /// Digest of the authored rationale string, not verified evidence content.
    pub declared_rationale_sha256: String,
}

/// One author event; all events remain visible even when outside the requested as-of date.
#[derive(Debug, Serialize)]
pub(super) struct TimelineRow {
    /// Parent item key.
    pub item_key: String,
    /// Optional parent milestone key.
    pub milestone_key: Option<String>,
    /// Immutable event key within that record.
    pub event_key: String,
    /// Original RFC3339 timestamp spelling supplied by the author.
    pub at: String,
    /// Prior declared state; null only for the initial planned assertion.
    pub from: Option<State>,
    /// Next declared assertion state.
    pub to: State,
    /// Explicit actor role identifier, not an authorization decision.
    pub actor_role_id: String,
    /// Explicit actor stable party key, without party name.
    pub actor_party_key: String,
    /// Digest of authored rationale text, not proof of decision authority.
    pub declared_rationale_sha256: String,
    /// True when the event's UTC date is after the explicit report date.
    pub after_as_of: bool,
    /// Parsed instant used solely for correct offset/fraction-aware ordering.
    #[serde(skip)]
    sort_at: DateTime<FixedOffset>,
}

/// One validated source selection, with file and canonical object digests kept distinct.
#[derive(Debug, Serialize)]
pub(super) struct TraceRow {
    /// Immutable selected-work key.
    pub item_key: String,
    /// Existing work UUID-v5.
    pub item_uuid: String,
    /// Closed supported source kind: finding or risk.
    pub kind: SourceKind,
    /// Original exact source stable key.
    pub source_key: String,
    /// Original source UUID, preserved rather than regenerated.
    pub source_uuid: String,
    /// Original explicitly selected result UUID.
    pub result_uuid: String,
    /// Computed canonical source object digest matched by full workflow selection.
    pub canonical_object_sha256: String,
    /// Exact whole-file AR digest, distinct from the canonical object digest.
    pub assessment_results_file_sha256: String,
    /// Exact generated percent-encoded AR-file reference plus original UUID fragment.
    pub native_href: String,
}

/// Prepared bytes plus bounded exact input originals; no method writes to an output path.
#[derive(Debug)]
pub struct PreparedPortfolio {
    /// Internally admitted typed report for the static HTML renderer.
    report: PortfolioReport,
    /// Entire bounded pretty JSON with a single final newline.
    json: Vec<u8>,
    /// Exact originals charged to aggregate bounds; no accumulated `PreparedWorkflow` objects.
    inputs: Vec<HeldInput>,
}

impl PreparedPortfolio {
    /// Borrow the complete typed report without transferring publication or freshness authority.
    #[must_use]
    pub fn report(&self) -> &PortfolioReport {
        &self.report
    }

    /// Borrow the complete JSON bytes; caller retains responsibility for safe output placement.
    #[must_use]
    pub fn json(&self) -> &[u8] {
        &self.json
    }

    /// Distinguish fully supplied artifact input from mixed/all-authoring preview.
    #[must_use]
    pub fn validation_scope(&self) -> &'static str {
        self.report.validation_scope
    }

    /// Count actual supplied native originals, not virtual generated projections.
    #[must_use]
    pub fn supplied_native_artifacts(&self) -> usize {
        self.report.counts.supplied_native_artifacts
    }

    /// Enumerate every original input path for Root-owned output-alias preflight.
    ///
    /// Paths are sensitive internal placement metadata and are absent from report JSON/HTML.
    pub(crate) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.inputs.iter().flat_map(|input| {
            std::iter::once(&input.authoring)
                .chain(input.native.iter())
                .chain(input.sources.iter().map(|source| &source.original))
                .map(move |original| input.root.join(&original.relative))
        })
    }

    /// Select valid-review exit1 when any complete row is overdue, due soon or blocked.
    #[must_use]
    pub fn review_required(&self) -> bool {
        self.report.counts.overdue > 0
            || self.report.counts.due_soon > 0
            || self.report.counts.blocked > 0
    }

    /// Freshly recheck all exact original identities/bytes and full five-source admission.
    ///
    /// Plans are reparsed/reprepared sequentially, then their temporary captures are dropped.
    /// This is a check at revalidation time, not an atomic multi-file publication guarantee
    /// or proof against identity reuse after an unseen delete/restore sequence. The caller
    /// must run it immediately before each output publication and protect output aliases.
    /// # Errors
    /// Refuses changed, replaced, unsafe or newly inadmissible original input occurrences.
    pub fn verify_inputs(&self) -> Result<(), ForgeError> {
        for input in &self.inputs {
            verify_held(input)?;
            let prepared = workflow::prepare(
                &input.path(),
                &input.authoring.bytes,
                &self.report.as_of,
                self.report.due_soon_days,
                None,
            )?;
            if let Some(native) = &input.native {
                match_native(prepared.artifact(), &native.bytes)?;
            }
            prepared.verify_inputs()?;
            verify_held(input)?;
        }
        Ok(())
    }
}

/// Prepare the S1 supplied-artifact profile; every plan requires an explicit native/authoring pair.
///
/// Root's shipping portfolio CLI must consume this entry point, not the optional
/// authoring preview. Native admission is still restricted to complete equality with
/// the current public workflow projection; no arbitrary extensions are imported.
/// # Errors
/// Refuses any missing native companion before input reads, plus all preparation failures.
pub fn prepare_native_portfolio(
    inputs: &[Input],
    as_of: &str,
    due_soon_days: u16,
) -> Result<PreparedPortfolio, ForgeError> {
    if inputs.iter().any(|input| input.native_artifact.is_none()) {
        return Err(error(
            "supplied-artifact portfolio requires an explicit native file for every authoring companion",
        ));
    }
    prepare(inputs, as_of, due_soon_days)
}

/// Resolve only explicit inputs, charge whole-portfolio bounds, and prepare every plan.
///
/// Optional authoring-only inputs are a preview, not S1 supplied-artifact support.
/// Exactly one current public `WorkflowManifest` and five-source workflow is admitted per
/// plan. M13's structural prior-only comparison is not used. Completion/risk-acceptance
/// history remains refused by `workflow::parse/prepare`; cancelled records remain counted.
/// Retained original source occurrences total at most 100 MiB. This is not a peak heap
/// bound: existing JSON/source validation has additional bounded transient allocations.
/// # Errors
/// Refuses empty/unbounded input, aliases, duplicate stable plans, unsupported native
/// extensions, invalid schedules, stale sources or complete-output bounds; never returns a prefix.
pub fn prepare(
    inputs: &[Input],
    as_of: &str,
    due_soon_days: u16,
) -> Result<PreparedPortfolio, ForgeError> {
    let date = full_date(as_of)?;
    if inputs.is_empty() || inputs.len() > MAX_PLANS {
        return Err(error("portfolio requires 1–32 explicit plans"));
    }
    if due_soon_days > workflow::MAX_DUE_SOON_DAYS {
        return Err(error("portfolio due-soon interval exceeds 365 days"));
    }
    let mut budget = Budget::default();
    let mut held = Vec::with_capacity(inputs.len());
    let mut input_paths = BTreeSet::new();
    let mut input_identities = BTreeSet::new();
    let mut plan_ids = BTreeSet::new();
    // Complete declaration/native admission and counts occur before source preparation.
    for input in inputs {
        let (root, relative) = authoring_location(&input.manifest)?;
        let authoring = capture(
            &root,
            &relative,
            (MAX_AUTHORING_BYTES - budget.authoring_bytes).min(
                usize::try_from(manifest::MAX_MANIFEST_BYTES)
                    .map_err(|_| error("manifest bound is unsupported on this platform"))?,
            ),
        )?;
        charge(&mut budget.authoring_bytes, authoring.bytes.len(), MAX_AUTHORING_BYTES)?;
        reserve_input(&root, &authoring, &mut input_paths, &mut input_identities)?;
        let declaration = workflow::parse(&authoring.bytes)?;
        if !plan_ids.insert(super::identity::document(&declaration.document.key).to_string()) {
            return Err(error("duplicate stable plan identity"));
        }
        charge_declaration(&mut budget, &declaration)?;
        let native = match &input.native_artifact {
            Some(relative) => {
                native_filename(relative)?;
                let original = capture(&root, relative, MAX_NATIVE_BYTES - budget.native_bytes)?;
                charge(&mut budget.native_bytes, original.bytes.len(), MAX_NATIVE_BYTES)?;
                reserve_input(&root, &original, &mut input_paths, &mut input_identities)?;
                Some(original)
            }
            None => None,
        };
        held.push(HeldInput { root, authoring, native, declaration, sources: Vec::new() });
    }
    // Exact raw source originals are bounded in aggregate, charged per plan even if shared.
    for input in &mut held {
        for (kind, pin) in source_pins(&input.declaration) {
            let original = capture(
                &input.root,
                &pin.artifact,
                (MAX_SOURCE_BYTES - budget.source_bytes).min(
                    usize::try_from(crate::io::MAX_FILE_SIZE)
                        .map_err(|_| error("source bound is unsupported on this platform"))?,
                ),
            )?;
            charge(&mut budget.source_bytes, original.bytes.len(), MAX_SOURCE_BYTES)?;
            if input_paths.contains(&input.root.join(&original.relative))
                || input_identities.contains(&original.identity)
            {
                return Err(error("authoring or native input aliases a source input"));
            }
            if crate::hashing::sha256_hex(&original.bytes) != pin.expected_sha256 {
                return Err(error(
                    "portfolio source file does not match its explicit whole-file pin",
                ));
            }
            input.sources.push(SourceOriginal { kind, original });
        }
    }
    held.sort_by(|a, b| a.declaration.document.key.cmp(&b.declaration.document.key));
    let mut plans = Vec::with_capacity(held.len());
    for input in &held {
        verify_held(input)?;
        let prepared =
            workflow::prepare(&input.path(), &input.authoring.bytes, as_of, due_soon_days, None)?;
        if let Some(native) = &input.native {
            match_native(prepared.artifact(), &native.bytes)?;
        }
        plans.push(project(input, prepared.artifact(), prepared.schedule(), date, due_soon_days)?);
        prepared.verify_inputs()?;
        verify_held(input)?;
        // The one plan's PreparedWorkflow and its source captures drop here.
    }
    let counts = count_plans(&plans)?;
    let report = PortfolioReport {
        schema_version: SCHEMA_VERSION,
        validation_scope: validation_scope(&counts),
        as_of: as_of.to_string(),
        due_soon_days,
        counts,
        plans,
        boundary: BOUNDARY,
    };
    validate_report(&report)?;
    let json = super::workflow_model::json_line(&report)?;
    let prepared = PreparedPortfolio { report, json, inputs: held };
    prepared.verify_inputs()?;
    Ok(prepared)
}

/// Whole declaration counts charged before any five-source workflow allocations.
#[derive(Debug, Default)]
struct Budget {
    /// Original authoring raw bytes.
    authoring_bytes: usize,
    /// Optional native raw bytes.
    native_bytes: usize,
    /// Original five-source raw occurrences.
    source_bytes: usize,
    /// Complete item denominator.
    items: usize,
    /// Complete milestone denominator.
    milestones: usize,
    /// Complete event denominator.
    events: usize,
    /// Complete exact selection occurrences.
    selections: usize,
    /// Complete owner declarations.
    owners: usize,
    /// Complete explicit DAG edges.
    edges: usize,
}

/// Bounded raw original paired with the actual confined reader identity.
#[derive(Debug)]
struct Original {
    /// Normalized confined relative path.
    relative: PathBuf,
    /// Exact original bytes; digests do not substitute for fresh byte comparison.
    bytes: Vec<u8>,
    /// Captured reader volume/file identity tuple, not an eternal generation identifier.
    identity: (u64, u64),
}

/// One explicitly declared source occurrence, including repeats in other plans.
#[derive(Debug)]
struct SourceOriginal {
    /// Fixed supported five-source label.
    kind: &'static str,
    /// Exact bounded original file.
    original: Original,
}

/// All explicit inputs for one admitted current public workflow.
#[derive(Debug)]
struct HeldInput {
    /// Once-canonicalized local bundle directory.
    root: PathBuf,
    /// Exact authoring file original.
    authoring: Original,
    /// Optional exact native original in the same directory.
    native: Option<Original>,
    /// Full public declaration, never a structural M13 prior profile.
    declaration: WorkflowManifest,
    /// All five source originals charged to the one aggregate ceiling.
    sources: Vec<SourceOriginal>,
}

impl HeldInput {
    /// Supply the original bundle base to the existing workflow source loader.
    fn path(&self) -> PathBuf {
        self.root.join(&self.authoring.relative)
    }
}

/// Normalize only the explicit authoring parent, retaining a single confined filename.
fn authoring_location(path: &Path) -> Result<(PathBuf, PathBuf), ForgeError> {
    let parent =
        path.parent().filter(|value| !value.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let root =
        parent.canonicalize().map_err(|_| error("cannot resolve explicit authoring directory"))?;
    let filename =
        path.file_name().ok_or_else(|| error("explicit authoring filename is required"))?;
    Ok((root, PathBuf::from(filename)))
}

/// Restrict native admission to a single UTF8 JSON filename beside its authoring companion.
fn native_filename(path: &Path) -> Result<(), ForgeError> {
    let text = path.to_str().ok_or_else(|| error("native filename must be UTF8"))?;
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || path.extension().and_then(|value| value.to_str()) != Some("json")
        || (text.contains('/') || text.contains('\\'))
        || text.chars().any(char::is_control)
    {
        return Err(error(
            "native artifact must be one normal JSON filename in the authoring directory",
        ));
    }
    Ok(())
}

/// Capture through the existing confined/single-link reader before any retained growth.
fn capture(root: &Path, relative: &Path, limit: usize) -> Result<Original, ForgeError> {
    let (bytes, identity) = crate::linkage::read_confined_local_file(root, relative, limit as u64)
        .map_err(|_| {
            error("explicit portfolio input is unsafe, unavailable or exceeds its remaining bound")
        })?;
    Ok(Original { relative: relative.to_path_buf(), bytes, identity })
}

/// Reject declaration/native aliases by normalized resolved path and actual captured identity.
fn reserve_input(
    root: &Path,
    original: &Original,
    paths: &mut BTreeSet<PathBuf>,
    identities: &mut BTreeSet<(u64, u64)>,
) -> Result<(), ForgeError> {
    if !paths.insert(root.join(&original.relative)) || !identities.insert(original.identity) {
        return Err(error("duplicate or aliased explicit authoring/native input"));
    }
    Ok(())
}

/// Refuse any identity or literal byte drift; a hash never transfers input freshness.
fn verify_original(root: &Path, original: &Original) -> Result<(), ForgeError> {
    let (bytes, identity) = crate::linkage::read_confined_local_file(
        root,
        &original.relative,
        original.bytes.len() as u64,
    )
    .map_err(|_| error("original portfolio input is no longer safely readable"))?;
    if identity != original.identity || bytes != original.bytes {
        return Err(error("original portfolio input identity or bytes changed"));
    }
    Ok(())
}

/// Recheck every declaration, optional native file and all five original source occurrences.
fn verify_held(input: &HeldInput) -> Result<(), ForgeError> {
    verify_original(&input.root, &input.authoring)?;
    if let Some(native) = &input.native {
        verify_original(&input.root, native)?;
    }
    for source in &input.sources {
        verify_original(&input.root, &source.original)?;
    }
    Ok(())
}

/// Enumerate the unchanged five declarations, without inference or directory traversal discovery.
fn source_pins(value: &WorkflowManifest) -> [(&'static str, &ArtifactManifest); 5] {
    [
        ("assessment-results", &value.source.assessment_results),
        ("assessment-plan", &value.source.context.assessment_plan),
        ("system-security-plan", &value.source.context.ssp),
        ("profile", &value.source.context.profile),
        ("catalog", &value.source.context.catalog),
    ]
}

/// Charge an entire complete collection before retaining its additional records.
fn charge(total: &mut usize, added: usize, limit: usize) -> Result<(), ForgeError> {
    let next = total.checked_add(added).filter(|next| *next <= limit).ok_or_else(|| {
        error("complete portfolio exceeds an aggregate input or collection bound")
    })?;
    *total = next;
    Ok(())
}

/// Charge all explicit item/milestone/event/reference/owner/edge occurrences before loading sources.
fn charge_declaration(budget: &mut Budget, value: &WorkflowManifest) -> Result<(), ForgeError> {
    charge(&mut budget.items, value.items.len(), MAX_ITEMS)?;
    for item in &value.items {
        charge(&mut budget.selections, item.source_refs.len(), MAX_RELATIONS)?;
        charge(&mut budget.owners, item.owners.len(), MAX_RELATIONS)?;
        charge(&mut budget.events, item.history.len(), MAX_EVENTS)?;
        charge(&mut budget.milestones, item.milestones.len(), MAX_MILESTONES)?;
        for step in &item.milestones {
            charge(&mut budget.owners, step.owners.len(), MAX_RELATIONS)?;
            charge(&mut budget.events, step.history.len(), MAX_EVENTS)?;
            charge(&mut budget.edges, step.depends_on.len(), MAX_RELATIONS)?;
        }
    }
    Ok(())
}

/// Decode bounded duplicate-free native JSON; strings may contain bounded compact declarations.
fn native_value(bytes: &[u8]) -> Result<Value, ForgeError> {
    if bytes.len() > MAX_NATIVE_BYTES {
        return Err(error("native portfolio input exceeds ten MiB"));
    }
    json_strict::parse_value(
        bytes,
        "POA&M portfolio native input",
        Limits { max_depth: 64, max_string_bytes: MAX_NATIVE_BYTES },
    )
    .map_err(|_| error("native input must be complete bounded duplicate-free JSON"))
}

/// Accept only complete decoded equality to the fresh supported native projection.
fn match_native(expected: &[u8], supplied: &[u8]) -> Result<(), ForgeError> {
    if native_value(expected)? != native_value(supplied)? {
        return Err(error(
            "supplied native artifact differs from its full supported authoring projection",
        ));
    }
    Ok(())
}

/// Closed decoder for the freshly generated schedule, never an arbitrary report import.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Schedule {
    /// Existing complete schedule format identity.
    schema_version: String,
    /// Actual explicit canonical report date.
    as_of: String,
    /// Actual caller interval.
    due_soon_days: u16,
    /// All source objects before work selection.
    source_objects: usize,
    /// All selected items.
    items: usize,
    /// All milestones.
    milestones: usize,
    /// Complete schedule records.
    rows: Vec<Scheduled>,
    /// Full overdue count.
    overdue: usize,
    /// Full inclusive near-target count.
    due_soon: usize,
    /// Full blocked count.
    blocked: usize,
    /// Original fixed schedule qualification.
    boundary: String,
}

/// Exactly one generated schedule record.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scheduled {
    /// Existing item/milestone UUID-v5.
    uuid: String,
    /// Exact authored item key.
    item_key: String,
    /// Optional exact milestone key.
    milestone_key: Option<String>,
    /// Explicit canonical target.
    target_date: String,
    /// Original as-of assertion, including null.
    state: Option<State>,
    /// Original overdue flag.
    overdue: bool,
    /// Original inclusive due-soon flag.
    due_soon: bool,
}

/// Project complete generated rows, source links and authored histories without source prose.
#[allow(
    clippy::too_many_lines,
    reason = "one complete admitted plan projection reconciles schedule, timeline and exact native trace before returning"
)]
fn project(
    input: &HeldInput,
    artifact: &[u8],
    schedule: &[u8],
    date: NaiveDate,
    due_soon_days: u16,
) -> Result<PlanRow, ForgeError> {
    let declared = &input.declaration;
    let schedule: Schedule = serde_json::from_slice(schedule)
        .map_err(|_| error("generated complete schedule shape is unavailable"))?;
    let milestone_count = declared.items.iter().map(|item| item.milestones.len()).sum::<usize>();
    if schedule.schema_version != "forge.poam-schedule/1"
        || schedule.as_of != date.to_string()
        || schedule.due_soon_days != due_soon_days
        || schedule.items != declared.items.len()
        || schedule.milestones != milestone_count
        || schedule.rows.len() != schedule.items + schedule.milestones
        || schedule.source_objects > 10_000
        || schedule.boundary.is_empty()
        || schedule.overdue != schedule.rows.iter().filter(|row| row.overdue).count()
        || schedule.due_soon != schedule.rows.iter().filter(|row| row.due_soon).count()
        || schedule.blocked
            != schedule.rows.iter().filter(|row| row.state == Some(State::Blocked)).count()
    {
        return Err(error("generated schedule does not preserve its complete denominators"));
    }
    let native = native_value(artifact)?;
    let root = native
        .get("plan-of-action-and-milestones")
        .ok_or_else(|| error("generated native plan root is unavailable"))?;
    let uuid = super::identity::document(&declared.document.key).to_string();
    if root.get("uuid").and_then(Value::as_str) != Some(uuid.as_str()) {
        return Err(error("generated native plan identity differs"));
    }
    let native_items = root
        .get("poam-items")
        .and_then(Value::as_array)
        .ok_or_else(|| error("generated native work records are unavailable"))?;
    let native_by_uuid: BTreeMap<_, _> = native_items
        .iter()
        .map(|item| {
            item.get("uuid")
                .and_then(Value::as_str)
                .map(|uuid| (uuid, item))
                .ok_or_else(|| error("generated native work identity is unavailable"))
        })
        .collect::<Result<_, _>>()?;
    if native_by_uuid.len() != declared.items.len() || native_items.len() != native_by_uuid.len() {
        return Err(error("generated native work denominator differs"));
    }
    let mut scheduled = BTreeMap::new();
    for row in schedule.rows {
        if scheduled.insert((row.item_key.clone(), row.milestone_key.clone()), row).is_some() {
            return Err(error("generated schedule repeats a work identity"));
        }
    }
    let mut records = Vec::with_capacity(schedule.items + schedule.milestones);
    let mut timeline = Vec::new();
    let mut trace = Vec::new();
    let mut items: Vec<_> = declared.items.iter().collect();
    items.sort_by(|a, b| a.key.cmp(&b.key));
    for item in items {
        let item_uuid = super::identity::item(&declared.document.key, &item.key).to_string();
        records.push(record(
            &mut scheduled,
            &item.key,
            None,
            &item_uuid,
            &item.target_date,
            item.state,
            &item.owners,
            &[],
        )?);
        append_events(&mut timeline, &item.key, None, &item.history, date)?;
        for step in &item.milestones {
            let step_uuid =
                super::identity::milestone(&declared.document.key, &item.key, &step.key)
                    .to_string();
            records.push(record(
                &mut scheduled,
                &item.key,
                Some(step.key.as_str()),
                &step_uuid,
                &step.target_date,
                step.state,
                &step.owners,
                &step.depends_on,
            )?);
            append_events(&mut timeline, &item.key, Some(step.key.as_str()), &step.history, date)?;
        }
        let links = native_by_uuid
            .get(item_uuid.as_str())
            .and_then(|value| value.get("links"))
            .and_then(Value::as_array)
            .ok_or_else(|| error("generated native source links are unavailable"))?;
        if links.len() != item.source_refs.len() {
            return Err(error("generated source link denominator differs"));
        }
        for (reference, link) in item.source_refs.iter().zip(links) {
            let href = link
                .get("href")
                .and_then(Value::as_str)
                .ok_or_else(|| error("generated native source href is unavailable"))?;
            if link.get("rel").and_then(Value::as_str) != Some("assessment-source")
                || !href.ends_with(&format!("#{}", reference.uuid))
            {
                return Err(error("generated native source reference differs"));
            }
            trace.push(TraceRow {
                item_key: item.key.clone(),
                item_uuid: item_uuid.clone(),
                kind: reference.kind,
                source_key: reference.key.clone(),
                source_uuid: reference.uuid.clone(),
                result_uuid: reference.result_uuid.clone(),
                canonical_object_sha256: reference.expected_sha256.clone(),
                assessment_results_file_sha256: declared
                    .source
                    .assessment_results
                    .expected_sha256
                    .clone(),
                native_href: href.to_string(),
            });
        }
    }
    if !scheduled.is_empty() {
        return Err(error("generated schedule contains an unconsumed identity"));
    }
    sort_timeline(&mut timeline);
    let resources = root
        .pointer("/back-matter/resources")
        .and_then(Value::as_array)
        .ok_or_else(|| error("generated five-source receipts are unavailable"))?;
    if resources.len() != 5 || input.sources.len() != 5 {
        return Err(error("five-source receipt denominator differs"));
    }
    let mut sources = Vec::with_capacity(5);
    for source in &input.sources {
        let resource = resources
            .iter()
            .find(|row| row.get("title").and_then(Value::as_str) == Some(source.kind))
            .ok_or_else(|| error("generated source receipt kind is unavailable"))?;
        let href = resource
            .pointer("/rlinks/0/href")
            .and_then(Value::as_str)
            .ok_or_else(|| error("generated source receipt href is unavailable"))?;
        sources.push(SourceFileRow {
            kind: source.kind,
            native_href: href.to_string(),
            sha256: crate::hashing::sha256_hex(&source.original.bytes),
            bytes: source.original.bytes.len(),
        });
    }
    Ok(PlanRow {
        key: declared.document.key.clone(),
        uuid,
        authoring_sha256: crate::hashing::sha256_hex(&input.authoring.bytes),
        generated_native_sha256: crate::hashing::sha256_hex(artifact),
        supplied_native_sha256: input
            .native
            .as_ref()
            .map(|value| crate::hashing::sha256_hex(&value.bytes)),
        supplied_native_matched: input.native.is_some(),
        result_uuid: declared.source.result.uuid.clone(),
        result_key: declared.source.result.key.clone(),
        source_objects: schedule.source_objects,
        sources,
        records,
        timeline,
        trace,
    })
}

/// Consume exactly one generated identity and preserve complete null/terminal scope.
#[allow(clippy::too_many_arguments)]
fn record(
    rows: &mut BTreeMap<(String, Option<String>), Scheduled>,
    item_key: &str,
    milestone_key: Option<&str>,
    uuid: &str,
    target: &str,
    declared_state: State,
    owners: &[Owner],
    depends_on: &[String],
) -> Result<RecordRow, ForgeError> {
    let row = rows
        .remove(&(item_key.to_string(), milestone_key.map(str::to_owned)))
        .ok_or_else(|| error("generated schedule omitted an authored work identity"))?;
    if row.uuid != uuid || row.target_date != target {
        return Err(error("generated schedule identity or target differs"));
    }
    Ok(RecordRow {
        uuid: row.uuid,
        item_key: row.item_key,
        milestone_key: row.milestone_key,
        target_date: row.target_date,
        state_at_as_of: row.state,
        declared_state,
        owners: owners
            .iter()
            .map(|value| OwnerRow {
                role_id: value.role_id.clone(),
                party_key: value.party_key.clone(),
                declared_rationale_sha256: crate::hashing::sha256_hex(value.rationale.as_bytes()),
            })
            .collect(),
        depends_on: depends_on.to_vec(),
        overdue: row.overdue,
        due_soon: row.due_soon,
    })
}

/// Preserve every event while classifying its UTC date against the explicit as-of input.
fn append_events(
    rows: &mut Vec<TimelineRow>,
    item: &str,
    milestone: Option<&str>,
    history: &[Event],
    date: NaiveDate,
) -> Result<(), ForgeError> {
    for event in history {
        let sort_at = DateTime::parse_from_rfc3339(&event.at)
            .map_err(|_| error("admitted event timestamp is invalid"))?;
        rows.push(TimelineRow {
            item_key: item.to_string(),
            milestone_key: milestone.map(str::to_owned),
            event_key: event.key.clone(),
            at: event.at.clone(),
            from: event.from,
            to: event.to,
            actor_role_id: event.actor.role_id.clone(),
            actor_party_key: event.actor.party_key.clone(),
            declared_rationale_sha256: crate::hashing::sha256_hex(event.rationale.as_bytes()),
            after_as_of: sort_at.with_timezone(&chrono::Utc).date_naive() > date,
            sort_at,
        });
    }
    Ok(())
}

/// Order original RFC3339 events by actual instant, with stable identity ties.
fn sort_timeline(rows: &mut [TimelineRow]) {
    rows.sort_by(|a, b| {
        a.sort_at
            .cmp(&b.sort_at)
            .then(a.item_key.cmp(&b.item_key))
            .then(a.milestone_key.cmp(&b.milestone_key))
            .then(a.event_key.cmp(&b.event_key))
    });
}

/// Sum all complete per-plan counts; no cross-plan source deduplication is inferred.
fn count_plans(plans: &[PlanRow]) -> Result<Counts, ForgeError> {
    let mut counts = Counts::default();
    charge(&mut counts.plans, plans.len(), MAX_PLANS)?;
    for plan in plans {
        if plan.supplied_native_matched {
            charge(&mut counts.supplied_native_artifacts, 1, MAX_PLANS)?;
        } else {
            charge(&mut counts.authoring_only_plans, 1, MAX_PLANS)?;
        }
        charge(&mut counts.source_files, plan.sources.len(), MAX_PLANS * 5)?;
        charge(&mut counts.source_objects, plan.source_objects, MAX_PLANS * 10_000)?;
        for source in &plan.sources {
            charge(&mut counts.source_bytes, source.bytes, MAX_SOURCE_BYTES)?;
        }
        charge(&mut counts.history_events, plan.timeline.len(), MAX_EVENTS)?;
        charge(&mut counts.source_selections, plan.trace.len(), MAX_RELATIONS)?;
        for row in &plan.records {
            charge(&mut counts.records, 1, MAX_ITEMS + MAX_MILESTONES)?;
            if row.milestone_key.is_some() {
                charge(&mut counts.milestones, 1, MAX_MILESTONES)?;
            } else {
                charge(&mut counts.items, 1, MAX_ITEMS)?;
            }
            charge(&mut counts.owner_declarations, row.owners.len(), MAX_RELATIONS)?;
            charge(&mut counts.dependency_edges, row.depends_on.len(), MAX_RELATIONS)?;
            counts.overdue += usize::from(row.overdue);
            counts.due_soon += usize::from(row.due_soon);
            counts.blocked += usize::from(row.state_at_as_of == Some(State::Blocked));
            counts.before_first_assertion += usize::from(row.state_at_as_of.is_none());
            counts.cancelled += usize::from(row.state_at_as_of == Some(State::Cancelled));
        }
    }
    Ok(counts)
}

/// Name the actual admitted input profile without granting virtual/native artifact credit.
fn validation_scope(counts: &Counts) -> &'static str {
    if counts.plans > 0 && counts.supplied_native_artifacts == counts.plans {
        "supplied-native-artifact-portfolio"
    } else {
        "authoring-preview-with-optional-native-pairs"
    }
}

/// Guard complete counts and the public nonterminal boundary before rendering.
pub(super) fn validate_report(report: &PortfolioReport) -> Result<(), ForgeError> {
    full_date(&report.as_of)?;
    if report.schema_version != SCHEMA_VERSION
        || report.boundary != BOUNDARY
        || report.validation_scope != validation_scope(&report.counts)
        || report.due_soon_days > workflow::MAX_DUE_SOON_DAYS
        || report.plans.is_empty()
        || report.counts != count_plans(&report.plans)?
    {
        return Err(error("portfolio report scope or complete denominators differ"));
    }
    let undisposed =
        |state| matches!(state, State::CompletedAsserted | State::AcceptedRiskAsserted);
    let mut plan_ids = BTreeSet::new();
    for plan in &report.plans {
        if !plan_ids.insert(&plan.uuid)
            || plan.sources.len() != 5
            || plan.records.is_empty()
            || plan.supplied_native_matched != plan.supplied_native_sha256.is_some()
            || plan.records.iter().any(|row| {
                undisposed(row.declared_state) || row.state_at_as_of.is_some_and(undisposed)
            })
            || plan
                .timeline
                .iter()
                .any(|event| undisposed(event.to) || event.from.is_some_and(undisposed))
        {
            return Err(error("portfolio report violates current public workflow admission"));
        }
    }
    Ok(())
}

/// Parse the caller date canonically without current time or locale defaults.
fn full_date(value: &str) -> Result<NaiveDate, ForgeError> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| error("portfolio as-of must be a valid Gregorian full date"))?;
    if value.len() != 10 || date.to_string() != value {
        return Err(error("portfolio as-of must be YYYY-MM-DD"));
    }
    Ok(date)
}

/// Return a fixed error without embedding private paths, authored prose or parser diagnostics.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.to_string())
}

/// Detached rendering/count fixture only; it establishes no native or source admission.
#[cfg(test)]
#[allow(
    clippy::too_many_lines,
    reason = "one detached complete report fixture retains all schedule and trace denominator dimensions"
)]
pub(super) fn test_report() -> PortfolioReport {
    let source_kinds =
        ["assessment-results", "assessment-plan", "system-security-plan", "profile", "catalog"];
    let owners = || {
        vec![OwnerRow {
            role_id: "owner".to_string(),
            party_key: "alice".to_string(),
            declared_rationale_sha256: crate::hashing::sha256_hex(b"SENSITIVE PLAINTEXT RATIONALE"),
        }]
    };
    let records = vec![
        RecordRow {
            uuid: "future-record-uuid".to_string(),
            item_key: "future-work".to_string(),
            milestone_key: None,
            target_date: "2026-03-01".to_string(),
            state_at_as_of: None,
            declared_state: State::Planned,
            owners: owners(),
            depends_on: vec![],
            overdue: false,
            due_soon: false,
        },
        RecordRow {
            uuid: "cancelled-record-uuid".to_string(),
            item_key: "cancelled-work".to_string(),
            milestone_key: None,
            target_date: "2026-02-01".to_string(),
            state_at_as_of: Some(State::Cancelled),
            declared_state: State::Cancelled,
            owners: owners(),
            depends_on: vec![],
            overdue: false,
            due_soon: false,
        },
        RecordRow {
            uuid: "blocked-record-uuid".to_string(),
            item_key: "blocked-work".to_string(),
            milestone_key: None,
            target_date: "2026-02-01".to_string(),
            state_at_as_of: Some(State::Blocked),
            declared_state: State::Blocked,
            owners: owners(),
            depends_on: vec![],
            overdue: false,
            due_soon: true,
        },
        RecordRow {
            uuid: "milestone-record-uuid".to_string(),
            item_key: "blocked-work".to_string(),
            milestone_key: Some("step".to_string()),
            target_date: "2026-01-20".to_string(),
            state_at_as_of: Some(State::Planned),
            declared_state: State::Planned,
            owners: owners(),
            depends_on: vec![],
            overdue: true,
            due_soon: false,
        },
    ];
    let sort_at = DateTime::parse_from_rfc3339("2026-03-01T01:00:00+01:00").unwrap();
    let timeline = vec![TimelineRow {
        item_key: "future-work".to_string(),
        milestone_key: None,
        event_key: "future-first-event".to_string(),
        at: "2026-03-01T01:00:00+01:00".to_string(),
        from: None,
        to: State::Planned,
        actor_role_id: "owner".to_string(),
        actor_party_key: "alice".to_string(),
        declared_rationale_sha256: crate::hashing::sha256_hex(b"SENSITIVE PLAINTEXT RATIONALE"),
        after_as_of: true,
        sort_at,
    }];
    let trace = vec![TraceRow {
        item_key: "blocked-work".to_string(),
        item_uuid: "blocked-record-uuid".to_string(),
        kind: SourceKind::Finding,
        source_key: "finding".to_string(),
        source_uuid: "original-source-uuid".to_string(),
        result_uuid: "original-result-uuid".to_string(),
        canonical_object_sha256: "b".repeat(64),
        assessment_results_file_sha256: "a".repeat(64),
        native_href: "nested/AR%20file%23name.json#original-source-uuid".to_string(),
    }];
    let plans = vec![PlanRow {
        key: "fixture-plan".to_string(),
        uuid: "fixture-plan-uuid".to_string(),
        authoring_sha256: "c".repeat(64),
        generated_native_sha256: "d".repeat(64),
        supplied_native_sha256: None,
        supplied_native_matched: false,
        result_uuid: "original-result-uuid".to_string(),
        result_key: "result".to_string(),
        source_objects: 7,
        sources: source_kinds
            .into_iter()
            .map(|kind| SourceFileRow {
                kind,
                native_href: format!("source/{kind}.json"),
                sha256: "a".repeat(64),
                bytes: 3,
            })
            .collect(),
        records,
        timeline,
        trace,
    }];
    let counts = count_plans(&plans).unwrap();
    PortfolioReport {
        schema_version: SCHEMA_VERSION,
        validation_scope: validation_scope(&counts),
        as_of: "2026-02-01".to_string(),
        due_soon_days: 0,
        counts,
        plans,
        boundary: BOUNDARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Shared result/file identities are charged per plan; null/cancelled rows remain denominators.
    #[test]
    fn complete_sums_keep_shared_sources_and_all_schedule_scope() {
        let mut report = test_report();
        let mut another = test_report().plans.pop().unwrap();
        another.key = "second-explicit-plan".to_string();
        another.uuid = "second-plan-uuid".to_string();
        report.plans.push(another);
        report.counts = count_plans(&report.plans).unwrap();
        validate_report(&report).unwrap();
        assert_eq!(report.counts.plans, 2);
        assert_eq!(report.counts.supplied_native_artifacts, 0);
        assert_eq!(report.counts.authoring_only_plans, 2);
        assert_eq!(report.counts.source_files, 10);
        assert_eq!(report.counts.source_bytes, 30);
        assert_eq!(report.counts.source_objects, 14);
        assert_eq!(report.counts.items, 6);
        assert_eq!(report.counts.milestones, 2);
        assert_eq!(report.counts.records, 8);
        assert_eq!(report.counts.source_selections, 2);
        assert_eq!(report.counts.history_events, 2);
        assert_eq!(report.counts.before_first_assertion, 2);
        assert_eq!(report.counts.cancelled, 2);
        assert_eq!(report.counts.blocked, 2);
        assert_eq!(report.counts.overdue, 2);
        assert_eq!(report.counts.due_soon, 2);
        report.plans[1].uuid = report.plans[0].uuid.clone();
        assert!(validate_report(&report).is_err());
    }

    /// Mixed/all-virtual scope remains explicit and the shipping entry point requires actual native inputs.
    #[test]
    fn native_artifact_denominators_never_grant_preview_s1_credit() {
        let mut report = test_report();
        assert_eq!(report.validation_scope, "authoring-preview-with-optional-native-pairs");
        report.plans[0].supplied_native_matched = true;
        report.plans[0].supplied_native_sha256 = Some("e".repeat(64));
        report.counts = count_plans(&report.plans).unwrap();
        report.validation_scope = validation_scope(&report.counts);
        assert_eq!(report.counts.supplied_native_artifacts, 1);
        assert_eq!(report.counts.authoring_only_plans, 0);
        assert_eq!(report.validation_scope, "supplied-native-artifact-portfolio");
        let mut other = test_report().plans.pop().unwrap();
        other.key = "other".to_string();
        other.uuid = "other-uuid".to_string();
        report.plans.push(other);
        report.counts = count_plans(&report.plans).unwrap();
        assert!(validate_report(&report).is_err());
        report.validation_scope = validation_scope(&report.counts);
        validate_report(&report).unwrap();
        assert_eq!(report.validation_scope, "authoring-preview-with-optional-native-pairs");
        assert_eq!(report.counts.supplied_native_artifacts, 1);
        assert_eq!(report.counts.authoring_only_plans, 1);
        let absent =
            Input { manifest: PathBuf::from("missing-authoring.json"), native_artifact: None };
        assert!(
            prepare_native_portfolio(&[absent], "2026-02-01", 0)
                .unwrap_err()
                .to_string()
                .contains("requires an explicit native file")
        );
    }

    /// RFC3339 offsets and fractions order by actual instant; all future events stay present.
    #[test]
    fn timeline_orders_actual_instants_instead_of_timestamp_spelling() {
        let actor =
            workflow::Actor { role_id: "owner".to_string(), party_key: "alice".to_string() };
        let event = |key: &str, at: &str| Event {
            key: key.to_string(),
            actor: actor.clone(),
            at: at.to_string(),
            from: None,
            to: State::Planned,
            rationale: "explicit".to_string(),
            closure: None,
        };
        let history = vec![
            event("z-later", "2026-03-01T00:30:00Z"),
            event("offset-earlier", "2026-03-01T01:00:00+01:00"),
            event("fraction-between", "2026-03-01T00:00:00.500Z"),
        ];
        let mut rows = Vec::new();
        append_events(&mut rows, "work", None, &history, full_date("2026-02-01").unwrap()).unwrap();
        sort_timeline(&mut rows);
        assert_eq!(
            rows.iter().map(|row| row.event_key.as_str()).collect::<Vec<_>>(),
            vec!["offset-earlier", "fraction-between", "z-later"]
        );
        assert!(rows.iter().all(|row| row.after_as_of));
        assert_eq!(rows.len(), history.len());
    }

    /// Syntactic authoring fixture only; absent source artifacts cannot confer workflow admission.
    fn declaration() -> Value {
        let pin = |name: &str| {
            json!({"artifact":format!("{name}.json"),"href":format!("{name}.json"),
            "expected_sha256":"a".repeat(64),"root_uuid":"11111111-1111-4111-8111-111111111111",
            "document_version":"1.0.0","oscal_version":"1.2.3"})
        };
        json!({"schema_version":"forge.poam/1","document":{"key":"plan","title":"Authored plan","version":"1.0.0","last_modified":"2026-02-20T00:00:00Z"},
            "source":{"assessment_results":pin("ar"),"result":{"uuid":"11111111-1111-4111-8111-111111111111","key":"result"},
                "context":{"assessment_plan":pin("ap"),"ssp":pin("ssp"),"profile":pin("profile"),"catalog":pin("catalog")}},
            "roles":[{"id":"owner","title":"Owner"},{"id":"reviewer","title":"Reviewer"}],
            "parties":[{"key":"alice","type":"person","name":"PRIVATE OWNER"},{"key":"bob","type":"person","name":"PRIVATE REVIEWER"}],
            "items":[{"key":"work","title":"Work","description":"Explicit work",
                "source_refs":[{"kind":"finding","key":"finding","uuid":"22222222-2222-4222-8222-222222222222","result_uuid":"11111111-1111-4111-8111-111111111111","expected_sha256":"b".repeat(64)}],
                "owners":[{"role_id":"owner","party_key":"alice","rationale":"Explicit responsibility"}],"target_date":"2026-02-01","state":"planned",
                "history":[{"key":"initial","actor":{"role_id":"owner","party_key":"alice"},"at":"2026-01-01T00:00:00Z","from":null,"to":"planned","rationale":"Explicit plan","closure":null}],"milestones":[{"key":"step","outcome":"Explicit milestone","target_date":"2026-01-20","depends_on":[],
                    "owners":[{"role_id":"owner","party_key":"alice","rationale":"Explicit responsibility"}],"state":"planned",
                    "history":[{"key":"initial","actor":{"role_id":"owner","party_key":"alice"},"at":"2026-01-01T00:00:00Z","from":null,"to":"planned","rationale":"Explicit plan","closure":null}]}]} ]})
    }

    /// The public entry point refuses terminal history and duplicate plans before any missing-source read.
    #[test]
    fn public_portfolio_preserves_default_terminal_refusal_and_stable_plan_uniqueness() {
        let dir = tempfile::tempdir().unwrap();
        let mut authored = declaration();
        let original = serde_json::to_vec(&authored).unwrap();
        workflow::parse(&original).unwrap();
        let path = dir.path().join("one.json");
        let other = dir.path().join("two.json");
        std::fs::write(&path, &original).unwrap();
        std::fs::write(&other, &original).unwrap();
        let input = Input { manifest: path.clone(), native_artifact: None };
        let duplicate = Input { manifest: other, native_artifact: None };
        let cause = prepare(&[input.clone(), duplicate], "2026-02-01", 0).unwrap_err().to_string();
        assert!(cause.contains("duplicate stable plan identity"));
        for state in ["completed-asserted", "accepted-risk-asserted"] {
            authored = declaration();
            authored["items"][0]["state"] = json!(state);
            authored["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"start","actor":{"role_id":"owner","party_key":"alice"},
                "at":"2026-01-05T00:00:00Z","from":"planned","to":"in-progress","rationale":"Explicit work start","closure":null}));
            authored["items"][0]["history"].as_array_mut().unwrap().push(json!({"key":"terminal","actor":{"role_id":"owner","party_key":"alice"},
                "at":"2026-01-10T00:00:00Z","from":"in-progress","to":state,"rationale":"Explicit assertion",
                "closure":{"reviewer":{"role_id":"reviewer","party_key":"bob"},"reviewed_at":"2026-01-10T12:00:00Z",
                    "rationale":"Declared review only","evidence":[{"key":"proof","href":"proof.json","expected_sha256":"c".repeat(64)}]}}));
            if state == "completed-asserted" {
                let step = &mut authored["items"][0]["milestones"][0];
                step["state"] = json!("completed-asserted");
                step["history"].as_array_mut().unwrap().push(json!({"key":"start","actor":{"role_id":"owner","party_key":"alice"},
                    "at":"2026-01-05T00:00:00Z","from":"planned","to":"in-progress","rationale":"Explicit milestone start","closure":null}));
                step["history"].as_array_mut().unwrap().push(json!({"key":"terminal","actor":{"role_id":"owner","party_key":"alice"},
                    "at":"2026-01-09T00:00:00Z","from":"in-progress","to":"completed-asserted","rationale":"Explicit milestone assertion",
                    "closure":{"reviewer":{"role_id":"reviewer","party_key":"bob"},"reviewed_at":"2026-01-09T12:00:00Z",
                        "rationale":"Declared review only","evidence":[{"key":"proof","href":"proof.json","expected_sha256":"c".repeat(64)}]}}));
            } else {
                authored["items"][0]["source_refs"].as_array_mut().unwrap().push(json!({"kind":"risk","key":"risk",
                    "uuid":"33333333-3333-4333-8333-333333333333","result_uuid":"11111111-1111-4111-8111-111111111111","expected_sha256":"c".repeat(64)}));
            }
            std::fs::write(&path, serde_json::to_vec(&authored).unwrap()).unwrap();
            let cause =
                prepare(std::slice::from_ref(&input), "2026-02-01", 0).unwrap_err().to_string();
            assert!(cause.contains("pending recorded closure disposition"), "{cause}");
        }
    }

    /// Equality preserves decoded tabs/newlines but refuses native extensions and receipt changes.
    #[test]
    fn native_pair_is_exact_and_closed_without_reverse_import() {
        let value = json!({"root":{"props":[{"value":"Authored\nline\twith tab"}],
            "href":"AR%20file%23name.json#original-uuid","sha256":"a".repeat(64)}});
        let pretty = serde_json::to_vec_pretty(&value).unwrap();
        assert!(match_native(&pretty, &serde_json::to_vec(&value).unwrap()).is_ok());
        let mut changed = value.clone();
        changed["root"]["extra"] = json!(true);
        assert!(match_native(&pretty, &serde_json::to_vec(&changed).unwrap()).is_err());
        changed = value.clone();
        changed["root"]["props"][0]["value"] = json!("Authored line with tab");
        assert!(match_native(&pretty, &serde_json::to_vec(&changed).unwrap()).is_err());
        for field in ["href", "sha256"] {
            changed = value.clone();
            changed["root"][field] = json!("changed");
            assert!(match_native(&pretty, &serde_json::to_vec(&changed).unwrap()).is_err());
        }
        assert!(native_value(br#"{"root":1,"root":1}"#).is_err());
        assert!(native_value(br#"{"root":1} {"another":2}"#).is_err());
    }

    /// This generation control exercises the actual confined reader; no workflow/native qualification.
    #[test]
    fn original_capture_refuses_literal_drift_and_same_byte_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let path = root.join("source.json");
        std::fs::write(&path, b"original").unwrap();
        let original = capture(&root, Path::new("source.json"), 8).unwrap();
        assert!(verify_original(&root, &original).is_ok());
        std::fs::write(&path, b"changed!").unwrap();
        assert!(verify_original(&root, &original).is_err());
        std::fs::write(&path, b"original").unwrap();
        let replacement = root.join("replacement.json");
        std::fs::write(&replacement, b"original").unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::rename(replacement, &path).unwrap();
        assert!(verify_original(&root, &original).is_err());
    }

    /// Aggregate exhaustion and actual file/input aliases fail before sources can be prepared.
    #[test]
    fn aggregate_and_explicit_input_aliases_refuse_whole_result() {
        let mut count = MAX_EVENTS - 1;
        charge(&mut count, 1, MAX_EVENTS).unwrap();
        assert!(charge(&mut count, 1, MAX_EVENTS).is_err());
        assert_eq!(count, MAX_EVENTS);
        let mut overflow = usize::MAX;
        assert!(charge(&mut overflow, 1, usize::MAX).is_err());
        let original =
            Original { relative: PathBuf::from("plan.json"), bytes: vec![], identity: (4, 5) };
        let root = Path::new("/explicit");
        let mut paths = BTreeSet::new();
        let mut identities = BTreeSet::new();
        reserve_input(root, &original, &mut paths, &mut identities).unwrap();
        assert!(reserve_input(root, &original, &mut paths, &mut identities).is_err());
        let alias =
            Original { relative: PathBuf::from("other.json"), bytes: vec![], identity: (4, 5) };
        assert!(reserve_input(root, &alias, &mut paths, &mut identities).is_err());
        for name in [
            "../other.json",
            "nested/native.json",
            "/absolute.json",
            "native.xml",
            "native\\other.json",
        ] {
            assert!(native_filename(Path::new(name)).is_err());
        }
        assert!(native_filename(Path::new("Native %23 é.json")).is_ok());
        assert!(prepare(&[], "2026-02-01", 0).is_err());
        assert!(prepare(&[], "2026-02-30", 0).is_err());
    }
}

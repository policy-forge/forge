//! Complete read-only closure-reference inspection over actual current shared captures.
//!
//! Author associations, current local byte/hash observations and explicit-date
//! freshness remain distinct. This module creates no native POA&M, verifies no
//! evidence sufficiency and admits no terminal workflow or actor authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use serde::ser::SerializeSeq as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::identity;
use super::manifest::{self, SourceReference};
use super::source::{self, CapturedSource};
use super::workflow::{Event, EvidenceAssertion, Milestone, WorkItem, WorkflowManifest};
use super::workflow_baseline;
use crate::ForgeError;
use crate::evidence_capture::{CaptureProof, CaptureRole, CaptureSession, ProjectionBudget};
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};
use crate::linkage::fresh::{self, LocalBindingStatus, PreparedFreshLinkage};
use crate::linkage::{EvidenceFreshness, EvidenceRecord, EvidenceReference, LinkRecord};

/// Complete closure-row ceiling; the shared graph budget can refuse earlier.
const MAX_ROWS: usize = 100_000;
/// Existing linkage declaration key ceiling in UTF-8 bytes.
const MAX_LINKAGE_KEY_BYTES: usize = 16 * 1024;
/// Closed local-reference bound without URI, query, fragment or traversal support.
const MAX_PATH_BYTES: usize = 4096;
/// Exact minimized output boundary, without any implicit evidence judgment.
const BOUNDARY: &str = "Current byte/hash and date metadata plus explicit author associations only; no evidence content, authenticity, sufficiency, effectiveness, closure admission or compliance judgment.";

/// Closed companion declaration; parsing alone establishes no actual input freshness.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceLinks {
    /// Fixed separate companion format, never the native POA&M format.
    pub(crate) schema_version: String,
    /// Exact immutable plan identity paired with the supplied workflow declaration.
    pub(crate) plan_key: String,
    /// Explicit current linkage manifest and raw-byte pin.
    pub(crate) linkage: LinkageDeclaration,
    /// Complete explicit associations; missing assertions are reported as unbound.
    bindings: Vec<Binding>,
}

/// Exact current linkage declaration; the actual file is captured before use.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkageDeclaration {
    /// Root-relative local manifest, with its own nested declaration base preserved.
    pub(crate) artifact: PathBuf,
    /// Expected SHA-256 of the complete original linkage manifest bytes.
    pub(crate) expected_sha256: String,
    /// Exact native linkage project key, without whitespace normalization.
    pub(crate) project_key: String,
}

/// One exact item/milestone/event/evidence association, without inferred links.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    /// Complete locator; null milestone is required explicitly for item history.
    locator: Locator,
    /// Explicit linkage link key whose evidence membership is inspected.
    link_key: String,
    /// Explicit native evidence key; unresolved keys remain action rows.
    evidence_key: String,
}

/// Exact closure evidence location, preserving the item/milestone distinction.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)] // Closed wire locator uses exact explicit key names.
struct Locator {
    /// Parent item immutable key.
    item_key: String,
    /// Exact milestone key or required null for the item's own history.
    #[serde(deserialize_with = "required_nullable")]
    milestone_key: Option<String>,
    /// Exact event key in the selected item or milestone history.
    event_key: String,
    /// Exact evidence assertion key in that event's closure declaration.
    assertion_evidence_key: String,
}

/// Borrowed, already validated locator; no repeated authored string allocation.
#[derive(Serialize)]
#[allow(clippy::struct_field_names)] // Borrowed wire locator preserves the same closed names.
struct RowLocator<'a> {
    /// Exact item key, not a source finding identity.
    item_key: &'a str,
    /// Required null or exact milestone identity.
    milestone_key: Option<&'a str>,
    /// Exact declared history event key.
    event_key: &'a str,
    /// Exact declared closure evidence key.
    assertion_evidence_key: &'a str,
}

/// Fixed, complete row-status partition independent of freshness classification.
#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum MatchStatus {
    /// No explicit binding was supplied for this existing assertion.
    Unbound,
    /// Actual local path/identity, three hashes and two sizes match.
    Matched,
    /// The declared link does not contain the declared evidence key.
    WrongLinkMembership,
    /// A relative local assertion cannot denote an unverified URI.
    ReferenceKindMismatch,
    /// Assertion root-relative path differs from the original evidence path.
    HrefMismatch,
    /// Asserted, approved or observed hash/size facts differ.
    HashMismatch,
    /// A declared native key is unresolved or actual local evidence is absent.
    Unavailable,
}

/// Borrow original exact source selections without cloning large stable keys per row.
struct SourceRefs<'a>(&'a [SourceReference]);

/// One exact selected finding/risk tuple; current source admission occurs before serialization.
#[derive(Serialize)]
struct SourceRefRow<'a> {
    /// Original finding/risk kind, without eligibility filtering.
    kind: &'a super::manifest::SourceKind,
    /// Exact original stable key within the existing 64 KiB source domain.
    key: &'a str,
    /// Exact original native UUID, not a new work identity.
    uuid: &'a str,
    /// Exact currently selected native result UUID.
    result_uuid: &'a str,
    /// Computed canonical source-object digest, separate from whole-file hashes.
    sha256: &'a str,
}

impl Serialize for SourceRefs<'_> {
    /// Stream complete borrowed source tuples with no prefix or per-row string cloning.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_seq(Some(self.0.len()))?;
        for source in self.0 {
            output.serialize_element(&SourceRefRow {
                kind: &source.kind,
                key: &source.key,
                uuid: &source.uuid,
                result_uuid: &source.result_uuid,
                sha256: &source.expected_sha256,
            })?;
        }
        output.end()
    }
}

/// One complete declared closure reference with minimized current metadata.
#[derive(Serialize)]
struct EvidenceRow<'a> {
    /// Complete original assertion locator, never a guessed successful prefix.
    locator: RowLocator<'a>,
    /// Existing stable work UUID-v5 protocol.
    item_uuid: String,
    /// Existing milestone UUID-v5 protocol or required null.
    milestone_uuid: Option<String>,
    /// Original reviewer-declared expected hash.
    assertion_sha256: &'a str,
    /// Explicit paired linkage project, null only for an unbound assertion.
    linkage_project_key: Option<&'a str>,
    /// Explicit declared link key, including unresolved declarations.
    link_key: Option<&'a str>,
    /// Native current link ID, never invented for an unresolved key.
    link_id: Option<&'a str>,
    /// Explicit declared evidence key, including unresolved declarations.
    linkage_evidence_key: Option<&'a str>,
    /// Actual native date/hash classification or explicit unmeasured null.
    freshness: Option<&'a EvidenceFreshness>,
    /// Independent local assertion/native membership match condition.
    match_status: MatchStatus,
    /// Existing local approved hash; URI expected hashes are not local approvals.
    approved_sha256: Option<&'a str>,
    /// Actual captured local digest, never copied from asserted approval alone.
    observed_sha256: Option<String>,
    /// Existing local approved full-u64 size domain.
    approved_size: Option<u64>,
    /// Actual captured local size bounded by its existing per-file cap.
    observed_size: Option<u64>,
    /// True only after an actual local original has been reconciled.
    local_bytes_revalidated: bool,
    /// Declared event's UTC calendar date falls after the explicit inspection date.
    asserted_after_as_of: bool,
    /// Every exact source tuple selected by this item's author.
    source_refs: SourceRefs<'a>,
    /// Existing native admitted date representation, without a new ten-byte restriction.
    recorded_valid_through: Option<String>,
}

/// Complete denominators and independent status/freshness partitions.
#[derive(Default, Serialize)]
struct Counts {
    /// Every decoded current item, including terminal and future histories.
    items: usize,
    /// Every decoded current milestone.
    milestones: usize,
    /// Every decoded item and milestone history event.
    history_events: usize,
    /// Every closure evidence entry, without as-of filtering.
    closure_assertions: usize,
    /// Complete closed companion binding count.
    bindings_supplied: usize,
    /// Complete output row count; exactly equals `closure_assertions`.
    assertion_rows: usize,
    /// Distinct declared project/evidence pairs in used bindings, including unresolved keys.
    distinct_referenced_evidence: usize,
    /// All actual successfully captured local evidence in the whole native manifest.
    distinct_captured_local_evidence: usize,
    /// Whole native evidence declaration count, including unused URI/absent entries.
    linkage_evidence: usize,
    /// Whole native link declaration count, including unused links.
    linkage_links: usize,
    /// Complete unbound status partition.
    unbound: usize,
    /// Four membership/kind/path/hash mismatch status partitions combined.
    mismatched: usize,
    /// Per-row current classification, including repeated references.
    current: usize,
    /// Per-row expiring classification.
    expiring: usize,
    /// Per-row expired classification.
    expired: usize,
    /// Per-row changed-byte or changed-size classification.
    changed: usize,
    /// Per-row actual absent local evidence classification.
    unavailable: usize,
    /// Per-row unverified URI classification; no network is used.
    unverified_uri: usize,
    /// Possibly overlapping count of future declared assertion dates.
    future_assertions: usize,
    /// Actual distinct present originals; unavailable observations are excluded.
    captured_original_generations: usize,
    /// Complete matched-local status partition, independently of date freshness.
    matched: usize,
    /// Complete unresolved/absent binding status partition.
    binding_unavailable: usize,
    /// Explicit null freshness count, never interpreted as current.
    freshness_unmeasured: usize,
}

/// Closed complete minimized report, encoded while rows borrow admitted declarations.
#[derive(Serialize)]
struct Inspection<'a> {
    /// Fixed separate inspection report contract.
    schema_version: &'static str,
    /// Complete whole-input result; malformed/over-bound input returns an error.
    status: &'static str,
    /// Read-only declaration/current-source/local-metadata scope only.
    validation_scope: &'static str,
    /// Exact immutable current plan key.
    plan_key: &'a str,
    /// Canonical caller-supplied calendar date.
    as_of: &'a str,
    /// Whole captured original plan bytes, including formatting.
    plan_sha256: String,
    /// Whole captured original companion bytes, including formatting.
    bindings_sha256: String,
    /// Actual paired linkage manifest whole-byte hash.
    linkage_manifest_sha256: &'a str,
    /// Actual current AR whole-file hash, distinct from object hashes.
    current_source_sha256: &'a str,
    /// Actual selected result canonical native UUID.
    current_result_uuid: &'a str,
    /// Actual selected result exact stable key, retaining its 64 KiB domain.
    current_result_key: &'a str,
    /// Always false; inspection creates no terminal admission.
    terminal_admitted: bool,
    /// Always false; no native artifact is emitted or validated here.
    artifact_validated: bool,
    /// Complete action/undisposed terminal-history condition.
    review_required: bool,
    /// Complete count partitions and source/linkage denominators.
    summary: Counts,
    /// Every item and milestone closure assertion, including future declarations.
    rows: Vec<EvidenceRow<'a>>,
    /// Fixed no-authority/no-content boundary.
    boundary: &'static str,
}

/// Complete current shared-original proof and bounded read-only report.
///
/// Private fields prevent detached caller-authored proof construction. No native
/// renderer accepts this type. Debug exposes only fixed counts and review.
pub struct PreparedEvidence {
    /// All actual present/absent original generations, with confined stream rechecks.
    proof: CaptureProof,
    /// Privately qualified current five-source inventory, with no duplicate original Vec.
    source: CapturedSource,
    /// Privately captured native linkage result, never a supplied index assertion.
    linkage: PreparedFreshLinkage,
    /// Complete bounded minimized JSON, not a partial successful prefix.
    bytes: Vec<u8>,
    /// Read-only action condition, never a closure-admission flag.
    review_required: bool,
}

impl std::fmt::Debug for PreparedEvidence {
    /// Omit original bytes, paths, plan/party/source/linkage keys and author prose.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedEvidence")
            .field("report_bytes", &self.bytes.len())
            .field("source_objects", &self.source.inventory().objects.len())
            .field("linkage_evidence", &self.linkage.index().evidence.len())
            .field("review_required", &self.review_required)
            .finish_non_exhaustive()
    }
}

impl PreparedEvidence {
    /// Borrow the complete encoded minimized report without granting artifact authority.
    #[must_use]
    pub fn report(&self) -> &[u8] {
        &self.bytes
    }

    /// Identify valid complete action/terminal inspection results for Root's exit-one mapping.
    #[must_use]
    pub fn review_required(&self) -> bool {
        self.review_required
    }

    /// Reconcile all actual originals immediately before caller publication or stdout.
    /// # Errors
    /// Refuses changed, missing, replacement, appeared or unsafe original generations.
    pub fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.proof.verify_inputs().map_err(|_| error("original inputs changed or became unsafe"))
    }

    /// Borrow all present and unavailable-local absolute paths for internal full-path preflight only.
    pub(super) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.proof.input_paths()
    }
}

/// Capture explicit current declarations, source closure and fresh native linkage for read-only inspection.
///
/// Completion/risk-acceptance history can be inspected structurally but never
/// reaches public workflow/native admission. Every terminal event requires review,
/// including future/cancelled declarations. As-of affects freshness/future labels,
/// never whether a closure row is retained. The single original pool is 100 MiB,
/// 10,137 observations and 100,000 complete relationships; output is <=10 MiB.
/// # Errors
/// Refuses malformed, aliased, stale, extraneous, over-bound or unsafe complete
/// inputs with fixed redacted categories; no original content or paths are output.
pub fn prepare(
    path: &Path,
    bindings_path: &Path,
    as_of: &str,
) -> Result<PreparedEvidence, ForgeError> {
    prepare_with_report(path, bindings_path, as_of, None)
}

/// Reserve Root's explicit report descendant before any source/declaration capture.
///
/// Reservation is an internal namespace guard, not an original-byte observation.
/// Public preparation retains its existing no-output-reservation behavior. The
/// caller still checks complete held input paths and rechecks before publication.
/// # Errors
/// Refuses unsafe or colliding reserved paths and the same complete input errors
/// as public preparation, using fixed redacted error categories.
pub(super) fn prepare_with_report(
    path: &Path,
    bindings_path: &Path,
    as_of: &str,
    report_relative: Option<&Path>,
) -> Result<PreparedEvidence, ForgeError> {
    let date = full_date(as_of)?;
    let plan_path = std::path::absolute(path).map_err(|_| error("plan path is invalid"))?;
    let root = plan_path.parent().ok_or_else(|| error("plan directory is invalid"))?;
    let mut capture = CaptureSession::new(root).map_err(|_| error("plan root is unsafe"))?;
    if let Some(report) = report_relative {
        capture
            .reserve_output(report)
            .map_err(|_| error("report path reservation is invalid or collides"))?;
    }
    let plan_relative = descendant(capture.root(), &plan_path)?;
    let bindings_absolute =
        std::path::absolute(bindings_path).map_err(|_| error("companion path is invalid"))?;
    let bindings_relative = descendant(capture.root(), &bindings_absolute)?;
    let plan = capture
        .required(&plan_relative, CaptureRole::WorkflowDeclaration, manifest::MAX_MANIFEST_BYTES)
        .map_err(|_| error("plan capture is invalid or over-bound"))?;
    let companion = capture
        .required(&bindings_relative, CaptureRole::EvidenceBindings, manifest::MAX_MANIFEST_BYTES)
        .map_err(|_| error("companion capture is invalid or over-bound"))?;
    let manifest = workflow_baseline::parse_declaration(plan.bytes())
        .map_err(|_| error("workflow declaration is invalid"))?;
    let bindings = parse_bindings(companion.bytes())?;
    if bindings.plan_key != manifest.document.key {
        return Err(error("companion plan identity differs"));
    }
    let mut summary = declaration_counts(&manifest, bindings.bindings.len(), &mut capture)?;
    let captured_plan_path = capture.root().join(plan.path());
    let source = source::load_with_capture(&captured_plan_path, &manifest.source, &mut capture)
        .map_err(|_| error("current source closure is invalid or stale"))?;
    validate_current_selection(&manifest, &source)?;
    let mut linkage_projection = ProjectionBudget::new();
    let linkage = fresh::prepare_fresh(
        &bindings.linkage.artifact,
        &bindings.linkage.expected_sha256,
        date,
        &mut capture,
        &mut linkage_projection,
    )
    .map_err(|_| error("current linkage closure is invalid or over-bound"))?;
    if linkage.index().project_key != bindings.linkage.project_key {
        return Err(error("current linkage project identity differs"));
    }
    let mut projection = ProjectionBudget::new();
    let rows = assertion_rows(
        &manifest,
        &bindings,
        &linkage,
        date,
        &mut summary,
        &mut capture,
        &mut projection,
    )?;
    summary.linkage_evidence = linkage.index().evidence.len();
    summary.linkage_links = linkage.index().links.len();
    summary.distinct_captured_local_evidence = linkage
        .index()
        .evidence
        .iter()
        .filter(|evidence| {
            matches!(
                &evidence.reference,
                EvidenceReference::Local { observed_sha256: Some(_), observed_size: Some(_), .. }
            )
        })
        .count();
    let review_required = manifest.items.iter().any(|item| {
        item.history
            .iter()
            .chain(item.milestones.iter().flat_map(|step| step.history.iter()))
            .any(|event| event.to.terminal())
    }) || rows.iter().any(|row| {
        row.match_status != MatchStatus::Matched
            || row.freshness != Some(&EvidenceFreshness::Current)
    });
    let proof = capture.finish();
    proof.verify_inputs().map_err(|_| error("original inputs changed during inspection"))?;
    summary.captured_original_generations = proof.captured_original_generations();
    reconcile(&summary, rows.len())?;
    let report = Inspection {
        schema_version: "forge.poam-evidence-inspection/1",
        status: "complete",
        validation_scope: "readonly-declaration-and-current-source-evidence-metadata",
        plan_key: &manifest.document.key,
        as_of,
        plan_sha256: sha256_hex(plan.bytes()),
        bindings_sha256: sha256_hex(companion.bytes()),
        linkage_manifest_sha256: &linkage.index().provenance.manifest_sha256,
        current_source_sha256: &source.inventory().source_sha256,
        current_result_uuid: &source.inventory().result_uuid,
        current_result_key: &source.inventory().result_key,
        terminal_admitted: false,
        artifact_validated: false,
        review_required,
        summary,
        rows,
        boundary: BOUNDARY,
    };
    let bytes = ProjectionBudget::encode(&report)
        .map_err(|_| error("complete report exceeds its byte bound"))?;
    Ok(PreparedEvidence { proof, source, linkage, bytes, review_required })
}

/// Parse only the explicit closed companion shape for preflight; this grants no current-byte authority.
/// # Errors
/// Refuses malformed/duplicate JSON, missing required nulls, unknown fields,
/// unsafe local paths, invalid keys/hash pins, duplicate locators or complete bounds.
pub(crate) fn parse_bindings(bytes: &[u8]) -> Result<EvidenceLinks, ForgeError> {
    if bytes.len() as u64 > manifest::MAX_MANIFEST_BYTES {
        return Err(error("companion exceeds its raw bound"));
    }
    let value = json_strict::parse_value(
        bytes,
        "POA&M evidence companion",
        Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("companion JSON is malformed, duplicate or unbounded"))?;
    if value
        .get("bindings")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| rows.len() > MAX_ROWS)
    {
        return Err(error("complete companion binding count exceeds its bound"));
    }
    let bindings: EvidenceLinks = serde_json::from_value(value)
        .map_err(|_| error("companion is not the closed required shape"))?;
    if bindings.schema_version != "forge.poam-evidence-links/1" {
        return Err(error("companion format is unsupported"));
    }
    key(&bindings.plan_key, 256)?;
    key(&bindings.linkage.project_key, MAX_LINKAGE_KEY_BYTES)?;
    relative(&bindings.linkage.artifact)?;
    json_strict::validate_lowercase_sha256(
        "linkage manifest hash",
        &bindings.linkage.expected_sha256,
    )
    .map_err(|_| error("companion linkage hash is invalid"))?;
    if bindings.bindings.len() > MAX_ROWS {
        return Err(error("complete companion binding count exceeds its bound"));
    }
    let mut locators = BTreeSet::new();
    for binding in &bindings.bindings {
        key(&binding.locator.item_key, 256)?;
        if let Some(milestone) = &binding.locator.milestone_key {
            key(milestone, 256)?;
        }
        key(&binding.locator.event_key, 256)?;
        key(&binding.locator.assertion_evidence_key, 256)?;
        key(&binding.link_key, MAX_LINKAGE_KEY_BYTES)?;
        key(&binding.evidence_key, MAX_LINKAGE_KEY_BYTES)?;
        if !locators.insert(locator_key(&binding.locator)) {
            return Err(error("companion duplicates an exact assertion locator"));
        }
    }
    Ok(bindings)
}

/// Require an explicit nullable field instead of allowing serde's missing-Option default.
fn required_nullable<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

/// Borrow the complete locator tuple; never normalize or join authored keys with ambiguous separators.
fn locator_key(locator: &Locator) -> (&str, Option<&str>, &str, &str) {
    (
        &locator.item_key,
        locator.milestone_key.as_deref(),
        &locator.event_key,
        &locator.assertion_evidence_key,
    )
}

/// Precharge every declaration/reference record before native selection maps or report adjacency grow.
fn declaration_counts(
    manifest: &WorkflowManifest,
    bindings: usize,
    capture: &mut CaptureSession,
) -> Result<Counts, ForgeError> {
    let mut counts =
        Counts { items: manifest.items.len(), bindings_supplied: bindings, ..Counts::default() };
    capture.relationships(bindings)?;
    for item in &manifest.items {
        capture.relationships(1)?;
        capture.relationships(item.source_refs.len())?;
        capture.relationships(item.owners.len())?;
        for event in &item.history {
            count_event(event, &mut counts, capture)?;
        }
        for milestone in &item.milestones {
            bounded_add(&mut counts.milestones, 1)?;
            capture.relationships(1)?;
            capture.relationships(milestone.depends_on.len())?;
            capture.relationships(milestone.owners.len())?;
            for event in &milestone.history {
                count_event(event, &mut counts, capture)?;
            }
        }
    }
    Ok(counts)
}

/// Preserve every history/closure denominator, including future and terminal declarations.
fn count_event(
    event: &Event,
    counts: &mut Counts,
    capture: &mut CaptureSession,
) -> Result<(), ForgeError> {
    bounded_add(&mut counts.history_events, 1)?;
    capture.relationships(1)?;
    if let Some(closure) = &event.closure {
        bounded_add(&mut counts.closure_assertions, closure.evidence.len())?;
        capture.relationships(closure.evidence.len())?;
    }
    Ok(())
}

/// Match every authored source tuple against the actually captured selected result without eligibility inference.
fn validate_current_selection(
    manifest: &WorkflowManifest,
    source: &CapturedSource,
) -> Result<(), ForgeError> {
    let inventory = source.inventory();
    if manifest.source.assessment_results.expected_sha256 != inventory.source_sha256
        || manifest.source.result.uuid != inventory.result_uuid
        || manifest.source.result.key != inventory.result_key
    {
        return Err(error("current source result tuple differs"));
    }
    for item in &manifest.items {
        source
            .validate_selection(&item.source_refs)
            .map_err(|_| error("current selected source object tuple differs"))?;
    }
    Ok(())
}

/// Join all closure assertions with complete explicit bindings and actual private linkage facts.
fn assertion_rows<'a>(
    manifest: &'a WorkflowManifest,
    bindings: &'a EvidenceLinks,
    linkage: &'a PreparedFreshLinkage,
    as_of: NaiveDate,
    summary: &mut Counts,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<Vec<EvidenceRow<'a>>, ForgeError> {
    let by_locator: BTreeMap<_, _> =
        bindings.bindings.iter().map(|binding| (locator_key(&binding.locator), binding)).collect();
    capture.relationships(linkage.index().links.len())?;
    capture.relationships(linkage.index().evidence.len())?;
    let links: BTreeMap<_, _> =
        linkage.index().links.iter().map(|link| (link.key.as_str(), link)).collect();
    let evidence: BTreeMap<_, _> =
        linkage.index().evidence.iter().map(|evidence| (evidence.key.as_str(), evidence)).collect();
    let mut used = BTreeSet::new();
    let mut evidence_pairs = BTreeSet::new();
    let mut rows = Vec::new();
    for item in &manifest.items {
        for (milestone, events) in std::iter::once((None, item.history.as_slice())).chain(
            item.milestones.iter().map(|milestone| (Some(milestone), milestone.history.as_slice())),
        ) {
            for event in events {
                let Some(closure) = &event.closure else { continue };
                for assertion in &closure.evidence {
                    if rows.len() >= MAX_ROWS {
                        return Err(error("complete assertion row count exceeds its bound"));
                    }
                    capture.relationships(1)?;
                    capture.relationships(item.source_refs.len())?;
                    let location = (
                        item.key.as_str(),
                        milestone.map(|m| m.key.as_str()),
                        event.key.as_str(),
                        assertion.key.as_str(),
                    );
                    let binding = by_locator.get(&location).copied();
                    if let Some(binding) = binding {
                        used.insert(location);
                        evidence_pairs.insert((
                            bindings.linkage.project_key.as_str(),
                            binding.evidence_key.as_str(),
                        ));
                    }
                    let mut row = row_base(manifest, item, milestone, event, assertion, as_of)?;
                    if let Some(binding) = binding {
                        join_row(
                            &mut row, binding, bindings, linkage, &links, &evidence, assertion,
                        )?;
                    }
                    projection
                        .admit_row(&row)
                        .map_err(|_| error("complete assertion metadata exceeds its byte bound"))?;
                    count_row(&row, summary)?;
                    rows.push(row);
                }
            }
        }
    }
    if used.len() != bindings.bindings.len() {
        return Err(error("companion contains an extraneous assertion locator"));
    }
    summary.distinct_referenced_evidence = evidence_pairs.len();
    rows.sort_by(|a, b| row_key(&a.locator).cmp(&row_key(&b.locator)));
    Ok(rows)
}

/// Construct only borrowed source/locator fields plus bounded generated UUIDs for one complete assertion.
fn row_base<'a>(
    manifest: &'a WorkflowManifest,
    item: &'a WorkItem,
    milestone: Option<&'a Milestone>,
    event: &'a Event,
    assertion: &'a EvidenceAssertion,
    as_of: NaiveDate,
) -> Result<EvidenceRow<'a>, ForgeError> {
    let asserted_after_as_of = DateTime::parse_from_rfc3339(&event.at)
        .map_err(|_| error("assertion timestamp is invalid"))?
        .with_timezone(&Utc)
        .date_naive()
        > as_of;
    Ok(EvidenceRow {
        locator: RowLocator {
            item_key: &item.key,
            milestone_key: milestone.map(|m| m.key.as_str()),
            event_key: &event.key,
            assertion_evidence_key: &assertion.key,
        },
        item_uuid: identity::item(&manifest.document.key, &item.key).to_string(),
        milestone_uuid: milestone
            .map(|m| identity::milestone(&manifest.document.key, &item.key, &m.key).to_string()),
        assertion_sha256: &assertion.expected_sha256,
        linkage_project_key: None,
        link_key: None,
        link_id: None,
        linkage_evidence_key: None,
        freshness: None,
        match_status: MatchStatus::Unbound,
        approved_sha256: None,
        observed_sha256: None,
        approved_size: None,
        observed_size: None,
        local_bytes_revalidated: false,
        asserted_after_as_of,
        source_refs: SourceRefs(&item.source_refs),
        recorded_valid_through: None,
    })
}

/// Resolve explicit link/evidence membership before invoking the sealed actual-local join.
fn join_row<'a>(
    row: &mut EvidenceRow<'a>,
    binding: &'a Binding,
    bindings: &'a EvidenceLinks,
    linkage: &'a PreparedFreshLinkage,
    links: &BTreeMap<&'a str, &'a LinkRecord>,
    evidence_records: &BTreeMap<&'a str, &'a EvidenceRecord>,
    assertion: &EvidenceAssertion,
) -> Result<(), ForgeError> {
    row.linkage_project_key = Some(&bindings.linkage.project_key);
    row.link_key = Some(&binding.link_key);
    row.linkage_evidence_key = Some(&binding.evidence_key);
    let link = links.get(binding.link_key.as_str()).copied();
    let evidence = evidence_records.get(binding.evidence_key.as_str()).copied();
    row.link_id = link.map(|link| link.link_id.as_str());
    let (Some(link), Some(evidence)) = (link, evidence) else {
        row.match_status = MatchStatus::Unavailable;
        return Ok(());
    };
    row.freshness = Some(&evidence.freshness);
    row.recorded_valid_through = evidence.valid_through.map(|date| date.to_string());
    if let EvidenceReference::Local {
        approved_sha256,
        approved_size,
        observed_sha256,
        observed_size,
        ..
    } = &evidence.reference
    {
        row.approved_sha256 = Some(approved_sha256);
        row.approved_size = Some(*approved_size);
        row.observed_sha256.clone_from(observed_sha256);
        row.observed_size = *observed_size;
    }
    if !link.evidence_keys.contains(&binding.evidence_key) {
        row.match_status = MatchStatus::WrongLinkMembership;
        return Ok(());
    }
    let observation = linkage
        .local_binding(
            &binding.evidence_key,
            Path::new(&assertion.href),
            &assertion.expected_sha256,
        )
        .map_err(|_| error("actual local evidence join is invalid or stale"))?;
    if observation.observed_sha256 != row.observed_sha256
        || observation.observed_size != row.observed_size
    {
        return Err(error("current evidence index and captured join observations differ"));
    }
    row.match_status = match observation.status {
        LocalBindingStatus::Matched => MatchStatus::Matched,
        LocalBindingStatus::ReferenceKindMismatch => MatchStatus::ReferenceKindMismatch,
        LocalBindingStatus::HrefMismatch => MatchStatus::HrefMismatch,
        LocalBindingStatus::HashMismatch => MatchStatus::HashMismatch,
        LocalBindingStatus::Unavailable => MatchStatus::Unavailable,
    };
    row.observed_sha256 = observation.observed_sha256;
    row.observed_size = observation.observed_size;
    row.local_bytes_revalidated = observation.local_bytes_revalidated;
    if row.match_status == MatchStatus::Matched
        && (!row.local_bytes_revalidated
            || row.approved_sha256 != Some(row.assertion_sha256)
            || row.observed_sha256.as_deref() != Some(row.assertion_sha256)
            || row.approved_size.is_none()
            || row.approved_size != row.observed_size)
    {
        return Err(error("sealed matched-local observation is inconsistent"));
    }
    if row.match_status == MatchStatus::Matched
        && !matches!(
            row.freshness,
            Some(
                EvidenceFreshness::Current
                    | EvidenceFreshness::Expiring
                    | EvidenceFreshness::Expired
            )
        )
    {
        return Err(error("sealed matched-local freshness is inconsistent"));
    }
    Ok(())
}

/// Borrow an unambiguous complete row sort key without key concatenation or normalization.
fn row_key<'a>(locator: &RowLocator<'a>) -> (&'a str, Option<&'a str>, &'a str, &'a str) {
    (locator.item_key, locator.milestone_key, locator.event_key, locator.assertion_evidence_key)
}

/// Keep status and freshness as separate complete partitions, never an inferred successful match.
fn count_row(row: &EvidenceRow<'_>, counts: &mut Counts) -> Result<(), ForgeError> {
    bounded_add(&mut counts.assertion_rows, 1)?;
    match row.match_status {
        MatchStatus::Unbound => bounded_add(&mut counts.unbound, 1)?,
        MatchStatus::Matched => bounded_add(&mut counts.matched, 1)?,
        MatchStatus::Unavailable => bounded_add(&mut counts.binding_unavailable, 1)?,
        MatchStatus::WrongLinkMembership
        | MatchStatus::ReferenceKindMismatch
        | MatchStatus::HrefMismatch
        | MatchStatus::HashMismatch => bounded_add(&mut counts.mismatched, 1)?,
    }
    match row.freshness {
        None => bounded_add(&mut counts.freshness_unmeasured, 1)?,
        Some(EvidenceFreshness::Current) => bounded_add(&mut counts.current, 1)?,
        Some(EvidenceFreshness::Expiring) => bounded_add(&mut counts.expiring, 1)?,
        Some(EvidenceFreshness::Expired) => bounded_add(&mut counts.expired, 1)?,
        Some(EvidenceFreshness::Changed) => bounded_add(&mut counts.changed, 1)?,
        Some(EvidenceFreshness::Unavailable) => bounded_add(&mut counts.unavailable, 1)?,
        Some(EvidenceFreshness::UnverifiedUri) => bounded_add(&mut counts.unverified_uri, 1)?,
    }
    if row.asserted_after_as_of {
        bounded_add(&mut counts.future_assertions, 1)?;
    }
    Ok(())
}

/// Require the complete status/freshness partitions and exact assertion-row denominator to agree.
fn reconcile(counts: &Counts, rows: usize) -> Result<(), ForgeError> {
    let statuses = [counts.matched, counts.unbound, counts.mismatched, counts.binding_unavailable]
        .into_iter()
        .try_fold(0_usize, usize::checked_add);
    let freshness = [
        counts.current,
        counts.expiring,
        counts.expired,
        counts.changed,
        counts.unavailable,
        counts.unverified_uri,
        counts.freshness_unmeasured,
    ]
    .into_iter()
    .try_fold(0_usize, usize::checked_add);
    if counts.closure_assertions != rows
        || counts.assertion_rows != rows
        || statuses != Some(rows)
        || freshness != Some(rows)
        || counts.future_assertions > rows
        || counts.distinct_referenced_evidence > counts.bindings_supplied
    {
        return Err(error("complete assertion count partitions differ"));
    }
    Ok(())
}

/// Checked whole-count addition without prefix success or per-item counter reset.
fn bounded_add(value: &mut usize, additional: usize) -> Result<(), ForgeError> {
    *value = value
        .checked_add(additional)
        .filter(|value| *value <= MAX_ROWS)
        .ok_or_else(|| error("complete metadata count exceeds its bound"))?;
    Ok(())
}

/// Validate exact keys under their own locator/linkage domains without truncation or normalization.
fn key(value: &str, maximum: usize) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > maximum
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(error("companion key is invalid or over-bound"));
    }
    Ok(())
}

/// Require a portable normalized local descendant spelling; no URI/query/fragment support is added.
fn relative(path: &Path) -> Result<(), ForgeError> {
    let value = path.to_str().ok_or_else(|| error("local path is not UTF-8"))?;
    if value.is_empty()
        || value.len() > MAX_PATH_BYTES
        || value.contains(['\\', ':', '?', '#'])
        || value.chars().any(char::is_control)
        || value.split('/').any(|part| {
            part.is_empty() || part == "." || part == ".." || part.ends_with(['.', ' '])
        })
        || path.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(error("local path is not a normalized confined descendant"));
    }
    Ok(())
}

/// Resolve only an explicit absolute input under the actual original qualified plan root.
fn descendant(root: &Path, input: &Path) -> Result<PathBuf, ForgeError> {
    let relative_path =
        input.strip_prefix(root).map_err(|_| error("input is outside the qualified plan root"))?;
    relative(relative_path)?;
    Ok(relative_path.to_path_buf())
}

/// Apply only the existing canonical ten-byte query-date boundary, never to recorded evidence dates.
fn full_date(value: &str) -> Result<NaiveDate, ForgeError> {
    if value.len() != 10 {
        return Err(error("inspection date is not canonical YYYY-MM-DD"));
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| error("inspection date is invalid"))?;
    if date.to_string() != value {
        return Err(error("inspection date is not canonical YYYY-MM-DD"));
    }
    Ok(date)
}

/// Emit a fixed redacted S4 category, never arbitrary path/prose/internal exception text.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(format!("S4 evidence inspection: {reason}"))
}

/// Synthetic syntax/partition/serialization controls; actual five-source/evidence admission is separate.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// Build closed companion syntax only; this fixture is not a current captured linkage proof.
    fn companion() -> Value {
        json!({"schema_version":"forge.poam-evidence-links/1","plan_key":"plan",
            "linkage":{"artifact":"nested/linkage.json","expected_sha256":"a".repeat(64),"project_key":"project"},
            "bindings":[{"locator":{"item_key":"work","milestone_key":null,"event_key":"completed","assertion_evidence_key":"proof"},
                "link_key":"link","evidence_key":"evidence"}]})
    }

    /// Construct syntactic artifact pins without qualifying their native source bytes.
    fn artifact(name: &str) -> Value {
        json!({"artifact":format!("{name}.json"),"href":format!("{name}.json"),
            "expected_sha256":"a".repeat(64),"root_uuid":"11111111-1111-4111-8111-111111111111",
            "document_version":"1.0.0","oscal_version":"1.2.3"})
    }

    /// Build a positive current structural profile with one item and milestone, never a native source proof.
    fn declaration() -> Value {
        let owner =
            json!({"role_id":"owner","party_key":"alice","rationale":"PRIVATE OWNER RATIONALE"});
        let initial = json!({"key":"initial","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-01T00:00:00Z","from":null,"to":"planned","rationale":"PRIVATE EVENT TEXT","closure":null});
        json!({"schema_version":"forge.poam/1","document":{"key":"plan","title":"PRIVATE PLAN TITLE","version":"1.0.0","last_modified":"2026-02-20T00:00:00Z"},
            "source":{"assessment_results":artifact("ar"),"result":{"uuid":"11111111-1111-4111-8111-111111111111","key":"result"},
                "context":{"assessment_plan":artifact("ap"),"ssp":artifact("ssp"),"profile":artifact("profile"),"catalog":artifact("catalog")}},
            "roles":[{"id":"owner","title":"Owner"},{"id":"reviewer","title":"Reviewer"}],
            "parties":[{"key":"alice","type":"person","name":"PRIVATE ALICE"},{"key":"bob","type":"person","name":"PRIVATE BOB"}],
            "items":[{"key":"work","title":"PRIVATE WORK","description":"PRIVATE DESCRIPTION",
                "source_refs":[{"kind":"finding","key":"finding","uuid":"22222222-2222-4222-8222-222222222222","result_uuid":"11111111-1111-4111-8111-111111111111","expected_sha256":"b".repeat(64)}],
                "owners":[owner.clone()],"target_date":"2026-02-01","state":"planned","history":[initial.clone()],
                "milestones":[{"key":"step","outcome":"PRIVATE OUTCOME","target_date":"2026-01-20","depends_on":[],"owners":[owner],"state":"planned","history":[initial]}]}]})
    }

    /// Append a coherent in-progress/completed history plus distinct declared reviewer, without source/evidence IO.
    fn complete(record: &mut Value) {
        let history = record["history"].as_array_mut().unwrap();
        history.push(json!({"key":"start","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-02T00:00:00Z","from":"planned","to":"in-progress","rationale":"PRIVATE START","closure":null}));
        history.push(json!({"key":"completed","actor":{"role_id":"owner","party_key":"alice"},
            "at":"2026-01-10T11:00:00Z","from":"in-progress","to":"completed-asserted","rationale":"PRIVATE COMPLETION",
            "closure":{"reviewer":{"role_id":"reviewer","party_key":"bob"},"reviewed_at":"2026-01-10T12:00:00Z",
                "rationale":"PRIVATE REVIEW","evidence":[{"key":"proof","href":"evidence/proof.bin","expected_sha256":"c".repeat(64)}]}}));
        record["state"] = json!("completed-asserted");
    }

    /// Complete a milestone review before its item's completion event, preserving the existing shape contract.
    fn complete_milestone(record: &mut Value) {
        complete(record);
        let event = record["history"].as_array_mut().unwrap().last_mut().unwrap();
        event["at"] = json!("2026-01-10T10:00:00Z");
        event["closure"]["reviewed_at"] = json!("2026-01-10T10:30:00Z");
    }

    /// Obtain an actual qualified empty capture session only for shared counter tests, not full source admission.
    fn counter_session(directory: &tempfile::TempDir) -> CaptureSession {
        CaptureSession::new(&directory.path().canonicalize().unwrap()).unwrap()
    }

    /// Positive companion syntax precedes malformed/duplicate/nullable/domain refusals; no current proof is inferred.
    #[test]
    fn closed_companion_requires_exact_nullable_locators_and_domains() {
        let positive = companion();
        let parsed = parse_bindings(&serde_json::to_vec(&positive).unwrap()).unwrap();
        assert_eq!(parsed.linkage.artifact, Path::new("nested/linkage.json"));
        assert_eq!(parsed.bindings.len(), 1);
        assert!(parsed.bindings[0].locator.milestone_key.is_none());
        for pointer in ["", "/linkage", "/bindings/0", "/bindings/0/locator"] {
            let mut invalid = positive.clone();
            invalid
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".to_string(), json!("PRIVATE"));
            assert!(parse_bindings(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
        let mut missing = positive.clone();
        missing["bindings"][0]["locator"].as_object_mut().unwrap().remove("milestone_key");
        assert!(parse_bindings(&serde_json::to_vec(&missing).unwrap()).is_err());
        let mut duplicate = positive.clone();
        duplicate["bindings"].as_array_mut().unwrap().push(positive["bindings"][0].clone());
        assert!(parse_bindings(&serde_json::to_vec(&duplicate).unwrap()).is_err());
        for path in [
            "../linkage.json",
            "/absolute.json",
            "https://example.invalid",
            "a\\b.json",
            "a.json?private",
            "a.json#fragment",
        ] {
            let mut invalid = positive.clone();
            invalid["linkage"]["artifact"] = json!(path);
            assert!(parse_bindings(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
        let mut limit = positive.clone();
        limit["plan_key"] = json!("p".repeat(256));
        limit["linkage"]["project_key"] = json!("q".repeat(MAX_LINKAGE_KEY_BYTES));
        assert!(parse_bindings(&serde_json::to_vec(&limit).unwrap()).is_ok());
        limit["plan_key"] = json!("p".repeat(257));
        assert!(parse_bindings(&serde_json::to_vec(&limit).unwrap()).is_err());
        assert!(parse_bindings(br#"{"schema_version":"x","schema_version":"x"}"#).is_err());
    }

    /// Inspect every item/milestone closure and future row after a valid positive shape, while public terminal refusal remains.
    #[test]
    fn complete_future_closure_denominators_keep_item_and_milestone_rows() {
        let mut value = declaration();
        assert!(super::super::workflow::parse(&serde_json::to_vec(&value).unwrap()).is_ok());
        complete_milestone(&mut value["items"][0]["milestones"][0]);
        complete(&mut value["items"][0]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let plan = workflow_baseline::parse_declaration(&bytes).unwrap();
        let refused = super::super::workflow::parse(&bytes).unwrap_err();
        assert!(refused.to_string().contains("pending recorded closure disposition"));
        let directory = tempfile::tempdir().unwrap();
        let mut capture = counter_session(&directory);
        let mut counts = declaration_counts(&plan, 0, &mut capture).unwrap();
        assert_eq!(
            (counts.items, counts.milestones, counts.history_events, counts.closure_assertions),
            (1, 1, 6, 2)
        );
        let item = &plan.items[0];
        let step = &item.milestones[0];
        let date = full_date("2026-01-09").unwrap();
        let event = item.history.last().unwrap();
        let row =
            row_base(&plan, item, None, event, &event.closure.as_ref().unwrap().evidence[0], date)
                .unwrap();
        let step_event = step.history.last().unwrap();
        let step_row = row_base(
            &plan,
            item,
            Some(step),
            step_event,
            &step_event.closure.as_ref().unwrap().evidence[0],
            date,
        )
        .unwrap();
        assert!(row.asserted_after_as_of && step_row.asserted_after_as_of);
        assert!(row.locator.milestone_key.is_none());
        assert_eq!(step_row.locator.milestone_key, Some("step"));
        assert_ne!(row.item_uuid, step_row.milestone_uuid.as_ref().unwrap().as_str());
        count_row(&row, &mut counts).unwrap();
        count_row(&step_row, &mut counts).unwrap();
        reconcile(&counts, 2).unwrap();
        assert_eq!(
            (counts.unbound, counts.freshness_unmeasured, counts.future_assertions),
            (2, 2, 2)
        );
        let rendered = serde_json::to_string(&step_row).unwrap();
        for private in ["PRIVATE", "evidence/proof.bin", "alice", "bob", "reviewer"] {
            assert!(!rendered.contains(private));
        }
    }

    /// Large admitted source keys remain exact, while mismatched/null/current partitions cannot counterfeit success counts.
    #[test]
    fn source_key_domain_and_independent_complete_partitions_are_preserved() {
        let mut value = declaration();
        value["source"]["result"]["key"] = json!("r".repeat(manifest::MAX_STRING_BYTES));
        value["items"][0]["source_refs"][0]["key"] = json!("s".repeat(manifest::MAX_STRING_BYTES));
        complete_milestone(&mut value["items"][0]["milestones"][0]);
        complete(&mut value["items"][0]);
        let plan =
            workflow_baseline::parse_declaration(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(plan.source.result.key.len(), manifest::MAX_STRING_BYTES);
        let item = &plan.items[0];
        let event = item.history.last().unwrap();
        let mut row = row_base(
            &plan,
            item,
            None,
            event,
            &event.closure.as_ref().unwrap().evidence[0],
            full_date("2026-01-10").unwrap(),
        )
        .unwrap();
        let encoded = serde_json::to_value(&row).unwrap();
        assert_eq!(
            encoded["source_refs"][0]["key"].as_str().unwrap(),
            "s".repeat(manifest::MAX_STRING_BYTES)
        );
        row.match_status = MatchStatus::HashMismatch;
        row.freshness = Some(&EvidenceFreshness::Current);
        let mut counts = Counts { closure_assertions: 1, ..Counts::default() };
        count_row(&row, &mut counts).unwrap();
        reconcile(&counts, 1).unwrap();
        assert_eq!((counts.mismatched, counts.current, counts.matched), (1, 1, 0));
        counts.matched = 1;
        assert!(reconcile(&counts, 1).is_err());
        let mut ceiling = MAX_ROWS;
        assert!(bounded_add(&mut ceiling, 1).is_err());
        assert_eq!(ceiling, MAX_ROWS);
    }

    /// Positive bounded rows precede escaped repeated-source metadata overflow; each refusal occurs before row retention.
    #[test]
    fn borrowed_row_escaping_is_charged_before_complete_vector_growth() {
        let mut value = declaration();
        let references: Vec<_> = (0..30)
            .map(|index| {
                let mut reference = value["items"][0]["source_refs"][0].clone();
                reference["key"] =
                    json!(format!("{index:02}{}{}", "s".repeat(40_000), "\0".repeat(10_000)));
                reference["uuid"] =
                    json!(identity::item("syntax-only", &index.to_string()).to_string());
                reference
            })
            .collect();
        value["items"][0]["source_refs"] = json!(references);
        complete_milestone(&mut value["items"][0]["milestones"][0]);
        complete(&mut value["items"][0]);
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(bytes.len() as u64 <= manifest::MAX_MANIFEST_BYTES);
        let plan = workflow_baseline::parse_declaration(&bytes).unwrap();
        let item = &plan.items[0];
        let event = item.history.last().unwrap();
        let row = row_base(
            &plan,
            item,
            None,
            event,
            &event.closure.as_ref().unwrap().evidence[0],
            full_date("2026-01-10").unwrap(),
        )
        .unwrap();
        let mut projection = ProjectionBudget::new();
        let mut admitted = 0;
        loop {
            match projection.admit_row(&row) {
                Ok(()) => admitted += 1,
                Err(_) => break,
            }
            assert!(admitted < 10);
        }
        assert!(admitted > 0 && admitted < 10);
        assert_eq!(row.source_refs.0.len(), 30);
        // This is a syntactic DTO/counting-writer control. No detached native
        // source/evidence proof or PreparedEvidence is fabricated here.
    }
}

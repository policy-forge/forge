//! Complete, minimized epoch-report syntax and deterministic bounded presentation.
//!
//! Prior-report admission is duplicate-safe, closed, nullable-required and bounded
//! before a second typed representation grows. The producer must first charge its
//! complete relationship ledger and derived J+V ceiling; this module does not own
//! native/source freshness, chronology, actors, family joins or count consistency.
//! Generic counters accept the producer's borrowed prospective Serialize mirror.
//! Renderers expose data only, never a captured-source or publication authority.

use std::io::{self, Write};
use std::sync::OnceLock;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use super::manifest::AssessmentMethod;
use crate::ForgeError;
use crate::json_strict::{self, Limits};
use crate::poam::manifest::{SourceKind, SourceReference};

/// Closed durable continuity companion protocol, independent of stdout view selection.
pub(crate) const SCHEMA_VERSION: &str = "forge.assessment-epoch-report/1";
/// Finite same-context, sealed-time-window inspection profile; no wider epoch admission.
pub(crate) const VALIDATION_SCOPE: &str = "same-context-sealed-epoch-append";
/// Continuity is an explicit caller assertion, never authenticated family authority.
pub(crate) const CONTINUITY_AUTHORITY: &str = "caller-asserted";
/// Fixed minimized trust wording; no source prose, actor authentication or remediation approval.
pub(crate) const TRUST_BOUNDARY: &str = "Caller-authored assessment and continuity assertions; identity and structure checks do not authenticate actors, evaluate evidence, or authorize remediation.";
/// Existing complete per-format encoded and prior-report raw byte ceiling.
pub(crate) const MAX_REPORT_BYTES: usize = 10 * 1024 * 1024;
/// Existing-dependency offline schema cache; all packaged references are local, no evidence fetch.
static VALIDATOR: OnceLock<Result<jsonschema::Validator, &'static str>> = OnceLock::new();
/// Existing decoded string UTF8 byte bound, separate from aggregate heap claims.
const MAX_STRING_BYTES: usize = 64 * 1024;
/// Complete prior/final epoch row roster bound, including unselected history.
const MAX_EPOCHS: usize = 1_000;
/// Complete all-kind object, family and continuity-edge bound.
const MAX_OBJECTS: usize = 10_000;
/// Both reciprocal directions are retained for every supplied continuity edge.
const MAX_RECIPROCALS: usize = 20_000;
/// Complete change and native graph occurrence ceiling before typed retention.
const MAX_RELATIONSHIPS: usize = 100_000;
/// Human-readable static text framing; dynamic contents are a complete pretty JSON dump.
const TEXT_PREFIX: &[u8] = b"Assessment epoch report\n\n";
/// Complete deterministic text framing, counted in V before any buffer growth.
const TEXT_SUFFIX: &[u8] = b"\n";
/// Static HTML framing contains no active links, scripts or external resource requests.
const HTML_PREFIX: &[u8] = b"<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Assessment epoch report</title></head><body><h1>Assessment epoch report</h1><pre>";
/// Static HTML closing markup and LF are part of the complete view ceiling.
const HTML_SUFFIX: &[u8] = b"</pre></body></html>\n";

/// Selected optional stdout presentation, never a durable-companion schema field.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EpochViewFormat {
    /// The exact already encoded required JSON companion, including its final LF.
    Json,
    /// Complete pretty JSON field dump with visible control-character escaping.
    Text,
    /// Complete escaped pretty JSON field dump inside fixed inert HTML markup.
    Html,
}

/// Report review state records presentation policy only, never an assessment verdict.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReportStatus {
    /// A syntactically complete report with no review action asserted here.
    Complete,
    /// Caller/producer review actions are required; the producer reconciles the actual roster.
    ReviewRequired,
}

/// Separate all-conclusion domain; the existing `SourceKind` intentionally excludes observations.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ConclusionKind {
    /// An assessment observation preserved or appended as an original graph node.
    Observation,
    /// An assessment finding preserved or appended as an original graph node.
    Finding,
    /// An assessment risk preserved or appended; no terminal authority is transferred.
    Risk,
}

/// Exact current four-companion model kinds; their source freshness is producer-owned.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ContextKind {
    /// Current Assessment Plan companion.
    AssessmentPlan,
    /// Current System Security Plan companion.
    SystemSecurityPlan,
    /// Current resolved Profile companion.
    Profile,
    /// Current resolved Catalog companion.
    Catalog,
}

/// Classification of one complete epoch in the after document.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum EpochClassification {
    /// An original history epoch, without inferred source-generation freshness.
    Preserved,
    /// The exactly one newly authored result epoch.
    Appended,
}

/// Classification of one full conclusion object tuple.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ObjectClassification {
    /// An original object whose Value preservation the producer independently checks.
    Preserved,
    /// An explicitly authored new conclusion object.
    Added,
}

/// One of the two explicitly paired reciprocal continuity trace directions.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReciprocalDirection {
    /// Trace from the earlier native risk tuple to its supplied successor.
    Predecessor,
    /// Trace back from the successor to the explicitly supplied predecessor.
    Successor,
}

/// Fixed descriptive change codes, without a closure, deletion or approval action.
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ChangeCode {
    /// Exactly one caller-authored epoch was appended.
    EpochAdded,
    /// A caller-authored observation, finding or risk was added.
    ObjectAdded,
    /// The explicitly supplied document version changed.
    DocumentVersionChanged,
    /// The explicitly supplied metadata modification time changed.
    DocumentModifiedTimeChanged,
    /// A caller-authored risk family identifier was introduced.
    RiskFamilyCreated,
    /// A supplied risk-continuity relation was recorded descriptively.
    RiskContinuityRecorded,
    /// Source-declared content hashes differ across an explicitly paired recurrence.
    RecurrenceContentChanged,
    /// Source-declared rationale hashes differ across an explicitly paired recurrence.
    RecurrenceRationaleChanged,
    /// Original native status strings differ across an explicitly paired recurrence.
    RecurrenceStatusChanged,
}

/// Full native risk reference reuses the current tuple; runtime admission requires kind Risk.
pub(crate) type RiskReference = SourceReference;

/// Closed complete report DTO; construction alone confers no source or publication authority.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct EpochReport {
    /// Exact durable companion protocol constant.
    pub(crate) schema_version: String,
    /// Exact same-context sealed-epoch profile constant.
    pub(crate) validation_scope: String,
    /// Finite report review-state label, reconciled by the producer.
    pub(crate) status: ReportStatus,
    /// Explicit review-action boolean; actual change consistency is producer-owned.
    pub(crate) review_required: bool,
    /// Must remain false; no assessment or remediation authority is granted.
    pub(crate) assessment_authority: bool,
    /// Exact caller-asserted continuity qualification constant.
    pub(crate) continuity_authority: String,
    /// Actual prior native document UUID and separately computed original raw digest.
    pub(crate) prior_native: PriorNativeIdentity,
    /// New native file identity and computed output raw digest.
    pub(crate) native: NativeIdentity,
    /// Complete four current companion rows, not an inferred unique-source union.
    pub(crate) context: Vec<ContextRow>,
    /// All complete before/after and occurrence denominators.
    pub(crate) counts: Counts,
    /// Every retained/appended epoch, preserving actual source order.
    pub(crate) epochs: Vec<EpochRow>,
    /// Every observation/finding/risk tuple from every result.
    pub(crate) objects: Vec<ObjectRow>,
    /// All supplied risk families, including uncontinued families.
    pub(crate) families: Vec<FamilyRow>,
    /// Complete explicit directed recurrence edge roster.
    pub(crate) continuity_edges: Vec<ContinuityEdge>,
    /// Both independently checked trace directions for every edge.
    pub(crate) reciprocal_rows: Vec<ReciprocalRow>,
    /// Complete descriptive changes; no prefix or clipped subset.
    pub(crate) changes: Vec<ChangeRow>,
    /// Fixed minimized trust wording rather than caller/native rationale prose.
    pub(crate) trust_boundary: String,
}

/// Complete counted denominators; all scalars are required integers, not estimates.
#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Counts {
    /// Number of actual original prior result epochs.
    pub(crate) before_epochs: usize,
    /// Number of complete after result epochs.
    pub(crate) after_epochs: usize,
    /// Number of source epochs asserted preserved by the producer oracle.
    pub(crate) preserved_epochs: usize,
    /// Number of explicitly appended result epochs.
    pub(crate) appended_epochs: usize,
    /// Prior observation object count across all results.
    pub(crate) before_observations: usize,
    /// After observation object count across all results.
    pub(crate) after_observations: usize,
    /// Prior finding object count across all results.
    pub(crate) before_findings: usize,
    /// After finding object count across all results.
    pub(crate) after_findings: usize,
    /// Prior risk object count across all results.
    pub(crate) before_risks: usize,
    /// After risk object count across all results.
    pub(crate) after_risks: usize,
    /// Complete prior all-kind object count.
    pub(crate) before_objects: usize,
    /// Complete after all-kind object count.
    pub(crate) after_objects: usize,
    /// Object rows asserted preserved by the producer oracle.
    pub(crate) preserved_objects: usize,
    /// Explicit new object rows.
    pub(crate) added_objects: usize,
    /// All prior native graph reference occurrences, including repeats.
    pub(crate) native_graph_occurrences_before: usize,
    /// All after native graph reference occurrences, including repeats.
    pub(crate) native_graph_occurrences_after: usize,
    /// Complete supplied family count.
    pub(crate) families: usize,
    /// Complete directed continuity edge count.
    pub(crate) continuity_edges: usize,
    /// Complete reciprocal trace row count, without collapsing directions.
    pub(crate) reciprocal_rows: usize,
    /// Number of new native risks explicitly classified by the caller.
    pub(crate) classified_next_risks: usize,
    /// Families not continued in this append; no closure is implied.
    pub(crate) uncontinued_families: usize,
    /// Actual distinct held original generations; the producer reconciles six or seven.
    pub(crate) captured_original_generations: usize,
    /// Complete descriptive review action count.
    pub(crate) review_actions: usize,
}

/// Prior native file identity keeps actual raw bytes separate from declared object hashes.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PriorNativeIdentity {
    /// Canonical original document UUID.
    pub(crate) uuid: String,
    /// Computed raw digest of the actually held original native file.
    pub(crate) raw_sha256: String,
}

/// New native output identity; this row is derived data, not a fresh original-file proof.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeIdentity {
    /// Actual new portable single JSON filename under the qualified root.
    pub(crate) artifact: String,
    /// Canonical preserved document UUID.
    pub(crate) uuid: String,
    /// Explicit caller-authored native document version, preserved exactly.
    pub(crate) document_version: String,
    /// Preserved native OSCAL version string.
    pub(crate) oscal_version: String,
    /// Computed raw digest of the complete encoded new native file.
    pub(crate) raw_sha256: String,
}

/// One complete actual-context pin; the producer owns its native schema and freshness join.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContextRow {
    /// Finite actual companion model kind.
    pub(crate) kind: ContextKind,
    /// Original raw SHA256, separately computed from the held file.
    pub(crate) sha256: String,
    /// Canonical original companion root UUID.
    pub(crate) root_uuid: String,
    /// Exact declared native document version.
    pub(crate) document_version: String,
    /// Exact declared OSCAL version.
    pub(crate) oscal_version: String,
}

/// One epoch row preserves full native result identity and source time spelling.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct EpochRow {
    /// Actual zero-based result array position; no implicit latest selection.
    pub(crate) position: usize,
    /// Exact original/new result key, up to the current 64KiB native key bound.
    pub(crate) key: String,
    /// Canonical native result UUID.
    pub(crate) uuid: String,
    /// Computed canonical whole-result digest, not a source-declared property digest.
    pub(crate) canonical_sha256: String,
    /// Exact caller/source start timestamp; chronology is producer-owned.
    pub(crate) start: String,
    /// Explicit sealed result end timestamp, never synthesized or null.
    pub(crate) end: String,
    /// Complete result observation denominator.
    pub(crate) observations: usize,
    /// Complete result finding denominator.
    pub(crate) findings: usize,
    /// Complete result risk denominator.
    pub(crate) risks: usize,
    /// Explicit preserved/appended row classification.
    pub(crate) classification: EpochClassification,
}

/// One complete source conclusion tuple; source-declared hashes remain separate facts.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObjectRow {
    /// Exact containing result key.
    pub(crate) result_key: String,
    /// Canonical containing result UUID.
    pub(crate) result_uuid: String,
    /// All-conclusion kind, including observations outside existing `SourceKind`.
    pub(crate) kind: ConclusionKind,
    /// Exact original native object key.
    pub(crate) key: String,
    /// Canonical original/new native object UUID.
    pub(crate) uuid: String,
    /// Computed canonical whole-object digest.
    pub(crate) computed_sha256: String,
    /// Exact source-declared content digest, not a computed freshness substitute.
    pub(crate) declared_content_sha256: String,
    /// Exact source-declared rationale digest, not borrowed rationale prose.
    pub(crate) declared_rationale_sha256: String,
    /// Required nullable exact source status; absence never infers closure.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) status: Option<String>,
    /// Explicit preserved/added classification.
    pub(crate) classification: ObjectClassification,
}

/// Complete caller-authored family, including explicit uncontinued current heads.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct FamilyRow {
    /// Exact caller family key within 256 UTF8 bytes, without normalization.
    pub(crate) family_key: String,
    /// Actual first native risk tuple; runtime admission requires kind Risk.
    pub(crate) first: RiskReference,
    /// Actual current native family head tuple; runtime admission requires kind Risk.
    pub(crate) current: RiskReference,
    /// Explicit continuation assertion for this append only.
    pub(crate) continued_in_append: bool,
}

/// Full explicit recurrence edge; this module does not authenticate its family meaning.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContinuityEdge {
    /// Exact supplied family identifier.
    pub(crate) family_key: String,
    /// Actual complete earlier native risk endpoint.
    pub(crate) predecessor: RiskReference,
    /// Actual complete later native risk endpoint.
    pub(crate) successor: RiskReference,
    /// Minimized supplied provenance with rationale digest only.
    pub(crate) provenance: ReportProvenance,
    /// Must be true; continuity remains caller asserted, not independently authenticated.
    pub(crate) caller_asserted_continuity: bool,
    /// Computed framed relation digest; no signature/actor authority is implied.
    pub(crate) edge_sha256: String,
}

/// Minimized continuity provenance retains exact supplied IDs/time/method and rationale digest.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReportProvenance {
    /// Supplied assessor party key; actual membership is producer-owned.
    pub(crate) assessor_key: String,
    /// Supplied role identifier; actual membership is producer-owned.
    pub(crate) role_id: String,
    /// Supplied exact RFC3339 start spelling.
    pub(crate) start: String,
    /// Required nullable end assertion; no implicit clock or filled date.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) end: Option<String>,
    /// Existing explicit assessment method domain, including UNKNOWN.
    pub(crate) method: AssessmentMethod,
    /// Digest of caller rationale, never the rationale prose itself.
    pub(crate) rationale_sha256: String,
}

/// One of two explicit complete relation traces, never collapsed into a unique edge union.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReciprocalRow {
    /// Exact family identifier.
    pub(crate) family_key: String,
    /// Same computed relation digest as the corresponding edge.
    pub(crate) edge_sha256: String,
    /// Finite earlier/later trace direction.
    pub(crate) direction: ReciprocalDirection,
    /// Complete source risk endpoint.
    pub(crate) from: RiskReference,
    /// Complete destination risk endpoint.
    pub(crate) to: RiskReference,
}

/// Full all-kind source locator; observations are preserved rather than coerced into risks.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObjectLocator {
    /// Exact containing result key.
    pub(crate) result_key: String,
    /// Canonical containing result UUID.
    pub(crate) result_uuid: String,
    /// Exact all-conclusion domain.
    pub(crate) kind: ConclusionKind,
    /// Exact native object key.
    pub(crate) key: String,
    /// Canonical native object UUID.
    pub(crate) uuid: String,
}

/// Complete descriptive change with required nullable fields; no inferred actor/work authority.
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChangeRow {
    /// Fixed descriptive change code.
    pub(crate) code: ChangeCode,
    /// Required nullable full native conclusion locator.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) locator: Option<ObjectLocator>,
    /// Required nullable exact supplied family key.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) family_key: Option<String>,
    /// Required nullable original scalar; source prose is not a permitted producer projection.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) old: Option<String>,
    /// Required nullable new scalar, preserved rather than normalized.
    #[serde(deserialize_with = "required_nullable")]
    pub(crate) new: Option<String>,
}

/// Deserialize an explicitly present null/value; absence is not a serde Option default.
fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// Strict-parse one held prior companion and validate every borrowed row before typed growth.
///
/// The complete original Value has separately bounded raw/depth/string decoding;
/// this is not a parser allocation bound. The caller must precharge every report
/// relationship and complete derived projection before invoking decode below.
pub(crate) fn parse_raw(bytes: &[u8]) -> Result<Value, ForgeError> {
    if bytes.len() > MAX_REPORT_BYTES {
        return Err(error("epoch report exceeds ten MiB"));
    }
    let value = json_strict::parse_value(
        bytes,
        "epoch report",
        Limits { max_depth: 128, max_string_bytes: MAX_STRING_BYTES },
    )
    .map_err(|_| error("epoch report is not bounded duplicate-free JSON"))?;
    validate_value(&value)?;
    Ok(value)
}

/// Typed-decode a fully shape-admitted borrowed Value after producer ledger/J+V precharge.
///
/// Shape admission is repeated rather than trusting a detached caller token. Native
/// joins, family/edge/count/context consistency and authority are NOT established.
pub(crate) fn decode(value: &Value) -> Result<EpochReport, ForgeError> {
    validate_value(value)?;
    EpochReport::deserialize(value).map_err(|_| error("epoch report typed shape is invalid"))
}

/// Validate all closed required fields and unselected array ceilings without typed rows or clones.
pub(crate) fn validate_value(value: &Value) -> Result<(), ForgeError> {
    let root = closed(value, ROOT_FIELDS)?;
    exact(&root["schema_version"], SCHEMA_VERSION)?;
    exact(&root["validation_scope"], VALIDATION_SCOPE)?;
    choice(&root["status"], &["complete", "review-required"])?;
    boolean(&root["review_required"])?;
    exact_bool(&root["assessment_authority"], false)?;
    exact(&root["continuity_authority"], CONTINUITY_AUTHORITY)?;
    exact(&root["trust_boundary"], TRUST_BOUNDARY)?;
    // Every complete roster is bounded before any row validator or typed decode.
    array(&root["context"], 4, 4)?;
    array(&root["epochs"], 2, MAX_EPOCHS)?;
    array(&root["objects"], 0, MAX_OBJECTS)?;
    array(&root["families"], 0, MAX_OBJECTS)?;
    array(&root["continuity_edges"], 0, MAX_OBJECTS)?;
    array(&root["reciprocal_rows"], 0, MAX_RECIPROCALS)?;
    array(&root["changes"], 0, MAX_RELATIONSHIPS)?;
    validate_native(&root["prior_native"], false)?;
    validate_native(&root["native"], true)?;
    validate_counts(&root["counts"])?;
    validate_roster(&root["context"], validate_context)?;
    validate_roster(&root["epochs"], validate_epoch)?;
    validate_roster(&root["objects"], validate_object)?;
    validate_roster(&root["families"], validate_family)?;
    validate_roster(&root["continuity_edges"], validate_edge)?;
    validate_roster(&root["reciprocal_rows"], validate_reciprocal)?;
    validate_roster(&root["changes"], validate_change)?;
    validate_schema(value)
}

/// Consume the packaged closed offline schema; validation never authenticates source or assertions.
fn validate_schema(value: &Value) -> Result<(), ForgeError> {
    let validator = VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../schemas/forge.assessment-epoch-report-1.schema.json"
            ))
            .map_err(|_| "epoch report schema is malformed")?;
            jsonschema::validator_for(&schema).map_err(|_| "epoch report schema is unavailable")
        })
        .as_ref()
        .map_err(|reason| error(reason))?;
    if !validator.is_valid(value) {
        return Err(error("epoch report closed schema is invalid"));
    }
    Ok(())
}

/// Exact root field roster; unknown keys and missing required-nullable keys refuse.
const ROOT_FIELDS: &[&str] = &[
    "schema_version",
    "validation_scope",
    "status",
    "review_required",
    "assessment_authority",
    "continuity_authority",
    "prior_native",
    "native",
    "context",
    "counts",
    "epochs",
    "objects",
    "families",
    "continuity_edges",
    "reciprocal_rows",
    "changes",
    "trust_boundary",
];
/// Exact complete required denominator roster, unchanged from the closed field contract.
const COUNT_FIELDS: &[&str] = &[
    "before_epochs",
    "after_epochs",
    "preserved_epochs",
    "appended_epochs",
    "before_observations",
    "after_observations",
    "before_findings",
    "after_findings",
    "before_risks",
    "after_risks",
    "before_objects",
    "after_objects",
    "preserved_objects",
    "added_objects",
    "native_graph_occurrences_before",
    "native_graph_occurrences_after",
    "families",
    "continuity_edges",
    "reciprocal_rows",
    "classified_next_risks",
    "uncontinued_families",
    "captured_original_generations",
    "review_actions",
];
/// Finite descriptive changes without lifecycle/approval actions.
const CHANGE_CODES: &[&str] = &[
    "epoch-added",
    "object-added",
    "document-version-changed",
    "document-modified-time-changed",
    "risk-family-created",
    "risk-continuity-recorded",
    "recurrence-content-changed",
    "recurrence-rationale-changed",
    "recurrence-status-changed",
];

/// Borrow a complete closed map only after checking every required key and map length.
fn closed<'a>(value: &'a Value, fields: &[&str]) -> Result<&'a Map<String, Value>, ForgeError> {
    let map = value.as_object().ok_or_else(|| error("epoch report needs a closed object"))?;
    if map.len() != fields.len() || fields.iter().any(|field| !map.contains_key(*field)) {
        return Err(error("epoch report has missing or unknown fields"));
    }
    Ok(map)
}

/// Bound a complete roster before its elements can be typed or retained in derived structures.
fn array(value: &Value, minimum: usize, maximum: usize) -> Result<&[Value], ForgeError> {
    let rows = value.as_array().ok_or_else(|| error("epoch report needs an array"))?;
    if rows.len() < minimum || rows.len() > maximum {
        return Err(error("epoch report complete array exceeds profile bounds"));
    }
    Ok(rows)
}

/// Traverse already bounded row Values by borrowing; no sorting, map/set retention or cloning.
fn validate_roster(
    value: &Value,
    validate: fn(&Value) -> Result<(), ForgeError>,
) -> Result<(), ForgeError> {
    for row in value.as_array().ok_or_else(|| error("epoch report needs an array"))? {
        validate(row)?;
    }
    Ok(())
}

/// Require bounded UTF8 source strings without normalization or a new owned String.
fn string(value: &Value, maximum: usize, nonempty: bool) -> Result<&str, ForgeError> {
    let text = value.as_str().ok_or_else(|| error("epoch report needs a string"))?;
    check_text(text, maximum, nonempty)?;
    Ok(text)
}

/// Check a borrowed caller/source string by UTF8 bytes, preserving exact spelling.
fn check_text(text: &str, maximum: usize, nonempty: bool) -> Result<(), ForgeError> {
    if text.len() > maximum || (nonempty && text.trim().is_empty()) {
        return Err(error("epoch report string exceeds profile bounds"));
    }
    Ok(())
}

/// Preserve the new exact family-key domain: nonempty bytes, with no trimming or normalization.
fn family_key(value: &Value) -> Result<&str, ForgeError> {
    let text = value.as_str().ok_or_else(|| error("epoch report family key needs a string"))?;
    check_family_key(text)?;
    Ok(text)
}

/// Check the caller-defined 256-byte family key, including explicit whitespace-only keys.
fn check_family_key(text: &str) -> Result<(), ForgeError> {
    if text.is_empty() || text.len() > 256 {
        return Err(error("epoch report family key exceeds profile bounds"));
    }
    Ok(())
}

/// Admit a required nullable bounded string; null and missing remain different cases.
fn nullable_string(value: &Value, maximum: usize) -> Result<(), ForgeError> {
    if !value.is_null() {
        string(value, maximum, false)?;
    }
    Ok(())
}

/// Require the exact fixed protocol/privacy label without echoing caller bytes.
fn exact(value: &Value, expected: &str) -> Result<(), ForgeError> {
    if value.as_str() != Some(expected) {
        return Err(error("epoch report fixed label is invalid"));
    }
    Ok(())
}

/// Require one finite protocol label; preserved native status is intentionally not this domain.
fn choice(value: &Value, allowed: &[&str]) -> Result<(), ForgeError> {
    if !value.as_str().is_some_and(|text| allowed.contains(&text)) {
        return Err(error("epoch report finite label is invalid"));
    }
    Ok(())
}

/// Require an actual JSON boolean, never a nullable/string/numeric surrogate.
fn boolean(value: &Value) -> Result<bool, ForgeError> {
    value.as_bool().ok_or_else(|| error("epoch report needs a boolean"))
}

/// Require fixed false authority or fixed true caller-assertion markers.
fn exact_bool(value: &Value, expected: bool) -> Result<(), ForgeError> {
    if boolean(value)? != expected {
        return Err(error("epoch report authority marker is invalid"));
    }
    Ok(())
}

/// Bound unsigned count scalars before a native usize conversion or typed row allocation.
fn integer(value: &Value, maximum: usize) -> Result<(), ForgeError> {
    if value.as_u64().is_none_or(|number| number > maximum as u64) {
        return Err(error("epoch report integer exceeds profile bounds"));
    }
    Ok(())
}

/// Check canonical UUID spelling using bounded borrowed bytes, without allocating parsed strings.
fn uuid(value: &Value) -> Result<(), ForgeError> {
    check_uuid(string(value, 36, true)?)
}

/// Require exactly lowercase hexadecimal UUID groups; no simple/URN/braced alias admission.
fn check_uuid(text: &str) -> Result<(), ForgeError> {
    let valid = text.len() == 36
        && text.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
            }
        });
    if !valid {
        return Err(error("epoch report UUID spelling is invalid"));
    }
    Ok(())
}

/// Check a complete lowercase SHA256 string, without transferring any freshness authority.
fn digest(value: &Value) -> Result<(), ForgeError> {
    check_digest(string(value, 64, true)?)
}

/// Require a complete lowercase digest by borrowed bytes only.
fn check_digest(text: &str) -> Result<(), ForgeError> {
    if text.len() != 64
        || !text.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(error("epoch report digest spelling is invalid"));
    }
    Ok(())
}

/// Borrow and check explicitly supplied RFC3339 spelling; chronology and actor windows are external.
fn timestamp(value: &Value) -> Result<(), ForgeError> {
    check_timestamp(string(value, MAX_STRING_BYTES, true)?)
}

/// Parse an actual supplied instant without choosing a clock, timezone normalization or actor.
fn check_timestamp(text: &str) -> Result<(), ForgeError> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map(|_| ())
        .map_err(|_| error("epoch report timestamp is invalid"))
}

/// Match the packaged native/companion OSCAL version domain before typed view bytes grow.
fn check_oscal_version(text: &str, current_native: bool) -> Result<(), ForgeError> {
    let accepted = if current_native {
        text == "1.2.3"
    } else {
        matches!(text, "1.2.0" | "1.2.1" | "1.2.2" | "1.2.3")
    };
    if !accepted {
        return Err(error("epoch report OSCAL version is outside profile"));
    }
    Ok(())
}

/// Require the maintained portable single JSON filename profile, without filesystem IO.
fn artifact(value: &Value) -> Result<(), ForgeError> {
    let text = string(value, 128, true)?;
    crate::authoring::output::validate_relative(text)
        .map_err(|_| error("epoch report artifact filename is invalid"))?;
    if text.contains('/')
        || std::path::Path::new(text).extension() != Some(std::ffi::OsStr::new("json"))
    {
        return Err(error("epoch report artifact filename is invalid"));
    }
    Ok(())
}

/// Admit the exact prior/new native identity shapes, keeping raw and declared digests distinct.
fn validate_native(value: &Value, current: bool) -> Result<(), ForgeError> {
    let fields: &[&str] = if current {
        &["artifact", "uuid", "document_version", "oscal_version", "raw_sha256"]
    } else {
        &["uuid", "raw_sha256"]
    };
    let row = closed(value, fields)?;
    uuid(&row["uuid"])?;
    digest(&row["raw_sha256"])?;
    if current {
        artifact(&row["artifact"])?;
        string(&row["document_version"], MAX_STRING_BYTES, true)?;
        string(&row["oscal_version"], MAX_STRING_BYTES, true)?;
    }
    Ok(())
}

/// Bound every complete count domain; actual equalities and native roster reconciliation are external.
fn validate_counts(value: &Value) -> Result<(), ForgeError> {
    let row = closed(value, COUNT_FIELDS)?;
    for field in COUNT_FIELDS {
        integer(&row[*field], count_limit(field))?;
    }
    Ok(())
}

/// Exact scalar upper ceilings use the same roster domains as before-growth shape admission.
fn count_limit(field: &str) -> usize {
    match field {
        "before_epochs" | "after_epochs" | "preserved_epochs" => MAX_EPOCHS,
        "appended_epochs" => 1,
        "native_graph_occurrences_before" | "native_graph_occurrences_after" | "review_actions" => {
            MAX_RELATIONSHIPS
        }
        "reciprocal_rows" => MAX_RECIPROCALS,
        "captured_original_generations" => 7,
        _ => MAX_OBJECTS,
    }
}

/// Check one complete current-context record; exact unique four-kind membership is producer-owned.
fn validate_context(value: &Value) -> Result<(), ForgeError> {
    let row = closed(value, &["kind", "sha256", "root_uuid", "document_version", "oscal_version"])?;
    choice(&row["kind"], &["assessment-plan", "system-security-plan", "profile", "catalog"])?;
    digest(&row["sha256"])?;
    uuid(&row["root_uuid"])?;
    string(&row["document_version"], MAX_STRING_BYTES, true)?;
    string(&row["oscal_version"], MAX_STRING_BYTES, true)?;
    Ok(())
}

/// Check one bounded sealed-time epoch row without inferring graph/chronology consistency.
fn validate_epoch(value: &Value) -> Result<(), ForgeError> {
    let row = closed(
        value,
        &[
            "position",
            "key",
            "uuid",
            "canonical_sha256",
            "start",
            "end",
            "observations",
            "findings",
            "risks",
            "classification",
        ],
    )?;
    integer(&row["position"], MAX_EPOCHS - 1)?;
    string(&row["key"], MAX_STRING_BYTES, true)?;
    uuid(&row["uuid"])?;
    digest(&row["canonical_sha256"])?;
    timestamp(&row["start"])?;
    timestamp(&row["end"])?;
    for field in ["observations", "findings", "risks"] {
        integer(&row[field], MAX_OBJECTS)?;
    }
    choice(&row["classification"], &["preserved", "appended"])
}

/// Check full all-kind object rows; exact source status is nullable and not rewritten.
fn validate_object(value: &Value) -> Result<(), ForgeError> {
    let row = closed(
        value,
        &[
            "result_key",
            "result_uuid",
            "kind",
            "key",
            "uuid",
            "computed_sha256",
            "declared_content_sha256",
            "declared_rationale_sha256",
            "status",
            "classification",
        ],
    )?;
    validate_locator_fields(row)?;
    digest(&row["computed_sha256"])?;
    digest(&row["declared_content_sha256"])?;
    digest(&row["declared_rationale_sha256"])?;
    nullable_string(&row["status"], MAX_STRING_BYTES)?;
    choice(&row["classification"], &["preserved", "added"])
}

/// Check actual all-conclusion locator fields by borrowing the already closed map.
fn validate_locator_fields(row: &Map<String, Value>) -> Result<(), ForgeError> {
    string(&row["result_key"], MAX_STRING_BYTES, true)?;
    uuid(&row["result_uuid"])?;
    choice(&row["kind"], &["observation", "finding", "risk"])?;
    string(&row["key"], MAX_STRING_BYTES, true)?;
    uuid(&row["uuid"])
}

/// Require the existing complete `SourceReference` shape and the explicit risk domain only.
fn validate_risk(value: &Value) -> Result<(), ForgeError> {
    let row = closed(value, &["kind", "key", "uuid", "result_uuid", "expected_sha256"])?;
    exact(&row["kind"], "risk")?;
    string(&row["key"], MAX_STRING_BYTES, true)?;
    uuid(&row["uuid"])?;
    uuid(&row["result_uuid"])?;
    digest(&row["expected_sha256"])
}

/// Check full explicit family records, including uncontinued heads and actual risk kinds.
fn validate_family(value: &Value) -> Result<(), ForgeError> {
    let row = closed(value, &["family_key", "first", "current", "continued_in_append"])?;
    family_key(&row["family_key"])?;
    validate_risk(&row["first"])?;
    validate_risk(&row["current"])?;
    boolean(&row["continued_in_append"])?;
    Ok(())
}

/// Check minimized provenance; rationale is a required digest, not accepted prose.
fn validate_provenance(value: &Value) -> Result<(), ForgeError> {
    let row =
        closed(value, &["assessor_key", "role_id", "start", "end", "method", "rationale_sha256"])?;
    string(&row["assessor_key"], MAX_STRING_BYTES, true)?;
    string(&row["role_id"], MAX_STRING_BYTES, true)?;
    timestamp(&row["start"])?;
    if !row["end"].is_null() {
        timestamp(&row["end"])?;
    }
    choice(&row["method"], &["EXAMINE", "INTERVIEW", "TEST", "UNKNOWN"])?;
    digest(&row["rationale_sha256"])
}

/// Check complete closed caller-asserted edge syntax; endpoint/family/DAG semantics are external.
fn validate_edge(value: &Value) -> Result<(), ForgeError> {
    let row = closed(
        value,
        &[
            "family_key",
            "predecessor",
            "successor",
            "provenance",
            "caller_asserted_continuity",
            "edge_sha256",
        ],
    )?;
    family_key(&row["family_key"])?;
    validate_risk(&row["predecessor"])?;
    validate_risk(&row["successor"])?;
    validate_provenance(&row["provenance"])?;
    exact_bool(&row["caller_asserted_continuity"], true)?;
    digest(&row["edge_sha256"])
}

/// Check both complete reciprocal trace shapes, preserving supplied endpoints and direction.
fn validate_reciprocal(value: &Value) -> Result<(), ForgeError> {
    let row = closed(value, &["family_key", "edge_sha256", "direction", "from", "to"])?;
    family_key(&row["family_key"])?;
    digest(&row["edge_sha256"])?;
    choice(&row["direction"], &["predecessor", "successor"])?;
    validate_risk(&row["from"])?;
    validate_risk(&row["to"])
}

/// Check every required nullable change field before typed Option allocation.
fn validate_change(value: &Value) -> Result<(), ForgeError> {
    let row = closed(value, &["code", "locator", "family_key", "old", "new"])?;
    choice(&row["code"], CHANGE_CODES)?;
    if !row["locator"].is_null() {
        let locator =
            closed(&row["locator"], &["result_key", "result_uuid", "kind", "key", "uuid"])?;
        validate_locator_fields(locator)?;
    }
    if !row["family_key"].is_null() {
        family_key(&row["family_key"])?;
    }
    nullable_string(&row["old"], MAX_STRING_BYTES)?;
    nullable_string(&row["new"], MAX_STRING_BYTES)
}

/// Count the complete durable compact JSON plus final LF without retaining encoded bytes.
///
/// The generic input permits a borrowed prospective report mirror before typed row
/// growth. It is NOT a shape/native/source proof; the producer owns that admission.
pub(crate) fn count_json<T: Serialize + ?Sized>(report: &T) -> Result<usize, ForgeError> {
    let mut writer = CappedWriter::counting(MAX_REPORT_BYTES);
    render(report, EpochViewFormat::Json, &mut writer)?;
    Ok(writer.count)
}

/// Count the complete selected view using the identical serializer/escaping/framing as encoding.
///
/// None is zero and Json equals the durable count even when output storage is
/// borrowed. The producer must add J+V once with checked3*N before any row growth.
pub(crate) fn count_view<T: Serialize + ?Sized>(
    report: &T,
    view: Option<EpochViewFormat>,
) -> Result<usize, ForgeError> {
    let Some(format) = view else {
        return Ok(0);
    };
    let mut writer = CappedWriter::counting(MAX_REPORT_BYTES);
    render(report, format, &mut writer)?;
    Ok(writer.count)
}

/// Encode a complete borrowed DTO/mirror under its cap and consume actual strict/schema admission.
///
/// This does not replace the producer's BEFORE-growth J+V/native/relationship
/// accounting, final native joins or original proof rechecks before publication.
/// The temporary decoded output is separately bounded raw/depth/string metadata,
/// dropped before return; this function claims no aggregate heap/capacity bound.
pub(crate) fn encode_json<T: Serialize + ?Sized>(
    report: &T,
    admitted_ceiling: usize,
) -> Result<Vec<u8>, ForgeError> {
    check_ceiling(admitted_ceiling)?;
    let mut writer = CappedWriter::collecting(admitted_ceiling);
    render(report, EpochViewFormat::Json, &mut writer)?;
    let bytes =
        writer.bytes.ok_or_else(|| error("epoch report collecting writer is unavailable"))?;
    let admitted = parse_raw(&bytes)?;
    drop(admitted);
    Ok(bytes)
}

/// Encode the complete optional view; omitted view has no bytes or another filesystem target.
///
/// Json can alternatively borrow the held durable JSON slice in `PreparedEpochAppend`;
/// this convenience function still counts/encodes the same complete byte sequence.
pub(crate) fn encode_view(
    report: &EpochReport,
    view: Option<EpochViewFormat>,
    admitted_ceiling: usize,
) -> Result<Option<Vec<u8>>, ForgeError> {
    check_ceiling(admitted_ceiling)?;
    let Some(format) = view else {
        return Ok(None);
    };
    validate_typed(report)?;
    let mut writer = CappedWriter::collecting(admitted_ceiling);
    render(report, format, &mut writer)?;
    writer.bytes.map(Some).ok_or_else(|| error("epoch report collecting writer is unavailable"))
}

/// Require the actual precharged encoded ceiling before any collecting writer is created.
fn check_ceiling(ceiling: usize) -> Result<(), ForgeError> {
    if ceiling > MAX_REPORT_BYTES {
        return Err(error("epoch report admitted ceiling exceeds profile"));
    }
    Ok(())
}

/// Run the same complete borrowed serializer for counters and buffers; only requested formats run.
fn render<T: Serialize + ?Sized>(
    report: &T,
    format: EpochViewFormat,
    writer: &mut CappedWriter,
) -> Result<(), ForgeError> {
    if format == EpochViewFormat::Json {
        {
            let mut escaped = EscapingWriter::new(writer, format);
            serde_json::to_writer(&mut escaped, report)
                .map_err(|_| error("epoch report complete encoding exceeds profile"))?;
            escaped.flush().map_err(|_| error("epoch report complete encoding exceeds profile"))?;
        }
        writer
            .write_all(b"\n")
            .map_err(|_| error("epoch report complete encoding exceeds profile"))?;
        return Ok(());
    }
    let (prefix, suffix) = if format == EpochViewFormat::Html {
        (HTML_PREFIX, HTML_SUFFIX)
    } else {
        (TEXT_PREFIX, TEXT_SUFFIX)
    };
    writer.write_all(prefix).map_err(|_| error("epoch report complete view exceeds profile"))?;
    {
        let mut escaped = EscapingWriter::new(writer, format);
        serde_json::to_writer_pretty(&mut escaped, report)
            .map_err(|_| error("epoch report complete view exceeds profile"))?;
        escaped.flush().map_err(|_| error("epoch report complete view exceeds profile"))?;
    }
    writer.write_all(suffix).map_err(|_| error("epoch report complete view exceeds profile"))
}

/// One writer charges every complete output chunk before a Vec can grow; None retains no bytes.
struct CappedWriter {
    /// Present only for actual encoding; counting never constructs a byte Vec.
    bytes: Option<Vec<u8>>,
    /// Complete charged encoded byte count, including escaping and framing.
    count: usize,
    /// Per-format ceiling; the separate producer aggregate J+V limit is still mandatory.
    cap: usize,
}

impl CappedWriter {
    /// Create a nonretaining counter, with no output-buffer capacity allocation.
    fn counting(cap: usize) -> Self {
        Self { bytes: None, count: 0, cap }
    }

    /// Create an initially empty collecting writer; every later chunk is charged first.
    fn collecting(cap: usize) -> Self {
        Self { bytes: Some(Vec::new()), count: 0, cap }
    }
}

impl Write for CappedWriter {
    /// Admit checked complete chunk length before retention, never return a truncated success.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .count
            .checked_add(bytes.len())
            .filter(|next| *next <= self.cap)
            .ok_or_else(|| io::Error::other("epoch report complete output exceeds cap"))?;
        if let Some(buffer) = &mut self.bytes {
            buffer.extend_from_slice(bytes);
        }
        self.count = next;
        Ok(bytes.len())
    }

    /// No IO target or publication exists; complete writer flush is a no-op.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Streaming UTF8-aware display escaping, with only a four-byte unfinished-codepoint buffer.
struct EscapingWriter<'a> {
    /// Actual capped counting/encoding sink, never a detached filesystem/stdout writer.
    writer: &'a mut CappedWriter,
    /// Selected format; Json/Text preserve markup as JSON, while Html escapes it.
    format: EpochViewFormat,
    /// At most one codepoint held across arbitrary serializer chunk boundaries.
    pending: [u8; 4],
    /// Number of actual pending UTF8 bytes; zero outside a split codepoint.
    used: usize,
    /// Complete expected UTF8 codepoint length for the pending first byte.
    expected: usize,
}

impl<'a> EscapingWriter<'a> {
    /// Borrow the one sink; there is no raw JSON copy or uncapped intermediate string.
    fn new(writer: &'a mut CappedWriter, format: EpochViewFormat) -> Self {
        Self { writer, format, pending: [0; 4], used: 0, expected: 0 }
    }

    /// Emit markup-safe ASCII or visible JSON escapes for non-formatting controls.
    fn ascii(&mut self, byte: u8) -> io::Result<()> {
        if (byte < 32 && byte != b'\n') || byte == 127 {
            return write!(self.writer, "\\u{:04x}", u32::from(byte));
        }
        let entity: Option<&[u8]> = if self.format == EpochViewFormat::Html {
            match byte {
                b'&' => Some(b"&amp;"),
                b'<' => Some(b"&lt;"),
                b'>' => Some(b"&gt;"),
                b'\"' => Some(b"&quot;"),
                b'\'' => Some(b"&#39;"),
                _ => None,
            }
        } else {
            None
        };
        match entity {
            Some(bytes) => self.writer.write_all(bytes),
            None => self.writer.write_all(&[byte]),
        }
    }

    /// Finish a complete valid Unicode scalar, escaping C1 controls as JSON rather than HTML aliases.
    fn unicode(&mut self) -> io::Result<()> {
        let text = std::str::from_utf8(&self.pending[..self.used])
            .map_err(|_| io::Error::other("epoch report view needs valid UTF8"))?;
        let scalar = text
            .chars()
            .next()
            .ok_or_else(|| io::Error::other("epoch report view needs a Unicode scalar"))?;
        if visible_scalar(scalar) {
            write!(self.writer, "\\u{:04x}", u32::from(scalar))?;
        } else {
            self.writer.write_all(text.as_bytes())?;
        }
        self.used = 0;
        self.expected = 0;
        Ok(())
    }

    /// Handle arbitrary byte/chunk boundaries without splitting an admitted Unicode scalar.
    fn byte(&mut self, byte: u8) -> io::Result<()> {
        if self.used != 0 {
            self.pending[self.used] = byte;
            self.used += 1;
            if self.used == self.expected {
                self.unicode()?;
            }
        } else if byte.is_ascii() {
            self.ascii(byte)?;
        } else {
            self.expected = match byte {
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => return Err(io::Error::other("epoch report view needs valid UTF8")),
            };
            self.pending[0] = byte;
            self.used = 1;
        }
        Ok(())
    }
}

/// Display all Unicode controls and directional/line-separator overrides as reversible JSON escapes.
fn visible_scalar(scalar: char) -> bool {
    scalar.is_control()
        || matches!(
            u32::from(scalar),
            0x061c | 0x200e | 0x200f | 0x2028 | 0x2029 | 0x202a..=0x202e | 0x2066..=0x2069
        )
}

impl Write for EscapingWriter<'_> {
    /// Preserve each complete UTF8 scalar while escaping every dynamic markup/control character.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for byte in bytes {
            self.byte(*byte)?;
        }
        Ok(bytes.len())
    }

    /// Refuse incomplete UTF8 rather than returning an apparently complete counted view.
    fn flush(&mut self) -> io::Result<()> {
        if self.used != 0 {
            return Err(io::Error::other("epoch report view has incomplete UTF8"));
        }
        self.writer.flush()
    }
}

/// Validate borrowed typed fields before encoded buffer growth; no semantic source joins are inferred.
fn validate_typed(report: &EpochReport) -> Result<(), ForgeError> {
    if report.schema_version != SCHEMA_VERSION
        || report.validation_scope != VALIDATION_SCOPE
        || report.continuity_authority != CONTINUITY_AUTHORITY
        || report.assessment_authority
        || report.trust_boundary != TRUST_BOUNDARY
    {
        return Err(error("epoch report fixed labels are invalid"));
    }
    if report.context.len() != 4
        || !(2..=MAX_EPOCHS).contains(&report.epochs.len())
        || report.objects.len() > MAX_OBJECTS
        || report.families.len() > MAX_OBJECTS
        || report.continuity_edges.len() > MAX_OBJECTS
        || report.reciprocal_rows.len() > MAX_RECIPROCALS
        || report.changes.len() > MAX_RELATIONSHIPS
    {
        return Err(error("epoch report complete array exceeds profile bounds"));
    }
    check_uuid(&report.prior_native.uuid)?;
    check_digest(&report.prior_native.raw_sha256)?;
    check_native_typed(&report.native)?;
    check_counts_typed(&report.counts)?;
    for row in &report.context {
        check_context_typed(row)?;
    }
    for row in &report.epochs {
        check_epoch_typed(row)?;
    }
    for row in &report.objects {
        check_object_typed(row)?;
    }
    for row in &report.families {
        check_family_key(&row.family_key)?;
        check_risk_typed(&row.first)?;
        check_risk_typed(&row.current)?;
    }
    for row in &report.continuity_edges {
        check_edge_typed(row)?;
    }
    for row in &report.reciprocal_rows {
        check_family_key(&row.family_key)?;
        check_digest(&row.edge_sha256)?;
        check_risk_typed(&row.from)?;
        check_risk_typed(&row.to)?;
    }
    for row in &report.changes {
        check_change_typed(row)?;
    }
    Ok(())
}

/// Check new filename/native metadata fields without an owned Value serialization.
fn check_native_typed(row: &NativeIdentity) -> Result<(), ForgeError> {
    check_text(&row.artifact, 128, true)?;
    crate::authoring::output::validate_relative(&row.artifact)
        .map_err(|_| error("epoch report artifact filename is invalid"))?;
    if row.artifact.contains('/')
        || std::path::Path::new(&row.artifact).extension() != Some(std::ffi::OsStr::new("json"))
    {
        return Err(error("epoch report artifact filename is invalid"));
    }
    check_uuid(&row.uuid)?;
    check_text(&row.document_version, MAX_STRING_BYTES, true)?;
    check_oscal_version(&row.oscal_version, true)?;
    check_digest(&row.raw_sha256)
}

/// Enumerate all typed denominators in the fixed field order for identical upper-bound checks.
fn check_counts_typed(counts: &Counts) -> Result<(), ForgeError> {
    let values = [
        counts.before_epochs,
        counts.after_epochs,
        counts.preserved_epochs,
        counts.appended_epochs,
        counts.before_observations,
        counts.after_observations,
        counts.before_findings,
        counts.after_findings,
        counts.before_risks,
        counts.after_risks,
        counts.before_objects,
        counts.after_objects,
        counts.preserved_objects,
        counts.added_objects,
        counts.native_graph_occurrences_before,
        counts.native_graph_occurrences_after,
        counts.families,
        counts.continuity_edges,
        counts.reciprocal_rows,
        counts.classified_next_risks,
        counts.uncontinued_families,
        counts.captured_original_generations,
        counts.review_actions,
    ];
    for (field, count) in COUNT_FIELDS.iter().zip(values) {
        if count > count_limit(field) {
            return Err(error("epoch report integer exceeds profile bounds"));
        }
    }
    Ok(())
}

/// Check exact context scalar syntax only; complete unique native membership is producer-owned.
fn check_context_typed(row: &ContextRow) -> Result<(), ForgeError> {
    check_digest(&row.sha256)?;
    check_uuid(&row.root_uuid)?;
    check_text(&row.document_version, MAX_STRING_BYTES, true)?;
    check_oscal_version(&row.oscal_version, false)
}

/// Check explicit complete epoch scalar shape without filling dates or inferring time order.
fn check_epoch_typed(row: &EpochRow) -> Result<(), ForgeError> {
    if row.position >= MAX_EPOCHS
        || [row.observations, row.findings, row.risks].iter().any(|count| *count > MAX_OBJECTS)
    {
        return Err(error("epoch report integer exceeds profile bounds"));
    }
    check_text(&row.key, MAX_STRING_BYTES, true)?;
    check_uuid(&row.uuid)?;
    check_digest(&row.canonical_sha256)?;
    check_text(&row.start, MAX_STRING_BYTES, true)?;
    check_text(&row.end, MAX_STRING_BYTES, true)?;
    check_timestamp(&row.start)?;
    check_timestamp(&row.end)
}

/// Check exact full locator scalars without another conclusion-kind projection or key rewrite.
fn check_locator_typed(row: &ObjectLocator) -> Result<(), ForgeError> {
    check_text(&row.result_key, MAX_STRING_BYTES, true)?;
    check_uuid(&row.result_uuid)?;
    check_text(&row.key, MAX_STRING_BYTES, true)?;
    check_uuid(&row.uuid)
}

/// Check all object strings and separately named hashes before output-buffer retention.
fn check_object_typed(row: &ObjectRow) -> Result<(), ForgeError> {
    check_text(&row.result_key, MAX_STRING_BYTES, true)?;
    check_uuid(&row.result_uuid)?;
    check_text(&row.key, MAX_STRING_BYTES, true)?;
    check_uuid(&row.uuid)?;
    check_digest(&row.computed_sha256)?;
    check_digest(&row.declared_content_sha256)?;
    check_digest(&row.declared_rationale_sha256)?;
    if let Some(status) = &row.status {
        check_text(status, MAX_STRING_BYTES, false)?;
    }
    Ok(())
}

/// Enforce the risk-only existing source tuple without claiming it matches an actual native file.
fn check_risk_typed(row: &RiskReference) -> Result<(), ForgeError> {
    if row.kind != SourceKind::Risk {
        return Err(error("epoch report continuity endpoint must be a risk"));
    }
    check_text(&row.key, MAX_STRING_BYTES, true)?;
    check_uuid(&row.uuid)?;
    check_uuid(&row.result_uuid)?;
    check_digest(&row.expected_sha256)
}

/// Check minimized typed provenance; actor/time-window equality remains producer-owned.
fn check_provenance_typed(row: &ReportProvenance) -> Result<(), ForgeError> {
    check_text(&row.assessor_key, MAX_STRING_BYTES, true)?;
    check_text(&row.role_id, MAX_STRING_BYTES, true)?;
    check_text(&row.start, MAX_STRING_BYTES, true)?;
    check_timestamp(&row.start)?;
    if let Some(end) = &row.end {
        check_text(end, MAX_STRING_BYTES, true)?;
        check_timestamp(end)?;
    }
    check_digest(&row.rationale_sha256)
}

/// Check every edge field without accepting a detached digest as family/source authority.
fn check_edge_typed(row: &ContinuityEdge) -> Result<(), ForgeError> {
    check_family_key(&row.family_key)?;
    check_risk_typed(&row.predecessor)?;
    check_risk_typed(&row.successor)?;
    check_provenance_typed(&row.provenance)?;
    if !row.caller_asserted_continuity {
        return Err(error("epoch report authority marker is invalid"));
    }
    check_digest(&row.edge_sha256)
}

/// Check every supplied nullable change scalar, preserving None and empty scalar distinctions.
fn check_change_typed(row: &ChangeRow) -> Result<(), ForgeError> {
    if let Some(locator) = &row.locator {
        check_locator_typed(locator)?;
    }
    if let Some(key) = &row.family_key {
        check_family_key(key)?;
    }
    if let Some(old) = &row.old {
        check_text(old, MAX_STRING_BYTES, false)?;
    }
    if let Some(new) = &row.new {
        check_text(new, MAX_STRING_BYTES, false)?;
    }
    Ok(())
}

/// Emit a fixed typed report error only; caller fields, paths and native prose never enter it.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::AssessmentResultsBuild(reason.to_string())
}

/// Proposed syntax/formatter controls only; these fixtures are not native captures or actor approval.
#[cfg(test)]
mod tests {
    use super::*;
    use serde::ser::SerializeSeq;
    use serde_json::json;

    /// Canonical bounded placeholder UUID for the retained result/document syntax fixture.
    const OLD_UUID: &str = "00000001-0000-4000-8000-000000000001";
    /// Distinct canonical result UUID for the appended result syntax fixture.
    const NEW_UUID: &str = "00000002-0000-4000-8000-000000000002";
    /// Complete lowercase synthetic digest, never an original-byte or canonical-source proof.
    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    /// Complete existing risk tuple shape; no current-native eligibility is inferred.
    fn risk(key: &str, uuid: &str, result_uuid: &str) -> Value {
        json!({"kind":"risk","key":key,"uuid":uuid,"result_uuid":result_uuid,"expected_sha256":SHA})
    }

    /// Complete all-kind object row with explicit nullable status and independently named hashes.
    fn object(result: &str, result_uuid: &str, kind: &str, added: bool) -> Value {
        json!({
            "result_key":result,"result_uuid":result_uuid,"kind":kind,"key":kind,
            "uuid":result_uuid,"computed_sha256":SHA,"declared_content_sha256":SHA,
            "declared_rationale_sha256":SHA,"status":if kind == "observation" {None} else if kind == "finding" {Some("not-satisfied")} else {Some("open")},
            "classification":if added {"added"} else {"preserved"}
        })
    }

    /// Exact complete locator shape; synthetic identifiers grant no tuple/native membership.
    fn locator(kind: &str) -> Value {
        json!({"result_key":"epoch-next","result_uuid":NEW_UUID,"kind":kind,"key":kind,"uuid":NEW_UUID})
    }

    /// Complete closed syntactic report, with every row kind/count/null field explicitly present.
    ///
    /// Repeated placeholder UUIDs/hashes are intentional: the producer owns actual
    /// native uniqueness, graph/family/source joins and evidence, not this fixture.
    fn fixture() -> Value {
        let counts = json!({
            "before_epochs":1,"after_epochs":2,"preserved_epochs":1,"appended_epochs":1,
            "before_observations":1,"after_observations":2,"before_findings":1,"after_findings":2,
            "before_risks":1,"after_risks":2,"before_objects":3,"after_objects":6,
            "preserved_objects":3,"added_objects":3,"native_graph_occurrences_before":2,
            "native_graph_occurrences_after":4,"families":1,"continuity_edges":1,"reciprocal_rows":2,
            "classified_next_risks":1,"uncontinued_families":0,"captured_original_generations":6,"review_actions":4
        });
        let context = ["assessment-plan","system-security-plan","profile","catalog"].map(|kind| {
            json!({"kind":kind,"sha256":SHA,"root_uuid":OLD_UUID,"document_version":"1","oscal_version":"1.2.3"})
        });
        let epochs = [
            json!({"position":0,"key":"epoch-old","uuid":OLD_UUID,"canonical_sha256":SHA,"start":"2026-10-03T00:00:00Z","end":"2026-10-03T01:00:00Z","observations":1,"findings":1,"risks":1,"classification":"preserved"}),
            json!({"position":1,"key":"epoch-next","uuid":NEW_UUID,"canonical_sha256":SHA,"start":"2026-10-04T00:00:00+01:00","end":"2026-10-04T01:00:00+01:00","observations":1,"findings":1,"risks":1,"classification":"appended"}),
        ];
        json!({
            "schema_version":SCHEMA_VERSION,"validation_scope":VALIDATION_SCOPE,
            "status":"review-required","review_required":true,"assessment_authority":false,
            "continuity_authority":CONTINUITY_AUTHORITY,
            "prior_native":{"uuid":OLD_UUID,"raw_sha256":SHA},
            "native":{"artifact":"results-next.json","uuid":OLD_UUID,"document_version":"2","oscal_version":"1.2.3","raw_sha256":SHA},
            "context":context,"counts":counts,"epochs":epochs,
            "objects":[object("epoch-old",OLD_UUID,"observation",false),object("epoch-old",OLD_UUID,"finding",false),object("epoch-old",OLD_UUID,"risk",false),object("epoch-next",NEW_UUID,"observation",true),object("epoch-next",NEW_UUID,"finding",true),object("epoch-next",NEW_UUID,"risk",true)],
            "families":[{"family_key":"family","first":risk("risk",OLD_UUID,OLD_UUID),"current":risk("risk",NEW_UUID,NEW_UUID),"continued_in_append":true}],
            "continuity_edges":[{"family_key":"family","predecessor":risk("risk",OLD_UUID,OLD_UUID),"successor":risk("risk",NEW_UUID,NEW_UUID),"provenance":{"assessor_key":"assessor","role_id":"assessor","start":"2026-10-04T00:00:00+01:00","end":null,"method":"UNKNOWN","rationale_sha256":SHA},"caller_asserted_continuity":true,"edge_sha256":SHA}],
            "reciprocal_rows":[
                {"family_key":"family","edge_sha256":SHA,"direction":"predecessor","from":risk("risk",OLD_UUID,OLD_UUID),"to":risk("risk",NEW_UUID,NEW_UUID)},
                {"family_key":"family","edge_sha256":SHA,"direction":"successor","from":risk("risk",NEW_UUID,NEW_UUID),"to":risk("risk",OLD_UUID,OLD_UUID)}],
            "changes":[
                {"code":"epoch-added","locator":null,"family_key":null,"old":null,"new":NEW_UUID},
                {"code":"object-added","locator":locator("observation"),"family_key":null,"old":null,"new":NEW_UUID},
                {"code":"object-added","locator":locator("finding"),"family_key":null,"old":null,"new":NEW_UUID},
                {"code":"object-added","locator":locator("risk"),"family_key":null,"old":null,"new":NEW_UUID}],
            "trust_boundary":TRUST_BOUNDARY
        })
    }

    /// Consume actual strict/schema syntax admission before a formatter/negative control.
    fn admitted() -> EpochReport {
        let bytes = serde_json::to_vec(&fixture()).expect("synthetic report serializes");
        let value = parse_raw(&bytes).expect("complete synthetic report shape admits");
        decode(&value).expect("typed report admits after explicit caller precharge in production")
    }

    /// Bind an intended refusal to a valid baseline and check its fixed, redacted branch reason.
    fn refused(value: &Value, reason: &str) {
        let wire = serde_json::to_vec(value).expect("negative syntax fixture serializes");
        let Err(error) = parse_raw(&wire) else {
            panic!("intended synthetic report refusal was not reached");
        };
        assert!(error.to_string().contains(reason));
    }

    /// Decode only the five explicit entities emitted by the actual inert HTML adapter, once.
    fn html_json(view: &[u8]) -> Value {
        let body = view
            .strip_prefix(HTML_PREFIX)
            .and_then(|body| body.strip_suffix(HTML_SUFFIX))
            .expect("complete static HTML frame");
        let decoded = std::str::from_utf8(body)
            .expect("UTF8 HTML body")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");
        serde_json::from_str(&decoded).expect("complete escaped JSON dump roundtrips")
    }

    /// Verify full same-DTO row/count/null/time/hash preservation in every actual renderer.
    #[test]
    fn complete_formats_preserve_all_fields_counts_arrays_and_nulls() {
        let report = admitted();
        let original = fixture();
        let j = count_json(&report).expect("complete JSON counter");
        let json = encode_json(&report, j).expect("actual JSON under admitted ceiling");
        assert_eq!(parse_raw(&json).expect("completed JSON validates"), original);
        assert_eq!(count_view(&report, Some(EpochViewFormat::Json)).unwrap(), j);
        assert_eq!(encode_view(&report, Some(EpochViewFormat::Json), j).unwrap().unwrap(), json);
        assert_eq!(count_view(&report, None).unwrap(), 0);
        assert!(encode_view(&report, None, 0).unwrap().is_none());
        for format in [EpochViewFormat::Text, EpochViewFormat::Html] {
            let v = count_view(&report, Some(format)).expect("complete view counter");
            let bytes = encode_view(&report, Some(format), v).unwrap().unwrap();
            assert_eq!(bytes.len(), v);
            let value = if format == EpochViewFormat::Html {
                html_json(&bytes)
            } else {
                let body = bytes
                    .strip_prefix(TEXT_PREFIX)
                    .and_then(|body| body.strip_suffix(TEXT_SUFFIX))
                    .unwrap();
                serde_json::from_slice(body).expect("complete text JSON field dump")
            };
            assert_eq!(value, original);
            assert_eq!(value["counts"].as_object().unwrap().len(), COUNT_FIELDS.len());
            assert_eq!(value["objects"].as_array().unwrap().len(), 6);
            assert_eq!(value["reciprocal_rows"].as_array().unwrap().len(), 2);
        }
    }

    /// Verify explicit nullable fields admit null yet every missing field refuses even direct serde.
    #[test]
    fn required_nullable_fields_never_default_when_missing() {
        let _positive = admitted();
        for (path, field) in [
            ("/objects/0", "status"),
            ("/continuity_edges/0/provenance", "end"),
            ("/changes/0", "locator"),
            ("/changes/0", "family_key"),
            ("/changes/0", "old"),
            ("/changes/0", "new"),
        ] {
            let mut missing = fixture();
            missing.pointer_mut(path).unwrap().as_object_mut().unwrap().remove(field);
            refused(&missing, "missing or unknown fields");
            assert!(EpochReport::deserialize(&missing).is_err());
        }
    }

    /// Verify all closed record levels refuse unknown fields, and nullable fields reject wrong types.
    #[test]
    fn closed_records_and_nullable_wrong_types_refuse() {
        let _positive = admitted();
        for path in [
            "",
            "/prior_native",
            "/native",
            "/context/0",
            "/counts",
            "/epochs/0",
            "/objects/0",
            "/families/0",
            "/families/0/first",
            "/continuity_edges/0",
            "/continuity_edges/0/provenance",
            "/reciprocal_rows/0",
            "/changes/0",
            "/changes/1/locator",
        ] {
            let mut unknown = fixture();
            unknown
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unknown".into(), json!(true));
            refused(&unknown, "missing or unknown fields");
        }
        for path in [
            "/objects/0/status",
            "/continuity_edges/0/provenance/end",
            "/changes/0/locator",
            "/changes/0/family_key",
            "/changes/0/old",
            "/changes/0/new",
        ] {
            let mut wrong = fixture();
            *wrong.pointer_mut(path).unwrap() = json!([]);
            assert!(parse_raw(&serde_json::to_vec(&wrong).unwrap()).is_err());
        }
    }

    /// Verify the actual closed packaged validator independently and through completed encoding.
    #[test]
    fn offline_schema_is_consumed_for_prior_and_completed_json() {
        let value = fixture();
        validate_schema(&value).expect("packaged syntax schema positive");
        let mut unknown = fixture();
        unknown.as_object_mut().unwrap().insert("unknown".into(), json!(true));
        assert!(validate_schema(&unknown).is_err());
        let j = count_json(&value).expect("borrowed Value can be counted without typed rows");
        let wire =
            encode_json(&value, j).expect("generic borrowed source mirror validates final bytes");
        assert_eq!(parse_raw(&wire).unwrap(), value);
        let mut wrong = fixture();
        wrong["schema_version"] = json!("wrong-protocol");
        let ceiling = count_json(&wrong).unwrap();
        assert!(encode_json(&wrong, ceiling).is_err());
    }

    /// Verify exact family spelling admits whitespace and enforces UTF8 bytes without normalization.
    #[test]
    fn exact_family_domain_preserves_whitespace_and_utf8_boundaries() {
        let _positive = admitted();
        for key in [" ".to_string(), "é".repeat(128)] {
            let mut value = fixture();
            value["families"][0]["family_key"] = json!(key);
            value["continuity_edges"][0]["family_key"] = json!(key);
            value["reciprocal_rows"][0]["family_key"] = json!(key);
            value["reciprocal_rows"][1]["family_key"] = json!(key);
            value["changes"][3]["family_key"] = json!(key);
            value["changes"][3]["code"] = json!("risk-continuity-recorded");
            let bytes = serde_json::to_vec(&value).unwrap();
            let raw = parse_raw(&bytes).expect("exact nonempty family key domain");
            let typed = decode(&raw).unwrap();
            let ceiling = count_json(&typed).unwrap();
            assert_eq!(parse_raw(&encode_json(&typed, ceiling).unwrap()).unwrap(), value);
        }
        for key in [String::new(), "é".repeat(129)] {
            let mut value = fixture();
            value["families"][0]["family_key"] = json!(key);
            refused(&value, "family key exceeds profile bounds");
        }
    }

    /// Verify finite domains/authority markers and risk-only `SourceReference` admission, not endpoints.
    #[test]
    fn enums_authority_and_risk_reference_domain_refuse() {
        let _positive = admitted();
        for (path, bad) in [
            ("/status", json!("approved")),
            ("/context/0/kind", json!("observation")),
            ("/objects/0/kind", json!("work")),
            ("/epochs/0/classification", json!("deleted")),
            ("/objects/0/classification", json!("closed")),
            ("/reciprocal_rows/0/direction", json!("none")),
            ("/changes/0/code", json!("risk-closed")),
            ("/assessment_authority", json!(true)),
            ("/continuity_edges/0/caller_asserted_continuity", json!(false)),
            ("/families/0/current/kind", json!("finding")),
        ] {
            let mut value = fixture();
            *value.pointer_mut(path).unwrap() = bad;
            assert!(parse_raw(&serde_json::to_vec(&value).unwrap()).is_err());
        }
        let mut typed = admitted();
        typed.families[0].current.kind = SourceKind::Finding;
        let Err(error) = encode_view(&typed, Some(EpochViewFormat::Text), MAX_REPORT_BYTES) else {
            panic!("typed finding must not become risk continuity");
        };
        assert!(error.to_string().contains("endpoint must be a risk"));
    }

    /// Verify every complete unselected roster is bounded before invalid row contents are visited.
    #[test]
    fn every_roster_bound_precedes_typed_or_row_retention() {
        let _positive = admitted();
        for (field, limit) in [
            ("context", 4),
            ("epochs", MAX_EPOCHS),
            ("objects", MAX_OBJECTS),
            ("families", MAX_OBJECTS),
            ("continuity_edges", MAX_OBJECTS),
            ("reciprocal_rows", MAX_RECIPROCALS),
            ("changes", MAX_RELATIONSHIPS),
        ] {
            let mut value = fixture();
            value[field] = Value::Array(vec![Value::Null; limit + 1]);
            // Null rows would fail a different branch if visited before this ceiling.
            refused(&value, "complete array exceeds profile bounds");
        }
    }

    /// Verify required integer domains reject signed/floating/unbounded counts and one-append excess.
    #[test]
    fn integer_counts_are_complete_and_profile_bounded() {
        let _positive = admitted();
        for (field, bad) in [
            ("after_objects", json!(-1)),
            ("after_objects", json!(1.0)),
            ("after_objects", json!(MAX_OBJECTS + 1)),
            ("appended_epochs", json!(2)),
            ("captured_original_generations", json!(8)),
            ("reciprocal_rows", json!(MAX_RECIPROCALS + 1)),
            ("review_actions", json!(MAX_RELATIONSHIPS + 1)),
        ] {
            let mut value = fixture();
            value["counts"][field] = bad;
            refused(&value, "integer exceeds profile bounds");
        }
        let mut missing = fixture();
        missing["counts"].as_object_mut().unwrap().remove("before_objects");
        refused(&missing, "missing or unknown fields");
    }

    /// Verify strict duplicate/trailing input and raw UTF8-byte caps through the consumed parser.
    #[test]
    fn strict_wire_and_decoded_utf8_bounds_are_consumed() {
        let positive = serde_json::to_vec(&fixture()).unwrap();
        parse_raw(&positive).expect("otherwise-valid baseline");
        let text = String::from_utf8(positive).unwrap();
        let needle = format!("\"schema_version\":\"{SCHEMA_VERSION}\"");
        assert_eq!(text.matches(&needle).count(), 1);
        let duplicate = text.replacen(&needle, &format!("{needle},{needle}"), 1);
        assert!(parse_raw(duplicate.as_bytes()).is_err());
        let trailing = format!("{text} {{}}");
        assert!(parse_raw(trailing.as_bytes()).is_err());
        let mut value = fixture();
        value["objects"][0]["key"] = json!("é".repeat(MAX_STRING_BYTES / 2));
        let at = serde_json::to_vec(&value).unwrap();
        assert!(parse_raw(&at).is_ok());
        value["objects"][0]["key"] = json!("é".repeat(MAX_STRING_BYTES / 2 + 1));
        assert!(parse_raw(&serde_json::to_vec(&value).unwrap()).is_err());
        let mut over_raw = serde_json::to_vec(&fixture()).unwrap();
        over_raw.resize(MAX_REPORT_BYTES + 1, b' ');
        let Err(error) = parse_raw(&over_raw) else {
            panic!("raw ceiling must refuse");
        };
        assert!(error.to_string().contains("exceeds ten MiB"));
    }

    /// Verify markup/Unicode controls are visible yet every complete same-DTO value roundtrips.
    #[test]
    fn malicious_markup_and_unicode_controls_are_escaped_losslessly() {
        let mut value = fixture();
        let attack = "<&\"'><script>alert('x')</script><a href='https://x'>é🙂\u{009b}\u{007f}\u{202e}\u{2028}\u{2067}";
        value["objects"][0]["key"] = json!(attack);
        let typed = decode(&parse_raw(&serde_json::to_vec(&value).unwrap()).unwrap()).unwrap();
        for format in [EpochViewFormat::Json, EpochViewFormat::Text, EpochViewFormat::Html] {
            let cap = count_view(&typed, Some(format)).unwrap();
            let bytes = encode_view(&typed, Some(format), cap).unwrap().unwrap();
            assert_eq!(bytes.len(), cap);
            let display = std::str::from_utf8(&bytes).unwrap();
            for scalar in ['\u{009b}', '\u{007f}', '\u{202e}', '\u{2028}', '\u{2067}'] {
                assert!(!display.contains(scalar));
            }
            let decoded = match format {
                EpochViewFormat::Html => {
                    assert!(!display.contains("<script"));
                    assert!(!display.contains("<a href"));
                    html_json(&bytes)
                }
                EpochViewFormat::Text => {
                    let body = bytes
                        .strip_prefix(TEXT_PREFIX)
                        .and_then(|body| body.strip_suffix(TEXT_SUFFIX))
                        .unwrap();
                    serde_json::from_slice(body).unwrap()
                }
                EpochViewFormat::Json => parse_raw(&bytes).unwrap(),
            };
            assert_eq!(decoded, value);
        }
    }

    /// Verify all view serializers use the caller's precharged ceiling before any excess growth.
    #[test]
    fn actual_admitted_ceiling_refuses_without_encode_then_cap() {
        let report = admitted();
        let json_cap = count_json(&report).unwrap();
        assert_eq!(encode_json(&report, json_cap).unwrap().len(), json_cap);
        let Err(error) = encode_json(&report, json_cap - 1) else {
            panic!("short JSON cap must refuse");
        };
        assert!(error.to_string().contains("encoding exceeds profile"));
        for format in [EpochViewFormat::Text, EpochViewFormat::Html] {
            let cap = count_view(&report, Some(format)).unwrap();
            assert_eq!(encode_view(&report, Some(format), cap).unwrap().unwrap().len(), cap);
            let Err(error) = encode_view(&report, Some(format), cap - 1) else {
                panic!("short view cap must refuse");
            };
            assert!(error.to_string().contains("view exceeds profile"));
        }
        assert!(encode_json(&report, MAX_REPORT_BYTES + 1).is_err());
        assert!(encode_view(&report, None, MAX_REPORT_BYTES + 1).is_err());
        let mut sink = CappedWriter::collecting(4);
        sink.write_all(b"12").unwrap();
        assert!(sink.write_all(b"345").is_err());
        assert_eq!(sink.count, 2);
        assert_eq!(sink.bytes.unwrap(), b"12");
    }

    /// Verify arbitrary serializer splits preserve Unicode and cap expanded escaped bytes first.
    #[test]
    fn streaming_unicode_chunks_and_expanded_sink_caps_are_exact() {
        let input = "<&é🙂\u{009b}\u{202e}";
        let mut counted = CappedWriter::counting(1000);
        {
            let mut writer = EscapingWriter::new(&mut counted, EpochViewFormat::Html);
            for byte in input.as_bytes() {
                writer.write_all(&[*byte]).unwrap();
            }
            writer.flush().unwrap();
        }
        assert!(counted.bytes.is_none());
        let mut retained = CappedWriter::collecting(counted.count);
        {
            let mut writer = EscapingWriter::new(&mut retained, EpochViewFormat::Html);
            writer.write_all(input.as_bytes()).unwrap();
            writer.flush().unwrap();
        }
        assert_eq!(retained.count, counted.count);
        assert_eq!(
            String::from_utf8(retained.bytes.unwrap()).unwrap(),
            "&lt;&amp;é🙂\\u009b\\u202e"
        );
        let mut short = CappedWriter::collecting(3);
        let mut escaped = EscapingWriter::new(&mut short, EpochViewFormat::Html);
        assert!(escaped.write_all(b"&").is_err());
        assert_eq!(short.count, 0);
        assert_eq!(short.bytes.unwrap(), [] as [u8; 0]);
    }

    /// Borrowed repeating syntax projection for output-cap controls, never a native/report proof.
    struct RepeatProjection<'a> {
        /// One bounded borrowed string reused without retaining a second row collection.
        value: &'a str,
        /// Finite synthetic repetition count; no parser/source authority exists.
        repeat: usize,
    }

    impl Serialize for RepeatProjection<'_> {
        /// Stream each borrowed string through the actual generic nonretaining/counting writer.
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut sequence = serializer.serialize_seq(Some(self.repeat))?;
            for _ in 0..self.repeat {
                sequence.serialize_element(self.value)?;
            }
            sequence.end()
        }
    }

    /// Verify fixed per-format counters refuse complete overflow without retaining a prefix Vec.
    #[test]
    fn borrowed_counters_refuse_complete_overflow_without_retaining_bytes() {
        let _positive = admitted();
        let one = "&".repeat(MAX_STRING_BYTES);
        let mirror = RepeatProjection { value: &one, repeat: 200 };
        assert!(count_json(&mirror).is_err());
        assert!(count_view(&mirror, Some(EpochViewFormat::Text)).is_err());
        assert!(count_view(&mirror, Some(EpochViewFormat::Html)).is_err());
        assert_eq!(count_view(&mirror, None).unwrap(), 0);
        // The generic encoder reaches its actual chunk cap before closed-report validation.
        let Err(error) = encode_json(&mirror, MAX_REPORT_BYTES) else {
            panic!("complete overflow must refuse");
        };
        assert!(error.to_string().contains("encoding exceeds profile"));
    }

    /// Verify typed views share the packaged finite native/companion version domain after admission.
    #[test]
    fn typed_views_and_wire_share_closed_oscal_version_domains() {
        let _positive = admitted();
        for version in ["1.2.0", "1.2.1", "1.2.2", "1.2.3"] {
            let mut value = fixture();
            value["context"][0]["oscal_version"] = json!(version);
            let typed = decode(&parse_raw(&serde_json::to_vec(&value).unwrap()).unwrap()).unwrap();
            let cap = count_view(&typed, Some(EpochViewFormat::Text)).unwrap();
            assert!(encode_view(&typed, Some(EpochViewFormat::Text), cap).is_ok());
        }
        for (native, version) in
            [(true, "1.2.2"), (true, "1.1.3"), (false, "1.1.3"), (false, "1.2.4")]
        {
            let mut value = fixture();
            if native {
                value["native"]["oscal_version"] = json!(version);
            } else {
                value["context"][0]["oscal_version"] = json!(version);
            }
            refused(&value, "closed schema is invalid");
            let mut typed = admitted();
            if native {
                typed.native.oscal_version = version.to_string();
            } else {
                typed.context[0].oscal_version = version.to_string();
            }
            for format in [EpochViewFormat::Text, EpochViewFormat::Html] {
                let Err(error) = encode_view(&typed, Some(format), MAX_REPORT_BYTES) else {
                    panic!("typed OSCAL version outside closed profile must refuse");
                };
                assert!(error.to_string().contains("OSCAL version is outside profile"));
            }
        }
    }
}

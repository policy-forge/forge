//! Same-context sealed Assessment Results epoch append over one actual capture proof.
//!
//! This private preparation profile preserves complete admitted historical Values.
//! Assessment and continuity facts remain caller assertions, without remediation authority.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use chrono::{DateTime, FixedOffset};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use super::manifest::SourceManifest;
use super::source::{self, CapturedEpochSource};
use crate::ForgeError;
use crate::assessment_results::{self, epoch_report, manifest, model};
use crate::evidence_capture::{CaptureProof, CaptureRole, CaptureSession, MAX_PROJECTION_BYTES};
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};

/// Complete raw request and compact next-manifest bound, before structural decoding.
const MAX_REQUEST: usize = 4 * 1024 * 1024;
/// Complete prior/final native raw and compact encoding ceiling, including final LF.
const MAX_NATIVE: usize = 50 * 1024 * 1024;
/// Complete retained prior report and every report delivery ceiling.
const MAX_REPORT: usize = 10 * 1024 * 1024;
/// Complete after-append epoch and all-kind conclusion cardinalities.
const MAX_EPOCHS: usize = 1_000;
/// Complete observations, findings and risks across all result epochs.
const MAX_OBJECTS: usize = 10_000;
/// Complete reference occurrences, shared with actual capture and graph consumers.
const MAX_RELATIONSHIPS: usize = 100_000;
/// Exact extension namespace already consumed by the maintained native producer.
const NS: &str = model::FORGE_ASSESSMENT_RESULTS_NS;
/// Fixed minimized assertion boundary used by the versioned report module.
const TRUST_BOUNDARY: &str = "Caller-authored assessment and continuity assertions; identity and structure checks do not authenticate actors, evaluate evidence, or authorize remediation.";
/// Surrogate fixed-width digests used only by nonretaining prospective accounting.
const HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// Surrogate fixed-width canonical identity used only by prospective accounting.
const UUID: &str = "00000000-0000-4000-8000-000000000000";
/// Offline closed request validator; Root packages the exact reviewed schema bytes.
static REQUEST_VALIDATOR: OnceLock<Result<jsonschema::Validator, &'static str>> = OnceLock::new();

/// Complete privately prepared native/report/view bytes with actual original holders.
pub(crate) struct PreparedEpochAppend {
    /// Complete native compact JSON with one LF, after the strict reverse oracle.
    native: Vec<u8>,
    /// Complete admitted report DTO; no detached public producer constructor exists.
    report: epoch_report::EpochReport,
    /// Required durable compact report JSON with one LF.
    report_json: Vec<u8>,
    /// Only selected text/HTML view storage; JSON views borrow `report_json` instead.
    view: Option<Vec<u8>>,
    /// Whether the explicit JSON stdout view borrows the required report delivery.
    json_view: bool,
    /// Actual qualified parent retained for Root's same-parent no-replace publication.
    root: PathBuf,
    /// All six/seven actual original generations plus both namespace reservations.
    proof: CaptureProof,
}

impl PreparedEpochAppend {
    /// Borrow complete native bytes, without publication or assessment authority.
    pub(crate) fn native_bytes(&self) -> &[u8] {
        &self.native
    }
    /// Borrow the complete minimized comparison and continuity DTO.
    pub(crate) fn report(&self) -> &epoch_report::EpochReport {
        &self.report
    }
    /// Borrow the required durable JSON report; view choice never changes its format.
    pub(crate) fn report_json(&self) -> &[u8] {
        &self.report_json
    }
    /// Borrow an explicitly requested complete stdout view, or None without a request.
    pub(crate) fn view_bytes(&self) -> Option<&[u8]> {
        if self.json_view { Some(&self.report_json) } else { self.view.as_deref() }
    }
    /// Borrow the actual qualified parent for private output resolution.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    /// Enumerate all actual original paths; no discovery or authority is inferred.
    pub(crate) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.proof.input_paths()
    }
    /// Reconcile both reserved namespaces and every actual original/ancestor generation.
    pub(crate) fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.proof.verify_inputs().map_err(|_| error("epoch append captured inputs changed"))
    }
    /// Every valid append adds a result and therefore has descriptive review actions.
    pub(crate) fn review_required(&self) -> bool {
        self.report().review_required
    }
}

/// Prepare the required native and JSON report pair with no implicit stdout view.
pub(crate) fn prepare(
    request: &Path,
    native: &Path,
    report: &Path,
) -> Result<PreparedEpochAppend, ForgeError> {
    prepare_with_view(request, native, report, None)
}

/// Prepare one complete sealed append from a single actual native/context capture.
///
/// Root publishes native, required JSON and optional stdout separately, verifying
/// this proof before each delivery. This function performs no publication.
/// # Errors
/// Refuses unsupported wire/profile/chronology/identity/continuity/bounds, unsafe
/// destinations or any changed actual original; errors omit caller prose/paths.
pub(crate) fn prepare_with_view(
    request_path: &Path,
    native_output: &Path,
    report_output: &Path,
    view: Option<epoch_report::EpochViewFormat>,
) -> Result<PreparedEpochAppend, ForgeError> {
    let (parent, request_relative) = declaration_root(request_path)?;
    portable_json(native_output, true)?;
    portable_json(report_output, true)?;
    let mut capture =
        CaptureSession::new(&parent).map_err(|_| error("epoch request root is unsafe"))?;
    capture
        .reserve_outputs(&[native_output, report_output])
        .map_err(|_| error("epoch output namespaces collide or are invalid"))?;
    preflight_destination(&capture, native_output)?;
    preflight_destination(&capture, report_output)?;
    let request_lease = capture
        .required(&request_relative, CaptureRole::AssessmentEpochRequest, MAX_REQUEST as u64)
        .map_err(|_| error("epoch request original is unavailable"))?;
    let request = parse_request(request_lease.bytes())?;
    let source_declaration = decode_prior_source(&request)?;
    let actual_path = capture.root().join(&request_relative);
    let actual =
        source::load_epoch_source_with_capture(&actual_path, &source_declaration, &mut capture)
            .map_err(|_| error("epoch prior native or complete captured context is invalid"))?;
    let prior = actual.original();
    let old_document = document(prior)?;
    validate_selected_inventory(&actual, &source_declaration)?;
    let next_raw = &request["next_epoch"];
    let old_counts = validate_epoch_profile(old_document, &mut capture)?;
    let next_counts = preflight_next(next_raw, old_counts.objects, &mut capture)?;
    let prior_report = capture_prior_report(&request["prior_report"], &mut capture)?;
    let plan = ContinuityPlan::new(
        old_document,
        next_raw,
        &request,
        prior_report.as_ref(),
        &source_declaration,
        &mut capture,
    )?;
    let prospective = ReportMirror {
        old: old_document,
        next: Next::Authored(&next_raw["result"]),
        plan: &plan,
        source: &source_declaration,
        output: native_output.to_str().ok_or_else(|| error("epoch native filename is not UTF8"))?,
        raw_hash: HASH,
        next_version: text(&next_raw["document"]["version"])?,
        old_counts,
        next_counts,
        generations: if prior_report.is_some() { 7 } else { 6 },
    };
    let accounting =
        preflight_projection(next_raw, prior, &source_declaration, &prospective, view)?;
    let built_value = build_next_native(next_raw, &actual, old_document, &plan, accounting.native)?;
    let next_document = document(&built_value)?;
    let next_result = one_result(next_document)?;
    let native = encode_limited(
        &AppendEnvelope { old: prior, next_document, next_result },
        accounting.envelope,
        true,
    )?;
    if native.len() > accounting.envelope {
        return Err(error("epoch native output exceeded its complete admitted ceiling"));
    }
    let final_value = strict(&native, MAX_NATIVE, 128)?;
    reverse_oracle(prior, &final_value, next_result)?;
    source::validate_derived_epoch_graph(&actual, &final_value, &mut capture)
        .map_err(|_| error("epoch complete derived native graph is invalid"))?;
    drop(final_value);
    let native_hash = sha256_hex(&native);
    let actual_mirror = ReportMirror {
        old: old_document,
        next: Next::Native(next_result),
        plan: &plan,
        source: &source_declaration,
        output: prospective.output,
        raw_hash: &native_hash,
        next_version: prospective.next_version,
        old_counts,
        next_counts,
        generations: prospective.generations,
    };
    let completed_report = prepare_complete_report(&actual_mirror, &accounting, view)?;
    let generations = prospective.generations;
    let root = capture.root().to_path_buf();
    drop(actual);
    let proof = capture.finish();
    if proof.captured_original_generations() != generations {
        return Err(error("epoch proof has an incomplete original-generation denominator"));
    }
    let prepared = PreparedEpochAppend {
        native,
        report: completed_report.report,
        report_json: completed_report.json,
        view: completed_report.view,
        json_view: matches!(view, Some(epoch_report::EpochViewFormat::Json)),
        root,
        proof,
    };
    prepared.verify_inputs()?;
    Ok(prepared)
}

/// Admit the complete explicit prior source and require exactly the same next context.
fn decode_prior_source(request: &Value) -> Result<SourceManifest, ForgeError> {
    let source_declaration: SourceManifest = decode_bounded(&request["prior_source"], MAX_REQUEST)?;
    super::manifest::validate_source(&source_declaration)
        .map_err(|_| error("epoch prior source declaration is invalid"))?;
    portable_json(&source_declaration.assessment_results.artifact, true)?;
    if source_declaration.context.evidence_index.is_some() {
        return Err(error("epoch append does not admit an evidence index"));
    }
    if serde_json::to_value(&source_declaration.context)
        .map_err(|_| error("epoch context declaration encoding failed"))?
        != request["next_epoch"]["context"]
    {
        return Err(error("epoch contexts differ from complete actual prior declaration"));
    }
    Ok(source_declaration)
}

/// Build and validate the actual next native result only after complete projection admission.
fn build_next_native(
    next_raw: &Value,
    actual: &source::CapturedEpochSource<'_>,
    old_document: &Value,
    plan: &ContinuityPlan<'_>,
    native_ceiling: usize,
) -> Result<Value, ForgeError> {
    let compact_next = encode_limited(next_raw, MAX_REQUEST, false)?;
    let typed_next =
        manifest::parse(&compact_next).map_err(|_| error("epoch next authoring is invalid"))?;
    drop(compact_next);
    let built = model::build(&typed_next, actual.context())
        .map_err(|_| error("epoch next native construction is invalid"))?;
    let built_bytes = encode_limited(&built.artifact, native_ceiling, true)?;
    drop(built);
    let built_value = strict(&built_bytes, MAX_NATIVE, 128)?;
    drop(built_bytes);
    assessment_results::validate_completed_json(&built_value)
        .map_err(|_| error("epoch next native schema is invalid"))?;
    let next_document = document(&built_value)?;
    validate_same_envelope(old_document, next_document)?;
    validate_append_window(old_document, next_document)?;
    let next_result = one_result(next_document)?;
    check_new_identities(old_document, next_document, next_result)?;
    plan.validate_actual_next(old_document, next_result)?;
    Ok(built_value)
}

/// Complete held report and delivery buffers admitted by the prospective encoded ceilings.
struct PreparedReport {
    /// Complete closed report DTO, decoded only after JSON admission and encoding.
    report: epoch_report::EpochReport,
    /// Complete compact durable JSON bytes with LF.
    json: Vec<u8>,
    /// Optional complete text or HTML view; JSON delivery borrows the durable buffer.
    view: Option<Vec<u8>>,
}

/// Encode the complete report and selected view without truncating their admitted projections.
fn prepare_complete_report(
    actual_mirror: &ReportMirror<'_>,
    accounting: &ProjectionAdmission,
    view: Option<epoch_report::EpochViewFormat>,
) -> Result<PreparedReport, ForgeError> {
    let json = epoch_report::encode_json(actual_mirror, accounting.json)
        .map_err(|_| error("epoch report encoding is invalid"))?;
    if json.len() > accounting.json {
        return Err(error("epoch report exceeded its prospective admission"));
    }
    let raw_report =
        epoch_report::parse_raw(&json).map_err(|_| error("epoch report shape is invalid"))?;
    let report =
        epoch_report::decode(&raw_report).map_err(|_| error("epoch complete report is invalid"))?;
    drop(raw_report);
    let view_bytes = match view {
        Some(epoch_report::EpochViewFormat::Json) | None => None,
        selected => epoch_report::encode_view(&report, selected, accounting.view)
            .map_err(|_| error("epoch view encoding failed"))?,
    };
    let delivered_view = if matches!(view, Some(epoch_report::EpochViewFormat::Json)) {
        json.len()
    } else {
        view_bytes.as_ref().map_or(0, Vec::len)
    };
    if delivered_view > accounting.view {
        return Err(error("epoch view exceeded its prospective admission"));
    }
    Ok(PreparedReport { report, json, view: view_bytes })
}

/// Qualify original path spelling before parent/filename/absolute operations can erase it.
fn declaration_root(path: &Path) -> Result<(PathBuf, PathBuf), ForgeError> {
    if path.as_os_str().is_empty()
        || path.components().any(|part| matches!(part, Component::CurDir | Component::ParentDir))
        || !crate::linkage::has_normalized_path_spelling(path)
    {
        return Err(error("epoch request path spelling is not normalized"));
    }
    #[cfg(windows)]
    if path.has_root() != path.is_absolute()
        || (path.components().any(|part| matches!(part, Component::Prefix(_)))
            && !path.is_absolute())
    {
        return Err(error("epoch request drive-relative spelling is ambiguous"));
    }
    let name =
        PathBuf::from(path.file_name().ok_or_else(|| error("epoch request needs a filename"))?);
    portable_json(&name, true)?;
    let parent =
        path.parent().filter(|part| !part.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    Ok((
        std::path::absolute(parent).map_err(|_| error("epoch request parent is unavailable"))?,
        name,
    ))
}

/// Validate every original descendant spelling and the conservative portable ASCII output subset.
fn portable_json(path: &Path, filename: bool) -> Result<(), ForgeError> {
    crate::linkage::fresh::validate_relative(path)
        .map_err(|_| error("epoch local path is not a normalized descendant"))?;
    let spelling = path.to_str().ok_or_else(|| error("epoch local path is not UTF8"))?;
    if filename {
        crate::authoring::output::validate_relative(spelling)
            .map_err(|_| error("epoch filename is outside the publisher portable profile"))?;
    }
    if spelling.len() > 65_536
        || spelling.contains(['\\', ':', '?', '#'])
        || spelling.chars().any(char::is_control)
        || path.extension().and_then(|part| part.to_str()) != Some("json")
        || (filename && (path.components().count() != 1 || !spelling.is_ascii()))
        || spelling
            .split('/')
            .any(|part| part.is_empty() || part.ends_with(['.', ' ']) || device(part))
    {
        return Err(error("epoch path must be a portable local JSON name"));
    }
    Ok(())
}

/// Refuse Windows device aliases on every host, without normalizing a caller name.
fn device(part: &str) -> bool {
    let upper = part.split('.').next().unwrap_or_default().to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            upper.strip_prefix(prefix).is_some_and(|tail| {
                matches!(
                    tail,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        })
}

/// Observe only leaf metadata before capture; no atomic absence or publication authority is created.
fn preflight_destination(capture: &CaptureSession, relative: &Path) -> Result<(), ForgeError> {
    match std::fs::symlink_metadata(capture.root().join(relative)) {
        Err(fault) if fault.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) | Err(_) => Err(error("epoch destination exists or is unavailable")),
    }
}

/// Strictly decode complete actual raw JSON under its separate raw/depth/string bounds.
fn strict(bytes: &[u8], limit: usize, depth: usize) -> Result<Value, ForgeError> {
    if bytes.len() > limit {
        return Err(error("epoch raw JSON exceeds its whole bound"));
    }
    json_strict::parse_value(
        bytes,
        "epoch JSON",
        Limits { max_depth: depth, max_string_bytes: 65_536 },
    )
    .map_err(|_| error("epoch JSON is not complete bounded duplicate-free JSON"))
}

/// Admit the closed actual request and all explicitly present optional authoring fields.
fn parse_request(bytes: &[u8]) -> Result<Value, ForgeError> {
    let value = strict(bytes, MAX_REQUEST, 64)?;
    let validator = REQUEST_VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../schemas/forge.assessment-epoch-append-1.schema.json"
            ))
            .map_err(|_| "epoch request schema is malformed")?;
            jsonschema::validator_for(&schema).map_err(|_| "epoch request schema is unavailable")
        })
        .as_ref()
        .map_err(|reason| error(reason))?;
    if !validator.is_valid(&value) {
        return Err(error("epoch request does not satisfy its closed offline schema"));
    }
    closed(
        &value,
        &[
            "schema_version",
            "prior_source",
            "next_epoch",
            "prior_report",
            "seed_families",
            "new_risks",
        ],
    )?;
    if value["schema_version"] != "forge.assessment-epoch-append/1" {
        return Err(error("epoch request version is unsupported"));
    }
    closed(
        &value["next_epoch"],
        &["schema_version", "document", "context", "roles", "parties", "result"],
    )?;
    let next = &value["next_epoch"];
    if next["schema_version"] != "forge.assessment-results/1" {
        return Err(error("epoch next authoring version is unsupported"));
    }
    closed(
        &next["result"],
        &[
            "key",
            "title",
            "description",
            "start",
            "end",
            "control_ids",
            "objective_ids",
            "observations",
            "findings",
            "risks",
            "relationships",
        ],
    )?;
    if next["context"]["evidence_index"] != Value::Null || next["result"]["end"].is_null() {
        return Err(error("epoch next context or sealed end is unsupported"));
    }
    for field in ["seed_families", "new_risks"] {
        for row in array(&value[field], MAX_OBJECTS)? {
            let keys = if field == "seed_families" {
                &["family_key", "risk", "caller_asserted_continuity", "provenance"][..]
            } else {
                &["next_key", "family_key", "prior", "caller_asserted_continuity", "provenance"][..]
            };
            closed(row, keys)?;
            if row["caller_asserted_continuity"] != true || family_key(&row["family_key"]).is_err()
            {
                return Err(error("epoch family assertion is invalid"));
            }
            validate_provenance_shape(&row["provenance"], true)?;
            if field == "seed_families" {
                validate_reference(&row["risk"])?;
            } else {
                text(&row["next_key"])?;
                if !row["prior"].is_null() {
                    validate_reference(&row["prior"])?;
                }
            }
        }
    }
    if !value["prior_report"].is_null() {
        closed(&value["prior_report"], &["artifact", "expected_sha256"])?;
        portable_json(Path::new(text(&value["prior_report"]["artifact"])?), true)?;
        digest(text(&value["prior_report"]["expected_sha256"])?)?;
    }
    Ok(value)
}

/// Capture and strict-admit the actual raw-pinned previous report in the same pool.
fn capture_prior_report(
    descriptor: &Value,
    capture: &mut CaptureSession,
) -> Result<Option<Value>, ForgeError> {
    if descriptor.is_null() {
        return Ok(None);
    }
    let lease = capture
        .required(
            Path::new(text(&descriptor["artifact"])?),
            CaptureRole::AssessmentEpochPriorReport,
            MAX_REPORT as u64,
        )
        .map_err(|_| error("epoch previous report original is unavailable"))?;
    if sha256_hex(lease.bytes()) != text(&descriptor["expected_sha256"])? {
        return Err(error("epoch previous report raw pin differs"));
    }
    let value = epoch_report::parse_raw(lease.bytes())
        .map_err(|_| error("epoch previous report is not closed complete JSON"))?;
    epoch_report::validate_value(&value)
        .map_err(|_| error("epoch previous report shape is invalid"))?;
    Ok(Some(value))
}

/// Deserialize one separately bounded declaration without a whole-Value clone.
fn decode_bounded<T: serde::de::DeserializeOwned>(
    value: &Value,
    limit: usize,
) -> Result<T, ForgeError> {
    let bytes = encode_limited(value, limit, false)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| error("epoch declaration has an invalid closed shape"))
}

/// Require exactly all listed keys; explicit null differs from an omitted field.
fn closed(value: &Value, keys: &[&str]) -> Result<(), ForgeError> {
    let map = value.as_object().ok_or_else(|| error("epoch record must be an object"))?;
    if map.len() != keys.len() || keys.iter().any(|key| !map.contains_key(*key)) {
        return Err(error("epoch record fields are incomplete or unknown"));
    }
    Ok(())
}

/// Borrow a nonempty bounded scalar without normalization or content disclosure.
fn text(value: &Value) -> Result<&str, ForgeError> {
    value
        .as_str()
        .filter(|s| !s.trim().is_empty() && s.len() <= 65_536)
        .ok_or_else(|| error("epoch string is empty, unbounded or invalid"))
}

/// Admit the new exact nonempty family domain without legacy key trimming or normalization.
fn family_key(value: &Value) -> Result<&str, ForgeError> {
    value
        .as_str()
        .filter(|key| !key.is_empty() && key.len() <= 256)
        .ok_or_else(|| error("epoch family key is empty, unbounded or invalid"))
}

/// Borrow a complete bounded array before any row registry grows.
fn array(value: &Value, limit: usize) -> Result<&[Value], ForgeError> {
    value
        .as_array()
        .filter(|rows| rows.len() <= limit)
        .map(Vec::as_slice)
        .ok_or_else(|| error("epoch array exceeds its complete bound or is invalid"))
}

/// Borrow an optional native array while preserving omission in the native oracle.
fn optional_array(value: Option<&Value>, limit: usize) -> Result<&[Value], ForgeError> {
    value.map_or(Ok(&[]), |value| array(value, limit))
}

/// Borrow the exact native envelope root without fabricating a source holder.
fn document(value: &Value) -> Result<&Value, ForgeError> {
    value
        .get("assessment-results")
        .filter(|v| v.is_object())
        .ok_or_else(|| error("epoch native root is missing"))
}

/// Couple the actual selected inventory to its exact full original result.
///
/// This consumes the holder's private borrowed inventory without using its
/// selection to exclude other epochs from preservation or continuity membership.
fn validate_selected_inventory(
    actual: &CapturedEpochSource<'_>,
    declaration: &SourceManifest,
) -> Result<(), ForgeError> {
    let inventory = actual.inventory();
    if inventory.source_sha256 != declaration.assessment_results.expected_sha256
        || inventory.result_uuid != declaration.result.uuid
        || inventory.result_key != declaration.result.key
        || inventory.validation_scope != "source-integrity-only"
        || inventory.workflow_validated
    {
        return Err(error("epoch actual selected inventory identity differs"));
    }
    let native = document(actual.original())?;
    let mut matches = array(&native["results"], MAX_EPOCHS - 1)?
        .iter()
        .filter(|result| result["uuid"].as_str() == Some(inventory.result_uuid.as_str()));
    let result =
        matches.next().ok_or_else(|| error("epoch selected inventory result is absent"))?;
    if matches.next().is_some() || property(result, "stable-key")? != inventory.result_key {
        return Err(error("epoch selected inventory result pair differs"));
    }
    let mut complete = 0;
    for (field, kind) in [("findings", "finding"), ("risks", "risk")] {
        for object in optional_array(result.get(field), MAX_OBJECTS)? {
            complete = add(complete, 1)?;
            let key = property(object, "stable-key")?;
            let mut rows =
                inventory.objects.iter().filter(|row| row.kind.as_str() == kind && row.key == key);
            let row =
                rows.next().ok_or_else(|| error("epoch selected inventory object is absent"))?;
            if rows.next().is_some()
                || object["uuid"].as_str() != Some(row.uuid.as_str())
                || row.result_uuid != inventory.result_uuid
                || row.sha256 != source::canonical_epoch_object_sha256(object)?
            {
                return Err(error("epoch selected inventory complete object tuple differs"));
            }
        }
    }
    if complete != inventory.objects.len() {
        return Err(error("epoch selected inventory has extra object rows"));
    }
    Ok(())
}

/// Borrow the maintained producer's single new result, never selecting a prior newest row.
fn one_result(document: &Value) -> Result<&Value, ForgeError> {
    let rows = array(&document["results"], 1)?;
    rows.first().ok_or_else(|| error("epoch next native has no result"))
}

/// Parse exact RFC3339 instants for chronological comparison, without changing spelling.
fn instant(value: &str) -> Result<DateTime<FixedOffset>, ForgeError> {
    DateTime::parse_from_rfc3339(value).map_err(|_| error("epoch timestamp is invalid"))
}

/// Validate canonical lowercase UUIDs rather than accepting alternate parser spellings.
fn canonical_uuid(value: &str) -> Result<(), ForgeError> {
    if uuid::Uuid::parse_str(value).map(|u| u.hyphenated().to_string()).ok().as_deref()
        != Some(value)
    {
        return Err(error("epoch UUID is not canonical"));
    }
    Ok(())
}

/// Validate the exact complete computed/raw digest grammar.
fn digest(value: &str) -> Result<(), ForgeError> {
    json_strict::validate_lowercase_sha256("epoch digest", value)
        .map_err(|_| error("epoch digest is invalid"))
}

/// Borrow one unique exact current Forge property; absence and duplicate declarations refuse.
fn property<'a>(object: &'a Value, name: &str) -> Result<&'a str, ForgeError> {
    let mut found = None;
    for prop in array(&object["props"], 1_000)? {
        if prop["ns"] == NS
            && prop["name"] == name
            && found.replace(text(&prop["value"])?).is_some()
        {
            return Err(error("epoch native property is duplicated"));
        }
    }
    found.ok_or_else(|| error("epoch native property is missing"))
}

/// Borrow an optional end declaration without inventing a conclusion time.
fn optional_property<'a>(object: &'a Value, name: &str) -> Result<Option<&'a str>, ForgeError> {
    let mut found = None;
    for prop in array(&object["props"], 1_000)? {
        if prop["ns"] == NS
            && prop["name"] == name
            && found.replace(text(&prop["value"])?).is_some()
        {
            return Err(error("epoch native property is duplicated"));
        }
    }
    Ok(found)
}

/// Checked arithmetic shared by every complete row, byte and relationship denominator.
fn add(left: usize, right: usize) -> Result<usize, ForgeError> {
    left.checked_add(right).ok_or_else(|| error("epoch accounting overflow"))
}
/// Checked conservative products, never saturating an admission threshold.
fn mul(left: usize, right: usize) -> Result<usize, ForgeError> {
    left.checked_mul(right).ok_or_else(|| error("epoch accounting overflow"))
}

/// Complete native/next authored row counts, with raw relationship occurrences.
#[derive(Clone, Copy, Default)]
struct GraphCounts {
    /// Complete observation rows, including unselected historical epochs.
    observations: usize,
    /// Complete finding rows, without eligibility filtering.
    findings: usize,
    /// Complete risk rows, regardless of recorded status.
    risks: usize,
    /// Complete all-kind conclusion rows.
    objects: usize,
    /// Complete recursive native/reference occurrences, not unique endpoints.
    occurrences: usize,
}

/// Validate every historical sealed window and complete provenance before retained row maps.
fn validate_epoch_profile(
    old: &Value,
    capture: &mut CaptureSession,
) -> Result<GraphCounts, ForgeError> {
    let rows = array(&old["results"], MAX_EPOCHS - 1)?;
    if rows.is_empty() {
        return Err(error("epoch prior native has no result"));
    }
    let mut counts = GraphCounts::default();
    let mut previous_end = None;
    for result in rows {
        let start = instant(text(&result["start"])?)?;
        let end = instant(text(&result["end"])?)?;
        if start > end || previous_end.is_some_and(|previous| previous > start) {
            return Err(error("epoch prior windows are open, overlapping or reversed"));
        }
        previous_end = Some(end);
        for (field, kind) in
            [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
        {
            let objects = optional_array(result.get(field), MAX_OBJECTS)?;
            counts.objects = add(counts.objects, objects.len())?;
            if counts.objects > MAX_OBJECTS {
                return Err(error("epoch complete conclusion bound exceeded"));
            }
            capture
                .relationships(objects.len())
                .map_err(|_| error("epoch complete relationship budget exceeded"))?;
            match kind {
                "observation" => counts.observations = add(counts.observations, objects.len())?,
                "finding" => counts.findings = add(counts.findings, objects.len())?,
                _ => counts.risks = add(counts.risks, objects.len())?,
            }
            for object in objects {
                validate_native_provenance(old, result, object, kind)?;
                if kind == "observation"
                    && !optional_array(object.get("relevant-evidence"), 1_000)?.is_empty()
                {
                    return Err(error("epoch evidence resources are unsupported"));
                }
            }
        }
    }
    counts.occurrences = occurrences(&old["results"])?;
    Ok(counts)
}

/// Count complete array/reference occurrences before additional graph/report traversal.
fn occurrences(value: &Value) -> Result<usize, ForgeError> {
    let mut count = 0;
    match value {
        Value::Array(rows) => {
            count = rows.len();
            for row in rows {
                count = add(count, occurrences(row)?)?;
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                count = add(count, occurrences(value)?)?;
            }
        }
        _ => {}
    }
    if count > MAX_RELATIONSHIPS {
        return Err(error("epoch complete graph occurrence bound exceeded"));
    }
    Ok(count)
}

/// Precharge complete raw next conclusions/reference occurrences before typed parse/model growth.
fn preflight_next(
    next: &Value,
    old_objects: usize,
    capture: &mut CaptureSession,
) -> Result<GraphCounts, ForgeError> {
    array(&next["roles"], 1_000)?;
    array(&next["parties"], 1_000)?;
    let result = &next["result"];
    let observations = array(&result["observations"], MAX_OBJECTS)?;
    let findings = array(&result["findings"], MAX_OBJECTS)?;
    let risks = array(&result["risks"], MAX_OBJECTS)?;
    let objects = add(add(observations.len(), findings.len())?, risks.len())?;
    if add(old_objects, objects)? > MAX_OBJECTS {
        return Err(error("epoch appended conclusion bound exceeded"));
    }
    for (rows, fields) in [
        (
            observations,
            &[
                "key",
                "title",
                "description",
                "provenance",
                "subjects",
                "task_uuids",
                "evidence_keys",
            ][..],
        ),
        (
            findings,
            &[
                "key",
                "title",
                "description",
                "provenance",
                "target",
                "implementation_statement_uuid",
            ][..],
        ),
        (
            risks,
            &[
                "key",
                "title",
                "description",
                "statement",
                "status",
                "severity",
                "confidence",
                "provenance",
            ][..],
        ),
    ] {
        for row in rows {
            closed(row, fields)?;
            validate_provenance_shape(&row["provenance"], true)?;
        }
    }
    for row in observations {
        if !array(&row["evidence_keys"], 1_000)?.is_empty() {
            return Err(error("epoch next evidence keys are unsupported"));
        }
        array(&row["subjects"], 1_000)?;
        array(&row["task_uuids"], 1_000)?;
    }
    let complete = occurrences(next)?;
    capture
        .relationships(complete)
        .map_err(|_| error("epoch next complete relationship budget exceeded"))?;
    Ok(GraphCounts {
        observations: observations.len(),
        findings: findings.len(),
        risks: risks.len(),
        objects,
        occurrences: complete,
    })
}

/// Validate a complete explicit provenance shape before actor/window contextual checks.
fn validate_provenance_shape(value: &Value, rationale: bool) -> Result<(), ForgeError> {
    let last = if rationale { "rationale" } else { "rationale_sha256" };
    closed(value, &["assessor_key", "role_id", "start", "end", "method", last])?;
    for key in ["assessor_key", "role_id", "start", "method", last] {
        text(&value[key])?;
    }
    if !matches!(text(&value["method"])?, "EXAMINE" | "INTERVIEW" | "TEST" | "UNKNOWN") {
        return Err(error("epoch assessment method is invalid"));
    }
    let start = instant(text(&value["start"])?)?;
    if !value["end"].is_null() && instant(text(&value["end"])?)? < start {
        return Err(error("epoch provenance range is reversed"));
    }
    if !rationale {
        digest(text(&value[last])?)?;
    }
    Ok(())
}

/// Bind actual native provenance to its complete window, role, party and origin/method facts.
fn validate_native_provenance(
    document: &Value,
    result: &Value,
    object: &Value,
    kind: &str,
) -> Result<(), ForgeError> {
    digest(property(object, "content-sha256")?)?;
    let rationale_hash = property(object, "rationale-sha256")?;
    digest(rationale_hash)?;
    if rationale_hash != sha256_hex(property(object, "rationale")?.as_bytes()) {
        return Err(error("epoch native rationale digest differs"));
    }
    let assessor = property(object, "assessor-key")?;
    let start = property(object, "assessment-start")?;
    let end = optional_property(object, "assessment-end")?;
    let method = property(object, "assessment-method")?;
    if !matches!(method, "EXAMINE" | "INTERVIEW" | "TEST" | "UNKNOWN") {
        return Err(error("epoch native assessment method is invalid"));
    }
    let role_party = native_actor(document, assessor)?;
    let origins = array(&object["origins"], 1_000)?;
    let mut actors = 0;
    for origin in origins {
        for actor in array(&origin["actors"], 1_000)? {
            actors = add(actors, 1)?;
            if actor["type"] != "party"
                || text(&actor["actor-uuid"])? != role_party
                || !array(&document["metadata"]["roles"], 1_000)?
                    .iter()
                    .any(|role| role["id"] == actor["role-id"])
            {
                return Err(error("epoch native provenance differs from actor facts"));
            }
        }
    }
    if actors != 1 {
        return Err(error("epoch append requires one explicit origin assessor per conclusion"));
    }
    window(result, start, end)?;
    if kind == "observation" {
        let methods = array(&object["methods"], 1)?;
        if methods.first().and_then(Value::as_str) != Some(method)
            || instant(text(&object["collected"])?)? != instant(start)?
        {
            return Err(error(
                "epoch observation provenance differs from method or collected facts",
            ));
        }
    }
    Ok(())
}

/// Resolve an exact declared assessor stable key through unchanged native metadata.
fn native_actor<'a>(document: &'a Value, key: &str) -> Result<&'a str, ForgeError> {
    let mut matched = None;
    for party in array(&document["metadata"]["parties"], 1_000)? {
        if property(party, "stable-key")? == key && matched.replace(text(&party["uuid"])?).is_some()
        {
            return Err(error("epoch assessor key is duplicated"));
        }
    }
    matched.ok_or_else(|| error("epoch assessor key is absent"))
}

/// Require provenance instants within the sealed result, preserving explicit nullable end.
fn window(result: &Value, start: &str, end: Option<&str>) -> Result<(), ForgeError> {
    let lo = instant(text(&result["start"])?)?;
    let hi = instant(text(&result["end"])?)?;
    let first = instant(start)?;
    let last = end.map(instant).transpose()?.unwrap_or(first);
    if first < lo || last < first || last > hi {
        return Err(error("epoch provenance is outside its sealed window"));
    }
    Ok(())
}

/// Validate complete same-context metadata/receipts before borrowing prior native containers.
fn validate_same_envelope(old: &Value, next: &Value) -> Result<(), ForgeError> {
    if old["uuid"] != next["uuid"]
        || old["import-ap"] != next["import-ap"]
        || old.get("back-matter") != next.get("back-matter")
    {
        return Err(error("epoch native root, import or complete receipts changed"));
    }
    let left =
        old["metadata"].as_object().ok_or_else(|| error("epoch prior metadata is invalid"))?;
    let right =
        next["metadata"].as_object().ok_or_else(|| error("epoch next metadata is invalid"))?;
    if left.len() != right.len()
        || left.iter().any(|(key, value)| {
            !matches!(key.as_str(), "version" | "last-modified") && right.get(key) != Some(value)
        })
    {
        return Err(error("epoch complete unchanged metadata or actors differ"));
    }
    Ok(())
}

/// Apply exact instant ordering to old last window, next sealed window and modified time.
fn validate_append_window(old: &Value, next: &Value) -> Result<(), ForgeError> {
    let last = array(&old["results"], MAX_EPOCHS - 1)?
        .last()
        .ok_or_else(|| error("epoch prior results are empty"))?;
    let new = one_result(next)?;
    let start = instant(text(&new["start"])?)?;
    let end = instant(text(&new["end"])?)?;
    let modified = instant(text(&next["metadata"]["last-modified"])?)?;
    if start > end
        || instant(text(&last["end"])?)? > start
        || modified < end
        || modified < instant(text(&old["metadata"]["last-modified"])?)?
    {
        return Err(error("epoch append sealed windows or modification time are invalid"));
    }
    for (field, kind) in
        [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
    {
        for object in optional_array(new.get(field), MAX_OBJECTS)? {
            validate_native_provenance(next, new, object, kind)?;
        }
    }
    Ok(())
}

/// Scan every prior native UUID and key before any generated new result is appended.
fn check_new_identities(old: &Value, next: &Value, result: &Value) -> Result<(), ForgeError> {
    let mut uuids = BTreeSet::new();
    collect_uuids(old, &mut uuids)?;
    let new_uuid = text(&result["uuid"])?;
    if uuids.contains(new_uuid) {
        return Err(error("epoch appended result UUID collides"));
    }
    let new_key = property(result, "stable-key")?;
    let mut new_ids = BTreeSet::new();
    new_ids.insert(new_uuid);
    for previous in array(&old["results"], MAX_EPOCHS - 1)? {
        if property(previous, "stable-key")? == new_key {
            return Err(error("epoch appended result key is reused"));
        }
    }
    for (field, kind) in
        [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
    {
        for object in optional_array(result.get(field), MAX_OBJECTS)? {
            let uuid = text(&object["uuid"])?;
            if uuids.contains(uuid) || !new_ids.insert(uuid) {
                return Err(error("epoch appended conclusion UUID collides"));
            }
            let key = property(object, "stable-key")?;
            for previous in array(&old["results"], MAX_EPOCHS - 1)? {
                if optional_array(previous.get(field), MAX_OBJECTS)?
                    .iter()
                    .any(|obj| property(obj, "stable-key").ok() == Some(key))
                {
                    return Err(error("epoch appended conclusion kind/key is reused"));
                }
            }
            let _ = kind;
        }
    }
    if old["uuid"] != next["uuid"] {
        return Err(error("epoch document key does not preserve native root"));
    }
    Ok(())
}

/// Collect all complete native UUID fields, including unselected parties and context receipts.
fn collect_uuids<'a>(value: &'a Value, ids: &mut BTreeSet<&'a str>) -> Result<(), ForgeError> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "uuid" {
                    let id = text(value)?;
                    canonical_uuid(id)?;
                    if !ids.insert(id) {
                        return Err(error("epoch prior native UUID collision"));
                    }
                } else {
                    collect_uuids(value, ids)?;
                }
            }
        }
        Value::Array(rows) => {
            for row in rows {
                collect_uuids(row, ids)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Nonretaining/capped native accounting admitted before typed/model/report projection growth.
struct ProjectionAdmission {
    /// Complete typed next compact native ceiling including LF.
    native: usize,
    /// Complete borrowed prior plus next and changed-leaf envelope ceiling.
    envelope: usize,
    /// Complete durable report JSON ceiling including LF.
    json: usize,
    /// Complete explicit stdout delivery ceiling, charged even for borrowed JSON.
    view: usize,
}

/// Consume the reviewed evidence-free N formula and complete generic report serializers.
fn preflight_projection(
    next: &Value,
    prior: &Value,
    source: &SourceManifest,
    report: &ReportMirror<'_>,
    view: Option<epoch_report::EpochViewFormat>,
) -> Result<ProjectionAdmission, ForgeError> {
    let next_manifest_bytes = encoded_count(next, MAX_REQUEST)?;
    let result = &next["result"];
    let conclusions = add(
        add(
            array(&result["observations"], MAX_OBJECTS)?.len(),
            array(&result["findings"], MAX_OBJECTS)?.len(),
        )?,
        array(&result["risks"], MAX_OBJECTS)?.len(),
    )?;
    let records = add(
        add(
            add(add(2, array(&next["roles"], 1000)?.len())?, array(&next["parties"], 1000)?.len())?,
            conclusions,
        )?,
        4,
    )?;
    let mut references = add(
        array(&result["control_ids"], MAX_OBJECTS)?.len(),
        array(&result["objective_ids"], MAX_OBJECTS)?.len(),
    )?;
    references = add(references, array(&result["relationships"], MAX_RELATIONSHIPS)?.len())?;
    references = add(references, conclusions)?;
    for observation in array(&result["observations"], MAX_OBJECTS)? {
        references = add(
            references,
            add(
                array(&observation["subjects"], 1000)?.len(),
                array(&observation["task_uuids"], 1000)?.len(),
            )?,
        )?;
    }
    let mut paths = 0;
    for artifact in [
        &source.context.assessment_plan,
        &source.context.ssp,
        &source.context.profile,
        &source.context.catalog,
    ] {
        paths = add(
            paths,
            artifact
                .artifact
                .to_str()
                .ok_or_else(|| error("epoch context path is not UTF8"))?
                .len(),
        )?;
    }
    // The 8192-per-risk record allowance includes <=1024 bytes for finite f64
    // decimal Display confidence. It is not a 24/32-byte scientific JSON claim.
    let next_native_ceiling = add(
        add(add(mul(6, next_manifest_bytes)?, mul(8192, records)?)?, mul(512, references)?)?,
        mul(18, paths)?,
    )?;
    let report_json_ceiling = epoch_report::count_json(report)
        .map_err(|_| error("epoch complete prospective report exceeds its bound"))?;
    let view_ceiling = epoch_report::count_view(report, view)
        .map_err(|_| error("epoch complete prospective view exceeds its bound"))?;
    if add(mul(3, next_native_ceiling)?, add(report_json_ceiling, view_ceiling)?)?
        > MAX_PROJECTION_BYTES
    {
        return Err(error("epoch shared conservative projection bound exceeded"));
    }
    let prior_native_bytes = encoded_count(prior, MAX_NATIVE)?;
    let old = document(prior)?;
    let mut growth = 0;
    for (old_key, new_key) in [("version", "version"), ("last-modified", "last_modified")] {
        let old_size = encoded_count(&old["metadata"][old_key], MAX_NATIVE)?;
        let new_size = encoded_count(&next["document"][new_key], MAX_REQUEST)?;
        growth = add(growth, new_size.saturating_sub(old_size))?;
    }
    let envelope = add(add(add(prior_native_bytes, next_native_ceiling)?, growth)?, 64)?;
    if envelope > MAX_NATIVE {
        return Err(error("epoch complete native envelope bound exceeded"));
    }
    Ok(ProjectionAdmission {
        native: next_native_ceiling,
        envelope,
        json: report_json_ceiling,
        view: view_ceiling,
    })
}

/// Serialize a complete original borrowing its envelope, metadata and prior result containers.
struct AppendEnvelope<'a> {
    /// Complete actual prior envelope, including unknown admitted native members.
    old: &'a Value,
    /// Actual maintained next document supplying only two metadata leaves.
    next_document: &'a Value,
    /// Actual maintained exactly one new result, appended after the untouched prefix.
    next_result: &'a Value,
}

impl Serialize for AppendEnvelope<'_> {
    /// Preserve all original outer members while forwarding only the admitted native root.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map = self
            .old
            .as_object()
            .ok_or_else(|| serde::ser::Error::custom("invalid epoch envelope"))?;
        let mut out = serializer.serialize_map(Some(map.len()))?;
        for (key, value) in map {
            if key == "assessment-results" {
                out.serialize_entry(
                    key,
                    &AppendDocument {
                        old: value,
                        next: self.next_document,
                        result: self.next_result,
                    },
                )?;
            } else {
                out.serialize_entry(key, value)?;
            }
        }
        out.end()
    }
}

/// Borrow complete prior native document members, replacing only metadata leaves/results.
struct AppendDocument<'a> {
    /// Complete actual prior native document members.
    old: &'a Value,
    /// Actual maintained next native document, never substituted as historical context.
    next: &'a Value,
    /// Exactly one admitted next result.
    result: &'a Value,
}
impl Serialize for AppendDocument<'_> {
    /// Retain every original document field and exact old array order.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map = self
            .old
            .as_object()
            .ok_or_else(|| serde::ser::Error::custom("invalid epoch document"))?;
        let mut out = serializer.serialize_map(Some(map.len()))?;
        for (key, value) in map {
            match key.as_str() {
                "metadata" => out.serialize_entry(
                    key,
                    &AppendMetadata { old: value, next: &self.next["metadata"] },
                )?,
                "results" => {
                    out.serialize_entry(key, &AppendResults { old: value, new: self.result })?;
                }
                _ => out.serialize_entry(key, value)?,
            }
        }
        out.end()
    }
}

/// Borrow exact old metadata, changing only the two explicitly supplied admitted leaves.
struct AppendMetadata<'a> {
    /// Actual original metadata whose complete keys and all other Values are retained.
    old: &'a Value,
    /// Actual maintained metadata providing explicit version and last-modified leaves.
    next: &'a Value,
}
impl Serialize for AppendMetadata<'_> {
    /// Preserve every other metadata Value and its container presence.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map = self
            .old
            .as_object()
            .ok_or_else(|| serde::ser::Error::custom("invalid epoch metadata"))?;
        let mut out = serializer.serialize_map(Some(map.len()))?;
        for (key, value) in map {
            out.serialize_entry(
                key,
                if matches!(key.as_str(), "version" | "last-modified") {
                    &self.next[key]
                } else {
                    value
                },
            )?;
        }
        out.end()
    }
}

/// Borrow every old result Value and append exactly one maintained typed result.
struct AppendResults<'a> {
    /// Complete actual old result array in its original order.
    old: &'a Value,
    /// Actual next maintained result Value, appended once.
    new: &'a Value,
}
impl Serialize for AppendResults<'_> {
    /// Serialize complete history without cloning or rebuilding old conclusions.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let rows = self
            .old
            .as_array()
            .ok_or_else(|| serde::ser::Error::custom("invalid epoch results"))?;
        let mut out = serializer.serialize_seq(Some(rows.len() + 1))?;
        for row in rows {
            out.serialize_element(row)?;
        }
        out.serialize_element(self.new)?;
        out.end()
    }
}

/// Require the complete final decoded tree to reverse to the actual original.
fn reverse_oracle(prior: &Value, final_value: &Value, next: &Value) -> Result<(), ForgeError> {
    let old = document(prior)?;
    let new = document(final_value)?;
    let before = array(&old["results"], MAX_EPOCHS - 1)?;
    let after = array(&new["results"], MAX_EPOCHS)?;
    if after.len() != add(before.len(), 1)?
        || after.last() != Some(next)
        || &after[..before.len()] != before
    {
        return Err(error("epoch historical results changed"));
    }
    for (left, right) in before.iter().zip(after) {
        if source::canonical_epoch_object_sha256(left)?
            != source::canonical_epoch_object_sha256(right)?
        {
            return Err(error("epoch historical result digest changed"));
        }
    }
    let before_map =
        prior.as_object().ok_or_else(|| error("epoch original envelope is invalid"))?;
    let after_map =
        final_value.as_object().ok_or_else(|| error("epoch final envelope is invalid"))?;
    if before_map.len() != after_map.len() {
        return Err(error("epoch envelope container presence changed"));
    }
    for (key, value) in before_map {
        if key != "assessment-results" && after_map.get(key) != Some(value) {
            return Err(error("epoch outer native tree changed"));
        }
    }
    let before_document =
        old.as_object().ok_or_else(|| error("epoch original document is invalid"))?;
    let after_document = new.as_object().ok_or_else(|| error("epoch final document is invalid"))?;
    if before_document.len() != after_document.len() {
        return Err(error("epoch native document container presence changed"));
    }
    for (key, value) in before_document {
        if !matches!(key.as_str(), "metadata" | "results") && after_document.get(key) != Some(value)
        {
            return Err(error("epoch original document tree changed"));
        }
    }
    let before_metadata =
        old["metadata"].as_object().ok_or_else(|| error("epoch original metadata is invalid"))?;
    let after_metadata =
        new["metadata"].as_object().ok_or_else(|| error("epoch final metadata is invalid"))?;
    if before_metadata.len() != after_metadata.len() {
        return Err(error("epoch metadata container presence changed"));
    }
    for (key, value) in before_metadata {
        if !after_metadata.contains_key(key)
            || (!matches!(key.as_str(), "version" | "last-modified")
                && after_metadata.get(key) != Some(value))
        {
            return Err(error("epoch original metadata tree changed"));
        }
    }
    // Complete borrowed virtual reversal: every outer/document/metadata member
    // above is Value-equal after substituting exactly the two old leaves, and
    // every result is Value-equal after removing exactly the last new result.
    // No duplicate complete historical byte buffer or parsed tree is created.
    Ok(())
}

/// Nonretaining compact-byte counter with a strict whole encoding ceiling.
struct CountWriter {
    /// Complete accepted encoded byte count without a retained output buffer.
    count: usize,
    /// Whole preselected ceiling checked before every counter update.
    limit: usize,
}
impl io::Write for CountWriter {
    /// Admit the full next serializer fragment before updating its byte denominator.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.count = self
            .count
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("epoch encoded bound"))?;
        Ok(bytes.len())
    }
    /// No retained bytes or external IO require a flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Capped private byte buffer that refuses before each retained fragment grows.
struct BufferWriter {
    /// Accepted complete serializer fragments; partial results never escape preparation.
    bytes: Vec<u8>,
    /// Already admitted complete ceiling checked before every retained growth.
    limit: usize,
}
impl io::Write for BufferWriter {
    /// Admit complete byte growth before allocating/retaining a serializer fragment.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let n = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("epoch encoded bound"))?;
        self.bytes.reserve(n - self.bytes.len());
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    /// Only an in-memory complete representation is retained.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Count a compact borrowed representation without retaining a JSON buffer.
fn encoded_count<T: Serialize + ?Sized>(value: &T, limit: usize) -> Result<usize, ForgeError> {
    let mut writer = CountWriter { count: 0, limit };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| error("epoch complete encoding exceeds its bound"))?;
    Ok(writer.count)
}

/// Encode only an already-admitted complete representation, with explicit final LF policy.
fn encode_limited<T: Serialize + ?Sized>(
    value: &T,
    limit: usize,
    lf: bool,
) -> Result<Vec<u8>, ForgeError> {
    let mut writer = BufferWriter { bytes: Vec::new(), limit };
    serde_json::to_writer(&mut writer, value)
        .map_err(|_| error("epoch complete encoding failed or exceeded its bound"))?;
    if lf {
        writer.write_all(b"\n").map_err(|_| error("epoch complete encoding exceeds its bound"))?;
    }
    Ok(writer.bytes)
}

/// Fixed redacted failures never contain native prose, author strings or local paths.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(reason.to_string())
}

/// One actual historical risk or one explicit next authored risk; no detached inventory exists.
#[derive(Clone, Copy)]
enum Endpoint<'a> {
    /// Actual original admitted native result/object with complete temporal position.
    Native {
        /// Actual historical containing result Value.
        result: &'a Value,
        /// Actual original risk object Value, including complete declared payload.
        object: &'a Value,
        /// Zero-based actual historical result position.
        position: usize,
    },
    /// Borrowed next authoring risk, resolved against actual typed native before output.
    Authored {
        /// Explicit caller-authored next result declaration.
        result: &'a Value,
        /// Explicit next risk declaration, resolved against actual native before output.
        risk: &'a Value,
        /// Complete prior result count, hence the new zero-based append position.
        position: usize,
    },
}

impl<'a> Endpoint<'a> {
    /// Borrow the exact key within the actual risk-kind domain.
    fn key(&self) -> Result<&'a str, ForgeError> {
        match self {
            Self::Native { object, .. } => property(object, "stable-key"),
            Self::Authored { risk, .. } => text(&risk["key"]),
        }
    }
    /// Return the original or proposed epoch position, never a newest-result selector.
    fn position(&self) -> usize {
        match self {
            Self::Native { position, .. } | Self::Authored { position, .. } => *position,
        }
    }
    /// Borrow the complete associated window for explicit continuity provenance.
    fn result(&self) -> &'a Value {
        match self {
            Self::Native { result, .. } | Self::Authored { result, .. } => result,
        }
    }
}

/// Complete borrowed family endpoints; classification implies no resolution or authority.
struct Family<'a> {
    /// Explicit unnormalized bounded caller family identity.
    key: &'a str,
    /// Exact first historical or explicitly new risk endpoint.
    first: Endpoint<'a>,
    /// Exact latest original or proposed risk endpoint after this append.
    current: Endpoint<'a>,
    /// Explicit next recurrence status only, never an inferred risk disposition.
    continued: bool,
}

/// Proposed recurrence with real latest predecessor, explicit next successor and provenance.
struct NewEdge<'a> {
    /// Existing caller family identity supplied by the classification row.
    family: &'a str,
    /// Actual latest retained native family member.
    predecessor: Endpoint<'a>,
    /// Explicit next authored member, later resolved to actual typed native.
    successor: Endpoint<'a>,
    /// Complete caller assertion; report projects only its digest and fixed provenance facts.
    provenance: &'a Value,
}

/// Privately validated borrowed complete ledger; all maps grow after monotonic relation admission.
struct ContinuityPlan<'a> {
    /// All complete actual pre-append epochs used by endpoint membership and counts.
    old: &'a Value,
    /// Exact optional previous report tree; old edge/provenance/reciprocal order stays intact.
    previous: Option<&'a Value>,
    /// Original complete next document declaration, for explicitly supplied changed leaves.
    next_document: &'a Value,
    /// Complete sorted family registry without cloned keys or source payloads.
    families: BTreeMap<&'a str, Family<'a>>,
    /// Complete sorted new recurrence registry, no implicit matching.
    edges: BTreeMap<&'a str, NewEdge<'a>>,
    /// Exact next risk classification count, including visible zero.
    classified: usize,
}

impl<'a> ContinuityPlan<'a> {
    /// Join actual native tuples to explicit first seed or complete held previous ledger.
    fn new(
        old: &'a Value,
        next: &'a Value,
        request: &'a Value,
        previous: Option<&'a Value>,
        declaration: &SourceManifest,
        capture: &mut CaptureSession,
    ) -> Result<Self, ForgeError> {
        let old_results = array(&old["results"], MAX_EPOCHS - 1)?;
        let seeds = array(&request["seed_families"], MAX_OBJECTS)?;
        let classes = array(&request["new_risks"], MAX_OBJECTS)?;
        let mut plan = Self {
            old,
            previous,
            next_document: &next["document"],
            families: BTreeMap::new(),
            edges: BTreeMap::new(),
            classified: classes.len(),
        };
        capture
            .relationships(add(seeds.len(), classes.len())?)
            .map_err(|_| error("epoch continuity relationship budget exceeded"))?;
        plan.load_families(old_results, seeds, declaration, capture)?;
        let next_risks = array(&next["result"]["risks"], MAX_OBJECTS)?;
        if classes.len() != next_risks.len() {
            return Err(error("epoch next risk classification is incomplete"));
        }
        let mut seen = BTreeSet::new();
        let mut touched = BTreeSet::new();
        for class in classes {
            let next_key = text(&class["next_key"])?;
            if !seen.insert(next_key) {
                return Err(error("epoch next risk is classified more than once"));
            }
            let mut matched =
                next_risks.iter().filter(|risk| risk["key"].as_str() == Some(next_key));
            let risk = matched
                .next()
                .ok_or_else(|| error("epoch classification names an absent next risk"))?;
            if matched.next().is_some() {
                return Err(error("epoch next authored risk key is duplicated"));
            }
            let successor =
                Endpoint::Authored { result: &next["result"], risk, position: old_results.len() };
            validate_continuity_provenance(old, &next["result"], &class["provenance"], true)?;
            let key = family_key(&class["family_key"])?;
            if !touched.insert(key) {
                return Err(error("epoch append branches or merges a family"));
            }
            if class["prior"].is_null() {
                if plan.families.contains_key(key) {
                    return Err(error("epoch new family identity already exists"));
                }
                plan.families.insert(
                    key,
                    Family { key, first: successor, current: successor, continued: false },
                );
            } else {
                let predecessor = resolve_reference(old, &class["prior"])?;
                let family = plan
                    .families
                    .get_mut(key)
                    .ok_or_else(|| error("epoch recurrence family is absent"))?;
                if !same_endpoint(predecessor, family.current)? {
                    return Err(error("epoch recurrence is not the latest exact family member"));
                }
                if predecessor.position() >= successor.position() {
                    return Err(error("epoch recurrence endpoint chronology is invalid"));
                }
                family.current = successor;
                family.continued = true;
                plan.edges.insert(
                    key,
                    NewEdge {
                        family: key,
                        predecessor,
                        successor,
                        provenance: &class["provenance"],
                    },
                );
            }
        }
        if plan.families.len() > MAX_OBJECTS
            || add(plan.previous_edges()?.len(), plan.edges.len())? > MAX_OBJECTS
        {
            return Err(error("epoch complete family or edge bound exceeded"));
        }
        capture
            .relationships(add(
                plan.families.len(),
                mul(3, add(plan.previous_edges()?.len(), plan.edges.len())?)?,
            )?)
            .map_err(|_| {
                error("epoch complete continuity projection relationship budget exceeded")
            })?;
        Ok(plan)
    }

    /// Load all first seeds or prior families after the complete continuity relationship charge.
    fn load_families(
        &mut self,
        old_results: &'a [Value],
        seeds: &'a [Value],
        declaration: &SourceManifest,
        capture: &mut CaptureSession,
    ) -> Result<(), ForgeError> {
        if let Some(report) = self.previous {
            if !seeds.is_empty() {
                return Err(error("epoch previous ledger cannot be reseeded"));
            }
            validate_previous_report(report, self.old, declaration, capture)?;
            for row in array(&report["families"], MAX_OBJECTS)? {
                let key = family_key(&row["family_key"])?;
                let first = resolve_reference(self.old, &row["first"])?;
                let current = resolve_reference(self.old, &row["current"])?;
                if self
                    .families
                    .insert(key, Family { key, first, current, continued: false })
                    .is_some()
                {
                    return Err(error("epoch prior family identity is duplicated"));
                }
            }
            self.validate_previous_edges(report)?;
        } else {
            if old_results.len() != 1 {
                return Err(error("epoch multi-result prior requires its actual previous report"));
            }
            let risks = optional_array(old_results[0].get("risks"), MAX_OBJECTS)?;
            if seeds.len() != risks.len() {
                return Err(error("epoch initial risk seeds are incomplete"));
            }
            let mut covered = BTreeSet::new();
            for row in seeds {
                let endpoint = resolve_reference(self.old, &row["risk"])?;
                if !covered.insert(endpoint.key()?) {
                    return Err(error("epoch initial risk is seeded more than once"));
                }
                validate_continuity_provenance(
                    self.old,
                    endpoint.result(),
                    &row["provenance"],
                    true,
                )?;
                let key = family_key(&row["family_key"])?;
                if self
                    .families
                    .insert(
                        key,
                        Family { key, first: endpoint, current: endpoint, continued: false },
                    )
                    .is_some()
                {
                    return Err(error("epoch initial family identity is duplicated"));
                }
            }
        }
        Ok(())
    }

    /// Borrow the exact previous edge prefix or its explicit empty initial history.
    fn previous_edges(&self) -> Result<&[Value], ForgeError> {
        self.previous.map_or(Ok(&[]), |r| array(&r["continuity_edges"], MAX_OBJECTS))
    }

    /// Replay every prior family edge against actual all-epoch tuples and reciprocal rows.
    fn validate_previous_edges(&self, report: &Value) -> Result<(), ForgeError> {
        let mut heads: BTreeMap<_, _> =
            self.families.iter().map(|(key, family)| (*key, family.first)).collect();
        let mut assigned = BTreeSet::new();
        for family in self.families.values() {
            if !assigned.insert(reference_identity(family.first)?) {
                return Err(error("epoch risk is first member of multiple families"));
            }
        }
        let edges = array(&report["continuity_edges"], MAX_OBJECTS)?;
        let reciprocal = array(&report["reciprocal_rows"], MAX_OBJECTS * 2)?;
        if reciprocal.len() != mul(2, edges.len())? {
            return Err(error("epoch prior reciprocal row count differs"));
        }
        let mut hashes = BTreeSet::new();
        let mut previous_position = 0;
        for edge in edges {
            let family = family_key(&edge["family_key"])?;
            let predecessor = resolve_reference(self.old, &edge["predecessor"])?;
            let successor = resolve_reference(self.old, &edge["successor"])?;
            let head =
                heads.get_mut(family).ok_or_else(|| error("epoch prior edge family is absent"))?;
            if !same_endpoint(*head, predecessor)?
                || predecessor.position() >= successor.position()
                || successor.position() < previous_position
                || !assigned.insert(reference_identity(successor)?)
            {
                return Err(error(
                    "epoch prior edges branch, cycle or differ from latest membership",
                ));
            }
            previous_position = successor.position();
            validate_continuity_provenance(
                self.old,
                successor.result(),
                &edge["provenance"],
                false,
            )?;
            let hash = continuity_hash(
                family,
                &reference(predecessor, None)?,
                &reference(successor, None)?,
                &provenance_row(&edge["provenance"], false)?,
            )?;
            if edge["edge_sha256"].as_str() != Some(&hash) || !hashes.insert(hash.clone()) {
                return Err(error("epoch prior continuity digest is duplicated or differs"));
            }
            let expected = [
                ("predecessor", &edge["predecessor"], &edge["successor"]),
                ("successor", &edge["successor"], &edge["predecessor"]),
            ];
            for (direction, from, to) in expected {
                let found = reciprocal
                    .iter()
                    .filter(|row| {
                        row["family_key"] == edge["family_key"]
                            && row["edge_sha256"] == edge["edge_sha256"]
                            && row["direction"] == direction
                            && &row["from"] == from
                            && &row["to"] == to
                    })
                    .count();
                if found != 1 {
                    return Err(error("epoch prior reciprocal endpoints differ"));
                }
            }
            *head = successor;
        }
        let mut risk_count = 0;
        for result in array(&self.old["results"], MAX_EPOCHS - 1)? {
            risk_count = add(risk_count, optional_array(result.get("risks"), MAX_OBJECTS)?.len())?;
        }
        if assigned.len() != risk_count {
            return Err(error("epoch prior ledger does not cover every actual risk"));
        }
        for (key, family) in &self.families {
            if !same_endpoint(heads[key], family.current)? {
                return Err(error(
                    "epoch prior family current is not its reconstructed latest member",
                ));
            }
        }
        Ok(())
    }

    /// Resolve every proposed successor to the actual maintained typed next result.
    fn validate_actual_next(&self, old: &Value, next: &Value) -> Result<(), ForgeError> {
        if optional_array(next.get("risks"), MAX_OBJECTS)?.len() != self.classified {
            return Err(error("epoch typed next risk denominator differs"));
        }
        for family in self.families.values() {
            if let Endpoint::Authored { risk, .. } = family.current {
                let actual = find_native_risk(next, text(&risk["key"])?)?;
                if text(&actual["status"])? != text(&risk["status"])? {
                    return Err(error("epoch typed risk status differs from caller assertion"));
                }
            }
        }
        for edge in self.edges.values() {
            validate_continuity_provenance(old, next, edge.provenance, true)?;
        }
        Ok(())
    }
}

/// Validate a complete risk reference without coercing kind, key, UUID or digest.
fn validate_reference(value: &Value) -> Result<(), ForgeError> {
    closed(value, &["kind", "key", "uuid", "result_uuid", "expected_sha256"])?;
    if value["kind"] != "risk" {
        return Err(error("epoch continuity endpoint kind is not risk"));
    }
    text(&value["key"])?;
    canonical_uuid(text(&value["uuid"])?)?;
    canonical_uuid(text(&value["result_uuid"])?)?;
    digest(text(&value["expected_sha256"])?)?;
    Ok(())
}

/// Resolve a complete exact supplied endpoint through all actual original epochs.
fn resolve_reference<'a>(document: &'a Value, value: &Value) -> Result<Endpoint<'a>, ForgeError> {
    validate_reference(value)?;
    for (position, result) in array(&document["results"], MAX_EPOCHS - 1)?.iter().enumerate() {
        if result["uuid"] == value["result_uuid"] {
            let object = find_native_risk(result, text(&value["key"])?)?;
            if object["uuid"] != value["uuid"]
                || source::canonical_epoch_object_sha256(object)?
                    != text(&value["expected_sha256"])?
            {
                return Err(error("epoch endpoint differs from complete actual native tuple"));
            }
            return Ok(Endpoint::Native { result, object, position });
        }
    }
    Err(error("epoch endpoint result is absent from actual source"))
}

/// Borrow one exact actual risk key, refusing duplicate/absent matches.
fn find_native_risk<'a>(result: &'a Value, key: &str) -> Result<&'a Value, ForgeError> {
    let mut found = None;
    for object in optional_array(result.get("risks"), MAX_OBJECTS)? {
        if property(object, "stable-key")? == key && found.replace(object).is_some() {
            return Err(error("epoch native risk key is duplicated"));
        }
    }
    found.ok_or_else(|| error("epoch risk key is absent from actual source"))
}

/// Compare complete actual endpoint generation content, never asserted digest properties.
fn same_endpoint(left: Endpoint<'_>, right: Endpoint<'_>) -> Result<bool, ForgeError> {
    match (left, right) {
        (
            Endpoint::Native { result: a, object: b, .. },
            Endpoint::Native { result: c, object: d, .. },
        ) => Ok(a["uuid"] == c["uuid"]
            && b["uuid"] == d["uuid"]
            && property(b, "stable-key")? == property(d, "stable-key")?
            && source::canonical_epoch_object_sha256(b)?
                == source::canonical_epoch_object_sha256(d)?),
        _ => Ok(false),
    }
}

/// Borrow a registry identity without retaining object prose or computed hash authority.
fn reference_identity(endpoint: Endpoint<'_>) -> Result<(&str, &str), ForgeError> {
    match endpoint {
        Endpoint::Native { result, object, .. } => {
            Ok((text(&result["uuid"])?, text(&object["uuid"])?))
        }
        Endpoint::Authored { .. } => Err(error("epoch previous ledger contains a future endpoint")),
    }
}

/// Bind caller/report provenance to exact unchanged actor/role and the selected sealed window.
fn validate_continuity_provenance(
    document: &Value,
    result: &Value,
    provenance: &Value,
    rationale: bool,
) -> Result<(), ForgeError> {
    validate_provenance_shape(provenance, rationale)?;
    native_actor(document, text(&provenance["assessor_key"])?)?;
    if !array(&document["metadata"]["roles"], 1000)?
        .iter()
        .any(|role| role["id"] == provenance["role_id"])
    {
        return Err(error("epoch continuity role is absent"));
    }
    window(result, text(&provenance["start"])?, provenance["end"].as_str())
}

/// Crosscheck complete previous report identity, native row tuples, context and after counts.
fn validate_previous_report(
    report: &Value,
    old: &Value,
    declaration: &SourceManifest,
    capture: &mut CaptureSession,
) -> Result<(), ForgeError> {
    if report["native"]["raw_sha256"] != declaration.assessment_results.expected_sha256
        || report["native"]["uuid"] != old["uuid"]
        || report["native"]["artifact"].as_str() != declaration.assessment_results.artifact.to_str()
        || report["native"]["document_version"] != old["metadata"]["version"]
        || report["native"]["oscal_version"] != old["metadata"]["oscal-version"]
    {
        return Err(error(
            "epoch previous report does not bind the actual prior native generation",
        ));
    }
    let results = array(&old["results"], MAX_EPOCHS - 1)?;
    let rows = array(&report["epochs"], MAX_EPOCHS)?;
    if results.len() != rows.len() || results.len() < 2 {
        return Err(error("epoch previous report epoch denominator differs"));
    }
    let objects = array(&report["objects"], MAX_OBJECTS)?;
    capture
        .relationships(add(add(rows.len(), objects.len())?, occurrences(report)?)?)
        .map_err(|_| error("epoch previous report complete relationship budget exceeded"))?;
    let mut count = GraphCounts::default();
    let mut indexed = BTreeSet::new();
    for (position, (result, row)) in results.iter().zip(rows).enumerate() {
        if row["position"].as_u64() != Some(position as u64)
            || row["uuid"] != result["uuid"]
            || row["key"].as_str() != Some(property(result, "stable-key")?)
            || row["canonical_sha256"].as_str()
                != Some(&source::canonical_epoch_object_sha256(result)?)
            || row["start"] != result["start"]
            || row["end"] != result["end"]
        {
            return Err(error("epoch previous report epoch facts differ from complete native"));
        }
        for (field, kind) in
            [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
        {
            let native_rows = optional_array(result.get(field), MAX_OBJECTS)?;
            if row[field].as_u64() != Some(native_rows.len() as u64) {
                return Err(error("epoch previous report result object denominator differs"));
            }
            match kind {
                "observation" => count.observations = add(count.observations, native_rows.len())?,
                "finding" => count.findings = add(count.findings, native_rows.len())?,
                _ => count.risks = add(count.risks, native_rows.len())?,
            }
            validate_previous_object_rows(result, native_rows, objects, kind, &mut indexed)?;
        }
    }
    count.objects = add(add(count.observations, count.findings)?, count.risks)?;
    if count.objects != objects.len() {
        return Err(error("epoch previous report has extraneous object rows"));
    }
    for (key, n) in [
        ("after_epochs", results.len()),
        ("after_observations", count.observations),
        ("after_findings", count.findings),
        ("after_risks", count.risks),
        ("after_objects", count.objects),
    ] {
        if report["counts"][key].as_u64() != Some(n as u64) {
            return Err(error("epoch previous report after denominator differs"));
        }
    }
    let last = results.last().ok_or_else(|| error("epoch previous report native is empty"))?;
    for (key, field, total) in [
        ("before_observations", "observations", count.observations),
        ("before_findings", "findings", count.findings),
        ("before_risks", "risks", count.risks),
    ] {
        let before = total
            .checked_sub(optional_array(last.get(field), MAX_OBJECTS)?.len())
            .ok_or_else(|| error("epoch previous report count underflow"))?;
        if report["counts"][key].as_u64() != Some(before as u64) {
            return Err(error("epoch previous report before denominator differs"));
        }
    }
    if report["counts"]["before_epochs"].as_u64() != Some((results.len() - 1) as u64)
        || report["counts"]["appended_epochs"] != 1
        || report["counts"]["preserved_epochs"] != report["counts"]["before_epochs"]
    {
        return Err(error("epoch previous report preservation counts differ"));
    }
    validate_context_rows(&report["context"], declaration)?;
    validate_previous_complete_denominators(report, old)?;
    Ok(())
}

/// Join every native conclusion to exactly one complete prior-report object tuple.
fn validate_previous_object_rows<'a>(
    result: &'a Value,
    native_rows: &'a [Value],
    objects: &[Value],
    kind: &'static str,
    indexed: &mut BTreeSet<(&'a str, &'static str, &'a str)>,
) -> Result<(), ForgeError> {
    for object in native_rows {
        let key = property(object, "stable-key")?;
        let mut matched = objects.iter().filter(|r| {
            r["result_uuid"] == result["uuid"]
                && r["kind"] == kind
                && r["key"].as_str() == Some(key)
        });
        let r =
            matched.next().ok_or_else(|| error("epoch previous report object coverage differs"))?;
        if matched.next().is_some() {
            return Err(error("epoch previous report object coverage differs"));
        }
        if r["uuid"] != object["uuid"]
            || r["result_key"].as_str() != Some(property(result, "stable-key")?)
            || r["computed_sha256"].as_str()
                != Some(&source::canonical_epoch_object_sha256(object)?)
            || r["declared_content_sha256"].as_str() != Some(property(object, "content-sha256")?)
            || r["declared_rationale_sha256"].as_str()
                != Some(property(object, "rationale-sha256")?)
            || r["status"].as_str() != object_status(object, kind)
            || !indexed.insert((text(&result["uuid"])?, kind, key))
        {
            return Err(error("epoch previous report complete object tuple differs"));
        }
    }
    Ok(())
}

/// Reconcile every23 stored count with the native/ledger facts actually available now.
///
/// The previous-before raw file is not captured: its stored raw hash remains an
/// identity-only descriptor. Optional metadata old values are asserted history,
/// while their new values, roster/cardinality and all native endpoints are checked.
fn validate_previous_complete_denominators(report: &Value, old: &Value) -> Result<(), ForgeError> {
    let results = array(&old["results"], MAX_EPOCHS - 1)?;
    let last =
        results.last().ok_or_else(|| error("epoch previous report has no last native result"))?;
    let last_uuid = text(&last["uuid"])?;
    let families = array(&report["families"], MAX_OBJECTS)?;
    let edges = array(&report["continuity_edges"], MAX_OBJECTS)?;
    let reciprocal = array(&report["reciprocal_rows"], MAX_OBJECTS * 2)?;
    let changes = array(&report["changes"], MAX_RELATIONSHIPS)?;
    let mut added = 0;
    for field in ["observations", "findings", "risks"] {
        added = add(added, optional_array(last.get(field), MAX_OBJECTS)?.len())?;
    }
    let after = array(&report["objects"], MAX_OBJECTS)?.len();
    let before = after
        .checked_sub(added)
        .ok_or_else(|| error("epoch prior object denominator underflow"))?;
    let mut uncontinued = 0;
    for family in families {
        let current = resolve_reference(old, &family["current"])?;
        let expected = edges.iter().any(|edge| {
            edge["family_key"] == family["family_key"]
                && edge["successor"]["result_uuid"].as_str() == Some(last_uuid)
        });
        if family["continued_in_append"] != expected {
            return Err(error("epoch previous family append marker differs"));
        }
        if current.position() < results.len() - 1 {
            uncontinued = add(uncontinued, 1)?;
        }
    }
    let graph_after = occurrences(&old["results"])?;
    let mut graph_before = results.len() - 1;
    for result in &results[..results.len() - 1] {
        graph_before = add(graph_before, occurrences(result)?)?;
    }
    let denominators = [
        ("before_objects", before),
        ("preserved_objects", before),
        ("added_objects", added),
        ("native_graph_occurrences_before", graph_before),
        ("native_graph_occurrences_after", graph_after),
        ("families", families.len()),
        ("continuity_edges", edges.len()),
        ("reciprocal_rows", reciprocal.len()),
        ("classified_next_risks", optional_array(last.get("risks"), MAX_OBJECTS)?.len()),
        ("uncontinued_families", uncontinued),
        ("captured_original_generations", if results.len() == 2 { 6 } else { 7 }),
        ("review_actions", changes.len()),
    ];
    for (key, n) in denominators {
        if report["counts"][key].as_u64() != Some(n as u64) {
            return Err(error("epoch previous complete report denominator differs"));
        }
    }
    for (position, row) in array(&report["epochs"], MAX_EPOCHS)?.iter().enumerate() {
        let expected = if position + 1 == results.len() { "appended" } else { "preserved" };
        if row["classification"] != expected {
            return Err(error("epoch previous epoch classification differs"));
        }
    }
    for row in array(&report["objects"], MAX_OBJECTS)? {
        let expected =
            if row["result_uuid"].as_str() == Some(last_uuid) { "added" } else { "preserved" };
        if row["classification"] != expected {
            return Err(error("epoch previous object classification differs"));
        }
    }
    if report["prior_native"]["uuid"] != old["uuid"]
        || report["review_required"] != true
        || report["status"] != "review-required"
        || changes.is_empty()
    {
        return Err(error("epoch previous report status or identity scope differs"));
    }
    digest(text(&report["prior_native"]["raw_sha256"])?)?;
    validate_previous_changes(changes, old, families, edges)?;
    Ok(())
}

/// Require the exact reconstructable descriptive action roster, admitting no extra/missing rows.
fn validate_previous_changes(
    changes: &[Value],
    old: &Value,
    families: &[Value],
    edges: &[Value],
) -> Result<(), ForgeError> {
    let results = array(&old["results"], MAX_EPOCHS - 1)?;
    let last = results.last().ok_or_else(|| error("epoch previous change native is empty"))?;
    let mut expected = 0;
    let mut require = |row: ChangeRow<'_>| {
        if changes.iter().filter(|value| change_matches(value, &row)).count() != 1 {
            return Err(error("epoch previous descriptive change roster differs"));
        }
        expected = add(expected, 1)?;
        Ok(())
    };
    require(ChangeRow {
        code: "epoch-added",
        locator: None,
        family_key: None,
        old: None,
        new: Some(Cow::Borrowed(property(last, "stable-key")?)),
    })?;
    for (field, kind) in
        [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
    {
        for object in optional_array(last.get(field), MAX_OBJECTS)? {
            require(ChangeRow {
                code: "object-added",
                locator: Some(locator(last, object, kind)?),
                family_key: None,
                old: None,
                new: None,
            })?;
        }
    }
    for family in families {
        let first = resolve_reference(old, &family["first"])?;
        if first.position() == results.len() - 1 {
            require(ChangeRow {
                code: "risk-family-created",
                locator: None,
                family_key: Some(family_key(&family["family_key"])?),
                old: None,
                new: None,
            })?;
        }
    }
    for edge in edges {
        let predecessor = resolve_reference(old, &edge["predecessor"])?;
        let successor = resolve_reference(old, &edge["successor"])?;
        if successor.position() != results.len() - 1 {
            continue;
        }
        let family = family_key(&edge["family_key"])?;
        require(ChangeRow {
            code: "risk-continuity-recorded",
            locator: None,
            family_key: Some(family),
            old: Some(Cow::Borrowed(text(&edge["predecessor"]["uuid"])?)),
            new: Some(Cow::Borrowed(text(&edge["successor"]["uuid"])?)),
        })?;
        let (Endpoint::Native { object: before, .. }, Endpoint::Native { object: after, .. }) =
            (predecessor, successor)
        else {
            return Err(error("epoch previous changes contain a future endpoint"));
        };
        for (code, prop) in [
            ("recurrence-content-changed", "content-sha256"),
            ("recurrence-rationale-changed", "rationale-sha256"),
        ] {
            let a = property(before, prop)?;
            let b = property(after, prop)?;
            if a != b {
                require(ChangeRow {
                    code,
                    locator: None,
                    family_key: Some(family),
                    old: Some(Cow::Borrowed(a)),
                    new: Some(Cow::Borrowed(b)),
                })?;
            }
        }
        let a = text(&before["status"])?;
        let b = text(&after["status"])?;
        if a != b {
            require(ChangeRow {
                code: "recurrence-status-changed",
                locator: None,
                family_key: Some(family),
                old: Some(Cow::Borrowed(a)),
                new: Some(Cow::Borrowed(b)),
            })?;
        }
    }
    drop(require);
    validate_previous_metadata_changes(changes, old, expected)
}

/// Check asserted metadata leaves and final roster cardinality after all native actions.
fn validate_previous_metadata_changes(
    changes: &[Value],
    old: &Value,
    mut expected: usize,
) -> Result<(), ForgeError> {
    // Previous-before metadata bytes are not available. The stored old leaf is
    // an explicit assertion; the new leaf must equal the actual current native.
    for (code, key) in [
        ("document-version-changed", "version"),
        ("document-modified-time-changed", "last-modified"),
    ] {
        let mut rows = changes.iter().filter(|row| row["code"] == code);
        if let Some(row) = rows.next() {
            if rows.next().is_some()
                || !row["locator"].is_null()
                || !row["family_key"].is_null()
                || row["new"] != old["metadata"][key]
                || text(&row["old"])? == text(&row["new"])?
            {
                return Err(error("epoch previous asserted metadata change is incoherent"));
            }
            if key == "last-modified" && instant(text(&row["old"])?)? > instant(text(&row["new"])?)?
            {
                return Err(error("epoch previous asserted modified-time change is reversed"));
            }
            expected = add(expected, 1)?;
        }
    }
    if expected != changes.len() {
        return Err(error("epoch previous report has extraneous or incomplete changes"));
    }
    Ok(())
}

/// Compare a complete borrowed expected action without cloning a large report row.
fn change_matches(value: &Value, row: &ChangeRow<'_>) -> bool {
    if value["code"] != row.code
        || value["family_key"].as_str() != row.family_key
        || value["old"].as_str() != row.old.as_deref()
        || value["new"].as_str() != row.new.as_deref()
    {
        return false;
    }
    match &row.locator {
        None => value["locator"].is_null(),
        Some(locator) => {
            let v = &value["locator"];
            v["result_key"] == locator.result_key
                && v["result_uuid"] == locator.result_uuid
                && v["kind"] == locator.kind
                && v["key"] == locator.key
                && v["uuid"] == locator.uuid
        }
    }
}

/// Require the complete four retained companion identity rows, with no inferred context union.
fn validate_context_rows(rows: &Value, source: &SourceManifest) -> Result<(), ForgeError> {
    let rows = array(rows, 4)?;
    if rows.len() != 4 {
        return Err(error("epoch previous report context count differs"));
    }
    for (kind, artifact) in [
        ("assessment-plan", &source.context.assessment_plan),
        ("system-security-plan", &source.context.ssp),
        ("profile", &source.context.profile),
        ("catalog", &source.context.catalog),
    ] {
        if rows
            .iter()
            .filter(|row| {
                row["kind"] == kind
                    && row["sha256"] == artifact.expected_sha256
                    && row["root_uuid"] == artifact.root_uuid
                    && row["document_version"] == artifact.document_version
                    && row["oscal_version"] == artifact.oscal_version
            })
            .count()
            != 1
        {
            return Err(error("epoch previous report complete context pin differs"));
        }
    }
    Ok(())
}

/// Borrow the actual recorded status domain while retaining observation null.
fn object_status<'a>(object: &'a Value, kind: &str) -> Option<&'a str> {
    match kind {
        "risk" => object["status"].as_str(),
        "finding" => object.pointer("/target/status/state").and_then(Value::as_str),
        _ => None,
    }
}

/// Full minimized endpoint wire value, using fixed-width surrogates only for prospective counting.
#[derive(Serialize)]
struct ReferenceRow<'a> {
    /// Fixed risk domain; a finding or observation cannot substitute for this tuple.
    kind: &'static str,
    /// Exact native or explicit next risk stable key.
    key: &'a str,
    /// Actual native object UUID, or a count-only fixed-width surrogate.
    uuid: Cow<'a, str>,
    /// Actual containing native result UUID, or a count-only surrogate.
    result_uuid: Cow<'a, str>,
    /// Computed complete object digest, or a count-only surrogate.
    expected_sha256: Cow<'a, str>,
}

/// Resolve a borrowed endpoint to either actual native fields or a bounded accounting surrogate.
fn reference<'a>(
    endpoint: Endpoint<'a>,
    next: Option<&'a Value>,
) -> Result<ReferenceRow<'a>, ForgeError> {
    match endpoint {
        Endpoint::Native { result, object, .. } => Ok(ReferenceRow {
            kind: "risk",
            key: property(object, "stable-key")?,
            uuid: Cow::Borrowed(text(&object["uuid"])?),
            result_uuid: Cow::Borrowed(text(&result["uuid"])?),
            expected_sha256: Cow::Owned(source::canonical_epoch_object_sha256(object)?),
        }),
        Endpoint::Authored { risk, .. } => {
            let key = text(&risk["key"])?;
            if let Some(result) = next {
                let actual = find_native_risk(result, key)?;
                Ok(ReferenceRow {
                    kind: "risk",
                    key,
                    uuid: Cow::Borrowed(text(&actual["uuid"])?),
                    result_uuid: Cow::Borrowed(text(&result["uuid"])?),
                    expected_sha256: Cow::Owned(source::canonical_epoch_object_sha256(actual)?),
                })
            } else {
                Ok(ReferenceRow {
                    kind: "risk",
                    key,
                    uuid: Cow::Borrowed(UUID),
                    result_uuid: Cow::Borrowed(UUID),
                    expected_sha256: Cow::Borrowed(HASH),
                })
            }
        }
    }
}

/// Minimized explicitly asserted provenance, with rationale content omitted.
#[derive(Serialize)]
struct ProvenanceRow<'a> {
    /// Exact explicitly declared assessor stable key, not an authenticated principal.
    assessor_key: &'a str,
    /// Exact retained role identifier containing the declared actor origin.
    role_id: &'a str,
    /// Original exact start spelling whose instant fits the selected sealed window.
    start: &'a str,
    /// Explicit nullable end; absence never creates a completion assertion.
    end: Option<&'a str>,
    /// Explicit EXAMINE, INTERVIEW or TEST method, without execution credit.
    method: &'a str,
    /// Digest of exact asserted rationale bytes; rationale text is omitted.
    rationale_sha256: Cow<'a, str>,
}

/// Borrow every preserved metadata fact and hash actual new rationale bytes only.
fn provenance_row(value: &Value, rationale: bool) -> Result<ProvenanceRow<'_>, ForgeError> {
    Ok(ProvenanceRow {
        assessor_key: text(&value["assessor_key"])?,
        role_id: text(&value["role_id"])?,
        start: text(&value["start"])?,
        end: value["end"].as_str(),
        method: text(&value["method"])?,
        rationale_sha256: if rationale {
            Cow::Owned(sha256_hex(text(&value["rationale"])?.as_bytes()))
        } else {
            Cow::Borrowed(text(&value["rationale_sha256"])?)
        },
    })
}

/// Hash exact framed endpoint/provenance bytes; this is content identity, not UUID or approval.
fn continuity_hash(
    family: &str,
    predecessor: &ReferenceRow<'_>,
    successor: &ReferenceRow<'_>,
    provenance: &ProvenanceRow<'_>,
) -> Result<String, ForgeError> {
    let mut hash = Sha256::new();
    framed(&mut hash, "forge.assessment-epoch-continuity/1")?;
    framed(&mut hash, family)?;
    for row in [predecessor, successor] {
        for value in [
            row.kind,
            row.key,
            row.uuid.as_ref(),
            row.result_uuid.as_ref(),
            row.expected_sha256.as_ref(),
        ] {
            framed(&mut hash, value)?;
        }
    }
    for value in [provenance.assessor_key, provenance.role_id, provenance.start] {
        framed(&mut hash, value)?;
    }
    match provenance.end {
        None => hash.update([0]),
        Some(end) => {
            hash.update([1]);
            framed(&mut hash, end)?;
        }
    }
    framed(&mut hash, provenance.method)?;
    framed(&mut hash, &provenance.rationale_sha256)?;
    Ok(crate::hashing::lower_hex(&hash.finalize()))
}

/// Feed one exact UTF8 scalar with checked u64 big-endian length framing.
fn framed(hash: &mut Sha256, value: &str) -> Result<(), ForgeError> {
    hash.update(
        u64::try_from(value.len())
            .map_err(|_| error("epoch continuity framing overflow"))?
            .to_be_bytes(),
    );
    hash.update(value.as_bytes());
    Ok(())
}

/// Either raw authored fields for prospective counting or actual typed native fields for output.
#[derive(Clone, Copy)]
enum Next<'a> {
    /// No object UUID/hash authority: fixed-width surrogates are used only by serializers' counters.
    Authored(
        /// Complete caller-authored result used only by prospective counting and later native resolution.
        &'a Value,
    ),
    /// The real maintained constructor's fully validated complete single-result Value.
    Native(
        /// Actual validated newly built native result used by the completed projection.
        &'a Value,
    ),
}

impl<'a> Next<'a> {
    /// Borrow the actual typed result only when actual native validation has completed.
    fn actual(self) -> Option<&'a Value> {
        match self {
            Self::Native(v) => Some(v),
            Self::Authored(_) => None,
        }
    }
    /// Borrow result stable key without guessing it from an identity.
    fn key(self) -> Result<&'a str, ForgeError> {
        match self {
            Self::Authored(v) => text(&v["key"]),
            Self::Native(v) => property(v, "stable-key"),
        }
    }
}

/// A streaming complete report mirror; only bounded registries retain borrowed endpoints.
struct ReportMirror<'a> {
    /// Actual complete original native root, including every historical result.
    old: &'a Value,
    /// Count-only authoring or actual fully validated new result.
    next: Next<'a>,
    /// Complete privately validated family/classification ledger.
    plan: &'a ContinuityPlan<'a>,
    /// Exact actual five-source declaration, not an independently reconstructed pin map.
    source: &'a SourceManifest,
    /// Explicit portable native output filename.
    output: &'a str,
    /// Actual raw final native SHA or count-only fixed-width hash.
    raw_hash: &'a str,
    /// Explicit caller next document version.
    next_version: &'a str,
    /// Complete all-kind historical counts.
    old_counts: GraphCounts,
    /// Complete next row denominators, distinct from reference occurrences.
    next_counts: GraphCounts,
    /// Actual required six/seven original generations, never deduplicated report occurrences.
    generations: usize,
}

/// Serialize one complete fixed root and streaming row arrays without DTO clones.
impl Serialize for ReportMirror<'_> {
    /// The exact same borrowed mirror feeds all prospective count and final JSON serializers.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(17))?;
        map.serialize_entry("schema_version", "forge.assessment-epoch-report/1")?;
        map.serialize_entry("validation_scope", "same-context-sealed-epoch-append")?;
        map.serialize_entry("status", "review-required")?;
        map.serialize_entry("review_required", &true)?;
        map.serialize_entry("assessment_authority", &false)?;
        map.serialize_entry("continuity_authority", "caller-asserted")?;
        map.serialize_entry(
            "prior_native",
            &PriorIdentity {
                uuid: text(&self.old["uuid"]).map_err(serde::ser::Error::custom)?,
                raw_sha256: &self.source.assessment_results.expected_sha256,
            },
        )?;
        map.serialize_entry(
            "native",
            &NativeIdentity {
                artifact: self.output,
                uuid: text(&self.old["uuid"]).map_err(serde::ser::Error::custom)?,
                document_version: self.next_version,
                oscal_version: &self.source.assessment_results.oscal_version,
                raw_sha256: self.raw_hash,
            },
        )?;
        map.serialize_entry("context", &Rows { mirror: self, kind: RowKind::Context })?;
        map.serialize_entry("counts", &CountRows { mirror: self })?;
        for (key, kind) in [
            ("epochs", RowKind::Epochs),
            ("objects", RowKind::Objects),
            ("families", RowKind::Families),
            ("continuity_edges", RowKind::Edges),
            ("reciprocal_rows", RowKind::Reciprocal),
            ("changes", RowKind::Changes),
        ] {
            map.serialize_entry(key, &Rows { mirror: self, kind })?;
        }
        map.serialize_entry("trust_boundary", TRUST_BOUNDARY)?;
        map.end()
    }
}

/// Exact identity-only previous native descriptor; stored prior raw generations are not recaptured.
#[derive(Serialize)]
struct PriorIdentity<'a> {
    /// Actual unchanged native document UUID.
    uuid: &'a str,
    /// Actual raw input digest; historical previous-before values are identity-only.
    raw_sha256: &'a str,
}
/// Exact newly encoded native identity; artifact is the explicit portable output filename.
#[derive(Serialize)]
struct NativeIdentity<'a> {
    /// Explicit portable filename for this complete native output.
    artifact: &'a str,
    /// Unchanged actual native document UUID.
    uuid: &'a str,
    /// Explicit next document version copied into one permitted metadata leaf.
    document_version: &'a str,
    /// Exact unchanged native OSCAL version.
    oscal_version: &'a str,
    /// Computed full encoded native output digest, including LF.
    raw_sha256: &'a str,
}
/// Exact one-row companion identity, without paths, content or source prose.
#[derive(Serialize)]
struct ContextRow<'a> {
    /// Exact one of the four maintained companion kinds.
    kind: &'static str,
    /// Actual original companion raw digest from the consumed source declaration.
    sha256: &'a str,
    /// Actual companion native root UUID.
    root_uuid: &'a str,
    /// Actual companion declared document version.
    document_version: &'a str,
    /// Actual companion supported OSCAL version.
    oscal_version: &'a str,
}
/// Complete minimized epoch fact row, including explicit preserved/appended classification.
#[derive(Serialize)]
struct EpochRow<'a> {
    /// Complete zero-based result position, preserving source order.
    position: usize,
    /// Exact result stable key from the retained native property.
    key: &'a str,
    /// Actual native result UUID, or a count-only surrogate.
    uuid: &'a str,
    /// Complete canonical result digest, separately named from its raw file hash.
    canonical_sha256: Cow<'a, str>,
    /// Exact original sealed result start spelling.
    start: &'a str,
    /// Exact nonnull sealed result end spelling.
    end: &'a str,
    /// Complete observation count for this result.
    observations: usize,
    /// Complete finding count for this result.
    findings: usize,
    /// Complete risk count for this result.
    risks: usize,
    /// Preserved for every prior row and appended for exactly the new row.
    classification: &'static str,
}
/// Complete minimized conclusion row; computed hashes and source-declared hashes remain separate.
#[derive(Serialize)]
struct ObjectRow<'a> {
    /// Actual containing result stable key.
    result_key: &'a str,
    /// Actual containing result native UUID.
    result_uuid: &'a str,
    /// Exact observation, finding or risk domain.
    kind: &'static str,
    /// Exact kind-scoped native stable key.
    key: &'a str,
    /// Actual object native UUID, or a count-only surrogate.
    uuid: &'a str,
    /// Computed canonical digest of the complete object Value.
    computed_sha256: Cow<'a, str>,
    /// Original producer-declared content digest, distinct from computed identity.
    declared_content_sha256: &'a str,
    /// Original producer-declared rationale digest, without rationale prose.
    declared_rationale_sha256: &'a str,
    /// Original native finding/risk status; observations explicitly remain null.
    status: Option<&'a str>,
    /// Preserved historical object or added explicit next object.
    classification: &'static str,
}
/// Complete family endpoints and actual append recurrence marker, without inferred closure.
#[derive(Serialize)]
struct FamilyRow<'a> {
    /// Exact caller-supplied nonempty UTF8 family domain, without trimming.
    family_key: &'a str,
    /// Complete first risk tuple of the explicitly supplied family.
    first: ReferenceRow<'a>,
    /// Complete latest retained or newly continued risk tuple.
    current: ReferenceRow<'a>,
    /// True only for an explicit recurrence in this append, never a new family.
    continued_in_append: bool,
}
/// Complete explicit recurrence edge, with preserved minimized caller provenance.
#[derive(Serialize)]
struct EdgeRow<'a> {
    /// Exact family whose latest member is explicitly continued.
    family_key: &'a str,
    /// Complete actual latest historical predecessor risk tuple.
    predecessor: ReferenceRow<'a>,
    /// Complete actual maintained next successor risk tuple.
    successor: ReferenceRow<'a>,
    /// Minimized caller assertion fitting the new sealed window.
    provenance: ProvenanceRow<'a>,
    /// Explicitly true caller assertion, without authentication or disposition authority.
    caller_asserted_continuity: bool,
    /// Computed framed continuity digest of complete identity and provenance facts.
    edge_sha256: String,
}
/// One member of the exact paired predecessor/successor reciprocal row set.
#[derive(Serialize)]
struct ReciprocalRow<'a> {
    /// Exact same family as the paired continuity edge.
    family_key: &'a str,
    /// Exact same computed framed edge digest in both directions.
    edge_sha256: String,
    /// Fixed predecessor or successor trace direction.
    direction: &'static str,
    /// Complete native source endpoint for this direction.
    from: ReferenceRow<'a>,
    /// Complete native destination endpoint for this direction.
    to: ReferenceRow<'a>,
}
/// Complete object locator, not a reduced kind/key or newest-result identifier.
#[derive(Serialize)]
struct ObjectLocator<'a> {
    /// Exact native containing result stable key.
    result_key: &'a str,
    /// Exact native containing result UUID.
    result_uuid: &'a str,
    /// Exact observation, finding or risk domain.
    kind: &'static str,
    /// Exact kind-scoped native object stable key.
    key: &'a str,
    /// Exact native object UUID.
    uuid: &'a str,
}
/// One descriptive action; nulls are required and no caller prose is projected.
#[derive(Serialize)]
struct ChangeRow<'a> {
    /// Fixed descriptive code; never a deletion, closure or approval action.
    code: &'static str,
    /// Explicit nullable full object locator for an object-scoped addition.
    locator: Option<ObjectLocator<'a>>,
    /// Explicit nullable exact family key for a continuity-scoped change.
    family_key: Option<&'a str>,
    /// Explicit nullable prior identity/status/declared hash, without prose.
    old: Option<Cow<'a, str>>,
    /// Explicit nullable next identity/status/declared hash, without prose.
    new: Option<Cow<'a, str>>,
}

/// One temporary borrowed row variant, streamed directly to a serializer/counter.
#[derive(Serialize)]
#[serde(untagged)]
enum Row<'a> {
    /// One minimized actual companion identity row.
    Context(
        /// Complete identity projection of one actually captured companion.
        ContextRow<'a>,
    ),
    /// One complete retained or appended epoch fact row.
    Epoch(
        /// Complete historical or next epoch facts for the actual or prospective stream.
        EpochRow<'a>,
    ),
    /// One complete retained or added conclusion tuple.
    Object(
        /// Complete conclusion tuple with both declared and computed hash fields.
        ObjectRow<'a>,
    ),
    /// One complete caller-supplied family endpoint row.
    Family(
        /// Complete explicit family identity and its exact first and current endpoints.
        FamilyRow<'a>,
    ),
    /// One explicit caller-asserted recurrence edge.
    Edge(
        /// Complete declared recurrence endpoints, provenance projection and framed digest.
        EdgeRow<'a>,
    ),
    /// One of exactly two reciprocal trace rows for an edge.
    Reciprocal(
        /// One complete directional row paired with its reverse in the same edge stream.
        ReciprocalRow<'a>,
    ),
    /// One minimized descriptive review action.
    Change(
        /// Complete descriptive action fields, without implying assessment or continuity authority.
        ChangeRow<'a>,
    ),
    /// One complete previously admitted edge/reciprocal row, preserving its Value.
    Preserved(
        /// Previously admitted complete ledger row retained in its original prefix order.
        &'a Value,
    ),
}

/// Finite complete row streams, no lazy omission of an admitted row category.
#[derive(Clone, Copy)]
enum RowKind {
    /// Complete four-companion identity stream.
    Context,
    /// Complete original-order epoch stream plus one append.
    Epochs,
    /// Complete all-epoch observation, finding and risk stream.
    Objects,
    /// Complete sorted family registry stream.
    Families,
    /// Complete old edge prefix followed by sorted new recurrence edges.
    Edges,
    /// Complete old reciprocal prefix followed by both new edge directions.
    Reciprocal,
    /// Complete descriptive action roster for this append.
    Changes,
}
/// Borrowed row source with no retaining vector or detached proof constructor.
struct Rows<'a> {
    /// Borrowed complete report source whose lifetime retains actual captured facts.
    mirror: &'a ReportMirror<'a>,
    /// Exactly one finite row category to visit completely.
    kind: RowKind,
}

impl Serialize for Rows<'_> {
    /// Emit every complete row in deterministic order, converting errors to fixed serializer faults.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(None)?;
        self.visit(|row| {
            sequence.serialize_element(&row).map_err(|_| error("epoch row serialization failed"))
        })
        .map_err(serde::ser::Error::custom)?;
        sequence.end()
    }
}

impl Rows<'_> {
    /// Visit borrowed facts with one shared increment and cap check before every emission.
    fn visit(
        &self,
        mut emit: impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<usize, ForgeError> {
        let mut count = 0;
        let mut push = |row: Row<'_>| {
            count = add(count, 1)?;
            if count > MAX_RELATIONSHIPS {
                return Err(error("epoch complete report row count exceeded"));
            }
            emit(row)
        };
        match self.kind {
            RowKind::Context => self.visit_context(&mut push)?,
            RowKind::Epochs => self.visit_epochs(&mut push)?,
            RowKind::Objects => self.visit_objects(&mut push)?,
            RowKind::Families => self.visit_families(&mut push)?,
            RowKind::Edges => self.visit_edges(&mut push)?,
            RowKind::Reciprocal => self.visit_reciprocal(&mut push)?,
            RowKind::Changes => self.visit_changes(&mut push)?,
        }
        Ok(count)
    }

    /// Emit the four companion identities in their existing order without detached copies.
    fn visit_context(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        for (kind, a) in [
            ("assessment-plan", &m.source.context.assessment_plan),
            ("system-security-plan", &m.source.context.ssp),
            ("profile", &m.source.context.profile),
            ("catalog", &m.source.context.catalog),
        ] {
            push(Row::Context(ContextRow {
                kind,
                sha256: &a.expected_sha256,
                root_uuid: &a.root_uuid,
                document_version: &a.document_version,
                oscal_version: &a.oscal_version,
            }))?;
        }
        Ok(())
    }

    /// Emit all historical epochs in source order, then the one actual or count-only append.
    fn visit_epochs(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        let old = array(&m.old["results"], MAX_EPOCHS - 1)?;
        for (position, result) in old.iter().enumerate() {
            push(Row::Epoch(epoch_row(position, result, false)?))?;
        }
        push(Row::Epoch(match m.next {
            Next::Native(result) => epoch_row(old.len(), result, true)?,
            Next::Authored(result) => EpochRow {
                position: old.len(),
                key: text(&result["key"])?,
                uuid: UUID,
                canonical_sha256: Cow::Borrowed(HASH),
                start: text(&result["start"])?,
                end: text(&result["end"])?,
                observations: m.next_counts.observations,
                findings: m.next_counts.findings,
                risks: m.next_counts.risks,
                classification: "appended",
            },
        }))?;
        Ok(())
    }

    /// Emit complete historical and next conclusion tuples in existing epoch/kind/object order.
    fn visit_objects(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        for result in array(&m.old["results"], MAX_EPOCHS - 1)? {
            for (field, kind) in
                [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
            {
                for object in optional_array(result.get(field), MAX_OBJECTS)? {
                    push(Row::Object(object_row(result, object, kind, false)?))?;
                }
            }
        }
        match m.next {
            Next::Native(result) => {
                for (field, kind) in
                    [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
                {
                    for object in optional_array(result.get(field), MAX_OBJECTS)? {
                        push(Row::Object(object_row(result, object, kind, true)?))?;
                    }
                }
            }
            Next::Authored(result) => {
                for (field, kind) in
                    [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
                {
                    for object in array(&result[field], MAX_OBJECTS)? {
                        push(Row::Object(ObjectRow {
                            result_key: text(&result["key"])?,
                            result_uuid: UUID,
                            kind,
                            key: text(&object["key"])?,
                            uuid: UUID,
                            computed_sha256: Cow::Borrowed(HASH),
                            declared_content_sha256: HASH,
                            declared_rationale_sha256: HASH,
                            status: if kind == "finding" {
                                object["target"]["state"].as_str()
                            } else if kind == "risk" {
                                object["status"].as_str()
                            } else {
                                None
                            },
                            classification: "added",
                        }))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Emit the complete sorted family registry using the original borrowed endpoint references.
    fn visit_families(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        for family in m.plan.families.values() {
            push(Row::Family(FamilyRow {
                family_key: family.key,
                first: reference(family.first, m.next.actual())?,
                current: reference(family.current, m.next.actual())?,
                continued_in_append: family.continued,
            }))?;
        }
        Ok(())
    }

    /// Emit the unchanged prior edge prefix, then each sorted new edge after its original hash work.
    fn visit_edges(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        for row in m.plan.previous_edges()? {
            push(Row::Preserved(row))?;
        }
        for edge in m.plan.edges.values() {
            let predecessor = reference(edge.predecessor, m.next.actual())?;
            let successor = reference(edge.successor, m.next.actual())?;
            let provenance = provenance_row(edge.provenance, true)?;
            let hash = continuity_hash(edge.family, &predecessor, &successor, &provenance)?;
            push(Row::Edge(EdgeRow {
                family_key: edge.family,
                predecessor,
                successor,
                provenance,
                caller_asserted_continuity: true,
                edge_sha256: hash,
            }))?;
        }
        Ok(())
    }

    /// Emit the unchanged prior reciprocal prefix and both original directions for every new edge.
    fn visit_reciprocal(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        if let Some(previous) = m.plan.previous {
            for row in array(&previous["reciprocal_rows"], MAX_OBJECTS * 2)? {
                push(Row::Preserved(row))?;
            }
        }
        for edge in m.plan.edges.values() {
            for direction in ["predecessor", "successor"] {
                let a = reference(edge.predecessor, m.next.actual())?;
                let b = reference(edge.successor, m.next.actual())?;
                let hash =
                    continuity_hash(edge.family, &a, &b, &provenance_row(edge.provenance, true)?)?;
                let (from, to) = if direction == "predecessor" { (a, b) } else { (b, a) };
                push(Row::Reciprocal(ReciprocalRow {
                    family_key: edge.family,
                    edge_sha256: hash,
                    direction,
                    from,
                    to,
                }))?;
            }
        }
        Ok(())
    }

    /// Emit append/object/metadata/family/recurrence actions in the unchanged descriptive roster order.
    fn visit_changes(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        push(Row::Change(ChangeRow {
            code: "epoch-added",
            locator: None,
            family_key: None,
            old: None,
            new: Some(Cow::Borrowed(m.next.key()?)),
        }))?;
        self.visit_object_changes(push)?;
        for (code, old, new) in [
            ("document-version-changed", text(&m.old["metadata"]["version"])?, m.next_version),
            (
                "document-modified-time-changed",
                text(&m.old["metadata"]["last-modified"])?,
                text(m.plan_next_modified())?,
            ),
        ] {
            if old != new {
                push(Row::Change(ChangeRow {
                    code,
                    locator: None,
                    family_key: None,
                    old: Some(Cow::Borrowed(old)),
                    new: Some(Cow::Borrowed(new)),
                }))?;
            }
        }
        for family in m.plan.families.values() {
            if matches!(family.first, Endpoint::Authored { .. }) {
                push(Row::Change(ChangeRow {
                    code: "risk-family-created",
                    locator: None,
                    family_key: Some(family.key),
                    old: None,
                    new: None,
                }))?;
            }
        }
        self.visit_recurrence_changes(push)?;
        Ok(())
    }

    /// Emit complete next object additions without allocating or retaining a second action roster.
    fn visit_object_changes(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        match m.next {
            Next::Native(result) => {
                for (field, kind) in
                    [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
                {
                    for object in optional_array(result.get(field), MAX_OBJECTS)? {
                        push(Row::Change(ChangeRow {
                            code: "object-added",
                            locator: Some(locator(result, object, kind)?),
                            family_key: None,
                            old: None,
                            new: None,
                        }))?;
                    }
                }
            }
            Next::Authored(result) => {
                for (field, kind) in
                    [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
                {
                    for object in array(&result[field], MAX_OBJECTS)? {
                        push(Row::Change(ChangeRow {
                            code: "object-added",
                            locator: Some(ObjectLocator {
                                result_key: text(&result["key"])?,
                                result_uuid: UUID,
                                kind,
                                key: text(&object["key"])?,
                                uuid: UUID,
                            }),
                            family_key: None,
                            old: None,
                            new: None,
                        }))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Emit each continuity row before its original content, rationale and status change checks.
    fn visit_recurrence_changes(
        &self,
        push: &mut impl FnMut(Row<'_>) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let m = self.mirror;
        for edge in m.plan.edges.values() {
            let before = reference(edge.predecessor, None)?;
            let after = reference(edge.successor, m.next.actual())?;
            push(Row::Change(ChangeRow {
                code: "risk-continuity-recorded",
                locator: None,
                family_key: Some(edge.family),
                old: Some(before.uuid),
                new: Some(after.uuid),
            }))?;
            let Endpoint::Native { object: previous, .. } = edge.predecessor else {
                return Err(error("epoch recurrence predecessor is not actual native"));
            };
            for (code, prop) in [
                ("recurrence-content-changed", "content-sha256"),
                ("recurrence-rationale-changed", "rationale-sha256"),
            ] {
                let old = property(previous, prop)?;
                let new = if let Some(actual) = m.next.actual() {
                    property(find_native_risk(actual, edge.successor.key()?)?, prop)?
                } else {
                    HASH
                };
                if m.next.actual().is_none() || old != new {
                    push(Row::Change(ChangeRow {
                        code,
                        locator: None,
                        family_key: Some(edge.family),
                        old: Some(Cow::Borrowed(old)),
                        new: Some(Cow::Borrowed(new)),
                    }))?;
                }
            }
            let old = text(&previous["status"])?;
            let new = match edge.successor {
                Endpoint::Authored { risk, .. } => text(&risk["status"])?,
                Endpoint::Native { .. } => {
                    return Err(error("epoch recurrence successor is not explicit authoring"));
                }
            };
            if m.next.actual().is_none() || old != new {
                push(Row::Change(ChangeRow {
                    code: "recurrence-status-changed",
                    locator: None,
                    family_key: Some(edge.family),
                    old: Some(Cow::Borrowed(old)),
                    new: Some(Cow::Borrowed(new)),
                }))?;
            }
        }
        Ok(())
    }
}

impl ReportMirror<'_> {
    /// Borrow explicitly supplied next modified time for both prospective and actual views.
    fn plan_next_modified(&self) -> &Value {
        // The typed next result has no document metadata, so continuity stores
        // the original complete request document in its own borrowed field.
        &self.plan.next_document["last_modified"]
    }
}

/// Construct a real native epoch row without reconstructing any old Value.
fn epoch_row(position: usize, result: &Value, appended: bool) -> Result<EpochRow<'_>, ForgeError> {
    Ok(EpochRow {
        position,
        key: property(result, "stable-key")?,
        uuid: text(&result["uuid"])?,
        canonical_sha256: Cow::Owned(source::canonical_epoch_object_sha256(result)?),
        start: text(&result["start"])?,
        end: text(&result["end"])?,
        observations: optional_array(result.get("observations"), MAX_OBJECTS)?.len(),
        findings: optional_array(result.get("findings"), MAX_OBJECTS)?.len(),
        risks: optional_array(result.get("risks"), MAX_OBJECTS)?.len(),
        classification: if appended { "appended" } else { "preserved" },
    })
}

/// Construct one complete native object tuple, with rationale/content declarations named separately.
fn object_row<'a>(
    result: &'a Value,
    object: &'a Value,
    kind: &'static str,
    added: bool,
) -> Result<ObjectRow<'a>, ForgeError> {
    Ok(ObjectRow {
        result_key: property(result, "stable-key")?,
        result_uuid: text(&result["uuid"])?,
        kind,
        key: property(object, "stable-key")?,
        uuid: text(&object["uuid"])?,
        computed_sha256: Cow::Owned(source::canonical_epoch_object_sha256(object)?),
        declared_content_sha256: property(object, "content-sha256")?,
        declared_rationale_sha256: property(object, "rationale-sha256")?,
        status: object_status(object, kind),
        classification: if added { "added" } else { "preserved" },
    })
}

/// Borrow every field of the exact object locator used by descriptive change rows.
fn locator<'a>(
    result: &'a Value,
    object: &'a Value,
    kind: &'static str,
) -> Result<ObjectLocator<'a>, ForgeError> {
    Ok(ObjectLocator {
        result_key: property(result, "stable-key")?,
        result_uuid: text(&result["uuid"])?,
        kind,
        key: property(object, "stable-key")?,
        uuid: text(&object["uuid"])?,
    })
}

/// Stream all23 complete denominator fields, using conservative widths only before actual construction.
struct CountRows<'a> {
    /// Borrowed complete report mirror supplying exact actual or count-only denominators.
    mirror: &'a ReportMirror<'a>,
}
impl Serialize for CountRows<'_> {
    /// Exact actual counts remain visible; count-only surrogates create no report authority.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let m = self.mirror;
        let prospective = m.next.actual().is_none();
        let epochs =
            array(&m.old["results"], MAX_EPOCHS - 1).map_err(serde::ser::Error::custom)?.len();
        let edges = add(
            m.plan.previous_edges().map_err(serde::ser::Error::custom)?.len(),
            m.plan.edges.len(),
        )
        .map_err(serde::ser::Error::custom)?;
        let actions = Rows { mirror: m, kind: RowKind::Changes }
            .visit(|_| Ok(()))
            .map_err(serde::ser::Error::custom)?;
        let uncontinued =
            m.plan.families.values().filter(|family| family.current.position() < epochs).count();
        let new_occurrences = if let Some(result) = m.next.actual() {
            occurrences(result).map_err(serde::ser::Error::custom)?
        } else {
            0
        };
        let before = m.old_counts;
        let after = m.next_counts;
        let mut map = serializer.serialize_map(Some(23))?;
        let sums = |left, right| add(left, right).map_err(serde::ser::Error::custom);
        let values = [
            ("before_epochs", epochs),
            ("after_epochs", add(epochs, 1).map_err(serde::ser::Error::custom)?),
            ("preserved_epochs", epochs),
            ("appended_epochs", 1),
            ("before_observations", before.observations),
            ("after_observations", sums(before.observations, after.observations)?),
            ("before_findings", before.findings),
            ("after_findings", sums(before.findings, after.findings)?),
            ("before_risks", before.risks),
            ("after_risks", sums(before.risks, after.risks)?),
            ("before_objects", before.objects),
            ("after_objects", sums(before.objects, after.objects)?),
            ("preserved_objects", before.objects),
            ("added_objects", after.objects),
            ("native_graph_occurrences_before", before.occurrences),
            (
                "native_graph_occurrences_after",
                sums(
                    before.occurrences,
                    add(new_occurrences, 1).map_err(serde::ser::Error::custom)?,
                )?,
            ),
            ("families", m.plan.families.len()),
            ("continuity_edges", edges),
            ("reciprocal_rows", mul(2, edges).map_err(serde::ser::Error::custom)?),
            ("classified_next_risks", m.plan.classified),
            ("uncontinued_families", uncontinued),
            ("captured_original_generations", m.generations),
            ("review_actions", actions),
        ];
        for (key, n) in values {
            if n > MAX_RELATIONSHIPS {
                return Err(serde::ser::Error::custom(
                    "epoch complete report count bound exceeded",
                ));
            }
            map.serialize_entry(key, &if prospective { MAX_RELATIONSHIPS } else { n })?;
        }
        map.end()
    }
}

/// Proposed genuine native-fixture and bounded predicate regressions; not executed by the author.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::oscal::catalog::OscalMetadata;
    use crate::oscal::parts::OscalPartName;
    use crate::oscal::{
        CatalogEnvelope, OscalCatalog, OscalControl, OscalPart, ProfileRoot, SelectionMode,
        SspComponentInput, build_assessment_plan, build_profile, build_ssp_skeleton,
    };
    use chrono::{TimeZone as _, Utc};
    use serde_json::json;
    use tempfile::TempDir;

    /// Owned actual four-companion/AR originals and the complete caller append request.
    struct Fixture {
        /// Keep the genuine private owned directory alive through preparation and rechecks.
        _directory: TempDir,
        /// Qualified canonical root used by actual confined capture, never a report field.
        root: PathBuf,
        /// Full explicit closed request whose native pin binds persisted original bytes.
        request: Value,
    }
    /// Persist deterministic fixture JSON and return the original bytes used by native pin declarations.
    fn write_json(path: &Path, value: &Value) -> Vec<u8> {
        let mut bytes = serde_json::to_vec_pretty(value).unwrap();
        bytes.push(b'\n');
        std::fs::write(path, &bytes).unwrap();
        bytes
    }

    /// Pin persisted native bytes and actual generated root metadata, never detached inventory labels.
    fn pin(name: &str, bytes: &[u8], value: &Value, root: &str) -> Value {
        json!({"artifact":name,"href":name,"expected_sha256":sha256_hex(bytes),
            "root_uuid":value[root]["uuid"],"document_version":value[root]["metadata"]["version"],
            "oscal_version":value[root]["metadata"]["oscal-version"]})
    }

    /// Build the existing supported Catalog shape with deliberately sensitive unselected source prose.
    fn native_catalog() -> OscalCatalog {
        OscalCatalog {
            uuid: "11111111-1111-4111-8111-111111111111".to_string(),
            metadata: OscalMetadata {
                title: "Synthetic catalog".to_string(),
                last_modified: "2026-01-01T00:00:00Z".to_string(),
                version: "1.0.0".to_string(),
                oscal_version: "1.2.3".to_string(),
            },
            controls: vec![OscalControl {
                id: "AC-1".to_string(),
                uuid: String::new(),
                title: "Synthetic access".to_string(),
                links: Vec::new(),
                params: Vec::new(),
                props: Vec::new(),
                parts: vec![
                    OscalPart {
                        id: "AC-1_smt".to_string(),
                        name: OscalPartName::Statement,
                        prose: "SENSITIVE SOURCE STATEMENT".to_string(),
                        parts: Vec::new(),
                        props: Vec::new(),
                    },
                    OscalPart {
                        id: "AC-1_obj".to_string(),
                        name: OscalPartName::Objective,
                        prose: "SENSITIVE SOURCE OBJECTIVE".to_string(),
                        parts: Vec::new(),
                        props: Vec::new(),
                    },
                ],
            }],
            groups: Vec::new(),
            back_matter: None,
        }
    }

    /// Generate four genuine companion models, bind the AP subject to the actual SSP component and pin bytes.
    fn native_companions(root: &Path) -> (Value, Value) {
        let catalog = native_catalog();
        let catalog_value =
            serde_json::to_value(CatalogEnvelope { catalog: catalog.clone() }).unwrap();
        let catalog_bytes = write_json(&root.join("catalog.json"), &catalog_value);
        let profile = build_profile(
            "catalog.json",
            vec!["AC-1".to_string()],
            SelectionMode::Include,
            &[],
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
        )
        .unwrap();
        let profile_value = serde_json::to_value(ProfileRoot { profile }).unwrap();
        let profile_bytes = write_json(&root.join("profile.json"), &profile_value);
        let ssp = build_ssp_skeleton(
            "Synthetic system",
            "1.0.0",
            &catalog,
            &[SspComponentInput {
                title: "Synthetic component".to_string(),
                description: "Synthetic boundary".to_string(),
                component_type: crate::oscal::ssp::ComponentType::Software,
            }],
            "profile.json",
        )
        .unwrap();
        let ssp_value = serde_json::to_value(ssp).unwrap();
        let ssp_bytes = write_json(&root.join("ssp.json"), &ssp_value);
        let subject =
            ssp_value["system-security-plan"]["system-implementation"]["components"][0]["uuid"]
                .clone();
        let mut ap_value = serde_json::to_value(
            build_assessment_plan(&["AC-1".to_string()], "ssp.json", "Synthetic assessment")
                .unwrap(),
        )
        .unwrap();
        ap_value["assessment-plan"]["reviewed-controls"]["control-objective-selections"] =
            json!([{"include-objectives":[{"objective-id":"AC-1_obj"}]}]);
        ap_value["assessment-plan"]["assessment-subjects"] = json!([{
            "type":"component","include-subjects":[{"subject-uuid":subject,"type":"component"}]}]);
        let ap_bytes = write_json(&root.join("assessment-plan.json"), &ap_value);
        (
            json!({
                "assessment_plan":pin("assessment-plan.json",&ap_bytes,&ap_value,"assessment-plan"),
                "ssp":pin("ssp.json",&ssp_bytes,&ssp_value,"system-security-plan"),
                "profile":pin("profile.json",&profile_bytes,&profile_value,"profile"),
                "catalog":pin("catalog.json",&catalog_bytes,&catalog_value,"catalog"),
            }),
            subject,
        )
    }

    /// Declare every optional authoring field explicitly; actual maintained model receives these bytes.
    fn declaration(
        context: &Value,
        subject: &Value,
        suffix: &str,
        start: &str,
        end: &str,
        version: &str,
        confidence: f64,
    ) -> Value {
        let provenance = json!({"assessor_key":"caller-assessor","role_id":"assessor","start":start,"end":null,"method":"EXAMINE","rationale":"Explicit caller assessment rationale"});
        json!({"schema_version":"forge.assessment-results/1",
            "document":{"key":"caller-assessment","title":"Caller assessment","version":version,"last_modified":end},
            "context":context,"roles":[{"id":"assessor","title":"Caller assessor role"}],
            "parties":[{"key":"caller-assessor","type":"person","name":"PRIVATE ACTOR NAME"}],
            "result":{"key":format!("epoch-{suffix}"),"title":"Caller sealed epoch","description":"PRIVATE EPOCH DESCRIPTION",
                "start":start,"end":end,"control_ids":["AC-1"],"objective_ids":["AC-1_obj"],
                "observations":[{"key":format!("observation-{suffix}"),"title":null,"description":"PRIVATE OBSERVATION",
                    "provenance":provenance,"subjects":[{"type":"component","uuid":subject}],"task_uuids":[],"evidence_keys":[]}],
                "findings":[{"key":format!("finding-{suffix}"),"title":"PRIVATE FINDING TITLE","description":"PRIVATE FINDING DESCRIPTION",
                    "provenance":provenance,"target":{"type":"objective-id","id":"AC-1_obj","state":"not-satisfied","reason":"fail"},"implementation_statement_uuid":null}],
                "risks":[{"key":format!("risk-{suffix}"),"title":"PRIVATE RISK TITLE","description":"PRIVATE RISK DESCRIPTION","statement":"PRIVATE RISK STATEMENT",
                    "status":"open","severity":null,"confidence":confidence,"provenance":provenance}],
                "relationships":[
                    {"from":{"type":"observation","key":format!("observation-{suffix}")},"to":{"type":"finding","key":format!("finding-{suffix}")}},
                    {"from":{"type":"finding","key":format!("finding-{suffix}")},"to":{"type":"risk","key":format!("risk-{suffix}")}}]}})
    }

    /// Build the genuine original through public manifest/context/model APIs, never a detached holder.
    fn fixture(confidence: f64) -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let (mut context, subject) = native_companions(&root);
        context["evidence_index"] = Value::Null;
        let original = declaration(
            &context,
            &subject,
            "one",
            "2026-01-01T00:00:00Z",
            "2026-01-02T00:00:00Z",
            "1.0.0",
            0.5,
        );
        let old_manifest_path = root.join("old-authoring.json");
        let bytes = write_json(&old_manifest_path, &original);
        let typed = manifest::parse(&bytes).unwrap();
        let loaded = assessment_results::context::load(&old_manifest_path, &typed.context).unwrap();
        let built = model::build(&typed, &loaded).unwrap();
        let native = serde_json::to_value(&built.artifact).unwrap();
        assessment_results::validate_completed_json(&native).unwrap();
        let native_bytes = write_json(&root.join("prior.json"), &native);
        let old = &native["assessment-results"]["results"][0];
        let old_risk = &old["risks"][0];
        let risk = json!({"kind":"risk","key":"risk-one","uuid":old_risk["uuid"],"result_uuid":old["uuid"],"expected_sha256":source::canonical_epoch_object_sha256(old_risk).unwrap()});
        let next = declaration(
            &context,
            &subject,
            "two",
            "2026-01-03T00:00:00Z",
            "2026-01-04T00:00:00Z",
            "1.1.0",
            confidence,
        );
        let request = json!({"schema_version":"forge.assessment-epoch-append/1",
            "prior_source":{"assessment_results":pin("prior.json",&native_bytes,&native,"assessment-results"),"result":{"key":"epoch-one","uuid":old["uuid"]},"context":context},
            "next_epoch":next,"prior_report":null,
            "seed_families":[{"family_key":"family-one","risk":risk,"caller_asserted_continuity":true,"provenance":original["result"]["risks"][0]["provenance"]}],
            "new_risks":[{"next_key":"risk-two","family_key":"family-two","prior":null,"caller_asserted_continuity":true,"provenance":next["result"]["risks"][0]["provenance"]}]});
        Fixture { _directory: directory, root, request }
    }

    /// Persist a complete authored request immediately before the actual one-session producer call.
    fn prepare_fixture(
        fixture: &Fixture,
        view: Option<epoch_report::EpochViewFormat>,
    ) -> Result<PreparedEpochAppend, ForgeError> {
        write_json(&fixture.root.join("request.json"), &fixture.request);
        prepare_with_view(
            &fixture.root.join("request.json"),
            Path::new("after.json"),
            Path::new("report.json"),
            view,
        )
    }

    /// Decode actual complete prepared bytes only in test assertions; no capture authority is fabricated.
    fn prepared_json(prepared: &PreparedEpochAppend) -> Value {
        serde_json::from_slice(prepared.report_json()).unwrap()
    }

    /// Persist the complete first pair, then author one explicit latest-family recurrence for a real third epoch.
    fn subsequent_fixture() -> Fixture {
        let mut fixture = fixture(0.5);
        let first = prepare_fixture(&fixture, None).unwrap();
        let native: Value = serde_json::from_slice(first.native_bytes()).unwrap();
        std::fs::write(fixture.root.join("after.json"), first.native_bytes()).unwrap();
        std::fs::write(fixture.root.join("report.json"), first.report_json()).unwrap();
        let report_hash = sha256_hex(first.report_json());
        let native_hash = sha256_hex(first.native_bytes());
        drop(first);
        let second = &native["assessment-results"]["results"][1];
        let risk = &second["risks"][0];
        let reference = json!({"kind":"risk","key":"risk-two","uuid":risk["uuid"],"result_uuid":second["uuid"],"expected_sha256":source::canonical_epoch_object_sha256(risk).unwrap()});
        fixture.request["prior_source"]["assessment_results"] = json!({"artifact":"after.json","href":"after.json","expected_sha256":native_hash,"root_uuid":native["assessment-results"]["uuid"],"document_version":"1.1.0","oscal_version":"1.2.3"});
        fixture.request["prior_source"]["result"] =
            json!({"key":"epoch-two","uuid":second["uuid"]});
        let subject =
            fixture.request["next_epoch"]["result"]["observations"][0]["subjects"][0]["uuid"]
                .clone();
        let next = declaration(
            &fixture.request["prior_source"]["context"],
            &subject,
            "three",
            "2026-01-05T00:00:00Z",
            "2026-01-06T00:00:00Z",
            "1.2.0",
            0.25,
        );
        fixture.request["next_epoch"] = next;
        fixture.request["prior_report"] =
            json!({"artifact":"report.json","expected_sha256":report_hash});
        fixture.request["seed_families"] = json!([]);
        fixture.request["new_risks"] = json!([{"next_key":"risk-three","family_key":"family-two","prior":reference,"caller_asserted_continuity":true,"provenance":fixture.request["next_epoch"]["result"]["risks"][0]["provenance"]}]);
        fixture
    }

    /// Prepare a later append from actual persisted pair and unchanged companions, preserving seven originals.
    fn prepare_subsequent(fixture: &Fixture) -> Result<PreparedEpochAppend, ForgeError> {
        write_json(&fixture.root.join("request-three.json"), &fixture.request);
        prepare(
            &fixture.root.join("request-three.json"),
            Path::new("after-three.json"),
            Path::new("report-three.json"),
        )
    }

    /// A real first append preserves every original Value, complete counts and the zero-edge family denominator.
    #[test]
    fn actual_first_append_is_complete_and_source_bound() {
        let fixture = fixture(0.5);
        let prepared = prepare_fixture(&fixture, None).unwrap();
        let old: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("prior.json")).unwrap())
                .unwrap();
        let native: Value = serde_json::from_slice(prepared.native_bytes()).unwrap();
        let report = prepared_json(&prepared);
        assert_eq!(native["assessment-results"]["results"].as_array().unwrap().len(), 2);
        assert_eq!(
            native["assessment-results"]["results"][0],
            old["assessment-results"]["results"][0]
        );
        assert_eq!(report["counts"]["before_objects"], 3);
        assert_eq!(report["counts"]["after_objects"], 6);
        assert_eq!(report["counts"]["families"], 2);
        assert_eq!(report["counts"]["continuity_edges"], 0);
        assert_eq!(report["counts"]["reciprocal_rows"], 0);
        assert_eq!(report["counts"]["uncontinued_families"], 1);
        assert_eq!(report["counts"]["captured_original_generations"], 6);
        assert_eq!(
            report["counts"]["native_graph_occurrences_before"],
            occurrences(&old["assessment-results"]["results"]).unwrap()
        );
        assert_eq!(
            report["counts"]["native_graph_occurrences_after"],
            occurrences(&native["assessment-results"]["results"]).unwrap()
        );
        assert_eq!(report["counts"]["review_actions"], report["changes"].as_array().unwrap().len());
        assert_eq!(
            report["changes"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["code"] == "risk-family-created")
                .count(),
            1
        );
        assert!(prepared.view_bytes().is_none());
        assert!(prepared.review_required());
        assert_eq!(prepared.input_paths().count(), 6);
        prepared.verify_inputs().unwrap();
        for private in [
            "PRIVATE ACTOR",
            "PRIVATE RISK",
            "PRIVATE FINDING",
            "PRIVATE OBSERVATION",
            "SENSITIVE SOURCE",
        ] {
            assert!(!String::from_utf8_lossy(prepared.report_json()).contains(private));
        }
    }

    /// Every selected stdout view uses the same complete actual DTO; JSON is byte-exact required JSON+LF.
    #[test]
    fn actual_json_text_html_views_are_complete_and_inert() {
        for view in [
            epoch_report::EpochViewFormat::Json,
            epoch_report::EpochViewFormat::Text,
            epoch_report::EpochViewFormat::Html,
        ] {
            let mut fixture = fixture(0.5);
            fixture.request["new_risks"][0]["family_key"] = json!("<script>\"&\u{0085}\u{202e}é");
            let prepared = prepare_fixture(&fixture, Some(view)).unwrap();
            let stdout = prepared.view_bytes().unwrap();
            assert!(prepared.report_json().ends_with(b"\n"));
            if view == epoch_report::EpochViewFormat::Json {
                assert_eq!(stdout, prepared.report_json());
            } else if view == epoch_report::EpochViewFormat::Html {
                let html = std::str::from_utf8(stdout).unwrap();
                assert!(!html.contains("<script>"));
                assert!(!html.contains("href="));
                assert!(html.contains("&lt;script&gt;"));
            }
            assert!(!std::str::from_utf8(stdout).unwrap().contains('\u{202e}'));
            let value = prepared_json(&prepared);
            assert!(
                value["families"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|row| row["family_key"] == fixture.request["new_risks"][0]["family_key"])
            );
        }
    }

    /// Subsequent preparation consumes the actual raw report and emits paired explicit recurrence edges.
    #[test]
    fn actual_subsequent_append_preserves_prior_ledger_and_reciprocal_counts() {
        let fixture = subsequent_fixture();
        let prepared = prepare_subsequent(&fixture).unwrap();
        let report = prepared_json(&prepared);
        assert_eq!(report["counts"]["before_epochs"], 2);
        assert_eq!(report["counts"]["after_epochs"], 3);
        assert_eq!(report["counts"]["before_objects"], 6);
        assert_eq!(report["counts"]["after_objects"], 9);
        assert_eq!(report["counts"]["continuity_edges"], 1);
        assert_eq!(report["counts"]["reciprocal_rows"], 2);
        assert_eq!(report["counts"]["captured_original_generations"], 7);
        assert_eq!(report["counts"]["classified_next_risks"], 1);
        assert_eq!(report["counts"]["uncontinued_families"], 1);
        assert_eq!(
            report["continuity_edges"][0]["predecessor"],
            fixture.request["new_risks"][0]["prior"]
        );
        assert_eq!(report["reciprocal_rows"][0]["from"], report["reciprocal_rows"][1]["to"]);
        assert_eq!(report["reciprocal_rows"][0]["to"], report["reciprocal_rows"][1]["from"]);
        assert!(
            report["families"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["family_key"] == "family-two" && row["continued_in_append"] == true)
        );
        assert_eq!(prepared.input_paths().count(), 7);
        prepared.verify_inputs().unwrap();
    }

    /// Each of all 23 previous counts must agree with actual native/ledger facts even after re-pinning forged raw report bytes.
    #[test]
    fn actual_previous_report_all_count_fields_reject_semantic_tampering() {
        let mut fixture = subsequent_fixture();
        let original: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("report.json")).unwrap())
                .unwrap();
        for key in original["counts"].as_object().unwrap().keys() {
            let mut forged = original.clone();
            let count = forged["counts"][key].as_u64().unwrap();
            forged["counts"][key] = json!(if key == "appended_epochs" { 0 } else { count + 1 });
            epoch_report::validate_value(&forged).unwrap();
            let raw = write_json(&fixture.root.join("report.json"), &forged);
            fixture.request["prior_report"]["expected_sha256"] = json!(sha256_hex(&raw));
            assert!(prepare_subsequent(&fixture).is_err(), "forged count {key} admitted");
        }
    }

    /// Historical previous-before raw hashes stay identity-only while current native/report raw pins remain actual.
    #[test]
    fn historical_previous_before_digest_is_not_falsely_claimed_fresh() {
        let mut fixture = subsequent_fixture();
        let mut previous: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("report.json")).unwrap())
                .unwrap();
        previous["prior_native"]["raw_sha256"] = json!("1".repeat(64));
        let raw = write_json(&fixture.root.join("report.json"), &previous);
        fixture.request["prior_report"]["expected_sha256"] = json!(sha256_hex(&raw));
        let prepared = prepare_subsequent(&fixture).unwrap();
        assert_eq!(
            prepared_json(&prepared)["prior_native"]["raw_sha256"],
            fixture.request["prior_source"]["assessment_results"]["expected_sha256"]
        );
        prepared.verify_inputs().unwrap();
    }

    /// All six/seven original byte generations are independently rechecked by the returned actual proof.
    #[test]
    fn actual_prepared_proof_refuses_each_original_byte_change() {
        for subsequent in [false, true] {
            let fixture = if subsequent { subsequent_fixture() } else { fixture(0.5) };
            let prepared = if subsequent {
                prepare_subsequent(&fixture)
            } else {
                prepare_fixture(&fixture, None)
            }
            .unwrap();
            let paths: Vec<_> = prepared.input_paths().collect();
            assert_eq!(paths.len(), if subsequent { 7 } else { 6 });
            for path in paths {
                let original = std::fs::read(&path).unwrap();
                let mut changed = original.clone();
                changed.push(b' ');
                std::fs::write(&path, &changed).unwrap();
                assert!(
                    prepared.verify_inputs().is_err(),
                    "one changed held input remained admitted"
                );
                std::fs::write(&path, &original).unwrap();
                prepared.verify_inputs().unwrap();
            }
        }
    }

    /// Same-byte replacement is generation drift under Unix's actual held file semantics.
    #[cfg(unix)]
    #[test]
    fn actual_prepared_proof_refuses_same_bytes_new_identity() {
        let fixture = fixture(0.5);
        let prepared = prepare_fixture(&fixture, None).unwrap();
        let path = fixture.root.join("prior.json");
        let original = std::fs::read(&path).unwrap();
        let replacement = fixture.root.join("replacement.json");
        std::fs::write(&replacement, &original).unwrap();
        std::fs::rename(&replacement, &path).unwrap();
        assert!(prepared.verify_inputs().is_err());
    }

    /// Tiny and boundary confidence values reach the genuine model and completed native schema before append.
    #[test]
    fn genuine_confidence_display_extremes_fit_admitted_native_ceiling() {
        for confidence in [1e-100, f64::from_bits(1), -0.0, 0.0, 1.0] {
            let fixture = fixture(confidence);
            let prepared = prepare_fixture(&fixture, None).unwrap();
            let native: Value = serde_json::from_slice(prepared.native_bytes()).unwrap();
            let risk = &native["assessment-results"]["results"][1]["risks"][0];
            let encoded = property(risk, "reviewer-declared-confidence").unwrap();
            assert_eq!(encoded, confidence.to_string());
            assert!(encoded.len() < 1024);
            assert_eq!(prepared_json(&prepared)["counts"]["after_risks"], 2);
        }
    }

    /// Genuine repeated references, escaping and non-ASCII native metadata consume the same prospective and retained ceilings.
    #[test]
    fn genuine_model_escaping_repeated_references_and_metadata_growth_fit_profile() {
        let mut fixture = fixture(0.25);
        fixture.request["next_epoch"]["result"]["description"] = json!("\"\\\n\t<>&é".repeat(300));
        fixture.request["next_epoch"]["result"]["risks"][0]["description"] =
            json!("Quoted \" path \\ with café metadata and\nnew line".repeat(100));
        fixture.request["next_epoch"]["document"]["version"] = json!("v".repeat(128));
        let base_finding = fixture.request["next_epoch"]["result"]["findings"][0].clone();
        for position in 0..8 {
            let key = format!("finding-two-extra-{position}");
            let mut finding = base_finding.clone();
            finding["key"] = json!(key);
            fixture.request["next_epoch"]["result"]["findings"]
                .as_array_mut()
                .unwrap()
                .push(finding);
            fixture.request["next_epoch"]["result"]["relationships"].as_array_mut().unwrap().extend([
                json!({"from":{"type":"observation","key":"observation-two"},"to":{"type":"finding","key":key}}),
                json!({"from":{"type":"finding","key":key},"to":{"type":"risk","key":"risk-two"}}),
            ]);
        }
        let prepared =
            prepare_fixture(&fixture, Some(epoch_report::EpochViewFormat::Text)).unwrap();
        let native: Value = serde_json::from_slice(prepared.native_bytes()).unwrap();
        assert_eq!(
            native["assessment-results"]["metadata"]["version"],
            fixture.request["next_epoch"]["document"]["version"]
        );
        assert_eq!(
            native["assessment-results"]["results"][0]["description"],
            "PRIVATE EPOCH DESCRIPTION"
        );
        assert_eq!(
            native["assessment-results"]["results"][1]["description"],
            fixture.request["next_epoch"]["result"]["description"]
        );
        assert!(!String::from_utf8_lossy(prepared.report_json()).contains("café metadata"));
        assert!(prepared.view_bytes().unwrap().len() <= MAX_REPORT);
        assert_eq!(prepared_json(&prepared)["counts"]["added_objects"], 11);
    }

    /// Real aggregate projection refuses a near-bound authoring expansion rather than encoding then accepting a prefix.
    #[test]
    fn complete_shared_projection_refuses_large_actual_authored_native_growth() {
        let mut fixture = fixture(0.5);
        let moderate = "e".repeat(32 * 1024);
        for field in ["title", "description", "statement"] {
            fixture.request["next_epoch"]["result"]["risks"][0][field] = json!(moderate);
        }
        fixture.request["next_epoch"]["result"]["description"] = json!(moderate);
        fixture.request["next_epoch"]["result"]["findings"][0]["description"] = json!(moderate);
        fixture.request["next_epoch"]["result"]["observations"][0]["description"] = json!(moderate);
        let bounded = prepare_fixture(&fixture, None).unwrap();
        bounded.verify_inputs().unwrap();
        drop(bounded);
        let expanded = "e".repeat(64 * 1024);
        for field in ["title", "description", "statement"] {
            fixture.request["next_epoch"]["result"]["risks"][0][field] = json!(expanded);
        }
        fixture.request["next_epoch"]["result"]["description"] = json!(expanded);
        fixture.request["next_epoch"]["result"]["title"] = json!(expanded);
        fixture.request["next_epoch"]["result"]["findings"][0]["title"] = json!(expanded);
        fixture.request["next_epoch"]["result"]["findings"][0]["description"] = json!(expanded);
        fixture.request["next_epoch"]["result"]["observations"][0]["title"] = json!(expanded);
        fixture.request["next_epoch"]["result"]["observations"][0]["description"] = json!(expanded);
        let fault =
            prepare_fixture(&fixture, Some(epoch_report::EpochViewFormat::Html)).err().unwrap();
        assert!(fault.to_string().contains("shared conservative projection bound"));
        assert!(!fixture.root.join("after.json").exists());
        assert!(!fixture.root.join("report.json").exists());
    }

    /// Missing/duplicate/foreign/older risk classifications cannot replace complete explicit latest tuple membership.
    #[test]
    fn actual_complete_classification_refuses_missing_duplicate_and_wrong_tuple() {
        for alteration in 0..4 {
            let mut fixture = fixture(0.5);
            match alteration {
                0 => fixture.request["new_risks"] = json!([]),
                1 => {
                    let duplicate = fixture.request["new_risks"][0].clone();
                    fixture.request["new_risks"].as_array_mut().unwrap().push(duplicate);
                }
                2 => {
                    fixture.request["new_risks"][0]["prior"] =
                        fixture.request["seed_families"][0]["risk"].clone();
                    fixture.request["new_risks"][0]["family_key"] = json!("family-two");
                }
                _ => {
                    fixture.request["seed_families"][0]["risk"]["expected_sha256"] =
                        json!("f".repeat(64));
                }
            }
            assert!(prepare_fixture(&fixture, None).is_err());
        }
    }

    /// The original unsafe request spelling is refused before any capture can normalize it away.
    #[test]
    fn original_request_aliases_and_portable_leaf_limits_refuse() {
        for path in [
            "./request.json",
            "parent/../request.json",
            "parent//request.json",
            "parent/./request.json",
        ] {
            assert!(declaration_root(Path::new(path)).is_err(), "unsafe original path {path}");
        }
        assert!(declaration_root(Path::new("request.json")).is_ok());
        assert!(portable_json(Path::new(&format!("{}.json", "a".repeat(123))), true).is_ok());
        assert!(portable_json(Path::new(&format!("{}.json", "a".repeat(124))), true).is_err());
        for name in ["CON.json", "a.json.", "nested/output.json", "é.json", "a.json/child.json"] {
            assert!(portable_json(Path::new(name), true).is_err(), "unsupported leaf {name}");
        }
    }

    /// Output metadata and folded namespace collisions refuse before opening missing original inputs.
    #[test]
    fn both_destinations_preflight_existing_files_directories_and_casefold_aliases() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let request = root.join("missing-request.json");
        std::fs::write(root.join("report.json"), b"sentinel").unwrap();
        let failure =
            prepare(&request, Path::new("after.json"), Path::new("report.json")).err().unwrap();
        assert!(failure.to_string().contains("destination exists"));
        assert_eq!(std::fs::read(root.join("report.json")).unwrap(), b"sentinel");
        std::fs::create_dir(root.join("after.json")).unwrap();
        assert!(prepare(&request, Path::new("after.json"), Path::new("new-report.json")).is_err());
        assert!(prepare(&request, Path::new("same.json"), Path::new("SAME.json")).is_err());
        let capture = CaptureSession::new(&root).unwrap();
        preflight_destination(&capture, Path::new("missing.json")).unwrap();
    }

    /// Existing output symlinks refuse before source capture without altering their targets.
    #[cfg(unix)]
    #[test]
    fn existing_destination_link_is_not_followed_or_replaced() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("sentinel.json"), b"owned sentinel").unwrap();
        std::os::unix::fs::symlink("sentinel.json", root.join("report.json")).unwrap();
        assert!(
            prepare(
                &root.join("missing-request.json"),
                Path::new("after.json"),
                Path::new("report.json")
            )
            .is_err()
        );
        assert_eq!(std::fs::read(root.join("sentinel.json")).unwrap(), b"owned sentinel");
    }

    /// Closed duplicate-safe request admission rejects duplicate known root and nested keys before typed decoding.
    #[test]
    fn raw_request_duplicate_known_keys_and_null_omissions_refuse() {
        let fixture = fixture(0.5);
        let raw = serde_json::to_string(&fixture.request).unwrap();
        parse_request(raw.as_bytes()).unwrap();
        let duplicate = raw.replacen("\"schema_version\":\"forge.assessment-epoch-append/1\"", "\"schema_version\":\"forge.assessment-epoch-append/1\",\"schema_version\":\"forge.assessment-epoch-append/1\"", 1);
        assert_ne!(duplicate, raw);
        assert!(parse_request(duplicate.as_bytes()).is_err());
        let duplicate_title = raw.replacen(
            "\"title\":\"Caller assessment\"",
            "\"title\":\"Caller assessment\",\"title\":\"Caller assessment\"",
            1,
        );
        assert_ne!(duplicate_title, raw);
        assert!(parse_request(duplicate_title.as_bytes()).is_err());
        let mut omitted = fixture.request.clone();
        omitted["next_epoch"]["result"]["risks"][0].as_object_mut().unwrap().remove("severity");
        assert!(parse_request(&serde_json::to_vec(&omitted).unwrap()).is_err());
    }

    /// Exact family domain accepts whitespace and multibyte boundaries without weakening legacy trimmed source keys.
    #[test]
    fn exact_family_utf8_boundary_and_legacy_key_admission_are_distinct() {
        assert_eq!(family_key(&json!(" ")).unwrap(), " ");
        assert!(family_key(&json!("é".repeat(128))).is_ok());
        assert!(family_key(&json!("é".repeat(129))).is_err());
        assert!(family_key(&json!("")).is_err());
        assert!(text(&json!(" ")).is_err());
        let mut fixture = fixture(0.5);
        fixture.request["new_risks"][0]["family_key"] = json!(" ");
        let prepared = prepare_fixture(&fixture, None).unwrap();
        assert!(
            prepared_json(&prepared)["families"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["family_key"] == " ")
        );
    }

    /// Count and retained writers enforce inclusive complete fragment+LF ceilings without partial growth.
    #[test]
    fn retained_writers_and_checked_arithmetic_refuse_before_growth() {
        let value = json!({"quoted":"\"\\é"});
        let actual = serde_json::to_vec(&value).unwrap();
        assert_eq!(encoded_count(&value, actual.len()).unwrap(), actual.len());
        assert!(encoded_count(&value, actual.len() - 1).is_err());
        assert_eq!(encode_limited(&value, actual.len() + 1, true).unwrap().len(), actual.len() + 1);
        assert!(encode_limited(&value, actual.len(), true).is_err());
        let mut writer = BufferWriter { bytes: Vec::new(), limit: 2 };
        writer.write_all(b"ab").unwrap();
        assert!(writer.write_all(b"c").is_err());
        assert_eq!(writer.bytes, b"ab");
        writer.flush().unwrap();
        let mut counter = CountWriter { count: 0, limit: 1 };
        counter.write_all(b"a").unwrap();
        counter.flush().unwrap();
        assert!(counter.write_all(b"b").is_err());
        assert_eq!(counter.count, 1);
        assert!(add(usize::MAX, 1).is_err());
        assert!(mul(usize::MAX, 2).is_err());
    }

    /// Synthetic full-tree predicate controls distinguish every preserved Value/container from the only two allowed leaves.
    #[test]
    fn complete_virtual_reverse_oracle_refuses_tree_and_history_mutations() {
        let prior = json!({"assessment-results":{"uuid":UUID,"metadata":{"version":"1","last-modified":"2026-01-01T00:00:00Z","title":"té\n\t\"\\","extra":{"keep":[1,null,{"x":true}]}},"results":[{"uuid":UUID,"description":"old\ntext","array":[1,2]}],"back-matter":{"resources":[]}},"extension":{"complete":[1,null,2]}});
        let next = json!({"uuid":"11111111-1111-4111-8111-111111111111","description":"new"});
        let mut final_value = prior.clone();
        final_value["assessment-results"]["metadata"]["version"] = json!("2");
        final_value["assessment-results"]["metadata"]["last-modified"] =
            json!("2026-01-02T00:00:00Z");
        final_value["assessment-results"]["results"].as_array_mut().unwrap().push(next.clone());
        reverse_oracle(&prior, &final_value, &next).unwrap();
        for alteration in 0..5 {
            let mut bad = final_value.clone();
            match alteration {
                0 => bad["assessment-results"]["results"][0]["description"] = json!("changed"),
                1 => {
                    bad["assessment-results"]["metadata"]["extra"]["keep"] =
                        json!([null,1,{"x":true}]);
                }
                2 => {
                    bad["assessment-results"].as_object_mut().unwrap().remove("back-matter");
                }
                3 => bad["extension"]["complete"] = json!([1, 2]),
                _ => bad["unexpected"] = json!(true),
            }
            assert!(reverse_oracle(&prior, &bad, &next).is_err());
        }
    }

    /// Real selected inventory may select an earlier epoch but cannot truncate complete append history or continuity.
    #[test]
    fn actual_source_selection_stays_distinct_from_all_epoch_continuity() {
        let mut fixture = subsequent_fixture();
        let native: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("after.json")).unwrap())
                .unwrap();
        fixture.request["prior_source"]["result"] =
            json!({"key":"epoch-one","uuid":native["assessment-results"]["results"][0]["uuid"]});
        let prepared = prepare_subsequent(&fixture).unwrap();
        let report = prepared_json(&prepared);
        assert_eq!(report["counts"]["before_epochs"], 2);
        assert_eq!(report["counts"]["before_objects"], 6);
        assert_eq!(report["continuity_edges"][0]["predecessor"]["key"], "risk-two");
        assert_eq!(report["epochs"][0]["key"], "epoch-one");
        assert_eq!(report["epochs"][1]["key"], "epoch-two");
    }

    /// Actual touching/zero windows compare parsed instants across different offset spellings; out-of-window assertions refuse.
    #[test]
    fn genuine_window_offsets_touching_and_provenance_boundaries() {
        let mut fixture = fixture(0.5);
        let boundary = "2026-01-02T01:00:00+01:00";
        fixture.request["next_epoch"]["result"]["start"] = json!(boundary);
        fixture.request["next_epoch"]["result"]["end"] = json!(boundary);
        fixture.request["next_epoch"]["document"]["last_modified"] = json!(boundary);
        for field in ["observations", "findings", "risks"] {
            fixture.request["next_epoch"]["result"][field][0]["provenance"]["start"] =
                json!(boundary);
            fixture.request["next_epoch"]["result"][field][0]["provenance"]["end"] =
                json!(boundary);
        }
        fixture.request["new_risks"][0]["provenance"] =
            fixture.request["next_epoch"]["result"]["risks"][0]["provenance"].clone();
        let valid = prepare_fixture(&fixture, None).unwrap();
        assert_eq!(prepared_json(&valid)["epochs"][1]["start"], boundary);
        drop(valid);
        for alteration in 0..4 {
            let mut invalid = fixture.request.clone();
            match alteration {
                0 => invalid["next_epoch"]["result"]["end"] = Value::Null,
                1 => invalid["next_epoch"]["result"]["start"] = json!("2026-01-01T23:59:59Z"),
                2 => {
                    invalid["next_epoch"]["result"]["risks"][0]["provenance"]["start"] =
                        json!("2026-01-02T00:00:01Z");
                }
                _ => {
                    invalid["next_epoch"]["document"]["last_modified"] =
                        json!("2026-01-01T23:59:59Z");
                }
            }
            write_json(&fixture.root.join("invalid-window.json"), &invalid);
            assert!(
                prepare(
                    &fixture.root.join("invalid-window.json"),
                    Path::new("after.json"),
                    Path::new("report.json")
                )
                .is_err()
            );
        }
    }

    /// Actual current contexts, actor declarations and evidence scope remain exact rather than being repaired during append.
    #[test]
    fn genuine_context_actor_evidence_and_old_key_changes_refuse() {
        for alteration in 0..5 {
            let mut fixture = fixture(0.5);
            match alteration {
                0 => {
                    fixture.request["next_epoch"]["context"]["catalog"]["expected_sha256"] =
                        json!("f".repeat(64));
                }
                1 => {
                    fixture.request["next_epoch"]["roles"][0]["title"] =
                        json!("Different caller role");
                }
                2 => {
                    fixture.request["next_epoch"]["result"]["observations"][0]["evidence_keys"] =
                        json!(["invented-evidence"]);
                }
                3 => {
                    fixture.request["next_epoch"]["result"]["observations"][0]["key"] =
                        json!("observation-one");
                }
                _ => {
                    fixture.request["next_epoch"]["document"]["key"] =
                        json!("different-native-document");
                }
            }
            assert!(prepare_fixture(&fixture, None).is_err());
            assert!(!fixture.root.join("after.json").exists());
            assert!(!fixture.root.join("report.json").exists());
        }
    }

    /// A consumed third append supplies the actual previous edge ledger, then refuses forged reciprocal rows despite current raw re-pinning.
    #[test]
    fn genuine_older_edge_prefix_and_reciprocal_forgery_are_checked() {
        let mut fixture = subsequent_fixture();
        let third = prepare_subsequent(&fixture).unwrap();
        let native: Value = serde_json::from_slice(third.native_bytes()).unwrap();
        let report = prepared_json(&third);
        std::fs::write(fixture.root.join("after-three.json"), third.native_bytes()).unwrap();
        std::fs::write(fixture.root.join("report-three.json"), third.report_json()).unwrap();
        let native_hash = sha256_hex(third.native_bytes());
        let report_hash = sha256_hex(third.report_json());
        drop(third);
        let latest = &native["assessment-results"]["results"][2];
        let risk = &latest["risks"][0];
        fixture.request["prior_source"]["assessment_results"] = json!({"artifact":"after-three.json","href":"after-three.json","expected_sha256":native_hash,"root_uuid":native["assessment-results"]["uuid"],"document_version":"1.2.0","oscal_version":"1.2.3"});
        fixture.request["prior_source"]["result"] =
            json!({"key":"epoch-three","uuid":latest["uuid"]});
        fixture.request["prior_report"] =
            json!({"artifact":"report-three.json","expected_sha256":report_hash});
        let subject =
            fixture.request["next_epoch"]["result"]["observations"][0]["subjects"][0]["uuid"]
                .clone();
        let next = declaration(
            &fixture.request["prior_source"]["context"],
            &subject,
            "four",
            "2026-01-07T00:00:00Z",
            "2026-01-08T00:00:00Z",
            "1.3.0",
            0.75,
        );
        fixture.request["next_epoch"] = next;
        fixture.request["new_risks"] = json!([{"next_key":"risk-four","family_key":"family-two","prior":{"kind":"risk","key":"risk-three","uuid":risk["uuid"],"result_uuid":latest["uuid"],"expected_sha256":source::canonical_epoch_object_sha256(risk).unwrap()},"caller_asserted_continuity":true,"provenance":fixture.request["next_epoch"]["result"]["risks"][0]["provenance"]}]);
        write_json(&fixture.root.join("request-four.json"), &fixture.request);
        let fourth = prepare(
            &fixture.root.join("request-four.json"),
            Path::new("after-four.json"),
            Path::new("report-four.json"),
        )
        .unwrap();
        let output = prepared_json(&fourth);
        assert_eq!(output["continuity_edges"][0], report["continuity_edges"][0]);
        assert_eq!(
            &output["reciprocal_rows"].as_array().unwrap()[..2],
            report["reciprocal_rows"].as_array().unwrap().as_slice()
        );
        assert_eq!(output["counts"]["continuity_edges"], 2);
        assert_eq!(output["counts"]["reciprocal_rows"], 4);
        drop(fourth);
        let mut forged = report;
        forged["reciprocal_rows"][0]["direction"] = json!("successor");
        let raw = write_json(&fixture.root.join("report-three.json"), &forged);
        fixture.request["prior_report"]["expected_sha256"] = json!(sha256_hex(&raw));
        write_json(&fixture.root.join("request-four.json"), &fixture.request);
        assert!(
            prepare(
                &fixture.root.join("request-four.json"),
                Path::new("after-four.json"),
                Path::new("report-four.json")
            )
            .is_err()
        );
    }

    /// Genuine zero-risk old/next graphs keep complete zero family, edge and classification counts visible.
    #[test]
    fn genuine_zero_risk_profile_keeps_complete_zero_denominators() {
        let mut fixture = fixture(0.5);
        let path = fixture.root.join("old-authoring.json");
        let mut authoring: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        authoring["result"]["risks"] = json!([]);
        authoring["result"]["relationships"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["to"]["type"] != "risk");
        let raw = write_json(&path, &authoring);
        let typed = manifest::parse(&raw).unwrap();
        let context = assessment_results::context::load(&path, &typed.context).unwrap();
        let built = model::build(&typed, &context).unwrap();
        let native = serde_json::to_value(&built.artifact).unwrap();
        assessment_results::validate_completed_json(&native).unwrap();
        let native_bytes = write_json(&fixture.root.join("prior.json"), &native);
        fixture.request["prior_source"]["assessment_results"] =
            pin("prior.json", &native_bytes, &native, "assessment-results");
        fixture.request["next_epoch"]["result"]["risks"] = json!([]);
        fixture.request["next_epoch"]["result"]["relationships"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["to"]["type"] != "risk");
        fixture.request["seed_families"] = json!([]);
        fixture.request["new_risks"] = json!([]);
        let prepared = prepare_fixture(&fixture, None).unwrap();
        let report = prepared_json(&prepared);
        for key in [
            "before_risks",
            "after_risks",
            "families",
            "continuity_edges",
            "reciprocal_rows",
            "classified_next_risks",
            "uncontinued_families",
        ] {
            assert_eq!(report["counts"][key], 0, "missing complete zero {key}");
        }
        assert_eq!(report["counts"]["before_objects"], 2);
        assert_eq!(report["counts"]["after_objects"], 4);
        assert_eq!(report["families"], json!([]));
        assert_eq!(report["continuity_edges"], json!([]));
    }

    /// Frozen independent protocol vectors distinguish null versus explicit end without claiming real tuple authority.
    #[test]
    fn continuity_hash_matches_frozen_framed_protocol_vectors() {
        let predecessor = ReferenceRow {
            kind: "risk",
            key: "risk:epoch-1",
            uuid: Cow::Borrowed("00000000-0000-4000-8000-000000000001"),
            result_uuid: Cow::Borrowed("00000000-0000-4000-8000-000000000011"),
            expected_sha256: Cow::Borrowed(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        };
        let successor = ReferenceRow {
            kind: "risk",
            key: "risk:epoch-2",
            uuid: Cow::Borrowed("00000000-0000-4000-8000-000000000002"),
            result_uuid: Cow::Borrowed("00000000-0000-4000-8000-000000000012"),
            expected_sha256: Cow::Borrowed(
                "1111111111111111111111111111111111111111111111111111111111111111",
            ),
        };
        for (end, expected) in [
            (None, "f5509df28797b58758596c5aa941be615267e6e770bfa67797b4c32ed6e55e1b"),
            (
                Some("2026-10-04T00:00:00Z"),
                "0e9f0bf16880d8b57fe7b209b0426017538bde4bf00427bfff2be917e2106e08",
            ),
        ] {
            let provenance = ProvenanceRow {
                assessor_key: "assessor-1",
                role_id: "assessor",
                start: "2026-10-04T00:00:00Z",
                end,
                method: "EXAMINE",
                rationale_sha256: Cow::Borrowed(
                    "2222222222222222222222222222222222222222222222222222222222222222",
                ),
            };
            assert_eq!(
                continuity_hash("family-1", &predecessor, &successor, &provenance).unwrap(),
                expected
            );
            assert_ne!(
                continuity_hash("family-1 ", &predecessor, &successor, &provenance).unwrap(),
                expected
            );
        }
    }

    /// Persist one actual provided complete output through a no-replace fixture publisher, not a detached source constructor.
    fn publish_fixture(root: &Path, relative: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(relative))
            .map_err(|_| error("fixture no-replace publication refused"))?;
        file.write_all(bytes).map_err(|_| error("fixture publication failed"))
    }

    /// Deterministic writer faults at the actual CLI seam; retained bytes expose the honest flush boundary.
    struct FaultWriter {
        /// Actual accepted view bytes, possibly complete before a flush fault.
        bytes: Vec<u8>,
        /// Fixed fault before any view fragment is accepted.
        fail_write: bool,
        /// Fixed fault after `write_all` has completed, without pretending bytes were retracted.
        fail_flush: bool,
    }

    impl io::Write for FaultWriter {
        /// Accept complete actual bytes or refuse before growth under the selected deterministic write fault.
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail_write {
                return Err(io::Error::other("fixture write fault"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        /// Report an actual flush fault after any accepted bytes remain observable.
        fn flush(&mut self) -> io::Result<()> {
            if self.fail_flush { Err(io::Error::other("fixture flush fault")) } else { Ok(()) }
        }
    }

    /// An actual second publication failure leaves the complete first native file and never starts selected stdout.
    #[test]
    fn actual_publish_seam_second_failure_preserves_complete_first_file() {
        let fixture = fixture(0.5);
        let prepared =
            prepare_fixture(&fixture, Some(epoch_report::EpochViewFormat::Json)).unwrap();
        let mut stdout = Vec::new();
        let mut calls = 0;
        let outcome = super::super::assessment_epochs_cli::publish_prepared(
            &prepared,
            Path::new("after.json"),
            Path::new("report.json"),
            &crate::cli::AssessmentResultsFailOn::Any,
            &mut stdout,
            |root, relative, bytes| {
                calls += 1;
                if calls == 2 {
                    return Err(error("fixture second publication fault"));
                }
                publish_fixture(root, relative, bytes)
            },
        );
        assert!(outcome.is_err());
        assert_eq!(calls, 2);
        assert_eq!(
            std::fs::read(fixture.root.join("after.json")).unwrap(),
            prepared.native_bytes()
        );
        assert!(!fixture.root.join("report.json").exists());
        assert_eq!(stdout, [] as [u8; 0]);
    }

    /// Actual original drift after either publication blocks every remaining delivery through its held proof.
    #[test]
    fn actual_publish_seam_drift_blocks_remaining_deliveries() {
        for fault_after in [1, 2] {
            let fixture = fixture(0.5);
            let prepared =
                prepare_fixture(&fixture, Some(epoch_report::EpochViewFormat::Json)).unwrap();
            let mut stdout = Vec::new();
            let mut calls = 0;
            let outcome = super::super::assessment_epochs_cli::publish_prepared(
                &prepared,
                Path::new("after.json"),
                Path::new("report.json"),
                &crate::cli::AssessmentResultsFailOn::Never,
                &mut stdout,
                |root, relative, bytes| {
                    publish_fixture(root, relative, bytes)?;
                    calls += 1;
                    if calls == fault_after {
                        let mut original = std::fs::read(root.join("prior.json")).unwrap();
                        original.push(b' ');
                        std::fs::write(root.join("prior.json"), &original).unwrap();
                    }
                    Ok(())
                },
            );
            assert!(outcome.is_err());
            assert_eq!(calls, fault_after);
            assert_eq!(
                std::fs::read(fixture.root.join("after.json")).unwrap(),
                prepared.native_bytes()
            );
            if fault_after == 1 {
                assert!(!fixture.root.join("report.json").exists());
            } else {
                assert_eq!(
                    std::fs::read(fixture.root.join("report.json")).unwrap(),
                    prepared.report_json()
                );
            }
            assert_eq!(stdout, [] as [u8; 0]);
            assert!(prepared.verify_inputs().is_err());
        }
    }

    /// Actual writer write/flush errors leave a complete published pair; flush cannot retract an already written complete view.
    #[test]
    fn actual_publish_seam_writer_faults_preserve_pair_and_error() {
        for fail_flush in [false, true] {
            let fixture = fixture(0.5);
            let prepared =
                prepare_fixture(&fixture, Some(epoch_report::EpochViewFormat::Json)).unwrap();
            let mut writer = FaultWriter { bytes: Vec::new(), fail_write: !fail_flush, fail_flush };
            let outcome = super::super::assessment_epochs_cli::publish_prepared(
                &prepared,
                Path::new("after.json"),
                Path::new("report.json"),
                &crate::cli::AssessmentResultsFailOn::Never,
                &mut writer,
                publish_fixture,
            );
            assert!(outcome.is_err());
            assert_eq!(
                std::fs::read(fixture.root.join("after.json")).unwrap(),
                prepared.native_bytes()
            );
            assert_eq!(
                std::fs::read(fixture.root.join("report.json")).unwrap(),
                prepared.report_json()
            );
            if fail_flush {
                assert_eq!(writer.bytes, prepared.view_bytes().unwrap());
            } else {
                assert_eq!(writer.bytes, [] as [u8; 0]);
            }
            prepared.verify_inputs().unwrap();
        }
    }

    /// With no requested view the real port publishes both complete files without touching even a faulting writer.
    #[test]
    fn actual_publish_seam_no_view_has_no_implicit_stdout() {
        let fixture = fixture(0.5);
        let prepared = prepare_fixture(&fixture, None).unwrap();
        let mut writer = FaultWriter { bytes: Vec::new(), fail_write: true, fail_flush: true };
        let review = super::super::assessment_epochs_cli::publish_prepared(
            &prepared,
            Path::new("after.json"),
            Path::new("report.json"),
            &crate::cli::AssessmentResultsFailOn::Any,
            &mut writer,
            publish_fixture,
        )
        .unwrap();
        assert!(review);
        assert_eq!(writer.bytes, [] as [u8; 0]);
        assert_eq!(
            std::fs::read(fixture.root.join("after.json")).unwrap(),
            prepared.native_bytes()
        );
        assert_eq!(
            std::fs::read(fixture.root.join("report.json")).unwrap(),
            prepared.report_json()
        );
    }

    /// A normalized descendant companion URI reaches actual native receipts and consumes the complete captured-path bound.
    #[test]
    fn genuine_descendant_companion_path_is_accounted_from_actual_context() {
        let mut fixture = fixture(0.5);
        let catalog: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("catalog.json")).unwrap())
                .unwrap();
        std::fs::create_dir(fixture.root.join("companions")).unwrap();
        let catalog_bytes = write_json(&fixture.root.join("companions/catalog.json"), &catalog);
        let mut profile: Value =
            serde_json::from_slice(&std::fs::read(fixture.root.join("profile.json")).unwrap())
                .unwrap();
        profile["profile"]["imports"][0]["href"] = json!("companions/catalog.json");
        let profile_bytes = write_json(&fixture.root.join("profile.json"), &profile);
        let mut context = fixture.request["prior_source"]["context"].clone();
        context["catalog"] = pin("companions/catalog.json", &catalog_bytes, &catalog, "catalog");
        context["profile"] = pin("profile.json", &profile_bytes, &profile, "profile");
        let author_path = fixture.root.join("old-authoring.json");
        let mut original: Value =
            serde_json::from_slice(&std::fs::read(&author_path).unwrap()).unwrap();
        original["context"] = context.clone();
        let raw = write_json(&author_path, &original);
        let typed = manifest::parse(&raw).unwrap();
        let loaded = assessment_results::context::load(&author_path, &typed.context).unwrap();
        let built = model::build(&typed, &loaded).unwrap();
        let native = serde_json::to_value(&built.artifact).unwrap();
        assessment_results::validate_completed_json(&native).unwrap();
        let native_bytes = write_json(&fixture.root.join("prior.json"), &native);
        fixture.request["prior_source"]["context"] = context.clone();
        fixture.request["next_epoch"]["context"] = context;
        fixture.request["prior_source"]["assessment_results"] =
            pin("prior.json", &native_bytes, &native, "assessment-results");
        fixture.request["seed_families"][0]["risk"]["expected_sha256"] = json!(
            source::canonical_epoch_object_sha256(
                &native["assessment-results"]["results"][0]["risks"][0]
            )
            .unwrap()
        );
        let prepared = prepare_fixture(&fixture, None).unwrap();
        let appended: Value = serde_json::from_slice(prepared.native_bytes()).unwrap();
        let resources =
            appended["assessment-results"]["back-matter"]["resources"].as_array().unwrap();
        assert!(resources.iter().any(|row| row["rlinks"][0]["href"] == "companions/catalog.json"));
        assert!(
            !String::from_utf8_lossy(prepared.report_json()).contains("companions/catalog.json")
        );
        assert_eq!(prepared.input_paths().count(), 6);
        prepared.verify_inputs().unwrap();
    }
}

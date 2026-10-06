//! Caller-supplied first-plan risk authoring over one actual seven-original proof.
//!
//! This producer adds neither actor authority nor remediation/terminal approval.
//! Original JSON values are retained; current workflow and native validators are
//! consumed before any successful authoring output leaves the private holder.

use std::collections::BTreeSet;
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use serde::Serialize;
use serde_json::Value;

use super::manifest::{ArtifactManifest, SourceKind, SourceManifest, SourceReference};
use super::{manifest, source, workflow, workflow_model};
use crate::ForgeError;
use crate::evidence_capture::{CaptureProof, CaptureRole, CaptureSession, MAX_PROJECTION_BYTES};
use crate::json_strict::{self, Limits};

/// Existing consumer-compatible raw/encoded authoring ceiling, including final LF.
const MAX_DECLARATION_BYTES: usize = 4 * 1024 * 1024;
/// Offline closed profile; schema compilation adds no source or actor authority.
static VALIDATOR: OnceLock<Result<jsonschema::Validator, &'static str>> = OnceLock::new();

/// Complete output with actual original holders; no detached public constructor exists.
pub(crate) struct PreparedRiskAuthoring {
    /// Original admitted workflow values encoded under the complete four MiB cap.
    bytes: Vec<u8>,
    /// Qualified actual root used only for private source/output base calculation.
    root: PathBuf,
    /// All seven actual file generations, root ancestry and reserved output guard.
    proof: CaptureProof,
}

impl PreparedRiskAuthoring {
    /// Borrow complete caller-supplied authoring bytes without publication authority.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Borrow the qualified actual source base for Root's same-parent publication.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Reconcile every actual original generation and safe ancestor before output.
    /// # Errors
    /// Refuses any missing, changed, aliased or unsafe original using a fixed reason.
    pub(crate) fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.proof.verify_inputs().map_err(|_| error("captured risk authoring inputs changed"))
    }

    /// Borrow only actual complete input paths for internal output collision checks.
    pub(crate) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.proof.input_paths()
    }
}

/// Prepare a complete first-plan workflow from explicit declarations and five native originals.
///
/// All schema/date/relationship/native projection and original-generation checks
/// precede successful output. No assessment prose, owner, date, actor, history or
/// hidden clock is inferred. Root owns actual stdout or one no-replace publication.
/// # Errors
/// Refuses invalid original spelling, bounded/closed declarations, stale source
/// pins, tuple unions, native projection, terminal assertions or changed originals.
pub(crate) fn prepare(
    scaffold_path: &Path,
    authoring_relative: &Path,
    as_of: &str,
    output_relative: Option<&Path>,
) -> Result<PreparedRiskAuthoring, ForgeError> {
    workflow::validate_authoring_date(as_of)
        .map_err(|_| error("as-of must be a canonical actual full date"))?;
    let (root, scaffold_relative) = declaration_root(scaffold_path)?;
    validate_json_relative(authoring_relative, false)?;
    if let Some(output) = output_relative {
        validate_json_relative(output, true)?;
    }
    let mut capture = CaptureSession::new(&root)
        .map_err(|_| error("risk authoring root is unsafe or unavailable"))?;
    if let Some(output) = output_relative {
        capture
            .reserve_output(output)
            .map_err(|_| error("risk authoring output namespace is invalid"))?;
        preflight_destination(&capture, output)?;
    }
    let scaffold = capture
        .required(
            &scaffold_relative,
            CaptureRole::RiskScaffoldDeclaration,
            manifest::MAX_MANIFEST_BYTES,
        )
        .map_err(|_| error("risk scaffold original is unsafe or unavailable"))?;
    let scaffold = manifest::parse(scaffold.bytes())
        .map_err(|_| error("risk scaffold must be a closed valid empty declaration"))?;
    let authoring = capture
        .required(
            authoring_relative,
            CaptureRole::RiskAuthoringRequest,
            manifest::MAX_MANIFEST_BYTES,
        )
        .map_err(|_| error("risk authoring original is unsafe or unavailable"))?;
    let request = parse_request(authoring.bytes())?;
    let accounting = preflight(&request, &mut capture)?;
    let original = &request["workflow"];
    let compact = encode(original, false)?;
    let admitted = workflow::parse(&compact)
        .map_err(|_| error("risk authoring does not satisfy the current workflow rules"))?;
    drop(compact);
    if admitted.document.key != scaffold.document.key
        || !same_source(&admitted.source, &scaffold.source)
    {
        return Err(error("risk authoring source or document identity differs from scaffold"));
    }
    let actual_path = capture.root().join(&scaffold_relative);
    let sources = source::load_with_capture(&actual_path, &admitted.source, &mut capture)
        .map_err(|_| error("risk source originals or complete native relationships are invalid"))?;
    validate_union(&request, &admitted, &sources)?;
    let native = workflow_model::render(&admitted)
        .map_err(|_| error("risk authoring does not satisfy the current native projection"))?;
    if native.len() > accounting.native_upper {
        return Err(error("risk native projection exceeded its conservative admission"));
    }
    drop(native);
    let bytes = encode(original, true)?;
    if bytes.len() != accounting.workflow_bytes {
        return Err(error("risk authoring encoding differs from its complete admission"));
    }
    verify_encoding(&bytes, original)?;
    let root = capture.root().to_path_buf();
    let proof = capture.finish();
    if proof.captured_original_generations() != 7 {
        return Err(error("risk authoring proof does not contain seven distinct originals"));
    }
    let prepared = PreparedRiskAuthoring { bytes, root, proof };
    prepared.verify_inputs()?;
    Ok(prepared)
}

/// Refuse an existing or unobservable destination before any original capture or growth.
///
/// Leaf metadata follows no symlink and grants no atomic absence/publication proof.
/// Original/root rechecks and the final no-replace publisher remain authoritative.
fn preflight_destination(capture: &CaptureSession, relative: &Path) -> Result<(), ForgeError> {
    match std::fs::symlink_metadata(capture.root().join(relative)) {
        Err(fault) if fault.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) | Err(_) => Err(error("risk authoring destination exists or is unavailable")),
    }
}

/// Qualify original caller spelling before absolute/parent/join operations can erase aliases.
fn declaration_root(path: &Path) -> Result<(PathBuf, PathBuf), ForgeError> {
    if path.as_os_str().is_empty()
        || path.components().any(|part| matches!(part, Component::CurDir | Component::ParentDir))
        || !crate::linkage::has_normalized_path_spelling(path)
    {
        return Err(error("risk scaffold path spelling is not normalized"));
    }
    let name = path.file_name().ok_or_else(|| error("risk scaffold needs a filename"))?;
    let relative = PathBuf::from(name);
    validate_json_relative(&relative, true)?;
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let root = std::path::absolute(parent)
        .map_err(|_| error("risk scaffold directory cannot be resolved"))?;
    Ok((root, relative))
}

/// Preserve the existing slash-separated portable local JSON subset without rewriting spelling.
fn validate_json_relative(path: &Path, filename_only: bool) -> Result<(), ForgeError> {
    crate::linkage::fresh::validate_relative(path)
        .map_err(|_| error("risk authoring path must be a normalized descendant"))?;
    let value = path.to_str().ok_or_else(|| error("risk authoring path must be UTF8"))?;
    if value.len() > manifest::MAX_STRING_BYTES
        || value.chars().any(char::is_control)
        || value.contains(['\\', ':', '?', '#'])
        || path.extension().and_then(|part| part.to_str()) != Some("json")
        || (filename_only && path.components().count() != 1)
        || value.split('/').any(unsafe_segment)
    {
        return Err(error("risk authoring path must be a portable local JSON name"));
    }
    Ok(())
}

/// Reject Windows device/trailing aliases on every supported host before file IO.
fn unsafe_segment(segment: &str) -> bool {
    if segment.is_empty() || matches!(segment, "." | "..") || segment.ends_with(['.', ' ']) {
        return true;
    }
    let stem = segment.split('.').next().unwrap_or_default().to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|number| {
                matches!(
                    number,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        })
}

/// Strict-parse the actual bounded original and consume the finite offline first-plan profile.
fn parse_request(bytes: &[u8]) -> Result<Value, ForgeError> {
    if bytes.len() > MAX_DECLARATION_BYTES {
        return Err(error("risk authoring request exceeds four MiB"));
    }
    let value = json_strict::parse_value(
        bytes,
        "risk authoring request",
        Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("risk authoring request is not bounded duplicate-free JSON"))?;
    let validator = VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../schemas/forge.poam-risk-authoring-1.schema.json"
            ))
            .map_err(|_| "risk authoring schema is malformed")?;
            jsonschema::validator_for(&schema).map_err(|_| "risk authoring schema is unavailable")
        })
        .as_ref()
        .map_err(|message| error(message))?;
    if !validator.is_valid(&value) {
        return Err(error("risk authoring request does not match the closed first-plan profile"));
    }
    Ok(value)
}

/// Strict-reparse complete bounded output and require every original value, array and null.
///
/// The temporary decoded form has the existing raw/depth/string limits; encoded
/// admission is not a heap limit. This check grants no detached source authority.
fn verify_encoding(bytes: &[u8], original: &Value) -> Result<(), ForgeError> {
    if bytes.len() > MAX_DECLARATION_BYTES {
        return Err(error("risk authoring output exceeds four MiB"));
    }
    let decoded = json_strict::parse_value(
        bytes,
        "risk authoring output",
        Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("risk authoring output is not bounded duplicate-free JSON"))?;
    if &decoded != original {
        return Err(error("risk authoring output differs from the complete original workflow"));
    }
    drop(decoded);
    Ok(())
}

/// Complete repeated structural counts, distinct from the deduplicated reviewed-risk union.
#[derive(Default)]
struct Counts {
    /// Complete supplied role records.
    roles: usize,
    /// Complete supplied party records.
    parties: usize,
    /// Complete supplied work items.
    items: usize,
    /// Complete supplied milestone records.
    milestones: usize,
    /// Every reference occurrence, including exact repeats across items.
    references: usize,
    /// Every supplied item/milestone owner record.
    owners: usize,
    /// Every supplied history record and its explicit actor.
    events: usize,
    /// Every supplied milestone dependency occurrence.
    dependencies: usize,
    /// Complete explicit reviewed declaration count, before uniqueness checking.
    reviewed: usize,
}

impl Counts {
    /// Traverse borrowed closed values before any typed/index/native growth.
    fn read(request: &Value) -> Result<Self, ForgeError> {
        let value = &request["workflow"];
        let items = array(value, "items")?;
        let mut counts = Self {
            roles: array(value, "roles")?.len(),
            parties: array(value, "parties")?.len(),
            items: items.len(),
            reviewed: array(request, "reviewed_risks")?.len(),
            ..Self::default()
        };
        for item in items {
            counts.references = checked_add(counts.references, array(item, "source_refs")?.len())?;
            counts.owners = checked_add(counts.owners, array(item, "owners")?.len())?;
            counts.events = checked_add(counts.events, array(item, "history")?.len())?;
            let steps = array(item, "milestones")?;
            counts.milestones = checked_add(counts.milestones, steps.len())?;
            for step in steps {
                counts.owners = checked_add(counts.owners, array(step, "owners")?.len())?;
                counts.events = checked_add(counts.events, array(step, "history")?.len())?;
                counts.dependencies =
                    checked_add(counts.dependencies, array(step, "depends_on")?.len())?;
            }
        }
        Ok(counts)
    }

    /// Charge complete local relationships, including one actor per event, before insertion.
    fn relationships(&self) -> Result<usize, ForgeError> {
        sum(&[
            self.roles,
            self.parties,
            self.reviewed,
            self.items,
            self.milestones,
            self.references,
            self.owners,
            self.dependencies,
            checked_mul(self.events, 2)?,
        ])
    }

    /// Count native fixed record and repeated generated-field overhead conservatively.
    fn native_upper(
        &self,
        compact: usize,
        ar_path: usize,
        paths: usize,
    ) -> Result<usize, ForgeError> {
        let records = sum(&[1, self.roles, self.parties, self.items, self.milestones])?;
        let occurrences =
            sum(&[self.references, self.owners, self.dependencies, checked_mul(self.events, 2)?])?;
        let uri = checked_add(checked_mul(ar_path, self.references)?, paths)?;
        sum(&[
            checked_mul(compact, 6)?,
            checked_mul(records, 8192)?,
            checked_mul(occurrences, 512)?,
            checked_mul(uri, 18)?,
        ])
    }
}

/// Complete nonretaining admission results, not native authority or a heap ceiling.
struct Preflight {
    /// Conservative encoded/derived native bound pinned to the existing finite renderer.
    native_upper: usize,
    /// Exact pretty authoring serialization plus LF byte count.
    workflow_bytes: usize,
}

/// Admit all repeated relationships and combined native/authoring projection before growth.
fn preflight(request: &Value, capture: &mut CaptureSession) -> Result<Preflight, ForgeError> {
    let counts = Counts::read(request)?;
    if counts.milestones > workflow::MAX_MILESTONES || counts.events > workflow::MAX_HISTORY_EVENTS
    {
        return Err(error("risk authoring exceeds a complete workflow record bound"));
    }
    capture
        .relationships(counts.relationships()?)
        .map_err(|_| error("risk authoring exceeds the complete relationship budget"))?;
    let value = &request["workflow"];
    let compact = measure(value, false)?;
    let workflow_bytes = measure(value, true)?;
    let source = &value["source"];
    let ar_path = string(&source["assessment_results"], "artifact")?.len();
    let mut paths = ar_path;
    for name in ["assessment_plan", "ssp", "profile", "catalog"] {
        paths = checked_add(paths, string(&source["context"][name], "artifact")?.len())?;
    }
    let native_upper = counts.native_upper(compact, ar_path, paths)?;
    let combined = checked_add(checked_mul(native_upper, 3)?, workflow_bytes)?;
    if combined > MAX_PROJECTION_BYTES {
        return Err(error("risk authoring exceeds conservative combined projection admission"));
    }
    Ok(Preflight { native_upper, workflow_bytes })
}

/// Compare every original source field, without canonical map cloning or hash-only summaries.
fn same_source(left: &SourceManifest, right: &SourceManifest) -> bool {
    left.result.uuid == right.result.uuid
        && left.result.key == right.result.key
        && same_artifact(&left.assessment_results, &right.assessment_results)
        && same_artifact(&left.context.assessment_plan, &right.context.assessment_plan)
        && same_artifact(&left.context.ssp, &right.context.ssp)
        && same_artifact(&left.context.profile, &right.context.profile)
        && same_artifact(&left.context.catalog, &right.context.catalog)
        && left.context.evidence_index.is_none()
        && right.context.evidence_index.is_none()
}

/// Compare all six exact native path/import/hash/identity/version pins.
fn same_artifact(left: &ArtifactManifest, right: &ArtifactManifest) -> bool {
    left.artifact == right.artifact
        && left.href == right.href
        && left.expected_sha256 == right.expected_sha256
        && left.root_uuid == right.root_uuid
        && left.document_version == right.document_version
        && left.oscal_version == right.oscal_version
}

/// Require exact current risk tuples and a complete unique union, preserving cross-item repeats.
fn validate_union(
    request: &Value,
    admitted: &workflow::WorkflowManifest,
    sources: &source::CapturedSource,
) -> Result<(), ForgeError> {
    let rows = array(request, "reviewed_risks")?;
    let mut reviewed = Vec::with_capacity(rows.len());
    for row in rows {
        if row["caller_asserted_reviewed"] != Value::Bool(true) {
            return Err(error("every selected risk requires an explicit caller review assertion"));
        }
        let reference: SourceReference = serde_json::from_value(row["source_ref"].clone())
            .map_err(|_| error("reviewed risk tuple has invalid closed fields"))?;
        if reference.kind != SourceKind::Risk {
            return Err(error("reviewed selections must be risk tuples"));
        }
        reviewed.push(reference);
    }
    sources
        .validate_selection(&reviewed)
        .map_err(|_| error("reviewed risk tuples do not identify exact current source objects"))?;
    let declared: BTreeSet<_> = reviewed.iter().collect();
    let mut selected = BTreeSet::new();
    for item in &admitted.items {
        if item.source_refs.iter().any(|value| value.kind != SourceKind::Risk) {
            return Err(error("authored work may select only the declared reviewed risks"));
        }
        sources
            .validate_selection(&item.source_refs)
            .map_err(|_| error("item risk tuples do not identify exact current source objects"))?;
        selected.extend(item.source_refs.iter());
    }
    if selected != declared {
        return Err(error("authored risk union differs from the complete reviewed declaration"));
    }
    Ok(())
}

/// Borrow one schema-admitted finite array, retaining fixed errors if a future caller changes shape.
fn array<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], ForgeError> {
    value
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| error("risk authoring expected a closed array"))
}

/// Borrow one schema-admitted UTF8 string without converting authored data into diagnostics.
fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str, ForgeError> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| error("risk authoring expected a closed string"))
}

/// Reject arithmetic overflow rather than losing any complete record/byte denominator.
fn checked_add(left: usize, right: usize) -> Result<usize, ForgeError> {
    left.checked_add(right).ok_or_else(|| error("risk authoring admission arithmetic overflow"))
}

/// Reject expansion-factor overflow before buffer or index growth.
fn checked_mul(left: usize, right: usize) -> Result<usize, ForgeError> {
    left.checked_mul(right).ok_or_else(|| error("risk authoring admission arithmetic overflow"))
}

/// Sum a fixed borrowed group of complete counts with checked arithmetic.
fn sum(values: &[usize]) -> Result<usize, ForgeError> {
    values.iter().try_fold(0, |total, value| checked_add(total, *value))
}

/// Nonretaining serializer sink; exact counts are distinct from encoded buffers and runtime tests.
struct CountingWriter {
    /// Complete encoded bytes admitted so far, with no retained serialization fragments.
    written: usize,
}

impl io::Write for CountingWriter {
    /// Refuse each fragment before it exceeds the maintained four MiB consumer cap.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.written = self
            .written
            .checked_add(bytes.len())
            .filter(|value| *value <= MAX_DECLARATION_BYTES)
            .ok_or_else(|| io::Error::other("risk declaration bound"))?;
        Ok(bytes.len())
    }

    /// Finish the private count without external IO.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Measure compact or pretty-plus-LF serialization without a complete encoded allocation.
fn measure(value: &impl Serialize, pretty: bool) -> Result<usize, ForgeError> {
    let mut writer = CountingWriter { written: 0 };
    if pretty {
        serde_json::to_writer_pretty(&mut writer, value)
            .map_err(|_| error("risk workflow exceeds the complete encoding bound"))?;
        writer
            .write_all(b"\n")
            .map_err(|_| error("risk workflow exceeds the complete encoding bound"))?;
    } else {
        serde_json::to_writer(&mut writer, value)
            .map_err(|_| error("risk workflow exceeds the complete encoding bound"))?;
    }
    Ok(writer.written)
}

/// Private retained sink; no complete oversized buffer or valid prefix is returned.
struct BoundedWriter {
    /// Only the admitted prefix; all bytes stay private until serialization succeeds.
    bytes: Vec<u8>,
}

impl io::Write for BoundedWriter {
    /// Check each complete fragment before growing retained bytes.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes
            .len()
            .checked_add(bytes.len())
            .filter(|value| *value <= MAX_DECLARATION_BYTES)
            .ok_or_else(|| io::Error::other("risk declaration bound"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    /// Finish the private bounded sink without external IO.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encode the original admitted values under the cap, adding LF only to final pretty output.
fn encode(value: &impl Serialize, pretty: bool) -> Result<Vec<u8>, ForgeError> {
    let mut writer = BoundedWriter { bytes: Vec::new() };
    if pretty {
        serde_json::to_writer_pretty(&mut writer, value)
            .map_err(|_| error("risk workflow exceeds the complete encoding bound"))?;
        writer
            .write_all(b"\n")
            .map_err(|_| error("risk workflow exceeds the complete encoding bound"))?;
    } else {
        serde_json::to_writer(&mut writer, value)
            .map_err(|_| error("risk workflow exceeds the complete encoding bound"))?;
    }
    Ok(writer.bytes)
}

/// Emit only a fixed typed invalid reason, never author prose or private native errors.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(reason.to_string())
}

/// Proposed isolated controls; syntax/native projection fixtures grant no source-capture authority.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Closed syntax fixture with explicitly supplied fields and placeholder native pins.
    fn request() -> Value {
        let artifact = |name: &str| {
            json!({"artifact":name,"href":name,
            "expected_sha256":"a".repeat(64),"root_uuid":"11111111-1111-4111-8111-111111111111",
            "document_version":"1.0.0","oscal_version":"1.2.3"})
        };
        let reference = |key: &str, uuid: &str| {
            json!({"kind":"risk","key":key,"uuid":uuid,
            "result_uuid":"22222222-2222-4222-8222-222222222222","expected_sha256":"b".repeat(64)})
        };
        let source = json!({"assessment_results":artifact("assessment-results.json"),
            "result":{"uuid":"22222222-2222-4222-8222-222222222222","key":"result-key"},
            "context":{"assessment_plan":artifact("assessment-plan.json"),"ssp":artifact("ssp.json"),
                "profile":artifact("profile.json"),"catalog":artifact("catalog.json"),"evidence_index":null}});
        let first = reference("risk-a", "33333333-3333-4333-8333-333333333333");
        let second = reference("risk-b", "44444444-4444-4444-8444-444444444444");
        let item = |key: &str, selected: &Value| {
            json!({"key":key,"title":"Caller Item",
            "description":"Caller work","source_refs":[selected],"owners":[{
                "role_id":"owner","party_key":"caller","rationale":"Explicit responsibility"}],
            "target_date":"2026-12-01","state":"planned","history":[initial_event("item-plan")],
            "milestones":[{"key":"step","outcome":"Explicit outcome","target_date":"2026-11-01",
                "depends_on":[],"owners":[{"role_id":"owner","party_key":"caller","rationale":"Explicit milestone ownership"}],
                "state":"planned","history":[initial_event("step-plan")]}]})
        };
        json!({"schema_version":"forge.poam-risk-authoring/1","workflow":{
            "schema_version":"forge.poam/1","document":{"key":"plan-key","title":"Caller Plan",
                "version":"1.0.0","last_modified":"2026-10-04T12:00:00Z"},"source":source,
            "roles":[{"id":"owner","title":"Declared Owner"}],
            "parties":[{"key":"caller","type":"person","name":"Declared Caller"}],
            "items":[item("item-a",&first),item("item-b",&second)]},
            "reviewed_risks":[{"source_ref":first,"caller_asserted_reviewed":true},
                {"source_ref":second,"caller_asserted_reviewed":true}]})
    }

    /// Explicit initial attribution, time, transition and null closure, without defaults.
    fn initial_event(key: &str) -> Value {
        json!({"key":key,"actor":{"role_id":"owner","party_key":"caller"},
            "at":"2026-10-04T10:00:00Z","from":null,"to":"planned",
            "rationale":"Explicit planned work","closure":null})
    }

    /// Serialize syntax fixtures before the same strict raw/profile admission as production.
    fn parsed(value: &Value) -> Value {
        parse_request(&encode(value, false).unwrap()).unwrap()
    }

    /// Obtain an actual qualified owned directory for isolated budget controls only.
    fn session(directory: &tempfile::TempDir) -> CaptureSession {
        CaptureSession::new(&directory.path().canonicalize().unwrap()).unwrap()
    }

    /// Preserve every admitted workflow JSON value, ordered array and explicit null.
    #[test]
    fn complete_original_values_are_preserved_without_inferred_review_fields() {
        let value = parsed(&request());
        let bytes = encode(&value["workflow"], true).unwrap();
        verify_encoding(&bytes, &value["workflow"]).unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), value["workflow"]);
        assert!(bytes.ends_with(b"\n"));
        assert!(value["workflow"].get("reviewed_risks").is_none());
        assert!(value["workflow"]["items"][0]["history"][0]["closure"].is_null());
    }

    /// Final oracle refuses changed values, duplicate/trailing JSON, UTF8 and raw oversize.
    #[test]
    fn final_encoding_requires_complete_original_values_and_strict_bytes() {
        let value = parsed(&request());
        let original = &value["workflow"];
        let mut changed = original.clone();
        changed["items"][0]["history"][0]["closure"] = json!("private-mismatch");
        let message =
            verify_encoding(&encode(&changed, true).unwrap(), original).unwrap_err().to_string();
        assert!(!message.contains("private-mismatch"));
        let valid = String::from_utf8(encode(original, false).unwrap()).unwrap();
        let field = r#""title":"Caller Plan""#;
        assert_eq!(valid.matches(field).count(), 1);
        let duplicate = valid.replacen(field, r#""title":"Caller Plan","title":"Caller Plan""#, 1);
        assert!(verify_encoding(duplicate.as_bytes(), original).is_err());
        let mut trailing = encode(original, true).unwrap();
        trailing.extend_from_slice(b"{}");
        assert!(verify_encoding(&trailing, original).is_err());
        assert!(verify_encoding(&[0xff], original).is_err());
        assert!(verify_encoding(&vec![b' '; MAX_DECLARATION_BYTES + 1], original).is_err());
    }

    /// Refuse missing/false/string review, unknown fields and missing explicit nullable fields.
    #[test]
    fn closed_review_and_nullable_fields_cannot_be_defaulted() {
        let original = request();
        for replacement in [json!(false), json!("true"), Value::Null] {
            let mut value = original.clone();
            value["reviewed_risks"][0]["caller_asserted_reviewed"] = replacement;
            assert!(parse_request(&encode(&value, false).unwrap()).is_err());
        }
        let mut value = original.clone();
        value["workflow"]["items"][0]["history"][0].as_object_mut().unwrap().remove("closure");
        assert!(parse_request(&encode(&value, false).unwrap()).is_err());
        let mut value = original;
        value["private"] = json!("must-not-be-reported");
        let message = parse_request(&encode(&value, false).unwrap()).unwrap_err().to_string();
        assert!(!message.contains("must-not-be-reported"));
    }

    /// Require explicit new planned work rather than admitting previous/terminal assertions.
    #[test]
    fn first_plan_state_history_and_risk_kind_are_closed() {
        for state in ["in-progress", "cancelled", "completed-asserted", "accepted-risk-asserted"] {
            let mut value = request();
            value["workflow"]["items"][0]["state"] = json!(state);
            value["workflow"]["items"][0]["history"][0]["to"] = json!(state);
            assert!(parse_request(&encode(&value, false).unwrap()).is_err());
        }
        let mut value = request();
        value["workflow"]["items"][0]["source_refs"][0]["kind"] = json!("finding");
        assert!(parse_request(&encode(&value, false).unwrap()).is_err());
        let mut value = request();
        let event = value["workflow"]["items"][0]["history"][0].clone();
        value["workflow"]["items"][0]["history"].as_array_mut().unwrap().push(event);
        assert!(parse_request(&encode(&value, false).unwrap()).is_err());
    }

    /// Reject recursive duplicate keys, malformed UTF8, BOM, trailing values and raw overflow.
    #[test]
    fn malformed_originals_never_reach_profile_authority() {
        let valid = String::from_utf8(encode(&request(), false).unwrap()).unwrap();
        assert!(parse_request(valid.as_bytes()).is_ok());
        for (field, duplicate) in [
            (
                r#""schema_version":"forge.poam-risk-authoring/1""#,
                r#""schema_version":"forge.poam-risk-authoring/1","schema_version":"forge.poam-risk-authoring/1""#,
            ),
            (r#""title":"Caller Plan""#, r#""title":"Caller Plan","title":"Caller Plan""#),
        ] {
            assert_eq!(valid.matches(field).count(), 1);
            let duplicated = valid.replacen(field, duplicate, 1);
            assert!(parse_request(duplicated.as_bytes()).is_err());
        }
        for bytes in [b"{\"x\":1,\"x\":2}".as_slice(), b"\xef\xbb\xbf{}", b"\xff", b"{} {}"] {
            assert!(parse_request(bytes).is_err());
        }
        assert!(parse_request(&vec![b' '; MAX_DECLARATION_BYTES + 1]).is_err());
        let mut value = request();
        value["workflow"]["items"][0]["description"] =
            json!("x".repeat(manifest::MAX_STRING_BYTES + 1));
        assert!(parse_request(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    /// A generic parse-valid LF title must still fail the consumed native single-line schema.
    #[test]
    fn native_consumer_single_line_refusal_is_not_hidden_by_workflow_parse() {
        let mut value = request();
        value["workflow"]["document"]["title"] = json!("Caller\nPlan");
        let value = parsed(&value);
        let typed = workflow::parse(&encode(&value["workflow"], false).unwrap()).unwrap();
        assert!(workflow_model::render(&typed).is_err());
    }

    /// Valid multiline work/rationale survives native property escaping and stays under its bound.
    #[test]
    fn native_multiline_prose_and_repeated_references_fit_conservative_admission() {
        let mut value = request();
        value["workflow"]["items"][0]["description"] =
            json!("Caller\nwork\twith \"quotes\" and \\ slash");
        value["workflow"]["items"][0]["owners"][0]["rationale"] = json!("Line\nTwo\tExplicit");
        let repeat = value["workflow"]["items"][0]["source_refs"][0].clone();
        value["workflow"]["items"][1]["source_refs"].as_array_mut().unwrap().push(repeat);
        let value = parsed(&value);
        let directory = tempfile::tempdir().unwrap();
        let accounting = preflight(&value, &mut session(&directory)).unwrap();
        let typed = workflow::parse(&encode(&value["workflow"], false).unwrap()).unwrap();
        let native = workflow_model::render(&typed).unwrap();
        assert!(native.len() <= accounting.native_upper);
        assert_eq!(accounting.workflow_bytes, encode(&value["workflow"], true).unwrap().len());
        assert_eq!(Counts::read(&value).unwrap().references, 3);
    }

    /// Whole native-context/source pins, not only raw hash fields, are locked to the scaffold.
    #[test]
    fn every_source_pin_and_result_identity_is_compared() {
        let value = request();
        let original: SourceManifest =
            serde_json::from_value(value["workflow"]["source"].clone()).unwrap();
        assert!(same_source(&original, &original));
        for pointer in [
            "/assessment_results/artifact",
            "/assessment_results/href",
            "/assessment_results/expected_sha256",
            "/assessment_results/root_uuid",
            "/assessment_results/document_version",
            "/assessment_results/oscal_version",
            "/result/key",
            "/result/uuid",
            "/context/assessment_plan/href",
            "/context/ssp/root_uuid",
            "/context/profile/document_version",
            "/context/catalog/expected_sha256",
        ] {
            let mut changed = value["workflow"]["source"].clone();
            *changed.pointer_mut(pointer).unwrap() = json!("different");
            let changed: SourceManifest = serde_json::from_value(changed).unwrap();
            assert!(!same_source(&original, &changed), "changed pin was not compared: {pointer}");
        }
    }

    /// Distinct native source key and workflow key limits remain meaningful domains.
    #[test]
    fn source_keys_are_not_silently_narrowed_to_workflow_key_lengths() {
        let mut value = request();
        let long = "s".repeat(300);
        value["workflow"]["items"][0]["source_refs"][0]["key"] = json!(long);
        value["reviewed_risks"][0]["source_ref"]["key"] = json!(long);
        let value = parsed(&value);
        assert!(workflow::parse(&encode(&value["workflow"], false).unwrap()).is_ok());
        let mut value = value;
        value["workflow"]["items"][0]["key"] = json!("w".repeat(257));
        assert!(parse_request(&encode(&value, false).unwrap()).is_err());
    }

    /// Repetition charges complete denominators and cannot be hidden by the unique reviewed set.
    #[test]
    fn repeated_path_expansion_refuses_before_source_or_native_growth() {
        let mut value = request();
        value["workflow"]["source"]["assessment_results"]["artifact"] =
            json!(format!("{}.json", "é".repeat(10_000)));
        let template = value["workflow"]["items"][0].clone();
        value["workflow"]["items"] = json!(
            (0..32)
                .map(|index| {
                    let mut item = template.clone();
                    item["key"] = json!(format!("item-{index}"));
                    item
                })
                .collect::<Vec<_>>()
        );
        let value = parsed(&value);
        let directory = tempfile::tempdir().unwrap();
        assert!(preflight(&value, &mut session(&directory)).is_err());
        assert_eq!(Counts::read(&value).unwrap().references, 32);
        assert_eq!(array(&value, "reviewed_risks").unwrap().len(), 2);
    }

    /// A meaningful near-bound series changes complete caller repetition rather than prefix admission.
    #[test]
    fn conservative_boundary_has_admitted_and_refused_complete_inputs() {
        let mut last = true;
        let mut observed_boundary = false;
        for count in [2, 8, 32, 64, 128, 256, 512] {
            let mut value = request();
            let template = value["workflow"]["items"][0].clone();
            value["workflow"]["items"] = json!(
                (0..count)
                    .map(|index| {
                        let mut item = template.clone();
                        item["key"] = json!(format!("item-{index}"));
                        item
                    })
                    .collect::<Vec<_>>()
            );
            let value = parsed(&value);
            let directory = tempfile::tempdir().unwrap();
            let admitted = preflight(&value, &mut session(&directory)).is_ok();
            if last && !admitted {
                observed_boundary = true;
            }
            assert!(last || !admitted, "complete growth must not reset admission");
            last = admitted;
        }
        assert!(observed_boundary, "the series must cross the fixed combined ceiling");
    }

    /// An already consumed source graph cannot get a fresh relationship budget for authoring.
    #[test]
    fn native_and_authoring_relationships_share_one_complete_ledger() {
        let value = parsed(&request());
        let directory = tempfile::tempdir().unwrap();
        let mut capture = session(&directory);
        capture.relationships(crate::evidence_capture::MAX_RELATIONSHIPS - 1).unwrap();
        assert!(preflight(&value, &mut capture).is_err());
        assert!(checked_add(usize::MAX, 1).is_err());
        assert!(checked_mul(usize::MAX, 3).is_err());
    }

    /// Count-only and retained sinks include JSON escaping and final LF before each growth.
    #[test]
    fn escaped_bytes_and_final_lf_are_real_complete_encoding_bounds() {
        let value = json!("\u{0000}".repeat(MAX_DECLARATION_BYTES / 6 + 1));
        assert!(measure(&value, false).is_err());
        assert!(encode(&value, false).is_err());
        let mut writer = BoundedWriter { bytes: vec![b'x'; MAX_DECLARATION_BYTES] };
        assert!(writer.write_all(b"\n").is_err());
        assert_eq!(writer.bytes.len(), MAX_DECLARATION_BYTES);
        let ordinary = json!("line\nnext\tquote\"slash\\");
        assert_eq!(measure(&ordinary, true).unwrap(), encode(&ordinary, true).unwrap().len());
    }

    /// Original path spelling is refused before host normalization, not after unsafe joining.
    #[test]
    fn original_alias_and_portable_device_spellings_are_refused() {
        for raw in [
            "./scaffold.json",
            "nested/../scaffold.json",
            "nested//scaffold.json",
            "scaffold.json/",
        ] {
            assert!(declaration_root(Path::new(raw)).is_err(), "raw alias accepted: {raw}");
        }
        for raw in [
            "NUL.json",
            "nested/COM1.json",
            "nested/trailing./file.json",
            "wrong.yaml",
            "dir\\file.json",
        ] {
            assert!(
                validate_json_relative(Path::new(raw), false).is_err(),
                "portable alias accepted: {raw}"
            );
        }
        assert!(validate_json_relative(Path::new("requests/reviewed.json"), false).is_ok());
        assert!(validate_json_relative(Path::new("requests/output.json"), true).is_err());
    }

    /// Existing regular and directory destinations refuse without original retention or replacement.
    #[test]
    fn existing_destinations_are_refused_before_original_capture() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("file.json"), b"sentinel").unwrap();
        std::fs::create_dir(directory.path().join("directory.json")).unwrap();
        for name in ["file.json", "directory.json"] {
            let mut capture = session(&directory);
            capture.reserve_output(Path::new(name)).unwrap();
            assert!(preflight_destination(&capture, Path::new(name)).is_err());
            assert_eq!(capture.finish().captured_original_generations(), 0);
        }
        assert_eq!(std::fs::read(directory.path().join("file.json")).unwrap(), b"sentinel");
        assert!(directory.path().join("directory.json").is_dir());
    }

    /// A missing leaf admits preflight only; a later creator must still cause refusal.
    #[test]
    fn missing_destination_preflight_does_not_claim_atomic_absence() {
        let directory = tempfile::tempdir().unwrap();
        let mut capture = session(&directory);
        capture.reserve_output(Path::new("output.json")).unwrap();
        preflight_destination(&capture, Path::new("output.json")).unwrap();
        std::fs::write(directory.path().join("output.json"), b"later creator").unwrap();
        assert!(preflight_destination(&capture, Path::new("output.json")).is_err());
        assert_eq!(capture.finish().captured_original_generations(), 0);
        assert_eq!(std::fs::read(directory.path().join("output.json")).unwrap(), b"later creator");
    }

    /// Both existing-target and dangling Unix links are leaves, never absent destination authority.
    #[cfg(unix)]
    #[test]
    fn destination_symlinks_are_refused_without_target_reads() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("target.json"), b"target sentinel").unwrap();
        for (link, target) in [("present.json", "target.json"), ("dangling.json", "absent.json")] {
            std::os::unix::fs::symlink(target, directory.path().join(link)).unwrap();
            let mut capture = session(&directory);
            capture.reserve_output(Path::new(link)).unwrap();
            assert!(preflight_destination(&capture, Path::new(link)).is_err());
            assert_eq!(capture.finish().captured_original_generations(), 0);
            assert!(
                std::fs::symlink_metadata(directory.path().join(link))
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
        assert_eq!(
            std::fs::read(directory.path().join("target.json")).unwrap(),
            b"target sentinel"
        );
        assert!(!directory.path().join("absent.json").exists());
    }

    /// Output reservation refuses input aliases before any observation or source read occurs.
    #[test]
    fn output_namespace_is_reserved_before_any_original() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("source.json"), b"actual original").unwrap();
        let mut unreserved = session(&directory);
        let original = unreserved
            .required(
                Path::new("source.json"),
                CaptureRole::RiskScaffoldDeclaration,
                manifest::MAX_MANIFEST_BYTES,
            )
            .unwrap();
        assert_eq!(original.bytes(), b"actual original");
        assert_eq!(unreserved.finish().captured_original_generations(), 1);
        drop(original);
        let mut capture = session(&directory);
        capture.reserve_output(Path::new("SOURCE.json")).unwrap();
        assert!(
            capture
                .required(
                    Path::new("source.json"),
                    CaptureRole::RiskScaffoldDeclaration,
                    manifest::MAX_MANIFEST_BYTES
                )
                .is_err()
        );
        assert_eq!(capture.finish().captured_original_generations(), 0);
    }
}

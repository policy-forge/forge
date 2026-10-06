//! Connector-neutral local handoff from an actual captured supported native POA&M.
//!
//! Selection and intent are explicit. No remote target, credential, plan, apply or
//! ticket state is admitted here. Declared state is not state-at-as-of or approval.
//! Native/source consistency does not establish remediation or actor authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::workflow::{PreparedWorkflow, State, WorkflowManifest};
use super::{identity, manifest, workflow, workflow_model};
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};

/// Closed outbound format; this proposal does not establish the complete PRD065 contract.
pub const CHANGE_SET_VERSION: &str = "forge.integration-change-set/1";
/// Separate closed explicit item/intent selection format, not remote configuration.
pub const SELECTION_VERSION: &str = "forge.poam-outbound-selection/1";
/// Complete native original and local output ceiling, independent of item counts.
const MAX_NATIVE_BYTES: usize = 10 * 1024 * 1024;
/// Fixed qualified boundary; IDs and hashes can still be sensitive metadata.
const BOUNDARY: &str = "Local outbound intent only. IDs and metadata can be sensitive. Declared state is not state-at-as-of. Source consistency is not actor, evidence, remediation or closure approval. No remote target, dry-run, apply, credentials or mutation is authorized.";

/// Cached existing-dependency offline validator for the proposed closed local handoff.
static CHANGE_SET_VALIDATOR: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

/// Desired external operation, always explicitly selected and never inferred from status.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Intent {
    /// Ask a future separately governed adapter to plan creation, not perform it.
    Create,
    /// Ask a future separately governed adapter to plan an update with remote preconditions.
    Update,
    /// Request a future close review without asserting remote or native closure authority.
    CloseRequest,
}

impl Intent {
    /// Return fixed domain-separated operation spelling for stable identity.
    const fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::CloseRequest => "close-request",
        }
    }

    /// Minimize rationale to a fixed intent code rather than authored remediation prose.
    const fn reason(self) -> &'static str {
        match self {
            Self::Create => "explicit-create-request",
            Self::Update => "explicit-update-request",
            Self::CloseRequest => "explicit-close-request",
        }
    }
}

/// One caller-selected stable item and exact desired operation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionEntry {
    /// Exact authored stable item key; no title matching, trimming or implicit selection.
    pub item_key: String,
    /// Explicit intent independent of the item's final declared state.
    pub intent: Intent,
}

/// Complete closed selection; no target URL, connector, credential or excerpt field exists.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Exact selection schema identifier.
    pub schema_version: String,
    /// Complete explicit operations, with one contradictory-or-identical duplicate refused per item.
    pub operations: Vec<SelectionEntry>,
}

/// Actual native source identity and offline schema binding, not generated-byte attestation.
#[derive(Debug, Serialize)]
pub struct NativeSourcePin {
    /// Exact supported OSCAL native root.
    pub artifact_type: &'static str,
    /// Root UUID read from the held original and matched to the complete fresh projection.
    pub root_uuid: String,
    /// SHA256 of original native file bytes, preserving whitespace and final newlines.
    pub raw_sha256: String,
    /// Complete original byte length, not a typed serialization length.
    pub raw_bytes: usize,
    /// Actual native metadata OSCAL release, equal to the offline supported projection.
    pub oscal_version: String,
    /// Name of the existing vendored native schema, not a URL to fetch.
    pub schema_file: &'static str,
    /// Exact compiled-in existing schema bytes hashed independently of source payload.
    pub schema_sha256: String,
}

/// Minimized responsibility IDs; these can still identify people and carry no authentication.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct OwnerIds {
    /// Exact declared role identifier, without its title.
    pub role_id: String,
    /// Exact declared party key, without name, contact data or rationale.
    pub party_key: String,
}

/// Finite minimized candidate fields; original source/title/rationale/evidence prose is absent.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fields {
    /// Generated generic title using only stable native item identity.
    pub title: String,
    /// Fixed explicit intent code, never an authored free-text rationale.
    pub reason_codes: Vec<String>,
    /// Sorted declared owner IDs, not remotely resolved assignments.
    pub owners: Vec<OwnerIds>,
    /// Exact authored full-date target, not a forecast or hidden clock.
    pub target_date: String,
    /// Final declared workflow state, explicitly not effective state at the preparation date.
    pub declared_state: State,
}

/// One deterministic outbound request with payload and source generations separate from identity.
#[derive(Debug, Serialize)]
pub struct Operation {
    /// Stable plan/item/intent key; mutable fields and source generations do not enter it.
    pub operation_key: String,
    /// Stable native UUID of the explicitly selected item.
    pub source_item_uuid: String,
    /// Exact selected stable key; this metadata may be sensitive.
    pub item_key: String,
    /// Explicit create/update/close-request selection, not a remote executed outcome.
    pub desired_operation: Intent,
    /// Closed allowed field names, leaving connector-specific translation to future governed plans.
    pub allowed_fields: Vec<&'static str>,
    /// Complete minimized typed metadata.
    pub fields: Fields,
    /// Exact deterministic typed payload digest; not a stable operation identity.
    pub payload_sha256: String,
    /// Fixed null: no remote object/version has been observed or authorized by this local export.
    pub remote_preconditions: Option<()>,
}

/// Complete bounded handoff; it is neither a remote plan nor an apply receipt.
#[derive(Debug, Serialize)]
pub struct ChangeSet {
    /// Exact closed change-set version.
    pub schema_version: &'static str,
    /// One original native source pin, qualified against an actual current five-source capture.
    pub source: NativeSourcePin,
    /// Original authoring file SHA256, independent of native or selected payload digests.
    pub authoring_manifest_sha256: String,
    /// Original selection byte SHA256; item ordering does not change stable operation identity.
    pub selection_sha256: String,
    /// Explicit four-digit full date used by actual workflow preparation, not final-state dating.
    pub preparation_as_of: String,
    /// Explicit bounded schedule interval used by preparation, never silently defaulted.
    pub preparation_due_soon_days: u16,
    /// Complete selected operation denominator before output serialization.
    pub operation_count: usize,
    /// Complete sorted operations; no truncated prefix returns successfully.
    pub operations: Vec<Operation>,
    /// Fixed false: no dry-run, target allowlist, confirmation or mutation is admitted here.
    pub remote_apply_authorized: bool,
    /// Truthful sensitivity, temporal and authority limits.
    pub boundary: &'static str,
}

/// Actual confined original file; identity cannot be fabricated through a public constructor.
struct Original {
    /// Confined path retained only privately for generation recheck and output preflight.
    relative: PathBuf,
    /// Exact original bytes, never reconstructed from parsed Value.
    bytes: Vec<u8>,
    /// Real platform file identity returned by the existing confined reader.
    identity: (u64, u64),
    /// Original complete read ceiling reused at generation recheck.
    limit: u64,
}

/// Sealed real capture reusable by sibling readers through a Root-owned shared extraction.
pub(super) struct CapturedNativeWorkflow {
    /// Resolved original manifest directory; never serialized as an outbound field.
    root: PathBuf,
    /// Real authoring file original, separately retained from typed declarations.
    authoring: Original,
    /// Real native file original; this is mandatory for outbound source authority.
    native: Original,
    /// Parsed current declaration originating only from the held authoring bytes.
    declaration: WorkflowManifest,
    /// One actual prepared source closure with all five originals and original artifact guards.
    prepared: PreparedWorkflow,
    /// Explicit preparation date, never generated by a clock.
    as_of: String,
    /// Explicit preparation interval, checked by existing workflow admission.
    due_soon_days: u16,
}

impl CapturedNativeWorkflow {
    /// Recheck authoring/native literal bytes and identities plus the actual held five-source closure.
    /// # Errors
    /// Refuses any changed, replaced, missing or unsafe original without exposing private paths.
    fn verify_inputs(&self) -> Result<(), ForgeError> {
        verify_original(&self.root, &self.authoring)?;
        verify_original(&self.root, &self.native)?;
        self.prepared
            .verify_inputs()
            .map_err(|_| error("current captured source generation changed"))
    }
}

/// Held complete local output and its original actual source generations, without a publisher.
pub struct PreparedOutbound {
    /// Sealed real source capture retained until the caller finishes local output.
    captured: CapturedNativeWorkflow,
    /// Complete bounded minimized JSON, with no partial-prefix authority.
    bytes: Vec<u8>,
}

/// Preserve the sealed capture boundary during ordinary diagnostic formatting.
impl std::fmt::Debug for PreparedOutbound {
    /// Show only the complete report byte count, without private originals, paths or fields.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedOutbound")
            .field("report_bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

impl PreparedOutbound {
    /// Borrow full actual authoring/native and retained five-source paths for sibling output preflight.
    pub(super) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        [
            self.captured.root.join(&self.captured.authoring.relative),
            self.captured.root.join(&self.captured.native.relative),
        ]
        .into_iter()
        .chain(self.captured.prepared.input_paths())
    }

    /// Refuse selection identities that alias any actual held authoring/native or source original.
    /// # Errors
    /// Returns a fixed refusal for captured input aliases, without exposing private paths.
    pub(super) fn reject_input_aliases(&self, identities: &[(u64, u64)]) -> Result<(), ForgeError> {
        if identities.iter().any(|identity| {
            *identity == self.captured.authoring.identity
                || *identity == self.captured.native.identity
        }) {
            return Err(error("outbound selection aliases an actual captured input"));
        }
        self.captured.prepared.reject_input_aliases(identities)
    }

    /// Borrow the complete local handoff; bytes confer no remote plan/apply authority.
    #[must_use]
    pub fn report(&self) -> &[u8] {
        &self.bytes
    }

    /// Recheck all seven actual source originals before local publication.
    ///
    /// Root must separately retain/recheck the original selection file and enforce
    /// output confinement, complete collision preflight and no-replace publication.
    /// # Errors
    /// Rejects original file generation drift; it does not prevent later external edits.
    pub fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.captured.verify_inputs()
    }
}

/// Prepare explicit local outbound intent from actual original authoring/native files.
///
/// Native input must reside beside the manifest and match the complete freshly
/// generated supported Value. Unknown extensions are refused; no reverse import,
/// remote target, excerpt opt-in, credential or status-to-close inference occurs.
/// # Errors
/// Rejects bad/duplicate/unknown selections, unsafe or stale originals, unsupported
/// workflow/native/schema data, unavailable source closure or complete output overflow.
pub fn prepare(
    manifest_path: &Path,
    native_relative: &Path,
    selection_bytes: &[u8],
    as_of: &str,
    due_soon_days: u16,
) -> Result<PreparedOutbound, ForgeError> {
    let selection = parse_selection(selection_bytes)?;
    let captured = capture(manifest_path, native_relative, as_of, due_soon_days)?;
    prepare_captured(captured, &selection, selection_bytes)
}

/// Capture actual native and authoring originals once and retain one real five-source preparation.
///
/// Root may extract this sealed seam for portfolio/outbound reuse; no detached
/// Value/hash-only constructor or duplicate second source preparation is provided.
/// # Errors
/// Rejects unsafe/aliased originals, invalid dates, current source/schema mismatch or native extras.
pub(super) fn capture(
    manifest_path: &Path,
    native_relative: &Path,
    as_of: &str,
    due_soon_days: u16,
) -> Result<CapturedNativeWorkflow, ForgeError> {
    date(as_of)?;
    native_filename(native_relative)?;
    let parent =
        manifest_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let root = parent.canonicalize().map_err(|_| error("authoring directory is unavailable"))?;
    let relative = PathBuf::from(
        manifest_path.file_name().ok_or_else(|| error("authoring filename is required"))?,
    );
    let authoring = capture_original(&root, &relative, manifest::MAX_MANIFEST_BYTES)?;
    let declaration = workflow::parse(&authoring.bytes)
        .map_err(|_| error("authoring workflow is unsupported or invalid"))?;
    let native = capture_original(
        &root,
        native_relative,
        u64::try_from(MAX_NATIVE_BYTES).map_err(|_| error("native bound conversion failed"))?,
    )?;
    if authoring.identity == native.identity {
        return Err(error("native and authoring originals alias"));
    }
    let prepared =
        workflow::prepare(&root.join(&relative), &authoring.bytes, as_of, due_soon_days, None)
            .map_err(|_| error("actual current source or workflow preparation is invalid"))?;
    prepared.reject_input_aliases(&[authoring.identity, native.identity])?;
    match_native(prepared.artifact(), &native.bytes)?;
    workflow_model::validate_native(&native.bytes)?;
    let captured = CapturedNativeWorkflow {
        root,
        authoring,
        native,
        declaration,
        prepared,
        as_of: as_of.into(),
        due_soon_days,
    };
    captured.verify_inputs()?;
    Ok(captured)
}

/// Compose complete minimized intent only from the sealed held actual source, without additional capture.
fn prepare_captured(
    captured: CapturedNativeWorkflow,
    selection: &Selection,
    selection_bytes: &[u8],
) -> Result<PreparedOutbound, ForgeError> {
    let operations = operations(&captured.declaration, selection)?;
    let report = ChangeSet {
        schema_version: CHANGE_SET_VERSION,
        source: source_pin(&captured)?,
        authoring_manifest_sha256: sha256_hex(&captured.authoring.bytes),
        selection_sha256: sha256_hex(selection_bytes),
        preparation_as_of: captured.as_of.clone(),
        preparation_due_soon_days: captured.due_soon_days,
        operation_count: operations.len(),
        operations,
        remote_apply_authorized: false,
        boundary: BOUNDARY,
    };
    let bytes = workflow_model::json_line(&report)?;
    validate_change_set(&bytes)?;
    captured.verify_inputs()?;
    Ok(PreparedOutbound { captured, bytes })
}

/// Strictly parse every original selection byte under existing authoring raw/string/depth bounds.
/// # Errors
/// Refuses duplicate/unknown fields, unsupported schema, empty or unbounded complete operations.
pub fn parse_selection(bytes: &[u8]) -> Result<Selection, ForgeError> {
    if u64::try_from(bytes.len()).map_err(|_| error("selection length conversion failed"))?
        > manifest::MAX_MANIFEST_BYTES
    {
        return Err(error("selection exceeds its complete raw bound"));
    }
    let value = json_strict::parse_value(
        bytes,
        "POA&M outbound selection",
        Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|_| error("selection JSON is malformed, duplicate or unbounded"))?;
    let selection: Selection = serde_json::from_value(value)
        .map_err(|_| error("selection is not the closed item-intent contract"))?;
    if selection.schema_version != SELECTION_VERSION
        || selection.operations.is_empty()
        || selection.operations.len() > manifest::MAX_ITEMS
    {
        return Err(error("selection schema or complete operation count is invalid"));
    }
    let mut keys = BTreeSet::new();
    for entry in &selection.operations {
        if entry.item_key.is_empty()
            || entry.item_key.len() > 256
            || entry.item_key.trim() != entry.item_key
            || entry.item_key.chars().any(char::is_control)
        {
            return Err(error("selected item key is not exact bounded stable identity text"));
        }
        if !keys.insert(&entry.item_key) {
            return Err(error("duplicate or contradictory item operations are refused"));
        }
    }
    Ok(selection)
}

/// Project only selected actual declared item identities; no status manufactures remote intent.
fn operations(
    manifest: &WorkflowManifest,
    selection: &Selection,
) -> Result<Vec<Operation>, ForgeError> {
    let items: BTreeMap<_, _> =
        manifest.items.iter().map(|item| (item.key.as_str(), item)).collect();
    let mut result = Vec::new();
    for entry in &selection.operations {
        let item = items
            .get(entry.item_key.as_str())
            .ok_or_else(|| error("selected item identity is absent"))?;
        let uuid = identity::item(&manifest.document.key, &item.key).to_string();
        let mut owners: Vec<_> = item
            .owners
            .iter()
            .map(|owner| OwnerIds {
                role_id: owner.role_id.clone(),
                party_key: owner.party_key.clone(),
            })
            .collect();
        owners.sort();
        let fields = Fields {
            title: format!("FORGE remediation item {uuid}"),
            reason_codes: vec![entry.intent.reason().into()],
            owners,
            target_date: item.target_date.clone(),
            declared_state: item.state,
        };
        let payload = serde_json::to_vec(&fields)
            .map_err(|_| error("typed outbound metadata cannot serialize"))?;
        result.push(Operation {
            operation_key: operation_key(&manifest.document.key, &item.key, entry.intent),
            source_item_uuid: uuid,
            item_key: item.key.clone(),
            desired_operation: entry.intent,
            allowed_fields: vec![
                "title",
                "reason_codes",
                "owners",
                "target_date",
                "declared_state",
            ],
            fields,
            payload_sha256: sha256_hex(&payload),
            remote_preconditions: None,
        });
    }
    result.sort_by(|a, b| a.item_key.cmp(&b.item_key));
    Ok(result)
}

/// Keep identity independent of mutable payload and source generations, with fixed UUID domains.
fn operation_key(plan: &str, item: &str, intent: Intent) -> String {
    format!("poam:{}:{}:{}", identity::document(plan), identity::item(plan, item), intent.label())
}

/// Derive identity/hash from actual original native bytes after exact full supported matching.
fn source_pin(captured: &CapturedNativeWorkflow) -> Result<NativeSourcePin, ForgeError> {
    let native = native_value(&captured.native.bytes)?;
    let root = native
        .get("plan-of-action-and-milestones")
        .ok_or_else(|| error("native root is unavailable"))?;
    let uuid = root
        .get("uuid")
        .and_then(Value::as_str)
        .ok_or_else(|| error("native root identity is unavailable"))?;
    let version = root
        .pointer("/metadata/oscal-version")
        .and_then(Value::as_str)
        .ok_or_else(|| error("native schema release is unavailable"))?;
    if uuid != identity::document(&captured.declaration.document.key).to_string() {
        return Err(error("native root identity differs from its actual current declaration"));
    }
    Ok(NativeSourcePin {
        artifact_type: "plan-of-action-and-milestones",
        root_uuid: uuid.into(),
        raw_sha256: sha256_hex(&captured.native.bytes),
        raw_bytes: captured.native.bytes.len(),
        oscal_version: version.into(),
        schema_file: "oscal_poam_schema.json",
        schema_sha256: sha256_hex(include_bytes!("../../schemas/oscal_poam_schema.json")),
    })
}

/// Read one real original through existing confined native file/identity checks.
fn capture_original(root: &Path, relative: &Path, limit: u64) -> Result<Original, ForgeError> {
    let (bytes, identity) = crate::linkage::read_confined_local_file(root, relative, limit)
        .map_err(|_| error("actual outbound original is missing, unsafe or unbounded"))?;
    Ok(Original { relative: relative.into(), bytes, identity, limit })
}

/// Reopen actual original bytes and identity; equal parsed JSON or hash-only records are insufficient.
fn verify_original(root: &Path, original: &Original) -> Result<(), ForgeError> {
    let (bytes, identity) =
        crate::linkage::read_confined_local_file(root, &original.relative, original.limit)
            .map_err(|_| error("actual outbound original is missing or unsafe at recheck"))?;
    if identity != original.identity || bytes != original.bytes {
        return Err(error("actual outbound original generation changed"));
    }
    Ok(())
}

/// Require one explicit JSON filename in the authoring directory, never an arbitrary root or URL.
fn native_filename(path: &Path) -> Result<(), ForgeError> {
    if !matches!(path.components().next(), Some(Component::Normal(_)))
        || path.components().count() != 1
        || path.extension().and_then(|s| s.to_str()) != Some("json")
        || path.to_str().is_none_or(|s| s.contains('\\') || s.contains(':'))
    {
        return Err(error(
            "native source must be one local JSON filename beside its authoring manifest",
        ));
    }
    Ok(())
}

/// Strictly decode bounded original native JSON, retaining duplicate/trailing/extra refusal.
fn native_value(bytes: &[u8]) -> Result<Value, ForgeError> {
    if bytes.len() > MAX_NATIVE_BYTES {
        return Err(error("complete native original exceeds ten MiB"));
    }
    json_strict::parse_value(
        bytes,
        "POA&M outbound native input",
        Limits { max_depth: 64, max_string_bytes: MAX_NATIVE_BYTES },
    )
    .map_err(|_| error("native source is not complete bounded duplicate-free JSON"))
}

/// Validate the complete local output against a closed offline schema and exact operation denominator.
///
/// This validates typed output shape only; it cannot construct a held source capture or remote plan.
fn validate_change_set(bytes: &[u8]) -> Result<(), ForgeError> {
    let value = native_value(bytes)?;
    let validator = CHANGE_SET_VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../schemas/forge_integration_change_set_schema.json"
            ))
            .map_err(|_| "offline change-set schema is malformed".to_string())?;
            jsonschema::validator_for(&schema)
                .map_err(|_| "offline change-set schema cannot compile".to_string())
        })
        .as_ref()
        .map_err(|_| error("offline change-set schema is unavailable"))?;
    if !validator.is_valid(&value) {
        return Err(error("local change set is not the closed supported contract"));
    }
    let operations =
        value["operations"].as_array().ok_or_else(|| error("local operations are absent"))?;
    let count =
        u64::try_from(operations.len()).map_err(|_| error("operation count conversion failed"))?;
    if value["operation_count"].as_u64() != Some(count) {
        return Err(error("complete operation count differs"));
    }
    let mut keys = BTreeSet::new();
    let mut items = BTreeSet::new();
    for operation in operations {
        if !keys.insert(operation["operation_key"].as_str())
            || !items.insert(operation["item_key"].as_str())
        {
            return Err(error("local change set repeats an operation or item identity"));
        }
        let intent: Intent = serde_json::from_value(operation["desired_operation"].clone())
            .map_err(|_| error("local explicit intent is invalid"))?;
        let fields: Fields = serde_json::from_value(operation["fields"].clone())
            .map_err(|_| error("local minimized fields are invalid"))?;
        let payload =
            serde_json::to_vec(&fields).map_err(|_| error("local payload cannot serialize"))?;
        if operation["payload_sha256"].as_str() != Some(sha256_hex(&payload).as_str())
            || fields.reason_codes.len() != 1
            || fields.reason_codes[0] != intent.reason()
        {
            return Err(error("local explicit intent or payload hash differs"));
        }
        let item_uuid = operation["source_item_uuid"]
            .as_str()
            .ok_or_else(|| error("local item UUID is absent"))?;
        let root_uuid = value["source"]["root_uuid"]
            .as_str()
            .ok_or_else(|| error("local native root UUID is absent"))?;
        let expected = format!("poam:{root_uuid}:{item_uuid}:{}", intent.label());
        if operation["operation_key"].as_str() != Some(expected.as_str())
            || fields.title != format!("FORGE remediation item {item_uuid}")
        {
            return Err(error("local stable identity or generic title differs"));
        }
    }
    Ok(())
}

/// Require complete supported native equality; unknown extras cannot ride an otherwise matching UUID.
fn match_native(expected: &[u8], original: &[u8]) -> Result<(), ForgeError> {
    if native_value(expected)? != native_value(original)? {
        return Err(error(
            "native original differs from the complete supported current projection",
        ));
    }
    Ok(())
}

/// Preserve the existing four-digit canonical Gregorian date admission without a hidden clock.
fn date(value: &str) -> Result<(), ForgeError> {
    if value.len() != 10 {
        return Err(error("preparation date must be YYYY-MM-DD"));
    }
    let parsed = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| error("preparation date is not a valid calendar date"))?;
    if parsed.to_string() != value {
        return Err(error("preparation date must use canonical YYYY-MM-DD"));
    }
    Ok(())
}

/// Fixed typed local refusal, without private filenames, content or source exception messages.
fn error(message: &'static str) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    //! Synthetic projection/parser controls do not fabricate validated source/native captures.
    use super::*;
    use serde_json::json;

    /// Disclosed syntactic-only workflow fixture; no native/source capture authority is constructed.
    fn declared() -> WorkflowManifest {
        let pin = |name: &str| {
            json!({"artifact":format!("{name}.json"),"href":format!("{name}.json"),"expected_sha256":"a".repeat(64),
            "root_uuid":"11111111-1111-4111-8111-111111111111","document_version":"1","oscal_version":"1.2.3"})
        };
        let owner = json!({"role_id":"owner","party_key":"person","rationale":"SENSITIVE ownership rationale"});
        let initial = json!({"key":"initial","actor":{"role_id":"owner","party_key":"person"},"at":"2026-01-01T00:00:00Z",
            "from":null,"to":"planned","rationale":"SENSITIVE status rationale","closure":null});
        let value = json!({"schema_version":"forge.poam/1","document":{"key":"plan","title":"SENSITIVE title","version":"1","last_modified":"2026-02-01T00:00:00Z"},
            "source":{"assessment_results":pin("ar"),"result":{"uuid":"11111111-1111-4111-8111-111111111111","key":"result"},
                "context":{"assessment_plan":pin("ap"),"ssp":pin("ssp"),"profile":pin("profile"),"catalog":pin("catalog")}},
            "roles":[{"id":"owner","title":"SENSITIVE role title"}],"parties":[{"key":"person","type":"person","name":"SENSITIVE name"}],
            "items":[{"key":"work","title":"SENSITIVE work title","description":"SENSITIVE source prose","owners":[owner],"target_date":"2026-02-01","state":"planned","history":[initial],
                "source_refs":[{"kind":"finding","key":"finding","uuid":"22222222-2222-4222-8222-222222222222","result_uuid":"11111111-1111-4111-8111-111111111111","expected_sha256":"b".repeat(64)}],
                "milestones":[{"key":"step","outcome":"SENSITIVE outcome","target_date":"2026-01-20","depends_on":[],"owners":[owner],"state":"planned","history":[initial]}]}]});
        workflow::parse(&serde_json::to_vec(&value).unwrap()).unwrap()
    }

    /// Encode a closed explicit selection, never a fabricated prepared native capture.
    fn selection(rows: Value) -> Selection {
        let value = Value::Object(serde_json::Map::from_iter([
            ("schema_version".into(), json!(SELECTION_VERSION)),
            ("operations".into(), rows),
        ]));
        parse_selection(&serde_json::to_vec(&value).unwrap()).unwrap()
    }

    /// Unknown/duplicate fields and contradictory identical-key intents fail before native capture.
    #[test]
    fn closed_explicit_selection_refuses_unknown_duplicate_and_contradictory_operations() {
        assert!(parse_selection(br#"{"schema_version":"forge.poam-outbound-selection/1","operations":[],"credential":"secret"}"#).is_err());
        assert!(
            parse_selection(br#"{"schema_version":"x","schema_version":"y","operations":[]}"#)
                .is_err()
        );
        for intents in [["create", "create"], ["create", "update"], ["create", "close-request"]] {
            let raw = json!({"schema_version":SELECTION_VERSION,"operations":[{"item_key":"work","intent":intents[0]},{"item_key":"work","intent":intents[1]}]});
            assert!(parse_selection(&serde_json::to_vec(&raw).unwrap()).is_err());
        }
        let unsupported = json!({"schema_version":SELECTION_VERSION,"operations":[{"item_key":"work","intent":"close"}]});
        assert!(parse_selection(&serde_json::to_vec(&unsupported).unwrap()).is_err());
        for key in [String::new(), " work".into(), "wo\nrk".into(), "x".repeat(257)] {
            let raw = json!({"schema_version":SELECTION_VERSION,"operations":[{"item_key":key,"intent":"create"}]});
            assert!(parse_selection(&serde_json::to_vec(&raw).unwrap()).is_err());
        }
        assert!(
            parse_selection(
                br#"{"schema_version":"forge.poam-outbound-selection/1","operations":[]}"#
            )
            .is_err()
        );
    }

    /// Projection removes authored prose while retaining explicit minimized, still-sensitive IDs/date.
    #[test]
    fn selected_fields_are_minimized_without_claiming_anonymity_or_remote_authority() {
        let rows =
            operations(&declared(), &selection(json!([{"item_key":"work","intent":"create"}])))
                .unwrap();
        let bytes = serde_json::to_vec(&rows).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("SENSITIVE"));
        assert!(!text.contains("expected_sha256"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].fields.owners[0].party_key, "person");
        assert_eq!(rows[0].fields.target_date, "2026-02-01");
        assert_eq!(rows[0].fields.declared_state, State::Planned);
        assert!(rows[0].remote_preconditions.is_none());
    }

    /// Mutable target/source/prose edits change generations or payloads but not stable operation identity.
    #[test]
    fn stable_operation_identity_is_distinct_from_payload_and_source_generation_hashes() {
        let original = declared();
        let request = selection(json!([{"item_key":"work","intent":"update"}]));
        let a = operations(&original, &request).unwrap();
        let mut changed = original.clone();
        changed.items[0].target_date = "2026-03-01".into();
        changed.items[0].description = "different private prose".into();
        let b = operations(&changed, &request).unwrap();
        assert_eq!(a[0].operation_key, b[0].operation_key);
        assert_ne!(a[0].payload_sha256, b[0].payload_sha256);
        assert_ne!(
            operation_key("plan", "work", Intent::Create),
            operation_key("plan", "work", Intent::Update)
        );
        assert_ne!(
            operation_key("plan", "work", Intent::Update),
            operation_key("other", "work", Intent::Update)
        );
    }

    /// Cancellation never manufactures close intent; this projection-only fixture is not admitted capture authority.
    #[test]
    fn cancelled_declared_state_does_not_infer_close_request() {
        let mut declaration = declared();
        declaration.items[0].state = State::Cancelled;
        let create =
            operations(&declaration, &selection(json!([{"item_key":"work","intent":"create"}])))
                .unwrap();
        let close = operations(
            &declaration,
            &selection(json!([{"item_key":"work","intent":"close-request"}])),
        )
        .unwrap();
        assert_eq!(create[0].desired_operation, Intent::Create);
        assert_eq!(close[0].desired_operation, Intent::CloseRequest);
        assert_eq!(create[0].fields.declared_state, State::Cancelled);
        assert_ne!(create[0].operation_key, close[0].operation_key);
    }

    /// Unknown keys never select by title/prose or receive a hidden empty export.
    #[test]
    fn unknown_selected_identity_is_refused_without_fallback() {
        assert!(
            operations(&declared(), &selection(json!([{"item_key":"absent","intent":"create"}])))
                .is_err()
        );
    }

    /// Native comparison is whole-value strict while preserving literal-byte hashes separately.
    #[test]
    fn native_matching_refuses_extras_duplicates_and_trailing_values() {
        let a = br#"{"root":{"uuid":"exact","value":"a\n\tb"}}"#;
        let same = br#"{ "root": { "value": "a\n\tb", "uuid": "exact" } }"#;
        assert!(match_native(a, same).is_ok());
        assert_ne!(sha256_hex(a), sha256_hex(same));
        assert!(
            match_native(a, br#"{"root":{"uuid":"exact","value":"a\n\tb","private":"extra"}}"#)
                .is_err()
        );
        assert!(native_value(br#"{"root":1,"root":1}"#).is_err());
        assert!(native_value(br#"{"root":1} {}"#).is_err());
    }

    /// Explicit canonical dates and local filename bounds retain the original guarded surface.
    #[test]
    fn dates_and_native_filenames_fail_closed_without_clock_or_external_roots() {
        for bad in ["-0001-01-01", "+10000-12-31", "2026-02-29", "2026-2-01"] {
            assert!(date(bad).is_err());
        }
        assert!(date("2024-02-29").is_ok());
        for bad in [
            "../native.json",
            "nested/native.json",
            "/native.json",
            "native.xml",
            "native\\other.json",
        ] {
            assert!(native_filename(Path::new(bad)).is_err());
        }
    }

    /// Syntactic output fixture proves only closed metadata shape, never real native/source authority.
    fn local_output_fixture() -> Value {
        let declaration = declared();
        let rows = operations(
            &declaration,
            &selection(json!([{
            "item_key":"work","intent":"create"}])),
        )
        .unwrap();
        json!({"schema_version":CHANGE_SET_VERSION,
            "source":{"artifact_type":"plan-of-action-and-milestones",
                "root_uuid":identity::document(&declaration.document.key).to_string(),
                "raw_sha256":"a".repeat(64),"raw_bytes":1,"oscal_version":"1.2.3",
                "schema_file":"oscal_poam_schema.json","schema_sha256":"b".repeat(64)},
            "authoring_manifest_sha256":"c".repeat(64),"selection_sha256":"d".repeat(64),
            "preparation_as_of":"2026-02-01","preparation_due_soon_days":7,
            "operation_count":1,"operations":rows,"remote_apply_authorized":false,"boundary":BOUNDARY})
    }

    /// The actual consumed validator refuses open fields, false counts, authority, intent and payload drift.
    #[test]
    fn complete_closed_output_validation_refuses_shape_and_semantic_drift() {
        let valid = local_output_fixture();
        assert!(validate_change_set(&serde_json::to_vec(&valid).unwrap()).is_ok());
        for mutation in 0..7 {
            let mut changed = valid.clone();
            match mutation {
                0 => changed["private-extra"] = json!("sensitive"),
                1 => changed["operation_count"] = json!(2),
                2 => changed["remote_apply_authorized"] = json!(true),
                3 => changed["operations"][0]["fields"]["rationale"] = json!("private prose"),
                4 => changed["operations"][0]["fields"]["target_date"] = json!("2026-03-01"),
                5 => changed["operations"][0]["desired_operation"] = json!("update"),
                _ => {
                    changed["operations"][0]["remote_preconditions"] =
                        json!({"remote":"unobserved"});
                }
            }
            assert!(validate_change_set(&serde_json::to_vec(&changed).unwrap()).is_err());
        }
        let mut repeated = valid.clone();
        repeated["operations"].as_array_mut().unwrap().push(valid["operations"][0].clone());
        repeated["operation_count"] = json!(2);
        assert!(validate_change_set(&serde_json::to_vec(&repeated).unwrap()).is_err());
    }

    /// Complete raw/report caps cannot return a truncated successful prefix.
    #[test]
    fn complete_raw_and_output_limits_refuse_prefix_success() {
        assert!(
            parse_selection(&vec![
                b' ';
                usize::try_from(manifest::MAX_MANIFEST_BYTES).unwrap() + 1
            ])
            .is_err()
        );
        assert!(native_value(&vec![b' '; MAX_NATIVE_BYTES + 1]).is_err());
        assert!(workflow_model::json_line(&"x".repeat(workflow::MAX_OUTPUT_BYTES)).is_err());
    }

    /// Build unique minimized owners and recompute the actual typed compact payload digest.
    /// This syntactic fixture has no captured native, declared-party or remote authority.
    fn local_output_with_owner_count(count: usize) -> Value {
        let mut value = local_output_fixture();
        let mut fields: Fields =
            serde_json::from_value(value["operations"][0]["fields"].clone()).unwrap();
        fields.owners = (0..count)
            .map(|ordinal| OwnerIds {
                role_id: "owner".into(),
                party_key: format!("person-{ordinal:02}"),
            })
            .collect();
        let payload = serde_json::to_vec(&fields).unwrap();
        value["operations"][0]["fields"] = serde_json::to_value(&fields).unwrap();
        value["operations"][0]["payload_sha256"] = json!(sha256_hex(&payload));
        value
    }

    /// The actual consumed closed validator accepts exactly 64 unique, correctly hashed owners.
    #[test]
    fn consumed_output_validator_accepts_sixty_four_unique_owners() {
        let value = local_output_with_owner_count(64);
        assert_eq!(value["operations"][0]["fields"]["owners"].as_array().unwrap().len(), 64);
        assert!(validate_change_set(&serde_json::to_vec(&value).unwrap()).is_ok());
    }

    /// A sixty-fifth unique owner is refused with a fresh payload digest and unchanged valid fields.
    #[test]
    fn consumed_output_validator_refuses_sixty_five_unique_owners() {
        let value = local_output_with_owner_count(65);
        assert_eq!(value["operations"][0]["fields"]["owners"].as_array().unwrap().len(), 65);
        let fields: Fields =
            serde_json::from_value(value["operations"][0]["fields"].clone()).unwrap();
        let payload = serde_json::to_vec(&fields).unwrap();
        assert_eq!(value["operations"][0]["payload_sha256"], json!(sha256_hex(&payload)));
        let result = validate_change_set(&serde_json::to_vec(&value).unwrap());
        assert!(matches!(result, Err(ForgeError::PoamBuild(message))
            if message == "local change set is not the closed supported contract"));
    }

    /// Make a complete syntactic declaration with unique declared parties for the owner boundary.
    /// Parsing this fixture does not construct a prepared source or native capture.
    fn declaration_bytes_with_owner_count(count: usize) -> Vec<u8> {
        let mut value = serde_json::to_value(declared()).unwrap();
        let parties = value["parties"].as_array_mut().unwrap();
        for ordinal in 0..count {
            parties.push(json!({
                "key": format!("person-{ordinal:02}"),
                "type": "person",
                "name": "Declared test party",
            }));
        }
        value["items"][0]["owners"] = json!(
            (0..count)
                .map(|ordinal| json!({
                    "role_id": "owner",
                    "party_key": format!("person-{ordinal:02}"),
                    "rationale": "Explicit syntactic responsibility binding",
                }))
                .collect::<Vec<_>>()
        );
        serde_json::to_vec(&value).unwrap()
    }

    /// Public declaration parsing retains the same 64-owner boundary without fabricated capture.
    #[test]
    fn public_workflow_parser_owner_boundary_matches_consumed_output_schema() {
        let valid = workflow::parse(&declaration_bytes_with_owner_count(64)).unwrap();
        assert_eq!(valid.items[0].owners.len(), 64);
        assert!(workflow::parse(&declaration_bytes_with_owner_count(65)).is_err());
    }
}

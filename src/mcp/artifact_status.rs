//! Captured artifact status with an explicit disclosure policy and separate wire adapter.
//!
//! This module defines no MCP revision, discovery handshake, route, writer or approval
//! mechanism. Actual originals are held by the existing confined reader. Caller policy
//! is supplied independently; neither project metadata nor recorded approval authenticates
//! an owner or grants permission to act. Every returned buffer still requires the final
//! original-generation fence immediately before transport publication.

/// Private operator-selected disclosure gate; no default caller policy is widened.
#[path = "disclosure.rs"]
pub(crate) mod disclosure;

/// Genuine same-owner held-original capture foundation.
#[path = "capture_v2.rs"]
pub(crate) mod capture_v2;
/// Exact shared-admission declaration codec for actual offline capture.
#[path = "declarations_v2.rs"]
pub(crate) mod declarations_v2;
/// Actual offline input controller; no index is built or published.
#[path = "index_build.rs"]
pub(crate) mod index_build;

/// Shipped offline schemas and actual tool admission.
#[path = "catalog.rs"]
pub(crate) mod catalog;
/// Complete captured read-only query adapters.
#[path = "queries.rs"]
pub(crate) mod queries;
/// Single-owner bounded stdio worker and platform input driver.
#[path = "stdio.rs"]
pub(crate) mod stdio;
/// Exact selected modern protocol parser and complete wire encoder.
#[path = "wire.rs"]
pub(crate) mod wire;

use std::collections::BTreeSet;
use std::io::{self, Write};
use std::path::{Component, Path};
use std::rc::Rc;

#[path = "native_applicability_v2.rs"]
mod native_applicability_v2;
/// Actual complete lifecycle/native-source closure over the held /2 owner.
#[path = "native_closure_v2.rs"]
mod native_closure_v2;
/// Genuine purpose-qualified native/offline gate; no raw/boolean proof conversion.
#[path = "native_domain_v2.rs"]
mod native_domain_v2;
/// Genuine maintained complete Catalog inventory; Profile domain remains unavailable.
#[path = "native_inventory_v2.rs"]
mod native_inventory_v2;
/// Exact complete native requirement tuples with actual located pointers.
#[path = "native_requirements_v2.rs"]
mod native_requirements_v2;
#[path = "native_sources_v2.rs"]
pub(crate) mod native_sources_v2;
/// Same-original operand/admitted-data work for genuine /2 native consumers.
#[path = "native_work_v2.rs"]
mod native_work_v2;
/// Pure borrowed native requirement traversal; no capture/approval constructor.
#[path = "requirement_walk.rs"]
pub(crate) mod requirement_walk;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::OscalModelType;
use crate::hashing::sha256_hex;
use crate::lifecycle::record::{self, FingerprintSet, LifecycleState, NamedHash};
use crate::lifecycle::status::{self, CurrentArtifacts};
use crate::linkage::fresh::{self, CapturedLocal, RootGeneration};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};

/// Fixed discovery filename; no arbitrary file becomes discovery authority.
const MANIFEST_PATH: &str = "forge.mcp.json";
/// Historical engineering draft identity, independent of any MCP protocol revision.
const MANIFEST_PROFILE: &str = "forge.mcp-project/1";
/// Maximum complete declared resources before inserting indexes or reading resources.
const MAX_RESOURCES: usize = 1000;
/// Manifest plus all declared resources form at most this many original observations.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
const MAX_ORIGINALS: usize = MAX_RESOURCES + 1;
/// Maximum actual retained discovery bytes, also counted inside the complete raw budget.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
/// Maximum actual retained bytes for one ordinary declared resource.
const MAX_RESOURCE_BYTES: usize = 10 * 1024 * 1024;
/// Complete retained original-byte ceiling, not a literal heap or parser-work ceiling.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
const MAX_RAW_BYTES: usize = 50 * 1024 * 1024;
/// Existing strict parser's per-string ceiling; derived identities have narrower limits.
const MAX_STRING_BYTES: usize = 64 * 1024;
/// Maximum complete retained typed-core response; transport framing must be charged too.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
const MAX_RESPONSE_BYTES: usize = 256 * 1024;
/// Maximum byte length of a caller-approved exact project-relative path.
const MAX_PATH_BYTES: usize = 512;
/// Maximum byte length of an exact native document version without truncation.
const MAX_VERSION_BYTES: usize = 4096;
/// Qualification repeated in available results without echoing record authors or paths.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
const TRUST_BOUNDARY: &str = "Caller-selected recorded approval; identities are not authenticated and this result grants no authority to act.";

/// Existing role spellings retain the workspace admission contract.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Role {
    /// Local policy source, exposed only through its complete approved lifecycle closure.
    PolicySource,
    /// Official native Catalog JSON under its exact captured native identity.
    OscalCatalogArtifact,
    /// Official native Component Definition JSON under its captured identity.
    OscalComponentArtifact,
    /// Native mapping JSON under its captured identity.
    MappingCollection,
    /// Retained dependency; this first status slice does not admit its disclosure.
    ApplicabilityManifest,
    /// Retained dependency; freshness of computed applicability needs its own later query.
    ApplicabilityReport,
    /// Retained dependency; stored trace metadata is not current source-span proof.
    TraceReport,
    /// Intrinsically validated recorded lifecycle dependency, never an approval authority.
    LifecycleRecord,
    /// Retained source dependency without a newly inferred lifecycle or ownership role.
    LifecycleSource,
    /// Official native Profile JSON under its captured identity.
    OscalProfileArtifact,
    /// Official native SSP JSON under its captured identity.
    OscalSspArtifact,
    /// Retained dependency; full impact computation is outside this first slice.
    FrameworkImpactManifest,
    /// Retained dependency; successor records confer no new action authority.
    SuccessorMap,
    /// Retained dependency; stored impact admission is not current computed freshness.
    FrameworkImpactReport,
    /// Retained dependency; dispositions do not become automatic approval.
    FrameworkImpactDispositions,
}

impl Role {
    /// Select only existing native validators; no POA&M or source-format authority is added.
    fn native_model(self) -> Option<OscalModelType> {
        match self {
            Self::OscalCatalogArtifact => Some(OscalModelType::Catalog),
            Self::OscalComponentArtifact => Some(OscalModelType::ComponentDefinition),
            Self::MappingCollection => Some(OscalModelType::Mapping),
            Self::OscalProfileArtifact => Some(OscalModelType::Profile),
            Self::OscalSspArtifact => Some(OscalModelType::SystemSecurityPlan),
            _ => None,
        }
    }

    /// Limit actual status disclosure to this explicit first-profile implementation roster.
    fn status_supported(self) -> bool {
        self == Self::PolicySource || self.native_model().is_some()
    }
}

/// Exact caller-declared native identity, compared with actual fully validated captured JSON.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeIdentity {
    /// Existing native model spelling, never a caller-selected alternate validator.
    pub(crate) model: String,
    /// Exact root UUID spelling from admitted bytes; no caller or display normalization.
    pub(crate) root_id: String,
    /// Exact native metadata version, bounded and never clipped.
    pub(crate) document_version: String,
    /// Exact native metadata OSCAL version, bounded and never inferred.
    pub(crate) oscal_version: String,
}

/// One exact caller-selected resource; this is policy data, not an original-byte proof.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourcePolicy {
    /// Opaque safe key used for exact request selection without publishing private paths.
    pub(crate) key: String,
    /// Existing role whose intrinsic admission is separate from visibility policy.
    pub(crate) role: Role,
    /// Exact private project-relative normalized portable filename or descendant.
    pub(crate) path: String,
    /// Required raw original SHA-256, not a normalized or regenerated JSON digest.
    pub(crate) expected_sha256: String,
    /// Explicit native identity for native roles; null for other roles in this profile.
    pub(crate) native_identity: Option<NativeIdentity>,
    /// Exact declared lifecycle-resource key, or null when current approval is unestablished.
    pub(crate) lifecycle_key: Option<String>,
    /// Caller-approved inert citation token; no content, URI or private path is projected.
    pub(crate) citation_label: String,
}

/// Complete closed draft manifest interpreted only after caller equality and raw-hash checks.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectDeclaration {
    /// Exact engineering draft identifier; not the selected MCP protocol revision.
    pub(crate) schema_version: String,
    /// Caller-selected safe local project token, not an authenticated project identity.
    pub(crate) project_key: String,
    /// Complete ordered declared resource roster, bounded before indexes or captures grow.
    pub(crate) resources: Vec<ResourcePolicy>,
}

/// Caller decision about the limited recorded-approval evidence accepted for this invocation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ApprovalBasis {
    /// Default safe refusal when approved-content policy has not been established by caller.
    #[default]
    Unavailable,
    /// Explicit caller choice to use intrinsically validated recorded-current approval.
    ///
    /// This choice is not owner authentication or D067/security/product acceptance. A
    /// future adapter must obtain it from its selected policy, never infer it from files.
    RecordedCurrent,
}

/// Independent caller policy carrier; project metadata never constructs this automatically.
pub(crate) struct CallerPolicy {
    /// Expected raw hash of the fixed manifest, supplied by the actual caller policy.
    pub(crate) manifest_sha256: String,
    /// Complete expected declaration; strict captured JSON must equal every supplied value.
    pub(crate) declaration: ProjectDeclaration,
    /// Complete subset allowed to reveal even fixed availability statuses, no wildcards.
    pub(crate) visible_keys: Vec<String>,
    /// Explicit evidence basis; default unavailability exposes no artifact payload or hash.
    #[cfg_attr(not(test), allow(dead_code))]
    // Retained complete caller/raw-byte facts exercised by the core regression path.
    pub(crate) approval_basis: ApprovalBasis,
}

/// One actual bounded original observation with independently validated minimal facts.
struct CapturedObject {
    /// Caller policy retained for exact closure selection, not a detached original proof.
    policy: ResourcePolicy,
    /// Actual present original or typed missing ancestry from the confined native constructor.
    original: CapturedLocal,
    /// Actual original digest, absent when no bytes were observed.
    digest: Option<String>,
    /// Whether actual raw bytes equal the independent caller pin.
    pin_matches: bool,
    /// Actual bounded schema-valid native identity, or null on invalid/non-native bytes.
    native: Option<NativeIdentity>,
}

/// Opaque complete held project capture; no detached constructor or mutable bytes accessor.
///
/// Its `Rc` and held files deliberately stay on one worker. It is not declared `Send`;
/// transport scheduling must not move this proof into a different thread by unsafe code.
pub(crate) struct CapturedProject {
    /// Actual qualified native root and held safe original ancestry.
    root: Rc<RootGeneration>,
    /// Actual manifest original, retained and rechecked with every output.
    manifest: CapturedLocal,
    /// Complete exact resource observations, including absent and unsupported-role entries.
    objects: Vec<CapturedObject>,
    /// Independently supplied complete disclosure subset, validated before any read.
    visible_keys: Vec<String>,
    /// Explicit caller choice, never inferred from registration or lifecycle state.
    approval_basis: ApprovalBasis,
    /// Complete original-byte accounting including the manifest; not a heap measurement.
    #[cfg_attr(not(test), allow(dead_code))]
    // Retained complete caller/raw-byte facts exercised by the core regression path.
    retained_raw_bytes: usize,
}

impl CapturedProject {
    /// Recheck every original identity, full bytes or typed absence through the actual reader.
    pub(crate) fn verify_inputs(&self, control: &mut dyn WorkControl) -> WorkResult<()> {
        fence(control, Stage::CaptureResource)?;
        fresh::verify_root(&self.root).map_err(|_| capture_failure())?;
        verify_original(&self.manifest)?;
        for object in &self.objects {
            fence(control, Stage::CaptureResource)?;
            verify_original(&object.original)?;
            fence(control, Stage::CaptureResource)?;
        }
        fresh::verify_root(&self.root).map_err(|_| capture_failure())?;
        fence(control, Stage::RetainPrepared)
    }
}

/// Fixed minimized reasons; they do not echo native errors, content, paths or record authors.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Availability {
    /// Complete permitted recorded-current approval and native facts were established.
    Available,
    /// Caller has not selected a usable approved-content evidence policy.
    CurrentApprovalUnavailable,
    /// The requested caller-visible resource has actual typed missing ancestry.
    Missing,
    /// Actual captured raw bytes differ from an independent expected original pin.
    ContentChanged,
    /// The current native JSON/schema/model identity cannot be admitted.
    InvalidArtifact,
    /// A required current native model/root/version differs from the independent declaration.
    IdentityMismatch,
    /// This first slice cannot establish status for the declared role.
    UnsupportedRole,
    /// No complete caller-bound current lifecycle dependency is available.
    LifecycleUnavailable,
    /// Intrinsic recorded lifecycle semantics cannot be validated.
    InvalidLifecycle,
    /// The complete recorded source/generated dependency set is not bound to captured inputs.
    IncompleteClosure,
    /// The current recorded state does not establish the required approved state.
    NotApproved,
    /// Latest recorded approval differs from the complete current captured fingerprints.
    ApprovedDrifted,
}

/// Narrow available payload; no lifecycle party, rationale, evidence, path or policy prose.
#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
pub(crate) struct AvailableArtifact {
    /// Caller-declared existing role admitted by the current captured facts.
    role: Role,
    /// Actual raw captured digest, disclosed only for approved/current caller-visible payloads.
    content_sha256: String,
    /// Actual native identity or null for an admitted policy source.
    native_identity: Option<NativeIdentity>,
    /// Caller-approved inert token, not an active href or untrusted content excerpt.
    citation_label: String,
    /// Fixed truthful recorded basis; not an authenticated approval or compliance judgment.
    approval_basis: &'static str,
    /// Exact scope of current-byte proof; no date-only review schedule is evaluated here.
    freshness_scope: &'static str,
    /// Fixed inert-data classification covering all caller/native metadata strings.
    content_classification: &'static str,
    /// Fixed qualification denying authentication and action authority.
    trust_boundary: &'static str,
}

/// Transport-neutral exact result for one authorized key; all nullable fields remain present.
#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
pub(crate) struct ArtifactStatus {
    /// Exact caller-visible opaque key, never a path or filesystem-discovered label.
    artifact_key: String,
    /// Fixed classification without partial trusted artifact content.
    availability: Availability,
    /// Complete available payload or explicit null; unavailable results contain no digest.
    artifact: Option<AvailableArtifact>,
}

/// Prepared bounded output retains all actual originals until the final transport fence.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
pub(crate) struct PreparedStatus {
    /// Real sealed capture; no detached status or Value can create a valid instance.
    captured: CapturedProject,
    /// Closed minimized typed result from the actual capture and explicit caller policy.
    status: ArtifactStatus,
    /// Complete bounded core JSON plus LF, not an MCP envelope or transport-framing claim.
    bytes: Vec<u8>,
}

#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
impl PreparedStatus {
    /// Borrow the actual minimized typed result for a Root-owned closed transport adapter.
    pub(crate) fn status(&self) -> &ArtifactStatus {
        &self.status
    }

    /// Borrow complete capped core bytes; a future wrapper must charge its own added bytes.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Require complete actual original stability immediately before caller publication.
    pub(crate) fn verify_inputs(&self, control: &mut dyn WorkControl) -> WorkResult<()> {
        self.captured.verify_inputs(control)
    }

    /// Return observed retained-original totals for private accounting only, never approval.
    pub(crate) fn retained_raw_bytes(&self) -> usize {
        self.captured.retained_raw_bytes
    }
}

/// Capture the complete caller-selected project once without temporary writes or subprocesses.
///
/// All caller paths, keys, disclosure choices and the complete roster are admitted before
/// filesystem reads. The fixed manifest must have the exact raw pin and decoded complete
/// declaration. Per-input remaining raw limits apply before reads; unsupported/invalid roles
/// are retained as untrusted observations rather than relabeled into native acceptance.
///
/// # Errors
/// Returns fixed safe failures for policy/capture/admission errors. Cooperative interruption
/// remains typed through every fence; parsers and syscalls themselves are not preempted.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
pub(crate) fn capture(
    project: &Path,
    caller: CallerPolicy,
    control: &mut dyn WorkControl,
) -> WorkResult<CapturedProject> {
    fence(control, Stage::ReadIndex)?;
    validate_policy(&caller)?;
    if project.as_os_str().is_empty()
        || project.components().any(|part| matches!(part, Component::CurDir | Component::ParentDir))
        || !crate::linkage::has_normalized_path_spelling(project)
    {
        return Err(policy_failure());
    }
    let absolute = std::path::absolute(project).map_err(|_| capture_failure())?;
    let root = Rc::new(fresh::qualify_root(&absolute).map_err(|_| capture_failure())?);
    let manifest = fresh::capture_local(
        Rc::clone(&root),
        Path::new(MANIFEST_PATH),
        MAX_MANIFEST_BYTES as u64,
        false,
    )
    .map_err(|_| capture_failure())?;
    let CapturedLocal::Present(raw_manifest, manifest_generation) = &manifest else {
        return Err(capture_failure());
    };
    if sha256_hex(raw_manifest) != caller.manifest_sha256 {
        return Err(policy_failure());
    }
    let actual = strict_value(raw_manifest, MAX_MANIFEST_BYTES)?;
    let expected_bytes = encode_limited(&caller.declaration, MAX_MANIFEST_BYTES)?;
    let expected = strict_value(&expected_bytes, MAX_MANIFEST_BYTES)?;
    if actual != expected {
        return Err(policy_failure());
    }
    drop(actual);
    drop(expected);
    drop(expected_bytes);
    fence(control, Stage::ReadIndex)?;
    let mut retained_raw_bytes = raw_manifest.len();
    let mut identities = BTreeSet::new();
    identities.insert(manifest_generation.identity());
    let mut objects = Vec::with_capacity(caller.declaration.resources.len());
    for policy in caller.declaration.resources {
        fence(control, Stage::CaptureResource)?;
        let maximum = if policy.role == Role::LifecycleRecord {
            usize::try_from(record::MAX_RECORD_BYTES).map_err(|_| limit_failure())?
        } else {
            MAX_RESOURCE_BYTES
        };
        let remaining = MAX_RAW_BYTES.checked_sub(retained_raw_bytes).ok_or_else(limit_failure)?;
        let original = fresh::capture_local(
            Rc::clone(&root),
            Path::new(&policy.path),
            maximum.min(remaining) as u64,
            true,
        )
        .map_err(|_| capture_failure())?;
        let (digest, pin_matches, native) = match &original {
            CapturedLocal::Present(bytes, generation) => {
                if !identities.insert(generation.identity()) || identities.len() > MAX_ORIGINALS {
                    return Err(capture_failure());
                }
                retained_raw_bytes = retained_raw_bytes
                    .checked_add(bytes.len())
                    .filter(|count| *count <= MAX_RAW_BYTES)
                    .ok_or_else(limit_failure)?;
                let digest = sha256_hex(bytes);
                let pin_matches = digest == policy.expected_sha256;
                fence(control, Stage::ValidateResource)?;
                let native = if policy.role.native_model().is_some() {
                    native_facts(bytes, policy.role)
                } else {
                    None
                };
                fence(control, Stage::ValidateResource)?;
                (Some(digest), pin_matches, native)
            }
            CapturedLocal::Absent(_) => (None, false, None),
        };
        objects.push(CapturedObject { policy, original, digest, pin_matches, native });
        fence(control, Stage::CaptureResource)?;
    }
    let captured = CapturedProject {
        root,
        manifest,
        objects,
        visible_keys: caller.visible_keys,
        approval_basis: caller.approval_basis,
        retained_raw_bytes,
    };
    captured.verify_inputs(control)?;
    Ok(captured)
}

/// Prepare minimized exact status from one real complete held capture, with no second capture.
///
/// Missing, stale, invalid, unsupported or unapproved dependencies produce a fixed reason and
/// null artifact payload. Query keys outside caller disclosure get the same safe ordinary
/// refusal regardless of whether a private file or registration exists.
///
/// # Errors
/// Returns fixed safe admission/output/original-drift failures or preserves typed interruption.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
pub(crate) fn get_artifact_status(
    captured: CapturedProject,
    key: &str,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedStatus> {
    fence(control, Stage::PrepareDomain)?;
    if !safe_key(key) || !captured.visible_keys.iter().any(|allowed| allowed == key) {
        return Err(policy_failure());
    }
    let object = captured
        .objects
        .iter()
        .find(|object| object.policy.key == key)
        .ok_or_else(policy_failure)?;
    let availability = evaluate(&captured, object, control)?;
    let artifact = if availability == Availability::Available {
        Some(AvailableArtifact {
            role: object.policy.role,
            content_sha256: object.digest.clone().ok_or_else(capture_failure)?,
            native_identity: object.native.clone(),
            citation_label: object.policy.citation_label.clone(),
            approval_basis: "recorded-current",
            freshness_scope: "held-originals-and-latest-recorded-approved-fingerprints; review-schedule-not-evaluated",
            content_classification: "untrusted-data",
            trust_boundary: TRUST_BOUNDARY,
        })
    } else {
        None
    };
    let status = ArtifactStatus { artifact_key: key.to_owned(), availability, artifact };
    captured.verify_inputs(control)?;
    let mut bytes = encode_limited(&status, MAX_RESPONSE_BYTES - 1)?;
    bytes.push(b'\n');
    fence(control, Stage::RetainPrepared)?;
    Ok(PreparedStatus { captured, status, bytes })
}

/// Validate complete caller policy before any resource observation or retained index growth.
fn validate_policy(caller: &CallerPolicy) -> WorkResult<()> {
    let declaration = &caller.declaration;
    if !valid_hash(&caller.manifest_sha256)
        || declaration.schema_version != MANIFEST_PROFILE
        || !safe_key(&declaration.project_key)
        || declaration.resources.len() > MAX_RESOURCES
        || caller.visible_keys.len() > declaration.resources.len()
    {
        return Err(policy_failure());
    }
    let mut keys = BTreeSet::new();
    let mut labels = BTreeSet::new();
    let mut paths = vec![MANIFEST_PATH.to_owned()];
    for resource in &declaration.resources {
        if !safe_key(&resource.key)
            || !safe_key(&resource.citation_label)
            || !keys.insert(resource.key.as_str())
            || !labels.insert(resource.citation_label.as_str())
            || !valid_hash(&resource.expected_sha256)
            || !portable_path(&resource.path)
            || resource.lifecycle_key.as_deref().is_some_and(|key| !safe_key(key))
            || !valid_native_policy(resource)
            || paths.iter().any(|prior| paths_overlap(prior, &resource.path))
        {
            return Err(policy_failure());
        }
        paths.push(resource.path.clone());
    }
    let mut visible = BTreeSet::new();
    for key in &caller.visible_keys {
        if !keys.contains(key.as_str()) || !visible.insert(key.as_str()) {
            return Err(policy_failure());
        }
    }
    for resource in &declaration.resources {
        if let Some(key) = &resource.lifecycle_key {
            if !declaration
                .resources
                .iter()
                .any(|entry| entry.key == *key && entry.role == Role::LifecycleRecord)
            {
                return Err(policy_failure());
            }
        }
    }
    Ok(())
}

/// Require exact native role/identity pairing without inventing native authority for metadata roles.
fn valid_native_policy(resource: &ResourcePolicy) -> bool {
    match (resource.role.native_model(), &resource.native_identity) {
        (Some(model), Some(identity)) => {
            identity.model == model.as_str()
                && !identity.root_id.is_empty()
                && identity.root_id.len() <= 45
                && uuid::Uuid::parse_str(&identity.root_id).is_ok()
                && identity.document_version.len() <= MAX_VERSION_BYTES
                && identity.oscal_version.len() <= 64
        }
        (None, None) => true,
        _ => false,
    }
}

/// Validate actual native JSON with existing official schema/version/semantic admission.
///
/// Parsed/schema forms are separately bounded by their raw/depth/string inputs; this is
/// not an encoded-output bound on native validation's internal heap or parser duration.
fn native_facts(bytes: &[u8], role: Role) -> Option<NativeIdentity> {
    let expected = role.native_model()?;
    let value = strict_value(bytes, MAX_RESOURCE_BYTES).ok()?;
    let detected = crate::validate::detect_model_type(&value).ok()?;
    if detected != expected
        || !crate::validate::run_full_validation("captured MCP artifact", &value, expected)
            .ok()?
            .is_valid()
    {
        return None;
    }
    let root = value.get(expected.as_str())?;
    let metadata = root.get("metadata")?;
    let root_id = root.get("uuid")?.as_str()?;
    let document_version = metadata.get("version")?.as_str()?;
    let oscal_version = metadata.get("oscal-version")?.as_str()?;
    if root_id.len() > 45 || document_version.len() > MAX_VERSION_BYTES || oscal_version.len() > 64
    {
        return None;
    }
    Some(NativeIdentity {
        model: expected.as_str().to_owned(),
        root_id: root_id.to_owned(),
        document_version: document_version.to_owned(),
        oscal_version: oscal_version.to_owned(),
    })
}

/// Evaluate recorded-current approval only after explicit caller basis and actual raw admission.
fn evaluate(
    captured: &CapturedProject,
    object: &CapturedObject,
    control: &mut dyn WorkControl,
) -> WorkResult<Availability> {
    if captured.approval_basis == ApprovalBasis::Unavailable {
        return Ok(Availability::CurrentApprovalUnavailable);
    }
    if !object.policy.role.status_supported() {
        return Ok(Availability::UnsupportedRole);
    }
    if object.digest.is_none() {
        return Ok(Availability::Missing);
    }
    if !object.pin_matches {
        return Ok(Availability::ContentChanged);
    }
    if object.policy.role.native_model().is_some() {
        let Some(native) = &object.native else {
            return Ok(Availability::InvalidArtifact);
        };
        if object.policy.native_identity.as_ref() != Some(native) {
            return Ok(Availability::IdentityMismatch);
        }
    }
    let Some(key) = &object.policy.lifecycle_key else {
        return Ok(Availability::LifecycleUnavailable);
    };
    let Some(lifecycle) = captured
        .objects
        .iter()
        .find(|entry| entry.policy.key == *key && entry.policy.role == Role::LifecycleRecord)
    else {
        return Ok(Availability::LifecycleUnavailable);
    };
    if !lifecycle.pin_matches {
        return Ok(Availability::LifecycleUnavailable);
    }
    let CapturedLocal::Present(bytes, _) = &lifecycle.original else {
        return Ok(Availability::LifecycleUnavailable);
    };
    fence(control, Stage::PrepareDomain)?;
    let Ok(record) = record::parse(bytes) else {
        return Ok(Availability::InvalidLifecycle);
    };
    fence(control, Stage::PrepareDomain)?;
    let Some(current) = complete_closure(captured, lifecycle, object, &record) else {
        return Ok(Availability::IncompleteClosure);
    };
    let Ok(projected) = status::status_from_captured(&record, &current, None) else {
        return Ok(Availability::InvalidLifecycle);
    };
    if projected.state != LifecycleState::Approved {
        return Ok(Availability::NotApproved);
    }
    if projected.approved_fingerprints.as_ref() != Some(&current.fingerprints)
        || !projected.artifact_identity_changes.is_empty()
        || !projected.blockers.is_empty()
        || projected.derived_status != "approved"
    {
        return Ok(Availability::ApprovedDrifted);
    }
    fence(control, Stage::PrepareDomain)?;
    Ok(Availability::Available)
}

/// Build the complete exact current source/generated tuple from already captured originals only.
///
/// Record paths are private record-parent-relative names. Every complete dependency must
/// resolve to the same explicit lifecycle binding; unsafe/outside or undeclared names fail
/// without any additional read. Latest approved hashes, rather than copied metadata labels,
/// are compared by the actual existing lifecycle projector.
fn complete_closure(
    captured: &CapturedProject,
    lifecycle: &CapturedObject,
    selected: &CapturedObject,
    record: &record::LifecycleRecord,
) -> Option<CurrentArtifacts> {
    let source_path = resolve_record_path(&lifecycle.policy.path, &record.policy.source.path)?;
    let source = closure_member(captured, &lifecycle.policy.key, &source_path)?;
    if !matches!(source.policy.role, Role::PolicySource | Role::LifecycleSource)
        || record.policy.source.oscal_type.is_some()
        || record.policy.source.root_uuid.is_some()
    {
        return None;
    }
    let mut selected_found = source.policy.key == selected.policy.key;
    let mut generated = Vec::with_capacity(record.policy.generated_artifacts.len());
    let mut identities_changed = Vec::new();
    for expected in &record.policy.generated_artifacts {
        let path = resolve_record_path(&lifecycle.policy.path, &expected.path)?;
        let actual = closure_member(captured, &lifecycle.policy.key, &path)?;
        let identity = actual.native.as_ref()?;
        if actual.policy.role.native_model().is_none()
            || actual.policy.native_identity.as_ref() != Some(identity)
        {
            return None;
        }
        if expected.oscal_type.as_deref() != Some(identity.model.as_str())
            || expected.root_uuid.as_deref() != Some(identity.root_id.as_str())
        {
            identities_changed.push(expected.path.clone());
        }
        selected_found |= actual.policy.key == selected.policy.key;
        generated.push(NamedHash { path: expected.path.clone(), sha256: actual.digest.clone()? });
    }
    if !selected_found {
        return None;
    }
    generated.sort();
    Some(CurrentArtifacts {
        fingerprints: FingerprintSet {
            source_sha256: source.digest.clone()?,
            generated_artifacts: generated,
        },
        identity_changes: identities_changed,
    })
}

/// Select one actual present pin/identity-admitted closure member with the exact lifecycle key.
fn closure_member<'a>(
    captured: &'a CapturedProject,
    lifecycle_key: &str,
    path: &str,
) -> Option<&'a CapturedObject> {
    let entry = captured.objects.iter().find(|entry| entry.policy.path == path)?;
    if !entry.pin_matches
        || entry.digest.is_none()
        || entry.policy.lifecycle_key.as_deref() != Some(lifecycle_key)
    {
        return None;
    }
    Some(entry)
}

/// Compose private record-relative names only after each original spelling is admitted.
fn resolve_record_path(record_path: &str, relative: &str) -> Option<String> {
    if !portable_path(record_path) || !portable_path(relative) {
        return None;
    }
    let Some((parent, _filename)) = record_path.rsplit_once('/') else {
        return Some(relative.to_owned());
    };
    let length = parent.len().checked_add(1)?.checked_add(relative.len())?;
    if length > MAX_PATH_BYTES {
        return None;
    }
    let mut joined = String::with_capacity(length);
    joined.push_str(parent);
    joined.push('/');
    joined.push_str(relative);
    portable_path(&joined).then_some(joined)
}

/// Require this conservative first-profile portable ASCII descendant, with no alias normalization.
fn portable_path(value: &str) -> bool {
    if value.is_empty()
        || value.len() > MAX_PATH_BYTES
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-/".contains(&byte))
        || fresh::validate_relative(Path::new(value)).is_err()
    {
        return false;
    }
    value.split('/').all(|part| {
        let upper = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        !part.is_empty()
            && part.len() <= 128
            && !part.ends_with('.')
            && !matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            && !(upper.len() == 4
                && (upper.starts_with("COM") || upper.starts_with("LPT"))
                && upper.as_bytes()[3].is_ascii_digit())
    })
}

/// Compare complete portable components after ASCII fold, including ancestor file conflicts.
fn paths_overlap(left: &str, right: &str) -> bool {
    let left = left.to_ascii_lowercase();
    let right = right.to_ascii_lowercase();
    left == right
        || left.strip_prefix(&right).is_some_and(|tail| tail.starts_with('/'))
        || right.strip_prefix(&left).is_some_and(|tail| tail.starts_with('/'))
}

/// Admit one safe explicit identifier or citation token, never a path, glob, URI or arbitrary prose.
fn safe_key(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

/// Require the exact lowercase raw SHA-256 representation used by the declared caller profile.
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Strict actual-byte JSON admission rejects duplicates/BOM/UTF-8/depth/string/work overflow.
fn strict_value(bytes: &[u8], maximum: usize) -> WorkResult<Value> {
    crate::workspace::contract::parse(bytes, maximum, MAX_STRING_BYTES)
        .map_err(|_| policy_failure())
}

/// Preserve typed cooperative cancellation/deadline/shutdown at every consumed boundary.
fn fence(control: &mut dyn WorkControl, stage: Stage) -> WorkResult<()> {
    control.checkpoint(stage, ProgressUpdate::Unchanged)?;
    if let Some(reason) = control.interruption() {
        return Err(WorkError::Interrupted(reason));
    }
    Ok(())
}

/// Recheck real original bytes or actual typed absence without constructing stand-in proofs.
fn verify_original(original: &CapturedLocal) -> WorkResult<()> {
    match original {
        CapturedLocal::Present(bytes, generation) => fresh::verify_file(generation, bytes),
        CapturedLocal::Absent(generation) => fresh::verify_absence(generation),
    }
    .map_err(|_| capture_failure())
}

/// Fixed safe caller-policy refusal, identical for unknown and undisclosed keys.
fn policy_failure() -> WorkError {
    WorkError::Failed(Error::new(
        "mcp-core-unavailable",
        "The requested status is unavailable under the supplied policy.",
        false,
    ))
}

/// Fixed safe original-generation failure, without native diagnostics or private paths.
fn capture_failure() -> WorkError {
    WorkError::Failed(Error::new(
        "mcp-core-input-changed-or-unavailable",
        "The original inputs cannot be verified.",
        false,
    ))
}

/// Fixed safe bounded-data refusal; these provisional core codes are not MCP wire errors.
fn limit_failure() -> WorkError {
    WorkError::Failed(Error::new(
        "mcp-core-limit-exceeded",
        "The bounded core result cannot be prepared.",
        false,
    ))
}

/// Retained serializer whose complete fragment is admitted before any buffer append.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
struct CappedWriter {
    /// Complete retained bytes admitted so far, never a partial success output.
    bytes: Vec<u8>,
    /// Complete immutable byte ceiling selected before the first serializer fragment.
    maximum: usize,
}

impl Write for CappedWriter {
    /// Reject the full fragment before extending when it would exceed the immutable limit.
    fn write(&mut self, fragment: &[u8]) -> io::Result<usize> {
        let end = self
            .bytes
            .len()
            .checked_add(fragment.len())
            .filter(|end| *end <= self.maximum)
            .ok_or_else(|| io::Error::other("bounded core output exceeded"))?;
        self.bytes.reserve(end - self.bytes.len());
        self.bytes.extend_from_slice(fragment);
        Ok(fragment.len())
    }

    /// No external writer or publication is performed by this retained local serializer.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encode complete borrowed typed data with before-fragment bounds; no `to_vec` then cap.
#[cfg_attr(not(test), allow(dead_code))] // Retained transport-neutral core regression port; production uses the complete disclosure/query path.
fn encode_limited(value: &impl Serialize, maximum: usize) -> WorkResult<Vec<u8>> {
    let mut writer = CappedWriter { bytes: Vec::new(), maximum };
    serde_json::to_writer(&mut writer, value).map_err(|_| limit_failure())?;
    Ok(writer.bytes)
}

/// Proposed actual-file controls; no compiler, test, platform or authority result is claimed.
#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::NaiveDate;
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::lifecycle::record::{
        APPROVAL_POLICY_VERSION, ApprovalPolicy, ArtifactFingerprint, DeclaredRole, Party,
        PolicyIdentity, ReviewSchedule, RoleRequirement, SCHEMA_VERSION, SeparationRules,
        TimezonePolicy, TransitionEvent,
    };
    use crate::workspace::preparation::{Interruption, NoopControl};

    /// Genuine synthetic local originals and independently declared caller policy for tests.
    struct TestProject {
        /// Owned ordinary temporary project, never a production or authoritative source.
        directory: TempDir,
        /// Complete caller declaration copied into the independently pinned local manifest.
        declaration: ProjectDeclaration,
    }

    impl TestProject {
        /// Write a fully native-valid Catalog, policy source and real intrinsic lifecycle history.
        #[allow(clippy::too_many_lines)] // Complete genuine source/native/lifecycle fixture remains visible together.
        fn new(approved: bool) -> Self {
            let directory = tempfile::tempdir().expect("synthetic project directory");
            let source = b"Synthetic private policy prose; not an instruction channel.\n";
            let catalog = json!({"catalog":{
                "uuid":"11111111-1111-4111-8111-111111111111",
                "metadata":{"title":"Synthetic private instruction title", "last-modified":"2026-09-01T00:00:00Z", "version":"1", "oscal-version":"1.2.3"},
                "controls":[{"id":"synthetic-a", "title":"Synthetic control"}]
            }});
            let catalog_bytes = serde_json::to_vec(&catalog).expect("native fixture JSON");
            fs::write(directory.path().join("source.txt"), source).expect("actual source fixture");
            fs::write(directory.path().join("catalog.json"), &catalog_bytes)
                .expect("actual native fixture");
            let mut lifecycle = record::LifecycleRecord {
                schema_version: SCHEMA_VERSION.to_owned(),
                policy: PolicyIdentity {
                    policy_key: "synthetic-policy".to_owned(),
                    version_key: "version-1".to_owned(),
                    title: "Sensitive local record title".to_owned(),
                    owner_keys: vec!["private-owner".to_owned()],
                    source: ArtifactFingerprint {
                        path: "source.txt".to_owned(),
                        sha256: sha256_hex(source),
                        oscal_type: None,
                        root_uuid: None,
                    },
                    generated_artifacts: vec![ArtifactFingerprint {
                        path: "catalog.json".to_owned(),
                        sha256: sha256_hex(&catalog_bytes),
                        oscal_type: Some("catalog".to_owned()),
                        root_uuid: Some("11111111-1111-4111-8111-111111111111".to_owned()),
                    }],
                },
                parties: vec![
                    Party { key: "private-owner".to_owned(), roles: vec![DeclaredRole::Owner] },
                    Party {
                        key: "private-reviewer".to_owned(),
                        roles: vec![DeclaredRole::Reviewer],
                    },
                    Party {
                        key: "private-approver".to_owned(),
                        roles: vec![DeclaredRole::Approver],
                    },
                ],
                approval_policy: ApprovalPolicy {
                    schema_version: APPROVAL_POLICY_VERSION.to_owned(),
                    required_roles: vec![
                        RoleRequirement { role: DeclaredRole::Reviewer, count: 1 },
                        RoleRequirement { role: DeclaredRole::Approver, count: 1 },
                    ],
                    separation: SeparationRules::default(),
                },
                review: ReviewSchedule {
                    cadence_days: 30,
                    next_review_date: NaiveDate::from_ymd_opt(2026, 12, 1).expect("synthetic date"),
                    due_soon_days: 7,
                    timezone_policy: TimezonePolicy::DateOnly,
                },
                state: LifecycleState::Draft,
                replaced_by: None,
                history: Vec::new(),
            };
            if approved {
                let fingerprints = FingerprintSet {
                    source_sha256: sha256_hex(source),
                    generated_artifacts: vec![NamedHash {
                        path: "catalog.json".to_owned(),
                        sha256: sha256_hex(&catalog_bytes),
                    }],
                };
                append_event(&mut lifecycle, LifecycleState::InReview, &fingerprints);
                append_event(&mut lifecycle, LifecycleState::Approved, &fingerprints);
            }
            record::validate(&lifecycle).expect("intrinsically admitted fixture record");
            let lifecycle_bytes =
                serde_json::to_vec(&lifecycle).expect("real lifecycle fixture JSON");
            fs::write(directory.path().join("lifecycle.json"), &lifecycle_bytes)
                .expect("actual record fixture");
            let native_identity = NativeIdentity {
                model: "catalog".to_owned(),
                root_id: "11111111-1111-4111-8111-111111111111".to_owned(),
                document_version: "1".to_owned(),
                oscal_version: "1.2.3".to_owned(),
            };
            let declaration = ProjectDeclaration {
                schema_version: MANIFEST_PROFILE.to_owned(),
                project_key: "synthetic-project".to_owned(),
                resources: vec![
                    ResourcePolicy {
                        key: "catalog".to_owned(),
                        role: Role::OscalCatalogArtifact,
                        path: "catalog.json".to_owned(),
                        expected_sha256: sha256_hex(&catalog_bytes),
                        native_identity: Some(native_identity),
                        lifecycle_key: Some("lifecycle".to_owned()),
                        citation_label: "synthetic-catalog".to_owned(),
                    },
                    ResourcePolicy {
                        key: "source".to_owned(),
                        role: Role::PolicySource,
                        path: "source.txt".to_owned(),
                        expected_sha256: sha256_hex(source),
                        native_identity: None,
                        lifecycle_key: Some("lifecycle".to_owned()),
                        citation_label: "synthetic-source".to_owned(),
                    },
                    ResourcePolicy {
                        key: "lifecycle".to_owned(),
                        role: Role::LifecycleRecord,
                        path: "lifecycle.json".to_owned(),
                        expected_sha256: sha256_hex(&lifecycle_bytes),
                        native_identity: None,
                        lifecycle_key: None,
                        citation_label: "synthetic-lifecycle".to_owned(),
                    },
                ],
            };
            let project = Self { directory, declaration };
            project.write_manifest();
            project
        }

        /// Return the actual canonical known fixture root, not an erased unsafe user spelling.
        fn root(&self) -> std::path::PathBuf {
            self.directory.path().canonicalize().expect("known ordinary fixture root")
        }

        /// Publish exact synthetic expected declaration before actual production capture.
        fn write_manifest(&self) {
            fs::write(
                self.directory.path().join(MANIFEST_PATH),
                serde_json::to_vec(&self.declaration).expect("closed caller declaration"),
            )
            .expect("ordinary fixture manifest");
        }

        /// Construct explicit caller policy, including a real manifest raw pin and chosen gate.
        fn caller(&self, approval_basis: ApprovalBasis) -> CallerPolicy {
            CallerPolicy {
                manifest_sha256: sha256_hex(
                    &fs::read(self.directory.path().join(MANIFEST_PATH))
                        .expect("actual fixture manifest"),
                ),
                declaration: self.declaration.clone(),
                visible_keys: vec!["catalog".to_owned(), "source".to_owned()],
                approval_basis,
            }
        }

        /// Update only the explicit caller pin after an intentional changed fixture generation.
        fn refresh_pin(&mut self, key: &str) {
            let entry = self
                .declaration
                .resources
                .iter_mut()
                .find(|entry| entry.key == key)
                .expect("declared fixture key");
            entry.expected_sha256 = sha256_hex(
                &fs::read(self.directory.path().join(&entry.path)).expect("changed actual fixture"),
            );
            self.write_manifest();
        }

        /// Capture real originals once through the production constructor and query exact key.
        fn status(&self, key: &str, approval_basis: ApprovalBasis) -> PreparedStatus {
            let mut control = NoopControl;
            let captured = capture(&self.root(), self.caller(approval_basis), &mut control)
                .expect("actual complete fixture capture");
            get_artifact_status(captured, key, &mut control).expect("actual exact captured status")
        }
    }

    /// Use the real deterministic lifecycle event-ID function; no forged approval schema fixture.
    fn append_event(
        record: &mut record::LifecycleRecord,
        next: LifecycleState,
        fingerprints: &FingerprintSet,
    ) {
        let sequence = u32::try_from(record.history.len() + 1).expect("bounded synthetic history");
        let (actor, role) = if next == LifecycleState::InReview {
            ("private-reviewer", DeclaredRole::Reviewer)
        } else {
            ("private-approver", DeclaredRole::Approver)
        };
        let mut event = TransitionEvent {
            sequence,
            event_id: String::new(),
            legacy_event_id: None,
            previous_state: record.state,
            next_state: next,
            actor_key: actor.to_owned(),
            declared_role: role,
            timestamp: format!("2026-09-01T{sequence:02}:00:00Z"),
            rationale: "Private rationale remains undisclosed".to_owned(),
            fingerprints: fingerprints.clone(),
            assertions: Vec::new(),
            impact_finding_ids: Vec::new(),
            replacement: None,
        };
        event.event_id = record::event_id(record, &event).expect("actual intrinsic event ID");
        record.history.push(event);
        record.state = next;
    }

    /// Actual approved/current native result contains exact digest and no private record/prose/path.
    #[test]
    fn approved_current_uses_real_native_and_recorded_closure_with_minimized_payload() {
        let project = TestProject::new(true);
        let before = fs::read(project.directory.path().join("catalog.json"))
            .expect("original native fixture");
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::Available);
        let artifact = prepared.status().artifact.as_ref().expect("available exact native facts");
        assert_eq!(artifact.content_sha256, sha256_hex(&before));
        assert_eq!(
            artifact.native_identity.as_ref().expect("actual native identity").document_version,
            "1"
        );
        assert_eq!(artifact.content_classification, "untrusted-data");
        let rendered = std::str::from_utf8(prepared.bytes()).expect("complete core JSON");
        for private in [
            "private-owner",
            "private-reviewer",
            "private-approver",
            "Private rationale",
            "Sensitive local record title",
            "Synthetic private instruction title",
            "source.txt",
            "catalog.json",
            "lifecycle.json",
        ] {
            assert!(!rendered.contains(private), "private original metadata leaked");
        }
        assert!(!rendered.contains(project.root().to_str().expect("fixture path")));
        prepared.verify_inputs(&mut NoopControl).expect("actual final original fence");
        assert_eq!(
            fs::read(project.directory.path().join("catalog.json")).expect("native after status"),
            before
        );
        assert!(prepared.retained_raw_bytes() <= MAX_RAW_BYTES);
        assert!(prepared.bytes().ends_with(b"\n"));
    }

    /// Recorded Approved bytes alone never select the caller's default-unavailable basis.
    #[test]
    fn default_caller_basis_has_null_payload_and_no_artifact_hash() {
        let project = TestProject::new(true);
        let prepared = project.status("catalog", ApprovalBasis::default());
        assert_eq!(prepared.status().availability, Availability::CurrentApprovalUnavailable);
        assert!(prepared.status().artifact.is_none());
        assert!(
            !std::str::from_utf8(prepared.bytes()).expect("safe JSON").contains("content_sha256")
        );
    }

    /// Intrinsically admitted draft state is unavailable despite valid current bytes and caller pins.
    #[test]
    fn current_native_without_recorded_approval_does_not_become_approved() {
        let project = TestProject::new(false);
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::NotApproved);
        assert!(prepared.status().artifact.is_none());
    }

    /// Changed actual native bytes differ from explicit caller pin and yield no partial trusted data.
    #[test]
    fn pin_mismatch_refuses_payload_even_when_changed_catalog_remains_native_valid() {
        let project = TestProject::new(true);
        let path = project.directory.path().join("catalog.json");
        let mut changed: Value =
            serde_json::from_slice(&fs::read(&path).expect("actual native")).expect("fixture JSON");
        changed["catalog"]["metadata"]["version"] = json!("2");
        fs::write(path, serde_json::to_vec(&changed).expect("changed valid native shape"))
            .expect("new original generation bytes");
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::ContentChanged);
        assert!(prepared.status().artifact.is_none());
    }

    /// Current explicit pins do not rewrite the real latest recorded approval fingerprint tuple.
    #[test]
    fn caller_repin_of_changed_policy_source_remains_approved_drifted() {
        let mut project = TestProject::new(true);
        fs::write(
            project.directory.path().join("source.txt"),
            b"Changed original private source.\n",
        )
        .expect("changed local source");
        project.refresh_pin("source");
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::ApprovedDrifted);
        assert!(prepared.status().artifact.is_none());
    }

    /// Valid native identity must equal the complete independent caller identity, not just its type.
    #[test]
    fn wrong_expected_root_identity_does_not_disclose_valid_current_native() {
        let mut project = TestProject::new(true);
        project.declaration.resources[0]
            .native_identity
            .as_mut()
            .expect("declared native identity")
            .root_id = "22222222-2222-4222-8222-222222222222".to_owned();
        project.write_manifest();
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::IdentityMismatch);
        assert!(prepared.status().artifact.is_none());
    }

    /// Complete closure cannot fetch an omitted source or infer a source dependency from metadata.
    #[test]
    fn undeclared_current_source_refuses_complete_lifecycle_closure() {
        let mut project = TestProject::new(true);
        project.declaration.resources.retain(|entry| entry.key != "source");
        project.write_manifest();
        let mut caller = project.caller(ApprovalBasis::RecordedCurrent);
        caller.visible_keys = vec!["catalog".to_owned()];
        let captured = capture(&project.root(), caller, &mut NoopControl)
            .expect("complete explicit reduced roster");
        let prepared = get_artifact_status(captured, "catalog", &mut NoopControl)
            .expect("bounded unavailable result");
        assert_eq!(prepared.status().availability, Availability::IncompleteClosure);
        assert!(prepared.status().artifact.is_none());
    }

    /// Invalid actual native JSON is unavailable after caller re-pin, without trusting a model label.
    #[test]
    fn native_schema_failure_survives_hash_repin_and_keeps_payload_null() {
        let mut project = TestProject::new(true);
        fs::write(
            project.directory.path().join("catalog.json"),
            br#"{"catalog":{"uuid":"11111111-1111-4111-8111-111111111111"}}"#,
        )
        .expect("actual schema-invalid original");
        project.refresh_pin("catalog");
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::InvalidArtifact);
        assert!(prepared.status().artifact.is_none());
    }

    /// Duplicate lifecycle fields are refused by the actual strict record parser, not normalized.
    #[test]
    fn duplicate_record_keys_cannot_become_intrinsically_valid_recorded_approval() {
        let mut project = TestProject::new(true);
        let path = project.directory.path().join("lifecycle.json");
        let raw = fs::read_to_string(&path).expect("actual record text");
        let duplicated = raw.replacen('{', "{\"schema_version\":\"forge.policy-lifecycle/2\",", 1);
        fs::write(path, duplicated).expect("duplicate-key actual original");
        project.refresh_pin("lifecycle");
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::InvalidLifecycle);
        assert!(prepared.status().artifact.is_none());
    }

    /// Unknown and known-but-undisclosed keys share the same fixed safe refusal with no lookup leak.
    #[test]
    fn caller_disclosure_subset_is_admitted_before_private_key_lookup() {
        let project = TestProject::new(true);
        for key in ["lifecycle", "private-missing"] {
            let captured = capture(
                &project.root(),
                project.caller(ApprovalBasis::RecordedCurrent),
                &mut NoopControl,
            )
            .expect("real capture");
            match get_artifact_status(captured, key, &mut NoopControl) {
                Err(WorkError::Failed(error)) => assert_eq!(error.code, "mcp-core-unavailable"),
                _ => panic!("undisclosed or absent key must share safe refusal"),
            }
        }
    }

    /// Capture records genuine missing ancestry and appearance invalidates that actual absence proof.
    #[test]
    fn missing_artifact_has_null_payload_and_appearance_fails_final_proof() {
        let project = TestProject::new(true);
        fs::remove_file(project.directory.path().join("catalog.json"))
            .expect("fixture removed before capture");
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        assert_eq!(prepared.status().availability, Availability::Missing);
        assert!(prepared.status().artifact.is_none());
        prepared.verify_inputs(&mut NoopControl).expect("same actual absence");
        fs::write(project.directory.path().join("catalog.json"), b"newly appeared bytes")
            .expect("ordinary new file");
        assert!(prepared.verify_inputs(&mut NoopControl).is_err());
    }

    /// Same bytes in a different actual native file instance do not satisfy held-generation proof.
    #[cfg(unix)]
    #[test]
    fn byte_equal_native_replacement_fails_actual_identity_recheck() {
        let project = TestProject::new(true);
        let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
        let path = project.directory.path().join("catalog.json");
        let bytes = fs::read(&path).expect("actual original bytes");
        fs::rename(&path, project.directory.path().join("old-catalog.json"))
            .expect("Unix held original relocation");
        fs::write(path, bytes).expect("different actual native instance");
        assert!(prepared.verify_inputs(&mut NoopControl).is_err());
    }

    /// Caller expected declaration is complete: an extra raw manifest field is never ignored.
    #[test]
    fn hash_repin_does_not_allow_unknown_manifest_fields_or_partial_value_equality() {
        let project = TestProject::new(true);
        let mut caller = project.caller(ApprovalBasis::RecordedCurrent);
        let mut manifest: Value = serde_json::from_slice(
            &fs::read(project.directory.path().join(MANIFEST_PATH)).expect("actual declaration"),
        )
        .expect("fixture declaration JSON");
        manifest["unexpected_authority"] = json!(true);
        let raw = serde_json::to_vec(&manifest).expect("modified actual declaration");
        fs::write(project.directory.path().join(MANIFEST_PATH), &raw)
            .expect("changed manifest original");
        caller.manifest_sha256 = sha256_hex(&raw);
        assert!(capture(&project.root(), caller, &mut NoopControl).is_err());
    }

    /// Original raw path spelling is refused before absolute/root joining can erase unsafe aliases.
    #[test]
    fn original_project_and_resource_alias_spellings_are_not_normalized_into_admission() {
        let project = TestProject::new(true);
        let raw = std::ffi::OsString::from(format!("{}/./", project.root().display()));
        assert!(
            capture(
                Path::new(&raw),
                project.caller(ApprovalBasis::RecordedCurrent),
                &mut NoopControl
            )
            .is_err()
        );
        for alias in [
            "./catalog.json",
            "native/../catalog.json",
            "native//catalog.json",
            "catalog.json/",
            "CON.json",
        ] {
            let mut caller = project.caller(ApprovalBasis::RecordedCurrent);
            caller.declaration.resources[0].path = alias.to_owned();
            assert!(
                validate_policy(&caller).is_err(),
                "original private alias must fail before IO"
            );
        }
    }

    /// Both duplicate and ancestor folded paths refuse before any attempted resource capture.
    #[test]
    fn complete_roster_rejects_folded_prefix_and_manifest_aliases() {
        let project = TestProject::new(true);
        for alias in ["SOURCE.TXT", "source.txt/leaf.json", "FORGE.MCP.JSON"] {
            let mut caller = project.caller(ApprovalBasis::RecordedCurrent);
            caller.declaration.resources[0].path = alias.to_owned();
            assert!(validate_policy(&caller).is_err());
        }
        assert!(!paths_overlap("a.json", "ab.json"));
        assert!(paths_overlap("A", "a/leaf.json"));
    }

    /// Complete roster overflow refuses from caller policy before creating indexes or native reads.
    #[test]
    fn whole_resource_count_is_not_a_partial_prefix_admission() {
        let project = TestProject::new(true);
        let mut caller = project.caller(ApprovalBasis::RecordedCurrent);
        let template = caller.declaration.resources[1].clone();
        caller.declaration.resources = (0..=MAX_RESOURCES)
            .map(|index| {
                let mut entry = template.clone();
                entry.key = format!("key-{index}");
                entry.citation_label = format!("label-{index}");
                entry.path = format!("source-{index}.txt");
                entry
            })
            .collect();
        caller.visible_keys.clear();
        assert!(validate_policy(&caller).is_err());
    }

    /// Exact retained fragment bounds refuse before appending an overflowing serializer fragment.
    #[test]
    fn retained_response_writer_rejects_full_overflow_fragment_before_growth() {
        let mut writer = CappedWriter { bytes: Vec::new(), maximum: 3 };
        assert_eq!(writer.write(b"abc").expect("exact ceiling"), 3);
        assert!(writer.write(b"d").is_err());
        assert_eq!(writer.bytes, b"abc");
        assert_eq!(encode_limited(&"a", 3).expect("exact JSON encoding"), b"\"a\"");
        assert!(encode_limited(&"a", 2).is_err());
    }

    /// Sticky interruption remains typed, rather than being converted to invalid or unavailable.
    #[test]
    fn cancellation_is_preserved_at_the_first_actual_capture_fence() {
        /// Synthetic runtime control retaining the selected cancellation reason.
        struct Stop;
        impl WorkControl for Stop {
            /// Refuse the reached boundary using the actual typed cancellation channel.
            fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            }
            /// Return the same sticky stop; no ordinary error may erase it.
            fn interruption(&self) -> Option<Interruption> {
                Some(Interruption::CancelRequested)
            }
        }
        let project = TestProject::new(true);
        assert!(matches!(
            capture(&project.root(), project.caller(ApprovalBasis::RecordedCurrent), &mut Stop),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
    }

    /// Nested recorded closures retain the exact portable project spelling on both platforms.
    #[test]
    fn nested_record_parent_uses_actual_full_captured_native_and_source_closure() {
        for parent in ["records", "records/child"] {
            let mut project = TestProject::new(true);
            fs::create_dir_all(project.directory.path().join(parent))
                .expect("ordinary known fixture descendants");
            for entry in &mut project.declaration.resources {
                let old_path = entry.path.clone();
                let new_path = format!("{parent}/{old_path}");
                fs::rename(
                    project.directory.path().join(&old_path),
                    project.directory.path().join(&new_path),
                )
                .expect("rebase actual fixture before capture");
                entry.path = new_path;
            }
            project.write_manifest();
            let prepared = project.status("catalog", ApprovalBasis::RecordedCurrent);
            assert_eq!(prepared.status().availability, Availability::Available);
            assert_eq!(
                prepared
                    .status()
                    .artifact
                    .as_ref()
                    .expect("actual nested native facts")
                    .native_identity
                    .as_ref()
                    .expect("native identity")
                    .root_id,
                "11111111-1111-4111-8111-111111111111"
            );
            assert_eq!(
                prepared.status().artifact.as_ref().expect("actual digest").content_sha256,
                project.declaration.resources[0].expected_sha256
            );
            prepared
                .verify_inputs(&mut NoopControl)
                .expect("complete nested actual original proof");
            let rendered = std::str::from_utf8(prepared.bytes()).expect("minimized nested JSON");
            assert!(!rendered.contains(parent));
            assert!(!rendered.contains("source.txt"));
            assert!(!rendered.contains("catalog.json"));
        }
    }

    /// Both original path spellings and the complete combined byte limit remain fail-closed.
    #[test]
    fn portable_record_composition_preserves_alias_refusal_and_exact_whole_path_bound() {
        assert_eq!(
            resolve_record_path("lifecycle.json", "catalog.json").as_deref(),
            Some("catalog.json")
        );
        assert_eq!(
            resolve_record_path("records/child/lifecycle.json", "native/catalog.json").as_deref(),
            Some("records/child/native/catalog.json")
        );
        for record in [
            "./records/lifecycle.json",
            "records/../lifecycle.json",
            "records//lifecycle.json",
            "records\\lifecycle.json",
        ] {
            assert!(
                resolve_record_path(record, "catalog.json").is_none(),
                "original record alias refused"
            );
        }
        for relative in
            ["./catalog.json", "../catalog.json", "native//catalog.json", "native\\catalog.json"]
        {
            assert!(
                resolve_record_path("records/lifecycle.json", relative).is_none(),
                "original relative alias refused"
            );
        }
        let parent = ["r".repeat(100), "s".repeat(100), "t".repeat(100), "u".repeat(100)].join("/");
        let record = format!("{parent}/lifecycle.json");
        let remaining = MAX_PATH_BYTES - parent.len() - 1;
        let exact_relative = format!("{}.json", "x".repeat(remaining - 5));
        let exact =
            resolve_record_path(&record, &exact_relative).expect("exact complete portable bound");
        assert_eq!(exact.len(), MAX_PATH_BYTES);
        let overflow_relative = format!("{}.json", "x".repeat(remaining - 4));
        assert!(portable_path(&record));
        assert!(portable_path(&overflow_relative));
        assert!(resolve_record_path(&record, &overflow_relative).is_none());
        assert!(resolve_record_path("records/lifecycle.json", "CON.json").is_none());
    }
}

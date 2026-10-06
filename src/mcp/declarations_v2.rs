//! Closed MCP-only /2 declarations, strictly decoded without creating disclosure authority.
//!
//! Every result is inert asserted data. These functions perform no file I/O, clock sampling,
//! capture, builder invocation, native validation, approval promotion, or server publication.
//! The separately reviewed real capture/ledger and caller-policy constructors must consume
//! the complete declarations before a /2 runtime can be activated; /1 remains unchanged.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[cfg(test)]
use super::limit_failure;
use super::{
    MAX_STRING_BYTES, NativeIdentity, ResourcePolicy, Role, paths_overlap, policy_failure,
    portable_path, safe_key, strict_value, valid_hash, valid_native_policy,
};
use crate::hashing::lower_hex;
#[cfg(test)]
use crate::hashing::sha256_hex;
use crate::workspace::preparation::{Stage, WorkError, WorkResult};

/// Exact reviewed design binding; it is an engineering contract pin, not owner authentication.
const PROPOSAL: &str = "365ac7ddbb2cb1058efdfc211a5a525babcbbd69c30557a5113126e0c616cde8";
/// Every actual config retains the existing complete one-MiB strict-byte ceiling.
const CONFIG_BYTES: usize = 1024 * 1024;
/// Each complete borrowed byte traversal uses deterministic 32-KiB logical units.
/// This is an engineering work descriptor, not measured CPU instructions or heap bytes.
const WORK_BYTES: usize = 32 * 1024;
/// Three actual configs leave this many roster slots in the existing 1,001-original pool.
const ROSTER_LIMIT: usize = 998;
/// Paths for both project config originals are unavailable as declared resource targets.
const PROJECT_CONFIGS: [&str; 2] = ["forge.mcp.json", "forge.mcp.visibility.json"];
/// Actual recorded read-decision pairs preserve the six existing /1 obligations.
const READ_PAIRS: [(&str, &str); 6] = [
    ("D067-P1", "product"),
    ("D067-P2", "product"),
    ("D067-S1", "security"),
    ("D067-E1", "engineering"),
    ("D067-S2", "product"),
    ("D067-S2", "security"),
];
/// Build/source/index scope is declared separately, never borrowed from a final read decision.
const BUILD_PAIRS: [(&str, &str); 6] = [
    ("F20-BUILD", "product"),
    ("F20-BUILD", "security"),
    ("F20-SOURCE", "product"),
    ("F20-SOURCE", "security"),
    ("F20-INDEX", "product"),
    ("F20-INDEX", "security"),
];

/// Complete closed /2 project roster; original base `ResourcePolicy` and Role are reused unchanged.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectDeclarationV2 {
    /// Exact `forge.mcp-project/2` selector; /1 is not upgraded or accepted here.
    pub(crate) schema_version: String,
    /// Asserted opaque project token, equal across all selected configs.
    pub(crate) project_key: String,
    /// Complete original base-resource roster, including hidden dependencies.
    pub(crate) resources: Vec<ResourcePolicy>,
    /// Complete separately discriminated companion roster, not extra workspace roles.
    pub(crate) companion_resources: Vec<Companion>,
}

/// Exact binding of one native requirement declaration to a base Catalog or Profile.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_field_names,
    reason = "The closed /2 wire schema requires these exact requirement, resource and resolved-Catalog key field names."
)]
pub(crate) struct RequirementBinding {
    /// Native linkage requirement key, reconciled against actual manifest bytes later.
    pub(crate) requirement_key: String,
    /// Exact base native resource key, never a caller-provided path reader.
    pub(crate) resource_key: String,
    /// Required nullable base Catalog key; nonnull only for a Profile requirement.
    pub(crate) resolved_catalog_key: Option<String>,
}

/// Exact local-only evidence declaration binding; URI evidence has no companion target.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalEvidenceBinding {
    /// Native evidence declaration key, reconciled with the full native graph later.
    pub(crate) evidence_key: String,
    /// Exact local-evidence companion key; repeated physical originals are not recaptured.
    pub(crate) resource_key: String,
}

/// Declared complete linkage relation; native path/hash/identity equality remains unproven here.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkageBindings {
    /// Ordered native requirement-key relations, at most the native 64-resource limit.
    pub(crate) requirements: Vec<RequirementBinding>,
    /// Exact base Component Definition or SSP implementation key.
    pub(crate) implementation_resource_key: String,
    /// Ordered local evidence-key relations; native URI members are not remapped here.
    pub(crate) local_evidence: Vec<LocalEvidenceBinding>,
}

/// Five closed MCP declaration kinds; they do not change fifteen protected workspace roles.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Companion {
    /// Pinned native linkage input with an explicit recorded-source dependency and joins.
    LinkageManifest {
        /// Unique complete-roster key.
        key: String,
        /// Exact portable contained file spelling.
        path: String,
        /// Raw input pin, never a normalized manifest digest.
        expected_sha256: String,
        /// Inert approved-label assertion, not content or URI disclosure.
        citation_label: String,
        /// Exact base `LifecycleRecord` key; no approval is inferred from this reference.
        lifecycle_key: String,
        /// Complete declared relation, subsequently checked against actual native originals.
        bindings: LinkageBindings,
    },
    /// Opaque evidence original or typed absence; changed bytes remain an observation later.
    LocalEvidence {
        /// Unique complete-roster key.
        key: String,
        /// Exact portable contained file spelling.
        path: String,
        /// Asserted raw evidence pin; a mismatch does not become missing evidence.
        expected_sha256: String,
        /// Asserted size at most ten MiB; native tighter limits still apply during capture.
        expected_size: u64,
        /// Inert citation token only, never evidence bytes or a sensitive title.
        citation_label: String,
    },
    /// Canonical inert report bytes; this declaration does not assert currentness.
    StaticReport {
        /// Unique complete-roster key.
        key: String,
        /// Exact portable contained report path.
        path: String,
        /// Exact raw report pin supplied through the selected declaration.
        expected_sha256: String,
        /// Inert citation token, not HTML or report prose.
        citation_label: String,
        /// Fixed `forge.workspace-report/1` decoder selection.
        report_format: String,
        /// Exact workspace-index-metadata companion key used for complete later comparison.
        workspace_index_key: String,
    },
    /// The existing closed index /1 or /2, without Snapshot creation or workspace mutation.
    WorkspaceIndexMetadata {
        /// Unique complete-roster key.
        key: String,
        /// Exactly `forge.workspace.json`, never a generic file selector.
        path: String,
        /// Raw index pin; actual registration/path relations remain separately checked.
        expected_sha256: String,
        /// Inert citation token only.
        citation_label: String,
    },
    /// Contained index target without a self-referential expected hash in discovery.
    SearchIndex {
        /// Unique complete-roster key selected explicitly by the visibility profile.
        key: String,
        /// Exact portable contained index target; actual absence/presence is not decoded here.
        path: String,
        /// Inert citation token only, not the index vocabulary.
        citation_label: String,
        /// Fixed `forge.mcp-search-index/1` artifact selector.
        index_format: String,
    },
}

impl Companion {
    /// Borrow exact opaque membership without copying another complete roster.
    pub(crate) fn key(&self) -> &str {
        match self {
            Self::LinkageManifest { key, .. }
            | Self::LocalEvidence { key, .. }
            | Self::StaticReport { key, .. }
            | Self::WorkspaceIndexMetadata { key, .. }
            | Self::SearchIndex { key, .. } => key,
        }
    }
    /// Borrow the original portable path assertion; this accessor performs no resolution.
    pub(crate) fn path(&self) -> &str {
        match self {
            Self::LinkageManifest { path, .. }
            | Self::LocalEvidence { path, .. }
            | Self::StaticReport { path, .. }
            | Self::WorkspaceIndexMetadata { path, .. }
            | Self::SearchIndex { path, .. } => path,
        }
    }
    /// Borrow a bounded inert label, separate from all source and evidence contents.
    fn label(&self) -> &str {
        match self {
            Self::LinkageManifest { citation_label, .. }
            | Self::LocalEvidence { citation_label, .. }
            | Self::StaticReport { citation_label, .. }
            | Self::WorkspaceIndexMetadata { citation_label, .. }
            | Self::SearchIndex { citation_label, .. } => citation_label,
        }
    }
}

/// Closed dynamic metadata families; empty selection permits no project resource family.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ResourceFamily {
    /// Minimized selected project inventory metadata.
    Inventory,
    /// Recorded/current status under independently captured approval closure.
    ArtifactStatus,
    /// Exact-ID native requirement metadata.
    RequirementMetadata,
    /// Complete admitted linkage/evidence metadata, never evidence bytes.
    ImplementationEvidence,
    /// Opaque minimized report metadata, never public HTML.
    StaticReport,
}

/// Complete /2 visibility assertion; schema-consumed constants preserve default privacy.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct VisibilityV2 {
    /// Exact `forge.mcp-visibility/2` selector.
    pub(crate) schema_version: String,
    /// Same opaque project token as the complete discovery declaration.
    pub(crate) project_key: String,
    /// Exact raw discovery digest, without JSON normalization.
    pub(crate) discovery_sha256: String,
    /// Existing seven tool names; no new tool is implied by companion admission.
    pub(crate) enabled_tools: Vec<String>,
    /// Exact base resource keys visible under the original selector meaning.
    pub(crate) visible_resource_keys: Vec<String>,
    /// Exact base keys for minimized noncurrent availability metadata.
    pub(crate) noncurrent_metadata_keys: Vec<String>,
    /// Explicit visible source keys for existing exact-span opt-in only.
    pub(crate) source_text_keys: Vec<String>,
    /// Explicit visible status keys with complete declared lifecycle dependencies.
    pub(crate) schedule_keys: Vec<String>,
    /// Exact linkage-manifest companion keys, never byte permissions.
    pub(crate) evidence_metadata_keys: Vec<String>,
    /// Exact static-report companion keys, never HTML permissions.
    pub(crate) static_report_keys: Vec<String>,
    /// Required nullable search-index companion key; nonnull for offline build intent.
    pub(crate) search_index_key: Option<String>,
    /// Fixed bounded-token-only identity disclosure.
    pub(crate) identity_disclosure: String,
    /// Fixed caller-established-recorded-policy label, not proof of caller selection.
    pub(crate) approved_current_basis: String,
    /// Existing omit or exact-span-opt-in mode, paired with source selectors.
    pub(crate) source_text_mode: String,
    /// Existing omit or explicit-as-of schedule mode.
    pub(crate) schedule_mode: String,
    /// Fixed minimized-metadata report policy.
    pub(crate) report_content_mode: String,
    /// Required false; no evidence content opt-in exists in this profile.
    pub(crate) evidence_bytes: bool,
    /// Required false; no remote evidence or URI fetch is authorized.
    pub(crate) network: bool,
    /// Required false; no command, plugin or builder is dispatched by the server.
    pub(crate) process_execution: bool,
    /// Required nullable explicit calendar date, never sampled from the clock.
    pub(crate) as_of: Option<String>,
    /// Unique explicit dynamic resource families; static shipped descriptors are separate.
    pub(crate) resource_families: Vec<ResourceFamily>,
    /// Explicit refuse or direct-search policy for a later fully validated selected index.
    pub(crate) index_fallback_mode: String,
}

/// Recorded provenance assertion reused structurally, never an authenticated owner decision.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyDecision {
    /// Closed purpose selector paired exactly with its declared role.
    pub(crate) subject: String,
    /// Product, security or engineering assertion, never an authenticated principal.
    pub(crate) role: String,
    /// Required approved assertion at semantic admission; it does not promote domain state.
    pub(crate) disposition: String,
    /// Required nullable declared owner key; approved rows require a real recorded token.
    pub(crate) declared_owner_key: Option<String>,
    /// Required nullable explicit calendar date; approved rows require it.
    pub(crate) recorded_on: Option<String>,
    /// Required nullable provenance record token, not an arbitrary source path.
    pub(crate) source_record_key: Option<String>,
    /// Required nullable raw provenance-record digest, not actual byte capture here.
    pub(crate) source_record_sha256: Option<String>,
    /// Exact reviewed engineering contract digest.
    pub(crate) proposal_index_sha256: String,
    /// Exact selected complete raw visibility digest.
    pub(crate) profile_sha256: String,
    /// Exact selected complete raw discovery digest.
    pub(crate) discovery_sha256: String,
}

/// Recorded evaluation-plan assertion; all acceptance fields remain explicitly unearned.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvaluationPlan {
    /// Pending or plan-selected declaration, not client/corpus acceptance.
    pub(crate) state: String,
    /// Required nullable declared owner token.
    pub(crate) declared_owner_key: Option<String>,
    /// Required nullable explicit recorded calendar date.
    pub(crate) recorded_on: Option<String>,
    /// Required nullable recorded provenance token, not a path reader.
    pub(crate) source_record_key: Option<String>,
    /// Required nullable raw provenance digest.
    pub(crate) source_record_sha256: Option<String>,
    /// Fixed unearned corpus acceptance.
    pub(crate) corpus_acceptance: String,
    /// Fixed unearned threshold acceptance.
    pub(crate) threshold_acceptance: String,
    /// Fixed unearned client acceptance.
    pub(crate) client_acceptance: String,
}

/// Final-read /2 decision declaration, with its noncircular external expected index pin.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DisclosureDecisionV2 {
    /// Exact `forge.mcp-disclosure-decision/2` selector; build intent is not interchangeable.
    pub(crate) schema_version: String,
    /// Exact bounded recorded decision key.
    pub(crate) decision_key: String,
    /// Same complete project token as discovery and visibility.
    pub(crate) project_key: String,
    /// Raw complete discovery digest.
    pub(crate) discovery_sha256: String,
    /// Raw complete visibility digest.
    pub(crate) profile_sha256: String,
    /// Actual reviewed successor contract digest, distinct from the protected /1 constant.
    pub(crate) proposal_index_sha256: String,
    /// Exact modern protocol revision; no handshake or protocol is implemented here.
    pub(crate) protocol_revision: String,
    /// Fixed declared-owner trust-boundary label, not authentication.
    pub(crate) operator_boundary: String,
    /// Complete exact visibility value; required nulls and every mode are retained.
    pub(crate) visibility_profile: VisibilityV2,
    /// Six complete distinct /1-compatible read-scope recorded rows.
    pub(crate) policy_decisions: Vec<PolicyDecision>,
    /// Complete evaluation declaration with unearned acceptance.
    pub(crate) evaluation_plan: EvaluationPlan,
    /// Null iff no index selected; otherwise the separately caller-bound final raw index pin.
    pub(crate) search_index_sha256: Option<String>,
}

/// Full ordered roster asserted by an offline intent, exactly equal to intended discovery.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CaptureRoster {
    /// Original base declarations in authorial discovery order.
    pub(crate) resources: Vec<ResourcePolicy>,
    /// Original companion declarations in authorial discovery order, including destination.
    pub(crate) companion_resources: Vec<Companion>,
}

/// Asserted exact source byte span; actual UTF-8/line/native relation is a later capture check.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceSpan {
    /// Inclusive UTF-8 byte offset in the original source.
    pub(crate) start_byte: u64,
    /// Exclusive UTF-8 byte offset, with start strictly before end.
    pub(crate) end_byte: u64,
}

/// Asserted search tuple mirroring maintained requirement/citation facts, without a proof factory.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SearchSourceTuple {
    /// Exact visible Catalog or Component Definition base key.
    pub(crate) artifact_key: String,
    /// Exact existing native role, equal to that base declaration.
    pub(crate) artifact_role: Role,
    /// Complete exact native identity assertion, equal to that base declaration.
    pub(crate) artifact_identity: NativeIdentity,
    /// Complete raw artifact digest, equal to its original declaration pin.
    pub(crate) artifact_raw_sha256: String,
    /// Original control ID or implemented-requirement UUID spelling, not normalized here.
    pub(crate) native_id: String,
    /// Original control ID used by the maintained search relationship.
    pub(crate) control_id: String,
    /// Exact declared native JSON pointer; no pointer is evaluated at admission.
    pub(crate) native_pointer: String,
    /// Exact base `PolicySource` or `LifecycleSource` relation, not a filename guess.
    pub(crate) source_key: String,
    /// Exact raw source digest, equal to that base declaration pin.
    pub(crate) source_raw_sha256: String,
    /// Asserted bounded byte span; actual captured source relation remains mandatory.
    pub(crate) source_span: SourceSpan,
    /// Exact shared lifecycle dependency of both the declared native and source.
    pub(crate) lifecycle_key: String,
}

/// Explicit selected contained output assertion, not permission to create or overwrite it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexDestination {
    /// Exact nonnull visibility-selected search-index companion key.
    pub(crate) key: String,
    /// Exact same declared portable contained path; later capture must prove absence.
    pub(crate) path: String,
    /// Fixed `forge.mcp-search-index/1` artifact format.
    pub(crate) index_format: String,
    /// Explicit bounded tokenizer/build identity, later compared with actual builder semantics.
    pub(crate) tokenizer_identity: String,
}

/// Inert offline build-intent record, deliberately separate from a final-read decision.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexBuildIntent {
    /// Exact `forge.mcp-index-build-intent/1` selector; no final decision substitution.
    pub(crate) schema_version: String,
    /// Bounded recorded intent key.
    pub(crate) intent_key: String,
    /// Exact intended finalized project token.
    pub(crate) project_key: String,
    /// Raw intended finalized discovery digest.
    pub(crate) discovery_sha256: String,
    /// Raw intended finalized visibility digest; no temporary null-index profile.
    pub(crate) profile_sha256: String,
    /// Exact reviewed /2 engineering proposal digest.
    pub(crate) proposal_index_sha256: String,
    /// Fixed declared-owner label, not actual operator authentication.
    pub(crate) operator_boundary: String,
    /// Complete intended final visibility value, including the nonnull destination selector.
    pub(crate) visibility_profile: VisibilityV2,
    /// Complete ordered discovery roster, including the selected target whose absence is later proved.
    pub(crate) capture_roster: CaptureRoster,
    /// Complete asserted selected search tuples; sealed capture must independently recompute them.
    pub(crate) search_sources: Vec<SearchSourceTuple>,
    /// Exact selected contained output and tokenizer identity.
    pub(crate) destination: IndexDestination,
    /// Six distinct product/security build/source/index provenance assertions.
    pub(crate) policy_decisions: Vec<PolicyDecision>,
}

/// Structurally admitted server declarations, with no held-original or approval capability.
#[derive(Debug)]
pub(crate) struct InertServerDeclarations {
    /// Complete /2 project assertion for the later real capture constructor.
    pub(crate) project: ProjectDeclarationV2,
    /// Complete visibility assertion, not an installed policy.
    pub(crate) profile: VisibilityV2,
    /// Complete final-read declaration; caller authority remains separate.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Retain the entire validated external decision in the inert codec result; later native authority still comes from its separately held complete original owner."
        )
    )]
    pub(crate) decision: DisclosureDecisionV2,
}

/// Structurally admitted offline intent, with no conversion to any captured/server scope.
#[derive(Debug)]
pub(crate) struct InertBuildDeclarations {
    /// Same unchanged intended finalized discovery assertion.
    pub(crate) project: ProjectDeclarationV2,
    /// Same unchanged intended finalized visibility assertion.
    pub(crate) profile: VisibilityV2,
    /// Inert recorded intent; cannot supply original proof or current approval.
    pub(crate) intent: IndexBuildIntent,
}

/// An inert work callback borrowing the caller's existing admission owner and control.
///
/// Implementations retain the original first Capacity or actual control failure across
/// these two ports, earlier capture work, failed phases and later native/final work.
/// They must not renew an allowance or deadline. Ordinary codec policy/schema/typed
/// errors are not sticky control failures. This trait creates no original/native proof.
pub(crate) trait DeclarationAdmission {
    /// Charge the complete logical descriptor before corresponding work or growth.
    /// Return the original first stop without inspecting a later control after it.
    fn charge(&mut self, amount: usize) -> WorkResult<()>;

    /// Fence the same actual caller control, including its retained interruption.
    /// Latch only an actual control failure; an already latched stop is preserved.
    fn checkpoint(&mut self, stage: Stage) -> WorkResult<()>;

    /// Latch a real codec bound/checked-arithmetic failure in the original ledger.
    /// Return the original first stop when one already exists. No allowance is added.
    fn capacity(&mut self) -> WorkError;
}

/// Complete typed operand with an actual parsed-tree descriptor, never an authority token.
struct DecodedConfig<T> {
    /// Ordinary typed declaration data; native/capture/owner semantics remain absent.
    value: T,
    /// Complete nodes, object fields and string byte units measured before typed growth.
    operand_work: usize,
}

/// Decode complete final-read declarations under the caller's existing admission owner.
///
/// Expected pins establish raw integrity only. On every ordinary failure the same
/// original final checkpoint is reached before propagation. Results contain no ledger.
pub(crate) fn decode_server_declarations(
    discovery: &[u8],
    profile: &[u8],
    decision: &[u8],
    expected_decision: &str,
    expected_profile: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<InertServerDeclarations> {
    admission.checkpoint(Stage::ReadIndex)?;
    let ordinary = decode_server_inner(
        discovery,
        profile,
        decision,
        expected_decision,
        expected_profile,
        admission,
    );
    admission.checkpoint(Stage::RetainPrepared)?;
    ordinary
}

/// Keep the whole ordinary final-read result until its caller-owned final fence.
fn decode_server_inner(
    discovery: &[u8],
    profile: &[u8],
    decision: &[u8],
    expected_decision: &str,
    expected_profile: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<InertServerDeclarations> {
    verify_pin(decision, expected_decision, admission)?;
    verify_pin(profile, expected_profile, admission)?;
    let project: DecodedConfig<ProjectDeclarationV2> =
        decode_config(discovery, include_str!("discovery-v2.schema.json"), admission)?;
    let visibility: DecodedConfig<VisibilityV2> =
        decode_config(profile, include_str!("profile-v2.schema.json"), admission)?;
    let record: DecodedConfig<DisclosureDecisionV2> =
        decode_config(decision, include_str!("decision-v2.schema.json"), admission)?;
    let discovery_pin = bounded_hash(discovery, admission)?;
    let scope_work = add_work(project.operand_work, visibility.operand_work, admission)?;
    run_phase(scope_work, Stage::ValidateResource, admission, |admission| {
        validate_scope(&project.value, &visibility.value, &discovery_pin, admission)
    })?;
    let all_work = add_work(scope_work, record.operand_work, admission)?;
    run_phase(all_work, Stage::ValidateResource, admission, |admission| {
        let record = &record.value;
        if record.project_key != project.value.project_key
            || record.discovery_sha256 != discovery_pin
            || record.profile_sha256 != expected_profile
            || record.visibility_profile != visibility.value
        {
            return Err(policy_failure());
        }
        validate_rows(
            &record.policy_decisions,
            &READ_PAIRS,
            &discovery_pin,
            expected_profile,
            admission,
        )?;
        if let Some(date) = &record.evaluation_plan.recorded_on {
            calendar_date(date)?;
        }
        if visibility.value.search_index_key.is_some() != record.search_index_sha256.is_some() {
            return Err(policy_failure());
        }
        Ok(())
    })?;
    Ok(InertServerDeclarations {
        project: project.value,
        profile: visibility.value,
        decision: record.value,
    })
}

/// Decode the distinct offline intent using the same existing admission owner.
/// No final decision, temporary null profile, original proof or server gate is made.
pub(crate) fn decode_build_intent(
    discovery: &[u8],
    profile: &[u8],
    intent: &[u8],
    expected_intent: &str,
    expected_profile: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<InertBuildDeclarations> {
    admission.checkpoint(Stage::ReadIndex)?;
    let ordinary = decode_build_inner(
        discovery,
        profile,
        intent,
        expected_intent,
        expected_profile,
        admission,
    );
    admission.checkpoint(Stage::RetainPrepared)?;
    ordinary
}

/// Preserve the complete ordinary build-intent result until its original final fence.
fn decode_build_inner(
    discovery: &[u8],
    profile: &[u8],
    intent: &[u8],
    expected_intent: &str,
    expected_profile: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<InertBuildDeclarations> {
    verify_pin(intent, expected_intent, admission)?;
    verify_pin(profile, expected_profile, admission)?;
    let project: DecodedConfig<ProjectDeclarationV2> =
        decode_config(discovery, include_str!("discovery-v2.schema.json"), admission)?;
    let visibility: DecodedConfig<VisibilityV2> =
        decode_config(profile, include_str!("profile-v2.schema.json"), admission)?;
    let record: DecodedConfig<IndexBuildIntent> =
        decode_config(intent, include_str!("index-build-intent.schema.json"), admission)?;
    let discovery_pin = bounded_hash(discovery, admission)?;
    let scope_work = add_work(project.operand_work, visibility.operand_work, admission)?;
    run_phase(scope_work, Stage::ValidateResource, admission, |admission| {
        validate_scope(&project.value, &visibility.value, &discovery_pin, admission)
    })?;
    let all_work = add_work(scope_work, record.operand_work, admission)?;
    run_phase(all_work, Stage::ValidateResource, admission, |admission| {
        validate_intent(
            &project.value,
            &visibility.value,
            &record.value,
            &discovery_pin,
            expected_profile,
            admission,
        )
    })?;
    Ok(InertBuildDeclarations {
        project: project.value,
        profile: visibility.value,
        intent: record.value,
    })
}

/// Pre-admit a phase, retain its entire ordinary result, then fence before propagating it.
/// An original adapter stop outranks ordinary policy errors or later control failures.
fn run_phase<T>(
    amount: usize,
    stage: Stage,
    admission: &mut dyn DeclarationAdmission,
    ordinary: impl FnOnce(&mut dyn DeclarationAdmission) -> WorkResult<T>,
) -> WorkResult<T> {
    admission.checkpoint(stage)?;
    let admitted = admission.charge(amount);
    if let Err(error) = admitted {
        admission.checkpoint(stage)?;
        return Err(error);
    }
    let result = ordinary(admission);
    admission.checkpoint(stage)?;
    result
}

/// Count complete borrowed byte traversals without allocating or overflowing a rounded length.
fn byte_work(bytes: usize) -> usize {
    bytes.div_ceil(WORK_BYTES)
}

/// Checked complete descriptor arithmetic records capacity in the original owner.
fn add_work(
    left: usize,
    right: usize,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<usize> {
    left.checked_add(right).ok_or_else(|| admission.capacity())
}

/// Admit raw size and canonical expected pin before each actual bounded original hash pass.
fn verify_pin(
    bytes: &[u8],
    expected: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    run_phase(1, Stage::ReadIndex, admission, |admission| {
        if bytes.len() > CONFIG_BYTES {
            return Err(admission.capacity());
        }
        if !valid_hash(expected) {
            return Err(policy_failure());
        }
        Ok(())
    })?;
    let actual = bounded_hash(bytes, admission)?;
    run_phase(1, Stage::ReadIndex, admission, |_| {
        if actual == expected { Ok(()) } else { Err(policy_failure()) }
    })
}

/// Hash one bounded original in borrowed 32-KiB chunks under complete pass admission.
/// Maintained `lower_hex` renders the exact 64-character digest; no Value is serialized.
fn bounded_hash(bytes: &[u8], admission: &mut dyn DeclarationAdmission) -> WorkResult<String> {
    run_phase(1, Stage::ReadIndex, admission, |admission| {
        if bytes.len() > CONFIG_BYTES { Err(admission.capacity()) } else { Ok(()) }
    })?;
    let work = add_work(byte_work(bytes.len()), 1, admission)?;
    run_phase(work, Stage::ReadIndex, admission, |admission| {
        let mut digest = Sha256::new();
        for chunk in bytes.chunks(WORK_BYTES) {
            admission.checkpoint(Stage::ReadIndex)?;
            digest.update(chunk);
            admission.checkpoint(Stage::ReadIndex)?;
        }
        Ok(lower_hex(&digest.finalize()))
    })
}

/// Scan complete raw syntax bounds before strict Value allocation, without decoding data.
/// Quote-aware separators plus one bound valid tree nodes/fields. Actual container nesting
/// is bounded by 65 (root-inclusive counterpart to maintained depth 64); string UTF-8
/// upper bounds use three bytes per escaped Unicode code unit, including surrogate halves.
/// That conservative string envelope can refuse earlier than the decoded string limit.
/// Malformed syntax still belongs to the maintained strict parser; no validity is issued.
fn preparse_work(bytes: &[u8], admission: &mut dyn DeclarationAdmission) -> WorkResult<usize> {
    run_phase(1, Stage::ReadIndex, admission, |admission| {
        if bytes.len() > CONFIG_BYTES { Err(admission.capacity()) } else { Ok(()) }
    })?;
    let scan_work = add_work(byte_work(bytes.len()), 1, admission)?;
    run_phase(scan_work, Stage::ReadIndex, admission, |admission| {
        let mut lexical = LexicalWork::new();
        for chunk in bytes.chunks(WORK_BYTES) {
            admission.checkpoint(Stage::ReadIndex)?;
            for byte in chunk {
                lexical.visit(*byte, admission)?;
            }
            admission.checkpoint(Stage::ReadIndex)?;
        }
        add_work(lexical.separators, 1, admission)
    })
}

/// Bounded scalar state of the complete borrowed preallocation scan, not parsed data.
struct LexicalWork {
    /// Whether the current raw byte occurs inside a quoted string.
    quoted: bool,
    /// Whether an unescaped backslash introduced the next raw escape character.
    escaped: bool,
    /// Remaining four hexadecimal-byte positions after a Unicode escape marker.
    unicode_remaining: usize,
    /// Conservative decoded UTF-8 size for the current complete quoted operand.
    string_upper: usize,
    /// Open-container count; mismatched closing syntax is left to strict parsing.
    containers: usize,
    /// Complete unquoted opening/comma/colon occurrences before Value allocation.
    separators: usize,
}

impl LexicalWork {
    /// Start scalar lexical accounting without building a token/tree/string buffer.
    fn new() -> Self {
        Self {
            quoted: false,
            escaped: false,
            unicode_remaining: 0,
            string_upper: 0,
            containers: 0,
            separators: 0,
        }
    }

    /// Inspect one pre-admitted byte and refuse growth outside the strict envelope.
    fn visit(&mut self, byte: u8, admission: &mut dyn DeclarationAdmission) -> WorkResult<()> {
        if self.quoted {
            if self.unicode_remaining != 0 {
                self.unicode_remaining -= 1;
            } else if self.escaped {
                self.escaped = false;
                let width = if byte == b'u' {
                    self.unicode_remaining = 4;
                    3
                } else {
                    1
                };
                self.string_upper = add_work(self.string_upper, width, admission)?;
            } else if byte == b'\\' {
                self.escaped = true;
            } else if byte == b'"' {
                self.quoted = false;
            } else {
                self.string_upper = add_work(self.string_upper, 1, admission)?;
            }
            if self.string_upper > MAX_STRING_BYTES {
                return Err(policy_failure());
            }
        } else if byte == b'"' {
            self.quoted = true;
            self.string_upper = 0;
        } else {
            if matches!(byte, b'[' | b'{') {
                self.containers = add_work(self.containers, 1, admission)?;
                if self.containers > 65 {
                    return Err(policy_failure());
                }
            } else if matches!(byte, b']' | b'}') {
                self.containers = self.containers.saturating_sub(1);
            }
            if matches!(byte, b'[' | b'{' | b',' | b':') {
                self.separators = add_work(self.separators, 1, admission)?;
            }
        }
        Ok(())
    }
}

/// Charge complete lexical tree/scratch work before the maintained strict byte consumer.
fn parse_config(bytes: &[u8], admission: &mut dyn DeclarationAdmission) -> WorkResult<Value> {
    let tree_upper = preparse_work(bytes, admission)?;
    let complete = add_work(tree_upper, byte_work(bytes.len()), admission)?;
    run_phase(complete, Stage::ReadIndex, admission, |_| strict_value(bytes, CONFIG_BYTES))
}

/// Consume strict schema, compiled schema, complete validation and typed phases separately.
/// Every whole ordinary result is held until the same original post-phase checkpoint.
fn decode_config<T: DeserializeOwned>(
    bytes: &[u8],
    schema: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<DecodedConfig<T>> {
    let value = parse_config(bytes, admission)?;
    let value_work = charge_tree(&value, admission)?;
    let schema_value = parse_config(schema.as_bytes(), admission)?;
    let schema_work = charge_tree(&schema_value, admission)?;
    let validator = run_phase(schema_work, Stage::ValidateResource, admission, |_| {
        jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .build(&schema_value)
            .map_err(|_| policy_failure())
    })?;
    let validation_work = add_work(schema_work, value_work, admission)?;
    run_phase(validation_work, Stage::ValidateResource, admission, |_| {
        if validator.is_valid(&value) { Ok(()) } else { Err(policy_failure()) }
    })?;
    let value = run_phase(value_work, Stage::PrepareDomain, admission, |_| {
        serde_json::from_value(value).map_err(|_| policy_failure())
    })?;
    Ok(DecodedConfig { value, operand_work: value_work })
}

/// Walk every complete actual node/property/string before typed registries or joins.
/// Per-entry work is charged before inspecting the corresponding bounded child.
fn charge_tree(value: &Value, admission: &mut dyn DeclarationAdmission) -> WorkResult<usize> {
    admission.checkpoint(Stage::ValidateResource)?;
    let ordinary = tree_work(value, admission);
    admission.checkpoint(Stage::ValidateResource)?;
    ordinary
}

/// Accumulate an actual parsed-tree descriptor while charging all inspected occurrences.
fn tree_work(value: &Value, admission: &mut dyn DeclarationAdmission) -> WorkResult<usize> {
    admission.charge(1)?;
    let mut work = 1;
    match value {
        Value::Array(values) => {
            for value in values {
                work = add_work(work, tree_work(value, admission)?, admission)?;
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                let field = add_work(1, byte_work(key.len()), admission)?;
                admission.charge(field)?;
                work = add_work(work, field, admission)?;
                work = add_work(work, tree_work(value, admission)?, admission)?;
            }
        }
        Value::String(value) => {
            let bytes = byte_work(value.len());
            admission.charge(bytes)?;
            work = add_work(work, bytes, admission)?;
        }
        _ => {}
    }
    Ok(work)
}

/// Validate complete roster and all selector kinds; no selected subset bypasses hidden declarations.
fn validate_scope(
    project: &ProjectDeclarationV2,
    profile: &VisibilityV2,
    discovery_pin: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    let count = project
        .resources
        .len()
        .checked_add(project.companion_resources.len())
        .ok_or_else(|| admission.capacity())?;
    if count > ROSTER_LIMIT {
        return Err(admission.capacity());
    }
    if profile.project_key != project.project_key || profile.discovery_sha256 != discovery_pin {
        return Err(policy_failure());
    }
    validate_roster(project, admission)?;
    for key in profile.visible_resource_keys.iter().chain(&profile.noncurrent_metadata_keys) {
        base(project, key, admission)?;
    }
    for key in &profile.source_text_keys {
        let source = base(project, key, admission)?;
        admit_string_comparisons(
            key,
            profile.visible_resource_keys.iter().map(String::as_str),
            admission,
        )?;
        if !profile.visible_resource_keys.contains(key)
            || !matches!(source.role, Role::PolicySource | Role::LifecycleSource)
        {
            return Err(policy_failure());
        }
    }
    for key in &profile.schedule_keys {
        let resource = base(project, key, admission)?;
        admit_string_comparisons(
            key,
            profile.visible_resource_keys.iter().map(String::as_str),
            admission,
        )?;
        if !profile.visible_resource_keys.contains(key)
            || !resource.role.status_supported()
            || resource.lifecycle_key.is_none()
        {
            return Err(policy_failure());
        }
    }
    if profile.source_text_mode == "omit" && !profile.source_text_keys.is_empty() {
        return Err(policy_failure());
    }
    if let Some(date) = &profile.as_of {
        calendar_date(date)?;
    }
    for key in &profile.evidence_metadata_keys {
        if !matches!(companion(project, key, admission)?, Companion::LinkageManifest { .. }) {
            return Err(policy_failure());
        }
    }
    for key in &profile.static_report_keys {
        if !matches!(companion(project, key, admission)?, Companion::StaticReport { .. }) {
            return Err(policy_failure());
        }
    }
    if let Some(key) = &profile.search_index_key {
        if !matches!(companion(project, key, admission)?, Companion::SearchIndex { .. }) {
            return Err(policy_failure());
        }
    }
    Ok(())
}

/// Admit all key/label/path identities together before validating any foreign-key relation.
fn validate_roster(
    project: &ProjectDeclarationV2,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    let mut identities = IdentityRows::new();
    for resource in &project.resources {
        if !valid_native_policy(resource) {
            return Err(policy_failure());
        }
        identities.admit(&resource.key, &resource.citation_label, &resource.path, admission)?;
    }
    for resource in &project.companion_resources {
        identities.admit(resource.key(), resource.label(), resource.path(), admission)?;
    }
    for resource in &project.resources {
        if let Some(key) = &resource.lifecycle_key {
            if base(project, key, admission)?.role != Role::LifecycleRecord {
                return Err(policy_failure());
            }
        }
    }
    for resource in &project.companion_resources {
        validate_companion(project, resource, admission)?;
    }
    Ok(())
}

/// Borrowed complete identity indexes shared across base and companion declarations.
struct IdentityRows<'a> {
    /// Unique exact opaque keys; no lookup alias is a second resource declaration.
    keys: BTreeSet<&'a str>,
    /// Unique inert citation labels across the same complete roster.
    labels: BTreeSet<&'a str>,
    /// All prior portable file paths, retained for folded equality/ancestor checks.
    paths: Vec<&'a str>,
}

impl<'a> IdentityRows<'a> {
    /// Begin one complete roster identity phase before any foreign-key join.
    fn new() -> Self {
        Self { keys: BTreeSet::new(), labels: BTreeSet::new(), paths: Vec::new() }
    }
    /// Charge complete comparisons before retaining a borrowed path/key/label row.
    fn admit(
        &mut self,
        key: &'a str,
        label: &'a str,
        path: &'a str,
        admission: &mut dyn DeclarationAdmission,
    ) -> WorkResult<()> {
        let identity_work = self.paths.len().checked_add(4).ok_or_else(|| admission.capacity())?;
        admission.charge(identity_work)?;
        let key_label = add_work(byte_work(key.len()), byte_work(label.len()), admission)?;
        let identity_bytes = add_work(key_label, byte_work(path.len()), admission)?;
        admission.charge(identity_bytes)?;
        admit_string_comparisons(key, self.keys.iter().copied(), admission)?;
        admit_string_comparisons(label, self.labels.iter().copied(), admission)?;
        admit_string_comparisons(path, PROJECT_CONFIGS.iter().copied(), admission)?;
        admit_string_comparisons(path, self.paths.iter().copied(), admission)?;
        if !safe_key(key)
            || !safe_key(label)
            || !portable_path(path)
            || PROJECT_CONFIGS.iter().any(|config| paths_overlap(config, path))
            || self.paths.iter().any(|prior| paths_overlap(prior, path))
            || !self.keys.insert(key)
            || !self.labels.insert(label)
        {
            return Err(policy_failure());
        }
        self.paths.push(path);
        Ok(())
    }
}

/// Enforce declared companion-specific kinds and nullable Profile companion relations.
fn validate_companion(
    project: &ProjectDeclarationV2,
    resource: &Companion,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    match resource {
        Companion::LinkageManifest { lifecycle_key, bindings, .. } => {
            if base(project, lifecycle_key, admission)?.role != Role::LifecycleRecord {
                return Err(policy_failure());
            }
            let implementation = base(project, &bindings.implementation_resource_key, admission)?;
            if !matches!(implementation.role, Role::OscalComponentArtifact | Role::OscalSspArtifact)
            {
                return Err(policy_failure());
            }
            validate_linkage_bindings(project, bindings, admission)?;
        }
        Companion::StaticReport { workspace_index_key, .. } => {
            if !matches!(
                companion(project, workspace_index_key, admission)?,
                Companion::WorkspaceIndexMetadata { .. }
            ) {
                return Err(policy_failure());
            }
        }
        Companion::LocalEvidence { .. }
        | Companion::WorkspaceIndexMetadata { .. }
        | Companion::SearchIndex { .. } => {}
    }
    Ok(())
}

/// Validate complete declared native-key joins without pretending to have inspected a native graph.
fn validate_linkage_bindings(
    project: &ProjectDeclarationV2,
    bindings: &LinkageBindings,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    let mut requirements = BTreeSet::new();
    for binding in &bindings.requirements {
        admission.charge(1)?;
        admit_string_comparisons(
            &binding.requirement_key,
            requirements.iter().copied(),
            admission,
        )?;
        if !requirements.insert(binding.requirement_key.as_str()) {
            return Err(policy_failure());
        }
        let resource = base(project, &binding.resource_key, admission)?;
        match (resource.role, &binding.resolved_catalog_key) {
            (Role::OscalCatalogArtifact, None) => {}
            (Role::OscalProfileArtifact, Some(key)) => {
                if base(project, key, admission)?.role != Role::OscalCatalogArtifact {
                    return Err(policy_failure());
                }
            }
            _ => return Err(policy_failure()),
        }
    }
    let mut evidence = BTreeSet::new();
    for binding in &bindings.local_evidence {
        admission.charge(1)?;
        admit_string_comparisons(&binding.evidence_key, evidence.iter().copied(), admission)?;
        if !evidence.insert(binding.evidence_key.as_str())
            || !matches!(
                companion(project, &binding.resource_key, admission)?,
                Companion::LocalEvidence { .. }
            )
        {
            return Err(policy_failure());
        }
    }
    Ok(())
}

/// Admit complete string operands and all candidate visits before a repeated lookup/growth.
/// Exact-size iterators provide the before-loop traversal denominator; no registry grows here.
fn admit_string_comparisons<'a>(
    needle: &str,
    candidates: impl ExactSizeIterator<Item = &'a str>,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    admission.charge(candidates.len())?;
    for candidate in candidates {
        let operands = add_work(byte_work(needle.len()), byte_work(candidate.len()), admission)?;
        let comparison = add_work(operands, 1, admission)?;
        admission.charge(comparison)?;
    }
    Ok(())
}

/// Resolve only an exact base foreign key under a before-join complete-work charge.
fn base<'a>(
    project: &'a ProjectDeclarationV2,
    key: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<&'a ResourcePolicy> {
    admission.charge(project.resources.len())?;
    admit_string_comparisons(
        key,
        project.resources.iter().map(|resource| resource.key.as_str()),
        admission,
    )?;
    project.resources.iter().find(|resource| resource.key == key).ok_or_else(policy_failure)
}

/// Resolve only an exact companion foreign key, never a path, fallback or mixed discriminator.
fn companion<'a>(
    project: &'a ProjectDeclarationV2,
    key: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<&'a Companion> {
    admission.charge(project.companion_resources.len())?;
    admit_string_comparisons(
        key,
        project.companion_resources.iter().map(Companion::key),
        admission,
    )?;
    project
        .companion_resources
        .iter()
        .find(|resource| resource.key() == key)
        .ok_or_else(policy_failure)
}

/// Require exact complete six-pair provenance and raw binding; assertions do not authenticate owners.
fn validate_rows(
    rows: &[PolicyDecision],
    expected: &[(&str, &str); 6],
    discovery: &str,
    profile: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    let mut pairs = BTreeSet::new();
    for row in rows {
        admission.charge(1)?;
        admit_string_comparisons(
            &row.subject,
            pairs.iter().map(|(subject, _)| *subject),
            admission,
        )?;
        admit_string_comparisons(&row.role, pairs.iter().map(|(_, role)| *role), admission)?;
        if row.disposition != "approved"
            || row.proposal_index_sha256 != PROPOSAL
            || row.discovery_sha256 != discovery
            || row.profile_sha256 != profile
            || !pairs.insert((row.subject.as_str(), row.role.as_str()))
            || row.declared_owner_key.as_deref().is_none_or(|key| !safe_key(key))
            || row.source_record_key.as_deref().is_none_or(|key| !safe_key(key))
            || row.source_record_sha256.as_deref().is_none_or(|hash| !valid_hash(hash))
        {
            return Err(policy_failure());
        }
        calendar_date(row.recorded_on.as_deref().ok_or_else(policy_failure)?)?;
    }
    for (subject, role) in expected {
        admit_string_comparisons(subject, pairs.iter().map(|(value, _)| *value), admission)?;
        admit_string_comparisons(role, pairs.iter().map(|(_, value)| *value), admission)?;
    }
    if pairs != BTreeSet::from(*expected) {
        return Err(policy_failure());
    }
    Ok(())
}

/// Compare exact intended configs/roster/destination and all asserted selected search tuples.
fn validate_intent(
    project: &ProjectDeclarationV2,
    profile: &VisibilityV2,
    intent: &IndexBuildIntent,
    discovery_pin: &str,
    profile_pin: &str,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    if intent.project_key != project.project_key
        || intent.discovery_sha256 != discovery_pin
        || intent.profile_sha256 != profile_pin
        || intent.visibility_profile != *profile
        || intent.capture_roster.resources != project.resources
        || intent.capture_roster.companion_resources != project.companion_resources
        || profile.search_index_key.as_deref() != Some(intent.destination.key.as_str())
    {
        return Err(policy_failure());
    }
    let selected = companion(project, &intent.destination.key, admission)?;
    let Companion::SearchIndex { path, index_format, .. } = selected else {
        return Err(policy_failure());
    };
    if path != &intent.destination.path || index_format != &intent.destination.index_format {
        return Err(policy_failure());
    }
    validate_rows(&intent.policy_decisions, &BUILD_PAIRS, discovery_pin, profile_pin, admission)?;
    let mut identities = BTreeSet::new();
    for source in &intent.search_sources {
        admission.charge(1)?;
        admit_string_comparisons(
            &source.artifact_key,
            identities.iter().map(|(key, _)| *key),
            admission,
        )?;
        admit_string_comparisons(
            &source.native_id,
            identities.iter().map(|(_, id)| *id),
            admission,
        )?;
        if !identities.insert((source.artifact_key.as_str(), source.native_id.as_str())) {
            return Err(policy_failure());
        }
        validate_search_tuple(project, profile, source, admission)?;
    }
    Ok(())
}

/// Bind tuple declarations to exact base identities and raw source pins without recomputation claims.
fn validate_search_tuple(
    project: &ProjectDeclarationV2,
    profile: &VisibilityV2,
    tuple: &SearchSourceTuple,
    admission: &mut dyn DeclarationAdmission,
) -> WorkResult<()> {
    let artifact = base(project, &tuple.artifact_key, admission)?;
    let source = base(project, &tuple.source_key, admission)?;
    admit_string_comparisons(
        &tuple.artifact_key,
        profile.visible_resource_keys.iter().map(String::as_str),
        admission,
    )?;
    if !profile.visible_resource_keys.contains(&tuple.artifact_key)
        || !matches!(artifact.role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact)
        || artifact.role != tuple.artifact_role
        || artifact.native_identity.as_ref() != Some(&tuple.artifact_identity)
        || artifact.expected_sha256 != tuple.artifact_raw_sha256
        || !matches!(source.role, Role::PolicySource | Role::LifecycleSource)
        || source.expected_sha256 != tuple.source_raw_sha256
        || artifact.lifecycle_key.as_deref() != Some(tuple.lifecycle_key.as_str())
        || source.lifecycle_key.as_deref() != Some(tuple.lifecycle_key.as_str())
        || tuple.source_span.start_byte >= tuple.source_span.end_byte
        || tuple.native_id.len() > 4096
        || tuple.control_id.len() > 4096
        || tuple.native_pointer.len() > 4096
        || !tuple.native_pointer.starts_with('/')
        || (artifact.role == Role::OscalCatalogArtifact && tuple.native_id != tuple.control_id)
        || (artifact.role == Role::OscalComponentArtifact
            && uuid::Uuid::parse_str(&tuple.native_id).is_err())
    {
        return Err(policy_failure());
    }
    Ok(())
}

/// Consume the same exact ten-byte explicit Gregorian date rule, without a system clock.
fn calendar_date(value: &str) -> WorkResult<NaiveDate> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return Err(policy_failure());
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| policy_failure())
}

/// Proposed pure byte/shape controls; they do not fabricate files, captures, or owner approval.
#[cfg(test)]
#[path = "declarations_v2_tests.rs"]
mod tests;

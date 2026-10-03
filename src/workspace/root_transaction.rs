//! Proposed sealed multi-target transaction port and exact root-bound journal facts.
//!
//! This uncompiled TEMP candidate never loops the existing single-file commit.
//! ROOT integrates Store acceptance, before-listener recovery and participating
//! workspace IO leases. CLI/editors remain external absent a real shared barrier.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

use super::super::contract::{Error, Result};
use super::super::index::{INDEX_PATH, validate_path};
use super::super::preparation::{ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};
use super::transaction_state::{TransactionState, recovery_required, unavailable, unix_seconds};
use super::{Captured, Root};

/// Physical current/proposed/index union admitted before supplied target reads.
const MAX_PATHS: usize = 100;
/// Complete generations, not a per-file or same-path-deduplicated allowance.
const MAX_CAPTURE_BYTES: usize = 50 * 1024 * 1024;
/// Shared complete retained pool, including planned journal/stage/backup storage.
const MAX_RETAINED_BYTES: usize = 20 * 1024 * 1024;
/// Conservative bounded metadata reserve within the same retained pool.
const JOURNAL_RESERVE: usize = 4 * 1024 * 1024;
/// One shared byte budget for all visible UTF8 diff strings.
const MAX_DIFF_BYTES: usize = 200_000;
/// Separate cooperative recovery/rollback attempt; never renews forward authority.
const SETTLEMENT_SECONDS: u64 = 30;

/// Actual held project identity and opaque binding; no public journal path authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RootIdentity {
    /// Native filesystem identity obtained from the held root directory.
    pub(super) volume: u64,
    /// Native directory object identity, not a path-supplied surrogate.
    pub(super) object: u64,
    /// Domain-separated hash of actual root identity and original canonical spelling.
    pub(super) binding_sha256: String,
}

/// Exact single-link object facts; proposed bytes never fabricate one of these.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFacts {
    /// Native volume identity from actual observed metadata.
    volume: u64,
    /// Actual inode/file index observed through the held object.
    object: u64,
    /// Exact bounded byte hash; no document normalization.
    sha256: String,
    /// Complete observed byte length.
    size: usize,
}

/// Durable per-target metadata; source byte vectors never enter the journal.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalTarget {
    /// Validated root-relative public destination, in authorial order with index last.
    path: String,
    /// Original current object facts, null only after verified original absence.
    before: Option<ObjectFacts>,
    /// Complete supplied/proposed bytes' SHA256, not a synthetic object identity.
    proposed_sha256: String,
    /// Exact proposed byte length admitted before acceptance.
    proposed_size: usize,
    /// Actual nearest existing parent identity at planning.
    nearest_parent: (u64, u64),
    /// Explicit missing parent paths, parent first; no implicit mkdir.
    missing_parents: Vec<String>,
    /// Predetermined private auxiliary directory name within the actual target parent.
    auxiliary_name: String,
    /// Actual created auxiliary identity, null until verified and journaled.
    auxiliary_identity: Option<(u64, u64)>,
    /// Actual proposed staging inode/hash/length, null before verified staging.
    staged: Option<ObjectFacts>,
    /// Actual original inode after atomic exchange into the private stage slot.
    backup: Option<ObjectFacts>,
    /// Exact owned object observed after publication; null before confirmed publication.
    published: Option<ObjectFacts>,
}

/// Exact explicitly planned new directory and its accepted ownership observations.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalDirectory {
    /// Valid public parent path; no absolute or journal-selected filesystem input.
    path: String,
    /// Actual nearest existing parent from complete pre-effect planning.
    nearest_parent: (u64, u64),
    /// Actual identity after exclusive creation, never inferred from path existence.
    created_identity: Option<(u64, u64)>,
}

/// Internal durable phase; no state transition authorizes an unconfirmed effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Phase {
    /// Exact accepted intent is durable, before202 and any source publication.
    Accepted,
    /// Complete new staging and original inode backup plans are in progress.
    Staging,
    /// Inputs/targets/parents/root were revalidated under the exclusive root lease.
    PublishIntent,
    /// Authorial targets are publishing; the index remains last.
    Publishing,
    /// Complete proposed state verified and durable decision recorded.
    CommitDecided,
    /// Failure before decision requires conditional reverse rollback.
    RollingBack,
    /// Exact source outcome or cleanup remains uncertain; evidence cannot expire.
    RecoveryRequired,
    /// Verified original state and verified cleanup, retaining only safe outcome.
    FinishedNone,
    /// Verified committed state and verified cleanup, retaining safe complete result.
    FinishedCommitted,
}

/// Truthful independently established source outcome, distinct from cleanup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum WriteOutcome {
    /// Accepted work has not measured a final source outcome.
    Unmeasured,
    /// Complete exact original bytes/absences were independently verified.
    None,
    /// Complete exact proposed bytes and committed decision were verified.
    Committed,
    /// Current whole state cannot be established safely.
    Unknown,
}

/// Safe bounded termination reason, never a raw platform error or private path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Failure {
    /// Runtime cancellation remains sticky before the durable commit decision.
    Cancelled,
    /// Original accepted forward deadline expired, without renewing publication.
    Deadline,
    /// Workspace shutdown interrupted before decision.
    Shutdown,
    /// Exact old/root/parent/input/target version changed.
    Conflict,
    /// Qualified platform operation failed; source outcome requires settlement.
    Platform,
    /// Journal/durability/conditional rollback or cleanup remains uncertain.
    Recovery,
}

/// Complete metadata-only accepted root record; only qualified state can persist it.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalRecord {
    /// Same pre-known nonauthorizing ID returned by preview, acceptance and outcome.
    pub(super) operation_id: String,
    /// Exact root binding independently repeated during startup and settlement.
    root: RootIdentity,
    /// Original Root-owned accepted reservation owner; never a public effect selector.
    pub(super) nonce: u64,
    /// Digest of actual admitted request method/path/query/body, not a private alias.
    pub(super) request_sha256: String,
    /// Complete exact source manifest identity retained by the full preview.
    pub(super) exact_manifest_sha256: String,
    /// Immutable complete checked retained charge, including native artifacts.
    pub(super) retained_bytes: usize,
    /// Observed acceptance timestamp; clocks never extend forward authority.
    accepted_at: u64,
    /// Last recorded safe state timestamp for outcome projection.
    updated_at: u64,
    /// Timestamp exists only after verified finished source state and cleanup.
    pub(super) finished_at: Option<u64>,
    /// Ordered complete target metadata; last entry is the index.
    targets: Vec<JournalTarget>,
    /// Every missing directory, unique and parent before child.
    directories: Vec<JournalDirectory>,
    /// Exact original complete current input observations, including removed registrations.
    inputs: Vec<(String, ObjectFacts)>,
    /// Durable recorded phase; accepted intent is the minimum write authority.
    phase: Phase,
    /// Exact safe initial operation JSON, durable before the public accepted reply.
    accepted_reply: String,
    /// Independently established source outcome, not inferred from an exception.
    write_outcome: WriteOutcome,
    /// Verified cleanup is separate from a known source outcome.
    cleanup_verified: bool,
    /// First safe latched forward termination reason, if any.
    failure: Option<Failure>,
}

/// Checked complete generation admission passed to native target planning.
struct GenerationAdmission {
    /// Total actual supplied old and proposed raw bytes before optional physical bases.
    generations: usize,
    /// Unique actual captured identities; proposed bytes never enter this set.
    identities: std::collections::BTreeSet<(u64, u64)>,
    /// Complete normalized current input spellings for registration admission.
    input_paths: std::collections::BTreeSet<String>,
}

impl JournalRecord {
    /// Check the complete closed native metadata before it can be journal authority.
    pub(super) fn validate(&self, expected_root: &RootIdentity) -> Result<()> {
        super::transaction_state::validate_operation_id(&self.operation_id)?;
        if &self.root != expected_root
            || !hex64(&self.root.binding_sha256)
            || !hex64(&self.request_sha256)
            || !hex64(&self.exact_manifest_sha256)
            || self.retained_bytes < JOURNAL_RESERVE
            || self.retained_bytes > MAX_RETAINED_BYTES
            || self.targets.is_empty()
            || self.targets.len() > MAX_PATHS
            || self.directories.len() > MAX_PATHS
            || self.inputs.len() > MAX_PATHS
            || self.targets.last().map(|target| target.path.as_str()) != Some(INDEX_PATH)
        {
            return Err(recovery_required());
        }
        if self.accepted_reply.len() > 4096
            || timestamp(self.accepted_at).is_err()
            || timestamp(self.updated_at).is_err()
        {
            return Err(recovery_required());
        }
        let initial: Value =
            super::super::contract::parse(self.accepted_reply.as_bytes(), 4096, 512)
                .map_err(|_| recovery_required())?;
        if initial.get("operation_id").and_then(Value::as_str) != Some(self.operation_id.as_str())
            || initial.get("state").and_then(Value::as_str) != Some("pending")
            || initial.get("write_outcome").and_then(Value::as_str) != Some("unmeasured")
        {
            return Err(recovery_required());
        }
        self.validate_membership()?;
        self.validate_directories()?;
        self.validate_phases(&initial)?;
        Ok(())
    }

    /// Validate ordered complete targets, captured inputs and their bounded path union.
    fn validate_membership(&self) -> Result<()> {
        let mut paths = std::collections::BTreeSet::new();
        let mut union = std::collections::BTreeSet::new();
        let mut published_gap = false;
        for target in &self.targets {
            validate_path(&target.path).map_err(|_| recovery_required())?;
            if !paths.insert(target.path.to_ascii_lowercase())
                || !hex64(&target.proposed_sha256)
                || target.proposed_size > file_limit(&target.path)
                || target.missing_parents.len() > 64
                || target.auxiliary_name
                    != auxiliary_name(&self.operation_id, self.nonce, &target.path)
            {
                return Err(recovery_required());
            }
            union.insert(target.path.to_ascii_lowercase());
            for facts in [&target.before, &target.staged, &target.backup, &target.published]
                .into_iter()
                .flatten()
            {
                if !hex64(&facts.sha256) || facts.size > file_limit(&target.path) {
                    return Err(recovery_required());
                }
            }
            if let Some(published) = &target.published {
                if published_gap
                    || published.sha256 != target.proposed_sha256
                    || published.size != target.proposed_size
                {
                    return Err(recovery_required());
                }
            } else {
                published_gap = true;
            }
            if target.backup.is_some() && target.backup != target.before {
                return Err(recovery_required());
            }
            for parent in &target.missing_parents {
                validate_path(parent).map_err(|_| recovery_required())?;
            }
        }
        let mut input_paths = std::collections::BTreeSet::new();
        for (path, facts) in &self.inputs {
            validate_path(path).map_err(|_| recovery_required())?;
            if !input_paths.insert(path.to_ascii_lowercase())
                || !hex64(&facts.sha256)
                || facts.size > file_limit(path)
            {
                return Err(recovery_required());
            }
            if let Some(target) =
                self.targets.iter().find(|target| target.path.eq_ignore_ascii_case(path))
            {
                if target.path != *path || target.before.as_ref() != Some(facts) {
                    return Err(recovery_required());
                }
            }
            union.insert(path.to_ascii_lowercase());
        }
        if self.targets.iter().any(|target| {
            target.before.is_some()
                && !self.inputs.iter().any(|(path, facts)| {
                    path == &target.path && target.before.as_ref() == Some(facts)
                })
        }) {
            return Err(recovery_required());
        }
        if union.len() > MAX_PATHS {
            return Err(recovery_required());
        }
        Ok(())
    }

    /// Require a complete unique parent plan consistent with every target ancestor suffix.
    fn validate_directories(&self) -> Result<()> {
        let mut directory_paths = std::collections::BTreeSet::new();
        let declared: std::collections::BTreeSet<_> =
            self.targets.iter().flat_map(|target| target.missing_parents.iter().cloned()).collect();
        for directory in &self.directories {
            validate_path(&directory.path).map_err(|_| recovery_required())?;
            if !directory_paths.insert(directory.path.to_ascii_lowercase()) {
                return Err(recovery_required());
            }
        }
        if declared != self.directories.iter().map(|entry| entry.path.clone()).collect() {
            return Err(recovery_required());
        }
        for target in &self.targets {
            let mut prefixes = Vec::new();
            let mut path = String::new();
            let segments: Vec<_> = target.path.split('/').collect();
            for component in &segments[..segments.len() - 1] {
                if !path.is_empty() {
                    path.push('/');
                }
                path.push_str(component);
                prefixes.push(path.clone());
            }
            if target.missing_parents.len() > prefixes.len()
                || !prefixes.ends_with(&target.missing_parents)
                || target.missing_parents.iter().any(|path| {
                    self.directories
                        .iter()
                        .find(|dir| dir.path == *path)
                        .is_none_or(|dir| dir.nearest_parent != target.nearest_parent)
                })
            {
                return Err(recovery_required());
            }
        }
        Ok(())
    }

    /// Verify phase, publication, accepted reply and terminal cleanup facts together.
    fn validate_phases(&self, initial: &Value) -> Result<()> {
        if matches!(self.phase, Phase::Staging | Phase::PublishIntent)
            && self
                .targets
                .iter()
                .any(|target| target.published.is_some() || target.backup.is_some())
        {
            return Err(recovery_required());
        }
        if self.phase == Phase::PublishIntent
            && self
                .targets
                .iter()
                .any(|target| target.staged.is_none() || target.auxiliary_identity.is_none())
        {
            return Err(recovery_required());
        }
        match self.phase {
            Phase::FinishedNone
                if self.write_outcome != WriteOutcome::None || !self.cleanup_verified =>
            {
                return Err(recovery_required());
            }
            Phase::FinishedCommitted
                if self.write_outcome != WriteOutcome::Committed || !self.cleanup_verified =>
            {
                return Err(recovery_required());
            }
            Phase::CommitDecided if self.write_outcome != WriteOutcome::Committed => {
                return Err(recovery_required());
            }
            Phase::Accepted | Phase::Staging | Phase::PublishIntent | Phase::Publishing
                if self.write_outcome != WriteOutcome::Unmeasured =>
            {
                return Err(recovery_required());
            }
            _ => {}
        }
        let mut initial_record = self.clone();
        initial_record.phase = Phase::Accepted;
        initial_record.write_outcome = WriteOutcome::Unmeasured;
        initial_record.cleanup_verified = false;
        initial_record.failure = None;
        initial_record.updated_at = self.accepted_at;
        if *initial != initial_record.safe_outcome()? {
            return Err(recovery_required());
        }
        if self.is_finished() != self.finished_at.is_some()
            || self.finished_at.is_some_and(|at| timestamp(at).is_err())
        {
            return Err(recovery_required());
        }
        if self.write_outcome == WriteOutcome::Committed
            && self.targets.iter().any(|target| target.published.is_none())
        {
            return Err(recovery_required());
        }
        if self.targets.iter().any(|target| {
            target.staged.as_ref().is_some_and(|facts| {
                facts.sha256 != target.proposed_sha256 || facts.size != target.proposed_size
            })
        }) {
            return Err(recovery_required());
        }
        if self.phase == Phase::Accepted
            && (self.targets.iter().any(|target| {
                target.auxiliary_identity.is_some()
                    || target.staged.is_some()
                    || target.backup.is_some()
                    || target.published.is_some()
            }) || self.directories.iter().any(|entry| entry.created_identity.is_some()))
        {
            return Err(recovery_required());
        }
        Ok(())
    }

    /// Refuse phase rollback, changed accepted membership or a second publication
    /// after a durable commit. Recovery cannot create fresh forward authority.
    pub(super) fn validate_transition(&self, next: &Self) -> Result<()> {
        if self.root != next.root
            || self.operation_id != next.operation_id
            || self.targets.len() != next.targets.len()
            || self.inputs != next.inputs
            || self.accepted_at != next.accepted_at
            || self.accepted_reply != next.accepted_reply
            || self.targets.iter().zip(&next.targets).any(|(old, new)| {
                old.path != new.path
                    || old.before != new.before
                    || old.proposed_sha256 != new.proposed_sha256
                    || old.proposed_size != new.proposed_size
                    || old.nearest_parent != new.nearest_parent
                    || old.missing_parents != new.missing_parents
                    || old.auxiliary_name != new.auxiliary_name
            })
            || self
                .directories
                .iter()
                .map(|entry| &entry.path)
                .ne(next.directories.iter().map(|entry| &entry.path))
            || self
                .directories
                .iter()
                .zip(&next.directories)
                .any(|(old, new)| old.nearest_parent != new.nearest_parent)
            || self.failure.is_some() && self.failure != next.failure
        {
            return Err(recovery_required());
        }
        let allowed = match self.phase {
            Phase::Accepted => matches!(
                next.phase,
                Phase::Accepted | Phase::Staging | Phase::RollingBack | Phase::RecoveryRequired
            ),
            Phase::Staging => matches!(
                next.phase,
                Phase::Staging
                    | Phase::PublishIntent
                    | Phase::RollingBack
                    | Phase::RecoveryRequired
            ),
            Phase::PublishIntent => matches!(
                next.phase,
                Phase::Publishing | Phase::RollingBack | Phase::RecoveryRequired
            ),
            Phase::Publishing => matches!(
                next.phase,
                Phase::Publishing
                    | Phase::CommitDecided
                    | Phase::RollingBack
                    | Phase::RecoveryRequired
            ),
            Phase::CommitDecided => matches!(
                next.phase,
                Phase::CommitDecided | Phase::FinishedCommitted | Phase::RecoveryRequired
            ),
            Phase::RollingBack => matches!(
                next.phase,
                Phase::RollingBack | Phase::FinishedNone | Phase::RecoveryRequired
            ),
            Phase::RecoveryRequired => matches!(
                next.phase,
                Phase::RollingBack
                    | Phase::CommitDecided
                    | Phase::FinishedNone
                    | Phase::FinishedCommitted
                    | Phase::RecoveryRequired
            ),
            Phase::FinishedNone | Phase::FinishedCommitted => false,
        };
        if !allowed
            || self.write_outcome == WriteOutcome::Committed
                && next.write_outcome != WriteOutcome::Committed
        {
            return Err(recovery_required());
        }
        Ok(())
    }

    /// Check immutable ownership and the durable outcome before settlement authority.
    /// A stale pre-decision record cannot roll back a trusted committed generation.
    pub(super) fn validate_settlement_owner(&self, record: &Self) -> Result<()> {
        if self.nonce != record.nonce
            || self.request_sha256 != record.request_sha256
            || self.exact_manifest_sha256 != record.exact_manifest_sha256
            || self.retained_bytes != record.retained_bytes
            || self.write_outcome != record.write_outcome
        {
            return Err(recovery_required());
        }
        let mut intended = record.clone();
        intended.phase = if record.write_outcome == WriteOutcome::Committed {
            Phase::CommitDecided
        } else {
            Phase::RollingBack
        };
        self.validate_transition(&intended)
    }

    /// Hold accepted unresolved charge/slot indefinitely rather than optimistic expiry.
    pub(super) fn unresolved(&self) -> bool {
        !self.is_finished()
    }

    /// Only independently verified source outcome plus cleanup is terminal retention.
    pub(super) fn is_finished(&self) -> bool {
        matches!(self.phase, Phase::FinishedNone | Phase::FinishedCommitted)
    }

    /// Project only bounded relative targets and safe known/unknown facts, never
    /// private names, root paths, source bytes, nonce, receipts or request material.
    pub(super) fn safe_outcome(&self) -> Result<Value> {
        let state = match self.phase {
            Phase::Accepted => "pending",
            Phase::Staging | Phase::PublishIntent | Phase::Publishing | Phase::RollingBack => {
                "running"
            }
            Phase::CommitDecided | Phase::FinishedCommitted => "succeeded",
            Phase::FinishedNone if self.failure == Some(Failure::Cancelled) => "cancelled",
            Phase::FinishedNone => "failed",
            Phase::RecoveryRequired => "recovery-required",
        };
        // An independently proven original state may still await cleanup. The public
        // nonterminal wire stays unmeasured until a terminal or recovery projection.
        let outcome = if matches!(state, "pending" | "running") {
            "unmeasured"
        } else {
            match self.write_outcome {
                WriteOutcome::Unmeasured => "unmeasured",
                WriteOutcome::None => "none",
                WriteOutcome::Committed => "committed",
                WriteOutcome::Unknown => "unknown",
            }
        };
        let cleanup = if self.cleanup_verified {
            "verified"
        } else if self.phase == Phase::RecoveryRequired {
            "unverified"
        } else if self.write_outcome == WriteOutcome::Committed {
            "pending"
        } else {
            "unmeasured"
        };
        let result = if self.write_outcome == WriteOutcome::Committed {
            json!({"write_committed":true,"exact_manifest_sha256":self.exact_manifest_sha256,"committed_targets":self.targets.iter().map(|target|json!({"path":target.path,"sha256":target.proposed_sha256,"size":target.proposed_size})).collect::<Vec<_>>(),"cleanup_state":cleanup})
        } else {
            Value::Null
        };
        let error = if self.phase == Phase::RecoveryRequired {
            json!({"code":"bundle-restore-recovery-required","message":"Source restore requires recovery before further project access.","retryable":false})
        } else if self.phase == Phase::FinishedNone {
            match self.failure {
                Some(Failure::Cancelled) => Value::Null,
                Some(Failure::Deadline) => {
                    json!({"code":"bundle-restore-budget-exceeded","message":"Source restore exceeded its work budget. Inspect the retained outcome.","retryable":false})
                }
                Some(Failure::Conflict) => {
                    json!({"code":"version-conflict","message":"Source restore inputs or targets changed. Reload and inspect the retained outcome.","retryable":true})
                }
                Some(Failure::Shutdown) => {
                    json!({"code":"shutdown-in-progress","message":"Workspace shutdown interrupted source restore.","retryable":false})
                }
                _ => {
                    json!({"code":"internal-error","message":"Source restore stopped without publishing a committed result.","retryable":false})
                }
            }
        } else {
            Value::Null
        };
        Ok(
            json!({"operation_id":self.operation_id,"kind":"bundle-restore","state":state,"created_at":timestamp(self.accepted_at)?,"updated_at":timestamp(self.updated_at)?,"cancel_requested":self.failure==Some(Failure::Cancelled),"write_outcome":outcome,"progress":Value::Null,"result":result,"error":error,"cleanup_state":cleanup}),
        )
    }
}

/// Exact independently measured transaction outcome; source and cleanup stay separate.
/// Public HTTP status re-reads the checked journal instead of trusting this
/// returned diagnostic. Native tests inspect both settlement fields directly.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct RestoreOutcome {
    /// Complete safe `SourceRestoreOperation` projection or a typed before-acceptance error.
    pub(crate) operation: Value,
    /// New participating access remains blocked if cleanup/recovery is unresolved.
    pub(crate) access_blocked: bool,
}

/// Root-owned accepted reservation and immutable deadline; callers cannot create
/// this through JSON, a project journal or an outcome lookup handle.
pub(crate) struct AcceptedRestore {
    /// Same pre-known nonauthorizing operation identifier.
    operation_id: String,
    /// Exact accepted owner nonce checked against the qualified durable record.
    nonce: u64,
    /// Original runtime deadline including queue; no worker-created renewal.
    deadline: Instant,
    /// Original exact durable pending reply; no capability or preview token.
    accepted_reply: Vec<u8>,
}

/// Complete opaque planned new directory with held nearest ancestor or recorded absence.
pub(crate) struct RestoreDirectoryPlan {
    /// Native conditional creation facts; no public path creates authority.
    journal: JournalDirectory,
}

/// Moved actual captures and proposed bytes plus opaque native parent/absence planning.
/// The producer adds role/key metadata to sanitized views, never fake object identities.
pub(crate) struct RestoreTargetPlan {
    /// Fixed original actual-root identity bound through the whole transaction.
    root: RootIdentity,
    /// Full actual current capture generations, including present raw index/removals.
    inputs: Vec<(String, Captured)>,
    /// Complete input/new/base generations without path-based byte deduplication.
    capture_bytes: usize,
    /// Complete public/private retained charge within the same bounded pool.
    retained_bytes: usize,
    /// Native handle-backed targets, in authorial order and index last.
    targets: Vec<NativeTarget>,
    /// Every explicit missing parent, unique and parent-first.
    directories: Vec<RestoreDirectoryPlan>,
    /// One bounded diff budget spent once across all target projections.
    views: Vec<Value>,
}

/// A native planned target. Its base references an existing moved capture or an
/// actual extra unregistered capture; new byte vectors never carry an object ID.
struct NativeTarget {
    /// Validated public relative target path.
    path: String,
    /// Exact new bytes, moved from the decoded source profile.
    proposed: Vec<u8>,
    /// Exact new SHA independently repeated by the planner.
    proposed_sha256: String,
    /// Current generation index within the complete moved input vector, if present.
    base: Option<usize>,
    /// Strong conditional planning version derived from real parent/current facts.
    version: String,
    /// Held nearest native parent and explicit absence suffix.
    #[cfg(unix)]
    parent: std::fs::File,
    /// Actual nearest parent identity retained through staging and settlement.
    parent_identity: (u64, u64),
    /// Missing public parent paths, not created by the planner.
    missing_parents: Vec<String>,
}

impl RestoreTargetPlan {
    /// Return the complete cached shared-budget public diff/target facts, never
    /// re-reading or allocating an unbounded per-file diff during preview.
    pub(crate) fn target_views(&self) -> Vec<Value> {
        self.views.clone()
    }

    /// Return every explicitly planned absent parent before any directory creation.
    pub(crate) fn directory_views(&self) -> Vec<Value> {
        self.directories.iter().map(|directory|json!({"path":directory.journal.path,"status":"create","nearest_existing_parent_version":parent_version(&directory.journal.path,directory.journal.nearest_parent)})).collect()
    }

    /// Return complete private reservation; Store separately adds public4x+4x charges.
    pub(crate) fn reserved_private_bytes(&self) -> usize {
        self.retained_bytes
    }

    /// Return actual complete current and proposed path hash/size facts. The producer
    /// supplies original registration metadata; an unregistered base remains nullable.
    pub(crate) fn input_views(&self) -> Vec<Value> {
        let mut rows = Vec::new();
        for (path, capture) in &self.inputs {
            let proposed = self.targets.iter().find(|target| target.path == *path);
            rows.push(json!({"path":path,"current_sha256":capture.sha256,"current_size":capture.bytes.len(),"proposed_sha256":proposed.map(|target|target.proposed_sha256.as_str()),"proposed_size":proposed.map(|target|target.proposed.len())}));
        }
        for target in &self.targets {
            if !self.inputs.iter().any(|(path, _)| path == &target.path) {
                rows.push(json!({"path":target.path,"current_sha256":Value::Null,"current_size":Value::Null,"proposed_sha256":target.proposed_sha256,"proposed_size":target.proposed.len()}));
            }
        }
        rows
    }
}

/// Retain the original stopped state without producing a renewal or raw error.
fn checkpoint(control: &mut dyn WorkControl, deadline: Option<Instant>) -> WorkResult<()> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if deadline.is_some_and(|end| Instant::now() >= end) {
        return Err(WorkError::Interrupted(
            super::super::preparation::Interruption::DeadlineExceeded,
        ));
    }
    Ok(())
}

/// Require an actual fixed lowercaseSHA rather than lenient string metadata.
fn hex64(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|ch| ch.is_ascii_digit() || (b'a'..=b'f').contains(&ch))
}

/// Preserve raw-index1MiB and ordinary source-file10MiB ceilings separately.
fn file_limit(path: &str) -> usize {
    if path == INDEX_PATH { 1024 * 1024 } else { 10 * 1024 * 1024 }
}

/// Derive only a private predetermined leaf, not a public path or capabilities.
fn auxiliary_name(operation_id: &str, nonce: u64, path: &str) -> String {
    format!(
        ".forge-source-{}",
        crate::hashing::sha256_hex(
            format!("forge.restore.aux/1:{operation_id}:{nonce}:{path}").as_bytes()
        )
    )
}

/// Hash actual parent binding for a preview without exposing native identities.
fn parent_version(path: &str, identity: (u64, u64)) -> String {
    crate::hashing::sha256_hex(
        format!("forge.restore.parent/1:{path}:{}:{}", identity.0, identity.1).as_bytes(),
    )
}

/// Format a validated clock value; malformed native records already fail closed.
fn timestamp(seconds: u64) -> Result<String> {
    i64::try_from(seconds)
        .ok()
        .and_then(|secs| chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0))
        .map(|value| value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .ok_or_else(recovery_required)
}

impl AcceptedRestore {
    /// Return the exact safe reply already durably accepted before HTTP202.
    pub(crate) fn accepted_reply_bytes(&self) -> &[u8] {
        &self.accepted_reply
    }
}

impl Root {
    /// Bind the actual held project directory and original canonical spelling.
    pub(super) fn restore_identity(&self) -> Result<RootIdentity> {
        #[cfg(unix)]
        {
            use std::os::unix::{ffi::OsStrExt as _, fs::MetadataExt as _};
            let meta = self.directory.metadata().map_err(|_| unavailable())?;
            if !meta.is_dir() {
                return Err(unavailable());
            }
            let mut raw =
                format!("forge.restore.root/1:{}:{}:", meta.dev(), meta.ino()).into_bytes();
            raw.extend_from_slice(self.canonical.as_os_str().as_bytes());
            Ok(RootIdentity {
                volume: meta.dev(),
                object: meta.ino(),
                binding_sha256: crate::hashing::sha256_hex(&raw),
            })
        }
        #[cfg(not(unix))]
        {
            Err(unavailable())
        }
    }

    /// Admit complete path/generation/quota facts before target reads or effects.
    /// Inputs and new vectors move into the opaque plan; proposed bytes never gain IDs.
    pub(crate) fn plan_restore_targets(
        &self,
        proposed: Vec<(String, Vec<u8>, String)>,
        inputs: Vec<(String, Captured)>,
        control: &mut dyn WorkControl,
    ) -> WorkResult<RestoreTargetPlan> {
        checkpoint(control, None)?;
        let root = self.restore_identity()?;
        let GenerationAdmission { generations, identities, input_paths } =
            Self::admit_restore_generations(&proposed, &inputs)?;
        self.validate_restore_index(&proposed, &inputs, &input_paths)?;
        let reserved = private_charge(generations)?;
        #[cfg(unix)]
        {
            let (mut inputs, mut generations, mut identities, mut reserved) =
                (inputs, generations, identities, reserved);
            let mut targets = Vec::new();
            let mut directories = std::collections::BTreeMap::<String, RestoreDirectoryPlan>::new();
            let mut diff_remaining = MAX_DIFF_BYTES;
            let mut views = Vec::new();
            for (path, bytes, sha) in proposed {
                checkpoint(control, None)?;
                let (parent, parent_identity, missing_parents) = self.plan_parent(&path)?;
                let base = if let Some(index) = inputs.iter().position(|row| row.0 == path) {
                    Some(index)
                } else {
                    // Inspect metadata before the bounded read; no extra generation bypasses quota.
                    let extra = self.prospective_base_size(&path)?;
                    if let Some(size) = extra {
                        let next = add(generations, size)?;
                        reserved = private_charge(next)?;
                        checkpoint(control, None)?;
                        let capture = self.read(&path, size)?;
                        if capture.bytes.len() != size || !identities.insert(capture.identity) {
                            return Err(super::conflict().into());
                        }
                        generations = next;
                        inputs.push((path.clone(), capture));
                        Some(inputs.len() - 1)
                    } else {
                        None
                    }
                };
                if let Some(index) = base {
                    verify_capture(self, &path, &inputs[index].1)?;
                } else if self.read_internal(&path, file_limit(&path))?.is_some() {
                    return Err(super::conflict().into());
                }
                let current = base.map(|index| &inputs[index].1);
                let version = target_version(&path, parent_identity, current, &missing_parents);
                let (diff, binary, truncated) = preview_diff(
                    current.map_or(&[][..], |value| value.bytes.as_slice()),
                    &bytes,
                    &mut diff_remaining,
                );
                views.push(json!({"path":path,"status":if current.is_some(){"overwrite"}else{"create"},"base_sha256":current.map(|value|value.sha256.as_str()),"base_size":current.map(|value|value.bytes.len()),"target_version":version,"exact_bytes_sha256":sha,"size":bytes.len(),"diff_text":diff,"diff_truncated":truncated,"binary":binary}));
                for name in &missing_parents {
                    if !directories.contains_key(name) && directories.len() >= MAX_PATHS {
                        return Err(limit().into());
                    }
                    directories.entry(name.clone()).or_insert_with(|| RestoreDirectoryPlan {
                        journal: JournalDirectory {
                            path: name.clone(),
                            nearest_parent: parent_identity,
                            created_identity: None,
                        },
                    });
                }
                targets.push(NativeTarget {
                    path,
                    proposed: bytes,
                    proposed_sha256: sha,
                    base,
                    version,
                    parent,
                    parent_identity,
                    missing_parents,
                });
            }
            if directories.len() > MAX_PATHS {
                return Err(limit().into());
            }
            let directories = ordered_restore_directories(directories);
            checkpoint(control, None)?;
            if self.restore_identity()? != root {
                return Err(super::conflict().into());
            }
            Ok(RestoreTargetPlan {
                root,
                inputs,
                capture_bytes: generations,
                retained_bytes: reserved,
                targets,
                directories,
                views,
            })
        }
        #[cfg(not(unix))]
        {
            let _ = (root, inputs, reserved, identities, control);
            Err(unavailable().into())
        }
    }

    /// Admit complete portable path aliases, raw identities and generation sizes before reads.
    fn admit_restore_generations(
        proposed: &[(String, Vec<u8>, String)],
        inputs: &[(String, Captured)],
    ) -> WorkResult<GenerationAdmission> {
        if proposed.is_empty()
            || proposed.len() > MAX_PATHS
            || inputs.len() > MAX_PATHS
            || proposed.last().map(|row| row.0.as_str()) != Some(INDEX_PATH)
        {
            return Err(limit().into());
        }
        let mut union = std::collections::BTreeSet::new();
        let mut spellings = std::collections::BTreeMap::new();
        let mut ancestor_spellings = std::collections::BTreeMap::new();
        for path in
            proposed.iter().map(|row| row.0.as_str()).chain(inputs.iter().map(|row| row.0.as_str()))
        {
            let mut prefix = String::new();
            for component in path.split('/') {
                if !prefix.is_empty() {
                    prefix.push('/');
                }
                prefix.push_str(component);
                if ancestor_spellings
                    .insert(prefix.to_ascii_lowercase(), prefix.clone())
                    .is_some_and(|old| old != prefix)
                {
                    return Err(Error::containment().into());
                }
            }
        }
        let mut target_paths = std::collections::BTreeSet::new();
        let mut input_paths = std::collections::BTreeSet::new();
        let mut generations = 0_usize;
        for (path, bytes, sha) in proposed {
            validate_path(path)?;
            if !target_paths.insert(path.to_ascii_lowercase())
                || !hex64(sha)
                || crate::hashing::sha256_hex(bytes) != *sha
            {
                return Err(Error::containment().into());
            }
            if bytes.len() > file_limit(path) {
                return Err(limit().into());
            }
            spellings.insert(path.to_ascii_lowercase(), path.clone());
            union.insert(path.to_ascii_lowercase());
            generations = add(generations, bytes.len())?;
        }
        let mut identities = std::collections::BTreeSet::new();
        for (path, capture) in inputs {
            validate_path(path)?;
            if !input_paths.insert(path.to_ascii_lowercase())
                || !identities.insert(capture.identity)
                || !hex64(&capture.sha256)
                || crate::hashing::sha256_hex(&capture.bytes) != capture.sha256
            {
                return Err(Error::containment().into());
            }
            if capture.bytes.len() > file_limit(path) {
                return Err(limit().into());
            }
            if spellings
                .insert(path.to_ascii_lowercase(), path.clone())
                .is_some_and(|old| old != *path)
            {
                return Err(Error::containment().into());
            }
            union.insert(path.to_ascii_lowercase());
            generations = add(generations, capture.bytes.len())?;
        }
        union.insert(INDEX_PATH.to_owned());
        if union.len() > MAX_PATHS {
            return Err(limit().into());
        }
        if union.iter().any(|left| {
            union.iter().any(|right| left != right && right.starts_with(&format!("{left}/")))
        }) {
            return Err(Error::containment().into());
        }
        Ok(GenerationAdmission { generations, identities, input_paths })
    }

    /// Compare the captured current index and exact proposed registration order before native planning.
    fn validate_restore_index(
        &self,
        proposed: &[(String, Vec<u8>, String)],
        inputs: &[(String, Captured)],
        input_paths: &std::collections::BTreeSet<String>,
    ) -> WorkResult<()> {
        let raw = self.read_index()?;
        let recorded_index = inputs.iter().find(|row| row.0 == INDEX_PATH).map(|row| &row.1);
        if !same_capture(raw.as_ref(), recorded_index) {
            return Err(super::conflict().into());
        }
        if let Some(current) = raw.as_ref() {
            let index = super::super::index::Index::parse(&current.bytes)?;
            if index.resources.len().saturating_add(1) > MAX_PATHS
                || index
                    .resources
                    .iter()
                    .any(|resource| !input_paths.contains(&resource.path.to_ascii_lowercase()))
            {
                return Err(limit().into());
            }
        }
        let proposed_index =
            super::super::index::Index::parse(&proposed.last().ok_or_else(Error::invalid)?.1)?;
        if raw.as_ref().is_some_and(|current| {
            super::super::index::Index::parse(&current.bytes)
                .is_ok_and(|index| index.schema_version == "forge.workspace/2")
        }) && proposed_index.schema_version != "forge.workspace/2"
        {
            return Err(Error::invalid().into());
        }
        if proposed_index
            .resources
            .iter()
            .map(|resource| resource.path.as_str())
            .ne(proposed[..proposed.len() - 1].iter().map(|row| row.0.as_str()))
        {
            return Err(Error::invalid().into());
        }
        Ok(())
    }

    /// Persist exact accepted intent and initial safe public reply before HTTP202.
    /// Root must call only after receipt/nonce/shared-pool admission under its Store lock;
    /// it supplies the original absolute deadline, not a fresh worker budget.
    #[allow(clippy::too_many_arguments)] // Explicit immutable receipt bindings cross one durable boundary.
    pub(crate) fn accept_restore(
        &self,
        plan: &RestoreTargetPlan,
        state: &TransactionState,
        operation_id: String,
        nonce: u64,
        request_sha256: String,
        exact_manifest_sha256: String,
        accepted_reply: Vec<u8>,
        deadline: Instant,
    ) -> Result<AcceptedRestore> {
        if private_charge(plan.capture_bytes)? != plan.retained_bytes {
            return Err(Error::invalid());
        }
        if Instant::now() >= deadline
            || self.restore_identity()? != plan.root
            || !hex64(&request_sha256)
            || !hex64(&exact_manifest_sha256)
        {
            return Err(unavailable());
        }
        super::transaction_state::validate_operation_id(&operation_id)?;
        let supplied = super::super::contract::parse(&accepted_reply, 4096, 512)
            .map_err(|_| Error::invalid())?;
        let created = supplied
            .get("created_at")
            .and_then(Value::as_str)
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp())
            .and_then(|value| u64::try_from(value).ok())
            .ok_or_else(Error::invalid)?;
        if created > unix_seconds()?.saturating_add(1) {
            return Err(Error::invalid());
        }
        let now = created;
        let mut record = JournalRecord {
            operation_id: operation_id.clone(),
            root: plan.root.clone(),
            nonce,
            request_sha256,
            exact_manifest_sha256,
            retained_bytes: plan.retained_bytes,
            accepted_at: now,
            updated_at: now,
            finished_at: None,
            targets: plan
                .targets
                .iter()
                .map(|target| JournalTarget {
                    path: target.path.clone(),
                    before: target.base.map(|index| facts(&plan.inputs[index].1)),
                    proposed_sha256: target.proposed_sha256.clone(),
                    proposed_size: target.proposed.len(),
                    nearest_parent: target.parent_identity,
                    missing_parents: target.missing_parents.clone(),
                    auxiliary_name: auxiliary_name(&operation_id, nonce, &target.path),
                    auxiliary_identity: None,
                    staged: None,
                    backup: None,
                    published: None,
                })
                .collect(),
            directories: plan.directories.iter().map(|entry| entry.journal.clone()).collect(),
            inputs: plan
                .inputs
                .iter()
                .map(|(path, capture)| (path.clone(), facts(capture)))
                .collect(),
            phase: Phase::Accepted,
            accepted_reply: String::new(),
            write_outcome: WriteOutcome::Unmeasured,
            cleanup_verified: false,
            failure: None,
        };
        if supplied != record.safe_outcome()? {
            return Err(Error::invalid());
        }
        record.accepted_reply =
            String::from_utf8(accepted_reply.clone()).map_err(|_| Error::invalid())?;
        if Instant::now() >= deadline {
            return Err(Error::new(
                "bundle-restore-budget-exceeded",
                "Source restore exceeded its work budget before acceptance.",
                false,
            ));
        }
        state.accept(record)?;
        Ok(AcceptedRestore { operation_id, nonce, deadline, accepted_reply })
    }
}

/// Checked complete arithmetic shared by generation and reservation preadmission.
fn add(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right).filter(|value| *value <= MAX_CAPTURE_BYTES).ok_or_else(limit)
}

/// Charge moved generations and native stage/backup generations plus four bounded
/// journal representations. Public wrapper/preview charges remain Root's shared pool.
fn private_charge(bytes: usize) -> Result<usize> {
    bytes
        .checked_mul(2)
        .and_then(|value| value.checked_add(JOURNAL_RESERVE))
        .filter(|value| *value <= MAX_RETAINED_BYTES)
        .ok_or_else(limit)
}

/// Safe typed configured-bound failure before any resource creation.
fn limit() -> Error {
    Error::new(
        "payload-too-large",
        "Complete source restore captures or retention exceed the bound.",
        false,
    )
}

/// Extract only actual observed identities, never supplied/new byte identities.
fn facts(capture: &Captured) -> ObjectFacts {
    ObjectFacts {
        volume: capture.identity.0,
        object: capture.identity.1,
        sha256: capture.sha256.clone(),
        size: capture.bytes.len(),
    }
}

/// Compare complete held generation observations including native identity.
fn same_capture(left: Option<&Captured>, right: Option<&Captured>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.identity == b.identity && a.sha256 == b.sha256 && a.bytes == b.bytes
        }
        _ => false,
    }
}

/// Re-read an admitted actual original generation before publication authority.
fn verify_capture(root: &Root, path: &str, expected: &Captured) -> Result<()> {
    if same_capture(root.read_internal(path, file_limit(path))?.as_ref(), Some(expected)) {
        Ok(())
    } else {
        Err(super::conflict())
    }
}

/// Consume the complete directory plan in stable parent-before-child order.
#[cfg(unix)]
fn ordered_restore_directories(
    directories: std::collections::BTreeMap<String, RestoreDirectoryPlan>,
) -> Vec<RestoreDirectoryPlan> {
    let mut directories: Vec<_> = directories.into_values().collect();
    directories.sort_by(|left, right| {
        left.journal
            .path
            .matches('/')
            .count()
            .cmp(&right.journal.path.matches('/').count())
            .then(left.journal.path.cmp(&right.journal.path))
    });
    directories
}

/// Domain-separated exact old/parent/missing-parent target conditional version.
fn target_version(
    path: &str,
    parent: (u64, u64),
    base: Option<&Captured>,
    missing: &[String],
) -> String {
    crate::hashing::sha256_hex(
        format!(
            "forge.restore.target/1:{path}:{}:{}:{}:{}",
            parent.0,
            parent.1,
            base.map_or_else(
                || "absent".to_owned(),
                |value| format!(
                    "{}:{}:{}:{}",
                    value.identity.0,
                    value.identity.1,
                    value.sha256,
                    value.bytes.len()
                )
            ),
            missing.join("/")
        )
        .as_bytes(),
    )
}

/// Bound the total visible textual preview while retaining exact raw hashes elsewhere.
/// Binary bytes never receive an invented textual diff or content normalization.
fn preview_diff(before: &[u8], after: &[u8], remaining: &mut usize) -> (String, bool, bool) {
    let (Ok(old), Ok(new)) = (std::str::from_utf8(before), std::str::from_utf8(after)) else {
        return (String::new(), true, false);
    };
    if old == new {
        return (String::new(), false, false);
    }
    let mut out = String::new();
    let mut truncated = false;
    for (label, value) in [("--- current\n", old), ("+++ proposed\n", new)] {
        for fragment in [label, value, "\n"] {
            let available = (*remaining).min(fragment.len());
            let mut end = available;
            while end > 0 && !fragment.is_char_boundary(end) {
                end -= 1;
            }
            out.push_str(&fragment[..end]);
            *remaining -= end;
            if end < fragment.len() {
                truncated = true;
                break;
            }
        }
        if truncated {
            break;
        }
    }
    (out, false, truncated)
}

#[cfg(unix)]
impl Root {
    /// Hold the nearest existing no-follow parent and record every absent suffix.
    fn plan_parent(&self, path: &str) -> Result<(std::fs::File, (u64, u64), Vec<String>)> {
        use std::os::unix::fs::MetadataExt as _;
        let mut parent = self.directory.try_clone().map_err(|_| unavailable())?;
        let segments: Vec<_> = path.split('/').collect();
        let mut prefix = String::new();
        let mut missing = Vec::new();
        let mut absent = false;
        for segment in &segments[..segments.len() - 1] {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(segment);
            if absent {
                missing.push(prefix.clone());
                continue;
            }
            match super::unix_open(&parent, std::ffi::OsStr::new(segment), true) {
                Ok(next) => parent = next,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    absent = true;
                    missing.push(prefix.clone());
                }
                Err(_) => return Err(Error::containment()),
            }
        }
        let meta = parent.metadata().map_err(|_| unavailable())?;
        if !meta.is_dir() {
            return Err(Error::containment());
        }
        Ok((parent, (meta.dev(), meta.ino()), missing))
    }

    /// Read only current metadata to admit an unregistered physical base before bytes.
    fn prospective_base_size(&self, path: &str) -> Result<Option<usize>> {
        use std::os::unix::fs::MetadataExt as _;
        let (parent, _, missing) = self.plan_parent(path)?;
        if !missing.is_empty() {
            return Ok(None);
        }
        let leaf = path.rsplit('/').next().ok_or_else(Error::invalid)?;
        let file = match super::unix_open(&parent, std::ffi::OsStr::new(leaf), false) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Error::containment()),
        };
        let meta = file.metadata().map_err(|_| Error::containment())?;
        if !meta.is_file() || meta.nlink() != 1 {
            return Err(Error::containment());
        }
        let size = usize::try_from(meta.len()).map_err(|_| limit())?;
        if size > file_limit(path) {
            return Err(limit());
        }
        Ok(Some(size))
    }
}

impl Root {
    /// Record lease-unavailable accepted work without reading or changing project data.
    /// Only the exact still-Accepted root/ID/nonce/reply owner may defer; reservation
    /// remains retained and fresh qualified recovery determines source and cleanup.
    /// This port grants no renewed forward deadline or publication authority.
    pub(crate) fn defer_restore_recovery(
        &self,
        state: &TransactionState,
        accepted: &AcceptedRestore,
    ) -> Result<()> {
        let mut record = state
            .records()?
            .into_iter()
            .find(|record| {
                record.operation_id == accepted.operation_id && record.nonce == accepted.nonce
            })
            .ok_or_else(recovery_required)?;
        if self.restore_identity()? != record.root
            || record.phase != Phase::Accepted
            || record.accepted_reply.as_bytes() != accepted.accepted_reply
        {
            return Err(recovery_required());
        }
        record.phase = Phase::RecoveryRequired;
        record.write_outcome = WriteOutcome::Unknown;
        record.cleanup_verified = false;
        record.failure.get_or_insert(Failure::Recovery);
        persist(state, &mut record)
    }

    /// Consume the exact durable accepted owner while Root holds its State write lease.
    /// This sealed port does not reacquire Root.read locks or assert a CLI barrier.
    /// Every forward failure/unwind attempts settlement; uncertain facts block access.
    #[allow(clippy::needless_pass_by_value)] // Consume the sealed plan and accepted owner exactly once.
    pub(crate) fn run_restore(
        &self,
        plan: RestoreTargetPlan,
        state: &TransactionState,
        accepted: AcceptedRestore,
        control: &mut dyn WorkControl,
    ) -> Result<RestoreOutcome> {
        let mut record = state
            .records()?
            .into_iter()
            .find(|record| {
                record.operation_id == accepted.operation_id && record.nonce == accepted.nonce
            })
            .ok_or_else(recovery_required)?;
        if record.root != plan.root
            || self.restore_identity()? != plan.root
            || record.phase != Phase::Accepted
            || record.retained_bytes != plan.retained_bytes
            || record.accepted_reply.as_bytes() != accepted.accepted_reply
        {
            return Err(recovery_required());
        }
        if record.targets.len() != plan.targets.len()
            || record.targets.iter().zip(&plan.targets).any(|(row, target)| {
                row.path != target.path
                    || row.proposed_sha256 != target.proposed_sha256
                    || row.proposed_size != target.proposed.len()
                    || row.nearest_parent != target.parent_identity
                    || row.missing_parents != target.missing_parents
                    || row.before != target.base.map(|index| facts(&plan.inputs[index].1))
            })
            || record.inputs
                != plan
                    .inputs
                    .iter()
                    .map(|(path, capture)| (path.clone(), facts(capture)))
                    .collect::<Vec<_>>()
        {
            return Err(recovery_required());
        }
        #[cfg(unix)]
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.forward_restore(&plan, state, &mut record, accepted.deadline, control)
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    latch_failure(&mut record, &error);
                }
                Err(_) => {
                    record.failure.get_or_insert(Failure::Platform);
                }
            }
            if record.phase != Phase::FinishedCommitted {
                let end = Instant::now()
                    .checked_add(Duration::from_secs(SETTLEMENT_SECONDS))
                    .ok_or_else(recovery_required)?;
                if self.settle_restore(state, &mut record, end).is_err() {
                    record.phase = Phase::RecoveryRequired;
                    record.cleanup_verified = false;
                    if record.write_outcome == WriteOutcome::Unmeasured {
                        record.write_outcome = WriteOutcome::Unknown;
                    }
                    record.failure.get_or_insert(Failure::Recovery);
                    let _ = persist(state, &mut record);
                }
            }
        }
        #[cfg(not(unix))]
        {
            let _ = (plan, control);
            record.failure = Some(Failure::Platform);
            record.phase = Phase::RecoveryRequired;
            record.write_outcome = WriteOutcome::Unknown;
            let _ = persist(state, &mut record);
        }
        Ok(RestoreOutcome {
            operation: record.safe_outcome()?,
            access_blocked: !record.is_finished(),
        })
    }

    /// Reconcile only already accepted trusted metadata. No source bytes from a
    /// journal can authorize a fresh publication or an unconfirmed rollforward.
    pub(super) fn recover_restore(
        &self,
        state: &TransactionState,
        mut record: JournalRecord,
    ) -> Result<()> {
        if self.restore_identity()? != record.root {
            return Err(recovery_required());
        }
        record.validate(&record.root)?;
        #[cfg(unix)]
        {
            let end = Instant::now()
                .checked_add(Duration::from_secs(SETTLEMENT_SECONDS))
                .ok_or_else(recovery_required)?;
            self.settle_restore(state, &mut record, end)
        }
        #[cfg(not(unix))]
        {
            let _ = state;
            Err(unavailable())
        }
    }
}

/// Preserve the original first safe cause without rendering private platform errors.
fn latch_failure(record: &mut JournalRecord, error: &WorkError) {
    use super::super::preparation::Interruption;
    let failure = match error {
        WorkError::Interrupted(Interruption::CancelRequested) => Failure::Cancelled,
        WorkError::Interrupted(Interruption::DeadlineExceeded) => Failure::Deadline,
        WorkError::Interrupted(Interruption::Shutdown) => Failure::Shutdown,
        WorkError::Failed(error) if error.code == "version-conflict" => Failure::Conflict,
        _ => Failure::Platform,
    };
    record.failure.get_or_insert(failure);
}

/// Journal one checked phase without inventing timestamp or allowing memory-only success.
fn persist(state: &TransactionState, record: &mut JournalRecord) -> Result<()> {
    record.updated_at = unix_seconds()?;
    state.update(record)
}

/// Fence the separate cooperative settlement attempt without resuming forward work.
fn settlement_check(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline { Err(recovery_required()) } else { Ok(()) }
}

#[cfg(unix)]
#[allow(unsafe_code)] // Individually documented descriptor-relative planned native effects.
impl Root {
    /// Stage every target before authorial publication, verify whole new state,
    /// then record a durable commit decision before cleanup.
    fn forward_restore(
        &self,
        plan: &RestoreTargetPlan,
        state: &TransactionState,
        record: &mut JournalRecord,
        deadline: Instant,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        checkpoint(control, Some(deadline))?;
        self.verify_original(record)?;
        record.phase = Phase::Staging;
        persist(state, record)?;
        self.create_restore_directories(state, record, deadline, control)?;
        self.stage_restore_targets(plan, state, record, deadline, control)?;
        checkpoint(control, Some(deadline))?;
        self.verify_original(record)?;
        record.phase = Phase::PublishIntent;
        persist(state, record)?;
        record.phase = Phase::Publishing;
        persist(state, record)?;
        for index in 0..record.targets.len() {
            checkpoint(control, Some(deadline))?;
            let target = record.targets[index].clone();
            let (parent, leaf) = self.resolve_leaf(&target.path, record)?;
            let auxiliary = open_auxiliary(&parent, &target)?;
            if read_facts(&parent, &leaf, file_limit(&target.path))? != target.before
                || read_facts(&auxiliary, "payload", file_limit(&target.path))? != target.staged
            {
                return Err(super::conflict().into());
            }
            checkpoint(control, Some(deadline))?;
            atomic_move(&auxiliary, "payload", &parent, &leaf, target.before.is_some())?;
            auxiliary
                .sync_all()
                .and_then(|()| parent.sync_all())
                .map_err(|_| recovery_required())?;
            let published = read_facts(&parent, &leaf, file_limit(&target.path))?;
            if published != target.staged {
                return Err(recovery_required().into());
            }
            record.targets[index].published = published;
            record.targets[index].backup = if target.before.is_some() {
                read_facts(&auxiliary, "payload", file_limit(&target.path))?
            } else {
                None
            };
            if record.targets[index].backup != target.before {
                return Err(recovery_required().into());
            }
            persist(state, record)?;
        }
        checkpoint(control, Some(deadline))?;
        self.verify_committed(record)?;
        checkpoint(control, Some(deadline))?;
        let mut decision = record.clone();
        decision.write_outcome = WriteOutcome::Committed;
        decision.phase = Phase::CommitDecided;
        persist(state, &mut decision)?;
        *record = decision;
        Ok(())
    }

    /// Stage every exact planned file under verified held parents before publication intent.
    /// Record each auxiliary identity and verified payload with the original checkpoints.
    fn stage_restore_targets(
        &self,
        plan: &RestoreTargetPlan,
        state: &TransactionState,
        record: &mut JournalRecord,
        deadline: Instant,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        for (index, target) in plan.targets.iter().enumerate() {
            checkpoint(control, Some(deadline))?;
            if directory_identity(&target.parent)? != target.parent_identity
                || target.version
                    != target_version(
                        &target.path,
                        target.parent_identity,
                        target.base.map(|index| &plan.inputs[index].1),
                        &target.missing_parents,
                    )
            {
                return Err(super::conflict().into());
            }
            let (parent, _leaf) = self.resolve_leaf(&target.path, record)?;
            let expected_parent = if target.missing_parents.is_empty() {
                target.parent_identity
            } else {
                record
                    .directories
                    .iter()
                    .find(|dir| {
                        Some(dir.path.as_str()) == target.missing_parents.last().map(String::as_str)
                    })
                    .and_then(|dir| dir.created_identity)
                    .ok_or_else(recovery_required)?
            };
            if directory_identity(&parent)? != expected_parent {
                return Err(super::conflict().into());
            }
            let name = record.targets[index].auxiliary_name.clone();
            if observed_directory(&parent, &name)?.is_some() {
                return Err(super::conflict().into());
            }
            checkpoint(control, Some(deadline))?;
            mkdir_exclusive(&parent, &name)?;
            let auxiliary = super::unix_open(&parent, std::ffi::OsStr::new(&name), true)
                .map_err(|_| recovery_required())?;
            record.targets[index].auxiliary_identity =
                Some(private_directory_identity(&auxiliary)?);
            auxiliary
                .sync_all()
                .and_then(|()| parent.sync_all())
                .map_err(|_| recovery_required())?;
            persist(state, record)?;
            checkpoint(control, Some(deadline))?;
            write_stage(&auxiliary, &target.proposed)?;
            record.targets[index].staged =
                read_facts(&auxiliary, "payload", file_limit(&target.path))?;
            if record.targets[index].staged.as_ref().is_none_or(|facts| {
                facts.sha256 != target.proposed_sha256 || facts.size != target.proposed.len()
            }) {
                return Err(recovery_required().into());
            }
            auxiliary.sync_all().map_err(|_| recovery_required())?;
            persist(state, record)?;
        }
        Ok(())
    }

    /// Create only planned exclusive directories, recording each held identity before staging files.
    fn create_restore_directories(
        &self,
        state: &TransactionState,
        record: &mut JournalRecord,
        deadline: Instant,
        control: &mut dyn WorkControl,
    ) -> WorkResult<()> {
        for index in 0..record.directories.len() {
            checkpoint(control, Some(deadline))?;
            let path = record.directories[index].path.clone();
            let (parent, leaf) = self.resolve_leaf(&path, record)?;
            if observed_directory(&parent, &leaf)?.is_some() {
                return Err(super::conflict().into());
            }
            let expected = record
                .directories
                .iter()
                .find(|entry| {
                    Some(entry.path.as_str()) == path.rsplit_once('/').map(|(name, _)| name)
                })
                .and_then(|entry| entry.created_identity)
                .unwrap_or(record.directories[index].nearest_parent);
            if directory_identity(&parent)? != expected {
                return Err(super::conflict().into());
            }
            checkpoint(control, Some(deadline))?;
            mkdir_exclusive(&parent, &leaf)?;
            let child = super::unix_open(&parent, std::ffi::OsStr::new(&leaf), true)
                .map_err(|_| recovery_required())?;
            record.directories[index].created_identity = Some(directory_identity(&child)?);
            child.sync_all().and_then(|()| parent.sync_all()).map_err(|_| recovery_required())?;
            persist(state, record)?;
        }
        Ok(())
    }

    /// Resolve a root-relative leaf through held no-follow parents. Newly created
    /// parent identities must match accepted records before use or cleanup.
    fn resolve_leaf(&self, path: &str, record: &JournalRecord) -> Result<(std::fs::File, String)> {
        if self.restore_identity()? != record.root {
            return Err(recovery_required());
        }
        let reopened = Root::open(&self.canonical)?;
        if reopened.restore_identity()? != record.root {
            return Err(super::conflict());
        }
        let mut parent = self.directory.try_clone().map_err(|_| recovery_required())?;
        let segments: Vec<_> = path.split('/').collect();
        let mut prefix = String::new();
        for name in &segments[..segments.len() - 1] {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(name);
            let child = super::unix_open(&parent, std::ffi::OsStr::new(name), true)
                .map_err(|_| recovery_required())?;
            if let Some(planned) = record.directories.iter().find(|dir| dir.path == prefix) {
                if planned.created_identity != Some(directory_identity(&child)?) {
                    return Err(recovery_required());
                }
            }
            parent = child;
        }
        Ok((parent, segments.last().ok_or_else(Error::invalid)?.to_string()))
    }

    /// Independently verify every original captured generation and target absence.
    /// Removed registrations remain inputs and are never implicitly deleted.
    fn verify_original(&self, record: &JournalRecord) -> Result<()> {
        if self.restore_identity()? != record.root {
            return Err(super::conflict());
        }
        for (path, expected) in &record.inputs {
            let actual = self.observe_path(path)?;
            if actual.as_ref() != Some(expected) {
                return Err(super::conflict());
            }
        }
        for target in &record.targets {
            if self.observe_path(&target.path)? != target.before {
                return Err(super::conflict());
            }
        }
        Ok(())
    }

    /// Verify all authorial proposed objects plus unchanged removed current inputs.
    fn verify_committed(&self, record: &JournalRecord) -> Result<()> {
        for target in &record.targets {
            if target.published.is_none() || self.observe_path(&target.path)? != target.published {
                return Err(recovery_required());
            }
        }
        for (path, expected) in &record.inputs {
            if !record.targets.iter().any(|target| target.path == *path)
                && self.observe_path(path)?.as_ref() != Some(expected)
            {
                return Err(recovery_required());
            }
        }
        Ok(())
    }

    /// Stream only an explicitly known relative path, returning verified absence
    /// for a missing ancestor and refusing links, kinds and over-limit byte reads.
    fn observe_path(&self, path: &str) -> Result<Option<ObjectFacts>> {
        let (parent, _, missing) = self.plan_parent(path)?;
        if !missing.is_empty() {
            return Ok(None);
        }
        read_facts(&parent, path.rsplit('/').next().ok_or_else(Error::invalid)?, file_limit(path))
    }

    /// Reconcile only identity-proven moves. A crash between exchange and its
    /// journal update can be recognized by both already recorded original/staged IDs.
    fn reconcile_target(&self, record: &mut JournalRecord, index: usize) -> Result<()> {
        let target = record.targets[index].clone();
        let (_parent, identity, missing) = self.plan_parent(&target.path)?;
        if !missing.is_empty() {
            if target.before.is_none()
                && target.auxiliary_identity.is_none()
                && target.staged.is_none()
                && target.published.is_none()
                && missing == target.missing_parents
                && identity == target.nearest_parent
            {
                return Ok(());
            }
            return Err(recovery_required());
        }
        let (parent, leaf) = self.resolve_leaf(&target.path, record)?;
        let actual = read_facts(&parent, &leaf, file_limit(&target.path))?;
        if target.auxiliary_identity.is_none() {
            if observed_directory(&parent, &target.auxiliary_name)?.is_some() {
                return Err(recovery_required());
            }
            if actual != target.before {
                return Err(recovery_required());
            }
            return Ok(());
        }
        let auxiliary = open_auxiliary(&parent, &target)?;
        let slot = read_facts(&auxiliary, "payload", file_limit(&target.path))?;
        if target.staged.is_some() && actual == target.staged && slot == target.before {
            record.targets[index].published = actual;
            record.targets[index].backup = slot;
        } else if actual == target.before && slot == target.staged {
            record.targets[index].published = None;
            record.targets[index].backup = None;
        } else if actual == target.before
            && slot.is_none()
            && target.before.is_none()
            && target.published.is_some()
        {
            record.targets[index].published = None;
            record.targets[index].backup = None;
            record.targets[index].staged = None;
        } else {
            return Err(recovery_required());
        }
        Ok(())
    }

    /// Remove only verified owned auxiliary objects; cleanup failure retains known source facts.
    fn cleanup_restore_auxiliaries(
        &self,
        state: &TransactionState,
        record: &mut JournalRecord,
        deadline: Instant,
        committed: bool,
    ) -> Result<()> {
        // Cleanup failure may preserve a known source outcome without cleanup credit.
        for index in 0..record.targets.len() {
            settlement_check(deadline)?;
            let target = record.targets[index].clone();
            let (_nearest, identity, missing) = self.plan_parent(&target.path)?;
            if !missing.is_empty() {
                if target.auxiliary_identity.is_none()
                    && missing == target.missing_parents
                    && identity == target.nearest_parent
                {
                    continue;
                }
                return Err(recovery_required());
            }
            let (parent, _leaf) = self.resolve_leaf(&target.path, record)?;
            if target.auxiliary_identity.is_none() {
                if observed_directory(&parent, &target.auxiliary_name)?.is_some() {
                    return Err(recovery_required());
                }
                continue;
            }
            if observed_directory(&parent, &target.auxiliary_name)?.is_none() {
                record.targets[index].auxiliary_identity = None;
                record.targets[index].staged = None;
                record.targets[index].backup = None;
                persist(state, record)?;
                continue;
            }
            let auxiliary = open_auxiliary(&parent, &target)?;
            let expected = if committed { target.before.as_ref() } else { target.staged.as_ref() };
            let current = read_facts(&auxiliary, "payload", file_limit(&target.path))?;
            if current.as_ref() != expected {
                return Err(recovery_required());
            }
            if let Some(expected) = expected {
                state.verify_settlement_owner(record)?;
                unlink_owned(&auxiliary, "payload", expected, file_limit(&target.path), deadline)?;
                auxiliary.sync_all().map_err(|_| recovery_required())?;
            }
            state.verify_settlement_owner(record)?;
            remove_empty_owned(
                &parent,
                &target.auxiliary_name,
                target.auxiliary_identity.ok_or_else(recovery_required)?,
                deadline,
            )?;
            parent.sync_all().map_err(|_| recovery_required())?;
            // Keep accepted/staged facts for source outcome; cleared ownership is verified.
            record.targets[index].auxiliary_identity = None;
            record.targets[index].staged = None;
            record.targets[index].backup = None;
            persist(state, record)?;
        }
        Ok(())
    }

    /// Settle within one separate absolute attempt. Before decision restore only
    /// verified owned objects; after decision never claim no-write or roll back.
    fn settle_restore(
        &self,
        state: &TransactionState,
        record: &mut JournalRecord,
        deadline: Instant,
    ) -> Result<()> {
        settlement_check(deadline)?;
        // A failed replace may already have installed a durable CommitDecided record.
        // A stale live record is never rollback authority while that owner is blocked.
        state.verify_settlement_owner(record)?;
        settlement_check(deadline)?;
        let committed = record.write_outcome == WriteOutcome::Committed;
        if committed {
            self.verify_committed(record)?;
        } else {
            record.phase = Phase::RollingBack;
            // Journal uncertainty stops before any further project settlement effect.
            persist(state, record)?;
            for index in 0..record.targets.len() {
                settlement_check(deadline)?;
                self.reconcile_target(record, index)?;
            }
            for index in (0..record.targets.len()).rev() {
                settlement_check(deadline)?;
                let target = record.targets[index].clone();
                if target.published.is_none() {
                    continue;
                }
                let (parent, leaf) = self.resolve_leaf(&target.path, record)?;
                let auxiliary = open_auxiliary(&parent, &target)?;
                if read_facts(&parent, &leaf, file_limit(&target.path))? != target.published {
                    return Err(recovery_required());
                }
                if let Some(before) = &target.before {
                    if read_facts(&auxiliary, "payload", file_limit(&target.path))?.as_ref()
                        != Some(before)
                    {
                        return Err(recovery_required());
                    }
                    settlement_check(deadline)?;
                    state.verify_settlement_owner(record)?;
                    atomic_move(&auxiliary, "payload", &parent, &leaf, true)?;
                } else {
                    state.verify_settlement_owner(record)?;
                    unlink_owned(
                        &parent,
                        &leaf,
                        target.published.as_ref().ok_or_else(recovery_required)?,
                        file_limit(&target.path),
                        deadline,
                    )?;
                }
                parent
                    .sync_all()
                    .and_then(|()| auxiliary.sync_all())
                    .map_err(|_| recovery_required())?;
                if read_facts(&parent, &leaf, file_limit(&target.path))? != target.before {
                    return Err(recovery_required());
                }
                record.targets[index].published = None;
                record.targets[index].backup = None;
                persist(state, record)?;
            }
            self.verify_original(record)?;
            record.write_outcome = WriteOutcome::None;
            persist(state, record)?;
        }
        self.cleanup_restore_auxiliaries(state, record, deadline, committed)?;
        if committed {
            self.verify_committed(record)?;
        } else {
            for index in (0..record.directories.len()).rev() {
                settlement_check(deadline)?;
                let directory = record.directories[index].clone();
                if let Some(identity) = directory.created_identity {
                    let (parent, leaf) = self.resolve_leaf(&directory.path, record)?;
                    if observed_directory(&parent, &leaf)?.is_some() {
                        state.verify_settlement_owner(record)?;
                        remove_empty_owned(&parent, &leaf, identity, deadline)?;
                    }
                    parent.sync_all().map_err(|_| recovery_required())?;
                    record.directories[index].created_identity = None;
                    persist(state, record)?;
                } else {
                    let (parent, _, missing) = self.plan_parent(&directory.path)?;
                    if missing.is_empty()
                        && observed_directory(
                            &parent,
                            directory.path.rsplit('/').next().ok_or_else(Error::invalid)?,
                        )?
                        .is_some()
                    {
                        return Err(recovery_required());
                    }
                }
            }
            self.verify_original(record)?;
        }
        settlement_check(deadline)?;
        record.cleanup_verified = true;
        record.phase = if committed { Phase::FinishedCommitted } else { Phase::FinishedNone };
        record.finished_at = Some(unix_seconds()?);
        persist(state, record)
    }
}

/// Observe the actual directory ID without deriving ownership from path spelling.
#[cfg(unix)]
fn directory_identity(file: &std::fs::File) -> Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt as _;
    let meta = file.metadata().map_err(|_| recovery_required())?;
    if !meta.is_dir() {
        return Err(recovery_required());
    }
    Ok((meta.dev(), meta.ino()))
}

/// Require an owned restrictive auxiliary directory before storing plaintext backups.
#[cfg(unix)]
#[allow(unsafe_code)] // geteuid is used solely for observed owner equality.
fn private_directory_identity(file: &std::fs::File) -> Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt as _;
    let meta = file.metadata().map_err(|_| recovery_required())?;
    // SAFETY: geteuid takes no pointer and changes no state.
    let uid = unsafe { libc::geteuid() };
    if meta.uid() != uid || meta.mode() & 0o077 != 0 {
        return Err(recovery_required());
    }
    directory_identity(file)
}

/// Query a known no-follow directory name; links or foreign kinds fail closed.
#[cfg(unix)]
fn observed_directory(parent: &std::fs::File, name: &str) -> Result<Option<(u64, u64)>> {
    match super::unix_open(parent, std::ffi::OsStr::new(name), true) {
        Ok(file) => Ok(Some(directory_identity(&file)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(recovery_required()),
    }
}

/// Reopen only an identity-proven private auxiliary; names alone never authorize cleanup.
#[cfg(unix)]
fn open_auxiliary(parent: &std::fs::File, target: &JournalTarget) -> Result<std::fs::File> {
    let file = super::unix_open(parent, std::ffi::OsStr::new(&target.auxiliary_name), true)
        .map_err(|_| recovery_required())?;
    if Some(private_directory_identity(&file)?) != target.auxiliary_identity {
        return Err(recovery_required());
    }
    Ok(file)
}

/// Stream bounded exact file facts with stable metadata and single-link identity.
#[cfg(unix)]
fn read_facts(parent: &std::fs::File, name: &str, limit: usize) -> Result<Option<ObjectFacts>> {
    use sha2::Digest as _;
    use std::io::Read as _;
    use std::os::unix::fs::MetadataExt as _;
    let file = match super::unix_open(parent, std::ffi::OsStr::new(name), false) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(recovery_required()),
    };
    let before = file.metadata().map_err(|_| recovery_required())?;
    if !before.is_file() || before.nlink() != 1 || before.len() > limit as u64 {
        return Err(recovery_required());
    }
    let mut hasher = sha2::Sha256::new();
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = [0_u8; 8192];
    let mut size = 0_usize;
    loop {
        let count = reader.read(&mut buffer).map_err(|_| recovery_required())?;
        if count == 0 {
            break;
        }
        size = size
            .checked_add(count)
            .filter(|value| *value <= limit)
            .ok_or_else(recovery_required)?;
        hasher.update(&buffer[..count]);
    }
    let after = reader.get_ref().metadata().map_err(|_| recovery_required())?;
    if metadata_stamp(&before) != metadata_stamp(&after) || size as u64 != before.len() {
        return Err(recovery_required());
    }
    Ok(Some(ObjectFacts {
        volume: after.dev(),
        object: after.ino(),
        sha256: crate::hashing::lower_hex(&hasher.finalize()),
        size,
    }))
}

/// Keep all metadata influencing stable bounded identity; no content is copied here.
#[cfg(unix)]
fn metadata_stamp(meta: &std::fs::Metadata) -> (u64, u64, u64, u64, u32, i64, i64, i64, i64) {
    use std::os::unix::fs::MetadataExt as _;
    (
        meta.dev(),
        meta.ino(),
        meta.len(),
        meta.nlink(),
        meta.mode(),
        meta.mtime(),
        meta.mtime_nsec(),
        meta.ctime(),
        meta.ctime_nsec(),
    )
}

/// Create exactly one planned directory exclusively against its held parent.
#[cfg(unix)]
#[allow(unsafe_code)] // single relative NUL-terminated component, held parent.
fn mkdir_exclusive(parent: &std::fs::File, name: &str) -> Result<()> {
    use std::os::{fd::AsRawFd as _, unix::ffi::OsStrExt as _};
    let name = std::ffi::CString::new(std::ffi::OsStr::new(name).as_bytes())
        .map_err(|_| recovery_required())?;
    // SAFETY: one validated component and live owned parent descriptor.
    if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
        return Err(recovery_required());
    }
    Ok(())
}

/// Write only one exclusive private staging slot with checked file durability.
#[cfg(unix)]
#[allow(unsafe_code)] // fresh successful openat descriptor is transferred once to File.
fn write_stage(directory: &std::fs::File, bytes: &[u8]) -> Result<()> {
    use std::io::Write as _;
    use std::os::fd::{AsRawFd as _, FromRawFd as _};
    let name = c"payload";
    // SAFETY: fixed name, held restrictive directory, exclusive/no-follow create.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(recovery_required());
    }
    // SAFETY: exactly one fresh successful native descriptor owner.
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    file.write_all(bytes).and_then(|()| file.sync_all()).map_err(|_| recovery_required())
}

/// Atomic no-replace create or exact original-inode exchange, never a copy/delete backup.
/// External writers remain subject to the existing final-rename race assumption.
#[cfg(unix)]
#[allow(unsafe_code)] // held descriptors and both fixed single-component names remain live.
fn atomic_move(
    from: &std::fs::File,
    from_name: &str,
    to: &std::fs::File,
    to_name: &str,
    exchange: bool,
) -> Result<()> {
    use std::os::fd::AsRawFd as _;
    let left = std::ffi::CString::new(from_name).map_err(|_| recovery_required())?;
    let right = std::ffi::CString::new(to_name).map_err(|_| recovery_required())?;
    #[cfg(target_os = "linux")]
    // SAFETY: two live held directories, single-component NUL-terminated names.
    let code = unsafe {
        libc::renameat2(
            from.as_raw_fd(),
            left.as_ptr(),
            to.as_raw_fd(),
            right.as_ptr(),
            if exchange { libc::RENAME_EXCHANGE } else { libc::RENAME_NOREPLACE },
        )
    };
    #[cfg(target_os = "macos")]
    // SAFETY: same held-descriptor/name invariants as the Linux qualified port.
    let code = unsafe {
        libc::renameatx_np(
            from.as_raw_fd(),
            left.as_ptr(),
            to.as_raw_fd(),
            right.as_ptr(),
            if exchange { libc::RENAME_SWAP } else { libc::RENAME_EXCL },
        )
    };
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let code = {
        let _ = (from, to, exchange);
        -1
    };
    if code != 0 {
        return Err(unavailable());
    }
    Ok(())
}

/// Delete only an exact owned regular single-link generation, preserving foreign writes.
#[cfg(unix)]
#[allow(unsafe_code)] // independently revalidated owned ID/hash, held parent, fixed leaf.
fn unlink_owned(
    parent: &std::fs::File,
    name: &str,
    expected: &ObjectFacts,
    limit: usize,
    deadline: Instant,
) -> Result<()> {
    use std::os::fd::AsRawFd as _;
    if read_facts(parent, name, limit)?.as_ref() != Some(expected) {
        return Err(recovery_required());
    }
    settlement_check(deadline)?;
    let name = std::ffi::CString::new(name).map_err(|_| recovery_required())?;
    // SAFETY: exact owned generation revalidated before the final external-writer window.
    if unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) } != 0 {
        return Err(recovery_required());
    }
    Ok(())
}

/// Remove only one exact owned empty directory; never recurse or delete by public path.
#[cfg(unix)]
#[allow(unsafe_code)] // actual ID equality and empty-only AT_REMOVEDIR operation.
fn remove_empty_owned(
    parent: &std::fs::File,
    name: &str,
    expected: (u64, u64),
    deadline: Instant,
) -> Result<()> {
    use std::os::fd::AsRawFd as _;
    if observed_directory(parent, name)? != Some(expected) {
        return Err(recovery_required());
    }
    settlement_check(deadline)?;
    let name = std::ffi::CString::new(name).map_err(|_| recovery_required())?;
    // SAFETY: one held-parent leaf, identity checked, kernel refuses nonempty directory.
    if unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
        return Err(recovery_required());
    }
    Ok(())
}

/// Unexecuted controls for moved planning facts, phase safety and owned native semantics.
#[cfg(test)]
mod tests {
    use super::super::super::preparation::{Interruption, NoopControl};
    use super::*;

    /// Canonical pending operation bytes supplied by Root before durable acceptance.
    fn reply(id: &str) -> Vec<u8> {
        let time = timestamp(unix_seconds().unwrap()).unwrap();
        serde_json::to_vec(&json!({"operation_id":id,"kind":"bundle-restore","state":"pending","created_at":time,"updated_at":time,"cancel_requested":false,"write_outcome":"unmeasured","progress":Value::Null,"result":Value::Null,"error":Value::Null,"cleanup_state":"unmeasured"})).unwrap()
    }

    /// One valid inert policy-source plus normalized authorial index-last new generation.
    fn proposed(path: &str, bytes: &[u8]) -> Vec<(String, Vec<u8>, String)> {
        let index=serde_json::to_vec(&json!({"schema_version":"forge.workspace/2","label":"Owned","resources":[{"key":"policy","role":"policy-source","path":path}]})).unwrap();
        vec![
            (path.into(), bytes.to_vec(), crate::hashing::sha256_hex(bytes)),
            (INDEX_PATH.into(), index.clone(), crate::hashing::sha256_hex(&index)),
        ]
    }

    /// Owned private root and state descriptors; production has no path override.
    #[cfg(unix)]
    fn fixture() -> (tempfile::TempDir, tempfile::TempDir, Root, TransactionState) {
        use std::os::unix::fs::PermissionsExt as _;
        let project = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir().unwrap();
        std::fs::set_permissions(private.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let root = Root::open(project.path()).unwrap();
        let state = super::super::transaction_state::owned_test_state(
            &root,
            std::fs::File::open(private.path()).unwrap(),
        )
        .unwrap();
        (project, private, root, state)
    }

    /// No per-file multiplication of the shared diff byte ceiling and no invalid UTF8 split.
    #[test]
    fn shared_diff_budget_is_utf8_bytes_and_binary_has_no_text() {
        let mut remaining = 19;
        let (first, binary, truncated) =
            preview_diff(b"old", "éééééééééé".as_bytes(), &mut remaining);
        assert!(!binary);
        assert!(truncated);
        assert!(std::str::from_utf8(first.as_bytes()).is_ok());
        assert!(first.len() <= 19);
        let (second, _, again) = preview_diff(b"more", b"replacement", &mut remaining);
        assert!(again);
        assert!(first.len() + second.len() <= 19);
        let (text, binary, _) = preview_diff(&[255], b"new", &mut remaining);
        assert!(binary);
        assert!(text.is_empty());
    }

    /// The same path's old and new generations both consume checked native retention.
    #[test]
    fn generation_charge_is_checked_without_path_dedup_or_overflow() {
        assert_eq!(private_charge(12).unwrap(), JOURNAL_RESERVE + 24);
        assert!(private_charge(usize::MAX).is_err());
        assert!(add(MAX_CAPTURE_BYTES, 1).is_err());
    }

    /// Actual unregistered base identity is retained, with explicit new parent absence.
    #[cfg(unix)]
    #[test]
    fn planner_moves_real_unregistered_base_and_does_not_create_directories() {
        let (project, _private, root, _state) = fixture();
        std::fs::write(project.path().join("policy.md"), b"old").unwrap();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        assert_eq!(plan.input_views()[0]["current_sha256"], crate::hashing::sha256_hex(b"old"));
        assert_eq!(plan.target_views()[0]["status"], "overwrite");
        assert!(plan.targets[0].base.is_some());
        let plan = root
            .plan_restore_targets(
                proposed("new/sub/policy.md", b"new"),
                Vec::new(),
                &mut NoopControl,
            )
            .unwrap();
        assert_eq!(plan.directory_views().len(), 2);
        assert!(!project.path().join("new").exists());
    }

    /// Whole union/aliases/index-last/fake-hash rejection happens before target creation.
    #[cfg(unix)]
    #[test]
    fn planner_rejects_incomplete_index_and_aliases_without_effects() {
        let (project, _private, root, _state) = fixture();
        let mut targets = proposed("policy.md", b"new");
        targets[0].2 = "a".repeat(64);
        assert!(root.plan_restore_targets(targets, Vec::new(), &mut NoopControl).is_err());
        let mut targets = proposed("policy.md", b"new");
        targets.swap(0, 1);
        assert!(root.plan_restore_targets(targets, Vec::new(), &mut NoopControl).is_err());
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 0);
    }

    /// Exact accepted reply is persisted before publication, not fabricated at worker start.
    #[cfg(unix)]
    #[test]
    fn acceptance_persists_reply_and_no_source_effect_before_dispatch() {
        let (project, _private, root, state) = fixture();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let original = reply("op_aaaaaaaaaaaa");
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_aaaaaaaaaaaa".into(),
                7,
                "a".repeat(64),
                "b".repeat(64),
                original.clone(),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        assert_eq!(accepted.accepted_reply_bytes(), original);
        assert_eq!(state.observe_outcome("op_aaaaaaaaaaaa").unwrap()["state"], "pending");
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 0);
        assert!(state.require_access().is_err());
    }

    /// Forward cooperative interruption is distinct from the separate settlement deadline.
    struct StopControl;
    impl WorkControl for StopControl {
        /// Refuse forward work before a syscall or resource-creation phase.
        fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        }
        /// Retain the original cancellation rather than inventing a renewed deadline.
        fn interruption(&self) -> Option<Interruption> {
            Some(Interruption::CancelRequested)
        }
    }

    /// Cancellation before staging independently proves original absences and cleanup.
    #[cfg(unix)]
    #[test]
    fn pre_stage_cancel_has_none_outcome_without_public_files() {
        let (project, _private, root, state) = fixture();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_bbbbbbbbbbbb".into(),
                8,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_bbbbbbbbbbbb"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let result = root.run_restore(plan, &state, accepted, &mut StopControl).unwrap();
        assert_eq!(result.operation["write_outcome"], "none");
        assert_eq!(result.operation["cleanup_state"], "verified");
        assert!(!result.access_blocked);
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 0);
    }

    /// Valid native candidate exchanges exact old inode, publishes index last and cleans.
    /// This source-only test is not Unix/platform delivery evidence until Root executes it.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn owned_success_verifies_all_targets_and_terminal_cleanup() {
        let (project, _private, root, state) = fixture();
        std::fs::write(project.path().join("policy.md"), b"old").unwrap();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_cccccccccccc".into(),
                9,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_cccccccccccc"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let result = root.run_restore(plan, &state, accepted, &mut NoopControl).unwrap();
        assert_eq!(result.operation["write_outcome"], "committed");
        assert_eq!(result.operation["cleanup_state"], "verified");
        assert!(!result.access_blocked);
        assert_eq!(std::fs::read(project.path().join("policy.md")).unwrap(), b"new");
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 2);
    }

    /// Foreign replacement after planning is preserved; no stale base gains write authority.
    #[cfg(unix)]
    #[test]
    fn changed_unregistered_base_is_preserved_by_settlement() {
        let (project, _private, root, state) = fixture();
        std::fs::write(project.path().join("policy.md"), b"old").unwrap();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_dddddddddddd".into(),
                10,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_dddddddddddd"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        std::fs::write(project.path().join("policy.md"), b"foreign").unwrap();
        let result = root.run_restore(plan, &state, accepted, &mut NoopControl).unwrap();
        assert_eq!(result.operation["state"], "recovery-required");
        assert!(result.access_blocked);
        assert_eq!(std::fs::read(project.path().join("policy.md")).unwrap(), b"foreign");
    }

    /// Observe actual durable progress and cancel only after the first owned publication.
    struct AfterPublish<'a> {
        /// Actual state used by the engine, not a hardcoded checkpoint-count oracle.
        state: &'a TransactionState,
    }
    impl WorkControl for AfterPublish<'_> {
        /// Stop at the next cooperative fence after an actual published target fact.
        fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
            if self
                .state
                .records()
                .unwrap()
                .iter()
                .any(|record| record.targets.iter().any(|target| target.published.is_some()))
            {
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            } else {
                Ok(())
            }
        }
        /// The checkpoint alone establishes the source-bound cancellation observation.
        fn interruption(&self) -> Option<Interruption> {
            None
        }
    }

    /// Conditional exchange rollback restores the original inode before the index publishes.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn cancellation_after_first_publication_restores_exact_original_inode() {
        let (project, _private, root, state) = fixture();
        std::fs::write(project.path().join("policy.md"), b"old").unwrap();
        let original = root.read("policy.md", 10).unwrap();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_eeeeeeeeeeee".into(),
                11,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_eeeeeeeeeeee"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let result =
            root.run_restore(plan, &state, accepted, &mut AfterPublish { state: &state }).unwrap();
        assert_eq!(result.operation["write_outcome"], "none");
        assert_eq!(result.operation["state"], "cancelled");
        assert!(!result.access_blocked);
        let restored = root.read("policy.md", 10).unwrap();
        assert_eq!(restored.identity, original.identity);
        assert_eq!(restored.sha256, original.sha256);
        assert!(!project.path().join(INDEX_PATH).exists());
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 1);
    }

    /// An entirely uncreated parent suffix remains verified absent after pre-stage cancel.
    #[cfg(unix)]
    #[test]
    fn pre_stage_cancel_with_missing_parents_does_not_invent_cleanup_uncertainty() {
        let (project, _private, root, state) = fixture();
        let plan = root
            .plan_restore_targets(
                proposed("new/sub/policy.md", b"new"),
                Vec::new(),
                &mut NoopControl,
            )
            .unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_ffffffffffff".into(),
                12,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_ffffffffffff"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let result = root.run_restore(plan, &state, accepted, &mut StopControl).unwrap();
        assert_eq!(result.operation["write_outcome"], "none");
        assert_eq!(result.operation["cleanup_state"], "verified");
        assert!(!result.access_blocked);
        assert!(!project.path().join("new").exists());
    }

    /// A real durable commit replacement followed by a verification error never
    /// authorizes live stale rollback; fresh trusted recovery keeps the new bytes.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn post_commit_decision_native_journal_error_preserves_published_state() {
        let (project, private, root, ordinary) = fixture();
        drop(ordinary);
        let state = super::super::transaction_state::owned_test_state_after_commit_decision_fault(
            &root,
            std::fs::File::open(private.path()).unwrap(),
        )
        .unwrap();
        std::fs::write(project.path().join("policy.md"), b"old").unwrap();
        let proposed = proposed("policy.md", b"new");
        let new_index = proposed.last().unwrap().1.clone();
        let plan = root.plan_restore_targets(proposed, Vec::new(), &mut NoopControl).unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_a1a1a1a1a1a1".into(),
                13,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_a1a1a1a1a1a1"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let result = root.run_restore(plan, &state, accepted, &mut NoopControl).unwrap();
        assert!(result.access_blocked);
        assert_eq!(result.operation["state"], "recovery-required");
        assert_eq!(result.operation["write_outcome"], "unknown");
        assert_eq!(result.operation["cleanup_state"], "unverified");
        assert!(state.records().is_err());
        assert!(state.require_access().is_err());
        assert_eq!(std::fs::read(project.path().join("policy.md")).unwrap(), b"new");
        assert_eq!(std::fs::read(project.path().join(INDEX_PATH)).unwrap(), new_index);
        assert!(std::fs::read_dir(project.path()).unwrap().count() > 2);
        drop(state);
        let recovered = super::super::transaction_state::owned_test_state(
            &root,
            std::fs::File::open(private.path()).unwrap(),
        )
        .unwrap();
        let records = recovered.records().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].phase, Phase::CommitDecided);
        assert_eq!(records[0].write_outcome, WriteOutcome::Committed);
        assert_eq!(
            recovered.observe_outcome("op_a1a1a1a1a1a1").unwrap()["write_outcome"],
            "committed"
        );
        recovered.recover(&root, true).unwrap();
        assert!(recovered.require_access().is_ok());
        assert_eq!(
            recovered.observe_outcome("op_a1a1a1a1a1a1").unwrap()["cleanup_state"],
            "verified"
        );
        assert_eq!(std::fs::read(project.path().join("policy.md")).unwrap(), b"new");
        assert_eq!(std::fs::read(project.path().join(INDEX_PATH)).unwrap(), new_index);
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 2);
    }

    /// Independently verified original state remains internal while cleanup is running;
    /// the closed public operation stays unmeasured until terminal or recovery state.
    #[cfg(unix)]
    #[test]
    fn intermediate_none_cleanup_projects_unmeasured_until_terminal_or_recovery() {
        let (_project, _private, root, state) = fixture();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let _accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_b2b2b2b2b2b2".into(),
                14,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_b2b2b2b2b2b2"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let mut record = state.records().unwrap().remove(0);
        root.verify_original(&record).unwrap();
        record.phase = Phase::RollingBack;
        record.write_outcome = WriteOutcome::None;
        state.update(&record).unwrap();
        let observed = state.observe_outcome("op_b2b2b2b2b2b2").unwrap();
        assert_eq!(state.records().unwrap()[0].write_outcome, WriteOutcome::None);
        assert_eq!(observed.as_object().unwrap().len(), 11);
        assert_eq!(observed["state"], "running");
        assert_eq!(observed["write_outcome"], "unmeasured");
        assert_eq!(observed["cleanup_state"], "unmeasured");
        assert!(observed["result"].is_null());
        assert!(observed["error"].is_null());
        record.phase = Phase::RecoveryRequired;
        state.update(&record).unwrap();
        let observed = state.observe_outcome("op_b2b2b2b2b2b2").unwrap();
        assert_eq!(observed["state"], "recovery-required");
        assert_eq!(observed["write_outcome"], "none");
        assert_eq!(observed["cleanup_state"], "unverified");
        assert!(observed["result"].is_null());
        assert!(!observed["error"].is_null());
    }

    /// Failed lease admission defers only its exact accepted owner, leaves project
    /// bytes untouched, and permits no worker publication before trusted recovery.
    #[cfg(unix)]
    #[test]
    fn accepted_lease_deferral_queries_and_reopens_without_project_effects() {
        let (project, private, root, state) = fixture();
        std::fs::write(project.path().join("policy.md"), b"old").unwrap();
        let old = root.read("policy.md", 10).unwrap();
        let plan = root
            .plan_restore_targets(proposed("policy.md", b"new"), Vec::new(), &mut NoopControl)
            .unwrap();
        let accepted = root
            .accept_restore(
                &plan,
                &state,
                "op_c3c3c3c3c3c3".into(),
                15,
                "a".repeat(64),
                "b".repeat(64),
                reply("op_c3c3c3c3c3c3"),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        let foreign = AcceptedRestore {
            operation_id: accepted.operation_id.clone(),
            nonce: accepted.nonce + 1,
            deadline: accepted.deadline,
            accepted_reply: accepted.accepted_reply.clone(),
        };
        assert!(root.defer_restore_recovery(&state, &foreign).is_err());
        let mut foreign = AcceptedRestore {
            operation_id: accepted.operation_id.clone(),
            nonce: accepted.nonce,
            deadline: accepted.deadline,
            accepted_reply: accepted.accepted_reply.clone(),
        };
        foreign.accepted_reply.push(b' ');
        assert!(root.defer_restore_recovery(&state, &foreign).is_err());
        assert_eq!(state.observe_outcome("op_c3c3c3c3c3c3").unwrap()["state"], "pending");
        root.defer_restore_recovery(&state, &accepted).unwrap();
        let observed = state.observe_outcome("op_c3c3c3c3c3c3").unwrap();
        assert_eq!(observed["state"], "recovery-required");
        assert_eq!(observed["write_outcome"], "unknown");
        assert_eq!(observed["cleanup_state"], "unverified");
        assert!(observed["result"].is_null());
        assert!(!observed["error"].is_null());
        assert!(state.require_access().is_err());
        assert!(root.defer_restore_recovery(&state, &accepted).is_err());
        assert!(root.run_restore(plan, &state, accepted, &mut NoopControl).is_err());
        let unchanged = root.read("policy.md", 10).unwrap();
        assert_eq!(unchanged.identity, old.identity);
        assert_eq!(unchanged.sha256, old.sha256);
        assert!(!project.path().join(INDEX_PATH).exists());
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 1);
        drop(state);
        let recovered = super::super::transaction_state::owned_test_state(
            &root,
            std::fs::File::open(private.path()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            recovered.observe_outcome("op_c3c3c3c3c3c3").unwrap()["state"],
            "recovery-required"
        );
        assert!(recovered.recover(&root, false).is_err());
        assert_eq!(std::fs::read(project.path().join("policy.md")).unwrap(), b"old");
        recovered.recover(&root, true).unwrap();
        assert!(recovered.require_access().is_ok());
        let observed = recovered.observe_outcome("op_c3c3c3c3c3c3").unwrap();
        assert_eq!(observed["write_outcome"], "none");
        assert_eq!(observed["cleanup_state"], "verified");
        assert_eq!(root.read("policy.md", 10).unwrap().identity, old.identity);
        assert!(!project.path().join(INDEX_PATH).exists());
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 1);
    }
}

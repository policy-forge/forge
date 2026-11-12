//! Dry-run plan, apply, and reconcile state machine (PRD 065 M-3/M-6..M-11/M-14).
//!
//! Planning is a pure local projection of a change set plus connector state.
//! Apply requires the exact unexpired plan (matching change-set and config
//! hashes), enforces the target allowlist, retries idempotently, and records
//! per-operation outcomes. Reconcile treats remote state as observations only:
//! it never mutates FORGE source status, approval, risk, or closure, and it
//! never rewrites local connector bindings. Audit reports are redacted and
//! deterministic functions of the plan and transport outcomes.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::adapter::{
    OperationKind, OperationRequest, Payload, Precondition, RemoteObject, Transport, TransportError,
};
use super::change_set::{ChangeSet, WorkItem};
use super::config::ConnectorConfig;
use super::fake::idempotency_key;
use super::{error, read_bounded, reject_secret_text, render_json};
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};

/// Versioned dry-run plan format.
pub const PLAN_SCHEMA_VERSION: &str = "forge.integration-plan/1";
/// Versioned local connector state format.
pub const STATE_SCHEMA_VERSION: &str = "forge.integration-state/1";
/// Versioned redacted audit report format.
pub const AUDIT_SCHEMA_VERSION: &str = "forge.integration-audit/1";
/// Raw plan/state/audit byte ceiling.
pub const MAX_DOCUMENT_BYTES: u64 = 10 * 1024 * 1024;
/// Default plan validity window in hours.
pub const DEFAULT_TTL_HOURS: i64 = 24;
/// Maximum transport attempts per operation, including the first (M-8/M-10).
pub const MAX_ATTEMPTS: u32 = 3;

/// One planned operation with its content hash and remote precondition (M-6).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlannedOperation {
    /// Stable FORGE work-item ID.
    pub work_item_id: String,
    /// Allowlisted `owner/repo` target.
    pub target: String,
    /// Desired remote operation.
    pub kind: OperationKind,
    /// Stable idempotency key (M-8).
    pub idempotency_key: String,
    /// Minimized payload (M-12).
    pub payload: Payload,
    /// Lowercase SHA-256 of the canonical payload JSON.
    pub payload_sha256: String,
    /// Remote version precondition for update/close-request (M-9).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precondition: Option<Precondition>,
}

/// Complete dry-run plan bound to exact change-set and config bytes (M-6).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// Exact [`PLAN_SCHEMA_VERSION`].
    pub schema_version: String,
    /// SHA-256 of the exact rendered change-set bytes this plan was built from.
    pub change_set_sha256: String,
    /// SHA-256 of the exact rendered connector config bytes.
    pub config_sha256: String,
    /// Explicit plan creation time.
    pub created_at: DateTime<Utc>,
    /// Explicit plan expiry; apply refuses expired plans (M-6).
    pub expires_at: DateTime<Utc>,
    /// Operations sorted by work-item ID.
    pub operations: Vec<PlannedOperation>,
}

impl Plan {
    /// Parse and validate a bounded plan document, recomputing payload hashes.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] for malformed, tampered, or invalid plans.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        let plan: Self = parse_document(bytes, "integration plan")?;
        if plan.schema_version != PLAN_SCHEMA_VERSION {
            return Err(error(format!(
                "unsupported schema_version '{}'; expected {PLAN_SCHEMA_VERSION}",
                plan.schema_version
            )));
        }
        if plan.expires_at <= plan.created_at {
            return Err(error("plan expiry must be after creation"));
        }
        for operation in &plan.operations {
            verify_operation(operation)?;
        }
        Ok(plan)
    }

    /// Render deterministic JSON bytes with one trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when serialization fails.
    pub fn render(&self) -> Result<String, ForgeError> {
        render_json(self, "plan")
    }
}

/// One recorded FORGE-to-remote identity binding. Contains no credentials (M-5).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RemoteBinding {
    /// Stable FORGE work-item ID.
    pub work_item_id: String,
    /// Allowlisted target the object lives in.
    pub target: String,
    /// Provider-assigned remote object ID.
    pub remote_id: String,
    /// Remote version after the last successful apply.
    pub remote_version: u64,
    /// Payload hash of the last successful apply.
    pub last_payload_sha256: String,
}

/// Local connector state mapping FORGE IDs to recorded remote identities.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConnectorState {
    /// Exact [`STATE_SCHEMA_VERSION`].
    pub schema_version: String,
    /// Bindings keyed by work-item ID.
    pub bindings: BTreeMap<String, RemoteBinding>,
}

impl ConnectorState {
    /// An empty state document.
    #[must_use]
    pub fn empty() -> Self {
        Self { schema_version: STATE_SCHEMA_VERSION.to_string(), bindings: BTreeMap::new() }
    }

    /// Parse and validate a bounded state document.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] for malformed or invalid state.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        let state: Self = parse_document(bytes, "integration state")?;
        if state.schema_version != STATE_SCHEMA_VERSION {
            return Err(error(format!(
                "unsupported schema_version '{}'; expected {STATE_SCHEMA_VERSION}",
                state.schema_version
            )));
        }
        for (key, binding) in &state.bindings {
            if key != &binding.work_item_id {
                return Err(error("state binding key does not match its work-item ID"));
            }
            reject_secret_text(&binding.remote_id, "state")?;
        }
        Ok(state)
    }

    /// Render deterministic JSON bytes with one trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when serialization fails.
    pub fn render(&self) -> Result<String, ForgeError> {
        render_json(self, "connector state")
    }
}

/// One redacted per-operation audit entry (M-10/M-14).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditEntry {
    /// Stable FORGE work-item ID.
    pub work_item_id: String,
    /// Target of the operation.
    pub target: String,
    /// Operation label.
    pub operation: String,
    /// Outcome: applied, failed, conflict, skipped-duplicate, target-refused,
    /// observed, drift, or missing.
    pub status: String,
    /// Closed error class, when the operation did not succeed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
    /// Redacted detail; never credentials or provider prose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Recorded remote identity, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_id: Option<String>,
    /// Recorded or observed remote version, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_version: Option<u64>,
    /// Observed remote state label (reconcile only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_state: Option<String>,
}

/// Deterministic redacted audit report for one plan/apply/reconcile run (M-14).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditReport {
    /// Exact [`AUDIT_SCHEMA_VERSION`].
    pub schema_version: String,
    /// Phase that produced the report: plan, apply, or reconcile.
    pub phase: String,
    /// SHA-256 of the plan the run was bound to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_sha256: Option<String>,
    /// Per-operation entries in plan order.
    pub entries: Vec<AuditEntry>,
}

impl AuditReport {
    /// Render deterministic JSON bytes with one trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when serialization fails.
    pub fn render(&self) -> Result<String, ForgeError> {
        render_json(self, "audit report")
    }
}

fn parse_document<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    label: &str,
) -> Result<T, ForgeError> {
    if bytes.len() as u64 > MAX_DOCUMENT_BYTES {
        return Err(error(format!("{label} exceeds the {MAX_DOCUMENT_BYTES} byte limit")));
    }
    let value = json_strict::parse_value(
        bytes,
        label,
        Limits { max_depth: 32, max_string_bytes: 64 * 1024 },
    )
    .map_err(|_| error(format!("invalid bounded duplicate-free {label} JSON")))?;
    serde_json::from_value(value)
        .map_err(|_| error(format!("{label} fields do not match the closed contract")))
}

/// Canonical payload hash shared by planner and verifier.
fn payload_hash(payload: &Payload) -> Result<String, ForgeError> {
    let bytes = serde_json::to_vec(payload)
        .map_err(|cause| error(format!("payload serialization failed: {cause}")))?;
    Ok(sha256_hex(&bytes))
}

fn verify_operation(operation: &PlannedOperation) -> Result<(), ForgeError> {
    let expected = payload_hash(&operation.payload)?;
    if expected != operation.payload_sha256 {
        return Err(error(format!(
            "payload hash mismatch for work item '{}'; plan was tampered with",
            operation.work_item_id
        )));
    }
    reject_secret_text(&operation.payload.title, "plan payload")?;
    reject_secret_text(&operation.payload.body, "plan payload")?;
    match operation.kind {
        OperationKind::Create => {
            if operation.precondition.is_some() {
                return Err(error("create operations must not carry a precondition"));
            }
        }
        OperationKind::Update | OperationKind::CloseRequest => {
            if operation.precondition.is_none() {
                return Err(error(
                    "update/close-request operations require a version precondition",
                ));
            }
        }
    }
    Ok(())
}

/// Deterministic minimized payload body for one work item (M-12).
fn render_body(item: &WorkItem) -> String {
    use std::fmt::Write as _;
    let mut body = format!(
        "FORGE work item: {}\nKind: {}\nKey: {}\nReason codes: {}\nOwners: {}\nSource: {} (sha256 {})\n",
        item.id,
        item.kind.as_str(),
        item.key,
        item.reason_codes.join(", "),
        item.owners.join(", "),
        item.provenance.source,
        item.provenance.sha256,
    );
    if let Some(due) = &item.due_date {
        let _ = writeln!(body, "Due: {due}");
    }
    if let Some(excerpt) = &item.excerpt {
        let _ = writeln!(body, "Excerpt (opt-in): {excerpt}");
    }
    body.push_str("Remote state is an observation only; FORGE approval, risk and closure are never changed by this ticket.");
    body
}

/// Build a dry-run plan from a validated change set and current connector state.
///
/// Every item targets the first allowlisted repository (Phase 1 single-target
/// reference semantics). Items with an existing binding become updates under
/// the recorded version precondition; new items become creates with stable
/// idempotency keys. Source artifacts are re-verified against their recorded
/// hashes before any operation is planned (M-3).
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] for source drift, invalid inputs, or
/// credential-shaped payloads.
#[allow(clippy::too_many_arguments)] // Plan identity binds config bytes, change-set bytes and state together.
pub fn build_plan(
    root: &Path,
    config: &ConnectorConfig,
    config_bytes: &[u8],
    change_set: &ChangeSet,
    change_set_bytes: &[u8],
    state: &ConnectorState,
    now: DateTime<Utc>,
    ttl_hours: i64,
) -> Result<Plan, ForgeError> {
    config.validate()?;
    if ttl_hours <= 0 || ttl_hours > 24 * 30 {
        return Err(error("ttl_hours must be within 1..=720"));
    }
    verify_sources(root, change_set)?;
    let target = config.targets[0].clone();
    let mut operations = Vec::new();
    for item in &change_set.items {
        let payload = Payload { title: item.title.clone(), body: render_body(item) };
        reject_secret_text(&payload.title, "payload")?;
        reject_secret_text(&payload.body, "payload")?;
        let hash = payload_hash(&payload)?;
        let (kind, precondition) = match state.bindings.get(&item.id) {
            Some(binding) => (
                OperationKind::Update,
                Some(Precondition {
                    remote_id: binding.remote_id.clone(),
                    remote_version: binding.remote_version,
                }),
            ),
            None => (OperationKind::Create, None),
        };
        operations.push(PlannedOperation {
            work_item_id: item.id.clone(),
            target: target.clone(),
            kind,
            idempotency_key: idempotency_key(&target, &item.id),
            payload,
            payload_sha256: hash,
            precondition,
        });
    }
    operations.sort_by(|left, right| left.work_item_id.cmp(&right.work_item_id));
    Ok(Plan {
        schema_version: PLAN_SCHEMA_VERSION.to_string(),
        change_set_sha256: sha256_hex(change_set_bytes),
        config_sha256: sha256_hex(config_bytes),
        created_at: now,
        expires_at: now + Duration::hours(ttl_hours),
        operations,
    })
}

/// Re-verify every provenance source file against its recorded hash and schema
/// version before planning or applying (M-3).
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when a source drifted or cannot be read.
pub fn verify_sources(root: &Path, change_set: &ChangeSet) -> Result<(), ForgeError> {
    for item in &change_set.items {
        let path = root.join(&item.provenance.source);
        let bytes = read_bounded(&path, MAX_DOCUMENT_BYTES)?;
        if sha256_hex(&bytes) != item.provenance.sha256 {
            return Err(error(format!(
                "source '{}' changed since export; re-export the change set",
                item.provenance.source
            )));
        }
        let value = json_strict::parse_value(
            &bytes,
            "source artifact",
            Limits { max_depth: 64, max_string_bytes: 64 * 1024 },
        )
        .map_err(|_| error(format!("source '{}' is not valid JSON", item.provenance.source)))?;
        if value["schema_version"].as_str() != Some(item.provenance.schema_version.as_str()) {
            return Err(error(format!(
                "source '{}' schema version drifted from provenance",
                item.provenance.source
            )));
        }
    }
    Ok(())
}

/// Load a plan from disk.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the file cannot be read or is invalid.
pub fn load_plan(path: &Path) -> Result<Plan, ForgeError> {
    Plan::parse(&read_bounded(path, MAX_DOCUMENT_BYTES)?)
}

/// Load connector state from disk; a missing file yields an empty state.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when an existing file is invalid.
pub fn load_state(path: &Path) -> Result<ConnectorState, ForgeError> {
    if !path.exists() {
        return Ok(ConnectorState::empty());
    }
    ConnectorState::parse(&read_bounded(path, MAX_DOCUMENT_BYTES)?)
}

/// Verify that a plan is the exact, unexpired plan for the supplied change set
/// and config (M-6). Called by apply before any transport activity.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the plan is expired or drifted.
pub fn verify_plan_binding(
    plan: &Plan,
    config: &ConnectorConfig,
    config_bytes: &[u8],
    change_set_bytes: &[u8],
    now: DateTime<Utc>,
) -> Result<(), ForgeError> {
    if now > plan.expires_at {
        return Err(error("dry-run plan has expired; re-run plan before apply"));
    }
    if now < plan.created_at {
        return Err(error("dry-run plan was created in the future relative to --as-of"));
    }
    if plan.change_set_sha256 != sha256_hex(change_set_bytes) {
        return Err(error("change set does not match the dry-run plan; re-run plan"));
    }
    if plan.config_sha256 != sha256_hex(config_bytes) {
        return Err(error("connector config does not match the dry-run plan; re-run plan"));
    }
    config.validate()
}

/// Summary shown before mutation; apply requires explicit confirmation (M-7).
#[must_use]
pub fn confirmation_summary(
    config: &ConnectorConfig,
    plan: &Plan,
    change_set: &ChangeSet,
) -> String {
    let creates = plan.operations.iter().filter(|op| op.kind == OperationKind::Create).count();
    let updates = plan.operations.iter().filter(|op| op.kind == OperationKind::Update).count();
    let closes = plan.operations.iter().filter(|op| op.kind == OperationKind::CloseRequest).count();
    let excerpts = change_set.items.iter().filter(|item| item.excerpt.is_some()).count();
    format!(
        "Connector: github-issues (fake Phase 1 transport)\n\
         Targets: {}\n\
         Operations: {} create, {} update, {} close-request\n\
         Sensitive fields: {} item(s) carry opt-in excerpts; all other payloads are minimized IDs/links\n\
         Irreversible effects: remote objects are created/updated; close-request asks the provider to close tickets.\n\
         FORGE source status, approval, risk and closure are never changed.\n\
         Re-run with --confirm to apply this exact plan.",
        config.targets.join(", "),
        creates,
        updates,
        closes,
        excerpts,
    )
}

/// Apply the exact unexpired plan through the transport, recording per-operation
/// outcomes and updating connector state only for successful operations (M-10).
///
/// Retries are idempotent: the same idempotency key is reused, and update
/// preconditions prevent silent last-write-wins after a lost response (M-8/M-9).
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the plan binding fails. Individual
/// operation failures are recorded in the audit report, not returned as errors.
#[allow(clippy::too_many_arguments)] // Apply re-verifies every plan binding input before mutation.
pub fn apply(
    config: &ConnectorConfig,
    config_bytes: &[u8],
    plan: &Plan,
    plan_bytes: &[u8],
    change_set_bytes: &[u8],
    state: &ConnectorState,
    transport: &mut dyn Transport,
    now: DateTime<Utc>,
) -> Result<(ConnectorState, AuditReport), ForgeError> {
    verify_plan_binding(plan, config, config_bytes, change_set_bytes, now)?;
    let mut next_state = state.clone();
    let mut entries = Vec::new();
    for operation in &plan.operations {
        let mut entry = AuditEntry {
            work_item_id: operation.work_item_id.clone(),
            target: operation.target.clone(),
            operation: operation.kind.as_str().to_string(),
            status: String::new(),
            error_class: None,
            detail: None,
            remote_id: operation.precondition.as_ref().map(|pre| pre.remote_id.clone()),
            remote_version: operation.precondition.as_ref().map(|pre| pre.remote_version),
            remote_state: None,
        };
        if !config.allows(&operation.target) {
            entry.status = "target-refused".to_string();
            entry.error_class = Some(TransportError::TargetRefused.class().to_string());
            entries.push(entry);
            continue;
        }
        // Reconcile before create: an existing binding means the object already
        // exists; never create a duplicate (M-8).
        if operation.kind == OperationKind::Create
            && let Some(binding) = state.bindings.get(&operation.work_item_id)
        {
            entry.status = "skipped-duplicate".to_string();
            entry.remote_id = Some(binding.remote_id.clone());
            entry.remote_version = Some(binding.remote_version);
            entries.push(entry);
            continue;
        }
        let request = OperationRequest {
            target: operation.target.clone(),
            idempotency_key: operation.idempotency_key.clone(),
            kind: operation.kind,
            payload: operation.payload.clone(),
            precondition: operation.precondition.clone(),
        };
        match execute_with_retry(transport, &config.network, &request) {
            Ok(remote) => {
                entry.status = "applied".to_string();
                entry.remote_id = Some(remote.remote_id.clone());
                entry.remote_version = Some(remote.remote_version);
                entry.remote_state = Some(remote.state.clone());
                next_state.bindings.insert(
                    operation.work_item_id.clone(),
                    RemoteBinding {
                        work_item_id: operation.work_item_id.clone(),
                        target: operation.target.clone(),
                        remote_id: remote.remote_id,
                        remote_version: remote.remote_version,
                        last_payload_sha256: operation.payload_sha256.clone(),
                    },
                );
            }
            Err(failure) => {
                entry.status = if matches!(failure, TransportError::Conflict { .. }) {
                    "conflict"
                } else {
                    "failed"
                }
                .to_string();
                entry.error_class = Some(failure.class().to_string());
                entry.detail = Some(failure.detail());
                if let TransportError::Conflict { observed, .. } = failure {
                    entry.remote_version = Some(observed);
                }
            }
        }
        entries.push(entry);
    }
    let report = AuditReport {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        phase: "apply".to_string(),
        plan_sha256: Some(sha256_hex(plan_bytes)),
        entries,
    };
    Ok((next_state, report))
}

/// Execute one operation with bounded idempotent retries (M-8/M-13).
fn execute_with_retry(
    transport: &mut dyn Transport,
    policy: &super::config::NetworkPolicy,
    request: &OperationRequest,
) -> Result<RemoteObject, TransportError> {
    let mut last = TransportError::Protocol("no attempt".to_string());
    for _ in 0..MAX_ATTEMPTS {
        match transport.execute(policy, request) {
            Ok(remote) => return Ok(remote),
            Err(TransportError::RateLimited { retry_after_secs }) => {
                // Bounded backoff; the fake returns 0 so fixtures stay fast.
                std::thread::sleep(std::time::Duration::from_millis(
                    retry_after_secs.saturating_mul(1000).min(5000),
                ));
                last = TransportError::RateLimited { retry_after_secs };
            }
            Err(failure @ (TransportError::Timeout | TransportError::ServerFailure { .. })) => {
                last = failure;
            }
            Err(failure) => return Err(failure),
        }
    }
    Err(last)
}

/// Reconcile recorded bindings against remote observations (M-11).
///
/// Remote state is reported as observations only. Neither FORGE source artifacts
/// nor the local connector state are mutated.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the config is invalid. Transport
/// failures are recorded per binding, not returned.
pub fn reconcile(
    config: &ConnectorConfig,
    state: &ConnectorState,
    transport: &mut dyn Transport,
) -> Result<AuditReport, ForgeError> {
    config.validate()?;
    let mut entries = Vec::new();
    for binding in state.bindings.values() {
        let mut entry = AuditEntry {
            work_item_id: binding.work_item_id.clone(),
            target: binding.target.clone(),
            operation: "observe".to_string(),
            status: String::new(),
            error_class: None,
            detail: None,
            remote_id: Some(binding.remote_id.clone()),
            remote_version: Some(binding.remote_version),
            remote_state: None,
        };
        if !config.allows(&binding.target) {
            entry.status = "target-refused".to_string();
            entry.error_class = Some(TransportError::TargetRefused.class().to_string());
            entries.push(entry);
            continue;
        }
        match transport.observe(&config.network, &binding.target, &binding.remote_id) {
            Ok(Some(remote)) => {
                entry.remote_state = Some(remote.state);
                entry.remote_version = Some(remote.remote_version);
                entry.status = if remote.remote_version == binding.remote_version {
                    "observed"
                } else {
                    "drift"
                }
                .to_string();
            }
            Ok(None) => {
                entry.status = "missing".to_string();
                entry.error_class = Some(TransportError::NotFound.class().to_string());
            }
            Err(failure) => {
                entry.status = "failed".to_string();
                entry.error_class = Some(failure.class().to_string());
                entry.detail = Some(failure.detail());
            }
        }
        entries.push(entry);
    }
    Ok(AuditReport {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        phase: "reconcile".to_string(),
        plan_sha256: None,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::change_set::{self, WorkItemKind};
    use crate::workflow::config::{ConnectorKind, NetworkPolicy};

    fn config() -> ConnectorConfig {
        ConnectorConfig {
            schema_version: super::super::config::CONFIG_SCHEMA_VERSION.to_string(),
            connector: ConnectorKind::GithubIssues,
            targets: vec!["policy-forge/forge".to_string()],
            network: NetworkPolicy::default(),
        }
    }

    fn work_item(key: &str) -> WorkItem {
        WorkItem {
            id: change_set::item_id(WorkItemKind::Gap, &"a".repeat(64), key),
            kind: WorkItemKind::Gap,
            key: key.to_string(),
            title: format!("gap {key}"),
            reason_codes: vec!["no-reviewed-mapping".to_string()],
            owners: vec!["owner".to_string()],
            due_date: None,
            provenance: change_set::Provenance {
                schema_version: "forge.applicability-report/1".to_string(),
                source: "reports/gaps.json".to_string(),
                sha256: "a".repeat(64),
            },
            excerpt: None,
        }
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-12T00:00:00Z").unwrap().with_timezone(&Utc)
    }

    #[test]
    fn plan_round_trip_and_tamper_detection() {
        let config = config();
        let config_bytes = config.render().unwrap();
        // build_plan re-verifies sources, so write the real artifact into a temp root.
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("reports")).unwrap();
        let source = serde_json::json!({"schema_version": "forge.applicability-report/1"});
        let source_bytes = serde_json::to_vec(&source).unwrap();
        std::fs::write(dir.path().join("reports/gaps.json"), &source_bytes).unwrap();
        let mut item = work_item("ac-1");
        item.provenance.sha256 = sha256_hex(&source_bytes);
        item.id = change_set::item_id(WorkItemKind::Gap, &item.provenance.sha256, "ac-1");
        let set = ChangeSet::new(vec![item]).unwrap();
        let set_bytes = set.render().unwrap();
        let plan = build_plan(
            dir.path(),
            &config,
            config_bytes.as_bytes(),
            &set,
            set_bytes.as_bytes(),
            &ConnectorState::empty(),
            now(),
            DEFAULT_TTL_HOURS,
        )
        .unwrap();
        let bytes = plan.render().unwrap();
        assert_eq!(bytes, plan.render().unwrap());
        let parsed = Plan::parse(bytes.as_bytes()).unwrap();
        assert_eq!(parsed, plan);
        // Tamper with the payload without updating the hash.
        let mut value = serde_json::to_value(&plan).unwrap();
        value["operations"][0]["payload"]["title"] = serde_json::json!("tampered");
        assert!(Plan::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn expired_or_drifted_plans_are_refused() {
        let config = config();
        let config_bytes = config.render().unwrap();
        let plan = Plan {
            schema_version: PLAN_SCHEMA_VERSION.to_string(),
            change_set_sha256: sha256_hex(b"set"),
            config_sha256: sha256_hex(config_bytes.as_bytes()),
            created_at: now(),
            expires_at: now() + Duration::hours(1),
            operations: Vec::new(),
        };
        assert!(
            verify_plan_binding(&plan, &config, config_bytes.as_bytes(), b"set", now()).is_ok()
        );
        assert!(
            verify_plan_binding(
                &plan,
                &config,
                config_bytes.as_bytes(),
                b"set",
                now() + Duration::hours(2)
            )
            .is_err()
        );
        assert!(
            verify_plan_binding(&plan, &config, config_bytes.as_bytes(), b"other", now()).is_err()
        );
    }

    #[test]
    fn state_round_trip_is_deterministic() {
        let mut state = ConnectorState::empty();
        state.bindings.insert(
            "id-1".to_string(),
            RemoteBinding {
                work_item_id: "id-1".to_string(),
                target: "a/b".to_string(),
                remote_id: "issue-1".to_string(),
                remote_version: 1,
                last_payload_sha256: "a".repeat(64),
            },
        );
        let bytes = state.render().unwrap();
        assert_eq!(bytes, state.render().unwrap());
        assert_eq!(ConnectorState::parse(bytes.as_bytes()).unwrap(), state);
    }
}

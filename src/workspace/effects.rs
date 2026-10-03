//! Session-owned exact-byte receipts, idempotent results, and single-file effects.
use super::contract::{self, Error, Result};
use super::preparation::ProgressUpdate;
use super::root::{Root, Target, conflict};
use super::services::{Snapshot, resource_id};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq as _;

const MAX_RETAINED: usize = 256;
const MAX_CONSUMED_INPUTS: usize = 100;
const MAX_PREVIEW_BYTES: usize = 20 * 1024 * 1024;
const RECEIPT_LIFETIME: Duration = Duration::from_secs(600);

pub(crate) struct Reply {
    pub value: Value,
    pub schema: &'static str,
    pub status: u16,
}
/// Server-authored export family; callers cannot select download media by filename.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArtifactFamily {
    /// Existing inert static review report.
    ReportHtml,
    /// Explicit index-and-hashes metadata, including sensitive labels and paths.
    MetadataJson,
    /// Exact source content, with a distinct fixed JSON download route.
    SourceBundleJson,
}
/// Session-bound proposed bytes and captured inputs for one explicit confirmed write.
struct Receipt {
    preview: Value,
    target: Target,
    bytes: Vec<u8>,
    snapshot_version: String,
    issued: Instant,
    used: bool,
    committed: bool,
    external: Vec<(String, super::root::Captured)>,
    /// Present only for a producer-owned export recipe.
    artifact_family: Option<ArtifactFamily>,
}
struct Replay {
    request_hash: String,
    value: Value,
    schema: &'static str,
    status: u16,
}
/// One off-lock preparation owner, bound to the same request hash as ready replays.
/// Pending preparations may drop their reservation; accepted restore attempts cannot.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    /// Unaccepted off-lock preparation with no durable intent.
    Preparation,
    /// One consumed source confirmation at the durable acceptance boundary.
    Restore,
}
/// In-flight idempotency reservation; its kind controls whether retries may release ownership.
struct PendingReplay {
    /// Governs safe retry/drop behavior independently of the canonical request.
    kind: PendingKind,
    /// Canonical method, public path, query and admitted Value fingerprint.
    request_hash: String,
    /// Unique generation prevents stale workers from removing newer state.
    nonce: String,
}
/// Bounded session receipts, operations, idempotency results and owned preparation reservations.
#[derive(Default)]
pub(crate) struct Store {
    receipts: BTreeMap<String, Receipt>,
    operations: BTreeMap<String, Value>,
    replays: BTreeMap<String, Replay>,
    /// Reserved keys share the existing replay capacity with terminal results.
    pending_replays: BTreeMap<String, PendingReplay>,
    /// Source capabilities/outcomes share all original retention ceilings.
    sources: source_receipts::SourceStore,
    retained_bytes: usize,
    /// Last complete capture count and immutable denominator for each active job.
    capture_progress: BTreeMap<String, (usize, usize)>,
}
#[path = "source_receipts.rs"]
mod source_receipts;
fn unavailable() -> Error {
    Error::new("not-found", "The session operation or preview was not found.", false)
}
fn capacity() -> Error {
    Error::new(
        "invalid-request",
        "Session retention limit reached. Finish pending work and start a new session.",
        false,
    )
}
/// Reuse the established closed error for request or reservation-owner mismatch.
fn idempotency_conflict() -> Error {
    Error::new(
        "idempotency-key-conflict",
        "The idempotency key already identifies a different request.",
        false,
    )
}
/// A prepared effect that consumes more inputs than the documented bound is a
/// request the caller resolves, not a session retention condition, so it never
/// reports the unrelated "start a new session" recovery.
fn too_many_inputs() -> Error {
    Error::new(
        "invalid-request",
        "The prepared effect consumes more than 100 inputs. Reduce the inputs this effect binds.",
        false,
    )
}
fn id(prefix: &str) -> Result<String> {
    Ok(format!("{prefix}_{}", *super::session::random_token()?))
}
fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

impl Store {
    /// Return the original reply or reject a conflicting/pending reserved request.
    pub(crate) fn replay(
        &self,
        key: &str,
        method: &str,
        path: &str,
        query: &str,
        request: &Value,
    ) -> Result<Option<Reply>> {
        if let Some(record) = self.pending_replays.get(key) {
            if record.request_hash != request_hash(method, path, query, request)? {
                return Err(idempotency_conflict());
            }
            if record.kind == PendingKind::Restore {
                return Err(Error::new(
                    "bundle-restore-in-progress",
                    "This consumed confirmation is unresolved. Inspect the preallocated operation ID before taking further action.",
                    false,
                ));
            }
            return Err(Error::new(
                "bundle-preparation-in-progress",
                "This bundle preparation is still running. Retry the same request shortly.",
                true,
            ));
        }
        if let Some(record) = self.replays.get(key) {
            if record.request_hash != request_hash(method, path, query, request)? {
                return Err(Error::new(
                    "idempotency-key-conflict",
                    "The idempotency key already identifies a different request.",
                    false,
                ));
            }
            return Ok(Some(Reply {
                value: record.value.clone(),
                schema: record.schema,
                status: record.status,
            }));
        }
        if self.replays.len().saturating_add(self.pending_replays.len()) >= MAX_RETAINED {
            return Err(capacity());
        }
        Ok(None)
    }
    /// Retain a ready ordinary reply without overwriting an off-lock reservation.
    pub(crate) fn remember(
        &mut self,
        key: &str,
        method: &str,
        path: &str,
        query: &str,
        request: &Value,
        reply: &Reply,
    ) -> Result<()> {
        if self.pending_replays.contains_key(key) {
            return Err(idempotency_conflict());
        }
        let hash = request_hash(method, path, query, request)?;
        self.replays.insert(
            key.to_owned(),
            Replay {
                request_hash: hash,
                value: reply.value.clone(),
                schema: reply.schema,
                status: reply.status,
            },
        );
        Ok(())
    }
    /// Reserve a fresh admitted request while the caller owns the shared Store lock.
    pub(crate) fn reserve(
        &mut self,
        key: &str,
        method: &str,
        path: &str,
        query: &str,
        request: &Value,
    ) -> Result<String> {
        if self.replay(key, method, path, query, request)?.is_some() {
            return Err(idempotency_conflict());
        }
        let nonce = id("prep")?;
        let request_hash = request_hash(method, path, query, request)?;
        self.pending_replays.insert(
            key.to_owned(),
            PendingReplay { request_hash, nonce: nonce.clone(), kind: PendingKind::Preparation },
        );
        Ok(nonce)
    }

    /// Release only this generation; a late worker never erases newer or ready state.
    pub(crate) fn release_reservation(&mut self, key: &str, nonce: &str) {
        if self.pending_replays.get(key).is_some_and(|pending| {
            pending.nonce == nonce && pending.kind == PendingKind::Preparation
        }) {
            self.pending_replays.remove(key);
        }
    }

    /// Charge a complete public wrapper conservatively before any shared retention.
    pub(crate) fn charge_reply(&mut self, reply: &Reply) -> Result<()> {
        let charge = contract::encode(&reply.value, 4 * 1024 * 1024, false)?
            .len()
            .checked_mul(4)
            .ok_or_else(capacity)?;
        let retained = self.retained_bytes.checked_add(charge).ok_or_else(capacity)?;
        if retained > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        self.retained_bytes = retained;
        Ok(())
    }

    /// Atomically transfer a fully validated local receipt and its original ready reply.
    /// The transport checks its sticky deadline after reacquiring this Store lock.
    pub(crate) fn retain_reserved(
        &mut self,
        key: &str,
        nonce: &str,
        mut local: Self,
        reply: &Reply,
    ) -> Result<()> {
        let pending = self.pending_replays.get(key).ok_or_else(idempotency_conflict)?;
        if pending.nonce != nonce || self.replays.contains_key(key) {
            return Err(idempotency_conflict());
        }
        let retained =
            self.retained_bytes.checked_add(local.retained_bytes).ok_or_else(capacity)?;
        if self.receipt_count().saturating_add(local.receipt_count()) > MAX_RETAINED
            || retained > MAX_PREVIEW_BYTES
            || !local.operations.is_empty()
            || local.sources.receipt_count() != 0
            || local.sources.operation_count() != 0
            || !local.replays.is_empty()
            || !local.pending_replays.is_empty()
            || local.receipts.keys().any(|id| self.receipts.contains_key(id))
        {
            return Err(capacity());
        }
        let retained_response = Replay {
            request_hash: pending.request_hash.clone(),
            value: reply.value.clone(),
            schema: reply.schema,
            status: reply.status,
        };
        self.retained_bytes = retained;
        self.receipts.append(&mut local.receipts);
        self.replays.insert(key.to_owned(), retained_response);
        self.pending_replays.remove(key);
        Ok(())
    }

    /// Admit one complete metadata plan, including every private binding, before mutation.
    pub(crate) fn preview_bundle(
        &mut self,
        plan: super::bundle_effects::PreparedBundle,
    ) -> Result<Value> {
        if plan.consumed_file_count > MAX_CONSUMED_INPUTS
            || plan.input_hashes.len() > MAX_CONSUMED_INPUTS
            || plan.external.len() > MAX_CONSUMED_INPUTS
        {
            return Err(too_many_inputs());
        }
        if self.receipt_count() >= MAX_RETAINED || plan.bytes.len() > 10 * 1024 * 1024 {
            return Err(capacity());
        }
        let preview_id = id("prev")?;
        let token = super::session::random_token()?;
        let expires = (chrono::Utc::now() + chrono::TimeDelta::seconds(600))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let (diff, truncated) = text_diff(
            plan.target.base.as_ref().map_or(&[], |base| base.bytes.as_slice()),
            &plan.bytes,
        );
        let preview = json!({"preview_id":preview_id,"operation_type":plan.kind,
            "target":{"path":plan.target.path,"status":if plan.target.base.is_some(){"overwrite"}else{"create"}},
            "base_sha256":plan.target.base.as_ref().map(|base|&base.sha256),"target_version":plan.target.version,
            "exact_bytes_sha256":crate::hashing::sha256_hex(&plan.bytes),"input_hashes":plan.input_hashes,
            "validation":super::services::validation(true,None),"semantic_summary":plan.semantic_summary,
            "diff_text":diff,"diff_truncated":truncated,"receipt":{"token":&*token,"expires_at":expires}});
        contract::validate_for(contract::ApiMajor::V2, "EffectPreview", &preview)?;
        let mut charge = plan
            .bytes
            .len()
            .checked_add(plan.target.base.as_ref().map_or(0, |base| base.bytes.len()))
            .and_then(|sum| {
                sum.checked_add(
                    contract::encode(&preview, 1024 * 1024, false).ok()?.len().checked_mul(4)?,
                )
            })
            .ok_or_else(capacity)?;
        for (_, captured) in &plan.external {
            charge = charge.checked_add(captured.bytes.len()).ok_or_else(capacity)?;
        }
        let retained = self.retained_bytes.checked_add(charge).ok_or_else(capacity)?;
        if retained > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        self.receipts.insert(
            preview_id,
            Receipt {
                preview: preview.clone(),
                target: plan.target,
                bytes: plan.bytes,
                snapshot_version: plan.snapshot_version,
                issued: Instant::now(),
                used: false,
                committed: false,
                external: plan.external,
                artifact_family: plan.artifact_family,
            },
        );
        self.retained_bytes = retained;
        Ok(preview)
    }

    /// Preserve ordinary single-file effect admission and its established preview shape.
    pub(crate) fn preview(
        &mut self,
        root: &Root,
        snapshot: &Snapshot,
        path: &str,
        kind: &str,
        bytes: Vec<u8>,
        inputs: &[&super::services::Item],
    ) -> Result<Value> {
        if inputs.len() > MAX_CONSUMED_INPUTS {
            return Err(too_many_inputs());
        }
        if self.receipt_count() >= MAX_RETAINED
            || bytes.len() > 10 * 1024 * 1024
            || self.retained_bytes.saturating_add(bytes.len()) > MAX_PREVIEW_BYTES
        {
            return Err(capacity());
        }
        let target = root.target(path)?;
        let preview_id = id("prev")?;
        let token = super::session::random_token()?;
        let expires = (chrono::Utc::now() + chrono::TimeDelta::seconds(600))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let (diff, diff_truncated) = text_diff(
            target.base.as_ref().map(|base| base.bytes.as_slice()).unwrap_or_default(),
            &bytes,
        );
        let preview = json!({"preview_id":preview_id,"operation_type":kind,"target":{"path":path,"status":if target.base.is_some(){"overwrite"}else{"create"}},
            "base_sha256":target.base.as_ref().map(|base|&base.sha256),"target_version":target.version,
            "exact_bytes_sha256":crate::hashing::sha256_hex(&bytes),"input_hashes":inputs.iter().map(|item|json!({"resource_id":resource_id(&item.registration),"sha256":item.captured.sha256})).collect::<Vec<_>>(),
            "validation":super::services::validation(true,None),"semantic_summary":semantic_summary(kind,&bytes),
            "diff_text":diff,"diff_truncated":diff_truncated,"receipt":{"token":&*token,"expires_at":expires}});
        contract::validate("EffectPreview", &preview)?;
        let retained = bytes
            .len()
            .saturating_add(target.base.as_ref().map_or(0, |base| base.bytes.len()))
            .saturating_add(
                contract::encode(&preview, 1024 * 1024, false)?.len().saturating_mul(4),
            );
        if self.retained_bytes.saturating_add(retained) > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        self.retained_bytes += retained;
        self.receipts.insert(
            preview_id,
            Receipt {
                preview: preview.clone(),
                target,
                bytes,
                snapshot_version: snapshot.version.clone(),
                issued: Instant::now(),
                used: false,
                committed: false,
                external: Vec::new(),
                artifact_family: (kind == "report-export").then_some(ArtifactFamily::ReportHtml),
            },
        );
        Ok(preview)
    }
    pub(crate) fn bind_external(
        &mut self,
        preview: &mut Value,
        registration: &super::index::Resource,
        captured: super::root::Captured,
    ) -> Result<()> {
        let id = preview["preview_id"].as_str().ok_or_else(Error::invalid)?.to_owned();
        if self.retained_bytes.saturating_add(captured.bytes.len()) > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        self.retained_bytes += captured.bytes.len();
        let receipt = self.receipts.get_mut(&id).ok_or_else(unavailable)?;
        preview["input_hashes"] =
            json!([{"resource_id":resource_id(registration),"sha256":captured.sha256}]);
        receipt.preview = preview.clone();
        receipt.external.push((registration.path.clone(), captured));
        Ok(())
    }
    pub(crate) fn get_preview(&self, id: &str) -> Result<Value> {
        let receipt = self.receipts.get(id).ok_or_else(unavailable)?;
        if receipt.issued.elapsed() > RECEIPT_LIFETIME {
            return Err(Error::new(
                "receipt-expired",
                "The preview expired. Prepare a new preview.",
                false,
            ));
        }
        if receipt.used {
            return Err(Error::new(
                "receipt-reused",
                "This preview has already been consumed.",
                false,
            ));
        }
        Ok(receipt.preview.clone())
    }
    pub(crate) fn operation(&self, id: &str) -> Result<Value> {
        self.operations.get(id).cloned().ok_or_else(unavailable)
    }
    pub(crate) fn conversion(&self, id: &str) -> Result<Value> {
        let op = self.operation(id)?;
        if op["kind"] != "conversion" || op["state"] != "succeeded" {
            return Err(unavailable());
        }
        Ok(op["result"].clone())
    }
    pub(crate) fn cancel(&mut self, id: &str) -> Result<Value> {
        let operation = self.operations.get_mut(id).ok_or_else(unavailable)?;
        if !matches!(operation["state"].as_str(), Some("pending" | "running")) {
            return Err(Error::new(
                "operation-not-cancellable",
                "This operation has already reached a terminal state.",
                false,
            ));
        }
        operation["cancel_requested"] = json!(true);
        operation["updated_at"] = json!(now());
        Ok(operation.clone())
    }
    /// Allocate one validated pending operation within the shared metadata/source record quota.
    pub(crate) fn begin(&mut self, kind: &str) -> Result<Value> {
        if self.operation_count() >= MAX_RETAINED {
            return Err(capacity());
        }
        let id = id("op")?;
        let operation = json!({"operation_id":id,"kind":kind,"state":"pending","created_at":now(),"updated_at":now(),"cancel_requested":false});
        contract::validate("Operation", &operation)?;
        self.operations.insert(id, operation.clone());
        Ok(operation)
    }
    /// Enter running once, settling an already cancelled pending job without work.
    pub(crate) fn running(&mut self, id: &str) -> Result<bool> {
        let operation = self.operations.get_mut(id).ok_or_else(unavailable)?;
        if !matches!(operation["state"].as_str(), Some("pending" | "running")) {
            return Ok(false);
        }
        operation["updated_at"] = json!(now());
        if operation["cancel_requested"] == true {
            operation["state"] = json!("cancelled");
            operation["progress"] = Value::Null;
            self.capture_progress.remove(id);
            return Ok(false);
        }
        operation["state"] = json!("running");
        Ok(true)
    }

    /// Publish only complete captured registrations for the original active job.
    /// A fixed denominator and nondecreasing prefix preserve truthful units;
    /// clearing the measured phase does not erase its internal last observation.
    pub(crate) fn observe_progress(&mut self, id: &str, update: ProgressUpdate) -> Result<bool> {
        let operation = self.operations.get_mut(id).ok_or_else(unavailable)?;
        if operation["state"] != "running" || operation["cancel_requested"] == true {
            return Ok(false);
        }
        match update {
            ProgressUpdate::Unchanged => return Ok(true),
            ProgressUpdate::Clear => operation["progress"] = Value::Null,
            ProgressUpdate::Capture { completed, total } => {
                if total > 1000
                    || completed > total
                    || self.capture_progress.get(id).is_some_and(|(previous, denominator)| {
                        total != *denominator || completed < *previous
                    })
                {
                    return Err(Error::invalid());
                }
                self.capture_progress.insert(id.to_owned(), (completed, total));
                operation["progress"] = json!({"completed_items":completed,"total_items":total});
            }
        }
        operation["updated_at"] = json!(now());
        Ok(true)
    }

    /// Settle an active job once and transfer its local receipts only on success.
    /// Cancellation wins before transfer; late workers cannot alter any terminal
    /// result, resurrect the job, or charge the same retained bytes twice.
    pub(crate) fn finish(
        &mut self,
        id: &str,
        prepared: Result<(Self, Reply)>,
        cancelled: bool,
    ) -> Result<()> {
        let mut operation = self.operations.get(id).cloned().ok_or_else(unavailable)?;
        if !matches!(operation["state"].as_str(), Some("pending" | "running")) {
            return Ok(());
        }
        operation["updated_at"] = json!(now());
        operation["progress"] = Value::Null;
        let mut retained = None;
        if cancelled || operation["cancel_requested"] == true {
            operation["state"] = json!("cancelled");
            operation["cancel_requested"] = json!(true);
        } else {
            let prepared = prepared.and_then(|(local, reply)| {
                if self.receipt_count().saturating_add(local.receipt_count()) > MAX_RETAINED
                    || self.retained_bytes.saturating_add(local.retained_bytes) > MAX_PREVIEW_BYTES
                {
                    return Err(capacity());
                }
                Ok((local, reply))
            });
            match prepared {
                Ok((local, reply)) => {
                    let mut result = reply.value["result"].clone();
                    if result.get("operation_id").is_some() {
                        result["operation_id"] = json!(id);
                    }
                    operation["state"] = json!("succeeded");
                    operation["result"] = result;
                    retained = Some(local);
                }
                Err(error) => {
                    operation["state"] = json!("failed");
                    operation["error"] =
                        serde_json::to_value(error).map_err(|_| Error::invalid())?;
                }
            }
        }
        if contract::validate("Operation", &operation).is_err() {
            // Reject an invalid internal reply before any local receipt transfer,
            // while preserving a queryable safe terminal result for the caller.
            operation.as_object_mut().ok_or_else(Error::invalid)?.remove("result");
            operation["state"] = json!("failed");
            operation["error"] = serde_json::to_value(Error::new(
                "internal-error",
                "The workspace operation could not be completed.",
                false,
            ))
            .map_err(|_| Error::invalid())?;
            retained = None;
            contract::validate("Operation", &operation)?;
        }
        self.operations.insert(id.to_owned(), operation);
        self.capture_progress.remove(id);
        if let Some(mut local) = retained {
            self.retained_bytes += local.retained_bytes;
            self.receipts.append(&mut local.receipts);
        }
        Ok(())
    }
    /// Retain one validated completed operation under the same shared record admission.
    pub(crate) fn completed(&mut self, kind: &str, mut result: Value) -> Result<Value> {
        if self.operation_count() >= MAX_RETAINED {
            return Err(capacity());
        }
        let operation_id = id("op")?;
        if kind == "conversion" || kind == "export" {
            result["operation_id"] = json!(operation_id);
        }
        let op = json!({"operation_id":operation_id,"kind":kind,"state":"succeeded","created_at":now(),"updated_at":now(),"cancel_requested":false,"result":result});
        contract::validate("Operation", &op)?;
        self.operations.insert(operation_id, op.clone());
        Ok(op)
    }
    /// Preserve the original v1 publication boundary for existing internal callers.
    pub(crate) fn commit(
        &mut self,
        root: &Root,
        request: &Value,
        stopped: &std::sync::atomic::AtomicBool,
    ) -> Result<Value> {
        self.commit_for_api(root, request, stopped, contract::ApiMajor::V1)
    }

    /// Revalidate publication using the session's immutable index-major boundary.
    pub(crate) fn commit_for_api(
        &mut self,
        root: &Root,
        request: &Value,
        stopped: &std::sync::atomic::AtomicBool,
        api_major: contract::ApiMajor,
    ) -> Result<Value> {
        if self.operation_count() >= MAX_RETAINED {
            return Err(capacity());
        }
        let token = request["receipt"].as_str().ok_or_else(Error::invalid)?;
        let receipt = self
            .receipts
            .values_mut()
            .find(|receipt| {
                receipt.preview["receipt"]["token"]
                    .as_str()
                    .is_some_and(|expected| expected.as_bytes().ct_eq(token.as_bytes()).into())
            })
            .ok_or_else(|| {
                Error::new(
                    "receipt-mismatch",
                    "The receipt does not identify this session's preview.",
                    false,
                )
            })?;
        if receipt.used {
            return Err(Error::new(
                "receipt-reused",
                "This preview has already been consumed.",
                false,
            ));
        }
        receipt.used = true; // Even an invalidated receipt is never usable again.
        if receipt.issued.elapsed() > RECEIPT_LIFETIME {
            return Err(Error::new(
                "receipt-expired",
                "The preview expired. Prepare a new preview.",
                false,
            ));
        }
        if request["observed_version"] != receipt.target.version {
            return Err(conflict());
        }
        if receipt.preview["exact_bytes_sha256"] != crate::hashing::sha256_hex(&receipt.bytes) {
            return Err(Error::invalid());
        }
        // Reserve and validate a queryable result before the commit syscall. No
        // serialization/allocation failure after publication can misreport it.
        let operation_id = id("op")?;
        let hash = crate::hashing::sha256_hex(&receipt.bytes);
        let result = json!({"write_committed":true,"committed_sha256":hash,"target_path":receipt.target.path,"new_version":hash});
        let op = json!({"operation_id":operation_id,"kind":"commit","state":"succeeded","created_at":now(),"updated_at":now(),"cancel_requested":false,"result":result});
        contract::validate("Operation", &op)?;
        root.commit(&receipt.target, &receipt.bytes, || {
            if stopped.load(std::sync::atomic::Ordering::Acquire) {
                return Err(Error::new(
                    "shutdown-in-progress",
                    "The workspace is stopping.",
                    false,
                ));
            }
            if Snapshot::capture_for_api(root, api_major)?.version != receipt.snapshot_version {
                return Err(conflict());
            }
            for (path, expected) in &receipt.external {
                let current = root.read(path, 10 * 1024 * 1024)?;
                if current.identity != expected.identity
                    || current.sha256 != expected.sha256
                    || current.bytes.len() != expected.bytes.len()
                {
                    return Err(conflict());
                }
            }
            Ok(())
        })?;
        receipt.committed = true;
        self.operations.insert(operation_id, op.clone());
        Ok(op)
    }
    /// Serve only committed report bytes through the existing HTML route.
    pub(crate) fn download(&self, root: &Root, operation_id: &str) -> Result<Vec<u8>> {
        self.download_family(root, operation_id, ArtifactFamily::ReportHtml)
    }

    /// Serve only committed metadata bytes through the new fixed JSON route.
    pub(crate) fn download_metadata(&self, root: &Root, operation_id: &str) -> Result<Vec<u8>> {
        self.download_family(root, operation_id, ArtifactFamily::MetadataJson)
    }

    /// Serve source bytes exclusively through their producer-owned fixed family.
    pub(crate) fn download_source(&self, root: &Root, operation_id: &str) -> Result<Vec<u8>> {
        self.download_family(root, operation_id, ArtifactFamily::SourceBundleJson)
    }

    /// Select a private producer family before reading exact current committed bytes.
    fn download_family(
        &self,
        root: &Root,
        operation_id: &str,
        family: ArtifactFamily,
    ) -> Result<Vec<u8>> {
        let op = self.operation(operation_id)?;
        if op["kind"] != "export" {
            return Err(unavailable());
        }
        let preview_id = op["result"]["preview"]["preview_id"].as_str().ok_or_else(unavailable)?;
        let receipt = self.receipts.get(preview_id).ok_or_else(unavailable)?;
        if !receipt.committed || receipt.artifact_family != Some(family) {
            return Err(unavailable());
        }
        let captured = root.read(&receipt.target.path, 10 * 1024 * 1024)?;
        if receipt.preview["exact_bytes_sha256"] != captured.sha256 {
            return Err(conflict());
        }
        Ok(captured.bytes)
    }
}
/// Fingerprint the admitted Value and exact method/public path/query replay identity.
fn request_hash(method: &str, path: &str, query: &str, value: &Value) -> Result<String> {
    Ok(crate::hashing::sha256_hex(
        &serde_json::to_vec(&json!([method, path, query, value])).map_err(|_| Error::invalid())?,
    ))
}
fn semantic_summary(kind: &str, bytes: &[u8]) -> String {
    match kind {
        "workspace-index-update" => "Register the explicitly selected file by updating the project index. Domain decisions remain in their own files.".into(),
        "resource-upload" => format!("Save {} explicitly uploaded bytes. Registration is a separate confirmed operation.",bytes.len()),
        "policy-conversion" => "Publish the prepared OSCAL conversion of supplied Markdown clauses. Registration is a separate confirmed operation.".into(),
        "applicability-manifest-write" => "Save the explicit applicability decision document and reviewed input pins. Omitted controls remain under review; report analysis requires a separate operation.".into(),
        "mapping-manifest-write" => "Save the explicit reviewed source/target relationships and their input pins. Rebuilding the mapping collection requires a separate operation.".into(),
        "applicability-report-write" => "Publish a gap analysis of the committed scope and mapping inputs. Classifications describe review state, not implementation or compliance.".into(),
        "mapping-report-write" => "Publish the mapping collection from committed reviewed relationships. Participation is not implementation, effectiveness, or approval.".into(),
        "report-export" => "Export an inert static report containing numeric review facts and opaque input hashes. Prose, reviewer names, rationale, and source excerpts are excluded.".into(),
        _ => "Write the explicitly prepared bytes to the reviewed destination.".into(),
    }
}

#[allow(clippy::naive_bytecount)] // Bounded UTF-8 documents do not justify another dependency.
fn text_diff(old: &[u8], new: &[u8]) -> (String, bool) {
    let lines = |bytes: &[u8]| {
        bytes.iter().filter(|byte| **byte == b'\n').count()
            + usize::from(!bytes.is_empty() && !bytes.ends_with(b"\n"))
    };
    let mut output = format!(
        "--- current\n+++ proposed\n@@ -{},{} +{},{} @@\n",
        usize::from(!old.is_empty()),
        lines(old),
        usize::from(!new.is_empty()),
        lines(new)
    );
    for (prefix, bytes) in [("-", old), ("+", new)] {
        let Ok(text) = std::str::from_utf8(bytes) else {
            return ("Binary content is not supported.".into(), true);
        };
        for line in text.split_inclusive('\n') {
            if output.len().saturating_add(line.len() + 2) > 190_000 {
                return (output, true);
            }
            output.push_str(prefix);
            output.push_str(line);
            if !line.ends_with('\n') {
                output.push_str("\n\\ No newline at end of file\n");
            }
        }
    }
    (output, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    /// A valid complete ordinary reply for reservation ownership checks.
    fn reservation_reply() -> Reply {
        Reply { value: json!({"state":"shutting-down"}), schema: "ShutdownResponse", status: 200 }
    }

    /// Pending requests bind method, public namespace, query and admitted payload together.
    #[test]
    fn pending_reservation_rejects_duplicate_and_conflicting_requests() {
        let mut store = Store::default();
        let request = json!({"bundle":"original"});
        let nonce =
            store.reserve("key", "POST", "/api/v2/project/bundle-imports", "", &request).unwrap();
        assert_eq!(
            store
                .replay("key", "POST", "/api/v2/project/bundle-imports", "", &request)
                .err()
                .unwrap()
                .code,
            "bundle-preparation-in-progress"
        );
        for (method, path, query, value) in [
            ("GET", "/api/v2/project/bundle-imports", "", request.clone()),
            ("POST", "/api/v1/project/bundle-imports", "", request.clone()),
            ("POST", "/api/v2/project/bundle-imports", "changed=1", request.clone()),
            ("POST", "/api/v2/project/bundle-imports", "", json!({"bundle":"different"})),
        ] {
            assert_eq!(
                store.replay("key", method, path, query, &value).err().unwrap().code,
                "idempotency-key-conflict"
            );
        }
        store.release_reservation("key", &nonce);
        assert!(
            store
                .replay("key", "POST", "/api/v2/project/bundle-imports", "", &request)
                .unwrap()
                .is_none()
        );
    }

    /// A stale worker cannot release a newer generation or replace its ready result.
    #[test]
    fn reservation_generation_preserves_newer_owner_and_original_ready_reply() {
        let mut store = Store::default();
        let request = json!({});
        let old = store.reserve("key", "POST", "/route", "", &request).unwrap();
        store.release_reservation("key", &old);
        let current = store.reserve("key", "POST", "/route", "", &request).unwrap();
        assert_ne!(old, current);
        store.release_reservation("key", &old);
        assert_eq!(store.pending_replays["key"].nonce, current);
        assert_eq!(
            store
                .retain_reserved("key", &old, Store::default(), &reservation_reply())
                .unwrap_err()
                .code,
            "idempotency-key-conflict"
        );
        store.retain_reserved("key", &current, Store::default(), &reservation_reply()).unwrap();
        store.release_reservation("key", &current);
        store.release_reservation("key", &old);
        let replay = store.replay("key", "POST", "/route", "", &request).unwrap().unwrap();
        assert_eq!(replay.value, reservation_reply().value);
        assert!(store.pending_replays.is_empty());
        assert_eq!(store.replays.len(), 1);
    }

    /// Pending and ready keys share one cap; failed retention transfers no local state.
    #[test]
    fn reservations_share_replay_capacity_and_transfer_atomically() {
        let mut store = Store::default();
        let request = json!({});
        let nonce = store.reserve("reserved", "POST", "/route", "", &request).unwrap();
        for index in 0..MAX_RETAINED - 1 {
            store
                .remember(
                    &format!("ready-{index}"),
                    "POST",
                    "/route",
                    "",
                    &request,
                    &reservation_reply(),
                )
                .unwrap();
        }
        assert!(store.replay("overflow", "POST", "/route", "", &request).is_err());
        let local = Store { retained_bytes: MAX_PREVIEW_BYTES + 1, ..Store::default() };
        assert!(store.retain_reserved("reserved", &nonce, local, &reservation_reply()).is_err());
        assert!(store.receipts.is_empty());
        assert_eq!(store.retained_bytes, 0);
        assert_eq!(store.replays.len(), MAX_RETAINED - 1);
        assert_eq!(store.pending_replays["reserved"].nonce, nonce);
        store.release_reservation("reserved", &nonce);
        assert!(store.replay("overflow", "POST", "/route", "", &request).unwrap().is_none());
    }

    /// Complete public replacement membership contributes to the same conservative byte cap.
    #[test]
    fn complete_reply_charge_refuses_without_changing_retained_bytes() {
        let mut store = Store { retained_bytes: MAX_PREVIEW_BYTES - 1, ..Store::default() };
        assert!(store.charge_reply(&reservation_reply()).is_err());
        assert_eq!(store.retained_bytes, MAX_PREVIEW_BYTES - 1);
    }

    /// Prepare a real metadata export receipt without publishing its destination.
    fn metadata_receipt(store: &mut Store, root: &Root, snapshot: &Snapshot) -> (Value, Value) {
        let plan = super::super::bundle_effects::prepare_export(
            root,
            snapshot,
            &json!({"target_path":"bundle.json","acknowledge_sensitive_metadata":true}),
            &mut super::super::preparation::NoopControl,
        )
        .unwrap();
        let preview = store.preview_bundle(plan).unwrap();
        let operation = store
            .completed(
                "export",
                json!({"operation_id":"op_000000000000",
            "preview":preview,"redaction_summary":{"removed_categories":["source-excerpts"]}}),
            )
            .unwrap();
        (preview, operation)
    }

    /// JSON and HTML routes select private committed families independently of path suffix.
    #[test]
    fn metadata_download_requires_commit_and_rejects_html_route() {
        let (_directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let (preview, operation) = metadata_receipt(&mut store, &root, &snapshot);
        let id = operation["operation_id"].as_str().unwrap();
        assert_eq!(store.download_metadata(&root, id).unwrap_err().code, "not-found");
        let request = json!({"receipt":preview["receipt"]["token"],
            "observed_version":preview["target_version"],"confirmed":true});
        store
            .commit_for_api(&root, &request, &AtomicBool::new(false), contract::ApiMajor::V2)
            .unwrap();
        let bytes = store.download_metadata(&root, id).unwrap();
        let bundle: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(bundle["content_profile"], "index-and-hashes");
        assert_eq!(bundle["index"]["resources"].as_array().unwrap().len(), 1);
        assert_eq!(store.download(&root, id).unwrap_err().code, "not-found");
        assert!(preview["semantic_summary"].as_str().unwrap().contains("paths"));
        assert!(!preview["semantic_summary"].as_str().unwrap().contains("secrets are excluded"));
    }

    /// Same-byte raw-index replacement invalidates a bound receipt without changing cursor identity.
    #[test]
    fn metadata_receipt_binds_raw_index_identity_in_addition_to_snapshot_hash() {
        let (directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let (preview, _) = metadata_receipt(&mut store, &root, &snapshot);
        let original =
            std::fs::read(directory.path().join(super::super::index::INDEX_PATH)).unwrap();
        std::fs::write(directory.path().join("replacement.json"), &original).unwrap();
        std::fs::rename(
            directory.path().join("replacement.json"),
            directory.path().join(super::super::index::INDEX_PATH),
        )
        .unwrap();
        assert_eq!(Snapshot::capture(&root).unwrap().version, snapshot.version);
        let request = json!({"receipt":preview["receipt"]["token"],
            "observed_version":preview["target_version"],"confirmed":true});
        assert_eq!(
            store
                .commit_for_api(&root, &request, &AtomicBool::new(false), contract::ApiMajor::V2)
                .unwrap_err()
                .code,
            "version-conflict"
        );
        assert!(!directory.path().join("bundle.json").exists());
        assert_eq!(
            store.get_preview(preview["preview_id"].as_str().unwrap()).unwrap_err().code,
            "receipt-reused"
        );
    }

    /// Complete private binding admission fails before installing any preview or byte charge.
    #[test]
    fn metadata_plan_capacity_failure_has_no_partial_retention() {
        let (_directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store { retained_bytes: MAX_PREVIEW_BYTES - 1, ..Store::default() };
        let plan = super::super::bundle_effects::prepare_export(
            &root,
            &snapshot,
            &json!({"target_path":"bundle.json","acknowledge_sensitive_metadata":true}),
            &mut super::super::preparation::NoopControl,
        )
        .unwrap();
        assert!(store.preview_bundle(plan).is_err());
        assert!(store.receipts.is_empty());
        assert_eq!(store.retained_bytes, MAX_PREVIEW_BYTES - 1);
    }
    #[test]
    fn cancellation_discards_prepared_receipts_without_publishing() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::open(dir.path()).unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        let mut store = Store::default();
        let op = store.begin("export").unwrap();
        let id = op["operation_id"].as_str().unwrap();
        assert!(store.running(id).unwrap());
        store.cancel(id).unwrap();
        let mut local = Store::default();
        let preview = local
            .preview(&root, &snapshot, "export.html", "report-export", b"prepared".to_vec(), &[])
            .unwrap();
        let value=local.completed("export",json!({"operation_id":id,"preview":preview,"redaction_summary":{"removed_categories":[]}})).unwrap();
        store
            .finish(id, Ok((local, Reply { value, schema: "Operation", status: 202 })), false)
            .unwrap();
        assert_eq!(store.operation(id).unwrap()["state"], "cancelled");
        assert!(store.receipts.is_empty());
        assert!(!dir.path().join("export.html").exists());
        assert_eq!(store.cancel(id).unwrap_err().code, "operation-not-cancellable");
    }
    #[test]
    fn overwritten_base_bytes_are_charged_to_the_retention_budget() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("output.json"), vec![b'x'; 10 * 1024 * 1024]).unwrap();
        let root = Root::open(dir.path()).unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        let mut store = Store::default();
        store
            .preview(&root, &snapshot, "output.json", "resource-upload", b"new".to_vec(), &[])
            .unwrap();
        assert!(
            store
                .preview(&root, &snapshot, "output.json", "resource-upload", b"new".to_vec(), &[])
                .is_err()
        );
    }
    #[test]
    fn expired_receipts_and_wrong_versions_never_write() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::open(dir.path()).unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        let mut store = Store::default();
        let preview = store
            .preview(&root, &snapshot, "output.json", "resource-upload", b"{}".to_vec(), &[])
            .unwrap();
        let id = preview["preview_id"].as_str().unwrap();
        store.receipts.get_mut(id).unwrap().issued =
            Instant::now().checked_sub(RECEIPT_LIFETIME + Duration::from_secs(1)).unwrap();
        let request = json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true});
        assert_eq!(
            store.commit(&root, &request, &AtomicBool::new(false)).unwrap_err().code,
            "receipt-expired"
        );
        assert!(!dir.path().join("output.json").exists());
        let preview = store
            .preview(&root, &snapshot, "output.json", "resource-upload", b"{}".to_vec(), &[])
            .unwrap();
        let request = json!({"receipt":preview["receipt"]["token"],"observed_version":"wrong-version","confirmed":true});
        assert_eq!(
            store.commit(&root, &request, &AtomicBool::new(false)).unwrap_err().code,
            "version-conflict"
        );
        assert!(!dir.path().join("output.json").exists());
    }
    #[test]
    fn shutdown_and_target_replacement_preserve_existing_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::open(dir.path()).unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        let mut store = Store::default();
        std::fs::write(dir.path().join("output.json"), b"old").unwrap();
        let preview = store
            .preview(&root, &snapshot, "output.json", "resource-upload", b"new".to_vec(), &[])
            .unwrap();
        let request = json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true});
        assert_eq!(
            store.commit(&root, &request, &AtomicBool::new(true)).unwrap_err().code,
            "shutdown-in-progress"
        );
        assert_eq!(std::fs::read(dir.path().join("output.json")).unwrap(), b"old");
        let preview = store
            .preview(&root, &snapshot, "output.json", "resource-upload", b"new".to_vec(), &[])
            .unwrap();
        std::fs::write(dir.path().join("output.json"), b"external").unwrap();
        let request = json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true});
        assert_eq!(
            store.commit(&root, &request, &AtomicBool::new(false)).unwrap_err().code,
            "version-conflict"
        );
        assert_eq!(std::fs::read(dir.path().join("output.json")).unwrap(), b"external");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn replay_rejects_changed_content_and_diff_preserves_unicode_and_newline_state() {
        let mut store = Store::default();
        let request = json!({"path":"a.json"});
        let reply = Reply { value: json!({}), schema: "ShutdownResponse", status: 200 };
        store.remember("same-key", "POST", "/route", "page=1", &request, &reply).unwrap();
        assert!(store.replay("same-key", "POST", "/route", "page=1", &request).unwrap().is_some());
        assert_eq!(
            store
                .replay("same-key", "POST", "/route", "page=1", &json!({"path":"b.json"}))
                .err()
                .unwrap()
                .code,
            "idempotency-key-conflict"
        );
        // A reused key with a different query string is a different request.
        assert_eq!(
            store.replay("same-key", "POST", "/route", "page=2", &request).err().unwrap().code,
            "idempotency-key-conflict"
        );
        let (diff, truncated) = text_diff("café".as_bytes(), "日本語\n".as_bytes());
        assert!(!truncated);
        assert!(diff.contains("-café\n\\ No newline at end of file\n+日本語\n"));
    }
    /// Build real registered input and destination sentinels for Store checkpoint tests.
    fn checkpoint_store_fixture() -> (tempfile::TempDir, Root, Snapshot) {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("policy.md"),
            b"# Explicit policy\n\nA supplied clause.\n",
        )
        .unwrap();
        let index = json!({"schema_version":"forge.workspace/1","label":"Checkpoint fixture",
            "resources":[{"key":"policy","role":"policy-source","path":"policy.md"}]});
        std::fs::write(
            directory.path().join("forge.workspace.json"),
            super::super::index::Index::parse(&serde_json::to_vec(&index).unwrap())
                .unwrap()
                .bytes()
                .unwrap(),
        )
        .unwrap();
        std::fs::write(directory.path().join("review.html"), b"EXISTING REVIEW SENTINEL\n")
            .unwrap();
        std::fs::write(directory.path().join("keeper.html"), b"EXISTING KEEPER SENTINEL\n")
            .unwrap();
        let root = Root::open(directory.path()).unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        (directory, root, snapshot)
    }

    /// Prepare an actual inert trace export and its local one-use receipt.
    fn checkpoint_store_prepared(
        root: &Root,
        snapshot: &Snapshot,
        target: &str,
    ) -> (Store, Reply, String) {
        let inputs = super::super::reports::inputs(snapshot, "trace").unwrap();
        let bytes = super::super::reports::render(snapshot, "trace", json!({
            "total_elements":0,"asserted_trace_elements":0,"current_source_locations":0,"unresolved_elements":0
        })).unwrap();
        let mut local = Store::default();
        let preview =
            local.preview(root, snapshot, target, "report-export", bytes, &inputs).unwrap();
        let preview_id = preview["preview_id"].as_str().unwrap().to_owned();
        let value = local
            .completed(
                "export",
                json!({"preview":preview,
            "redaction_summary":{"removed_categories":[]}}),
            )
            .unwrap();
        (local, Reply { value, schema: "Operation", status: 202 }, preview_id)
    }

    /// Seed an unrelated real receipt so interruption tests detect accidental clearing.
    fn checkpoint_store_keeper(
        store: &mut Store,
        root: &Root,
        snapshot: &Snapshot,
    ) -> (String, Value, usize) {
        let inputs = super::super::reports::inputs(snapshot, "trace").unwrap();
        let bytes = super::super::reports::render(snapshot, "trace", json!({
            "total_elements":0,"asserted_trace_elements":0,"current_source_locations":0,"unresolved_elements":0
        })).unwrap();
        let preview =
            store.preview(root, snapshot, "keeper.html", "report-export", bytes, &inputs).unwrap();
        let id = preview["preview_id"].as_str().unwrap().to_owned();
        (id, preview, store.retained_bytes)
    }

    /// Assert that preparation neither publishes nor modifies registered fixture bytes.
    fn checkpoint_store_sources_unchanged(directory: &std::path::Path, snapshot: &Snapshot) {
        for item in &snapshot.items {
            assert_eq!(
                std::fs::read(directory.join(&item.registration.path)).unwrap(),
                item.captured.bytes
            );
        }
        assert_eq!(
            std::fs::read(directory.join("forge.workspace.json")).unwrap(),
            snapshot.index.bytes().unwrap()
        );
    }

    /// Adversarial observations after one of three captured items: regression, denominator change, overflow and cap excess.
    fn checkpoint_store_invalid_counters() -> [super::super::preparation::ProgressUpdate; 5] {
        use super::super::preparation::ProgressUpdate;
        [
            ProgressUpdate::Capture { completed: 0, total: 3 },
            ProgressUpdate::Capture { completed: 2, total: 4 },
            ProgressUpdate::Capture { completed: 4, total: 3 },
            ProgressUpdate::Capture { completed: 1, total: 1001 },
            ProgressUpdate::Capture { completed: usize::MAX, total: usize::MAX },
        ]
    }

    /// Inactive operations ignore observations and active capture rejects invalid whole counters atomically.
    #[test]
    fn checkpoint_progress_requires_running_and_fixed_valid_capture_counters() {
        use super::super::preparation::ProgressUpdate;
        let mut store = Store::default();
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(
            !store
                .observe_progress(id, ProgressUpdate::Capture { completed: 0, total: 3 })
                .unwrap()
        );
        assert_eq!(store.operation(id).unwrap(), operation);
        assert!(store.running(id).unwrap());
        assert!(
            store.observe_progress(id, ProgressUpdate::Capture { completed: 0, total: 3 }).unwrap()
        );
        assert!(
            store.observe_progress(id, ProgressUpdate::Capture { completed: 1, total: 3 }).unwrap()
        );
        let observed = store.operation(id).unwrap();
        for update in checkpoint_store_invalid_counters() {
            assert_eq!(store.observe_progress(id, update).unwrap_err().code, "invalid-request");
            assert_eq!(store.operation(id).unwrap(), observed);
        }
        assert!(store.observe_progress(id, ProgressUpdate::Unchanged).unwrap());
        assert_eq!(
            store.operation(id).unwrap()["progress"],
            json!({"completed_items":1,"total_items":3})
        );
        assert!(
            store.observe_progress(id, ProgressUpdate::Capture { completed: 3, total: 3 }).unwrap()
        );
        assert!(store.observe_progress(id, ProgressUpdate::Clear).unwrap());
        let cleared = store.operation(id).unwrap();
        assert!(cleared["progress"].is_null());
        for update in [
            ProgressUpdate::Capture { completed: 2, total: 3 },
            ProgressUpdate::Capture { completed: 3, total: 4 },
        ] {
            assert_eq!(store.observe_progress(id, update).unwrap_err().code, "invalid-request");
            assert_eq!(store.operation(id).unwrap(), cleared);
        }
        super::super::contract::validate("Operation", &store.operation(id).unwrap()).unwrap();
    }

    /// Empty capture and the complete 1,000-registration boundary remain representable facts.
    #[test]
    fn checkpoint_progress_accepts_empty_and_full_capture_boundaries() {
        use super::super::preparation::ProgressUpdate;
        for total in [0, 1000] {
            let mut store = Store::default();
            let operation = store.begin("export").unwrap();
            let id = operation["operation_id"].as_str().unwrap();
            assert!(store.running(id).unwrap());
            assert!(
                store
                    .observe_progress(id, ProgressUpdate::Capture { completed: 0, total })
                    .unwrap()
            );
            assert!(
                store
                    .observe_progress(id, ProgressUpdate::Capture { completed: total, total })
                    .unwrap()
            );
            assert_eq!(
                store.operation(id).unwrap()["progress"],
                json!({"completed_items":total,"total_items":total})
            );
            super::super::contract::validate("Operation", &store.operation(id).unwrap()).unwrap();
        }
    }

    /// A running cancellation acknowledges work, suppresses further progress, and discards only its local receipt.
    #[test]
    fn checkpoint_cancelled_transfer_preserves_keeper_and_project_sentinels() {
        use super::super::preparation::ProgressUpdate;
        let (directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let (keeper_id, keeper, retained) = checkpoint_store_keeper(&mut store, &root, &snapshot);
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(store.running(id).unwrap());
        assert!(
            store.observe_progress(id, ProgressUpdate::Capture { completed: 1, total: 1 }).unwrap()
        );
        let acknowledgement = store.cancel(id).unwrap();
        assert_eq!(acknowledgement["state"], "running");
        assert_eq!(acknowledgement["cancel_requested"], true);
        assert!(
            !store
                .observe_progress(id, ProgressUpdate::Capture { completed: 1, total: 1 })
                .unwrap()
        );
        assert_eq!(store.operation(id).unwrap(), acknowledgement);
        let (local, reply, discarded_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        store.finish(id, Ok((local, reply)), false).unwrap();
        let terminal = store.operation(id).unwrap();
        assert_eq!(terminal["state"], "cancelled");
        assert_eq!(terminal["cancel_requested"], true);
        assert!(terminal["result"].is_null());
        assert!(terminal["error"].is_null());
        assert!(terminal["progress"].is_null());
        assert_eq!(store.get_preview(&discarded_id).unwrap_err().code, "not-found");
        assert_eq!(store.get_preview(&keeper_id).unwrap(), keeper);
        assert_eq!(store.receipts.len(), 1);
        assert_eq!(store.retained_bytes, retained);
        assert_eq!(
            std::fs::read(directory.path().join("review.html")).unwrap(),
            b"EXISTING REVIEW SENTINEL\n"
        );
        assert_eq!(
            std::fs::read(directory.path().join("keeper.html")).unwrap(),
            b"EXISTING KEEPER SENTINEL\n"
        );
        checkpoint_store_sources_unchanged(directory.path(), &snapshot);
        super::super::contract::validate("Operation", &terminal).unwrap();
    }

    /// Once success owns a real receipt, late cancellation, running, progress, and finish cannot replace it.
    #[test]
    fn checkpoint_succeeded_operation_cannot_resurrect_or_merge_twice() {
        use super::super::preparation::ProgressUpdate;
        let (directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(store.running(id).unwrap());
        assert!(
            store.observe_progress(id, ProgressUpdate::Capture { completed: 1, total: 1 }).unwrap()
        );
        let (local, reply, retained_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        store.finish(id, Ok((local, reply)), false).unwrap();
        let terminal = store.operation(id).unwrap();
        assert_eq!(terminal["state"], "succeeded");
        assert_eq!(terminal["result"]["operation_id"], id);
        assert!(terminal["progress"].is_null());
        let preview = store.get_preview(&retained_id).unwrap();
        let bytes = store.retained_bytes;
        assert_eq!(store.cancel(id).unwrap_err().code, "operation-not-cancellable");
        assert!(!store.running(id).unwrap());
        assert!(
            !store
                .observe_progress(id, ProgressUpdate::Capture { completed: 0, total: 1 })
                .unwrap()
        );
        assert!(!store.observe_progress(id, ProgressUpdate::Clear).unwrap());
        let (late_local, late_reply, late_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        store.finish(id, Ok((late_local, late_reply)), true).unwrap();
        store.finish(id, Err(Error::invalid()), false).unwrap();
        assert_eq!(store.operation(id).unwrap(), terminal);
        assert_eq!(store.get_preview(&retained_id).unwrap(), preview);
        assert_eq!(store.get_preview(&late_id).unwrap_err().code, "not-found");
        assert_eq!(store.receipts.len(), 1);
        assert_eq!(store.retained_bytes, bytes);
        assert_eq!(
            std::fs::read(directory.path().join("review.html")).unwrap(),
            b"EXISTING REVIEW SENTINEL\n"
        );
        checkpoint_store_sources_unchanged(directory.path(), &snapshot);
    }

    /// A real failure is terminal and an obsolete successful local reply cannot attach a preview later.
    #[test]
    fn checkpoint_failed_operation_rejects_late_progress_and_receipt_transfer() {
        use super::super::preparation::ProgressUpdate;
        let (directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let (keeper_id, keeper, retained) = checkpoint_store_keeper(&mut store, &root, &snapshot);
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(store.running(id).unwrap());
        assert!(
            store.observe_progress(id, ProgressUpdate::Capture { completed: 1, total: 1 }).unwrap()
        );
        store.finish(id, Err(Error::invalid()), false).unwrap();
        let terminal = store.operation(id).unwrap();
        assert_eq!(terminal["state"], "failed");
        assert_eq!(terminal["error"]["code"], "invalid-request");
        assert!(terminal["result"].is_null());
        assert!(terminal["progress"].is_null());
        assert!(!store.running(id).unwrap());
        assert!(!store.observe_progress(id, ProgressUpdate::Unchanged).unwrap());
        let (local, reply, discarded_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        store.finish(id, Ok((local, reply)), false).unwrap();
        assert_eq!(store.operation(id).unwrap(), terminal);
        assert_eq!(store.get_preview(&discarded_id).unwrap_err().code, "not-found");
        assert_eq!(store.get_preview(&keeper_id).unwrap(), keeper);
        assert_eq!(store.receipts.len(), 1);
        assert_eq!(store.retained_bytes, retained);
        checkpoint_store_sources_unchanged(directory.path(), &snapshot);
        super::super::contract::validate("Operation", &terminal).unwrap();
    }

    /// A cancelled pending job never starts, and obsolete completion cannot replace its cancelled terminal.
    #[test]
    fn checkpoint_pending_cancellation_is_terminal_before_any_late_completion() {
        use super::super::preparation::ProgressUpdate;
        let (_, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert_eq!(store.cancel(id).unwrap()["state"], "pending");
        assert!(!store.running(id).unwrap());
        let terminal = store.operation(id).unwrap();
        assert_eq!(terminal["state"], "cancelled");
        assert!(terminal["progress"].is_null());
        assert!(!store.running(id).unwrap());
        assert!(!store.observe_progress(id, ProgressUpdate::Clear).unwrap());
        let (local, reply, discarded_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        store.finish(id, Ok((local, reply)), false).unwrap();
        assert_eq!(store.operation(id).unwrap(), terminal);
        assert_eq!(store.receipts.keys().collect::<Vec<_>>(), [] as [&String; 0]);
        assert_eq!(store.retained_bytes, 0);
        assert_eq!(store.get_preview(&discarded_id).unwrap_err().code, "not-found");
        super::super::contract::validate("Operation", &terminal).unwrap();
    }

    /// A final deadline/shutdown fence overrides both success and ordinary failure without charging local receipts.
    #[test]
    fn checkpoint_stop_fence_wins_over_success_and_failure_before_transfer() {
        let (directory, root, snapshot) = checkpoint_store_fixture();
        for succeeds in [false, true] {
            let mut store = Store::default();
            let (keeper_id, keeper, retained) =
                checkpoint_store_keeper(&mut store, &root, &snapshot);
            let operation = store.begin("export").unwrap();
            let id = operation["operation_id"].as_str().unwrap();
            assert!(store.running(id).unwrap());
            let prepared = if succeeds {
                let (local, reply, _) = checkpoint_store_prepared(&root, &snapshot, "review.html");
                Ok((local, reply))
            } else {
                Err(Error::invalid())
            };
            store.finish(id, prepared, true).unwrap();
            let terminal = store.operation(id).unwrap();
            assert_eq!(terminal["state"], "cancelled");
            assert!(terminal["result"].is_null());
            assert!(terminal["error"].is_null());
            assert!(terminal["progress"].is_null());
            assert_eq!(store.get_preview(&keeper_id).unwrap(), keeper);
            assert_eq!(store.receipts.len(), 1);
            assert_eq!(store.retained_bytes, retained);
            assert_eq!(
                std::fs::read(directory.path().join("review.html")).unwrap(),
                b"EXISTING REVIEW SENTINEL\n"
            );
            checkpoint_store_sources_unchanged(directory.path(), &snapshot);
            super::super::contract::validate("Operation", &terminal).unwrap();
        }
    }

    /// Actual ten-MiB destination bases collide at final retention without partially merging local receipts.
    #[test]
    fn checkpoint_retention_collision_preserves_existing_receipts_and_large_targets() {
        let (directory, root, snapshot) = checkpoint_store_fixture();
        let keeper_bytes = vec![b'k'; 10 * 1024 * 1024];
        let review_bytes = vec![b'r'; 10 * 1024 * 1024];
        std::fs::write(directory.path().join("keeper.html"), &keeper_bytes).unwrap();
        std::fs::write(directory.path().join("review.html"), &review_bytes).unwrap();
        let mut store = Store::default();
        let (keeper_id, keeper, retained) = checkpoint_store_keeper(&mut store, &root, &snapshot);
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(store.running(id).unwrap());
        let (local, reply, discarded_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        assert!(retained <= MAX_PREVIEW_BYTES);
        assert!(local.retained_bytes <= MAX_PREVIEW_BYTES);
        assert!(retained + local.retained_bytes > MAX_PREVIEW_BYTES);
        store.finish(id, Ok((local, reply)), false).unwrap();
        let terminal = store.operation(id).unwrap();
        assert_eq!(terminal["state"], "failed");
        assert_eq!(terminal["error"]["code"], "invalid-request");
        assert!(terminal["result"].is_null());
        assert!(terminal["progress"].is_null());
        assert_eq!(store.receipts.len(), 1);
        assert_eq!(store.retained_bytes, retained);
        assert_eq!(store.get_preview(&keeper_id).unwrap(), keeper);
        assert_eq!(store.get_preview(&discarded_id).unwrap_err().code, "not-found");
        assert_eq!(std::fs::read(directory.path().join("keeper.html")).unwrap(), keeper_bytes);
        assert_eq!(std::fs::read(directory.path().join("review.html")).unwrap(), review_bytes);
        checkpoint_store_sources_unchanged(directory.path(), &snapshot);
    }

    /// Real retained receipt cardinality rejects a 257th merge without dropping any of the existing 256 previews.
    #[test]
    fn checkpoint_receipt_count_collision_preserves_every_retained_preview() {
        let (directory, root, snapshot) = checkpoint_store_fixture();
        let mut store = Store::default();
        let mut keepers = Vec::new();
        for _ in 0..MAX_RETAINED {
            let (id, preview, _) = checkpoint_store_keeper(&mut store, &root, &snapshot);
            keepers.push((id, preview));
        }
        assert_eq!(store.receipts.len(), MAX_RETAINED);
        let retained = store.retained_bytes;
        let operation = store.begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(store.running(id).unwrap());
        let (local, reply, discarded_id) =
            checkpoint_store_prepared(&root, &snapshot, "review.html");
        store.finish(id, Ok((local, reply)), false).unwrap();
        let terminal = store.operation(id).unwrap();
        assert_eq!(terminal["state"], "failed");
        assert_eq!(terminal["error"]["code"], "invalid-request");
        assert!(terminal["result"].is_null());
        assert!(terminal["progress"].is_null());
        assert_eq!(store.receipts.len(), MAX_RETAINED);
        assert_eq!(store.retained_bytes, retained);
        for (keeper_id, preview) in keepers {
            assert_eq!(store.get_preview(&keeper_id).unwrap(), preview);
        }
        assert_eq!(store.get_preview(&discarded_id).unwrap_err().code, "not-found");
        assert_eq!(
            std::fs::read(directory.path().join("keeper.html")).unwrap(),
            b"EXISTING KEEPER SENTINEL\n"
        );
        assert_eq!(
            std::fs::read(directory.path().join("review.html")).unwrap(),
            b"EXISTING REVIEW SENTINEL\n"
        );
        checkpoint_store_sources_unchanged(directory.path(), &snapshot);
    }

    /// Invalid internal result contracts fail safely before receipt transfer, while an acknowledged cancellation retains priority.
    #[test]
    fn checkpoint_invalid_internal_result_settles_safely_without_partial_merge() {
        use super::super::preparation::ProgressUpdate;
        let (directory, root, snapshot) = checkpoint_store_fixture();
        for cancelled in [false, true] {
            let mut store = Store::default();
            let (keeper_id, keeper, retained) =
                checkpoint_store_keeper(&mut store, &root, &snapshot);
            let operation = store.begin("export").unwrap();
            let id = operation["operation_id"].as_str().unwrap();
            assert!(store.running(id).unwrap());
            assert!(
                store
                    .observe_progress(id, ProgressUpdate::Capture { completed: 1, total: 1 })
                    .unwrap()
            );
            let (local, mut reply, discarded_id) =
                checkpoint_store_prepared(&root, &snapshot, "review.html");
            reply.value["result"] = json!({"private_source":"PRIVATE INTERNAL TEST SENTINEL"});
            if cancelled {
                store.cancel(id).unwrap();
            }
            store.finish(id, Ok((local, reply)), false).unwrap();
            let terminal = store.operation(id).unwrap();
            if cancelled {
                assert_eq!(terminal["state"], "cancelled");
                assert!(terminal["error"].is_null());
            } else {
                assert_eq!(terminal["state"], "failed");
                assert_eq!(terminal["error"]["code"], "internal-error");
                assert_eq!(
                    terminal["error"]["message"],
                    "The workspace operation could not be completed."
                );
                assert_eq!(terminal["error"]["retryable"], false);
            }
            assert!(terminal["result"].is_null());
            assert!(terminal["progress"].is_null());
            assert!(!terminal.to_string().contains("PRIVATE INTERNAL TEST SENTINEL"));
            assert_eq!(store.receipts.len(), 1);
            assert_eq!(store.retained_bytes, retained);
            assert_eq!(store.get_preview(&keeper_id).unwrap(), keeper);
            assert_eq!(store.get_preview(&discarded_id).unwrap_err().code, "not-found");
            assert!(!store.running(id).unwrap());
            assert!(!store.observe_progress(id, ProgressUpdate::Unchanged).unwrap());
            assert_eq!(store.operation(id).unwrap(), terminal);
            assert_eq!(
                std::fs::read(directory.path().join("review.html")).unwrap(),
                b"EXISTING REVIEW SENTINEL\n"
            );
            checkpoint_store_sources_unchanged(directory.path(), &snapshot);
            super::super::contract::validate("Operation", &terminal).unwrap();
        }
    }
}

//! Session-owned exact-byte receipts, idempotent results, and single-file effects.
use super::contract::{self, Error, Result};
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
struct Receipt {
    preview: Value,
    target: Target,
    bytes: Vec<u8>,
    snapshot_version: String,
    issued: Instant,
    used: bool,
    committed: bool,
    external: Vec<(String, super::root::Captured)>,
}
struct Replay {
    request_hash: String,
    value: Value,
    schema: &'static str,
    status: u16,
}
#[derive(Default)]
pub(crate) struct Store {
    receipts: BTreeMap<String, Receipt>,
    operations: BTreeMap<String, Value>,
    replays: BTreeMap<String, Replay>,
    retained_bytes: usize,
}
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
    pub(crate) fn replay(
        &self,
        key: &str,
        method: &str,
        path: &str,
        query: &str,
        request: &Value,
    ) -> Result<Option<Reply>> {
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
        if self.replays.len() >= MAX_RETAINED {
            return Err(capacity());
        }
        Ok(None)
    }
    pub(crate) fn remember(
        &mut self,
        key: &str,
        method: &str,
        path: &str,
        query: &str,
        request: &Value,
        reply: &Reply,
    ) -> Result<()> {
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
        if self.receipts.len() >= MAX_RETAINED
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
    pub(crate) fn begin(&mut self, kind: &str) -> Result<Value> {
        if self.operations.len() >= MAX_RETAINED {
            return Err(capacity());
        }
        let id = id("op")?;
        let operation = json!({"operation_id":id,"kind":kind,"state":"pending","created_at":now(),"updated_at":now(),"cancel_requested":false});
        contract::validate("Operation", &operation)?;
        self.operations.insert(id, operation.clone());
        Ok(operation)
    }
    pub(crate) fn running(&mut self, id: &str) -> Result<bool> {
        let operation = self.operations.get_mut(id).ok_or_else(unavailable)?;
        if operation["cancel_requested"] == true {
            operation["state"] = json!("cancelled");
            return Ok(false);
        }
        operation["state"] = json!("running");
        operation["updated_at"] = json!(now());
        Ok(true)
    }
    pub(crate) fn finish(
        &mut self,
        id: &str,
        prepared: Result<(Self, Reply)>,
        cancelled: bool,
    ) -> Result<()> {
        let operation = self.operations.get_mut(id).ok_or_else(unavailable)?;
        operation["updated_at"] = json!(now());
        if cancelled || operation["cancel_requested"] == true {
            operation["state"] = json!("cancelled");
            operation["cancel_requested"] = json!(true);
            return Ok(());
        }
        let prepared = prepared.and_then(|(local, reply)| {
            if self.receipts.len() + local.receipts.len() > MAX_RETAINED
                || self.retained_bytes.saturating_add(local.retained_bytes) > MAX_PREVIEW_BYTES
            {
                return Err(capacity());
            }
            Ok((local, reply))
        });
        match prepared {
            Ok((mut local, reply)) => {
                let mut result = reply.value["result"].clone();
                if result.get("operation_id").is_some() {
                    result["operation_id"] = json!(id);
                }
                operation["state"] = json!("succeeded");
                operation["result"] = result;
                contract::validate("Operation", operation)?;
                self.retained_bytes += local.retained_bytes;
                self.receipts.append(&mut local.receipts);
            }
            Err(error) => {
                operation["state"] = json!("failed");
                operation["error"] = serde_json::to_value(error).map_err(|_| Error::invalid())?;
            }
        }
        Ok(())
    }
    pub(crate) fn completed(&mut self, kind: &str, mut result: Value) -> Result<Value> {
        if self.operations.len() >= MAX_RETAINED {
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
    pub(crate) fn commit(
        &mut self,
        root: &Root,
        request: &Value,
        stopped: &std::sync::atomic::AtomicBool,
    ) -> Result<Value> {
        if self.operations.len() >= MAX_RETAINED {
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
            if Snapshot::capture(root)?.version != receipt.snapshot_version {
                return Err(conflict());
            }
            for (path, expected) in &receipt.external {
                let current = root.read(path, 10 * 1024 * 1024)?;
                if current.identity != expected.identity || current.sha256 != expected.sha256 {
                    return Err(conflict());
                }
            }
            Ok(())
        })?;
        receipt.committed = true;
        self.operations.insert(operation_id, op.clone());
        Ok(op)
    }
    pub(crate) fn download(&self, root: &Root, operation_id: &str) -> Result<Vec<u8>> {
        let op = self.operation(operation_id)?;
        if op["kind"] != "export" {
            return Err(unavailable());
        }
        let preview_id = op["result"]["preview"]["preview_id"].as_str().ok_or_else(unavailable)?;
        let receipt = self.receipts.get(preview_id).ok_or_else(unavailable)?;
        if !receipt.committed {
            return Err(unavailable());
        }
        let captured = root.read(&receipt.target.path, 10 * 1024 * 1024)?;
        if receipt.preview["exact_bytes_sha256"] != captured.sha256 {
            return Err(conflict());
        }
        Ok(captured.bytes)
    }
}
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
}

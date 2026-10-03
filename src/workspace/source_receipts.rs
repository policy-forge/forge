//! Session-owned one-time source restore previews and shared retained reservations.
//!
//! Native accepted intent is a separate durable
//! boundary; no client Value or this Store alone authorizes project writes.

use super::*;
use crate::workspace::root::{RestoreTargetPlan, TransactionState};

/// A source preview owns the sealed real plan until its one explicit confirmation.
struct SourceReceipt {
    /// Complete validated public preview; token remains bound to this session.
    preview: Value,
    /// Exactly one owned current/new generation, moved rather than duplicated.
    plan: Option<RestoreTargetPlan>,
    /// Monotonic one-time preview deadline independent of outcome retention.
    issued: Instant,
    /// Failed confirmation never revives the same preview authority.
    used: bool,
}

#[cfg(all(test, unix))]
mod tests {
    use super::super::super::preparation::NoopControl;
    use super::*;

    /// Build a real empty-root sealed restore plan through the consumed producer.
    /// This fixture grants no accepted native intent and performs no project write.
    fn preview_fixture() -> (tempfile::TempDir, Store, Value, Value) {
        let project = tempfile::tempdir().unwrap();
        let root = Root::open(project.path()).unwrap();
        let snapshot = Snapshot::capture_source_bundle_effect_with_control(
            &root,
            None,
            None,
            &mut NoopControl,
        )
        .unwrap();
        assert!(snapshot.index.resources.is_empty());
        // Export admission requires a present donor index; the recipient is
        // deliberately absent and remains untouched by these pure Store tests.
        let donor = tempfile::tempdir().unwrap();
        std::fs::write(
            donor.path().join("forge.workspace.json"),
            super::super::super::index::Index::empty().bytes().unwrap(),
        )
        .unwrap();
        let donor_root = Root::open(donor.path()).unwrap();
        let donor_snapshot = Snapshot::capture_source_bundle_effect_with_control(
            &donor_root,
            None,
            None,
            &mut NoopControl,
        )
        .unwrap();
        let raw =
            super::super::super::source_bundles::encode(&donor_snapshot, &mut NoopControl).unwrap();
        let bundle: Value = serde_json::from_slice(&raw).unwrap();
        let request = json!({"bundle":bundle,"target_index_schema_version":1,
            "acknowledge_index_replacement":true,"acknowledge_source_content":true,"acknowledge_replace_files":true});
        let decoded = super::super::super::source_bundles::decode(&raw, &mut NoopControl).unwrap();
        let plan = super::super::super::source_bundle_effects::prepare_import(
            &root,
            snapshot,
            decoded,
            &request,
            &mut NoopControl,
        )
        .unwrap();
        let mut store = Store::default();
        let (preview, replacement) = store.preview_source_restore(plan).unwrap();
        assert_eq!(preview["targets"].as_array().unwrap().len(), 1);
        assert_eq!(preview["targets"][0]["path"], super::super::super::index::INDEX_PATH);
        (project, store, preview, replacement)
    }

    /// Construct the exact admitted confirmation from a real server-owned preview.
    fn confirmation(preview: &Value) -> Value {
        json!({"receipt":preview["receipt"]["token"],"observed_batch_version":preview["observed_batch_version"],"acknowledge_exact_restore":true})
    }

    /// Return one path-bound acceptance for safe pure Store controls only.
    fn take(store: &mut Store, preview: &Value) -> SourceAcceptance {
        let id = preview["preview_id"].as_str().unwrap();
        store
            .take_source_confirmation(
                id,
                "confirm",
                &format!("/api/v2/project/bundle-restores/{id}/commit"),
                "",
                &confirmation(preview),
            )
            .unwrap()
    }

    /// A wrong token leaves authority intact; the correct token with stale version consumes it without writing.
    #[test]
    fn token_and_batch_fences_preserve_no_write_and_one_time_consumption() {
        let (project, mut store, preview, _) = preview_fixture();
        let id = preview["preview_id"].as_str().unwrap();
        let mut request = confirmation(&preview);
        request["receipt"] = json!("f".repeat(64));
        assert_eq!(
            store
                .take_source_confirmation(
                    id,
                    "bad",
                    "/api/v2/project/bundle-restores/p/commit",
                    "",
                    &request
                )
                .err()
                .unwrap()
                .code,
            "receipt-mismatch"
        );
        assert!(!store.sources.receipts[id].used);
        request = confirmation(&preview);
        request["observed_batch_version"] = json!("0".repeat(64));
        assert_eq!(
            store
                .take_source_confirmation(
                    id,
                    "stale",
                    "/api/v2/project/bundle-restores/p/commit",
                    "",
                    &request
                )
                .err()
                .unwrap()
                .code,
            "version-conflict"
        );
        assert!(store.sources.receipts[id].used);
        assert!(store.pending_replays.is_empty());
        assert!(!project.path().join("forge.workspace.json").exists());
    }

    /// Consumed confirmations cannot be erased by a preparation guard or reused after handoff uncertainty.
    #[test]
    fn restore_pending_owner_survives_generic_drop_and_uncertain_handoff() {
        let (project, mut store, preview, _) = preview_fixture();
        let acceptance = take(&mut store, &preview);
        assert!(store.pending_replays["confirm"].kind == PendingKind::Restore);
        store.release_reservation("confirm", &acceptance.reservation_nonce);
        assert!(store.pending_replays.contains_key("confirm"));
        store
            .source_acceptance_uncertain(&acceptance.operation_id, &acceptance.reservation_nonce)
            .unwrap();
        assert_eq!(
            store
                .source_progress(&acceptance.operation_id, ProgressUpdate::Unchanged)
                .unwrap_err()
                .code,
            "bundle-restore-recovery-required"
        );
        assert_eq!(store.sources.jobs[&acceptance.operation_id].value["cancel_requested"], false);
        assert_eq!(
            store.replay("confirm", "POST", "/wrong", "", &json!({})).err().unwrap().code,
            "idempotency-key-conflict"
        );
        assert!(!project.path().join("forge.workspace.json").exists());
    }

    /// Generation mismatch keeps the reservation; exact acceptance publishes one original immutable replay.
    #[test]
    fn accepted_replay_is_generation_bound_and_never_reencoded_by_store() {
        let (_project, mut store, preview, _) = preview_fixture();
        let mut acceptance = take(&mut store, &preview);
        let nonce = acceptance.reservation_nonce.clone();
        acceptance.reservation_nonce = "obsolete".into();
        assert_eq!(
            store.source_accepted("confirm", &acceptance).unwrap_err().code,
            "bundle-restore-recovery-required"
        );
        assert_eq!(store.pending_replays["confirm"].nonce, nonce);
        acceptance.reservation_nonce = nonce;
        store.source_accepted("confirm", &acceptance).unwrap();
        let path = format!(
            "/api/v2/project/bundle-restores/{}/commit",
            preview["preview_id"].as_str().unwrap()
        );
        let replay =
            store.replay("confirm", "POST", &path, "", &confirmation(&preview)).unwrap().unwrap();
        assert_eq!(
            contract::encode(&replay.value, 1024 * 1024, false).unwrap(),
            acceptance.accepted_reply
        );
        assert_eq!(replay.status, 202);
        assert!(store.pending_replays.is_empty());
        assert!(store.source_accepted("confirm", &acceptance).is_err());
    }

    /// Only an explicit sticky cancellation returns the cooperative cancellation signal.
    #[test]
    fn system_handoff_errors_are_distinct_from_explicit_cancel_and_fixed_progress() {
        let (_project, mut store, preview, _) = preview_fixture();
        let acceptance = take(&mut store, &preview);
        assert_eq!(
            store
                .source_progress(&acceptance.operation_id, ProgressUpdate::Unchanged)
                .unwrap_err()
                .code,
            "bundle-restore-recovery-required"
        );
        store.source_accepted("confirm", &acceptance).unwrap();
        assert!(
            store
                .source_progress(
                    &acceptance.operation_id,
                    ProgressUpdate::Capture { completed: 0, total: 1 }
                )
                .unwrap()
        );
        assert!(
            store
                .source_progress(
                    &acceptance.operation_id,
                    ProgressUpdate::Capture { completed: 1, total: 2 }
                )
                .is_err()
        );
        assert!(
            store
                .source_progress(
                    &acceptance.operation_id,
                    ProgressUpdate::Capture { completed: 1, total: 1 }
                )
                .unwrap()
        );
        assert!(
            store
                .source_progress(
                    &acceptance.operation_id,
                    ProgressUpdate::Capture { completed: 0, total: 1 }
                )
                .is_err()
        );
        let cancelled = store
            .cancel_source_restore(&acceptance.operation_id, acceptance.reply.value.clone())
            .unwrap();
        assert_eq!(cancelled["cancel_requested"], true);
        assert!(
            !store.source_progress(&acceptance.operation_id, ProgressUpdate::Unchanged).unwrap()
        );
    }

    /// Complete wrapper capacity failure preserves the original preparation owner and transfers no preview.
    #[test]
    fn complete_wrapper_retention_is_atomic_and_uses_the_shared_pool() {
        let (_project, mut local, preview, replacement) = preview_fixture();
        let reply = Reply {
            value: json!({"validation":super::super::super::services::validation(true,None),"preview":preview,
            "replacement":replacement}),
            schema: "ProjectSourceBundleImportPreview",
            status: 200,
        };
        contract::validate_for(contract::ApiMajor::V2, reply.schema, &reply.value).unwrap();
        local.charge_reply(&reply).unwrap();
        let mut store = Store::default();
        let request = json!({"fixture":"admitted body placeholder for reservation only"});
        let nonce = store
            .reserve("prepare", "POST", "/api/v2/project/source-bundle-imports", "", &request)
            .unwrap();
        store.retained_bytes = MAX_PREVIEW_BYTES;
        assert!(store.retain_source_reserved("prepare", &nonce, local, &reply).is_err());
        assert!(store.sources.receipts.is_empty());
        assert!(store.replays.is_empty());
        assert_eq!(store.pending_replays["prepare"].nonce, nonce);
    }

    /// Expired matching authority is consumed without constructing accepted intent or touching the index.
    #[test]
    fn expiry_consumes_capability_without_native_acceptance() {
        let (project, mut store, preview, _) = preview_fixture();
        let id = preview["preview_id"].as_str().unwrap();
        store.sources.receipts.get_mut(id).unwrap().issued =
            Instant::now().checked_sub(RECEIPT_LIFETIME).unwrap();
        assert_eq!(
            store
                .take_source_confirmation(
                    id,
                    "expired",
                    "/api/v2/project/bundle-restores/p/commit",
                    "",
                    &confirmation(&preview)
                )
                .err()
                .unwrap()
                .code,
            "receipt-expired"
        );
        assert!(store.sources.jobs.is_empty());
        assert!(store.pending_replays.is_empty());
        assert!(!project.path().join("forge.workspace.json").exists());
    }

    /// Restored outcome counts and bytes bound ordinary export acceptance before any mutation.
    #[test]
    fn ordinary_export_acceptance_shares_recovered_outcome_cardinality_and_bytes() {
        let mut store = Store::default();
        for n in 0..MAX_RETAINED {
            store.sources.recovered.insert(format!("op_{n:016x}"), 1);
        }
        assert!(store.begin("export").is_err());
        assert!(
            store
                .accept_source_export(
                    "key",
                    "/api/v2/project/source-bundle-exports",
                    "",
                    &json!({})
                )
                .is_err()
        );
        assert!(store.operations.is_empty());
        assert!(store.replays.is_empty());
        store.sources.recovered.clear();
        store.retained_bytes = MAX_PREVIEW_BYTES;
        assert!(
            store
                .accept_source_export(
                    "key",
                    "/api/v2/project/source-bundle-exports",
                    "",
                    &json!({})
                )
                .is_err()
        );
        assert!(store.operations.is_empty());
        assert!(store.replays.is_empty());
        store.retained_bytes = 0;
        let (reply, bytes) = store
            .accept_source_export("key", "/api/v2/project/source-bundle-exports", "", &json!({}))
            .unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), reply.value);
        assert_eq!(store.operation_count(), 1);
        assert!(store.retained_bytes >= bytes.len() * 4);
        assert_eq!(
            store
                .replay("key", "POST", "/api/v2/project/source-bundle-exports", "", &json!({}))
                .unwrap()
                .unwrap()
                .value,
            reply.value
        );
    }
}

/// Volatile control facts for an already durably accepted restore operation.
struct SourceJob {
    /// Original pending reply, with server-owned mutable control observations.
    value: Value,
    /// Last actual numeric publication observation, with a fixed denominator.
    progress: Option<(usize, usize)>,
    /// Generation of this exact confirmation owner.
    nonce: String,
    /// True only after the native durable acceptance succeeds.
    accepted: bool,
}

/// One exact admitted confirmation, moved outside the shared Store lock.
pub(crate) struct SourceAcceptance {
    /// Owned sealed native plan with the original captured generations.
    pub(crate) plan: RestoreTargetPlan,
    /// Preallocated nonauthorizing operation lookup ID from the preview.
    pub(crate) operation_id: String,
    /// Same pending reservation generation that owns the original request.
    pub(crate) reservation_nonce: String,
    /// Bounded native correlation value derived from that complete generation.
    pub(crate) native_nonce: u64,
    /// Exact canonical admitted confirmation digest, not raw HTTP provenance.
    pub(crate) request_sha256: String,
    /// Exact manifest from the complete confirmed source preview.
    pub(crate) exact_manifest_sha256: String,
    /// Fully encoded and validated original202 response, durable before transport.
    pub(crate) accepted_reply: Vec<u8>,
    /// Complete original public reply for immutable idempotent replay retention.
    pub(crate) reply: Reply,
}

/// Separate source maps still consume the original Store count/byte ceilings.
#[derive(Default)]
pub(super) struct SourceStore {
    /// One-time preview capabilities, never seeded from persisted outcomes.
    receipts: BTreeMap<String, SourceReceipt>,
    /// Same-session accepted/control observations, keyed by preallocated lookup ID.
    jobs: BTreeMap<String, SourceJob>,
    /// Outcome-only native records observed at restart, granting no old receipt.
    recovered: BTreeMap<String, usize>,
}

impl SourceStore {
    /// Count the complete current source receipt population.
    pub(super) fn receipt_count(&self) -> usize {
        self.receipts.len()
    }

    /// Count same-session and recovered outcome IDs once.
    pub(super) fn operation_count(&self) -> usize {
        self.jobs
            .len()
            .saturating_add(self.recovered.keys().filter(|id| !self.jobs.contains_key(*id)).count())
    }
}

/// Return one fixed safe failure without private journal or parser details.
fn recovery_required() -> Error {
    Error::new(
        "bundle-restore-recovery-required",
        "The accepted restore requires verified recovery before project access.",
        false,
    )
}

impl Store {
    /// Atomically retain a complete ordinary source-export acceptance and replay
    /// before dispatch; failures publish no operation or receipt authority.
    pub(crate) fn accept_source_export(
        &mut self,
        key: &str,
        wire_path: &str,
        raw_query: &str,
        request: &Value,
    ) -> Result<(Reply, Vec<u8>)> {
        if self.operation_count() >= MAX_RETAINED
            || self.replay(key, "POST", wire_path, raw_query, request)?.is_some()
        {
            return Err(capacity());
        }
        let mut local = Self::default();
        let operation = local.begin("export")?;
        let reply = Reply { value: operation, schema: "Operation", status: 202 };
        contract::validate_for(contract::ApiMajor::V2, reply.schema, &reply.value)?;
        let bytes = contract::encode(&reply.value, 1024 * 1024, false)?;
        local.charge_reply(&reply)?;
        let retained =
            self.retained_bytes.checked_add(local.retained_bytes).ok_or_else(capacity)?;
        if retained > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        let hash = request_hash("POST", wire_path, raw_query, request)?;
        self.replays.insert(
            key.to_owned(),
            Replay {
                request_hash: hash,
                value: reply.value.clone(),
                schema: reply.schema,
                status: reply.status,
            },
        );
        self.operations.append(&mut local.operations);
        self.retained_bytes = retained;
        Ok((reply, bytes))
    }

    /// Count source and existing previews under one retained cardinality ceiling.
    pub(crate) fn receipt_count(&self) -> usize {
        self.receipts.len().saturating_add(self.sources.receipt_count())
    }

    /// Count source outcomes and existing operations under one retained ceiling.
    pub(crate) fn operation_count(&self) -> usize {
        self.operations.len().saturating_add(self.sources.operation_count())
    }

    /// Seed outcome-only native reservations after restart; old receipt authority
    /// is never reconstructed, and unresolved native records cannot be evicted.
    pub(crate) fn seed_source_outcomes(&mut self, state: &TransactionState) -> Result<()> {
        let rows = state.retained_charges()?;
        let mut recovered = BTreeMap::new();
        let mut charge = 0usize;
        for (operation_id, bytes, _) in rows {
            if self.sources.jobs.contains_key(&operation_id)
                || recovered.insert(operation_id, bytes).is_some()
            {
                return Err(recovery_required());
            }
            charge = charge.checked_add(bytes).ok_or_else(capacity)?;
        }
        let retained = self.retained_bytes.checked_add(charge).ok_or_else(capacity)?;
        if retained > MAX_PREVIEW_BYTES
            || self.operation_count().saturating_add(recovered.len()) > MAX_RETAINED
        {
            return Err(capacity());
        }
        if !self.sources.recovered.is_empty() {
            return Err(recovery_required());
        }
        self.sources.recovered = recovered;
        self.retained_bytes = retained;
        Ok(())
    }

    /// Retain one complete proposed source plan and every public preview byte.
    pub(crate) fn preview_source_restore(
        &mut self,
        plan: crate::workspace::source_bundle_effects::PreparedSourceRestore,
    ) -> Result<(Value, Value)> {
        if self.receipt_count() >= MAX_RETAINED
            || self.sources.jobs.contains_key(&plan.operation_id)
            || self.sources.recovered.contains_key(&plan.operation_id)
            || plan.consumed_file_count > MAX_CONSUMED_INPUTS
            || plan.targets.is_empty()
            || plan.targets.len() > MAX_CONSUMED_INPUTS
            || plan.input_bindings.len() > MAX_CONSUMED_INPUTS
            || plan.directories.len() > MAX_CONSUMED_INPUTS
            || plan.reserved_private_bytes != plan.plan.reserved_private_bytes()
        {
            return Err(capacity());
        }
        let proposed_hash = crate::hashing::sha256_hex(&plan.proposed_index.bytes()?);
        if plan.replacement["proposed_index_sha256"] != proposed_hash {
            return Err(Error::invalid());
        }
        // Proposed-index bytes were admitted by the producer and remain bound to
        // the sealed index target; the public replacement is never authority.
        let preview_id = id("prev")?;
        let token = super::super::session::random_token()?;
        let expires = (chrono::Utc::now() + chrono::TimeDelta::seconds(600))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let preview = json!({"preview_id":preview_id,"operation_type":"project-source-restore",
            "snapshot_version":plan.snapshot_version,"observed_batch_version":plan.observed_batch_version,
            "exact_manifest_sha256":plan.exact_manifest_sha256,"targets":plan.targets,
            "input_bindings":plan.input_bindings,"directories":plan.directories,
            "validation":plan.validation,"semantic_summary":plan.semantic_summary,
            "receipt":{"token":&*token,"expires_at":expires},"operation_id":plan.operation_id});
        contract::validate_for(contract::ApiMajor::V2, "SourceRestorePreview", &preview)?;
        contract::validate_for(
            contract::ApiMajor::V2,
            "SourceBundleIndexReplacement",
            &plan.replacement,
        )?;
        let public = contract::encode(&preview, 1024 * 1024, false)?;
        let charge = public
            .len()
            .checked_mul(4)
            .and_then(|sum| sum.checked_add(plan.reserved_private_bytes))
            .ok_or_else(capacity)?;
        let retained = self.retained_bytes.checked_add(charge).ok_or_else(capacity)?;
        if retained > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        self.sources.receipts.insert(
            preview_id,
            SourceReceipt {
                preview: preview.clone(),
                plan: Some(plan.plan),
                issued: Instant::now(),
                used: false,
            },
        );
        self.retained_bytes = retained;
        Ok((preview, plan.replacement))
    }

    /// Move complete source receipts with the same atomic reservation/replay rule
    /// used by metadata imports. The caller checks its deadline under this lock.
    pub(crate) fn retain_source_reserved(
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
        if retained > MAX_PREVIEW_BYTES
            || self.receipt_count().saturating_add(local.receipt_count()) > MAX_RETAINED
            || !local.receipts.is_empty()
            || !local.operations.is_empty()
            || !local.replays.is_empty()
            || !local.pending_replays.is_empty()
            || local.sources.operation_count() != 0
            || local.sources.receipts.keys().any(|id| self.sources.receipts.contains_key(id))
        {
            return Err(capacity());
        }
        let retained_response = Replay {
            request_hash: pending.request_hash.clone(),
            value: reply.value.clone(),
            schema: reply.schema,
            status: reply.status,
        };
        self.sources.receipts.append(&mut local.sources.receipts);
        self.retained_bytes = retained;
        self.replays.insert(key.to_owned(), retained_response);
        self.pending_replays.remove(key);
        Ok(())
    }

    /// Admit path/token/version/acknowledgment and charge the original pending
    /// reply before moving its sealed plan to the durable acceptance boundary.
    pub(crate) fn take_source_confirmation(
        &mut self,
        preview_id: &str,
        key: &str,
        wire_path: &str,
        raw_query: &str,
        request: &Value,
    ) -> Result<SourceAcceptance> {
        contract::validate_for(contract::ApiMajor::V2, "CommitSourceRestoreRequest", request)?;
        if self.operation_count() >= MAX_RETAINED {
            return Err(capacity());
        }
        let token = request["receipt"].as_str().ok_or_else(Error::invalid)?;
        let receipt = self.sources.receipts.get(preview_id).ok_or_else(unavailable)?;
        if !receipt.preview["receipt"]["token"]
            .as_str()
            .is_some_and(|expected| expected.as_bytes().ct_eq(token.as_bytes()).into())
        {
            return Err(Error::new(
                "receipt-mismatch",
                "The receipt does not identify this session's complete source preview.",
                false,
            ));
        }
        if receipt.used {
            return Err(Error::new(
                "receipt-reused",
                "This source preview has already been consumed.",
                false,
            ));
        }
        let expired = receipt.issued.elapsed() >= RECEIPT_LIFETIME;
        let version_matches =
            request["observed_batch_version"] == receipt.preview["observed_batch_version"];
        let operation_id =
            receipt.preview["operation_id"].as_str().ok_or_else(Error::invalid)?.to_owned();
        let exact_manifest_sha256 = receipt.preview["exact_manifest_sha256"]
            .as_str()
            .ok_or_else(Error::invalid)?
            .to_owned();
        // The exact capability is consumed even if its later version fence fails.
        self.sources.receipts.get_mut(preview_id).ok_or_else(unavailable)?.used = true;
        if expired {
            return Err(Error::new(
                "receipt-expired",
                "The source preview expired. Prepare a new preview.",
                false,
            ));
        }
        if !version_matches {
            return Err(conflict());
        }
        let created = now();
        let value = json!({"operation_id":operation_id,"kind":"bundle-restore","state":"pending",
            "created_at":created,"updated_at":created,"cancel_requested":false,"progress":null,
            "write_outcome":"unmeasured","cleanup_state":"unmeasured","result":null,"error":null});
        contract::validate_for(contract::ApiMajor::V2, "SourceRestoreOperation", &value)?;
        let accepted_reply = contract::encode(&value, 1024 * 1024, false)?;
        let charge = accepted_reply.len().checked_mul(4).ok_or_else(capacity)?;
        let retained = self.retained_bytes.checked_add(charge).ok_or_else(capacity)?;
        if retained > MAX_PREVIEW_BYTES {
            return Err(capacity());
        }
        let request_sha256 = request_hash("POST", wire_path, raw_query, request)?;
        let nonce = self.reserve(key, "POST", wire_path, raw_query, request)?;
        self.pending_replays.get_mut(key).ok_or_else(recovery_required)?.kind =
            PendingKind::Restore;
        let nonce_digest = crate::hashing::sha256_hex(nonce.as_bytes());
        let native_nonce =
            u64::from_str_radix(&nonce_digest[..16], 16).map_err(|_| Error::invalid())?;
        let plan = self
            .sources
            .receipts
            .get_mut(preview_id)
            .ok_or_else(unavailable)?
            .plan
            .take()
            .ok_or_else(unavailable)?;
        self.sources.jobs.insert(
            operation_id.clone(),
            SourceJob {
                value: value.clone(),
                progress: None,
                nonce: nonce.clone(),
                accepted: false,
            },
        );
        self.retained_bytes = retained;
        Ok(SourceAcceptance {
            plan,
            operation_id,
            reservation_nonce: nonce,
            native_nonce,
            request_sha256,
            exact_manifest_sha256,
            accepted_reply,
            reply: Reply { value, schema: "SourceRestoreOperation", status: 202 },
        })
    }

    /// Publish the original replay only after native intent and exact reply are
    /// durable. Generation mismatch never erases a newer pending owner.
    pub(crate) fn source_accepted(
        &mut self,
        key: &str,
        acceptance: &SourceAcceptance,
    ) -> Result<()> {
        let pending = self.pending_replays.get(key).ok_or_else(recovery_required)?;
        let job =
            self.sources.jobs.get_mut(&acceptance.operation_id).ok_or_else(recovery_required)?;
        if pending.nonce != acceptance.reservation_nonce
            || job.nonce != acceptance.reservation_nonce
            || job.accepted
            || self.replays.contains_key(key)
        {
            return Err(recovery_required());
        }
        job.accepted = true;
        self.replays.insert(
            key.to_owned(),
            Replay {
                request_hash: pending.request_hash.clone(),
                value: acceptance.reply.value.clone(),
                schema: acceptance.reply.schema,
                status: 202,
            },
        );
        self.pending_replays.remove(key);
        Ok(())
    }

    /// A durable acceptance error is uncertain: retain its reservation and
    /// one-time capability consumption until qualified recovery resolves it.
    pub(crate) fn source_acceptance_uncertain(
        &mut self,
        operation_id: &str,
        nonce: &str,
    ) -> Result<()> {
        let job = self.sources.jobs.get_mut(operation_id).ok_or_else(recovery_required)?;
        if job.nonce != nonce {
            return Err(recovery_required());
        }
        job.value["state"] = json!("recovery-required");
        job.value["write_outcome"] = json!("unknown");
        job.value["cleanup_state"] = json!("unverified");
        job.value["error"] =
            serde_json::to_value(recovery_required()).map_err(|_| Error::invalid())?;
        job.value["updated_at"] = json!(now());
        Ok(())
    }

    /// Observe pending cancellation without acquiring the project-data lease.
    /// Only actual native publication progress may advance measured file counts.
    pub(crate) fn source_progress(
        &mut self,
        operation_id: &str,
        update: ProgressUpdate,
    ) -> Result<bool> {
        let job = self.sources.jobs.get_mut(operation_id).ok_or_else(unavailable)?;
        if !job.accepted || !matches!(job.value["state"].as_str(), Some("pending" | "running")) {
            return Err(recovery_required());
        }
        if job.value["cancel_requested"] == true {
            return Ok(false);
        }
        match update {
            ProgressUpdate::Unchanged => {}
            ProgressUpdate::Clear => job.value["progress"] = Value::Null,
            ProgressUpdate::Capture { completed, total } => {
                if total > 100
                    || completed > total
                    || job
                        .progress
                        .is_some_and(|(old, denominator)| denominator != total || completed < old)
                {
                    return Err(Error::invalid());
                }
                job.progress = Some((completed, total));
                job.value["progress"] = json!({"completed_files":completed,"total_files":total});
            }
        }
        job.value["state"] = json!("running");
        job.value["updated_at"] = json!(now());
        Ok(true)
    }

    /// Request sticky cancellation in this session; restart observation supplies
    /// no resurrected worker or old confirmation capability.
    pub(crate) fn cancel_source_restore(
        &mut self,
        operation_id: &str,
        durable: Value,
    ) -> Result<Value> {
        contract::validate_for(contract::ApiMajor::V2, "SourceRestoreOperation", &durable)?;
        if durable["operation_id"] != operation_id {
            return Err(recovery_required());
        }
        if let Some(job) = self.sources.jobs.get_mut(operation_id) {
            if job.accepted && matches!(durable["state"].as_str(), Some("pending" | "running")) {
                job.value["cancel_requested"] = json!(true);
                job.value["updated_at"] = json!(now());
            }
        }
        self.source_operation(operation_id, durable)
    }

    /// Native durable outcome controls terminal truth; volatile controls supplement
    /// only the same accepted nonterminal operation in this current session.
    pub(crate) fn source_operation(&self, operation_id: &str, mut durable: Value) -> Result<Value> {
        contract::validate_for(contract::ApiMajor::V2, "SourceRestoreOperation", &durable)?;
        if durable["operation_id"] != operation_id {
            return Err(recovery_required());
        }
        if matches!(durable["state"].as_str(), Some("pending" | "running")) {
            if let Some(job) = self.sources.jobs.get(operation_id).filter(|job| job.accepted) {
                durable["cancel_requested"] = job.value["cancel_requested"].clone();
                durable["progress"] = job.value["progress"].clone();
                durable["updated_at"] = job.value["updated_at"].clone();
            }
        }
        contract::validate_for(contract::ApiMajor::V2, "SourceRestoreOperation", &durable)?;
        Ok(durable)
    }
}

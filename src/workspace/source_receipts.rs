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
        assert!(store.non_stage_retention().unwrap().bytes >= bytes.len() * 4);
        assert_eq!(
            store
                .replay("key", "POST", "/api/v2/project/source-bundle-exports", "", &json!({}))
                .unwrap()
                .unwrap()
                .value,
            reply.value
        );
    }
    /// The exact source confirmation reserves separate job/replay/native owners before moving its real sealed plan.
    #[test]
    fn shared_source_confirmation_worker_release_keeps_native_outcome_authority() {
        let (project, mut store, preview, _) = preview_fixture();
        let acceptance = take(&mut store, &preview);
        assert_eq!(store.retained_entity_count().unwrap(), 5);
        store.source_accepted("confirm", &acceptance).unwrap();
        let id = acceptance.operation_id.clone();
        let nonce = acceptance.reservation_nonce.clone();
        store.source_worker_released(&id, "stale");
        assert_eq!(store.retained_entity_count().unwrap(), 5);
        let bytes = store.non_stage_retention().unwrap().bytes;
        drop(acceptance);
        store.source_worker_released(&id, &nonce);
        assert_eq!(store.retained_entity_count().unwrap(), 4);
        assert_eq!(bytes - store.non_stage_retention().unwrap().bytes, 4096);
        assert!(store.retention_claims.contains_key(&RetentionOwner::Source(id)));
        assert!(store.replays.contains_key("confirm"));
        assert!(!project.path().join("forge.workspace.json").exists());
    }

    /// Source confirmation cannot bypass shared old reply occupancy or publish pending authority after refusal.
    #[test]
    fn shared_source_confirmation_refuses_complete_entity_overflow_before_plan_move() {
        let (project, mut store, preview, _) = preview_fixture();
        for ordinal in 0..252 {
            let reply = Reply {
                value: json!({"state":"shutting-down"}),
                schema: "ShutdownResponse",
                status: 200,
            };
            store
                .remember(
                    &format!("source-fill-{ordinal}"),
                    "POST",
                    "/accounting-control",
                    "",
                    &json!({}),
                    &reply,
                )
                .unwrap();
        }
        assert_eq!(store.retained_entity_count().unwrap(), 253);
        let id = preview["preview_id"].as_str().unwrap();
        let request = confirmation(&preview);
        assert_eq!(
            store
                .take_source_confirmation(
                    id,
                    "confirm",
                    &format!("/api/v2/project/bundle-restores/{id}/commit"),
                    "",
                    &request
                )
                .err()
                .unwrap()
                .code,
            "invalid-request"
        );
        assert!(store.sources.receipts[id].plan.is_some());
        assert!(store.sources.jobs.is_empty());
        assert!(store.pending_replays.is_empty());
        assert!(store.retention_claims.is_empty());
        assert_eq!(store.retained_entity_count().unwrap(), 253);
        assert!(!project.path().join("forge.workspace.json").exists());
    }

    /// Claimed local preparation is exchanged for the complete wrapper and original replay, without renewing its generation.
    #[test]
    fn shared_source_prepared_reply_is_atomic_and_replayable_after_claim_release() {
        let (project, mut fixture, preview, replacement) = preview_fixture();
        let mut store = Store::default();
        let request = json!({});
        let nonce = store
            .reserve(
                "source-prepare",
                "POST",
                "/api/v2/project/source-bundle-imports",
                "",
                &request,
            )
            .unwrap();
        let mut local = store.preparation_store("source-prepare", &nonce).unwrap();
        local.retained_bytes = fixture.retained_bytes;
        local.sources.receipts.append(&mut fixture.sources.receipts);
        drop(fixture);
        let reply = Reply {
            value: json!({"validation":super::super::super::services::validation(true,None),"preview":preview,"replacement":replacement}),
            schema: "ProjectSourceBundleImportPreview",
            status: 200,
        };
        store.retain_source_reserved("source-prepare", &nonce, local, &reply).unwrap();
        store.release_reservation("source-prepare", &nonce);
        assert!(store.pending_replays.is_empty());
        assert!(store.retention_claims.is_empty());
        assert_eq!(store.retained_entity_count().unwrap(), 2);
        assert_eq!(
            store
                .replay(
                    "source-prepare",
                    "POST",
                    "/api/v2/project/source-bundle-imports",
                    "",
                    &request
                )
                .unwrap()
                .unwrap()
                .value,
            reply.value
        );
        assert!(!project.path().join("forge.workspace.json").exists());
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

/// Bound the complete source terminal/control DTO using exact sealed path/hash/size membership.
/// This over-approximation includes committed recovery plus progress and all bounded safe Error fields.
fn source_control_charge(preview: &Value, pending: &Value) -> Result<usize> {
    let targets=preview["targets"].as_array().ok_or_else(Error::invalid)?.iter().map(|target|
        json!({"path":target["path"],"sha256":target["exact_bytes_sha256"],"size":target["size"]})).collect::<Vec<_>>();
    let mut bound = pending.clone();
    bound["state"] = json!("recovery-required");
    bound["write_outcome"] = json!("committed");
    bound["cleanup_state"] = json!("unverified");
    bound["updated_at"] = json!("0".repeat(40));
    bound["cancel_requested"] = json!(true);
    bound["progress"] = json!({"completed_files":100,"total_files":100});
    bound["result"] = json!({"write_committed":true,"exact_manifest_sha256":preview["exact_manifest_sha256"],
        "committed_targets":targets,"cleanup_state":"unverified"});
    bound["error"] = control_error_bound();
    staged_value_charge(&bound)?.checked_mul(2).ok_or_else(capacity)
}

impl Store {
    /// Retain the accepted ordinary job and distinct ready replay together, with failure/worker allowance reserved.
    pub(crate) fn accept_source_export(
        &mut self,
        key: &str,
        wire_path: &str,
        raw_query: &str,
        request: &Value,
    ) -> Result<(Reply, Vec<u8>)> {
        self.accept_operation(
            OperationRequestIdentity { key, method: "POST", wire_path, raw_query, request },
            "export",
        )
    }
    /// Atomically accept an ordinary job and original ready reply before dispatch; no partial operation on quota refusal.
    pub(crate) fn accept_operation(
        &mut self,
        identity: OperationRequestIdentity<'_>,
        kind: &str,
    ) -> Result<(Reply, Vec<u8>)> {
        let OperationRequestIdentity { key, method, wire_path, raw_query, request } = identity;
        if self.replay(key, method, wire_path, raw_query, request)?.is_some() {
            return Err(idempotency_conflict());
        }
        let mut local = Self::default();
        let value = local.begin(kind)?;
        let reply = Reply { value, schema: "Operation", status: 202 };
        contract::validate("Operation", &reply.value)?;
        let bytes = contract::encode(&reply.value, 1024 * 1024, false)?;
        let input_bytes = staged_value_charge(request)?;
        let owner = RetentionOwner::Ordinary(
            reply.value["operation_id"].as_str().ok_or_else(Error::invalid)?.into(),
        );
        let claim = local.retention_claims.get_mut(&owner).ok_or_else(capacity)?;
        claim.bytes = claim.bytes.checked_add(input_bytes).ok_or_else(capacity)?;
        claim.input_bytes = input_bytes;
        let usage = local.non_stage_retention()?;
        let current = self.non_stage_retention()?;
        self.admit_shared_retention(OtherPoolUsage {
            bytes: current
                .bytes
                .checked_add(usage.bytes)
                .and_then(|sum| sum.checked_add(staged_value_charge(&reply.value).ok()?))
                .and_then(|sum| sum.checked_add(4096))
                .ok_or_else(capacity)?,
            records: current
                .records
                .checked_add(usage.records)
                .and_then(|sum| sum.checked_add(1))
                .ok_or_else(capacity)?,
        })?;
        let ready_record = Replay {
            request_hash: request_hash(method, wire_path, raw_query, request)?,
            value: reply.value.clone(),
            schema: reply.schema,
            status: 202,
        };
        self.operations.append(&mut local.operations);
        self.retention_claims.append(&mut local.retention_claims);
        self.replays.insert(key.into(), ready_record);
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
        self.admit_growth(charge, 0, recovered.len())?;
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
        self.admit_growth(charge, staged_value_charge(&preview)?, 1)?;
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

    /// Atomically exchange the exact preparation allowance for complete source receipts and distinct ready reply.
    pub(crate) fn retain_source_reserved(
        &mut self,
        key: &str,
        nonce: &str,
        mut local: Self,
        reply: &Reply,
    ) -> Result<()> {
        let pending = self.pending_replays.get(key).ok_or_else(idempotency_conflict)?;
        if pending.kind != PendingKind::Preparation
            || pending.nonce != nonce
            || self.replays.contains_key(key)
        {
            return Err(idempotency_conflict());
        }
        if !local.receipts.is_empty()
            || !local.operations.is_empty()
            || !local.replays.is_empty()
            || !local.pending_replays.is_empty()
            || !local.retention_claims.is_empty()
            || local.sources.operation_count() != 0
            || local.transfers.retained_records() != 0
            || local.sources.receipts.keys().any(|id| self.sources.receipts.contains_key(id))
        {
            return Err(capacity());
        }
        let owner = RetentionOwner::Preparation(nonce.into());
        let base = self.usage_without_claim(&owner)?;
        let usage = local.non_stage_retention()?;
        self.admit_shared_retention(OtherPoolUsage {
            bytes: base
                .bytes
                .checked_add(usage.bytes)
                .and_then(|sum| sum.checked_add(staged_value_charge(&reply.value).ok()?))
                .ok_or_else(capacity)?,
            records: base.records.checked_add(usage.records).ok_or_else(capacity)?,
        })?;
        let retained =
            self.retained_bytes.checked_add(local.retained_bytes).ok_or_else(capacity)?;
        let ready_record = Replay {
            request_hash: pending.request_hash.clone(),
            value: reply.value.clone(),
            schema: reply.schema,
            status: reply.status,
        };
        self.sources.receipts.append(&mut local.sources.receipts);
        self.replays.insert(key.into(), ready_record);
        self.pending_replays.remove(key);
        self.retention_claims.remove(&owner);
        self.retained_bytes = retained;
        Ok(())
    }

    /// Admit path/token/version/acknowledgment and charge the original pending
    /// reply before moving its sealed plan to the durable acceptance boundary.
    #[expect(
        clippy::too_many_lines,
        reason = "Keep capability consumption, exact request identity, complete pre-admission and native plan transfer in one audited acceptance boundary."
    )]
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
        let control = source_control_charge(&self.sources.receipts[preview_id].preview, &value)?;
        let replay_floor = staged_value_charge(&value)?;
        let request_sha256 = request_hash("POST", wire_path, raw_query, request)?;
        if self.replay(key, "POST", wire_path, raw_query, request)?.is_some() {
            return Err(idempotency_conflict());
        }
        let nonce = id("prep")?;
        // Job + pending-to-ready replay + held native worker + retained native outcome owner.
        // Native raw/plan reservation was already admitted with the receipt and is not refunded by its move.
        let future = charge
            .checked_add(control)
            .and_then(|sum| sum.checked_add(replay_floor))
            .and_then(|sum| sum.checked_add(4096))
            .ok_or_else(capacity)?;
        self.admit_growth(future, control / 2, 4)?;
        let claim_bytes = control.checked_add(replay_floor).ok_or_else(capacity)?;
        let pending = PendingReplay {
            kind: PendingKind::Restore,
            request_hash: request_sha256.clone(),
            nonce: nonce.clone(),
        };
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
        self.pending_replays.insert(key.into(), pending);
        self.retention_claims.insert(
            RetentionOwner::Source(operation_id.clone()),
            RetentionClaim {
                nonce: nonce.clone(),
                bytes: claim_bytes,
                records: 2,
                control_bytes: control,
                input_bytes: 0,
                fallback: None,
                worker: true,
                allocated: true,
            },
        );
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

    /// Release the exact native worker only after its real plan and accepted handle have physically dropped.
    /// Durable outcome/control/replay reservations remain; this never grants cancellation or rollback proof.
    pub(crate) fn source_worker_released(&mut self, operation_id: &str, nonce: &str) {
        if let Some(claim) =
            self.retention_claims.get_mut(&RetentionOwner::Source(operation_id.into()))
        {
            if claim.nonce == nonce && claim.worker {
                claim.worker = false;
                claim.records = claim.records.saturating_sub(1);
            }
        }
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

// Staged transport shares the existing Store retention and privacy boundary.
use crate::workspace::source_transfers::{
    DecodedPart, OtherPoolUsage, PreparationLease, PreparationTicket, SourceTransfers, StageRequest,
};

/// Ephemeral producer-owned identity for one committed staged export, never a public path.
pub(crate) struct StagedExportDescriptor {
    /// Exact confined target originally selected and confirmed by the export producer.
    pub(crate) path: String,
    /// Original exact artifact hash from the bound preview.
    pub(crate) sha256: String,
    /// Original exact private byte count, capped independently at ten MiB.
    pub(crate) bytes: usize,
}

/// Actual admitted wire identity, retained without canonicalizing away method/path/query facts.
#[derive(Clone, Copy)]
pub(crate) struct StageRequestIdentity<'a> {
    /// Original same-session idempotency key validated by the selected operation contract.
    pub(crate) key: &'a str,
    /// Actual selected-major public path, including this stage identity for preview preparation.
    pub(crate) wire_path: &'a str,
    /// Actual admitted raw query string, never silently dropped from replay identity.
    pub(crate) raw_query: &'a str,
    /// Closed original request Value after strict raw duplicate/encoding/body admission.
    pub(crate) request: &'a Value,
}

/// Bound one actual retained JSON copy conservatively; never ignore encode failure.
pub(super) fn staged_value_charge(value: &Value) -> Result<usize> {
    contract::encode(value, 4 * 1024 * 1024, false)?.len().checked_mul(4).ok_or_else(capacity)
}

/// Stop complete retained-value enumeration as soon as its conservative floor exceeds the pool.
pub(super) fn staged_charge_sum(sum: usize, value: &Value) -> Result<usize> {
    let next = sum.checked_add(staged_value_charge(value)?).ok_or_else(capacity)?;
    if next > MAX_PREVIEW_BYTES {
        return Err(capacity());
    }
    Ok(next)
}

impl SourceStore {
    /// Charge every source Value, with the accepted complete control/result floor reserved before acceptance.
    fn staged_public_charge(
        &self,
        claims: &BTreeMap<RetentionOwner, RetentionClaim>,
    ) -> Result<usize> {
        let previews = self
            .receipts
            .values()
            .map(|record| &record.preview)
            .try_fold(0_usize, staged_charge_sum)?;
        self.jobs.iter().try_fold(previews, |sum, (id, job)| {
            let floor = claims
                .get(&RetentionOwner::Source(id.clone()))
                .map_or(0, |claim| claim.control_bytes / 2);
            sum.checked_add(staged_value_charge(&job.value)?.max(floor)).ok_or_else(capacity)
        })
    }
}

impl Store {
    /// Count retained entities once; separately stored replay Values remain separate entities.
    /// `capture_progress` and recovered IDs already represented by jobs are alias indexes.
    pub(crate) fn retained_entity_count(&self) -> Result<usize> {
        [
            self.receipt_count(),
            self.operation_count(),
            self.replays.len(),
            self.pending_replays.len(),
            self.transfers.retained_records(),
            self.retention_claims.values().try_fold(0_usize, |sum, claim| {
                sum.checked_add(claim.records).ok_or_else(capacity)
            })?,
        ]
        .into_iter()
        .try_fold(0_usize, |sum, count| sum.checked_add(count).ok_or_else(capacity))
    }

    /// Add a complete conservative wire-copy floor to the old private/generation ledger.
    /// The intentionally duplicated charge lowers usability; it is not a new pool or RSS bound.
    pub(crate) fn non_stage_retention(&self) -> Result<OtherPoolUsage> {
        let records = self
            .retained_entity_count()?
            .checked_sub(self.transfers.retained_records())
            .ok_or_else(capacity)?;
        if records > MAX_RETAINED {
            return Err(capacity());
        }
        let ordinary = self
            .receipts
            .values()
            .map(|record| &record.preview)
            .chain(self.replays.values().map(|record| &record.value))
            .try_fold(0_usize, staged_charge_sum)?;
        // This finite reserve covers bounded identifiers/map metadata separately from Values.
        // All returned local Stores and accepted native workers reserve additional complete claims here.
        let metadata = records.checked_mul(4096).ok_or_else(capacity)?;
        let operations = self.operations.iter().try_fold(0_usize, |sum, (id, value)| {
            let floor = self
                .retention_claims
                .get(&RetentionOwner::Ordinary(id.clone()))
                .map_or(0, |claim| claim.control_bytes / 2);
            sum.checked_add(staged_value_charge(value)?.max(floor)).ok_or_else(capacity)
        })?;
        let claims = self
            .retention_claims
            .values()
            .try_fold(0_usize, |sum, claim| sum.checked_add(claim.bytes).ok_or_else(capacity))?
            .checked_add(self.retention_claims.len().checked_mul(4096).ok_or_else(capacity)?)
            .ok_or_else(capacity)?;
        let bytes = self
            .retained_bytes
            .checked_add(claims)
            .and_then(|sum| sum.checked_add(ordinary))
            .and_then(|sum| sum.checked_add(operations))
            .and_then(|sum| {
                sum.checked_add(self.sources.staged_public_charge(&self.retention_claims).ok()?)
            })
            .and_then(|sum| sum.checked_add(metadata))
            .ok_or_else(capacity)?;
        Ok(OtherPoolUsage { bytes, records })
    }

    /// Check prospective complete usage before an old or new admission mutates retained state.
    /// The caller includes future complete wire copies/reservations, not only private Vec growth.
    pub(crate) fn admit_shared_retention(&self, non_stage: OtherPoolUsage) -> Result<()> {
        let bytes =
            non_stage.bytes.checked_add(self.transfers.retained_bytes()?).ok_or_else(capacity)?;
        let records = non_stage
            .records
            .checked_add(self.transfers.retained_records())
            .ok_or_else(capacity)?;
        if bytes > MAX_PREVIEW_BYTES
            || records > MAX_RETAINED
            || self.local_limit.is_some_and(|limit| bytes > limit.bytes || records > limit.records)
        {
            return Err(capacity());
        }
        Ok(())
    }

    /// Create one private stage and its original ready replay atomically under the Store lock.
    /// The parent strictly admits the original raw request and checks its original deadline first.
    pub(crate) fn create_source_stage(
        &mut self,
        identity: StageRequestIdentity<'_>,
        admitted: StageRequest,
        observed: Instant,
        utc: chrono::DateTime<chrono::Utc>,
    ) -> Result<(Reply, Vec<u8>)> {
        let StageRequestIdentity { key, wire_path, raw_query, request } = identity;
        if let Some(reply) = self.replay(key, "POST", wire_path, raw_query, request)? {
            let bytes = contract::encode(&reply.value, 1024 * 1024, false)?;
            return Ok((reply, bytes));
        }
        if !admitted.matches(request)? {
            return Err(Error::invalid());
        }
        let current = self.non_stage_retention()?;
        let mut local = SourceTransfers::default();
        let total_existing = OtherPoolUsage {
            bytes: current
                .bytes
                .checked_add(self.transfers.retained_bytes()?)
                .ok_or_else(capacity)?,
            records: current
                .records
                .checked_add(self.transfers.retained_records())
                .ok_or_else(capacity)?,
        };
        let stage_id = id("bst")?;
        let value = local.create(&stage_id, admitted, observed, utc, total_existing)?;
        let reply = Reply { value, schema: "SourceTransferStage", status: 201 };
        contract::validate_for(contract::ApiMajor::V2, reply.schema, &reply.value)?;
        let bytes = contract::encode(&reply.value, 1024 * 1024, false)?;
        let charge = staged_value_charge(&reply.value)?;
        let retained = self.retained_bytes.checked_add(charge).ok_or_else(capacity)?;
        // The monotonic charge and actual retained replay copy are both conservatively charged.
        let future = OtherPoolUsage {
            bytes: current
                .bytes
                .checked_add(charge)
                .and_then(|sum| sum.checked_add(charge))
                .and_then(|sum| sum.checked_add(4096))
                .ok_or_else(capacity)?,
            records: current.records.checked_add(1).ok_or_else(capacity)?,
        };
        let ready_record = Replay {
            request_hash: request_hash("POST", wire_path, raw_query, request)?,
            value: reply.value.clone(),
            schema: reply.schema,
            status: reply.status,
        };
        // All fallible validation, encoding, charging and collision checks precede either map move.
        self.transfers.retain_created(future, local)?;
        self.replays.insert(key.to_owned(), ready_record);
        self.retained_bytes = retained;
        Ok((reply, bytes))
    }

    /// Install already decoded bytes without parser/WorkControl or project I/O under this lock.
    pub(crate) fn put_source_stage(
        &mut self,
        stage_id: &str,
        ordinal: usize,
        part: &DecodedPart,
        observed: Instant,
    ) -> Result<Value> {
        self.transfers.put(stage_id, ordinal, part, observed)
    }

    /// Observe actual counters without extending the original stage lifetime.
    pub(crate) fn source_stage(&mut self, stage_id: &str, observed: Instant) -> Result<Value> {
        self.transfers.status(stage_id, observed)
    }

    /// Discard unconfirmed transport only; held raw allocations remain in shared accounting.
    pub(crate) fn discard_source_stage(
        &mut self,
        stage_id: &str,
        observed: Instant,
    ) -> Result<Value> {
        self.transfers.discard(stage_id, observed)
    }

    /// Acquire one fresh attempt generation; the caller immediately releases the Store lock.
    pub(crate) fn lease_source_stage(
        &mut self,
        stage_id: &str,
        observed: Instant,
    ) -> Result<PreparationLease> {
        self.transfers.begin_preparation(stage_id, observed)
    }

    /// Abandon only the exact released lease ticket, never a newer preparation attempt.
    pub(crate) fn abandon_source_stage(
        &mut self,
        ticket: &PreparationTicket,
        observed: Instant,
    ) -> Result<()> {
        self.transfers.abandon_preparation(ticket, observed)
    }

    /// Observe retirement outside callback-owned Store locks; held lease bytes remain charged.
    pub(crate) fn source_stage_preparation_active(
        &self,
        ticket: &PreparationTicket,
        observed: Instant,
    ) -> Result<bool> {
        self.transfers.preparation_active(ticket, observed)
    }

    /// Transfer a fully prepared local source receipt and ready replay in one shared mutation.
    /// No optimistic raw-drop credit is taken; a retained closure has no fallible post-insertion work.
    pub(crate) fn retain_staged_source_reserved(
        &mut self,
        ticket: &PreparationTicket,
        observed: Instant,
        identity: StageRequestIdentity<'_>,
        nonce: &str,
        mut local: Self,
        reply: &Reply,
    ) -> Result<()> {
        let StageRequestIdentity { key, wire_path, raw_query, request } = identity;
        let pending = self.pending_replays.get(key).ok_or_else(idempotency_conflict)?;
        if pending.kind != PendingKind::Preparation
            || pending.nonce != nonce
            || wire_path
                != format!("/api/v2/project/source-transfer-stages/{}/preview", ticket.stage_id())
            || pending.request_hash != request_hash("POST", wire_path, raw_query, request)?
            || self.replays.contains_key(key)
        {
            return Err(idempotency_conflict());
        }
        if !local.receipts.is_empty()
            || !local.operations.is_empty()
            || !local.replays.is_empty()
            || !local.pending_replays.is_empty()
            || !local.retention_claims.is_empty()
            || local.sources.operation_count() != 0
            || local.sources.receipt_count() != 1
            || local.transfers.retained_records() != 0
            || local.sources.receipts.keys().any(|id| self.sources.receipts.contains_key(id))
            || reply.status != 200
            || reply.schema != "ProjectSourceBundleImportPreview"
        {
            return Err(capacity());
        }
        contract::validate_for(contract::ApiMajor::V2, reply.schema, &reply.value)?;
        let owner = RetentionOwner::Preparation(nonce.into());
        let old = self.usage_without_claim(&owner)?;
        let local_usage = local.non_stage_retention()?;
        let plan_bytes = local_usage
            .bytes
            .checked_add(staged_value_charge(&reply.value)?)
            .ok_or_else(capacity)?;
        // The existing pending entity is replaced by one replay, so its count does not grow.
        let plan_records = local_usage.records;
        let retained =
            self.retained_bytes.checked_add(local.retained_bytes).ok_or_else(capacity)?;
        let ready_record = Replay {
            request_hash: pending.request_hash.clone(),
            value: reply.value.clone(),
            schema: reply.schema,
            status: reply.status,
        };
        let Self {
            transfers,
            sources,
            replays,
            pending_replays,
            retained_bytes,
            retention_claims,
            ..
        } = self;
        transfers.retain_prepared(ticket, observed, old, plan_bytes, plan_records, || {
            sources.receipts.append(&mut local.sources.receipts);
            replays.insert(key.to_owned(), ready_record);
            pending_replays.remove(key);
            retention_claims.remove(&owner);
            *retained_bytes = retained;
            Ok(())
        })
    }

    /// Reconcile actual unconfirmed storage drops during ordinary Store housekeeping.
    /// Accepted/native authorities elsewhere are never swept, cancelled or credited here.
    pub(crate) fn sweep_source_stages(&mut self, observed: Instant) -> Result<()> {
        self.transfers.sweep(observed)
    }

    /// Copy only bounded descriptor facts under Store; actual target I/O happens outside it.
    /// Old inline getters keep selecting their original private family and cannot serve Bundle4.
    pub(crate) fn staged_export_descriptor(
        &self,
        operation_id: &str,
        observed: Instant,
    ) -> Result<StagedExportDescriptor> {
        let operation = self.operations.get(operation_id).ok_or_else(unavailable)?;
        if operation["kind"] != "export" {
            return Err(unavailable());
        }
        let id = operation["result"]["preview"]["preview_id"].as_str().ok_or_else(unavailable)?;
        let receipt = self.receipts.get(id).ok_or_else(unavailable)?;
        if !receipt.committed
            || receipt.artifact_family != Some(ArtifactFamily::StagedSourceBundleJson)
            || observed
                .checked_duration_since(receipt.issued)
                .is_none_or(|elapsed| elapsed >= RECEIPT_LIFETIME)
            || receipt.bytes.is_empty()
            || receipt.bytes.len() > crate::workspace::source_transfers::MAX_ARTIFACT_BYTES
        {
            return Err(unavailable());
        }
        let sha256 = receipt.preview["exact_bytes_sha256"].as_str().ok_or_else(unavailable)?;
        Ok(StagedExportDescriptor {
            path: receipt.target.path.clone(),
            sha256: sha256.into(),
            bytes: receipt.bytes.len(),
        })
    }
}

/// Actual-method Store controls; no HTTP/native/confirmation acceptance is inferred.
#[cfg(test)]
mod staged_adapter_tests {
    use super::*;
    use crate::workspace::preparation::NoopControl;
    use crate::workspace::source_transfers::PART_BYTES;

    /// Describe one exact inert raw artifact with both explicit sensitivity acknowledgments.
    fn declaration(raw: &[u8]) -> Value {
        json!({"schema_version":"forge.workspace-index-bundle/4",
            "profile":"index-and-source-hex-staged","artifact_sha256":crate::hashing::sha256_hex(raw),
            "artifact_size_bytes":raw.len(),"chunk_size_bytes":PART_BYTES,
            "chunk_count":raw.len().div_ceil(PART_BYTES),"acknowledge_sensitive_metadata":true,
            "acknowledge_source_content":true})
    }

    /// Enter through the actual atomic stage/replay adapter, returning its original public facts.
    fn create(store: &mut Store, raw: &[u8], observed: Instant) -> (Reply, Vec<u8>) {
        let request = declaration(raw);
        store
            .create_source_stage(
                StageRequestIdentity {
                    key: "stage-create-control-0001",
                    wire_path: "/api/v2/project/source-transfer-stages",
                    raw_query: "",
                    request: &request,
                },
                StageRequest::parse(&request).unwrap(),
                observed,
                chrono::Utc::now(),
            )
            .unwrap()
    }

    /// Install genuine byte transport through the actual decoder and Store PUT method.
    fn ready(store: &mut Store, raw: &[u8], observed: Instant) -> String {
        let (reply, _) = create(store, raw, observed);
        let stage_id = reply.value["stage_id"].as_str().unwrap().to_owned();
        for (ordinal, bytes) in raw.chunks(PART_BYTES).enumerate() {
            use std::fmt::Write as _;
            let mut hex = String::new();
            for byte in bytes {
                write!(hex, "{byte:02x}").unwrap();
            }
            let part = DecodedPart::parse(
                &json!({"sha256":crate::hashing::sha256_hex(bytes),
                "size_bytes":bytes.len(),"hex":hex}),
                &mut NoopControl,
            )
            .unwrap();
            store.put_source_stage(&stage_id, ordinal, &part, observed).unwrap();
        }
        stage_id
    }

    /// Same key/body returns the exact original ready bytes and never creates a second stage.
    #[test]
    fn create_replay_is_original_and_changed_request_retains_no_new_stage() {
        let mut store = Store::default();
        let observed = Instant::now();
        let (first, first_bytes) = create(&mut store, b"{}", observed);
        let (again, again_bytes) = create(&mut store, b"{}", observed + Duration::from_secs(1));
        assert_eq!(first.value, again.value);
        assert_eq!(first_bytes, again_bytes);
        assert_eq!(store.retained_entity_count().unwrap(), 2);
        let changed = declaration(b"[]");
        let error = store
            .create_source_stage(
                StageRequestIdentity {
                    key: "stage-create-control-0001",
                    wire_path: "/api/v2/project/source-transfer-stages",
                    raw_query: "",
                    request: &changed,
                },
                StageRequest::parse(&changed).unwrap(),
                observed,
                chrono::Utc::now(),
            )
            .err()
            .unwrap();
        assert_eq!(error.code, "idempotency-key-conflict");
        assert_eq!(store.transfers.retained_records(), 1);
    }

    /// Distinct retained Values count separately; an alias capture index does not add an entity.
    /// Synthetic values isolate accounting and are not advertised as authentic Operation DTOs.
    #[test]
    fn aggregate_entities_refuse_before_stage_or_replay_insertion() {
        let mut store = Store::default();
        for ordinal in 0..MAX_RETAINED {
            store.operations.insert(format!("op_fixture_{ordinal}"), json!({}));
        }
        store.capture_progress.insert("op_fixture_0".into(), (0, 1));
        assert_eq!(store.retained_entity_count().unwrap(), MAX_RETAINED);
        let request = declaration(b"{}");
        let before = store.retained_bytes;
        let result = store.create_source_stage(
            StageRequestIdentity {
                key: "stage-create-control-0001",
                wire_path: "/api/v2/project/source-transfer-stages",
                raw_query: "",
                request: &request,
            },
            StageRequest::parse(&request).unwrap(),
            Instant::now(),
            chrono::Utc::now(),
        );
        assert!(result.is_err());
        assert_eq!(store.retained_bytes, before);
        assert!(store.replays.is_empty());
        assert_eq!(store.transfers.retained_records(), 0);
    }

    /// A different pre-admitted descriptor cannot be bound to this actual replay request.
    #[test]
    fn create_refuses_cross_request_admission_before_any_retention() {
        let mut store = Store::default();
        let request = declaration(b"{}");
        let other = declaration(b"[]");
        let result = store.create_source_stage(
            StageRequestIdentity {
                key: "stage-create-control-0001",
                wire_path: "/api/v2/project/source-transfer-stages",
                raw_query: "",
                request: &request,
            },
            StageRequest::parse(&other).unwrap(),
            Instant::now(),
            chrono::Utc::now(),
        );
        assert!(result.is_err());
        assert_eq!(store.transfers.retained_records(), 0);
        assert!(store.replays.is_empty());
    }

    /// Actual held raw bytes remain charged across discard until the final lease releases them.
    #[test]
    fn discarded_held_raw_storage_has_no_optimistic_shared_credit() {
        let mut store = Store::default();
        let observed = Instant::now();
        let raw = vec![b' '; PART_BYTES + 7];
        let id = ready(&mut store, &raw, observed);
        let lease = store.lease_source_stage(&id, observed).unwrap();
        let before = store.transfers.retained_bytes().unwrap();
        store.discard_source_stage(&id, observed).unwrap();
        store.sweep_source_stages(observed).unwrap();
        assert_eq!(store.transfers.retained_bytes().unwrap(), before);
        assert!(!store.source_stage_preparation_active(lease.ticket(), observed).unwrap());
        drop(lease);
        store.sweep_source_stages(observed).unwrap();
        assert_eq!(before - store.transfers.retained_bytes().unwrap(), raw.len());
    }

    /// Future old-route additions cannot ignore currently held staging storage.
    #[test]
    fn whole_shared_bytes_refuse_an_other_pool_overfill() {
        let mut store = Store::default();
        create(&mut store, b"{}", Instant::now());
        let stage_bytes = store.transfers.retained_bytes().unwrap();
        assert!(
            store
                .admit_shared_retention(OtherPoolUsage {
                    bytes: MAX_PREVIEW_BYTES - stage_bytes + 1,
                    records: 1
                })
                .is_err()
        );
        assert!(
            store
                .admit_shared_retention(OtherPoolUsage {
                    bytes: MAX_PREVIEW_BYTES - stage_bytes,
                    records: 1
                })
                .is_ok()
        );
    }

    /// A malformed local handoff cannot consume the pending owner or publish a receipt/replay.
    #[test]
    fn invalid_staged_handoff_is_atomic_and_preserves_the_preparation_owner() {
        let mut store = Store::default();
        let observed = Instant::now();
        let id = ready(&mut store, b"{}", observed);
        let request = json!({"target_index_schema_version":2,"acknowledge_index_replacement":true,
            "acknowledge_source_content":true,"acknowledge_replace_files":true});
        let path = format!("/api/v2/project/source-transfer-stages/{id}/preview");
        let nonce =
            store.reserve("stage-preview-control-0001", "POST", &path, "", &request).unwrap();
        let ticket = store.lease_source_stage(&id, observed).unwrap().into_ticket();
        let reply =
            Reply { value: json!({}), schema: "ProjectSourceBundleImportPreview", status: 200 };
        assert!(
            store
                .retain_staged_source_reserved(
                    &ticket,
                    observed,
                    StageRequestIdentity {
                        key: "stage-preview-control-0001",
                        wire_path: &path,
                        raw_query: "",
                        request: &request
                    },
                    &nonce,
                    Store::default(),
                    &reply
                )
                .is_err()
        );
        assert_eq!(store.source_stage(&id, observed).unwrap()["state"], "preparing");
        assert!(store.pending_replays.contains_key("stage-preview-control-0001"));
        assert!(!store.replays.contains_key("stage-preview-control-0001"));
        assert_eq!(store.sources.receipt_count(), 0);
        store.abandon_source_stage(&ticket, observed).unwrap();
    }

    /// A ticket for one stage cannot settle a reservation made for another public stage path.
    #[test]
    fn stage_ticket_and_wire_reservation_must_name_the_same_stage() {
        let mut store = Store::default();
        let observed = Instant::now();
        let id = ready(&mut store, b"{}", observed);
        let request = json!({"target_index_schema_version":2,"acknowledge_index_replacement":true,
            "acknowledge_source_content":true,"acknowledge_replace_files":true});
        let wrong_path = "/api/v2/project/source-transfer-stages/bst_0123456789ab/preview";
        let nonce =
            store.reserve("stage-preview-control-0002", "POST", wrong_path, "", &request).unwrap();
        let ticket = store.lease_source_stage(&id, observed).unwrap().into_ticket();
        let reply =
            Reply { value: json!({}), schema: "ProjectSourceBundleImportPreview", status: 200 };
        let result = store.retain_staged_source_reserved(
            &ticket,
            observed,
            StageRequestIdentity {
                key: "stage-preview-control-0002",
                wire_path: wrong_path,
                raw_query: "",
                request: &request,
            },
            &nonce,
            Store::default(),
            &reply,
        );
        assert_eq!(result.err().unwrap().code, "idempotency-key-conflict");
        assert!(store.pending_replays.contains_key("stage-preview-control-0002"));
        assert_eq!(store.source_stage(&id, observed).unwrap()["state"], "preparing");
        store.abandon_source_stage(&ticket, observed).unwrap();
    }
}

//! Consumed source preparation/confirmation ports for the bounded local HTTP runtime.
//!
//! The parent admits selected major, closed operation, session
//! capability, read-only policy and the original body/queue deadline first.

use super::*;
use crate::workspace::root::TransactionState;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// Require independently qualified durable restore authority, never a public path.
fn transactions(state: &State) -> Result<&TransactionState> {
    state.restores.as_ref().ok_or_else(|| {
        Error::new(
            "bundle-restore-unavailable",
            "Qualified source restore is unavailable for this project.",
            false,
        )
    })
}

/// A participating reader acquires its Root-owned lease before Store or project I/O.
/// Pure status/cancellation never uses this lease; arbitrary CLI/editor I/O is external.
pub(super) fn project_read(state: &State) -> Result<RwLockReadGuard<'_, ()>> {
    let lease = state.project_io.try_read().map_err(|_| {
        Error::new("invalid-request", "Project data is busy. Retry the read shortly.", true)
    })?;
    if let Some(restores) = &state.restores {
        restores.require_access()?;
    }
    Ok(lease)
}

/// Existing single-file commits acquire the same exclusive participating lease
/// before the shared Store lock; this ordering avoids worker/progress deadlock.
pub(super) fn project_write(state: &State) -> Result<RwLockWriteGuard<'_, ()>> {
    let lease = state.project_io.try_write().map_err(|_| {
        Error::new(
            "invalid-request",
            "Project data is busy. Retry after inspecting active operations.",
            true,
        )
    })?;
    if let Some(restores) = &state.restores {
        restores.require_access()?;
    }
    Ok(lease)
}

/// Confirmed restore control retains the original forward stop and consumes
/// cancellation/progress under a short Store lock, outside journal/project I/O.
struct SourceControl<'a> {
    /// One current authenticated session and its original accepted operation.
    state: &'a State,
    /// Preview-authored nonauthorizing lookup ID, never a new worker ID.
    operation_id: &'a str,
    /// The original body/dispatch/queue deadline, never renewed by a worker.
    deadline: Instant,
    /// First observed shutdown/deadline/cancellation wins permanently.
    interruption: Option<Interruption>,
}

impl WorkControl for SourceControl<'_> {
    /// Check before real native boundaries; counts represent measured files only.
    fn checkpoint(&mut self, _stage: Stage, update: ProgressUpdate) -> WorkResult<()> {
        if let Some(reason) = self.interruption {
            return Err(WorkError::Interrupted(reason));
        }
        let mut reason = if self.state.stopped.load(Ordering::Acquire) {
            Some(Interruption::Shutdown)
        } else if Instant::now() >= self.deadline {
            Some(Interruption::DeadlineExceeded)
        } else {
            None
        };
        if reason.is_none() {
            let mut store = self.state.effects.lock().map_err(|_| internal())?;
            reason = if self.state.stopped.load(Ordering::Acquire) {
                Some(Interruption::Shutdown)
            } else if Instant::now() >= self.deadline {
                Some(Interruption::DeadlineExceeded)
            } else if !store.source_progress(self.operation_id, update)? {
                Some(Interruption::CancelRequested)
            } else {
                None
            };
        }
        if let Some(reason) = reason {
            self.interruption = Some(reason);
            return Err(WorkError::Interrupted(reason));
        }
        Ok(())
    }

    /// Return the original latched stop without granting renewed forward work.
    fn interruption(&self) -> Option<Interruption> {
        self.interruption
    }
}

/// Encode one closed source reply completely before final shared retention.
fn source_reply_response(reply: &super::super::effects::Reply) -> Result<Response<Full<Bytes>>> {
    contract::validate_for(ApiMajor::V2, reply.schema, &reply.value).map_err(|_| internal())?;
    let bytes = contract::encode(&reply.value, MAX_BODY, false)?;
    Ok(response(reply.status, "application/json", bytes))
}

/// Retain the fully encoded source-export acceptance before worker dispatch,
/// preserving the absolute body/queue budget and immutable same-session replay.
pub(super) fn export_response(
    state: &Arc<State>,
    key: &str,
    wire_path: &str,
    raw_query: &str,
    request: &Value,
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let deadline = deadline.ok_or_else(|| {
        Error::new(
            "bundle-preparation-budget-exceeded",
            "Source export has no available original work budget.",
            false,
        )
    })?;
    let (reply, bytes, permit) = {
        let mut store = state.effects.lock().map_err(|_| internal())?;
        if state.stopped.load(Ordering::Acquire) || Instant::now() >= deadline {
            return Err(Error::new(
                "bundle-preparation-budget-exceeded",
                "Source export stopped before acceptance.",
                false,
            ));
        }
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            return source_reply_response(&reply);
        }
        let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "Another operation is running. Retry shortly.", true)
        })?;
        let (reply, bytes) = store.accept_source_export(key, wire_path, raw_query, request)?;
        (reply, bytes, permit)
    };
    let id = reply.value["operation_id"].as_str().ok_or_else(internal)?.to_owned();
    let shared = Arc::clone(state);
    let request = request.clone();
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let proceed = shared
            .effects
            .lock()
            .ok()
            .and_then(|mut store| store.running(&id).ok())
            .unwrap_or(false);
        if !proceed {
            return;
        }
        let mut control = OperationControl {
            state: &shared,
            id: &id,
            deadline: Some(deadline),
            interruption: None,
        };
        let result = (|| {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let mut local = super::super::effects::Store::default();
            let completed =
                prepare_export(&shared, &mut local, &request, &reply.value, &mut control)?;
            Ok((local, completed))
        })();
        let result = match control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear) {
            Ok(()) => result,
            Err(error) => Err(error),
        };
        if let Ok(mut store) = shared.effects.lock() {
            let cancelled = control.interruption().is_some()
                || shared.stopped.load(Ordering::Acquire)
                || Instant::now() >= deadline;
            let _ = store.finish(&id, result.map_err(WorkError::into_error), cancelled);
        }
    });
    Ok(response(202, "application/json", bytes))
}

/// Prepare all supplied source bytes under one shared read lease and original
/// ten-second body/queue budget, then atomically retain its complete wrapper.
pub(super) fn import_response(
    state: &Arc<State>,
    key: &str,
    wire_path: &str,
    raw_query: &str,
    request: &Value,
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let mut control = ReadControl { state, deadline, interruption: None };
    control
        .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    let _io = project_read(state)?;
    // A port that cannot preserve trusted accepted/recovery state has no restore
    // capability; preparation never substitutes a weaker path-copy fallback.
    let _qualified = transactions(state)?;
    let (nonce, _permit) = {
        let mut store = state.effects.lock().map_err(|_| internal())?;
        control
            .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
            .map_err(bundle_preparation_error)?;
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            let response = source_reply_response(&reply)?;
            control
                .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
                .map_err(bundle_preparation_error)?;
            return Ok(response);
        }
        let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "Another operation is running. Retry shortly.", true)
        })?;
        (store.reserve(key, "POST", wire_path, raw_query, request)?, permit)
    };
    let reservation = ReservationGuard { state, key, nonce };
    let prepared: WorkResult<_> = (|| {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let raw = contract::encode(
            &request["bundle"],
            super::super::source_bundles::MAX_SOURCE_BUNDLE_BYTES,
            false,
        )?;
        let decoded = super::super::source_bundles::decode(&raw, &mut control)?;
        let snapshot = Snapshot::capture_source_bundle_effect_with_control(
            &state.root,
            Some(&decoded.index),
            None,
            &mut control,
        )?;
        let plan = super::super::source_bundle_effects::prepare_import(
            &state.root,
            snapshot,
            decoded,
            request,
            &mut control,
        )?;
        control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
        let mut local = super::super::effects::Store::default();
        let (preview, replacement) = local.preview_source_restore(plan)?;
        let reply = super::super::effects::Reply {
            value: json!({"validation":super::super::services::validation(true,None),"preview":preview,"replacement":replacement}),
            schema: "ProjectSourceBundleImportPreview",
            status: 200,
        };
        let response = source_reply_response(&reply)?;
        local.charge_reply(&reply)?;
        control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)?;
        Ok((local, reply, response))
    })();
    let (local, reply, response) = prepared.map_err(bundle_preparation_error)?;
    let mut store = state.effects.lock().map_err(|_| internal())?;
    control
        .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    store.retain_source_reserved(key, &reservation.nonce, local, &reply)?;
    Ok(response)
}

/// Keep safe lookup/cancellation reachable while the native worker owns project I/O.
pub(super) fn outcome_response(
    state: &State,
    operation_id: &str,
    cancel: bool,
) -> Result<Response<Full<Bytes>>> {
    let durable = transactions(state)?.observe_outcome(operation_id)?;
    let mut store = state.effects.lock().map_err(|_| internal())?;
    let value = if cancel {
        store.cancel_source_restore(operation_id, durable)?
    } else {
        store.source_operation(operation_id, durable)?
    };
    json_response_for(ApiMajor::V2, value, "SourceRestoreOperation")
}

/// Confirm an exact one-time source receipt and make accepted intent plus the
/// already encoded original reply durable before worker dispatch or HTTP202.
pub(super) fn commit_response(
    state: &Arc<State>,
    preview_id: &str,
    key: &str,
    wire_path: &str,
    raw_query: &str,
    request: &Value,
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let deadline = deadline.ok_or_else(|| {
        Error::new(
            "bundle-restore-budget-exceeded",
            "Source restore has no available original work budget.",
            false,
        )
    })?;
    if state.stopped.load(Ordering::Acquire) || Instant::now() >= deadline {
        return Err(Error::new(
            "bundle-restore-budget-exceeded",
            "Source restore stopped before acceptance.",
            false,
        ));
    }
    let native = transactions(state)?;
    // A ready same-session original reply remains replayable while its accepted
    // worker owns unresolved project access; replay creates no forward authority.
    {
        let store = state.effects.lock().map_err(|_| internal())?;
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            return source_reply_response(&reply);
        }
    }
    native.require_access()?;
    let (acceptance, permit) = {
        let mut store = state.effects.lock().map_err(|_| internal())?;
        if state.stopped.load(Ordering::Acquire) || Instant::now() >= deadline {
            return Err(Error::new(
                "bundle-restore-budget-exceeded",
                "Source restore stopped before acceptance.",
                false,
            ));
        }
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            return source_reply_response(&reply);
        }
        let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
            Error::new(
                "bundle-restore-in-progress",
                "Another project operation is running. Inspect it before confirmation.",
                true,
            )
        })?;
        let acceptance =
            store.take_source_confirmation(preview_id, key, wire_path, raw_query, request)?;
        (acceptance, permit)
    };
    // No generic reservation-drop guard applies after this boundary: an error
    // may follow actual journal publication and is not proof of no accepted intent.
    let accepted = match state.root.accept_restore(
        &acceptance.plan,
        native,
        acceptance.operation_id.clone(),
        acceptance.native_nonce,
        acceptance.request_sha256.clone(),
        acceptance.exact_manifest_sha256.clone(),
        acceptance.accepted_reply.clone(),
        deadline,
    ) {
        Ok(accepted) => accepted,
        Err(error) => {
            if let Ok(mut store) = state.effects.lock() {
                let _ = store.source_acceptance_uncertain(
                    &acceptance.operation_id,
                    &acceptance.reservation_nonce,
                );
            }
            return Err(error);
        }
    };
    let original = accepted.accepted_reply_bytes().to_vec();
    // The byte-identical pending reply was schema-validated before acceptance;
    // no new serializer or fallible transformation can misreport the native handoff.
    let retained = state
        .effects
        .lock()
        .map_err(|_| internal())
        .and_then(|mut store| store.source_accepted(key, &acceptance));
    if retained.is_err() {
        if let Ok(mut store) = state.effects.lock() {
            let _ = store.source_acceptance_uncertain(
                &acceptance.operation_id,
                &acceptance.reservation_nonce,
            );
        }
    }
    dispatch_restore(state, acceptance.operation_id, acceptance.plan, accepted, permit, deadline);
    retained?;
    Ok(response(202, "application/json", original))
}

/// Own accepted work through eventual project leasing or durable recovery deferral.
/// The original deadline and a separately bounded settlement queue survive dispatch.
fn dispatch_restore(
    state: &Arc<State>,
    operation_id: String,
    plan: crate::workspace::root::RestoreTargetPlan,
    accepted: crate::workspace::root::AcceptedRestore,
    permit: tokio::sync::OwnedSemaphorePermit,
    deadline: Instant,
) {
    let shared = Arc::clone(state);
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let mut control = SourceControl {
            state: &shared,
            operation_id: &operation_id,
            deadline,
            interruption: None,
        };
        // The worker is always the owner of one eventual exclusive lease, even
        // when its forward budget stops while queued. Native settlement receives
        // the latched stop; acquiring a lease never renews publication authority.
        let mut settlement_queue_deadline = None;
        let _io = loop {
            match shared.project_io.try_write() {
                Ok(lease) => break lease,
                Err(std::sync::TryLockError::WouldBlock) => {
                    if control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).is_err()
                    {
                        let end = settlement_queue_deadline.get_or_insert_with(|| {
                            Instant::now().checked_add(Duration::from_secs(30))
                        });
                        if end.is_none_or(|end| Instant::now() >= end) {
                            if let Some(native) = shared.restores.as_ref() {
                                let _ = shared.root.defer_restore_recovery(native, &accepted);
                            }
                            return;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(std::sync::TryLockError::Poisoned(_)) => {
                    if let Some(native) = shared.restores.as_ref() {
                        let _ = shared.root.defer_restore_recovery(native, &accepted);
                    }
                    return;
                }
            }
        };
        if let Some(native) = shared.restores.as_ref() {
            // Native journal decides terminal truth, never this worker's fallible
            // return or a generic Operation finish transition after publication.
            let _ = shared.root.run_restore(plan, native, accepted, &mut control);
        }
    });
}

/// Prepare a source export using the existing normal export operation/receipt;
/// the source-only pre-capture output slot and private media family are required.
pub(super) fn prepare_export(
    state: &State,
    local: &mut super::super::effects::Store,
    request: &Value,
    operation: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<super::super::effects::Reply> {
    let _io = project_read(state)?;
    let output = request["target_path"].as_str().ok_or_else(Error::invalid)?;
    let snapshot = Snapshot::capture_source_bundle_effect_with_control(
        &state.root,
        None,
        Some(output),
        control,
    )?;
    let plan = super::super::source_bundle_effects::prepare_export(
        &state.root,
        &snapshot,
        request,
        control,
    )?;
    let preview = local.preview_bundle(plan)?;
    let mut completed = operation.clone();
    completed["state"] = json!("succeeded");
    completed["updated_at"] =
        json!(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    completed["progress"] = Value::Null;
    completed["result"] = json!({"operation_id":operation["operation_id"],"preview":preview,"redaction_summary":{"removed_categories":[]}});
    contract::validate_for(ApiMajor::V2, "Operation", &completed)?;
    let reply = super::super::effects::Reply { value: completed, schema: "Operation", status: 202 };
    local.charge_reply(&reply)?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::super::super::preparation::NoopControl;
    use super::*;

    /// Exercise the consumed source producer and complete old export envelope,
    /// keeping actual donor inputs unchanged and no artifact published.
    #[test]
    fn complete_source_export_reply_preserves_required_redaction_summary() {
        let project = tempfile::tempdir().unwrap();
        let index = super::super::super::index::Index::empty().bytes().unwrap();
        std::fs::write(project.path().join("forge.workspace.json"), &index).unwrap();
        let root = Root::open(project.path()).unwrap();
        let state = State {
            api_major: ApiMajor::V2,
            root,
            project_io: std::sync::RwLock::new(()),
            restores: None,
            host: "127.0.0.1:1".into(),
            origin: "http://127.0.0.1:1".into(),
            session: std::sync::Mutex::new(Session::new(Mode::Machine, false, None).unwrap()),
            stopped: AtomicBool::new(false),
            rate: std::sync::Mutex::new((Instant::now(), 0)),
            effects: std::sync::Mutex::new(super::super::super::effects::Store::default()),
            work: Arc::new(tokio::sync::Semaphore::new(2)),
            jobs: Arc::new(tokio::sync::Semaphore::new(1)),
        };
        let operation = state.effects.lock().unwrap().begin("export").unwrap();
        let mut local = super::super::super::effects::Store::default();
        let request = json!({"target_path":"source.json","acknowledge_sensitive_metadata":true,"acknowledge_source_content":true});
        let reply =
            prepare_export(&state, &mut local, &request, &operation, &mut NoopControl).unwrap();
        contract::validate_for(ApiMajor::V2, "Operation", &reply.value).unwrap();
        contract::validate_for(ApiMajor::V1, "Operation", &reply.value).unwrap();
        assert_eq!(reply.value["result"]["redaction_summary"]["removed_categories"], json!([]));
        assert_eq!(reply.value["result"]["operation_id"], operation["operation_id"]);
        assert!(
            reply.value["result"]["preview"]["semantic_summary"]
                .as_str()
                .unwrap()
                .contains("exact registered source bytes")
        );
        assert_eq!(std::fs::read(project.path().join("forge.workspace.json")).unwrap(), index);
        assert!(!project.path().join("source.json").exists());
    }
}

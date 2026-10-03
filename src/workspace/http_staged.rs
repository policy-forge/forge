//! Consumed staged-source HTTP adapters for exact selected API2.4 only.
//!
//! The parent keeps raw duplicate-safe/media/body admission, capability/Host/Origin
//! checks and immutable admission deadlines. No private path rewrite is accepted here.

use super::*;
use crate::workspace::effects::{Reply, StageRequestIdentity, Store};
use crate::workspace::source_transfers::{
    DecodedPart, PreparationLease, PreparationTicket, StageRequest,
};

/// The eight exact public operations, carrying only contract-admitted opaque path values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route<'a> {
    /// Create one same-session acknowledged artifact declaration.
    Create,
    /// Put exactly one part into an existing unconfirmed transport stage.
    Put {
        /// Original opaque stage identifier, never a path.
        stage_id: &'a str,
        /// Bounded contract integer ordinal; the original wire spelling remains separate.
        ordinal: usize,
    },
    /// Observe complete current-session transport facts without a project lease.
    Status {
        /// Original opaque stage identifier.
        stage_id: &'a str,
    },
    /// Retire only unconfirmed transport, never accepted native work.
    Discard {
        /// Original opaque stage identifier.
        stage_id: &'a str,
    },
    /// Prepare a complete native receipt from one exact leased raw artifact.
    Preview {
        /// The same stage identity later checked against the lease ticket.
        stage_id: &'a str,
    },
    /// Prepare an asynchronous private Bundle4 export using existing confirmation.
    Export,
    /// Read the manifest of an actual same-session committed private family.
    Manifest {
        /// Producer-authored operation lookup, never download authority on its own.
        operation_id: &'a str,
    },
    /// Read one exact part only after verifying the whole committed generation.
    Part {
        /// Same-session producer operation whose private receipt binds the file.
        operation_id: &'a str,
        /// Bounded contract integer ordinal; the original wire spelling remains separate.
        ordinal: usize,
    },
}

/// Route only the new selected-major families; the parent validates operations first.
pub(super) fn handles(method: &str, wire_path: &str) -> bool {
    matches!(method, "GET" | "POST" | "PUT" | "DELETE")
        && (wire_path == "/api/v2/project/source-transfer-stages"
            || wire_path.starts_with("/api/v2/project/source-transfer-stages/")
            || wire_path == "/api/v2/project/source-stream-exports"
            || wire_path.starts_with("/api/v2/project/source-stream-exports/"))
}

/// Admit a literal opaque ID; percent encodings, separators and foreign prefixes remain invalid.
fn opaque_id<'a>(value: &'a str, prefix: &str) -> Result<&'a str> {
    let tail = value.strip_prefix(prefix).ok_or_else(Error::invalid)?;
    if !(12..=80).contains(&tail.len())
        || !tail.bytes().all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
    {
        return Err(Error::invalid());
    }
    Ok(value)
}

/// Parse the same unsigned integer representation as the contract, preserving the original wire path.
fn ordinal(value: &str) -> Result<usize> {
    let ordinal = value.parse::<usize>().map_err(|_| Error::invalid())?;
    if ordinal >= 320 {
        return Err(Error::invalid());
    }
    Ok(ordinal)
}

/// Resolve exact public segments; no API1 translation or slash trimming occurs.
fn route<'a>(method: &str, wire_path: &'a str) -> Result<Route<'a>> {
    if wire_path.len() > 2048 {
        return Err(Error::invalid());
    }
    let parts: Vec<_> = wire_path.split('/').collect();
    match parts.as_slice() {
        ["", "api", "v2", "project", "source-transfer-stages"] if method == "POST" => {
            Ok(Route::Create)
        }
        ["", "api", "v2", "project", "source-transfer-stages", id] => {
            let stage_id = opaque_id(id, "bst_")?;
            match method {
                "GET" => Ok(Route::Status { stage_id }),
                "DELETE" => Ok(Route::Discard { stage_id }),
                _ => Err(Error::invalid()),
            }
        }
        ["", "api", "v2", "project", "source-transfer-stages", id, "chunks", part]
            if method == "PUT" =>
        {
            Ok(Route::Put { stage_id: opaque_id(id, "bst_")?, ordinal: ordinal(part)? })
        }
        ["", "api", "v2", "project", "source-transfer-stages", id, "preview"]
            if method == "POST" =>
        {
            Ok(Route::Preview { stage_id: opaque_id(id, "bst_")? })
        }
        ["", "api", "v2", "project", "source-stream-exports"] if method == "POST" => {
            Ok(Route::Export)
        }
        ["", "api", "v2", "project", "source-stream-exports", id, "manifest"]
            if method == "GET" =>
        {
            Ok(Route::Manifest { operation_id: opaque_id(id, "op_")? })
        }
        ["", "api", "v2", "project", "source-stream-exports", id, "chunks", part]
            if method == "GET" =>
        {
            Ok(Route::Part { operation_id: opaque_id(id, "op_")?, ordinal: ordinal(part)? })
        }
        _ => Err(Error::invalid()),
    }
}

/// Encode the closed response before delivery, preserving the complete source schema and bound.
fn reply_response(reply: &Reply) -> Result<Response<Full<Bytes>>> {
    contract::validate_for(ApiMajor::V2, reply.schema, &reply.value)?;
    let bytes = contract::encode(&reply.value, MAX_BODY, false)?;
    Ok(response(reply.status, "application/json", bytes))
}

/// Repeat closed route/body/query/major admission for direct callers before any retained mutation.
/// Capability and raw-byte admission remain mandatory parent responsibilities, not this Value port.
pub(super) fn dispatch(
    state: &Arc<State>,
    method: &str,
    wire_path: &str,
    raw_query: &str,
    idempotency: Option<&str>,
    payload: Option<&Value>,
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    if state.api_major != ApiMajor::V2
        || state.api_major.version() != "2.4.0"
        || !raw_query.is_empty()
    {
        return Err(Error::invalid());
    }
    let selected = route(method, wire_path)?;
    contract::operation_request_for(ApiMajor::V2, method, wire_path, &[], idempotency, payload)?;
    if method != "GET" && state.session.lock().map_err(|_| internal())?.read_only {
        return Err(Error::new(
            "read-only-session",
            "This session does not permit project changes.",
            false,
        ));
    }
    let mut control = ReadControl { state, deadline, interruption: None };
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).map_err(|error| {
        if method == "GET" { query_error(error) } else { bundle_preparation_error(error) }
    })?;
    match selected {
        Route::Create => create_response(
            state,
            idempotency.ok_or_else(Error::invalid)?,
            wire_path,
            raw_query,
            payload.ok_or_else(Error::invalid)?,
            &mut control,
        ),
        Route::Put { stage_id, ordinal } => put_response(
            state,
            stage_id,
            ordinal,
            payload.ok_or_else(Error::invalid)?,
            &mut control,
        ),
        Route::Status { stage_id } => stage_response(state, stage_id, false, &mut control),
        Route::Discard { stage_id } => stage_response(state, stage_id, true, &mut control),
        Route::Preview { stage_id } => preview_response(
            state,
            stage_id,
            idempotency.ok_or_else(Error::invalid)?,
            wire_path,
            raw_query,
            payload.ok_or_else(Error::invalid)?,
            deadline,
        ),
        Route::Export => export_response(
            state,
            idempotency.ok_or_else(Error::invalid)?,
            wire_path,
            raw_query,
            payload.ok_or_else(Error::invalid)?,
            deadline,
        ),
        Route::Manifest { operation_id } => {
            stream_response(state, operation_id, None, &mut control)
        }
        Route::Part { operation_id, ordinal } => {
            stream_response(state, operation_id, Some(ordinal), &mut control)
        }
    }
}

/// Parse the fixed declaration off Store, then atomically create stage plus original replay.
fn create_response(
    state: &State,
    key: &str,
    wire_path: &str,
    raw_query: &str,
    request: &Value,
    control: &mut ReadControl<'_>,
) -> Result<Response<Full<Bytes>>> {
    control
        .checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)
        .map_err(bundle_preparation_error)?;
    let admitted = StageRequest::parse(request)?;
    control
        .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    let mut store = state.effects.lock().map_err(|_| internal())?;
    control
        .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    let identity = StageRequestIdentity { key, wire_path, raw_query, request };
    let (reply, bytes) =
        store.create_source_stage(identity, admitted, Instant::now(), chrono::Utc::now())?;
    control
        .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    Ok(response(reply.status, "application/json", bytes))
}

/// Decode/verify exactly one bounded part off Store before the live ordinal mutation.
fn put_response(
    state: &State,
    stage_id: &str,
    ordinal: usize,
    request: &Value,
    control: &mut ReadControl<'_>,
) -> Result<Response<Full<Bytes>>> {
    let part = DecodedPart::parse(request, control).map_err(bundle_preparation_error)?;
    let mut store = state.effects.lock().map_err(|_| internal())?;
    control
        .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    let value = store.put_source_stage(stage_id, ordinal, &part, Instant::now())?;
    let response = json_response_for(ApiMajor::V2, value, "SourceTransferChunkAck")?;
    control
        .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    Ok(response)
}

/// Read/discard transport under a short lock; neither route acquires project I/O or native authority.
fn stage_response(
    state: &State,
    stage_id: &str,
    discard: bool,
    control: &mut ReadControl<'_>,
) -> Result<Response<Full<Bytes>>> {
    let mut store = state.effects.lock().map_err(|_| internal())?;
    control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged).map_err(|error| {
        if discard { bundle_preparation_error(error) } else { query_error(error) }
    })?;
    let (value, schema) = if discard {
        (store.discard_source_stage(stage_id, Instant::now())?, "SourceTransferDiscarded")
    } else {
        (store.source_stage(stage_id, Instant::now())?, "SourceTransferStage")
    };
    let response = json_response_for(ApiMajor::V2, value, schema)?;
    control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged).map_err(|error| {
        if discard { bundle_preparation_error(error) } else { query_error(error) }
    })?;
    Ok(response)
}

/// Own every off-lock preview allocation until its exact generation is retained or abandoned.
struct PreviewOwner<'a> {
    /// Original same-session shared Store owner.
    state: &'a State,
    /// Actual admitted request key whose allowance is never released for a newer generation.
    key: &'a str,
    /// Original generation nonce returned by Store admission.
    nonce: String,
    /// Claimed local result allowance and all actual native generations/preview holders.
    local: Option<Store>,
    /// The complete raw allocation remains charged until this exact lease drops.
    lease: Option<PreparationLease>,
    /// Consumed raw-free identity for final retention or abandonment.
    ticket: Option<PreparationTicket>,
}

impl PreviewOwner<'_> {
    /// Drop the real raw holder before yielding the nonauthorizing same-generation ticket.
    fn release_raw(&mut self) {
        if let Some(lease) = self.lease.take() {
            self.ticket = Some(lease.into_ticket());
        }
    }
}

impl Drop for PreviewOwner<'_> {
    /// Actual holders drop before ticket abandonment and quota release, including unwinding exits.
    fn drop(&mut self) {
        self.release_raw();
        drop(self.local.take());
        if self.nonce.is_empty() {
            return;
        }
        if let Ok(mut store) = self.state.effects.lock() {
            if let Some(ticket) = self.ticket.as_ref() {
                let _ = store.abandon_source_stage(ticket, Instant::now());
            }
            store.release_reservation(self.key, &self.nonce);
        }
    }
}

/// Observe one stage generation off Store and latch explicit retirement as typed interruption.
struct StageControl<'state, 'ticket> {
    /// Original absolute deadline/shutdown controller, never recreated between parser phases.
    read: ReadControl<'state>,
    /// Exact existing raw-generation identity; no public or native authority.
    ticket: &'ticket PreparationTicket,
}

impl WorkControl for StageControl<'_, '_> {
    /// Release Store before producer work; recheck time after lock contention and stage retirement.
    fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
        self.read.checkpoint(stage, progress)?;
        let store = self.read.state.effects.lock().map_err(|_| internal())?;
        self.read.checkpoint(stage, progress)?;
        if !store.source_stage_preparation_active(self.ticket, Instant::now())? {
            self.read.interruption = Some(Interruption::CancelRequested);
            return Err(WorkError::Interrupted(Interruption::CancelRequested));
        }
        Ok(())
    }

    /// Preserve shutdown/deadline/retirement across ordinary domain failures and final boundaries.
    fn interruption(&self) -> Option<Interruption> {
        self.read.interruption()
    }
}

/// Lease one complete stage and claimed result allowance before off-lock decode/capture/preparation.
#[expect(
    clippy::too_many_lines,
    reason = "Keep replay-first admission and exact raw/local holder handoff adjacent to final retention fences."
)]
fn preview_response(
    state: &State,
    stage_id: &str,
    key: &str,
    wire_path: &str,
    raw_query: &str,
    request: &Value,
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let mut read = ReadControl { state, deadline, interruption: None };
    read.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    // A same-key ready reply survives prepared/raw-reclaimed stage state. It is not new native authority.
    {
        let store = state.effects.lock().map_err(|_| internal())?;
        read.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
            .map_err(bundle_preparation_error)?;
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            let response = reply_response(&reply)?;
            read.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
                .map_err(bundle_preparation_error)?;
            return Ok(response);
        }
    }
    let _io = source::project_read(state)?;
    let _qualified = state.restores.as_ref().ok_or_else(|| {
        Error::new(
            "bundle-restore-unavailable",
            "Qualified source restore is unavailable for this project.",
            false,
        )
    })?;
    let mut owner =
        PreviewOwner { state, key, nonce: String::new(), local: None, lease: None, ticket: None };
    let _permit = {
        let mut store = state.effects.lock().map_err(|_| internal())?;
        read.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
            .map_err(bundle_preparation_error)?;
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            let response = reply_response(&reply)?;
            read.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
                .map_err(bundle_preparation_error)?;
            return Ok(response);
        }
        let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "Another operation is running. Retry shortly.", true)
        })?;
        owner.nonce = store.reserve(key, "POST", wire_path, raw_query, request)?;
        owner.local = Some(store.preparation_store(key, &owner.nonce)?);
        owner.lease = Some(store.lease_source_stage(stage_id, Instant::now())?);
        permit
    };
    let (prepared, mut read) = {
        let lease = owner.lease.as_ref().ok_or_else(internal)?;
        let mut control = StageControl { read, ticket: lease.ticket() };
        let prepared: WorkResult<_> = (|| {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
            let decoded = lease.decode(&mut control)?;
            let snapshot = Snapshot::capture_source_bundle_effect_with_control(
                &state.root,
                Some(&decoded.index),
                None,
                &mut control,
            )?;
            let plan = crate::workspace::source_bundle_effects::prepare_staged_import(
                &state.root,
                snapshot,
                decoded,
                request,
                &mut control,
            )?;
            control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
            let local = owner.local.as_mut().ok_or_else(internal)?;
            let (preview, replacement) = local.preview_source_restore(plan)?;
            let reply = Reply {
                value: json!({"validation":crate::workspace::services::validation(true,None),
            "preview":preview,"replacement":replacement}),
                schema: "ProjectSourceBundleImportPreview",
                status: 200,
            };
            let response = reply_response(&reply)?;
            local.charge_reply(&reply)?;
            control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)?;
            Ok((reply, response))
        })();
        // A late typed stop takes precedence over a best-effort domain error.
        let prepared = match control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged) {
            Ok(()) => prepared,
            Err(error) => Err(error),
        };
        (prepared, control.read)
    };
    owner.release_raw();
    let (reply, response) = prepared.map_err(bundle_preparation_error)?;
    let ticket = owner.ticket.as_ref().ok_or_else(internal)?;
    let mut store = state.effects.lock().map_err(|_| internal())?;
    read.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    if !store.source_stage_preparation_active(ticket, Instant::now())? {
        return Err(bundle_preparation_error(WorkError::Interrupted(
            Interruption::CancelRequested,
        )));
    }
    let local = owner.local.take().ok_or_else(internal)?;
    store.retain_staged_source_reserved(
        ticket,
        Instant::now(),
        StageRequestIdentity { key, wire_path, raw_query, request },
        &owner.nonce,
        local,
        &reply,
    )?;
    Ok(response)
}

/// Capture only an owned committed descriptor under the project read lease, outside Store I/O.
fn stream_response(
    state: &State,
    operation_id: &str,
    ordinal: Option<usize>,
    control: &mut ReadControl<'_>,
) -> Result<Response<Full<Bytes>>> {
    let _io = source::project_read(state)?;
    let descriptor = {
        let store = state.effects.lock().map_err(|_| internal())?;
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).map_err(query_error)?;
        store.staged_export_descriptor(operation_id, Instant::now())?
    };
    let captured =
        crate::workspace::source_stream_reads::capture(&state.root, &descriptor, control)
            .map_err(query_error)?;
    let (value, schema) = if let Some(ordinal) = ordinal {
        (captured.part(operation_id, ordinal, control).map_err(query_error)?, "SourceStreamChunk")
    } else {
        (captured.manifest(operation_id).map_err(query_error)?, "SourceStreamManifest")
    };
    let response = json_response_for(ApiMajor::V2, value, schema)?;
    control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged).map_err(query_error)?;
    Ok(response)
}

/// Own the accepted operation result allowance through preparation, unwind and terminal transfer.
struct ExportOwner {
    /// Current authenticated session; the original operation ID is not replaced.
    state: Arc<State>,
    /// Original accepted background lookup.
    id: String,
    /// Actual local Store/plan holders whose allowance is released only after drop.
    local: Option<Store>,
    /// Complete accepted public Value still physically owned by this worker.
    accepted: Option<Reply>,
    /// Original admitted preparation body held until no off-lock consumer remains.
    request: Option<Value>,
    /// Complete terminal transfer has already resolved this worker reservation.
    finished: bool,
}

impl Drop for ExportOwner {
    /// An unwinding/declined worker drops payload first, installs safe failure, then releases its claim.
    fn drop(&mut self) {
        drop(self.local.take());
        drop(self.accepted.take());
        drop(self.request.take());
        if let Ok(mut store) = self.state.effects.lock() {
            if !self.finished {
                let _ = store.finish(&self.id, Err(internal()), false);
            }
            store.release_operation_worker(&self.id);
        }
    }
}

/// Accept exactly one original stream-export reply and claim its complete worker allowance.
fn export_response(
    state: &Arc<State>,
    key: &str,
    wire_path: &str,
    raw_query: &str,
    request: &Value,
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let mut read = ReadControl { state, deadline, interruption: None };
    let (id, reply, bytes, permit, local) = {
        let mut store = state.effects.lock().map_err(|_| internal())?;
        read.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
            .map_err(bundle_preparation_error)?;
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            let response = reply_response(&reply)?;
            read.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
                .map_err(bundle_preparation_error)?;
            return Ok(response);
        }
        let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "Another operation is running. Retry shortly.", true)
        })?;
        let (reply, bytes) = store.accept_source_export(key, wire_path, raw_query, request)?;
        let id = reply.value["operation_id"].as_str().ok_or_else(internal)?.to_owned();
        let local = match store.operation_preparation_store(&id) {
            Ok(local) => local,
            Err(error) => {
                let _ = store.finish(&id, Err(internal()), false);
                return Err(error);
            }
        };
        (id, reply, bytes, permit, local)
    };
    let owner = ExportOwner {
        state: Arc::clone(state),
        id,
        local: Some(local),
        accepted: Some(reply),
        request: Some(request.clone()),
        finished: false,
    };
    let shared = Arc::clone(state);
    read.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
        .map_err(bundle_preparation_error)?;
    tokio::task::spawn_blocking(move || {
        // Move the complete Drop owner into the worker, including its terminal bookkeeping.
        let mut owner = owner;
        let _permit = permit;
        let proceed = shared
            .effects
            .lock()
            .ok()
            .and_then(|mut store| store.running(&owner.id).ok())
            .unwrap_or(false);
        if !proceed {
            return;
        }
        let mut control =
            OperationControl { state: &shared, id: &owner.id, deadline, interruption: None };
        let prepared: WorkResult<_> = (|| {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
            let local = owner.local.as_mut().ok_or_else(internal)?;
            let request = owner.request.as_ref().ok_or_else(internal)?;
            let accepted = owner.accepted.as_ref().ok_or_else(internal)?;
            let completed = prepare_export(&shared, local, request, &accepted.value, &mut control)?;
            control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear)?;
            Ok(completed)
        })();
        let prepared = match control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear) {
            Ok(()) => prepared,
            Err(error) => {
                drop(prepared);
                Err(error)
            }
        };
        let cancelled = control.interruption().is_some()
            || shared.stopped.load(Ordering::Acquire)
            || budget_expired(deadline, Instant::now());
        if let Ok(mut store) = shared.effects.lock() {
            // Final state is checked again after Store-lock contention without a recursive callback.
            let cancelled = cancelled
                || shared.stopped.load(Ordering::Acquire)
                || budget_expired(deadline, Instant::now());
            let result = match prepared {
                Ok(reply) if !cancelled => {
                    owner.local.take().ok_or_else(internal).map(|local| (local, reply))
                }
                Ok(reply) => {
                    drop(reply);
                    drop(owner.local.take());
                    Err(bundle_preparation_error(WorkError::Interrupted(
                        control.interruption().unwrap_or(Interruption::DeadlineExceeded),
                    )))
                }
                Err(error) => {
                    drop(owner.local.take());
                    Err(bundle_preparation_error(error))
                }
            };
            if store.finish(&owner.id, result, cancelled).is_ok() {
                owner.finished = true;
            }
        }
    });
    Ok(response(202, "application/json", bytes))
}

/// Prepare the actual private Bundle4 export with no accepted write or downloadable uncommitted file.
fn prepare_export(
    state: &State,
    local: &mut Store,
    request: &Value,
    operation: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<Reply> {
    let _io = source::project_read(state)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let output = request["target_path"].as_str().ok_or_else(Error::invalid)?;
    let snapshot = Snapshot::capture_source_bundle_effect_with_control(
        &state.root,
        None,
        Some(output),
        control,
    )?;
    let plan = crate::workspace::source_bundle_effects::prepare_staged_export(
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
    completed["result"] = json!({"operation_id":operation["operation_id"],"preview":preview,
        "redaction_summary":{"removed_categories":[]}});
    contract::validate_for(ApiMajor::V2, "Operation", &completed)?;
    let reply = Reply { value: completed, schema: "Operation", status: 202 };
    local.charge_reply(&reply)?;
    control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)?;
    Ok(reply)
}

/// Real-adapter controls execute synthetic owned fixtures through transport admission.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::preparation::NoopControl;
    use crate::workspace::source_transfers::{MAX_ARTIFACT_BYTES, PART_BYTES};
    use std::fmt::Write as _;

    /// Own an empty confined project and an ordinary machine session with no durable restore authority.
    fn state(major: ApiMajor, read_only: bool) -> (tempfile::TempDir, Arc<State>) {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("forge.workspace.json"),
            serde_json::to_vec(&json!({"schema_version":"forge.workspace/2","label":"Synthetic staged fixture","resources":[]})).unwrap(),
        ).unwrap();
        std::fs::write(directory.path().join("sentinel.txt"), b"UNRELATED PRIVATE SENTINEL")
            .unwrap();
        let state = State {
            api_major: major,
            root: Root::open(directory.path()).unwrap(),
            project_io: RwLock::new(()),
            restores: None,
            host: "127.0.0.1:1".into(),
            origin: "http://127.0.0.1:1".into(),
            session: Mutex::new(Session::new(Mode::Machine, read_only, None).unwrap()),
            stopped: AtomicBool::new(false),
            rate: Mutex::new((Instant::now(), 0)),
            effects: Mutex::new(Store::default()),
            work: Arc::new(tokio::sync::Semaphore::new(2)),
            jobs: Arc::new(tokio::sync::Semaphore::new(1)),
        };
        (directory, Arc::new(state))
    }

    /// Preserve one absolute future test budget; production establishes it before body admission.
    fn deadline() -> Option<Instant> {
        Instant::now().checked_add(Duration::from_secs(10))
    }

    /// Decode the actual bounded adapter response rather than inspecting a fabricated return DTO.
    fn body(response: Response<Full<Bytes>>) -> Value {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let bytes = runtime.block_on(response.into_body().collect()).unwrap().to_bytes();
        contract::parse(&bytes, MAX_BODY, MAX_BODY).unwrap()
    }

    /// Author valid empty Bundle4 bytes from the actual captured empty index, with two exact transport parts.
    fn artifact(state: &State) -> Vec<u8> {
        let snapshot = Snapshot::capture_for_api(&state.root, ApiMajor::V2).unwrap();
        let mut raw =
            crate::workspace::staged_source_bundles::encode(&snapshot, &mut NoopControl).unwrap();
        raw.extend(std::iter::repeat_n(b' ', PART_BYTES));
        assert!(raw.len() > PART_BYTES && raw.len() < PART_BYTES * 2);
        raw
    }

    /// Use exact original bytes for the acknowledged closed transport declaration.
    fn declaration(raw: &[u8]) -> Value {
        json!({"schema_version":"forge.workspace-index-bundle/4","profile":"index-and-source-hex-staged",
            "artifact_sha256":crate::hashing::sha256_hex(raw),"artifact_size_bytes":raw.len(),
            "chunk_size_bytes":PART_BYTES,"chunk_count":raw.len().div_ceil(PART_BYTES),
            "acknowledge_sensitive_metadata":true,"acknowledge_source_content":true})
    }

    /// Encode exact test bytes independently of the production transport encoder.
    fn hex(raw: &[u8]) -> String {
        raw.iter().fold(String::with_capacity(raw.len() * 2), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
    }

    /// Build canonical exact part bytes; these synthetic facts grant no native restoration authority.
    fn part(raw: &[u8]) -> Value {
        let hex = hex(raw);
        json!({"sha256":crate::hashing::sha256_hex(raw),"size_bytes":raw.len(),"hex":hex})
    }

    /// Create and upload through the actual new adapters, retaining original stage identity and expiry.
    fn ready(state: &Arc<State>, raw: &[u8]) -> (String, Value) {
        let response = dispatch(
            state,
            "POST",
            "/api/v2/project/source-transfer-stages",
            "",
            Some("staged-http-create-key"),
            Some(&declaration(raw)),
            deadline(),
        )
        .unwrap();
        assert_eq!(response.status(), 201);
        let original = body(response);
        let id = original["stage_id"].as_str().unwrap().to_owned();
        for (ordinal, chunk) in raw.chunks(PART_BYTES).enumerate() {
            let path = format!("/api/v2/project/source-transfer-stages/{id}/chunks/{ordinal}");
            let response =
                dispatch(state, "PUT", &path, "", None, Some(&part(chunk)), deadline()).unwrap();
            assert_eq!(response.status(), 200);
            let ack = body(response);
            assert_eq!(ack["expires_at"], original["expires_at"]);
            assert_eq!(ack["chunk_ordinal"], ordinal);
        }
        (id, original)
    }

    /// Supply every native replacement acknowledgment without automatically accepting a restore.
    fn preview_request() -> Value {
        json!({"target_index_schema_version":2,"acknowledge_index_replacement":true,
            "acknowledge_source_content":true,"acknowledge_replace_files":true})
    }

    /// Every exact new route is literal/public; malformed or privately translated paths are refused.
    #[test]
    fn exact_eight_routes_and_closed_path_values() {
        let stage = "bst_0123456789abcdef";
        let operation = "op_0123456789abcdef";
        for (method, path) in [
            ("POST", "/api/v2/project/source-transfer-stages".into()),
            ("PUT", format!("/api/v2/project/source-transfer-stages/{stage}/chunks/319")),
            ("GET", format!("/api/v2/project/source-transfer-stages/{stage}")),
            ("DELETE", format!("/api/v2/project/source-transfer-stages/{stage}")),
            ("POST", format!("/api/v2/project/source-transfer-stages/{stage}/preview")),
            ("POST", "/api/v2/project/source-stream-exports".into()),
            ("GET", format!("/api/v2/project/source-stream-exports/{operation}/manifest")),
            ("GET", format!("/api/v2/project/source-stream-exports/{operation}/chunks/0")),
        ] {
            assert!(handles(method, &path));
            assert!(route(method, &path).is_ok());
        }
        for path in [
            "/api/v1/project/source-transfer-stages",
            "/api/v2/project/source-transfer-stages/",
            "/api/v2/project/source-transfer-stages/bst_0123456789abcdef/preview/",
            "/api/v2/project/source-transfer-stages/bst_0123456789abcdef/chunks/320",
            "/api/v2/project/source-transfer-stages/bst_%30abcdef012345/chunks/0",
        ] {
            assert!(route("PUT", path).is_err());
        }
        assert!(
            route("GET", "/api/v2/project/source-stream-exports/prev_0123456789abcdef/manifest")
                .is_err()
        );
        assert!(matches!(
            route("PUT", "/api/v2/project/source-transfer-stages/bst_0123456789abcdef/chunks/00"),
            Ok(Route::Put { ordinal: 0, .. })
        ));
    }

    /// Closed query/body/header/version admission happens before retained state or private project I/O.
    #[test]
    fn direct_admission_refuses_foreign_scope_without_retention() {
        let (directory, selected) = state(ApiMajor::V2, false);
        let original_index = std::fs::read(directory.path().join("forge.workspace.json")).unwrap();
        let raw = artifact(&selected);
        let request = declaration(&raw);
        assert!(
            dispatch(
                &selected,
                "POST",
                "/api/v2/project/source-transfer-stages",
                "unexpected=1",
                Some("staged-http-create-key"),
                Some(&request),
                deadline()
            )
            .is_err()
        );
        let mut extra = request.clone();
        extra["private"] = json!(true);
        assert!(
            dispatch(
                &selected,
                "POST",
                "/api/v2/project/source-transfer-stages",
                "",
                Some("staged-http-create-key"),
                Some(&extra),
                deadline()
            )
            .is_err()
        );
        assert!(
            dispatch(
                &selected,
                "GET",
                "/api/v2/project/source-transfer-stages/bst_0123456789abcdef",
                "",
                Some("staged-http-create-key"),
                None,
                deadline()
            )
            .is_err()
        );
        let (_other, foreign) = state(ApiMajor::V1, false);
        assert!(
            dispatch(
                &foreign,
                "POST",
                "/api/v2/project/source-transfer-stages",
                "",
                Some("staged-http-create-key"),
                Some(&request),
                deadline()
            )
            .is_err()
        );
        assert_eq!(selected.effects.lock().unwrap().retained_entity_count().unwrap(), 0);
        assert_eq!(
            std::fs::read(directory.path().join("sentinel.txt")).unwrap(),
            b"UNRELATED PRIVATE SENTINEL"
        );
        assert_eq!(
            std::fs::read(directory.path().join("forge.workspace.json")).unwrap(),
            original_index
        );
    }

    /// Actual create/PUT/status/discard replies preserve all transport facts without project mutation.
    #[test]
    fn actual_transport_adapters_preserve_original_bytes_and_no_native_authority() {
        let (directory, selected) = state(ApiMajor::V2, false);
        let original_index = std::fs::read(directory.path().join("forge.workspace.json")).unwrap();
        let raw = artifact(&selected);
        let (id, original) = ready(&selected, &raw);
        let path = format!("/api/v2/project/source-transfer-stages/{id}");
        let status = body(dispatch(&selected, "GET", &path, "", None, None, deadline()).unwrap());
        assert_eq!(status["state"], "ready");
        assert_eq!(status["received_bytes"], raw.len());
        assert_eq!(status["received_chunk_count"], raw.len().div_ceil(PART_BYTES));
        assert_eq!(status["expires_at"], original["expires_at"]);
        assert_eq!(status["artifact_sha256"], crate::hashing::sha256_hex(&raw));
        let discarded = body(
            dispatch(&selected, "DELETE", &path, "", None, Some(&json!({})), deadline()).unwrap(),
        );
        assert_eq!(discarded, json!({"stage_id":id,"discarded":true}));
        let status = body(dispatch(&selected, "GET", &path, "", None, None, deadline()).unwrap());
        assert_eq!(status["state"], "discarded");
        assert_eq!(status["received_bytes"], raw.len());
        assert_eq!(
            std::fs::read(directory.path().join("forge.workspace.json")).unwrap(),
            original_index
        );
        assert_eq!(
            std::fs::read(directory.path().join("sentinel.txt")).unwrap(),
            b"UNRELATED PRIVATE SENTINEL"
        );
    }

    /// Identical create replay returns its exact original JSON/lifetime; changed body fails without new stage.
    #[test]
    fn actual_create_replay_does_not_renew_or_replace_the_original() {
        let (_directory, selected) = state(ApiMajor::V2, false);
        let raw = artifact(&selected);
        let request = declaration(&raw);
        let path = "/api/v2/project/source-transfer-stages";
        let first = body(
            dispatch(
                &selected,
                "POST",
                path,
                "",
                Some("staged-http-create-key"),
                Some(&request),
                deadline(),
            )
            .unwrap(),
        );
        let count = selected.effects.lock().unwrap().retained_entity_count().unwrap();
        let second = body(
            dispatch(
                &selected,
                "POST",
                path,
                "",
                Some("staged-http-create-key"),
                Some(&request),
                deadline(),
            )
            .unwrap(),
        );
        assert_eq!(first, second);
        let mut changed = request;
        changed["artifact_sha256"] = json!("a".repeat(64));
        assert_eq!(
            dispatch(
                &selected,
                "POST",
                path,
                "",
                Some("staged-http-create-key"),
                Some(&changed),
                deadline()
            )
            .unwrap_err()
            .code,
            "idempotency-key-conflict"
        );
        assert_eq!(selected.effects.lock().unwrap().retained_entity_count().unwrap(), count);
    }

    /// Equality/absence/shutdown and read-only mutation gates abstain before stage creation.
    #[test]
    fn typed_stop_and_read_only_precede_creation() {
        let (_directory, selected) = state(ApiMajor::V2, false);
        let raw = artifact(&selected);
        let request = declaration(&raw);
        for end in [None, Some(Instant::now())] {
            assert_eq!(
                dispatch(
                    &selected,
                    "POST",
                    "/api/v2/project/source-transfer-stages",
                    "",
                    Some("staged-http-create-key"),
                    Some(&request),
                    end
                )
                .unwrap_err()
                .code,
                "bundle-preparation-budget-exceeded"
            );
        }
        selected.stopped.store(true, Ordering::Release);
        assert_eq!(
            dispatch(
                &selected,
                "POST",
                "/api/v2/project/source-transfer-stages",
                "",
                Some("staged-http-create-key"),
                Some(&request),
                deadline()
            )
            .unwrap_err()
            .code,
            "shutdown-in-progress"
        );
        let (_directory, readonly) = state(ApiMajor::V2, true);
        assert_eq!(
            dispatch(
                &readonly,
                "POST",
                "/api/v2/project/source-transfer-stages",
                "",
                Some("staged-http-create-key"),
                Some(&request),
                deadline()
            )
            .unwrap_err()
            .code,
            "read-only-session"
        );
        assert_eq!(selected.effects.lock().unwrap().retained_entity_count().unwrap(), 0);
        assert_eq!(readonly.effects.lock().unwrap().retained_entity_count().unwrap(), 0);
    }

    /// Explicit discard remains a sticky typed stop while the genuine raw lease is still owned.
    #[test]
    fn stage_control_preserves_retirement_and_held_raw_charge() {
        let (_directory, selected) = state(ApiMajor::V2, false);
        let raw = artifact(&selected);
        let (id, _) = ready(&selected, &raw);
        let lease =
            selected.effects.lock().unwrap().lease_source_stage(&id, Instant::now()).unwrap();
        let mut control = StageControl {
            read: ReadControl { state: &selected, deadline: deadline(), interruption: None },
            ticket: lease.ticket(),
        };
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear).unwrap();
        selected.effects.lock().unwrap().discard_source_stage(&id, Instant::now()).unwrap();
        let metadata_only_headroom = crate::workspace::source_transfers::OtherPoolUsage {
            bytes: 20 * 1024 * 1024 - 64 * 1024,
            records: 0,
        };
        assert!(
            selected
                .effects
                .lock()
                .unwrap()
                .admit_shared_retention(metadata_only_headroom)
                .is_err()
        );
        assert!(matches!(
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(matches!(
            control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
        let ticket = lease.into_ticket();
        assert_eq!(
            selected
                .effects
                .lock()
                .unwrap()
                .abandon_source_stage(&ticket, Instant::now())
                .unwrap_err()
                .code,
            "version-conflict"
        );
        selected.effects.lock().unwrap().sweep_source_stages(Instant::now()).unwrap();
        assert!(
            selected.effects.lock().unwrap().admit_shared_retention(metadata_only_headroom).is_ok()
        );
        assert_eq!(
            selected.effects.lock().unwrap().source_stage(&id, Instant::now()).unwrap()["state"],
            "discarded"
        );
    }

    /// Unwind drops actual raw/local holders before exact ticket/allowance release and enables same-stage retry.
    #[test]
    fn owned_preview_unwind_does_not_leave_or_credit_live_holders() {
        let (_directory, selected) = state(ApiMajor::V2, false);
        let raw = artifact(&selected);
        let (id, _) = ready(&selected, &raw);
        let key = "staged-http-preview-key";
        let path = format!("/api/v2/project/source-transfer-stages/{id}/preview");
        let request = preview_request();
        let before = selected.effects.lock().unwrap().non_stage_retention().unwrap();
        let (nonce, local, lease) = {
            let mut store = selected.effects.lock().unwrap();
            let nonce = store.reserve(key, "POST", &path, "", &request).unwrap();
            let local = store.preparation_store(key, &nonce).unwrap();
            let lease = store.lease_source_stage(&id, Instant::now()).unwrap();
            (nonce, local, lease)
        };
        let owner = PreviewOwner {
            state: &selected,
            key,
            nonce,
            local: Some(local),
            lease: Some(lease),
            ticket: None,
        };
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _owner = owner;
            panic!("synthetic owned-worker fault");
        }));
        assert!(panicked.is_err());
        let mut store = selected.effects.lock().unwrap();
        let after = store.non_stage_retention().unwrap();
        assert_eq!((after.bytes, after.records), (before.bytes, before.records));
        assert!(store.replay(key, "POST", &path, "", &request).unwrap().is_none());
        let lease = store.lease_source_stage(&id, Instant::now()).unwrap();
        drop(store);
        let ticket = lease.into_ticket();
        selected.effects.lock().unwrap().abandon_source_stage(&ticket, Instant::now()).unwrap();
    }

    /// Native preparation cannot be substituted by mere complete transport on an unqualified session.
    #[test]
    fn ready_transport_without_durable_authority_is_unavailable_and_unmodified() {
        let (directory, selected) = state(ApiMajor::V2, false);
        let original_index = std::fs::read(directory.path().join("forge.workspace.json")).unwrap();
        let raw = artifact(&selected);
        let (id, _) = ready(&selected, &raw);
        let path = format!("/api/v2/project/source-transfer-stages/{id}/preview");
        assert_eq!(
            dispatch(
                &selected,
                "POST",
                &path,
                "",
                Some("staged-http-preview-key"),
                Some(&preview_request()),
                deadline()
            )
            .unwrap_err()
            .code,
            "bundle-restore-unavailable"
        );
        assert_eq!(
            selected.effects.lock().unwrap().source_stage(&id, Instant::now()).unwrap()["state"],
            "ready"
        );
        assert_eq!(
            std::fs::read(directory.path().join("forge.workspace.json")).unwrap(),
            original_index
        );
        assert_eq!(
            std::fs::read(directory.path().join("sentinel.txt")).unwrap(),
            b"UNRELATED PRIVATE SENTINEL"
        );
    }

    /// A real local plan retains its exact ready replay after raw reclamation, while a new prepared-stage lease refuses.
    #[cfg(unix)]
    #[test]
    fn prepared_stage_lost_reply_replays_original_wrapper_without_repreparing() {
        let (directory, selected) = state(ApiMajor::V2, false);
        let original_index = std::fs::read(directory.path().join("forge.workspace.json")).unwrap();
        let raw = artifact(&selected);
        let (id, _) = ready(&selected, &raw);
        let key = "staged-http-preview-key";
        let path = format!("/api/v2/project/source-transfer-stages/{id}/preview");
        let request = preview_request();
        let (nonce, mut local, lease) = {
            let mut store = selected.effects.lock().unwrap();
            let nonce = store.reserve(key, "POST", &path, "", &request).unwrap();
            let local = store.preparation_store(key, &nonce).unwrap();
            let lease = store.lease_source_stage(&id, Instant::now()).unwrap();
            (nonce, local, lease)
        };
        let decoded = lease.decode(&mut NoopControl).unwrap();
        let snapshot = Snapshot::capture_source_bundle_effect_with_control(
            &selected.root,
            Some(&decoded.index),
            None,
            &mut NoopControl,
        )
        .unwrap();
        let plan = crate::workspace::source_bundle_effects::prepare_staged_import(
            &selected.root,
            snapshot,
            decoded,
            &request,
            &mut NoopControl,
        )
        .unwrap();
        let (preview, replacement) = local.preview_source_restore(plan).unwrap();
        let reply = Reply {
            value: json!({"validation":crate::workspace::services::validation(true,None),"preview":preview,"replacement":replacement}),
            schema: "ProjectSourceBundleImportPreview",
            status: 200,
        };
        let expected = body(reply_response(&reply).unwrap());
        local.charge_reply(&reply).unwrap();
        let ticket = lease.into_ticket();
        selected
            .effects
            .lock()
            .unwrap()
            .retain_staged_source_reserved(
                &ticket,
                Instant::now(),
                StageRequestIdentity { key, wire_path: &path, raw_query: "", request: &request },
                &nonce,
                local,
                &reply,
            )
            .unwrap();
        assert_eq!(
            selected.effects.lock().unwrap().source_stage(&id, Instant::now()).unwrap()["state"],
            "prepared"
        );
        let replayed_wrapper = body(
            dispatch(&selected, "POST", &path, "", Some(key), Some(&request), deadline()).unwrap(),
        );
        assert_eq!(replayed_wrapper, expected);
        assert!(selected.effects.lock().unwrap().lease_source_stage(&id, Instant::now()).is_err());
        let mut changed = request.clone();
        changed["target_index_schema_version"] = json!(1);
        assert_eq!(
            dispatch(&selected, "POST", &path, "", Some(key), Some(&changed), deadline())
                .unwrap_err()
                .code,
            "idempotency-key-conflict"
        );
        assert_eq!(
            dispatch(
                &selected,
                "POST",
                &path,
                "",
                Some("staged-http-new-preview-key"),
                Some(&request),
                deadline()
            )
            .unwrap_err()
            .code,
            "bundle-restore-unavailable"
        );
        assert_eq!(
            std::fs::read(directory.path().join("forge.workspace.json")).unwrap(),
            original_index
        );
        assert_eq!(
            std::fs::read(directory.path().join("sentinel.txt")).unwrap(),
            b"UNRELATED PRIVATE SENTINEL"
        );
    }

    /// An accepted export has no download authority until explicit same-session commitment of its real preview.
    #[test]
    fn actual_export_then_commit_manifest_and_part_preserve_family_and_full_generation() {
        let (directory, selected) = state(ApiMajor::V2, false);
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        let request = json!({"target_path":"staged-output.json","acknowledge_sensitive_metadata":true,"acknowledge_source_content":true});
        let accepted = runtime.block_on(async {
            dispatch(
                &selected,
                "POST",
                "/api/v2/project/source-stream-exports",
                "",
                Some("staged-http-export-key"),
                Some(&request),
                deadline(),
            )
            .unwrap()
        });
        assert_eq!(accepted.status(), 202);
        let accepted = body(accepted);
        let id = accepted["operation_id"].as_str().unwrap();
        let operation = runtime.block_on(async {
            let end = Instant::now() + Duration::from_secs(10);
            loop {
                let value = selected.effects.lock().unwrap().operation(id).unwrap();
                if !matches!(value["state"].as_str(), Some("pending" | "running")) {
                    break value;
                }
                assert!(Instant::now() < end);
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        assert_eq!(operation["state"], "succeeded", "terminal error: {}", operation["error"]);
        assert!(!directory.path().join("staged-output.json").exists());
        let manifest_path = format!("/api/v2/project/source-stream-exports/{id}/manifest");
        assert!(dispatch(&selected, "GET", &manifest_path, "", None, None, deadline()).is_err());
        let preview = &operation["result"]["preview"];
        selected
            .effects
            .lock()
            .unwrap()
            .commit_for_api(
                &selected.root,
                &json!({"receipt":preview["receipt"]["token"],
            "observed_version":preview["target_version"]}),
                &selected.stopped,
                ApiMajor::V2,
            )
            .unwrap();
        let raw = std::fs::read(directory.path().join("staged-output.json")).unwrap();
        assert!(raw.len() <= MAX_ARTIFACT_BYTES);
        let manifest =
            body(dispatch(&selected, "GET", &manifest_path, "", None, None, deadline()).unwrap());
        assert_eq!(manifest["artifact_sha256"], crate::hashing::sha256_hex(&raw));
        assert_eq!(manifest["artifact_size_bytes"], raw.len());
        let chunk_path = format!("/api/v2/project/source-stream-exports/{id}/chunks/0");
        let value =
            body(dispatch(&selected, "GET", &chunk_path, "", None, None, deadline()).unwrap());
        assert_eq!(value["artifact_sha256"], manifest["artifact_sha256"]);
        assert_eq!(value["size_bytes"], raw.len().min(PART_BYTES));
        assert_eq!(value["hex"], hex(&raw[..raw.len().min(PART_BYTES)]));
        std::fs::write(directory.path().join("staged-output.json"), b"FOREIGN GENERATION").unwrap();
        assert_eq!(
            dispatch(&selected, "GET", &manifest_path, "", None, None, deadline())
                .unwrap_err()
                .code,
            "version-conflict"
        );
        assert_eq!(
            std::fs::read(directory.path().join("sentinel.txt")).unwrap(),
            b"UNRELATED PRIVATE SENTINEL"
        );
    }
}

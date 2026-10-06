//! Bounded HTTP/1 transport. No project content is served outside the API.

use std::convert::Infallible;
use std::io::Write as _;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use http_body_util::{BodyExt as _, Full, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use serde_json::{Value, json};

use super::contract::{self, ApiMajor, Error, Result};
use super::preparation::{Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};
use super::root::Root;
use super::services::{Snapshot, filtered, paginate};
use super::session::{Mode, Session};

#[path = "http_source.rs"]
mod source;

#[path = "http_staged.rs"]
mod staged;

const MAX_BODY: usize = 1024 * 1024;
const MAX_CONNECTIONS: usize = 16;
const READ_QUERY_BUDGET: Duration = Duration::from_secs(10);

/// Session-owned transport, immutable API namespace and bounded preparation stores.
struct State {
    /// One launch-selected namespace, never inferred from project files or callers.
    api_major: ApiMajor,
    root: Root,
    /// Participating runtime reads and commits share one project-generation lease.
    project_io: RwLock<()>,
    /// Qualified private journal authority; unsupported ports offer no restore.
    restores: Option<super::root::TransactionState>,
    host: String,
    origin: String,
    session: Mutex<Session>,
    stopped: AtomicBool,
    rate: Mutex<(Instant, u32)>,
    effects: Mutex<super::effects::Store>,
    work: Arc<tokio::sync::Semaphore>,
    jobs: Arc<tokio::sync::Semaphore>,
}

/// Cooperative control for one accepted, session-owned background preparation.
struct OperationControl<'a> {
    /// Session stop flag and operation store; locks never cover producer I/O.
    state: &'a State,
    /// The admitted operation whose cancellation and progress this worker owns.
    id: &'a str,
    /// One checked acceptance deadline, including any wait before worker entry.
    deadline: Option<Instant>,
    /// First observed stop remains sticky through all later checkpoints.
    interruption: Option<Interruption>,
}

/// An unrepresentable deadline fails closed; exact equality exhausts the budget.
fn budget_expired(deadline: Option<Instant>, observed: Instant) -> bool {
    deadline.is_none_or(|deadline| observed >= deadline)
}

impl OperationControl<'_> {
    /// Check a supplied monotonic observation using the same production fence.
    fn checkpoint_at(&mut self, update: ProgressUpdate, observed: Instant) -> WorkResult<()> {
        if let Some(reason) = self.interruption {
            return Err(WorkError::Interrupted(reason));
        }
        let reason = if self.state.stopped.load(Ordering::Acquire) {
            Some(Interruption::Shutdown)
        } else if budget_expired(self.deadline, observed) {
            Some(Interruption::DeadlineExceeded)
        } else {
            let mut store = self.state.effects.lock().map_err(|_| internal())?;
            // Waiting for the store is part of the same budget. Recheck after
            // acquisition before publishing progress or allowing producer I/O.
            if self.state.stopped.load(Ordering::Acquire) {
                Some(Interruption::Shutdown)
            } else if budget_expired(self.deadline, Instant::now()) {
                Some(Interruption::DeadlineExceeded)
            } else if store.observe_progress(self.id, update)? {
                None
            } else {
                Some(Interruption::CancelRequested)
            }
        };
        if let Some(reason) = reason {
            self.interruption = Some(reason);
            return Err(WorkError::Interrupted(reason));
        }
        Ok(())
    }
}

impl WorkControl for OperationControl<'_> {
    /// Check cancellation, shutdown and the original deadline at a real boundary.
    fn checkpoint(&mut self, _stage: Stage, update: ProgressUpdate) -> WorkResult<()> {
        self.checkpoint_at(update, Instant::now())
    }

    /// Return the first observed stop without renewing work or inspecting files.
    fn interruption(&self) -> Option<Interruption> {
        self.interruption
    }
}

/// Bounded synchronous read control; no operation or effect store is consulted.
struct ReadControl<'a> {
    /// Session shutdown is the only externally signalled read interruption.
    state: &'a State,
    /// Admission time includes the wait for the blocking worker.
    deadline: Option<Instant>,
    /// Preserve the first observed reason through later parser/error boundaries.
    interruption: Option<Interruption>,
}

impl ReadControl<'_> {
    /// Check the immutable budget at a supplied monotonic observation.
    fn checkpoint_at(&mut self, observed: Instant) -> WorkResult<()> {
        if let Some(reason) = self.interruption {
            return Err(WorkError::Interrupted(reason));
        }
        let reason = if self.state.stopped.load(Ordering::Acquire) {
            Some(Interruption::Shutdown)
        } else if budget_expired(self.deadline, observed) {
            Some(Interruption::DeadlineExceeded)
        } else {
            None
        };
        if let Some(reason) = reason {
            self.interruption = Some(reason);
            return Err(WorkError::Interrupted(reason));
        }
        Ok(())
    }
}

impl WorkControl for ReadControl<'_> {
    /// Check before/after real work without publishing invented numeric progress.
    fn checkpoint(&mut self, _stage: Stage, _update: ProgressUpdate) -> WorkResult<()> {
        self.checkpoint_at(Instant::now())
    }

    /// Return the same latched stop without inspecting files or renewing work.
    fn interruption(&self) -> Option<Interruption> {
        self.interruption
    }
}

/// Convert a typed query stop at its final boundary, preserving safe failures.
fn query_error(error: WorkError) -> Error {
    match error {
        WorkError::Failed(error) => error,
        WorkError::Interrupted(Interruption::DeadlineExceeded) => Error::new(
            "query-budget-exceeded",
            "The query exceeded its time budget. Narrow the scope and retry.",
            true,
        ),
        WorkError::Interrupted(Interruption::Shutdown) => {
            Error::new("shutdown-in-progress", "The workspace is stopping.", false)
        }
        WorkError::Interrupted(Interruption::CancelRequested) => Error::new(
            "query-interrupted",
            "The query stopped before a complete result was available. Retry the read.",
            true,
        ),
    }
}

/// Preserve direct preparation stops with scope-specific safe retry guidance.
fn bundle_preparation_error(error: WorkError) -> Error {
    match error {
        WorkError::Failed(error) => error,
        WorkError::Interrupted(Interruption::DeadlineExceeded) => Error::new(
            "bundle-preparation-budget-exceeded",
            "The bundle preparation exceeded its time budget. Reduce the inputs and retry.",
            true,
        ),
        WorkError::Interrupted(Interruption::Shutdown) => {
            Error::new("shutdown-in-progress", "The workspace is stopping.", false)
        }
        WorkError::Interrupted(Interruption::CancelRequested) => Error::new(
            "bundle-preparation-interrupted",
            "The bundle preparation stopped before retaining a receipt. Retry the request.",
            true,
        ),
    }
}

/// Own every actual ordinary worker copy until settlement and physical release, including unwind exits.
struct OperationWorkerOwner {
    /// Shared Store used after all held private data have physically dropped.
    state: Arc<State>,
    /// Exact accepted operation identity, not a new request generation.
    operation_id: String,
    /// Claimed local receipt/result pool, taken once by the actual preparation body.
    local: Option<super::effects::Store>,
    /// Optional original pending acceptance Value used by the source-export producer.
    acceptance: Option<super::effects::Reply>,
    /// Actual accepted request copy charged separately for the worker's complete lifetime.
    request: Option<Value>,
    /// True only after Store finish successfully installs a queryable terminal outcome.
    finished: bool,
}
impl Drop for OperationWorkerOwner {
    /// Drop actual result/request/reply holders before fallback settlement and exact worker-release credit.
    fn drop(&mut self) {
        drop(self.local.take());
        drop(self.acceptance.take());
        drop(self.request.take());
        if let Ok(mut store) = self.state.effects.lock() {
            if !self.finished {
                let _ = store.finish(&self.operation_id, Err(internal()), false);
            }
            store.release_operation_worker(&self.operation_id);
        }
    }
}

/// Release one off-lock preparation generation on every ordinary or unwinding exit.
struct ReservationGuard<'a> {
    /// Shared session Store; no producer I/O occurs while its lock is held.
    state: &'a State,
    /// Exact admitted idempotency key.
    key: &'a str,
    /// This worker's unique owner generation, never a newer reservation.
    nonce: String,
    /// Actual claimed local holder before the producer takes it; drop precedes any reservation credit.
    local: Option<super::effects::Store>,
}

impl Drop for ReservationGuard<'_> {
    /// A ready reply is immutable; release only a still-pending matching owner.
    fn drop(&mut self) {
        drop(self.local.take());
        if let Ok(mut store) = self.state.effects.lock() {
            store.release_reservation(self.key, &self.nonce);
        }
    }
}

/// Prepare a complete index replacement off-lock, then retain one owned original receipt.
fn bundle_import_response(
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
    let _io = source::project_read(state)?;
    let (nonce, _permit, reserved_local) = {
        let mut store = state.effects.lock().map_err(|_| internal())?;
        control
            .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
            .map_err(bundle_preparation_error)?;
        if let Some(reply) = store.replay(key, "POST", wire_path, raw_query, request)? {
            let response = json_response_for(state.api_major, reply.value, reply.schema)?;
            control
                .checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)
                .map_err(bundle_preparation_error)?;
            return Ok(response);
        }
        let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "Another operation is running. Retry shortly.", true)
        })?;
        let nonce = store.reserve(key, "POST", wire_path, raw_query, request)?;
        let local = match store.preparation_store(key, &nonce) {
            Ok(local) => local,
            Err(error) => {
                store.release_reservation(key, &nonce);
                return Err(error);
            }
        };
        (nonce, permit, local)
    };
    let mut reservation = ReservationGuard { state, key, nonce, local: Some(reserved_local) };
    let prepared: WorkResult<_> = (|| {
        let mut local = reservation.local.take().ok_or_else(internal)?;
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let incoming = super::bundles::decode_bundle_for_api(&request["bundle"], state.api_major)?;
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let snapshot = Snapshot::capture_bundle_effect_with_control(
            &state.root,
            state.api_major,
            Some(&incoming.index),
            &mut control,
        )?;
        let mut plan =
            super::bundle_effects::prepare_import(&state.root, &snapshot, request, &mut control)?;
        let replacement = plan.replacement.take().ok_or_else(internal)?;
        control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
        let preview = local.preview_bundle(plan)?;
        let reply = super::effects::Reply {
            value: json!({"validation":super::services::validation(true,None),
                "preview":preview,"replacement":replacement}),
            schema: "ProjectBundleImportPreview",
            status: 200,
        };
        let response = json_response_for(state.api_major, reply.value.clone(), reply.schema)?;
        local.charge_reply(&reply)?;
        control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear)?;
        Ok((local, reply, response))
    })();
    let mut store = state.effects.lock().map_err(|_| internal())?;
    // This fence includes Store-lock contention; no fallible response work follows retention.
    let prepared = match control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear) {
        Ok(()) => prepared,
        Err(error) => {
            drop(prepared);
            Err(error)
        }
    };
    match prepared {
        Ok((local, reply, response)) => {
            if let Err(error) = store.retain_reserved(key, &reservation.nonce, local, &reply) {
                store.release_reservation(key, &reservation.nonce);
                return Err(error);
            }
            Ok(response)
        }
        Err(error) => {
            store.release_reservation(key, &reservation.nonce);
            Err(bundle_preparation_error(error))
        }
    }
}

/// The nine versioned read services; identities remain exact registered tokens.
#[derive(Clone, Copy)]
enum ReadQuery<'a> {
    LifecycleRecords,
    LifecycleDetail(&'a str),
    LifecycleHistory(&'a str),
    LifecycleQueue,
    ImpactComparisons,
    ImpactDetail(&'a str),
    ImpactChanges(&'a str),
    ImpactFindings(&'a str),
    ImpactPriorDispositions(&'a str),
}

impl<'a> ReadQuery<'a> {
    /// Resolve only an already-admitted private dispatch path, never an alias.
    fn route(path: &'a str) -> Option<Self> {
        match path {
            "/api/v1/lifecycle/records" => Some(Self::LifecycleRecords),
            "/api/v1/lifecycle/queue" => Some(Self::LifecycleQueue),
            "/api/v1/framework-impact/comparisons" => Some(Self::ImpactComparisons),
            _ => {
                if let Some(tail) = path.strip_prefix("/api/v1/lifecycle/records/") {
                    let (id, suffix) = tail.split_once('/').unwrap_or((tail, ""));
                    match suffix {
                        "" => Some(Self::LifecycleDetail(id)),
                        "history" => Some(Self::LifecycleHistory(id)),
                        _ => None,
                    }
                } else if let Some(tail) =
                    path.strip_prefix("/api/v1/framework-impact/comparisons/")
                {
                    let (id, suffix) = tail.split_once('/').unwrap_or((tail, ""));
                    match suffix {
                        "" => Some(Self::ImpactDetail(id)),
                        "changes" => Some(Self::ImpactChanges(id)),
                        "findings" => Some(Self::ImpactFindings(id)),
                        "prior-dispositions" => Some(Self::ImpactPriorDispositions(id)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Return the exact selected-major schema for this complete result.
    fn schema(self) -> &'static str {
        match self {
            Self::LifecycleRecords => "LifecycleRecordPage",
            Self::LifecycleDetail(_) => "LifecycleRecordDetail",
            Self::LifecycleHistory(_) => "LifecycleHistoryPage",
            Self::LifecycleQueue => "LifecycleQueuePage",
            Self::ImpactComparisons => "FrameworkImpactComparisonPage",
            Self::ImpactDetail(_) => "FrameworkImpactComparisonDetail",
            Self::ImpactChanges(_) => "FrameworkImpactChangePage",
            Self::ImpactFindings(_) => "FrameworkImpactFindingPage",
            Self::ImpactPriorDispositions(_) => "FrameworkImpactPriorDispositionPage",
        }
    }

    /// Admit closed query keys and real calendar dates before any index capture.
    fn validate_query(self, query: &[(String, String)]) -> Result<()> {
        let allowed: &[&str] = match self {
            Self::LifecycleRecords => &["as_of", "owner", "state", "page_size", "cursor"],
            Self::LifecycleDetail(_) => &["as_of"],
            Self::LifecycleHistory(_)
            | Self::ImpactComparisons
            | Self::ImpactPriorDispositions(_) => &["page_size", "cursor"],
            Self::LifecycleQueue => &["as_of", "owner", "page_size", "cursor"],
            Self::ImpactDetail(_) => &[],
            Self::ImpactChanges(_) => &["change_class", "page_size", "cursor"],
            Self::ImpactFindings(_) => &[
                "group",
                "decision_state",
                "policy_source",
                "priority",
                "owner",
                "page_size",
                "cursor",
            ],
        };
        let parsed = super::inspection::Query::new(query, allowed)?;
        parsed.date(matches!(self, Self::LifecycleDetail(_) | Self::LifecycleQueue))?;
        Ok(())
    }

    /// Consume the same immutable capture and control throughout domain work.
    fn run(
        self,
        snapshot: &Snapshot,
        query: &[(String, String)],
        control: &mut dyn WorkControl,
    ) -> WorkResult<Value> {
        match self {
            Self::LifecycleRecords => super::lifecycle::records(snapshot, query, control),
            Self::LifecycleDetail(id) => super::lifecycle::detail(snapshot, id, query, control),
            Self::LifecycleHistory(id) => super::lifecycle::history(snapshot, id, query, control),
            Self::LifecycleQueue => super::lifecycle::queue(snapshot, query, control),
            Self::ImpactComparisons => super::impact::comparisons(snapshot, query, control),
            Self::ImpactDetail(id) => super::impact::detail(snapshot, id, query, control),
            Self::ImpactChanges(id) => super::impact::changes(snapshot, id, query, control),
            Self::ImpactFindings(id) => super::impact::findings(snapshot, id, query, control),
            Self::ImpactPriorDispositions(id) => {
                super::impact::prior_dispositions(snapshot, id, query, control)
            }
        }
    }
}

/// Validate and encode a complete selected-major response inside the same budget.
fn controlled_query_response(
    api_major: ApiMajor,
    value: &Value,
    schema: &str,
    control: &mut dyn WorkControl,
) -> WorkResult<Response<Full<Bytes>>> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let validated = contract::validate_for(api_major, schema, value).map_err(|_| internal());
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    validated?;
    let encoded = contract::encode(value, 4 * 1024 * 1024, false);
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    let bytes = encoded?;
    Ok(response(200, "application/json", bytes))
}

/// Capture once and return no result after an interrupted or incomplete query.
fn read_query_response(
    state: &State,
    route: ReadQuery<'_>,
    query: &[(String, String)],
    deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    route.validate_query(query)?;
    let mut control = ReadControl { state, deadline, interruption: None };
    let result = (|| {
        let _io = source::project_read(state)?;
        let snapshot =
            Snapshot::capture_with_control_for_api(&state.root, state.api_major, &mut control)?;
        let value = route.run(&snapshot, query, &mut control)?;
        controlled_query_response(state.api_major, &value, route.schema(), &mut control)
    })();
    result.map_err(query_error)
}

fn internal() -> Error {
    Error::new("internal-error", "The workspace operation could not be completed.", false)
}
fn unauthorized() -> Error {
    Error::new("unauthorized", "A valid local session request is required.", false)
}

/// Validate selected index compatibility before exposing the loopback session.
pub(super) fn launch(
    project: &Path,
    read_only: bool,
    machine: bool,
    no_open: bool,
    api_major: ApiMajor,
) -> Result<()> {
    let root = Root::open(project)?;
    // Qualify and settle prior confirmed intent before index reads, prompts, credentials or listening.
    // Unix qualification/trust failures never hide potentially unresolved state.
    #[cfg(unix)]
    let restores = Some(super::root::TransactionState::open_qualified(&root)?);
    #[cfg(not(unix))]
    let restores: Option<super::root::TransactionState> = None;
    if let Some(state) = &restores {
        state.recover(&root, api_major == ApiMajor::V2 && !read_only)?;
    }
    let mut effects = super::effects::Store::default();
    if let Some(state) = &restores {
        effects.seed_source_outcomes(state)?;
    }
    // Validate the index and containment before exposing a listener or credentials.
    Snapshot::capture_for_api(&root, api_major)?;
    let mode = if machine { Mode::Machine } else { Mode::Browser };
    let passphrase = if machine { None } else { Some(super::session::prompt()?) };
    let mut session = Session::new(mode, read_only, passphrase)?;
    let capability = if machine { Some(session.machine_capability()?) } else { None };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(2)
        .enable_all()
        .build()
        .map_err(|_| internal())?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.map_err(|_| internal())?;
        let host = listener.local_addr().map_err(|_| internal())?.to_string();
        let origin = format!("http://{host}");
        if let Some(capability) = capability {
            let mut descriptor = json!({"base_url":origin,"api_version":api_major.version(),"session_id":session.id,
                "capability":&*capability,"mode":"machine","read_only":read_only,"pid":std::process::id()});
            if api_major == ApiMajor::V2 { descriptor["api_major"] = json!(2); }
            let mut stdout = std::io::stdout().lock();
            serde_json::to_writer(&mut stdout, &descriptor).map_err(|_| internal())?;
            stdout.write_all(b"\n").and_then(|()| stdout.flush()).map_err(|_| internal())?;
        } else {
            eprintln!("Local workspace: {origin}");
            eprintln!("Stop with Ctrl-C. Local unlock does not authenticate reviewer identity.");
            if !no_open { open_browser(&origin)?; }
        }
        let state = Arc::new(State { api_major, root, project_io:RwLock::new(()), restores, host, origin, session:Mutex::new(session), stopped:AtomicBool::new(false), rate:Mutex::new((Instant::now(),0)),effects:Mutex::new(effects),work:Arc::new(tokio::sync::Semaphore::new(2)),jobs:Arc::new(tokio::sync::Semaphore::new(1)) });
        let stop_state = Arc::clone(&state);
        let signal = tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() { stop_state.stopped.store(true, Ordering::Release); }
        });
        let permits = Arc::new(tokio::sync::Semaphore::new(MAX_CONNECTIONS));
        let mut connections = tokio::task::JoinSet::new();
        while !state.stopped.load(Ordering::Acquire) {
            while connections.try_join_next().is_some() {}
            let accepted = tokio::time::timeout(Duration::from_millis(100), listener.accept()).await;
            let (stream, peer) = match accepted {
                Ok(Ok(accepted)) => accepted,
                Ok(Err(_)) => {
                    // An accept that fails immediately must not spin this loop.
                    tokio::time::sleep(Duration::from_millis(75)).await;
                    continue;
                }
                Err(_) => continue,
            };
            if !peer.ip().is_loopback() { continue; }
            let Ok(permit) = Arc::clone(&permits).try_acquire_owned() else { continue; };
            let state = Arc::clone(&state);
            connections.spawn(async move {
                let _permit = permit;
                let service = hyper::service::service_fn(move |request| respond(Arc::clone(&state), request));
                let mut builder = hyper::server::conn::http1::Builder::new();
                builder.keep_alive(false).max_headers(100).max_buf_size(32 * 1024)
                    .timer(hyper_util::rt::TokioTimer::new()).header_read_timeout(Duration::from_secs(5));
                let connection = builder.serve_connection(hyper_util::rt::TokioIo::new(stream), service);
                let _ = tokio::time::timeout(Duration::from_secs(30), connection).await;
            });
        }
        drop(listener);
        signal.abort();
        state.session.lock().map_err(|_| internal())?.stop();
        // Let the confirmed shutdown response finish, then close lingering sockets.
        let _ = tokio::time::timeout(Duration::from_secs(1), async { while connections.join_next().await.is_some() {} }).await;
        connections.abort_all();
        Ok(())
    })
}

fn open_browser(origin: &str) -> Result<()> {
    // This is only the platform browser launcher over a server-generated URL.
    // Project content and user-supplied command fragments never reach a process.
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(origin).spawn();
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", origin])
        .spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(origin).spawn();
    #[cfg(not(any(unix, windows)))]
    return Err(Error::new("invalid-request", "Use --no-open on this platform.", false));
    #[cfg(any(unix, windows))]
    result.map(|_| ()).map_err(|_| {
        Error::new(
            "invalid-request",
            "The browser could not be opened. Restart with --no-open.",
            false,
        )
    })
}

fn single_header<'a>(request: &'a Request<Incoming>, name: &str) -> Result<Option<&'a str>> {
    let mut values = request.headers().get_all(name).iter();
    let first = values.next();
    if values.next().is_some() {
        return Err(unauthorized());
    }
    first.map(|value| value.to_str().map_err(|_| unauthorized())).transpose()
}

/// Enforce transport, selected namespace and scoped capability before dispatch.
fn guard(state: &State, request: &Request<Incoming>) -> Result<()> {
    if state.stopped.load(Ordering::Acquire) {
        return Err(Error::new("shutdown-in-progress", "The workspace is stopping.", false));
    }
    if request.uri().to_string().len() > 2048
        || request.uri().scheme().is_some()
        || request.uri().authority().is_some()
        || single_header(request, "host")? != Some(&state.host)
        || request.headers().iter().any(|(name, value)| {
            name.as_str().starts_with("x-forwarded-")
                || name == "forwarded"
                || name == "via"
                || value.len() > 8192
        })
    {
        return Err(unauthorized());
    }
    let mut rate = state.rate.lock().map_err(|_| internal())?;
    if rate.0.elapsed() >= Duration::from_secs(1) {
        *rate = (Instant::now(), 0);
    }
    rate.1 += 1;
    if rate.1 > 60 {
        return Err(Error::new(
            "invalid-request",
            "Session request rate exceeded. Retry shortly.",
            true,
        ));
    }
    drop(rate);
    if !request.uri().path().starts_with("/api/") {
        return Ok(());
    }
    let origin = single_header(request, "origin")?;
    let site = single_header(request, "sec-fetch-site")?;
    let browser = origin.is_some()
        || site.is_some()
        || request.headers().keys().any(|key| key.as_str().starts_with("sec-fetch-"));
    let writes_body = request.method() != hyper::Method::GET;
    if browser
        && (site != Some("same-origin")
            || !matches!(single_header(request, "sec-fetch-mode")?, Some("cors" | "same-origin"))
            || single_header(request, "sec-fetch-dest")? != Some("empty")
            || origin.is_some_and(|value| value != state.origin)
            || (writes_body && origin != Some(&state.origin)))
    {
        return Err(unauthorized());
    }
    if writes_body {
        // Media-type parameters (`application/json; charset=utf-8`) are valid:
        // compare type/subtype case-insensitively and ignore parameters.
        let json = single_header(request, "content-type")?
            .and_then(|value| value.split(';').next())
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"));
        if !json {
            return Err(Error::new(
                "unsupported-media-type",
                "Use application/json for API requests.",
                false,
            ));
        }
    }
    if request.headers().contains_key("content-encoding")
        || request.headers().contains_key("expect")
    {
        return Err(Error::invalid());
    }
    let private_path = state.api_major.canonical_path(request.uri().path())?;
    if private_path == "/api/v1/session/unlock" && request.method() == hyper::Method::POST {
        if !browser {
            return Err(unauthorized());
        }
    } else {
        let token = single_header(request, "authorization")?
            .and_then(|header| header.strip_prefix("Bearer "))
            .ok_or_else(unauthorized)?;
        let mutation = is_mutation(request.method().as_str(), &private_path);
        state.session.lock().map_err(|_| internal())?.authorize(token, browser, mutation)?;
    }
    Ok(())
}

/// Build a selected-major Session before minting a browser capability.
fn unlock_response(
    state: &State,
    query: &[(String, String)],
    idempotency: Option<&str>,
    bytes: &[u8],
) -> Result<Response<Full<Bytes>>> {
    let mut payload = contract::parse(bytes, 4096, 512).ok();
    let valid = contract::operation_request_for(
        state.api_major,
        "POST",
        &format!("{}/session/unlock", state.api_major.prefix()),
        query,
        idempotency,
        payload.as_ref(),
    )
    .is_ok();
    let passphrase = zeroize::Zeroizing::new(
        match payload
            .as_mut()
            .and_then(Value::as_object_mut)
            .and_then(|value| value.remove("passphrase"))
        {
            Some(Value::String(secret)) => secret,
            _ => String::new(),
        },
    );
    // Everything that can fail before delivery is built first, so a snapshot or
    // session-view failure can never consume a capability the client never sees.
    let _io = source::project_read(state)?;
    let label = Snapshot::capture_for_api(&state.root, state.api_major)?.index.label;
    let session = session_view(state, &label)?;
    let capability = state.session.lock().map_err(|_| internal())?.unlock(if valid {
        &passphrase
    } else {
        ""
    })?;
    let response = json_response_for(
        state.api_major,
        json!({"capability":&*capability,"session":session}),
        "SessionUnlockResponse",
    );
    if response.is_err() {
        // Response validation/encoding is the only remaining post-mint failure;
        // the token was never delivered, so reclaim its slot.
        if let Ok(mut session) = state.session.lock() {
            session.revoke(&capability);
        }
    }
    response
}

/// Preserve write scope except for documented read-only query operations.
fn is_mutation(method: &str, path: &str) -> bool {
    method != "GET"
        && !(method == "POST" && path == "/api/v1/project/bundle-verifications")
        && !matches!(
            path,
            "/api/v1/session/unlock"
                | "/api/v1/session/shutdown"
                | "/api/v1/validation/runs"
                | "/api/v1/applicability/draft/validation"
                | "/api/v1/mapping/draft/validation"
                | "/api/v1/mapping/checks"
        )
}

/// Classify only transport budgets/body parsing; public operation admission still precedes dispatch.
fn staged_transport_family(api_major: ApiMajor, wire_path: &str) -> bool {
    api_major == ApiMajor::V2
        && (wire_path == "/api/v2/project/source-transfer-stages"
            || wire_path.starts_with("/api/v2/project/source-transfer-stages/")
            || wire_path == "/api/v2/project/source-stream-exports"
            || wire_path.starts_with("/api/v2/project/source-stream-exports/"))
}

/// Bound request I/O and route static or admitted selected-major API work.
async fn respond(
    state: Arc<State>,
    request: Request<Incoming>,
) -> std::result::Result<Response<Full<Bytes>>, Infallible> {
    let result = async {
        guard(&state, &request)?;
        let method = request.method().as_str().to_owned();
        let path = request.uri().path().to_owned();
        let query = request.uri().query().unwrap_or_default().to_owned();
        if !path.starts_with("/api/") {
            return static_response(state.api_major, &method, &path);
        }
        // This immutable admission timestamp precedes body I/O and blocking queue wait.
        let admitted_at = Instant::now();
        let permit = Arc::clone(&state.work).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "The workspace is busy. Retry shortly.", true)
        })?;
        let body_limit = if method == "POST"
            && state.api_major.canonical_path(&path)? == "/api/v1/resources/upload"
        {
            14 * 1024 * 1024
        } else {
            MAX_BODY
        };
        let length = single_header(&request, "content-length")?
            .map(str::parse::<usize>)
            .transpose()
            .map_err(|_| Error::invalid())?;
        if length.is_some_and(|length| length > body_limit) {
            return Err(Error::new("payload-too-large", "The request body is too large.", false));
        }
        let idempotency = single_header(&request, "idempotency-key")?.map(str::to_owned);
        // Direct bundle preparation counts bounded body reading and blocking queue wait.
        let private_path = state.api_major.canonical_path(&path)?;
        let bundle_deadline = if state.api_major == ApiMajor::V2
            && staged_transport_family(state.api_major, &path)
        {
            let budget = if method == "POST" && path == "/api/v2/project/source-stream-exports" {
                Duration::from_secs(30)
            } else {
                READ_QUERY_BUDGET
            };
            Some(admitted_at.checked_add(budget))
        } else if method == "POST"
            && state.api_major == ApiMajor::V2
            && (private_path == "/api/v1/project/source-bundle-exports"
                || private_path.starts_with("/api/v1/project/bundle-restores/")
                    && private_path.ends_with("/commit"))
        {
            Some(Instant::now().checked_add(Duration::from_secs(30)))
        } else if method == "POST"
            && state.api_major == ApiMajor::V2
            && matches!(
                private_path.as_str(),
                "/api/v1/project/bundle-imports" | "/api/v1/project/source-bundle-imports"
            )
        {
            Some(Instant::now().checked_add(READ_QUERY_BUDGET))
        } else {
            None
        };
        let body = tokio::time::timeout(
            Duration::from_secs(5),
            Limited::new(request.into_body(), body_limit).collect(),
        )
        .await
        .map_err(|_| Error::invalid())?
        .map_err(|_| {
            Error::new(
                "payload-too-large",
                "The request body could not be read within its limits.",
                false,
            )
        })?
        .to_bytes();
        let body = zeroize::Zeroizing::new(body.to_vec());
        let read_deadline =
            bundle_deadline.unwrap_or_else(|| Instant::now().checked_add(READ_QUERY_BUDGET));
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            dispatch(&state, &method, &path, &query, idempotency.as_deref(), &body, read_deadline)
        })
        .await
        .map_err(|_| internal())?
    }
    .await;
    Ok(match result {
        Ok(response) => response,
        Err(error) => error_response(&error),
    })
}

/// Validate each public response against its launch-selected normative document.
#[allow(clippy::needless_pass_by_value)] // Release owned response data after serialization.
fn json_response_for(
    api_major: ApiMajor,
    value: Value,
    schema: &str,
) -> Result<Response<Full<Bytes>>> {
    contract::validate_for(api_major, schema, &value).map_err(|_| internal())?;
    let bytes = contract::encode(&value, 4 * 1024 * 1024, false)?;
    Ok(response(200, "application/json", bytes))
}

/// Encode the immutable generic effect reply without acquiring project I/O authority.
fn effect_reply_response(
    api_major: ApiMajor,
    reply: super::effects::Reply,
) -> Result<Response<Full<Bytes>>> {
    let mut response = json_response_for(api_major, reply.value, reply.schema)?;
    *response.status_mut() = hyper::StatusCode::from_u16(reply.status).map_err(|_| internal())?;
    Ok(response)
}

/// Publish exact immutable negotiation metadata without capability material.
fn session_view(state: &State, label: &str) -> Result<Value> {
    let session = state.session.lock().map_err(|_| internal())?;
    Ok(json!({"session_id":session.id,"mode":session.mode,"read_only":session.read_only,
        "api_major":state.api_major.number(),"contract_version":state.api_major.version(),"project_label":label,"launched_at":session.launched_at}))
}

/// Validate the normative operation and dispatch captured queries or explicit effects.
/// Dispatch authenticated requests, preserving admission, replay and terminal fences.
#[allow(clippy::too_many_lines)] // Explicit normative route dispatch; domain rules stay in services.
fn dispatch(
    state: &Arc<State>,
    method: &str,
    path: &str,
    raw_query: &str,
    idempotency: Option<&str>,
    bytes: &[u8],
    read_deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let wire_path = path;
    let private_path = state.api_major.canonical_path(wire_path)?;
    let path = private_path.as_str();
    let json_response = |value, schema| json_response_for(state.api_major, value, schema);
    let staged_route = staged_transport_family(state.api_major, wire_path);
    if method == "GET" && !bytes.is_empty() && !staged_route {
        return Err(Error::invalid());
    }
    let query: Vec<(String, String)> =
        url::form_urlencoded::parse(raw_query.as_bytes()).into_owned().collect();
    if method == "POST" && path == "/api/v1/session/unlock" {
        return unlock_response(state, &query, idempotency, bytes);
    }
    let payload = if bytes.is_empty() {
        None
    } else {
        Some(contract::parse(
            bytes,
            if path == "/api/v1/resources/upload" { 14 * 1024 * 1024 } else { MAX_BODY },
            if path == "/api/v1/resources/upload" { 13_981_016 } else { 64 * 1024 },
        )?)
    };
    match state.api_major {
        ApiMajor::V1 => {
            contract::operation_request(method, wire_path, &query, idempotency, payload.as_ref())?
        }
        ApiMajor::V2 => contract::operation_request_for(
            state.api_major,
            method,
            wire_path,
            &query,
            idempotency,
            payload.as_ref(),
        )?,
    };
    // Reclaim only retired unconfirmed transport after normative request admission.
    // Actual outstanding raw leases remain charged; accepted native authorities are separate.
    state.effects.lock().map_err(|_| internal())?.sweep_source_stages(Instant::now())?;
    if staged_route && staged::handles(method, wire_path) {
        return staged::dispatch(
            state,
            method,
            wire_path,
            raw_query,
            idempotency,
            payload.as_ref(),
            read_deadline,
        );
    }
    if method == "POST" && path == "/api/v1/session/shutdown" {
        state.stopped.store(true, Ordering::Release);
        return json_response(json!({"state":"shutting-down"}), "ShutdownResponse");
    }
    if state.api_major == ApiMajor::V2
        && method == "GET"
        && let Some(route) = ReadQuery::route(path)
    {
        return read_query_response(state, route, &query, read_deadline);
    }
    if state.api_major == ApiMajor::V2
        && method == "POST"
        && path == "/api/v1/project/bundle-imports"
    {
        return bundle_import_response(
            state,
            idempotency.ok_or_else(Error::invalid)?,
            wire_path,
            raw_query,
            payload.as_ref().ok_or_else(Error::invalid)?,
            read_deadline,
        );
    }
    if state.api_major == ApiMajor::V2 {
        if method == "POST" && path == "/api/v1/project/source-bundle-imports" {
            return source::import_response(
                state,
                idempotency.ok_or_else(Error::invalid)?,
                wire_path,
                raw_query,
                payload.as_ref().ok_or_else(Error::invalid)?,
                read_deadline,
            );
        }
        if method == "POST" && path == "/api/v1/project/source-bundle-exports" {
            return source::export_response(
                state,
                idempotency.ok_or_else(Error::invalid)?,
                wire_path,
                raw_query,
                payload.as_ref().ok_or_else(Error::invalid)?,
                read_deadline,
            );
        }
        if let Some(tail) = path.strip_prefix("/api/v1/project/bundle-restores/") {
            if method == "POST"
                && let Some(id) = tail.strip_suffix("/commit")
            {
                return source::commit_response(
                    state,
                    id,
                    idempotency.ok_or_else(Error::invalid)?,
                    wire_path,
                    raw_query,
                    payload.as_ref().ok_or_else(Error::invalid)?,
                    read_deadline,
                );
            }
            if method == "POST"
                && let Some(id) = tail.strip_suffix("/cancel")
            {
                return source::outcome_response(state, id, true);
            }
            if method == "GET" {
                return source::outcome_response(state, tail, false);
            }
        }
    }
    if method == "GET" {
        // Pure status/preview reads remain available during native settlement.
        // Every subsequent download acquires project data before the Store lock.
        let pure = path.starts_with("/api/v1/operations/")
            || path.starts_with("/api/v1/effects/previews/")
            || path.starts_with("/api/v1/conversions/");
        let _io = if pure { None } else { Some(source::project_read(state)?) };
        let store = state.effects.lock().map_err(|_| internal())?;
        if let Some(id) = path.strip_prefix("/api/v1/operations/") {
            return json_response(store.operation(id)?, "Operation");
        }
        if let Some(id) = path.strip_prefix("/api/v1/effects/previews/") {
            return json_response(store.get_preview(id)?, "EffectPreview");
        }
        if let Some(id) = path.strip_prefix("/api/v1/conversions/") {
            return json_response(store.conversion(id)?, "ConversionResult");
        }
        if state.api_major == ApiMajor::V2
            && let Some(id) = path
                .strip_prefix("/api/v1/project/source-bundle-exports/")
                .and_then(|tail| tail.strip_suffix("/download"))
        {
            let bytes = store.download_source(&state.root, id)?;
            let mut response = response(200, "application/json", bytes);
            response.headers_mut().insert(
                "content-disposition",
                hyper::header::HeaderValue::from_static(
                    "attachment; filename=\"forge-workspace-index-and-source-content.json\"",
                ),
            );
            return Ok(response);
        }
        if state.api_major == ApiMajor::V2
            && let Some(id) = path
                .strip_prefix("/api/v1/project/bundle-exports/")
                .and_then(|tail| tail.strip_suffix("/download"))
        {
            let bytes = store.download_metadata(&state.root, id)?;
            let mut response = response(200, "application/json", bytes);
            response.headers_mut().insert(
                "content-disposition",
                hyper::header::HeaderValue::from_static(
                    "attachment; filename=\"forge-workspace-index-and-hashes.json\"",
                ),
            );
            return Ok(response);
        }
        if let Some(id) =
            path.strip_prefix("/api/v1/exports/").and_then(|tail| tail.strip_suffix("/download"))
        {
            let bytes = store.download(&state.root, id)?;
            let mut response = response(200, "text/html; charset=utf-8", bytes);
            response.headers_mut().insert(
                "content-disposition",
                hyper::header::HeaderValue::from_static(
                    "attachment; filename=forge-review-report.html",
                ),
            );
            return Ok(response);
        }
    }
    if method == "POST"
        && path.starts_with("/api/v1/operations/")
        && path.ends_with("/cancellation")
    {
        let id = path
            .strip_prefix("/api/v1/operations/")
            .and_then(|tail| tail.strip_suffix("/cancellation"))
            .ok_or_else(Error::invalid)?;
        return json_response(
            state.effects.lock().map_err(|_| internal())?.cancel(id)?,
            "Operation",
        );
    }
    if let Some(key) = idempotency {
        let request = payload.clone().unwrap_or_else(|| json!({}));
        // Retained replies are pure session observations. Wait only on the short
        // Store owner, release it, then acquire any new project-data authority.
        // This also lets a response-loss retry observe the original commit once
        // its Store-serialized publication has completed.
        {
            let store = state.effects.lock().map_err(|_| internal())?;
            if let Some(reply) = store.replay(key, method, wire_path, raw_query, &request)? {
                return effect_reply_response(state.api_major, reply);
            }
        }
        let write_lease = if path == "/api/v1/effects/commits" {
            Some(source::project_write(state)?)
        } else {
            None
        };
        let _read = if write_lease.is_none() { Some(source::project_read(state)?) } else { None };
        let mut store = state.effects.lock().map_err(|_| internal())?;
        let reply = if let Some(reply) =
            store.replay(key, method, wire_path, raw_query, &request)?
        {
            reply
        } else {
            let kind = match path {
                "/api/v1/conversions" => Some("conversion"),
                "/api/v1/applicability/analyses" => Some("applicability-analysis"),
                "/api/v1/mapping/builds" => Some("mapping-build"),
                "/api/v1/exports" => Some("export"),
                "/api/v1/project/bundle-exports" if state.api_major == ApiMajor::V2 => {
                    Some("export")
                }
                _ => None,
            };
            let reply = if let Some(kind) = kind {
                let permit = Arc::clone(&state.jobs).try_acquire_owned().map_err(|_| {
                    Error::new(
                        "invalid-request",
                        "Another operation is running. Retry shortly.",
                        true,
                    )
                })?;
                let (accepted, _bytes) = store.accept_operation(
                    super::effects::OperationRequestIdentity {
                        key,
                        method,
                        wire_path,
                        raw_query,
                        request: &request,
                    },
                    kind,
                )?;
                let operation = accepted.value.clone();
                let accepted_at = Instant::now();
                let deadline = accepted_at.checked_add(Duration::from_secs(30));
                let id = operation["operation_id"].as_str().ok_or_else(internal)?.to_owned();
                let local = store.operation_preparation_store(&id);
                let shared = Arc::clone(state);
                let method = method.to_owned();
                let path = path.to_owned();
                let request = request.clone();
                match local {
                    Err(error) => {
                        store.finish(&id, Err(error), false)?;
                    }
                    Ok(claimed_local) => {
                        tokio::task::spawn_blocking(move || {
                            let _permit = permit;
                            let mut owner = OperationWorkerOwner {
                                state: Arc::clone(&shared),
                                operation_id: id.clone(),
                                local: Some(claimed_local),
                                acceptance: None,
                                request: Some(request),
                                finished: false,
                            };
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
                                deadline,
                                interruption: None,
                            };
                            let result = (|| {
                                let mut local = owner.local.take().ok_or_else(internal)?;
                                let request = owner.request.as_ref().ok_or_else(internal)?;
                                let _io = source::project_read(&shared)?;
                                let mut snapshot = if shared.api_major == ApiMajor::V2
                                    && path == "/api/v1/project/bundle-exports"
                                {
                                    Snapshot::capture_bundle_effect_with_control(
                                        &shared.root,
                                        shared.api_major,
                                        None,
                                        &mut control,
                                    )?
                                } else {
                                    Snapshot::capture_with_control_for_api(
                                        &shared.root,
                                        shared.api_major,
                                        &mut control,
                                    )?
                                };
                                let reply = match shared.api_major {
                                    ApiMajor::V1 => super::actions::prepare_with_control(
                                        &mut local,
                                        &shared.root,
                                        &mut snapshot,
                                        &method,
                                        &path,
                                        request,
                                        &mut control,
                                    )?,
                                    ApiMajor::V2 => super::actions::prepare_with_control_for_api(
                                        &mut local,
                                        &shared.root,
                                        &mut snapshot,
                                        &method,
                                        (shared.api_major, &path),
                                        request,
                                        &mut control,
                                    )?,
                                };
                                Ok((local, reply))
                            })();
                            // The final checkpoint also overrides a local safe failure
                            // when a stop arrives before any receipts can be transferred.
                            let result = match control
                                .checkpoint(Stage::RetainPrepared, ProgressUpdate::Clear)
                            {
                                Ok(()) => result,
                                Err(error) => {
                                    drop(result);
                                    Err(error)
                                }
                            };
                            if let Ok(mut store) = shared.effects.lock() {
                                let cancelled = control.interruption().is_some()
                                    || shared.stopped.load(Ordering::Acquire)
                                    || budget_expired(deadline, Instant::now());
                                owner.finished = store
                                    .finish(&id, result.map_err(WorkError::into_error), cancelled)
                                    .is_ok();
                            }
                        });
                    }
                }
                accepted
            } else if path == "/api/v1/effects/commits" {
                store.commit_replayed(
                    &state.root,
                    &state.stopped,
                    state.api_major,
                    super::effects::OperationRequestIdentity {
                        key,
                        method,
                        wire_path,
                        raw_query,
                        request: &request,
                    },
                )?
            } else {
                let nonce = store.reserve(key, method, wire_path, raw_query, &request)?;
                let local = match store.preparation_store(key, &nonce) {
                    Ok(local) => local,
                    Err(error) => {
                        store.release_reservation(key, &nonce);
                        return Err(error);
                    }
                };
                // Synchronous local retention stays within the same claimed pool; failure drops it before release.
                let prepared = (|| {
                    let mut local = local;
                    let mut snapshot = Snapshot::capture_for_api(&state.root, state.api_major)?;
                    let reply = match state.api_major {
                        ApiMajor::V1 => super::actions::prepare(
                            &mut local,
                            &state.root,
                            &mut snapshot,
                            method,
                            path,
                            &request,
                        )?,
                        ApiMajor::V2 => super::actions::prepare_for_api(
                            &mut local,
                            &state.root,
                            &mut snapshot,
                            method,
                            path,
                            &request,
                            state.api_major,
                        )?,
                    };
                    Ok((local, reply))
                })();
                match prepared {
                    Ok((local, reply)) => {
                        if let Err(error) = store.retain_reserved(key, &nonce, local, &reply) {
                            store.release_reservation(key, &nonce);
                            return Err(error);
                        }
                        reply
                    }
                    Err(error) => {
                        store.release_reservation(key, &nonce);
                        return Err(error);
                    }
                }
            };
            // Every branch has retained the original replay before publication/dispatch; no fallible post-write remember.
            reply
        };
        return effect_reply_response(state.api_major, reply);
    }
    let _io = source::project_read(state)?;
    let mut snapshot = Snapshot::capture_for_api(&state.root, state.api_major)?;
    match (method, path) {
        ("GET", "/api/v1/project/bundle-preview") => json_response(
            match state.api_major {
                ApiMajor::V1 => super::bundles::preview(&snapshot)?,
                ApiMajor::V2 => super::bundles::preview_for_api(&snapshot, state.api_major)?,
            },
            "ProjectBundlePreview",
        ),
        ("POST", "/api/v1/project/bundle-verifications") => json_response(
            match state.api_major {
                ApiMajor::V1 => super::bundles::verify_registered(
                    &snapshot,
                    payload.as_ref().ok_or_else(Error::invalid)?,
                )?,
                ApiMajor::V2 => super::bundles::verify_registered_for_api(
                    &snapshot,
                    payload.as_ref().ok_or_else(Error::invalid)?,
                    state.api_major,
                )?,
            },
            "ProjectBundleVerification",
        ),
        ("GET", "/api/v1/session") => {
            json_response(session_view(state, &snapshot.index.label)?, "Session")
        }
        ("GET", "/api/v1/project/config-status") => {
            json_response(super::services::config_status(&state.root)?, "ProjectConfigStatus")
        }
        ("GET", "/api/v1/applicability/draft") => {
            json_response(super::domain::draft(&snapshot, false)?, "ApplicabilityDraft")
        }
        ("GET", "/api/v1/mapping/draft") => {
            json_response(super::domain::draft(&snapshot, true)?, "MappingDraft")
        }
        ("GET", "/api/v1/applicability/report") => {
            json_response(snapshot.report_view()?, "ApplicabilityReportView")
        }
        ("POST", "/api/v1/validation/runs") => json_response(
            snapshot.validate_selection(payload.as_ref().ok_or_else(Error::invalid)?)?,
            "ValidationReport",
        ),
        ("POST", "/api/v1/applicability/draft/validation" | "/api/v1/mapping/draft/validation") => {
            let validation = super::domain::validate_draft(
                &mut snapshot,
                path.contains("/mapping/"),
                &payload.as_ref().ok_or_else(Error::invalid)?["manifest"],
            )?;
            json_response(validation, "ValidationReport")
        }
        ("POST", "/api/v1/mapping/checks") => json_response(
            super::services::validation(super::domain::mapping(&snapshot).is_ok(), None),
            "ValidationReport",
        ),
        ("GET", "/api/v1/mapping/subjects") => json_response(
            paginate(
                filtered(
                    super::domain::subjects_for(
                        &snapshot,
                        query
                            .iter()
                            .find(|(key, _)| key == "resource_id")
                            .map(|(_, value)| value.as_str()),
                        query
                            .iter()
                            .find(|(key, _)| key == "side")
                            .map(|(_, value)| value.as_str()),
                    )?,
                    &query,
                    &["side", "resource_id"],
                ),
                &snapshot.version,
                &query,
            )?,
            "SubjectPage",
        ),
        ("GET", "/api/v1/provenance/entries") => {
            let graph = super::provenance::Graph::build(&snapshot)?;
            let anchor = query
                .iter()
                .find(|(key, _)| key == "anchor")
                .map(|(_, value)| value.as_str())
                .ok_or_else(Error::invalid)?;
            json_response(
                paginate(
                    filtered(graph.entries(anchor), &query, &["kind"]),
                    &snapshot.version,
                    &query,
                )?,
                "ProvenanceEntryPage",
            )
        }
        ("GET", path) if path.starts_with("/api/v1/provenance/excerpts/") => json_response(
            super::provenance::Graph::build(&snapshot)?
                .excerpt(&path["/api/v1/provenance/excerpts/".len()..])?,
            "ProvenanceExcerpt",
        ),
        ("GET", "/api/v1/project/summary") => json_response(snapshot.summary(), "ProjectSummary"),
        ("GET", "/api/v1/review-queue/counts") => {
            json_response(snapshot.counts(), "ReviewQueueCounts")
        }
        ("GET", "/api/v1/review-queue/items") => json_response(
            paginate(
                filtered(
                    snapshot.queue(),
                    &query,
                    &["reason_code", "classification", "resource_id"],
                ),
                &snapshot.version,
                &query,
            )?,
            "QueueItemPage",
        ),
        ("GET", "/api/v1/applicability/controls") => json_response(
            paginate(
                filtered(snapshot.controls()?, &query, &["classification", "decision_state"]),
                &snapshot.version,
                &query,
            )?,
            "ControlPage",
        ),
        ("GET", "/api/v1/resources") => {
            let mut items: Vec<_> =
                snapshot.items.iter().map(|item| item.metadata.clone()).collect();
            items.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
            let items = filtered(items, &query, &["role", "validation_state", "stale"]);
            json_response(paginate(items, &snapshot.version, &query)?, "ResourcePage")
        }
        ("GET", path) if path.starts_with("/api/v1/resources/") => {
            let id = &path["/api/v1/resources/".len()..];
            if let Some(id) = id.strip_suffix("/validation") {
                json_response(snapshot.item(id)?.validation.clone(), "ValidationReport")
            } else {
                json_response(snapshot.item(id)?.metadata.clone(), "Resource")
            }
        }
        _ => Err(Error::new("not-found", "The requested API operation was not found.", false)),
    }
}

/// Serve content-addressed assets and immutable selected-major bootstrap metadata.
fn static_response(api_major: ApiMajor, method: &str, path: &str) -> Result<Response<Full<Bytes>>> {
    if method != "GET" {
        return Err(Error::new("not-found", "The requested asset was not found.", false));
    }
    if path == "/" {
        Ok(response(
            200,
            "text/html; charset=utf-8",
            super::assets::shell_for(api_major).into_bytes(),
        ))
    } else if path == super::assets::script_path() {
        Ok(response(
            200,
            "text/javascript; charset=utf-8",
            super::assets::SCRIPT.as_bytes().to_vec(),
        ))
    } else if path == super::assets::style_path() {
        Ok(response(200, "text/css; charset=utf-8", super::assets::STYLE.as_bytes().to_vec()))
    } else {
        Err(Error::new("not-found", "The requested asset was not found.", false))
    }
}

/// Encode a safe typed error, preserving legacy statuses and the inspection stop codes.
fn error_response(error: &Error) -> Response<Full<Bytes>> {
    let status = match error.code {
        "unauthorized" | "unlock-failed" => 401,
        "read-only-session" | "resource-containment" => 403,
        "not-found" => 404,
        "version-conflict"
        | "idempotency-key-conflict"
        | "receipt-reused"
        | "receipt-mismatch"
        | "operation-not-cancellable"
        | "bundle-preparation-in-progress"
        | "bundle-restore-in-progress" => 409,
        "receipt-expired" => 410,
        "payload-too-large" => 413,
        "unsupported-media-type" => 415,
        "validation-failed" => 422,
        "unlock-throttled" => 429,
        "internal-error" => 500,
        "bundle-restore-unavailable"
        | "bundle-restore-recovery-required"
        | "bundle-restore-budget-exceeded"
        | "query-budget-exceeded"
        | "query-interrupted"
        | "bundle-preparation-budget-exceeded"
        | "bundle-preparation-interrupted" => 503,
        _ => 400,
    };
    let bytes = serde_json::to_vec(&error).unwrap_or_else(|_| b"{\"code\":\"internal-error\",\"message\":\"Response unavailable.\",\"retryable\":false}".to_vec());
    response(status, "application/json", bytes)
}

fn response(status: u16, media_type: &str, bytes: Vec<u8>) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from(bytes)));
    *response.status_mut() = hyper::StatusCode::from_u16(status).expect("fixed valid HTTP status");
    let headers = response.headers_mut();
    for (name, value) in [
        ("content-type", media_type),
        ("cache-control", "no-store"),
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("referrer-policy", "no-referrer"),
        ("permissions-policy", "camera=(), microphone=(), geolocation=()"),
        (
            "content-security-policy",
            "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'none'; base-uri 'none'; object-src 'none'; frame-ancestors 'none'; form-action 'none'",
        ),
        ("connection", "close"),
    ] {
        headers.insert(
            hyper::header::HeaderName::from_static(name),
            value.parse().expect("fixed valid header"),
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Preserve explicit v1 negotiation in existing browser capability controls.
    fn browser_state(root: Root, passphrase: &str) -> State {
        State {
            api_major: ApiMajor::V1,
            root,
            project_io: RwLock::new(()),
            restores: None,
            host: "127.0.0.1:1".into(),
            origin: "http://127.0.0.1:1".into(),
            session: Mutex::new(
                Session::new(
                    Mode::Browser,
                    false,
                    Some(zeroize::Zeroizing::new(passphrase.to_owned())),
                )
                .unwrap(),
            ),
            stopped: AtomicBool::new(false),
            rate: Mutex::new((Instant::now(), 0)),
            effects: Mutex::new(super::super::effects::Store::default()),
            work: Arc::new(tokio::sync::Semaphore::new(2)),
            jobs: Arc::new(tokio::sync::Semaphore::new(1)),
        }
    }

    /// Retained identical confirmations remain pure reads while a project lease is held.
    /// Changed keys/bodies cannot acquire a fresh publication authority through replay.
    #[test]
    fn generic_commit_replay_precedes_project_lease_for_both_majors() {
        for major in [ApiMajor::V1, ApiMajor::V2] {
            let dir = tempfile::tempdir().unwrap();
            let mut state =
                browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
            state.api_major = major;
            let state = Arc::new(state);
            let path = format!("/api/v{}/effects/commits", major.number());
            let request = json!({"receipt":"synthetic-immutable-receipt-token", "observed_version":"a".repeat(64),"confirmed":true});
            let operation = state.effects.lock().unwrap().completed("commit",json!({"write_committed":true,"committed_sha256":"b".repeat(64),"target_path":"result.md","new_version":"b".repeat(64)})).unwrap();
            let reply = super::super::effects::Reply {
                value: operation.clone(),
                schema: "Operation",
                status: 202,
            };
            state
                .effects
                .lock()
                .unwrap()
                .remember("synthetic-replay-key", "POST", &path, "", &request, &reply)
                .unwrap();
            let _busy = state.project_io.write().unwrap();
            let response = dispatch(
                &state,
                "POST",
                &path,
                "",
                Some("synthetic-replay-key"),
                &serde_json::to_vec(&request).unwrap(),
                None,
            )
            .unwrap();
            assert_eq!(response.status(), 202);
            let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
            let bytes = runtime.block_on(response.into_body().collect()).unwrap().to_bytes();
            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), operation);
            let mut changed = request.clone();
            changed["observed_version"] = json!("c".repeat(64));
            assert_eq!(
                dispatch(
                    &state,
                    "POST",
                    &path,
                    "",
                    Some("synthetic-replay-key"),
                    &serde_json::to_vec(&changed).unwrap(),
                    None
                )
                .unwrap_err()
                .code,
                "idempotency-key-conflict"
            );
            assert_eq!(
                dispatch(
                    &state,
                    "POST",
                    &path,
                    "",
                    Some("synthetic-fresh-key"),
                    &serde_json::to_vec(&request).unwrap(),
                    None
                )
                .unwrap_err()
                .code,
                "invalid-request"
            );
            assert!(!dir.path().join("result.md").exists());
        }
    }

    /// Both bootstrap majors publish exact immutable metadata and bounded secured asset responses.
    #[test]
    fn selected_bootstrap_metadata_assets_and_rejections() {
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        for major in [ApiMajor::V1, ApiMajor::V2] {
            let shell = static_response(major, "GET", "/").unwrap();
            assert_eq!(shell.status(), 200);
            assert_eq!(shell.headers()["content-type"], "text/html; charset=utf-8");
            assert_eq!(shell.headers()["cache-control"], "no-store");
            assert_eq!(shell.headers()["x-content-type-options"], "nosniff");
            assert_eq!(shell.headers()["x-frame-options"], "DENY");
            assert!(
                shell.headers()["content-security-policy"]
                    .to_str()
                    .unwrap()
                    .contains("default-src 'none'")
            );
            let body = runtime.block_on(shell.into_body().collect()).unwrap().to_bytes();
            let html = std::str::from_utf8(&body).unwrap();
            assert_eq!(html.matches("name=\"forge-api-major\"").count(), 1);
            assert!(
                html.contains(&format!("name=\"forge-api-major\" content=\"{}\"", major.number()))
            );
            assert_eq!(
                html.matches("name=\"forge-api-contract-version\"").count(),
                usize::from(major == ApiMajor::V2)
            );
            if major == ApiMajor::V2 {
                assert!(html.contains("name=\"forge-api-contract-version\" content=\"2.4.0\""));
            }
            assert!(html.contains("id=\"workspace\" hidden"));
            for (asset_path, expected, media) in [
                (
                    super::super::assets::script_path(),
                    super::super::assets::SCRIPT,
                    "text/javascript; charset=utf-8",
                ),
                (
                    super::super::assets::style_path(),
                    super::super::assets::STYLE,
                    "text/css; charset=utf-8",
                ),
            ] {
                assert!(html.contains(&asset_path));
                let response = static_response(major, "GET", &asset_path).unwrap();
                assert_eq!(response.status(), 200);
                assert_eq!(response.headers()["content-type"], media);
                assert_eq!(response.headers()["cache-control"], "no-store");
                let bytes = runtime.block_on(response.into_body().collect()).unwrap().to_bytes();
                assert_eq!(bytes.as_ref(), expected.as_bytes());
                assert_eq!(
                    static_response(major, "POST", &asset_path).unwrap_err().code,
                    "not-found"
                );
            }
            assert_eq!(
                static_response(major, "GET", "/assets/unknown.js").unwrap_err().code,
                "not-found"
            );
            assert_eq!(static_response(major, "POST", "/").unwrap_err().code, "not-found");
        }
    }

    fn capability_count(state: &State) -> usize {
        state.session.lock().unwrap_or_else(std::sync::PoisonError::into_inner).capability_count()
    }

    #[test]
    fn snapshot_failure_never_consumes_a_capability_slot() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("forge.workspace.json"),
            br#"{"schema_version":"forge.workspace/1","label":"Example","resources":[{"key":"missing","role":"policy-source","path":"missing.md"}]}"#,
        )
        .unwrap();
        let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        let body = br#"{"passphrase":"correct long passphrase"}"#;
        assert!(unlock_response(&state, &[], None, body).is_err());
        assert_eq!(capability_count(&state), 0);
        std::fs::write(dir.path().join("missing.md"), "# Missing\n").unwrap();
        assert!(unlock_response(&state, &[], None, body).is_ok());
        assert_eq!(capability_count(&state), 1);
    }

    #[test]
    fn malformed_and_wrong_unlock_requests_share_the_generic_failure() {
        for body in [
            r#"{"passphrase":"incorrect long passphrase"}"#,
            r#"{"passphrase":"short"}"#,
            r#"{"passphrase":null}"#,
            r#"{"passphrase":"correct long passphrase","extra":true}"#,
            "[]",
            "null",
            "not json",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
            let error = unlock_response(&state, &[], None, body.as_bytes())
                .map(|_| ())
                .expect_err("invalid unlock must fail");
            assert_eq!(error.code, "unlock-failed");
            assert_eq!(error.message, "The workspace could not be unlocked.");
            assert_eq!(
                unlock_response(&state, &[], None, b"{}")
                    .map(|_| ())
                    .expect_err("retry must be throttled")
                    .code,
                "unlock-throttled"
            );
        }
    }
    /// Expired read admission prevents capture and remains sticky after later shutdown.
    #[test]
    fn read_query_budget_covers_capture_and_keeps_first_stop() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.workspace.json"), b"invalid index").unwrap();
        let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        let observed = Instant::now();
        let mut control =
            ReadControl { state: &state, deadline: Some(observed), interruption: None };
        assert!(matches!(
            Snapshot::capture_with_control_for_api(&state.root, ApiMajor::V2, &mut control),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        state.stopped.store(true, Ordering::Release);
        assert!(matches!(
            control.checkpoint_at(observed),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        assert_eq!(control.interruption(), Some(Interruption::DeadlineExceeded));
        let mut shutdown = ReadControl { state: &state, deadline: None, interruption: None };
        assert!(matches!(
            shutdown.checkpoint_at(observed),
            Err(WorkError::Interrupted(Interruption::Shutdown))
        ));
        state.stopped.store(false, Ordering::Release);
        assert!(matches!(
            shutdown.checkpoint_at(observed),
            Err(WorkError::Interrupted(Interruption::Shutdown))
        ));
    }

    /// Query interruption has a closed safe envelope instead of an invalid-input error.
    #[test]
    fn read_query_interruption_errors_keep_typed_boundary() {
        for (reason, code, status) in [
            (Interruption::DeadlineExceeded, "query-budget-exceeded", 503),
            (Interruption::CancelRequested, "query-interrupted", 503),
            (Interruption::Shutdown, "shutdown-in-progress", 400),
        ] {
            let error = query_error(WorkError::Interrupted(reason));
            assert_eq!(error.code, code);
            assert_eq!(error_response(&error).status(), status);
            let value = serde_json::to_value(&error).unwrap();
            contract::validate_for(ApiMajor::V2, "Error", &value).unwrap();
            if reason != Interruption::Shutdown {
                assert!(contract::validate_for(ApiMajor::V1, "Error", &value).is_err());
            }
        }
        let safe = query_error(Error::containment().into());
        assert_eq!(safe.code, "resource-containment");
        assert_eq!(error_response(&safe).status(), 403);
    }

    /// Direct bundle failures use only the new closed API2 codes and retain shutdown semantics.
    #[test]
    fn bundle_preparation_interruptions_preserve_scoped_safe_envelopes() {
        for (reason, code, status) in [
            (Interruption::DeadlineExceeded, "bundle-preparation-budget-exceeded", 503),
            (Interruption::CancelRequested, "bundle-preparation-interrupted", 503),
            (Interruption::Shutdown, "shutdown-in-progress", 400),
        ] {
            let error = bundle_preparation_error(WorkError::Interrupted(reason));
            assert_eq!(error.code, code);
            assert_eq!(error_response(&error).status(), status);
            contract::validate_for(ApiMajor::V2, "Error", &serde_json::to_value(&error).unwrap())
                .unwrap();
            if reason != Interruption::Shutdown {
                assert!(
                    contract::validate_for(
                        ApiMajor::V1,
                        "Error",
                        &serde_json::to_value(&error).unwrap()
                    )
                    .is_err()
                );
            }
        }
        assert_eq!(
            bundle_preparation_error(Error::containment().into()).code,
            "resource-containment"
        );
    }

    /// A strict empty metadata bundle for budget/capture failure boundaries, without fake pin hashes.
    fn empty_bundle_import_request() -> Value {
        let index = super::super::index::Index::empty();
        json!({"bundle":{"schema_version":"forge.workspace-index-bundle/1",
            "content_profile":"index-and-hashes","index":index,
            "index_sha256":crate::hashing::sha256_hex(&index.bytes().unwrap()),"pins":[]},
            "target_index_schema_version":1,"acknowledge_index_replacement":true})
    }

    /// An already exhausted body/queue budget stops before malformed index capture or reservation.
    #[test]
    fn bundle_import_exhausted_admission_retains_no_receipt_or_key() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.workspace.json"), b"invalid index sentinel").unwrap();
        let mut state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        state.api_major = ApiMajor::V2;
        let state = Arc::new(state);
        let request = empty_bundle_import_request();
        let error = bundle_import_response(
            &state,
            "key",
            "/api/v2/project/bundle-imports",
            "",
            &request,
            Some(Instant::now()),
        )
        .unwrap_err();
        assert_eq!(error.code, "bundle-preparation-budget-exceeded");
        assert_eq!(error_response(&error).status(), 503);
        assert!(
            state
                .effects
                .lock()
                .unwrap()
                .replay("key", "POST", "/api/v2/project/bundle-imports", "", &request)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            std::fs::read(dir.path().join("forge.workspace.json")).unwrap(),
            b"invalid index sentinel"
        );
    }

    /// Failed capture releases its own key, permitting a later fresh attempt without reviving state.
    #[test]
    fn bundle_import_capture_failure_releases_owned_reservation() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.workspace.json"), b"invalid index sentinel").unwrap();
        let mut state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        state.api_major = ApiMajor::V2;
        let state = Arc::new(state);
        let request = empty_bundle_import_request();
        let error = bundle_import_response(
            &state,
            "key",
            "/api/v2/project/bundle-imports",
            "",
            &request,
            Instant::now().checked_add(Duration::from_secs(10)),
        )
        .unwrap_err();
        assert_eq!(error.code, "invalid-request");
        assert!(
            state
                .effects
                .lock()
                .unwrap()
                .replay("key", "POST", "/api/v2/project/bundle-imports", "", &request)
                .unwrap()
                .is_none()
        );
        assert_eq!(state.jobs.available_permits(), 1);
    }

    /// Worker unwinding releases its reservation instead of leaving an unretryable pending key.
    #[test]
    fn bundle_reservation_guard_releases_owned_key_on_worker_unwind() {
        let dir = tempfile::tempdir().unwrap();
        let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        let request = json!({});
        let nonce =
            state.effects.lock().unwrap().reserve("key", "POST", "/route", "", &request).unwrap();
        let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let local = state.effects.lock().unwrap().preparation_store("key", &nonce).unwrap();
            let _reservation =
                ReservationGuard { state: &state, key: "key", nonce, local: Some(local) };
            panic!("synthetic producer unwind");
        }));
        assert!(stopped.is_err());
        assert!(
            state
                .effects
                .lock()
                .unwrap()
                .replay("key", "POST", "/route", "", &request)
                .unwrap()
                .is_none()
        );
    }

    /// The consumed monotonic fence includes exact equality and fails closed.
    #[test]
    fn budget_expiry_includes_equality_and_unrepresentable_deadline() {
        let deadline = Instant::now() + Duration::from_secs(30);
        assert!(!budget_expired(
            Some(deadline),
            deadline.checked_sub(Duration::from_nanos(1)).unwrap()
        ));
        assert!(budget_expired(Some(deadline), deadline));
        assert!(budget_expired(Some(deadline), deadline + Duration::from_nanos(1)));
        assert!(budget_expired(None, deadline));
    }

    /// Progress cannot renew the acceptance deadline or count later stopped work.
    #[test]
    fn operation_progress_keeps_original_deadline() {
        let dir = tempfile::tempdir().unwrap();
        let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        let operation = state.effects.lock().unwrap().begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(state.effects.lock().unwrap().running(id).unwrap());
        let accepted = Instant::now();
        let deadline = accepted + Duration::from_secs(30);
        let mut control =
            OperationControl { state: &state, id, deadline: Some(deadline), interruption: None };
        control
            .checkpoint_at(ProgressUpdate::Capture { completed: 0, total: 101 }, accepted)
            .unwrap();
        control
            .checkpoint_at(
                ProgressUpdate::Capture { completed: 1, total: 101 },
                deadline.checked_sub(Duration::from_nanos(1)).unwrap(),
            )
            .unwrap();
        assert!(matches!(
            control.checkpoint_at(ProgressUpdate::Capture { completed: 2, total: 101 }, deadline),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        let observed = state.effects.lock().unwrap().operation(id).unwrap();
        assert_eq!(observed["progress"], json!({"completed_items":1,"total_items":101}));
        assert_eq!(control.deadline, Some(deadline));
        state.effects.lock().unwrap().finish(id, Err(Error::invalid()), true).unwrap();
        let settled = state.effects.lock().unwrap().operation(id).unwrap();
        assert_eq!(settled["state"], "cancelled");
        assert!(settled["progress"].is_null());
    }

    /// First observed cancellation, shutdown or expiry survives later boundaries.
    #[test]
    fn operation_checkpoint_stop_is_sticky_before_index_read() {
        for expected in
            [Interruption::CancelRequested, Interruption::Shutdown, Interruption::DeadlineExceeded]
        {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join("forge.workspace.json"), b"invalid index sentinel")
                .unwrap();
            let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
            let operation = state.effects.lock().unwrap().begin("export").unwrap();
            let id = operation["operation_id"].as_str().unwrap();
            assert!(state.effects.lock().unwrap().running(id).unwrap());
            let observed = Instant::now();
            let deadline = if expected == Interruption::DeadlineExceeded {
                observed
            } else {
                observed + Duration::from_secs(30)
            };
            if expected == Interruption::CancelRequested {
                state.effects.lock().unwrap().cancel(id).unwrap();
            }
            if expected == Interruption::Shutdown {
                state.stopped.store(true, Ordering::Release);
            }
            let mut control = OperationControl {
                state: &state,
                id,
                deadline: Some(deadline),
                interruption: None,
            };
            assert!(
                matches!(control.checkpoint_at(ProgressUpdate::Unchanged, observed), Err(WorkError::Interrupted(reason)) if reason == expected)
            );
            state.stopped.store(true, Ordering::Release);
            assert!(
                matches!(Snapshot::capture_with_control(&state.root, &mut control), Err(WorkError::Interrupted(reason)) if reason == expected)
            );
            assert_eq!(control.interruption(), Some(expected));
            state.effects.lock().unwrap().finish(id, Err(Error::invalid()), true).unwrap();
            assert_eq!(state.effects.lock().unwrap().operation(id).unwrap()["state"], "cancelled");
            assert_eq!(
                std::fs::read(dir.path().join("forge.workspace.json")).unwrap(),
                b"invalid index sentinel"
            );
        }
    }

    /// An expired queued job stops before capture rather than starting a fresh budget.
    #[test]
    fn operation_already_expired_at_worker_start_does_not_capture() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("forge.workspace.json"), b"invalid index sentinel").unwrap();
        let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        assert!(Snapshot::capture(&state.root).is_err());
        let operation = state.effects.lock().unwrap().begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(state.effects.lock().unwrap().running(id).unwrap());
        let mut control = OperationControl {
            state: &state,
            id,
            deadline: Some(Instant::now()),
            interruption: None,
        };
        assert!(matches!(
            Snapshot::capture_with_control(&state.root, &mut control),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        let observed = state.effects.lock().unwrap().operation(id).unwrap();
        assert!(observed.get("progress").is_none());
        assert!(observed.get("result").is_none());
    }

    /// An eligible pre-lock observation cannot permit work after budget expiry.
    #[test]
    fn operation_checkpoint_rechecks_budget_after_store_acquisition() {
        let dir = tempfile::tempdir().unwrap();
        let state = browser_state(Root::open(dir.path()).unwrap(), "correct long passphrase");
        let operation = state.effects.lock().unwrap().begin("export").unwrap();
        let id = operation["operation_id"].as_str().unwrap();
        assert!(state.effects.lock().unwrap().running(id).unwrap());
        let deadline = Instant::now();
        let mut control =
            OperationControl { state: &state, id, deadline: Some(deadline), interruption: None };
        let before_lock = deadline.checked_sub(Duration::from_nanos(1)).unwrap();
        assert!(matches!(
            control.checkpoint_at(ProgressUpdate::Capture { completed: 0, total: 1 }, before_lock),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        assert!(state.effects.lock().unwrap().operation(id).unwrap().get("progress").is_none());
    }
}

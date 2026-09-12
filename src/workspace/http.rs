//! Bounded HTTP/1 transport. No project content is served outside the API.

use std::convert::Infallible;
use std::io::Write as _;
use std::net::Ipv4Addr;
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use http_body_util::{BodyExt as _, Full, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response};
use serde_json::{Value, json};

use super::contract::{self, Error, Result};
use super::root::Root;
use super::services::{Snapshot, filtered, paginate};
use super::session::{Mode, Session};

const MAX_BODY: usize = 1024 * 1024;
const MAX_CONNECTIONS: usize = 16;

struct State {
    root: Root,
    host: String,
    origin: String,
    session: Mutex<Session>,
    stopped: AtomicBool,
    rate: Mutex<(Instant, u32)>,
    effects: Mutex<super::effects::Store>,
    work: Arc<tokio::sync::Semaphore>,
    jobs: Arc<tokio::sync::Semaphore>,
}

fn internal() -> Error {
    Error::new("internal-error", "The workspace operation could not be completed.", false)
}
fn unauthorized() -> Error {
    Error::new("unauthorized", "A valid local session request is required.", false)
}

pub(super) fn launch(project: &Path, read_only: bool, machine: bool, no_open: bool) -> Result<()> {
    let root = Root::open(project)?;
    // Validate the index and containment before exposing a listener or credentials.
    Snapshot::capture(&root)?;
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
            let descriptor = json!({"base_url":origin,"api_version":contract::VERSION,"session_id":session.id,
                "capability":&*capability,"mode":"machine","read_only":read_only,"pid":std::process::id()});
            let mut stdout = std::io::stdout().lock();
            serde_json::to_writer(&mut stdout, &descriptor).map_err(|_| internal())?;
            stdout.write_all(b"\n").and_then(|()| stdout.flush()).map_err(|_| internal())?;
        } else {
            eprintln!("Local workspace: {origin}");
            eprintln!("Stop with Ctrl-C. Local unlock does not authenticate reviewer identity.");
            if !no_open { open_browser(&origin)?; }
        }
        let state = Arc::new(State { root, host, origin, session:Mutex::new(session), stopped:AtomicBool::new(false), rate:Mutex::new((Instant::now(),0)),effects:Mutex::new(super::effects::Store::default()),work:Arc::new(tokio::sync::Semaphore::new(2)),jobs:Arc::new(tokio::sync::Semaphore::new(1)) });
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
    if request.uri().path() == "/api/v1/session/unlock" && request.method() == hyper::Method::POST {
        if !browser {
            return Err(unauthorized());
        }
    } else {
        let token = single_header(request, "authorization")?
            .and_then(|header| header.strip_prefix("Bearer "))
            .ok_or_else(unauthorized)?;
        let mutation = is_mutation(request.method().as_str(), request.uri().path());
        state.session.lock().map_err(|_| internal())?.authorize(token, browser, mutation)?;
    }
    Ok(())
}

fn unlock_response(
    state: &State,
    query: &[(String, String)],
    idempotency: Option<&str>,
    bytes: &[u8],
) -> Result<Response<Full<Bytes>>> {
    let mut payload = contract::parse(bytes, 4096, 512).ok();
    let valid = contract::operation_request(
        "POST",
        "/api/v1/session/unlock",
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
    let label = Snapshot::capture(&state.root)?.index.label;
    let session = session_view(state, &label)?;
    let capability = state.session.lock().map_err(|_| internal())?.unlock(if valid {
        &passphrase
    } else {
        ""
    })?;
    let response = json_response(
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

fn is_mutation(method: &str, path: &str) -> bool {
    method != "GET"
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
            return static_response(&method, &path);
        }
        let permit = Arc::clone(&state.work).try_acquire_owned().map_err(|_| {
            Error::new("invalid-request", "The workspace is busy. Retry shortly.", true)
        })?;
        let body_limit = if method == "POST" && path == "/api/v1/resources/upload" {
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
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            dispatch(&state, &method, &path, &query, idempotency.as_deref(), &body)
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

#[allow(clippy::needless_pass_by_value)] // Release owned response data after serialization.
fn json_response(value: Value, schema: &str) -> Result<Response<Full<Bytes>>> {
    contract::validate(schema, &value).map_err(|_| internal())?;
    let bytes = contract::encode(&value, 4 * 1024 * 1024, false)?;
    Ok(response(200, "application/json", bytes))
}

fn session_view(state: &State, label: &str) -> Result<Value> {
    let session = state.session.lock().map_err(|_| internal())?;
    Ok(json!({"session_id":session.id,"mode":session.mode,"read_only":session.read_only,
        "api_major":1,"contract_version":contract::VERSION,"project_label":label,"launched_at":session.launched_at}))
}

#[allow(clippy::too_many_lines)] // Explicit normative route dispatch; domain rules stay in services.
fn dispatch(
    state: &Arc<State>,
    method: &str,
    path: &str,
    raw_query: &str,
    idempotency: Option<&str>,
    bytes: &[u8],
) -> Result<Response<Full<Bytes>>> {
    if method == "GET" && !bytes.is_empty() {
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
    contract::operation_request(method, path, &query, idempotency, payload.as_ref())?;
    if method == "POST" && path == "/api/v1/session/shutdown" {
        state.stopped.store(true, Ordering::Release);
        return json_response(json!({"state":"shutting-down"}), "ShutdownResponse");
    }
    if method == "GET" {
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
        let mut store = state.effects.lock().map_err(|_| internal())?;
        let reply = if let Some(reply) = store.replay(key, method, path, raw_query, &request)? {
            reply
        } else {
            let kind = match path {
                "/api/v1/conversions" => Some("conversion"),
                "/api/v1/applicability/analyses" => Some("applicability-analysis"),
                "/api/v1/mapping/builds" => Some("mapping-build"),
                "/api/v1/exports" => Some("export"),
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
                let operation = store.begin(kind)?;
                let id = operation["operation_id"].as_str().ok_or_else(internal)?.to_owned();
                let shared = Arc::clone(state);
                let method = method.to_owned();
                let path = path.to_owned();
                let request = request.clone();
                tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    let start = Instant::now();
                    let proceed = shared
                        .effects
                        .lock()
                        .ok()
                        .and_then(|mut store| store.running(&id).ok())
                        .unwrap_or(false);
                    if !proceed {
                        return;
                    }
                    let result = (|| {
                        let mut local = super::effects::Store::default();
                        let mut snapshot = Snapshot::capture(&shared.root)?;
                        let reply = super::actions::prepare(
                            &mut local,
                            &shared.root,
                            &mut snapshot,
                            &method,
                            &path,
                            &request,
                        )?;
                        Ok((local, reply))
                    })();
                    let cancelled = shared.stopped.load(Ordering::Acquire)
                        || start.elapsed() > Duration::from_secs(30);
                    if let Ok(mut store) = shared.effects.lock() {
                        let _ = store.finish(&id, result, cancelled);
                    }
                });
                super::effects::Reply { value: operation, schema: "Operation", status: 202 }
            } else if path == "/api/v1/effects/commits" {
                super::effects::Reply {
                    value: store.commit(&state.root, &request, &state.stopped)?,
                    schema: "Operation",
                    status: 202,
                }
            } else {
                let mut snapshot = Snapshot::capture(&state.root)?;
                super::actions::prepare(
                    &mut store,
                    &state.root,
                    &mut snapshot,
                    method,
                    path,
                    &request,
                )?
            };
            store.remember(key, method, path, raw_query, &request, &reply)?;
            reply
        };
        let mut response = json_response(reply.value, reply.schema)?;
        *response.status_mut() =
            hyper::StatusCode::from_u16(reply.status).map_err(|_| internal())?;
        return Ok(response);
    }
    let mut snapshot = Snapshot::capture(&state.root)?;
    match (method, path) {
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

fn static_response(method: &str, path: &str) -> Result<Response<Full<Bytes>>> {
    if method != "GET" {
        return Err(Error::new("not-found", "The requested asset was not found.", false));
    }
    if path == "/" {
        Ok(response(200, "text/html; charset=utf-8", super::assets::shell().into_bytes()))
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

fn error_response(error: &Error) -> Response<Full<Bytes>> {
    let status = match error.code {
        "unauthorized" | "unlock-failed" => 401,
        "read-only-session" | "resource-containment" => 403,
        "not-found" => 404,
        "version-conflict"
        | "idempotency-key-conflict"
        | "receipt-reused"
        | "receipt-mismatch"
        | "operation-not-cancellable" => 409,
        "receipt-expired" => 410,
        "payload-too-large" => 413,
        "unsupported-media-type" => 415,
        "validation-failed" => 422,
        "unlock-throttled" => 429,
        "internal-error" => 500,
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

    fn browser_state(root: Root, passphrase: &str) -> State {
        State {
            root,
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
}

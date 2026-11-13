//! In-process loopback fake GitHub Issues server (PRD 065 M-15).
//!
//! A plain `std::net::TcpListener` HTTP/1.1 fake that implements the same
//! idempotency, version-precondition, rate-limit, and redaction semantics a live
//! GitHub adapter must honor. It exists so the plan/apply/reconcile state machine
//! and the network-safety envelope can be contract-tested with zero outbound
//! access and no new dependencies. It is never used for real work.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::adapter::{
    CredentialProvider, OperationKind, OperationRequest, RemoteObject, Transport, TransportError,
};
use super::config::NetworkPolicy;
use super::{error, reject_secret_text};
use crate::ForgeError;
use crate::hashing::sha256_hex;

/// One fake remote issue.
#[derive(Debug, Clone)]
struct Issue {
    remote_id: String,
    version: u64,
    state: String,
    title: String,
    body: String,
}

/// Deterministic injection knobs for contract fixtures. All default to off.
#[derive(Debug, Default)]
pub struct Injections {
    /// Answer the next N requests with 429 + `Retry-After: 0`.
    pub rate_limit_next: u32,
    /// Fail the next N creates with a 500 (set above `MAX_ATTEMPTS` to force a
    /// persistent partial failure across all retries).
    pub fail_next_creates: u32,
    /// Answer the next request with a 302 redirect once.
    pub redirect_next: bool,
    /// Answer the next request with an oversized body once.
    pub oversize_next: bool,
    /// Delay every response by this many milliseconds (timeout fixtures).
    pub delay_ms: u64,
    /// Externally bump a remote object's version to simulate drift (conflict fixtures).
    pub drift: Option<(String, u64)>,
}

/// Redacted request log row: (method, path, `idempotency_key`). Never the auth header.
type RequestLogRow = (String, String, String);
/// Fake response: (status, extra headers, body).
type FakeResponse = (u16, Vec<(String, String)>, Vec<u8>);

/// Shared fake provider state.
#[derive(Default)]
struct State {
    /// (target, `idempotency_key`) -> recorded create outcome, for duplicate prevention.
    creates: BTreeMap<(String, String), Issue>,
    /// `remote_id` -> issue.
    issues: BTreeMap<String, Issue>,
    next_id: u64,
    injections: Injections,
    /// Redacted request log. Never the auth header.
    requests: Vec<RequestLogRow>,
}

/// A running loopback fake server. Drop stops the listener thread.
pub struct FakeServer {
    state: Arc<Mutex<State>>,
    addr: std::net::SocketAddr,
    stop: Arc<std::sync::atomic::AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl FakeServer {
    /// Bind an ephemeral loopback port and start serving.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when the listener cannot bind.
    pub fn start() -> Result<Self, ForgeError> {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|cause| error(format!("fake server bind failed: {cause}")))?;
        let addr = listener.local_addr().map_err(|cause| error(cause.to_string()))?;
        listener.set_nonblocking(true).map_err(|cause| error(cause.to_string()))?;
        let state = Arc::new(Mutex::new(State::default()));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let server_state = Arc::clone(&state);
        let server_stop = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !server_stop.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let state = Arc::clone(&server_state);
                        std::thread::spawn(move || {
                            let _ = serve_connection(stream, &state);
                        });
                    }
                    Err(cause) if cause.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self { state, addr, stop, handle: Some(handle) })
    }

    /// The loopback endpoint the transport must dial.
    #[must_use]
    pub const fn addr(&self) -> std::net::SocketAddr {
        self.addr
    }

    /// Mutate injection knobs before issuing requests.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] if the state lock is poisoned.
    pub fn inject(&self, mutate: impl FnOnce(&mut Injections)) -> Result<(), ForgeError> {
        let mut state = self.state.lock().map_err(|_| error("fake state poisoned"))?;
        mutate(&mut state.injections);
        Ok(())
    }

    /// Number of remote issues currently stored (duplicate-prevention assertions).
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] if the state lock is poisoned.
    pub fn issue_count(&self) -> Result<usize, ForgeError> {
        Ok(self.state.lock().map_err(|_| error("fake state poisoned"))?.issues.len())
    }

    /// Redacted request log for assertions; never contains credentials.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] if the state lock is poisoned.
    pub fn requests(&self) -> Result<Vec<RequestLogRow>, ForgeError> {
        Ok(self.state.lock().map_err(|_| error("fake state poisoned"))?.requests.clone())
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Synthetic credential provider for Phase 1 fixtures. The token is a fixed
/// synthetic value, never a real secret, and never enters reports or state.
pub struct FakeCredentials;

impl CredentialProvider for FakeCredentials {
    fn token(&self) -> Result<String, TransportError> {
        Ok("fake-phase1-token".to_string())
    }
}

/// Loopback transport speaking minimal HTTP/1.1 to the fake server while
/// enforcing the closed network-safety envelope (M-13).
pub struct FakeTransport {
    addr: std::net::SocketAddr,
    credentials: Box<dyn CredentialProvider>,
}

impl FakeTransport {
    /// Build a transport bound to one fake server endpoint.
    #[must_use]
    pub fn new(addr: std::net::SocketAddr, credentials: Box<dyn CredentialProvider>) -> Self {
        Self { addr, credentials }
    }

    fn round_trip(
        &self,
        policy: &NetworkPolicy,
        method: &str,
        path: &str,
        idempotency_key: &str,
        body: Option<&[u8]>,
    ) -> Result<FakeResponse, TransportError> {
        if let Some(bytes) = body
            && bytes.len() > policy.max_request_bytes
        {
            return Err(TransportError::RequestTooLarge);
        }
        let token = self.credentials.token()?;
        let mut stream = TcpStream::connect(self.addr)
            .map_err(|_| TransportError::Protocol("connect failed".to_string()))?;
        let timeout = Duration::from_millis(policy.timeout_ms);
        stream
            .set_read_timeout(Some(timeout))
            .and_then(|()| stream.set_write_timeout(Some(timeout)))
            .map_err(|_| TransportError::Protocol("socket timeout setup failed".to_string()))?;
        let body = body.unwrap_or(&[]);
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {token}\r\n\
             Idempotency-Key: {idempotency_key}\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n",
            policy.host,
            body.len()
        );
        stream
            .write_all(request.as_bytes())
            .and_then(|()| stream.write_all(body))
            .map_err(|_| TransportError::Protocol("request write failed".to_string()))?;
        read_response(&mut stream, policy.max_response_bytes)
    }
}

impl Transport for FakeTransport {
    fn execute(
        &mut self,
        policy: &NetworkPolicy,
        request: &OperationRequest,
    ) -> Result<RemoteObject, TransportError> {
        reject_secret_text(&request.payload.title, "payload")
            .and_then(|()| reject_secret_text(&request.payload.body, "payload"))
            .map_err(|_| {
                TransportError::Protocol("payload contains credential-shaped content".into())
            })?;
        let (method, path) = match request.kind {
            OperationKind::Create => ("POST", format!("/repos/{}/issues", request.target)),
            OperationKind::Update | OperationKind::CloseRequest => {
                let precondition = request
                    .precondition
                    .as_ref()
                    .ok_or_else(|| TransportError::Protocol("missing precondition".into()))?;
                ("PATCH", format!("/repos/{}/issues/{}", request.target, precondition.remote_id))
            }
        };
        let mut payload = json!({
            "title": request.payload.title,
            "body": request.payload.body,
            "operation": request.kind.as_str(),
        });
        if let Some(precondition) = &request.precondition {
            payload["expected_version"] = json!(precondition.remote_version);
        }
        let body = serde_json::to_vec(&payload)
            .map_err(|_| TransportError::Protocol("payload encode failed".into()))?;
        let (status, headers, bytes) =
            self.round_trip(policy, method, &path, &request.idempotency_key, Some(&body))?;
        classify(status, &headers, &bytes)
    }

    fn observe(
        &mut self,
        policy: &NetworkPolicy,
        target: &str,
        remote_id: &str,
    ) -> Result<Option<RemoteObject>, TransportError> {
        let path = format!("/repos/{target}/issues/{remote_id}");
        let (status, headers, bytes) = self.round_trip(policy, "GET", &path, "", None)?;
        if status == 404 {
            return Ok(None);
        }
        classify(status, &headers, &bytes).map(Some)
    }
}

/// Map an HTTP response onto the closed transport outcome/error vocabulary.
fn classify(
    status: u16,
    headers: &[(String, String)],
    bytes: &[u8],
) -> Result<RemoteObject, TransportError> {
    match status {
        200 | 201 => {
            let value: Value = serde_json::from_slice(bytes)
                .map_err(|_| TransportError::Protocol("malformed provider response".into()))?;
            Ok(RemoteObject {
                remote_id: value["remote_id"].as_str().unwrap_or_default().to_string(),
                remote_version: value["remote_version"].as_u64().unwrap_or_default(),
                state: value["state"].as_str().unwrap_or_default().to_string(),
            })
        }
        301 | 302 | 303 | 307 | 308 => Err(TransportError::RedirectRefused),
        404 => Err(TransportError::NotFound),
        409 => {
            let value: Value = serde_json::from_slice(bytes).unwrap_or(Value::Null);
            Err(TransportError::Conflict {
                expected: value["expected"].as_u64().unwrap_or_default(),
                observed: value["observed"].as_u64().unwrap_or_default(),
            })
        }
        429 => {
            let retry = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("retry-after"))
                .and_then(|(_, value)| value.parse::<u64>().ok())
                .unwrap_or(1);
            Err(TransportError::RateLimited { retry_after_secs: retry })
        }
        413 => Err(TransportError::RequestTooLarge),
        500..=599 => Err(TransportError::ServerFailure { status }),
        other => Err(TransportError::ServerFailure { status: other }),
    }
}

/// Read one bounded HTTP response, refusing oversized bodies (M-13).
fn read_response(stream: &mut TcpStream, max_bytes: usize) -> Result<FakeResponse, TransportError> {
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).map_err(|_| TransportError::Timeout)?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| TransportError::Protocol("malformed status line".into()))?;
    let mut headers = Vec::new();
    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|_| TransportError::Timeout)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            let name = name.trim().to_string();
            let value = value.trim().to_string();
            if name.eq_ignore_ascii_case("content-length") {
                content_length =
                    value.parse().map_err(|_| TransportError::Protocol("bad length".into()))?;
            }
            headers.push((name, value));
        }
    }
    if content_length > max_bytes {
        return Err(TransportError::ResponseTooLarge);
    }
    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body).map_err(|_| TransportError::Timeout)?;
    Ok((status, headers, body))
}

/// Serve one HTTP connection against the shared fake state.
fn serve_connection(stream: TcpStream, state: &Arc<Mutex<State>>) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();
    let mut content_length = 0_usize;
    let mut idempotency_key = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            let name = name.trim();
            let value = value.trim();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse().unwrap_or(0);
            } else if name.eq_ignore_ascii_case("idempotency-key") {
                idempotency_key = value.to_string();
            }
            // The Authorization header is deliberately never retained (M-14).
        }
    }
    let mut body = vec![0_u8; content_length];
    reader.read_exact(&mut body)?;

    let (status, extra_headers, response) = {
        let mut state = state.lock().map_err(|_| std::io::Error::other("poisoned"))?;
        state.requests.push((method.clone(), path.clone(), idempotency_key.clone()));
        handle_request(&mut state, &method, &path, &idempotency_key, &body)
    };

    let mut stream = stream;
    let mut header_text = String::new();
    for (name, value) in &extra_headers {
        let _ = write!(header_text, "{name}: {value}\r\n");
    }
    let head = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\n{header_text}\
         Content-Length: {}\r\nConnection: close\r\n\r\n",
        reason(status),
        response.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&response)?;
    stream.flush()
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        302 => "Found",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        _ => "Status",
    }
}

/// Pure fake state machine. Returns (status, extra headers, body).
#[allow(clippy::too_many_arguments)] // One request carries method, path, key and body separately.
fn handle_request(
    state: &mut State,
    method: &str,
    path: &str,
    idempotency_key: &str,
    body: &[u8],
) -> FakeResponse {
    if state.injections.delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(state.injections.delay_ms));
    }
    if state.injections.rate_limit_next > 0 {
        state.injections.rate_limit_next -= 1;
        return (
            429,
            vec![("Retry-After".to_string(), "0".to_string())],
            b"{\"error\":\"rate limited\"}".to_vec(),
        );
    }
    if state.injections.redirect_next {
        state.injections.redirect_next = false;
        return (
            302,
            vec![("Location".to_string(), "https://elsewhere.example/".to_string())],
            Vec::new(),
        );
    }
    if state.injections.oversize_next {
        state.injections.oversize_next = false;
        return (200, Vec::new(), vec![b'x'; 8 * 1024 * 1024]);
    }

    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    // /repos/{owner}/{repo}/issues[/{id}]
    if segments.len() < 4 || segments[0] != "repos" || segments[3] != "issues" {
        return (404, Vec::new(), b"{\"error\":\"not found\"}".to_vec());
    }
    let target = format!("{}/{}", segments[1], segments[2]);
    let remote_id = segments.get(4).map(|value| (*value).to_string());

    match (method, remote_id) {
        ("POST", None) => {
            if state.injections.fail_next_creates > 0 {
                state.injections.fail_next_creates -= 1;
                return (500, Vec::new(), b"{\"error\":\"injected failure\"}".to_vec());
            }
            let key = (target, idempotency_key.to_string());
            if let Some(existing) = state.creates.get(&key) {
                return (200, Vec::new(), issue_json(existing));
            }
            let payload: Value = match serde_json::from_slice(body) {
                Ok(value) => value,
                Err(_) => return (400, Vec::new(), b"{\"error\":\"bad payload\"}".to_vec()),
            };
            state.next_id += 1;
            let issue = Issue {
                remote_id: format!("issue-{}", state.next_id),
                version: 1,
                state: "open".to_string(),
                title: payload["title"].as_str().unwrap_or_default().to_string(),
                body: payload["body"].as_str().unwrap_or_default().to_string(),
            };
            state.issues.insert(issue.remote_id.clone(), issue.clone());
            state.creates.insert(key, issue.clone());
            (201, Vec::new(), issue_json(&issue))
        }
        ("PATCH", Some(id)) => {
            if let Some((drift_id, version)) = state.injections.drift.take()
                && let Some(issue) = state.issues.get_mut(&drift_id)
            {
                issue.version = version;
            }
            let payload: Value = match serde_json::from_slice(body) {
                Ok(value) => value,
                Err(_) => return (400, Vec::new(), b"{\"error\":\"bad payload\"}".to_vec()),
            };
            let Some(issue) = state.issues.get_mut(&id) else {
                return (404, Vec::new(), b"{\"error\":\"not found\"}".to_vec());
            };
            let expected = payload["expected_version"].as_u64().unwrap_or(0);
            if expected != issue.version {
                return (
                    409,
                    Vec::new(),
                    serde_json::to_vec(&json!({"expected": expected, "observed": issue.version}))
                        .unwrap_or_default(),
                );
            }
            issue.version += 1;
            issue.title = payload["title"].as_str().unwrap_or(&issue.title).to_string();
            issue.body = payload["body"].as_str().unwrap_or(&issue.body).to_string();
            if payload["operation"].as_str() == Some("close-request") {
                issue.state = "closed".to_string();
            }
            (200, Vec::new(), issue_json(issue))
        }
        ("GET", Some(id)) => match state.issues.get(&id) {
            Some(issue) => (200, Vec::new(), issue_json(issue)),
            None => (404, Vec::new(), b"{\"error\":\"not found\"}".to_vec()),
        },
        _ => (404, Vec::new(), b"{\"error\":\"not found\"}".to_vec()),
    }
}

fn issue_json(issue: &Issue) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "remote_id": issue.remote_id,
        "remote_version": issue.version,
        "state": issue.state,
    }))
    .unwrap_or_default()
}

/// Stable idempotency key for one work item against one target (M-8).
#[must_use]
pub fn idempotency_key(target: &str, item_id: &str) -> String {
    format!("idem_{}", sha256_hex(format!("{target}\0{item_id}").as_bytes()))
}

/// Minimal payload body rendering shared by planner and adapter previews (M-12).
#[derive(Debug, Serialize, Deserialize)]
pub struct WirePayload {
    /// Remote title.
    pub title: String,
    /// Deterministic minimized body.
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::adapter::{Payload, Precondition};

    fn policy() -> NetworkPolicy {
        NetworkPolicy::default()
    }

    fn create_request(target: &str, key: &str) -> OperationRequest {
        OperationRequest {
            target: target.to_string(),
            idempotency_key: key.to_string(),
            kind: OperationKind::Create,
            payload: Payload { title: "t".to_string(), body: "b".to_string() },
            precondition: None,
        }
    }

    #[test]
    fn create_is_idempotent_and_records_identity() {
        let server = FakeServer::start().unwrap();
        let mut transport = FakeTransport::new(server.addr(), Box::new(FakeCredentials));
        let request = create_request("a/b", "key-1");
        let first = transport.execute(&policy(), &request).unwrap();
        let second = transport.execute(&policy(), &request).unwrap();
        assert_eq!(first, second);
        assert_eq!(server.issue_count().unwrap(), 1);
    }

    #[test]
    fn update_requires_matching_version() {
        let server = FakeServer::start().unwrap();
        let mut transport = FakeTransport::new(server.addr(), Box::new(FakeCredentials));
        let created = transport.execute(&policy(), &create_request("a/b", "k")).unwrap();
        let mut update = create_request("a/b", "k");
        update.kind = OperationKind::Update;
        update.precondition =
            Some(Precondition { remote_id: created.remote_id.clone(), remote_version: 99 });
        let conflict = transport.execute(&policy(), &update).unwrap_err();
        assert_eq!(conflict, TransportError::Conflict { expected: 99, observed: 1 });
        update.precondition =
            Some(Precondition { remote_id: created.remote_id.clone(), remote_version: 1 });
        let updated = transport.execute(&policy(), &update).unwrap();
        assert_eq!(updated.remote_version, 2);
    }

    #[test]
    fn rate_limit_and_redirect_are_refused() {
        let server = FakeServer::start().unwrap();
        let mut transport = FakeTransport::new(server.addr(), Box::new(FakeCredentials));
        server.inject(|i| i.rate_limit_next = 1).unwrap();
        assert_eq!(
            transport.execute(&policy(), &create_request("a/b", "k")).unwrap_err(),
            TransportError::RateLimited { retry_after_secs: 0 }
        );
        server.inject(|i| i.redirect_next = true).unwrap();
        assert_eq!(
            transport.execute(&policy(), &create_request("a/b", "k")).unwrap_err(),
            TransportError::RedirectRefused
        );
    }

    #[test]
    fn oversized_response_and_timeout_are_refused() {
        let server = FakeServer::start().unwrap();
        let mut transport = FakeTransport::new(server.addr(), Box::new(FakeCredentials));
        server.inject(|i| i.oversize_next = true).unwrap();
        assert_eq!(
            transport.execute(&policy(), &create_request("a/b", "k")).unwrap_err(),
            TransportError::ResponseTooLarge
        );
        server.inject(|i| i.delay_ms = 200).unwrap();
        let mut short = policy();
        short.timeout_ms = 20;
        assert_eq!(
            transport.execute(&short, &create_request("a/b", "k2")).unwrap_err(),
            TransportError::Timeout
        );
    }

    #[test]
    fn request_log_never_contains_credentials() {
        let server = FakeServer::start().unwrap();
        let mut transport = FakeTransport::new(server.addr(), Box::new(FakeCredentials));
        transport.execute(&policy(), &create_request("a/b", "k")).unwrap();
        let log = server.requests().unwrap();
        assert_eq!(log.len(), 1);
        let rendered = format!("{log:?}");
        assert!(!rendered.contains("fake-phase1-token"));
        assert!(!rendered.to_ascii_lowercase().contains("bearer"));
    }
}

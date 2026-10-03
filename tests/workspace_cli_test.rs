//! Real process/HTTP coverage for the local workspace boundary.

use std::fmt::Write as _;
use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use serde_json::{Value, json};

struct Server {
    process: Child,
    host: String,
    capability: String,
    project: tempfile::TempDir,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Server {
    fn launch(with_resource: bool) -> Self {
        Self::launch_mode(with_resource, true)
    }

    /// Launch one authenticated machine child within 20 seconds; failed construction
    /// retains bounded redacted diagnostics and always cleans up the owned child.
    fn launch_mode(with_resource: bool, read_only: bool) -> Self {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("unregistered.md"), "PRIVATE UNREGISTERED CONTENT")
            .unwrap();
        if with_resource {
            std::fs::write(
                project.path().join("policy.md"),
                "# Example\n\nA human-supplied clause.\n",
            )
            .unwrap();
            let index = json!({"schema_version":"forge.workspace/1","label":"Example project","resources":[{"key":"policy","role":"policy-source","path":"policy.md"}]});
            std::fs::write(
                project.path().join("forge.workspace.json"),
                serde_json::to_vec(&index).unwrap(),
            )
            .unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_forge"));
        command.args(["workspace", "--project"]).arg(project.path()).arg("--machine-session");
        if read_only {
            command.arg("--read-only");
        }
        // A regular owned file avoids a stderr pipe filling before the descriptor.
        let mut stderr = tempfile::tempfile().expect("owned startup stderr sink");
        let process = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(stderr.try_clone().expect("startup stderr descriptor"))
            .spawn()
            .unwrap();
        // Install the existing Drop owner before any post-spawn fallible check.
        let mut server = Self { process, host: String::new(), capability: String::new(), project };
        let stdout = server.process.stdout.take().expect("machine stdout pipe");
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result =
                std::io::BufReader::new(stdout).read_line(&mut line).map(|bytes| (bytes, line));
            let _ = send.send(result);
        });
        let line = match receive.recv_timeout(Duration::from_secs(20)) {
            Ok(Ok((0, _))) => server.startup_failure(&mut stderr, "descriptor EOF"),
            Ok(Ok((_, line))) => line,
            Ok(Err(_)) => server.startup_failure(&mut stderr, "descriptor read error"),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                server.startup_failure(&mut stderr, "descriptor deadline exceeded")
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                server.startup_failure(&mut stderr, "descriptor reader disconnected")
            }
        };
        let descriptor: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => server.startup_failure(&mut stderr, "descriptor JSON invalid"),
        };
        server.host =
            match descriptor["base_url"].as_str().and_then(|url| url.strip_prefix("http://")) {
                Some(host) if host.starts_with("127.0.0.1:") => host.to_owned(),
                _ => server.startup_failure(&mut stderr, "descriptor loopback URL invalid"),
            };
        server.capability = match descriptor["capability"].as_str() {
            Some(capability) => capability.to_owned(),
            None => server.startup_failure(&mut stderr, "descriptor capability absent"),
        };
        if descriptor["mode"] != "machine" || descriptor["read_only"] != read_only {
            server.startup_failure(&mut stderr, "descriptor mode or scope mismatch");
        }
        server
    }

    /// Reap the failed direct child and expose only status plus whitelisted startup
    /// classifications; never render descriptor bytes, credentials or arbitrary stderr.
    fn startup_failure(&mut self, stderr: &mut std::fs::File, reason: &str) -> ! {
        use std::io::Seek as _;

        // EOF can precede the parent's exit notification; briefly collect the actual status.
        let started = std::time::Instant::now();
        let mut observed = None;
        let mut status_error = false;
        loop {
            match self.process.try_wait() {
                Ok(Some(status)) => {
                    observed = Some(status);
                    break;
                }
                Ok(None) if started.elapsed() < Duration::from_secs(2) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Ok(None) => break,
                Err(_) => {
                    status_error = true;
                    break;
                }
            }
        }
        let forced_cleanup = observed.is_none();
        let mut kill_error = false;
        if forced_cleanup {
            kill_error = self.process.kill().is_err();
            match self.process.wait() {
                Ok(status) => observed = Some(status),
                Err(_) => status_error = true,
            }
        }
        let status = observed.map_or_else(|| "unavailable".to_owned(), |status| status.to_string());
        let total_stderr_bytes = stderr.metadata().ok().map(|meta| meta.len());
        let mut bytes = Vec::new();
        let stderr_read = stderr
            .rewind()
            .and_then(|()| (&mut *stderr).take(16 * 1024 + 1).read_to_end(&mut bytes));
        let truncated =
            bytes.len() > 16 * 1024 || total_stderr_bytes.is_some_and(|len| len > 16 * 1024);
        bytes.truncate(16 * 1024);
        let text = String::from_utf8_lossy(&bytes);
        let classification = if text
            .contains("A qualified source restore transaction is unavailable for this project.")
        {
            "qualified source restore unavailable"
        } else if bytes.is_empty() {
            "empty"
        } else {
            "unrecognized startup stderr redacted"
        };
        panic!(
            "workspace startup failed: reason={reason}; child_status={status}; forced_cleanup={forced_cleanup}; kill_error={kill_error}; status_error={status_error}; stderr_read_ok={}; stderr_bytes={total_stderr_bytes:?}; stderr_truncated={truncated}; stderr_class={classification}",
            stderr_read.is_ok()
        );
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        authorized: bool,
        host: Option<&str>,
        extra: &str,
        body: &str,
    ) -> (u16, String, Vec<u8>) {
        let mut stream = TcpStream::connect(&self.host).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
        let auth = if authorized {
            format!("Authorization: Bearer {}\r\n", self.capability)
        } else {
            String::new()
        };
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\n{auth}{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            host.unwrap_or(&self.host),
            body.len()
        );
        stream.write_all(request.as_bytes()).unwrap();
        let mut bytes = Vec::new();
        stream.take(8 * 1024 * 1024).read_to_end(&mut bytes).unwrap();
        let boundary = bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .expect("HTTP response headers");
        let headers = String::from_utf8(bytes[..boundary].to_vec()).unwrap();
        let status = headers.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, headers, bytes[boundary + 4..].to_vec())
    }
}

#[test]
fn machine_setup_is_authenticated_offline_and_non_scanning() {
    let server = Server::launch(false);
    let (status, headers, body) = server.request("GET", "/", false, None, "", "");
    assert_eq!(status, 200);
    assert!(headers.contains("content-security-policy:"));
    assert!(!String::from_utf8(body).unwrap().contains("PRIVATE UNREGISTERED"));
    assert_eq!(server.request("GET", "/api/v1/project/summary", false, None, "", "").0, 401);
    let (status, headers, body) =
        server.request("GET", "/api/v1/project/summary", true, None, "", "");
    assert_eq!(status, 200);
    assert!(headers.contains("no-store"));
    let summary: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary["health"], "setup");
    assert_eq!(summary["resource_counts"]["total"], 0);
    assert_eq!(
        server.request("GET", "/api/v1/project/summary", true, Some("localhost:1234"), "", "").0,
        401
    );
    assert_eq!(
        server
            .request(
                "GET",
                "/api/v1/project/summary",
                true,
                None,
                "X-Forwarded-Host: example.test\r\n",
                ""
            )
            .0,
        401
    );
    assert_eq!(
        server
            .request(
                "GET",
                "/api/v1/project/summary",
                true,
                None,
                "Origin: https://hostile.test\r\nSec-Fetch-Site: cross-site\r\n",
                ""
            )
            .0,
        401
    );
    assert_eq!(
        server
            .request(
                "POST",
                "/api/v1/resources/register",
                true,
                None,
                "Content-Type: application/json\r\n",
                "{}"
            )
            .0,
        403
    );
    assert_eq!(
        server
            .request(
                "POST",
                "/api/v1/session/shutdown",
                true,
                None,
                "Content-Type: application/json\r\n",
                "{}"
            )
            .0,
        200
    );
}

#[test]
fn registered_resources_are_typed_and_raw_content_is_not_a_route() {
    let server = Server::launch(true);
    let (status, _, body) = server.request("GET", "/api/v1/resources", true, None, "", "");
    assert_eq!(status, 200);
    let result: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["page"]["total_matching"], 1);
    assert_eq!(result["page"]["items"][0]["validation_state"], "valid");
    assert!(!String::from_utf8(body).unwrap().contains("human-supplied clause"));
    assert_eq!(server.request("GET", "/policy.md", true, None, "", "").0, 404);
    assert_eq!(
        server.request("GET", "/api/v1/resources?path=unregistered.md", true, None, "", "").0,
        400
    );
}

impl Server {
    fn json(&self, method: &str, path: &str, key: Option<&str>, value: &Value) -> (u16, Value) {
        let mut headers = String::from("Content-Type: application/json\r\n");
        if let Some(key) = key {
            write!(headers, "Idempotency-Key: {key}\r\n").unwrap();
        }
        let body =
            if method == "GET" { String::new() } else { serde_json::to_string(value).unwrap() };
        let (status, _, bytes) = self.request(method, path, true, None, &headers, &body);
        let mut value: Value = serde_json::from_slice(&bytes).unwrap();
        if status == 202 && value["kind"] != "commit" {
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while matches!(value["state"].as_str(), Some("pending" | "running")) {
                assert!(std::time::Instant::now() < deadline, "operation deadline");
                std::thread::sleep(Duration::from_millis(100));
                let query =
                    format!("/api/v1/operations/{}", value["operation_id"].as_str().unwrap());
                let (_, _, bytes) = self.request("GET", &query, true, None, "", "");
                value = serde_json::from_slice(&bytes).unwrap();
            }
        }
        (status, value)
    }
}

#[test]
fn exact_preview_commit_is_idempotent_and_registration_is_separate() {
    let server = Server::launch_mode(false, false);
    let request = json!({"role":"policy-source","path":"unregistered.md","key":"policy"});
    let (status, first) =
        server.json("POST", "/api/v1/resources/register", Some("register-policy-0001"), &request);
    assert_eq!(status, 200, "{first}");
    assert!(!server.project.path().join("forge.workspace.json").exists());
    let (_, retry) =
        server.json("POST", "/api/v1/resources/register", Some("register-policy-0001"), &request);
    assert_eq!(first, retry);
    // The same key with different content is a typed conflict, never a replay.
    let (status, conflict) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some("register-policy-0001"),
        &json!({"role":"policy-source","path":"unregistered.md","key":"other"}),
    );
    assert_eq!(status, 409, "{conflict}");
    assert_eq!(conflict["code"], "idempotency-key-conflict");
    // The request hash binds the raw query string: an otherwise identical retry
    // under a different raw query is a different request. A fresh key on that
    // same raw query is accepted, proving the query itself is not rejected.
    let (status, fresh) =
        server.json("POST", "/api/v1/resources/register?&", Some("register-policy-0002"), &request);
    assert_eq!(status, 200, "{fresh}");
    let (status, conflict) =
        server.json("POST", "/api/v1/resources/register?&", Some("register-policy-0001"), &request);
    assert_eq!(status, 409, "{conflict}");
    assert_eq!(conflict["code"], "idempotency-key-conflict");
    let preview = &first["preview"];
    let commit = json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true});
    let (status, operation) =
        server.json("POST", "/api/v1/effects/commits", Some("commit-policy-00001"), &commit);
    assert_eq!(status, 202, "{operation}");
    assert_eq!(operation["state"], "succeeded");
    let bytes = std::fs::read(server.project.path().join("forge.workspace.json")).unwrap();
    let (status, retry) =
        server.json("POST", "/api/v1/effects/commits", Some("commit-policy-00001"), &commit);
    assert_eq!(status, 202, "{retry}");
    assert_eq!(operation, retry);
    assert_eq!(bytes, std::fs::read(server.project.path().join("forge.workspace.json")).unwrap());
    let (status, reuse) =
        server.json("POST", "/api/v1/effects/commits", Some("commit-policy-00002"), &commit);
    assert_eq!(status, 409, "{reuse}");
    assert_eq!(reuse["code"], "receipt-reused");
    let op_path = format!("/api/v1/operations/{}", operation["operation_id"].as_str().unwrap());
    assert_eq!(server.json("GET", &op_path, None, &json!({})).1, operation);
}

#[test]
fn registration_rechecks_unregistered_input_and_preserves_index_on_drift() {
    let server = Server::launch_mode(false, false);
    let (_, response) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some("register-policy-0001"),
        &json!({"role":"policy-source","path":"unregistered.md","key":"policy"}),
    );
    let preview = &response["preview"];
    std::fs::write(server.project.path().join("unregistered.md"), "changed after preview").unwrap();
    let (status,error)=server.json("POST","/api/v1/effects/commits",Some("commit-policy-00001"),&json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}));
    assert_eq!(status, 409, "{error}");
    assert!(!server.project.path().join("forge.workspace.json").exists());
}

fn register_and_commit(server: &Server, path: &str, role: &str, key: &str) {
    let (status, value) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some(&format!("register-{key}-0001")),
        &json!({"path":path,"role":role,"key":key}),
    );
    assert_eq!(status, 200, "register {path}: {value}");
    commit_preview(server, &value["preview"], &format!("commit-register-{key}"));
}
fn commit_preview(server: &Server, preview: &Value, key: &str) -> Value {
    let (status,value)=server.json("POST","/api/v1/effects/commits",Some(key),&json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}));
    assert_eq!(status, 202, "{value}");
    assert_eq!(value["state"], "succeeded");
    value
}

/// No JSON string in a portable artifact may carry a rooted path prefix.
fn assert_no_absolute_strings(value: &Value) {
    match value {
        Value::String(text) => {
            let bytes = text.as_bytes();
            let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
            assert!(
                !text.starts_with('/') && !text.starts_with("\\\\") && !drive,
                "portable artifact carries an absolute path: {text}"
            );
        }
        Value::Array(items) => items.iter().for_each(assert_no_absolute_strings),
        Value::Object(entries) => entries.values().for_each(assert_no_absolute_strings),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

#[test]
fn deterministic_conversion_reuses_domain_pipeline_and_preserves_portable_sources() {
    let left = Server::launch_mode(true, false);
    let right = Server::launch_mode(true, false);
    let convert = |server: &Server| {
        std::fs::write(
            server.project.path().join("policy.md"),
            "# Portable policy\n\n## Access\n\n- Operators must review every request.\n",
        )
        .unwrap();
        let (_, resources) = server.json("GET", "/api/v1/resources", None, &json!({}));
        let source = &resources["page"]["items"][0]["resource_id"];
        let (status,operation)=server.json("POST","/api/v1/conversions",Some("convert-policy-0001"),&json!({"source_resource_id":source,"output_kind":"oscal-catalog","target_path":"converted.json"}));
        assert_eq!(status, 202, "{operation}");
        commit_preview(server, &operation["result"]["preview"], "commit-conversion-0001");
        std::fs::read(server.project.path().join("converted.json")).unwrap()
    };
    let a = convert(&left);
    let b = convert(&right);
    assert_eq!(a, b);
    // Portable means portable on every platform: neither project root may appear
    // and no JSON string may carry a path prefix, not merely a macOS /private one.
    let text = String::from_utf8(a.clone()).unwrap();
    for project in [left.project.path(), right.project.path()] {
        let prefix = project.to_str().unwrap();
        assert!(!text.contains(prefix), "artifact leaks the project root {prefix}");
    }
    assert_no_absolute_strings(&serde_json::from_slice::<Value>(&a).unwrap());
}

#[test]
fn cancellation_route_rejects_completed_and_unknown_operations() {
    let server = Server::launch_mode(false, false);
    let listing = |server: &Server| {
        let mut names: Vec<String> = std::fs::read_dir(server.project.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    };
    let (_, response) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some("cancel-register-0001"),
        &json!({"role":"policy-source","path":"unregistered.md","key":"policy"}),
    );
    let completed = commit_preview(&server, &response["preview"], "cancel-commit-000001");
    assert_eq!(completed["state"], "succeeded");
    let id = completed["operation_id"].as_str().unwrap();
    let before = listing(&server);
    // A terminal operation is not cancellable and the rejection does not mutate it.
    let (status, rejected) =
        server.json("POST", &format!("/api/v1/operations/{id}/cancellation"), None, &json!({}));
    assert_eq!(status, 409, "{rejected}");
    assert_eq!(rejected["code"], "operation-not-cancellable");
    let (status, unchanged) =
        server.json("GET", &format!("/api/v1/operations/{id}"), None, &json!({}));
    assert_eq!(status, 200, "{unchanged}");
    assert_eq!(unchanged, completed);
    // An unknown operation id is a typed not-found, not a cancellation.
    let unknown = format!("/api/v1/operations/op_{}/cancellation", "0".repeat(64));
    let (status, rejected) = server.json("POST", &unknown, None, &json!({}));
    assert_eq!(status, 404, "{rejected}");
    assert_eq!(rejected["code"], "not-found");
    // Neither rejected cancellation created or removed a project file.
    assert_eq!(before, listing(&server));
}

#[test]
fn headless_scope_workflow_reconciles_counts_and_commits_only_explicit_decisions() {
    let server = Server::launch_mode(false, false);
    let framework = json!({"catalog":{"uuid":"22222222-2222-4222-8222-222222222222","metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":"control-a","title":"Synthetic A"},{"id":"control-b","title":"Synthetic B"}]}});
    std::fs::write(
        server.project.path().join("framework.json"),
        serde_json::to_vec(&framework).unwrap(),
    )
    .unwrap();
    register_and_commit(&server, "framework.json", "oscal-catalog-artifact", "framework");
    let (_, resources) = server.json("GET", "/api/v1/resources", None, &json!({}));
    let framework_id = &resources["page"]["items"][0]["resource_id"];
    let (status, initial) = server.json(
        "POST",
        "/api/v1/applicability/initializations",
        Some("initialize-scope-0001"),
        &json!({"framework_resource_id":framework_id,"target_path":"scope.json"}),
    );
    assert_eq!(status, 200, "{initial}");
    commit_preview(&server, &initial["preview"], "commit-initial-scope");
    register_and_commit(&server, "scope.json", "applicability-manifest", "scope");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(server.project.path().join("scope.json")).unwrap())
            .unwrap();
    let (status, controls) = server.json("GET", "/api/v1/applicability/controls", None, &json!({}));
    assert_eq!(status, 200, "{controls}");
    assert_eq!(controls["page"]["total_matching"], 2);
    assert!(
        controls["page"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["decision_state"] == "under-review")
    );
    let (_, counts) = server.json("GET", "/api/v1/review-queue/counts", None, &json!({}));
    let (_, queue) = server.json(
        "GET",
        "/api/v1/review-queue/items?reason_code=scope-decision-required",
        None,
        &json!({}),
    );
    assert_eq!(counts["total_open"], queue["page"]["total_matching"]);
    let (_, draft) = server.json("GET", "/api/v1/applicability/draft", None, &json!({}));
    let mut edited = manifest;
    edited["reviewers"] = json!([{"key":"reviewer","type":"person","name":"Synthetic Reviewer"}]);
    edited["decisions"] = json!([{"control_id":"control-a","state":"not-applicable","reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Synthetic scope exclusion."}]);
    let (status, preview) = server.json(
        "PUT",
        "/api/v1/applicability/draft",
        Some("edit-scope-0000001"),
        &json!({"manifest":edited,"observed_version":draft["version"]}),
    );
    assert_eq!(status, 200, "{preview}");
    commit_preview(&server, &preview["preview"], "commit-scope-000001");
    let (status, filtered) = server.json(
        "GET",
        "/api/v1/applicability/controls?decision_state=not-applicable",
        None,
        &json!({}),
    );
    assert_eq!(status, 200, "{filtered}");
    assert_eq!(filtered["page"]["total_matching"], 1);
    let (status, analysis) = server.json(
        "POST",
        "/api/v1/applicability/analyses",
        Some("analyze-scope-00001"),
        &json!({}),
    );
    assert_eq!(status, 202, "{analysis}");
    commit_preview(&server, &analysis["result"]["report_preview"], "commit-analysis-0001");
    register_and_commit(&server, "applicability-report.json", "applicability-report", "report");
    let cli = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(server.project.path())
        .args(["applicability", "analyze", "--manifest", "scope.json", "--format", "json"])
        .output()
        .unwrap();
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    assert_eq!(
        cli.stdout,
        std::fs::read(server.project.path().join("applicability-report.json")).unwrap()
    );
    let (status, report) = server.json("GET", "/api/v1/applicability/report", None, &json!({}));
    assert_eq!(status, 200, "{report}");
    assert_eq!(report["eligible_controls"], 2);
    assert_eq!(report["stale"], false);
    let before = std::fs::read(server.project.path().join("applicability-report.json")).unwrap();
    std::fs::write(server.project.path().join("second-report.json"), &before).unwrap();
    register_and_commit(&server, "second-report.json", "applicability-report", "second-report");
    let (status, ambiguous) = server.json(
        "POST",
        "/api/v1/applicability/analyses",
        Some("ambiguous-analysis-1"),
        &json!({}),
    );
    assert_eq!(status, 202, "{ambiguous}");
    assert_eq!(ambiguous["state"], "failed", "{ambiguous}");
    assert_eq!(ambiguous["error"]["code"], "validation-failed");
    assert_eq!(
        before,
        std::fs::read(server.project.path().join("applicability-report.json")).unwrap()
    );
    assert_eq!(before, std::fs::read(server.project.path().join("second-report.json")).unwrap());
}

#[test]
fn headless_mapping_initialization_build_export_and_download_are_separate_effects() {
    let server = Server::launch_mode(false, false);
    for (name, id, uuid) in [
        ("policy", "policy-a", "11111111-1111-4111-8111-111111111111"),
        ("framework", "framework-a", "22222222-2222-4222-8222-222222222222"),
    ] {
        let catalog = json!({"catalog":{"uuid":uuid,"metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":id,"title":"Synthetic control"},{"id":format!("{id}-unmapped"),"title":"Explicitly unmapped control"}]}});
        std::fs::write(
            server.project.path().join(format!("{name}.json")),
            serde_json::to_vec(&catalog).unwrap(),
        )
        .unwrap();
        register_and_commit(&server, &format!("{name}.json"), "oscal-catalog-artifact", name);
    }
    let (_, resources) = server.json("GET", "/api/v1/resources", None, &json!({}));
    let rows = resources["page"]["items"].as_array().unwrap();
    let find =
        |key: &str| rows.iter().find(|item| item["key"] == key).unwrap()["resource_id"].clone();
    let request = json!({"source_resource_id":find("policy"),"target_resource_id":find("framework"),"target_path":"mapping-manifest.json","scope":"control-only","maps":[{"key":"initial-none","relationship":"no-relationship","sources":[{"type":"control","id_ref":"policy-a"}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit initial review."}],"review":{
        "collection":{"key":"synthetic-map","title":"Synthetic mapping","version":"1","last_modified":"2026-09-10T00:00:00Z"},
        "reviewers":[{"key":"reviewer","type":"person","name":"PRIVATE REVIEWER NAME"}],
        "provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Explicit synthetic review.","reviewer_keys":["reviewer"],"reviewed_at":"2026-09-10T00:00:00Z"}}});
    let (status, initialized) = server.json(
        "POST",
        "/api/v1/mapping/initializations",
        Some("initialize-mapping-01"),
        &request,
    );
    assert_eq!(status, 200, "{initialized}");
    commit_preview(&server, &initialized["preview"], "commit-init-mapping-01");
    register_and_commit(&server, "mapping-manifest.json", "mapping-collection", "mapping");
    let (status, subjects) = server.json("GET", "/api/v1/mapping/subjects", None, &json!({}));
    assert_eq!(status, 200, "{subjects}");
    assert_eq!(subjects["page"]["total_matching"], 4);
    assert_unmapped_subject_provenance(&server, &subjects);
    let (_, draft) = server.json("GET", "/api/v1/mapping/draft", None, &json!({}));
    let mut manifest = draft["manifest"].clone();
    manifest["mapping"]["maps"] = json!([{"key":"explicit-none","relationship":"no-relationship","sources":[{"type":"control","id_ref":"policy-a"}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit absence of a relationship."}]);
    let (status, edited) = server.json(
        "PUT",
        "/api/v1/mapping/draft",
        Some("mapping-edit-0000001"),
        &json!({"manifest":manifest,"observed_version":draft["version"]}),
    );
    assert_eq!(status, 200, "{edited}");
    commit_preview(&server, &edited["preview"], "commit-map-edit-0001");
    let (status, built) =
        server.json("POST", "/api/v1/mapping/builds", Some("mapping-build-00001"), &json!({}));
    assert_eq!(status, 202, "{built}");
    assert_eq!(built["state"], "succeeded", "{built}");
    assert_eq!(built["result"]["no_relationship_count"], 1);
    assert_eq!(built["result"]["positive_relationship_count"], 0);
    commit_preview(&server, &built["result"]["report_preview"], "commit-map-build-0001");
    let cli = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(server.project.path())
        .args(["mapping", "build", "--manifest", "mapping-manifest.json"])
        .output()
        .unwrap();
    assert!(cli.status.success(), "{}", String::from_utf8_lossy(&cli.stderr));
    assert_eq!(
        cli.stdout,
        std::fs::read(server.project.path().join("mapping-collection.json")).unwrap()
    );
    let (status, export) = server.json(
        "POST",
        "/api/v1/exports",
        Some("mapping-export-0001"),
        &json!({"report_kind":"mapping-collection","target_path":"review.html"}),
    );
    assert_eq!(status, 202, "{export}");
    assert_eq!(export["state"], "succeeded", "{export}");
    let download = format!("/api/v1/exports/{}/download", export["operation_id"].as_str().unwrap());
    assert_eq!(server.request("GET", &download, true, None, "", "").0, 404);
    commit_preview(&server, &export["result"]["preview"], "commit-export-00001");
    let (status, headers, html) = server.request("GET", &download, true, None, "", "");
    assert_eq!(status, 200);
    assert!(headers.contains("attachment; filename=forge-review-report.html"));
    let html = String::from_utf8(html).unwrap();
    assert!(!html.contains("PRIVATE REVIEWER NAME"));
    assert!(!html.contains("<script"));
    std::fs::write(server.project.path().join("review.html"), "tampered").unwrap();
    assert_eq!(server.request("GET", &download, true, None, "", "").0, 409);
    assert_ambiguous_mapping_is_not_ready(&server);
}

#[test]
fn trace_drilldown_uses_exact_registered_sources_and_hash_bound_reports() {
    let server = Server::launch_mode(true, false);
    std::fs::write(
        server.project.path().join("policy.md"),
        "# Synthetic policy\n\n## Scope\n\n- The operator must review this supplied clause.\n",
    )
    .unwrap();
    let (_, resources) = server.json("GET", "/api/v1/resources", None, &json!({}));
    let source = &resources["page"]["items"][0]["resource_id"];
    let (status,op)=server.json("POST","/api/v1/conversions",Some("trace-convert-0001"),&json!({"source_resource_id":source,"output_kind":"oscal-catalog","target_path":"converted.json"}));
    assert_eq!(status, 202, "{op}");
    commit_preview(&server, &op["result"]["preview"], "trace-convert-commit");
    register_and_commit(&server, "converted.json", "oscal-catalog-artifact", "converted");
    let (_, resources) = server.json("GET", "/api/v1/resources", None, &json!({}));
    let artifact = resources["page"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["key"] == "converted")
        .unwrap()["resource_id"]
        .as_str()
        .unwrap();
    let (status, graph) = server.json(
        "GET",
        &format!("/api/v1/provenance/entries?anchor={artifact}"),
        None,
        &json!({}),
    );
    assert_eq!(status, 200, "{graph}");
    let entries = graph["page"]["items"].as_array().unwrap();
    let trace = entries
        .iter()
        .find(|entry| entry["label"].as_str().unwrap().starts_with("Asserted trace"))
        .unwrap_or_else(|| {
            panic!(
                "shared walker trace missing: {graph}; artifact: {}",
                std::fs::read_to_string(server.project.path().join("converted.json")).unwrap()
            )
        });
    assert!(trace["label"].as_str().unwrap().contains("original source hash is not supplied"));
    let excerpt = entries
        .iter()
        .flat_map(|entry| entry["excerpt_refs"].as_array().into_iter().flatten())
        .next()
        .unwrap()
        .as_str()
        .unwrap();
    let (_, value) =
        server.json("GET", &format!("/api/v1/provenance/excerpts/{excerpt}"), None, &json!({}));
    assert_eq!(
        value["sha256"],
        resources["page"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["key"] == "policy")
            .unwrap()["sha256"]
    );
    let (status, op) = server.json(
        "POST",
        "/api/v1/exports",
        Some("trace-export-0001"),
        &json!({"report_kind":"trace","target_path":"trace.html"}),
    );
    assert_eq!(status, 202, "{op}");
    commit_preview(&server, &op["result"]["preview"], "trace-export-commit");
    register_and_commit(&server, "trace.html", "trace-report", "trace");
    let (_, before) = server.json("GET", "/api/v1/resources", None, &json!({}));
    assert!(
        before["page"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["key"] == "trace" && item["validation_state"] == "valid")
    );
    std::fs::write(
        server.project.path().join("policy.md"),
        "# Changed source\n\nDifferent captured bytes.\n",
    )
    .unwrap();
    let (_, after) = server.json("GET", "/api/v1/resources", None, &json!({}));
    assert!(
        after["page"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["key"] == "trace" && item["stale"] == true)
    );
}

#[test]
fn hostile_requests_are_bounded_and_do_not_reflect_private_values() {
    let server = Server::launch_mode(true, false);
    for (body, status) in [
        (r#"{"scope":"all","scope":"selected"}"#.to_owned(), 400),
        (r#"{"scope":"all","unknown":"PRIVATE SECRET VALUE"}"#.to_owned(), 400),
        (format!("{{\"scope\":\"{}\"}}", "x".repeat(70000)), 400),
        ("[".repeat(100) + &"]".repeat(100), 400),
    ] {
        let (actual, headers, bytes) = server.request(
            "POST",
            "/api/v1/validation/runs",
            true,
            None,
            "Content-Type: application/json\r\n",
            &body,
        );
        assert_eq!(actual, status);
        assert!(headers.contains("cache-control: no-store"));
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("PRIVATE SECRET VALUE"));
        assert!(!text.contains(server.project.path().to_str().unwrap()));
    }
    for headers in [
        "Origin: https://attacker.invalid\r\nSec-Fetch-Site: cross-site\r\n",
        "Origin: http://127.0.0.1:1\r\nSec-Fetch-Site: same-origin\r\n",
        "Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-Dest: document\r\n",
        "Forwarded: host=attacker.invalid\r\n",
    ] {
        assert_eq!(server.request("GET", "/api/v1/resources", true, None, headers, "").0, 401);
    }
    // Lexical traversal never reaches containment: the request contract rejects
    // it before path handling, and the rejection reflects no private value.
    let (status, error) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some("hostile-path-0001"),
        &json!({"role":"policy-source","path":"../PRIVATE-SECRET.md"}),
    );
    assert_eq!(status, 400, "{error}");
    assert_eq!(error["code"], "invalid-request", "{error}");
    assert!(!error.to_string().contains("PRIVATE-SECRET"));
    // A portable-path alias passes the request pattern but is refused by the
    // containment check, whose rejection also reflects no private value.
    let (status, error) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some("hostile-path-0002"),
        &json!({"role":"policy-source","path":"PRIVATE-SECRET."}),
    );
    assert_eq!(status, 403, "{error}");
    assert_eq!(error["code"], "resource-containment", "{error}");
    assert!(!error.to_string().contains("PRIVATE-SECRET"));
}

#[test]
fn response_loss_and_process_interruption_leave_complete_or_absent_outputs() {
    use base64::Engine as _;
    for delay in [0, 2, 20] {
        let mut server = Server::launch_mode(false, false);
        let source = format!("# Synthetic\n\n{}", "- Supplied test clause.\n".repeat(20000));
        let (status,preview)=server.json("POST","/api/v1/resources/upload",Some("interrupt-upload-0001"),&json!({"role":"policy-source","target_path":"output.md","filename":"output.md","content_base64":base64::engine::general_purpose::STANDARD.encode(source.as_bytes())}));
        assert_eq!(status, 200, "{preview}");
        let preview = &preview["preview"];
        let body=json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}).to_string();
        let mut stream = TcpStream::connect(&server.host).unwrap();
        write!(stream,"POST /api/v1/effects/commits HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nIdempotency-Key: interrupt-commit-001\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",server.host,server.capability,body.len(),body).unwrap();
        stream.flush().unwrap();
        // Deliberately abandon the response. The final iteration verifies replay;
        // earlier iterations kill the process at different points in publication.
        std::thread::sleep(Duration::from_millis(delay));
        if delay == 20 {
            let (status, op) = server.json(
                "POST",
                "/api/v1/effects/commits",
                Some("interrupt-commit-001"),
                &serde_json::from_str(&body).unwrap(),
            );
            assert_eq!(status, 202, "{op}");
            assert_eq!(op["state"], "succeeded");
        }
        server.process.kill().unwrap();
        server.process.wait().unwrap();
        drop(stream);
        if let Ok(bytes) = std::fs::read(server.project.path().join("output.md")) {
            assert_eq!(bytes, source.as_bytes());
        }
    }
}

fn assert_unmapped_subject_provenance(server: &Server, subjects: &Value) {
    let (status, queue) = server.json(
        "GET",
        "/api/v1/review-queue/items?reason_code=no-reviewed-mapping",
        None,
        &json!({}),
    );
    assert_eq!(status, 200, "{queue}");
    assert_eq!(queue["page"]["total_matching"], 2, "{queue}");
    for item in queue["page"]["items"].as_array().unwrap() {
        let subject = subjects["page"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|subject| subject["provenance_ref"] == item["evidence_refs"][0])
            .unwrap();
        assert_eq!(subject["resource_id"], item["resource_id"]);
        assert!(item["summary"].as_str().unwrap().contains(subject["label"].as_str().unwrap()));
        let anchor = item["evidence_refs"][0].as_str().unwrap();
        let (status, graph) = server.json(
            "GET",
            &format!("/api/v1/provenance/entries?anchor={anchor}"),
            None,
            &json!({}),
        );
        assert_eq!(status, 200, "{graph}");
        assert!(
            graph["page"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["entry_id"] == anchor)
        );
    }
}

fn assert_ambiguous_mapping_is_not_ready(server: &Server) {
    std::fs::copy(
        server.project.path().join("mapping-manifest.json"),
        server.project.path().join("other-mapping.json"),
    )
    .unwrap();
    register_and_commit(server, "other-mapping.json", "mapping-collection", "other-mapping");
    let (status, summary) = server.json("GET", "/api/v1/project/summary", None, &json!({}));
    assert_eq!(status, 200, "{summary}");
    assert_eq!(summary["health"], "needs-attention");
    assert_eq!(summary["resource_counts"]["invalid"], 2);
    let (status, queue) = server.json(
        "GET",
        "/api/v1/review-queue/items?reason_code=invalid-resource",
        None,
        &json!({}),
    );
    assert_eq!(status, 200, "{queue}");
    assert_eq!(queue["page"]["total_matching"], 2);
}

/// Ordered registration fields used to independently reproduce the documented index encoding.
#[derive(serde::Serialize)]
struct BundleFixtureResource<'a> {
    /// Explicit authorial registration key, never inferred from the filename.
    key: &'a str,
    /// Existing closed workspace role spelling.
    role: &'a str,
    /// Portable project-relative registration path.
    path: &'a str,
}

/// Index field order is part of the normalized pretty-JSON-plus-newline hash contract.
#[derive(serde::Serialize)]
struct BundleFixtureIndex<'a> {
    /// Existing index schema, independent of the bundle family version.
    schema_version: &'a str,
    /// Author-supplied project label preserved as metadata.
    label: &'a str,
    /// Registrations remain in authorial array order.
    resources: Vec<BundleFixtureResource<'a>>,
}

/// Hash supplied fixture bytes independently of the workspace's private digest helper.
fn bundle_fixture_sha256(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    let mut encoded = String::with_capacity(64);
    for byte in sha2::Sha256::digest(bytes) {
        write!(encoded, "{byte:02x}").expect("format synthetic SHA256");
    }
    encoded
}

/// Reproduce `Index::bytes` field order and final newline without calling private production code.
fn bundle_fixture_index_bytes(index: &Value) -> Vec<u8> {
    let resources = index["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| BundleFixtureResource {
            key: resource["key"].as_str().unwrap(),
            role: resource["role"].as_str().unwrap(),
            path: resource["path"].as_str().unwrap(),
        })
        .collect();
    let ordered = BundleFixtureIndex {
        schema_version: index["schema_version"].as_str().unwrap(),
        label: index["label"].as_str().unwrap(),
        resources,
    };
    let mut bytes = serde_json::to_vec_pretty(&ordered).unwrap();
    bytes.push(b'\n');
    bytes
}

/// Construct a closed metadata-only bundle with one original-byte pin per authorial entry.
fn bundle_fixture(index: &Value, source_bytes: &[&[u8]]) -> Value {
    let registrations = index["resources"].as_array().unwrap();
    assert_eq!(registrations.len(), source_bytes.len());
    let pins: Vec<Value> = registrations
        .iter()
        .zip(source_bytes)
        .map(|(registration, bytes)| {
            json!({"key":registration["key"],"sha256":bundle_fixture_sha256(bytes),"size_bytes":bytes.len()})
        })
        .collect();
    json!({
        "schema_version":"forge.workspace-index-bundle/1",
        "content_profile":"index-and-hashes",
        "index":index,
        "index_sha256":bundle_fixture_sha256(&bundle_fixture_index_bytes(index)),
        "pins":pins,
    })
}

/// Compare complete closed response fields, excluding effect, operation and receipt placeholders.
fn bundle_assert_keys(value: &Value, expected: &[&str]) {
    let mut actual: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
    let mut expected = expected.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected, "unexpected fields: {value}");
}

/// Require a bounded safe typed error without reflecting source content, credentials or root paths.
fn bundle_assert_error(server: &Server, value: &Value, code: &str) {
    bundle_assert_keys(value, &["code", "message", "retryable"]);
    assert_eq!(value["code"], code, "{value}");
    let text = value.to_string();
    for private in [
        "PRIVATE UNREGISTERED CONTENT",
        "PRIVATE BUNDLE SOURCE BYTES",
        "PRIVATE REJECTED VALUE",
        "PRIVATE-REJECTED-VALUE",
        server.project.path().to_str().unwrap(),
        server.capability.as_str(),
    ] {
        assert!(!text.contains(private), "private value reflected: {value}");
    }
}

/// Fetch the actual authenticated preview and check the fixed metadata disclosure contract.
fn bundle_preview(server: &Server) -> Value {
    let (status, headers, bytes) =
        server.request("GET", "/api/v1/project/bundle-preview", true, None, "", "");
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&bytes));
    assert!(headers.contains("cache-control: no-store"));
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    bundle_assert_keys(
        &value,
        &[
            "bundle",
            "source_index_present",
            "snapshot_version",
            "included_metadata",
            "source_content_included",
        ],
    );
    assert_eq!(value["source_index_present"], true);
    assert_eq!(value["source_content_included"], false);
    assert_eq!(
        value["included_metadata"],
        json!([
            "project-label",
            "resource-keys",
            "typed-roles",
            "project-relative-paths",
            "sha256-fingerprints",
            "byte-lengths",
        ])
    );
    assert_eq!(value["snapshot_version"].as_str().unwrap().len(), 64);
    bundle_assert_keys(
        &value["bundle"],
        &["schema_version", "content_profile", "index", "index_sha256", "pins"],
    );
    for pin in value["bundle"]["pins"].as_array().unwrap() {
        bundle_assert_keys(pin, &["key", "sha256", "size_bytes"]);
    }
    value
}

/// Verify through the actual read-only POST route and require a complete non-effect response.
fn bundle_verify(server: &Server, bundle: &Value) -> Value {
    let (status, value) = server.json(
        "POST",
        "/api/v1/project/bundle-verifications",
        None,
        &json!({"bundle":bundle}),
    );
    assert_eq!(status, 200, "{value}");
    bundle_assert_keys(
        &value,
        &[
            "scope",
            "snapshot_version",
            "source_index_present",
            "state",
            "current_resources",
            "current_only_resources",
            "expected_index_matches_current",
            "expected_resources",
            "matched_resources",
            "unregistered_resources",
            "mismatched_resources",
            "items",
            "source_content_included",
        ],
    );
    assert_eq!(value["scope"], "registered-fingerprints-only");
    assert_eq!(value["source_content_included"], false);
    for row in value["items"].as_array().unwrap() {
        bundle_assert_keys(
            row,
            &["key", "status", "reason_codes", "observed_resource_validation_state"],
        );
    }
    value
}

/// Assert exact expected/observed denominators; extra registrations remain separately visible.
fn bundle_assert_counts(value: &Value, counts: [usize; 6]) {
    for (field, expected) in [
        "expected_resources",
        "matched_resources",
        "unregistered_resources",
        "mismatched_resources",
        "current_resources",
        "current_only_resources",
    ]
    .into_iter()
    .zip(counts)
    {
        assert_eq!(value[field], json!(expected), "{field}: {value}");
    }
    assert_eq!(value["items"].as_array().unwrap().len(), counts[0]);
    assert_eq!(counts[1] + counts[2] + counts[3], counts[0]);
}

/// Capture fixture files and top-level entries to detect query publication or source-byte changes.
fn bundle_project_files(server: &Server) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<_> = std::fs::read_dir(server.project.path())
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            (entry.file_name().into_string().unwrap(), std::fs::read(entry.path()).unwrap())
        })
        .collect();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

/// Metadata preview and comparison use all registrations without publishing files or source excerpts.
#[test]
fn readonly_bundle_preview_and_verification_preserve_registered_bytes() {
    let server = Server::launch_mode(false, true);
    let first = b"# Private source\n\nPRIVATE BUNDLE SOURCE BYTES\n";
    let second = b"# Other source\r\n\r\nA supplied test clause.\r\n";
    let index = json!({"schema_version":"forge.workspace/1","label":"Authorial order","resources":[
        {"key":"z-first","role":"policy-source","path":"first.md"},
        {"key":"a-second","role":"policy-source","path":"second.md"},
    ]});
    std::fs::write(server.project.path().join("first.md"), first).unwrap();
    std::fs::write(server.project.path().join("second.md"), second).unwrap();
    let raw_index = serde_json::to_vec(&index).unwrap();
    std::fs::write(server.project.path().join("forge.workspace.json"), &raw_index).unwrap();
    let before = bundle_project_files(&server);
    let preview = bundle_preview(&server);
    assert_eq!(preview["bundle"], bundle_fixture(&index, &[first, second]));
    assert_ne!(preview["bundle"]["index_sha256"], bundle_fixture_sha256(&raw_index));
    assert!(!preview.to_string().contains("PRIVATE BUNDLE SOURCE BYTES"));
    assert!(!preview.to_string().contains("PRIVATE UNREGISTERED CONTENT"));
    let verification = bundle_verify(&server, &preview["bundle"]);
    assert_eq!(verification["state"], "matched");
    assert_eq!(verification["source_index_present"], true);
    assert_eq!(verification["expected_index_matches_current"], true);
    assert_eq!(verification["snapshot_version"], preview["snapshot_version"]);
    bundle_assert_counts(&verification, [2, 2, 0, 0, 2, 0]);
    assert_eq!(
        verification["items"],
        json!([
            {"key":"z-first","status":"matched","reason_codes":[],"observed_resource_validation_state":"valid"},
            {"key":"a-second","status":"matched","reason_codes":[],"observed_resource_validation_state":"valid"},
        ])
    );
    assert!(!verification.to_string().contains("PRIVATE BUNDLE SOURCE BYTES"));
    let mut reordered = preview["bundle"].clone();
    reordered["index"]["resources"].as_array_mut().unwrap().swap(0, 1);
    reordered["pins"].as_array_mut().unwrap().swap(0, 1);
    reordered["index_sha256"] =
        json!(bundle_fixture_sha256(&bundle_fixture_index_bytes(&reordered["index"])));
    let order_only = bundle_verify(&server, &reordered);
    assert_eq!(order_only["state"], "matched");
    assert_eq!(order_only["expected_index_matches_current"], false);
    bundle_assert_counts(&order_only, [2, 2, 0, 0, 2, 0]);
    assert_eq!(order_only["items"][0]["key"], "a-second");
    let mut relabeled = preview["bundle"].clone();
    relabeled["index"]["label"] = json!("Different authorial label");
    relabeled["index_sha256"] =
        json!(bundle_fixture_sha256(&bundle_fixture_index_bytes(&relabeled["index"])));
    let label_only = bundle_verify(&server, &relabeled);
    assert_eq!(label_only["state"], "matched");
    assert_eq!(label_only["expected_index_matches_current"], false);
    bundle_assert_counts(&label_only, [2, 2, 0, 0, 2, 0]);
    // A retry is another comparison, not an idempotent effect or persisted replay.
    assert_eq!(bundle_verify(&server, &preview["bundle"]), verification);
    assert_eq!(bundle_project_files(&server), before);
}

/// Absent setup never becomes an invented empty bundle; an authored empty index has a zero denominator.
#[test]
fn bundle_queries_distinguish_missing_and_explicit_empty_indexes() {
    let server = Server::launch_mode(false, true);
    let index =
        json!({"schema_version":"forge.workspace/1","label":"Explicit empty","resources":[]});
    let empty = bundle_fixture(&index, &[]);
    let (status, error) = server.json("GET", "/api/v1/project/bundle-preview", None, &json!({}));
    assert_eq!(status, 404, "{error}");
    bundle_assert_error(&server, &error, "not-found");
    let missing = bundle_verify(&server, &empty);
    assert_eq!(missing["state"], "missing-index");
    assert_eq!(missing["source_index_present"], false);
    assert_eq!(missing["expected_index_matches_current"], false);
    bundle_assert_counts(&missing, [0, 0, 0, 0, 0, 0]);
    let nonempty_index = json!({"schema_version":"forge.workspace/1","label":"Expected only","resources":[
        {"key":"not-registered","role":"policy-source","path":"unregistered.md"},
    ]});
    let missing_nonempty =
        bundle_verify(&server, &bundle_fixture(&nonempty_index, &[b"Expected bytes"]));
    assert_eq!(missing_nonempty["state"], "missing-index");
    bundle_assert_counts(&missing_nonempty, [1, 0, 1, 0, 0, 0]);
    assert_eq!(
        missing_nonempty["items"],
        json!([
            {"key":"not-registered","status":"not-registered","reason_codes":["registration-not-found"],"observed_resource_validation_state":"not-registered"},
        ])
    );
    assert!(!server.project.path().join("forge.workspace.json").exists());
    std::fs::write(
        server.project.path().join("forge.workspace.json"),
        serde_json::to_vec(&index).unwrap(),
    )
    .unwrap();
    let before = bundle_project_files(&server);
    let preview = bundle_preview(&server);
    assert_eq!(preview["bundle"], empty);
    let matched = bundle_verify(&server, &empty);
    assert_eq!(matched["state"], "matched");
    assert_eq!(matched["source_index_present"], true);
    assert_eq!(matched["expected_index_matches_current"], true);
    bundle_assert_counts(&matched, [0, 0, 0, 0, 0, 0]);
    assert_eq!(bundle_project_files(&server), before);
}

/// Every supplied row receives an ordered result; unknown directories are not opened as resources.
#[test]
fn bundle_verification_reconciles_mixed_rows_and_registration_conflicts() {
    let server = Server::launch_mode(false, true);
    let bytes = b"# Registered source\n\nA supplied clause.\n";
    let current = json!({"schema_version":"forge.workspace/1","label":"Current label","resources":[
        {"key":"first","role":"policy-source","path":"first.md"},
        {"key":"second","role":"policy-source","path":"second.md"},
        {"key":"conflict","role":"policy-source","path":"conflict.md"},
        {"key":"extra","role":"policy-source","path":"extra.md"},
    ]});
    for path in ["first.md", "second.md", "conflict.md", "extra.md"] {
        std::fs::write(server.project.path().join(path), bytes).unwrap();
    }
    std::fs::write(
        server.project.path().join("forge.workspace.json"),
        serde_json::to_vec(&current).unwrap(),
    )
    .unwrap();
    let index_before = std::fs::read(server.project.path().join("forge.workspace.json")).unwrap();
    let directory = server.project.path().join("private-directory.md");
    std::fs::create_dir(&directory).unwrap();
    let sentinel = directory.join("private.txt");
    std::fs::write(&sentinel, b"PRIVATE BUNDLE SOURCE BYTES").unwrap();
    let expected = json!({"schema_version":"forge.workspace/1","label":"Different expected label","resources":[
        {"key":"second","role":"policy-source","path":"second.md"},
        {"key":"directory","role":"policy-source","path":"private-directory.md"},
        {"key":"first","role":"policy-source","path":"first.md"},
        {"key":"conflict","role":"policy-source","path":"other.md"},
        {"key":"sentinel","role":"policy-source","path":"unregistered.md"},
    ]});
    let mut bundle = bundle_fixture(&expected, &[bytes, bytes, bytes, bytes, bytes]);
    bundle["pins"][0]["sha256"] = json!("0".repeat(64));
    bundle["pins"][0]["size_bytes"] = json!(bytes.len() + 1);
    let result = bundle_verify(&server, &bundle);
    assert_eq!(result["state"], "mismatched");
    assert_eq!(result["expected_index_matches_current"], false);
    bundle_assert_counts(&result, [5, 1, 2, 2, 4, 1]);
    assert_eq!(
        result["items"],
        json!([
            {"key":"second","status":"mismatched","reason_codes":["sha256-mismatch","size-mismatch"],"observed_resource_validation_state":"valid"},
            {"key":"directory","status":"not-registered","reason_codes":["registration-not-found"],"observed_resource_validation_state":"not-registered"},
            {"key":"first","status":"matched","reason_codes":[],"observed_resource_validation_state":"valid"},
            {"key":"conflict","status":"mismatched","reason_codes":["registration-conflict"],"observed_resource_validation_state":"valid"},
            {"key":"sentinel","status":"not-registered","reason_codes":["registration-not-found"],"observed_resource_validation_state":"not-registered"},
        ])
    );
    let mut hash_only = bundle.clone();
    hash_only["pins"][0]["size_bytes"] = json!(bytes.len());
    assert_eq!(
        bundle_verify(&server, &hash_only)["items"][0]["reason_codes"],
        json!(["sha256-mismatch"])
    );
    let mut size_only = bundle.clone();
    size_only["pins"][0]["sha256"] = json!(bundle_fixture_sha256(bytes));
    assert_eq!(
        bundle_verify(&server, &size_only)["items"][0]["reason_codes"],
        json!(["size-mismatch"])
    );
    let alias_index = json!({"schema_version":"forge.workspace/1","label":"Supplied key is not registered","resources":[
        {"key":"unknown-alias","role":"policy-source","path":"first.md"},
    ]});
    let alias = bundle_verify(&server, &bundle_fixture(&alias_index, &[bytes]));
    bundle_assert_counts(&alias, [1, 0, 1, 0, 4, 4]);
    assert_eq!(alias["items"][0]["status"], "not-registered");
    assert_eq!(alias["items"][0]["reason_codes"], json!(["registration-not-found"]));
    let mut role_conflict = bundle.clone();
    role_conflict["index"]["resources"][3]["path"] = json!("conflict.md");
    role_conflict["index"]["resources"][3]["role"] = json!("oscal-catalog-artifact");
    role_conflict["index_sha256"] =
        json!(bundle_fixture_sha256(&bundle_fixture_index_bytes(&role_conflict["index"])));
    let changed_role = bundle_verify(&server, &role_conflict);
    assert_eq!(changed_role["items"][3], result["items"][3]);
    let subset_index = json!({"schema_version":"forge.workspace/1","label":"Current label","resources":[
        {"key":"first","role":"policy-source","path":"first.md"},
    ]});
    let subset = bundle_verify(&server, &bundle_fixture(&subset_index, &[bytes]));
    assert_eq!(subset["state"], "matched");
    assert_eq!(subset["expected_index_matches_current"], false);
    bundle_assert_counts(&subset, [1, 1, 0, 0, 4, 3]);
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"PRIVATE BUNDLE SOURCE BYTES");
    assert_eq!(
        std::fs::read(server.project.path().join("unregistered.md")).unwrap(),
        b"PRIVATE UNREGISTERED CONTENT"
    );
    assert_eq!(
        std::fs::read(server.project.path().join("forge.workspace.json")).unwrap(),
        index_before
    );
    for path in ["first.md", "second.md", "conflict.md", "extra.md"] {
        assert_eq!(std::fs::read(server.project.path().join(path)).unwrap(), bytes);
    }
    assert!(!result.to_string().contains("PRIVATE BUNDLE SOURCE BYTES"));
    assert!(!result.to_string().contains("PRIVATE UNREGISTERED CONTENT"));
    // Successful verification of the directory row specifically rules out opening
    // that unregistered nonregular path as a captured file; registered reads remain expected.
}

/// Byte agreement remains informational when current role validation is invalid or historical/stale.
#[test]
fn bundle_fingerprint_agreement_retains_invalid_and_stale_resource_states() {
    let server = Server::launch_mode(false, true);
    let invalid = b" \n\t ";
    let historical = json!({
        "schema_version":"forge.applicability-report/1","manifest_sha256":"a".repeat(64),
        "framework":{"resource_type":"catalog","href":"framework.json","raw_sha256":"b".repeat(64),"root_uuid":"22222222-2222-4222-8222-222222222222","document_version":"1","oscal_version":"1.2.3"},
        "mapping_collections":[],"reviewers":[],
        "counts":{"total":0,"applicable_mapped":0,"applicable_reviewed_no_relationship":0,"applicable_unmapped":0,"not_applicable":0,"deferred":0,"under_review":0},
        "filters":{},"matched_controls":0,"controls":[],"review_queue":[],
    });
    let report_bytes = serde_json::to_vec(&historical).unwrap();
    let index = json!({"schema_version":"forge.workspace/1","label":"Validation remains separate","resources":[
        {"key":"invalid","role":"policy-source","path":"invalid.md"},
        {"key":"historical","role":"applicability-report","path":"historical.json"},
    ]});
    std::fs::write(server.project.path().join("invalid.md"), invalid).unwrap();
    std::fs::write(server.project.path().join("historical.json"), &report_bytes).unwrap();
    std::fs::write(
        server.project.path().join("forge.workspace.json"),
        serde_json::to_vec(&index).unwrap(),
    )
    .unwrap();
    let before = bundle_project_files(&server);
    let (status, resources) = server.json("GET", "/api/v1/resources", None, &json!({}));
    assert_eq!(status, 200, "{resources}");
    let observed = resources["page"]["items"].as_array().unwrap();
    let invalid_row = observed.iter().find(|row| row["key"] == "invalid").unwrap();
    let historical_row = observed.iter().find(|row| row["key"] == "historical").unwrap();
    assert_eq!(invalid_row["validation_state"], "invalid");
    assert_eq!(historical_row["validation_state"], "stale");
    let preview = bundle_preview(&server);
    assert_eq!(preview["bundle"], bundle_fixture(&index, &[invalid, &report_bytes]));
    let result = bundle_verify(&server, &preview["bundle"]);
    assert_eq!(result["state"], "matched");
    bundle_assert_counts(&result, [2, 2, 0, 0, 2, 0]);
    assert_eq!(
        result["items"],
        json!([
            {"key":"invalid","status":"matched","reason_codes":[],"observed_resource_validation_state":"invalid"},
            {"key":"historical","status":"matched","reason_codes":[],"observed_resource_validation_state":"stale"},
        ])
    );
    assert_eq!(bundle_project_files(&server), before);
}

/// Exercise literal and decoded duplicate keys through the actual raw HTTP decoder.
fn bundle_assert_raw_duplicates_rejected(server: &Server, valid: &Value, pin_hash: &str) {
    let text = valid.to_string();
    let hash_property = format!("\"sha256\":\"{pin_hash}\"");
    let duplicate_pin =
        text.replacen(&hash_property, &format!("{hash_property},\"sha256\":\"{pin_hash}\""), 1);
    let decoded_pin = text.replacen(
        &hash_property,
        &format!("{hash_property},\"sha\\u003256\":\"{pin_hash}\""),
        1,
    );
    assert_ne!(duplicate_pin, text);
    assert_ne!(decoded_pin, text);
    for body in [
        format!("{{\"bundle\":{},\"bundle\":{}}}", valid["bundle"], valid["bundle"]),
        format!("{{\"bundle\":{},\"bund\\u006ce\":{}}}", valid["bundle"], valid["bundle"]),
        duplicate_pin,
        decoded_pin,
    ] {
        let (status, _, response) = server.request(
            "POST",
            "/api/v1/project/bundle-verifications",
            true,
            None,
            "Content-Type: application/json\r\n",
            &body,
        );
        assert_eq!(status, 400, "{}", String::from_utf8_lossy(&response));
        bundle_assert_error(server, &serde_json::from_slice(&response).unwrap(), "invalid-request");
    }
}

/// Intrinsic corruption and duplicate decoded keys reject before any supplied path can become authority.
#[test]
fn bundle_verification_rejects_closed_contract_and_duplicate_encodings() {
    let server = Server::launch_mode(false, true);
    let bytes = b"# Source\n\nA supplied clause.\n";
    let index = json!({"schema_version":"forge.workspace/1","label":"Closed fixture","resources":[
        {"key":"alpha","role":"policy-source","path":"alpha.md"},
        {"key":"beta","role":"policy-source","path":"beta.md"},
    ]});
    for path in ["alpha.md", "beta.md"] {
        std::fs::write(server.project.path().join(path), bytes).unwrap();
    }
    std::fs::write(
        server.project.path().join("forge.workspace.json"),
        serde_json::to_vec(&index).unwrap(),
    )
    .unwrap();
    let before = bundle_project_files(&server);
    let bundle = bundle_fixture(&index, &[bytes, bytes]);
    assert_eq!(bundle_preview(&server)["bundle"], bundle);
    let valid = json!({"bundle":bundle});
    let mut invalid = Vec::new();
    for (pointer, replacement) in [
        ("/content", json!("PRIVATE REJECTED VALUE")),
        ("/bundle/content", json!("PRIVATE REJECTED VALUE")),
        ("/bundle/approval", json!(true)),
        ("/bundle/index/approval", json!(true)),
        ("/bundle/index/resources/0/content", json!("PRIVATE REJECTED VALUE")),
        ("/bundle/pins/0/path", json!("PRIVATE REJECTED VALUE")),
        ("/bundle/schema_version", json!("forge.workspace-index-bundle/2")),
        ("/bundle/content_profile", json!("source-inclusive")),
        ("/bundle/index/schema_version", json!("forge.workspace/2")),
        ("/bundle/index_sha256", json!("A".repeat(64))),
        ("/bundle/index_sha256", json!("0".repeat(64))),
        ("/bundle/pins/0/sha256", json!("not-a-hash")),
        ("/bundle/pins/0/size_bytes", json!(-1)),
        ("/bundle/pins/0/size_bytes", json!(10 * 1024 * 1024 + 1)),
        ("/bundle/pins/0/size_bytes", json!(1.5)),
        ("/bundle/pins/0/key", json!("other-key")),
        ("/bundle/index/resources/0/path", json!("../PRIVATE-REJECTED-VALUE.md")),
    ] {
        let mut body = valid.clone();
        let parts: Vec<&str> = pointer.trim_start_matches('/').split('/').collect();
        let mut target = &mut body;
        for part in &parts[..parts.len() - 1] {
            target = if let Ok(index) = part.parse::<usize>() {
                &mut target[index]
            } else {
                &mut target[*part]
            };
        }
        target[parts[parts.len() - 1]] = replacement;
        invalid.push((pointer.to_owned(), body));
    }
    let mut removed = valid.clone();
    removed["bundle"]["pins"].as_array_mut().unwrap().pop();
    invalid.push(("missing pin".into(), removed));
    let mut swapped = valid.clone();
    swapped["bundle"]["pins"].as_array_mut().unwrap().swap(0, 1);
    invalid.push(("pin order".into(), swapped));
    let mut duplicate = valid.clone();
    duplicate["bundle"]["pins"][1] = duplicate["bundle"]["pins"][0].clone();
    invalid.push(("duplicate pin key".into(), duplicate));
    for (case, body) in invalid {
        let (status, error) =
            server.json("POST", "/api/v1/project/bundle-verifications", None, &body);
        assert_eq!(status, 400, "{case}: {error}");
        bundle_assert_error(&server, &error, "invalid-request");
    }
    bundle_assert_raw_duplicates_rejected(&server, &valid, &bundle_fixture_sha256(bytes));
    assert_eq!(bundle_verify(&server, &valid["bundle"])["state"], "matched");
    assert_eq!(bundle_project_files(&server), before);
}

/// The complete POST envelope keeps the existing byte ceiling and duplicate-safe decoded bounds.
#[test]
fn bundle_request_envelope_obeys_existing_raw_and_decoded_limits() {
    let server = Server::launch(true);
    let preview = bundle_preview(&server);
    let body = json!({"bundle":preview["bundle"]}).to_string();
    let limit = 1024 * 1024;
    let padded = format!("{}{body}", " ".repeat(limit - body.len()));
    assert_eq!(padded.len(), limit);
    let (status, _, bytes) = server.request(
        "POST",
        "/api/v1/project/bundle-verifications",
        true,
        None,
        "Content-Type: application/json\r\n",
        &padded,
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&bytes));
    assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap()["state"], "matched");
    // Send only the oversized declared headers: the server must reject before
    // waiting for the absent body, rather than relying on a client write/reset race.
    let mut stream = TcpStream::connect(&server.host).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    write!(stream, "POST /api/v1/project/bundle-verifications HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", server.host, server.capability, limit + 1).unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (headers, raw) = response.split_once("\r\n\r\n").unwrap();
    assert_eq!(headers.split_whitespace().nth(1).unwrap(), "413");
    bundle_assert_error(&server, &serde_json::from_str(raw).unwrap(), "payload-too-large");
    for body in [
        format!("{{\"bundle\":{{\"content\":\"{}\"}}}}", "x".repeat(64 * 1024 + 1)),
        format!("{{\"bundle\":{}0{}}}", "[".repeat(65), "]".repeat(65)),
    ] {
        let (status, _, bytes) = server.request(
            "POST",
            "/api/v1/project/bundle-verifications",
            true,
            None,
            "Content-Type: application/json\r\n",
            &body,
        );
        assert_eq!(status, 400);
        bundle_assert_error(&server, &serde_json::from_slice(&bytes).unwrap(), "invalid-request");
    }
}

/// Only the exact verification POST is read-only; authentication and existing write guards stay active.
#[test]
fn bundle_queries_preserve_authentication_and_exact_readonly_post_exception() {
    let server = Server::launch(true);
    let preview = bundle_preview(&server);
    let body = json!({"bundle":preview["bundle"]}).to_string();
    let before = bundle_project_files(&server);
    for (method, path, request_body) in [
        ("GET", "/api/v1/project/bundle-preview", ""),
        ("POST", "/api/v1/project/bundle-verifications", body.as_str()),
    ] {
        for (authorized, host, headers) in [
            (false, None, "Content-Type: application/json\r\n"),
            (
                false,
                None,
                "Content-Type: application/json\r\nAuthorization: Bearer PRIVATE REJECTED VALUE\r\n",
            ),
            (true, Some("hostile.invalid:1234"), "Content-Type: application/json\r\n"),
            (
                true,
                None,
                "Content-Type: application/json\r\nOrigin: https://attacker.invalid\r\nSec-Fetch-Site: cross-site\r\n",
            ),
            (true, None, "Content-Type: application/json\r\nForwarded: host=attacker.invalid\r\n"),
        ] {
            let (status, _, bytes) =
                server.request(method, path, authorized, host, headers, request_body);
            assert_eq!(status, 401, "{}", String::from_utf8_lossy(&bytes));
            bundle_assert_error(&server, &serde_json::from_slice(&bytes).unwrap(), "unauthorized");
        }
        for (suffix, headers) in [
            ("?private=PRIVATE-REJECTED-VALUE", "Content-Type: application/json\r\n"),
            ("", "Content-Type: application/json\r\nIdempotency-Key: bundle-query-0001\r\n"),
        ] {
            let (status, _, bytes) = server.request(
                method,
                &format!("{path}{suffix}"),
                true,
                None,
                headers,
                request_body,
            );
            assert_eq!(status, 400, "{}", String::from_utf8_lossy(&bytes));
            bundle_assert_error(
                &server,
                &serde_json::from_slice(&bytes).unwrap(),
                "invalid-request",
            );
        }
    }
    for headers in ["", "Content-Type: text/plain\r\n"] {
        let (status, _, bytes) = server.request(
            "POST",
            "/api/v1/project/bundle-verifications",
            true,
            None,
            headers,
            &body,
        );
        assert_eq!(status, 415);
        bundle_assert_error(
            &server,
            &serde_json::from_slice(&bytes).unwrap(),
            "unsupported-media-type",
        );
    }
    for (method, path) in [
        ("POST", "/api/v1/project/bundle-preview"),
        ("PUT", "/api/v1/project/bundle-verifications"),
        ("PATCH", "/api/v1/project/bundle-verifications"),
        ("DELETE", "/api/v1/project/bundle-verifications"),
        ("POST", "/api/v1/resources/register"),
    ] {
        let (status, _, bytes) =
            server.request(method, path, true, None, "Content-Type: application/json\r\n", &body);
        assert_eq!(status, 403, "{method} {path}: {}", String::from_utf8_lossy(&bytes));
        bundle_assert_error(&server, &serde_json::from_slice(&bytes).unwrap(), "read-only-session");
    }
    let (status, _, bytes) = server.request(
        "POST",
        "/api/v1/project/bundle-verifications",
        true,
        None,
        "Content-Type: Application/JSON; charset=utf-8\r\n",
        &body,
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&bytes));
    assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap()["state"], "matched");
    assert_eq!(bundle_project_files(&server), before);
}

/// Full actual HTTP queries preserve 101 authorial entries beyond the unrelated 100-input effect cap.
#[test]
fn bundle_queries_cover_101_resources_without_effect_prefix_truncation() {
    let server = Server::launch_mode(false, true);
    let mut resources = Vec::new();
    let mut sources = Vec::new();
    for ordinal in (0..101).rev() {
        let key = format!("source-{ordinal:04}");
        let path = format!("{key}.md");
        let bytes =
            format!("# Source {ordinal}\n\nSupplied fixture clause {ordinal}.\n").into_bytes();
        std::fs::write(server.project.path().join(&path), &bytes).unwrap();
        resources.push(json!({"key":key,"role":"policy-source","path":path}));
        sources.push(bytes);
    }
    let index = json!({"schema_version":"forge.workspace/1","label":"Complete 101 inventory","resources":resources});
    std::fs::write(
        server.project.path().join("forge.workspace.json"),
        serde_json::to_vec(&index).unwrap(),
    )
    .unwrap();
    let before = bundle_project_files(&server);
    let refs: Vec<&[u8]> = sources.iter().map(Vec::as_slice).collect();
    let preview = bundle_preview(&server);
    assert_eq!(preview["bundle"], bundle_fixture(&index, &refs));
    assert_eq!(preview["bundle"]["pins"].as_array().unwrap().len(), 101);
    let result = bundle_verify(&server, &preview["bundle"]);
    assert_eq!(result["state"], "matched");
    assert_eq!(result["expected_index_matches_current"], true);
    bundle_assert_counts(&result, [101, 101, 0, 0, 101, 0]);
    for (row, registration) in
        result["items"].as_array().unwrap().iter().zip(index["resources"].as_array().unwrap())
    {
        assert_eq!(row["key"], registration["key"]);
        assert_eq!(row["status"], "matched");
        assert_eq!(row["reason_codes"], json!([]));
        assert_eq!(row["observed_resource_validation_state"], "valid");
    }
    assert_eq!(bundle_project_files(&server), before);
}

/// Real HTTP checkpoint observations without the existing helper's implicit polling.
mod checkpoint_http {
    use super::{Server, bundle_fixture_sha256};
    use std::fmt::Write as _;
    use std::io::BufRead as _;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use serde_json::{Value, json};

    /// Redact ephemeral session authority while preserving the actual response structure.
    fn checkpoint_redact_authority(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                for (key, value) in fields {
                    if matches!(key.as_str(), "token" | "capability") {
                        *value = json!("<session-authority-redacted>");
                    } else {
                        checkpoint_redact_authority(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    checkpoint_redact_authority(value);
                }
            }
            _ => {}
        }
    }

    /// Retain the actual status, original response digest and redacted parsed body.
    fn checkpoint_record_response(
        method: &str,
        path: &str,
        key: Option<&str>,
        status: u16,
        bytes: &[u8],
        response: &Value,
    ) {
        let mut redacted = response.clone();
        checkpoint_redact_authority(&mut redacted);
        eprintln!(
            "checkpoint_http_response {}",
            json!({
                "method":method,"path":path,"idempotency_key":key,"status":status,
                "original_response_sha256":bundle_fixture_sha256(bytes),"response":redacted,
                "authority_redaction":"token and capability values only; request authorization omitted"
            })
        );
    }

    /// Send one JSON request and retain its immediate response, below 30 requests/second.
    fn checkpoint_raw(
        server: &Server,
        method: &str,
        path: &str,
        key: Option<&str>,
        value: &Value,
    ) -> (u16, Value) {
        std::thread::sleep(Duration::from_millis(50));
        let mut headers = String::from("Content-Type: application/json\r\n");
        if let Some(key) = key {
            write!(headers, "Idempotency-Key: {key}\r\n").unwrap();
        }
        let body = if method == "GET" { String::new() } else { value.to_string() };
        let (status, _, bytes) = server.request(method, path, true, None, &headers, &body);
        let response = serde_json::from_slice(&bytes).expect("checkpoint HTTP response is JSON");
        checkpoint_record_response(method, path, key, status, &bytes, &response);
        (status, response)
    }

    /// Admit actual preparation and preserve the accepted pending envelope before any GET.
    fn checkpoint_accept(server: &Server, path: &str, key: &str, request: &Value) -> Value {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let (status, operation) = checkpoint_raw(server, "POST", path, Some(key), request);
            if status == 400
                && operation["code"] == "invalid-request"
                && operation["retryable"] == true
            {
                assert!(
                    std::time::Instant::now() < deadline,
                    "preparation admission remained busy"
                );
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            assert_eq!(status, 202, "preparation admission status");
            assert_eq!(operation["state"], "pending");
            assert_eq!(operation["cancel_requested"], false);
            checkpoint_shape(&operation);
            return operation;
        }
    }

    /// Check active-only measured progress and the distinct terminal result/error contracts.
    fn checkpoint_shape(operation: &Value) {
        let active = matches!(operation["state"].as_str(), Some("pending" | "running"));
        if let Some(progress) = operation.get("progress").filter(|value| !value.is_null()) {
            assert!(active, "terminal operation retained capture progress");
            let completed = progress["completed_items"].as_u64().expect("completed capture count");
            let total = progress["total_items"].as_u64().expect("complete capture denominator");
            assert!(completed <= total, "capture count exceeds its denominator");
        }
        match operation["state"].as_str().expect("operation state") {
            "pending" | "running" | "cancelled" => {
                assert!(operation.get("result").is_none(), "non-success exposed a result");
                assert!(operation.get("error").is_none(), "non-failure exposed an error");
            }
            "succeeded" => {
                assert!(operation.get("result").is_some(), "success lacks a result");
                assert!(operation.get("error").is_none(), "success retained an error");
            }
            "failed" => {
                assert!(operation.get("error").is_some(), "failure lacks a safe error");
                assert!(operation.get("result").is_none(), "failure exposed a result");
            }
            _ => panic!("unknown operation state"),
        }
    }

    /// Poll an admitted ID through raw GETs without counting polls as additional test cases.
    fn checkpoint_terminal(server: &Server, accepted: &Value) -> Value {
        let id = accepted["operation_id"].as_str().expect("accepted operation ID");
        let path = format!("/api/v1/operations/{id}");
        let deadline = std::time::Instant::now() + Duration::from_secs(40);
        loop {
            let (status, operation) = checkpoint_raw(server, "GET", &path, None, &json!({}));
            assert_eq!(status, 200, "operation polling status");
            assert_eq!(operation["operation_id"], accepted["operation_id"]);
            assert_eq!(operation["kind"], accepted["kind"]);
            checkpoint_shape(&operation);
            if !matches!(operation["state"].as_str(), Some("pending" | "running")) {
                return operation;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "operation failed to settle within its budget"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Retain a byte fingerprint for every ordinary file in this flat owned fixture.
    fn checkpoint_fingerprints(server: &Server) -> std::collections::BTreeMap<String, String> {
        std::fs::read_dir(server.project.path())
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                assert!(
                    entry.file_type().unwrap().is_file(),
                    "fixture entry is not an ordinary file"
                );
                let name = entry.file_name().into_string().expect("UTF-8 fixture name");
                let bytes = std::fs::read(entry.path()).unwrap();
                (name, bundle_fixture_sha256(&bytes))
            })
            .collect()
    }

    /// Prepare a synchronous preview through the real API without implicit confirmation.
    fn checkpoint_preview(server: &Server, path: &str, key: &str, request: &Value) -> Value {
        let (status, response) = checkpoint_raw(server, "POST", path, Some(key), request);
        assert_eq!(status, 200, "synchronous preview status");
        assert!(response["preview"].is_object(), "synchronous route lacks a preview");
        response["preview"].clone()
    }

    /// Explicitly confirm exactly the supplied session receipt and verify committed write truth.
    fn checkpoint_confirm(server: &Server, preview: &Value, key: &str) -> Value {
        let request = json!({"receipt":preview["receipt"]["token"],
            "observed_version":preview["target_version"],"confirmed":true});
        let (status, operation) =
            checkpoint_raw(server, "POST", "/api/v1/effects/commits", Some(key), &request);
        assert_eq!(status, 202, "explicit commit status");
        assert_eq!(operation["kind"], "commit");
        assert_eq!(operation["state"], "succeeded");
        assert_eq!(operation["result"]["write_committed"], true);
        checkpoint_shape(&operation);
        operation
    }

    /// Register a supplied fixture by preparing and explicitly confirming its index edit.
    fn checkpoint_register(server: &Server, path: &str, role: &str, key: &str) {
        let preview = checkpoint_preview(
            server,
            "/api/v1/resources/register",
            &format!("checkpoint-register-{key}"),
            &json!({"path":path,"role":role,"key":key}),
        );
        checkpoint_confirm(server, &preview, &format!("checkpoint-register-commit-{key}"));
    }

    /// Look up one declared key in the complete small fixture's actual resource metadata.
    fn checkpoint_resource(server: &Server, key: &str) -> Value {
        let (status, resources) =
            checkpoint_raw(server, "GET", "/api/v1/resources", None, &json!({}));
        assert_eq!(status, 200, "resource inventory status");
        assert!(resources["page"]["next_cursor"].is_null(), "small fixture unexpectedly paginated");
        resources["page"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["key"] == key)
            .expect("declared fixture resource")
            .clone()
    }

    /// Construct 100 valid policy registrations below per-file and aggregate capture limits.
    fn checkpoint_capture_fixture(server: &Server) {
        let content = format!(
            "# Checkpoint policy\n\n## Scope\n\n{}",
            "Supplied checkpoint fixture text remains ordinary policy input.\n".repeat(6144)
        );
        assert!(content.len() < 400 * 1024);
        let resources: Vec<_> = (0..100)
            .map(|index| {
                let key = format!("checkpoint-policy-{index}");
                let path = format!("checkpoint-policy-{index}.md");
                std::fs::write(server.project.path().join(&path), &content).unwrap();
                json!({"key":key,"role":"policy-source","path":path})
            })
            .collect();
        let index = json!({"schema_version":"forge.workspace/1",
            "label":"Bounded checkpoint capture fixture","resources":resources});
        let index_bytes = serde_json::to_vec(&index).unwrap();
        assert!(content.len() * 100 + index_bytes.len() < 50 * 1024 * 1024);
        std::fs::write(server.project.path().join("forge.workspace.json"), index_bytes).unwrap();
        std::fs::write(server.project.path().join("checkpoint-trace.html"), "UNCHANGED TARGET")
            .unwrap();
    }

    /// Verify the same-key reply, terminal latch, and pre-commit download boundary.
    fn checkpoint_terminal_race(
        server: &Server,
        accepted: &Value,
        key: &str,
        request: &Value,
        terminal: &Value,
    ) {
        let id = accepted["operation_id"].as_str().unwrap();
        let query = format!("/api/v1/operations/{id}");
        let (status, replay) =
            checkpoint_raw(server, "POST", "/api/v1/exports", Some(key), request);
        assert_eq!(status, 202);
        assert!(accepted == &replay, "same-key replay changed its original admission envelope");
        let (status, rejection) =
            checkpoint_raw(server, "POST", &format!("{query}/cancellation"), None, &json!({}));
        assert_eq!(status, 409);
        assert_eq!(rejection["code"], "operation-not-cancellable");
        let (status, stable) = checkpoint_raw(server, "GET", &query, None, &json!({}));
        assert_eq!(status, 200);
        assert!(terminal == &stable, "late cancel or callback changed terminal state");
        assert_eq!(
            server.request("GET", &format!("/api/v1/exports/{id}/download"), true, None, "", "").0,
            404
        );
    }

    /// Return actual running/progress/acknowledgment observations from one bounded cancel race.
    fn checkpoint_cancel_attempt(
        server: &Server,
        request: &Value,
        attempt: usize,
    ) -> (bool, bool, bool) {
        let key = format!("checkpoint-observe-trace-{attempt}");
        let accepted = checkpoint_accept(server, "/api/v1/exports", &key, request);
        let id = accepted["operation_id"].as_str().unwrap();
        let query = format!("/api/v1/operations/{id}");
        let (status, observed) = checkpoint_raw(server, "GET", &query, None, &json!({}));
        assert_eq!(status, 200, "live operation GET status");
        assert_eq!(observed["operation_id"], accepted["operation_id"]);
        checkpoint_shape(&observed);
        let running = observed["state"] == "running";
        let capture = !observed["progress"].is_null();
        if capture {
            assert_eq!(observed["progress"]["total_items"], 100);
        }
        let (status, cancellation) =
            checkpoint_raw(server, "POST", &format!("{query}/cancellation"), None, &json!({}));
        let acknowledged = status == 200;
        if acknowledged {
            assert_eq!(cancellation["cancel_requested"], true);
            assert!(matches!(cancellation["state"].as_str(), Some("pending" | "running")));
            checkpoint_shape(&cancellation);
        } else {
            assert_eq!(status, 409, "terminal cancellation status");
            assert_eq!(cancellation["code"], "operation-not-cancellable");
        }
        let terminal = checkpoint_terminal(server, &accepted);
        assert_eq!(terminal["state"], if acknowledged { "cancelled" } else { "succeeded" });
        checkpoint_terminal_race(server, &accepted, &key, request, &terminal);
        eprintln!(
            "checkpoint_http_attempt attempt={attempt} admitted=pending observed_state={} capture_completed={:?} capture_total={:?} cancellation_status={status} cancellation_reply_state={} terminal_state={}",
            observed["state"].as_str().unwrap(),
            observed["progress"]["completed_items"].as_u64(),
            observed["progress"]["total_items"].as_u64(),
            cancellation["state"].as_str().unwrap_or("not-cancellable"),
            terminal["state"].as_str().unwrap()
        );
        (running, capture, acknowledged)
    }

    /// Observe real cancellation races; fast completion is reported without running-cancel credit.
    #[test]
    fn checkpoint_http_reports_observed_capture_cancellation_and_terminal_races() {
        let mut server = Server::launch_mode(false, false);
        let unrelated = checkpoint_preview(
            &server,
            "/api/v1/resources/register",
            "checkpoint-unrelated-register",
            &json!({"path":"unregistered.md",
                "role":"policy-source","key":"unrelated"}),
        );
        checkpoint_capture_fixture(&server);
        let before = checkpoint_fingerprints(&server);
        let request = json!({"report_kind":"trace","target_path":"checkpoint-trace.html"});
        let mut running_observations = 0;
        let mut capture_observations = 0;
        let mut acknowledged_cancellations = 0;
        let mut running_cancel_acknowledgments = 0;
        let mut fast_terminals = 0;
        let mut attempts = 0;
        for attempt in 0..3 {
            attempts += 1;
            let (running, capture, acknowledged) =
                checkpoint_cancel_attempt(&server, &request, attempt);
            running_observations += usize::from(running);
            capture_observations += usize::from(capture);
            acknowledged_cancellations += usize::from(acknowledged);
            running_cancel_acknowledgments += usize::from(running && acknowledged);
            fast_terminals += usize::from(!acknowledged);
            assert!(
                before == checkpoint_fingerprints(&server),
                "preparation changed fixture files"
            );
            if running && capture && acknowledged {
                break;
            }
        }
        let (status, retained) = checkpoint_raw(
            &server,
            "GET",
            &format!("/api/v1/effects/previews/{}", unrelated["preview_id"].as_str().unwrap()),
            None,
            &json!({}),
        );
        assert_eq!(status, 200);
        assert!(unrelated == retained, "unrelated retained preview changed");
        eprintln!(
            "checkpoint_http_observation attempts={attempts} running_gets={running_observations} capture_progress_gets={capture_observations} cancellation_acks={acknowledged_cancellations} running_get_then_cancel_ack_same_attempt={running_cancel_acknowledgments} terminal_before_cancel={fast_terminals}"
        );
        if running_cancel_acknowledgments == 0 || capture_observations == 0 {
            eprintln!(
                "checkpoint_http_observation qualification=one_or_more_running_capture_cancel_observations_not_obtained"
            );
        }
        checkpoint_stop(&mut server);
    }

    /// Stop through the real endpoint and wait for an actual successful process exit.
    fn checkpoint_stop(server: &mut Server) {
        let (status, response) =
            checkpoint_raw(server, "POST", "/api/v1/session/shutdown", None, &json!({}));
        assert_eq!(status, 200, "graceful shutdown status");
        assert_eq!(response["state"], "shutting-down");
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        loop {
            if let Some(status) = server.process.try_wait().unwrap() {
                assert!(status.success(), "workspace process exited unsuccessfully");
                return;
            }
            assert!(std::time::Instant::now() < deadline, "workspace process did not shut down");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Launch a fresh writable process against the same still-owned project after graceful exit.
    fn checkpoint_start_fresh(server: &mut Server) {
        assert!(server.process.try_wait().unwrap().is_some(), "previous process is still running");
        server.process = Command::new(env!("CARGO_BIN_EXE_forge"))
            .args(["workspace", "--project"])
            .arg(server.project.path())
            .arg("--machine-session")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = server.process.stdout.take().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = std::io::BufReader::new(stdout).read_line(&mut line).map(|_| line);
            let _ = send.send(result);
        });
        let line = receive
            .recv_timeout(Duration::from_secs(20))
            .expect("fresh workspace launch timed out")
            .expect("fresh machine descriptor read");
        let descriptor: Value = serde_json::from_str(&line).expect("fresh machine descriptor JSON");
        assert_eq!(descriptor["mode"], "machine");
        assert_eq!(descriptor["read_only"], false);
        descriptor["base_url"]
            .as_str()
            .unwrap()
            .strip_prefix("http://")
            .expect("fresh loopback URL")
            .clone_into(&mut server.host);
        assert!(server.host.starts_with("127.0.0.1:"));
        descriptor["capability"].as_str().unwrap().clone_into(&mut server.capability);
    }

    /// Reject the old capability, operation, preview, and receipt using fresh correct authority.
    fn checkpoint_reject_old_session(
        server: &Server,
        old_capability: &str,
        old_accepted: &Value,
        old_preview: &Value,
    ) {
        let old_auth = format!("Authorization: Bearer {old_capability}\r\n");
        let (status, _, bytes) =
            server.request("GET", "/api/v1/project/summary", false, None, &old_auth, "");
        let response = serde_json::from_slice(&bytes).expect("old authority rejection is JSON");
        checkpoint_record_response(
            "GET",
            "/api/v1/project/summary",
            None,
            status,
            &bytes,
            &response,
        );
        assert_eq!(status, 401);
        for path in [
            format!("/api/v1/operations/{}", old_accepted["operation_id"].as_str().unwrap()),
            format!("/api/v1/effects/previews/{}", old_preview["preview_id"].as_str().unwrap()),
        ] {
            let (status, rejected) = checkpoint_raw(server, "GET", &path, None, &json!({}));
            assert_eq!(status, 404);
            assert_eq!(rejected["code"], "not-found");
        }
        let old_commit = json!({"receipt":old_preview["receipt"]["token"],
            "observed_version":old_preview["target_version"],"confirmed":true});
        let (status, rejected) = checkpoint_raw(
            server,
            "POST",
            "/api/v1/effects/commits",
            Some("checkpoint-old-receipt"),
            &old_commit,
        );
        assert_eq!(status, 409);
        assert_eq!(rejected["code"], "receipt-mismatch");
    }

    /// Confirm only the fresh receipt, preserve all other files, and keep committed truth latched.
    fn checkpoint_confirm_fresh(server: &Server, preview: &Value) {
        let mut before = checkpoint_fingerprints(server);
        let committed = checkpoint_confirm(server, preview, "checkpoint-fresh-confirmed-commit");
        let output =
            std::fs::read(server.project.path().join("checkpoint-converted.json")).unwrap();
        assert_eq!(bundle_fixture_sha256(&output), preview["exact_bytes_sha256"].as_str().unwrap());
        assert_eq!(committed["result"]["committed_sha256"], preview["exact_bytes_sha256"]);
        let committed_files = checkpoint_fingerprints(server);
        let query = format!("/api/v1/operations/{}", committed["operation_id"].as_str().unwrap());
        let (status, rejected) =
            checkpoint_raw(server, "POST", &format!("{query}/cancellation"), None, &json!({}));
        assert_eq!(status, 409);
        assert_eq!(rejected["code"], "operation-not-cancellable");
        let (status, stable) = checkpoint_raw(server, "GET", &query, None, &json!({}));
        assert_eq!(status, 200);
        assert!(stable == committed, "late cancellation changed committed write truth");
        assert!(
            committed_files == checkpoint_fingerprints(server),
            "late cancellation changed files"
        );
        let mut after = checkpoint_fingerprints(server);
        before.remove("checkpoint-converted.json").expect("existing target sentinel");
        after.remove("checkpoint-converted.json").expect("explicitly committed target");
        assert!(
            before == after,
            "explicit confirmation changed a source, index, or unrelated file"
        );
    }

    /// Reject old session authority and require a fresh changed-input preview before writing.
    #[test]
    fn checkpoint_http_graceful_restart_rejects_old_state_and_recaptures_changed_input() {
        let mut server = Server::launch_mode(true, false);
        std::fs::write(
            server.project.path().join("policy.md"),
            "# Initial policy\n\n## Scope\n\n- Operators must review every initial request.\n",
        )
        .unwrap();
        std::fs::write(server.project.path().join("checkpoint-converted.json"), "UNCHANGED TARGET")
            .unwrap();
        let before = checkpoint_fingerprints(&server);
        let source = checkpoint_resource(&server, "policy");
        let request = json!({"source_resource_id":source["resource_id"],
            "output_kind":"oscal-catalog","target_path":"checkpoint-converted.json"});
        let key = "checkpoint-restart-conversion";
        let old_accepted = checkpoint_accept(&server, "/api/v1/conversions", key, &request);
        let old_terminal = checkpoint_terminal(&server, &old_accepted);
        assert_eq!(old_terminal["state"], "succeeded");
        let old_preview = old_terminal["result"]["preview"].clone();
        let old_capability = server.capability.clone();
        checkpoint_stop(&mut server);
        assert!(
            before == checkpoint_fingerprints(&server),
            "shutdown or unconfirmed preview wrote files"
        );
        let changed = "# Changed policy\n\n## Scope\n\n- Every changed request must receive explicit review.\n";
        std::fs::write(server.project.path().join("policy.md"), changed).unwrap();
        let changed_before = checkpoint_fingerprints(&server);
        checkpoint_start_fresh(&mut server);
        assert!(old_capability != server.capability, "fresh session reused its capability");
        checkpoint_reject_old_session(&server, &old_capability, &old_accepted, &old_preview);
        assert!(changed_before == checkpoint_fingerprints(&server), "old authority changed files");
        let current = checkpoint_resource(&server, "policy");
        assert_eq!(current["resource_id"], source["resource_id"]);
        assert_eq!(current["sha256"], bundle_fixture_sha256(changed.as_bytes()));
        let accepted = checkpoint_accept(&server, "/api/v1/conversions", key, &request);
        assert_ne!(accepted["operation_id"], old_accepted["operation_id"]);
        let terminal = checkpoint_terminal(&server, &accepted);
        assert_eq!(terminal["state"], "succeeded");
        let preview = &terminal["result"]["preview"];
        assert_eq!(
            preview["input_hashes"],
            json!([{"resource_id":current["resource_id"],"sha256":current["sha256"]}])
        );
        assert_ne!(preview["exact_bytes_sha256"], old_preview["exact_bytes_sha256"]);
        assert!(
            changed_before == checkpoint_fingerprints(&server),
            "fresh preparation wrote files"
        );
        checkpoint_confirm_fresh(&server, preview);
        checkpoint_stop(&mut server);
        eprintln!(
            "checkpoint_http_restart graceful_process_exits=2 old_capability_rejected=1 old_operation_rejected=1 old_preview_rejected=1 old_receipt_rejected=1 changed_source_hash_bound=1 explicit_fresh_commit=1"
        );
    }

    /// Build a small valid supplied-policy/catalog fixture for all four preparation adapters.
    fn checkpoint_four_route_fixture(server: &Server) {
        std::fs::write(
            server.project.path().join("policy.md"),
            "# Supplied policy\n\n## Scope\n\n- Operators must review every supplied request.\n",
        )
        .unwrap();
        for (key, id, uuid) in [
            ("source", "policy-a", "11111111-1111-4111-8111-111111111111"),
            ("framework", "framework-a", "22222222-2222-4222-8222-222222222222"),
        ] {
            let catalog = json!({"catalog":{"uuid":uuid,"metadata":{"title":"Checkpoint catalog",
                "last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},
                "controls":[{"id":id,"title":"Supplied checkpoint control"}]}});
            let path = format!("checkpoint-{key}.json");
            std::fs::write(
                server.project.path().join(&path),
                serde_json::to_vec(&catalog).unwrap(),
            )
            .unwrap();
            checkpoint_register(server, &path, "oscal-catalog-artifact", key);
        }
        let source = checkpoint_resource(server, "source");
        let framework = checkpoint_resource(server, "framework");
        let scope = checkpoint_preview(
            server,
            "/api/v1/applicability/initializations",
            "checkpoint-scope-initialize",
            &json!({"framework_resource_id":framework["resource_id"],
                "target_path":"checkpoint-scope.json"}),
        );
        checkpoint_confirm(server, &scope, "checkpoint-scope-initialize-commit");
        checkpoint_register(server, "checkpoint-scope.json", "applicability-manifest", "scope");
        let mapping_request = json!({"source_resource_id":source["resource_id"],
            "target_resource_id":framework["resource_id"],"target_path":"checkpoint-mapping.json",
            "scope":"control-only","maps":[{"key":"explicit-none","relationship":"no-relationship",
                "sources":[{"type":"control","id_ref":"policy-a"}],
                "targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer",
                "reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit supplied absence."}],
            "review":{"collection":{"key":"checkpoint-map","title":"Checkpoint mapping",
                "version":"1","last_modified":"2026-09-10T00:00:00Z"},
                "reviewers":[{"key":"reviewer","type":"person","name":"Checkpoint Reviewer"}],
                "provenance":{"method":"human","matching_rationale":"semantic","status":"draft",
                    "mapping_description":"Explicit supplied review.","reviewer_keys":["reviewer"],
                    "reviewed_at":"2026-09-10T00:00:00Z"}}});
        let mapping = checkpoint_preview(
            server,
            "/api/v1/mapping/initializations",
            "checkpoint-mapping-initialize",
            &mapping_request,
        );
        checkpoint_confirm(server, &mapping, "checkpoint-mapping-initialize-commit");
        checkpoint_register(server, "checkpoint-mapping.json", "mapping-collection", "mapping");
    }

    /// Exercise normal raw admission and real domain results for each background preparation kind.
    #[test]
    fn checkpoint_http_all_four_preparation_routes_settle_without_implicit_writes() {
        let mut server = Server::launch_mode(true, false);
        checkpoint_four_route_fixture(&server);
        let policy = checkpoint_resource(&server, "policy");
        for path in [
            "checkpoint-output.json",
            "applicability-report.json",
            "mapping-collection.json",
            "checkpoint-review.html",
        ] {
            std::fs::write(server.project.path().join(path), "UNCHANGED TARGET").unwrap();
        }
        let before = checkpoint_fingerprints(&server);
        for (path, kind, request, preview_member) in [
            (
                "/api/v1/conversions",
                "conversion",
                json!({"source_resource_id":policy["resource_id"],
                "output_kind":"oscal-catalog","target_path":"checkpoint-output.json"}),
                "preview",
            ),
            (
                "/api/v1/applicability/analyses",
                "applicability-analysis",
                json!({}),
                "report_preview",
            ),
            ("/api/v1/mapping/builds", "mapping-build", json!({}), "report_preview"),
            (
                "/api/v1/exports",
                "export",
                json!({"report_kind":"trace",
                "target_path":"checkpoint-review.html"}),
                "preview",
            ),
        ] {
            let key = format!("checkpoint-four-routes-{kind}");
            let accepted = checkpoint_accept(&server, path, &key, &request);
            assert_eq!(accepted["kind"], kind);
            let terminal = checkpoint_terminal(&server, &accepted);
            assert_eq!(terminal["state"], "succeeded", "normal preparation kind: {kind}");
            let preview = &terminal["result"][preview_member];
            assert!(preview.is_object(), "normal preparation lacks the expected preview");
            let (status, retained) = checkpoint_raw(
                &server,
                "GET",
                &format!("/api/v1/effects/previews/{}", preview["preview_id"].as_str().unwrap()),
                None,
                &json!({}),
            );
            assert_eq!(status, 200);
            assert!(preview == &retained, "prepared preview was not retained exactly");
            let (status, replay) = checkpoint_raw(&server, "POST", path, Some(&key), &request);
            assert_eq!(status, 202);
            assert!(accepted == replay, "same-key retry admitted a different operation");
            assert!(before == checkpoint_fingerprints(&server), "normal preparation wrote files");
            match kind {
                "conversion" => assert_eq!(terminal["result"]["validation"]["state"], "valid"),
                "applicability-analysis" => assert_eq!(terminal["result"]["eligible_controls"], 1),
                "mapping-build" => {
                    assert_eq!(terminal["result"]["maps_total"], 1);
                    assert_eq!(terminal["result"]["no_relationship_count"], 1);
                }
                "export" => assert!(terminal["result"]["redaction_summary"].is_object()),
                _ => panic!("unknown checkpoint route fixture"),
            }
            eprintln!(
                "checkpoint_http_normal_route kind={kind} accepted=pending terminal=succeeded confirmed=false"
            );
        }
        checkpoint_stop(&mut server);
    }
}
/// Exercise index-version admission and metadata bundles through authenticated real HTTP.
mod s3_registration_and_bundles {
    use super::{
        Server, bundle_assert_counts, bundle_assert_error, bundle_assert_keys, bundle_fixture,
        bundle_fixture_index_bytes, bundle_fixture_sha256, bundle_project_files,
    };
    use serde_json::{Value, json};
    use std::fmt::Write as _;
    use std::io::{BufRead as _, Read as _, Write as _};
    use std::net::TcpStream;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    /// Assert only descriptor keys so failure output never prints capability values.
    fn s3_descriptor_keys(descriptor: &Value) {
        let mut fields: Vec<&str> = descriptor
            .as_object()
            .expect("API2 descriptor object")
            .keys()
            .map(String::as_str)
            .collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            [
                "api_major",
                "api_version",
                "base_url",
                "capability",
                "mode",
                "pid",
                "read_only",
                "session_id"
            ]
        );
    }

    impl Server {
        /// Launch an explicitly selected API2 process without changing legacy arguments, retaining owner cleanup before any bootstrap assertion.
        fn start_api2(with_resource: bool, read_only: bool) -> Self {
            let project = tempfile::tempdir().unwrap();
            std::fs::write(project.path().join("unregistered.md"), "PRIVATE UNREGISTERED CONTENT")
                .unwrap();
            if with_resource {
                std::fs::write(
                    project.path().join("policy.md"),
                    "# Example\n\nA human-supplied clause.\n",
                )
                .unwrap();
                let index = json!({"schema_version":"forge.workspace/1","label":"Example project","resources":[{"key":"policy","role":"policy-source","path":"policy.md"}]});
                std::fs::write(
                    project.path().join("forge.workspace.json"),
                    serde_json::to_vec(&index).unwrap(),
                )
                .unwrap();
            }
            let mut command = Command::new(env!("CARGO_BIN_EXE_forge"));
            command.args(["workspace", "--project"]).arg(project.path()).args([
                "--machine-session",
                "--api-major",
                "2",
            ]);
            if read_only {
                command.arg("--read-only");
            }
            let mut process = command
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let stdout = process.stdout.take().unwrap();
            let mut server =
                Self { process, host: String::new(), capability: String::new(), project };
            let (send, receive) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let mut line = String::new();
                let result = std::io::BufReader::new(stdout).read_line(&mut line).map(|_| line);
                let _ = send.send(result);
            });
            let result = receive.recv_timeout(Duration::from_secs(20));
            if result.is_err() {
                let _ = server.process.kill();
                let _ = server.process.wait();
                panic!("API2 workspace launch timed out");
            }
            let descriptor: Value = serde_json::from_str(&result.unwrap().unwrap())
                .expect("API2 machine descriptor JSON");
            s3_descriptor_keys(&descriptor);
            assert!(descriptor["api_version"] == "2.3.0", "API2 descriptor version mismatch");
            assert!(descriptor["api_major"] == 2, "API2 descriptor major mismatch");
            assert!(descriptor["mode"] == "machine", "API2 descriptor mode mismatch");
            assert!(descriptor["read_only"] == read_only, "API2 descriptor scope mismatch");
            let host = descriptor["base_url"]
                .as_str()
                .unwrap()
                .strip_prefix("http://")
                .unwrap()
                .to_owned();
            assert!(host.starts_with("127.0.0.1:"), "API2 descriptor must select loopback");
            server.host = host;
            descriptor["capability"].as_str().unwrap().clone_into(&mut server.capability);
            let (status, session) = s3_json(&server, "GET", "/api/v2/session", None, &json!({}));
            assert_eq!(status, 200, "{session}");
            bundle_assert_keys(
                &session,
                &[
                    "session_id",
                    "mode",
                    "read_only",
                    "api_major",
                    "contract_version",
                    "project_label",
                    "launched_at",
                ],
            );
            assert_eq!(session["api_major"], 2);
            assert_eq!(session["contract_version"], "2.3.0");
            assert_eq!(session["session_id"], descriptor["session_id"]);
            assert_eq!(session["mode"], "machine");
            assert_eq!(session["read_only"], read_only);
            server
        }
    }

    /// Decode one real API2 response without the legacy helper's implicit API1 operation polling.
    fn s3_json(
        server: &Server,
        method: &str,
        path: &str,
        key: Option<&str>,
        value: &Value,
    ) -> (u16, Value) {
        let mut headers = String::from("Content-Type: application/json\r\n");
        if let Some(key) = key {
            write!(headers, "Idempotency-Key: {key}\r\n").unwrap();
        }
        let body = if method == "GET" { String::new() } else { value.to_string() };
        let (status, _, bytes) = server.request(method, path, true, None, &headers, &body);
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    /// Commit the exact API2 receipt and require the existing immediately settled commit contract.
    fn s3_commit(server: &Server, preview: &Value, key: &str) -> Value {
        let (status, operation) = s3_json(
            server,
            "POST",
            "/api/v2/effects/commits",
            Some(key),
            &json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}),
        );
        assert_eq!(status, 202, "{operation}");
        assert_eq!(operation["kind"], "commit");
        assert_eq!(operation["state"], "succeeded");
        operation
    }

    /// Fetch the actual API2 metadata preview with no receipt, source excerpt or write authority.
    fn s3_bundle_preview(server: &Server) -> Value {
        let (status, headers, bytes) =
            server.request("GET", "/api/v2/project/bundle-preview", true, None, "", "");
        assert_eq!(status, 200, "{}", String::from_utf8_lossy(&bytes));
        assert!(headers.contains("cache-control: no-store"));
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        bundle_assert_keys(
            &value,
            &[
                "bundle",
                "source_index_present",
                "snapshot_version",
                "included_metadata",
                "source_content_included",
            ],
        );
        assert_eq!(value["source_index_present"], true);
        assert_eq!(value["source_content_included"], false);
        assert_eq!(
            value["included_metadata"],
            json!([
                "project-label",
                "resource-keys",
                "typed-roles",
                "project-relative-paths",
                "sha256-fingerprints",
                "byte-lengths"
            ])
        );
        bundle_assert_keys(
            &value["bundle"],
            &["schema_version", "content_profile", "index", "index_sha256", "pins"],
        );
        for pin in value["bundle"]["pins"].as_array().unwrap() {
            bundle_assert_keys(pin, &["key", "sha256", "size_bytes"]);
        }
        value
    }

    /// Compare through API2 without a preparation receipt, while retaining every expected row and separate metadata state.
    fn s3_bundle_verify(server: &Server, bundle: &Value) -> Value {
        let (status, value) = s3_json(
            server,
            "POST",
            "/api/v2/project/bundle-verifications",
            None,
            &json!({"bundle":bundle}),
        );
        assert_eq!(status, 200, "{value}");
        bundle_assert_keys(
            &value,
            &[
                "scope",
                "snapshot_version",
                "source_index_present",
                "state",
                "current_resources",
                "current_only_resources",
                "expected_index_matches_current",
                "expected_resources",
                "matched_resources",
                "unregistered_resources",
                "mismatched_resources",
                "items",
                "source_content_included",
            ],
        );
        assert_eq!(value["scope"], "registered-fingerprints-only");
        assert_eq!(value["source_content_included"], false);
        for row in value["items"].as_array().unwrap() {
            bundle_assert_keys(
                row,
                &["key", "status", "reason_codes", "observed_resource_validation_state"],
            );
        }
        value
    }

    /// Send literal and decoded duplicate keys to API2's raw transport rather than a reserialized Value.
    fn s3_raw_duplicates_rejected(server: &Server, valid: &Value, pin_hash: &str) {
        let text = valid.to_string();
        let property = format!("\"sha256\":\"{pin_hash}\"");
        let duplicate =
            text.replacen(&property, &format!("{property},\"sha256\":\"{pin_hash}\""), 1);
        let decoded =
            text.replacen(&property, &format!("{property},\"sha\\u003256\":\"{pin_hash}\""), 1);
        assert_ne!(duplicate, text);
        assert_ne!(decoded, text);
        for body in [
            format!("{{\"bundle\":{},\"bundle\":{}}}", valid["bundle"], valid["bundle"]),
            format!("{{\"bundle\":{},\"bund\\u006ce\":{}}}", valid["bundle"], valid["bundle"]),
            duplicate,
            decoded,
        ] {
            let (status, _, response) = server.request(
                "POST",
                "/api/v2/project/bundle-verifications",
                true,
                None,
                "Content-Type: application/json\r\n",
                &body,
            );
            assert_eq!(status, 400, "{}", String::from_utf8_lossy(&response));
            bundle_assert_error(
                server,
                &serde_json::from_slice(&response).unwrap(),
                "invalid-request",
            );
        }
    }

    /// A selected namespace rejects foreign majors before capture; later API1 capture fences an external index2.
    #[test]
    fn s3_major_namespaces_and_later_api1_capture_remain_closed() {
        let two = Server::start_api2(true, false);
        let before = bundle_project_files(&two);
        std::fs::write(two.project.path().join("forge.workspace.json"), b"not a valid index")
            .unwrap();
        for (method, path, body) in [
            ("GET", "/api/v1/resources", json!({})),
            (
                "POST",
                "/api/v1/resources/register",
                json!({"path":"missing.bin","role":"lifecycle-source"}),
            ),
            ("GET", "/api/v2x/resources", json!({})),
        ] {
            let (status, error) = s3_json(&two, method, path, None, &body);
            assert_eq!(status, 404, "{error}");
            bundle_assert_error(&two, &error, "not-found");
        }
        std::fs::write(
            two.project.path().join("forge.workspace.json"),
            &before.iter().find(|(path, _)| path == "forge.workspace.json").unwrap().1,
        )
        .unwrap();
        assert_eq!(s3_json(&two, "GET", "/api/v2/resources", None, &json!({})).0, 200);
        assert_eq!(bundle_project_files(&two), before);
        let one = Server::launch_mode(false, true);
        let index = json!({"schema_version":"forge.workspace/2","label":"External migration","resources":[{"key":"missing-source","role":"lifecycle-source","path":"missing.bin"}]});
        let bytes = serde_json::to_vec(&index).unwrap();
        std::fs::write(one.project.path().join("forge.workspace.json"), &bytes).unwrap();
        let (status, error) = one.json("GET", "/api/v1/resources", None, &json!({}));
        assert_eq!(status, 400, "{error}");
        bundle_assert_error(&one, &error, "invalid-request");
        assert_eq!(error["message"], "This workspace index requires --api-major 2.");
        assert_eq!(std::fs::read(one.project.path().join("forge.workspace.json")).unwrap(), bytes);
    }

    /// Preserve the complete closed role set without claiming domain-valid fixture bytes.
    const S3_ROLES: [&str; 15] = [
        "policy-source",
        "oscal-catalog-artifact",
        "oscal-component-artifact",
        "mapping-collection",
        "applicability-manifest",
        "applicability-report",
        "trace-report",
        "lifecycle-record",
        "lifecycle-source",
        "oscal-profile-artifact",
        "oscal-ssp-artifact",
        "framework-impact-manifest",
        "successor-map",
        "framework-impact-report",
        "framework-impact-dispositions",
    ];

    /// Read the authorial index after a real conditional commit, retaining its complete semantic value.
    fn s3_index(server: &Server) -> Value {
        serde_json::from_slice(
            &std::fs::read(server.project.path().join("forge.workspace.json")).unwrap(),
        )
        .unwrap()
    }

    /// Prepare a registration or migration without treating a preview as a persisted write.
    fn s3_prepare(server: &Server, request: &Value, key: &str) -> Value {
        let (status, response) =
            s3_json(server, "POST", "/api/v2/resources/register", Some(key), request);
        assert_eq!(status, 200, "{response}");
        assert_eq!(response["preview"]["operation_type"], "workspace-index-update");
        response["preview"].clone()
    }

    /// Send a genuine rejected request and require its typed, redacted response and unchanged files.
    fn s3_reject(server: &Server, request: &Value, key: &str, status: u16, code: &str) {
        let before = bundle_project_files(server);
        let (actual, error) =
            s3_json(server, "POST", "/api/v2/resources/register", Some(key), request);
        assert_eq!(actual, status, "{error}");
        bundle_assert_error(server, &error, code);
        assert_eq!(bundle_project_files(server), before);
    }

    /// Extend only the test envelope version; original index ordering and original-byte pins stay exact.
    fn s3_bundle(index: &Value, source_bytes: &[&[u8]]) -> Value {
        let mut bundle = bundle_fixture(index, source_bytes);
        if index["schema_version"] == "forge.workspace/2" {
            bundle["schema_version"] = json!("forge.workspace-index-bundle/2");
        }
        bundle
    }

    /// A legacy null key derives a key, while migration needs exact confirmation and preserves order.
    #[test]
    fn s3_legacy_null_key_and_migration_preserve_authorial_index() {
        let server = Server::start_api2(true, false);
        let original_files = bundle_project_files(&server);
        let preview = s3_prepare(
            &server,
            &json!({"path":"unregistered.md","role":"policy-source","key":null}),
            "s3-register-legacy-null",
        );
        assert_eq!(bundle_project_files(&server), original_files);
        s3_commit(&server, &preview, "s3-commit-legacy-null");
        let legacy = s3_index(&server);
        assert_eq!(legacy["schema_version"], "forge.workspace/1");
        assert_eq!(legacy["resources"].as_array().unwrap().len(), 2);
        assert_eq!(legacy["resources"][0]["key"], "policy");
        assert_eq!(legacy["resources"][1]["path"], "unregistered.md");
        assert!(legacy["resources"][1]["key"].as_str().is_some_and(|key| !key.is_empty()));
        let before = bundle_project_files(&server);
        let migration = json!({"migration":{"from":"forge.workspace/1","to":"forge.workspace/2"}});
        let preview = s3_prepare(&server, &migration, "s3-migrate-index");
        assert_eq!(bundle_project_files(&server), before);
        // This is the server's literal confirmation guard, not a native UI rejection claim.
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/effects/commits",
            Some("s3-reject-confirmation"),
            &json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":false}),
        );
        assert_eq!(status, 400, "{error}");
        bundle_assert_error(&server, &error, "invalid-request");
        assert_eq!(bundle_project_files(&server), before);
        s3_commit(&server, &preview, "s3-confirm-migration");
        let mut expected = legacy;
        expected["schema_version"] = json!("forge.workspace/2");
        assert_eq!(s3_index(&server), expected);
        let bytes = std::fs::read(server.project.path().join("forge.workspace.json")).unwrap();
        assert_eq!(bytes, bundle_fixture_index_bytes(&expected));
        assert_eq!(preview["exact_bytes_sha256"], bundle_fixture_sha256(&bytes));
        for (path, bytes) in before.iter().filter(|(path, _)| path != "forge.workspace.json") {
            assert_eq!(std::fs::read(server.project.path().join(path)).unwrap(), *bytes);
        }
        s3_reject(&server, &migration, "s3-migrate-already-two", 422, "validation-failed");
    }

    /// Version admission precedes a missing-file read; explicit /2 admits opaque empty and binary sources.
    #[test]
    fn s3_explicit_version_admission_and_opaque_source_registration() {
        let server = Server::start_api2(false, false);
        let migration = json!({"migration":{"from":"forge.workspace/1","to":"forge.workspace/2"}});
        s3_reject(&server, &migration, "s3-migrate-absent", 422, "validation-failed");
        s3_reject(
            &server,
            &json!({"path":"missing.bin","role":"lifecycle-source"}),
            "s3-role-before-missing-read",
            400,
            "invalid-request",
        );
        std::fs::write(server.project.path().join("empty.bin"), []).unwrap();
        let preview = s3_prepare(
            &server,
            &json!({"path":"empty.bin","role":"lifecycle-source","key":null,"index_schema_version":"forge.workspace/2"}),
            "s3-explicit-two-empty",
        );
        assert!(!server.project.path().join("forge.workspace.json").exists());
        s3_commit(&server, &preview, "s3-commit-two-empty");
        let first = s3_index(&server);
        assert_eq!(first["schema_version"], "forge.workspace/2");
        assert_eq!(first["resources"][0]["role"], "lifecycle-source");
        let binary = [0xff, 0x00, 0x80, b'\n'];
        std::fs::write(server.project.path().join("binary.bin"), binary).unwrap();
        let preview = s3_prepare(
            &server,
            &json!({"path":"binary.bin","role":"lifecycle-source","key":"binary-source"}),
            "s3-current-two-old-shape",
        );
        s3_commit(&server, &preview, "s3-commit-current-two");
        let (status, resources) = s3_json(&server, "GET", "/api/v2/resources", None, &json!({}));
        assert_eq!(status, 200, "{resources}");
        let rows = resources["page"]["items"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        for row in rows {
            assert_eq!(row["validation_state"], "valid");
            assert_eq!(row["validation_profile"], "opaque-fingerprint-bytes");
        }
        let empty = rows.iter().find(|row| row["path"] == "empty.bin").unwrap();
        assert_eq!(empty["size_bytes"], 0);
        assert_eq!(empty["sha256"], bundle_fixture_sha256(&[]));
        let observed = rows.iter().find(|row| row["path"] == "binary.bin").unwrap();
        assert_eq!(observed["sha256"], bundle_fixture_sha256(&binary));
        assert_eq!(std::fs::read(server.project.path().join("binary.bin")).unwrap(), binary);
    }

    /// Mixed alternatives, unknown fields and null version selectors cannot prepare an index write.
    #[test]
    fn s3_closed_registration_alternatives_preserve_bytes() {
        let server = Server::start_api2(true, false);
        let migration = json!({"from":"forge.workspace/1","to":"forge.workspace/2"});
        let invalid = [
            json!({"path":"unregistered.md","role":"policy-source","index_schema_version":null}),
            json!({"path":"unregistered.md","role":"policy-source","index_schema_version":"forge.workspace/1"}),
            json!({"path":"unregistered.md","role":"lifecycle-source"}),
            json!({"path":"unregistered.md","role":"policy-source","unknown":true}),
            json!({"migration":migration,"path":"unregistered.md","role":"policy-source"}),
            json!({"migration":null}),
            json!({"migration":{"from":"forge.workspace/2","to":"forge.workspace/2"}}),
            json!({"migration":{"from":"forge.workspace/1","to":"forge.workspace/2","approved":true}}),
        ];
        for (number, request) in invalid.iter().enumerate() {
            s3_reject(
                &server,
                request,
                &format!("s3-invalid-alternative-{number}"),
                400,
                "invalid-request",
            );
        }
        s3_reject(
            &server,
            &json!({"path":"unregistered.md","role":"lifecycle-record","index_schema_version":"forge.workspace/2","key":"invalid-record"}),
            "s3-strict-domain-rejection",
            422,
            "validation-failed",
        );
        let preview = s3_prepare(
            &server,
            &json!({"path":"unregistered.md","role":"lifecycle-source","index_schema_version":"forge.workspace/2","key":"new-source"}),
            "s3-alternative-positive",
        );
        s3_commit(&server, &preview, "s3-alternative-positive-commit");
        assert_eq!(s3_index(&server)["schema_version"], "forge.workspace/2");
        assert_eq!(s3_index(&server)["resources"].as_array().unwrap().len(), 2);
    }

    /// Conditional commits retain concurrent index bytes and recheck the exact external source bytes.
    #[test]
    fn s3_migration_registration_rechecks_index_and_external_source() {
        let server = Server::start_api2(true, false);
        let request = json!({"path":"unregistered.md","role":"lifecycle-source","key":"external-source","index_schema_version":"forge.workspace/2"});
        let preview = s3_prepare(&server, &request, "s3-drift-preview-index");
        let mut concurrent = s3_index(&server);
        concurrent["label"] = json!("Concurrent authorial label");
        let concurrent_bytes = serde_json::to_vec(&concurrent).unwrap();
        std::fs::write(server.project.path().join("forge.workspace.json"), &concurrent_bytes)
            .unwrap();
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/effects/commits",
            Some("s3-drift-index-commit"),
            &json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}),
        );
        assert_eq!(status, 409, "{error}");
        assert_eq!(
            std::fs::read(server.project.path().join("forge.workspace.json")).unwrap(),
            concurrent_bytes
        );
        let preview = s3_prepare(&server, &request, "s3-drift-preview-source");
        std::fs::write(server.project.path().join("unregistered.md"), b"changed opaque source")
            .unwrap();
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/effects/commits",
            Some("s3-drift-source-commit"),
            &json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}),
        );
        assert_eq!(status, 409, "{error}");
        assert_eq!(
            std::fs::read(server.project.path().join("forge.workspace.json")).unwrap(),
            concurrent_bytes
        );
        let preview = s3_prepare(&server, &request, "s3-drift-fresh-preview");
        s3_commit(&server, &preview, "s3-drift-fresh-commit");
        assert_eq!(s3_index(&server)["label"], "Concurrent authorial label");
        let bundle = s3_bundle_preview(&server)["bundle"].clone();
        assert_eq!(bundle["pins"][1]["sha256"], bundle_fixture_sha256(b"changed opaque source"));
    }

    /// All fifteen registered roles enter metadata bundles; fingerprint equality never upgrades domain validity.
    #[test]
    fn s3_bundle_two_all_roles_and_legacy_subset_keep_validation_separate() {
        let server = Server::start_api2(false, true);
        let bytes = b"# Source\n\nPRIVATE BUNDLE SOURCE BYTES\n";
        let resources: Vec<Value> = S3_ROLES
            .iter()
            .enumerate()
            .map(|(number, role)| {
                let path = format!("role-{number}.bin");
                std::fs::write(server.project.path().join(&path), bytes).unwrap();
                json!({"key":format!("role-{number}"),"role":role,"path":path})
            })
            .collect();
        let index = json!({"schema_version":"forge.workspace/2","label":"All declared roles","resources":resources});
        std::fs::write(
            server.project.path().join("forge.workspace.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let before = bundle_project_files(&server);
        let preview = s3_bundle_preview(&server);
        assert_eq!(preview["bundle"], s3_bundle(&index, &[bytes.as_slice(); 15]));
        let result = s3_bundle_verify(&server, &preview["bundle"]);
        assert_eq!(result["state"], "matched");
        assert_eq!(result["expected_index_matches_current"], true);
        bundle_assert_counts(&result, [15, 15, 0, 0, 15, 0]);
        let (status, resources) = s3_json(&server, "GET", "/api/v2/resources", None, &json!({}));
        assert_eq!(status, 200, "{resources}");
        let observed = resources["page"]["items"].as_array().unwrap();
        assert_eq!(observed.len(), 15);
        for row in result["items"].as_array().unwrap() {
            let metadata = observed.iter().find(|item| item["key"] == row["key"]).unwrap();
            assert_eq!(row["observed_resource_validation_state"], metadata["validation_state"]);
        }
        assert_eq!(
            observed.iter().find(|row| row["role"] == "lifecycle-source").unwrap()["validation_profile"],
            "opaque-fingerprint-bytes"
        );
        assert_eq!(
            observed.iter().find(|row| row["role"] == "lifecycle-record").unwrap()["validation_state"],
            "invalid"
        );
        let subset = json!({"schema_version":"forge.workspace/1","label":"All declared roles","resources":[index["resources"][0].clone()]});
        let legacy = s3_bundle_verify(&server, &s3_bundle(&subset, &[bytes]));
        assert_eq!(legacy["state"], "matched");
        assert_eq!(legacy["expected_index_matches_current"], false);
        bundle_assert_counts(&legacy, [1, 1, 0, 0, 15, 14]);
        for value in [&preview, &result, &resources, &legacy] {
            assert!(!value.to_string().contains("PRIVATE BUNDLE SOURCE BYTES"));
            assert!(!value.to_string().contains("PRIVATE UNREGISTERED CONTENT"));
        }
        assert_eq!(bundle_project_files(&server), before);
    }

    /// Absent indexes differ from authored empty versions; zero subset matches do not imply index equality.
    #[test]
    fn s3_bundle_versions_keep_missing_and_zero_denominators_distinct() {
        let server = Server::start_api2(false, true);
        let one =
            json!({"schema_version":"forge.workspace/1","label":"Empty index","resources":[]});
        let mut two = one.clone();
        two["schema_version"] = json!("forge.workspace/2");
        let empty_one = s3_bundle(&one, &[]);
        let empty_two = s3_bundle(&two, &[]);
        let (status, error) =
            s3_json(&server, "GET", "/api/v2/project/bundle-preview", None, &json!({}));
        assert_eq!(status, 404, "{error}");
        bundle_assert_error(&server, &error, "not-found");
        for bundle in [&empty_one, &empty_two] {
            let result = s3_bundle_verify(&server, bundle);
            assert_eq!(result["state"], "missing-index");
            assert_eq!(result["source_index_present"], false);
            bundle_assert_counts(&result, [0, 0, 0, 0, 0, 0]);
        }
        for (index, bundle) in [(&one, &empty_one), (&two, &empty_two)] {
            std::fs::write(
                server.project.path().join("forge.workspace.json"),
                serde_json::to_vec(index).unwrap(),
            )
            .unwrap();
            let before = bundle_project_files(&server);
            assert_eq!(s3_bundle_preview(&server)["bundle"], *bundle);
            let result = s3_bundle_verify(&server, bundle);
            assert_eq!(result["state"], "matched");
            assert_eq!(result["expected_index_matches_current"], true);
            bundle_assert_counts(&result, [0, 0, 0, 0, 0, 0]);
            assert_eq!(bundle_project_files(&server), before);
        }
        let cross = s3_bundle_verify(&server, &empty_one);
        assert_eq!(cross["state"], "matched");
        assert_eq!(cross["expected_index_matches_current"], false);
        bundle_assert_counts(&cross, [0, 0, 0, 0, 0, 0]);
    }

    /// Bundle /2 preserves closed version pairing, ordered pin bijection and duplicate-safe raw transport.
    #[test]
    fn s3_bundle_two_rejects_intrinsic_corruption_before_positive_comparison() {
        let server = Server::start_api2(false, true);
        let bytes = b"opaque lifecycle source";
        let index = json!({"schema_version":"forge.workspace/2","label":"Closed two","resources":[
            {"key":"alpha","role":"lifecycle-source","path":"alpha.bin"},
            {"key":"beta","role":"lifecycle-source","path":"beta.bin"}]});
        for path in ["alpha.bin", "beta.bin"] {
            std::fs::write(server.project.path().join(path), bytes).unwrap();
        }
        std::fs::write(
            server.project.path().join("forge.workspace.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let before = bundle_project_files(&server);
        let bundle = s3_bundle(&index, &[bytes, bytes]);
        let valid = json!({"bundle":bundle});
        let mut invalid = Vec::new();
        for (pointer, replacement) in [
            ("/bundle/schema_version", json!("forge.workspace-index-bundle/1")),
            ("/bundle/schema_version", json!("forge.workspace-index-bundle/3")),
            ("/bundle/index/schema_version", json!("forge.workspace/1")),
            ("/bundle/content_profile", json!("source-inclusive")),
            ("/bundle/pins/0/size_bytes", json!(10 * 1024 * 1024 + 1)),
            ("/bundle/pins/0/key", json!("unknown-key")),
        ] {
            let mut body = valid.clone();
            *body.pointer_mut(pointer).unwrap() = replacement;
            invalid.push(body);
        }
        let mut unknown = valid.clone();
        unknown["bundle"]["approval"] = json!(true);
        invalid.push(unknown);
        let mut removed = valid.clone();
        removed["bundle"]["pins"].as_array_mut().unwrap().pop();
        invalid.push(removed);
        let mut swapped = valid.clone();
        swapped["bundle"]["pins"].as_array_mut().unwrap().swap(0, 1);
        invalid.push(swapped);
        let mut duplicate = valid.clone();
        duplicate["bundle"]["pins"][1] = duplicate["bundle"]["pins"][0].clone();
        invalid.push(duplicate);
        for body in invalid {
            let (status, error) =
                s3_json(&server, "POST", "/api/v2/project/bundle-verifications", None, &body);
            assert_eq!(status, 400, "{error}");
            bundle_assert_error(&server, &error, "invalid-request");
        }
        s3_raw_duplicates_rejected(&server, &valid, &bundle_fixture_sha256(bytes));
        assert_eq!(s3_bundle_verify(&server, &bundle)["state"], "matched");
        assert_eq!(bundle_project_files(&server), before);
    }
    /// Supplied index2 paths are not opened when current index1 lacks their registrations.
    #[test]
    fn s3_supplied_bundle_two_on_current_one_never_opens_unregistered_paths() {
        let server = Server::start_api2(true, true);
        let directory = server.project.path().join("unregistered-directory");
        std::fs::create_dir(&directory).unwrap();
        let before = std::fs::read(server.project.path().join("forge.workspace.json")).unwrap();
        let supplied = json!({"schema_version":"forge.workspace/2","label":"Supplied registrations","resources":[
            {"key":"missing-source","role":"lifecycle-source","path":"missing.bin"},
            {"key":"private-source","role":"lifecycle-source","path":"unregistered.md"},
            {"key":"nonregular-source","role":"lifecycle-source","path":"unregistered-directory"}]});
        let bundle = s3_bundle(
            &supplied,
            &[b"expected missing bytes", b"different private bytes", b"expected directory bytes"],
        );
        let result = s3_bundle_verify(&server, &bundle);
        assert_eq!(result["state"], "mismatched");
        assert_eq!(result["expected_index_matches_current"], false);
        bundle_assert_counts(&result, [3, 0, 3, 0, 1, 1]);
        for row in result["items"].as_array().unwrap() {
            assert_eq!(row["status"], "not-registered");
            assert_eq!(row["observed_resource_validation_state"], "not-registered");
            assert_eq!(row["reason_codes"], json!(["registration-not-found"]));
        }
        assert_eq!(
            std::fs::read(server.project.path().join("forge.workspace.json")).unwrap(),
            before
        );
        assert_eq!(
            std::fs::read(server.project.path().join("unregistered.md")).unwrap(),
            b"PRIVATE UNREGISTERED CONTENT"
        );
        assert!(directory.is_dir());
        assert!(!server.project.path().join("missing.bin").exists());
        assert!(!result.to_string().contains("PRIVATE UNREGISTERED CONTENT"));
    }

    /// API2 retains exact raw-envelope and checked declared-byte bounds without reading supplied missing files.
    #[test]
    fn s3_api2_bundle_envelope_and_aggregate_boundaries_are_consumed() {
        let server = Server::start_api2(false, true);
        let empty =
            json!({"schema_version":"forge.workspace/2","label":"Budget fixture","resources":[]});
        std::fs::write(
            server.project.path().join("forge.workspace.json"),
            serde_json::to_vec(&empty).unwrap(),
        )
        .unwrap();
        let before = bundle_project_files(&server);
        let valid = json!({"bundle":s3_bundle(&empty, &[])}).to_string();
        let limit = 1024 * 1024;
        let padded = format!("{}{valid}", " ".repeat(limit - valid.len()));
        assert_eq!(padded.len(), limit);
        let (status, _, bytes) = server.request(
            "POST",
            "/api/v2/project/bundle-verifications",
            true,
            None,
            "Content-Type: application/json\r\n",
            &padded,
        );
        assert_eq!(status, 200, "{}", String::from_utf8_lossy(&bytes));
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap()["state"], "matched");
        // Header-only oversize avoids relying on a write/reset race after server rejection.
        let mut stream = TcpStream::connect(&server.host).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        stream.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
        write!(stream, "POST /api/v2/project/bundle-verifications HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", server.host, server.capability, limit + 1).unwrap();
        stream.flush().unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let (headers, body) = response.split_once("\r\n\r\n").unwrap();
        assert_eq!(headers.split_whitespace().nth(1).unwrap(), "413");
        bundle_assert_error(&server, &serde_json::from_str(body).unwrap(), "payload-too-large");
        let resources: Vec<Value> = (0..5).map(|number| json!({"key":format!("expected-{number}"),"role":"lifecycle-source","path":format!("missing-{number}.bin")})).collect();
        let index = json!({"schema_version":"forge.workspace/2","label":"Declared byte budget","resources":resources});
        let mut bundle = s3_bundle(&index, &[b"".as_slice(); 5]);
        let per_pin = 10 * 1024 * 1024;
        for pin in bundle["pins"].as_array_mut().unwrap() {
            pin["size_bytes"] = json!(per_pin);
        }
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-verifications",
            None,
            &json!({"bundle":bundle}),
        );
        assert_eq!(status, 413, "{error}");
        bundle_assert_error(&server, &error, "payload-too-large");
        // These are declared expected-byte counts; no corresponding payload is allocated or supplied path opened.
        bundle["pins"][4]["size_bytes"] = json!(per_pin - bundle_fixture_index_bytes(&index).len());
        let result = s3_bundle_verify(&server, &bundle);
        bundle_assert_counts(&result, [5, 0, 5, 0, 0, 0]);
        assert_eq!(result["state"], "mismatched");
        assert_eq!(bundle_project_files(&server), before);
    }

    /// A denied index capture distinguishes pre-capture authentication/query/major admission.
    #[test]
    fn s3_queries_are_admitted_before_capture_and_never_expand_api1() {
        let server = Server::start_api2(false, true);
        std::fs::create_dir(server.project.path().join("forge.workspace.json")).unwrap();
        let capture = s3_json(&server, "GET", "/api/v2/lifecycle/records", None, &json!({}));
        assert_eq!(capture.0, 403, "{}", capture.1);
        bundle_assert_error(&server, &capture.1, "resource-containment");
        for path in [
            "/api/v2/lifecycle/records?as_of=2025-02-29",
            "/api/v2/lifecycle/records?as_of=2026-10-02&as_of=2026-10-03",
            "/api/v2/lifecycle/records?owner=%20owner",
            "/api/v2/lifecycle/records?state=ready",
            "/api/v2/lifecycle/queue",
            "/api/v2/lifecycle/records/res_11111111111111111111111111111111",
            "/api/v2/framework-impact/comparisons?page_size=201",
            "/api/v2/framework-impact/comparisons/res_11111111111111111111111111111111/findings?priority=critical",
            "/api/v2/framework-impact/comparisons/res_11111111111111111111111111111111/findings?decision_state=satisfied",
            "/api/v2/framework-impact/comparisons/res_11111111111111111111111111111111?owner=hidden",
        ] {
            let (status, value) = s3_json(&server, "GET", path, None, &json!({}));
            assert_eq!(status, 400, "{path}: {value}");
            bundle_assert_error(&server, &value, "invalid-request");
        }
        let too_long = format!("/api/v2/lifecycle/records?owner={}", "x".repeat(4097));
        // The existing raw request-target guard is2048bytes, before decoded filter admission.
        assert_eq!(s3_json(&server, "GET", &too_long, None, &json!({})).0, 401);
        assert_eq!(server.request("GET", "/api/v2/lifecycle/records", false, None, "", "").0, 401);
        assert_eq!(server.request("GET", "/api/v1/lifecycle/records", true, None, "", "").0, 404);
        assert!(server.project.path().join("forge.workspace.json").is_dir());
        assert_eq!(
            std::fs::read(server.project.path().join("unregistered.md")).unwrap(),
            b"PRIVATE UNREGISTERED CONTENT"
        );
        let legacy = Server::launch(false);
        std::fs::create_dir(legacy.project.path().join("forge.workspace.json")).unwrap();
        for path in [
            "/api/v1/lifecycle/records",
            "/api/v1/framework-impact/comparisons",
            "/api/v2/lifecycle/records",
        ] {
            assert_eq!(legacy.request("GET", path, true, None, "", "").0, 404);
        }
    }
    // Source-only appendable integration-test helpers. Parent test module already supplies
    // bundle_fixture_sha256 (immutable702 tests/workspace_cli_test.rs:853). Root may import
    // its parent helper; no private production module is exposed for this fixture.

    /// Write native synthetic Catalog bytes using the existing exercised PRD057 fixture shape.
    fn s3_native_catalog(version: &str, controls: &[(&str, &str)]) -> serde_json::Value {
        serde_json::json!({"catalog":{"uuid":"77777777-7777-4777-8777-777777777777",
        "metadata":{"title":"PRIVATE synthetic framework","last-modified":"2026-08-25T12:00:00Z",
            "version":version,"oscal-version":"1.2.3"},
        "groups":[{"id":"group-1","title":"PRIVATE group","controls":controls.iter().map(|(id, prose)|
            serde_json::json!({"id":id,"title":format!("PRIVATE Control {id}"),"parts":[{"id":format!("{id}_smt"),"name":"statement","prose":prose}]})).collect::<Vec<_>>()}]}})
    }

    /// Write one valid native lifecycle/comparison portfolio for actual authenticated nine-route GETs.
    /// Direct fixture setup is explicit; queries must preserve these bytes and cannot create effects.
    fn write_s3_native_http_fixture(root: &std::path::Path) -> serde_json::Value {
        let old = serde_json::to_vec_pretty(&s3_native_catalog(
            "1.0.0",
            &[
                ("unchanged", "PRIVATE same"),
                ("changed", "PRIVATE old"),
                ("removed", "PRIVATE removed"),
            ],
        ))
        .expect("native old bytes");
        let new = serde_json::to_vec_pretty(&s3_native_catalog(
            "2.0.0",
            &[
                ("unchanged", "PRIVATE same"),
                ("changed", "PRIVATE revised"),
                ("added", "PRIVATE added"),
            ],
        ))
        .expect("native new bytes");
        std::fs::write(root.join("old.json"), &old).expect("owned old fixture");
        std::fs::write(root.join("new.json"), &new).expect("owned new fixture");
        let resource = |path: &str, bytes: &[u8], version: &str| {
            serde_json::json!({"type":"catalog","artifact":path,
        "expected_sha256":bundle_fixture_sha256(bytes),"root_uuid":"77777777-7777-4777-8777-777777777777",
        "document_version":version,"oscal_version":"1.2.3"})
        };
        let manifest = serde_json::json!({"schema_version":"forge.framework-impact/1",
        "old":resource("old.json",&old,"1.0.0"),"new":resource("new.json",&new,"2.0.0"),"mapping_collections":[]});
        std::fs::write(
            root.join("impact.json"),
            serde_json::to_vec(&manifest).expect("exact manifest bytes"),
        )
        .expect("owned impact fixture");
        std::fs::write(root.join("unrelated.bin"), b"PRIVATE unrelated sentinel")
            .expect("owned sentinel");
        let mut rows = vec![
            serde_json::json!({"key":"comparison","role":"framework-impact-manifest","path":"impact.json"}),
            serde_json::json!({"key":"old","role":"oscal-catalog-artifact","path":"old.json"}),
            serde_json::json!({"key":"new","role":"oscal-catalog-artifact","path":"new.json"}),
            serde_json::json!({"key":"unrelated","role":"lifecycle-source","path":"unrelated.bin"}),
        ];
        rows.extend(write_registered_lifecycle_fixture(root, "new.json", &new));
        let index = serde_json::json!({"schema_version":"forge.workspace/2","label":"Synthetic S3 HTTP portfolio","resources":rows});
        std::fs::write(
            root.join("forge.workspace.json"),
            serde_json::to_vec(&index).expect("explicit index bytes"),
        )
        .expect("owned index fixture");
        index
    }

    /// Write explicit synthetic lifecycle registrations bound to supplied native Catalog bytes.
    ///
    /// The caller owns the directory and separately writes/registers the supplied Catalog.
    /// This fixture reads no paths, captures no Root, launches no process, and authenticates
    /// no declared actor. Real authenticated HTTP tests exercise these native fixture bytes.
    fn write_registered_lifecycle_fixture(
        root: &std::path::Path,
        generated_path: &str,
        generated_bytes: &[u8],
    ) -> Vec<serde_json::Value> {
        use forge::lifecycle::record::{
            self, DeclaredRole, FingerprintSet, LifecycleRecord, LifecycleState, NamedHash,
            TransitionEvent,
        };
        assert!(!generated_path.starts_with('/') && !generated_path.contains('\\'));
        assert!(!generated_path.split('/').any(|part| matches!(part, "" | "." | "..")));
        let native: serde_json::Value =
            serde_json::from_slice(generated_bytes).expect("actual native fixture bytes");
        assert_eq!(
            forge::validate::detect_model_type(&native).expect("native Catalog root"),
            forge::OscalModelType::Catalog
        );
        let uuid = native["catalog"]["uuid"].as_str().expect("actual Catalog UUID");
        uuid::Uuid::parse_str(uuid).expect("valid native UUID");
        let source = b"Synthetic lifecycle source fixture.";
        let source_hash = bundle_fixture_sha256(source);
        let generated_hash = bundle_fixture_sha256(generated_bytes);
        let mut record:LifecycleRecord=serde_json::from_value(serde_json::json!({
        "schema_version":"forge.policy-lifecycle/2",
        "policy":{"policy_key":"http-policy","version_key":"v1","title":"PRIVATE fixture title",
            "owner_keys":["fixture-owner"],"source":{"path":"lifecycle-source.bin","sha256":source_hash},
            "generated_artifacts":[{"path":generated_path,"sha256":generated_hash,"oscal_type":"catalog","root_uuid":uuid}]},
        "parties":[{"key":"fixture-owner","roles":["owner","reviewer","approver"]}],
        "approval_policy":{"schema_version":"forge.approval-policy/1",
            "required_roles":[{"role":"reviewer","count":1},{"role":"approver","count":1}],"separation":{}},
        "review":{"cadence_days":30,"next_review_date":"2026-10-12","due_soon_days":7,"timezone_policy":"date-only"},
        "state":"draft","history":[]
    })).expect("typed lifecycle fixture");
        record::validate(&record).expect("intrinsic draft");
        for (next, role, time) in [
            (LifecycleState::InReview, DeclaredRole::Reviewer, "2026-10-01T00:00:00Z"),
            (LifecycleState::Approved, DeclaredRole::Approver, "2026-10-02T00:00:00Z"),
        ] {
            let mut event = TransitionEvent {
                sequence: u32::try_from(record.history.len() + 1).expect("small fixture"),
                event_id: String::new(),
                legacy_event_id: None,
                previous_state: record.state,
                next_state: next,
                actor_key: "fixture-owner".to_owned(),
                declared_role: role,
                timestamp: time.to_owned(),
                rationale: "PRIVATE recorded rationale".to_owned(),
                fingerprints: FingerprintSet {
                    source_sha256: record.policy.source.sha256.clone(),
                    generated_artifacts: vec![NamedHash {
                        path: generated_path.to_owned(),
                        sha256: bundle_fixture_sha256(generated_bytes),
                    }],
                },
                assertions: vec![],
                impact_finding_ids: vec![],
                replacement: None,
            };
            event.event_id =
                record::event_id(&record, &event).expect("actual deterministic event ID");
            record.state = next;
            record.history.push(event);
            record::validate(&record).expect("intrinsic declared history");
        }
        std::fs::write(root.join("lifecycle-source.bin"), source).expect("owned source fixture");
        std::fs::write(
            root.join("lifecycle-record.json"),
            serde_json::to_vec(&record).expect("typed record bytes"),
        )
        .expect("owned record fixture");
        vec![
            serde_json::json!({"key":"http-lifecycle-source","role":"lifecycle-source","path":"lifecycle-source.bin"}),
            serde_json::json!({"key":"http-lifecycle-record","role":"lifecycle-record","path":"lifecycle-record.json"}),
        ]
    }

    /// Exercise all nine real authenticated routes over explicit native captured inputs without writes.
    #[test]
    fn s3_native_nine_reads_preserve_snapshot_counts_redaction_and_files() {
        let server = Server::start_api2(false, true);
        write_s3_native_http_fixture(server.project.path());
        let before = bundle_project_files(&server);
        let (status, records) =
            s3_json(&server, "GET", "/api/v2/lifecycle/records", None, &json!({}));
        assert_eq!(status, 200, "{records}");
        assert_eq!(records["counts"]["registered_records"], 1);
        assert!(records["page"]["items"][0]["derived_status"].is_null());
        let record = records["page"]["items"][0]["record_id"].as_str().unwrap();
        let (status, comparisons) =
            s3_json(&server, "GET", "/api/v2/framework-impact/comparisons", None, &json!({}));
        assert_eq!(status, 200, "{comparisons}");
        assert_eq!(comparisons["counts"]["registered_comparisons"], 1);
        assert_eq!(comparisons["page"]["items"][0]["freshness"], "not-computed");
        let comparison = comparisons["page"]["items"][0]["comparison_id"].as_str().unwrap();
        let paths = [
            "/api/v2/lifecycle/records".to_owned(),
            format!("/api/v2/lifecycle/records/{record}?as_of=2026-10-02"),
            format!("/api/v2/lifecycle/records/{record}/history"),
            "/api/v2/lifecycle/queue?as_of=2026-10-02".to_owned(),
            "/api/v2/framework-impact/comparisons".to_owned(),
            format!("/api/v2/framework-impact/comparisons/{comparison}"),
            format!("/api/v2/framework-impact/comparisons/{comparison}/changes"),
            format!("/api/v2/framework-impact/comparisons/{comparison}/findings"),
            format!("/api/v2/framework-impact/comparisons/{comparison}/prior-dispositions"),
        ];
        for path in paths {
            let (status, headers, raw) = server.request("GET", &path, true, None, "", "");
            assert_eq!(status, 200, "{path}: {}", String::from_utf8_lossy(&raw));
            assert!(headers.contains("cache-control: no-store"));
            let value: Value = serde_json::from_slice(&raw).unwrap();
            assert_eq!(value["snapshot_version"], records["snapshot_version"]);
            assert_eq!(value["resource_version"].as_str().unwrap().len(), 64);
            let text = String::from_utf8(raw).unwrap();
            for secret in [
                "PRIVATE",
                "recorded rationale",
                "Synthetic lifecycle source fixture.",
                "rationale",
                "unregistered.md",
            ] {
                assert!(!text.contains(secret), "{path}: leaked {secret}");
            }
        }
        let (_, detail) = s3_json(
            &server,
            "GET",
            &format!("/api/v2/lifecycle/records/{record}?as_of=2026-10-02"),
            None,
            &json!({}),
        );
        assert_eq!(detail["state"], "approved");
        assert_eq!(detail["derived_status"], "approved");
        assert!(detail["trust_boundary"].as_str().unwrap().contains("not authenticated"));
        let (_, history) = s3_json(
            &server,
            "GET",
            &format!("/api/v2/lifecycle/records/{record}/history"),
            None,
            &json!({}),
        );
        assert_eq!(history["counts"]["total_events"], 2);
        let (_, comparison_detail) = s3_json(
            &server,
            "GET",
            &format!("/api/v2/framework-impact/comparisons/{comparison}"),
            None,
            &json!({}),
        );
        assert_eq!(comparison_detail["freshness"], "captured-current");
        assert_eq!(comparison_detail["summary"]["old_controls"], 3);
        assert_eq!(comparison_detail["summary"]["new_controls"], 3);
        let (status, hidden) = s3_json(
            &server,
            "GET",
            &format!(
                "/api/v2/framework-impact/comparisons/{comparison}/findings?owner=no-such-owner"
            ),
            None,
            &json!({}),
        );
        assert_eq!(status, 200, "{hidden}");
        assert_eq!(hidden["page"]["total_matching"], 0);
        assert_eq!(hidden["full_summary"], comparison_detail["summary"]);
        assert_eq!(hidden["counts"]["total_findings"], comparison_detail["summary"]["findings"]);
        assert_eq!(hidden["comparison_version"], comparison_detail["resource_version"]);
        assert_ne!(hidden["resource_version"], hidden["comparison_version"]);
        assert_eq!(bundle_project_files(&server), before);
    }

    /// Real cursors cannot cross page size, endpoint, filters or changed capture inputs.
    #[test]
    fn s3_native_http_cursors_and_missing_closure_fail_completely() {
        let server = Server::start_api2(false, true);
        let mut index = write_s3_native_http_fixture(server.project.path());
        let (_, comparisons) =
            s3_json(&server, "GET", "/api/v2/framework-impact/comparisons", None, &json!({}));
        let id = comparisons["page"]["items"][0]["comparison_id"].as_str().unwrap();
        let base = format!("/api/v2/framework-impact/comparisons/{id}");
        let (status, first) =
            s3_json(&server, "GET", &format!("{base}/changes?page_size=1"), None, &json!({}));
        assert_eq!(status, 200, "{first}");
        assert_eq!(first["counts"]["total_changes"], 4);
        let cursor = first["page"]["next_cursor"].as_str().unwrap();
        let (status, next) = s3_json(
            &server,
            "GET",
            &format!("{base}/changes?page_size=1&cursor={cursor}"),
            None,
            &json!({}),
        );
        assert_eq!(status, 200, "{next}");
        assert_eq!(next["counts"], first["counts"]);
        assert_ne!(next["page"]["items"][0], first["page"]["items"][0]);
        for path in [
            format!("{base}/changes?page_size=2&cursor={cursor}"),
            format!("{base}/changes?page_size=1&change_class=added&cursor={cursor}"),
            format!("{base}/findings?page_size=1&cursor={cursor}"),
        ] {
            let (status, error) = s3_json(&server, "GET", &path, None, &json!({}));
            assert_eq!(status, 409, "{path}: {error}");
            bundle_assert_keys(&error, &["code", "message", "retryable", "resource_version"]);
            assert_eq!(error["resource_version"].as_str().unwrap().len(), 64);
            let mut safe = error.clone();
            safe.as_object_mut().unwrap().remove("resource_version");
            bundle_assert_error(&server, &safe, "version-conflict");
        }
        std::fs::write(
            server.project.path().join("unrelated.bin"),
            b"changed PRIVATE unrelated sentinel",
        )
        .unwrap();
        assert_eq!(
            s3_json(
                &server,
                "GET",
                &format!("{base}/changes?page_size=1&cursor={cursor}"),
                None,
                &json!({})
            )
            .0,
            409
        );
        index["resources"].as_array_mut().unwrap().retain(|row| row["key"] != "new");
        std::fs::write(
            server.project.path().join("forge.workspace.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
        let before = bundle_project_files(&server);
        let (status, error) = s3_json(&server, "GET", &base, None, &json!({}));
        assert_eq!(status, 422, "{error}");
        bundle_assert_error(&server, &error, "validation-failed");
        assert_eq!(bundle_project_files(&server), before);
    }
    /// Poll a real accepted API2 job to its bounded terminal result without synthetic progress.
    fn s6_poll(server: &Server, accepted: &Value) -> Value {
        let id = accepted["operation_id"].as_str().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        loop {
            let (status, operation) =
                s3_json(server, "GET", &format!("/api/v2/operations/{id}"), None, &json!({}));
            assert_eq!(status, 200, "{operation}");
            if !matches!(operation["state"].as_str(), Some("pending" | "running")) {
                assert_eq!(operation["state"], "succeeded", "{operation}");
                return operation;
            }
            assert!(std::time::Instant::now() < deadline, "API2 preparation did not settle");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Create valid existing incoming files and a strict metadata request without source hydration.
    fn s6_incoming_request(server: &Server, target: u8) -> Value {
        let source = b"# Incoming policy\n\n## Rules\nS6 PRIVATE SOURCE SENTINEL\n";
        std::fs::write(server.project.path().join("incoming.md"), source).unwrap();
        let index = json!({"schema_version":"forge.workspace/1","label":"Explicit replacement",
            "resources":[{"key":"incoming","role":"policy-source","path":"incoming.md"}]});
        json!({"bundle":bundle_fixture(&index, &[source]), "target_index_schema_version":target,
            "acknowledge_index_replacement":true})
    }

    /// Native export retries retain one accepted job; publication and JSON download remain separate.
    #[test]
    fn s6_metadata_export_requires_commit_and_private_json_family() {
        let server = Server::start_api2(true, false);
        let before = bundle_project_files(&server);
        let request = json!({"target_path":"bundle.json","acknowledge_sensitive_metadata":true});
        let (status, accepted) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-exports",
            Some("native-control-s6-export"),
            &request,
        );
        assert_eq!(status, 202, "{accepted}");
        let operation = s6_poll(&server, &accepted);
        let preview = &operation["result"]["preview"];
        assert_eq!(
            operation["result"]["redaction_summary"]["removed_categories"],
            json!(["source-excerpts"])
        );
        assert_eq!(bundle_project_files(&server), before);
        let id = operation["operation_id"].as_str().unwrap();
        let download = format!("/api/v2/project/bundle-exports/{id}/download");
        assert_eq!(server.request("GET", &download, true, None, "", "").0, 404);
        s3_commit(&server, preview, "native-control-s6-export-commit");
        let (status, headers, bytes) = server.request("GET", &download, true, None, "", "");
        assert_eq!(status, 200);
        assert!(headers.contains("content-type: application/json"));
        assert!(headers.contains("attachment; filename=\"forge-workspace-index-and-hashes.json\""));
        assert_eq!(std::fs::read(server.project.path().join("bundle.json")).unwrap(), bytes);
        assert_eq!(
            serde_json::from_slice::<Value>(&bytes).unwrap()["content_profile"],
            "index-and-hashes"
        );
        assert!(!String::from_utf8_lossy(&bytes).contains("human-supplied clause"));
        assert_eq!(
            server.request("GET", &format!("/api/v2/exports/{id}/download"), true, None, "", "").0,
            404
        );
        let (status, retry) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-exports",
            Some("native-control-s6-export"),
            &request,
        );
        assert_eq!(status, 202);
        assert_eq!(retry, accepted);
        let changed = json!({"target_path":"different.json","acknowledge_sensitive_metadata":true});
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-exports",
            Some("native-control-s6-export"),
            &changed,
        );
        assert_eq!(status, 409);
        assert_eq!(error["code"], "idempotency-key-conflict");
        std::fs::write(server.project.path().join("bundle.json"), b"changed metadata").unwrap();
        assert_eq!(server.request("GET", &download, true, None, "", "").0, 409);
    }

    /// Complete membership is disclosed before one exact index write; removed source files remain intact.
    #[test]
    fn s6_index_import_discloses_complete_replacement_and_replays_original_receipt() {
        let server = Server::start_api2(true, false);
        let request = s6_incoming_request(&server, 2);
        let before = bundle_project_files(&server);
        let (status, reply) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-imports",
            Some("native-control-s6-import"),
            &request,
        );
        assert_eq!(status, 200, "{reply}");
        bundle_assert_keys(&reply, &["validation", "preview", "replacement"]);
        assert_eq!(reply["replacement"]["previous_index"]["schema_version"], "forge.workspace/1");
        assert_eq!(reply["replacement"]["proposed_index"]["schema_version"], "forge.workspace/2");
        assert_eq!(reply["replacement"]["removed_resource_keys"], json!(["policy"]));
        assert_eq!(reply["replacement"]["consumed_file_count"], 3);
        assert_eq!(reply["preview"]["input_hashes"].as_array().unwrap().len(), 2);
        assert_eq!(bundle_project_files(&server), before);
        let (status, retry) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-imports",
            Some("native-control-s6-import"),
            &request,
        );
        assert_eq!(status, 200);
        assert_eq!(retry, reply);
        s3_commit(&server, &reply["preview"], "native-control-s6-import-commit");
        let committed = std::fs::read(server.project.path().join("forge.workspace.json")).unwrap();
        assert_eq!(
            bundle_fixture_sha256(&committed),
            reply["replacement"]["proposed_index_sha256"]
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&committed).unwrap(),
            reply["replacement"]["proposed_index"]
        );
        let after = bundle_project_files(&server);
        for (path, bytes) in &before {
            if path != "forge.workspace.json" {
                assert_eq!(after.iter().find(|(name, _)| name == path).unwrap().1, *bytes);
            }
        }
        let (status, retry) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-imports",
            Some("native-control-s6-import"),
            &request,
        );
        assert_eq!(status, 200);
        assert_eq!(retry, reply);
        let (status, used) = s3_json(
            &server,
            "GET",
            &format!(
                "/api/v2/effects/previews/{}",
                reply["preview"]["preview_id"].as_str().unwrap()
            ),
            None,
            &json!({}),
        );
        assert_eq!(status, 409);
        assert_eq!(used["code"], "receipt-reused");
    }

    /// Actual simultaneous requests produce one original receipt or a truthful pending retry.
    #[test]
    fn s6_simultaneous_import_requests_keep_single_original_receipt() {
        let server = Server::start_api2(true, false);
        let request = s6_incoming_request(&server, 1);
        let before = bundle_project_files(&server);
        let barrier = std::sync::Barrier::new(2);
        let replies = std::thread::scope(|scope| {
            let one = scope.spawn(|| {
                barrier.wait();
                s3_json(
                    &server,
                    "POST",
                    "/api/v2/project/bundle-imports",
                    Some("native-control-s6-race"),
                    &request,
                )
            });
            let two = scope.spawn(|| {
                barrier.wait();
                s3_json(
                    &server,
                    "POST",
                    "/api/v2/project/bundle-imports",
                    Some("native-control-s6-race"),
                    &request,
                )
            });
            [one.join().unwrap(), two.join().unwrap()]
        });
        let success =
            replies.iter().find(|(status, _)| *status == 200).expect("one complete import preview");
        for (status, value) in &replies {
            if *status == 200 {
                assert_eq!(value, &success.1);
            } else {
                assert_eq!(*status, 409);
                assert_eq!(value["code"], "bundle-preparation-in-progress");
            }
        }
        let (status, retry) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-imports",
            Some("native-control-s6-race"),
            &request,
        );
        assert_eq!(status, 200);
        assert_eq!(retry, success.1);
        assert_eq!(bundle_project_files(&server), before);
    }

    /// Incoming exact-byte drift consumes a matched receipt without changing the prior index.
    #[test]
    fn s6_import_commit_rechecks_unregistered_input_and_preserves_previous_index() {
        let server = Server::start_api2(true, false);
        let request = s6_incoming_request(&server, 1);
        let old = std::fs::read(server.project.path().join("forge.workspace.json")).unwrap();
        let (status, reply) = s3_json(
            &server,
            "POST",
            "/api/v2/project/bundle-imports",
            Some("native-control-s6-drift"),
            &request,
        );
        assert_eq!(status, 200, "{reply}");
        std::fs::write(
            server.project.path().join("incoming.md"),
            b"# Changed\n\nDifferent supplied clause\n",
        )
        .unwrap();
        let commit = json!({"receipt":reply["preview"]["receipt"]["token"],"observed_version":reply["preview"]["target_version"],"confirmed":true});
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/effects/commits",
            Some("native-control-s6-drift-commit"),
            &commit,
        );
        assert_eq!(status, 409, "{error}");
        assert_eq!(error["code"], "version-conflict");
        assert_eq!(std::fs::read(server.project.path().join("forge.workspace.json")).unwrap(), old);
        let (status, error) = s3_json(
            &server,
            "POST",
            "/api/v2/effects/commits",
            Some("native-control-s6-drift-reuse"),
            &commit,
        );
        assert_eq!(status, 409);
        assert_eq!(error["code"], "receipt-reused");
    }

    /// API1 lacks these routes and read-only API2 cannot prepare either authoritative effect.
    #[test]
    fn s6_bundle_effect_routes_preserve_namespace_and_readonly_authority() {
        let one = Server::launch_mode(false, false);
        let export = json!({"target_path":"bundle.json","acknowledge_sensitive_metadata":true});
        assert_eq!(
            one.json(
                "POST",
                "/api/v1/project/bundle-exports",
                Some("native-control-s6-old"),
                &export
            )
            .0,
            404
        );
        let server = Server::start_api2(true, true);
        let request = s6_incoming_request(&server, 1);
        let before = bundle_project_files(&server);
        for (path, value) in [
            ("/api/v2/project/bundle-exports", &export),
            ("/api/v2/project/bundle-imports", &request),
        ] {
            let (status, error) =
                s3_json(&server, "POST", path, Some("native-control-s6-readonly"), value);
            assert_eq!(status, 403);
            assert_eq!(error["code"], "read-only-session");
        }
        assert_eq!(
            s3_json(&server, "GET", "/api/v2/project/bundle-preview", None, &json!({})).0,
            200
        );
        assert_eq!(bundle_project_files(&server), before);
    }
}

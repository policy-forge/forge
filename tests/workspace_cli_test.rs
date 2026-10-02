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
        let mut process = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = process.stdout.take().unwrap();
        // A launch deadline prevents a broken server from hanging the suite.
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = std::io::BufReader::new(stdout).read_line(&mut line).map(|_| line);
            let _ = send.send(result);
        });
        let result = receive.recv_timeout(Duration::from_secs(20));
        if result.is_err() {
            let _ = process.kill();
            let _ = process.wait();
            panic!("workspace launch timed out");
        }
        let line = result.unwrap().unwrap();
        let descriptor: Value =
            serde_json::from_str(&line).expect("machine descriptor must be JSON");
        let host =
            descriptor["base_url"].as_str().unwrap().strip_prefix("http://").unwrap().to_owned();
        let capability = descriptor["capability"].as_str().unwrap().to_owned();
        assert!(host.starts_with("127.0.0.1:"));
        assert_eq!(descriptor["mode"], "machine");
        assert_eq!(descriptor["read_only"], read_only);
        Self { process, host, capability, project }
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

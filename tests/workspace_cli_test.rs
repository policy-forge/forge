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
    let (_, error) = server.json(
        "POST",
        "/api/v1/resources/register",
        Some("hostile-path-0001"),
        &json!({"role":"policy-source","path":"../PRIVATE-SECRET.md"}),
    );
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

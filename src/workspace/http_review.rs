//! S-1 interactive response-file client: one confined synchronous write through
//! the same shared review `respond()` path the CLI uses. The portable response
//! wire contract is unchanged; asserted identities never grant domain approval.

use super::*;
use crate::review::commands::{RespondOptions, Supersedes};
use crate::review::wire::Disposition;
use crate::workspace::{index, root};
use serde::Deserialize;
use std::path::Path;

/// Closed family selector mirroring the explicit CLI subcommands; the selected
/// family's own decoder still refuses a mismatched queue original.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ReviewFamily {
    /// Mapping/applicability `/1` exchange (`forge review respond`).
    Review,
    /// Lifecycle `/2` exchange.
    Lifecycle,
    /// Authoring `/3` exchange.
    Authoring,
    /// Edit-capable `/4` exchange.
    Edit,
}

/// Explicit self-supersession pair; omission never guesses a predecessor.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SupersedesRequest {
    response_id: String,
    raw_sha256: String,
}

/// Closed request envelope; every path is project-root relative and confined.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewResponseRequest {
    family: ReviewFamily,
    queue: String,
    rationale_file: String,
    item_key: String,
    reviewer_key: String,
    reviewer_role: String,
    disposition: Disposition,
    responded_at: String,
    response_id: String,
    #[serde(default)]
    abstention_reason: Option<String>,
    #[serde(default)]
    supersedes: Option<SupersedesRequest>,
    output: String,
}

/// Fixed safe refusal; no review parser prose, private rationale or path escapes.
fn refused() -> Error {
    Error::new(
        "review-response-refused",
        "The closed review contract refused this response declaration.",
        false,
    )
}

/// Publish one immutable response file through the shared review command path.
/// Session write scope, exact-Host pin and loopback transport are already
/// enforced by the parent guard; this handler adds project confinement and the
/// review contract's own capture/publication fences.
pub(super) fn write_response(
    state: &State,
    payload: &Value,
    read_deadline: Option<Instant>,
) -> Result<Response<Full<Bytes>>> {
    let request: ReviewResponseRequest =
        serde_json::from_value(payload.clone()).map_err(|_| Error::invalid())?;
    for path in [&request.queue, &request.rationale_file, &request.output] {
        index::validate_path(path)?;
    }
    // Participating exclusive lease: the response is a new project file, and
    // concurrent workspace effects must not interleave with its publication.
    let _io = source::project_write(state)?;
    let root_path = state.root.project_path();
    let supersedes = request
        .supersedes
        .as_ref()
        .map(|prior| Supersedes { response_id: &prior.response_id, raw_sha256: &prior.raw_sha256 });
    let options = RespondOptions {
        project_root: root_path,
        queue: Path::new(&request.queue),
        rationale_file: Path::new(&request.rationale_file),
        item_key: &request.item_key,
        reviewer_key: &request.reviewer_key,
        reviewer_role: &request.reviewer_role,
        disposition: request.disposition,
        responded_at: &request.responded_at,
        response_id: &request.response_id,
        abstention_reason: request.abstention_reason.as_deref(),
        supersedes,
        proposed_patch: None,
        output: Path::new(&request.output),
    };
    let mut control = ReadControl { state, deadline: read_deadline, interruption: None };
    let result = match request.family {
        ReviewFamily::Review => crate::review::commands::respond(&options, &mut control),
        ReviewFamily::Lifecycle => {
            crate::review::lifecycle_commands::respond(&options, &mut control)
        }
        ReviewFamily::Authoring => {
            crate::review::authoring_commands::respond(&options, &mut control)
        }
        ReviewFamily::Edit => crate::review::edit_commands::respond(&options, &mut control),
    };
    match result {
        Ok(()) => json_response_for(
            state.api_major,
            json!({"output": request.output, "response_id": request.response_id}),
            "ReviewResponseWriteResponse",
        ),
        Err(crate::review::commands::CommandError::Publication) => Err(root::publish_failed()),
        Err(_) => Err(refused()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::applicability_capture::tests::Fixture;
    use crate::review::commands::InitOptions;
    use crate::review::decode::{self, ContractLedger};
    use crate::workspace::preparation::NoopControl;
    use std::sync::{Arc, Mutex, RwLock, atomic::AtomicBool};

    const RESPONSE_ID: &str = "66666666-6666-4666-8666-666666666666";

    /// Production always supplies a real admission budget; `None` means expired.
    fn budget() -> Option<Instant> {
        Instant::now().checked_add(Duration::from_secs(10))
    }

    /// One machine-write session over a genuine initialized review fixture root.
    fn machine_state(root: Root, read_only: bool) -> State {
        State {
            api_major: ApiMajor::V2,
            root,
            project_io: RwLock::new(()),
            restores: None,
            host: "127.0.0.1:1".into(),
            origin: "http://127.0.0.1:1".into(),
            session: Mutex::new(Session::new(Mode::Machine, read_only, None).unwrap()),
            stopped: AtomicBool::new(false),
            rate: Mutex::new((Instant::now(), 0)),
            effects: Mutex::new(crate::workspace::effects::Store::default()),
            work: Arc::new(tokio::sync::Semaphore::new(2)),
            jobs: Arc::new(tokio::sync::Semaphore::new(1)),
        }
    }

    /// Initialize a genuine /1 queue inside the fixture using the shared command path.
    fn initialized_queue(fixture: &Fixture) {
        let items = vec![json!({
            "key": "item-c1", "subject_id": "c1", "policy_key": "policy",
            "author_keys": ["source-author"],
            "assignments": [
                {"reviewer_key":"a-reviewer","role_key":"review"},
                {"reviewer_key":"b-reviewer","role_key":"review"}
            ], "due_at": null
        })];
        std::fs::write(
            fixture.root().join("review-policy.json"),
            serde_json::to_vec_pretty(&json!({
                "schema_version":"forge.review-init/1", "roles":[{"key":"review"}],
                "reviewers":[
                    {"key":"a-reviewer","role_keys":["review"]},
                    {"key":"b-reviewer","role_keys":["review"]}
                ], "policies":[{
                    "key":"policy", "seats":[{"role_key":"review","count":2}],
                    "substitutions":[], "abstention_rule":"nonapproving",
                    "empty_abstention_reasons":[], "author_separation":"declared-keys"
                }], "items":items
            }))
            .unwrap(),
        )
        .unwrap();
        crate::review::commands::init(
            &InitOptions {
                project_root: fixture.root(),
                sources: Path::new("locator.json"),
                policy: Path::new("review-policy.json"),
                queue_id: "55555555-5555-4555-8555-555555555555",
                created_at: "2026-10-04T10:00:00Z",
                output: Path::new("queue.json"),
            },
            &mut NoopControl,
        )
        .unwrap();
        std::fs::write(fixture.root().join("rationale.txt"), b"PRIVATE-REVIEW-RATIONALE").unwrap();
    }

    fn request_body(output: &str) -> Value {
        json!({
            "family":"review", "queue":"queue.json", "rationale_file":"rationale.txt",
            "item_key":"item-c1", "reviewer_key":"a-reviewer", "reviewer_role":"review",
            "disposition":"approve", "responded_at":"2026-10-04T12:00:00Z",
            "response_id":RESPONSE_ID, "output":output
        })
    }

    /// S-1 round-trip: the route publishes the same portable response the CLI
    /// `respond()` path produces, decodable by the closed review decoder.
    #[test]
    fn response_file_write_round_trips_through_shared_respond_path() {
        let fixture = Fixture::new(false);
        initialized_queue(&fixture);
        let state = machine_state(Root::open(fixture.root()).unwrap(), false);
        let response = write_response(&state, &request_body("response.json"), budget()).unwrap();
        assert_eq!(response.status(), 200);
        let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let bytes = runtime.block_on(response.into_body().collect()).unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["output"], "response.json");
        assert_eq!(body["response_id"], RESPONSE_ID);
        let raw = std::fs::read(fixture.root().join("response.json")).unwrap();
        let closed =
            decode::decode_response(&raw, &mut ContractLedger::default(), &mut NoopControl)
                .unwrap();
        let document = closed.document();
        assert_eq!(document.response_id, RESPONSE_ID);
        assert_eq!(document.item_key, "item-c1");
        assert_eq!(document.queue_id, "55555555-5555-4555-8555-555555555555");
        // The private rationale round-trips into the response (it is a response
        // field); exports, not the response file, are what redact it.
        assert_eq!(document.rationale, "PRIVATE-REVIEW-RATIONALE");
    }

    /// Confinement refusal: absolute/traversal paths never reach the review capture.
    #[test]
    fn response_write_refuses_unconfined_paths() {
        let fixture = Fixture::new(false);
        initialized_queue(&fixture);
        let state = machine_state(Root::open(fixture.root()).unwrap(), false);
        for path in ["../escape.json", "/absolute.json", "a/../b.json", ".hidden.json"] {
            let error = write_response(&state, &request_body(path), budget()).unwrap_err();
            assert_eq!(error.code, "resource-containment", "{path}");
        }
        assert!(!fixture.root().join("response.json").exists());
    }

    /// Closed-contract refusal: a mismatched family/queue never publishes, and the
    /// safe diagnostic carries no private rationale or parser prose.
    #[test]
    fn response_write_refuses_mismatched_family_without_leaking_prose() {
        let fixture = Fixture::new(false);
        initialized_queue(&fixture);
        let state = machine_state(Root::open(fixture.root()).unwrap(), false);
        let mut body = request_body("response.json");
        body["family"] = json!("lifecycle");
        let error = write_response(&state, &body, budget()).unwrap_err();
        assert_eq!(error.code, "review-response-refused");
        assert!(!error.message.contains("PRIVATE-REVIEW-RATIONALE"));
        assert!(!fixture.root().join("response.json").exists());
    }

    /// No-replace publication: an existing destination is refused, never overwritten.
    #[test]
    fn response_write_refuses_existing_destination() {
        let fixture = Fixture::new(false);
        initialized_queue(&fixture);
        std::fs::write(fixture.root().join("response.json"), b"ORIGINAL").unwrap();
        let state = machine_state(Root::open(fixture.root()).unwrap(), false);
        let error = write_response(&state, &request_body("response.json"), budget()).unwrap_err();
        assert_eq!(error.code, "invalid-request");
        assert_eq!(std::fs::read(fixture.root().join("response.json")).unwrap(), b"ORIGINAL");
    }

    /// Read-only sessions never write a response file: the guard's mutation scope
    /// check refuses before any project I/O, exactly as for other write routes.
    #[test]
    fn response_write_is_refused_in_read_only_session() {
        let fixture = Fixture::new(false);
        initialized_queue(&fixture);
        let state = machine_state(Root::open(fixture.root()).unwrap(), true);
        let token = state.session.lock().unwrap().machine_capability().unwrap();
        let error = state.session.lock().unwrap().authorize(&token, false, true).unwrap_err();
        assert_eq!(error.code, "read-only-session");
        assert!(!fixture.root().join("response.json").exists());
    }

    /// An invalid capability never authorizes the write; the guard refuses first.
    #[test]
    fn response_write_requires_a_valid_capability() {
        let fixture = Fixture::new(false);
        initialized_queue(&fixture);
        let state = machine_state(Root::open(fixture.root()).unwrap(), false);
        let error =
            state.session.lock().unwrap().authorize(&"0".repeat(64), false, true).unwrap_err();
        assert_eq!(error.code, "unauthorized");
        assert!(!fixture.root().join("response.json").exists());
    }
}

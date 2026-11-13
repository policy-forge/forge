# Staged source larger-capacity qualification

Date: 2026-11-12 (America/Los_Angeles). Branch `feat/restore-staged-capacity`,
base `b4226bb`. This receipt records the agent-runnable remainder of ticket #42
("staged capacity follow-through until qualified"). It closes no requirement,
records no approval, and authorizes no merge, publication or external contact.

## Finding: evidence, not missing code

The gap audit in ticket comment 1302 is confirmed against current `main`. The
staged API2.4 transport implementation (`http_staged.rs`, `source_transfers.rs`,
`staged_source_bundles.rs`) is merged and complete. Shared bounded retention
including unresolved reservations is DONE, and deterministic restart recovery is
covered by `restart_after_interrupted_publication_rolls_back_owned_targets`. The
one open agent-runnable item was:

> Staged capacity follow-through OPEN (not attempted) — Bounded staged API2.4
> transport is on main. Larger-capacity qualification remains open.

The existing native staged workflow only ever exercised a ~1.2 MiB artifact; the
10 MiB logical transport cap, the 20 MiB shared retention pool boundary, and
unresolved-reservation retention had never been qualified natively. No production
code change was required. This slice delivers that native qualification plus a
prepared (not run) Windows harness.

## Native macOS larger-capacity qualification

`scripts/test_workspace_staged_capacity_workflow.py` reuses the maintained client
and the existing staged workflow helpers. It ran five consecutive times against
the release binary (`sha256 3e5715d0…`, Rust 1.99.1, `aarch64-apple-darwin`), each
passing all eight fixed checks with two normal session exits (5/5 robustness
confirmation; the machine receipt binds each run):

| Check | Observation |
|---|---|
| full-capacity-stage-create-and-ready | A stage at the exact 10 MiB logical cap creates and fills across 320 parts; readiness grants no write. |
| logical-cap-boundary-exact-and-one-over | One byte over the cap is refused at parse (`invalid-request`) before pool admission; the exact cap is admitted by the first check. |
| unresolved-reservation-blocks-shared-pool-overflow | A held unresolved 10 MiB reservation (zero parts received) blocks a second full-cap stage (`payload-too-large`) against the one shared 20 MiB pool. |
| discard-releases-unresolved-reservation | Discarding the unresolved reservation releases its charge; a new full-cap stage is then admitted. |
| conservative-shared-admission-refuses-below-cap-preview | An 8 MiB artifact (4 MiB opaque content, below the 10 MiB transport cap) transports completely, but its private plan (2×content + 4 MiB journal reserve) plus held stage raw exceeds the shared pool, so preview is refused (`invalid-request`, session-retention) with no project byte change. |
| larger-restorable-preview-and-exact-commit | A 3 MiB opaque source previews valid and commits; exact bytes restore, the index publishes last, and a removed-registration foreign file is retained. |
| abnormal-termination-durable-outcome-survives-restart | After a settled commit, SIGKILL and a fresh writable restart preserve the exact bytes and the known committed outcome. |
| fresh-read-only-restart-outcome-and-write-refusal | A fresh read-only restart reads the known committed outcome and refuses staged writes (`read-only-session`). |

The conservative-refusal check qualifies the documented behavior that shared
admission may refuse an artifact below the transport cap. The private-plan charge
is `2 × capture_bytes + 4 MiB` journal reserve (`root_transaction.rs`
`private_charge`), admitted against the same 20 MiB pool that holds the stage raw.

Native interrupted-publication rollback is qualified by the deterministic Rust
control `restart_after_interrupted_publication_rolls_back_owned_targets` (both
before- and after-index interruption, foreign replacement preserved). There is no
production phase-fault hook for racy native mid-restore injection, so the native
script terminates only after a settled commit and asserts durable recovery; it
does not claim a racy native crash-injection observation.

### SIGKILL restart race: root cause and fix

An earlier revision terminated the session as soon as the public restore status
read `succeeded/committed`. That status is projected at the `commit-decided`
phase, before cleanup and the terminal journal persist
(`root_transaction.rs` `safe_outcome`). A SIGKILL landing inside the terminal
`NativeJournal::replace` leaves `transactions.next` on disk, and
`NativeJournal::open` then fails closed permanently (`recovery-required`) until
that file is resolved — no launch retry can succeed. One of four early runs
failed exactly this way (journal forensics: `commit-decided`,
`cleanup_verified: false`, `transactions.next` present).

The flock itself is not the race: on macOS a SIGKILL'd process releases its
flock synchronously at exit (probed: acquirable within 0.2 ms after reap, 20/20
trials). The fix is the kill point, not a sleep: `larger_restore` now polls
until `cleanup_state == "verified"` — the terminal record is fsync'd — before
the abnormal termination, exactly as the maintained staged workflow's
`restore_confirm` does. The earlier settle-sleep/bounded-retry shim around
restart was removed; a plain fresh `session` launch is correct once the commit
is settled. The five-run confirmation above ran with this final script.

## Deterministic fixture controls

`scripts/test_workspace_staged_capacity_controls.py` (5 passed, 0 failed) proves
the new fixture builders are strict Bundle4 with exact ordered identities, that
the logical-cap padding is exact and one-over is client-refused, that the refused
content is below the transport cap, and that the check identities are unique and
fixed. These controls launch no process; the native workflow qualifies the server.

## Windows preparation (prep only — no Windows run)

`scripts/test_workspace_windows_staged_qualification.py` is the agent-prepared
harness for the later Windows hardware gate. On macOS it fails closed with a typed
`unsupported-platform` receipt (`prepared-not-run`, 0 measured checks, 12 prepared
identities). It records no synthetic Windows credit. The Windows source-restore
port currently returns typed `unavailable` until its native implementation and
qualification land, so the transaction/crash/rollback checks stay unmeasured.

Prepared Windows check identities: qualified private-state owner/DACL; directory
durability and no-link ancestry; full-capacity transport; shared-pool unresolved
reservation retention; conservative shared-admission refusal; larger exact
index-last restore; no-replace publication preserving foreign replacements; crash
before commit (conditional rollback); crash after commit (durable outcome);
restart recovery with truthful none/committed/unknown; read-only restart with no
recovery write; sticky cancellation/deadline.

## Gates run for this slice

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` passed with
zero warnings (no Rust production code changed). The full `cargo test` suite
passed **4498 tests, 0 failed, 3 ignored** (77 summaries); the pre-commit hook
re-runs it at commit. These results are recorded in the machine receipt and the
pull request. The native capacity workflow and fixture controls are registered in
`.github/workflows/workspace-verification.yml` alongside the existing staged
controls.

## Remaining open gates

- Native Windows staged transaction, platform/crash/rollback and no-replace
  publication acceptance (later hardware gate; this slice prepares only).
- Independent contract review, dependency/OS-denial qualification,
  assistive-technology/participant acceptance, and the final integrated
  documentation review.

The [machine-readable receipt](2026-11-12-f19-staged-capacity-qualification.json)
binds the exact source, binary, native run and Windows-prep identities. A
test/check count is not a requirement or acceptance count; cohorts remain separate.
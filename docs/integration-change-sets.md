# Integration change sets (PRD 065 Phase 1)

FORGE exports deterministic, connector-neutral change sets from existing local
reports and plans them against an explicit target allowlist. Phase 1 ships the
schema, the dry-run/apply/reconcile state machine, and an in-process **fake**
GitHub Issues transport used only for contract fixtures. The live GitHub HTTPS
adapter is **Phase 2** and is blocked pending owner approval of an HTTP-client
dependency and the auth/TLS design (see "Phase 2 remainder" below). No code in
`src/workflow/` performs outbound network access.

## Commands

```bash
# 1. Export a deterministic change set from existing FORGE artifacts.
forge integration export \
  --root . \
  --framework-report impact.json \
  --applicability-report gaps.json \
  --lifecycle-record policy-lifecycle.json \
  --poam-manifest poam.json \
  --review-queue queue.json \
  --output change-set.json

# 2. Dry-run plan against an explicit allowlist (no remote mutation).
forge integration plan \
  --root . --change-set change-set.json \
  --connector-config connector.json --state state.json \
  --as-of 2026-11-12T00:00:00Z --output plan.json

# 3. Apply the exact unexpired plan. Without --confirm this prints the
#    target/count/sensitivity summary and mutates nothing.
forge integration apply \
  --root . --change-set change-set.json --connector-config connector.json \
  --plan plan.json --state state.json --confirm --report audit.json

# 4. Reconcile recorded bindings against remote observations (read-only).
forge integration reconcile --connector-config connector.json --state state.json \
  --report reconcile.json
```

`--root` defaults to `.`; source links are repository-relative (M-12). Excerpts
are omitted by default; pass `--include-excerpts` to opt in to bounded source
excerpts. `--as-of` pins the plan/apply clock for deterministic runs; it defaults
to now.

## Connector configuration

`connector.json` is a closed `forge.integration-connector/1` document:

```json
{
  "schema_version": "forge.integration-connector/1",
  "connector": "github-issues",
  "targets": ["policy-forge/forge"],
  "network": {
    "host": "api.github.com",
    "tls_required": true,
    "follow_redirects": false,
    "max_request_bytes": 262144,
    "max_response_bytes": 4194304,
    "timeout_ms": 30000,
    "max_requests_per_minute": 60
  }
}
```

The network-safety envelope (M-13) is validated on load: the host is fixed to
`api.github.com`, TLS verification is mandatory, redirects are never followed,
and byte/timeout/rate bounds are closed ranges. The fake transport enforces the
same rules so the contract is exercised before any live transport exists.

## Change-set schema

`forge.integration-change-set/1` carries work items sorted by stable FORGE ID.
Each item has a content-addressed UUID v5 (`kind`, source SHA-256, item key),
a `kind` (`finding`, `gap`, `lifecycle-review`, `framework-impact`,
`poam-milestone`, `review`), a stable `key`, `title`, `reason_codes`, `owners`,
optional `due_date`, and `provenance` binding the exact source artifact
(`schema_version`, repository-relative `source`, `sha256`). Default payloads are
minimized IDs/titles/reason-codes/owners/dates/links; excerpts require opt-in and
are never credential-shaped. Duplicate IDs, unknown fields, and credential-shaped
content are refused.

## Plan / apply / reconcile contract

- **Plan** re-verifies every source artifact hash and schema version (M-3), then
  projects each item to a create (new) or update (existing binding) against the
  first allowlisted target, recording a stable idempotency key, payload hash, and
  remote version precondition. The plan binds the exact change-set and config
  byte hashes and an expiry (M-6).
- **Apply** requires the exact unexpired plan (matching change-set/config hashes),
  re-verifies sources, and requires `--confirm` (M-7). It enforces the allowlist,
  reconciles before create, retries idempotently, and records per-operation
  outcomes. Only successful operations update local state (M-10).
- **Reconcile** treats remote state as observations only (M-11): it reports
  `observed`, `drift`, or `missing` per binding and never mutates FORGE source
  status/approval/risk/closure or the local connector state.

Audit reports (`forge.integration-audit/1`) are redacted and deterministic
functions of the plan and transport outcomes (M-14); they never carry
credentials, request bodies, or provider prose.

## Adapter contract fixtures

`tests/workflow_contract_test.rs` runs the state machine against the in-process
loopback fake (`src/workflow/fake.rs`, a `std::net::TcpListener` HTTP/1.1 server)
with zero outbound access and no new dependencies. The fixtures pin the contract
a live adapter must honor:

| Fixture | Guarantee |
|---------|-----------|
| `repeated_apply_creates_zero_duplicates` | G-3 idempotent retries, zero duplicates |
| `remote_drift_is_a_conflict_not_last_write_wins` | G-4 conflicts, no silent last-write-wins |
| `rate_limited_create_retries_without_duplicates` | M-13 rate limits, safe retry |
| `partial_failure_records_outcomes_and_resumes` | M-10 partial failure, resumability |
| `non_allowlisted_targets_are_refused` | M-4 allowlist refusal |
| `reports_and_state_are_redacted_and_deterministic` | M-14 secret redaction |
| `reconcile_observes_without_mutating_state_or_sources` | M-11 observations only |
| `plan_gates_refuse_expiry_drift_and_tampering` | M-6/G-2 exact unexpired plan |
| `credential_shaped_payloads_are_refused` | M-5 credential boundary |
| `cli_export_is_deterministic_and_minimized` | M-1/M-2/M-12 CLI export |
| `cli_plan_apply_reconcile_flow` | M-1/M-6/M-7 CLI flow |

The fake supports deterministic injection knobs (`Injections`): rate limits,
one-shot create failure, redirect, oversized response, response delay, and remote
version drift. A live adapter must implement the same `Transport` trait
(`src/workflow/adapter.rs`) and pass these same fixtures.

## Phase 2 remainder (blocked)

The live GitHub Issues HTTPS transport is deferred. Before it can ship, the owner
must approve:

1. **An HTTP-client dependency** (e.g. a TLS-capable client). No new crate may be
   added without approval per the dependency policy (PRD 069). The approved
   `hyper`/`tokio` stack is server-side; a client + TLS stack is a separate
   approval.
2. **Auth/TLS design**: an approved external credential provider supplying tokens
   at request time (never in manifests, flags, logs, or state), certificate
   verification, and the fixed-host/no-redirect envelope bound to a real client.

Phase 2 must bind the real client to the exact `NetworkPolicy` limits, implement
`Transport` over HTTPS, and pass the contract fixtures above against a live (or
recorded) GitHub endpoint. Reconciliation must remain observation-only.

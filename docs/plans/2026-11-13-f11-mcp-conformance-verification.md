# F11 MCP 2026-07-28 two-client conformance — verification receipt

Date: 2026-11-13. Tickets: veans `#55` (id 1450) and `#56` (id 1451).
Decision basis: D067 ratified 2026-11-13
([owner decision record](2026-11-13-owner-decision-record.md)): implemented
confined-roots model + re-pin proposal (sha256 `a15bac36…`), two-client pair =
MCP Inspector (modern era) + Claude Code (v2 runtime), MCP revision 2026-07-28.
Base: `origin/main` at `41aa68b4ef9a856375b6c0efa7ac72e3495ff685`.

Status: **Development conformance verification, not release acceptance.** This
receipt records exactly what was run offline and what still requires the real
MCP Inspector and an online/agent session. It closes no human gate.

## 1. What was built

`tests/mcp_conformance_test.rs` — an integration harness that spawns the real
`forge mcp serve` binary (`CARGO_BIN_EXE_forge`) and drives it over stdio with
**two independent client roles** implementing the same 2026-07-28 wire contract
with different strategies:

- **Role A (inspector-style):** integer request IDs, full namespaced `_meta`
  (`protocolVersion`, `clientCapabilities`, `clientInfo`, inert
  `org.example/trace` extension), and independent re-validation of every
  `structuredContent` against the shipped per-tool output schemas
  (`jsonschema`, same crate the worker uses).
- **Role B (claude-code-style):** string request IDs (`req-N`), bare mandatory
  `_meta` only, envelope-level assertions.

Both roles run the **same fixture and the same battery**. The fixture is
synthetic declared-owner data built entirely through public APIs
(`forge::lifecycle::record`, `forge::mapping::execute_build`,
`forge::applicability::execute_analyze`, public inventory/manifest parsers):
a native OSCAL Catalog with two controls and real trace props, an approved
lifecycle record, a human-reviewed mapping collection (uppercase native UUID
spelling preserved), an applicability manifest with one explicit decision and
one omission, and the maintained report — all pinned by exact raw SHA-256 in
two disjoint confined roots. No fixture grants real approval authority.

The driver honors the documented worker contract: stdin stays open while
awaiting responses; one request is paced per response (real-client behavior);
a separate burst mode exercises the bounded queue; EOF only at the end.

## 2. Conformance checks established (both roles, offline)

| Check | Result |
| --- | --- |
| Mandatory `server/discover`: `resultType:"complete"`, `supportedVersions:["2026-07-28"]`, `ttlMs:0`, `cacheScope:"private"`, tools-only capabilities, inert instructions | Pass |
| `tools/list`: exactly the seven tools, closed input/output schemas, `readOnlyHint:true`, `openWorldHint:false`, complete typing | Pass |
| Historical `initialize` is **not** part of the revision: `-32601` | Pass |
| `tools/list` refuses cursors (one complete page): `-32602` | Pass |
| Request-specific namespaced `_meta` is never cached: second request without `_meta` → `-32602`; wrong revision → `-32022` with `data.supported`/`data.requested` | Pass |
| Complete result typing on all seven tools: `resultType`, fixed inert text content, `isError`, schema-validated `structuredContent` (`forge.mcp-query/1`) | Pass |
| Seven read-only tools return exact recorded results: list_policies (1 approved policy), search (deterministic), get_requirement (exact source span, native pointer), trace_control (1 direct + 3 recorded-mapping pairs, original UUID spelling), get_recorded_applicability (explicit vs implicit-under-review), get_gap_summary (full maintained counts), get_artifact_status (raw pin, recorded approval, no schedule without opt-in) | Pass |
| Exact-ID trace/search: exact ID ranks first (`exact-id`), weaker some-token matches follow deterministically | Pass |
| Deterministic search: identical queries → byte-identical results; generation-pinned cursors (`<sha256>:<offset>`); foreign-generation cursor → `generation-changed` null-data error | Pass |
| Null-content rules: static-only worker → all seven tools `approval-unavailable` with `data:null`; absent selection → `not-found`; excerpt without profile opt-in → `visibility-refused`; invisible key → `visibility-refused`; wrong role → `unsupported-role` | Pass |
| Bounded input: >64 KiB frame drained and refused `-32700` without an invented ID, worker survives; `limit:51` and unknown arguments → `-32602`; unknown method → `-32601` | Pass |
| Bounded work: one active + two queued; fourth concurrent request → local `-32000` before retained growth | Pass |
| Duplicate accepted ID retires the transport: non-zero exit, fixed message, **zero** publications after retirement | Pass |
| Cancellation: `notifications/cancelled` for in-flight work suppresses its response; worker stays alive and answers the next request | Pass |
| Confined roots: nested decision root and symlinked project-root ancestry both fail closed (`approval-unavailable`); static discovery never opens either root | Pass |
| Wrong raw decision pin fails closed without private diagnostics | Pass |
| Privacy: no response contains fixture prose, titles, owner keys or private paths | Pass |

Execution: `cargo test --test mcp_conformance_test` — **19 passed, 0 failed**
(15 conformance tests + 4 inherited `common` module tests), deterministic,
isolated temp roots, no network, full-suite-safe.

## 3. Real-client evidence obtained offline

**Claude Code 2.1.323** (installed at `/opt/homebrew/bin/claude`, Mach-O arm64)
was run against the worker with no network:

```text
$ claude mcp add-json forge-f11 '{"type":"stdio","command":"<worktree>/target/debug/forge","args":["mcp","serve","--project","/tmp/f11-cc-project"]}' -s user
Added stdio MCP server forge-f11 to user config
$ claude mcp get forge-f11
forge-f11:
  Scope: User config (available in all your projects)
  Status: ✔ Connected
  Type: stdio
$ claude mcp list | grep forge-f11
forge-f11: …/target/debug/forge mcp serve --project /tmp/f11-cc-project - ✔ Connected
```

The health check performs the client's real connection handshake against the
worker's stdio transport; the binary contains the 2026-07-28 `server/discover`
probe path (verified in the shipped executable). The temporary user-config
entry was removed afterwards (`claude mcp remove forge-f11 -s user`).

**What this does NOT establish:** an authenticated agent session exercising
`tools/call`, inert-content handling and cancellation under a production agent
requires Claude Code sign-in/model access (online). That remains a human gate.

## 4. What requires the real clients / online (honest gates)

1. **MCP Inspector (modern era):** not installed and not present in the local
   npx cache (`~/.npm/_npx` contains no `@modelcontextprotocol/inspector`; the
   cached `@modelcontextprotocol/sdk` 1.30.0/1.32.1 speak the initialize-first
   era, latest `2025-11-25`, and cannot establish 2026-07-28 conformance).
   Fetching it requires network:
   `npx @modelcontextprotocol/inspector --cli <worker command>` — **human/online
   gate**. Role A in the harness is a stand-in, not the Inspector.
2. **Claude Code agent session:** connection/health verified offline (§3);
   authenticated `tools/call` evaluation with a real model is an **online
   gate**. Role B is a stand-in for the wire behavior, not the product.
3. **Six project-specific owner-record rows** (D067-P1/P2, S1/S2, E1) remain a
   separate disclosure gate; synthetic fixture rows grant no real authority.
4. Historical initialize-first checklists (2026-10-04 evidence) do **not**
   establish current conformance and were not reused as such.

## 5. Commands

```bash
cargo test --test mcp_conformance_test        # 19 passed; 0 failed
cargo fmt --check                             # clean
cargo clippy --all-targets -- -D warnings     # clean
cargo test                                    # full suite: 79 test binaries, all ok, 0 failures
```

The full suite was run with `CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0`
because the host disk was at capacity; debug-info stripping changes no test
behavior. An earlier default-profile full-suite attempt failed only with
`No space left on device` linker errors, not test failures.

Environment: macOS darwin arm64, rustc 1.99.1 (Homebrew), node v22.22.3,
Claude Code 2.1.323. No new crate dependencies; the harness uses only existing
deps (`serde_json`, `jsonschema`, `tempfile`, `sha2` via `tests/common`,
`chrono`, public `forge` APIs).
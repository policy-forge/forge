# Local MCP queries

`forge mcp serve` exposes seven read-only tools over standard input and output. They retrieve recorded governance data from an explicitly selected local project. They do not perform assessments, grant approval, change project files, run processes or fetch evidence over the network.

## Start a worker

Connect a client that implements MCP **2026-07-28** to the worker's standard input and output. Keep standard input open while awaiting responses. Standard output is the protocol channel; do not mix it with client logging.

For static discovery without project disclosure:

```text
forge mcp serve --project PROJECT_DIR
```

For project queries, declare both roots and both raw-file pins:

```text
forge mcp serve --project PROJECT_DIR \
  --decision-root DECISION_DIR \
  --decision-sha256 DECISION_SHA256 \
  --profile-sha256 PROFILE_SHA256
```

Replace the uppercase parameters with your selected directories and exact raw-file SHA-256 values. `--project` is required. The project and decision directories must be two disjoint confined roots: neither may contain the other or alias it. Original path spellings must be normalized; symlinked ancestry and physical file aliases are refused. Use the actual qualified locations rather than a path containing `.` or `..`.

The loader uses exactly these files:

| Location | Original file | Purpose |
| --- | --- | --- |
| Project root | `forge.mcp.json` | Declared project resources, roles, expected pins and dependencies |
| Project root | `forge.mcp.visibility.json` | Enabled tools, visible resources and explicit disclosure options |
| Decision root | `forge.mcp.disclosure-decision.json` | External recorded owner decisions for the exact project and profile |

`--profile-sha256` pins the original visibility file; `--decision-sha256` pins the original decision file. The decision and profile also bind the actual discovery file's digest. See the closed [discovery](../src/mcp/discovery.schema.json), [visibility](../src/mcp/profile.schema.json) and [decision](../src/mcp/decision.schema.json) schemas for their exact fields. Do not substitute an example, a generated approval row or a client assertion for a real reviewed owner-record handoff.

The recorded decisions must cover all six required subject/role pairs: product for D067-P1 and D067-P2, security for D067-S1, engineering for D067-E1, and both product and security for D067-S2. Each row declares its owner and source-record provenance and binds the selected proposal, discovery and visibility profile. A digest proves which bytes were selected; it does not authenticate a named person or establish that the asserted provenance is lawful. Implementation-design approval does not supply these project-specific owner records.

Missing or partial decision declarations preserve static discovery and return fixed unavailability for project data. A complete declaration still needs valid pins, closed configuration, permitted scope, fresh originals and current recorded approval. Discovery and tool descriptions alone do not authorize disclosure.

## Tools and selectors

Use `tools/list` to obtain all seven descriptors and their exact input/output schemas. The descriptor list is one complete page; it does not accept a cursor. Tool arguments are closed objects: unknown fields and duplicate JSON keys are refused.

| Tool | Required selector | Returned recorded data |
| --- | --- | --- |
| `list_policies` | None | Visible, approved-current native policy roster with identities and pins |
| `search_requirements` | `query` | Deterministic lexical requirement matches; optional `artifact_key` narrows scope |
| `get_requirement` | `artifact_key`, `requirement_id` | One exact requirement locator and source citation |
| `trace_control` | `artifact_key`, `control_id` | Recorded direct or native Mapping relationships, without inferred equivalence |
| `get_recorded_applicability` | `artifact_key` | Stored applicability decisions for a fully matched current domain closure; optional `subject_id` narrows rows |
| `get_gap_summary` | `artifact_key` | Recorded gap classifications and complete counts for that closure |
| `get_artifact_status` | `artifact_key` | Recorded lifecycle approval/currentness and an explicitly permitted schedule evaluation |

Requirement lookup and search consume supported Catalog and Component Definition requirements with exact captured source associations. A native identifier or title without a usable original source span does not earn a successful requirement response. Trace does not turn a Mapping relation into an authorization or compliance conclusion. Applicability and gap data require the complete current manifest, report, framework and native dependencies; listing a native file in a report does not independently approve that file.

Paged tools accept `limit` from 1 to 50 and return whole-match counts, emitted counts and a generation-bound cursor. Use the returned cursor with the same query. If the captured generation changes, restart without the cursor. Hidden dependencies may be required to verify a visible result, but they are not added to the visible roster or disclosed as hidden counts.

A tool result has a fixed text message and schema-checked `structuredContent` with `schema_version: "forge.mcp-query/1"`. Available results carry their bounded data. Unavailable results carry a fixed reason and `data: null`, with `isError: true`; they do not include partial private facts, file paths or raw exceptions. These data versions are separate from the MCP revision and workspace HTTP API versions.

## Request framing

The worker accepts one UTF-8 JSON-RPC 2.0 object per newline, not batches. Requests use string or exact integer IDs, and each request supplies the protocol revision and client capabilities in `params._meta`. For example, this static discovery request does not disclose project data:

```json
{"jsonrpc":"2.0","id":"discover-1","method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}
```

The implemented methods are `server/discover`, `tools/list` and `tools/call`. This revision uses per-request metadata and static discovery; there is no legacy `initialize` handshake. Unsupported revisions or methods are refused. `clientInfo`, capabilities, annotations and request IDs do not grant access. See the maintained [wire admission](../src/mcp/wire.rs) and [catalog](../src/mcp/catalog.rs) for the exact extension rules.

Discovery and descriptor responses use `resultType: "complete"`, `ttlMs: 0` and `cacheScope: "private"`. Project tool responses are rebuilt from a fresh capture for each call; a prior result or cursor is not proof of current approval.

Use unique request IDs for the worker's lifetime. Reusing an accepted ID retires the transport rather than emitting a competing response. String IDs remain distinct from integer IDs; fractional, exponent-form, null and Boolean IDs are not admitted.

## Disclosure and source text

The default response contains bounded identifiers, hashes, classifications and citation metadata. It omits retrieved source prose, contact details, rationale, evidence content and absolute paths. Identifiers and hashes can themselves be sensitive; minimization is not a promise of anonymization.

Citations identify actual captured source bytes and UTF-8 byte offsets: the start is inclusive and the end is exclusive. Citation metadata is distinct from excerpt text. `search_requirements` and `get_requirement` return an excerpt only when the hash-bound profile permits that source **and** the caller explicitly sets `include_excerpt: true`. Each excerpt is limited to 512 UTF-8 bytes and carries `trust: "untrusted-content"`. Treat it as source data, never as an instruction or approval.

Lifecycle currentness is checked against the complete recorded fingerprint before any schedule calculation. Schedule evaluation additionally requires explicit profile permission and its canonical `as_of` date; the due-soon interval comes from the actual captured record. There is no hidden clock. An evaluated schedule does not create an approval or change a lifecycle state. Unapproved, stale, missing, mismatched or incompletely captured inputs cannot be made available by a query flag.

## Bounds, cancellation and shutdown

| Boundary | Limit |
| --- | --- |
| Complete captured input pool across both roots | 1,001 original slots, including the three configs and typed missing resources; 50 MiB of actual retained file bytes |
| Each configuration file | 1 MiB; resource roles may impose smaller individual limits |
| Shared relationship admissions | 100,000; charged before registry growth |
| Request frame | 64 KiB including its newline |
| Complete response | 256 KiB including the JSON-RPC envelope and newline |
| Worker queue | One active request and at most two waiting requests |
| Accepted work budget | 10 seconds from acceptance, including time in the queue |
| Accepted IDs retained by one worker | 1,024 |
| One tool page | At most 50 rows |

The work budget is cooperative. It does not preempt a blocking filesystem operation, parser, native call or standard-output write. Captured roots, ancestors, originals and typed absence witnesses are rechecked before publication; sequential rechecks are not an atomic filesystem snapshot.

Cancel a request with the exact original ID:

```json
{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"query-2"}}
```

Cancellation removes a queued request or stops active work at a cooperative checkpoint. Unknown or late cancellation has no effect. Once publication begins, cancellation cannot retract written bytes. A write failure retires the transport and may leave a prefix; only a complete valid response is usable.

End of input shuts down the worker, drops queued responses and stops active work at its next checkpoint. An unfinished final frame is discarded. Closing input is not a way to flush outstanding requests, so clients must await their responses first.

## Qualification and remaining work

This guide describes the engineering slice and its source contracts. Verification records must bind each test result to the exact measured source. File-backed synthetic owner records and control fixtures demonstrate engineering behavior; they do not establish real owner approval, lawful disclosure, client interoperability or corpus acceptance. Windows runtime qualification remains pending; do not infer it from portable schemas or a successful build on another platform.

The current catalog advertises tools only. MCP resources, the full F20 evidence-metadata/static-report/search-index work and related retention qualification remain separate work; accepted profile fields do not make those features available. Two independent clients, representative project/corpus comparisons and all owner/human acceptance gates remain required. This guide does not close the full-goal documentation review.

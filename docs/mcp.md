# Local MCP queries

`forge mcp serve` exposes seven read-only tools over standard input and output. They retrieve recorded governance data from an explicitly selected local project. They do not perform assessments, grant approval, change project files, run processes or fetch evidence over the network.

The worker defaults to the original declaration family (`--declaration-family v1`). The sections below describe that path. The separate [F20 `/2` development checkpoint](#f20-2-development-checkpoint) records the narrower candidate scope, explicit selector and unfinished qualification gates; it does not change the default path or establish release availability.

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

## F20 `/2` development checkpoint

The paragraphs in this section retain the **2026-10-05** measured interval,
including its command-registration gap and failed App controls. The later
[qualified `/2` candidate](#supported-candidate-queries) below records the
subsequent implementation and qualification; its results do not relabel this
earlier interval.

The separate `/2` candidate adds closed server-decision and offline build-intent
declarations, one held input owner and shared admission accounting. The intended
read-only `forge mcp check-index-inputs` consumer was not registered at that
measured checkpoint. Checking complete inputs would not build or publish an
index, nor add a server resource or tool. The existing `/1` worker examples above remain the
current CLI contract.

The offline native Catalog/Component slice has been exercised separately. The
later applicability candidate requires the actual complete applicability
manifest, source-domain Lifecycle closure, independently Approved/current
framework closure and full native report comparison. Visible unsupported
Profile, Mapping or linkage scope is refused as a whole; profile fields and
recorded hashes cannot substitute for missing native producers.

The latest 2026-10-05 same-owner App reuse run compiled and passed 22 selected
native controls and three controller controls, with two native failures:
25 passes and 2 failures across the 27-control producer. Its complete factory
path now passes the saved-report fixture's first preparation, but standalone
report preparation and the distinct-manifest case still reach Capacity. Both
earlier and successor first-capacity diagnostic histories remain retained. The
complete App gate is unaccepted and managed F11/server coupling remains open;
further borrowed-preparation/admission design is not a qualified runtime result.

The fixed budget and first-stop behavior remain unchanged. `/2` jointly bounds
retained original/proof/native data within a logical 50 MiB pool, with 100,000
work units and 1,001 original/handle ceilings. These logical limits do not measure
heap usage or guarantee every near-limit input fits; they do not replace the
`/1` table above.

Evidence metadata trace, static-report resources, an actual deterministic optional
index builder and complete server publication coupling remain open. Actual
project-owner records, client/corpus evaluation, retention/privacy/platform
qualification and all human acceptance gates remain required.

## Qualification and remaining work

This guide describes the engineering slice and its source contracts. Verification records must bind each test result to the exact measured source. File-backed synthetic owner records and control fixtures demonstrate engineering behavior; they do not establish real owner approval, lawful disclosure, client interoperability or corpus acceptance. Windows runtime qualification remains pending; do not infer it from portable schemas or a successful build on another platform.

The current catalog advertises tools only. MCP resources, the full F20 evidence-metadata/static-report/search-index work and related retention qualification remain separate work; accepted profile fields do not make those features available. Two independent clients, representative project/corpus comparisons and all owner/human acceptance gates remain required. This guide does not close the full-goal documentation review.
## F20 /2 development checkpoint

This section describes the `/2` engineering candidate on an exact locally qualified source image. Compilation, normal tests, source documentation, fresh library coverage and the compiled macOS child campaign have closed; final integration and hosted/client/owner acceptance remain separate gates. The original declaration family remains the default; selecting `/2` requires an explicit flag before any project capture begins:

```text
forge mcp serve --declaration-family v2 --project PROJECT_DIR \
  --decision-root DECISION_DIR \
  --decision-sha256 DECISION_SHA256 \
  --profile-sha256 PROFILE_SHA256
```

Use exact raw-file pins and genuine project-specific owner records. The `/2` path retains the same three configuration filenames, confined disjoint roots and six required owner subject/role pairs described above. It uses closed [project `/2`](../src/mcp/discovery-v2.schema.json), [visibility `/2`](../src/mcp/profile-v2.schema.json) and [disclosure-decision `/2`](../src/mcp/decision-v2.schema.json) schemas. The project declaration contains the complete base resource roster and an explicit companion roster; declaring a companion does not activate a missing native producer. Hidden originals are captured and checked as part of the complete closure without disclosing their contents or counts.

The MCP revision remains **2026-07-28**, and the seven tool names and `forge.mcp-query/1` output envelope remain unchanged. `/2` identifies the declaration family, not a new protocol or output version. Static discovery remains available without project disclosure; discovery, client capabilities and configuration hashes do not establish native approval or reviewer authority.

### Supported candidate queries

| Tool | `/2` candidate scope |
| --- | --- |
| `list_policies` | Visible, independently approved-current Catalog and Component Definition metadata; policy-source identity fields remain null |
| `search_requirements` | Complete native Catalog and Component Definition requirements with exact captured source citations |
| `get_requirement` | An exact native identifier in one supported visible artifact |
| `trace_control` | Recorded direct native control relationships; no inferred equivalence or Mapping producer |
| `get_artifact_status` | Recorded lifecycle approval/currentness; schedule fields are null and schedule evaluation is `not-evaluated` |
| `get_recorded_applicability` | A complete current applicability manifest/report closure using an independently approved visible Catalog framework and an empty Mapping list |
| `get_gap_summary` | The same complete applicability closure, with full stored-report equality and complete gap counts |

Applicability source approval and framework approval are checked independently. The complete stored-report comparison includes private native fields, all rows and all counts before the minimized query projection. Multiple matching manifests refuse the result. A source hash hint or a borrowed parsed value cannot issue an approved-current native closure.

Profile frameworks, nonempty Mapping dependencies, SSP, linkage/evidence closure, static-report resources and a selected search index remain unavailable in this slice. Unsupported visible families or unsupported dependencies needed by a visible result refuse the whole gate; the worker does not drop inputs to obtain a smaller successful closure. A declared Linkage Manifest companion is refused before its declared content is read. The factory also requires omitted source text and schedule modes, null `as_of` and search-index selection, and empty noncurrent, excerpt, schedule, evidence-metadata, static-report and resource-family opt-in lists. Schema fields alone do not deliver PRD 067 S-1, S-2 or S-4. S-3 still requires real project-owner profile and disclosure records.

### Exact selectors, search and citations

Artifact keys retain their existing safe-token rule. `/2` requirement and control IDs are exact nonempty native UTF-8 strings of at most 4,096 bytes; they are not normalized into another alphabet or UUID spelling. The original `/1` selector behavior remains unchanged. A complete native projection that cannot preserve its supported public fields refuses the result rather than clipping or replacing private identifiers.

Search accepts a nonempty query of at most 4,096 bytes, with no control characters and at most 32 distinct lexical tokens. The native matcher uses Unicode lowercase tokens and deterministic exact-ID, all-token and some-token rank buckets. Duplicate query tokens do not multiply a match. Results retain complete match counts before a page is selected. Citation paths and UTF-8 byte/line spans come from the actual captured source association, including CRLF boundaries; an ID alone does not supply a citation.

Pages contain 1–50 rows. Both page size and cursor are excluded from the canonical query generation, so changing page size preserves the same query generation. The remaining query selection and the complete physical input generation still bind the cursor. A changed input generation requires restarting without the old cursor. `/2` does not expose source excerpts: `include_excerpt: true` receives fixed visibility refusal. Citation metadata is not source prose, authorization or a compliance conclusion.

### Capture lifetime and bounds

One original controller and admission ledger span configuration decoding, complete declared capture, native/lifecycle checks, projection, finite encoding, readback and final verification. A retained successful query owner is checked again before stdout publication. A query that becomes unavailable after a genuine owner exists still checks that complete owner; static or malformed factory refusal has no native owner to verify. Capacity, interruption and control failure remain sticky first stops across ordinary parser, schema and domain failures. A later phase does not obtain a renewed allowance.

| `/2` capture boundary | Fixed limit |
| --- | --- |
| Original registration attempts | 1,001, including fixed configs, hidden originals and absence registrations |
| Retained and temporarily reserved native handle geometry | 1,001 |
| Shared retained data | 50 MiB of admitted original bytes, native proofs, derived data and retained output |
| Each ordinary declared file | At most 10 MiB; the exact role may impose a smaller limit |
| Each fixed configuration | 1 MiB |
| Shared admitted work | 100,000 units across decoding, native preparation, repeated verification and output |

These are declared logical accounting bounds, not measurements of allocator heap use. A valid large closure may reach byte, work or geometry capacity before the maximum number of original attempts. Request framing, complete response size, queue limits, accepted-ID limits and the cooperative ten-second budget remain as described above. Whole root, file, hash and absence checks are sequential rechecks, not an atomic filesystem snapshot. Cancellation and output-write failure retain the existing transport behavior.

### Offline input validation

The candidate also provides a read-only offline gate with a separate intent root and closed [index-build intent](../src/mcp/index-build-intent.schema.json):

```text
forge mcp check-index-inputs --project PROJECT_DIR \
  --intent-root INTENT_DIR \
  --intent-sha256 INTENT_SHA256 \
  --profile-sha256 PROFILE_SHA256
```

The intent cannot substitute for the server's external disclosure decision. A successful gate consumes genuine complete captured inputs, checks the selected destination's held absence and emits exactly one fixed line after final verification:

```text
MCP index inputs captured and checked; no index built.
```

This command does not build or publish an index and does not complete S-4. It supplies no raw-input shortcut, reusable currentness token or later publication authority.

### Qualification checkpoint and remaining gates

The final 1,753-file TEMP source passed all four formatting commands and strict all-feature/all-target Clippy. The full normal all-feature run passed **3,699 tests, with zero failures and three existing ignored golden fixtures**, across 74 groups, including 2,611 library tests and 24 doctests. The ignored fixtures supply no execution credit. The source, basis and 1,751-file managed image were unchanged throughout the measured intervals.

The source documentation census covers 36 changed physical Rust files. All **602 selected callable bodies and 104 selected named types** have adjacent Rustdoc. The callable population is 356 production lexical bodies, 18 inline test bodies and 228 physical external test bodies. This is a lexical/position selection without AST, macro or cfg expansion; it excludes fields, enum variants and semicolon prototypes. The broader complete changed-file callable population has 866 documented bodies out of 960, leaving 94 unchanged legacy gaps. These populations overlap and must not be added. All 141 selected literal test declarations correlate by unique names to passing actual normal results; that correlation supplies no additional tests or platform credit.

A fresh library-only LLVM run passed **2,611 tests, with zero failures and zero ignored controls**. Four new exclusive raw profiles were retained; the Cargo target was reused. Merge and export preserve their actual binary/source/profile interval pins. The physical file measurements below include full unchanged lines, inline tests, derived code and legacy bodies; they do not measure only changed code or the selected Rustdoc bodies.

| Measured physical file cohort | Covered lines | Covered functions |
| --- | --- | --- |
| All 36 changed physical Rust files | 16,275 / 18,206 (89.394%) | 1,385 / 1,557 (88.953%) |
| 28 source files, including inline/derived/legacy code | 12,095 / 13,959 (86.647%) | 1,076 / 1,232 (87.338%) |
| Eight external test files | 4,180 / 4,247 (98.422%) | 309 / 325 (95.077%) |

All 36 physical files are present in the actual export. Branches have a zero denominator and remain unmeasured; no branch, MC/DC, production-only, changed-line, compiled-CLI instrumentation or multi-platform coverage follows from these figures. Normal integration/doctest populations are separate from library instrumentation.

The compiled macOS CLI campaign passed on the same final source image: eight child commands (one build, one Applicability scaffold and six Lifecycle commands) exited successfully, followed by two real stdio workers that both exited cleanly with empty stderr and trailing output. The workers completed 14 actual transactions: 13 on explicit `/2` (discovery, tool listing, all seven tools, two cursor continuations, source-drift refusal and restoration) and one default `/1` refusal of `/2` declarations. Source, basis, managed files and the compiled binary were unchanged throughout that measured interval. These transactions are a separate population from tests and library coverage. They qualify this synthetic macOS campaign; they do not establish two independent clients, broader platform runtime, network isolation or an active publication-race proof.

The earlier campaign using `--output app.json` failed during Applicability scaffold setup with exit 2, before any worker or transaction ran. The successful campaign uses `--output ./app.json`, with an explicit parent. The bare-filename parent-resolution limitation is retained; this recipe change is not a product fix.

The earlier 187-control component campaign remains a separate historical population and must not be added to the normal run. Its 182-pass/1-failure predecessor and 0-pass/1-failure diagnostic remain preserved. The combined strict V1–V3 failures are retained; V4's success does not retroactively change them. Original App 25-pass/2-failure and subsequent qualified repair intervals remain dated evidence, not new owner acceptance.

The intended managed integration and hosted/platform qualification remain pending for this exact candidate. Real external project-owner records, two independent MCP clients, adjudicated corpus/raw-document baseline evaluation, absent resource/native families and human acceptance remain open. This guide does not complete PRD 067 S-1–S-4, make a validation-only offline gate an index builder/publisher, or close the full-goal documentation review.

### Draft integration checkpoint

A later **2026-10-06 02:16 UTC** readback confirms [PR #220](https://github.com/policy-forge/forge/pull/220)
is **open and draft** at `7bd8a6f336632660952085aae3574c83a2612e2a`, based on
`codex/f11-read-only-mcp-discovery`. Its committed MCP guide and PRD match the
two reviewed pages from the TEMP checkpoint above. This records draft-source
integration separately from those earlier measured intervals; it does not
reclassify their prospective integration notes or add a new LLVM, compiled-child
or platform measurement.

At that readback, all ten workflow jobs were still in progress. No hosted
qualification follows from this snapshot, and an open draft is neither merged
main nor a released or accepted feature. The unsupported native/resource
families, index builder/publisher, real owner records, independent client/corpus
evaluation, platform and human acceptance gates remain open. The final goal-wide
documentation review also remains open.

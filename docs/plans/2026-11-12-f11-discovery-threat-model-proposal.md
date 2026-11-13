# F11 project discovery and threat model — durable proposal (reconstruction re-pin)

Date: 2026-11-12. Baseline for every code citation below: `main` at
`b4226bb35d796222c30f93235ac22118dd26a344` (PR #259). Tickets: veans `#56`
(id 1451, review prerequisite) and `#55` (id 1450, F11 read-only MCP core).
Decision register item: **D067** (owner decision packet 2026-11-12, §1).

> **HONESTY NOTICE — RECONSTRUCTION RE-PIN, NOT THE ORIGINAL APPROVED BYTES.**
> This document is a **reconstruction** of the F11 discovery and threat-model
> proposal, rebuilt from the implemented code at `b4226bb` and the retained
> verification evidence. It is **not** the proposal that the 2026-10-04
> implementation-design approval bound. That original file
> (`/private/tmp/forge-f11-discovery-implementation-review-v3.md`, 6,443 bytes,
> SHA-256 `60c8dff16bf32c69134f19fe5de19ec59b2bb216ac5b0186ca7624fe986ca848`)
> is **lost**: the path no longer exists and no copy of its bytes survives in
> the repository or the retained evidence archives (only hash references
> remain; see §2). **No byte-identity with the original is claimed, and no
> approval is inherited by this document.** The 2026-10-04 design approval
> stands as history of the approved design intent; **D067 ratification must
> approve THIS document with a fresh, recorded owner decision** before it can
> bind reviewable artifacts. Until that fresh approval is recorded, this
> document grants no authority, closes no gate, and changes no board status.

Status: **Proposed — owner decision required.** Documents only: this re-pin
changes no code, records no approval, and moves no ticket status.

---

## 1. Purpose and concrete decision request

PRD 067 requires a recorded project-discovery and threat-model approval before
F11 public implementation (`docs/PRD/067-prd-read-only-mcp-governance-interface.md`,
ticket `#55` prerequisite; packet §1 "Exact question", lines 41–46). The
2026-10-04 approval bound proposal bytes that lived only in `/private/tmp`;
those bytes are gone (packet finding **F4**, lines 617–621). D067 ratification
cannot bind reviewable artifacts until a durable proposal exists in `docs/`.
This document is that durable artifact.

**Decision requested of the owner (D067):**

1. Approve **this document** — the implemented confined-roots project-discovery
   model and read-only threat model described in §§4–6 — as the recorded D067
   proposal, binding its committed path, byte count and SHA-256 (recorded in
   the delivery comment on `#56`/`#55` and the draft PR). This is packet
   **Option A** (§1 "Options and tradeoffs", lines 100–121), conditional on the
   re-pin this document performs.
2. Name the **two independent MCP clients** for conformance to MCP revision
   `2026-07-28` (§8; proposed pair, owner-decision-required).
3. Keep the six project-specific owner-record rows a **separate, still-required
   disclosure gate** (§5.4). Design approval does not supply them.

What this approval would **not** earn (carried forward from the 2026-10-04
record's `unearned` list, approval JSON line 11): new dependency or
supply-chain audit approval; authenticated owner identity/signatures;
two-client interoperability; adjudicated corpus/raw baseline and thresholds;
pilot, product, security and release acceptance. The F20 `/2` checkpoint is a
distinct, narrower development scope and is **not** folded into this approval
(§7; packet lines 116–121, 161–166).

## 2. Historical record: the lost approved proposal

The
[implementation-design approval](2026-10-04-f11-implementation-design-approval-v1.json)
(`docs/plans/2026-10-04-f11-implementation-design-approval-v1.json`) records:

- Line 5 — `"answer": "Approve this design for implementation"`; authority
  (line 4): direct human reply, recorded `2026-10-04T19:14:02Z` (line 3).
- Line 6 — the bound proposal, **historical reference only**:
  - path: `/private/tmp/forge-f11-discovery-implementation-review-v3.md`
  - bytes: `6443`
  - sha256: `60c8dff16bf32c69134f19fe5de19ec59b2bb216ac5b0186ca7624fe986ca848`
- Line 7 — `v2_index_sha256`:
  `d52da047f0166d9d8cd002221ee9608577f935e7c803969a178b91e797801f49`.
- Line 8 — `handoff_index_sha256`:
  `27e405e4db5d8e5ff125bf5cd258ab9d66956f9ac62bdec95e7343f5cffc12fc`.
- Line 9 — scope: "F11/F20 discovery and threat-model design for
  implementation: seven read-only tools, minimized approved/current
  disclosure, hash-pinned profiles with explicit external owner-record handoff
  and MCP2026-07-28 stdio, as completely described in the pinned proposal."
- Line 10 — `runtime_owner_record`: "Not supplied or inferred."
- Line 11 — the `unearned` list reproduced in §1.

**Loss verification performed for this re-pin (2026-11-12):**

- `/private/tmp/forge-f11-discovery-implementation-review-v3.md` does not exist
  on disk (`ls` fails: No such file or directory).
- The retained complete evidence archive
  `docs/plans/evidence/2026-10-04-f11-current-complete-evidence-v3.json.gz`
  (1,269,862 bytes; SHA-256
  `f6d14490c52535540fd8bd8bce5c5b3c9c5a92e9a4047df73acfe2b16603459a`, matching
  the pin in `2026-10-04-f11-pre-pr-verification-v3.json` lines 64–68) contains
  exactly **one** occurrence of the proposal hash — a reference inside the
  embedded approval record — and **no proposal body**.
- Repository-wide content search finds the hash only as a reference:
  `2026-10-04-f11-implementation-design-approval-v1.json:6`,
  `2026-10-04-f11-pre-pr-verification-v2.json:20`,
  `2026-10-04-f11-pre-pr-verification-v3.json:20`. In git history the hash
  appears only in the commits that introduced those references (`5f57f1a`,
  `75785a2` per `git log --all -S <hash>`); no commit in any branch ever added
  a file named `*forge-f11-discovery-implementation-review*`
  (`git log --all --diff-filter=A` for that pattern is empty).
- The v2 proposal index (`d52da047…`) and handoff index (`27e405e4db5d…`) are
  likewise present only as hash references (approval JSON lines 7–8;
  `src/mcp/disclosure.rs:40`; `src/mcp/decision.schema.json:38`); their bytes
  are not in the repository either.

**Consequence.** The original approval remains valid history for the design it
approved, but its bound artifact is unrecoverable. This reconstruction may
differ in wording and coverage from the lost 6,443-byte original; it must be
reviewed on its own merits and approved fresh. The runtime disclosure gate
still pins the historical v2 index hash (`src/mcp/disclosure.rs:40`,
`src/mcp/decision.schema.json:37-39`); this document does **not** change that
constant. Any successor binding of the runtime gate to new proposal/index bytes
is a separate, reviewed code change requiring its own owner approval.

## 3. Assets, adversaries and trust boundaries

**Assets.** Recorded governance facts: policy requirements, OSCAL artifact
identities, recorded applicability decisions and gaps, lifecycle
approval/currentness, and bounded source citations. Also protected: the
operator's local filesystem outside the two selected roots, and the privacy of
source prose, evidence bytes, contact fields and absolute paths
(PRD 067 M-12, line 157; `docs/mcp.md:78`).

**Adversaries / failure modes.**

- A malicious or buggy MCP client attempting off-root reads, path traversal,
  symlink/alias escapes, oversized or malformed frames, ID reuse, notification
  floods, or cancellation races (PRD 067 M-2/M-11/M-15, lines 147, 156, 160).
- Untrusted project content (policy prose, artifact fields) attempting prompt
  injection or tool chaining (PRD 067 M-10, line 155).
- Self-declared client identity, manifests, annotations or hashes attempting to
  impersonate owner approval (ticket `#55`: "a manifest or self-declared MCP
  client identity cannot supply that approval").
- Stale, swapped or partially captured originals attempting to pass as
  approved/current data (PRD 067 M-5, line 150).

**Trust boundaries.**

1. **Operator selection at launch** — the only source of disclosure authority:
   explicit roots and raw-file SHA-256 pins on the command line
   (`src/cli/mod.rs:905-921`; `docs/mcp.md:17-26`).
2. **External recorded owner decision** — the decision-root file with six
   subject/role rows (§5.4). Neither project metadata nor the recorded approval
   authenticates an owner or grants permission to act
   (`src/mcp/artifact_status.rs:1-7`, `:103`). A digest proves which bytes were
   selected; it does not authenticate a named person (`docs/mcp.md:38`).
3. **Everything else is untrusted**: caller frames, `_meta`, capabilities,
   `clientInfo`, annotations, request IDs, and all retrieved content
   (`src/mcp/wire.rs:31-32`, `:285-301`; `src/mcp/catalog.rs:1-3`, `:318-323`;
   `src/mcp/queries.rs:259-267`).

Read-only data never authenticates an owner or authorizes domain mutation
(ticket `#55`; PRD 067 M-3, line 148; non-goals, lines 98–104).

## 4. Project discovery model (implemented, v1 default)

### 4.1 Two disjoint confined roots

`forge mcp serve --project PROJECT_DIR` is the static-discovery form; project
queries additionally require `--decision-root`, `--decision-sha256` and
`--profile-sha256` (`src/cli/mod.rs:905-921`; `docs/mcp.md:11-26`). `--project`
is required; missing or half-paired decision declarations are never inferred
(`src/cli/mod.rs:3360-3376` test; `docs/mcp.md:40`).

- The project and decision directories must be **two disjoint confined roots**:
  neither may contain the other or alias it (`docs/mcp.md:26`).
- Raw root spellings are validated **before** any path composition or open:
  non-empty, ≤ 64 KiB, no `.`/`..` components, normalized spelling
  (`src/mcp/disclosure.rs:668-679` `qualify`;
  `src/linkage/mod.rs:1450-1470` `has_normalized_path_spelling`, which on
  Windows also refuses trailing-space/dot component aliases).
- Roots are qualified by walking **every actual component through no-link
  native directory opens**, retaining held handles and native (volume, file)
  identities (`src/linkage/fresh.rs:139-163` `qualify_root`, `:42-57`
  `RootGeneration`). Unix uses `openat` with
  `O_RDONLY | O_CLOEXEC | O_NOFOLLOW | O_DIRECTORY`
  (`src/linkage/fresh.rs:715-737` `open_child`, `:739-760` `open_root`).
  Windows walks real no-delete-share directories and **rejects every reparse
  handle** (`src/linkage/fresh.rs:695-703`, `:818-885`). Unsupported platforms
  refuse rather than fabricate path-only proof
  (`src/linkage/fresh.rs:705-710`; `src/mcp/mod.rs:31-32`).
- Overlap refusal combines **actual ancestry identity** comparison with a
  conservative ASCII-folded portable-spelling comparison, so equal, nested, or
  case/separator-folded alias roots are refused; shared ancestors such as `/`
  do not make disjoint siblings overlap
  (`src/mcp/disclosure.rs:682-689` `reject_overlap`;
  `src/linkage/fresh.rs:415-424` `roots_overlap`;
  `src/mcp/artifact_status.rs:808-815` `paths_overlap`).
- Declared resource paths must be portable ASCII descendants (≤ 512 bytes,
  ≤ 128-byte components, no reserved Windows device names, no trailing dots,
  no alias normalization) (`src/mcp/artifact_status.rs:786-806`
  `portable_path`; `src/linkage/fresh.rs:128-136` `validate_relative`).
  Original alias spellings fail **before IO**
  (`src/mcp/artifact_status.rs:1362-1390` tests).

### 4.2 Exactly three configuration originals, raw-file SHA-256 pins

The loader uses exactly (`src/mcp/disclosure.rs:29-31` `CONFIG_PATHS`;
`docs/mcp.md:28-36`):

| Location | Original file | Purpose |
| --- | --- | --- |
| Project root | `forge.mcp.json` | Declared resources, roles, expected pins, dependencies |
| Project root | `forge.mcp.visibility.json` | Enabled tools, visible resources, disclosure options |
| Decision root | `forge.mcp.disclosure-decision.json` | External recorded owner decisions |

- `--decision-sha256` pins the **raw bytes** of the decision file;
  `--profile-sha256` pins the **raw bytes** of the visibility file. Both must be
  exact 64-hex SHA-256 (`src/mcp/disclosure.rs:557-559`;
  `src/mcp/artifact_status.rs:824-828` `valid_hash`). These are raw-file
  digests, never normalized/regenerated JSON digests
  (`src/mcp/artifact_status.rs:182-184`).
- The decision and profile each bind the discovery file's digest
  (`discovery_sha256`), and the decision binds the profile digest
  (`profile_sha256`), the project key, the proposal index and the protocol
  revision (`src/mcp/decision.schema.json:25-44`;
  `src/mcp/profile.schema.json:30-37`; cross-checks in
  `src/mcp/disclosure.rs:827-844`).
- All three config identities are opened and admitted **before any config
  content is read** (`src/mcp/disclosure.rs:563-565`, `:691-699`
  `open_configs`).
- Every publication path re-hashes the actual retained raw bytes against the
  pins and re-verifies root ancestry and file generations
  (`src/mcp/disclosure.rs:260-283` `verify_inputs`;
  `src/linkage/fresh.rs:166-169` `verify_root`, `:458-477` `verify_file`;
  `src/mcp/artifact_status.rs:845-852` `verify_original`). A pin mismatch
  refuses the payload even when the changed file remains natively valid
  (`src/mcp/artifact_status.rs:1210` test).

### 4.3 Closed discovery schema

`src/mcp/discovery.schema.json` (`urn:forge:draft:mcp-project:1`):

- `additionalProperties: false` at the root and on every resource item
  (lines 7, 26); required `schema_version`, `project_key`, `resources`
  (lines 8–12); `schema_version` const `forge.mcp-project/1` (lines 14–16).
- `project_key` and resource `key`: `^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$`
  (lines 17–20, 37–40).
- `resources`: at most 1,000 items (line 23); each item requires `key`, `role`,
  `path`, `expected_sha256`, `native_identity`, `lifecycle_key`,
  `citation_label` (lines 27–35).
- `role` is a closed 15-value enum (lines 41–59): `policy-source`,
  `oscal-catalog-artifact`, `oscal-component-artifact`, `mapping-collection`,
  `applicability-manifest`, `applicability-report`, `trace-report`,
  `lifecycle-record`, `lifecycle-source`, `oscal-profile-artifact`,
  `oscal-ssp-artifact`, `framework-impact-manifest`, `successor-map`,
  `framework-impact-report`, `framework-impact-dispositions`.
- `expected_sha256`: `^[0-9a-f]{64}$` (lines 65–68); `path` ≤ 512 bytes
  (lines 60–64).
- The schema itself is marked an unapproved engineering draft: runtime
  portable-path, physical-alias, expected-identity, full-closure and
  lifecycle/current-approval checks are additionally required (lines 4–5).
  Schema admission alone never grants disclosure.

The `/2` successor (`src/mcp/discovery-v2.schema.json`, const
`forge.mcp-project/2`, lines 11–13) adds required `companion_resources`
(line 8) and keeps the same closed item shape; it belongs to the separate F20
checkpoint (§7), not to this approval.

### 4.4 Seven read-only tools, closed typed contracts

The fixed catalog is exactly seven tools with shipped, compile-time-embedded
input **and** output JSON schemas (`src/mcp/catalog.rs:12-48` `TOOLS`;
`src/mcp/tool-schemas/` — 14 files, one input and one output schema per tool):

| Tool | Required selector | Returned recorded data |
| --- | --- | --- |
| `list_policies` | none | Visible approved-current native policy roster with identities and pins |
| `search_requirements` | `query` | Deterministic lexical matches; optional `artifact_key` |
| `get_requirement` | `artifact_key`, `requirement_id` | One exact requirement locator and source citation |
| `trace_control` | `artifact_key`, `control_id` | Recorded direct/native Mapping relations, no inferred equivalence |
| `get_recorded_applicability` | `artifact_key` | Stored decisions for a fully matched current domain closure |
| `get_gap_summary` | `artifact_key` | Recorded gap classifications and complete counts |
| `get_artifact_status` | `artifact_key` | Recorded lifecycle approval/currentness; permitted schedule evaluation |

(Tool table: `docs/mcp.md:46-56`; PRD 067 M-7, line 152.)

- No caller can add methods, schemas or dynamic resolvers
  (`src/mcp/catalog.rs:11`). Invalid shipped schemas fail startup closed
  (`src/mcp/catalog.rs:93-112`; `src/mcp/mod.rs:26`).
- Tool arguments are closed objects: `deny_unknown_fields`
  (`src/mcp/catalog.rs:214-241` `Arguments`), schema validation before typed
  admission (`src/mcp/catalog.rs:163-178`), duplicate JSON keys refused at the
  wire layer (`src/mcp/wire.rs:242-283` strict decoder).
- Every encoded response is re-validated against the tool's **output** schema
  and the 256 KiB envelope bound before publication
  (`src/mcp/catalog.rs:191-203` `validate_encoded`;
  `src/mcp/stdio.rs:536-558`).
- Typed query core is shared library code, not a shell-out
  (`src/mcp/queries.rs:388-460` `prepare`; PRD 067 M-4, line 149).
- The visibility profile's `enabled_tools` is a closed enum of exactly these
  seven names, 1–7 unique entries (`src/mcp/profile.schema.json:38-54`);
  a disabled tool is refused (`src/mcp/queries.rs:396-398`).

## 5. Threat model: disclosure, null content, bounds, concurrency, inertness

### 5.1 Minimized approved/current disclosure

- Default responses contain bounded identifiers, hashes, classifications and
  citation metadata only; source prose, contact details, rationale, evidence
  content and absolute paths are omitted (`docs/mcp.md:78`; PRD 067 M-12,
  line 157). Identifiers/hashes can themselves be sensitive; minimization is
  not anonymization (`docs/mcp.md:78`).
- The profile fixes `identity_disclosure: "bounded-token-only"`,
  `approved_current_basis: "caller-established-recorded-policy"`,
  `report_content_mode: "minimized-metadata"`, `evidence_bytes: false`
  (`src/mcp/profile.schema.json:120-143`).
- Excerpts require **both** the hash-bound profile permitting that source
  **and** the caller's explicit `include_excerpt: true`; each excerpt is
  ≤ 512 UTF-8 bytes on a char boundary and carries `trust: "untrusted-content"`
  (`docs/mcp.md:80`; `src/mcp/queries.rs:36-37`, `:259-267`, `:641-655`).
- Lifecycle currentness is checked against the complete recorded fingerprint
  before any schedule calculation; schedule evaluation additionally requires
  explicit profile permission and a canonical `as_of` date — no hidden clock
  (`docs/mcp.md:82`; `src/mcp/profile.schema.json:132-137`, `:150-163`;
  `src/mcp/artifact_status.rs:632-702` `evaluate`).
- Static discovery (`server/discover`, `tools/list`) never opens project roots
  and discloses no project fact when the decision is absent
  (`src/mcp/stdio.rs:452-485` `execute`; `src/mcp/catalog.rs:287-303`;
  `docs/mcp.md:40`).

### 5.2 Fail-closed gate and null-content rules

- Per request, the disclosure gate is loaded fresh from the operator's raw
  selections. Missing or partial decision declarations return
  `DisclosureGate::Unavailable`; any admission failure collapses to
  unavailability (never a partial scope), while sticky interruption is
  preserved (`src/mcp/disclosure.rs:525-547` `load_disclosure_decision`).
- Unavailable results are **root-created null data**: fixed `schema_version`,
  `tool`, `availability`, a closed-domain `reason`, and `data: null` — never
  arbitrary error text, partial private facts, file paths or raw exceptions
  (`src/mcp/queries.rs:173-186` `QueryResponse`, `:496-510` `unavailable`;
  `docs/mcp.md:60`). Wire results set `isError: true` for unavailable data
  (`src/mcp/wire.rs:589-607` `ToolResult`; `docs/mcp.md:60`).
- The closed reason domain (`src/mcp/queries.rs:42-72` `Reason`):
  `approval-unavailable`, `unapproved`, `noncurrent`, `invalid-artifact`,
  `incomplete-closure`, `source-span-unavailable`, `visibility-refused`,
  `unsupported-role`, `not-found`, `generation-changed`,
  `output-bound-exceeded`, `query-budget-exceeded` (kebab-case serialization;
  the `Interrupted` variant is reserved and never serialized — transport
  cancellation suppresses the response instead).
  Unknown and known-but-undisclosed keys share one fixed safe refusal — no
  lookup leak (`src/mcp/artifact_status.rs:853-869`, `:1298-1311` tests).
- Per-artifact availability refuses: missing digest, changed content vs pin,
  invalid native artifact, identity mismatch, missing lifecycle binding,
  unapproved or noncurrent lifecycle tuples
  (`src/mcp/artifact_status.rs:632-702` `evaluate`, `:704-750`
  `complete_closure`). Detached values or hashes are not proof; actual
  captured original generations and complete lifecycle/approval dependency
  closure govern visibility (ticket `#55`).
- Available-but-optional fields are explicit `null`, never omitted guesses:
  source spans, citation identity, schedule, native mapping IDs
  (`src/mcp/queries.rs:225-257`, `:298-313`, `:340-365`).

### 5.3 Bounded input, work and output

All limits are admitted before retained growth (ticket `#55`; PRD 067 M-11,
line 156; summary table `docs/mcp.md:86-100`):

| Boundary | Limit | Citation |
| --- | --- | --- |
| Captured original slots (both roots, incl. typed absences) | 1,001 | `src/mcp/disclosure.rs:32`; `src/mcp/artifact_status.rs:80-83` |
| Retained raw file bytes (shared pool) | 50 MiB | `src/mcp/disclosure.rs:34`; `src/mcp/artifact_status.rs:91` |
| Each config original | 1 MiB | `src/mcp/disclosure.rs:36`; `src/mcp/artifact_status.rs:86` |
| Per-resource bytes | ≤ 10 MiB by role | `src/mcp/artifact_status.rs:88` |
| Relationship admissions (shared, monotonic) | 100,000 | `src/mcp/disclosure.rs:38` |
| Request frame incl. LF | 64 KiB | `src/mcp/wire.rs:18` |
| JSON tree depth / nodes | 64 / 100,000 | `src/mcp/wire.rs:22-24` |
| String request ID / method | 128 / 256 bytes | `src/mcp/wire.rs:26-28` |
| Complete response envelope incl. LF | 256 KiB | `src/mcp/wire.rs:20`; `src/mcp/queries.rs:27` |
| Page rows | 1–50 (default 50) | `src/mcp/queries.rs:31`; `src/mcp/catalog.rs:282-285` |
| Search tokens / query bytes | 32 / 4,096 | `src/mcp/queries.rs:33-35` |
| Excerpt bytes | 512 | `src/mcp/queries.rs:37` |
| Waiting accepted requests | 2 (plus one active) | `src/mcp/stdio.rs:32` |
| Accepted-ID roster (no eviction/reuse) | 1,024 | `src/mcp/stdio.rs:34` |
| Accepted work budget (incl. queue wait) | 10 s | `src/mcp/stdio.rs:30` |
| Pump bursts | 4,096 bytes / 8 frames | `src/mcp/stdio.rs:36-38` |
| Streaming scratch (independent of pool) | 32 KiB | `src/linkage/fresh.rs:31` |

Oversized frames are drained to LF without retaining further payload
(`src/mcp/wire.rs:139-171` `Framer`); oversized/malformed input never grows
unbounded state. Bounded private buffers mean a failed write/flush may already
have emitted a non-retractable prefix; the worker reports that honestly and
retires (`src/mcp/stdio.rs:326-348` `write_complete`; `src/mcp/mod.rs:28`).

### 5.4 External owner-record handoff (six rows)

The decision file must carry `policy_decisions` rows covering exactly six
subject/role pairs, each `disposition: "approved"`, each binding the proposal
index, profile pin and discovery pin, and each declaring
`declared_owner_key`, `source_record_key`, `source_record_sha256` and a
canonical `recorded_on` date (`src/mcp/disclosure.rs:827-876`
`validate_decision`; expected set at lines 865–872;
`src/mcp/decision.schema.json:262-340`):

| Subject | Role |
| --- | --- |
| D067-P1 | product |
| D067-P2 | product |
| D067-S1 | security |
| D067-E1 | engineering |
| D067-S2 | product |
| D067-S2 | security |

A digest proves which bytes were selected; it does not authenticate a named
person or establish lawful provenance. Implementation-design approval does not
supply these project-specific owner records (`docs/mcp.md:38`; approval JSON
line 10). The decision schema pins `proposal_index_sha256` to the historical
v2 index hash and `protocol_revision` to `2026-07-28`
(`src/mcp/decision.schema.json:37-42`); `operator_boundary` is
`operator-selected-declared-owner-record` (lines 43–45).

### 5.5 Cancellation reader, one active worker, bounded queues, generation fences

- **Single thread, single owner.** One thread owns stdin framing, admission,
  every captured scope and stdout publication; no reader thread is spawned or
  detached (`src/mcp/stdio.rs:1-7`, `:81-86` `InputPort` contract,
  `:412-449` run loop). Windows uses observed synchronous byte-pipe input;
  pending mode queries refuse startup; other input profiles refuse
  (`src/mcp/windows_input.rs:1-5`; `src/mcp/stdio.rs:695-712`).
- **Cancellation reader stays alive during work.** Every work checkpoint pumps
  a bounded amount of *ready* input (zero-wait `service`) without recursively
  dispatching queued work, so `notifications/cancelled` is observed while a
  query runs (`src/mcp/stdio.rs:350-360` `observe`, `:392-396` `checkpoint`,
  `:251-291` `service`, `:293-315` `message`). Cancellation stops known active
  work or removes known queued work; unknown/late IDs do nothing
  (`src/mcp/stdio.rs:184-193` `cancel`). The free-form cancellation reason is
  discarded (`src/mcp/wire.rs:122-124`, `:204-215`).
- **One active worker, bounded queues.** Admission checks capacity and ID
  uniqueness before any container grows: one active slot, at most two waiting,
  1,024-ID roster with no eviction; a repeated accepted ID retires the whole
  transport rather than emitting a competing response
  (`src/mcp/stdio.rs:142-162` `accept`, `:164-182` `next`;
  `src/mcp/mod.rs:30`; `docs/mcp.md:74`). Each accepted request carries one
  absolute 10 s deadline including queue wait
  (`src/mcp/stdio.rs:152-153`, `:195-205` `stop`).
- **Original-generation fences before publication.** Immediately before any
  trusted byte is written, the worker pumps ready input once more
  (`enter_publication`), then re-verifies the complete captured generation:
  root ancestry, decision/profile/discovery raw re-hashing, and every held
  original's file generation (`src/mcp/stdio.rs:361-380`;
  `src/mcp/queries.rs:383-385` `PreparedQuery::verify_inputs`;
  `src/mcp/disclosure.rs:260-283`; `src/linkage/fresh.rs:166-169`,
  `:458-477`). Stage fences bracket capture/validate/prepare/retain phases
  (`src/workspace/preparation.rs:56-76` `Stage`;
  `src/mcp/artifact_status.rs:836-843` `fence`).
- **Suppression rules.** Cancellation, EOF and shutdown suppress success
  publication; only a genuine deadline may emit a fixed null-data failure
  after the deadline (`src/mcp/stdio.rs:371-388` `publish`/`terminal`,
  `:591-608` `stopped`; `docs/mcp.md:102`). EOF retires all accepted work
  without success publication (`src/mcp/stdio.rs:70-78`, `:256-262`).
  Generation-bound cursors refuse cross-generation continuation
  (`src/mcp/queries.rs:61-62`, `:210-222`; `docs/mcp.md:58`).

### 5.6 Inert untrusted caller content

- Per-request `_meta` must carry the namespaced protocol revision and client
  capabilities; metadata is bounded and extensible but **never cached as
  authority** (`src/mcp/wire.rs:29-32`, `:220-238`, `:285-301` `valid_meta`).
  Reserved `io.modelcontextprotocol/*` keys are validated; unknown reserved
  namespaces are refused (`src/mcp/wire.rs:303-347`).
- `clientInfo`, capabilities, annotations and request IDs grant no access;
  tool annotations are hints only (`docs/mcp.md:70`;
  `src/mcp/catalog.rs:318-340`).
- Retrieved text is inert data: excerpts are labeled `untrusted-content`,
  never parsed as instructions, and no tool chains another tool
  (`src/mcp/queries.rs:259-267`; `docs/mcp.md:80`; PRD 067 M-10, line 155).
  The fixed discovery `instructions` string states source inertness
  (`src/mcp/catalog.rs:301-316`).
- Fixed runtime errors only: private OS/domain/serde strings are never
  serialized (`src/mcp/stdio.rs:87-100` `RunError`; `src/mcp/mod.rs:24-35`;
  `src/mcp/wire.rs:381-384`).

### 5.7 No HTTP/SSE, network, or process execution

- The worker is stdio-only (PRD 067 M-1, line 146; `docs/mcp.md:3,9`). There is
  no HTTP/SSE listener, no outbound fetch, no subprocess launch, no plugin
  load, no dynamic code execution, no hidden staging writes
  (PRD 067 M-13, line 158; ticket `#55`).
- The visibility profile schema **constants** `network: false` and
  `process_execution: false` make any other value schema-invalid
  (`src/mcp/profile.schema.json:144-149`).
- Verified by search at `b4226bb`: `src/mcp/` contains no `Command::new`,
  `TcpListener`, or HTTP client usage (the only `std::process` reference is
  `std::process::id()` in a test fixture,
  `src/mcp/windows_input_tests.rs:88`).
- Authenticated local HTTP transport is an explicit **Could-have future**
  under a separate threat model (PRD 067 C-1, line 172), not part of F11.

## 6. Wire contract: MCP 2026-07-28 over stdio

- **Revision.** Exactly `2026-07-28`, selected immutably; no session
  negotiation, no legacy `initialize` handshake
  (`src/mcp/wire.rs:15-16` `REVISION`; `docs/mcp.md:70`). Unsupported revisions
  receive the normative version error carrying the bounded requested revision
  (`src/mcp/wire.rs:226-238`).
- **Framing.** One UTF-8 JSON-RPC 2.0 object per newline; no batches; no BOM;
  exact integer/string IDs (fractional, exponent, null, boolean IDs refused)
  (`src/mcp/wire.rs:174-199`, `:35-76` `RequestId`; `docs/mcp.md:64,74`).
- **Mandatory `server/discover`.** Implemented alongside `tools/list` and
  `tools/call`; unknown methods are protocol errors
  (`src/mcp/catalog.rs:149-181` `admit`; `docs/mcp.md:70`). Method parameters
  are closed (`src/mcp/catalog.rs:205-211` `closed`); `tools/list` is one
  complete page and refuses cursors (`src/mcp/catalog.rs:155-162`).
- **Request-specific namespaced metadata.** Every request supplies
  `io.modelcontextprotocol/protocolVersion` and
  `io.modelcontextprotocol/clientCapabilities` in `params._meta`
  (`src/mcp/wire.rs:29-32`, `:220-238`; example `docs/mcp.md:66-68`).
- **Complete result typing.** Discovery/list results carry
  `resultType: "complete"`, `ttlMs: 0`, `cacheScope: "private"`
  (`src/mcp/catalog.rs:287-316`; `docs/mcp.md:72`). Tool results carry fixed
  inert text plus schema-checked `structuredContent`
  (`schema_version: "forge.mcp-query/1"`) and explicit `isError`
  (`src/mcp/wire.rs:571-607`; `docs/mcp.md:60`). These data versions are
  separate from the MCP revision (`docs/mcp.md:60`).
- **Capabilities.** Only `tools` is advertised; resources/F20 metadata remain
  separate required implementation (`src/mcp/catalog.rs:295-296`, `:318-323`).
- Historical initialize-first checklists do **not** establish current
  conformance (ticket `#55`); conformance must be demonstrated with the
  modern-era clients of §8.

## 7. F20 `/2` checkpoint — explicitly out of scope for this approval

The `/2` family is a distinct, narrower development checkpoint selected only
by the explicit `--declaration-family v2` flag (default remains `v1`)
(`src/cli/mod.rs:403-410`, `:905-909`; `src/mcp/mod.rs:56-84`
`serve_selected`; `docs/mcp.md:5,110-150,157-269`). Its closed declarations
(`discovery-v2.schema.json`, `profile-v2.schema.json` const
`forge.mcp-visibility/2` at lines 29–31, `decision-v2.schema.json` const
`forge.mcp-disclosure-decision/2` at lines 19–21,
`index-build-intent.schema.json` const `forge.mcp-index-build-intent/1` at
lines 19–21) and the offline `forge mcp check-index-inputs` consumer
(`src/cli/mod.rs:923-935`, `:2405-2412`; `src/mcp/mod.rs:38-54`) build no index
and publish nothing. Its null-content rules (policy-source identity fields
null; schedule `not-evaluated`) and whole-gate refusal of unsupported visible
families are documented at `docs/mcp.md:176-196`. Evidence-metadata trace
(S-1), static-report resources (S-2), the deterministic index builder (S-4)
and genuine owner records for hash-pinned profiles (S-3) remain open
(PRD 067 line 252). None of these gates are closed or re-decided by this
document (packet lines 116–121, 161–166).

## 8. Two-client conformance pair — OWNER DECISION REQUIRED

PRD 067 Definition of Ready requires "Two independent MCP clients pass
interoperability fixtures" (line 231). Ticket `#55` warns that historical
initialize-first checklists do not establish current conformance. The proposed
pair (packet §1 "Two-client conformance choice", lines 123–149) is:

1. **MCP Inspector, pinned to the `modern` protocol era.** The official
   conformance/debug tool; its `modern` era pins exactly `2026-07-28` with no
   fallback, so a non-modern server fails loudly
   ([Inspector protocol eras](https://modelcontextprotocol.io/docs/2026-07-28/tools/inspector/protocol-eras)).
   **Caveat:** the Inspector default era is `legacy` (initialize-first); the
   harness MUST pass `--protocol-era modern` (or `protocolEra: modern`) or it
   will not exercise `server/discover`. This is the deterministic conformance
   baseline for namespaced metadata and complete result typing.
2. **Claude Code, v2 client runtime.** A real agent client whose v2 runtime is
   built on MCP TypeScript SDK 2.0, "which adds MCP protocol revision
   2026-07-28," default on v2.1.274+ in sessions that do not fetch feature
   flags ([Claude Code MCP client runtimes](https://code.claude.com/docs/en/mcp)).
   Exercises `tools/call`, inert-content handling and cancellation under a
   production agent.

*Alternative considered:* two agent CLIs (e.g. Claude Code + Goose). Rejected
as the primary pair because Goose's 2026-07-28 support was not verified, while
the Inspector is the spec-authoritative conformance instrument (packet
lines 145–149). The exact Inspector and Claude Code versions must be recorded
at harness time; both must pass the **same** fixtures.

**This pair is a proposal. Naming the two clients is an explicit owner
decision within D067; no conformance is claimed or earned by this document.**

## 9. Retained verification evidence (history, not new acceptance)

- [Pre-PR verification v3](2026-10-04-f11-pre-pr-verification-v3.md) (JSON
  sibling, `source_basis` `5f57f1ab…`, working-tree pins — not a published
  commit): dedicated-target isolated whole-workspace/all-feature LLVM passed
  **3,536 tests, 0 failed, 3 ignored** with strict all-target/all-feature
  Clippy (md line 7); an actual CLI smoke of **22 complete exchanges** covered
  missing/half-paired owner declarations, all seven unavailable tools,
  refusal/recovery, trace-log isolation and normal EOF (md line 9). Earlier
  shared-target counts (2,343 / 3,431) are **withheld** as F11 evidence
  (md line 5). The complete evidence archive is pinned in the JSON
  (lines 64–68) and retained at
  `docs/plans/evidence/2026-10-04-f11-current-complete-evidence-v3.json.gz`.
- [Hosted source verification v2](2026-10-04-f11-hosted-source-verification-v2.md):
  candidate `75785a2a`, tested merge `9f52ea96`; Windows job `111565827638`
  passed tests, API contract validation, lint, release build and maintained
  local client conformance — library **2,410/0/0**, all **25** Windows input
  controls including native byte/message/overlapped pipe cases (md line 3;
  JSON lines 10–35). Overall CI **failed**: cargo-vet found **20 dependencies
  lacking safe-to-deploy audits** (md line 7; JSON lines 56–79). That gap is
  recorded, not closed, by this re-pin.
- Open gates carried by the v3 record (JSON lines 54–62): two independent MCP
  clients; real project owner records and lawful disclosure authority;
  adjudicated corpus/raw baseline; MCP resources and remaining F20 work;
  full-goal final documentation review; dependency/schema/pilot/security/
  product/release gates and verified 2.0.0 RC.

These are authentic engineering checks only — no human, client, corpus, audit,
product, release, merge or publication acceptance (v3 JSON line 63).

## 10. Re-pin mechanics and remaining gates

1. This document is committed to `docs/plans/` on branch
   `docs/f11-proposal-repin`; its exact byte count and SHA-256 are recorded in
   the draft PR and in the delivery comments on veans `#56` and `#55` so the
   owner's fresh D067 approval can bind reviewable bytes.
2. The owner's D067 ratification must reference **this file's** committed
   hash — not the lost `60c8dff1…` hash — and must separately name the
   two-client pair (§8). The 2026-10-04 approval JSON is left untouched as
   history; no approval is recorded by this re-pin.
3. Still required after ratification (unchanged): the six project-specific
   owner-record rows (§5.4); two-client conformance fixtures (§8); the
   adjudicated corpus and raw-document baseline (PRD 067 M-16, line 161);
   dependency audit/vet closure; security, product and release acceptance;
   and the separate F20 `/2` qualification (§7).
4. Known limitation, recorded not fixed: the runtime gate's
   `proposal_index_sha256` constant still binds the historical v2 index hash
   `d52da047…` (`src/mcp/disclosure.rs:40`;
   `src/mcp/decision.schema.json:37-39`), whose bytes are also absent from the
   repository. Re-binding the runtime to successor proposal/index bytes is a
   separate reviewed code change requiring its own owner approval; it is out
   of scope for this documents-only re-pin.

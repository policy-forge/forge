# Owner decision packets — 2026-11-12

Date: 2026-11-12 (America/Los_Angeles). Baseline: `main` at `b4226bb`
(PR #259, F08 POA&M completion). Status: **Proposed; no owner disposition is
recorded by this packet.** It closes no requirement, authorizes no merge,
publication, or external contact, and changes no board status.

These packets unblock phases **P3** and **P5** of the
[remaining-work completion plan](2026-11-12-remaining-work-completion-plan.md)
(§4 decision register, §5 phases). That plan is the coordinator's working
document (local commit `11e1698` on `main`, not yet on `origin/main` at
`b4226bb`); section references below are to it.

The [2026-10-25 owner dispositions](2026-10-25-owner-dispositions.md) remain
**binding** (D064, D068, F10 add-only export). Where a question below is already
answered by that record, this packet **reconciles and cites** the answer rather
than re-deciding it. Recommendations are concrete and conservative; every option
names its tradeoff. Nothing here is pre-approved.

## Reading guide

| Packet | Plan §4 item | Tickets | Disposition state |
|---|---|---|---|
| [1. D067 — F11 discovery + threat model](#1-d067--f11-project-discovery-and-threat-model) | 1 | #55, #56 | Design approved 2026-10-04; project ratification + two-client choice open |
| [2. D069 — F12/F13 scope split](#2-d069--f12f13-scope-split) | 2 | #21, #59 | **Already recorded** in D068 (2026-10-25); reconcile only |
| [3. F10 Q1–Q7](#3-f10-q1q7-reconciliation) | 4 | #19, #68, #69, #70 | **Q1–Q6 answered**; Q7 within scope (audit over-reading), confirm |
| [4. F12 M-10 patch contract](#4-f12-m-10-proposed-edit-patch-contract) | 5 | #21, #59 | Open design proposal |
| [5. F12 M-13 / M-15](#5-f12-m-13-sensitive-export-and-m-15-windows-publication) | 6 | #21, #71 | M-13 deferred by D068; M-15 mechanism merged, qualification open |
| [6. F16 confinement design](#6-f16-confinement-design-proposal) | 8 | new under #2 | Open design proposal |
| [7. D068 quorum verification](#7-d068-quorum-disposition-verification) | 3 | #21 | Recorded; verified against merged code, no re-decision |

Cross-cutting reconciliation findings are collected in
[§8](#8-reconciliation-findings).

---

## 1. D067 — F11 project discovery and threat model

**Plan §4 item 1; tickets #55 (id 1450), #56 (id 1451).**

### Exact question

Approve the **implemented confined-roots project-discovery model and read-only
threat model** as the recorded D067 approval that gates F11 *public*
implementation (#55), and **name the two independent MCP clients** that will
establish conformance to MCP revision **2026-07-28**.

### What is already approved (do not re-decide)

The [F11 implementation-design approval](2026-10-04-f11-implementation-design-approval-v1.json)
records `"answer": "Approve this design for implementation"` (authority: direct
human reply, 2026-10-04). Its `scope` covers "F11/F20 discovery and threat-model
design for implementation: seven read-only tools, minimized approved/current
disclosure, hash-pinned profiles with explicit external owner-record handoff and
MCP2026-07-28 stdio." Its `unearned` list explicitly **withholds**: new
dependency/supply-chain approval, authenticated owner identity, native/source/
currentness implementation, **two-client interoperability**, adjudicated corpus,
and pilot/product/security/release acceptance. `runtime_owner_record` is "Not
supplied or inferred."

So the *design* is approved; what remains open is (a) ratifying the **implemented**
model as the durable approval artifact, (b) the **two-client** conformance choice,
and (c) the **project-specific owner records** needed for real disclosure.

### Evidence: the implemented model

- **Two disjoint confined roots.** [`docs/mcp.md`](../mcp.md) "Start a worker"
  (lines 17–26): `--project` and `--decision-root` must be disjoint; neither may
  contain or alias the other; symlinked ancestry and physical aliases are
  refused; `--decision-sha256` / `--profile-sha256` pin the original decision and
  visibility files.
- **Closed discovery schema.** [`src/mcp/discovery.schema.json`](../../src/mcp/discovery.schema.json):
  `additionalProperties: false`, `schema_version` const `forge.mcp-project/1`,
  ≤1000 resources, each with `expected_sha256`, `native_identity`,
  `lifecycle_key`, and a closed `role` enum. Visibility
  ([`profile.schema.json`](../../src/mcp/profile.schema.json)) and decision
  ([`decision.schema.json`](../../src/mcp/decision.schema.json)) schemas are
  likewise closed.
- **Six subject/role owner pairs.** `docs/mcp.md` line 38: product for D067-P1
  and D067-P2, security for D067-S1, engineering for D067-E1, both product and
  security for D067-S2. Missing/partial declarations preserve static discovery
  and return fixed unavailability (line 40).
- **Fail-closed read-only transport.** [`src/mcp/mod.rs`](../../src/mcp/mod.rs)
  `serve()` (lines 12–36) maps every runtime error to a fixed safe message; no
  write/network/process capability exists.
- **Protocol revision pinned.** [`src/mcp/disclosure.rs`](../../src/mcp/disclosure.rs)
  line 42 `const PROTOCOL: &str = "2026-07-28"`;
  [`src/mcp/wire.rs`](../../src/mcp/wire.rs) line 16 `REVISION`;
  [`src/mcp/catalog.rs`](../../src/mcp/catalog.rs) line 151 handles
  `server/discover`. Request framing uses namespaced
  `_meta.io.modelcontextprotocol/protocolVersion` (`docs/mcp.md` line 67).
- **Threat-model boundaries.** PRD 067
  ([`067-prd-read-only-mcp-governance-interface.md`](../PRD/067-prd-read-only-mcp-governance-interface.md))
  M-1…M-16 (lines 146–161), AC-1…AC-6 (lines 184–189), and the Risks table
  (prompt injection, excess disclosure, stale-as-approved, search-as-answer,
  write creep). `docs/mcp.md` "Bounds, cancellation and shutdown" (line 84)
  records one active worker, bounded queues, cancellation reader, and
  original-generation fences.

### Options and tradeoffs

- **A — Approve the implemented confined-roots model as-is (recommended).**
  The evidence above shows disjoint roots, alias refusal, raw-file pins,
  fail-closed unavailability, inert untrusted content, and no network/process
  surface — exactly the approved design. *Tradeoff:* approval binds the current
  implementation; any later schema/discovery change needs a fresh approval. This
  is the conservative, evidence-backed choice and matches the plan's
  recommendation (§4 item 1).
- **B — Approve with an explicit client allowlist.** PRD 067 Open Question
  [Security, blocking] (line 221) asks whether stdio isolation suffices or an
  allowlist is needed. *Tradeoff:* an allowlist adds a client-identity surface
  that the threat model deliberately refuses — ticket #55 states "a manifest or
  self-declared MCP client identity cannot supply that approval." No evidence
  shows stdio isolation insufficient. Adds scope and a new decision surface for
  no demonstrated gain.
- **C — Defer ratification until F20 `/2` qualification.** *Tradeoff:* blocks #55
  unnecessarily. The v1 default path is separately qualified
  ([F11 pre-PR verification v3](2026-10-04-f11-pre-pr-verification-v3.md):
  3,536 tests, 0 failed, 3 ignored, dedicated target). The `/2` candidate is a
  distinct, narrower development checkpoint (`docs/mcp.md` line 110) and its
  gates should not hold the v1 approval hostage.

### Two-client conformance choice (explicit)

PRD 067 Definition of Ready (line 231) requires "Two independent MCP clients pass
interoperability fixtures." Ticket #55 warns that "Historical initialize-first
checklists do not establish current conformance." Recommended pair:

1. **MCP Inspector, pinned to the `modern` protocol era.** The official
   conformance/debug tool. Its `modern` era "Pin[s] exactly `2026-07-28`. No
   fallback, so a non-modern server fails loudly"
   ([Inspector protocol eras](https://modelcontextprotocol.io/docs/2026-07-28/tools/inspector/protocol-eras)).
   **Caveat:** the Inspector default era is `legacy` (initialize-first); the
   harness MUST pass `--protocol-era modern` (or `protocolEra: modern`) or it
   will not exercise `server/discover`. This is the deterministic conformance
   baseline for namespaced metadata and complete result typing.
2. **Claude Code, v2 client runtime.** A real agent client. Its v2 runtime is
   built on MCP TypeScript SDK 2.0, "which adds MCP protocol revision
   2026-07-28," and is the default on v2.1.274+ in sessions that do not fetch
   feature flags
   ([Claude Code MCP client runtimes](https://code.claude.com/docs/en/mcp)).
   Exercises `tools/call`, inert-content handling, and cancellation under a
   production agent.

*Alternative considered:* two agent CLIs (e.g. Claude Code + Goose). Rejected as
the primary pair because Goose's 2026-07-28 support was not verified here, while
the Inspector is the spec-authoritative conformance instrument. Record the exact
Inspector and Claude Code versions at harness time; both must pass the **same**
fixtures.

### Recommendation

Approve **Option A** (implemented confined-roots model + read-only threat model)
as the recorded D067 project approval, **conditional on re-pinning the durable
proposal/threat-model bytes into the repo** (see finding F4 in §8 — the approved
proposal currently lives at a `/private/tmp` path no longer on disk). Name the
two-client pair **Inspector (`modern`) + Claude Code (v2 runtime)**. Keep the six
project-specific owner-record rows a separate, still-required disclosure gate;
design approval does not supply them.

### What it unblocks

- #56 → #55: audit the merged v1 worker against the approved threat model and
  build the two-independent-client conformance harness (plan §5 P3).
- F20 `/2` checkpoint gates proceed independently; they are not folded into this
  approval.

---

## 2. D069 — F12/F13 scope split

**Plan §4 item 2; tickets #21 (id 1416), #59 (id 1454).**

### Exact question

Which of the remaining M-3 adapters (057 impact, 063 Assessment Results, 064
POA&M, 066 suggestions, clause-level 061) and the S-1 interactive workspace
client land in **F12** versus **F13**?

### This is already recorded — reconcile, do not re-decide

The [D068 disposition](2026-10-25-owner-dispositions.md) "Scope split"
(lines 47–50) **already answers this**:

> F12 is the 055/056 portable review core. The remaining M-3 adapters, M-10
> proposed edits, S-1 response-file client, S-2 signed envelopes, and S-4
> supersession for the Lifecycle `/2` and Authoring `/3` families belong to F13.
> Ticket #21's "21 acceptance entries" means 17 Musts plus 4 Shoulds.

Ticket #21's description already carries this "Scope update 2026-10-25 (owner
disposition, D068)" and links F13 (task 1454). The plan (§1, lines 9–10) states
the 2026-10-25 dispositions "remain binding."

### Evidence: current module layout

- [`src/review/adapters/mod.rs`](../../src/review/adapters/mod.rs) exposes only
  `mapping` and `applicability` — the F12 055/056 core.
- [`src/review/wire.rs`](../../src/review/wire.rs) `Domain` enum (lines 38–43) is
  `MappingAssertion` / `ApplicabilityDecision` only.
- The Lifecycle `/2` ([`wire_v2.rs`](../../src/review/wire_v2.rs)) and Authoring
  `/3` ([`wire_v3.rs`](../../src/review/wire_v3.rs)) families exist as separate
  typed successors (F21), consistent with the split.
- Impact capture ([`impact_capture.rs`](../../src/review/impact_capture.rs))
  exists for native preparation but is **not** a review-queue adapter; the 057/
  063/064/066/clause-061 adapters remain F13 work (ticket #21 checklist).

### Options and tradeoffs

- **A — Keep the recorded D068 split (recommended).** All five remaining M-3
  adapters + M-10 + S-1 + S-2 + S-4(`/2`,`/3`) → **F13**; F12 stays the 055/056
  core. *Tradeoff:* none new — this is the binding owner decision. F12 can close
  its core checkboxes by evidence; F13 (#59) owns the adapter backlog.
- **B — Adopt the plan §4 item 2 alternative** ("portable-file adapters → F12;
  interactive client → F13"). *Tradeoff:* this **contradicts** the recorded
  disposition, which sends *all* remaining adapters to F13. Re-opening it would
  require a new owner decision and would invalidate ticket #21's already-updated
  scope. Not recommended.

### Recommendation

**Keep Option A.** Record that the plan's §4 item 2 phrasing is superseded by the
binding D068 scope split; do not re-decide. Note the identifier collision
(finding F3, §8): the plan labels this "D069," but the
[2026-10-02 packet](2026-10-02-roadmap-owner-decisions.md) uses **D069 for
dependency policy (F02/F03)**. Recommend the plan relabel this item "D068 scope
split" (or assign a fresh ID) to avoid cross-referencing the wrong decision.

### What it unblocks

- F13 (#59): adapter implementation ordering (057/063/064/066/clause-061), M-10,
  S-1 client (reuses PRD 062 workspace APIs, plan §5 P3), S-4(`/2`,`/3`).
- F12 (#21): close the delivered 055/056 core checkboxes by evidence.

---

## 3. F10 Q1–Q7 reconciliation

**Plan §4 item 4; ticket #19 (id 1414) comment 1307; tickets #68, #69, #70.**

The F10 coordinator audit (ticket #19, comment 1307) posed Q1–Q7. The
[2026-10-25 dispositions](2026-10-25-owner-dispositions.md) "F10" section
(lines 64–84) and D064 (lines 13–36) **already answer Q1–Q6**; Q7 is a scope
confirmation, not an override (see the Q7 sub-item).
Per the plan (§5 P3), the expected outcome is **evidence closure, not new code**.

| Q | Question (comment 1307) | Already answered? | Binding disposition | Recommendation |
|---|---|---|---|---|
| Q1 | S-3 initial states other than `planned`? | **Yes** | F10 lines 66–67: "Export seeds new `planned` work only." | Keep new-planned-work only; later transitions authored in `forge.poam/1`. |
| Q2 | Export into an existing plan? | **Yes** | F10 lines 68–70: add-only approved; new items, new stable identities; M-13 baseline reports additions. | Keep add-only (merged #257). |
| Q3 | Open/overlapping epoch windows? | **Yes** | F10 line 71: "Epoch windows stay sealed and non-overlapping." | Keep refusal (`validate_append_window`). |
| Q4 | Recurring-risk identity / branching / native continuity? | **Yes** | F10 lines 72–76: UUIDs never reused; family key + predecessor UUID in Forge-namespaced props; strictly linear, no branch/merge. | Keep distinct identities + native props. |
| Q5 | Mixed envelopes (different AP/context/actors/evidence)? | **Yes** | F10 lines 77–80: appended epoch may add parties/actors/evidence; different AP/context → new AR document (single `import-ap`). | Keep additive-only within one context. |
| Q6 | Cross-epoch references? | **Yes** | F10 lines 81–82: backward observation refs only; earlier findings/risks refused. | Keep backward-observation-only. |
| Q7 | Assessment Results in `forge validate`? | **Within scope** | PRD 054 excludes AR *generation* and adding AP/SSP to `forge validate`, **not** AR *validation*; #253 added validation only. | **Confirm** AR validation is in scope; optional clarifying note (see below). |

### Code evidence that the answered behavior is merged

- Q1: [`src/poam/risk_authoring.rs`](../../src/poam/risk_authoring.rs) closed
  first-plan profile, test `first_plan_state_history_and_risk_kind_are_closed`.
- Q2: add-only export merged via #257 (`63b57bd` "feat(poam): add opt-in add-only
  reviewed-risk export into existing plans"), through #259.
- Q3: [`src/poam/assessment_epochs.rs`](../../src/poam/assessment_epochs.rs)
  `validate_append_window` refuses open/overlapping windows.
- Q4: `assessment_epochs.rs` lines 44–46 (family-key + predecessor-UUID native
  props); merged via #256 (`ca17ac6` "feat(poam): admit backward observation
  edges and linear continuity props").
- Q6: `assessment_epochs.rs` line 43 `LINKS = "prior_observation_links"`,
  `resolve_observation` (line 348) admits only earlier-epoch observations.
- D064 terminal states (shared with F08): admitted via #255 (`a8fd6e4`
  "feat(poam): admit D064 asserted terminal states and linked reopen items");
  `completed-asserted` / `accepted-risk-asserted` require an independent
  reviewer + evidence hash/capture time (dispositions lines 21–28).

### Q7 sub-item — confirm AR validation is within scope (coordinator-corrected)

**The audit's "scope expansion" flag was an over-reading.** The F10 audit
(comment 1307) and the plan §4 item 4 framed Q7 as Assessment Results in
`forge validate` being "beyond PRD 054." That is incorrect. PRD 054
([`054-prd-oscal-1-2-3-compatibility.md`](../PRD/054-prd-oscal-1-2-3-compatibility.md))
Out of Scope excludes exactly:

> - Assessment Results or POA&M **generation**. (line 72)
> - Adding **Assessment Plan or SSP** to the public `forge validate`, `forge export`, or round-trip model set. (line 73)

It does **not** exclude Assessment Results *validation*. **PR #253 added AR
validation only** (auto-detect the `assessment-results` root + `--schema-type
assessment-results`; no generation, no export/diff/round-trip), and
[`docs/usage-guide.md`](../usage-guide.md) lines 203–221 already document it.
There was **no exclusion to override**.

**State of record.** Ticket **#70** (id 1477) delivered via **PR #253**,
**merged 2026-10-26** (merge commit `5e79563`, head `2249500`). Comment 1317:
"No user decision is open for this ticket"; `cargo test` 4454 passed / 0 failed /
3 ignored. The [dispositions](2026-10-25-owner-dispositions.md) lines 83–84 call
it "a small scope expansion beyond PRD 054's original boundary" — that wording
inherits the same over-reading; it records owner approval but mischaracterizes
PRD 054, which never excluded AR validation.

- **Exact question:** Confirm that Assessment Results *validation* in
  `forge validate` is **within scope** (it is — PRD 054 lines 72–73 exclude
  generation and AP/SSP, not AR validation).
- **Options:** (A) Confirm in scope; **no PRD 054 amendment required**
  (recommended); (B) optionally add a one-line clarifying note to PRD 054 that
  AR *validation* is admitted while AR *generation* remains excluded, to prevent
  future misreadings (low-cost, optional); (C) treat it as an override needing
  ratification — rejected, there was no exclusion to override.
- **Recommendation: A (+ optional B).** No amendment is required because nothing
  was excluded. If the owner wants belt-and-suspenders clarity, add the optional
  note (B); do not reframe the merged behavior as an override. **Scope limit:**
  this covers **Assessment Results validation only**. AR generation stays
  excluded (line 72); Assessment Plan / SSP stay out of `forge validate` /
  `export` / round-trip (line 73).

### Recommendation

Keep export **"new planned work + add-only into existing plans"**; later
transitions authored in `forge.poam/1` (Q1–Q6 already binding). **Confirm Q7**
(AR validation is within scope; optional clarifying note). Close #68/#69/#70 by
evidence; no new code.

### What it unblocks

- #68 (S-4 epochs), #69 (S-3 export), #70 (AR validate): evidence closure.
- #66 (dispositions record): close-ready (see §8, F6).

---

## 4. F12 M-10 proposed-edit patch contract

**Plan §4 item 5; PRD 068 M-10 (line 135), AC-5 (line 169); tickets #21, #59.**

### Exact question

What is the **proposed-edit envelope shape**, with **target-hash binding** and
**AC-5 wrong-hash rejection**, consistent with the existing `src/review` wire
formats?

### Current state

All three response families carry an explicit-null edit slot:
[`wire.rs`](../../src/review/wire.rs) lines 363–364,
[`wire_v2.rs`](../../src/review/wire_v2.rs) lines 200–201,
[`wire_v3.rs`](../../src/review/wire_v3.rs) lines 204–205 — `proposed_edit: ()`
("domain edit syntax is unsupported by this slice"). Responses already bind
`subject_sha256`, `context_sha256`, `policy_sha256`, and `queue_raw_sha256`
(`wire.rs` `ResponseDocument`, lines 322–360). D068 (dispositions lines 51–54)
permits **zero** edit-supporting adapters in F12 and requires **per-response**
validation in F13: "a response with an invalid or stale edit is classified on
that response, and the rest of the merge continues (AC-5)."

### Proposed envelope (design proposal)

Replace the null slot with a **closed optional object** in the F13 edit-capable
family:

```text
proposed_edit: Option<ProposedEdit>   // null = no edit (today's behavior)

ProposedEdit {
  target_subject_sha256: String,  // MUST equal item.subject_sha256 AND response.subject_sha256
  patch_format: PatchFormat,      // closed enum; start with ONE format
  patch: String,                  // bounded UTF-8 bytes (propose <= 64 KiB)
  patch_sha256: String,           // digest of the exact patch bytes
}
```

- **Target-hash binding (AC-5).** If `target_subject_sha256` ≠ the item's
  `subject_sha256`, the edit targets a different/stale subject: classify **that
  response** and never apply; the rest of the merge continues. This reuses the
  existing `subject_sha256` binding rather than inventing a new hash channel.
- **Never apply during merge** (M-10). Merge validates syntax + hash only;
  promotion remains a separate, ordinary domain approval path (PRD 068 G-4).
- **Classification.** Wrong target hash → reuse `stale` (the subject is no longer
  current). Malformed patch syntax → add **one** closed classification variant
  `invalid-edit` to `ResponseClassification` (`wire.rs` lines 96–113). Both are
  per-response and never block other responses (D068 AC-5).
- **Versioning.** `()` → `Option<object>` is a breaking decode change. Follow the
  repo convention ("old /1 and /2 source bodies are untouched"): F13 ships
  edit-capable responses under a **new closed schema version**; F12 keeps
  `proposed_edit: ()` (zero adapters, per D068). Do not mutate the delivered
  `/1` `/2` `/3` bodies.
- **Bounds & safety.** Closed `patch_format` enum, bounded patch bytes, no
  execution, no network. The patch is inert untrusted data until a human promotes
  it through the normal domain path.

### Options and tradeoffs

- **A — Closed optional object + new family version (recommended).** Consistent
  with existing hash bindings and the untouched-old-bodies convention; AC-5 falls
  out of the existing `subject_sha256` check. *Tradeoff:* one new classification
  variant and a new schema version to qualify.
- **B — Reuse `/1` and make `proposed_edit` optional in place.** *Tradeoff:*
  breaks decoding of already-delivered null-edit responses and violates the
  repo's separate-typed-successor convention. Rejected.
- **C — Free-form patch string with no format enum.** *Tradeoff:* unbounded,
  unvalidatable syntax; M-10 requires validating patch syntax. Rejected.

### Recommendation

Adopt **Option A** as the F13 M-10 contract. F12 stays zero-edit. The first
`patch_format` should be the single format the first F13 adapter actually needs
(do not build a multi-format registry up front).

### What it unblocks

- F13 (#59) M-10 implementation + the AC-5 wrong-hash regression test.
- F12 (#21) M-10 checkbox closes as "zero adapters, per D068."

---

## 5. F12 M-13 sensitive export and M-15 Windows publication

**Plan §4 item 6; PRD 068 M-13 (line 138), M-15 (line 140); ticket #71 (id 1478).**

### M-13 — sensitive/excerpt export mode

**Exact question:** Should Forge add an explicit sensitive/excerpt export mode,
and under what gate?

**Current state.** [`wire.rs`](../../src/review/wire.rs) `Sensitivity` enum
(lines 72–75) has only `IdsAndHashes`; `ContextSnapshot` carries `reason_codes`
and `related_subject_ids`, **no excerpts**. D068 (dispositions lines 55–57):

> No sensitive export mode is required while Forge exports only IDs, hashes,
> reason codes and bounded context. A mode is added only when a real workflow
> needs excerpts or personal data, and it then requires explicit opt-in.

**Options.** (A) Keep the conservative default; record an opt-in contract for
F13 to implement only when a real workflow demands excerpts/PII (recommended).
(B) Build the mode now — rejected: D068 says it is not required, and no workflow
has demanded it (YAGNI). (C) Allow excerpts by default — rejected: violates
M-13's privacy default.

**Proposed opt-in contract (when needed).** A new closed `Sensitivity` variant
(e.g. `excerpts`), gated by an **explicit CLI flag + queue-level declaration**,
bounded excerpt bytes, labeled untrusted, never the default. Record the contract
now; defer implementation.

**Recommendation: A — explicit opt-in, conservative default retained, defer
implementation** until a real workflow needs it.

### M-15 — Windows publication

**Exact question:** Lift the Windows publication refusal, or keep it until
Windows qualification?

**State of record — the mechanism is already merged.** Ticket #71 delivered the
shared Windows no-replace path via **PR #254** (merge `d047040`, ancestor of
`main`). [`src/authoring/output.rs`](../../src/authoring/output.rs) `windows`
module (lines 630–631) is a **std-only hard-link no-replace publisher**; file
publication now supports Linux, macOS, and Windows (lines 148–156). Docs were
updated (`ad44d88`) so the review/POA&M/epoch guides link
[Windows file publication](../authoring.md#windows-file-publication) instead of stating a refusal.
**Directory publication remains Linux/macOS only** (`output.rs` lines 48–56:
"atomic no-replace authoring directory publication is unsupported on this
platform").

**Reconciliation.** The plan §4 item 6 / assignment phrase "currently refused" is
**stale for file publication** (the mechanism shipped) but **accurate for
directory publication and for qualification**. The no-replace *mechanism* is not
the same as Windows *qualification* (real-hardware run, plan §5 P7).

**Options.** (A) Keep M-15 acceptance **open** and refuse to claim
Windows-qualified until P7 hardware qualification passes; the mechanism exists
but qualification evidence is the gate (recommended). (B) Flip M-15 to done on
the merged mechanism alone — rejected: no Windows qualification evidence. (C)
Re-refuse file publication — rejected: it is delivered and documented.

**Recommendation: A — keep the refusal of *qualified* Windows publication until
P7 qualification passes.** #71 stays **In Review** (per coordinator). If any
review/POA&M/epoch output requires *directory* publication on Windows, that
remains refused; file publication is the delivered path.

### What it unblocks

- M-13: an opt-in contract F13 can implement without re-decision.
- M-15: a Windows qualification plan (P4 fixtures/scripts → P7 hardware), feeding
  #71 and F04 (#6).

---

## 6. F16 confinement design proposal

**Plan §4 item 8, §5 P5; PRD 066 S-1 (line 143) and line 260; new ticket under #2.**

This is a **design proposal, not implementation.** No code changes, no new
dependency without PRD 069 approval.

### Exact question

Which **per-platform OS mechanism** provides filesystem/network confinement,
verified-byte execution, and descendant cleanup for the local-model adapter, so
`ProcessModelAdapter::invoke` can stop refusing?

### Current state

[`src/suggest/adapter.rs`](../../src/suggest/adapter.rs) line 17
`PROCESS_EXECUTION_UNAVAILABLE`: "process adapter execution is disabled until OS
filesystem/network confinement, verified-byte execution and process-tree cleanup
are available; use `--recorded-response` …". `invoke` always refuses. PRD 066
line 260 records the same boundary. Partial mitigations already exist
([2026-09-12 suggestions plan](2026-09-12-066-suggestions-pipeline.md) lines
528–535): the adapter re-verifies executable bytes immediately before spawn,
reader threads are bounded, and on Unix a refusal kills the process group — but
the plan explicitly states the adapter is **not sandboxed** and implies no egress
guarantee.

### Per-platform mechanisms (named)

| Platform | Mechanism | Pros | Cons | New dependency? |
|---|---|---|---|---|
| **macOS** | `sandbox-exec` / Seatbelt (`sandbox_init` profile) | Built-in; deny-default file + network; profile inherited by children | `sandbox-exec` is deprecated/private surface; profile language undocumented; must verify on macOS hardware | **No** — spawn `/usr/bin/sandbox-exec` via `std::process::Command`, or `libc` FFI |
| **Linux** | **Landlock** (filesystem LSM, kernel ≥5.13, unprivileged) + **seccomp-BPF** (deny socket/network syscalls) | No root; path allowlist + network deny; ruleset/seccomp inherited across fork | Landlock is kernel-version dependent (refuse on old kernels); seccomp policy must cover `socketcall`/`io_uring` etc. | **No** — `libc` is already a unix dependency ([`Cargo.toml`](../../Cargo.toml) line 51) |
| **Windows** | **Job Objects** (`KILL_ON_JOB_CLOSE` for descendant cleanup) + **AppContainer** (low-privilege sandbox, network isolated unless a capability is granted) | Job Object kills the whole tree on handle close; AppContainer denies network by default | AppContainer/Job Objects need Win32 FFI; file-access ACL setup is complex; Job Objects alone do not confine filesystem/network | **Yes** — `windows-sys`/`winapi` → **PRD 069 approval required** |

### Descendant-cleanup strategy

- **Unix:** `setsid`/process group + `killpg` on refusal/timeout (already partly
  present); children inherit the same confinement (Landlock ruleset, seccomp
  filter, Seatbelt profile all inherit across fork/exec).
- **Windows:** assign the child to a Job Object with
  `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` **before resume**, so every descendant
  dies when the job handle closes.
- **Verified-byte execution:** re-hash the executable immediately before spawn
  (already done) and bind consent to `executable_sha256`
  (`adapter.rs` line 96); refuse if the path was replaced.
- **Network confinement:** deny by default (seccomp socket deny / Seatbelt
  network deny / AppContainer with no network capability). Preserves the
  offline-core invariant.

### Options and tradeoffs

- **A — Approve the design direction; implement Linux + macOS first (no new
  deps); keep Windows live execution refused until a `windows` crate is approved
  and qualified (recommended).** Recorded-response replay stays the
  cross-platform default. *Tradeoff:* Windows live-process generation is deferred
  for 2.0.0 unless the owner approves a dependency.
- **B — Implement all three platforms now, adding a `windows` crate.**
  *Tradeoff:* requires PRD 069 dependency approval + supply-chain review before
  any code; largest unknown (plan §7 risk). Not conservative.
- **C — Rely on the existing process-group kill + byte re-verification without
  OS confinement.** *Tradeoff:* provides **no** filesystem/network egress
  guarantee; PRD 066 line 260 and `adapter.rs` explicitly refuse execution until
  real confinement exists. Rejected.

### Recommendation

Approve **Option A** as the design direction: **Landlock + seccomp (Linux),
`sandbox-exec` (macOS), Job Objects + AppContainer (Windows, gated on dependency
approval)**. Implement Linux + macOS first with **no new crate**; keep Windows
live execution refused until a `windows` crate is approved and qualified.
Per-OS mechanism must be named (as above) before any implementation estimate is
credible (plan §7).

### What it unblocks

- F16 implementation ticket (create under #2 when P5 starts): filesystem/network
  confinement, verified bytes, limits/timeouts, descendant cleanup per platform.
- F17 (task contracts) and F18 (review fast-follows) downstream; recorded-response
  operation remains available throughout.

---

## 7. D068 quorum disposition verification

**Plan §4 item 3; ticket #21. Verify against merged code; do not re-decide.**

The D068 "Minimum contract" (dispositions lines 43–46) is the approved minimum:
Mapping first then Applicability; immutable snapshots + hash-bound source
references; unique declared reviewers, role counts, explicit substitutions,
separation of authors from reviewers by declared key, and abstentions that never
approve. Verification against [`src/review/quorum_v3.rs`](../../src/review/quorum_v3.rs)
and its decode path:

| D068 minimum | Merged evidence | Match |
|---|---|---|
| Unique declared reviewers | [`decode_v3.rs`](../../src/review/decode_v3.rs) `roster()` (lines 128–150) + `ordered()` (lines 68–84): strictly increasing keys, `previous >= value` → `Invalid` (duplicates refused) | ✓ |
| Role counts | `quorum_v3.rs` `seats()` (lines 36–59): `checked_add`, count `1..=100`, else `Capacity` | ✓ |
| Explicit substitutions | `quorum_v3.rs` `eligible()` (lines 62–90): checks `policy.substitutions` edges (`reviewer_key`, `asserted_role`, `seat_role`) | ✓ |
| Author/reviewer separation by declared key | `decode_v3.rs` `eligible()` line 233: `contains(&item.author_keys, reviewer)` → `Invalid`; `AuthorSeparation::DeclaredKeys` (`wire.rs` lines 88–91) | ✓ |
| Abstentions never approve | `quorum_v3.rs` `edges()`/`candidates()` require `Disposition::Approve` && `Classification::Current`; `AbstentionRule::Nonapproving` (`wire.rs` lines 80–82) | ✓ |
| One key owns at most one seat | `quorum_v3.rs` `Matching.witnesses: Vec<Option<usize>>` (lines 22–27), maximum matching | ✓ |
| Immutable snapshots + hash-bound refs | `ReviewItem.subject_sha256`/`context_sha256`/`policy_sha256`, `SourcePin.raw_sha256` (`wire.rs`) | ✓ |

**Finding: no mismatch.** `quorum_v3` implements the D068 minimum. The same
invariants hold in `quorum.rs` (`/1`) and `quorum_v2.rs` (`/2`) for their
families. The module header (`quorum_v3.rs` line 4) correctly states matching
"never authenticates keys, removes dissent or proves native currency" —
consistent with the asserted-identity boundary.

**Action:** close ticket #21's D068 checkbox **by this evidence**, not by
re-decision. No code change.

---

## 8. Reconciliation findings

These are documentation/record reconciliations surfaced while preparing the
packets. None re-decides a binding disposition.

- **F1 — Q7 "scope expansion" flag is an over-reading.** Plan §4 item 4 and the
  F10 audit framed AR-in-`forge validate` as beyond PRD 054. PRD 054 lines 72–73
  exclude AR *generation* and adding AP/SSP to `forge validate`, **not** AR
  *validation*; PR #253 added validation only and is merged (`5e79563`,
  2026-10-26; ticket #70 comment 1317 "No user decision is open"). Action:
  **confirm in scope, no amendment required**; optional clarifying note
  (§3, Q7 sub-item). The dispositions' "scope expansion" wording (lines 83–84)
  inherits the over-reading. AR generation and AP/SSP remain excluded.
- **F2 — D069 split recommendation contradicts the record.** Plan §4 item 2
  ("portable-file adapters → F12") conflicts with the binding D068 scope split
  (all remaining adapters → F13, dispositions lines 47–50). Action: keep the
  recorded split (§2).
- **F3 — D069 identifier collision.** The plan uses "D069" for the F12/F13 split;
  the [2026-10-02 packet](2026-10-02-roadmap-owner-decisions.md) uses **D069 for
  dependency policy (F02/F03)**. Action: relabel the plan item to avoid
  cross-referencing the wrong decision.
- **F4 — F11 approved proposal bytes are not in the repo.** The design approval
  binds `sha256 60c8dff1…` at `/private/tmp/forge-f11-discovery-implementation-review-v3.md`,
  which is no longer on disk. Action: re-pin the durable proposal/threat-model
  bytes into `docs/` before D067 ratification so the approval binds reviewable
  artifacts (§1).
- **F5 — M-15 "currently refused" is stale for file publication.** The Windows
  no-replace file publisher merged (PR #254, `d047040`); directory publication
  and Windows *qualification* remain open. Action: keep M-15 acceptance open
  pending P7 qualification; #71 stays In Review (§5).
- **F6 — Close-ready tickets.** #66 (dispositions record), #67 (D064 terminal
  states), #68 (S-4 epochs), #69 (S-3 add-only export) are **close-ready**: PRs
  #252 (`20f46ac`), #255 (`c16845d`), #256 (`a14f299`), #257 (`f21299b`) all
  merged via #259 (`b4226bb`). #253 (`5e79563`) and #254 (`d047040`) are direct
  ancestors of `main`. **#71 stays In Review** pending Windows qualification.
  Closing is a human/merge-hook action; this packet changes no status.

---

## Sources

- [Remaining-work completion plan](2026-11-12-remaining-work-completion-plan.md)
  (coordinator working document, local `11e1698`), §4 decision register, §5 P3/P5.
- [2026-10-25 owner dispositions](2026-10-25-owner-dispositions.md) — D064, D068,
  F10, cross-cutting (binding).
- [2026-10-02 roadmap owner decision packets](2026-10-02-roadmap-owner-decisions.md)
  — D067 proposal, D069 dependency policy.
- [F11 implementation-design approval v1](2026-10-04-f11-implementation-design-approval-v1.json);
  [F11 pre-PR verification v3](2026-10-04-f11-pre-pr-verification-v3.md).
- PRDs [054](../PRD/054-prd-oscal-1-2-3-compatibility.md),
  [063](../PRD/063-prd-oscal-assessment-results.md),
  [064](../PRD/064-prd-oscal-poam-workflow.md),
  [066](../PRD/066-prd-ai-assisted-suggestions.md),
  [067](../PRD/067-prd-read-only-mcp-governance-interface.md),
  [068](../PRD/068-prd-collaborative-review-queues.md).
- [MCP guide](../mcp.md); `src/mcp/` (mod, disclosure, wire, catalog, schemas);
  `src/review/` (wire, wire_v2, wire_v3, decode_v3, quorum_v3, adapters);
  `src/poam/` (risk_authoring, assessment_epochs); `src/suggest/adapter.rs`;
  `src/authoring/output.rs`.
- [2026-09-12 suggestions pipeline plan](2026-09-12-066-suggestions-pipeline.md).
- External: [MCP Inspector protocol eras](https://modelcontextprotocol.io/docs/2026-07-28/tools/inspector/protocol-eras);
  [Claude Code MCP client runtimes](https://code.claude.com/docs/en/mcp).
- veans tickets #2 (1390), #19 (1414, comment 1307), #21 (1416), #55 (1450),
  #56 (1451), #59 (1454), #70 (1477, comment 1317), #71 (1478).

# Roadmap owner decision packets

Date: 2026-10-02 (America/Los_Angeles). Baseline: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`.
Status: **Proposed; no owner disposition recorded by this packet.**

These packets make unresolved choices reviewable. Their contents are evidence and
proposals, never authorization. Previously recorded authoring dispositions and
local-only suggestion boundaries remain in force. Missing human measurements,
independent review, signatures and audit approval remain incomplete.

## D069 — Dependency policy (F02/F03)

Owner: Engineering/Product (Brian Luby). PRD 069 Open Questions and Definition of
Ready have no later disposition. Existing imports are Embark Studios and Mozilla;
configuration alone is not a new owner trust decision.

Recommended policy:

- Accept human source audits and agent-assisted reviews only after an explicit
  named human sign-off. Preserve reviewer, exact crate/source/version, criteria,
  method, evidence and signer. An agent review remains pending until signed off.
- Every new exception has a named owner, rationale, exact crate/source/version,
  creation date, review date and expiry; maximum 90 days before renewed review.
  Permanent exceptions are not accepted. Dates and owners are operator supplied.
- Retain only the existing Embark Studios and Mozilla import sources, subject to
  owner confirmation. Bind imported bytes to digests and recorded source commits,
  require explicit reviewed refresh, and require owner approval for new sources.
- Runtime code is safe-to-deploy; build/proc-macro/dev-only code is safe-to-run.
  Record the supported target and feature matrix and artifact hashes alongside
  the normalized graph. A dependency used in both classes takes the stronger one.
- Freeze legacy bare exemptions as a separately labeled, exact-version baseline
  that does not earn audit credit. Never seed invented owners or validity dates.
  New unvetted runtime versions and stale owned exceptions yield exit 1; malformed
  owned exceptions or inconsistent inputs yield exit 2. Legacy baseline prevents
  a flag day while the full runtime acceptance denominator stays open.

Alternatives: human-only source review (slower but simplest provenance); allow
agent-only audits (requires a different trust decision); permanent named
exceptions (does not meet proposed expiry policy); disable external imports and
review locally (larger workload). No alternative is silently selected.

Acceptance: approved policy recorded in PRD 069 or the introducing PR; offline
inventory distinguishes legacy/unvetted, owned exception, pending review and
accepted audit; differential audit requires a reviewed exact base, never an
exemption. Current Codex chacha20 delta is not proof of a reviewed base.

## D064 — POA&M editable authority and scaffold (F07/F08)

Owners: Product/Engineering/Compliance. PRD 064 has three blocking questions.
Recommended design: the closed local `forge.poam/1` manifest remains the editable
source of truth; OSCAL POA&M is deterministic generated output. Pin official
POA&M schema to the same OSCAL release as Assessment Results (currently 1.2.3),
with URL/release/checksum provenance. Implement every M-1–M-17 and S-1–S-4 through
F07/F08, using typed items, risks, findings, parties and milestones; scaffold
selects no remediation and assigns no owners or dates.

Retain the PRD's terminal states `completed-asserted` and
`accepted-risk-asserted`: require an explicit independent reviewer assertion,
role, time, rationale and fresh evidence-reference metadata; do not authenticate
authority or claim verified closure. A reopen creates an explicitly linked
superseding item rather than rewriting terminal history. The [official-schema/typed-model spike](2026-10-02-poam-contract-spike.md) verifies
that an empty native POA&M is invalid, so init emits only the unselected manifest.
Finding-only milestone storage, native date-time versus full-date mapping, and
assertion-status projection need explicit decisions; no timestamp is fabricated.
Exact evidence fields and official-schema subset need owner disposition before
freezing the public contract.

Alternatives: editable OSCAL as source of truth (requires round-trip mutation
contract); dual editable stores (requires explicit conflict reconciliation).
Acceptance: schema/model spike and closed contract are reviewable, real owner
status/evidence disposition exists, and independent-tool and remediation pilot
results stay open until observed.

## D067 — MCP discovery, client and evaluation boundary (F11/F20)

Owners: Product/Security/Engineering/Evaluation. Recommend a dedicated closed,
hash-bound project discovery manifest built from the workspace's typed resource
index conventions, rather than glob discovery or assuming forge.workspace/1 is
already the normative MCP manifest. Keep stdio, permanent read-only capabilities,
canonical-root containment and lifecycle/hash validation. Initial evaluation
jobs: exact requirement retrieval, control trace, and recorded applicability/gap
lookup; all seven PRD tools remain in scope.

Decide whether approved stdio clients require an allowlist. A process-only
boundary cannot authenticate arbitrary client identity. Prepare two-client
protocol fixtures and a separate human-adjudicated agent corpus/raw-document
baseline, with identical lawful documents, frozen tasks and model settings,
supported-answer/citation/abstention/disclosure/latency/token measures. Approve
visibility profile integrity and optional contained full-text index in F20;
conditional Shoulds stay in scope. No corpus labels or client acceptance invented.

## D068 — Portable review authority (F12/F13)

Owners: Product/Compliance/Engineering/Security. Recommend mapping first, then
applicability, with immutable minimal snapshots plus hash-bound local source
references. Declared reviewer/role keys stay asserted; unique declared reviewers,
role counts, allowed substitutions, author/reviewer separation and abstentions
are explicit policy fields. Conflicting responses remain preserved and
quorum-met never grants domain approval. Decide the minimum approved quorum and
separation semantics and portable snapshot boundary before freezing contracts.

Signed response envelopes remain in scope under S-2 despite the MVP exclusion
of signatures. Prepare a separate versioned envelope and identity/crypto design;
new crates require prior approval and audit. Later domain adapters, proposed
edits, response-file workspace client, supersession and notification export stay
in F13, without sending notifications.

## Other remaining owner and evidence gates

- PRD 059 M-11: current component publication uses coordinated staged replacement
  and rollback. Engineering must approve the stated transaction limit or require
  stronger publication; no crash-recovery or multi-file atomicity claim.
- PRD 060 S-3: approve supported model scope only after per-model lossless,
  schema-valid overlay round trips; linkage never proves evidence sufficiency.
- PRD 062: preserve accepted ADRs; independent security disposition, full browser
  and OS offline-runtime evidence, WCAG 2.2 AA keyboard/screen-reader exercises,
  five-user study and remaining Should features stay open.
- PRD 063 S-4: validate real multi-epoch workflow and stable source/baseline
  semantics before claiming conditional acceptance.
- PRD 065: provider/auth boundary, credential handling, Jira data residency and
  signed connector-package permission design need recorded owner decisions.
- PRD 066: corpus/adjudicator/threshold and first-generation-task decisions remain
  open. The [evaluation foundation packet](2026-10-02-suggestion-evaluation-design.md)
  specifies lawful inputs, authentic adjudication, frozen labels and incomplete
  outcomes. Local-only, quarantine and human disposition decisions are retained.
- F21: lawful partner workflows and actual measurements/judgments are required;
  preparing a packet cannot satisfy them. Outreach is not authorized.
- F22: current installed Rust 1.99 adds Clippy failures in the unchanged baseline.
  Keep this toolchain drift visible; Rust 1.98.1 compatibility validation does not
  establish current-stable CI readiness. Decide the supported compiler/tooling
  baseline and reconcile source lints before release.
- F22: release remains held for those gates; 2.0.0 candidate preparation is
  authorized, merge and publication require separate authorization.

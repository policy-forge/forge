# Forge roadmap completion tranches

Date: 2026-10-01 (America/Los_Angeles)

Status: Proposed execution plan. This document does not close requirements,
record approvals, or authorize a release.

## Goal and scope

Complete the remaining post-v1.1.0 roadmap, PRDs 055–069. The proposed finish
line includes their Must Have and Should Have features, implementation evidence,
interoperability and participant exercises, and the recorded acceptance and
release gates. Could Have and Won't Have items remain outside this finite scope.
Conditional Should Haves include their prerequisite design and workflow decisions;
they are not silently dropped from the completion plan.

The recorded public Rust API compatibility decision is **2.0.0**. “v1.x roadmap”
describes the initiative set; it does not override that release decision.
See [the authoring gate register](../authoring-gates.md).

## Verified baseline

- Veans identifies project **3, Forge**, Kanban view **12**. The project task
  endpoint returned `total: 0`; the five Kanban buckets also reported zero tasks.
  There are no existing ticket IDs to reconcile. No board records were changed.
- GitHub main was verified and local remote references refreshed to
  `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e` (PR #159). Differences from the earlier
  inspected main are confined to dependency files.
- The working checkout is the dependency branch at `704df721`. Its tracked code
  matches current main. The user's Veans setup changes are preserved.
- `/Users/bluby/repos/forge-product-planning` is clean on `codex/product-planning`
  at `52fce93`. That commit is an ancestor of current main; its original PRDs
  056–068 are already merged. The folder adds no separate unmerged feature scope.
- Five dependency PRs are open: #160 rayon, #161 clap, #162 argon2, #163
  pdf-extract, and #164 getrandom. They need individual review and current audit
  dispositions; their existence does not establish readiness to merge.
- This was a board, source and document review. Tests, browser acceptance,
  participant exercises and release checks were not executed for this plan.

## What is actually unfinished

| Area | Baseline | Remaining work |
|---|---|---|
| Original WI-1–WI-50 | Recorded complete | Keep this historical scope closed. |
| 055–058 | Must/Should implementations recorded complete | Reconcile actual product, pilot and release gates. |
| 059 components | All four Should features have implementation/test evidence despite unchecked boxes | Requirement/evidence reconciliation and residual human gates. |
| 060 evidence linkage | Maintenance queue, impact/lifecycle references and HTML trace exist | Lossless OSCAL back-matter overlay, including per-model proof and approval. |
| 061 authoring | Both phases and Should features have technical evidence | Measured partner workflows and release acceptance. |
| 062 workspace | API, embedded UI, shared services, safe effect workflow and local browser harness are merged | Browser CI, full platform/browser and runtime-offline evidence, security/accessibility acceptance, user studies, and remaining Should features. |
| 063 assessment results | One result epoch, scaffolding and HTML reporting exist | Selected reviewed-risk export into POA&M and multiple epochs after workflow validation. |
| 064 POA&M | No command family implemented | Full remediation workflow, including baseline comparison and Should features. |
| 065 integrations | No command family implemented | Export/plan/apply/reconcile, reference and additional connectors, and signed package boundary. |
| 066 suggestions | Retrieval and recorded-response pipeline exist | Corpora/thresholds, confined local execution, complete mapping task semantics, qualified generation and review fast follows. |
| 067 MCP | No command family implemented | Bounded read-only shared queries, stdio protocol, clients/evaluation and Should features. |
| 068 review queues | No portable review command family implemented | Immutable packages, responses/dispositions, quorum/conflicts, domain adapters, local client and signed responses. |
| 069 dependency security | Existing vet/audit/deny jobs and some audit records | Offline inventory, classification, owned exceptions, dedicated gate, runtime reviews and release cadence. |

Unchecked PRD boxes are not sufficient evidence of missing code. In particular,
do not create duplicate implementation work for 059 or 061. The workspace's
existing local review queue is not the portable collaboration feature in 068.

## First tranche: establish a trustworthy base and start remediation

These are proposed work-package identifiers, **not Veans ticket numbers**.
Split a package into child tasks when its contract, implementation and acceptance
work need separate owners or reviewable PRs.

| Package | Concrete deliverable | Acceptance boundary | Dependencies |
|---|---|---|---|
| F01 — Reconcile requirements and decisions | Current Must/Should evidence matrix for 055–069; corrected roadmap status; named remaining gates; decision packets for audit policy, POA&M semantics, MCP discovery and review identity | Every item points to implementation evidence or an explicit remaining task; prior owner decisions retained; no approval inferred from code or tests | Current source and PRDs |
| F02 — Dependency inventory and gate | Offline deterministic inventory, runtime/build classification, validated owned exception records, stale reporting and CI gate | New unvetted runtime version fails; malformed exception fails; stale exception requires review; same inputs give identical bytes; audit/exception policy has an owner disposition | F01 policy decisions; no new runtime dependencies |
| F03 — Review the shipping dependency stack | Exact-version audits of workspace transport/session/crypto transitives, trusted import decisions, differential workflow and dependency PR review | Each crate in the frozen runtime set has attributable review evidence; no bare runtime exemption; current advisory and CI evidence retained | F02; freeze actual supported feature/platform graph |
| F04 — Put workspace verification in CI | Reuse and pin the existing browser harness; publish API/browser/parity artifacts; execute supported platform/browser and packaged-runtime offline checks | Hosted evidence covers the agreed matrix; browser request blocking is not treated as proof of full OS network denial | F03 for dependency acceptance; development-tool approval if a new dependency is needed |
| F05 — Close workspace security and accessibility defects | Scoped independent review, adversarial cases, keyboard/screen-reader evaluation, targeted fixes and current acceptance register | No undispositioned launch blocker; evidence covers WCAG 2.2 AA golden path and stated filesystem/transaction limits; actual owner dispositions recorded | F04; can prepare review cases in parallel |
| F06 — Build suggestion evaluation foundation | Versioned human-adjudicated task and injection corpora, frozen labels, bounded evaluator and task-selection packet | Approved per-task quality/safety thresholds have authentic evidence; missing judgments remain incomplete; first generation task selected only after corpus work | F01; lawful inputs and real adjudicators |
| F07 — Build the POA&M foundation | Release-matched official schema, bounded manifest, source binding, explicit item selection, stable item/milestone IDs and deterministic scaffold | Invalid/stale source fails without output; scaffold asserts no selected remediation; official schema and identity fixtures pass | F01 schema/status/source-of-truth decisions; existing 063 identity contracts |

Prepare pilot materials alongside this tranche: lawful inputs, pseudonymous
participant keys, repeatable tasks, before/after measurements and evidence forms.
Preparing a packet does not satisfy participant or owner acceptance.

Suggested execution lanes:

1. **Dependency lane:** F02 → F03; review open dependency PRs as part of the exact
   version workflow rather than merging them wholesale.
2. **Workspace lane:** F04 → F05, reusing the current API/browser harness.
3. **Domain lane:** F07 → F08 below; prepare F06 evaluation artifacts while
   human adjudication is pending.

One coordinating owner handles shared CLI/error/schema/manifest contracts and
integration. Keep large Cargo jobs sequential. Scope implementation PRs so their
acceptance evidence can be reviewed independently.

## Subsequent tranches: finish the feature set

### Domain completion and read-only access

| Package | Scope and finish line | Prerequisites |
|---|---|---|
| F08 — Full POA&M | Ownership, milestone cycles/dates, append-only status, closure assertions, explicit-as-of schedule, baseline impacts, validated deterministic output, portfolio/HTML/evidence links and connector-neutral export. Cover every 064 Must, including M-13 baseline, and S-1–S-4. Independent-tool validation and remediation pilots are acceptance work completed with F21. | F07, fresh 060 references, three representative sanitized remediation plans |
| F09 — Lossless evidence overlay | 060 S-3 back-matter overlay with approved supported-model scope and schema-valid lossless round trips. Preserve unrelated model data; do not treat linkage as evidence sufficiency. | Per-model design and interoperability proof |
| F10 — Assessment extensions | 063 S-3 selected reviewed-risk export to the 064 scaffold; S-4 multiple result epochs with stable identity, source and baseline semantics. Export creates no automatic owners/dates. | F08 source contracts; validated epoch workflow |
| F11 — Read-only MCP core | Approved project discovery and threat model; shared typed queries; stdio resources/tools; exact-ID trace, deterministic search, applicability/gaps, lifecycle visibility, hash freshness, bounds/privacy/cancellation. Complete every 067 Must, two-client interoperability, and a separate adjudicated agent corpus compared with a raw-document baseline for supported-answer accuracy, citations, abstention, disclosure, latency and token size. | F01 discovery decisions; dependency approval/audit for any new crate |

### Collaboration and external handoff

| Package | Scope and finish line | Prerequisites |
|---|---|---|
| F12 — Portable review core | First 055/056 adapters; closed queue/response/disposition schemas; immutable hash-bound snapshots; response/merge/status/HTML; assignment, quorum, staleness, conflict and separation rules. Conflicting responses remain preserved. | Approved first workflow and asserted identity/quorum boundary |
| F13 — Complete collaborative review | Authoring plan/clause, lifecycle/impact/AI/assessment/POA&M adapters; proposed edits; workspace response-file client; notification export; supersession; signed response envelopes after identity/crypto design. Complete 068 Musts and Shoulds; no implicit domain approval. | F08/F10/F12, existing 061 and suggestion source contracts; signature design and dependency approval |
| F14 — Integration core and GitHub | Connector-neutral change sets, source adapters, fake-server suite; exact dry-run plan/apply/reconcile; allowlisted GitHub Issues; approved credential handling, preconditions, idempotency, conflicts and resumable receipts. Remote observations never change Forge authority. | Stable source reports; provider/auth decision; runtime audit boundary |
| F15 — Remaining integration features | GitLab and Jira adapters, non-authoritative remote comments/status, signed versioned connector packages with explicit permissions. Use the same contract suite; complete 065 S-1–S-4. | F14; Jira auth/data-residency and package-signing design decisions |

### Qualified suggestions and workspace completeness

| Package | Scope and finish line | Prerequisites |
|---|---|---|
| F16 — Confined local model execution | Filesystem/network confinement, verified executable bytes, limits/timeouts and ownership/cleanup of all descendants across supported platforms. Retain recorded-response operation. | F06 gates; approved confinement design; reviewed dependencies |
| F17 — Complete both suggestion tasks | Mapping context/preparation/destination/promotion semantics plus policy drafting; qualified local execution, citations/assumptions, quarantine and proposed unapproved patches. Maintain both versioned task contracts and regression gates. | F06/F16; relevant 055/061 destination contracts |
| F18 — Suggestion review fast follows | Evidence/impact-based batch ordering; source/candidate/diff workspace review; expired-input re-evaluation without transferring approval. Complete 066 S-2–S-4; S-1 is F16. | F17 and workspace APIs |
| F19 — Workspace Should features | Verify existing coverage, then finish static redacted exports, guided authoring, lifecycle/impact views, bounded long-operation progress/cancel/restart, signed platform launch integration and explicit manifest/hash bundle import/export. Complete 062 S-1–S-6. | Existing 057/058/061 services; F04/F05; platform packaging decisions |
| F20 — MCP Should features | Evidence metadata trace, static report links, owner-bound visibility profiles and a deterministic contained optional full-text index. Complete 067 S-1–S-4; evidence bytes remain excluded. | F11/F19 and existing 060 linkage metadata; profile integrity design |

## Dated F13/F20 candidate checkpoint: 2026-10-05

The [Lifecycle `/2` documentation proposal](../lifecycle-review-exchange.md)
describes a separate re-review adapter whose implementation candidate registers
`forge review lifecycle init/respond/merge/status`. One authentic coupled run
passed 63 controls: 11 receiver, 7 binding, 9 finalizer, 7 current-command,
8 response, 12 export and 9 init/CLI controls. This is one cohort, not cumulative
credit for earlier overlapping runs. Later exact formatted candidate checks
passed normal regressions, strict lint and fresh library-only LLVM instrumentation,
plus a separate macOS child workflow; the linked guide records the denominators.
Other platform and acceptance gates remain open. The other
F13 adapters, proposed edits, workspace client and signature design remain open;
existing F12 notification/supersession work does not complete those requirements.

The separate [Authoring `/3` candidate](../authoring-review-exchange.md) covers one
complete current saved `forge.authoring-plan/1` and the
`forge review authoring init/respond/merge/status` routes. Closed TEMP qualification
includes 4,248 passing full-suite controls with three unchanged ignored controls,
strict lint, a lexical Rustdoc census, physical library LLVM coverage and a genuine
compiled CLI campaign. The linked guide preserves exact populations and limits.
Final integrated checks and hosted/platform/human acceptance remain open. This
adapter grants no native promotion and does not complete individual-clause or
component-plan review, other F13 adapters, proposed edits, client or signature
gates. The historical requirement and Ready checkboxes remain unchanged.

The [MCP `/2` checkpoint](../mcp.md#f20-2-development-checkpoint) records an
offline native-input consumer and a later App candidate. The latest same-owner
reuse run passed 25 selected native/controller controls and failed two native
controls; standalone report preparation and the distinct-manifest case still
reach Capacity. Both failed-run diagnostic histories remain retained. No
accepted App capability or completed S-1–S-4 claim follows. Managed F11/server
coupling, evidence metadata/report resources, the actual deterministic index
builder, real owner records, clients and corpus evaluation remain open.

This is a development checkpoint, not a change to package finish lines or owner
dispositions. Apply and re-review its documentation against the eventual
integrated source; final docstring/coverage checks and the user's goal-wide docs
review remain required before the next draft PR.

## Acceptance and release closeout

**F21 — Execute real combined workflows.** Run the authoring protocol over at
least five policy workflows; record time to a reviewable skeleton, revision
burden and retained sections. Complete the workspace's five-user study and
manual accessibility exercises. Run independent-tool OSCAL interoperability,
two MCP clients and the separate MCP agent corpus/raw-document baseline,
review-conflict exercises, connector sandbox retries and the adjudicated
suggestion evaluation. Reconcile remaining 055–060/063 gates using
the requirements ledger. Record results and actual owner dispositions; do not
substitute synthetic participants or test counts for these gates.

**F22 — Prepare the complete release candidate.** Refresh exact-head CI,
advisory/audit and security evidence; finish migration/help/examples and
versioned contract documentation; verify platform packages and consumer
installation; include dependency inventory/trends in release artifacts.
Complete 069 cadence, bump reminders, duplication reduction and remaining
runtime audit work. Cross-check the audited set against the vendored schema
manifest and release provenance/checksum claims (069 M-11). Confirm the
runtime-reviewed denominator, owned exception
ages and all remaining gate owners before requesting release approval.

The existing release remains held until its recorded gates pass. This plan
prepares a reviewable candidate; publication and participant outreach require
their own authorization.

## Completion rules

- A feature is technically delivered only after its scoped changes are reviewed,
  tested appropriately and merged, with the requirement evidence updated.
- PRD completion additionally requires its participant, interoperability and
  owner acceptance gates. A documented gate is not a passed gate.
- New crates require approval recorded in the PRD or introducing PR and a
  supply-chain entry. Use existing alternatives first.
- Keep the 066 boundary local-only. No provider egress, autonomous approval,
  hidden authoritative mutation or invented usefulness claim is introduced.
- When tickets are created, use real child tasks for independent work, explicit
  dependencies and HTML descriptions. Read back each tracker mutation. Agent
  delivery moves work to In Review; humans or the merge hook close it.

## Primary repository references

- [Canonical roadmap](../FORGE_PRODUCT_ROADMAP.md)
- [Authoring gate register](../authoring-gates.md)
- [Workspace verification and limitations](../local-workspace.md)
- [Workspace requirement and verification contract](../PRD/062-prd-local-web-workspace.md)
- [Evidence linkage implementation audit](../PRD/060-implementation-audit.md)
- [Assessment Results](../PRD/063-prd-oscal-assessment-results.md)
- [POA&M](../PRD/064-prd-oscal-poam-workflow.md)
- [Integrations](../PRD/065-prd-external-workflow-integrations.md)
- [Suggestions, including later local-only owner decisions](../PRD/066-prd-ai-assisted-suggestions.md)
- [Read-only MCP](../PRD/067-prd-read-only-mcp-governance-interface.md)
- [Collaborative reviews](../PRD/068-prd-collaborative-review-queues.md)
- [Dependency security](../PRD/069-prd-dependency-security-audit.md)

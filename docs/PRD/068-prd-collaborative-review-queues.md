# 068-prd-collaborative-review-queues

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `068-collaborative-review-queues`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will package mapping, applicability, change-impact, authoring, assessment, and remediation review items into deterministic local review queues. Reviewers exchange append-only response files through an existing shared filesystem or source-control workflow; FORGE validates assignments, subject hashes, dispositions, conflicts, and queue completion without authenticating identities or operating a hosted collaboration service. Live multi-user collaboration is deferred until the asynchronous contract proves useful.

## Context

### Background :red_circle: `@human-required`

FORGE's evidence-first features deliberately require human review, but each currently assumes one manifest author and has no common mechanism for assigning work, collecting independent decisions, resolving disagreement, or proving that the reviewed subject bytes match the current project. Building live collaboration first would introduce accounts, authorization, tenancy, and databases before the review semantics are stable.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | PRDs 055–066 all create human-required decisions or review queues with stable IDs/hashes. | A common envelope can coordinate review without changing domain-specific authority. |
| Product boundary | Local manifests preserve declared parties but do not authenticate identity. | Response files must state this limitation and must not be called digital signatures. |
| Product hypothesis | Portable asynchronous queues solve most initial collaboration pain without a hosted platform. | Measure completed review cycles and conflict resolution before building live service mode. |

No team workflow research, identity design, merge corpus, or hosted-service business case was supplied. Queue adoption targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Versioned `forge.review-queue/1` package containing immutable review-item snapshots
- Item sources from PRDs 055–058 and 061, 063, 064, and 066
- Declared reviewers, roles, assignments, due dates, quorum/separation policy, and sensitivity labels
- Append-only `forge.review-response/1` files with approve, reject, request-changes, abstain, or superseded dispositions
- Exact subject hashes, reviewer rationale, explicit time, proposed edits where supported, and source finding IDs
- Deterministic merge, conflict, stale-response, quorum, and completion reports
- Git-friendly file layout and static HTML queue report

**Out of Scope:**

- Authentication, authorization, e-signatures, SSO, accounts, organization tenancy, or non-repudiation
- Hosted database/service, real-time editing, presence, comments chat, or notifications
- Automatic approval, tie-breaking, conflict resolution, or domain-artifact mutation
- Git operations, pull-request creation, email, or ticketing performed by FORGE
- Evidence content or unrestricted policy/framework excerpts in queue packages

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/055-prd-control-mapping.md` | Mapping-review items |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Applicability-review items |
| `docs/PRD/057-prd-framework-change-impact-monitoring.md` | Change-impact findings |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Declared approval policies |
| `docs/PRD/061-prd-framework-guided-policy-authoring.md` | Draft questions/clauses |
| `docs/PRD/062-prd-local-web-workspace.md` | Optional local visual review client |
| `docs/PRD/066-prd-ai-assisted-suggestions.md` | Suggestion dispositions |

---

## Problem Statement :red_circle: `@human-required`

Compliance teams need multiple people to review mappings, exclusions, policy clauses, findings, and remediation decisions, but ad hoc spreadsheets and comments lose the exact reviewed bytes and decision policy. Without a common asynchronous review contract, FORGE's mandatory human gates become difficult to coordinate and may be satisfied by stale or conflicting responses.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Bind review to exact subjects. | Every accepted response references an item ID, subject type, and exact subject/input hashes. |
| G-2 | Coordinate multi-reviewer policy. | 100% of seeded quorum, role, separation, duplicate, stale, and conflict cases produce the expected deterministic state. |
| G-3 | Preserve independent decisions. | Merge never rewrites a response; disagreements and proposed edits remain visible. |
| G-4 | Keep domain authority separate. | Queue completion produces a validated disposition bundle but never mutates or approves the underlying domain artifact automatically. |
| G-5 | Reduce review coordination time. | Five teams complete a 50-item two-reviewer queue 30% faster than their current process. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- Declared reviewer identity is not authenticated or legally signed.
- Queue completion does not itself approve a policy, mapping, finding, or POA&M item.
- The MVP does not choose between conflicting reviewer decisions.
- FORGE does not transmit packages or operate source control.
- Review counts are not a quality or compliance score.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Assign review work (P0)

> As a compliance lead, I want stable items assigned by role and due date so that required decisions have visible owners.

### US-2 — Review exact snapshots (P0)

> As a reviewer, I want each item to show exact source identity, bounded context, and required decision so that my response cannot attach to changed content silently.

### US-3 — Merge responses safely (P0)

> As a compliance engineer, I want deterministic quorum and conflict reports so that multiple response files never become last-write-wins.

### US-4 — Preserve disagreement (P0)

> As an auditor, I want all independent dispositions and rationales retained so that dissent and requested changes remain part of the record.

### US-5 — Review visually (P1)

> As a reviewer, I want a local accessible queue view so that I can work without editing JSON directly.

## Review Model :yellow_circle: `@human-review`

Each item is an immutable snapshot with stable item key/UUID, domain type, action requested, priority reason, subject hashes, bounded context, allowed disposition vocabulary, and required policy. Queue generation never copies full sensitive source by default.

Responses are separate append-only files named by queue ID, item ID, and declared reviewer key. A response applies only when the queue/item/subject hashes match. `request-changes` may carry a domain-specific proposed patch, but the patch remains untrusted and un-applied.

Completion states are `unassigned`, `assigned`, `in-review`, `conflicted`, `changes-requested`, `quorum-met`, `expired`, or `stale`. `quorum-met` means the declared review policy is satisfied, not that the underlying artifact is approved.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge review init`, `respond`, `merge`, `status`, and `export-html` with explicit queue/response paths.
- [ ] **M-2 — Closed schemas:** Validate bounded `forge.review-queue/1`, `forge.review-response/1`, and `forge.review-dispositions/1` schemas.
- [ ] **M-3 — Domain adapters:** Generate immutable items from approved versioned outputs without redefining their domain semantics.
- [ ] **M-4 — Snapshot integrity:** Record source artifact IDs/hashes, subject ID/hash, context hash, and queue schema/version for every item.
- [ ] **M-5 — Assignment policy:** Support declared reviewers/roles, required counts, allowed substitutions, separation constraints, due date, and optional abstention rules.
- [ ] **M-6 — Response evidence:** Require exact item hashes, reviewer key/role assertion, disposition, explicit time, and non-empty rationale except documented abstention.
- [ ] **M-7 — Staleness:** Reject or classify stale responses when any bound subject/context/policy hash changes; never transfer a disposition automatically.
- [ ] **M-8 — Append-only merge:** Preserve every unique response byte-for-byte or by recorded hash; reject duplicate-key ambiguity and never last-write-wins.
- [ ] **M-9 — Conflict/quorum:** Compute deterministic per-item state from the declared policy and all valid responses; expose every conflicting disposition.
- [ ] **M-10 — Proposed edits:** Validate proposed patch syntax and target hash where supported but never apply it during merge.
- [ ] **M-11 — Disposition bundle:** Emit a deterministic bundle suitable for a separate domain-specific promotion step; label `quorum-met` as review-policy status only.
- [ ] **M-12 — Identity disclaimer:** State in CLI/report/package that reviewer keys and roles are asserted, not authenticated or signed.
- [ ] **M-13 — Privacy:** Default to IDs, hashes, reason codes, and bounded context; require explicit sensitive export for excerpts/PII.
- [ ] **M-14 — Determinism:** Sort items/responses/states predictably and require explicit `--as-of` for due/expired classification.
- [ ] **M-15 — Safe I/O:** Operate offline with project containment, regular-file validation, resource bounds, alias rejection, and atomic outputs.
- [ ] **M-16 — Static HTML:** Emit a self-contained, inert, accessible, redacted review report with no mutation or remote assets.
- [ ] **M-17 — Tests:** Cover domain adapters, stale hashes, quorum/separation, conflicts, duplicate responses, proposed patches, privacy, determinism, and HTML injection.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** PRD 062 interactive local client that writes the same response files.
- [ ] **S-2:** Signed review response envelope after identity/cryptography design.
- [ ] **S-3:** Connector-neutral queue notification export for PRD 065 without sending it.
- [ ] **S-4:** Queue supersession linking old/new item IDs after upstream change impact.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Hosted authenticated review service with RBAC, tenancy, audit logs, and data residency.
- [ ] **C-2:** Real-time comments/presence and conflict-aware editing.
- [ ] **C-3:** Organization directory/IdP integration and verified signatures.

### Won't Have (W) — This release :red_circle: `@human-required`

- Accounts, auth, signatures, hosted service, real-time editing, notifications, Git/ticket operations, auto-approval, or automatic conflict resolution.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | A two-reviewer quorum and two matching approvals | Merge runs | State is `quorum-met` and no domain artifact changes |
| AC-2 | One approval and one rejection | Merge runs | State is `conflicted` with both responses preserved |
| AC-3 | Subject bytes change after queue generation | Old response is merged | It is stale and does not satisfy quorum |
| AC-4 | The same reviewer response appears twice | Merge runs | It is deduplicated by exact identity or rejected if bytes conflict |
| AC-5 | A proposed edit targets a different subject hash | Merge runs | The patch is invalid/stale and never applied |
| AC-6 | Context contains script markup | HTML export runs | Content is inert and no remote asset loads |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | State correctness | 100% seeded quorum/conflict/stale cases | Contract fixtures |
| Leading | Review task completion | 4 of 5 teams | Moderated pilot |
| Leading | Silent disposition transfer | Zero | Change-impact tests |
| Lagging | Coordination time | 30% median reduction | Partner comparison |
| Lagging | Repeat review | Three teams complete two queue revisions | Opt-in evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** At least one stable domain source schema and shared stable identity/hash conventions.
- **Phase 1:** PRD 055/056 queue adapters, response/merge/status, static HTML.
- **Phase 2:** Lifecycle/change-impact/AI/assessment/POA&M adapters and proposed patches.
- **Phase 3:** Signed envelopes investigation and hosted-collaboration discovery.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Reviewer key mistaken for verified identity | False assurance | Persistent declared-identity disclaimer; signatures separate |
| Queue duplicates domain approval logic | Conflicting truth | Adapter preserves domain semantics; promotion remains domain-specific |
| Stale approvals carry forward | Governance failure | Exact subject/context hashes and no automatic transfer |
| Git merge conflicts make packages painful | Adoption failure | One response per item/reviewer and deterministic merge command |
| Sensitive context spreads in review packages | Confidentiality harm | Minimal snapshots and explicit sensitive export |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Which first review job—mapping, applicability, or AI suggestions—has the strongest multi-reviewer need?
- **[Compliance, blocking]** Which quorum and separation policies must the generic schema support without becoming a workflow engine?
- **[Engineering, blocking]** Should queue packages contain snapshots or references plus a mandatory local artifact bundle?
- **[Security, non-blocking]** Should signed response envelopes become a separate PRD before any hosted collaboration?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product selects one initial domain adapter and review workflow.
- [ ] Compliance approves generic disposition/quorum semantics and authority boundary.
- [ ] Engineering approves immutable snapshot, response, merge, and promotion contracts.
- [ ] Security/privacy reviews package contents and identity disclaimer.
- [ ] Three teams provide representative multi-reviewer workflows.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Start with portable asynchronous queues | Validates review semantics without SaaS/auth complexity | Hosted real-time collaboration |
| 2026-08-24 | Keep review completion separate from domain approval | Generic quorum cannot replace feature-specific governance rules | Queue auto-approval |
| 2026-08-24 | Bind every response to subject hashes | Prevents stale decisions from silently carrying forward | Item ID only |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for deterministic asynchronous collaborative review queues |

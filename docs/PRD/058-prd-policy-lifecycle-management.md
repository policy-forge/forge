# 058-prd-policy-lifecycle-management

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `058-policy-lifecycle-management`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will manage a local, reviewable lifecycle record for policy sources and their generated OSCAL artifacts: ownership, version, state, approvals, review dates, supersession, and retirement. The MVP is a deterministic CLI workflow and append-only transition history; it preserves declared accountability but does not authenticate identities, provide electronic signatures, send notifications, or replace Git and document-management systems.

## Context

### Background :red_circle: `@human-required`

FORGE validates policy-derived artifacts but does not currently say whether a policy is a draft, approved, overdue for review, superseded, or retired. Teams therefore keep lifecycle metadata in filenames, document headers, ticketing systems, and spreadsheets that can drift away from the exact source and OSCAL bytes used for mappings or assessments.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | FORGE already fingerprints inputs, creates deterministic identifiers, embeds metadata, and detects artifact drift. | Lifecycle records can bind approvals to exact source and generated-artifact bytes. |
| Product boundary | FORGE is local, offline, CLI-first, and does not authenticate reviewer identities. | The MVP should preserve asserted roles and actions without claiming identity assurance. |
| Product hypothesis | Owners and review queues increase continued use after initial conversion. | Measure completed review cycles and overdue reduction, not record count. |

No design-partner lifecycle corpus, approval-policy research, or production review-cycle metrics were supplied. Workflow and retention targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- A bounded `forge.policy-lifecycle/1` local JSON record per policy
- Stable policy key, title, version, owner roles, source artifact, and generated-artifact fingerprints
- States `draft`, `in-review`, `approved`, `superseded`, and `retired`
- Documented allowed transitions with actor, role, time, rationale, and exact input/output hashes
- Configurable declared separation-of-duties rules
- Review cadence, next-review date, expiry/overdue calculation, and deterministic review queues
- Supersession links between stable policy identities or versions
- Read-only validation, transition proposal/application, status, and text/JSON report commands
- Drift detection when approved bytes no longer match current source or generated artifacts

**Out of Scope:**

- Identity proof, login, RBAC enforcement, digital signatures, PKI, or legal e-signature
- Collaborative document editing, comments, notifications, calendars, or ticket creation
- Storage of policy source content in a database
- Git commit, branch, pull-request, or release manipulation
- Automatic approval, automatic ownership assignment, or automatic lifecycle transitions
- Records-retention law interpretation or organization-specific approval-policy design

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/013-prd-catalog-pipeline.md` | Generated Catalog pipeline |
| `docs/PRD/016-prd-traceability-model.md` | Source-to-artifact provenance |
| `docs/PRD/043-prd-diff-report.md` | Artifact drift reporting |
| `docs/PRD/052-prd-github-action-drift-enforcement.md` | CI gate conventions |
| `docs/PRD/053-prd-stable-id-migration.md` | Policy identity evolution |
| `docs/PRD/057-prd-framework-change-impact-monitoring.md` | External changes that may trigger re-review |
| `docs/PRD/059-prd-reusable-policy-components.md` | Component dependencies within policy versions |

---

## Problem Statement :red_circle: `@human-required`

Policy owners and auditors need to know which exact policy version was approved, who declared the approval, when it must be reviewed, and what replaced it. Without a deterministic lifecycle record tied to content fingerprints, an apparently approved document can be edited, regenerated, or superseded without an actionable integrity warning.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Bind lifecycle state to exact artifacts. | Every approved record fingerprints the source and each declared generated artifact. |
| G-2 | Make transitions explicit and auditable. | 100% of accepted state changes preserve prior state, actor assertion, role, time, rationale, and deterministic event ID. |
| G-3 | Prevent invalid workflow states. | 100% of seeded invalid transitions, missing approvals, conflicting versions, and broken supersession links fail safely. |
| G-4 | Surface review work deterministically. | Status reports classify overdue, due-soon, drifted, superseded, and current policies reproducibly. |
| G-5 | Improve maintenance behavior. | Three design partners complete two policy review cycles with at least 30% fewer overdue policies. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- FORGE preserves declared approvals but does not prove who performed them.
- Approval does not establish legal validity, implementation, effectiveness, or compliance.
- Lifecycle records do not replace source control, document repositories, or retention systems.
- The MVP does not send reminders or mutate external systems.
- The MVP does not define one universal approval workflow for all organizations.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Approve exact policy bytes (P0)

> As a policy owner, I want approval tied to the exact source and generated artifacts so that later edits cannot retain a misleading approved state.

### US-2 — Enforce valid transitions (P0)

> As a compliance lead, I want lifecycle rules validated before state changes so that draft, review, approval, supersession, and retirement histories remain coherent.

### US-3 — Find overdue and drifted policies (P0)

> As a compliance engineer, I want a deterministic review queue so that maintenance work is visible and prioritizable.

### US-4 — Trace supersession (P0)

> As an auditor, I want to know which approved version replaced another and why so that historical evidence remains interpretable.

### US-5 — Apply organization-specific role separation (P1)

> As a compliance lead, I want declared author/reviewer/approver separation rules so that FORGE can reject workflow records that violate our stated process.

### US-6 — Consume lifecycle status in CI (P1)

> As a DevSecOps engineer, I want stable JSON and exit statuses so that unapproved drift or expired review can block publication.

## Lifecycle Model :yellow_circle: `@human-review`

### States and Transitions

| State | Meaning | Allowed Next States |
|-------|---------|---------------------|
| `draft` | Content is being authored and is not approved. | `in-review`, `retired` |
| `in-review` | A declared review is active against exact fingerprints. | `draft`, `approved`, `retired` |
| `approved` | Required declared approvals exist for exact fingerprints. | `in-review`, `superseded`, `retired` |
| `superseded` | Another identified policy/version replaces this one. | `retired` |
| `retired` | Policy is no longer active and cannot return to an active state. | None |

Changing approved source or generated-artifact bytes does not silently change the recorded historical state. Current status becomes `approved-drifted`, a derived blocking condition requiring a new transition through `in-review`.

### Identity and Versioning

- `policy-key` identifies the logical policy across revisions.
- `version-key` identifies one immutable reviewed version.
- Event IDs are UUID v5 values derived from schema version, policy key, version key, sequence, transition, actor key, time, and approved fingerprints.
- Sequence numbers must be contiguous; event order is explicit and never inferred from array order alone.
- Supersession must identify the replacement policy/version and cannot form cycles.

### Declared Roles

Actors are local manifest parties with stable keys and roles such as author, reviewer, approver, owner, and custodian. FORGE validates references and configured separation rules but prominently states that it does not authenticate identity or authority.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge lifecycle init`, `check`, `transition`, and `status` with text/JSON output and safe file options.
- [ ] **M-2 — Closed schema:** Parse bounded `forge.policy-lifecycle/1` JSON and reject unknown keys, duplicate decoded keys, unsupported versions, gaps in sequence, and exceeded limits.
- [ ] **M-3 — Artifact identity:** Fingerprint the policy source plus explicitly listed generated OSCAL artifacts and validate their type/root metadata where supported.
- [ ] **M-4 — State machine:** Enforce only documented state transitions and make `retired` terminal.
- [ ] **M-5 — Transition evidence:** Require actor, declared role, timestamp, rationale, previous state, next state, and exact relevant fingerprints for every event.
- [ ] **M-6 — Approval policy:** Require a versioned local approval policy defining required declared roles/counts; reject approval when requirements are unmet.
- [ ] **M-7 — Separation rules:** Validate optional author/reviewer/approver key separation while stating that identity is not authenticated.
- [ ] **M-8 — Drift:** Derive `approved-drifted` when current bytes differ from approved fingerprints; never preserve clean approved status through content change.
- [ ] **M-9 — Review schedule:** Validate cadence, next-review date, timezone policy, and derived `due-soon`/`overdue` status from an explicit `--as-of` date.
- [ ] **M-10 — Reproducible time:** Require explicit event times and report `--as-of`; do not use wall-clock time in deterministic JSON fixtures or event identity.
- [ ] **M-11 — Supersession:** Validate replacement references, chronological consistency, no self-reference, and no cycles within a supplied portfolio.
- [ ] **M-12 — Append-only history:** Preserve all accepted events; transition application may append one event but never rewrite or delete prior events.
- [ ] **M-13 — Safe mutation:** Validate the complete proposed record, write atomically, reject aliases/symlinks per existing safe-I/O policy, and leave the original intact on failure.
- [ ] **M-14 — Status report:** Emit stable states, derived conditions, owners, due dates, blockers, current hashes, approved hashes, and event IDs without policy prose by default.
- [ ] **M-15 — Exit contract:** Exit `0` for valid policy under the selected gate, `1` for valid lifecycle action required, and `2` for invalid input or transition.
- [ ] **M-16 — Tests:** Cover every transition, approval/separation rule, drift case, date boundary, supersession cycle, determinism, and safe-write failure.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Portfolio status command over explicitly supplied lifecycle files.
- [ ] **S-2:** Machine-readable review queue grouped by owner and due date.
- [ ] **S-3:** Link PRD 057 impact finding IDs as reasons for re-entering review.
- [ ] **S-4:** Emit unsigned approval attestations suitable for external signing without implementing signatures.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Cryptographic signatures and identity-provider verification under a dedicated threat model.
- [ ] **C-2:** Web-based review and approval backed by the same transition contract.
- [ ] **C-3:** External ticketing/calendar notifications through opt-in connectors.

### Won't Have (W) — This release :red_circle: `@human-required`

- Authentication, authorization, e-signature claims, remote storage, collaborative editing, notification delivery, or Git mutation.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | A draft with required reviewer and approver events against identical hashes | Approval transition runs | State becomes `approved` and all evidence is retained |
| AC-2 | An approved policy source changes by one byte | Status runs | Derived status is `approved-drifted`, exit is `1` under the default publication gate |
| AC-3 | An author attempts approval where separation is required | Transition runs | Exit is `2` and the original lifecycle file is byte-unchanged |
| AC-4 | A retired policy is transitioned to draft | Transition runs | The terminal-state violation is rejected |
| AC-5 | Two policies supersede each other | Portfolio check runs | The cycle is reported as invalid |
| AC-6 | The same record and `--as-of` date run twice | Reports are compared | Status, ordering, and JSON bytes match |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Valid workflow completion | 4 of 5 partners complete draft-to-approved without maintainer edits | Moderated pilot |
| Leading | Drift detection | 100% seeded post-approval changes detected | Automated fixtures |
| Leading | Transition integrity | 100% invalid state/separation scenarios rejected | Contract tests |
| Lagging | Overdue reduction | 30% reduction after two review cycles | Partner lifecycle snapshot |
| Lagging | Repeat use | Three organizations maintain at least five policies through two revisions | Opt-in partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** Existing hashing, safe writes, metadata parsing, traceability, and deterministic report conventions.
- **Phase 1:** Single-policy state machine, approvals, drift, and review dates.
- **Phase 2:** Portfolio status, supersession graphs, review queues, and PRD 057 finding links.
- **Phase 3:** Design-partner workflow validation and external unsigned-attestation export.
- **Does not block:** PRD 056 or PRD 059; their artifacts may be linked after their contracts stabilize.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Declared actor is mistaken for authenticated signer | False audit assurance | Persistent warnings, no signature terminology, explicit trust boundary |
| Lifecycle file duplicates Git history | Low adoption | Focus on approval/review semantics Git does not encode and bind to Git-managed bytes |
| Organization workflows differ | Overfitted state machine | Minimal common states plus versioned approval policy; no arbitrary workflow engine in MVP |
| Wall-clock behavior breaks reproducibility | Inconsistent CI | Explicit event time and `--as-of` date |
| Mutating history destroys evidence | Audit failure | Append-only validation and atomic writes |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Is `superseded` meaningfully distinct from `retired` for the first design partners?
- **[Compliance, blocking]** What minimum declared approval evidence should the default policy require?
- **[Engineering, blocking]** Should lifecycle records be one file per logical policy or one portfolio file with transactional updates?
- **[Security, non-blocking]** Which signature formats should a future attestation export anticipate without claiming support now?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves states, derived conditions, and default review queue.
- [ ] Compliance approves the declared-identity disclaimer and reference approval policy.
- [ ] Engineering approves append-only event identity, mutation safety, time handling, and limits.
- [ ] Security reviews future-signature boundaries and path/content privacy.
- [ ] Three design partners provide representative approval flows.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Bind approval to exact source and generated bytes | State without artifact identity can survive unauthorized or accidental edits | Version string only |
| 2026-08-24 | Use a small fixed state machine | Covers the common lifecycle without building a workflow platform | Arbitrary user-defined states |
| 2026-08-24 | Preserve declared accountability without identity claims | Local CLI cannot authenticate people or authority | Implicit trusted usernames |
| 2026-08-24 | Require explicit time input for deterministic status | Reproducible results matter in CI and audits | Hidden wall-clock time |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for deterministic policy lifecycle records and review queues |

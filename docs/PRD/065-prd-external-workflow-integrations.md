# 065-prd-external-workflow-integrations

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `065-external-workflow-integrations`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will export deterministic connector-neutral change sets and ship one reference GitHub Issues adapter for explicitly approved creation/update of review and remediation work. Dry-run is mandatory before apply, remote identity is recorded, retries are idempotent, secrets never enter manifests, and reconciliation never treats external ticket state as FORGE approval or assessment truth. GitLab, Jira, and GRC adapters remain follow-on implementations of the same contract.

## Context

### Background :red_circle: `@human-required`

FORGE findings, gaps, lifecycle reviews, framework impacts, and POA&M milestones become valuable when they enter operational work systems. Hand-copying loses stable IDs and provenance; naive synchronization can duplicate tickets, leak sensitive policy data, or let a remote status silently rewrite governed artifacts.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | FORGE already produces stable IDs, JSON reports, and predictable status classifications. | Integrations should consume versioned outputs rather than scrape human text. |
| Product principle | Core FORGE is local, standards-native, and vendor-neutral. | Network adapters remain optional boundaries around a connector-neutral change-set schema. |
| Product hypothesis | Reliable operational handoff increases repeat use more than another export format. | Measure reconciled work and duplication/errors, not API call volume. |

No customer integration ranking, API spike, credential-handling design, or production sync corpus was supplied. GitHub-first and adoption targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Versioned `forge.integration-change-set/1` generated from PRDs 056–058 and 063–064 outputs
- Connector-neutral create/update/close-request operations with stable idempotency keys
- One reference GitHub Issues adapter using explicit repository allowlisting
- Read-only remote planning and diff followed by separate explicit apply
- Local state mapping FORGE object IDs to remote object IDs/versions
- Retry, rate-limit, partial-failure, conflict, and reconciliation reporting
- Redacted payload preview and field-level sensitivity controls

**Out of Scope:**

- Automatic apply, hidden background sync, webhook listener, or bidirectional authority
- GitLab, Jira, ServiceNow, or GRC adapters in the MVP
- Remote systems changing FORGE approval, finding, risk, lifecycle, or POA&M truth
- Credential storage, OAuth application hosting, SSO, or secret-manager implementation
- Uploading policy/framework/evidence content by default

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/052-prd-github-action-drift-enforcement.md` | GitHub/CI conventions |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Gap work items |
| `docs/PRD/057-prd-framework-change-impact-monitoring.md` | Framework review findings |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Policy review items |
| `docs/PRD/063-prd-oscal-assessment-results.md` | Assessment findings/risks |
| `docs/PRD/064-prd-oscal-poam-workflow.md` | Remediation milestones |

---

## Problem Statement :red_circle: `@human-required`

Compliance and remediation teams must manually transfer FORGE action items into the tools where work is assigned. Without a stable, reviewed integration contract, transfers become stale or duplicated and remote ticket state can be mistaken for an authoritative change to policy, assessment, or remediation records.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Preserve identity through handoff. | Every applied remote object records one stable FORGE source ID and local mapping state. |
| G-2 | Prevent unintended mutation. | 100% of applies require a current dry-run plan, explicit target allowlist, and confirmation. |
| G-3 | Make retries safe. | Repeating an accepted change set creates zero duplicate remote objects. |
| G-4 | Expose conflicts and partial failure. | Every operation has an independent result and unresolved conflicts never become silent last-write-wins. |
| G-5 | Reduce manual re-keying. | Five pilots transfer 25 work items with 80% less manual field entry. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- External ticket status does not approve or mutate FORGE source artifacts.
- The MVP is not a general automation/plugin platform.
- FORGE does not store credentials in project files or outputs.
- The adapter does not upload source prose, evidence, or licensed framework content by default.
- Supporting one GitHub adapter does not establish parity with all vendors.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Preview operational handoff (P0)

> As a compliance engineer, I want to see exactly which issues and fields would change so that I can prevent accidental disclosure or mutation.

### US-2 — Apply idempotently (P0)

> As a remediation lead, I want retries to update the intended issue rather than create duplicates so that operational state remains usable.

### US-3 — Detect remote conflicts (P0)

> As an auditor, I want local and remote versions reconciled without silent overwrites so that the handoff history remains trustworthy.

### US-4 — Keep authority separated (P0)

> As a policy owner, I want remote completion reported as external state only so that tickets cannot silently close governed findings or lifecycle reviews.

### US-5 — Add another connector (P1)

> As an integrator, I want a documented contract fixture so that a Jira or GRC adapter can implement equivalent safety semantics.

## Integration Model :yellow_circle: `@human-review`

The change set contains stable operation keys, source artifact IDs/hashes, desired remote operation, allowed fields, redacted preview, and preconditions. A plan records remote object ID/version and payload hash. Apply requires that exact unexpired plan and rejects remote-version drift.

The local connector state file maps source/operation keys to remote provider, tenant/repository label, object ID, remote version, last applied payload hash, and time. It contains no tokens. Credentials come from an approved external provider mechanism and are never printed.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge integration export`, `plan`, `apply`, and `reconcile` with explicit connector/config/state files.
- [ ] **M-2 — Change-set schema:** Validate bounded `forge.integration-change-set/1` with stable operation/source keys and closed fields.
- [ ] **M-3 — Source validation:** Verify source artifact type, schema version, root ID, and hash before planning or applying operations.
- [ ] **M-4 — Reference adapter:** Support GitHub Issues create/update/close-request against explicitly allowlisted owner/repository targets.
- [ ] **M-5 — Credential boundary:** Use an approved external credential provider; never accept secrets in manifests, flags visible in process listings, output, logs, or state.
- [ ] **M-6 — Dry-run gate:** Apply only an exact, unexpired plan whose remote preconditions and payload hash still match.
- [ ] **M-7 — Explicit confirmation:** Require connector, target, counts, sensitive-field summary, and irreversible effects to be confirmed before mutation.
- [ ] **M-8 — Idempotency:** Derive stable idempotency keys and reconcile before create so repeated apply does not duplicate objects.
- [ ] **M-9 — Conflict handling:** Reject last-write-wins when remote version/content changed; emit a bounded field-level conflict report.
- [ ] **M-10 — Partial failure:** Record per-operation success/failure, preserve resumability, and never mark an unattempted/failed operation applied.
- [ ] **M-11 — Authority boundary:** Reconcile remote state as observations only; never mutate FORGE source status, approval, risk, or closure.
- [ ] **M-12 — Data minimization:** Default payloads to IDs, titles, reason codes, owners, dates, and repository-relative links; require opt-in for excerpts.
- [ ] **M-13 — Network safety:** Enforce fixed provider hosts, TLS verification, redirect restrictions, request/response bounds, timeouts, rate limits, and no arbitrary URLs.
- [ ] **M-14 — Audit report:** Emit redacted deterministic local reports for plan/apply/reconcile without tokens or sensitive response bodies.
- [ ] **M-15 — Tests:** Use a fake server plus contract fixtures for retries, conflicts, rate limits, partial failures, secret redaction, target allowlists, and duplicate prevention.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** GitLab Issues adapter using the same contract tests.
- [ ] **S-2:** Jira Cloud adapter after auth/data-residency review.
- [ ] **S-3:** Signed connector packages with explicit permissions and version pinning.
- [ ] **S-4:** Import remote comments/status as non-authoritative review observations.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** GRC adapters selected from validated design-partner demand.
- [ ] **C-2:** Webhook-driven reconciliation service with authenticated delivery and replay protection.
- [ ] **C-3:** Hosted connector broker under a separate tenancy/privacy architecture.

### Won't Have (W) — This release :red_circle: `@human-required`

- Background or bidirectional sync, arbitrary endpoints, credential storage, webhook listener, remote authority, or multiple production connectors.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Twenty-five valid source work items | Plan runs | Exact target, operations, payloads, and sensitive fields are previewed without mutation |
| AC-2 | The same accepted plan is applied twice | Second apply runs | No duplicate issue is created |
| AC-3 | A remote issue changes after plan | Apply runs | The operation conflicts and is not overwritten |
| AC-4 | Operation 10 fails after nine successes | Apply ends | Nine successes and one failure are recorded; remaining work is resumable |
| AC-5 | A payload contains policy prose | Default export runs | Prose is excluded unless explicit sensitive opt-in is configured |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Duplicate rate on retry | 0% | Fake-server and pilot runs |
| Leading | Conflict/data-leak prevention | 100% seeded cases | Security/contract tests |
| Leading | Handoff completion | 4 of 5 pilots | Moderated task |
| Lagging | Manual field-entry reduction | 80% | Partner comparison |
| Lagging | Reconciled reuse | Three teams complete two handoff/reconcile cycles | Opt-in evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** At least one stable source report contract; GitHub API/auth security design.
- **Phase 1:** Connector-neutral export and fake-server contract suite.
- **Phase 2:** GitHub plan/apply/reconcile with pilot repository allowlists.
- **Phase 3:** Evaluate GitLab/Jira/GRC demand and publish adapter SDK only after contract stability.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Duplicate or wrong-target tickets | Operational harm | Target allowlists, stable idempotency, dry-run, confirmation |
| Sensitive data leaves repository | Confidentiality/legal harm | Minimal fields, redacted preview, explicit excerpt opt-in |
| Remote status gains false authority | Governance failure | One-way authority boundary and observation-only reconcile |
| Provider API drift | Broken integration | Contract fixtures, versioned adapter, fail-closed unknown fields |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Is GitHub Issues the correct reference adapter for design partners?
- **[Security, blocking]** Which credential provider and minimum token scopes are acceptable?
- **[Engineering, blocking]** Should adapters be in-tree modules or separately signed executables?
- **[Legal/privacy, blocking]** Which metadata fields may cross organizational boundaries by default?

## Definition of Ready :red_circle: `@human-required`

- [ ] One source report schema is stable and versioned.
- [ ] Product confirms the reference connector and exact use case.
- [ ] Security approves credentials, hosts, redirects, scopes, and redaction.
- [ ] Engineering approves adapter/state/idempotency contracts.
- [ ] Three design partners authorize sandbox targets.
- [ ] Every Must Have maps to an executable contract/security test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Ship a neutral change set plus one adapter | Proves safety and value without premature breadth | Simultaneous GitHub/GitLab/Jira/GRC |
| 2026-08-24 | Require plan before apply | External mutations need reviewable intent and preconditions | Direct create/update |
| 2026-08-24 | Keep remote state non-authoritative | Tickets cannot approve governed FORGE artifacts | Bidirectional status sync |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for safe connector-neutral workflow integrations |

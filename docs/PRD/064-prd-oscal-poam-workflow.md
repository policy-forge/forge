# 064-prd-oscal-poam-workflow

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `064-oscal-poam-workflow`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will convert explicitly selected reviewed risks and findings into a schema-valid OSCAL Plan of Action and Milestones with accountable owners, milestones, target dates, status history, and exact source provenance. The MVP validates remediation plans and change history; it does not invent remediation, assign owners, calculate risk acceptance, close findings, create tickets, or attest that milestones were completed.

## Context

### Background :red_circle: `@human-required`

Assessment findings become operational only when teams assign remediation outcomes, owners, milestones, and dates. Those decisions commonly move into spreadsheets or tickets that lose their connection to the exact assessment result and affected controls. PRD 063 supplies reviewed results; a separate POA&M workflow preserves remediation accountability without conflating detection with resolution.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Roadmap evidence | POA&M is explicitly deferred pending remediation data and milestone tracking. | The feature needs a human-owned status/history contract, not simple serialization. |
| Product boundary | FORGE cannot verify work completion or risk acceptance authority. | Status changes and closures require explicit responsible-party and reviewer assertions. |
| Product hypothesis | Traceable POA&M output reduces handoff loss between assessors and remediation owners. | Measure completed, correctly linked maintenance cycles. |

No remediation-team research, ticket-system corpus, or independent POA&M interoperability result was supplied. Targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Release-pinned official OSCAL POA&M JSON schema and typed model
- One local PRD 063 Assessment Results artifact plus required context companions
- Explicit selection of findings/risks for remediation planning
- Reviewer-authored remediation items, owners, milestones, target dates, status, rationale, and history
- `planned`, `in-progress`, `blocked`, `completed-asserted`, `accepted-risk-asserted`, and `cancelled` workflow states
- Deterministic overdue/due-soon reporting using explicit `--as-of`
- Exact source/result/control provenance, baseline comparison, and schema validation

**Out of Scope:**

- Automatic remediation recommendations, prioritization, owner assignment, or due dates
- Verification of completion or risk-acceptance authority
- Ticket creation/synchronization; PRD 065 owns external integrations
- Notifications, calendars, collaboration, evidence collection, or continuous monitoring
- XML/YAML before JSON interoperability is proven

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/053-prd-stable-id-migration.md` | Stable identity evolution |
| `docs/PRD/054-prd-oscal-1-2-3-compatibility.md` | Schema provenance |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Append-only transition concepts |
| `docs/PRD/060-prd-evidence-implementation-linking.md` | Evidence references after remediation |
| `docs/PRD/063-prd-oscal-assessment-results.md` | Authoritative findings/risks input |
| `docs/PRD/065-prd-external-workflow-integrations.md` | Future ticket synchronization |

---

## Problem Statement :red_circle: `@human-required`

Remediation owners need a durable plan connecting assessment risks and findings to accountable work, dates, and status history. Without standards-native identity and provenance, spreadsheet and ticket records drift from the assessment baseline and can imply closure without preserving who asserted it or what changed.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Produce interoperable POA&M artifacts. | 100% of accepted output validates against the pinned schema and parses in one independent tool. |
| G-2 | Preserve remediation accountability. | Every item, milestone, status change, completion, and risk-acceptance assertion records responsible parties, time, and rationale. |
| G-3 | Prevent source drift. | 100% of seeded stale/missing result references and conflicting status histories fail before output. |
| G-4 | Surface schedule risk reproducibly. | Identical inputs and `--as-of` produce byte-identical overdue/due-soon reports. |
| G-5 | Improve remediation handoff. | Four of five pilots create an actionable 20-item plan without re-keying assessment references. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- FORGE does not decide which risks require remediation or acceptance.
- `completed-asserted` does not prove remediation effectiveness or finding closure.
- `accepted-risk-asserted` does not prove the actor had authority.
- FORGE does not create or update external tickets in the MVP.
- The workflow does not replace the source Assessment Results history.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Create remediation items from reviewed results (P0)

> As a remediation lead, I want selected findings and risks carried into a POA&M without re-keying identity so that work remains tied to assessment evidence.

### US-2 — Assign accountable milestones (P0)

> As a control owner, I want explicit owners, outcomes, milestones, and dates so that remediation work is actionable.

### US-3 — Preserve status assertions (P0)

> As an auditor, I want every status change and closure assertion attributed so that the history cannot silently rewrite prior commitments.

### US-4 — Find overdue work (P0)

> As a compliance engineer, I want a deterministic schedule report so that overdue and blocked milestones are visible.

### US-5 — Compare plan revisions (P1)

> As a remediation lead, I want stable change impact so that moved dates, changed owners, removed work, and status transitions are reviewable.

## Workflow Model :yellow_circle: `@human-review`

Status is an assertion with append-only history. `completed-asserted` and `accepted-risk-asserted` are terminal for the current item version but may be superseded by a new explicitly linked item if reassessment reopens work. Milestone dates use full-date semantics; reports require explicit `--as-of` and never use hidden wall-clock time.

Risk/findings absent from the POA&M remain absent; FORGE does not infer `no action required`. A scaffold may list eligible source objects but creates no remediation decision.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge poam init --assessment-results <FILE>` and `forge poam build/check --manifest <FILE>` with baseline/report options.
- [ ] **M-2 — Standards baseline:** Vendor the official release-matched POA&M JSON schema with source URL, release, and checksum.
- [ ] **M-3 — Closed manifest:** Parse bounded `forge.poam/1` JSON and reject unknown/duplicate keys, unsupported versions, and exceeded limits.
- [ ] **M-4 — Source validation:** Validate Assessment Results and required companions by type, UUID, version, OSCAL version, and hash.
- [ ] **M-5 — Explicit selection:** Require every remediation item to name one or more valid source findings/risks; scaffold creates no selected items.
- [ ] **M-6 — Stable item identity:** Require immutable item/milestone keys and deterministic UUID v5 IDs independent of order and mutable prose.
- [ ] **M-7 — Ownership:** Require responsible-party references and rationale for each item; validate roles without authenticating authority.
- [ ] **M-8 — Milestones:** Validate ordered milestones, target dates, dependencies within the item, outcomes, and no dependency cycles.
- [ ] **M-9 — Status history:** Enforce allowed transitions and append-only events with actor, role, explicit time, rationale, and prior/next state.
- [ ] **M-10 — Closure boundary:** Require explicit reviewer evidence for completion/risk acceptance and label both as assertions, not verification.
- [ ] **M-11 — Schedule report:** Derive overdue/due-soon/blocked conditions from manifest data and explicit `--as-of`; preserve all denominators.
- [ ] **M-12 — Typed/schema output:** Construct through typed Rust models and validate completed JSON against the official schema.
- [ ] **M-13 — Baseline:** Report owner/date/outcome/status/rationale/source changes, stale references, additions, removals, and reopened work by stable identity.
- [ ] **M-14 — Non-mutation:** Never change Assessment Results, close findings, or post to external systems.
- [ ] **M-15 — Safety/privacy:** Operate offline, bound resources, use safe writes, omit sensitive excerpts/absolute paths, and escape terminal content.
- [ ] **M-16 — Exit contract:** Exit `0` for valid ungated state, `1` for valid schedule/review actions, and `2` for invalid input/history.
- [ ] **M-17 — Tests:** Cover references, every transition, dates, cycles, closures, baseline impact, determinism, schema validity, and safe I/O.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Portfolio report across explicitly supplied POA&M artifacts.
- [ ] **S-2:** Static HTML timeline and source-to-remediation trace view.
- [ ] **S-3:** Connector-neutral outbound change set for PRD 065 without performing external mutations.
- [ ] **S-4:** Link fresh PRD 060 evidence references to completion assertions without verifying them.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Bidirectional ticket synchronization through approved connectors.
- [ ] **C-2:** Cryptographically signed risk-acceptance assertions.
- [ ] **C-3:** Reassessment workflow that explicitly supersedes completed items.

### Won't Have (W) — This release :red_circle: `@human-required`

- Automated remediation, risk acceptance, verified closure, ticket mutation, notifications, or continuous monitoring.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Valid reviewed risk and human remediation plan | Build runs | Schema-valid deterministic POA&M is emitted with exact source provenance |
| AC-2 | A source finding is absent from the supplied result | Build runs | Exit is `2` and no artifact is written |
| AC-3 | A milestone dependency cycle exists | Build runs | The cycle is rejected with stable manifest paths |
| AC-4 | An owner asserts completion without required reviewer evidence | Build runs | The terminal transition is rejected |
| AC-5 | A target date is before explicit `--as-of` | Check runs | It appears overdue without wall-clock dependence |
| AC-6 | Identical inputs build twice | Outputs are compared | JSON/report bytes match |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Valid references/status histories | 100% | Automated fixtures |
| Leading | Plan task completion | 4 of 5 pilots | Moderated study |
| Leading | Unverified closure language | Zero | Golden terminology tests |
| Lagging | Handoff re-keying | 80% fewer manually copied identifiers | Partner comparison |
| Lagging | Maintained plans | Three partners complete two plan revisions | Opt-in evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** PRD 063 Assessment Results and PRD 054 schema provenance.
- **Phase 1:** Scaffold, item/milestone/status model, JSON output, and schedule report.
- **Phase 2:** Baseline impact, evidence references, static HTML, connector-neutral change sets.
- **Phase 3:** Independent-tool interoperability and remediation-team pilots.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Assertion is mistaken for verified closure | False assurance | Explicit state names, reviewer evidence, and report disclaimer |
| POA&M duplicates ticket systems | Low adoption | Standards-native source of truth plus connector-neutral handoff |
| Dates/statuses change without history | Audit failure | Append-only transitions and baseline impact |
| Sensitive findings leak in reports | Confidentiality harm | IDs/hashes by default and bounded opt-in detail |

## Open Questions :yellow_circle: `@human-review`

- **[Compliance, blocking]** Which terminal states and evidence are sufficient to record an assertion without implying verification?
- **[Engineering, blocking]** What POA&M schema subset supports a complete useful MVP?
- **[Product, blocking]** Is the OSCAL artifact or the local manifest the editable source of truth after first build?
- **[Legal, non-blocking]** What language distinguishes declared risk acceptance from authorized acceptance?

## Definition of Ready :red_circle: `@human-required`

- [ ] PRD 063's stable result identity is approved.
- [ ] Official POA&M schema provenance and typed-model spike are complete.
- [ ] Compliance approves status and closure semantics.
- [ ] Engineering approves manifest, transitions, and supported schema subset.
- [ ] Three remediation teams supply representative sanitized plans.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Keep POA&M separate from Assessment Results | Remediation ownership and assessment judgment have different actors and lifecycles | Combined assessment/remediation command |
| 2026-08-24 | Label completion and acceptance as assertions | FORGE cannot verify work or authority | Plain `completed`/`accepted` states |
| 2026-08-24 | Require explicit `--as-of` | Schedule reports must be reproducible | Hidden wall-clock time |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for human-owned OSCAL POA&M planning and status history |

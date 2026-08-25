# 056-prd-framework-applicability-gap-analysis

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `056-framework-applicability-gap-analysis`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will let a compliance engineer declare which controls in a supplied OSCAL Catalog or resolved Profile are applicable, not applicable, deferred, or still under review, then combine those decisions with human-reviewed PRD 055 Mapping Collections to produce a deterministic policy-gap report. The MVP reports review state and mapping participation; it does not infer applicability, generate framework mappings, assess implementation, or declare compliance.

## Context

### Background :red_circle: `@human-required`

FORGE can convert policies to OSCAL and publish reviewed policy-to-framework mappings. A mapping alone does not establish which framework controls apply to a particular organization, and an unmapped control may be irrelevant, deferred, or simply not reviewed. Teams currently reconcile these distinctions in spreadsheets, making scope decisions difficult to reproduce and easy to detach from the exact framework version.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | PRD 055 emits validated Mapping Collections, participation reports, resource fingerprints, and explicit gaps. | Applicability analysis should consume those artifacts rather than create a second mapping model. |
| Product principle | FORGE prioritizes correctness, traceability, determinism, and local operation. | Every applicability decision needs stable identity, rationale, reviewer attribution, and exact framework provenance. |
| Standards boundary | Mapping participation does not establish implementation, effectiveness, or compliance. | Reports must use `mapped` and `unmapped`, never `compliant` and `non-compliant`. |
| Product hypothesis | A reproducible applicability and policy-gap workflow is more valuable than a framework-wide blank policy generator. | Validate with observed design-partner tasks before adding authoring automation. |

No completed user interviews, paid-pilot results, or production applicability corpus were supplied. Time-savings and adoption targets in this PRD are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- One user-supplied OSCAL Catalog or Profile as the framework baseline
- A required resolved Catalog companion when the baseline is a Profile
- A versioned local applicability manifest containing explicit control decisions
- `applicable`, `not-applicable`, `deferred`, and `under-review` decision states
- Mandatory rationale and reviewer attribution for `not-applicable` and `deferred`
- Zero or more schema-valid PRD 055 Mapping Collections whose target is the exact framework resource
- Deterministic control inventory, decision validation, mapped/unmapped classification, and text/JSON reports
- Baseline fingerprints, stale-reference detection, duplicate/conflicting-decision rejection, and predictable exit statuses
- A scaffold command that inventories controls without deciding applicability

**Out of Scope:**

- Automatic applicability decisions or regulatory/legal advice
- Automatic policy-to-framework mapping or relationship approval
- Compliance, implementation, effectiveness, maturity, or audit-readiness scores
- Bundled or remotely downloaded framework content
- Policy text generation, policy lifecycle workflow, evidence collection, or remediation tracking
- Hosted services, databases, web UI, authentication, notifications, or multi-user collaboration

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/FORGE_PRODUCT_VISION.md` | Product personas and correctness/traceability principles |
| `docs/PRD/030-prd-profile-generation.md` | Framework tailoring through Profiles |
| `docs/PRD/036-prd-oscal-cli-profile-resolution.md` | Resolved Profile companion workflow |
| `docs/PRD/053-prd-stable-id-migration.md` | Explicit identity transitions without guessed successors |
| `docs/PRD/055-prd-control-mapping.md` | Human-reviewed mappings and mapping participation |
| `docs/PRD/057-prd-framework-change-impact-monitoring.md` | Downstream monitoring of framework revisions |

---

## Problem Statement :red_circle: `@human-required`

Compliance engineers need to define the controls in scope for their organization and identify which applicable controls have reviewed policy relationships. Today, scope decisions and mapping gaps are commonly mixed in spreadsheets, so teams cannot reliably distinguish a true policy gap from an exclusion, deferral, or unfinished review when the framework changes.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Make framework scope explicit and reviewable. | Every non-default decision records a stable control ID, state, rationale, reviewer, review time, and framework fingerprint. |
| G-2 | Produce truthful policy-gap classifications. | 100% of seeded applicable mapped, reviewed-with-no-relationship, unmapped, excluded, deferred, and unreviewed controls are classified correctly without compliance language. |
| G-3 | Reject stale or conflicting scope data. | All seeded missing IDs, duplicate decisions, resource mismatches, and conflicting mappings fail before a report is written. |
| G-4 | Make repeated analysis deterministic. | Identical inputs produce byte-identical JSON reports across supported platforms. |
| G-5 | Reduce initial gap-analysis effort. | In five design-partner trials, median time to produce a reviewed 100-control scope and gap report is 40% below the partner's current workflow. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- FORGE does not decide whether a control legally or contractually applies.
- A mapped applicable control is not labeled implemented, effective, satisfied, or compliant.
- An unmapped applicable control does not prove that no relevant policy language exists.
- The MVP does not create or rewrite policy documents.
- The MVP does not redistribute proprietary framework content.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Declare framework applicability (P0)

> As a compliance engineer, I want to record reviewed applicability decisions against an exact framework version so that organizational scope is reproducible.

### US-2 — Identify applicable policy gaps (P0)

> As a compliance engineer, I want to see applicable controls with and without reviewed policy mappings so that I can prioritize policy review and authoring.

### US-3 — Preserve exclusion and deferral rationale (P0)

> As an auditor, I want every exclusion and deferral attributed and explained so that the gap report does not silently hide controls.

### US-4 — Reject stale decisions and mappings (P0)

> As an auditor, I want FORGE to reject decisions or mappings tied to another framework version so that the report does not combine incompatible evidence.

### US-5 — Scaffold review without making claims (P1)

> As a compliance engineer, I want a complete control inventory scaffold so that I can begin review without FORGE preselecting answers.

### US-6 — Enforce analysis in CI (P1)

> As a DevSecOps engineer, I want versioned JSON and stable exit statuses so that unresolved or stale scope can require human review in a pull request.

## Product Guardrails :red_circle: `@human-required`

1. **Applicability is a human decision.** FORGE validates and reports decisions but never supplies them.
2. **Mapping is not compliance.** `applicable-mapped` means at least one explicit reviewed relationship participates; it says nothing about implementation or effectiveness.
3. **Absence remains ambiguous.** `applicable-unmapped` is a review queue item, not proof that the policy library lacks relevant language.
4. **Exclusions are visible.** `not-applicable` and `deferred` remain in totals and always carry rationale.
5. **Exact resource identity matters.** The framework and every consumed mapping must reconcile by type, root UUID, version, OSCAL version, and SHA-256.

## Functional Model :yellow_circle: `@human-review`

### Decision States

| State | Meaning | Required Evidence |
|-------|---------|-------------------|
| `applicable` | The reviewer includes the control in organizational scope. | Reviewer and review time; rationale optional |
| `not-applicable` | The reviewer excludes the control from scope. | Reviewer, review time, and non-empty rationale |
| `deferred` | The decision is intentionally postponed until a named review date or trigger. | Reviewer, review time, rationale, and revisit date |
| `under-review` | No applicability conclusion has been approved. | Optional assignee and note; never counted as applicable |

Omitted controls default to `under-review`; omission never means `not-applicable`.

### Gap Classification

Each eligible framework control appears exactly once as one of:

- `applicable-mapped`
- `applicable-reviewed-no-relationship`
- `applicable-unmapped`
- `not-applicable`
- `deferred`
- `under-review`

A control is `applicable-mapped` only when it participates on the framework side of at least one valid, non-stale positive relationship in an accepted Mapping Collection. A control participating only in explicit `no-relationship` maps is `applicable-reviewed-no-relationship`; it has been reviewed but still lacks a positive policy relationship. If a control has both positive and `no-relationship` edges to different policy subjects, the primary classification is `applicable-mapped` and the report preserves the `no-relationship` edge count as a secondary review fact.

### Aggregation

The MVP may consume multiple Mapping Collections only when all target the same exact framework resource. Duplicate policy sources are allowed, but duplicate Mapping Collection UUIDs or contradictory resource fingerprints are fatal. Counts reconcile to the complete eligible control inventory.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge applicability init --framework <FILE> --output <FILE>` and `forge applicability analyze --manifest <FILE>` with text/JSON report and output options.
- [ ] **M-2 — Closed manifest:** Parse a bounded `forge.applicability/1` JSON manifest; reject unknown keys, duplicate decoded keys, unsupported versions, and exceeded limits.
- [ ] **M-3 — Framework validation:** Accept a local JSON Catalog or Profile; require and fingerprint a resolved Catalog companion for a Profile.
- [ ] **M-4 — Inventory:** Recursively inventory eligible framework controls and reject duplicate or ambiguous IDs.
- [ ] **M-5 — Decisions:** Validate state vocabulary, control references, reviewer references, timestamps, required rationale, and revisit dates.
- [ ] **M-6 — Default:** Classify omitted controls as `under-review`; never infer exclusions or applicability.
- [ ] **M-7 — Mapping inputs:** Validate every Mapping Collection against the pinned schema and require its framework-side identity to match the manifest exactly.
- [ ] **M-8 — Classification:** Emit exactly one deterministic classification per eligible control and reconcile category counts to the inventory total.
- [ ] **M-9 — Terminology:** Use mapping participation and review-state language; prohibit compliance, effectiveness, implementation, or certification labels in generated reports.
- [ ] **M-10 — Provenance:** Report exact framework and mapping fingerprints, root UUIDs, metadata versions, OSCAL versions, manifest hash, reviewers, and analysis schema version.
- [ ] **M-11 — Conflict handling:** Reject duplicate decisions, conflicting control states, duplicate mapping UUIDs, stale references, and mismatched resource sides before writing output.
- [ ] **M-12 — Determinism:** Sort by stable control ID and produce byte-identical JSON for identical inputs without timestamps, absolute paths, or environment data.
- [ ] **M-13 — Safe I/O:** Operate offline, bound input sizes/depth/counts, reject output/input aliases, and use atomic safe writes.
- [ ] **M-14 — Exit contract:** Exit `0` for complete valid analysis, `1` when review-action categories are present under the selected gate policy, and `2` for invalid input or analysis failure.
- [ ] **M-15 — Tests:** Cover all state, mismatch, stale-reference, conflict, determinism, safety, and terminology scenarios.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Filter reports by group, control prefix, state, reviewer, or policy source without changing totals.
- [ ] **S-2:** Emit a machine-readable review queue containing stable reason codes and owner/revisit metadata.
- [ ] **S-3:** Support an approved, explicit gate policy such as no `applicable-unmapped` or overdue `deferred` controls.
- [ ] **S-4:** Produce a static HTML report from the same versioned report model.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Interactive web review backed by the same manifest/report contracts.
- [ ] **C-2:** Policy-authoring recommendations based only on approved gaps and explicit organization context.
- [ ] **C-3:** Multi-framework portfolio rollups that retain per-framework denominators and versions.

### Won't Have (W) — This release :red_circle: `@human-required`

- Automatic applicability, mapping, policy generation, compliance scoring, framework downloads, or remote workflow actions.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | A 100-control framework with 60 applicable, 10 excluded, 5 deferred, and 25 omitted controls | Analysis runs | Totals reconcile to 100 and omitted controls are `under-review` |
| AC-2 | Forty of 60 applicable controls participate in valid reviewed maps | Analysis runs | Report shows 40 `applicable-mapped` and 20 `applicable-unmapped`, never 66.7% compliant |
| AC-3 | A `not-applicable` decision lacks rationale | Analysis runs | Exit is `2`, the manifest path is identified, and no report is written |
| AC-4 | A Mapping Collection fingerprints a different framework revision | Analysis runs | It is rejected rather than merged into the analysis |
| AC-5 | The same inputs are analyzed on two supported platforms | Outputs are compared | JSON bytes match exactly |
| AC-6 | A framework contains a control omitted from the manifest | Scaffold or analysis runs | The control remains visible and is never silently excluded |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Valid task completion | 4 of 5 partners complete a 100-control review without maintainer edits | Moderated pilot |
| Leading | Classification comprehension | 5 of 5 distinguish mapped participation from compliance | Post-task questions |
| Leading | Invalid-state prevention | 100% seeded invalid or stale decisions rejected | Automated fixtures |
| Lagging | Workflow time | 40% median reduction versus partner baseline | Within-participant comparison |
| Lagging | Reuse | Three partners rerun the analysis after a policy or mapping revision within 90 days | Opt-in partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** PRD 054 schema baseline and PRD 055 Mapping Collection build/validation.
- **Phase 1:** Catalog baseline, explicit decisions, single Mapping Collection, deterministic report.
- **Phase 2:** Profile companion, multiple mapping aggregation, CI gate policy, review queue.
- **Phase 3:** Design-partner validation and optional static HTML report.
- **Enables:** PRD 057 framework-change impact and a later framework-guided policy authoring workspace.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Users interpret mapped controls as satisfied controls | False assurance | Guardrail terminology, comprehension tests, and no compliance score |
| Users exclude controls to improve totals | Misleading scope | Visible exclusion counts, mandatory rationale, reviewer attribution, immutable provenance |
| Licensed content leaks through reports | Legal and confidentiality exposure | User-supplied inputs, IDs/hashes by default, no bundled standards or default excerpts |
| Framework IDs change | Stale analysis | Exact fingerprints, fatal mismatches, and PRD 057 impact workflow |
| Large inventories become unusable | Review fatigue | Deterministic queues and filters without hiding denominator totals |

## Open Questions :yellow_circle: `@human-review`

- **[Product, non-blocking]** Should controls with both positive and `no-relationship` edges receive a dedicated mixed-review flag in addition to their primary classification?
- **[Compliance, blocking]** Which roles may approve `not-applicable` and `deferred` decisions in the reference workflow?
- **[Engineering, non-blocking]** Should gate policy live in the applicability manifest or `.forge.toml` once the core report contract is stable?
- **[Legal, non-blocking]** What default report fields minimize licensed-framework excerpt risk across design-partner inputs?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves state semantics and non-compliance language.
- [ ] Compliance approves rationale and reviewer requirements.
- [ ] Engineering approves manifest v1, resource matching, limits, and exit contract.
- [ ] Legal approves framework-content handling guidance.
- [ ] Three design partners provide lawfully usable test inputs.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Separate applicability from mapping | A relationship cannot determine whether a control is in organizational scope | Derive scope from mapping presence |
| 2026-08-24 | Report mapping participation, not compliance | Policies, implementation, evidence, and effectiveness are distinct | Coverage/compliance percentage |
| 2026-08-24 | Default omitted controls to `under-review` | Absence must not become an exclusion | Default applicable; default not applicable |
| 2026-08-24 | Consume PRD 055 artifacts | One mapping truth avoids duplicate relationship semantics | Embed mappings in applicability manifest |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for reviewed framework applicability and policy-gap analysis |

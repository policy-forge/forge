# 061-prd-framework-guided-policy-authoring

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `061-framework-guided-policy-authoring`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will turn an approved applicability and gap analysis into a traceable policy drafting plan and deterministic policy skeleton. A reviewer-supplied authoring pack connects framework control IDs to policy topics, required organization questions, and reusable components; organization answers and human-authored clauses fill the plan. The MVP does not invent applicability, bundle restricted standards, generate final prose with AI, or claim that a completed draft satisfies a framework.

## Context

### Background :red_circle: `@human-required`

PRD 056 identifies applicable controls that lack reviewed policy relationships, while PRD 059 composes reusable policy text. Users still need to decide which policies should address those gaps, what organization-specific facts must be supplied, and how each draft clause traces to the originating gap. A generic blank-page generator would obscure those decisions and encourage boilerplate that is structurally complete but operationally false.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | FORGE already supports policy ingestion, traceability, deterministic IDs, Profiles, mappings, and gap reports. | Authoring should consume those reviewed artifacts instead of reinterpreting framework prose. |
| Product boundary | Framework licensing and applicability decisions remain user responsibilities. | Authoring packs are user-supplied and fingerprinted; FORGE bundles only synthetic examples. |
| Product hypothesis | Guided gap-to-draft work is more valuable and trustworthy than unrestricted policy generation. | Measure approved-clause completion and revision burden, not words generated. |

No design-partner authoring study, approved framework pack, or validated time-savings baseline was supplied. Targets remain hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- One valid PRD 056 applicability/gap report tied to an exact framework resource
- A versioned, reviewer-authored local authoring pack mapping control IDs to policy topics and organization questions
- Explicit policy-family assignments for applicable gap controls
- Required organization-context answers with source, owner, review time, and sensitivity label
- Deterministic drafting plan, unresolved-question queue, and policy skeletons
- Optional use of pinned PRD 059 components and explicit human-authored clause files
- Clause-to-gap, clause-to-control, answer-to-clause, and component-to-clause provenance
- Validation that every included claim has an approved input origin

**Out of Scope:**

- Automatic applicability, control mapping, legal interpretation, or compliance conclusions
- Bundled ISO, SOC 2, CIS, PCI DSS, or other restricted framework content
- AI-generated prose or semantic mapping; PRD 066 owns suggestion behavior
- Policy approval/lifecycle transitions; PRD 058 owns those decisions
- Web editing, collaboration, evidence, implementation, or external connectors

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/016-prd-traceability-model.md` | Clause provenance |
| `docs/PRD/055-prd-control-mapping.md` | Reviewed policy/framework relationships |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Authoritative applicable-gap input |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Draft review and approval after authoring |
| `docs/PRD/059-prd-reusable-policy-components.md` | Pinned reusable sections |
| `docs/PRD/066-prd-ai-assisted-suggestions.md` | Optional future drafting suggestions |

---

## Problem Statement :red_circle: `@human-required`

Compliance engineers know which applicable controls lack reviewed policy relationships but still reconstruct policy outlines, questionnaires, and traceability manually. Without a guided authoring contract, teams either write from a blank page or accept generic boilerplate that may conflict with actual roles, systems, and risk decisions.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Convert approved gaps into actionable drafting work. | Every applicable gap is assigned, explicitly deferred, or remains visibly unresolved. |
| G-2 | Require organization-specific context before claims appear. | 100% of seeded missing required answers block affected clauses without blocking unrelated drafts. |
| G-3 | Preserve end-to-end provenance. | Every skeleton section and included clause traces to gap IDs, control IDs, authoring-pack bytes, answers, and component/source bytes. |
| G-4 | Keep output deterministic and reviewable. | Identical inputs produce byte-identical plans, skeletons, and provenance reports. |
| G-5 | Reduce time to a reviewable first draft. | Five pilots reach a complete reviewable skeleton 50% faster than their current process. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- A complete skeleton is not an approved policy or compliance determination.
- FORGE does not decide which policy family should own a control unless the authoring pack explicitly declares it.
- FORGE does not manufacture organization facts or silently apply defaults to substantive questions.
- The MVP does not create an authoring-pack marketplace or remote registry.
- The MVP does not replace legal, compliance, or executive review.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Turn gaps into a drafting plan (P0)

> As a compliance engineer, I want applicable gaps grouped into explicit policy work so that no control disappears between analysis and authoring.

### US-2 — Answer organization questions once (P0)

> As a policy owner, I want required facts and decisions presented before drafting so that policy text reflects actual operations.

### US-3 — Build traceable skeletons (P0)

> As a policy author, I want deterministic policy outlines populated only with reviewed components and clauses so that I can focus on unresolved content.

### US-4 — Review unsupported claims (P0)

> As an auditor, I want every clause with missing or stale provenance blocked or flagged so that polished text cannot conceal unsupported assumptions.

### US-5 — Re-plan after a gap change (P1)

> As a compliance engineer, I want a baseline comparison so that framework or applicability changes reveal affected drafts without rewriting unrelated policies.

## Authoring Model :yellow_circle: `@human-review`

The authoring pack is a closed `forge.authoring-pack/1` JSON document containing stable topic keys, eligible framework control IDs, policy-family keys, required question definitions, optional PRD 059 component references, and human-reviewed rationale. It is a mapping claim and must record reviewer provenance and the exact framework fingerprint.

Organization answers are typed data with stable answer keys, owner, source label, review time, sensitivity, and optional expiry. Missing answers remain explicit placeholders in the drafting plan; FORGE never substitutes inferred values.

Draft state is one of `planned`, `blocked-context`, `skeleton-ready`, or `human-draft-present`. These are authoring states, not PRD 058 lifecycle approval states.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge author plan --manifest <FILE>` and `forge author build --manifest <FILE>` with text/JSON reports and output-directory controls.
- [ ] **M-2 — Closed inputs:** Validate bounded `forge.authoring-pack/1` and `forge.author-project/1` schemas; reject unknown/duplicate keys and unsupported versions.
- [ ] **M-3 — Exact baseline:** Require the authoring pack and project to match the PRD 056 framework and report fingerprints exactly.
- [ ] **M-4 — Gap accounting:** Assign every applicable gap to one or more policy topics, an explicit deferral, or an unresolved queue; totals must reconcile.
- [ ] **M-5 — Human provenance:** Require reviewer, time, and rationale for control-to-topic and topic-to-policy-family assignments.
- [ ] **M-6 — Question model:** Support bounded typed questions, required/optional status, owner, sensitivity, expiry, validation constraints, and explicit no-answer state.
- [ ] **M-7 — No fabricated context:** Block only dependent sections when required answers are missing, stale, or invalid; never apply substantive hidden defaults.
- [ ] **M-8 — Skeleton output:** Emit deterministic Markdown skeletons with one policy title, ordered sections, visible unresolved markers, and no assertion that a control is satisfied.
- [ ] **M-9 — Approved content only:** Include only explicitly supplied human clause files or hash-pinned PRD 059 component instances; never synthesize prose.
- [ ] **M-10 — Provenance:** Emit a machine-readable graph from output spans to policy topic, gaps, control IDs, answers, clauses, component instances, and all input hashes.
- [ ] **M-11 — Safe output:** Use project-root containment, alias rejection, atomic writes, bounded content, and no absolute paths in artifacts.
- [ ] **M-12 — Determinism:** Exclude wall-clock time, locale, environment, and directory location from identity and canonical outputs.
- [ ] **M-13 — Baseline impact:** Distinguish added/removed gaps, answer changes, pack changes, component drift, human-clause changes, and unaffected skeletons.
- [ ] **M-14 — Terminology:** Never emit compliant, certified, implemented, effective, or approved based on authoring completeness.
- [ ] **M-15 — Tests:** Cover gap reconciliation, missing context, stale packs, component drift, provenance completeness, safe paths, and deterministic rebuilds.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Scaffold an empty authoring pack from a valid framework inventory without creating assignments.
- [ ] **S-2:** Generate a static HTML drafting plan and provenance view.
- [ ] **S-3:** Link built drafts into PRD 058 lifecycle records as `draft` without approving them.
- [ ] **S-4:** Allow multiple policy families to share one gap while preserving responsibility boundaries.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** PRD 066 AI suggestions for unanswered questions, outlines, or clauses with mandatory review.
- [ ] **C-2:** PRD 062 web authoring workspace.
- [ ] **C-3:** Lawfully distributed signed authoring-pack registries under a separate supply-chain design.

### Won't Have (W) — This release :red_circle: `@human-required`

- AI prose, automatic assignments, framework downloads, compliance scoring, approval, collaboration, or hosted authoring.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Twenty applicable gaps and an approved authoring pack | Planning runs | All twenty are assigned, deferred, or visibly unresolved and totals reconcile |
| AC-2 | One required organization answer is missing | Build runs | Only dependent sections are `blocked-context`; no value is inferred |
| AC-3 | A pinned component changes | Build runs | Exit is `2` and existing outputs remain unchanged |
| AC-4 | A human clause addresses three gap IDs | Build runs | All three relationships and exact source spans appear in provenance |
| AC-5 | The same project builds in two directories | Outputs are compared | Plan, skeleton, and provenance bytes match |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Gap accounting | 100% of seeded gaps reconciled | Invariant tests |
| Leading | First-draft task completion | 4 of 5 pilots complete without maintainer edits | Moderated study |
| Leading | Unsupported claims | Zero hidden defaults or synthesized clauses | Fixtures and human review |
| Lagging | Time to reviewable skeleton | 50% median reduction | Partner before/after comparison |
| Lagging | Draft acceptance | At least 70% of skeleton sections retained through human review | Sanitized diff analysis |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** PRD 056 applicability/gap output and existing traceability/safe-I/O contracts.
- **Phase 1:** Plan, questions, skeletons, human clause files, and provenance.
- **Phase 2:** PRD 059 components, impact analysis, static HTML, and lifecycle handoff.
- **Phase 3:** Design-partner authoring packs and measured gap-to-draft exercises.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Skeleton is mistaken for a complete policy | False assurance | Visible draft state, unresolved markers, no coverage/compliance language |
| Authoring pack embeds unlicensed content | Legal exposure | User-supplied files, hashes/IDs by default, rights attestation and no bundled standards |
| Question answers contain sensitive data | Confidentiality risk | Sensitivity labels, minimal output, hash values in reports by default |
| Topic assignments become opaque mappings | Audit failure | Per-assignment rationale and reviewer provenance |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** What is the smallest authoring-pack schema that supports ISO/SOC 2-style workflows without encoding framework-specific assumptions?
- **[Compliance, blocking]** Which organization-answer classes must always require independent review?
- **[Legal, blocking]** Can authoring packs include control titles, or should the portable contract default to IDs only?
- **[Engineering, non-blocking]** Should one build emit multiple policy files atomically or one policy per invocation?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product and Compliance approve authoring states, assignment semantics, and disclaimers.
- [ ] Legal approves authoring-pack content boundaries.
- [ ] Engineering approves schemas, provenance graph, bounds, and atomic multi-output behavior.
- [ ] Three design partners supply lawful framework inputs and real gap-to-draft workflows.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Generate plans and skeletons before prose | Organization context and reviewed ownership must precede policy assertions | Blank-page AI policy generation |
| 2026-08-24 | Make authoring packs user-supplied and fingerprinted | Framework mappings and content rights require accountable provenance | Bundled universal packs |
| 2026-08-24 | Keep authoring state separate from approval state | Draft completeness is not governance approval | Reuse lifecycle states |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for framework-guided, provenance-preserving policy authoring |

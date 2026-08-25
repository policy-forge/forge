# 057-prd-framework-change-impact-monitoring

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `057-framework-change-impact-monitoring`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will compare an approved framework baseline with a caller-supplied revision and produce a deterministic review queue showing which applicability decisions, mappings, policy links, and gap classifications may be stale. The MVP detects and explains impact; it never downloads framework updates, guesses renamed controls, rewrites mappings, or declares continued compliance.

## Context

### Background :red_circle: `@human-required`

PRD 055 can identify changes affecting one Mapping Collection, and PRD 056 defines organization-specific applicability and policy-gap state. Compliance teams still need a portfolio-level answer when a framework revision adds, removes, renames, or changes controls: what must be reviewed, which policy relationships are affected, and which prior decisions remain unchanged?

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | `forge diff`, PRD 053 migration analysis, and PRD 055 baseline checks already distinguish stable identity, content change, stale references, and new gaps. | Reuse these semantics and aggregate their impact rather than inventing fuzzy successor matching. |
| Product principle | Resource bytes, stable IDs, and explicit human decisions are evidence. | Monitoring must fingerprint both versions and preserve the exact dependency path for each finding. |
| Product hypothesis | Teams will maintain mappings more consistently when framework changes produce a bounded review queue. | Measure completed update cycles, not alert volume. |

No live framework feed, design-partner revision corpus, or completed maintenance-time study was supplied. Monitoring value and time-savings targets remain hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Local old/new OSCAL Catalogs or Profile-plus-resolved-Catalog pairs
- Exact resource fingerprints and deterministic subject inventories
- Explicit optional PRD 053 identity migration input for reviewed renames/splits/merges
- PRD 055 Mapping Collections and PRD 056 applicability manifests/reports tied to the old baseline
- Added, removed, content-changed, identity-migrated, and unchanged control classifications
- Dependency traversal from changed controls to applicability decisions, maps, policy sources, and gap states
- Prioritized deterministic text/JSON impact reports and CI-friendly exit statuses
- A machine-readable review queue with stable finding IDs and reason codes

**Out of Scope:**

- Network polling, subscriptions, vendor feeds, scheduled execution, email, or chat notifications
- Automatic successor detection, fuzzy matching, semantic equivalence, or mapping repair
- Automatic applicability changes, policy edits, lifecycle transitions, or evidence invalidation
- Framework redistribution or a hosted framework registry
- Compliance conclusions or claims that unchanged text remains effective

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/043-prd-diff-report.md` | Deterministic artifact differences |
| `docs/PRD/052-prd-github-action-drift-enforcement.md` | CI enforcement and drift conventions |
| `docs/PRD/053-prd-stable-id-migration.md` | Human-declared identity migration semantics |
| `docs/PRD/055-prd-control-mapping.md` | Per-mapping baseline impact |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Applicability and gap state affected by revisions |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Policy review scheduling that may consume impact findings |

---

## Problem Statement :red_circle: `@human-required`

When a framework changes, compliance engineers must manually determine which scope decisions, crosswalks, and policies need review. Without an exact dependency-aware impact report, teams either re-review everything, miss stale relationships, or rely on guessed successor mappings that obscure the difference between evidence and inference.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Detect framework subject changes completely. | 100% of seeded additions, removals, same-ID content changes, and explicit identity migrations are classified correctly. |
| G-2 | Explain downstream blast radius. | Every affected applicability decision and map includes a stable dependency path to the changed subject. |
| G-3 | Avoid invented continuity. | No successor or unchanged-status claim is emitted without stable identity or an approved migration record. |
| G-4 | Support repeatable review gates. | Identical inputs produce byte-identical findings, priorities, and exit status across supported platforms. |
| G-5 | Reduce maintenance effort. | Design partners complete a framework revision review in 50% less time than their prior process. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- FORGE does not discover or download new framework releases.
- FORGE does not interpret regulatory significance or effective dates.
- FORGE does not infer that similar prose represents a rename or equivalent control.
- FORGE does not mutate mappings, applicability decisions, policies, or evidence records.
- A clean structural report does not certify continuing compliance or implementation effectiveness.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — See the framework delta (P0)

> As a compliance engineer, I want exact added, removed, and changed controls so that I can understand the revision before updating downstream work.

### US-2 — Find impacted decisions and mappings (P0)

> As a compliance engineer, I want each framework change traced to applicability decisions and policy mappings so that I review only the affected scope.

### US-3 — Preserve identity uncertainty (P0)

> As an auditor, I want possible renames treated as unresolved unless a reviewer supplies a migration decision so that FORGE does not manufacture continuity.

### US-4 — Gate stale baselines in CI (P0)

> As a DevSecOps engineer, I want stable exit statuses and JSON findings so that a framework update can block publication until required reviews are complete.

### US-5 — Track review disposition (P1)

> As a compliance engineer, I want stable finding IDs that another workflow can resolve or waive so that repeated runs preserve review continuity.

## Impact Model :yellow_circle: `@human-review`

### Change Classes

| Class | Meaning | Default Action |
|-------|---------|----------------|
| `added` | A new eligible control exists only in the new baseline. | Applicability review required |
| `removed` | An old control no longer exists and has no approved migration. | All dependent decisions/maps require review |
| `content-changed` | Stable ID remains but the canonical eligible subtree changed. | Applicability and relationship rationale require review |
| `identity-migrated` | An approved PRD 053 record connects old and new identity. | Review migration cardinality and all dependent relationships |
| `unchanged` | Stable ID and canonical content fingerprint match. | No structural-review action |

`unchanged` means unchanged within the defined canonical comparison, not unchanged legal meaning, applicability, effectiveness, or evidence sufficiency.

### Finding Priority

| Priority | Condition |
|----------|-----------|
| `blocking` | Stale mapped reference, invalid resource identity, ambiguous/invalid migration, or corrupt baseline |
| `review-required` | Applicable control added, mapped control content changed, approved split/merge, or exclusion rationale tied to changed content |
| `informational` | Unmapped control changed, metadata-only resource revision, or unchanged dependency |

Priority is rule-based and documented; it is not an AI confidence score.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Command:** Provide `forge framework impact --manifest <FILE>` with text/JSON report, output, and gate-policy options.
- [ ] **M-2 — Closed manifest:** Parse bounded `forge.framework-impact/1` JSON containing old/new resources and optional applicability, mapping, and migration inputs.
- [ ] **M-3 — Resource validation:** Schema-validate old/new Catalogs or Profile companion pairs and verify declared type, root identity, metadata version, OSCAL version, and hashes.
- [ ] **M-4 — Canonical inventory:** Recursively inventory controls and hash the documented eligible subtree using the same contract as PRD 055 where applicable.
- [ ] **M-5 — Exact classification:** Classify additions, removals, stable-ID content changes, and unchanged controls without fuzzy matching.
- [ ] **M-6 — Migration input:** Accept only validated PRD 053 migration records for rename, split, merge, or continuity claims; preserve their reviewer evidence.
- [ ] **M-7 — Dependency validation:** Require every applicability and mapping input to reference the exact old baseline and reject stale, mixed, or ambiguous portfolios.
- [ ] **M-8 — Blast radius:** Link each changed control to affected applicability decisions, mappings, policy resource identities, and prior gap classification.
- [ ] **M-9 — Stable findings:** Derive finding IDs from the impact schema version, old/new resource fingerprints, subject identity, change class, and dependency identity.
- [ ] **M-10 — Priorities:** Apply documented deterministic priority rules and preserve the reason code and dependency path for every finding.
- [ ] **M-11 — Review queue:** Emit sorted machine-readable findings with required action, old/new IDs and hashes, affected artifact IDs, and no framework prose by default.
- [ ] **M-12 — Non-mutation:** Never rewrite or approve mappings, applicability decisions, lifecycle records, policies, or migration records.
- [ ] **M-13 — Determinism:** Identical input bytes yield byte-identical JSON and ordering without runtime timestamps or absolute paths.
- [ ] **M-14 — Safety:** Operate offline with bounded reads, depth/count limits, alias rejection, safe writes, and terminal-safe text rendering.
- [ ] **M-15 — Exit contract:** Exit `0` for no gated findings, `1` for valid review-required/blocking findings, and `2` for invalid analysis.
- [ ] **M-16 — Tests:** Cover all change classes, migration cardinalities, dependency paths, mixed-baseline rejection, determinism, and safe I/O.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Accept a prior impact report plus disposition file and preserve resolved, accepted-risk, and still-open findings without changing raw detection.
- [ ] **S-2:** Produce Markdown and static HTML summaries from the same versioned report model.
- [ ] **S-3:** Filter by framework group, decision state, policy source, impact priority, or owner.
- [ ] **S-4:** Emit GitHub-compatible annotations without posting them or mutating repository state.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Scheduled local monitoring wrapper that invokes the deterministic core.
- [ ] **C-2:** Signed upstream framework registries and authenticated update feeds under a separate supply-chain design.
- [ ] **C-3:** Web review queue and assignments backed by the same finding contract.

### Won't Have (W) — This release :red_circle: `@human-required`

- Remote retrieval, semantic successor suggestions, automatic repair, automatic dispositions, or compliance recertification.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | One control is added, one removed, one same-ID control changed, and one unchanged | Impact runs | Each receives exactly the correct change class |
| AC-2 | A removed control participates in three maps and one applicability decision | Impact runs | One subject change and four explicit dependency impacts are reported with stable paths |
| AC-3 | Similar prose appears under a new ID without migration evidence | Impact runs | The old ID is removed and the new ID is added; no rename is inferred |
| AC-4 | An approved split migration maps one old ID to two new IDs | Impact runs | The split and both new review targets are shown without transferring approval |
| AC-5 | A Mapping Collection targets a different old-baseline hash | Impact runs | Analysis exits `2` and emits no partial portfolio result |
| AC-6 | The same inputs run twice | Reports are compared | Finding IDs, priority, ordering, and bytes match |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Seeded detection completeness | 100% | Human-adjudicated fixtures |
| Leading | False continuity | Zero inferred successors | Contract tests and pilot review |
| Leading | Finding actionability | 4 of 5 partners identify required artifact review without maintainer explanation | Moderated task |
| Lagging | Revision review time | 50% median reduction | Partner before/after comparison |
| Lagging | Completed maintenance cycles | Three organizations complete two framework revisions within six months | Opt-in partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** PRD 053 identity migrations, PRD 055 mapping baseline/fingerprints, and PRD 056 applicability contracts.
- **Phase 1:** Catalog-to-Catalog delta plus mapping/applicability blast radius.
- **Phase 2:** Profile companions, migration cardinalities, dispositions, and CI annotations.
- **Phase 3:** Design-partner framework-revision exercises and release gate.
- **Integrates later with:** PRD 058 review schedules and PRD 060 evidence-link review queues.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Alert volume causes users to ignore findings | Review failure | Aggregate by changed subject, expose dependency counts, retain deterministic filters |
| Same-ID text change is cosmetically noisy | Excess review | Canonicalize only documented structural volatility; never hide substantive fields |
| Users expect legal interpretation | Incorrect assurance | Structural-impact wording and explicit non-goals in every report |
| Explicit migrations are tedious | Temptation to infer | Reuse PRD 053 scaffolding and measure unresolved workload before adding suggestions |
| Mixed framework versions create false results | Corrupt impact graph | Exact fingerprints and fatal portfolio consistency checks |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Which impact classes should default to exit `1`, and should users be able to make the gate stricter but not weaker?
- **[Compliance, blocking]** Does a same-ID content change always require re-approval of an exclusion, or can an approved materiality rule suppress it?
- **[Engineering, blocking]** Should the portfolio input reference raw PRD 056 manifests, reports, or both as the authoritative dependency source?
- **[Product, non-blocking]** Should disposition tracking remain in this PRD or be owned entirely by PRD 058 lifecycle workflows?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves impact classes, priorities, and gate defaults.
- [ ] Compliance approves re-review semantics and non-certification language.
- [ ] Engineering approves canonical hashing, portfolio consistency, stable finding IDs, and bounds.
- [ ] Synthetic old/new framework fixtures cover every supported change and migration class.
- [ ] Three design partners provide representative revision workflows.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Monitor explicit local revisions | Keeps framework acquisition, licensing, and trust outside the deterministic core | Built-in update feed |
| 2026-08-24 | Require reviewed migration evidence for identity continuity | Similarity is not proof of succession | Fuzzy or semantic rename matching |
| 2026-08-24 | Report dependency impact without mutation | Human review must precede applicability or mapping changes | Automatic repair |
| 2026-08-24 | Evaluate completed update cycles, not alert count | More alerts are not more customer value | Alert-volume adoption metric |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for deterministic framework-change blast-radius monitoring |

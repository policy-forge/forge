# 057-prd-framework-change-impact-monitoring

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Technical implementation complete — human release gates pending
> **Last Updated:** 2026-08-25 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `057-framework-change-impact-monitoring`
**Created**: 2026-08-24
**Status**: Technical implementation complete — human release gates pending
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
| `blocking` | A removed control still has an exact mapped reference that requires repair or explicit review. Invalid identity, migration, or baseline evidence instead makes analysis incomplete and exits `2` without a report. |
| `review-required` | Applicable control added, mapped control content changed, approved split/merge, or exclusion rationale tied to changed content |
| `informational` | Unmapped control changed or metadata-only resource revision |

Priority is rule-based and documented; it is not an AI confidence score.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [x] **M-1 — Command:** Provide `forge framework impact --manifest <FILE>` with text/JSON report, output, and gate-policy options.
- [x] **M-2 — Closed manifest:** Parse bounded `forge.framework-impact/1` JSON containing old/new resources and optional applicability, mapping, and migration inputs.
- [x] **M-3 — Resource validation:** Schema-validate old/new Catalogs or Profile companion pairs and verify declared type, root identity, metadata version, OSCAL version, and hashes.
- [x] **M-4 — Canonical inventory:** Recursively inventory controls and hash the documented eligible subtree using the same contract as PRD 055 where applicable.
- [x] **M-5 — Exact classification:** Classify additions, removals, stable-ID content changes, and unchanged controls without fuzzy matching.
- [x] **M-6 — Migration input:** Accept only validated PRD 053 migration records for rename, split, merge, or continuity claims; preserve their reviewer evidence.
- [x] **M-7 — Dependency validation:** Require every applicability and mapping input to reference the exact old baseline and reject stale, mixed, or ambiguous portfolios.
- [x] **M-8 — Blast radius:** Link each changed control to affected applicability decisions, mappings, policy resource identities, and prior gap classification.
- [x] **M-9 — Stable findings:** Derive finding IDs from the impact schema version, old/new resource fingerprints, subject identity, change class, and dependency identity.
- [x] **M-10 — Priorities:** Apply documented deterministic priority rules and preserve the reason code and dependency path for every finding.
- [x] **M-11 — Review queue:** Emit sorted machine-readable findings with required action, old/new IDs and hashes, affected artifact IDs, and no framework prose by default.
- [x] **M-12 — Non-mutation:** Never rewrite or approve mappings, applicability decisions, lifecycle records, policies, or migration records.
- [x] **M-13 — Determinism:** Identical input bytes yield byte-identical JSON and ordering without runtime timestamps or absolute paths.
- [x] **M-14 — Safety:** Operate offline with bounded reads, depth/count limits, alias rejection, safe writes, and terminal-safe text rendering.
- [x] **M-15 — Exit contract:** Exit `0` for no gated findings, `1` for valid review-required/blocking findings, and `2` for invalid analysis.
- [x] **M-16 — Tests:** Cover all change classes, migration cardinalities, dependency paths, mixed-baseline rejection, determinism, and safe I/O.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [x] **S-1:** Accept a prior impact report plus disposition file and preserve resolved, accepted-risk, and still-open findings without changing raw detection.
- [x] **S-2:** Produce Markdown and static HTML summaries from the same versioned report model.
- [x] **S-3:** Filter by framework group, decision state, policy source, impact priority, or owner.
- [x] **S-4:** Emit GitHub-compatible annotations without posting them or mutating repository state.

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

### Executable Requirement Traceability :white_circle: `@auto`

| Requirements | Executable coverage |
|--------------|---------------------|
| M-1, M-5, M-9, M-10, M-11, M-13, M-15; AC-1, AC-3, AC-6 | `classifies_exact_changes_and_emits_byte_identical_reports`; `identical_prose_under_a_new_id_is_not_an_inferred_successor`; `gate_thresholds_and_destination_aliases_are_enforced` |
| M-2, M-3, M-14 | `duplicate_manifest_keys_are_rejected`; `closed_manifest_enforces_unknown_field_depth_and_size_bounds_without_output`; `mixed_catalog_and_profile_revisions_are_rejected`; `symlinked_manifest_is_rejected_without_output`; `text_report_escapes_terminal_control_characters` |
| M-4 | `classifies_exact_changes_and_emits_byte_identical_reports`; Mapping inventory unit and integration suites |
| M-6; AC-4 | `declared_successor_split_and_merge_preserve_cardinality_and_review_evidence`; successor-map parser and migration engine unit tests |
| M-7; AC-5 | `rejects_mapping_from_a_different_baseline_without_partial_output`; `applicability_impacts_preserve_prior_gap_state_and_feed_lifecycle_review` |
| M-8; AC-2 | `traverses_exact_mapping_dependencies_with_stable_paths_and_priorities`; `removed_control_reports_three_mapping_and_one_applicability_dependency` |
| M-12 | Destination-alias and byte-preservation assertions across framework-impact integration tests |
| M-16 | Complete framework unit and integration suites, including every row above |
| S-1 | `dispositions_preserve_raw_findings_control_gates_and_retain_prior_only_history` |
| S-2 | `markdown_and_html_cli_formats_render_complete_static_reports`; `markdown_and_html_reports_are_deterministic_and_escape_injected_content`; `report_escaping_covers_markdown_and_html_control_characters` |
| S-3 | `filters_exact_finding_details_without_changing_totals_or_gate`; `filters_reject_ambiguous_groups_invalid_values_and_unsafe_policy_hrefs`; `migration_group_filter_uses_sorted_old_and_new_group_union` |
| S-4 | `github_annotations_are_deterministic_content_safe_workflow_commands`; `github_workflow_command_fields_are_escaped` |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Seeded detection completeness | 100% | Human-adjudicated fixtures |
| Leading | False continuity | Zero inferred successors | Contract tests and pilot review |
| Leading | Finding actionability | 4 of 5 partners identify required artifact review without maintainer explanation | Moderated task |
| Lagging | Revision review time | 50% median reduction | Partner before/after comparison |
| Lagging | Completed maintenance cycles | Three organizations complete two framework revisions within six months | Opt-in partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** The implemented PRD 053 `forge.successor-map/1` declaration contract (M-8, M-9, and M-23), PRD 055 mapping baseline/fingerprints, and PRD 056 applicability contracts.
- **Phase 1:** Catalog-to-Catalog delta plus Mapping Collection blast radius.
- **Phase 2:** Profile companions, applicability blast radius, and PRD 058 lifecycle handoff.
- **Phase 3:** PRD 053 migration cardinalities, dispositions, and CI annotations. **Implemented.**
- **Fast follows:** Markdown/static HTML output and exact detail filters. **Implemented.**
- **Phase 4:** Design-partner framework-revision exercises and release gate.
- **Integrates with:** PRD 058 review schedules now; PRD 060 evidence-link review queues later.

### Implementation Status :white_circle: `@auto`

The implemented core provides `forge framework impact --manifest`, the closed
`forge.framework-impact/1` manifest, Catalog and attested Profile companion
comparisons, exact resource evidence, canonical control classification, PRD 055
Mapping Collection traversal, and PRD 056 applicability blast radius. Raw
`forge.applicability/1` is authoritative: FORGE reruns its analysis and requires
its old-baseline evidence and complete Mapping Collection portfolio to match the
impact inputs exactly. Findings retain prior gap state, owner, policy sources,
stable IDs, deterministic paths and priorities, and the `0/1/2` exit contract.
PRD 058 can consume emitted IDs on transitions into `in-review`. The shared
`forge.successor-map/1` contract now supplies reviewed successor, split, and
merge declarations to both PRD 053 policy migration and PRD 057 framework
impact; declarations remain unauthenticated caller evidence and never inferred.
Report-bound `forge.framework-impact-dispositions/1` records preserve resolved,
accepted-risk, still-open, and prior-only review history without deleting raw
findings. Resolved and accepted-risk findings suppress gates for the exact stable
finding ID; still-open and undispositioned findings retain normal gate behavior.
Markdown and static HTML provide escaped deterministic summaries, while the
`github` format emits escaped workflow commands without network access or
repository mutation. Exact filters cover group, prior decision state, policy
source, priority, and owner with AND semantics. They narrow rendered details but
never full totals or gate evaluation. All Must Have and Should Have engineering
requirements are implemented; design-partner validation and human release
approval remain pending.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Alert volume causes users to ignore findings | Review failure | Aggregate by changed subject, expose dependency counts, retain deterministic filters |
| Same-ID text change is cosmetically noisy | Excess review | Canonicalize only documented structural volatility; never hide substantive fields |
| Users expect legal interpretation | Incorrect assurance | Structural-impact wording and explicit non-goals in every report |
| Explicit migrations are tedious | Temptation to infer | Reuse PRD 053 scaffolding and measure unresolved workload before adding suggestions |
| Mixed framework versions create false results | Corrupt impact graph | Exact fingerprints and fatal portfolio consistency checks |

## Open Questions :yellow_circle: `@human-review`

No implementation questions remain. The conservative technical contracts are
recorded below; Product and Compliance approval of those contracts remains a
human release gate.

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves impact classes, priorities, and gate defaults.
- [ ] Compliance approves re-review semantics and non-certification language.
- [ ] Engineering approves canonical hashing, portfolio consistency, stable finding IDs, and bounds.
- [x] Synthetic old/new framework fixtures cover every supported change and migration class.
- [ ] Three design partners provide representative revision workflows.
- [x] Every Must Have maps to executable coverage in the traceability table.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Monitor explicit local revisions | Keeps framework acquisition, licensing, and trust outside the deterministic core | Built-in update feed |
| 2026-08-24 | Require reviewed migration evidence for identity continuity | Similarity is not proof of succession | Fuzzy or semantic rename matching |
| 2026-08-24 | Report dependency impact without mutation | Human review must precede applicability or mapping changes | Automatic repair |
| 2026-08-24 | Evaluate completed update cycles, not alert count | More alerts are not more customer value | Alert-volume adoption metric |
| 2026-08-25 | Start with the implemented Catalog and PRD 055 contracts | Delivers an independently testable dependency-aware slice while PRD 056 and approved migration input contracts remain unavailable | Invent placeholder applicability or migration schemas |
| 2026-08-25 | Default the gate to `review-required` and omit a disabled mode | Blocking and review-required findings stop the default workflow; callers may select `blocking` or stricter `any` without converting a clean exit into a compliance claim | Default `any`; allow `never` |
| 2026-08-25 | Re-run raw PRD 056 applicability manifests as the authoritative dependency source | Prevents detached-report trust and lets impact reject stale old-baseline evidence or a mixed Mapping portfolio before emitting findings | Accept reports; duplicate PRD 056 classification |
| 2026-08-25 | Pass stable finding IDs into PRD 058 review history without automatic transitions | Preserves explicit human workflow state while providing a deterministic handoff | Mutate lifecycle records during impact analysis |
| 2026-08-25 | Reuse `forge.successor-map/1` for framework identities | One closed reviewer-declaration contract supports policy stable IDs and framework control IDs while each consumer validates references against its own inventory | Create a PRD 057-only migration schema |
| 2026-08-25 | Keep dispositions in a report-bound PRD 057 file | Exact prior-report hashing and stable finding IDs preserve per-finding review outcomes; PRD 058 remains the policy review history and trigger handoff | Store all finding state in lifecycle records |
| 2026-08-25 | Suppress gates, not raw findings, for resolved and accepted risk | Review decisions can unblock repeat execution without concealing structural detection or changing priority totals | Delete findings; ignore dispositions during gating |
| 2026-08-25 | Always require review when same-ID content changes affect an exclusion | Avoids an implicit materiality judgment; an exact report-bound accepted-risk disposition may suppress the gate without deleting the finding | Automatically preserve exclusions; add an inferred materiality threshold |
| 2026-08-25 | Make filters presentation-only | Exact AND filters reduce reviewer noise while complete totals and hidden open findings continue to drive the gate | Gate only visible findings; recompute totals after filtering |
| 2026-08-25 | Derive Markdown and static HTML from the versioned report | Keeps every format deterministic, offline, escaped, and semantically aligned | Separate report pipelines; client-side rendering |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for deterministic framework-change blast-radius monitoring |
| 0.2 | 2026-08-25 | Codex | Began Phase 1 Catalog and Mapping Collection impact implementation |
| 0.3 | 2026-08-25 | Codex | Integrated Profile companions, PRD 056 applicability blast radius, and PRD 058 finding-ID handoff |
| 0.4 | 2026-08-25 | Codex | Added shared successor declarations, durable dispositions, and GitHub-compatible annotations; completed Must Have engineering scope |
| 0.5 | 2026-08-25 | Codex | Added deterministic Markdown/HTML, exact detail filters, path-safety hardening, and executable requirement traceability; completed Should Have engineering scope |

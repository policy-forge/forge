# Forge roadmap requirement and gate ledger

Date: 2026-10-02 (America/Los_Angeles). Verified main baseline:
`aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. Release target: **2.0.0**.

Scope: every Must Have and Should Have in PRDs 055–069, including conditional
Shoulds. Original WI-1–WI-50 remains complete. Could/Won't requirements are excluded.

This source inspection covers **327 requirements**: 198 with technical evidence,
28 partial, 93 missing, and 8 conditional. These are implementation/evidence
classifications, never product acceptance. Requirements were classified by source inspection; local baseline validation
and hosted exact-head CI are recorded separately. Source and test presence
does not satisfy human judgments, independent-tool parsing or actual pilots.

The [machine-readable ledger](2026-10-02-roadmap-requirement-ledger.json) retains
every exact requirement, source line and PRD digest, evidence path, remaining gate,
owner role and F01–F22 package. It also preserves 47 named gates and indexes 59
acceptance/readiness/measurement sections. [Owner decision packets](2026-10-02-roadmap-owner-decisions.md)
remain proposals; [F02 design](2026-10-02-dependency-inventory-design.md) and the
[baseline receipt](2026-10-02-roadmap-baseline.json) make the first delivery concrete. The [POA&M schema spike](2026-10-02-poam-contract-spike.md)
and [suggestion evaluation packet](2026-10-02-suggestion-evaluation-design.md) expose
reviewable contracts while their owner/adjudication gates remain pending.

## Live baseline and preservation

- Forge project 3 and Kanban 12 verified live; board was empty before setup.
- Completion epic [#2](https://kanban.luby.us/tasks/1390); F01–F07 are
  [#3](https://kanban.luby.us/tasks/1391), [#4](https://kanban.luby.us/tasks/1392),
  [#5](https://kanban.luby.us/tasks/1393), [#6](https://kanban.luby.us/tasks/1394),
  [#7](https://kanban.luby.us/tasks/1395), [#8](https://kanban.luby.us/tasks/1396),
  [#9](https://kanban.luby.us/tasks/1397). Parent and reciprocal prerequisite
  relations were read back. F01 was assigned to bot-forge and moved to Doing.
- Main [CI run 36094582840](https://github.com/policy-forge/forge/actions/runs/36094582840)
  passed Linux/macOS/Windows platform tests, API contract, strict lint and maintained
  headless client conformance. Audit/deny steps passed; cargo-vet failed with 20
  unvetted dependencies. These are 2026-09-25 hosted receipts, not fresh local checks.
- Open dependency PRs #160–#164 have exact frozen head/check snapshots in the
  baseline receipt. All have failed supply-chain checks and require individual review.
- Primary checkout remains dependency branch `704df721`, whose tracked code matches
  main; `.claude/settings.json`, untracked `.veans.yml` and original plan are preserved.
- Product-planning remains clean at `52fce93`, an ancestor of main. Local main is
  behind by four commits. Existing stash and other worktrees remain untouched,
  including two prunable registration records; no cleanup was performed.
- Delivery uses managed worktree `roadmap-f01-reconciliation` on
  `codex/roadmap-f01-reconciliation`, created from refreshed origin/main.

## Implementation summary

| PRD | Technical evidence | Partial | Missing | Conditional |
|---|---:|---:|---:|---:|
| 055 | 34 | 0 | 0 | 0 |
| 056 | 19 | 0 | 0 | 0 |
| 057 | 20 | 0 | 0 | 0 |
| 058 | 20 | 0 | 0 | 0 |
| 059 | 19 | 1 | 0 | 0 |
| 060 | 22 | 0 | 0 | 1 |
| 061 | 19 | 0 | 0 | 0 |
| 062 | 19 | 9 | 3 | 1 |
| 063 | 18 | 0 | 1 | 1 |
| 064 | 0 | 0 | 21 | 0 |
| 065 | 0 | 0 | 17 | 2 |
| 066 | 8 | 9 | 3 | 1 |
| 067 | 0 | 0 | 19 | 1 |
| 068 | 0 | 0 | 20 | 1 |
| 069 | 0 | 9 | 9 | 0 |

## Scope corrections and retained decisions

- PRD059 S-1–S-4 and PRD061 S-1–S-4 have delivered code/tests; do not duplicate them.
- PRD059 M-11 is partial: sequential staged replacement with rollback does not
  prove multi-output atomic publication. Engineering transaction acceptance stays open.
- PRD060 S-3 is missing conditional overlay work: every supported model needs
  approved scope and lossless schema-valid round-trip evidence (F09).
- PRD062 baseline is merged; existing cross-platform API/headless CI is reusable.
  Browser coverage, full OS offline runtime evidence, security signoff, ASVS artifact,
  manual accessibility and user studies remain open alongside Should features.
- PRD063 S-3 export and S-4 epochs stay scoped; existing one-epoch code is reused.
- PRD066 both task schemas exist, but mapping preparation refuses operation and
  process invocation fails closed. Recorded response drafting is implemented.
  Preserve 2026-09-12 local-only/corpus-first/quarantine decisions.
- PRD067 M-16 uses a separate adjudicated MCP corpus and raw-document baseline;
  suggestion evaluation never substitutes for it.
- PRD068 S-2 signatures and PRD065 S-3 signed connector packages stay scoped
  through prerequisite identity/crypto decisions despite unsigned MVP boundaries.
- PRD061 owner dispositions in authoring-gates.md remain satisfied as recorded;
  partner readiness is distinct from measured GATE-PILOTS. Release stays held.

## Requirement index

Labels below are abbreviated for navigation; exact wording and all evidence
references are in the JSON ledger. Each row names its remaining completion package.

### PRD 055

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Command](../PRD/055-prd-control-mapping.md#L223) | technical-evidence | F01, F21, F22 |
| [M-2: Blocking version](../PRD/055-prd-control-mapping.md#L224) | technical-evidence | F01, F21, F22 |
| [M-3: JSON inputs](../PRD/055-prd-control-mapping.md#L225) | technical-evidence | F01, F21, F22 |
| [M-4: Offline boundary](../PRD/055-prd-control-mapping.md#L226) | technical-evidence | F01, F21, F22 |
| [M-5: Profile companion](../PRD/055-prd-control-mapping.md#L227) | technical-evidence | F01, F21, F22 |
| [M-6: Manifest contract](../PRD/055-prd-control-mapping.md#L228) | technical-evidence | F01, F21, F22 |
| [M-7: Reviewer records](../PRD/055-prd-control-mapping.md#L229) | technical-evidence | F01, F21, F22 |
| [M-8: Per-map human evidence](../PRD/055-prd-control-mapping.md#L230) | technical-evidence | F01, F21, F22 |
| [M-9: Resource validation](../PRD/055-prd-control-mapping.md#L231) | technical-evidence | F01, F21, F22 |
| [M-10: Subject inventory](../PRD/055-prd-control-mapping.md#L232) | technical-evidence | F01, F21, F22 |
| [M-11: Reference validation](../PRD/055-prd-control-mapping.md#L233) | technical-evidence | F01, F21, F22 |
| [M-12: Cardinality](../PRD/055-prd-control-mapping.md#L234) | technical-evidence | F01, F21, F22 |
| [M-13: Vocabulary](../PRD/055-prd-control-mapping.md#L235) | technical-evidence | F01, F21, F22 |
| [M-14: Relationship direction](../PRD/055-prd-control-mapping.md#L236) | technical-evidence | F01, F21, F22 |
| [M-15: No inferred claims](../PRD/055-prd-control-mapping.md#L237) | technical-evidence | F01, F21, F22 |
| [M-16: Provenance](../PRD/055-prd-control-mapping.md#L238) | technical-evidence | F01, F21, F22 |
| [M-17: Resource evidence](../PRD/055-prd-control-mapping.md#L239) | technical-evidence | F01, F21, F22 |
| [M-18: Stable UUIDs](../PRD/055-prd-control-mapping.md#L240) | technical-evidence | F01, F21, F22 |
| [M-19: Schema-valid output](../PRD/055-prd-control-mapping.md#L241) | technical-evidence | F01, F21, F22 |
| [M-20: Gap summaries](../PRD/055-prd-control-mapping.md#L242) | technical-evidence | F01, F21, F22 |
| [M-21: Deterministic report](../PRD/055-prd-control-mapping.md#L243) | technical-evidence | F01, F21, F22 |
| [M-22: Deterministic bytes](../PRD/055-prd-control-mapping.md#L244) | technical-evidence | F01, F21, F22 |
| [M-23: Change impact](../PRD/055-prd-control-mapping.md#L245) | technical-evidence | F01, F21, F22 |
| [M-24: Baseline integrity](../PRD/055-prd-control-mapping.md#L246) | technical-evidence | F01, F21, F22 |
| [M-25: Stream and write safety](../PRD/055-prd-control-mapping.md#L247) | technical-evidence | F01, F21, F22 |
| [M-26: Bounds](../PRD/055-prd-control-mapping.md#L248) | technical-evidence | F01, F21, F22 |
| [M-27: Confidence safety](../PRD/055-prd-control-mapping.md#L249) | technical-evidence | F01, F21, F22 |
| [M-28: Compatibility](../PRD/055-prd-control-mapping.md#L250) | technical-evidence | F01, F21, F22 |
| [S-1: Scaffold](../PRD/055-prd-control-mapping.md#L254) | technical-evidence | F01, F21, F22 |
| [S-2: CI check mode](../PRD/055-prd-control-mapping.md#L255) | technical-evidence | F01, F21, F22 |
| [S-3: Fail policy](../PRD/055-prd-control-mapping.md#L256) | technical-evidence | F01, F21, F22 |
| [S-4: Scope selection](../PRD/055-prd-control-mapping.md#L257) | technical-evidence | F01, F21, F22 |
| [S-5: Bounded excerpts](../PRD/055-prd-control-mapping.md#L258) | technical-evidence | F01, F21, F22 |
| [S-6: Stable machine codes](../PRD/055-prd-control-mapping.md#L259) | technical-evidence | F01, F21, F22 |

### PRD 056

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/056-prd-framework-applicability-gap-analysis.md#L168) | technical-evidence | F01, F21, F22 |
| [M-2: Closed manifest](../PRD/056-prd-framework-applicability-gap-analysis.md#L169) | technical-evidence | F01, F21, F22 |
| [M-3: Framework validation](../PRD/056-prd-framework-applicability-gap-analysis.md#L170) | technical-evidence | F01, F21, F22 |
| [M-4: Inventory](../PRD/056-prd-framework-applicability-gap-analysis.md#L171) | technical-evidence | F01, F21, F22 |
| [M-5: Decisions](../PRD/056-prd-framework-applicability-gap-analysis.md#L172) | technical-evidence | F01, F21, F22 |
| [M-6: Default](../PRD/056-prd-framework-applicability-gap-analysis.md#L173) | technical-evidence | F01, F21, F22 |
| [M-7: Mapping inputs](../PRD/056-prd-framework-applicability-gap-analysis.md#L174) | technical-evidence | F01, F21, F22 |
| [M-8: Classification](../PRD/056-prd-framework-applicability-gap-analysis.md#L175) | technical-evidence | F01, F21, F22 |
| [M-9: Terminology](../PRD/056-prd-framework-applicability-gap-analysis.md#L176) | technical-evidence | F01, F21, F22 |
| [M-10: Provenance](../PRD/056-prd-framework-applicability-gap-analysis.md#L177) | technical-evidence | F01, F21, F22 |
| [M-11: Conflict handling](../PRD/056-prd-framework-applicability-gap-analysis.md#L178) | technical-evidence | F01, F21, F22 |
| [M-12: Determinism](../PRD/056-prd-framework-applicability-gap-analysis.md#L179) | technical-evidence | F01, F21, F22 |
| [M-13: Safe I/O](../PRD/056-prd-framework-applicability-gap-analysis.md#L180) | technical-evidence | F01, F21, F22 |
| [M-14: Exit contract](../PRD/056-prd-framework-applicability-gap-analysis.md#L181) | technical-evidence | F01, F21, F22 |
| [M-15: Tests](../PRD/056-prd-framework-applicability-gap-analysis.md#L182) | technical-evidence | F01, F21, F22 |
| [S-1: Filter reports by group, control prefix, state, reviewer, or policy source without changin](../PRD/056-prd-framework-applicability-gap-analysis.md#L186) | technical-evidence | F01, F21, F22 |
| [S-2: Emit a machine-readable review queue containing stable reason codes and owner/revisit meta](../PRD/056-prd-framework-applicability-gap-analysis.md#L187) | technical-evidence | F01, F21, F22 |
| [S-3: Support an approved, explicit gate policy such as no `applicable-unmapped` or overdue `def](../PRD/056-prd-framework-applicability-gap-analysis.md#L188) | technical-evidence | F01, F21, F22 |
| [S-4: Produce a static HTML report from the same versioned report model.](../PRD/056-prd-framework-applicability-gap-analysis.md#L189) | technical-evidence | F01, F21, F22 |

### PRD 057

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Command](../PRD/057-prd-framework-change-impact-monitoring.md#L142) | technical-evidence | F01, F21, F22 |
| [M-2: Closed manifest](../PRD/057-prd-framework-change-impact-monitoring.md#L143) | technical-evidence | F01, F21, F22 |
| [M-3: Resource validation](../PRD/057-prd-framework-change-impact-monitoring.md#L144) | technical-evidence | F01, F21, F22 |
| [M-4: Canonical inventory](../PRD/057-prd-framework-change-impact-monitoring.md#L145) | technical-evidence | F01, F21, F22 |
| [M-5: Exact classification](../PRD/057-prd-framework-change-impact-monitoring.md#L146) | technical-evidence | F01, F21, F22 |
| [M-6: Migration input](../PRD/057-prd-framework-change-impact-monitoring.md#L147) | technical-evidence | F01, F21, F22 |
| [M-7: Dependency validation](../PRD/057-prd-framework-change-impact-monitoring.md#L148) | technical-evidence | F01, F21, F22 |
| [M-8: Blast radius](../PRD/057-prd-framework-change-impact-monitoring.md#L149) | technical-evidence | F01, F21, F22 |
| [M-9: Stable findings](../PRD/057-prd-framework-change-impact-monitoring.md#L150) | technical-evidence | F01, F21, F22 |
| [M-10: Priorities](../PRD/057-prd-framework-change-impact-monitoring.md#L151) | technical-evidence | F01, F21, F22 |
| [M-11: Review queue](../PRD/057-prd-framework-change-impact-monitoring.md#L152) | technical-evidence | F01, F21, F22 |
| [M-12: Non-mutation](../PRD/057-prd-framework-change-impact-monitoring.md#L153) | technical-evidence | F01, F21, F22 |
| [M-13: Determinism](../PRD/057-prd-framework-change-impact-monitoring.md#L154) | technical-evidence | F01, F21, F22 |
| [M-14: Safety](../PRD/057-prd-framework-change-impact-monitoring.md#L155) | technical-evidence | F01, F21, F22 |
| [M-15: Exit contract](../PRD/057-prd-framework-change-impact-monitoring.md#L156) | technical-evidence | F01, F21, F22 |
| [M-16: Tests](../PRD/057-prd-framework-change-impact-monitoring.md#L157) | technical-evidence | F01, F21, F22 |
| [S-1: Accept a prior impact report plus disposition file and preserve resolved, accepted-risk, a](../PRD/057-prd-framework-change-impact-monitoring.md#L161) | technical-evidence | F01, F21, F22 |
| [S-2: Produce Markdown and static HTML summaries from the same versioned report model.](../PRD/057-prd-framework-change-impact-monitoring.md#L162) | technical-evidence | F01, F21, F22 |
| [S-3: Filter by framework group, decision state, policy source, impact priority, or owner.](../PRD/057-prd-framework-change-impact-monitoring.md#L163) | technical-evidence | F01, F21, F22 |
| [S-4: Emit GitHub-compatible annotations without posting them or mutating repository state.](../PRD/057-prd-framework-change-impact-monitoring.md#L164) | technical-evidence | F01, F21, F22 |

### PRD 058

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/058-prd-policy-lifecycle-management.md#L151) | technical-evidence | F01, F21, F22 |
| [M-2: Closed schema](../PRD/058-prd-policy-lifecycle-management.md#L152) | technical-evidence | F01, F21, F22 |
| [M-3: Artifact identity](../PRD/058-prd-policy-lifecycle-management.md#L153) | technical-evidence | F01, F21, F22 |
| [M-4: State machine](../PRD/058-prd-policy-lifecycle-management.md#L154) | technical-evidence | F01, F21, F22 |
| [M-5: Transition evidence](../PRD/058-prd-policy-lifecycle-management.md#L155) | technical-evidence | F01, F21, F22 |
| [M-6: Approval policy](../PRD/058-prd-policy-lifecycle-management.md#L156) | technical-evidence | F01, F21, F22 |
| [M-7: Separation rules](../PRD/058-prd-policy-lifecycle-management.md#L157) | technical-evidence | F01, F21, F22 |
| [M-8: Drift](../PRD/058-prd-policy-lifecycle-management.md#L158) | technical-evidence | F01, F21, F22 |
| [M-9: Review schedule](../PRD/058-prd-policy-lifecycle-management.md#L159) | technical-evidence | F01, F21, F22 |
| [M-10: Reproducible time](../PRD/058-prd-policy-lifecycle-management.md#L160) | technical-evidence | F01, F21, F22 |
| [M-11: Supersession](../PRD/058-prd-policy-lifecycle-management.md#L161) | technical-evidence | F01, F21, F22 |
| [M-12: Append-only history](../PRD/058-prd-policy-lifecycle-management.md#L162) | technical-evidence | F01, F21, F22 |
| [M-13: Safe mutation](../PRD/058-prd-policy-lifecycle-management.md#L163) | technical-evidence | F01, F21, F22 |
| [M-14: Status report](../PRD/058-prd-policy-lifecycle-management.md#L164) | technical-evidence | F01, F21, F22 |
| [M-15: Exit contract](../PRD/058-prd-policy-lifecycle-management.md#L165) | technical-evidence | F01, F21, F22 |
| [M-16: Tests](../PRD/058-prd-policy-lifecycle-management.md#L166) | technical-evidence | F01, F21, F22 |
| [S-1: Portfolio status command over explicitly supplied lifecycle files.](../PRD/058-prd-policy-lifecycle-management.md#L170) | technical-evidence | F01, F21, F22 |
| [S-2: Machine-readable review queue grouped by owner and due date.](../PRD/058-prd-policy-lifecycle-management.md#L171) | technical-evidence | F01, F21, F22 |
| [S-3: Link PRD 057 impact finding IDs as reasons for re-entering review.](../PRD/058-prd-policy-lifecycle-management.md#L172) | technical-evidence | F01, F21, F22 |
| [S-4: Emit unsigned approval attestations suitable for external signing without implementing sig](../PRD/058-prd-policy-lifecycle-management.md#L173) | technical-evidence | F01, F21, F22 |

### PRD 059

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/059-prd-reusable-policy-components.md#L156) | technical-evidence | F01, F21, F22 |
| [M-2: Closed schemas](../PRD/059-prd-reusable-policy-components.md#L157) | technical-evidence | F01, F21, F22 |
| [M-3: Local containment](../PRD/059-prd-reusable-policy-components.md#L158) | technical-evidence | F01, F21, F22 |
| [M-4: Pinning](../PRD/059-prd-reusable-policy-components.md#L159) | technical-evidence | F01, F21, F22 |
| [M-5: Flat graph](../PRD/059-prd-reusable-policy-components.md#L160) | technical-evidence | F01, F21, F22 |
| [M-6: Structure](../PRD/059-prd-reusable-policy-components.md#L161) | technical-evidence | F01, F21, F22 |
| [M-7: Parameters](../PRD/059-prd-reusable-policy-components.md#L162) | technical-evidence | F01, F21, F22 |
| [M-8: Safe substitution](../PRD/059-prd-reusable-policy-components.md#L163) | technical-evidence | F01, F21, F22 |
| [M-9: Completeness](../PRD/059-prd-reusable-policy-components.md#L164) | technical-evidence | F01, F21, F22 |
| [M-10: Stable instances](../PRD/059-prd-reusable-policy-components.md#L165) | technical-evidence | F01, F21, F22 |
| [M-11: Outputs](../PRD/059-prd-reusable-policy-components.md#L166) | partial | F01, F21, F22 |
| [M-12: Provenance](../PRD/059-prd-reusable-policy-components.md#L167) | technical-evidence | F01, F21, F22 |
| [M-13: Determinism](../PRD/059-prd-reusable-policy-components.md#L168) | technical-evidence | F01, F21, F22 |
| [M-14: Validation chain](../PRD/059-prd-reusable-policy-components.md#L169) | technical-evidence | F01, F21, F22 |
| [M-15: Sensitive data](../PRD/059-prd-reusable-policy-components.md#L170) | technical-evidence | F01, F21, F22 |
| [M-16: Tests](../PRD/059-prd-reusable-policy-components.md#L171) | technical-evidence | F01, F21, F22 |
| [S-1: Build a reverse dependency index from explicitly supplied composition manifests.](../PRD/059-prd-reusable-policy-components.md#L175) | technical-evidence | F01, F21, F22 |
| [S-2: Produce a deterministic component-update impact report without modifying locks.](../PRD/059-prd-reusable-policy-components.md#L176) | technical-evidence | F01, F21, F22 |
| [S-3: Preserve component/instance provenance in generated OSCAL trace reports.](../PRD/059-prd-reusable-policy-components.md#L177) | technical-evidence | F01, F21, F22 |
| [S-4: Scaffold a component manifest from an existing Markdown section without approving it.](../PRD/059-prd-reusable-policy-components.md#L178) | technical-evidence | F01, F21, F22 |

### PRD 060

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/060-prd-evidence-implementation-linking.md#L159) | technical-evidence | F01, F21, F22 |
| [M-2: Closed manifest](../PRD/060-prd-evidence-implementation-linking.md#L160) | technical-evidence | F01, F21, F22 |
| [M-3: Artifact validation](../PRD/060-prd-evidence-implementation-linking.md#L161) | technical-evidence | F01, F21, F22 |
| [M-4: Inventories](../PRD/060-prd-evidence-implementation-linking.md#L162) | technical-evidence | F01, F21, F22 |
| [M-5: Link cardinality](../PRD/060-prd-evidence-implementation-linking.md#L163) | technical-evidence | F01, F21, F22 |
| [M-6: Review evidence](../PRD/060-prd-evidence-implementation-linking.md#L164) | technical-evidence | F01, F21, F22 |
| [M-7: Local evidence](../PRD/060-prd-evidence-implementation-linking.md#L165) | technical-evidence | F01, F21, F22 |
| [M-8: URI evidence](../PRD/060-prd-evidence-implementation-linking.md#L166) | technical-evidence | F01, F21, F22 |
| [M-9: Evidence metadata](../PRD/060-prd-evidence-implementation-linking.md#L167) | technical-evidence | F01, F21, F22 |
| [M-10: Fingerprints](../PRD/060-prd-evidence-implementation-linking.md#L168) | technical-evidence | F01, F21, F22 |
| [M-11: Freshness](../PRD/060-prd-evidence-implementation-linking.md#L169) | technical-evidence | F01, F21, F22 |
| [M-12: Missing evidence](../PRD/060-prd-evidence-implementation-linking.md#L170) | technical-evidence | F01, F21, F22 |
| [M-13: Output](../PRD/060-prd-evidence-implementation-linking.md#L171) | technical-evidence | F01, F21, F22 |
| [M-14: Baseline](../PRD/060-prd-evidence-implementation-linking.md#L172) | technical-evidence | F01, F21, F22 |
| [M-15: Stable identity](../PRD/060-prd-evidence-implementation-linking.md#L173) | technical-evidence | F01, F21, F22 |
| [M-16: No assessment claims](../PRD/060-prd-evidence-implementation-linking.md#L174) | technical-evidence | F01, F21, F22 |
| [M-17: Safety/privacy](../PRD/060-prd-evidence-implementation-linking.md#L175) | technical-evidence | F01, F21, F22 |
| [M-18: Exit contract](../PRD/060-prd-evidence-implementation-linking.md#L176) | technical-evidence | F01, F21, F22 |
| [M-19: Tests](../PRD/060-prd-evidence-implementation-linking.md#L177) | technical-evidence | F01, F21, F22 |
| [S-1: Aggregate explicitly supplied linkage projects into an owner/evidence maintenance queue.](../PRD/060-prd-evidence-implementation-linking.md#L181) | technical-evidence | F01, F21, F22 |
| [S-2: Link PRD 057 framework-impact finding IDs and PRD 058 policy versions without transferring](../PRD/060-prd-evidence-implementation-linking.md#L182) | technical-evidence | F01, F21, F22 |
| [S-3: Emit an OSCAL-compatible back-matter overlay only after schema-valid, lossless round-trip ](../PRD/060-prd-evidence-implementation-linking.md#L183) | conditional | F01, F09, F21, F22 |
| [S-4: Export a static HTML trace view from requirement through implementation to evidence metada](../PRD/060-prd-evidence-implementation-linking.md#L184) | technical-evidence | F01, F21, F22 |

### PRD 061

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/061-prd-framework-guided-policy-authoring.md#L126) | technical-evidence | F01, F21, F22 |
| [M-2: Closed inputs](../PRD/061-prd-framework-guided-policy-authoring.md#L127) | technical-evidence | F01, F21, F22 |
| [M-3: Exact baseline](../PRD/061-prd-framework-guided-policy-authoring.md#L128) | technical-evidence | F01, F21, F22 |
| [M-4: Gap accounting](../PRD/061-prd-framework-guided-policy-authoring.md#L129) | technical-evidence | F01, F21, F22 |
| [M-5: Human provenance](../PRD/061-prd-framework-guided-policy-authoring.md#L130) | technical-evidence | F01, F21, F22 |
| [M-6: Question model](../PRD/061-prd-framework-guided-policy-authoring.md#L131) | technical-evidence | F01, F21, F22 |
| [M-7: No fabricated context](../PRD/061-prd-framework-guided-policy-authoring.md#L132) | technical-evidence | F01, F21, F22 |
| [M-8: Skeleton output](../PRD/061-prd-framework-guided-policy-authoring.md#L133) | technical-evidence | F01, F21, F22 |
| [M-9: Approved content only](../PRD/061-prd-framework-guided-policy-authoring.md#L134) | technical-evidence | F01, F21, F22 |
| [M-10: Provenance](../PRD/061-prd-framework-guided-policy-authoring.md#L135) | technical-evidence | F01, F21, F22 |
| [M-11: Safe output](../PRD/061-prd-framework-guided-policy-authoring.md#L136) | technical-evidence | F01, F21, F22 |
| [M-12: Determinism](../PRD/061-prd-framework-guided-policy-authoring.md#L137) | technical-evidence | F01, F21, F22 |
| [M-13: Baseline impact](../PRD/061-prd-framework-guided-policy-authoring.md#L138) | technical-evidence | F01, F21, F22 |
| [M-14: Terminology](../PRD/061-prd-framework-guided-policy-authoring.md#L139) | technical-evidence | F01, F21, F22 |
| [M-15: Tests](../PRD/061-prd-framework-guided-policy-authoring.md#L140) | technical-evidence | F01, F21, F22 |
| [S-1: Scaffold an empty authoring pack from a valid framework inventory without creating assignm](../PRD/061-prd-framework-guided-policy-authoring.md#L144) | technical-evidence | F01, F21, F22 |
| [S-2: Generate a static HTML drafting plan and provenance view.](../PRD/061-prd-framework-guided-policy-authoring.md#L145) | technical-evidence | F01, F21, F22 |
| [S-3: Link built drafts into PRD 058 lifecycle records as `draft` without approving them.](../PRD/061-prd-framework-guided-policy-authoring.md#L146) | technical-evidence | F01, F21, F22 |
| [S-4: Allow multiple policy families to share one gap while preserving responsibility boundaries](../PRD/061-prd-framework-guided-policy-authoring.md#L147) | technical-evidence | F01, F21, F22 |

### PRD 062

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Launch and lifecycle](../PRD/062-prd-local-web-workspace.md#L779) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-2: Local unlock and session capability](../PRD/062-prd-local-web-workspace.md#L784) | technical-evidence | F01, F03, F04, F05, F21, F22 |
| [M-3: Project containment](../PRD/062-prd-local-web-workspace.md#L788) | partial | F01, F04, F05, F21, F22 |
| [M-4: Offline packaged assets](../PRD/062-prd-local-web-workspace.md#L791) | technical-evidence | F01, F04, F21, F22 |
| [M-5: Shared services](../PRD/062-prd-local-web-workspace.md#L794) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-6: Project index](../PRD/062-prd-local-web-workspace.md#L797) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-7: Project overview](../PRD/062-prd-local-web-workspace.md#L800) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-8: Policy onboarding](../PRD/062-prd-local-web-workspace.md#L803) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-9: Applicability review](../PRD/062-prd-local-web-workspace.md#L806) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-10: Mapping review](../PRD/062-prd-local-web-workspace.md#L809) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-11: Queue and analysis](../PRD/062-prd-local-web-workspace.md#L812) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-12: Provenance navigation](../PRD/062-prd-local-web-workspace.md#L815) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-13: Explicit writes](../PRD/062-prd-local-web-workspace.md#L818) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-14: Atomicity and conflicts](../PRD/062-prd-local-web-workspace.md#L821) | partial | F01, F04, F05, F21, F22 |
| [M-15: Web security](../PRD/062-prd-local-web-workspace.md#L824) | technical-evidence | F01, F03, F04, F05, F21, F22 |
| [M-16: Confidentiality](../PRD/062-prd-local-web-workspace.md#L827) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-17: Accessibility](../PRD/062-prd-local-web-workspace.md#L831) | partial | F01, F04, F05, F21, F22 |
| [M-18: Error recovery](../PRD/062-prd-local-web-workspace.md#L835) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-19: Parity and determinism](../PRD/062-prd-local-web-workspace.md#L838) | partial | F01, F04, F21, F22 |
| [M-20: Release evidence](../PRD/062-prd-local-web-workspace.md#L841) | partial | F01, F03, F04, F05, F21, F22 |
| [M-21: Normative API contract](../PRD/062-prd-local-web-workspace.md#L845) | technical-evidence | F01, F04, F21, F22 |
| [M-22: Complete browser API coverage](../PRD/062-prd-local-web-workspace.md#L848) | partial | F01, F04, F05, F21, F22 |
| [M-23: Conditional and idempotent effects](../PRD/062-prd-local-web-workspace.md#L852) | technical-evidence | F01, F04, F05, F21, F22 |
| [M-24: Queryable operations](../PRD/062-prd-local-web-workspace.md#L855) | partial | F01, F04, F05, F19, F21, F22 |
| [M-25: API compatibility](../PRD/062-prd-local-web-workspace.md#L858) | technical-evidence | F01, F04, F21, F22 |
| [M-26: Headless conformance](../PRD/062-prd-local-web-workspace.md#L862) | technical-evidence | F01, F04, F21, F22 |
| [S-1: Static reports](../PRD/062-prd-local-web-workspace.md#L869) | partial | F01, F19, F21, F22 |
| [S-2: Guided authoring](../PRD/062-prd-local-web-workspace.md#L871) | missing | F01, F19, F21, F22 |
| [S-3: Lifecycle and impact views](../PRD/062-prd-local-web-workspace.md#L873) | missing | F01, F19, F21, F22 |
| [S-4: Long operations](../PRD/062-prd-local-web-workspace.md#L875) | partial | F01, F19, F21, F22 |
| [S-5: Zero-terminal launcher](../PRD/062-prd-local-web-workspace.md#L878) | conditional | F01, F19, F22 |
| [S-6: Project bundle](../PRD/062-prd-local-web-workspace.md#L880) | missing | F01, F19, F21, F22 |

### PRD 063

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Command](../PRD/063-prd-oscal-assessment-results.md#L121) | technical-evidence | F01, F21, F22 |
| [M-2: Standards baseline](../PRD/063-prd-oscal-assessment-results.md#L122) | technical-evidence | F01, F21, F22 |
| [M-3: Closed manifest](../PRD/063-prd-oscal-assessment-results.md#L123) | technical-evidence | F01, F21, F22 |
| [M-4: Context validation](../PRD/063-prd-oscal-assessment-results.md#L124) | technical-evidence | F01, F21, F22 |
| [M-5: Subject inventory](../PRD/063-prd-oscal-assessment-results.md#L125) | technical-evidence | F01, F21, F22 |
| [M-6: Human provenance](../PRD/063-prd-oscal-assessment-results.md#L126) | technical-evidence | F01, F21, F22 |
| [M-7: Explicit relationships](../PRD/063-prd-oscal-assessment-results.md#L127) | technical-evidence | F01, F21, F22 |
| [M-8: Evidence boundary](../PRD/063-prd-oscal-assessment-results.md#L128) | technical-evidence | F01, F21, F22 |
| [M-9: Typed model](../PRD/063-prd-oscal-assessment-results.md#L129) | technical-evidence | F01, F21, F22 |
| [M-10: Stable IDs](../PRD/063-prd-oscal-assessment-results.md#L130) | technical-evidence | F01, F21, F22 |
| [M-11: Determinism](../PRD/063-prd-oscal-assessment-results.md#L131) | technical-evidence | F01, F21, F22 |
| [M-12: Baseline](../PRD/063-prd-oscal-assessment-results.md#L132) | technical-evidence | F01, F21, F22 |
| [M-13: No inferred verdicts](../PRD/063-prd-oscal-assessment-results.md#L133) | technical-evidence | F01, F21, F22 |
| [M-14: Safety/privacy](../PRD/063-prd-oscal-assessment-results.md#L134) | technical-evidence | F01, F21, F22 |
| [M-15: Exit contract](../PRD/063-prd-oscal-assessment-results.md#L135) | technical-evidence | F01, F21, F22 |
| [M-16: Tests](../PRD/063-prd-oscal-assessment-results.md#L136) | technical-evidence | F01, F21, F22 |
| [S-1: Scaffold a result manifest from an Assessment Plan without creating observations or findin](../PRD/063-prd-oscal-assessment-results.md#L140) | technical-evidence | F01, F21, F22 |
| [S-2: Static HTML assessor/reviewer report from the same versioned model.](../PRD/063-prd-oscal-assessment-results.md#L141) | technical-evidence | F01, F21, F22 |
| [S-3: Export selected reviewed risks into a PRD 064 POA&M scaffold without assigning owners or d](../PRD/063-prd-oscal-assessment-results.md#L142) | missing | F01, F08, F10, F21, F22 |
| [S-4: Support multiple result epochs when the schema and user workflow are validated.](../PRD/063-prd-oscal-assessment-results.md#L143) | conditional | F01, F10, F21, F22 |

### PRD 064

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/064-prd-oscal-poam-workflow.md#L123) | missing | F01, F07, F21, F22 |
| [M-2: Standards baseline](../PRD/064-prd-oscal-poam-workflow.md#L124) | missing | F01, F07, F21, F22 |
| [M-3: Closed manifest](../PRD/064-prd-oscal-poam-workflow.md#L125) | missing | F01, F07, F21, F22 |
| [M-4: Source validation](../PRD/064-prd-oscal-poam-workflow.md#L126) | missing | F01, F07, F21, F22 |
| [M-5: Explicit selection](../PRD/064-prd-oscal-poam-workflow.md#L127) | missing | F01, F07, F21, F22 |
| [M-6: Stable item identity](../PRD/064-prd-oscal-poam-workflow.md#L128) | missing | F01, F07, F21, F22 |
| [M-7: Ownership](../PRD/064-prd-oscal-poam-workflow.md#L129) | missing | F01, F08, F21, F22 |
| [M-8: Milestones](../PRD/064-prd-oscal-poam-workflow.md#L130) | missing | F01, F08, F21, F22 |
| [M-9: Status history](../PRD/064-prd-oscal-poam-workflow.md#L131) | missing | F01, F08, F21, F22 |
| [M-10: Closure boundary](../PRD/064-prd-oscal-poam-workflow.md#L132) | missing | F01, F08, F21, F22 |
| [M-11: Schedule report](../PRD/064-prd-oscal-poam-workflow.md#L133) | missing | F01, F08, F21, F22 |
| [M-12: Typed/schema output](../PRD/064-prd-oscal-poam-workflow.md#L134) | missing | F01, F07, F08, F21, F22 |
| [M-13: Baseline](../PRD/064-prd-oscal-poam-workflow.md#L135) | missing | F01, F08, F21, F22 |
| [M-14: Non-mutation](../PRD/064-prd-oscal-poam-workflow.md#L136) | missing | F01, F08, F21, F22 |
| [M-15: Safety/privacy](../PRD/064-prd-oscal-poam-workflow.md#L137) | missing | F01, F08, F21, F22 |
| [M-16: Exit contract](../PRD/064-prd-oscal-poam-workflow.md#L138) | missing | F01, F08, F21, F22 |
| [M-17: Tests](../PRD/064-prd-oscal-poam-workflow.md#L139) | missing | F01, F07, F08, F21, F22 |
| [S-1: Portfolio report across explicitly supplied POA&M artifacts.](../PRD/064-prd-oscal-poam-workflow.md#L143) | missing | F01, F08, F21, F22 |
| [S-2: Static HTML timeline and source-to-remediation trace view.](../PRD/064-prd-oscal-poam-workflow.md#L144) | missing | F01, F08, F21, F22 |
| [S-3: Connector-neutral outbound change set for PRD 065 without performing external mutations.](../PRD/064-prd-oscal-poam-workflow.md#L145) | missing | F01, F08, F14, F21, F22 |
| [S-4: Link fresh PRD 060 evidence references to completion assertions without verifying them.](../PRD/064-prd-oscal-poam-workflow.md#L146) | missing | F01, F08, F21, F22 |

### PRD 065

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/065-prd-external-workflow-integrations.md#L123) | missing | F01, F14, F21, F22 |
| [M-2: Change-set schema](../PRD/065-prd-external-workflow-integrations.md#L124) | missing | F01, F14, F21, F22 |
| [M-3: Source validation](../PRD/065-prd-external-workflow-integrations.md#L125) | missing | F01, F14, F21, F22 |
| [M-4: Reference adapter](../PRD/065-prd-external-workflow-integrations.md#L126) | missing | F01, F14, F21, F22 |
| [M-5: Credential boundary](../PRD/065-prd-external-workflow-integrations.md#L127) | missing | F01, F14, F21, F22 |
| [M-6: Dry-run gate](../PRD/065-prd-external-workflow-integrations.md#L128) | missing | F01, F14, F21, F22 |
| [M-7: Explicit confirmation](../PRD/065-prd-external-workflow-integrations.md#L129) | missing | F01, F14, F21, F22 |
| [M-8: Idempotency](../PRD/065-prd-external-workflow-integrations.md#L130) | missing | F01, F14, F21, F22 |
| [M-9: Conflict handling](../PRD/065-prd-external-workflow-integrations.md#L131) | missing | F01, F14, F21, F22 |
| [M-10: Partial failure](../PRD/065-prd-external-workflow-integrations.md#L132) | missing | F01, F14, F21, F22 |
| [M-11: Authority boundary](../PRD/065-prd-external-workflow-integrations.md#L133) | missing | F01, F14, F21, F22 |
| [M-12: Data minimization](../PRD/065-prd-external-workflow-integrations.md#L134) | missing | F01, F14, F21, F22 |
| [M-13: Network safety](../PRD/065-prd-external-workflow-integrations.md#L135) | missing | F01, F14, F21, F22 |
| [M-14: Audit report](../PRD/065-prd-external-workflow-integrations.md#L136) | missing | F01, F14, F21, F22 |
| [M-15: Tests](../PRD/065-prd-external-workflow-integrations.md#L137) | missing | F01, F14, F21, F22 |
| [S-1: GitLab Issues adapter using the same contract tests.](../PRD/065-prd-external-workflow-integrations.md#L141) | missing | F01, F15, F21, F22 |
| [S-2: Jira Cloud adapter after auth/data-residency review.](../PRD/065-prd-external-workflow-integrations.md#L142) | conditional | F01, F15, F21, F22 |
| [S-3: Signed connector packages with explicit permissions and version pinning.](../PRD/065-prd-external-workflow-integrations.md#L143) | conditional | F01, F15, F21, F22 |
| [S-4: Import remote comments/status as non-authoritative review observations.](../PRD/065-prd-external-workflow-integrations.md#L144) | missing | F01, F15, F21, F22 |

### PRD 066

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/066-prd-ai-assisted-suggestions.md#L123) | partial | F01, F06, F16, F17, F21, F22 |
| [M-2: Task scope](../PRD/066-prd-ai-assisted-suggestions.md#L124) | partial | F01, F06, F17, F21, F22 |
| [M-3: Context allowlist](../PRD/066-prd-ai-assisted-suggestions.md#L125) | technical-evidence | F01, F06, F17, F21, F22 |
| [M-4: Payload preview](../PRD/066-prd-ai-assisted-suggestions.md#L126) | technical-evidence | F01, F06, F16, F17, F21, F22 |
| [M-5: Explicit consent](../PRD/066-prd-ai-assisted-suggestions.md#L127) | technical-evidence | F01, F06, F16, F17, F21, F22 |
| [M-6: Provider boundary](../PRD/066-prd-ai-assisted-suggestions.md#L128) | partial | F01, F06, F16, F17, F21, F22 |
| [M-7: Structured output](../PRD/066-prd-ai-assisted-suggestions.md#L129) | technical-evidence | F01, F06, F17, F21, F22 |
| [M-8: Citation validation](../PRD/066-prd-ai-assisted-suggestions.md#L130) | technical-evidence | F01, F06, F17, F21, F22 |
| [M-9: Assumptions](../PRD/066-prd-ai-assisted-suggestions.md#L131) | partial | F01, F06, F17, F21, F22 |
| [M-10: Quarantine](../PRD/066-prd-ai-assisted-suggestions.md#L132) | technical-evidence | F01, F06, F17, F21, F22 |
| [M-11: Disposition](../PRD/066-prd-ai-assisted-suggestions.md#L133) | technical-evidence | F01, F06, F17, F21, F22 |
| [M-12: Promotion](../PRD/066-prd-ai-assisted-suggestions.md#L134) | partial | F01, F06, F17, F21, F22 |
| [M-13: Provenance](../PRD/066-prd-ai-assisted-suggestions.md#L135) | partial | F01, F06, F16, F17, F21, F22 |
| [M-14: Evaluation gate](../PRD/066-prd-ai-assisted-suggestions.md#L136) | missing | F01, F06, F16, F17, F21, F22 |
| [M-15: Regression](../PRD/066-prd-ai-assisted-suggestions.md#L137) | missing | F01, F06, F16, F17, F21, F22 |
| [M-16: No telemetry/training consent](../PRD/066-prd-ai-assisted-suggestions.md#L138) | technical-evidence | F01, F06, F16, F17, F21, F22 |
| [M-17: Tests](../PRD/066-prd-ai-assisted-suggestions.md#L139) | partial | F01, F06, F16, F17, F21, F22 |
| [S-1: Approved local-model adapter with identical quarantine and evaluation contracts.](../PRD/066-prd-ai-assisted-suggestions.md#L143) | conditional | F01, F06, F16, F17, F21, F22 |
| [S-2: Batch review ordering by deterministic evidence support and impact, never raw model confid](../PRD/066-prd-ai-assisted-suggestions.md#L144) | partial | F01, F18, F21, F22 |
| [S-3: Side-by-side source/candidate/diff review in PRD 062.](../PRD/066-prd-ai-assisted-suggestions.md#L145) | missing | F01, F17, F18, F21, F22 |
| [S-4: Re-evaluate expired suggestions after input changes without carrying forward approval.](../PRD/066-prd-ai-assisted-suggestions.md#L146) | partial | F01, F17, F18, F21, F22 |

### PRD 067

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Command/transport](../PRD/067-prd-read-only-mcp-governance-interface.md#L133) | missing | F01, F11, F21, F22 |
| [M-2: Project containment](../PRD/067-prd-read-only-mcp-governance-interface.md#L134) | missing | F01, F11, F21, F22 |
| [M-3: Read-only capability](../PRD/067-prd-read-only-mcp-governance-interface.md#L135) | missing | F01, F11, F21, F22 |
| [M-4: Shared core](../PRD/067-prd-read-only-mcp-governance-interface.md#L136) | missing | F01, F11, F21, F22 |
| [M-5: Validation/freshness](../PRD/067-prd-read-only-mcp-governance-interface.md#L137) | missing | F01, F11, F21, F22 |
| [M-6: Lifecycle visibility](../PRD/067-prd-read-only-mcp-governance-interface.md#L138) | missing | F01, F11, F21, F22 |
| [M-7: Structured tools](../PRD/067-prd-read-only-mcp-governance-interface.md#L139) | missing | F01, F11, F21, F22 |
| [M-8: Grounding](../PRD/067-prd-read-only-mcp-governance-interface.md#L140) | missing | F01, F11, F21, F22 |
| [M-9: Search semantics](../PRD/067-prd-read-only-mcp-governance-interface.md#L141) | missing | F01, F11, F21, F22 |
| [M-10: Prompt-injection boundary](../PRD/067-prd-read-only-mcp-governance-interface.md#L142) | missing | F01, F11, F21, F22 |
| [M-11: Bounds](../PRD/067-prd-read-only-mcp-governance-interface.md#L143) | missing | F01, F11, F21, F22 |
| [M-12: Privacy](../PRD/067-prd-read-only-mcp-governance-interface.md#L144) | missing | F01, F11, F21, F22 |
| [M-13: No network/process](../PRD/067-prd-read-only-mcp-governance-interface.md#L145) | missing | F01, F11, F21, F22 |
| [M-14: Session behavior](../PRD/067-prd-read-only-mcp-governance-interface.md#L146) | missing | F01, F11, F21, F22 |
| [M-15: Protocol/security tests](../PRD/067-prd-read-only-mcp-governance-interface.md#L147) | missing | F01, F11, F21, F22 |
| [M-16: Evaluation](../PRD/067-prd-read-only-mcp-governance-interface.md#L148) | missing | F01, F11, F21, F22 |
| [S-1: Read-only implementation/evidence metadata trace from PRD 060 with no evidence excerpts.](../PRD/067-prd-read-only-mcp-governance-interface.md#L152) | missing | F01, F20, F21, F22 |
| [S-2: Client-facing resource links to static PRD 062 reports.](../PRD/067-prd-read-only-mcp-governance-interface.md#L153) | missing | F01, F20, F21, F22 |
| [S-3: Configurable policy visibility profiles signed or hash-pinned by project owners.](../PRD/067-prd-read-only-mcp-governance-interface.md#L154) | conditional | F01, F20, F21, F22 |
| [S-4: Optional local full-text index whose bytes remain under the project root and rebuild deter](../PRD/067-prd-read-only-mcp-governance-interface.md#L155) | missing | F01, F20, F21, F22 |

### PRD 068

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Commands](../PRD/068-prd-collaborative-review-queues.md#L126) | missing | F01, F12, F21, F22 |
| [M-2: Closed schemas](../PRD/068-prd-collaborative-review-queues.md#L127) | missing | F01, F12, F21, F22 |
| [M-3: Domain adapters](../PRD/068-prd-collaborative-review-queues.md#L128) | missing | F01, F12, F13, F21, F22 |
| [M-4: Snapshot integrity](../PRD/068-prd-collaborative-review-queues.md#L129) | missing | F01, F12, F21, F22 |
| [M-5: Assignment policy](../PRD/068-prd-collaborative-review-queues.md#L130) | missing | F01, F12, F21, F22 |
| [M-6: Response evidence](../PRD/068-prd-collaborative-review-queues.md#L131) | missing | F01, F12, F21, F22 |
| [M-7: Staleness](../PRD/068-prd-collaborative-review-queues.md#L132) | missing | F01, F12, F21, F22 |
| [M-8: Append-only merge](../PRD/068-prd-collaborative-review-queues.md#L133) | missing | F01, F12, F21, F22 |
| [M-9: Conflict/quorum](../PRD/068-prd-collaborative-review-queues.md#L134) | missing | F01, F12, F21, F22 |
| [M-10: Proposed edits](../PRD/068-prd-collaborative-review-queues.md#L135) | missing | F01, F12, F13, F21, F22 |
| [M-11: Disposition bundle](../PRD/068-prd-collaborative-review-queues.md#L136) | missing | F01, F12, F21, F22 |
| [M-12: Identity disclaimer](../PRD/068-prd-collaborative-review-queues.md#L137) | missing | F01, F12, F21, F22 |
| [M-13: Privacy](../PRD/068-prd-collaborative-review-queues.md#L138) | missing | F01, F12, F21, F22 |
| [M-14: Determinism](../PRD/068-prd-collaborative-review-queues.md#L139) | missing | F01, F12, F21, F22 |
| [M-15: Safe I/O](../PRD/068-prd-collaborative-review-queues.md#L140) | missing | F01, F12, F21, F22 |
| [M-16: Static HTML](../PRD/068-prd-collaborative-review-queues.md#L141) | missing | F01, F12, F21, F22 |
| [M-17: Tests](../PRD/068-prd-collaborative-review-queues.md#L142) | missing | F01, F12, F21, F22 |
| [S-1: PRD 062 interactive local client that writes the same response files.](../PRD/068-prd-collaborative-review-queues.md#L146) | missing | F01, F13, F21, F22 |
| [S-2: Signed review response envelope after identity/cryptography design.](../PRD/068-prd-collaborative-review-queues.md#L147) | conditional | F01, F13, F21, F22 |
| [S-3: Connector-neutral queue notification export for PRD 065 without sending it.](../PRD/068-prd-collaborative-review-queues.md#L148) | missing | F01, F13, F14, F21, F22 |
| [S-4: Queue supersession linking old/new item IDs after upstream change impact.](../PRD/068-prd-collaborative-review-queues.md#L149) | missing | F01, F13, F21, F22 |

### PRD 069

| Requirement | Evidence state | Remaining packages |
|---|---|---|
| [M-1: Inventory](../PRD/069-prd-dependency-security-audit.md#L148) | missing | F02 |
| [M-2: Criteria](../PRD/069-prd-dependency-security-audit.md#L149) | partial | F02 |
| [M-3: Runtime stack first](../PRD/069-prd-dependency-security-audit.md#L150) | missing | F03 |
| [M-4: Audits recorded](../PRD/069-prd-dependency-security-audit.md#L151) | missing | F03 |
| [M-5: Exceptions owned](../PRD/069-prd-dependency-security-audit.md#L152) | missing | F02, F03 |
| [M-6: Gate](../PRD/069-prd-dependency-security-audit.md#L153) | partial | F02 |
| [M-7: Differential workflow](../PRD/069-prd-dependency-security-audit.md#L154) | partial | F03 |
| [M-8: Imports pinned](../PRD/069-prd-dependency-security-audit.md#L155) | partial | F02, F03 |
| [M-9: Stale reporting](../PRD/069-prd-dependency-security-audit.md#L156) | missing | F02 |
| [M-10: Offline](../PRD/069-prd-dependency-security-audit.md#L157) | partial | F02 |
| [M-11: Provenance cross-check](../PRD/069-prd-dependency-security-audit.md#L158) | partial | F22 |
| [M-12: Exit contract](../PRD/069-prd-dependency-security-audit.md#L159) | missing | F02 |
| [M-13: Tests](../PRD/069-prd-dependency-security-audit.md#L160) | missing | F02, F03 |
| [M-14: Documentation](../PRD/069-prd-dependency-security-audit.md#L161) | partial | F02, F03 |
| [S-1: Reduce the exempted surface by identifying unused or duplicated transitive crates.](../PRD/069-prd-dependency-security-audit.md#L165) | partial | F22 |
| [S-2: Machine-readable trend report (audited vs excepted counts and median age) recorded per rel](../PRD/069-prd-dependency-security-audit.md#L166) | missing | F22 |
| [S-3: Automated reminder when a dependency bump introduces a new unvetted version.](../PRD/069-prd-dependency-security-audit.md#L167) | partial | F02, F22 |
| [S-4: Publish the inventory with release artifacts so consumers can see the reviewed set.](../PRD/069-prd-dependency-security-audit.md#L168) | missing | F22 |

## Local verification

With installed Rust 1.98.1 and matching Clippy/rustfmt, fmt and strict all-target
Clippy passed; the full suite passed 2,692 tests with 3 ignored, and the dedicated
API rerun passed 8. Audit passed with the allowed ttf-parser unmaintained warning
RUSTSEC-2026-0192; deny passed with duplicate warnings. Cargo-vet failed with the
same 20 unvetted dependencies as hosted main. Benchmarks were not run.

Installed Rust 1.99.0 failed strict Clippy on unchanged source with new lints.
This current-stable/toolchain-baseline gate stays open under F22. The 1.98.1
compatibility result is not a green full-CI or release-readiness claim.

The offline ledger check and seven negative cases passed: removing a requirement,
duplicating an ID, assigning an unknown package, changing a PRD digest, inventing
acceptance, clearing named gates and drifting Markdown state all fail closed.

## Review and continuation

Run the offline scope/evidence check:

```sh
rtk proxy python3 scripts/check_roadmap_ledger.py
```

F01 produces reconciliation and decision packets, not owner acceptance. Once
the draft is reviewable, move its task to In Review with the exact PR/head.
F02/F03 policy acceptance waits for the requested D069 disposition; implementation
can be prepared as a proposal without inventing audits or exceptions. F06 can
prepare lawful-input forms/evaluator support while real adjudication remains pending.
F07 prepares a schema/typed-model spike before freezing its public contract.
Create subsequent F08–F22 tasks as their actual scope becomes actionable.

Only the coordinating owner stages/commits/pushes shared contract changes. Keep
large Cargo jobs sequential. Merge, participant outreach and publication require
separate authorization; prepared packets and green tests do not close those gates.

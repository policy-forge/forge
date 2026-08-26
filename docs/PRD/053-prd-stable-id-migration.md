# 053-prd-stable-id-migration

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Implementation in progress — successor declarations added
> **Last Updated:** 2026-08-25 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `053-stable-id-migration`
**Created**: 2026-08-22
**Status**: Implementation in progress — successor declarations added
**Input**: FORGE v1.2 roadmap priority 3

---

## Executive Summary 🟡 `@human-review`

FORGE already generates deterministic UUID v5 requirement identifiers and can warn when `--stable-id-baseline` finds an ID change at the same section path, source line, and atom index. That warning is intentionally narrow: it produces only a count, misses requirements that move, and cannot explain additions, retirements, reordering, atomization changes, or successor relationships.

This PRD introduces `forge migrate OLD_POLICY NEW_POLICY`, a read-only policy-to-policy analysis command. It inventories both policies through the same ingestion, parsing, atomization, and stable-ID pipeline used by conversion; classifies unchanged, added, retired, relocated/reindexed, substantively changed, split/merged, ambiguous, and unmatched requirements; and emits deterministic human-readable text or versioned JSON. FORGE may report observed ID transitions and candidate relationships, but it must never silently rewrite generated IDs or present an inferred relationship as human-approved. Explicit successor, split, or merge relationships are accepted only through an optional reviewer-authored mapping file and are preserved with approval metadata.

---

## Context

### Background 🔴 `@human-required`

Stable identifiers are central to FORGE's deterministic and auditable value proposition. The current generator assigns each atomized requirement a UUID v5 derived from normalized requirement text plus section path, source line, and atom index. This avoids collisions between identical requirements at different locations, but it also means that moving a requirement, renaming its section, inserting lines before it, or changing how a compound statement is atomized can rotate the generated UUID even when some or all prose remains unchanged.

`forge convert --stable-id-baseline OLD_POLICY NEW_POLICY` currently compares requirements only when their section path, source line, and atom index are identical, then warns with a count if their stable IDs differ. `forge diff`, by contrast, compares generated OSCAL artifacts using `control-id` and reports artifact-level added, removed, changed, and UUID-only changes. Neither command provides an auditor-ready migration account from one source-policy revision to the next.

### Problem Statement 🔴 `@human-required`

Compliance engineers and auditors need to explain how policy requirements evolved across versions without manually reconciling source documents and generated OSCAL files. Today, FORGE cannot distinguish an unchanged requirement whose generated ID rotated because it moved from a truly rewritten requirement, cannot represent one-to-many atomization changes, and cannot preserve a reviewer-approved old-ID to new-ID successor decision. The result is manual analysis, noisy downstream diffs, and a risk that inferred relationships are treated as authoritative without evidence.

### Target Users 🟡 `@human-review`

- **Primary:** Compliance engineers maintaining policy-as-code repositories and generated OSCAL artifacts.
- **Primary:** Auditors reviewing policy evolution, control retirement, and traceability across assessment periods.
- **Secondary:** Platform and security engineers enforcing policy migration checks in CI.
- **Secondary:** GRC integrators consuming stable-ID relationships in downstream workflows.

### Scope Boundaries 🟡 `@human-review`

**In Scope:**

- A read-only `forge migrate OLD_POLICY NEW_POLICY` command for Markdown, PDF, and DOCX policy inputs supported by the existing ingestion pipeline.
- Requirement inventories produced by the current shared pipeline through atomization and stable-ID assignment.
- Classification of unchanged, added, retired, observed deterministic ID changes, substantive-change candidates, declared successors, declared splits/merges, atomization-change candidates, and ambiguous/unmatched requirements.
- Old and new stable IDs, normalized-content fingerprints, source locations, section paths, atom indexes, and provenance limitations needed to audit every classification.
- An optional, reviewer-authored successor mapping file that records explicit one-to-one, one-to-many, and many-to-one relationships with reviewer and rationale metadata.
- Deterministic human-readable text and versioned machine-readable JSON.
- Exit-code behavior compatible with `forge diff` and CI usage.
- Clear treatment of source reordering, section renaming, line shifts, duplicate prose, and atomization changes.

**Out of Scope:**

- Reassigning, overriding, persisting, or mutating stable IDs in either input policy or generated OSCAL artifacts.
- Automatically approving successor, replacement, split, or merge relationships.
- Semantic or AI-based claims that two differently worded requirements have equivalent meaning.
- Applying migrations to SSPs, Assessment Plans, Assessment Results, POA&Ms, Profiles, or Control Mapping artifacts.
- Three-way merge, policy editing, or conflict resolution.
- A general migration database, GRC connector, web UI, or hosted service.

### Glossary 🟡 `@human-review`

| Term | Definition |
|------|------------|
| Stable ID | UUID v5 assigned to an atomized `PolicyRequirement`; copied to the UUID of a generated Catalog control. |
| Requirement Locator | Section path, 1-based normalized source line, and 0-based atom index used to locate an atomized requirement. |
| Observed ID Change | A factual report that old and new generated IDs differ while normalized requirement text matches uniquely; it is not a successor approval. |
| Candidate Relationship | A deterministic, evidence-backed possible relationship that still requires human review. |
| Declared Successor | A relationship supplied in a reviewer-authored mapping file and preserved as explicit human input; FORGE validates references but does not authenticate the reviewer. |
| Retirement | An old requirement with no remaining exact, declared, observed, candidate, or ambiguous relationship to the new inventory. |
| Addition | A new requirement with no remaining exact, declared, observed, candidate, or ambiguous relationship to the old inventory. |
| Atomization Change | One source statement becoming multiple atomic requirements, multiple requirements becoming one, or atom boundaries changing between versions. |
| Source Location | A file label, section path/title, normalized line number, and atom index. For PDF/DOCX, the line number refers to extracted normalized text, not a native page or paragraph coordinate. |
| Analysis Error | A condition that prevents FORGE from producing a complete, trustworthy report, distinct from a completed report that contains changes or ambiguities. |

### Related Documents ⚪ `@auto`

| Document | Link | Relationship |
|----------|------|--------------|
| Product Roadmap | `docs/FORGE_PRODUCT_ROADMAP.md` | v1.2 product context |
| Product Vision | `docs/FORGE_PRODUCT_VISION.md` | Determinism, auditability, and traceability principles |
| UUID Generation PRD | `docs/PRD/007-prd-uuid-generation.md` | Original stable-ID requirements |
| Golden-File Edge Cases PRD | `docs/PRD/022-prd-golden-file-edge-cases.md` | Whitespace and substantive-change fixtures |
| Diff Report PRD | `docs/PRD/043-prd-diff-report.md` | Existing artifact-to-artifact comparison |
| Traceability Model PRD | `docs/PRD/016-prd-traceability-model.md` | Source-location and trace-link concepts |
| Traceability Embedding PRD | `docs/PRD/017-prd-traceability-embedding.md` | Provenance embedded in OSCAL output |

---

## Goals 🔴 `@human-required`

- **G-1 — Complete migration inventory:** Classify 100% of old and new atomized requirements into exactly one top-level outcome, or place them in an explicit ambiguity group; never silently drop an ID.
- **G-2 — Audit-safe identity handling:** Produce zero automatically approved successor relationships and zero mutations of input or generated stable IDs in all tests and pilot runs.
- **G-3 — Deterministic automation:** Produce byte-for-byte identical JSON and text for identical input bytes, options, successor map, and FORGE version, excluding no hidden clock or environment fields.
- **G-4 — Actionable CI signal:** Give CI a documented `0`/`1`/`2` contract that distinguishes no migration impact, reviewable changes, and analysis failure.
- **G-5 — Reduce manual reconciliation:** As a launch hypothesis, reduce median time for a practitioner to reconcile a representative policy revision by at least 50% compared with manual source/OSCAL comparison, without lowering reviewer accuracy.

## Non-Goals 🔴 `@human-required`

- **NG-1 — Automatic ID preservation:** FORGE will not force a new requirement to retain an old generated UUID; doing so would violate the current deterministic generation contract.
- **NG-2 — Semantic equivalence engine:** Differently worded requirements will not be declared equivalent by lexical similarity, embeddings, or an LLM in the MVP; such output would be too easy to over-trust.
- **NG-3 — Migration application:** The command will not edit policies, OSCAL artifacts, baselines, or successor maps. It reports evidence and validates declarations only.
- **NG-4 — Replacement for `forge diff`:** `forge migrate` compares source-policy revisions and requirement identities; `forge diff` remains the OSCAL artifact comparison command.
- **NG-5 — Native PDF/DOCX coordinates:** The MVP will not claim PDF page, bounding-box, Word paragraph, revision-mark, comment, or tracked-change provenance that the current extractors do not preserve.
- **NG-6 — Cross-policy framework mapping:** Mapping controls between NIST, ISO, CIS, or other frameworks belongs to the Control Mapping initiative, not stable-ID migration.

---

## User Stories & Priorities 🔴 `@human-required`

### US-1 — Explain policy evolution (P0)

> As a compliance engineer, I want a single report of unchanged, added, retired, moved, and substantively changed requirements so that I can review policy evolution without manually correlating two documents.

### US-2 — Preserve audit truth about relationships (P0)

> As an auditor, I want observed ID changes, inferred candidates, and explicit reviewer-approved successors labeled differently so that I do not mistake a heuristic for an authoritative migration decision.

### US-3 — Detect reordering and atomization effects (P0)

> As a policy maintainer, I want FORGE to show when a stable ID changed because a requirement moved or its compound statement was re-atomized so that I can separate structural churn from prose changes.

### US-4 — Enforce migration review in CI (P0)

> As a platform engineer, I want machine-readable output and predictable exit codes so that policy pull requests can require review when identities or relationships change.

### US-5 — Retain useful source provenance (P1)

> As an auditor, I want old and new source locations and provenance quality in each relationship so that I can verify the reported migration against the source documents.

### US-6 — Review unresolved cases (P1)

> As a compliance engineer, I want duplicate and uncertain matches grouped with their candidate IDs and evidence so that I can resolve them without FORGE choosing arbitrarily.

---

## Classification Model 🟡 `@human-review`

Each old and new requirement must participate in exactly one top-level classification. Matching is deterministic and proceeds in the following precedence order:

1. **Exact stable ID:** Same generated ID on both sides. Classify as `unchanged` unless source-file provenance changed, in which case attach a `source_location_changed` detail without inventing a new identity relationship. If normalized text differs for the same generated ID, fail the analysis as an integrity anomaly.
2. **Declared relationship:** Apply a valid reviewer-authored `successor`, `split`, or `merge` declaration to still-unmatched IDs. Declarations take precedence over automatic candidates but never change either side's generated ID.
3. **Unique exact normalized text:** If one unmatched old requirement and one unmatched new requirement have identical normalized text, classify as `observed_id_change`. Report which stable-ID seed fields changed: section path, normalized source line, or atom index. This is observed deterministic rotation, not approval of a successor.
4. **Unique same locator, changed text:** If one old and one new requirement occupy the same section path, normalized source line, and atom index but their normalized text differs, classify as `substantive_change_candidate`. A locator match is evidence, not proof of logical succession.
5. **Atomization lineage:** Use source anchor, `parent_text`, atom index, and normalized atom text to group possible one-to-many or many-to-one changes. Classify only uniquely attributable groups as `atomization_change_candidate`; otherwise classify the group as `ambiguous`.
6. **Ambiguity:** If duplicate normalized prose, duplicated locators, or competing candidates prevent a unique classification, list all involved old/new IDs and the evidence that caused ambiguity. Never resolve by collection order.
7. **Unmatched remainder:** Classify remaining old IDs as `retired` and remaining new IDs as `added`.

The MVP does not use a fuzzy similarity threshold to establish relationships. A future version may surface lexical similarity as additional evidence, but it must remain a candidate until explicitly approved.

### Relationship and Approval Semantics 🟡 `@human-review`

| Classification | Generated by FORGE | Human-approved | May be consumed as an authoritative successor |
|----------------|--------------------|----------------|-----------------------------------------------|
| `unchanged` | Yes, exact evidence | Not required | Yes, identity is unchanged |
| `observed_id_change` | Yes, exact normalized text | No | No |
| `substantive_change_candidate` | Yes, locator evidence | No | No |
| `atomization_change_candidate` | Yes, lineage evidence | No | No |
| `ambiguous` | Yes | No | No |
| `declared_successor` / `declared_split` / `declared_merge` | No; supplied by mapping file | Declared by reviewer | Yes, subject to consumer trust policy |
| `retired` / `added` | Yes, after matching | Not required | N/A |

FORGE validates that declared IDs exist on the correct side and that declarations do not conflict. It records `approved_by`, `approved_at`, and `rationale` verbatim but does not authenticate the identity or authority of the reviewer. Documentation and JSON must call these relationships **declared**, not **verified**.

---

## Requirements

### Must Have (M) — MVP launch blockers 🔴 `@human-required`

- [ ] **M-1 — Command:** The CLI shall provide `forge migrate <OLD_POLICY> <NEW_POLICY>` and accept each supported policy input format (`.md`, `.markdown`, `.pdf`, `.docx`).
- [ ] **M-2 — Shared pipeline:** The command shall use the same ingestion, extraction, atomization, normalization, and stable-ID assignment implementation as `forge convert`; it shall not implement a second ID algorithm.
- [ ] **M-3 — Complete inventory:** Every old and new stable ID shall appear in exactly one top-level classification or ambiguity group, and summary counts shall reconcile to both inventories.
- [ ] **M-4 — Core classes:** The report shall distinguish `unchanged`, `observed_id_change`, `substantive_change_candidate`, `atomization_change_candidate`, `ambiguous`, `retired`, and `added` outcomes.
- [ ] **M-5 — ID evidence:** Every paired or grouped entry shall include old/new stable IDs, normalized-content SHA-256 fingerprints, source locations, section paths, atom indexes, classification evidence, and confidence basis.
- [ ] **M-6 — No silent mutation:** The command shall be read-only with respect to old/new policies, successor maps, and generated artifacts; it shall never override, reassign, or write stable IDs.
- [ ] **M-7 — Qualified inference:** Automatically inferred relationships shall be labeled `observed` or `candidate` and shall never be serialized as approved successors.
- [x] **M-8 — Successor declarations:** An optional `--successor-map <FILE>` shall accept versioned JSON declarations for one-to-one successor, one-to-many split, and many-to-one merge relationships. Each declaration shall require non-empty `approved_by`, `approved_at`, and `rationale` fields.
- [x] **M-9 — Mapping validation:** FORGE shall reject a successor map that references absent IDs, reuses an ID in conflicting declarations, maps an ID to itself, uses an unsupported relationship type/schema version, or is malformed/oversized. Rejection shall not suppress the error or modify inputs.
- [ ] **M-10 — Reordering:** Unique identical normalized prose with a changed stable ID shall be reported as an observed ID change with all changed seed fields, including line shifts, section moves/renames, and atom-index changes.
- [ ] **M-11 — Atomization:** One-to-many, many-to-one, and changed-boundary cases shall be represented as grouped candidate or declared relationships; FORGE shall not flatten them into arbitrary one-to-one pairs.
- [ ] **M-12 — Ambiguity:** Duplicate prose or competing matches shall produce a deterministic ambiguity group containing all candidate IDs and evidence; no candidate shall be selected by traversal or hash-map iteration order.
- [ ] **M-13 — Source-location changes:** Every paired/grouped entry shall show old and new source location and enumerate changes to file label, section path/title, normalized line, and atom index.
- [ ] **M-14 — PDF/DOCX provenance:** PDF and DOCX report locations shall be labeled `normalized_extracted_text_line`; reports and help text shall state that they are not native PDF page or Word paragraph coordinates. Raw-file SHA-256 and input format shall be included for both inputs.
- [ ] **M-15 — Extraction limitations:** Scanned PDFs with no extractable text, encrypted/unreadable documents, unsupported DOCX structure, and extraction failures shall stop analysis with a descriptive error rather than produce a partial migration report.
- [ ] **M-16 — Text output:** Human-readable text shall be the default and shall contain source fingerprints, summary counts, grouped classifications, relationship evidence, approval status, and ambiguity/unmatched guidance.
- [ ] **M-17 — JSON output:** `--format json` shall emit a documented, versioned JSON contract suitable for CI and downstream audit tooling. Data shall be serialized by a JSON serializer, not string concatenation.
- [ ] **M-18 — Determinism:** Both formats shall use a documented category order and stable ordering within categories by old stable ID, then new stable ID. Reports shall omit wall-clock generation timestamps, randomized identifiers, canonical absolute paths, and environment-dependent values.
- [ ] **M-19 — Stream separation:** Reports shall go to stdout or `--output`; warnings and errors shall go to stderr. JSON stdout shall never be mixed with progress, warning, or logging text.
- [ ] **M-20 — CI exit codes:** Exit `0` when analysis completes with only unchanged requirements and no source-location change; exit `1` when analysis completes and contains any added, retired, observed ID change, candidate, declared relationship, ambiguity, or source-location change; exit `2` when a trustworthy complete report cannot be produced, including invalid input or successor-map errors. The report shall be written before returning exit `1`.
- [ ] **M-21 — Integrity anomaly:** If the same generated stable ID is associated with different normalized text within or across inventories, analysis shall fail with exit `2` and identify the affected ID without claiming a migration relationship.
- [ ] **M-22 — Input safety:** Apply existing file-type, regular-file, maximum-size, and recursion/split limits independently to both policies; apply a documented size limit to the successor map.
- [x] **M-23 — Output safety:** `--output` shall not overwrite either input policy or the successor-map path and shall use existing safe output handling conventions.
- [ ] **M-24 — Compatibility:** Existing `forge convert --stable-id-baseline` and `forge diff` behavior shall remain unchanged in the MVP.

### Should Have (S) — High-value fast follows 🟡 `@human-review`

- [ ] **S-1 — Summary-only:** `--summary-only` should omit entry prose while retaining reconciled counts, IDs, evidence codes, approval state, and exit behavior.
- [ ] **S-2 — Path redaction:** `--redact-paths` should replace user-supplied directory components with basenames while preserving raw content hashes and provenance type.
- [ ] **S-3 — CI policy:** `--fail-on <any|identity|ambiguous|retirement|never>` should allow repositories to refine which completed outcomes return exit `1`, while the default remains `any`.
- [ ] **S-4 — Baseline guidance:** When `--stable-id-baseline` is used, CLI help should point users to `forge migrate` for a full report; deprecation should require separate adoption evidence and is not implied by this PRD.
- [ ] **S-5 — Reusable engine:** Inventory and classification types should be library-accessible so the official GitHub Action can consume structured results without parsing human text.
- [ ] **S-6 — Machine evidence codes:** JSON entries should use stable evidence codes such as `exact_id`, `unique_normalized_text`, `same_locator`, `atomization_lineage`, and `reviewer_declaration`.

### Could Have (C) — Desirable if time permits 🟢 `@llm-autonomous`

- [ ] **C-1 — Reviewer template:** `--emit-successor-template <FILE>` could write unresolved candidates as an unapproved mapping template without altering either policy.
- [ ] **C-2 — Selected prose:** `--include-prose` could include full old/new requirement text in JSON; the default machine report could use fingerprints and bounded previews to reduce accidental disclosure.
- [ ] **C-3 — Artifact cross-reference:** Optional Catalog artifacts could enrich entries with derived OSCAL `control-id` values while stable UUIDs remain the migration keys.
- [ ] **C-4 — Lexical evidence:** A future deterministic lexical-similarity score could rank unresolved candidates, but could not approve or automatically select a successor.

### Won't Have (W) — Explicitly excluded this release 🔴 `@human-required`

- [ ] **W-1 — `--apply` mode:** No command will rewrite IDs or policies; auditability requires the source and generated ID algorithm to remain authoritative.
- [ ] **W-2 — AI/embedding matching:** No nondeterministic or opaque semantic relationship inference in v1.2.
- [ ] **W-3 — Native watch daemon:** CI integration belongs to the GitHub Action initiative; users may invoke `forge migrate` from existing file watchers.
- [ ] **W-4 — Approval authentication/signing:** FORGE preserves reviewer declarations but does not verify identity, signatures, organizational authority, or non-repudiation in the MVP.
- [ ] **W-5 — Binary document layout reconstruction:** No PDF page geometry, OCR, DOCX tracked-change resolution, comments, footnotes, or native paragraph IDs.
- [ ] **W-6 — Downstream artifact rewriting:** No automated repair of Catalog, Component Definition, SSP, Assessment Plan, or other references after an ID change.

---

## Interface Contract 🟡 `@human-review`

### CLI

```text
forge migrate <OLD_POLICY> <NEW_POLICY> \
  [--format text|json] \
  [--output <PATH>] \
  [--successor-map <PATH>] \
  [--max-size <MB>]
```

Defaults are text to stdout, no successor declarations, the same maximum policy size as `forge convert`, and exit behavior from M-20.

### Successor Map JSON

```json
{
  "schema_version": "forge.successor-map/1",
  "relationships": [
    {
      "relationship": "successor",
      "old_ids": ["11111111-1111-5111-8111-111111111111"],
      "new_ids": ["22222222-2222-5222-8222-222222222222"],
      "approved_by": "Jane Reviewer",
      "approved_at": "2026-08-22T17:00:00Z",
      "rationale": "The password-length requirement was revised from 12 to 14 characters."
    }
  ]
}
```

Cardinality rules:

- `successor`: exactly one `old_id` and one `new_id`
- `split`: exactly one `old_id` and two or more `new_ids`
- `merge`: two or more `old_ids` and exactly one `new_id`

### Migration Report JSON

The versioned `forge.migration-report/1` envelope shall contain `analysis_complete`, old/new source provenance, reconciled summary counts, and a sorted `entries` array. Each entry shall carry a classification, evidence codes, approval status, old/new ID arrays, normalized-text fingerprints, and old/new locations. The schema must define every enum and field and avoid unstable map ordering. `analysis_complete: true` appears only in a successfully emitted report; failures return stderr instead of a partial report masquerading as complete.

---

## Acceptance Criteria 🟡 `@human-review`

| AC ID | Traces To | Given | When | Then |
|-------|-----------|-------|------|------|
| AC-1 | M-1, M-2, M-3, M-16 | Two valid Markdown policy revisions | Running `forge migrate old.md new.md` | A complete text report is emitted and every old/new stable ID reconciles exactly once |
| AC-2 | M-4, M-5 | Revisions containing unchanged, added, retired, moved, and edited requirements | Running migration | Each outcome is labeled separately with IDs, fingerprints, locations, and evidence |
| AC-3 | M-6, M-7 | A substantive-change candidate without a successor map | Running migration | The relationship remains a candidate; neither input is modified and no approved successor is emitted |
| AC-4 | M-8, M-9 | A valid reviewer-authored one-to-one mapping | Running with `--successor-map` | The entry is `declared_successor` and preserves reviewer, approval time, and rationale |
| AC-5 | M-8, M-11 | Valid declared split and merge relationships | Running with `--successor-map` | Cardinality is preserved as one grouped relationship rather than flattened pairs |
| AC-6 | M-9 | A map references an ID absent from its corresponding inventory | Running migration | Analysis stops, stderr names the invalid reference, no complete report is emitted, and exit code is `2` |
| AC-7 | M-10, M-13 | Unchanged prose moves from line 10 to line 20 | Running migration | FORGE reports an `observed_id_change`, both locations, changed seed fields, and `not_approved` status |
| AC-8 | M-10, M-12 | The same normalized prose appears twice on both sides and exact IDs do not resolve it | Running migration | FORGE emits one deterministic ambiguity group and does not pair by traversal order |
| AC-9 | M-11 | One compound requirement becomes two atoms | Running migration | FORGE emits a grouped atomization candidate or a declared split, never two arbitrary successors |
| AC-10 | M-14, M-15 | Text-bearing PDF and DOCX revisions | Running migration | Locations use `normalized_extracted_text_line`, raw hashes and formats are present, and no native page/paragraph claim is made |
| AC-11 | M-15 | A scanned PDF has no extractable text | Running migration | A descriptive extraction/OCR error is printed to stderr, no report is emitted, and exit code is `2` |
| AC-12 | M-17, M-18, M-19 | Identical inputs/options run twice with JSON output | Comparing stdout bytes | Reports are byte-identical valid JSON and contain no logging text or wall-clock field |
| AC-13 | M-20 | Identical requirement inventories and locations | Running migration | Report is emitted and process exits `0` |
| AC-14 | M-20 | Any completed migration with a change, declaration, location change, or ambiguity | Running migration | Report is emitted and process exits `1` without an `Error:` prefix |
| AC-15 | M-20, M-21 | Invalid input or a stable-ID integrity anomaly | Running migration | Process exits `2`, explains why no trustworthy complete report was produced, and does not emit relationship claims |
| AC-16 | M-22, M-23 | An oversized input or `--output` matching an input path | Running migration | FORGE rejects the operation without modifying either input |
| AC-17 | M-24 | Existing `forge diff` and `convert --stable-id-baseline` fixtures | Running the full test suite | Their CLI contracts and outputs remain unchanged |

### Edge Cases 🟢 `@llm-autonomous`

- [ ] **EC-1:** Old and new policies contain zero extracted requirements; summary totals are zero and exit code is `0` if both documents otherwise parse successfully.
- [ ] **EC-2:** Only YAML frontmatter or non-requirement prose changes; requirement migration remains unchanged, while source hashes show that input bytes differ.
- [ ] **EC-3:** Blank-line insertion shifts normalized source lines for many unchanged requirements; each uniquely matched item is an observed ID change, not a substantive rewrite.
- [ ] **EC-4:** A section rename changes the stable-ID seed for every child requirement; unique exact prose is reported as observed ID changes with `section_path_changed` evidence.
- [ ] **EC-5:** Two identical requirements in different sections remain distinguishable when their exact IDs persist; duplicates become ambiguous only after exact evidence is exhausted.
- [ ] **EC-6:** Atom order changes within one compound statement; grouped atomization evidence is retained and no order-based successor is asserted.
- [ ] **EC-7:** A requirement is both moved and substantively edited; without a declaration, it remains a candidate or ambiguous rather than being labeled an observed exact-text transition.
- [ ] **EC-8:** A successor map contains duplicate IDs inside one relationship, contradictory relationships, or an empty approval field; analysis fails with exit `2`.
- [ ] **EC-9:** Requirement text contains ANSI escapes or other terminal control characters; text output renders them inert while JSON preserves data through standard escaping.

---

## Technical Constraints 🟡 `@human-review`

- **Language:** Rust edition 2024 on the repository's pinned stable toolchain.
- **Dependencies:** Reuse existing `serde_json`, `sha2`, ingestion, atomization, UUID, and output facilities. No new crate is required for the MVP.
- **Stable-ID authority:** `crate::uuid::assign_stable_ids` remains the only production assignment path. Migration code consumes its output and must not reproduce its seed construction.
- **Comparison unit:** Internal atomized `PolicyRequirement`, not raw Markdown blocks or generated OSCAL `control-id` strings.
- **Normalization:** Reuse `crate::uuid::normalize_for_hashing`; store a SHA-256 fingerprint of normalized text in reports rather than treating UUID v5 as a content hash.
- **Bounds:** Preserve existing maximum input size, maximum atom splits, and maximum section depth. Add bounded successor-map entries, string lengths, and total relationship count.
- **Ordering:** Avoid relying on `HashMap` iteration. Sort inventories and output explicitly using Unicode scalar-value byte order for serialized strings.
- **No clock:** Do not add report generation time. Reviewer-supplied `approved_at` is data from the mapping file and therefore deterministic for the same inputs.
- **Error handling:** No panics on untrusted policy or mapping content. Errors must identify the affected file/field without logging full sensitive prose.
- **Compatibility:** The new command may reuse diff concepts, but must not change the current `DiffHasChanges` behavior or artifact extraction contract.

### Reconciliation Invariants 🟡 `@human-review`

- The union of `old_ids` across all entries equals the old inventory exactly.
- The union of `new_ids` across all entries equals the new inventory exactly.
- No ID appears in two top-level entries.
- Entry cardinality agrees with classification (`retired` has old only, `added` has new only, split is one-to-many, merge is many-to-one).
- Summary counts are computed from entries, not independently accumulated.
- A declared relationship retains its approval metadata verbatim and its evidence includes `reviewer_declaration`.
- A candidate relationship never carries an approved status.

---

## Security & Privacy Considerations 🟡 `@human-review`

| Risk | Impact | Required Mitigation |
|------|--------|---------------------|
| Policy disclosure through report prose | Reports can reveal security requirements and how posture changed | Treat reports as having the same sensitivity as both source policies; default JSON to hashes/bounded previews; document `--include-prose` risk if added |
| Local path disclosure | Absolute paths can reveal usernames and repository layout | Preserve stable user-supplied labels rather than canonical absolute paths in output; add `--redact-paths` as a fast follow |
| Terminal escape injection | Malicious policy text could alter terminal presentation | Escape or render control characters inert in human-readable output |
| Malicious PDF/DOCX/JSON | Parser exhaustion, decompression bombs, malformed structure | Reuse file-size/recursion bounds, bound ZIP expansion and successor-map collections, return errors without panic |
| False authority | Users may over-trust heuristic matches or unauthenticated reviewer names | Use `observed`, `candidate`, and `declared` terminology consistently; never say a reviewer identity was verified |
| Stale downstream references | ID changes may break linked OSCAL artifacts | Report all old/new IDs and explicit relationships; do not claim downstream artifacts were updated |
| Sensitive logging | Debug output could leak requirement prose or approval rationale | Log IDs, counts, evidence codes, and bounded path labels only; do not log full prose or successor rationale by default |
| Accidental overwrite | Output path could target an input or approval record | Reject output aliases to inputs and use existing safe output conventions |

The feature performs no network access, executes no external process, and requires no authentication. Its main security property is integrity of classification and clarity about evidence, not confidentiality provided by FORGE itself.

---

## Dependencies & Interactions 🟡 `@human-review`

- **Requires:** Existing policy ingestion (`src/ingest`), shared document preparation (`src/pipeline.rs`), atomization (`src/parse/atomize.rs`), stable-ID generation (`src/uuid.rs`), traceability source locations, and CLI/output/error infrastructure.
- **Reuses concepts from:** `forge diff` report sorting, stdout behavior, and `0`/`1`/error CI convention; it does not reuse artifact matching by `control-id` as the primary algorithm.
- **Consumes evidence from:** Current `--stable-id-baseline` locator logic and golden edge-case fixtures for whitespace/substantive changes.
- **Enables:** Official GitHub Action drift checks and later Control Mapping change-impact reporting.
- **Independent of:** OSCAL 1.2.3 schema upgrade; this feature compares internal policy requirements before OSCAL serialization.
- **Configuration interaction:** Project-level `.forge.toml` may later provide default `max-size`, output format, or CI policy. Command-line flags must retain precedence and the MVP must work without a config file.

---

## Risks & Mitigations 🟡 `@human-review`

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|------------|--------|------------|
| R-1 | Location-sensitive stable IDs create large reports after line insertion or section movement | High | Medium | Unique exact-text matching; grouped summaries; summary-only fast follow; explicitly label structural causes |
| R-2 | Same-locator matching pairs unrelated replacements after reordering | Medium | High | Label as candidate only; do not approve; surface old/new evidence and ambiguity |
| R-3 | Duplicate prose causes arbitrary pairing | High | High | Exhaust exact IDs first; group competing candidates; never use traversal order to resolve |
| R-4 | Atomizer behavior changes between FORGE versions | Medium | High | Record FORGE version and atomization evidence; use grouped cardinality; include cross-version fixtures |
| R-5 | PDF/DOCX extraction locations are mistaken for original native coordinates | Medium | High | Required location-basis label and explicit limitation in every non-Markdown report |
| R-6 | Successor map becomes an unaudited way to launder guesses into authoritative mappings | Medium | High | Require reviewer, time, rationale; validate references; label declarations unauthenticated; preserve candidate evidence separately |
| R-7 | JSON schema churn breaks CI consumers | Medium | Medium | Version the report and successor-map schemas; additive changes within v1; contract fixtures |
| R-8 | Exit `1` is interpreted as execution failure rather than review signal | Medium | Medium | Match `forge diff` convention; document shell/GitHub Action handling; reserve `2` for incomplete analysis |

---

## Success Metrics — Hypotheses 🔴 `@human-required`

### Leading Indicators

| Hypothesis | Success Threshold | Stretch | Measurement |
|------------|-------------------|---------|-------------|
| H-1: Practitioners can complete a migration review from the report | ≥80% of five pilot tasks completed without maintainer intervention | 100% | Moderated task completion using sanitized policy pairs |
| H-2: The report reduces reconciliation time | Median time ≥50% lower than manual source plus OSCAL comparison | ≥70% lower | Within-subject timed comparison on the same fixture complexity |
| H-3: Classification is accurate enough to earn trust | ≥95% agreement with two-person human review across fixture and pilot entries | ≥98% | Blind reviewer adjudication; candidates measured separately from declarations |
| H-4: Ambiguity is visible rather than hidden | 100% of seeded duplicate/many-to-many cases emitted as ambiguity or grouped atomization candidates | 100% | Golden and adversarial fixture suite |
| H-5: CI integration is predictable | 100% correct exit status and valid JSON across no-change/change/error matrix | 100% | Cross-platform integration tests |

### Lagging Indicators

| Hypothesis | Success Threshold | Evaluation Window | Measurement |
|------------|-------------------|-------------------|-------------|
| H-6: Teams adopt repeatable migration review | At least 3 external repositories run migration checks on 2 or more policy revisions | 90 days post-release | Opt-in design-partner observation; no telemetry required |
| H-7: Reports support audit evidence | At least 3 of 5 design partners say the report is sufficient to explain a sampled ID transition | 60 days | Structured interview and evidence-review rubric |
| H-8: The feature avoids relationship corrections | <5% of `observed_id_change` entries are rejected by reviewers; candidate acceptance tracked separately | 90 days | Pilot review logs supplied voluntarily |

### Technical Quality Gates

- 100% reconciliation invariant coverage across Markdown, PDF, and DOCX fixtures.
- Byte-for-byte determinism tests for text and JSON on macOS, Linux, and Windows.
- 100% test coverage of classification branches and the exit-code matrix.
- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` pass.
- Zero panics and zero input mutations across adversarial tests.

---

## Rollout & Phasing 🟡 `@human-review`

### Phase 0 — Contract and fixture lock

- Approve classification precedence and versioned schemas, then lock fixtures and a human-adjudicated answer key for every core and provenance case.

### Phase 1 — Inventory and deterministic core

- Deliver inventory reconciliation; exact, moved, edited, added, retired, and ambiguous classes; deterministic text/JSON; and `0`/`1`/`2` behavior while keeping `--stable-id-baseline` unchanged.

### Phase 2 — Atomization and declarations

- Add atomization groups and declared mappings, complete PDF/DOCX fixtures, and review untrusted-map parsing and terminal rendering.

### Phase 3 — Pilot and release gate

- Test with at least five sanitized real-world revision pairs from design partners.
- Measure task completion, time, reviewer agreement, and ambiguity rate.
- Publish JSON/successor-map schemas and CI examples.
- Release only after all reconciliation invariants and cross-platform determinism gates pass.

### Post-MVP decision points

- Add configurable `--fail-on` only if repository pilots need different CI policies.
- Consider lexical candidate ranking only if unresolved cases materially block users and reviewers can evaluate false-positive risk.
- Consider deprecating `--stable-id-baseline` only after `forge migrate` adoption and compatibility data justify it.

---

## Open Questions 🟡 `@human-review`

- **[Product + Security, non-blocking]** Should path redaction become the default for JSON before v1.2 release, even though full user-supplied paths can strengthen local audit context?
- **[Product + Engineering, non-blocking]** Should `--fail-on` ship with the MVP or wait until GitHub Action pilots demonstrate which policies teams actually need?
- **[Compliance, non-blocking]** What minimum reviewer metadata is sufficient for customers that require non-repudiation, and should signed successor maps become a separate future initiative?
- **[Engineering, non-blocking]** Should the report record only the FORGE semantic version or also an explicit stable-ID/atomizer algorithm version for future cross-version migrations?
- **[Product, non-blocking]** Should full requirement prose be included by default in human-readable reports while JSON defaults to hashes and bounded previews?

---

## Decision Log 🟡 `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-22 | Add `forge migrate` rather than expanding the baseline warning | A dedicated report can express additions, retirements, grouped relationships, ambiguity, provenance, and CI output without overloading conversion | Add more warning text to `forge convert`; replace `forge diff` |
| 2026-08-22 | Treat migration as read-only | Stable IDs are derived output; silently preserving or rewriting them would make artifacts non-reproducible and obscure audit evidence | Write old IDs into new source; maintain an implicit cache |
| 2026-08-22 | Separate observed, candidate, and declared relationships | Exact facts, deterministic hints, and human decisions have different evidentiary strength | Automatically promote high-confidence matches to successors |
| 2026-08-22 | Support explicit one-to-one, split, and merge declarations | Atomization changes are naturally one-to-many or many-to-one and must not be flattened | One-to-one mappings only; arbitrary many-to-many mappings |
| 2026-08-22 | Use `0` no change, `1` reviewable change, `2` incomplete/error | Matches common diff semantics and supports CI without conflating expected drift with a broken analysis | Always exit 0; unique exit code for every classification |
| 2026-08-22 | Label PDF/DOCX locations as normalized extracted-text lines | Current ingestion converts these formats to synthetic line-oriented text and does not preserve native coordinates | Claim page/paragraph accuracy; exclude PDF/DOCX entirely |
| 2026-08-22 | Exclude fuzzy semantic matching from MVP | False relationship claims are more damaging than visible ambiguity in an audit workflow | Token similarity threshold; embeddings; LLM matching |

---

## Definition of Ready 🔴 `@human-required`

### Readiness Checklist

- [ ] Classification vocabulary and precedence reviewed by Product and Engineering
- [ ] JSON report and successor-map v1 contracts reviewed
- [ ] Exit-code contract approved for CLI and GitHub Action consumption
- [ ] PDF/DOCX provenance wording reviewed for accuracy
- [ ] Fixture adjudication answer key approved by a compliance practitioner
- [ ] Security/privacy mitigations accepted
- [ ] All Must Have requirements trace to testable acceptance criteria

### Sign-off

| Role | Name | Date | Decision |
|------|------|------|----------|
| Product Owner | Brian Luby | YYYY-MM-DD | [Ready / Not Ready] |

---

## Changelog ⚪ `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-22 | Codex | Initial draft from FORGE v1.2 roadmap priority 3, grounded in current stable-ID, diff, traceability, atomization, and ingestion behavior |
| 0.2 | 2026-08-25 | Codex | Implemented closed reviewer-authored successor, split, and merge declarations with validation, approval evidence, deterministic reports, and output safety |

---

## Review Checklist 🟢 `@llm-autonomous`

- [x] Problem, users, goals, and non-goals are explicit
- [x] User stories are prioritized and independently testable
- [x] Requirements use MoSCoW prioritization and unique IDs
- [x] Must Have requirements have Given/When/Then acceptance coverage
- [x] Observed, candidate, and declared relationship semantics are distinct
- [x] Reordering, line shifts, duplicate prose, and atomization changes are addressed
- [x] PDF/DOCX provenance constraints are explicit
- [x] CI exit codes and deterministic output are defined
- [x] Security and privacy risks are documented
- [x] Success metrics are labeled as hypotheses with measurement methods
- [x] Dependencies, phasing, decision log, and open questions are included

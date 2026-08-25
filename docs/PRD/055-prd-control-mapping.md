# 055-prd-control-mapping

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Technical implementation complete; human release gates pending
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `055-control-mapping`
**Created**: 2026-08-22
**Status**: Technical implementation complete; human release gates pending
**Input**: FORGE v1.3 roadmap priority 5

---

## Context

### Background :red_circle: `@human-required`

FORGE currently converts policy documents into OSCAL Catalogs, generates Profiles, embeds source traceability, validates supported OSCAL artifacts, and compares Catalog or Component Definition revisions. These capabilities make policy requirements machine-readable, but they do not express reviewed relationships between a policy Catalog or Profile and another control framework. Compliance engineers still maintain those crosswalks in spreadsheets, where relationship direction, granularity, provenance, gaps, and source versions are easy to lose.

OSCAL v1.2.3 introduces a released Control Mapping model whose `mapping-collection` root represents mappings between Catalog or Profile resources. It supports control- and statement-level subjects, one-to-one and grouped cardinalities, set-theory relationships, matching rationale, method, confidence estimates, qualifiers, responsible parties, and source/target gap summaries. The official model also cautions that a mapping affirms relationships that are present; users must not infer meaning from relationships that are absent.

This PRD defines a human-reviewed MVP. A reviewer supplies two local OSCAL resources and a versioned mapping manifest containing explicit relationships. FORGE validates the resources and every referenced subject, preserves provenance and reviewer-authored rationale, emits schema-valid OSCAL v1.2.3 `mapping-collection` JSON, and produces deterministic coverage, gap, and change-impact reports. FORGE does not decide that controls are equivalent.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| OSCAL specification | A mapping collection declares relationships but does not make absence meaningful. | Gaps and coverage must be explicitly scoped and must not be presented as proof of non-compliance. |
| OSCAL specification | A mapping entry accepts one or more `source` items and one or more `target` items of type `control` or `statement`. | The manifest must preserve many-to-many cardinality instead of flattening relationships into pairs. |
| OSCAL specification | Standard relationships are `equivalent-to`, `equal-to`, `subset-of`, `superset-of`, `intersects-with`, and `no-relationship`; matching rationale is syntactic, semantic, or functional. | FORGE must validate vocabulary and direction while leaving the judgment to the reviewer. |
| Repository evidence | Catalog controls and statement parts expose OSCAL `id` values; Profile generation exists, while Profile resolution delegates to `oscal-cli`. | Catalog references can be checked directly. Profile mappings require a caller-supplied resolved Catalog for effective-subject validation. |
| Repository evidence | `forge diff` already sorts control changes deterministically, and traceability records source locations for generated artifacts. | Mapping reports can reuse deterministic ordering and trace concepts, but need a separate model-specific subject inventory. |
| Repository evidence | Current validation recognizes Catalog, Component Definition, and Profile schemas only; PRD 054 upgrades their baseline but explicitly leaves Mapping to this PRD. | PRD 054 must establish the v1.2.3 baseline first; PRD 055 then adds the release-matched Mapping schema and model support. |
| Product hypothesis | A standards-native, reviewer-controlled crosswalk will be more trustworthy and reusable than a spreadsheet for GRC teams. | Validate through design-partner tasks; do not treat roadmap ranking as user-research evidence. |

No user interviews, support-ticket counts, paid-pilot evidence, or production mapping corpus were supplied. Adoption and time-savings targets in this PRD are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- One mapping collection containing exactly one Control Mapping between one source and one target OSCAL Catalog/Profile resource
- Local JSON Catalog and Profile inputs supplied by the user; no network retrieval
- A caller-supplied resolved Catalog companion for each unresolved Profile used as a source or target
- A versioned, reviewer-authored JSON mapping manifest
- Validation of OSCAL schemas, document types, element references, relationship vocabulary, cardinality, duplicate references, provenance, and reviewer records
- Control-level and statement-level maps with one-to-one, one-to-many, many-to-one, and many-to-many cardinality
- Deterministic UUIDs for collection, mapping, map, gap-summary, party, and supporting objects from stable manifest keys
- OSCAL-native provenance fields plus namespaced FORGE properties where the OSCAL model lacks a dedicated field
- Schema-valid OSCAL v1.2.3 `mapping-collection` JSON output
- Deterministic human-readable and JSON review reports for mapping participation, unmapped controls, statement-scope gaps, validation findings, and baseline change impact
- Read-only validation of a prior mapping collection against current resource versions and stable mapping keys

**Out of Scope:**

- Automatic semantic, syntactic, functional, AI, embedding, fuzzy, or heuristic crosswalk generation
- Automatic approval or promotion of a relationship based on confidence
- Bundled NIST, ISO, CIS, SOC 2, PCI DSS, or other framework Catalog content
- Downloading Catalogs, resolving remote `href` values, crawling URLs, or authenticating to GRC systems
- Native Profile Resolution; callers use `forge resolve`/`oscal-cli` and supply the result
- Editing source Catalogs/Profiles or automatically repairing missing/renamed identifiers
- Assessment Results, POA&M, SSP, or Component Definition mapping
- XML/YAML Mapping output until existing export paths demonstrate semantically lossless mapping-model round trips
- General plugin hooks, hosted services, web UI, or an automatic mapping suggestion engine

### Related Documents and Standards :white_circle: `@auto`

| Document | Link | Relationship |
|----------|------|--------------|
| Product Vision | `docs/FORGE_PRODUCT_VISION.md` | Correctness, traceability, determinism, CLI-first, and standards-native principles |
| Product Roadmap | `docs/FORGE_PRODUCT_ROADMAP.md` | Completed v1.1 context and v1.3 planning input |
| OSCAL v1.2.3 Compatibility PRD | `docs/PRD/054-prd-oscal-1-2-3-compatibility.md` | Blocking standards-baseline dependency; Mapping remains owned here |
| Catalog Pipeline PRD | `docs/PRD/013-prd-catalog-pipeline.md` | Current Catalog structure and generation |
| Profile Generation PRD | `docs/PRD/030-prd-profile-generation.md` | Current Profile input and selection behavior |
| Profile Resolution PRD | `docs/PRD/036-prd-oscal-cli-profile-resolution.md` | Existing delegated resolution path |
| Traceability Model PRD | `docs/PRD/016-prd-traceability-model.md` | Existing stable requirement and source-location concepts |
| Diff Report PRD | `docs/PRD/043-prd-diff-report.md` | Existing deterministic change-report conventions |
| Stable-ID Migration PRD | `docs/PRD/053-prd-stable-id-migration.md` | Human-declared identity transitions and downstream change-impact concepts |
| NIST Control Mapping overview | [OSCAL Control Mapping Model](https://pages.nist.gov/OSCAL/learn/concepts/layer/control/mapping/) | Model purpose, relationship semantics, and mapping guidance |
| NIST v1.2.3 JSON reference | [Control Mapping JSON Format Reference](https://pages.nist.gov/OSCAL-Reference/models/v1.2.3/mapping/json-reference/) | Authoritative fields, cardinalities, and constraints |
| NIST v1.2.3 JSON schema | [Control Mapping JSON Schema](https://github.com/usnistgov/OSCAL/releases/download/v1.2.3/oscal_mapping_schema.json) | Release-pinned validation contract |

---

## Problem Statement :red_circle: `@human-required`

Compliance engineers and auditors need to explain how requirements in one authoritative control source relate to requirements in another. Spreadsheet crosswalks can capture reviewer judgment, but they rarely validate that referenced controls still exist, preserve many-to-many semantics cleanly, identify source versions, or produce a standards-native artifact suitable for automation. Without a human-reviewed OSCAL mapping workflow, FORGE users either maintain a second, weakly governed source of truth or depend on opaque mapping claims that are difficult to audit when frameworks change.

---

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Produce interoperable mappings from explicit human decisions. | 100% of accepted manifests emit JSON that validates against the release-pinned OSCAL v1.2.3 Mapping schema. |
| G-2 | Prevent stale or invalid references from becoming mapping claims. | 100% of seeded missing, wrong-side, wrong-type, duplicate, and ambiguous subject references fail before OSCAL output is written. |
| G-3 | Preserve audit provenance. | Every emitted mapping collection names its input resource versions/fingerprints and responsible reviewer parties; every map preserves rationale and stable identity. |
| G-4 | Make review scope and gaps reproducible. | Identical input bytes and manifest bytes produce byte-identical mapping JSON and reports; inventory totals reconcile exactly. |
| G-5 | Expose framework change impact without inventing successors. | Baseline checks identify 100% of seeded removed, renamed, content-changed, and newly unmapped subjects while leaving resolution to a human. |
| G-6 | Reduce crosswalk maintenance effort. | In a five-partner pilot, median time to validate and publish a 100-entry reviewed mapping is at least 40% lower than each partner's current spreadsheet workflow. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- **No automated relationship judgment.** FORGE validates declarations; it does not generate `equivalent-to`, `subset-of`, or any other relationship from control prose.
- **No compliance verdict.** A mapping, confidence value, participation percentage, or lack of a map does not establish compliance, implementation, effectiveness, or audit sufficiency.
- **No framework redistribution.** Users must supply lawfully obtained OSCAL resources; FORGE will not package restricted standards or excerpts.
- **No hidden Profile resolution.** An unresolved Profile must have an explicitly supplied resolved Catalog companion; FORGE will not implement a partial resolver.
- **No identity repair.** Missing or renamed `id-ref` values are reported as change impact; FORGE never guesses or rewrites a replacement.
- **No full mapping editor.** The MVP uses a reviewable JSON manifest and CLI, not an interactive UI or collaborative approval system.

---

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Publish a reviewed mapping collection (P0)

> As a compliance engineer, I want to turn my reviewed mapping decisions into valid OSCAL so that downstream tools can consume the crosswalk without reinterpreting a spreadsheet.

**Independent Test:** Provide two valid Catalogs and a manifest with reviewed one-to-one and grouped maps; verify a valid `mapping-collection` is emitted with unchanged relationship direction and cardinality.

### US-2 — Reject invalid mapping references (P0)

> As an auditor, I want every source and target identifier validated against the exact resource version so that the published mapping does not contain dangling or wrong-type claims.

**Independent Test:** Reference an absent source control, a target control on the source side, and a non-statement part as `statement`; verify each fails with an actionable path and no output artifact.

### US-3 — Preserve provenance and human responsibility (P0)

> As an auditor, I want reviewer, method, matching rationale, decision rationale, resource fingerprint, and review time preserved so that I can evaluate who made each mapping claim and on what basis.

**Independent Test:** Build a mapping with two reviewers and per-map rationale; verify metadata parties, provenance responsible parties, standard fields, and namespaced per-map review properties survive serialization.

### US-4 — See deterministic mapping gaps (P0)

> As a compliance engineer, I want explicit unmapped-control lists and scoped participation counts so that I can plan remaining review without treating absence as evidence of no relationship.

**Independent Test:** Map 7 of 10 target controls; verify the report shows 70% target control participation and the three sorted gaps, while avoiding a compliance-coverage claim.

### US-5 — Detect framework change impact (P0)

> As a compliance engineer, I want to check an existing mapping against revised Catalog/Profile resources so that removed, changed, or newly unmapped subjects are sent back for review.

**Independent Test:** Remove one referenced control, change the prose of another, add one target control, and rerun with a baseline; verify each impact is distinct and no successor is selected.

### US-6 — Map effective Profile selections (P1)

> As a control owner, I want to map a tailored Profile's effective controls so that the crosswalk reflects the baseline I actually review rather than its entire source Catalog.

**Independent Test:** Supply a Profile plus its resolved Catalog; verify references are validated against the resolved effective inventory and the emitted resource remains identified as a Profile.

### US-7 — Consume reports in CI (P1)

> As a DevSecOps engineer, I want versioned JSON reports and predictable exit statuses so that pull requests can require human review when a framework update invalidates mappings.

**Independent Test:** Run the no-impact, review-required, and analysis-error matrix and verify valid JSON, clean stdout, and documented statuses.

---

## Product Guardrails :red_circle: `@human-required`

1. **Human review is mandatory.** Every map must originate in the manifest and carry a reviewer reference and non-empty rationale. FORGE-generated suggestions are not part of this feature.
2. **Confidence is an estimate, not truth.** It is preserved verbatim after range/vocabulary validation and never changes acceptance, sorting, gap status, or exit behavior.
3. **Direction matters.** `subset-of` and `superset-of` are evaluated from source to target; FORGE must never reverse them while sorting or rendering.
4. **Absence is not `no-relationship`.** Unmapped items are gaps in the reviewed manifest, not automatically emitted `no-relationship` maps.
5. **Granularity remains explicit.** A control and one of its statement parts are distinct subjects. FORGE must not expand or collapse them silently.
6. **Resource bytes are evidence.** Reports identify the exact local artifacts by SHA-256, root UUID, metadata version, and OSCAL version without embedding their full content.

---

## Functional Model :yellow_circle: `@human-review`

### Subject Inventory

For a Catalog, FORGE recursively inventories root controls, grouped controls, child controls, and nested parts. An eligible `control` subject is keyed by its OSCAL control `id`. An eligible `statement` subject is a part with `name: "statement"` and a non-empty `id`; other part names cannot be referenced as statements in the MVP.

For a Profile, the caller supplies both the original Profile JSON and its resolved Catalog JSON. FORGE schema-validates both, inventories effective subjects from the resolved Catalog, fingerprints both files, and records that the mapped resource type is `profile`. The MVP does not prove that a separately produced resolved Catalog came from the supplied Profile; it records both hashes and requires a reviewer attestation in the manifest.

Inventories reject duplicate control IDs, duplicate statement IDs within a resource, ambiguous IDs shared across eligible types, excessive nesting, and malformed identifiers. FORGE does not fetch imports or follow links.

### Relationship and Cardinality Semantics

| Manifest Value | OSCAL Meaning | Reversible? | MVP Treatment |
|----------------|---------------|-------------|---------------|
| `equal-to` | Same effective requirements | Yes | Preserve reviewer claim |
| `equivalent-to` | Similar information with same effective meaning | Yes | Preserve reviewer claim; never infer |
| `subset-of` | Source requirements are contained by target requirements | Reverse is `superset-of` | Preserve direction exactly |
| `superset-of` | Source requirements contain target requirements | Reverse is `subset-of` | Preserve direction exactly |
| `intersects-with` | Partial overlap in both directions | Yes | Preserve reviewer claim and rationale |
| `no-relationship` | Affirmed lack of overlap for a reviewed edge case | Yes | Require explicit reviewer declaration; never derive from a gap |

Every map has at least one source and one target. Repeated subjects inside one side are errors. One-to-many, many-to-one, and many-to-many sets remain one OSCAL `map` object. The manifest order is not authoritative; canonical output sorts subjects by `(type, id-ref)` and maps by their deterministic UUID without changing source/target direction.

### Coverage and Gap Semantics

FORGE reports **review participation**, not semantic coverage:

```text
target_control_participation = unique target control IDs present in maps
                               / eligible target control IDs in scope
```

The same calculation is produced separately for source controls and, when statement scope is enabled, for source and target statements. A control that appears in multiple maps counts once. `no-relationship` counts as reviewed participation because a human explicitly reviewed that pair; it does not count as compliance coverage.

Sorted unmapped control IDs are emitted in OSCAL `source-gap-summary` and `target-gap-summary`. Statement gaps and all denominator details appear in the FORGE report because the standard gap-summary structure is control-selection based. Standard OSCAL `coverage.target-coverage` is omitted by default: deriving partial coverage for `subset-of` or `intersects-with` requires a judgment the MVP cannot make. A reviewer may supply an OSCAL coverage estimate only with `generation-method: "arbitrary"`; the report labels it `reviewer_estimate` and keeps it separate from deterministic participation.

### Stable Identity and Change Impact

- `collection.key`, each `mapping.key`, and each `map.key` are immutable manifest identifiers chosen by the reviewer.
- FORGE derives UUID v5 values from a documented namespace plus object kind and stable key. Reordering does not change UUIDs.
- Source and target items continue to use OSCAL `id-ref`; the baseline stores their canonical element SHA-256 as a namespaced property.
- Resource references store root UUID, metadata version, OSCAL version, and raw-file SHA-256 as namespaced properties.
- Baseline comparison matches maps by stable UUID, then reports resource fingerprint changes, reference removal, subject-type change, content-fingerprint change, relationship/rationale change, additions, removals, and gap-count changes.
- A missing identifier is `stale_reference`. A same-ID content change is `subject_changed`. A new identifier is `new_gap` until a reviewer updates the manifest. None is an inferred successor.

---

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [x] **M-1 — Command:** Provide `forge mapping build --manifest <FILE>` with optional `--output <FILE>`, `--report <FILE>`, `--report-format text|json`, and `--baseline <MAPPING_JSON>`.
- [x] **M-2 — Blocking version:** PRD 054's v1.2.3 compatibility gate shall pass before implementation ships; this feature shall pin and embed the Mapping schema from the same v1.2.3 release, never v1.2.0 or a moving `latest` URL.
- [x] **M-3 — JSON inputs:** Accept local `.json` Catalog/Profile resources only. XML/YAML inputs and output are rejected with guidance until lossless mapping support is proven.
- [x] **M-4 — Offline boundary:** Read only caller-supplied local files. Do not fetch resource `href`, Profile imports, links, schemas, or back-matter URLs.
- [x] **M-5 — Profile companion:** Require a schema-valid resolved Catalog companion for every Profile resource and record fingerprints of both. Never perform partial Profile resolution.
- [x] **M-6 — Manifest contract:** Parse a bounded `forge.mapping-manifest/1` JSON document with closed top-level and nested schemas; reject unknown keys, duplicate keys after decoding, unsupported versions, invalid Unicode, and oversized values.
- [x] **M-7 — Reviewer records:** Require at least one reviewer party, a mapping-reviewer role, responsible-party references, and a non-empty review timestamp. Validate references but state that FORGE does not authenticate identity or authority.
- [x] **M-8 — Per-map human evidence:** Require every map to name a reviewer key and contain non-empty decision rationale. Preserve the reviewer key/time in namespaced properties and rationale in `remarks` without rewriting prose.
- [x] **M-9 — Resource validation:** Schema-validate each source/target Catalog/Profile and verify declared type, root UUID, metadata version, and OSCAL version. A manifest `expected-sha256`, when present, must match.
- [x] **M-10 — Subject inventory:** Recursively index eligible control and statement IDs from each effective resource. Duplicate or ambiguous identifiers are fatal rather than last-write-wins.
- [x] **M-11 — Reference validation:** Every source item shall resolve on the source side and every target item on the target side with the declared type. Errors identify manifest JSON path, side, type, and bounded identifier.
- [x] **M-12 — Cardinality:** Require one or more unique source items and one or more unique target items per map; preserve grouped one-to-many, many-to-one, and many-to-many cardinality in one OSCAL map.
- [x] **M-13 — Vocabulary:** Accept only OSCAL v1.2.3 standard relationship, subject-type, method, matching-rationale, status, confidence, and qualifier values when the OSCAL namespace is used. Custom vocabularies require an explicit absolute namespace and are out of MVP output.
- [x] **M-14 — Relationship direction:** Preserve source-to-target direction exactly, especially for `subset-of` and `superset-of`; never canonicalize by swapping sides.
- [x] **M-15 — No inferred claims:** Emit only map entries present in the reviewer manifest. Never create `no-relationship` from absence or promote gaps/candidates/confidence into maps.
- [x] **M-16 — Provenance:** Populate standard `provenance` method, matching-rationale, status, mapping-description, confidence/coverage when supplied, and responsible parties. Preserve per-mapping overrides where present.
- [x] **M-17 — Resource evidence:** Add namespaced properties for source/target raw SHA-256, root UUID, document version, OSCAL version, and Profile resolved-companion SHA-256 where applicable.
- [x] **M-18 — Stable UUIDs:** Derive valid UUID v5 identifiers for collection, mapping, map, gap summaries, parties, and other generated identified objects from documented stable manifest keys; reject duplicate keys and UUID collisions.
- [x] **M-19 — Schema-valid output:** Serialize typed structures with `serde`, validate the completed JSON against the embedded v1.2.3 Mapping schema, and write no artifact if validation fails.
- [x] **M-20 — Gap summaries:** Compute exact sorted unmapped control IDs for each source and target inventory and emit non-empty OSCAL source/target gap summaries without interpreting gaps as `no-relationship`.
- [x] **M-21 — Deterministic report:** Produce versioned text or JSON with inventory totals, unique referenced totals, review-participation ratios, unmapped IDs, statement gaps when requested, validation results, resource fingerprints, and reviewer-estimate fields kept separate.
- [x] **M-22 — Deterministic bytes:** Given identical resource bytes, manifest bytes, options, baseline bytes, and FORGE version, produce byte-identical OSCAL JSON and reports. Require manifest-supplied metadata time; do not read the clock or emit absolute canonical paths.
- [x] **M-23 — Change impact:** With `--baseline`, match mapping objects and maps by stable UUID and report added/removed/changed maps, resource changes, stale references, subject content changes, subject-type changes, and gap changes without selecting replacements.
- [x] **M-24 — Baseline integrity:** Schema-validate the baseline as OSCAL v1.2.3 Mapping JSON and verify expected FORGE stable-key/fingerprint properties before comparison; otherwise fail as incomplete analysis.
- [x] **M-25 — Stream and write safety:** OSCAL JSON goes to stdout or `--output`; requested reports go only to `--report`; diagnostics go to stderr. Reject output/report paths that alias any input or each other and use existing safe-write conventions.
- [x] **M-26 — Bounds:** Reuse file-size and JSON-depth protections and add documented limits for resources, mappings, maps, subjects per side, reviewers, qualifiers, strings, and report entries. No malformed input may panic or exhaust unbounded memory.
- [x] **M-27 — Confidence safety:** Validate category or decimal `0..=1`, preserve the chosen representation, label it author confidence, and ensure it never changes validation, gap membership, ordering, approval, or exit status.
- [x] **M-28 — Compatibility:** Existing convert, profile, resolve, validate, trace, diff, export, and migrate contracts remain unchanged except that validation gains the Mapping model through this PRD after PRD 054's baseline upgrade.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [x] **S-1 — Scaffold:** `forge mapping init --source <FILE> --target <FILE>` should emit a deterministic unapproved manifest skeleton containing resource fingerprints, inventories, empty maps, and no relationship claims.
- [x] **S-2 — CI check mode:** `forge mapping check --manifest <FILE> --baseline <FILE>` should be a read-only alias focused on impact reporting, with `0` for no review impact, `1` for completed analysis requiring review, and `2` for incomplete/error.
- [x] **S-3 — Fail policy:** `--fail-on <stale|subject-change|gap-increase|any|never>` should refine exit `1` without changing report contents.
- [x] **S-4 — Scope selection:** The manifest should allow explicit control-only or control-plus-statement review scope so denominators and gaps are declared, not inferred from which maps happen to exist.
- [x] **S-5 — Bounded excerpts:** Text reports should optionally include short subject titles/prose excerpts for local review while JSON defaults to IDs and hashes.
- [x] **S-6 — Stable machine codes:** JSON findings should use versioned codes such as `stale_reference`, `subject_changed`, `map_added`, `map_removed`, `relationship_changed`, and `new_gap`.

### Could Have (C) — Desirable if time permits :green_circle: `@llm-autonomous`

- [ ] **C-1 — Mapping diff:** A future `forge diff` extension could compare two Mapping artifacts after the dedicated change-impact contract is stable.
- [ ] **C-2 — Reviewer signature references:** The manifest could preserve detached-signature links or external approval IDs without verifying them in core.
- [ ] **C-3 — Multiple resource pairs:** One invocation could build several `mapping` objects from more than one source/target pair after single-pair usability is validated.
- [ ] **C-4 — Format export:** XML/YAML could be enabled only after typed serializers, schema validation, and JSON-to-format-to-JSON semantic equivalence tests cover every Mapping field used by the MVP.

### Won't Have (W) — Explicitly excluded this release :red_circle: `@human-required`

- [ ] **W-1 — Suggestions:** No NLP, LLM, embeddings, string similarity, or automatic relationship candidates.
- [ ] **W-2 — Confidence threshold:** No `--auto-approve`, score cutoff, or conversion of confidence into truth.
- [ ] **W-3 — Framework packages:** No embedded or downloaded NIST, ISO, CIS, SOC 2, PCI DSS, FedRAMP, or vendor control content.
- [ ] **W-4 — Native Profile resolver:** Continue using the existing delegated resolution workflow.
- [ ] **W-5 — Remote resolution:** No HTTP client, registry, OAuth, credential, or GRC connector behavior.
- [ ] **W-6 — General mapping studio:** No web UI, spreadsheet editor, workflow engine, assignment queue, or approval database.
- [ ] **W-7 — Compliance calculation:** No claim that map count, coverage, or gaps measure implementation or audit readiness.

---

## Interface Contract :yellow_circle: `@human-review`

### CLI

```text
forge mapping build \
  --manifest mappings/access-to-framework.json \
  --output generated/access-mapping.json \
  --report generated/access-mapping-report.json \
  --report-format json \
  [--baseline baselines/access-mapping.json]
```

Build without `--output` writes only OSCAL JSON to stdout. Reports require an explicit `--report` path so machine-readable stdout is never mixed with commentary. A valid build with gaps succeeds because partial mappings are allowed; baseline review policy is handled by `check`/`--fail-on`, not by treating every gap as an execution error.

### Manifest JSON v1

```json
{
  "schema_version": "forge.mapping-manifest/1",
  "collection": {"key": "access-policy-to-target-v1", "title": "Access policy to target framework", "version": "1.0.0", "last_modified": "2026-08-22T17:00:00Z"},
  "reviewers": [
    {"key": "reviewer-1", "type": "person", "name": "Jane Reviewer"}
  ],
  "provenance": {
    "method": "human",
    "matching_rationale": "semantic",
    "status": "draft",
    "mapping_description": "Human-reviewed relationship set.",
    "reviewer_keys": ["reviewer-1"],
    "reviewed_at": "2026-08-22T17:00:00Z"
  },
  "mapping": {
    "key": "access-policy-to-target",
    "scope": "control-plus-statement",
    "source": {"type": "catalog", "artifact": "../catalogs/access-policy.json", "href": "../catalogs/access-policy.json", "expected_sha256": "<64 lowercase hex characters>"},
    "target": {"type": "profile", "artifact": "../profiles/target-profile.json", "resolved_catalog": "../profiles/target-profile-resolved.json", "resolved_catalog_attestation": true, "href": "../profiles/target-profile.json"},
    "maps": [
      {
        "key": "access-authorization",
        "matching_rationale": "semantic",
        "relationship": "subset-of",
        "sources": [{"type": "control", "id_ref": "POL-AC-001"}],
        "targets": [{"type": "statement", "id_ref": "ac-2_smt"}, {"type": "statement", "id_ref": "ac-3_smt"}],
        "reviewer_key": "reviewer-1",
        "reviewed_at": "2026-08-22T17:00:00Z",
        "rationale": "The policy requirement addresses authorization but not all target procedures.",
        "confidence_score": {"category": "medium"}
      }
    ]
  }
}
```

Manifest paths resolve relative to the manifest directory. Output retains the reviewer-supplied `href` as a URI reference but does not dereference it. `artifact` and `resolved_catalog` are local processing paths and are not copied into OSCAL output.

### Output and Exit Contract

| Operation | Exit | Meaning |
|-----------|------|---------|
| `mapping build` | `0` | Inputs, manifest, and output are valid; gaps may exist and are reported |
| `mapping build` | `2` | A trustworthy artifact cannot be produced; no mapping output is written |
| `mapping check` / build with baseline policy | `0` | Analysis complete and selected review-impact policy is not triggered |
| `mapping check` / build with baseline policy | `1` | Analysis complete and human review is required |
| `mapping check` / build with baseline policy | `2` | Analysis incomplete because input, baseline, or validation failed |

---

## Acceptance Criteria :green_circle: `@llm-autonomous`

| ID | Traces To | Given | When | Then |
|----|-----------|-------|------|------|
| AC-1 | M-1, M-19 | Two valid v1.2.3 Catalogs and a valid manifest | Building the mapping | A schema-valid `mapping-collection` JSON artifact is emitted |
| AC-2 | M-11 | A map references a missing source control | Building the mapping | The command exits `2`, identifies the manifest path and source ID, and writes no artifact |
| AC-3 | M-11 | A `statement` item references a guidance part | Building the mapping | The reference is rejected as the wrong semantic type |
| AC-4 | M-12, M-14 | One source maps to three targets using `subset-of` | Building the mapping | One map with one source and three targets is emitted in the original direction |
| AC-5 | M-13 | A standard relationship is misspelled or a confidence percentage is `1.1` | Building the mapping | Validation fails with allowed values/range and no artifact is written |
| AC-6 | M-7, M-8, M-16 | Reviewer, review time, method, rationale, and parties are present | Inspecting output | Standard provenance plus namespaced per-map review evidence are present and resolvable |
| AC-7 | M-15 | Three target controls are absent from every map | Building the mapping | They appear as gaps only; no `no-relationship` maps are generated |
| AC-8 | M-20, M-21 | Seven of ten target controls occur in explicit maps | Producing the report | It shows `7/10` and `70%` participation plus three sorted unmapped IDs, not 70% compliance |
| AC-9 | M-27 | A reviewer supplies low confidence | Building and checking | Low confidence is preserved but does not fail, approve, reorder, or alter gap membership |
| AC-10 | M-5 | A Profile is supplied without a resolved companion | Building the mapping | The command exits `2` with resolution guidance and does not invoke `oscal-cli` |
| AC-11 | M-4 | A resource `href` is HTTPS and a local artifact is supplied | Building the mapping offline | The local file is used, the href is preserved, and no network request occurs |
| AC-12 | M-18, M-22 | The same files and manifest are built twice in different working directories | Comparing bytes | UUIDs, ordering, JSON, and reports are byte-identical and contain no absolute canonical paths |
| AC-13 | M-23 | A baseline-referenced ID disappears from a revised target | Running baseline check | `stale_reference` is reported and no replacement is inferred |
| AC-14 | M-23 | A referenced ID remains but its canonical element content changes | Running baseline check | `subject_changed` reports old/new fingerprints while retaining the stable map UUID |
| AC-15 | M-23 | A new unmapped target control appears | Running baseline check | `new_gap` is reported separately from existing gaps and relationship changes |
| AC-16 | M-24 | A baseline lacks required stable-key properties or is not Mapping JSON | Running baseline check | Analysis exits `2` rather than performing positional comparison |
| AC-17 | M-25 | Output aliases a source artifact or report aliases output | Building the mapping | The request is rejected before any file is modified |
| AC-18 | M-26 | A deeply nested or oversized malicious artifact is supplied | Building the mapping | Processing stops at a documented bound without panic or partial output |
| AC-19 | M-28 | Existing command fixtures | Running the full suite | Existing command outputs and exit contracts remain unchanged |
| AC-20 | M-2 | PRD 054's verified v1.2.3 baseline and the vendored Mapping schema | Running the standards-provenance gate | The Mapping schema resolves to the same pinned OSCAL release and checksum process, with no moving URL or second baseline |
| AC-21 | M-3 | An XML or YAML Catalog/Profile resource or a non-JSON output request | Building the mapping | The command exits `2` with JSON-only guidance and writes no mapping artifact |
| AC-22 | M-6 | A manifest with an unknown key, duplicate decoded key, unsupported schema version, invalid Unicode, or an exceeded bound | Parsing the manifest | The command exits `2`, identifies the bounded manifest error, and reads no framework resource |
| AC-23 | M-9 | A resource with the wrong declared type/root UUID or an `expected_sha256` mismatch | Validating inputs | The command exits `2` before subject inventory or output and identifies the mismatched expectation |
| AC-24 | M-10 | A Catalog or resolved Profile companion with duplicate or type-ambiguous eligible IDs | Building the subject inventory | The command fails deterministically rather than selecting a last-seen subject |
| AC-25 | M-17 | Valid Catalog and Profile resources with a resolved companion | Inspecting the emitted mapping and report | Source/target hashes, root UUIDs, document/OSCAL versions, and resolved-companion hash are present under the documented namespace |

---

## Technical Constraints :yellow_circle: `@human-review`

- **Language:** Rust edition 2024 on stable 1.93.0 or the repository's approved successor.
- **Architecture:** Extend the existing crate with dedicated mapping model, manifest, inventory, validation, report, and CLI modules; do not introduce a service or database.
- **Dependencies:** Reuse `serde`, `serde_json`, `jsonschema`, `sha2`, `uuid`, tracing, existing I/O bounds, and output helpers. Any new dependency requires documented justification.
- **Schema:** After PRD 054 establishes the v1.2.3 baseline, embed the matching official Mapping JSON schema in this feature and record its upstream source/version. Do not load schemas from the network at runtime.
- **Serialization:** Typed serialization only. No JSON construction by string concatenation and no untyped pass-through that bypasses field constraints.
- **Hashing:** Raw artifact SHA-256 identifies file bytes. Canonical element SHA-256 uses a documented JSON canonicalization that excludes FORGE fingerprint props to avoid self-reference.
- **UUIDs:** Use the existing deterministic UUID namespace utility where compatible. Seed schemas must be versioned and must not include collection order, absolute paths, current time, or confidence.
- **Ordering:** Sort resources by stable mapping key, maps by UUID, subjects by `(type, id-ref)`, gaps by identifier, and report findings by stable severity/code/path order.
- **Validation:** Perform manifest validation, resource schema validation, inventory/reference checks, typed construction, then completed Mapping schema validation. Collect independent errors where safe.
- **No network/process:** Mapping commands do not use HTTP or invoke `oscal-cli`; Profile companions are explicit inputs.
- **Quality gates:** Test first. `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` must pass.

---

## Security, Privacy, Licensing, and Provenance :yellow_circle: `@human-review`

| Risk | Impact | Required Mitigation |
|------|--------|---------------------|
| False authority | Consumers may treat a crosswalk as a compliance determination | Require human provenance and rationale; label mappings as claims; publish limitations in help/report output |
| Confidence laundering | A numeric score may appear objective or auto-approved | Preserve as author estimate; never threshold, approve, rank, or substitute it for review |
| Stale framework version | Valid IDs may point to materially revised requirements | Fingerprint exact resources and elements; baseline checks flag changed content and versions |
| Restricted framework content | Bundling or excerpting licensed standards may violate terms | Ship no third-party framework content; accept user-supplied files; output IDs/hashes by default; require users to verify rights |
| Sensitive control prose | Reports could disclose internal requirements or security design | Default machine reports to IDs/hashes and bounded metadata; make excerpts opt-in and document report sensitivity |
| Reviewer personal data | Names/emails may be copied into durable OSCAL artifacts | Require only a display name or organization identifier; make contact fields optional; document retention and sharing implications |
| Reviewer impersonation | A manifest may name a person who did not approve it | State that FORGE preserves but does not authenticate reviewer identity; signatures remain future work |
| SSRF/data exfiltration | OSCAL resources can contain remote links | Never dereference links or imports; operate on explicit local files only |
| Path disclosure | Canonical paths can expose usernames or repository layout | Output reviewer-supplied hrefs and relative labels, never canonical absolute processing paths |
| Parser/resource exhaustion | Large arrays, deep JSON, or duplicate-heavy content can consume resources | Bound bytes, depth, collection sizes, string lengths, and error counts; reject duplicates early |
| Terminal injection | Untrusted titles/rationale may manipulate terminal output | Escape control characters and render ANSI/bidirectional controls inert |
| Accidental overwrite | Output could replace evidence inputs | Reject aliases and use existing safe-write behavior |

The feature provides integrity checks, not authorization, confidentiality, digital signatures, or legal permission to use source content. Mapping provenance identifies declared responsibility; it does not prove identity or ownership.

---

## Dependencies and Interactions :yellow_circle: `@human-review`

- **Blocking:** PRD 054 must land its OSCAL v1.2.3 baseline, constants, compatibility fixtures, and release-pinned provenance. PRD 055 separately owns Mapping schema embedding, model detection, and validation.
- **Requires:** Existing local JSON I/O limits, safe output behavior, serde/jsonschema validation, UUID and SHA-256 utilities, and CLI error/report conventions.
- **Reuses:** Catalog recursive traversal concepts, Profile detection, traceability naming, deterministic sorting from `forge diff`, and stable-ID/change-impact terminology from PRD 053.
- **Profile dependency:** Existing `forge resolve` remains the path to create required resolved companions, but mapping commands never invoke it implicitly.
- **Configuration interaction:** `.forge.toml` may later provide output/report defaults; explicit command flags retain precedence and this feature must work without project config.
- **GitHub Action interaction:** A future Action may run mapping checks, but Action YAML, annotations, and repository discovery remain separately scoped.
- **Does not require:** A native resolver, a database, telemetry, framework downloads, assessment-layer models, or a plugin system.

---

## Risks and Mitigations :yellow_circle: `@human-review`

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|------------|--------|------------|
| R-1 | Teams expect automatic crosswalk creation and find a manifest too manual | High | Medium | Position as validation/publishing first; measure reviewer workflow before considering suggestions |
| R-2 | Profile companion does not actually correspond to the supplied Profile | Medium | High | Fingerprint both; require attestation; document limitation; consider oscal-cli provenance verification later |
| R-3 | Participation percentage is mistaken for semantic/control coverage | High | High | Use `review_participation` labels, show numerator/denominator, omit standard coverage by default |
| R-4 | Mapping relationship direction is reversed during authoring | Medium | High | Repeat source-to-target wording in manifest errors/reports and add asymmetric subset/superset fixtures |
| R-5 | Resource ID churn invalidates large crosswalks | High | High | Stable map keys/UUIDs, element fingerprints, baseline impact report, and no guessed successors |
| R-6 | Mapping schema is new and changes in later OSCAL releases | Medium | Medium | Pin 1.2.3, isolate typed model, keep manifest version independent, add official-schema fixtures |
| R-7 | Per-map provenance needs exceed OSCAL native fields | Medium | Medium | Use a documented FORGE namespace and retain standard responsible parties at provenance scope |
| R-8 | Many-to-many maps obscure precise relationships | Medium | Medium | Preserve cardinality but encourage statement-level granularity and require rationale; report large sets |
| R-9 | Licensed content leaks through optional excerpts | Medium | High | Default to IDs/hashes, bound excerpts, avoid bundled fixtures from restricted standards |

---

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Hypothesis | Success Threshold | Measurement |
|------------|-------------------|-------------|
| H-1: Practitioners can publish a mapping without maintainer help | At least 4 of 5 pilots complete a 25-map task | Moderated task completion with sanitized/licensed inputs |
| H-2: Invalid claims are blocked reliably | 100% of seeded reference/type/cardinality/vocabulary failures rejected | Contract, golden, and adversarial tests |
| H-3: Output interoperates with the standard | 100% schema pass rate and parse by one independent OSCAL tool | Release-pinned schema plus external-tool fixture run |
| H-4: Review scope is understandable | At least 4 of 5 pilots distinguish participation from compliance coverage | Short comprehension test |
| H-5: Change impact is complete | 100% of seeded stale, changed, new-gap, and relationship-change cases found | Human-adjudicated baseline fixtures |
| H-6: Workflow is faster | Median publish time at least 40% below partner spreadsheet baseline | Within-participant timed comparison |
| H-7: Teams maintain and reuse mappings | Three repositories complete two revisions; two consume output outside FORGE within 90 days | Opt-in partner evidence; no telemetry required |

### Technical Quality Gates

- 100% Must Have requirement coverage by executable tests.
- Byte-for-byte deterministic JSON/report fixtures on macOS, Linux, and Windows.
- Official schema validation for every golden Mapping artifact.
- 100% seeded invalid-reference and cardinality rejection.
- Zero network requests and zero external process invocations in mapping tests.
- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` pass.

---

## Rollout and Phasing :yellow_circle: `@human-review`

### Phase 0 — Standards and contract lock

- Complete PRD 054 and pin the official v1.2.3 Mapping schema.
- Approve manifest v1, FORGE property namespace, UUID seed version, coverage terminology, limits, and a legally redistributable synthetic fixture set.

### Phase 1 — Catalog-to-Catalog core

- Deliver typed Mapping serialization, reviewer/provenance preservation, control/statement inventories, validation, stable UUIDs, gaps, deterministic reports, and schema validation for one resource pair.

### Phase 2 — Profile companions and change impact

- Add Profile plus resolved-Catalog inputs, effective-inventory evidence, baseline validation, stable map comparison, CI report JSON, and exit-policy tests.

### Phase 3 — Design-partner release gate

- Run five observed tasks using partner-supplied, sanitized, lawfully usable Catalog/Profile resources.
- Measure completion, interpretation, invalid-reference detection, change-impact completeness, and time against current workflow.
- Release only after schema interoperability and all security/quality gates pass.

### Post-MVP Decision Points

- Add a read-only scaffold command if users struggle to author inventories by hand.
- Consider suggestion tooling only under a separate PRD with explicit evaluation, provenance, and mandatory human approval.
- Enable XML/YAML only when full semantic round-trip evidence exists for Mapping artifacts.
- Expand to multiple resource pairs only if real mappings require it and reports remain understandable.

---

## Implementation Decisions and Remaining Questions :yellow_circle: `@human-review`

- **Resolved implementation contract:** v1 requires a reviewer key, review time, and non-empty rationale on every map; mapping-level responsibility does not replace per-map evidence.
- **Resolved fingerprint contract:** hash the complete canonical eligible control/statement subtree after removing only FORGE-generated `subject-sha256` properties.
- **Resolved bounds:** manifest 2 MiB; resources/baselines 50 MiB; 100 reviewers; 10,000 maps; 100 subjects per side; 100 qualifiers per map; depth 64; 64 KiB strings; 100 schema errors; 10,000 impact findings; 1,000 optional excerpts.
- **Resolved same-resource behavior:** identical source/target bytes are allowed for deliberate same-framework mappings and are covered by a contract test.
- **Resolved scaffold scope:** `mapping init` is included and emits inventories, fingerprints, empty maps, and no relationship claims.
- **Resolved report privacy default:** default reports omit reviewer names and prose; `--include-excerpts` is an explicit sensitive-output opt-in.
- **Resolved documentation:** usage guidance states that FORGE does not authenticate reviewer identity/authority and that users remain responsible for framework-content rights.
- **[Engineering, non-blocking]** Can resolved-Profile provenance from the current `oscal-cli` output be verified strongly enough to remove the explicit attestation limitation later?

---

## Definition of Ready :red_circle: `@human-required`

- [x] PRD 054's OSCAL v1.2.3 compatibility gate is complete.
- [ ] Product and Compliance approve the human-review, participation, gap, and no-compliance-verdict language.
- [ ] Engineering approves manifest v1, the stable-key/UUID seed contract, subject fingerprint fields, and resource limits.
- [ ] Security and Legal review reviewer-data handling, local-file boundaries, and framework licensing guidance.
- [x] The official v1.2.3 Mapping schema and its provenance/checksum record are verified.
- [x] A synthetic, redistributable Catalog-to-Catalog fixture set covers every supported relationship and cardinality.
- [ ] At least three design partners agree to evaluate the manifest workflow with lawfully usable inputs.
- [x] Every Must Have requirement maps to an executable acceptance scenario or release-gate test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-22 | Require reviewer-authored relationships and rationale | FORGE must preserve accountable human judgment rather than manufacture compliance claims | Automated semantic mapping; confidence-based approval |
| 2026-08-24 | Lock manifest v1 implementation defaults | Per-map review evidence, complete-subtree fingerprints, explicit limits, and same-resource support provide a deterministic auditable contract | Mapping-level-only reviewer; field allowlist fingerprints; unbounded inputs; rejecting identical resources |
| 2026-08-24 | Include scaffold/check fast follows in the technical implementation | They make the manifest authorable and baseline impact enforceable without adding relationship suggestions | Defer both commands to a later release |
| 2026-08-24 | Use the already-locked `same-file` crate for destination alias checks | Stable Rust does not expose portable hard-link identity, while Mapping must reject aliases consistently on Unix and Windows | Unix-only metadata checks; unstable Windows metadata APIs; path-only canonicalization |
| 2026-08-22 | Limit MVP to one source/target resource pair | Proves the authoring, validation, provenance, and reporting contract before multi-pair orchestration | Multiple mappings per collection in v1 |
| 2026-08-22 | Require resolved Catalog companions for Profile inputs | Effective Profile subjects cannot be validated safely without resolution, and native resolution is out of scope | Partial local resolver; unchecked Profile references |
| 2026-08-22 | Report review participation instead of compliance coverage | Map presence and absence do not establish implementation, effectiveness, or compliance | Treat mapped percentage as coverage |
| 2026-08-22 | Preserve grouped cardinality and source-to-target direction | Flattening or reversing relationships would change the reviewer's claim | Expand every group into pairs; canonical side swapping |
| 2026-08-22 | Use stable manifest keys to derive UUID v5 identifiers | Deterministic identity supports review and baseline impact without relying on order or prose | UUID v4; content-only identities |
| 2026-08-22 | Ship JSON-only, local, and offline | Narrows serializer risk and prevents remote content or credential handling in the MVP | XML/YAML output; remote framework retrieval |
| 2026-08-22 | Block implementation on PRD 054 | Mapping must share one verified OSCAL v1.2.3 baseline and provenance process | Independently vendor another standards baseline |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-22 | Codex | Initial draft for human-reviewed OSCAL Control Mapping and deterministic change-impact reporting |

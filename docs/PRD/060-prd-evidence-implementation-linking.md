# 060-prd-evidence-implementation-linking

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `060-evidence-implementation-linking`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will create a deterministic local linkage index connecting policy/framework control subjects to implementation statements and evidence references. The MVP validates exact subject identities, records owners and collection metadata, fingerprints local evidence without copying its contents, and reports missing, stale, or expiring links. It deliberately does not judge evidence sufficiency, test control effectiveness, collect credentials, execute checks, or emit OSCAL Assessment Results.

## Context

### Background :red_circle: `@human-required`

FORGE can generate Catalogs, Component Definitions, Profiles, Assessment Plan scaffolding, and SSP templates, but teams still maintain the connection from policy requirement to implementation and supporting evidence in spreadsheets or GRC forms. Those links often lose the exact control version, evidence hash, collection time, owner, or implementation context, making later assessment preparation expensive and difficult to reproduce.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | Catalog controls, Component Definition implemented requirements, and SSP templates already expose stable OSCAL identifiers. | Linkage should validate existing subjects rather than create parallel uncontrolled IDs. |
| Repository evidence | FORGE supports back matter, citations, hashes, traceability, and deterministic reports. | Evidence references can be indexed without ingesting or evaluating sensitive content. |
| Standards boundary | Evidence references are not assessment observations, findings, risks, or effectiveness conclusions. | Assessment Results and POA&M remain separate future workflows. |
| Product hypothesis | A trustworthy link index reduces assessment preparation and reveals evidence-maintenance gaps. | Measure verified-link completion and revalidation time, not uploaded file count. |

No evidence repository study, assessor validation, production corpus, or proof of time savings was supplied. Metrics in this PRD are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- One or more local policy/framework OSCAL Catalog or resolved Profile subjects
- One local OSCAL Component Definition or SSP implementation artifact per linkage project
- A bounded reviewer-authored `forge.linkage/1` manifest
- Explicit many-to-many links among requirement subjects, implementation subjects, and evidence references
- Local evidence files fingerprinted by SHA-256 without copying content into the output
- Explicit non-fetched URI references with reviewer-supplied metadata and optional expected hashes
- Evidence metadata: stable key, title, type, owner, collected-at, valid-through, sensitivity label, and source label
- Implementation metadata: stable link key, responsible role, implementation status assertion, rationale, and review attribution
- Deterministic JSON linkage index plus text/JSON completeness, freshness, and stale-reference reports
- Baseline comparison for changed subjects, implementation text, evidence hashes, expiry, additions, and removals

**Out of Scope:**

- Evidence sufficiency, authenticity, admissibility, quality, effectiveness, or compliance judgments
- Executing tests, querying cloud providers, taking screenshots, collecting credentials, or following URIs
- Uploading, embedding, encrypting, redacting, or retaining evidence content
- OSCAL Assessment Results, findings, risks, attestations, POA&M, or continuous-control-monitoring output
- Automatic link suggestions, implementation generation, or evidence classification
- Hosted storage, databases, web UI, RBAC, signatures, notifications, or external connectors

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/008-prd-citation-extraction.md` | Reference extraction and back-matter patterns |
| `docs/PRD/012-prd-back-matter.md` | OSCAL resource/link handling |
| `docs/PRD/014-prd-component-definition-structure.md` | Component Definition model |
| `docs/PRD/015-prd-component-implemented-requirements.md` | Implemented-requirement identifiers |
| `docs/PRD/041-prd-assessment-plan-controls.md` | Assessment scope scaffolding |
| `docs/PRD/045-prd-ssp-template-structure.md` | SSP implementation-layer structure |
| `docs/PRD/055-prd-control-mapping.md` | Policy/framework relationship provenance |
| `docs/PRD/057-prd-framework-change-impact-monitoring.md` | Upstream changes that may stale links |

---

## Problem Statement :red_circle: `@human-required`

Compliance engineers and auditors need to trace an approved policy or framework requirement to the exact implementation statement and evidence reference used during review. Without a versioned, validated linkage index, teams repeatedly reconstruct those relationships and may unknowingly rely on missing, changed, expired, or wrong-version material.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Validate end-to-end link identity. | Every accepted link resolves all declared requirement and implementation IDs against exact fingerprinted artifacts. |
| G-2 | Preserve evidence provenance without copying content. | Every local evidence reference records stable key, relative label, SHA-256, size, collection metadata, and owner while output contains no evidence bytes. |
| G-3 | Surface maintenance gaps truthfully. | 100% of seeded missing, changed, expired, unowned, and stale references are reported without sufficiency or compliance language. |
| G-4 | Make linkage reproducible. | Identical inputs and explicit `--as-of` date produce byte-identical JSON and stable finding IDs. |
| G-5 | Reduce assessment preparation effort. | Five design partners reduce time to validate 50 requirement-to-evidence chains by 40%. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- A link means a reviewer associated records; it does not prove implementation or control effectiveness.
- A current hash does not prove evidence authenticity, completeness, or admissibility.
- FORGE does not inspect evidence content or decide whether it supports a requirement.
- The MVP does not perform assessments or generate Assessment Results/POA&M.
- The MVP does not retrieve evidence from external systems or manage secrets.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Link requirements to implementations (P0)

> As a compliance engineer, I want reviewed requirement-to-implementation links so that policy intent can be followed into the system description.

### US-2 — Attach exact evidence references (P0)

> As a control owner, I want evidence files or URIs fingerprinted and attributed so that reviewers know exactly what material was referenced.

### US-3 — Reject stale or wrong-version subjects (P0)

> As an auditor, I want every identifier checked against exact artifact versions so that the linkage index cannot silently point to obsolete controls or implementations.

### US-4 — Find missing and expiring links (P0)

> As a compliance engineer, I want a maintenance queue for unlinked implementations and stale or expiring evidence so that assessment preparation is predictable.

### US-5 — Detect linkage impact after changes (P1)

> As a policy owner, I want baseline comparison to show which links require review after policy, implementation, or evidence changes.

### US-6 — Consume the index in automation (P1)

> As a DevSecOps engineer, I want versioned JSON and stable exit statuses so that publication can require complete, current link metadata.

## Linkage Model :yellow_circle: `@human-review`

### Subject Types

| Side | MVP Subject | Validation Source |
|------|-------------|-------------------|
| Requirement | Catalog control or statement; effective Profile control or statement | Catalog or caller-supplied resolved Profile companion |
| Implementation | Component Definition implemented requirement or SSP implemented requirement/statement with stable ID | Schema-valid local implementation artifact |
| Evidence | Local regular file or explicit non-fetched absolute URI | File metadata/hash or reviewer declaration |

Each link contains at least one requirement subject and one implementation subject. Evidence is optional at link creation so missing-evidence work can be represented, but `evidence-required: true` makes absence a gated finding. Many-to-many cardinality remains explicit in one stable link object.

### Assertions and Status

Implementation status values (`planned`, `partial`, `implemented`, `not-applicable`, `unknown`) are reviewer assertions. `implemented` never changes evidence sufficiency, assessment, or compliance status. `not-applicable` requires rationale and reviewer evidence and is not derived from PRD 056 applicability.

Evidence freshness is metadata-only:

- `current`: local hash matches and `valid-through` is after `--as-of`, when supplied;
- `expiring`: valid-through falls within a configured deterministic window;
- `expired`: valid-through is before `--as-of`;
- `changed`: local bytes differ from the approved hash;
- `unavailable`: local path is missing or URI metadata cannot be locally verified;
- `unverified-uri`: a URI is recorded but never fetched.

These states do not express evidentiary quality.

### Privacy Defaults

The linkage index contains IDs, hashes, bounded metadata, and reviewer-supplied labels only. It excludes evidence bytes, excerpts, canonical absolute paths, credentials, query strings, and URI fragments. Text reports redact URI user-info and query data.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge linkage init`, `build --manifest <FILE>`, and `check --manifest <FILE>` with text/JSON output and explicit `--as-of`.
- [ ] **M-2 — Closed manifest:** Parse bounded `forge.linkage/1` JSON; reject unknown/duplicate keys, unsupported versions, invalid Unicode, and exceeded limits.
- [ ] **M-3 — Artifact validation:** Schema-validate requirement and implementation artifacts, require Profile companions, and fingerprint all exact input bytes/root identities.
- [ ] **M-4 — Inventories:** Recursively inventory eligible requirement and implementation subjects; reject missing, duplicate, wrong-side, wrong-type, or ambiguous IDs.
- [ ] **M-5 — Link cardinality:** Require unique stable link keys, one or more requirement subjects, one or more implementation subjects, and unique subjects per side.
- [ ] **M-6 — Review evidence:** Require reviewer key, review time, and non-empty rationale for each link and each `not-applicable` assertion.
- [ ] **M-7 — Local evidence:** Accept only bounded regular files under declared roots; reject traversal, unsafe symlinks, devices, FIFOs, sockets, aliases, and path disclosure.
- [ ] **M-8 — URI evidence:** Accept only explicit absolute `https` or organization-approved custom-scheme URIs; never resolve or fetch them; strip credentials/query/fragment from default reports.
- [ ] **M-9 — Evidence metadata:** Require stable key, title, evidence type, owner, collection time, sensitivity label, and either local expected hash or explicit URI-unverified status.
- [ ] **M-10 — Fingerprints:** Compute SHA-256 and byte size for local evidence, compare expected values, and report changes without reading more than configured bounds.
- [ ] **M-11 — Freshness:** Derive current/expiring/expired status only from explicit metadata and `--as-of`; never use hidden wall-clock time.
- [ ] **M-12 — Missing evidence:** Preserve links with no evidence as explicit gaps; never invent or auto-discover evidence.
- [ ] **M-13 — Output:** Emit deterministic `forge.linkage-index/1` JSON and text/JSON reports containing provenance, link graph, gap/freshness findings, and stable reason codes.
- [ ] **M-14 — Baseline:** Compare a prior linkage index to current artifacts/evidence and distinguish removed subjects, changed content, changed evidence, expiry, additions, removals, and relationship edits.
- [ ] **M-15 — Stable identity:** Derive UUID v5 link/finding IDs from versioned schemas and stable keys, never from paths, array order, current time, or evidence prose.
- [ ] **M-16 — No assessment claims:** Generated terminology and schemas must not label links sufficient, effective, passed, compliant, certified, or audit-ready.
- [ ] **M-17 — Safety/privacy:** Operate offline, use safe atomic outputs, reject aliases, bound resources, escape terminal text, and emit no evidence content or absolute paths.
- [ ] **M-18 — Exit contract:** Exit `0` when the selected deterministic gate has no findings, `1` for valid maintenance/action findings, and `2` for invalid analysis.
- [ ] **M-19 — Tests:** Cover every subject type, cardinality, file hazard, URI rule, freshness boundary, stale change, privacy redaction, determinism, and terminology guardrail.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Aggregate explicitly supplied linkage projects into an owner/evidence maintenance queue.
- [ ] **S-2:** Link PRD 057 framework-impact finding IDs and PRD 058 policy versions without transferring approval.
- [ ] **S-3:** Emit an OSCAL-compatible back-matter overlay only after schema-valid, lossless round-trip behavior is demonstrated for each supported model.
- [ ] **S-4:** Export a static HTML trace view from requirement through implementation to evidence metadata.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Authenticated evidence connectors with least-privilege credential handling under separate security PRDs.
- [ ] **C-2:** Assessment Results generation from explicit assessor observations and findings.
- [ ] **C-3:** Automated evidence collection and continuous control monitoring with per-collector threat models.
- [ ] **C-4:** Web evidence review backed by the same local linkage contracts.

### Won't Have (W) — This release :red_circle: `@human-required`

- Evidence ingestion/storage, external retrieval, credential handling, automated tests, sufficiency scoring, effectiveness judgments, Assessment Results, POA&M, or compliance claims.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Valid requirement and implementation IDs plus one local evidence file | Build runs | A deterministic link records all exact artifact and evidence hashes without evidence bytes |
| AC-2 | A requirement ID exists only in a different framework version | Build runs | Exit is `2` and no linkage index is written |
| AC-3 | Evidence changes by one byte after approval | Check runs | A `changed` finding shows old/new hashes and requires review without judging sufficiency |
| AC-4 | A URI contains credentials and a query string | Report runs | The stored contract follows policy and default text output exposes neither credentials nor query values |
| AC-5 | Evidence expires on the explicit `--as-of` date | Check runs | The documented boundary classification is stable and tested |
| AC-6 | A FIFO or symlink escape is supplied as local evidence | Build runs | It is rejected before content is read or output is modified |
| AC-7 | Identical inputs run in different directories | Outputs are compared | JSON bytes match and contain no canonical absolute paths |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Link validation | 100% seeded wrong-ID/version/hash cases rejected | Automated fixtures |
| Leading | Task completion | 4 of 5 partners build 50 valid chains without maintainer edits | Moderated pilot |
| Leading | Interpretation | 5 of 5 partners distinguish current hash from sufficient evidence | Post-task questions |
| Lagging | Preparation time | 40% median reduction to validate 50 chains | Within-participant comparison |
| Lagging | Maintenance | Three partners complete a second evidence refresh using baseline impact within 90 days | Opt-in partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** Stable OSCAL subject inventories, safe local I/O, SHA-256 utilities, deterministic reports, and PRD 054 schema baseline.
- **Phase 1:** Catalog-to-Component Definition links, local file evidence, index/report, freshness, and baseline.
- **Phase 2:** Profile companions, SSP implementation subjects, URI references, portfolio queue, and impact links.
- **Phase 3:** Design-partner assessment-preparation trials and independent OSCAL-model review.
- **Future boundary:** Assessment Results and authenticated connectors require separate PRDs and threat models.

## Security, Privacy, and Legal Risks :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Evidence contains secrets or personal data | Confidentiality/privacy harm | Never copy content; hashes/labels only; sensitivity metadata and clear output warnings |
| Path traversal or special files leak data/hang processing | Host compromise or denial of service | Root containment and open-time regular-file validation with bounded reads |
| URI retrieval causes SSRF or credential exposure | Network/data exposure | Never fetch; restrict schemes; redact user-info/query/fragment |
| Hash is mistaken for authenticity | False assurance | State that SHA-256 detects byte change but does not prove origin or custody |
| Link is mistaken for evidence sufficiency | False audit conclusion | Mandatory terminology guardrail and user comprehension gate |
| Framework/policy prose leaks into reports | Licensing/confidentiality harm | IDs/hashes by default; no excerpts in MVP |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Is a FORGE-native linkage index acceptable for MVP, or must launch include an OSCAL back-matter representation?
- **[Engineering, blocking]** Which exact Component Definition and SSP identifier classes are stable enough for the first implementation inventory?
- **[Security, blocking]** Should URI evidence be `https`-only in MVP, with custom schemes deferred?
- **[Compliance, blocking]** Which evidence metadata fields are necessary to make a link useful without implying sufficiency?
- **[Legal, non-blocking]** What retention and sharing warning should accompany hashes and sensitive evidence labels?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves the native linkage-index MVP and assessment boundary.
- [ ] Compliance approves implementation-status and evidence-freshness semantics.
- [ ] Engineering approves subject inventories, link schema, hashing, limits, and baseline behavior.
- [ ] Security approves local-file and URI trust boundaries.
- [ ] Legal/privacy reviews metadata, path, label, and hash handling.
- [ ] Three design partners provide sanitized evidence workflows.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Index evidence references without storing content | Minimizes sensitive-data exposure while preserving change detection | Evidence repository |
| 2026-08-24 | Separate linkage from assessment | Association and freshness do not establish sufficiency or effectiveness | Generate Assessment Results immediately |
| 2026-08-24 | Require exact subject/resource identity | Prevents wrong-version links from appearing valid | Best-effort ID lookup |
| 2026-08-24 | Use explicit `--as-of` for freshness | Makes time-dependent reports reproducible | Hidden wall-clock time |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for requirement, implementation, and evidence linkage |

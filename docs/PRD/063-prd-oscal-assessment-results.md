# 063-prd-oscal-assessment-results

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `063-oscal-assessment-results`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will turn explicit assessor-authored observations, findings, risks, reviewed evidence references, and an exact Assessment Plan/System Security Plan context into schema-valid OSCAL Assessment Results. The MVP validates and packages human judgments; it does not execute tests, infer findings, score effectiveness, certify compliance, or create remediation plans.

## Context

### Background :red_circle: `@human-required`

FORGE generates Assessment Plan scaffolding and SSP templates but stops before the assessment layer. PRD 060 can index implementation and evidence relationships without making assessment claims. Assessors still need a deterministic, standards-native way to record what was examined, what was observed, which controls were affected, and who made each judgment.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | FORGE already validates OSCAL artifacts and models assessment scope, subjects, implementations, and evidence links. | Assessment Results should reference exact upstream artifacts rather than duplicate their identity. |
| Trust boundary | Observations and findings are professional human judgments. | Every conclusion needs assessor provenance and must originate in the manifest. |
| Product hypothesis | Standards-native results reduce report assembly and improve handoff into remediation. | Validate with assessors and downstream tool interoperability. |

No assessor interviews, OSCAL Assessment Results corpus, independent-tool round trip, or schema implementation spike was supplied. Targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Release-pinned official OSCAL Assessment Results JSON schema and typed model
- One local Assessment Plan plus its referenced local SSP/Profile/Catalog companions as required
- Optional PRD 060 linkage index for evidence identity only
- Reviewer-authored observations, findings, risks, methods, subjects, dates, and responsible parties
- Exact control/objective references validated against the supplied assessment context
- Deterministic IDs, provenance, back-matter references, schema validation, and text/JSON summary
- Baseline comparison for result revisions and stale upstream references

**Out of Scope:**

- Test execution, evidence retrieval, automatic findings, risk calculation, or effectiveness scoring
- Compliance certification, auditor signatures, legal attestation, or assessor identity authentication
- Evidence content storage or redaction
- POA&M generation; PRD 064 owns remediation planning
- XML/YAML until JSON round-trip fidelity is independently demonstrated

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/041-prd-assessment-plan-controls.md` | Assessment scope controls |
| `docs/PRD/042-prd-assessment-plan-subjects.md` | Assessment subject model |
| `docs/PRD/045-prd-ssp-template-structure.md` | System implementation context |
| `docs/PRD/054-prd-oscal-1-2-3-compatibility.md` | Release-pinned schema provenance |
| `docs/PRD/060-prd-evidence-implementation-linking.md` | Evidence and implementation references |
| `docs/PRD/064-prd-oscal-poam-workflow.md` | Remediation workflow consuming reviewed risks/findings |

---

## Problem Statement :red_circle: `@human-required`

Assessors need to publish findings in a machine-readable form that preserves scope, evidence references, affected controls, and responsible judgment. Manual reports and spreadsheets weaken identity and provenance, while automatically generated findings would overstate what FORGE can know from policy and evidence metadata alone.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Produce interoperable Assessment Results. | 100% of accepted outputs validate against the pinned official schema and parse in one independent tool. |
| G-2 | Preserve accountable human judgment. | Every observation, finding, and risk records assessor party, time, method, rationale, and exact subject references. |
| G-3 | Prevent stale assessment context. | 100% of seeded wrong-version, missing, wrong-type, or changed upstream references are blocked or explicitly flagged. |
| G-4 | Keep revisions deterministic. | Identical manifests and inputs produce byte-identical IDs, ordering, JSON, and summaries. |
| G-5 | Reduce report assembly time. | Five assessors create a 25-finding result package 40% faster than their existing process. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- FORGE does not decide whether a control passed or failed.
- FORGE does not authenticate assessor identity or provide an audit opinion.
- Evidence linkage does not establish sufficient or appropriate audit evidence.
- Risk severity remains an explicit human assertion, not a FORGE calculation.
- The MVP does not create remediation milestones or POA&M status.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Record an assessment result (P0)

> As an assessor, I want to publish reviewed observations, findings, and risks against exact scope so that downstream tools can consume my conclusions without reinterpretation.

### US-2 — Validate every reference (P0)

> As an auditor, I want affected controls, objectives, subjects, implementations, and evidence checked against exact artifacts so that no dangling claim is published.

### US-3 — Preserve methods and responsibility (P0)

> As a report reviewer, I want to see who declared each result and how it was assessed so that I can evaluate the basis of the conclusion.

### US-4 — Compare revised results (P1)

> As an assessment lead, I want stable identities and revision impact so that reassessment changes remain distinguishable from reordered data.

## Product Guardrails :red_circle: `@human-required`

1. Every assessment claim originates in explicit assessor input.
2. FORGE validates structure and references, not professional sufficiency or correctness.
3. Missing evidence never becomes a failed control automatically.
4. Numeric severity/confidence is preserved but never auto-promotes, suppresses, or closes a finding.
5. Output identifies exact source bytes and trust limitations.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [x] **M-1 — Command:** Provide `forge assessment results build --manifest <FILE>` with output/report/baseline options.
- [x] **M-2 — Standards baseline:** Vendor the official release-matched Assessment Results JSON schema with recorded URL, release, and checksum.
- [x] **M-3 — Closed manifest:** Parse bounded `forge.assessment-results/1` JSON; reject unknown/duplicate keys, unsupported versions, and exceeded limits.
- [x] **M-4 — Context validation:** Validate Assessment Plan and required companion artifacts, their types, root UUIDs, versions, OSCAL versions, and hashes.
- [x] **M-5 — Subject inventory:** Validate every control, objective, subject, implementation, task, and evidence reference supported by the MVP.
- [x] **M-6 — Human provenance:** Require assessor party, role, assessment time/range, method, and non-empty rationale for each conclusion-bearing object.
- [x] **M-7 — Explicit relationships:** Preserve observation-to-finding-to-risk relationships exactly; reject missing, duplicate, circular, wrong-side, or wrong-type references.
- [x] **M-8 — Evidence boundary:** Reference PRD 060 evidence keys/hashes without copying content or treating link presence as sufficiency.
- [x] **M-9 — Typed model:** Construct Assessment Results through typed Rust structures and validate the completed JSON against the pinned schema.
- [x] **M-10 — Stable IDs:** Derive UUID v5 IDs from immutable reviewer keys and versioned seed schemas, never array order, prose, or wall-clock time.
- [x] **M-11 — Determinism:** Canonically order parties, results, observations, findings, risks, links, and report findings.
- [x] **M-12 — Baseline:** Report additions, removals, content/rationale changes, status changes, stale subjects, and upstream fingerprint changes by stable identity.
- [x] **M-13 — No inferred verdicts:** Emit only assessor-declared status/severity and no generated pass/fail, compliance, effectiveness, or certification conclusion.
- [x] **M-14 — Safety/privacy:** Operate offline, bound all resources, use safe writes, escape terminal text, and omit excerpts/absolute paths by default.
- [x] **M-15 — Exit contract:** Exit `0` for valid build/check, `1` for valid baseline review actions, and `2` for invalid input/analysis.
- [x] **M-16 — Tests:** Cover every supported object/reference, stale context, provenance, invalid graph, determinism, privacy, and official-schema fixture.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [x] **S-1:** Scaffold a result manifest from an Assessment Plan without creating observations or findings.
- [x] **S-2:** Static HTML assessor/reviewer report from the same versioned model.
- [ ] **S-3:** Export selected reviewed risks into a PRD 064 POA&M scaffold without assigning owners or dates automatically.
- [ ] **S-4:** Support multiple result epochs when the schema and user workflow are validated.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Signed assessor attestations under a separate identity and cryptography design.
- [ ] **C-2:** Connector-imported evidence or test output with explicit review gates.
- [ ] **C-3:** XML/YAML after independent lossless round-trip evidence.

### Won't Have (W) — This release :red_circle: `@human-required`

- Test execution, auto-findings, risk scoring, auditor signatures, certification, POA&M mutation, or evidence storage.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Valid context and assessor-authored observation/finding/risk | Build runs | Schema-valid deterministic Assessment Results are emitted |
| AC-2 | A finding references a control absent from exact scope | Build runs | Exit is `2` and no artifact is written |
| AC-3 | Evidence is linked but the assessor supplies no conclusion | Build runs | No finding or pass status is invented |
| AC-4 | A severity value changes in a revision | Baseline runs | The reviewer-authored change is reported without automatic reprioritization |
| AC-5 | Identical inputs build twice | Outputs are compared | JSON and report bytes match |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Schema/reference validity | 100% | Official-schema and adversarial fixtures |
| Leading | Assessor task completion | 4 of 5 | Moderated pilot |
| Leading | Invented conclusions | Zero | Golden tests and human adjudication |
| Lagging | Report assembly time | 40% median reduction | Within-assessor comparison |
| Lagging | Interoperability | Successful use in two independent downstream workflows | Partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** PRDs 041/042, PRD 054 schema provenance, and stable input validation; PRD 060 is optional evidence context.
- **Phase 1:** Standards spike, Catalog/Assessment Plan scope, observations/findings/risks, JSON output.
- **Phase 2:** SSP/implementation/evidence references, baseline comparison, static report.
- **Phase 3:** Independent-tool interoperability and assessor pilots.

### Engineering Implementation Status :white_circle: `@auto`

The bounded JSON MVP is implemented on `forge.assessment-results/1`. It accepts
one Assessment Plan result epoch and exact local SSP, Profile, and Catalog
companions; the Profile subset is one explicit Catalog import using `with-ids`
or `include-all`/`exclude-controls` (wildcard matching is rejected). Supported
subjects are component, inventory item, location, party, user, and resource.
Optional PRD 060 input remains an identity-only `forge.linkage-index/1`
adapter. The output is built through dedicated typed structures, validated
against the pristine OSCAL 1.2.3 Assessment Results schema, and accompanied by
deterministic text, JSON, or static HTML review reports.

Executable coverage maps M-1/M-4/M-5/M-6/M-8/M-9/M-10/M-11/M-13/M-14/M-15
and AC-1 through AC-5 to `tests/assessment_results_test.rs`; M-2 to
`tests/schema_provenance_test.rs` and
`tests/oscal_1_2_3_compatibility_test.rs`; M-3/M-7 to manifest unit tests plus
the CLI adversarial cases; and M-12 to the baseline revision tests. The fixture
covers both target types, every supported subject type, SSP implementation and
Assessment Plan task references, optional evidence identity, explicit graph
edges, all three conclusion object types, deterministic bytes, privacy, stale
context, and independent validation against the official schema.

This is engineering completion, not release approval. Compliance/Legal
terminology approval, Engineering product approval of the subset, three
sanitized assessor workflows, independent downstream interoperability, the
success-metric pilots, and human release approval remain open. S-3 stays with
PRD 064 and S-4 remains deferred until multi-epoch workflows are validated.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Generated artifact appears to be an audit opinion | Legal/reputational harm | Assessor-authored labeling and explicit non-attestation language |
| OSCAL assessment model complexity causes partial semantics | Invalid interoperability | Typed model, official schema, independent-tool gate, narrow phase |
| Evidence links are mistaken for sufficiency | False assurance | Guardrail wording and comprehension tests |
| Sensitive findings leak | Confidentiality harm | IDs/hashes by default and explicit sensitive report handling |

## Open Questions :yellow_circle: `@human-review`

- **[Compliance, blocking]** Which minimum assessor fields and result statuses are necessary without implying an audit opinion?
- **[Engineering, blocking]** Which Assessment Results subset can be implemented completely in the first release?
- **[Legal, blocking]** What disclaimer and artifact labeling are required for non-attested output?
- **[Product, non-blocking]** Should HTML reporting be required for assessor adoption or remain a fast follow?

## Definition of Ready :red_circle: `@human-required`

- [x] Official schema provenance and typed-model spike are complete.
- [ ] Compliance and Legal approve judgment boundaries and terminology.
- [ ] Engineering approves the supported model subset and manifest.
- [ ] Three assessors provide sanitized workflows and fixtures.
- [x] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Separate Assessment Results from POA&M | Assessment judgment and remediation ownership are distinct workflows and models | One combined artifact builder |
| 2026-08-24 | Require assessor-authored claims | FORGE cannot infer professional conclusions from policy/evidence metadata | Automated findings |
| 2026-08-24 | Ship JSON first | Narrows interoperability risk for a complex new OSCAL model | Simultaneous JSON/XML/YAML |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.2 | 2026-08-27 | Codex | Implemented the bounded JSON MVP, schema provenance, exact context/reference validation, deterministic typed output, baseline/reporting, scaffold, tests, and engineering-status audit; human release gates remain open |
| 0.1 | 2026-08-24 | Codex | Initial draft for human-authored OSCAL Assessment Results |

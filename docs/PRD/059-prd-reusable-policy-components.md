# 059-prd-reusable-policy-components

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `059-reusable-policy-components`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will compose policy documents from versioned local Markdown components such as purpose, roles, access review, incident reporting, exceptions, and enforcement. A closed composition manifest pins component bytes, provides bounded typed parameter values, and emits assembled Markdown plus a provenance manifest that lets downstream OSCAL elements trace through the assembled policy to the original component file and line. The MVP deliberately avoids remote registries, conditional logic, nested includes, and a general-purpose template language.

## Context

### Background :red_circle: `@human-required`

Organizations repeat approved policy language across many documents. Copy-and-paste makes later corrections inconsistent, while unconstrained templates introduce hidden logic, unstable output, and difficult provenance. FORGE can already ingest structured documents and trace extracted requirements, but it has no deterministic source-composition layer for shared policy clauses.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | FORGE ingests Markdown and maintains line-oriented source traceability through conversion. | The composition output must retain a source map from assembled lines to component lines. |
| Repository evidence | Stable IDs and meaningful diffs depend on deterministic source structure. | Component identity, order, parameters, and hashes must be explicit and versioned. |
| Product principle | Correctness and auditability take precedence over convenience. | Use a small declarative substitution model, not code execution or arbitrary templates. |
| Product hypothesis | Shared components reduce policy drift and review effort across a policy library. | Measure changed-policy review scope and reuse, not component count. |

No customer component library, authoring study, or measured copy/paste defect rate was supplied. Reuse and time-savings targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Local UTF-8 Markdown component files with required metadata in a closed sidecar manifest
- Stable component key, semantic version string, owner, status, source hash, title, and declared parameters
- A flat composition manifest defining one output policy and ordered component instances
- Required/optional string, integer, boolean, and string-list parameters with defaults and validation constraints
- A reserved non-executable placeholder syntax used only in text content
- Deterministic assembled Markdown, composition lock file, and line-level provenance map
- Duplicate instance support through stable instance keys
- Validation of pinned hashes, parameters, placeholder completeness, heading structure, path containment, and output aliases
- Downstream conversion that preserves original component provenance where technically feasible

**Out of Scope:**

- Remote component registries, package installation, dependency resolution, or automatic updates
- Conditional statements, loops, expressions, functions, shell execution, environment interpolation, or network access
- Components including other components
- Rich-text DOCX/PDF component authoring in the MVP
- Collaborative editing, approval, lifecycle state, or marketplace discovery
- Automatic policy language generation or framework coverage claims

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/002-prd-markdown-ingestion.md` | Markdown source contract |
| `docs/PRD/003-prd-structural-extraction-headings.md` | Heading extraction constraints |
| `docs/PRD/016-prd-traceability-model.md` | Source provenance model |
| `docs/PRD/034-prd-parameter-extraction.md` | Existing policy parameter concepts |
| `docs/PRD/043-prd-diff-report.md` | Deterministic change review |
| `docs/PRD/053-prd-stable-id-migration.md` | Identity stability across source changes |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Approval and lifecycle of composed policies/components |

---

## Problem Statement :red_circle: `@human-required`

Policy authors need to reuse approved language without copying it into many independently maintained documents. Existing copy/paste and general template engines either create silent drift or make the rendered policy difficult to trace, reproduce, and review, undermining FORGE's stable-identity and source-provenance guarantees.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Compose policies reproducibly. | Identical component bytes, manifest, and parameter values produce byte-identical Markdown, lock, and provenance files. |
| G-2 | Preserve component origin. | 100% of assembled non-separator lines map to one component file/line or one explicit generated-metadata origin. |
| G-3 | Prevent hidden template behavior. | The MVP grammar supports no executable expressions, implicit environment values, remote includes, or nested components. |
| G-4 | Detect component drift. | Every unpinned or hash-mismatched component fails before output; pin refresh is always explicit. |
| G-5 | Reduce repeated review. | In five partner policy sets, a shared-clause update reduces manually reviewed duplicate lines by at least 50%. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- Components are not proof that their text is appropriate for every policy context.
- Composition does not approve, publish, or transition policy lifecycle state.
- The MVP is not a programming language or document-layout engine.
- The MVP does not fetch, install, trust, or license third-party component packages.
- Composition does not imply framework applicability or coverage.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Assemble a policy from reviewed components (P0)

> As a policy author, I want to assemble ordered approved clauses into a policy so that shared language has one maintained source.

### US-2 — Customize bounded organization values (P0)

> As a policy author, I want to supply declared values such as owner role or review interval so that reuse does not require copying the component.

### US-3 — Trace rendered text to its component (P0)

> As an auditor, I want every assembled requirement linked to the exact component bytes and line so that I can verify provenance.

### US-4 — Detect shared-component updates (P0)

> As a compliance engineer, I want pinned hashes and a lock file so that component changes never flow into policies silently.

### US-5 — Review component dependency impact (P1)

> As a policy owner, I want to list every composed policy that uses a component so that I can scope review after an update.

### US-6 — Convert composed output directly (P1)

> As a DevSecOps engineer, I want composition and OSCAL conversion to preserve component provenance so that CI can reproduce the complete artifact chain.

## Component Model :yellow_circle: `@human-review`

### Component Contract

Each component is one Markdown file plus manifest metadata:

- immutable `component-key`
- author-supplied version string
- title and owner key
- lifecycle label (`draft`, `approved`, `deprecated`) preserved but not authenticated by this feature
- expected raw SHA-256
- parameter declarations
- optional replacement component key when deprecated

The component body begins with a level-two heading and may contain deeper headings. Level-one headings are reserved for the composition manifest's policy title. Components cannot include other files or components.

### Parameter Contract

The reserved syntax is `{{forge:param:<name>}}`. Parameter names are ASCII kebab-case. Substitution occurs only after UTF-8 decoding and placeholder tokenization; values are treated as data, never reparsed as placeholders. Values containing Markdown structural characters are escaped according to a documented text-context rule. Placeholders in fenced code, inline code, URLs, raw HTML, or heading markers are rejected in the MVP.

### Composition and Provenance

The composition manifest contains policy metadata and an ordered list of stable instance keys referencing components and parameter values. Output order is manifest order. FORGE emits:

1. assembled Markdown;
2. a lock file containing manifest schema version, component keys/versions/hashes, instance keys, parameter-value hashes, and output hash; and
3. a provenance file mapping assembled line/column spans to component file labels, raw hashes, source line/column spans, instance keys, and parameter origins.

Secrets must not be supplied as parameters. Reports and lock files hash parameter values by default and include clear sensitivity guidance.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge policy compose --manifest <FILE>`, `forge policy component check <FILE>`, and `forge policy compose check --manifest <FILE>`.
- [ ] **M-2 — Closed schemas:** Parse bounded `forge.policy-component/1` and `forge.policy-composition/1` JSON manifests; reject unknown/duplicate keys and unsupported versions.
- [ ] **M-3 — Local containment:** Resolve only explicit local component paths under the declared project root; reject traversal, unsafe symlinks, non-regular files, and aliases.
- [ ] **M-4 — Pinning:** Require expected component SHA-256 and fail on mismatch before rendering any output.
- [ ] **M-5 — Flat graph:** Reject component includes, recursive manifests, or any nesting beyond the one composition-to-components layer.
- [ ] **M-6 — Structure:** Require exactly one manifest-generated level-one policy heading; validate component heading level and deterministic separators.
- [ ] **M-7 — Parameters:** Support only declared string, integer, boolean, and string-list values with required/default, length/range/count, and optional regex/enum constraints.
- [ ] **M-8 — Safe substitution:** Tokenize the reserved syntax, substitute once, escape by documented context, reject unsupported contexts, and never evaluate output as code or another template pass.
- [ ] **M-9 — Completeness:** Reject missing required values, unknown values, undeclared placeholders, unused supplied values, duplicate names, and unresolved placeholders.
- [ ] **M-10 — Stable instances:** Require unique instance keys; allow repeated component keys only through distinct instances and preserve their separate provenance.
- [ ] **M-11 — Outputs:** Emit assembled Markdown, composition lock, and provenance map atomically; reject collisions among inputs and outputs.
- [ ] **M-12 — Provenance:** Map every assembled content span to component or generated metadata origin, including parameter substitutions without exposing values by default.
- [ ] **M-13 — Determinism:** Identical bytes and values produce byte-identical outputs without absolute paths, wall-clock time, locale, or environment dependence.
- [ ] **M-14 — Validation chain:** Optionally run the existing Markdown ingestion/conversion validation after composition without changing composition output.
- [ ] **M-15 — Sensitive data:** Warn and fail by default on parameter names matching documented secret patterns; provide no environment-variable or secret-manager interpolation.
- [ ] **M-16 — Tests:** Cover grammar boundaries, escaping, hash drift, path attacks, duplicate instances, headings, provenance completeness, determinism, and atomic-write failures.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Build a reverse dependency index from explicitly supplied composition manifests.
- [ ] **S-2:** Produce a deterministic component-update impact report without modifying locks.
- [ ] **S-3:** Preserve component/instance provenance in generated OSCAL trace reports.
- [ ] **S-4:** Scaffold a component manifest from an existing Markdown section without approving it.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** A signed local component package format and explicit trust policy.
- [ ] **C-2:** Web component library and composition editor backed by the same schemas.
- [ ] **C-3:** Conditional variants only if user evidence justifies a separately specified, non-Turing-complete grammar.

### Won't Have (W) — This release :red_circle: `@human-required`

- Remote registries, auto-update, nested includes, executable templates, conditionals, loops, environment interpolation, secrets, or rich-text components.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Three pinned components and valid values | Composition runs | Markdown, lock, and provenance outputs are complete and deterministic |
| AC-2 | A component byte changes after pinning | Composition runs | Exit is `2` and no output is created or replaced |
| AC-3 | A value contains another valid placeholder token | Composition runs | The value is emitted as escaped data and is not recursively expanded |
| AC-4 | A placeholder appears inside a fenced code block | Validation runs | The unsupported context is reported with component line and no output |
| AC-5 | One component is instantiated twice | Composition runs | Both instances render in order with distinct provenance instance keys |
| AC-6 | The same project is composed in different absolute directories | Outputs are compared | Output bytes match and contain no absolute paths |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Successful composition | 4 of 5 partners compose a policy without maintainer edits | Moderated pilot |
| Leading | Provenance completeness | 100% rendered content spans attributable | Automated invariant tests |
| Leading | Silent drift | Zero unpinned component changes accepted | Adversarial fixtures |
| Lagging | Duplicate review reduction | 50% fewer manually reviewed duplicate lines for one shared update | Partner review exercise |
| Lagging | Reuse depth | Three partners use at least five components across three policies within 90 days | Opt-in partner evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** Existing safe I/O, hashing, Markdown ingestion, traceability, and stable-ID contracts.
- **Phase 1:** Flat Markdown composition, parameters, pins, lock, and provenance.
- **Phase 2:** Reverse dependency impact and downstream OSCAL trace preservation.
- **Phase 3:** Design-partner component libraries and interoperability validation.
- **Integrates with:** PRD 058 lifecycle records may approve component versions; this PRD only preserves the label/reference.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Template grammar expands into a programming language | Complexity and security risk | Flat graph, fixed placeholder grammar, no evaluation or second pass |
| Reused language is contextually wrong | Policy quality failure | Human review, visible instance provenance, no automatic applicability claim |
| Parameter values alter Markdown structure | Trace or injection defects | Context restrictions, escaping, and structural validation |
| Component update causes widespread churn | Review overload | Hash pins and explicit reverse-dependency impact before refresh |
| Locks expose sensitive values | Confidentiality loss | Hash values by default and reject likely secret parameter names |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Is parameterization necessary for the first design partners, or should v1 prove exact-fragment reuse first?
- **[Engineering, blocking]** What span model best composes with current line-based traceability without changing existing single-source output?
- **[Security, blocking]** Is Markdown escaping sufficient for all accepted text contexts, or should v1 allow plain paragraph/list text only?
- **[Product, non-blocking]** Should component lifecycle labels be validated only as metadata until PRD 058 is implemented?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves flat composition and parameter scope.
- [ ] Engineering approves placeholder grammar, span provenance, hash/lock contracts, and path bounds.
- [ ] Security reviews substitution contexts, path handling, and sensitive-value controls.
- [ ] Three representative policy sets identify genuinely shared clauses.
- [ ] Synthetic components cover every supported Markdown and parameter edge case.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Use flat, pinned local components | Maximizes reproducibility and limits dependency/supply-chain risk | Remote packages; recursive includes |
| 2026-08-24 | Emit source maps and lock files | Reuse must preserve audit provenance and update intent | Rendered Markdown only |
| 2026-08-24 | Use one-pass typed placeholders | Covers common customization without executable templating | Handlebars/Liquid; arbitrary expressions |
| 2026-08-24 | Reserve heading level one for the policy | Prevents ambiguous document hierarchy | Automatic arbitrary heading rebasing |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for pinned reusable Markdown policy components |

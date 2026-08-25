# 067-prd-read-only-mcp-governance-interface

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `067-read-only-mcp-governance-interface`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will expose approved, local governance artifacts through a read-only Model Context Protocol server. Agents can discover policies, search requirements, inspect applicability and gaps, and follow traceability to exact source/artifact identifiers with bounded citations. The server uses stdio in the MVP, receives one explicit project root, performs no model calls or network access, and exposes no mutation, command execution, evidence content, or automatic compliance decision.

## Context

### Background :red_circle: `@human-required`

FORGE's machine-readable artifacts can help coding and operations agents answer which policies apply and why, but raw OSCAL is cumbersome and unconstrained retrieval can leak sensitive content or let untrusted policy text influence tool behavior. A small read-only MCP surface can make approved governance queryable while preserving artifact identity, provenance, and least privilege.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Product vision | FORGE positions OSCAL as a shared truth layer for agents and mentions MCP-native use. | A query interface is strategically aligned if it remains grounded in approved artifacts. |
| Repository evidence | FORGE already inventories, validates, traces, diffs, maps, and reports deterministically. | MCP methods should be thin read-only views over shared core queries. |
| Security boundary | Policy prose and retrieved content are untrusted data, not instructions. | Responses must separate data from protocol guidance and never execute embedded directives. |
| Product hypothesis | Citation-grounded governance queries reduce agent policy mistakes and token use. | Evaluate answer support and abstention, not model self-confidence. |

No MCP client study, threat model, approved tool schema, latency benchmark, or agent evaluation corpus was supplied. Targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Local stdio MCP server started with one explicit project root
- Resources for project inventory, approved policy metadata, artifact/report metadata, and schema/version information
- Read-only tools for listing policies, searching requirements, getting one requirement, tracing a control, explaining recorded applicability, and summarizing gaps
- Lifecycle-aware default visibility: approved/current artifacts only where PRD 058 data exists
- Bounded excerpts with stable source labels, line/span metadata, hashes, and artifact IDs
- Deterministic filtering, pagination, error codes, and freshness checks
- Server capability declaration with no mutating tools

**Out of Scope:**

- Writes, lifecycle transitions, mapping changes, policy generation, shell execution, or external tool invocation
- HTTP/SSE transport, remote hosting, authentication service, or multiple project roots
- Model inference, RAG embeddings, web browsing, or AI suggestions
- Raw evidence content, secrets, reviewer PII, or licensed framework prose by default
- Compliance/effectiveness conclusions not explicitly present as human-authored assertions

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/FORGE_PRODUCT_VISION.md` | Agent-native shared-truth positioning |
| `docs/PRD/016-prd-traceability-model.md` | Source citations |
| `docs/PRD/038-prd-traceability-report.md` | Existing trace report |
| `docs/PRD/055-prd-control-mapping.md` | Reviewed relationships |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Applicability/gap queries |
| `docs/PRD/058-prd-policy-lifecycle-management.md` | Approved/current visibility |
| `docs/PRD/060-prd-evidence-implementation-linking.md` | Metadata-only implementation/evidence trace |

---

## Problem Statement :red_circle: `@human-required`

Agents need concise, attributable answers about policy requirements and framework scope, but parsing whole documents or OSCAL artifacts wastes context and encourages unsupported interpretation. Without a strict read-only query layer, MCP access could expose excessive sensitive content or allow policy text to become an instruction channel rather than governed data.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Return grounded governance data. | 100% of substantive response items include resolvable artifact/source identity and no fabricated requirement IDs. |
| G-2 | Enforce read-only least privilege. | All seeded write, off-root read, process, and network attempts are unavailable or rejected. |
| G-3 | Prefer safe abstention. | Missing, stale, ambiguous, or unapproved data produces explicit unavailable/ambiguous responses, never guessed answers. |
| G-4 | Bound context and disclosure. | Every tool enforces result, excerpt, byte, and pagination limits with redacted defaults. |
| G-5 | Improve agent task grounding. | On an adjudicated corpus, agents using MCP cite supported policy requirements 30% more accurately than raw-document baseline. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- The server does not decide what a policy means beyond recorded structured data.
- MCP responses do not grant an agent authority to act.
- The server does not expose draft/unapproved content by default.
- Search relevance is not evidence of applicability, compliance, or precedence.
- The MVP is not a remote enterprise knowledge service.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Find applicable requirements (P0)

> As a coding agent, I want to query approved requirements and recorded applicability so that I can surface relevant guardrails with citations.

### US-2 — Trace the source of a rule (P0)

> As a developer, I want an agent response to identify exact policy/artifact provenance so that I can verify it independently.

### US-3 — Refuse unsupported answers (P0)

> As a security owner, I want stale, ambiguous, or absent governance data reported explicitly so that an agent does not invent policy.

### US-4 — Limit sensitive disclosure (P0)

> As a compliance lead, I want metadata-first bounded responses so that agents receive only what the task requires.

### US-5 — Integrate multiple MCP clients (P1)

> As an engineer, I want protocol-conformant schemas and fixtures so that approved clients receive consistent results.

## MCP Surface :yellow_circle: `@human-review`

Initial tools:

- `list_policies`
- `search_requirements`
- `get_requirement`
- `trace_control`
- `get_recorded_applicability`
- `get_gap_summary`
- `get_artifact_status`

Tools return structured objects before prose. Search accepts explicit filters and returns matches, not an answer synthesized by FORGE. Every excerpt is labeled `untrusted-content` and includes source identity. Resource templates expose only versioned metadata and bounded content requested by exact ID.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Command/transport:** Provide `forge mcp serve --project <DIR>` over stdio only in the MVP.
- [ ] **M-2 — Project containment:** Resolve only explicit project manifests/artifacts beneath the canonical project root; reject traversal, unsafe links, special files, and aliases.
- [ ] **M-3 — Read-only capability:** Declare and implement no write, command, network, approval, suggestion, or external-mutation tools.
- [ ] **M-4 — Shared core:** Use typed read-only library queries also covered by direct core tests; do not shell out.
- [ ] **M-5 — Validation/freshness:** Validate artifact schemas/hashes before serving; return stale/invalid status rather than partial trusted content.
- [ ] **M-6 — Lifecycle visibility:** Serve approved/current policies by default; require explicit server-start opt-in to expose draft/superseded metadata and label every response.
- [ ] **M-7 — Structured tools:** Implement the initial tool set with closed input/output schemas, stable errors, deterministic sorting, and cursor pagination.
- [ ] **M-8 — Grounding:** Include artifact type/root ID/version/hash and source label/span for every requirement, mapping, applicability, and gap item.
- [ ] **M-9 — Search semantics:** Use deterministic lexical/structured search in v1; return ranked matches with explained fields, not a generated answer.
- [ ] **M-10 — Prompt-injection boundary:** Treat all retrieved text as inert untrusted content; never parse it as MCP instructions or trigger another tool.
- [ ] **M-11 — Bounds:** Limit request size, query complexity, result count, excerpt bytes, total response bytes, traversal depth, and processing time.
- [ ] **M-12 — Privacy:** Omit evidence bytes, absolute paths, URI secrets, reviewer contact fields, and full framework/policy prose by default.
- [ ] **M-13 — No network/process:** Perform no outbound network request, subprocess launch, plugin load, or dynamic code execution.
- [ ] **M-14 — Session behavior:** Keep no authoritative mutation state; cache only validated read data keyed by hashes and invalidate on file changes.
- [ ] **M-15 — Protocol/security tests:** Cover malformed clients, unknown methods, oversized requests, cancellation, path attacks, content injection, stale cache, and capability assertions.
- [ ] **M-16 — Evaluation:** Compare supported-answer accuracy, citation validity, abstention, disclosure, latency, and token size against raw-document baseline.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Read-only implementation/evidence metadata trace from PRD 060 with no evidence excerpts.
- [ ] **S-2:** Client-facing resource links to static PRD 062 reports.
- [ ] **S-3:** Configurable policy visibility profiles signed or hash-pinned by project owners.
- [ ] **S-4:** Optional local full-text index whose bytes remain under the project root and rebuild deterministically.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Authenticated local HTTP transport under a separate threat model.
- [ ] **C-2:** Organization retrieval index and reranking after privacy/evaluation gates.
- [ ] **C-3:** Separate mutating MCP server only if explicit authorization and approval workflows are designed; not an extension of this server.

### Won't Have (W) — This release :red_circle: `@human-required`

- Writes, remote transport, model inference, embeddings, browsing, evidence content, tool chaining, or unapproved content by default.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | An approved policy with traced requirements | `search_requirements` runs | Matches include exact IDs/hashes/spans and bounded inert excerpts |
| AC-2 | A client asks to edit a mapping or execute a command | It inspects/calls the server | No such capability exists and the request is rejected |
| AC-3 | A policy contains instructions to call another tool | It is retrieved | Text is labeled untrusted and no tool or process runs |
| AC-4 | An artifact hash no longer matches lifecycle state | It is queried | Response is stale/unavailable rather than trusted partial data |
| AC-5 | A path points outside project root | It is resolved | Access fails before content is read |
| AC-6 | A query has no supported match | It completes | Response explicitly abstains with zero invented IDs |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Citation validity | 100% | Contract fixtures |
| Leading | Read-only boundary | 100% seeded attempts blocked | Security suite |
| Leading | Unsupported answer rate | Zero fabricated IDs; approved abstention threshold | Adjudicated corpus |
| Lagging | Grounded-answer improvement | 30% over raw-document baseline | Controlled agent evaluation |
| Lagging | Adoption | Three teams use two approved MCP clients for 60 days | Opt-in pilot evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** Stable project/artifact discovery, read-only shared core queries, and a completed threat model.
- **Phase 1:** Static resources plus exact-ID `get_requirement`/`trace_control` prototype.
- **Phase 2:** Search, applicability/gaps, lifecycle filtering, bounds, and client interoperability.
- **Phase 3:** Agent evaluation and security release gate.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Policy content performs prompt injection | Agent misuse | Inert structured fields, explicit labels, no tool chaining |
| Excess data disclosure | Confidentiality/licensing harm | Metadata-first defaults and strict response bounds |
| Stale policy served as approved | Incorrect agent behavior | Hash/lifecycle validation and fail-safe unavailability |
| Search match mistaken for policy interpretation | False assurance | Match-only semantics and citations |
| Future writes creep into same server | Privilege expansion | Permanent read-only capability contract and separate future server boundary |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Which three queries represent the strongest initial agent job?
- **[Security, blocking]** Is stdio process isolation sufficient for supported clients, or is an explicit client allowlist needed?
- **[Engineering, blocking]** What project manifest defines authoritative artifact discovery?
- **[Evaluation, blocking]** Which agent tasks and raw-document baseline will measure grounded-answer improvement?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product selects the initial query jobs and approved content defaults.
- [ ] Security approves the MCP threat model and permanent read-only boundary.
- [ ] Engineering approves tool/resource schemas, project discovery, and bounds.
- [ ] An adjudicated agent evaluation corpus exists.
- [ ] Two independent MCP clients pass interoperability fixtures.
- [ ] Every Must Have maps to an executable test or evaluation gate.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Ship stdio read-only MCP first | Minimizes network/auth complexity and authority | Remote HTTP mutating server |
| 2026-08-24 | Return matches/data, not generated answers | FORGE should preserve truth rather than add another model layer | Embedded RAG/LLM responses |
| 2026-08-24 | Default to approved/current artifacts | Agents should not mistake drafts for policy | Serve every project file |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for a local read-only MCP governance interface |

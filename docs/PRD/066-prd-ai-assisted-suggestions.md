# 066-prd-ai-assisted-suggestions

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `066-ai-assisted-suggestions`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will optionally produce quarantined AI suggestion bundles for mapping candidates and policy drafting. Suggestions must cite exact supplied source spans, state assumptions, preserve provider/model/prompt provenance, and remain inert until a human explicitly accepts, edits, or rejects each item. AI output never changes applicability, approved mappings, policy lifecycle, evidence sufficiency, assessment findings, or compliance status.

## Context

### Background :red_circle: `@human-required`

Human-reviewed mappings and policy drafting can be labor intensive. Language models may accelerate candidate discovery and first drafts, but they are non-deterministic, can invent facts or citations, and may expose sensitive policy/framework content to providers. FORGE needs a hard boundary that captures AI output as untrusted proposals without contaminating deterministic artifacts.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Product principle | Existing mapping and authoring PRDs require explicit human provenance and no inferred compliance. | Suggestions must use a separate schema and explicit promotion step. |
| Security/privacy boundary | Policy and framework prose may be confidential or licensed. | Users preview exact provider payloads and opt into every external transmission. |
| Product hypothesis | Citation-grounded candidates reduce reviewer effort without reducing decision quality. | Measure accepted-with-edit rates, missed candidates, and review time on adjudicated corpora. |

No approved model/provider, evaluation corpus, privacy review, cost model, or user evidence was supplied. The feature cannot be Ready until those exist.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- Mapping candidate suggestions for PRD 055 manifests
- Policy topic, question, outline, and clause suggestions for PRD 061 projects
- Explicit local context bundle, payload preview, redaction, token/size estimate, and user consent
- Provider-neutral request/response adapter with one approved reference provider after review
- Citation validation against supplied source spans and IDs
- Assumptions, uncertainty, alternatives, and structured rationale
- Quarantined `forge.suggestions/1` bundle plus deterministic human disposition manifest
- Offline evaluation harness and release thresholds

**Out of Scope:**

- Automatic applicability, mapping approval, policy approval, evidence sufficiency, findings, risk, remediation, or compliance decisions
- Autonomous agents, tool use, browsing, repository mutation, or external workflow mutation
- Training/fine-tuning on user content
- Background submission, hidden prompts, undisclosed provider logging, or secret collection
- Ranking by self-reported model confidence alone

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/PRD/055-prd-control-mapping.md` | Human-reviewed mapping destination |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Applicability boundary AI cannot decide |
| `docs/PRD/061-prd-framework-guided-policy-authoring.md` | Drafting destination |
| `docs/PRD/062-prd-local-web-workspace.md` | Later review interface |
| `docs/PRD/068-prd-collaborative-review-queues.md` | Suggestion disposition workflow |

---

## Problem Statement :red_circle: `@human-required`

Compliance engineers spend substantial time discovering plausible control relationships and preparing policy drafts, but opaque AI generation can manufacture authoritative-looking claims and leak sensitive content. Users need acceleration only if every suggestion remains traceable, reviewable, measurable, and incapable of silently entering approved artifacts.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Reduce candidate review time safely. | Median reviewed mapping/drafting task time improves 30% on an adjudicated pilot corpus. **Hypothesis.** |
| G-2 | Prevent unsupported citations. | 100% emitted citations resolve to supplied exact spans; unresolved citations are rejected. |
| G-3 | Preserve human authority. | Zero suggestions alter authoritative artifacts without an explicit disposition and normal downstream validation. |
| G-4 | Make model behavior evaluable. | Every bundle records provider, model/version, prompt-template hash, parameters, input hashes, output hash, and cost/usage metadata where available. |
| G-5 | Protect sensitive content. | Every external request has an exact preview and consent record; seeded secrets/licensed exclusions are blocked or redacted. |

## Non-Goals :red_circle: `@human-required`

- FORGE does not describe AI output as correct, approved, compliant, or expert advice.
- Model confidence does not substitute for evidence-supported evaluation.
- The MVP does not run autonomously or call tools based on model output.
- The MVP does not retain provider credentials or transmit content without preview.
- Suggestions are not deterministic FORGE outputs; only packaging and dispositions are deterministic.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Review grounded mapping candidates (P0)

> As a compliance engineer, I want candidate relationships with exact citations and assumptions so that I can review faster without trusting opaque similarity.

### US-2 — Draft from approved context (P0)

> As a policy author, I want clause suggestions constrained to approved organization answers and gaps so that unsupported facts remain visible.

### US-3 — Control data disclosure (P0)

> As a security owner, I want to preview and redact the exact provider payload so that sensitive material is not transmitted accidentally.

### US-4 — Preserve dispositions (P0)

> As an auditor, I want acceptance, edits, rejection, reviewer, and rationale recorded so that AI involvement is transparent.

### US-5 — Evaluate model changes (P1)

> As a product owner, I want regression metrics by task and model version so that cost or speed improvements do not hide quality loss.

## Suggestion Trust Model :yellow_circle: `@human-review`

Suggestions live outside authoritative manifests. Each suggestion is `pending`, `accepted-as-is`, `accepted-edited`, `rejected`, or `expired`. Promotion generates a proposed downstream manifest change that must pass the ordinary PRD 055 or PRD 061 validation/review path; promotion never approves it.

Evidence support is rated `high`, `medium`, `low`, or `unsupported` using deterministic citation/claim checks and human adjudication rules, not model self-confidence. Exact provider prompts/responses may contain sensitive content and are opt-in retained separately from the default redacted bundle.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Commands:** Provide `forge suggest prepare`, `run`, `validate`, `review`, and `promote` as separate explicit steps.
- [ ] **M-2 — Task scope:** Support only versioned mapping-candidate and policy-drafting task schemas in v1.
- [ ] **M-3 — Context allowlist:** Include only explicitly selected source spans, IDs, approved answers, and bounded metadata; exclude secrets and unrelated files.
- [ ] **M-4 — Payload preview:** Write an exact redacted payload preview, provider/model target, data-handling notice, and size/cost estimate before any external request.
- [ ] **M-5 — Explicit consent:** Require a fresh consent token tied to the exact payload hash, provider, model, and retention policy.
- [ ] **M-6 — Provider boundary:** Use a typed adapter with fixed allowlisted hosts, TLS, timeouts, response bounds, no redirects to unapproved hosts, and credentials outside project files/logs.
- [ ] **M-7 — Structured output:** Require closed-schema responses and treat all model text as untrusted data; reject unknown, oversized, malformed, or tool-call content.
- [ ] **M-8 — Citation validation:** Require every substantive suggestion to cite supplied source IDs/spans; reject missing, nonexistent, or altered citations.
- [ ] **M-9 — Assumptions:** Require explicit assumptions and unresolved questions; forbid fabricated organization facts.
- [ ] **M-10 — Quarantine:** Store suggestions only in `forge.suggestions/1`; never write authoritative mapping, policy, lifecycle, evidence, assessment, or POA&M artifacts directly.
- [ ] **M-11 — Disposition:** Require reviewer key/time/rationale for accept, edit, reject, or expiry and preserve original plus edited content/hashes.
- [ ] **M-12 — Promotion:** Produce a proposed downstream patch that passes normal validation and remains unapproved.
- [ ] **M-13 — Provenance:** Record provider/model/version where available, template/version/hash, parameters, input/output hashes, request ID, usage/cost, and redaction policy.
- [ ] **M-14 — Evaluation gate:** Ship only after adjudicated precision/recall, citation validity, harmful unsupported-claim, leakage, latency, and cost thresholds are approved per task.
- [ ] **M-15 — Regression:** Block model/prompt/provider changes that cross approved quality or safety thresholds.
- [ ] **M-16 — No telemetry/training consent:** Separate provider processing consent from any product telemetry or training use; default both off.
- [ ] **M-17 — Tests:** Cover prompt injection, citation fabrication, data exfiltration requests, malformed output, provider errors, secret redaction, quarantine, promotion, and eval regressions.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Approved local-model adapter with identical quarantine and evaluation contracts.
- [ ] **S-2:** Batch review ordering by deterministic evidence support and impact, never raw model confidence.
- [ ] **S-3:** Side-by-side source/candidate/diff review in PRD 062.
- [ ] **S-4:** Re-evaluate expired suggestions after input changes without carrying forward approval.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Additional suggestion tasks after separate corpora and acceptance thresholds exist.
- [ ] **C-2:** Organization-owned retrieval index limited to explicitly approved content.
- [ ] **C-3:** Multiple-model comparison as a reviewer aid, not consensus authority.

### Won't Have (W) — This release :red_circle: `@human-required`

- Autonomous changes, tool use, browsing, applicability/compliance decisions, evidence assessment, auto-approval, hidden provider calls, or training reuse.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | Selected source spans and an external provider | Prepare runs | Exact payload/redaction/provider details are available before consent |
| AC-2 | A response cites a span not sent | Validate runs | The suggestion is rejected and cannot be reviewed/promoted |
| AC-3 | A model requests a tool call or unrelated file | Run/validate completes | No tool/file action occurs and unsupported content is quarantined/rejected |
| AC-4 | A reviewer accepts an edited mapping candidate | Promote runs | A proposed PRD 055 manifest patch is created but remains unapproved |
| AC-5 | A model version reduces recall below threshold | Regression runs | Release is blocked despite lower cost/latency |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Citation validity | 100% emitted suggestions | Deterministic validator |
| Leading | Unsupported harmful claim rate | Below approved task threshold; target under 2% | Human-adjudicated corpus |
| Leading | Leakage | Zero seeded prohibited data transmitted | Security tests |
| Lagging | Review time | 30% median reduction | Controlled pilot |
| Lagging | Accepted-with-edit profile | At least 60% useful; no more than 20% accepted without substantive review | Disposition analysis |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** Stable PRD 055 and PRD 061 schemas, approved corpus, provider/privacy/security review.
- **Phase 0:** Corpus, human rubric, threat model, privacy/legal review, and no-network offline harness.
- **Phase 1:** Prepare/validate/review with recorded responses; no live provider required.
- **Phase 2:** One approved provider adapter, mapping candidates, strict release gate.
- **Phase 3:** Policy drafting, local model, and UI review after task-specific evaluation.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Hallucinated authority/citations | False compliance claims | Exact citation validator, quarantine, human disposition |
| Sensitive/licensed content sent externally | Legal/privacy harm | Payload preview, allowlist/redaction, explicit consent, provider review |
| Automation bias | Weak review | No auto-approval, disposition rationale, comprehension/usability testing |
| Model drift | Silent quality loss | Version pinning and regression gates |
| Non-determinism contaminates artifacts | Unreviewable output | Separate suggestion schema and deterministic promotion proposal |

## Open Questions :yellow_circle: `@human-review`

- **[Product, blocking]** Which single task—mapping candidates or clause drafting—has enough user value and corpus quality to launch first?
- **[Security/privacy, blocking]** Must v1 be local-model-only, or can one external provider meet content-handling requirements?
- **[Data/evaluation, blocking]** What precision/recall and unsupported-claim thresholds are acceptable per task?
- **[Legal, blocking]** Which framework/policy content may be transmitted to each approved provider?

## Definition of Ready :red_circle: `@human-required`

- [ ] One task and target user workflow are selected.
- [ ] Human-adjudicated evaluation and prompt-injection corpora exist.
- [ ] Product approves quality, usefulness, latency, and cost gates.
- [ ] Security/privacy/legal approve provider and payload handling.
- [ ] Engineering approves quarantine/promotion/provider contracts.
- [ ] Every Must Have maps to an executable test or human evaluation gate.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Quarantine all AI output | Non-deterministic suggestions cannot become governance truth directly | Direct artifact generation |
| 2026-08-24 | Require exact citations and assumptions | Reviewers need evidence, not fluent confidence | Uncited prose and similarity score |
| 2026-08-24 | Separate payload consent from product telemetry | Provider processing and analytics are distinct privacy decisions | One blanket consent |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for citation-grounded, quarantined AI assistance |

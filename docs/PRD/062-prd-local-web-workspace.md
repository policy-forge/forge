# 062-prd-local-web-workspace

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-24 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `062-local-web-workspace`
**Created**: 2026-08-24
**Status**: Draft
**Input**: Post-v1.3 product planning

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will provide a local-first browser workspace for importing policies, reviewing traceability and gaps, editing manifests, validating changes, and exporting the same artifacts produced by the CLI. The MVP binds only to loopback, requires a per-launch capability token, restricts file access to one explicit project root, and invokes shared Rust library APIs rather than shell commands. It is not a hosted SaaS product and includes no accounts, cloud storage, multitenancy, or remote collaboration.

## Context

### Background :red_circle: `@human-required`

FORGE's primary compliance-engineer persona benefits from structured policy workflows but currently must use a terminal and edit JSON manifests directly. A GUI can broaden adoption and make provenance-heavy review understandable, but a premature cloud platform would add authentication, tenancy, storage, billing, and privacy obligations before the core workflow is validated.

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Product vision | FORGE is CLI-first and non-technical authors are currently an anti-persona. | The workspace is an intentional persona expansion and must preserve CLI parity. |
| Repository evidence | Core capabilities are local, offline, deterministic, and library-backed. | The UI should call the same core contracts and create no second business-logic implementation. |
| Product hypothesis | Visual review of gaps and traceability will improve successful first use more than adding more CLI models. | Validate task completion before expanding to hosted collaboration. |

No usability test, interaction design, accessibility audit, or deployment research was supplied. Adoption targets are hypotheses.

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- A local process serving a single-user web application on loopback only
- One explicit project root selected at launch
- Import, inventory, validation, mapping/gap review, traceability views, manifest editing, safe output previews, and export
- Initial workflow coverage for existing conversion/validation plus PRDs 055 and 056
- Versioned local workspace metadata containing relative paths and artifact hashes
- Accessible keyboard-operable interface targeting WCAG 2.1 AA
- Shared Rust service/application APIs with CLI parity tests
- Per-launch authentication token, CSRF protection, CSP, request limits, and secure shutdown

**Out of Scope:**

- Hosted deployment, LAN binding, remote access, accounts, SSO, multitenancy, billing, or cloud persistence
- Collaborative simultaneous editing or review queues; PRD 068 owns collaboration
- Browser extensions, arbitrary command execution, embedded terminal, plugin execution, or user-supplied JavaScript
- Automatic framework downloads, telemetry by default, or background update checks
- Full rich-text document fidelity for DOCX/PDF source editing

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `docs/FORGE_PRODUCT_VISION.md` | CLI-first principle and persona boundary |
| `docs/PRD/044-prd-summary-dashboard.md` | Existing summary model |
| `docs/PRD/055-prd-control-mapping.md` | Mapping review workflow |
| `docs/PRD/056-prd-framework-applicability-gap-analysis.md` | Initial gap dashboard workflow |
| `docs/PRD/061-prd-framework-guided-policy-authoring.md` | Later guided authoring workflow |
| `docs/PRD/068-prd-collaborative-review-queues.md` | Future shared review layer |

---

## Problem Statement :red_circle: `@human-required`

Compliance engineers who are uncomfortable with terminal and JSON workflows cannot readily use FORGE's traceability, mapping, and gap capabilities. Wrapping commands in a browser without a strict local trust boundary would improve ergonomics while creating serious file-access, request-forgery, and output-integrity risks.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Make the core workflow usable without terminal commands. | Four of five target users complete import-to-gap-report without assistance. |
| G-2 | Preserve CLI correctness and parity. | Every UI mutation serializes a documented manifest and produces artifacts byte-equivalent to the matching library/CLI operation. |
| G-3 | Constrain local authority. | All seeded off-root access, forged requests, remote binds, unsafe outputs, and script injection attempts fail. |
| G-4 | Make provenance understandable. | Four of five users can trace a reported gap to framework, mapping, and source policy in under two minutes. |
| G-5 | Improve activation. | At least 60% of pilot users complete a first valid project during their first session. **Hypothesis.** |

## Non-Goals :red_circle: `@human-required`

- The MVP is not a hosted service or enterprise administration console.
- The UI does not gain capabilities unavailable to the underlying core contracts.
- Browser rendering does not replace machine-readable artifacts as the source of truth.
- The workspace does not silently save, overwrite, or publish changes.
- The MVP does not edit original PDF or DOCX files.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — Complete a local project visually (P0)

> As a compliance engineer, I want to import, validate, map, and analyze local policies in a guided interface so that I do not need to author commands manually.

### US-2 — Review provenance side by side (P0)

> As an auditor, I want to navigate from a finding to the exact source and artifact metadata so that I can understand the basis of the result.

### US-3 — Preview every write (P0)

> As a policy owner, I want a diff and explicit confirmation before files change so that the UI cannot silently overwrite evidence or policy artifacts.

### US-4 — Recover safely (P0)

> As a user, I want failed or interrupted operations to preserve prior files and explain the next action so that I can resume without reconstructing the project.

### US-5 — Share a static report (P1)

> As a compliance engineer, I want to export a self-contained redacted HTML report so that reviewers can inspect results without running FORGE.

## Workspace and Trust Model :yellow_circle: `@human-review`

- Bind to a randomly selected port on `127.0.0.1` and `::1` only; fail closed if loopback binding cannot be guaranteed.
- Generate a random per-launch bearer capability and deliver it through a launch URL fragment or equivalent mechanism that is not logged or sent in referrers.
- Require token plus same-origin/CSRF defenses for every state-changing request.
- Resolve all file operations beneath one canonical project root and validate file type at open time.
- Use shared in-process Rust APIs; never concatenate shell commands or pass user data through a shell.
- Escape all policy/framework content as untrusted data and enforce a restrictive CSP with no remote scripts.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Launch:** Provide `forge workspace --project <DIR>` with loopback-only random-port binding and a documented shutdown path.
- [ ] **M-2 — Capability token:** Require a cryptographically random per-launch token for all API access; never persist or log it.
- [ ] **M-3 — Project containment:** Permit reads/writes only beneath the explicit project root using open-time symlink/file-type defenses and existing safe-I/O rules.
- [ ] **M-4 — Shared core:** Expose existing operations through typed library interfaces; prohibit shell invocation and duplicate UI-only validation logic.
- [ ] **M-5 — Initial workflows:** Support project inventory, policy conversion, artifact validation, PRD 055 mapping review, PRD 056 applicability analysis, and trace/report viewing.
- [ ] **M-6 — Manifest source of truth:** Read and write versioned on-disk manifests; UI state alone is never authoritative.
- [ ] **M-7 — Explicit writes:** Show target path, overwrite status, validation result, and semantic/text diff before every material write; require confirmation.
- [ ] **M-8 — Transaction safety:** Use atomic writes and preserve original bytes on cancellation, validation error, browser disconnect, or process failure.
- [ ] **M-9 — Web security:** Enforce CSP, no remote code, escaped content, same-origin checks, CSRF protection, bounded requests, secure headers, and no wildcard CORS.
- [ ] **M-10 — No network:** Make no outbound requests, telemetry, font/CDN loads, or update checks during workspace operation.
- [ ] **M-11 — Sensitive output:** Redact absolute paths, URI secrets, reviewer PII, and policy excerpts by default; make sensitive views explicit and non-cacheable.
- [ ] **M-12 — Accessibility:** Meet WCAG 2.1 AA for core workflows, keyboard navigation, focus order, status announcements, contrast, and non-color error cues.
- [ ] **M-13 — Concurrency:** Serialize conflicting writes, detect external file changes by hash, and require reload/reconciliation rather than last-write-wins.
- [ ] **M-14 — Error recovery:** Provide stable error codes, actionable messages, operation status, and restart-safe project files without hidden partial state.
- [ ] **M-15 — Parity tests:** Verify representative UI operations produce byte-equivalent artifacts and exit/result classifications to direct core calls.
- [ ] **M-16 — Security tests:** Cover DNS rebinding assumptions, remote bind attempts, token/CSRF failures, XSS, path traversal, symlink races, special files, large bodies, and unsafe downloads.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Static, self-contained, redacted HTML reports from the versioned report models.
- [ ] **S-2:** PRD 061 guided authoring and PRD 058 lifecycle status views.
- [ ] **S-3:** Resumable long-running operations with user cancellation and bounded progress events.
- [ ] **S-4:** Import/export project bundles containing manifests and hashes but no source content by default.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Signed desktop packaging that embeds the local workspace process.
- [ ] **C-2:** PRD 068 collaborative service mode after authentication, authorization, audit, and tenancy designs exist.
- [ ] **C-3:** Optional local-only accessibility preferences and saved layouts.

### Won't Have (W) — This release :red_circle: `@human-required`

- Hosted/LAN access, accounts, collaboration, plugins, shell/terminal, remote assets, automatic writes, or telemetry by default.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | A new user and valid policy project | Core workflow runs | The user reaches a valid gap report without terminal use |
| AC-2 | A request lacks the launch token or CSRF proof | It attempts mutation | It is rejected without touching project files |
| AC-3 | A path traverses or symlinks outside the project root | It is opened | The operation fails before external content is read |
| AC-4 | A source contains script/HTML payloads | It is rendered | The content remains inert text under CSP |
| AC-5 | A target changes externally after preview | Save is confirmed | The UI detects hash mismatch and refuses last-write-wins |
| AC-6 | The same operation runs through UI and core API | Outputs are compared | Artifacts are byte-equivalent |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | First-session activation | 60% | Moderated/unmoderated pilot |
| Leading | Core task completion | 4 of 5 users without assistance | Usability test |
| Leading | Provenance comprehension | 4 of 5 within two minutes | Timed task |
| Leading | Security boundary | 100% seeded attacks blocked | Automated security suite |
| Lagging | Repeat use | Three organizations reopen and update projects within 60 days | Opt-in pilot evidence |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** Stable typed core interfaces for existing commands plus PRDs 055 and 056.
- **Phase 1:** Read-only inventory, validation, reports, and traceability prototype.
- **Phase 2:** Manifest editing, previews, safe writes, mapping/applicability workflow, and accessibility gate.
- **Phase 3:** Static reports, guided authoring/lifecycle views, packaging, and pilot release.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Local web server exposes filesystem authority | Host data loss/disclosure | Loopback, capability token, strict root, safe open/write, no shell |
| UI and CLI behavior diverge | Incorrect artifacts | Shared core APIs and parity fixtures |
| GUI becomes a premature SaaS platform | Delivery failure | Explicit local-only non-goals and separate collaboration PRD |
| Untrusted policy prose creates XSS | Code execution/data exposure | Escaping, CSP, no raw HTML rendering |
| Accessibility is deferred | Excludes target users | Core WCAG gate before write workflows ship |

## Open Questions :yellow_circle: `@human-review`

- **[Engineering, blocking]** Which current command modules must be refactored into application services before UI work begins?
- **[Security, blocking]** Should the first release open the system browser or embed a hardened desktop webview?
- **[Design, blocking]** Which single golden-path workflow should drive the first prototype: conversion, mapping review, or gap analysis?
- **[Product, non-blocking]** Should static HTML reports precede all mutation capabilities as the smallest adoption experiment?

## Definition of Ready :red_circle: `@human-required`

- [ ] Product approves the local-only MVP and golden workflow.
- [ ] Engineering approves the shared-core boundary and project format.
- [ ] Security approves the local-server threat model and test matrix.
- [ ] Design completes a keyboard-accessible prototype with five target-user tests.
- [ ] Every Must Have maps to an executable acceptance/security test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-08-24 | Start local-first and single-user | Validates workflow value without premature SaaS obligations | Hosted multitenant service |
| 2026-08-24 | Call shared Rust APIs, not the shell | Prevents command injection and behavior drift | CLI subprocess wrapper |
| 2026-08-24 | Keep manifests as source of truth | Preserves reviewability and CLI interoperability | UI database only |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-24 | Codex | Initial draft for a secure local-first FORGE web workspace |

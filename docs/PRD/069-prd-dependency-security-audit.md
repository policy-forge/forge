# 069-prd-dependency-security-audit

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-09-11 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `069-dependency-security-audit`
**Created**: 2026-09-11
**Status**: Draft
**Input**: PR #146 supply-chain finding (CWE-1104) and the 293 pre-existing `safe-to-deploy` exemptions

---

## Executive Summary :yellow_circle: `@human-review`

FORGE will audit its dependency graph to recorded criteria, replace blanket exemptions
with auditable records or named expiring exceptions, and gate new dependencies in CI.
The MVP produces offline-verifiable evidence that a reviewer can check without trusting
FORGE; it does not certify third-party code, replace advisory scanning, or change
licence policy.

## Context

### Background :red_circle: `@human-required`

`supply-chain/config.toml` currently exempts every dependency the project has not
audited: 293 entries before the PRD-062 workspace tranche, plus the 26 versions that
tranche requires. `cargo vet --locked` therefore passes because nothing is required to
be reviewed, not because anything was. `supply-chain/audits.toml` holds only the
cargo-vet scaffold header. `cargo audit` and `cargo deny` cover known advisories and
licences; neither answers "has a human reviewed this crate's source for what it can do
at runtime".

### Evidence and Product Hypotheses :yellow_circle: `@human-review`

| Type | Observation | Product Implication |
|------|-------------|---------------------|
| Repository evidence | 293 exemptions pre-date this PRD; 26 more arrived with the PRD-062 network stack; total unvetted `safe-to-deploy` crates is the entire graph. | The release gate currently rests on unreviewed third-party code. |
| Repository evidence | `cargo vet`, `cargo audit` and `cargo deny` already run in CI, so the tooling and the CI seam exist. | The work is review capacity and policy, not new infrastructure. |
| Product boundary | An audit is a human judgement about a specific version and criteria. | Exemptions must be owned and time-boxed; audits must be attributable. |
| Product hypothesis | Auditing the runtime stack first shrinks real risk faster than auditing uniformly. | Sequence by what ships and what parses untrusted input, not alphabetically. |

### Scope Boundaries :yellow_circle: `@human-review`

**In Scope:**

- A deterministic, offline inventory of every locked crate with its review status
- Recorded audit criteria and the class each crate belongs to (`safe-to-deploy` for
  shipped runtime code, `safe-to-run` for test and build-only code)
- Replacement of exemptions with audit records, differential audits, or named
  expiring exceptions
- A CI gate that fails on a **new** unvetted `safe-to-deploy` dependency
- First tranche: the PRD-062 loopback transport and session stack and its transitives
- Exemption age reporting and a lockfile-driven refresh cadence

**Out of Scope:**

- Penetration testing, SAST, fuzzing or runtime hardening (PRD-062's security record
  owns the workspace threat model)
- Licence and advisory policy, which `cargo deny` and `cargo audit` already enforce
- Certifying, endorsing or claiming safety of third-party code
- Vendoring, forking or patching upstream crates
- Reducing the dependency graph; that is a separate design decision if it is wanted
- Any change to runtime behaviour, public API, or supported platforms

### Related Documents :white_circle: `@auto`

| Document | Relationship |
|----------|--------------|
| `supply-chain/config.toml`, `audits.toml`, `imports.lock` | The store this PRD governs |
| `docs/PRD/062-prd-local-web-workspace.md` | First tranche: the loopback transport and unlock stack |
| `docs/SEC/062-sec-local-web-workspace.md` | Threat model whose mitigations depend on those crates |
| `docs/PRD/054-prd-oscal-1-2-3-compatibility.md` | Existing pin-and-record practice for schemas |
| `docs/plans/2026-09-11-066-mvp-reuse-plan.md` | Example of recording decisions and evidence per tranche |

---

## Problem Statement :red_circle: `@human-required`

A `safe-to-deploy` exemption asserts that nobody has reviewed the crate. Because it is
indistinguishable from an audit in the passing CI job, the project cannot tell a
reviewed dependency from an unreviewed one, and a release that depends on the loopback
listener, the password KDF or the async runtime ships on unreviewed third-party code.

## Goals :red_circle: `@human-required`

| ID | Goal | Measurable Outcome |
|----|------|--------------------|
| G-1 | Make the dependency graph legible. | One deterministic, offline inventory names every locked crate with its status, criteria and reviewer or exception owner. |
| G-2 | Audit what ships. | 100% of `safe-to-deploy` runtime crates carry an audit, a differential audit, or a named unexpired exception at the MVP exit. |
| G-3 | Stop unvetted additions. | A new unvetted `safe-to-deploy` dependency fails CI until audited or excepted. |
| G-4 | Make bumps cheap. | A version bump is reviewed as a delta against the audited version, not re-audited from scratch. |
| G-5 | Keep the store honest. | Every exception has an owner and a review date, and stale exceptions are reported. |

## Non-Goals :red_circle: `@human-required`

- FORGE does not certify that a crate is free of defects or backdoors.
- The audit does not replace threat modelling, tests or review of FORGE's own code.
- Audits are not a licence or vulnerability gate; those remain `cargo deny` and `cargo audit`.
- The MVP does not audit the whole graph to completion; it audits the runtime stack and
  establishes the mechanism for the rest.

## Personas and Prioritized User Stories :red_circle: `@human-required`

### US-1 — See what is unreviewed (P0)

> As a maintainer, I want one inventory of what is audited, imported or exempted so that I
> know where the honest gaps are.

### US-2 — Review a dependency change (P0)

> As a reviewer, I want the audit record and the delta for a version bump so that I can
> approve it on evidence rather than on the absence of a complaint.

### US-3 — Trust the gate (P0)

> As a release manager, I want CI to fail on a new unreviewed runtime dependency so that
> the supply-chain job means something.

### US-4 — Add a dependency properly (P1)

> As a contributor, I want a documented, bounded path for adding a crate so that a
> legitimate need is not blocked by an unbounded audit requirement.

## Audit Model :yellow_circle: `@human-review`

Four evidence kinds, all recorded in the store and surfaced by the inventory:

- **Audit** — a reviewer examined the crate at an exact version against a criterion.
- **Differential audit** — a reviewer examined the diff between two exact versions and
  inherits the prior audit for the unchanged remainder.
- **Imported audit** — an audit from a configured, pinned trusted source, refreshed
  explicitly rather than silently.
- **Exception** — an exemption that carries an owner, a rationale, a review date and a
  bounded validity, and that the inventory reports with its age.

The gate fails on a **new** unvetted `safe-to-deploy` dependency; it does not fail merely
because legacy exemptions still exist, so progress is possible without a flag day. Exit
status follows the repository convention: `0` clean, `1` review action required (new
unvetted crate or stale exception), `2` invalid store or input.

## Requirements

### Must Have (M) — MVP launch blockers :red_circle: `@human-required`

- [ ] **M-1 — Inventory:** Produce a deterministic, offline per-crate inventory report (status, criteria, version, auditor or exception owner, age) without adding runtime dependencies.
- [ ] **M-2 — Criteria:** Document the criteria and record which class each crate belongs to; test and build-only crates are `safe-to-run`, shipped runtime code is `safe-to-deploy`.
- [ ] **M-3 — Runtime stack first:** Audit the PRD-062 transport, session and crypto stack and its transitives before any further feature work depends on them.
- [ ] **M-4 — Audits recorded:** Every exemption removed for the runtime stack is replaced by an audit or differential audit entry in `supply-chain/audits.toml`.
- [ ] **M-5 — Exceptions owned:** An exemption requires an owner, a rationale, and a review date, and appears in the inventory with its age.
- [ ] **M-6 — Gate:** CI fails when a new unvetted `safe-to-deploy` dependency appears without an audit or a registered, unexpired exception.
- [ ] **M-7 — Differential workflow:** Document and exercise the differential path (exact old and new version, review notes) so patch bumps do not require a full re-audit.
- [ ] **M-8 — Imports pinned:** Imported audits come only from the configured trusted sources, are pinned, and are refreshed explicitly; the refresh is a reviewable change.
- [ ] **M-9 — Stale reporting:** Report every exception past its review date, and fail the review status for the run.
- [ ] **M-10 — Offline:** The inventory and the gate run offline against the committed store and lockfile.
- [ ] **M-11 — Provenance cross-check:** Reconcile the audited set with the vendored schema manifest and the release provenance/checksum claims.
- [ ] **M-12 — Exit contract:** Exit `0` clean, `1` review action required, `2` invalid store or input, consistently across the inventory and gate commands.
- [ ] **M-13 — Tests:** Cover the gate (new crate, audited crate, excepted crate, stale exception), the inventory determinism, the exception validation, and the differential record.
- [ ] **M-14 — Documentation:** A maintainer guide states the criteria, the record format, the exception process, and what an audit does and does not claim.

### Should Have (S) — High-value fast follows :yellow_circle: `@human-review`

- [ ] **S-1:** Reduce the exempted surface by identifying unused or duplicated transitive crates.
- [ ] **S-2:** Machine-readable trend report (audited vs excepted counts and median age) recorded per release.
- [ ] **S-3:** Automated reminder when a dependency bump introduces a new unvetted version.
- [ ] **S-4:** Publish the inventory with release artifacts so consumers can see the reviewed set.

### Could Have (C) — Future considerations :green_circle: `@llm-autonomous`

- [ ] **C-1:** Reproducible-build verification for released binaries.
- [ ] **C-2:** Sigstore/attestation linkage for audited crates.
- [ ] **C-3:** A per-release machine-checkable statement of the audited dependency set.

### Won't Have (W) — This release :red_circle: `@human-required`

- Claiming that an audit establishes safety, correctness or absence of vulnerabilities.
- Auditing crates that are not in the shipped graph, or fork maintenance.

## Acceptance Criteria — Given / When / Then :yellow_circle: `@human-review`

| ID | Given | When | Then |
|----|-------|------|------|
| AC-1 | A fresh clone with no network | The inventory runs | Every locked crate appears once with status, criteria and owner or exception age |
| AC-2 | A newly added unvetted `safe-to-deploy` dependency | The gate runs | Exit is `1` and the crate is named |
| AC-3 | An exemption with no owner or review date | The store is validated | Exit is `2` and the entry is named |
| AC-4 | A patch bump of an audited crate | A differential audit is recorded | The bump is accepted without a full re-audit and the delta is named |
| AC-5 | An exception past its review date | The inventory runs | The exception is reported stale and the run exits `1` |
| AC-6 | The runtime stack after the first tranche | The store is checked | No `safe-to-deploy` crate in that set relies on a bare exemption |
| AC-7 | The same lockfile and store | The inventory runs twice | Output bytes are identical |

## Success Metrics — Hypotheses :red_circle: `@human-required`

| Type | Metric | Target | Measurement |
|------|--------|--------|-------------|
| Leading | Runtime crates audited | 100% at MVP exit | Inventory report |
| Leading | Unowned or stale exceptions | Zero | Inventory report |
| Leading | Time to audit a patch bump | Under one day | Differential records |
| Lagging | Exempted crates overall | Trending down each release | Trend report (S-2) |

## Dependencies and Phasing :yellow_circle: `@human-review`

- **Requires:** the existing `cargo vet` / `cargo audit` / `cargo deny` CI jobs and the
  PRD-062 workspace stack being complete enough to freeze its dependency set.
- **Phase 1:** inventory report, criteria, exception policy, and the CI gate.
- **Phase 2:** audit the PRD-062 transport and session stack and its transitives.
- **Phase 3:** differential audits for the remaining `safe-to-deploy` crates.
- **Phase 4:** cadence, stale reporting and refresh automation.

## Risks and Mitigations :yellow_circle: `@human-review`

| Risk | Impact | Mitigation |
|------|--------|------------|
| Review fatigue produces rubber-stamped audits | False assurance | Named auditors, recorded rationale and deltas, and a store that distinguishes audit from exception |
| Differential audits drift from the audited base | Silent gaps | Require the exact version pair and reject a differential whose base has no audit |
| Imported audits are trusted without inspection | Upstream compromise | Pin imports, record refresh as a reviewable change, and keep the trusted list explicit |
| Effort is underestimated (≈300 crates) | Initiative stalls | Sequence by runtime exposure and accept a bounded, owned exception set as an interim state |
| The gate blocks legitimate contributions | Delivery friction | Document a bounded path (audit, differential, or temporary exception) in the maintainer guide |
| Audits outlive the code they cover | Stale evidence | Stale reporting and refresh on lockfile change |

## Open Questions :yellow_circle: `@human-review`

- **[Engineering, blocking]** Who signs an audit: a human only, or an agent-assisted review with explicit human sign-off, and what must the record say for each?
- **[Product, blocking]** May an exception be permanent with a recorded rationale, or must every exception expire and be revisited?
- **[Engineering, blocking]** Which trusted external audit sources may be imported, and who approves adding one?
- **[Engineering, non-blocking]** Is dev/test-only crates to `safe-to-run` sufficient, or does the release process need `safe-to-deploy` for them too?
- **[Product, non-blocking]** Should the project reduce the graph (deduplicate or drop crates) instead of auditing it, and who owns that evaluation?

## Definition of Ready :red_circle: `@human-required`

- [ ] Engineering approves the criteria and the class assignment rule.
- [ ] Product approves the exception policy (owner, rationale, expiry or permanence).
- [ ] The trusted import list is decided and recorded.
- [ ] The runtime-stack audit is estimated and its auditor named.
- [ ] Every Must Have maps to an executable acceptance test.

## Decision Log :yellow_circle: `@human-review`

| Date | Decision | Rationale | Alternatives Considered |
|------|----------|-----------|-------------------------|
| 2026-09-11 | Gate on **new** unvetted runtime dependencies, not on the total exemption count | A flag day would block all work and incentivise mass rubber-stamping | Fail while any exemption exists; ignore new dependencies |
| 2026-09-11 | Exceptions carry an owner and a review date | An unattributed exemption is indistinguishable from an unreviewed dependency | Exemptions without metadata |
| 2026-09-11 | Sequence by runtime exposure, not alphabetically | The loopback listener and KDF parse untrusted input and ship | Audit the graph uniformly |

## Changelog :white_circle: `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-09-11 | Codex | Initial draft for a full dependency audit: inventory, criteria, owned exceptions, differential audits and a CI gate |

# PRD-061 authoring gates

Open and satisfied gates for
[PRD-061 framework-guided policy authoring](PRD/061-prd-framework-guided-policy-authoring.md).
Phase 1 (PR #144) and Phase 2 (PR #145, merge `3ac6815`) are merged technical
phases; S-1 is implemented in the closeout PR.

Owners recorded the dispositions below on **2026-09-11**. They are asserted human
decisions, attributed to their owner — the same standard the authoring contracts
apply to reviewer metadata. FORGE does not authenticate the decision-maker, prove
independence, or grant rights. Nothing here emits compliant, certified,
implemented, effective or approved language (M-14), and no approval or lifecycle
evidence is created.

## Status

| Gate | Owner | Status |
|---|---|---|
| GATE-API | Engineering owner / release maintainer | **Satisfied** — `2.0.0` decision + migration guidance |
| GATE-PRODUCT | Product owner | **Satisfied** |
| GATE-COMPLIANCE | Compliance owner | **Satisfied** — asserted disposition |
| GATE-LEGAL | Legal owner | **Satisfied** — asserted disposition |
| GATE-ENGINEERING | Engineering owner | **Satisfied** |
| GATE-PARTNERS | Product owner with partner leads | **Satisfied as readiness** — reviewers identified; exercise pending |
| GATE-PILOTS | Product owner | **Open** — no measured data yet |
| GATE-RELEASE | Release maintainer | **Open** — release deliberately held |

## Satisfied dispositions

| ID | Decision (2026-09-11) | Evidence |
|---|---|---|
| GATE-API | The Rust library is a supported public surface. Because `AuthorCommand` and `ForgeError` are public and not `#[non_exhaustive]`, the release carrying PRD-061 is **`2.0.0`** with consumer migration guidance. Version and `Cargo.lock` stay unchanged until that release, which is gated by GATE-RELEASE. | [Authoring library API migration](authoring-api-migration.md); [Phase 2 contracts](authoring-phase2.md); `CHANGELOG.md` |
| GATE-PRODUCT | Authoring states (`planned`, `blocked-context`, `skeleton-ready`, `human-draft-present`), gap-to-topic and topic-to-family assignment semantics, and the disclaimers are accepted as shipped. Draft state is never presented as lifecycle approval. | [Phase 2 contracts](authoring-phase2.md); [Phase 2 acceptance evidence](authoring-phase2-evidence.md); `generated_reports_use_only_authoring_outcome_terminology` |
| GATE-COMPLIANCE | Any **required** answer whose sensitivity is `confidential` or `restricted` must be independently reviewed before dependent clauses are usable; `public` and `internal` answers need only their supplied reviewer provenance. No contract change: the rule maps onto existing `required`, `sensitivity` and `review` fields. FORGE records asserted provenance and cannot authenticate independence. | Pack/project contracts; `missing_stale_invalid_expired_and_no_answer_block_only_the_selected_section`; `protected_values_and_secret_names_fail_before_rendering` |
| GATE-LEGAL | The portable pack contract remains **IDs-only**; control titles are never bundled. Titles appear only when rendered locally from user-supplied framework inputs under the pack's `content_rights` attestation. | `schemas/authoring-pack.schema.json` (`control_assignment` has no title); [Phase 2 contracts](authoring-phase2.md) |
| GATE-ENGINEERING | The closed `/1` and `/2` schemas, the provenance graph, the byte/record/span bounds and single atomic no-replace multi-output publication are approved. One build emits every selected policy as one generation; sequential-rollback atomicity is explicitly not claimed; Linux/macOS are the supported publication platforms. This resolves the non-blocking engineering open question. | [Phase 2 interface freeze](authoring-phase2-plan.md); [Phase 2 acceptance evidence](authoring-phase2-evidence.md); `output.rs` interruption/atomicity cases |
| GATE-PARTNERS | Three design-partner reviewers are identified at the owner's employer and will supply lawful framework inputs and real gap-to-draft workflows through the documented offline onboarding path. Organization and individual names are intentionally **not recorded in the repository**; reviewers are referred to only by pseudonymous keys. This satisfies readiness; the measured exercise is GATE-PILOTS. | [Phase 2 contracts](authoring-phase2.md) onboarding path; synthetic examples and the full CLI/contract suite |

## Open gates

| ID | Owner | What is still required |
|---|---|---|
| GATE-PILOTS | Product owner | Measured before/after results from the identified partners using the protocol below. The numeric 50% target was retired as an unsupported claim; the remaining success-metric values are unvalidated hypotheses, not commitments. |
| GATE-RELEASE | Release maintainer | Release approval. Per the owner's 2026-09-11 decision the tranches are **held out of any release** until the remaining PRDs in flight are finished and GATE-PILOTS produces data; the `2.0.0` decision from GATE-API takes effect only in that release. |

### Pilot measurement protocol (supersedes the retired 50% claim)

Once partners exercise real workflows, record:

- Median time to a reviewable skeleton (`skeleton-ready` or
  `human-draft-present`) per policy, before vs. after, over at least five policy
  workflows.
- Revision burden: maintainer edits needed per draft.
- Draft retention: share of skeleton sections retained through human review.

Until those measurements exist, no numeric time-savings claim is made.

## Definition of Ready status

| Definition-of-Ready item | Status |
|---|---|
| Product and Compliance approve authoring states, assignment semantics and disclaimers | **Satisfied** — GATE-PRODUCT, GATE-COMPLIANCE (2026-09-11) |
| Legal approves authoring-pack content boundaries | **Satisfied** — GATE-LEGAL (2026-09-11) |
| Engineering approves schemas, provenance graph, bounds and atomic multi-output behavior | **Satisfied** — GATE-ENGINEERING (2026-09-11) |
| Three design partners supply lawful framework inputs and real gap-to-draft workflows | **Satisfied as readiness** — GATE-PARTNERS; measured workflows pending under GATE-PILOTS |
| Every Must Have maps to an executable acceptance test | **Satisfied** — M-1…M-15 mapped to named executable cases in [Phase 2 acceptance evidence](authoring-phase2-evidence.md#must-have-requirement-evidence) |

## Requirement-checkbox reconciliation

The PRD's requirement checkboxes are unchanged except M-13, which was already
checked under its established wording. S-1 is implemented, and S-2/S-3/S-4 have
technical implementation/tests, but their Should-Have boxes remain unchecked:
the PRD's only established check convention is "technical evidence, not human
acceptance", and this register's dispositions do not convert Should-Have
implementation into a checkbox claim. The technical evidence for S-1…S-4 is in
[Phase 2 acceptance evidence](authoring-phase2-evidence.md).

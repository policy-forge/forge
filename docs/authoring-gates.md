# PRD-061 authoring gates

Open human and release gates for
[PRD-061 framework-guided policy authoring](PRD/061-prd-framework-guided-policy-authoring.md).
Phase 1 (PR #144) and Phase 2 (PR #145, merge `3ac6815`) are merged technical
phases; this register records what each gate still needs. Nothing here satisfies
a gate. Technical evidence can inform a decision but never substitutes for the
named human owner's acceptance, and it never emits compliant, certified,
implemented, effective or approved language (M-14).

Agents and maintainers must keep every gate below **open** until the named owner
records acceptance in the PRD or its successor review record.

## Gate register

| ID | Gate | Owner | Decision required | Evidence required | Evidence that already exists |
|---|---|---|---|---|---|
| GATE-API | Public Rust API, semver and migration guidance (retained from PR #144) | Engineering owner / release maintainer | Whether the additive `AuthorCommand`, `ForgeError` and exhaustive-enum changes ship as a compatible release, and what migration guidance consumers receive | Written source-compatibility decision; migration note for downstream exhaustive matches; release-version decision; confirmation that package version and `Cargo.lock` stay unchanged until release | `docs/authoring-phase2.md` records that Phase 2 adds `AuthorCommand` fields/variants and that `ForgeError` is unchanged in the Phase 2 diff; `CHANGELOG.md` `Changed` entry; new S-1 `AuthorCommand::Scaffold` variant feeds the same gate; Phase 1 review retained the gate |
| GATE-PRODUCT | Product acceptance of authoring states, assignment semantics and disclaimers ([Product, blocking] open questions) | Product owner | Approve the authoring state machine (`planned`, `blocked-context`, `skeleton-ready`, `human-draft-present`), gap-to-topic and topic-to-family assignment semantics, and the user-facing disclaimers and output wording | Signed product disposition of the blocking product Open Question and the Definition-of-Ready product line; confirmation that draft/authoring states are never presented as lifecycle approval | Phase 1/Phase 2 usage guides describe the states and non-approval boundary; `docs/authoring-phase2-evidence.md` maps the executable blocking/terminology cases; `generated_reports_use_only_authoring_outcome_terminology` asserts output wording |
| GATE-COMPLIANCE | Organization-answer review classes ([Compliance, blocking] open question) | Compliance owner | Which organization-answer classes always require independent review before dependent clauses are usable | Compliance disposition naming the answer classes and the required independent-review evidence; mapping to pack question definitions and answer records | Question definitions already carry `required`, `owner`, `sensitivity`, `max_age_days`; component and Phase 1 tests prove missing/stale/invalid/expired/no-answer blocking and confidential/secret rejection; no compliance class list exists yet |
| GATE-LEGAL | Authoring-pack content boundaries, including whether control titles may ship ([Legal, blocking] open question) | Legal owner | Whether portable authoring packs may include control titles, or must default to control IDs only; and the content-rights statement the pack must carry | Legal/content boundary decision and approved `content_rights` wording; confirmation that FORGE still bundles no restricted framework content | Pack contract requires a reviewer-supplied `content_rights` statement and `source_label`; synthetic examples only; `docs/authoring-phase2.md` documents user-supplied packs and ID-based findings |
| GATE-ENGINEERING | Schemas, provenance graph, bounds and atomic multi-output behavior | Engineering owner | Approve the closed `/1` and `/2` schemas, provenance graph shape, byte/record/span bounds, and single no-replace multi-output publication semantics (including the [Engineering, non-blocking] "multiple policy files atomically vs one policy per invocation" question) | Engineering disposition covering each closed contract, documented bounds, platform support boundary, and the multi-policy atomicity decision | `docs/authoring-phase2-plan.md` freezes contracts and the acceptance matrix; `docs/authoring-phase2-evidence.md` maps every matrix row to executable cases and reports `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked` (2,337 passed; 0 failed; 3 ignored) and `git diff --check` clean; `output.rs` interruption/atomicity cases |
| GATE-PARTNERS | Design-partner readiness — three design partners | Product owner with partner leads | Which three partners supply lawful framework inputs and real gap-to-draft workflows, under what data-handling terms | Named partners, lawful input confirmation, and a documented onboarding path for real (non-synthetic) packs | Synthetic fixtures, examples and the full CLI/contract suite exist; no partner has been named or onboarded |
| GATE-PILOTS | Phase 3 measured pilots (G-5 time-to-reviewable-skeleton hypothesis) | Product owner | Whether the G-5 hypothesis (five pilots reach a reviewable skeleton 50% faster) is validated, revised or retired, and with what measurement method | Recorded pilot protocol and before/after results for the success metrics (reviewable-skeleton completion, revision burden, draft retention) | Metrics and targets remain hypotheses in the PRD; no design-partner authoring study exists |
| GATE-RELEASE | Release approval | Release maintainer | Whether the technically merged authoring tranches ship in a release, with which version and release notes | Release review confirming CI, changelog, docs, migration guidance and all other gates dispositioned | Merged Phases 1–2 plus closeout evidence; package version and `Cargo.lock` intentionally unchanged; no release is authorized |

## Definition of Ready status

| Definition-of-Ready item | Status |
|---|---|
| Product and Compliance approve authoring states, assignment semantics and disclaimers | Open — GATE-PRODUCT, GATE-COMPLIANCE |
| Legal approves authoring-pack content boundaries | Open — GATE-LEGAL |
| Engineering approves schemas, provenance graph, bounds and atomic multi-output behavior | Open — GATE-ENGINEERING (technical evidence available) |
| Three design partners supply lawful framework inputs and real gap-to-draft workflows | Open — GATE-PARTNERS |
| Every Must Have maps to an executable acceptance test | Technical evidence available for M-1…M-15; M-13 checked under the "technical evidence, not human acceptance" convention. The remaining requirement boxes stay unchecked because human/product acceptance is still pending |

## Requirement-checkbox reconciliation

The PRD's requirement checkboxes are unchanged by the closeout except M-13, which
was already checked under its established wording. S-1 is now technically
implemented (`forge author scaffold`), and S-2/S-3 already have Phase 2 technical
implementation, but their Should-Have boxes remain unchecked: checking them would
claim product acceptance that GATE-PRODUCT and GATE-RELEASE still hold, and the
Phase 2 disposition already declined to expand checkbox scope. The technical
evidence for S-1, S-2 and S-3 is recorded in
[docs/authoring-phase2-evidence.md](authoring-phase2-evidence.md) instead.

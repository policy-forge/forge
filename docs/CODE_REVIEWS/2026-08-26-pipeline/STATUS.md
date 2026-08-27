# Code-Review Remediation Pipeline — Status

Source review: `docs/CODE_REVIEWS/ocr_review_2026-08-16.md` (1,074 findings).
Scope decision (user, 2026-08-26): code/tests/build-CI config only (715 findings);
`specs/`, `.gemini/`, `.specify/`, `docs/` prose excluded (359 out-of-scope).
All severities, staged. Snapshot regeneration allowed for intentional behavior
changes. One commit per tranch.

## Wave 1 — Validation: COMPLETE (715/715 adjudicated)

12 slices at `slices/`, validated against HEAD `b22e2d5` by 12 read-only scouts.
Mid-wave the subagent provider quota exhausted (429, reset 2026-08-28 07:09 UTC);
the user granted a 20k-token bridge on 2026-08-26 and all remaining verdicts were
recovered the same day (compact-verdict protocol + scouts persisting full reports).

**Consolidated index: `VALIDATED_FINDINGS.md`** — 634 valid · 53 partial ·
19 invalid · 9 duplicate → **687 actionable** (2 critical [remediated], 53 high,
310 medium, 322 low). Per-finding evidence in `validated/slice01..12.md`
(full reports for slices 01–04, 07, 09, 11–12; compact directive lists for 05,
06, 08, 10 — slice08 carries the scout's full JSON rationale).

## Tranches

| Tranch | Scope | Status |
|---|---|---|
| 1 — criticals | F0480, F1054 fixed; F0383 invalid | **done, commit `c3d68d1`** — fmt/clippy/test green (1,784 passed) |
| 2 — high (53) | 52 remediated; F0617 reclassified low by validator | **done, commit pending** — fmt/clippy/test green (full suite passed); intentional fixtures + snapshots regenerated |
| 3 — medium (310) | pending | active next |
| 4 — low (322) | pending | pending |

## Remediation grouping (for tranch 2+)

Group by file ownership, not severity, to avoid edit conflicts. Hotspots (citations
≥6): `src/config.rs`, `src/migration/inventory.rs`, `src/oscal_cli/invoker.rs`,
`src/export/xml_deserializer.rs`, `src/io.rs`, `src/pipeline.rs`,
`.github/workflows/release.yml`, `src/mapping/inventory.rs`, `src/model/assemble.rs`,
`src/oscal/assessment_plan.rs`, `src/framework/mod.rs`, `src/mapping/baseline.rs`,
`src/parse/modality.rs`, `tests/common/fixture_generator.rs`,
`tests/golden_file_tests.rs`, `benches/*`, `ci/integration-test.sh`, `src/ingest/mod.rs`.

Cross-finding clusters (validator-suggested, fix together):
- Shared bounded-read helper in `io::` : F0432/F0454/F0484/F0485/F0507/F0540 (+F0330/F0372/F0395/F0750/F0804 same pattern).
- Exit-code contract cluster: F0473/F0474/F0527/F0501 (+F0405/F0408/F0816).
- UUID-seed determinism family: F0298, F0560, F0585, F0589, F0595, F0607, F0713, F0774 — coordinate in one tranch (snapshot impact).
- Title-truncation + front-matter parsing: F0006/F0007 (golden snapshots will change; regenerate with `cargo insta accept` per user policy).

## Rules for remediation agents

1. Work only from `VALIDATED_FINDINGS.md` + the cited slice entry; PARTIAL findings
   need the validator's corrected premise read first.
2. Never edit files outside your ownership group; report cross-group needs via hub.
3. Scoped `cargo test` per fix; no project-wide fmt/clippy mid-tranch (integration
   owner runs the full gate: fmt, clippy --all-targets -D warnings, test, bench
   smoke, audit, deny).
4. Snapshot changes only where behavior intentionally changed (user-approved);
   list every regenerated snapshot in the tranch commit.

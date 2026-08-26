# Code-Review Remediation Pipeline — Status

Source review: `docs/CODE_REVIEWS/ocr_review_2026-08-16.md` (1,074 findings).
Scope decision (user, 2026-08-26): code/tests/build-CI config only (~715 findings);
`specs/`, `.gemini/`, `.specify/`, `docs/` prose excluded. All severities, staged.
Snapshot regeneration allowed for intentional behavior changes. One commit per tranch.

## Wave 1 — Validation (read-only scouts)

12 slices (~60 findings each) at `slices/`. Outcome:

| Slice | Findings | Status | Report |
|---|---|---|---|
| slice01 | 60 (3 critical, 6 unspecified→high, 51 high) | validated in scout context; **verdicts pending quota reset** | narration at `narration_slice01.md` |
| slice02 | 60 (9 high, 51 medium) | **recovered** — 54 valid, 4 partial, 2 duplicate, 0 invalid | `validated/slice02.md` |
| slice03 | 62 medium | validated in scout context; **verdicts pending quota reset** | `narration_slice03.md` |
| slice04 | 63 medium | **recovered** — 47 valid, 13 partial, 3 invalid | `validated/slice04.md` |
| slice05 | 60 medium | validated in scout context; **verdicts pending quota reset** | `narration_slice05.md` |
| slice06 | 61 medium | validated in scout context; **verdicts pending quota reset** | `narration_slice06.md` |
| slice07 | 60 (27 medium, 33 low) | **recovered** — 56 valid, 1 partial, 3 invalid | `validated/slice07.md` |
| slice08 | 61 low | **recovered** — 59 valid, 1 partial, 1 invalid (JSON in session artifact; report pending) | — |
| slice09 | 60 low | validated in scout context; **verdicts pending quota reset** | `narration_slice09.md` |
| slice10 | 60 low | validated in scout context (42 valid / 18 partial per scout); **verdicts pending quota reset** | `narration_slice10.md` |
| slice11 | 61 low | **recovered** — 58 valid, 3 duplicate | `validated/slice11.md` |
| slice12 | 47 low | validated in scout context; **verdicts pending quota reset** | `narration_slice12.md` |

**Blocker**: the subagent model quota was exhausted mid-wave (429 insufficient_quota,
resets 2026-08-28 07:09 UTC). Seven scouts finished verifying their slices but could
not emit reports or JSON. Their session contexts persist
(`~/.omp/agent/sessions/-repos-forge/2026-08-26T20-42-00-330Z_*/ValSliceNN.jsonl`);
after the reset, one `hub send` per scout ("reply with your JSON verdict payload")
recovers the full verdicts without re-validation.

## Criticals (tranch 1) — adjudicated and remediated by Main

| ID | Site | Verdict | Action |
|---|---|---|---|
| F0480 | `src/lifecycle/mod.rs` + `record.rs` | **VALID (critical)** | Remediated 2026-08-26. Stored artifact paths must be relative: `validate_artifact_path_shape` rejects absolute, Windows drive-prefixed, and `.`-component paths at record parse; `confined_join` re-asserts at all four join sites (`current_artifacts` source + artifacts, `execute_transition` proposal-alias loop, `validate_report_destination`). `..` components remain permitted — codified contract (`relative_path_handles_sibling_directories`). Tests: `record::tests::artifact_paths_must_be_relative`, `tests::confined_join_rejects_anchor_discarding_paths`. |
| F1054 | `.github/workflows/release.yml:170` | **VALID (critical)** | Remediated 2026-08-26. SLSA generator pinned to commit SHA `f7dd8c54c2067bafc12ca7a55595d5ee9b75204a` (= v2.1.0, verified via GitHub API). |
| F0383 | `src/config.rs` symlink-containment walk | **INVALID** | The described attacks are already rejected: `resolve_inside_root` lexically normalizes and rejects `..` escapes before the walk; the walk canonicalizes the deepest *existing* ancestor so symlink-expanded prefixes fail `starts_with(canonical_root)`. The review's exact attack shape (dir symlink → outside, non-existent tail) is codified as rejected by `symlinked_output_path_escape_is_rejected` (config.rs tests). No residual found in code trace. |

Validation quality note: baseline HEAD `b22e2d5` already contains hardening that
postdates parts of the review (e.g. successor-map symlink race); scouts judged
against current HEAD, not the review date.

## Wave 2 — Remediation plan (post-reset)

1. Re-ping the 7 stalled scouts → recover verdict JSON → write `validated/sliceNN.md`.
2. Merge all 12 verdict sets into `VALIDATED_FINDINGS.md` (valid/partial only,
   deduped), grouped by file ownership for conflict-free parallel remediation:
   hotspots `src/config.rs` (10), `src/migration/inventory.rs` (10), `src/oscal_cli/invoker.rs` (9),
   `src/export/xml_deserializer.rs` (9), `src/io.rs` (9), `src/pipeline.rs` (9), … 148 distinct files.
3. Tranches: critical+high → medium → low; each tranch ends with full CI gate
   (`fmt --check`, `clippy --all-targets -D warnings`, `test`, bench smoke, audit, deny)
   and one commit referencing this directory.
4. Slice04 cross-finding ordering suggestion (from scout): shared bounded-read helper
   in `io::` fixes F0432/F0454/F0484/F0485/F0507/F0540 together; exit-code contract
   cluster F0473+F0474+F0527+F0501.

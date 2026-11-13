# Dependency review packet — 2026-11-12 (PRs 261–265)

Five open Dependabot PRs have bounded **agent-assisted** differential review
evidence. This packet provides replayable source, advisory and hosted-check
receipts; it does **not** record accepted source audits, human signoff, merge
permission or full F03 acceptance. No dependency, lockfile or cargo-vet policy
was changed by this review. PR #260 (zip) is handled separately and is excluded.

| PR | Exact reviewed head | Version scope | Review |
|---|---|---|---|
| [261](https://github.com/policy-forge/forge/pull/261) | `e15c5a7861069aa19c2d5487933647052908363e` | hyper 1.11.1 → 1.12.0 | [Note](pr261/README.md) |
| [262](https://github.com/policy-forge/forge/pull/262) | `335bb6a1414172fbf0b8e4d58d55f2ad2c413afb` | jsonschema family 0.57.0 → 0.60.2 (MINOR) | [Note](pr262/README.md) |
| [263](https://github.com/policy-forge/forge/pull/263) | `df3c748424452f2e948e97c5830cd0605169b6de` | serde_json 1.0.151 → 1.0.154 | [Note](pr263/README.md) |
| [264](https://github.com/policy-forge/forge/pull/264) | `7158f37870de2936c8d9e903e51da96bdda61cf8` | quick-xml 0.41.0 → 0.42.0 (MINOR) — **build broken** | [Note](pr264/README.md) |
| [265](https://github.com/policy-forge/forge/pull/265) | `24616651b0f72a81d6d23682f8370139301ef516` | zip 8.6.0 → 9.0.0 in retained 2026-10-02 evidence manifests only | [Note](pr265/README.md) |

All share base `b4226bb35d796222c30f93235ac22118dd26a344`. All five were OPEN
at review time (2026-11-12). Hosted runs: 41067129491, 41067147939,
41067162528, 41067183887 (2026-10-30) and 41756372632 (2026-11-04); each run's
`headSha` equals the reviewed head above. Job-level conclusions are retained
per PR (`pr*/ci/jobs.jsonl`) because the `gh pr checks` rollup can lag.

## Red flags

- **PR 264 (quick-xml 0.42): does not compile.** The bytes→str name/attribute
  API change breaks 22 call sites in `src/cli/export.rs`, `src/ingest/mod.rs`,
  `src/oscal_cli/invoker.rs`. Reproduced locally (exit 101) and confirmed by
  all three hosted Test jobs. XML round-trip behavior could not be exercised;
  a source migration plus full human audit is required before any merge. The
  migration is tracked on branch `feat/quick-xml-0.42-migration` (worktree
  `forge-quickxml`, branched from this base; not yet pushed at review time).
- **PR 265 edits immutable receipts.** It bumps zip only inside the retained
  2026-10-02 evidence lockfiles, invalidating that packet's SHA-256 index.
  Recommend close-without-merge; alert dismissal/re-baselining is an owner
  action, not a Dependabot merge.
- **PR 262 (jsonschema 0.60)** passes all 78 executed validation tests, but
  resolver vocabulary precedence and `$ref` error-location semantics changed;
  full human audit still required.

## Alert provenance (owner correction, 2026-11-12)

Verified by the P0 owner via the Dependabot API and `cargo audit` on main:
**all 25 open Dependabot alerts target the retained evidence lockfiles**
`docs/dependency-reviews/2026-10-02/pr16{0..4}/Cargo.lock` (zip 8.6.0), **not**
the shipping root `Cargo.lock` (zip 9.1.0, outside every advisory range;
`cargo audit` on main shows no zip finding). Consequently none of these five
PRs closes a live shipping vulnerability; PR #260 is defense-in-depth
hardening, not a shipping-vulnerability fix. The live `cargo audit` failures
on main are pulldown-cmark RUSTSEC-2026-0414/0433 and rustix RUSTSEC-2026-0486
(informational), handled on branch `fix/p0-supply-chain-security`; this packet
records their status only. The evidence lockfiles are immutable receipts:
this packet never proposes editing them, and dismissal is an owner action.

## Shared advisory baseline

Local `cargo audit --no-fetch` (cargo-audit 0.22.1, advisory DB commit
`999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b`, 1490 advisories) on the base lock
([audit-base.json](audit-base.json)) and on every candidate root lock
(`pr*/audit/cargo-audit.json`) returns the **same** findings:

- Vulnerabilities: RUSTSEC-2026-0433 and RUSTSEC-2026-0414, both
  pulldown-cmark 0.13.4 (wikilink memory-exhaustion; patched ≥ 0.13.6 /
  ≥ 0.13.5). Baseline, not introduced by any candidate.
- Warnings: ttf-parser / unsafe-libyaml unmaintained
  (RUSTSEC-2026-0192, RUSTSEC-2024-0449); rustix 1.1.4 unsound
  (RUSTSEC-2026-0486, informational).

No candidate introduces or resolves an advisory in the shipping graph. The
hosted Supply-chain audits job fails on the same baseline advisories (its DB
had 1395/1403 advisories at run time, hence 0414-only on the October runs);
deny/vet steps never executed. The Dependency inventory gate fails on every
root-lock bump because the frozen F02 graph drifts (reproduced for PR 261 in
`pr261/ci/dependency-graph-check.json`); PR 265 leaves the root graph
unchanged and fails the gate on the standing review-required backlog instead.

## Executed candidate checks (this review)

Throwaway `git worktree` checkouts under `/tmp/deprev-2026-11-12/` (never under
`repos/`), macOS arm64, rustc 1.99.1, `--locked` throughout:

| PR | Check | Result |
|---|---|---|
| 261 | `cargo check --lib` | pass |
| 262 | `cargo test` validate/api_contract/schema_provenance/oscal_1_2_3/fixture_validity/ocr_schema | 78 passed, 0 failed |
| 263 | `cargo check --lib`; `cargo test` golden_file/golden_edge_case/fixture_determinism | pass; 50 passed, 0 failed |
| 264 | `cargo test` xml/export/round-trip/golden surface | **build failure, 22 errors, exit 101** |
| 265 | root audit; graph check; inventory gate | baseline advisories; qualified; review-required |

These are candidate executions on one platform, not the pinned qualification
matrix; the hosted three-OS Test jobs (pass for 261/262/263/265, fail for 264)
are the cross-platform record.

## Read the retained evidence

[retained-evidence-index.json](retained-evidence-index.json) is the SHA-256 and
byte-count index of every retained file (self-excluded). Each `pr*/` directory
holds: `receipt.json` (immutable head/base, run URL, scope), `Cargo.lock` +
`Cargo.lock.diff` + `lock-transitions.json` (exact checksums), `audit/`
(cargo-audit JSON), `ci/` (run + job conclusions, supply-chain logs, gate
output), `deltas/` (old→new registry source diffs, `.cargo_vcs_info.json`
omitted), and executed `checks/`/`tests/` logs where applicable. PR 265
additionally retains per-manifest audits of the three evidence lockfiles in
`pr265/audit-evidence/`.

Source diffs were produced with `diff -ru` between the exact registry versions
named in each lock transition (sources fetched via `cargo fetch --locked` in
the throwaway worktrees; uuid 1.24.1 extracted from its `.crate` archive).
They are review aids, not source-audit acceptance.

## Disposition

Agent-assisted review only. No accepted audit, no human signoff, no merge
permission is recorded for any of the five PRs. Predecessor versions remain
exempted rather than locally source-audited; these differentials cannot
inherit that approval. Full human audit and the F03 acceptance gate remain
open.
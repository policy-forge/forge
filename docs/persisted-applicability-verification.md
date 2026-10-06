# Persisted applicability count verification

The existing parser now rejects an overflowing classification sum with `ForgeError::Validation("Invalid persisted applicability report")`. It preserves representable filtered denominators and complete parsed reports. This is a focused F12 prerequisite for an existing runtime boundary; PRD 068 portable queues and their owner/human acceptance gates remain open.

The baseline is `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. Final `src/applicability/mod.rs` is 75,602 bytes, SHA-256 `5d1c246314497af08c33882a3cda7ce27a390181a066c77b0cfd6fda3a1cff72`. The [JSON record](persisted-applicability-verification.json) binds source snapshots, raw command receipts, fixtures/outcomes, LLVM reports, preserved executables and planning provenance. The [focused plan](plans/2026-10-02-f12-persisted-report-counts.md) defines this slice.

## Observed behavior

The baseline execution used the new regression tests over source `d21ec48…`, whose production prefix is byte-identical to immutable `aef0ab24`. Both baseline modes report seven passes and two failures. The default test mode catches all 30 ordered MAX-plus-one parser panics. Explicit `profile.test.package.forge.overflow-checks=false` instead accepts all 30 wrapped sums. In both modes all seven zero/MAX-plus-zero positive cases preserve the entire parsed JSON.

Both baseline workspace consumer tests stop at their first overflow-rejection assertion. Ordinary and representable-mismatch consumer assertions are not reached and receive no baseline red credit. The overflow-disabled consumer returning true is source/failed-assertion inference, not a separately printed boolean result.

Final source runs reject all 30 pairs with the exact Validation payload and Display message `Validation error: Invalid persisted applicability report`; no caught panic or wrapped acceptance is recorded. All seven positive cases preserve entire JSON. The final workspace registration test completes its overflow rejection, ordinary acceptance and total-mismatch rejection assertions. This reaches `services::validate_bytes` at line 86, not an unused validator. The parser also serves historical snapshot metadata, report rendering and staleness at lines 181, 563 and 635. `report_view` retains `validation-failed` / `The committed report is invalid.` / retryable false.

No new CLI exit-code assertion was run. The existing generic Validation mapper is exit 3 at `src/error.rs:560–561`; this slice does not claim exit 2 or introduce a CLI report-import command.

## Final local checks

| Execution | Passed | Failed | Ignored | Scope |
|---|---:|---:|---:|---|
| Fresh default LLVM run | 1,868 | 0 | 0 | Library 1,822; applicability CLI 35; workspace CLI 11 |
| Forge test-profile overflow checks disabled | 9 | 0 | 0 | Persisted-report unit filter, all 37 direct fixtures |
| Standalone default suite | 2,695 | 0 | 3 | 69 test summary lines |

Final standalone `cargo clippy --locked --all-targets -- -D warnings` exits zero. The suite ignores are the existing WI-31 `--set-param` cases: `golden_include_with_params`, `edge_conflicting_set_param`, and `schema_with_set_param`. Counts describe separate executions and are not added together as unique tests. No separate uninstrumented final default nine-test run is claimed.

These runs use Rust 1.98.1 on macOS ARM64, native usize64 and default features. The explicit overflow-disabled Forge test profile demonstrates the prior wrapping behavior; it is not a release build, MSRV or other-platform result. Fixtures are synthetic development inputs, not authenticated inventories or independent acceptance evidence.

## Coverage and source documentation

Fresh final coverage v2 (`fb0f913e…`, 73,845 bytes) uses every unique raw LCOV DA entry before `#[cfg(test)]` line 1428. Production is **1,024/1,160**, with all **136 zero-count lines** retained. The parser span 1321–1426 is a separate **79/82** metric; zeros are **1338, 1355 and 1390**. Exactly added production instrumented lines 1360–1371 are **12/12** covered. Six added Rustdoc lines have no DA entry. All added test lines 1620–1797 and all 18 named test declarations are excluded from production.

Native whole-file line coverage is **1,311/1,497**, including cfg(test) mappings and a different aggregation. It must not replace the production or parser denominator. The JSON preserves all exact zero lines and added-line counts; the pinned producer record preserves all 1,160 line counts and current-source mapping. Current compiler families have 101/124 raw records, distinct from 50 named production declarations. Named-start binding does not establish whole-body proof for every anonymous/generated region. Branch, statement and MC/DC coverage are not established; zero native branch/MC/DC counts mean unavailable evidence.

Selected adjacent Rustdoc is **6/6**: one production parser, two new helpers and three new tests. Whole production is **6/50** (48 standalone declarations plus two methods), leaving **44 legacy undocumented declarations**. Test documentation is separately 6/18 and excluded from production. This is a named source inventory, not a rustdoc compilation or full documentation acceptance claim.

## Retained attempts and remaining gates

The first guard `ad7654…` is retained with its passing focused runs and historical coverage v1. Its strict Clippy attempt exits 101 with three errors in new test code: two similar-name diagnostics and one missing semicolon. Renaming the error bindings and adding the semicolon produced recording-time source `9173c5…`; its format check exits 1 for a line wrap. Format application exits zero and yields final `5d1c246…`. The production prefix is byte-identical across first guard and final source. Fresh final instrumentation supersedes historical v1 for final mapping claims; earlier receipts are preserved unchanged. No lint suppression or weakened assertion was introduced.

The candidate replay v1 mistakenly described enclosing job source pins as absent after reviewing a displayed subset. Retained successor v2 corrects that reviewer statement and replays all five job pins; raw fixture/outcome hashes were not erroneous. Mutable Cargo target paths and source paths nested in old receipts are recording-time identities. Explicit retained snapshots and executables provide current byte pins.

The **mandatory commit hook remains outstanding at this pre-commit recording**. It must run after artifact application without bypass; its real external receipt will be reported with the PR. Standalone local checks do not establish that future hook, exact-head hosted CI, merge or release.

Product first-workflow selection, Compliance disposition/reviewer/quorum/separation rules, Engineering immutable snapshot/reference and queue/response/merge/promotion contracts, and Security/privacy review remain open under D068. Conditional signed envelopes require separate identity/key/trust/revocation/cryptography design and dependency approval. All 17 PRD 068 Must requirements, S-1/S-3/S-4, conditional S-2, AC-1–AC-6, real team workflows/metrics, later adapters and owner/human acceptance remain open. No new dependency, schema, count cap or queue feature is accepted here. The full integrated documentation review after the scoped roadmap packages remains open.

# Workspace bundle query verification

The applied F19 slice adds unreleased API 1.2.0 metadata preview and registered-fingerprint comparison, with maintained `Workspace.bundle_preview()` and `verify_bundle(bundle)` methods. It has local source-bound verification. This record precedes the mandatory commit hook and F19 PR/hosted CI; it establishes no release, full S-6 or human acceptance. Root-provided branch is `codex/f19-workspace-bundle-queries`, stacked on PR177/F12 `5a12daad1f31f33827cde5b50ad6f0915d9e12ec`; parent #23/child #24 are IDs 1418/1419.

## Source and execution

The [JSON record](workspace-bundle-verification.json) retains exact SHA-256/byte pins, commands, raw logs, source snapshots, binary identities and whole zero-line sets. Execution source is bound by `/private/tmp/forge-f19-final-source-and-binaries/index.json` (`9a432586…`, 13,700 bytes; 37 source inputs and 5 binaries). Runtime Rust/Python/OpenAPI/fixtures stayed unchanged. Root later removed two literal patch markers from local-workspace prose and clarified that the 1 MiB POST bound covers the entire JSON body; current guide/local bytes and the exact prose-only correction record are separately pinned. The plan now links this packet through a separately pinned root application record; its historical preparation statement remains intact.

| Recorded execution | Passed / failed / ignored | Scope |
|---|---:|---|
| Full `cargo test --locked` | 2,721 / 0 / 3 | 69 raw summary lines, local default features |
| Selected instrumented run | 1,867 / 0 / 0 | 1,839 library, 9 API-contract and 19 HTTP; 3 summaries |
| Maintained client conformance | Exit 0 | One synthetic workflow; no Rust test denominator |
| Continuous HTTP repeat | 19 / 0 / 0 | Same already-built executable; zero new unique cases |
| Instrumented client repeat | Exit 0 | Same maintained workflow; zero new unique cases |

The F12 full-suite baseline is 2,695/0/3; 17 new unit, 8 HTTP and 1 API test declarations give 2,721 passes. Focused, selected, repeated and full-suite outcomes overlap and are not added together. The 53 schema fixtures include 13 new entries; file counts are separate from test counts.

The client checks the new methods in a read-only machine session over six registrations, including exact ordered hashes/sizes/counts, normalized-index identity and regular-file bytes before/after. It also checks rejected writes and clean shutdown. This evidence covers that synthetic API workflow, not browser behavior, arbitrary Unicode interoperability or full bundle import/export. Jobs record command paths and source pins before/after; later retained binary hashes are recording-time identities, not invented executable-hash attestations at process startup/shutdown. The scope is local Rust 1.98.1 on macOS ARM64; MSRV and other-platform qualification remain open.

## Coverage and its limits

Final producer derivation is `/private/tmp/forge-f19-coverage-v3/coverage.json` (`fcca05f3…`, 205,533 bytes), with index `50ffd392…` and independent raw/producer comparison pins in JSON. Unique LCOV DA uses the maximum count per source line across records, including every mapped line before the first `#[cfg(test)]`; no zero, comment, blank or closing-call line is pruned.

| Unique mapped lines | Covered / total | Zero scope |
|---|---:|---|
| Bundle production before line 238 | 141 /142 | Line 150: `return Err(Error::invalid());` |
| HTTP production before line 711 | 413 /556 | 143 exact zero lines retained in JSON |
| Exact added HTTP runtime lines | 8 /9 | Line 525: closing fallible call `)?,` |
| Bundle cfg(test) partition | 348 /348 | Excluded from production |
| Bundle whole module | 489 /490 | Includes tests; separate denominator |
| HTTP whole file | 471 /614 | Includes tests; separate denominator |

Five new bundle functions have positive production instances: `preview` 36/36 DA lines, `verify_registered` 54/55, `decode_bundle` 28/28, `normalized_index_hash` 3/3 and `current_items` 20/20. The two changed HTTP functions have positive production instances, yielding seven selected authored functions. This is function-entry/line evidence, not complete error-path or branch proof. Native bundle tool totals 498/500 lines, 35/36 functions and 47/49 instantiations include a different compiled/test scope and cannot replace the production DA denominator. Generated Bundle/Pin serde-family metrics and branch/MC/DC coverage are unavailable; absent records are not 100% or a zero-percent behavioral finding.

The original successful selected run and the Darwin continuous 19-test repeat both retain 0/9 added HTTP DA in their original exports. They receive no route-coverage credit. Final v3 separately merges eight original profiles with two normal-profile outputs from the maintained instrumented client and authenticated session shutdown, using four exact objects. Only those observed v3 counters support 8/9. No general causal explanation for earlier zero counters is asserted. Thirty-seven unique raw-profile/merged-profdata copies are preserved in `/private/tmp/forge-f19-profile-inputs-final/index.json` (`7080bce5…`); the original profdata is retained separately. Repeated runs and extra compiled objects add no unique tests.

## Documentation and authentic failures

The source audit is 50/50 selected named functions: 7 production Rust, 2 production Python, 21 cfg(test) and 20 integration-test declarations. All 16 new fields are documented: 8 production and 8 test-fixture fields. Whole-file documentation remains bundles 26/26, HTTP 2/19 (production 2/15; four cfg(test) functions), API tests 6/35, HTTP tests 19/39 and client 2/11. The legacy gaps and exact named inventories remain in JSON; this is not product-wide documentation completion. The author of focused guide/plan/inserts excludes those docs from its own independent-review credit; the HTTP-test author likewise excludes its tests.

Original failed records remain unchanged: first API/HTTP compilation stopped at the new test digest helper's unsupported LowerHex formatting; second API attempt ran 8 passes/1 failure on raw decoded-duplicate bytes in the schema-fixture corpus, then stopped before HTTP. That raw fixture is retained outside the duplicate-free JSON schema corpus, while actual transport regressions remain. First strict Clippy stopped on three Rustdoc warnings; second stopped on a long test, fixed through helper extraction without suppression. Third strict all-target Clippy passed on the pre-final-format test bytes; final rustfmt changed only the layout of one helper call and its check passed. The mandatory hook must still recheck delivered formatted test bytes. These failures establish no product-red baseline or extra successful denominator.


Root also replayed the completed independent evidence packet (`54c6270b…`, 14,329 bytes). It independently reconciles the raw metrics, exact source starts, documentation scopes and denominators; authored HTTP test correctness and authored focused docs remain excluded from that reviewer’s independent credit. The JSON root application record pins that packet and the plan-link delta.

## Remaining gates

The mandatory hook is pending and must run after this record is applied. Root will record its authentic outcome externally before PR, avoiding a pre-commit receipt that invents future success. F19 commit/push/PR and hosted CI have no result here. All six PRD 062 Should acceptance gates remain open where recorded, including full S-6 browser export/metadata preview/source-content opt-in, explicitly confirmed writable import, batch binding and publication/retention capacity. Security/privacy/authority, real browser/keyboard/AT/WCAG, supported-platform/interoperability, pilot/release/public API and human acceptance remain open. D068 authority/quorum and immutable-first workflow gates remain open. The user's final full integrated documentation review/update is still required.

The [bundle guide](workspace-index-bundles.md) explains the complete expected denominator, absent versus explicitly empty index, extra current registrations, whole-index equality, metadata sensitivity and separate observed domain state. No dependency, OSCAL schema asset, index role, signing/identity or domain approval decision is introduced; the additive index-bundle wire is explicit.

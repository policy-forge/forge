# Workspace operation browser verification

This record checks the S4 browser consumer prerequisite after integrating the
checkpoint producer `f9a74ce347718021b0af3385584b45c4671f9398` with the complete F05
browser lineage `04bf2aac1a948c5b1380566d90a5aef04761c175`. It records source and
execution evidence before the local integration commit. The final delivery must
verify both ancestors, committed source, the mandatory hook and the exact draft
PR head. [Machine-readable evidence](workspace-operation-browser-verification.json)
contains source, job, log, coverage and artifact hashes. Historical F05 receipts
remain unchanged; they do not count as current executions.

The production change beyond the carried F05 lineage is the caption
`Captured registrations: X of Y reported.` and its adjacent JSDoc. Identity,
safe-integer, state, cancellation, generation, recovery and terminal guards are
preserved. A complete captured set does not mean preparation succeeded or output
was saved. Null/absent progress stays indeterminate; zero/zero remains reported
facts; completed greater than total remains unreconciled. Three new source tests
exercise 1/101, 101/101 and generic 1001/1001 counters. The consumer does not turn
the producer's 1,000-registration or 100-consumed-input limits into schema limits.
API 1.2.0, OpenAPI and dependencies are unchanged. The carried F05 native
fixtures are unchanged by the caption update.

## Fresh execution

These jobs used the integrated source on macOS 27.0.1/aarch64 with Rust 1.98.1.
The source suite used Node 22.22.3. Native runs used Playwright 1.62.1, installed
Chrome 154.0.8037.97 and bundled Node 24.19.0. They establish this local development
scope only. Tests and observations in different rows overlap; their counts are
not added.

| Check | Actual outcome and scope |
| --- | --- |
| `cargo test --locked -- --nocapture` | 2,755 passed, 0 failed, 3 ignored; 69 result summaries. |
| `cargo clippy --locked --all-targets -- -D warnings` | Exit 0. |
| Instrumented Rust scope | 1,901 passed, 0 failed, 0 ignored; library, API-contract and workspace-CLI suites. Authentic LLVM JSON/LCOV, 4 normal plus 4 LLVM objects, 12 raw profiles and 1 merged profile are preserved. |
| Whole-source Node/V8 suite | 89 passed, 0 failed, 0 skipped/cancelled/todo. The actual complete embedded JS runs in the source-DOM transport/timer harness. |
| Maintained headless client | Existing complete workflow passed, including metadata bundle preview/verification, idempotent commit, read-only behavior and shutdown. This is a workflow result, not another Rust test count. |
| Existing writable native harness | Passed; 60 focus observations and 640/320 CSS-pixel reflow. No transient capture-counter response or rendering was observed. |
| Existing read-only native harness | Passed; 5 focus observations and 640/320 CSS-pixel reflow. No transient capture-counter response or rendering was observed. |
| Bounded native companion, final campaign | Three attempts observed real running capture and acknowledged running cancellation, then terminal `cancelled`. Each had 2 numeric API responses and 2 matching DOM observations, with pending-cancel focus observed. Separate genuine-failure and successful-unconfirmed preparation controls passed. |
| Preserved first companion campaign | Exit 2/incomplete: three operations completed too quickly to observe capture/cancellation. Zero counter responses or matching DOM observations; no credit for these missing observations. |

The final companion used 101 ordered registrations: one policy and 100 synthetic
invalid trace envelopes, totaling 51,068,714 bytes within the existing capture
limits. Each envelope reached bounded strict JSON parsing before rejection of
invalid Report fields. No producer pause, supplied operation state/counters or polling change
created the observations. Five conversion preparations, 20 operation GETs and
three cancellation requests occurred; no commit request occurred. Six matched
API/DOM observations belong to the three cancellation attempts. Across all five
operations there were 12 numeric API responses with matching DOM observations,
28 API events and 24 DOM records. These are repeated observations, not extra
tests or a promise to capture every intermediate update.

All final source/tool pins stayed unchanged during execution; fixture hashes
matched before/after and unconfirmed target files remained absent. The preserved
CLI served JS `fc9b518942b230bf22770ea55a4c1de71301ac222cfcf3aa9bf656e7698fb89e`
and CSS `d4ce961b5043cd2fc756b032583bb99b40d5f42563a3526ab13ebfbbae22fa02`.
Recording-time binary/object/profile hashes are evidence bindings, not process
startup attestations. Root preserved the incomplete first campaign and its
unchanged source/tool pins; it launched before the worker's explicit final
handoff. The separately frozen final campaign does not repair that history.

Fresh full and instrumented Rust logs each contain 57 relevant HTTP response
records: 32 status 200, 19 status 202, one status 401, two status 404 and three
status 409. They include actual running capture/cancellation, all four preparation
kinds and two graceful restart exits. The executions overlap. Response digests
bind original bodies; retained parsed bodies redact known authentication fields
and omit request authentication, without claiming universal redaction.

## Docstring coverage before drafting

Adjacent documentation presence is counted separately from semantic quality.
Selected declarations and whole-file inventories have distinct denominators;
callbacks are explicitly separate. New selected named helpers are documented.

| Scope | Documented / inventoried |
| --- | --- |
| Selected Rust production functions | 5 / 5, plus the existing Error type body and 1 / 1 selected field. |
| Selected Rust test functions | 3 / 3. |
| Whole two-module Rust production functions | 14 / 31. |
| Whole two-module Rust `cfg(test)` functions | 7 / 15. The earlier inline `capability_count` helper is test-owned; the preserved first lexical inventory was superseded explicitly. |
| Selected Python runner helper | 1 / 1. Module-level PTY teardown is outside the function denominator. |
| Selected production JS named declarations/bindings | 40 / 40. |
| Whole production JS named declarations/bindings | 40 / 60; 20 unchanged legacy omissions. 102 unbound callbacks are separate. |
| Selected CommonJS test helpers | 86 / 86: 18 native and 68 source-harness helpers. |
| Whole CommonJS named helpers | 86 / 87; unchanged native `register` is the remaining omission. 66 property callbacks and 235 anonymous/unbound callbacks are separate. |

Nine source-harness helper JSDoc comments were added after the Rust/native jobs.
The used Rust, production assets and native drivers remained byte-identical.
Normalized executable ASTs match, preserving values/directives, and the final
Node/V8 suite was rerun successfully against the comment successor.

## Test coverage before drafting

Rust coverage uses maximum-per-source-line unions of authentic LCOV `DA` records
and exact current LLVM function mappings. The scope is the two Rust modules
carried from F05, relative to the checkpoint producer. It includes their test
ownership explicitly, rather than truncating at the first `cfg(test)` attribute.

| Dimension | Covered / mapped; exclusions retained |
| --- | --- |
| Added/modified production lines relative to the producer | 17 / 17 mapped; 13 additional textual lines have no executable mapping. |
| Selected production body line union | 46 / 48; `Session::unlock` lines 220 and 235 remain zero. |
| Selected production named body entries | 5 / 5 positive; individual runtime families can still be zero. |
| Two-module production line union | 385 / 443; all 58 zero lines retained. |
| Whole authored production body entries | 27 / 31 positive; `Bounded::flush`, `internal`, `Session::revoke` and `prompt` remain zero. |
| Whole-file line union including tests | 585 / 643, reported separately from production. |
| Branch and MC/DC coverage | Unavailable; empty LLVM dimensions are not passing percentages. |

V8 coverage binds the complete production asset's exact UTF-16 offsets and the
fresh final Node execution. Maximum counts are unioned only for exact function
or range identities. Of 124 unique recorded functions, 99 are positive and 25
remain zero; of 528 ranges, 424 are positive and 104 remain zero. All 40 selected
named functions have positive exact primary mappings. Across 60 syntactic named
functions, 51 are positive, 7 map to zero and 2 are absent
(`mappingCreationForm::options`, `initializeForm::reload`). Anonymous/module
records remain separate. These block ranges are not branch or MC/DC coverage,
nor native, accessibility or human acceptance evidence.

The compact JSON successor retains zero/unmapped/absent dimensions and exact
source/observation records while pinning the complete original evidence archive
and raw exports. Historical receipts are not rewritten or refreshed. The mandatory
formatting, strict Clippy and full-suite commit hook is required separately;
it had not run at this record's pre-commit recording time. Hosted CI is also not
credited here: its main-only triggers require exact PR head/base/run readback.

## Remaining acceptance gates

This is a scoped S4 engineering prerequisite. Full S4 and all six PRD062 Should
Have requirements remain open, including the remaining S6 browser metadata
download/raw-file verification and confirmed writable import/batch work.
Platform/MSRV/offline/signed-launcher qualification, security/privacy review,
assistive-technology/WCAG checks and owner/human evaluations remain separate.
Every remaining Must/Should in PRDs055–069 stays in scope. The full documentation
review/update must reconcile the immutable integrated 2.0.0 release candidate
after the remaining work. This record does not complete that final review or
authorize merging, release publication or participant contact.

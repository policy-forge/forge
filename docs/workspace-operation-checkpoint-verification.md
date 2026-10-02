# Workspace operation checkpoint verification

This F19 slice supplies cooperative producer checkpoints for PRD 062 S-4. It is
stacked on the bundle-query candidate `d8340f43e05f59db20d4388e945f97bc61b6f1c0`.
Veans parent #23 and child #25 are IDs 1418 and 1420. This local record precedes
the mandatory commit hook, push and draft PR; those outcomes must be recorded
separately. Full S-4, the six PRD 062 Should Have acceptance gates and the final
integrated documentation review remain open.

The [JSON record](workspace-operation-checkpoint-verification.json) retains exact
source, command, log, coverage, review and binary/profile pins. Source snapshots
contain 19 files and ten baseline copies. Eight normal/instrumented executables
and thirteen unique raw/merged profile inputs are preserved separately. Later
binary hashes identify retained bytes; they are not startup/shutdown attestations.
The scope is Rust 1.98.1 on macOS ARM64 with default features. Other platforms,
MSRV and release qualification are separate gates.

## Executed checks

| Check | Passed / failed / ignored | Scope |
|---|---:|---|
| Full `cargo test --locked -- --nocapture` | 2,753 / 0 / 3 | 69 raw summaries |
| Selected instrumented run | 1,899 / 0 / 0 | 1,868 library, 9 API-contract and 22 HTTP tests |
| Strict all-target Clippy | Exit 0 | Final executable source, `-D warnings` |
| Maintained client conformance | Exit 0 | One existing synthetic API workflow |

The baseline has 2,721 passing tests. Twenty-nine new component/runtime tests and
three new HTTP declarations account for the 32 additional passes. Focused, full,
instrumented, repeated and maintained-client workflows overlap and cannot be
added together. Selected interruption tests cover eleven contexts over eight
stage names; they do not establish a full ten-stage/three-reason matrix.

The normal and instrumented logs each observed an actual running conversion at
6/100 captured registrations. In the normal run, cancellation returned 200 with
`running`, `cancel_requested: true` and 13/100, then GET observed `cancelled` with
null progress. The tests retain same-key acceptance replay, terminal cancellation
409, unchanged project bytes and an unrelated preview. Timing-dependent running
observations are reported from actual logs rather than inferred from assertions.

Another test gracefully stops and restarts the same project, rejects an old
capability (401), old operation/preview IDs (404) and an old receipt (409), then
binds a fresh conversion to changed source bytes and explicitly confirms a fresh
commit. All four preparation kinds also succeed with retained previews, original
acceptance replay and no implicit project writes. The three new tests close four
processes gracefully. The full log contains 57 selected structured response
records; these are not every request in the suite. Token/capability values are
redacted and request Authorization is omitted, while synthetic labels and
`diff_text` can remain. Original-response digests plus redacted parsed bodies
cannot reconstruct authority-bearing wire bytes or establish broad privacy proof.

## Docstrings and coverage

All 104 selected named function bodies have adjacent Rustdoc: 33 production,
49 cfg(test) and 22 integration-test bodies. Both separate trait method
declarations are documented. New named fields are 19/19, variants 20/20 and type
declarations 10/10. The two unnamed WorkError payload slots have documented
enclosing-variant semantics; their inline slot documentation is 0/2. Whole
production documentation remains 50/115 bodies, with 65 unchanged legacy omissions.
These counts establish the selected audit, not product-wide documentation completion.

Unique LCOV DA uses the maximum observed count per source line across records.
Production ends before each file's first cfg(test) module. Every mapped zero,
including closing expressions, is retained; no zero is pruned to raise coverage.

| Production module | Covered / mapped lines |
|---|---:|
| preparation | 11 / 14 |
| services | 449 / 595 |
| domain | 411 / 492 |
| HTTP | 425 / 597 |
| effects | 365 / 412 |
| actions | 256 / 306 |
| Six changed modules | 1,917 / 2,416 |

The separate seven-module context includes mod.rs at 9/9, yielding 1,926/2,425.
Textual production additions relative to d834 have 339/386 positive/mapped lines
under the exact Git diff; the production-prefix SequenceMatcher cohort has
338/385. The difference is one covered effects.rs closing brace, while the
prefix alignment includes two unmapped braces. Both methods and unmapped changes
are retained separately. All 47 mapped zero additions remain explicit.

The 33 selected production bodies have current first-region mappings in both
compiled crate families. Thirty-one have positive summed function-entry counts;
`NoopControl::interruption` and `domain::inventory` remain wholly zero. The
independent replay also matches all 115 whole-production body starts to current
mapping records. Native LLVM whole-file line/function/instantiation summaries
include a different compiled/test scope and remain separate from unique production
DA. Branch, MC/DC and generated-family evidence is unavailable where records are
absent. Function entry, lines and selected tests do not prove every error branch.

## Review and retained failures

Complementary bounded reviews cover core preparation, root adapters, Store/action
tests and final source/doc/coverage binding. Each reviewer excludes correctness
of its own authored code or tests. The docstring derivation reproduces byte for
byte; the raw coverage derivation also reproduces byte for byte. The exact Git
textual alignment remains a separate frozen comparison.
Reviews authored before full execution retain that temporal limitation. They do
not establish independent human acceptance.

The first core build stopped before tests on integration errors. Its successor
ran 90 passes and one failure: a missing-file test expected resource-containment,
while the preserved ordinary error is not-found. After correction, all 91 core
tests passed. First strict Clippy found a private enum-name issue, a long capture
function, Rustdoc formatting, a semicolon and unchecked subtraction; the next
found a missed legacy stage-name reference and two clone assignments. The final
strict run passes. A temporary fix constructor failed its formatted-source anchor
before any managed write; its corrected successor applied. Original receipts
remain separate and receive no successful-test credit.

Recursive retained-reference replay also found three stale references in an
earlier design packet: internal-contract.rs.txt, proposal.json and proposal.md.
Their expected and observed hashes/sizes remain explicit in JSON. Current source,
execution and final coverage evidence is independently bound; those historical
canonical references receive no validation credit and are not silently repaired.
All three expected originals are available byte for byte under the preserved
provisional-before-final-budget-choice archive; a separate readback verifies their
expected hashes and sizes without rewriting historical records.

## Remaining work

The [operation guide](workspace-operations.md) explains complete-registration
capture counters, original-key replay, sticky cooperative interruption, one
acceptance-time 30-second budget and fresh-session recovery. In-flight syscalls,
parsers and domain engines can exceed the budget until their next boundary; this
is not a hard wall-time or preemption guarantee. Confirmed single-file commit
publication retains its existing path and cannot be relabeled cancelled afterward.

The API shape/version stays 1.2.0. No new crate, index role, schema family, wire
field, persisted authority or public-library API is introduced. F05 browser
consumer integration, supported-platform/security/privacy/interoperability,
native keyboard/assistive-technology and human acceptance remain open. The
main-only hosted CI trigger does not schedule for the intended stacked base;
local checks do not establish hosted platform validation. Full S-6 bundle
export/import, explicit confirmation and batch/capacity work remain in scope.
The final full documentation review/update and verified 2.0.0 candidate remain
end-of-roadmap gates.

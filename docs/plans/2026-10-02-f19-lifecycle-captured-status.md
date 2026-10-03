# F19 lifecycle captured-status prerequisite

The root integration basis is immutable `e52ae8cc5c52ff6734fb47bc5dc17436b5bd9a55`, the
documentation delivery in draft PR #190. The original TEMP preparation used
`3d9345bda6f8c7eb99e69186b6897a874eff4579` with four retained inputs equal to its earlier
`0e2b803` preflight basis. The frozen v3 preparation records the explicit rebase to `e52` and
discloses intervening documentation/context changes. Those preparation packets and the original
[static audit](2026-10-02-f19-lifecycle-captured-status-static-audit-v1.json) remain historical;
they are not relabeled as compiler, test, coverage, release, or hosted results.

The root engineering owner selected this internal interface:

```rust
pub(crate) fn status_from_captured(
    record: &LifecycleRecord,
    current: &CurrentArtifacts,
    as_of: Option<NaiveDate>,
) -> Result<StatusReport, ForgeError>
```

The existing `status_report` wrapper captures required artifacts and calls the projector.
Existing check, status and queue entrypoints, confined reads, transition behavior, publication
functions and serialized report shapes are retained. The directly callable projector validates
intrinsic record structure and exact captured hash/path correspondence before projection. It
performs no I/O, clock sampling, process execution or network access. This is an engineering
integration choice; it grants no human authority or acceptance.

The source change is confined to `src/lifecycle/mod.rs`, new `src/lifecycle/status.rs`, and two
appended tests in `tests/lifecycle_cli_test.rs`. Documentation comprises this plan, its guide,
the historical static audit and a new root validation audit. Record/schema parsing, CLI options,
dependencies, workspace/API/UI code, prior tests and historical plans remain unchanged. No new
crate is proposed.

Eight new unit tests address full report shape and immutable inputs; approval drift priority and
identity/history evidence; explicit date boundaries; date overflow; ten malformed captured
inventories; two invalid intrinsic records; legacy source-only/raw-path spelling; and latest
approval selection. Two CLI tests address all three complete report shapes, repeated bytes and
source preservation, plus all three missing-artifact failures with absent/preexisting report
destinations. Cases within a test are not separate test declarations.

Root formatting and actual compilation found four new test patterns using struct syntax for
the tuple variant `Lifecycle(String)`. Only those test patterns were corrected. The first exit
101 and its four E0769 errors are retained; it executed no tests. The corrected focused eight
unit tests and all 26 CLI regressions passed. Fresh instrumented runs passed 19 lifecycle
library tests and 26 CLI tests. Two strict private Rustdoc link checks passed. A versioned
comparison harness preserved the earlier harness's erroneous zero-exit assumption and then
verified 24 full report/stdout/stderr/expected-exit comparisons with the preserved prior build.

The [root validation audit](2026-10-02-f19-lifecycle-captured-status-checks-v2.json) binds exact
source, authentic raw receipts, named docstring inventory, LLVM counts and zero/unmapped records,
and the independent bounded source review. It is a pre-commit checkpoint; the enabled repository
hook, exact commit and draft PR are recorded separately. Static source equality and development
fixtures establish neither branch coverage nor human approval.

Full PRD 062 S-3 includes lifecycle and impact registration, capture services, freshness and
read-only views. Those integrations, their platform/browser and human gates, and the final
integrated documentation review remain open. This prerequisite receives no full S-3 acceptance
credit. See the [scoped guide](../lifecycle-captured-status.md).

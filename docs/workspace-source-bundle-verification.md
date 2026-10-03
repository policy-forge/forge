# Source bundle development verification

This checkpoint covers the consumed API **2.3.0 / 57-operation** inline source
workflow and native Unix restore outcomes. API1 stays **1.2.0 / 39 operations**.
The product release target remains **2.0.0**. The [source workflow guide](workspace-source-bundles.md)
describes the user-visible acknowledgments, complete preview, exact confirmation
and known-ID recovery. The [machine-readable receipt](plans/2026-10-03-f19-source-bundle-verification.json)
retains exact source/evidence hashes, commands, counts, zeros and remaining gates.

The checkpoint precedes the source commit and hosted verification. Passing local
checks supplies development evidence; the full PRD062 S6 and release gates remain
open. Historical metadata-only and prior failure receipts retain their original
source and API versions.

| Check | Observed result | Scope |
| --- | --- | --- |
| Instrumented all-feature Rust suite |2931 passed,0 failed,3 ignored |68 suite summaries; ignored cases receive no credit |
| Final strict Clippy | Passed | All targets/all features, locked inputs, warnings denied |
| Rust docstrings |313/313 named functions;39/39 named types | Selected lexical cohort across 15 exact-source files |
| Rust selected entries |306/313 positive;7 zero;0 unmapped | Entry observations, not complete function bodies |
| LLVM reported lines |69437/75953,91.4210% | Reported aggregate across production, test and generated records |
| LLVM reported functions |6283/7360,85.3668% | Native function records, separate from the selected source cohort |
| JavaScript documentation/entries |55/55 documented and positive | Selected named production UI functions under V8 and Node FakeDOM |
| Node navigation controls |248 passed,0 failed/skipped/cancelled/todo | Mocked DOM controls, separate from Chrome |
| Maintained Python mock controls |117 passed,0 failures/errors/skips |79 existing plus35 source and3 metadata-version controls |
| Python docstrings |106/106 selected;200/200 across 9 files | AST named declarations, including helpers/tests and the unchanged version companion |
| Python selected entries |104 positive,2 observed zero,0 unmeasured | Exact first-body-line trace observations; broader unchanged declarations remain separate |
| Real maintained source workflow |13 checks passed;4 normal server exits | Actual authenticated macOS process, exact binary/source/client hashes |
| Actual installed Chrome source flows | Export, restore and fresh read-only lookup passed | One restore confirmation; exact bytes/index and retained removed files; desktop/320px captures |
| Real maintained metadata workflow |16 compatibility checks passed on 2.3.0 | Exact input boundaries, no-write previews, confirmed index-only replacement and read-only preservation |
| Actual metadata Chrome companions | Writable and read-only passed on 2.3.0 | Existing metadata controls/version observed; normal native exits |

The seven zero-entry Rust declarations remain explicit: `http::read_query_response`,
`http_source::SourceControl::interruption`, `source_validation::Sources::native_mapping`,
`transaction_state::{capacity,unavailable}`, and the test controls
`root_transaction::tests::{StopControl,AfterPublish}::interruption`. The two selected
Python zeros are the launchers' bounded `wait_owned` cleanup fallbacks. Branch and
MC/DC coverage are unavailable because the export contains no instrumented
denominator. None of these observations supplies whole-repository documentation,
production-only line coverage or exhaustive body/branch execution.

The broader Python census records149 positive entries,4 observed zeros and47
unmeasured declarations. Those47 belong to the unchanged version companion.
Two initial trace commands discovered zero tests; their exit-zero logs are
preserved as incomplete. Corrected runners asserted discovery and actually ran
all3 metadata-version and7 lifecycle controls before supplying trace evidence.

## Reproduce the native consumers

Use the exact checkout being reviewed, a compiled native Unix Forge binary, and
the approved Node/Playwright graph with the installed Chrome executable. Each
output directory must be new. The source launcher accepts an optional `--rtk`
wrapper; its default Node child is direct. Pass the actual compiled Forge binary
as `--forge`. No command installs packages or downloads a browser.

```sh
rtk proxy cargo build --locked
rtk proxy python3 -B scripts/test_workspace_source_bundle_client.py
rtk proxy python3 -B scripts/test_workspace_bundle_workflow_controls.py
rtk proxy python3 -B scripts/test_workspace_source_workflow.py \
  --forge target/debug/forge --out /tmp/forge-source-native-new
rtk proxy node --test ui/tests/workspace-navigation.cjs
rtk proxy python3 -B scripts/test_workspace_source_browser.py \
  --forge target/debug/forge --node /absolute/path/to/approved-node \
  --chrome /absolute/path/to/installed-chrome --source-root . \
  --driver ui/tests/workspace-source-browser.cjs --out /tmp/forge-source-browser-new
```

The native workflow exercises original 202 replay, no-write preparation, exact
committed download/headers/private artifact family, complete resource/index and
directory preview, preaccepted-ID absence, durable pending 202/replay, exact binary
and Markdown restoration, retained removed files, terminal cancellation truth,
and fresh read-only/writable lookup without reviving old receipt authority. It
observes four authenticated shutdown responses and normal zero exits before
ordinary client cleanup.

The Chrome launcher uses real synthetic files, real UI interactions and real
HTTP; it injects neither responses nor application state. All three native
servers and Node children exit normally. Browser/context closure is recorded.
The status paragraph is scrolled into the viewport after each width change so
its outcome appears in the actual pixels. The source restore publishes its
three targets in order, including the index last, verifies their exact bytes,
and preserves the retired source file. Read-only lookup makes no project write.
Page routing does not establish OS-wide network denial; direct-child reap does
not measure an empty descendant tree. Screenshots and focus checks remain
separate from assistive-technology and human acceptance.

The workflow adds separate Unix native source and installed-Chrome source CI
steps. It preserves the original four-campaign F04 verifier's denominator and
the approved development dependency scope. The source Windows transaction
continues to return typed unavailable pending native implementation/qualification.

## Remaining gates

Larger staged transfers, complete shared-capacity qualification, native Windows
transactions, cross-platform crash/rollback/recovery, independent security/privacy,
assistive-technology/WCAG and participant acceptance remain open. Exact committed
and pushed source/hosted results, dependency audit and Linux OS-denial
qualification are separate gates. The final integrated documentation review
across all completed roadmap packages and verified 2.0.0 release provenance remain
open. Merge and publication require separate user authorization.

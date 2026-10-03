# Source bundle development verification

The inline source workflow uses API **2.3.0 / 57 operations**. API1 remains
**1.2.0 / 39 operations**, and the product release target remains **2.0.0**.
The [source workflow guide](workspace-source-bundles.md) describes the complete
preview, explicit confirmation and known-ID recovery. The
[startup repair receipt](plans/2026-10-03-f19-source-startup-verification.json)
binds the repaired source, commands, documentation census, coverage observations
and preserved failures. These are development checks; full PRD062 S6 acceptance
and the release gates remain open.

The repair handles concurrent creation of a private-state parent: an existing
component is reopened with the same no-follow, owner, permission and identity
checks. CLI test startup failures now retain bounded redacted diagnostics and
reap the owned child. Windows restore remains typed unavailable; platform-specific
code and tests are conditioned accordingly, without granting Windows restore
authority.

| Startup-repair checkpoint check | Observed result | Scope |
| --- | --- | --- |
| Rust 1.99 strict Clippy | Passed | Locked, all targets and all features on macOS |
| Fresh instrumented Rust suite | 2,933 passed, 0 failed, 3 ignored | 68 summaries; ignored cases receive no credit |
| Rust docstrings | 319/319 functions; 39/39 types | Selected lexical cohort in 15 exact-source files |
| Selected Rust entry observations | 311/319 positive; 7 zero; 1 unmapped | Function entries, without whole-body coverage credit |
| LLVM reported lines | 69,500/76,011 (91.4341%) | Production, test and generated records together |
| LLVM reported functions | 6,291/7,366 (85.4059%) | Native records, separate from the lexical cohort |
| Native private-state controls | 4 passed | Actual concurrent creation and trust-boundary checks |
| Final-binary cold starts | 64/64 passed | 64 authenticated shutdowns and normal zero exits; no forced cleanup |
| Maintained native source workflow | 13 checks passed; 4 normal exits | Rust 1.99 instrumented binary and authenticated macOS processes |
| Maintained Python docs/entries | 106/106 selected docs; 104 positive, 2 zero | AST declarations and first-body-line trace observations |

The seven selected Rust zero-entry declarations are
`http::read_query_response`, `http_source::SourceControl::interruption`,
`source_validation::Sources::native_mapping`, `transaction_state::capacity`,
the test controls `root_transaction::tests::{StopControl,AfterPublish}::interruption`,
and `Server::startup_failure`. The unsupported-platform native planning test is
unmapped in the macOS LLVM export. The broader Rust census retains 124 legacy
documentation gaps among 664 declarations. Branch and MC/DC coverage are
unavailable because their exported denominators are zero. No whole-repository,
production-only line or exhaustive body coverage is inferred.

The two selected Python zeros remain the launchers' bounded `wait_owned` fallback
helpers. The broader census retains 149 positive entries, 4 zero entries and
47 unmeasured declarations; the unchanged version companion supplies those
47 unmeasured entries. The corrected native trace binds the changed workflow
bytes; earlier traces remain applicable only to their exact unchanged sources.

The native workflow observed a successful committed operation while cleanup was
still pending, then verified settlement. Its checks now poll that separate phase
under the original 65-second budget. The first traced run failed its terminal
cleanup assertion after seven checks and is retained without 13-check credit;
it did not record the failing outcome, so its exact cause remains unconfirmed.
Two later diagnostic runs and the corrected maintained workflow retain their
own results. A zero-test command caused by an incorrect module filter also
remains incomplete; the corrected command actually executed all four controls.

## Platform lint follow-up

The [platform follow-up receipt](plans/2026-10-03-f19-source-platform-verification.json)
binds fresh checks after two Clippy corrections. At the preceding commit
`bc1acfd76da00a9f24839de310f1a03eedae253e`, hosted Linux and Windows tests
passed before strict Clippy rejected one platform-specific expression each.
Linux now uses an equivalent `if let` branch; the unsupported Windows path
explicitly releases its owned record. Windows restore remains unavailable.

Fresh macOS Rust 1.99 checks passed formatting, strict Clippy, 2,933 instrumented
tests, 13 native workflow checks and 64 cold starts. The exact-source census
retains all 319 documented selected functions. Entry observations remain
311 positive, 7 zero and 1 unmapped; aggregate lines remain 69,500/76,011
and native functions 6,291/7,366. The native workflow and all cold launches use
the newly recorded instrumented binary and exit normally without forced cleanup.
The table above retains the startup-repair checkpoint; its receipt and the
original source receipt are preserved. New-commit hosted outcomes remain a
separate gate, including Windows compilation, audit and Linux OS denial.

## Preserved source checkpoint

The [original source receipt](plans/2026-10-03-f19-source-bundle-verification.json)
is preserved byte-for-byte. It records the pre-repair Rust 1.98 evidence:
2,931 instrumented passes, 313 documented selected functions and 306 positive
entries. Its 248 Node FakeDOM controls, 55 selected UI functions, 117 Python mock
controls, 16 native metadata checks, and installed-Chrome source/metadata flows
remain bound to their recorded source and binary bytes. They are not relabelled
as runs of the repaired Rust binary. Hosted results must name the exact new
commit; successful checks from an earlier head do not establish that gate.

## Reproduce the native consumers

Use the checkout being reviewed, a native Unix Forge binary, and the approved
Node/Playwright graph with the installed Chrome executable. Every output directory
must be new. No command installs packages or downloads a browser.

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

The native workflow checks original 202 replay, no-write preparation, exact
committed download and headers, private artifact-family isolation, complete
resource/index/directory preview, preaccepted-ID absence, durable acceptance,
exact binary and Markdown restoration, preserved retired files, cancellation
truth, and fresh read-only/writable lookup without reviving old receipts.

The Chrome consumer uses actual files, UI interactions and HTTP. Page routing
does not establish OS-wide network denial; direct-child reap does not prove an
empty descendant tree. Screenshots and focus checks are separate from
assistive-technology and human acceptance.

## Remaining gates

Larger staged transfers, complete shared-capacity qualification, native Windows
transactions, cross-platform crash/rollback/recovery, independent security/privacy,
assistive-technology/WCAG and participant acceptance remain open. Exact committed
and pushed source, hosted checks, dependency audit and Linux OS-denial evidence
are separate gates. The full integrated documentation review across all completed
goal work and verified 2.0.0 release provenance also remain open. Merge and
publication require separate user authorization.

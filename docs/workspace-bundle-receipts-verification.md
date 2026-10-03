# Metadata bundle receipt development verification

API2/2.2.0 adds three operations to the 48-operation lifecycle/impact contract: acknowledged metadata export preparation, committed metadata download and direct complete index-replacement preparation. The selected namespace has 51 operations; default API1/1.2.0 retains 39. Product release target 2.0.0 is separate. The source basis is `0342c42723719f95c9ae1c8b6d51e5a19dc2f7ca`.

The [workflow guide](workspace-bundle-receipts.md) explains sensitive metadata, explicit confirmation, complete membership, existing-file admission and limits. The [versioned verification record](plans/2026-10-03-f19-metadata-bundle-receipts-verification.json) preserves exact source pins, commands, logs, census records, raw-export hashes, zeros and original failures. This checkpoint precedes the enabled commit hook, immutable commit and hosted readback.

## Executed checks

| Check | Outcome | Scope |
|---|---:|---|
| Final instrumented Rust | 2,065 passed; zero failures or ignored tests | 1,956 library, 22 contract, 21 framework-impact CLI, 26 lifecycle CLI and 40 workspace HTTP |
| Maintained Python controls | 130 passed | 25 version, 34 extension, seven lifecycle, 51 receipt and 13 metadata-client controls |
| Actual maintained client | 16 grouped workflow/boundary checks passed | Real local HTTP and exact compiled source; normal shutdown |
| Browser source controls | 207 passed | Actual asset bytes in the Node VM/fake DOM harness |
| Actual installed Chrome | Writable and read-only passed | Real browser-mode server/assets, no substituted responses; normal shutdown |
| Strict Clippy | Passed | All targets and all features, warnings denied; adjacent type-doc successor preserves runtime tokens |

Native checks exercise strict duplicate-member and BOM refusal, the exact 1 MiB raw-body boundary, local file-plus-wrapper refusal, declared and streamed excess bodies, explicit numeric index selection and downgrade rejection. Export preparation writes no file; only confirmation publishes the bound metadata bytes. Index replacement preserves source-file bytes and reports complete previous/proposed registration membership independently of the truncated text diff. Real concurrent same-key preparation retains one original receipt.

## Documentation and coverage

The selected 11-file Rust cohort has **105/105 documented functions**, **11/11 documented changed types**, **104/105 positive direct function entries** and zero unmapped names. Its function scopes are 50 production, 45 cfg-test and ten integration functions. The retained zero is `crate::workspace::bundle_effects::payload_limit`. Entry observation does not establish full-body or branch coverage. The whole lexical inventory retains 483 named functions; the selected cohort is not a repository-wide census.

The final LLVM export reports **56,470/70,136 lines (80.51%)**, **5,321/6,796 functions (78.30%)** and **97,859/120,294 regions (81.35%)**. It retains all 12,394 raw records and 5,666 zero records. These are LLVM-reported aggregates for the selected suites and gracefully terminated native sessions. Branch and MC/DC instrumentation were not requested. Historical S3 and earlier S6 profiles/exports remain preserved separately.

The seven-file Python AST cohort has **244/244 documented functions** and **241/244 positive first-body lines** in the mock/native trace union. The remaining zeros are the client worker-thread `read_descriptor`, version-fixture worker `readline` and lifecycle test `kill`. This tracer installs no thread hooks. The maintained client alone is **27/27 documented and 26/27 positive**; the native companion is **10/10 documented and positive**. The additional maintained browser launcher has **4/4 documented functions and positive first-body lines** in a separately traced actual writable Chrome campaign; this is entry observation, not whole-body or branch coverage. Repeated runs preserve their original unique-control denominators.

The selected JavaScript cohort has **48/48 adjacent documentation comments** and **47/48 positive V8 entries**; `resumeInspectionView` remains zero and no selected name is unmapped. The whole asset retains **223/258 positive first entries**, 35 zero records and 18 unchanged bounded lexical documentation gaps. Anonymous/generated records remain available. This is not a complete JavaScript AST or line/branch denominator. The native browser driver has **8/8 documented named helpers and positive Node V8 entries**, with zero unmapped names, in that separately instrumented real writable campaign. Its complete raw anonymous/zero driver records remain retained. Driver instrumentation does not measure the native browser UI asset; the207 unique source-control denominator remains unchanged.

## Browser observations and reproduction

Installed Chrome 154.0.8037.98, approved Playwright 1.62.1 and Node 24.19.0 exercised the actual embedded asset hash. Both modes observed zero page errors and zero non-loopback page requests. Native keyboard checks covered passphrase focus, view-heading focus, sensitivity acknowledgment, preview dismissal with Keep editing/Escape, return focus, confirmed-save focus and workspace shutdown. Writable controls checked exact committed download bytes and complete two-to-two registration replacement with source bytes preserved. Read-only controls retained metadata queries/downloads and disabled both effect preparations without changing the project.

Long labels and a 185-character selected filename reflowed at 640 and 320 pixels; measured page width equaled viewport width. Screenshots retain actual pixels from synthetic fixtures. These observations do not establish assistive-technology, complete WCAG, Windows/Linux browser parity, OS-wide network denial or human acceptance.

The retained launcher is `scripts/test_workspace_bundle_browser.py`; its driver is `ui/tests/workspace-bundle-browser.cjs`. It uses an owned temporary fixture, POSIX terminal passphrase setup and the approved existing runtime. It installs nothing. On a qualified macOS machine with installed Chrome and the approved tools, run each mode into a fresh output directory:

```sh
rtk cargo build --locked
rtk proxy python3 -B scripts/test_workspace_bundle_browser.py --forge target/debug/forge --node /absolute/path/to/approved/node --source-root /absolute/path/to/forge --mode writable --out /tmp/forge-s6-browser-writable
rtk proxy python3 -B scripts/test_workspace_bundle_browser.py --forge target/debug/forge --node /absolute/path/to/approved/node --source-root /absolute/path/to/forge --mode read-only --out /tmp/forge-s6-browser-read-only
rtk proxy python3 -B scripts/test_workspace_bundle_workflow.py --forge target/debug/forge
```

The existing hosted API jobs now run the metadata mock controls and native API2.2 companion on Linux, macOS and Windows, retaining a redacted boundary artifact. The approved Linux Chrome job also runs the207 Node source controls. This workflow configuration is not hosted execution evidence.

## Review and remaining gates

Independent bounded reviews found and resolved a UI consumed-file-count lower-bound check, stale guide claims about API1 queries versus API2.2 effects and overly broad receipt-consumption wording. Root corrected invalid new idempotency-key fixtures, a browser-harness field-name assertion, a stale exact-version fixture and strict Clippy findings. The original failed records remain unchanged; the disk-exhausted build and invalid launch-mode invocation receive no execution credit. Own normative/producer authorship is excluded from independent approval.

This is the metadata-only prerequisite to full PRD062 S6. Source-content opt-in, complete content/pin binding, confirmed multi-file import, rollback and crash recovery remain open, along with broader capacity/platform qualification. Dependency audit and Linux IP-denial parent jobs remain failed. Current API2.2 hosted parity, authentic interoperability/evaluation, native accessibility/security/privacy/human acceptance, release provenance and the **full final integrated documentation review across all completed work** remain open. Passing checks and a focused docs update do not close those gates.

## Hosted Rust lint successor

The f493 checkpoint passed the hosted Workspace API jobs on Linux, macOS and Windows, the installed Chrome Linux prerequisite and Windows controlling-terminal prerequisite. All three Rust CI lint steps failed at Rust 1.99 Clippy `assert_is_empty` in one test assertion. The workflow runs its test steps before lint; the retained failed-step log alone supplies no Rust test totals. The successor replaces that assertion with equality against an explicitly typed empty array; production code and dependency graphs are unchanged.

The changed test has an adjacent docstring and a positive exact-source LLVM function entry, with one selected test passing. [The successor record](plans/2026-10-03-f19-metadata-hosted-lint-successor.json) preserves the exact source/report/log pins and the initial coverage-command syntax failure with zero credit. The complete f493 cohorts above remain historical evidence; this focused result does not relabel them as successor execution. The enabled full commit hook and subsequent hosted checks are separate records. Supply-chain audit approval and Linux IP-denial qualification remain open; this lint correction supplies neither.

# Staged source transfer verification

The API 2.4 staged source transfer implementation passed the local checks below against Rust 1.99.0. It adds eight operations to API 2 (65 total) and the explicit `forge.workspace-index-bundle/4` profile. The default inline workflow, its request limit, and API 1/1.2 remain available. The source parent is `2064e2933844a827e13b9880c8cfefd1cb2184ee`; these results describe its staged successor and do not establish a merge or release.

The [machine-readable verification report](plans/2026-10-03-f19-staged-source-verification.json) records exact source, binary, receipt, log, census and coverage identities. Historical source-bundle reports and failed attempts remain separate. A test count, a documented function and a native check are distinct quantities.

## Local results

| Check | Actual result | Scope |
|---|---|---|
| All-feature Rust LLVM suite | 2,992 passed, 0 failed, 3 ignored; 68 summaries | Final Rust sources, including the documentation cleanup |
| Rust formatting and strict Clippy | Passed | All-target/all-feature strict lint run; the enabled commit hook also checks its own default-feature tree |
| Python mock client suite | 161 passed, including 47 staged controls | Maintained client and mock protocol observations |
| UI control suite | 266 passed, 0 skipped | Actual shipping JavaScript loaded into the bounded control harness |
| Native staged workflow | 29 checks passed, four normal workspace exits | Actual authenticated Unix HTTP workflow, complete preview and durable result |
| Native inline and metadata regressions | 13 and 16 checks passed respectively | Separate legacy workflow campaigns; no combined acceptance denominator |
| Installed Chrome staged workflow | Export, restore and fresh read-only lookup passed | Chrome 154.0.8037.98, approved Node 24.19.0 and Playwright 1.62.1; three normal workspace and Node exits |

The real staged artifact is 1,229,739 bytes, SHA-256 `6fc530decbde02c7ea00a1df2d383d4877bbf1ed49af828ccc9d7701af0013c2`, transferred in 38 ordered parts. The native verifier checked original create/part/preview replies, an explicitly retried lost preview reply, complete input/target/directory membership, index-last publication, retained removed-registration files and fresh-session authority boundaries. It observed committed bytes and verified cleanup; it did not invent a pending-cleanup observation where none occurred.

Chrome used the genuine served assets and HTTP replies. It checked the exact downloaded bytes, a complete no-write restore preview, one explicit restore confirmation, the committed result, and a fresh read-only lookup with zero project writes. Eight actual synthetic-fixture screenshots cover 1440 px and 320 px viewports. Focus observations and viewport bounds are recorded; they do not substitute for assistive-technology or participant acceptance. Direct child reaping is measured; the full descendant tree is unmeasured.

## Documentation and execution coverage

Against immutable parent `2064e293`, all 228 selected changed/new Rust named bodies have attached Rustdoc: 131 production lexical, 89 cfg-test and 8 integration bodies. The 65 selected direct test declarations are 59 new and 6 changed. Selected types 26/26, fields 93/93 and variants 17/17 are documented. The whole thirteen-file lexical inventory retains 80 legacy function documentation gaps (499/579 documented); these are not added to or hidden within the selected cohort.

LLVM records execution at 227 of 228 selected function headers, with no unmapped selected header. The fixed capacity-error helper `staged_source_bundles::oversized` remains a recorded zero. Production 130/131, cfg-test 89/89 and integration 8/8 are separate header cohorts, not proof of complete function bodies. The combined mixed production/test/generated export reports 73,160/79,761 lines (91.7240%) and 6,573/7,675 functions (85.6417%). Branch and MC/DC denominators are unavailable. Raw aliases, zeros, source conditions and exclusions are retained.

All nine changed/new shipping Python files have 216/216 function, 9/9 class and 9/9 module docstrings; 139 selected functions are documented. The initial staged client/control/native trace cohort has 122/122 selected function docstrings. Exact first executable body-line traces show 121/122 observed; the nested mock `Response.getheader` remains zero. Its whole three-file cohort has 169/169 docstrings and 166 observed bodies; `Workspace.__enter__` and `__exit__` are the two additional zeros. Trace observations use a maximum union across the separate mock and native campaigns, never a sum. The new Chrome launcher adds 11 separately documented functions; compatibility verifiers and the browser driver's 14 lexical JSDoc declarations have separate census records. These documentation extensions receive no inferred trace coverage.

All 32 selected production JavaScript functions have JSDoc and positive V8 observations. Whole-file production documentation is 169/187; 18 legacy gaps remain. The whole named production cohort has 175 positive, 10 mapped-zero and 2 unmapped functions. Raw script records are 307/345 positive primary functions and 1,630/1,927 positive ranges. The distinct lexical, AST and raw V8 cohorts are not interchangeable, and none establishes full-body or branch coverage.

## Reproduction and retained evidence

Use the existing locked dependency graph and approved development tools. No browser download is needed. Standard checks include:

```sh
rtk cargo fmt --check
rtk cargo clippy --locked --all-targets --all-features -- -D warnings
rtk cargo llvm-cov --workspace --all-features --locked --json --output-path coverage.json
rtk proxy python3 -B -m unittest discover -s scripts -p 'test_workspace_*client*.py'
rtk proxy node --test ui/tests/workspace-navigation.cjs
rtk proxy python3 -B scripts/test_workspace_staged_source_workflow.py --forge /absolute/path/to/forge --out /absolute/fresh/output
rtk proxy python3 -B scripts/test_workspace_staged_source_browser.py --forge /absolute/path/to/forge --node /absolute/path/to/node --chrome /absolute/path/to/installed/chrome --source-root /absolute/path/to/checkout --driver /absolute/path/to/checkout/ui/tests/workspace-staged-source-browser.cjs --out /absolute/fresh/browser-output
```

The report binds the actual commands, toolchain, compiled binary, full raw coverage exports and preserved profile index. Earlier disk-exhaustion runs and the browser driver's inherited inline-filename assertion failure remain retained without pass credit. The filename expectation was corrected to the staged filename; the production UI needed no corresponding change. Subsequent module-documentation cleanup changed nine comment lines and no executable lines; the final suite rebuilt a byte-identical instrumented binary and all native campaigns were rerun.

## Remaining gates

Hosted staged checks are pending on the new draft. Linux/macOS/Windows source-parent CI successes are a separate synthetic-merge result. Its supply-chain job reported 20 cargo-vet gaps, and its Linux OS-denial experiment was incomplete with `tool-untrusted`; neither is waived by these local checks.

Native Windows transaction, platform/crash/rollback, qualified OS confinement, audit/owner decisions, browser/assistive-technology/human evaluation, remaining F01–F22 interoperability and the final integrated goal-wide documentation review remain open. See the [staged workflow](workspace-staged-source-bundles.md), [integrated documentation scope checkpoint](plans/2026-10-03-integrated-documentation-staged-successor.md) and [authoring gate register](authoring-gates.md). No participant judgments, owner approval, merge, tag, release or publication are recorded here.

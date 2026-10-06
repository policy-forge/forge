# Workspace keyboard development verification

This successor covers F05 child #16 on baseline
`b4207ea8280304c655cda2ad018f28324403a0d0`. The
[machine-readable receipt](workspace-keyboard-verification.json) binds exact
source, harness, API, tool, preserved debug executable, raw coverage and diagnostics.
Historical navigation/live-state JSON receipts remain unchanged. These focused
checks do not close F05 or the final integrated documentation review.

## Documentation and source tests

All **16/16 selected JavaScript production functions** have adjacent JSDoc and
positive V8 function observations. All **8/8 changed Rust functions** have
rustdoc: **5 production functions/methods** and **3 embedded tests**. The final
JavaScript test-helper scope is **13/13 documented**: 11 changed/new semantic
declarations, a moved activation helper and a clarified focus helper. The earlier
frozen 12/12 inventory omitted the new named `emit` arrow. Its successor adds the
docstring and reruns the tests; the earlier packet remains unchanged.

A whole-asset AST audit finds **33/50 named function declarations** and **7/10
identifier function bindings** documented. Those separate scopes exclude 102
unnamed/default/call-site callbacks. They are not a whole-repository percentage.

Node **v22.22.3** executes the complete exact production file through `vm.Script`
using its actual file URI. **86/86 cases pass** with zero failures, cancellations,
skips or todo cases: all 58 previous cases plus 28 new cases. The source hash is
`9c1f8be532f8cd1eaa73926c34ebbb7a8ca3d346ea0caeb67d91faa103191d8e`.
V8 observes **99/124 function entries** and **424/528 raw ranges** with positive
counts. Every production observation and raw-file pin is retained. These are
**not line, statement, branch or condition coverage**. Fake DOM/queued-close
checks do not supply native layout, focus-trap or assistive-technology evidence.

Cases cover locked/pending/throttled unlock, duplicate activation and focus
ownership; exact confirmation keys, registration/export focus, rejection and
saved-but-refresh-failed outcomes; Escape/navigation/replacement/shutdown races;
retained-pager recovery and stale success/error/finally guards; and an initially
ineligible background preparation preserving a visible preview. Failed download
retains Saved and does not replay the write.

## Rust behavior and measured coverage

Rust **1.98.1** passes **47 scoped library workspace tests** and **11 real-process
HTTP tests**. LLVM **22.1.8**, via cargo-llvm-cov **0.8.7**, reruns the 47 library
tests. Server retry guidance uses the existing monotonic deadline, rounds positive
fractions up and reports zero after expiry. Existing backoff remains 2–64 seconds.
Borrowed static errors and bounded owned server guidance pass the same closed
Error schema; wire fields are unchanged.

LCOV includes all instrumented production lines, including unchanged and
uncovered code, while excluding embedded tests and the earlier `cfg(test)`
`Session::capability_count` helper:

| Module | Covered / instrumented production lines |
| --- | --- |
| session.rs | 162 / 207 |
| contract.rs | 216 / 236 |

All **17/17 added instrumented production lines** execute. Blank/uninstrumented
lines are excluded; branch/condition/platform coverage is not measured. The first
session calculation included three covered test-only helper lines (165/210);
its corrected successor excludes them and retains the original calculation.
The mandatory commit hook enforces formatting, strict default-feature all-target
Clippy and the full Rust suite. Its log and PR validation report that separate
outcome; these focused counters are not full-suite counters.

## Native browser observations

Both modes use the separately preserved macOS arm64 debug executable, SHA-256
`5ed6bf992b47d5e75d209252ef2479766cb371cdccac786b48689d48ce4b5967`,
68,044,280 bytes, copied before subsequent Cargo hooks. Both verify the exact
served candidate JS/CSS. It is a development executable, not a release package.
The earlier live-state executable remains separately preserved.

Chrome **154.0.8037.97**, Playwright **1.62.1**, driver Node **v24.19.0** and Python
**3.14.8** pass writable and read-only fixtures. The source-test Node runtime is
separate. Writable mode observes **60 exact focus checks**; read-only observes
**5**. A held synthetic typed 429 reports safe fixed guidance verbatim, clears the
field and retains retry focus. Repeated field Enter sends one request over the
explicit 150 ms observation; a subsequent actual unlock succeeds. This synthetic
response qualifies UI handling, not authentic server timing.

Real Enter confirmation observes Saved, modal close and heading/error focus.
Native Tab continues to Refresh and the committed export download; Enter obtains
the bound report. Expected prerequisite/refresh faults remain explicit. Both
modes report zero page errors and zero non-loopback page requests and retain
storage/route checks. Pages fit 640/320 CSS pixels; writable mode retains five
complete operation IDs within their rows at both widths. Input-border ratios
remain 6.598:1 against the fill and 6.136:1 against the adjacent page. These fixture
measurements do not establish full accessibility or OS network denial.

The first native attempt failed a harness assertion because Playwright
`isDisabled` includes `aria-disabled`. The successor checks native `disabled`
and `isConnected` while retaining the accessibility-state assertion. No product
change was needed; the failed log remains. Earlier source failures/hangs remain,
including an initial timeout that lost buffered stdout; it earns no credit. The
first LLVM argument error ran no tests; the corrected command supplies actual
coverage. Historical receipts retain their original bytes and denominators.

## Remaining gates

Full keyboard golden-path coverage, real AT announcement timing, the supported
browser/platform/AT matrix, WCAG/ASVS/security/owner acceptance, dependencies and
release provenance, and real-user evaluation remain open. A11Y-C-1 literally
requires invoker restoration on every close; a confirmed refresh detaches its
invoker and this slice uses deliberate heading/error fallback. That scope needs
disposition. A11Y-F-5 broader envelope/field/resource linkage also remains open;
safe throttle text alone does not close it.

At the end of all roadmap work, review and update integrated CLI/API, examples,
architecture, schemas, dependency/provenance claims, platform support,
verification instructions, requirements and acceptance status. This focused
record, passing tests and draft PR do not satisfy that final docs gate.

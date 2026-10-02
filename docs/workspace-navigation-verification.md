# Workspace navigation development verification

This record covers the focused F05 navigation fix against baseline `aef0ab24`.
It records historical applied-source development checks before PR drafting.
The later [live-state record](workspace-live-state-verification.md) binds the successor assets. It does not close
F05, PRD-062, a browser/platform acceptance gate, or the final documentation review.
The [machine-readable receipt](workspace-navigation-verification.json) pins the
source, both harnesses, API contract, development binary, logs and raw V8 output.

## Documentation and coverage

All **7/7 changed named production functions** have adjacent JSDoc comments and
executed in the actual-source Node run: `renderView`, `allowViewChange`,
`focusViewTitle`, `navigate`, `confirmDiscard`, `button`, and `showProvenance`.
All **4/4 new browser helpers** have adjacent JSDoc comments. The check matches one
closed comment immediately before its declaration; it cannot cross intervening
comments. Across the asset's 27 named function declarations, 7 have JSDoc. That
whole-asset denominator does not include anonymous callbacks or arrow functions.
The Python harness change adds no function; its module docstring remains present.
No Rust API or dependency is added.

Node **v22.22.3** executed the complete actual `ui/workspace.js` through `vm.Script`.
All **16 tests passed**, with zero failures, cancellations, skips or todo cases.
V8 reported **28/64 function entries executed** and **88/160 observed raw ranges
executed**. Function identity uses name plus UTF-16 start/end offsets; range identity
uses offsets inside that function. Positive counts are aggregated only for the
exact source file URI. The receipt retains every covered/uncovered range.
These are neither line nor branch/condition coverage percentages. The broad asset
includes workflows outside the navigation suite. Executing all changed named
functions does not establish complete path coverage.

Reproduce source checks without installing dependencies:

```sh
rtk proxy node --check ui/workspace.js
rtk proxy node --check ui/tests/workspace.cjs
rtk proxy node --test ui/tests/workspace-navigation.cjs
rtk proxy env NODE_V8_COVERAGE=/private/tmp/forge-f05-v8 node --test ui/tests/workspace-navigation.cjs
```

## Actual browser runs

Installed Google Chrome **154.0.8037.97**, driven by development-only Playwright
**1.62.1** and browser-driver Node **v24.19.0**, ran on macOS.
The historical JSON receipt already records that exact browser-driver runtime;
Node **v22.22.3** above belongs to the separate actual-source suite. Both POSIX wrappers exited
**0**. Writable mode completed **21** exact focus checks and read-only mode completed
**2**. Both served JavaScript bodies matched that historical candidate byte-for-byte with
SHA-256 `724368a21e2a500f9cb20417ba26a1ff4f079514a18c4cd6305581d6fd54bfdd`.
The development debug binary checksum is recorded separately in the receipt.
No browser/tool installation or new project dependency was needed.

Writable cases include navigation failure/recovery; native Keep editing/Escape;
registration and full scope-document preservation; failed provenance retry;
explicit discard; Trace cancellation with an unsaved report path; and destination
heading focus. Existing registration/conversion/scope/mapping/report workflows,
preview expiry and saved-write refresh failure also ran. The read-only mode checks
its absent write controls and navigation failure/recovery. Both report zero
unexpected page requests and page errors, empty cookies/local/session storage,
and successful narrow-viewport reflow. Other existing focus assertions are outside
the `focusChecks` counter; the receipt names this bounded denominator explicitly.

A dialog becomes hidden before its queued native close handler finishes. The
browser helper therefore waits for exact active-element equality and then asserts
it again; it never moves focus. The fake DOM's synchronous close model alone could
not establish this timing. Initial immediate-assertion failures are retained as
historical diagnostics, not credited as successful runs.

The POSIX wrapper closes its owned PTY master before signalling/reaping its server.
Keeping that master open caused macOS exiting-server reaping to stall during the
initial runs. Final bound runs complete through cleanup. Synthetic passphrases and
files are used; project state is not injected by privileged browser calls.

Run with installed development tooling and a built candidate binary:

```sh
rtk proxy python3 scripts/test_workspace_browser.py --forge /absolute/path/to/forge --node /absolute/path/to/node
rtk proxy python3 scripts/test_workspace_browser.py --forge /absolute/path/to/forge --node /absolute/path/to/node --read-only
```

Set `NODE_PATH` to an existing Playwright installation and
`FORGE_TEST_BROWSER_EXECUTABLE` to the installed Chrome executable as needed.
No installation approval or dependency qualification is inferred from this example.

## Remaining evidence

These runs cover one local macOS Chrome version and synthetic API faults. They do
not establish OS-enforced network denial, the supported browser/platform matrix,
manual screen-reader behavior, the full keyboard-only golden path, WCAG 2.2 AA,
independent security acceptance, authentic user studies, or release qualification.
The [F05 plan](plans/2026-10-02-f05-navigation-remediation.md) preserves remaining
findings and gates. After all scoped roadmap work, review and update the full docs
against integrated CLI/API, behavior, schemas, dependencies and acceptance evidence.

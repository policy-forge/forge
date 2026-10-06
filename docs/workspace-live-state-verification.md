# Workspace live-state development verification

This is the historical pagination/live-state slice. The separate
[keyboard verification successor](workspace-keyboard-verification.md) records
later unlock and confirmation changes; this slice’s JSON receipt and observations
remain unchanged.

This record binds the applied F05 pagination, operation-status and control-border
slice on baseline `01251ab5963d00ca3d8dbfb4132d57cdbb8f2de8`. The
[machine-readable receipt](workspace-live-state-verification.json) retains exact
source, harness, stylesheet, API, tool, debug binary and diagnostic hashes.
These are development checks before draft PR creation. They do not complete F05,
PRD-062, accessibility/security/platform acceptance or the final full docs review.

## Docstrings and executed source

All **24/24 changed named production functions** have adjacent JSDoc and executed
in the applied-source suite. The inventory includes nested named actions/read
guards and const arrow helpers. Across the whole asset's **40 named function
declarations**, **23** have adjacent JSDoc; this separate denominator includes
nested declarations but excludes arrow declarations and anonymous callbacks.

All **29/29 changed/new named test helpers or methods** have useful adjacent
JSDoc/docstrings: **18** in the Node harness, **10** in the browser harness and
**1** in the Python fixture launcher. The Node count includes an existing helper
whose synthetic provenance literal changed; the browser count includes the
existing navigation helper's explicit prerequisite-error case. Unchanged helpers
and anonymous callbacks are excluded. Historical proposal audits retain their
original narrower counts; they are not rewritten as the final inventory.

Node **v22.22.3** executed the complete exact JavaScript asset through `vm.Script`
with its actual file URI. **58/58 tests passed**, with zero failures,
cancellations, skips or todo cases. The JS SHA-256 is
`c61cb6a87ee9473e254e320402ac38c46f244af7797d004696b362755f753f9d`.

V8 reports **84/111 function entries** and **335/440 observed raw range entries**
with positive counts. Function identity uses name and UTF-16 start/end offsets;
range identity uses offsets within that function. Only the exact production file
URI contributes. Every covered/uncovered observation and raw receipt hash is
retained. These are **not line, statement, branch or condition coverage**. The
suite uses fake DOM and timers; all changed named functions executing does not
establish every path, browser layout or assistive-technology behavior.

Cases cover staged cursors/filters and version-bound counts, retained previous
rows, retry/restart and stale replies; progress facts and deduplicated live text;
cancellation acknowledgment versus terminal state, stale success/error races;
known-ID GET recovery versus original-key initial-request retry; retained
preparation after preview-read failure; local errors and focus across navigation;
explicit review and shutdown invalidation. No Rust, schema, API or crate
contract is changed by this slice.

Reproduce the source checks using an existing Node installation:

```sh
rtk proxy node --check ui/workspace.js
rtk proxy node --check ui/tests/workspace.cjs
rtk proxy node --check ui/tests/workspace-navigation.cjs
rtk proxy env NODE_V8_COVERAGE=/private/tmp/forge-f05-live-state-v8 node --test ui/tests/workspace-navigation.cjs
```

The required Rust 1.98.1 commit hook passed formatting, strict all-target Clippy
and the full Rust suite: **2,692 passed / 3 ignored**, zero failures. Its exact log
is retained. The documentation amendment must pass the same hook before drafting;
its result is recorded with the PR/tracker. The scoped V8 receipt does not claim
Rust or whole-repository coverage.

## Native Chrome and rendered controls

Both POSIX wrappers exited **0** on local macOS arm64, Python **3.14.8**, installed
Chrome **154.0.8037.97**, development-only Playwright **1.62.1** and browser-driver
Node **v24.19.0**. Source-test and browser-driver Node are different measured
runtimes. No installation or new dependency was required.

Both served JS and CSS bodies matched the candidate files byte-for-byte. The
stylesheet SHA-256 is
`d4ce961b5043cd2fc756b032583bb99b40d5f42563a3526ab13ebfbbae22fa02`;
the supplied debug binary SHA-256 is
`b3a2a99f3c2fed8a847f36f8d102189eba807775f15cb3074aacdc5d893eb857`.
It is preserved in a separate content-hash directory outside the mutable Cargo
target; both final native runs use that file, whose hash matches before/after.
This debug binary is not a release package or provenance-qualified candidate.

Writable mode completed **38** explicit exact-focus checks and read-only mode
completed **2**. The counter covers calls to the maintained focus helper;
additional workflow assertions are separate. The writable suite exercises a
53-control synthetic Catalog, real pagination and filtered matching/total counts,
retained rows after a documented read fault, native result/error focus, conversion
preparation, known-ID GET recovery, explicit review and confirmed report export.
Prerequisite inventory failures before scope/mapping registration are checked
against exact expected validation text, initializer visibility and error focus.
Ordinary post-registration navigation still requires heading focus.

Both modes report zero non-loopback page requests and page errors, empty cookies
and local/session storage, and successful shutdown. Request interception is not
OS-enforced network denial; the request-shape list does not establish that every
API route was exercised successfully or full API/CLI parity.

Actual passphrase-control border/fill/page colors are respectively
`rgb(82, 96, 91)`, `rgb(255, 255, 255)` and `rgb(246, 247, 244)`.
Measured border contrast is **6.597586:1** against its fill and **6.135505:1**
against the adjacent page. The shared input/select/textarea rule changed from
`#8c9b95` to `#52605b`. This qualifies that authored fixture boundary, not native
subcontrols, forced colors, all UI contrast or WCAG acceptance.

Final-page reflow passed at both **640** and **320 CSS pixels** in both modes.
Writable mode retained **five** persistent operation rows; every heading still
contains its complete operation ID and fits within its row. Long operation text
inherits `overflow-wrap: anywhere` from the operation region. An isolated
before/after CSS fixture reproduces the old overflow and verifies wrapping at
320, 640 and 1280 pixels without changing IDs or text. These observations cover
the tested final pages; they do not establish all views, text sizes or zoom levels.

Run the installed development harness against an explicitly built binary:

```sh
rtk proxy python3 scripts/test_workspace_browser.py --forge /absolute/path/to/forge --node /absolute/path/to/node
rtk proxy python3 scripts/test_workspace_browser.py --forge /absolute/path/to/forge --node /absolute/path/to/node --read-only
```

Set `NODE_PATH` to the existing Playwright installation and
`FORGE_TEST_BROWSER_EXECUTABLE` to the installed Chrome executable when needed.
The receipt identifies the actual paths used. Browser tests still run locally;
the current hosted CI does not run this native-browser harness.

## Retained failures and limits

Earlier failed runs remain pinned as diagnostics. The harness now fulfills a
one-shot fault before removing its route, observes async unlock completion
without setting focus, and recognizes only the declared pre-manifest inventory
errors. Those changes preserve assertions; the readiness wait qualifies
post-unlock behavior, not pending/throttled-unlock keyboard behavior.

A later native run exposed operation IDs overflowing the 640-pixel page. The
production wrapping correction was independently source-reviewed, rebuilt and
rerun in both modes. A disk-full build attempt remains a failed observation;
removing disposable completed build caches and rebuilding did not overwrite raw
coverage, source packets, historical receipts or verification binaries.

The historical navigation JSON is byte-for-byte preserved. Its prose now correctly
identifies browser-driver Node v24.19.0; the JSON already carried that value.
Proposal-side packets and narrower helper inventories remain historical evidence,
separate from this final applied receipt. The first successful final-asset browser
runs used debug bytes `031017…`; the mandatory Rust hook rebuilt that mutable
binary path. Those logs and the first receipt are preserved, with the original
binary's non-retention stated explicitly. The final runs were repeated against
the preserved `b3a2…` artifact rather than relabelling the historical runs.

Complete keyboard-only golden paths, pending-unlock actions, screen-reader live
announcements, the approved browser/platform/AT matrix, WCAG 2.2 AA, independent
security acceptance, dependency review, real user studies, packaging/provenance
and overall PRD-062 completion remain open. After all roadmap work, review and
update the full integrated docs: CLI/API, examples, architecture, schemas,
dependencies/provenance, platform claims, verification instructions, requirement
status and acceptance evidence. This focused review does not satisfy that gate.

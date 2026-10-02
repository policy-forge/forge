# Hosted browser first-fault diagnostics

## Immutable hosted outcome

[Run 37070371407](https://github.com/policy-forge/forge/actions/runs/37070371407)
tested requested head `ad149144b98fadd22f32c7db84cdf839644f890e`, base
`058bb47b81316c4f6aecc8e855111df4709a34d2`, and merge
`f587ce77f44f47c8ebb6692a558adc63628b67cd`. Ordered parents and equal
merge/head trees were verified. The artifact digest, exact requested-head input
pins, release/tool identities and four ordered campaign rows were independently
qualified. Wrapper exit was 2 and producer exit was 1; the hosted job failed.

Default read-only, default writable and long read-only passed with actual
natural Forge/Node zero exits, terminal EOF, empty owned trees, no forced
cleanup and both credential submissions observed without echo. Their focus
counts were 16, 71 and 18; resource denominators were 2, 6 and 2; local download
sizes were 645, 1382 and 809 bytes, each with the separate exact 11-byte raw
envelope overhead. The default campaigns retained two viewport records;
long read-only retained six. All three captured-registration counter tuples
were zero, so no transient running-counter observation is claimed.

Long writable failed `browser-failed` with a null observation and forced
cleanup. The actual failing assertion, timeout or other cause was not retained.
This result establishes three campaign passes and one failure, and cannot earn
the four-campaign prerequisite. Canonical installed Google Chrome was
154.0.8037.57, with the approved Node/npm and package graph. Earlier startup
failures remain preserved. This result does not prove their hypothesized cause.
Sibling API artifacts remain separate observations; no full CI or browser
matrix acceptance follows from them.

## Diagnostic contract

Producer `/2` and wrapper `/3` add a nullable `browser_failure` to each campaign.
Passed rows require null. A nonzero actual Node exit can attach a diagnostic
only when the complete bounded stdout is exactly one closed
`forge.workspace-browser-failure/1` object with `stage`, `category` and
`assertion_operator`. The Python producer adds `node_exit_code` from its actual
Popen result. Unknown, oversized, duplicate-key, nonfinite, multiple or malformed
JSON leaves a failed row with null diagnostic; it can never grant pass.
Shared API `/2`, aggregate pipe limits and the four-campaign denominator remain
unchanged. Neither CJS nor its diagnostic can supply a claimed exit status.

The 23 stage values describe coarse maintained workflow phases. They do not
identify a particular assertion or prove its cause. Only main-flow statements
set stages; asynchronous observation callbacks supply no stage values.
Categories are `assertion`, `timeout` and `unclassified`. Assertion classification
uses an actual Node AssertionError; operators are limited to `strictEqual`,
`deepStrictEqual`, `match` and `==`. The actual Node `assert(...)` operator is
`==`. A timeout uses the approved Playwright TimeoutError class. Name-only
impostors, unknown operators, accessors and proxy reflection faults cannot add
private diagnostic data. No messages, stack, DOM, labels, URLs, tokens, paths,
actual/expected values or screenshot contents enter this envelope.

The first fault and its stage remain primary even if private diagnostics,
screenshot capture or cleanup later fail. Startup is covered before browser
ownership. Both owned context/browser closes are independently attempted.
Successful observation output is deferred until cleanup reconciliation;
a cleanup-only failure revokes success. Exactly one outcome object is emitted.
The wrapper still requires actual child zero exits, EOF, no forced cleanup,
owned-tree proof, complete ordered campaigns and unchanged source/tools for pass.
Arbitrary stderr and screenshots remain private and are not uploaded.

## Before-draft source and control evidence

[The versioned control audit](2026-10-02-f04-hosted-chrome-control-audit-v4.json) records exact source pins, Python controls,
named docstrings/actual calls and every compiler zero, plus the separate actual
Node V8 inventories. Python passed **54 controls** with no failures/errors/skips.
All **128/128** named functions, **3/3** classes and **4/4** modules have
docstrings; **126/128** functions were actually called. All **21/21** changed
or new named functions were documented and called. `OwnedTree.settle` and
`file_limits` remain uncalled. Root observed **950/1180** compiler-mapped
physical lines and retains all **230** zeros. The author collector inventoried four fewer module/class docstring zero
entries, reporting 950/1176 with 226 zeros; the common positive line sets
match. Imports preceded tracing; threads/children
and native execution are unmeasured. The 47 preceding cases preserve their
assertion calls except the disclosed wrapper expected-version literal
change from `/2` to `/3`; fixture adapters add null diagnostic/source pins. The new Node controls passed **17/17**, with no failures,
skips or cancellations, using the exact approved Node and installed package
files. All **29/29** selected helper/control named declarations have adjacent
JSDoc and actual positive V8 primary-range counts. The helper has 9/9 callable
records observed; controls have 52/55. Zero ranges and unobserved records are
retained. This is a call observation, not full-body or branch coverage.

An actual AST comparison using the already approved Playwright bundled parser
found all **136 original assertion calls** and their exact source bytes equal.
Explicit waits and default timeout calls also remain AST-identical. The actual
maintained CJS was executed in three startup mocks only: its inventory has 86
callable AST nodes, while V8 instantiated 56 records and observed two. No UI
interaction, native browser or product execution is credited to these controls.
The workflow adds only the diagnostic control step after approved provisioning;
it introduces no dependency, install or browser download.

This is a precommit source snapshot. Mandatory enabled hook results and a fresh
immutable hosted outcome must be recorded separately. Previous audits and
hosted receipts remain byte-for-byte historical evidence.

## Remaining gates

Draft PR #185 and Veans #31 remain In Review under open F04 #6. Fresh complete
installed-Chrome campaigns, normative Windows/macOS browser slots, full API
functional coverage, IPv4/IPv6 OS denial and attempted-egress observation,
accessibility/AT/human/security/pilot acceptance, dependency source audits and
release provenance remain open. The full docs review/update at the end of all
roadmap work remains required. No GitHub merge, tracker closure, publication or
participant contact is authorized by these local controls.

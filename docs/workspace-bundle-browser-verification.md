# Browser metadata verification

This unreleased, precommit development record is based on `66c2002dc910f64292c9fb936155a0ca43ca1b01`. The
candidate commit is deliberately unset; the enabled hook and actual draft
delivery have separate later receipts. It does not establish full PRD 062 S-6
or roadmap acceptance. The [machine record](workspace-bundle-browser-verification.json)
retains exact source pins, complete production V8 mappings/zero records, helper
inventories and all seven scoped native campaigns.

## Docstrings and changed-code coverage

The fresh final source run passed **105 tests, 0 failures, 0 cancellations,
0 skips and 0 todos**. Its source helper includes the 16 fixture-comment
successor; complete executable AST equality was checked against its predecessor.
All **23/23** changed production named helpers have adjacent JSDoc and positive
exact primary V8 records. These include 20 new helpers and 3 changed existing
helpers relative to the immutable base. Calls do not establish complete branches.

| Scope | Documentation / total | Observed coverage |
|---|---:|---|
| Changed production named helpers | 23 / 23 | 23 positive primary records |
| Whole production named helpers | 61 / 80 | 71 positive, 7 mapped zero, 2 absent |
| Whole raw V8 functions | presence inventory separate | 125 / 150 positive; 25 zero |
| Whole raw V8 ranges | presence inventory separate | 553 / 677 positive; 124 zero |
| Changed source-helper core functions/methods | 13 / 13 | Documentation inventory only |
| Whole source-helper core functions/methods | 78 / 78 | Documentation inventory only |
| Changed source route-fixture property callbacks | 16 / 16 | Separate property cohort |
| Whole source route-fixture property callbacks | 16 / 82 | 66 unchanged legacy gaps |
| Changed native helper core functions | 2 / 2 | Documentation inventory only |
| Whole native helper core functions | 20 / 21 | Unchanged `register` gap retained |
| Changed Python launcher functions | 1 / 1 | Whole launcher 2 / 2 |

Production's 108 unbound syntax callbacks are separately mapped: 53 positive,
18 mapped zero and 37 absent. One positive module wrapper is separate. The source
and native helper inventories separately retain 214 and 64 anonymous/unbound
callbacks. Neither callbacks nor V8 functions are test counts. Raw ranges use
UTF-16 offsets and maximum counts per exact identity; they are not statement,
branch or MC/DC coverage. Whole production documentation retains 19 legacy gaps.
The original helper selection of 17 collided getter/setter keys and is
superseded by the accessor-aware 13, with historical inputs preserved. The lane
that authored fixture comments is excluded from independent correctness credit.

Rust/API/client/schema/dependency source is unchanged from the base. The earlier
full Rust run passed **2,755 tests with 3 ignored**, across 69 summaries, before
the CSS and helper-comment successors; production JS was identical. Final local
CI and the enabled hook are separate current-source checks. No new Rust LLVM
line cohort is claimed, and predecessor coverage is not promoted to new credit.
The maintained client regression also passed against the preserved final binary.

## Actual browser observations

Four final macOS Chrome **154.0.8037.97** campaigns used production JS
`7fe7f9d4b424d86319e1d536cc0ecc21361c05688a1cec8b0ed2be8a42de190b`,
CSS `fd141402dcc1c52cd8955615d45c994243d22f2fbc1e376536c7f32323caf79e`
and preserved CLI `bcd751b02fdfba65bb27ccd0df0a38a569880d000c6174c80ca12bb1284f3198`.
Native driver/launcher bytes match the final candidate. The source-only comment
successor was added after native execution, preserves its executable AST, and
was covered by the fresh 105-test source run. Recording-time binary pins do not
claim startup attestation. Production and served-asset hash assertions ran.

| Campaign | Focus observations | Actual preview/comparison rows | Download / POST bytes | Duplicate-key status |
|---|---:|---:|---:|---:|
| Default writable | 71 | 6 | 1,382 / 1,393 | 400 |
| Default read-only | 16 | 2 | 645 / 656 | 400 |
| Long writable | 73 | 6 | 1,569 / 1,580 | 400 |
| Long read-only | 18 | 2 | 809 / 820 | 400 |

Each bound helper checked the local download against the actual GET bundle,
sent its original bytes in the 11-byte POST wrapper, reconciled the complete
2/6-row expected inventory, observed strict duplicate rejection and cleared
acknowledgment on refresh. The native File was supplied through the file input;
the OS chooser was not exercised. Raw downloads and complete response bodies
were not separately retained. Log digests and executed byte-equality assertions
are retained; no independent new digest of omitted bytes is claimed.

Both long campaigns rendered the authored 200-scalar literal label and
185-scalar filename. Four actual measurements per campaign—after preview and
after file choice at 640/320 pixels—matched viewport/global page widths. Internal
table scrolling was allowed; the helper explicitly restored viewport/focus.
Ordinary final-page reflow also matched 640/320. Long writable begins with a
preauthored empty index before normal UI registration; default writable retains
its original missing-index start. These fixtures are separately scoped.

The preserved unwrapped read-only predecessor failed before comparison: page
widths were 1777/1761 after preview and 2148/2132 after choosing a file at
640/320-pixel viewports. It has no later comparison, duplicate rejection,
final-download digest, final focus count or full-workflow credit. The focused
`[data-bundle-panel] { overflow-wrap: anywhere; }` successor corrected that
observed overflow. Two earlier ordinary campaigns remain historical.

All four final campaigns had zero page errors and zero non-loopback page
requests. This is page-level observation, not OS-wide network denial. They
observed zero transient capture counters and add no running/cancellation credit.
Focus observations overlap and are not independent test or participant totals.

## Repository checks and remaining gates

The required local CI exit was **255**.
Recorded steps: cargo fmt --check: passed; cargo clippy --all-targets -- -D warnings: passed; cargo test: passed; API contract validation (PRD-062): passed; cargo audit: passed; cargo deny check: passed; cargo vet --locked: failed. The full Rust phase passed 2,755 tests with 3 ignored;
the subsequent contract phase repeated 9 of those tests. Their 2,764 aggregate
is not a unique-test total. Benchmarks were skipped by the repository default.
Audit passed with one allowed warning; deny passed. Vet failed with 20 unvetted
dependencies lacking `safe-to-deploy` approval. This approval gate remains open.
Exact logs and digests are retained in the machine record. Guide grammar/link
corrections and the new report followed that run; the enabled commit hook checks
the final staged tree. All CI and release gates are not claimed passed.

Full S-6 still requires receipt-backed server publication/export, source-content
opt-in, confirmed writable import, reviewed batch binding and capacity/retention
qualification. Source-double 1,000-row cases and existing backend capacity tests
do not establish native 1,000-row or maximum-file acceptance. Platform, named
browser/interoperability, MSRV, OS-wide offline, security/privacy, human
keyboard/AT/WCAG, pilot and release acceptance remain separate. The requested
full integrated documentation review across all completed work remains an end
gate, followed by a verified 2.0.0 candidate and publication authorization.

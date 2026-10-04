# Evidence inspection Windows path correction verification

This is the earlier source-bound path-correction checkpoint. Subsequent hosted
Windows exposed a raw-spelling test-fixture normalization issue; production
predicates remain unchanged. See the [raw-spelling fixture successor](poam-evidence-inspection-windows-raw-fixture-verification.md)
for its measured correction and pending corrected-head Windows qualification.

The inspector accepts a valid native nested companion path on Windows while
keeping portable manifest names and raw path spelling checks. It checks the
original absolute spelling before extracting the descendant, then checks native
relative spelling before making a portable view. Capture keeps the native path.

Hosted Windows tests at `d5606c1408403e4c07f5093bb9d0d802b115c252` failed in
seven S4 integration controls at their shared nested-companion setup. Linux and
macOS Rust tests and all three platform API jobs passed at that same head.
The first correction's new regression test caught a trailing-separator alias:
its local run had 2250 passes and one failure. Both failures remain preserved.

The corrected source has five affected documented functions, including three
new registered regression controls. All three control bodies remain unchanged
from the failing local run. The reused spelling helper's body is unchanged.
The whole 13-file S4 cohort has 404 documented
functions out of 592; broader omissions stay
distinct from the five affected functions.

The actual macOS all-feature LLVM run passed 3262 tests, failed
0, and retained 3 existing ignored tests across
70 summaries. It measured 82988 of
90849 lines (91.3472%),
7403 of 8800 functions,
and 145379 of 160352 regions.
Every affected function has a positive exact-header alias. All its zero aliases,
the complete raw records from the two changed files, and global zero indices
remain in the [machine record](plans/2026-10-04-f08-evidence-inspection-windows-path-verification.json).

These are local source-bound measurements. Windows execution of this successor
remains pending hosted CI. Branch and full-body coverage are not inferred from
header entry. The previous [integration verification](poam-evidence-inspection-verification.md)
and its machine record retain their original source generation and measurements.
Owner acceptance, release acceptance and the final full documentation review
remain open. The normal commit hook is required for delivery; this precommit
record does not claim it has run.

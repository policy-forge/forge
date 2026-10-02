# Current stable Clippy assertion compatibility

Date: 2026-10-02. Forge tracker task: #11. Base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`.

## Trigger and change

The workspace CI follows the current stable Rust toolchain. Rust 1.99 reports the
`clippy::assert_is_empty` warning, which fails the existing `-D warnings` gate.
The initial F07 CI diagnostics identified 76 existing assertions. Complete local
all-target/all-feature checking exposed further sites; the final correction
covers 110 assertions across 44 Rust files (34 library test modules and ten
integration test files).

This change replaces those assertions with equivalent equality comparisons:
88 empty checks use `assert_eq!`, and 22 non-empty checks use `assert_ne!`.
Collection comparisons use explicit empty element types where inference needs
help; strings compare against the empty string. The asserted expressions and
truth conditions are preserved, and failures now display the compared values.
Custom collection wrappers and types without the comparison traits retain their
existing assertions. Production logic, dependency versions, fixtures, feature
flags, and lint policy are unchanged.

## Before drafting the PR

Docstring coverage audit: this slice introduces no production functions, types,
fields, methods, or signatures. Existing Rust documentation remains unchanged;
there are no newly introduced public API items requiring documentation.

Test coverage audit: all Rust edits are existing assertions in integration tests
or `#[cfg(test)]` modules. Each library file's bytes preceding its test module
were compared before and after the edits and remained identical. There are no
new production executable lines, so this slice has no changed production-line
coverage denominator and does not claim a whole-repository coverage percentage.
The existing tests exercise the same behavior and preserve every selected
empty/non-empty expectation. No duplicate tests are added for this mechanical
assertion change.

Required coordinator checks before PR creation are current stable
`cargo clippy --locked --offline --all-targets --all-features -- -D warnings`,
formatting, and the mandatory pre-commit test gates. Retain the exact head,
Rust version, command results, test totals, and ignored-test count in the PR.
Strict Rust 1.99 all-target/all-feature Clippy passed after the corrections.
Formatting and the mandatory full commit hook still require completion before
drafting. The [source audit](2026-10-02-current-stable-clippy-audit.json) records
each changed file and confirms unchanged production prefixes. Hosted CI must be read back at the resulting
PR head, separately from local results.

## Remaining gates and final documentation review

This slice addresses assertion compatibility only. Existing dependency audit
and acceptance requirements remain active and must not be waived to obtain a
green Clippy job. It does not establish roadmap or release acceptance.

At the end of the full roadmap work, review and update the complete documentation
against the integrated implementation and verified behavior, as requested by
the user. Reconcile CLI and API references, examples, architecture, schema and
dependency provenance, supported platforms, verification evidence, requirement
status, and acceptance gates. Keep human and external acceptance decisions
explicit until authentic evidence satisfies them.

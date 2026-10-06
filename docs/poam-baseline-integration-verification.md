# POA&M baseline integration verification

This records the read-only baseline slice for the frozen original PR #204 cohort
at `62dbea7a6ccb54bc13da9672a4a4a031bec7e0c1`. It does not close full F08 or the
goal-wide final docs review. The
[machine-readable receipt](plans/2026-10-03-f08-baseline-integration-verification.json)
retains source pins, every lexical row and omission, complete native aliases,
historical failures and recorded documentation differences.

| Lexical dimension | Selected new/changed | Whole six owned Rust files |
|---|---:|---:|
| Functions with Rustdoc | 61/61 | 210/262 |
| Types with direct Rustdoc | 15/15 | 71/71 |
| Members with direct Rustdoc | 102/102 | 544/627 |
| Modules with Rustdoc, separate optional dimension | 2/3 | 14/26 |

Selected functions comprise 30 production, 18 cfg-test and 13 integration
declarations, including 23 new literal test controls. The selected module gap is
the private baseline cfg-test module. Unchanged whole-file omissions remain
explicit; they are not counted as newly documented work.

Actual LLVM observed all 61 selected headers with positive entry counts, no zero
headers and no unmapped headers. All 99 exact aliases are retained, including
seven zero-count aliases. Across the six files, all 262 headers have positive
entries; 374 aliases include 34 zero-count instances. These are exact first regular
code-region filename/header-line/UTF-8-column associations. Nearby closures or
generated functions never substitute for an unmapped header. Positive entries do
not prove complete bodies, branches, every instantiation or every platform.

The full managed LLVM run passed **3,159 tests**, with zero failures and three
ignored in 70 summaries. Whole-workspace lines were **77,357/84,817 (91.20%)**;
functions were **6,960/8,204 (84.84%)**. Instantiations were 9,546/14,265 and
regions 135,177/149,131. These aggregates include production, tests and generated
code. Branch and MC/DC denominators are unavailable. The full export retains
18,106 raw function records, including 5,920 count-zero records; that raw record
count is separate from aggregate functions and lexical declarations.

All six Rust files in that frozen original cohort match the full run byte-for-byte.
After that run, the command guide gained an explicit as-of clarification: comparison
uses final supplied states and complete histories, without reconstructing historical
state. Both guide versions and their source-bound semantic review remain recorded.
Formatting passed on that integrated cohort. Strict all-target/all-feature Clippy
passed on its byte-identical Rust source in the isolated prototype.

The subsequent Windows cfg test correction changes only
`tests/poam_foundation_test.rs`; the five production Rust files remain unchanged.
That correction is not measured by the original LLVM export or lexical census
above. Fresh checks, the current mandatory commit hook and later hosted checks
are separate evidence recorded when executed.

## Authentic controls and corrections

The original public CLI with only the nine new CLI controls passed two refusal
cases and failed seven positive cases because `poam baseline` was absent. An
earlier sccache bootstrap failure ran zero tests and earns no product-red credit.
The combined proposal first encountered a cfg-only borrow compile error; a cloned
test digest corrected it without changing production predicates. Then 10 core,
two renderer and all 53 foundation/CLI cases passed.

Formatting was corrected separately. Strict Clippy initially found one test
allocation cast, nine empty-output assertion forms and one test assignment
semicolon. Checked conversion and equivalent assertions fixed those cfg-only
diagnostics. Original proposals, failure logs and versioned corrections remain
unchanged. The final full LLVM run executes the corrected final source; earlier
prototype counts are never summed into its cases. A separate pure reader writer
NameError is retained as a metadata error, not a product or native failure.

The controls cover complete stable identity partitions and changed fields, current
source membership, empty/removal/terminal/history refusals, explicit reopened-key
relationships, original-byte hashes, strict dates, bounded closed inputs,
complete escaped text, safe new-file output, source drift and ordinary artifact
guard preservation. The comparison always reports `artifact_validated: false`
and proposed reopening always reports `accepted_for_build: false`.

## Remaining gates

Accepted closure/supersession semantics still require D064 disposition. Full F08
portfolio/HTML/outbound adapters, fresh evidence links, independent OSCAL consumer
qualification, representative plans and real remediation pilot judgments remain
open. Native build/check retain their existing terminal/removal/history rules.
No actor authority, evidence sufficiency, audit approval, human acceptance,
merged delivery or release readiness is inferred from this report.

# Evidence inspection Windows raw-spelling fixture verification

The Windows negative fixture now constructs raw absolute path spellings with
`OsString` concatenation. It avoids `PathBuf::join` normalization of dot segments
under a canonical Windows verbatim prefix. The production descendant guard,
capture limits and authority predicates are unchanged.

Hosted Windows at the earlier correction still failed this one raw-alias control.
That retained failure motivated a fixture correction, not a relaxed production
predicate. The original Windows path report and initial S4 audit stay byte-exact.

The actual source-bound local whole LLVM run passed 3262 tests,
failed 0 and retained 3 existing ignored tests
across 70 summaries. Its format job also completed successfully.
The one changed registered control is documented. Its header association is
exact-header: 1 positive and
0 zero compiler aliases. Related in-range raw
compiler aliases, every changed-file raw record, global zero indices and complete
13-file lexical omissions are retained in the
[machine record](plans/2026-10-04-f08-evidence-inspection-windows-raw-fixture-verification.json).

These local measurements do not establish corrected Windows execution, branch
coverage or native/human acceptance. [Earlier Windows path verification](poam-evidence-inspection-windows-path-verification.md) remains a historical
source generation. Fresh corrected-head Windows CI, owner/release acceptance
and final full-goal documentation review remain open. This precommit record does
not predict a hook, commit, PR or CI outcome.

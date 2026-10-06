# POA&M foundation integration verification

The integrated source foundation captures five confined native JSON companions,
selects an Assessment Results epoch by UUID and stable key, and emits an empty
`forge.poam/1` scaffold. Source-only checking validates the capture and complete
selected-result inventory. It does not assign remediation owners, choose eligible
items, assert review or closure, or build a native POA&M document.

This report was prepared **before the draft PR** on parent
`2de6c5e8c347a1e81048b009fb8abd3a3014076e`. The [machine-readable report](plans/2026-10-03-f07-foundation-integration-verification.json)
binds all 34 final source files, 27 Rust files, retained lexical declarations,
actual raw LLVM entries and nine completed job receipts/logs. Root's native
association agrees with the independently derived peer metadata on every one of
824 ordered named occurrences and all exact header aliases. The peer did not run
native tests or confer product acceptance. Retained source copies allow pure
replay after later stack changes; current files must be checked separately.

## Documentation census

| Denominator | Documented | Total |
|---|---:|---:|
| Selected named functions | 190 | 190 |
| Selected brace-bodied types | 23 | 23 |
| Selected named fields and variants | 77 | 77 |
| Whole named functions in 27 files | 395 | 824 |
| Whole brace-bodied types | 85 | 124 |
| Whole named fields and variants | 314 | 631 |

Selected functions comprise 104 production, 45 cfg-test and 41 integration lexical
rows. There are 65 selected literal test declarations and 400 whole-file literal
test declarations; these are source counts, not runtime outcomes. The report
retains all 429/39/317 whole-file documentation omissions. An adjacent Rustdoc
comment demonstrates lexical presence, not its semantic completeness or quality.
Platform alternatives, multiline attributes, UTF8 offsets and ordered repeated
declarations remain explicit. The sole Rust formatter bridge wraps a new negative
control in `workspace/services.rs` without changing its tokens; the plan's updated
application introduction is a separate non-Rust change.

## Actual tests and coverage

| Completed job | Passed | Failed | Ignored |
|---|---:|---:|---:|
| Library filter `poam` (includes related controls) | 37 | 0 | 0 |
| Foundation integration (22 direct plus 4 common) | 26 | 0 | 0 |
| Schema provenance | 11 | 0 | 0 |
| OSCAL compatibility | 9 | 0 | 0 |
| Complete-value export regression | 22 | 0 | 0 |
| Assessment baseline regression | 16 | 0 | 0 |
| Whole workspace LLVM run, 70 summaries | 3,096 | 0 | 3 |

Format and strict all-target/all-feature Clippy also passed. Repeated jobs are not
added to a unique test denominator. Controls cover complete finding/risk inventory,
explicit second-result selection, stale pins, same-byte replacement, mismatched
context/imports, confinement, duplicate tuples/keys, schema identity, empty native
document refusal, new-file publication, exporter POA&M refusal and all five
incompatible existing workspace resource roles. Existing integer export and
single-epoch baseline regressions passed unchanged. No pre-fix runtime red is
claimed for a previously absent new module.

The exact named-header association measured **186 positive, 4 zero and 0 unmapped
out of 190 selected functions**. All 309 matching aliases remain, including 51
zero aliases. The four selected zero entries are:

- `cli/config_check.rs`: `schema_type_label` (line 149).
- `cli/validate.rs`: `build_round_trip_result` (line 257).
- `poam/report.rs`: `BoundedJson::flush` (line 109).
- `poam/source.rs`: `CanonicalHash::flush` (line 843).

Whole-file named entries measured **803 positive, 11 zero and 10 unmapped out of
824**. All ten unmapped rows have explicit platform conditions: five alternate
same-name lifecycle bodies, three Windows helper-module functions and two Windows
tests. They remain unqualified. A positive header means at least one exact entry
count is positive; it establishes neither full-body coverage nor all alias,
branch, platform or acceptance coverage. The report retains every matching zero
and nonheader entry as diagnostic data without substitution.

Raw LLVM totals across the entire export are 6,729/7,928
functions and 74,923/82,287 lines. These mix production,
test and generated code and do not replace the selected denominator. Branch and
MC/DC denominators are zero and therefore unavailable. Integration tests omitted
from `files[]` retain their `functions[]` records but receive no fabricated line
coverage. Per-job before/after source pins agree; no process-wide writer, executable
or profile provenance, or timestamp attestation is inferred.

## Scope still open

Full F07 item-selection/remediation contracts and F08 owner, milestone, history,
closure, schedule/as-of, baseline, portfolio and native build workflows remain.
D064 authority/native-subset disposition, independent OSCAL consumer validation,
real remediation-team judgments and remaining platform/audit/release acceptance
are separate gates. The existing nested Assessment Results producer-href issue
needs its own correction; exact companion resolution remains enforced.

The pristine POA&M schema is OSCAL 1.2.3, 148,253 bytes, SHA-256
`f4fd94487408a9589954b5b92d88965c984d4f79b85365cc574b860898759437`.
The manifest's 12 assets comprise eight runtime JSON schemas and four test XSDs;
this does not add a schema to release archives automatically. Generic export,
trace and diff refuse POA&M, and the workspace closed role/API/Bundle3/4 contracts
are unchanged. No new dependency is introduced.

The earlier standalone draft and measurements remain historical. The complete
goal-wide integrated documentation review and verified 2.0.0 release candidate
are still required. See the [source guide](poam-foundation.md) and
[integration plan](plans/2026-10-03-f07-foundation-integration.md).

# POA&M portfolio reports

Use `forge poam portfolio` to review several explicitly supplied plans in one complete JSON report, with an optional static HTML view. Supply the authoring declaration and its matching native POA&M for every plan. The command reads the plans and their source bundles; it does not change them or assert that remediation has been completed.

## Choose the inputs and outputs

`--manifest` and `--native` are repeatable and paired by position: the first manifest with the first native filename, then the second pair, and so on. Supply the same number of each, from 1 to 32. Manifests may be in different directories. Each `--native` value is a single `.json` filename beside its corresponding manifest, not an absolute path or a nested path.

Choose an existing directory for `--output-root`. This option is required even when JSON goes to stdout. Optional `--report` and `--html` values are new filenames inside that directory, with `.json` and `.html` suffixes respectively. Neither output may already exist, collide with an input, or alias the other output. Use portable ASCII names containing letters, digits, hyphens, underscores and dots, at most 128 bytes; device names and trailing dots are refused.

For example, with `./portfolio-reports` already present and both output filenames unused:

```bash
forge poam portfolio \
  --manifest ./plan-a/workflow.json --native native-poam.json \
  --manifest ./plan-b/workflow.json --native native-poam.json \
  --as-of 2026-02-06 --due-soon-days 7 \
  --output-root ./portfolio-reports \
  --report portfolio.json --html portfolio.html
```

Omit `--report` to send the complete JSON to stdout; `--html` remains independently optional:

```bash
forge poam portfolio \
  --manifest ./plan-a/workflow.json --native native-poam.json \
  --as-of 2026-02-06 --due-soon-days 0 \
  --output-root ./portfolio-reports
```

There is no `--format`, authoring-only preview mode or terminal override for this command. `--as-of` is required, calendar-valid `YYYY-MM-DD`, and never taken from the clock. The inclusive `--due-soon-days` interval is 0–365; omission selects zero, which still includes open records due on the as-of date.

## What the pair must establish

The native file must match the entire supported native POA&M generated from its authoring declaration and the actual current five-file source bundle. Matching only a UUID or selected fields is insufficient. Unknown native fields or extensions are refused. Outer JSON whitespace and member order may differ, but the complete decoded value, including source href/hash values and namespaced declaration strings, must match. This is a supported companion-pair check, not a general OSCAL import or interoperability claim.

Each plan must pass the existing nonterminal workflow and current source checks. Completed or risk-accepted history remains refused under the existing closure boundary. Cancelled work is retained in reports and does not close a source finding or risk. Duplicate stable plan identities, unsafe or aliased input files, incomplete pairs, stale sources and any invalid plan fail the whole command; a successful prefix is never returned.

## Read the complete report

JSON uses `forge.poam-portfolio/1`. JSON and HTML show the same complete plan, source, schedule, timeline and selected source-to-work information. Counts sum occurrences within each plan; they are not a unique union of shared source objects across the portfolio. A shared source file or selected source object can therefore contribute again in another plan.

Every item and milestone stays in the denominator. Target dates are required; a null `milestone_key` identifies an item row. A null `state_at_as_of` means there was no assertion on or before that UTC date; it does not mean the row was omitted or healthy. The report distinguishes that state from the final supplied declaration. Cancelled rows and events after the as-of date remain visible, and the timeline retains complete history rather than trimming future events.

Overdue means an open row is due before the as-of date. Due soon includes open rows due from the as-of date through the selected interval, including both endpoints. Blocked rows are classified from their assertion at as-of. Exit statuses are:

| Exit | Meaning |
|---|---|
| 0 | Complete valid report with no overdue, due-soon or blocked row. |
| 1 | Complete valid report with at least one overdue, due-soon or blocked row requiring review. |
| 2 | Invalid input, failed source recheck, rendering bound or output failure. An earlier output can remain. |

These classifications do not establish source eligibility, effective remediation, authenticated ownership or approval.

## Publication and limits

Both requested outputs are fully prepared and bounded before anything is emitted. Output collision checks compare complete absolute paths against every captured authoring, native and source input, including conservative ASCII case aliases. Inputs are rechecked immediately before JSON publication and again before HTML publication. Each output is published separately with no replacement; if a later recheck or HTML write fails, earlier JSON or stdout output is not rolled back. Treat exit 2 as failure even if a file looks valid. File publication uses the existing Linux/macOS no-replacement publisher and refuses unsupported platforms; stdout is a separate destination.

Native hrefs remain display text relative to each original plan bundle. The HTML never activates them as links, and changing `--output-root` does not rebase them. Keep the original bundle context when interpreting those references.

Whole-portfolio ceilings apply before a complete success:

| Limit | Ceiling |
|---|---:|
| Explicit native/authoring pairs | 32 |
| Retained original authoring bytes, summed | 16 MiB |
| Supplied native bytes, summed | 10 MiB |
| Five-source raw byte occurrences, summed per plan | 100 MiB |
| Selected items / milestones | 10,000 each |
| Item and milestone history events | 100,000 |
| Source selections / owner declarations / dependency edges | 100,000 each |
| Complete JSON / complete HTML | 10 MiB each |

Each authoring file is limited to 4 MiB and each source file to 50 MiB. Existing depth, string, per-record history, ownership and dependency bounds also apply. Exceeding a bound refuses the complete result rather than truncating it. The retained-byte ceilings are not a promise about peak memory use.

## Sensitive metadata and remaining scope

Reports omit authored remediation and assessment prose, but keys, UUIDs, role/party identifiers, dates, filenames, source hrefs and rationale digests can still identify people or correlate activity. A digest is not anonymization. Original supplied-file hashes and generated native-byte hashes are distinct: formatting differences can produce different hashes even when the decoded native pair matches. Hashes and source rechecks establish limited integrity at the check time, not freshness, authority or evidence sufficiency.

Use the [authored command guide](poam-cli-workflow.md) for preparing supported plans and the [source foundation guide](poam-foundation.md) for the source profile. Full F08, D064 closure disposition, evidence workflows, representative remediation pilots, independent OSCAL consumer qualification, accessibility and human acceptance remain separate gates. This guide records command behavior; it does not claim those gates, final runtime coverage or release readiness.

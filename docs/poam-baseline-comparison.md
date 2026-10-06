# POA&M baseline comparison

Compare two explicit authoring declarations with the current captured source
bundle. The report preserves added, removed and changed item/milestone identities,
including revisions that the ordinary native producer refuses.

```bash
forge poam baseline --manifest ./assessment-bundle/authored.json \
  --baseline prior.json --as-of 2026-02-06 --format json
forge poam baseline --manifest ./assessment-bundle/authored.json \
  --baseline prior.json --as-of 2026-02-06 \
  --format text --report baseline-review.txt
```

`--manifest`, `--baseline` and `--as-of` are required. There is no date default;
the date must use the ten-character `YYYY-MM-DD` calendar form. Invalid dates,
signed years and expanded years fail. `--baseline` and optional `--reopens` are
confined descendants of the current manifest's canonical directory. A report is
a new portable filename in that directory, with `.json` or `.txt` matching its
format. Without `--report`, the complete report goes to stdout.

Exit 0 means a complete comparison needs no review. Exit 1 means the complete
report contains changes, stale references or refused revisions that need review.
Exit 2 means malformed, unsafe, unbounded or otherwise invalid input/output;
invalid input produces no report. An exit-1 report can still contain a revision
that is compatible with the existing successor rules, such as a date or owner
change. Compatibility and review need are separate report fields.

## What the report establishes

`forge.poam-baseline/1` retains the complete previous/current item and milestone
union using the stable plan/item/milestone UUID protocol. Its separate counts cover
previous, current, common, added, removed and changed identities. Every common
row retains both projections and all applicable changed-field categories:
ownership bindings, target dates, titles, outcomes, state, history, rationale,
source tuples, milestone order and dependencies.

Party names, responsibility prose, item descriptions, titles and history rationale
are represented by digests. Exact keys, role/party IDs, dates, states and source
tuples remain visible and can be sensitive. The report is minimized and does not
promise anonymity or general secret redaction. Text output preserves the complete
JSON field structure while escaping Unicode and control characters to printable
ASCII. Complete output is bounded to 10 MiB; oversized reports fail rather than
returning a partial row set.

Both declarations pass the closed, duplicate-safe structural/history profile.
The comparison can inspect empty declarations, removals and terminal assertions
that ordinary `poam build/check --workflow` refuses. It records those refusals;
it never converts them into accepted remediation or removal authority.

The actual current five-file Assessment Results/context capture is validated.
Every complete prior and current source-selection tuple is tested against that
current inventory. A prior tuple that no longer matches current capture does not
prove that its historical source was invalid. The report's
`previous_validation` and `reference_basis` explicitly describe this limit;
the command does not reconstruct a historical source bundle.

Original current/prior declaration digests bind their complete file bytes,
including formatting. History-prefix checks compare the decoded ordered events.
Typed projection digests and original file digests describe different things.

The comparison uses each declaration's final asserted state and complete history.
The explicit as-of date records the comparison date; it does not reconstruct
historical state or trim events to that date.

`artifact_validated` is always false: this comparison creates no native POA&M.
`revision_rules_compatible` does not establish a new artifact's validity,
authenticated ownership, effectiveness, evidence sufficiency or approval.

## Explicit proposed reopening relationships

Optional `--reopens relations.json` accepts a closed JSON array of relationships.
Each member must contain exactly `previous_item_key`, `current_item_key` and
`previous_manifest_sha256`. Supply the SHA-256 of the complete original baseline
file bytes, an existing terminal prior item and a distinct new item key present in
the current declaration. Duplicate or contradictory relations fail. Similar prose
or source selections never infer a relationship.

Every reported relationship has `accepted_for_build: false` and requires review.
It does not reopen a source finding, authenticate an actor or enable a terminal
revision in `poam build/check`. The D064 closure/supersession disposition remains
required for accepted terminal workflows.

## Input and publication integrity

The existing 4 MiB declaration/relation raw bound, depth 64, 64 KiB string bound,
10,000-item bound and authoring history/milestone bounds apply. The complete
current source bundle retains its existing bounded source profile.

Output preflight refuses existing files and aliases of the current declaration,
baseline, reopening declarations or any of the five source files, including
case-folded filename collisions. Inputs are read through confined, single-link
capture and the actual source, file identities and complete original declaration
bytes are rechecked before stdout or no-replacement publication. This is a
bounded observation of those inputs; it is not an atomic filesystem snapshot.
File publication uses the existing Linux/macOS support. Unsupported platform
publication fails closed; no Windows result is inferred from local tests.

The [authored command guide](poam-cli-workflow.md) covers native production and
schedule reports. Full F08 still requires accepted closure/reopening semantics,
portfolio/HTML/outbound adapters, fresh evidence links, independent consumer
qualification, representative plans and real pilot judgments. The full goal-wide
documentation review and release gates remain open.

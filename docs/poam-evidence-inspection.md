# Inspect POA&M evidence locally

Use `forge poam evidence` to compare every declared closure-evidence assertion
with an explicit linkage binding and the current local files. The report keeps
byte matches, recorded freshness dates and unresolved bindings separate. A
matching hash does not establish authenticity, sufficiency, remediation
effectiveness or approval to close an item.

The command reads the workflow declaration, its actual Assessment Results
source and AP/SSP/Profile/Catalog companions, the linkage manifest and its
required native inputs, and declared local evidence. It does not fetch remote
evidence, publish a native POA&M artifact or change the plan or evidence files.
Use the [workflow guide](poam-cli-workflow.md) for the authoring declaration and
the [linkage guide](evidence-linkage.md) for linkage resources and evidence.

## Run an inspection

```bash
# Write the complete inspection JSON to standard output.
forge poam evidence \
  --manifest plans/remediation/plan.json \
  --links review/evidence-links.json \
  --as-of 2026-10-04

# Publish a new report beside plan.json; an existing file is never replaced.
forge poam evidence \
  --manifest plans/remediation/plan.json \
  --links review/evidence-links.json \
  --as-of 2026-10-04 \
  --report evidence-inspection.json
```

All three input flags are required. `--as-of` must be a real calendar date in
exact `YYYY-MM-DD` spelling; the command does not use the current clock. The
date controls freshness and marks assertions recorded after that date. It does
not remove future assertions or reconstruct an approved state at that date.
The expiry window comes from the linkage project's `expiring_window_days`;
there is no separate due-soon or terminal-override flag for this command.

`--links` is relative to the workflow manifest's directory. In the example it
means `plans/remediation/review/evidence-links.json`, regardless of the current
working directory. Absolute paths, `..`, ambiguous path spellings and unsafe
linked ancestry are refused. Every captured input must stay under the actual
qualified plan root.

The optional `--report` accepts one new portable filename ending in lowercase
`.json`, in that same plan directory. It cannot be a nested path, overwrite an
existing file or collide with any present input, missing-evidence observation
or declared evidence directory. New-file publication is supported on Linux and
macOS. On Windows, use standard output; file publication fails closed.

## Bind an assertion explicitly

Supply a closed `forge.poam-evidence-links/1` companion. Each binding names the
exact item, optional milestone, history event and closure-evidence assertion,
plus its linkage link and evidence keys. `milestone_key` is required and is
`null` for item-level history.

```json
{
  "schema_version": "forge.poam-evidence-links/1",
  "plan_key": "remediation-plan",
  "linkage": {
    "artifact": "linkage/review.json",
    "expected_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
    "project_key": "review-project"
  },
  "bindings": [
    {
      "locator": {
        "item_key": "item-1",
        "milestone_key": null,
        "event_key": "review-1",
        "assertion_evidence_key": "evidence-1"
      },
      "link_key": "control-review-1",
      "evidence_key": "evidence-1"
    }
  ]
}
```

This is an illustrative shape. Replace the all-zero hash with the complete
original linkage manifest's lowercase SHA-256 and use keys that actually exist
in your plan and linkage manifest. The example grants no authority and is not
a verified source fixture.

`linkage.artifact` is also relative to the plan directory, even when the binding
companion is nested. In this example it means
`plans/remediation/linkage/review.json`. Resources and evidence-root paths
inside that linkage manifest use the linkage manifest's own directory as their
base. Its declared local evidence paths then use the named evidence root.

The command requires exact locator and link membership, reference kind, local
path and hash relationships. It does not join an assertion to evidence merely
because a displayed href looks similar. Duplicate JSON members, unknown fields,
duplicate locators, bindings for nonexistent assertions and stale manifest
hashes are refused. An assertion with no binding remains an `unbound` row;
omitting a binding does not remove that assertion from the report. The linkage
manifest must contain at least one reviewed link.

## Read the complete result

The `forge.poam-evidence-inspection/1` JSON includes every closure-evidence
assertion in item and milestone histories, including future and terminal
events. It does not silently filter completed or cancelled declarations.

`match_status` describes the join: `matched`, `unbound`,
`wrong-link-membership`, `reference-kind-mismatch`, `href-mismatch`,
`hash-mismatch` or `unavailable`. Freshness is a separate field:

- `current`: local size and bytes match their approved values, with no expired
  or near-expiry recorded date. An absent `valid_through` date adds no expiry
  condition; it does not prove how recently the evidence was collected.
- `expiring`: the matching local evidence expires after `--as-of` but on or
  before the end of the linkage project's expiry window.
- `expired`: the recorded `valid_through` date is on or before `--as-of`.
- `changed`: the observed local hash or size differs from the approved values.
- `unavailable`: a safely observed local path is missing.
- `unverified-uri`: the evidence is a declared URI; no remote bytes were fetched.
- `null`: no freshness observation could be joined to that assertion.

A `matched` row may still be `expiring` or `expired`. Missing local bytes and
changed bytes take precedence over recorded dates. `local_bytes_revalidated`
only records an actual local-byte check; it supplies no authentication or
closure authority.

Summary counts cover the complete declaration and linkage inputs. Assertion
status counts and freshness counts each partition the assertion rows; they are
separate dimensions. `future_assertions` overlaps those rows. Repeated uses of
one evidence record still produce separate assertion rows. Distinct referenced
evidence, all captured local evidence and all declared linkage evidence are
different counts. `captured_original_generations` counts retained present files;
missing paths and directory proofs remain checked but are not present files.

Every terminal declaration requires review, even when every local join and date
matches. The report always keeps `terminal_admitted: false` and
`artifact_validated: false`. Inspecting a terminal history does not bypass the
existing native workflow refusal or resolve decision D064.

| Exit | Meaning |
| --- | --- |
| `0` | A complete valid inspection was written, with no reported review condition. |
| `1` | A complete valid inspection was written, but review is required: a terminal declaration, a non-matching assertion or freshness other than `current`. |
| `2` | Inputs, paths, originals, bounds or output could not be qualified. Treat this as a failed inspection; a failed stdout write can leave partial bytes. |

Exit `0` is a local inspection result, not acceptance of the evidence or closure.

## Local capture, bounds and privacy

The command retains the actual original bytes and file identities used for
inspection. Before publication it rechecks the complete source closure,
linkage inputs, all local evidence observations, the plan root and declared
evidence directories, including unused declarations. A safely missing local
path has an explicit absence proof; unreadable, unsafe, aliased or over-limit
files are not treated as missing. Same bytes at a replacement identity, a
changed missing path or changed ancestry cause refusal. These checks bind a
local observation; they do not lock out external editors after the final check.

The complete inspection has these limits:

| Bound | Limit |
| --- | --- |
| Workflow declaration and binding companion | 4 MiB each |
| Linkage manifest | 2 MiB |
| Each native or local evidence file | At most 50 MiB; a declared evidence limit may be lower |
| All retained original bytes across the inspection | 100 MiB |
| Present and absent physical input observations | 10,137 |
| Declared evidence directory proofs | 64 |
| Complete repeated inventory, reference, link and assertion relationships | 100,000 |
| Complete inspection JSON | 10 MiB |

The same 10 MiB limit also bounds intermediate context and linkage metadata
projections. Their conservative accounting and the shared relationship budget
can refuse an inspection before its final JSON reaches 10 MiB. These are whole
inspection limits, not allowances reset for each item. Parsed forms have their
own limits; this is not a promise about total process memory.

The report omits evidence contents, assessment and remediation prose, actor
contact details, raw URIs and absolute paths. It retains stable identifiers,
declared associations, hashes, sizes and dates so a reviewer can check the
result. Those values can still be sensitive metadata, and a digest is not a
secrecy mechanism. Protect the report according to your evidence-handling
policy. No network fetch, connector update or remote authorization is performed.

This inspection slice leaves the native terminal/evidence decision and human
review gates open. Slice verification results are recorded separately; the
goal-wide final documentation and acceptance gates remain open. See
[PRD 064](PRD/064-prd-oscal-poam-workflow.md) for the wider workflow scope.

See the [verification guide](poam-evidence-inspection-verification.md) for the
exact documentation census, measured coverage and remaining validation gaps.

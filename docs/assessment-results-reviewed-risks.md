# Reviewed-risk POA&M authoring

The `assessment results export-poam` command turns explicitly selected,
caller-reviewed risks and a complete authored workflow into a `forge.poam/1`
declaration. You supply every owner, date, milestone, actor, history event and work
description. The output can then be checked and built with the maintained POA&M
commands. The separate [sealed epoch append command](assessment-results-epochs.md)
delivers a bounded same-context multiple-epoch profile. Broader epoch workflows
and recorded owner, assessor, pilot and interoperability acceptance remain open.

## Prepare and consume the workflow

Start with the existing `forge poam init` command to create an empty scaffold
bound to your local Assessment Results, Assessment Plan, SSP, Profile and Catalog.
Retain those originals at their declared relative paths. See the
[foundation guide](poam-foundation.md) for the required source context and pins.

Inspect the current source inventory before authoring your selections:

```sh
forge poam check --manifest /absolute/bundle/empty-plan.json \
  --source-only --format json
```

For each selected risk row in `objects`, copy `kind`, `key`, `uuid` and
`result_uuid` into its `source_ref`, and copy that row's computed `sha256` into
`source_ref.expected_sha256`. The top-level `source_sha256` hashes the whole
Assessment Results file and is not the object digest. Neither
`declared_content_sha256` nor `declared_rationale_sha256` replaces the computed
object hash. A saved inventory is guidance for authoring, not capture authority:
the export checks the actual originals and exact selected tuples again.

Create a JSON request matching
[`forge.poam-risk-authoring/1`](../schemas/forge.poam-risk-authoring-1.schema.json).
It has exactly three top-level fields:

- `schema_version`: `forge.poam-risk-authoring/1`.
- `workflow`: the complete caller-authored `forge.poam/1` workflow described in
  the [item workflow guide](poam-item-workflow.md).
- `reviewed_risks`: an array of records, each containing exactly `source_ref`
  and `caller_asserted_reviewed: true`. Each `source_ref` contains the exact
  current risk's `kind`, `key`, `uuid`, `result_uuid` and `expected_sha256`.

Copy the scaffold's complete `source` field and immutable `document.key` into
the workflow. Supply the remaining document metadata yourself. Select risks by
their existing kind, key, UUID, result UUID and computed whole-object hash. Do not
substitute a rationale hash, selection position or a latest-result assumption.

Every work item and milestone in this first-plan profile must explicitly declare
`state: "planned"` and exactly one attributed history event with `from: null`,
`to: "planned"` and `closure: null`. Nullable fields are still required. Existing
generic workflows retain their separately documented transition rules.

The complete unique risk union across all work items must equal `reviewed_risks`.
The same current risk may appear in different items. Duplicate references within
an item and duplicate reviewed selections are invalid. An explicitly selected
closed source risk remains a valid source reference; selection makes no
remediation-eligibility judgment.

With the scaffold in your bundle directory and the request below that directory:

```sh
forge assessment results export-poam \
  --scaffold /absolute/bundle/empty-plan.json \
  --authoring requests/reviewed-risks.json \
  --as-of 2026-10-04 --output plan-workflow.json

forge poam check --manifest /absolute/bundle/plan-workflow.json \
  --workflow --as-of 2026-10-04 --format json
forge poam build --manifest /absolute/bundle/plan-workflow.json \
  --as-of 2026-10-04 --output native-plan.json --format json
```

The date is explicit, canonical and calendar-valid. `--authoring` is relative to
the scaffold's parent, including when your shell is elsewhere. `--output` is a
new single `.json` filename in that same parent. The publisher admits at most
128 bytes with ASCII letters, digits, dash, underscore and dot, and refuses
reserved device names. Existing files, relocated destinations, ambiguous path
spellings, unsafe ancestry and input aliases are refused.

Without `--output`, the export writes one complete JSON document plus a newline
to stdout. File publication supports Linux, macOS and Windows
([Windows file publication](authoring.md#windows-file-publication)); other platforms retain the existing fixed refusal. Stdout export is separately supported by the
source confinement implementation.

## Add new work to an existing plan

Use `--existing-plan` instead of `--scaffold` to add new work for further
reviewed risks to a plan you already maintain. This opt-in mode is add-only, as
recorded in the F10 section of the
[2026-10-25 owner dispositions](plans/2026-10-25-owner-dispositions.md). The two
options are mutually exclusive, and the empty-scaffold mode is unchanged.

```sh
forge assessment results export-poam \
  --existing-plan /absolute/bundle/plan-workflow.json \
  --authoring requests/new-reviewed-risks.json \
  --as-of 2026-10-05 --output plan-workflow-2.json

forge poam baseline --manifest /absolute/bundle/plan-workflow-2.json \
  --baseline plan-workflow.json --as-of 2026-10-05
```

The existing plan must be a valid, nonempty `forge.poam/1` workflow. An empty
scaffold is refused here; use `--scaffold` for it. The request uses the same
`forge.poam-risk-authoring/1` profile. Its `workflow` contains:

- the existing plan's exact `source`, `roles` and `parties`;
- the same `document.key`, with your new `title`, `version` and `last_modified`
  (it must not move backward, and new history must not follow it);
- only the new items. Each item and milestone is `planned`, with exactly one
  initial history event, as in the first-plan profile.

The output is the existing plan's values with your document metadata and the new
items appended after the existing ones. Existing items, milestones, history,
terminal assertions and `supersedes` links are retained exactly; Forge checks
this before output. New items get new stable identities from the existing
plan/item key scheme. Existing native UUIDs do not change.

The export refuses the whole request when:

- a new item key, or a derived item or milestone UUID, collides with an existing one;
- the request's source, roles, parties or plan key differ from the existing plan;
- a reviewed risk is already referenced by an existing item (matched by key or
  UUID), including a terminal or superseded item;
- the existing plan's sources are stale, or any existing selection no longer
  matches the current Assessment Results. Rebinding an existing plan to a new
  Assessment Results generation is not supported;
- a new item is not planned, has more than one history event, or declares
  `supersedes`. Reopening keeps its separate workflow.

The existing plan file is never rewritten. `--output` must be a new filename;
the existing plan and other inputs are refused as destinations. The M-13
baseline comparison of the output against the existing plan reports the new
items and milestones as `added`, with no changed or removed rows. It exits `1`
because additions need review. Whether re-exporting a risk that an existing item
already references should ever be allowed, and whether export may add new roles
or parties, are open owner questions; both are refused today.

## Validation and outcomes

The export checks the actual empty scaffold, complete source pins, all five native
originals, exact selections and current workflow rules. It consumes the native
POA&M renderer and official schema validation before returning an authored
declaration. Native single-line titles therefore must pass their native rules;
multiline work descriptions remain allowed where that schema permits them.
Decoded workflow values, ordered arrays and explicit nulls are preserved.
Whitespace and object-field order may change during serialization.

Raw scaffold and request, complete nested workflow and final output are each
bounded at four MiB. Complete repeated relationships and native projection share
finite admission limits, so a request within the raw byte limit can still be
refused for derived size. The command returns a fixed error and no successful
prefix for an invalid request.

Export exit 0 means a complete admitted declaration was emitted. Exit 2 means
invalid input or output failure. Subsequent POA&M check/build commands retain
their own exit 1 for a valid schedule needing action. An OS stdout fault may
leave a partial stream; a publication durability failure may leave a complete
new file. Either export failure returns exit 2. Inspect an existing destination
before retrying with a fresh filename.

The caller-review flag authenticates no reviewer and is consumed by the export,
without being added to the output workflow. Export seeds new `planned` work
only, so it still refuses terminal completion and risk acceptance. Those
assertions are made later in the manifest's append-only history, as the
[2026-10-25 owner dispositions](plans/2026-10-25-owner-dispositions.md) record.
Authored prose, party names, rationales and relative source paths may be
sensitive; the workflow is not a redacted report. Source records are unchanged.

This guide describes the current first-plan command and its source contract.
Slice measurements and omissions are recorded separately in
[reviewed-risk verification](assessment-results-reviewed-risks-verification.md)
and its [machine-readable audit](plans/2026-10-04-f10-risk-authoring-integration-verification.json).
Those records grant no owner, assessor, interoperability, pilot, platform or human
acceptance. PRD 063 S-3 and S-4 acceptance and the full goal-wide documentation
review remain open.

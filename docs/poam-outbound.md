# Local POA&M outbound change sets

`forge poam outbound` prepares one complete `forge.integration-change-set/1`
JSON handoff from an explicitly supplied current authoring/native pair and a closed
item selection. It implements the local export portion of PRD 064 S-3. Remote
planning, credentials, adapter translation, apply/reconcile, and PRD 065 acceptance
remain separate work.

## Prepare a request

Start with an authored `forge.poam/1` manifest and native POA&M JSON created by
the supported `forge poam build` workflow. Native and selection inputs must each
be a single `.json` filename beside that manifest. The native input must equal the
complete supported projection for the supplied authoring file, explicit date and
interval, and pass the existing offline OSCAL 1.2.3 POA&M schema. Unknown native
extensions and arbitrary imported POA&M documents are refused.

Write the complete item selection as JSON:

```json
{
  "schema_version": "forge.poam-outbound-selection/1",
  "operations": [
    { "item_key": "work", "intent": "create" }
  ]
}
```

Each selected key must exist in the current authoring manifest. Intent is exactly
`create`, `update`, or `close-request`. Choose it explicitly; declared item status
never selects an operation. One operation per item is allowed. Duplicate or
contradictory selections, unknown fields, empty selections, duplicate JSON object
keys, trailing JSON values, and unsupported intent are refused before native
capture. Omitted items are omitted from the handoff rather than silently selected.

The following illustrative command assumes that the bundle, selection and existing
output directory have already been prepared; it is not a fixture installation:

```bash
forge poam outbound --manifest ./assessment-bundle/authored.json \
  --native native-poam.json --selection outbound-selection.json \
  --as-of 2026-02-06 --due-soon-days 7 --output-root ./outbound-reports \
  --report change-set.json
```

`--as-of` is required and accepts a canonical four-digit Gregorian full date.
`--due-soon-days` accepts 0–365 and defaults to 0. Use the same inputs that produced
the native artifact; this command does not recreate a historical source generation.
The output directory is required and must already exist. Omitting `--report`
writes the complete JSON, including its final newline, to stdout. With `--report`,
the command publishes one new file and leaves stdout empty.

Exit 0 means a valid local handoff was emitted, including when the workflow's
schedule is overdue or blocked. Exit 2 means invalid input or output failure.
This command does not use exit 1 as a schedule gate. For the schedule action gate,
use the authored workflow check/build commands.

## Original files and output safety

The CLI holds literal bytes and actual file identities for eight originals: the
authoring manifest, native POA&M, selection, Assessment Results, Assessment Plan,
SSP, Profile and Catalog. It uses one resolved bundle directory for authoring,
native and selection capture. All eight are rechecked immediately before emission;
same decoded JSON, a recorded hash, or a same-byte replacement is insufficient.
Rechecks describe that instant and do not prevent subsequent edits by another
process. The public prepared-output API retains seven originals; callers must
independently hold/recheck the selection original as the CLI does.

Each requested report name must satisfy the existing portable publisher profile:
one ASCII filename, at most 128 bytes, a lowercase `.json` suffix, and no reserved
device name or path syntax. The existing directory and filename are preflighted
before output. Complete absolute path components are compared conservatively for
ASCII-case aliases against all eight held inputs, preserving distinct output
directories rather than comparing filenames alone. Existing outputs are refused.
No input file is changed.

File publication uses the existing qualified no-replacement writer on Linux and
macOS. Other platforms refuse file publication with exit 2 and no created report;
stdout remains the portable output route. A write failure may leave a partial
stdout stream; success is reported only after the complete write succeeds.

## Contents and identity

Operations are sorted by exact item key and include a generic UUID-based title,
one fixed intent reason code, sorted declared role/party IDs, authored target date
and final declared workflow state. Declared state is not state at `--as-of`.
Assessment prose, item descriptions, names/contact details, rationale, milestone
outcomes, evidence bytes and private file paths are omitted. IDs, dates and hashes
remain potentially sensitive metadata; minimization does not establish anonymity.
Debug formatting of the sealed prepared object reveals only its report byte count.

The stable operation key combines plan UUID, item UUID and explicit intent.
Changing a target date or source file bytes changes the corresponding payload or
generation digest without changing that operation key. A changed intent has a
different key. The payload digest hashes compact typed fields; native, authoring
and selection digests hash their literal original bytes. Native whitespace and
object ordering may vary if the complete decoded value still matches, while the
literal-byte source digest changes.

The compiled-in closed [change-set schema](../schemas/forge_integration_change_set_schema.json)
and consumed validator check complete output shape, counts, operation/item
uniqueness, stable-key construction, generic titles, intent reason codes and
payload digests. Schema validation cannot create a real captured input proof.
Every operation has `remote_preconditions: null`, and the complete handoff has
`remote_apply_authorized: false`. No remote object/version has been observed.
`close-request` neither completes native work nor closes an Assessment Results
finding. Completion/risk acceptance remain refused under the existing D064 gate.
Declared owners are preserved IDs, not authenticated actors or approved remote
assignments.

## Complete bounds and remaining gates

Selection is capped at 4 MiB, JSON nesting at 64 and selection strings at 64 KiB;
item keys are at most 256 UTF-8 bytes and at most 10,000 distinct operations are
allowed. Existing workflow/source admission applies, including at most 64 owners
per item. Native originals and complete change-set JSON are each capped at 10 MiB.
These are input/output bounds rather than a measured process heap bound. A failed
bound returns an error instead of a successful truncated prefix.

Synthetic real-file controls and source reviews establish only the exercised
implementation behavior. Independent OSCAL consumers, representative remediation
plans and team pilots, fresh PRD 060 completion evidence, D064 closure decisions,
and governed remote connectors remain open F08/F14/F21 acceptance work. The
goal-wide final documentation review remains required after the remaining work.

# Sealed assessment epochs and explicit risk continuity

This command appends one sealed result epoch to an existing FORGE
Assessment Results artifact. It preserves every existing result and records
caller-declared risk families in a separate, complete JSON companion. Continued
risks also carry their family key and predecessor UUID in the native file.
Assessment and continuity assertions do not authenticate an assessor, evaluate
evidence, or authorize remediation.

```sh
forge assessment results append-epoch \
  --request epoch-request.json \
  --output next-assessment-results.json \
  --report next-epoch-report.json
```

Both output arguments are new single portable ASCII `.json` filenames, at most
128 bytes, relative to the request file's actual parent. The command reserves
both names before capturing inputs and refuses existing destinations. Request
path spelling must already be normalized; spell `epoch-request.json` rather
than `./epoch-request.json`. Companion files may have normalized descendant
paths, but the prior Assessment Results file and optional prior report use
single JSON filenames on this same parent. Receipt locations are preserved.

The command defaults to a review gate: a valid append returns status **1** because
it adds review actions. `--fail-on never` returns **0** for the same valid
artifacts. Invalid input or an output failure returns **2**. This policy does not
change the report's assertions or authorize their acceptance.

## Author the complete request

The closed `forge.assessment-epoch-append/1` request requires six fields and
accepts one optional field:

| Field | Required content |
| --- | --- |
| `schema_version` | `forge.assessment-epoch-append/1` |
| `prior_source` | The complete existing POA&M source declaration: native Assessment Results pin, explicit result selector, and the four companion pins with `evidence_index: null` |
| `next_epoch` | A complete six-field `forge.assessment-results/1` manifest, using the same document key, title and context. It repeats every earlier role and party unchanged and may add new ones |
| `prior_report` | Explicit `null` for the first append, or `{artifact, expected_sha256}` for the actual prior JSON companion |
| `seed_families` | Every existing risk classified once for the first append; an empty array for later appends |
| `new_risks` | Every next risk classified once as a new family or an explicit continuation |
| `prior_observation_links` | Optional. Each row links one new finding to one observation from an earlier epoch. Omitting the field is the same as an empty array |

The optional field was added without a version change: every request that was
valid before stays valid.

The exact request and companion fields are specified in the closed
[append schema](../schemas/forge.assessment-epoch-append-1.schema.json) and
[report schema](../schemas/forge.assessment-epoch-report-1.schema.json).

The next manifest uses the existing [Assessment Results authoring
contract](assessment-results.md), with its optional fields explicitly present.
Nullable fields use `null`; empty optional arrays use `[]`. This profile requires
a non-null result end and excludes evidence indices and linked evidence.

Every old and new epoch is sealed with a start and end. Epoch order is explicit:
the previous end must be at or before the next start. Conclusion and continuity
provenance must fit their associated epoch, and the new document modification
instant must be at or after the new end and the old modification instant. Exact
RFC 3339 instants govern these comparisons; touching windows are allowed.

The existing typed authoring model constructs the new result. Its result key and
all new conclusion `(kind, key)` identities and UUIDs must be distinct from the
complete historical set. The caller supplies the existing document key; the command does not recover
it from a native UUID or rename reused keys. In the preserved native envelope, only
the explicitly supplied document version and modification time can change, and
only new roles and parties can be added.

## Add roles and parties

An appended epoch may add metadata roles and parties, for example a second
assessor. List every earlier role and party in `next_epoch` exactly as before,
then add the new ones. The command refuses the append if an earlier role or
party is changed or left out. In the output, the earlier roles and parties keep
their bytes and positions, and the new ones follow them. A new party's UUID must
not match any earlier native UUID. New conclusions and continuity rows may name a
new assessor and role.

The report records each added role as a `role-added` change (its role ID) and
each added party as a `party-added` change (its party UUID). Party names are not
copied into the report. Back-matter receipts must stay unchanged: this profile
does not add evidence receipts (see [Outside this profile](#outside-this-profile)).

## Declare every risk family

A seed contains `family_key`, a complete `risk` source reference,
`caller_asserted_continuity: true`, and complete assessment provenance. The
reference contains `kind: "risk"`, the exact key, object UUID, result UUID and
computed whole-object SHA-256. A declared content property is a different hash
and cannot substitute for this reference.

A next-risk classification contains `next_key`, `family_key`, `prior`,
`caller_asserted_continuity: true`, and complete provenance. Use `prior: null`
only for an explicitly new family. A continuation names the actual latest
historical member of that family through its full source reference. Each family
can have at most one successor in an append. Branches, merges, cycles, future or
same-epoch predecessors, and reused native identities are refused.

### Native continuity properties

Native UUIDs are never reused across epochs. Each continued risk in the new
epoch carries two Forge-namespaced properties
(`https://policy-forge.github.io/ns/assessment-results`):

| Property | Value |
| --- | --- |
| `continuity-family-key` | The exact caller family key |
| `continuity-predecessor-uuid` | The UUID of the family's previous risk |

A tool that reads only the native file can follow these properties back to the
family's first member. A first member, whether seeded or newly created, carries
neither property. The output stays valid against the OSCAL 1.2.3 schema, so
every family key, including a new or seed key, must be a valid OSCAL property
value: no leading or trailing whitespace and no line breaks. This ensures a
new family can be continued later.

The POA&M source loader checks these properties in every result. Both must
appear together. The predecessor must be a risk in an earlier result, can have
only one successor, and must be the latest member of the same family. The
declared `content-sha256` of the risk does not change; its computed whole-object
digest includes the properties. A later append also checks that each stored
property pair matches its edge in the prior report. Risks appended before these
properties existed may lack them.

Every older family remains in the report, including families that receive no
successor. An uncontinued family is not a declaration that its risk was resolved.
Each edge has two reciprocal rows and a framed content digest binding its family,
endpoints and declared provenance. This digest is not a signature or an approval.

## Link new findings to earlier observations

A new finding may reference an observation from an earlier epoch. Add one
`prior_observation_links` row for each link:

```json
{
  "finding_key": "finding-next",
  "observation": {
    "kind": "observation",
    "key": "synthetic-observation",
    "uuid": "<earlier observation UUID>",
    "result_uuid": "<earlier result UUID>",
    "expected_sha256": "<computed whole-object SHA-256>"
  }
}
```

The finding key must name one new finding. The observation reference must match
an observation in the prior document exactly, so a link can only point backward.
A link with kind `finding` or `risk` is refused, and so is a `next_epoch`
relationship that names an earlier finding or risk. Each new finding still needs
at least one relationship to an observation in its own epoch. The command adds the
earlier observation UUID to the finding's `related-observations` and records a
`prior-observation-linked` change with the finding's locator. The POA&M source
loader accepts an observation reference to the same or an earlier result and
refuses a forward one.

## Consume the prior report

For a later append, pin the actual prior JSON report and supply no new seeds. The
command checks its all-epoch identities, counts, context, family membership and
reciprocal history against the captured current native artifact. Stored historical
hashes do not replace the capture of current original generations. The report
does not recapture the generation that preceded the current native file: its
historical raw hash and optional old metadata leaves remain stored caller
assertions. The `role-added` and `party-added` rows of a prior report are also
stored assertions, because the generation before them is not captured; each must
still name a current role or party exactly once. Backward links are rebuilt from
the native file and must match the prior report exactly. The current native file,
all four companions, request and any prior report are captured and rechecked for
this append.

## Inspect the complete companion

The required report file always uses `forge.assessment-epoch-report/1` JSON. It
contains every epoch and observation, finding and risk; complete before and after
counts; full object locators; computed and declared hashes; every family, edge,
reciprocal row and descriptive change; four context identity rows; and the count
of original generations captured for this append. It omits native conclusion prose
and continuity rationales, retaining rationale digests.

To display the same complete report after both files are published, select one
optional stdout view:

```sh
forge assessment results append-epoch \
  --request epoch-request.json \
  --output next-assessment-results.json \
  --report next-epoch-report.json \
  --view-format html \
  --fail-on never
```

`--view-format` accepts `json`, `text` or `html`. Without it, stdout is empty. JSON
stdout equals the durable JSON companion. Text represents dynamic strings with
visible escaping; HTML is static and escapes every dynamic field, with no active
links or external resources. Text and HTML views cannot serve as a later prior
report. View selection does not change the two saved formats or add a third file.

## Bounds and publication

The request is limited to 4 MiB, a prior native artifact to 50 MiB, and a prior
report to 10 MiB. The single capture pool has the existing 100 MiB raw-original,
10,137-observation and 100,000 repeated-relationship limits. The complete output
has at most 1,000 epochs and 10,000 conclusions across all epochs and kinds.
Decoded strings have a 64 KiB UTF8 limit. Family keys are exact nonempty strings
of at most 256 UTF8 bytes; whitespace is preserved without trimming.

The complete next native and report projections are admitted before their
retention. The unchanged 10 MiB projection profile includes both the durable JSON
and any selected stdout view, even when JSON shares its backing buffer. A view can
therefore exceed the aggregate profile when the same no-view append fits. Outputs
are refused as a whole when they cannot fit; rows are not truncated. These are
encoded representation bounds, not a heap limit. Captured context and borrowed
continuity membership registries are established earlier under separate complete
cardinality and relationship bounds; the encoded ceiling covers subsequent typed
next-result, native and report projections and delivery buffers.

The actual captured originals are rechecked before native publication, before
report publication, and before any selected stdout byte. Files use the existing
confined no-replace publisher. Publication of the two files is independent: a late
failure can leave the first complete file, both complete files, or partial stdout.
Published files are not rolled back. An existing destination is never overwritten.
The current publisher supports Linux and macOS; a Windows publication attempt
returns an error.

## Consume an explicitly selected epoch

Existing `poam init` and source checks can select an old nonlatest epoch or the new
epoch through its explicit result UUID and key. The new native file requires a
new raw pin even when an old conclusion's complete object tuple is preserved.
Callers must explicitly rebind a source declaration to that new generation; this
command does not mutate an existing plan. Reviewed-risk export and workflow build
retain their existing validation and human-disposition boundaries.

The existing one-result `assessment results build --baseline` interface continues
to refuse multiple result epochs. The new companion supplies the separate
all-epoch comparison; it does not change that baseline interface into a newest-result
selector.

## Outside this profile

This is a same-context sealed profile and a partial PRD 063 S-4 implementation.
The 2026-10-25 [owner dispositions](plans/2026-10-25-owner-dispositions.md)
(F10) approved additive actors, native continuity properties and backward
observation references, which this profile now implements. These remain outside
the profile and are refused:

- A different Assessment Plan or context generation. Start a new Assessment
  Results document, because OSCAL Assessment Results has a single `import-ap`.
- Changed or removed earlier roles, parties or back-matter receipts.
- Added evidence receipts. The record approves them, but the POA&M source
  foundation this command reads does not admit an evidence index. How to
  represent a changed evidence index is an open owner question.
- Open or overlapping windows.
- Reused native identities, and branching or merging families.
- References to earlier findings or risks, and forward references.

Representative assessor workflows, independent interoperability and recorded
owner acceptance remain separate completion gates.

Measured engineering checks and their coverage limits are recorded in the
[verification receipt](plans/2026-10-04-f10-epochs-integration-verification.md) and
[complete metadata projection](plans/2026-10-04-f10-epochs-integration-verification.json).
The later Windows fixture correction has its own [fresh measured successor](plans/2026-10-04-f10-epochs-windows-fixture-verification-v1.md)
and [complete successor projection](plans/2026-10-04-f10-epochs-windows-fixture-verification-v1.json);
the initial receipts remain unchanged.
Append validates native output against the vendored OSCAL 1.2.3 Assessment Results
schema; generic public `forge validate` does not currently accept this model.

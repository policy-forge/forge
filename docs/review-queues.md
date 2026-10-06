# Portable review queues

> Integration update — October 6, 2026: the applicable workspace, MCP, assessment
> and review implementations are now on main. See the [merge reconciliation](plans/2026-10-06-merge-documentation-reconciliation.md)
> for exact commits and qualification scopes. Earlier draft states and failed
> receipts below retain their dated meanings; human and release gates remain separate.

This guide describes the F12 implementation candidate. Its current CLI supports
re-review of selected OSCAL Mapping assertions and explicit applicability
decisions in recorded Approved/current closures. Team evaluation, reviewer
authority, interoperability and final acceptance remain open.

Reviewer keys, roles, author keys and times are explicit assertions. They are not
authenticated or signed identities. `quorum-met` means that the declared review
policy is satisfied; it does not approve or modify a Mapping, applicability
decision, lifecycle record or other domain artifact.

## Lifecycle review exchange candidate

[Lifecycle re-review](lifecycle-review-exchange.md) uses a separately closed `/2`
family. Its implementation candidate registers `forge review lifecycle
init/respond/merge/status`; the linked guide records open-draft integration and
remaining acceptance gates. It does not extend the `/1` Mapping/applicability inputs or the
seven commands documented below. Currentness depends on the complete actual
native source closure; asserted review quorum grants no Lifecycle transition or
reviewer authority.

## Authoring review exchange candidate

[Whole saved-plan Authoring review](authoring-review-exchange.md) uses a separate
`/3` family with `forge review authoring init/respond/merge/status`. Its native
receiver regenerates the complete applicability baseline and saved
`forge.authoring-plan/1`; it does not require an Approved Lifecycle record or
change authoring state. A completed compiled CLI campaign exercised all four
routes in the TEMP candidate; the linked guide records full-test, strict-lint,
Rustdoc and physical LLVM scopes. Its dated draft-source checkpoint records
integration separately; final hosted/platform/human acceptance remains open.
This candidate leaves the seven `/1` commands below and
Lifecycle `/2` unchanged.

## Prepare the private inputs

Use an explicit absolute project root. Every input and output file path is a
normalized descendant relative to that root. Parent directories must exist.
Outputs are new files; an existing destination is refused. The current publisher
supports Linux and macOS and refuses publication on Windows.

The source locator is a private routing file, separate from the portable queue.
Its closed format selects either Mapping or applicability; a mixed-domain queue
is rejected. For Mapping, it lists the complete physical source set, including
the Mapping manifest,
lifecycle record, generated Mapping, source/target resources, resolved Catalog
companions for Profiles, and every additional lifecycle-generated artifact.
The factory validates that exact union and rebuilds the native Mapping through
the maintained producer. Schema validity or a declared hash alone is insufficient.
The lifecycle record must contain a current, complete recorded Approved tuple.

This example describes a Catalog-to-Catalog layout. Replace paths and keys with
the actual complete project roster; add required generated artifacts/companions,
and keep `sources` sorted by key. It does not supply an approval record:

```json
{
  "schema_version": "forge.review-source-locator/1",
  "mapping_manifest_key": "manifest",
  "lifecycle_record_key": "lifecycle",
  "mapping_key": "mapping",
  "sources": [
    {"key": "lifecycle", "path": "lifecycle.json", "model": "lifecycle-record"},
    {"key": "manifest", "path": "manifest.json", "model": "mapping-manifest"},
    {"key": "mapping", "path": "mapping.json", "model": "mapping"},
    {"key": "source", "path": "source.json", "model": "catalog"},
    {"key": "target", "path": "target.json", "model": "catalog"}
  ]
}
```

For applicability, use `forge.review-applicability-locator/1`. The example below
shows a Catalog framework and one Mapping; replace it with the complete actual
project roster. `mapping_keys` must follow the applicability manifest's Mapping
path order. Keep `sources` sorted by key, and include all native resources,
resolved Catalog companions and additional lifecycle-generated artifacts.
Every source row requires `resolved_catalog_key`: the key of a source row with
model `resolved-catalog` for a Profile, and explicit `null` for every other model.

```json
{
  "schema_version": "forge.review-applicability-locator/1",
  "applicability_manifest_key": "manifest",
  "applicability_report_key": "report",
  "lifecycle_record_key": "lifecycle",
  "framework_key": "framework",
  "mapping_keys": ["mapping"],
  "sources": [
    {"key": "framework", "path": "framework.json", "model": "catalog", "resolved_catalog_key": null},
    {"key": "lifecycle", "path": "lifecycle.json", "model": "lifecycle-record", "resolved_catalog_key": null},
    {"key": "manifest", "path": "applicability.json", "model": "applicability-manifest", "resolved_catalog_key": null},
    {"key": "mapping", "path": "mapping.json", "model": "mapping", "resolved_catalog_key": null},
    {"key": "report", "path": "applicability-report.json", "model": "applicability-report", "resolved_catalog_key": null},
    {"key": "source", "path": "source.json", "model": "catalog", "resolved_catalog_key": null}
  ]
}
```

The applicability lifecycle source must be the exact full, unfiltered
`forge.applicability-report/1` original, recorded as a non-native source without
an OSCAL UUID. The factory checks the current Approved tuple and reconstructs
the complete report through the maintained applicability engine. Filtered,
copied or stale reports cannot substitute for that closure.

Applicability items select exact control IDs with explicit manifest decisions.
Omitted decisions cannot be selected; they remain in the complete framework
denominator as under-review controls. All six category counts, the full native
Mapping count and Cartesian pair count remain complete. Native positive and
no-relationship edge-target counts are distinct from Cartesian pair counts.
Selected subject hashes bind the entire original decision and current framework
tuple; private rationale and paths are not copied into default queue context.

The private policy file contains declarations only. Replace the example
`subject_id` with an exact canonical map UUID from the captured native Mapping,
or an exact control ID for an explicit captured applicability decision. Mapping
IDs are canonical 36-character UUIDs; applicability IDs are bounded tokens up
to 256 bytes. Every item requires an explicit nullable `due_at`; omission is
rejected. Keep declared roles, reviewers, policies, items and nested key lists
sorted and unique.

```json
{
  "schema_version": "forge.review-init/1",
  "roles": [{"key": "reviewer"}],
  "reviewers": [
    {"key": "alice", "role_keys": ["reviewer"]},
    {"key": "author", "role_keys": ["reviewer"]},
    {"key": "bob", "role_keys": ["reviewer"]}
  ],
  "policies": [{
    "key": "two-reviewers",
    "seats": [{"role_key": "reviewer", "count": 2}],
    "substitutions": [],
    "abstention_rule": "nonapproving",
    "empty_abstention_reasons": ["recused"],
    "author_separation": "declared-keys"
  }],
  "items": [{
    "key": "one",
    "subject_id": "11111111-1111-4111-8111-111111111111",
    "policy_key": "two-reviewers",
    "author_keys": ["author"],
    "assignments": [
      {"reviewer_key": "alice", "role_key": "reviewer"},
      {"reviewer_key": "bob", "role_key": "reviewer"}
    ],
    "due_at": null
  }]
}
```

Source pins, subject/context hashes and item IDs are derived from actual captured
facts. The policy file cannot provide them or a success proof.

## Create, respond and collect

The following commands assume `private/` and `review/` already exist under the
project root. UUIDs and times are illustrative explicit values.

```sh
forge review init --project-root /absolute/project \
  --sources private/sources.json --policy private/policy.json \
  --queue-id 22222222-2222-4222-8222-222222222222 \
  --created-at 2026-10-04T12:00:00Z --output review/queue.json

forge review respond --project-root /absolute/project \
  --queue review/queue.json --item-key one \
  --reviewer-key alice --reviewer-role reviewer --disposition approve \
  --responded-at 2026-10-04T12:01:00Z \
  --response-id 33333333-3333-4333-8333-333333333333 \
  --rationale-file private/alice-rationale.txt --output review/alice.json

forge review respond --project-root /absolute/project \
  --queue review/queue.json --item-key one \
  --reviewer-key bob --reviewer-role reviewer --disposition approve \
  --responded-at 2026-10-04T12:02:00Z \
  --response-id 44444444-4444-4444-8444-444444444444 \
  --rationale-file private/bob-rationale.txt --output review/bob.json

forge review merge --project-root /absolute/project \
  --sources private/sources.json --queue review/queue.json \
  --response review/alice.json --response review/bob.json \
  --as-of 2026-10-04T12:03:00Z --output review/dispositions.json

forge review status --project-root /absolute/project \
  --sources private/sources.json --queue review/queue.json \
  --response review/alice.json --response review/bob.json \
  --as-of 2026-10-04T12:03:00Z

forge review export-html --project-root /absolute/project \
  --dispositions review/dispositions.json --output review/dispositions.html
```

Use `--response` once for every supplied occurrence; directory discovery is not
implicit. Exact duplicate bytes are counted separately from distinct responses.
The collector retains rejection, changes requested, abstention, conflicts and
superseded history. Each asserted key fills at most one approval seat, and
declared authors are excluded. Abstention never reduces the required seats.
Responses at or after an item's due time are late; a timely completed quorum
remains satisfied when evaluated later.

`respond` binds an immutable private response to the exact queue original and
checks its declared eligibility. `merge` and `status` recapture the complete
source set and queue/response cohort, evaluate the explicit `--as-of`, validate
the complete serialized record against the queue and recheck held originals
before output. Reformatting a queue changes its raw identity. Source drift or
missing native closure evidence refuses current output.

Self-supersession requires both `--supersedes-id` and `--supersedes-sha256` for the
exact prior original. It cannot erase a different asserted reviewer's dissent.
For an empty abstention rationale, use a reason explicitly permitted by the
policy with `--abstention-reason`; an ordinary empty rationale is rejected.

## Offline notification export

Create a complete local notification artifact from an explicitly named recorded
queue:

```bash
forge review export-notifications --project-root /absolute/project \
  --queue review/queue.json --output review/notifications.json
```

The closed `forge.review-notifications/1` artifact has one `request-review` row
for each explicit reviewer/role assignment and one `assign-reviewer` row with
null recipient fields for each unassigned item. It preserves complete queue,
assignment and recorded source-pin counts. Authors, substitutes and other roster
members are not inferred as recipients.

The export captures the exact queue bytes and checks those held originals again
before publishing a new file. It reads no responses or native source closure,
evaluates no current quorum or deadline state, and sends nothing. Its fixed
labels are `captured-queue-recorded-snapshot-only` and `local-export-only`.
Reviewer keys, roles, source pins and times remain recorded assertions. Only the
queue original is freshly captured; the embedded native pins remain metadata.

The versioned notification key binds the queue ID, derived item ID, intention
and nullable recipient key/role. Reformatting a queue changes its recorded raw
hash while preserving notice keys when its typed items remain the same. Semantic
changes to an item, its policy or deadline change its derived item ID and thus
its notice keys. This key supplies no recipient authentication or remote retry
receipt.

The field whitelist omits dedicated source paths, URLs, rationale, excerpts and
related-subject context. Exact identifiers and schema/version labels can still
contain sensitive author-selected metadata; inspect the artifact before sharing
it. The complete export is bounded to 100,000 rows and 10 MiB including its final
newline, within the same shared invocation ledger. Exhaustion refuses the whole
export. Existing output files are refused. Publication uses native
no-replace rename on Linux and macOS; the qualification for this slice is local
macOS. Windows publication refuses.

## Link a queue revision after change impact

Create the new queue with `forge review init` against its complete current native
source closure and recorded Approved lifecycle tuple. Preserve the complete old
queue original and the historical framework and Mapping/applicability originals
needed by the declared Impact manifest. A supersession companion records explicit
lineage between those queues; it transfers no responses, quorum or domain approval.

The private `forge.review-queue-impact-locator/1` file names the native Impact
manifest and the complete stored current Impact report. Replace the source keys
below with the exact framework keys in each respective queue. For Profiles,
provide both exact resolved-Catalog source keys. For Catalogs, retain explicit
`null`; omitting either nullable field is rejected.

```json
{
  "schema_version": "forge.review-queue-impact-locator/1",
  "manifest_path": "private/impact.json",
  "report_path": "private/impact-report.json",
  "old_framework_source_key": "framework",
  "new_framework_source_key": "framework",
  "old_resolved_source_key": null,
  "new_resolved_source_key": null
}
```

The complete Impact reader follows the maintained manifests' declared resources,
including configured applicability, successor and paired prior-report/disposition
inputs. It reconstructs the full unfiltered native report and compares every
stored field, including private version, href and history data, before producing
the minimized companion. Historical policy hrefs, Profile imports and old Mapping
producer manifests are not file routes for this reader. The new native source
locator still requires its complete normal current closure.

Supply a closed `forge.review-queue-links-request/1` request with explicit old/new item
IDs and current native finding IDs. Replace the example UUIDs with actual IDs
from the two queues and full current report. Sort links by `(old_item_id,
new_item_id)` and finding IDs within each link; reject duplicate pairs or IDs.
An empty `finding_ids` array is permitted. No correspondence is inferred.

```json
{
  "schema_version": "forge.review-queue-links-request/1",
  "links": [{
    "old_item_id": "11111111-1111-4111-8111-111111111111",
    "new_item_id": "22222222-2222-4222-8222-222222222222",
    "finding_ids": []
  }]
}
```

```sh
forge review supersede --project-root /absolute/project \
  --old-queue review/old-queue.json --new-queue review/new-queue.json \
  --sources private/new-sources.json --impact private/impact-locator.json \
  --links private/links.json \
  --supersession-id 55555555-5555-4555-8555-555555555555 \
  --created-at 2026-10-05T12:00:00Z --output review/supersession.json
```

Each explicit link must use the same supported domain on both endpoints. Explicit many-to-many links
preserve distinct endpoint counts separately from edge and finding-occurrence
counts. Referenced findings must exist in the full native report and match the
captured old dependency and declared old source pin. All unreferenced findings
and unmatched queue items remain represented. The asserted companion time must
be no earlier than either queue's declared creation time.

The closed `forge.review-queue-supersession/1` output contains exact whole-queue
raw and source pins, eleven fixed endpoint fields, typed differences, associated
native finding references, full native denominators and all fifteen summary
counts. Native Impact source pins preserve every declared read occurrence in
order, with synthetic keys `impact:<role>:<ordinal>`. Default output omits private
paths, hrefs, prose, rationale and document versions. Exact identifiers and hashes
can still disclose sensitive project metadata; inspect the artifact before sharing.

`no-detected-native-change` requires all four substantive change counters to be
zero and the complete native finding list to be empty. The total change count
includes unchanged controls. Loaded recorded labels remain inert data: the old
queue is `historical-unverified`, and its companion's recorded new-current label
does not replace a fresh native check. The command keeps both queues and all
originals held through encoding, semantic output validation and the final native
publication fence. Existing output files are refused.

The command shares the ordinary cooperative deadline and derived/work ledger.
It admits two whole queues of at most 10 MiB each, three private auxiliaries of at
most 1 MiB each, and the same complete 100-registration/50 MiB source pool. Links
are bounded to 10,000 edges and 100,000 finding occurrences; the final compact
JSON plus newline is at most 32 MiB. Near-limit inputs may exhaust the shared
budget and refuse the whole result. Publication uses the existing Linux/macOS
no-replace publisher; Windows publication refuses.

## Sharing and limits

Queue, response, dispositions, notification and supersession formats are closed and versioned
by their `schemas/forge.review-*-1.schema.json` files. Private responses retain rationale.
Default queue context and dispositions/HTML omit response rationale and source
excerpts. Identifiers, schema labels and hashes can still reveal sensitive
information; review the actual artifacts before sharing them.

HTML is a static rendering of a captured recorded bundle. It does not perform a
fresh source check or authenticate its recorded labels. The current CLI performs
no remote upload, notification delivery or automatic domain edit. Non-null proposed edits
are rejected. Supported proposed edits and remaining native adapters are
pending. The interactive client and signed response design/envelope remain open
requirements. Supersession currently covers the Mapping and applicability domains;
remaining domain adapters and acceptance gates stay open. Local notification export is implemented;
its connector/delivery integration and acceptance gates remain open.

The command shares one cooperative 30-second deadline and one monotonic ledger.
Its logical derived allowance is 32 MiB, with separately bounded work and raw
input pools: 100 source registrations/50 MiB total, 10 MiB per source, 10 MiB
queue, 10,000 response occurrences/32 MiB total and 1 MiB per response. Private
rationale is at most 8 KiB. Encoded dispositions and HTML are each at most 32 MiB.
These ceilings do not guarantee that every near-limit combination fits the
shared derived/work budget. Exhaustion refuses the whole result.

The deadline is cooperative; parser/schema/native calls are not forcibly
preempted. A stdout failure can leave a partial stream. A late durability failure
can leave the complete newly published file. Treat reported failures explicitly
and inspect the destination before retrying with a new name.

This focused guide does not close the final roadmap-wide documentation review or
the outstanding platform, privacy/accessibility, participant and release gates.

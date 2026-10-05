# Portable review queues

This guide describes the F12 implementation candidate. Its current CLI supports
re-review of selected assertions in a recorded Approved/current OSCAL Mapping
closure. The applicability adapter is being implemented. Team evaluation,
reviewer authority, interoperability and final acceptance remain open.

Reviewer keys, roles, author keys and times are explicit assertions. They are not
authenticated or signed identities. `quorum-met` means that the declared review
policy is satisfied; it does not approve or modify a Mapping, applicability
decision, lifecycle record or other domain artifact.

## Prepare the private inputs

Use an explicit absolute project root. Every input and output file path is a
normalized descendant relative to that root. Parent directories must exist.
Outputs are new files; an existing destination is refused. The current publisher
supports Linux and macOS and refuses publication on Windows.

The source locator is a private routing file, separate from the portable queue.
It lists the complete physical source set, including the Mapping manifest,
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

The private policy file contains declarations only. Replace the example
`subject_id` with an exact map UUID from the captured native Mapping. Every item
requires an explicit nullable `due_at`; omission is rejected. Keep declared
roles, reviewers, policies, items and nested key lists sorted and unique.

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

## Sharing and limits

Queue, response and dispositions formats are closed and versioned by the three
`schemas/forge.review-*-1.schema.json` files. Private responses retain rationale.
Default queue context and dispositions/HTML omit response rationale and source
excerpts. Identifiers, schema labels and hashes can still reveal sensitive
information; review the actual artifacts before sharing them.

HTML is a static rendering of a captured recorded bundle. It does not perform a
fresh source check or authenticate its recorded labels. The current CLI performs
no remote upload, notification or automatic domain edit. Non-null proposed edits
are rejected. Signed envelopes and remaining F13 workflows are pending.

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

# Authoring review exchange candidate

The implementation candidate adds `forge review authoring init`, `respond`,
`merge` and `status` with separate closed `/3` documents. Closed TEMP qualification
includes full tests, strict lint and a genuine compiled CLI campaign on macOS;
the qualification checkpoint below records exact documentation and LLVM scopes.
These instructions apply to the integrated open draft described in the
[readback checkpoint](#open-draft-integration-readback-2026-10-06-utc). Published
v1.1.0 does not include this exchange. Remaining hosted/platform/human acceptance
and release gates stay open.

Reviewer keys, roles, author keys and times are assertions. `quorum-met` means
that the declared review policy is satisfied. It does not approve a plan, certify
content, authenticate a reviewer, update an answer or clause, or perform a native
Lifecycle transition. The [authoring gate register](authoring-gates.md) retains
its separate partner, pilot and release dispositions.

## Select one complete saved plan

The adapter reviews one complete `forge.authoring-plan/1` generated from
`forge.author-project/1`. It recomputes the complete declared applicability/gap
baseline and native plan, then compares the saved reports with those full native
results. Private reviewer provenance, rationale, assignments, questions, answers,
clauses, counts and ordering participate in that comparison. A matching summary,
a schema-valid plan or a supplied hash is insufficient.

Native drafting states such as `planned`, `blocked-context`, `skeleton-ready` and
`human-draft-present` remain drafting states. A complete current plan can be
reviewed in any of those states; review does not resolve missing context. Unlike
[Lifecycle `/2`](lifecycle-review-exchange.md), this adapter does not require an
Approved Lifecycle record. Individual-clause selection, component
`forge.authoring-plan/2` envelopes, proposed edits and handoff are outside this
slice. Existing [native authoring](authoring.md) commands are unchanged.

Use an explicit normalized absolute project root. Every input and new destination
must be confined below it, with existing parent directories. The author project
retains its own directory as the base for native dependencies; its native
provenance label is its actual manifest filename. Prepare the private locator:

```json
{
  "schema_version": "forge.review-authoring-plan-inputs/1",
  "project": {"path": "project.json"},
  "plan": {"path": "saved-plan.json"}
}
```

The [locator schema](../schemas/forge.review-authoring-plan-inputs-1.schema.json)
is closed. It supplies routes, not a native input subset or currentness claim.
The receiver reads every dependency declared by the maintained authoring and
applicability manifests: pack, saved gap report, framework, any explicit resolved
Catalog, Mapping collections and human clause files. Native originals must remain
distinct after portable-label and physical-identity checks. The saved plan must
be a distinct Source original with no route or physical alias to a native
dependency. Equal bytes do not waive these rules.

## Declare the private review policy

The private `forge.review-authoring-plan-init/1` request contains exactly
`schema_version`, `roles`, `reviewers`, `policies` and `items`, with one policy and
one item. The item contains only `key`, `policy_key`, `author_keys`, `assignments`
and required explicitly nullable `due_at`. Native subject identity, complete
source pins, context and fingerprints derive from the actual native inputs.
They are not supplied in this request.

This example assigns two distinct reviewers and excludes the declared author:

```json
{
  "schema_version": "forge.review-authoring-plan-init/1",
  "roles": [{"key": "reviewer"}],
  "reviewers": [
    {"key": "alice", "role_keys": ["reviewer"]},
    {"key": "bob", "role_keys": ["reviewer"]}
  ],
  "policies": [{
    "key": "two-reviewers",
    "seats": [{"role_key": "reviewer", "count": 2}],
    "substitutions": [],
    "abstention_rule": "nonapproving",
    "empty_abstention_reasons": [],
    "author_separation": "declared-keys"
  }],
  "items": [{
    "key": "whole-plan",
    "policy_key": "two-reviewers",
    "author_keys": ["writer"],
    "assignments": [
      {"reviewer_key": "alice", "role_key": "reviewer"},
      {"reviewer_key": "bob", "role_key": "reviewer"}
    ],
    "due_at": null
  }]
}
```

Use the [init-request schema](../schemas/forge.review-authoring-plan-init-1.schema.json)
with the integrated draft. Keys and ordered declaration lists must be
unique and sorted as required by the closed format. Assignments and substitutions
must reference declared roles and reviewers and cannot make an author eligible.
A non-null due time must be later than queue creation. Review times use real
canonical UTC seconds (`YYYY-MM-DDTHH:MM:SSZ`). The native project's `as_of` keeps
its exact original spelling; review `--as-of` neither refreshes that date nor
extends an answer's validity.

## Candidate command sequence

Replace the project root, explicit UUIDs and timestamps with your chosen inputs.
These examples describe the integrated candidate, not the published binary.

```sh
forge review authoring init \
  --project-root /absolute/project \
  --sources review-sources.json --policy review-policy.json \
  --queue-id 11111111-1111-4111-8111-111111111111 \
  --created-at 2026-10-05T12:00:00Z --output review-queue.json

forge review authoring respond \
  --project-root /absolute/project --queue review-queue.json \
  --item-key whole-plan --reviewer-key alice --reviewer-role reviewer \
  --disposition approve --responded-at 2026-10-05T13:00:00Z \
  --response-id 22222222-2222-4222-8222-222222222222 \
  --rationale-file alice-rationale.txt --output alice-response.json

forge review authoring merge \
  --project-root /absolute/project --sources review-sources.json \
  --queue review-queue.json --response alice-response.json \
  --as-of 2026-10-05T14:00:00Z --output dispositions.json

forge review authoring status \
  --project-root /absolute/project --sources review-sources.json \
  --queue review-queue.json --response alice-response.json \
  --as-of 2026-10-05T14:00:00Z
```

`init` creates one native-derived Queue `/3`; the new queue is an output, not an
input Queue registration. `respond` writes one ordinary Response `/3` bound to
the exact queue bytes. It captures only that queue and the private rationale,
so it makes no current native-plan claim. Rationale is UTF-8 and at most 8 KiB.
Empty rationale is allowed only for abstention with a reason explicitly permitted
by the selected policy.

`merge` and `status` recheck the complete native plan, saved plan, actual queue
and every supplied response occurrence. Repeat `--response` for additional files;
ordering and duplicate occurrences remain part of the input cohort. There is no
directory discovery. `merge` writes a new Dispositions `/3` file; `status` writes
the same document family to stdout. One approval in this example cannot satisfy
its two-reviewer policy. With no responses, the item can remain `assigned` with
`blocking: false`; that flag does not establish a satisfied quorum.

## Exact bindings and preserved dissent

Queue, Response and Disposition `/3` are independently typed closed formats.
They preserve all eight source-pin fields: artifact key, kind, raw SHA-256, byte
length, schema identity, validation profile, native model and native root UUID.
Required nullable fields stay explicit. There is no conversion through
Mapping/applicability `/1` or Lifecycle `/2` envelopes.

Complete native provenance includes every maintained native input in its original
order. It excludes the saved plan and review-only locator, policy, queue,
responses and output. The public source roster additionally includes the distinct
saved plan. Final checks also retain every actual operation original, including
all Auxiliary and repeated Response occurrences, plus native physical proofs and
root/absence observations. A caller-selected subset cannot replace these sets.

The public subject is `authoring-plan:<64-hex>` derived from the private native
project key. Exact native provenance, saved-plan bytes, source pins, subject,
context and asserted policy fingerprints bind the one item. Loaded hashes or a
recorded `Current` label confer no live currentness; current results require the
genuine owner and fresh full-input verification.

Responses retain classification and history for foreign queues, stale bindings,
future/late times and unassigned assertions. Identical repeated originals are
accounted for; one response UUID with differing bytes is ambiguous and refused.
A supersession must name both the previous response UUID and its exact raw hash
with `--supersedes-id` and `--supersedes-sha256`; it cannot withdraw another
reviewer's decision. Malformed withdrawal links, missing targets, self-links,
forks and cycles refuse the entire result. Independent incompatible heads from
the same reviewer remain ordinary policy conflict facts, with dissent retained.

Quorum uses distinct eligible reviewer keys, excludes authors, and treats
abstention as nonapproving. Rejections and requests for changes remain visible;
approval and rejection from different eligible keys produce conflict rather than
silently selecting an approval. Due-time equality is late. `quorum-met` describes
the asserted policy only, with the contributing witness and dissent retained.

## Privacy, bounds and publication

Default portable output contains IDs, hashes, bounded reason codes, asserted
review keys and policy metadata. It omits native paths, framework versions/hrefs,
questions, answers, clauses, source prose and private native reviewer rationale.
Private Response files contain their supplied reviewer rationale. Identifiers,
UUIDs and hashes can still disclose project relationships; review every artifact
before sharing through your own filesystem or source-control workflow. FORGE
sends no notifications, messages or remote mutations.

Limits include a 10 MiB queue, 1 MiB per response, 32 MiB dispositions including
the final LF, 1 MiB locator and init request, 100 Source registrations and 50 MiB
distinct Source raw bytes. There are at most 10,000 response registrations and
32 MiB distinct Response raw bytes. Native project/pack and clause limits remain
in force. All stages share the original resource/work allowance and cooperative
30-second control. Exceeding it refuses the operation without truncation or a
fresh budget. Logical reservations are not an allocator heap guarantee;
checkpoints do not hard-preempt an individual native/parser call.

The candidate uses the existing Linux/macOS guarded no-replace publisher and
retains production Windows publication refusal. No new platform qualification
follows from that source choice. Destinations must be new and parent directories
must exist. Namespace reservation before capture is lexical; it is not an
observation that a destination is absent. Complete input checks run after
construction and readback, and again at the publication guard. Those checks are
sequential, not an atomic filesystem snapshot. A late publication/sync error may
leave a complete visible file; a stdout write may be partial if output or control
fails.

## Qualification checkpoint and remaining work

The completed TEMP qualification uses one exact 1,864-file source candidate over
its 6f5 basis. It has format/check and strict all-target/all-feature Clippy success.
The normal all-feature run passed 4,248 tests with zero failures and three
existing ignored WI-31 (`--set-param`) golden controls across 75 result groups,
including 24 doctests. Ignored controls receive no execution or feature credit.

A genuine compiled Forge binary completed 21 child invocations: four maintained
native setup commands and 17 review commands, with 22 completed observations.
It exercised all four Authoring routes through the real CLI parser and dispatch,
complete native saved-plan preparation, exact eight-field pins, two-key quorum,
dissent, duplicate/order generation, malformed originals, real native drift,
private-output exclusion and no-replace publication. The native plan's expected
action-required exit 1 and expected publication/input refusals are preserved.
Source, managed-file and binary pins remained equal within that completed
interval. This local macOS campaign does not establish Linux or Windows acceptance.

The lexical Rustdoc census covers 44 changed/new physical Rust paths. All 638
selected named callable bodies and 145 selected named types have adjacent outer
Rustdoc. The function population contains 394 production-lexical, 243 external
cfg-test and one inline cfg-test body. Selection includes formatter/position
changes and is not a whole-repository or documentation-quality score. The broader
whole changed-file population is 859 documented bodies out of 1,025, with 166
remaining unchanged legacy gaps outside the selected population. Its 150 literal
registered test declarations are a lexical inventory, not an executed denominator.

Fresh library-only LLVM instrumentation passed 3,157 library tests, with no
failed or ignored tests. It is separate from the full normal integration and
doctest run. Physical file-summary coverage is:

| Physical changed-file cohort | Reported files | Covered lines | Covered functions |
| --- | ---: | ---: | ---: |
| All present changed/new Rust files | 43 of 44 | 17,454 / 19,480 (89.60%) | 1,457 / 1,715 (84.96%) |
| Source paths, including inline/generated/legacy bodies | 35 | 13,110 / 15,110 (86.76%) | 1,146 / 1,401 (81.80%) |
| External physical test files | 8 | 4,344 / 4,370 (99.41%) | 311 / 314 (99.04%) |

These are complete-file line/function populations, not changed-line or pure
production-code coverage. `src/review/mod.rs` is genuinely absent from this
library export and has null coverage; it is not assigned invented zero coverage.
Included cfg leaves with `../` spellings are correlated to their actual physical
files. Branch and MC/DC coverage are unmeasured. The standalone binary entrypoint
is outside this library target; the CLI campaign is separate execution evidence,
not CLI instrumentation. LLVM used a reused build cache and four current-run
profiles; recorded binary pins attest only the producer/merge/export intervals.

Earlier component evidence remains separate. The 268-control command run includes
16 new direct production-command controls; it does not become a CLI run merely
because the later campaign passed. Init/export V2 passed 112 selected controls,
including 18 new export controls. The binding/policy/finalizer trial ran eight
Cargo groups successfully with 234 passing controls; its original recorder failed
a 36-versus-37 substring-selection inventory check. A retained data-only audit
reconciles the extra unchanged `suggest::review` control. These overlapping
cohorts are not added together. Zero-test compiler failures, the failed command
fixture trial and earlier strict-lint failures remain historical evidence.

At this earlier TEMP checkpoint, exact managed integration and final
source/documentation readback were required before drafting. The later
[open-draft readback](#open-draft-integration-readback-2026-10-06-utc) records the
integrated PR and mixed hosted gates. Real review-team workflows, reviewer
authority, interoperability, additional platform and privacy qualification,
pilot measurements and release acceptance remain separate.
Other F13 adapters, individual-clause and component-plan review, proposed
edits, local response clients and signed envelopes remain open. This is a scoped
documentation proposal; the user's final goal-wide documentation review is not
complete.

### Open-draft integration readback: 2026-10-06 UTC

[Draft PR #219](https://github.com/policy-forge/forge/pull/219) contains the
Authoring `/3` exchange at head
`304b31f8ea3913eba4a7d776b833b9b3db4ad3a4`, based on the Lifecycle review branch.
It remains open and is not included in verified `main`
`aef0ab24f77559593b6d0b4fe5027f8a5eaa857e` or the published v1.1.0 binary.
The prior TEMP qualification checkpoint retains its exact source, test,
documentation, compiled-child and library coverage populations; the integration
readback is a separate later observation.

Hosted Test and Workspace API jobs pass on macOS, Linux and Windows at this
head. Two hosted gates still fail:
[Supply-chain audits](https://github.com/policy-forge/forge/actions/runs/37397458081/job/112056718700)
and the
[Linux headless IP-denial prerequisite](https://github.com/policy-forge/forge/actions/runs/37397458213/job/112056719119).
Those failed rows are retained without converting them to acceptance or assigning
unverified detailed failure counts. Passing tests do not change production
Windows publication refusal, provide an installed-platform CLI campaign, or
supply reviewer authority, real-team workflows, privacy/interoperability studies,
pilot measurements or release approval.

No Must/Should or Ready checkbox is changed by this readback. Other adapters,
individual-clause/component-plan review, proposed edits, local response clients
and signed envelopes remain separate unfinished work. The whole-goal
documentation review remains OPEN.

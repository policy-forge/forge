# Lifecycle review exchange candidate

The implementation candidate registers four local commands: `forge review
lifecycle init`, `respond`, `merge` and `status`. These instructions apply after
the candidate code and documentation are integrated. Published v1.1.0 does not
include this exchange. One authentic 2026-10-05 coupled run passed 63 controls,
including a real in-process clap-to-execute workflow. Later local checks cover
normal regressions, strict lint, library-only LLVM and a macOS child workflow;
other platform and human acceptance remain open.

The [portable review guide](review-queues.md) covers the separate F12
Mapping/applicability `/1` commands. [Native Lifecycle commands](../README.md#policy-lifecycle)
retain their own transition and approval rules.

## Prepare and review one policy version

Use an explicit normalized project root. Every input and new file destination
must be confined beneath it; there is no implicit project or response discovery.
The native Lifecycle record must already be Approved/current. Prepare the
complete private `forge.review-lifecycle-inputs/1` locator and the separate
`forge.review-lifecycle-init/1` asserted policy request. The request has exactly
`schema_version`, `roles`, `reviewers`, `policies` and `items`, with one policy
and one item. That item contains only `key`, `policy_key`, `author_keys`,
`assignments` and required explicitly nullable `due_at`. Native subject identity,
pins and fingerprints come from the complete actual closure, not this request.

Use the complete [private locator schema](../schemas/forge.review-lifecycle-inputs-1.schema.json)
and [asserted init-request schema](../schemas/forge.review-lifecycle-init-1.schema.json)
when preparing those files. They define required nullable fields and the complete
record/source/generated roster; the command does not discover omitted inputs.

The examples assume those private files already exist at the root. Replace
`/absolute/project` with that actual root, use new destinations, and choose keys
matching the explicit policy. Here the item is `policy-review`, the asserted
reviewer is `reviewer-a` and the assigned role is `security`. UUIDs and times are
explicit assertions in canonical form. These examples supply no native approval
or reviewer authority.

```bash
forge review lifecycle init \
  --project-root /absolute/project \
  --sources lifecycle-review-inputs.json \
  --policy lifecycle-review-policy.json \
  --queue-id 11111111-1111-4111-8111-111111111111 \
  --created-at 2026-10-05T12:00:00Z \
  --output lifecycle-review-queue.json

forge review lifecycle respond \
  --project-root /absolute/project \
  --queue lifecycle-review-queue.json \
  --item-key policy-review \
  --reviewer-key reviewer-a \
  --reviewer-role security \
  --disposition approve \
  --responded-at 2026-10-05T13:00:00Z \
  --response-id 22222222-2222-4222-8222-222222222222 \
  --rationale-file private-review-rationale.txt \
  --output lifecycle-review-response-a.json

forge review lifecycle merge \
  --project-root /absolute/project \
  --sources lifecycle-review-inputs.json \
  --queue lifecycle-review-queue.json \
  --response lifecycle-review-response-a.json \
  --as-of 2026-10-05T14:00:00Z \
  --output lifecycle-review-dispositions.json

forge review lifecycle status \
  --project-root /absolute/project \
  --sources lifecycle-review-inputs.json \
  --queue lifecycle-review-queue.json \
  --response lifecycle-review-response-a.json \
  --as-of 2026-10-05T14:00:00Z
```

`init` derives a new queue from the complete held native closure and private
policy. `respond` binds the exact recorded queue and UTF-8 rationale, at most
8 KiB, without a fresh native-currentness claim. `merge` checks the complete
native closure, exact queue and every supplied response occurrence before
publishing a new recorded disposition. `status` performs the same current
preparation and writes complete disposition JSON to stdout.

Repeat `--response` for every intended occurrence, including repeats; input order
is retained. Omitting the flag means no response occurrences, not an automatic
search. `--as-of` is explicit canonical UTC seconds. `status` has no `--output`;
the other commands require a new output destination. For `respond`, the allowed
dispositions are `approve`, `reject`, `request-changes`, `abstain` and
`superseded`. An empty abstention rationale requires `--abstention-reason`
permitted by the explicit policy. An explicit response chain uses both
`--supersedes-id` and `--supersedes-sha256`; it supplies no vote-transfer or
identity authority. Inspect the private response rationale before sharing it.

## Separate formats and native authority

Lifecycle re-review uses closed `forge.review-queue/2`,
`forge.review-response/2` and `forge.review-dispositions/2` documents. It does not
cast, upgrade or widen `/1`. Its sole domain is `lifecycle-policy-version`, with
requested action re-review; proposed edits remain explicitly null.

A complete recorded Approved/current native tuple is required for queue export
and current merge/status. The adapter checks the intrinsic record, exact opaque
source bytes and every declared generated artifact. Opaque sources may be empty
or non-UTF8. Generated artifacts are checked for recognized JSON model/root and
parseable UUID identity. That predicate does not establish full OSCAL schema or
domain validity, and native root UUID text retains its original spelling.

Reviewer keys, roles, authors, assignments and timestamps are asserted. Quorum
or a recorded `Current` label does not authenticate reviewers, grant authority,
approve a policy, change native Lifecycle state, or establish implementation or
effectiveness. A loaded label is ordinary recorded data; each fresh current
operation needs the complete actual input closure.

## Exact pins and complete response evidence

Each public source pin has exactly eight fields:

| Field | Meaning |
|---|---|
| `artifact_key` | Fixed role/ordinal token, not a private path or policy key |
| `kind` | Lifecycle record, opaque source or generated-artifact occurrence |
| `raw_sha256` | Exact original byte digest, including whitespace and final newline |
| `byte_length` | Full original extent; an opaque source may have zero bytes |
| `schema_identity` | Intrinsic record marker, otherwise explicit `null` |
| `validation_profile` | Intrinsic-record, opaque-byte or generated-identity predicate |
| `native_model` | Recognized generated model, otherwise explicit `null` |
| `native_root_uuid` | Original parseable generated UUID spelling, otherwise explicit `null` |

Nullable fields must be present. Hashes or equal copied bytes alone cannot
establish the actual source owner. Current binding includes the complete pin
roster, subject/context/policy fingerprints, exact queue and every ordered
response registration, including repeated occurrences.

Foreign, stale, late, conflicting and duplicate evidence remains present for
classification and generation accounting. It is not silently discarded or made
current. Only eligible responses contribute to asserted policy evaluation; seat
matching requires distinct reviewer keys, excludes authors and preserves
dissent. Abstention never approves.

## Disclosure, bounds and failed output

Queue and disposition projections omit dedicated private source paths, policy
prose, rationale and evidence excerpts. Identifiers, digests and asserted
reviewer metadata can still reveal sensitive information. This exchange adds no
signatures, verified identities, external messaging, evidence-content retrieval
or native mutation.

One operation retains its original cooperative 30-second deadline and admission
ledger through parsing, native checks, finalization, finite encoding and output.
Ordinary validation failures still receive final stop checks; the first Capacity,
Interrupted or control failure stays binding. Encoded queues are bounded at
10 MiB, responses at 1 MiB and dispositions at 32 MiB, including the final LF.
The logical derived allowance is 32 MiB, separate from bounded original-input
pools. Logical admission units do not measure heap use or guarantee that every
near-limit combination fits. A blocking parser/native call cannot be forcibly
preempted.

Actual owners remain held through complete strict output readback and the final
whole-input fence. Sequential verification is not an atomic filesystem snapshot.
Output namespace reservation before input reads is lexical; it does not prove
destination absence. Missing or invalid input can refuse before the publisher
checks an existing destination. File publication uses the maintained guarded
no-replace route on Linux/macOS and refuses on Windows; this source behavior
does not establish new Lifecycle platform qualification. A writer failure can
leave a prefix, and a late stop or durability error can leave a complete new
file. Inspect the destination after a reported failure before another attempt.

## Qualification checkpoint and remaining work

One authentic coupled component run passed **63 controls** with zero failures or
ignored tests: **11 receiver, 7 binding, 9 finalizer, 7 current-command,
8 response, 12 queue-export and 9 init/CLI controls**. The cohort exercises real
captured originals, native currentness, quorum/dissent, complete response
ordering, finite typed output/readback, native no-replace publication and
calibrated original-control stops, including all four in-process CLI routes.
This is one preformat TEMP source cohort, not cumulative totals from overlapping
runs or a separate executable/platform matrix.

The later exact formatted TEMP candidate passed formatting/check, strict
all-target/all-feature Clippy and the normal all-feature suite: **4,098 passing
tests, zero failures and three ignored tests across 75 result groups**, including
integration and doctests. Historical failed producers remain retained; these
local results do not establish managed integration or hosted qualification.

The final lexical census found adjacent Rustdoc on **492/492 selected function
bodies** (282 production and 210 external cfg-test positions) and **137/137 selected
named types**. Its 119 literal selected test declarations each have one passing
name correlation in the normal log. This is not an expanded AST or whole-repository
docstring metric, and declarations are distinct from execution totals. Across the
whole physical changed files, 661/753 function bodies have adjacent Rustdoc; the
remaining positions include unchanged code and tests.

A separate fresh **library-only** LLVM run passed **3,007 tests** with zero
failures/ignored tests. Merge/export used only its four fresh profiles. The changed
physical file cohort emitted rows for 36 of 37 files: **12,733/14,315 executable
lines (88.95%)** and **1,047/1,196 functions (87.54%)**. `src/review/mod.rs` emitted
no row. These physical files include tests, generated functions and unchanged
lines; these are not pure-production or changed-line percentages. No branch or
MC/DC denominator was emitted. No integration/doctest coverage follows from the
library instrumentation.

A separate macOS executable run completed 13 command checks: four help commands,
one argument refusal, three native fixture init/transitions, all four review
commands and one existing-destination refusal. It preserved eight complete pins
and two supplied response occurrences (one unique response and one exact repeat),
and verified original inputs unchanged. Its synthetic approvals and model/UUID
identity fixtures establish no human authority or full OSCAL validity.

Linux/Windows Lifecycle execution, platform/privacy/accessibility qualification,
real reviewer/owner/team acceptance and release gates remain open. Other F13
adapters, proposed edits, interactive response-file clients and signed envelopes
remain separate work. This guide does not finish the final goal-wide docs review;
reconcile it again against the eventual integrated source.

# Authored POA&M item workflow

The integrated workflow consumes the exact F07 source foundation through
[explicit check/build commands](poam-cli-workflow.md). This is a technical F08
tranche; the remaining reporting and acceptance gates are listed below. The empty `forge.poam/1` scaffold and explicit source-only checks retain
their source-integrity scope.

The workflow parser accepts the same manifest version with the closed
item shape below through a separate, explicitly selected command mode. The
foundation parser continues refusing authored work. An item names an exact
result/kind/key/native UUID/computed object SHA256 tuple from the captured AR
inventory. Its owner and each milestone's owner are declared parties and roles
with explicit rationale. Forge chooses no item, owner, outcome or date.

Items declare `key`, `title`, `description`, `source_refs`, `owners`, `target_date`,
`state`, `history`, `milestones` and an optional `supersedes` link (see below). Milestones declare `key`, `outcome`,
`target_date`, `depends_on`, `owners`, `state` and `history`. Every history event
names its immutable key, actor party/role, explicit RFC3339 time, prior state,
next state and rationale. The first event is an attributed `planned` assertion;
subsequent events must be strictly chronological. Item and milestone UUIDs use
F07's existing exact-key UUID-v5 protocol, independent of dates and prose.

Supported states are `planned`, `in-progress`, `blocked`, `cancelled` and, under
the D064 [owner disposition](plans/2026-10-25-owner-dispositions.md) recorded on 2026-10-25, the
terminal assertions `completed-asserted` and `accepted-risk-asserted`. The
transition table is: `planned` to `in-progress`, `blocked`, `accepted-risk-asserted`
or `cancelled`; `in-progress` to `blocked`, `completed-asserted`,
`accepted-risk-asserted` or `cancelled`; `blocked` to `in-progress`,
`accepted-risk-asserted` or `cancelled`. Terminal records never transition again.

A terminal event must carry a `closure` reviewer assertion:

```json
"closure": {
  "reviewer": {"role_id": "reviewer", "party_key": "independent-reviewer"},
  "reviewed_at": "2026-01-20T12:00:00Z",
  "rationale": "Reviewer rationale",
  "evidence": [{
    "key": "proof",
    "href": "evidence/proof.json",
    "expected_sha256": "<64 lowercase hex>",
    "captured_at": "2026-01-19T09:30:00-08:00"
  }]
}
```

The reviewer party must differ from the asserting actor and from every owner of
the record; for a milestone that means the milestone owners and the parent item
owners. `reviewed_at` cannot precede the assertion or exceed the document
`last_modified`. At least one evidence reference is required; each needs a
confined relative `href`, a lowercase SHA-256 and an explicit RFC3339 `captured_at`
with an offset, no later than the review. Forge never defaults a missing capture
time, time of day or time zone. Risk acceptance requires a selected native risk and
is item-only. Item completion requires every milestone to be `completed-asserted`
with its review no later than the item assertion. Anything less is invalid input
(exit 2) with a specific reason.

An admitted terminal state records that someone asserted the outcome. Forge does
not read evidence bytes, judge sufficiency or freshness, or authenticate the
reviewer, and it never presents the record as verified closure. Risk acceptance is
declared, not approved by Forge. Cancellation closes no source finding or risk.

Milestone dependencies are within the item and must name earlier declared
milestones, without duplicates. Targets use canonical Gregorian `YYYY-MM-DD`,
no later than the item target, and no earlier than a prerequisite's target. This
ordered subset rejects missing, self, forward and cyclic dependencies. It does
not silently reorder the author's plan.

The report requires explicit `as_of`. The optional `due_soon_days` interval is
0 through 365 and defaults to zero in the CLI. A record
is overdue only when its target is strictly before as-of and its state at as-of
is open. Due-soon includes the as-of date through the inclusive supplied interval.
State at as-of uses the UTC date of each explicit event. Records before their
first assertion and terminal records (cancelled or asserted) remain in the complete
denominator and are never overdue or due soon. Counts
are separate item, milestone, source-object, overdue, due-soon and blocked
quantities; overlapping conditions are not unique item counts. Reports contain
stable keys/UUIDs, dates and assertion states, without actor names, rationale,
evidence locations, assessment prose or private paths.

The typed native subset has official POA&M metadata, explicit items,
a confined SSP import and five exact source file receipts. Items link to external
AR objects and carry fixed namespaced properties for exact source tuples, owner
bindings, dates, milestone identities and history. A `completed-asserted` or
`accepted-risk-asserted` item also carries `assertion-status` and a compact JSON
`terminal-assertion` property (status, asserting actor and time, reviewer key,
role and party UUID, review time, rationale and every evidence reference with its
hash and capture time). A reopened item carries `supersedes` with the superseded
item's UUID. All of these use the Forge namespace. It creates no local native
findings or risks, so an assertion never sets a native risk status such as
`closed` or `deviation-approved`, and it emits no dangling local
`related-findings`/`related-risks` references. Every native date-time is copied
from an author-supplied RFC3339 value; full-date targets stay in properties. The native artifact includes author-supplied work prose and party
names and can be sensitive. It is not a redacted schedule report. Independent
native semantic review and independent-tool interpretation remain required.

Reopening never edits a terminal item. A new item with its own immutable key sets
`"supersedes": "<terminal item key>"`. The link must name a different terminal
item in the same plan, each terminal item can have at most one successor, and the
successor's first event must be strictly after the superseded item's final event.
The terminal item stays in the manifest with its complete history.

An explicitly supplied prior authoring manifest enforces exact event-prefix
preservation and immutable plan/item/milestone keys. This does not prove history
before that supplied baseline. Removal, any change to a terminal record and any
added, removed or changed `supersedes` link on an existing item are refused. The
separate [baseline comparison command](poam-baseline-comparison.md) reports added,
removed and changed identities and every reopen relation with its
`accepted_for_build` admission. Prior
baseline bytes receive structural validation only in this native-production lane;
current source bytes and the exact selected source tuple are independently checked.

Bounds remain four MiB raw manifest, 64 KiB decoded strings, 10,000 items, 1,000
roles and parties each, 1,000 source references/milestones/events per record,
10,000 milestones and 100,000 history events across the complete manifest. Each
native/report output is at most ten MiB. F07's five confined source inputs and
100 MiB whole capture ceiling are unchanged. These are standalone POA&M bounds,
not changes to workspace 100-path, retention or request limits. Temporary parse,
projection and serializer allocations are not claimed as an exact heap bound.

The CLI captures and rechecks original manifest/baseline bytes and file identities
and all five source generations before using the existing no-replacement publisher.
Generated relative source links require a new filename in the manifest directory.
The native artifact and report are separate publications. Invalid input or output
failure returns exit2; complete valid overdue/due-soon/blocked actions return exit1,
otherwise exit0. Structured native property values use compact bounded JSON so
authored newline/tab content is escaped and round-trips exactly; the complete
native artifact and JSON schedule retain pretty formatting and one final newline.

See the [implementation plan](plans/2026-10-03-f08-item-workflow-proposal.md) for
Root-owned ports, unresolved decisions, proposed controls and remaining F08 scope.

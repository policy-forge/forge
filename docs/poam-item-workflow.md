# Authored POA&M item workflow

The integrated nonterminal workflow consumes the exact F07 source foundation
through [explicit check/build commands](poam-cli-workflow.md). This is a technical
F08 tranche with the remaining baseline, closure, reporting and acceptance gates
below. The empty `forge.poam/1` scaffold and explicit source-only checks retain
their source-integrity scope.

The workflow parser accepts the same manifest version with the closed
item shape below through a separate, explicitly selected command mode. The
foundation parser continues refusing authored work. An item names an exact
result/kind/key/native UUID/computed object SHA256 tuple from the captured AR
inventory. Its owner and each milestone's owner are declared parties and roles
with explicit rationale. Forge chooses no item, owner, outcome or date.

Items declare `key`, `title`, `description`, `source_refs`, `owners`, `target_date`,
`state`, `history` and `milestones`. Milestones declare `key`, `outcome`,
`target_date`, `depends_on`, `owners`, `state` and `history`. Every history event
names its immutable key, actor party/role, explicit RFC3339 time, prior state,
next state and rationale. The first event is an attributed `planned` assertion;
subsequent events must be strictly chronological. Item and milestone UUIDs use
F07's existing exact-key UUID-v5 protocol, independent of dates and prose.

The first shipping boundary supports `planned`, `in-progress`, `blocked` and
explicit `cancelled` assertions. Completion and risk-acceptance history is refused
by public parse/preparation, even when its proposed review fields are structurally
valid. There is no approval flag or public bypass. The candidate separately
validates the proposed reviewer/evidence shape so the pending closure decision is
concrete; that validation does not establish that the evidence is sufficient,
fresh, effective or provided by an authorized reviewer.

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
first assertion and terminal records remain in the complete denominator. Counts
are separate item, milestone, source-object, overdue, due-soon and blocked
quantities; overlapping conditions are not unique item counts. Reports contain
stable keys/UUIDs, dates and assertion states, without actor names, rationale,
evidence locations, assessment prose or private paths.

The typed native subset has official POA&M metadata, explicit items,
a confined SSP import and five exact source file receipts. Items link to external
AR objects and carry fixed namespaced properties for exact source tuples, owner
bindings, dates, milestone identities and history. It creates no local native
findings or risks and emits no dangling local `related-findings`/`related-risks`
references. The native artifact includes author-supplied work prose and party
names and can be sensitive. It is not a redacted schedule report. Independent
native semantic review and independent-tool interpretation remain required.

An explicitly supplied prior authoring manifest enforces exact event-prefix
preservation and immutable plan/item/milestone keys. This does not prove history
before that supplied baseline. The first lane refuses removal and terminal
identity revision; a full baseline impact/removal/reopening workflow is still
required. Prior baseline bytes receive structural validation only in this lane;
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

# F08 first authored-item implementation proposal

Root integration note: the source proposal below records its frozen, unexecuted
author-lane basis. The current integration consumes it with the actual nested
producer/consumer path correction, compact declaration successor V3 and the
clap scope-conflict correction. Original failures and successor checks are
retained separately; this historical proposal grants no runtime or acceptance
credit. Current behavior is described in the [authored command guide](../poam-cli-workflow.md).

Basis: frozen F07 foundation V5, index SHA
`10dad5d8dca3cd5dd2b3feb4f4da001850d8f45bc0f1e84522579060cbcd5041`,
composed from immutable `309b5e5f7b8265f5adec4e959152d26b44c3dfc9`
and the explicitly retained formatted F09 prerequisite. This is a virtual source
basis, not a claim that a current managed checkout or F07 commit has these bytes.
Root must reconcile its formatting/final F07 source successor before application.
The original F07 packet is immutable and all historical F09/F10 evidence remains
separate.

This candidate adds `src/poam/workflow.rs`, `workflow_model.rs`, two module
declarations and appended actual-source library controls in the existing F07
integration file. It does not modify shared CLI, error, config, workspace roles,
schemas, API, transport or export source. No dependency or feature is introduced.
The proposed command integration is Root-owned.

## Concrete first useful lane

`workflow::parse(bytes)` validates the closed nonterminal authoring contract.
`workflow::prepare(manifest_path, raw_bytes, as_of, due_soon_days, prior_bytes)`
returns `PreparedWorkflow`: complete artifact/schedule byte slices, a valid-state
review-action boolean and `verify_inputs()`. It consumes actual F07 `source::load`
and `source::validate_selection` with original companion/result identity checks.
No dummy snapshots, rewritten AR identities or automatic selections are used.
The pure producer performs no output write, network operation or tool execution.

Root should add an explicit `poam build --manifest ... --as-of ...
--due-soon-days ...` and an explicit workflow check mode. Preserve every existing
valid init/source-only request and reject ambiguous source-only/workflow flags.
The old foundation control asserting no build is advertised will need a disclosed
new-command adapter when that separate integration actually occurs; its old bytes
are preserved by this source-only candidate. No CLI compatibility result is claimed.

Before `prepare`, Root captures the actual manifest and optional baseline through
the confined held-generation reader. Before publication, it rechecks both and
calls `PreparedWorkflow::verify_inputs`. A native artifact destination stays at
the bundle root and is a new no-replacement target. The command must expose real
publisher failures; do not claim rollback or atomicity for separate artifact and
report outputs. Root maps `review_required` to valid exit1, false to exit0, and
invalid parse/source/history/schema/I/O to exit2. Escaped JSON report output can
be the first consumed report surface; future text/HTML must escape authored keys.

## Decisions and approval boundaries

The actual PRD064 remains Draft. Lines197–199 ask about terminal evidence,
supported native subset and editable source-of-truth; lines204–209 require
specific owner/compliance dispositions and representative plans. No affirmative
D064 disposition was found in the bounded retained decision/source inputs, and
none is inferred here. Root requested a nonterminal fail-closed shipping boundary.

| Choice | Reviewable proposal | Present boundary |
|---|---|---|
| Editable source | Local closed manifest is authoritative authoring input; native POA&M is generated, never reverse-edited implicitly | Root engineering choice pending; no owner acceptance claimed |
| Native subset | Explicit native items/metadata/SSP import/source receipts; source links plus fixed namespaced workflow properties, no fabricated native risks/findings | Root must review exact native semantics; independent-tool acceptance open |
| Closure evidence | Different declared reviewer party, coherent explicit review time/rationale and at least one confined metadata hash pin; completion after every milestone review; acceptance requires a selected risk | Structurally proposed only; public parser/preparer refuses completion/acceptance history |
| Terminal reopening | New explicitly linked identity and full baseline impact, not an outgoing transition from a terminal identity | No reopening producer yet; same-key terminal revision refused |
| History trust | Exact prefix comparison to explicitly supplied prior authoring bytes | No authentication, persisted-history guarantee or full baseline impact inferred |
| Evidence freshness | Consume existing fresh PRD060 linkage machinery in a later lane | Current evidence metadata is not fetched or verified |

The terminal evidence sufficiency/compliance question is the genuine unresolved
owner gate. It does not block independent nonterminal owner/date/DAG/history
implementation. Ordinary numeric bounds and ordered subset selection are proposed
engineering choices for Root review, not invented human approval requirements.
Legal wording continues to say assertions and unverified authority.

## Complete F08 requirement map

| Requirement | This candidate / still required |
|---|---|
| M1 commands | Off-store producer ready; Root-owned build/workflow-check CLI wiring and safe output controls remain |
| M2 schema | Reuses exact F07 official OSCAL1.2.3 schema/provenance; no new measured provenance claim |
| M3 manifest | Closed duplicate-safe raw four MiB parsing; all proposed named fields documented |
| M4 sources | Consumes original F07 native AR/four-companion capture/pair/import/UUID/version/hash validation |
| M5 selection | Every explicit item requires exact existing captured source tuples; no eligibility filtering or implicit work |
| M6 identities | Existing item/milestone UUID-v5 keys; native milestone property has explicit stable UUID |
| M7 ownership | Declared role/party resolution and rationale; authentication/authority not inferred |
| M8 milestones | Ordered local DAG, complete counts, full dates, outcomes and stable index-path cycle errors |
| M9 history | Attributed initial/transition events and explicit-baseline prefix guard; persistent/full revision workflow still required |
| M10 closure | Concrete proposed fields/structural checks; shipping terminal assertions refused until disposition |
| M11 schedule | Complete deterministic explicit-as-of/interval rows and separate conditions/denominators |
| M12 native output | Typed subset and whole official-schema validation; native semantic review/interoperability open |
| M13 baseline | Prefix/identity refusal only; owner/date/outcome/status/rationale/source impact, additions/removals/reopening reports still required |
| M14 nonmutation | Producer never mutates AR, closes findings or posts externally; Root publication must preserve it |
| M15 safety/privacy | Complete standalone bounds and minimized report; actual safe publication/platform tests remain |
| M16 exits | Producer action boolean/typed errors; Root CLI exit0/1/2 integration remains |
| M17 tests | Proposed actual parser/native/source/history/date/DAG/closure/refusal/determinism tests; all unexecuted here |
| S1 portfolio | Required later bounded complete portfolio producer, not included |
| S2 HTML/trace | Required later escaped static timeline/source-to-remediation view, not included |
| S3 outbound change set | Required later connector-neutral hash/precondition-bound package; no external mutation authority |
| S4 fresh evidence | Required later actual PRD060 freshness linkage; metadata assertions here are not substitutes |

Three representative sanitized remediation plans, independent-tool native
interoperability, remediation pilots, F10 selected-risk/epoch integration, full
F08 coverage and all owner/human/release gates remain open. This is not F08
completion and does not narrow the full completion plan.

## Proposed regression and evidence work

Unit declarations exercise exact explicit owners/actors, all transition pairs,
duplicate/unknown JSON, calendars/leap boundaries, DAG/cycle/order/date relations,
strict history times and prior states, proposed review structural failures,
default terminal refusal, append-only prefixes, complete schedule denominators,
redaction, stable IDs and whole output/count overflow. Proposed native-fixture
library tests use F07's actual five-file AR producer and exact inventory rows,
without response injection. They cover source-field drift, companion-byte drift,
post-prepare revalidation, deterministic schema-valid output, nonmutation and
baseline rewriting. Existing integration bytes are an unchanged prefix.

Root must compile/format/lint, execute exact controls and shared caller regressions,
add actual command/safe-write/platform cases, and bind a fresh source census and
coverage receipt. No proposed tests, native schema compilation, format/lint or
runtime coverage have executed in this author lane. No synthetic identities or
assertions supply owner/human acceptance. Parent authorship of the source and
historical F07 work excludes independent own-source correctness credit.

See the [workflow guide](../poam-item-workflow.md) for the closed fields, report
semantics, limits and native subset boundary.

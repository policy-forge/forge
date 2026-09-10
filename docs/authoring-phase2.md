# Components, impact, HTML and draft handoff

This technical extension builds drafting artifacts from explicitly supplied
local inputs. PRD-061 as a whole, human readiness, legal/content review,
design-partner packs, measured authoring exercises, pilot metrics and release
approval remain pending. Reviewer and component-status metadata are assertions,
not authenticated identities or independent review evidence.

## Compatibility and contracts

Existing `forge.author-project/1` and `forge.authoring-pack/1` are unchanged.
Omitting new options preserves Phase 1 commands, default artifacts and bytes.
There is no implicit migration or pin refresh. JSON rejects unknown and duplicate
decoded keys, null where absence is intended, unsupported versions, unsafe paths
and exceeded bounds. JSON Schema preflight is not a replacement for runtime
validation, exact PRD-056 recomputation and confined filesystem capture.

| Contract | Purpose |
|---|---|
| `forge.author-components/1` | Explicit component extension pinned to exact project bytes |
| `forge.authoring-plan/2` | Envelope containing the base plan and redacted component evidence |
| `forge.authoring-provenance/2` | Exact output spans plus component/parameter origins and captured hashes |
| `forge.authoring-component-lock/1` | Project, extension, output, provenance and parameter hashes |
| `forge.authoring-impact/1` | Explicit old/new snapshot pins and optional reviewed control correspondence |
| `forge.authoring-impact-report/1` | Hash-bound findings, change axes, affected and unaffected dependencies |
| `forge.author-handoff/1` | Exact prior build pins and explicitly supplied lifecycle governance |
| `forge.author-handoff-receipt/1` | Generation/source/record hashes and parsed handoff contract digest |

Public CLI enums gain fields and variants. Downstream exhaustive pattern matches
may need new fields/arms; adding a wildcard or selecting only needed fields is a
consumer migration decision. The existing public Rust API compatibility gate
remains open: neither these additions nor Phase 1 error variants are claimed to
be a source-compatible release. Package version and Cargo.lock remain unchanged;
no release is authorized by this tranche.

## Explicit components

```sh
forge author plan --manifest project.json --components components.json
forge author build --manifest project.json --components components.json \
  --output-dir generation --html
```

`--components` is a portable path relative to the author project directory. The
[component extension schema](../schemas/author-components.schema.json) describes
its closed input. Every instance has a stable key, explicit policy/topic/gap
assignment, reviewer/time/rationale, exact sidecar and source pins, and explicit
parameter bindings. The extension pins the project itself. Repeated use of one
source requires distinct instance keys; one exact file capture may support those
instances, while different paths to one file are rejected.

Bindings are `literal` with a supplied typed value and sensitivity, or `answer`
with question key, exact answer key and record hash. Every declared parameter
must be explicitly bound, including declarations with PRD-059 defaults. To use
a default value, consciously supply that value as a literal. A missing, invalid,
stale or expired answer blocks its dependent section; it is never replaced with
a literal or default. A mismatched component answer-record pin is a stale binding and blocks its
section, retaining expected and observed record hashes. A wrong answer/question
relationship remains an invalid binding. Phase 1 human-clause pin errors retain
their established behavior.
Confidential/restricted substitutions and secret-like names are rejected before
rendering, even if their section is blocked. Only public/internal values may be
substituted into Markdown. Such draft prose is intentionally visible; reports,
locks, impact and HTML contain hashes, not raw answer/parameter values. Hashes
are not encryption and can reveal low-entropy values through guessing.

The adapter uses PRD-059's grammar, parameter checks, one-pass rendering and
escaping. It adds conservative authoring restrictions: no raw HTML, code blocks,
document-wide reference definitions or unsafe links. One component may supply a
policy/topic section in this extension version. Its first H2 must exactly match
the escaped topic title. The heading is retained as supplied; it is not stripped
or rebased. Deeper headings remain inside that component, and explicitly supplied
human clauses can follow. The policy retains one generated H1 and topic ordering.
Components never set `human-draft-present` or policy approval state.

Provenance spans use zero-based UTF-8 byte offsets with exclusive ends. Source,
parameter and generated-newline origins remain distinct. Empty substitutions
retain zero-width parameter evidence. Instance evidence binds inclusion review,
exact source/sidecar pins and parameter/answer records; policy/topic references
resolve the reviewed gap/control assignments in the same graph. All referenced
components are captured and validated before output, including blocked sections.

## M-13 baseline and dependency impact

```sh
forge author impact --manifest impact.json --format json
forge author impact --manifest impact.json --output-dir impact-view --html
```

The [closed impact schema](../schemas/authoring-impact.schema.json) pins two
independently validated projects and optional component extensions:

```json
{
  "schema_version": "forge.authoring-impact/1",
  "old": {"project": {"path": "old/project.json", "expected_sha256": "<64 lowercase hex>"}},
  "new": {"project": {"path": "new/project.json", "expected_sha256": "<64 lowercase hex>"}}
}
```

Replace the explanatory hash placeholders with exact local SHA-256 values. Each
project's component pin, when present, is also relative to the impact request
root and must lie within its project directory. Identical old/new references
are supported for no-op comparisons.

Exact framework and resolved-catalog hashes permit automatic same-control-ID
correspondence. Changed framework resources require a reviewed, one-to-one
control correspondence bound to both exact hashes and any resolved-catalog
hashes. Matching IDs or UUIDs in unrelated frameworks alone are insufficient.
Without correspondence the report is unsupported and has no unaffected claims.
Every supplied correspondence must contain at least one reviewed pair and cover
both sides of every control ID surviving in both inventories, including an optional
map supplied for identical framework and resolved-catalog hashes. A supplied map
replaces automatic same-control-ID matching. Unpaired surviving
IDs produce an unsupported report. Controls absent from the opposite inventory
are explicit additions/removals, with no inferred successor. Explicit pack
control/topic dependencies remain visible even when they have no current gap.
Gap IDs include exact report hashes; comparison preserves both IDs while using
validated control correspondence. Report whitespace churn therefore produces
binding findings, not fabricated wholesale gap additions/removals.

Findings distinguish gaps, assignments, deferrals, answers, definitions, expiry,
packs, components, human clauses, policies and sections. They keep substantive
content, provenance, authoring state and exact output byte changes separate.
Affected sections come from the union of old/new explicit dependencies, including
removed or moved bindings. An unaffected section/policy has validated inputs,
identical exact output bytes and no changed relevant dependencies or state.
Global project/pack/report binding churn remains visible without claiming every
section's content changed. Provenance-only differences are still differences;
"unaffected" is not a claim that all generation artifacts are byte-identical.
The request hash binds finding identities to the explicit snapshot selection.

Missing, invalid or drifted inputs produce an incomplete comparison with closed
failure-phase metadata and no unaffected output. Both report hashes
are explicit JSON `null` for an incomplete comparison. Fully verified comparisons
retain the exact old/new 64-character lowercase-hex report hashes. The tool does not accept or
render mismatched content, update pins, migrate answers, rewrite assignments,
regenerate published drafts, or mutate lifecycle records.

Exit codes: 0 for a complete comparison without findings, 1 for complete review
work, 2 for invalid, incomplete or unsupported comparison/output. An incomplete
report may still be published as a new report generation when requested.

## Offline HTML

`--html` requires an output directory and adds `plan.html` and, on build,
`provenance.html`; impact adds `impact.html`. These views display the same
validated redacted data as JSON as escaped text. Labels, titles, rationale,
filenames and component metadata cannot create HTML elements or active URLs.
There are no scripts, forms, assets, tracking, downloads or network dependencies.
Unresolved work and drafting states remain visible, without outcome claims.

## Explicit draft lifecycle handoff

```sh
forge author handoff --manifest handoff.json --output-dir draft-records --html
```

The [handoff schema](../schemas/author-handoff.schema.json) requires project and
optional component pins, the exact previously emitted provenance, and every
emitted policy's exact bytes. Each policy also requires a human-supplied lifecycle
policy/version key, title, owner keys, parties, approval-policy role counts and
all separation options, and review dates/cadence/timezone. No lifecycle state,
history, actor event or approval assertion is accepted in the request.

The command independently replays the authoring build and compares every pinned
prior policy and provenance byte before producing PRD-058 records. Every policy
must be selected exactly once. Only valid `draft` records with empty histories
are created; component status cannot create policy approval. Ordinary author
plan/build never creates or updates lifecycle records.

The new generation contains the complete build, `<author-policy-key>.lifecycle.json`
records at its root and `handoff.json` receipt. Records use portable
`policies/<key>.md` references within that generation. They remain valid after
relocating the generation or removing the old build. The receipt binds exact
provenance, source and record hashes. Its `manifest_contract_sha256` hashes the
deterministic pretty-serialized parsed handoff contract with a trailing newline;
array ordering is retained. Authoring provenance is deliberately not
put in PRD-058 `generated_artifacts`, which requires OSCAL artifacts.

```sh
forge lifecycle check --record draft-records/sample-policy.lifecycle.json
```

Existing destinations and histories are never overwritten or transitioned.
Handoff reports drafting action work using the existing 0/1/2 authoring convention.
Exit 1 can follow successful publication when the plan still requires authoring
work; the emitted `handoff.json` receipt records the completed generation. It does
not mean the destination is absent, and retrying against it is rejected.

## Publication and limits

All artifacts belonging to one generation are prepared together and exposed by
one atomic no-replace directory rename on Linux/macOS. The parent must already
exist within the request/project root. Unsupported platforms fail closed before
publication; read-only operations remain available. Interruption may leave a
private staging directory, but the destination is absent or complete. A sync
failure after rename may report an error with the full generation visible.
Sequential writes with rollback are not described as interruption-safe atomicity.
Concurrent source changes still require external serialization.

The source budget is 50 MiB across each invocation, including both impact
snapshots or handoff's prior outputs. Duplicate capture needed for independent
validation is counted conservatively. If the old impact capture fails, its
unavailable partial byte count reserves the remaining allowance; the new snapshot
is left unverified and the comparison is incomplete. Every read receives its remaining budget
before allocation. The output generation budget is 50 MiB with at most 2,048
artifacts; serializers and escaped HTML use the remaining budget. Components
are limited to 1,000 instances, 128 bindings/gap references per instance,
100,000 provenance/dependency records, 1 MiB source files and 2 MiB extension
manifests. Handoff accepts at most 128 policies. Upstream stricter record limits
remain enforced. These are source/output byte limits, not a claim of exact heap
usage: validated representations and private snapshots may retain extra copies.

See the [interface freeze and acceptance matrix](authoring-phase2-plan.md),
[Phase 2 review dispositions](authoring-phase2-review.md) and
[Phase 1 review dispositions](authoring-phase1-review.md) for retained boundaries.

# Framework-guided policy authoring

`forge author` converts an exact, reviewed applicability/gap baseline and explicit
human assignments into a drafting plan and Markdown skeletons. Default Phase 1
builds write no substantive text beyond supplied human clause files and never
interpolate answer values. Opt-in Phase 2 components substitute only explicitly
bound public/internal values into Markdown; reports never copy raw values.

This is a technical foundation. PRD-061 readiness, human review, legal content
decisions, design-partner exercises, time-savings studies, and release gates
remain pending. Authoring states do not change PRD-058 lifecycle state.

Rust library release compatibility also remains a gate: this feature extends
the public exhaustive `ForgeError` and CLI command enums. Downstream exhaustive
matches may require new arms. A release must review the public API, document
migration, and select a semver-compatible version before publication; this
technical tranche does not claim source compatibility for those matches.

## Commands

```sh
forge author plan --manifest project.json --format text
forge author plan --manifest project.json --format json
forge author plan --manifest project.json --output-dir planning
forge author build --manifest project.json --output-dir drafts --format json
```

Both commands print the requested plan format to stdout; diagnostics use stderr.
The [synthetic example](../examples/authoring/README.md) provides a complete pinned
input set that can be copied to a new directory and built offline.
`plan` optionally writes `plan.json` and `plan.txt`. `build` requires a new output
directory and writes those reports, `provenance.json`, and one
`policies/<policy-key>.md` file per explicitly selected policy.

All input paths and output-directory paths resolve from the author manifest's
directory. Phase 1 requires `project_root` to be exactly `"."`. Supply canonical
relative descendant input paths: no absolute paths, drive prefixes, backslashes,
symlinks, hard links, or aliases. A nested PRD-056 manifest may use leading `../`
segments only when the dependency remains inside the author root. Internal
canceled descents such as `sub/../framework.json`, `./`, repeated separators, and
trailing separators are rejected before capture; discarded directory components
cannot hide aliases.
The manifest's directory ancestry must also be free of symlinks. Use the physical
project path (for example, `/private/tmp` rather than macOS's `/tmp` alias).
Remote links embedded in framework resources are never fetched.

Exit statuses:

| Code | Meaning |
|------|---------|
| 0 | Valid plan or build; every applicable gap is assigned or deferred and required context is available |
| 1 | Valid plan or build with unresolved gaps or sections blocked by required context |
| 2 | Invalid input, mismatched fingerprint, unsupported contract, unsafe path, or output failure |

Exit 0 describes authoring bookkeeping only. A skeleton without human clauses
can still be `skeleton-ready`; it is not a finished policy.

## Scaffolding an empty pack (S-1)

```sh
forge author scaffold --manifest project.json
```

`scaffold` writes an empty `forge.authoring-pack/1` template to the path pinned by
`authoring_pack` in the manifest. It captures and exactly regenerates the pinned
framework inventory, copies the project's exact baseline fingerprints, creates no
topics, questions, policy families, control assignments or family assignments, and
leaves the required `reviewers` collection and `content_rights` reviewer fields
empty. A reviewer must supply that provenance — and update the `authoring_pack`
byte pin — before `plan` or `build` accepts the pack; the template is intentionally
not yet a usable pack (the `forge mapping init` fail-until-attested convention).
The command never replaces an existing file, never follows a symlinked
destination, and creates no assignment, approval evidence or lifecycle state. The
destination's parent directory must already exist, mirroring the authoring output
boundary. It fails closed on platforms without atomic no-replace publication. The
command adds a public `AuthorCommand` variant, so it is included in the pending
public Rust API/semver and migration gate.

## Local input contracts

The Phase 1 closed JSON schemas are [authoring-pack](../schemas/authoring-pack.schema.json)
and [author-project](../schemas/author-project.schema.json). Runtime validation
also enforces cross-record references, exact hashes, logical uniqueness, value
constraints, and relationships that JSON Schema alone cannot express.

An authoring pack (`forge.authoring-pack/1`) contains:

- A stable key, version, exact baseline binding, and declared content-rights evidence.
- Framework control IDs, human-authored topic titles, policy-family keys, and questions.
- Explicit control-to-topic and topic-to-family assignments, each with reviewer,
  review time, and rationale.

An author project (`forge.author-project/1`) contains:

- A stable project key, explicit `as_of`, baseline review, and local reviewer registry.
- Byte-pinned applicability manifest, gap report, and authoring pack.
- Explicit policies, typed question answers, gap deferrals, and human clause references.

Reviewer names, keys, times, rights statements, and rationales are asserted
provenance. FORGE does not authenticate reviewers, prove independent review,
grant rights to framework content, or decide whether an assignment is appropriate.
Portable framework relationships use control IDs. Repository fixtures contain
synthetic framework content only.

## Exact baseline binding

PRD-056 emits `forge.applicability-report/1` without a report hash or gap ID.
Authoring pins SHA-256 of the **exact supplied report bytes**, including whitespace.
The framework and optional resolved Catalog companion also use exact raw-file
SHA-256. Pack and project baseline bindings must agree.

The loader securely captures the applicability manifest, framework, companion,
and Mapping Collections, then recomputes the complete unfiltered PRD-056 analysis
in a private local snapshot. The supplied report must equal that analysis after
strict JSON decoding. Filtering a report, deleting controls, changing totals,
adding unsupported fields, or pinning a fabricated report is rejected.

Applicable authoring gaps are exactly the PRD-056 categories
`applicable-unmapped` and `applicable-reviewed-no-relationship`. Mapped controls,
scope deferrals, exclusions, and unfinished applicability decisions are not
silently added to this set. Every gap is assigned to at least one explicit
policy/topic, explicitly deferred, or visibly unresolved. Multiple destinations
count once in `total = assigned + deferred + unresolved`.

Gap IDs hash a length-prefixed, versioned tuple of report SHA and control ID.
They change with an exact report revision; the original control ID remains
visible. Phase 1 rejects stale inputs but does not compare old and new baselines.

## Questions and clauses

Question types are string, integer, boolean, and string-list. Definitions include
required/optional state, owner, sensitivity, source label, optional maximum age,
and bounded validation constraints. No substantive defaults exist.

Each answer is explicitly `provided` or `no-answer`. Its question hash and pack
hash bind the answer to the reviewed definition. Evaluation uses only the
manifest's explicit `as_of`, never the current clock. Missing, stale, expired,
or invalid answers are visible and block only dependent sections. Optional
unavailable answers remain visible without blocking unrelated content. Reports
include answer-record hashes and metadata, not values.

Human clauses name a policy, topic, exact gap IDs, byte-pinned Markdown source,
and any required answer records by key and canonical SHA-256. Editing an answer
record requires an explicit update to dependent clause pins. A stale clause pin
or changed source file fails the invocation, even if that clause would otherwise
be blocked. Human clauses are validated before either planning or building.

Each policy has one generated title and ordered topic sections. Human clauses
are preserved verbatim; unsupported Markdown structure is rejected rather than
silently rewritten. Missing content is a visible unresolved marker. Authoring
states are `planned`, `blocked-context`, `skeleton-ready`, and
`human-draft-present`. None is a governance or control-outcome judgment.

## Provenance and deterministic output

`forge.authoring-provenance/1` relates exact output spans to policy/topic keys,
gap IDs, control IDs, reviewed assignment records, answer hashes, human clauses,
and all captured input hashes. Spans are zero-based UTF-8 byte offsets with an
exclusive end. They partition the Markdown bytes, including generated metadata
and separators. Human clause spans identify the exact source bytes they preserve.

Plans, skeletons, reports, and provenance exclude absolute processing paths,
temporary-directory names, wall-clock timestamps, locale, and environment data.
The same exact input bytes produce identical artifact bytes in different
directories and across repeated builds. Build to distinct output-directory names
to compare generations; destination names do not enter artifact identity.

Sensitive answers remain in the local input file and are hashed in reports.
Hashing is not encryption: low-entropy values can be guessed. Human clause text
is intentionally emitted verbatim and must be reviewed for its intended audience.

## Atomic publication and limits

The complete generation is staged under an existing, confined output parent and
published with a single atomic no-replace directory rename. The destination must
not already exist, even as an empty directory. There is no overwrite flag and no
implicit parent-directory creation. Source files are rechecked immediately before
publication. Keep concurrent edits externally serialized; input hashes identify
the captured generation, not an authenticated or continuously locked workspace.

Directory publication supports Linux and macOS. On other platforms it fails
closed before staging; read-only planning remains available. This limitation is
explicit because the existing per-file rollback writer cannot guarantee an
all-or-nothing result after interruption.

A process interruption before publication can leave a private hidden staging
directory, while the destination remains absent. After the atomic rename, the
destination is complete. A later directory-sync failure may report an error even
though a complete generation has been published; it never exposes a partial
generation. Existing output generations remain unchanged on validation failures.

Limits for the two authoring contracts include 2 MiB per manifest, 16 KiB per
string, depth 32, 256 reviewers, 1,000 topics/families/policies, 4,096
questions/answers, 10,000
assignments/deferrals/clauses, 128 references per record, 1 MiB per human clause,
and 50 MiB total captured source bytes and rendered artifacts. The schema and
runtime validators specify narrower type-specific bounds where needed.

Imported PRD-056 manifests, reports, framework resources, and Mapping Collections
retain their upstream semantic limits within the same 50 MiB total captured-input
budget. They do not inherit the narrower authoring-contract string or manifest
limits. Each confined read is bounded by the remaining aggregate budget before
allocation.

## Reuse candidates (PRD-066 Phase 0)

```sh
forge author reuse --manifest project.json --corpus corpus.json --format text
forge author reuse --manifest project.json --corpus corpus.json \
  --format json --output-dir reuse --html
```

`reuse` ranks **verbatim** blocks of operator-supplied Markdown against the
sections of the drafting plan that are not `human-draft-present`. It is
retrieval only: there is no model, no network, no credentials, and no prompt
assembly, so the report can only point at bytes the operator supplied. It never
writes into an authoring pack, project, plan, or pinned input.

The corpus (`forge.reuse-corpus/1`, [schema](../schemas/forge.reuse-corpus-1.schema.json))
names each document by a portable descendant path relative to the corpus
manifest's directory, an exact SHA-256, an operator `status` (`approved` or
`draft`), and asserted rights and source labels, with optional topic/control
hints. The corpus manifest itself is read through the confined, no-follow path
and must be a regular file of at most 2 MiB: symlinks, hard links, devices,
FIFOs and directories are rejected, and the manifest and every document are
revalidated immediately before any generation is published. Documents are
captured through the same confined, bounded path as every
other authoring input: no symlinks, hard links, aliases, escaping paths, or
stale pins. `draft` documents are excluded unless `--include-draft` is passed.

The report (`forge.authoring-reuse/1`,
[schema](../schemas/forge.authoring-reuse-1.schema.json)) carries section
metadata, every captured input with its role, path and SHA-256, and per candidate
`source_key`, `source_path`, `source_sha256`, a zero-based half-open `span`,
a six-decimal score, machine-readable `reasons` (`control-match`, `topic-terms`,
`question-terms`, `same-family`), and `low_confidence`. Rankings are
deterministic: `BM25` (`k1 = 1.2`, `b = 0.75`) over lowercased alphanumeric
tokens plus a `+2.0` exact control-ID boost and a `+0.5` shared scope-title
token boost, ordered by score, then source path, span start and source hash.
Scores are quantised to six decimals, so repeated runs and separate directories
produce byte-identical reports and published trees.

`--max-candidates` defaults to 5 and accepts at most 100; `--min-score` drops
candidates below a value and defaults to 0, while candidates scoring below
`1.0` are still emitted and flagged `low_confidence` rather than hidden. Only
`approved` documents compete by default, at most 100 blocks per document are
ranked, and at most 100 candidates are kept per section. Output artifacts
(≤ 50 MiB total, ≤ 2048 files) are published with the same single-generation
atomic no-replace rename as `plan`/`build`; `--html` adds an inert, escaped
`reuse.html` view and requires `--output-dir`.

| Code | Meaning |
|------|---------|
| 0 | Valid report; every reported section has at least one candidate |
| 1 | Valid report; some section has no candidate or only low-confidence candidates |
| 2 | Invalid input, mismatched pin, unsafe path, or output failure |

A candidate is a pointer, not a recommendation: nothing in the report asserts
that a candidate is correct, current, applicable, or approved. Ranking is a
lexical heuristic and its score is not a quality claim. Adopting a candidate
remains a manual act — a human writes the bytes into a clause file the way they
do today. The retrieval-first tranche is described in the
[MVP reuse plan](plans/2026-09-11-066-mvp-reuse-plan.md); generation, critique
and provider phases remain in [PRD-066](PRD/066-prd-ai-assisted-suggestions.md).

## Phase 2 extensions and remaining scope

Explicit component instances, M-13 impact comparisons, offline HTML, and draft-only
lifecycle handoff are described in [Phase 2 usage](authoring-phase2.md). These
opt-in contracts preserve the Phase 1 inputs and default command behavior above.
Phase 1 stale-input rejection alone did not implement M-13; its technical evidence
belongs to the Phase 2 comparison matrix. Phase 3 design-partner packs, measured
authoring exercises, legal/content acceptance, human readiness, pilot metrics and
release approval remain pending. AI, web editing, remote registries, downloads,
connectors, collaboration and hosted services remain outside this tranche.

See [the implementation plan](authoring-phase1-plan.md) for interface ownership,
requirement disposition, and the verification/delivery workflow.
The [review dispositions](authoring-phase1-review.md) record confirmed fixes and
source-backed explanations for the initial exact-commit OCR findings.

The optional Phase 2 [component extension](../schemas/author-components.schema.json),
[impact request](../schemas/authoring-impact.schema.json) and
[handoff request](../schemas/author-handoff.schema.json) have separate closed
schemas. Runtime checks additionally enforce byte limits (JSON Schema string
lengths count characters), exact pins, cross-record relationships and valid dates;
see [Phase 2 contracts](authoring-phase2.md).

The [Phase 2 interface matrix](authoring-phase2-plan.md) and
[Phase 2 review dispositions](authoring-phase2-review.md) retain implementation,
review and compatibility evidence.

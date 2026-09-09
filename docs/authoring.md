# Framework-guided policy authoring — Phase 1

`forge author` converts an exact, reviewed applicability/gap baseline and explicit
human assignments into a drafting plan and Markdown skeletons. It writes no
substantive text beyond the human clause files supplied by the project. Answer
values are never interpolated into Markdown or copied into reports.

This is a technical foundation. PRD-061 readiness, human review, legal content
decisions, design-partner exercises, time-savings studies, and release gates
remain pending. Authoring states do not change PRD-058 lifecycle state.

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
symlinks, hard links, or aliases. A nested PRD-056 manifest may reference a parent
directory only when the normalized dependency remains inside the author root.
Remote links embedded in framework resources are never fetched.

Exit statuses:

| Code | Meaning |
|------|---------|
| 0 | Valid plan or build; every applicable gap is assigned or deferred and required context is available |
| 1 | Valid plan or build with unresolved gaps or sections blocked by required context |
| 2 | Invalid input, mismatched fingerprint, unsupported contract, unsafe path, or output failure |

Exit 0 describes authoring bookkeeping only. A skeleton without human clauses
can still be `skeleton-ready`; it is not a finished policy.

## Local input contracts

The closed JSON schemas are [authoring-pack](../schemas/authoring-pack.schema.json)
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

Limits include 2 MiB per author manifest, 16 KiB per string, depth 32, 256
reviewers, 1,000 topics/families/policies, 4,096 questions/answers, 10,000
assignments/deferrals/clauses, 128 references per record, 1 MiB per human clause,
and 50 MiB total captured source bytes and rendered artifacts. The schema and
runtime validators specify narrower type-specific bounds where needed.

## Remaining scope

M-13 baseline impact remains an unchecked Phase 2 requirement despite its MVP
Must Have label in PRD-061. PRD-059 component rendering, static HTML, PRD-058
lifecycle handoff, and design-partner exercises are also deferred. Phase 1
integrity checks are not a substitute for baseline impact analysis. AI, web
editing, remote registries, downloads, connectors, collaboration, and hosted
services are outside this tranche.

See [the implementation plan](authoring-phase1-plan.md) for interface ownership,
requirement disposition, and the verification/delivery workflow.

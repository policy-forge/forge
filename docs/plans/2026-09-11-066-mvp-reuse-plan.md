# 066 MVP: deterministic reuse candidates for unresolved authoring sections

Status: implementation plan for review. No implementation, release, or acceptance claim.
Owner: Brian Luby. Author: coordinating agent, 2026-09-11.

This plan carves a first, deliberately narrow tranche out of
[PRD-066](../PRD/066-prd-ai-assisted-suggestions.md): **retrieval of previously
human-authored text**, not generation. It is the "ground before generate" layer
the PRD needs anyway, and it is the only part of 066 that can ship without a
network dependency, a provider decision, or a conflict with PRD-061 M-9.

## Decision summary

- The MVP returns **verbatim, byte-exact excerpts** of sources the operator
  supplies. It never rewrites, summarizes, paraphrases, or merges text, so
  synthesised prose is impossible by construction rather than by policy.
- **No model, no network, no credentials, no prompt assembly.** Ranking is a
  deterministic lexical score over operator-supplied documents.
- The input is the authoring plan that already ships (`forge author plan`), so a
  candidate set is produced for exactly the sections that still need human text.
- Adoption stays manual: a tester reads a candidate and writes it into a clause
  file the way they do today. Nothing writes into a pack, project, or manifest.
- The generation, critique and provider layers stay in PRD-066 as later phases
  and are listed explicitly under "Not in this MVP".

## Why this shape

1. **It attacks the real remaining cost.** PRD-061's pipeline already emits
   deterministic skeletons; what is left for the operator is prose. Reuse from
   the organisation's own approved policies is the cheapest way to supply some
   of it, and it is what a careful author does by hand today.
2. **It is M-9-safe.** Every returned byte exists in a source the operator
   supplied, with an exact span and hash. Nothing in the output is new text, so
   the "approved content only" rule is untouched and no amendment is required
   for this tranche to be correct.
3. **It is a strict subset of the eventual AI path.** Corpus pinning, ranking,
   span grounding, candidate records and the quarantine-style report are all
   reusable when generation is added; nothing here is throwaway.

## In the MVP

- `forge author reuse --manifest <project.json> --corpus <corpus.json>` with
  `--format text|json`, optional `--output-dir`, optional `--html`,
  `--max-candidates`, `--min-score`, `--include-draft`.
- A closed, bounded `forge.reuse-corpus/1` manifest naming operator-supplied
  candidate documents (portable relative paths, exact SHA-256 pins, rights
  label, status label, optional topic/control hints).
- A closed, bounded `forge.authoring-reuse/1` report: per unresolved section,
  ranked candidates with source path, exact UTF-8 byte spans, source hash,
  score, and machine-readable match reasons.
- Deterministic ranking and byte-identical reports across repeated runs and
  across directories.
- A text report and, behind `--html`, a static escaped view reusing the
  existing deterministic HTML encoder.
- Exit codes consistent with authoring: `0` every section has at least one
  candidate, `1` valid report with sections that have none or only
  low-confidence candidates, `2` invalid input or unsafe output.
- Tests proving span exactness, absence of generated text, determinism, bounds,
  path safety, and unchanged behaviour of every existing command.

## Not in this MVP

Explicit exclusions. Each is deferred, not rejected; the seam is named where it
matters so the later phase does not require rework.

**Models and AI**

- No LLM or any model call; no provider configuration, API keys, endpoints,
  model names, or `[ai]` config section.
- No generation, completion, rewriting, summarising, paraphrasing, merging, or
  "suggested wording" of any kind.
- No adversarial, critique, QA, or self-consistency model passes. The
  generator/critic design stays in PRD-066 Phase 1+; the report schema reserves
  no critique field in `/1` (a later `/2` adds it).
- No embeddings, vector store, semantic search, or model files. Ranking is
  lexical and deterministic.
- No training, telemetry, prompt/response logging, or corpus upload.
- No AI-origin or adoption-attribution fields in `/1`; they belong to the phase
  that can produce non-human text.

**Network and supply chain**

- No outbound HTTP/TLS client and no new crate dependency. `Cargo.toml` and
  `Cargo.lock` are unchanged by this tranche.
- No remote corpus fetch, no provider connectivity check, no egress allowlist
  machinery, no consent-token or payload-preview flow (they exist for egress
  that does not occur here).
- No change to the package version; the tranche lands inside the already
  committed `2.0.0` release line documented in
  [authoring-api-migration](../authoring-api-migration.md).

**Authoring and approval systems**

- No automatic adoption: nothing writes into `forge.authoring-pack/1`,
  `forge.author-project/1`, the plan, or any pinned input; no pin rewriting and
  no manifest editing.
- No materialising a candidate into a clause file in this tranche (decision D2).
- No PRD-059 component ranking, parameter binding, or component changes; the
  component path is read-only if a corpus entry happens to reference one.
- No PRD-058 lifecycle records, no approval or transition of any kind.
- No PRD-055 mapping changes and no PRD-056 baseline changes.

**Other product surfaces**

- No PRD-062 workspace integration (no API route, no openapi change, no UI
  change), no PRD-067 MCP tool, no PRD-068 review queue, no PRD-065
  integrations, no PRD-064 POA&M.
- No cross-project or portfolio reuse, no shared or hosted corpus, no
  multi-user anything, no audit-service.
- No interactive HTML, no JavaScript, no editor, no diff view. HTML, when
  requested, is static escaped text with no scripts, forms, URLs or remote
  assets.

**Artifacts and claims**

- No XML or YAML report formats; JSON and text, plus optional HTML.
- No release approval, no compliance/effectiveness/approval language, and no
  "validated" or "approved" state for a candidate. A candidate is a pointer to
  text, with a score.
- No pilot metrics claimed as results: the tester programme produces evidence
  for the PRD-066 gates, never acceptance.

## Frozen contracts

| Item | Value |
|---|---|
| New input | `forge.reuse-corpus/1`, closed and bounded, operator-supplied |
| New output | `forge.authoring-reuse/1`, closed and bounded |
| Unchanged inputs | `forge.author-project/1`, `forge.authoring-pack/1`, `forge.authoring-plan/1` and every `/2` component contract |
| Command | `forge author reuse` (new `AuthorCommand` variant) |
| Exit codes | `0` / `1` / `2` as above, matching existing authoring semantics |
| Determinism | no wall clock, locale, environment, or absolute path in identity or bytes |

Corpus entry fields: `key`, `path`, `title`, `status` (`approved` | `draft`),
`rights_label`, `source_label`, `expected_sha256`, optional `topic_keys`,
`control_ids`, optional `supersedes`. Documents are Markdown; only
heading-scoped blocks are ranked.

Report shape: `schema_version`, `project_key`, `as_of`, `corpus_sha256`,
`inputs` (every captured file with role, portable path, SHA-256, byte length),
`sections` (policy key, topic key, title, state, `gap_ids`, `control_ids`,
`candidates`), and `counts` (`sections`, `with_candidates`, `low_confidence`,
`candidates`). Each candidate carries `source_key`, `source_path`,
`source_sha256`, `span` (zero-based, exclusive end), `score`, `reasons`
(`control-match`, `topic-terms`, `question-terms`, `same-family`), and
`low_confidence`.

## Algorithm and grounding rules

1. Capture the project, its pinned inputs, and every corpus file through the
   existing confined, bounded capture path; reject pins, aliases, symlinks and
   non-portable paths as authoring already does.
2. Segment each corpus document into blocks: a heading establishes a scope, and
   a block is a maximal run of non-blank lines inside it. Record exact byte
   spans. Reject documents whose text is not valid UTF-8.
3. Build the query per unresolved section from the plan: topic title tokens, the
   prompts of that section's evaluated questions, and its control IDs.
4. Score each block with BM25 (`k1 = 1.2`, `b = 0.75`) over lowercased
   alphanumeric tokens, plus fixed boosts: +2.0 for an exact control-ID match in
   the block, +0.5 when the block's scope title shares a token with the topic
   title. Scores are quantised to six decimals so the bytes are stable.
5. Sort by score descending, then source path, span start, and source SHA-256.
   Keep the top `--max-candidates` (default 5). All blocks of an `approved`
   entry compete; `draft` entries are excluded unless `--include-draft` is
   passed. A candidate below the floor (1.0) is emitted with
   `low_confidence: true` rather than hidden.
6. Order output by `(policy key, section order, topic key)`.

Grounding invariant: the report contains candidate metadata and **verbatim
spans only**. A test asserts each span's bytes equal the source bytes at those
offsets, and a second test asserts every string value in the report is either a
declared metadata value or a verbatim corpus span, which is what makes
"never synthesise prose" mechanically checkable rather than aspirational.

## Provenance and trust boundary

Every candidate points at exact source bytes with the source file's hash, so a
reader can verify the text without trusting FORGE. Nothing in the report asserts
that a candidate is correct, current, applicable, or approved: `status` is the
operator's label on their own corpus, and `rights_label` is an assertion, not a
grant. Ranking is a lexical heuristic and its score is not a quality claim.

## Safe I/O and bounds

- Reports are published with the existing single-generation atomic no-replace
  directory publish; existing destinations are rejected; unsupported platforms
  fail closed.
- Bounds: corpus manifest 2 MiB, 512 documents, 1 MiB per document, 50 MiB
  aggregate capture, 16 KiB per string, 100 blocks per document ranked, 100
  candidates per section, 50 MiB per output generation.
- No absolute paths, no terminal control characters, no raw HTML in text
  reports; HTML output is escaped and inert.

## PRD-061 M-9 reconciliation

This tranche needs **no** M-9 change: it returns only supplied bytes. The
amendment below is required before the *generation* phase, and is recorded here
so it is not rediscovered later.

Current M-9: "Approved content only: Include only explicitly supplied human
clause files or hash-pinned PRD 059 component instances; never synthesize
prose."

Proposed wording to add: "Reuse and suggestion features may return only
verbatim, span-exact excerpts of explicitly supplied sources, or candidates a
human explicitly adopts by supplying the exact bytes as a clause file or
hash-pinned component. No tool writes prose into a pack, project, or plan. When
a human adopts machine-authored text, the artifact records the adoption event
and the AI origin (provider, model, version, parameters, prompt hash) and never
presents the text as human-authored."

## Verification plan

| Matrix row | Required executable cases |
|---|---|
| Legacy behaviour | Full locked suite unchanged; existing `author plan/build/scaffold/impact/handoff` byte-identical outputs; no new default artifacts |
| Closed contracts | Unknown/duplicate/null/forward-version rejection for corpus and report; schema/runtime agreement; bounds and string limits |
| Corpus handling | Missing/stale/oversized/aliased/symlinked corpus entries; duplicate keys; non-UTF-8; draft exclusion and `--include-draft` |
| Ranking | Stability under document and block reordering; tie-break order; boost effects; `--max-candidates`; low-confidence marking |
| Grounding | Every span maps to exact source bytes; report contains no non-supplied text; corrupt corpus cannot yield a well-formed span |
| Determinism | Repeated runs and separate directories produce byte-identical reports and published trees |
| Safe I/O | Overwrite refused; missing parent refused; traversal/alias/symlink rejected; aggregate and per-file bounds; interruption leaves absent-or-complete destination |
| Reports | Text/JSON parity of facts; HTML inert, escaped, deterministic, no active content; exit 0/1/2 for each documented case |
| M-9 boundary | No write path into pack/project/plan; report contains no adopted or generated text |
| Delivery | `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, `git diff --check` |

## Tester programme

Who: the three design-partner reviewers already recorded in
[authoring-gates](../authoring-gates.md) (GATE-PARTNERS), plus the maintainer.

What each tester does, on a real project of their own:

1. Assemble a corpus from policies they already approved, with rights and
   status labels.
2. Run `forge author reuse --manifest project.json --corpus corpus.json
   --format json --output-dir reuse`.
3. Read the text report and, for each candidate, mark useful / irrelevant /
   wrong source. Adopt anything they want by writing a clause file as they do
   today, then run `forge author plan` and compare unresolved counts.
4. Return the report JSON (no confidential prose is required: the report
   carries paths, hashes, spans and scores) plus the ratings.

What we measure (hypotheses, not results): sections with at least one candidate;
candidates rated useful; low-confidence rate; unresolved-section count before
and after adoption. Stage exit criteria: every tester completes one project,
zero span mismatches, and the never-synthesise test holds on every report —
documented as technical evidence with the measured ratings, and nothing more.

Feedback capture: a short issue-template form in the tester repository, plus an
opt-in plain-text summary. No telemetry, no automatic upload.

## Tasks

| ID | Task | Verification |
|---|---|---|
| T1 | `forge.reuse-corpus/1` closed schema, typed model, bounds and error taxonomy | Unit tests for every rejection; schema/runtime agreement test |
| T2 | Confined corpus capture reusing the authoring capture path | Alias/symlink/pin/oversize traversal suite |
| T3 | Segmentation with exact spans; UTF-8 and heading-scope handling | Span-exactness tests on synthetic and CJK/CRLF fixtures |
| T4 | Deterministic BM25 ranking, boosts, tie-breaks, low-confidence marking | Reorder-stability and golden-score tests |
| T5 | `forge.authoring-reuse/1` report (text/JSON) and `forge author reuse` wiring | CLI end-to-end tests, exit-code matrix, no-new-default-artifact check |
| T6 | Publication, `--html`, bounds, escaping | Safe-I/O suite, inert-HTML test, cross-directory byte-equality |
| T7 | Never-synthesise and legacy-equivalence assertions | Failing-then-passing tests named in the matrix |
| T8 | Docs: PRD-066 Phase 0 carve-out, usage guide, gates, changelog and roadmap rows | Doc review; links resolve |

T1–T4 are independent of T5–T6 and can proceed in parallel once T1 is frozen.
Sequencing gate: T5 starts only after T1's contract is frozen, so the report
schema is not designed twice.

## Open decisions

- **D1** Command placement: `forge author reuse` (proposed) or a top-level
  `forge reuse`. Affects the public `AuthorCommand` enum and the API gate.
- **D2** Whether `--emit <candidate> --output <file>` (materialise a candidate
  as a new clause file, atomic no-replace) is in this tranche or the next.
  Proposed: next, to keep every write path out of the first tester build.
- **D3** Corpus manifest name and whether PRD-059 components may be named as
  corpus entries in `/1` (proposed: no; components are ranked in a later phase
  because they carry parameter bindings).
- **D4** The low-confidence floor value (proposed: 1.0) and default
  `--max-candidates` (proposed: 5).
- **D5** Where this tranche's gates are recorded: a `GATE-REUSE-*` section in
  `docs/authoring-gates.md` (proposed) or a separate register.
- **D6** Whether the tester form lives in this repository or the testers'.

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-09-11 | coordinating agent | Initial MVP plan: deterministic reuse candidates, explicit exclusions, verification and tester programme |

# PRD-061 Phase 2 interface freeze and acceptance matrix

Status: local implementation checks passed; external review pending. No release or human acceptance claim.

The coordinator fetched origin and verified `origin/main` at
`32ed4efc19798efde594012364780bbab906c29d`, the live merge of PR #144.
The Phase 1 files and final review dispositions are present. The starting
checkout was clean; existing worktrees remain intact. Work uses isolated
`/private/tmp/forge-prd061-phase2`, branch
`codex/061-guided-policy-authoring-phase2`. Baseline locked tests passed:
2,234 passed, three ignored. Three focused agents completed read-only discovery
before this freeze. Only the coordinator performs Git mutations; substantial
Cargo processes run sequentially.

## Versioning and frozen boundaries

Existing `forge.author-project/1`, `forge.authoring-pack/1`, plan/provenance `/1`
inputs and default outputs retain their meaning and bytes. Components use a
separate closed `forge.author-components/1` extension explicitly selected with
`--components`. It pins the exact project bytes. Component-enabled plans and
provenance use explicit `/2` output contracts, never extra fields disguised as
`/1`. Existing author plan/build CLI and 0/1/2 exits remain: action work is 1,
invalid input/output is 2. New flags on plan/build: `--components FILE`, `--html`
(requires an output directory). Impact and handoff use closed manifests:
`forge author impact --manifest FILE [--format text|json] [--output-dir DIR]
[--html]`; `forge author handoff --manifest FILE --output-dir DIR [--html]`.

No implicit migration, pin refresh, answer update, or assignment rewrite occurs.
New exhaustive public CLI variants require downstream match changes. Public
Rust API review, migration guidance, and a semver-compatible release decision
remain pending. Package version and dependency lockfile are not changed.

## Agent ownership and shared APIs

- Components agent: new `components.rs`, `component_model.rs`, component schema
  and focused tests, plus pure fragment adapter in `policy/render.rs`.
  `prepare(bytes, loaded, plan, pinned_read_callback) -> LoadedComponents`;
  `LoadedComponents::apply_plan(&mut AuthoringPlan)`. Redacted evidence and
  optional fragment bytes/spans are separate. No publication.
- Impact agent: `impact.rs`, impact schema and focused tests. Owns closed
  manifest, snapshot/report types, pure comparison and text/JSON rendering.
  Coordinator supplies independently validated loaded projects, plans, rendered
  per-policy/section byte digests, canonical control fingerprints and redacted
  component evidence. Incomplete comparisons contain no unaffected claims.
- Presentation/handoff agent: `html.rs`, `handoff.rs`, handoff schema and focused
  tests. HTML consumes only validated redacted report/provenance data. Handoff
  parses its closed manifest and prepares draft records and a hash-bound receipt
  from revalidated exact generation bytes, without writing.
- Coordinator: `input.rs`, `render.rs`, `mod.rs`, CLI, publication orchestration,
  complete-generation bounds, integration tests, docs, roadmap and all Git work.

## Component rules

Stable instance keys identify explicit policy/topic/gap assignments. Sidecar
and source pins, reviewer/time/rationale, and every parameter binding are
required. Literals explicitly declare public/internal sensitivity; answer
bindings name question key, answer key and exact record hash. Missing records and stale component answer pins
block dependent sections; wrong question relationships and source pins fail. Confidential/restricted
bindings and secret-like parameter names fail before rendering. Reports contain
hashes and provenance, never values. All declarations require explicit bindings,
including PRD-059 defaults; defaults cannot satisfy missing context.

Reuse PRD-059 tokenization, validation, escaping, and one-pass substitution.
At most one component per policy/topic in this extension version; its first H2
must exactly match the generated topic heading and supplies that section's
heading without rebasing or discarding structure. One policy H1 remains.
Human clauses may follow. All referenced sources and grammar are validated even
for blocked sections. Repeated components need distinct instance keys.

## M-13 comparison rules

Snapshots pin project and optional component extension independently. Exact
framework and resolved-catalog hashes permit automatic same-control-ID
correspondence. Changed framework resources require explicit reviewed one-to-one
control correspondence bound to both exact resources, otherwise comparison is
unsupported. Report hash churn alone never creates added/removed gaps. Findings
retain old/new gap IDs and report hashes. Compare substantive dependencies,
provenance bindings, authoring state, and exact output bytes separately. Global
binding churn is reported independently of local affected sections. Unaffected
requires validated inputs and identical section/policy bytes and dependencies.
Missing/drifted input never produces an unaffected conclusion or renders the
mismatched content. Comparison never changes pins, drafts or lifecycle state.

## HTML and lifecycle publication

HTML is deterministic escaped text with no scripts, forms, URLs, remote assets,
tracking or Markdown execution. It never consumes raw answers or draft prose.
Handoff explicitly pins project, optional components, emitted provenance and
policy bytes, and supplies policy/version identity, parties, owners, approval
configuration and dates. Rebuild in memory and compare exact bytes before
preparing records. Only PRD-058 draft records with empty history are allowed.
Records at generation root retain `policies/<key>.md` references. A separate
receipt binds provenance and record hashes because PRD-058 generated artifacts
require OSCAL, not authoring provenance. No approval evidence is created.

Stage the complete generation (reports, Markdown, locks, provenance, optional
HTML and handoff records) together, then use the existing single no-replace
directory rename on Linux/macOS. Other platforms fail closed. No overwrite,
history rewrite, partial multi-policy set, or sequential-rollback atomicity
claim. Preserve aggregate input/output, record and span bounds and portable
slash-separated labels.

## Acceptance matrix

| Capability | Required executable cases |
|---|---|
| Legacy behavior | Existing CLI suite and synthetic example exact repeated/cross-directory artifacts; no new default artifacts |
| Closed contracts | Unknown/decoded duplicate/null/forward-version rejection; schema/runtime agreement; bounded arrays, strings, graphs |
| Components | Exact sidecar/source drift; blocked-section drift; explicit defaults; missing/stale/invalid/expired answer dependency; confidential and secret names; repeated instances; Unicode; one-pass substitutions; unsupported contexts and heading mismatch |
| Provenance | Exact UTF-8 partition, source and parameter spans, answer and assignment records, every input hash, repeated-instance distinction, no value disclosure |
| M-13 | No-op, report reformat, unrelated frameworks, reviewed correspondence, added/removed gaps, assignments/deferrals, answer value/definition/expiry, unused question, pack, component content/binding/drift, human clause, policy additions/removals, moved dependencies, unaffected sections |
| HTML | Injection in every label/title/rationale/path/metadata, Unicode, bounded expansion, no active content or sensitive values, same data as reports |
| Handoff | Exact-byte tampering, supplied identity/dates/parties/approval rules, deterministic two-policy draft records, empty history, normal lifecycle check acceptance, retained source refs, existing records unchanged |
| Safe I/O | Symlink/hard-link/case/Unicode aliases, traversal, overwrite, missing parent, aggregate budget, interruption before/after single publish, multi-output failure atomicity |
| Delivery | fmt, strict all-target clippy, locked full tests, diff check, independent complete-diff review, exact-commit OCR positive selection and complete terminal coverage, exact-head CI and thread disposition |

M-13 technical completion requires the complete comparison test matrix; Phase 1
stale rejection is insufficient. Phase 3 packs, measured exercises, legal/content
acceptance, human readiness, pilot metrics and release approval remain pending.

## Local technical evidence

The integrated candidate passed formatting, strict locked all-target clippy,
2,300 locked tests (three ignored), and diff whitespace checks. M-13 has explicit
unit/CLI cases for every change category, including raw report/extension churn,
private incomplete component inputs, reviewed framework correspondence and
unchanged local dependencies. Exact commit OCR and hosted checks remain separate
delivery evidence; local green checks do not establish merge or release.

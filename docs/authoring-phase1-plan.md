# PRD-061 Phase 1 implementation plan

Status: In Progress — Phase 1 technical foundation. Product, compliance, legal,
security, engineering acceptance, design-partner, pilot, and release gates remain
pending. Reviewer metadata records assertions, not authenticated identity or
independent review. This work does not transition PRD-058 lifecycle state.

## Scope and M-13 disposition

Implement local plans, questions, Markdown skeletons, explicitly pinned human
clause files, exact baseline binding, provenance, and safe output. PRD-061 M-13
is listed as an MVP Must Have but impact analysis is allocated to Phase 2.
For this tranche, the explicit Phase 2 allocation governs: M-13 remains unchecked
and deferred. Input drift rejection is an integrity check, not baseline impact
analysis. M-9's PRD-059 component branch, static HTML, and lifecycle handoff also
remain deferred. No whole-PRD or product-release completion claim is warranted.

## Verified starting point and sequencing

The coordinator fetched origin and verified local main and origin/main at the
same live commit, `0ce98cf81832ef33ecfa7ab11cef78c0c800ed92`, with a clean main
checkout. This records an observation, not an expected baseline for future work.
The branch `codex/061-guided-policy-authoring-phase1` was created from that
fresh origin/main in an isolated worktree. Baseline `rtk cargo test --locked`
passed with 2,114 tests and 3 ignored before parallel implementation started.
All three agents completed a read-only first
pass before this interface freeze. Only the coordinator stages, commits,
rebases, pushes, or opens the PR; Cargo builds run sequentially.

Before final verification, origin advanced to
`150115247f39f63ed449b2e10ec61c4a1d1de1c3`. The isolated branch was fast-forwarded
to that freshly fetched baseline while preserving the authoring work. The
complete integrated tree passed formatting, strict clippy, and the locked test
suite (2,205 passed, 3 ignored) before candidate commits were created.

## Existing contracts and reuse

- `applicability::model::ApplicabilityReport` is the existing typed, serialize-only
  `forge.applicability-report/1`. It has a raw applicability manifest SHA,
  `ResourceEvidence`, mappings, reviewers, counts, filters, controls, and queue.
  It has neither a report fingerprint nor gap IDs. JSON uses pretty typed
  serialization with one trailing LF.
- Authoring's report fingerprint is SHA-256 of the exact supplied report bytes.
  Recompute a full, unfiltered PRD-056 report from securely captured inputs and
  compare strict-decoded JSON values. This rejects fabricated, filtered, missing,
  stale, unknown-field, and inconsistent reports without creating a second
  applicability model. Raw-byte pins remain separate from semantic comparison.
- The applicable-gap set is exactly `applicable-unmapped` plus
  `applicable-reviewed-no-relationship`. Other applicability categories remain
  baseline evidence and are not silently converted into authoring gaps.
- Gap IDs use SHA-256 of a versioned, length-prefixed tuple containing
  `forge.authoring-gap/1`, report SHA, and control ID. They change when the exact
  baseline changes; control IDs remain separately visible.
- Reuse `json_strict` for duplicate decoded keys/depth/count/string bounds,
  `hashing::sha256_hex`, typed Serde JSON, and portable path validation patterns.
- Existing `policy::commit_outputs` has sequential renames and best-effort
  rollback; it cannot guarantee interruption atomicity. Authoring publishes a
  complete generation directory in one no-replace atomic rename. Existing output
  directories are rejected, including empty directories. Unsupported platforms
  fail closed, with explicit documentation and tests.
- PRD-056 accepts paths outside its manifest directory. Authoring must capture
  every dependency through confined reads and run PRD-056 against a private
  snapshot preserving relative paths; it must not delegate containment to the
  older loader. `linkage::open_confined_evidence` provides the existing held-handle
  traversal pattern. No source/target resource href is dereferenced.

## Frozen ownership

| Owner | Files |
|-------|-------|
| Agent 1 — contracts | `src/authoring/manifest.rs`, `src/authoring/model.rs`, `schemas/authoring-pack.schema.json`, `schemas/author-project.schema.json`, `tests/fixtures/authoring/contracts/` |
| Agent 2 — planning | `src/authoring/plan.rs`, `src/authoring/report.rs`, their inline unit tests |
| Agent 3 — rendering/output | `src/authoring/render.rs`, `src/authoring/output.rs`, their inline unit tests |
| Coordinator | `src/authoring/mod.rs`, `src/authoring/input.rs`, CLI/error/lib integration, confined-read wrapper in `src/linkage/mod.rs`, `tests/authoring_cli_test.rs`, shared test helpers, synthetic examples, usage/roadmap/PR documentation |

Agents must request interface changes before altering another owner's contract.
Agent 1 materializes shared types first; other agents implement against this
freeze and the resulting type definitions. No new crate dependency is planned.

## Frozen input types

All input structs are closed, including nested structs, and use snake_case JSON
fields. Enums use kebab-case. Keys are bounded lowercase ASCII kebab-case;
framework control IDs retain validated framework spelling. Arrays reject
duplicate logical relationships as well as duplicate stable keys.

- `PinnedFile { path: PathBuf, expected_sha256: String }`.
- `BaselineBinding { framework_sha256: String, resolved_catalog_sha256:
  Option<String>, report_sha256: String }`.
- `Reviewer { key: String, name: String }` and `Review { reviewer_key: String,
  reviewed_at: String, rationale: String }`. Pack and project reviewer registries
  are separate. All references resolve in their own registry.
- `ContentRights { source_label: String, statement: String, review: Review }`.
- `AuthoringPack { schema_version, pack_key, version, baseline, reviewers,
  content_rights, topics, policy_families, questions, control_assignments,
  family_assignments }` with schema `forge.authoring-pack/1`.
- `Topic { key, title, order: u32, question_keys: Vec<String> }`;
  `PolicyFamily { key, title }`.
- `Question { key, prompt, question_type, required: bool, owner, sensitivity,
  source_label, max_age_days: Option<u32>, constraints: QuestionConstraints }`.
  `question_type` serializes as `type`: string, integer, boolean, string-list.
  Sensitivity: public, internal, confidential, restricted. Constraints contain
  optional min/max length, min/max integer, min/max items, bounded regex, and
  typed allowed values. No defaults, conditions, interpolation, or inference.
- `ControlAssignment { key, control_id, topic_key, review }`;
  `FamilyAssignment { key, topic_key, policy_family_key, review }`.
- `AuthorProject { schema_version, project_key, project_root: PathBuf, baseline,
  applicability_manifest: PinnedFile, gap_report: PinnedFile,
  authoring_pack: PinnedFile, as_of: String, reviewers, baseline_review: Review,
  policies, answers, deferrals, human_clauses }` with schema `forge.author-project/1`.
  Phase 1 requires `project_root` exactly `"."`, making the manifest directory
  the unambiguous root for both the manifest and every direct dependency.
- `Policy { key, policy_family_key, title }`; one project policy per family.
  Sections exist only through explicit pack topic-to-family relationships.
- `Answer { key, question_key, question_sha256, authoring_pack_sha256, owner,
  source_label, sensitivity, review: Review, expires_at: Option<String>,
  state: AnswerState, value: Option<serde_json::Value> }`.
  State: provided or no-answer. A provided answer has a bounded supported value;
  no-answer has no value. Type/constraint-invalid values are evaluated as invalid,
  while malformed contracts fail. Owner/sensitivity mismatch, question/pack
  staleness, future review time, expiry and age are explicit evaluation outcomes.
- `Deferral { key, gap_id, review, revisit_date: Option<String> }`.
- `AnswerPin { answer_key, expected_sha256 }`;
  `HumanClause { key, policy_key, topic_key, gap_ids: Vec<String>,
  answer_refs: Vec<AnswerPin>, source: PinnedFile, review: Review }`.
  Every clause pins the complete answer records it depends on. Clause references
  must be valid for that explicit policy/topic/gap relationship.

Canonical question and answer hashes use compact serialized JSON values with
recursively sorted object keys, domain-separated by `forge.authoring-question/1`
and `forge.authoring-answer/1`. Raw pack/project/file hashes always bind exact
bytes. A changed inline answer requires an explicit matching clause answer pin;
it is never silently substituted into an existing human clause.

Conservative limits: 2 MiB per manifest, depth 32, 16 KiB per string, 256
reviewers, 1,000 topics/families/policies, 4,096 questions/answers, 10,000
assignments/deferrals/clauses, 128 references per record, 1 MiB per clause,
50 MiB total captured source bytes and 50 MiB total rendered artifacts. Add
explicit count/regex bounds wherever these composite limits are insufficient.

## Frozen internal and report interfaces

Agent 1 defines shared `model.rs` types. Coordinator prepares
`LoadedAuthorProject { project: AuthorProject, pack: AuthoringPack,
baseline_report: ApplicabilityReport, inputs: Vec<InputFingerprint>,
project_sha256: String, pack_sha256: String, report_sha256: String,
clauses: BTreeMap<String, LoadedClause> }`.
`LoadedClause { source: HumanClause, bytes: Vec<u8> }` and
`InputFingerprint { role: String, path: String, sha256: String, byte_length: u64 }`
carry only normalized project-relative labels in serializable evidence.
Held files and root paths stay in coordinator-only preparation state.

`plan::build_plan(&LoadedAuthorProject) -> Result<AuthoringPlan, ForgeError>`.
The shared report types are:

- `AuthoringPlan { schema_version: String, project_key, as_of, provenance:
  PlanProvenance, counts: GapCounts, gaps: Vec<GapPlan>, policies: Vec<PolicyPlan>,
  questions: Vec<QuestionEvaluation>, unresolved_questions: Vec<QuestionEvaluation>,
  unresolved_gaps: Vec<GapPlan> }`.
- `PlanProvenance { project_sha256, pack_sha256, report_sha256,
  framework: ResourceEvidence, inputs: Vec<InputFingerprint>,
  baseline_review: Review, pack_reviewers: Vec<Reviewer>,
  project_reviewers: Vec<Reviewer> }`.
- `GapCounts { total, assigned, deferred, unresolved }` (usize).
- `GapPlan { gap_id, control_id, classification: String, disposition:
  GapDisposition, assignments: Vec<GapAssignment>, deferral: Option<Deferral> }`.
- `GapAssignment { policy_key, topic_key, control_assignment: ControlAssignment,
  family_assignment: FamilyAssignment }`.
- `PolicyPlan { policy_key, policy_family_key, title, state: DraftState,
  sections: Vec<SectionPlan> }`.
- `SectionPlan { topic_key, title, order: u32, state: DraftState,
  gap_ids: Vec<String>, control_ids: Vec<String>, assignments: Vec<GapAssignment>,
  questions: Vec<QuestionEvaluation>, clause_keys: Vec<String> }`.
- `QuestionEvaluation { question_key, question_sha256, required: bool, owner,
  sensitivity, source_label, answer_key: Option<String>, answer_sha256:
  Option<String>, state: AnswerStatus, review: Option<Review>,
  expires_at: Option<String> }`.
- `DraftState`: planned, blocked-context, skeleton-ready, human-draft-present.
  `GapDisposition`: assigned, deferred, unresolved. `AnswerStatus`: available,
  missing, no-answer, stale, expired, invalid.

`manifest::{parse_pack,parse_project}` parse bytes; `validate_relationships`
validates cross-pack/project references; `question_sha256`, `answer_sha256`,
`gap_id`, and `evaluate_question` provide shared hash/evaluation functions.
Exact signatures may be filled in by Agent 1 without changing these semantics.

`report::render_json(&AuthoringPlan) -> Result<Vec<u8>, ForgeError>` and
`report::render_text(&AuthoringPlan) -> String`. Report schema is
`forge.authoring-plan/1`; answer values are never copied into reports.
Every gap is counted exactly once even with multiple assignments. An assignment
only counts when it reaches an explicit project policy. Assigned+deferred is
invalid. Total equals assigned + deferred + unresolved at runtime.

Required missing/no-answer/stale/expired/invalid answers block only dependent
sections. Explicit clause dependencies also require available answers. Optional
unavailable answers remain visible. Empty policy is planned; ready section with
a valid human clause is human-draft-present, otherwise skeleton-ready. A policy
is blocked when any section is blocked, without suppressing unrelated sections.
Ordering is stable by keys, with section order `(order, topic_key)`. No wall clock.

`render::render(&LoadedAuthorProject, &AuthoringPlan) ->
Result<RenderedAuthorProject, ForgeError>` returns policies and provenance.
Agent 3 owns rendering-only types: `RenderedAuthorProject { policies:
Vec<RenderedPolicy>, provenance: Vec<u8> }` and `RenderedPolicy { policy_key,
relative_path: String, markdown: Vec<u8> }`.
`output::publish(root: &Path, relative_output_dir: &Path,
artifacts: &[OutputArtifact]) -> Result<(), ForgeError>`;
`OutputArtifact { relative_path: String, bytes: Vec<u8> }`.
The output module owns confined directory publication, not input loading.

Provenance schema `forge.authoring-provenance/1` partitions every output byte
with zero-based UTF-8 byte half-open spans, including generated separators.
Every span has policy/topic/gap/control/assignment/answer/clause references as
applicable and all input hashes via the bound provenance graph. Human clause
bytes are preserved verbatim and traced to exact source spans. No answer-value
interpolation occurs. Raw HTML and headings that could alter the policy outline
are rejected; visible unresolved markers never manufacture substantive text.

## CLI and publication

`forge author plan --manifest FILE [--format text|json] [--output-dir DIR]`
prints a plan and optionally publishes plan.json plus plan.txt in a new directory.
`forge author build --manifest FILE --output-dir DIR [--format text|json]`
publishes plan.json, plan.txt, provenance.json and policies/POLICY-KEY.md as one
generation, then prints the requested plan format. Output paths resolve from the
project root. No overwrite or implicit parent-directory creation is allowed.
All validation, source revalidation, rendering and serialization precede commit.
Exit 0: valid plan/build without required context or unresolved gaps; exit 1:
valid artifacts with required authoring action; exit 2: input/validation/output
failure. Deferrals remain visible and are not themselves errors.

## Verification and delivery

Test exact pins and complete gap reconciliation, missing/invalid/stale/expired
answers, dependent-only blocking, deferrals, unresolved gaps, stable ordering,
verbatim clauses and exact provenance spans, changed clauses/answers, contained
paths, symlinks/hard links/aliases/overwrites, interruption atomicity, repeated
builds, cross-directory byte equivalence, and generated terminology. Fixtures
contain synthetic framework content only. Preserve existing functionality.

Before commits run fmt --check, clippy --locked --all-targets -- -D warnings,
test --locked and diff --check, then independent full-diff review. Create an
immutable candidate commit and run OCR with --commit, never an equal-endpoint
range. No-items-selected is a failed attempt. Validate and disposition every
finding, remediate confirmed defects, and rerun verification and exact-commit OCR.
Push the feature branch, open a PR, monitor CI and current review threads, and
resolve confirmed findings with evidence. Do not merge without authorization.

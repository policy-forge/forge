# PRD-061 Phase 1 review dispositions

This record covers the initial immutable candidates and their remediation. It is
technical review evidence, not human acceptance, authenticated reviewer identity,
legal/content clearance, pilot validation, or release approval. M-13 and the
other documented Phase 2 work remain deferred.

## Exact-commit OCR intake

OCR 1.11.6 used the configured Qwen provider and `qwen3.8-flash`. Every invocation
used `--commit` against the named commit's parent, with Phase 1 business context.
All selected items completed; none was failed, waived, reused, or empty.

| Candidate | Session | Selected/completed | Findings |
|-----------|---------|-------------------:|---------:|
| Contracts `f262d0bdd0bf4e3fe9b5fed938e0380a1d28a75a` | `abd46439-85bf-4f6d-ba38-2e0dae23fa9f` | 4/4 | 19 (C1–C19) |
| Engine `bfa5a88d91edeef039d99479a4428b943119fec8` | `bf26069e-ecbd-49ff-a8af-76f69918bc55` | 6/6 | 10 (E1–E10) |
| Integration `ca0cc647001db93e86a49cb5959e75c394e44bd4` | `08cec700-38e8-4161-bbcd-2923f7e9a014` | 11/11 | 7 (I1–I7) |

The identifiers above number each run's final comments in output order. A
contracts planning request timed out before fallback review; there were also
four failed context-tool calls across the runs (later-commit file lookup,
invalid line ranges, and malformed search syntax). These did not leave selected
files unreviewed. The terminal manifests report `complete`, with zero failed
items. Findings were independently checked against the integrated source.

OCR defaults excluded Markdown and test paths. Independent complete-diff reviews
covered those files. The remediation rereview explicitly includes the CLI test
file so a test-only correction cannot produce a no-items-selected result. Its
exact commit, terminal coverage, and hosted-check results are recorded in the PR.

## Dispositions

| Finding(s) | Disposition and evidence |
|------------|--------------------------|
| C1, C2 | Fixed timestamp schema guards. Lexical patterns reject malformed timestamps without relying on optional JSON Schema format assertions. They preserve the locked Chrono parser's accepted spellings and precision. Calendar validity and UTF-8 byte limits remain explicit runtime checks; parser/schema regressions cover the distinction. |
| C3, C14 | False positive: an explicitly provided empty string or list can be valid when its question constraints permit it. An empty set is an explicit human answer, not inferred absence. `explicit_empty_values_are_distinct_from_no_answer_when_constraints_allow_them` distinguishes these values from `no-answer`; declared minimum constraints still produce invalid context. |
| C4 | Fixed schema maintenance gap: removed unreachable definitions, retained standalone offline schemas, and added shared-definition equality and local-reference reachability checks. |
| C5, E1, E3 | Intermediate-commit findings, absent in the integrated candidate. `src/lib.rs` exports `authoring`; `src/authoring/mod.rs` declares its modules; `src/linkage/mod.rs::read_confined_local_file` exists; CLI dispatch, formats, error variants, and exit-code arms were added by `ca0cc64`. Integrated tests and clippy compile and exercise them. |
| C6, C15 | Fixed repeated graph scans and answer canonicalization through private `ClauseIndex` lookup tables and one cached digest per answer. Required-answer sets are precomputed per topic. Regex validation remains a bounded constant amount per question: `plan::evaluate_questions` evaluates each question once and sections reuse those results. |
| C7 | Removed the redundant null branch; recursive `reject_null` remains authoritative before Serde. Explicit no-answer-with-value rejection remains. |
| C8 | Pin diagnostics now identify the supplied field name. |
| C9 | No current defect: `GapPlan.classification` is populated only from the upstream closed enum's `as_str()` in `plan::build_gaps`. The report spelling and frozen output shape remain unchanged. |
| C10, C12 | Fixed published text/single-line schema constraints, including whitespace-only content, control/bidi characters, and trimmed single-line fields. Added negative parser/schema regressions. |
| C11 | Runtime-authoritative path rules remain intentional and documented in both schema metadata and usage. `validate_local_path` rejects device names, unsafe characters, whitespace/dot aliases, and components over 255 UTF-8 bytes. JSON Schema character lengths cannot express those byte limits; passing schema preflight alone is not runtime acceptance. |
| C13 | False ambiguity claim: both `plan::build_policies` and `render::render_policy` sort sections by `(order, topic_key)`. Equal order values are legal and deterministic, as frozen in the implementation plan. |
| C16 | Fixed freeform labels: a dedicated validator permits ordinary colons such as `Interview: CISO, 2026-08`, while rejecting rooted local paths and file URIs. Matching parser/schema tests cover punctuation and unsafe roots. |
| C17 | False positive across layers: lexical ASCII folding is only an early check. `CaptureSet::read` compares held-handle `(volume, file)` identities and rejects actual duplicate files independently of Unicode spelling. The Unicode alias regression exercises case/decomposition aliases on normalizing filesystems. |
| C18 | No discrepancy: topics, families, and policies intentionally share the same conservative 1,000-record bound. Diagnostics name the actual collection, and both schemas use the same current bound. Separate future constants are optional refactoring. |
| C19 | No current spelling defect: the closed enums' exhaustive `as_str()` matches and Serde kebab-case spellings agree. Existing text/JSON and deterministic rendering tests exercise the output. Runtime serialization inside a string accessor is unnecessary for this change. |
| E2 | Fixed imported JSON bounds: strict duplicate/depth parsing uses the already-bounded captured byte length rather than the new authoring contract's 16 KiB string limit. Real PRD-056 tests include 20 KiB rationale/note and 70 KiB synthetic framework prose. |
| E4, E8 | Fixed the imported applicability-manifest limit and duplicate local constants. The role uses PRD-056's limit, constrained before allocation by authoring's remaining 50 MiB aggregate source budget. A real baseline above 2 MiB and a remaining-budget regression cover this boundary. |
| E5 | Fixed with an explicit conservative boundary: leading `../` from a contained nested manifest is supported; internal canceled descents and dot/empty aliases fail before capture. No unverified directories are manufactured. Tests include a canceled symlink to matching external bytes and successful nested-parent binding. |
| E6 | Bounded optimization opportunity, not a correctness defect. Captured bytes are retained for exact revalidation and copied for parsing/clauses. The 50 MiB limit describes source bytes, not total heap usage; every read is additionally bounded by the remaining source budget. |
| E7 | Rejected recommendation: automatically canonicalizing a user-supplied symlinked root would bypass the requested alias/symlink rejection boundary. Input capture and publication both use no-follow traversal and reject that ancestry. Usage explicitly requires the physical manifest directory path. |
| E9 | Fixed the marker to say unresolved context blocks clause inclusion, covering both required questions and explicit clause dependencies on optional answers. A CLI regression verifies the optional-context marker, truthful cause, and omitted clause bytes. |
| E10 | The amplification is fixed by C6/C15. Full relationship validation remains intentional because public `build_plan` accepts directly constructed in-memory inputs outside the loader. Its API documentation now states that boundary. |
| I1 | Fixed example ignore rules to cover the publisher's `.forge-authoring-stage-*` directories after interruption. Verified with `git check-ignore`. |
| I2 | Added confined-reader precondition checks before I/O for absolute normalized roots and nonempty normalized descendants, including spellings hidden by `Path::components`. Three regressions cover invalid arguments and successful confined reading. Existing caller diagnostics retain the input role. |
| I3 | No uncovered exit behavior: CLI regressions already assert action-required exit 1 and invalid-input/output exit 2, including unchanged outputs. The suggested unit test would duplicate those mappings and pin incidental diagnostic wording; no functional change is needed. |
| I4 | Added the `forge.author-project/1` manifest description to both command help arguments. |
| I5 | No unsafe reachable CLI state: Clap requires build's destination, dispatch always passes `Some`, and the public execution function rejects missing build destinations before input reads or writes. Alternative entry-point signatures are optional refactoring. |
| I6 | No current identity defect: the tuple is produced in one helper from the existing named identity fields and used only for whole-value equality/set membership by capture and revalidation. No caller destructures or transposes it. The shared signature stays unchanged. |
| I7 | Added a persistent checked-in-example guard: copy exact input bytes to a temporary project, validate both schemas, run plan with exit 0 and two assigned gaps, and compare repeated complete builds on supported platforms. |

Windows CI additionally exposed an unused publication-test helper/import. Both
now use the same Linux/macOS cfg as their callers; read-only planning tests remain
enabled on Windows. Independent review confirmed the predicates match.

The remediation preserves Phase 1 output semantics except for the corrected
blocking explanation, admits valid upstream inputs within the documented budget,
and rejects ambiguous paths earlier. No output verdict, lifecycle transition,
component rendering, or M-13 impact analysis was added.

## Hosted review of `aa83e27`

Copilot reviewed all 39 changed files and raised one compile-error claim at
`src/cli/mod.rs:1674`. This was a false positive: `execute` receives `&Cli` and
matches `&cli.command`, so the binding is already `&PathBuf` and coerces to the
expected `&Path`. Exact-commit local checks and Ubuntu, macOS, and Windows tests,
clippy, and release builds passed. The thread records that evidence and is
resolved.

CodeRabbit reviewed 34 selected files from `1501152` to `aa83e27`; its configured
path filter excluded the five `docs/` files covered by independent review. Run
`76360faa-ad97-48a3-b9b1-a721d61e8588` produced five findings:

| Finding | Disposition |
|---------|-------------|
| Nested baseline dependency labels on Windows | Fixed. Build validated UTF-8 components into a slash-separated label without native `PathBuf::push` separators. Unit tests assert exact strings, and an unconditional CLI regression covers root, nested, and contained parent dependencies with exact report regeneration and portable provenance paths. |
| Duplicate question regex compilation | Bounded refactoring opportunity, not a current scalability defect. The reviewer explicitly identifies one duplicate compilation per eligible answered question. Existing question/string/regex limits and once-per-question evaluation bound this work; the earlier per-clause amplification is already removed. No API refactor is needed for this tranche. |
| Pin `jsonschema` to 0.41.0 | Stale repository advice. Fresh `origin/main`, this branch's unchanged Cargo manifest and lockfile, and `.github/copilot-instructions.md` use 0.45. The tested APIs remain supported. Downgrading would reverse current dependency policy. |
| Public `ForgeError` variants and semver | Valid release compatibility concern, not a false positive. Exhaustive downstream matches can break, as can matches on the extended public CLI enums. This is an unreleased technical tranche. Keep distinct domain errors and established exit behavior; public API review, migration documentation, and a semver-compatible release decision are now explicit pending gates in the PRD, plan, and usage documentation. No release or full Rust source-compatibility claim is made. |
| Duplicate-decoded-key fixture assertion | Fixed. A dedicated regression asserts the duplicate-object-key diagnostic and decoded `schema_version` key, so missing required fields cannot satisfy it. |

## Exact-commit OCR of `aa83e27`

Session `98ccf4f4-b2ac-4c78-942e-5b759561a944` completed all 10 selected files,
explicitly including the CLI test file, with zero failed or waived items. It
produced seven final comments, numbered R1–R7 in output order. The run recovered
from a planning timeout and a malformed comment-tool invocation; a context
compression request was canceled during completion. These execution issues are
recorded separately from the terminal `complete` coverage manifest.

| Finding(s) | Disposition and evidence |
|------------|--------------------------|
| R1 | Fixed the example staging ignore pattern for nested output parents. `git check-ignore` covers root, one-level, and deeper staging paths while leaving ordinary nested source files visible. |
| R2 | Fixed source-budget diagnostics without interpreting error strings. Reads remain bounded before allocation; partial-budget failures include the remaining aggregate allowance and original cause, zero budget fails before I/O, and tests verify exact-fit success and unchanged capture state after failure. |
| R3 | Corrected pack schema metadata to describe its actual fields and lexical timestamp checks. Full calendar and baseline binding checks remain runtime responsibilities. |
| R4 | Fixed Windows trailing-dot/space aliases in normal root and descendant components before I/O. Imported relative inputs already had the stricter portable guard; the reachable missing guard was the explicit manifest root. Tests preserve valid prefixes and Unix filenames, reject aliases before filesystem/network access, and exercise the Windows CLI root. This was an alias-rejection gap, not a demonstrated project-root escape. |
| R5 | False positive. `deferrals` and `human_clauses` are required arrays in the project schema and non-defaulted Rust fields. `Fixture::new` initializes both, and every `refresh_baseline` caller retains them. Empty arrays safely iterate; missing fields are not valid project inputs. |
| R6, R7 | Fixed blank-only formatting content in both schemas and runtime. The common predicate ignores whitespace and the cited format characters for blankness only. It preserves joiners inside readable Persian, Indic, and emoji text, unlike a blanket prohibition. Clause validation shares the predicate; tests prove exact Unicode clause bytes and hashes remain intact. This is a conservative nonblank check, not font-independent glyph visibility validation. |

The follow-up patch was independently reviewed across non-overlapping lanes,
including the Windows guards, dependency labels, budget diagnostics, schemas,
Unicode preservation, CLI regressions, and release-gate wording. Final commit,
OCR coverage, verification totals, and hosted results are recorded in the PR.

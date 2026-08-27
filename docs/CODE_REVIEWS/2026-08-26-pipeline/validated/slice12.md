# Validated slice: slice12 — 47 findings (all low severity)

Validation pass: 2026-08-26 against HEAD `b22e2d5` ("Harden successor map opening against symlink races").
Review baseline: docs/CODE_REVIEWS/ocr_review_2026-08-16.md.

**Summary:** 36 valid, 7 partial, 4 invalid, 0 duplicate. Cited line numbers matched HEAD for
every finding except where noted. Two findings (F0782, F0783) rest on a misreading of serde
default semantics (defaults fire only when a key is ABSENT, so present values are preserved).
F0794 describes a multi-type schema message shape that the embedded OSCAL schemas can never
produce. F1048 misjudges a cargo-vet criteria split that correctly mirrors the dev-only vs
deployed dependency graph.

---

## VALID findings

### F0781 — SEC-8 count invariant depends on exhaustive two-variant filtering
- **File:lines:** src/validate/error_types.rs:140-143
- **Symbol(s):** `ValidationReport::new_with_schema_context`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `schema_error_count` and `semantic_error_count` are each computed by
  filtering on one of the two `ValidationErrorCategory` variants. The documented SEC-8
  invariant (`schema + semantic == errors.len()`, doc at lines 47-48) holds only because every
  error happens to be one of these two variants; a future third category would silently break
  the invariant while serde round-trips keep succeeding.
- **Evidence:** Lines 140-143 confirmed: two independent `.filter().count()` passes. SEC-8
  assertions exist (error_types.rs `report_new_mixed_errors_counts_correct`; mod.rs:776-780)
  but only over the current two variants.
- **Remediation:** Keep the Schema filter and derive the second count arithmetically:
  `let semantic_error_count = errors.len() - schema_error_count;` with a comment citing SEC-8.
  No test changes needed; the arithmetic form makes the invariant hold for any future category.
  No snapshot impact.

### F0788 — per-node path String allocation in orphaned-link walk
- **File:lines:** src/validate/semantic.rs:113-133
- **Symbol(s):** `walk_for_orphaned_links_inner`
- **Category:** performance | **Severity:** low
- **Root cause:** Every visited object key and array element allocates a fresh formatted path
  String (`format!("{current_path}.{key}")` line 114, `format!("{current_path}[{i}]")` line
  126) although the path is consumed only when an orphaned-href error is pushed, giving
  O(nodes x depth) allocation churn on large catalogs.
- **Evidence:** Lines 113-133 confirmed; the `errors.push` at lines 105-112 is the sole
  consumer of rendered paths.
- **Remediation:** Carry a reusable segment stack (e.g. `enum Segment { Key(String), Index(usize) }`
  or a `String` buffer with truncation to the saved length) down the recursion; render the path
  string only when constructing a `ValidationError` ("{segments}.href"). Preserve
  `MAX_WALK_DEPTH` (line 71) semantics and the `trace!` call at line 97. Behavior identical;
  existing semantic tests unchanged; no snapshot impact.

### F0789 — check_missing_references doc understates the no-op scope
- **File:lines:** src/validate/semantic.rs:139-149
- **Symbol(s):** `check_missing_references`
- **Category:** documentation | **Severity:** low
- **Root cause:** The doc promises "For Catalog: no-op (return empty vec)" (line 144) but the
  match arm at line 148 silently no-ops for `Profile` and `Mapping` as well. Profiles carry
  import/include-control references plausibly in scope for PRD M-4, so the narrower-than-stated
  catch arm hides a behavioral gap.
- **Evidence:** Line 144 doc vs line 148 arm
  `OscalModelType::Catalog | OscalModelType::Profile | OscalModelType::Mapping => vec![]`.
- **Remediation:** Update the doc to: "For Catalog, Profile, and Mapping: no-op. PRD M-4
  reference checks apply to Component Definitions only; Profile import targets are external
  hrefs that forge does not follow (SEC-5)." If Profile checks are actually wanted, file a
  follow-up work item instead of the doc change. Doc-only change is acceptable at this severity.

### F0795 — extraction helpers fabricate plausible-sounding sentinel output
- **File:lines:** src/validate/formatter.rs:213-238 (fallback at 222; related sites)
- **Symbol(s):** `extract_length_constraint`, `extract_parenthesized`, `extract_trailing_quoted` fallback call sites, `extract_actual_value`
- **Category:** maintainability | **Severity:** low
- **Root cause:** When the assumed raw-message shape does not match, helpers return invented
  text presented as fact: "limit exceeded" (line 222, rendered as "max length: limit
  exceeded"), "unknown" (`extract_parenthesized` line 238; the trailing-quoted fallbacks in
  `classify_type_mismatch` and `classify_pattern_or_format`), and "(not found)"
  (`extract_actual_value` line 82). A user reading "max length: limit exceeded" believes the
  schema really specifies that limit.
- **Evidence:** All sentinel returns confirmed at the cited lines; `classify_length_constraint`
  interpolates the fabricated string directly into `message` and `expected`.
- **Remediation:** Replace fabricated sentinels with an honest marker such as
  "(constraint details unavailable)" in `extract_length_constraint`, `extract_parenthesized`,
  and the `extract_trailing_quoted` fallback call sites; keep "(not found)" only for genuine
  pointer-resolution failure and document it. Update formatter.rs unit tests asserting the old
  sentinel strings (grep for "limit exceeded"/"unknown" in the tests module). Longer term,
  source values from `jsonschema::ValidationErrorKind` instead of parsing Display prose.

### F0796 — catch-all actual-value preview fully serializes deep payloads
- **File:lines:** src/validate/formatter.rs:66-78
- **Symbol(s):** `extract_actual_value`
- **Category:** performance | **Severity:** low
- **Root cause:** Non-scalar values are rendered with `other.to_string()` (line 76),
  materializing the entire sub-document into a heap String only to truncate to 100 chars.
  The empty-pointer branch (lines 66-68) serializes the WHOLE document for a root-level error.
- **Evidence:** Lines 66-78 confirmed: scalar match arms, `other => other.to_string()`, then
  `truncate_value(&serialized, 100)`.
- **Remediation:** Add container arms before serialization:
  `Value::Array(items) => format!("[{} items]", items.len())` and
  `Value::Object(entries) => format!("{{{} keys}}", entries.len())`; apply the same structural
  summary in the empty-pointer branch. SEC-1 truncation contract unchanged; no snapshot impact.

### F0798 — text renderer success branch drops supported_input and trailing newline
- **File:lines:** src/validate/report.rs:24-32
- **Symbol(s):** `render_text_report`
- **Category:** bug | **Severity:** low
- **Root cause:** The valid branch returns an unqualified "Valid: ... passes all validation."
  that (a) ignores `report.supported_input()` — a report built via the public
  `ValidationReport::new_with_context(.., false, vec![])` renders as a clean pass while the
  JSON renderer exposes `"supported_input": false` — and (b) ends without the trailing
  newline that every failure-path line emits via `writeln!`/`write!`, giving line-oriented
  consumers differently shaped output per mode.
- **Evidence:** Lines 24-32 confirmed. The shipped CLI cannot currently produce
  is_valid==true with unsupported input (run_full_validation injects the version error,
  mod.rs:309-312), so impact is confined to direct library consumers; cli/validate.rs:90-93
  papers over the newline by appending one for the valid branch.
- **Remediation:** In the valid branch append "\\n  Warning: input support policy violation
  reported" when `!report.supported_input()` and terminate the success string with a trailing
  newline to match the failure path. Then simplify cli/validate.rs:91-93 (drop the extra
  `format!("{}\\n", ...)` wrapper). Add renderer tests per F0801. Snapshot impact: none in
  tests/snapshots/ (renderer output is asserted inline, not snapshotted).

### F0799 — renderer computes the same truth twice (counts vs sections)
- **File:lines:** src/validate/report.rs:36-50 (summary) vs 57-95 (sections)
- **Symbol(s):** `render_text_report`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The summary line is built from `report.schema_error_count()`/
  `semantic_error_count()` while the Schema/Semantic sections independently re-filter
  `report.errors()` by category. Today the constructor enforces equality (SEC-8), but two
  computations of the same classification could diverge after a third category or a count
  change, letting the header claim errors that are never rendered.
- **Evidence:** Lines 38-50 use accessor counts; lines 57-60 and 78-82 re-filter independently.
- **Remediation:** At the top of the failure path, filter once into `schema_errors` /
  `semantic_errors` `Vec<&ValidationError>` and drive both the summary parts (`len()` with
  pluralization) and the sections from those vectors; remove the accessor calls from the
  renderer. Existing renderer tests unchanged.

### F0801 — no renderer test exercises supported_input == false
- **File:lines:** src/validate/report.rs:253-274 (tests module)
- **Symbol(s):** `render_text_report` / `render_json_report` tests
- **Category:** test | **Severity:** low
- **Root cause:** The report.rs test module covers contextual reports only with
  `supported_input = true` (`contextual_report_exposes_declared_and_actual_baselines`, line
  254). error_types.rs has deserialization tests with `supported_input: false`, but neither
  renderer is ever asserted for an unsupported report, so the PRD S-1/S-2 surfacing of that
  flag is unguarded.
- **Evidence:** Full read of both test modules confirms no supported_input=false renderer
  case.
- **Remediation:** Add two tests in report.rs tests: (1) JSON — build
  `ValidationReport::new_with_context("legacy.json".to_string(), "catalog".to_string(),
  Some("1.0.0".to_string()), false, vec![])`, render_json_report, parse, assert
  `supported_input == false` and `is_valid == true`; (2) text — same report through
  render_text_report, assert the output carries the support warning after F0798 lands (locks
  both paths).

### F0806 — unnecessary clone of version.error in run_full_validation
- **File:lines:** src/validate/mod.rs:310-312
- **Symbol(s):** `run_full_validation`
- **Category:** performance | **Severity:** low
- **Root cause:** `version` is a locally owned `VersionInspection`; after the if-let only the
  disjoint fields `version.declared` and `version.supported` are read (lines 331-332), so
  `version.error.clone()` duplicates the whole `ValidationError` (three Strings) needlessly.
- **Evidence:** Line 310 confirmed: `if let Some(error) = version.error.clone()`. The sibling
  `validate_artifact` already moves `version.error` without cloning (line 239) and reads
  `version.declared` afterwards, proving the partial-move pattern compiles here.
- **Remediation:** Change line 310 to `if let Some(error) = version.error {`. No behavior
  change; no tests affected.

### F0807 — detect_model_type doc omits two roots and AmbiguousArtifact
- **File:lines:** src/validate/mod.rs:112-118 (body 120-153)
- **Symbol(s):** `detect_model_type`
- **Category:** documentation | **Severity:** low
- **Root cause:** The doc enumerates only `"catalog"` and `"component-definition"` roots,
  omitting `"profile"` and `"mapping-collection"` (handled at lines 137-151), and the
  `# Errors` section lists only `ValidateError::UnknownModelType` while the function also
  returns `ValidateError::AmbiguousArtifact` (lines 143-145).
- **Evidence:** Doc line 114 lists two roots; the ambiguity check at 131-145 predates the doc
  text and is unmentioned.
- **Remediation:** Update the doc: "Inspects top-level keys: `catalog` → Catalog,
  `component-definition` → ComponentDefinition, `profile` → Profile, `mapping-collection` →
  Mapping. Roots mapped to null or non-object values still count as present. Exactly one
  recognized root must be present." Add `AmbiguousArtifact` to `# Errors`. Doc-only change.

### F0813 — silent blanket rejection if baseline version constants drift
- **File:lines:** src/validate/version.rs:28-40
- **Symbol(s):** `is_supported_oscal_version` (constants at lines 8-10)
- **Category:** maintainability | **Severity:** low
- **Root cause:** `SCHEMA_VERSION_USED` aliases `crate::oscal::metadata::OSCAL_VERSION` and
  `MIN_SUPPORTED_OSCAL_VERSION` is hand-maintained. If either drifts to a non-canonical form,
  `parse_version` returns `None` and lines 32-37 quietly classify every document as
  unsupported, blaming the input instead of revealing the misconfiguration.
- **Evidence:** Lines 32-37 confirmed (`let-else` with bare `return false`); constants at
  lines 8-10; `OSCAL_VERSION = "1.2.3"` (src/oscal/metadata.rs:11).
- **Remediation:** Add a unit test in version.rs asserting `parse_version(MIN_SUPPORTED_OSCAL_VERSION)`
  and `parse_version(SCHEMA_VERSION_USED)` are both `Some` and minimum <= maximum, so drift
  fails loudly; optionally add `debug_assert!` in both let-else arms. No behavior change.

### F0817 — three tests exercise the identical nonexistent-file command
- **File:lines:** tests/cli_integration.rs:234-252, 897-913, 947-980
- **Symbol(s):** `convert_nonexistent_file_shows_not_found_error`, `exit_code_1_for_file_not_found`, `test_error_message_missing_file`; also `help_shows_convert_and_validate` (19-27) vs `test_help_text_lists_all_subcommands` (1013-1033), `max_size_flag_is_recognized_by_clap` (331-337) vs `test_convert_help_lists_all_options` (1035-1049)
- **Category:** maintainability | **Severity:** low
- **Root cause:** Three tests run the identical "convert a nonexistent .md" command, each
  pinning slightly different stderr vocabulary ("not found" / "No such file" / SEC-4 leak
  guards) that can drift apart when error rendering changes, at the cost of redundant process
  spawns. `help_shows_convert_and_validate` is a strict subset of
  `test_help_text_lists_all_subcommands`, and `max_size_flag_is_recognized_by_clap` is a
  subset of `test_convert_help_lists_all_options`.
- **Evidence:** All three missing-file bodies confirmed: identical `convert nonexistent.md
  --strategy catalog --format json` invocations with diverging assertions; help/flag
  duplication confirmed.
- **Remediation:** Consolidate each cluster into one test asserting everything (exit code,
  stderr wording, absence of internal paths/panics per SEC-4). Keep one canonical name per
  cluster; update any T-number traceability comments that point at removed tests (see F0818).

### F0818 — traceability IDs reused for unrelated tests
- **File:lines:** tests/cli_integration.rs (T008 at 103 and 618; T016 at 339 and 813; T022 at 501/738 and 1035; T023 at 849/895 and 1052)
- **Symbol(s):** test comments/doc-comments carrying T-numbers
- **Category:** documentation | **Severity:** low
- **Root cause:** Traceability IDs (a load-bearing practice — specs/*/tasks.md cite these
  T-numbers) label unrelated tests: T008 covers both the catalog-stdout [US2] test and the
  component --output [US1] test; T016 covers the edge-case banner and the directory-as-profile
  test; T022/T023 appear under different user stories. "T016 fails" grep lands on two
  unrelated tests.
- **Evidence:** Confirmed occurrences at the cited lines (e.g. `// T008 [US2]` line 103 vs
  `/// T008 [US1]` line 618; `// T023 [US5]` line 895 vs `/// T023 [US3]` line 1052).
- **Remediation:** Give each test a unique ID or prefix with its WI/story (e.g. `WI27-T016`)
  in the test comments, then update the corresponding spec task references under specs/ that
  cite the ambiguous T-numbers.

### F0819 — log-control tests assert generic level tokens instead of app events
- **File:lines:** tests/cli_integration.rs:1082-1153
- **Symbol(s):** `test_verbose_flag_shows_pipeline_stages`, `test_quiet_flag_suppresses_output`
- **Category:** test | **Severity:** low
- **Root cause:** The verbose test demands stderr contain the bare words DEBUG/INFO/TRACE
  (lines 1109-1112) and the quiet test fails if those words EVER appear (lines 1144-1147) —
  including inside a legitimate ERROR payload, after a tracing_subscriber rendering change
  (ANSI/padding), or during an unexpected diagnostic on a successful run. Latent
  false-positive/false-negative sources coupled to subscriber formatting rather than app events.
- **Evidence:** Confirmed at lines 1108-1112 and 1142-1147; the pipeline emits its own
  info!/debug! messages (e.g. src/pipeline.rs `tracing::info!` calls) that could be matched
  instead.
- **Remediation:** Assert on stable application-owned markers (a known pipeline info!/debug!
  message payload or span name emitted by forge itself, e.g. "Building component definition"
  or a catalog-pipeline stage message) for the verbose case; for quiet, assert those same
  markers are absent rather than the bare level words. Keep stderr/stdout separation asserts.

### F0823 — hand-derived 'Section X.Y' cross-references in fixture prose
- **File:lines:** tests/common/fixture_generator.rs:384 (plus 144, 608, 614, 616, 740, 1062)
- **Symbol(s):** DOMAINS requirement strings referencing "Section N.M"
- **Category:** maintainability | **Severity:** low
- **Root cause:** All "Section X.Y" cross-references (8.3 at line 384, plus 1.1, 1.2, 2.4, 3.1,
  7.1) are hand-derived from the current ordering of DOMAINS and subsections. Adding, removing,
  or reordering a domain/subsection silently turns these strings into dangling references —
  there is no linkage between the prose and the headings actually emitted
  (`### {domain_num}.{sub_num}` at line 1330).
- **Evidence:** Cited strings confirmed at the listed lines; heading numbers are computed
  independently at lines 1316/1330.
- **Remediation:** Derive the numbers programmatically — build a title -> "{domain_idx}.{sub_idx}"
  map from DOMAINS before emitting and substitute placeholder tokens (e.g. "See Section
  {{BACKUP}}" resolved during generation) — or at minimum add a test pinning the expected
  section indices for each referenced subsection title. No committed-fixture change unless the
  numbers actually shift (fixture_determinism_test enforces byte-sync).

### F0825 — documented statistics do not match the generated corpus
- **File:lines:** tests/common/fixture_generator.rs:1281-1287
- **Symbol(s):** `generate_synthetic_policy` doc comment
- **Category:** documentation | **Severity:** low
- **Root cause:** The doc claims "~30 citations/references (\"[NIST SP 800-53 AC-2]\")" but the
  generator appends one supplementary-guidance paragraph per subsection (40 subsections: 4 per
  domain x 10 domains), each rendering the NIST_REFERENCES entry as plain prose ("aligned with
  NIST SP 800-53 AC-2", lines 1361-1362 + write_supplementary_guidance) — 40 occurrences,
  unbracketed, not ~30. "~20 compound statements" and "~25,000 words" are likewise unverified
  estimates. Consumers tune benchmarks/parsers off these numbers.
- **Evidence:** 40 Subsection definitions counted in DOMAINS; write_supplementary_guidance
  (line 1421+) interpolates nist_ref unbracketed; no bracketed "[NIST ...]" strings exist in
  the generator (grep confirmed).
- **Remediation:** Rewrite the doc line to "40 plain-text standard references (one per
  subsection, cycling NIST_REFERENCES)" and verify/fix the compound-statement and word-count
  figures (or compute them in the suggested regression assertion in
  fixture_determinism_test.rs). Doc-only unless the determinism test gains stat assertions.

### F0827 — discarded fmt::Write Results rely on an undocumented invariant
- **File:lines:** tests/common/fixture_generator.rs:1316, 1330, 1401, 1428
- **Symbol(s):** `generate_synthetic_policy`, `write_scope_paragraph`, `write_supplementary_guidance`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `let _ = write!(doc, ...)` discards Results; sound today solely because the
  sink is a String whose fmt::Write impl is infallible. If the generator is ever retargeted at
  a fallible sink (BufWriter, compressed stream), partial-write failures would be swallowed and
  the deterministic-output guarantee would break invisibly.
- **Evidence:** Four `let _ = write!` sites confirmed at the cited lines.
- **Remediation:** Add a comment documenting the invariant ("writes target a String, whose
  fmt::Write impl is infallible; if retargeted at a fallible writer, propagate fmt::Result")
  at the first site, or change the helpers to return fmt::Result and `?`-propagate in
  generate_synthetic_policy. Behavior unchanged either way.

### F0831 — snapshot normalization special-cases the exact "last-modified" key
- **File:lines:** tests/common/mod.rs:35-37
- **Symbol(s):** `normalize_for_snapshot`
- **Category:** maintainability | **Severity:** low
- **Root cause:** Normalization special-cases the exact serialized key "last-modified". This
  silently depends on every OscalMetadata-like struct keeping that serde rename forever; a
  rename, casing change, or any new date-bearing field (e.g. "published", "updated") escapes
  normalization and resurfaces as flaky snapshot diffs that look like product bugs.
- **Evidence:** Key check confirmed at lines 35-37; all scrubbable timestamps share one
  RFC3339 shape.
- **Remediation:** Prefer shape-based replacement — regex-match ISO-8601 datetime strings
  wherever they appear (e.g. `^\d{4}-\d{2}-\d{2}T[0-9:.]+(Z|[+-]\d{2}:\d{2})$` →
  "2026-01-01T00:00:00Z") — or centralize the magic key list in a shared constant colocated
  with the serde renames in src/oscal/metadata.rs. Update snapshot expectations only if the
  new matcher rewrites values previously left alone.

### F0834 — eighteen copy-pasted format-pair tests
- **File:lines:** tests/export_format_pairs.rs:27-141
- **Symbol(s):** `format_pair_catalog_*` / `format_pair_component_*` tests; helper `export_and_read`
- **Category:** maintainability | **Severity:** low
- **Root cause:** Eighteen tests are copy-pastes differing only in (fixture, input format,
  output format) plus two assertion flavors. Adding a new OutputFormat or model type requires
  hand-duplicating three more tests, and drift between copies is likely.
- **Evidence:** All 18 tests confirmed structurally identical modulo fixture/format strings;
  helper at lines 12-23.
- **Remediation:** Replace with a table-driven layout: iterate a (model, input-format,
  output-format) matrix deriving fixture paths, sharing one root-key validator per model
  (JSON: parse + `get(model)`; YAML: parse + key; XML: first-element name == model), or use
  the `test-case` crate to keep one distinct test per pair so failures stay isolated. Keep the
  existing fixture paths; no snapshot impact (no insta snapshots in this file).

### F0835 — export helper swallows error context in a wide matrix
- **File:lines:** tests/export_format_pairs.rs:21-22
- **Symbol(s):** `export_and_read`
- **Category:** test | **Severity:** low
- **Root cause:** The shared helper used by all 18 tests calls `.unwrap()` on both
  `export_artifact` and `read_to_string`; any failure panics with a bare serde/io message and
  no indication of which step or which fixture/format failed, making matrix failures slow to
  triage.
- **Evidence:** Lines 21-22 confirmed: `export_artifact(&input, format, Some(&output)).unwrap();`
  and `std::fs::read_to_string(&output).unwrap()`.
- **Remediation:** Use `unwrap_or_else` with context:
  `panic!("export_artifact failed for {relative_path:?} -> {format:?}: {e}")` and
  `panic!("reading exported file {}: {e}", output.display())`. Test-only change.

### F0841 — missing --format test duplicates Command wiring, never inspects stderr
- **File:lines:** tests/export_integration.rs:99-107
- **Symbol(s):** `cli_export_missing_format_arg`; helper `run_export` (lines 16-27)
- **Category:** test | **Severity:** low
- **Root cause:** This test hand-rolls Command wiring instead of reusing `run_export`, and
  asserts only `exit_code != 0` without inspecting stderr — a misquoted path or a parse
  failure would still "pass" as long as exit is non-zero. The suite also has no negative case
  for an unsupported `--format` value.
- **Evidence:** Lines 99-107 confirmed; `run_export` exists at lines 16-27 and is used by
  every sibling test.
- **Remediation:** Rewrite as `let (exit_code, _stdout, stderr) = run_export(&[CATALOG_JSON]);`
  assert non-zero, and assert `stderr.contains("--format")`. Add a companion test
  `cli_export_invalid_format_value_nonzero_exit` using `--format docx`. Test-only change.

### F0842 — run_export fixed-slice signature forces to_str().unwrap() churn
- **File:lines:** tests/export_integration.rs:16
- **Symbol(s):** `run_export`
- **Category:** style | **Severity:** low
- **Root cause:** `run_export(args: &[&str])` is fine today, but callers repeatedly build
  arrays just to append paths joined from TempDir, converting via `to_str().unwrap()`. A more
  generic signature avoids the churn as cases accumulate.
- **Evidence:** Signature confirmed at line 16; caller pattern confirmed across the file.
- **Remediation:** Optionally widen to `fn run_export<I, S>(args: I) -> (i32, String, String)
  where I: IntoIterator<Item = S>, S: AsRef<std::ffi::OsStr>`. Low priority, ergonomic only;
  no behavior change.

### F0847 — unwrap_or(&vec![]) allocates a throwaway Vec per parameterless control
- **File:lines:** tests/integration_cross_feature.rs:148-150
- **Symbol(s):** parameter-collection closure (collect_param_values helper region)
- **Category:** style | **Severity:** low
- **Root cause:** `param["values"].as_array().unwrap_or(&vec![])` heap-allocates and drops a
  temporary Vec on every param-less control visited just to produce a reference; this runs
  recursively across every control in every parsed catalog.
- **Evidence:** Lines 149-150 confirmed.
- **Remediation:** Rewrite with `map(...).unwrap_or_default()`: collect
  `vs.iter().filter_map(|v| v.as_str().map(ToString::to_string))` inside the map closure. Test
  helper only; no behavior change.

### F0849 — fixture/baseline paths resolve against the process CWD
- **File:lines:** tests/integration_round_trip.rs:59-62 (also 166-171, 247-252); tests/golden_file_tests.rs:477, 552, 556, 701
- **Symbol(s):** `catalog_json_xml_json_round_trip` and siblings; `run_catalog`/`run_component`/`SOURCE_PROFILE` in golden_file_tests
- **Category:** maintainability | **Severity:** low
- **Root cause:** Fixture/baseline paths like `"tests/fixtures/golden/small/input.md"` and
  `SOURCE_PROFILE = "./baselines/nist-800-53.json"` resolve against the test process CWD.
  Plain cargo test happens to chdir to the package root, but IDE adapters, sandboxed build
  systems, or workspace-root invocations fail confusingly — awkward beside golden_file_tests'
  own NORMALIZED_PATH portability rationale.
- **Evidence:** Cited literals confirmed at integration_round_trip.rs:62, golden_file_tests.rs:477/552/556/701.
  Note: `./baselines/nist-800-53.json` is used only as an OSCAL href string (pipeline never
  reads it — src/pipeline.rs passes source_profile through to
  build_control_implementations/sanitize_artifact_path only), so mis-resolution would surface
  only for fixtures that ARE read (input.md paths).
- **Remediation:** Anchor all read fixture/baseline resolution to `env!("CARGO_MANIFEST_DIR")`:
  build paths via `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/...")`
  uniformly (load_fixture, run_catalog/run_component inputs, the convert invocations in the
  round-trip suite, schema_validation_tests, determinism_tests). SOURCE_PROFILE may stay
  relative since it is metadata-only, but document that. No output change.

### F0850 — hardcoded 1.2.3 literal duplicates the OSCAL_VERSION constant
- **File:lines:** tests/integration_regression.rs:62-64
- **Symbol(s):** `phase1_catalog_structure_regression`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The test pins `assert_eq!(oscal_version, "1.2.3")` while
  `src/oscal/metadata.rs` defines `pub const OSCAL_VERSION: &str = "1.2.3"` and the comment
  says the value "must use the current baseline". On a baseline bump this test silently goes
  stale and needs a manual two-place edit. The crate publishes a library target, so the
  constant is referenceable.
- **Evidence:** Literal confirmed at line 64; constant confirmed at src/oscal/metadata.rs:11;
  `forge::oscal::metadata` is public (src/lib.rs `pub mod oscal`).
- **Remediation:** Replace the literal with `forge::oscal::metadata::OSCAL_VERSION` and update
  the assertion message. Test-only change.

### F0851 — uuid assertions accept placeholders (only non-emptiness)
- **File:lines:** tests/integration_regression.rs:57-59 (and 105-107 for component-definition.uuid)
- **Symbol(s):** `phase1_catalog_structure_regression`, `phase1_component_structure_regression`
- **Category:** test | **Severity:** low
- **Root cause:** Asserting only non-emptiness accepts placeholder/corrupted identifiers such
  as "uuid", "TODO", or a truncated id — precisely the Phase 2 output corruption this file is
  meant to catch. The `uuid` crate is already a dependency (Cargo.toml), so parsing closes the
  loophole without new dev-dependencies.
- **Evidence:** `assert!(!uuid.is_empty())` confirmed at lines 59 and 107.
- **Remediation:** In both tests: `let uuid = catalog["catalog"]["uuid"].as_str().expect("must
  be a string"); uuid::Uuid::parse_str(uuid).unwrap_or_else(|e| panic!("must be a well-formed
  UUID, got '{uuid}': {e}"))`. Same for component-definition.uuid. Test-only change.

### F0857 — edge-case fixture inventory enumerated in three independent places
- **File:lines:** tests/golden_edge_case_tests.rs:235-249 (required_dirs), 434-449 (dual array), 538-560 (BTreeSets with magic sizes 7/1/8)
- **Symbol(s):** `fixture_contract_completeness_smoke_test`, `strategy_matrix_dual_strategy_and_agnostic_coverage`, `strategy_constants_match_expected_scope`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The fixture inventory is hand-listed in three places that can drift; the
  existence-contract test already omits the supplemental fixtures exercised by
  `supplemental_citation_positions_and_parameter_like_content` (ec-citation-unusual-positions,
  ec-parameter-like-content, confirmed at lines 512-513). The loop also only checks directory
  existence, never that required files (input.md, expected-catalog.json, expected-error.txt)
  are present, so a truncated fixture checkout passes.
- **Evidence:** Three independent lists confirmed at the cited lines; supplemental slugs
  confirmed absent from required_dirs.
- **Remediation:** Hoist shared constants (CONVERT_FIXTURES, SUPPLEMENTAL_FIXTURES,
  VALIDATION_ONLY_FIXTURES) at module level; have the smoke test iterate all of them and
  additionally assert per-directory expected files exist (input.md always; expected-catalog.json
  for success cases; expected-error.txt for error cases). Keep the size asserts derived from
  the constants rather than magic numbers.

### F0858 — edge-case harness loses diagnostics and treats signaled exit as pass
- **File:lines:** tests/golden_edge_case_tests.rs:103-110 (run_convert_with_baseline), 131-134 (assert_edge_case_error)
- **Symbol(s):** `run_convert_with_baseline`, `assert_edge_case_error`
- **Category:** bug | **Severity:** low
- **Root cause:** Two triage defects: (1) `serde_json::from_str(&text).expect("convert output
  must be JSON")` panics with only the serde error — captured stderr and raw text are dropped,
  and stdout is never captured at all; (2) `status.code().unwrap_or(-1)` maps a signal-killed
  process to -1, which `assert_ne!(run.code, 0)` then treats as a passing non-zero exit — a
  crashed (SIGSEGV/OOM) convert run satisfies the EC error-path tests if any expected
  substring happens to appear in stderr.
- **Evidence:** Lines 103-110 confirmed (no stdout capture; bare expect); sentinel -1 confirmed
  at line 105 feeding assert_edge_case_error at 131-134.
- **Remediation:** Capture stdout alongside stderr; replace the expect with
  `unwrap_or_else(|e| panic!("convert output must be JSON ({e}); status={}\\n--- stderr ---\\n{stderr}\\n--- stdout ---\\n{stdout}", output.status))` (OutputStatus Debug shows signals);
  and in assert_edge_case_error distinguish signaled termination (or assert on a
  captured ExitStatus) instead of the numeric sentinel. Test-only change.

### F0859 — EC-7 golden test fabricates a Citation instead of using fixture output
- **File:lines:** tests/golden_edge_case_tests.rs:384-408
- **Symbol(s):** `ec07_malformed_citation_is_retained_with_unvalidated_marker`
- **Category:** other | **Severity:** low
- **Root cause:** Inside an EC-7 golden-file test that reads expectations from
  tests/fixtures/edge-cases/ec07-malformed-citation/, the block fabricates a Citation
  (arbitrary id "ec07-malformed-citation", text "Malformed citation", url "htp://not-a-url",
  source_requirement_id "req-1") and unit-tests `generate_back_matter` on it, decoupled from
  what the parser actually extracts from the fixture (whose real input.md cites
  "https://example.com/retention" — a valid URL, not the fabricated malformed one). If the
  pipeline stopped emitting malformed URLs, this assertion would still pass.
- **Evidence:** Synthetic Citation at lines 396-401 vs fixture input.md containing only a
  valid https URL; the expected-catalog comparison above it does not cover the unvalidated
  marker.
- **Remediation:** Extract the citation(s) from the converted `output` JSON (walk
  back-matter resources for the url-status prop) and assert on those; or move the synthetic
  assertion into the generate_back_matter unit tests (src/oscal/back_matter.rs) with a comment
  that it is intentionally decoupled from the fixture. Keep the golden comparison as-is.

### F0871 — trace assertions hardcode positional paths tied to iteration order
- **File:lines:** tests/trace_integration.rs:107-111
- **Symbol(s):** trace path format test
- **Category:** maintainability | **Severity:** low
- **Root cause:** The assertions hardcode `paths[0..2]` positional values encoding both the
  path format and build_catalog's internal grouping/flattening order; `paths[0..2]` also
  panics index-out-of-bounds instead of a clear assert if fewer links are recorded.
- **Evidence:** Lines 107-111 confirmed; `TraceLinkCollection::by_oscal_element` exists
  (src/model/trace.rs:107) enabling position-independent lookup.
- **Remediation:** First `assert_eq!(trace_links.len(), 3)`; then rebuild/inspect the catalog
  and derive expected paths: for each (group, control) position look up the link via
  `by_oscal_element(&control.uuid)` and assert its `oscal_json_path` equals
  `format!("catalog.groups[{gi}].controls[{ci}]")`. Preserves the path-format contract without
  encoding insertion order.

### F0872 — stale 'WI-15 not merged' comment in trace test
- **File:lines:** tests/trace_integration.rs:194-196
- **Symbol(s):** component-definition trace-empty test
- **Category:** maintainability | **Severity:** low
- **Root cause:** The comment claims "WI-15 not merged: no implemented-requirements", but the
  current source wires `crate::oscal::implemented_requirements::build_control_implementations`
  into `build_component_definition` whenever `source_profile` is `Some`
  (src/oscal/component_definition.rs:150-155). The assertion holds only because this test
  passes `None` as the profile; the rationale encodes obsolete feature state.
- **Evidence:** Comment at line 194; wiring confirmed in component_definition.rs.
- **Remediation:** Restate the comment in terms of inputs: "No source_profile supplied -> no
  control-implementations are built, so trace capture must remain empty." Comment-only change.

### F0873 — trace suite never exercises stable_id: None through build_catalog
- **File:lines:** tests/trace_integration.rs:13-15
- **Symbol(s):** `test_requirement` helper
- **Category:** test | **Severity:** low
- **Root cause:** The helper forces `stable_id: Some(..)`, leaving the `None` case
  unexercised in the integration suite — yet `build_catalog` returns
  `ForgeError::CatalogBuild` for requirements missing a stable id (src/oscal/catalog.rs:359-364),
  an error path that interacts with trace capture (links recorded before the failure). A unit
  test for the bare error exists (catalog.rs `catalog_missing_stable_id`, line 818) but not
  the trace-interaction variant.
- **Evidence:** Helper at lines 13-15 forces Some; error path confirmed; no None-stable-id
  case in trace_integration.rs.
- **Remediation:** Add `test_requirement_opt(text, Option<&str>, line)` and keep
  `test_requirement` delegating with Some; add a test asserting build_catalog with tracing
  enabled and one None-stable-id requirement returns CatalogBuild and inspect what was
  recorded in the TraceLinkCollection prior to failure (document the partial-links contract).

### F0876 — opaque unwrap of detector executable_path invariant
- **File:lines:** tests/oscal_cli_round_trip.rs:28
- **Symbol(s):** `invoker_if_available`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `info.executable_path.unwrap()` assumes available && functional implies
  executable_path is Some (true today for PathDetector, src/oscal_cli/detector.rs:35-62, and
  confirmed by cli/validate.rs defending the same invariant explicitly). If a detector
  regresses, the failure surfaces as a bare Option unwrap with no hint that test
  infrastructure — not the feature — broke.
- **Evidence:** Line 28 confirmed; OscalCliInfo.executable_path is Option<PathBuf>
  (src/oscal_cli/mod.rs:24).
- **Remediation:** Replace with `.expect("detector reported available+functional without
  executable_path")` (optionally `as_ref().expect(...).clone()` if borrowing matters). Test
  helper only.

### F0877 — hard-coded 30s per-stage timeout for external oscal-cli chain
- **File:lines:** tests/oscal_cli_round_trip.rs:19
- **Symbol(s):** `TIMEOUT`; applied per stage by `run_round_trip_chain` (src/round_trip/chain.rs:34-50)
- **Category:** test | **Severity:** low
- **Root cause:** The 30-second Duration is applied independently to each of the three
  oscal-cli conversions (JSON→XML, XML→YAML, YAML→JSON — confirmed: chain.rs passes the same
  timeout per step), so worst-case wall time is ~90s plus per-stage JVM startup on loaded
  shared CI runners, while remaining generous on fast machines.
- **Evidence:** Const at line 19; per-step application confirmed in run_round_trip_chain.
- **Remediation:** Source the timeout from an environment variable with fallback:
  `fn round_trip_timeout() -> Duration { std::env::var("FORGE_ROUND_TRIP_TIMEOUT_SECS").ok()
  .and_then(|v| v.parse().ok()).map(Duration::from_secs).unwrap_or(Duration::from_secs(30)) }`
  and use it where TIMEOUT is consumed. Test-only change.

### F0879 — #[ignore] test with todo!() body yields zero compile-checked coverage
- **File:lines:** tests/profile_validation_tests.rs:119-125 (also 287-293)
- **Symbol(s):** `schema_with_set_param`, `edge_conflicting_set_param`
- **Category:** test | **Severity:** low
- **Root cause:** Pairing `#[ignore]` with a `todo!()` body means the only safety net if
  someone drops the attribute before WI-31 lands is a panicking CI job rather than a
  meaningful failure, and the intended arrange/act/assert is never compile-checked.
- **Evidence:** Lines 119-125 and 287-293 confirmed; `build_profile` already accepts
  `param_overrides: &[(String, String)]` (src/oscal/profile.rs:204), so the skeleton compiles
  today.
- **Remediation:** Give `schema_with_set_param` its full skeleton: make_catalog_file,
  build_profile(&catalog_path, vec!["AC-1".into()], SelectionMode::Include,
  &[("prm-ac-1".to_string(), "60 days".to_string())], None), serialize ProfileRoot,
  validate_artifact(.., OscalModelType::Profile), assert is_valid — keep `#[ignore]` and
  update the TODO to "implement body and remove #[ignore] when --set-param ships". Apply the
  same to edge_conflicting_set_param. Note: verify the profile schema accepts set-parameters
  before removing #[ignore]; the skeleton's purpose is compile-checking intent.

### F0883 — property-test +1 tolerance encodes a stale rationale and weakens the invariant
- **File:lines:** tests/property_tests.rs:257-262
- **Symbol(s):** `cleaned_text_not_longer` (P-14)
- **Category:** test | **Severity:** low
- **Root cause:** The `+ 1` tolerance claims strip_matches may add a space, but
  `strip_matches` (src/citation.rs:212-230) replaces each matched range (length >= 1) with
  exactly one space — it can only shrink or hold — and every subsequent normalize_prose step
  (orphan-paren removal, whitespace collapse, punctuation-artifact cleanup, trim;
  src/citation.rs:241-269) is non-growing. `cleaned.len() <= text.len()` holds provably; the
  slack byte lets a future regression that inserts one stray space slip past.
- **Evidence:** strip_matches pushes one ' ' per range and copies between-range text verbatim;
  normalize_prose replacements all map equal-or-longer needles to shorter replacements.
- **Remediation:** Tighten to `cleaned.len() <= text.len()` and move the reasoning into the
  assertion message ("every pipeline stage is non-growing: strip_matches swaps N>=1 matched
  bytes for one space; collapse/trim/artifact removal never grow"). Test-only change; proptest
  corpus unaffected.

---

## PARTIAL findings

### F0814 — missing report-fidelity tests for unsupported declarations
- **File:lines:** src/validate/version.rs:163-173 (tests module)
- **Symbol(s):** version.rs tests; `inspect_oscal_version`
- **Category:** test | **Severity:** low
- **Root cause:** The genuine coverage gap: no test asserts `VersionInspection.declared` for an
  *unsupported* declaration, nor that hostile content is escaped in diagnostics while the
  reported value stays exact. However, the finding's proposed assertions contradict CURRENT
  behavior: for unsupported/malformed declarations, `inspect_oscal_version` stores the ESCAPED
  copy in `declared` (version.rs:87-89: `declared: Some(safe_declared.clone())`), so
  `declared == "1.2.4\tgarbage"` verbatim would fail; the existing
  `invalid_declaration_is_bounded_in_report_context` test already pins the escaped form.
- **Evidence:** Lines 87-89 confirmed (escaped copy stored); escaping test exists but no test
  pins declared fidelity for plain-but-unsupported strings like "1.3.0".
- **Remediation:** Add the unsupported-verbatim case in a form consistent with current
  behavior: for "1.3.0" assert `declared == Some("1.3.0")` AND `error.actual == "1.3.0"` AND
  message contains "1.3.0" (escaping is identity for ASCII). For control-char input, assert
  `declared` equals the escaped form (matches current code) and `error.message`/`actual`
  contain no raw control chars; if verbatim `declared` is desired instead, that is a behavior
  change to raise separately. Add an over-long declaration case pinning the 100-char bound.

### F0826 — module-wide allow(dead_code) on fixture_generator
- **File:lines:** tests/common/fixture_generator.rs:8
- **Symbol(s):** module attribute `#![allow(dead_code)]`
- **Category:** style | **Severity:** low
- **Root cause:** The blanket `#![allow(dead_code)]` suppresses unused-code diagnostics for
  every item, and would hide genuinely forgotten definitions if the generator grows. BUT a
  naive removal breaks the build: tests/common/mod.rs is compiled as a separate module per
  integration-test binary (`mod common;` in ~9 test files), and each binary only uses a subset
  of items (e.g. fixture_generator is used only by fixture_determinism_test.rs), so unused-code
  warnings become hard errors under the project's `-D warnings`-equivalent CI posture.
- **Evidence:** Attribute confirmed at line 8; per-binary compilation of tests/common/mod.rs
  confirmed by the `mod common;` declarations in assessment_plan_test.rs, atomize_integration.rs,
  fixture_determinism_test.rs, oscal_1_2_3_compatibility_test.rs, profile_golden_file_tests.rs,
  xml_catalog_test.rs, xml_component_test.rs, xml_validation_test.rs.
- **Remediation:** Do NOT remove outright. Narrow to a targeted, documented form: keep the
  module attribute but add a comment explaining the per-binary compilation reason, OR
  gate conditionally with `#[cfg_attr(not(test), allow(dead_code))]`-style narrowing on the
  specific unused-per-binary items. Lowest-risk: keep + document.

### F0830 — anchored UUID regex misses UUID substrings (latent only)
- **File:lines:** tests/common/mod.rs:24-30 (UUID_RE), 48-55 (string branch)
- **Symbol(s):** `normalize_for_snapshot`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The fully-anchored UUID_RE (`^...$`) normalizes only whole-string UUIDs;
  composite values embedding a UUID (e.g. `run-<uuid>.json`) would bypass normalization and
  break byte-stable snapshots. PARTIAL because no current generator output embeds dynamic
  UUIDs inside larger strings: pipeline UUIDs are v5-deterministic (src/uuid.rs), and the only
  per-run UUIDs flow into fields normalized by other means — no evidence of a leak today.
- **Evidence:** Anchored regex confirmed at lines 24-29; back-matter resource UUIDs are v5
  (src/oscal/back_matter.rs:258) and metadata UUIDs are v4 but only appear as whole-string
  fields. No snapshot contains a UUID-in-string.
- **Remediation:** Either document the whole-string-only contract on normalize_for_snapshot,
  or add an unanchored variant applied to specific composite fields if/when they appear.
  Preemptive `replace_all` on all strings risks rewriting legitimate UUID-bearing content
  (rlink hashes etc.), so documentation is the proportionate fix at this severity.

### F0846 — copy-pasted round-trip test matrices (cited drift is intentional)
- **File:lines:** tests/integration_cross_feature.rs:196-266, 337-410; tests/integration_round_trip.rs (four round-trip tests)
- **Symbol(s):** `normative_props_survive_xml/yaml_round_trip`, `param_elements_survive_xml/yaml_round_trip`; `catalog_*` / `component_definition_*` round-trip tests
- **Category:** maintainability | **Severity:** low
- **Root cause:** The duplication is real: four near-identical XML/YAML round-trip tests in
  each file repeat the convert→export→export→compare skeleton and identical failure-message
  blocks that must stay in sync with EquivalenceDiff's shape. PARTIAL because the finding's
  claimed drift is NOT drift: the XML component test applies `clear_control_implementations`
  and the YAML sibling does not BY DESIGN (XML serialization intentionally omits
  control-implementations per WI-28/EC-5, documented at integration_round_trip.rs:37-47 and
  commented at the YAML test line ~253).
- **Evidence:** Duplication confirmed; intentional asymmetry documented in-file.
- **Remediation:** Factor shared skeletons into local helpers (`assert_round_trip_preserves(dir,
  intermediate_format, check)` in cross_feature; a pipeline driver + shared difference
  formatter in integration_round_trip) that TAKE a normalization flag rather than burying it,
  preserving the documented XML-vs-YAML asymmetry. No behavior change expected; tests must
  still pass per-format.

### F0861 — empty-selection (C-2) Profile coverage gap (schema claim wrong)
- **File:lines:** tests/profile_golden_file_tests.rs:24-66; tests/profile_validation_tests.rs:136-138; src/oscal/profile.rs:256-258
- **Symbol(s):** `golden_include_only`, `golden_exclude_only`; `build_profile`
- **Category:** test | **Severity:** low
- **Root cause:** Genuine gap: nothing locks the documented `build_profile(vec![])` behavior
  (Profile with `"imports": []`, src/oscal/profile.rs:256-258 + doc at 201/210) — active
  snapshots cover only non-empty selections. PARTIAL because the finding's premise that such a
  profile is "OSCAL v1.2.3 schema-valid" is WRONG: schemas/oscal_profile_schema.json requires
  imports with `minItems: 1` (lines 24-26), so an empty-imports Profile is schema-INVALID.
- **Evidence:** Schema minItems:1 confirmed; no unit or integration test pins imports == []
  for build_profile(vec![]) (profile.rs tests all use non-empty ids).
- **Remediation:** Add a test pinning the ACTUAL documented behavior: `build_profile(..,
  vec![], ..)` yields `profile.imports.is_empty()` and serializes `"imports": []`. Separately,
  decide (product decision, raise as follow-up) whether C-2 should emit a schema-valid shape or
  the C-2 case should be rejected before serialize; do NOT add a schema-validity assertion for
  empty imports against the current schema. Extract the build→serialize→normalize→snapshot
  sequence into a helper while adding the third case.

### F0868 — anchored UUID regex in golden_file_tests (latent only)
- **File:lines:** tests/golden_file_tests.rs:62-65 (UUID_RE), 93-110 (normalize_string_value)
- **Symbol(s):** `UUID_RE`, `normalize_string_value`
- **Category:** bug | **Severity:** low
- **Root cause:** The fully-anchored regex normalizes only strings that are EXACTLY a UUID;
  nondeterministic values merely containing a UUID (`urn:uuid:<uuid>`, `<base>/resource/<uuid>`,
  prose-embedded UUIDs) would leak into snapshots. PARTIAL because no current serializer output
  embeds run-dynamic UUIDs in composite strings: catalog/component UUIDs are whole-string
  fields; hrefs use deterministic v5 ids (#<uuid> fragments); absolute-path hrefs are handled
  by the dedicated href branch (lines 100-107). All UUID-bearing fields today are either
  whole-string (matched) or deterministic (stable anyway).
- **Evidence:** Anchored regex confirmed at lines 62-65; href/last-modified special cases at
  94-107; no snapshot contains an unnormalized embedded UUID.
- **Remediation:** Either (a) document on normalize_string_value why bare-only matching is
  sufficient (all dynamic UUIDs are whole-string fields; fragment hrefs are v5-deterministic),
  or (b) switch to unanchored `UUID_RE.replace_all(s, NORMALIZED_UUID)` — but (b) would rewrite
  deterministic v5 href fragments too, perturbing every snapshot, so (a) is the proportionate
  fix unless a real leak appears.

### F1051 — non-stable pins exempted at safe-to-deploy
- **File:lines:** supply-chain/config.toml:803-813 (toml/toml_datetime/toml_parser), 935-937 (wasip3)
- **Symbol(s):** cargo-vet exemptions
- **Category:** maintainability | **Severity:** low
- **Root cause:** Genuine concern: wasip3 `0.4.0+wasi-0.3.0-rc-2026-01-06` is a release
  candidate exempted at safe-to-deploy and deserves a documented rationale + tracking issue.
  PARTIAL because the "dead weight" framing for the toml `+spec-1.1.0` pins misreads cargo-vet:
  exemption versions must EXACTLY match the lockfile entry (Cargo.lock pins
  `0.9.12+spec-1.1.0`, `0.7.5+spec-1.1.0`, `1.1.3+spec-1.1.0`), so the rows are live and
  correct today; cargo-vet regenerates them on graph re-resolution, so they cannot "linger" —
  they update automatically when upstream ships stable.
- **Evidence:** config.toml rows confirmed at 803-813 and 935-937; matching Cargo.lock entries
  confirmed (lines 1875-1900, 2150-2153); `cargo vet --locked` enforced in CI
  (.github/workflows/ci.yml:82-84).
- **Remediation:** Add a comment (or tracking issue reference) next to the wasip3 exemption
  explaining why the WASI p3 RC is acceptable for deployment (needed by the wasm toolchain of
  a dependency); confirm the file remains cargo-vet-generated and is never hand-edited beyond
  such comments. No row changes.

---

## INVALID findings

### F0782 — serde default "discards" stored schema_version_used on deserialize
- **File:lines:** src/validate/error_types.rs:79-80
- **One-line rationale:** INVALID — serde's `default = "current_schema_version"` fires ONLY
  when the key is ABSENT from the input; a present `schema_version_used` value is kept
  verbatim (Raw field, line 80, then passed to new_with_schema_context). Stored provenance is
  never discarded; the review misreads serde default semantics. The json_round_trip test
  (report.rs:230-241) confirms round-trip fidelity.

### F0783 — serialized model_type kept while schema context invented
- **File:lines:** src/validate/error_types.rs:75-76
- **One-line rationale:** INVALID — same serde misreading as F0782: `default =
  "unknown_model_type"` applies only when the key is absent. A stored `model_type` is
  deserialized as-is and paired with the stored `schema_version_used`, not a freshly invented
  one; no provenance mixing occurs.

### F0794 — multi-type schema message loses all but the last quoted type
- **File:lines:** src/validate/formatter.rs:139-140
- **One-line rationale:** INVALID — none of the four embedded OSCAL v1.2.3 schemas contains a
  multi-type `"type": [...]` declaration (grep across schemas/ found zero; all type keywords
  are single-valued), so the claimed `1 is not of type "string", "integer"` message shape can
  never be produced by this validator; `extract_trailing_quoted` always sees a single type.
  Theoretical only if schemas change; not a shipped defect.

### F1048 — asymmetric rand-chain clearance is brittle/wrong
- **File:lines:** supply-chain/config.toml:607-629 (rand exemptions)
- **One-line rationale:** INVALID — the split exactly mirrors the dependency graph: the 0.9
  chain (rand 0.9.4, rand_chacha 0.9.0, rand_core 0.9.5, ppv-lite86) is reachable ONLY via
  proptest/rand_xorshift (dev-dependencies; Cargo.lock confirms proptest 1.10.0 is the sole
  rand 0.9.4 dependent), so safe-to-run is correct, while rand 0.10.2 is pulled by lopdf
  (via pdf-extract, a normal dependency) and is correctly safe-to-deploy. CI runs
  `cargo vet --locked` (ci.yml:82-84), which fails immediately if the graph changes such that
  a deployed target pulls the 0.9 chain — the "brittle" failure mode is already guarded.

---

## DUPLICATE findings

(none — no two findings in this slice share a root cause)

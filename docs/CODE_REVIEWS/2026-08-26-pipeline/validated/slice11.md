# Validated slice: slice11 — 61 findings (all low severity)

Validation pass: 2026-08-26 against HEAD `b22e2d5` ("Harden successor map opening against symlink races").
Review baseline: docs/CODE_REVIEWS/ocr_review_2026-08-16.md.

**Summary:** 58 valid, 0 partial, 0 invalid, 3 duplicate. All findings were checked
against the current source; cited line numbers matched HEAD for every finding except
where noted. No finding was already fixed between review and HEAD.

---

## VALID findings

### F0703 — avoidable `title_buf.clone()` on per-heading hot path
- **File:lines:** src/parse/mod.rs:125-136
- **Symbol(s):** `extract_sections` (`Event::End(TagEnd::Heading)` arm)
- **Category:** performance | **Severity:** low
- **Root cause:** In the heading-end arm, the node is built with `title: title_buf.clone()`
  (line 126) even though the buffer is cleared and reused for the next heading
  (`title_buf.clear()` at line 119). `clear()` keeps capacity, so every heading pays one
  full allocation; `std::mem::take` would move the allocation out and leave an empty
  (zero-cap) String to be refilled.
- **Evidence:** `title_buf.clone()` confirmed at src/parse/mod.rs:126; `title_buf.clear()`
  at :119; buffer is only reused across headings, never observed between.
- **Remediation:** Replace `title: title_buf.clone()` with `title: std::mem::take(&mut title_buf)`.
  Behavior identical (the buffer is cleared before reuse on the next heading anyway).
  No snapshot impact; add no new test (pure perf; existing heading tests pin behavior).

### F0707 — whitespace-only body runs allocate `Some("\n")`
- **File:lines:** src/parse/mod.rs:168-195 (arm cited as 182-187)
- **Symbol(s):** `accumulate_body_text` (`Event::End(TagEnd::Paragraph|TagEnd::Item|TagEnd::List)` arm), `finalize_body`
- **Category:** other | **Severity:** low
- **Root cause:** `accumulate_body_text` unconditionally `get_or_insert_with`s a body
  (line 183) even for whitespace-only payloads (blank runs between headings emit
  Text("\n")/SoftBreak events that get appended at lines 171/180). Correctness is only
  rescued later by `finalize_body` trimming to `None`. The two functions silently share
  a normalization contract with no documentation or guard.
- **Evidence:** Lines 170-186 push text/newlines and always insert; `finalize_body` at
  :222-231 trims and converts empty to `None`; test `heading_with_no_body_has_none`
  (:510-515) passes only because of the finalize trim.
- **Remediation:** Either (a) in the `Text` arm skip accumulation when
  `text.trim().is_empty()` AND the current body is still `None` (avoids creating
  whitespace-only bodies), or (b) document on `accumulate_body_text` that
  `finalize_body` is the single normalization point. No output change; no snapshot impact.

### F0704 — `finalize_body` reallocates via `trim_end().to_string()`
- **File:lines:** src/parse/mod.rs:221-231
- **Symbol(s):** `finalize_body`
- **Category:** performance | **Severity:** low
- **Root cause:** `let trimmed = body.trim_end().to_string();` (line 224) allocates a
  fresh String for every non-empty body even when nothing needs trimming (the common
  case), doubling per-body memory during finalization.
- **Evidence:** Line 224 confirmed; `finalize_body` recurses over every section node
  (:230), so this runs once per section in the tree.
- **Remediation:** Trim in place: `let trimmed_len = body.trim_end().len(); if trimmed_len == 0 {
  node.body_text = None; } else { body.truncate(trimmed_len); }`. `truncate` on the
  computed boundary is safe because `trim_end()` returns a prefix slice of the same
  string at a char boundary. Identical output; no snapshot impact.

### F0706 — `offset_to_line` edge-case contract undocumented
- **File:lines:** src/parse/mod.rs:262-267
- **Symbol(s):** `offset_to_line`, `build_line_starts`
- **Category:** documentation | **Severity:** low
- **Root cause:** `partition_point(|&start| start <= offset)` (line 266) resolves an
  offset pointing at a trailing '\n' to the line before the next start, and an offset at
  EOF of a '\n'-terminated file yields `line_starts.len()` (a phantom line beyond the
  last real line). Neither convention is documented; downstream `src/model/assemble.rs`
  builds ownership ranges `[section.source_line, next_sibling.source_line)`
  (assemble.rs:116-123) from these values, so any change shifts requirement attribution.
- **Evidence:** `build_line_starts` pushes `i + 1` for every '\n' including the last
  (:253-257); callers in clauses.rs (:203, :289, :356) and mod.rs (:121) pass pulldown-cmark
  offsets which are always inside content, so the phantom-line case is unreachable today —
  hence documentation, not behavior fix.
- **Remediation:** Extend the doc comment on `offset_to_line` to pin the contract:
  "returns the 1-based line containing the byte at `offset`; newline bytes belong to the
  line they terminate; callers must pass offsets within the parsed content (offsets at/past
  EOF of a '\n'-terminated file resolve to a phantom line)". Optionally add
  `debug_assert!(line_starts.first() == Some(&0));`. No behavior change.

### F0708 — hard-coded `md_count == 25` fixture pin; error path untested
- **File:lines:** src/parse/mod.rs:573-594
- **Symbol(s):** `tests::all_example_policy_documents_produce_non_empty_sections`, `extract_sections`
- **Category:** test | **Severity:** low
- **Root cause:** Line 593 asserts exactly 25 `.md` files in `example_data/`, coupling
  this parsing test to the fixture inventory (adding/removing any sample breaks it for a
  non-parsing reason). Additionally `extract_sections` documents `ForgeError::Parse`
  (# Errors at :90-93) but no test exercises the error branch.
- **Evidence:** `assert_eq!(md_count, 25, ...)` at :593; example_data/ currently holds
  exactly 25 POL-*.md files; no test covers a Parse-error return.
- **Remediation:** Change to `assert!(md_count > 0, ...)` (or move the exact count to a
  dedicated fixture-inventory test). Optionally add a test documenting that
  `extract_sections` never actually returns `Err` with pulldown-cmark, or test the
  documented branch if a failure mode exists. No snapshot impact.

### F0685 — "Not part of the public API" claim is false for `modality` items
- **File:lines:** src/parse/modality.rs:40-57
- **Symbol(s):** `ModalityResult`, `detect_modality`
- **Category:** documentation | **Severity:** low
- **Root cause:** lib.rs declares `pub mod parse;` and parse/mod.rs declares
  `pub mod modality;`, so `forge::parse::modality::ModalityResult` / `detect_modality`
  are publicly reachable despite the doc comment "Not part of the public API — used
  internally by [`annotate_modalities`]" (line 43). External users can grow dependencies
  on a diagnostic type intended to be free-form.
- **Evidence:** lib.rs:56 `pub mod parse;`; parse/mod.rs:7 `pub mod modality;`;
  modality.rs:45 `pub struct ModalityResult`, :72 `pub fn detect_modality`. In-crate
  callers only today (annotate_section, pipeline step 7c).
- **Remediation:** Make both crate-private: `pub struct ModalityResult` →
  `pub(crate) struct ModalityResult` and `pub fn detect_modality` →
  `pub(crate) fn detect_modality`; replace the disclaimer line with "Internal
  classification output (crate-visible only).". Only `annotate_modalities` (re-exported
  at parse/mod.rs:19) needs to stay public.

### F0686 — `is_default`/`has_conflict` booleans encode a four-way outcome
- **File:lines:** src/parse/modality.rs:45-57
- **Symbol(s):** `ModalityResult` fields `is_default`, `has_conflict`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `ModalityResult` encodes four mutually exclusive outcomes
  (normative/advisory/default/conflict) in two booleans plus `modality`; mutual
  exclusivity is enforced only by convention and the unit test
  `invariant_is_default_and_has_conflict_mutually_exclusive` (:352-371). Illegal
  combinations (`is_default = true && has_conflict = true`) remain constructible.
- **Evidence:** Fields at :53, :56; all four construction sites in `detect_modality`
  (:88-118) set the booleans by hand; invariant test exists precisely because the type
  doesn't enforce it.
- **Remediation:** Introduce `pub(crate) enum ModalityOutcome { ExplicitNormative,
  ExplicitAdvisory, Default, Conflict }` (or carry verbs variant-wise), replace the two
  booleans with one `outcome` field, update `annotate_section`'s `is_default ||
  has_conflict` checks (:146-155) to match arms, and delete/replace the invariant test
  (the enum makes it provable). Tests asserting `is_default`/`has_conflict`
  (T008/T009/T024/T025/T036/T037) must be updated to assert the outcome variant.

### F0687 — per-hit lowercase allocations in `detect_modality`
- **File:lines:** src/parse/modality.rs:75-79
- **Symbol(s):** `detect_modality`
- **Category:** performance | **Severity:** low
- **Root cause:** Every regex hit is lowercased into a fresh `String` (lines 75-76,
  78-79), and BOTH vectors are built unconditionally even when one side is discarded
  (pure-normative arm drops `advisory_hits` after building it). Over full-document
  enrichment this is O(total keyword hits) heap allocations for presence flags.
- **Evidence:** `captures_iter(text).map(|c| c[1].to_lowercase()).collect()` at :75-76
  and :78-79; `matched_verbs` is used only for `tracing::debug!`/`warn!` output
  (:146-163) and test assertions, so borrowed `&str` suffices.
- **Remediation:** Collect `Vec<&str>` via `c[1].as_str()` (captures borrow from
  `text`); make `ModalityResult.matched_verbs` either `Vec<String>` built only when a
  warn/debug log is actually emitted, or lifetime-parameterize the crate-private type.
  Keep debug output identical.

### F0729 — citation→resource UUID map discarded; resources ship unreferenced
- **File:lines:** src/pipeline.rs:172-175
- **Symbol(s):** `run_catalog_pipeline`, `crate::oscal::generate_back_matter`, `generate_control_links`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `let (back_matter_resources, _resource_map) = generate_back_matter(...)`
  (line 174) discards the citation-id→UUID map. `generate_control_links` (the API built
  to consume it, oscal/back_matter.rs:295) is only ever called from tests and is
  re-exported from lib.rs (:85) as public API, so every emitted back-matter resource has
  no `href="#<uuid>"` link from any control.
- **Evidence:** Pipeline never calls `generate_control_links`; grep confirms only test
  call sites (back_matter.rs tests). Shipped catalogs contain resources no control references.
- **Remediation:** Either wire the map through — during `build_catalog`/trace embedding,
  attach `OscalLink { href: format!("#{uuid}"), rel: "reference", text }` to the control
  whose requirement text produced each citation — or document at the `_resource_map`
  binding (rename to `resource_map` + comment) that links are intentionally deferred.
  If wiring, snapshots containing back-matter change (new links on controls) — update via
  `cargo insta review`.

### F0730 — hand-rebuilt catalog silently drops builder fields
- **File:lines:** src/pipeline.rs:184-195
- **Symbol(s):** `run_catalog_pipeline`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The pipeline constructs a fresh `OscalCatalog { uuid, metadata, controls:
  vec![], groups: catalog.groups, back_matter }` instead of mutating the `build_catalog`
  result. `build_catalog` currently returns `controls: vec![]` (oscal/catalog.rs:435), so
  this is harmless today, but any future top-level controls or new `OscalCatalog` fields
  would be silently deleted here.
- **Evidence:** pipeline.rs:184-195 vs catalog.rs:427-438; only `uuid`, `metadata`, and
  `back_matter` are the placeholders needing replacement.
- **Remediation:** `let mut oscal_catalog = catalog; oscal_catalog.uuid =
  real_metadata.uuid.to_string(); oscal_catalog.metadata = OscalMetadata { ... };
  oscal_catalog.back_matter = back_matter;`. Identical output today; no snapshot impact.

### F0725 — serialize→parse round-trip for validation input
- **File:lines:** src/pipeline.rs:44-62
- **Symbol(s):** `validate_and_serialize`
- **Category:** performance | **Severity:** low
- **Root cause:** `serde_json::to_string_pretty(envelope)` (:45) is immediately followed
  by `serde_json::from_str(&json)` (:47-48) to obtain a `Value` for schema validation.
  The parse re-does the full lexing work; `serde_json::to_value(envelope)` performs the
  same Serialize validation directly.
- **Evidence:** Lines 45-48 confirmed; the pretty string is re-emitted at the end (`Ok(json)`
  :61), so the final output must remain pretty-serialized.
- **Remediation:** `let json_value = serde_json::to_value(envelope).map_err(|e|
  ForgeError::Serialization(e.to_string()))?;` → validate `json_value` → then
  `let json = serde_json::to_string_pretty(&json_value)...?`. Same error semantics;
  output byte-identical.

### F0727 — "output will have empty groups" warning misstates clause-only behavior
- **File:lines:** src/pipeline.rs:97-99
- **Symbol(s):** `prepare_document`
- **Category:** documentation | **Severity:** low
- **Root cause:** When `sections.is_empty()` but clauses exist (the SEC-5 clause-only
  path), `assemble_document`→`map_sections` synthesizes a single "Preamble" section
  (model/assemble.rs:57-64) from orphaned list items, so the catalog gets one non-empty
  "Preamble" group — not empty groups. The warning at pipeline.rs:98 misleads operators.
  (Note: if clauses contain only tables/paragraphs without list items the document is
  empty, so the message should mention both outcomes.)
- **Evidence:** assemble.rs:57-64 Preamble fallback; pipeline.rs:97-99 warning text;
  `has_clause_structure` at :94 gates this path.
- **Remediation:** Replace the warning text with e.g. "No identifiable sections found in
  input — list-item content will be grouped under a single synthetic 'Preamble' group".
  Logging-only change; no snapshots.

### F0670 — successful `ConvertResult.warnings` discarded in round-trip chain
- **File:lines:** src/round_trip/chain.rs:49
- **Symbol(s):** `run_round_trip_chain`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `invoker.convert(&args)?;` (line 49) drops the `ConvertResult`
  wholesale, so non-fatal stderr warnings (oscal-cli exit 0 with stderr, defined as
  `ConvertResult.warnings` in oscal_cli/mod.rs:95-100) never surface. For a round-trip
  fidelity pipeline these warnings are exactly the coercion signals operators need.
- **Evidence:** chain.rs:49; `ConvertResult { output_path, warnings }` confirmed;
  chain tests use `warnings: vec![]` only.
- **Remediation:** `let result = invoker.convert(&args)?; for warning in &result.warnings
  { tracing::warn!(step = step_num + 1, "oscal-cli convert warning: {warning}"); }`.
  Extend `MockInvoker` with canned warnings and assert they are not lost.

### F0697 — `json_path` built without RFC 6901 escaping in comparator
- **File:lines:** src/round_trip/comparator.rs:39, 58 (array-index paths elsewhere are numeric and fine)
- **Symbol(s):** `compare_values` (object arms)
- **Category:** bug | **Severity:** low
- **Root cause:** `format!("{path}/{key}")` (lines 39 and 58) interpolates raw JSON
  object keys. Any key containing `/` or `~` produces an invalid JSON Pointer, so
  consumers resolving `Divergence.json_path` address the wrong node. The codebase
  already implements the correct escape in `src/testing/semantic_eq.rs:52-54`
  (`escape_json_pointer_token`).
- **Evidence:** comparator.rs:39, :58 use bare `format!`; `Divergence.json_path` is
  documented as "RFC 6901 JSON Pointer" (divergence.rs:10-11).
- **Remediation:** Add a local `fn escape_json_pointer_token(token: &str) -> String {
  token.replace('~', "~0").replace('/', "~1") }` (or hoist to a shared location) and use
  `format!("{path}/{}", escape_json_pointer_token(key))` at both sites. Add a unit test
  with a key like `"a/b~c"` asserting the emitted path is `/a~1b~0c`. No impact on
  current snapshots (OSCAL keys are kebab-case identifiers today).

### F0675 — inconsistent enum serialization casing in divergence report + duplicated Display
- **File:lines:** src/round_trip/divergence.rs:26-70
- **Symbol(s):** `CompatibilityClassification`, `DivergenceClass`, `ResolutionStatus`, `Display for CompatibilityClassification`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `CompatibilityClassification` serializes kebab-case
  (`#[serde(rename_all = "kebab-case")]`, line 49) while sibling enums
  `DivergenceClass`/`ResolutionStatus` (:26-46) serialize as PascalCase; the same
  divergence-log JSON therefore mixes "advisory-older-model-baseline" with "ForgeFix".
  The hand-written `Display` (:62-70) duplicates the rename_all strings, so a future
  rename of one silently diverges from the other.
- **Evidence:** divergence.rs:27 `ForgeFix` (no rename_all) vs :49 kebab-case attribute;
  Display impl at :62-70 hardcodes the same three strings.
- **Remediation:** Pick one convention for all report enums (kebab-case recommended) and
  apply `rename_all` uniformly. strum is NOT a dependency (verified in Cargo.toml), so
  derive-free: pin Display↔serde correspondence with a test — for each variant assert
  `serde_json::to_value(v).unwrap() == Value::String(v.to_string())`. Changing
  `DivergenceClass` casing changes existing divergence-log JSON; if backwards compat
  matters, keep PascalCase for those two and only back the duplicated Display strings
  with the correspondence test. Snapshot `forge__round_trip__log__tests__*.snap` files
  update if serialized values change.

### F0674 — exact `"1.0.3"` string match for version classification
- **File:lines:** src/round_trip/divergence.rs:76-84
- **Symbol(s):** `classify_oscal_cli_compatibility`
- **Category:** bug | **Severity:** low
- **Root cause:** `Some("1.0.3")` (line 79) matches only the exact string. The version
  comes from `parse_version_from_output` (oscal_cli/detector.rs:136) which grabs the first
  whitespace-delimited token starting with a digit and containing '.' — so "1.0.3+build"
  or prerelease suffixes fall through to `Some(_)` and still classify
  `AdvisoryOlderModelBaseline` but WITHOUT the documented `Some("1.1.2")` model
  baseline. Classification is inconsistent for the same real tool version.
- **Evidence:** divergence.rs:79; detector.rs:136-146 returns raw token incl. suffixes;
  test `unknown_cli_baseline_is_advisory` (:132-135) confirms the catch-all drops the
  baseline.
- **Remediation:** Strip trailing metadata/prerelease (e.g. `version.split('+').next()
  .and_then(|v| v.split('-').next())`) or parse with semver before matching, so
  legitimate 1.0.3 banners always map to the documented baseline. Add tests for
  "1.0.3+build" and "1.0.3-rc.1" pinning the intended classification.

### F0679 — companion `ArtifactType` enum definition (style follow-up to F0678)
- **File:lines:** src/round_trip/divergence.rs:88-89
- **Symbol(s):** proposed `ArtifactType` enum
- **Category:** style | **Severity:** low
- **Root cause:** Companion finding to F0678: the strongly-typed enum replacing
  `artifact_type: String` needs a definition with serde derives.
- **Evidence:** Same locus as F0678.
- **Remediation:** Implement as part of F0678: `#[derive(Debug, Clone, Copy, PartialEq,
  Eq, Serialize, Deserialize)] pub enum ArtifactType { Catalog, ComponentDefinition }`
  with `Display` mirroring the existing string values ("Catalog"/"ComponentDefinition")
  used in log.rs test fixtures. See F0678 for caller migration.

### F0677 — `RoundTripResult` lacks symmetric derives
- **File:lines:** src/round_trip/divergence.rs:88-89
- **Symbol(s):** `RoundTripResult`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `#[derive(Debug, Serialize)]` (line 88) while the child type
  `Divergence` (:8) derives Debug/Clone/Serialize/Deserialize/PartialEq. The aggregate
  record of a round-trip validation run cannot itself be deserialized/compared/cloned —
  consumers cannot reload a divergence log for diffing or regression testing.
- **Evidence:** divergence.rs:88; `write_divergence_log` serializes `RoundTripResult`
  to disk (log.rs:17-27), confirming reload is the natural use.
- **Remediation:** `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`. Requires
  all fields' types to implement Deserialize/PartialEq (PathBuf, Option<String>, bool,
  Vec<Divergence>, CompatibilityClassification all do). Add a round-trip test:
  serialize → deserialize → assert equality.

### F0678 — `artifact_type: String` allows arbitrary values
- **File:lines:** src/round_trip/divergence.rs:90-91
- **Symbol(s):** `RoundTripResult.artifact_type`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The field documents exactly two legal values but accepts any string.
  The sole producer is src/cli/validate.rs:250 which already maps from `OscalModelType`
  (or "Unknown" when detection failed), so a closed type fits; "Unknown" is the only
  extra case to model.
- **Evidence:** divergence.rs:90-91; cli/validate.rs:249-250, :257; :310, :326 print it;
  log.rs tests construct "Catalog"/"ComponentDefinition".
- **Remediation:** Use `ArtifactType` from F0679 (or `Option<ArtifactType>` with
  None = unknown). Update cli/validate.rs:250 construction and the two `result.artifact_type`
  print sites (:310, :326) to use `Display`. Update log.rs test fixtures accordingly;
  keep serialized strings identical so snapshots don't change.

### F0683 — snapshots parse JSON to Value, hiding formatting regressions
- **File:lines:** src/round_trip/log.rs:30-148 (tests)
- **Symbol(s):** `tests::clean_pass_result_snapshot`, `divergences_with_resolutions_snapshot`, `unresolved_divergence_shows_null_resolution_snapshot`
- **Category:** test | **Severity:** low
- **Root cause:** Every test does `serde_json::from_str(&read_to_string(...))` then
  `assert_json_snapshot!`, normalizing away whitespace. `write_divergence_log`'s
  documented contract is *pretty-printed* output (`to_writer_pretty`, line 21), but the
  snapshots can't detect a regression to compact output.
- **Evidence:** log.rs:57-59, :101-103, :137-139 parse into `Value` before snapshotting.
- **Remediation:** Add one test asserting on raw text: `let raw = std::fs::read_to_string
  (&output).unwrap(); insta::assert_snapshot!("pretty_format", raw);` — the snapshot then
  pins indentation/line breaks. Keep the Value snapshots for structure.

### F0681 — `File::create` error lacks path context
- **File:lines:** src/round_trip/log.rs:20
- **Symbol(s):** `write_divergence_log`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `let file = std::fs::File::create(output_path)?;` (line 20) converts
  `io::Error` via `From` into `ForgeError::Io` with no indication of which path failed.
  On a missing parent dir during a batch run the operator sees a bare "No such file or
  directory".
- **Evidence:** log.rs:20; context-enriched `ForgeError::Serialization(String)` pattern
  is used elsewhere (pipeline.rs:46).
- **Remediation:** `let file = std::fs::File::create(output_path).map_err(|e|
  ForgeError::Serialization(format!("failed to create divergence log '{}': {e}",
  output_path.display())))?;`. Update tests if any assert the error variant for create
  failure (none do today).

### F0682 — serialization error message lacks context
- **File:lines:** src/round_trip/log.rs:25
- **Symbol(s):** `write_divergence_log`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `ForgeError::Serialization(e.to_string())` (line 25) surfaces the bare
  serde message with no hint it came from the divergence log. The finding concedes the
  string flattening itself matches sibling modules; the actionable part is context.
- **Evidence:** log.rs:21-25.
- **Remediation:** `return Err(ForgeError::Serialization(format!("failed to serialize
  divergence log: {e}")));`. No test impact.

### F0699 — re-export path must follow any `log` module rename (follow-up to F0698)
- **File:lines:** src/round_trip/mod.rs:18
- **Symbol(s):** `pub use log::write_divergence_log;`
- **Category:** maintainability | **Severity:** low
- **Root cause:** Companion to F0698 — the re-export at mod.rs:18 and any internal
  `super::log` references must be updated together with the rename.
- **Evidence:** mod.rs:18; log.rs tests use `use super::*` (unaffected by module rename
  within the same file).
- **Remediation:** When executing F0698, change to `pub use divergence_log::write_divergence_log;`.

### F0698 — private `mod log` shadows the `log` facade crate name
- **File:lines:** src/round_trip/mod.rs:9
- **Symbol(s):** `mod log;`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `mod log;` (line 9) collides with the ubiquitous `log` crate name.
  Nothing misresolves today (`log` is not a direct dependency — verified in Cargo.toml),
  but adding it later, or writing `use log::...` inside this module tree, would silently
  bind to the divergence-log module. The name also misleads: it writes a structured
  divergence report, not diagnostics.
- **Evidence:** mod.rs:9; no `log` dependency in Cargo.toml (grep confirmed).
- **Remediation:** Rename file `src/round_trip/log.rs` → `src/round_trip/divergence_log.rs`,
  `mod divergence_log;`, update re-export per F0699. Snapshot files under
  src/round_trip/snapshots/ are keyed by test path within the same module, so they remain
  valid as long as the test module path is preserved.

### F0702 — `OscalComparisonRules` exposes raw collections, no derives
- **File:lines:** src/round_trip/rules.rs:5-24
- **Symbol(s):** `OscalComparisonRules`, `Default for OscalComparisonRules`
- **Category:** style | **Severity:** low
- **Root cause:** The public config struct has no `Debug`/`Clone` derives and exposes
  `HashSet<String>`/`Vec<String>` fields with no validation, coupling callers to the
  storage representation. The `Default` impl also builds the set via `.iter().map().collect()`
  (lines 16-20) instead of `HashSet::from`.
- **Evidence:** rules.rs:6-24; the struct is `pub` and re-exported (round_trip/mod.rs:19).
- **Remediation:** `#[derive(Debug, Clone)]` on the struct; rewrite Default as
  `unordered_array_paths: HashSet::from(["props".to_string(), "links".to_string(),
  "parts".to_string()]), ignored_paths: Vec::new()`. Keep fields public for now
  (callers: cli/validate.rs, comparator tests).

### F0717 — tab/newline preservation trade-off undocumented
- **File:lines:** src/sanitize.rs:1-12
- **Symbol(s):** `strip_control_chars`
- **Category:** documentation | **Severity:** low
- **Root cause:** The doc comment (:1-2) states the behavior but not the caller
  obligation: preserving LF/HT lets untrusted input forge extra plausible lines in
  single-line surfaces (logs, CI annotations, status bars). The SEC-5 contract stays
  implicit.
- **Evidence:** sanitize.rs:1-11 preserves 0x09/0x0A by design; used in the trace
  formatter for element ids/sections which can carry attacker-influenced markdown headings.
- **Remediation:** Extend the doc comment: "Tab and newline are deliberately preserved.
  Callers feeding untrusted input into single-line surfaces (log entries, status bars)
  must additionally collapse newlines/tabs, since embedded LFs allow forging extra
  plausible output lines." No behavior change.

### F0718 — unit tests miss numeric-predicate boundaries
- **File:lines:** src/sanitize.rs:15-52 (tests)
- **Symbol(s):** `tests::strip_ansi_escape` et al., predicate in `strip_control_chars`
- **Category:** test | **Severity:** low
- **Root cause:** Tests cover ESC, DEL, tab/newline, empty, and clean strings, but never
  NUL (0x00), CR (0x0D), VT/FF (0x0B/0x0C), the 0x1F-stripped vs 0x20-kept boundary, or
  multibyte non-ASCII passthrough. An off-by-one like `b >= 0x21` would pass all current
  tests.
- **Evidence:** sanitize.rs tests :15-52; predicate at :6-10 uses `b >= 0x20 && b != 0x7F`.
- **Remediation:** Add boundary tests:
  `assert_eq!(strip_control_chars("a\u{0}\r\u{b}\u{c}\u{1}b\u{1f} c"), "ab c")`;
  `assert_eq!(strip_control_chars("中文 café"), "中文 café")`; DEL residue already pinned.
  Note the finding's suggested "\u{9b}[31m comes out empty" assertion FAILS today (C1
  0x9B passes through the predicate), so keep that test out until a C1 fix lands or mark
  it as documenting the gap.

### F0760 — error-message list and error counter can disagree
- **File:lines:** src/summary/format.rs:110-126
- **Symbol(s):** `format_validation_detail_lines`
- **Category:** bug | **Severity:** low
- **Root cause:** `shown` is capped by `msgs.len()` (line 117) while `total` is the
  separate `validation_errors` counter (line 116). If a caller stores more than 3 messages
  while `validation_errors <= 3`, extras are silently dropped with no "and N more..."
  indicator — a self-contradicting listing. Currently production never populates either
  (pipeline aborts on validation failure, per summary/mod.rs:44-53 docs), so this is a
  latent invariant break for the formatter's contract; tests today only exercise the
  consistent direction (errors=5 with 3 messages).
- **Evidence:** format.rs:114-123; ConversionStatistics docs confirm zero in production.
- **Remediation:** Defensive clamp at the top: `let total = total.max(msgs.len());` so
  the listing never contradicts the stored messages. No snapshot impact (existing tests
  already have total >= msgs.len()).

### F0758 — unchecked usize subtraction chain depends on silent constant couplings
- **File:lines:** src/summary/format.rs:155-157 (title math :164-170)
- **Symbol(s):** `format_summary_dashboard`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `let vw = w - LABEL_WIDTH + 1;` (line 157) and the title padding math
  (`w - title.len()`, :165) rely on MIN_WIDTH(41) > LABEL_WIDTH(22) > title byte
  length(24). Lengthening a label, mis-sizing MIN_WIDTH, or making the title
  longer/non-ASCII (`title.len()` counts bytes) turns these into debug panics / release
  wrapped widths with silent box misalignment.
- **Evidence:** format.rs:16-17 constants; :157; :164-170. Math is safe today (verified:
  MIN_WIDTH 41 > LABEL_WIDTH 22; title "FORGE Conversion Summary" = 24 bytes < 41).
- **Remediation:** Add `const _: () = assert!(MIN_WIDTH > LABEL_WIDTH);` at module level
  and `debug_assert!(w >= title.len(), ...)` before the title padding.

### F0759 — escape scanner assumes SGR/CSI termination on ASCII letters
- **File:lines:** src/summary/format.rs:22-38
- **Symbol(s):** `visible_len`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `visible_len` ends an escape run at any ASCII alphabetic char
  (lines 27-29), correct only for the module's own SGR sequences. OSC sequences
  (terminated by BEL/ST) or letters inside escape payloads would be miscounted, shrinking
  visible length and breaking column alignment. Today all colored input comes from the
  module's own constants (GREEN/RED/YELLOW/BOLD/RESET), so the invariant holds but is
  undocumented.
- **Evidence:** format.rs:23-38; color constants at :9-13; `color()` (:18-20) and
  dashboard rows are the only producers.
- **Remediation:** Document the supported-subset invariant next to `visible_len`:
  "Assumes SGR-style CSI sequences (ESC '[' + params + final letter). OSC sequences or
  letters inside escape payloads are NOT handled; all colored input must originate from
  this module's constants." Optionally add a unit test pinning `visible_len` for a
  BOLD+RESET string.

### F0720 — hand-written `Default` bakes `strategy: Strategy::Catalog` sentinel
- **File:lines:** src/summary/mod.rs:63-79
- **Symbol(s):** `Default for ConversionStatistics`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The hand-written `Default` (:63-79) duplicates the full field list and
  hardcodes `strategy: Strategy::Catalog` (:73), so an unpopulated statistics record
  already advertises a concrete conversion strategy — the dashboard renders
  "Strategy: Catalog" for never-run records with no sentinel. Both pipelines overwrite it
  (pipeline.rs:215, :364), but the default masquerades as a real run.
- **Evidence:** mod.rs:63-79; both `run_catalog_pipeline` and `run_component_pipeline`
  set `strategy` explicitly via struct-literal update.
- **Remediation:** Either document that `strategy: Strategy::Catalog` in `Default` is
  a placeholder callers must overwrite, or model it as `Option<Strategy>` with None =
  not run (then the display site format.rs:179 `{:<vw$}` on `stats.strategy` needs an
  unwrap_or fallback). Simplest safe step: keep the type, add the doc note.

### F0721 — `count_catalog_controls` depth>64 truncation branch untested
- **File:lines:** src/summary/mod.rs:96-103
- **Symbol(s):** `count_catalog_controls`, `count_group_controls`
- **Category:** test | **Severity:** low
- **Root cause:** The `if depth > 64` early-return (:99-101) has no coverage; all tests
  build shallow catalogs. The truncation behavior (which silently under-counts by not
  descending into groups below the cutoff) is unverified.
- **Evidence:** mod.rs:99-101; tests at :152-316 use ≤3 nesting levels.
- **Remediation:** Add `count_catalog_controls_deep_nesting`: build a chain of 66+
  groups each carrying one control (construct `OscalControl` inline — there is no
  `make_control_for_test` helper; reuse the closure pattern from
  `count_catalog_controls_multiple_groups`) and assert the expected count. Either pin
  today's truncated count (compute exactly when writing the test) or switch
  `count_group_controls` to an iterative traversal and assert the full count.

### F0714 — testing module compiled unconditionally into release builds
- **File:lines:** src/testing/mod.rs:6 (and src/lib.rs:64-65)
- **Symbol(s):** `pub mod testing;`, `pub mod semantic_eq;`
- **Category:** test | **Severity:** low
- **Root cause:** `pub mod testing;` in lib.rs (:65, already `#[doc(hidden)]`) has no
  cfg gate, so `semantic_eq`'s comparison logic ships in every downstream consumer's
  release build despite being test-only.
- **Evidence:** lib.rs:64-65; testing/mod.rs:6; consumers are integration tests
  (tests/integration_round_trip.rs:7, tests/round_trip_test.rs:18).
- **Remediation:** Gate the module: `#[cfg(any(test, feature = "testing"))] pub mod testing;`
  in lib.rs and mirror on `pub mod semantic_eq;` in testing/mod.rs; add a `testing`
  feature to Cargo.toml (Cargo.toml currently has no [features] table). Verify
  `cargo test` still links (integration tests of the same crate need the feature enabled
  for dev builds — simplest: gate on `any(test, feature = "testing")` and enable the
  feature in CI, or keep doc-examples compiling). If gating is too disruptive, keep
  ungated but document the decision.

### F0715 — dual public paths for `semantic_eq` items
- **File:lines:** src/testing/mod.rs:6-8
- **Symbol(s):** `pub mod semantic_eq;`, re-export list
- **Category:** maintainability | **Severity:** low
- **Root cause:** `semantic_eq` is `pub` AND its items are re-exported, so
  `forge::testing::EquivalenceResult` and `forge::testing::semantic_eq::EquivalenceResult`
  are both public — two stability commitments for one item.
- **Evidence:** testing/mod.rs:6-8; integration tests use the flat path
  (`forge::testing::assert_semantic_equivalence`).
- **Remediation:** Drop `pub` from the module: `mod semantic_eq;` — keep the flat
  re-export as the only public entry point. No caller uses the nested path (verified via
  grep over tests/ and src/).

### F0738 — array length mismatch reports only one aggregate diff
- **File:lines:** src/testing/semantic_eq.rs:102-124
- **Symbol(s):** `compare_arrays`
- **Category:** other | **Severity:** low
- **Root cause:** When lengths differ, only `min_len` elements are compared (:120-124);
  surplus/missing element contents are never examined, so round-trip failure reports show
  a single "array length mismatch" instead of the offending data.
- **Evidence:** semantic_eq.rs:112-124; the mismatch diff records lengths as strings
  (:115-119), not contents.
- **Remediation:** Either document the coarse-granularity contract on `compare_arrays`
  and `assert_semantic_equivalence` (order-significant comparison cannot reliably align
  shifted lists), or push per-index diffs for the surplus range:
  `for i in min_len..exp_arr.len() { diffs.push(EquivalenceDiff { path: format!("{path}/{i}"),
  description: "missing element".into(), expected: Some(format_value(&exp_arr[i])), actual: None }); }`
  plus the symmetric actual-extra loop. Add a test asserting the chosen contract.

### F0735 — byte/char metric mismatch in truncation; `Value::String` never truncated
- **File:lines:** src/testing/semantic_eq.rs:166-181
- **Symbol(s):** `format_value`
- **Category:** bug | **Severity:** low
- **Root cause:** The threshold `s.len() > 500` (:175) is bytes, but truncation
  `chars().take(500)` and the "(N chars total)" suffix are chars. A ~300-char multibyte
  payload (~900 bytes) crosses the byte threshold, gets no truncation (already ≤500
  chars), yet acquires a misleading suffix. Worse, the `Value::String` arm (:171)
  returns `format!("\"{s}\"")` with NO truncation at all — a multi-MB string floods
  the diff output verbatim.
- **Evidence:** semantic_eq.rs:171 (String arm untruncated); :175-177 mixed metrics;
  existing test `format_value_truncates_multibyte_json_values_without_panic` (:541)
  only proves no panic, not consistent metrics.
- **Remediation:** Measure both threshold and total in one unit (chars preferred):
  `let char_count = s.chars().count(); if char_count > 500 { ... }`. Apply the same
  guard to the `Value::String` arm. Update the multibyte test to assert exact suffix
  semantics.

### F0737 — doc comment for `compare_values` sits on `escape_json_pointer_token`
- **File:lines:** src/testing/semantic_eq.rs:47-54
- **Symbol(s):** `escape_json_pointer_token`, `compare_values`
- **Category:** documentation | **Severity:** low
- **Root cause:** The first two doc lines (:47-49, "Recursive comparison of JSON Value
  nodes… Accumulates all differences…") describe `compare_values` (defined at :56 with
  NO doc comment), but they're attached to `escape_json_pointer_token` (:52).
- **Evidence:** semantic_eq.rs:47-56 confirmed; `compare_values` has no `///` block.
- **Remediation:** Move the two sentences onto `compare_values`; leave only the RFC 6901
  escaping description on `escape_json_pointer_token`. Doc-only change.

### F0736 — `compare_values` recursion has no depth bound
- **File:lines:** src/testing/semantic_eq.rs:56-67
- **Symbol(s):** `compare_values`
- **Category:** bug | **Severity:** low
- **Root cause:** Recursion depth is unbounded. serde_json caps parse nesting at 128, but
  `Value`s built programmatically (test builders, loops) bypass that; a deeply nested
  input overflows the stack and aborts the whole test process instead of yielding a diff.
- **Evidence:** semantic_eq.rs:56; recursion via `compare_objects`/`compare_arrays` →
  `compare_values`.
- **Remediation:** Add a `depth: usize` parameter (starting at 0), and at a cap (e.g.
  128, matching serde_json's default) push a single `EquivalenceDiff { description:
  "nesting deeper than 128 levels".into(), ... }` and stop descending. Thread through
  `compare_objects`/`compare_arrays`. Add a test building a ~150-deep nested object
  asserting a non-equivalent result with the cap message rather than panicking.

### F0724 — sentinel-based absence encoding in trace metadata extraction
- **File:lines:** src/trace/extractor.rs:38-46
- **Symbol(s):** `extract_trace_metadata`
- **Category:** maintainability | **Severity:** low
- **Root cause:** Absence is encoded with raw sentinels: `source_file.unwrap_or_default()`
  ("" = unattributed), `source_line.unwrap_or(0)` (0 = no line), and the mapping gate
  `let section = source_section?;` (:39) accepts an EMPTY `source-section` value as
  "mapped" — conflating unmapped with mapped-to-empty-heading. Consumers use three
  different conventions (`source_section != ""`, `source_line == 0`, `source_file == ""`).
- **Evidence:** extractor.rs:39-46; downstream: formatter.rs:131 checks
  `source_line == 0 && element_type == Group`; report.rs:40-41 documents the 0-sentinel.
- **Remediation:** Short term: `let section = source_section.filter(|s| !s.is_empty())?;`
  and document the sentinel conventions on `TraceMetadata`. Long term: model with
  `Option<String>`/`Option<usize>` in `TraceMetadata` and update formatter/resolver
  consumers. Add a test: element with `source-section` value `""` — currently returns
  `Some` (mapped); pin the chosen semantics after the fix.

### F0773 — footer statistics taken from `summary` field, not rendered entries
- **File:lines:** src/trace/formatter.rs:109-121
- **Symbol(s):** `format_trace_table`
- **Category:** maintainability | **Severity:** low
- **Root cause:** The footer (:109-118) prints totals from `report.summary` while the
  table rows render from `report.entries`. `TraceReport` has public fields and cannot
  enforce `summary == TraceSummary::from_entries(&entries)`; a desynchronized builder
  produces a footer contradicting the rendered rows — corrupted audit evidence.
- **Evidence:** formatter.rs:110-118 vs rows from `report.entries` (:21-42); test helper
  `make_report` always builds summary from entries, hiding the hazard.
- **Remediation:** Recompute locally: `let s = TraceSummary::from_entries(&report.entries);`
  or add `debug_assert_eq!(report.summary.total_elements, report.entries.len())` before
  printing. No snapshot impact with consistent inputs.

### F0772 — two of three `format_source_line` outcome branches untested
- **File:lines:** src/trace/formatter.rs:124-142
- **Symbol(s):** `format_source_line`
- **Category:** test | **Severity:** low
- **Root cause:** Three branches exist: Group+line 0 → em dash (:131-133), other types
  line 0 → "0 ⚠" (:134-136), out-of-range line → "{line} ⚠" (:138-141). Tests exercise
  only the em-dash path (`format_with_group_em_dash`, :232-245) and in-range lines; the
  two warning branches have zero coverage.
- **Evidence:** formatter.rs:131-141; test list at :150-291 contains no direct
  `format_source_line` tests.
- **Remediation:** Add direct unit tests in the same module:
  `format_source_line(0, ElementType::Control, 100) == "0 ⚠"`,
  `format_source_line(101, ElementType::Control, 100) == "101 ⚠"` (⚠ is '\u{26A0}'),
  and an `ImplementedRequirement` line-0 case.

### F0771 — rows eagerly materialized as `Vec<[String; 4]>` for two-pass widths
- **File:lines:** src/trace/formatter.rs:21-55
- **Symbol(s):** `format_trace_table`
- **Category:** performance | **Severity:** low
- **Root cause:** All rows are materialized into `Vec<[String; 4]>` (:21-42) — ~5 heap
  allocations per entry — purely to compute widths in a second pass. Widths could be
  computed in one pass over `&TraceEntry` (recomputing char counts), then rows streamed
  directly from `report.entries.iter()`.
- **Evidence:** formatter.rs:21-42 allocation; :44-55 width pass; rows consumed only once
  afterwards (:95-107).
- **Remediation:** Replace the materialized rows with a closure `display_cells(entry) ->
  [String; 4]`: first pass computes widths from the closure's output (or recomputes
  char widths), second pass writes rows streaming from `report.entries.iter()`.
  Preserve exact output — existing snapshots (`format_mapped_entries` etc.) must pass
  unchanged.

### F0769 — header widths in bytes, cell widths in chars
- **File:lines:** src/trace/formatter.rs:44-55
- **Symbol(s):** `format_trace_table`
- **Category:** bug | **Severity:** low
- **Root cause:** `widths[i] = header.len()` (:46) uses byte length while cells use
  `chars().count()` (:50), but `{:<w$}` padding counts chars. With today's ASCII headers
  both coincide; a future non-ASCII header would get an undersized width. Neither metric
  equals terminal display width (CJK/combining chars survive `strip_control_chars`), so
  wide-character element ids misalign columns regardless.
- **Evidence:** formatter.rs:46 vs :50; headers at :18 are ASCII constants today.
- **Remediation:** Use one metric everywhere: `widths[i] = header.chars().count();` via
  a shared `display_width` closure. If exact terminal alignment for CJK matters, add
  the `unicode-width` crate (new dependency — check with maintainers first). Snapshots
  unchanged (ASCII).

### F0753 — `# Errors` block for `generate_trace_report` is incomplete/inaccurate
- **File:lines:** src/trace/mod.rs:31-36
- **Symbol(s):** `generate_trace_report`
- **Category:** documentation | **Severity:** low
- **Root cause:** The `# Errors` doc (:31-36) says `TraceUnsupportedArtifact` is raised
  only for "unrecognized" artifacts, but `detect_artifact_type` (walker.rs:19-52) also
  rejects recognized-but-unsupported (Profile), ambiguous (both keys), and non-object
  roots. It also omits `ForgeError::FileTooLarge` (read_file's `check_file_size` guard,
  mod.rs:97-103) and `ForgeError::PermissionDenied` (read_file :106-108).
- **Evidence:** trace/mod.rs:31-36; walker.rs:19-52; mod.rs:97-111.
- **Remediation:** Update the doc block: "`ForgeError::TraceUnsupportedArtifact` if the
  artifact type is unrecognized or unsupported (Profile, Mapping, ambiguous, or non-object
  root)", plus "`ForgeError::FileTooLarge` if artifact/source exceeds `MAX_FILE_SIZE`"
  and "`ForgeError::PermissionDenied`". Doc-only.

### F0754 — comment misstates FileNotFound handling
- **File:lines:** src/trace/mod.rs:41
- **Symbol(s):** `generate_trace_report`
- **Category:** documentation | **Severity:** low
- **Root cause:** Comment "(handles FileNotFound via Io mapping)" (:41) is wrong:
  `read_file` deliberately lets NotFound surface through `read_to_string` and maps it
  to the dedicated `ForgeError::FileNotFound { path }` variant (:104-105), not a generic
  Io mapping.
- **Evidence:** mod.rs:41 vs :104-105.
- **Remediation:** Replace with "(missing files surface as ForgeError::FileNotFound from
  the actual read below)" or delete the parenthetical.

### F0752 — entire source policy read into String just to count lines
- **File:lines:** src/trace/mod.rs:46-48
- **Symbol(s):** `generate_trace_report`
- **Category:** performance | **Severity:** low
- **Root cause:** `let source_content = read_file(source_path)?;` (:47) loads the whole
  policy into a String; only `lines().count()` (:48) survives. For large policies this
  needlessly allocates the full file content.
- **Evidence:** mod.rs:46-48; content unused afterwards.
- **Remediation:** Stream the count while preserving the existing error normalization:
  keep the `check_file_size` pre-check (FileTooLarge guard), then
  `let file = std::fs::File::open(source_path).map_err(|e| match e.kind() {
  ErrorKind::NotFound => ForgeError::FileNotFound { path: source_path.to_path_buf() },
  ErrorKind::PermissionDenied => ForgeError::PermissionDenied { path: source_path.to_path_buf() },
  _ => ForgeError::Io(e) })?; let source_line_count = std::io::BufReader::new(file)
  .lines().count();`. Behavior identical; the staleness check (:82) still uses
  `source_path` directly.

### F0755 — silent `Value::Null` indexing for the catalog arm
- **File:lines:** src/trace/mod.rs:53-56
- **Symbol(s):** `generate_trace_report` (Catalog arm)
- **Category:** maintainability | **Severity:** low
- **Root cause:** `let catalog = &json["catalog"];` (:54) relies on `Value`'s `Index`
  impl silently yielding `Null` when the key is missing/typed wrong. Correctness depends
  on the cross-module invariant that `detect_artifact_type` just validated key presence
  (walker.rs:34-39). A future reorder or second walk entry point turns a regression into
  a valid-looking empty report instead of an error — and the style diverges from the
  `.get(...)` chain used for `metadata.last-modified` at :76-80.
- **Evidence:** mod.rs:54; walker.rs:34-39 validation; contrast with :76-80.
- **Remediation:** `let catalog = json.get("catalog").ok_or_else(||
  ForgeError::TraceUnsupportedArtifact { detail: "missing 'catalog' key".to_string() })?;`
  — fail locally and loudly (prefer the explicit error over `unwrap_or(&Value::Null)`).

### F0756 — same silent-Null hazard for the component-definition arm
- **File:lines:** src/trace/mod.rs:57-60
- **Symbol(s):** `generate_trace_report` (ComponentDefinition arm)
- **Category:** maintainability | **Severity:** low
- **Root cause:** Identical pattern to F0755: `let compdef = &json["component-definition"];`
  (:58) silently yields Null on detector regression.
- **Evidence:** mod.rs:58.
- **Remediation:** Same as F0755 with key "component-definition".

### F0749 — no test for hand-built inconsistent `TraceSummary` saturating behavior
- **File:lines:** src/trace/report.rs:78-91 (tests section :119-231)
- **Symbol(s):** `TraceSummary::unmapped_elements`, `TraceSummary::coverage_percent`
- **Category:** test | **Severity:** low
- **Root cause:** `TraceSummary` has public fields so users can construct
  `mapped_elements > total_elements`. `unmapped_elements` uses `saturating_sub`
  (:80-81), but coverage itself can exceed 100.0 for hand-built summaries (no clamp).
  Neither degenerate contract is pinned by a test; all tests go through `from_entries`
  which cannot create the inconsistent state.
- **Evidence:** report.rs:60-91; tests :119-231 only via `from_entries`.
- **Remediation:** Add `summary_inconsistent_fields_saturate`: construct
  `TraceSummary { total_elements: 2, mapped_elements: 5 }`; assert
  `unmapped_elements() == 0` and document current behavior
  `(coverage_percent() - 250.0).abs() < 1e-9` (or decide to clamp coverage at 100.0 and
  assert that instead — a behavior decision to make explicit).

### F0747 — `ElementType` missing `Ord`/`Hash` derives
- **File:lines:** src/trace/report.rs:3-6
- **Symbol(s):** `ElementType`
- **Category:** style | **Severity:** low
- **Root cause:** The closed fieldless enum derives only Debug/Clone/Copy/PartialEq/Eq
  (:4). Missing `PartialOrd`/`Ord`/`Hash` forces hand-written key closures for
  deterministic sorting/grouping of entries. Note: the finding suggests `strum` —
  strum is NOT a dependency (verified in Cargo.toml), so skip that part.
- **Evidence:** report.rs:4; no strum in Cargo.toml.
- **Remediation:** `#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]`.
  Keep the hand-written `as_str`/`Display`. No behavior change.

### F0748 — coverage_percent doc range imprecise
- **File:lines:** src/trace/report.rs:83
- **Symbol(s):** `TraceSummary::coverage_percent`
- **Category:** documentation | **Severity:** low
- **Root cause:** Doc reads "(0.0–100.0)" without stating interval closure; with
  hand-built summaries the value can exceed 100.0 (see F0749), so the doc is also
  incomplete about that escape hatch.
- **Evidence:** report.rs:83-91.
- **Remediation:** "Coverage percentage in `[0.0, 100.0]` for summaries built via
  `from_entries`; returns `0.0` when `total_elements == 0`. Hand-built summaries
  with `mapped_elements > total_elements` may exceed 100.0." (Adjust after F0749's
  decision.)

### F0740 — strict `>` mtime comparison misclassifies same-second edits as fresh
- **File:lines:** src/trace/resolver.rs:24-25
- **Symbol(s):** `check_source_staleness`
- **Category:** bug | **Severity:** low
- **Root cause:** `source_time > oscal_time` (:25) treats equality as fresh. On
  filesystems with second-granular mtimes (FAT, network mounts) an edit within the same
  truncated second compares equal and passes as fresh even though it happened after
  artifact generation. This predicate drives the "regenerate"/staleness signal; erring
  toward stale on ties is safer.
- **Evidence:** resolver.rs:24-25; result feeds `TraceReport.source_stale` (mod.rs:82)
  and the formatter's staleness warning.
- **Remediation:** Change to `source_time >= oscal_time` with a comment: "Treat ties as
  stale: coarse mtime granularity can make a post-generation edit compare equal to the
  artifact timestamp." No existing test asserts fresh-on-equal (tests only cover
  None/unparseable fallbacks at :39-46), but add one pinning the tie→stale contract.

### F0741 — `validate_line_reference` overloads `usize` with 0-sentinel
- **File:lines:** src/trace/resolver.rs:28-34
- **Symbol(s):** `validate_line_reference`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `line_number == 0` (:33) means "group reference", overloading bare
  `usize`; an upstream bug zeroing a real line number silently passes validation instead
  of failing.
- **Evidence:** resolver.rs:32-33; callers: formatter.rs:138; extractor's 0-sentinel
  (F0724) feeds this.
- **Remediation:** Model explicitly: `pub enum LineRef { Group, Line(NonZeroUsize) }` or
  `Option<NonZeroUsize>` (None = group); change signature to
  `validate_line_ref(reference: LineRef, source_line_count: usize) -> bool` matching on
  variants. Update formatter.rs:131-139 callers and the extractor's output. If the type
  change is too broad for now, at minimum document the sentinel on the function.

### F0768 — walkers silently yield empty results on wrong-typed subfields
- **File:lines:** src/trace/walker.rs:133-147
- **Symbol(s):** `walk_compdef_elements` (symmetrically `walk_catalog_elements` :60-77)
- **Category:** bug | **Severity:** low
- **Root cause:** `compdef.get("components").and_then(|c| c.as_array())` (:136) treats
  "present but not an array" identically to "absent" — silently empty. Since callers pass
  `&json["component-definition"]` (itself a silent-index hazard, F0755/F0756), a
  malformed artifact produces an empty report with zero diagnostics.
- **Evidence:** walker.rs:136-147; catalog walker :63-76 has the same pattern.
- **Remediation:** Match on presence vs type:
  `match compdef.get("components") { Some(Value::Array(components)) => { ... } Some(_) =>
  { tracing::warn!("'components' is present but not an array; skipping"); } None => {} }`
  (apply to `capabilities`, `groups`, `controls` in both walkers). The warn path is
  the minimal fix; returning `Result<Vec<TraceEntry>, ForgeError>` is the larger option.

### F0763 — CLI naming asymmetry `--strategy component` vs `--to component-definition`
- **File:lines:** src/types.rs:47-53
- **Symbol(s):** `Strategy::Component`, `OutputType::ComponentDefinition`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `Strategy::Component` reaches the CLI as `--strategy component`
  (default kebab-case from ValueEnum, types.rs:48-53) while the semantically identical
  `OutputType::ComponentDefinition` is spelled `--to component-definition` via
  `#[value(name = "component-definition")]` (:82-83). Same concept, two spellings
  depending on the flag; the two flags interact in `forge convert`.
- **Evidence:** types.rs:48-53 vs :78-83; `Strategy::as_str` returns "component" (:61).
- **Remediation:** Either add `#[value(name = "component-definition")]` to
  `Strategy::Component` (breaking change for scripts using `--strategy component` —
  requires a changelog note; Display is independent of the clap value name, so the
  `strategy_display_component` test at types.rs:157-159 asserting `to_string() ==
  "component"` can stay), or document on `Strategy::Component` that the shorthand is
  intentional. Decide based on the release policy for CLI compatibility.

### F0777 — `section.title.clone()` to work around aliasing borrow conflict
- **File:lines:** src/uuid.rs:229-232
- **Symbol(s):** `assign_stable_ids`, `assign_stable_ids_to_section`
- **Category:** performance | **Severity:** low
- **Root cause:** `assign_stable_ids_to_section(section, &section.title.clone())` (:231)
  heap-allocates per top-level section purely because `&mut section` and `&section.title`
  can't coexist as arguments. The clone is avoidable via destructuring into disjoint
  borrows.
- **Evidence:** uuid.rs:230-232; `PolicySection` fields title/requirements/children are
  disjoint.
- **Remediation:** `for section in &mut document.sections { let PolicySection { title,
  requirements, children, .. } = section; /* stamp requirements with title, then recurse
  into children building "{title}/{child.title}" paths */ }`. The recursion currently
  threads `section_path: &str` built from titles (uuid.rs:277-283), so the destructure
  rewrite must preserve identical path strings — determinism tests in uuid.rs must pass
  unchanged (same IDs across runs).

### F0776 — UUID v5 (SHA-1) trust-model note and NUL-delimiter ambiguity
- **File:lines:** src/uuid.rs:253-260
- **Symbol(s):** `assign_stable_ids_to_section_inner`
- **Category:** security | **Severity:** low
- **Root cause:** UUID v5 is SHA-1 based (no collision resistance) and the hash seed
  `format!("{normalized}\0{section_path}\0{}\0{}", ...)` (:255-258) concatenates
  attacker-influenced text with bare NUL delimiters. Text containing '\0' makes the tuple
  encoding ambiguous (distinct (text, path) pairs can hash identically). Note:
  `normalize_for_hashing` (uuid.rs:121-131) uses `split_whitespace` which does NOT
  split on NUL, so NUL bytes survive normalization into the hash. The IDs are fine for
  dedup/change tracking but must never anchor integrity/authorization.
- **Evidence:** uuid.rs:255-260; `normalize_for_hashing` :121-131; section_path built
  from titles (:277-283).
- **Remediation:** Add the trust-model doc comment near the namespace/seed docs: "UUID v5
  = SHA-1 based with NUL-delimited seeds: treat these IDs as best-effort content tags
  (dedup/change detection), never as integrity or authorization anchors." Defensively,
  either reject/filter NUL bytes in `normalize_for_hashing` or length-prefix the fields —
  but changing the seed changes ALL generated IDs; if ID stability matters, do the
  doc-only part and document the limitation instead of silently altering hashes.

### F0778 — stable_id immediately downgraded to String
- **File:lines:** src/uuid.rs:269
- **Symbol(s):** `assign_stable_ids_to_section_inner`, `PolicyRequirement.stable_id`
- **Category:** maintainability | **Severity:** low
- **Root cause:** `requirement.stable_id = Some(uuid.to_string());` (:269) allocates a
  36-byte String per requirement and erases the typed `Uuid` distinction; consumers
  needing the raw Uuid must re-parse.
- **Evidence:** uuid.rs:269; model field `stable_id: Option<String>` (model module,
  used in modality.rs test helper :180 and serialization paths).
- **Remediation:** Widen `PolicyRequirement.stable_id` to `Option<Uuid>` (uuid crate
  already has serde support per Cargo.toml features), assign the typed value, and convert
  to string only at OSCAL serialization boundaries. This touches every reader of
  `stable_id` (preliminary_id, component definition builder, tests) — a mechanical but
  cross-cutting migration; the serialized form stays the same hyphenated UUID string, so
  no snapshot changes.

---

## DUPLICATE findings

- **F0689** (src/parse/modality.rs:43-45, documentation · low) — duplicate of **F0685**.
  Identical text, same locus, same remediation.
- **F0690** (src/parse/modality.rs:52-56, maintainability · low) — duplicate of **F0686**.
  Identical text, same locus, same remediation.
- **F0691** (src/parse/modality.rs:75-79, performance · low) — duplicate of **F0687**.
  Identical text, same locus, same remediation.

## INVALID findings

(none — all cited code exists at HEAD and each described defect was confirmed)

## PARTIAL findings

(none)

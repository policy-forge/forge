# Validation slice slice12 — 47 findings
Severity mix: low×47


══════ F0781 │ src/validate/error_types.rs:140-143 │ [maintainability · low] ══════
[maintainability · low] `schema_error_count` and `semantic_error_count` are computed by
independently filtering on exactly the two current category variants. The documented SEC-8 invariant
(`schema + semantic == errors.len()`) only holds because every error happens to be one of these two
variants; adding a future `ValidationErrorCategory` variant would silently violate the invariant
while deserialization/serialization continue to succeed with no error raised. Derive one count
arithmetically from the total so the invariant cannot drift.

+         // Derive, don't enumerate: guarantees schema + semantic == errors.len()
+         // even if new categories are introduced later (SEC-8).
          let schema_error_count =
              errors.iter().filter(|e| e.category == ValidationErrorCategory::Schema).count();
-         let semantic_error_count =
-             errors.iter().filter(|e| e.category == ValidationErrorCategory::Semantic).count();
+         let semantic_error_count = errors.len() - schema_error_count;


══════ F0783 │ src/validate/error_types.rs:75-76 │ [maintainability · low] ══════
[maintainability · low] Same issue as `schema_version_used`: a serialized `model_type` value is only
read back if absent; present non-null values would be kept, but paired with the constant-injected
`schema_version_used` fallbacks the deserialized report can mix a stored `model_type` with freshly
invented schema context. Prefer `#[serde(default)] Option<String>` consistently and substitute the
`"unknown"` default after deserialization so persisted provenance is never discarded.

-             #[serde(default = "unknown_model_type")]
+             #[serde(default = "unknown_model_type_opt")]
              model_type: String,
+             // ... or:
+             // #[serde(default)] model_type: Option<String>,
+             // let model_type = raw.model_type.unwrap_or_else(|| "unknown".to_string());


══════ F0782 │ src/validate/error_types.rs:79-80 │ [bug · low] ══════
[bug · low] The `schema_version_used` value persisted by `Serialize` is silently discarded on
deserialization: when the field is present it is still replaced via `default =
"current_schema_version"` with this build's compile-time constant. Round-tripping an old report
through serde therefore rewrites audit-relevant provenance (the report claims validation ran under
today's pinned baseline even if it originally ran under an older one). Use `Option<String>` with
`#[serde(default)]` and fall back to the constant only when the field is genuinely absent
(`schema_version_used: raw.schema_version_used.unwrap_or_else(current_schema_version)`), so stored
evidence survives intact.

-             #[serde(default = "current_schema_version")]
-             schema_version_used: String,
+             #[serde(default)]
+             schema_version_used: Option<String>,


══════ F0794 │ src/validate/formatter.rs:139-140 │ [bug · low] ══════
[bug · low] For multi-type schemas the crate message lists all accepted types (e.g. `1 is not of
type "string", "integer"`); taking only the last quoted token turns that into `wrong type: expected
integer`, falsely implying that only `integer` was legal and steering users toward the wrong fix
(e.g. converting a legitimate string to an integer). Report the complete expected-types set — again
best obtained from `ValidationErrorKind::Type { got, expected_types }` rather than parsed from
prose.


══════ F0795 │ src/validate/formatter.rs:218-223 │ [maintainability · low] ══════
[maintainability · low] When the assumed message shape does not match, the extraction helpers
fabricate plausible-sounding but invented output that is presented as fact: `expected: max length:
limit exceeded`, `message: invalid format: expected unknown`, `message: unexpected additional
properties: unknown`, `(not found)`. A user reading `max length: limit exceeded` believes the schema
really specifies that limit. Prefer an honest sentinel such as `(constraint details unavailable)` so
consumers can tell extraction failed, or (better) stop extracting from strings entirely and source
real values from the error kind/context.

          if !num.is_empty() {
              return format!("{num} characters");
          }
      }
-     "limit exceeded".to_string()
+     "(constraint details unavailable)".to_string()
  }


══════ F0796 │ src/validate/formatter.rs:76-78 │ [performance · low] ══════
[performance · low] The catch-all arm serializes the whole sub-document with `to_string()` only to
immediately throw away all but the first 100 characters: a deep payload caught by a top-level
constraint gets fully materialized into a heap buffer just to build a preview. Bound the work
instead — for containers emit a structural summary (`[42 items]`, `{3 keys}`), which also reduces
how much raw content is echoed downstream.

+                 Value::Array(items) => format!("[{} items]", items.len()),
+                 Value::Object(entries) => format!("{{{} keys}}", entries.len()),
                  other => other.to_string(),
              };
              truncate_value(&serialized, 100)


══════ F0807 │ src/validate/mod.rs:114-114 │ [documentation · low] ══════
[documentation · low] The doc is stale/incomplete: it enumerates only two of the four supported
roots (`profile` and `mapping-collection` are missing) and the `# Errors` section omits
`ValidateError::AmbiguousArtifact`, which this function now returns. Consumers choosing a schema
from this API's contract will miss supported variants.

- /// Inspects top-level keys: `"catalog"` → `Catalog`, `"component-definition"` → `ComponentDefinition`.
+ /// Inspects top-level keys: `"catalog"` → `Catalog`, `"component-definition"` → `ComponentDefinition`,
+ /// `"profile"` → `Profile`, `"mapping-collection"` → `Mapping`. Roots mapped to JSON `null` (or any
+ /// non-object value) are ignored. Exactly one recognized root must be present.
+ ///
+ /// # Errors
+ ///
+ /// Returns `ValidateError::UnknownModelType` if no recognized top-level key is found, or
+ /// `ValidateError::AmbiguousArtifact` when several recognized roots appear in the same document.


══════ F0806 │ src/validate/mod.rs:310-312 │ [performance · low] ══════
[performance · low] Unnecessary `clone()` on `version.error`: `version` is a locally owned struct
and only the unrelated fields (`declared`, `supported`) are read afterwards, so a partial move
compiles and skips duplicating the whole `ValidationError` including its `String` fields.

-     if let Some(error) = version.error.clone() {
+     if let Some(error) = version.error {
          schema_errors.push(error);
      }


══════ F0798 │ src/validate/report.rs:24-32 │ [bug · low] ══════
[bug · low] The success branch decides validity solely from `is_valid()` and ignores
`supported_input()`. A report constructible through the public API
(`ValidationReport::new_with_context(.., /* supported_input */ false, vec![])`) renders as "Valid:
<model> artifact passes all validation." even though the input was flagged unsupported — a signal
that `render_json_report` does expose (`"supported_input": false`) but the text renderer drops on
every path. Human consumers of PRD S-2 output therefore receive an unqualified pass for potentially
rejected input. Also, this early-return string ends without a trailing newline while every line
written on the failure path uses `writeln!`, giving line-oriented consumers a differently-shaped
output between the two modes.

      if report.is_valid() {
+         let support_note = if report.supported_input() {
+             String::new()
+         } else {
+             "\n  Warning: input support policy violation reported".to_string()
+         };
          return format!(
-             "Valid: {} artifact passes all validation.\n  Artifact: {}\n  Declared OSCAL version: {}\n  Schema version used: {}",
+             "Valid: {} artifact passes all validation.\n  Artifact: {}\n  Declared OSCAL version: {}\n  Schema version used: {}{support_note}\n",
              report.model_type(),
              report.artifact_path(),
              declared,
              report.schema_version_used()
          );
      }


══════ F0801 │ src/validate/report.rs:253-254 │ [test · low] ══════
[test · low] There is no test covering `supported_input == false`, neither in JSON output nor in the
text renderer. Given the test module already exercises contextual reports via `new_with_context`,
adding one case per renderer would lock in how the "unsupported input" evidence is surfaced to
consumers on both PRD S-1 and S-2 paths.

+     #[test]
+     fn text_unsupported_input_report_shows_warning() {
+         let report = ValidationReport::new_with_context(
+             "legacy.json".to_string(),
+             "catalog".to_string(),
+             Some("1.0.0".to_string()),
+             false,
+             vec![],
+         );
+         let text = render_text_report(&report);
+         assert!(!text.contains("Valid:"), "unsupported input must not be reported as valid");
+     }
+
      #[test]
      fn contextual_report_exposes_declared_and_actual_baselines() {


══════ F0799 │ src/validate/report.rs:36-44 │ [maintainability · low] ══════
[maintainability · low] The summary line is computed from
`schema_error_count()`/`semantic_error_count()` while the sections independently re-filter
`report.errors()` by category. Today the constructor enforces `counters == len(filtered)` (SEC-8),
but keeping two computations of the same truth means any future change (a third category variant, or
counts no longer derived from `errors`) would make the claimed header count disagree with the
entries actually listed below it — e.g. silently claiming errors that are never rendered. Filter
once up front and derive both the summary and the sections from those same vectors.

+     // Classify once; summary line and sections share these vectors so the
+     // claimed counts can never diverge from the rendered entries.
+     let schema_errors: Vec<_> = report
+         .errors()
+         .iter()
+         .filter(|e| e.category == ValidationErrorCategory::Schema)
+         .collect();
+     let semantic_errors: Vec<_> = report
+         .errors()
+         .iter()
+         .filter(|e| e.category == ValidationErrorCategory::Semantic)
+         .collect();
+
      // Summary line
      let mut parts = Vec::new();
-     if report.schema_error_count() > 0 {
+     if !schema_errors.is_empty() {
          parts.push(format!(
              "{} schema error{}",
-             report.schema_error_count(),
-             if report.schema_error_count() == 1 { "" } else { "s" }
+             schema_errors.len(),
+             if schema_errors.len() == 1 { "" } else { "s" }
          ));
      }


══════ F0788 │ src/validate/semantic.rs:113-114 │ [performance · low] ══════
[performance · low] Every visited node allocates a fresh formatted path String even though paths are
only consumed when an error is pushed, giving O(nodes × depth) allocation/formatting churn on large
catalogs. Carry a reusable stack of segments (keys/indices) down the recursion and render the path
string lazily only when constructing a ValidationError.

              for (key, child) in map {
-                 let child_path = format!("{current_path}.{key}");
+                 segments.push(Segment::Key(key));
+                 walk_for_orphaned_links_inner(child, segments, resource_uuids, errors, depth + 1);
+                 segments.pop(); // render "{segments}" into a String only when reporting an error


══════ F0789 │ src/validate/semantic.rs:144-144 │ [documentation · low] ══════
[documentation · low] Doc-comment/docstring mismatch: the summary promises a Catalog no-op, but the
implementation also silently no-ops for `Profile` and `Mapping`. Profiles carry
import/include-control references plausibly in scope for PRD M-4, so either implement the Profile
check or explicitly document why Profile and Mapping are excluded; a hidden behavioral gap behind a
narrower-than-stated catch arm misleads future readers.


══════ F0814 │ src/validate/version.rs:163-173 │ [test · low] ══════
[test · low] The test suite never exercises the report-fidelity path most callers rely on: there is
no assertion that `VersionInspection.declared` equals the raw declaration for an *unsupported*
version string (this is exactly where the sanitized copy is currently stored), nor that the escaped
diagnostic keeps hostile content inert (control chars) within the bounded size while the reported
`declared` stays exact. Add cases pinning: unsupported-but-valid-format ("1.3.0") keeps `declared ==
"1.3.0"` verbatim; a declaration containing a control character appears escaped in
`error.actual`/`error.message` but unchanged in `declared`; and an over-long declaration overflows
no field beyond the crate's truncation policy.

      #[test]
      fn inspection_reports_declared_and_schema_versions_separately() {
          let json = serde_json::json!({
              "catalog": {"metadata": {"version": "1.2.3", "oscal-version": "1.2.0"}}
          });
          let inspection = inspect_oscal_version(&json, OscalModelType::Catalog);
          assert_eq!(inspection.declared.as_deref(), Some("1.2.0"));
          assert!(inspection.supported);
          assert!(inspection.error.is_none());
          assert_eq!(SCHEMA_VERSION_USED, "1.2.3");
+     }
+
+     #[test]
+     fn unsupported_declaration_is_reported_verbatim_and_diagnostics_escaped() {
+         let json = serde_json::json!({
+             "catalog": {"metadata": {"oscal-version": "1.3.0"}}
+         });
+         let inspection = inspect_oscal_version(&json, OscalModelType::Catalog);
+         // Exact declaration preserved for reporting...
+         assert_eq!(inspection.declared.as_deref(), Some("1.3.0"));
+         let error = inspection.error.expect("unsupported version must error");
+         // ...while diagnostics stay escaped/bounded.
+         assert!(error.message.contains("1.3.0"));
+         assert_eq!(error.actual, "1.3.0");
+     }
+
+     #[test]
+     fn hostile_declaration_is_escaped_in_diagnostics_only() {
+         let json = serde_json::json!({
+             "catalog": {"metadata": {"oscal-version": "1.2.4\tgarbage"}}
+         });
+         let inspection = inspect_oscal_version(&json, OscalModelType::Catalog);
+         assert_eq!(inspection.declared.as_deref(), Some("1.2.4\tgarbage"));
+         let error = inspection.error.expect("unsupported version must error");
+         assert!(!error.message.contains('\t'));
+         assert!(!error.actual.contains('\t'));
      }


══════ F0813 │ src/validate/version.rs:32-37 │ [maintainability · low] ══════
[maintainability · low] Silent blanket rejection if a baseline constant stops being canonical.
`SCHEMA_VERSION_USED` aliases the independently maintained `crate::osc�al::metadata::OSCAL_VERSION`;
if either constant ever drifts to a non-canonical form (typo like "1.23" or added suffix),
`parse_version` returns `None` and this function quietly classifies *every* document as unsupported,
with error messages blaming the input instead of revealing the misconfiguration. Make policy-config
failure loud: add a compile-time/unit-test guard for the constants rather than letting them degrade
into bad-input diagnoses.

      let Some(minimum) = parse_version(MIN_SUPPORTED_OSCAL_VERSION) else {
+         debug_assert!(false, "MIN_SUPPORTED_OSCAL_VERSION must be canonical: {MIN_SUPPORTED_OSCAL_VERSION:?}");
          return false;
      };
      let Some(maximum) = parse_version(SCHEMA_VERSION_USED) else {
+         debug_assert!(false, "SCHEMA_VERSION_USED must be canonical: {SCHEMA_VERSION_USED:?}");
          return false;
      };


══════ F1048 │ supply-chain/config.toml:563-565 │ [security · low] ══════
[security · low] Asymmetric clearance across the RNG stack: rand 0.9.4 / rand_chacha 0.9.0 /
rand_core 0.9.5 / ppv-lite86 0.2.21 are only 'safe-to-run', while the parallel 0.10 generation (plus
both getrandoms) is 'safe-to-deploy'. In cargo-vet every dependent needs its own entry, so if any
deployed target actually pulls the 0.9 chain through a normal dependency, vetting will fail here (or
someone will be tempted to hand-bump just this row); conversely if these are truly reachable only
via dev/test dependencies (e.g. proptest/rand_xorshift), 'safe-to-run' is correct but brittle —
adding such a crate as a regular dep later silently inherits an unrated chain. Confirm the split is
intentional and that the whole file stays tool-generated (`cargo vet`) rather than manually relaxed.


══════ F1051 │ supply-chain/config.toml:935-937 │ [maintainability · low] ══════
[maintainability · low] Non-stable pins exempted at safe-to-deploy: wasip3
'0.4.0+wasi-0.3.0-rc-2026-01-06' is a release candidate, and the toml/toml_datetime/toml_parser pins
carry '+spec-1.1.0' build metadata. Cargo treats build metadata as part of exact matching here, so
these rows will never satisfy a plain '0.4.0'-style requirement elsewhere and will linger as dead
weight once upstream ships the stable release unless the graph is re-resolved. Document why the RC
is accepted for deployment (WASI p3 preview presumably needed by wit-bindgen/wasm-encoder work) and
make sure a tracking issue exists to swap to the stable version.


══════ F0819 │ tests/cli_integration.rs:1108-1112 │ [test · low] ══════
[test · low] Both log-control tests are coupled to generic level tokens rather than the tool's own
events: this test demands stderr contain the bare words DEBUG/INFO/TRACE, while the sibling quiet
test fails if any of those words EVER appear in stderr — including inside a legitimate ERROR
payload, a changed rendering format (ANSI/padding changes in tracing_subscriber), or an unexpected
diagnostic during a successful run. That makes them latent false-positive/false-negative sources.
Assert on stable application-owned markers instead (e.g. a known debug!/info! message payload or
span field emitted by the pipeline stages) and assert the level column separately only if the
subscriber config guarantees plain-text output.


══════ F0818 │ tests/cli_integration.rs:895-895 │ [documentation · low] ══════
[documentation · low] Traceability IDs are reused for unrelated tests within this same file,
defeating spec-task tracing (a load-bearing practice in this repo — specs/*/ tasks cite these
T-numbers): T008 labels both the catalog-stdout '[US2]' test and the component '--output' file test;
T016 labels both the XML-format edge case and the directory-as-source-profile error test; T022 and
T023 each appear under different user stories (US1/US3/US5). When a task doc says 'T016 fails', grep
lands on two unrelated tests. Give each test a unique ID or prefix with its WI/story (e.g.
`WI27-T016`) and update the corresponding spec task references.


══════ F0817 │ tests/cli_integration.rs:897-898 │ [maintainability · low] ══════
[maintainability · low] Three tests execute the identical 'convert a nonexistent .md' command (this
one, `convert_nonexistent_file_shows_not_found_error`, and `test_error_message_missing_file`), each
asserting slightly different stderr substrings — the vocabulary each pins ('not found', 'No such
file', SEC-4 leak guards) risks drifting apart when the error rendering changes, and costs redundant
process spawns. Similarly, `help_shows_convert_and_validate` duplicates
`test_help_text_lists_all_subcommands`, and `max_size_flag_is_recognized_by_clap` is a strict subset
of `test_convert_help_lists_all_options`. Consolidate each cluster into one test that asserts
everything (exit code, stderr wording, absence of internal paths/panics) so the contract has a
single source of truth.


══════ F0825 │ tests/common/fixture_generator.rs:1287-1287 │ [documentation · low] ══════
[documentation · low] This documented statistic does not match the generated corpus: the
supplementary-guidance paragraph is appended once per subsection and there are 40 subsections (4 per
domain x 10), yielding 40 cycled references rendered as plain prose ('aligned with NIST SP 800-53
AC-2'), not bracketed citations, and not ~30. Similarly '~20 compound statements' and '~25,000
words' are unverified estimates. Since consumers tune benchmarks/parsers off these numbers, fix the
counts and format description (or compute and print the stats in the suggested regression test).

- /// - ~30 citations/references ("[NIST SP 800-53 AC-2]")
+ /// - 40 plain-text standard references (one per subsection, cycling NIST_REFERENCES)


══════ F0827 │ tests/common/fixture_generator.rs:1316-1316 │ [maintainability · low] ══════
[maintainability · low] Discarding fmt::Write Results via 'let _ =' is sound today solely because
the sink is a String, whose Write impl is infallible. That invariant lives only in the author's
head: if this generator is ever retargeted at a fallible sink (a BufWriter, a truncated scratch
buffer, a compressed stream), partial-write failures would be swallowed silently and the
deterministic-output guarantee breaks invisibly. Document the invariant here (or return fmt::Result
from the generator) so future retargeting cannot lose errors.

+ // Invariant: writes target a String, whose fmt::Write impl is infallible,
+ // so discarding Results cannot drop errors. If this generator is ever
+ // changed to target a fallible writer, propagate fmt::Result instead.
          let _ = write!(doc, "## {}. {}\n\n", domain_num, domain.title);


══════ F0823 │ tests/common/fixture_generator.rs:384-384 │ [maintainability · low] ══════
[maintainability · low] All 'Section X.Y' cross-references (this 8.3, plus 1.1, 1.2, 2.4, 3.1, 7.1
sprinkled through DOMAINS) are hand-derived from the current ordering of DOMAINS and their
subsections. Adding, removing, or reordering a domain/subsection silently turns every such string
into a dangling reference — there is no compile-time or runtime linkage between the prose and the
heading numbers actually emitted ('### {domain_num}.{sub_num}'). Derive the numbers programmatically
(build a title -> "{domain_idx}.{sub_idx}" map from DOMAINS before emitting, or use placeholder
tokens substituted during generation), or at minimum add assertions/tests pinning the expected
section indices.


══════ F0826 │ tests/common/fixture_generator.rs:8-8 │ [style · low] ══════
[style · low] A module-wide #![allow(dead_code)] suppresses unused-code diagnostics for every item
here, but all structs, consts, and functions are currently referenced; it also hides genuinely
forgotten definitions if this generator grows. Remove it, or narrow the suppression to the specific
items that need it, so future fixture-extension mistakes still surface as warnings.


══════ F0831 │ tests/common/mod.rs:35-37 │ [maintainability · low] ══════
[maintainability · low] Normalization special-cases the exact serialized key "last-modified". This
silently depends on every OscalMetadata-like struct keeping that serde rename forever: a rename,
casing change, or any new date-bearing field (e.g. 'published', 'updated') will escape
normalization, resurfacing as flaky snapshot diffs that look like product bugs. Since all scrubbable
timestamps share one RFC3339 shape, prefer shape-based replacement (regex-match ISO-8601 datetime
strings wherever they appear), or centralize the magic key list in a shared constant reused next to
the serde renames.


══════ F0830 │ tests/common/mod.rs:50-52 │ [maintainability · low] ══════
[maintainability · low] The UUID pattern is fully anchored (^...$), so any dynamic value containing
a UUID as a substring is not normalized: e.g. a source_path/back-matter rlink of the shape
'/abs/path/runs/7c9e6679-7425-40de-944b-e07fc1f90ae7/out.json' becomes 'NORMALIZED_PATH' only by
luck of being path-shaped, while a bare filename 'run-<uuid>.json' passes through unchanged and
breaks byte-stable snapshots. If generator output can embed IDs inside larger strings, either use an
unanchored variant for composite fields or document that only whole-string UUIDs are scrubbed —
otherwise newly emitted shapes will silently bypass normalization.


══════ F0835 │ tests/export_format_pairs.rs:21-22 │ [test · low] ══════
[test · low] The shared helper used by all 18 tests swallows the error chain: if `export_artifact`
fails, every affected test reports a bare `unwrap()` panic with no indication of which step failed
(input parsing, serialization, or file write/read) or for which fixture/format. Panics are fine in
tests, but carry context so failures among this wide matrix localize instantly.

-     export_artifact(&input, format, Some(&output)).unwrap();
-     std::fs::read_to_string(&output).unwrap()
+     export_artifact(&input, format, Some(&output)).unwrap_or_else(|e| {
+         panic!("export_artifact failed for {relative_path:?} -> {ext:?}: {e}")
+     });
+     std::fs::read_to_string(&output)
+         .unwrap_or_else(|e| panic!("reading exported file {}: {e}", output.display()))


══════ F0834 │ tests/export_format_pairs.rs:27-32 │ [maintainability · low] ══════
[maintainability · low] Eighteen tests are copy-pastes differing only in (fixture, input format,
output format) plus two assertion flavors. Adding a new OutputFormat or model type requires writing
three more hand-duplicated tests, and drift between copies is likely (indeed some copies swap only
strings today). Replace with a table-driven layout: iterate a (model, input-format, output-format)
matrix deriving fixture paths from the inputs, and share a single root-key validator per model — or
use `rstest`/`test-case` to keep one distinct test per pair so failures stay isolated.

- #[test]
- fn format_pair_catalog_json_to_json() {
-     let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Json);
+ // Single source of truth for the matrix:
+ // const MODELS: &[(Model)] ... iterate output formats x input formats and validate.
+ fn assert_export_root(model: &str, rel_fixture: &str, fmt: OutputFormat) {
+     let c = export_and_read(rel_fixture, fmt);
+     match fmt {
+         OutputFormat::Json => {
      let v: serde_json::Value = serde_json::from_str(&c).unwrap();
-     assert!(v.get("catalog").is_some());
+             assert!(v.get(model).is_some());
+         }
+         OutputFormat::Yaml => {
+             let v: serde_yaml::Value = serde_yaml::from_str(&c).unwrap();
+             assert!(v.get(model).is_some());
+         }
+         OutputFormat::Xml => {
+             let mut r = quick_xml::Reader::from_str(&c);
+             // scan first element and require name == model
+         }
+     }
  }


══════ F0841 │ tests/export_integration.rs:101-107 │ [test · low] ══════
[test · low] This duplicates the hand-rolled Command wiring instead of reusing run_export, and it
never inspects stderr: with only `assert_ne!(exit_code, 0)`, a test that merely leaks the fixture
path via env misquoting, or the file not being parsed at all, would still "pass" as long as exit !=
0. Reuse the helper and assert that the usage error actually names the missing `--format` flag;
while here, note that this suite otherwise has no negative case for an unsupported `--format` value
(e.g. `--format docx`).

-     let output = Command::new(env!("CARGO_BIN_EXE_forge"))
-         .args(["export", CATALOG_JSON])
-         .output()
-         .expect("Failed to execute forge binary");
-
-     let exit_code = output.status.code().unwrap_or(-1);
+     let (exit_code, _stdout, stderr) = run_export(&[CATALOG_JSON]);
      assert_ne!(exit_code, 0, "Expected non-zero exit code for missing --format");
+     assert!(
+         stderr.contains("--format"),
+         "usage error should name the missing --format flag. Got: {stderr}"
+     );
+
+ #[test]
+ fn cli_export_invalid_format_value_nonzero_exit() {
+     let (exit_code, _stdout, _stderr) = run_export(&[CATALOG_JSON, "--format", "docx"]);
+     assert_ne!(exit_code, 0, "Expected non-zero exit code for unsupported --format");
+ }


══════ F0842 │ tests/export_integration.rs:16-16 │ [style · low] ══════
[style · low] `run_export(args)` takes a fixed slice type; accepting `args: &[&str]` is fine today,
but callers repeatedly build arrays just to append paths joined from TempDir. If more cases
accumulate, taking `IntoIterator<Item = AsRef<OsStr>>` (or simply `&[&OsStr]`) would avoid the extra
`to_str().unwrap()` calls in each test; low priority, purely ergonomic.

- fn run_export(args: &[&str]) -> (i32, String, String) {
+ fn run_export<I, S>(args: I) -> (i32, String, String)
+ where
+     I: IntoIterator<Item = S>,
+     S: AsRef<std::ffi::OsStr>,
+ {


══════ F0858 │ tests/golden_edge_case_tests.rs:100-110 │ [bug · low] ══════
[bug · low] Two diagnostic/context defects in the test harness make failing triage harder. (1) When
the produced out.json is present but unparsable, `serde_json::from_str(&text).expect("convert output
must be JSON")` panics with only the serde error — the captured stderr (the most likely explanation
for the corrupted/non-JSON output) and the raw text are dropped, and stdout is never captured at
all, so crash dumps printed by the binary to stdout are lost. (2) `status.code()` maps a process
killed by a signal to the sentinel `-1`, which assert_edge_case_error's `assert_ne!(run.code, 0)`
then treats as a passing non-zero exit — a signaled (crash-like, e.g. SIGSEGV/SIGKILL/OOM) convert
run satisfies the EC-1/EC-9 error-path tests as long as some substring appears in stderr. Prefer
formatting `output.status` (its Debug shows signals) in messages and asserting success()/exit-code
identity rather than the numeric != 0 sentinel.

      let output = cmd.output().expect("run forge convert command");
+     let stdout = String::from_utf8_lossy(&output.stdout).to_string();
      let stderr = String::from_utf8_lossy(&output.stderr).to_string();
-     let code = output.status.code().unwrap_or(-1);

      let output_json = if output_path.exists() {
          let text = std::fs::read_to_string(&output_path)
              .unwrap_or_else(|e| panic!("failed reading {}: {e}", output_path.display()));
-         Some(serde_json::from_str(&text).expect("convert output must be JSON"))
+         let parsed = serde_json::from_str::<Value>(&text).unwrap_or_else(|e| panic!(
+             "convert output must be JSON ({e}); status={}\n--- stderr ---\n{stderr}\n--- stdout ---\n{stdout}",
+             output.status,
+         ));
+         Some(parsed)
      } else {
          None
      };


══════ F0857 │ tests/golden_edge_case_tests.rs:233-243 │ [maintainability · low] ══════
[maintainability · low] The edge-case fixture inventory is enumerated independently in three places:
`required_dirs` here, the `dual`/`validation_only` arrays in
strategy_matrix_dual_strategy_and_agnostic_coverage, and the two BTreeSets in
strategy_constants_match_expected_scope (which additionally assert magic sizes 7/1/8). Drift between
these copies silently weakens coverage — indeed this existence-contract test already omits the
supplemental fixtures exercised by supplemental_citation_positions_and_parameter_like_content. Also
note the loop only checks that each directory exists; it never verifies the required files
(input.md, expected-catalog.json, expected-error.txt, …) are present, so a truncated fixture
checkout passes. Hoist the lists into shared constants and derive per-directory expected file names
from them.

-     let required_dirs = [
+ const CONVERT_FIXTURES: &[&str] = &[
          "ec01-no-headings",
          "ec02-compound-atomic",
          "ec03-empty-sections",
          "ec04-missing-metadata",
          "ec05-whitespace-only",
          "ec06-substantive-change",
          "ec07-malformed-citation",
-         "ec09-file-not-found",
-         "ec10-multiple-errors",
      ];
+ const SUPPLEMENTAL_FIXTURES: &[&str] =
+     &["ec-citation-unusual-positions", "ec-parameter-like-content"];
+ const VALIDATION_ONLY_FIXTURES: &[&str] = &["ec10-multiple-errors"];
+
+ // In the smoke test:
+     for dir in EDGE_ROOT_DIR_NAMES {
+         let path = fixture_dir(dir);
+         assert!(path.exists(), "missing fixture directory: {}", path.display());
+         assert!(fixture_input(dir, "input.md").exists(), "{dir}: missing input.md");
+     }


══════ F0859 │ tests/golden_edge_case_tests.rs:394-399 │ [other · low] ══════
[other · low] Despite living in an EC-7 golden-file test that reads its expectations from
tests/fixtures/edge-cases/ec07-malformed-citation/, this block fabricates a Citation out of thin air
(arbitrary id 'ec07-malformed-citation', text 'Malformed citation', source_requirement_id 'req-1')
instead of exercising the citation data the fixture/converter pipeline actually produced. It
therefore only unit-tests generate_back_matter with synthetic input and can diverge from whatever
the md parser extracts for this fixture (e.g. if the pipeline stopped emitting malformed URLs at
all, this assertion would still pass). Extract the citation (and its URL/status prop) from the
converted output, or move this assertion next to the back-matter generation unit tests with a
comment explaining it is intentionally decoupled from the fixture.


══════ F0868 │ tests/golden_file_tests.rs:62-65 │ [bug · low] ══════
[bug · low] The regex is fully anchored (`^...$`), so it only normalizes strings that are *exactly*
a UUID. Any nondeterministic value that merely *contains* a UUID — `urn:uuid:<uuid>`,
`<base>/resource/<uuid>`, or UUIDs quoted inside prose/back-matter descriptions — leaks into
snapshots unnormalized and makes comparisons environment/run-dependent, defeating the suite's
purpose (stable cross-machine regression testing). Match without anchors and rewrite via
`replace_all`, or document why bare-only matching is sufficient given current serializer output.

  static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
-     Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
+     Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
          .expect("UUID regex is valid")
- });
+ }); // then in normalize_string_value: Value::String(UUID_RE.replace_all(s, NORMALIZED_UUID).into_owned())


══════ F0847 │ tests/integration_cross_feature.rs:148-150 │ [style · low] ══════
[style · low] `unwrap_or(&vec![])` heap-allocates and drops a temporary `Vec` on every param-less
control visited just to produce a reference. Avoid the needless allocation with
`map(...).unwrap_or_default()` (or `match` on the `Option<&Vec<_>>`). Minor, but this runs
recursively across every control in every catalog parsed by these helpers.

                      let mut values: Vec<String> = param["values"]
                          .as_array()
-                         .unwrap_or(&vec![])
+                         .map(|vs| {
+                             vs.iter()
+                                 .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
+                                 .collect()
+                         })
+                         .unwrap_or_default();


══════ F0846 │ tests/integration_cross_feature.rs:193-196 │ [maintainability · low] ══════
[maintainability · low] Copy-pasted round-trip test matrices invite predictable drift.
tests/integration_cross_feature.rs repeats four round-trip tests (XML/YAML × modality-props/params)
differing only in the intermediate format string and message labels; tests/integration_round_trip.rs
duplicates the same convert → export → export → compare pipeline across its four tests, each
repeating an identical 15-line failure-message block that must stay in sync with EquivalenceDiff's
shape — and drift already exists (the XML component test applies the normalization, the YAML sibling
does not). Factor the shared skeleton in each file into local helpers — e.g.
`assert_round_trip_preserves(dir, intermediate_format)` driving the pipeline, and
`assert_equivalent(label, &original, &rt)` formatting result.differences once — so any future fix
(timeout handling, deeper comparison, fixture change) propagates to all siblings instead of being
missed in stale copies.

- // ── M-5 / AC-8: normative/advisory props survive JSON→XML→JSON round-trip ───
-
- #[test]
- fn normative_props_survive_xml_round_trip() {
+ fn assert_round_trip_preserves(dir: &TempDir, intermediate_format: &str, check: impl Fn(&Value, &Value)) {
+     let catalog_path = catalog_from_mixed_policy(dir);
+     let original = read_json(&catalog_path);
+     let mid_path = dir.path().join(format!("catalog.{intermediate_format}"));
+     run_forge(&["export", catalog_path.to_str().unwrap(), "--format", intermediate_format, "--output", mid_path.to_str().unwrap()]);
+     let rt_path = dir.path().join("catalog_rt.json");
+     run_forge(&["export", mid_path.to_str().unwrap(), "--format", "json", "--output", rt_path.to_str().unwrap()]);
+     check(&original, &read_json(&rt_path));
+ }


══════ F0851 │ tests/integration_regression.rs:57-59 │ [test · low] ══════
[test · low] Regression strength below intent: asserting only non-emptiness accepts
placeholder/corrupted identifiers such as "uuid", "TODO", or a truncated id — precisely the kind of
Phase 2 output corruption this file is meant to catch (the same gap exists for
`component-definition.uuid`). The `uuid` crate is already a project dependency; parsing the value
closes the loophole without adding dev-dependencies, and also lets the unwrap_or("") fallback become
an explicit string-type expectation.

-     // uuid must be a non-empty string
-     let uuid = catalog["catalog"]["uuid"].as_str().unwrap_or("");
-     assert!(!uuid.is_empty(), "catalog.uuid must be a non-empty string");
+     // uuid must be a well-formed UUID string
+     let uuid = catalog["catalog"]["uuid"]
+         .as_str()
+         .expect("catalog.uuid must be a string");
+     uuid::Uuid::parse_str(uuid)
+         .unwrap_or_else(|e| panic!("catalog.uuid must be a well-formed UUID, got '{uuid}': {e}"));


══════ F0850 │ tests/integration_regression.rs:62-63 │ [maintainability · low] ══════
[maintainability · low] Hardcoded literal duplicates the single source of truth:
`src/oscal/metadata.rs` defines `pub const OSCAL_VERSION: &str = "1.2.3"`, and the comment here says
the value "must use the current baseline". When the baseline bumps (e.g., 1.2.4), this test silently
becomes stale and needs a manual two-place edit (this exact failure happened historically with
legacy 1.2.0 fixtures). Since the crate publishes a library target, reference the constant so the
pin stays structural rather than duplicated.

      let oscal_version = catalog["catalog"]["metadata"]["oscal-version"].as_str().unwrap_or("");
-     assert_eq!(oscal_version, "1.2.3", "catalog.metadata.oscal-version must be '1.2.3'");
+     assert_eq!(
+         oscal_version,
+         forge::oscal::metadata::OSCAL_VERSION,
+         "catalog.metadata.oscal-version must match the project OSCAL baseline"
+     );


══════ F0849 │ tests/integration_round_trip.rs:59-62 │ [maintainability · low] ══════
[maintainability · low] Fixture/baseline paths resolve against the test process's working directory:
`run_forge(&[..., "tests/fixtures/golden/small/input.md", ...])` in tests/integration_round_trip.rs,
and `Path::new("tests/fixtures/golden")` plus `SOURCE_PROFILE = "./baselines/nist-800-53.json"`
(which lives OUTSIDE the fixtures tree, so mis-resolution surfaces only as a generic mid-pipeline
failure rather than the decent missing-input message) in tests/golden_file_tests.rs. Plain `cargo
test`/`cargo nextest` happens to chdir to the package root, but IDE test adapters, sandboxed build
systems, workspace-root invocations, or hand-launched binaries fail confusingly — which also sits
awkwardly beside golden_file_tests' own portability rationale for NORMALIZED_PATH. Anchor all
fixture and baseline resolution to `env!("CARGO_MANIFEST_DIR")` (apply uniformly: load_fixture,
run_catalog/run_component inputs, SOURCE_PROFILE, schema_validation_tests, determinism_tests, and
the convert invocations in the round-trip suite).

-     // Step 1: convert Markdown → Catalog JSON
+ let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
      run_forge(&[
          "convert",
-         "tests/fixtures/golden/small/input.md",
+     manifest_dir.join("tests/fixtures/golden/small/input.md").to_str().unwrap(),


══════ F0877 │ tests/oscal_cli_round_trip.rs:19-19 │ [test · low] ══════
[test · low] Hard-coded 30-second timeout contributes to CI flakiness: run_round_trip_chain applies
this Duration independently to each of the three external oscal-cli processes (JSON→XML, XML→YAML,
YAML→JSON), so worst-case wall time is ~90s plus JVM startup costs per stage on loaded shared
runners, while remaining generous on fast dev machines. Consider sourcing the timeout from an
environment variable so slow/frozen CI hosts can raise it without code changes.

- const TIMEOUT: Duration = Duration::from_secs(30);
+ fn round_trip_timeout() -> Duration {
+     let secs = std::env::var("FORGE_ROUND_TRIP_TIMEOUT_SECS")
+         .ok()
+         .and_then(|v| v.parse::<u64>().ok())
+         .unwrap_or(30);
+     Duration::from_secs(secs)
+ }
+ const TIMEOUT: Duration = round_trip_timeout_const_fallback();


══════ F0876 │ tests/oscal_cli_round_trip.rs:28-28 │ [maintainability · low] ══════
[maintainability · low] Opaque unwrap of a cross-module invariant: this assumes available &&
functional implies executable_path is Some. That currently holds for PathDetector/OscalCliInfo, but
if a detector implementation regresses, the failure surfaces as a bare Option unwrap panic with no
indication that test infrastructure, not the feature under test, broke. Use an expect that states
the violated detector invariant.

-     let path = info.executable_path.unwrap();
+     let path = info
+         .executable_path
+         .as_ref()
+         .expect("detector reported available+functional without executable_path")
+         .clone();


══════ F0861 │ tests/profile_golden_file_tests.rs:65-66 │ [test · low] ══════
[test · low] Coverage gap around the empty-selection (C-2) Profile: nothing locks the documented
`build_profile(vec![])` behavior of emitting a Profile with `{"imports": []}` — nor that such an
imports-less artifact remains OSCAL v1.2.3 schema-valid. In tests/profile_golden_file_tests.rs the
active snapshots cover only the two simplest success cases (non-empty selections, empty overrides);
extract the repeated build→serialize→normalize→snapshot sequence into a shared local helper and add
a third active snapshot for the empty-selection Profile so regressions in `imports` shaping are
caught. In tests/profile_validation_tests.rs a comment asserts the C-2 semantics moved to
`parse_control_ids`/execute, yet no direct test pins it. As written, a regression that fabricates an
import entry — or renders the empty artifact schema-invalid so users hit validation errors only at
runtime — passes silently in both suites.


══════ F0879 │ tests/profile_validation_tests.rs:119-125 │ [test · low] ══════
[test · low] Pairing #[ignore] with a todo!() body yields zero coverage now, and the only safety net
if someone drops the #[ignore] attribute before WI-31 lands is a panicking CI job rather than a
meaningful failure message. Prefer giving the ignored test its expected arrange/act/assert skeleton
(calling build_profile with a couple of (&id, "value") params and asserting the serialized
`modify.set-parameters` shape) marked #[ignore], so the intent is compile-checked and switching it
on verifies behavior rather than just flipping a panic.

- // TODO(WI-31): remove #[ignore] when --set-param is implemented
+ // TODO(WI-31): implement body and remove #[ignore] when --set-param ships.
  /// Profile with parameter overrides passes OSCAL v1.2.3 schema validation.
  #[test]
  #[ignore = "WI-31 (--set-param) not yet implemented"]
  fn schema_with_set_param() {
-     todo!("Enable when WI-31 (--set-param) is implemented")
+     // Skeleton kept compile-checked so enabling WI-31 only needs the
+     // #[ignore] attribute removed, not new test code written from scratch.
+     let catalog = make_catalog_file();
+     let catalog_path = catalog.path().to_string_lossy().to_string();
+     let profile = build_profile(
+         &catalog_path,
+         vec!["AC-1".into()],
+         SelectionMode::Include,
+         &[("prm-ac-1".to_string(), "60 days".to_string())],
+         None,
+     )
+     .expect("build_profile should succeed with param override");
+     let root = ProfileRoot { profile };
+     let value = serde_json::to_value(&root).expect("Serialization must succeed");
+     let result = validate_artifact(&value, OscalModelType::Profile)
+         .expect("validate_artifact should not return a framework error");
+     assert!(result.is_valid, "Param-overridden Profile must be schema-valid: {:?}", result.errors);
  }


══════ F0883 │ tests/property_tests.rs:258-258 │ [test · low] ══════
[test · low] The `+ 1` tolerance encodes a stale rationale and weakens the invariant. Reading
src/citation.rs: strip_matches replaces each matched byte range (length >= 1) with exactly one space
— it can only shrink or hold, never grow — and the subsequent normalize_prose steps (paren cleanup,
whitespace collapse, punctuation artifact removal, trim) are all non-growing. So `cleaned.len() <=
text.len()` holds provably, and the slack byte lets a future regression that inserts one stray space
around a stripped citation slip past this property undetected. Tighten to equality-of-bound without
the magic offset, keeping the reasoning in the assertion message.

-                 cleaned.len() <= text.len() + 1, // +1 for potential space from strip_matches
+                 // Every pipeline stage (strip_matches swaps N>=1 matched bytes for
+                 // one space; collapse/trim/artifact removal) is non-growing.
+                 cleaned.len() <= text.len(),


══════ F0871 │ tests/trace_integration.rs:107-111 │ [maintainability · low] ══════
[maintainability · low] These assertions hardcode positional paths based on `iter()` insertion
order, encoding both the path format and the internal grouping/flattening order of `build_catalog`.
If grouping internals ever change (e.g., nested sections becoming sub-groups), this breaks even
though the path-to-element correspondence may still be correct. Also, `paths[0..2]` will panic with
an index-out-of-bounds instead of a clear assert message if fewer links are recorded — guard with a
length assert and validate paths against the actual catalog rather than raw iteration order.

-     // Verify dot-notation path format
-     let paths: Vec<&str> = trace_links.iter().map(|l| l.oscal_json_path.as_str()).collect();
-     assert_eq!(paths[0], "catalog.groups[0].controls[0]");
-     assert_eq!(paths[1], "catalog.groups[0].controls[1]");
-     assert_eq!(paths[2], "catalog.groups[1].controls[0]");
+     assert_eq!(trace_links.len(), 3);
+
+     // Derive expected paths from the actual catalog positions instead of
+     // hardcoding indices tied to current iteration order.
+     let catalog = forge::oscal::build_catalog(&doc, Some(&mut trace_links)).unwrap();
+     for (gi, group) in catalog.groups.iter().enumerate() {
+         for (ci, control) in group.controls.iter().enumerate() {
+             let link = trace_links.by_oscal_element(&control.uuid).unwrap();
+             assert_eq!(link.oscal_json_path, format!("catalog.groups[{gi}].controls[{ci}]"));
+         }
+     }


══════ F0873 │ tests/trace_integration.rs:13-15 │ [test · low] ══════
[test · low] This helper forces `stable_id: Some(..)`, leaving the `stable_id: None` case
unexercised in this integration suite — yet `build_catalog` explicitly returns
`ForgeError::CatalogBuild` for requirements missing a stable id, an error path that interacts
directly with trace capture (partial links recorded before the failure). Add a variant accepting
`Option<&str>` and cover both the error behavior with tracing enabled and with `None`.

  fn test_requirement(text: &str, stable_id: &str, line: usize) -> PolicyRequirement {
+     test_requirement_opt(text, Some(stable_id), line)
+ }
+
+ fn test_requirement_opt(
+     text: &str,
+     stable_id: Option<&str>,
+     line: usize,
+ ) -> PolicyRequirement {
      PolicyRequirement {
-         stable_id: Some(stable_id.to_string()),
+         stable_id: stable_id.map(str::to_string),


══════ F0872 │ tests/trace_integration.rs:194-196 │ [maintainability · low] ══════
[maintainability · low] The comment claims 'WI-15 not merged', but the current source already wires
`crate::oscal::implemented_requirements::build_control_implementations` into
`build_component_definition` (when `source_profile` is `Some`). The rationale is stale; the
assertion itself holds only because this test passes `None` as the profile. Restate the invariant in
terms of inputs ('no source profile -> no control-implementations -> no trace links') so the test
does not encode obsolete feature-state assumptions that future merges will invalidate.

-     // WI-15 not merged: no implemented-requirements → empty trace collection
+     // No source_profile supplied -> no control-implementations are built,
+     // so trace capture must remain empty regardless of WI-15 status.
      assert!(trace_links.is_empty());
      assert_eq!(envelope.component_definition.components.len(), 1);

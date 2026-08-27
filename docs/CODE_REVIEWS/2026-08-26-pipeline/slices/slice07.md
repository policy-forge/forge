# Validation slice slice07 — 60 findings
Severity mix: medium×27, low×33


══════ F0833 │ tests/export_format_pairs.rs:29-31 │ [test · medium] ══════
[test · medium] Acceptance criteria for serialized output are too weak across suites to catch
degraded conversions. In tests/export_format_pairs.rs, all 18 pair tests verify only that the ROOT
key exists for JSON (`v.get("catalog").is_some()` — an export dropping all controls/groups/metadata
passes), while YAML/XML targets get mere substring searches (`contains("catalog:")`,
`contains("<catalog")`) that match anywhere (quoted descriptions, prefixes like `<catalogues>`) and
pass even when output does not deserialize (bad indentation, truncated lists). In
tests/integration_profile_e2e.rs the sole S-2 alternate-format guard is equally smoke-level: the XML
branch accepts any document containing `<profile` (a corrupted file missing
`imports`/`include-controls` passes) and YAML only requires a non-empty `profile.uuid`. Parse every
produced artifact with the appropriate reader (serde_json / serde_yaml / an XML reader, mirroring
the JSON checks) so invalid-but-matching output fails, and strengthen at least one discriminating
assertion per format: round-trip equivalence against the parsed fixture model or spot-checked
representative nested fields for catalog exports; required `include-controls` markers and survival
of the requested control ID in YAML for profiles.

      let c = export_and_read("tests/fixtures/export/catalog.json", OutputFormat::Json);
      let v: serde_json::Value = serde_json::from_str(&c).unwrap();
-     assert!(v.get("catalog").is_some());
+     let cat = v
+         .get("catalog")
+         .unwrap_or_else(|| panic!("expected root 'catalog' in exported JSON:\n{c}"));
+     // Spot-check nested content so a truncated/empty export fails.
+     assert!(cat.get("uuid").is_some(), "'catalog.uuid' lost in export\n{c}");
+     assert!(
+         cat.pointer("/metadata/title").is_some(),
+         "'catalog.metadata.title' lost in export\n{c}"
+     );


══════ F0840 │ tests/export_integration.rs:23-26 │ [test · medium] ══════
[test · medium] Subprocess runners in every integration suite rely on blocking `Command::output()`
(tests/export_integration.rs `run_export`, tests/integration_cross_feature.rs `run_forge`,
tests/integration_round_trip.rs, tests/integration_regression.rs `run_forge` plus the direct call in
`phase1_validate_still_passes`): a hung forge (deadlock, stdin wait, infinite conversion loop)
stalls the entire cargo test target with zero diagnostics until a CI job-level timeout, since cargo
has no per-test timeout by default. Spawn the child and poll `try_wait()` against a bounded
deadline, killing on expiry and failing loudly with the failing args plus collected stdout/stderr
(wait_timeout crate also works). Additionally in tests/export_integration.rs, collapsing
`status.code().unwrap_or(-1)` treats a child killed by a signal (OOM killer, segfault) as the
arbitrary -1 sentinel, indistinguishable from an intentional non-zero failure and weakening every
downstream assert_*! — carry the full ExitStatus (whose Debug shows signals) or panic on abnormal
termination instead of a numeric != 0 proxy.

+     if !output.status.code().is_some() {
+         // Killed by a signal: do not mask the crash behind a fake exit code.
+         panic!("forge terminated abnormally: {:?}", output.status);
+     }
      let exit_code = output.status.code().unwrap_or(-1);
      let stdout = String::from_utf8_lossy(&output.stdout).to_string();
      let stderr = String::from_utf8_lossy(&output.stderr).to_string();
      (exit_code, stdout, stderr)


══════ F0837 │ tests/export_integration.rs:83-86 │ [test · medium] ══════
[test · medium] Loose, OR-chained substring assertions on CLI output admit the very outcomes being
guarded (repeated pattern across suites). tests/export_integration.rs: `contains("not a valid
OSCAL") || contains("OSCAL")` is effectively tautological — any stderr merely mentioning OSCAL
satisfies it, making the intended predicate dead code and letting unrelated diagnostics pass.
tests/integration_regression.rs: `stdout.contains("Valid") || stdout.contains("valid")` passes on a
FAILING validation report because "Validation failed:" itself contains "Valid" — exit-code success
becomes the only real gate, so drifted exit semantics green-light an invalid catalog.
tests/integration_profile_e2e.rs: the same pattern is doubly unsafe (`"Invalid".contains("valid")`
is true) and defeats M-4/AC-5; and the mutual-exclusion test's `contains("cannot be used with") ||
contains("error")` passes whenever `forge profile` fails for ANY reason, never discriminating the
clap conflict from missing files or IO errors (also include stderr in diagnostics).
tests/profile_validation_tests.rs: `mutually exclusive || --include || --exclude` accepts the
unrelated InvalidArgument("Either --include or --exclude must be provided") and mis-wrapped parse
failures, weakening PRD S-4/AC-9. Replace each chain with the exact discriminating phrase/signature
— case-insensitive whole-token compare for success wordings, and the clap conflict signature (plus
conventional usage-error exit code 2) for flag conflicts — so the pinned contracts stay enforced.

      assert!(
-         stderr.contains("not a valid OSCAL") || stderr.contains("OSCAL"),
-         "stderr should contain descriptive error. Got: {stderr}"
+         stderr.contains("not a valid OSCAL"),
+         "stderr should report the document as invalid OSCAL. Got: {stderr}"
      );


══════ F0839 │ tests/export_integration.rs:9-10 │ [test · medium] ══════
[test · medium] Fixture dependencies are not verified up front, so deletions/partial syncs surface
as confusing CLI-level failures far from the cause. tests/export_integration.rs depends on three
fixture files with no existence check whatsoever; tests/integration_regression.rs inconsistently
pre-checks only `sample_profile.json` while sibling tests consume
`tests/fixtures/golden/small/input.md` and `tests/fixtures/full_policy.json` unchecked (failure
appears as a bare `forge exited 1` deep inside a conversion). Assert existence for every fixture
this suite depends on at the top (ideally once in a shared helper) so accidental removal produces an
actionable "fixture missing" message right where the dependency is introduced. Additionally for
tests/export_integration.rs, strengthen conversion postconditions beyond bare `contains` on raw
stdout: currently truncated/partially converted output passes as long as the opening marker
implicitly appears — validating round-trip parse (serde_xml_rs / serde_yaml / serde_json) or
asserting both opening and closing tags would catch degraded conversions.

- const CATALOG_JSON: &str =
-     concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/export/catalog.json");
+ fn require_fixtures() {
+     for (name, path) in [
+         ("catalog.json", CATALOG_JSON),
+         ("catalog.xml", CATALOG_XML),
+         ("catalog.yaml", CATALOG_YAML),
+     ] {
+         assert!(std::path::Path::new(path).exists(), "fixture missing: {name}");
+     }
+ }


══════ F0838 │ tests/export_integration.rs:93-96 │ [test · medium] ══════
[test · medium] Both alternatives assume POSIX errno wording. On Windows the underlying io::Error
renders as "The system cannot find the file specified.", so neither substring matches and this test
fails even though the CLI behaved correctly. Match cross-platform markers (including "cannot find",
i.e. os error 2 wordings), or better, rely on forge's own stable app-level message rather than the
OS-rendered one.

      assert!(
-         stderr.contains("not found") || stderr.contains("No such file"),
+         stderr.contains("not found")
+             || stderr.contains("No such file")
+             || stderr.contains("cannot find"), // Windows io::Error wording
          "stderr should report file not found. Got: {stderr}"
      );


══════ F0854 │ tests/golden_edge_case_tests.rs:141-149 │ [test · medium] ══════
[test · medium] Golden normalization (normalize_string_value / normalize_value) is unsound in ways
that both mask regressions and permit flakes. (1) Any string merely matching UUID SHAPE is rewritten
to NORMALIZED_UUID, so golden files can contain completely wrong or duplicated stable IDs and still
pass — real ID regressions are masked at every consumer of assert_expected_output and the insta
snapshots (which only ever see the post-normalization value). Reject obviously-degenerate inputs
(the all-zero UUID itself, or `s.parse::<uuid::Uuid>()`) and ideally pin the
deterministic-prefix/segment structure instead of collapsing to a constant. (2) Key-conditioned
rules fire exclusively on the exact parent key ("last-modified"), and array elements are recursed
with `parent_key = None`, discarding key context entirely: any nondeterministic date/time emitted
under a different OSCAL key, or appearing directly inside an array, silently escapes normalization
and produces flaky failures once content changes. Document the assumed invariants, thread the parent
key (or sibling key entry) down through arrays where the intent matters, and broaden the predicate
to keys capable of carrying nondeterministic dates/times.

+ fn is_normalizable_uuid(s: &str) -> bool {
+     UUID_RE.is_match(s)
+         && s.chars().filter(char::is_ascii_hexdigit).any(|c| c != '0') // don't mask all-zero IDs
+ }
+
+ // In normalize_value:
          Value::String(s) => {
-             if parent_key == Some("last-modified") {
+             // Only rewrite fields that are genuinely ID/time-shaped; everything else must be
+             // compared verbatim so unexpected or malformed content still fails the golden diff.
+             let is_time_field =
+                 matches!(parent_key, Some(k) if k.ends_with("-modified") || k.ends_with("-created"));
+             if is_time_field {
                  Value::String(NORMALIZED_TIMESTAMP.to_string())
-             } else if UUID_RE.is_match(s) {
+             } else if is_normalizable_uuid(s) {
                  Value::String(NORMALIZED_UUID.to_string())
              } else {
                  Value::String(s.clone())
              }
          }


══════ F0855 │ tests/golden_edge_case_tests.rs:368-372 │ [test · medium] ══════
[test · medium] This test only verifies the 'rotation' half of the stable-ID contract ('at least one
ID changed'). It never checks that requirements NOT touched by the substantive edit kept their
original UUIDs, nor that no requirement appeared/vanished — so an implementation that regenerates
every UUID whenever any change is detected would pass here. That defeats the stated purpose of ec06
('substantive_change_rotates_stable_ids') given the sibling ec05 test exists specifically to prove
wholesale regeneration does not happen for whitespace edits. Strengthen the assertions to cover ID
retention of unchanged controls and key-set equality between the two runs (assuming this fixture
only mutates prose and does not add/remove requirements). For reference, the CLI side
(src/cli/convert.rs `emit_stable_id_change_warning_if_needed`) treats the baseline as a policy
markdown doc, so passing input-original.md is correct.

      let changed_count = ids_a
          .iter()
          .filter(|(control_id, id_a)| ids_b.get(*control_id).is_some_and(|id_b| id_b != *id_a))
          .count();
      assert!(changed_count >= 1, "EC-6 should rotate at least one stable ID");
+
+     // Stability half of the contract: requirements untouched by the substantive edit must
+     // keep their UUIDs, otherwise a regenerate-everything implementation would pass.
+     let retained = ids_a
+         .iter()
+         .filter(|(control_id, id_a)| ids_b.get(*control_id).is_some_and(|id_b| id_b == *id_a))
+         .count();
+     assert!(retained >= 1, "EC-6 unchanged requirements must keep their stable IDs");
+     // The fixture only mutates prose, so the control-id universe must not change.
+     assert_eq!(ids_a.len(), ids_b.len(), "EC-6 must not add/remove requirements");


══════ F0856 │ tests/golden_edge_case_tests.rs:482-487 │ [test · medium] ══════
[test · medium] The matrix re-runs most fixtures but only asserts exit codes (plus EC-1's error
text): for EC-2/EC-3/EC-4/EC-5/EC-6/EC-7 neither side compares against the golden JSON, even though
load_expected_json/assert_expected_output helpers are available right above. The recorded values are
hardcoded literals, so the snapshot just mirrors this weak logic and cannot detect
content/serialization regressions that the dedicated per-fixture tests do catch. If a dedicated test
is later renamed/deleted, nothing forces parity with these lists either. Either delegate each case
to the same golden/stable-ID assertions used by the focused tests (a shared per-fixture helper), or
drop the duplication and drive the matrix from the dedicated tests themselves.

                  let cat = run_convert(&input, Strategy::Catalog);
                  let comp = run_convert(&input, Strategy::Component);
                  assert_eq!(cat.code, 0, "matrix {slug} catalog failed: {}", cat.stderr);
                  assert_eq!(comp.code, 0, "matrix {slug} component failed: {}", comp.stderr);
+                 // Compare actual output against goldens; exit-code-only checks hide content regressions.
+                 for (label, run) in [("catalog", &cat), ("component", &comp)] {
+                     let out = run.output_json.as_ref().expect("matrix output should exist");
+                     assert_expected_output(out, &fixture_input(slug, expected_file(slug, label)),
+                         &format!("matrix {slug} {label}"));
+                 }
                  catalog_status.insert(slug.to_string(), "success".to_string());
                  component_status.insert(slug.to_string(), "success".to_string());


══════ F0867 │ tests/golden_file_tests.rs:105-107 │ [bug · medium] ══════
[bug · medium] href normalization only rewrites *POSIX-absolute* paths (`Path::is_absolute()`). Two
leaks remain: (1) Windows-style absolute hrefs like `file:///C:/Users/me/...` or `C:\\Users\\...`
are NOT absolute on Unix targets (the platform these tests typically run under), so the
machine-specific path passes through verbatim into the snapshot and breaks reproducibility across
contributor machines/CI runners; (2) relative hrefs produced from the CWD-dependent input path (see
related comment on `run_catalog`'s `Path::new("tests/fixtures/golden")`) embed whatever directory
the test process was launched from. Either normalize any href whose path segment contains a
plausible path prefix (drive letter, file:// scheme, or leading `./../` segments that resolve above
the fixtures dir), or normalize hrefs unconditionally like source-file props are handled.


══════ F0866 │ tests/golden_file_tests.rs:155-158 │ [bug · medium] ══════
[bug · medium] The catalog extractor only reads controls exactly one level deep:
`/catalog/groups[*].controls[*].id`. OSCAL permits nested sub-groups (group.controls[].controls[])
and top-level `catalog.controls`; if build_catalog ever nests groups (or an expected golden fixture
models them), those control IDs become invisible to the accuracy gate — the report silently
under-counts and can pass at "100%" while missing an arbitrary number of extracted controls. Walk
group.controls recursively (and include `catalog.controls` if applicable), or derive ID locations
from the actual model types instead of hand-written pointer paths.


══════ F0865 │ tests/golden_file_tests.rs:189-193 │ [bug · medium] ══════
[bug · medium] Silent default arms in stringly-typed dispatch hide wiring mistakes behind degraded
lookups that fail confusingly far from the cause. tests/golden_file_tests.rs measure_accuracy:
unknown strategy strings fall through `_ => Vec::new()`, so a caller typo yields a well-formed (0%)
AccuracyReport detected only via the threshold assertion — or passable vacuously combined with the
`expected_count == 0` → 100% path — instead of signaling the programmer error; use the existing
Strategy::Catalog / Strategy::Component enums so a bad call site cannot compile. Same pattern in
tests/oscal_cli_round_trip.rs: an unexpected artifact_type maps to the JSON-pointer prefix
"/unknown/metadata/oscal-version", which misses unconditionally, so declared_oscal_version becomes
None and later trips a misleading "left/right mismatch" assert_eq! against SCHEMA_VERSION_USED far
from the actual cause; only "Catalog" and "ComponentDefinition" are valid — make anything else a
loud test-infrastructure error.

      match strategy {
-         "catalog" => extract_catalog_control_ids(json),
-         "component" => extract_component_control_ids(json),
-         _ => Vec::new(),
-     }
+     Strategy::Catalog => extract_catalog_control_ids(json),
+     Strategy::Component => extract_component_control_ids(json),
+ } // exhaustive: no silent fallback arm


══════ F0863 │ tests/golden_file_tests.rs:208-215 │ [test · medium] ══════
[test · medium] measure_accuracy silently treats an empty expected fixture as a 100% pass. If
`expected-catalog.json` / `expected-component-definition.json` is empty, malformed (no
`groups[*].controls[*]` array), or accidentally truncated by a bad golden-file regeneration, the
>=95% MS-4 gate passes vacuously. Note the asymmetry: the insta snapshot would still compare
structurally, but this independent accuracy assertion — whose whole purpose is to gate extraction
quality — reports success while measuring nothing. Consider failing (or warning loudly) when
`expected_count == 0`, since every committed golden fixture is supposed to contain requirements.


══════ F0864 │ tests/golden_file_tests.rs:223-229 │ [test · medium] ══════
[test · medium] Accuracy is measured purely as set-membership recall over expected IDs, so it never
penalizes what the pipeline wrongly emits: spurious hallucinated controls inflate nothing, and
duplicate emitted requirements (e.g., the same `control-id` emitted twice for two Markdown
requirement lines) still match via the HashSet even though the resulting OSCAL has duplicated
entries. Combined with the schema checks living only in `schema_validation_tests` (which see the
expected files, not the raw output — see related comment), the 95% gate can pass while actual
extraction precision regresses. At minimum, also penalize extras/duplicates: compute precision
against the actual ID multiset, or at least count how many *actual* IDs are unexpected and fold it
into the report/assertion.


══════ F0862 │ tests/golden_file_tests.rs:31-31 │ [documentation · medium] ══════
[documentation · medium] The module docs instruct developers to regenerate golden files via
`UPDATE_GOLDEN_FILES=1 cargo test golden`, but no code in this file implements the env-var hook —
nothing ever writes the actual pipeline output to `expected-catalog.json` /
`expected-component-definition.json`. Spec task T028 explicitly requires this support, and the
sibling test `tests/ssp_template_test.rs` already implements the pattern (`if
std::env::var("UPDATE_GOLDEN_FILES").is_ok() { ... write ... }`). Following the documented workflow
silently does nothing, which will confuse maintainers during intentional pipeline changes.

- //! - For expected JSON files: `UPDATE_GOLDEN_FILES=1 cargo test golden`
+ // In each `assert_accuracy`, before comparing:
+ if std::env::var("UPDATE_GOLDEN_FILES").is_ok() {
+     std::fs::write(&expected_path, serde_json::to_string_pretty(actual).unwrap() + "\n")
+         .unwrap_or_else(|e| panic!("Failed to update golden file {}: {e}", expected_path.display()));
+     return;
+ }


══════ F0869 │ tests/golden_file_tests.rs:643-649 │ [test · medium] ══════
[test · medium] `validate_artifact` is applied only to the hand-maintained `expected-*.json`
fixtures, never to the raw pipeline output that actually ships. A schema regression in the
serializer therefore surfaces only indirectly (as a snapshot diff to be manually accepted during
`cargo insta review`), and the committed golden snapshots themselves are never proven schema-valid —
a hand-edited or stale `expected-catalog.json` could bake in invalid structure that this gate keeps
blessing. Wire the same OSCAL Catalog/ComponentDefinition validation into `run_catalog` /
`run_component` immediately after parsing `actual` (fail the golden test itself when the live output
violates the schema), keeping these dedicated tests as the tripwire for fixture validity.

-             let result =
-                 forge::validate::validate_artifact(&json, forge::validate::OscalModelType::Catalog);
-             assert!(
-                 result.is_ok(),
-                 "{fixture_dir}/expected-catalog.json schema validation error: {:?}",
-                 result.unwrap_err()
-             );
+ // e.g. inside run_catalog()/run_component(), right after parsing `actual`:
+ let v = forge::validate::validate_artifact(&actual, forge::validate::OscalModelType::Catalog)
+     .unwrap_or_else(|e| panic!("validator error: {e}"));
+ assert!(v.is_valid, "pipeline output failed OSCAL schema: {:?}", v.errors);


══════ F0843 │ tests/integration_cross_feature.rs:167-171 │ [bug · medium] ══════
[bug · medium] `count_controls` only sums top-level `group.controls` arrays, while
`collect_modality_from_controls`/`collect_params_from_controls` recurse into nested
`control["controls"]`. The two views of "a control" disagree: with nested children the collectors
see more entries than the counter, so the `>= 5` gate guarding the atomizer split (EC-4) can pass
while the compared collections include controls the count never accounted for — or conversely mask a
miscount. Make counting recursive like the collectors, and prefer asserting the exact expected total
(== 5, as documented in the fixture header) so over-generation/duplication is also caught.

  fn count_controls(catalog: &Value) -> usize {
-     catalog["catalog"]["groups"].as_array().map_or(0, |groups| {
-         groups.iter().map(|g| g["controls"].as_array().map_or(0, Vec::len)).sum()
+     fn count_rec(controls: &[Value]) -> usize {
+         controls
+             .iter()
+             .map(|c| 1 + c["controls"].as_array().map_or(0, |sub| count_rec(sub)))
+             .sum()
+     }
+     catalog["catalog"]["groups"]
+         .as_array()
+         .map_or(0, |groups| {
+             groups
+                 .iter()
+                 .map(|g| g["controls"].as_array().map_or(0, |cs| count_rec(cs)))
+                 .sum()
      })
  }


══════ F0845 │ tests/integration_cross_feature.rs:98-99 │ [test · medium] ══════
[test · medium] Both round-trip comparisons erase per-control association: modalities are flattened
into one sorted global list, and params are keyed only by `id` with per-param value sorting. As a
result, realistic round-trip fidelity bugs — swapping prop values between two controls, attaching a
param to the wrong control, reordering controls — produce byte-identical collections and pass.
Compare keyed by control identity instead (e.g., `BTreeMap<control_id_or_text, Vec<(prop_name,
value)>>` / `BTreeMap<control_id, Vec<(param_id, values)>>`), keeping normalization only where the
format genuinely doesn't guarantee ordering within a single object.

- fn collect_modality_props(catalog: &Value) -> Vec<String> {
-     let mut result = Vec::new();
+ /// Per-control map preserves attribution across formats:
+ /// control_id -> sorted [("modality", value)]
+ fn collect_modality_props(catalog: &Value) -> std::collections::BTreeMap<String, Vec<(String, String)>> {
+     let mut result = std::collections::BTreeMap::new();


══════ F0852 │ tests/integration_profile_e2e.rs:389-393 │ [bug · medium] ══════
[bug · medium] `xml_content[..len().min(200)]` slices at byte index 200. If position 200 falls mid
multi-byte UTF-8 character (nonzero probability whenever any failing output contains non-ASCII
text), slicing panics *inside the assertion-failure path*, replacing the intended diagnostic with a
confusing byte-index panic. Truncate at the nearest char boundary instead.

+     let mut preview_end = xml_content.len().min(200);
+     while !xml_content.is_char_boundary(preview_end) {
+         preview_end -= 1;
+     }
      assert!(
          xml_content.contains("<profile"),
          "profile XML must contain <profile element, got: {}...",
-         &xml_content[..xml_content.len().min(200)]
+         &xml_content[..preview_end]
      );


══════ F0853 │ tests/integration_profile_e2e.rs:55-60 │ [test · medium] ══════
[test · medium] Two weaknesses compound here: (1) `.take(2)` is applied *before* `filter_map`, so if
either of the first two control objects lacks a string `id`, the vector silently shrinks (e.g., to
1) and every dependent test quietly loses coverage of multi-ID selection; (2) dropped entries never
produce a signal, so a degraded/renamed fixture is surfaced only far downstream as an unrelated
index/lookup failure. Fail fast with an explicit expected-count assertion and move `.take(2)` after
the filter.

      let ids: Vec<String> = controls
          .iter()
-         .take(2)
          .filter_map(|c| c["id"].as_str())
          .map(std::string::ToString::to_string)
+         .take(2)
          .collect();
+     assert_eq!(
+         ids.len(),
+         2,
+         "fixture degradation: expected 2 usable control IDs in first non-empty group"
+     );


══════ F0848 │ tests/integration_round_trip.rs:39-47 │ [test · medium] ══════
[test · medium] Silent-noop normalization: every step of the pointer/array lookup chain is fallible
and swallowed by the `if let`/`and_then` chain. If the CLI ever changes the artifact shape (root key
renamed, `components` flattened to a map, wrapper object added), neither side gets stripped,
`control-implementations` cancels out on both sides of the EC-5 comparison, and this normalization
silently stops verifying the WI-28 omission contract instead of failing loudly. Have the helper
report what it did (count of removals or discovery of the container) and assert that expectation in
the tests.

-     if let Some(comp_def) = value.pointer_mut("/component-definition")
-         && let Some(components) = comp_def.get_mut("components").and_then(Value::as_array_mut)
-     {
-         for component in components {
-             if let Some(obj) = component.as_object_mut() {
-                 obj.remove("control-implementations");
-             }
-         }
+ /// Removes `control-implementations` from every component.
+ /// Panics if the `component-definition/components` structure is absent, so an
+ /// unexpected artifact shape surfaces instead of silently skipping normalization.
+ fn clear_control_implementations(value: &mut Value) -> usize {
+     let components = value
+         .pointer_mut("/component-definition/components")
+         .and_then(Value::as_array_mut)
+         .expect("expected component-definition/components array in CLI output");
+     components
+         .iter_mut()
+         .filter_map(Value::as_object_mut)
+         .filter(|obj| obj.remove("control-implementations").is_some())
+         .count()
      }


══════ F0875 │ tests/oscal_cli_round_trip.rs:176-176 │ [test · medium] ══════
[test · medium] Vacuous green on machines without oscal-cli: when PathDetector finds nothing,
SC-001/SC-002 return early and cargo test still reports 'ok', so CI can show green while zero
round-trip conversions, divergence-log writes, or ForgeFix-count assertions actually ran (the only
signal is an unsturctured eprintln on stderr). If CI does not guarantee oscal-cli installation, this
permanently disables WI-37 coverage. Add an opt-in strict mode (e.g., FORGE_REQUIRE_OSCAL_CLI=1 in
CI jobs that install the tool) so unavailability becomes a loud failure where coverage is expected,
while local developer runs keep skipping gracefully.

-     let Some((invoker, oscal_cli_version)) = skip_if_no_oscal_cli() else { return };
+     let Some((invoker, oscal_cli_version)) = skip_if_no_oscal_cli() else {
+         if std::env::var_os("FORGE_REQUIRE_OSCAL_CLI").is_some() {
+             panic!("FORGE_REQUIRE_OSCAL_CLI is set but oscal-cli was not detected on PATH");
+         }
+         eprintln!("SKIP: oscal-cli unavailable; round-trip coverage not exercised");
+         return;
+     };


══════ F0860 │ tests/profile_golden_file_tests.rs:28-33 │ [test · medium] ══════
[test · medium] These golden tests leave the `timestamp_override` parameter as `None`, so
`last-modified` is generated live and determinism depends entirely on `normalize_for_snapshot`
recognizing the exact key name `"last-modified"` and the timestamp format. If the metadata assembly
ever renames that field or changes its serialization shape, normalization silently stops applying
and the snapshots become flaky against wall-clock time. Passing a fixed `timestamp_override` makes
the artifact genuinely deterministic at the source, keeps the golden file closest to the real
expected output, and decouples stability from regex/key-name rules in the test helper. Applies to
both `golden_include_only` and `golden_exclude_only`.

+ let fixed_ts: chrono::DateTime<chrono::Utc> =
+     "2026-01-01T00:00:00Z"
+         .parse()
+         .expect("valid fixed timestamp");
+
+ let profile = build_profile(
+     "/fixed/path/catalog.json",
          vec!["AC-1".into(), "AC-2".into(), "AC-3".into()],
          SelectionMode::Include,
          &[],
-         None,
+     Some(fixed_ts),
      )
      .expect("build_profile should succeed");


══════ F0878 │ tests/profile_validation_tests.rs:236-238 │ [test · medium] ══════
[test · medium] Hardcoded "/tmp/nonexistent-catalog-99999.json" makes this test nondeterministic:
another test, CI job, or leftover user file occupying that exact path flips .exists() to true and
turns the FileNotFound assertion into a spurious failure; on Windows the leading "/" resolves
against the current drive and behaves inconsistently across environments. Derive a guaranteed-unused
path from the OS temp dir instead.

  #[test]
  fn edge_invalid_catalog_path() {
-     let nonexistent = std::path::Path::new("/tmp/nonexistent-catalog-99999.json");
+     // Unique, platform-independent missing path rooted in the OS temp dir:
+     // avoids collision with stray files in shared /tmp and Windows quirks.
+     let tmp = tempfile::tempdir().expect("Failed to create temp dir");
+     let nonexistent = tmp.path().join("nonexistent-catalog.json");


══════ F0880 │ tests/property_tests.rs:16-16 │ [test · medium] ══════
[test · medium] Coverage gap: this strategy (and `.{0,300}` elsewhere) excludes newlines/tabs, and
proptest's `.` never matches anything outside printable ASCII, so the headline 'never panics on
arbitrary input' property actually exercises only single-line printable ASCII. Both SUTs do raw
byte-range slicing on the input (`text[last_end..range.start]` in src/parse/atomize.rs and
`text[m.start()..m.end()]`/strip_matches in src/citation.rs) and normalize_prose explicitly
advertises handling of newlines, tabs, and Unicode whitespace — exactly the input classes most
likely to surface panics (UTF-8 boundary issues, list-bullet characters) are untested here. Since
these are safety-net properties, extend at least one strategy to span the realistic domain, e.g. use
a filtered-any strategy excluding only control bytes ('\p{C}' removal keeps emoji/CJK/accented
Latin/newlines) and/or add explicit multiline/bullet/UTF-8 fixture cases.

-     fn atomize_never_panics(text in "[ -~]{0,500}") {
+     fn atomize_never_panics(
+         text in "[^\p{C}]{0,500}" // printable Unicode incl. newlines, accents, CJK
+     ) {


══════ F0882 │ tests/property_tests.rs:200-202 │ [test · medium] ══════
[test · medium] The citation properties P-9, P-11, P-12 and P-14 guard their bodies with `if let
Ok(...)`, and this arm accepts paired Err results while discarding the error values. Net effect: if
the extractor ever regresses to failing on (nearly) all inputs, this entire half of the suite passes
vacuously — zero properties are exercised and CI stays green. Add a cheap liveness check so silent
total failure is impossible, and preserve at least one error value for diagnostics when arms
disagree.

-             (Err(_), Err(_)) => {}
+         // Liveness guard: refuse to pass vacuously if extraction regresses to always-Err.
+         prop_assert!(
+             forge::citation::extract_citations_from_text("req-prop", "comply with policy")
+                 .is_ok(),
+             "extractor failed on trivial input; subsequent guarded properties would be vacuous"
+         );
+         match (&r1, &r2) {
+             (Ok((t1, c1)), Ok((t2, c2))) => {
+                 prop_assert_eq!(t1, t2);
+                 prop_assert_eq!(c1.len(), c2.len());
+                 for (a, b) in c1.iter().zip(c2.iter()) {
+                     prop_assert_eq!(&a.id, &b.id);
+                     prop_assert_eq!(&a.text, &b.text);
+                 }
+             }
+             (Err(e1), Err(e2)) => {
+                 prop_assert_eq!(format!("{e1}"), format!("{e2}"), "Error determinism violation");
+             }
              _ => prop_assert!(false, "Determinism violation: one succeeded and one failed"),
          }


══════ F0881 │ tests/property_tests.rs:45-46 │ [test · medium] ══════
[test · medium] Bare `.unwrap()` on fallible SUT calls inside proptest bodies conflates a legitimate
rejection with a harness bug: on Err the run dies as a plain panic with no SUT context instead of a
structured prop_assert failure. Affected representative sites in tests/property_tests.rs:
`atomize_requirement(&req).unwrap()` in the atomization properties, and
`forge::citation::extract_citations_from_text("req-prop", &text).unwrap()` in P-13 — inconsistent
with the sibling P-11/P-12 citation properties, which degrade gracefully via verify-or-fail.
Proptest still captures/shrinks the seed either way, but diagnosis quality suffers and a deliberate
Err-for-edge-cases contract change reads as a crash. Map failures into assertions / use the
verify-or-fail pattern at least at these sites so shrinking stays intact and intent is stated.

-         let result = atomize_requirement(&req).unwrap();
+         let result = match atomize_requirement(&req) {
+             Ok(r) => r,
+             Err(e) => {
+                 prop_assert!(false, "atomize_requirement rejected {:?}: {e}", req.text);
+                 unreachable!()
+             }
+         };
          prop_assert!(!result.requirements.is_empty());


══════ F0870 │ tests/trace_integration.rs:80-83 │ [test · medium] ══════
[test · medium] This only verifies aggregate counts, so a balanced mismatch (one control duplicated
in trace links and another missing) would still pass. Since `TraceLink.oscal_element_id` equals the
control's `uuid` (see catalog.rs), pair each link back to its control id instead of relying solely
on totals.

-     // One TraceLink per control
      let total_controls: usize = catalog.groups.iter().map(|g| g.controls.len()).sum();
      assert_eq!(trace_links.len(), total_controls);
-     assert_eq!(trace_links.len(), 3);
+
+     // Pair every link back to its control UUID, not just compare totals.
+     let mut control_uuids: Vec<&str> = catalog
+         .groups
+         .iter()
+         .flat_map(|g| g.controls.iter().map(|c| c.uuid.as_str()))
+         .collect();
+     control_uuids.sort_unstable();
+     let mut link_ids: Vec<&str> = trace_links.iter().map(|l| l.oscal_element_id.as_str()).collect();
+     link_ids.sort_unstable();
+     assert_eq!(control_uuids, link_ids);
+     assert_eq!(control_uuids.len(), 3);


══════ F0948 │ .cargo/config.toml:2-2 │ [maintainability · low] ══════
[maintainability · low] This checked-in config applies to every contributor workstation and CI
runner, but contains no explanation of why retry=10 was chosen (e.g., a flaky internal proxy?) or
whether it is meant as a universal default versus local-network tuning. Without documented
rationale, a future maintainer cannot distinguish intentional team-wide tuning from leftover local
debugging, and nobody knows when it is safe to remove. If this is machine-specific tuning, scope it
to ~/.cargo/config.toml or set CARGO_NET_RETRY locally instead; otherwise keep it and record the
reason inline.

+ # Retry up to 10 times to work around intermittent failures on the
+ # internal crates.io mirror/proxy — intended for ALL devs & CI.
+ # Remove once <ticket/link> resolves the proxy instability.
  retry = 10


══════ F1074 │ .gitattributes:28-29 │ [maintainability · low] ══════
[maintainability · low] Unconditional `* text=auto` leaves frequently-shared non-source files
(Cargo.lock, LICENSE/NOTICE, *.ps1, Makefile) subject to environment-dependent treatment: each
developer's core.autocrlf/core.eol decides their working-copy endings, causing spurious
lockfile/repo-metadata churn in PRs and unstable on-disk hashes across platforms. While
normalization itself is consistent once committed, explicitly declaring eol=lf for these high-churn
categories removes reviewer noise and keeps checksum-style auditing deterministic. This is
opportunistic hardening, not a correctness defect in the current ruleset.

  *.bat text eol=crlf
- *.cmd text eol=crlf
+ *.cmd eol=crlf text
+ # Deterministic endings for lockfiles, metadata and Windows PowerShell scripts
+ Cargo.lock text eol=lf
+ *.ps1 text eol=crlf



──────── Project Summary ────────

# Full-Repo Scan Summary (200 files, 1074 findings)

## Top Issues

1. **Valid OSCAL structures are silently dropped by serializers and walkers.** `src/export/xml_serializer.rs` never writes nested `group.groups` nor top-level `catalog.controls`; the same blind spot repeats in `src/diff/extractor.rs`, `src/oscal/catalog.rs` (`collect_control_ids_from_catalog`), `src/oscal/trace_embedding.rs`, `specs/041-assessment-plan-controls`, and `tests/golden_file_tests.rs` (one-level-deep extraction). A conformant third-party catalog loses subgroups/root controls in every downstream path — export, diff, traceability, and reporting all disagree with the metaschema.

2. **Check-then-act file handling leaves every size/input guard advisory.** `io::check_file_size` stats then a later unbounded `fs::read` re-opens the path in ~15 call sites (`src/cli/export.rs`, `src/cli/validate.rs`, `src/diff/mod.rs`, `src/diff/canonical.rs`, `src/config.rs`, `src/ingest/mod.rs`, `src/mapping/inventory.rs`, `src/validate/mod.rs`, `src/framework/disposition.rs`, …). Compounded by genuinely unbounded work inside guards: DOCX zip-bomb decompression (`src/ingest/mod.rs`), unbounded `word/document.xml`, and JSON/YAML/XML parsing limits applied only *after* full allocation (`src/json_strict.rs`, `src/lifecycle/record.rs`).

3. **Typed errors are flattened to `String` at nearly every module boundary**, severing exit-code selection and source chains: `src/batch/orchestrator.rs`, `src/cli/migrate.rs`, `src/cli/validate.rs`, `src/export/yaml.rs`, `src/json_strict.rs` (`Result<Value,String>`), `src/oscal/component_definition.rs`, `src/oscal/ssp.rs`, `src/parameter/mod.rs`, `src/config.rs`, `src/framework/mod.rs`, `src/mapping/baseline.rs`, `src/round_trip/log.rs`. Paired with **panicking `todo!()` stubs exported through the public facade** (`src/lib.rs` → parameter/citation/pipeline contracts; `specs/034`, `specs/013`, `specs/020`) and **dead error contracts** where infallible functions promise `ForgeError` variants (`src/oscal/implemented_requirements.rs`, `src/parse/clauses.rs`, `src/model/assemble.rs`).

4. **Stable-ID determinism is structurally unsound — undermining the core traceability claim.** UUIDv5 seeds embed volatile layout data (`source_line`, `atom_index`, positional `global_index`: `src/uuid.rs`, `src/oscal/implemented_requirements.rs`), ambiguous concatenation with collidable delimiters (`'|'` in `src/oscal/profile.rs`, NUL-joined free text in `specs/015`), and basename-sanitized paths that collide distinct profiles (`src/oscal/implemented_requirements.rs`, `src/oscal/assessment_plan.rs`). Cosmetic edits re-roll IDs wholesale; distinct inputs mint identical IDs.

5. **Untrusted-input security gaps in a parser-heavy pipeline**: manifest-supplied artifact/resolved_catalog paths accept absolute/traversal values (`src/mapping/manifest.rs`), `javascript:` payloads leak into citation titles (`src/oscal/back_matter.rs`), sanitizers miss C1 controls against terminal/HTML/Markdown/GitHub sinks (`src/sanitize.rs`, `src/trace/formatter.rs`, `src/framework/mod.rs`), untrusted keys flow unescaped into logs (`src/json_strict.rs`, `src/config.rs`), binary sniffing is evadable with 512-byte sampling (`specs/023/input_validation`), and the Windows env allowlist breaks Winsock (`src/oscal_cli/invoker.rs`). Meanwhile `supply-chain/config.toml` grants safe-to-deploy exemptions to exactly these parsers (lopdf, pdf-extract, quick-xml, zip, wasmparser).

6. **Quality gates produce false greens.** Benches/tests silently return success on missing fixtures (`benches/export_bench.rs`, `benches/xml_benchmark.rs`, `tests/cli_integration.rs`, `tests/common/mod.rs` opt-in guard convention); golden tests never regenerate via the documented `UPDATE_GOLDEN_FILES=1` hook and validate only hand-maintained fixtures, never raw pipeline output (`tests/golden_file_tests.rs`); property suites skip assertions via `if let Ok(...)` (`tests/property_tests.rs`); format-pair tests assert root-key presence only (`tests/export_format_pairs.rs`); accuracy metrics are recall-only with no precision penalty (`tests/golden_file_tests.rs`).

7. **CI/release plumbing suppresses its own safeguards**: `ci.yml` runs steps sequentially so a test failure skips Security audit/License check/vet entirely; `cargo install`s audit tooling from source despite an ineffective cache key (`**/Cargo.lock` any-depth); no job sets `timeout-minutes` or concurrency; `release.yml` pins the SLSA generator to a mutable tag `v2.1.0` with `contents: write`; `sha256sum` steps lack `pipefail`.

8. **Secondary/emitted artifacts ship unvalidated with fabricated data.** Assessment Plans serialize straight to disk while the primary artifact goes through full schema+semantic validation (`src/pipeline.rs`, twice); SSP skeletons emit `TODO-system-id`/`TODO-profile.json` and invented scheme URIs (`src/oscal/ssp.rs`); the XML deserializer fabricates `rel="reference"` and `part @id=""` for absent attributes (`src/export/xml_deserializer.rs`). Downstream consumers cannot distinguish placeholder from authoritative content.

9. **The `specs/` contract corpus is rotted and mostly non-compilable.** Bodyless signatures at module scope (invalid Rust) recur across `specs/007`, `014`, `020`, `022`, `027`, `028`, `043`, `044`; `specs/020` replaces real enums with `type OscalModelType = ()`; stage enumerations in `specs/013`/`specs/024` predate WI-33/WI-34 and validation; `specs/030 profile_types.rs` mandates signatures the shipped `src/oscal/profile.rs` already outgrew; several files reference unimported symbols (`ForgeError`, `ResolveArgs`).

10. **Dev-env automation is destructive or injectable**: `.specify/scripts/bash/setup-plan.sh` overwrites edited `plan.md` on re-run; `configure-worktree.sh` truncates `config.json` before `jq` runs and falls back destructively without jq; `update-agent-context.sh` builds `newline=$(printf '\n')` (always empty → deletes `\n` markers), traps mismatched temp names, leaks reviewer PII into published reports (`src/migration/types.rs` is related); `common.sh` emits `eval`-able `KEY='VALUE'` heredocs interpolating branch names verbatim.

## Module Hotspots

| Area | Why |
|---|---|
| `src/oscal/*` | Densest severity cluster: determinism (`profile.rs`, `implemented_requirements.rs`, `assessment_plan.rs`, `ssp.rs`), provenance/path leakage (`metadata.rs`, `parts.rs`), dedup/last-write bugs (`back_matter.rs`, `component_definition.rs`), minItems violations with empty arrays (`catalog.rs`, `component_definition.rs`) |
| `src/export/` | Serializer drops subtrees; deserializer invents data; YAML side lacks depth/resource caps; inverse test is self-defeating (`xml_deserializer.rs`) |
| `src/trace/`, `src/model/trace.rs` | Sentinel-riddled protocol (line 0 = group, `""` = unmapped), fail-open staleness (`resolver.rs`), unbounded walker recursion, summary/entries desync (`report.rs`) |
| `src/cli/` | Per-command inconsistency: `execute_ssp` bypasses `resolve_source_profile`; `--stable-id-baseline` ignored on SSP path; export detects format after full read; `validate.rs` duplicates `execute()` with drifted step labels |
| `src/batch/`, `src/mapping/`, `src/framework/` | Error-flattening, disk-blind output naming (`output_naming.rs` can clobber inputs when `--output-dir` unset), TOCTOU in inventory, fingerprint-skew and stale-summary bugs in analysis |
| `benches/` | Systemic validity: clones inside timed regions (`parameter_extraction.rs`), production-pipeline divergence (missing modality/parameter stages, deduped citations), payload-size-derived Criterion IDs orphaning baselines, thrice-replicated 10 MB constant |
| `specs/*/contracts/` | ~25 files; non-compiling bodies, `todo!()` public stubs, placeholder type aliases, stale stage lists |
| `.gemini/commands/`, `.specify/scripts/`, `ci/`, `scripts/` | Placeholder-mixup bugs (`$ARGUMENTS` vs `{{args}}`), unsafe quoting recipes taught to agents, numbering races, destructive defaults |

## Cross-Cutting Concerns

- **Lossy error conversion at boundaries** — representative: `src/batch/orchestrator.rs`, `src/cli/migrate.rs`, `src/mapping/inventory.rs`, `src/export/yaml.rs`, `src/json_strict.rs`.
- **Stat-then-read TOCTOU / advisory limits** — representative: `src/io.rs` + its 15+ callers listed above; also `src/migrate.rs` alias checks.
- **Empty-string / zero sentinels instead of `Option`** — representative: `src/diff/types.rs`, `src/diff/engine.rs`, `src/trace/resolver.rs`, `src/trace/report.rs`, `src/model/trace.rs`, `src/oscal/component_definition.rs`.
- **Unbounded recursion on hostile/programmatic trees** — `xml_serializer.rs::write_part`, `trace/walker.rs`, `testing/semantic_eq.rs`, `migration/inventory.rs`, `round_trip/comparator.rs`, `export/yaml.rs`.
- **Doc-comment ↔ implementation contradiction as a systemic habit** — `src/cli/convert.rs` (`Ok(None)` vs Err), `src/oscal/implemented_requirements.rs` (`REQ-%02d` claim), `src/oscal/metadata.rs` ("five mandatory fields"), `src/trace/mod.rs` (NotFound routing), `src/error.rs` exit-code matrix vs actual arms, OSCAL version claimed "1.2.0" in prose vs `OSCAL_VERSION = "1.2.3"`.
- **Duplicated single-sources-of-truth** — ingest cap ×3 (`benches/export_bench.rs`, `xml_benchmark.rs`, `tests/common/mod.rs`), ArtifactType rendering ×3 (`error.rs` Display / `diff/canonical.rs` / `cli/drift.rs`), config key allowlists mirroring serde structs (`src/config.rs`), fallback JSON schema duplicating `ValidationReport` (`src/validate/report.rs`), exit-code matrix re-coded in `main.rs`.
- **Serde round-trip asymmetry** — `Serialize` without `Deserialize` (`specs/034/parameter_types.rs`, `framework/model.rs::ImpactReport`, `round_trip/divergence.rs`), `skip_serializing_if` without `#[serde(default)]` (`specs/017`, live in `src` mirrors), integer-vs-float equality traps in all three comparators (`diff/canonical.rs`, `testing/semantic_eq.rs`, `round_trip/comparator.rs`).
- **Logging hygiene**: requirement text at WARN violating SEC-1 (`src/parse/modality.rs`), echoed `actual` values surfacing credentials (`src/validate/formatter.rs`), raw paths/usernames in artifacts (`src/io.rs` sanitize fallback, `examples/simple-access-control/output/profile.json`, `src/cli/profile.rs`).

## Quick Wins

- `sonar-project.properties`: correct the LCOV property key and add `sonar.exclusions=**/target/**` (currently produces a fake 0% gate and scans build dirs).
- Commit `Cargo.lock` (reverses `.gitignore`, which contradicts your own SEC-9 release mandate); add `rust-version` + `description`/`repository` to `[package]` in `Cargo.toml`.
- Workflows: add `timeout-minutes` + `concurrency`; split security-audit/license/vet into independent jobs (or `continue-on-error:false` with `needs:` guarded ordering); replace `cargo install` of audit tools with cached/prebuilt action pins to SHA.
- Replace the tripled 10 MB literals with `forge::io::MAX_FILE_SIZE`; extract the shared ingest→atomize→assign_ids bench setup; switch `iter(|| {...clone()...})` to `iter_batched` so clones leave the timed region.
- Hoist format/extension detection before `fs::read` in `src/cli/export.rs`; validate extension before ingesting in `src/migration/inventory.rs`; bound decompressed `word/document.xml`.
- Add `#[serde(default)]` beside every `skip_serializing_if` in prop/link fields; derive `Deserialize` on `ImpactReport`/`RoundTripResult`; widen SHA pattern and `HeadingLevel` validation.
- One-liner bug fixes: literal `{context}` in two `tracing` calls (`src/oscal_cli/invoker.rs`); remove duplicate `eprintln!`+warn in `src/cli/profile.rs`; `newline=$(printf '\n')` and trap-glob mismatch in `update-agent-context.sh`; quote `eval $(get_feature_paths)`; drop dead `OscalComparisonRules::ignored_paths` or wire it; delete `build_control_props` stub; extend escape sets to C1 range in `sanitize.rs`; add word boundaries around qualifier keywords in `src/parameter/matchers.rs`; use `mem::take(&mut title_buf)` and trim-in-place in `src/parse/mod.rs`.
- Make missing-fixture guards panic loudly in `tests/common/mod.rs`, `benches/export_bench.rs`, `tests/cli_integration.rs` so coverage cannot vanish silently.
- Align expired prose: version strings (1.2.0↔1.2.3), `REQ-{index}` docs, `implemented_requirements` truncation doc, `src/summary/mod.rs` undocumented depth-64 cutoff, `exit_codes.rs` enumeration lag — cheap edits that stop consumers coding against fiction.


══════ F1053 │ .github/dependabot.yml:10-11 │ [other · low] ══════
[other · low] Optional hardening suggestion: consider enabling explicit grouping or setting a
pull-request limit/commit-message prefix to keep the weekly cadence manageable. Also note that
without a 'reviewers', 'assignees', or commit prefix convention, weekly Cargo PRs can accumulate
quickly for projects with many transitive-facing direct deps. Not a defect — schema itself is valid
('version: 2', 'package-ecosystem', 'directory', and 'schedule.interval' are all correctly spelled
keys, so no risk of silently ignored config).


══════ F1066 │ .github/workflows/ci.yml:38-39 │ [maintainability · low] ══════
[maintainability · low] `**/Cargo.lock` matches any depth, so if this repository ever gains a
vendored/example crate with its own lockfile (common with fixture-based test data like this
project's OSCAL spec fixtures), a change there would silently bust every OS's cache. Pin the hash to
the root manifest: `hashFiles('Cargo.lock')`.

-           key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
+           key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}
            restore-keys: ${{ runner.os }}-cargo-


══════ F1064 │ .github/workflows/ci.yml:44-45 │ [maintainability · low] ══════
[maintainability · low] `cargo test` in the previous step already compiles and runs
`tests/schema_provenance_test.rs` (autotests includes all files under `tests/`; there is no
`[[test]] harness = false` opt-out in Cargo.toml). This step reruns the same binary verbatim, adding
compilation-cache-warm but still nonzero runtime and implying a verification that doesn't actually
differ from the normal suite. Remove it, or if the intent is guaranteed-single-target execution
(e.g., fast-fail on provenance regressions), state that and consider making `cargo test` itself the
gate for provenance.


══════ F1067 │ .github/workflows/ci.yml:58-60 │ [test · low] ══════
[test · low] Criterion benchmarks run as a blocking CI step with hard timings on shared hosted
runners, where CPU steal and noisy neighbors routinely produce 10–20% variance — expect intermittent
red builds from genuine-timing failures unrelated to code quality. Consider removing
`--warm-up-time`/`--measurement-time` overrides to accept Criterion defaults, adding tolerance
thresholds, or moving benchmarks to a scheduled/nightly workflow so latency-noise doesn't gate
merges.


══════ F1061 │ .github/workflows/release.yml:113-117 │ [maintainability · low] ══════
[maintainability · low] Redundant dependency: this job waits on `build`, even though it never
touches build artifacts and its inputs are only the checked-out source. Dropping `needs: [build]`
lets SBOM generation run in parallel with the 3-OS build matrix, shortening total release time.

    sbom:
      name: Generate SBOM
+     needs: []
      runs-on: ubuntu-latest
      permissions:
        contents: read


══════ F1060 │ .github/workflows/release.yml:131-132 │ [bug · low] ══════
[bug · low] This step assumes cargo-cyclonedx emits exactly `forge.cdx.json` at the repo root. The
default output is `<package-name>.cdx.json` per crate; if the project ever becomes a multi-crate
workspace (or the package is renamed), `mv` fails hard — or worse, in some versions/cargo-cyclonedx
configs multiple .cdx.json files land in different directories and only one gets moved. Prefer a
glob-move (`mv *.cdx.json "forge-${TAG_NAME}-sbom.cdx.json"`) or an explicit `-o/--output-path` so
the artifact name is pinned by configuration, not implicit defaults.

            cargo cyclonedx --format json
-           mv forge.cdx.json "forge-${TAG_NAME}-sbom.cdx.json"
+           mv ./*.cdx.json "forge-${TAG_NAME}-sbom.cdx.json"


══════ F1069 │ .gitignore:2-4 │ [maintainability · low] ══════
[maintainability · low] Unanchored directory patterns `debug/`, `release/` (and likewise `coverage/`
further down) match a directory of that name at ANY depth in the tree, not just at the repository
root. Today this is latent (no such directories exist yet), but a future legitimate path like
src/fixtures/debug/ or tests/fixtures/coverage/ would be silently excluded from version control and
hard to diagnose. Anchor them to the root (leading slash) to limit scope; `target/` artifacts
already live under the rooted /target/ entry.

  /target/
- debug/
- release/
+ /debug/
+ /release/


══════ F1070 │ .gitignore:27-29 │ [maintainability · low] ══════
[maintainability · low] `*.prof*` matches any path whose name contains ".prof" anywhere in the tree
— far beyond profiling artifacts. Intended non-tool files such as fixtures/data named e.g.
`model.profile.json` or `run.profiles.csv` would be silently un-commitable, mirroring the classic
problem with overly broad .gitignore globs. Narrow it to concrete profiler outputs commonly produced
by cargo-flamegraph / LLVM tools.

  *~
  .DS_Store
  *.tmp


══════ F1071 │ .gitignore:54-56 │ [maintainability · low] ══════
[maintainability · low] Same-name-but-different-comment details aside, this block re-lists
.DS_Store, which is already ignored in the 'IDEs & Editors' section above. Exact duplicates are
harmless to git but confuse auditors grepping the file (they may assume subtle intent differences)
and can drift over time. Consolidate into one entry under the OS section (or drop this block
entirely) to keep the file unambiguous.

  # === OS ===
  Thumbs.db
- .DS_Store


══════ F0996 │ .rustfmt.toml:3-3 │ [style · low] ══════
[style · low] "use_small_heuristics = \"Max\"" sets ALL width heuristics (struct literals,
function-call args, imports, match arms, chain elements, ...) to max_width = 100. This allows very
dense single-line constructs (e.g., deeply nested function calls or large struct initializers
collapsed onto one line), which reduces diff granularity and can hurt readability on code-heavy
files. If this density is not a deliberate team-wide decision, consider "Default"/"Small", or
document why Max was chosen here.

+ # Deliberate trade-off: permit one-line constructs up to max_width to reduce vertical noise.
+ # Re-evaluate if reviewers flag overly dense diffs.
  use_small_heuristics = "Max"


══════ F1023 │ Cargo.toml:12-12 │ [maintainability · low] ══════
[maintainability · low] Version requirements mix very loose carets (`clap = "4"`, `regex = "1"`)
with arbitrarily precise floors (`serde_json = "1.0.149"`, `uuid = "1.20.0"`, `tempfile =
"3.25.0"`). Because caret requirements are not pins, the precise forms neither improve
reproducibility nor restrict upgrades (Cargo.lock does that); they just pointlessly raise the
minimum compatible version for anyone building from source and force churn whenever a transitive
constraint conflicts with those exact patch levels. Pick one granularity policy across the manifest
(typical: major-only carets like `serde_json = "1"`), and rely on the lockfile for reproducible
builds.

- serde_json = "1.0.149"
+ serde_json = "1"


══════ F1021 │ Cargo.toml:22-22 │ [maintainability · low] ══════
[maintainability · low] `jsonschema` has `default-features = false`, which drops the resolver
features (`resolve-http`, `resolve-file`). That is safe today because every call site uses local
catalog schemas via `validator_for()`, but it means any schema that ever grows an absolute `$ref` to
an HTTP/file URI will compile yet fail to resolve with a confusing runtime error instead of being
caught at build/config load time. Add a short comment marking this as deliberate so a future schema
edit does not quietly reintroduce broken references.

+ # Deliberately no resolvers: all schemas are embedded/local and validated via validator_for();
+ # re-enable `resolve-http`/`resolve-file` only if remote $ref support is actually needed.
  jsonschema = { version = "0.45.0", default-features = false }


══════ F1022 │ Cargo.toml:4-4 │ [documentation · low] ══════
[documentation · low] `edition = "2024"` implies a hard toolchain floor (Rust 1.85+), but no
`rust-version` is declared. Without it, CI cannot enforce an MSRV gate, `cargo publish` will accept
builds from contributors on older stable toolchains until they hit edition-specific syntax errors,
and downstream consumers get no machine-readable compatibility signal. Declare the explicit MSRV (or
wire the edition floor into repo policy/toolchain config).

  edition = "2024"
+ rust-version = "1.85"


══════ F1025 │ Cargo.toml:5-5 │ [documentation · low] ══════
[documentation · low] The `[package]` section lacks `description` and `repository` (and optionally
`readme`/`keywords`). Beyond hurting discoverability, crates.io rejects publishes without a
description and warns on a missing repository, so this manifest is not currently publish-clean. Fill
in the core metadata before any release.

  license = "MIT"
+ description = "Document forge: ingestion, validation, and export pipeline for structured documents"
+ repository = "https://github.com/<org>/forge"
+ readme = "README.md"


══════ F0043 │ benches/atomize.rs:27-29 │ [maintainability · low] ══════
[maintainability · low] unwrap() runs inside the timed closure (also in
atomize_requirement/atomic_passthrough and atomize_document/100_mixed_requirements). A regression
that makes atomization return Err aborts the entire criterion run with a bare panic message carrying
no bench name, leaving a generic 'called Result::unwrap() on an Err' to debug. Label the failure
with the bench identifier, or validate the input once before the timing loop and keep an unwrapped
fast path inside it.

      c.bench_function("atomize_requirement/compound_2part", |b| {
-         b.iter(|| atomize_requirement(black_box(&req)).unwrap());
+         b.iter(|| {
+             atomize_requirement(black_box(&req))
+                 .expect("atomize_requirement/compound_2part: fixture must atomize")
+         });
      });


══════ F0034 │ benches/export_bench.rs:22-23 │ [maintainability · low] ══════
[maintainability · low] This locally re-declares the 10 MB ingest limit that also exists as the
crate's `forge::io::MAX_FILE_SIZE` (used inside `export_artifact`). If the production cap changes,
this copy drifts and the benchmark exercises a limit that no longer matches production behavior.
Reference the public constant instead.

- /// Maximum file size for ingest (10 MB).
- const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
+ /// Maximum file size for ingest.
+ use forge::io::MAX_FILE_SIZE as MAX_SIZE_BYTES;


══════ F0033 │ benches/export_bench.rs:29-33 │ [maintainability · low] ══════
[maintainability · low] ~15 bare `unwrap()`s span distinct bootstrap stages (ingest → parse →
atomize → citation → catalog build → serialization). When the synthetic fixture ever breaks, the
panic surfaces only the innermost error with no indication of which stage failed, making diagnosis
painful. Cheap fix in test/bench code: use `.expect("<stage>")` labels or wrap phases with context.

  fn build_catalog_json(fixture_path: &Path) -> String {
-     let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
+     let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES)
+         .expect("bench setup: ingest_file failed");
      let content = ingested.reconstruct_content();
-     let sections = forge::parse::extract_sections(&content).unwrap();
-     let clauses = forge::parse::extract_clauses(&content).unwrap();
+     let sections = forge::parse::extract_sections(&content)
+         .expect("bench setup: extract_sections failed");
+     let clauses = forge::parse::extract_clauses(&content)
+         .expect("bench setup: extract_clauses failed");


══════ F0035 │ benches/export_bench.rs:77-78 │ [other · low] ══════
[other · low] `doc-header says fixture yields "a 500KB+ artifact"`, but the benchmark name uses the
computed size; if the synthetic fixture regresses below the documented threshold the benchmark still
passes happily. Consider asserting the minimum expected size (e.g., `debug_assert!(json_size_kb >=
500)`) so size regression is caught.


══════ F0027 │ benches/parameter_extraction.rs:3-7 │ [documentation · low] ══════
[documentation · low] These claims overstate what the benchmark demonstrates. (1) The sample texts
are roughly 50-65 characters, not "~100". (2) The corpus is 10 fixed parameterized + 7 fixed plain
strings with fixed one-third parameterization density, `nesting_depth: 0`, empty citations, and
`modality: None` — so any conclusion about linear scaling (SEC-3) or the p95 ≤ 1s target holds only
for this narrow synthetic shape, not for longer, denser, nested, or citation-bearing real-world
requirements. Soften the wording to scope the claims to this synthetic corpus, or extend the corpus
(varying lengths/density/nesting) before citing it as empirical evidence.

  //! Measures `extract_parameters()` on a synthetic `PolicyDocument` with 500
- //! requirements (mix of parameterized and non-parameterized), each ~100 characters.
+ //! requirements drawn from 10 fixed parameterized and 7 fixed plain sample texts
+ //! (~50-65 chars each, one-third parameterized, flat nesting, no citations).
+ //!
+ //! Results are indicative only for this synthetic shape; see bench TODO for
+ //! varying-length/nested/citation-bearing corpora before generalizing.
  //!
  //! PRD performance target (NF-1): p95 completion ≤1 second for the full corpus.
- //! SEC-3 complementary: demonstrates `regex` crate's linear-time guarantee empirically.


══════ F0028 │ benches/parameter_extraction.rs:89-94 │ [maintainability · low] ══════
[maintainability · low] All three benchmark bodies are byte-for-byte identical except for the size
constant and bench name. Extracting a helper keeps future fixes (e.g., switching to `iter_batched`)
from needing to be applied in three places.

- /// T040: Benchmark `extract_parameters()` on 500 requirements.
- ///
- /// PRD NF-1 target: p95 ≤1 second. This bench verifies the implementation
- /// is fast enough on a representative corpus.
+ fn bench_extract(c: &mut Criterion, name: &str, n: usize) {
+     let doc = make_synthetic_document(n);
+
+     c.bench_function(name, |b| {
+         b.iter_batched(
+             || doc.clone(),
+             |mut d| {
+                 black_box(extract_parameters(&mut d))
+                     .expect("extract_parameters must not fail");
+             },
+             BatchSize::SmallInput,
+         );
+     });
+ }
+
  fn bench_extract_parameters_500(c: &mut Criterion) {
-     let doc = make_synthetic_document(500);
+     bench_extract(c, "extract_parameters/500_requirements", 500);
+ }


══════ F0055 │ benches/pipeline_benchmark.rs:144-144 │ [documentation · low] ══════
[documentation · low] Cross-benchmark comparability caveat worth documenting: per-stage timings here
run on fully warm, shared pre-computed inputs (same &ingested/&sections instance reused across
iterations; doc_for_catalog constructed once and then re-measured by both catalog_assembly and
serialization, where serializer caches in serde internals may differ from cold runs). Consequently
per-stage values will not sum to full_pipeline/catalog_50page cold-run time, and may differ
per-stage from their cost within the full pipeline. Add a note to the module docs warning against
treating stage numbers as additive components of the end-to-end figure.


══════ F0054 │ benches/pipeline_benchmark.rs:160-161 │ [test · low] ══════
[test · low] All per-stage bodies and especially this out-of-loop pre-computation chain use bare
.unwrap(). If ingest semantics or max-size handling change, the failure surfaces as a bare 'called
Option::unwrap()/Result::unwrap()' with no indication of which stage broke — unlike the descriptive
expect("Pipeline must not fail...") used in bench_full_pipeline. Give each prep unwrap an expect
naming the stage, e.g. expect("stage prep: ingest_file failed"), so benchmark failures
self-diagnose.


══════ F0022 │ benches/uuid_benchmark.rs:12-12 │ [maintainability · low] ══════
[maintainability · low] The benchmark samples are inline literals, and the same domain sentence
appears twice in different forms (padded here, clean in bench_generate_stable_id). This invites
copy-paste drift: editing one copy silently changes what that individual benchmark measures while
the rest keep the old shape. Hoist the sample strings to module-level constants shared by all three
benchmarks.

-     let text = "All users must use multi-factor authentication";
+ const CLEAN_SHORT_SAMPLE: &str = "All users must use multi-factor authentication";
+
+ fn bench_generate_stable_id(c: &mut Criterion) {
+     let text = CLEAN_SHORT_SAMPLE;


══════ F0023 │ benches/uuid_benchmark.rs:19-19 │ [test · low] ══════
[test · low] All three inputs are plain ASCII prose, so the suite never exercises the shapes most
likely to behave differently in these helpers: empty/single-char strings, non-ASCII text, Unicode
whitespace variants, or trailing-only separators. Edge-case cliffs in normalization/hashing would go
unmeasured before a production regression ships. Consider adding dedicated bench cases for those
inputs, and configure Throughput::Bytes(len) (per sample) so short- and long-input runs can be
compared in throughput terms rather than raw nanoseconds that mostly reflect fixed per-call
overhead.

-     let text = "Organizations shall implement comprehensive security controls including but not limited to multi-factor authentication, role-based access control, encryption at rest and in transit, continuous monitoring, incident response procedures, and regular security assessments to ensure compliance with applicable regulatory requirements";
+ use criterion::{Throughput, Criterion};
+
+ fn bench_normalize_edge_cases(c: &mut Criterion) {
+     for name in ["empty", "ascii", "unicode_ws"] {}
+     let mut group = c.benchmark_group("normalize_edge_cases");
+     for (name, sample) in EDGE_CASE_SAMPLES {
+         group.throughput(Throughput::Bytes(sample.len() as u64));
+         group.bench_function(name, |b| {
+             b.iter(|| normalize_for_hashing(std::hint::black_box(sample)))
+         });
+     }
+     group.finish();
+ }


══════ F0104 │ benches/xml_benchmark.rs:22-25 │ [maintainability · low] ══════
[maintainability · low] Third copy of the 10 MB ingest cap: this constant also exists in
tests/common/mod.rs (used by xml_catalog_test/xml_component_test/xml_validation_test) and in other
benches, while the authoritative value lives only as the CLI's --max-size default. If the production
cap ever changes, some copies get updated and some don't, and the benchmark ends up exercising a
different ingest boundary than real callers. Export a single public constant (e.g.
forge::DEFAULT_MAX_SIZE_BYTES) from the library and reference it here.

- /// Maximum file size for ingest (10 MB).
- ///
- /// Duplicated here because benchmarks cannot use the `tests/common` module.
- const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
+ // In the library (single source of truth):
+ // pub const DEFAULT_MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
+ const MAX_SIZE_BYTES: u64 = forge::DEFAULT_MAX_SIZE_BYTES;


══════ F0100 │ benches/xml_benchmark.rs:43-44 │ [bug · low] ══════
[bug · low] The back-matter inputs differ from production: run_catalog_pipeline collects citations
via oscal::component_definition::collect_all_citations(&doc.sections), which dedupes by citation id
(HashSet) and caps at MAX_CITATIONS (10k) before calling generate_back_matter.
PolicyDocument::collect_citations() does neither — it clones every citation from every requirement.
On a 50-page fixture with repeated references this benchmark serializes a back-matter section larger
than (and structurally unlike) any real export — duplicate resources can even repeat the same UUID —
skewing measurements in the opposite direction from a genuine artifact. Use the same deduplicating
collector as the pipeline.

-     let citations = doc.collect_citations();
+     // Same collection semantics as the pipeline (dedupe by id, bounded count).
+     let citations = forge::oscal::component_definition::collect_all_citations(&doc.sections);
      let (back_matter_resources, _) = forge::oscal::generate_back_matter(&citations).unwrap();


══════ F0102 │ benches/xml_benchmark.rs:59-60 │ [maintainability · low] ══════
[maintainability · low] This hand-assembled OscalCatalog currently mirrors run_catalog_pipeline's
Step 11 assembly exactly (build_catalog itself always emits controls: vec![], and src/pipeline.rs
also hardcodes controls: vec![]), so nothing is lost today. But it is a second, manually
kept-in-sync copy of that assembly logic: if build_catalog or the pipeline starts populating
root-level controls or changes metadata mapping, this benchmark keeps silently serializing a stale
shape instead of failing or following along. Prefer using the returned `catalog` directly (metadata
it carries is placeholder anyway and assembling metadata separately adds cost/risk), or add a
debug_assert sanity check that the reused data is non-empty so drift degrades loudly rather than
quietly shrinking the measured workload.

          controls: vec![],
          groups: catalog.groups,
+         // Guard against silent drift if upstream starts populating data this
+         // reconstruction drops:
+         // debug_assert!(!catalog.groups.is_empty(), "benchmark catalog has no groups");


══════ F1036 │ deny.toml:16-18 │ [maintainability · low] ══════
[maintainability · low] This allow-list covers the licenses most core ecosystems crates ship under,
but several widely occurring permissive licenses are absent (ISC, BSD-2-Clause, MPL-2.0, Apache-2.0
WITH LLVM-exception, CC0-1.0/Unlicense, CDLA-Permissive-2.0). Any future transitive dependency under
one of them will hard-fail the license check with a name/error diagnostic that must be adjudicated
by hand. Before assuming the list is complete, run `cargo deny check licenses` against the current
lockfile and add new deps through a review step; alternatively document the intended procedure
(raise allowlist via PR, citing the crate) so newcomers aren't tempted to relax other gates to make
CI green.


══════ F1033 │ deny.toml:19-19 │ [documentation · low] ══════
[documentation · low] confidence-threshold = 0.8 merely restates cargo-deny's built-in default
(0.8). As written this line adds no behavior — it only looks like a deliberate tuning knob when it
isn't. Either remove it to reduce config surface, or keep it deliberately with a comment saying why
0.8 is sufficient / why it isn't raised toward 0.9+, so a future reviewer knows intentality.


══════ F0019 │ examples/component-based/generate_ssp.py:138-139 │ [bug · low] ══════
[bug · low] Silent success on empty extraction: if output/component-definition.json contains no
implemented requirements (or the script is run against a stale/empty output), the SSP is still
written with an empty control-implementation and the run exits 0 with a success banner. Downstream
automation gets a vacuous document indistinguishable from a good one. Fail fast when no controls
were extracted.

+ if not control_ids:
+     raise SystemExit("No implemented requirements found in output/component-definition.json; refusing to write empty SSP")
+
  with open("output/ssp.json", "w") as f:
      json.dump(ssp, f, indent=2)


══════ F0009 │ examples/component-based/output/catalog-new.json:42-44 │ [maintainability · low] ══════
[maintainability · low] Generator design smell: each control's '_gdn' part is not control-specific
guidance — it verbatim-copies the bulleted requirement list of the whole source subsection (here all
three SI-1 statements are repeated inside POL-CSP-020, -021 and -022 alike; the same pattern holds
for AC-1/AC-2/DP-1/DP-2/AL-1/AL-2/SI-2). This bloats the catalog ~3x and makes every guidance text
ambiguous about which sibling control it applies to. If 'single source paragraph per control
guidance' is intended, it should at least be marked up as context rather than flattened guidance;
otherwise emit per-control guidance only.

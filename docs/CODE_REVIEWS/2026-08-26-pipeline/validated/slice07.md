# Validated findings — slice07 (60 findings)

Review source: docs/CODE_REVIEWS/ocr_review_2026-08-16.md (review date 2026-08-16).
Validated against HEAD b22e2d5 ("Harden successor map opening against symlink races") on 2026-08-26.

**Counts:** valid 56 · partial 1 · invalid 3 · duplicate 0

All findings in this slice target test suites, benchmarks, repo config, CI workflows, and example scripts. No finding was already fixed at HEAD; no intra-slice duplicate root causes were found. Identifiers below are given verbatim (no code decoration).

---

## VALID findings

### F0833 — Weak acceptance assertions in format-pair and profile e2e suites
- **File:lines:** tests/export_format_pairs.rs:28-141 (all 18 pair tests); tests/integration_profile_e2e.rs:367-413 (S-2 test profile_xml_yaml_formats)
- **Symbols:** format_pair_* tests; export_and_read; profile_xml_yaml_formats
- **Category:** test · **Severity:** medium
- **Root cause:** JSON targets are gated only on root-key presence (v.get("catalog").is_some(), e.g. line 31), so an export dropping metadata/groups/controls passes. XML/YAML targets use substring probes (contains("<catalog"), contains("catalog:")) that match anywhere and pass even when output is malformed or truncated. The S-2 profile guard accepts any XML containing "<profile" (lines 391-394) and a YAML with merely non-empty profile.uuid (lines 411-412), never checking imports/include-controls.
- **Evidence:** Read all 18 pair tests; lines 29-31 parse JSON but assert only the root key; lines 37-38 and 42-43 are bare contains checks.
- **Remediation:** In every JSON pair test, spot-check nested structure (catalog.metadata.title, non-empty catalog.groups array, at least one control id). For XML targets parse with quick-xml or at minimum assert both opening and closing catalog tags plus one nested element; for YAML parse with serde_yaml then assert the same nested pointers as JSON. In profile_xml_yaml_formats, parse the XML (assert imports + include-controls + the requested id) and for YAML assert profile.imports[0].include-controls[0].with-ids contains ids[0]. No snapshot impact.

### F0840 — Blocking subprocess runners with no timeout; signal-kills masked as -1
- **File:lines:** tests/export_integration.rs:16-27; tests/integration_cross_feature.rs:55-56; tests/integration_round_trip.rs:17-18; tests/integration_regression.rs:19-20 and 135-138
- **Symbols:** run_export (export_integration.rs), run_forge (three suites), direct Command call in phase1_validate_still_passes
- **Category:** test · **Severity:** medium
- **Root cause:** All runners use blocking Command::output(); a hung forge stalls the whole test target with zero diagnostics until the CI job timeout (cargo has no per-test timeout). Additionally run_export collapses output.status.code().unwrap_or(-1) (line 23), so a signal-killed child (OOM/segfault) is indistinguishable from an intentional non-zero exit, weakening every downstream assert_ne!(exit_code, 0).
- **Evidence:** Verified all four runners call .output() with no timeout; unwrap_or(-1) at export_integration.rs:23.
- **Remediation:** Replace output() with spawn() plus a bounded try_wait() poll loop (e.g. 120 s deadline), killing the child on expiry and panicking with the failing args plus collected stdout/stderr (or add the wait-timeout crate as a dev-dependency). In run_export, assert output.status.code().is_some() before extracting the code and panic with the Debug of ExitStatus otherwise so signal terminations surface. No snapshot impact.

### F0837 — Tautological OR-chained substring assertions on CLI output
- **File:lines:** tests/export_integration.rs:83-86; tests/integration_regression.rs:142-145; tests/integration_profile_e2e.rs:219-222 and 359-362; tests/profile_validation_tests.rs:227-231
- **Symbols:** cli_export_invalid_input_nonzero_exit; phase1_validate_still_passes; profile validate/conflict tests; edge_both_flags_returns_error
- **Category:** test · **Severity:** medium
- **Root cause:** contains("not a valid OSCAL") || contains("OSCAL") is satisfied by any stderr merely mentioning OSCAL. stdout.contains("Valid") || stdout.contains("valid") also matches "Validation failed:" output — and since "Invalid".contains("valid") is true, the profile assertions pass on failure text. The clap-conflict check contains("cannot be used with") || contains("error") passes for any error. The library-level mutual-exclusion check accepts any InvalidArgument mentioning either flag, including the unrelated "Either --include or --exclude must be provided".
- **Evidence:** Verified at all five sites; forge's real error is ForgeError::ExportInvalidOscal ("Input is not a valid OSCAL artifact: ...", src/error.rs:264-267).
- **Remediation:** Pin exact discriminating phrases: assert stderr.contains("not a valid OSCAL artifact"); for success paths assert the exact success token emitted by forge validate case-sensitively instead of substring "valid"; for flag conflicts assert clap's "cannot be used with" signature AND the conventional usage-error exit code 2; in edge_both_flags_returns_error require the exact mutual-exclusion message rather than any InvalidArgument. No snapshot impact.

### F0839 — Fixture dependencies not verified up front; weak conversion postconditions
- **File:lines:** tests/export_integration.rs:9-13 (constants) and all tests; tests/integration_regression.rs:79-82 vs lines 46, 89, 126
- **Symbols:** CATALOG_JSON / CATALOG_XML / CATALOG_YAML; phase1_component_conversion pre-check
- **Category:** test · **Severity:** medium
- **Root cause:** export_integration.rs consumes three fixtures with no existence check; a deletion or partial sync surfaces as a CLI-level failure far from the cause. integration_regression.rs pre-checks only sample_profile.json while sibling tests also consume tests/fixtures/golden/small/input.md and tests/fixtures/full_policy.md unchecked (the finding said full_policy.json; the actual fixture is full_policy.md). Conversion postconditions are bare contains checks on stdout, so truncated output passes as long as the opening marker appears.
- **Evidence:** No Path::exists guard anywhere in export_integration.rs; integration_regression.rs:79-82 guards one fixture only.
- **Remediation:** Add a shared require_fixtures() helper asserting existence of every consumed fixture with a message naming the missing path, called at the top of each test. Strengthen conversion postconditions per F0833's prescription (parse output, check nested fields). No snapshot impact.

### F0854 — Unsound golden normalization (UUID shape collapse + lost key context)
- **File:lines:** tests/golden_edge_case_tests.rs:143-165 (normalize_value), esp. 145-149 and 155-157
- **Symbols:** normalize_value, normalize_for_comparison, UUID_RE, NORMALIZED_UUID
- **Category:** test · **Severity:** medium
- **Root cause:** (1) Any string merely matching UUID shape collapses to NORMALIZED_UUID, so goldens can contain wrong or duplicated stable IDs and still pass — real ID regressions masked at every assert_expected_output consumer and in the insta snapshots. (2) Array elements recurse with parent_key = None (lines 155-157), so a nondeterministic timestamp inside any array escapes normalization and flakes; the time-field rule fires only on the exact key "last-modified".
- **Evidence:** Verified both behaviors in the current normalize_value; the same pattern exists in tests/golden_file_tests.rs:121-141.
- **Remediation:** Refuse to normalize degenerate UUIDs (all-zero, or failing uuid::Uuid::parse_str) and compare them verbatim; thread parent-key context through arrays (pass the enclosing key, or the sibling key for objects inside arrays); broaden the time-field predicate to keys ending in -modified/-created or matching the timestamp shape. Apply symmetrically in golden_file_tests.rs. Snapshot impact: none for well-formed inputs; degenerate IDs will now diff (intended).

### F0855 — EC-6 tests only the rotation half of the stable-ID contract
- **File:lines:** tests/golden_edge_case_tests.rs:368-372 (ec06_substantive_change_rotates_stable_ids)
- **Symbols:** ec06_substantive_change_rotates_stable_ids, extract_stable_ids
- **Category:** test · **Severity:** medium
- **Root cause:** Only changed_count >= 1 is asserted. An implementation that regenerates every UUID whenever any change is detected passes here, defeating ec06's purpose given sibling ec05 exists specifically to prove whitespace edits don't rotate. Nothing checks that untouched requirements retain IDs or that the control-id universe is unchanged.
- **Evidence:** Verified: the only ID assertion is changed_count >= 1 (lines 370-372).
- **Remediation:** Add a retained count of controls whose IDs are equal across runs and assert retained >= 1 (the fixture mutates prose of one requirement only — confirm first); assert ids_a.len() == ids_b.len() so requirements cannot appear/vanish. No snapshot impact.

### F0856 — Strategy matrix re-runs fixtures asserting only exit codes
- **File:lines:** tests/golden_edge_case_tests.rs:451-497 (strategy_matrix_dual_strategy_and_agnostic_coverage, dual matrix arms)
- **Symbols:** strategy_matrix_dual_strategy_and_agnostic_coverage, run_convert
- **Category:** test · **Severity:** medium
- **Root cause:** For EC-2/EC-3/EC-4/EC-5/EC-6/EC-7 the matrix arms assert only code == 0; neither side compares against the golden JSON even though load_expected_json/assert_expected_output are available right above. The recorded statuses are hardcoded literals, so the snapshot mirrors this weak logic; content/serialization regressions are invisible here, and renaming/deleting a dedicated test leaves nothing enforcing parity.
- **Evidence:** Verified matrix arms at lines 470-497 assert only exit codes.
- **Remediation:** Extract a per-fixture helper shared with the dedicated tests that runs both strategies and calls assert_expected_output against each fixture's expected-catalog.json/expected-component-definition.json (plus the EC-5/EC-6 stable-ID assertions); drive the matrix status map from real outcomes. No snapshot impact unless output shape changes.

### F0866 — Catalog accuracy extractor is one level deep
- **File:lines:** tests/golden_file_tests.rs:155-166 (extract_catalog_control_ids)
- **Symbols:** extract_catalog_control_ids
- **Category:** bug · **Severity:** medium
- **Root cause:** The extractor reads only catalog.groups[*].controls[*].id. OscalCatalog models root-level controls (src/oscal/catalog.rs:37-39) and OscalGroup models nested sub-groups (src/oscal/catalog.rs:63-65); OSCAL permits both. If any fixture or future output uses them, the accuracy gate under-counts and can report 100% while missing extracted controls.
- **Evidence:** Struct fields and one-level traversal verified. Latent today: forge's build_catalog flattens child sections into one group per top-level section and leaves catalog.controls / group.groups empty, so current fixtures don't trigger it.
- **Remediation:** Walk groups recursively (including nested groups) and include catalog.controls; better, deserialize into the typed OscalCatalog/OscalGroup structs instead of hand-written pointer paths. No snapshot impact.

### F0865 — Silent default arms in stringly-typed test dispatch
- **File:lines:** tests/golden_file_tests.rs:189-194 (extract_control_ids); tests/oscal_cli_round_trip.rs:123-128 (artifact_type_root)
- **Symbols:** extract_control_ids, artifact_type_root
- **Category:** bug · **Severity:** medium
- **Root cause:** Unknown strategy strings fall through the "_ => Vec::new()" arm, yielding a well-formed 0% AccuracyReport (or vacuous 100% combined with F0863) instead of signaling programmer error. In oscal_cli_round_trip.rs, an unexpected artifact_type maps to pointer prefix "/unknown/metadata/oscal-version", which misses unconditionally; declared_oscal_version becomes None and the later assert_eq!(content["declared_oscal_version"], SCHEMA_VERSION_USED) (line 141) fails with a misleading left/right mismatch far from the cause.
- **Evidence:** Both silent fallback arms verified.
- **Remediation:** Change extract_control_ids to take the Strategy enum (or a test-local enum) and match exhaustively, deleting the default arm. In artifact_type_root, replace the fallback "unknown" arm with a panic naming the unsupported type (loud test-infrastructure error). No snapshot impact.

### F0863 — measure_accuracy passes vacuously on empty expected fixture
- **File:lines:** tests/golden_file_tests.rs:205-215 (measure_accuracy)
- **Symbols:** measure_accuracy, AccuracyReport
- **Category:** test · **Severity:** medium
- **Root cause:** expected_count == 0 returns 100% accuracy, so an empty, malformed (no groups[*].controls[*] array), or accidentally truncated golden passes the >= 95% MS-4 gate while measuring nothing. The insta snapshot still diffs structurally, but this independent accuracy gate's purpose is defeated.
- **Evidence:** Early-return at lines 207-214 verified.
- **Remediation:** Treat expected_count == 0 as a hard failure: panic that the expected fixture contains no requirements (golden corrupted or truncated). All committed golden fixtures contain requirements, so no false positives. No snapshot impact.

### F0864 — Accuracy is recall-only; extras and duplicates unpenalized
- **File:lines:** tests/golden_file_tests.rs:217-240 (measure_accuracy scoring loop)
- **Symbols:** measure_accuracy
- **Category:** test · **Severity:** medium
- **Root cause:** Scoring is set-membership recall over expected IDs via a HashSet, so hallucinated extra controls inflate nothing and duplicate emitted requirements (same control-id twice) still match — the OSCAL artifact can carry duplicated entries while the 95% gate passes. Combined with schema checks living only in schema_validation_tests (which see expected files, not raw output), precision can regress undetected.
- **Evidence:** actual_set HashSet at lines 222-223; the loop iterates expected IDs only.
- **Remediation:** Compute precision too: count actual_ids entries absent from the expected set and duplicates (actual_ids.len() minus unique count); extend AccuracyReport with extra_requirements and duplicate_count and assert both zero (or fold into the threshold); update the eprintln report. No snapshot impact.

### F0862 — Documented UPDATE_GOLDEN_FILES workflow is not implemented
- **File:lines:** tests/golden_file_tests.rs:31 (module doc) — no implementation anywhere in the file
- **Symbols:** assert_accuracy (catalog variant ~line 496, component variant ~line 579)
- **Category:** documentation · **Severity:** medium
- **Root cause:** Module docs instruct "UPDATE_GOLDEN_FILES=1 cargo test golden" to regenerate expected JSON, but nothing in this file reads that env var; following the documented workflow silently does nothing. Sibling tests/ssp_template_test.rs:154-158 already implements the pattern, proving intent.
- **Evidence:** UPDATE_GOLDEN_FILES appears only in the doc comment here; implemented in ssp_template_test.rs.
- **Remediation:** In each assert_accuracy, before comparing: if env var UPDATE_GOLDEN_FILES is set, write serde_json::to_string_pretty(actual) plus trailing newline to expected_path and return early; mirror ssp_template_test.rs's write style and error message. No snapshot impact.

### F0869 — Schema validation gates only hand-maintained fixtures, never raw pipeline output
- **File:lines:** tests/golden_file_tests.rs:629-696 (schema_validation_tests); absent from run_catalog (~476) and run_component (~555)
- **Symbols:** all_expected_catalog_files_pass_schema_validation, all_expected_component_files_pass_schema_validation, run_catalog, run_component
- **Category:** test · **Severity:** medium
- **Root cause:** validate_artifact is applied only to the hand-maintained expected-*.json fixtures, never to the raw pipeline output that ships. A serializer schema regression surfaces only indirectly as an insta diff to be manually accepted, and a hand-edited/stale fixture can bake in invalid structure this gate keeps blessing.
- **Evidence:** Verified neither run_catalog nor run_component calls forge::validate::validate_artifact.
- **Remediation:** In run_catalog/run_component, immediately after parsing actual, call forge::validate::validate_artifact(&actual, OscalModelType::Catalog / ComponentDefinition) and panic on error or invalid result. Keep the fixture-only tests as the tripwire for fixture validity. Snapshot impact: none unless current pipeline output violates the schema, in which case tests fail loudly (intended).

### F0843 — count_controls disagrees with recursive collectors
- **File:lines:** tests/integration_cross_feature.rs:167-171 (count_controls) vs 116-132 and 150-165 (recursive collectors); gate at 293-297
- **Symbols:** count_controls, collect_modality_from_controls, collect_params_from_controls
- **Category:** bug · **Severity:** medium
- **Root cause:** count_controls sums only top-level group.controls lengths, while collect_modality_from_controls/collect_params_from_controls recurse into nested control["controls"]. With nested children the two views of "a control" disagree: the >= 5 gate guarding the EC-4 atomizer split can pass while the compared collections include controls the count never accounted for (or conversely mask a miscount). The gate also uses >= where the fixture documents exactly 5.
- **Evidence:** Verified non-recursive count_controls (lines 167-171) vs recursive collectors; gate total_controls >= 5 at lines 294-297.
- **Remediation:** Make count_controls recursive (1 plus recursive count per control, mirroring the collectors) and assert == 5 for the documented fixture total so over-generation/duplication is caught too. No snapshot impact.

### F0845 — Round-trip comparisons erase per-control attribution
- **File:lines:** tests/integration_cross_feature.rs:99-112 (collect_modality_props), 134-148 (collect_params)
- **Symbols:** collect_modality_props, collect_params
- **Category:** test · **Severity:** medium
- **Root cause:** Modalities are flattened into one globally sorted list; params are keyed only by id with per-param value sorting. Swapping prop values between two controls, attaching a param to the wrong control, or reordering controls yields byte-identical collections and passes the round-trip fidelity checks.
- **Evidence:** Verified global result.sort() at line 111 and id-keyed param collection with value sorting.
- **Remediation:** Key collections by control identity: BTreeMap from control id to sorted Vec of (prop_name, value) for modalities, and BTreeMap from control id to Vec of (param_id, values) for params, recursing with attribution; keep normalization only where the format genuinely doesn't guarantee ordering within one object. Update the round-trip tests consuming these helpers. No snapshot impact.

### F0852 — Byte-index slicing can panic inside the assertion-failure path
- **File:lines:** tests/integration_profile_e2e.rs:390-395 (profile_xml_yaml_formats XML branch)
- **Symbols:** profile_xml_yaml_formats
- **Category:** bug · **Severity:** medium
- **Root cause:** The preview slice xml_content[..len().min(200)] cuts at byte index 200; whenever a failing output contains non-ASCII text there is a nonzero probability 200 falls mid multi-byte UTF-8 character, replacing the intended diagnostic with a byte-index panic inside the assertion-failure path.
- **Evidence:** Verified the slice verbatim at line 394.
- **Remediation:** Back off to the nearest char boundary before slicing (loop decrementing end while !xml_content.is_char_boundary(end)), or build the preview from chars().take(200). No snapshot impact.

### F0853 — take(2) before filter_map silently shrinks the ID vector
- **File:lines:** tests/integration_profile_e2e.rs:52-68 (extract_control_ids)
- **Symbols:** extract_control_ids
- **Category:** test · **Severity:** medium
- **Root cause:** .take(2) is applied before .filter_map(|c| c["id"].as_str()), so if either of the first two control objects lacks a string id, the vector silently shrinks (e.g. to 1) and every dependent test quietly loses multi-ID coverage; dropped entries produce no signal, so a degraded fixture surfaces far downstream.
- **Evidence:** Verified order take(2).filter_map(...) at lines 56-60.
- **Remediation:** Move .take(2) after filter_map and map, then assert ids.len() == 2 with a "fixture degradation" message before returning. No snapshot impact.

### F0848 — Silent-noop normalization in round-trip component comparison
- **File:lines:** tests/integration_round_trip.rs:35-47 (clear_control_implementations)
- **Symbols:** clear_control_implementations
- **Category:** test · **Severity:** medium
- **Root cause:** Every step of the pointer/array lookup chain is fallible and swallowed by if-let/and_then. If the CLI artifact shape changes (root key renamed, components flattened, wrapper added), neither side gets stripped; since control-implementations cancels out on both sides of the EC-5 comparison, this normalization silently stops verifying the WI-28 omission contract instead of failing loudly.
- **Evidence:** Verified the silent if-let chain.
- **Remediation:** Make the helper panic when /component-definition/components is absent (the structure is mandatory in these tests) and return the count of removals; assert in the component round-trip test that at least one removal happened so normalization demonstrably ran. No snapshot impact.

### F0875 — Vacuous green when oscal-cli is absent
- **File:lines:** tests/oscal_cli_round_trip.rs:24-36 (invoker_if_available / skip_if_no_oscal_cli), 178, 213 (early returns in SC-001/SC-002)
- **Symbols:** skip_if_no_oscal_cli, catalog_json_xml_yaml_json_round_trip, component_json_xml_yaml_json_round_trip
- **Category:** test · **Severity:** medium
- **Root cause:** When PathDetector finds nothing, both round-trip tests return early and cargo test reports "ok"; the only signal is an unstructured eprintln on stderr. Zero conversions, divergence-log writes, or ForgeFix-count assertions run. If CI doesn't install oscal-cli, WI-37 coverage is permanently disabled while green.
- **Evidence:** Verified early-return pattern with eprintln SKIP message.
- **Remediation:** In the None branch, panic when env var FORGE_REQUIRE_OSCAL_CLI is set; set that variable in the CI job that installs oscal-cli so unavailability fails loudly where coverage is expected, while local runs keep skipping gracefully. No snapshot impact.

### F0860 — Profile golden tests rely on normalization instead of a fixed timestamp
- **File:lines:** tests/profile_golden_file_tests.rs:25-40 (golden_include_only), 50-66 (golden_exclude_only)
- **Symbols:** golden_include_only, golden_exclude_only, build_profile
- **Category:** test · **Severity:** medium
- **Root cause:** Both tests pass timestamp_override = None, so last-modified is generated live; determinism depends entirely on common::normalize_for_snapshot recognizing the exact key "last-modified" and format. If metadata assembly renames the field or changes serialization shape, normalization silently stops applying and snapshots flake against wall-clock time.
- **Evidence:** Verified None passed as the fifth argument; build_profile (src/oscal/profile.rs:216-222) accepts timestamp_override: Option<DateTime<Utc>> and routes it into MetadataOptions.
- **Remediation:** Parse a fixed timestamp (e.g. 2026-01-01T00:00:00Z) and pass Some(fixed_ts) in both tests. Snapshot impact: golden_include_only.snap / golden_exclude_only.snap change only if the fixed value differs from the normalized placeholder — align the fixed timestamp with the normalizer's replacement or run cargo insta review once.

### F0878 — Hardcoded /tmp path makes the missing-file test environment-dependent
- **File:lines:** tests/profile_validation_tests.rs:239-241 (edge_invalid_catalog_path)
- **Symbols:** edge_invalid_catalog_path
- **Category:** test · **Severity:** medium
- **Root cause:** Path::new("/tmp/nonexistent-catalog-99999.json") is a fixed shared path: another test, CI job, or leftover user file flips .exists() to true and turns the FileNotFound assertion into a spurious failure; on Windows the leading "/" resolves against the current drive inconsistently.
- **Evidence:** Verified the hardcoded literal at line 240.
- **Remediation:** Use tempfile::tempdir() and join a nonexistent filename (tmp.path().join("nonexistent-catalog.json")); hold the TempDir for the test lifetime. No snapshot impact.

### F0880 — Property strategies exclude exactly the input classes most likely to panic
- **File:lines:** tests/property_tests.rs:16 ("[ -~]{0,500}"), 34 and 52 (".{0,300}" / ".{0,500}"); SUT slicing at src/parse/atomize.rs:218,228 and src/citation.rs:224-231
- **Symbols:** atomize_never_panics, atomize_produces_at_least_one, atomize_bounded_output
- **Category:** test · **Severity:** medium
- **Root cause:** The strategies generate only single-line printable ASCII (proptest's "." excludes newlines entirely), so the headline "never panics on arbitrary input" property exercises a narrow domain. Both SUTs do raw byte-range slicing on input (text[last_end..range.start] in atomize.rs/citation.rs) and normalize_prose advertises handling newlines/tabs/Unicode whitespace — precisely the classes (UTF-8 boundaries, list bullets, multi-line) most likely to surface panics are untested.
- **Evidence:** Verified strategy literals and the byte-range slicing sites.
- **Remediation:** Change at least the never-panics strategy to a Unicode-aware domain, e.g. string_regex over non-control characters (keeps emoji/CJK/accents/newlines), or any::<String>() filtered of control bytes; add explicit multiline/bullet/UTF-8 fixture cases to the atomize suite. No snapshot impact.

### F0882 — Citation properties pass vacuously when the extractor returns Err
- **File:lines:** tests/property_tests.rs:181-188 (P-9), 192-208 (P-10, paired-Err arm at 205), 211-222 (P-11), 226-235 (P-12), 255-264 (P-14)
- **Symbols:** citation_no_double_spaces, citation_deterministic, citation_ids_are_valid_uuids, citation_text_is_substring_of_original, cleaned_text_not_longer
- **Category:** test · **Severity:** medium
- **Root cause:** Bodies are guarded by "if let Ok(...)"; P-10's match arm accepts paired Err and discards both error values. If the extractor regresses to failing on (nearly) all inputs, this entire half of the suite passes vacuously — zero properties exercised, CI green.
- **Evidence:** Verified all five guarded bodies and the paired-Err discard arm.
- **Remediation:** Add a cheap liveness prop_assert that extract_citations_from_text("req-prop", "comply with policy") succeeds (trivial input must always parse); in P-10 compare the two error displays in the paired-Err arm instead of discarding them. No snapshot impact.

### F0881 — Bare unwrap() on fallible SUT calls inside proptest bodies
- **File:lines:** tests/property_tests.rs:45-46 (atomize_produces_at_least_one), 63 (atomize_bounded_output), 79 (atomize_all_have_stable_id), ~247 (P-13 url_citations_contain_scheme)
- **Symbols:** atomize_produces_at_least_one, atomize_bounded_output, atomize_all_have_stable_id, url_citations_contain_scheme
- **Category:** test · **Severity:** medium
- **Root cause:** .unwrap() on SUT results conflates a legitimate rejection with a harness bug: on Err the run dies as a plain panic with no SUT context instead of a structured prop_assert failure — inconsistent with sibling P-11/P-12 which degrade gracefully via verify-or-fail. Shrinking still works, but diagnosis quality suffers and a deliberate Err-for-edge-cases contract change reads as a crash.
- **Evidence:** Verified the unwrap sites; P-13 at ~line 247 unwraps extract_citations_from_text.
- **Remediation:** Map Err into prop_assert!(false, "... rejected {:?}: {e}", req.text) followed by unreachable!() at the atomize sites; apply the same verify-or-fail pattern to P-13. No snapshot impact.

### F0870 — Trace link test verifies only aggregate counts
- **File:lines:** tests/trace_integration.rs:80-84 (catalog_trace_one_link_per_control)
- **Symbols:** catalog_trace_one_link_per_control; TraceLink.oscal_element_id
- **Category:** test · **Severity:** medium
- **Root cause:** The test asserts trace_links.len() == total_controls == 3 but never pairs links to controls; a balanced mismatch (one control duplicated in trace links, another missing) still passes because both totals stay 3.
- **Evidence:** Verified; TraceLink.oscal_element_id equals the control's uuid (src/oscal/catalog.rs sets oscal_element_id = stable_id.clone(), and OscalControl.uuid = stable_id).
- **Remediation:** Collect sorted catalog.groups[*].controls[*].uuid and sorted trace_links[*].oscal_element_id, assert vector equality, then keep the length-3 check. No snapshot impact.

### F0948 — Unexplained cargo net retry=10 in checked-in config
- **File:lines:** .cargo/config.toml:1-2
- **Symbols:** n/a (config: [net] retry = 10)
- **Category:** maintainability · **Severity:** low
- **Root cause:** The file applies to every contributor workstation and CI runner but carries no rationale; maintainers cannot distinguish intentional team-wide tuning from leftover local debugging, nor know when it is safe to remove.
- **Evidence:** File contains exactly "[net]" and "retry = 10".
- **Remediation:** Add a comment documenting why retry=10 (flaky proxy/mirror, CI reliability) and the removal condition; if it is machine-specific tuning, scope it to ~/.cargo/config.toml or CARGO_NET_RETRY instead.

### F1074 — Unconditional "* text=auto" leaves high-churn files environment-dependent
- **File:lines:** .gitattributes:2 and 28-29
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** Frequently-shared non-source files without explicit rules (Cargo.lock — committed per git ls-files, LICENSE/NOTICE, *.ps1, Makefile) follow each developer's core.autocrlf/core.eol, causing spurious lockfile/repo-metadata churn in PRs and unstable on-disk hashes across platforms. Opportunistic hardening, not a correctness defect in the current ruleset.
- **Evidence:** Verified "* text=auto" with explicit eol rules only for the listed extensions.
- **Remediation:** Add explicit rules: Cargo.lock text eol=lf, *.ps1 text eol=crlf (and LICENSE/NOTICE text eol=lf if present). No code impact.

### F1066 — CI cache key hashes Cargo.lock at any depth
- **File:lines:** .github/workflows/ci.yml:38-39 (same pattern in .github/workflows/release.yml:37-40)
- **Symbols:** n/a (hashFiles('**/Cargo.lock'))
- **Category:** maintainability · **Severity:** low
- **Root cause:** The any-depth glob means a future vendored/example crate with its own lockfile would silently bust every OS's cache.
- **Evidence:** Verified at ci.yml:39 and release.yml:40.
- **Remediation:** Pin the hash to the root manifest: hashFiles('Cargo.lock') in both workflows.

### F1064 — Redundant schema-provenance re-run in CI
- **File:lines:** .github/workflows/ci.yml:44-45
- **Symbols:** n/a (step "Verify OSCAL schema provenance")
- **Category:** maintainability · **Severity:** low
- **Root cause:** The previous "cargo test" step already compiles and runs tests/schema_provenance_test.rs (autotests include all tests/*.rs; Cargo.toml has no [[test]] harness=false opt-out for it — verified). The explicit re-run adds runtime without additional verification.
- **Evidence:** Cargo.toml verified: only benches opt out via [[bench]] harness=false; the step exists at lines 44-45.
- **Remediation:** Remove the step, or if the intent is guaranteed single-target execution (fast-fail on provenance regressions), document that intent.

### F1067 — Criterion benchmarks gate merges with hard timings on shared runners
- **File:lines:** .github/workflows/ci.yml:58-60
- **Symbols:** n/a
- **Category:** test · **Severity:** low
- **Root cause:** The blocking bench step runs with --warm-up-time 1 --measurement-time 3 on shared hosted runners where CPU steal/noisy neighbors routinely produce 10-20% variance, risking intermittent red builds from timing noise unrelated to code quality. No threshold tolerance config exists in the repo.
- **Evidence:** Step verified; no criterion.toml or threshold tuning found.
- **Remediation:** Remove the timing overrides to accept Criterion defaults, add tolerance thresholds (committed criterion.toml regression_threshold), or move benchmarks to a scheduled/nightly workflow so latency noise does not gate merges.

### F1060 — SBOM step assumes exactly forge.cdx.json at repo root
- **File:lines:** .github/workflows/release.yml:131-132
- **Symbols:** n/a (Generate CycloneDX SBOM step)
- **Category:** bug · **Severity:** low
- **Root cause:** "mv forge.cdx.json ..." relies on cargo-cyclonedx's default output name (package-name.cdx.json). A package rename or multi-crate workspace makes the mv fail hard; some cargo-cyclonedx versions/configs can emit files in different directories.
- **Evidence:** Verified "cargo cyclonedx --format json" followed by "mv forge.cdx.json".
- **Remediation:** Use a glob move (mv ./*.cdx.json "forge-TAG-sbom.cdx.json") or pin the artifact name via an explicit output-path option so naming is configured, not implicit.

### F1069 — Unanchored directory patterns in .gitignore
- **File:lines:** .gitignore:3-4 (debug/, release/); same latent issue for coverage/ (~line 20)
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** Unanchored debug/ and release/ (and coverage/) match at any depth; a future legitimate path like src/fixtures/debug/ or tests/fixtures/coverage/ would be silently excluded from version control and hard to diagnose. Latent today — no such directories exist.
- **Evidence:** Patterns verified in .gitignore.
- **Remediation:** Anchor to the root: /debug/, /release/, /coverage/. target/ is already rooted.

### F1070 — Overly broad *.prof* gitignore glob
- **File:lines:** .gitignore:7 (finding cited 27-29 — stale line numbers; the pattern lives in the Rust/Cargo section)
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** *.prof* matches any path containing ".prof" anywhere in the tree, so legitimate files such as model.profile.json or run.profiles.csv would silently become un-commitable.
- **Evidence:** Pattern "*.prof*" verified at .gitignore:7.
- **Remediation:** Narrow to concrete profiler outputs: *.profdata, *.profraw, perf.data*, *.pprof.

### F1071 — Duplicate .DS_Store entry in .gitignore
- **File:lines:** .gitignore:54-55 (OS section) vs ~line 27 (IDEs and Editors section)
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** .DS_Store is listed twice; harmless to git but confuses auditors grepping the file and can drift over time.
- **Evidence:** Both entries verified.
- **Remediation:** Consolidate into one entry under the OS section (or drop the duplicate block).

### F0996 — use_small_heuristics = "Max" is an undocumented density trade-off
- **File:lines:** .rustfmt.toml:3
- **Symbols:** n/a
- **Category:** style · **Severity:** low
- **Root cause:** All width heuristics are set to max_width = 100, allowing very dense single-line constructs that reduce diff granularity; nothing records whether this density is a deliberate team-wide decision.
- **Evidence:** File verified: edition 2024, max_width 100, use_small_heuristics Max.
- **Remediation:** Add a comment documenting the deliberate choice, or switch to Default/Small after team discussion (would require a one-time reformat).

### F1023 — Mixed version-requirement granularity in Cargo.toml
- **File:lines:** Cargo.toml:8-28 (dependencies)
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** Loose carets (clap = "4", regex = "1") mix with arbitrarily precise floors (serde_json = "1.0.149", uuid = "1.20.0", tempfile = "3.25.0"). Caret requirements are not pins, so the precise forms neither improve reproducibility (Cargo.lock is committed) nor restrict upgrades — they only raise minimum compatible versions and force churn on transitive conflicts.
- **Evidence:** Verified in current Cargo.toml.
- **Remediation:** Pick one granularity policy across the manifest (typical: major-only carets, e.g. serde_json = "1") and rely on the lockfile for reproducible builds.

### F1021 — jsonschema default-features=false is an undocumented deliberate choice
- **File:lines:** Cargo.toml:22
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** default-features = false drops the resolver features (resolve-http, resolve-file). Safe today because every call site uses embedded/local schemas via validator_for(), but any schema that grows an absolute $ref to an HTTP/file URI will compile yet fail to resolve at runtime with a confusing error. (Note: current version is 0.45.0.)
- **Evidence:** Verified jsonschema = { version = "0.45.0", default-features = false }.
- **Remediation:** Add a short comment marking the choice deliberate and stating when to re-enable resolvers.

### F1022 — Edition 2024 implies an MSRV but no rust-version is declared
- **File:lines:** Cargo.toml:4
- **Symbols:** n/a
- **Category:** documentation · **Severity:** low
- **Root cause:** edition = "2024" implies a Rust 1.85+ toolchain floor, but without rust-version there is no MSRV gate and no machine-readable compatibility signal for downstream consumers.
- **Evidence:** Verified no rust-version key in Cargo.toml.
- **Remediation:** Declare rust-version = "1.85" (or the true floor, e.g. 1.93.0 per project instructions).

### F1025 — Package metadata missing description/repository
- **File:lines:** Cargo.toml:1-5 ([package])
- **Symbols:** n/a
- **Category:** documentation · **Severity:** low
- **Root cause:** [package] has only name/version/edition/license; crates.io rejects publishes without a description and warns on missing repository, so the manifest is not publish-clean.
- **Evidence:** Verified [package] section contents.
- **Remediation:** Add description, repository, and readme = "README.md" (keywords optional).

### F0043 — unwrap() inside timed bench closures without context
- **File:lines:** benches/atomize.rs:28, 34 and the document-bench body (~64-80)
- **Symbols:** bench_atomize_compound, bench_atomize_atomic, bench_atomize_document_100
- **Category:** maintainability · **Severity:** low
- **Root cause:** A regression making atomization return Err aborts the entire criterion run with a bare "called Result::unwrap() on an Err" panic carrying no bench name.
- **Evidence:** Verified .unwrap() in the timed closures.
- **Remediation:** Use .expect("bench-name: fixture must atomize") labels, or validate the input once before the timing loop and keep an unwrapped fast path inside it.

### F0034 — Benchmark re-declares the ingest cap, drifting from production
- **File:lines:** benches/export_bench.rs:22-23
- **Symbols:** MAX_SIZE_BYTES (bench) vs forge::io::MAX_FILE_SIZE (src/io.rs:8, now 50 MB — the export guard at src/cli/export.rs:283) and the CLI --max-size default 10 MB (src/cli/mod.rs:259-263)
- **Category:** maintainability · **Severity:** low
- **Root cause:** The local const duplicates the ingest limit. The finding cited forge::io::MAX_FILE_SIZE as the authoritative 10 MB value, but that constant is now 50 MB and governs the export path; the 10 MB cap lives as the CLI --max-size default. Either way, the local copy can drift from whichever limit production applies.
- **Evidence:** Bench constant verified; MAX_FILE_SIZE = 50 MB at src/io.rs:8; CLI default "10" (MB) at src/cli/mod.rs:260.
- **Remediation:** Export a single public constant for the convert-path ingest cap (e.g. forge::DEFAULT_MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024 used by the CLI default) and reference it here and in tests/common/mod.rs.

### F0033 — Bare unwrap()s across distinct bench bootstrap stages
- **File:lines:** benches/export_bench.rs:29-46 (build_catalog_json)
- **Symbols:** build_catalog_json
- **Category:** maintainability · **Severity:** low
- **Root cause:** Roughly 15 bare unwrap()s span ingest → parse → atomize → citation → catalog build → serialization; when the synthetic fixture breaks, the panic surfaces only the innermost error with no indication of which stage failed.
- **Evidence:** Verified the unwrap chain in build_catalog_json.
- **Remediation:** Use .expect("bench setup: <stage> failed") per call, or wrap phases with context.

### F0035 — Bench name encodes fixture size but nothing enforces it
- **File:lines:** benches/export_bench.rs:77-80
- **Symbols:** bench_export_pipeline
- **Category:** other · **Severity:** low
- **Root cause:** The doc header promises a 500KB+ artifact, but the benchmark name merely interpolates the computed size; if the synthetic fixture regresses below the documented threshold the benchmark still passes happily.
- **Evidence:** json_size_kb computed and interpolated into the bench id with no lower-bound assertion.
- **Remediation:** Add assert!(json_size_kb >= 500, ...) before the benchmark group so a size regression is caught.

### F0027 — Benchmark doc claims overstate corpus representativeness
- **File:lines:** benches/parameter_extraction.rs:3-7
- **Symbols:** n/a (module docs)
- **Category:** documentation · **Severity:** low
- **Root cause:** Docs claim each requirement is "~100 characters" and imply linear-scaling (SEC-3) / p95 conclusions, but sample texts are roughly 50-65 chars and the corpus is 10 fixed parameterized + 7 fixed plain strings with fixed one-third density, nesting_depth 0, empty citations, modality None — conclusions hold only for this narrow synthetic shape.
- **Evidence:** Verified PARAMETERIZED (10 entries) and PLAIN (7 entries) constants and the doc comment.
- **Remediation:** Soften the wording to scope claims to the synthetic corpus, or extend the corpus (varying lengths/density/nesting/citations) before citing it as empirical evidence.

### F0028 — Three byte-identical benchmark bodies
- **File:lines:** benches/parameter_extraction.rs:89-131
- **Symbols:** bench_extract_parameters_500, bench_extract_parameters_100, bench_extract_parameters_single
- **Category:** maintainability · **Severity:** low
- **Root cause:** All three bodies are identical except the size constant and bench name; future fixes (e.g. switching to iter_batched) must be applied in three places.
- **Evidence:** Verified three identical bodies.
- **Remediation:** Extract a shared helper fn bench_extract(c, name, n) and call it three times; use iter_batched with BatchSize::SmallInput so the doc.clone() leaves the timed region.

### F0055 — Per-stage bench numbers are not additive with the end-to-end figure
- **File:lines:** benches/pipeline_benchmark.rs:140-160 (bench_per_stage docs and pre-computation)
- **Symbols:** bench_per_stage
- **Category:** documentation · **Severity:** low
- **Root cause:** Stage timings run on fully warm, shared pre-computed inputs (same ingested/sections instance reused; the doc is constructed once and re-measured by both catalog_assembly and serialization), so per-stage values do not sum to the cold-run full-pipeline time and may differ from per-stage cost inside the pipeline.
- **Evidence:** Verified the shared pre-computation chain at lines 152-183.
- **Remediation:** Add a module-doc caveat warning against treating stage numbers as additive components of the end-to-end figure.

### F0054 — Bare unwrap()s in per-stage prep chain
- **File:lines:** benches/pipeline_benchmark.rs:152-183 (prep unwraps) vs labeled expect at line 83 (bench_full_pipeline)
- **Symbols:** bench_per_stage
- **Category:** test · **Severity:** low
- **Root cause:** All per-stage bodies and the out-of-loop pre-computation chain use bare unwrap(); failures surface as bare panic messages naming no stage, unlike the descriptive expect("Pipeline must not fail during benchmark") in the full-pipeline bench.
- **Evidence:** Verified both styles coexist.
- **Remediation:** Give each prep unwrap an expect naming the stage, e.g. expect("stage prep: ingest_file failed").

### F0022 — Copy-paste-prone inline sample strings in uuid benchmark
- **File:lines:** benches/uuid_benchmark.rs:5, 12, 19
- **Symbols:** bench_normalize_for_hashing, bench_generate_stable_id, bench_generate_stable_id_long_text
- **Category:** maintainability · **Severity:** low
- **Root cause:** The same domain sentence appears twice in different forms (padded here, clean in bench_generate_stable_id); editing one copy silently changes what that benchmark measures while the rest keep the old shape.
- **Evidence:** Verified inline literals in all three benches.
- **Remediation:** Hoist sample strings to module-level constants shared by all three benchmarks.

### F0023 — UUID bench inputs are all plain ASCII prose
- **File:lines:** benches/uuid_benchmark.rs:19 (and whole file)
- **Symbols:** same three bench fns
- **Category:** test · **Severity:** low
- **Root cause:** Inputs never exercise the shapes most likely to behave differently (empty/single-char strings, non-ASCII text, Unicode whitespace variants, trailing-only separators); edge-case cliffs in normalization/hashing would go unmeasured. No Throughput configured, so runs compare only raw nanoseconds dominated by fixed overhead.
- **Evidence:** Verified three ASCII-only samples.
- **Remediation:** Add a benchmark group covering those edge-case samples with per-sample Throughput::Bytes(len).

### F0104 — Third copy of the 10 MB ingest cap
- **File:lines:** benches/xml_benchmark.rs:22-25
- **Symbols:** MAX_SIZE_BYTES
- **Category:** maintainability · **Severity:** low
- **Root cause:** The cap is duplicated here, in benches/export_bench.rs, and in tests/common/mod.rs, while the authoritative value lives only as the CLI --max-size default; if production changes, some copies update and some don't, and the benchmark exercises a different ingest boundary than real callers. The comment even admits the duplication.
- **Evidence:** Verified constant and comment "Duplicated here because benchmarks cannot use the tests/common module".
- **Remediation:** Export one public library constant (e.g. forge::DEFAULT_MAX_SIZE_BYTES) and reference it everywhere.

### F0100 — XML bench back-matter diverges from production (no dedupe/cap)
- **File:lines:** benches/xml_benchmark.rs:44 (identical chain in benches/export_bench.rs:45) vs src/pipeline.rs:172-175
- **Symbols:** build_catalog_from_fixture (bench) vs run_catalog_pipeline (src)
- **Category:** bug · **Severity:** low
- **Root cause:** The pipeline collects citations via oscal::component_definition::collect_all_citations(&doc.sections), which dedupes by citation id (HashSet) and caps at MAX_CITATIONS = 10_000 (src/oscal/component_definition.rs:237-275). The benches instead call doc.collect_citations() (src/model/mod.rs:265), which clones every citation from every requirement without dedupe or cap. On a 50-page fixture with repeated references the benchmark serializes a back-matter section larger than and structurally unlike any real export — duplicate resources can even repeat the same UUID — skewing measurements in the opposite direction from a genuine artifact.
- **Evidence:** Verified both collectors and the pipeline call site.
- **Remediation:** Replace doc.collect_citations() with forge::oscal::component_definition::collect_all_citations(&doc.sections) in both benches.

### F0102 — Hand-assembled OscalCatalog in bench is a second copy of pipeline assembly
- **File:lines:** benches/xml_benchmark.rs:53-65
- **Symbols:** build_catalog_from_fixture
- **Category:** maintainability · **Severity:** low
- **Root cause:** The hand-assembled catalog currently mirrors run_catalog_pipeline's Step 11 exactly (build_catalog and src/pipeline.rs both emit controls: vec![]), so nothing is lost today — but it is a second, manually kept-in-sync copy of that assembly logic; if build_catalog or the pipeline starts populating root-level controls or changes metadata mapping, the benchmark keeps silently serializing a stale shape.
- **Evidence:** Verified assembly with controls: vec![] at line 62.
- **Remediation:** Prefer using the returned catalog directly, or add debug_assert(!catalog.groups.is_empty(), ...) so drift degrades loudly rather than quietly shrinking the measured workload.

### F1036 — License allowlist missing common permissive licenses
- **File:lines:** deny.toml:11-18 ([licenses].allow)
- **Symbols:** n/a
- **Category:** maintainability · **Severity:** low
- **Root cause:** ISC, BSD-2-Clause, MPL-2.0, Apache-2.0 WITH LLVM-exception, CC0-1.0/Unlicense, CDLA-Permissive-2.0 are absent; any future transitive dependency under one will hard-fail the license check and require manual adjudication.
- **Evidence:** Verified allow list: MIT, MIT-0, Apache-2.0, BSD-3-Clause, Zlib, Unicode-3.0.
- **Remediation:** Run cargo deny check licenses against the current lockfile; add needed licenses via a review step and document the intended procedure (raise allowlist by PR, citing the crate).

### F1033 — confidence-threshold restates cargo-deny's built-in default
- **File:lines:** deny.toml:19
- **Symbols:** n/a
- **Category:** documentation · **Severity:** low
- **Root cause:** confidence-threshold = 0.8 merely restates cargo-deny's default (0.8); as written it adds no behavior and only looks like deliberate tuning.
- **Evidence:** Line verified present.
- **Remediation:** Remove it to reduce config surface, or keep it with a comment stating why 0.8 suffices.

### F0019 — generate_ssp.py writes a vacuous SSP on empty extraction
- **File:lines:** examples/component-based/generate_ssp.py:14-18 (extraction) and 138-143 (write + success banner)
- **Symbols:** n/a (script top-level)
- **Category:** bug · **Severity:** low
- **Root cause:** If output/component-definition.json contains no implemented requirements (or the script runs against a stale/empty output), control_ids is empty, the SSP is still written with an empty control-implementation, and the run exits 0 printing "SSP generated with 0 implemented requirements" — downstream automation gets a vacuous document indistinguishable from a good one.
- **Evidence:** Verified: no emptiness guard before json.dump.
- **Remediation:** After building control_ids, raise SystemExit("No implemented requirements found ...") when the list is empty.

### F0009 — Catalog "_gdn" guidance duplicates the whole subsection for every control (redirected locus)
- **File:lines:** examples/component-based/output/catalog-new.json:42-44 (and ~22 sibling occurrences); root cause in src/oscal/catalog.rs:388 and src/parse/mod.rs:165-186 (accumulate_body_text)
- **Symbols:** build_catalog → build_control_parts(&control_id, req, req_sec.body_text.as_deref()); accumulate_body_text
- **Category:** maintainability · **Severity:** low
- **Root cause:** catalog-new.json is a generated artifact; the generator behavior lives in src/. accumulate_body_text folds list-item text into the section's body_text (parse/mod.rs appends on TagEnd::Item/List), and build_catalog attaches the entire section body_text as the guidance part of every control in that section (catalog.rs:388). Hence POL-CSP-020/-021/-022 each carry all three SI-1 statements verbatim (same for AC-1/AC-2/DP-1/DP-2/AL-1/AL-2/SI-2), bloating the catalog ~3x and leaving every guidance text ambiguous about which sibling control it applies to.
- **Evidence:** Verified identical _gdn prose across sibling controls in catalog-new.json and the wiring in src.
- **Remediation:** Decide intent: either stop attributing section-level body text as per-control guidance (attach only when the section has a single requirement, or emit it as group-level context), or exclude list-item text from body_text when that same list is the source of the section's requirements (track requirement spans and subtract them). Then regenerate examples/component-based/output/*.json via the README's forge convert commands. Snapshot impact: golden catalogs containing guidance parts will change — regenerate via UPDATE_GOLDEN_FILES + cargo insta review.

---

## PARTIAL findings

### F0867 — href normalization misses Windows-absolute paths (CWD-leak half unconfirmed)
- **File:lines:** tests/golden_file_tests.rs:101-114 (normalize_string_value href arm; Path::new(path_part).is_absolute() at line 110); run_catalog at line 477 uses the relative tests/fixtures/golden input path
- **Symbols:** normalize_string_value
- **Category:** bug · **Severity:** medium
- **Root cause (confirmed half):** The href rule rewrites only POSIX-absolute paths; a Windows-absolute href (file:///C:/... or C:\...) is not is_absolute() on Unix runners, so a machine-specific path would pass through verbatim into the snapshot and break reproducibility across contributor machines/CI runners.
- **Why partial:** The second claim — relative hrefs embedding the CWD — could not be confirmed: href values are produced through crate::io::sanitize_artifact_path (which extracts the filename, as documented in build_profile and the profile golden tests' comment), and the input paths here are repo-relative, so no CWD embedding was observable. The Windows-path leak is real but currently latent: forge always emits sanitized relative names, and no committed snapshot is affected today.
- **Remediation:** In the href arm, additionally normalize paths with a drive-letter prefix (^[A-Za-z]:), a file:// scheme, or leading ./ or ../ segments that resolve above the fixtures dir; alternatively normalize hrefs unconditionally the way source-file props are handled (lines 124-127). No snapshot impact for current fixtures.

---

## INVALID findings (one-line rationale)

- **F0838** (tests/export_integration.rs:93-96, bug · medium): Forge's missing-file path returns its own stable app-level message "File not found: '<path>'" (ForgeError::FileNotFound, src/error.rs:38-42, raised by src/cli/export.rs:278-279 before any io::Error occurs), which contains "not found" on every OS — no OS-rendered errno wording is reachable on this CLI path, so there is no Windows failure mode.
- **F1053** (.github/dependabot.yml:10-11, other · low): The finding itself states "Not a defect — schema itself is valid"; it is a purely optional suggestion with no defect to remediate.
- **F1061** (.github/workflows/release.yml:113-117, maintainability · low): Factually wrong premise — the sbom job declares no needs: at all and already runs in parallel with the build matrix; only hash/provenance/release wait on it, which is required for their inputs.

## DUPLICATE findings

None within this slice. F0034 and F0104 share the "duplicated ingest cap" theme but live in different files and call for consolidating onto a single public constant; F0854 and F0862 touch the same golden-test file but have distinct root causes (normalization soundness vs missing regeneration hook). Keeping all as separate, valid findings.
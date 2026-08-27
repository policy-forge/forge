# slice06 — validated verdicts (compact recovery)

Recovered from ValSlice06 scout context 2026-08-26 after provider-quota interruption.
Format: `F####|status|locus|fix directive`. V=valid, P=partial, I=invalid, D=duplicate.
Slice composition: 61 medium findings. Original texts: `../all_findings.json` (by id).

Status counts: 53 valid, 5 partial, 1 invalid, 2 duplicate.

```
F0668|V|src/round_trip/chain.rs:23-25|low; unique per-run intermediate names or document exclusive temp_dir precondition
F0669|V|src/round_trip/chain.rs:49|add RoundTripStep{step,from,to,#[source]} error variant wrapping convert legs
F0671|V|src/round_trip/chain.rs:23-52|low; pre-clean targets and remove intermediates on step failure
F0672|V|src/round_trip/chain.rs:82-92|record per-call format/input/output/timeout tuples; assert exact sequence
F0696|V|src/round_trip/comparator.rs:116-124|move timestamp suffixes into rules incl published/updated; thread rules
F0695|V|src/round_trip/comparator.rs:250-256|precompute uuid/name-ns indexes; avoid quadratic deep-equality rescans
F0694|V|src/round_trip/comparator.rs:253-256|carry expected_index/actual_index on Divergence; skip_serializing_if None
F0693|V|src/round_trip/comparator.rs:303-305|fall through to name/ns/positional when uuid lookup misses
F0676|V|src/round_trip/divergence.rs:104-105|low; replace passed field with computed passed() keeping serialized key
F0673|V|src/round_trip/divergence.rs:83|introduce UnverifiedBaseline variant for unknown versions; update tests
F0680|V|src/round_trip/log.rs:20-26|serialize to memory then write, or temp-file plus rename
F0701|V|src/round_trip/rules.rs:7-8|rename unordered_array_keys; document last-segment any-depth matching contract
F0700|V|src/round_trip/rules.rs:9-10|delete dead ignored_paths field or wire into compare_values
F0716|V|src/sanitize.rs:6-10|filter via char::is_control keeping tab/newline; optionally strip bidi
F0757|V|src/summary/format.rs:51-64|bucket at rounded>=59.95; assert 59.994s renders 1m 0s
F0719|V|src/summary/mod.rs:97-106|low; explicit-stack iterative control count replacing depth-64 recursion
F0734|V|src/testing/semantic_eq.rs:132-134|compare Number pairs numerically via as_f64 before type-mismatch branch
F0723|V|src/trace/extractor.rs:27-28|skip props with missing or non-string name/value
F0722|V|src/trace/extractor.rs:33|first-valid-wins assignment so malformed dupes cannot clobber
F0770|V|src/trace/formatter.rs:26-27|replace newline/cr/tab with spaces per cell after stripping
F0751|V|src/trace/mod.rs:47-82|snapshot mtime beside content read; pass to staleness check
F0750|V|src/trace/mod.rs:98-108|normalize pre-check PermissionDenied; use take(MAX+1) bounded read
F0744|V|src/trace/report.rs:104-108|low; replace stored summary with summary() computed from entries
F0746|V|src/trace/report.rs:110-111|low; fix doc contract; usize::MAX sentinel claim is fabricated
F0745|V|src/trace/report.rs:207|use assert_eq! or fixed 1e-9 tolerance not EPSILON
F0743|V|src/trace/report.rs:40-42|change source_line to Option<usize>; migrate extractor/formatter/resolver
F0739|V|src/trace/resolver.rs:11-27|return Staleness enum Fresh/Stale/Unknown instead of bool fallback
F0742|V|src/trace/resolver.rs:42-50|add fresh/stale/non-UTC tests against known file timestamps
F0765|V|src/trace/walker.rs:167-171|index-unique fallback ids plus warn for missing control-id
F0767|V|src/trace/walker.rs:173-177|prefix element_id with container uuid or source
F0764|V|src/trace/walker.rs:19-24|include ValidateError Display in TraceUnsupportedArtifact detail
F0762|V|src/types.rs:23-34|add ALL const and FromStr; dedupe lifecycle/config string tables
F0761|P|src/types.rs:48-53|co-locate Strategy::output_type(); misstated exhaustiveness mechanism; keep both enums mapped
F0775|V|src/uuid.rs:271-278|low; use >=, warn, report skipped requirements count to caller
F0780|V|src/validate/error_types.rs:41-44|private actual plus constructor enforcing truncate_value SEC-1 cap
F0779|V|src/validate/error_types.rs:83-95|default supported_input false or recompute from declared version
F0792|V|src/validate/formatter.rs:44-53|bracket-quote non-identifier segments; numeric keys are not indexes
F0793|V|src/validate/formatter.rs:63-81|redact secret-named keys; never echo whole document first chars
F0802|V|src/validate/mod.rs:138-148|require Some(Value::Object) single pass; reject null roots
F0803|V|src/validate/mod.rs:188-212|cache structured failure or retry compilation preserving source chain
F0804|V|src/validate/mod.rs:269-281|bound actual read via take(MAX+1); document symlink following
F0805|V|src/validate/mod.rs:300-337|extract shared collect_schema_and_version_errors; one path notation
F0797|V|src/validate/report.rs:104-112|serialize synthesized ValidationReport instead of hand-written JSON literal
F0800|I|src/validate/report.rs:52|invariant is_valid==errors.is_empty makes empty failure header unreachable
F0787|V|src/validate/semantic.rs:88-91|explicit-stack walk or emit validation-incomplete semantic warning
F0786|V|src/validate/semantic.rs:97-99|lowercase both uuid sides; special-case empty bare-# fragment
F0812|V|src/validate/version.rs:125-127|escape first then truncate_value to honor SEC-1 bound
F0811|V|src/validate/version.rs:84-86|store exact declared; reserve escaped rendering for error fields
F1018|V|supply-chain/audits.toml:1-3|import upstream audits per AR-051 or document cadence
F1047|P|supply-chain/config.toml:175-181|low; no stale pairs resolved in Cargo.lock; run vet prune after bumps
F1050|P|supply-chain/config.toml:4-5|add [[policy]] and imports; CI cargo vet gate already exists
F0810|V|tests/atomize_integration.rs:113-115|assert requirements.len()==1 before indexing preserved text
F0808|V|tests/atomize_integration.rs:131-142|insert stable_ids into HashSet; fail on duplicates
F0809|V|tests/atomize_integration.rs:53-61|assert fragment texts non-empty distinct and parent_text exact
F0816|V|tests/cli_integration.rs:937-943|comment validate-vs-convert exit taxonomy; add invalid-document exit test
F0824|P|tests/common/fixture_generator.rs:1294|extend fixture_determinism_test with size-range and structural counts
F0820|V|tests/common/fixture_generator.rs:212|emit numbered table captions or interpolate table references programmatically
F0821|V|tests/common/fixture_generator.rs:289|emit Appendix A section or remove dangling pointer
F0822|D-of-F0821|tests/common/fixture_generator.rs:525|same missing-appendix root cause; single fix covers both
F0828|V|tests/common/mod.rs:50-54|normalize only repo-local paths; preserve slash-prefixed OSCAL hrefs
F0829|V|tests/common/mod.rs:63-69|match UNC and extended-length verbatim Windows path prefixes
```

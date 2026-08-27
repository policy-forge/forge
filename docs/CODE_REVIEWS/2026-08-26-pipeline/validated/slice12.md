# slice12 — validated verdicts (compact recovery)

Recovered from ValSlice12 scout context 2026-08-26 after provider-quota interruption.
Format: `F####|status|locus|fix directive`. V=valid, P=partial, I=invalid, D=duplicate.
Slice composition: 47 low findings. Original texts: `../all_findings.json` (by id).

Status counts: 34 valid, 9 partial, 4 invalid, 0 duplicate.

```
F0781|V|src/validate/error_types.rs:140-143|derive semantic count as errors.len() - schema count
F0782|I|src/validate/error_types.rs:79-80|serde default fires only when field absent; value preserved
F0783|I|src/validate/error_types.rs:75-76|serde default fires only when field absent; premise wrong
F0788|V|src/validate/semantic.rs:113-133|reuse segment stack; render path only on error
F0789|V|src/validate/semantic.rs:143-148|document Profile/Mapping no-op or implement M-4 check
F0794|I|src/validate/formatter.rs:139-140|embedded schemas have no multi-type arrays; unreachable
F0795|V|src/validate/formatter.rs:218-233|replace fabricated sentinels with honest unavailable marker
F0796|V|src/validate/formatter.rs:66-78|emit structural summaries for arrays/objects before truncation
F0798|V|src/validate/report.rs:24-32|add support warning and trailing newline to valid branch
F0799|V|src/validate/report.rs:36-95|filter once; share vectors between summary and sections
F0801|V|src/validate/report.rs:253|add supported_input=false tests for both renderers
F0806|V|src/validate/mod.rs:310-312|move version.error instead of cloning it
F0807|V|src/validate/mod.rs:112-118|document profile, mapping-collection roots and AmbiguousArtifact error
F0813|V|src/validate/version.rs:32-37|add debug_assert/test guarding canonical baseline constants
F0814|P|src/validate/version.rs:163-173|gap real; verbatim-declared expectation contradicts current escaping
F0817|V|tests/cli_integration.rs:897-948|consolidate three missing-file tests into one
F0818|V|tests/cli_integration.rs:895|unique traceability IDs per test; prefix with WI
F0819|V|tests/cli_integration.rs:1108-1146|assert app-owned log markers, not level tokens
F0823|V|tests/common/fixture_generator.rs:384|derive section numbers programmatically or pin them in tests
F0825|V|tests/common/fixture_generator.rs:1281-1287|fix stats: 40 plain-text references, not 30 bracketed
F0826|P|tests/common/fixture_generator.rs:8|removing allow breaks per-binary clippy; gate module instead
F0827|V|tests/common/fixture_generator.rs:1316|document infallible-String invariant or propagate fmt::Result
F0830|P|tests/common/mod.rs:50-52|latent only; no current output embeds UUID substrings
F0831|V|tests/common/mod.rs:35-37|use shape-based timestamp normalization, not key name
F0834|V|tests/export_format_pairs.rs:27-32|table-drive the 18 format-pair tests
F0835|V|tests/export_format_pairs.rs:21-22|attach fixture/format context to unwrap failures
F0841|V|tests/export_integration.rs:99-107|reuse run_export; assert stderr names --format
F0842|V|tests/export_integration.rs:16|accept IntoIterator AsRef OsStr args ergonomically
F0846|P|tests/integration_cross_feature.rs:193-196|dedupe real; cited drift is intentional XML normalization
F0847|V|tests/integration_cross_feature.rs:148-150|replace unwrap_or and vec![] with map unwrap_or_default
F0849|V|tests/integration_round_trip.rs:62|anchor fixture and baseline paths to CARGO_MANIFEST_DIR
F0850|V|tests/integration_regression.rs:63|reference forge::oscal::metadata::OSCAL_VERSION instead of literal
F0851|V|tests/integration_regression.rs:57-59|parse uuid with uuid::Uuid::parse_str
F0857|V|tests/golden_edge_case_tests.rs:236-245|hoist fixture lists to shared constants; check files
F0858|V|tests/golden_edge_case_tests.rs:103-110|capture stdout; treat signaled exit as failure
F0859|V|tests/golden_edge_case_tests.rs:394-399|use fixture-derived citations, not synthetic Citation
F0861|P|tests/profile_golden_file_tests.rs:65-66|gap real; empty imports violates schema minItems:1
F0868|P|tests/golden_file_tests.rs:62-65|latent only; current UUID fields are whole-string
F0871|V|tests/trace_integration.rs:107-111|derive paths from catalog positions via by_oscal_element
F0872|V|tests/trace_integration.rs:194-196|restate invariant as inputs; WI-15 is merged
F0873|V|tests/trace_integration.rs:13-15|add Option stable_id variant covering CatalogBuild error
F0876|V|tests/oscal_cli_round_trip.rs:28|expect stating detector invariant instead of unwrap
F0877|V|tests/oscal_cli_round_trip.rs:19|source timeout from env var with fallback
F0879|V|tests/profile_validation_tests.rs:119-125|give ignored test compile-checked skeleton body
F0883|V|tests/property_tests.rs:258|drop +1; pipeline stages are provably non-growing
F1048|I|supply-chain/config.toml:563-565|split mirrors dev-only vs deployed graph; vet guards
F1051|P|supply-chain/config.toml:935-937|document wasip3 RC; dead-weight claim misreads cargo-vet
```

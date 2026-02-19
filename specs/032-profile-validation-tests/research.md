# Research: Profile Validation and Golden-File Tests (WI-32)

**Branch**: `032-profile-validation-tests` | **Date**: 2026-02-18

## Decisions

### D-1: WI-31 (--set-param) Is NOT Yet Implemented

**Decision:** Tests requiring parameter tailoring (`--set-param`, `modify` section) are implemented as `#[ignore]`-annotated stubs with a clear `// TODO: enable when WI-31 is implemented` comment.

**Evidence:** No `--set-param` flag exists in `src/cli/mod.rs`. The profile CLI handler (`src/cli/profile.rs`) is annotated "(WI-30)" and accepts only `--include`, `--exclude`, `--format`, `--output`. `build_profile()` in `src/oscal/profile.rs` does not generate a `modify` section.

**Impact on WI-32 scope:**
- FR-003 (parameter override validation) → implemented as `#[ignore]` stub
- FR-008 (conflicting params edge case) → implemented as `#[ignore]` stub
- Golden-file scenario 3 (include + set-param) → implemented as `#[ignore]` stub
- All other tests (FR-001, FR-002, FR-004, FR-005 include/exclude, FR-006, FR-007, FR-009, FR-010) → fully implementable now

**Alternatives considered:**
- Skip these tests entirely — loses traceability to PRD requirements
- Implement unconditionally — compilation errors (no --set-param CLI arg)
- Feature-flag guard — over-engineering; `#[ignore]` is the Rust-idiomatic approach

---

### D-2: OSCAL v1.2.0 Profile JSON Schema Source

**Decision:** Download from the official NIST OSCAL v1.2.0 GitHub release:
`https://github.com/usnistgov/OSCAL/releases/download/v1.2.0/oscal_profile_schema.json`

Embed in the binary at compile time via `include_str!("../../schemas/oscal_profile_schema.json")` in `load_schema()`, consistent with the WI-19 Catalog and Component schemas.

**Rationale:** OSCAL v1.2.0 is confirmed as the latest stable release (released 2025-12-12). The catalog and component schemas were embedded from the same release. The profile schema is distributed as a release asset, not in a raw GitHub directory (GitHub raw URL returns 404 for this release).

**Alternatives considered:**
- v1.1.3 schema (older, missing v1.2.0 Control Mapping additions; inconsistent with existing schemas)
- NIST csrc.nist.gov URL (referenced in the catalog schema `$id` but not a stable download endpoint)
- Generate from metaschema (over-engineering; official release artifact is the correct source)

---

### D-3: Normalization Utility Location

**Decision:** Add a `normalize_for_snapshot(json: &serde_json::Value) -> serde_json::Value` helper to `tests/common/mod.rs`.

**Rationale:** `tests/common/mod.rs` is the established location for shared test utilities (already used by `tests/adversarial_input_test.rs`, etc.). Adding normalization there makes it reusable for profile snapshot tests without modifying `tests/golden_file_tests.rs` (which is WI-21 infrastructure — AR guardrail: "DO NOT MODIFY WI-21 Golden-File Comparison Framework"). The same UUID/timestamp normalization rules apply identically to Profile output.

**Normalization rules (identical to WI-21 pattern):**
- All UUID-format strings → `"00000000-0000-0000-0000-000000000000"`
- `last-modified` field values → `"2026-01-01T00:00:00Z"`
- Absolute path hrefs → `"NORMALIZED_PATH"`
- Applied recursively; idempotent

**Alternatives considered:**
- Inline normalization in each test file — duplication; harder to maintain
- Move normalize from `golden_file_tests.rs` and import — modifies WI-21 code (forbidden by AR)
- Use `insta`'s built-in redaction feature — more powerful but adds complexity; WI-21 pattern is established

---

### D-4: Test File Organization

**Decision:** Two new integration test files:
- `tests/profile_validation_tests.rs` — schema validation tests + edge case tests
- `tests/profile_golden_file_tests.rs` — insta snapshot golden-file tests

**Rationale:** Mirrors the WI-21 pattern (`tests/golden_file_tests.rs`). Separating schema/edge case tests from snapshot tests: (1) keeps clippy and fmt checks fast; (2) makes `cargo test --test profile_validation_tests` run the deterministic tests without snapshot management overhead; (3) profile_golden_file_tests.rs is clearly associated with `cargo insta review` workflow.

**Alternatives considered:**
- Single test file — mixes concerns; harder to run subsets
- Add Profile tests to existing `golden_file_tests.rs` — modifies WI-21 code (forbidden by AR)
- Subdirectory structure (`tests/profile/`) — unnecessary given small file count

---

### D-5: Edge Case for --include and --exclude Together

**Decision:** Test verifies error assertion — both flags together produces a `--include and --exclude are mutually exclusive` error with non-zero exit code.

**Rationale:** Per spec clarification Q2, WI-30's implementation treats `--include` and `--exclude` as mutually exclusive via clap's `conflicts_with`. The test invokes the CLI and asserts the error message and exit code. This is NOT a golden-file scenario.

**Alternatives considered:**
- Combined-selection golden file — impossible; WI-30 returns error for this combination
- Skip test — loses EC-6 / S-3 coverage

---

### D-6: FR-000 Semantic Validation Scope

**Decision:** Do NOT extend `SemanticValidator` for Profile in this WI. Schema validation only (`validate_artifact` with `OscalModelType::Profile`).

**Rationale:** WI-32 is test-only. Profile semantic validation (e.g., cross-referencing imported control IDs against actual catalog) requires significant new production code and is explicitly out of scope (AR Option 2 was rejected; Option 1 is schema + golden files only). The `SemanticValidator` in `src/validate/semantic.rs` is Catalog/Component-specific and should not be modified.

**Alternatives considered:**
- Add Profile semantic validation — over-engineering; AR Option 2 rejected for exactly this reason
- Return empty semantic errors for Profile in SemanticValidator — unnecessary; `validate_artifact` (not `run_full_validation`) is used for profile tests

---

### D-7: Profile Test Fixture Catalog Structure

**Decision:** Use an inline minimal Rust `&str` constant for the test catalog JSON in profile tests — no separate fixture file.

**Rationale:** Profile tests do NOT parse or read the catalog file content — `build_profile()` only takes the catalog path string and uses it as the `href` in the Profile `imports` section. Any existing file path satisfies the "catalog exists" check. A single `tempfile::NamedTempFile` with minimal valid JSON content suffices for all tests.

**Alternatives considered:**
- Dedicated fixture files in `tests/fixtures/profiles/` — unnecessary; catalog content is not consumed by WI-30 profile generation
- Reuse existing `tests/fixtures/golden/small/expected-catalog.json` — works but heavyweight; inline temp file is simpler

---

### D-8: validate_artifact vs run_full_validation for Profile Tests

**Decision:** Use `validate_artifact(json, OscalModelType::Profile)` for profile schema validation tests, NOT `run_full_validation()`.

**Rationale:** `run_full_validation()` invokes `SemanticValidator` which is Catalog/Component-specific (D-6). For Profile validation, only schema validation is needed. `validate_artifact()` is the correct function — it loads the embedded schema and validates without semantic checks. `ValidationResult.is_valid` and `ValidationResult.errors` provide all needed assertions.

---

## Summary of Open Questions Resolved

| Question | Answer |
|----------|--------|
| WI-31 implemented? | No — parameter tailoring tests are `#[ignore]` stubs |
| OSCAL Profile schema URL | `https://github.com/usnistgov/OSCAL/releases/download/v1.2.0/oscal_profile_schema.json` |
| Normalization utility location | `tests/common/mod.rs` (new addition) |
| Schema validation function | `validate_artifact()` not `run_full_validation()` |
| Test file structure | Two new files: `profile_validation_tests.rs`, `profile_golden_file_tests.rs` |
| Golden-file framework | `insta::assert_json_snapshot!()` — same as WI-21 |
| EC-6/S-3 behavior | Error assertion (mutually exclusive), not golden-file |
| Semantic validation for Profile | Not in scope (D-6) |

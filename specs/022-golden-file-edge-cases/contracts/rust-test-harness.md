# Rust Test Harness Contract: Golden File Edge Cases (WI-22)

This contract defines the WI-22 integration test helper surface for `tests/golden_edge_case_tests.rs`.
It is implementation-facing and intentionally aligned to Rust/CLI testing patterns used by WI-21.

---

## Helper Inputs

```rust
pub enum StrategyMode {
    Catalog,
    Component,
    Agnostic,
}

pub struct EdgeCaseFixture {
    pub id: &'static str,          // ec01-no-headings, ec02-compound-atomic, etc.
    pub fixture_dir: &'static str, // tests/fixtures/edge-cases/<id>/
    pub strategy_mode: StrategyMode,
}
```

---

## Required Helper Functions

```rust
/// Run conversion for one fixture and strategy, returning parsed JSON output plus stderr.
fn run_fixture_convert(fixture: &EdgeCaseFixture, strategy: StrategyMode) -> Result<ConvertResult, TestHarnessError>;

/// Assert failure semantics using required substrings (cause, offending input/path, remediation hint).
fn assert_edge_case_error(stderr: &str, expected_substrings: &[String]) -> Result<(), TestHarnessError>;

/// Assert warnings by substring list from expected-warnings.txt.
fn assert_expected_warnings(stderr: &str, expected_substrings: &[String]) -> Result<(), TestHarnessError>;

/// Extract stable IDs from output for pairwise comparison.
fn extract_stable_ids(output: &serde_json::Value) -> Vec<String>;

/// Assert ID equality for whitespace-only variants (EC-5).
fn assert_stable_ids_match(lhs: &[String], rhs: &[String]) -> Result<(), TestHarnessError>;

/// Assert changed IDs for non-whitespace requirement edits (EC-6).
fn assert_stable_ids_differ(lhs: &[String], rhs: &[String]) -> Result<(), TestHarnessError>;

/// Assert all expected validation issue categories are present (EC-10).
fn assert_all_issue_categories(stderr: &str, expected_issue_substrings: &[String]) -> Result<(), TestHarnessError>;
```

---

## Required Behavioral Invariants

1. Failure assertions MUST be substring-based and MUST NOT require full-string equality.
2. Metadata default assertions MUST validate:
   - `title = input filename stem`
   - `version = "0.0.0"`
   - `author = "Unknown"`
   - one warning per missing field.
3. EC-1/2/3/4/5/6/7/10 MUST run under both strategies.
4. EC-9 MUST run once as strategy-agnostic.
5. Malformed citations MUST retain back-matter `prop name="url-status" value="unvalidated"`.

---

## Output Contract Notes

- Expected JSON artifacts are fixture-local and strategy-specific where applicable.
- Expected warning/error files hold required substrings, not full messages.
- Snapshot filenames should be deterministic and explicitly named in test code (no wildcard-only task targeting).

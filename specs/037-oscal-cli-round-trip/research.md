# Research: oscal-cli Round-Trip Validation (WI-37)

**Branch**: `037-oscal-cli-round-trip` | **Date**: 2026-03-12

All questions were fully resolved from existing codebase analysis, the clarification session (5 questions), and the AR-037/SEC-037 documents. No external research was required.

---

## Finding 1: oscal-cli `convert` Command Syntax

**Question**: What is the correct oscal-cli command to convert between OSCAL formats?

**Decision**: `oscal-cli convert --to=<format> <input-file> <output-file>`

Where `<format>` is one of: `json`, `xml`, `yaml`.

**Rationale**: Confirmed by the oscal-cli documentation referenced in AR-037 and by the WI-36 integration pattern in `src/oscal_cli/invoker.rs`. The `--to=` flag specifies the output format. Input format is auto-detected from the file extension.

**Alternatives considered**: `oscal-cli convert --to <format>` (space-separated) — oscal-cli uses the `=` form; the AR-036 code uses `["profile", "resolve", "-to=json"]` pattern confirming this style.

---

## Finding 2: JSON Comparison Algorithm

**Question**: Should `assert_json_diff` crate be used, or a custom `serde_json::Value` tree walker?

**Decision**: Custom recursive `serde_json::Value` tree walker. No `assert_json_diff` dependency.

**Rationale**: The clarification session (Q3) confirmed this. The OSCAL-specific unordered-array rules (`props`, `links`, `parts`) require custom matching logic (identity key lookup by `uuid` or `name`) that a generic diff crate cannot express without wrapping. A purpose-built walker is ~50 lines, is directly unit-testable, and adds no dependency.

The existing `src/testing/semantic_eq.rs` module provides a solid reference for the recursive comparison pattern. The new `round_trip/comparator.rs` will extend this with:
- Path-aware unordered array matching (for `props`, `links`, `parts`)
- `Divergence` output type (not `EquivalenceDiff`) for structured classification

**Alternatives considered**: `assert_json_diff` (MIT) — rejected per clarification Q3; cannot express OSCAL unordered-array identity keys without wrapper overhead.

---

## Finding 3: Unordered OSCAL Array Fields

**Question**: Which OSCAL array fields must be compared without regard to element order?

**Decision**: `props`, `links`, `parts` (confirmed in clarification Q2).

**Rationale**: These are the fields most commonly reordered by oscal-cli during format conversion. OSCAL specification does not mandate ordering for these arrays. Controls, groups, and parameters are ordered by definition and must be compared positionally.

**Element identity resolution** (for unordered matching):
1. If element has `uuid` field → use `uuid` as identity key
2. If element has `name` field (props) → use `name` + `ns` (namespace, optional) as composite key
3. Otherwise → fall back to positional comparison (conservative, avoids false "acceptable" classification)

**Alternatives considered**: "All arrays unordered" — rejected; control ordering within groups is semantically significant in OSCAL. "No unordered exceptions" — rejected; produces false positives on every prop reordering, defeating the purpose of semantic comparison.

---

## Finding 4: Divergence Log Format and Location

**Decision**: JSON file written to a configurable path; default `divergences.json` in the working directory (confirmed in clarification Q1).

**Rationale**: Machine-readable; enables C-2 (automated tracking, deferred to issue #64); `serde_json` is already a production dependency. The `RoundTripResult` struct derives `Serialize`, so the log writer is a thin `serde_json::to_writer_pretty` call.

**Log schema** (matches `RoundTripResult`):
```json
{
  "artifact_type": "Catalog",
  "source_path": "/tmp/xxx/catalog.json",
  "passed": false,
  "divergences": [
    {
      "json_path": "/catalog/metadata/title",
      "expected": "\"My Policy\"",
      "actual": "\"my-policy\"",
      "classification": "ForgeFix",
      "description": "Title case not preserved through XML round-trip"
    }
  ]
}
```

**Alternatives considered**: Stdout only (Option A) — rejected; not persisted for audit trail. Markdown file (Option C) — rejected; not machine-readable. Both JSON + Markdown (Option D) — rejected; unnecessary duplication.

---

## Finding 5: Subprocess Timeout

**Decision**: 30 seconds per `oscal-cli convert` invocation (confirmed in clarification Q4).

**Rationale**: JVM cold-start is typically 2–5 seconds on modern hardware. 30 seconds gives substantial headroom for slow CI runners without letting a hung process block indefinitely. Each invocation gets its own independent 30-second timeout (not shared across the 3-step chain) per SEC-6.

The existing `ProcessInvoker` in `src/oscal_cli/invoker.rs` already implements a poll-based timeout via `child.try_wait()` — the new `convert` implementation reuses this exact pattern.

**Timeout behavior**: A timeout is a **hard error** (`ForgeError::OscalCliTimeout`), not a graceful skip. Only "oscal-cli not found" is a graceful skip.

**Alternatives considered**: 60 seconds — unnecessarily long; 10 seconds — too short for CI on resource-constrained runners; configurable via env var — deferred (not needed for this WI).

---

## Finding 6: Existing Infrastructure Reuse

**What already exists** (no re-implementation needed):

| Component | Location | Reuse |
|-----------|----------|-------|
| `OscalCliDetect` trait | `src/oscal_cli/mod.rs` | Used as-is for availability detection |
| `PathDetector` | `src/oscal_cli/detector.rs` | Used as-is in integration tests |
| `ProcessInvoker` | `src/oscal_cli/invoker.rs` | Extended with `convert()` implementation |
| `OscalCliInfo` | `src/oscal_cli/mod.rs` | Used as-is (available, functional, executable_path) |
| `ForgeError` variants | `src/error.rs` | `OscalCliExecution`, `OscalCliTimeout` reused; `Io` variant for file errors |
| `sanitize::strip_control_chars` | `src/sanitize.rs` | Reused in convert() stderr sanitization |
| `tempfile::TempDir` | dev-dep | Already present in `Cargo.toml` |

**What needs to be added** to `src/oscal_cli/mod.rs`:
```rust
pub enum OscalFormat { Json, Xml, Yaml }

pub struct ConvertArgs {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub output_format: OscalFormat,
    pub timeout: Duration,
}

pub struct ConvertResult {
    pub output_path: PathBuf,
    pub warnings: Vec<String>,
}

pub trait OscalCliInvoke {
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError>;
    fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError>; // NEW
}
```

---

## Finding 7: `OscalCliInvoke` Trait Extension Strategy

**Decision**: Add `convert` method to the existing `OscalCliInvoke` trait.

**Rationale**: The trait is already the abstraction boundary for subprocess invocations. Adding `convert` keeps all oscal-cli operations under one trait, making it easy to mock in unit tests. The trait has a single concrete implementation (`ProcessInvoker`) and is used in a handful of call sites.

**Impact on existing code**: Any existing mock implementations of `OscalCliInvoke` (in tests for WI-36) will need to add a stub `convert()` implementation. Audit needed: search for `impl OscalCliInvoke` in test files.

**Alternatives considered**: Separate `OscalCliConvert` trait — rejected; unnecessary fragmentation for what will be tested together.

# Implementation Plan: 030-prd-profile-generation

**Branch**: `030-prd-profile-generation` | **Date**: 2026-02-17 | **Spec**: [spec.md](./spec.md)
**Input**: docs/PRD/030-prd-profile-generation.md · docs/AR/030-ar-profile-generation.md · docs/SEC/030-sec-profile-generation.md

---

## Summary

Add a `forge profile` subcommand to FORGE that accepts `--catalog <path>`, `--include <ids>` or `--exclude <ids>` (mutually exclusive), `--format json`, and `--output <path>`. Generates a valid OSCAL v1.2.0 Profile JSON with an `imports[]` array referencing the source Catalog. The Profile `uuid` and metadata are generated via WI-11's `assemble_metadata`. No catalog reading, no parameter tailoring, no profile resolution — those are WI-32, WI-31, and NIST oscal-cli concerns respectively.

**Architecture Decision**: Option 1 — Direct Struct Building (selected in AR-030).

---

## Technical Context

**Language/Version**: Rust, edition 2024, stable 1.93.0
**Primary Dependencies**: clap 4.x (derive), serde 1.0.228, serde_json 1.0.149, uuid 1.20.0, chrono 0.4, thiserror 2.0.18 — all already in `Cargo.toml`; **no new dependencies required**
**Storage**: N/A — reads/writes local files only
**Testing**: `cargo test` (unit + integration); target >90% coverage for new code
**Target Platform**: Local CLI (Linux, macOS, Windows)
**Project Type**: Single Rust crate
**Performance Goals**: Profile generation is a trivial struct construction + JSON serialization; no performance target beyond sub-second response
**Constraints**: `cargo clippy -- -D warnings` must pass; `cargo fmt --check` must pass
**Scale/Scope**: ~150–200 LOC new code; 2 new source files; additions to 3 existing files

---

## Constitution Check

*GATE: Must pass before implementation.*

| Principle | Check | Status |
|-----------|-------|--------|
| I. Crate-First | Single crate; new logic in `src/oscal/profile.rs` (library) + `src/cli/profile.rs` (thin dispatcher). Business logic separated from CLI dispatch. | ✅ PASS |
| II. Rust-First | Pure Rust; no FFI, no unsafe, no new external tool dependencies | ✅ PASS |
| III. Contract-First | Structs + function signatures defined in `contracts/` before implementation | ✅ PASS |
| IV. TDD | Tests written before or alongside implementation; target >90% for new code | ✅ PASS (plan requires TDD order) |
| V. Error Handling | `thiserror` ForgeError; no silent failures; actionable messages | ✅ PASS |
| YAGNI | No builder pattern (Option 2), no template engine (Option 3), no modify section (WI-31), no multi-catalog (future) | ✅ PASS |
| DRY | Reuses WI-11 `assemble_metadata`; reuses export file-writing pattern | ✅ PASS |

**Complexity Tracking**: No violations requiring justification.

---

## Project Structure

### Documentation (this feature)

```text
specs/030-prd-profile-generation/
├── plan.md              # This file
├── spec.md              # Feature specification (derived from PRD/AR/SEC)
├── research.md          # Phase 0 research (all NEEDS CLARIFICATION resolved)
├── data-model.md        # Phase 1 data model
├── contracts/
│   ├── profile_types.rs # Type + function interface contract
│   └── cli_profile.rs   # CLI subcommand contract
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code Changes (repository root)

```text
src/
├── oscal/
│   ├── mod.rs           # MODIFY: add `pub mod profile;` and re-exports
│   └── profile.rs       # CREATE: OscalProfile, ProfileImport, ControlSelection,
│                        #         ProfileRoot, SelectionMode, build_profile,
│                        #         parse_control_ids
├── cli/
│   ├── mod.rs           # MODIFY: add Profile variant to Commands, dispatch in execute()
│   └── profile.rs       # CREATE: execute() handler for forge profile subcommand

tests/
└── profile_generation_test.rs  # CREATE: integration tests for forge profile
```

**Structure Decision**: Single-crate with new modules in `src/oscal/` (library logic) and `src/cli/` (CLI dispatch). Parallel to existing `catalog.rs`, `component_definition.rs`, `export.rs`, `validate.rs` patterns.

---

## Phase 0: Research

**Status**: ✅ Complete — see [research.md](./research.md)

Key resolved decisions:
- `assemble_metadata` takes `&DocumentMetadata`, not a bare string → create `DocumentMetadata { title: "Policy Baseline Profile", version: "1.0.0", ..Default::default() }`
- Single crate (not workspace) — add modules to existing `src/` structure
- CLI variant is inline in `Commands` enum (not a separate Args struct)
- `ProfileRoot { profile: OscalProfile }` wrapper for OSCAL root key convention
- `parse_control_ids`: split on `,`, trim, remove empty, dedup (order-preserving), error if empty
- Catalog existence checked in `cli/profile.rs`; catalog content NOT parsed (WI-32 scope)
- Profile title: `"Policy Baseline Profile"` (hardcoded default for WI-30)
- Profile structs in `src/oscal/profile.rs`, CLI handler in `src/cli/profile.rs`

---

## Phase 1: Design & Contracts

**Status**: ✅ Complete

### Data Model

See [data-model.md](./data-model.md) for full ER diagram and field-level documentation.

**Core structs:**

```rust
ProfileRoot { profile: OscalProfile }
OscalProfile { uuid: Uuid, metadata: OscalMetadata, imports: Vec<ProfileImport> }
ProfileImport { href: String, include_controls: Option<Vec<ControlSelection>>,
                exclude_controls: Option<Vec<ControlSelection>> }
ControlSelection { with_ids: Vec<String> }  // serde rename: "with-ids"
SelectionMode { Include | Exclude }
```

### API Contracts

See [contracts/profile_types.rs](./contracts/profile_types.rs) and [contracts/cli_profile.rs](./contracts/cli_profile.rs).

Key function signatures:
```rust
pub fn build_profile(catalog_path: &str, control_ids: Vec<String>, mode: SelectionMode)
    -> Result<OscalProfile, ForgeError>

pub fn parse_control_ids(raw: &str) -> Result<Vec<String>, ForgeError>

// In cli/profile.rs:
pub fn execute(catalog: &PathBuf, include: Option<&str>, exclude: Option<&str>,
               format: &OutputFormat, output: Option<&Path>)
    -> Result<(), ForgeError>
```

---

## Phase 2: Implementation Order

> **Note**: `/speckit.tasks` generates `tasks.md` with granular work items. This section defines the implementation phases and ordering guardrails.

### Guardrails (from AR-030)

- ❌ DO NOT read or parse the source Catalog file
- ❌ DO NOT generate a `modify` section (WI-31)
- ❌ DO NOT implement Profile Resolution (NIST oscal-cli)
- ❌ DO NOT embed control content in the Profile
- ✅ MUST make `--include` and `--exclude` mutually exclusive (clap `conflicts_with`)
- ✅ MUST wrap output in `{"profile": {...}}` root object
- ✅ MUST reuse `assemble_metadata` from WI-11
- ✅ MUST use catalog path as-is for `href` (no normalization)

### Implementation Sequence

**Step 1 — Type definitions (TDD RED):**
Write tests for Profile struct serialization first, then implement:
1. `src/oscal/profile.rs`: `ProfileRoot`, `OscalProfile`, `ProfileImport`, `ControlSelection`, `SelectionMode`
2. `src/oscal/mod.rs`: add `pub mod profile;` and re-export public items

**Step 2 — Core logic (TDD RED → GREEN):**
Write unit tests for `parse_control_ids` (trim, dedup, empty-error), then implement.
Write unit tests for `build_profile` (include path, exclude path, metadata fields), then implement.

**Step 3 — CLI integration (TDD RED → GREEN):**
Write integration tests for CLI dispatch first, then:
1. `src/cli/profile.rs`: `execute()` function with catalog-exists check, output writing
2. `src/cli/mod.rs`: add `Profile { ... }` variant to `Commands`, dispatch in `execute()`

**Step 4 — Integration tests:**
1. `tests/profile_generation_test.rs`: full CLI invocation tests (include, exclude, file output, mutual exclusivity, missing catalog)

**Step 5 — Verification:**
```bash
cargo test                   # All tests pass
cargo clippy -- -D warnings  # Zero warnings
cargo fmt --check            # Formatting clean
```

### Testing Matrix

| Layer | Test | Location |
|-------|------|----------|
| Unit | `ProfileRoot` serializes with `"profile"` root key | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `OscalProfile` with `include-controls` → correct JSON shape | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `OscalProfile` with `exclude-controls` → correct JSON shape | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `ProfileImport.href` matches catalog path | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `build_profile` sets metadata title, oscal-version | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `parse_control_ids` trims whitespace | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `parse_control_ids` deduplicates (order-preserving) | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `parse_control_ids` errors on empty string | `src/oscal/profile.rs` #[cfg(test)] |
| Unit | `parse_control_ids` single ID (no comma) | `src/oscal/profile.rs` #[cfg(test)] |
| Integration | `forge profile --include` happy path | `tests/profile_generation_test.rs` |
| Integration | `forge profile --exclude` happy path | `tests/profile_generation_test.rs` |
| Integration | `forge profile --include --exclude` → clap conflict error | `tests/profile_generation_test.rs` |
| Integration | `forge profile` no selection flags → error | `tests/profile_generation_test.rs` |
| Integration | `forge profile --catalog missing.json` → Io error | `tests/profile_generation_test.rs` |
| Integration | `forge profile --output <path>` → file created | `tests/profile_generation_test.rs` |
| Security | `href` stored as-is (no normalization) | `src/oscal/profile.rs` #[cfg(test)] |
| Security | No catalog content in output JSON | `src/oscal/profile.rs` #[cfg(test)] |

---

## Security Requirements Verification

| SEC ID | Requirement | Verification |
|--------|-------------|--------------|
| SEC-1 | No policy text in Profile — only ID references and metadata | Unit test: serialized JSON contains only `uuid`, `metadata`, `imports[].href`, `imports[].*.with-ids` |
| SEC-2 | `--include` / `--exclude` mutual exclusivity | Integration test: providing both → clap error |
| SEC-3 | Empty control ID → descriptive error | Unit test: `parse_control_ids("")` → `Err(ForgeError::InvalidArgument)` |
| SEC-4 | Catalog path stored as-is | Unit test: `build_profile("/absolute/path/catalog.json", ...)` → `href == "/absolute/path/catalog.json"` |
| SEC-5 | OSCAL v1.2.0 Profile JSON shape | Unit test: assert `imports[0]` structure, `oscal-version == "1.2.0"` |

---

## Post-Design Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First | ✅ PASS | `src/oscal/profile.rs` = library; `src/cli/profile.rs` = thin dispatcher |
| II. Rust-First | ✅ PASS | No new dependencies |
| III. Contract-First | ✅ PASS | Contracts written before implementation begins |
| IV. TDD | ✅ PASS | Tests written first per Step sequence |
| YAGNI | ✅ PASS | No builder, no template engine, no modify section |

**No violations. Implementation is cleared to proceed.**

---

## Quickstart for Implementation

```bash
# 1. Verify you're on the right branch
git checkout 030-prd-profile-generation

# 2. Start with TDD — write tests first, then implement
# See Phase 2 Implementation Sequence above

# 3. After implementation, verify
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# 4. Manual smoke test
echo '{"catalog": {"uuid": "test", "metadata": {"title": "T", "last-modified": "2026-01-01T00:00:00Z", "version": "1.0", "oscal-version": "1.2.0"}, "groups": []}}' > /tmp/test-catalog.json
cargo run -- profile --catalog /tmp/test-catalog.json --include POL-AC-001,POL-AC-002
```

---

## Artifact Summary

| Artifact | Path | Status |
|---------|------|--------|
| Feature Spec | specs/030-prd-profile-generation/spec.md | ✅ |
| Research | specs/030-prd-profile-generation/research.md | ✅ |
| Data Model | specs/030-prd-profile-generation/data-model.md | ✅ |
| Type Contract | specs/030-prd-profile-generation/contracts/profile_types.rs | ✅ |
| CLI Contract | specs/030-prd-profile-generation/contracts/cli_profile.rs | ✅ |
| Tasks | specs/030-prd-profile-generation/tasks.md | ✅ |

# Research: YAML Output (WI-27)

**Date**: 2026-02-15
**Status**: Complete — all items resolved

## Research Tasks

### R-1: serde_yaml_ng Compatibility with OSCAL Model Structs

**Decision**: `serde_yaml_ng` 0.10 (aliased as `serde_yaml`) is fully compatible.

**Rationale**: The crate is already a production dependency in `Cargo.toml` (used for YAML frontmatter parsing in `src/model/frontmatter.rs:66`). It implements the standard serde `Serializer` trait, meaning any type that derives `serde::Serialize` can be serialized to YAML via `serde_yaml::to_string()`. All OSCAL model structs (`CatalogEnvelope`, `ComponentDefinitionEnvelope`, etc.) derive `Serialize`.

**Alternatives considered**: `yaml-rust2` (manual serialization — rejected per AR Option 2), custom writer (rejected per AR Option 3), original `serde_yaml` crate (unmaintained, `serde_yaml_ng` is the maintained successor).

### R-2: serde_json::Value Serialization to YAML

**Decision**: `serde_json::Value` serializes correctly to YAML via `serde_yaml::to_string()`.

**Rationale**: `DocumentaryComponent.control_implementations` is `Vec<serde_json::Value>`. Both `serde_json::Value` and `serde_yaml` operate on the serde data model. `serde_json::Value` implements `Serialize`, so it produces valid YAML output. JSON objects become YAML mappings, arrays become sequences, strings become scalars. This is a fundamental serde interoperability guarantee.

**Alternatives considered**: Converting `serde_json::Value` to `serde_yaml::Value` first (unnecessary — serde handles the conversion transparently).

### R-3: Schema Validation Strategy for Non-JSON Output

**Decision**: Validate via `serde_json::Value` regardless of output format; serialize to requested format after validation passes.

**Rationale**: OSCAL schemas are JSON Schema. The `jsonschema` crate validates against `serde_json::Value`. For YAML output: (1) serialize model → `serde_json::Value` via `serde_json::to_value()`, (2) validate, (3) if valid, serialize model → YAML string via `serde_yaml::to_string()`. This eliminates the current JSON string → Value round-trip and makes validation format-independent.

**Alternatives considered**: Validating YAML string by parsing to `serde_json::Value` (adds unnecessary conversion step), skipping validation for YAML (unacceptable — PRD M-6 requires valid output).

### R-4: Semantic Equivalence Testing Approach

**Decision**: Compare via `serde_json::Value` — serialize model to JSON, serialize model to YAML, parse both to `serde_json::Value`, assert equality.

**Rationale**: OSCAL model structs derive `Serialize` but NOT `Deserialize` or `PartialEq`. Adding these derives would require changes to all model structs plus handling `serde_json::Value` fields. Instead, comparing via `serde_json::Value` is non-invasive: `serde_json::to_value(&model)` produces a Value from JSON serialization, and `serde_yaml::from_str::<serde_json::Value>(&yaml_str)` produces a Value from YAML. If both Values are equal, semantic equivalence is verified.

**Alternatives considered**: Adding `Deserialize` + `PartialEq` to all model structs (too invasive, risks `serde_json::Value` field issues), byte-level string comparison (rejected — formatting differences between JSON and YAML are expected).

### R-5: YAML Special Character Handling (SEC-2)

**Decision**: `serde_yaml_ng` handles quoting automatically for strings containing YAML-special characters.

**Rationale**: When serializing a Rust `String` to YAML, `serde_yaml` outputs the value as a YAML string scalar with appropriate quoting. Characters like `:`, `#`, `[`, `]`, `{`, `}`, `---`, `...` are handled by the emitter. Boolean-like words (`yes`, `no`, `true`, `false`) are quoted when they come from a Rust `String` type because serde preserves the type distinction. Must verify with unit tests (SEC-2, SEC-3).

**Alternatives considered**: Manual post-processing of YAML output (violates AR guardrail — no custom formatting), adding `#[serde(serialize_with)]` attributes (rejected — model must remain format-agnostic).

### R-6: YAML Type Tag Safety (SEC-1)

**Decision**: `serde_yaml_ng` does not emit YAML type tags (`!!tag`) when serializing standard Rust types.

**Rationale**: `serde_yaml` maps Rust types to standard YAML nodes: `String` → plain/quoted scalar, `Vec<T>` → sequence, struct → mapping, `Option<T>` → value or null. It does NOT emit language-specific tags like `!!python/exec` or `!!ruby/object`. Only custom serde serializers producing tagged values would trigger this, and FORGE uses standard derives. Must verify with negative test (SEC-1).

**Alternatives considered**: Post-processing scan for `!!` patterns (belt-and-suspenders — add as test assertion, not runtime filter).

### R-7: `forge export` Status

**Decision**: Defer M-4 (`--format yaml` on `forge export`) to WI-29.

**Rationale**: The `forge export` subcommand does not exist in the current codebase. The CLI only has `Convert` and `Validate` commands. PRD scope explicitly states: "forge export subcommand implementation — deferred to WI-29." The YAML serializer module (`src/export/yaml.rs`) built in this WI will be directly reusable when `forge export` is implemented.

**Alternatives considered**: Implementing a stub `forge export` command (scope creep — violates PRD boundaries).

### R-8: `write_output` Function Naming

**Decision**: Rename the `json` parameter in `write_output` to `content` for format-agnosticism.

**Rationale**: The `write_output` function in `pipeline.rs:21` takes a `json: &str` parameter but is actually format-agnostic (just writes a string). Renaming the parameter to `content` makes it accurate for both JSON and YAML output without changing behavior.

**Alternatives considered**: Creating a separate `write_yaml_output` function (unnecessary duplication — the function is already format-agnostic).

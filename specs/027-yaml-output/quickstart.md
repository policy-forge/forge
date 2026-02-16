# Quickstart: YAML Output (WI-27)

**Date**: 2026-02-15

## Prerequisites

- Rust 1.93.0+ (stable)
- All Phase 1 tests passing: `cargo test`
- Branch: `027-yaml-output`

## Implementation Order

Follow this order strictly (TDD: tests first, then implementation):

### Step 1: Create YAML Serializer Module

**Files**: `src/export/yaml.rs`, `src/export/mod.rs`

1. Write unit tests in `tests/yaml_serializer_test.rs` (RED)
2. Create `src/export/yaml.rs` with `serialize_to_yaml` and `deserialize_from_yaml`
3. Update `src/export/mod.rs` to declare the `yaml` submodule
4. Run tests (GREEN)

### Step 2: Refactor Pipeline for Format Dispatch

**Files**: `src/pipeline.rs`, `src/cli/convert.rs`

1. Write integration tests in `tests/yaml_equivalence_test.rs` (RED)
2. Add `format: &OutputFormat` parameter to `run_catalog_pipeline` and `run_component_pipeline`
3. Replace `serde_json::to_string_pretty` + `serde_json::from_str` with `serde_json::to_value` for validation
4. Add format dispatch after validation: JSON → `serde_json::to_string_pretty`, YAML → `serialize_to_yaml`
5. Remove the non-JSON guard in `src/cli/convert.rs:24`
6. Pass format from CLI to pipeline
7. Run tests (GREEN)

### Step 3: Security Tests

**Files**: `tests/yaml_security_test.rs`

1. Write SEC-1 test: no `!!` type tags in YAML output
2. Write SEC-2 test: YAML-special characters properly quoted
3. Write SEC-3 test: policy text serialized as string scalars
4. Write SEC-4 test: semantic equivalence with adversarial input
5. All tests pass (GREEN)

### Step 4: CLI Integration Tests

**Files**: `tests/cli_integration.rs`

1. Add `--format yaml` tests for catalog and component strategies
2. Add stdout and file output tests for YAML
3. Verify YAML output parses back to valid data

### Step 5: Verify

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Key Constraints (from AR Guardrails)

- **DO NOT** write custom YAML formatting logic — use `serde_yaml::to_string()` only
- **DO NOT** add YAML-specific `#[serde]` attributes to OSCAL model structs
- **DO NOT** verify semantic equivalence via string comparison — compare deserialized `serde_json::Value` structs
- **DO NOT** implement round-trip testing — deferred to WI-28
- **MUST** handle all `serde_yaml` errors with `ForgeError::Serialization`
- **MUST** validate via `serde_json::Value` before serializing to any output format

## Quick Verification

After implementation, verify manually:

```bash
# Convert sample policy to YAML
cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format yaml

# Convert to YAML file
cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format yaml --output /tmp/catalog.yaml

# Verify YAML is valid (parse with Python)
python3 -c "import yaml; yaml.safe_load(open('/tmp/catalog.yaml'))"

# Compare JSON and YAML structurally
cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format json --output /tmp/catalog.json
python3 -c "
import json, yaml
j = json.load(open('/tmp/catalog.json'))
y = yaml.safe_load(open('/tmp/catalog.yaml'))
assert j == y, 'Semantic equivalence failed'
print('Semantic equivalence verified')
"
```

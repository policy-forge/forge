# Quickstart: WI-34 Parameter Extraction

**Branch**: `034-prd-parameter-extraction`

---

## Prerequisites

```bash
# Verify Rust version (1.93.0+ required for std::sync::LazyLock)
rustc --version  # should be >= 1.93.0

# Verify project builds cleanly before starting
cargo build
cargo test
```

---

## TDD Workflow

This feature uses strict TDD. Write tests BEFORE implementation. Each task below follows Red → Green → Refactor.

### Phase A: Domain Model (no logic, just types)

```bash
# 1. Add types to src/model/mod.rs (PolicyParameter, ParameterType, etc.)
# 2. Add `parameters: Vec<PolicyParameter>` to PolicyRequirement
# 3. Fix all test compile errors (add `parameters: vec![]` to constructors)
cargo test --lib 2>&1 | head -30
# Expected: compile errors until fields added, then test failures on changed constructors

# After fixing: should compile and pass
cargo test --lib
```

### Phase B: Matchers (TDD per matcher)

```bash
# For each matcher (TimeWindow → Threshold → Frequency → Quantity):
# 1. Write test in src/parameter/matchers.rs
# 2. Run — MUST FAIL (Red)
cargo test parameter::matchers::tests::time_window  # expected: fail (no impl)
# 3. Implement the matcher
# 4. Run — MUST PASS (Green)
cargo test parameter::matchers::tests::time_window  # expected: pass
# 5. Refactor if needed
```

### Phase C: Extraction Orchestration

```bash
# Write extract_parameters_from_text tests first
cargo test parameter::tests::extract_from_text  # Red
# Implement
cargo test parameter::tests  # Green

# Idempotence test (required by SEC-6)
cargo test parameter::tests::idempotent
```

### Phase D: OSCAL Integration

```bash
# Write to_oscal_param tests
cargo test parameter::tests::to_oscal_param  # Red → implement → Green

# Build catalog with params
cargo test oscal::catalog::tests::control_with_params  # Red → implement → Green

# Pipeline integration
cargo test pipeline  # Red → implement → Green
```

---

## Running Tests

```bash
# All tests
cargo test

# Just parameter module
cargo test parameter

# Just model tests (verify PolicyRequirement compile)
cargo test model::tests

# With logging (useful for debugging)
RUST_LOG=debug cargo test parameter 2>&1 | grep -E "(parameter|PASS|FAIL)"

# Coverage (install once: cargo install cargo-tarpaulin)
cargo tarpaulin --lib --out Html
# target: >90% for src/parameter/ module

# Negative fixture tests (false positive prevention - SEC-2)
cargo test parameter::tests::no_false_positive
cargo test parameter::tests::section_reference_not_extracted
cargo test parameter::tests::standard_number_not_extracted
```

---

## Testing the Feature End-to-End

Create a test fixture and run the full pipeline:

```bash
cat > /tmp/test_params.md << 'EOF'
# Access Control Policy

## Password Requirements

- Passwords must be changed within 30 days of compromise
- Passwords must be minimum 12 characters
- Sessions must timeout after no more than 15 minutes of inactivity

## Training Requirements

- Security training must be completed at least annually
- MFA must require no fewer than 3 authentication factors
EOF

# Run catalog pipeline with parameter extraction
cargo run -- convert /tmp/test_params.md --strategy catalog --format json | \
  python3 -m json.tool | grep -A 20 '"params"'

# Expected: param elements in controls with values and constraints
```

---

## Verify Regex Patterns (ReDoS prevention - SEC-3)

```bash
# Test patterns with adversarial long input
cargo test parameter::tests::redos_adversarial

# Manual: generate a very long string and verify it completes quickly
echo "Generating adversarial input..."
python3 -c "print('a' * 10000 + ' within')" > /tmp/adversarial.txt
# Run extraction on requirement text containing this string:
# Should complete in < 1ms
```

---

## Verify Idempotence (SEC-6)

```bash
# Run extraction twice, verify identical output
cargo test parameter::tests::double_extraction_identical
```

---

## Quality Gates

Before marking tasks complete:

```bash
# Format
cargo fmt --check

# Lint (zero warnings)
cargo clippy -- -D warnings

# All tests
cargo test

# Security audit (no new advisories)
cargo audit

# Documentation builds
cargo doc --no-deps 2>&1 | grep -E "(warning|error)" | head -10
```

---

## Key Patterns to Follow

### Adding a new matcher (future extension)

1. Add static `LazyLock<Regex>` pattern in `src/parameter/matchers.rs`
2. Create a new struct implementing `ParameterMatcher`
3. Add the matcher to the list in `extract_parameters_from_text`
4. Write unit tests for the new matcher
5. No changes to `src/parameter/mod.rs` needed (composable by design)

### Placeholder format

```
{{ insert: param, id-ref: {requirement_id}_prm_{position} }}
```

Example: `{{ insert: param, id-ref: POL-AC-001_prm_0 }}`

The double-brace format is required by OSCAL markup specification. Do not alter.

---

## Troubleshooting

| Problem | Solution |
|---------|---------|
| Test fails: "parameters field missing" | Add `parameters: vec![]` to all `PolicyRequirement` constructors in tests |
| Clippy: "patterns variable not used" | Move `LazyLock` statics to module level, not inside function |
| Test fails: parameter extracted from section ref | Check that qualifier word is required in pattern; add negative test fixture |
| OSCAL schema validation fails | Verify `params` field ordering (before `parts` in JSON output) |
| Double extraction changes text | Check placeholder format does not match any pattern; add idempotence test |

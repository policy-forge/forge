# Quickstart: 028 Multi-Format Round-Trip Testing

## Prerequisites

- Rust stable 1.93.0+ (edition 2024)
- All WI-26 (XML serialization) and WI-27 (YAML serialization) code merged to branch
- Golden test fixtures at `tests/fixtures/golden/{small,medium,complex}/`

## Dev-Dependencies to Add

```toml
# In [dev-dependencies] section of Cargo.toml
assert_json_diff = "4"    # Rich JSON diff assertions
```

## Dependency Modification

```toml
# Enable serde feature on quick-xml for XML deserialization
quick-xml = { version = "0.37", features = ["serde"] }
```

## Key Files to Create/Modify

### New Files
| File | Purpose |
|------|---------|
| `src/testing/mod.rs` | Testing utilities module (public) |
| `src/testing/semantic_eq.rs` | Semantic equivalence comparison |
| `src/export/xml_deserializer.rs` | XML deserialization for Catalog and Component Definition |
| `tests/round_trip_test.rs` | Integration tests for all round-trip paths |

### Modified Files
| File | Change |
|------|--------|
| `src/lib.rs` | Add `pub mod testing;` |
| `src/export/mod.rs` | Add `pub mod xml_deserializer;` and re-exports |
| `Cargo.toml` | Add `assert_json_diff` dev-dep; add `serde` feature to `quick-xml` |
| `src/oscal/parts.rs` | Add XML serde annotations to `OscalProp` (if needed for quick-xml serde) |
| `src/oscal/back_matter.rs` | Add XML serde annotations to `OscalLink` (if needed for quick-xml serde) |

## Build & Test

```bash
# Build with new dependencies
cargo build

# Run all tests (includes new round-trip tests)
cargo test

# Run only round-trip tests
cargo test --test round_trip_test

# Run with verbose output for debugging
cargo test --test round_trip_test -- --nocapture

# Lint check
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```

## Implementation Order

1. Add `assert_json_diff` dev-dep and `serde` feature to `quick-xml` in `Cargo.toml`
2. Create `src/testing/semantic_eq.rs` with `EquivalenceResult`, `EquivalenceDiff`, `assert_semantic_equivalence`
3. Write unit tests for `assert_semantic_equivalence`
4. Create `src/export/xml_deserializer.rs` with `deserialize_catalog_from_xml` and `deserialize_component_from_xml`
5. Write unit tests for XML deserialization
6. Create `tests/round_trip_test.rs` with all round-trip test cases
7. Add YAML type coercion edge-case tests
8. Run full test suite, clippy, fmt

# Quickstart: Component Implemented Requirements (WI-15)

## Prerequisites

- Rust 1.93.0+ (edition 2024)
- All dependencies already in Cargo.toml (no new additions)
- WI-14 complete (component definition builder with empty control-implementations)

## Build & Test

```bash
cargo build                      # Verify clean compilation
cargo test                       # Run all tests (existing + new)
cargo clippy -- -D warnings      # Lint check
cargo fmt --check                # Format check
```

## Implementation Order

### Phase 1: Infrastructure (UUID namespaces + visibility)

1. Compute and add `CONTROL_IMPL_NAMESPACE` and `IMPL_REQ_NAMESPACE` to `src/uuid.rs`
2. Change `resolve_abbreviation` in `src/oscal/catalog.rs` to `pub(crate)`

### Phase 2: Core Builder (TDD — tests first)

3. Create `src/oscal/implemented_requirements.rs` with:
   - `build_control_implementations(&PolicyDocument, &str) -> Result<Value, ForgeError>`
   - `map_requirement_to_implemented(&PolicyRequirement, &str, usize) -> Result<Value, ForgeError>`
   - UUID generation helpers
   - Control-id derivation (reusing catalog utilities)

### Phase 3: Integration

4. Modify `build_component_definition` in `src/oscal/component_definition.rs` to accept `source_profile: Option<&str>` and inject control-implementations
5. Add `--source-profile` flag to CLI in `src/cli/mod.rs`
6. Add `run_component_pipeline` to `src/pipeline.rs`
7. Wire component strategy in `src/cli/convert.rs`

### Phase 4: Edge Cases & Validation

8. Zero requirements warning
9. Empty text placeholder
10. Missing stable_id fallback
11. Empty source-profile error

## Key Files

| File | Action | Purpose |
|------|--------|---------|
| `src/uuid.rs` | MODIFY | Add 2 namespace constants |
| `src/oscal/catalog.rs` | MODIFY | Make `resolve_abbreviation` pub(crate) |
| `src/oscal/implemented_requirements.rs` | NEW | Core WI-15 builder logic |
| `src/oscal/mod.rs` | MODIFY | Re-export new module |
| `src/oscal/component_definition.rs` | MODIFY | Accept source_profile, inject control-implementations |
| `src/cli/mod.rs` | MODIFY | Add --source-profile flag |
| `src/cli/convert.rs` | MODIFY | Validate source-profile, wire component pipeline |
| `src/pipeline.rs` | MODIFY | Add run_component_pipeline |
| `tests/component_pipeline_test.rs` | NEW | Integration test |

## Verification Checklist

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo fmt --check` produces no violations
- [ ] Control-ids match between Catalog and Component for same document
- [ ] UUIDs are deterministic across repeated runs
- [ ] `--source-profile` required for component strategy
- [ ] Empty source-profile produces descriptive error
- [ ] Zero requirements produces empty array + warning
- [ ] Empty requirement text produces placeholder description

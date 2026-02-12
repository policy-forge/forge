# Code Style & Conventions

## Formatting
- `.rustfmt.toml`: edition = 2024, max_width = 100, use_small_heuristics = Max

## Lints (Cargo.toml)
- `clippy::all = "warn"`
- `clippy::pedantic = "warn"`
- `unsafe_code = "warn"`

## Error Handling
- `thiserror` derive macro with `ForgeError` enum in `src/error.rs`
- Public functions return `Result<T, ForgeError>`
- Internal helpers that can't fail return bare values

## Visibility
- `pub` for items exported from the crate (reachable via `forge::`)
- `pub(crate)` for internal helpers shared across modules (e.g. `parse_frontmatter`, `map_sections`, `build_line_starts`)
- Private (`fn`) for module-internal helpers

## Derive Patterns
- Domain model structs: `#[derive(Debug, Clone, Serialize)]` (some add `PartialEq`)
- Internal structs not serialized: `#[derive(Debug, Clone, PartialEq)]`
- Enums: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]` for fieldless enums
- CLI: `#[derive(Parser)]`, `#[derive(Subcommand)]`, `#[derive(ValueEnum)]`
- `#[must_use]` on key public constructors/accessors

## Static Patterns
- `std::sync::LazyLock` for compiled regexes (not `lazy_static` or `once_cell`)

## Documentation
- Doc comments on all public items
- Structured sections: `# Arguments`, `# Returns`, `# Errors`, `# Panics`, `# Examples`, `# Algorithm`
- Doc tests for key public functions

## Testing
- Unit tests: `#[cfg(test)] mod tests` within each module (all 10 source files have them)
- Integration tests: `tests/` directory
  - `cli_integration.rs` — process-based (spawn binary, check stdout/stderr/exit code)
  - `atomize_integration.rs` — atomization with shared helpers from `tests/common/`
  - `pipeline_test.rs` — end-to-end: ingest → parse → extract → assemble
- Shared test helpers: `tests/common/mod.rs` (factory functions: `make_req`, `make_section`, `make_doc`)
- Test fixtures: `tests/fixtures/`
- Benchmarks: `benches/` using criterion 0.8 (`atomize.rs`, `uuid_benchmark.rs`)

## Module Structure
- Each concern in its own directory with `mod.rs`
- Sub-files for distinct responsibilities (e.g. `parse/mod.rs` + `parse/clauses.rs` + `parse/atomize.rs`)
- Re-exports in `lib.rs` for public API surface

## Task Completion Checklist
1. `cargo fmt --check` — Verify formatting
2. `cargo clippy -- -D warnings` — No linter warnings
3. `cargo test` — All tests pass
4. `cargo test --doc` — Doc tests pass

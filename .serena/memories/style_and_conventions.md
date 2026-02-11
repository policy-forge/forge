# Code Style & Conventions

## Formatting
- `.rustfmt.toml`: edition = 2024, max_width = 100, use_small_heuristics = Max

## Lints (Cargo.toml)
- `clippy::all = "warn"`
- `clippy::pedantic = "warn"`
- `unsafe_code = "warn"`

## Patterns
- Error handling: `thiserror` derive macro with `ForgeError` enum
- CLI: `clap` derive macros (`Parser`, `Subcommand`, `ValueEnum`)
- Serialization: `serde` derive `Serialize` on data structs
- Testing: extensive unit tests in `#[cfg(test)] mod tests` within each module
- Integration tests: process-based (spawn binary, check stdout/stderr/exit code)
- Documentation: doc comments with `# Errors`, `# Arguments`, `# Returns`, `# Examples` sections
- Module structure: each concern in its own directory with `mod.rs`

## Task Completion Checklist
1. `cargo fmt --check` — Verify formatting
2. `cargo clippy -- -D warnings` — No linter warnings
3. `cargo test` — All tests pass
4. `cargo test --doc` — Doc tests pass

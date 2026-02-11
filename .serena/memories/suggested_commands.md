# Suggested Commands

## Build
- `cargo build` — Debug build
- `cargo build --release` — Release build
- `cargo run -- convert <file.md>` — Run convert subcommand
- `cargo run -- validate <file>` — Run validate subcommand (stub)

## Testing
- `cargo test` — Run all tests (unit + integration)
- `cargo test --lib` — Run only library unit tests
- `cargo test --doc` — Run documentation tests
- `cargo test <test_name>` — Run a single test by name

## Linting & Formatting
- `cargo fmt` — Format code
- `cargo fmt --check` — Check formatting without modifying
- `cargo clippy` — Run linter
- `cargo clippy -- -D warnings` — Treat all warnings as errors

## Mutation Testing
- `cargo mutants` — Run mutation testing (cargo-mutants must be installed)

## System (macOS / Darwin)
- `git`, `ls`, `cd`, `grep`, `find` — Standard Unix commands available

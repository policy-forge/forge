# Suggested Commands

## Build
- `cargo build` — Debug build
- `cargo build --release` — Release build

## Run CLI
- `cargo run -- convert <file.md>` — Convert Markdown policy to OSCAL JSON (default format)
- `cargo run -- convert <file.md> --format json|xml|yaml` — Specify output format
- `cargo run -- convert <file.md> --output <out.json>` — Write to file instead of stdout
- `cargo run -- convert <file.md> --strategy catalog|component` — Set conversion strategy
- `cargo run -- convert <file.md> --max-size 20` — Override max file size (MB, default 10)
- `cargo run -- -v convert <file.md>` — Verbose output
- `cargo run -- -q convert <file.md>` — Quiet mode
- `cargo run -- validate <file>` — Validate an OSCAL artifact (stub, not yet implemented)

## Testing
- `cargo test` — Run all tests (unit + integration + doc)
- `cargo test --lib` — Run only library unit tests
- `cargo test --doc` — Run documentation tests
- `cargo test <test_name>` — Run a single test by name
- `cargo test --test cli_integration` — Run CLI integration tests only
- `cargo test --test atomize_integration` — Run atomization integration tests only
- `cargo test --test pipeline_test` — Run end-to-end pipeline tests only

## Linting & Formatting
- `cargo fmt` — Format code
- `cargo fmt --check` — Check formatting without modifying
- `cargo clippy` — Run linter
- `cargo clippy -- -D warnings` — Treat all warnings as errors

## Benchmarks
- `cargo bench` — Run all benchmarks (criterion)
- `cargo bench --bench atomize` — Run atomization benchmarks only
- `cargo bench --bench uuid_benchmark` — Run UUID generation benchmarks only

## Mutation Testing
- `cargo mutants` — Run mutation testing (cargo-mutants must be installed)

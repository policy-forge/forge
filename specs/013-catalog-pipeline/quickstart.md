# Quickstart: End-to-End Catalog Pipeline (WI-13)

## Prerequisites

- Rust 1.93.0+ stable (`rustup update stable`)
- All WI-1 through WI-12 code merged and tests passing
- `cargo test` passes on current branch

## Build and Run

```bash
# Build
cargo build

# Convert a Markdown policy to OSCAL Catalog JSON (stdout)
cargo run -- convert policy.md --strategy catalog --format json

# Convert and write to file
cargo run -- convert policy.md --strategy catalog --format json --output catalog.json

# With verbose output
cargo run -- -v convert policy.md --strategy catalog --format json
```

## Test Commands

```bash
# Run all tests (including WI-13 smoke test)
cargo test

# Run only WI-13 smoke test
cargo test catalog_pipeline

# Run CLI integration tests
cargo test cli_integration

# Lint + format check
cargo clippy -- -D warnings
cargo fmt --check
```

## Expected Output Structure

```json
{
  "catalog": {
    "uuid": "xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx",
    "metadata": {
      "title": "Sample Security Policy",
      "last-modified": "2026-02-12T...",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "groups": [
      {
        "id": "access-control",
        "title": "Access Control",
        "controls": [
          {
            "id": "POL-AC-001",
            "uuid": "...",
            "title": "All users must authenticate...",
            "parts": [{"id": "POL-AC-001_smt", "name": "statement", "prose": "..."}]
          }
        ]
      }
    ]
  }
}
```

## Key Files

| File | Purpose |
|------|---------|
| `src/pipeline.rs` | Pipeline orchestrator (run_catalog_pipeline, write_output) |
| `src/cli/convert.rs` | CLI handler (dispatches to pipeline) |
| `src/cli/mod.rs` | CLI flag definitions (--strategy, --format, --output) |
| `src/error.rs` | Error types (including new Serialization variant) |
| `tests/catalog_pipeline_test.rs` | End-to-end smoke test |
| `tests/fixtures/full_policy.md` | Test fixture for smoke test |

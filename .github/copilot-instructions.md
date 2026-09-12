# Copilot Instructions for FORGE

## Project Overview

**FORGE** (Framework for OSCAL Risk & Governance Execution) is a Rust CLI tool that converts Markdown security policy documents into OSCAL (Open Security Controls Assessment Language) artifacts — NIST's standard for machine-readable compliance. It produces OSCAL Catalogs and Component Definitions in JSON, XML, and YAML.

- **Language**: Rust, Edition 2024, stable 1.93.0
- **Binary crate**: `src/main.rs` (CLI entry point)
- **Library crate**: `src/lib.rs` (all reusable modules)
- **License**: MIT

## Build Commands

Always run these from the repository root:

```bash
cargo build                  # Debug build
cargo build --release        # Release build
cargo run -- [args]          # Run with arguments
```

## Testing

```bash
cargo test                   # Run all tests (unit + integration)
cargo test <test_name>       # Run a single test by name
cargo test --lib             # Run only library unit tests
cargo test --doc             # Run documentation tests
```

Integration tests live in `tests/` (individual `*.rs` files). Snapshot (golden-file) tests use `insta` — run `cargo insta review` to approve new snapshots. Test fixtures are in `tests/fixtures/`.

## Linting & Formatting

**Always run both before committing.** CI enforces zero warnings:

```bash
cargo fmt --check            # Check formatting (CI gate)
cargo fmt                    # Auto-fix formatting
cargo clippy -- -D warnings  # Lint (all warnings = errors in CI)
```

The project enables `clippy::all` and `clippy::pedantic` lint groups in `Cargo.toml`.

## Full CI Replication (run locally before opening a PR)

```bash
./scripts/ci-local.sh
```

This runs: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → `cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3` → `cargo audit` → `cargo deny check`.

Individual CI steps (mirrors `.github/workflows/ci.yml`):

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3
cargo audit          # requires: cargo install cargo-audit --locked
cargo deny check     # requires: cargo install cargo-deny --locked
```

## Project Layout

```
src/
  main.rs            # CLI entry point; wires clap subcommands
  lib.rs             # Re-exports all public API
  cli/               # Subcommand handlers and output writer
  pipeline.rs        # End-to-end catalog pipeline orchestrator
  model/             # Core domain types (PolicyDocument, PolicyRequirement, …)
  types.rs           # OutputFormat, Strategy, OscalModelType enums
  error.rs           # ForgeError (thiserror), exit_code()
  oscal/             # OSCAL data structures (Catalog, ComponentDefinition, …)
  ingest/            # File reading and size validation
  parse/             # Markdown → PolicyDocument (pulldown-cmark)
  batch/             # Parallel batch conversion (rayon)
  export/            # OSCAL format conversion (JSON ↔ XML ↔ YAML)
  validate/          # JSON schema validation (jsonschema)
  round_trip/        # Round-trip serialization tests support
  trace/             # Traceability report generation
  summary/           # Conversion statistics and dashboard
  diff/              # OSCAL artifact diff report
  oscal_cli/         # Profile resolution subcommand
  parameter/         # Parameter extraction from policy prose
  citation.rs        # URL/reference extraction → OSCAL back-matter
  sanitize.rs        # Input sanitization
  uuid.rs            # Deterministic UUID v5 generation
  io.rs              # File I/O helpers
  testing/           # Test helpers (doc(hidden))

tests/               # Integration tests (one file per feature area)
  fixtures/          # Sample Markdown policies and OSCAL files
  snapshots/         # insta golden-file snapshots (checked in)
  common/            # Shared test helpers

schemas/             # Embedded OSCAL JSON schemas (include_str! at compile time)
example_data/        # 25 sample policies (acceptable use → incident response)
docs/                # Product roadmap and design documents
scripts/             # ci-local.sh, install-hooks.sh, pre-commit.sh
benches/             # Criterion benchmarks
```

## Key Architectural Patterns

- **Pipeline outputs** are returned as `PipelineOutput { content, format, secondary_outputs, statistics }`. The pipeline does **no** stdout or file I/O — all output is handled by `cli::output::write_output`.
- **Error handling**: use `ForgeError` (thiserror) in library code; `anyhow` is reserved for the binary crate (`main.rs`).
- **Serialization**: JSON via `serde_json`, XML via `quick-xml` (feature `serialize`), YAML via `serde_yaml_ng` (aliased as `serde_yaml` in `Cargo.toml`).
- **Schemas** are embedded at compile time with `include_str!` from the `schemas/` directory.
- **UUIDs** are deterministic (v5) for stable identifiers; random (v4) for unique run metadata.
- **No network dependencies** — reads and writes local files only.

## Dependencies

**New crate dependencies require approval before they are added.** Check for an
existing alternative first, then record the approval where a reviewer can find
it — in the PRD that mandates the dependency, or in the pull request that
introduces it — naming the crate, the reason, and who approved it. An
unapproved dependency must not be added; an approved one needs a supply-chain
entry, and the audit/exception policy is [PRD 069](../docs/PRD/069-prd-dependency-security-audit.md).

Crates already approved and in use:

Version ranges mirror `Cargo.toml`; use `Cargo.lock` when an exact resolved
version matters.

| Purpose | Crate |
|---------|-------|
| CLI parsing | `clap 4` (derive feature) |
| Markdown parsing | `pulldown-cmark 0.13` |
| Regex | `regex 1` |
| Serialization | `serde 1` |
| JSON | `serde_json 1` |
| XML | `quick-xml 0.41` (serialize feature) |
| YAML | `serde_yaml_ng 0.10` (aliased `serde_yaml`) |
| Error types | `thiserror 2` |
| Logging | `tracing 0.1`, `tracing-subscriber 0.3` |
| UUIDs | `uuid 1` (v4, v5, serde) |
| Timestamps | `chrono 0.4` |
| Hashing | `sha2 0.11` through the shared `src/hashing.rs` helper |
| Schema validation | `jsonschema 0.45` (resolver defaults disabled) |
| Parallel execution | `rayon 1` |
| Project configuration | `toml 1.1` (parse and serde features) |
| PDF ingestion | `pdf-extract 0.12` |
| DOCX ingestion | `zip 8` (deflate only) |
| Snapshot testing | `insta 1` (json feature) |
| Temp files | `tempfile 3` |

Approved for the local workspace transport and session unlock (mandated by
[PRD 062](../docs/PRD/062-prd-local-web-workspace.md) M-1, which requires a
loopback HTTP/1.1 listener with an Argon2id unlock; audited under PRD 069):

| Purpose | Crate |
|---------|-------|
| HTTP/1.1 server | `hyper 1` (server, http1) |
| Hyper/Tokio glue | `hyper-util 0.1` (tokio) |
| Body helpers | `http-body-util 0.1` |
| Async runtime | `tokio 1` (rt-multi-thread, net, signal, time, sync) |
| Password KDF | `argon2 0.5` (zeroize) |
| Constant-time comparison | `subtle 2` |
| Secret hygiene | `zeroize 1` |
| CSPRNG for salts and tokens | `getrandom 0.4` |
| No-echo passphrase prompt | `rpassword 7` |
| Base64 envelope decoding | `base64 0.22` |

## Important Notes

- Clippy pedantic is enabled — avoid `unwrap()`/`expect()` in library code; propagate errors with `?`.
- All lint warnings are treated as errors in CI (`-D warnings`).
- Snapshot files in `tests/snapshots/` are checked into git. After changing serialization output, run `cargo insta review` to update them.
- The `testing` module (`src/testing/`) is `#[doc(hidden)]` and only for test helpers; do not use it in production code paths.
- Format conversion (`export` subcommand) auto-detects input format from file extension.

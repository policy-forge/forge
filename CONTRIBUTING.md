# Contributing to FORGE

Thank you for your interest in contributing! FORGE is built on a spec-driven, test-first workflow. This guide walks through everything from setting up your environment to getting a PR merged.

## Dev Environment Setup

### Prerequisites

- **Rust** stable 1.93.0 or later ([rustup](https://rustup.rs/))
- **Git** for version control
- **cargo-audit** — `cargo install cargo-audit --locked`
- **cargo-deny** — `cargo install cargo-deny --locked`

### Clone and Build

```bash
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build
```

Verify everything works end-to-end:

```bash
# Quick smoke test: produce JSON output
cargo run -- convert tests/fixtures/sample_policy.md --strategy catalog --format json
```

### Install Pre-commit Hook

A local pre-commit hook runs format, lint, and test on every commit:

```bash
./scripts/install-hooks.sh
```

To bypass it once: `SKIP_FORGE_PRECOMMIT=1 git commit -m "..."`.
For stricter checks (bench + audit + deny): `FORGE_PRECOMMIT_STRICT=1 git commit -m "..."`.

### Run Full CI Locally

Before opening a PR, replicate the CI pipeline:

```bash
./scripts/ci-local.sh
```

This runs, in order: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → `cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3` → `cargo audit` → `cargo deny check`. All must pass.

## Spec-Driven Workflow

FORGE uses a structured specification process that produces living documents checked into the repo alongside the code. Every feature starts as a spec.

### Where Specs Live

```
specs/                      # All feature specifications
  NNN-feature-name/
    spec.md                 # Feature specification (user stories, acceptance criteria)
    plan.md                 # Implementation plan (technical context, constitution check, project structure)
    tasks.md                # Task breakdown (phased, checkpoints, per-story grouping)
    research.md             # Phase 0: research and decision logs
    data-model.md           # Phase 1: data model design
    quickstart.md           # Phase 1: API surface and example usage
    contracts/              # Phase 1: Rust interface contracts
    checklists/
      requirements.md       # Quality checklist
```

Supporting documents are generated into `docs/`:

```
docs/
  PRD/NNN-prd-*.md          # Product Requirement Documents
  SEC/NNN-sec-*.md          # Security review
  AR/NNN-ar-*.md            # Architecture Review
```

### How to Write a Spec

1. **Pick the next work item number** from `docs/FORGE_PRODUCT_ROADMAP.md`.
2. **Create the spec directory**: `specs/NNN-brief-name/` (use the WI number, zero-padded to 3 digits).
3. **Write `spec.md`** with user stories and acceptance scenarios. Model from existing specs like `specs/043-diff-report/spec.md`.
4. **Write `plan.md`** with technical context, constitution check, and project structure. Model from `specs/043-diff-report/plan.md`.
5. **Generate supporting docs**: PRD, SEC, AR, research, data-model, quickstart, contracts.
6. **Write `tasks.md`**: phased task breakdown with checkpoints, grouped by user story.
7. **Open a PR** for the spec documents first (or commit them as the first commit on your feature branch).

### Constitution Check GATE

Every `plan.md` must pass the project's 11-point constitution check before implementation begins. See any existing `plan.md` for the template. The check covers: crate-first architecture, Rust-first implementation, contract-first development, test-first development, complete requirement delivery, performance/scope discipline, security-first design, error handling standards, observability, simplicity, and dependency policy.

## Testing

### Running Tests

```bash
cargo test                       # All tests (unit + integration)
cargo test --all                 # Same — runs all tests (workspace-compatible alias)
cargo test --lib                 # Library unit tests only
cargo test --doc                 # Documentation tests
cargo test <test_name>           # Run a single test by name (e.g., `cargo test diff_added`)
```

FORGE targets zero test failures. As of v0.3.0, there are 1,433 passing tests across 24 integration test files and extensive unit tests.

### Snapshot Tests

Snapshots use the `insta` crate. Snapshot files in `tests/snapshots/` are checked into git. When you change serialization output:

```bash
cargo test                       # Tests that change snapshots will FAIL
cargo insta review               # Interactively review and accept new snapshots
cargo test                       # Verify everything passes
```

### Benchmarks

```bash
cargo bench                                  # All benchmarks
cargo bench --bench pipeline_benchmark       # Pipeline benchmark only
```

Benchmarks use Criterion. CI runs a quick 3-second pipeline benchmark; full benchmarks are for local regression testing.

### Mutation Testing

```bash
cargo mutants                               # Requires cargo-mutants
```

### Test Conventions

- **TDD is mandatory** — write tests before implementation.
- Library code uses `ForgeError` (thiserror); binary code (`main.rs`) uses `anyhow`.
- Avoid `unwrap()` / `expect()` in library code — propagate errors with `?`.
- Integration tests live in `tests/*.rs` with one file per feature area.
- Test fixtures are in `tests/fixtures/`; shared helpers in `tests/common/`.
- Use `insta` for snapshot (golden-file) testing of JSON/XML/YAML output.

## PR Process

### Branching

1. Fork the repo or create a branch directly.
2. Create a feature branch from `main` named `NNN-short-name` (matching the spec directory): `git checkout -b 044-my-feature`.
3. Commit spec documents as your first commit(s).

### Commit Message Style

Follow conventional commits with a work-item prefix:

```
feat(wi-044): add summary dashboard subcommand

Implemented `forge summary` that prints pipeline statistics and
conversion metadata to stderr. Added 12 tests.

Closes WI-44
```

Prefixes: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`.
Include the work item number when applicable.

### Before Opening a PR

Run the full CI pipeline locally and verify **all gates pass**:

```bash
./scripts/ci-local.sh
```

Expected output:

```
[ci-local] cargo fmt --check
[ci-local] cargo clippy -- -D warnings
[ci-local] cargo test
[ci-local] cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3
[ci-local] cargo audit
[ci-local] cargo deny check
[ci-local] all checks passed
```

A PR that fails CI will not be reviewed. Save reviewer time by running locally first.

### Code Style

- **Rust edition 2024**, max width 100 columns (see `.rustfmt.toml`).
- `clippy::all` + `clippy::pedantic` enabled in `Cargo.toml`.
- `unsafe_code = "warn"` — avoid unsafe Rust.
- No network dependencies — reads and writes local files only.
- Do not add new crate dependencies without checking for existing alternatives first.

### Review Expectations

- **Spec first**: ensure `spec.md`, `plan.md`, and `tasks.md` exist and match the implementation.
- **Tests**: every user story must have corresponding tests. TDD is the expectation.
- **Zero warnings**: `cargo clippy -- -D warnings` must pass clean.
- **Zero format violations**: `cargo fmt --check` must pass clean.
- **Security**: no vulnerabilities (`cargo audit`), allowed licenses only (`cargo deny check`).
- Maintainers may request changes; address them in the same branch.

### After Merge

- Update `CHANGELOG.md` with your changes under the appropriate version header.
- Mark tasks as done in the spec's `tasks.md` (checkboxes: `[x]`).

## Project Structure Reference

```
src/
  main.rs            # CLI entry point
  lib.rs             # Public API re-exports
  cli/               # Subcommand handlers (convert, export, validate, profile, diff, trace)
  pipeline.rs        # Catalog pipeline orchestrator
  model/             # Core domain types
  oscal/             # OSCAL data structures (catalog, component, profile, assessment_plan)
  parse/             # Markdown → PolicyDocument
  export/            # Format conversion (JSON ↔ XML ↔ YAML)
  validate/          # JSON schema validation
  diff/              # Artifact diff report
  trace/             # Traceability report generation
  summary/           # Conversion statistics dashboard
  batch/             # Parallel batch conversion (rayon)
  round_trip/        # Format round-trip serialization support
  parameter/         # Parameter extraction from policy prose
  oscal_cli/         # Profile resolution subcommand
  testing/           # Test helpers (doc(hidden))

tests/               # Integration tests
  fixtures/          # Sample input files
  snapshots/         # insta golden files (checked in)
  common/            # Shared test utilities

specs/               # Feature specifications (spec.md, plan.md, tasks.md, ...)
docs/                # PRD, SEC, AR documents and roadmap
schemas/             # OSCAL JSON schemas (compile-time embedded)
scripts/             # ci-local.sh, pre-commit.sh, install-hooks.sh
benches/             # Criterion benchmarks
example_data/        # 25 sample policies
```

## Getting Help

If you're stuck, open a draft PR with what you have and describe the blocker. The spec documents themselves are the best source of truth — check `specs/` for how prior features were designed.

---

License: MIT — your contributions will be licensed under the same terms.

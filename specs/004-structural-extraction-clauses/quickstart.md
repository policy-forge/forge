# Quickstart: Structural Extraction — Clauses & Tables

**Feature Branch**: `004-structural-extraction-clauses`

## Prerequisites

- Rust stable 1.93.0+ (`rustup update stable`)
- Existing codebase builds: `cargo build`
- WI-3 (heading extraction) tests pass: `cargo test --lib`

## Development Setup

```bash
# Switch to feature branch
git checkout 004-structural-extraction-clauses

# Verify existing tests pass
cargo test --lib

# Run clippy
cargo clippy -- -D warnings
```

## Key Files

| File | Purpose |
|------|---------|
| `src/parse/mod.rs` | Existing heading extraction; will re-export clause types + promote utilities to `pub(crate)` |
| `src/parse/clauses.rs` | **NEW** — Clause/table/paragraph extraction module |
| `src/error.rs` | Existing error types (no changes expected) |
| `src/lib.rs` | Module declarations (no changes needed; `parse` already declared) |

## Key Reference Documents

| Document | Path | What to Follow |
|----------|------|----------------|
| Architecture Review | `docs/AR/004-ar-structural-extraction-clauses.md` | Selected option, implementation guardrails, anti-patterns |
| Security Review | `docs/SEC/004-sec-structural-extraction-clauses.md` | SEC-1 through SEC-7 requirements |
| Data Model | `specs/004-structural-extraction-clauses/data-model.md` | Entity definitions (use AR naming: `ListType`, not `ListItemKind`) |
| Contract | `specs/004-structural-extraction-clauses/contracts/api.rs` | Public API types and function signature |

## AR Implementation Guardrails (MUST follow)

- **DO NOT** extract headings (WI-3 scope)
- **DO NOT** classify list items as requirements vs informational (downstream logic)
- **DO NOT** atomize compound statements (WI-6 scope)
- **DO NOT** detect normative vs advisory language (WI-33 scope)
- **DO NOT** associate items with sections (WI-5 scope)
- **MUST** use `Options::ENABLE_TABLES` (SEC-7)
- **MUST** use depth counter, not indentation parsing (SEC-4, SEC-6)
- **MUST** preserve source line numbers on every element (M-4)
- **MUST** strip inline formatting to plain text (SEC-3)
- **MUST** use `u8` for nesting_depth with saturation (SEC-4)

## TDD Workflow

Following Constitution Principle IV (Test-First Development):

```bash
# 1. Write a failing test in src/parse/clauses.rs
# 2. Verify it fails
cargo test extract_clauses

# 3. Implement minimal code to pass
# 4. Verify it passes
cargo test extract_clauses

# 5. Refactor, re-run tests
cargo test --lib
```

## AR Suggested Implementation Order

1. Define types: `ListType`, `ExtractedListItem`, `ExtractedTable`, `ExtractedParagraph`, `ExtractedContent`
2. Failing tests for basic ordered list extraction (3 items)
3. Implement list extraction with depth counter
4. Tests for unordered lists, nested lists, mixed list types
5. Failing tests for table extraction (header + 3 rows)
6. Implement table extraction with state machine
7. Tests for edge cases: empty tables, empty cells, deeply nested lists
8. Tests for inline formatting stripping
9. Implement paragraph text capture
10. Wire `extract_clauses` into the pipeline

## Verification Commands

```bash
cargo build                  # Compiles
cargo test --lib             # All unit tests pass
cargo clippy -- -D warnings  # Zero warnings
cargo fmt --check            # Formatting clean
```

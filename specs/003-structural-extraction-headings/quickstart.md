# Quickstart: Structural Extraction — Headings

**Feature Branch**: `003-structural-extraction-headings`
**Date**: 2026-02-11

## Prerequisites

- Rust stable 1.93.0+ (`rustup update stable`)
- Project checked out on branch `003-structural-extraction-headings`
- WI-2 (markdown ingestion) already merged into this branch

## Setup

### 1. Add pulldown-cmark dependency

```bash
cargo add pulldown-cmark@0.13
```

Verify in `Cargo.toml`:
```toml
[dependencies]
pulldown-cmark = "0.13"
```

### 2. Run security checks on new dependency

```bash
cargo audit
cargo deny check advisories 2>/dev/null || echo "cargo-deny not installed — check manually"
```

### 3. Verify the project builds

```bash
cargo build
cargo test
```

## Implementation Location

All heading extraction code goes in `src/parse/mod.rs`:

```rust
// src/parse/mod.rs

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use crate::ForgeError;

#[derive(Debug, Clone, PartialEq)]
pub struct SectionNode {
    pub title: String,
    pub heading_level: u8,
    pub source_line: usize,
    pub body_text: Option<String>,
    pub children: Vec<SectionNode>,
}

pub fn extract_sections(content: &str) -> Result<Vec<SectionNode>, ForgeError> {
    // Stack-based implementation here
    todo!()
}
```

## TDD Workflow

Follow the constitution TDD cycle (Principle IV):

1. **Write a failing test** in `src/parse/mod.rs` under `#[cfg(test)] mod tests`
2. **Verify it fails**: `cargo test parse::tests::test_name`
3. **Implement minimal code** to make the test pass
4. **Verify it passes**: `cargo test parse::tests::test_name`
5. **Refactor** while keeping tests green

### Suggested test order (from AR):

1. Single H1 heading
2. H1 + H2 (basic nesting)
3. H1 + H2 + H3 (deeper nesting)
4. Multiple H1 headings (multiple roots)
5. H1 → H3 skip (irregular nesting)
6. H3 as first heading (no preceding H1)
7. No headings (empty Vec)
8. Empty heading text
9. Body text between headings
10. Body text with heading having no body
11. Source line number accuracy
12. All 25 example policy documents

## Quality Gates

Before completing:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

## Key Files

| File | Role |
|------|------|
| `src/parse/mod.rs` | SectionNode struct + extract_sections implementation |
| `src/error.rs` | ForgeError (existing — no changes needed) |
| `src/ingest/mod.rs` | IngestedDocument (existing — DO NOT MODIFY) |
| `src/cli/convert.rs` | Wire extract_sections into convert pipeline |
| `example_data/*.md` | 25 policy documents for integration testing |

## Scope Boundaries

- **DO NOT** extract lists, tables, or clauses (WI-4)
- **DO NOT** create PolicySection domain model structs (WI-5)
- **DO NOT** use regex for heading detection
- **DO NOT** modify `src/ingest/mod.rs`
- **DO NOT** modify `src/model/mod.rs`

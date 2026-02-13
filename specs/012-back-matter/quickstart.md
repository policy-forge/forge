# Quickstart: OSCAL Back Matter Generation

**Feature**: 012-back-matter | **Branch**: `012-back-matter`

## Prerequisites

- Rust 1.93.0+ (edition 2024)
- Existing FORGE codebase with WI-7 (UUID generation) and WI-9 (catalog structure)

## New Dependency

```bash
cargo add url
```

## Key Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `src/oscal/back_matter.rs` | CREATE | Back matter structs + generation functions |
| `src/oscal/mod.rs` | MODIFY | Add `pub mod back_matter;` + re-exports |
| `src/oscal/catalog.rs` | MODIFY | Add `back_matter` to `OscalCatalog`, `links` to `OscalControl` |
| `src/model/mod.rs` | MODIFY | Add `Citation` struct |
| `src/error.rs` | MODIFY | Add `BackMatter` error variant |
| `src/uuid.rs` | MODIFY | Add `BACK_MATTER_NAMESPACE` constant |
| `src/lib.rs` | MODIFY | Add re-exports |

## TDD Workflow

```bash
# 1. Write tests first (RED)
cargo test --lib oscal::back_matter -- --nocapture
# Tests should FAIL

# 2. Implement (GREEN)
# Write minimal code to pass

# 3. Verify
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Usage Pattern (after implementation)

```rust
use forge::{Citation, generate_back_matter, generate_control_links};

// Create citations (normally from WI-8 extraction)
let citations = vec![
    Citation {
        id: "cit-1".into(),
        text: "NIST SP 800-53 Rev 5".into(),
        url: Some("https://nvd.nist.gov/800-53".into()),
        source_requirement_id: Some("req-uuid-here".into()),
    },
];

// Generate back matter resources + resource map
let (resources, resource_map) = generate_back_matter(&citations)?;

// Generate control links using the resource map
let links = generate_control_links(&citations, &resource_map);
```

## Verification Checklist

- [ ] `cargo test` — all tests pass
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — no formatting issues
- [ ] URL-based citations → rlinks resources
- [ ] Bibliographic citations → citation.text resources
- [ ] Malformed URLs → preserved with `prop url-status=unvalidated`
- [ ] Empty URLs → treated as malformed
- [ ] Non-http/https schemes → flagged as unvalidated
- [ ] Zero citations → back-matter omitted entirely
- [ ] Deterministic UUIDs — same input = same UUID
- [ ] No data in `remarks` fields

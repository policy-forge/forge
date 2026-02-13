# Quickstart: Traceability Model (WI-16)

**Phase**: 1 — Design & Contracts
**Date**: 2026-02-13

## Prerequisites

- Rust stable 1.93.0+
- All existing dependencies (no new crates needed)
- WI-9 (Catalog Generation) and WI-14 (Component Definition) already merged

## New File

`src/model/trace.rs` — Contains all traceability types and collection logic.

## Usage Examples

### Create a SourceLocation

```rust
use std::path::PathBuf;
use forge::model::trace::SourceLocation;

let loc = SourceLocation {
    file_path: PathBuf::from("policy.md"),
    section_title: "Access Control".to_string(),
    line_number: 42,
};
```

### Create a TraceLink

```rust
use forge::model::trace::{TraceLink, SourceLocation};

let link = TraceLink {
    requirement_stable_id: "req-uuid-001".to_string(),
    oscal_json_path: "catalog.groups[0].controls[0]".to_string(),
    oscal_element_id: "ctrl-uuid-001".to_string(),
    source_location: SourceLocation {
        file_path: PathBuf::from("policy.md"),
        section_title: "Access Control".to_string(),
        line_number: 42,
    },
};
```

### Record and Query TraceLinks

```rust
use forge::model::trace::{TraceLinkCollection, TraceLink, SourceLocation};

let mut collection = TraceLinkCollection::new();

// Record a trace link (during generation)
let link = TraceLink { /* ... */ };
collection.record(link)?;

// Forward lookup: requirement -> OSCAL elements
let links = collection.by_requirement("req-uuid-001");
assert_eq!(links.len(), 1);

// Reverse lookup: OSCAL element -> source
let found = collection.by_oscal_element("ctrl-uuid-001");
assert!(found.is_some());

// Iteration and diagnostics
println!("Total trace links: {}", collection.len());
for link in collection.iter() {
    println!("{} -> {}", link.requirement_stable_id, link.oscal_element_id);
}
```

### Duplicate Detection

```rust
// Attempting to record a duplicate oscal_element_id returns an error
let result = collection.record(duplicate_link);
assert!(result.is_err()); // TraceError::DuplicateElement
```

## Build & Test

```bash
cargo build          # Verify compilation
cargo test           # Run all tests including trace model tests
cargo clippy         # Lint check
cargo fmt --check    # Format check
```

## Key Patterns

1. **Immutability**: TraceLink instances are never modified after creation
2. **Append-only**: TraceLinkCollection only grows during generation
3. **Ownership transfer**: Builders receive `&mut TraceLinkCollection` during generation, downstream consumers receive `&TraceLinkCollection` for read-only access
4. **Graceful missing lookups**: `by_requirement()` returns empty slice, `by_oscal_element()` returns None

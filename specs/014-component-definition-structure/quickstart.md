# Quickstart: OSCAL Component Definition Structure

**Phase 1 output** | **Date**: 2026-02-12

## Usage

```rust
use forge::model::{DocumentMetadata, PolicyDocument};
use forge::oscal::component_definition::build_component_definition;

// 1. Have a PolicyDocument (from the pipeline or constructed for testing)
let doc = PolicyDocument {
    id: "policy-001".into(),
    metadata: DocumentMetadata {
        title: "Corporate Security Policy".into(),
        version: "2.0".into(),
        ..Default::default()
    },
    sections: vec![],
};

// 2. Build the Component Definition
let envelope = build_component_definition(&doc).unwrap();

// 3. Serialize to JSON
let json = serde_json::to_string_pretty(&envelope).unwrap();

// Output:
// {
//   "component-definition": {
//     "uuid": "<document-uuid-v4>",
//     "metadata": {
//       "title": "Corporate Security Policy",
//       "last-modified": "2026-02-12T...",
//       "version": "2.0",
//       "oscal-version": "1.2.0"
//     },
//     "components": [
//       {
//         "uuid": "<component-uuid-v5>",
//         "type": "policy",
//         "title": "Corporate Security Policy",
//         "description": "Documentary component representing the Corporate Security Policy policy document.",
//         "control-implementations": []
//       }
//     ]
//   }
// }
```

## Key Patterns

### Deterministic Component UUID
```rust
// Same input always produces the same component UUID
let env1 = build_component_definition(&doc).unwrap();
let env2 = build_component_definition(&doc).unwrap();
assert_eq!(
    env1.component_definition.components[0].uuid,
    env2.component_definition.components[0].uuid,
);
// Note: document-level UUIDs WILL differ (v4 random)
```

### Empty Title Default
```rust
let doc = PolicyDocument {
    id: "empty-title".into(),
    metadata: DocumentMetadata {
        title: String::new(),  // empty
        ..Default::default()
    },
    sections: vec![],
};
let env = build_component_definition(&doc).unwrap();
assert_eq!(env.component_definition.components[0].title, "Untitled Policy Document");
```

### Back Matter Inclusion
```rust
// Back matter is automatically included when the PolicyDocument has citations
// (reuses WI-12 generate_back_matter). If no citations, back-matter is omitted
// from the JSON output via skip_serializing_if.
```

## Files to Modify

| File | Change |
|------|--------|
| `src/oscal/component_definition.rs` | **NEW** -- Builder function and types |
| `src/oscal/mod.rs` | Add `pub mod component_definition;` and re-exports |
| `src/uuid.rs` | Add `COMPONENT_NAMESPACE` constant |
| `src/error.rs` | Add `ComponentDefinitionBuild(String)` variant |
| `src/lib.rs` | Add re-exports for new public types |

## Build & Test

```bash
cargo test --lib                    # Run library unit tests
cargo test component_definition     # Run only component definition tests
cargo clippy -- -D warnings         # Lint check
cargo fmt --check                   # Format check
```

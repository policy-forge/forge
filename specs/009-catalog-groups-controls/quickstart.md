# Quickstart: OSCAL Catalog Groups and Controls

**Feature**: 009-catalog-groups-controls

## Developer Guide

### What this feature does

Converts a `PolicyDocument` (domain model from WI-5 through WI-8) into an OSCAL Catalog JSON structure:
- `PolicySection` → `OscalGroup` (with slugified group ID)
- `PolicyRequirement` → `OscalControl` (with `POL-{ABBR}-{NNN}` control ID)

### Prerequisites

- WI-7 (UUID Generation) must have run — all `PolicyRequirement.stable_id` fields must be `Some`
- No new dependencies to install; `serde`, `serde_json`, `tracing`, `thiserror` are already in `Cargo.toml`

### Usage

```rust
use forge::oscal::catalog::{build_catalog, CatalogEnvelope};
use forge::model::PolicyDocument;

// Assume `document` is a fully enriched PolicyDocument from the pipeline
let catalog = build_catalog(&document)?;

// Wrap in envelope for OSCAL-compliant JSON
let envelope = CatalogEnvelope { catalog };

// Serialize to JSON
let json = serde_json::to_string_pretty(&envelope)?;
println!("{json}");
```

### Output Example

```json
{
  "catalog": {
    "uuid": "00000000-0000-0000-0000-000000000000",
    "metadata": {
      "title": "placeholder",
      "last-modified": "1970-01-01T00:00:00Z",
      "version": "0.0.0",
      "oscal-version": "1.2.0"
    },
    "groups": [
      {
        "id": "access-control",
        "title": "Access Control",
        "controls": [
          {
            "id": "POL-AC-001",
            "uuid": "a1b2c3d4-...",
            "title": "Systems shall require MFA for all privileged access."
          }
        ]
      }
    ]
  }
}
```

### Key Files

| File | Purpose |
|------|---------|
| `src/oscal/catalog.rs` | OSCAL structs + `build_catalog()` + ID generation functions |
| `src/oscal/mod.rs` | Module root (re-exports) |
| `src/error.rs` | `ForgeError::CatalogBuild` variant |

### Running Tests

```bash
cargo test --lib          # Run unit tests (includes catalog tests)
cargo test catalog        # Run only catalog-related tests
cargo clippy -- -D warnings  # Lint check
cargo fmt --check         # Format check
```

### TDD Workflow

Each function follows Red-Green-Refactor:

1. Write test in `#[cfg(test)] mod tests` block in `src/oscal/catalog.rs`
2. Verify test fails (`cargo test`)
3. Implement minimal code to pass
4. Verify test passes
5. Refactor if needed

### Design Constraints (from AR Implementation Guardrails)

- **DO NOT** add statement parts (`parts[]`) to controls — deferred to WI-10
- **DO NOT** populate real metadata — deferred to WI-11
- **DO NOT** add back matter or links — deferred to WI-12
- **DO NOT** use `serde_json::Value` — use typed structs
- **DO NOT** mutate the `PolicyDocument` — read-only transformation
- **DO NOT** generate UUIDs — use `stable_id` from WI-7
- **MUST** generate unique control IDs following `POL-{ABBR}-{NNN}` pattern
- **MUST** preserve section and requirement ordering
- **MUST** serialize with `serde_json` producing valid, parseable JSON

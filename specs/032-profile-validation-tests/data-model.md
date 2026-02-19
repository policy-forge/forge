# Data Model: Profile Validation and Golden-File Tests (WI-32)

**Branch**: `032-profile-validation-tests` | **Date**: 2026-02-18

> **Note:** WI-32 is a test-only work item. There is no new production data model. This document describes the OSCAL Profile JSON structure that tests operate on, the synthetic test fixture data, and the normalization rules applied before snapshot comparison.

---

## Entity 1: OSCAL Profile (WI-30 output, test subject)

The Profile JSON produced by `forge profile` has this structure (WI-30 implementation):

```json
{
  "profile": {
    "uuid": "<uuid-v4>",
    "metadata": {
      "title": "Generated OSCAL Profile",
      "last-modified": "<RFC3339 timestamp>",
      "version": "0.0.1",
      "oscal-version": "1.2.0"
    },
    "imports": [
      {
        "href": "<catalog-path>",
        "include-controls": [
          { "with-ids": ["CTRL-1", "CTRL-2"] }
        ]
      }
    ]
  }
}
```

For exclusion mode, `exclude-controls` replaces `include-controls`:
```json
{
  "imports": [
    {
      "href": "<catalog-path>",
      "exclude-controls": [
        { "with-ids": ["CTRL-3"] }
      ]
    }
  ]
}
```

For parameter tailoring (WI-31, not yet implemented — `#[ignore]` tests only):
```json
{
  "profile": {
    "imports": [...],
    "modify": {
      "set-parameters": [
        {
          "param-id": "param-1",
          "values": ["new-value"]
        }
      ]
    }
  }
}
```

**Required OSCAL v1.2.0 Profile schema fields:**
- `profile.uuid` — UUID v4 format, required
- `profile.metadata` — required
  - `metadata.title` — required string
  - `metadata.last-modified` — required RFC3339 datetime string
  - `metadata.version` — required string
  - `metadata.oscal-version` — required string (must be valid OSCAL version)
- `profile.imports` — required array (at least one entry), each with:
  - `imports[].href` — required string (catalog href, must be non-empty)

---

## Entity 2: Normalized Profile Snapshot

Before golden-file comparison, the Profile JSON is normalized to eliminate dynamic fields that change on every run. This produces a stable JSON value suitable for `insta::assert_json_snapshot!()`.

**Normalization rules (identical to WI-21):**

| Field/Pattern | Before | After |
|--------------|--------|-------|
| UUID-format strings (any field) | `"a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d"` | `"00000000-0000-0000-0000-000000000000"` |
| `last-modified` values | `"2026-02-18T15:30:00Z"` | `"2026-01-01T00:00:00Z"` |
| Absolute path hrefs | `"/var/folders/.../catalog.json"` | `"NORMALIZED_PATH"` |

**Pattern for UUID detection:** A UUID-format string matches the regex:
`^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$`

**Pattern for absolute path detection:** A string starting with `/` or a Windows drive letter (`C:\`, etc.) is treated as an absolute path.

**Normalization is applied recursively** — all JSON values (objects, arrays, strings) are traversed.

---

## Entity 3: Synthetic Test Fixture Catalog

Profile tests need a file at a valid path for the `--catalog` argument (the catalog path is used as the `imports[0].href` value). The file content is NOT parsed by WI-30 profile generation — only existence is checked.

**Minimal catalog JSON for tempfile:**
```json
{
  "catalog": {
    "uuid": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
    "metadata": {
      "title": "Test Catalog",
      "last-modified": "2026-01-01T00:00:00Z",
      "version": "1.0",
      "oscal-version": "1.2.0"
    },
    "controls": [
      {"id": "AC-1", "title": "Access Control Policy"},
      {"id": "AC-2", "title": "Account Management"},
      {"id": "AC-3", "title": "Access Enforcement"},
      {"id": "AC-4", "title": "Information Flow Enforcement"},
      {"id": "AC-5", "title": "Separation of Duties"},
      {"id": "AC-6", "title": "Least Privilege"},
      {"id": "AC-7", "title": "Unsuccessful Logon Attempts"},
      {"id": "AC-8", "title": "System Use Notification"},
      {"id": "AC-9", "title": "Previous Logon Notification"},
      {"id": "AC-10", "title": "Concurrent Session Control"}
    ]
  }
}
```

**Rationale for 10 controls:** Edge case tests require "all-controls selection" (FR-007). Using exactly 10 controls keeps the fixture small while enabling `--include AC-1,AC-2,...,AC-10`.

---

## Entity 4: OscalModelType::Profile (FR-000 extension)

The `OscalModelType` enum in `src/validate/mod.rs` gains a `Profile` variant:

```rust
pub enum OscalModelType {
    Catalog,              // existing
    ComponentDefinition,  // existing
    Profile,              // NEW (FR-000)
}
```

**Display mapping:**
- `Catalog` → `"catalog"`
- `ComponentDefinition` → `"component-definition"`
- `Profile` → `"profile"` (NEW)

**`detect_model_type()` mapping:**
- `json.get("catalog")` → `Catalog`
- `json.get("component-definition")` → `ComponentDefinition`
- `json.get("profile")` → `Profile` (NEW)
- else → `UnknownModelType` error

**`load_schema()` mapping:**
- `Catalog` → `include_str!("../../schemas/oscal_catalog_schema.json")`
- `ComponentDefinition` → `include_str!("../../schemas/oscal_component_schema.json")`
- `Profile` → `include_str!("../../schemas/oscal_profile_schema.json")` (NEW)

---

## Test Fixture Relationships

```
Test Function
    │
    ├── tempfile::NamedTempFile (minimal catalog JSON)
    │       └── path used as --catalog arg and as href in Profile imports
    │
    ├── forge profile CLI (build_profile + ProfileRoot serialization)
    │       └── stdout Profile JSON string
    │
    ├── Schema Validation Path:
    │       serde_json::from_str() → validate_artifact(OscalModelType::Profile)
    │               └── ValidationResult { is_valid, errors }
    │
    └── Golden-File Path:
            serde_json::from_str() → normalize_for_snapshot()
                    └── insta::assert_json_snapshot!("name", normalized)
                            └── tests/snapshots/profile_golden_file_tests__<name>.snap
```

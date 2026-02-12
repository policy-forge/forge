# Data Model: OSCAL Metadata Assembly

**Phase**: 1 | **Date**: 2026-02-12

## Entities

### OscalMetadata

The metadata object attached to every OSCAL artifact. Contains five required fields per OSCAL v1.2.0.

| Field | Rust Type | Serde Name | Source | Validation |
|-------|-----------|------------|--------|------------|
| `uuid` | `Uuid` | `uuid` | `Uuid::new_v4()` or `MetadataOptions.uuid_override` | Valid UUID v4 format (version nibble = 4, variant bits correct) |
| `title` | `String` | `title` | `DocumentMetadata.title` (clone) | Any string; warn if empty (EC-1) |
| `last_modified` | `DateTime<Utc>` | `last-modified` | `Utc::now()` or `MetadataOptions.timestamp_override` | Valid ISO 8601 UTC timestamp |
| `version` | `String` | `version` | `DocumentMetadata.version` (clone) | Any string; "0.0.0" is valid default (EC-2) |
| `oscal_version` | `String` | `oscal-version` | `OSCAL_VERSION` constant (`"1.2.0"`) | Always "1.2.0" |

**Derives**: `Debug`, `Clone`, `Serialize`
**Serde**: Selective `#[serde(rename)]` on `last_modified` and `oscal_version` fields

### MetadataOptions

Optional configuration for overriding auto-generated values. Primary use: deterministic testing.

| Field | Rust Type | Default | Purpose |
|-------|-----------|---------|---------|
| `uuid_override` | `Option<Uuid>` | `None` | Inject fixed UUID for test assertions |
| `timestamp_override` | `Option<DateTime<Utc>>` | `None` | Inject fixed timestamp for test assertions |

**Derives**: `Debug`, `Default`

### DocumentMetadata (existing — READ ONLY)

From `src/model/mod.rs` (WI-5). Only `title` and `version` fields consumed by metadata assembly.

| Field | Type | Consumed By |
|-------|------|-------------|
| `title` | `String` | `OscalMetadata.title` |
| `version` | `String` | `OscalMetadata.version` |
| `author` | `Option<String>` | Not consumed |
| `date` | `Option<String>` | Not consumed |
| `source_path` | `PathBuf` | Not consumed |
| `content_hash` | `Option<String>` | Not consumed |

## Constants

| Name | Type | Value | Purpose |
|------|------|-------|---------|
| `OSCAL_VERSION` | `&str` | `"1.2.0"` | OSCAL specification version; single point of change for future bumps |

## Relationships

```text
DocumentMetadata (WI-5, existing)
    ├── .title ──────────► OscalMetadata.title
    └── .version ────────► OscalMetadata.version

MetadataOptions (optional)
    ├── .uuid_override ──► OscalMetadata.uuid (if Some)
    └── .timestamp_override ► OscalMetadata.last_modified (if Some)

Uuid::new_v4() ──────────► OscalMetadata.uuid (if no override)
Utc::now() ──────────────► OscalMetadata.last_modified (if no override)
OSCAL_VERSION ───────────► OscalMetadata.oscal_version (always)
```

## State Transitions

N/A — metadata assembly is a single-pass construction. No state machine.

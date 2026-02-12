# Research: OSCAL Catalog Groups and Controls

**Feature**: 009-catalog-groups-controls
**Date**: 2026-02-11
**Status**: Complete — all unknowns resolved

## Research Summary

No NEEDS CLARIFICATION items existed in the Technical Context. All technologies and patterns are established in the existing codebase. The three ambiguities identified during `/speckit.clarify` have been resolved:

### R-1: Abbreviation Collision Resolution Strategy

**Decision**: Numeric suffix (first section keeps base abbreviation; subsequent collisions get `AC2`, `AC3`, etc.)
**Rationale**: Simplest deterministic approach. No complex "longer abbreviation" heuristics needed. Generalizes cleanly to any number of collisions.
**Alternatives considered**: Longer abbreviation (take more letters from words — rejected due to heuristic complexity and non-intuitive results), Hybrid (try longer first, fall back to numeric — rejected as over-engineering for this use case).

### R-2: Control Title Derivation Strategy

**Decision**: Extract first sentence (up to first `.`, `!`, or `?`), capped at 120 characters with `...` suffix if exceeded.
**Rationale**: First sentence is the most semantically meaningful unit. 120 characters is a practical display width for OSCAL tooling. The `...` suffix signals truncation clearly.
**Alternatives considered**: Always truncate at 80 chars (too aggressive, loses context), First sentence with no cap (arbitrarily long titles hurt readability), Full requirement text (too long for a "title" field).

### R-3: Nested Section Handling

**Decision**: Flat mapping for MVP — only top-level `document.sections` become groups. Child sections' requirements are recursively collected into the parent group's controls.
**Rationale**: Flat mapping is simpler to implement, test, and reason about. OSCAL supports nested groups but the MVP does not need them. Recursive nesting can be added in a future WI if needed.
**Alternatives considered**: Recursive nested OSCAL groups (complex struct handling, deeper collision tracking — deferred), Flat with warning on nested sections (unnecessary noise for MVP).

## Technology Best Practices

### Serde Serialization for OSCAL

- Use `#[derive(Debug, Clone, Serialize)]` on all OSCAL structs
- Use `#[serde(rename = "...")]` for hyphenated OSCAL field names (e.g., `last-modified` → `last_modified`)
- Use `#[serde(skip_serializing_if = "Vec::is_empty")]` to omit empty collections (per OSCAL conventions)
- Use `serde_json::to_string_pretty()` for human-readable output
- Wrap the catalog in a `CatalogEnvelope` struct with `#[serde(rename = "catalog")]` field for the OSCAL root key

### Slugification Pattern

- Lowercase the title
- Replace non-alphanumeric characters with hyphens
- Collapse consecutive hyphens into one
- Trim leading/trailing hyphens
- This is a standard pattern; no external crate needed — implement with simple string operations

### Section Title Abbreviation

- Split title into words
- Remove stop words: `and`, `the`, `of`, `for`, `in`, `to`, `a`, `an`
- Take first character of each remaining word, uppercase
- Track used abbreviations in a `HashMap<String, usize>` for collision detection
- On collision: append count (e.g., `AC` → `AC2` for second occurrence)

### ForgeError Pattern

- Add a `CatalogBuild(String)` variant to the existing `ForgeError` enum
- Follows the established pattern of `Parse(String)`, `Validation(String)`, `Config(String)`
- Error messages should include the requirement text and section title for diagnosis

## Dependencies

No new dependencies required. All needed crates are already in `Cargo.toml`:

| Crate | Version | Purpose |
|-------|---------|---------|
| serde | 1.0.228 | `#[derive(Serialize)]` for OSCAL structs |
| serde_json | 1.0.149 | JSON serialization |
| thiserror | 2.0.18 | `ForgeError::CatalogBuild` variant |
| tracing | 0.1.44 | DEBUG-level logging for group/control counts |

## OSCAL v1.2.0 Reference

From the research document (`docs/research/OSCAL_Research.md`), the expected JSON structure:

```json
{
  "catalog": {
    "uuid": "...",
    "metadata": {
      "title": "...",
      "last-modified": "...",
      "version": "...",
      "oscal-version": "1.2.0"
    },
    "groups": [
      {
        "id": "access-control",
        "title": "Access Control Policies",
        "controls": [
          {
            "id": "POL-AC-001",
            "uuid": "...",
            "title": "..."
          }
        ]
      }
    ]
  }
}
```

Key observations:
- Root object has a single `"catalog"` key (envelope pattern)
- `metadata` has four required fields, all strings
- `groups` is an array of group objects
- Each group has `id`, `title`, and `controls` array
- Each control has `id`, `uuid`, and `title`
- `parts` (WI-10), `props`, and `links` (WI-12) are added by later WIs

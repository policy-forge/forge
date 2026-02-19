# Feature Spec: 030-prd-profile-generation

**Feature**: OSCAL Profile Generation (`forge profile` subcommand)
**Branch**: `030-prd-profile-generation`
**Derived From**: docs/PRD/030-prd-profile-generation.md (WI-30)

---

## Summary

Add a `forge profile` subcommand that accepts a source Catalog path and comma-separated control ID lists (`--include` or `--exclude`), then produces a valid OSCAL v1.2.0 Profile JSON with the `imports[]` structure. This is the foundational Profile capability that WI-31 (parameter tailoring) and WI-32 (validation) build upon.

## Goals

- **M-1**: `forge profile` subcommand via clap derive macros
- **M-2**: `--catalog <path>` flag (source Catalog file path → stored as href)
- **M-3**: `--include <ids>` flag (comma-separated control IDs → `include-controls`)
- **M-4**: `--exclude <ids>` flag (comma-separated control IDs → `exclude-controls`)
- **M-5**: `imports[]` array with entry whose `href` = `--catalog` value
- **M-6**: `include-controls[0].with-ids` when `--include` used
- **M-7**: `exclude-controls[0].with-ids` when `--exclude` used
- **M-8**: Profile metadata (`uuid`, `title`, `last-modified`, `version`, `oscal-version`) via WI-11 `assemble_metadata`
- **M-9**: Root JSON key `"profile"` wrapping the Profile struct; OSCAL v1.2.0 compliant
- **S-1**: `--format json` flag (default: `json`)
- **S-2**: `--output <path>` flag (default: stdout)
- **S-3**: Actionable error when `--catalog` file not found (catalog JSON validity check deferred to WI-32)
- **S-4**: Error when both `--include` and `--exclude` provided (mutually exclusive)

## Non-Goals (Deferred)

- Parameter tailoring / `modify` section (WI-31)
- Profile validation and golden-file tests (WI-32)
- Profile Resolution engine (delegates to NIST oscal-cli)
- Multiple catalog imports per Profile
- Merge directives (`merge` section)
- XML/YAML Profile output (uses existing WI-26/27 infrastructure if needed)

## Acceptance Criteria

| AC ID | Condition | Expected Outcome |
|-------|-----------|-----------------|
| AC-1 | `forge profile --help` | Shows `--catalog`, `--include`, `--exclude` flags |
| AC-2 | `forge profile --catalog catalog.json --include POL-AC-001,POL-AC-002` | `imports[0].href="catalog.json"`, `imports[0].include-controls[0].with-ids=["POL-AC-001","POL-AC-002"]` |
| AC-3 | `forge profile --catalog catalog.json --exclude POL-AC-003` | `imports[0].exclude-controls[0].with-ids=["POL-AC-003"]` |
| AC-4 | Any generation request | `profile.metadata` has all 5 required fields |
| AC-5 | Generated JSON | Root key is `"profile"`; OSCAL v1.2.0 shape |
| AC-6 | `--output baseline.json` | File created with valid Profile JSON |
| AC-7 | No `--output` | Profile JSON on stdout |
| AC-8 | `--catalog missing.json` | Actionable error message |
| AC-9 | Both `--include` and `--exclude` | Clear mutual-exclusivity error |

## Edge Cases

- **EC-1**: Single control ID (no comma) → single-element `with-ids`
- **EC-2**: IDs with extra whitespace → trimmed
- **EC-3**: No flags → helpful error about required arguments
- **EC-4**: Duplicate control IDs → deduplicated in `with-ids`
- **EC-5**: Empty `--include` string → descriptive error
- **EC-6**: Catalog path exists but not valid JSON → deferred to WI-32 (profile validation); WI-30 only checks existence
- **EC-7**: Root JSON key is `"profile"` (OSCAL convention) *(traces to M-9)*

## Security Requirements

> Source of truth: `docs/SEC/030-sec-profile-generation.md`. Keep this table in sync when updating either document.

| Req ID | Requirement |
|--------|------------|
| SEC-1 | Profile JSON must not embed policy text — only control ID references and metadata |
| SEC-2 | `--include` and `--exclude` mutually exclusive via clap `conflicts_with` |
| SEC-3 | Empty control ID string → descriptive error, not empty Profile |
| SEC-4 | Catalog path stored as-is in href — no filesystem operations on it (see note below) |
| SEC-5 | Generated Profile JSON conforms to OSCAL v1.2.0 Profile schema structure |

> **Constitution §VII departure (accepted):** Constitution §VII requires file paths to be canonicalized. The `--catalog` path is intentionally stored as-is in the Profile `href` field to preserve OSCAL portability (relative paths must remain relative). This applies only to the `href` reference string; the catalog existence check in `cli/profile.rs` uses the path normally. Accepted as risk R3 in `docs/SEC/030-sec-profile-generation.md` (approver: Brian Luby, 2026-02-11).

## Dependencies

- **Requires**: WI-1 (project scaffolding), WI-11 (`assemble_metadata`), WI-29 (export/output patterns)
- **Blocks**: WI-31 (parameter tailoring), WI-32 (Profile validation)

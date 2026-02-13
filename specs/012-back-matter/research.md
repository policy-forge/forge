# Research: OSCAL Back Matter Generation

**Feature**: 012-back-matter | **Date**: 2026-02-12

## R-1: URL Validation Strategy

**Decision**: Use the `url` crate (latest stable, currently ~2.5.x) for URL parsing and scheme validation.

**Rationale**: Standard Rust URL parsing crate (WHATWG-compliant), 418M+ downloads, actively maintained by the servo project. Dual MIT/Apache-2.0 license. Provides `Url::parse()` which returns `Result<Url, ParseError>` with specific error variants, and `.scheme()` for http/https filtering. Empty strings return `Err(ParseError::RelativeUrlWithoutBase)`.

**Alternatives Considered**:
- Regex-based validation: fragile, incomplete coverage, hard to maintain — rejected
- Manual parsing: error-prone, reinvents WHATWG logic — rejected
- hyper::Uri: HTTP-focused only, not general URL validation — rejected

**Implementation Notes**:
- Pre-check empty/whitespace-only strings before `Url::parse` for clear error messages
- After successful parse, check `url.scheme() == "http" || url.scheme() == "https"`
- All other schemes (ftp, mailto, javascript, data, file) flagged as `unvalidated`
- Malformed URLs (parse failure) also flagged as `unvalidated` with URL preserved in rlinks

## R-2: UUID v5 Namespace Strategy

**Decision**: Derive a dedicated back-matter namespace via `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"back-matter")` and hardcode the resulting bytes as `BACK_MATTER_NAMESPACE`.

**Rationale**: The AR explicitly specifies "BACK_MATTER_NAMESPACE = UUID v5 derived from 'forge:back-matter' in FORGE's root namespace." A dedicated namespace provides strict collision isolation — back matter UUIDs are in an entirely separate UUID space from control UUIDs. Consistent with the existing `FORGE_NAMESPACE_UUID` pattern of hardcoded byte arrays.

**Alternatives Considered**:
- Content prefix (`format!("back-matter:{content}")`) with single namespace: simpler but less explicit; collision avoidance relies on content-level separation — rejected per AR
- Random UUID v4 for resources: not deterministic, violates M-4 — rejected

**Implementation Notes**:
- Compute `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"back-matter")` once, extract bytes, hardcode as `const BACK_MATTER_NAMESPACE: Uuid = Uuid::from_bytes([...])` in `uuid.rs`
- Resource UUID: `Uuid::new_v5(&BACK_MATTER_NAMESPACE, normalized_content.as_bytes())`
- Content normalization reuses existing `normalize_for_hashing` function

## R-3: Citation Domain Model (WI-8 Input Contract)

**Decision**: Define a `Citation` struct in `src/model/mod.rs` as the input contract for back matter generation.

**Rationale**: Constitution III requires contract-first development. The Citation struct represents the output of WI-8 and input to WI-12. Fields derived from PRD assumption A-1: "text, optional URL, and a reference to the source requirement."

**Alternatives Considered**:
- Inline tuples or untyped data: violates contract-first principle — rejected
- Separate citation crate: overkill for a single struct — rejected

**Implementation Notes**:
- `Citation { id: String, text: String, url: Option<String>, source_requirement_id: Option<String> }`
- `id` field enables the `HashMap<String, Uuid>` resource map
- `url` is `Option<String>` — `None` means bibliographic-only; `Some("")` treated as malformed

## R-4: OscalCatalog Integration

**Decision**: Add `back_matter` field to `OscalCatalog` and `links` field to `OscalControl`.

**Rationale**: Back matter is a top-level OSCAL Catalog field. Control links are per-control elements. Both integrate with existing structs in `src/oscal/catalog.rs`.

**Alternatives Considered**:
- Separate back-matter JSON merged post-generation: fragile, breaks type safety — rejected
- Back matter as a separate output file: not valid OSCAL — rejected

**Implementation Notes**:
- `OscalCatalog.back_matter: Option<BackMatter>` with `skip_serializing_if = "Option::is_none"` (omitted when zero citations)
- `OscalControl.links: Vec<OscalLink>` with `skip_serializing_if = "Vec::is_empty"`
- Serde rename `back_matter` to `back-matter` for OSCAL JSON compliance

# Rust Interface Contracts: Internal Domain Model

**Feature**: 005-domain-model
**Date**: 2026-02-11
**Module**: `src/model/`

## Overview

This document specifies the Rust module interfaces for the internal domain model. Since this is not a REST/GraphQL API but an internal Rust library, the "contracts" are the public struct definitions and function signatures that downstream work items (WI-6+) will depend on.

**Contract Stability**: These interfaces are the public API boundary. Changes to public fields or function signatures constitute breaking changes and require coordination with dependent work items.

---

## Module: `model::mod.rs`

### Public Structs

#### PolicyDocument

```rust
/// Top-level domain model for a parsed policy document.
///
/// This is the canonical internal representation consumed by all
/// downstream pipeline stages (atomization, UUID generation, OSCAL generation).
///
/// # Lifecycle
/// - Created by `assemble_document` function (WI-5)
/// - Enriched by WI-6 (atomization)
/// - Enriched by WI-7 (UUID generation populates stable_id fields)
/// - Enriched by WI-8 (citation extraction)
/// - Consumed by WI-9+ (OSCAL generation)
///
/// # Ownership
/// Passed through pipeline using functional transformation: each WI takes
/// ownership and returns an enriched instance.
#[derive(Debug, Clone)]
pub struct PolicyDocument {
    /// Document identifier (derived from filename or frontmatter)
    pub id: String,

    /// Document-level metadata
    pub metadata: DocumentMetadata,

    /// Top-level sections (may contain nested children)
    pub sections: Vec<PolicySection>,
}
```

**Contract Guarantees**:
- `id` is always non-empty
- `sections` may be empty (valid for documents with no structure)
- All fields are public for downstream access
- Derives `Debug` and `Clone` for debugging and data flow

---

#### DocumentMetadata

```rust
/// Metadata about the source document.
///
/// Extracted from YAML frontmatter when available, with fallback to
/// heading-based extraction and sensible defaults.
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// Document title
    /// - Frontmatter `title` field, OR
    /// - First H1 heading text, OR
    /// - Filename (without extension)
    pub title: String,

    /// Document version (semantic versioning format preferred)
    /// - Frontmatter `version` field, OR
    /// - Default "0.0.0"
    pub version: String,

    /// Document author (from frontmatter only)
    pub author: Option<String>,

    /// Document date (ISO 8601 format preferred; from frontmatter only)
    pub date: Option<String>,

    /// Path to the source file
    pub source_path: PathBuf,

    /// SHA-256 hash of source content (from IngestedDocument)
    /// None if not computed by ingestion layer
    pub content_hash: Option<String>,
}
```

**Contract Guarantees**:
- `title` is always non-empty
- `version` is always non-empty (defaults to "0.0.0")
- `source_path` is a valid PathBuf
- `author` and `date` are None if not present in frontmatter
- `content_hash` is None if not provided by ingestion (WI-2)

---

#### PolicySection

```rust
/// A hierarchical section within a policy document, mapped from a Markdown heading.
///
/// Sections form a tree structure with parent-child relationships.
#[derive(Debug, Clone)]
pub struct PolicySection {
    /// Section title (heading text)
    pub title: String,

    /// Heading level: 1 for H1, 2 for H2, ..., 6 for H6
    pub heading_level: u8,

    /// Source line number in the original document (1-based)
    pub source_line: usize,

    /// Text content between this heading and first child heading or next sibling
    /// None if section has no body text
    pub body_text: Option<String>,

    /// Child sections (deeper heading levels)
    pub children: Vec<PolicySection>,

    /// Policy requirements extracted from list items within this section
    pub requirements: Vec<PolicyRequirement>,
}
```

**Contract Guarantees**:
- `title` is always non-empty
- `heading_level` is in range 1-6 inclusive
- `source_line` is >= 1 (1-based line numbering)
- `children` may be empty (valid for leaf sections)
- `requirements` may be empty (valid for sections with only body text)
- Child sections have `heading_level > parent.heading_level`

---

#### PolicyRequirement

```rust
/// An individual policy requirement extracted from a list item or clause pattern.
///
/// Requirements represent the atomic units of policy that will map to OSCAL controls.
#[derive(Debug, Clone)]
pub struct PolicyRequirement {
    /// Stable UUID for this requirement
    /// - None until populated by WI-7 (UUID generation)
    /// - Some(uuid) after WI-7
    pub stable_id: Option<String>,

    /// Full text of the requirement
    /// Pre-atomization: may be compound statement
    /// Post-atomization (WI-6): atomic statement
    pub text: String,

    /// Source line number in the original document (1-based)
    pub source_line: usize,

    /// Nesting depth from extraction (0 = top-level list item, 1 = nested once, etc.)
    pub nesting_depth: u8,
}
```

**Contract Guarantees**:
- `text` is always non-empty
- `source_line` is >= 1 (1-based line numbering)
- `stable_id` is None until WI-7 populates it
- After WI-7, `stable_id` is always Some(uuid_string)

**Temporary Identity** (before `stable_id` is assigned):
- For testing/debugging, requirements can be identified by `(source_line, text_hash)` tuple
- Not part of struct; computed on demand

---

### Public Functions

#### assemble_document

```rust
/// Assemble a PolicyDocument from ingestion and extraction outputs.
///
/// Bridges the three extraction outputs into a unified domain model:
/// - `IngestedDocument` provides file path, content hash, raw content
/// - `Vec<SectionNode>` provides heading hierarchy
/// - `ExtractedContent` provides list items, tables, paragraphs
///
/// # Arguments
/// * `ingested` - Reference to the ingested document (from WI-2): provides source_path,
///   fingerprint (mapped to content_hash), and lines (content reconstructed from lines)
/// * `sections` - Section hierarchy tree (from WI-3)
/// * `clauses` - Extracted content (from WI-4)
///
/// # Returns
/// * `Ok(PolicyDocument)` - Complete domain model with all sections and requirements
/// * `Err(ForgeError)` - Fatal error preventing assembly (e.g., invalid section tree)
///
/// # Errors
/// Returns `ForgeError::Parse` if:
/// - Section tree structure is invalid
/// - Data inconsistency prevents assembly
///
/// Does NOT error on:
/// - Malformed YAML frontmatter (warns to stderr, uses fallback)
/// - Empty document (returns Ok with empty sections)
/// - Missing frontmatter (uses heading/filename fallback)
///
/// # Examples
/// ```rust
/// let ingested = ingest_file("policy.md")?;
/// let sections = extract_sections(&ingested.content)?;
/// let clauses = extract_clauses(&ingested.content)?;
///
/// let document = assemble_document(&ingested, &sections, &clauses)?;
/// assert_eq!(document.metadata.title, "Security Policy");
/// ```
pub fn assemble_document(
    ingested: &IngestedDocument,
    sections: &[SectionNode],
    clauses: &ExtractedContent,
) -> Result<PolicyDocument, ForgeError>;
```

**Contract Guarantees**:
- Returns `Ok(PolicyDocument)` with complete data or fallback values
- Never panics; all errors return `Err(ForgeError)`
- Malformed YAML → warns to stderr, returns Ok with fallback metadata (SEC-1)
- Empty document → returns Ok with empty sections (SEC-2)
- Missing frontmatter → returns Ok with heading/filename fallback (SEC-3)
- All source_line fields preserved from extraction inputs (SEC-4)
- No silent data loss; all sections and requirements assembled (SEC-5)

**Error Handling** (per Clarification Q4 and Assumption A-8):
- Recoverable issues: emit warning to stderr via `eprintln!`, return Ok
- Fatal issues: return `Err(ForgeError::Parse(...))`

---

## Module: `model::frontmatter.rs` (Internal)

### parse_frontmatter (Internal function)

```rust
/// Parse YAML frontmatter from document content.
///
/// Frontmatter format:
/// ```yaml
/// ---
/// title: "Policy Title"
/// version: "1.0"
/// author: "Author Name"
/// date: "2026-02-11"
/// ---
/// ```
///
/// # Arguments
/// * `content` - Full document content as UTF-8 string
///
/// # Returns
/// * `Some(FrontmatterData)` - Successfully parsed frontmatter
/// * `None` - No frontmatter present, or parsing failed (warning emitted)
///
/// # Security
/// - Frontmatter region bounded to prevent excessive parsing (SEC F1 - optional)
/// - serde_yaml handles YAML bombs and recursive anchors
/// - Malformed YAML causes warning, not panic (SEC-1, SEC-6)
fn parse_frontmatter(content: &str) -> Option<FrontmatterData>;
```

**Internal Contract**: Not part of public API; may be refactored without breaking downstream WIs.

---

## Module: `model::assemble.rs` (Internal)

### map_sections (Internal function)

```rust
/// Convert SectionNode tree to PolicySection tree, associating requirements.
///
/// Associates extracted list items with sections by line range heuristic:
/// - Requirement belongs to section if its source_line falls within the section's range
/// - Section range: [section.source_line, next_sibling.source_line) or document end
///
/// # Arguments
/// * `section_nodes` - Section hierarchy from WI-3
/// * `list_items` - Borrowed list item references from WI-4
///
/// # Returns
/// * `Vec<PolicySection>` - Mapped sections with nested requirements
fn map_sections(
    section_nodes: &[SectionNode],
    list_items: &[&ExtractedListItem],
) -> Vec<PolicySection>;
```

**Internal Contract**: Not part of public API; implementation details may change.

---

## Derived Traits

All public structs derive:
- `Debug`: For debugging and error messages
- `Clone`: For data flow through pipeline (enables owned transformations)

**Future Consideration** (Not in WI-5 scope):
- `Serialize` / `Deserialize`: May add for debugging/testing in later WIs
- `PartialEq` / `Eq`: May add for testing in later WIs

---

## Implementation Guardrails (from AR)

The following guardrails from `docs/AR/005-ar-domain-model.md` **MUST** be followed:

### MUST DO

- ✅ **Use `Option` for fields populated by later WIs** (stable_id, citations)
- ✅ **Derive `Debug` and `Clone` on all domain model structs** (PRD S-1)
- ✅ **Fall back to first H1 heading for title if no frontmatter** (PRD M-4, AC-3)
- ✅ **Default version to "0.0.0" when not in frontmatter** (PRD AC-3)
- ✅ **Preserve source line numbers on all sections and requirements** (PRD M-6)
- ✅ **Write tests before implementation (TDD)** (Constitution principle IV)

### MUST NOT DO

- ❌ **DO NOT include OSCAL-specific fields** in domain model (no `group_id`, `control_id`, `oscal_version`)
- ❌ **DO NOT generate stable UUIDs** — leave `stable_id` as None; WI-7 populates this
- ❌ **DO NOT extract citations** — that is WI-8
- ❌ **DO NOT atomize compound statements** — that is WI-6
- ❌ **DO NOT create traits for domain model structs** — plain structs only
- ❌ **DO NOT use `unwrap()` on serde_yaml deserialization** (SEC-6)

---

## Security Requirements (from SEC)

The following security requirements from `docs/SEC/005-sec-domain-model.md` **MUST** be satisfied:

| Req ID | Requirement | Verification |
|--------|-------------|--------------|
| SEC-1 | YAML frontmatter parsing must be fault-tolerant: malformed YAML → warning + fallback, not error | Unit Test: `tests/unit/frontmatter_test.rs` |
| SEC-2 | Empty documents must produce valid PolicyDocument, not error | Unit Test: `tests/unit/model_test.rs` |
| SEC-3 | Documents with no frontmatter/headings must use sensible defaults (filename, "0.0.0") | Unit Test: `tests/unit/frontmatter_test.rs` |
| SEC-4 | All PolicySection and PolicyRequirement must preserve source_line from extraction | Unit Test: `tests/unit/assemble_test.rs` |
| SEC-5 | Assembly must not silently drop sections or requirements | Unit Test: `tests/unit/assemble_test.rs` |
| SEC-6 | Do not use `unwrap()` on serde_yaml deserialization — handle errors gracefully | Code Review |
| SEC-7 | Domain model structs must use `Option<T>` for fields populated by later WIs | Code Review |

---

## Breaking Change Policy

**What constitutes a breaking change**:
- Removing or renaming public struct fields
- Changing field types (e.g., `String` → `Option<String>`, `Vec<T>` → `HashMap<K, T>`)
- Changing `assemble_document` signature (parameters or return type)
- Removing derives (`Debug`, `Clone`)

**How to introduce breaking changes**:
1. Consult dependent WIs (WI-6, WI-7, WI-8, WI-9+)
2. Create migration plan (e.g., deprecation period, adapter functions)
3. Update AR document with decision record

**Non-breaking changes** (additive only):
- Adding new optional fields (`Option<T>`) to structs
- Adding new derives (`Serialize`, `PartialEq`)
- Adding new internal helper functions
- Improving error messages

---

## Testing Contract

Downstream WIs can depend on:
- `PolicyDocument` being constructible via `assemble_document`
- All fields being publicly accessible
- `Debug` and `Clone` being available for test assertions
- `source_line` being accurate for traceability tests

Example test pattern for downstream WIs:

```rust
#[test]
fn test_downstream_enrichment() {
    let document = assemble_document(&ingested, sections, clauses)?;
    assert_eq!(document.sections.len(), 3);

    // Clone for owned transformation
    let enriched = enrich_with_uuids(document);
    assert!(enriched.sections[0].requirements[0].stable_id.is_some());
}
```

---

## Next Steps

Proceed to quickstart.md generation (Phase 1 continued).

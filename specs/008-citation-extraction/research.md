# Research: Citation and Reference Extraction (WI-8)

## R-1: Citation ID Generation Scheme

**Decision**: Use UUID v5 with `FORGE_NAMESPACE_UUID` namespace, hashing `"{requirement_stable_id}:{citation_text}"`.

**Rationale**: Consistent with WI-7's `generate_stable_id` pattern. UUID v5 is deterministic (same input → same UUID), which satisfies idempotency requirement S-3. Including the requirement's stable_id in the hash input ensures that identical citation text in different requirements produces different citation IDs (each citation is scoped to its source requirement).

**Alternatives considered**:
- UUID v4 (random): Rejected — breaks idempotency (S-3) since re-running extraction would produce different IDs.
- Sequential index: Rejected — fragile, not globally unique, breaks if requirement order changes.
- SHA-256 hash: Rejected — preliminary_id pattern exists in atomize.rs but WI-7 replaced it with UUID v5 for all stable identifiers.

**Implementation**: Create `generate_citation_id(requirement_id: &str, citation_text: &str) -> String` that hashes `"{requirement_id}:{citation_text}"` via UUID v5 with `FORGE_NAMESPACE_UUID`.

---

## R-2: Citation Struct — `validated` Field

**Decision**: Do NOT add a `validated: bool` field to the `Citation` struct.

**Rationale**: The existing `Citation` struct (defined in `model/mod.rs`, consumed by `oscal/back_matter.rs`) was designed without `validated`. The back matter module's `classify_url` function performs URL validation at OSCAL serialization time, producing `Prop { name: "url-status", value: "unvalidated" }` for malformed URLs. Adding `validated` to Citation would be redundant and would break the existing back_matter API.

The PRD M-5 requirement ("preserve with a flag indicating it is unvalidated") is satisfied by the OSCAL prop annotation generated downstream — the "flag" is the OSCAL property, not a field on the domain model. This maintains the domain model's format-agnosticism (no OSCAL-specific fields on domain types).

**Alternatives considered**:
- Add `validated: bool` per PRD interface contract: Rejected — would duplicate logic already in back_matter.rs and require changes to existing WI-12 code.

---

## R-3: Enrichment Function Signature

**Decision**: Use `extract_citations(document: &mut PolicyDocument)` (in-place mutation).

**Rationale**: Follows the precedent set by WI-7's `assign_stable_ids(&mut PolicyDocument)`. Both are enrichment passes that modify existing fields rather than restructuring the document. The `&mut` pattern avoids unnecessary cloning of the entire document tree.

Note: The lower-level function `extract_citations_from_text` uses a functional return `(String, Vec<Citation>)`, keeping the core logic pure. The top-level `extract_citations` is the imperative shell.

**Alternatives considered**:
- Functional return `(&PolicyDocument) -> Result<PolicyDocument, ForgeError>`: Used by atomize_document (WI-6), but atomization restructures the document (splits requirements), so a new document makes sense. Citation extraction only modifies existing fields, making `&mut` more appropriate.

---

## R-4: `PolicyRequirement.citations` Field

**Decision**: Add `pub citations: Vec<Citation>` to `PolicyRequirement`, defaulting to `vec![]`.

**Rationale**: Citations are attached to their source requirement (PRD M-4). The `Vec<Citation>` is empty initially and populated by WI-8. This is consistent with how `stable_id: Option<String>` is populated by WI-7.

**Breaking change impact**: All test helpers that construct `PolicyRequirement` instances must be updated to include `citations: vec![]`. Affected files:
- `src/model/mod.rs` (test helpers and tests)
- `src/parse/atomize.rs` (test helpers)
- `src/parse/clauses.rs` (test helpers)
- `src/uuid.rs` (test helpers)
- `src/model/assemble.rs`
- `src/oscal/catalog.rs`
- `src/oscal/parts.rs`
- `benches/*.rs`

---

## R-5: Overlapping Pattern Matches

**Decision**: Process patterns in priority order: URLs first, then bibliographic references, then cross-references. Track matched byte ranges and skip any match that overlaps with an already-extracted citation.

**Rationale**: URL patterns are the most precise and unambiguous. A bibliographic reference like "NIST SP 800-53 at https://nvd.nist.gov/800-53" should extract the URL as one citation and "NIST SP 800-53" as a separate bibliographic citation — they are semantically distinct. Only skip if byte ranges literally overlap.

---

## R-6: Citation Type Discriminator

**Decision**: Do NOT add a `type` or `kind` field to Citation.

**Rationale**: Citation type can be inferred from the `url` field:
- `url: Some(...)` → URL-based citation
- `url: None` → bibliographic or cross-reference

The C-2 requirement (summary log by type) can be satisfied by counting matches per pattern type in the extraction function and logging via `tracing`.

---

## R-7: Scheme-less URL Detection (EC-3)

**Decision**: Add a secondary regex pattern for scheme-less URLs (`www.` prefix). Scheme-less URLs are extracted as Citations with `url: Some("www.example.com/...")` — the back_matter module's `classify_url` will classify them as malformed and annotate with `url-status: "unvalidated"`.

**Rationale**: EC-3 explicitly requires detecting "www.example.com/policy" as a malformed URL citation. The primary URL regex won't match these. A secondary pattern catches common scheme-less URLs.

---

## R-8: Regex Patterns — Best Practices

**Decision**: Use `std::sync::LazyLock<Regex>` for all compiled patterns (consistent with atomize.rs).

**Patterns**:
- **URL**: `https?://[^\s\)\]>,;]+` — matches http/https URLs
- **Scheme-less URL**: `\bwww\.[^\s\)\]>,;]+` — matches www-prefixed URLs
- **Bibliographic**: `\b(?:NIST\s+SP|ISO|RFC|FIPS)\s+[\d]+[-\w.]*(?:\s+Rev\.?\s*\d+)?(?:,?\s+Section\s+[\w.-]+)?` — matches standard names with optional revision and section
- **Cross-reference**: `\b(?:Section|Appendix|Table)\s+[\dA-Z]+(?:\.\d+)*\b` — capital letter required

**Security**: All patterns use Rust `regex` crate (RE2-style, linear-time guarantee). No PCRE features. Per SEC-1.

---

## R-9: Prose Cleanup After Stripping

**Decision**: After stripping each citation match from the text:
1. Replace matched text with a single space
2. Collapse multiple consecutive spaces to one space
3. Remove leading/trailing whitespace
4. Handle common punctuation artifacts (double commas, trailing commas before periods)

**Rationale**: PRD M-2 requires "clean prose suitable for OSCAL control statements." EC-2 specifically calls out whitespace normalization.

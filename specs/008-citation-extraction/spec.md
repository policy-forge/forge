# Feature Specification: Citation and Reference Extraction

**Feature Branch**: `008-citation-extraction`
**Created**: 2026-02-12
**Status**: Draft
**Input**: Derived from docs/PRD/008-prd-citation-extraction.md (WI-8)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Extract Inline URLs from Requirement Text (Priority: P1)

A policy requirement contains one or more inline URLs that should be extracted and stored as citations, leaving clean prose suitable for OSCAL control statements.

> As a developer working on FORGE, I want inline URLs in requirement text to be detected and extracted into Citation objects so that OSCAL back matter can reference them as structured resources.

**Why this priority**: Inline URLs are the most common and unambiguous citation type. Extracting them is essential to satisfy parent PRD requirement M-9 and enables all downstream OSCAL resource generation.

**Independent Test**: Parse a requirement containing an inline URL, run citation extraction, and verify the URL appears in a Citation object and is removed from the prose.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text "Access must comply with https://example.com/policy requirements", **When** citation extraction runs, **Then** a Citation is created with url = "https://example.com/policy" and the requirement text becomes "Access must comply with requirements" (URL stripped, whitespace normalized).
2. **Given** a PolicyRequirement with multiple URLs in text, **When** citation extraction runs, **Then** one Citation is created per URL and all URLs are stripped from the prose.

*(Traces to: M-1, M-2, M-4 | Acceptance Criteria: AC-1, AC-2, AC-3, AC-5)*

---

### User Story 2 — Extract Bibliographic References (Priority: P1)

A policy requirement references external standards or documents by name that should be extracted as citations.

> As a compliance engineer, I want bibliographic references (e.g., "NIST SP 800-53 Rev 5") extracted into Citation objects so that they appear as named resources in OSCAL back matter rather than unstructured inline text.

**Why this priority**: Bibliographic references are fundamental to compliance documents. Extracting them enables structured linking in OSCAL output and is critical for compliance traceability.

**Independent Test**: Parse a requirement referencing "NIST SP 800-53 Rev 5", run citation extraction, and verify a Citation object captures the reference text.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text "Controls shall align with NIST SP 800-53 Rev 5, Section AC-2", **When** citation extraction runs, **Then** a Citation is created with text = "NIST SP 800-53 Rev 5, Section AC-2" and the reference is stripped from the prose.
2. **Given** a requirement referencing multiple standards, **When** citation extraction runs, **Then** each standard produces a separate Citation object.

*(Traces to: S-1 | Acceptance Criteria: AC-6)*

---

### User Story 3 — Handle Malformed URLs Gracefully (Priority: P2)

A policy requirement contains a scheme-less URL (e.g., www.example.com) that should be detected and preserved as a citation for downstream validation.

> As a developer working on FORGE, I want scheme-less URLs (e.g., www.example.com) to be detected and preserved as citations so that no data is lost, with downstream back_matter (WI-12) classifying them via OSCAL prop annotations.

**Why this priority**: Per parent PRD EC-7, scheme-less URLs must be preserved (not silently dropped). The back_matter module (WI-12) classifies URLs via `classify_url` and annotates unvalidated ones with OSCAL properties. Data loss is unacceptable for compliance tooling.

**Independent Test**: Parse a requirement with "www.example.com/policy", run citation extraction, and verify a Citation is created with `url: Some("www.example.com/policy")`.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text containing "See www.example.com/policy for details", **When** citation extraction runs, **Then** a Citation is created with `url = Some("www.example.com/policy")` (downstream back_matter classifies as unvalidated via OSCAL prop).
2. **Given** a requirement with a scheme-less URL alongside a full URL, **When** citation extraction runs, **Then** both are extracted as separate Citations — the full URL with its scheme, the scheme-less URL with `url: Some("www....")` for downstream classification.

*(Traces to: M-5 | Acceptance Criteria: AC-4)*

---

### User Story 4 — Detect Cross-References Between Sections (Priority: P2)

A policy requirement references another section within the same document.

> As a developer working on FORGE, I want internal cross-references (e.g., "See Section 3.2") detected and stored as Citation objects so that OSCAL link elements can be generated downstream.

**Why this priority**: Cross-references are common in policy documents and enable internal linking in OSCAL output, but they are lower priority than external URLs and bibliographic references.

**Independent Test**: Parse a requirement containing "See Section 3.2", run citation extraction, and verify a Citation is created capturing the cross-reference.

**Acceptance Scenarios**:

1. **Given** a PolicyRequirement with text "This control supplements Section 3.2 requirements", **When** citation extraction runs, **Then** a Citation is created with text = "Section 3.2" and no URL (internal reference).
2. **Given** a requirement referencing "Appendix A" or "Table 2", **When** citation extraction runs, **Then** Citations are created for each cross-reference.

*(Traces to: S-2 | Acceptance Criteria: AC-7)*

---

### User Story 5 — Pipeline Enrichment Function (Priority: P1)

Citation extraction operates as a pipeline enrichment step that processes an entire document.

> As a developer working on FORGE, I want citation extraction to be a single enrichment function that takes a PolicyDocument and returns an enriched PolicyDocument with all citations extracted and prose cleaned across every requirement.

**Why this priority**: The pipeline function is the integration point — without it, individual extraction logic cannot be applied to documents in the conversion pipeline.

**Independent Test**: Pass a PolicyDocument with multiple requirements containing various citation types, run the enrichment function, and verify all requirements are processed with citations attached and prose cleaned.

**Acceptance Scenarios**:

1. **Given** a PolicyDocument with multiple requirements containing citations of various types, **When** the enrichment function runs, **Then** all requirements are processed, citations are attached to their source requirements, and prose is cleaned.
2. **Given** a PolicyDocument with zero requirements, **When** the enrichment function runs, **Then** no error occurs and the document is returned unchanged.

*(Traces to: M-6 | Acceptance Criteria: AC-5)*

---

### Edge Cases

- **EC-1** (M-1): When a requirement contains no citations, URLs, or references, then the text is unchanged and no Citations are created.
- **EC-2** (M-2): When stripping a URL leaves awkward whitespace or punctuation (e.g., double spaces, trailing commas), then the prose is normalized (extra whitespace collapsed).
- **EC-3** (M-5): When a URL is missing its scheme (e.g., "www.example.com/policy"), then it is extracted as a Citation with `url: Some("www.example.com/policy")` for downstream back_matter classification.
- **EC-4** (M-1): When a URL appears in parentheses (e.g., "(https://example.com)"), then the URL is extracted without the surrounding parentheses.
- **EC-5** (M-1): When the same URL appears multiple times in one requirement, then each occurrence produces a separate Citation (deduplication is deferred to WI-12).
- **EC-6** (S-2): When text contains a partial cross-reference pattern that is ambiguous (e.g., "section" in lowercase without a number), then it is not extracted (conservative matching).
- **EC-7** (M-6): When citation extraction is run on a document with zero requirements, then no error occurs and the document is returned unchanged.

## Requirements *(mandatory)*

### Functional Requirements

**Must Have (M) — MVP, launch blockers:**

- **M-1**: The system MUST detect inline URLs (http://, https://) in PolicyRequirement text and extract them into Citation objects. *(Traces to: Parent PRD M-9)*
- **M-2**: The system MUST strip extracted citation text from PolicyRequirement text, producing clean prose suitable for OSCAL control statements. *(Traces to: Parent PRD M-9)*
- **M-3**: Each Citation MUST include fields for: unique identifier, link to the source requirement, citation display text, and optional URL. *(Traces to: Parent PRD M-9, data model)*
- **M-4**: Each extracted Citation MUST be linked to the PolicyRequirement from which it was extracted. *(Traces to: Parent PRD M-9)*
- **M-5**: When a scheme-less URL is detected (e.g., www.example.com), the system MUST preserve it as a Citation with `url: Some(matched_text)` for downstream back_matter classification via OSCAL prop annotations. *(Traces to: Parent PRD EC-7)*
- **M-6**: Citation extraction MUST be implemented as a pipeline enrichment function that takes a PolicyDocument and returns an enriched PolicyDocument with citations populated. *(Traces to: Parent PRD M-9)*

**Should Have (S) — High value, not blocking:**

- **S-1**: The system SHOULD detect bibliographic references to well-known standards (NIST SP, ISO, RFC, FIPS) and extract them as Citation objects with descriptive text.
- **S-2**: The system SHOULD detect internal cross-references (e.g., "Section X.Y", "Appendix X", "Table N") and extract them as Citation objects without a URL.
- **S-3**: The citation extraction function SHOULD be idempotent — running it twice on the same document produces the same result.

**Could Have (C) — Nice to have, if time permits:**

- **C-1**: The system COULD detect Markdown-style links (`[text](url)`) and extract both the display text and URL into the Citation.
- **C-2**: The system COULD provide a summary log (count of citations extracted by type: URL, bibliographic, cross-reference) for CLI output.

**Won't Have (W) — Explicitly deferred:**

- **W-1**: OSCAL back matter resource generation — *Deferred to WI-12 (Back Matter & Link Patterns)*
- **W-2**: Citation deduplication across requirements — *Deferred to WI-12 when assembling back matter*
- **W-3**: URL reachability validation — *Out of scope; FORGE does not perform network requests during conversion*
- **W-4**: NLP-based citation detection — *Regex/pattern matching is sufficient for MVP; ML deferred to future phases*

### Key Entities

- **Citation**: A reference extracted from policy requirement text. Contains a unique identifier, a link to its source requirement, display text, and an optional URL. One requirement can produce zero or more citations. URL validation is handled downstream by back_matter (WI-12).
- **PolicyRequirement** (existing, extended): Gains a collection of extracted citations. After citation extraction, its text field contains clean prose with citation content removed.
- **PolicyDocument** (existing): The top-level domain model. Passed through the citation extraction enrichment step, which processes all contained requirements.

### Assumptions

- [A-1] Citation extraction operates on PolicyRequirement text after the domain model is assembled (WI-5).
- [A-2] Citation patterns can be detected via pattern matching — no NLP or ML is required at this stage.
- [A-3] The Citation collection is added to PolicyRequirement, consistent with the enrichment pattern used for stable_id in WI-7.
- [A-4] Citation extraction runs as a pipeline enrichment step, similar to atomization (WI-6) and UUID generation (WI-7).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of well-formed inline URLs in requirement text are detected and extracted into Citation objects, verified by unit tests against test fixtures.
- **SC-002**: After citation extraction, requirement prose contains no residual citation text — all extracted references are removed with whitespace normalized.
- **SC-003**: 100% of scheme-less URLs (www. prefix) are preserved as Citations with `url: Some(matched_text)` — none are silently dropped.
- **SC-004**: Every extracted Citation is linked to the source requirement it was extracted from, verified by unit tests.
- **SC-005**: Citation extraction completes within 1 second for documents with 1000+ requirements (per SEC-6).
- **SC-006**: The extraction function is idempotent — running it twice on the same document produces identical results.
- **SC-007**: Test coverage for the citation extraction module exceeds 90%.

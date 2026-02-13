# Feature Specification: OSCAL Back Matter Generation

**Feature Branch**: `012-back-matter`
**Created**: 2026-02-12
**Status**: Draft
**Input**: PRD docs/PRD/012-prd-back-matter.md (WI-12: OSCAL Back Matter)
**Depends On**: WI-8 (Citation Extraction)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Citations Appear as Back Matter Resources (Priority: P1)

A compliance engineer converts a policy document containing citations and expects every citation to appear as a structured resource in the OSCAL `back-matter` section — not buried inline in control prose or dumped into `remarks` fields.

> As a compliance engineer, I want policy citations to appear as OSCAL back matter resources so that they are queryable, linkable, and conform to NIST best practices for resource management.

**Why this priority**: Parent PRD M-9 mandates that citations be extracted into back matter as resources. This is a Must Have for MS-2 (first valid OSCAL Catalog). Without it, citation data is lost or non-conformant.

**Independent Test**: Convert a Markdown policy with 3 citations (2 URLs, 1 bibliographic reference) and verify all 3 appear in `back-matter.resources[]` with correct structure.

**Acceptance Scenarios**:

1. **Given** a policy with a citation referencing `https://nvd.nist.gov/800-53`, **When** converting to OSCAL Catalog, **Then** `back-matter.resources[]` contains a resource with an `rlinks` entry whose `href` matches that URL.
2. **Given** a policy with a bibliographic citation "NIST SP 800-53 Rev 5, Security and Privacy Controls", **When** converting, **Then** `back-matter.resources[]` contains a resource with a `citation.text` field containing the bibliographic text.
3. **Given** a policy with 5 citations, **When** converting, **Then** the back matter contains exactly 5 resources, each with a unique UUID.

---

### User Story 2 — Control Bodies Link to Back Matter Resources (Priority: P1)

A compliance engineer expects controls that reference citations to contain `link` elements pointing to the corresponding back matter resources, completing the OSCAL reference pattern.

> As a compliance engineer, I want controls to link to their referenced citations via OSCAL link elements so that the relationship between control text and supporting references is machine-readable and navigable.

**Why this priority**: Without links from controls to back matter resources, the back matter is orphaned — resources exist but nothing points to them. The link completes the OSCAL reference pattern required by M-9.

**Independent Test**: Convert a policy where control text references a citation, and verify the control contains a `link` element with `href="#<resource-uuid>"` matching the back matter resource.

**Acceptance Scenarios**:

1. **Given** a control whose source text references a citation, **When** converting, **Then** the control contains a `link` element with `rel: "reference"` and `href: "#<resource-uuid>"` pointing to the back matter resource.
2. **Given** a control that references 2 different citations, **When** converting, **Then** the control contains 2 `link` elements, each pointing to the correct back matter resource UUID.

---

### User Story 3 — No Arbitrary Data in Remarks (Priority: P1)

A compliance engineer expects generated OSCAL to comply with NIST guidance: no arbitrary data in `remarks` fields. All structured data uses `prop`, `link`, or back matter `resource` patterns.

> As a compliance engineer, I want generated OSCAL artifacts to follow NIST guidance on `remarks` usage so that the output passes both schema validation and best-practice audits.

**Why this priority**: Parent PRD M-11 explicitly mandates that the converter shall not store arbitrary data in `remarks`. This is a core compliance constraint that affects auditability.

**Independent Test**: Convert a policy with citations and structured metadata, then inspect the output for any `remarks` field containing non-human-readable content or data that should be in `prop`/`link`.

**Acceptance Scenarios**:

1. **Given** any generated OSCAL artifact with back matter, **When** inspecting all `remarks` fields, **Then** no `remarks` field contains structured data, URIs, citation text, or machine-readable metadata that should be in `prop`, `link`, or back matter.
2. **Given** citation metadata that does not fit standard OSCAL fields, **When** converting, **Then** the data is stored as `prop` annotations on the resource, not in `remarks`.

---

### Edge Cases

- **EC-1** (M-1): When a policy has zero citations, then `back-matter` is omitted entirely from the OSCAL output (valid per OSCAL schema).
- **EC-2** (M-2): When a citation URL contains query parameters or fragments, the full URL is preserved in `rlinks[].href` without modification.
- **EC-3** (M-3): When a bibliographic citation text is very long (>500 characters), it is preserved in full in `citation.text` without truncation.
- **EC-4** (M-6): When a control references a citation that was not successfully extracted (orphan reference), a warning is emitted and no broken `link` is generated.
- **EC-5** (M-4): When two citations have identical content, they produce the same deterministic UUID — this is automatic via UUID v5 determinism. C-1 (Could Have) extends this by actively detecting duplicates and merging resource entries; EC-5 merely guarantees UUID stability.
- **EC-6** (M-8): When a citation URL is an empty string or uses a non-http/https scheme, it is treated as malformed: the URL is preserved as-is in `rlinks[].href` (including `href=""` for empty strings) and annotated with `prop name="url-status" value="unvalidated"`.
- **EC-7** (M-7): When citation metadata does not fit standard OSCAL fields, it is stored as `prop` annotations, never in `remarks`.

## Requirements *(mandatory)*

### Must Have (M) — MVP, launch blockers

- **M-1**: The converter shall generate an OSCAL `back-matter` object containing a `resources[]` array from extracted citations. *(Traces to: Parent PRD M-9)*
- **M-2**: Each URL-based citation shall produce a resource with an `rlinks[]` entry containing the URL as `href`. *(Traces to: Parent PRD M-9)*
- **M-3**: Each bibliographic (non-URL) citation shall produce a resource with a `citation` object containing the reference text in `text`. *(Traces to: Parent PRD M-9)*
- **M-4**: Each back matter resource shall have a deterministic UUID generated using the WI-7 UUID v5 strategy (namespace + citation content hash). *(Traces to: Parent PRD M-8, M-9)*
- **M-5**: Each back matter resource shall have a `title` field: when citation text is available, use it as the title; for URL-only citations, use the full URL as the title. *(Traces to: Parent PRD M-9)*
- **M-6**: Control bodies that reference citations shall contain `link` elements with `rel: "reference"` and `href: "#<resource-uuid>"` pointing to the corresponding back matter resource. *(Traces to: Parent PRD M-9)*
- **M-7**: The converter shall not store arbitrary data (citation text, URLs, structured metadata) in OSCAL `remarks` fields; all such data shall use `prop`, `link`, or back matter `resource` patterns. *(Traces to: Parent PRD M-11)*
- **M-8**: When a citation URL is malformed (fails `url::Url::parse`) or uses a scheme other than `http` or `https`, the resource shall preserve the URL in `rlinks` and include a `prop` with `name: "url-status"` and `value: "unvalidated"` to flag the issue. *(Traces to: Parent PRD EC-7)*

### Should Have (S) — High value, not blocking

- **S-1**: Back matter resources should include a `description` field providing context about the citation (e.g., where in the policy it was referenced).
- **S-2**: Resources with URL-based rlinks should include a `media-type` prop when the media type can be inferred from the URL extension (e.g., `.pdf` -> `application/pdf`).

### Could Have (C) — Nice to have, if time permits

- **C-1**: The converter could detect duplicate citations (same URL or same bibliographic text) and merge them into a single back matter resource with multiple link references from controls.

### Won't Have (W) — Explicitly deferred

- **W-1**: Evidence/attachment resources (binary files, screenshots) — deferred to Phase 3 ecosystem work; requires resource storage strategy.
- **W-2**: Hash verification of referenced resources (`hash` element in rlinks) — deferred to Phase 3; requires fetching external content.
- **W-3**: XML/YAML back matter serialization — deferred to WI-26/WI-27 (output format expansion).
- **W-4**: Back matter for Component Definition artifacts — deferred to WI-14/WI-15; same pattern applies.

### Key Entities

- **BackMatter**: The top-level OSCAL structure containing a `resources[]` array of all reference materials for the catalog.
- **Resource**: An entry in `back-matter.resources[]` with a UUID, title, optional description, and either rlinks (for URLs) or a citation object (for bibliographic references). May include `prop` annotations for metadata.
- **Rlink**: A resolvable link within a resource that provides a URL (`href`) to external content, with an optional media type.
- **Citation**: A bibliographic reference within a resource containing the reference text for non-URL citations.
- **Link**: An OSCAL element in control bodies that references a back matter resource via `href="#<resource-uuid>"` with `rel: "reference"`.
- **Prop**: An OSCAL property element for structured metadata annotations (name-value pairs), used instead of `remarks` per NIST guidance.

## Assumptions

- WI-8 (citation extraction) provides `Citation` objects with text, optional URL, and a reference to the source requirement.
- WI-9 (catalog structure) provides the control skeleton into which `link` elements can be inserted.
- WI-7 (UUID generation) provides a deterministic UUID v5 strategy that can be applied to back matter resource identifiers with a dedicated namespace.
- OSCAL v1.2.0 `back-matter` schema is stable and well-documented for the resource/rlink/citation structure.
- Malformed URLs are preserved (not discarded or sanitized) and annotated with a prop flag per Parent PRD EC-7.

## Dependencies

- **Requires**: WI-8 (Citation Extraction) — provides extracted `Citation` objects as input.
- **Requires**: WI-9 (Catalog Groups & Controls) — provides the control skeleton for link insertion.
- **Parallel With**: WI-10 (Catalog Statement Parts), WI-11 (OSCAL Metadata).
- **Blocks**: WI-13 (End-to-End Catalog Pipeline).

## Clarifications

### Session 2026-02-12

- Q: Which URL schemes should be considered "valid" (no `url-status: unvalidated` annotation)? → A: Only `http` and `https` schemes are valid; all other schemes (e.g., `javascript:`, `data:`, `file:`, `ftp:`, `mailto:`) are flagged with `prop name="url-status" value="unvalidated"`.
- Q: Should `back-matter` be omitted or include an empty `resources[]` when there are zero citations? → A: Omit `back-matter` entirely when zero citations.
- Q: How should `title` be derived for URL-based citation resources (M-5)? → A: Use the full URL as the title; when a citation has both text and a URL, prefer the citation text as the title.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of extracted citations are converted to back matter resources — no citation data is lost during conversion.
- **SC-002**: 100% of control-to-citation links resolve to valid back matter resource UUIDs — no orphaned or broken links.
- **SC-003**: Zero instances of arbitrary data in `remarks` fields across all generated OSCAL artifacts.
- **SC-004**: Deterministic UUID generation produces identical resource UUIDs across repeated conversions of the same policy.
- **SC-005**: Malformed URLs are preserved in resources with an "unvalidated" annotation — no silent data loss.

### Acceptance Criteria Traceability

| AC ID | Requirement | User Story | Given                                                           | When                            | Then                                                                                                                  |
| ----- | ----------- | ---------- | --------------------------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| AC-1  | M-1, M-2    | US-1       | A policy with 2 URL citations                                   | Converting to OSCAL Catalog     | `back-matter.resources[]` contains 2 resources, each with `rlinks[].href` matching the URLs                           |
| AC-2  | M-1, M-3    | US-1       | A policy with 1 bibliographic citation ("NIST SP 800-53 Rev 5") | Converting to OSCAL Catalog     | `back-matter.resources[]` contains 1 resource with `citation.text` = "NIST SP 800-53 Rev 5"                           |
| AC-3  | M-4         | US-1       | Same policy converted twice                                      | Comparing back matter UUIDs     | Resource UUIDs are identical across runs                                                                               |
| AC-4  | M-5         | US-1       | A citation with title text                                       | Converting                      | The back matter resource `title` field is populated                                                                    |
| AC-5  | M-6         | US-2       | A control referencing a citation                                 | Converting                      | The control contains a `link` with `rel: "reference"` and `href: "#<resource-uuid>"` matching the back matter resource |
| AC-6  | M-6         | US-2       | A control referencing 2 citations                                | Converting                      | The control contains 2 `link` elements with correct `href` values                                                     |
| AC-7  | M-7         | US-3       | Any generated OSCAL artifact with back matter                    | Inspecting all `remarks` fields | No `remarks` field contains citation text, URLs, or structured metadata                                                |
| AC-8  | M-8         | US-1       | A citation with a malformed URL (e.g., "htp://bad url")          | Converting                      | The resource contains the URL in `rlinks` and a `prop` with `name: "url-status"`, `value: "unvalidated"`              |

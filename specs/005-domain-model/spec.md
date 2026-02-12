# Feature Specification: Internal Domain Model

**Feature Branch**: `005-domain-model`
**Created**: 2026-02-11
**Status**: Draft
**Input**: User description: "Read docs/PRD/005-prd-domain-model.md as the source of truth for this feature's requirements. This work item defines the internal domain model that bridges extracted Markdown structure to OSCAL concepts — the core data types for controls, groups, parameters, parts, and properties. Use the Problem Statement, User Stories, Requirements (Must/Should/Could/Won't), and Acceptance Criteria for the specification. Preserve all requirement IDs for traceability. This depends on WI-3 and WI-4. The feature branch is 005-domain-model."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Build PolicyDocument from Extracted Data (Priority: P1)

After ingestion and extraction, the pipeline assembles a complete PolicyDocument ready for OSCAL generation.

> As a developer working on FORGE, I want a well-defined domain model so that OSCAL generators can work with clean, typed data structures rather than raw extraction output.

**Why this priority**: The domain model is the central data contract. All downstream work items (WI-6 through WI-18) consume it. Without this, no OSCAL generation can proceed.

**Independent Test**: Parse a test policy document through ingestion and extraction, construct a PolicyDocument, and verify all sections and requirements are present with correct metadata.

**Acceptance Scenarios**:

1. **Given** a policy document with 3 hierarchical sections and 10 extracted requirements, **When** assembled into a PolicyDocument, **Then** the document contains exactly 3 PolicySection structures with a total of 10 PolicyRequirement structures distributed across the sections.

2. **Given** a policy document with YAML frontmatter containing title "Security Policy" and version "1.0", **When** assembled into PolicyDocument, **Then** the DocumentMetadata.title is "Security Policy" and DocumentMetadata.version is "1.0".

3. **Given** a policy document with no frontmatter but an H1 heading "Access Control Policy", **When** assembled into PolicyDocument, **Then** the DocumentMetadata.title is "Access Control Policy" and DocumentMetadata.version defaults to "0.0.0".

---

### User Story 2 — Preserve Source Traceability in Domain Model (Priority: P1)

Every domain model element preserves its source location for downstream traceability.

> As a compliance engineer, I want every extracted requirement to track its source location so that traceability links can be generated in OSCAL output.

**Why this priority**: Traceability is non-negotiable per product principle P-2. Compliance frameworks require evidence of where each control came from. Source locations must be preserved from extraction through to OSCAL generation.

**Independent Test**: Construct a PolicyDocument from a test file with known line numbers and verify each PolicyRequirement and PolicySection has a valid source_line reference that matches the source document.

**Acceptance Scenarios**:

1. **Given** requirements extracted from lines 15, 22, and 30 of the source document, **When** stored in the domain model, **Then** each PolicyRequirement.source_line equals exactly 15, 22, and 30 respectively.

2. **Given** section headings at lines 10, 20, and 35 of the source document, **When** stored in the domain model, **Then** each PolicySection.source_line equals exactly 10, 20, and 35 respectively.

---

### Edge Cases

- **EC-1**: When no frontmatter and no headings exist in the source document, then title defaults to the filename and version defaults to "0.0.0".
- **EC-2**: When a section has body text but no requirements (only descriptive content), then the section exists in the domain model with an empty requirements collection.
- **EC-3**: When the source document is structurally empty (no sections or requirements after parsing), then an empty PolicyDocument is created with default metadata.
- **EC-4**: When frontmatter is present but contains malformed YAML syntax, then a warning is emitted to stderr and metadata falls back to heading-based or default values. Assembly returns Ok(PolicyDocument) with fallback metadata.

## Clarifications

### Session 2026-02-11

- Q: How does PolicyDocument evolve through downstream WIs (WI-6 atomization, WI-7 UUID generation, WI-8 citation extraction) — in-place mutation or functional transformation? → A: Functional transformation - Each WI takes ownership and returns a new/enriched PolicyDocument (assemble → atomize → identify → cite).

- Q: How are requirements uniquely identified before WI-7 assigns stable_id, for testing and debugging purposes? → A: (source_line, text_hash) tuple provides temporary unique identity - source_line alone isn't sufficient for edge cases with duplicate lines, and a lightweight hash (e.g., first 64 chars + source_line) gives practical uniqueness for intermediate stages.

- Q: What is the expected document scale (number of sections and requirements) for typical policy documents? → A: Medium (100-1000 requirements) - based on typical security policy documents like NIST SP 800-53 (~1000 controls), ISO 27001 (~100 controls), and organizational policies (50-500 requirements). This scale supports Vec-based structures without premature optimization.

- Q: How are assembly errors and warnings surfaced to the CLI user? → A: Warnings to stderr via eprintln!, fatal errors return Err - simplest Unix-like approach. Recoverable issues (malformed YAML, missing frontmatter) emit warnings to stderr but return Ok(PolicyDocument) with fallback values. Fatal errors (data inconsistency preventing assembly) return Err(ForgeError).

- Q: What are the performance expectations for assemble_document function? → A: No specific target (development tool) - focus on correctness and simplicity. As a local CLI tool processing 100-1000 requirements, sub-second assembly will naturally result from straightforward implementation. Optimize only if profiling reveals actual bottlenecks in real usage.

## Requirements *(mandatory)*

### Functional Requirements

#### Must Have (M) — MVP, launch blockers

- **M-1**: The domain model shall include a PolicyDocument structure containing document metadata and a collection of PolicySections. *(Traces to: Parent PRD M-1)*

- **M-2**: The domain model shall include a PolicySection structure with title, heading level, source line, body text, child sections, and contained PolicyRequirements. *(Traces to: Parent PRD M-1)*

- **M-3**: The domain model shall include a PolicyRequirement structure with text content, source line number, nesting depth, and a placeholder for stable_id (populated later by WI-7). *(Traces to: Parent PRD M-1, M-2)*

- **M-4**: The domain model shall include DocumentMetadata with title and version fields, populated from YAML frontmatter when present, or from first heading as fallback. *(Traces to: Parent PRD M-5)*

- **M-5**: The assembly function shall wire ingestion output (WI-2), section tree (WI-3), and extracted clauses (WI-4) into a complete PolicyDocument without data loss. *(Traces to: Parent PRD M-1)*

- **M-6**: All domain model structures shall preserve source line numbers for traceability to the original document. *(Traces to: Parent PRD M-10)*

#### Should Have (S) — High value, not blocking

- **S-1**: PolicyDocument shall provide a human-readable summary (section count, requirement count) for CLI output.

- **S-2**: DocumentMetadata shall include optional author and date fields if present in YAML frontmatter.

#### Could Have (C) — Nice to have, if time permits

- **C-1**: The domain model could include a source_file_hash field from ingestion (WI-2) for content integrity tracking.

#### Won't Have (W) — Explicitly deferred

- **W-1**: stable_id generation — *Reason: Deferred to WI-7 (UUID generation); M-3 includes placeholder only*
- **W-2**: Citation model — *Reason: Deferred to WI-8 (citation extraction)*
- **W-3**: Modality field (normative/advisory) — *Reason: Deferred to WI-33*
- **W-4**: Parameter extraction — *Reason: Deferred to WI-34*

### Key Entities

- **PolicyDocument**: The top-level structure representing a complete parsed policy document. Contains document-level metadata and all hierarchical sections. This is the canonical representation that all downstream OSCAL generation operates against.

- **DocumentMetadata**: Metadata about the source policy document including title, version, optional author and date, and the source file path. Title and version are extracted from YAML frontmatter when available, with fallback to first heading and default version "0.0.0".

- **PolicySection**: A hierarchical section within a policy document, mapped from extracted headings. Contains title, heading level (1-6), source line number, optional body text, child sections (for hierarchical structure), and contained requirements.

- **PolicyRequirement**: An individual policy requirement extracted from list items or clause patterns. Contains text content, source line number, nesting depth (0-based), and a placeholder for stable_id (populated by WI-7). Before stable_id assignment, requirements can be temporarily identified by (source_line, text_hash) tuple for testing and debugging.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All data extracted from source documents is successfully assembled into the domain model with zero data loss (100% of sections and requirements preserved).

- **SC-002**: Every requirement and section in the domain model correctly tracks its source line number, enabling full traceability back to the original document (100% accuracy verified by tests).

- **SC-003**: Document metadata (title and version) is successfully extracted from YAML frontmatter when present, or falls back to heading-based extraction with appropriate defaults (100% success rate across test cases).

- **SC-004**: The domain model provides a clean, format-agnostic interface that decouples extraction logic from OSCAL generation, enabling independent testing of both sides (verified by ability to write unit tests for assembly without OSCAL dependencies).

- **SC-005**: Domain model structures support incremental enrichment by later work items (WI-6, WI-7, WI-8) without breaking changes to existing code (verified by optional fields for data added by downstream work items).

## Assumptions *(optional)*

- **A-1**: The domain model is serialization-agnostic — it does not depend on OSCAL JSON structure. OSCAL-specific fields are not included in this work item.

- **A-2**: Frontmatter parsing (YAML) is a standard feature that can use an existing library without custom implementation.

- **A-3**: Requirements at this stage are pre-atomization and may contain compound statements. Splitting compound requirements is deferred to WI-6.

- **A-4**: The outputs from WI-2 (ingestion), WI-3 (section extraction), and WI-4 (clause extraction) are well-formed and validated by their respective implementations.

- **A-5**: Pipeline functions follow functional transformation semantics: `assemble_document` returns an owned PolicyDocument, and downstream WIs (WI-6, WI-7, WI-8) consume ownership and return enriched instances rather than mutating in place.

- **A-6**: Before stable_id is populated by WI-7, requirements are temporarily identifiable by (source_line, text_hash) tuple for testing and debugging purposes. This temporary identity is not persisted and is not part of the domain model schema.

- **A-7**: Expected document scale is medium (100-1000 requirements, 10-100 sections) based on typical security policy documents. Vec-based data structures are appropriate for this scale without requiring indexed structures or memory optimization.

- **A-8**: Error handling follows Unix conventions: recoverable issues (malformed YAML, missing frontmatter) emit warnings to stderr via eprintln! and continue with fallback values, returning Ok(PolicyDocument). Only unrecoverable errors (data inconsistency preventing assembly) return Err(ForgeError) and halt the pipeline.

- **A-9**: No specific performance targets are defined for assembly. As a local CLI development tool processing medium-scale documents (100-1000 requirements), implementation prioritizes correctness and readability over optimization. Performance optimization is deferred until profiling identifies actual bottlenecks.

## Dependencies *(optional)*

### Requires (blocking)

- **001-prd-project-scaffolding**: Project structure must exist for module organization
- **003-prd-structural-extraction-headings**: Section tree data is input to domain model assembly
- **004-prd-structural-extraction-clauses**: Extracted clauses are input to domain model assembly

### Blocks (downstream)

- **006-prd-requirement-atomization**: Atomization operates on PolicyRequirement structures
- **008-prd-citation-extraction**: Citation extraction enriches PolicyRequirement structures
- **009-prd-catalog-groups-controls**: OSCAL generation consumes PolicyDocument structures
- All subsequent OSCAL generation work items (WI-9 through WI-18)

## Risks *(optional)*

| Risk ID | Risk Description | Likelihood | Impact | Mitigation |
|---------|------------------|------------|--------|------------|
| R-1 | Domain model needs significant changes as OSCAL generation is implemented | Medium | Medium | Design with extensibility in mind; use optional fields for data added by later WIs (stable_id, citations) |
| R-2 | Frontmatter format varies across policy documents | Low | Low | Support YAML frontmatter as primary format; fall back to first H1 for title if no frontmatter; use sensible defaults |

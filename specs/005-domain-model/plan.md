# Implementation Plan: Internal Domain Model

**Branch**: `005-domain-model` | **Date**: 2026-02-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-domain-model/spec.md`, PRD, AR, and SEC documents

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Define and implement the internal domain model that bridges extracted Markdown structure to OSCAL concepts. The domain model provides a clean, format-agnostic interface between the ingestion/extraction pipeline (WI-2, WI-3, WI-4) and downstream OSCAL generation (WI-6+). Core structures include PolicyDocument, DocumentMetadata, PolicySection, and PolicyRequirement. The assembly function wires extraction outputs into a complete PolicyDocument using functional transformation semantics (each WI takes ownership and returns enriched instances). YAML frontmatter parsing with fallback to heading-based metadata extraction provides document title and version.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: serde 1.x (serialization), serde_yaml (YAML frontmatter parsing), thiserror 2.0.18 (error handling), pulldown-cmark 0.13.x (existing from WI-3/4)
**Storage**: N/A (in-memory processing only; no persistent storage)
**Testing**: cargo test (TDD mandatory per constitution principle IV)
**Target Platform**: Local CLI tool (macOS, Linux, Windows)
**Project Type**: Single Rust library + CLI (existing FORGE structure)
**Performance Goals**: No specific targets (development tool); correctness and readability prioritized over optimization (Clarification Q5)
**Constraints**: Functional transformation semantics (ownership-based pipeline, no in-place mutation per Clarification Q1); Unix-like error handling (warnings to stderr, recoverable issues return Ok with fallbacks per Clarification Q4)
**Scale/Scope**: Medium-scale documents (100-1000 requirements, 10-100 sections typical per Clarification Q3); Vec-based structures appropriate without indexing

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: ⚠️ Constitution file (`.specify/memory/constitution.md`) is a template and has not been populated yet. The following checks are based on principles referenced in PRD and AR documents:

| Principle | Status | Notes |
|-----------|--------|-------|
| **TDD Mandatory** (Referenced in PRD Technical Constraints) | ✅ PASS | Tests required before implementation; comprehensive unit test strategy documented in AR |
| **YAGNI / Simplicity** (Referenced in AR Constitution Principle X) | ✅ PASS | Plain struct hierarchy (Option 1) selected; trait-based and enum-based approaches rejected as over-engineering |
| **No traits with single implementation** (Referenced in AR) | ✅ PASS | Domain model uses plain structs with derives; no trait abstractions |
| **Contract-first** (Referenced in AR Driving Requirements) | ✅ PASS | Struct definitions and interface contracts specified in AR before implementation |
| **Decoupling** (Referenced in PRD design constraint) | ✅ PASS | Domain model decoupled from both extraction types and OSCAL structure |

**Recommendation**: Populate `.specify/memory/constitution.md` using `/speckit.constitution` command to formalize project principles.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── model/                  # THIS FEATURE: Domain model structs
│   ├── mod.rs             # PolicyDocument, DocumentMetadata, PolicySection, PolicyRequirement + inline unit tests
│   ├── assemble.rs        # assemble_document, map_sections + inline unit tests
│   └── frontmatter.rs     # YAML frontmatter parsing with fallback + inline unit tests
├── ingest/                # FROM WI-2: File ingestion
│   └── mod.rs             # IngestedDocument type
├── parse/                 # FROM WI-3/4: Structural extraction
│   ├── mod.rs             # SectionNode type, extract_sections (WI-3)
│   └── clauses.rs         # ExtractedContent type, extract_clauses (WI-4)
├── cli/                   # CLI entry point
│   └── convert.rs         # Wires pipeline (calls assemble_document)
├── error.rs               # ForgeError type (thiserror)
└── lib.rs                 # Crate root

tests/
└── pipeline_test.rs       # Integration test: ingest → extract → assemble
```

**Structure Decision**: Single Rust library + CLI (existing FORGE structure). The `model/` module is the new component added by this feature. It consumes types from `ingest/` and `parse/` modules (WI-2, WI-3, WI-4) and produces `PolicyDocument` for downstream WIs (WI-6+). No new top-level directories required.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

**Status**: ✅ No constitution violations requiring justification. The domain model follows YAGNI principles (plain structs, no trait abstractions), contract-first design (structs specified before implementation), and TDD mandatory approach.

---

## Phase 0: Research (Complete)

**Status**: ✅ Complete

All technical unknowns were resolved during the formal clarification workflow (see `spec.md` Clarifications section). The research phase documented architectural decisions and their rationale.

**Generated Artifacts**:
- ✅ `research.md` - Documents all architectural decisions:
  - Decision 1: Plain struct hierarchy with Option fields (AR Option 1)
  - Decision 2: Functional transformation ownership semantics
  - Decision 3: (source_line, text_hash) for temporary requirement identity
  - Decision 4: Medium-scale documents (100-1000 requirements)
  - Decision 5: Unix-like error handling (warnings to stderr)
  - Decision 6: No specific performance targets
  - Decision 7: serde_yaml with fault-tolerant parsing

**Key Findings**:
- All "NEEDS CLARIFICATION" items resolved before research phase
- Technology stack confirmed: Rust + serde + serde_yaml + thiserror
- Architecture review completed with Option 1 selected (plain structs)
- Security review completed with 7 security requirements identified

---

## Phase 1: Design & Contracts (Complete)

**Status**: ✅ Complete

**Generated Artifacts**:
- ✅ `data-model.md` - Complete entity specifications:
  - PolicyDocument, DocumentMetadata, PolicySection, PolicyRequirement structs
  - assemble_document function specification
  - Entity relationship diagram
  - Validation rules and state transitions
  - Data flow diagrams
  - Mapping from extraction types
  - Testing strategy
  - Security considerations

- ✅ `contracts/rust-interfaces.md` - Rust module interface contracts:
  - Public struct definitions with full documentation
  - assemble_document function contract with guarantees
  - Implementation guardrails from AR (MUST DO / MUST NOT DO)
  - Security requirements from SEC (SEC-1 through SEC-7)
  - Breaking change policy
  - Testing contract for downstream WIs

- ✅ `quickstart.md` - Developer guide:
  - Understanding the domain model's role in pipeline
  - Core structure reference
  - Assembling a PolicyDocument (code examples)
  - Working with the domain model (traversal, access patterns)
  - Writing tests (unit and integration examples)
  - Extending for downstream WIs (WI-6, WI-7, WI-9)
  - Common patterns and troubleshooting

- ✅ Agent context updated: `CLAUDE.md`
  - Added Rust (edition 2024, stable 1.93.0)
  - Added serde 1.x, serde_yaml, thiserror 2.0.18, pulldown-cmark 0.13.x
  - Added N/A for storage (in-memory processing only)

**Constitution Check Re-evaluation** (Post-Design):
All gates still pass:
- ✅ TDD Mandatory: Comprehensive test strategy documented
- ✅ YAGNI / Simplicity: Plain structs confirmed, no over-engineering
- ✅ No traits with single implementation: No traits in design
- ✅ Contract-first: All interfaces specified before implementation
- ✅ Decoupling: Domain model isolated from both extraction and OSCAL

---

## Next Steps

**Phase 2**: Task Decomposition
- Command: `/speckit.tasks`
- Input: This implementation plan + data model + contracts
- Output: `tasks.md` with dependency-ordered implementation tasks

**After Planning**: Implementation
- Command: `/speckit.implement` (executes tasks.md)
- Follow TDD approach: Write tests → Get approval → Implement
- Adhere to Implementation Guardrails from AR
- Satisfy all Security Requirements from SEC

---

## References

- [Feature Specification](./spec.md) - User stories, requirements, success criteria
- [Product Requirements Document](../../docs/PRD/005-prd-domain-model.md) - Detailed MoSCoW requirements
- [Architecture Review](../../docs/AR/005-ar-domain-model.md) - Architecture decisions and guardrails
- [Security Review](../../docs/SEC/005-sec-domain-model.md) - Security requirements and threat model
- [Research Findings](./research.md) - Architectural decision rationale
- [Data Model](./data-model.md) - Entity specifications
- [Rust Interfaces](./contracts/rust-interfaces.md) - Public API contracts
- [Quickstart Guide](./quickstart.md) - Developer onboarding

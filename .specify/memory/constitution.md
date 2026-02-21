# FORGE Constitution

## Core Principles

### I. Crate-First Architecture

- All feature work MUST be implemented within the existing Rust crate layout unless a new crate is explicitly justified in the feature plan.
- New modules MUST have a clear runtime or test responsibility tied to documented requirements.

### II. Rust-First Implementation

- Production and test implementations MUST use stable Rust.
- `unsafe` code MUST NOT be introduced without an explicit architecture justification and reviewer sign-off.

### III. Contract-First Development

- A feature MUST define or update interface contracts before implementation tasks are finalized.
- Contracts MUST align with the actual implementation style (for example: Rust helper contracts for Rust test harness work).

### IV. Test-First Development (Non-Negotiable)

- For each new behavior, tests MUST be authored before implementation is considered complete.
- Implementation is done when targeted tests pass and regression tests remain green.

### V. Complete Requirement Delivery

- All Must-Have requirements in the active feature scope MUST be covered by executable tasks.
- A feature is not complete if a baseline acceptance criterion has no verification task.

### VI. Performance and Scope Discipline

- Performance targets MUST be measurable when performance is in scope.
- Features explicitly marked out-of-scope for performance benchmarking MUST not add speculative benchmark work.

### VII. Security-First Design

- Security requirements from SEC artifacts MUST be mapped to tasks when applicable.
- If SEC marks work as test-only/no attack surface, the plan MUST still preserve secure defaults (local fixtures, no secret handling, no network dependence).

### VIII. Error Handling Standards

- User-facing failures MUST be actionable and testable.
- Assertions against failure text SHOULD favor stable substrings over brittle full-message equality when wording may evolve.

### IX. Observability and Debuggability

- Test and runtime behavior MUST remain diagnosable through deterministic artifacts (snapshots, fixtures, structured outputs, and clear diagnostics).
- Silent failure handling is prohibited.

### X. Simplicity and Pragmatism

- Extend existing harnesses and patterns before introducing new frameworks.
- Scope expansion MUST be explicit and justified by requirements.

### XI. Dependency Policy

- New dependencies MUST be avoided unless required by documented requirements and approved in plan artifacts.
- Existing dependencies SHOULD be reused when they satisfy feature needs.

## Additional Constraints

- Required quality gates for implementation-ready work: `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check`.
- Feature artifacts (`spec.md`, `plan.md`, `tasks.md`) MUST stay mutually consistent on scope, strategy coverage, and acceptance behavior.

## Development Workflow

1. Clarify requirements and resolve high-impact ambiguities.
2. Produce implementation plan and design artifacts.
3. Generate tasks with explicit requirement coverage.
4. Implement with test-first execution and pass all quality gates.

## Governance

- This constitution governs all feature-level specs, plans, tasks, and implementation decisions in this repository.
- Constitution conflicts are resolved by updating feature artifacts to comply; weakening constitutional rules requires a separate explicit constitution amendment.
- Amendments MUST include rationale, affected principles, and migration impact on active features.

**Version**: 1.0.0 | **Ratified**: 2026-02-21 | **Last Amended**: 2026-02-21

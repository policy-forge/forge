# Research: Project Scaffolding

**Feature**: 001-project-scaffolding
**Date**: 2026-02-11
**Status**: Complete — no NEEDS CLARIFICATION items in Technical Context

## Overview

All technology choices for project scaffolding are pre-determined by the project constitution and architecture review (001-ar-project-scaffolding). No research spikes or unknowns exist. This document records the ratified decisions and best practices for reference during implementation.

## Decisions

### D-1: CLI Framework — clap 4.x with Derive Macros

**Decision**: Use clap 4.x with derive macros for CLI argument parsing.
**Rationale**: Specified in the constitution technology stack (Clap 4.x). Derive macros reduce boilerplate, are type-safe, and auto-generate help text from struct annotations. The AR evaluated builder pattern as an alternative and rejected it (more verbose, less type-safe).
**Alternatives considered**:
- clap builder pattern — rejected: more verbose, less type-safe
- argh — rejected: less feature-rich, not in constitution tech stack
- structopt — rejected: merged into clap 4.x derive

**Best practices for clap derive in CLI scaffolding**:
- Define `Cli` struct with `#[derive(Parser)]` and `#[command(...)]` attributes
- Use `#[derive(Subcommand)]` enum for `convert` and `validate` commands
- Use `#[derive(ValueEnum)]` for strategy and format argument enums
- Let clap auto-generate help text and error messages from annotations
- Place CLI definitions in `src/cli/mod.rs`, separate handler files per subcommand

### D-2: Error Handling — thiserror

**Decision**: Use thiserror for structured, composable error types in a single `ForgeError` enum.
**Rationale**: Specified in the constitution (principle VIII). thiserror derives `Display` and `Error` traits from enum annotations, producing clear error messages. A single top-level enum avoids premature abstraction.
**Alternatives considered**:
- anyhow — rejected for library code (constitution: "anyhow ONLY in binary crates and tests")
- Custom Error impl — rejected: unnecessary boilerplate when thiserror handles it
- Per-module error enums — rejected: premature abstraction for scaffolding with only stubs

**Best practices for thiserror in project scaffolding**:
- Define `ForgeError` in `src/error.rs` with `#[derive(Debug, thiserror::Error)]`
- Use `#[error("...")]` for Display formatting on each variant
- Use `#[from]` for automatic conversion from `std::io::Error`
- Provide at least: `Io`, `Parse`, `Validation`, `Config` variants
- Re-export from `lib.rs` for ergonomic imports

### D-3: Project Structure — Single Crate with Flat Modules

**Decision**: Single Cargo crate with 7 top-level modules matching pipeline stages.
**Rationale**: AR Option 1 (selected). Simplest approach meeting all PRD requirements (M-1 through M-6). Workspace extraction deferred to milestone boundary when justified by real code.
**Alternatives considered**:
- Cargo workspace from day one (AR Option 2) — rejected: over-engineering for sprint 1 with only stub modules

### D-4: CI Pipeline — GitHub Actions

**Decision**: GitHub Actions CI enforcing `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`.
**Rationale**: Standard CI platform for GitHub-hosted projects. Constitution quality gates mandate these three checks from sprint 1.
**Alternatives considered**:
- None seriously considered; GitHub Actions is the default per assumptions

### D-5: Module Stub Pattern

**Decision**: Empty module stubs that compile and pass `cargo test` without `#[allow(dead_code)]`.
**Rationale**: AR anti-pattern guidance: "Don't add `#[allow(dead_code)]` to suppress warnings on stubs." Module stubs should be minimal — an empty `mod.rs` file that compiles cleanly.
**Alternatives considered**:
- `todo!()` macros in stub functions — only if a public API is needed; for pure module stubs, an empty file suffices
- `unimplemented!()` — not needed; no functions to stub yet

# FORGE support matrix — DRAFT for 2.0.0

> **Status:** DRAFT — prepared 2026-11-12 for owner review; not a published
> support commitment. Cells marked *unverified* have no exact-head evidence at
> the preparation snapshot and must be re-qualified before publication.
>
> Preparation basis: `b4226bb35d796222c30f93235ac22118dd26a344` (origin/main).
> Every claim below cites its source; dated verification receipts under
> `docs/plans/` bind to their own heads, not automatically to this one.

## Platforms

Release targets are the literal build matrix in
[`.github/workflows/release.yml`](../../.github/workflows/release.yml) (`build`
job); the same four targets are pinned in
`supply-chain/dependency-graph.json` (`targets`) and gated by
`python3 -B scripts/dependency_graph.py check`.

| Platform | Rust target | Archive | Build in release CI | Runtime qualification |
|---|---|---|---|---|
| macOS arm64 (Apple Silicon) | `aarch64-apple-darwin` | `tar.gz` | yes (`macos-latest`) | qualified on this host family; see receipts below |
| macOS x86_64 (Intel) | `x86_64-apple-darwin` | `tar.gz` | yes (`macos-latest`, cross-target) | **unverified at 2.0.0 head** — cross-built; no Intel-Mac runtime receipt found |
| Linux x86_64 (glibc) | `x86_64-unknown-linux-gnu` | `tar.gz` | yes (`ubuntu-latest`) | three-OS CI runs the suite on `ubuntu-latest`; native hosted qualification receipts exist for workspace flows (`docs/plans/2026-10-06-linux-support-root.md`); full 2.0.0-head runtime receipt pending |
| Windows x86_64 (MSVC) | `x86_64-pc-windows-msvc` | `zip` | yes (`windows-latest`) | three-OS CI runs the suite on `windows-latest`; Windows fixture receipts are host-side simulations (`docs/plans/2026-10-04-f10-epochs-windows-fixture-verification-v1.md` ran on `aarch64-apple-darwin`); native Windows qualification is an open human gate |

Not built or supported: Linux arm64/musl, Windows arm64, 32-bit targets, BSDs.
PRD 049 S-1 (Linux ARM64) is a *Should* not in the current matrix.

## Toolchain

| Item | Value | Source | Status |
|---|---|---|---|
| Declared MSRV | Rust **1.88** | `Cargo.toml` `rust-version` | declared; no MSRV CI job exists — **unverified by CI** |
| Edition | 2024 | `Cargo.toml` | — |
| CI toolchain | current stable (`dtolnay/rust-toolchain@…# stable`) | `.github/workflows/ci.yml`, `release.yml` | floats with stable |
| Recent qualification toolchain | Rust 1.99.0 (receipts, e.g. `docs/plans/2026-10-04-f10-epochs-integration-verification.md`) | dated receipts | binds to those heads only |
| Known toolchain gate | Rust 1.99.0 introduced strict-Clippy failures on unchanged source; recorded open under F22 | `docs/plans/2026-10-02-roadmap-requirement-ledger.md` (Local verification), `docs/plans/2026-10-02-current-stable-clippy.md` | **open** — must be resolved or explicitly dispositioned before release |

The preparation host ran `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; no full-suite
run at this head is claimed by this document.

## OSCAL compatibility

Source: [`docs/OSCAL_COMPATIBILITY.md`](../OSCAL_COMPATIBILITY.md) and the
pinned asset inventory it references; enforced offline by
`cargo test --test schema_provenance_test` and
`cargo test --test oscal_1_2_3_compatibility_test`.

| Item | Contract |
|---|---|
| Generated output | `metadata.oscal-version: 1.2.3` for Catalog, Component Definition, Profile, Assessment Plan, SSP (plus FORGE Mapping, POAM, Assessment Results models — `src/types.rs` `OscalModelType`) |
| Accepted input | Catalog / Component Definition / Profile declaring v1.2.0–v1.2.3, validated against the pinned v1.2.3 schema baseline; declared version never selects a historical schema and never triggers network access |
| Validation authority | embedded, vendored v1.2.3 JSON schemas (8 runtime schemas + 4 XSDs); fully offline |
| oscal-cli | optional external oracle only; v1.0.3 results are `advisory-older-model-baseline` (model baseline 1.1.2), never authoritative v1.2.3 validation |
| Round-trip | JSON/YAML → JSON/YAML preserves the decoded tree for supported artifacts (exact i64/u64 integers only; all f64 refused); XML is a partial projection |

## Input formats

Source: `src/ingest/mod.rs` (extension dispatch, case-insensitive).

| Format | Extensions | Engine | Notes |
|---|---|---|---|
| Markdown | `.md`, `.markdown` | `pulldown-cmark` | primary path |
| PDF | `.pdf` | `pdf-extract` | text extraction; layout-heavy PDFs degrade — quality **unverified per platform** |
| DOCX | `.docx` | `zip` (deflate) | text extraction from OOXML |

## Output formats

Source: `src/types.rs` (`OutputFormat`), `src/export/`.

| Format | Extension | Notes |
|---|---|---|
| JSON | `.json` | default |
| XML | `.xml` | `quick-xml`; partial projection for round-trip (see OSCAL table) |
| YAML | `.yaml` | `serde_yaml_ng` |

`forge export` auto-detects the input format from the file extension.

## Evidence still required before this matrix can be published

1. Three-OS green CI at the exact release head (`release.yml` `test` job).
2. Per-platform binary smoke receipts (PRD 050 S-3: download, `forge --help`,
   sample conversion) — human/hosted gate.
3. Native Windows qualification and Intel-macOS runtime evidence, or an
   explicit owner disposition marking those cells unsupported-for-2.0.0.
4. MSRV verification job or owner acceptance of the declared-but-untested 1.88
   floor, and closure/disposition of the current-stable Clippy gate.
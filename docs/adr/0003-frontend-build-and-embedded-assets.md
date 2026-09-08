# ADR-0003: Frontend Build and Deterministic Embedded Assets

> **Document Type:** Architecture Decision Record
> **Audience:** LLM agents, human reviewers, engineering, release
> **Status:** Accepted (PRD-062 Slice 0, 2026-09-08)
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

Related: ADR-0001 (contract ownership), ADR-0004 (browser/accessibility
matrix), `docs/plans/2026-09-08-062-slice0-service-boundaries.md`, PRD 062
"Local means local", M-4, M-16, AC-16, AC-25, PRD Open Questions (front-end/
build, blocking) and risk table ("New frontend toolchain harms
reproducibility or maintainability").

## Context

PRD 062 needs a browser shell whose runtime is fully offline: all assets
embedded with the FORGE binary, no runtime outbound requests, telemetry,
remote fonts, schema fetches, or update checks (M-4, AC-16). Releases are
deterministic and supply-chain controlled: cargo-only CI
(`.github/workflows/ci.yml`), and a release pipeline
(`.github/workflows/release.yml`) that builds per-target, packages archives,
and attaches CycloneDX SBOM, SHA-256 checksums, and SLSA L3 provenance
(`docs/AR/049-ar-cross-platform-release.md`). The browser security model
requires a restrictive CSP with no inline or evaluated script, and bundled
assets must fail closed on API-major mismatch (AC-25). The PRD's blocking
question asks which front-end/build approach satisfies embedded offline
assets, deterministic releases, CSP, WCAG 2.2 AA, and contributor
maintainability, and forbids making framework selection a domain contract.
There is no Node toolchain in the repository today; the established embedding
pattern is `include_str!`/`include_bytes!` of vendored content (for example
the OSCAL schemas compiled in `src/validate/mod.rs`).

Constraints: no CDN or remote fonts; content-hashed assets served without
nonces (external files only, per CSP); locked dependencies; an offline
packaged golden-path test; contributor maintainability; no service workers
or browser storage.

## Decision

The workspace UI is a **static, framework-free shell hand-written in
semantic HTML, plain CSS, and vanilla ECMAScript modules — with no build
step**. Source files live under a UI asset directory in the repository and
are embedded into the binary at compile time via `include_str!`/
`include_bytes!`, following the vendored-schema pattern already used by
`src/validate/mod.rs`. At process start the server computes the SHA-256 of
each embedded asset (via the existing `src/hashing.rs` `sha256_hex`), serves
each asset at a content-hashed URL, and generates the bootstrap HTML
referencing those hashed URLs — giving subresource-integrity-style
content-addressed assets with zero JS toolchain. The browser is an API client
only (PRD principle 3); no server-rendered project data exists.

Trade-offs weighed honestly:

- **No build step, vanilla JS modules** (chosen): zero new toolchain; the
  offline/deterministic property is preserved exactly; anyone who can build
  the Rust crate can build the product; hashed assets come from the server
  side at startup rather than from a bundler. Cost: no static type checking
  and no bundler conveniences. Mitigations: the interactive surface is one
  bounded golden path (~six primary views); JSDoc type annotations with
  `// @ts-check` give editors local checking without any CI toolchain;
  behavior is pinned by the ADR-0001 fixtures, the headless conformance
  client, and browser/API coverage tests rather than by UI type checks;
  modules are small and hand-reviewable.
- **Pinned, vendored esbuild with TypeScript** (runner-up): gives real types
  and cheap bundling/hashing, and esbuild runs as a standalone binary without
  Node. Rejected for now: it introduces a per-platform vendored binary into a
  cross-platform, supply-chain-audited build (three CI operating systems,
  four release targets), a second lockfile domain, and a build-cache story —
  precisely the reproducibility risk the PRD flags. Revisit if the UI grows
  beyond the review workflow (reversal point below).
- **Framework (React/Svelte/Solid et al.) plus bundler**: largest toolchain,
  adds runtime and dependency mass that must be vendored, hashed, and
  audited; framework markup abstractions also work against the tight
  WCAG 2.2 AA and keyboard/screen-reader gates (ADR-0004), where direct
  control of semantics is valuable. Rejected.

Requirements fixed regardless of choice (binding on later slices):

1. No CDN, remote fonts, or any runtime-fetched asset; everything is embedded
   (M-4). The offline-runtime test (AC-16) must pass against the packaged
   binary.
2. Assets are served as external files at content-hashed URLs under the
   restrictive CSP: no inline/eval script, no object/embed, no framing; the
   CSP therefore needs no nonces.
3. The embedded-asset manifest hash is recorded as part of release evidence,
  flowing into the existing checksum/SBOM/provenance controls
  (`.github/workflows/release.yml`), so a released binary is attributable to
  exact asset bytes.
4. The bootstrap carries the UI's supported API major and an asset-contract
  version; on API-major mismatch the shell fails closed with a safe upgrade
  error and performs no project operation (AC-25). Supported v1 clients keep
  working across additive v1 changes.
5. No service workers, no localStorage/sessionStorage/IndexedDB of project
  content or tokens; drafts are page memory only (PRD "Source of Truth").

UI implementation itself is out of scope for Slice 0. This ADR sets the
direction so Slice 1 can build the read-only shell on it.

## Consequences

- Positive: the release pipeline stays cargo-only and fully offline; asset
  determinism is trivially auditable (hashes over committed files); no new
  supply-chain surface for cargo-audit/deny/vet or SBOM scope; contributors
  need no JS toolchain knowledge to review or modify the shell.
- Positive: server-generated hashed URLs mean stale-asset bugs are impossible
  by construction — a hash mismatch is a missing file, not a wrong file.
- Negative: no compile-time type safety in UI code; discipline is required to
  keep modules small and JSDoc annotations current. The headless conformance
  client and contract fixtures are the real behavioral guardrails.
- Negative: hand-written ES modules ship unbundled (one request per module);
  acceptable on loopback, and the module count is bounded by the golden-path
  scope.
- Negative: DOM updates are manual; this is the recurring pressure point that
  could reopen the decision.

Reversal point: if Slices 2-3 show the interactive surface growing
materially beyond the review workflow (rich editors, virtualized queues) such
that vanilla modules become the dominant maintenance cost, revisit with a
pinned, vendored esbuild + TypeScript proposal under a new ADR. That revision
must preserve every fixed requirement above.

## Alternatives considered

- **Pinned, vendored esbuild (or swc) + TypeScript, no framework.** See
  runner-up analysis above; deferred with an explicit reversal trigger rather
  than rejected outright.
- **Framework + bundler (React, Svelte, Solid, or a Rust-to-WASM stack).**
  Rejected: maximal toolchain and dependency mass; WASM additionally raises
  CSP (no-eval) and asset-size questions; abstraction layers add risk to the
  WCAG gate; none of their scaling benefits apply to a single-user local
  tool.
- **Server-side rendering of views from Rust templates.** Rejected: the PRD
  prohibits server-rendered project content; the shell must remain a static
  API client with no privileged data path.
- **No bundled UI at all (drive everything through an external browser
  page).** Not viable: violates M-4 (offline packaged assets) and the
  loopback-only trust model.

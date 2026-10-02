# PR 164 — getrandom differential review evidence

Agent-assisted read-only evidence; not a human audit, accepted cargo-vet record,
owner disposition, full F03 acceptance or merge approval. No Cargo, installation,
repository write, Git mutation or external posting was performed.

## Immutable identity and existing checks

- PR: https://github.com/policy-forge/forge/pull/164
- Head: `e5cce34634056262abc0cf503b976fba901e60a7`
- Base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`
- CI: https://github.com/policy-forge/forge/actions/runs/36143890874
- Existing September 25 Ubuntu/macOS/Windows tests, cargo-audit and cargo-deny
  passed. Cargo-vet failed on 21 unvetted safe-to-deploy versions: twenty shared
  baseline gaps plus getrandom 0.4.3. Run metadata confirms exact head/base;
  final-state snapshot records latest reread.

## Exact lock and source effects

Cargo.toml is unchanged. The new package version is getrandom 0.4.3 replacing
0.4.2, with matching locked checksum verified against the published archive.
Rust minimum 1.85, public fill API, feature definitions and build.rs are unchanged.
Verified upstream archive commits are
`4d826731b20a09e69cca91c66aea57ab3cf00072` →
`5e7cd5733536844a9856dc7259bd4696bbe5e3ae`.

Executable source changes are confined to WASI preview 2/3 manual random
bindings and wasm-family gating that adds wasm64 JS support. The reviewed
linux/android fallback, Windows and Apple native backend files and build script
are byte-identical across these two package versions. Forge's frozen supported
default graph is native Linux, Windows and two macOS architectures, and its
direct uses fill session token/salt buffers, failing to a generic internal error
on entropy failure. The changes do not add a fallback random source on those
native paths. WASI binding comments explicitly assume component type metadata
comes from std/other bindings; no WASI/wasm qualification is inferred.

The lockfile differential is broader than the single package substitution:
seventeen package versions are removed, including old getrandom 0.4.2 and sixteen
WASI-binding/code-generation transitives. wasip2 remains through other paths;
getrandom 0.4.3 removes its own wasip2/wasip3 dependency edges. indexmap 2.13.0
loses serde/serde_core edges, and wit-bindgen 0.51.0 loses its macro dependency;
these are resolver feature effects without package version changes.

The supported native graph also changes: unchanged tempfile 3.27.0 now selects
already-locked getrandom 0.3.4 instead of 0.4.2. A verified tempfile package
manifest explicitly permits >=0.3.0,<0.5 on Unix/Windows/WASI with default
features disabled on that dependency. This route is allowed, but it means the
candidate requires refreshed graph evidence rather than assuming every native
consumer moves to 0.4.3. Direct Forge, lopdf, rand and uuid edges do move to 0.4.3.

No actionable Forge regression was identified in this bounded differential
inspection. Source receipts, full lock transitions and exact source differences
are retained for a reviewer; source inspection and prior green CI do not qualify
an accepted entropy-stack audit.

## Advisory and acceptance boundaries

The current nontruncated RustSec tree
`117edb3bed98e9be112f277b7615eea3252e7c43` contains no getrandom advisory
record. Two records match rand_core in the frozen getrandom context; unchanged
locked versions satisfy their patched ranges. Immutable definitions are retained.
This scoped index check is not a proof of no vulnerabilities. Existing whole-lock
CI retains the ttf-parser maintenance warning; deny.toml explicitly records an
ignore rationale tracked by deps-rotation.

Remaining: candidate graph/feature regeneration, authentic attributable audit
evidence under D069, pinned MSRV and all native target/architecture verification.
Old versions are exempted rather than locally source-audited; the unchanged
remainder cannot inherit approval from this review. User-required full
documentation review/update at completed roadmap integration remains open.

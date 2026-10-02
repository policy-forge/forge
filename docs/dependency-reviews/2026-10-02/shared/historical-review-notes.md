# F03 dependency PR review evidence — 2026-10-02

This packet is agent-assisted, read-only review evidence. It is not a human audit,
accepted cargo-vet record, owner disposition, full F03 acceptance, or merge approval.
No Cargo process, installation, repository write, external review post, merge, or
tracker closure was performed.

## Immutable scope and existing checks

Common base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`.

| PR | Head | Exact package transitions | Existing CI |
|---|---|---|---|
| [160](https://github.com/policy-forge/forge/pull/160) | `9d215ad9b77c123e9b439198521b908ef2d9f9a7` | rayon 1.11.0 → 1.12.0 | [36143852105](https://github.com/policy-forge/forge/actions/runs/36143852105): Ubuntu, macOS and Windows tests passed; cargo-audit and cargo-deny passed; cargo-vet failed |
| [161](https://github.com/policy-forge/forge/pull/161) | `f91a77cdda2eb7dd95b231c155672c2bc17feee2` | clap 4.6.1 → 4.6.7; clap_builder 4.6.0 → 4.6.7; clap_derive 4.6.1 → 4.6.7; anstyle 1.0.13 → 1.0.14 | [36143862573](https://github.com/policy-forge/forge/actions/runs/36143862573): same step outcomes |

Both heads and the base were rechecked at 2026-10-02 09:25 UTC and remained open
with merge state BLOCKED. CI ran September 25 against these exact heads/base;
the checkout logs used GitHub synthetic merge commits
`2c5ed482e933a174b014481c53d806ae34c8e64a` and
`b8ec4f1ca992bfab9b7b5ead41f61f4a0ef206d9`, respectively.

The lockfile has no other package or existing package-record changes. PR 161
changes clap_derive's dependency edge from existing syn 2.0.117 to existing syn
3.0.6; syn 3.0.6 is not a newly added package version in this PR.

## Source observations and risks

All ten old/new published archives were fetched from static.crates.io, bounded,
SHA-256 checked against immutable Cargo.lock checksums, and safely extracted
under this temporary directory. `source-receipts.json` records checksums,
published VCS identities, Rust minimum versions, features and package sizes.

Rayon: runtime dependencies, feature set and Rust 1.80 minimum are unchanged.
The inspected differential includes the Range<char> surrogate-boundary fix,
array-window support, and CollectResult's drop changing from a temporary slice
reference to a raw slice pointer. Forge's use is the parallel slice iterator in
src/batch/orchestrator.rs, including mapped-result collection and optional local
thread pool construction. No character-range or window use was found. The
window implementation and character fix have upstream regression tests, but
these upstream tests were not executed here. No actionable Forge regression was
identified in this bounded source review.

Clap: default and derive features remain enabled; stable default feature sets
and Rust 1.85 minimum are unchanged. The differential changes attribute parsing,
adds opt-in deferred subcommand construction, migrates derive parsing to syn 3,
and changes help rendering for partially optional value names and aliases.
Deferred construction defaults false without unstable-v5; Forge enables neither
that feature nor a defer attribute. Builder parser changes inspected were
equivalent renames/to_owned/Self refactors; behavioral changes concentrate in
help output and derive generation. Anstyle's Rust source changes are two
documentation corrections; its runtime dependency set remains empty. No
actionable Forge regression was identified in this bounded source review.
Syn 3.0.6's registry metadata declares Rust 1.71 and matches the existing lock
checksum, but its full source was not independently audited in this slice.

Both PR descriptions contain sibling-crate comparison links: rayon-core tags
in PR 160 and clap_complete tags in PR 161. They are not the reviewed crate
identity. Proposed crate-prefixed tag comparison URLs were unavailable (404),
so the source review used checksum-verified archives and successful immutable
upstream commit comparisons instead: rayon
`7af20d7692d5decbcb4adcaa079cd607e3e50814` →
`c9ced185ae3508246a9eb70c8407a1199bb1b77f` and clap
`ac5fda6a799e4c640d671edd1111d4a5e723dc1a` →
`13f2db5072d600c11d8d6298e4e9ba53ffc6c1ab`.

## Current advisory and graph context

The F02 graph's lock digest exactly matches this immutable PR base. Context is
default features across aarch64-apple-darwin, x86_64-apple-darwin,
x86_64-pc-windows-msvc and x86_64-unknown-linux-gnu. This is baseline context,
not a newly resolved graph for either candidate head.

RustSec tree `117edb3bed98e9be112f277b7615eea3252e7c43` was nontruncated
and inspected for all package names in the frozen normal/build closures of
rayon and clap. No advisory records were present for the five updated package
names or syn. Four records matched transitives: anstream 1.0.0 is patched for
RUSTSEC-2024-0404 (>=0.6.8); crossbeam-deque 0.8.6 for RUSTSEC-2021-0093
(>=0.8.1); crossbeam-epoch 0.9.20 for RUSTSEC-2026-0204 (>=0.9.20);
crossbeam-utils 0.8.21 for RUSTSEC-2022-0041 (>=0.8.7). These versions are
unchanged by the two PRs. Immutable definitions and an index receipt are saved.
This scoped index check does not establish absence of vulnerabilities.

## Remaining acceptance boundaries

Cargo-vet reported 21 missing safe-to-deploy versions in PR 160 and 24 in PR 161.
Twenty are shared with the baseline; the additional entries are exactly rayon
1.12.0 or the four PR 161 updated packages. Existing predecessor records are
exemptions, not local source audits for these old versions; no full-source audit
of the unchanged remainder can be inferred from this differential review.

The D069 owner decision on audit signer/agent assistance, exception validity,
and trusted imports remains open. CI uses stable Rust, so it does not establish
the declared Rust 1.85 MSRV or cover each exact frozen target/architecture.
Current integrated-candidate verification and authentic accepted attributable
evidence remain required. PRs 162–164 and the shipping transport/session/crypto
stack were not reviewed in this slice. User-required full documentation review
and update at completed roadmap integration remains open.

# PR 162 — argon2 differential review evidence

Agent-assisted read-only evidence; not a human audit, accepted cargo-vet record,
owner disposition, full F03 acceptance or merge approval. No Cargo, installation,
repository write, Git mutation or external posting was performed.

## Immutable identity and checks

- PR: https://github.com/policy-forge/forge/pull/162
- Head: `eee2916fd7a2b52ea313cf309e4a1550cf4cbe53`
- Base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`
- Hosted CI: https://github.com/policy-forge/forge/actions/runs/36143871707
- Existing September 25 Ubuntu/macOS/Windows tests, cargo-audit and cargo-deny
  passed. Cargo-vet failed on 26 unvetted safe-to-deploy versions: twenty shared
  baseline gaps plus the six added versions below. Run metadata confirms exact
  head/base identities. Final-state snapshot records the latest reread.

## Exact lock and feature effects

Cargo.toml changes the production argon2 requirement from 0.5 to 0.6 and retains
explicit zeroize plus default features. Lock transitions are fully retained in
`lock-transitions.json` and `Cargo.lock.diff`:

- argon2 0.5.3 → 0.6.0
- blake2 0.10.6 → 0.11.0
- password-hash 0.5.0 → 0.6.1
- newly resolved phc 0.6.1, ctutils 0.4.2, cmov 0.5.4
- old rand_core 0.6.4 disappears; other already-locked rand_core versions remain
- argon2 now selects already-locked cpufeatures 0.3.0 rather than 0.2.17
- blake2 now selects already-locked digest 0.11.3 rather than 0.10.7; digest
  0.10.7 loses its subtle edge, and digest 0.11.3 gains ctutils through mac

New argon2 defaults are alloc/getrandom/password-hash. The optional parallel and
kdf features are not selected by Forge's declaration. phc/getrandom support is
newly active through password-hash defaults; this is broader than a one-package
version bump. Every introduced crate declares Rust 1.85, matching Forge's
declared floor; compatibility was tested by hosted stable Rust, not pinned 1.85.
No new crate version was added by this review itself.

## Focused source observations

Checksum-verified published archives were read for old/new argon2, blake2,
password-hash, cpufeatures and digest, plus new phc/ctutils/cmov. Receipt files
record exact archive hashes and VCS commits. The old/new argon2 VCS commits
resolve upstream; GitHub reports their histories diverged, so the actual review
differential is the exact published archive comparison, not merge-base diff.

Forge uses raw `hash_password_into`, explicit Argon2id V0x13, 65,536 KiB,
three iterations, one lane, a 32-byte tag and a 16-byte salt in the immutable
baseline `src/workspace/session.rs`. The new optional PHC and KDF APIs are not
used by that path. Allocation now uses a fallible custom Blocks allocation,
returning OutOfMemory; Forge maps errors to its generic internal error. Output
length and valid parameter checks remain present. The six upstream raw KAT
expected tags are unchanged between published versions, but these upstream
tests were not executed here and this does not prove cryptographic equivalence.

Argon2's memory traversal now uses a private NonNull-backed SegmentView even in
the sequential configuration. Bounds and lane/slice checks guard shared and
mutable access; the sequential feature path visits one segment at a time. This
is material unsafe-code change requiring full accepted source review, despite
no concrete new aliasing regression identified in the focused inspection.
cpufeatures 0.3 preserves AVX2-gated x86 dispatch while extending CPU detection;
Apple aarch64 and x86 source paths were inspected. No target-specific execution
or side-channel qualification was performed.

Zeroize remains enabled for Argon2 and Forge wraps session secret material in
Zeroizing. The inspected Argon2 Blocks destructor deallocates KDF scratch memory
without an explicit wipe, as the prior Vec<Block> path also did. This review
does not establish that KDF working memory is erased and does not classify that
baseline behavior as a newly introduced vulnerability.

No actionable Forge regression was identified in this bounded review. The
large crypto migration and wholly new crate source bodies remain outside any
claim of complete accepted audit coverage.

## Advisory and acceptance boundaries

RustSec tree `117edb3bed98e9be112f277b7615eea3252e7c43` is nontruncated.
Baseline native argon2 closures plus head-added package names matched five
records. New cmov 0.5.4 satisfies the >=0.4.4 patched range for
RUSTSEC-2026-0003; blake2 0.11.0, generic-array 0.14.7 and remaining rand_core
versions satisfy the other matching records' patched ranges. Immutable advisory
definitions/summaries are retained. This is a scoped advisory check, not a new
resolved feature graph or proof of no vulnerabilities. Existing whole-lock CI
also reports ttf-parser's informational maintenance warning; deny.toml records
an ignore rationale tracked by deps-rotation.

Remaining: exact candidate graph/feature regeneration after integration,
source review attributable under the D069 signer/import/exception decision,
MSRV and complete target verification, and independent cryptographic known-answer
evidence for the newly selected path. Predecessor versions have exemptions,
not local source audits, so this differential cannot inherit unchanged-source
approval. The user-required full documentation review/update at completed
roadmap integration remains open.

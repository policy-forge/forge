# F03 individual dependency review preparation — 2026-10-02

The [retained review packet](../dependency-reviews/2026-10-02/README.md) records
bounded agent-assisted differential reviews of PRs 160–164 at immutable heads
on base `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. This implements the
individual dependency-PR review slice of
[F03 tranche context](../dependency-reviews/2026-10-02/shared/historical-roadmap-completion-tranches.txt), with exact source/archive,
advisory and existing hosted-CI evidence. It does not close F03 or certify any
crate under [PRD 069](../PRD/069-prd-dependency-security-audit.md).

## Delivered evidence

- Five concise per-PR notes and explicit nonacceptance receipts bind exact old/new
  package transitions, immutable head/base and existing CI outcomes.
- Original historical receipts, source deltas, advisory definitions and failed
  supply-chain logs are retained byte-for-byte; a new SHA-256/byte-count index
  distinguishes them from newly prepared notes, excerpts and receipts.
- Expanded third-party packages and compressed archives are omitted. Versioned
  upstream URLs, archive hashes and omitted-member hashes allow bounded replay;
  retained source deltas preserve package license texts where available; the
  published PDF package declares MIT but omits a standalone license text.
- Frozen F02 default-feature native graph context matches the base lock/manifest
  digests. It is explicitly separate from unresolved candidate graph evidence.

## Review outcome and remaining gates

No new actionable Forge regression was identified in the bounded source review.
Argon2's broad crypto and unsafe memory traversal changes still require complete
attributable review. Pdf-extract's page font-cache correction lacks direct
regression evidence. Getrandom's native source is unchanged but the candidate
reroutes tempfile's entropy dependency, so a new graph is required.

Retained snapshots show all five PRs OPEN/BLOCKED. Existing three-OS tests and
audit/deny passed; cargo-vet failed on new versions plus twenty shared baseline
gaps. Stable CI does not establish the Rust 1.85 floor or all four architectures.
Exempt predecessor versions cannot supply an accepted unchanged-source audit.

Keep the existing ttf-parser maintenance disposition: deps-rotation,
REVIEW-BY 2026-12-31 and removal after lopdf drops the dependency. These reviews
do not silently change it or create a new audit/exception approval.

D069 signer/agent-assistance, exception and trusted-import decisions remain open.
Full shipping transport/session/crypto source reviews, accepted attributable
records, candidate graph/MSRV/target checks and disposition of validated baseline
findings remain required. No audits.toml/config.toml, dependencies, CI policies,
Git state or external reviews were changed by packet preparation. No Cargo ran.

The user's full documentation review/update after completed roadmap integration
is a later required gate; this slice does not satisfy or remove it.

## Before drafting the evidence PR

Docstring and test coverage audit: this slice adds review documents and retained
receipts only. It adds no executable production code, public Rust API or test
behavior; there is no changed production-line coverage denominator. The
coordinator verified all 152 retained files against their exact sizes and
SHA-256 digests. Historical copy identity and local document links were also
checked independently. No source audit acceptance is inferred from these checks.

The mandatory commit hook still runs formatting, strict Clippy and the full
Rust tests before drafting. Report its toolchain and results with the PR;
current-stable compatibility is separate task #11. This preparation note does
not claim those pending hook checks have passed.

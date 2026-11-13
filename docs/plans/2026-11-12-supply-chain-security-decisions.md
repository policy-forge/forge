# Supply-chain security update decisions — 2026-11-12

Status: **The three exact-identity temporary exceptions are
OWNER-APPROVED (2026-11-13, see [Owner approval record](#owner-approval-record));
human audit sign-off and the Dependabot alert dismissal remain open.** This
packet records agent-assisted differential review evidence for those
exceptions. It is not an accepted source audit, not human sign-off, not merge
permission, and not release acceptance. Policy:
[PRD 069](../PRD/069-prd-dependency-security-audit.md); conventions follow
the [2026-10-24 renewal record](2026-10-24-dependency-renewal-and-update-decisions.md).

## Why action is required

1. **The 25 open Dependabot alerts are retained-evidence artifacts, not
   shipping-graph vulnerabilities.** All 25 (5 advisories × 5 manifests)
   point at `docs/dependency-reviews/2026-10-02/pr16{0..4}/Cargo.lock` —
   byte-for-byte retained October review receipts pinning zip **8.6.0**,
   which is inside every advisory range. The shipping root lock pins zip
   **9.1.0**, outside all five ranges (GHSA-m665-v7pm-7grj patched 9.0.5;
   GHSA-hjqx-j4rw-g7xv, GHSA-q7v9-x93c-xqv3, GHSA-9hj8-73fh-9963,
   GHSA-8hxf-qr8p-62x4 patched 9.0.0). Verified 2026-11-12 via the
   Dependabot API (`dependency.manifest_path`) and `cargo audit` on main
   (no zip finding). The evidence locks must stay immutable; dismissing
   the 25 alerts with a "tolerable risk / retained evidence" reason is an
   owner action proposed here, not a code change.
2. **zip 9.1.0 → 9.1.2 (PR #260, head
   `3dd72673a3ef1cbd5354b96d5f2565062865ba4d`, base `b4226bb`) is still
   recommended**: it hardens the parser Forge uses (checked offset
   arithmetic, exact duplicate-name accounting) and the extract path
   (symlink-safe file creation), closing the advisory attack classes
   outright rather than relying on Forge's narrow usage. #260's CI failed
   only the supply-chain gates (retired 9.1.0 exception, stale graph
   digest); all three-OS test jobs passed.
3. **Two pulldown-cmark advisories are the live `cargo audit` failures on
   main**: RUSTSEC-2026-0414 (exponential event growth) and
   RUSTSEC-2026-0433 (quadratic nested image wikilinks), both
   `ENABLE_WIKILINKS`-only, both high (7.5). Forge never enables that
   option (`src/parse/mod.rs:119`, `src/parse/clauses.rs:494` enable only
   tables/YAML front matter), so current exposure is nil — but the clean
   fix is the 0.13.6 upgrade, not an ignore entry.
4. **rustix RUSTSEC-2026-0486** (INFO, unsound): xattr functions mark
   unwritten bytes initialized on zero-length buffers; issued 2026-11-11,
   patched ≥1.1.6. Runtime reach is via `tempfile` only; Forge does not
   call the affected `rustix::fs::*xattr` functions directly.

## Proposed decision: three lockfile-only security updates

No `Cargo.toml` requirement changes, no new crate names, no new direct
dependencies. All identities use source
`registry+https://github.com/rust-lang/crates.io-index`.

| Crate | Transition | Criteria | New archive SHA-256 | Advisories closed |
| --- | --- | --- | --- | --- |
| zip | 9.1.0 → 9.1.2 | safe-to-deploy | `ec2e3161de27091f18a831d9494e9c53378a34833af5c1dda35ad85849da18d6` | None against 9.1.0 (already out of range); hardens the GHSA-m665-v7pm-7grj / GHSA-hjqx-j4rw-g7xv / GHSA-q7v9-x93c-xqv3 attack classes in the parser and extract path |
| pulldown-cmark | 0.13.4 → 0.13.6 | safe-to-deploy | `3db2be46835f013dd3f4fa7f5f7402c537787b54ebb6fe2b94bbdd7e2669bb7b` | RUSTSEC-2026-0414, RUSTSEC-2026-0433 |
| rustix | 1.1.4 → 1.1.6 | safe-to-deploy | `e5e76742e5294153d4dde1207dbec5bdd81eef6fabdc0d2118babadabfd858aa` | RUSTSEC-2026-0486 |

Retired identities: zip 9.1.0 (checksum
`2886b00003a72c4ccee3f42cfc34908d33b2508a74d827dc2a4244706475c704`),
pulldown-cmark 0.13.4 (`e9f068eba8e7071c5f9511831b44f32c740d5adf574e990f946ddb53db2f314e`),
rustix 1.1.4 (`b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190`).
The zip 9.1.0 exception (owner-approved 2026-10-24, expires 2026-11-23) is
replaced, not renewed; pulldown-cmark and rustix had bare vet exemptions
with no inventory exception — the proposed entries bring them under the
same owned-exception discipline.

Proposed exception terms (matching the October pattern): owner
`brianluby`, created 2026-11-12, review by 2026-11-26, expiry 2026-12-26
(44 days, within the 90-day policy maximum). Entries are drafted in
`supply-chain/dependency-exceptions.json` and matching native vet
exemptions in `supply-chain/config.toml`. They were drafted as **PROPOSED
pending owner approval**; on approval the PROPOSED markers are struck and the
decision recorded below, and on rejection the three lockfile bumps and the
store changes would be reverted together.

## Owner approval record

- **Decision:** the three proposed exceptions are approved as proposed:
  `zip 9.1.2`, `pulldown-cmark 0.13.6` and `rustix 1.1.6`; owner `brianluby`,
  created 2026-11-12, review by 2026-11-26, expiry 2026-12-26.
- **Approved by:** `brianluby`, as an attributable GitHub review comment
  ("Approved.") on the CodeRabbit finding about pending exceptions:
  <https://github.com/policy-forge/forge/pull/266#discussion_r4559418015>, posted 2026-11-13 (UTC).
- **Recorded in:** the PROPOSED markers in
  `supply-chain/dependency-exceptions.json` and `supply-chain/config.toml`
  were struck on the owner's instruction; each record's `approval_reference`
  points at this document.
- **Not covered by this approval:** human audit sign-off (F03, ticket #5),
  merge permission, release acceptance, dismissal of the 25 retained-evidence
  Dependabot alerts (still a proposed owner action below), and whether the
  `pulldown-cmark`/`rustix` updates should move to a separate PR.

## Bounded differential review evidence (agent-assisted)

Sources: exact crates.io archives from the local registry cache, diffed
old → new. This is not a full-rooted source audit.

### zip 9.1.0 → 9.1.2

Changed source files: `src/read/extract.rs`, `src/read/stream.rs`,
`src/read/zip_archive.rs`, `src/types.rs`, `src/write.rs`,
`src/write/options.rs`.

- `read/extract.rs`: new `create_outfile` removes an existing symlink at
  the output path before creating the file and re-verifies after open
  (GHSA-hjqx-j4rw-g7xv); symlink-as-file helpers now carry `unix_mode`.
- `read/zip_archive.rs`: duplicate-name detection now compares the raw
  entry count against the deduplicated map size (exact accounting instead
  of a bool set at parse time). The CDFH/LFH consistency validation
  referenced by GHSA-m665-v7pm-7grj predates 9.1.0; the 9.1.x delta does
  not change header-matching semantics.
- `write.rs` / `write/options.rs`: opt-in extra-field validation,
  duplicate-name rejection in `merge_archive`, stale-footer blanking,
  Unicode Path extra field that would rename an entry is rejected.
- `types.rs`: UTF-8 language-encoding flag set for non-ASCII comments
  when the name is valid UTF-8 (write-side only).

**Forge exposure:** production use is read-only, in-memory, single-entry
(`src/ingest/mod.rs:174-205`): `ZipArchive::new(Cursor)` →
`by_name("word/document.xml")` → bounded `take(cap)` read with a
decompression budget. Forge never calls `extract()`, never enables AES
or legacy Shrink/Reduce/Implode decoders (deflate-only feature), and
never writes ZIPs outside tests. The extract-path fixes are therefore
defense-in-depth for Forge; the duplicate-name accounting change affects
the `ZipArchive` reader Forge does use (stricter duplicate detection).
No behavior change expected for well-formed DOCX.

### pulldown-cmark 0.13.4 → 0.13.6

Changed source files: `src/firstpass.rs`, `src/parse.rs`.

- `parse.rs`: wikilink parsing bails on malformed delimiters
  (`end_ix <= start_ix` underflow guard), handles empty display text
  with a zero-length synthetic node, and bounds nested-image wikilink
  recursion — the RUSTSEC-2026-0414/0433 fixes.
- `firstpass.rs`: container scanning carries an explicit flag; list-item
  and definition-list tightness moved to `Cell<ListFlags>`; a multibyte
  offset correction (upstream issue 1142) keeps container starts on
  character boundaries. `Options` remains `Copy`.

**Forge exposure:** `ENABLE_WIKILINKS` is never set, so the vulnerable
paths were unreachable. The list-state refactor touches code Forge does
exercise (tables, YAML front matter, headings, clauses); the full
library + contract + parse test suites are the regression gate.

### rustix 1.1.4 → 1.1.6

63 source files differ (~1318 changed lines) — broader than the xattr
fix alone; includes the 1.1.5 release. Security-relevant core:
`src/fs/xattr.rs` (+232 lines) special-cases zero-length buffers
(`assume_init(0)`) and adds explicit `*_query` size-query APIs
(RUSTSEC-2026-0486). Remaining delta is backend syscall wrappers,
type/constant additions, and `check_types` maintenance. No Forge code
calls rustix directly; reach is `tempfile` (runtime) and
`insta`/`proptest` (dev). Full test suite is the regression gate.

## Verification (this branch, head recorded at PR time)

- `cargo audit`: 0 vulnerabilities; 2 allowed warnings (ttf-parser,
  unsafe-libyaml — owner-approved ignores in `deny.toml`, review by
  2026-12-31).
- `cargo vet --locked`: succeeded (2 fully audited, 1 partially audited,
  296 exempted) with the PROPOSED exemptions.
- `python3 -B scripts/dependency_graph.py check`: qualified (graph
  refreshed via `refresh --output supply-chain/dependency-graph.json`;
  300 packages, four release targets).
- `python3 -B scripts/dependency_inventory.py gate --as-of 2026-11-12`:
  `review-required` (exit 1) — the standing human-audit gap, unchanged in
  character: 3 fresh owned exceptions (the proposed ones), 170 legacy
  unowned, 211 missing of 215 runtime denominator. No input errors.
- Functional suites at head `3dd7267` + store changes: `cargo test --lib`
  3350 passed / 0 failed; `cargo test --test api_contract_validation`
  30 passed / 0 failed. Full suite + fmt/clippy recorded in the PR body.

## Proposed owner action: dismiss the 25 retained-evidence alerts

The five `docs/dependency-reviews/2026-10-02/pr16*/Cargo.lock` files are
immutable historical receipts (their SHA-256 index is part of the F03
evidence). They must not be edited to silence Dependabot. Proposed
disposition: dismiss all 25 alerts as "tolerable risk — retained
historical review evidence, not a shipping manifest; shipping zip 9.1.0
is out of all advisory ranges". This is an owner click in the Dependabot
UI (or an authorized API action), recorded here on approval.

## What this does not do

- Does not close F03 (#5): attributable accepted human audit of the
  shipping graph remains outstanding.
- Does not authorize merging #260–#265 or this branch.
- Does not certify any crate safe; exceptions are integration permission
  with a review clock, per PRD 069.
- Does not touch the two unmaintained-crate ignores (separate tracked
  decisions, review by 2026-12-31).

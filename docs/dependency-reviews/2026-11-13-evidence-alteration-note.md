# F03 retained-evidence alteration note — 2026-11-13

Status: **Authorized alteration recorded; chain re-pinned.** This note documents
an owner-authorized change to retained F03 evidence. It does not record a new
source audit, human signoff, merge permission, or F03 acceptance.

## What happened

[PR #265](https://github.com/policy-forge/forge/pull/265) ("chore(deps): bump
the cargo group across 3 directories with 1 update", head commit
`24616651b0f72a81d6d23682f8370139301ef516`, merge commit
`52ab80e0ffe0dd7901bffe0c899228e8b8a39108`, merged 2026-11-13T15:42:58Z) edited
six retained files of the
[2026-10-02 F03 review packet](2026-10-02/README.md): the `Cargo.lock` and
`Cargo.toml` snapshots retained for PRs 162, 163, and 164. In each, the locked
`zip` dependency moved 8.6.0 → 9.0.0 (registry checksum
`2d04a6b5381502aa6087c94c669499eb1602eb9c5e8198e534de571f7154809b` →
`711e8c9646791c49b7eb799a19c4ae70a9b8372ddc6a1fa4778defd1e2aa5d5e`) and the
manifest requirement moved `zip = "8"` → `zip = "9"`. All three merged locks
resolve to zip 9.0.0; the decision record's "9.0.0/9.0.5" shorthand refers to
this same authorized bump family, not to a second version present in the
retained locks. Byte counts are unchanged. The pr162/pr163 locks differ only
in their zip entries; the pr164 lock additionally re-resolved one dependency
edge (`getrandom 0.3.4` → `getrandom 0.4.3`) as part of the same regeneration.

These files were retained as immutable review evidence. Their alteration
invalidates the original SHA-256 pins in
[retained-evidence-index.json](2026-10-02/retained-evidence-index.json).

## Owner disposition

Owner decision **D3b** of the 2026-11-13 owner decision record
(`docs/plans/2026-11-13-owner-decision-record.md`): **keep #265 merged and add
a dated F03 note**. The SHA-256 index discrepancy for these six files is
**authorized, not tampering**. The historical receipt files themselves are not
edited by this note; the retained `.diff` receipts are untouched and still
describe the original PR 162/163/164 changes.

## Re-pinned hashes

Original pins below are the values committed in `a46b07d` ("Retain exact
dependency PR review evidence") and remain recoverable from git history. New
pins were computed from the merged #265 content on 2026-11-13 and are recorded
in `retained-evidence-index.json` by the PR carrying this note. The other 145
retained entries were re-verified against the index on 2026-11-13 and match.

| Path (under `docs/dependency-reviews/2026-10-02/`) | Bytes | Original SHA-256 | New SHA-256 |
|---|---:|---|---|
| `pr162/Cargo.lock` | 74787 | `26163aa672bbe2d54448bac821ebe4246003c917b98a0f48dbe20bcd8af71e3f` | `7d33c63c56ddb11e9efc034aaa1f4e3b850f6209eb59760b4a45154bf4f9dff9` |
| `pr162/Cargo.toml` | 2344 | `e97a2d27d6718ed1eb60c6506781875ebf902f92d81fefd22d9253f298a84c95` | `b70fe2143086e33d7186837f8bb32da22750bfdf70d3c6b1bf1697da5d9801dc` |
| `pr163/Cargo.lock` | 74338 | `fd483e330e292cd81131d59b30b3f89771206365470d3da03e713610ab561703` | `2e44746103e673e726611b2aeee5b1e60934522fedd0d7922adaabb2e5e1a6fe` |
| `pr163/Cargo.toml` | 2344 | `6adc23a0b01a5b19336cb6d3a6382388a605b60df54dd022d2e092aca500872a` | `eec5af9a829aeeb3240eab5e5e7ef0eb2cf104a2709116deed243129b4a63199` |
| `pr164/Cargo.lock` | 70103 | `ed61f9997164f0b6a1a75525a61f3e1bd1a3ba594f8219b7441f5ac8c871674d` | `a182758923a9275294dff0d0cc4b01689c9417273ab2dd7d9c9e37d4372af933` |
| `pr164/Cargo.toml` | 2344 | `6adc23a0b01a5b19336cb6d3a6382388a605b60df54dd022d2e092aca500872a` | `eec5af9a829aeeb3240eab5e5e7ef0eb2cf104a2709116deed243129b4a63199` |

The packet `README.md` is itself indexed; its entry is re-pinned in the same
index update because this note adds a link to it.

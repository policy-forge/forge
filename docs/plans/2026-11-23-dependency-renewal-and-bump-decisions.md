# Dependency exception renewal and bump decision — approved November 23, 2026

This packet covers three owner decisions made on November 23, 2026:

1. Renewal of the 40 owned exceptions whose review date (November 20, 2026)
   had passed.
2. `url` 2.5.9 in place of 2.5.8, with the `form_urlencoded` 1.2.3 it
   requires, clearing RUSTSEC-2026-0592 so the CI `cargo audit` step passes.
3. Approval of the exact versions proposed by the five open Dependabot pull
   requests (#296 through #300), each to be integrated individually with its
   own supply-chain bookkeeping.

Neither decision signs off a source audit or releases the product.

## Why action was required

From November 21 the inventory gate (`dependency_inventory.py gate`, run by the
CI `Dependency inventory gate` job with the current UTC date) reported 40 owned
exceptions as stale, because each had `review_by: 2026-11-20`. Every pull
request failed that gate whatever it changed. Separately, RUSTSEC-2026-0592
(published November 20: stack overflow in `Url::origin()` on nested `blob:`
URLs, `url` 2.5.8, fixed in 2.5.9) made the CI `cargo audit` step fail. `url`
is a direct runtime dependency of Forge (`url = "2"` in `Cargo.toml`). Forge
does not call `Url::origin()` and does not build `blob:` URLs.

The active ruleset on `main` does not require these checks for merging, so the
gates held by convention only. That gap is outside this packet.

## Decision 1: renewal of 40 existing exceptions

The owner renewed the 40 entries in `supply-chain/dependency-exceptions.json`
without changing any identity, checksum, criteria, owner or `created_on`:

- `review_by` moves from 2026-11-20 to **2026-12-23**.
- `expires_on` moves to each entry's own 90-day `max_exception_days` cap:
  **2027-01-04** for the 27 entries created 2026-10-06, **2027-01-22** for the
  8 created 2026-10-24 and **2027-01-23** for the 5 created 2026-10-25.
- `approval_reference` points to this packet. Each rationale keeps its original
  text and names its original approval, with one renewal sentence appended.

The 20 entries with review dates of 2026-11-26 through 2026-11-28 are not part
of this decision and are unchanged. The renewed identities' full-rooted source
audits remain outstanding.

## Decision 2: `url` security update

The owner approved `cargo update -p url --precise 2.5.9`. The requirement in
`Cargo.toml` is unchanged and no Forge source changed.

All identities use source `registry+https://github.com/rust-lang/crates.io-index`.
Owner GitHub `brianluby`; created November 23, review by December 23, expiry
February 21, 2027.

| Crate | Version | Criteria | Archive SHA-256 |
| --- | --- | --- | --- |
| url | 2.5.8 → 2.5.9 | safe-to-deploy | `f00cd2407bb42c0ef17be88d4cf020f5cceffd121665bac9b8b9493a369b3b1a` |
| form_urlencoded | 1.2.2 → 1.2.3 | safe-to-deploy | `afa489199eaffb1211795df1c6236ce1ad0d41a6fa45250add73e3b02420f06e` |

Each new identity has a matching native cargo-vet exemption in
`supply-chain/config.toml` whose notes reference this packet. Older-version
exemptions remain until `cargo vet prune`. The source examination, advisory
scan and verification are recorded in
[the review note](../dependency-reviews/2026-11-23/README.md).

## Decision 3: Dependabot version bumps

The owner approved these exact versions. They add no crate names and no direct
dependency. `base64` is the only one that changes a `Cargo.toml` requirement
(`"0.22"` to `"0.23"`).

| Crate | Version | Also moves | PR |
| --- | --- | --- | --- |
| clap | 4.6.7 → 4.6.8 | clap_derive 4.6.8; proc-macro2 1.0.107; quote 1.0.47; syn 3.0.10 | #296 |
| zeroize | 1.9.0 → 1.9.1 | none | #297 |
| base64 | 0.22.1 → 0.23.1 | none | #298 |
| jsonschema | 0.61.1 → 0.61.2 | none | #299 |
| sha2 | 0.11.0 → 0.11.1 | none | #300 |

Each lands individually through its own pull request, merged one at a time
after this packet is on `main`. For every identity it introduces, that request
adds an exact owned exception (same owner, source and dates as Decision 2) and
a cargo-vet exemption, and retires the record of any identity that leaves
`Cargo.lock`. Development, build and procedural-macro identities outside the
runtime closure receive only the cargo-vet exemption.

| Identity | Archive SHA-256 |
| --- | --- |
| clap 4.6.8 | `ac230478587d0269d8d6998b82ae69ea51509d55df66d9e92edd1f986904cd4a` |
| clap_derive 4.6.8 | `895c9d8d2ac2dd8b8a29f3724a14e72aaaa6debcf8c38c82f97305c6a172767a` |
| proc-macro2 1.0.107 | `985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9` |
| quote 1.0.47 | `1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001` |
| syn 3.0.10 | `4ae9ccbea3a7b0b09443036490c8f34a8b1b760fd345e5c0dae0aac718faea79` |
| zeroize 1.9.1 | `e13084392c5e4bc371903e2935a5eaeed24905a7511356b883835e18a78f6879` |
| base64 0.23.1 | `ac07cdecf99051d9a5238b80f35af32cdeba5b336e55d957b318b50137e18da5` |
| jsonschema 0.61.2 | `0e37f27df77c7887edbc5f5a59ecd8e5ec4a669319266ec1b61068667a603183` |
| sha2 0.11.1 | `47d7069beb7d6ac7b9acd1039986e73443f24234f41074da099d6f994ac9ad19` |

This decision covers only the identities and checksums listed above. Any other
identity a later rebase introduces needs its own approval.

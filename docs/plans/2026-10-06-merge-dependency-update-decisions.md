# Dependency update decision packet — approved October 6, 2026

This packet concerns only original PRs #160–164. The prior approved 21 exceptions
remain exact-version decisions and do not cover these newer identities.
The owner approved these updates and exact exceptions; no source-audit signoff
is supplied.

## Owner-approved disposition

The owner approved integration of the five updates and the three new transitive crate names
`cmov 0.5.4`, `ctutils 0.4.2` and `phc 0.6.1` required by Argon2's existing
password-hash/Blake2 chain. Existing alternatives are the old Argon2 0.5.3 graph
or deferring #162; Forge uses the maintained Argon2 raw-tag interface and keeps
its centrally fixed Argon2id parameters and unlock/session contracts.
These transitive dependencies are introduced by the existing proposed upstream
Argon2 graph; Forge adds no new direct dependency or custom cryptography.

The owner approved the 13 exact checksum-bound temporary exceptions below under the existing
incremental policy: owner GitHub `brianluby`, created October 6, review by
October 20, expiry November 5, 2026. The rationale is to integrate the bounded
maintenance updates while full-rooted accepted source audits remain outstanding.
The review deadline requires action from October 21 even before expiry.
This is neither a declaration that the crates are safe nor release approval.

All identities use source `registry+https://github.com/rust-lang/crates.io-index`.
The proposed native cargo-vet coverage must retain matching exact identity and
criteria; proc-macro generated code also receives safe-to-deploy coverage.

| Crate | Version | Proposed criteria | Metadata parents | Archive SHA-256 |
| --- | --- | --- | --- | --- |
| anstyle | 1.0.14 | safe-to-deploy | anstream, anstyle-wincon, clap_builder | `940b3a0ca603d1eade50a4846a2afffd5ef57a9feac2c0e2ec2e14f9ead76000` |
| argon2 | 0.6.0 | safe-to-deploy | forge | `134c52ddac6d63c576bef8168db10c83c49c26444ecbc68060fef078925a901c` |
| blake2 | 0.11.0 | safe-to-deploy | argon2 | `5b5d4d889834ee8ecfc0f8426ad30faf7cdcb10f741a8e6d7224d95325479f6f` |
| clap | 4.6.7 | safe-to-deploy | criterion, forge | `aa8876b300ab35ba921adea3dfd70157a46249b33f95c9084ae5709785478946` |
| clap_builder | 4.6.7 | safe-to-deploy | clap | `ec0797fb7aeb1406c84efac526901f7ec3ead2124f946b494e72879d4b54704d` |
| clap_derive | 4.6.7 | safe-to-run + safe-to-deploy (generated code) | clap | `f9c751b79415d4e559e3d1fcf128e09e720eb673a06d26cf6f392d37d75b66e0` |
| cmov | 0.5.4 | safe-to-deploy | ctutils | `0c9ea0ac24bc397ab3c98583a3c9ba74fa56b09a4449bbe172b9b1ddb016027a` |
| ctutils | 0.4.2 | safe-to-deploy | digest, phc | `7d5515a3834141de9eafb9717ad39eea8247b5674e6066c404e8c4b365d2a29e` |
| getrandom | 0.4.3 | safe-to-deploy | forge, lopdf, password-hash, phc, rand, uuid | `300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099` |
| password-hash | 0.6.1 | safe-to-deploy | argon2 | `aab41826031698d6ffcd9cff78ef56ef998e39dc7e5067cdfebe373842d4723b` |
| pdf-extract | 0.12.1 | safe-to-deploy | forge | `e5c4820f5811e424ce037d08493ac87752ecc54339e0f1e40c3d17da93278861` |
| phc | 0.6.1 | safe-to-deploy | password-hash | `44dc769b75f93afdddd8c7fa12d685292ddeff1e66f7f0f3a234cf1818afe892` |
| rayon | 1.12.0 | safe-to-deploy | criterion, forge | `fb39b166781f92d482534ef4b4b1b2568f42613b53e5b6c160e24cfbfa30926d` |

## Prepared evidence and limits

A proposal-only copy outside the repository composes the existing five update
patches with reviewed source `b6488aef5e0bfa0b472d6b62483154ef4380d74f`.
The proposal initially kept Argon2 and its new names outside the managed repository
candidate until the explicit owner decision below. Its Cargo manifest SHA-256 is
`e97a2d27d6718ed1eb60c6506781875ebf902f92d81fefd22d9253f298a84c95`;
its reconciled lockfile SHA-256 is
`49a828c55af7e117b4490adc0de6b53caf43cd899668c6e7a5f6026237a1a429`.
The initial locked fetch refused a stale composed lock graph; an offline precise
Argon2 update reconciled it without changing the 13 new identities. The failed
initial attempt remains preserved.

All 13 cached crates.io archives hash to their proposed Cargo checksums.
Offline locked metadata and independent maintained graph qualification pass:
298 packages, four release targets, 520 dependency edges on each target.
The qualified graph SHA-256 is
`4aaa44b72fc0e04abe91a36369985c71c40a745efba61cafda4226d6394f58e6`.
This is graph evidence; it does not prove final binary contents or source safety.

All seven actual workspace session controls pass using the combined proposal,
including passphrase bounds, unlock throttling/capability issuance and machine
session restrictions. This is a local focused compatibility check. Full tests,
normal repository hooks, strict lint, fresh audit/deny/vet and hosted/native
qualification remain required after approval and final composition. The first
requested integration-test target did not exist and executed zero tests; no
credit is assigned to it.

The maintained inventory reports no input errors, 12 new unvetted runtime
identities and one unreviewed proc-macro identity. Existing configured import
snapshots contain no accepted audit paths for these versions. Old native/legacy
exemptions cannot seed new-version audit credit. We do not fabricate a human
source-audit signoff; these are explicit owner-approved exceptions.

The original WI/Ready/human/product/security/privacy/accessibility, independent
client/corpus, D064 and release boundaries remain unchanged. Declared compiler
minimums are not measured MSRV qualification. Existing historical legacy
baseline, original audits and prior 21 exceptions remain intact.

## Human decision receipt

In Codex chat `01a110f1-0063-72a0-a56c-c1d3e8cf236d`, after this complete
checksum-bound packet was opened and the new crate names and dates were named,
the human user answered **“Approve updates and 13 listed exceptions”** on
October 6, 2026. This approves the five version updates, Argon2’s three new
transitive crate names and the 13 listed exceptions with owner `brianluby`,
review October 20 and expiry November 5. It does not sign off source audits or
release the product. The earlier 21 approved entries remain unchanged.

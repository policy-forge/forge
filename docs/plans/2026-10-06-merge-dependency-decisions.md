# Forge merge dependency decisions — approved 2026-10-06

The initial dependency decision covered 20 existing exact-version dependencies without accepted `safe-to-deploy` audit paths. A separate owner-approved chacha20 exception below brings the recorded set to 21 exact identities. Refreshing the already-configured Mozilla and Embark sources did not close any of these gaps. No dependency version or audit is changed. The approved policy and owner-bound exceptions are recorded in the sidecars and native store.

## Recorded owner disposition

The owner approved the existing PR #166 incremental policy: exact locked identities and checksums; human sign-off for agent-assisted audits; existing Mozilla/Embark sources pinned to the committed snapshot; exceptions with an accountable owner, rationale, review date and expiry, capped at 90 days. Legacy exemptions remain visibly unreviewed. Build/proc-macro code that generates deployed code still needs native `safe-to-deploy` coverage.

The owner approved the following 20 temporary exceptions, owned by GitHub `brianluby`, created October 6, reviewed by October 20, expiring November 5, 2026. This permits integration under the incremental gate while real audits continue. It does not approve a release, certify safety, satisfy strict/full-store acceptance, or erase the 319 unowned historical exemptions. The review date makes the gate require action starting October 21 even before expiry. No new versions or dependencies are introduced by this proposal.

The alternative is to keep merges held until full-rooted accepted audits/sign-offs exist for every gap. Source-review suggestions seeded by old blanket exemptions cannot establish a fully audited base.

## Exact proposed set

| Crate | Version | crates.io archive SHA-256 |
| --- | --- | --- |
| fancy-regex | 0.19.2 | `d301f5bf187b3c295fce6468d3875037a0bccc5f6b151c63cac2f85babf21912` |
| fraction | 0.17.0 | `e246562084dde8ebbcc943b261c406ce4f68e5032ec28029a251a47d6a295500` |
| hashbrown | 0.17.1 | `ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a` |
| jsonschema | 0.57.0 | `71160ed5f6dbe36a2d6be79f4ca03ee09a99d2866f340faed49458913449aefa` |
| jsonschema-regex | 0.57.0 | `48d2120d8466ffcdc1b3be4b88ff0e9191bf5b6b09b1b1af1eef9aff8f465ea3` |
| jsonschema-value | 0.57.0 | `3fbfa40a42415369d940b3848f3ce08099f1f1ac04d73e8580f4d652617c4f44` |
| micromap | 0.3.0 | `c2a86d3146ed3995b5913c414f6664344b9617457320782e64f0bb44afd49d74` |
| num-bigint | 0.4.8 | `c89e69e7e0f03bea5ef08013795c25018e101932225a656383bd384495ecc367` |
| referencing | 0.57.0 | `b06f6798be4fed305e74df8b1fb90cf59c6b8cc17aa3febeb1830c0c6b627b3e` |
| regex | 1.13.1 | `f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d` |
| regex-automata | 0.4.18 | `ad8553b9b26413251cbf30e620595c7a41b3887f03da04579c0e6b0d6a06b4b2` |
| regex-syntax | 0.8.11 | `d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4` |
| serde_json | 1.0.151 | `c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14` |
| strum | 0.28.0 | `9628de9b8791db39ceda2b119bbe13134770b56c138ec1d3af810d045c04f9bd` |
| strum_macros | 0.28.0 | `ab85eea0270ee17587ed4156089e10b9e6880ee688791d45a905f5b1ca36f664` |
| syn | 3.0.6 | `8593e8e72159ed2257d083c7a454a85cbf854f37a0966d8d483aff8c8a3ebcee` |
| thiserror | 2.0.21 | `09e52cb86a36cede5cb101bf8908837b3e4c6e5e59fe7fd85c23fb56200d189e` |
| thiserror-impl | 2.0.21 | `fe5197923287db20a58125f0bc85c062f7f2c892de97b18c356f9efb14b28524` |
| uuid | 1.26.1 | `2ef6dac1e96601b4fb3acccccff2139741fcb757cb9a36089bf5be91cfb285ce` |
| zmij | 1.0.23 | `29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b` |

## Approval evidence and limits

In Codex chat `01a110f1-0063-72a0-a56c-c1d3e8cf236d`, the human user answered **“Approve policy and listed temporary exceptions”** to the complete policy/20-package question on 2026-10-06. GitHub account identity was verified as `brianluby`. This records that exception/policy disposition; it is not a source-audit sign-off or product/release approval.

The committed import snapshot remains unchanged. Legacy baseline bytes remain unchanged. Existing native audit records remain unchanged; no synthetic review sidecars are added. The default incremental gate may pass under this disposition. Strict runtime review and full-store validation still expose legacy unowned exemptions and remain unfinished acceptance work.

## Additional chacha20 disposition — approved 2026-10-06

The stronger inventory identified a separate gap omitted from the initial native-vet list: `chacha20 0.10.2` has an agent-authored differential audit from `0.10.1`, rooted in a legacy exemption. That path does not establish a full-rooted accepted review.

The owner approved this additional temporary exception in the same Codex chat on October 6. After being told the exact version, owner, review and expiry dates and given the checksum-bound addendum, the user answered **“Approved”**. The exception binds crates.io source `registry+https://github.com/rust-lang/crates.io-index` and archive SHA-256 `65c35e4b699c7e15ccbe7ee35c005e4fc0a278d22238a2857e6ce2dadeda1b06`; owner `brianluby`; created October 6; review by October 20; expiry November 5, 2026; criterion `safe-to-deploy`.

This authorizes incremental integration while source review remains outstanding. Existing native audit records, exemptions, import snapshot and legacy baseline remain unchanged. It grants no source-audit sign-off or release acceptance. The total owner-approved exception set is now 21 identities; the original 20-package table remains the record of the earlier decision.

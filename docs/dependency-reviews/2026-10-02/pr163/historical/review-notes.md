# PR 163 — pdf-extract coordinated review evidence

Read the independent detailed source note in `reviewer-notes.md`. This coordinated
packet is agent-assisted read-only evidence, not human signoff, accepted cargo-vet
certification, owner disposition or F03 acceptance. No Cargo or repository
mutations were performed.

- PR: https://github.com/policy-forge/forge/pull/163
- Head: `db34fdfccaf00262e9706e8f3cd294f9d1f3bc01`
- Base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`
- Existing CI: https://github.com/policy-forge/forge/actions/runs/36143881134

Only pdf-extract 0.12.0 → 0.12.1 changes in the lockfile; production manifest,
dependency edges, package features and build behavior are unchanged. Both
archives match exact lock checksums and their upstream VCS identities resolve.
The sole executable delta clears cached fonts before each page's resource
lookup, correcting cross-page reuse of a font name with different font data.
Forge reaches this behavior through extract_text(path). No actionable new
regression was identified by the independent reviewer.

Existing September 25 Ubuntu/macOS/Windows tests, cargo-audit and cargo-deny
passed; cargo-vet failed with twenty shared baseline gaps plus pdf-extract
0.12.1. The existing Forge PDF fixture is a blank PDF and does not establish
the corrected multi-page font behavior. No new regression test was executed.

The frozen native PDF closures match ten records in RustSec tree
`117edb3bed98e9be112f277b7615eea3252e7c43`. Vulnerability/unsound records
have patched locked versions, including lopdf 0.42.0 and postscript 0.14.1.
ttf-parser 0.25.1 has the informational unmaintained RUSTSEC-2026-0192 record,
with no patched version. This warning is already retained in CI cargo-audit logs
and explicitly ignored by existing deny.toml with a deps-rotation tracking
rationale. Preserve that existing disposition; it is not a newly introduced
vulnerability. D069/F02 still must qualify its owner/validity evidence.

Independent baseline source limitations remain explicit: output bounds apply
after extraction; malformed PDF panics and recursive Form resource handling are
not made safe by this patch; input bytes are fingerprinted before reopening the
path for extraction. These observations are not newly validated vulnerability
findings, and no complete baseline PDF parser audit is claimed.

Remaining: accepted attributable audit under D069, corrected-font regression
evidence and supported candidate verification, inherited baseline dispositions,
and the user-required full documentation review/update after completed roadmap
integration. Old pdf-extract has an exemption rather than a local source audit;
this differential does not establish unchanged-source approval.

# Forge F02/F03 design and owner decision packet

This is a proposal, not an audit or acceptance record. No approval, exception ownership, human signature, or trusted-source acceptance is inferred from existing code or documentation.

## Verified local baseline

- Inspected implementation checkout HEAD `704df7213b1aa9967e805bb0b0bd40bacbcfc12d`.
- `Cargo.lock` SHA-256: `5b899b7f1bf0ab12075c1bf5542f308b1f1ca7c18fedec84a4926510ee4f846f`.
- Lockfile contains 312 packages: first-party Forge and 311 external packages. Store has 319 bare exemptions, 268 marked safe-to-deploy and 51 safe-to-run; their fields are only version and criteria. 31 exemptions name versions absent from the current lock.
- Local audits contain one `who = "Codex"` chacha20 `0.10.1 -> 0.10.2` differential. Its 0.10.1 base is an exemption, not an audited base; it cannot establish PRD069 M-7 acceptance. No workspace stack exemption has an attributable local source audit in the inspected store.
- Configured imported sources are Embark Studios and Mozilla. `imports.lock` has full audits for similar 2.2.1 and utf8parse 0.2.1 from Embark, plus deltas to similar 2.7.0 and utf8parse 0.2.2 from Mozilla. No extra registry should be inferred from README's historical reference.
- CI already separates cargo-audit, cargo-deny and cargo-vet from lint/test. Existing `cargo vet --locked` prevents fetching new imported audits, not all network access. Local help says `--frozen` requires locked and cached/vendored sources; `--no-minimize-exemptions` prevents check/certify's automatic exemption cleanup.
- Existing schemas/oscal-schema-manifest.json and release SBOM, checksum and provenance jobs are reconciliation seams. They do not establish an audited denominator or per-release inventory evidence.
- Current local changes observed: .claude/settings.json modified; .veans.yml and the completion-tranche plan untracked. All preserved.

## Decisions requiring a real owner disposition

The PRD still lists the following three blocking questions, and no later disposition was found in its decision log or the authoring gate register. Existing policy decisions (new-runtime gate rather than flag day, owned/reviewed exceptions, runtime exposure ordering) remain intact.

1. **Audit authority.** Recommended: agents may prepare evidence and review notes, but a local audit becomes accepted evidence only after a named human explicitly signs off on the exact crate source/version/checksum and criterion. Record assistant identity separately from signer identity. Preserve the existing Codex delta as assisted/pending evidence unless an authentic sign-off and audited base exist. Alternative: human-only review with no agent contribution. Owner must name the auditor and explain the sign-off record format.
2. **Exceptions.** Recommended: every accepted exception is bounded and revisited, with nonblank owner and rationale, recorded creation/approval date, review-by date, expiry and an approval reference. No permanent exception in this tranche. Proposed maximum initial term: 90 days, with explicit renewal producing a new reviewable record. The maximum term is a proposed product choice; it is not already approved. Review-day boundary should be `as_of > review_by` for stale and `as_of > expires` for expired, consistent with “past its review date”. Owner must confirm both the term policy and the handling of all current legacy rows.
3. **Trusted imports.** Recommended: retain only the two currently configured sources in the proposal, bind approval to source name, exact URL and committed snapshot digest, and require Brian Luby's recorded approval for additions or URL changes. Refresh only through an explicit maintainer command yielding a normal diff. A configured URL by itself is not a fabricated approval. Owner must confirm the current trusted list and approver.

Non-blocking questions can be scoped explicitly: class test/build-only packages safe-to-run and runtime packages safe-to-deploy over the supported release matrix; identify duplication/retired exemptions before proposing graph reduction. No source/version update is needed for F02.

## Minimal implementation using existing alternatives

Use a repository maintenance script, `scripts/dependency_inventory.py`, with Python 3.11+ standard library (`tomllib`, `json`, `hashlib`, `datetime`, `statistics`, `argparse`). There are already Python verification scripts. This adds no runtime dependency and avoids modifying Forge CLI/error/schema contracts. CI should select Python 3.11+ explicitly. If a shipping CLI becomes an actual requirement, existing `toml`, `serde`, `serde_json`, `chrono`, and shared hashing already supply the Rust alternative; no new crate is justified.

Proposed commands:

- `inventory --as-of YYYY-MM-DD --format json|text`: committed inputs only, canonical byte output, exact lock identities and explicit evidence status.
- `gate --as-of YYYY-MM-DD --baseline <immutable-baseline>`: same evaluator, report-only legacy gap treatment only if owner explicitly authorizes it; no acceptance upgrade from baseline membership.
- `validate-store`: strict parsing and schema/proof validation using the same 0/1/2 contract.
- `refresh-graph`: explicitly runs locked Cargo metadata and emits a normal committed graph diff; never invoked by inventory/gate. Requires the local crate cache and therefore is not fresh-clone offline verification.
- `refresh-imports`: documented cargo-vet explicit refresh operation; a maintainer reviews the exact snapshot diff. Never silently called from CI or inventory.

Inputs:

- Cargo.lock and Cargo.toml, keyed by package name/version/source and checksum rather than just name.
- Existing supply-chain/config.toml, audits.toml and imports.lock, preserved as cargo-vet stores.
- Versioned `supply-chain/dependency-graph.json`: normalized package identities, edge kinds, proc-macro flags, supported targets, explicit feature set and hashes of lock and every workspace manifest. This is essential because cargo metadata --offline needs cached crate manifests; it cannot satisfy AC-1 on a fresh offline clone by itself.
- Versioned `supply-chain/policy.toml`: actual owner decisions, criteria implications, approved source URLs/snapshot hash bindings, date rules and any explicit legacy baseline authorization. A proposal must remain pending until disposition.
- Versioned `supply-chain/exceptions.toml` or JSON sidecar: exact exception identity/source/checksum plus owner, rationale, created/review/expiry dates and approval reference. This avoids relying on cargo-vet accepting additional fields in exemption tables.
- If needed, a separate signed-off local review metadata sidecar binds cargo-vet records to the exact evidence record and human signer. Do not rewrite external auditor identities or add invented audits.

Graph rules: start at the first-party normal dependencies for each supported target and traverse normal edges; stop proc-macro packages as build-time nodes. Runtime reachability wins if the same package also appears in a test/build path. Record build/proc-macro/unsupported-target reachability separately rather than calling every locked crate shipping. Preserve all locked entries in inventory, including packages outside the supported shipping closure. Validate exact identity coverage and manifest/lock digests. Host build-script/proc-macro feature resolution needs explicit verification before freezing acceptance, because metadata's dev-feature unification may over-approximate the shipping graph.

Proof evaluation: safe-to-deploy implies safe-to-run; the reverse never satisfies runtime review. A full audit starts a path. Differential records extend only from an already audited exact version with the required criterion; a bare exemption does not establish an audited base. Reject orphan delta paths and avoid treating cycles as proof. Imported paths use only approved configured source snapshots; every imported source must remain explicitly pinned. Distinguish local full/delta, imported full/delta, owned exception, legacy unowned, unreviewed and first-party statuses. When reporting an audited chain, include the actual path and all signer/source references, not only the terminal record.

Dates: canonical output requires an explicit as-of date. Age and stale status are not deterministic if an implicit clock changes. CI supplies the date as a visible input; repeated inventory with identical lock/store/graph/policy/as-of date must produce identical bytes. Omit absolute local paths, transient timestamps and nondeterministic iteration order from canonical JSON.

Exit contract: `2` dominates malformed or inconsistent input (TOML/JSON parse error, unknown schema field, missing identity, stale graph hash, invalid exception dates/owner, unsupported source, orphan differential or unapproved import policy). Valid inputs needing action produce `1` (new unvetted runtime identity/version, stale/expired exception). `0` means the defined gate is clean, not that every dependency was audited or that the product is accepted. Diagnostic JSON keeps exact named identities and consistent issue codes.

## The legacy policy seam cannot be papered over

PRD069 requires both a gate that does not fail solely because old bare exemptions exist and an exemption policy requiring owner/rationale/review dates. These are different completion stages. Do not assign Brian as owner of 319 exceptions merely because he owns the PRD, or fabricate creation/expiry dates.

Preferred proposal: the inventory reports all legacy rows honestly; a committed baseline may allow the incremental new-runtime gate to run while a separate strict store/readiness report remains blocked. Baseline additions or same-name version changes are never accepted as legacy. The strict MVP gate cannot be declared accepted until legacy exceptions are converted to owner-approved bounded records or authentic audits. Owner must approve transitional behavior explicitly; without it, the new implementation should report the malformed/unowned store honestly and CI acceptance remains pending. Mechanical tests and a decision packet can be completed independently of this disposition.

## Meaningful acceptance cases

1. Fresh clone with empty Cargo cache and denied network: inventory/gate still operate using committed input files only.
2. Every locked identity appears exactly once, including duplicate crate names, build/proc-macro packages and unsupported-target packages. First-party Forge is labeled explicitly.
3. Same full input set and as-of date yields byte-identical output despite permuted input ordering and different checkout paths.
4. New runtime name or exact version lacks proof: exit 1, exact identity named. Adding it to a changed baseline does not silently grandfather it.
5. Exact full local/imported accepted audit: exit 0 for that identity; safe-to-run proof cannot satisfy safe-to-deploy.
6. Delta with an authenticated reviewed base and exact version pair succeeds; exemption-only base, reversed path, orphan chain and audit cycle fail closed.
7. Valid owned/unexpired exception succeeds; missing owner/rationale/review date, invalid date or impossible ordering exits 2.
8. Review date equal to as-of is not past due; the following day reports stale and exit 1. Expiry uses the approved boundary and never masks stale reporting.
9. Imported source absent from config/approved list, source URL change or snapshot mismatch fails closed; explicit reviewed snapshot refresh is the sole update path.
10. Lock/manifest feature/target change makes graph stale rather than inheriting old class labels; proc-macro/build-only paths and runtime+dev dual paths have correct class precedence.
11. Baseline retains legacy unreviewed status and metrics; expired/stale owned exceptions still require action. No count labels a legacy exemption reviewed.
12. Current first runtime tranche fails strict readiness because its 41 provisional runtime identities are bare-exempted. This is an authentic expected baseline, not an acceptance failure to disguise.

## F03 preparation and remaining release work

Freeze supported graph and source archive/checksum evidence before source review. The scratch class graph at [committed exploratory classification receipt](2026-10-02-dependency-classification-exploration.json) derives 212 external runtime identities and a 41-package PRD062 normal-edge closure over the four existing release triples. Per platform: Linux x86_64 207 runtime /39 stack; Intel macOS 207/39; arm64 macOS 207/38; Windows MSVC 205/38. Unfiltered metadata would instead include 229 external runtime/51 stack and wrongly enlarge the shipping scope to unsupported targets. These are provisional classification counts, not proof of compiled artifact membership or completed audits.

Cargo metadata commands used `--offline --locked --format-version 1 --filter-platform <target>` with default features. Child RUSTC_WRAPPER was cleared after sandbox-blocked sccache failed rustc -vV. No Cargo build/test ran, and store/manifest/lock were not changed. The graph file carries HEAD, lock/manifest hashes, invocations, traversal rule and bounds.

Prepare auditable packets for the 41 actual stack candidates: exact source bytes/checksum, criterion, platform/feature exposure, capabilities, untrusted-input handlers, unsafe/FFI/build hooks, reviewed files, findings, uncertainties and real signer disposition. Dependency PRs need individual exact-head reviews and current advisory/CI results; the script must not mass-certify packages or blindly refresh exemptions.

F22 must reconcile exact release graph/inventory to SBOM and schema manifest hashes, report audited/excepted/unreviewed denominators and exception-age statistics, retain per-release historical trends, and include inventory/trend/schema evidence in artifact hashes and provenance subjects. The current final checksum glob accepts only archives and CycloneDX JSON and will require an explicit extension when inventory artifacts are introduced. Publishing remains separately authorized.

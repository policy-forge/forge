# FORGE support policy — DRAFT proposal for owner approval

> **Status:** PROPOSED — drafted 2026-11-12 by the F22 release-scaffolding
> slice. Nothing here is in force until the owner approves it. Owner:
> Brian Luby. Basis: `b4226bb35d796222c30f93235ac22118dd26a344`.
>
> This proposal deliberately picks conservative, small-team defaults. Every
> window below is a decision the owner may change; the machinery referenced
> (supply-chain gates, exception discipline) already exists and is not
> renegotiated here.

## 1. Version support window (proposed)

- **Supported line:** the latest released minor of the current major only.
  On 2.0.0 publication, the 2.0.x line is supported; 1.x enters
  security-only maintenance (Section 2) for **90 days**, then end-of-life.
- **Patch cadence:** patches ship when a fix or security update warrants it;
  no calendar promise. Every release goes through the same gates
  ([runbook](runbook.md)); there is no fast lane that skips supply-chain gates.
- **Pre-releases:** RC tags (`v2.0.0-rc.N`) are test artifacts. They receive no
  support and must never be presented as releases; the release workflow
  currently treats any `v*` tag as a release trigger, so RC discipline is an
  owner/manual gate (see runbook §2).
- **EOL:** an EOL version receives no builds, no fixes, and no security
  backports. EOL dates are announced in the release notes of the superseding
  version and in `CHANGELOG.md`.

## 2. Security-update policy (proposed)

Grounded in [PRD 069](../PRD/069-prd-dependency-security-audit.md) and the
existing store discipline in [`supply-chain/README.md`](../../supply-chain/README.md):

1. **Advisory intake.** `cargo audit` (RustSec) and `cargo deny check` run in
   CI on every push/PR and are release gates. A published advisory affecting
   a locked runtime (`safe-to-deploy`) dependency is triaged within **7 days**;
   actively-exploited or trivially-reachable issues are triaged within **48
   hours** (owner judgment, recorded in the ticket).
2. **Fix path.** Update the dependency, refresh the committed graph
   (`python3 -B scripts/dependency_graph.py refresh` after review), re-run
   `cargo vet --locked` and the inventory gate
   (`python3 -B scripts/dependency_inventory.py gate --as-of <date> --strict`),
   and ship a patch release through the normal runbook. A security patch may
   not widen the exempted surface: new versions need an audit, a differential
   audit (PRD 069 M-7), or a named exception — never a silent exemption.
3. **Exception discipline (existing, not renegotiated).** Every temporary
   exception in `supply-chain/dependency-exceptions.json` carries an owner,
   rationale, exact identity + checksum, `review_by` and `expires_on` dates
   (PRD 069 M-5/M-6/M-9). Exceptions are integration permission only — not
   audit credit, safety certification, or release acceptance. **No release is
   authorized while an exception covering a shipped runtime crate is expired
   or past its review date without an owner renewal recorded in
   `docs/plans/`.** As of the preparation date all 47 standing exceptions
   carry `review_by 2026-11-07` / `expires_on 2026-11-23`; the review date has
   passed and renewal or audit closure is a live release blocker.
4. **Disclosure.** Security fixes are described in release notes and
   `CHANGELOG.md` with the advisory ID. FORGE has no private disclosure
   channel today; establishing one (security contact / advisory process) is
   proposed as a Should for the owner to accept or defer explicitly.
5. **Backports.** Security fixes are backported only to the supported line
   and the security-only maintenance window (Section 1). Older lines get a
   documented "upgrade to supported version" statement instead.

## 3. Deprecation rules (proposed)

- **CLI surface:** a subcommand, flag, or output contract is deprecated in a
  minor release with (a) a `CHANGELOG.md` entry, (b) a runtime warning where
  practical, and (c) a stated removal version at least **one minor release
  later**. Removal happens only in a major release unless the deprecated item
  is a documented security hazard.
- **Output contracts:** OSCAL output stays on the pinned schema baseline
  ([OSCAL_COMPATIBILITY.md](../OSCAL_COMPATIBILITY.md)); a schema-baseline
  upgrade follows that document's patch-evaluation runbook, never a silent
  bump. FORGE-native JSON contracts (`forge.*/N`) version by suffix; a new
  suffix ships alongside the old one for at least one minor release.
- **Public Rust API:** the library crate follows semver as gated by the API
  contract tests (`cargo test --test api_contract_validation`); breaking
  changes require a major version and a migration note.
- **Platform/format support:** dropping a platform target or input format is
  a deprecation under the same one-minor notice rule and must be reflected in
  the [support matrix](support-matrix.md) in the same PR.
- **Dependencies:** removing or replacing a dependency follows PRD 069; the
  approval record (PRD/PR naming crate, reason, approver) is the audit trail.

## 4. What this policy does not do

- It does not certify any third-party crate as safe (PRD 069 non-goal).
- It does not promise response-time SLAs to external users; the triage
  windows are internal working targets.
- It does not authorize any release. Publication remains the owner's
  explicit act ([runbook](runbook.md) §7).

## Approval record

- [ ] Owner approves / amends Sections 1–3 (record decision in `docs/plans/`
      and link it here).
- [ ] Owner accepts or defers the disclosure-channel Should (§2.4).
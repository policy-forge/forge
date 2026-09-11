# Authoring library API migration

Guidance for the next major release carrying PRD-061 Phase 1, Phase 2 and S-1.
This is a recorded decision plus consumer guidance, not a release authorization;
package version and `Cargo.lock` stay unchanged until the release itself.

## Decision

The tranches add variants and fields to **public, non-`#[non_exhaustive]` enums**,
so downstream exhaustive matches can stop compiling. Under Cargo semver that is a
breaking change: the release containing these tranches is **`2.0.0`**, not a minor
bump. (Owner disposition, 2026-09-11; see [authoring gates](authoring-gates.md).)

## What changed

- `forge::cli::AuthorCommand` is a public enum. PRD-061 added `Plan`, `Build`,
  `Impact`, `Handoff` and (S-1) `Scaffold`, and later tranches added fields such
  as `components`, `html` and `format`. It is also the clap-parsed CLI surface.
- `forge::error::ForgeError` is a public enum that gained authoring variants
  (for example `Authoring` and `AuthoringActionRequired`) in Phase 1. Phase 2 did
  not change it further, but it is part of this gate.
- New public modules and types were added under `forge::authoring` (closed
  manifest types, plan/provenance types, `execute_*` entry points) and new
  closed JSON schemas were added. Additions are not breaking by themselves.

## Consumer migration

- **Exhaustive `match` on `AuthorCommand` or `ForgeError`:** add the new arms, or
  add a trailing `_ => ...`. Prefer matching the variants you handle and letting
  the rest fall through, so future additions do not break compilation again.
- **Constructing `AuthorCommand`:** unchanged; the enum is only produced by clap
  parsing in the binary (`forge author <command> --manifest ...`).
- **Constructing or matching `ForgeError`:** existing variants keep their meaning.
  New authoring variants report invalid authoring input (`Authoring`) and
  "valid artifacts, authoring work remains" (`AuthoringActionRequired`, exit 1).
- **Machine contracts are unaffected:** `forge.author-project/1`,
  `forge.authoring-pack/1`, `forge.authoring-plan/1` default outputs,
  `forge.authoring-provenance/1`, `/2` component outputs, and exit codes 0/1/2
  keep their meaning and bytes. Schema versions bump only where documented
  (`/2` component plans and provenance).
- **Not changed:** package version, `Cargo.lock`, and every existing `/1`
  contract.

## Alternatives considered

- **Mark the public enums `#[non_exhaustive]` and ship a minor release.**
  Adding `#[non_exhaustive]` is itself breaking for downstream exhaustive
  matches and for external struct-literal construction, so it cannot land in a
  minor release either. It remains a reasonable follow-up *inside* `2.0.0` so
  that later variants become minor-compatible; it is not required for this
  migration and no source change is made here.
- **Declare the Rust library an unsupported/CLI-only surface and ship `1.2.0`.**
  Rejected: `src/lib.rs` re-exports the modules the CLI uses and the `testing`
  feature explicitly targets downstream test harnesses, so "unsupported" would
  have to be enforced rather than asserted.

## Evidence required at release

The release review must confirm this guidance still matches the shipped public
surface and link it from the release notes. The gate's supporting decision is
recorded in [authoring gates](authoring-gates.md); the release itself remains
open there.

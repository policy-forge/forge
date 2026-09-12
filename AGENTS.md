# forge-022-golden-file-edge-cases Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-02-21

## Active Technologies

- Rust edition 2024, stable 1.93.0 + Existing `serde_json`, `insta` (`json` feature), `tempfile`, `regex`, `tracing`; no new crate dependencies required (022-golden-file-edge-cases)

## Project Structure

```text
src/
tests/
```

## Commands

- `cargo test`
- `cargo clippy`

## Code Style

Rust edition 2024, stable 1.93.0: Follow standard conventions

## Recent Changes

- 022-golden-file-edge-cases: Added Rust edition 2024, stable 1.93.0 + Existing `serde_json`, `insta` (`json` feature), `tempfile`, `regex`, `tracing`; no new crate dependencies required

<!-- MANUAL ADDITIONS START -->

## Dependency policy (supersedes the generated "no new crate dependencies" line above)

That line records the constraints of feature 022 and is auto-generated; it is not
the current policy. **New crate dependencies require approval before they are
added.** Check for an existing alternative first, then record the approval where a
reviewer can find it — in the PRD that mandates the dependency, or in the pull
request that introduces it — naming the crate, the reason, and who approved it.
An unapproved dependency must not be added; an approved one needs a supply-chain
entry. See [PRD 069](docs/PRD/069-prd-dependency-security-audit.md) for the audit
and exception policy, and `.github/copilot-instructions.md` for the approved set.
<!-- MANUAL ADDITIONS END -->

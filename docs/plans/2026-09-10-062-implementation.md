# PRD-062 implementation plan

Implementation starts from freshly fetched `origin/main` at
`3ac6815868295d3d306c69cc555108870d8acffb`, including the merged Slice 0
contract and PRD-061 Phases 1 and 2. Work is isolated on
`codex/062-local-web-workspace`. Existing worktrees are preserved.

## Delivery order

1. Read-only services, confined resource index, bounded local session and HTTP
   adapter, maintained headless client, embedded explorer.
2. Prepared single-file effects, exact-byte receipts, conditional commit,
   idempotency and response-loss recovery. This isolated implementation develops
   read and write paths together behind explicit launch; no internal release or
   Slice 1 acceptance is claimed before its remaining gates pass.
3. Shared applicability/mapping services and review editors, conversion,
   trace navigation and static reports, with API/browser/CLI parity.
4. Packaged/offline checks, documentation and recovery guidance. Pilot
   distribution and release approval remain human gates.

## Contract ownership

The committed OpenAPI 3.1 artifact remains normative. Closed domain `/1`
contracts retain their meaning. Runtime types and browser/headless clients
must validate against the existing fixtures and schema; any necessary contract
correction is separately documented and tested before its implementation.
The UI follows accepted ADR-0003: semantic HTML, CSS and native JavaScript
modules embedded in the binary, without a build toolchain or remote assets.

Shared services own validation, preparation and classification. HTTP owns
transport/authentication, the browser owns presentation, and CLI exit/text
compatibility remains unchanged. New public Rust APIs require explicit
compatibility review; implementation internals stay crate-private where possible.

## Acceptance evidence

| Boundary | Required evidence |
|---|---|
| Baseline | Locked full Rust suite on the freshly fetched baseline |
| Index | Closed schema, duplicates, versions, ordering, no scanning, bounds, path/alias rejection |
| Containment | Symlink/reparse/hard-link/special-file rejection, root and ancestor replacement, outside-root non-disclosure |
| Session | No-echo prompt, fixed Argon2id parameters, scoped independent random capabilities, constant-time verification, throttling, shutdown |
| HTTP | Exact Host/Origin/Fetch Metadata, no CORS, pre-body rejection, bounded framing/body/depth/rate/concurrency, safe errors |
| Read views | Full denominators, deterministic version-bound pagination, validated domain state, no inferred decisions |
| Effects | Exact preview/hash/version bindings, expiry, single-use, idempotency conflict/replay, stale input, response loss, interruption |
| UI | All requests mapped to OpenAPI operations; offline assets; escaping; memory-only secrets/drafts; keyboard, focus, reflow and live status |
| Parity | Shared service outputs and existing CLI artifacts/classifications match; headless and browser use the same API |
| Delivery | fmt, strict clippy, locked tests, contract tests, complete-diff review, exact-commit OCR coverage and exact-head CI |

Human product/security/accessibility approval, five-user studies, assistive
technology evaluation, pilot metrics and release approval are not inferred
from implementation, merged architecture documents, or passing tests.

## Current contract decisions

Unreleased API 1.1.0 adds closed applicability/mapping initialization operations
and optional provenance references. Mapping initialization requires its first
explicit reviewed relationship; the existing PRD-055 `/1` nonempty constraint
is preserved. The private workspace-report/1 canonical HTML envelope binds
redacted counts to exact input hashes. See `docs/local-workspace.md` for limits,
transaction assumptions, migration implications, and remaining human gates.

External OCR review of the PRD-062 diff and fetching the development-only
Playwright CI dependency await destination/dependency approval following automatic
approval-review rejections. Existing local browser tooling may be used without
downloading dependencies. These are not completed review/CI gates.

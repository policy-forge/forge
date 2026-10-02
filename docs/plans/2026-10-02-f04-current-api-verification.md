# Current workspace API verification

The current verifier reconciles the delivered F04 foundation with the F19
operation and metadata-bundle workflows. The older
[F04 plan](2026-10-02-f04-workspace-verification.md) and its development
receipts remain historical evidence. Their 12 check groups and 37-operation
denominator do not describe the current client.

The `Workspace verification` workflow is configured to retain a separate API receipt on Linux,
macOS and Windows. Both it and the main `CI` workflow now accept ordinary
`pull_request` events for every base branch, including stacked PRs. Pushes remain
limited to `main`; the dedicated workflow also supports manual dispatch. The
existing strict lint, tests and supply-chain checks still apply. Actions remain
pinned, token permissions remain `contents: read`, and checkout does not persist
credentials. This change adds no browser installation or hosted browser job.

## Current receipt contracts

`forge.workspace-client-verification/2` has 16 required assertion groups. The
current normative OpenAPI contains 39 operation identifiers. Each actual run
retains a complete observed/unobserved partition and only integer counts for
`succeeded`, `rejected` and `transport_failed` outcomes. Assertion groups,
distinct operations and actual request attempts are separate denominators.

The two synchronous client sessions share a 10,000-attempt bound. Every dispatched
attempt must reconcile to one permitted outcome. Undocumented routes,
classification errors, unexpected delegate errors, exceeded bounds and broken
accounting latch failure, including errors swallowed by session cleanup.
Historical client `/1`, floating-point or boolean count headers, and an aggregate
outcome total above the shared bound cannot earn current pass credit. Python
optimization is rejected because it disables the conformance assertions.

`forge.workspace-verification/2` runs three predefined suites: the Rust API
contract suite, Rust workspace process fixtures, and the maintained Python
client. The wrapper captures exact input and supplied release-binary hashes
before and after execution. It also hashes the bounded raw Git commit object
using its Git object envelope, confirms HEAD again, and retains its ordered
parents. Authors, commit messages and raw subprocess output are discarded.

The `checkout` object distinguishes the event, checkout kind, requested head,
requested base and actual tested commit. A PR merge checkout requires exactly
`[requested_base, requested_head]` as its ordered parents and must match the
requested tested commit. A push, manual run or explicit PR-head checkout must
test its requested head. Raw-object capture works in a shallow checkout without
loading the parent objects. Local runs allow an optional expected commit but
receive no hosted-context credit. Workflow context is an assertion checked
against the captured Git object; it is not independent remote-event, startup,
reproducibility or release-provenance attestation.

Only the maintained client uses the supplied release executable bytes. Cargo
suites use test-profile executables; their executable hashes remain unmeasured.
The build's locked/offline/default-feature/release fields record a workflow-step
assertion. They do not establish OS runtime network denial or independently prove
how an arbitrary supplied executable was produced.

## Status and safe publication

A wrapper is `passed` only when all three suites pass, the provided build outcome
is successful, required tool fields are available, tracked source is clean and
captured inputs remain unchanged. A failed suite retains its failed status while
the other selected suites continue. Missing suites, ignored/filtered/measured
Rust cases, a dirty source tree or an unqualified build prevent complete slice
credit. Wrapper exit codes are 0 for passed, 1 for incomplete and 2 for failed.
All acceptance gates listed in the receipt remain open after a slice passes.

Receipts use sorted ASCII JSON with one LF and private staging followed by
no-replacement hard-link publication. An existing destination, including a
dangling symlink, is preserved. Unsupported publication or cleanup returns a
failure. Cleanup can fail **after** the final hard link exists, leaving a
passed-looking file and a private staging file while the producer exits nonzero.
The wrapper requires producer exit 0 before reading a client receipt; a file's
`status` alone is insufficient. Retained errors are fixed categories without raw
capabilities, passphrases, request/response bodies, project/native paths or
exception text.

The line-ending policy explicitly pins existing JS, CJS, CSS and Python files to
LF for future cross-platform input identity. Official schema assets and other
binary/provenance exceptions keep their existing rules. Historical Windows CRLF
observations are preserved rather than rewritten as current measurements.

## Reproducing the API slice

Provision locked Cargo inputs first; provisioning may use the network. Run the
following from a clean checkout after a successful release build. Use an empty,
new output directory for every attempt:

```sh
cargo fetch --locked
cargo build --locked --offline --release
python3 -B scripts/test_verify_workspace.py
python3 -B scripts/verify_workspace.py \
  --forge target/release/forge \
  --output-dir workspace-evidence-new \
  --expected-commit "$(git rev-parse HEAD)" \
  --build-outcome success \
  --suite api-contract --suite api-workflow --suite maintained-client
```

On Windows, use the selected Python interpreter and `target/release/forge.exe`.
The workflow supplies its own event/head/base fields through quoted environment
arguments; the command above intentionally records local context. A pre-commit
run can pass every suite while the wrapper remains incomplete because tracked
source is dirty. The repository's mandatory commit hook remains enabled.

See the [current development verification record](../workspace-current-api-verification.md)
for exact source, documentation and primary-thread Python line observations.
The record retains zero lines, unobserved operations, failed coverage-collector
attempts and the pre-commit incomplete result. Definition/signature hits are
separate from function body execution; background threads and subprocess line
events are untraced. Browser/Rust branch coverage is outside that measurement.

## Remaining gates

The full supported browser/platform matrix, Windows controlling-terminal unlock,
named Chrome/Edge/Firefox/Safari products, packaged-runtime OS network denial,
complete API/CLI parity, automated accessibility rules and manual assistive
technology evaluation remain separate. Dependency/security approval, user and
pilot evaluation, package/provenance/release approval and all remaining scoped
Must/Should requirements also remain open. Local tests and draft PRs establish
neither those acceptance decisions nor publication authorization.

The final documentation review must reconcile all completed roadmap work against
the integrated CLI/API, examples, architecture, schemas, dependencies and
provenance, platform claims, requirement evidence and acceptance status. This
focused verification guide does not close that end-of-roadmap gate.

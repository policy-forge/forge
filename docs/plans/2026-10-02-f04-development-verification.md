# F04 API foundation: development verification

Documentation and coverage were checked before drafting the PR. The
[machine-readable receipt](2026-10-02-f04-development-verification.json) retains
source hashes, uncovered lines, tools, execution bindings and raw coverage hashes.
Base: `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. No Rust source, crate or lockfile changes.

| Production script | Function/method docstrings | Executable lines covered |
| --- | ---: | ---: |
| `scripts/verify_workspace.py` | 15/15 | 329/366 (89.89%) |
| `scripts/test_workspace_client.py` | 4/4 | 145/154 (94.16%) |
| **Combined** | **19/19 (100%)** | **474/520 (91.15%)** |

Docstring counts use Python AST declarations, including nested functions. This
checks presence, with API contracts separately reviewed; it is not whole-repository
documentation coverage. Installed Python 3.14.8 standard-library
tracing and compiler executable-line identification supply the line denominator.
Fixture, live verifier, live client and actual CLI counts combine only when source
hashes agree. Comments, docstrings and uninstrumented lines are excluded; branch,
path and subprocess coverage are not inferred. The client was additionally traced
directly against the actual release binary. Rust coverage is outside this percentage.

**24 fixture tests passed, 0 skipped.** Cases cover bounded reads/output,
malformed/duplicate contracts, measured/ignored/filtered/empty denominators,
source drift, unavailable tools/builds, publication preservation, redacted
failures, operation accounting, execution bindings and process timeout.

Real local execution passed **8 API-contract tests, 11 workspace-process tests,
and 12 maintained-client assertions**. The client observed **21/37 declared
operations** and lists all 16 unobserved operations. Its release binary hash is
retained; Cargo test executables are explicitly identified as separate unmeasured
builds. The actual runner CLI returned **exit 1 / incomplete** because tracked
script edits were dirty during measurement, even though all three suites passed.
This grants no clean-commit qualification.

Remaining uncovered lines include CLI argument/error branches, I/O/capture-change
and cleanup errors, unsupported record forms and client failure branches.
Coverage review fixed four reachable defects: silent operation omission,
bounds after allocation, measured tests receiving success credit, and ambiguous
release/test-binary identity. Existing current-stable strict Clippy remains required.

The commit hook separately runs formatting, strict default-feature Clippy and the
full Rust suite. Clean committed and hosted results are reported with the PR;
these development receipts do not replace them. Browser products, OS network
denial, full parity, security/accessibility acceptance, dependency audits, authentic
product studies, packages/provenance and final documentation review remain open
as listed in the [F04 contract](2026-10-02-f04-workspace-verification.md).

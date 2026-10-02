# F04 workspace verification: API foundation

This slice adds hosted, retained verification for the existing local workspace.
It does not close F04, PRD 062, dependency acceptance, or release approval.
The full roadmap target remains 2.0.0, including every Must/Should requirement.
The final documentation review requested by the owner remains an integration
gate after the scoped roadmap work is completed.

## Executed suites and scope

The dedicated `Workspace verification` workflow runs on Linux, macOS and
Windows independently of the existing CI lint gate. It fetches locked Cargo
inputs, builds the release binary offline, then executes these predefined
suites sequentially through `scripts/verify_workspace.py`:

- `api-contract`: `cargo test --locked --offline --test api_contract_validation`
  runs the existing OpenAPI, closed schema, fixtures and capability-matrix gates.
- `api-workflow`: `cargo test --locked --offline --test workspace_cli_test`
  runs the existing real process/HTTP, authorization, transaction, provenance,
  redaction and mapping/applicability parity tests. Counts come from the test
  runner, not from historical documentation or a hardcoded success total.
- `maintained-client`: the existing standard-library Python client performs
  the documented API workflow against the actual release binary. Twelve
  explicit assertions cover unconfirmed upload, exact idempotent replay,
  conversion, explicit mapping meaning, omitted scope, validation, analysis,
  redacted committed download, read-only behavior and clean shutdown.

The contract inventory reads at most its byte bound plus one before UTF-8
decoding. Its supported YAML layout requires plain path/method keys and plain
operation IDs; underscores and hyphens are supported in IDs. Quoted/malformed
keys or IDs, missing IDs, duplicate declarations and unsupported layouts fail
explicitly. An operation is never silently dropped from the denominator.

The maintained-client receipt records only documented operation identifiers and
outcome counts. It exposes the complete declared operation denominator and the
unobserved operations. This is evidence of the executed client workflow; it
does not prove that every API operation or every browser action was exercised.

These three runner hosts do not establish every release architecture: the
separate x86_64 and aarch64 macOS package slots still need execution against
their exact packaged bytes. Receipts identify the actual Rust host rather
than assuming an architecture from the runner label.

The process tests compare API mapping and applicability bytes with ordinary
CLI output. Their conversion test compares independent workspace directories.
Complete API/CLI conversion, validation, trace and typed result-classification
parity remains open; the existing fixture suite is not mislabeled full parity.

## Receipt contract and failure behavior

`forge.workspace-verification/1` is canonical sorted ASCII JSON with one LF.
It binds the source commit and tracked cleanliness, the provided release binary
SHA-256/size under `identity.provided_release_binary`,
Cargo files, OpenAPI/capability/index contracts, embedded script/style/shell
source, the harness and Rust test sources. It records Python, Cargo, Rust,
Rust host, OS/release and architecture versions. Inputs and the binary are
hashed before and after execution; a change fails verification.

Raw command output is drained into bounded transient memory and never uploaded.
No bootstrap descriptor, bearer capability, passphrase, source/framework prose,
reviewer identity, native absolute path, request/response body or raw application
error is copied to a receipt. Only allowlisted checks, operation identifiers,
counts, hashes and fixed diagnostic codes are retained. Receipt publication
refuses existing output, and the output directory must be empty.

The receipt also records the release-build step outcome. A failed/skipped or
unrecorded build cannot qualify the slice, even if a cached binary remains.
This is a declared workflow-step binding, not independent build provenance.

All selected suites run even after another suite fails. A missing tool, binary,
malformed receipt, ambiguous/missing Rust summary, source drift or subprocess
failure fails the slice. An unselected suite, dirty tracked source, unavailable
tool identity, ignored/measured/filtered tests or another incomplete denominator prevents
a passing slice. A passing slice requires all three suites, a successful declared release build and clean unchanged
tracked inputs. Exit codes are 0 for this slice passed, 1 incomplete, 2 failed.
The workflow retains the receipt with `always()` even after a suite/build
failure; missing artifacts themselves fail upload. Job cancellation or failed
checkout can prevent artifact creation and must be reported as missing evidence.

Each suite has an explicit `execution` binding. Only `maintained-client`
executes the supplied release binary and carries its measured SHA-256/size.
The Rust suites execute Cargo's `test` profile integration-test artifacts; the
process suite also uses Cargo's `CARGO_BIN_EXE_forge` test-profile executable.
Their executable hashes are explicitly unmeasured. Their bindings identify the
source commit, Cargo manifest/lock, Rust/Cargo/host and exact test-source pins,
without assigning the provided release binary's hash to those executions.

The CI release build binds source and the supplied release binary through the build step and receipt;
this is not an independent proof of build provenance. The binary is not called
a verified published package. Existing current-stable strict Clippy remains
required and untouched; passing this workflow does not waive its failures.
The observed Rust 1.99 baseline lint regression and missing runtime crate audits
need their own corrections/dispositions. Rust 1.98.1 local results cannot be
substituted for a green current-stable hosted lint gate.

Local invocation after an explicit build (use a fresh output directory):

```sh
python3 -B scripts/verify_workspace.py --forge target/release/forge \
  --output-dir /tmp/forge-workspace-evidence --build-outcome success \
  --suite api-contract --suite api-workflow --suite maintained-client
```

Python 3.11 is selected in hosted CI. No browser packages, Node toolchain, rule
engine or Rust dependency are added by this slice. Existing production UI
assets remain framework-free and embedded by Cargo.

## Remaining verification and owner gates

Accepted ADR-0004 retains the following browser product slots; an engine proxy
is not substituted for the named product:

| Product | Platforms | Required releases | Depth | This slice |
|---|---|---|---|---|
| Chrome | Windows, macOS | current stable and previous major | full golden path, automated/manual | incomplete |
| Firefox | Windows, macOS | current stable and previous major | full golden path, automated/manual | incomplete |
| Edge | Windows | current stable | smoke | incomplete |
| Safari | macOS | current stable and previous major | full golden path, automated/manual | incomplete |

Actual product/version slots must be frozen and reported when executed. The
existing Chromium-only POSIX PTY browser harness is not run here. The previous
local record used already-installed Playwright 1.62.1 and Chrome; there is no
repository npm lockfile or approved hosted fetching evidence. Its documented
CI provisioning/lockfile gate remains pending. Playwright WebKit is not Safari
product evidence. Windows needs a genuine interactive unlock harness compatible
with its terminal boundary; machine-session evidence is not browser-unlock proof.

Packaged runtime OS denial is incomplete on Linux, macOS and Windows. It must
execute the packaged golden path with a qualified OS mechanism covering Forge,
browser and descendants, preserve loopback, deny external IPv4/IPv6, and retain
address-only attempt evidence plus positive/negative enforcement probes.
Request interception, browser launch flags, locked/offline Cargo, and a successful
API client are insufficient to close AC-16.

Automated accessibility rule-engine checks and manual keyboard, 200% zoom,
NVDA/Chrome, NVDA/Firefox and VoiceOver/Safari remain open. Existing narrow
viewport assertions do not establish WCAG 2.2 AA. F05 must retain security and
accessibility findings, actual owner dispositions and ASVS 5.0.0 scoping evidence.
Independent security acceptance, F03 dependency audits, five-user product study,
complete browser API coverage, packaging/checksums/SBOM/provenance, release
approval and the final full documentation review remain distinct gates.

The process runner kills the POSIX process group on timeout. Windows cleanup
currently kills the direct child and marks unclosed subprocess output as failed;
this helper is not a qualified cross-platform descendant-confinement mechanism.
The symlink fixture reports an explicit skip when Windows denies symlink creation
to the test user. Neither case is substituted for the independent F05/runtime
platform gates.

Script fixtures are synthetic tests of receipt failure behavior. They are not
hosted workspace results, participant judgments or acceptance evidence. Replace
no historical receipts or sign-off records with these fixtures.

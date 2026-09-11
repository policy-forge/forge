# PRD-062 implementation verification

This is technical evidence for the isolated local-workspace implementation,
not approval of PRD-062, a release, a pilot, or human readiness. The baseline
was freshly fetched main at `3ac6815868295d3d306c69cc555108870d8acffb`.

## Executable evidence

| Area | Evidence |
|---|---|
| Existing behavior | Full `cargo test --locked` passes, including existing authoring, lifecycle, composition and CLI suites |
| Static checks | `cargo fmt --check`, locked all-target clippy with warnings denied, and `git diff --check` |
| API contracts | Eight contract tests validate OpenAPI 3.1, all internal refs/component schemas, fixtures, the resource schema and both directions of the 37-operation capability matrix |
| API workflow | Ten process/HTTP integration tests cover bootstrap, authorization, resource roles, registration, conversion, scope/mapping initialization and edits, analysis, export, provenance and stale reports |
| Parity | Mapping and applicability bytes equal ordinary CLI output for the same explicit inputs; workspace conversion bytes repeat across independent directories |
| Transactions | Unit/process tests cover stale input/target, expired and reused receipts, idempotency conflict/replay, response loss, cancellation, shutdown, staged-source tampering, parent replacement and process kills |
| Containment | Unix symlink, hard-link, non-regular-file, traversal, alias and root replacement tests; Windows GNU cross-check compiles |
| Session/privacy | Fixed-cost generic wrong/malformed unlock tests, scope separation, read-only restrictions, throttling, bounded input parsing and non-reflecting errors |
| Headless client | Maintained Python client completes upload/register/convert, mapping initialization/edit/build, scope decisions, analysis, trace export/download, idempotent confirmation, read-only and shutdown |
| Browser | Installed Playwright 1.62.1 with installed Chrome: writable workflow and read-only registered-resource browsing; method/path contract checks, no non-loopback page requests, no page errors/cookies/storage, 640px reflow |
| Supply chain | `cargo deny --frozen check` passes using cached advisory data; this is not a fresh advisory assessment. `cargo vet --locked --frozen --no-minimize-exemptions` reports 26 uncovered new dependency versions; no audit or exemption is fabricated |

The process-kill test samples interruption times. It does not prove every
instruction boundary or power-loss durability. Cross-checking does not prove
Windows runtime behavior. Page-request interception does not prove that all
browser background or operating-system traffic is network-denied.

## Review remediations

The coordinating agent's complete-diff local review includes source, contracts,
fixtures, tests, clients, assets, dependency changes, workflow and documentation.
OCR delegation supplies local deterministic selection/rules; it is not an
independent Qwen review. Exact-commit selection and file accounting are retained
with candidate review artifacts during delivery.

Confirmed issues fixed during review:

- Initial hosted CI exposed a slow-machine retry-delay bug: delay started
  before Argon2 verification and could expire during the hash. The delay now
  starts after verification. Assertion failures no longer debug-print an
  unexpectedly successful capability. Windows also exposed native path
  normalization accepting backslashes; references now validate portable lexical
  spelling before any native path decomposition. Hosted checks must rerun on
  these fixes. The follow-up Windows test suite passed, then lint caught a
  wildcard import and stack construction of the 64 KiB rename buffer. The
  buffer now initializes directly in bounded, aligned heap storage; UTF-16
  collection is capped before allocation. The final Windows lint/build/client
  job must rerun on this remediation.
- Malformed unlock JSON/short values now use the same generic, fixed-cost
  failure path; array input no longer risks object-indexing panic.
- Overwritten base bytes and retained preview representations count toward
  session memory limits. Staged source identity and bytes are rechecked.
- Mapping scope controls filter initial subject choices; unmapped queue items
  preserve exact subject provenance. Ambiguous mapping manifests are invalid,
  never an empty apparently ready queue.
- Ambiguous applicability report destinations fail without overwriting either
  report. Both mapping and applicability output bytes are checked against CLI.
- Non-retryable terminal failures do not offer a misleading same-request retry.
  Source excerpts get focus and a return control. Overview counts open matching
  resource lists. The maintained client paces requests below the server limit.
- Guided forms support adding further relationships and linking built mapping
  collections into scope analysis without hand-editing JSON. If a guided
  inventory fails, the full decision document stays available for explicit repair.
  The confirmation dialog stays open through refresh and the old view is inert,
  preventing edits against an obsolete form version immediately after a save.
- Fault-injected browser recovery checks preserve unsaved edits after a failed
  confirmation, focus the modal error, clear it for a fresh preview, and keep a
  post-commit refresh failure visible after the dialog closes.
- Browser coverage verifies HTTP methods as well as path templates. Read-only
  browsing uses registered synthetic inputs.

## Pending gates

- Independent full-diff and exact-commit Qwen OCR review: destination-specific
  approval is pending following automatic approval review rejection. Local
  delegate selection/host review does not satisfy that external review gate.
- New dependency audit coverage: 26 cargo-vet versions remain uncovered.
  Fresh advisory checks and exact-head hosted CI/review results are required.
- Browser CI dependency installation/lockfile: approval pending after automatic
  review rejected downloads. Local tests use already-installed tools. The
  production UI remains embedded and requires no Node runtime or build step.
- Windows/macOS/Linux hosted execution, the complete supported browser matrix,
  and a full adversarial/security acceptance sweep.
- External uncooperating writer isolation across final recheck and rename;
  portable overwrite is not filesystem compare-and-swap. Single-file effects
  do not claim interruption-safe multi-file transactions.
- WCAG 2.2 AA and screen-reader/keyboard acceptance, security/product signoff,
  five-user studies, pilot metrics, package/release approval, and public Rust
  exhaustive-enum compatibility decision for `Commands::Workspace`.

Do not check off all PRD requirements or release this feature on the strength
of the tests above. No merge or release is authorized by this record.

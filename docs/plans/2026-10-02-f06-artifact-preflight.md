# F06 artifact-only corpus preflight

This slice implements structural preflight of explicitly supplied artifacts. It
creates no authentic corpus, labels, rights, reviewer independence, thresholds,
model execution, task-selection decision, quality score or acceptance evidence.
All fixtures are synthetic development cases. F06's full Must/Should and
acceptance scope, F16/F17/F18/F21 follow-on work, and the final full documentation
review remain open.

The full proposed `forge.suggest-eval-corpus/1` family remains future work: its
protocol, taxonomy, split, target-universe and authentic label qualification are
not consumed by this narrower closed inventory version.

## Interfaces and wire contracts

`forge::suggest::eval::preflight(root, manifest)` returns a report without file
publication. `preflight_to(root, manifest, output_dir)` uses the same capture
session and publishes only `preflight.json` through the existing confined atomic
no-replace authoring generation primitive. `root` selects the private corpus
directory; `manifest` and an optional `output_dir` are portable root-relative
paths. Directory publication currently supports Linux and macOS; other platforms
fail before staging. Read-only preflight uses the existing cross-platform
confined input reader. The coordinator owns CLI
wiring, stdout and exit codes. A returned report requires action (exit 1); a
trusted contract/capture/binding violation is invalid input (exit 2).

The closed `forge.suggest-eval-preflight-corpus/1` first-slice metadata contains
`schema_version`, `corpus_key`, explicit `version`, complete `cases`, and optional
`evidence`. Each case contains `case_key`, an existing `task` identity, honest
`origin`, `stratum`, `source_family`, `workflow_group`, `source_base`, optional
prepared `request`, explicit `sources`, and tagged `outcome`. Outcome is
`not-run`, `execution-blocked`, `tool-error`, or `response` with exact `run` and
`response` references, optional `bundle`, and `adjudication` missing/disputed.
Response outcomes require a request. Artifact references contain only portable
root-relative `path`, lowercase raw-file `sha256`, and exact `bytes`.

`evidence` may inventory rights, judgments, thresholds, execution_profile, freeze
and candidate references. These files are opaque: this slice checks their bytes
and confinement, but never interprets approval strings or claims the other
proposed evaluation schemas are qualified. Missing references remain missing;
present references are explicitly `opaque-unsupported`. Corpus metadata rejects
unknown/duplicate keys, nulls, unsupported versions and duplicate stable keys.

The distinct report contract is `forge.suggest-eval-preflight/1`, scoped to
`artifact-preflight`. It records the exact case denominator and every case state,
structural counts, declared run mode, content-free fingerprints and missing or
unsupported evidence gates. `acceptance_eligible`, `generation_enabled` and
`task_selection_ready` are always false. A structurally complete chain may set
`preflight_complete=true`; status remains incomplete pending authentic evidence.
An invalid candidate response sets status failed without dropping any case.
There is no passed evaluation state. Empty valid responses remain unadjudicated;
no-response and tool failure are not model abstention.

## Capture and validation order

1. Capture bounded metadata and strictly decode its closed types/cardinalities.
   Before case files are read, check all explicitly declared paths for conflicting
   pins/case aliases and check unique-artifact, aggregate-byte and per-group byte
   ceilings. Request payload and retained-response links are discovered and
   bounded after their parent documents are parsed.
2. Sort the complete case inventory by stable key; consume one case at a time.
3. Capture only declared sources and requests, verifying exact hashes/lengths
   before decoding. Payload links resolve from the request directory. SourceRef
   paths resolve from the explicit source_base, with key/path/hash/span binding.
4. Resolve response links from the run directory and verify success-record
   bindings to exact request, payload and raw response bytes.
5. Use the shared pure suggestion validator for ordinary target/citation/quote/
   redaction checks. Pinned malformed candidate content is response-invalid;
   malformed trusted request/run/bundle metadata or stale bindings is invalid.
6. Compare a supplied quarantine bundle against reconstruction from the captured
   bytes. A retained raw-response link resolves from the bundle directory and
   binds the copied bytes; no raw response is copied by this preflight.
7. Inventory opaque references and recompute every state count. Encode the report
   through a bounded writer. Revalidate all original content and file-identity
   pins immediately before returning a read-only report or publishing it.

The session retains hashes, lengths and file identities, rather than persistent
file descriptors. Each capture/revalidation uses the existing confined
held-handle reader and closes its file afterward. Pure response validation uses
the captured bytes. Final revalidation safely reopens each original path against
the same pins immediately before publication; it cannot guarantee subsequent
input stability.

Confinement reuses the existing held-file linkage reader: regular files only,
no symlinks, hard-link aliases, case aliases, escaping paths or directory scans.
Repeated references to the same exact path/pin are allowed; different paths
aliasing one file are refused. Portable links retain forward slashes on every
platform. Request/payload/source spans must fit actual captured UTF-8 boundaries.
Redaction can change source and payload lengths; no false equality is asserted
between transformed payload text and original source spans.

## Proposed implementation ceilings

These are hard engineering guardrails for this slice, not approved evaluation
sample sizes or an enforced operating-system RSS/wall-time profile:

| Input/resource | Ceiling |
|---|---|
| Corpus metadata | 2 MiB, depth 32, strings 16 KiB |
| Cases / sources per case / references per opaque gate | 256 / 128 / 64 |
| Unique captured artifacts / total unique captured bytes | 4,096 / 512 MiB |
| Unique captured bytes per case or opaque-reference group | 50 MiB |
| One opaque record / encoded aggregate report | 2 MiB / 2 MiB |
| One supplied source | Existing reuse ceiling, 1 MiB |
| Request / payload / response / run / bundle | Existing 2 MiB / 8 MiB / 8 MiB / 256 KiB / 50 MiB ceilings |

Reads use the smaller per-file, remaining-case and remaining-total bound before
allocation. The session retains only pins across cases; each case releases its
captured bytes. The bounded manifest and its decoded inventory remain available
for the run. Per-case accounting measures distinct logical artifact bytes;
repeated reads, decoded objects and validation copies can use additional memory. Fixed raw
bounds precede JSON decoding; they do not certify a process RSS ceiling. Report
encoding accounts for escaping and never truncates a successful receipt. Its
public encoder refuses upgraded qualification flags, an incorrect scope/version,
or case/count/completeness/status contradictions in caller-mutated reports.
This consistency guard does not authenticate a caller-constructed receipt.

Aggregate reports retain opaque operator-selected keys/version, fixed state and
gate names, declared origin/task/run mode, hashes and counts. They omit artifact
paths, source/response prose, reviewer names, raw attestations and diagnostics.
Operator keys/version are asserted metadata and must be chosen appropriately;
this slice does not authenticate identities or promise automatic anonymization.

## Remaining qualification

Authentic lawful inputs, complete gold target/content-unit universes, human
rubrics/judgments and conflict resolution, reviewed freeze/splits/exposure,
approved quality/safety thresholds and resource profile, candidate
model/template/runtime identity, qualified confined execution, full metrics and
regression, usefulness measurements and corpus-first task selection remain open.
A process-mode string is asserted provenance, not evidence an adapter ran or F16
confinement passed. Replay elapsed time is not generation latency. Citation
ratings/counts do not establish entailment, truth, harmfulness or usefulness.
Both versioned tasks remain in scope; complete mapping preparation/promotion and
local execution are not enabled here. Synthetic tests prove ingestion/refusal
behavior and cannot become held-out product labels or acceptance evidence.

The focused tests cover nested importer/source binding, fresh-hash invalid
candidate responses, exact accounting across failures, missing/opaque evidence,
forged process provenance, unresolved disputes, empty responses, closed metadata,
cardinality/byte/path/link boundaries, declarations before missing-file reads,
UTF-8 spans, mutated receipt invariants, same-byte replacement identity,
quarantine reconstruction/retained-copy binding, symbolic root aliases,
deterministic publication and no replacement of prior receipts. The coordinator runs Cargo/coverage and records
actual results; this implementation note supplies no fabricated test receipt.

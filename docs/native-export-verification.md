# Native JSON/YAML export verification

Measured on 2026-10-02 for Veans prerequisite #18 under F09 #17. This is a
Catalog/Component Definition export prerequisite to PRD-060 S-3; it does not
approve an evidence overlay or establish that feature's acceptance.
The [machine-readable record](native-export-verification.json) pins source,
fixtures, raw logs, complete line counts, docstring inventories and executables.

## Behavior and supported scope

`forge export` validates the complete decoded JSON/YAML input using the pinned
OSCAL 1.2.3 schema and semantic checks, then emits that same value for JSON/YAML
targets. Authored metadata, native identifiers, statements, capabilities,
back-matter resources, declared hashes, strings, array order and imported version
declarations remain intact. This is decoded-tree equality: formatting, object key
order and original input bytes are not the output contract. Input files remain
unchanged. Declared hashes are retained metadata, not proof of external content.

Only Catalog and Component Definition export are supported. Profile, SSP and
Control Mapping roots are explicitly refused, including JSON and YAML input.
Declarations 1.2.0–1.2.3 are checked against the single pinned 1.2.3 schema;
no historical schema is selected or fetched. The manifest contains eleven
runtime/test assets. No schema, dependency or public error mapping changed.

XML input or output and the public `OscalModel`/typed serialization helpers
retain Forge's partial generation-model projection. They may omit native fields
and do not inherit the JSON/YAML CLI preservation guarantee. Existing XML tests
that remove control implementations cannot establish full native equivalence.

YAML must decode as one duplicate-free JSON-compatible document with string
mapping keys and finite values. Ordinary aliases are expanded by the existing
decoder. Local custom tags exposed to the visitor are rejected; some URI-form
tags are erased by the decoder, so general YAML tag preservation or rejection
is unqualified. Comments, anchor syntax and YAML presentation are not preserved.

The regular-file reader caps raw input at 50 MiB. Postdecode checks cap tree
depth at 100 and each decoded string/key at the raw input length. These checks
and the decoder's recursion/repetition limits do not establish expanded-alias
allocation, RSS or hard wall-time confinement.

## Executed evidence

Two rich synthetic native fixtures independently validate for eight model/version
combinations: two models times four supported declarations. Their synthetic
contacts, resources and hashes are development data, with no authentic participant,
receipt or acceptance credit.

The original baseline attempt had 1 pass and 18 failures, but its negative helper
incorrectly expected exit 2 for all rejections. That attempt is retained and those
exit mismatches receive no product-defect credit. After correcting the established
exit contract (1 for export/parser errors, 3 for schema/semantic failures), the
frozen original 19-test baseline had **6 passes and 13 failures**. Those failures
overlap; they are not thirteen distinct defects. Baseline CLI source matched
`aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`; the new YAML helper was present but
unused by that CLI. Preserved-binary JSON outputs lost 120 first-absent Catalog
subtrees and 67 Component subtrees; descendants are not double-counted.

The initial candidate passed all 19 tests, all 20 existing export/version
integration tests and all 12 strict parser tests. A test successor then added
native depth-boundary checks and expanded unsupported-model refusals. The final
instrumented run passed **20 preservation tests + 20 existing integration tests
+ 1,828 library tests = 1,868**, with no failures or ignored tests.

Whole-value comparisons cover JSON→JSON and JSON→YAML→JSON for both models,
stdout and file output, and all four declarations. Each YAML and returned JSON
value is compared directly to the original tree, with no field pruning. Rejection
cases require the specific exit contract, zero stdout, diagnostics, unchanged
source and existing binary destination, and no newly created destination.
The depth test additionally requires the decoded-bound diagnostic; decoder
recursion failure cannot earn its pass.

The measured host was macOS 27.0.1 ARM64, Rust 1.98.1 (`48a229cea`), LLVM 22.1.8
and cargo-llvm-cov 0.8.7. This local run does not establish a hosted platform
matrix or external-tool interoperability. The preserved baseline, initial
candidate and final instrumented executables have separate byte identities.

## Docstrings and executed production coverage

Adjacent Rustdoc covers **51/51 selected declarations**: 13 production functions
(six new, two behavioral changes, five doc-only changes), nine new parser tests,
and 29 integration declarations (20 tests and nine helpers). The whole production
inventory is **24/39 documented**; 15 legacy parser functions remain undocumented.
Inherited trait docs, closures, types and generated items are excluded. This is
source inspection, not a claim that a rustdoc build passed.

| Production file | Executed lines | Added instrumented lines executed |
|---|---:|---:|
| `src/json_strict.rs` | 165/179 (92.18%) | 22/25 (88.00%) |
| `src/cli/export.rs` | 225/297 (75.76%) | 38/38 (100.00%) |

All 44 `cfg(test)` functions are excluded from these production denominators.
Every instrumented production line, including zero counts, is retained in the
JSON record. Added-line counts come from the exact main diff and may include
moved/Rustfmt lines; they are not statement or branch coverage. The three
uncovered added parser lines implement the owned-string key visitor path.
LLVM recorded no branch/MCDC instrumentation, so no branch percentage is claimed.
Earlier coverage and source snapshots remain unchanged. The first commit hook
stopped at formatting before Clippy/full tests; Rustfmt corrected only integration
test layout and optional trailing commas. A fresh instrumented run again passed
1,868 tests and binds the final formatted delivery bytes. Its production
denominators and uncovered sets are unchanged. The pre-format test, focused
record and failed hook log are retained; the active JSON is an explicit successor.
Strict Clippy subsequently required eliding the new visitor impl's unused lifetime
and removing unnecessary raw-string hashes in a test, with identical literal
content. Both findings were fixed without suppression; strict all-target Clippy
passes. A fresh 1,868-pass instrumented run binds the final Clippy-clean source
and test bytes, with unchanged production denominators and uncovered sets.
All earlier source, records and failed logs remain retained.

Reproduce the focused checks from the repository root:

```bash
cargo test --locked --lib json_strict::
cargo test --locked --test export_lossless_integration --test export_integration \
  --test export_format_pairs --test oscal_1_2_3_compatibility_test
cargo llvm-cov test --locked --lib --test export_lossless_integration \
  --test export_integration --test export_format_pairs \
  --test oscal_1_2_3_compatibility_test
```

The mandatory commit hook additionally requires formatting, strict default
all-target Clippy and the full Rust suite before a draft PR. Its actual result is
reported separately in the PR/commit receipt; the focused counts above do not
infer that result. No hook bypass or new crate is authorized by this record.

## Remaining gates

XML losslessness, other models, approved overlay representation/scope/collision
decisions, per-model overlay round trips, independent interoperability,
privacy/owner/pilot evidence, hosted CI, dependency/schema/release reconciliation
and human acceptance remain open. This prerequisite emits no overlay.

At the end of all scoped roadmap work, review and update the complete integrated
CLI/API docs, examples, architecture, schemas, dependencies/provenance, supported
platforms, verification instructions, requirements and acceptance status.
This focused review does not satisfy that final documentation gate.

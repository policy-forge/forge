# F09 prerequisite: preserve JSON/YAML export fields

## Scope and baseline

Veans Forge child #18 under F09 #17 and epic #2 tracks this executable
prerequisite to PRD-060 S-3. Branch `codex/f09-lossless-data-export` begins at
refreshed `origin/main` `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. It reuses the
clean owned F03 worktree, retaining its pushed dependency-review branch and docs.
The original primary checkout and unrelated configuration are preserved.

Current `forge export` casts JSON/YAML through partial generated-model envelopes,
then validates their reduced serialization. Valid native metadata, Component
statement identifiers/capabilities and back-matter resource/link hashes can be
silently removed. A reduced value passing validation does not establish lossless
preservation of the supplied artifact.

Preserve the original complete supported Catalog or Component Definition value
for JSON/YAML targets, and validate that value before publication. Exact-tree
preservation includes identifiers, timestamps, authored strings, array ordering,
roles/parties, declarations and existing references. Formatting, JSON object key
order, YAML presentation and original bytes are separate from semantic equality.
No new model, dependency, authoritative decision or overlay syntax is introduced.
Declared OSCAL compatibility remains 1.2.0–1.2.3 against the pinned 1.2.3 schemas.

## Parsing and limits

Reuse existing strict duplicate-key JSON parsing and its value visitor for YAML.
Reject malformed/duplicate keys, non-JSON-compatible scalar/mapping values,
multiple YAML documents and unknown schema fields before writing. The existing
YAML decoder rejects some local custom tags but can normalize or erase nonlocal
URI tags before the visitor; those presentation/tag semantics are unqualified. Preserve the
existing YAML error source when syntax decoding fails. The regular-file reader
retains its 50 MiB raw-byte cap; decoded depth/string checks remain explicit.
Post-decoding checks and decoder recursion limits do not alone establish a hard
expanded-alias allocation bound. Do not claim an unmeasured RSS/time profile.

## Executable evidence

Build genuinely schema-valid rich native fixtures and exercise the published
CLI: original to JSON and original to YAML to JSON must equal the whole original
value without field deletion. Verify source bytes remain unchanged. Invalid
input must preserve an existing destination and create no new destination.
Cover nested duplicate JSON/YAML keys, malformed syntax, multiple documents,
nonfinite/nonstring YAML, decoder-rejected local tags, unknown schema data and
unsupported versions. Qualify decoder-erased URI tags separately.
Keep raw baseline failures and final candidate successes with exact source,
fixture, executable and runtime hashes. Root runs all Cargo processes sequentially.

Before drafting, audit every changed/new named production and test helper’s
adjacent rustdoc, distinguish whole-file scope, and measure meaningful executed
production-line coverage with complete uncovered denominators. Run mandatory
formatting, strict default all-target Clippy and full Rust suite without bypass.
Independent review must check the preservation contract and fixture validity.

## Separate contracts and remaining gates

Existing typed/XML helpers remain a partial projection until separately proven;
this slice does not establish XML lossless equivalence. Existing integration
normalizations that delete control implementations cannot earn that proof.
Other OSCAL models and actual overlay emission require their own contracts.
PRD-060 S-3 supported-model/representation/collision/scope approval, per-model
round trips and independent interoperability, privacy/owner/pilot evidence and
linkage/evidence freshness acceptance remain open. Passing synthetic export
fixtures does not establish evidence sufficiency or approved overlay scope.

At the end of all scoped roadmap work, review and update the complete integrated
docs: CLI/API, examples, architecture, schemas, dependencies/provenance, supported
platforms, verification instructions, requirements and acceptance status. This
focused prerequisite does not satisfy the final documentation gate.

## Measured prerequisite result

The [verification record](../native-export-verification.md) retains the authentic
baseline and candidate sequence, executable/source pins, whole-tree checks,
51/51 selected adjacent Rustdoc scope and complete production-line denominators.
Corrected original baseline: 6/19 pass. Initial candidate: 19/19 pass; final
boundary/depth successor: 20/20 preservation and 1,868 instrumented tests pass.
This is synthetic prerequisite evidence, with all broader F09 and final integrated
documentation gates above still open. Mandatory commit-hook/hosted CI results
are reported separately; they are not inferred from the scoped record.

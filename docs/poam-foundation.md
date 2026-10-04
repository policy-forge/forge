# POA&M source foundation

The source foundation supplies an explicit `forge.poam/1` scaffold and an exact source integrity
check. A successful check does not approve remediation, authenticate an actor,
prove assessment review, or validate ownership, milestones, status history or
closure. Full F07 source-selection workflows and F08 remediation workflows remain
open, including the recorded D064 disposition and human acceptance gates.
The native OSCAL schema is the pristine NIST v1.2.3 release asset.

## Create an unselected scaffold

Keep a local Assessment Results artifact and its Assessment Plan, SSP, Profile
and Catalog companions in one bundle. Supply the exact result UUID and Forge
stable key; the command never chooses the first result or selects findings.
All source filenames are normalized descendants of `--root`. Document metadata
is explicit and never taken from the clock.

```sh
forge poam init --root ./assessment-bundle \
  --assessment-results assessment-results.json \
  --assessment-plan assessment-plan.json --ssp ssp.json \
  --profile profile.json --catalog catalog.json \
  --result-uuid UUID-FROM-SOURCE --result-key KEY-FROM-SOURCE \
  --document-key STABLE-PLAN-KEY --title "Remediation planning scaffold" \
  --document-version 0.1.0 --last-modified 2026-10-02T12:00:00Z \
  --output poam.json
```

The command records the exact five hashes, root identities and declared
versions, with empty items, roles and parties. It creates no remediation
owners, dates, status assertions or selected work. `--output` is a new `.json`
filename directly in the bundle root. Existing destinations, source files,
symlinks, hard links and special files are preserved. Atomic no-replace file
publication is implemented for Linux/macOS; unsupported platforms
fail without creating output. Omitting `--output` emits the same scaffold on
stdout and requires saving it in the same bundle root to preserve relative pins.

## Check exact source integrity

```sh
forge poam check --manifest ./assessment-bundle/poam.json \
  --source-only --format json
```

`--source-only` is required. JSON reports include
`validation_scope: source-integrity-only` and `workflow_validated: false`.
Every finding and risk in the explicitly identified result appears in the
inventory, including satisfied findings and closed risks. Absence from a plan
never means no action is required. Reports contain stable keys, exact UUIDs,
computed canonical object hashes, original states and source-derived control
references. They omit assessment prose and actor fields, and do not add resolved processing
paths. Authored stable keys and control IDs remain unchanged; callers must not
put sensitive content into those identifiers.

Exit 0 means valid foundation/source integrity; exit 2 means invalid,
unsupported, stale or unsafe input. There is no schedule/review exit 1 in this
foundation. Nonempty items/roles/parties remain unsupported in source-only mode.
The separate [authored workflow commands](poam-cli-workflow.md) validate
nonterminal work and generate native output with an explicit as-of date; they
retain their own schedule/action exits and pending full F08 acceptance.

## Source and bounds

The supported source is Forge Assessment Results JSON declared as OSCAL 1.2.3.
Companion compatibility remains 1.2.0–1.2.3 with exact declared-version pins and
validation against the pristine 1.2.3 schemas. Supported input compatibility is
separate from independent-tool interoperability or PRD 063 acceptance.
All five files are captured through held-handle confinement, with single-link
regular file identity and exact-byte revalidation. Context validation uses a
private snapshot of captured companion bytes. No network or child process runs.
The complete AR/AP/SSP/Profile/Catalog import chain and all context receipts
must agree. The supported current-producer profile requires all four unique
back-matter context receipts as well as all four metadata hash pins. Missing
receipts are explicitly unsupported. Source selection compares
kind/key/UUID/result/computed hash as one tuple. Declared content/rationale digests do not replace computed object hashes.

The authored manifest is capped at 4 MiB, nesting 64 and strings 64 KiB. Source files
are each capped at 50 MiB, total 100 MiB before snapshot construction; source trees
use depth 128, with at most 1,000 results, 10,000 conclusions, 1,000 references per
record, 64 properties per record and 100,000 aggregate reference edges. Reports
are capped at 10 MiB before serialization growth. Unknown native source extensions
fail with a supported-subset error instead of being silently discarded.

The integrated AR producer now emits context receipt hrefs from declared artifact
paths relative to its manifest bundle, rather than copying importer-relative
strings. POA&M source checks resolve each receipt from the actual AR directory
and require the declared captured companion. Old malformed receipts and wrong
targets remain refused even when a decoy has identical bytes. This correction
does not add AR output relocation or relax the supported source subset.

## Validate a native POA&M

`forge validate native-poam.json` detects the native
`plan-of-action-and-milestones` model and uses the pinned offline NIST schema.
An explicit `--schema-type poam` is also available. Forge additionally requires
`import-ssp` or `system-id`, because the pristine schema does not encode that
model choice. Generic native validation does not perform the foundation's exact
source binding or the planned remediation workflow checks.

## Remaining gates

D064 editable authority, supported native projections and terminal assertion
policy; PRD 063 source acceptance; full-date/native milestone projection; F08
workflow/build/report/baseline features; Windows publication qualification;
independent OSCAL consumer results and authentic remediation-team evaluation
remain open. No source check or synthetic fixture supplies those approvals.

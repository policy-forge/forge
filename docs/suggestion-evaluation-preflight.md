# Suggestion evaluation artifact preflight

`forge suggest eval preflight` checks a declared inventory of local suggestion
artifacts. It verifies exact bytes, importer-relative links, source spans and the
ordinary suggestion contracts, then accounts for every case. It does not run a
model, create judgments, calculate quality scores or select a generation task.
Authentic corpus and evaluation qualification remain required.

## Run a preflight

```sh
forge suggest eval preflight --root /path/to/private-corpus \
  --manifest corpus.json --format json

# Optional receipt in a new directory below the selected corpus root:
forge suggest eval preflight --root /path/to/private-corpus \
  --manifest corpus.json --output-dir receipt --format json
```

The manifest and optional output directory are portable paths relative to
`--root`. The root and output parent must already exist. Linux and macOS support
atomic directory publication; Windows currently refuses publication before
staging. Read-only input preflight uses the existing confined platform reader.
An existing receipt directory is never overwritten.

A returned report always exits **1**, because evaluation and acceptance require
further action. Structurally invalid candidate responses stay in the denominator
with report status `failed` and exit 1. Invalid trusted metadata, unsafe paths,
stale hashes, incorrect bindings or failed publication exit **2**. JSON stdout
contains only the report; diagnostics use stderr. Text output states that the
evaluation gates remain open.

## Start with an explicit unfinished case

Save this synthetic development inventory as `corpus.json` in an existing private
directory. This example reads only its manifest and earns no acceptance credit:

```json
{
  "schema_version": "forge.suggest-eval-preflight-corpus/1",
  "corpus_key": "development-inventory",
  "version": "1",
  "cases": [
    {
      "case_key": "case-one",
      "task": {
        "kind": "policy-drafting",
        "schema_version": "forge.suggest-task-drafting/1"
      },
      "origin": "synthetic-development",
      "stratum": "development",
      "source_family": "example-family",
      "workflow_group": "example-workflow",
      "source_base": "inputs",
      "sources": [],
      "outcome": {"state": "not-run"}
    }
  ]
}
```

The resulting report declares one expected case and one `not-run` case. Empty
inventories are invalid. Add complete existing prepared request/run/response
artifacts explicitly; do not omit failed, blocked or unfinished cases. Mapping
uses the existing `mapping-candidates` / `forge.suggest-task-mapping/1` task.

## Artifact contracts

Every reference supplies a root-relative `path`, lowercase SHA-256 of the raw
file and exact byte count. Case source references also name the SourceRef key.
`source_base` resolves request source paths. Request payload links resolve from
the request directory; run response links resolve from the run directory. A
supplied quarantine bundle is compared with reconstruction from captured bytes;
its optional retained-response link resolves from the bundle directory.

The closed manifest rejects unknown or duplicate keys, nulls, unsupported
versions, duplicate case/source keys and conflicting file references. Inputs
must be confined regular files. Symbolic links, hard-link aliases, case aliases
and escaping paths are refused. Reads and report serialization have fixed bounds;
see the [implementation ceilings](plans/2026-10-02-f06-artifact-preflight.md#proposed-implementation-ceilings).
Final verification safely reopens inputs to check their content and file identity
immediately before returning or publishing. Input stability after that point is
not guaranteed.

The narrow corpus version is distinct from the future full evaluation corpus
contract. Supporting `rights`, `judgments`, `thresholds`, `execution_profile`,
`freeze` and `candidate` references are opaque inventories. Their presence never
qualifies rights, approval, independence, execution or evaluation success.

## Interpret the receipt

`forge.suggest-eval-preflight/1` records every case, state and exact artifact pin.
`preflight_complete` means every declared case has a structurally valid candidate
chain. It does not establish quality or acceptance. `acceptance_eligible`,
`generation_enabled` and `task_selection_ready` remain false. Valid empty
responses remain unadjudicated; no-response and tool failures are not abstentions.
Mechanical citation/count ratings do not establish semantic entailment.

Reports omit artifact paths and source/response prose, but retain opaque keys,
versions, hashes and declared task/origin/run mode. Choose appropriate opaque
keys; operator assertions do not authenticate identities or anonymize metadata.
Keep private corpus inputs under their existing access controls.

Authentic lawful inputs, gold target/content-unit universes, independent human
judgments and dispute resolution, reviewed splits/freeze/exposure, approved
thresholds and execution profile, model/template/runtime identity, confined local
execution, full metrics/regressions, usefulness measurements, task selection and
the final full roadmap documentation review remain open.

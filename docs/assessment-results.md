# OSCAL Assessment Results

`forge assessment results` packages explicit assessor-authored conclusions into
OSCAL 1.2.3 Assessment Results JSON. FORGE validates structure, exact local
artifact identities, scope, references, and graph integrity. It does not run
assessment procedures, inspect evidence content, infer effectiveness, certify
compliance, authenticate assessor identity, or create remediation ownership.

## Scaffold a manifest

Keep the Assessment Plan and its exact SSP, Profile, and Catalog companions in
the directory where the manifest will be written. Their OSCAL import `href`
values must match the declared companion `href` values exactly.

```bash
forge assessment results init \
  --assessment-plan assessment-plan.json \
  --ssp ssp.json \
  --profile profile.json \
  --catalog catalog.json \
  --evidence-index linkage-index.json \
  --output assessment-results-manifest.json
```

Initialization validates the complete companion chain against the pinned
schemas, records SHA-256/root UUID/document-version/OSCAL-version identity, and
copies the Assessment Plan's reviewed control and objective scope. It creates
no observation, finding, risk, verdict, owner, or date. Replace every
`REPLACE...` value and add reviewed conclusions before building.

The closed `forge.assessment-results/1` manifest contains:

- one stable document key and one stable result-epoch key;
- exact local identities for an Assessment Plan, SSP, Profile, and Catalog;
- optional identity-only `forge.linkage-index/1` evidence input;
- declared roles and parties;
- a reviewed control/objective scope;
- observations with subjects, tasks, evidence keys, and human provenance;
- findings with explicit OSCAL statement/objective target status;
- risks with assessor-declared status, severity, and confidence; and
- explicit `observation -> finding -> risk` relationships.

Each observation, finding, and risk requires an assessor key, role ID, RFC 3339
assessment start/end, OSCAL method, and non-empty rationale. Stable keys are
immutable identities: FORGE derives UUID v5 values from them, never from prose,
array order, or wall-clock time.

## Build and review

```bash
forge assessment results build \
  --manifest assessment-results-manifest.json \
  --output assessment-results.json \
  --report assessment-results-review.json \
  --report-format json

forge assessment results build \
  --manifest assessment-results-manifest.json \
  --baseline prior-assessment-results.json \
  --output assessment-results.json \
  --report assessment-results-review.html \
  --report-format html \
  --fail-on any
```

The completed typed artifact is validated against the pristine vendored OSCAL
1.2.3 Assessment Results schema before any destination is changed. Baseline
comparison uses stable identities and reports object additions/removals,
content/rationale/status changes, stale references, and changed upstream
fingerprints. Text, JSON, and static HTML reports share the same deterministic,
content-minimizing model.

Exit `0` means a trustworthy artifact was built and the selected baseline gate
did not fire. Exit `1` means a valid baseline comparison has review actions.
Exit `2` means no trustworthy build could be produced. `--fail-on never` keeps
valid baseline review actions at exit `0`; it does not remove them from the
report.

All processing is offline. Inputs must be confined regular JSON files beneath
the manifest directory; symlink traversal, stale hashes, output/input aliases,
unknown or duplicate keys, unsupported versions, oversized resources, and
invalid references fail before writes. Output includes evidence keys and hashes
only—never evidence content or absolute local paths. PRD 064 POA&M export and
multi-epoch workflows are deliberately outside this command.

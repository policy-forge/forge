# Evidence and Implementation Linking

`forge linkage` creates a local, deterministic `forge.linkage-index/1` that associates exact OSCAL
requirement subjects with exact OSCAL implementation subjects and reviewer-declared evidence
metadata. It records identities, hashes, sizes, dates, and labels. It does not copy evidence bytes,
retrieve URIs, run tests, or derive control-outcome or evidence-quality judgments.

## Supported subjects

Requirement resources are local OSCAL Catalogs or Profiles. Profiles require a caller-supplied,
reviewed resolved Catalog companion. Eligible requirement subjects are control IDs and statement
part IDs.

The single implementation resource is either a schema-valid OSCAL Component Definition or System
Security Plan. Eligible implementation subjects are `implemented-requirement.uuid` and nested
`statement.uuid` values. All artifact bytes, root UUIDs, document versions, OSCAL versions, and
canonical subject fingerprints are retained as provenance.

## Commands

```bash
# Create a hash-pinned scaffold. Profile requirements also need --resolved-catalog.
forge linkage init \
  --requirement catalog.json \
  --implementation component-definition.json \
  --output linkage.json

# Build the deterministic index and a separate JSON maintenance report.
forge linkage build \
  --manifest linkage.json \
  --as-of 2026-08-27 \
  --output linkage-index.json \
  --report linkage-report.json \
  --format json

# Recheck current artifacts/evidence and compare a prior index.
forge linkage check \
  --manifest linkage.json \
  --as-of 2026-09-30 \
  --baseline linkage-index.json \
  --format text

# Aggregate only explicitly named projects into an owner queue.
forge linkage queue \
  --manifest project-a/linkage.json \
  --manifest project-b/linkage.json \
  --as-of 2026-09-30 \
  --format json
```

Reports support `text`, `json`, and static `html`. HTML is a metadata-only trace from requirement
IDs through implementation UUIDs to evidence metadata and fingerprints.

## Manifest contract

The manifest is closed and duplicate-key-safe. Paths are relative descendants of the manifest or a
declared evidence root. The example below omits a second requirement statement only for brevity.

```json
{
  "schema_version": "forge.linkage/1",
  "project": {
    "key": "payments-production",
    "title": "Payments production linkage",
    "expiring_window_days": 30,
    "max_evidence_bytes": 10485760,
    "approved_uri_schemes": []
  },
  "reviewers": [
    { "key": "reviewer-1", "name": "Assigned reviewer" }
  ],
  "requirement_resources": [
    {
      "key": "policy",
      "type": "catalog",
      "artifact": "catalog.json",
      "href": "catalog.json",
      "expected_sha256": "REPLACE_WITH_64_LOWERCASE_HEX_CHARACTERS"
    }
  ],
  "implementation_resource": {
    "key": "component",
    "type": "component-definition",
    "artifact": "component.json",
    "href": "component.json",
    "expected_sha256": "REPLACE_WITH_64_LOWERCASE_HEX_CHARACTERS"
  },
  "evidence_roots": [
    { "key": "exports", "path": "evidence" }
  ],
  "evidence": [
    {
      "key": "access-review-2026q3",
      "title": "Quarterly access review export",
      "evidence_type": "access-review-export",
      "owner": "identity-team",
      "collected_at": "2026-08-20T15:00:00Z",
      "valid_through": "2026-11-20",
      "sensitivity_label": "restricted",
      "source_label": "reviewed local export",
      "location": {
        "kind": "local",
        "root_key": "exports",
        "path": "access-review.bin",
        "expected_sha256": "REPLACE_WITH_64_LOWERCASE_HEX_CHARACTERS",
        "expected_size": 12345
      }
    }
  ],
  "links": [
    {
      "key": "access-control-link",
      "requirements": [
        { "resource_key": "policy", "type": "control", "id_ref": "ac-2" }
      ],
      "implementations": [
        {
          "type": "implemented-requirement",
          "id_ref": "00000000-0000-4000-8000-000000000000"
        }
      ],
      "evidence_keys": ["access-review-2026q3"],
      "evidence_required": true,
      "responsible_role": "control-owner",
      "implementation_status": "implemented",
      "review": {
        "reviewer_key": "reviewer-1",
        "reviewed_at": "2026-08-21T18:00:00Z",
        "rationale": "Reviewer associated these exact versioned subjects and evidence metadata."
      },
      "impact_finding_ids": [],
      "policy_version_keys": ["payments-policy-v3"]
    }
  ]
}
```

URI evidence uses a nested location such as:

```json
{
  "location": {
    "kind": "uri",
    "uri": "https://records.example/evidence/123?token=not-retained#section",
    "unverified": true
  }
}
```

Only `https` and explicitly listed organization-approved custom schemes are accepted. FORGE never
resolves or fetches the URI. User-info, query, and fragment data are removed from the index and all
default reports.

## Freshness and exit behavior

Every freshness decision uses the required `--as-of` date; the wall clock is never consulted.
`valid-through` equal to `--as-of` is classified as `expired`. A later date inside the inclusive
configured window is `expiring`. A matching local hash/size outside that window is `current`.
Changed bytes, missing paths, and non-fetched URIs are reported independently of date findings.
The unavoidable `unverified-uri` finding is informational under the default `required` gate; use
`--fail-on any` when automation should stop on any URI reference that cannot be verified locally.

`--fail-on` accepts `required` (default), `changed`, `expired`, `any`, or `never`:

- Exit `0`: valid analysis and no finding selected by the gate.
- Exit `1`: valid index/report with at least one selected maintenance finding.
- Exit `2`: invalid manifest, artifact, subject, path, URI, schema, bound, or alias. A build does not
  write its index after an invalid analysis.

## Safety and privacy

- Local evidence must be a bounded regular file below a declared root. Descriptor-bound traversal
  prevents validation-to-open path replacement. Traversal, symbolic links, any hard-link aliases,
  devices, FIFOs, sockets, and output/input aliases are rejected.
- Indexes contain labels, IDs, hashes, sizes, dates, and reviewer metadata—not evidence content,
  excerpts, credentials, query strings, URI fragments, or canonical absolute paths.
- Treat hashes, labels, reviewer identities, and source metadata as potentially sensitive. Apply the
  organization's approved access, retention, and sharing policy to linkage outputs even though the
  underlying evidence bytes are absent.
- SHA-256 detects a byte difference from an approved value. It does not prove origin, custody, or
  evidentiary quality.
- An association or an `implemented` reviewer assertion does not establish a control outcome.
- Implementation subjects outside every reviewed link are surfaced as maintenance work; FORGE does
  not create a relationship for them.
- Baseline findings distinguish subject removals/content changes, evidence additions/removals/byte
  changes/reference changes/expiry edits, link additions/removals, and relationship membership
  edits.

OSCAL back-matter overlay generation remains deferred until each supported model has demonstrated
schema-valid, lossless round-trip behavior.

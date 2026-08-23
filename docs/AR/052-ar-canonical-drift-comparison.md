# 052-ar-canonical-drift-comparison

> **Document Type:** Architecture Decision Record
> **Status:** Proposed
> **Last Updated:** 2026-08-23
> **Owner:** Brian Luby

## Context

PRD 052 requires a generated-artifact drift gate with zero false positives from
FORGE's volatile metadata. Catalog and Component Definition generation currently
assigns a UUID v4 at the artifact root and the current UTC time to
`metadata.last-modified`. Raw byte or parsed-JSON equality therefore reports
drift even when policy-derived content is unchanged.

The existing `forge diff` command is intentionally human-oriented. It extracts
selected control fields and prints field-level content, so it is neither a
complete artifact comparison nor safe as the default source for CI summaries.

## Decision

Introduce a separate, versioned canonical comparison contract and expose it as:

```text
forge drift <committed.json> <generated.json> [--format text|json]
```

Contract v1 compares the complete parsed JSON value for Catalog and Component
Definition artifacts after removing exactly:

- the artifact root `uuid`; and
- the artifact root `metadata.last-modified`.

No other UUID, timestamp, metadata, array, or policy-derived field is ignored.
JSON object key order and whitespace are naturally insignificant after parsing;
array order remains significant. Both inputs must have the same supported OSCAL
model. The command returns content-free status only:

- exit `0`: clean;
- exit `1`: substantive drift;
- exit `2`: unreadable, invalid, unsupported, or mismatched artifacts.

The machine result contains `status`, `artifact_type`, and
`comparison_contract`. It contains no file paths, control identifiers, titles,
statements, UUID values, or JSON excerpts.

## Rationale

This approach gives the Action a complete fail-closed comparison without
duplicating OSCAL semantics in JavaScript or leaking policy content. A dedicated
command keeps the detailed `forge diff` UX unchanged. Versioning makes any future
change to ignored fields explicit and testable.

Deterministic generation was considered, but a stable artifact UUID needs a
separate identity policy: deriving it from content changes identity on every
edit, while deriving it from paths makes relocation meaningful. Canonical
comparison resolves the known CI false-positive problem without prematurely
settling that identity policy.

## Consequences

- A manually changed root UUID or `metadata.last-modified` does not count as
  drift under contract v1 because those fields are already nondeterministic in
  normal FORGE output.
- Every nested UUID and all nonvolatile metadata remain significant.
- Validation remains a separate required phase before comparison; this command
  detects supported model shape but does not replace `forge validate`.
- Missing and extra files remain the Action orchestrator's responsibility.
- Expanding the exclusion list requires a contract-version increment, negative
  fixtures, and security review for false-clean risk.

## Verification

- Unit coverage for Catalog and Component Definition comparisons.
- A 100-iteration unchanged-output test with varying root UUID/timestamp.
- Negative tests for policy text, nested UUID, and metadata version changes.
- CLI tests for clean/drift exit codes, JSON output, type mismatch, and content
  non-disclosure.

# API Compatibility and Deprecation Policy

> **Document Type:** Compatibility Policy
> **Audience:** Engineering, LLM agents, local client developers, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

---

## Scope

`/api/v1` is a supported local product interface for the bundled web UI and
documented same-host clients; it is not an undocumented implementation
detail. This policy defines what may change within `v1`, what requires
`/api/v2`, how long superseded majors are retained, and how CI treats the
contract. It implements the PRD 062 "Compatibility and Support Policy"
section of [../PRD/062-prd-local-web-workspace.md](../PRD/062-prd-local-web-workspace.md).
The normative contract is
[forge-workspace-v1.openapi.yaml](forge-workspace-v1.openapi.yaml) with the
capability matrix at [capability-matrix.md](capability-matrix.md) /
[capability-matrix.json](capability-matrix.json), fixtures under
[fixtures/](fixtures/), and the `forge.workspace/1` index schema at
[schemas/forge.workspace-1.schema.json](../../schemas/forge.workspace-1.schema.json).

The support promise never authorizes a remote bind. Every PRD 062 API client
uses the same loopback, capability, project-root, and offline-runtime
controls regardless of contract version.

## Changes Allowed Within v1 (Additive Only)

1. **New operations.** New paths, methods, and operationIds may be added.
   They must receive a capability-matrix entry before implementation.
2. **New optional request fields.** A new request field must be optional
   with a documented default; existing requests must keep behaving
   identically.
3. **New optional response fields.** A new response field must be optional.
   Clients MUST tolerate unknown fields on read: closed schemas
   (`additionalProperties: false`) are an authoring-time guarantee that the
   server emits only documented fields in a given contract version, not a
   client-side parsing rule.
4. **New error codes, and new enum values only where documented as open.**
   The error `code` list in the OpenAPI `Error` schema is open to additive
   extension within v1. Every other enum in the contract (operation states,
   reason codes, classifications, decision states, roles, capability
   scopes, media kinds, redaction profiles) is closed: adding a value
   requires `/api/v2` unless the enum's description explicitly documents it
   as open to addition.
5. **New optional query parameters and headers**, and **new response codes
   that are strictly more precise** than an existing one for the same
   condition (for example splitting an existing 400 into a more specific
   typed 422), provided the documented meaning of existing codes does not
   change.
6. **Tightening validation** only when no valid v1 request becomes
   invalid (for example reducing an internal bound that the contract
   never promised).

Anything not listed here is presumed breaking.

## Changes That Require /api/v2

- Removing or renaming any operation, path, parameter, header, or field.
- Making an optional field required, or narrowing an accepted value set
  (including tightening any documented bound that valid clients rely on).
- Changing established semantics of an operation, including its effect,
  ordering, pagination, idempotency, versioning, receipt, or conflict
  behavior.
- Reinterpreting an existing error code or HTTP status meaning, including
  changing which condition produces it or what `retryable` implies.
- Adding a value to any enum not documented as open, or removing any enum
  value.
- Weakening the security model (authentication, scoping, exact-Host
  enforcement, containment rules) in any way observable to clients.

A successor major lives under `/api/v2` with its own OpenAPI document and
fixtures. Both majors may be served by the same process during the support
window; v1 and v2 capabilities are never interchangeable.

## Support Window

- When a successor major is introduced, the previous API major is retained
  for **at least one stable FORGE minor release** and published migration
  guidance precedes any removal.
- Deprecations are announced in release notes with the replacing operation,
  the release that removes the deprecated surface, and the migration step
  for each affected workflow.
- On-disk schemas (such as `forge.workspace/1`) and API models version
  independently; an API major never silently renumbers a domain schema
  version.

## Bootstrap Version Publication and Fail-Closed Clients

- The server publishes its API major and exact `contract_version` during
  bootstrap: in the `--machine-session` stdout descriptor, in the `Session`
  resource (`api_major`, `contract_version`), and with the static bootstrap
  assets, which declare their supported API major.
- A client that observes an unsupported API major **fails closed** with a
  safe version error and performs no project operation (PRD 062 AC-25).
  Supported v1 clients continue to work across additive v1 changes by
  tolerating unknown fields and codes as described above.
- The capability itself never encodes version authority; version checks
  happen at bootstrap and session read, not per request.

## Contract Artifacts as Release Artifacts

The OpenAPI document, the `forge.workspace/1` schema, the capability matrix,
and the fixture set under [fixtures/](fixtures/) are release artifacts: they
are committed with source and included in packaged release artifacts, and
release notes identify contract additions, deprecations, and breaking
versions. The `info.version` field of the OpenAPI document is the
`contract_version` published by the server.

## CI Drift Detection

Per ADR [../adr/0001-openapi-contract-ownership.md](../adr/0001-openapi-contract-ownership.md)
(contract-authored ownership; one normative artifact — two independently
maintained schemas are prohibited):

- The committed OpenAPI document is **normative**. HTTP handlers, fixtures,
  generated or type-checked clients, and the bundled UI must conform to it;
  any drift fails CI (PRD 062 M-21, AC-19).
- Fixtures are validated mechanically against the schemas named in
  `fixtures/index.json` (see [fixtures/README.md](fixtures/README.md)),
  including strict JSON parsing with duplicate-key rejection.
- The capability matrix is checked bidirectionally: every operationId in the
  contract appears in at least one matrix entry, and every operationId in
  the matrix exists verbatim in the contract.
- Compatibility checks fail when a contract change is non-additive within
  v1, when a fixture expectation flips without a contract rationale, or
  when the published `contract_version` does not match the document.

## Unreleased contract 1.1.0

Adds optional `provenance_ref` to `ControlInventoryItem` and `MappingSubject`.
The opaque reference lets clients traverse provenance without guessing identity
from display labels, including controls whose IDs exceed the anchor length bound.
Existing 1.0.0 request documents, fixtures, and required response fields retain
their meaning. No on-disk `/1` schema changes and no release are implied.

The same unreleased revision adds `initializeApplicabilityDraft` and
`initializeMappingDraft` for M-9/M-10 new-project onboarding. Both prepare a
single-file preview; selected inputs are explicit and hash-pinned, scope starts
with no decisions, initial mappings require at least one explicit reviewed relationship under the
unchanged PRD-055 `/1` contract, and reviewer metadata must be supplied. Existing operations retain their semantics.

`WorkspaceReportEnvelope` documents the separately versioned, canonical
`forge.workspace-report/1` static export. The trace-report role now has an
explicit accepted format; generic HTML and CLI text are rejected. This resolves
an underspecified prerelease role, rather than migrating any released input.
Historical applicability reports continue using the PRD-056 contract.
The pre-initialization `listMappingSubjects` path requires both existing optional
selectors (`resource_id`, `side`); with a selected manifest, the existing inventory
and filter semantics remain unchanged. Existing closed response clients should
negotiate the returned contract version when adopting optional provenance fields.

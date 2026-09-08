# API Contract Fixtures

> **Document Type:** Contract Fixture Set
> **Audience:** LLM agents, human reviewers, CI maintainers
> **Status:** Draft
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

---

## Purpose

This directory holds representative request, response, workspace, error,
pagination, version-conflict, and operation fixtures for the normative
`/api/v1` contract at [../forge-workspace-v1.openapi.yaml](../forge-workspace-v1.openapi.yaml)
and the `forge.workspace/1` project index schema at
[../../../schemas/forge.workspace-1.schema.json](../../../schemas/forge.workspace-1.schema.json).
They are release artifacts per PRD 062 ("Contract-First Artifacts") and are
the Slice 0 executable evidence that the contract is internally consistent
before any handler exists. See
[docs/PRD/062-prd-local-web-workspace.md](../../../docs/PRD/062-prd-local-web-workspace.md).

## Format

`index.json` is the machine-readable manifest (`forge.api-fixtures/1`). Each
entry declares:

| Field | Meaning |
|-------|---------|
| `file` | Fixture path relative to this directory |
| `kind` | One of `request`, `response`, `workspace`, `error`, `pagination`, `version-conflict`, `operation` |
| `schema` | `openapi:components/schemas/<Name>` resolved live from the OpenAPI document, or `workspace:forge.workspace/1` resolved from the vendored schema |
| `expectation` | `valid` (the fixture must validate) or `invalid` (the fixture must fail validation) |
| `description` | What the fixture proves; for error fixtures, the stable code exercised |

## Validation rules enforced by CI

1. **Strict parse.** Every fixture is parsed as complete JSON with duplicate
   object keys rejected, matching the repository's strict JSON conventions in
   `src/json_strict.rs`. There are intentionally no duplicate-key fixture
   files in this set; the strict-parse rule is asserted for all fixtures by
   the validator itself rather than by example files.
2. **Schema resolution.** Every `openapi:components/schemas/<Name>` reference
   must resolve against the committed OpenAPI document, and every
   `workspace:forge.workspace/1` reference must resolve to the vendored
   schema. Unknown references fail.
3. **Expectation assertion.** `valid` fixtures must validate against their
   declared schema; `invalid` fixtures must fail validation for the single
   documented reason (each invalid fixture is otherwise well-formed JSON).
4. **Coverage.** The set must keep covering all fixture kinds, both
   directions of the workspace schema, and the representative response,
   error-envelope, pagination, version-conflict, and operation shapes.

## Conventions

- All identifiers, hashes, versions, cursors, and capability-like strings in
  these fixtures are synthetic. No real project content, reviewer data, or
  secret material appears here.
- Hash-like values are 64-character lowercase hexadecimal strings; opaque
  identifiers follow the `res_`, `op_`, `prev_`, `qi_`, `prov_`, `ex_`, and
  `sess_` prefixes used by the contract.
- Timestamps are RFC 3339 with a `Z` offset.
- Error fixtures are *valid* instances of the closed error envelope; the
  stable `code` each exercises is named in its `description`.

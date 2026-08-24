# Data Model: OSCAL v1.2.3 Compatibility

## Schema Manifest

The top-level record contains `repository`, `tag`, `release_commit`, `published_at`, `schema_version`, and `assets`.

Each asset contains `name`, `url`, `local_path`, `size`, `sha256`, `format` (`json-schema` or `xsd`), `model`, and `role` (`runtime` or `test`). Paths are repository-relative and must remain inside the allowlisted `schemas/` or `tests/fixtures/` roots.

## Supported Version

An internal value object stores numeric `major`, `minor`, and `patch`. Parsing accepts only canonical `N.N.N` ASCII-decimal input. Policy accepts the inclusive enumerated set `1.2.0..=1.2.3`; parsed values never select a schema path.

## Validation Report

Existing invariant `is_valid == errors.is_empty()` remains. `supported_input` is true only after a present string declaration parses and is in policy. Model, declared version, and schema version are mandatory for completed validation reports.

## Round-Trip Result

Existing divergence invariants remain. Compatibility classification describes external conversion evidence and is not a schema-conformance result.

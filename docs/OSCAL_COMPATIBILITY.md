# OSCAL Compatibility and Schema Upgrades

FORGE generates OSCAL v1.2.3 output and validates supported input against one
pinned, offline v1.2.3 schema baseline. This document is the maintainer runbook
for verifying the current baseline and evaluating a future patch release.

## Compatibility contract

- Generated Catalog, Component Definition, Profile, Assessment Plan, and SSP
  documents declare `metadata.oscal-version: 1.2.3`.
- Catalog, Component Definition, and Profile inputs declaring v1.2.0 through
  v1.2.3 are supported when compatible with the pinned v1.2.3 schema.
- The document root or existing explicit model override selects the schema
  family. The declared OSCAL version never selects a historical schema and
  never triggers a network request.
- Export preserves the imported `metadata.oscal-version` and the user-owned
  `metadata.version` across JSON, XML, and YAML.
- Runtime validation, export, and non-oscal-cli compatibility tests are fully
  offline. JSON Schema and XSD references must resolve from vendored files.

Validation output reports `declared_oscal_version` separately from
`schema_version_used`. A v1.2.0 declaration can therefore pass the v1.2.3
compatibility schema without being represented as having declared v1.2.3.

## Current asset pin

The authoritative inventory is
[`schemas/oscal-schema-manifest.json`](../schemas/oscal-schema-manifest.json).
It pins `usnistgov/OSCAL` tag `v1.2.3`, release commit `e061961`, publication
date `2026-08-07`, exact release URLs, byte sizes, and SHA-256 digests for all
nine runtime and compatibility-test assets. Vendored schema bytes must not be
edited locally.

Verify the checked-in baseline from a clean checkout:

```bash
cargo test --test schema_provenance_test
cargo test --test oscal_1_2_3_compatibility_test
cargo test --test assessment_plan_test
cargo test --test ssp_template_test
```

The provenance test fails closed for an unlisted asset, path escape, remote
schema reference, size mismatch, digest mismatch, or inconsistent release
identity.

## oscal-cli compatibility

Embedded v1.2.3 schemas are the structural-validation authority. oscal-cli is
an optional external conversion oracle. Its successful JSON → XML → YAML →
JSON chain proves only that the tested artifact survived that tool path.

oscal-cli v1.0.3 documents OSCAL v1.1.2 model support. Results from that
version are classified `advisory-older-model-baseline`, record tool version
`1.0.3` and model baseline `1.1.2`, and must not be described as authoritative
v1.2.3 validation. Unrecognized oscal-cli versions are also advisory until
their model baseline is documented and the compatibility matrix has been run;
the presence of a newer-looking version number is not sufficient evidence.

## SSP template sentinel policy

SSP skeletons must distinguish completion guidance from asserted compliance
facts. Optional leveraged authorizations are omitted until the caller supplies
a real authorization, because `party-uuid` and `date-authorized` are required
inside each entry and fabricated values would look authoritative. The required
`import-profile.href` uses the visible `TODO-profile.json` placeholder only
when no source profile is supplied. Legacy builder guidance that has no native
OSCAL v1.2.3 field is preserved in `remarks`, `links`, or namespaced `props`
rather than silently discarded.

Human-readable validation output now adds model, declared-version, and
schema-baseline context. Consumers that parse validation stdout should migrate
to the stable JSON report format instead of depending on line layout.

## Future patch-upgrade procedure

Use this workflow only for an allowlisted official `usnistgov/OSCAL` release.
Normal FORGE execution never performs these network operations.

1. Start from a clean branch and create a temporary staging directory outside
   the repository. Do not overwrite checked-in schemas yet.

   ```bash
   git status --short
   schema_stage="$(mktemp -d)"
   gh release view vNEXT --repo usnistgov/OSCAL --json tagName,targetCommitish,publishedAt,url
   gh release download vNEXT --repo usnistgov/OSCAL --dir "$schema_stage" \
     --pattern 'oscal_catalog_schema.json' \
     --pattern 'oscal_component_schema.json' \
     --pattern 'oscal_profile_schema.json' \
     --pattern 'oscal_assessment-plan_schema.json' \
     --pattern 'oscal_ssp_schema.json' \
     --pattern 'oscal_catalog_schema.xsd' \
     --pattern 'oscal_component_schema.xsd' \
     --pattern 'oscal_profile_schema.xsd' \
     --pattern 'oscal_complete_schema.xsd'
   ```

2. Confirm that the release tag, commit, publication date, asset names, sizes,
   and download URLs match the official GitHub release metadata. Reject extra,
   missing, renamed, redirected, or archive-only substitutes.

3. Compute staged digests before copying anything. Compare every size and
   SHA-256 to the release metadata and record them in a proposed manifest.

   ```bash
   for asset in "$schema_stage"/*; do
     shasum -a 256 "$asset"
     wc -c "$asset"
   done
   ```

4. Review schema changes before replacement. Produce a concise per-model diff
   of changed definitions, required fields, constraints, enums, `$id` values,
   and XML namespaces. Treat unexpected remote references or a release-identity
   mismatch as a hard stop.

   ```bash
   git diff --no-index schemas/oscal_catalog_schema.json "$schema_stage/oscal_catalog_schema.json"
   git diff --no-index tests/fixtures/xsd/oscal_catalog_schema.xsd "$schema_stage/oscal_catalog_schema.xsd"
   ```

   Repeat for every asset and save the reviewed summary with the upgrade PR.

5. Only after verification, replace each allowlisted local path and update
   `schemas/oscal-schema-manifest.json` in the same change. Do not edit schema
   contents, `$id` values, comments, whitespace, namespaces, or constraints.

6. Update the shared `OSCAL_VERSION`, supported-version boundary tests,
   generated-current fixtures, oscal-cli compatibility mapping, and this
   document. Never bulk-replace the immutable `tests/fixtures/legacy/` tree.

7. Run the complete compatibility matrix and quality gates:

   ```bash
   cargo test --test schema_provenance_test
   cargo test --test oscal_1_2_3_compatibility_test
   cargo test --test assessment_plan_test
   cargo test --test ssp_template_test
   cargo test --test round_trip_test --test integration_round_trip
   cargo test --test oscal_cli_round_trip
   cargo test
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --check
   ```

   The oscal-cli suite may skip when the tool is unavailable; record that as
   `unavailable`, not as a passing interoperability result. Every other gate is
   mandatory and offline.

8. Review the final diff. Generated fixture changes must be limited to the new
   version declaration or a documented compatibility correction. Confirm that
   no historical schema registry, runtime downloader, `--schema-version`
   selector, Control Mapping model, or AP/SSP public validation/export surface
   was added.

If any identity, digest, offline-resolution, schema, semantic-equivalence, or
quality check fails, stop the upgrade and leave the current baseline intact.

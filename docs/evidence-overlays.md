# Evidence overlays

`forge linkage overlay` creates a new OSCAL JSON Catalog or Component Definition
with complete evidence-linkage metadata in `back-matter.resources`. It preserves
the entire decoded original document, including its existing resources and
sensitive prose or embedded content. Keep the original and derived artifact under
the same access and sharing policy.

The command uses the existing reviewed linkage manifest. Select one exact declared
Catalog requirement key or Component Definition implementation key that
participates in a link. Profile, resolved-Catalog companion, SSP, Assessment
Results and POA&M targets are not admitted by this candidate. The supported-model
disposition and independent consumer interoperability remain open F09 gates.

```bash
forge linkage overlay \
  --manifest project/linkage.json \
  --target-resource policy \
  --as-of 2026-10-04 \
  --output native/catalog-overlay.json
```

This example assumes `project/linkage.json` declares the Catalog key `policy`
at `native/catalog.json`. The new output is `project/native/catalog-overlay.json`;
its path is relative to the manifest's parent, regardless of the current directory.

All four flags are required. `--as-of` is a literal, valid `YYYY-MM-DD` calendar
date. `--manifest` retains its original normalized spelling: dot, parent,
redundant-separator and symlink ancestry aliases refuse. A bare manifest filename
uses the current directory as its root. Manifest declarations retain their
existing root-relative path rules and hash requirements.

`--output` is a new portable `.json` path relative to the manifest's parent. Its
parent must already exist and must be the selected target's exact directory.
Keeping the same directory preserves the base of the original relative hrefs.
The command creates no directories, overwrites no destination and emits no
artifact to stdout. Local publication uses the existing no-replacement publisher
on Linux and macOS; other platforms refuse publication. A durability failure
after publication can leave a complete destination, which must not be replaced
on retry.

## Added records and their interpretation

Every added resource contains exactly a UUID and two properties in
`https://policy-forge.github.io/ns/linkage-overlay/1`: `profile` has the literal
`forge.linkage-overlay/1`, and `record` has a complete compact JSON object. The
packaged [resource schema](../schemas/forge.linkage-overlay-resource-1.schema.json)
and [decoded-record schema](../schemas/forge.linkage-overlay-record-1.schema.json)
are both consumed during preparation, alongside full native OSCAL validation.

The generated suffix contains one provenance resource, every project link in
key order, and every declared evidence record in key order. It includes evidence
unreferenced by a link and unavailable local or non-fetched URI evidence. A
many-to-many link retains its two complete subject arrays and exact evidence
membership; it is not expanded into invented pairwise links.

Provenance records include the explicit date, original manifest hash, selected
target identity, complete native source identities and link/evidence counts.
Link records retain exact subject fingerprints, evidence-resource references,
recorded implementation status and existing reviewer key/time. Evidence records
retain current, expiring, expired, changed, unavailable or unverified-URI states,
recorded validity dates and the explicit approved/observed hash-size pairs or
nullable URI expected hash. URI contents are never retrieved.

Added records omit source paths, URI values, evidence bytes, excerpts, titles,
owners, roles, rationale and policy/impact prose. Keys, reviewer identifiers,
versions, hashes and dates can still be sensitive. The original document's
content is preserved, so the whole artifact is not sanitized. Recorded linkage,
status, review and freshness remain association metadata; generation supplies
no new assessment conclusion, evidence-quality judgment or approval.

## Identity, preservation and refusal

Generated UUIDs use a separate version-5 namespace and eight length-framed stable
fields: profile, project, target side, target key, model, canonical target root
UUID, record kind and record key. Paths, raw hashes, dates, prose and declaration
order do not affect these identities. Original UUID spelling stays intact in the
preserved document and source metadata. Every complete UUID-valued original
string is checked by parsed UUID value; a generated collision refuses rather
than reusing or renumbering an identity.

One actual capture holds the complete original source/evidence closure, including
unused declared roots and absent local observations. Selection and native
freshness preparation separately parse the same held manifest bytes. The command
rechecks the complete original generations immediately before publication. A
source change, unsafe path, output/input namespace collision, ambiguity, invalid
schema or exceeded bound refuses with exit `2`; success returns `0`. Freshness
states alone do not change this exit contract into an assessment gate.

The preservation oracle requires the complete serialized/reparsed document to
equal its expected tree. Removing only the generated suffix and any containers
newly introduced for it must recover the entire original JSON value. This
includes original arrays, resource order, metadata, schema-admitted fields and
integer values. Original formatting/object-key order, arbitrary numeric
representations, YAML/XML output and validation of external href targets are
outside this guarantee. Duplicate keys, floating-point/exponent numeric spellings,
out-of-range integers, BOM and trailing JSON refuse through existing strict
native decoding.

The command shares the existing 100 MiB retained-original raw-byte budget,
10,137 input-observation slots, 100,000-relationship budget and 10 MiB projection
budget. Directory proofs remain separate from file/absence input slots. UUID
occurrences and copied associations are admitted before
retained growth; both compact payloads and escaped wrappers are conservatively
charged. A capped writer admits at most 50 MiB of complete final output, including
the final newline. These bounds do not claim a fixed heap footprint, syscall
preemption or an atomic filesystem snapshot.

## Verification and remaining gates

See the [integration verification record](evidence-overlays-verification.md) for exact source pins,
documentation census, measured test coverage, failures and platform limits. Local
tests and a draft PR establish neither independent consumer interoperability nor
supported-model, human, security, privacy or launch acceptance. Additional models
and formats remain scoped F09 work; the goal-wide documentation review and
verified 2.0.0 release candidate remain required.

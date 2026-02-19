# Research: 030-prd-profile-generation

**Phase**: 0 — Research
**Status**: Complete — no NEEDS CLARIFICATION items remain

---

## Decision 1: assemble_metadata signature compatibility

**Decision**: Create a `DocumentMetadata` with `title: "Policy Baseline Profile"` and `version: "1.0.0"`, pass it to `assemble_metadata(&doc_meta, None)`.

**Rationale**: The actual `assemble_metadata` function signature is `fn(doc_metadata: &DocumentMetadata, options: Option<MetadataOptions>) -> Result<OscalMetadata, ForgeError>`. It does not accept a bare string. The AR's pseudocode was simplified. `DocumentMetadata` implements `Default` so minimal construction is needed.

**Alternatives Considered**: Duplicating metadata generation logic inline — rejected (violates DRY/WI-11 reuse requirement).

---

## Decision 2: Project structure — single crate, not workspace

**Decision**: Add `Profile` subcommand inline to the existing single-crate `src/` structure. No new crate or workspace needed.

**Rationale**: Despite the constitution template showing a workspace layout, FORGE is a single Cargo crate with modules at `src/{cli,export,ingest,model,oscal,parse,testing,validate}/`. The constitution's workspace template is aspirational guidance, not current structure. Constitution Principle I applies to future crate extraction if/when complexity warrants it.

**Alternatives Considered**: Creating a new `crates/profile/` workspace member — rejected as over-engineering for ~150 LOC that closely follows the existing CLI dispatch pattern.

---

## Decision 3: CLI module pattern — inline or separate file

**Decision**: Add `cli/profile.rs` as a new module parallel to `cli/convert.rs`, `cli/export.rs`, and `cli/validate.rs`. Add `Profile { ... }` variant to `Commands` enum in `cli/mod.rs`.

**Rationale**: Matches the established pattern exactly. Each subcommand has its own `cli/{name}.rs` handler file. `Commands` uses inline struct fields (not separate `Args` structs) per the existing `Convert`, `Export`, `Validate` patterns.

**Alternatives Considered**: `ProfileArgs` struct with `#[command(flatten)]` — rejected for inconsistency with existing pattern.

---

## Decision 4: OSCAL Profile JSON root key

**Decision**: Use a `ProfileRoot { profile: OscalProfile }` wrapper struct serialized by serde to produce `{"profile": {...}}`.

**Rationale**: OSCAL v1.2.0 requires the root JSON object key to be `"profile"` (parallel to `"catalog"` for Catalogs). The existing `CatalogEnvelope` pattern wraps with the root key. The `ProfileRoot` struct mirrors this approach.

**Alternatives Considered**: Manual `serde_json::json!` construction — rejected (bypasses type safety).

---

## Decision 5: Control ID parsing (split, trim, dedup)

**Decision**: Split on `,`, trim whitespace from each element, collect into `Vec<String>`, then dedup using a seen-set (preserving order). Reject empty strings with `ForgeError::InvalidArgument`.

**Rationale**: PRD EC-2 requires whitespace trimming; EC-4 requires deduplication; EC-5 requires rejection of empty strings. Order-preserving dedup (seen `HashSet`) is idiomatic Rust. Using `BTreeSet` would sort IDs — not desirable, user order should be respected.

**Alternatives Considered**: Sorting before dedup — rejected (changes user-specified order without benefit).

---

## Decision 6: Error for missing catalog path

**Decision**: Check `Path::new(catalog_path).exists()` in `cli/profile.rs` before calling `build_profile`. Return `ForgeError::Io` with a descriptive message if the file does not exist.

**Rationale**: PRD S-3 requires an actionable error. The catalog path is NOT read (per AR guardrails and PRD anti-patterns), but existence should be checked for usability. Actual JSON validation of catalog content is WI-32's scope.

**Alternatives Considered**: Not checking at all — rejected (PRD S-3 requires actionable error).

---

## Decision 7: Profile metadata title

**Decision**: Default profile title to `"Policy Baseline Profile"`. No `--title` flag in WI-30 scope.

**Rationale**: PRD scope does not include a `--title` flag. The AR specifies `"Policy Baseline Profile"` as the default. A future WI can add `--title` if needed. Using a `DocumentMetadata` with this title satisfies WI-11 reuse requirement.

**Alternatives Considered**: Deriving title from the catalog filename — rejected (adds catalog-reading dependency, deferred to WI-32).

---

## Decision 8: Module location for Profile structs

**Decision**: Add `src/oscal/profile.rs` for `OscalProfile`, `ProfileImport`, `ControlSelection`, `ProfileRoot`, `SelectionMode`, and `build_profile`. Expose via `src/oscal/mod.rs` re-exports. The CLI handler lives at `src/cli/profile.rs`.

**Rationale**: Follows the established pattern: `OscalCatalog` is in `src/oscal/catalog.rs`, `OscalComponentDefinition` in `src/oscal/component_definition.rs`. Profile types belong in the same module family.

**Alternatives Considered**: Putting structs directly in `cli/profile.rs` — rejected (business logic must be in library crate, CLI is a thin dispatcher).

---

## OSCAL v1.2.0 Profile JSON Structure (verified against NIST reference)

```json
{
  "profile": {
    "uuid": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    "metadata": {
      "title": "Policy Baseline Profile",
      "last-modified": "2026-09-22T10:00:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "imports": [
      {
        "href": "./policy-catalog.json",
        "include-controls": [
          {
            "with-ids": ["POL-AC-001", "POL-AC-002"]
          }
        ]
      }
    ]
  }
}
```

Key structural facts:
- Root key: `"profile"` (not `"catalog"` or `"component-definition"`)
- `imports` is an array; WI-30 produces exactly one entry
- Exactly one of `include-controls` or `exclude-controls` appears (not both)
- Each is an array; WI-30 produces a single-element array with one `ControlSelection`
- `with-ids` is an array of strings (control IDs)
- `metadata` follows the same shape as Catalog/Component Definition metadata
- The `modify` section is absent (WI-31 scope)

---

## No Remaining Unknowns

All NEEDS CLARIFICATION items resolved. Implementation can proceed to Phase 1.

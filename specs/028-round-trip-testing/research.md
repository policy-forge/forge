# Research: 028 Multi-Format Round-Trip Testing

**Date**: 2026-02-15
**Status**: Complete

## Research Items

### R-1: XML Deserialization Gap (CRITICAL)

**Question**: WI-26 delivered XML serialization (`serialize_catalog_to_xml`, `serialize_component_definition_to_xml`) but NOT XML deserialization. How do we deserialize XML back to internal model structs for JSON → XML → JSON round-trip testing?

**Finding**: The codebase has no XML deserialization. The XML serializer in `src/export/xml_serializer.rs` uses `quick_xml::Writer` with custom element-by-element serialization (not serde-based). The OSCAL model structs (`OscalCatalog`, `CatalogEnvelope`, `ComponentDefinition`, `ComponentDefinitionEnvelope`) derive both `Serialize` and `Deserialize` from serde.

**Options Evaluated**:

1. **Enable `quick-xml` serde feature and use `quick_xml::de::from_str()`**
   - Pros: Leverages existing serde `Deserialize` derives on model structs; minimal new code
   - Cons: The custom XML serializer may produce XML that's incompatible with quick-xml's serde deserializer (e.g., namespace prefixes, XML declaration, element ordering). Requires testing/tuning.
   - Risk: Medium — may require serde XML attribute annotations (`#[serde(rename)]`) to match OSCAL XML element names

2. **Implement custom XML deserialization with `quick_xml::Reader`**
   - Pros: Full control over parsing; mirrors the custom serializer
   - Cons: Significant code (~500+ LOC per model type); duplicates model knowledge
   - Risk: Low correctness risk, high effort

3. **Value-level round-trip (bypass typed model for XML path)**
   - Approach: Parse XML into a generic tree, convert to `serde_json::Value` using mapping rules
   - Pros: No need for typed deserialization
   - Cons: Complex XML-to-JSON mapping; fragile; doesn't test the actual model round-trip
   - Risk: High complexity, may not satisfy PRD requirements

**Decision**: Option 1 — Enable `quick-xml` serde feature. The OSCAL structs already derive `Deserialize`. We need to add `#[serde(rename)]` attributes where the XML element names differ from the Rust field names (e.g., `back-matter`, `last-modified`, `oscal-version`). These renames already exist in the structs for JSON serialization. The quick-xml serde deserializer respects serde attributes. If compatibility issues arise with the custom XML output, we add targeted serde XML annotations (e.g., `$value` for text content).

**Rationale**: Option 1 is the lowest-effort path that produces real model-level round-trip testing. The OSCAL structs already use `#[serde(rename = "...")]` for hyphenated field names, which quick-xml's serde integration respects. This approach also benefits WI-29 (export subcommand) which will need XML deserialization for format conversion.

**Alternatives Rejected**: Option 2 is too much code for a testing work item. Option 3 doesn't test the actual model fidelity.

---

### R-2: Semantic Equivalence Approach

**Question**: How should semantic equivalence comparison work for `serde_json::Value` trees?

**Finding**: The existing `yaml_equivalence_test.rs` already uses `assert_eq!` on `serde_json::Value` for YAML equivalence. This works because `serde_json::Value::Object` uses `serde_json::Map<String, Value>` which implements `PartialEq` with key-order-independent comparison (it's backed by `BTreeMap` or an insertion-ordered map where `PartialEq` compares entries as sets).

**Decision**: Implement a custom `assert_semantic_equivalence` function that provides richer diff output than `assert_eq!`. The function recursively compares `serde_json::Value` nodes and collects `EquivalenceDiff` entries with JSON Pointer paths. Supplement with `assert_json_diff` crate for enhanced failure messages in test output.

**Rationale**: `assert_eq!` on `Value` already handles key-order-independence and type preservation. A custom function adds structured diff output (PRD M-7) and a reusable `EquivalenceResult` struct (PRD S-3). The `assert_json_diff` crate provides developer-friendly diff formatting for free.

---

### R-3: Test Fixture Strategy

**Question**: What test fixtures should be used for round-trip testing?

**Finding**: The codebase has golden fixtures at:
- `tests/fixtures/golden/small/` — small catalog + component definition JSON (~7.5K, ~5.2K)
- `tests/fixtures/golden/medium/` — medium catalog + component definition JSON (~21K, ~14.6K)
- `tests/fixtures/golden/complex/` — complex catalog + component definition JSON (~53K, ~36K)

The existing `yaml_equivalence_test.rs` runs the pipeline from Markdown fixtures to generate JSON, then round-trips through YAML. For round-trip testing, we should use the pre-built golden JSON fixtures directly (avoiding pipeline execution overhead and timestamp/UUID non-determinism).

**Decision**: Use golden JSON fixtures from `tests/fixtures/golden/` for round-trip testing. Parse the JSON fixture to `serde_json::Value`, deserialize to the typed model, serialize to target format, deserialize back, re-serialize to `serde_json::Value`, and compare. Use all three sizes (small, medium, complex) for comprehensive coverage. Create additional small fixtures for YAML type coercion edge cases.

**Rationale**: Golden fixtures are pre-validated OSCAL artifacts with realistic structure. Using them directly avoids pipeline execution cost and non-determinism. The complex fixture (~53K, many controls) satisfies PRD S-4 (50+ control fixture).

---

### R-4: `assert_json_diff` Crate Evaluation

**Question**: Should `assert_json_diff` be added as a dev-dependency?

**Finding**: `assert_json_diff` (MIT license, well-maintained, 3M+ downloads) provides `assert_json_eq!` and `assert_json_include!` macros with rich diff output. It compares `serde_json::Value` trees structurally.

**Decision**: Add `assert_json_diff` as a dev-dependency. Use it as a supplement to the custom `assert_semantic_equivalence` function for developer-friendly failure output in tests.

**Rationale**: Small dependency footprint (only depends on `serde` and `serde_json`, already in the dependency tree). Provides immediate developer productivity on test failures. Aligns with AR recommendation.

---

### R-5: Module Placement for Semantic Equivalence Utility

**Question**: Where should the `semantic_eq` module live?

**Finding**: The AR suggests `src/testing/semantic_eq.rs`. However, the codebase currently has no `src/testing/` module. Test support code lives in `tests/common/` (e.g., `fixture_generator.rs`, `mod.rs`).

**Options**:
1. `src/testing/semantic_eq.rs` — library code, accessible from all tests and downstream crates
2. `tests/common/semantic_eq.rs` — test support code, accessible from integration tests only

**Decision**: Place in `src/testing/semantic_eq.rs` as a library module gated behind `#[cfg(test)]` or a `testing` feature flag. This satisfies PRD S-3 (reusable module for WI-29 and WI-37). If downstream crates need it, it can be exposed via a `testing` feature flag. For now, use `#[cfg(test)]` gating since all consumers are within the same crate's test suite.

Actually, since integration tests in `tests/` need access and `#[cfg(test)]` only applies to `cargo test` on library code, we should make it a public module accessible during testing. Use `pub mod testing` in `lib.rs` without `#[cfg(test)]` gating — the module contains only data types and a pure comparison function with no side effects, so it's safe to include in production builds. Alternatively, use a Cargo feature flag `testing` that consumers enable in `[dev-dependencies]`.

**Revised Decision**: Add `pub mod testing` to `src/lib.rs` containing the `semantic_eq` submodule. The module exports `EquivalenceResult`, `EquivalenceDiff`, and `assert_semantic_equivalence`. Since these are lightweight data types and a pure function, the runtime cost is zero if unused. This is the simplest approach that satisfies reusability (PRD S-3).

---

### R-6: XML Deserialization — Serde Compatibility Assessment

**Question**: Will `quick_xml::de::from_str()` work with the XML output from the custom `quick_xml::Writer`-based serializer?

**Finding**: The custom XML serializer in `src/export/xml_serializer.rs`:
- Produces XML declaration: `<?xml version="1.0" encoding="UTF-8"?>`
- Adds OSCAL namespace on root: `xmlns="http://csrc.nist.gov/ns/oscal/1.0"`
- Uses hyphenated element names: `back-matter`, `last-modified`, `oscal-version`
- Uses `prop` elements with attributes: `<prop name="..." value="..." />`
- Uses text content in elements: `<title>text</title>`

`quick_xml`'s serde deserializer:
- Handles XML declarations automatically (skips them)
- Handles namespaces (can ignore them with appropriate configuration or serde annotations)
- Maps element names to struct field names (respects `#[serde(rename = "...")]`)
- Maps attributes to fields with `@` prefix convention or `#[serde(rename = "@attr")]`

**Potential Issues**:
1. Namespace prefix on root element — quick-xml serde may include namespace in element name
2. `prop` elements use attributes (`name`, `value`, `ns`) — need `#[serde(rename = "@name")]` annotations for XML deserialization
3. `OscalProp` currently uses struct fields without `@` prefix — works for JSON but may conflict with XML attribute deserialization

**Decision**: Enable `quick-xml` `serde` feature. Add XML-specific serde annotations to structs that use XML attributes (primarily `OscalProp`, `OscalLink`). Use `#[serde(rename = "@name")]` for XML attribute fields. This may require a serde field alias strategy to support both JSON and XML from the same structs.

**Risk Mitigation**: If quick-xml serde compatibility issues are too complex, fall back to implementing targeted XML deserialization functions that parse XML with `quick_xml::Reader` and construct model structs manually — but only for Catalog and Component Definition (not a general solution).

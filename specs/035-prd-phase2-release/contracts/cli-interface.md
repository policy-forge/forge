# CLI Interface Reference: Phase 2 Commands

**Feature**: WI-35 | **Date**: 2026-02-19
**Note**: No new interfaces introduced in WI-35. This documents existing Phase 2 interfaces under test.

---

## forge convert (WI-26, WI-27 additions)

```
forge convert <INPUT> --strategy <catalog|component> --format <json|xml|yaml>
              [--source-profile <path>]  # required for component strategy
              [--output <path>]          # if omitted, stdout
              [--verbose]
```

**Formats added in Phase 2**: `xml` (WI-26), `yaml` (WI-27). `json` existed in Phase 1.

---

## forge export (WI-29)

```
forge export <INPUT>  --format <json|xml|yaml>
             [--output <path>]           # if omitted, stdout
             [--verbose]
```

**Auto-detects** input format from file extension (`.json`, `.xml`, `.yaml`/`.yml`).
**Supports all format pairs**: json→xml, json→yaml, xml→json, xml→yaml, yaml→json, yaml→xml.

---

## forge profile (WI-30, WI-31)

```
forge profile --catalog <path>
              [--include <id1,id2,...>]   # select by control ID
              [--exclude <id1,id2,...>]   # exclude by control ID
              [--set-param <id> <value>]  # repeatable; modify.set-parameters
              [--format <json|xml|yaml>]  # default: json
              [--output <path>]           # if omitted, stdout
              [--verbose]
```

**Behavior**:
- `--include` → `imports[].include-controls[].with-ids`
- `--exclude` → `imports[].exclude-controls[].with-ids`
- `--set-param` (repeatable) → `modify.set-parameters[]`
- If neither `--include` nor `--exclude` specified → includes all controls (full import)

---

## forge validate (WI-19)

```
forge validate <INPUT>
               [--verbose]
```

**Returns**:
- Exit 0 + "Valid" message: artifact passes OSCAL schema validation
- Exit non-zero + error message: schema violation details

**Supports**: Catalog (`.json`/`.xml`/`.yaml`), Component Definition, Profile — auto-detected from JSON/XML structure.

---

## forge --version

```
forge --version
```

**Expected output in v0.2.0**: `forge 0.2.0`

---

## Integration Test Stubs

```rust
// tests/integration_round_trip.rs
fn forge_bin() -> std::process::Command {
    std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
}

// Round-trip: JSON → XML → JSON
fn catalog_json_xml_json_round_trip() {
    // 1. forge convert ... --format json --output catalog.json
    // 2. forge export catalog.json --format xml --output catalog.xml
    // 3. forge export catalog.xml --format json --output catalog_rt.json
    // 4. assert_semantic_equivalence(original_json, round_tripped_json)
}

// tests/integration_profile_e2e.rs
fn profile_include_produces_valid_oscal() {
    // 1. forge convert ... --format json --output catalog.json
    // 2. forge profile --catalog catalog.json --include <ids> --format json --output profile.json
    // 3. assert!(profile["profile"]["imports"][0]["include-controls"].is_array())
    // 4. forge validate profile.json → exit 0
}

// tests/integration_cross_feature.rs
fn normative_props_survive_xml_round_trip() {
    // 1. forge convert MIXED_POLICY --format json --output catalog.json
    // 2. Parse: assert prop[name="modality"] exists on normative controls
    // 3. forge export catalog.json --format xml --output catalog.xml
    // 4. forge export catalog.xml --format json --output catalog_rt.json
    // 5. Parse: assert prop[name="modality"] preserved in round-tripped JSON
}

// tests/integration_regression.rs
fn phase1_catalog_structure_regression() {
    // 1. forge convert tests/fixtures/golden/small/input.md --format json --output catalog.json
    // 2. Parse JSON
    // 3. Assert catalog.uuid is present
    // 4. Assert catalog.metadata.oscal-version == "1.2.0"
    // 5. Assert catalog.groups is non-empty
    // 6. Assert each control has id, title, parts[0].prose
    // (Allow: new prop and param elements from Phase 2 enrichment)
}
```

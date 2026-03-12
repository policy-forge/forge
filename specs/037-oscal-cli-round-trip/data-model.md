# Data Model: oscal-cli Round-Trip Validation (WI-37)

**Branch**: `037-oscal-cli-round-trip` | **Date**: 2026-03-12

---

## New Entities (`src/round_trip/`)

### `Divergence` (`divergence.rs`)

Represents a single difference discovered between the original FORGE-generated OSCAL artifact and the round-tripped artifact.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `json_path` | `String` | Yes | RFC 6901 JSON Pointer path to the differing element (e.g., `/catalog/metadata/title`) |
| `expected` | `serde_json::Value` | Yes | Value from the original FORGE output |
| `actual` | `serde_json::Value` | Yes | Value from the round-tripped output |
| `classification` | `DivergenceClass` | Yes | Classification of the divergence |
| `description` | `String` | Yes | Human-readable explanation of the difference |
| `resolution` | `Option<ResolutionStatus>` | No | Resolution status; `None` until investigated and actioned (PRD M-6, AC-6) |

**Validation rules**:
- `json_path` must start with `/` (RFC 6901 format)
- `expected` and `actual` are never both `Value::Null` simultaneously (a missing key produces a specific classification, not null values)
- `resolution` is `None` on initial discovery; set after investigation

**Serialization**: `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]`

---

### `DivergenceClass` (`divergence.rs`)

Classifies the cause and appropriate action for a divergence.

| Variant | Meaning | Action |
|---------|---------|--------|
| `ForgeFix` | FORGE output is non-conformant; OSCAL spec is clear | Fix FORGE output |
| `OscalCliDiff` | oscal-cli introduces a non-standard transformation | Report upstream to NIST; document |
| `Acceptable` | Acceptable variation (empty array vs. omitted, whitespace, formatting) | Document; no fix needed |

**Serialization**: `#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]`

---

### `ResolutionStatus` (`divergence.rs`)

Tracks the resolution state of a divergence after investigation (PRD M-6, AC-6, spec US2 AS-2).

| Variant | Meaning |
|---------|---------|
| `Fixed` | FORGE output has been corrected; divergence no longer occurs on re-run |
| `Accepted` | Divergence is an acceptable variation; no fix required; documented |
| `ReportedUpstream` | Divergence caused by oscal-cli; issue reported to NIST |

**Lifecycle**: `resolution` starts as `None` (undecided) on initial discovery and is set after investigation. A `RoundTripResult` where all `ForgeFix` divergences have `resolution: Some(Fixed)` satisfies M-5.

**Serialization**: `#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]`

---

### `RoundTripResult` (`divergence.rs`)

Aggregate result of a single round-trip validation run for one artifact type.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `artifact_type` | `String` | Yes | OSCAL artifact type: `"Catalog"` or `"ComponentDefinition"` |
| `source_path` | `PathBuf` | Yes | Path to the original FORGE-generated JSON artifact |
| `passed` | `bool` | Yes | `true` if no divergences with `ForgeFix` or `OscalCliDiff` classification |
| `divergences` | `Vec<Divergence>` | Yes | All divergences found (including `Acceptable`); empty on clean pass |

**Pass definition**: `passed = divergences.iter().all(|d| d.classification == DivergenceClass::Acceptable)`

**Serialization**: `#[derive(Debug, Serialize)]`

---

### `OscalComparisonRules` (`rules.rs`)

Configuration driving the semantic comparison algorithm.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `unordered_array_paths` | `HashSet<String>` | `["props", "links", "parts"]` | JSON key names whose array values are compared without regard to element order (O(1) lookup) |
| `ignored_paths` | `Vec<String>` | `[]` | JSON Pointer prefixes to skip entirely during comparison (reserved for future use) |

**Construction**: `OscalComparisonRules::default()` returns the standard OSCAL rules (props, links, parts unordered).

**Matching strategy for unordered arrays**:
1. Try to match by `uuid` field (OSCAL primary identity key for most elements)
2. Fall back to `name` + `ns` composite key (for props without uuid)
3. Fall back to positional comparison (conservative; never silently classifies as acceptable)

---

## Extended Entities (`src/oscal_cli/`)

### `OscalFormat` (`mod.rs`)

Enumerates the OSCAL serialization formats supported by oscal-cli's `convert` command.

| Variant | CLI flag | Description |
|---------|----------|-------------|
| `Json` | `--to=json` | JSON format |
| `Xml` | `--to=xml` | XML format |
| `Yaml` | `--to=yaml` | YAML format |

**Serialization**: Not serialized directly; converted to CLI flag string via `impl Display`.

---

### `ConvertArgs` (`mod.rs`)

Arguments for a single `oscal-cli convert` invocation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `input_path` | `PathBuf` | Yes | Canonicalized absolute path to the input OSCAL file |
| `output_path` | `PathBuf` | Yes | Path where the converted output will be written |
| `output_format` | `OscalFormat` | Yes | Target format for this conversion step |
| `timeout` | `Duration` | Yes | Per-invocation timeout (30 seconds per clarification Q4) |

---

### `ConvertResult` (`mod.rs`)

Successful result of an `oscal-cli convert` invocation.

| Field | Type | Description |
|-------|------|-------------|
| `output_path` | `PathBuf` | Absolute path to the written output file (mirrors `ConvertArgs::output_path`) |
| `warnings` | `Vec<String>` | Any stderr lines from oscal-cli when exit code was 0 |

---

## Relationships

```
RoundTripResult
  └── divergences: Vec<Divergence>
        ├── classification: DivergenceClass
        └── resolution: Option<ResolutionStatus>

OscalComparisonRules
  └── used by: compare_oscal_json()
        └── returns: Vec<Divergence>
              └── consumed by: RoundTripResult

ConvertArgs
  └── output_format: OscalFormat
  └── used by: OscalCliInvoke::convert()
        └── returns: ConvertResult

run_round_trip_chain()
  └── calls: OscalCliInvoke::convert() × 3
  └── returns: PathBuf (final round-tripped JSON)

write_divergence_log()
  └── serializes: RoundTripResult → JSON file
```

---

## State Transitions

**Divergence resolution lifecycle**:
```
Discovered → ForgeFix (default)
           → OscalCliDiff (after investigation proves oscal-cli is the source)
           → Acceptable (after investigation proves it's a harmless variation)

ForgeFix → [FORGE code fixed] → Re-run → No divergence (removed from log)
OscalCliDiff → [Reported upstream] → Resolution = "reported_upstream"
Acceptable → Resolution = "accepted"
```

---

## Serialized Divergence Log Schema (JSON)

Written by `write_divergence_log()` to the configured output path:

```json
{
  "artifact_type": "Catalog",
  "source_path": "/path/to/catalog.json",
  "passed": true,
  "divergences": []
}
```

When divergences exist:
```json
{
  "artifact_type": "Catalog",
  "source_path": "/path/to/catalog.json",
  "passed": false,
  "divergences": [
    {
      "json_path": "/catalog/metadata/title",
      "expected": "My Security Policy",
      "actual": "my-security-policy",
      "classification": "ForgeFix",
      "description": "Title casing not preserved through XML serialization",
      "resolution": "Fixed"
    },
    {
      "json_path": "/catalog/groups/0/controls",
      "expected": [],
      "actual": null,
      "classification": "Acceptable",
      "description": "FORGE emits empty array; oscal-cli omits key when empty",
      "resolution": "Accepted"
    }
  ]
}
```

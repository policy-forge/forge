# Quickstart: oscal-cli Round-Trip Validation (WI-37)

**Branch**: `037-oscal-cli-round-trip` | **Date**: 2026-03-12

---

## Prerequisites

1. **Rust 1.93.0+** (stable):
   ```bash
   rustup show
   ```

2. **oscal-cli** (Java-based, requires JRE 11+):
   ```bash
   oscal-cli --version
   # Expected: oscal-cli x.x.x
   ```
   If not installed: https://github.com/usnistgov/oscal-cli

3. **Verify the build compiles**:
   ```bash
   cargo build
   ```

---

## Running Unit Tests (no oscal-cli required)

Unit tests for the comparator, divergence types, and log writer run without oscal-cli:

```bash
cargo test --lib
# Or specifically:
cargo test round_trip
```

---

## Running Integration Tests (oscal-cli required)

The oscal-cli round-trip integration tests live in `tests/oscal_cli_round_trip.rs`. They skip automatically if oscal-cli is not found:

```bash
cargo test --test oscal_cli_round_trip
```

Expected output when oscal-cli is available:
```
test catalog_json_xml_yaml_json_round_trip ... ok
test component_json_xml_yaml_json_round_trip ... ok
test round_trip_skip_when_oscal_cli_unavailable ... ok
```

Expected output when oscal-cli is NOT available:
```
test catalog_json_xml_yaml_json_round_trip ... SKIP: oscal-cli not available
test component_json_xml_yaml_json_round_trip ... SKIP: oscal-cli not available
```

---

## Reading the Divergence Log

After an integration test run, a `divergences.json` file is written to the working directory (or the path specified in the test). Inspect it:

```bash
cat divergences.json | jq .
```

Example output (clean pass):
```json
{
  "artifact_type": "Catalog",
  "source_path": "/tmp/forge-rt-xxx/catalog.json",
  "passed": true,
  "divergences": []
}
```

Example output (with a divergence):
```json
{
  "artifact_type": "Catalog",
  "source_path": "/tmp/forge-rt-xxx/catalog.json",
  "passed": false,
  "divergences": [
    {
      "json_path": "/catalog/metadata/title",
      "expected": "My Security Policy",
      "actual": "my-security-policy",
      "classification": "ForgeFix",
      "description": "Title casing not preserved through XML round-trip",
      "resolution": null
    }
  ]
}
```

---

## Using the Library API (Rust)

```rust
use std::path::Path;
use std::time::Duration;
use forge::round_trip::{
    compare_oscal_json, run_round_trip_chain, write_divergence_log,
    DivergenceClass, OscalComparisonRules, RoundTripResult,
};
use forge::oscal_cli::{detector::PathDetector, invoker::ProcessInvoker, OscalCliDetect};
use tempfile::TempDir;

// Path to a FORGE-generated OSCAL Catalog JSON file
let original_json_path = Path::new("catalog.json");

// 1. Detect oscal-cli
let detector = PathDetector::new();
let info = detector.detect();
if !info.available {
    eprintln!("oscal-cli not available — skipping");
    return;
}
let invoker = ProcessInvoker::new(info.executable_path.unwrap());

// 2. Run the conversion chain (JSON → XML → YAML → JSON)
let temp_dir = TempDir::new().unwrap();
let timeout = Duration::from_secs(30);
let round_tripped_path = run_round_trip_chain(
    original_json_path,
    &invoker,
    temp_dir.path(),
    timeout,
).unwrap();

// 3. Compare semantically
let original = serde_json::from_str(&std::fs::read_to_string(original_json_path).unwrap()).unwrap();
let round_tripped = serde_json::from_str(&std::fs::read_to_string(&round_tripped_path).unwrap()).unwrap();
let rules = OscalComparisonRules::default();
let divergences = compare_oscal_json(&original, &round_tripped, "", &rules);

// 4. Build result and write log
let result = RoundTripResult {
    artifact_type: "Catalog".to_string(),
    source_path: original_json_path.to_path_buf(),
    passed: divergences.iter().all(|d| d.classification == DivergenceClass::Acceptable),
    divergences,
};
write_divergence_log(&result, Path::new("divergences.json")).unwrap();

println!("Round-trip passed: {}", result.passed);
```

---

## Quality Gates

Before marking this feature complete, all of the following must pass:

```bash
cargo test                        # All unit + integration tests
cargo clippy -- -D warnings       # Zero warnings
cargo fmt --check                 # Zero formatting violations
```

And verify the success criteria from the spec:
- **SC-001/002**: Catalog + Component Definition JSON→XML→JSON: zero FORGE-caused divergences
- **SC-003**: Full three-format round-trip: zero FORGE-caused divergences
- **SC-004**: `divergences.json` documents all divergences with classification + resolution
- **SC-005**: Tests skip gracefully (no crash, no failure) when oscal-cli is unavailable

---

## Troubleshooting

| Problem | Likely Cause | Fix |
|---------|-------------|-----|
| `SKIP: oscal-cli not available` | oscal-cli not on PATH | Install oscal-cli or set `PATH` |
| `ForgeError::OscalCliTimeout` | oscal-cli hung or JVM startup slow | Increase timeout or check JRE install |
| `ForgeError::OscalCliExecution` | oscal-cli rejected the OSCAL file | Check the FORGE output is valid OSCAL 1.2.0 |
| Divergences in `divergences.json` | FORGE output differs from oscal-cli canonical | Investigate each divergence; classify and fix |
| `thread panicked` in integration test | TempDir cleanup failed | Check /tmp permissions |

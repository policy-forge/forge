# Data Model: oscal-cli Profile Resolution Integration

**Date**: 2026-03-10
**Feature**: 036-oscal-cli-profile-resolution

## Entities

### OscalCliInfo

Detection result from `OscalCliDetect::detect()`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| available | bool | yes | Whether oscal-cli was found on the system (exists on PATH or at the given path) |
| version | Option\<String\> | no | Version string (e.g., "1.0.3"), None if not detected |
| executable_path | Option\<PathBuf\> | no | Absolute path to the oscal-cli executable |
| functional | bool | yes | Whether `oscal-cli --version` succeeded (distinguishes "found but broken" from "found and working") |

**Validation rules**:
- If `available` is false, `version` and `executable_path` must be None
- If `available` is true and `functional` is false, `executable_path` should be Some (binary found) but `version` may be None (version check failed)

### ResolveResult

Successful invocation result from `OscalCliInvoke::resolve_profile()`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| output_path | PathBuf | yes | Absolute path where resolved Catalog was written |
| warnings | Vec\<String\> | yes | Any stderr warnings from oscal-cli (exit code 0 but stderr non-empty) |

### ResolveArgs

Arguments for a resolve-profile invocation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| profile_path | PathBuf | yes | Canonicalized absolute path to input Profile JSON |
| output_path | PathBuf | yes | Path where resolved Catalog will be written |
| timeout | Duration | yes | Maximum execution time (default: 60s) |

## Traits

### OscalCliDetect

```rust
pub trait OscalCliDetect {
    fn detect(&self) -> OscalCliInfo;
}
```

**Implementations**:
- `PathDetector` — production: searches PATH for oscal-cli, runs `--version`
- `MockDetector` — test: returns preconfigured OscalCliInfo

### OscalCliInvoke

```rust
pub trait OscalCliInvoke {
    fn resolve_profile(&self, args: &ResolveArgs) -> Result<ResolveResult, ForgeError>;
}
```

**Implementations**:
- `ProcessInvoker` — production: spawns `oscal-cli profile resolve` via `std::process::Command`
- `MockInvoker` — test: returns preconfigured success/failure

## Error Variants (additions to ForgeError)

| Variant | Fields | Exit Code | Trigger |
|---------|--------|-----------|---------|
| OscalCliNotFound | — | 4 | oscal-cli not on PATH and no --oscal-cli-path |
| OscalCliNotFunctional | path: PathBuf, detail: String | 4 | oscal-cli found but --version fails |
| OscalCliExecution | exit_code: Option\<i32\>, message: String, stderr: String | 1 | oscal-cli exits non-zero |
| OscalCliTimeout | timeout: Duration | 1 | Execution exceeds timeout |
| ResolveInputNotJson | path: PathBuf | 1 | Input file lacks .json extension |

**Exit code rationale**: Exit code 4 for "external dependency unavailable" — distinct from existing codes 1 (input/IO), 2 (parse/structure), 3 (validation/config).

## State Transitions

```
forge resolve invoked
    → [validate input file exists + is JSON]
        → FAIL: ForgeError::FileNotFound or ResolveInputNotJson
    → [detect oscal-cli]
        → NOT FOUND: ForgeError::OscalCliNotFound (exit 4)
        → FOUND BUT NOT FUNCTIONAL: ForgeError::OscalCliNotFunctional (exit 4)
        → FOUND AND FUNCTIONAL: proceed
    → [invoke oscal-cli profile resolve]
        → TIMEOUT: ForgeError::OscalCliTimeout (exit 1)
        → NON-ZERO EXIT: ForgeError::OscalCliExecution (exit 1)
        → ZERO EXIT + stderr: ResolveResult with warnings
        → ZERO EXIT + no stderr: ResolveResult (clean success)
```

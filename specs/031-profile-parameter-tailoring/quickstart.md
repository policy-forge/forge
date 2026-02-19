# Quick Start: Profile Parameter Tailoring (WI-31)

**Branch**: `031-profile-parameter-tailoring` | **Date**: 2026-02-18

## What It Does

Extends `forge profile` with OSCAL parameter tailoring via the `--set-param` CLI flag. Organizations can override catalog default parameter values when generating a Profile baseline. The generated Profile includes a `modify.set-parameters` section containing the specified overrides.

## Basic Usage

### Set a single parameter value

```bash
forge profile \
  --catalog catalog.json \
  --include POL-AC-001 \
  --set-param POL-AC-001_prm "60 days" \
  --format json
```

Output `modify` section:
```json
"modify": {
  "set-parameters": [
    { "param-id": "POL-AC-001_prm", "values": ["60 days"] }
  ]
}
```

### Set multiple parameters in one command

```bash
forge profile \
  --catalog catalog.json \
  --include POL-AC-001,POL-IR-001 \
  --set-param POL-AC-001_prm "60 days" \
  --set-param POL-IR-001_prm "4 hours" \
  --format json
```

Output `modify` section (entries ordered alphabetically by `param-id`):
```json
"modify": {
  "set-parameters": [
    { "param-id": "POL-AC-001_prm", "values": ["60 days"] },
    { "param-id": "POL-IR-001_prm", "values": ["4 hours"] }
  ]
}
```

### Aggregate multiple values for one parameter

```bash
forge profile \
  --catalog catalog.json \
  --include POL-AC-001 \
  --set-param POL-AC-001_prm "60 days" \
  --set-param POL-AC-001_prm "quarterly" \
  --format json
```

Output — duplicate `param-id` values are combined:
```json
"modify": {
  "set-parameters": [
    { "param-id": "POL-AC-001_prm", "values": ["60 days", "quarterly"] }
  ]
}
```

### No parameter tailoring (WI-30 backward compatible)

```bash
forge profile \
  --catalog catalog.json \
  --include POL-AC-001 \
  --format json
```

Output — no `"modify"` key (identical to WI-30 behavior):
```json
{
  "profile": {
    "uuid": "...",
    "metadata": { ... },
    "imports": [ ... ]
  }
}
```

## Key Behaviors

| Behavior | Detail |
|----------|--------|
| **Ordering** | `set-parameters` entries are alphabetically ordered by `param-id` |
| **Deterministic** | Same inputs always produce byte-for-byte identical output |
| **Aggregation** | Same `param-id` used twice → single entry with combined `values` array |
| **Spaces in values** | Shell-quote: `--set-param prm "60 days"` preserves value as single string |
| **Empty value** | `--set-param prm ""` is valid; generates `values: [""]` |
| **Backward compat** | No `--set-param` → no `"modify"` section in output |

## Building and Testing

```bash
cargo build                        # Build
cargo test                         # Run all tests
cargo test build_modify            # Run modify section unit tests
cargo clippy -- -D warnings        # Lint check
cargo fmt --check                  # Format check
```

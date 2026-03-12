# Quickstart: Assessment Plan Scaffolding — Controls (WI-41)

**Feature Branch**: `041-assessment-plan-controls`
**Date**: 2026-03-12

---

## What This Feature Does

When you pass `--import-ssp <path>` to `forge convert`, FORGE generates an Assessment
Plan JSON skeleton alongside the Catalog or Component Definition. The Assessment Plan's
`reviewed-controls` is populated with all control IDs from the conversion output.

---

## Basic Usage

```bash
# Catalog strategy — generates both catalog.json and policy-assessment-plan.json
forge convert policy.md \
  --strategy catalog \
  --output ./out/catalog.json \
  --import-ssp ./ssp/system-ssp.json

# Component strategy — generates both component.json and policy-assessment-plan.json
forge convert policy.md \
  --strategy component \
  --source-profile ./profiles/nist-800-53.json \
  --output ./out/component.json \
  --import-ssp ./ssp/system-ssp.json

# Without --output (writes to current directory)
# Generates: ./catalog.json  AND  ./policy-assessment-plan.json
forge convert policy.md \
  --strategy catalog \
  --import-ssp ./ssp/system-ssp.json
```

---

## Output Files

Given `forge convert policy.md --strategy catalog --output ./out/catalog.json --import-ssp ./ssp.json`:

| File | Description |
|------|-------------|
| `./out/catalog.json` | OSCAL Catalog (existing behavior) |
| `./out/policy-assessment-plan.json` | NEW: OSCAL Assessment Plan skeleton |

---

## Assessment Plan Structure

```json
{
  "assessment-plan": {
    "uuid": "<deterministic-uuid-v5>",
    "metadata": {
      "title": "Assessment Plan for Corporate Security Policy",
      "last-modified": "2026-03-12T10:30:00Z",
      "version": "1.0.0",
      "oscal-version": "1.2.0"
    },
    "import-ssp": {
      "href": "./ssp/system-ssp.json"
    },
    "reviewed-controls": {
      "description": "Controls derived from Corporate Security Policy for assessment review.",
      "control-selections": [
        {
          "include-controls": [
            { "control-id": "POL-AC-001" },
            { "control-id": "POL-AC-002" },
            { "control-id": "POL-DP-001" }
          ]
        }
      ]
    }
  }
}
```

---

## Error Cases

| Scenario | Error Message |
|----------|---------------|
| `--import-ssp` omitted | *(no AP generated — backward compatible)* |
| `--import-ssp ""` (empty string) | `Validation error: --import-ssp must not be empty` |
| 2+ input files with `--import-ssp` | Warning: AP generation not supported in batch mode |
| Policy with zero controls | AP written with empty `include-controls`; warning emitted |

---

## Determinism

Generating from the same input twice produces identical AP output:

```bash
forge convert policy.md --strategy catalog --import-ssp ./ssp.json --output ./out/a.json
forge convert policy.md --strategy catalog --import-ssp ./ssp.json --output ./out/b.json

# Compare APs (excluding last-modified timestamp):
diff <(jq 'del(.["assessment-plan"].metadata["last-modified"])' ./out/policy-assessment-plan.json) \
     <(jq 'del(.["assessment-plan"].metadata["last-modified"])' ./out/policy-assessment-plan.json)
# (no diff — all UUIDs identical)
```

---

## Building and Testing

```bash
# Build
cargo build

# Run all tests
cargo test

# Run AP-specific tests
cargo test assessment_plan

# Lint + format check (required before commit)
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Scope Notes (WI-41)

- **Included**: Assessment Plan root, metadata, `import-ssp`, `reviewed-controls` with `control-selections`
- **Not included** (WI-42): `assessment-tasks`, `assessment-subjects`
- **Not included** (future): AP schema validation, XML/YAML output for AP, `back-matter`, `assessment-assets`

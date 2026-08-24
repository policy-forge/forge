# Validation and Round-Trip Result Contract

Validation JSON retains all existing fields and adds:

```json
{
  "artifact_path": "policy-catalog.json",
  "model_type": "catalog",
  "declared_oscal_version": "1.2.0",
  "schema_version_used": "1.2.3",
  "supported_input": true,
  "is_valid": true,
  "errors": [],
  "schema_error_count": 0,
  "semantic_error_count": 0
}
```

Text output names the same model, declared version, and schema baseline. Unsupported declarations return non-zero and name both the declaration and available baseline.

Round-trip JSON retains `artifact_type`, `source_path`, `passed`, and `divergences`, and adds:

- `declared_oscal_version`
- `schema_version_used`
- `oscal_cli_version` (nullable only when unavailable)
- `compatibility_classification`: `verified-conversion`, `advisory-older-model-baseline`, or `unavailable`

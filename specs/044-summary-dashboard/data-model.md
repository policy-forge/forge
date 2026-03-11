# Data Model: Summary Dashboard (044)

## Entities

### ConversionStatistics

Accumulates pipeline stage counts during a single conversion run.

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `sections_parsed` | `usize` | `prepare_document` → `PolicyDocument.sections` (recursive count) | Total sections in input document |
| `requirements_extracted` | `usize` | `prepare_document` → `PolicyDocument.sections[*].requirements` (recursive count) | Total requirements after atomization |
| `controls_generated` | `usize` | Catalog: `OscalCatalog.groups[*].controls` (recursive); Component: implemented-requirements count | Total OSCAL controls/implemented-reqs produced |
| `validation_status` | `ValidationStatus` | `validate_catalog_json` / `validate_component_json` result | Schema validation outcome |
| `validation_errors` | `usize` | `ValidationReport.errors().len()` | Count of validation errors |
| `validation_warnings` | `usize` | Reserved for future use (currently always 0) | Count of validation warnings |
| `validation_error_messages` | `Vec<String>` | First 3 from `ValidationReport.errors()` | Up to 3 error messages for display |
| `strategy` | `String` | CLI `--strategy` argument | "catalog" or "component" |
| `output_path` | `String` | CLI `--output` argument or "stdout" | Where the artifact was written |
| `elapsed` | `Duration` | `Instant::now()` / `.elapsed()` | Pipeline execution time |

**Derived fields**:
- `mapping_coverage()` → `f64`: `(controls_generated / requirements_extracted) * 100.0` (returns 0.0 when `requirements_extracted == 0`)

### ValidationStatus

Enum representing OSCAL schema validation outcome.

| Variant | Meaning | When Set |
|---------|---------|----------|
| `Passed` | Schema validation passed with no errors | `validate_*` returns `Ok(_)` |
| `PassedWithWarnings` | Passed but with warnings | Reserved for future validation warning support |
| `Failed` | Schema validation found errors | `validate_*` returns `Err(SchemaValidation)` (pipeline aborts; dashboard not shown) |
| `NotRun` | Validation was not executed | Default; used when validation infrastructure is unavailable |

**Note**: In the current implementation, `Failed` will never appear in the dashboard because validation failure causes the pipeline to abort before the dashboard is printed (per spec EC-5). The variant exists for forward compatibility and for unit testing the formatting function.

## Relationships

```
ConversionStatistics 1──1 ValidationStatus
ConversionStatistics 1──* String (validation_error_messages, max 3)
```

## State Transitions

`ConversionStatistics` is not stateful — it is populated once during pipeline execution and consumed once by the formatter. No state machine.

## Validation Rules

- `sections_parsed >= 0` (guaranteed by `usize`)
- `requirements_extracted >= 0` (guaranteed by `usize`)
- `controls_generated >= 0` (guaranteed by `usize`); can exceed `requirements_extracted` due to atomization
- `validation_error_messages.len() <= 3`
- `mapping_coverage()` returns `0.0` when `requirements_extracted == 0` (zero-division guard)
- `mapping_coverage()` can exceed `100.0` when `controls_generated > requirements_extracted`

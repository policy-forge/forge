# Data Model: Batch Conversion

## Entities

### FileResult

Represents the outcome of converting a single file in a batch.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| input_path | PathBuf | Yes | Path to the input file |
| output_path | Option\<PathBuf\> | No | Path to the generated output file (None on failure) |
| success | bool | Yes | Whether the conversion succeeded |
| error_message | Option\<String\> | No | Error description (None on success) |
| duration | Duration | Yes | Wall-clock time for this file's conversion |

**Invariants**:
- If `success == true`, then `output_path` is `Some` and `error_message` is `None`
- If `success == false`, then `error_message` is `Some` and `output_path` is `None`

### BatchSummary

Aggregation of all `FileResult`s for a batch conversion run.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| total_files | usize | Yes | Total number of input files |
| succeeded | usize | Yes | Count of successful conversions |
| failed | usize | Yes | Count of failed conversions |
| total_duration | Duration | Yes | Wall-clock time for the entire batch |
| results | Vec\<FileResult\> | Yes | Per-file results, sorted by input filename |

**Invariants**:
- `succeeded + failed == total_files`
- `results.len() == total_files`
- `results` is sorted by `input_path` filename (not full path)

### BatchConfig (internal, not persisted)

Configuration for a batch conversion run, derived from CLI arguments.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| input_paths | Vec\<PathBuf\> | Yes | Validated input file paths |
| strategy | Strategy | Yes | Catalog or Component |
| format | OutputFormat | Yes | JSON, XML, or YAML |
| output_dir | Option\<PathBuf\> | No | Output directory (None = current dir) |
| jobs | usize | Yes | Parallelism level (0 = auto/num_cpus) |
| max_size | u64 | Yes | Max input file size in MB |
| source_profile | Option\<String\> | No | Source profile for component strategy |

## Relationships

```
BatchConfig 1──* FileResult : produces
FileResult *──1 BatchSummary : aggregated into
```

## State Transitions

No persistent state. All entities are ephemeral within a single CLI invocation:

```
CLI Args → BatchConfig → [parallel processing] → Vec<FileResult> → BatchSummary → stderr output
```

## Error Types (new)

Added to `ForgeError` enum:

| Variant | Message Pattern | Exit Code |
|---------|----------------|-----------|
| BatchConversion(String) | "Batch conversion error: {0}" | 1 |

This variant covers batch-level errors (thread pool creation failure, output directory validation). Per-file errors use existing `ForgeError` variants and are captured in `FileResult.error_message`.

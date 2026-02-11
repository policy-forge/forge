# CLI Interface Contract: Project Scaffolding

**Feature**: 001-project-scaffolding
**Date**: 2026-02-11

## Overview

This contract defines the FORGE CLI argument interface. All subcommands are stubs in this work item — they accept arguments but print a "not yet implemented" message instead of performing actual operations.

## Global Options

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--help` | `-h` | bool | false | Print help information |
| `--version` | `-V` | bool | false | Print version information |
| `--verbose` | `-v` | bool | false | Enable verbose output |
| `--quiet` | `-q` | bool | false | Suppress non-essential output |

**Conflicts**: `--verbose` and `--quiet` are mutually exclusive.

## Subcommands

### `convert`

Convert a policy document to OSCAL format.

```text
forge convert [OPTIONS] <INPUT>
```

| Argument | Position/Flag | Type | Required | Default | Description |
|----------|--------------|------|----------|---------|-------------|
| `<INPUT>` | Positional | PathBuf | Yes | — | Path to the input policy document |
| `--strategy` | Named | Strategy | No | — | Conversion strategy: `catalog` or `component` |
| `--format` | Named | OutputFormat | No | `json` | Output format: `json`, `xml`, or `yaml` |
| `--output` | Named | PathBuf | No | stdout | Output file path |

**Stub behavior**: Prints `"Convert command not yet implemented"` and exits with code 0.

### `validate`

Validate an OSCAL artifact against schemas.

```text
forge validate <INPUT>
```

| Argument | Position/Flag | Type | Required | Default | Description |
|----------|--------------|------|----------|---------|-------------|
| `<INPUT>` | Positional | PathBuf | Yes | — | Path to the OSCAL artifact to validate |

**Stub behavior**: Prints `"Validate command not yet implemented"` and exits with code 0.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (including stub "not yet implemented" responses) |
| 1 | Application error (ForgeError) |
| 2 | CLI argument parsing error (invalid arguments, missing required args) |

## Error Responses

| Scenario | Behavior |
|----------|----------|
| No subcommand provided | Print help text, exit 0 |
| Unknown subcommand | Print error with available subcommands, exit 2 |
| Missing required argument | Print error with usage hint, exit 2 |
| `--help` flag | Print help text, exit 0 |
| `--version` flag | Print version from Cargo.toml, exit 0 |

## Example Invocations

```bash
# Show help
forge --help
forge convert --help

# Convert (stub)
forge convert policy.md
forge convert policy.md --strategy catalog --format json --output out.json

# Validate (stub)
forge validate artifact.json

# Version
forge --version

# Verbose mode
forge -v convert policy.md
```

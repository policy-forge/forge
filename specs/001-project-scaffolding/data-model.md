# Data Model: Project Scaffolding

**Feature**: 001-project-scaffolding
**Date**: 2026-02-11

## Overview

Project scaffolding introduces three core types: the CLI argument structure, the error type enum, and supporting value enums. No persistent data model or database entities exist in this work item.

## Entities

### ForgeError

The project-wide error type used as the standard `Result` error across all modules.

| Variant | Fields | Source Conversion | Display Message |
|---------|--------|-------------------|-----------------|
| `Io` | `std::io::Error` | `#[from]` auto-conversion | `"I/O error: {0}"` |
| `Parse` | `String` | Manual construction | `"Parse error: {0}"` |
| `Validation` | `String` | Manual construction | `"Validation error: {0}"` |
| `Config` | `String` | Manual construction | `"Configuration error: {0}"` |

**Traits derived**: `Debug`, `thiserror::Error`
**Location**: `src/error.rs`
**Re-exported from**: `src/lib.rs`

### Cli (argument structure)

Top-level CLI definition parsed from command-line arguments.

| Field | Type | Description |
|-------|------|-------------|
| `command` | `Commands` | The subcommand to execute |
| `verbose` | `bool` | Enable verbose output (global flag) |
| `quiet` | `bool` | Suppress non-essential output (global flag) |

**Traits derived**: `Parser`
**Attributes**: `#[command(name = "forge", about = "...", version)]`
**Location**: `src/cli/mod.rs`

### Commands (subcommand enum)

Subcommand routing enum defining available CLI operations.

| Variant | Fields | Description |
|---------|--------|-------------|
| `Convert` | `input: PathBuf`, `strategy: Option<Strategy>`, `format: OutputFormat`, `output: Option<PathBuf>` | Convert a policy document to OSCAL |
| `Validate` | `input: PathBuf` | Validate an OSCAL artifact against schemas |

**Traits derived**: `Subcommand`
**Location**: `src/cli/mod.rs`

### Strategy (value enum)

Conversion strategy for the `convert` subcommand.

| Variant | CLI Value | Description |
|---------|-----------|-------------|
| `Catalog` | `catalog` | Generate OSCAL catalog |
| `Component` | `component` | Generate OSCAL component definition |

**Traits derived**: `ValueEnum`, `Clone`, `Debug`
**Location**: `src/cli/mod.rs`

### OutputFormat (value enum)

Output format for the `convert` subcommand.

| Variant | CLI Value | Description |
|---------|-----------|-------------|
| `Json` | `json` | JSON output (default) |
| `Xml` | `xml` | XML output |
| `Yaml` | `yaml` | YAML output |

**Traits derived**: `ValueEnum`, `Clone`, `Debug`
**Default**: `Json`
**Location**: `src/cli/mod.rs`

## Relationships

```text
Cli --has--> Commands (1:1, via subcommand)
Commands::Convert --uses--> Strategy (0..1, optional)
Commands::Convert --uses--> OutputFormat (1:1, default Json)
All modules --return--> Result<T, ForgeError>
```

## State Transitions

N/A — No state machines in scaffolding. Subcommand stubs print a message and exit.

## Validation Rules

- `input` (PathBuf): Validated by the OS at runtime (file existence checked in WI-2+, not in stubs)
- `strategy`: Optional; when provided, must be a valid `Strategy` variant (enforced by clap `ValueEnum`)
- `format`: Must be a valid `OutputFormat` variant; defaults to `Json` (enforced by clap `ValueEnum`)
- `verbose` and `quiet`: Mutually exclusive (enforced via clap conflict rules)

# Project Configuration (`.forge.toml`)

> PRD 051 · Schema version 1 · FORGE v1.2

A checked-in `.forge.toml` establishes one reviewable source for FORGE command
defaults. It removes repeated flags from developer commands, scripts, and CI
workflows while preserving every existing CLI behavior for repositories that
do not adopt configuration.

## Quick start

Create `.forge.toml` at your repository root:

```toml
schema-version = 1

[convert]
strategy = "catalog"
format = "json"
output = "generated/oscal"   # directory for batch, file for single input
max-size-mb = 10
jobs = 0                     # 0 = auto parallelism
summary = false

[validate]
format = "text"
timeout-seconds = 30
```

Then run commands with explicit inputs only:

```bash
forge convert policies/policy-a.md policies/policy-b.md
forge validate generated/oscal/policy-a.json
```

Validate your configuration at any time:

```bash
forge config check            # uses --config > $FORGE_CONFIG > discovery
forge config check --config path/to/forge.toml
```

## Selection: exactly one config

FORGE selects **at most one** configuration file; files are never merged:

1. `--config <path>` (global option, accepted before or after the subcommand)
2. `FORGE_CONFIG` environment variable
3. Nearest `.forge.toml`, searching from the current working directory upward
   through each ancestor directory
4. No project config (built-in defaults apply)

An explicitly selected file that is missing or invalid is an error — FORGE
never falls back to discovery.

The global `--config` option is consumed only by `convert`, `validate`, and
`config`; using it with any other subcommand is an error rather than being
silently ignored.

## Cross-field validation

`forge config check` rejects configurations that could never run successfully,
such as `convert.strategy = "component"` without `convert.source-profile`.
Command execution still allows an explicit CLI value to satisfy a requirement
that the project file alone leaves unmet.

## Precedence

Effective settings resolve highest-first. Each setting is sourced
independently — overriding one value never discards others from a lower layer.

1. Explicit CLI value
2. Supported environment override (`FORGE_JOBS` in schema version 1)
3. Selected project config value
4. Built-in default

Example: default jobs `0`, config `jobs = 4`, `FORGE_JOBS=2`, CLI `--jobs 1`
resolves to `1`.

## Supported settings (schema version 1)

| Config Key | Type / Allowed Values | CLI Equivalent | Default |
|------------|----------------------|----------------|---------|
| `schema-version` | Integer, exactly `1` | — | required |
| `convert.strategy` | `catalog` or `component` | `--strategy` | none; required |
| `convert.format` | `json`, `xml`, or `yaml` | `--format` | `json` |
| `convert.output` | Project-relative path | `--output` | stdout / current dir |
| `convert.max-size-mb` | Integer `1..=51200` | `--max-size` | `10` |
| `convert.source-profile` | Project-relative regular-file path | `--source-profile` | none |
| `convert.jobs` | Integer `0..=256` | `--jobs` | `0`; `$FORGE_JOBS` may override |
| `convert.stable-id-baseline` | Project-relative regular-file path | `--stable-id-baseline` | none |
| `convert.to` | `catalog`, `component-definition`, or `ssp` | `--to` | none |
| `convert.import-ssp` | Non-empty URI/reference string | `--import-ssp` | none |
| `convert.summary` | Boolean (`--summary` / `--no-summary`) | `--summary` | `false` |
| `validate.schema-type` | `catalog` or `component-definition` | `--schema-type` | auto-detect |
| `validate.format` | `text` or `json` | `--format` | `text` |
| `validate.output` | Project-relative path | `--output` | stdout |
| `validate.timeout-seconds` | Integer `1..=3600` | `--timeout` | `30` |

The schema is **closed**: unknown top-level keys and unknown keys inside
`[convert]` / `[validate]` are errors (with a suggestion when one unambiguous
close key exists). Deliberately unsupported in version 1: command inputs,
globs, `round-trip`, `oscal-cli-path`, hooks, plugins, secrets, and any
configuration for `resolve`, `trace`, `diff`, or `profile`.

## Path rules

- Config paths must be **relative** and resolve from the config file's parent
  directory (the project root).
- Paths must remain inside the project root after lexical normalization;
  `..` segments are allowed only when the result stays inside.
- Absolute config paths are rejected. No `~`, `${VAR}`, `%VAR%`, shell, or glob
  expansion is performed.
- Referenced inputs (`source-profile`, `stable-id-baseline`) must be existing
  regular files.
- The selected config itself must be a regular file, not a symbolic link,
  and no larger than 1 MiB.

## Environment variables

Schema version 1 supports only this allowlist:

| Variable | Purpose | Value Rules |
|----------|---------|-------------|
| `FORGE_CONFIG` | Select one config file when `--config` is absent | File path relative to CWD; empty values are errors |
| `FORGE_JOBS` | Override `convert.jobs` for runner capacity | Integer `0..=256`; invalid values are errors |

No generic environment mapping exists; other `FORGE_*` variables are ignored.

## Validation and failure behavior

All parsing, schema checks, range checks, conflict checks, and path validation
run **before** any output files, directories, or external processes. Invalid
configurations fail with the offending key identified and exit code `3`.
Use `forge config check` in CI to validate without side effects.

## Migration from CLI-only usage

| Before | After (in `.forge.toml`) |
|--------|--------------------------|
| `--strategy catalog` | `convert.strategy = "catalog"` |
| `--format yaml` | `convert.format = "yaml"` |
| `--output dist/oscal` | `convert.output = "dist/oscal"` |
| `--max-size 20` | `convert.max-size-mb = 20` |
| `--jobs 4` | `convert.jobs = 4` (or `FORGE_JOBS=4`) |
| `--summary` | `convert.summary = true` |
| `--timeout 20` | `validate.timeout-seconds = 20` |

One-off exceptions stay on the command line: `forge convert policy.md --jobs 1`.

## Reproducibility limitation

A checked-in configuration makes **option resolution** deterministic. It does
**not** make generated artifacts byte-identical: production catalog/component
metadata currently embeds runtime UUID v4 identifiers and the current UTC
time. Byte-level drift enforcement requires a separate deterministic-generation
or canonical-comparison contract (tracked for the official GitHub Action).
Do not store secrets in `.forge.toml`; it is intended to be committed.

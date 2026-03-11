# Quickstart: oscal-cli Profile Resolution Integration

## Prerequisites

- Rust 1.93.0+ (edition 2024)
- NIST oscal-cli installed and on PATH (optional — graceful degradation if missing)
  - Install: https://github.com/usnistgov/oscal-cli
  - Requires Java runtime

## New Files to Create

```
src/
├── oscal_cli/
│   ├── mod.rs          # Module declaration, re-exports
│   ├── detector.rs     # OscalCliDetect trait + PathDetector implementation
│   └── invoker.rs      # OscalCliInvoke trait + ProcessInvoker implementation
└── cli/
    └── resolve.rs      # forge resolve subcommand handler
```

## Files to Modify

```
src/lib.rs              # Add `pub mod oscal_cli;`
src/cli/mod.rs          # Add `pub mod resolve;`, add Resolve variant to Commands enum
src/error.rs            # Add OscalCli* error variants + exit_code mappings
src/main.rs             # No changes needed (dispatched via cli::execute)
```

## Build Sequence

1. **Error variants first** — add to `src/error.rs` (other modules depend on these)
2. **Data structs + traits** — `src/oscal_cli/mod.rs` (defines OscalCliInfo, ResolveArgs, ResolveResult, traits)
3. **Detector** — `src/oscal_cli/detector.rs` (PathDetector + tests)
4. **Invoker** — `src/oscal_cli/invoker.rs` (ProcessInvoker + tests)
5. **CLI subcommand** — `src/cli/resolve.rs` (wires detector + invoker)
6. **CLI wiring** — update `src/cli/mod.rs` (Commands enum + execute dispatch)
7. **Module registration** — update `src/lib.rs`

## Key Implementation Notes

- oscal-cli subcommand is `profile resolve` (not `resolve-profile`)
- Invocation: `oscal-cli profile resolve -to=json <input> <output>`
- Output file is a required positional arg for oscal-cli (not stdout)
- Use `Command::env_clear()` + allowlist (PATH, HOME, JAVA_HOME, TMPDIR)
- Use argument arrays only — never shell string interpolation
- Timeout via thread-based watchdog that calls `Child::kill()`
- No new crate dependencies required (stdlib sufficient)

## Verification

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

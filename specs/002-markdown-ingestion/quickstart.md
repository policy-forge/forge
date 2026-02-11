# Quickstart: Markdown Ingestion

**Feature**: 002-markdown-ingestion

## Usage

### Ingest a Markdown file

```bash
forge convert policy.md
```

Output (JSON to stdout):
```json
{
  "source_path": "/absolute/path/to/policy.md",
  "fingerprint": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "lines": [
    { "number": 1, "text": "# Access Control Policy" },
    { "number": 2, "text": "" },
    { "number": 3, "text": "All users must authenticate before accessing systems." }
  ]
}
```

### Handle unsupported formats

```bash
forge convert policy.pdf
# Error: Unsupported file format '.pdf'. Only Markdown files (.md, .markdown) are supported.
#        Consider converting with pandoc or markitdown.
# Exit code: 1
```

### Override file size limit

```bash
# Default limit is 10MB; override for large files:
forge convert large-policy.md --max-size 50
```

## Development

### Build and run

```bash
cargo build
cargo run -- convert policy.md
```

### Run tests

```bash
cargo test                    # All tests
cargo test --lib              # Unit tests only
cargo test ingest             # Ingestion tests only
```

### Key source files

| File | Purpose |
|------|---------|
| `src/ingest/mod.rs` | `IngestedDocument`, `SourceLine`, `ingest_file()` |
| `src/error.rs` | `ForgeError` with ingestion variants |
| `src/cli/mod.rs` | CLI arg definitions (`--max-size`) |
| `src/cli/convert.rs` | Convert command handler (calls ingest) |

### New dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.x | Serialization framework |
| `serde_json` | 1.x | JSON output |
| `sha2` | 0.10.x | SHA-256 fingerprint |

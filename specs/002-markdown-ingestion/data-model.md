# Data Model: Markdown Ingestion

**Feature**: 002-markdown-ingestion
**Date**: 2026-02-11

## Entities

### IngestedDocument

Represents a Markdown file that has been successfully read from the filesystem. This is the output of the ingestion process and the input to all downstream pipeline stages.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `source_path` | `PathBuf` (serialized as `String`) | Canonical path to the original source file | Must be an existing, readable regular file |
| `fingerprint` | `String` | SHA-256 hex digest of the raw file content | 64-character lowercase hex string |
| `lines` | `Vec<SourceLine>` | Ordered collection of source lines | May be empty (0-byte file); preserves original order |

**Serialization** (JSON output via `serde`):
```json
{
  "source_path": "/absolute/path/to/policy.md",
  "fingerprint": "a1b2c3d4e5f6...",
  "lines": [
    { "number": 1, "text": "# Access Control Policy" },
    { "number": 2, "text": "" },
    { "number": 3, "text": "All users must authenticate..." }
  ]
}
```

**Derivations**: `Debug`, `Serialize`

### SourceLine

Represents a single line from the source document. Enables traceability from any downstream artifact back to the exact source line.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `number` | `usize` | 1-based line number in the source file | >= 1 |
| `text` | `String` | Text content of the line (without trailing newline) | May be empty string for blank lines |

**Derivations**: `Debug`, `Serialize`, `PartialEq`

## Relationships

```
IngestedDocument 1 ──── * SourceLine
     │                      │
     │ source_path          │ number (1-based, ordered)
     │ fingerprint          │ text
     │ lines[]              │
```

- `IngestedDocument` owns a `Vec<SourceLine>` (composition, not reference).
- Lines are ordered by `number` (ascending, contiguous starting at 1).
- An empty file produces an `IngestedDocument` with an empty `lines` vector.

## Error Variants (ForgeError extensions)

| Variant | Fields | When Raised | User Message |
|---------|--------|-------------|--------------|
| `UnsupportedFormat` | `extension: String` | File extension is not `.md` or `.markdown` | "Unsupported file format '.{ext}'. Only Markdown files (.md, .markdown) are supported. Consider converting with pandoc or markitdown." |
| `FileTooLarge` | `path: PathBuf, size_bytes: u64, limit_bytes: u64` | File size exceeds `--max-size` limit | "File '{path}' is {size}MB, exceeding the {limit}MB limit. Use --max-size to increase the limit." |
| `InvalidEncoding` | `path: PathBuf` | File content is not valid UTF-8 | "File '{path}' is not valid UTF-8 text. FORGE requires UTF-8 encoded Markdown files." |
| `NotAFile` | `path: PathBuf` | Path refers to a directory or special file | "'{path}' is not a regular file." |

Existing `ForgeError::Io` variant covers: file not found, permission denied.

## Validation Rules

1. **Extension check** (FR-002): `Path::extension()` compared case-insensitively against `["md", "markdown"]`. Files with no extension are rejected.
2. **Regular file check** (FR-011): `fs::metadata().is_file()` must be true (rejects directories, symlinks to directories, devices).
3. **Size check** (FR-010): `fs::metadata().len()` must be <= limit (default 10MB = 10 * 1024 * 1024 bytes). Checked before reading content.
4. **UTF-8 check** (FR-007): After `fs::read()` returns raw bytes (needed for fingerprint), `String::from_utf8()` validates encoding; failure is converted to `InvalidEncoding`.
5. **Fingerprint** (FR-008): SHA-256 computed over the raw byte content of the file (before line splitting).

## State Transitions

None. Ingestion is a single-shot read operation. `IngestedDocument` is immutable after construction.

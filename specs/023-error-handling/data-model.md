# Data Model: Error Handling & Robustness (WI-23)

**Phase 1 output** | **Date**: 2026-02-13

## Entity: ForgeError (Modified)

The existing `ForgeError` enum in `src/error.rs` is extended with new variants. Existing variants are preserved unchanged.

### New Variants

| Variant | Fields | PRD Requirement | User-Facing Message |
|---------|--------|-----------------|---------------------|
| `FileNotFound` | `path: PathBuf` | M-6 | `"File not found: '{path}'"` |
| `PermissionDenied` | `path: PathBuf` | S-1 | `"Permission denied: '{path}'"` |
| `EmptyInput` | `path: PathBuf` | M-7 | `"File is empty: '{path}' — provide a non-empty Markdown policy document"` |
| `BinaryFile` | `path: PathBuf` | S-2 | `"File appears to be binary, not a text document: '{path}'. FORGE accepts UTF-8 Markdown (.md) files."` |
| `NoStructureDetected` | `path: PathBuf` | M-8 | `"No policy structure detected in '{path}' — expected Markdown headings (# Section) or numbered clauses"` |

### Existing Variants (Unchanged)

| Variant | Fields | Status |
|---------|--------|--------|
| `Io` | `#[from] std::io::Error` | Keep — fallback for I/O errors not disaggregated to specific variants |
| `Parse` | `String` | Keep — used throughout parser modules |
| `Validation` | `String` | Keep — used for validation messages |
| `Config` | `String` | Keep — used for configuration errors |
| `UnsupportedFormat` | `extension: String` | Keep — extension-based format detection |
| `FileTooLarge` | `path, size_bytes, limit_bytes` | Keep — already satisfies S-3 |
| `InvalidEncoding` | `path: PathBuf` | Keep — UTF-8 validation after binary check |
| `NotAFile` | `path: PathBuf` | Keep — already satisfies EC-7 |
| `CatalogBuild` | `String` | Keep |
| `BackMatter` | `String` | Keep |
| `ComponentDefinitionBuild` | `String` | Keep |
| `Serialization` | `String` | Keep |

### Variant Ordering (Full Enum)

```
ForgeError
├── FileNotFound { path }          # NEW — exit code 1
├── PermissionDenied { path }      # NEW — exit code 1
├── EmptyInput { path }            # NEW — exit code 1
├── BinaryFile { path }            # NEW — exit code 1
├── UnsupportedFormat { extension } # existing — exit code 1
├── FileTooLarge { path, ... }     # existing — exit code 1
├── InvalidEncoding { path }       # existing — exit code 1
├── NotAFile { path }              # existing — exit code 1
├── Io(std::io::Error)             # existing — exit code 1
├── NoStructureDetected { path }   # NEW — exit code 2
├── Parse(String)                  # existing — exit code 2
├── CatalogBuild(String)           # existing — exit code 2
├── BackMatter(String)             # existing — exit code 2
├── ComponentDefinitionBuild(String) # existing — exit code 2
├── Validation(String)             # existing — exit code 3
├── Config(String)                 # existing — exit code 3
└── Serialization(String)          # existing — exit code 1
```

## Entity: Exit Code Mapping

| Exit Code | Category | Variants |
|-----------|----------|----------|
| 0 | Success | — |
| 1 | Input/IO errors | FileNotFound, PermissionDenied, EmptyInput, BinaryFile, UnsupportedFormat, FileTooLarge, InvalidEncoding, NotAFile, Io, Serialization |
| 2 | Parse/Structure errors | NoStructureDetected, Parse, CatalogBuild, BackMatter, ComponentDefinitionBuild |
| 3 | Validation/Config errors | Validation, Config |

## Entity: Binary Detection Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `BINARY_CHECK_SAMPLE_SIZE` | 512 | Number of bytes to sample for binary detection |
| `NULL_BYTE_THRESHOLD` | 0.10 (10%) | Null byte ratio above which file is classified as binary |
| `MAGIC_BYTES_PNG` | `[0x89, 0x50, 0x4E, 0x47]` | PNG signature |
| `MAGIC_BYTES_JPEG` | `[0xFF, 0xD8, 0xFF]` | JPEG signature |
| `MAGIC_BYTES_PDF` | `[0x25, 0x50, 0x44, 0x46]` | PDF signature (`%PDF`) |
| `MAGIC_BYTES_ZIP` | `[0x50, 0x4B, 0x03, 0x04]` | ZIP/DOCX/XLSX signature |
| `MAGIC_BYTES_ELF` | `[0x7F, 0x45, 0x4C, 0x46]` | ELF binary signature |

## Relationships

```
ingest_file(path) → Result<IngestedDocument, ForgeError>
    ├── validates extension → UnsupportedFormat
    ├── checks metadata → FileNotFound | PermissionDenied | Io
    ├── checks is_file → NotAFile
    ├── checks size → FileTooLarge
    ├── reads bytes → FileNotFound | PermissionDenied | Io
    ├── checks empty → EmptyInput
    ├── checks binary → BinaryFile
    └── checks utf8 → InvalidEncoding

prepare_document(path) → Result<PolicyDocument, ForgeError>
    ├── calls ingest_file → (all ingest errors)
    ├── extract_sections → Parse
    ├── extract_clauses → Parse
    ├── checks structure → NoStructureDetected
    ├── assemble_document → Parse
    ├── atomize_document → Parse
    └── extract_citations → Parse

main() → ExitCode
    ├── cli::execute() → Result<(), ForgeError>
    └── exit_code(&ForgeError) → u8 {1, 2, 3}
```

## State Transitions

No state machines in this feature. All error handling is immediate (fail-fast, no retry).

## Validation Rules

| Rule | Enforcement |
|------|------------|
| File paths in error messages must be user-provided, not canonicalized | Use `path` parameter, not `IngestedDocument.source_path` |
| Error messages must not contain internal module names | All Display impls use domain terminology only |
| Error messages must follow "what happened + context + how to fix" pattern | Enforced by thiserror `#[error()]` attributes |
| Every ForgeError variant must map to exactly one exit code | `exit_code()` function covers all variants exhaustively |

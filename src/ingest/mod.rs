//! Markdown file ingestion module.
//!
//! Reads Markdown files from the filesystem, validates format/encoding/size,
//! computes a SHA-256 content fingerprint, and tracks 1-based line numbers
//! for downstream OSCAL conversion.

use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::ForgeError;

/// A single line from a source document, with its 1-based line number.
#[derive(Debug, Serialize, PartialEq)]
pub struct SourceLine {
    /// 1-based line number in the source file.
    pub number: usize,
    /// Text content of the line, without trailing newline.
    pub text: String,
}

/// A Markdown file that has been read and fingerprinted for downstream processing.
#[derive(Debug, Serialize)]
pub struct IngestedDocument {
    /// Canonical path to the original source file.
    pub source_path: PathBuf,
    /// SHA-256 hex digest of the raw file content.
    pub fingerprint: String,
    /// Ordered collection of source lines.
    pub lines: Vec<SourceLine>,
}

impl IngestedDocument {
    /// Reconstruct the full document content by joining all source lines.
    #[must_use]
    pub fn reconstruct_content(&self) -> String {
        self.lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n")
    }
}

const MAGIC_BYTES: &[(&[u8], &str)] = &[
    (&[0x89, 0x50, 0x4E, 0x47], "PNG"),
    (&[0xFF, 0xD8, 0xFF], "JPEG"),
    (&[0x25, 0x50, 0x44, 0x46], "PDF"),
    (&[0x50, 0x4B, 0x03, 0x04], "ZIP"),
    (&[0x7F, 0x45, 0x4C, 0x46], "ELF"),
];
const BINARY_CHECK_SAMPLE_SIZE: usize = 512;
const NULL_BYTE_THRESHOLD: f64 = 0.10;

#[allow(clippy::cast_precision_loss, clippy::naive_bytecount)]
fn is_binary_content(bytes: &[u8]) -> bool {
    for (signature, _name) in MAGIC_BYTES {
        if bytes.starts_with(signature) {
            return true;
        }
    }
    let sample_size = bytes.len().min(BINARY_CHECK_SAMPLE_SIZE);
    if sample_size == 0 {
        return false;
    }
    let null_count = bytes[..sample_size].iter().filter(|&&b| b == 0).count();
    let ratio = null_count as f64 / sample_size as f64;
    ratio > NULL_BYTE_THRESHOLD
}

/// Read a Markdown file, validate it, and produce an [`IngestedDocument`].
///
/// # Arguments
///
/// * `path` - Path to the Markdown file (`.md` or `.markdown`, case-insensitive).
/// * `max_size_bytes` - Maximum allowed file size in bytes.
///
/// # Errors
///
/// Returns [`ForgeError::UnsupportedFormat`] if the file extension is not `.md` or `.markdown`.
/// Returns [`ForgeError::NotAFile`] if the path is not a regular file.
/// Returns [`ForgeError::FileTooLarge`] if the file exceeds `max_size_bytes`.
/// Returns [`ForgeError::InvalidEncoding`] if the file is not valid UTF-8.
/// Returns [`ForgeError::FileNotFound`] if the file does not exist.
/// Returns [`ForgeError::PermissionDenied`] if the file cannot be read due to permissions.
/// Returns [`ForgeError::Io`] for other filesystem errors.
pub fn ingest_file(path: &Path, max_size_bytes: u64) -> Result<IngestedDocument, ForgeError> {
    // Extension validation (must execute before any file I/O)
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown") {
        return Err(ForgeError::UnsupportedFormat { extension: ext.to_string() });
    }

    // Metadata checks: regular file + size limit
    let metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: path.to_path_buf() }
        }
        _ => ForgeError::Io(e),
    })?;
    if !metadata.is_file() {
        return Err(ForgeError::NotAFile { path: path.to_path_buf() });
    }
    if metadata.len() > max_size_bytes {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: metadata.len(),
            limit_bytes: max_size_bytes,
        });
    }

    let bytes = std::fs::read(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: path.to_path_buf() }
        }
        _ => ForgeError::Io(e),
    })?;

    if bytes.is_empty() {
        return Err(ForgeError::EmptyInput { path: path.to_path_buf() });
    }

    if is_binary_content(&bytes) {
        return Err(ForgeError::BinaryFile { path: path.to_path_buf() });
    }

    let fingerprint = format!("{:x}", Sha256::digest(&bytes));

    let content = String::from_utf8(bytes)
        .map_err(|_| ForgeError::InvalidEncoding { path: path.to_path_buf() })?;

    let lines: Vec<SourceLine> = content
        .lines()
        .enumerate()
        .map(|(i, text)| SourceLine { number: i + 1, text: text.to_string() })
        .collect();

    let source_path = path.canonicalize()?;

    Ok(IngestedDocument { source_path, fingerprint, lines })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_temp_md(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn valid_md_returns_ingested_document_with_correct_source_path() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.md", "# Title\n\nSome content.\n");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        let canonical = path.canonicalize().unwrap();
        assert_eq!(doc.source_path, canonical);
    }

    #[test]
    fn fingerprint_is_64_char_lowercase_hex_sha256() {
        use sha2::{Digest, Sha256};

        let dir = TempDir::new().unwrap();
        let content = "hello world\n";
        let path = create_temp_md(&dir, "test.md", content);
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();

        assert_eq!(doc.fingerprint.len(), 64);
        assert!(
            doc.fingerprint.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "Fingerprint must be lowercase hex: {}",
            doc.fingerprint
        );

        // Independently compute expected SHA-256
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let expected = format!("{:x}", hasher.finalize());
        assert_eq!(doc.fingerprint, expected);
    }

    #[test]
    fn lines_have_correct_count() {
        let dir = TempDir::new().unwrap();
        let content = "line one\nline two\nline three";
        let path = create_temp_md(&dir, "test.md", content);
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines.len(), 3);
    }

    #[test]
    fn lines_have_1_based_numbering_and_correct_text() {
        let dir = TempDir::new().unwrap();
        let content = "first\nsecond\nthird";
        let path = create_temp_md(&dir, "test.md", content);
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();

        assert_eq!(
            doc.lines,
            vec![
                SourceLine { number: 1, text: "first".to_string() },
                SourceLine { number: 2, text: "second".to_string() },
                SourceLine { number: 3, text: "third".to_string() },
            ]
        );
    }

    #[test]
    fn empty_file_returns_empty_input_error() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "empty.md", "");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match &err {
            ForgeError::EmptyInput { path: err_path } => {
                assert_eq!(err_path, &path);
            }
            other => panic!("Expected ForgeError::EmptyInput, got: {other:?}"),
        }
    }

    #[test]
    fn same_content_produces_same_fingerprint() {
        let dir = TempDir::new().unwrap();
        let content = "deterministic content\n";
        let path1 = create_temp_md(&dir, "a.md", content);
        let path2 = create_temp_md(&dir, "b.md", content);

        let doc1 = ingest_file(&path1, 10 * 1_048_576).unwrap();
        let doc2 = ingest_file(&path2, 10 * 1_048_576).unwrap();
        assert_eq!(doc1.fingerprint, doc2.fingerprint);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_to_valid_md_is_followed() {
        let dir = TempDir::new().unwrap();
        let target = create_temp_md(&dir, "real.md", "symlinked content\n");
        let link = dir.path().join("link.md");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let doc = ingest_file(&link, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].text, "symlinked content");
    }

    #[test]
    fn file_path_with_spaces_is_handled() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "my policy.md", "spaced path content\n");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].text, "spaced path content");
    }

    // --- US2: Extension validation tests ---

    #[test]
    fn pdf_file_returns_unsupported_format() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.pdf", "fake pdf");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match err {
            ForgeError::UnsupportedFormat { extension } => assert_eq!(extension, "pdf"),
            other => panic!("Expected UnsupportedFormat, got: {other:?}"),
        }
    }

    #[test]
    fn docx_file_returns_unsupported_format() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.docx", "fake docx");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        assert!(matches!(err, ForgeError::UnsupportedFormat { .. }));
    }

    #[test]
    fn no_extension_returns_unsupported_format() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy", "no extension");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match err {
            ForgeError::UnsupportedFormat { extension } => assert_eq!(extension, ""),
            other => panic!("Expected UnsupportedFormat, got: {other:?}"),
        }
    }

    #[test]
    fn uppercase_md_extension_is_accepted() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.MD", "uppercase md");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines[0].text, "uppercase md");
    }

    #[test]
    fn mixed_case_markdown_extension_is_accepted() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.Markdown", "mixed case");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines[0].text, "mixed case");
    }

    #[test]
    fn uppercase_markdown_extension_is_accepted() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.MARKDOWN", "uppercase markdown");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines[0].text, "uppercase markdown");
    }

    #[test]
    fn lowercase_markdown_extension_is_accepted() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "policy.markdown", "lowercase markdown");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        assert_eq!(doc.lines[0].text, "lowercase markdown");
    }

    // --- US1: Missing or unreadable input file tests ---

    #[test]
    fn nonexistent_file_returns_file_not_found() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.md");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match &err {
            ForgeError::FileNotFound { path: err_path } => {
                assert_eq!(err_path, &path);
            }
            other => panic!("Expected ForgeError::FileNotFound, got: {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_file_returns_permission_denied() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "unreadable.md", "secret content");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        // Restore permissions for cleanup
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        match &err {
            ForgeError::PermissionDenied { path: err_path } => {
                assert_eq!(err_path, &path);
            }
            other => panic!("Expected ForgeError::PermissionDenied, got: {other:?}"),
        }
    }

    // --- US3: File access error tests (legacy, updated for US1) ---

    #[test]
    fn nonexistent_file_returns_io_not_found() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.md");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match &err {
            ForgeError::FileNotFound { path: err_path } => assert_eq!(err_path, &path),
            other => panic!("Expected ForgeError::FileNotFound, got: {other:?}"),
        }
    }

    #[test]
    fn directory_path_returns_not_a_file() {
        let dir = TempDir::new().unwrap();
        // Create a directory with .md extension to pass extension check
        let dir_path = dir.path().join("subdir.md");
        fs::create_dir(&dir_path).unwrap();
        let err = ingest_file(&dir_path, 10 * 1_048_576).unwrap_err();
        assert!(matches!(err, ForgeError::NotAFile { .. }));
    }

    #[test]
    fn non_utf8_file_returns_invalid_encoding() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("binary.md");
        fs::write(&path, [0xFF, 0xFE]).unwrap();
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        assert!(matches!(err, ForgeError::InvalidEncoding { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn permission_denied_returns_io_error() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "restricted.md", "secret");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        // Restore permissions for cleanup
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        match &err {
            ForgeError::PermissionDenied { path: err_path } => {
                assert_eq!(err_path, &path);
            }
            other => panic!("Expected ForgeError::PermissionDenied, got: {other:?}"),
        }
    }

    // --- US4: File size validation tests ---

    #[test]
    fn file_exceeding_max_size_returns_file_too_large() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("big.md");
        // Write 1025 bytes, set limit to 1024
        let content = "x".repeat(1025);
        fs::write(&path, &content).unwrap();
        let err = ingest_file(&path, 1024).unwrap_err();
        match err {
            ForgeError::FileTooLarge { size_bytes, limit_bytes, .. } => {
                assert_eq!(size_bytes, 1025);
                assert_eq!(limit_bytes, 1024);
            }
            other => panic!("Expected FileTooLarge, got: {other:?}"),
        }
    }

    #[test]
    fn file_exactly_at_limit_is_accepted() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("exact.md");
        let content = "x".repeat(1024);
        fs::write(&path, &content).unwrap();
        let doc = ingest_file(&path, 1024).unwrap();
        assert_eq!(doc.lines.len(), 1);
    }

    #[test]
    fn file_within_custom_higher_limit_is_accepted() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("medium.md");
        let content = "x".repeat(2048);
        fs::write(&path, &content).unwrap();
        let doc = ingest_file(&path, 4096).unwrap();
        assert_eq!(doc.lines.len(), 1);
    }

    // --- US2: Binary detection tests (T011) ---

    #[test]
    fn is_binary_detects_png_magic_bytes() {
        let mut bytes = vec![0x89, 0x50, 0x4E, 0x47];
        bytes.extend_from_slice(b"rest of file");
        assert!(is_binary_content(&bytes));
    }

    #[test]
    fn is_binary_detects_jpeg_magic_bytes() {
        let mut bytes = vec![0xFF, 0xD8, 0xFF];
        bytes.extend_from_slice(b"rest of file");
        assert!(is_binary_content(&bytes));
    }

    #[test]
    fn is_binary_detects_pdf_magic_bytes() {
        let mut bytes = vec![0x25, 0x50, 0x44, 0x46];
        bytes.extend_from_slice(b"rest of file");
        assert!(is_binary_content(&bytes));
    }

    #[test]
    fn is_binary_detects_zip_magic_bytes() {
        let mut bytes = vec![0x50, 0x4B, 0x03, 0x04];
        bytes.extend_from_slice(b"rest of file");
        assert!(is_binary_content(&bytes));
    }

    #[test]
    fn is_binary_detects_elf_magic_bytes() {
        let mut bytes = vec![0x7F, 0x45, 0x4C, 0x46];
        bytes.extend_from_slice(b"rest of file");
        assert!(is_binary_content(&bytes));
    }

    #[test]
    fn is_binary_detects_high_null_byte_ratio() {
        // 60 null bytes out of 100 = 60% > 10% threshold
        let mut bytes = vec![0u8; 60];
        bytes.extend_from_slice(&[b'a'; 40]);
        assert!(is_binary_content(&bytes));
    }

    #[test]
    fn is_binary_rejects_clean_ascii_text() {
        let bytes = b"This is clean ASCII text with no binary content.\n";
        assert!(!is_binary_content(bytes));
    }

    #[test]
    fn is_binary_rejects_empty_bytes() {
        assert!(!is_binary_content(&[]));
    }

    // --- US2: Empty file ingest test (T012) ---

    #[test]
    fn ingest_empty_file_returns_empty_input() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "blank.md", "");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match &err {
            ForgeError::EmptyInput { path: err_path } => {
                assert_eq!(err_path, &path);
            }
            other => panic!("Expected ForgeError::EmptyInput, got: {other:?}"),
        }
    }

    // --- US2: Binary file ingest test (T013) ---

    #[test]
    fn ingest_binary_file_returns_binary_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("image.md");
        // Write PNG magic bytes as content
        let mut bytes = vec![0x89, 0x50, 0x4E, 0x47];
        bytes.extend_from_slice(b"not real png data");
        fs::write(&path, &bytes).unwrap();
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match &err {
            ForgeError::BinaryFile { path: err_path } => {
                assert_eq!(err_path, &path);
            }
            other => panic!("Expected ForgeError::BinaryFile, got: {other:?}"),
        }
    }
}

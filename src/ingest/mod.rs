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

/// Ingests a Markdown file: validates extension, size, and UTF-8 encoding, then returns a canonicalized document with a SHA-256 fingerprint and 1-based source lines.
///
/// On success returns an `IngestedDocument` containing the canonical `source_path`, a lowercase hex SHA-256 `fingerprint` of the raw file bytes, and `lines` where each `SourceLine` has a 1-based `number` and the line `text` (no trailing newline).
///
/// # Errors
///
/// - [`ForgeError::UnsupportedFormat`] if the file extension is not `.md` or `.markdown` (case-insensitive).
/// - [`ForgeError::NotAFile`] if `path` does not refer to a regular file.
/// - [`ForgeError::FileTooLarge`] if the file size exceeds `max_size_bytes`.
/// - [`ForgeError::InvalidEncoding`] if the file bytes are not valid UTF-8.
/// - [`ForgeError::Io`] for filesystem I/O errors (e.g., not found, permission denied, or failure to canonicalize).
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// // `example.md` should exist for this example to run in a real environment.
/// let doc = ingest_file(Path::new("example.md"), 10_000).unwrap();
/// assert!(!doc.fingerprint.is_empty());
/// ```
pub fn ingest_file(path: &Path, max_size_bytes: u64) -> Result<IngestedDocument, ForgeError> {
    // Extension validation (must execute before any file I/O)
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext.to_ascii_lowercase().as_str(), "md" | "markdown") {
        return Err(ForgeError::UnsupportedFormat { extension: ext.to_string() });
    }

    // Metadata checks: regular file + size limit
    let metadata = std::fs::metadata(path)?;
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

    let bytes = std::fs::read(path)?;

    let content = String::from_utf8(bytes.clone())
        .map_err(|_| ForgeError::InvalidEncoding { path: path.to_path_buf() })?;

    let fingerprint = format!("{:x}", Sha256::digest(&bytes));

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

    /// Creates a file named `name` containing `content` inside the given temporary directory.
    ///
    /// # Examples
    ///
    /// ```
    /// use tempfile::tempdir;
    /// use std::fs;
    /// let dir = tempdir().unwrap();
    /// let path = create_temp_md(&dir, "example.md", "# Hello\n");
    /// assert!(path.exists());
    /// let txt = fs::read_to_string(path).unwrap();
    /// assert_eq!(txt, "# Hello\n");
    /// ```
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
        use sha2::{Digest, Sha256};
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
    fn empty_file_returns_empty_lines_and_known_hash() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_md(&dir, "empty.md", "");
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();

        assert!(doc.lines.is_empty());
        assert_eq!(
            doc.fingerprint,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
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

    // --- US3: File access error tests ---

    #[test]
    fn nonexistent_file_returns_io_not_found() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("missing.md");
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        match &err {
            ForgeError::Io(io_err) => assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound),
            other => panic!("Expected ForgeError::Io(NotFound), got: {other:?}"),
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
        fs::write(&path, &[0xFF, 0xFE]).unwrap();
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
            ForgeError::Io(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::PermissionDenied)
            }
            other => panic!("Expected ForgeError::Io(PermissionDenied), got: {other:?}"),
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
}
//! Policy document ingestion module.
//!
//! Reads Markdown, PDF, and DOCX files from the filesystem, validates format/encoding/size,
//! computes a SHA-256 content fingerprint, and tracks 1-based line numbers
//! for downstream OSCAL conversion.

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use quick_xml::Reader;
use quick_xml::events::Event as XmlEvent;
use serde::Serialize;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::ForgeError;

/// A single line from a source document, with its 1-based line number.
#[derive(Debug, Serialize, PartialEq)]
pub struct SourceLine {
    /// 1-based line number in the source file.
    pub number: usize,
    /// Text content of the line, without trailing newline.
    pub text: String,
}

/// A policy document that has been read and fingerprinted for downstream processing.
#[derive(Debug, Serialize)]
pub struct IngestedDocument {
    /// Canonical path to the original source file.
    pub source_path: PathBuf,
    /// SHA-256 hex digest of the raw file content.
    pub fingerprint: String,
    /// Ordered collection of source lines.
    pub lines: Vec<SourceLine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputFormat {
    Markdown,
    Pdf,
    Docx,
}

impl IngestedDocument {
    /// Reconstruct the full document content by joining all source lines.
    ///
    /// This normalizes line endings to LF and does not restore a trailing newline;
    /// [`Self::fingerprint`] instead covers the original input bytes.
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

/// Read a policy file, validate it, and produce an [`IngestedDocument`].
///
/// # Arguments
///
/// * `path` - Path to the policy file (`.md`, `.markdown`, `.pdf`, or `.docx`, case-insensitive).
/// * `max_size_bytes` - Maximum allowed file size in bytes.
///
/// # Errors
///
/// Returns [`ForgeError::UnsupportedFormat`] if the file extension is unsupported.
/// Returns [`ForgeError::NotAFile`] if the path is not a regular file.
/// Returns [`ForgeError::FileTooLarge`] if the file exceeds `max_size_bytes`.
/// Returns [`ForgeError::InvalidEncoding`] if the file is not valid UTF-8.
/// Returns [`ForgeError::FileNotFound`] if the file does not exist.
/// Returns [`ForgeError::PermissionDenied`] if the file cannot be read due to permissions.
/// Returns [`ForgeError::Io`] for other filesystem errors.
pub fn ingest_file(path: &Path, max_size_bytes: u64) -> Result<IngestedDocument, ForgeError> {
    // Extension validation (must execute before any file I/O)
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let format = detect_format(ext)?;

    // Metadata checks: regular file + size limit
    let metadata = std::fs::metadata(path).map_err(map_io_error(path))?;
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

    let bytes = std::fs::read(path).map_err(map_io_error(path))?;

    if bytes.is_empty() {
        return Err(ForgeError::EmptyInput { path: path.to_path_buf() });
    }

    if format == InputFormat::Markdown && is_binary_content(&bytes) {
        return Err(ForgeError::BinaryFile { path: path.to_path_buf() });
    }

    let fingerprint = format!("{:x}", Sha256::digest(&bytes));

    let content = match format {
        InputFormat::Markdown => String::from_utf8(bytes)
            .map_err(|_| ForgeError::InvalidEncoding { path: path.to_path_buf() })?,
        InputFormat::Pdf => extract_pdf_content(path)?,
        InputFormat::Docx => extract_docx_content(path, &bytes, max_size_bytes)?,
    };

    let lines: Vec<SourceLine> = content
        .lines()
        .enumerate()
        .map(|(i, text)| SourceLine { number: i + 1, text: text.to_string() })
        .collect();

    let source_path = path.canonicalize().map_err(map_io_error(path))?;

    Ok(IngestedDocument { source_path, fingerprint, lines })
}

fn detect_format(extension: &str) -> Result<InputFormat, ForgeError> {
    match extension.to_ascii_lowercase().as_str() {
        "md" | "markdown" => Ok(InputFormat::Markdown),
        "pdf" => Ok(InputFormat::Pdf),
        "docx" => Ok(InputFormat::Docx),
        _ => Err(ForgeError::UnsupportedFormat { extension: extension.to_string() }),
    }
}

fn map_io_error(path: &Path) -> impl Fn(std::io::Error) -> ForgeError + '_ {
    move |error| match error.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: path.to_path_buf() }
        }
        _ => ForgeError::Io(error),
    }
}

fn extract_pdf_content(path: &Path) -> Result<String, ForgeError> {
    let extracted = pdf_extract::extract_text(path)
        .map_err(|e| ForgeError::Parse(format!("failed to extract PDF text: {e}")))?;
    if extracted.trim().is_empty() {
        return Err(ForgeError::OcrNotSupported { path: path.to_path_buf() });
    }
    Ok(markdownize_extracted_text(&extracted))
}

fn extract_docx_content(
    path: &Path,
    bytes: &[u8],
    max_size_bytes: u64,
) -> Result<String, ForgeError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| ForgeError::Parse(format!("failed to open DOCX archive: {e}")))?;
    let entry = archive
        .by_name("word/document.xml")
        .map_err(|e| ForgeError::Parse(format!("DOCX missing word/document.xml: {e}")))?;
    let cap = max_size_bytes.saturating_mul(64);
    let mut document_xml = Vec::new();
    entry
        .take(cap.saturating_add(1))
        .read_to_end(&mut document_xml)
        .map_err(|e| ForgeError::Parse(format!("failed to read DOCX document.xml: {e}")))?;
    if u64::try_from(document_xml.len()).is_ok_and(|len| len > cap) {
        return Err(ForgeError::Parse(
            "DOCX word/document.xml exceeds decompression budget".to_owned(),
        ));
    }
    let document_xml = String::from_utf8(document_xml)
        .map_err(|_| ForgeError::Parse("DOCX word/document.xml is not valid UTF-8".to_owned()))?;

    let extracted = extract_docx_document_xml(&document_xml)?;
    if extracted.trim().is_empty() {
        return Err(ForgeError::EmptyInput { path: path.to_path_buf() });
    }
    Ok(extracted)
}

fn extract_docx_document_xml(xml: &str) -> Result<String, ForgeError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut output = Vec::new();
    let mut paragraph = String::new();
    let mut style: Option<String> = None;
    let mut in_paragraph = false;
    let mut in_table = false;
    let mut in_row = false;
    let mut in_cell = false;
    let mut cell = String::new();
    let mut row = Vec::new();
    let mut is_list_item = false;

    loop {
        match reader.read_event() {
            Ok(XmlEvent::Start(element)) => match element.name().as_ref() {
                b"w:p" => {
                    in_paragraph = true;
                    paragraph.clear();
                    style = None;
                    is_list_item = false;
                }
                b"w:tbl" => in_table = true,
                b"w:tr" => {
                    in_row = true;
                    row.clear();
                }
                b"w:tc" => {
                    in_cell = true;
                    cell.clear();
                }
                b"w:numPr" if in_paragraph => is_list_item = true,
                b"w:pStyle" if in_paragraph => {
                    for attr in element.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            style = Some(String::from_utf8_lossy(attr.value.as_ref()).into_owned());
                        }
                    }
                }
                _ => {}
            },
            Ok(XmlEvent::Empty(element)) => match element.name().as_ref() {
                b"w:numPr" if in_paragraph => is_list_item = true,
                b"w:pStyle" if in_paragraph => {
                    for attr in element.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            style = Some(String::from_utf8_lossy(attr.value.as_ref()).into_owned());
                        }
                    }
                }
                _ => {}
            },
            Ok(XmlEvent::Text(text)) => {
                let decoded = String::from_utf8_lossy(text.as_ref());
                if in_cell {
                    cell.push_str(&decoded);
                } else if in_paragraph {
                    paragraph.push_str(&decoded);
                }
            }
            Ok(XmlEvent::End(element)) => match element.name().as_ref() {
                b"w:p" => {
                    let text = paragraph.trim();
                    if !text.is_empty() && !in_table {
                        output.push(format_docx_paragraph(text, style.as_deref(), is_list_item));
                    }
                    in_paragraph = false;
                }
                b"w:tc" => {
                    if in_row {
                        row.push(cell.trim().to_string());
                    }
                    in_cell = false;
                }
                b"w:tr" => {
                    if !row.is_empty() {
                        output.push(format!("| {} |", row.join(" | ")));
                    }
                    in_row = false;
                }
                b"w:tbl" => in_table = false,
                _ => {}
            },
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(ForgeError::Parse(format!("failed to parse DOCX XML: {e}"))),
            _ => {}
        }
    }

    Ok(output.join("\n"))
}

fn format_docx_paragraph(text: &str, style: Option<&str>, is_list_item: bool) -> String {
    if let Some(style) = style
        && let Some(level) = heading_level_from_docx_style(style)
    {
        return format!("{} {text}", "#".repeat(level));
    }
    if is_list_item { format!("- {text}") } else { text.to_string() }
}

fn heading_level_from_docx_style(style: &str) -> Option<usize> {
    let normalized = style.replace(' ', "").to_ascii_lowercase();
    normalized
        .strip_prefix("heading")
        .and_then(|level| level.parse::<usize>().ok())
        .filter(|level| (1..=6).contains(level))
}

/// Convert extracted document text to the limited Markdown consumed by the parser.
///
/// This heuristic is irreversible: it promotes the first non-empty line and
/// heading-like lines, and logs each rewrite at debug level for auditability.
fn markdownize_extracted_text(text: &str) -> String {
    let mut output = Vec::new();
    let mut emitted_title = false;
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if !emitted_title {
            tracing::debug!(rewritten_line = line, "promoting extracted line to Markdown title");
            output.push(format!("# {line}"));
            emitted_title = true;
        } else if looks_like_heading(line) {
            tracing::debug!(rewritten_line = line, "promoting extracted line to Markdown heading");
            output.push(format!("## {line}"));
        } else {
            output.push(line.to_string());
        }
    }
    output.join("\n")
}

fn looks_like_heading(line: &str) -> bool {
    let word_count = line.split_whitespace().count();
    word_count <= 8
        && !line.ends_with('.')
        && !line.ends_with(';')
        && !line.contains(" must ")
        && !line.contains(" shall ")
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

    fn create_temp_bytes(dir: &TempDir, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    fn create_docx(dir: &TempDir, document_xml: &str) -> PathBuf {
        let path = dir.path().join("policy.docx");
        let file = fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("word/document.xml", options).unwrap();
        zip.write_all(document_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
        path
    }

    fn minimal_blank_pdf() -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let objects = [
            b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".as_slice(),
            b"2 0 obj\n<< /Type /Pages /Count 0 /Kids [] >>\nendobj\n".as_slice(),
        ];
        let mut offsets = Vec::new();
        for object in objects {
            offsets.push(pdf.len());
            pdf.extend_from_slice(object);
        }
        let xref_offset = pdf.len();
        pdf.extend_from_slice(b"xref\n0 3\n0000000000 65535 f \n");
        for offset in offsets {
            pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!("trailer\n<< /Size 3 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n")
                .as_bytes(),
        );
        pdf
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

    #[cfg(windows)]
    #[test]
    fn symlink_to_valid_md_is_followed() {
        let dir = TempDir::new().unwrap();
        let target = create_temp_md(&dir, "real.md", "symlinked content\n");
        let link = dir.path().join("link.md");
        std::os::windows::fs::symlink_file(&target, &link).unwrap();

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
    fn pdf_with_no_extractable_text_returns_ocr_not_supported() {
        let dir = TempDir::new().unwrap();
        let path = create_temp_bytes(&dir, "policy.pdf", &minimal_blank_pdf());
        let err = ingest_file(&path, 10 * 1_048_576).unwrap_err();
        assert!(matches!(err, ForgeError::OcrNotSupported { .. }), "got: {err:?}");
    }

    #[test]
    fn docx_extracts_headings_lists_and_tables() {
        let dir = TempDir::new().unwrap();
        let path = create_docx(
            &dir,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>
  <w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Access Control</w:t></w:r></w:p>
  <w:p><w:pPr><w:numPr><w:ilvl w:val="0"/></w:numPr></w:pPr><w:r><w:t>Users must authenticate with MFA.</w:t></w:r></w:p>
  <w:tbl><w:tr><w:tc><w:p><w:r><w:t>Role</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Requirement</w:t></w:r></w:p></w:tc></w:tr></w:tbl>
</w:body></w:document>"#,
        );
        let doc = ingest_file(&path, 10 * 1_048_576).unwrap();
        let content = doc.reconstruct_content();
        assert!(content.contains("# Access Control"));
        assert!(content.contains("- Users must authenticate with MFA."));
        assert!(content.contains("| Role | Requirement |"));
    }

    #[test]
    fn docx_document_xml_exceeding_decompression_budget_is_rejected() {
        let dir = TempDir::new().unwrap();
        let max_size_bytes = 1_024;
        let budget = max_size_bytes * 64;
        let document_xml = "x".repeat(usize::try_from(budget + 1).expect("test budget fits usize"));
        let path = create_docx(&dir, &document_xml);

        assert!(fs::metadata(&path).unwrap().len() <= max_size_bytes);
        let err = ingest_file(&path, max_size_bytes).unwrap_err();

        assert!(matches!(
            err,
            ForgeError::Parse(ref message)
                if message == "DOCX word/document.xml exceeds decompression budget"
        ));
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

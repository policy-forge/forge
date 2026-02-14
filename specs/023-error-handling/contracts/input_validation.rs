// Contract: Input validation additions to ingest_file() (WI-23)
// These checks are added to the existing ingest_file() function,
// not a separate function. They extend the validation chain.

use std::path::Path;

use crate::error::ForgeError;

/// Known binary file magic byte signatures.
/// Checked before null-byte heuristic for reliable detection (SEC-8).
const MAGIC_BYTES: &[(&[u8], &str)] = &[
    (&[0x89, 0x50, 0x4E, 0x47], "PNG"),      // PNG image
    (&[0xFF, 0xD8, 0xFF], "JPEG"),             // JPEG image
    (&[0x25, 0x50, 0x44, 0x46], "PDF"),        // PDF document
    (&[0x50, 0x4B, 0x03, 0x04], "ZIP"),        // ZIP/DOCX/XLSX
    (&[0x7F, 0x45, 0x4C, 0x46], "ELF"),        // ELF binary
];

/// Number of bytes to sample for null-byte binary detection heuristic.
const BINARY_CHECK_SAMPLE_SIZE: usize = 512;

/// Null byte ratio threshold above which a file is classified as binary.
const NULL_BYTE_THRESHOLD: f64 = 0.10;

/// Check if raw bytes appear to be binary content.
///
/// Two-tier detection:
/// 1. Check for known binary file signatures (magic bytes)
/// 2. If no signature match, check null byte ratio in first 512 bytes
///
/// # Returns
/// `true` if the content appears to be binary, `false` if it appears to be text.
fn is_binary_content(bytes: &[u8]) -> bool {
    // Tier 1: Magic byte signatures
    for (signature, _name) in MAGIC_BYTES {
        if bytes.starts_with(signature) {
            return true;
        }
    }

    // Tier 2: Null byte ratio heuristic
    let sample_size = bytes.len().min(BINARY_CHECK_SAMPLE_SIZE);
    if sample_size == 0 {
        return false; // Empty file is not binary
    }

    let null_count = bytes[..sample_size].iter().filter(|&&b| b == 0).count();
    let ratio = null_count as f64 / sample_size as f64;

    ratio > NULL_BYTE_THRESHOLD
}

// The following checks are added to ingest_file() in this order,
// after reading raw bytes and before UTF-8 conversion:
//
// 1. IoError disaggregation (metadata + read):
//    std::fs::metadata(path).map_err(|e| match e.kind() {
//        ErrorKind::NotFound => ForgeError::FileNotFound { path: path.into() },
//        ErrorKind::PermissionDenied => ForgeError::PermissionDenied { path: path.into() },
//        _ => ForgeError::Io(e),
//    })?;
//
// 2. Empty file check (after reading bytes):
//    if bytes.is_empty() {
//        return Err(ForgeError::EmptyInput { path: path.into() });
//    }
//
// 3. Binary content check (before UTF-8 conversion):
//    if is_binary_content(&bytes) {
//        return Err(ForgeError::BinaryFile { path: path.into() });
//    }
//
// 4. Existing checks continue: UTF-8 → InvalidEncoding

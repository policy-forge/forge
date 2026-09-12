//! Shared closed-contract helpers for the local suggestion pipeline.
//!
//! Every contract in this family parses through one path: a raw byte cap, then
//! `json_strict::parse_value` (duplicate keys, depth and string bounds), then a
//! null rejection, then a closed serde decode. Runtime validation is
//! authoritative over the published JSON Schema in `schemas/`.

use std::path::Path;

use serde_json::Value;

use crate::ForgeError;
use crate::json_strict;

/// Maximum bytes in one declared free-text or single-line string.
pub(super) const MAX_STRING_BYTES: usize = 16 * 1024;
/// Maximum bytes in one stable key.
pub(super) const MAX_KEY_BYTES: usize = 64;
/// Maximum bytes in one human label.
pub(super) const MAX_LABEL_BYTES: usize = 256;
/// Maximum bytes in one portable relative artifact path.
pub(super) const MAX_PATH_BYTES: usize = 1024;

pub(super) fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// Resolve the directory a document lives in.
///
/// A bare filename has an empty parent, which is the working directory; the
/// empty-parent fallback is what makes `--request request.json` work in both
/// editions of the path handling.
///
/// # Errors
/// Returns an authoring error when the directory cannot be resolved.
pub(super) fn document_root(path: &Path, flag: &str) -> Result<std::path::PathBuf, ForgeError> {
    let base = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::canonicalize(base)
        .map_err(|cause| error(format!("cannot resolve the {flag} directory: {cause}")))
}

/// Reject `null` anywhere in a decoded document: absence is omission.
pub(super) fn reject_nulls(value: &Value, path: &str) -> Result<(), ForgeError> {
    match value {
        Value::Null => Err(error(format!("{path} must not be null; omit the key instead"))),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_nulls(item, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        Value::Object(entries) => {
            for (key, item) in entries {
                reject_nulls(item, &format!("{path}.{}", json_strict::bounded(key)))?;
            }
            Ok(())
        }
        Value::Bool(_) | Value::Number(_) | Value::String(_) => Ok(()),
    }
}

/// Lowercase ASCII kebab-case identifier, at most [`MAX_KEY_BYTES`].
pub(super) fn key(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > MAX_KEY_BYTES
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(error(format!(
            "{name} must be lowercase ASCII kebab-case, at most {MAX_KEY_BYTES} bytes"
        )));
    }
    Ok(())
}

/// One non-empty line with no control characters or line separators.
pub(super) fn single_line(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > MAX_STRING_BYTES
        || value.trim() != value
        || value.chars().any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err(error(format!("{name} must be one non-empty line")));
    }
    Ok(())
}

/// Free text: non-empty, no control characters except newline and tab, no
/// bidirectional overrides.
pub(super) fn text(name: &str, value: &str) -> Result<(), ForgeError> {
    if !has_nonblank_text(value)
        || value.len() > MAX_STRING_BYTES
        || value.chars().any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
        || value.chars().any(|ch| {
            matches!(
                ch,
                '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
            )
        })
    {
        return Err(error(format!(
            "{name} must be non-blank text of at most {MAX_STRING_BYTES} bytes"
        )));
    }
    Ok(())
}

/// One human label: a single line that is not a rooted local path or file URI.
pub(super) fn label(name: &str, value: &str) -> Result<(), ForgeError> {
    single_line(name, value)?;
    if value.len() > MAX_LABEL_BYTES {
        return Err(error(format!("{name} must be at most {MAX_LABEL_BYTES} bytes")));
    }
    let bytes = value.as_bytes();
    let rooted_drive = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\');
    let file_uri = value.get(..5).is_some_and(|prefix| prefix.eq_ignore_ascii_case("file:"));
    if value.starts_with(['/', '\\']) || file_uri || rooted_drive {
        return Err(error(format!("{name} must not be a rooted local path")));
    }
    Ok(())
}

/// Lowercase hexadecimal SHA-256 digest.
pub(super) fn sha256(name: &str, value: &str) -> Result<(), ForgeError> {
    json_strict::validate_lowercase_sha256(name, value).map_err(error)
}

/// Canonical lowercase hyphenated UUID.
pub(super) fn uuid(name: &str, value: &str) -> Result<(), ForgeError> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 36
        && [8, 13, 18, 23].iter().all(|&index| bytes[index] == b'-')
        && bytes.iter().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                *byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)
            }
        });
    if !valid {
        return Err(error(format!("{name} must be a lowercase hyphenated UUID")));
    }
    Ok(())
}

/// Portable relative artifact path, validated as authoring already validates one.
pub(super) fn relative_path(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.len() > MAX_PATH_BYTES {
        return Err(error(format!("{name} must be at most {MAX_PATH_BYTES} bytes")));
    }
    crate::authoring::manifest::validate_local_path(name, Path::new(value))
}

/// Non-empty list of bounded free-text entries with no duplicates.
pub(super) fn text_list(name: &str, values: &[String], limit: usize) -> Result<(), ForgeError> {
    if values.len() > limit {
        return Err(error(format!("{name} exceeds {limit} entries")));
    }
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        text(name, value)?;
        if !seen.insert(value.as_str()) {
            return Err(error(format!("{name} entries must be unique")));
        }
    }
    Ok(())
}

fn has_nonblank_text(value: &str) -> bool {
    value.chars().any(|ch| !ch.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_reject_uppercase_separators_and_trailing_hyphens() {
        for valid in ["mapping-candidates", "unit-1", "a", "u-0001"] {
            assert!(key("key", valid).is_ok(), "{valid}");
        }
        for invalid in ["", "Upper", "under_score", "trailing-", "double--hyphen", "1leading"] {
            assert!(key("key", invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn single_line_rejects_newlines_and_padded_text() {
        assert!(single_line("field", "one line").is_ok());
        assert!(single_line("field", "two\nlines").is_err());
        assert!(single_line("field", " padded").is_err());
        assert!(single_line("field", "").is_err());
    }

    #[test]
    fn text_allows_newlines_but_rejects_controls_and_bidi_overrides() {
        assert!(text("field", "line one\nline two\tindented").is_ok());
        assert!(text("field", "   ").is_err());
        assert!(text("field", "bell\u{7}").is_err());
        assert!(text("field", "override\u{202e}").is_err());
    }

    #[test]
    fn labels_reject_rooted_paths_and_file_uris() {
        for (value, valid) in [
            ("Repository synthetic fixture", true),
            ("C: interview", true),
            ("C:/input", false),
            ("c:\\input", false),
            ("/private/input", false),
            ("\\server\\share", false),
            ("file:/input", false),
            ("FILE:/input", false),
        ] {
            assert_eq!(label("label", value).is_ok(), valid, "{value}");
        }
    }

    #[test]
    fn uuids_require_canonical_lowercase_hyphenation() {
        assert!(uuid("id", "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607").is_ok());
        for invalid in [
            "",
            "3F2B1A4C-5D6E-4F70-8A91-B2C3D4E5F607",
            "3f2b1a4c5d6e4f708a91b2c3d4e5f607",
            "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f60",
            "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f60z",
        ] {
            assert!(uuid("id", invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn text_lists_reject_duplicates_and_oversized_entries() {
        let duplicate = vec!["same".to_string(), "same".to_string()];
        assert!(text_list("assumptions", &duplicate, 64).is_err());
        let oversized = vec!["x".repeat(MAX_STRING_BYTES + 1)];
        assert!(text_list("assumptions", &oversized, 64).is_err());
        let too_many = vec!["distinct".to_string(); 65];
        assert!(text_list("assumptions", &too_many, 64).is_err());
    }
}

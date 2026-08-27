//! YAML frontmatter parsing for policy documents.
//!
//! Extracts metadata from YAML frontmatter delimited by `---` at the start
//! of a document. Used by `assemble_document` to populate `DocumentMetadata`.
//!
//! # Security Requirements (SEC-005)
//!
//! - SEC-1: Malformed YAML produces a warning, not a panic
//! - SEC-3: Empty input returns `FrontmatterOutcome::Absent` without error
//! - SEC-6: No `unwrap()` on deserialization — graceful error handling

use serde::Deserialize;

/// Frontmatter data extracted from YAML between `---` delimiters.
///
/// All fields are optional to tolerate partial or incomplete frontmatter.
/// Missing fields default to `None` and are resolved by downstream assembly
/// logic (e.g., fallback to first H1 heading for title).
#[derive(Debug, Deserialize)]
pub(crate) struct FrontmatterData {
    /// Document title (maps to `DocumentMetadata::title`).
    pub title: Option<String>,

    /// Document version, preferably in semantic versioning format.
    pub version: Option<String>,

    /// Document author.
    pub author: Option<String>,

    /// Document date (ISO 8601 format preferred).
    pub date: Option<String>,
}

/// The outcome of parsing a leading frontmatter block.
///
/// `Malformed` distinguishes an authoring error from intentionally absent
/// frontmatter while preserving fault-tolerant metadata fallback.
#[derive(Debug)]
pub(crate) enum FrontmatterOutcome {
    /// The document has no leading frontmatter fence.
    Absent,
    /// A leading fence was found but the block was incomplete or invalid YAML.
    Malformed,
    /// Successfully parsed frontmatter data.
    Data(FrontmatterData),
}

/// Return the byte offset of the first complete closing frontmatter fence.
///
/// Fences are recognized only when an entire line is `---`; scanning line by
/// line accepts mixed line endings and handles an immediately closed block.
fn closing_fence_offset(content: &str) -> Option<usize> {
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        let line_without_newline = line.strip_suffix('\n').unwrap_or(line);
        let fence = line_without_newline.strip_suffix('\r').unwrap_or(line_without_newline);
        if fence == "---" {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

/// Parse YAML frontmatter from document content.
///
/// Frontmatter must be delimited by a `---` line at the very start of the
/// document. Both LF and CRLF line endings are accepted.
/// [`FrontmatterOutcome::Absent`] denotes intentional absence, while
/// [`FrontmatterOutcome::Malformed`] denotes an incomplete or invalid block.
/// Malformed YAML emits a warning via tracing, not a panic *(SEC-1, SEC-6)*.
pub(crate) fn parse_frontmatter(content: &str) -> FrontmatterOutcome {
    if content.is_empty() {
        return FrontmatterOutcome::Absent;
    }

    let Some(rest) = content.strip_prefix("---\r\n").or_else(|| content.strip_prefix("---\n"))
    else {
        return FrontmatterOutcome::Absent;
    };
    let Some(end) = closing_fence_offset(rest) else {
        return FrontmatterOutcome::Malformed;
    };
    let raw_yaml = &rest[..end];
    let normalized_yaml;
    let yaml = if raw_yaml.contains('\r') {
        normalized_yaml = raw_yaml.replace("\r\n", "\n");
        &normalized_yaml
    } else {
        raw_yaml
    };

    if yaml.trim().is_empty() {
        return FrontmatterOutcome::Data(FrontmatterData {
            title: None,
            version: None,
            author: None,
            date: None,
        });
    }

    let value: serde_yaml::Value = match serde_yaml::from_str(yaml) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!("Failed to parse YAML frontmatter: {error}");
            return FrontmatterOutcome::Malformed;
        }
    };

    if let Some(mapping) = value.as_mapping() {
        for key in mapping.keys() {
            let key = key.as_str().unwrap_or("<non-string>");
            if !matches!(key, "title" | "version" | "author" | "date") {
                tracing::warn!(key, "Ignoring unrecognized YAML frontmatter key");
            }
        }
    }

    match serde_yaml::from_value(value) {
        Ok(data) => FrontmatterOutcome::Data(data),
        Err(error) => {
            tracing::warn!("Failed to parse YAML frontmatter: {error}");
            FrontmatterOutcome::Malformed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T005: Frontmatter Parsing Tests (TDD) ────────────────────────

    /// Valid YAML with all four fields returns `Some(FrontmatterData)` with all fields populated.
    #[test]
    fn valid_yaml_all_fields() {
        let content = "\
---
title: Security Policy
version: 1.0.0
author: Jane Doe
date: 2026-01-15
---
# Body content here
";
        let FrontmatterOutcome::Data(fm) = parse_frontmatter(content) else {
            panic!("Expected parsed frontmatter for valid YAML with all fields");
        };
        assert_eq!(fm.title.as_deref(), Some("Security Policy"));
        assert_eq!(fm.version.as_deref(), Some("1.0.0"));
        assert_eq!(fm.author.as_deref(), Some("Jane Doe"));
        assert_eq!(fm.date.as_deref(), Some("2026-01-15"));
    }

    /// Valid YAML with only title and version returns `Some` with author=None, date=None.
    #[test]
    fn valid_yaml_partial_fields() {
        let content = "\
---
title: Access Control
version: 2.0.0
---
# Content
";
        let FrontmatterOutcome::Data(fm) = parse_frontmatter(content) else {
            panic!("Expected parsed frontmatter for valid YAML with partial fields");
        };
        assert_eq!(fm.title.as_deref(), Some("Access Control"));
        assert_eq!(fm.version.as_deref(), Some("2.0.0"));
        assert!(fm.author.is_none(), "author should be None when not provided");
        assert!(fm.date.is_none(), "date should be None when not provided");
    }

    /// Content with no frontmatter delimiters reports `Absent`.
    #[test]
    fn no_frontmatter_returns_none() {
        let content = "# Just a heading\n\nSome body text.\n";
        assert!(matches!(parse_frontmatter(content), FrontmatterOutcome::Absent));
    }

    /// Malformed YAML reports `Malformed` (fault-tolerant per SEC-1, SEC-6).
    #[test]
    fn malformed_yaml_returns_none() {
        let content = "\
---
title: [unclosed bracket
version: : : invalid
---
# Content
";
        assert!(matches!(parse_frontmatter(content), FrontmatterOutcome::Malformed));
    }

    /// Empty string reports `Absent` (SEC-3).
    #[test]
    fn empty_string_returns_none() {
        assert!(matches!(parse_frontmatter(""), FrontmatterOutcome::Absent));
    }

    /// Frontmatter with only an opening delimiter reports `Malformed`.
    #[test]
    fn missing_closing_delimiter_returns_none() {
        let content = "\
---
title: Incomplete
version: 1.0.0
# No closing delimiter
";
        assert!(matches!(parse_frontmatter(content), FrontmatterOutcome::Malformed));
    }

    /// Frontmatter with all fields set to None (empty YAML block) returns `Some` with all None.
    #[test]
    fn empty_yaml_block_returns_some_with_none_fields() {
        let content = "---\n\n---\n# Content\n";
        let FrontmatterOutcome::Data(fm) = parse_frontmatter(content) else {
            panic!("Expected parsed empty frontmatter");
        };
        assert!(fm.title.is_none());
        assert!(fm.version.is_none());
        assert!(fm.author.is_none());
        assert!(fm.date.is_none());
    }

    /// Triple dashes appearing mid-line in YAML content must NOT be treated as a closing delimiter.
    #[test]
    fn mid_line_triple_dash_not_treated_as_delimiter() {
        let content = "---\ntitle: some---value\nversion: \"1.0\"\n---\n";
        let FrontmatterOutcome::Data(data) = parse_frontmatter(content) else {
            panic!("Mid-line --- should not close the frontmatter block");
        };
        assert_eq!(data.title.as_deref(), Some("some---value"));
        assert_eq!(data.version.as_deref(), Some("1.0"));
    }

    /// Content that does not start with `---` returns `None` even if delimiters exist later.
    #[test]
    fn delimiters_not_at_start_returns_none() {
        let content = "Some text before\n---\ntitle: Policy\n---\n";
        assert!(matches!(parse_frontmatter(content), FrontmatterOutcome::Absent));
    }

    // ── F0568: CRLF front matter must parse like its LF twin ──

    #[test]
    fn crlf_frontmatter_parses_like_lf_twin() {
        let lf = "---\ntitle: CRLF Policy\nversion: \"2.0\"\n---\n\n# Section\n";
        let crlf = "---\r\ntitle: CRLF Policy\r\nversion: \"2.0\"\r\n---\r\n\r\n# Section\r\n";
        let FrontmatterOutcome::Data(a) = parse_frontmatter(lf) else {
            panic!("LF front matter parses");
        };
        let FrontmatterOutcome::Data(b) = parse_frontmatter(crlf) else {
            panic!("CRLF front matter must also parse (F0568)");
        };
        assert_eq!(a.title, b.title);
        assert_eq!(a.version, b.version);
    }

    #[test]
    fn immediately_closed_frontmatter_is_valid() {
        let FrontmatterOutcome::Data(parsed) = parse_frontmatter("---\n---\n# Body") else {
            panic!("empty frontmatter is valid");
        };
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.version, None);
    }

    #[test]
    fn first_mixed_ending_fence_terminates_frontmatter() {
        let content = "---\ntitle: First\r\n---\r\ntitle: Not Frontmatter\n---\n";
        let FrontmatterOutcome::Data(parsed) = parse_frontmatter(content) else {
            panic!("first fence terminates frontmatter");
        };
        assert_eq!(parsed.title.as_deref(), Some("First"));
    }
}

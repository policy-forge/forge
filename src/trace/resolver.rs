use std::path::Path;

/// Check whether the source file has been modified after the OSCAL artifact
/// was generated, by comparing source file mtime against the OSCAL
/// `metadata.last-modified` ISO 8601 timestamp.
///
/// Returns `true` if source appears newer (stale), `false` otherwise.
/// Returns `false` if either timestamp cannot be determined (graceful fallback).
#[must_use]
pub fn check_source_staleness(source_path: &Path, metadata_last_modified: Option<&str>) -> bool {
    let Some(last_modified_str) = metadata_last_modified else {
        return false;
    };

    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(last_modified_str) else {
        return false;
    };
    let oscal_time = parsed.to_utc();

    let Ok(mtime) = std::fs::metadata(source_path).and_then(|m| m.modified()) else {
        return false;
    };

    let source_time: chrono::DateTime<chrono::Utc> = mtime.into();
    source_time > oscal_time
}

/// Validate that a source line number is within the actual file's line count.
///
/// Returns `true` if `line_number == 0` (groups) or `line_number <= source_line_count`.
#[must_use]
pub fn validate_line_reference(line_number: usize, source_line_count: usize) -> bool {
    line_number == 0 || line_number <= source_line_count
}

#[cfg(test)]
mod tests {
    use super::*;

    // T041: check_source_staleness tests

    #[test]
    fn staleness_missing_last_modified_returns_false() {
        assert!(!check_source_staleness(Path::new("nonexistent.md"), None));
    }

    #[test]
    fn staleness_unparseable_timestamp_returns_false() {
        assert!(!check_source_staleness(Path::new("nonexistent.md"), Some("not-a-date")));
    }

    // T042: validate_line_reference tests

    #[test]
    fn line_within_range() {
        assert!(validate_line_reference(5, 10));
    }

    #[test]
    fn line_at_boundary() {
        assert!(validate_line_reference(10, 10));
    }

    #[test]
    fn line_beyond_range() {
        assert!(!validate_line_reference(11, 10));
    }

    #[test]
    fn line_zero_always_valid() {
        assert!(validate_line_reference(0, 0));
        assert!(validate_line_reference(0, 10));
    }

    #[test]
    fn line_in_empty_file() {
        assert!(!validate_line_reference(1, 0));
    }
}

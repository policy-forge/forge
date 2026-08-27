use std::time::SystemTime;

/// Staleness relationship between a source-file snapshot and OSCAL metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staleness {
    /// The source snapshot is no newer than the artifact timestamp.
    Fresh,
    /// The source snapshot is newer than the artifact timestamp.
    Stale,
    /// The source snapshot time or artifact timestamp could not be determined.
    Unknown,
}

/// Check whether a source-file snapshot was modified after the OSCAL artifact
/// was generated, by comparing its mtime against the OSCAL
/// `metadata.last-modified` ISO 8601 timestamp.
#[must_use]
pub fn check_source_staleness(
    source_modified: Option<SystemTime>,
    metadata_last_modified: Option<&str>,
) -> Staleness {
    let (Some(source_modified), Some(last_modified_str)) =
        (source_modified, metadata_last_modified)
    else {
        return Staleness::Unknown;
    };
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(last_modified_str) else {
        return Staleness::Unknown;
    };

    let source_time: chrono::DateTime<chrono::Utc> = source_modified.into();
    if source_time > parsed.to_utc() { Staleness::Stale } else { Staleness::Fresh }
}

/// Validate an optional source line reference against the file's line count.
///
/// A missing line is valid for section-level mappings; concrete lines must be
/// 1-based and no greater than the observed source line count.
#[must_use]
pub fn validate_line_reference(source_line: Option<usize>, source_line_count: usize) -> bool {
    source_line.is_none_or(|line| line > 0 && line <= source_line_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn staleness_is_unknown_without_artifact_timestamp() {
        assert_eq!(check_source_staleness(Some(SystemTime::UNIX_EPOCH), None), Staleness::Unknown);
    }

    #[test]
    fn staleness_is_unknown_for_unparseable_timestamp() {
        assert_eq!(
            check_source_staleness(Some(SystemTime::UNIX_EPOCH), Some("not-a-date")),
            Staleness::Unknown
        );
    }

    #[test]
    fn staleness_detects_fresh_and_stale_source_snapshots() {
        let artifact_time = Some("1970-01-01T00:00:10Z");
        assert_eq!(
            check_source_staleness(
                Some(SystemTime::UNIX_EPOCH + Duration::from_secs(9)),
                artifact_time
            ),
            Staleness::Fresh
        );
        assert_eq!(
            check_source_staleness(
                Some(SystemTime::UNIX_EPOCH + Duration::from_secs(11)),
                artifact_time
            ),
            Staleness::Stale
        );
    }

    #[test]
    fn staleness_normalizes_non_utc_artifact_timestamps() {
        assert_eq!(
            check_source_staleness(
                Some(SystemTime::UNIX_EPOCH + Duration::from_secs(9)),
                Some("1970-01-01T05:30:10+05:30"),
            ),
            Staleness::Fresh
        );
    }

    #[test]
    fn line_within_range() {
        assert!(validate_line_reference(Some(5), 10));
    }

    #[test]
    fn line_at_boundary() {
        assert!(validate_line_reference(Some(10), 10));
    }

    #[test]
    fn line_beyond_range() {
        assert!(!validate_line_reference(Some(11), 10));
    }

    #[test]
    fn missing_line_is_valid_for_section_mapping() {
        assert!(validate_line_reference(None, 10));
    }

    #[test]
    fn zero_is_not_a_valid_concrete_line() {
        assert!(!validate_line_reference(Some(0), 10));
    }

    #[test]
    fn line_in_empty_file() {
        assert!(!validate_line_reference(Some(1), 0));
    }
}

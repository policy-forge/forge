//! Refusal-first redaction for the suggestion payload.
//!
//! A payload is assembled from operator-supplied material, and it is refused
//! rather than silently scrubbed when it still matches a known secret shape.
//! Redaction itself is a literal substitution the operator declared in a rules
//! file; FORGE inserts one fixed marker and never invents replacement text. The
//! rule text is hashed for the record and never written to the request.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::ForgeError;

use super::shared;

/// Maximum size of one redaction rules file.
pub const MAX_RULES_BYTES: u64 = 64 * 1024;
/// Maximum declared redaction rules.
pub const MAX_RULES: usize = 64;
/// Fixed marker substituted for a redacted literal.
pub const MARKER: &str = "[redacted]";
/// Maximum size of one redacted unit, matching the per-unit payload bound.
pub const MAX_REDACTED_UNIT_BYTES: usize = 1024 * 1024;

/// One operator-declared redaction rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionRule {
    /// Stable rule identifier.
    pub rule_id: String,
    /// Literal text to remove, held only in memory.
    pub literal: String,
    /// SHA-256 of the rule's own text, recorded in the request.
    pub rule_sha256: String,
}

/// Secret shapes that are refused even after redaction.
///
/// The names are used in diagnostics; the matched bytes never are.
static SECRET_PATTERNS: LazyLock<[(&str, Regex); 6]> = LazyLock::new(|| {
    [
        (
            "private-key block",
            Regex::new(r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----").expect("valid pattern"),
        ),
        (
            "access-key identifier",
            Regex::new(r"\bAKIA[0-9A-Z]{16}\b").expect("valid pattern"),
        ),
        (
            "bearer token",
            Regex::new(r"(?i)\bbearer\s+[A-Za-z0-9._~+/=-]{20,}").expect("valid pattern"),
        ),
        ("provider key", Regex::new(r"\bsk-[A-Za-z0-9]{20,}\b").expect("valid pattern")),
        (
            "basic authorization",
            Regex::new(r"(?i)\bauthorization\s*:\s*basic\s+[A-Za-z0-9+/=]{16,}")
                .expect("valid pattern"),
        ),
        (
            // Quoted JSON-style credentials and bare assignments alike.
            "assigned credential",
            Regex::new(
                r#"(?i)"?(?:password|passphrase|secret|token|api[_-]?key|private[_-]?key|credential)"?\s*[:=]\s*"?[^\s"]{8,}"#,
            )
            .expect("valid pattern"),
        ),
    ]
});

/// Parse an operator redaction rules file: one `rule-id<TAB>literal` per line.
///
/// Blank lines and `#` comments are ignored.
///
/// # Errors
/// Returns an authoring error for an oversized file, more than
/// [`MAX_RULES`] rules, a malformed line, a duplicate rule identifier, an
/// invalid rule identifier, or an empty literal.
pub fn parse_rules(bytes: &[u8]) -> Result<Vec<RedactionRule>, ForgeError> {
    if bytes.len() as u64 > MAX_RULES_BYTES {
        return Err(shared::error(format!(
            "redaction rules exceed the {MAX_RULES_BYTES} byte limit"
        )));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| shared::error("redaction rules must be UTF-8 text"))?;
    let mut rules = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let (rule_id, literal) = line.split_once('\t').ok_or_else(|| {
            shared::error(format!("redaction rule line {number} must be 'rule-id<TAB>literal'"))
        })?;
        shared::key(&format!("redaction rule line {number} id"), rule_id)?;
        if literal.is_empty() || literal.trim() != literal {
            return Err(shared::error(format!(
                "redaction rule line {number} must name a literal without surrounding whitespace"
            )));
        }
        if !seen.insert(rule_id.to_string()) {
            return Err(shared::error(format!(
                "redaction rule '{rule_id}' is declared more than once"
            )));
        }
        if rules.len() >= MAX_RULES {
            return Err(shared::error(format!("redaction rules exceed {MAX_RULES} entries")));
        }
        rules.push(RedactionRule {
            rule_id: rule_id.to_string(),
            rule_sha256: crate::hashing::sha256_hex(line.as_bytes()),
            literal: literal.to_string(),
        });
    }
    Ok(rules)
}

/// Apply every rule that matches one unit's text, returning the redacted text
/// and the rules that matched, in file order.
///
/// Every match is found in the **original** text, so an inserted marker is never
/// re-scanned by a later rule: re-scanning let one rule's marker feed the next
/// rule's literal and expanded a one-byte input into megabytes. Overlapping
/// matches resolve deterministically by earliest start, then longest literal,
/// then rule identifier.
///
/// # Errors
/// Returns an authoring error when redaction would expand the unit past
/// [`MAX_REDACTED_UNIT_BYTES`].
pub fn apply<'a>(
    text: &str,
    rules: &'a [RedactionRule],
) -> Result<(String, Vec<&'a RedactionRule>), ForgeError> {
    let mut matches: Vec<(usize, usize, &RedactionRule)> = Vec::new();
    for rule in rules {
        for (start, _) in text.match_indices(rule.literal.as_str()) {
            matches.push((start, start + rule.literal.len(), rule));
        }
    }
    if matches.is_empty() {
        return Ok((text.to_string(), Vec::new()));
    }
    matches.sort_by(|left, right| {
        left.0.cmp(&right.0).then(right.1.cmp(&left.1)).then(left.2.rule_id.cmp(&right.2.rule_id))
    });
    // A rule "matched" when its literal appears in the original text, whether or
    // not it wins an overlap; that keeps "matched nothing" honest.
    let mut applied: Vec<&RedactionRule> = Vec::new();
    for rule in rules {
        if matches.iter().any(|(_, _, matched)| matched.rule_id == rule.rule_id)
            && !applied.iter().any(|prior| prior.rule_id == rule.rule_id)
        {
            applied.push(rule);
        }
    }
    let mut redacted = String::with_capacity(text.len());
    let mut cursor = 0;
    for (start, end, _) in matches {
        if start < cursor {
            continue;
        }
        redacted.push_str(&text[cursor..start]);
        redacted.push_str(MARKER);
        cursor = end;
    }
    redacted.push_str(&text[cursor..]);
    if redacted.len() > MAX_REDACTED_UNIT_BYTES {
        return Err(shared::error(format!(
            "redaction expands a unit past the {MAX_REDACTED_UNIT_BYTES} byte bound; \
             use shorter literals or fewer rules"
        )));
    }
    Ok((redacted, applied))
}

/// The first declared rule that never matched any unit.
#[must_use]
pub fn unmatched_rule<'a>(
    rules: &'a [RedactionRule],
    matched: &BTreeSet<&str>,
) -> Option<&'a RedactionRule> {
    rules.iter().find(|rule| !matched.contains(rule.rule_id.as_str()))
}

/// Refuse a payload that still matches a known secret shape.
///
/// # Errors
/// Returns an authoring error naming the pattern and the byte offset. The
/// matched bytes are never included.
pub fn refuse_secrets(payload: &str) -> Result<(), ForgeError> {
    for (name, pattern) in SECRET_PATTERNS.iter() {
        if let Some(found) = pattern.find(payload) {
            return Err(shared::error(format!(
                "payload matches the {name} secret pattern at byte {}; add a redaction rule or remove the material",
                found.start()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    const RULES: &[u8] = b"# comment\n\naccount-id\t123456789012\n";

    #[test]
    fn rules_parse_comments_blanks_and_record_a_rule_hash() {
        let rules = parse_rules(RULES).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].rule_id, "account-id");
        assert_eq!(rules[0].literal, "123456789012");
        assert_eq!(rules[0].rule_sha256, crate::hashing::sha256_hex(b"account-id\t123456789012"));
    }

    #[test]
    fn rules_reject_malformed_duplicate_and_unbounded_input() {
        assert!(parse_rules(b"no-tab-here").is_err());
        assert!(parse_rules(b"Bad-Id\tvalue").is_err());
        assert!(parse_rules(b"id\tvalue\nid\tother\n").is_err());
        assert!(parse_rules(b"id\t  padded\n").is_err());
        assert!(parse_rules(b"id\t\n").is_err());
        assert!(parse_rules(b"\xff\xfe").is_err());
        let mut many = String::new();
        for index in 0..=MAX_RULES {
            let _ = writeln!(many, "rule-{index}\tvalue-{index}");
        }
        assert!(parse_rules(many.as_bytes()).is_err());
    }

    #[test]
    fn applying_a_rule_replaces_every_occurrence_and_reports_it() {
        let rules = parse_rules(RULES).unwrap();
        let (redacted, applied) = apply("call 123456789012 now", &rules).unwrap();
        assert_eq!(redacted, "call [redacted] now");
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].rule_id, "account-id");
    }

    #[test]
    fn a_rule_that_matches_nothing_is_reported_as_unmatched() {
        let rules = parse_rules(RULES).unwrap();
        let (redacted, applied) = apply("nothing to redact", &rules).unwrap();
        assert_eq!(redacted, "nothing to redact");
        assert!(applied.is_empty());
        let matched: BTreeSet<&str> = applied.iter().map(|rule| rule.rule_id.as_str()).collect();
        assert_eq!(unmatched_rule(&rules, &matched).unwrap().rule_id, "account-id");
    }

    #[test]
    fn rules_past_their_byte_bound_are_refused_before_decoding() {
        let oversized = vec![b'a'; usize::try_from(MAX_RULES_BYTES).unwrap() + 1];
        assert!(parse_rules(&oversized).is_err());
    }

    #[test]
    fn secret_shapes_are_refused_without_echoing_the_match() {
        for payload in [
            "-----BEGIN RSA PRIVATE KEY-----\nMIIE",
            "key AKIAIOSFODNN7EXAMPLE here",
            "Authorization: Bearer abcdefghijklmnopqrstuvwxyz012345",
            "token sk-abcdefghijklmnopqrstuvwxyz",
            "password: hunter2-secret",
        ] {
            let error = refuse_secrets(payload).unwrap_err().to_string();
            assert!(error.contains("secret pattern"), "{payload}: {error}");
            assert!(!error.contains("AKIAIOSFODNN7EXAMPLE"), "must not echo the match");
            assert!(!error.contains("hunter2-secret"), "must not echo the match");
        }
        assert!(refuse_secrets("ordinary supplied policy prose").is_ok());
    }

    #[test]
    fn basic_authorization_and_quoted_credentials_are_refused() {
        for payload in [
            "Authorization: Basic dXNlcjpzeW50aGV0aWMtcGFzc3dvcmQ=",
            "{\"token\": \"synthetic-review-secret\"}",
            "{\"password\":\"synthetic-review-secret\"}",
        ] {
            let error = refuse_secrets(payload).unwrap_err().to_string();
            assert!(error.contains("secret pattern"), "{payload}: {error}");
            assert!(!error.contains("synthetic-review-secret"), "must not echo the match");
        }
    }

    #[test]
    fn a_marker_never_feeds_a_later_rule_and_expansion_is_bounded() {
        // Twenty rules whose literal is one byte must not amplify the input:
        // the marker contains `e`, and re-scanning it doubled the text each time.
        let mut rules = String::new();
        for index in 0..20 {
            let _ = writeln!(rules, "rule-{index}\te");
        }
        let parsed = parse_rules(rules.as_bytes()).unwrap();
        let (redacted, applied) = apply("e", &parsed).unwrap();
        assert_eq!(redacted, MARKER);
        assert_eq!(applied.len(), 20, "every declared rule matched the original");

        // A unit whose redaction would exceed the unit bound is refused.
        let big = "e".repeat(MAX_REDACTED_UNIT_BYTES);
        assert!(apply(&big, &parsed).is_err());
    }

    #[test]
    fn overlapping_matches_resolve_deterministically() {
        let rules = parse_rules(b"short\tabc\nlong\tabcd\n").unwrap();
        let (redacted, applied) = apply("abcd", &rules).unwrap();
        // The longest literal at the same start wins the replacement, while both
        // rules are reported as having matched.
        assert_eq!(redacted, MARKER);
        assert_eq!(
            applied.iter().map(|rule| rule.rule_id.as_str()).collect::<Vec<_>>(),
            vec!["short", "long"]
        );

        let (again, _) = apply("abcd", &rules).unwrap();
        assert_eq!(again, redacted);
    }

    #[test]
    fn redaction_clears_a_secret_before_the_refusal_check() {
        let rules = parse_rules(b"example-key\tAKIAIOSFODNN7EXAMPLE\n").unwrap();
        let (redacted, applied) = apply("key AKIAIOSFODNN7EXAMPLE", &rules).unwrap();
        assert!(redacted.contains(MARKER));
        assert_eq!(applied.len(), 1);
        assert!(refuse_secrets(&redacted).is_ok());
    }
}

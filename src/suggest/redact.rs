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
static SECRET_PATTERNS: LazyLock<[(&str, Regex); 5]> = LazyLock::new(|| {
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
            "assigned credential",
            Regex::new(
                r"(?i)\b(?:password|passphrase|secret|token|api[_-]?key|private[_-]?key|credential)\b\s*[:=]\s*\S{8,}",
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
/// A rule is matched per unit; the caller decides whether a rule that matched
/// no unit at all is an error.
#[must_use]
pub fn apply<'a>(text: &str, rules: &'a [RedactionRule]) -> (String, Vec<&'a RedactionRule>) {
    let mut redacted = text.to_string();
    let mut applied = Vec::new();
    for rule in rules {
        if redacted.contains(rule.literal.as_str()) {
            redacted = redacted.replace(rule.literal.as_str(), MARKER);
            applied.push(rule);
        }
    }
    (redacted, applied)
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
        let (redacted, applied) = apply("call 123456789012 now", &rules);
        assert_eq!(redacted, "call [redacted] now");
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].rule_id, "account-id");
    }

    #[test]
    fn a_rule_that_matches_nothing_is_reported_as_unmatched() {
        let rules = parse_rules(RULES).unwrap();
        let (redacted, applied) = apply("nothing to redact", &rules);
        assert_eq!(redacted, "nothing to redact");
        assert!(applied.is_empty());
        let matched: BTreeSet<&str> = applied.iter().map(|rule| rule.rule_id.as_str()).collect();
        assert_eq!(unmatched_rule(&rules, &matched).unwrap().rule_id, "account-id");
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
    fn redaction_clears_a_secret_before_the_refusal_check() {
        let rules = parse_rules(b"example-key\tAKIAIOSFODNN7EXAMPLE\n").unwrap();
        let (redacted, applied) = apply("key AKIAIOSFODNN7EXAMPLE", &rules);
        assert!(redacted.contains(MARKER));
        assert_eq!(applied.len(), 1);
        assert!(refuse_secrets(&redacted).is_ok());
    }
}

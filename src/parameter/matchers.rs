//! Parameter matcher implementations for WI-34.
//!
//! Provides the `ParameterMatcher` trait and four concrete implementations:
//! - `TimeWindowMatcher` — detects "within N days", "after N weeks", "every N months"
//! - `ThresholdMatcher` — detects "at least N", "minimum N", "no more than N"
//! - `FrequencyMatcher` — detects "annually", "quarterly", "at least monthly"
//! - `QuantityMatcher` — detects "no fewer than 3 factors", "at least 2 generations"
//!
//! All regex patterns use `std::sync::LazyLock` for one-time compilation (SEC-4).
//! All patterns use bounded quantifiers with qualifier words (SEC-2, SEC-3).

use std::sync::LazyLock;

use regex::Regex;

use crate::model::{ConstraintType, ParameterConstraint, ParameterType};

/// Intermediate matcher result (crate-private).
///
/// Produced by `ParameterMatcher::find_parameters()`.
/// Consumed by `extract_parameters_from_text()`.
pub(crate) struct ParameterMatch {
    /// Byte offset of the match start in the source text.
    pub start: usize,
    /// Byte offset of the match end in the source text (exclusive).
    pub end: usize,
    /// The full matched text span.
    #[allow(dead_code)]
    pub matched_text: String,
    /// The extracted parameter value (e.g., "30 days", "128-bit", "annually").
    pub value: String,
    /// The semantic category of this parameter.
    pub parameter_type: ParameterType,
    /// Human-readable label derived from the matched text.
    pub label: String,
    /// Value domain constraint inferred from qualifier words.
    pub constraint: Option<ParameterConstraint>,
}

/// Common interface for type-specific parameter matchers (crate-private).
pub(crate) trait ParameterMatcher {
    /// Find all parameters of this type in the given text.
    fn find_parameters(&self, text: &str) -> Vec<ParameterMatch>;
}

// ─── TimeWindowMatcher ──────────────────────────────────────────────────────

/// Pattern: qualifier + digit + time unit.
/// Bounded quantifiers only (SEC-3). `(?i)` for case-insensitive matching.
/// "within"/"after" → Minimum; "every" → Exact.
static TIME_WINDOW: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?P<qualifier>within|after|every)\s+(?P<value>\d+)\s+(?P<unit>days?|weeks?|months?|years?)",
    )
    .expect("TIME_WINDOW regex is valid")
});

/// Detects time window parameters: "within N days", "after N weeks", "every N months".
pub(crate) struct TimeWindowMatcher;

impl ParameterMatcher for TimeWindowMatcher {
    fn find_parameters(&self, text: &str) -> Vec<ParameterMatch> {
        TIME_WINDOW
            .captures_iter(text)
            .map(|cap| {
                let m = cap.get(0).expect("full match always present");
                let qualifier = cap.name("qualifier").expect("qualifier group").as_str();
                let value_str = cap.name("value").expect("value group").as_str();
                let unit = cap.name("unit").expect("unit group").as_str();
                let value = format!("{value_str} {unit}");
                let label = format!("{} {}", qualifier.to_lowercase(), value);
                let constraint_type = if qualifier.eq_ignore_ascii_case("every") {
                    ConstraintType::Exact
                } else {
                    ConstraintType::Minimum
                };
                ParameterMatch {
                    start: m.start(),
                    end: m.end(),
                    matched_text: m.as_str().to_string(),
                    value: value.clone(),
                    parameter_type: ParameterType::TimeWindow,
                    label,
                    constraint: Some(ParameterConstraint { constraint_type, value }),
                }
            })
            .collect()
    }
}

// ─── ThresholdMatcher ───────────────────────────────────────────────────────

/// Minimum threshold pattern: qualifier + numeric value (with optional suffix).
static THRESHOLD_MIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?P<qualifier>at\s+least|minimum|no\s+fewer\s+than|no\s+less\s+than)\s+(?P<value>\d+(?:\.\d+)?(?:-bit)?)",
    )
    .expect("THRESHOLD_MIN regex is valid")
});

/// Maximum threshold pattern: qualifier + numeric value (with optional suffix).
static THRESHOLD_MAX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?P<qualifier>no\s+more\s+than|maximum|at\s+most)\s+(?P<value>\d+(?:\.\d+)?(?:-bit)?)")
        .expect("THRESHOLD_MAX regex is valid")
});

/// Detects threshold parameters: "at least N", "minimum N", "no more than N".
pub(crate) struct ThresholdMatcher;

/// Whether a numeric capture ends at a token boundary rather than a date or identifier.
fn has_numeric_value_boundary(text: &str, end: usize) -> bool {
    text[end..].chars().next().is_none_or(|character| {
        !character.is_alphanumeric() && character != '_' && !matches!(character, '-' | '/' | '.')
    })
}
impl ParameterMatcher for ThresholdMatcher {
    fn find_parameters(&self, text: &str) -> Vec<ParameterMatch> {
        let mut matches = Vec::new();

        for captures in THRESHOLD_MIN.captures_iter(text) {
            let (Some(full_match), Some(qualifier), Some(value)) =
                (captures.get(0), captures.name("qualifier"), captures.name("value"))
            else {
                continue;
            };
            if !has_numeric_value_boundary(text, full_match.end()) {
                continue;
            }
            let value = value.as_str().to_string();
            let qualifier_norm: String =
                qualifier.as_str().split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
            let label = format!("{qualifier_norm} {value}");
            matches.push(ParameterMatch {
                start: full_match.start(),
                end: full_match.end(),
                matched_text: full_match.as_str().to_string(),
                value: value.clone(),
                parameter_type: ParameterType::Threshold,
                label,
                constraint: Some(ParameterConstraint {
                    constraint_type: ConstraintType::Minimum,
                    value,
                }),
            });
        }

        for captures in THRESHOLD_MAX.captures_iter(text) {
            let (Some(full_match), Some(qualifier), Some(value)) =
                (captures.get(0), captures.name("qualifier"), captures.name("value"))
            else {
                continue;
            };
            if !has_numeric_value_boundary(text, full_match.end()) {
                continue;
            }
            let value = value.as_str().to_string();
            let qualifier_norm: String =
                qualifier.as_str().split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
            let label = format!("{qualifier_norm} {value}");
            matches.push(ParameterMatch {
                start: full_match.start(),
                end: full_match.end(),
                matched_text: full_match.as_str().to_string(),
                value: value.clone(),
                parameter_type: ParameterType::Threshold,
                label,
                constraint: Some(ParameterConstraint {
                    constraint_type: ConstraintType::Maximum,
                    value,
                }),
            });
        }

        matches.sort_by_key(|parameter_match| parameter_match.start);
        matches
    }
}

// ─── FrequencyMatcher ───────────────────────────────────────────────────────

/// Pattern: optional "at least" prefix + frequency word.
/// Presence of "at least" prefix → Minimum; absent → Exact.
static FREQUENCY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?P<prefix>at\s+least\s+)?(?P<value>annually|quarterly|monthly|weekly|daily|biannually|semi-annually)",
    )
    .expect("FREQUENCY regex is valid")
});

/// Detects frequency parameters: "annually", "quarterly", "at least monthly".
pub(crate) struct FrequencyMatcher;

/// Reject matches that begin inside hyphenated or alphabetic compound cadence terms.
fn is_compound_frequency_suffix(text: &str, start: usize) -> bool {
    text[..start]
        .chars()
        .next_back()
        .is_some_and(|character| character == '-' || character.is_ascii_alphabetic())
}
impl ParameterMatcher for FrequencyMatcher {
    fn find_parameters(&self, text: &str) -> Vec<ParameterMatch> {
        FREQUENCY
            .captures_iter(text)
            .filter_map(|captures| {
                let (Some(full_match), Some(value)) = (captures.get(0), captures.name("value"))
                else {
                    return None;
                };
                if is_compound_frequency_suffix(text, full_match.start()) {
                    return None;
                }
                let has_prefix =
                    captures.name("prefix").is_some_and(|prefix| !prefix.as_str().is_empty());
                let value = value.as_str().to_lowercase();
                let (label, constraint_type) = if has_prefix {
                    (format!("at least {value}"), ConstraintType::Minimum)
                } else {
                    (value.clone(), ConstraintType::Exact)
                };
                Some(ParameterMatch {
                    start: full_match.start(),
                    end: full_match.end(),
                    matched_text: full_match.as_str().to_string(),
                    value: value.clone(),
                    parameter_type: ParameterType::Frequency,
                    label,
                    constraint: Some(ParameterConstraint { constraint_type, value }),
                })
            })
            .collect()
    }
}

// ─── QuantityMatcher ────────────────────────────────────────────────────────

/// Pattern: qualifier + digit + countable unit noun.
static QUANTITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)(?P<qualifier>at\s+least|no\s+fewer\s+than|minimum)\s+(?P<value>\d+)\s+(?P<unit>\w+)",
    )
    .expect("QUANTITY regex is valid")
});

/// Detects quantity parameters: "no fewer than 3 factors", "at least 2 generations".
pub(crate) struct QuantityMatcher;

impl ParameterMatcher for QuantityMatcher {
    fn find_parameters(&self, text: &str) -> Vec<ParameterMatch> {
        QUANTITY
            .captures_iter(text)
            .map(|cap| {
                let m = cap.get(0).expect("full match always present");
                let qualifier = cap.name("qualifier").expect("qualifier group").as_str();
                let value = cap.name("value").expect("value group").as_str().to_string();
                let unit = cap.name("unit").expect("unit group").as_str().to_string();
                let qualifier_norm: String =
                    qualifier.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
                let label = format!("{qualifier_norm} {value} {unit}");
                ParameterMatch {
                    start: m.start(),
                    end: m.end(),
                    matched_text: m.as_str().to_string(),
                    // value is just the digit (e.g., "3"); unit stays in label
                    value: value.clone(),
                    parameter_type: ParameterType::Quantity,
                    label,
                    constraint: Some(ParameterConstraint {
                        constraint_type: ConstraintType::Minimum,
                        value,
                    }),
                }
            })
            .collect()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn constraint_type(m: &ParameterMatch) -> &ConstraintType {
        m.constraint.as_ref().map(|c| &c.constraint_type).expect("constraint present")
    }

    // ── T008: TimeWindowMatcher positive tests ───────────────────────────

    #[test]
    fn time_window_within_30_days() {
        let m = TimeWindowMatcher;
        let results = m.find_parameters("Passwords must be changed within 30 days");
        assert_eq!(results.len(), 1, "Expected 1 match");
        assert_eq!(results[0].parameter_type, ParameterType::TimeWindow);
        assert_eq!(results[0].value, "30 days");
        assert_eq!(results[0].label, "within 30 days");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
        assert!(results[0].start < results[0].end);
    }

    #[test]
    fn time_window_after_6_weeks() {
        let m = TimeWindowMatcher;
        let results = m.find_parameters("Reviews must be completed after 6 weeks");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "6 weeks");
        assert_eq!(results[0].label, "after 6 weeks");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn time_window_every_90_days() {
        let m = TimeWindowMatcher;
        let results = m.find_parameters("Backups must be tested every 90 days");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "90 days");
        assert_eq!(results[0].label, "every 90 days");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Exact);
    }

    #[test]
    fn time_window_within_2_years() {
        let m = TimeWindowMatcher;
        let results = m.find_parameters("Records must be retained within 2 years");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "2 years");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn time_window_plural_and_singular_units() {
        let m = TimeWindowMatcher;
        // Singular unit: "day"
        let r1 = m.find_parameters("within 1 day");
        assert_eq!(r1.len(), 1);
        assert_eq!(r1[0].value, "1 day");
        // Plural: "months"
        let r2 = m.find_parameters("every 3 months");
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0].value, "3 months");
    }

    #[test]
    fn time_window_case_insensitive() {
        let m = TimeWindowMatcher;
        let results = m.find_parameters("WITHIN 30 DAYS");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, ParameterType::TimeWindow);
    }

    #[test]
    fn time_window_offsets_are_correct() {
        let m = TimeWindowMatcher;
        let text = "Foo within 30 days bar";
        let results = m.find_parameters(text);
        assert_eq!(results.len(), 1);
        assert_eq!(&text[results[0].start..results[0].end], "within 30 days");
    }

    // ── T009: TimeWindowMatcher negative tests (SEC-2) ───────────────────

    #[test]
    fn time_window_no_match_section_reference() {
        let m = TimeWindowMatcher;
        assert!(m.find_parameters("See Section 3.2 for details").is_empty());
        assert!(m.find_parameters("Per NIST SP 800-53 guidance").is_empty());
    }

    #[test]
    fn time_window_no_match_bare_number() {
        let m = TimeWindowMatcher;
        // Bare numbers without qualifier words must not match
        assert!(m.find_parameters("30 days").is_empty());
        assert!(m.find_parameters("The 90 day period").is_empty());
    }

    #[test]
    fn time_window_no_match_rfc_reference() {
        let m = TimeWindowMatcher;
        assert!(m.find_parameters("per RFC 2119 requirements").is_empty());
    }

    // ── T013: ThresholdMatcher positive tests ────────────────────────────

    #[test]
    fn threshold_at_least_128_bit() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("Keys must be at least 128-bit");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, ParameterType::Threshold);
        assert_eq!(results[0].value, "128-bit");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn threshold_minimum_12_characters() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("Passwords must have minimum 12 characters");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "12");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn threshold_no_fewer_than() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("System must have no fewer than 3 redundant components");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "3");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn threshold_no_less_than() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("Retention period of no less than 90 days");
        assert_eq!(results.len(), 1);
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn threshold_no_more_than_15_minutes() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("Recovery time of no more than 15 minutes");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "15");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Maximum);
    }

    #[test]
    fn threshold_maximum_5() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("Allow maximum 5 failed login attempts");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "5");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Maximum);
    }

    #[test]
    fn threshold_at_most_3() {
        let m = ThresholdMatcher;
        let results = m.find_parameters("Sessions at most 3 per user");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "3");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Maximum);
    }

    #[test]
    fn threshold_label_format() {
        let m = ThresholdMatcher;
        let min_results = m.find_parameters("at least 128-bit encryption");
        assert_eq!(min_results[0].label, "at least 128-bit");
        let max_results = m.find_parameters("no more than 15 retries");
        assert_eq!(max_results[0].label, "no more than 15");
    }

    // ── T016: FrequencyMatcher positive tests ────────────────────────────

    #[test]
    fn frequency_at_least_annually_minimum() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Reviews must occur at least annually");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, ParameterType::Frequency);
        assert_eq!(results[0].value, "annually");
        assert_eq!(results[0].label, "at least annually");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn frequency_at_least_monthly_minimum() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Backups must run at least monthly");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "monthly");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn frequency_quarterly_exact() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Audits are performed quarterly");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "quarterly");
        assert_eq!(results[0].label, "quarterly");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Exact);
    }

    #[test]
    fn frequency_weekly_exact() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Reports sent weekly");
        assert_eq!(results.len(), 1);
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Exact);
    }

    #[test]
    fn frequency_daily_exact() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Logs reviewed daily");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "daily");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Exact);
    }

    #[test]
    fn frequency_biannually_exact() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Training conducted biannually");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "biannually");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Exact);
    }

    #[test]
    fn frequency_semi_annually_exact() {
        let m = FrequencyMatcher;
        let results = m.find_parameters("Audits semi-annually");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "semi-annually");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Exact);
    }

    #[test]
    fn frequency_value_lowercased() {
        let m = FrequencyMatcher;
        // Case-insensitive match, value normalized to lowercase
        let results = m.find_parameters("ANNUALLY");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "annually");
    }

    // ── T019: QuantityMatcher positive tests ─────────────────────────────

    #[test]
    fn quantity_no_fewer_than_3_factors() {
        let m = QuantityMatcher;
        let results = m.find_parameters("MFA must require no fewer than 3 authentication factors");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, ParameterType::Quantity);
        // value is just the number; unit goes to label
        assert_eq!(results[0].value, "3");
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn quantity_at_least_2_generations() {
        let m = QuantityMatcher;
        let results = m.find_parameters("Keep at least 2 generations of backups");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, ParameterType::Quantity);
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn quantity_minimum_8_password_characters() {
        let m = QuantityMatcher;
        let results = m.find_parameters("Require minimum 8 password characters");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].parameter_type, ParameterType::Quantity);
        assert_eq!(*constraint_type(&results[0]), ConstraintType::Minimum);
    }

    #[test]
    fn quantity_label_format() {
        let m = QuantityMatcher;
        let results = m.find_parameters("at least 3 factors");
        assert_eq!(results[0].label, "at least 3 factors");
    }

    // ── Overlap: QuantityMatcher vs ThresholdMatcher (T023 reference) ────

    #[test]
    fn overlap_quantity_wins_over_threshold_with_unit_noun() {
        // "no fewer than 3 authentication factors" — QuantityMatcher produces a longer
        // match (includes unit noun) vs ThresholdMatcher (stops at "3")
        let q = QuantityMatcher;
        let t = ThresholdMatcher;
        let text = "MFA must require no fewer than 3 authentication factors";
        let q_results = q.find_parameters(text);
        let t_results = t.find_parameters(text);
        // Both match but QuantityMatcher should win (longer match at same start)
        assert!(!q_results.is_empty());
        assert!(!t_results.is_empty());
        assert!(q_results[0].end > t_results[0].end, "QuantityMatcher should produce longer match");
        assert_eq!(q_results[0].start, t_results[0].start);
    }

    #[test]
    fn overlap_threshold_wins_without_unit_noun() {
        // "no fewer than 3" without a unit noun — only ThresholdMatcher fires
        let q = QuantityMatcher;
        let t = ThresholdMatcher;
        let text = "allow no fewer than 3";
        let q_results = q.find_parameters(text);
        let t_results = t.find_parameters(text);
        // QuantityMatcher requires a unit noun after the number
        assert!(
            q_results.is_empty(),
            "QuantityMatcher should not match without a unit noun following the number AND a non-digit word"
        );
        assert!(!t_results.is_empty(), "ThresholdMatcher should match");
    }

    // ── SEC-3: ReDoS adversarial input ───────────────────────────────────

    #[test]
    fn redos_adversarial_all_matchers() {
        // 10,000 'a's + " within" — should complete without panic or hang
        let adversarial = format!("{} within", "a".repeat(10_000));
        let tw = TimeWindowMatcher;
        let th = ThresholdMatcher;
        let fr = FrequencyMatcher;
        let qu = QuantityMatcher;
        // Must complete without panic (bounded quantifiers guarantee linear time)
        let _ = tw.find_parameters(&adversarial);
        let _ = th.find_parameters(&adversarial);
        let _ = fr.find_parameters(&adversarial);
        let _ = qu.find_parameters(&adversarial);
    }

    #[test]
    fn threshold_rejects_qualifiers_embedded_in_identifiers() {
        let matcher = ThresholdMatcher;
        assert!(matcher.find_parameters("MAXIMUMLoginAge 3").is_empty());
        assert!(matcher.find_parameters("prefixminimum 4").is_empty());
    }

    #[test]
    fn threshold_rejects_hyphenated_and_date_values() {
        let matcher = ThresholdMatcher;
        assert!(matcher.find_parameters("no less than 90-180 days").is_empty());
        assert!(matcher.find_parameters("at most 12/31/2026").is_empty());
        assert_eq!(matcher.find_parameters("minimum 128-bit")[0].value, "128-bit");
    }

    #[test]
    fn frequency_rejects_compound_cadence_suffixes() {
        let matcher = FrequencyMatcher;
        assert!(matcher.find_parameters("Backups run semi-monthly").is_empty());
        assert!(matcher.find_parameters("Reviews are bi-weekly").is_empty());
        assert_eq!(matcher.find_parameters("Reviews are monthly")[0].value, "monthly");
    }
}

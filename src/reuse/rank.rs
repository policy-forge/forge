//! Deterministic lexical ranking of corpus blocks for unresolved sections.
//!
//! Ranking is deliberately boring: `BM25` (`k1 = 1.2`, `b = 0.75`) over
//! lowercased alphanumeric tokens, two fixed boosts, and a total order that does
//! not depend on the order documents, blocks or tokens were supplied in. Scores
//! are quantised to six decimals before sorting, so the same bytes always
//! produce the same candidates and the same report.
//!
//! No block is ever rewritten: a candidate is a pointer into the captured
//! document bytes, and the caller slices the source at the recorded span.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::authoring::model::SectionPlan;

use super::blocks::blocks;
use super::capture::CapturedCorpus;
use super::corpus::{Corpus, CorpusDocument, DocumentStatus};

/// Default number of candidates kept per section.
pub const MAX_CANDIDATES: usize = 5;
/// Hard upper bound for the per-section candidate cap.
pub const MAX_CANDIDATE_LIMIT: usize = 100;
/// Number of blocks of one document that take part in ranking.
pub const MAX_RANKED_BLOCKS_PER_DOCUMENT: usize = 100;
/// Score below which a candidate is flagged low confidence.
pub const FLOOR: f64 = 1.0;
/// `BM25` term-frequency saturation constant.
pub const K1: f64 = 1.2;
/// `BM25` length-normalisation constant.
pub const B: f64 = 0.75;

const SCORE_SCALE: f64 = 1_000_000.0;
const CONTROL_BOOST: f64 = 2.0;
const SCOPE_BOOST: f64 = 0.5;

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// Why a block was kept as a candidate. Order is fixed for stable bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reason {
    /// An exact control identifier assigned to the section occurs in the block.
    ControlMatch,
    /// A token of the section title occurs in the block.
    TopicTerms,
    /// A token of an evaluated question prompt occurs in the block.
    QuestionTerms,
    /// The source document declares the section topic or a matching control family.
    SameFamily,
}

impl Reason {
    /// Stable contract spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ControlMatch => "control-match",
            Self::TopicTerms => "topic-terms",
            Self::QuestionTerms => "question-terms",
            Self::SameFamily => "same-family",
        }
    }
}

/// The lexical query derived from one unresolved authoring section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionQuery {
    /// Stable authoring topic key.
    pub topic_key: String,
    /// Lowercased tokens of the topic title.
    pub title_tokens: BTreeSet<String>,
    /// Lowercased tokens of the section's evaluated question prompts.
    pub question_tokens: BTreeSet<String>,
    /// Framework control identifiers assigned to the section.
    pub control_ids: Vec<String>,
}

impl SectionQuery {
    /// Build a query from one section and the pack's question prompts.
    ///
    /// Unknown question keys contribute no terms rather than failing: the plan
    /// is the authoritative section inventory, and prompts are only an extra
    /// signal.
    #[must_use]
    pub fn from_section(section: &SectionPlan, prompts: &BTreeMap<String, String>) -> Self {
        let mut title_tokens = BTreeSet::new();
        for token in tokens(&section.title) {
            let _ = title_tokens.insert(token);
        }
        let mut question_tokens = BTreeSet::new();
        for question in &section.questions {
            if let Some(prompt) = prompts.get(&question.question_key) {
                for token in tokens(prompt) {
                    let _ = question_tokens.insert(token);
                }
            }
        }
        Self {
            topic_key: section.topic_key.clone(),
            title_tokens,
            question_tokens,
            control_ids: section.control_ids.clone(),
        }
    }

    /// Every distinct term scored by `BM25`; control identifiers drive a boost
    /// instead so their punctuation cannot dilute the query.
    fn terms(&self) -> BTreeSet<&str> {
        self.title_tokens.iter().chain(self.question_tokens.iter()).map(String::as_str).collect()
    }
}

/// One ranked candidate: a verbatim pointer, never paraphrased text.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// Stable corpus document key.
    pub source_key: String,
    /// Portable descendant path of the captured document.
    pub source_path: PathBuf,
    /// Lowercase hexadecimal SHA-256 of the captured document.
    pub source_sha256: String,
    /// Inclusive byte offset of the span in the document.
    pub start: usize,
    /// Exclusive byte offset of the span in the document.
    pub end: usize,
    /// Quantised score, six decimals.
    pub score: f64,
    /// Machine-readable match reasons, in contract order.
    pub reasons: Vec<Reason>,
    /// Whether the score is below [`FLOOR`].
    pub low_confidence: bool,
}

/// Ranking switches fixed for one run.
#[derive(Debug, Clone, Copy)]
pub struct RankOptions {
    /// Whether `draft` documents compete.
    pub include_draft: bool,
    /// Maximum candidates kept per section.
    pub max_candidates: usize,
    /// Candidates below this inclusive score are dropped before truncation.
    pub min_score: f64,
}

impl Default for RankOptions {
    fn default() -> Self {
        Self { include_draft: false, max_candidates: MAX_CANDIDATES, min_score: 0.0 }
    }
}

/// Rank the corpus for every query, returning candidates in query order.
///
/// # Errors
///
/// Returns an authoring error when a captured document is not valid UTF-8,
/// yields more than the block bound, or was not declared by the corpus.
pub fn rank_sections(
    corpus: &Corpus,
    captured: &CapturedCorpus,
    queries: &[SectionQuery],
    options: &RankOptions,
) -> Result<Vec<Vec<Candidate>>, ForgeError> {
    let index = Index::build(corpus, captured, options.include_draft)?;
    Ok(queries.iter().map(|query| index.rank(query, options)).collect())
}

/// One ranked document block, borrowing the captured bytes.
struct Entry<'a> {
    key: &'a str,
    path: &'a Path,
    sha256: &'a str,
    hints: usize,
    start: usize,
    end: usize,
    lowered: String,
    scope_tokens: BTreeSet<String>,
    length: u32,
    frequency: BTreeMap<String, u32>,
}

/// A prebuilt lexical index over every document block that competes.
struct Index<'a> {
    entries: Vec<Entry<'a>>,
    postings: BTreeMap<String, Vec<(usize, u32)>>,
    average_length: f64,
    topic_hints: Vec<BTreeSet<String>>,
    family_hints: Vec<BTreeSet<String>>,
}

impl<'a> Index<'a> {
    fn build(
        corpus: &'a Corpus,
        captured: &'a CapturedCorpus,
        include_draft: bool,
    ) -> Result<Self, ForgeError> {
        let declared: BTreeMap<&str, &CorpusDocument> =
            corpus.documents().iter().map(|document| (document.key.as_str(), document)).collect();
        let mut entries = Vec::new();
        let mut postings: BTreeMap<String, Vec<(usize, u32)>> = BTreeMap::new();
        let mut topic_hints = Vec::new();
        let mut family_hints = Vec::new();
        let mut total_length: u64 = 0;
        for captured_document in &captured.documents {
            let document = declared.get(captured_document.key.as_str()).ok_or_else(|| {
                error(format!("captured document '{}' is not declared", captured_document.key))
            })?;
            if document.status == DocumentStatus::Draft && !include_draft {
                continue;
            }
            let text = std::str::from_utf8(&captured_document.bytes).map_err(|invalid| {
                error(format!(
                    "reuse document '{}' is not valid UTF-8 at byte {}",
                    document.key,
                    invalid.valid_up_to()
                ))
            })?;
            topic_hints.push(document.topic_keys.iter().cloned().collect());
            family_hints.push(document.control_ids.iter().map(|id| control_family(id)).collect());
            let hints = topic_hints.len() - 1;
            for block in
                blocks(&captured_document.bytes)?.into_iter().take(MAX_RANKED_BLOCKS_PER_DOCUMENT)
            {
                let block_text = text.get(block.start..block.end).ok_or_else(|| {
                    error(format!("reuse document '{}' has a non-text span", document.key))
                })?;
                let mut frequency: BTreeMap<String, u32> = BTreeMap::new();
                let mut length: u64 = 0;
                for token in tokens(block_text) {
                    length = length.saturating_add(1);
                    *frequency.entry(token).or_insert(0) += 1;
                }
                let scope_tokens = block
                    .scope_title
                    .as_deref()
                    .map(|title| tokens(title).into_iter().collect())
                    .unwrap_or_default();
                let index = entries.len();
                for (token, count) in &frequency {
                    postings.entry(token.clone()).or_default().push((index, *count));
                }
                total_length = total_length.saturating_add(length);
                entries.push(Entry {
                    key: document.key.as_str(),
                    path: document.path.as_path(),
                    sha256: captured_document.sha256.as_str(),
                    hints,
                    start: block.start,
                    end: block.end,
                    lowered: block_text.to_lowercase(),
                    scope_tokens,
                    length: u32::try_from(length).unwrap_or(u32::MAX),
                    frequency,
                });
            }
        }
        let average_length =
            if entries.is_empty() { 0.0 } else { ratio(total_length, entries.len()) };
        Ok(Self { entries, postings, average_length, topic_hints, family_hints })
    }

    fn rank(&self, query: &SectionQuery, options: &RankOptions) -> Vec<Candidate> {
        if self.entries.is_empty() {
            return Vec::new();
        }
        let mut scores = vec![0.0f64; self.entries.len()];
        let count = as_f64(self.entries.len());
        for term in query.terms() {
            let Some(list) = self.postings.get(term) else { continue };
            if list.is_empty() {
                continue;
            }
            let document_frequency = as_f64(list.len());
            let idf = (1.0 + (count - document_frequency + 0.5) / (document_frequency + 0.5)).ln();
            for (index, frequency) in list {
                let entry = &self.entries[*index];
                let term_frequency = f64::from(*frequency);
                let length = f64::from(entry.length);
                scores[*index] += idf * (term_frequency * (K1 + 1.0))
                    / (term_frequency + K1 * (1.0 - B + B * length / self.average_length));
            }
        }
        let query_families = control_families(&query.control_ids);
        let mut candidates = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let control_match = contains_control_id(&entry.lowered, &query.control_ids);
            let topic_terms = shares_token(&entry.frequency, &query.title_tokens);
            let question_terms = shares_token(&entry.frequency, &query.question_tokens);
            let same_family = self.topic_hints[entry.hints].contains(&query.topic_key)
                || !self.family_hints[entry.hints].is_disjoint(&query_families);
            let scope_match =
                entry.scope_tokens.iter().any(|token| query.title_tokens.contains(token));
            let mut score = scores[index];
            if control_match {
                score += CONTROL_BOOST;
            }
            if scope_match {
                score += SCOPE_BOOST;
            }
            let score = quantise(score);
            if score <= 0.0 || score < options.min_score {
                continue;
            }
            let mut reasons = Vec::new();
            if control_match {
                reasons.push(Reason::ControlMatch);
            }
            if topic_terms {
                reasons.push(Reason::TopicTerms);
            }
            if question_terms {
                reasons.push(Reason::QuestionTerms);
            }
            if same_family {
                reasons.push(Reason::SameFamily);
            }
            candidates.push(Candidate {
                source_key: entry.key.to_owned(),
                source_path: entry.path.to_path_buf(),
                source_sha256: entry.sha256.to_owned(),
                start: entry.start,
                end: entry.end,
                score,
                reasons,
                low_confidence: score < FLOOR,
            });
        }
        candidates.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.source_path.cmp(&right.source_path))
                .then_with(|| left.start.cmp(&right.start))
                .then_with(|| left.source_sha256.cmp(&right.source_sha256))
        });
        candidates.truncate(options.max_candidates);
        candidates
    }
}

fn shares_token(frequency: &BTreeMap<String, u32>, tokens: &BTreeSet<String>) -> bool {
    tokens.iter().any(|token| frequency.contains_key(token))
}

fn control_families(control_ids: &[String]) -> BTreeSet<String> {
    control_ids.iter().map(|id| control_family(id)).collect()
}

fn control_family(identifier: &str) -> String {
    identifier.split(['-', '.', '(', ')']).next().unwrap_or_default().to_lowercase()
}

fn contains_control_id(lowered: &str, control_ids: &[String]) -> bool {
    if control_ids.is_empty() {
        return false;
    }
    control_ids.iter().any(|identifier| contains_bounded(lowered, &identifier.to_lowercase()))
}

fn contains_bounded(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut search = 0;
    while let Some(offset) = haystack[search..].find(needle) {
        let start = search + offset;
        let end = start + needle.len();
        let left = haystack[..start].chars().next_back().is_none_or(|ch| !is_control_char(ch));
        let right = haystack[end..].chars().next().is_none_or(|ch| !is_control_char(ch));
        if left && right {
            return true;
        }
        search = end;
    }
    false
}

fn is_control_char(value: char) -> bool {
    value.is_alphanumeric() || matches!(value, '-' | '.' | '(' | ')')
}

/// Lowercase alphanumeric tokens, Unicode-aware and allocation-light.
fn tokens(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            result.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

fn quantise(value: f64) -> f64 {
    (value * SCORE_SCALE).round() / SCORE_SCALE
}

fn as_f64(value: usize) -> f64 {
    f64::from(u32::try_from(value).unwrap_or(u32::MAX))
}

fn ratio(numerator: u64, denominator: usize) -> f64 {
    f64::from(u32::try_from(numerator).unwrap_or(u32::MAX)) / as_f64(denominator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::sha256_hex;

    fn document(
        key: &str,
        path: &str,
        status: DocumentStatus,
        topic_keys: &[&str],
        control_ids: &[&str],
    ) -> CorpusDocument {
        CorpusDocument {
            key: key.to_owned(),
            path: PathBuf::from(path),
            title: format!("Title for {key}"),
            status,
            rights_label: "Repository synthetic fixture".to_owned(),
            source_label: "Synthetic interview".to_owned(),
            expected_sha256: "a".repeat(64),
            topic_keys: topic_keys.iter().map(|value| (*value).to_owned()).collect(),
            control_ids: control_ids.iter().map(|value| (*value).to_owned()).collect(),
            supersedes: None,
        }
    }

    fn captured(documents: &[CorpusDocument], bodies: &[&[u8]]) -> CapturedCorpus {
        CapturedCorpus {
            root: PathBuf::from("/synthetic"),
            documents: documents
                .iter()
                .zip(bodies)
                .map(|(document, bytes)| crate::reuse::capture::CapturedDocument {
                    key: document.key.clone(),
                    path: document.path.clone(),
                    sha256: sha256_hex(bytes),
                    bytes: (*bytes).to_vec(),
                })
                .collect(),
        }
    }

    fn corpus(documents: Vec<CorpusDocument>) -> Corpus {
        Corpus {
            schema_version: crate::reuse::corpus::CORPUS_SCHEMA_VERSION.to_owned(),
            corpus_key: "synthetic-corpus".to_owned(),
            title: None,
            documents,
        }
    }

    fn query(topic_key: &str, title: &str, control_ids: &[&str]) -> SectionQuery {
        SectionQuery {
            topic_key: topic_key.to_owned(),
            title_tokens: tokens(title).into_iter().collect(),
            question_tokens: BTreeSet::new(),
            control_ids: control_ids.iter().map(|value| (*value).to_owned()).collect(),
        }
    }

    fn approved() -> DocumentStatus {
        DocumentStatus::Approved
    }

    fn only(results: Vec<Vec<Candidate>>) -> Vec<Candidate> {
        results.into_iter().next().unwrap()
    }

    type Shape = (String, PathBuf, usize, usize, u64, Vec<Reason>, bool);

    fn shape(candidates: &[Candidate]) -> Vec<Shape> {
        candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.source_key.clone(),
                    candidate.source_path.clone(),
                    candidate.start,
                    candidate.end,
                    candidate.score.to_bits(),
                    candidate.reasons.clone(),
                    candidate.low_confidence,
                )
            })
            .collect()
    }

    #[test]
    fn reordering_documents_and_corpus_preserves_candidates() {
        let documents = vec![
            document("alpha", "alpha.md", approved(), &[], &[]),
            document("bravo", "bravo.md", approved(), &[], &[]),
            document("charlie", "charlie.md", approved(), &[], &[]),
        ];
        let bodies: Vec<&[u8]> =
            vec![b"# Topic\n\nalpha bravo\n", b"# Topic\n\nalpha\n", b"# Topic\n\ncharlie\n"];
        let forward = corpus(documents.clone());
        let forward_captured = captured(&documents, &bodies);
        let queries = [query("topic", "alpha", &[])];

        let mut reversed_documents = documents.clone();
        reversed_documents.reverse();
        let mut reversed_bodies = bodies.clone();
        reversed_bodies.reverse();
        let reversed = corpus(reversed_documents.clone());
        let reversed_captured = captured(&reversed_documents, &reversed_bodies);

        let first = rank_sections(&forward, &forward_captured, &queries, &RankOptions::default());
        let second =
            rank_sections(&reversed, &reversed_captured, &queries, &RankOptions::default());
        let first = only(first.unwrap());
        let second = only(second.unwrap());
        assert_eq!(shape(&first), shape(&second));
    }

    #[test]
    fn equal_scores_tie_break_on_path_then_span() {
        let documents = vec![
            document("bravo", "b.md", approved(), &[], &[]),
            document("alpha", "a.md", approved(), &[], &[]),
        ];
        let bodies: Vec<&[u8]> = vec![b"alpha\n\nalpha\n", b"alpha\n\nalpha\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &[])],
            &RankOptions::default(),
        )
        .unwrap();
        let candidates = only(results);
        let order: Vec<_> = candidates
            .iter()
            .map(|candidate| {
                (candidate.source_path.to_string_lossy().into_owned(), candidate.start)
            })
            .collect();
        assert_eq!(
            order,
            [
                ("a.md".to_owned(), 0),
                ("a.md".to_owned(), 7),
                ("b.md".to_owned(), 0),
                ("b.md".to_owned(), 7),
            ]
        );
    }

    #[test]
    fn control_id_match_adds_an_exact_two_point_boost() {
        let documents = vec![document("policy", "policy.md", approved(), &[], &[])];
        let bodies: Vec<&[u8]> = vec![b"# Topic\n\nalpha ac-1\n\nalpha beta x\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &["ac-1"])],
            &RankOptions::default(),
        )
        .unwrap();
        let candidates = only(results);
        assert_eq!(candidates.len(), 2);
        let with_control = candidates[0].score;
        let without_control = candidates[1].score;
        assert!((with_control - without_control - CONTROL_BOOST).abs() < 2e-6);
        assert_eq!(candidates[0].reasons, [Reason::ControlMatch, Reason::TopicTerms]);
        assert_eq!(candidates[1].reasons, [Reason::TopicTerms]);
    }

    #[test]
    fn a_shared_scope_token_adds_the_half_point_boost() {
        let documents = vec![document("policy", "policy.md", approved(), &[], &[])];
        let bodies: Vec<&[u8]> = vec![b"# alpha\n\ncontent\n\n# beta\n\ncontent\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &[])],
            &RankOptions::default(),
        )
        .unwrap();
        let candidates = only(results);
        assert_eq!(candidates.len(), 1);
        assert!((candidates[0].score - SCOPE_BOOST).abs() < 1e-6);
        assert!(candidates[0].low_confidence);
    }

    #[test]
    fn max_candidates_caps_each_section() {
        let documents = vec![document("policy", "policy.md", approved(), &[], &[])];
        let bodies: Vec<&[u8]> = vec![b"alpha\n\nalpha\n\nalpha\n\nalpha\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &[])],
            &RankOptions { max_candidates: 2, ..RankOptions::default() },
        )
        .unwrap();
        assert_eq!(only(results).len(), 2);
    }

    #[test]
    fn strong_and_weak_candidates_are_flagged_by_floor() {
        let documents = vec![
            document("weak", "weak.md", approved(), &[], &[]),
            document("strong", "strong.md", approved(), &[], &[]),
        ];
        let bodies: Vec<&[u8]> = vec![b"# alpha\n\nplain\n", b"# beta\n\nac-1\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &["ac-1"])],
            &RankOptions::default(),
        )
        .unwrap();
        let candidates = only(results);
        assert_eq!(candidates.len(), 2);
        let strong = candidates.iter().find(|item| item.source_key == "strong").unwrap();
        let weak = candidates.iter().find(|item| item.source_key == "weak").unwrap();
        assert!(!strong.low_confidence);
        assert!(strong.reasons.contains(&Reason::ControlMatch));
        assert!(weak.low_confidence);
        assert!((weak.score - SCOPE_BOOST).abs() < 1e-6);
    }

    #[test]
    fn min_score_drops_weak_candidates() {
        let documents = vec![document("policy", "policy.md", approved(), &[], &[])];
        let bodies: Vec<&[u8]> = vec![b"# alpha\n\ncontent\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &[])],
            &RankOptions { min_score: FLOOR, ..RankOptions::default() },
        )
        .unwrap();
        assert!(only(results).is_empty());
    }

    #[test]
    fn draft_documents_are_excluded_unless_included() {
        let documents = vec![
            document("approved", "approved.md", approved(), &[], &[]),
            document("draft", "draft.md", DocumentStatus::Draft, &[], &[]),
        ];
        let bodies: Vec<&[u8]> = vec![b"alpha\n", b"alpha\n"];
        let corpus = corpus(documents.clone());
        let captured = captured(&documents, &bodies);
        let queries = [query("topic", "alpha", &[])];

        let default =
            only(rank_sections(&corpus, &captured, &queries, &RankOptions::default()).unwrap());
        assert_eq!(default.len(), 1);
        assert_eq!(default[0].source_key, "approved");

        let included = only(
            rank_sections(
                &corpus,
                &captured,
                &queries,
                &RankOptions { include_draft: true, ..RankOptions::default() },
            )
            .unwrap(),
        );
        assert_eq!(included.len(), 2);
    }

    #[test]
    fn declared_topic_and_control_family_hints_add_same_family() {
        let documents = vec![
            document("hinted", "hinted.md", approved(), &["topic"], &["ac-7"]),
            document("plain", "plain.md", approved(), &[], &[]),
        ];
        let bodies: Vec<&[u8]> = vec![b"alpha\n", b"alpha\n"];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &["ac-1"])],
            &RankOptions::default(),
        )
        .unwrap();
        let candidates = only(results);
        let hinted = candidates.iter().find(|item| item.source_key == "hinted").unwrap();
        let plain = candidates.iter().find(|item| item.source_key == "plain").unwrap();
        assert!(hinted.reasons.contains(&Reason::SameFamily));
        assert!(!plain.reasons.contains(&Reason::SameFamily));
    }

    #[test]
    fn ranked_blocks_per_document_are_bounded() {
        let mut body = String::from("alpha\n\n");
        for _ in 1..MAX_RANKED_BLOCKS_PER_DOCUMENT + 20 {
            body.push_str("alpha\n\n");
        }
        let documents = vec![document("policy", "policy.md", approved(), &[], &[])];
        let bodies: Vec<&[u8]> = vec![body.as_bytes()];
        let results = rank_sections(
            &corpus(documents.clone()),
            &captured(&documents, &bodies),
            &[query("topic", "alpha", &[])],
            &RankOptions { max_candidates: MAX_CANDIDATE_LIMIT, ..RankOptions::default() },
        )
        .unwrap();
        assert_eq!(only(results).len(), MAX_RANKED_BLOCKS_PER_DOCUMENT);
    }

    #[test]
    fn tokenizer_is_unicode_alphanumeric_and_lowercase() {
        assert_eq!(tokens("Access-Control (AC-2) Café"), ["access", "control", "ac", "2", "café"]);
        assert_eq!(tokens("秘密"), ["秘密"]);
    }
}

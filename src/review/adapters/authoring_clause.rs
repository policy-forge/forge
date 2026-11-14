//! Minimized per-clause Authoring re-review facts from the genuine sealed plan closure.
//!
//! Each captured human clause becomes one reviewable subject bound to its exact
//! original raw identity. The sole authority-bearing input is the receiver-issued
//! `CurrentAuthoringPlanClosure`; hashes, selected keys and asserted policy quorum
//! create no native approval, currentness or publication capability.

use std::collections::BTreeMap;

use crate::hashing::sha256_hex;
use crate::review::authoring_capture::CurrentAuthoringPlanClosure;
use crate::review::decode::{ContractError, ContractLedger};
use crate::review::hash_v3;
use crate::workspace::preparation::WorkControl;

/// Complete captured human-clause count ceiling, not a partial prefix.
const MAX_CLAUSES: usize = 10_000;
/// Exact existing path-free review subject token ceiling in UTF-8 bytes.
const MAX_SUBJECT_ID: usize = 256;

/// One selected actual clause subject, constructible only by this captured adapter.
pub(crate) struct ClauseFact<'a> {
    /// Exact native clause key, never inferred from a locator or path.
    subject_id: &'a str,
    /// Full domain-separated per-clause subject fingerprint; prose is hashed, not exported.
    subject_sha256: String,
    /// Exact complete original raw identity borrowed from the sealed member.
    raw_sha256: &'a str,
    /// Full original extent.
    byte_length: u64,
}

impl ClauseFact<'_> {
    /// Borrow the actual selected native clause key.
    pub(crate) fn subject_id(&self) -> &str {
        self.subject_id
    }
    /// Borrow the complete domain-separated per-clause subject binding.
    pub(crate) fn subject_sha256(&self) -> &str {
        &self.subject_sha256
    }
    /// Borrow the exact original raw digest this subject is bound to.
    pub(crate) fn raw_sha256(&self) -> &str {
        self.raw_sha256
    }
    /// Borrow the full original byte extent.
    pub(crate) fn byte_length(&self) -> u64 {
        self.byte_length
    }
}

/// Per-clause facts borrowing one genuine complete current authoring-plan capture owner.
pub(crate) struct PreparedAuthoringClauseReview<'a> {
    /// Sole authority-bearing input is the opaque receiver-issued captured closure.
    closure: &'a CurrentAuthoringPlanClosure,
    /// All selected facts, in exact selected native clause-key order.
    facts: Vec<ClauseFact<'a>>,
    /// Complete captured clause denominator before selection.
    complete_clauses: usize,
}

impl<'a> PreparedAuthoringClauseReview<'a> {
    /// Borrow the genuine complete native owner, never a detached proof.
    pub(crate) fn closure(&self) -> &'a CurrentAuthoringPlanClosure {
        self.closure
    }
    /// Borrow the complete captured clause denominator.
    pub(crate) fn complete_clauses(&self) -> usize {
        self.complete_clauses
    }
    /// Borrow the selected per-clause facts in exact native key order.
    pub(crate) fn facts(&self) -> &[ClauseFact<'a>] {
        &self.facts
    }
    /// Consume the selected clause facts' complete queue binding under the same ledger.
    /// Each clause subject is bound to its own sealed ordinal source pin by exact original
    /// raw identity; a missing or altered clause pin refuses, never re-binds. The genuine
    /// closure owner is re-verified so no detached fact vector can issue this binding.
    pub(crate) fn bind_queue(
        &self,
        queue: &crate::review::wire_v3::QueueDocumentV3,
        ledger: &mut ContractLedger,
        control: &mut dyn WorkControl,
    ) -> Result<(), ContractError> {
        use crate::review::wire_v3::SourceKindV3;
        ledger.checkpoint(control)?;
        self.closure().verify_inputs(ledger, control)?;
        let pins: Vec<_> =
            queue.source_pins.iter().filter(|pin| pin.kind == SourceKindV3::HumanClause).collect();
        ledger.visits(pins.len())?;
        if pins.len() != self.complete_clauses() || pins.len() != self.facts().len() {
            return Err(ContractError::Binding);
        }
        // Clause pins carry gap-free declaration ordinals from the native project's
        // human_clauses order; bind each subject to its own exact original pin.
        let declared: Vec<&str> = self
            .closure()
            .facts()
            .loaded
            .project
            .human_clauses
            .iter()
            .map(|clause| clause.key.as_str())
            .collect();
        ledger.derived(
            declared
                .len()
                .checked_mul(std::mem::size_of::<&str>())
                .ok_or(ContractError::Capacity)?,
        )?;
        for fact in self.facts() {
            ledger.visits(1)?;
            let ordinal = declared
                .iter()
                .position(|key| *key == fact.subject_id())
                .ok_or(ContractError::Binding)?;
            let expected = format!("authoring:clause:{ordinal}");
            ledger.derived(expected.len())?;
            let pin = pins
                .iter()
                .find(|pin| pin.artifact_key == expected)
                .ok_or(ContractError::Binding)?;
            if pin.raw_sha256 != fact.raw_sha256() || pin.byte_length != fact.byte_length() {
                return Err(ContractError::Binding);
            }
            crate::review::chain::compare(&pin.raw_sha256, fact.subject_sha256(), ledger)?;
        }
        Ok(())
    }
}

/// Admit explicit selection and derive per-clause facts only from the sealed closure.
///
/// The sealed input has consumed the complete maintained native comparison and every
/// human-clause original. This adapter re-verifies each selected clause's exact original
/// digest against its sealed member before issuing the subject binding; a tampered
/// original is refused, never silently re-bound. No file is opened and no approval is
/// promoted here.
pub(crate) fn prepare<'a>(
    closure: &'a CurrentAuthoringPlanClosure,
    selected: &[&'a str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedAuthoringClauseReview<'a>, ContractError> {
    ledger.bound(|ledger| {
        ledger.checkpoint(control)?;
        let result = prepare_inner(closure, selected, ledger, control);
        ledger.checkpoint(control)?;
        result
    })
}

/// Preserve one outer post-phase fence after all ordinary binding outcomes.
fn prepare_inner<'a>(
    closure: &'a CurrentAuthoringPlanClosure,
    selected: &[&'a str],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<PreparedAuthoringClauseReview<'a>, ContractError> {
    if selected.is_empty() || selected.len() > MAX_CLAUSES {
        return Err(ContractError::Invalid);
    }
    ledger.visits(selected.len())?;
    ledger.bytes(selected.len().checked_mul(MAX_SUBJECT_ID).ok_or(ContractError::Capacity)?)?;
    for key in selected {
        crate::review::validate::token(key, MAX_SUBJECT_ID)?;
    }
    if selected.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(ContractError::Invalid);
    }

    // Recheck all actual physical inputs under the same original invocation work owner.
    closure.verify_inputs(ledger, control)?;

    let clauses = &closure.facts().loaded.clauses;
    if clauses.len() > MAX_CLAUSES {
        return Err(ContractError::Capacity);
    }
    let complete_clauses = clauses.len();

    // Correlate every sealed clause member to its exact native key. Key-based matching
    // is unambiguous even when two clauses share identical original bytes.
    let mut sealed: BTreeMap<&str, &str> = BTreeMap::new();
    for (key, raw_sha256) in closure.clause_members() {
        ledger.visits(1)?;
        ledger.derived(std::mem::size_of::<(&str, &str)>() + key.len() + raw_sha256.len())?;
        if sealed.insert(key, raw_sha256).is_some() {
            return Err(ContractError::Binding);
        }
    }

    let mut facts = Vec::new();
    ledger.derived(
        selected
            .len()
            .checked_mul(std::mem::size_of::<ClauseFact<'a>>())
            .ok_or(ContractError::Capacity)?,
    )?;
    for key in selected {
        ledger.visits(1)?;
        let clause = clauses.get(*key).ok_or(ContractError::Binding)?;
        let raw_sha256 = sealed.get(key).copied().ok_or(ContractError::Binding)?;
        // Re-derive the bound original digest from the actual parsed clause allocation;
        // a tampered clause original refuses rather than issuing a stale binding.
        if sha256_hex(&clause.bytes) != raw_sha256 {
            return Err(ContractError::Binding);
        }
        let byte_length = u64::try_from(clause.bytes.len()).map_err(|_| ContractError::Capacity)?;
        let subject_sha256 =
            hash_v3::clause_subject(key, raw_sha256, byte_length, ledger, control)?;
        facts.push(ClauseFact { subject_id: key, subject_sha256, raw_sha256, byte_length });
    }

    Ok(PreparedAuthoringClauseReview { closure, facts, complete_clauses })
}

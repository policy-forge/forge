// Genuine per-clause 061 adapter controls: each sealed human clause is one reviewable
// subject bound to its exact original; a tampered clause original is refused.
use super::*;
use crate::review::adapters::authoring_clause as adapter;
use crate::review::hash_v3;

/// Each captured clause binds its own exact original digest and a distinct per-clause subject.
#[test]
fn per_clause_subject_binds_each_sealed_original() {
    let mut f = Fixture::new(false, false);
    f.answer();
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();

    let prepared =
        adapter::prepare(&owner, &["access-clause", "operations-clause"], &mut ledger, &mut control)
            .unwrap();
    assert_eq!(prepared.complete_clauses(), 2);
    let facts = prepared.facts();
    assert_eq!(facts.len(), 2);
    assert_eq!(facts[0].subject_id(), "access-clause");
    assert_eq!(facts[1].subject_id(), "operations-clause");
    // Each subject is bound to its own sealed original raw identity, not the whole-plan digest.
    assert_eq!(facts[0].raw_sha256(), sha256_hex(&f.clauses[0]));
    assert_eq!(facts[1].raw_sha256(), sha256_hex(&f.clauses[1]));
    assert_eq!(facts[0].byte_length(), f.clauses[0].len() as u64);
    assert_ne!(facts[0].subject_sha256(), facts[1].subject_sha256());
    // The per-clause subject profile reproduces the bound fingerprint independently.
    assert_eq!(
        facts[0].subject_sha256(),
        hash_v3::clause_subject(
            "access-clause",
            facts[0].raw_sha256(),
            facts[0].byte_length(),
            &mut ledger,
            &mut control
        )
        .unwrap()
    );
    // The genuine owner stays borrowed; no detached proof is issued.
    assert_eq!(prepared.closure().source_pins().len(), owner.source_pins().len());
}

/// A tampered clause original refuses the whole preparation, never issuing a stale binding.
#[test]
fn tampered_clause_original_is_refused() {
    let mut f = Fixture::new(false, false);
    f.answer();
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    std::fs::write(f.root.join("clause.md"), b"tampered complete human clause original").unwrap();
    binding(adapter::prepare(
        &owner,
        &["access-clause", "operations-clause"],
        &mut ledger,
        &mut control,
    ));
}

/// Selection must be a nonempty strictly-increasing list of captured clause keys.
#[test]
fn selection_order_and_membership_are_enforced() {
    let f = Fixture::new(false, false);
    originals(&f);
    let mut ledger = ContractLedger::default();
    let mut caller = NoopControl;
    let mut control = ReviewControl::accept(&mut caller);
    let owner = init(&f, &mut ledger, &mut control).unwrap();
    // Empty selection and unsorted selection are structural refusals.
    assert!(matches!(
        adapter::prepare(&owner, &[], &mut ledger, &mut control),
        Err(ContractError::Invalid)
    ));
    assert!(matches!(
        adapter::prepare(&owner, &["operations-clause", "access-clause"], &mut ledger, &mut control),
        Err(ContractError::Invalid)
    ));
    // Unknown clause key.
    binding(adapter::prepare(&owner, &["missing-clause"], &mut ledger, &mut control));
}

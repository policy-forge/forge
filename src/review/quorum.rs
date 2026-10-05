//! Deterministic maximum matching of declared approval keys to finite seats.
//! Matching never authenticates keys, removes dissent or proves native currency.

use super::chain::{ResponseRegistry, compare, reserved, visit};
use super::decode::{ContractError, ContractLedger};
use super::wire::{Disposition, QueueDocument, ResponseClassification, ReviewItem, ReviewPolicy};
use crate::workspace::preparation::WorkControl;

/// One expanded positive declared approval seat, borrowing its original role.
pub(super) struct Seat<'a> {
    /// Exact declared role, without normalization.
    pub(super) role: &'a str,
    /// Original zero-based ordinal within that role's positive requirement.
    pub(super) ordinal: u32,
}

/// Private complete matching facts; one key owns at most one seat.
pub(super) struct Matching<'a> {
    /// Every declared seat, in role/ordinal order.
    pub(super) seats: Vec<Seat<'a>>,
    /// Original unique response index per met seat; none preserves unmet seats.
    pub(super) witnesses: Vec<Option<usize>>,
}

/// Every repeated node, edge and backtrack obeys the same control and ledger.
fn step(ledger: &mut ContractLedger, control: &mut dyn WorkControl) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.matching(1)
}

/// Materialize complete finite seats only after checked count admission.
fn seats<'a>(
    policy: &'a ReviewPolicy,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<Seat<'a>>, ContractError> {
    let mut count = 0_usize;
    for required in &policy.seats {
        visit(ledger, control)?;
        count = count
            .checked_add(usize::try_from(required.count).map_err(|_| ContractError::Capacity)?)
            .ok_or(ContractError::Capacity)?;
    }
    if count == 0 || count > 100 {
        return Err(ContractError::Capacity);
    }
    let mut result = reserved(count, ledger)?;
    for required in &policy.seats {
        for ordinal in 0..required.count {
            step(ledger, control)?;
            result.push(Seat { role: &required.role_key, ordinal });
        }
    }
    Ok(result)
}

/// Charge complete original comparison extents before inspecting a declared edge.
fn eligible(
    item: &ReviewItem,
    policy: &ReviewPolicy,
    response: &super::wire::ResponseDocument,
    seat: &Seat<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<bool, ContractError> {
    let same_role = compare(&response.reviewer_role, seat.role, ledger)?.is_eq();
    for assigned in &item.assignments {
        step(ledger, control)?;
        if compare(&assigned.reviewer_key, &response.reviewer_key, ledger)?.is_eq()
            && compare(&assigned.role_key, &response.reviewer_role, ledger)?.is_eq()
            && same_role
        {
            return Ok(true);
        }
    }
    for edge in &policy.substitutions {
        step(ledger, control)?;
        if compare(&edge.reviewer_key, &response.reviewer_key, ledger)?.is_eq()
            && compare(&edge.asserted_role, &response.reviewer_role, ledger)?.is_eq()
            && compare(&edge.seat_role, seat.role, ledger)?.is_eq()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Preserve roster order and select only keys with a live nonconflicted approval.
fn candidates(
    queue: &QueueDocument,
    item: &ReviewItem,
    registry: &ResponseRegistry<'_>,
    classifications: &[ResponseClassification],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<usize>, ContractError> {
    let mut keys = reserved(queue.reviewers.len(), ledger)?;
    for (key_index, key) in queue.reviewers.iter().enumerate() {
        visit(ledger, control)?;
        let mut approving = false;
        for (index, &classification) in classifications.iter().enumerate() {
            visit(ledger, control)?;
            let response = registry.get(index).document();
            if classification == ResponseClassification::Current
                && response.disposition == Disposition::Approve
                && compare(&response.item_key, &item.key, ledger)?.is_eq()
                && compare(&response.reviewer_key, &key.key, ledger)?.is_eq()
            {
                approving = true;
            }
        }
        if approving {
            keys.push(key_index);
        }
    }
    Ok(keys)
}

/// Build a checked dense matrix of exact approval witnesses, never fake edges.
fn edges(
    selection: (&QueueDocument, &ReviewItem, &ReviewPolicy),
    registry: &ResponseRegistry<'_>,
    classes: &[ResponseClassification],
    seats: &[Seat<'_>],
    keys: &[usize],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<Option<usize>>, ContractError> {
    let (queue, item, policy) = selection;
    let count = keys.len().checked_mul(seats.len()).ok_or(ContractError::Capacity)?;
    if count > 10_000 {
        return Err(ContractError::Capacity);
    }
    let mut result = reserved(count, ledger)?;
    for &key in keys {
        for seat in seats {
            let mut witness = None;
            for (index, &class) in classes.iter().enumerate() {
                step(ledger, control)?;
                let response = registry.get(index).document();
                if class == ResponseClassification::Current
                    && response.disposition == Disposition::Approve
                    && compare(&response.item_key, &item.key, ledger)?.is_eq()
                    && compare(&response.reviewer_key, &queue.reviewers[key].key, ledger)?.is_eq()
                    && eligible(item, policy, response, seat, ledger, control)?
                {
                    witness = Some(index);
                    break;
                }
            }
            result.push(witness);
        }
    }
    Ok(result)
}

/// Reset and reserve bounded search state once; all repeated resets are charged.
fn reset<T: Copy>(
    count: usize,
    value: T,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<T>, ContractError> {
    let mut result = reserved(count, ledger)?;
    for _ in 0..count {
        step(ledger, control)?;
        result.push(value);
    }
    Ok(result)
}

/// One iterative breadth-first augmenting path, with no recursive matching.
fn augment(
    root: usize,
    width: usize,
    edges: &[Option<usize>],
    key_match: &mut [Option<usize>],
    seat_match: &mut [Option<usize>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut seen_keys = reset(key_match.len(), false, ledger, control)?;
    let mut parents = reset(width, None, ledger, control)?;
    let mut queue = reserved(key_match.len(), ledger)?;
    seen_keys[root] = true;
    queue.push(root);
    let mut head = 0;
    while head < queue.len() {
        step(ledger, control)?;
        let key = queue[head];
        head += 1;
        for seat in 0..width {
            step(ledger, control)?;
            if edges[key * width + seat].is_none() || parents[seat].is_some() {
                continue;
            }
            parents[seat] = Some(key);
            match seat_match[seat] {
                None => return flip(seat, &parents, key_match, seat_match, ledger, control),
                Some(owner) if !seen_keys[owner] => {
                    seen_keys[owner] = true;
                    queue.push(owner);
                }
                Some(_) => {}
            }
        }
    }
    Ok(())
}

/// Replace the full augmenting path; no key gains a second occupied seat.
fn flip(
    free: usize,
    parents: &[Option<usize>],
    key_match: &mut [Option<usize>],
    seat_match: &mut [Option<usize>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut cursor = Some(free);
    while let Some(seat) = cursor {
        step(ledger, control)?;
        let key = parents[seat].ok_or(ContractError::Invalid)?;
        let previous = key_match[key];
        key_match[key] = Some(seat);
        seat_match[seat] = Some(key);
        cursor = previous;
    }
    Ok(())
}

/// Compute maximum seat cardinality; conflicts are removed by the caller first.
pub(super) fn evaluate<'a>(
    queue: &'a QueueDocument,
    item: &'a ReviewItem,
    policy: &'a ReviewPolicy,
    registry: &ResponseRegistry<'_>,
    classifications: &[ResponseClassification],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Matching<'a>, ContractError> {
    let seats = seats(policy, ledger, control)?;
    let keys = candidates(queue, item, registry, classifications, ledger, control)?;
    let edges =
        edges((queue, item, policy), registry, classifications, &seats, &keys, ledger, control)?;
    let mut key_match = reset(keys.len(), None, ledger, control)?;
    let mut seat_match = reset(seats.len(), None, ledger, control)?;
    for key in 0..keys.len() {
        step(ledger, control)?;
        augment(key, seats.len(), &edges, &mut key_match, &mut seat_match, ledger, control)?;
    }
    let mut witnesses = reserved(seats.len(), ledger)?;
    for (seat, owner) in seat_match.into_iter().enumerate() {
        step(ledger, control)?;
        witnesses.push(owner.and_then(|key| edges[key * seats.len() + seat]));
    }
    Ok(Matching { seats, witnesses })
}

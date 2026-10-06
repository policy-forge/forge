//! Exact response-original registry and complete self-supersession graph.
//! Every distinct original is retained by the caller; this module borrows it.
//! Chain facts are assertions, not authenticated withdrawals or native proof.

use super::decode::{ContractError, ContractLedger, Decoded, OriginalRelation, original_relation};
use super::wire::ResponseDocument;
use crate::workspace::preparation::WorkControl;
use std::cmp::Ordering;

/// Complete borrowed originals plus deterministic unique identity/chain indices.
pub(crate) struct ResponseRegistry<'a> {
    /// Every actual original, including exact duplicate files.
    originals: &'a [Decoded<'a, ResponseDocument>],
    /// One representative per exact UUID/raw identity, ordered by UUID.
    unique: Vec<usize>,
    /// Exact prior index in the unique registry, or no withdrawal link.
    targets: Vec<Option<usize>>,
    /// Additional files with exactly identical UUID and complete bytes.
    duplicates: usize,
}

impl<'a> ResponseRegistry<'a> {
    /// Preserve every raw holder; a unique representative never releases one.
    pub(crate) fn originals(&self) -> &'a [Decoded<'a, ResponseDocument>] {
        self.originals
    }
    /// Number of distinct actual response identities, not file count.
    pub(crate) fn len(&self) -> usize {
        self.unique.len()
    }
    /// Complete additional exact-file count.
    pub(crate) fn duplicates(&self) -> usize {
        self.duplicates
    }
    /// Borrow one actual admitted original through its deterministic index.
    pub(crate) fn get(&self, index: usize) -> &'a Decoded<'a, ResponseDocument> {
        &self.originals[self.unique[index]]
    }
    /// Borrow the validated direct prior edge; no timestamp chooses a branch.
    pub(crate) fn target(&self, index: usize) -> Option<usize> {
        self.targets[index]
    }
}

/// Precharge a finite typed vector capacity before allocation or insertion.
pub(super) fn reserved<T>(
    count: usize,
    ledger: &mut ContractLedger,
) -> Result<Vec<T>, ContractError> {
    let extent = count
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<T>>()))
        .ok_or(ContractError::Capacity)?;
    ledger.derived(extent)?;
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(|_| ContractError::Capacity)?;
    Ok(result)
}

/// Check the same original caller control before an explicit repeated visit.
pub(super) fn visit(
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    ledger.checkpoint(control)?;
    ledger.visits(1)
}

/// Inspect both complete string extents before their lexical comparison.
pub(super) fn compare(
    left: &str,
    right: &str,
    ledger: &mut ContractLedger,
) -> Result<Ordering, ContractError> {
    ledger.bytes(left.len().checked_add(right.len()).ok_or(ContractError::Capacity)?)?;
    Ok(left.cmp(right))
}

/// Iterative stable merge sort of borrowed indices with all comparisons charged.
fn ordered(
    originals: &[Decoded<'_, ResponseDocument>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Vec<usize>, ContractError> {
    let mut indices = reserved(originals.len(), ledger)?;
    let mut scratch = reserved(originals.len(), ledger)?;
    for index in 0..originals.len() {
        visit(ledger, control)?;
        indices.push(index);
        scratch.push(index);
    }
    let mut width = 1_usize;
    while width < indices.len() {
        let mut start = 0_usize;
        while start < indices.len() {
            visit(ledger, control)?;
            let middle = start.saturating_add(width).min(indices.len());
            let end = middle.saturating_add(width).min(indices.len());
            merge_window(originals, &indices, &mut scratch, (start, middle, end), ledger, control)?;
            start = end;
        }
        std::mem::swap(&mut indices, &mut scratch);
        width = width.checked_mul(2).ok_or(ContractError::Capacity)?;
    }
    Ok(indices)
}

/// Merge one bounded window without cloning documents or omitting repeats.
fn merge_window(
    originals: &[Decoded<'_, ResponseDocument>],
    indices: &[usize],
    scratch: &mut [usize],
    window: (usize, usize, usize),
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let (start, middle, end) = window;
    let mut left = start;
    let mut right = middle;
    for slot in &mut scratch[start..end] {
        visit(ledger, control)?;
        let choose_left = right == end
            || (left < middle
                && compare(
                    &originals[indices[left]].document().response_id,
                    &originals[indices[right]].document().response_id,
                    ledger,
                )? != Ordering::Greater);
        ledger.bytes(std::mem::size_of::<usize>())?;
        *slot = if choose_left {
            let value = indices[left];
            left += 1;
            value
        } else {
            let value = indices[right];
            right += 1;
            value
        };
    }
    Ok(())
}

/// Locate an exact UUID without case folding or unbounded registry scans.
fn locate(
    registry: &ResponseRegistry<'_>,
    id: &str,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<Option<usize>, ContractError> {
    let mut low = 0;
    let mut high = registry.len();
    while low < high {
        visit(ledger, control)?;
        let middle = low + (high - low) / 2;
        match compare(&registry.get(middle).document().response_id, id, ledger)? {
            Ordering::Less => low = middle + 1,
            Ordering::Greater => high = middle,
            Ordering::Equal => return Ok(Some(middle)),
        }
    }
    Ok(None)
}

/// A withdrawal binds the complete same queue/item/reviewer snapshot.
/// Two complete raw extents conservatively cover all repeated typed fields.
fn same_subject(
    left: &Decoded<'_, ResponseDocument>,
    right: &Decoded<'_, ResponseDocument>,
    ledger: &mut ContractLedger,
) -> Result<bool, ContractError> {
    ledger
        .bytes(left.raw().len().checked_add(right.raw().len()).ok_or(ContractError::Capacity)?)?;
    let left = left.document();
    let right = right.document();
    Ok(left.queue_id == right.queue_id
        && left.queue_raw_sha256 == right.queue_raw_sha256
        && left.item_key == right.item_key
        && left.item_id == right.item_id
        && left.domain == right.domain
        && left.adapter_version == right.adapter_version
        && left.requested_action == right.requested_action
        && left.source_pins == right.source_pins
        && left.subject_sha256 == right.subject_sha256
        && left.context_sha256 == right.context_sha256
        && left.policy_sha256 == right.policy_sha256
        && left.reviewer_key == right.reviewer_key)
}

/// Validate every edge, including future/late/history facts, before retirement.
fn edges(
    registry: &mut ResponseRegistry<'_>,
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut successors = reserved(registry.len(), ledger)?;
    for _ in 0..registry.len() {
        visit(ledger, control)?;
        successors.push(false);
        registry.targets.push(None);
    }
    for index in 0..registry.len() {
        visit(ledger, control)?;
        if let Some(prior) = &registry.get(index).document().supersedes {
            let target = locate(registry, &prior.response_id, ledger, control)?
                .ok_or(ContractError::InvalidChain)?;
            ledger.bytes(
                prior
                    .raw_sha256
                    .len()
                    .checked_add(registry.get(target).raw_sha256().len())
                    .ok_or(ContractError::Capacity)?,
            )?;
            if target == index
                || successors[target]
                || prior.raw_sha256 != registry.get(target).raw_sha256()
                || !same_subject(registry.get(index), registry.get(target), ledger)?
            {
                return Err(ContractError::InvalidChain);
            }
            successors[target] = true;
            registry.targets[index] = Some(target);
        }
    }
    acyclic(&registry.targets, ledger, control)
}

/// Detect cycles iteratively; no recursive stack or convenient partial chain.
fn acyclic(
    targets: &[Option<usize>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<(), ContractError> {
    let mut colors = reserved(targets.len(), ledger)?;
    let mut path = reserved(targets.len(), ledger)?;
    for _ in 0..targets.len() {
        visit(ledger, control)?;
        colors.push(0_u8);
    }
    for start in 0..targets.len() {
        visit(ledger, control)?;
        if colors[start] != 0 {
            continue;
        }
        path.clear();
        let mut cursor = Some(start);
        while let Some(index) = cursor {
            visit(ledger, control)?;
            if index >= targets.len() {
                return Err(ContractError::InvalidChain);
            }
            if colors[index] == 1 {
                return Err(ContractError::InvalidChain);
            }
            if colors[index] == 2 {
                break;
            }
            colors[index] = 1;
            path.push(index);
            cursor = targets[index];
        }
        for &index in &path {
            visit(ledger, control)?;
            colors[index] = 2;
        }
    }
    Ok(())
}

/// Reject any ambiguous original/chain as a whole, without consuming holders.
pub(crate) fn prepare<'a>(
    originals: &'a [Decoded<'a, ResponseDocument>],
    ledger: &mut ContractLedger,
    control: &mut dyn WorkControl,
) -> Result<ResponseRegistry<'a>, ContractError> {
    ledger.bound(|ledger| {
        if originals.len() > 10_000 {
            return Err(ContractError::Capacity);
        }
        let sorted = ordered(originals, ledger, control)?;
        let mut unique = reserved(sorted.len(), ledger)?;
        let mut duplicates = 0_usize;
        for index in sorted {
            visit(ledger, control)?;
            let relation = match unique.last() {
                Some(&prior) => {
                    original_relation(&originals[prior], &originals[index], ledger, control)?
                }
                None => OriginalRelation::DifferentIdentity,
            };
            match relation {
                OriginalRelation::DifferentIdentity => unique.push(index),
                OriginalRelation::ExactDuplicate => {
                    duplicates = duplicates.checked_add(1).ok_or(ContractError::Capacity)?;
                }
                OriginalRelation::IdentityConflict => return Err(ContractError::IdentityConflict),
            }
        }
        let targets = reserved(unique.len(), ledger)?;
        let mut registry = ResponseRegistry { originals, unique, targets, duplicates };
        edges(&mut registry, ledger, control)?;
        ledger.checkpoint(control)?;
        Ok(registry)
    })
}

#[cfg(test)]
/// Isolated topology controls; no cryptographic raw-pin cycle is fabricated.
mod tests {
    use super::acyclic;
    use crate::review::decode::{ContractError, ContractLedger};
    use crate::workspace::preparation::NoopControl;

    /// Direct finite cyclic graph reaches the real iterative cycle predicate.
    #[test]
    fn cyclic_topology_refuses_without_fabricating_raw_hash_fixed_points() {
        assert!(matches!(
            acyclic(&[Some(1), Some(2), Some(0)], &mut ContractLedger::default(), &mut NoopControl),
            Err(ContractError::InvalidChain)
        ));
    }

    /// Complete maximum-length topology uses no recursive stack or truncated prefix.
    #[test]
    fn ten_thousand_vertex_linear_topology_is_complete_and_iterative() {
        let targets = (0..10_000)
            .map(|index| if index == 0 { None } else { Some(index - 1) })
            .collect::<Vec<_>>();
        acyclic(&targets, &mut ContractLedger::default(), &mut NoopControl).unwrap();
    }
}

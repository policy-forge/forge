use std::collections::{BTreeMap, BTreeSet};

use super::types::{
    ApprovalStatus, Classification, ConfidenceBasis, EvidenceCode, InventoryRequirement,
    MIGRATION_REPORT_SCHEMA_VERSION, MigrationEntry, MigrationOutcomeCounts, MigrationReport,
    MigrationSummary, RequirementInventory, RequirementLocation,
};
use crate::error::ForgeError;

pub(crate) fn classify(
    old: RequirementInventory,
    new: RequirementInventory,
) -> Result<MigrationReport, ForgeError> {
    validate_cross_inventory_ids(&old.requirements, &new.requirements)?;

    let mut old_matched = vec![false; old.requirements.len()];
    let mut new_matched = vec![false; new.requirements.len()];
    let mut entries = Vec::new();

    match_exact_ids(
        &old.requirements,
        &new.requirements,
        &mut old_matched,
        &mut new_matched,
        &mut entries,
    );
    match_unique_normalized_text(
        &old.requirements,
        &new.requirements,
        &mut old_matched,
        &mut new_matched,
        &mut entries,
    );
    match_unique_locators(
        &old.requirements,
        &new.requirements,
        &mut old_matched,
        &mut new_matched,
        &mut entries,
    );
    group_ambiguities(
        &old.requirements,
        &new.requirements,
        &mut old_matched,
        &mut new_matched,
        &mut entries,
    );
    add_unmatched(&old.requirements, &new.requirements, &old_matched, &new_matched, &mut entries);

    sort_entries(&mut entries);
    let summary = summarize(old.requirements.len(), new.requirements.len(), &entries);
    validate_reconciliation(&old.requirements, &new.requirements, &entries, &summary)?;

    Ok(MigrationReport {
        schema_version: MIGRATION_REPORT_SCHEMA_VERSION,
        forge_version: env!("CARGO_PKG_VERSION"),
        analysis_complete: true,
        old_source: old.source,
        new_source: new.source,
        summary,
        entries,
    })
}

fn validate_cross_inventory_ids(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
) -> Result<(), ForgeError> {
    let new_by_id: BTreeMap<_, _> = new.iter().map(|item| (&item.stable_id, item)).collect();
    for old_item in old {
        if let Some(new_item) = new_by_id.get(&old_item.stable_id)
            && old_item.normalized_text != new_item.normalized_text
        {
            return Err(ForgeError::MigrationError(format!(
                "stable-ID integrity anomaly for '{}'",
                old_item.stable_id
            )));
        }
    }
    Ok(())
}

fn match_exact_ids(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    old_matched: &mut [bool],
    new_matched: &mut [bool],
    entries: &mut Vec<MigrationEntry>,
) {
    let new_by_id: BTreeMap<_, _> =
        new.iter().enumerate().map(|(index, item)| (&item.stable_id, index)).collect();
    for (old_index, old_item) in old.iter().enumerate() {
        let Some(&new_index) = new_by_id.get(&old_item.stable_id) else {
            continue;
        };
        old_matched[old_index] = true;
        new_matched[new_index] = true;
        let mut evidence = vec![EvidenceCode::ExactId];
        evidence.extend(location_changes(&old_item.location, &new[new_index].location));
        evidence.sort_unstable();
        evidence.dedup();
        entries.push(entry(
            Classification::Unchanged,
            evidence,
            ConfidenceBasis::Exact,
            ApprovalStatus::NotRequired,
            vec![old_item.clone()],
            vec![new[new_index].clone()],
        ));
    }
}

fn match_unique_normalized_text(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    old_matched: &mut [bool],
    new_matched: &mut [bool],
    entries: &mut Vec<MigrationEntry>,
) {
    let old_groups = group_unmatched_by(old, old_matched, |item| item.normalized_text.clone());
    let new_groups = group_unmatched_by(new, new_matched, |item| item.normalized_text.clone());
    for (normalized, old_indexes) in old_groups {
        let Some(new_indexes) = new_groups.get(&normalized) else {
            continue;
        };
        if old_indexes.len() != 1 || new_indexes.len() != 1 {
            continue;
        }
        let old_index = old_indexes[0];
        let new_index = new_indexes[0];
        old_matched[old_index] = true;
        new_matched[new_index] = true;
        let mut evidence = vec![EvidenceCode::UniqueNormalizedText];
        evidence.extend(location_changes(&old[old_index].location, &new[new_index].location));
        evidence.sort_unstable();
        evidence.dedup();
        entries.push(entry(
            Classification::ObservedIdChange,
            evidence,
            ConfidenceBasis::Exact,
            ApprovalStatus::NotApproved,
            vec![old[old_index].clone()],
            vec![new[new_index].clone()],
        ));
    }
}

fn match_unique_locators(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    old_matched: &mut [bool],
    new_matched: &mut [bool],
    entries: &mut Vec<MigrationEntry>,
) {
    let old_groups = group_unmatched_by(old, old_matched, locator_key);
    let new_groups = group_unmatched_by(new, new_matched, locator_key);
    for (locator, old_indexes) in old_groups {
        let Some(new_indexes) = new_groups.get(&locator) else {
            continue;
        };
        if old_indexes.len() != 1 || new_indexes.len() != 1 {
            continue;
        }
        let old_index = old_indexes[0];
        let new_index = new_indexes[0];
        if old[old_index].normalized_text == new[new_index].normalized_text {
            continue;
        }
        old_matched[old_index] = true;
        new_matched[new_index] = true;
        entries.push(entry(
            Classification::SubstantiveChangeCandidate,
            vec![EvidenceCode::SameLocator],
            ConfidenceBasis::Candidate,
            ApprovalStatus::NotApproved,
            vec![old[old_index].clone()],
            vec![new[new_index].clone()],
        ));
    }
}

fn group_ambiguities(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    old_matched: &mut [bool],
    new_matched: &mut [bool],
    entries: &mut Vec<MigrationEntry>,
) {
    let mut groups: Vec<(BTreeSet<usize>, BTreeSet<usize>, BTreeSet<EvidenceCode>)> = Vec::new();
    append_candidate_groups(
        &mut groups,
        group_unmatched_by(old, old_matched, |item| item.normalized_text.clone()),
        &group_unmatched_by(new, new_matched, |item| item.normalized_text.clone()),
        EvidenceCode::DuplicateNormalizedText,
    );
    append_candidate_groups(
        &mut groups,
        group_unmatched_by(old, old_matched, locator_key),
        &group_unmatched_by(new, new_matched, locator_key),
        EvidenceCode::CompetingLocator,
    );

    let mut merged: Vec<(BTreeSet<usize>, BTreeSet<usize>, BTreeSet<EvidenceCode>)> = Vec::new();
    for mut group in groups {
        let mut index = 0;
        while index < merged.len() {
            if !group.0.is_disjoint(&merged[index].0) || !group.1.is_disjoint(&merged[index].1) {
                let other = merged.remove(index);
                group.0.extend(other.0);
                group.1.extend(other.1);
                group.2.extend(other.2);
                index = 0;
            } else {
                index += 1;
            }
        }
        merged.push(group);
    }

    for (old_indexes, new_indexes, evidence) in merged {
        for &index in &old_indexes {
            old_matched[index] = true;
        }
        for &index in &new_indexes {
            new_matched[index] = true;
        }
        entries.push(entry(
            Classification::Ambiguous,
            evidence.into_iter().collect(),
            ConfidenceBasis::Unresolved,
            ApprovalStatus::NotApproved,
            old_indexes.into_iter().map(|index| old[index].clone()).collect(),
            new_indexes.into_iter().map(|index| new[index].clone()).collect(),
        ));
    }
}

fn append_candidate_groups<K: Ord>(
    output: &mut Vec<(BTreeSet<usize>, BTreeSet<usize>, BTreeSet<EvidenceCode>)>,
    old_groups: BTreeMap<K, Vec<usize>>,
    new_groups: &BTreeMap<K, Vec<usize>>,
    evidence: EvidenceCode,
) {
    for (key, old_indexes) in old_groups {
        let Some(new_indexes) = new_groups.get(&key) else {
            continue;
        };
        if old_indexes.len() == 1 && new_indexes.len() == 1 {
            continue;
        }
        output.push((
            old_indexes.into_iter().collect(),
            new_indexes.iter().copied().collect(),
            BTreeSet::from([evidence]),
        ));
    }
}

fn add_unmatched(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    old_matched: &[bool],
    new_matched: &[bool],
    entries: &mut Vec<MigrationEntry>,
) {
    for (index, item) in old.iter().enumerate().filter(|(index, _)| !old_matched[*index]) {
        let _ = index;
        entries.push(entry(
            Classification::Retired,
            Vec::new(),
            ConfidenceBasis::Unmatched,
            ApprovalStatus::NotRequired,
            vec![item.clone()],
            Vec::new(),
        ));
    }
    for (index, item) in new.iter().enumerate().filter(|(index, _)| !new_matched[*index]) {
        let _ = index;
        entries.push(entry(
            Classification::Added,
            Vec::new(),
            ConfidenceBasis::Unmatched,
            ApprovalStatus::NotRequired,
            Vec::new(),
            vec![item.clone()],
        ));
    }
}

fn group_unmatched_by<K: Ord>(
    items: &[InventoryRequirement],
    matched: &[bool],
    key: impl Fn(&InventoryRequirement) -> K,
) -> BTreeMap<K, Vec<usize>> {
    let mut groups = BTreeMap::new();
    for (index, item) in items.iter().enumerate().filter(|(index, _)| !matched[*index]) {
        groups.entry(key(item)).or_insert_with(Vec::new).push(index);
    }
    groups
}

fn locator_key(item: &InventoryRequirement) -> (String, usize, usize) {
    (item.location.section_path.clone(), item.location.line, item.location.atom_index)
}

fn location_changes(old: &RequirementLocation, new: &RequirementLocation) -> Vec<EvidenceCode> {
    let mut changes = Vec::new();
    if old.file_label != new.file_label {
        changes.push(EvidenceCode::SourceFileChanged);
    }
    if old.section_path != new.section_path {
        changes.push(EvidenceCode::SectionPathChanged);
    }
    if old.line != new.line {
        changes.push(EvidenceCode::SourceLineChanged);
    }
    if old.atom_index != new.atom_index {
        changes.push(EvidenceCode::AtomIndexChanged);
    }
    changes
}

fn entry(
    classification: Classification,
    evidence: Vec<EvidenceCode>,
    confidence_basis: ConfidenceBasis,
    approval_status: ApprovalStatus,
    mut old: Vec<InventoryRequirement>,
    mut new: Vec<InventoryRequirement>,
) -> MigrationEntry {
    old.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    new.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    MigrationEntry { classification, evidence, confidence_basis, approval_status, old, new }
}

fn sort_entries(entries: &mut [MigrationEntry]) {
    entries.sort_by(|left, right| {
        left.classification
            .rank()
            .cmp(&right.classification.rank())
            .then_with(|| first_id(&left.old).cmp(first_id(&right.old)))
            .then_with(|| first_id(&left.new).cmp(first_id(&right.new)))
    });
}

fn first_id(items: &[InventoryRequirement]) -> &str {
    items.first().map_or("", |item| item.stable_id.as_str())
}

fn summarize(total_old: usize, total_new: usize, entries: &[MigrationEntry]) -> MigrationSummary {
    let count = |classification| {
        entries.iter().filter(|entry| entry.classification == classification).count()
    };
    MigrationSummary {
        total_old,
        total_new,
        unchanged: count(Classification::Unchanged),
        observed_id_changes: count(Classification::ObservedIdChange),
        substantive_change_candidates: count(Classification::SubstantiveChangeCandidate),
        atomization_change_candidates: count(Classification::AtomizationChangeCandidate),
        ambiguity_groups: count(Classification::Ambiguous),
        retired: count(Classification::Retired),
        added: count(Classification::Added),
        old_requirements: outcome_counts(entries, |entry| entry.old.len()),
        new_requirements: outcome_counts(entries, |entry| entry.new.len()),
    }
}

fn outcome_counts(
    entries: &[MigrationEntry],
    side_len: impl Fn(&MigrationEntry) -> usize,
) -> MigrationOutcomeCounts {
    let mut counts = MigrationOutcomeCounts::default();
    for entry in entries {
        let requirement_count = side_len(entry);
        match entry.classification {
            Classification::Unchanged => counts.unchanged += requirement_count,
            Classification::ObservedIdChange => {
                counts.observed_id_changes += requirement_count;
            }
            Classification::SubstantiveChangeCandidate => {
                counts.substantive_change_candidates += requirement_count;
            }
            Classification::AtomizationChangeCandidate => {
                counts.atomization_change_candidates += requirement_count;
            }
            Classification::Ambiguous => counts.ambiguous += requirement_count,
            Classification::Retired => counts.retired += requirement_count,
            Classification::Added => counts.added += requirement_count,
        }
    }
    counts
}

fn validate_reconciliation(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    entries: &[MigrationEntry],
    summary: &MigrationSummary,
) -> Result<(), ForgeError> {
    let expected_old: BTreeSet<_> = old.iter().map(|item| item.stable_id.as_str()).collect();
    let expected_new: BTreeSet<_> = new.iter().map(|item| item.stable_id.as_str()).collect();
    let actual_old: Vec<_> = entries
        .iter()
        .flat_map(|entry| entry.old.iter().map(|item| item.stable_id.as_str()))
        .collect();
    let actual_new: Vec<_> = entries
        .iter()
        .flat_map(|entry| entry.new.iter().map(|item| item.stable_id.as_str()))
        .collect();
    let actual_old_set: BTreeSet<_> = actual_old.iter().copied().collect();
    let actual_new_set: BTreeSet<_> = actual_new.iter().copied().collect();
    if actual_old.len() != actual_old_set.len()
        || actual_new.len() != actual_new_set.len()
        || actual_old_set != expected_old
        || actual_new_set != expected_new
        || summary.old_requirements.total() != summary.total_old
        || summary.new_requirements.total() != summary.total_new
    {
        return Err(ForgeError::MigrationError(
            "internal reconciliation invariant failed".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::types::{InputFormat, LocationBasis, SourceProvenance};

    fn item(id: &str, text: &str, section: &str, line: usize, atom: usize) -> InventoryRequirement {
        InventoryRequirement {
            stable_id: id.to_string(),
            normalized_text_sha256: format!("hash-{text}"),
            normalized_text: text.to_string(),
            location: RequirementLocation {
                file_label: "policy.md".to_string(),
                section_path: section.to_string(),
                section_title: section.to_string(),
                line,
                line_basis: LocationBasis::SourceLine,
                atom_index: atom,
            },
        }
    }

    fn inventory(items: Vec<InventoryRequirement>) -> RequirementInventory {
        RequirementInventory {
            source: SourceProvenance {
                label: "policy.md".to_string(),
                format: InputFormat::Markdown,
                sha256: "source-hash".to_string(),
                location_basis: LocationBasis::SourceLine,
            },
            requirements: items,
        }
    }

    #[test]
    fn classifies_exact_moved_edited_retired_and_added() {
        let old = inventory(vec![
            item("same", "same", "A", 1, 0),
            item("old-moved", "moved", "A", 2, 0),
            item("old-edited", "old prose", "A", 3, 0),
            item("retired", "retired", "A", 4, 0),
        ]);
        let new = inventory(vec![
            item("same", "same", "A", 1, 0),
            item("new-moved", "moved", "B", 20, 0),
            item("new-edited", "new prose", "A", 3, 0),
            item("added", "added", "A", 5, 0),
        ]);

        let report = classify(old, new).unwrap();
        assert_eq!(report.summary.unchanged, 1);
        assert_eq!(report.summary.observed_id_changes, 1);
        assert_eq!(report.summary.substantive_change_candidates, 1);
        assert_eq!(report.summary.retired, 1);
        assert_eq!(report.summary.added, 1);
        assert!(report.has_reviewable_changes());
    }

    #[test]
    fn duplicate_prose_is_one_deterministic_ambiguity_group() {
        let old = inventory(vec![
            item("old-a", "duplicate", "A", 1, 0),
            item("old-b", "duplicate", "B", 2, 0),
        ]);
        let new = inventory(vec![
            item("new-a", "duplicate", "C", 3, 0),
            item("new-b", "duplicate", "D", 4, 0),
        ]);

        let report = classify(old, new).unwrap();
        assert_eq!(report.summary.ambiguity_groups, 1);
        assert_eq!(report.summary.old_requirements.ambiguous, 2);
        assert_eq!(report.summary.new_requirements.ambiguous, 2);
        assert_eq!(report.summary.old_requirements.total(), report.summary.total_old);
        assert_eq!(report.summary.new_requirements.total(), report.summary.total_new);
        assert_eq!(report.entries[0].old.len(), 2);
        assert_eq!(report.entries[0].new.len(), 2);
    }

    #[test]
    fn differing_text_for_same_id_is_integrity_error() {
        let result = classify(
            inventory(vec![item("collision", "old", "A", 1, 0)]),
            inventory(vec![item("collision", "new", "A", 1, 0)]),
        );
        assert!(matches!(result, Err(ForgeError::MigrationError(_))));
    }

    #[test]
    fn identical_inventories_are_clean() {
        let requirements = vec![item("same", "same", "A", 1, 0)];
        let report = classify(inventory(requirements.clone()), inventory(requirements)).unwrap();
        assert!(!report.has_reviewable_changes());
    }
}

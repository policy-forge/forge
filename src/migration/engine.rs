use std::collections::{BTreeMap, BTreeSet};

use super::successor::{RelationshipType, SuccessorMap};
use super::types::{
    ApprovalStatus, Classification, ConfidenceBasis, DeclarationEvidence, EvidenceCode,
    InventoryRequirement, MIGRATION_REPORT_SCHEMA_VERSION, MigrationEntry, MigrationOutcomeCounts,
    MigrationReport, MigrationSummary, RequirementInventory, RequirementLocation,
};
use crate::error::ForgeError;

type CandidateGroup = (BTreeSet<usize>, BTreeSet<usize>, BTreeSet<EvidenceCode>);

pub(crate) fn classify(
    old: RequirementInventory,
    new: RequirementInventory,
    successor_map: Option<&SuccessorMap>,
) -> Result<MigrationReport, ForgeError> {
    validate_unique_stable_ids("old", &old.requirements)?;
    validate_unique_stable_ids("new", &new.requirements)?;
    let new_by_id: BTreeMap<&str, usize> = new
        .requirements
        .iter()
        .enumerate()
        .map(|(index, item)| (item.stable_id.as_str(), index))
        .collect();
    validate_cross_inventory_ids(&old.requirements, &new.requirements, &new_by_id)?;

    let mut old_matched = vec![false; old.requirements.len()];
    let mut new_matched = vec![false; new.requirements.len()];
    let mut entries = Vec::new();

    match_exact_ids(
        &old.requirements,
        &new.requirements,
        &new_by_id,
        &mut old_matched,
        &mut new_matched,
        &mut entries,
    );
    if let Some(successor_map) = successor_map {
        match_declared_relationships(
            &old.requirements,
            &new.requirements,
            successor_map,
            &mut old_matched,
            &mut new_matched,
            &mut entries,
        )?;
    }
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
    match_unique_normalized_text(
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

fn validate_unique_stable_ids(
    side: &str,
    requirements: &[InventoryRequirement],
) -> Result<(), ForgeError> {
    let mut stable_ids = BTreeSet::new();
    for requirement in requirements {
        if !stable_ids.insert(requirement.stable_id.as_str()) {
            return Err(ForgeError::MigrationError(format!(
                "{side} inventory contains duplicate stable identifier '{}'",
                requirement.stable_id
            )));
        }
    }
    Ok(())
}

fn match_declared_relationships(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    successor_map: &SuccessorMap,
    old_matched: &mut [bool],
    new_matched: &mut [bool],
    entries: &mut Vec<MigrationEntry>,
) -> Result<(), ForgeError> {
    let old_by_id: BTreeMap<_, _> =
        old.iter().enumerate().map(|(index, item)| (item.stable_id.as_str(), index)).collect();
    let new_by_id: BTreeMap<_, _> =
        new.iter().enumerate().map(|(index, item)| (item.stable_id.as_str(), index)).collect();
    for relationship in &successor_map.relationships {
        let old_indexes = declared_indexes("old", &relationship.old_ids, &old_by_id, old_matched)?;
        let new_indexes = declared_indexes("new", &relationship.new_ids, &new_by_id, new_matched)?;
        for &index in &old_indexes {
            old_matched[index] = true;
        }
        for &index in &new_indexes {
            new_matched[index] = true;
        }
        let classification = match relationship.relationship {
            RelationshipType::Successor => Classification::DeclaredSuccessor,
            RelationshipType::Split => Classification::DeclaredSplit,
            RelationshipType::Merge => Classification::DeclaredMerge,
        };
        entries.push(declared_entry(
            classification,
            old_indexes.into_iter().map(|index| old[index].clone()).collect(),
            new_indexes.into_iter().map(|index| new[index].clone()).collect(),
            DeclarationEvidence {
                approved_by: relationship.approved_by.clone(),
                approved_at: relationship
                    .approved_at
                    .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true),
                rationale: relationship.rationale.clone(),
            },
        ));
    }
    Ok(())
}

fn declared_indexes(
    side: &str,
    ids: &[String],
    by_id: &BTreeMap<&str, usize>,
    matched: &[bool],
) -> Result<Vec<usize>, ForgeError> {
    ids.iter()
        .map(|id| {
            let index = by_id.get(id.as_str()).copied().ok_or_else(|| {
                ForgeError::MigrationError(format!(
                    "successor map references {side} identifier '{id}' absent from the inventory"
                ))
            })?;
            if matched[index] {
                return Err(ForgeError::MigrationError(format!(
                    "successor map {side} identifier '{id}' is already reconciled"
                )));
            }
            Ok(index)
        })
        .collect()
}

fn validate_cross_inventory_ids(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    new_by_id: &BTreeMap<&str, usize>,
) -> Result<(), ForgeError> {
    for old_item in old {
        if let Some(&new_index) = new_by_id.get(old_item.stable_id.as_str())
            && old_item.normalized_text != new[new_index].normalized_text
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
    new_by_id: &BTreeMap<&str, usize>,
    old_matched: &mut [bool],
    new_matched: &mut [bool],
    entries: &mut Vec<MigrationEntry>,
) {
    for (old_index, old_item) in old.iter().enumerate() {
        let Some(&new_index) = new_by_id.get(old_item.stable_id.as_str()) else {
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
    let old_groups = group_unmatched_by(old, old_matched, |item| item.normalized_text.as_str());
    let new_groups = group_unmatched_by(new, new_matched, |item| item.normalized_text.as_str());
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
    let mut groups: Vec<CandidateGroup> = Vec::new();
    append_candidate_groups(
        &mut groups,
        group_unmatched_by(old, old_matched, |item| item.normalized_text.as_str()),
        &group_unmatched_by(new, new_matched, |item| item.normalized_text.as_str()),
        EvidenceCode::DuplicateNormalizedText,
    );
    append_candidate_groups(
        &mut groups,
        group_unmatched_by(old, old_matched, locator_key),
        &group_unmatched_by(new, new_matched, locator_key),
        EvidenceCode::CompetingLocator,
    );

    for (old_indexes, new_indexes, evidence) in merge_candidate_groups(groups) {
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
    output: &mut Vec<CandidateGroup>,
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

fn merge_candidate_groups(groups: Vec<CandidateGroup>) -> Vec<CandidateGroup> {
    let mut components = DisjointSet::new(groups.len());
    let mut old_owner = BTreeMap::new();
    let mut new_owner = BTreeMap::new();

    for (group_index, (old_indexes, new_indexes, _)) in groups.iter().enumerate() {
        for &old_index in old_indexes {
            if let Some(previous_group) = old_owner.insert(old_index, group_index) {
                components.union(group_index, previous_group);
            }
        }
        for &new_index in new_indexes {
            if let Some(previous_group) = new_owner.insert(new_index, group_index) {
                components.union(group_index, previous_group);
            }
        }
    }

    let mut merged: BTreeMap<usize, CandidateGroup> = BTreeMap::new();
    for (group_index, (old_indexes, new_indexes, evidence)) in groups.into_iter().enumerate() {
        let root = components.find(group_index);
        let component = merged.entry(root).or_default();
        component.0.extend(old_indexes);
        component.1.extend(new_indexes);
        component.2.extend(evidence);
    }
    merged.into_values().collect()
}

struct DisjointSet {
    parents: Vec<usize>,
    ranks: Vec<u8>,
}

impl DisjointSet {
    fn new(len: usize) -> Self {
        Self { parents: (0..len).collect(), ranks: vec![0; len] }
    }

    fn find(&mut self, mut index: usize) -> usize {
        let mut root = index;
        while self.parents[root] != root {
            root = self.parents[root];
        }
        while self.parents[index] != index {
            let parent = self.parents[index];
            self.parents[index] = root;
            index = parent;
        }
        root
    }

    fn union(&mut self, left: usize, right: usize) {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root == right_root {
            return;
        }
        match self.ranks[left_root].cmp(&self.ranks[right_root]) {
            std::cmp::Ordering::Less => self.parents[left_root] = right_root,
            std::cmp::Ordering::Greater => self.parents[right_root] = left_root,
            std::cmp::Ordering::Equal => {
                self.parents[right_root] = left_root;
                self.ranks[left_root] += 1;
            }
        }
    }
}

fn add_unmatched(
    old: &[InventoryRequirement],
    new: &[InventoryRequirement],
    old_matched: &[bool],
    new_matched: &[bool],
    entries: &mut Vec<MigrationEntry>,
) {
    for item in
        old.iter().enumerate().filter(|(index, _)| !old_matched[*index]).map(|(_, item)| item)
    {
        entries.push(entry(
            Classification::Retired,
            Vec::new(),
            ConfidenceBasis::Unmatched,
            ApprovalStatus::NotRequired,
            vec![item.clone()],
            Vec::new(),
        ));
    }
    for item in
        new.iter().enumerate().filter(|(index, _)| !new_matched[*index]).map(|(_, item)| item)
    {
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

fn group_unmatched_by<'a, K: Ord>(
    items: &'a [InventoryRequirement],
    matched: &[bool],
    key: impl Fn(&'a InventoryRequirement) -> K,
) -> BTreeMap<K, Vec<usize>> {
    let mut groups: BTreeMap<K, Vec<usize>> = BTreeMap::new();
    for (index, item) in items.iter().enumerate().filter(|(index, _)| !matched[*index]) {
        groups.entry(key(item)).or_default().push(index);
    }
    groups
}

fn locator_key(item: &InventoryRequirement) -> (&str, usize, usize) {
    (item.location.section_path.as_str(), item.location.line, item.location.atom_index)
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
    MigrationEntry {
        classification,
        evidence,
        confidence_basis,
        approval_status,
        declaration: None,
        old,
        new,
    }
}

fn declared_entry(
    classification: Classification,
    mut old: Vec<InventoryRequirement>,
    mut new: Vec<InventoryRequirement>,
    declaration: DeclarationEvidence,
) -> MigrationEntry {
    old.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    new.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    MigrationEntry {
        classification,
        evidence: vec![EvidenceCode::ReviewerDeclaration],
        confidence_basis: ConfidenceBasis::Declared,
        approval_status: ApprovalStatus::Declared,
        declaration: Some(declaration),
        old,
        new,
    }
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
        declared_successors: count(Classification::DeclaredSuccessor),
        declared_splits: count(Classification::DeclaredSplit),
        declared_merges: count(Classification::DeclaredMerge),
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
            Classification::DeclaredSuccessor => {
                counts.declared_successors += requirement_count;
            }
            Classification::DeclaredSplit => counts.declared_splits += requirement_count,
            Classification::DeclaredMerge => counts.declared_merges += requirement_count,
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
    let duplicated_old = duplicate_id_samples(&actual_old);
    let duplicated_new = duplicate_id_samples(&actual_new);
    let missing_old: Vec<_> = expected_old.difference(&actual_old_set).take(3).copied().collect();
    let missing_new: Vec<_> = expected_new.difference(&actual_new_set).take(3).copied().collect();
    let old_outcomes = summary.old_requirements.total();
    let new_outcomes = summary.new_requirements.total();
    let summary_error = summary.validate();

    if !duplicated_old.is_empty()
        || !duplicated_new.is_empty()
        || actual_old_set != expected_old
        || actual_new_set != expected_new
        || summary_error.is_err()
    {
        return Err(ForgeError::MigrationError(format!(
            "internal reconciliation invariant failed: duplicated old IDs {duplicated_old:?}; duplicated new IDs {duplicated_new:?}; missing old IDs {missing_old:?}; missing new IDs {missing_new:?}; old outcome count {old_outcomes} vs total_old {}; new outcome count {new_outcomes} vs total_new {}",
            summary.total_old, summary.total_new
        )));
    }
    Ok(())
}

fn duplicate_id_samples<'a>(ids: &[&'a str]) -> Vec<&'a str> {
    let mut counts = BTreeMap::new();
    for id in ids {
        *counts.entry(*id).or_insert(0_usize) += 1;
    }
    counts.into_iter().filter_map(|(id, count)| (count > 1).then_some(id)).take(3).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::successor::{RelationshipType, SuccessorMap, SuccessorRelationship};
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

    fn declaration(
        relationship: RelationshipType,
        old_ids: &[&str],
        new_ids: &[&str],
    ) -> SuccessorRelationship {
        SuccessorRelationship {
            relationship,
            old_ids: old_ids.iter().map(|id| (*id).to_string()).collect(),
            new_ids: new_ids.iter().map(|id| (*id).to_string()).collect(),
            approved_by: "reviewer".to_string(),
            approved_at: chrono::DateTime::parse_from_rfc3339("2026-08-25T12:00:00Z")
                .expect("fixed test timestamp is RFC 3339"),
            rationale: "Reviewed relationship.".to_string(),
        }
    }

    #[test]
    fn duplicate_inventory_stable_ids_are_rejected() {
        let error = classify(
            inventory(vec![
                item("duplicate", "first", "Section", 1, 0),
                item("duplicate", "second", "Section", 2, 0),
            ]),
            inventory(Vec::new()),
            None,
        )
        .expect_err("duplicate stable identifiers must not be collapsed");
        assert!(
            error
                .to_string()
                .contains("old inventory contains duplicate stable identifier 'duplicate'")
        );
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

        let report = classify(old, new, None).unwrap();
        assert_eq!(report.summary.unchanged, 1);
        assert_eq!(report.summary.observed_id_changes, 1);
        assert_eq!(report.summary.substantive_change_candidates, 1);
        assert_eq!(report.summary.retired, 1);
        assert_eq!(report.summary.added, 1);
        assert!(report.has_reviewable_changes());
    }

    #[test]
    fn residual_unique_text_match_after_locator_match_is_classified() {
        let old = inventory(vec![
            item("old-x", "duplicate", "A", 1, 0),
            item("old-y", "other", "B", 2, 0),
        ]);
        let new = inventory(vec![
            item("new-p", "duplicate", "B", 2, 0),
            item("new-q", "duplicate", "A", 1, 0),
        ]);

        let report = classify(old, new, None).unwrap();
        let matched = report
            .entries
            .iter()
            .find(|entry| entry.old.iter().any(|item| item.stable_id == "old-x"))
            .unwrap();

        assert_eq!(matched.classification, Classification::ObservedIdChange);
        assert_eq!(matched.old[0].stable_id, "old-x");
        assert_eq!(matched.new[0].stable_id, "new-q");
        assert!(matched.evidence.contains(&EvidenceCode::UniqueNormalizedText));
    }

    #[test]
    fn encoded_title_and_nested_section_have_distinct_locator_keys() {
        let encoded_title = item("encoded", "text", "Parent/Access Control %2F Audit", 1, 0);
        let nested_section = item("nested", "text", "Parent/Access Control/Audit", 1, 0);

        assert_ne!(locator_key(&encoded_title), locator_key(&nested_section));
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

        let report = classify(old, new, None).unwrap();
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
            None,
        );
        assert!(matches!(result, Err(ForgeError::MigrationError(_))));
    }

    #[test]
    fn identical_inventories_are_clean() {
        let requirements = vec![item("same", "same", "A", 1, 0)];
        let report =
            classify(inventory(requirements.clone()), inventory(requirements), None).unwrap();
        assert!(!report.has_reviewable_changes());
    }

    #[test]
    fn ambiguity_groups_merge_transitively() {
        let groups = vec![
            (
                BTreeSet::from([0]),
                BTreeSet::from([0]),
                BTreeSet::from([EvidenceCode::DuplicateNormalizedText]),
            ),
            (
                BTreeSet::from([1]),
                BTreeSet::from([0]),
                BTreeSet::from([EvidenceCode::CompetingLocator]),
            ),
            (
                BTreeSet::from([1]),
                BTreeSet::from([2]),
                BTreeSet::from([EvidenceCode::DuplicateNormalizedText]),
            ),
        ];

        let merged = merge_candidate_groups(groups);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].0, BTreeSet::from([0, 1]));
        assert_eq!(merged[0].1, BTreeSet::from([0, 2]));
        assert_eq!(
            merged[0].2,
            BTreeSet::from(
                [EvidenceCode::DuplicateNormalizedText, EvidenceCode::CompetingLocator,]
            )
        );
    }

    #[test]
    fn declared_split_and_merge_reconcile_each_requirement_once() {
        let old = inventory(vec![
            item("split-old", "split", "A", 1, 0),
            item("merge-old-a", "merge a", "B", 2, 0),
            item("merge-old-b", "merge b", "C", 3, 0),
        ]);
        let new = inventory(vec![
            item("split-new-a", "split a", "A", 1, 0),
            item("split-new-b", "split b", "A", 1, 1),
            item("merge-new", "merged", "B", 2, 0),
        ]);
        let successor_map = SuccessorMap {
            schema_version: "forge.successor-map/1".to_string(),
            relationships: vec![
                declaration(
                    RelationshipType::Split,
                    &["split-old"],
                    &["split-new-a", "split-new-b"],
                ),
                declaration(
                    RelationshipType::Merge,
                    &["merge-old-a", "merge-old-b"],
                    &["merge-new"],
                ),
            ],
        };
        let report = classify(old, new, Some(&successor_map)).unwrap();
        assert_eq!(report.summary.declared_splits, 1);
        assert_eq!(report.summary.declared_merges, 1);
        assert_eq!(report.summary.old_requirements.total(), 3);
        assert_eq!(report.summary.new_requirements.total(), 3);
        assert!(report.entries.iter().all(|entry| entry.declaration.is_some()));
    }

    #[test]
    fn declaration_with_absent_identifier_is_rejected() {
        let successor_map = SuccessorMap {
            schema_version: "forge.successor-map/1".to_string(),
            relationships: vec![declaration(RelationshipType::Successor, &["missing"], &["new"])],
        };
        let result = classify(
            inventory(vec![item("old", "old", "A", 1, 0)]),
            inventory(vec![item("new", "new", "A", 1, 0)]),
            Some(&successor_map),
        );
        assert!(result.unwrap_err().to_string().contains("absent from the inventory"));
    }

    #[test]
    fn reconciliation_error_identifies_duplicated_requirement() {
        let old_requirement = item("duplicated-old", "text", "A", 1, 0);
        let entries = vec![entry(
            Classification::Retired,
            Vec::new(),
            ConfidenceBasis::Unmatched,
            ApprovalStatus::NotRequired,
            vec![old_requirement.clone(), old_requirement.clone()],
            Vec::new(),
        )];
        let summary = summarize(1, 0, &entries);

        let error =
            validate_reconciliation(&[old_requirement], &[], &entries, &summary).unwrap_err();

        assert!(error.to_string().contains("duplicated-old"));
    }
}
